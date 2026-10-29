// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{PluginRun, core_error, error};
use om9_core::{
    associative::{PortKind, feature_provenance, typed_edges},
    budget::{Limits, Usage},
    geometry::{Curve, Geometry as CoreGeometry, Transform},
    history::{EvaluationState, Recipe},
    identity::{GroupId, HistoryId, LayerId, ObjectId},
    layers::{LayerAction, LayerPlan},
    model::{Attributes, Document, Entity, Extension, Layer, Permissions},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest},
};
use om9_sdk::{
    AngleValue, AttributesPatch, BlockDefinitionInfo, Coordinates, CurveDefinition, CurveInfo,
    CurveSample, CurveSegment, CurveSemantics, Dimension, DocumentInfo, DocumentTolerance,
    DocumentUnits, FeatureDependencyInfo, FeatureInputInfo, FeatureProvenanceInfo,
    FeatureRecordInfo, Geometry, GroupInfo, HostRequest, KnotMode, LayerInfo, LayerUpdate,
    LengthValue, MeasuredAngle, MeasuredArea, MeasuredDistance, ObjectAttributes, ObjectBounds,
    ObjectExtension, ObjectInfo, ObjectPage, ObjectProvenance, ObjectQuery, ObjectSummary,
    ParameterDomain, ParameterValue, PluginMessage, ProjectChange, ProjectInfo, Result, SdkError,
    abi::{self, ReplyFn},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};

/// Bound for one typed numeric field. Longer input is a budget refusal, not a parse attempt.
const MAX_NUMERIC_TEXT: usize = 256;
/// Host default and maximum for one object query page.
const DEFAULT_PAGE: u64 = 256;
const MAX_PAGE: u64 = 1024;
/// Bound on parameter declarations/supplied values checked in one call.
const MAX_PARAMETERS: usize = 128;
/// One command may report at most this many progress/warning messages.
const MAX_MESSAGES: usize = 32;
/// One message is one line of at most this many bytes.
const MAX_MESSAGE_BYTES: usize = 512;
/// Bound on curve samples evaluated in one call.
const MAX_CURVE_SAMPLES: usize = 1024;
/// Bound on curve control points (and half as many knots) returned in one definition.
const MAX_CURVE_CONTROL_POINTS: usize = 8192;
/// Bound on mesh vertices inspected for a bounding box.
const MAX_MESH_BOUNDS: usize = 100_000;

/// The host project snapshot a plugin may read: file path (if any) and dirty state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectState {
    pub path: Option<std::path::PathBuf>,
    pub dirty: bool,
}

/// Which source write a command has already staged for one object. The document
/// transaction allows one source write per object per command, so mixing these
/// kinds would otherwise fail only at commit.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EditKind {
    Placement,
    Attributes,
    Label,
    Layer,
    Extension,
}

/// How a staged selection request changes the current set.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectionMode {
    Replace,
    Add,
    Remove,
    Toggle,
}

pub(super) struct Staged<'a> {
    coordinator: &'a TransactionCoordinator,
    base: &'a Document,
    selection: &'a [ObjectId],
    limits: &'a Limits,
    added: BTreeMap<ObjectId, Entity>,
    deleted: BTreeSet<ObjectId>,
    placements: BTreeMap<ObjectId, Transform>,
    attributes: BTreeMap<ObjectId, Attributes>,
    labels: BTreeMap<ObjectId, String>,
    edits: BTreeMap<ObjectId, EditKind>,
    created_layers: BTreeMap<LayerId, Layer>,
    updated_layers: BTreeMap<LayerId, Layer>,
    /// `Some(None)` clears the current layer; `Some(Some(id))` selects one.
    current_layer: Option<Option<LayerId>>,
    /// Created layer that the command makes current through its creation plan.
    created_current: Option<LayerId>,
    assignments: BTreeMap<ObjectId, LayerId>,
    extensions: BTreeMap<ObjectId, BTreeMap<String, Option<Extension>>>,
    groups_created: Vec<om9_core::model::Group>,
    groups_deleted: BTreeSet<GroupId>,
    tolerance: Option<om9_core::history::Tolerance>,
    staged_selection: Option<BTreeSet<ObjectId>>,
    messages: Vec<PluginMessage>,
    project: ProjectState,
    last_change: Option<ProjectChange>,
    save_requested: bool,
    save_as_requested: bool,
    used: Usage,
    calls: usize,
    failed: Option<SdkError>,
}
impl<'a> Staged<'a> {
    pub(super) fn new(
        coordinator: &'a TransactionCoordinator,
        base: &'a Document,
        selection: &'a [ObjectId],
        limits: &'a Limits,
    ) -> Result<Self> {
        for id in selection {
            if !base.permissions(*id, false).map_err(core_error)?.selectable {
                return Err(error(
                    "selection",
                    "selection contains an unavailable object",
                ));
            }
        }
        Ok(Self {
            coordinator,
            base,
            selection,
            limits,
            added: BTreeMap::new(),
            deleted: BTreeSet::new(),
            placements: BTreeMap::new(),
            attributes: BTreeMap::new(),
            labels: BTreeMap::new(),
            edits: BTreeMap::new(),
            created_layers: BTreeMap::new(),
            updated_layers: BTreeMap::new(),
            current_layer: None,
            created_current: None,
            assignments: BTreeMap::new(),
            extensions: BTreeMap::new(),
            groups_created: Vec::new(),
            groups_deleted: BTreeSet::new(),
            tolerance: None,
            staged_selection: None,
            messages: Vec::new(),
            project: ProjectState::default(),
            last_change: None,
            save_requested: false,
            save_as_requested: false,
            used: Usage::default(),
            calls: 0,
            failed: None,
        })
    }
    /// Attach the host project snapshot the command may read; the app calls this before
    /// the plugin runs. A command cannot change the path or document swap.
    pub fn set_project(&mut self, project: ProjectState) {
        self.project = project;
    }
    /// Attach the change batch recorded for the last plugin command.
    pub fn set_last_change(&mut self, change: Option<ProjectChange>) {
        self.last_change = change;
    }
    fn id(&self, text: &str) -> Result<ObjectId> {
        let id = ObjectId(uuid::Uuid::parse_str(text).map_err(|e| error("id", e))?);
        if id.to_string() != text || id.is_nil() {
            return Err(error("id", "expected canonical nonnil object UUID"));
        }
        Ok(id)
    }
    fn layer_id(&self, text: &str) -> Result<LayerId> {
        let id = LayerId(uuid::Uuid::parse_str(text).map_err(|e| error("layer", e))?);
        if id.to_string() != text || id.is_nil() {
            return Err(error("layer", "expected canonical nonnil layer UUID"));
        }
        Ok(id)
    }
    fn entity(&self, id: ObjectId) -> Result<&Entity> {
        if self.deleted.contains(&id) {
            return Err(error("missing", "object was deleted in this command"));
        }
        self.added
            .get(&id)
            .or_else(|| self.base.entities.get(&id))
            .ok_or_else(|| error("missing", "object"))
    }
    fn editable(&self, id: ObjectId) -> Result<()> {
        let e = self.entity(id)?;
        if !self.merged_permissions(e)?.editable {
            return Err(error("permission", "object or layer is hidden/locked"));
        }
        Ok(())
    }
    /// Staged layer records win over the captured document: a command reads back
    /// the layers it has already created or edited.
    fn merged_layer(&self, id: LayerId) -> Option<&Layer> {
        self.created_layers
            .get(&id)
            .or_else(|| self.updated_layers.get(&id))
            .or_else(|| self.base.layers.get(&id))
    }
    fn merged_layer_permissions(&self, id: Option<LayerId>) -> Result<Permissions> {
        self.base
            .layer_permissions_with(id, false, |id| self.merged_layer(id))
            .map_err(core_error)
    }
    /// Staged attributes and assignments win over the captured document, so a
    /// plugin reads back what its own command has already staged.
    fn merged_attributes(&self, e: &Entity) -> Attributes {
        let mut attributes = self
            .attributes
            .get(&e.id)
            .cloned()
            .unwrap_or_else(|| e.attributes.clone());
        if let Some(layer) = self.assignments.get(&e.id) {
            // The application's assign clears the object colour, so the layer colour applies.
            attributes.layer = Some(*layer);
            attributes.color = None;
        }
        attributes
    }
    fn merged_label(&self, e: &Entity) -> String {
        self.labels
            .get(&e.id)
            .cloned()
            .unwrap_or_else(|| e.label.clone())
    }
    fn merged_permissions(&self, e: &Entity) -> Result<Permissions> {
        self.base
            .permissions_for_attributes_with(&self.merged_attributes(e), false, |id| {
                self.merged_layer(id)
            })
            .map_err(core_error)
    }
    /// One attributes edit is blocked only by the object's layer lock. The
    /// object-level `locked`/`hidden` flags stay editable so a plugin can
    /// unlock or unhide an object, exactly as the property editor does.
    fn layer_editable(&self, attributes: &Attributes) -> Result<()> {
        if !self.merged_layer_permissions(attributes.layer)?.editable {
            return Err(error("permission", "object layer is locked"));
        }
        Ok(())
    }
    /// Record the source write kind for one captured object. A second kind on the
    /// same object is refused here instead of failing at commit with an opaque
    /// conflicting-writes error.
    fn record_edit(&mut self, id: ObjectId, kind: EditKind) -> Result<()> {
        match self.edits.get(&id) {
            Some(previous) if *previous != kind => Err(error(
                "conflict",
                "one command may apply only one of placement, attributes, rename, layer or extension to the same object",
            )),
            _ => {
                self.edits.insert(id, kind);
                Ok(())
            }
        }
    }
    fn staged_attributes_revision(&self, e: &Entity) -> u64 {
        let changed = self.attributes.contains_key(&e.id)
            || self.labels.contains_key(&e.id)
            || self.assignments.contains_key(&e.id);
        if changed && self.base.entities.contains_key(&e.id) {
            e.attributes_revision.saturating_add(1)
        } else {
            e.attributes_revision
        }
    }
    fn object_info(&self, e: &Entity) -> Result<ObjectInfo> {
        let attributes = self.merged_attributes(e);
        let p = self
            .base
            .permissions_for_attributes_with(&attributes, false, |id| self.merged_layer(id))
            .map_err(core_error)?;
        Ok(ObjectInfo {
            id: e.id.to_string(),
            label: self.merged_label(e),
            geometry_kind: e.geometry.kind().tag().into(),
            layer: attributes.layer.map(|id| id.to_string()),
            visible: p.visible,
            selectable: p.selectable,
            editable: p.editable,
            placement: self.placements.get(&e.id).copied().unwrap_or(e.placement).0,
            generation: e.generation,
            geometry_revision: e.geometry_revision,
            attributes_revision: self.staged_attributes_revision(e),
        })
    }
    fn object_attributes(&self, id: ObjectId) -> Result<ObjectAttributes> {
        let e = self.entity(id)?;
        let attributes = self.merged_attributes(e);
        let p = self
            .base
            .permissions_for_attributes_with(&attributes, false, |id| self.merged_layer(id))
            .map_err(core_error)?;
        Ok(ObjectAttributes {
            id: e.id.to_string(),
            label: self.merged_label(e),
            layer: attributes.layer.map(|id| id.to_string()),
            color: attributes.color,
            visible: attributes.visible,
            hidden: attributes.hidden,
            selectable: attributes.selectable,
            locked: attributes.locked,
            effective_visible: p.visible,
            effective_selectable: p.selectable,
            effective_editable: p.editable,
            generation: e.generation,
            geometry_revision: e.geometry_revision,
            attributes_revision: self.staged_attributes_revision(e),
        })
    }
    fn set_attributes(&mut self, id: ObjectId, patch: AttributesPatch) -> Result<ObjectAttributes> {
        let current = {
            let e = self.entity(id)?;
            self.merged_attributes(e)
        };
        // A staged addition is written on the owned entity; a captured object
        // becomes one source write in the command's transaction.
        let added = self.added.contains_key(&id);
        if !added {
            self.layer_editable(&current)?;
            self.record_edit(id, EditKind::Attributes)?;
        }
        let mut next = current.clone();
        if let Some(color) = &patch.color {
            next.color = if color.is_empty() {
                None
            } else {
                Some(parse_color(color)?)
            };
        }
        if let Some(value) = patch.visible {
            next.visible = value;
        }
        if let Some(value) = patch.hidden {
            next.hidden = value;
        }
        if let Some(value) = patch.selectable {
            next.selectable = value;
        }
        if let Some(value) = patch.locked {
            next.locked = value;
        }
        if added {
            if let Some(entity) = self.added.get_mut(&id) {
                entity.attributes = next;
            }
        } else if self
            .base
            .entities
            .get(&id)
            .is_some_and(|e| e.attributes == next)
        {
            // The edit restores the captured value; no mutation is needed.
            self.attributes.remove(&id);
        } else {
            self.attributes.insert(id, next);
        }
        self.object_attributes(id)
    }
    fn rename(&mut self, id: ObjectId, label: String) -> Result<()> {
        if label.is_empty()
            || label.len() > self.limits.max_name_bytes
            || label.trim() != label
            || label.chars().any(char::is_control)
        {
            return Err(error("label", "object label grammar"));
        }
        let added = self.added.contains_key(&id);
        if !added {
            let current = {
                let e = self.entity(id)?;
                self.merged_attributes(e)
            };
            self.layer_editable(&current)?;
            self.record_edit(id, EditKind::Label)?;
        }
        if added {
            if let Some(entity) = self.added.get_mut(&id) {
                entity.label = label;
            }
        } else if self
            .base
            .entities
            .get(&id)
            .is_some_and(|e| e.label == label)
        {
            self.labels.remove(&id);
        } else {
            self.labels.insert(id, label);
        }
        Ok(())
    }
    fn query(&self, query: ObjectQuery) -> Result<ObjectPage> {
        let limit = if query.limit == 0 {
            DEFAULT_PAGE
        } else if query.limit > MAX_PAGE {
            return Err(error("budget", "object query page limit exceeds 1024"));
        } else {
            query.limit
        };
        let layer = match &query.layer {
            Some(text) => Some(self.layer_id(text)?),
            None => None,
        };
        let selected: BTreeSet<ObjectId> = self.selection.iter().copied().collect();
        let mut matched: Vec<&Entity> = Vec::new();
        for e in self.base.entities.values().chain(self.added.values()) {
            if self.deleted.contains(&e.id) {
                continue;
            }
            let attributes = self.merged_attributes(e);
            if let Some(layer) = layer
                && attributes.layer != Some(layer)
            {
                continue;
            }
            if let Some(kind) = &query.geometry_kind
                && e.geometry.kind().tag() != kind.as_str()
            {
                continue;
            }
            if let Some(text) = &query.name_contains
                && !self.merged_label(e).contains(text.as_str())
            {
                continue;
            }
            if query.only_selected && !selected.contains(&e.id) {
                continue;
            }
            if !query.include_hidden && (!attributes.visible || attributes.hidden) {
                continue;
            }
            matched.push(e);
        }
        let total = matched.len() as u64;
        let start = usize::try_from(query.offset)
            .unwrap_or(usize::MAX)
            .min(matched.len());
        let end = start.saturating_add(limit as usize).min(matched.len());
        let mut objects = Vec::with_capacity(end - start);
        for e in &matched[start..end] {
            objects.push(self.object_info(e)?);
        }
        Ok(ObjectPage {
            total,
            offset: query.offset,
            objects,
        })
    }
    /// Stage one owned output on the current layer; shared by every add_* service.
    fn add_output(&mut self, label: String, geometry: CoreGeometry) -> Result<String> {
        if label.len() > self.limits.max_name_bytes {
            return Err(error("budget", "object label"));
        }
        let current_layer = self.current_layer();
        // A layer created by this command can receive output: the document transaction
        // evaluates the proposed layer record. A layer whose flags change in this
        // transaction still cannot, because a permission read must never be based on
        // captured state this transaction rewrites.
        if let Some(id) = current_layer
            && self.updated_layers.contains_key(&id)
        {
            return Err(error(
                "layer",
                "the current layer's flags change in this command; add objects before editing it or in the next command",
            ));
        }
        if !self.merged_layer_permissions(current_layer)?.editable {
            return Err(error("permission", "current layer is locked"));
        }
        let usage = geometry.validate(self.limits).map_err(core_error)?;
        self.used.add(usage, self.limits).map_err(core_error)?;
        self.used
            .add(
                Usage {
                    objects: 1,
                    nodes: 1,
                    bytes: std::mem::size_of::<Entity>() as u64 + label.capacity() as u64,
                    ..Usage::default()
                },
                self.limits,
            )
            .map_err(core_error)?;
        if self.added.len() as u64 >= self.limits.max_outputs {
            return Err(error("budget", "plugin output count"));
        }
        let entity = Entity::new(label, geometry, current_layer);
        let id = entity.id;
        self.added.insert(id, entity);
        Ok(id.to_string())
    }
    fn extension_slot(slot: &str) -> Result<()> {
        if slot.is_empty() || slot.len() > 64 || slot.bytes().any(|byte| byte == 0) {
            return Err(error("extension", "extension slot name"));
        }
        Ok(())
    }
    fn set_extension(
        &mut self,
        id: ObjectId,
        slot: String,
        extension: Option<Extension>,
    ) -> Result<()> {
        Self::extension_slot(&slot)?;
        // An object created by this command is still staged: its extension is part of
        // the inserted record, not a separate mutation against a base stamp.
        if self.added.contains_key(&id) {
            return Err(error(
                "extension",
                "object was created in this command; tag it in the next command",
            ));
        }
        self.entity(id)?;
        self.record_edit(id, EditKind::Extension)?;
        self.extensions
            .entry(id)
            .or_default()
            .insert(slot, extension);
        Ok(())
    }
    fn object_extensions(&self, id: ObjectId) -> Result<BTreeMap<String, ObjectExtension>> {
        let entity = self.entity(id)?;
        let mut slots = entity
            .extensions
            .iter()
            .map(|(slot, extension)| {
                (
                    slot.clone(),
                    ObjectExtension {
                        version: extension.version,
                        payload: extension.payload.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        if let Some(staged) = self.extensions.get(&id) {
            for (slot, extension) in staged {
                match extension {
                    Some(extension) => {
                        slots.insert(
                            slot.clone(),
                            ObjectExtension {
                                version: extension.version,
                                payload: extension.payload.clone(),
                            },
                        );
                    }
                    None => {
                        slots.remove(slot);
                    }
                }
            }
        }
        Ok(slots)
    }
    fn layer_info(&self, id: LayerId) -> Result<LayerInfo> {
        let layer = self
            .merged_layer(id)
            .ok_or_else(|| error("layer", "unknown layer"))?;
        let permissions = self.merged_layer_permissions(Some(id))?;
        Ok(LayerInfo {
            id: id.to_string(),
            name: layer.name.clone(),
            visible: permissions.visible,
            locked: !permissions.editable,
            selectable: permissions.selectable,
            current: self.current_layer() == Some(id),
            color: layer.color,
        })
    }
    fn curve(&self, id: ObjectId) -> Result<&Curve> {
        match &self.entity(id)?.geometry {
            CoreGeometry::Curve(curve) => Ok(curve),
            _ => Err(error("unsupported", "object geometry is not a curve")),
        }
    }
    fn curve_info(&self, id: ObjectId) -> Result<CurveInfo> {
        let entity = self.entity(id)?;
        let mut info = CurveInfo {
            id: entity.id.to_string(),
            kind: String::new(),
            domain: None,
            degree: None,
            poles: None,
            periodic: None,
            reversed: false,
            segments: 0,
        };
        match self.curve(id)? {
            Curve::Line { domain, .. } => {
                info.kind = "line".into();
                info.domain = Some(*domain);
            }
            Curve::Circle { domain, .. } => {
                info.kind = "circle".into();
                info.domain = Some(*domain);
            }
            Curve::Nurbs(curve) => {
                info.kind = "nurbs".into();
                info.domain = Some(curve.domain);
                info.degree = Some(curve.degree as u64);
                info.poles = Some(curve.poles.len() as u64);
                info.periodic = Some(curve.periodic);
                info.reversed = curve.reversed;
            }
            Curve::Polycurve { segments, reversed } => {
                info.kind = "polycurve".into();
                info.reversed = *reversed;
                info.segments = segments.len() as u64;
            }
        }
        Ok(info)
    }
    fn evaluate_curve_samples(
        &self,
        id: ObjectId,
        parameters: Vec<f64>,
        order: u8,
    ) -> Result<Vec<CurveSample>> {
        if order > 3 {
            return Err(error("unsupported", "derivative order above D3"));
        }
        if parameters.len() > MAX_CURVE_SAMPLES {
            return Err(error("budget", "curve sample count"));
        }
        let curve = self.curve(id)?;
        if matches!(curve, Curve::Polycurve { .. }) {
            return Err(error(
                "unsupported",
                "polycurve requires an explicit segment parameter",
            ));
        }
        self.evaluate_samples(curve, parameters, order)
    }
    fn evaluate_samples(
        &self,
        curve: &Curve,
        parameters: Vec<f64>,
        order: u8,
    ) -> Result<Vec<CurveSample>> {
        let mut samples = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            let mut values = om9_core::evaluation::evaluate_curve(
                curve,
                om9_core::evaluation::Parameter::Native(parameter),
                order as usize,
                Transform::identity(),
                self.limits,
            )
            .map_err(curve_error)?;
            let point = values
                .first()
                .copied()
                .ok_or_else(|| error("curve", "evaluation returned no point"))?;
            if !values.is_empty() {
                values.remove(0);
            }
            samples.push(CurveSample {
                parameter,
                point,
                derivatives: values,
            });
        }
        Ok(samples)
    }
    fn segment_info(&self, segment: &Curve, reversed: bool) -> Result<CurveSegment> {
        let (kind, domain, angle_domain) = match segment {
            Curve::Line { domain, .. } => ("line", Some(*domain), None),
            Curve::Circle {
                domain,
                angle_domain,
                ..
            } => ("circle", Some(*domain), *angle_domain),
            Curve::Nurbs(curve) => ("nurbs", Some(curve.domain), None),
            Curve::Polycurve { .. } => {
                return Err(error("unsupported", "nested polycurve segment"));
            }
        };
        Ok(CurveSegment {
            kind: kind.into(),
            domain,
            angle_domain,
            reversed,
        })
    }
    fn curve_segments_of(&self, id: ObjectId) -> Result<Vec<CurveSegment>> {
        let curve = self.curve(id)?;
        match curve {
            Curve::Polycurve { segments, reversed } => {
                if segments.len() > MAX_CURVE_SAMPLES {
                    return Err(error("budget", "curve segment count"));
                }
                segments
                    .iter()
                    .map(|segment| self.segment_info(segment, *reversed))
                    .collect()
            }
            other => Ok(vec![self.segment_info(other, false)?]),
        }
    }
    fn evaluate_curve_segment(
        &self,
        id: ObjectId,
        index: u32,
        parameters: Vec<f64>,
        order: u8,
    ) -> Result<Vec<CurveSample>> {
        if order > 3 {
            return Err(error("unsupported", "derivative order above D3"));
        }
        if parameters.len() > MAX_CURVE_SAMPLES {
            return Err(error("budget", "curve sample count"));
        }
        let segment = match self.curve(id)? {
            Curve::Polycurve { segments, .. } => segments
                .get(index as usize)
                .ok_or_else(|| error("curve", "segment index"))?,
            curve => {
                if index != 0 {
                    return Err(error("curve", "segment index"));
                }
                curve
            }
        };
        if matches!(segment, Curve::Polycurve { .. }) {
            return Err(error("unsupported", "nested polycurve segment"));
        }
        self.evaluate_samples(segment, parameters, order)
    }
    fn curve_definition(&self, id: ObjectId) -> Result<CurveDefinition> {
        let entity = self.entity(id)?;
        let curve = self.curve(id)?;
        let mut definition = CurveDefinition {
            id: entity.id.to_string(),
            kind: String::new(),
            domain: None,
            angle_domain: None,
            degree: None,
            poles: None,
            weights: None,
            knots: None,
            periodic: None,
            reversed: false,
            seam: None,
            segment_count: 0,
        };
        match curve {
            Curve::Line { domain, .. } => {
                definition.kind = "line".into();
                definition.domain = Some(*domain);
            }
            Curve::Circle {
                domain,
                angle_domain,
                ..
            } => {
                definition.kind = "circle".into();
                definition.domain = Some(*domain);
                definition.angle_domain = *angle_domain;
            }
            Curve::Nurbs(curve) => {
                if curve.poles.len() > MAX_CURVE_CONTROL_POINTS
                    || curve.knots.len() > MAX_CURVE_CONTROL_POINTS * 2
                {
                    return Err(error("budget", "curve control data"));
                }
                definition.kind = "nurbs".into();
                definition.domain = Some(curve.domain);
                definition.degree = Some(curve.degree as u64);
                definition.poles = Some(curve.poles.clone());
                definition.weights = Some(curve.weights.clone());
                definition.knots = Some(curve.knots.clone());
                definition.periodic = Some(curve.periodic);
                definition.reversed = curve.reversed;
                definition.seam = curve.seam;
            }
            Curve::Polycurve { segments, reversed } => {
                definition.kind = "polycurve".into();
                definition.reversed = *reversed;
                definition.segment_count = segments.len() as u64;
            }
        }
        Ok(definition)
    }
    fn curve_semantics_of(
        &self,
        id: ObjectId,
        tolerance_mm: Option<f64>,
    ) -> Result<CurveSemantics> {
        let curve = self.curve(id)?;
        let tolerance = match tolerance_mm {
            Some(tolerance) => tolerance,
            None => {
                self.base
                    .effective_tolerance()
                    .map_err(tolerance_error)?
                    .absolute_mm
            }
        };
        let measured = om9_core::rhino_semantics::classify_curve(curve, tolerance, self.limits)
            .map_err(curve_error)?;
        let kind = match measured.form {
            om9_core::rhino_semantics::CurveForm::Line => "line",
            om9_core::rhino_semantics::CurveForm::Circle => "circle",
            om9_core::rhino_semantics::CurveForm::Polyline => "polyline",
            om9_core::rhino_semantics::CurveForm::Polycurve => "polycurve",
            om9_core::rhino_semantics::CurveForm::Nurbs => "nurbs",
        };
        Ok(CurveSemantics {
            id: id.to_string(),
            kind: kind.into(),
            closed: measured.closed,
            planar: measured.planar,
            endpoints: measured.endpoints,
            tolerance_mm: measured.tolerance,
        })
    }
    /// The command's own selection when it staged one, otherwise the captured selection,
    /// with objects deleted by this command removed.
    fn current_selection(&self) -> Vec<ObjectId> {
        match &self.staged_selection {
            Some(selection) => selection
                .iter()
                .copied()
                .filter(|id| !self.deleted.contains(id))
                .collect(),
            None => self
                .selection
                .iter()
                .copied()
                .filter(|id| !self.deleted.contains(id))
                .collect(),
        }
    }
    fn stage_selection(&mut self, ids: Vec<String>, mode: SelectionMode) -> Result<()> {
        let mut next: BTreeSet<ObjectId> = if mode == SelectionMode::Replace {
            BTreeSet::new()
        } else {
            self.staged_selection
                .clone()
                .unwrap_or_else(|| self.selection.iter().copied().collect())
        };
        next.retain(|id| !self.deleted.contains(id));
        for text in ids {
            let id = self.id(&text)?;
            if mode == SelectionMode::Remove
                || (mode == SelectionMode::Toggle && next.contains(&id))
            {
                // Leaving the set never needs the object to still be selectable.
                next.remove(&id);
                continue;
            }
            if self.deleted.contains(&id) {
                return Err(error("selection", "object was deleted in this command"));
            }
            let entity = self.entity(id)?;
            if !self.merged_permissions(entity)?.selectable {
                return Err(error(
                    "permission",
                    "selection contains an unavailable object",
                ));
            }
            next.insert(id);
        }
        if next.len() as u64 > self.limits.max_outputs {
            return Err(error("budget", "selection count"));
        }
        self.staged_selection = Some(next);
        Ok(())
    }
    fn current_layer(&self) -> Option<LayerId> {
        if let Some(created) = self.created_current {
            return Some(created);
        }
        self.current_layer.unwrap_or(self.base.current_layer)
    }
    fn layer_create(&mut self, name: String, color: [u8; 4], make_current: bool) -> Result<String> {
        if name.is_empty()
            || name.len() > self.limits.max_name_bytes
            || name.trim() != name
            || name.chars().any(char::is_control)
        {
            return Err(error("label", "layer name grammar"));
        }
        // Name uniqueness and the layer budget belong to the document's own layer
        // admission; they are checked when the command prepares.
        let layer = Layer::new(name, color);
        let id = layer.id;
        if make_current {
            self.set_created_current(id);
        }
        self.created_layers.insert(id, layer);
        Ok(id.to_string())
    }
    /// Only one creation plan may set the current layer; a later choice replaces it.
    fn set_created_current(&mut self, id: LayerId) {
        self.created_current = Some(id);
        self.current_layer = None;
    }
    fn layer_update(&mut self, id: LayerId, update: LayerUpdate) -> Result<()> {
        let (mut layer, created) = if let Some(layer) = self.created_layers.get(&id) {
            (layer.clone(), true)
        } else if let Some(layer) = self.updated_layers.get(&id) {
            (layer.clone(), false)
        } else {
            let layer = self
                .base
                .layers
                .get(&id)
                .ok_or_else(|| error("layer", "unknown layer"))?;
            if !self.merged_layer_permissions(Some(id))?.editable {
                return Err(error("permission", "layer is locked"));
            }
            (layer.clone(), false)
        };
        if let Some(name) = update.name {
            layer.name = name;
        }
        if let Some(color) = update.color {
            layer.color = color;
        }
        if let Some(value) = update.visible {
            layer.visible = value;
        }
        if let Some(value) = update.selectable {
            layer.selectable = value;
        }
        if let Some(value) = update.locked {
            layer.locked = value;
        }
        if created {
            self.created_layers.insert(id, layer);
        } else {
            self.updated_layers.insert(id, layer);
        }
        Ok(())
    }
    fn set_current_layer(&mut self, id: Option<LayerId>) -> Result<()> {
        if let Some(id) = id {
            if self.merged_layer(id).is_none() {
                return Err(error("layer", "unknown layer"));
            }
            if self.updated_layers.contains_key(&id) {
                return Err(error(
                    "layer",
                    "the layer's flags change in this command; select it current before editing it or in the next command",
                ));
            }
            if self.created_layers.contains_key(&id) {
                // A layer created by this command becomes current through its
                // creation plan; a separate SetCurrent cannot resolve it yet.
                self.set_created_current(id);
                return Ok(());
            }
        }
        self.created_current = None;
        self.current_layer = Some(id);
        Ok(())
    }
    fn assign_layer(&mut self, ids: Vec<String>, layer: LayerId) -> Result<()> {
        if self.merged_layer(layer).is_none() {
            return Err(error("layer", "unknown layer"));
        }
        if self.updated_layers.contains_key(&layer) {
            return Err(error(
                "layer",
                "the target layer's flags change in this command; assign before editing it or in the next command",
            ));
        }
        if ids.is_empty() {
            return Ok(());
        }
        if ids.len() as u64 > self.limits.max_outputs {
            return Err(error("budget", "layer assignment count"));
        }
        for text in ids {
            let id = self.id(&text)?;
            let current = {
                let e = self.entity(id)?;
                self.merged_attributes(e)
            };
            if self.added.contains_key(&id) {
                if let Some(entity) = self.added.get_mut(&id) {
                    entity.attributes.layer = Some(layer);
                    entity.attributes.color = None;
                }
                continue;
            }
            if let Some(source_layer) = current.layer
                && self.updated_layers.contains_key(&source_layer)
            {
                return Err(error(
                    "layer",
                    "the source layer's flags change in this command; assign before editing it or in the next command",
                ));
            }
            // The source side of the move is protected too: an object on a locked
            // layer must not be moved off it.
            self.layer_editable(&current)?;
            self.record_edit(id, EditKind::Layer)?;
            self.assignments.insert(id, layer);
        }
        Ok(())
    }
    fn history_id(&self, text: &str) -> Result<HistoryId> {
        let id = HistoryId(uuid::Uuid::parse_str(text).map_err(|e| error("id", e))?);
        if id.to_string() != text || id.is_nil() {
            return Err(error("id", "expected canonical nonnil history UUID"));
        }
        Ok(id)
    }
    fn history_record(&self, id: HistoryId) -> Result<FeatureRecordInfo> {
        self.base
            .history
            .recipes
            .get(&id)
            .ok_or_else(|| error("missing", "history feature"))
            .and_then(history_record_info)
    }
    fn history_records(&self) -> Result<Vec<FeatureRecordInfo>> {
        let order = self.base.history.execution_order().map_err(core_error)?;
        order
            .into_iter()
            .map(|id| self.history_record(id))
            .collect()
    }
    fn handle(&mut self, request: HostRequest) -> Result<Value> {
        self.calls += 1;
        if self.calls > 4096 {
            return Err(error("budget", "too many host calls in one command"));
        }
        match request {
            HostRequest::Capabilities => json(&vec![
                "document_info",
                "document_units",
                "objects_list",
                "objects_query",
                "object_attributes",
                "object_set_attributes",
                "object_rename",
                "selection_get",
                "selection_set",
                "selection_add",
                "selection_remove",
                "selection_toggle",
                "layers_list",
                "layer_create",
                "layer_update",
                "set_current_layer",
                "assign_to_layer",
                "object_geometry",
                "object_add",
                "object_delete",
                "object_transform",
                "parse_length",
                "parse_angle",
                "parse_coordinates",
                "validate_parameter",
                "apply_parameters",
                "progress",
                "warning",
                "add_mesh",
                "add_circle_on_frame",
                "set_object_placement",
                "object_extensions",
                "set_object_extension",
                "clear_object_extension",
                "last_change",
                "last_change",
                "project_state",
                "save_project",
                "save_project_as",
                "save_project_as",
                "curve_info",
                "curve_evaluate",
                "add_arc",
                "add_interpolated_curve",
                "add_nurbs_curve",
                "add_ellipse",
                "add_rectangle",
                "add_polygon",
                "measure_distance",
                "measure_angle",
                "create_group",
                "object_info",
                "object_placement",
                "clear_current_layer",
                "block_definitions",
                "groups",
                "measure_mesh_area",
                "layer_order",
                "report_measurement",
                "object_summary",
                "measure_curve_radius",
                "measure_curve_length",
                "object_bounds",
                "curve_segments",
                "curve_evaluate_segment",
                "curve_definition",
                "curve_semantics",
                "history_list",
                "history_feature",
                "history_dependencies",
                "history_provenance",
                "history_rebuild_plan",
            ]),
            HostRequest::DocumentInfo => json(&DocumentInfo {
                id: self.base.id.to_string(),
                name: self.base.name.clone(),
                revision: self.base.revision,
                current_layer: self.current_layer().map(|id| id.to_string()),
                unit: self.base.page_unit.symbol().into(),
            }),
            HostRequest::DocumentUnits => json(&DocumentUnits {
                unit: self.base.page_unit.symbol().into(),
                millimeters_per_unit: self.base.page_unit.factor(),
                // A document that states no tolerance reports `None`; no default is invented.
                tolerance: self.base.tolerance().map(|tolerance| DocumentTolerance {
                    absolute_mm: tolerance.absolute_mm,
                    relative: tolerance.relative,
                    angular_rad: tolerance.angular_rad,
                }),
            }),
            HostRequest::ParseLength { text } => {
                // One typed parser shared with the document's own unit contract.
                let millimeters =
                    om9_core::units::LengthMm::parse(numeric_text(&text)?, self.base.page_unit)
                        .map_err(unit_error)?;
                json(&LengthValue {
                    millimeters: millimeters.mm(),
                })
            }
            HostRequest::ParseAngle { text } => {
                let degrees =
                    om9_core::units::AngleDeg::parse(numeric_text(&text)?).map_err(unit_error)?;
                json(&AngleValue {
                    degrees: degrees.deg(),
                    radians: degrees.to_radians().rad(),
                })
            }
            HostRequest::ParseCoordinates { text } => {
                // The command prompt's grammar and error labels, not a second parser.
                let point =
                    om9_core::coords::parse_local(numeric_text(&text)?, self.base.page_unit)
                        .map_err(|refusal| error("coordinate", refusal))?;
                json(&Coordinates {
                    millimeters: point.xyz(),
                })
            }
            HostRequest::ObjectsQuery { query } => json(&self.query(query)?),
            HostRequest::ObjectAttributes { id } => {
                let id = self.id(&id)?;
                json(&self.object_attributes(id)?)
            }
            HostRequest::ObjectSetAttributes { id, patch } => {
                let id = self.id(&id)?;
                json(&self.set_attributes(id, patch)?)
            }
            HostRequest::ObjectRename { id, label } => {
                let id = self.id(&id)?;
                self.rename(id, label)?;
                json(&())
            }
            HostRequest::SelectionGet => json(
                &self
                    .current_selection()
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>(),
            ),
            HostRequest::SelectionSet { ids } => {
                self.stage_selection(ids, SelectionMode::Replace)?;
                json(&())
            }
            HostRequest::SelectionAdd { ids } => {
                self.stage_selection(ids, SelectionMode::Add)?;
                json(&())
            }
            HostRequest::SelectionRemove { ids } => {
                self.stage_selection(ids, SelectionMode::Remove)?;
                json(&())
            }
            HostRequest::SelectionToggle { ids } => {
                self.stage_selection(ids, SelectionMode::Toggle)?;
                json(&())
            }
            HostRequest::LayersList => {
                if self
                    .base
                    .layers
                    .len()
                    .saturating_add(self.created_layers.len())
                    > abi::MAX_MESSAGE_BYTES / 512
                {
                    return Err(error("budget", "layer query needs pagination"));
                }
                let mut ids: Vec<LayerId> = self.base.layer_order.clone();
                for id in self.created_layers.keys() {
                    if !self.base.layers.contains_key(id) {
                        ids.push(*id);
                    }
                }
                let mut layers = Vec::new();
                for id in ids {
                    layers.push(self.layer_info(id)?);
                }
                json(&layers)
            }
            HostRequest::ObjectsList => {
                if self.base.entities.len().saturating_add(self.added.len())
                    > abi::MAX_MESSAGE_BYTES / 512
                {
                    return Err(error("budget", "object query needs pagination"));
                }
                let mut objects = Vec::new();
                for e in self.base.entities.values().chain(self.added.values()) {
                    if !self.deleted.contains(&e.id) {
                        objects.push(self.object_info(e)?);
                    }
                }
                json(&objects)
            }
            HostRequest::ObjectGeometry { id } => {
                json(&from_core(&self.entity(self.id(&id)?)?.geometry)?)
            }
            HostRequest::ObjectAdd { label, geometry } => {
                let geometry = to_core(geometry, self.limits)?;
                json(&self.add_output(label, geometry)?)
            }
            HostRequest::ObjectAddArc {
                label,
                start,
                through,
                end,
            } => {
                let curve =
                    om9_core::curve_build::arc_through(start, through, end).map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(curve))?)
            }
            HostRequest::ObjectAddEllipse {
                label,
                origin,
                x,
                y,
                radius_x,
                radius_y,
            } => {
                let curve =
                    om9_core::curve_build::ellipse(origin, x, y, radius_x, radius_y, self.limits)
                        .map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(Curve::Nurbs(curve)))?)
            }
            HostRequest::ObjectAddRectangle {
                label,
                origin,
                x,
                y,
                width,
                height,
            } => {
                let curve = om9_core::curve_build::rectangle(origin, x, y, width, height)
                    .map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(curve))?)
            }
            HostRequest::ObjectAddPolygon { label, points } => {
                let mut closed = points;
                if closed.len() >= 2 && closed.first() == closed.last() {
                    closed.pop();
                }
                if closed.len() < 3 {
                    return Err(error(
                        "curve",
                        "polygon needs at least three distinct points",
                    ));
                }
                closed.push(closed[0]);
                let curve = om9_core::curve_build::polyline(&closed).map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(curve))?)
            }
            HostRequest::MeasureDistance { first, second } => {
                let millimeters =
                    om9_core::measure::distance_mm(first, second).map_err(measure_error)?;
                json(&MeasuredDistance { millimeters })
            }
            HostRequest::MeasureAngle {
                from_a,
                to_a,
                from_b,
                to_b,
            } => {
                let degrees = om9_core::measure::angle_degrees(from_a, to_a, from_b, to_b)
                    .map_err(measure_error)?;
                json(&MeasuredAngle { degrees })
            }
            HostRequest::ObjectAddNurbsCurve {
                label,
                degree,
                poles,
                weights,
                knots,
            } => {
                if poles.len() > MAX_CURVE_CONTROL_POINTS
                    || knots.len() > MAX_CURVE_CONTROL_POINTS * 2
                {
                    return Err(error("budget", "curve control data"));
                }
                let degree = usize::try_from(degree)
                    .map_err(|_| error("curve", "NURBS degree exceeds platform size"))?;
                if degree == 0 || poles.len() <= degree || knots.len() <= poles.len() {
                    return Err(error("curve", "NURBS degree/pole/knot counts"));
                }
                if weights.len() != poles.len() {
                    return Err(error("curve", "NURBS weight count"));
                }
                let nurbs = om9_core::geometry::NurbsCurve {
                    degree,
                    domain: [knots[degree], knots[poles.len()]],
                    periodic: false,
                    reversed: false,
                    seam: None,
                    poles,
                    weights,
                    knots,
                };
                nurbs.validate(self.limits).map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(Curve::Nurbs(nurbs)))?)
            }
            HostRequest::ObjectAddInterpolatedCurve {
                label,
                points,
                degree,
                mode,
            } => {
                let mode = match mode {
                    KnotMode::ChordLength => om9_core::curve_build::KnotMode::ChordLength,
                    KnotMode::Uniform => om9_core::curve_build::KnotMode::Uniform,
                };
                let curve =
                    om9_core::curve_build::interpolate(&points, degree as usize, mode, self.limits)
                        .map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(Curve::Nurbs(curve)))?)
            }
            HostRequest::LayerCreate {
                name,
                color,
                make_current,
            } => json(&self.layer_create(name, color, make_current)?),
            HostRequest::LayerUpdate { id, update } => {
                let id = self.layer_id(&id)?;
                self.layer_update(id, update)?;
                json(&())
            }
            HostRequest::SetCurrentLayer { id } => {
                let id = match id {
                    Some(text) => Some(self.layer_id(&text)?),
                    None => None,
                };
                self.set_current_layer(id)?;
                json(&())
            }
            HostRequest::AssignToLayer { ids, layer } => {
                let layer = self.layer_id(&layer)?;
                self.assign_layer(ids, layer)?;
                json(&())
            }
            HostRequest::ValidateParameter { spec, value } => {
                if spec.name.is_empty() || spec.name.len() > self.limits.max_name_bytes {
                    return Err(error("parameter", "parameter name"));
                }
                let domain = core_domain(spec.domain)?;
                let typed: ParameterValue =
                    serde_json::from_value(value).map_err(|refusal| error("parameter", refusal))?;
                let core_value = core_parameter(typed);
                domain
                    .check(&core_value, self.limits)
                    .map_err(parameter_error)?;
                json(&sdk_parameter(&core_value)?)
            }
            HostRequest::ApplyParameters { schema, supplied } => {
                if schema.len() > MAX_PARAMETERS || supplied.len() > MAX_PARAMETERS {
                    return Err(error("budget", "too many parameters in one call"));
                }
                // Same typed schema and fill/check rule the command contract uses.
                let mut command = om9_core::command_schema::CommandSchema::new("PluginParameters");
                for spec in schema {
                    command = command.with_option(om9_core::command_schema::OptionSpec::new(
                        spec.name,
                        core_domain(spec.domain)?,
                        spec.default.map(core_parameter),
                        spec.required,
                    ));
                }
                let mut options = om9_core::request::Options::new();
                for (name, value) in supplied {
                    let typed: ParameterValue = serde_json::from_value(value)
                        .map_err(|refusal| error("parameter", refusal))?;
                    options.insert(name, core_parameter(typed));
                }
                let applied = command
                    .apply(options, self.limits)
                    .map_err(parameter_error)?;
                let mut typed = BTreeMap::new();
                for (name, value) in applied {
                    typed.insert(name, sdk_parameter(&value)?);
                }
                json(&typed)
            }
            HostRequest::ObjectAddMesh {
                label,
                vertices,
                faces,
            } => {
                let mesh = om9_core::geometry::Mesh {
                    vertices,
                    faces,
                    normals: Vec::new(),
                    colors: Vec::new(),
                    corner_uvs: Vec::new(),
                };
                mesh.validate(self.limits).map_err(mesh_error)?;
                json(&self.add_output(label, CoreGeometry::Mesh(mesh))?)
            }
            HostRequest::ObjectAddCircleFrame {
                label,
                center,
                x,
                y,
                radius,
            } => {
                let curve = Curve::Circle {
                    center,
                    x,
                    y,
                    radius,
                    domain: [0., std::f64::consts::TAU],
                    angle_domain: None,
                };
                curve.validate(self.limits).map_err(curve_error)?;
                json(&self.add_output(label, CoreGeometry::Curve(curve))?)
            }
            HostRequest::ObjectSetPlacement { id, matrix } => {
                let id = self.id(&id)?;
                if self.added.contains_key(&id) {
                    return Err(error(
                        "placement",
                        "object was created in this command; its insert already carries the placement",
                    ));
                }
                self.editable(id)?;
                let placement = Transform(matrix);
                placement.validate().map_err(placement_error)?;
                self.record_edit(id, EditKind::Placement)?;
                self.placements.insert(id, placement);
                json(&())
            }
            HostRequest::ObjectExtensions { id } => {
                let id = self.id(&id)?;
                json(&self.object_extensions(id)?)
            }
            HostRequest::ObjectSetExtension {
                id,
                slot,
                version,
                payload,
            } => {
                let id = self.id(&id)?;
                let extension = Extension::new(version, payload).map_err(extension_error)?;
                self.set_extension(id, slot, Some(extension))?;
                json(&())
            }
            HostRequest::ObjectClearExtension { id, slot } => {
                let id = self.id(&id)?;
                self.set_extension(id, slot, None)?;
                json(&())
            }
            HostRequest::LastChange => json(&self.last_change),
            HostRequest::ProjectState => json(&ProjectInfo {
                path: self
                    .project
                    .path
                    .as_ref()
                    .map(|path| path.to_string_lossy().into_owned()),
                dirty: self.project.dirty,
            }),
            HostRequest::SaveProject => {
                if self.project.path.is_none() {
                    return Err(error(
                        "save",
                        "project has no path; save it in the host first",
                    ));
                }
                if self.save_as_requested {
                    return Err(error("save", "save and save-as cannot be combined"));
                }
                self.save_requested = true;
                json(&())
            }
            HostRequest::SaveProjectAs => {
                if self.save_requested {
                    return Err(error("save", "save and save-as cannot be combined"));
                }
                self.save_as_requested = true;
                json(&())
            }
            HostRequest::CurveInfo { id } => {
                let id = self.id(&id)?;
                json(&self.curve_info(id)?)
            }
            HostRequest::MeasureMeshArea { id } => {
                let id = self.id(&id)?;
                let entity = self.entity(id)?;
                let CoreGeometry::Mesh(mesh) = &entity.geometry else {
                    return Err(error("unsupported", "area needs a mesh"));
                };
                let mut square_millimeters = 0.0;
                for face in &mesh.faces {
                    let Some(first) = face.first() else { continue };
                    let origin = mesh.vertices[*first as usize];
                    for pair in face[1..].windows(2) {
                        let a = mesh.vertices[pair[0] as usize];
                        let b = mesh.vertices[pair[1] as usize];
                        let u = [a[0] - origin[0], a[1] - origin[1], a[2] - origin[2]];
                        let v = [b[0] - origin[0], b[1] - origin[1], b[2] - origin[2]];
                        let cross = [
                            u[1] * v[2] - u[2] * v[1],
                            u[2] * v[0] - u[0] * v[2],
                            u[0] * v[1] - u[1] * v[0],
                        ];
                        square_millimeters += cross[0].hypot(cross[1]).hypot(cross[2]) / 2.0;
                    }
                }
                json(&MeasuredArea { square_millimeters })
            }
            HostRequest::DefinitionsList => {
                let definitions = self
                    .base
                    .definitions
                    .values()
                    .map(|definition| BlockDefinitionInfo {
                        id: definition.id.to_string(),
                        label: definition.label.clone(),
                        members: definition.members.len() as u64,
                        revision: definition.revision,
                    })
                    .collect::<Vec<_>>();
                json(&definitions)
            }
            HostRequest::GroupDelete { id } => {
                let group = GroupId(uuid::Uuid::parse_str(&id).map_err(|e| error("id", e))?);
                let exists = self.base.groups.contains_key(&group)
                    || self.groups_created.iter().any(|g| g.id == group);
                if !exists {
                    return Err(error("missing", "group"));
                }
                self.groups_deleted.insert(group);
                json(&())
            }
            HostRequest::GroupCreate { name, members } => {
                if name.is_empty() || name.len() > self.limits.max_name_bytes {
                    return Err(error("group", "group name"));
                }
                let mut ids = Vec::with_capacity(members.len());
                for text in members {
                    let id = self.id(&text)?;
                    self.entity(id)?;
                    ids.push(id);
                }
                let group = om9_core::model::Group::new(name, ids);
                let id = group.id;
                self.groups_created.push(group);
                json(&id.to_string())
            }
            HostRequest::IsObjectPickable { id } => {
                let id = self.id(&id)?;
                let attributes = self.object_attributes(id)?;
                json(&(attributes.effective_visible && attributes.effective_selectable))
            }
            HostRequest::ObjectProvenance { id } => {
                let id = self.id(&id)?;
                self.entity(id)?;
                let report = self.base.provenance(id).map(|proof| ObjectProvenance {
                    source_name: proof.source_name,
                    source_sha256: proof
                        .source_sha256
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect(),
                    archive_version: proof.archive_version,
                    import_mode: proof.import_mode.map(import_mode_name),
                    kind: record_kind_name(&proof.kind),
                    original_uuid: proof.original_uuid.to_string(),
                    original_index: proof.original_index,
                    original_family: proof.original_family,
                    disposition: disposition_name(proof.disposition),
                });
                json(&report)
            }
            HostRequest::SetTolerance {
                absolute_mm,
                relative,
                angular_rad,
            } => {
                let tolerance =
                    om9_core::history::Tolerance::try_new(absolute_mm, relative, angular_rad)
                        .map_err(tolerance_error)?;
                self.tolerance = Some(tolerance);
                json(&())
            }
            HostRequest::LayerInfoGet { id } => {
                let layer = LayerId(uuid::Uuid::parse_str(&id).map_err(|e| error("id", e))?);
                json(&self.layer_info(layer)?)
            }
            HostRequest::ObjectInfoGet { id } => {
                let id = self.id(&id)?;
                let entity = self.entity(id)?;
                json(&self.object_info(entity)?)
            }
            HostRequest::ObjectPlacement { id } => {
                let id = self.id(&id)?;
                let entity = self.entity(id)?;
                let placement = self
                    .placements
                    .get(&id)
                    .copied()
                    .unwrap_or(entity.placement);
                json(&placement.0)
            }
            HostRequest::LayerClearCurrent => {
                if self.current_layer().is_some() {
                    self.current_layer = Some(None);
                    self.created_current = None;
                }
                json(&())
            }
            HostRequest::GroupsList => {
                let mut groups = self
                    .base
                    .groups
                    .values()
                    .map(|group| GroupInfo {
                        id: group.id.to_string(),
                        name: group.name.clone(),
                        members: group.members.iter().map(|id| id.to_string()).collect(),
                        revision: group.revision,
                    })
                    .collect::<Vec<_>>();
                groups.retain(|group| {
                    group
                        .id
                        .parse::<uuid::Uuid>()
                        .is_ok_and(|id| !self.groups_deleted.contains(&GroupId(id)))
                });
                for group in &self.groups_created {
                    groups.push(GroupInfo {
                        id: group.id.to_string(),
                        name: group.name.clone(),
                        members: group.members.iter().map(|id| id.to_string()).collect(),
                        revision: group.revision,
                    });
                }
                json(&groups)
            }
            HostRequest::LayerOrder => {
                let mut order: Vec<String> = self
                    .base
                    .layer_order
                    .iter()
                    .map(|id| id.to_string())
                    .collect();
                for id in self.created_layers.keys() {
                    let text = id.to_string();
                    if !order.contains(&text) {
                        order.push(text);
                    }
                }
                json(&order)
            }
            HostRequest::MeasureLabel { text } => {
                if text.is_empty() || text.len() > MAX_MESSAGE_BYTES {
                    return Err(error("message", "measurement label"));
                }
                if self.messages.len() >= MAX_MESSAGES {
                    return Err(error("budget", "measurement message count"));
                }
                self.messages.push(PluginMessage {
                    level: om9_sdk::MessageLevel::Progress,
                    text,
                });
                json(&())
            }
            HostRequest::ObjectSummary => {
                let mut by_kind: BTreeMap<String, u64> = BTreeMap::new();
                let mut total = 0u64;
                for entity in self.added.values() {
                    *by_kind
                        .entry(entity.geometry.kind().tag().to_string())
                        .or_default() += 1;
                    total += 1;
                }
                for (id, entity) in &self.base.entities {
                    if self.added.contains_key(id) || self.deleted.contains(id) {
                        continue;
                    }
                    *by_kind
                        .entry(entity.geometry.kind().tag().to_string())
                        .or_default() += 1;
                    total += 1;
                }
                json(&ObjectSummary { total, by_kind })
            }
            HostRequest::MeasureCurveRadius { id } => {
                let id = self.id(&id)?;
                let entity = self.entity(id)?;
                let CoreGeometry::Curve(Curve::Circle { radius, .. }) = &entity.geometry else {
                    return Err(error("unsupported", "radius needs a circle curve"));
                };
                json(&MeasuredDistance {
                    millimeters: *radius,
                })
            }
            HostRequest::MeasureCurveLength { id } => {
                let id = self.id(&id)?;
                let entity = self.entity(id)?;
                let CoreGeometry::Curve(curve) = &entity.geometry else {
                    return Err(error("unsupported", "length needs a curve"));
                };
                let points = om9_core::surface_kernel::sample_curve(curve, 33, self.limits)
                    .map_err(measure_error)?;
                let mut millimeters = 0.0;
                for pair in points.windows(2) {
                    millimeters += (pair[1][0] - pair[0][0])
                        .hypot(pair[1][1] - pair[0][1])
                        .hypot(pair[1][2] - pair[0][2]);
                }
                json(&MeasuredDistance { millimeters })
            }
            HostRequest::ObjectBounds { id } => {
                let id = self.id(&id)?;
                let entity = self.entity(id)?;
                let points: Vec<[f64; 3]> = match &entity.geometry {
                    CoreGeometry::Point(point) => vec![*point],
                    CoreGeometry::Curve(curve) => {
                        om9_core::surface_kernel::sample_curve(curve, 33, self.limits)
                            .map_err(bounds_error)?
                    }
                    CoreGeometry::Mesh(mesh) => {
                        if mesh.vertices.len() > MAX_MESH_BOUNDS {
                            return Err(error("budget", "mesh bound vertices"));
                        }
                        mesh.vertices.clone()
                    }
                    _ => return Err(error("unsupported", "object bounds geometry")),
                };
                let mut min = [f64::INFINITY; 3];
                let mut max = [f64::NEG_INFINITY; 3];
                for point in points {
                    for axis in 0..3 {
                        min[axis] = min[axis].min(point[axis]);
                        max[axis] = max[axis].max(point[axis]);
                    }
                }
                if !min[0].is_finite() || !max[0].is_finite() {
                    return Err(error("bounds", "object has no bounded points"));
                }
                let size = std::array::from_fn(|axis| max[axis] - min[axis]);
                let center = std::array::from_fn(|axis| (min[axis] + max[axis]) / 2.0);
                json(&ObjectBounds {
                    min,
                    max,
                    size,
                    center,
                })
            }
            HostRequest::CurveSegments { id } => {
                let id = self.id(&id)?;
                json(&self.curve_segments_of(id)?)
            }
            HostRequest::CurveEvaluateSegment {
                id,
                index,
                parameters,
                order,
            } => {
                let id = self.id(&id)?;
                json(&self.evaluate_curve_segment(id, index, parameters, order)?)
            }
            HostRequest::CurveDefinition { id } => {
                let id = self.id(&id)?;
                json(&self.curve_definition(id)?)
            }
            HostRequest::CurveSemantics { id, tolerance_mm } => {
                let id = self.id(&id)?;
                json(&self.curve_semantics_of(id, tolerance_mm)?)
            }
            HostRequest::CurveEvaluate {
                id,
                parameters,
                order,
            } => {
                let id = self.id(&id)?;
                json(&self.evaluate_curve_samples(id, parameters, order)?)
            }
            HostRequest::Message { level, text } => {
                if text.is_empty() || text.len() > MAX_MESSAGE_BYTES {
                    return Err(error("budget", "message text is empty or over 512 bytes"));
                }
                if text.chars().any(char::is_control) {
                    return Err(error(
                        "message",
                        "message is one line without control characters",
                    ));
                }
                if self.messages.len() >= MAX_MESSAGES {
                    return Err(error("budget", "too many messages in one command"));
                }
                self.messages.push(PluginMessage { level, text });
                json(&())
            }
            HostRequest::ObjectDelete { id } => {
                let id = self.id(&id)?;
                self.editable(id)?;
                if self.added.remove(&id).is_none() {
                    self.deleted.insert(id);
                }
                self.placements.remove(&id);
                json(&())
            }
            HostRequest::HistoryList => json(&self.history_records()?),
            HostRequest::HistoryFeature { id } => {
                let id = self.history_id(&id)?;
                json(&self.history_record(id)?)
            }
            HostRequest::HistoryDependencies => {
                let edges =
                    typed_edges(self.base, &self.base.history, self.limits).map_err(core_error)?;
                let dependencies: Vec<FeatureDependencyInfo> = edges
                    .iter()
                    .map(|edge| FeatureDependencyInfo {
                        producer: edge.producer.map(|id| id.to_string()),
                        consumer: edge.consumer.to_string(),
                        role: edge.role.clone(),
                        kind: port_label(&edge.kind),
                        source: edge.source.to_string(),
                    })
                    .collect();
                json(&dependencies)
            }
            HostRequest::HistoryProvenance { id } => {
                let id = self.history_id(&id)?;
                let recipe = self
                    .base
                    .history
                    .recipes
                    .get(&id)
                    .ok_or_else(|| error("missing", "history feature"))?;
                json(&provenance_info(recipe)?)
            }
            HostRequest::HistoryRebuildPlan => {
                let plan = om9_core::history_ops::plan_recompute(&self.base.history)
                    .map_err(core_error)?;
                json(&plan.iter().map(ToString::to_string).collect::<Vec<_>>())
            }
            HostRequest::ObjectTransform { id, matrix } => {
                let id = self.id(&id)?;
                self.editable(id)?;
                let old = self
                    .placements
                    .get(&id)
                    .copied()
                    .unwrap_or(self.entity(id)?.placement);
                let next = Transform(matrix).compose(old).map_err(core_error)?;
                if let Some(entity) = self.added.get_mut(&id) {
                    entity.placement = next;
                } else {
                    self.record_edit(id, EditKind::Placement)?;
                    self.placements.insert(id, next);
                }
                json(&())
            }
        }
    }
    pub(super) fn finish(self, label: String, result: Value) -> Result<PluginRun> {
        // Even if a plugin ignores a rejected host call, its partial edits may
        // never be committed. This also preserves the public Redo stack.
        if let Some(error) = self.failed {
            return Err(error);
        }
        let selection = self.staged_selection.as_ref().map(|selection| {
            selection
                .iter()
                .copied()
                .filter(|id| !self.deleted.contains(id))
                .collect::<Vec<_>>()
        });
        let mut mutations = Vec::new();
        // Layer and current-layer plans come first so an assignment can target a layer
        // created by this same command.
        for (id, layer) in self.created_layers {
            let action = LayerAction::Create {
                layer,
                make_current: self.created_current == Some(id),
            };
            mutations.push(Mutation::Layers(Box::new(
                LayerPlan::capture(self.coordinator, self.base.id, action, None)
                    .map_err(core_error)?,
            )));
        }
        for (_, layer) in self.updated_layers {
            mutations.push(Mutation::Layers(Box::new(
                LayerPlan::capture(
                    self.coordinator,
                    self.base.id,
                    LayerAction::Update { layer },
                    None,
                )
                .map_err(core_error)?,
            )));
        }
        if let Some(target) = self.current_layer {
            mutations.push(Mutation::Layers(Box::new(
                LayerPlan::capture(
                    self.coordinator,
                    self.base.id,
                    LayerAction::SetCurrent { layer: target },
                    None,
                )
                .map_err(core_error)?,
            )));
        }
        let mut assigned: BTreeMap<LayerId, Vec<om9_core::identity::SourceStamp>> = BTreeMap::new();
        for (id, layer) in self.assignments {
            assigned
                .entry(layer)
                .or_default()
                .push(self.base.stamp(id).map_err(core_error)?);
        }
        for (layer, sources) in assigned {
            mutations.push(Mutation::Layers(Box::new(
                LayerPlan::capture(
                    self.coordinator,
                    self.base.id,
                    LayerAction::Assign { layer, sources },
                    None,
                )
                .map_err(core_error)?,
            )));
        }
        for (_, entity) in self.added {
            mutations.push(Mutation::Insert {
                document: self.base.id,
                entity,
            });
        }
        for id in self.deleted {
            mutations.push(Mutation::Delete {
                source: self.base.stamp(id).map_err(core_error)?,
            });
        }
        for (id, placement) in self.placements {
            if self.base.entities[&id].placement != placement {
                mutations.push(Mutation::Placement {
                    source: self.base.stamp(id).map_err(core_error)?,
                    placement,
                });
            }
        }
        for (id, attributes) in self.attributes {
            mutations.push(Mutation::Attributes {
                source: self.base.stamp(id).map_err(core_error)?,
                attributes,
            });
        }
        if let Some(tolerance) = self.tolerance {
            mutations.push(Mutation::Tolerance {
                document: self.base.id,
                expected_revision: self.base.revision,
                tolerance,
            });
        }
        for group in self.groups_deleted {
            mutations.push(Mutation::GroupDelete {
                document: self.base.id,
                expected_revision: self.base.revision,
                group,
            });
        }
        for group in self.groups_created {
            mutations.push(Mutation::GroupCreate {
                document: self.base.id,
                expected_revision: self.base.revision,
                group: Box::new(group),
            });
        }
        for (id, slots) in self.extensions {
            let source = self.base.stamp(id).map_err(core_error)?;
            for (slot, extension) in slots {
                mutations.push(Mutation::Extension {
                    source,
                    slot,
                    extension,
                });
            }
        }
        for (id, label) in self.labels {
            mutations.push(Mutation::Label {
                source: self.base.stamp(id).map_err(core_error)?,
                label,
            });
        }
        Ok(PluginRun {
            request: (!mutations.is_empty()).then(|| TransactionRequest::new(label, mutations)),
            result,
            messages: self.messages,
            selection,
            save: self.save_requested,
            save_as: self.save_as_requested,
        })
    }
}
fn json(value: &impl serde::Serialize) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| error("json", e))
}
/// Bound one numeric field before handing it to the document's typed parser.
fn numeric_text(text: &str) -> Result<&str> {
    if text.len() > MAX_NUMERIC_TEXT {
        return Err(error("budget", "numeric text exceeds 256 bytes"));
    }
    Ok(text)
}
/// Parse `#rrggbb` or `#rrggbbaa`, the object-colour form the property editor shows.
fn parse_color(text: &str) -> Result<[u8; 4]> {
    let hex = text.strip_prefix('#').unwrap_or(text);
    if !matches!(hex.len(), 6 | 8) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error("color", "expected #rrggbb or #rrggbbaa"));
    }
    let (body, alpha) = if hex.len() == 8 {
        hex.split_at(6)
    } else {
        (hex, "ff")
    };
    let mut color = [0u8; 4];
    for (index, channel) in color.iter_mut().take(3).enumerate() {
        *channel = u8::from_str_radix(&body[index * 2..index * 2 + 2], 16)
            .map_err(|refusal| error("color", refusal))?;
    }
    color[3] = u8::from_str_radix(alpha, 16).map_err(|refusal| error("color", refusal))?;
    Ok(color)
}
fn core_dimension(dimension: Dimension) -> om9_core::request::Dimension {
    match dimension {
        Dimension::Length => om9_core::request::Dimension::Length,
        Dimension::Angle => om9_core::request::Dimension::Angle,
        Dimension::Ratio => om9_core::request::Dimension::Ratio,
        Dimension::Count => om9_core::request::Dimension::Count,
    }
}
/// The SDK domain becomes the document owner's own option domain, so ranges, steps,
/// the integer count rule, choices and text bounds are checked by one implementation.
fn core_domain(domain: ParameterDomain) -> Result<om9_core::request::OptionDomain> {
    Ok(match domain {
        ParameterDomain::Number {
            dimension,
            min,
            min_inclusive,
            max,
            step,
        } => om9_core::request::OptionDomain::Number {
            dimension: core_dimension(dimension),
            min,
            min_inclusive,
            max,
            step,
        },
        ParameterDomain::Choice { choices } => om9_core::request::OptionDomain::Choice(choices),
        ParameterDomain::Boolean => om9_core::request::OptionDomain::Boolean,
        ParameterDomain::Text { max_bytes } => om9_core::request::OptionDomain::Text {
            max_bytes: usize::try_from(max_bytes)
                .map_err(|_| error("parameter", "text bound exceeds platform size"))?,
        },
    })
}
fn core_parameter(value: ParameterValue) -> om9_core::request::Value {
    use om9_core::request::{Dimension as CoreDimension, Value as CoreValue};
    match value {
        ParameterValue::Length { millimeters } => CoreValue::Quantity {
            value: millimeters,
            dimension: CoreDimension::Length,
        },
        ParameterValue::Angle { degrees } => CoreValue::Quantity {
            value: degrees,
            dimension: CoreDimension::Angle,
        },
        ParameterValue::Ratio { value } => CoreValue::Quantity {
            value,
            dimension: CoreDimension::Ratio,
        },
        ParameterValue::Count { value } => CoreValue::Quantity {
            value: value as f64,
            dimension: CoreDimension::Count,
        },
        ParameterValue::Choice { value } => CoreValue::Choice(value),
        ParameterValue::Boolean { value } => CoreValue::Boolean(value),
        ParameterValue::Text { value } => CoreValue::Text(value),
        // A record may carry a bare number from legacy data; applying it is refused by the
        // option domain (missing unit), never silently treated as millimetres.
        ParameterValue::Number { value } => CoreValue::Number(value),
    }
}
fn sdk_parameter(value: &om9_core::request::Value) -> Result<ParameterValue> {
    use om9_core::request::{Dimension as CoreDimension, Value as CoreValue};
    Ok(match value {
        CoreValue::Quantity { value, dimension } => match dimension {
            CoreDimension::Length => ParameterValue::Length {
                millimeters: *value,
            },
            CoreDimension::Angle => ParameterValue::Angle { degrees: *value },
            CoreDimension::Ratio => ParameterValue::Ratio { value: *value },
            CoreDimension::Count => ParameterValue::Count {
                value: count_value(*value)?,
            },
        },
        CoreValue::Choice(value) => ParameterValue::Choice {
            value: value.clone(),
        },
        CoreValue::Boolean(value) => ParameterValue::Boolean { value: *value },
        CoreValue::Text(value) => ParameterValue::Text {
            value: value.clone(),
        },
        CoreValue::Number(value) => ParameterValue::Number { value: *value },
    })
}
fn count_value(value: f64) -> Result<u64> {
    if !value.is_finite() || value < 0. || value.fract() != 0. || value > u64::MAX as f64 {
        return Err(error("parameter", "count must be a non-negative integer"));
    }
    Ok(value as u64)
}

/// The state label a history reader sees; the reason travels beside it, never inside it.
fn state_label(state: &EvaluationState) -> (String, Option<String>) {
    match state {
        EvaluationState::Clean => ("clean".into(), None),
        EvaluationState::Dirty(reason) => ("dirty".into(), Some(reason.clone())),
        EvaluationState::Pending => ("pending".into(), None),
        EvaluationState::Unresolved(reason) => ("unresolved".into(), Some(reason.clone())),
        EvaluationState::Failed(reason) => ("failed".into(), Some(reason.clone())),
        EvaluationState::Detached => ("detached".into(), None),
    }
}

/// The public label of a recorded port. Keeping it in one function means the record, the
/// dependency edge and the provenance always describe a type the same way.
fn port_label(kind: &PortKind) -> String {
    match kind {
        PortKind::Geometry { kinds } => {
            let names: Vec<&str> = kinds.iter().map(|kind| kind.tag()).collect();
            format!("geometry:{}", names.join("|"))
        }
        PortKind::Length => "length".into(),
        PortKind::Angle => "angle".into(),
        PortKind::Ratio => "ratio".into(),
        PortKind::Count => "count".into(),
        PortKind::Text => "text".into(),
        PortKind::Untyped => "untyped".into(),
    }
}

fn feature_inputs(recipe: &Recipe) -> Vec<FeatureInputInfo> {
    recipe
        .inputs
        .iter()
        .enumerate()
        .map(|(index, input)| FeatureInputInfo {
            role: input.role.clone(),
            kind: recipe
                .input_ports
                .get(index)
                .map(|port| port_label(&port.kind))
                .unwrap_or_else(|| "untyped".into()),
            object: input.source.object.to_string(),
        })
        .collect()
}

fn feature_parameters(
    parameters: &om9_core::request::Options,
) -> Result<BTreeMap<String, ParameterValue>> {
    parameters
        .iter()
        .map(|(name, value)| Ok((name.clone(), sdk_parameter(value)?)))
        .collect()
}

fn history_record_info(recipe: &Recipe) -> Result<FeatureRecordInfo> {
    let (state, state_reason) = state_label(&recipe.state);
    Ok(FeatureRecordInfo {
        id: recipe.id.to_string(),
        command: recipe.command.clone(),
        state,
        state_reason,
        revision: recipe.revision,
        parameters: feature_parameters(&recipe.parameters)?,
        inputs: feature_inputs(recipe),
        outputs: recipe.outputs.iter().map(ToString::to_string).collect(),
    })
}

fn provenance_info(recipe: &Recipe) -> Result<FeatureProvenanceInfo> {
    let provenance = feature_provenance(recipe).map_err(core_error)?;
    Ok(FeatureProvenanceInfo {
        recipe: provenance.recipe.to_string(),
        command: provenance.command,
        command_version: provenance.command_version,
        schema_version: provenance.schema_version,
        solver: provenance.solver,
        solver_version: provenance.solver_version,
        unit: provenance.unit.symbol().into(),
        parameters: feature_parameters(&provenance.parameters)?,
        inputs: provenance
            .inputs
            .iter()
            .map(|input| FeatureInputInfo {
                role: input.role.clone(),
                kind: port_label(&input.kind),
                object: input.stamp.object.to_string(),
            })
            .collect(),
        outputs: provenance.outputs.iter().map(ToString::to_string).collect(),
    })
}
/// A bounds refusal is the analysis/sampling owner's decision.
fn bounds_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "bounds",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// A mesh refusal is the mesh record's admission decision.
fn mesh_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "mesh",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// A placement refusal is the document's affine-matrix admission decision.
fn placement_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "placement",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// An extension refusal is the document record's admission decision.
fn extension_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "extension",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// A missing document tolerance is a policy refusal, not a malformed curve.
fn import_mode_name(mode: om9_core::retained::ImportMode) -> String {
    match mode {
        om9_core::retained::ImportMode::Preserve => "preserve",
        om9_core::retained::ImportMode::Geometry => "geometry",
    }
    .to_string()
}

fn disposition_name(disposition: om9_core::retained::RecordDisposition) -> String {
    match disposition {
        om9_core::retained::RecordDisposition::Unchanged => "unchanged",
        om9_core::retained::RecordDisposition::Edited => "edited",
        om9_core::retained::RecordDisposition::Deleted => "deleted",
        om9_core::retained::RecordDisposition::RetainedOnly => "retained_only",
    }
    .to_string()
}

fn record_kind_name(kind: &om9_core::retained::NativeRecordKind) -> String {
    use om9_core::retained::NativeRecordKind as Kind;
    match kind {
        Kind::Geometry => "geometry".into(),
        Kind::Layer => "layer".into(),
        Kind::Definition => "definition".into(),
        Kind::Material => "material".into(),
        Kind::Light => "light".into(),
        Kind::History => "history".into(),
        Kind::Group => "group".into(),
        Kind::LineType => "line_type".into(),
        Kind::DimStyle => "dim_style".into(),
        Kind::HatchPattern => "hatch_pattern".into(),
        Kind::TextureMapping => "texture_mapping".into(),
        Kind::Image => "image".into(),
        Kind::RenderContent => "render_content".into(),
        Kind::EmbeddedFile => "embedded_file".into(),
        Kind::Other { native_type } => format!("other:{native_type}"),
    }
}
fn tolerance_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "tolerance",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// Measurement refusals are the analysis owner's admission decision.
fn measure_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "measure",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// A curve refusal is the owner's numerical decision, not a transport error.
fn curve_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Unsupported(_) => "unsupported",
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "curve",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// A parameter refusal is the document's typed-option decision, not a transport error.
fn parameter_error(refusal: om9_core::Error) -> SdkError {
    let code = match refusal {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => "budget",
        _ => "parameter",
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
/// Map the document's typed-unit refusal onto a stable SDK error code. An unknown unit
/// and a malformed/non-finite number stay distinct so a plugin can report them apart.
fn unit_error(refusal: om9_core::units::UnitError) -> SdkError {
    let code = if matches!(refusal, om9_core::units::UnitError::UnsupportedUnit { .. }) {
        "unit"
    } else {
        "number"
    };
    SdkError {
        code: code.into(),
        message: refusal.to_string(),
    }
}
fn to_core(geometry: Geometry, limits: &Limits) -> Result<CoreGeometry> {
    Ok(match geometry {
        Geometry::Point { point } => CoreGeometry::Point(point),
        Geometry::Line { start, end } => CoreGeometry::Curve(Curve::Line {
            start,
            end,
            domain: [0., 1.],
        }),
        Geometry::Polyline { points } => {
            Usage::array::<[f64; 3]>(points.len())
                .map_err(core_error)?
                .check(limits)
                .map_err(core_error)?;
            CoreGeometry::Curve(om9_core::curve_build::polyline(&points).map_err(core_error)?)
        }
        Geometry::Circle {
            center,
            x,
            y,
            radius,
        } => CoreGeometry::Curve(Curve::Circle {
            center,
            x,
            y,
            radius,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
    })
}
fn from_core(geometry: &CoreGeometry) -> Result<Geometry> {
    Ok(match geometry {
        CoreGeometry::Point(point) => Geometry::Point { point: *point },
        CoreGeometry::Curve(Curve::Line { start, end, .. }) => Geometry::Line {
            start: *start,
            end: *end,
        },
        CoreGeometry::Curve(Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        }) if *domain == [0., std::f64::consts::TAU] && angle_domain.is_none() => {
            Geometry::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: *radius,
            }
        }
        CoreGeometry::Curve(Curve::Polycurve {
            segments,
            reversed: false,
        }) => {
            let mut points = Vec::new();
            for segment in segments {
                let Curve::Line { start, end, .. } = segment else {
                    return Err(error(
                        "unsupported",
                        "polycurve contains nonlinear segments",
                    ));
                };
                if points.is_empty() {
                    points.push(*start);
                } else if points.last() != Some(start) {
                    return Err(error("unsupported", "disconnected polycurve"));
                }
                points.push(*end);
            }
            Geometry::Polyline { points }
        }
        _ => {
            return Err(error(
                "unsupported",
                "SDK v1 geometry read supports only point, line, connected polyline and full circle",
            ));
        }
    })
}

/// # Safety
/// Called synchronously only with the live `Staged` pointer supplied by prepare;
/// input and reply follow the SDK's borrowed-buffer ABI contract.
pub(super) unsafe extern "C" fn call(
    context: *mut c_void,
    input: *const u8,
    len: usize,
    reply_context: *mut c_void,
    reply: ReplyFn,
) -> i32 {
    if context.is_null() {
        return abi::STATUS_ERROR;
    }
    // SAFETY: callback context is a unique borrow for this synchronous call.
    let staged = unsafe { &mut *context.cast::<Staged<'_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Value> {
        if input.is_null() || len > abi::MAX_MESSAGE_BYTES {
            return Err(error("protocol", "invalid host input buffer"));
        }
        // SAFETY: the plugin guarantees input is readable for len bytes.
        let bytes = unsafe { std::slice::from_raw_parts(input, len) };
        let request = serde_json::from_slice(bytes).map_err(|e| error("request", e))?;
        if let Some(error) = &staged.failed {
            return Err(error.clone());
        }
        staged.handle(request)
    }))
    .unwrap_or_else(|_| Err(error("panic", "host callback panicked")));
    let (status, bytes) = match result {
        Ok(value) => (abi::STATUS_OK, serde_json::to_vec(&value)),
        Err(error) => {
            staged.failed = Some(error.clone());
            (abi::STATUS_ERROR, serde_json::to_vec(&error))
        }
    };
    let (status, bytes) = match bytes {
        Ok(bytes) if bytes.len() <= abi::MAX_MESSAGE_BYTES => (status, bytes),
        _ => {
            let failure = error("budget", "host response exceeds byte limit");
            staged.failed = Some(failure.clone());
            (
                abi::STATUS_ERROR,
                serde_json::to_vec(&failure).expect("fixed bounded error"),
            )
        }
    };
    // SAFETY: synchronous callback with a readable borrowed buffer.
    if unsafe { reply(reply_context, bytes.as_ptr(), bytes.len()) } != abi::STATUS_OK {
        staged.failed = Some(error("protocol", "host reply was rejected"));
        return abi::STATUS_ERROR;
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{
        geometry::Unit,
        identity::DocumentId,
        model::Layer,
        transaction::{TransactionCoordinator, World},
    };
    use om9_sdk::{
        AttributesPatch, Dimension, Host, MessageLevel, ObjectQuery, ParameterDomain,
        ParameterSpec, ParameterValue, PluginMessage, abi::HostApi,
    };

    fn with_host<T>(staged: &mut Staged<'_>, f: impl FnOnce(&mut Host<'_>) -> T) -> T {
        let api = HostApi {
            abi_version: 1,
            struct_size: std::mem::size_of::<HostApi>() as u32,
            context: (staged as *mut Staged<'_>).cast(),
            call: Some(call),
        };
        // SAFETY: table/context/callback live on this thread for the closure only.
        let mut host = unsafe { Host::from_raw(&api) }.unwrap();
        f(&mut host)
    }
    /// One document sits in a coordinator, so a staged command can capture layer plans.
    fn coordinator_with(
        document: Document,
        limits: Limits,
    ) -> (TransactionCoordinator, DocumentId) {
        let id = document.id;
        (
            TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits)
                .unwrap(),
            id,
        )
    }
    #[test]
    fn composed_edits_are_one_core_transaction_and_no_units_are_rescaled() {
        let limits = Limits::default();
        let mut document = Document::new("inch display");
        document.page_unit = Unit::Inch;
        let entity = Entity::new("source", CoreGeometry::Point([25.4, 0., 0.]), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let selected = [id];
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &selected, &limits).unwrap();
        with_host(&mut staged, |host| {
            assert_eq!(host.document_info().unwrap().unit, "in");
            assert_eq!(host.selected_objects().unwrap(), vec![id.to_string()]);
            assert_eq!(
                host.object_geometry(id.to_string()).unwrap(),
                Geometry::Point {
                    point: [25.4, 0., 0.]
                }
            );
            host.transform_object(id.to_string(), Transform::translation([1., 0., 0.]).0)
                .unwrap();
            host.transform_object(id.to_string(), Transform::translation([2., 0., 0.]).0)
                .unwrap();
            let fresh = host.add_point("new", [2., 0., 0.]).unwrap();
            host.transform_object(&fresh, Transform::translation([4., 0., 0.]).0)
                .unwrap();
            host.delete_object(fresh).unwrap();
        });
        let run = staged.finish("composed".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert_eq!(
            coordinator.world().documents[&document.id].entities.len(),
            1
        );
        assert_eq!(
            coordinator.world().documents[&document.id].entities[&id]
                .placement
                .0[0][3],
            3.
        );
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);
    }
    #[test]
    fn ignored_host_error_poisoning_discards_partial_output() {
        let document = Document::new("rollback");
        let limits = Limits::default();
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.add_point("valid", [1., 2., 3.]).unwrap();
            assert!(host.add_line("bad", [0.; 3], [0.; 3]).is_err());
        });
        assert!(staged.finish("ignored error".into(), Value::Null).is_err());
        assert!(document.entities.is_empty());
    }
    #[test]
    fn locked_objects_cannot_be_deleted_but_hidden_current_layer_can_receive_output() {
        let limits = Limits::default();
        let mut document = Document::new("permissions");
        let mut entity = Entity::new("locked", CoreGeometry::Point([0.; 3]), None);
        entity.attributes.locked = true;
        let id = entity.id;
        document.entities.insert(id, entity);
        {
            let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
            let base = &coordinator.world().documents[&doc_id];
            let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
            with_host(&mut staged, |host| {
                assert!(host.delete_object(id.to_string()).is_err())
            });
            assert!(staged.finish("locked".into(), Value::Null).is_err());
        }
        let mut layer = Layer::new("hidden", [1, 2, 3, 255]);
        layer.visible = false;
        let layer_id = layer.id;
        document.layers.insert(layer_id, layer);
        document.layer_order.push(layer_id);
        document.current_layer = Some(layer_id);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.add_point("output", [1.; 3]).unwrap();
        });
        let run = staged.finish("hidden output".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert_eq!(
            coordinator
                .world()
                .documents
                .values()
                .next()
                .unwrap()
                .entities
                .len(),
            2
        );
    }

    #[test]
    fn units_are_read_and_typed_numbers_parse_with_the_document_contract() {
        let limits = Limits::default();
        let mut document = Document::new("inch display");
        document.page_unit = Unit::Inch;
        assert!(document.tolerance().is_none());
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let units = host.document_units().unwrap();
            assert_eq!(units.unit, "in");
            assert_eq!(units.millimeters_per_unit, 25.4);
            // A document that states no tolerance must report `None`, not the template.
            assert_eq!(units.tolerance, None);
            // A bare number uses the document display unit; a token carries its own.
            assert_eq!(host.parse_length("1").unwrap().millimeters, 25.4);
            assert_eq!(host.parse_length("2.5 mm").unwrap().millimeters, 2.5);
            assert!((host.parse_length("0.1 in").unwrap().millimeters - 2.54).abs() < 1e-12);
            assert_eq!(host.parse_length("1.5 cm").unwrap().millimeters, 15.0);
            let right = host.parse_angle("90").unwrap();
            assert_eq!(right.degrees, 90.0);
            assert_eq!(right.radians, std::f64::consts::FRAC_PI_2);
            assert_eq!(host.parse_angle("180deg").unwrap().degrees, 180.0);
            assert_eq!(
                host.parse_angle("1.5707963267948966rad").unwrap().degrees,
                90.0
            );
            assert!((host.parse_angle("1rad").unwrap().degrees - 57.29577951308232).abs() < 1e-12);
            // The command prompt's coordinate grammar, converted to canonical mm.
            assert_eq!(
                host.parse_coordinates("1,2").unwrap().millimeters,
                [25.4, 50.8, 0.0]
            );
            assert_eq!(
                host.parse_coordinates("1, 2, 3").unwrap().millimeters,
                [25.4, 50.8, 25.4 * 3.0]
            );
        });
        let run = staged.finish("read only".into(), Value::Null).unwrap();
        // Reading units or parsing text never becomes a mutation.
        assert!(run.request.is_none());
    }

    /// A refused host call poisons the invocation, so each refusal case needs its own
    /// staged invocation to observe the refusal that call itself produced.
    #[test]
    fn typed_number_refusals_name_the_unit_number_and_coordinate_cases() {
        let limits = Limits::default();
        let mut document = Document::new("inch display");
        document.page_unit = Unit::Inch;
        let refusal = |request: &dyn Fn(&mut Host<'_>) -> Result<Value>| {
            let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
            let base = &coordinator.world().documents[&doc_id];
            let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
            let api = HostApi {
                abi_version: 1,
                struct_size: std::mem::size_of::<HostApi>() as u32,
                context: (&mut staged as *mut Staged<'_>).cast(),
                call: Some(call),
            };
            // SAFETY: table/context/callback live on this thread for this borrow only.
            let mut host = unsafe { Host::from_raw(&api) }.unwrap();
            request(&mut host)
        };
        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| {
            refusal(f).expect_err("typed refusal").code
        };

        assert_eq!(
            code(&|host| host.parse_length("1.5 rad").map(|_| Value::Null)),
            "unit"
        );
        // A trailing alphabetic token is read as a unit, so "nan"/"inf" leave no
        // number to parse and are refused as numeric text rather than as a unit.
        assert_eq!(
            code(&|host| host.parse_length("nan").map(|_| Value::Null)),
            "number"
        );
        // Overflowing text becomes a non-finite value, which is refused, not zero.
        assert_eq!(
            code(&|host| host.parse_length("1e999").map(|_| Value::Null)),
            "number"
        );
        assert_eq!(
            code(&|host| host.parse_length("1.2.3").map(|_| Value::Null)),
            "number"
        );
        assert_eq!(
            code(&|host| {
                host.parse_length("1".repeat(MAX_NUMERIC_TEXT + 1))
                    .map(|_| Value::Null)
            }),
            "budget"
        );
        assert_eq!(
            code(&|host| host.parse_angle("1000001").map(|_| Value::Null)),
            "number"
        );
        assert_eq!(
            code(&|host| host.parse_angle("1.5 mm").map(|_| Value::Null)),
            "unit"
        );
        assert_eq!(
            code(&|host| host.parse_coordinates("1").map(|_| Value::Null)),
            "coordinate"
        );
        assert_eq!(
            code(&|host| host.parse_coordinates("1,2,nan").map(|_| Value::Null)),
            "coordinate"
        );
        assert_eq!(
            code(&|host| host.parse_coordinates("1,2,3,4").map(|_| Value::Null)),
            "coordinate"
        );
    }

    #[test]
    fn a_stated_tolerance_is_reported_in_canonical_units() {
        let limits = Limits::default();
        let mut document = Document::with_template_tolerance("tolerance");
        document.page_unit = Unit::Micrometer;
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let units = host.document_units().unwrap();
            assert_eq!(units.unit, "um");
            assert_eq!(units.millimeters_per_unit, 0.001);
            let tolerance = units.tolerance.expect("template tolerance");
            assert_eq!(tolerance.absolute_mm, 0.001);
            assert_eq!(tolerance.relative, 1.0e-4);
            assert_eq!(tolerance.angular_rad, std::f64::consts::PI / 180.0);
            // Typed lengths stay canonical mm in a micrometre document.
            assert_eq!(host.parse_length("1000").unwrap().millimeters, 1.0);
            assert_eq!(
                host.parse_coordinates("1000,0").unwrap().millimeters,
                [1.0, 0.0, 0.0]
            );
        });
    }

    /// Run one invocation against a fresh staged document and return the refusal.
    fn refusal(
        document: &Document,
        selection: &[ObjectId],
        f: impl FnOnce(&mut Host<'_>) -> Result<Value>,
    ) -> SdkError {
        let limits = Limits::default();
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, selection, &limits).unwrap();
        let api = HostApi {
            abi_version: 1,
            struct_size: std::mem::size_of::<HostApi>() as u32,
            context: (&mut staged as *mut Staged<'_>).cast(),
            call: Some(call),
        };
        // SAFETY: table/context/callback live on this thread for the closure only.
        let mut host = unsafe { Host::from_raw(&api) }.unwrap();
        f(&mut host).expect_err("expected a typed refusal")
    }

    #[test]
    fn attribute_and_label_edits_are_staged_and_readable_before_commit() {
        let limits = Limits::default();
        let mut document = Document::new("attributes");
        let first = Entity::new("first", CoreGeometry::Point([0.; 3]), None);
        let second = Entity::new("second", CoreGeometry::Point([1.; 3]), None);
        let (first_id, second_id) = (first.id, second.id);
        document.entities.insert(first_id, first);
        document.entities.insert(second_id, second);

        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let before = host.object_attributes(first_id.to_string()).unwrap();
            assert_eq!(before.label, "first");
            assert_eq!(before.color, None);
            assert!(before.effective_visible && before.effective_editable);
            let after = host
                .set_object_attributes(
                    first_id.to_string(),
                    AttributesPatch {
                        color: Some("#0a141e".into()),
                        hidden: Some(true),
                        ..AttributesPatch::default()
                    },
                )
                .unwrap();
            assert_eq!(after.color, Some([10, 20, 30, 255]));
            assert!(after.hidden);
            assert!(!after.effective_visible);
            assert!(!after.effective_editable);
            assert_eq!(after.attributes_revision, before.attributes_revision + 1);
            // Reads in the same command see the staged record.
            assert_eq!(host.object_attributes(first_id.to_string()).unwrap(), after);
            assert!(
                !host
                    .objects()
                    .unwrap()
                    .iter()
                    .find(|object| object.id == first_id.to_string())
                    .unwrap()
                    .visible
            );
            // A rename on a different object is a separate source write.
            host.rename_object(second_id.to_string(), "renamed")
                .unwrap();
            assert_eq!(
                host.object_attributes(second_id.to_string()).unwrap().label,
                "renamed"
            );
        });
        let run = staged.finish("attributes".into(), Value::Null).unwrap();
        let request = run.request.expect("attribute and label mutations");
        let prepared = coordinator.prepare(request).unwrap();
        coordinator.commit(prepared).unwrap();
        let updated = &coordinator.world().documents[&document.id];
        assert_eq!(
            updated.entities[&first_id].attributes.color,
            Some([10, 20, 30, 255])
        );
        assert!(updated.entities[&first_id].attributes.hidden);
        assert_eq!(
            updated.entities[&first_id].attributes_revision,
            document.entities[&first_id].attributes_revision + 1
        );
        assert_eq!(updated.entities[&second_id].label, "renamed");
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);
    }

    #[test]
    fn object_query_filters_pages_and_sees_staged_additions() {
        let limits = Limits::default();
        let mut document = Document::new("query");
        let layer = Layer::new("Gems", [1, 2, 3, 255]);
        let layer_id = layer.id;
        document.layers.insert(layer_id, layer);
        document.layer_order.push(layer_id);
        document.current_layer = Some(layer_id);
        let mut ids = Vec::new();
        for (index, label) in ["gem one", "gem two", "plain"].into_iter().enumerate() {
            let entity = Entity::new(label, CoreGeometry::Point([index as f64; 3]), layer_id);
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let selection = [ids[0]];
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &selection, &limits).unwrap();
        with_host(&mut staged, |host| {
            let page = host.query_objects(ObjectQuery::default()).unwrap();
            assert_eq!(page.total, 3);
            assert_eq!(page.objects.len(), 3);
            let mut expected = ids.clone();
            expected.sort();
            assert_eq!(
                page.objects
                    .iter()
                    .map(|object| object.id.clone())
                    .collect::<Vec<_>>(),
                expected.iter().map(ToString::to_string).collect::<Vec<_>>()
            );
            let page = host
                .query_objects(ObjectQuery {
                    offset: 1,
                    limit: 1,
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!((page.total, page.offset, page.objects.len()), (3, 1, 1));
            let page = host
                .query_objects(ObjectQuery {
                    name_contains: Some("gem".into()),
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.total, 2);
            let page = host
                .query_objects(ObjectQuery {
                    geometry_kind: Some("Point".into()),
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.total, 3);
            let page = host
                .query_objects(ObjectQuery {
                    layer: Some(layer_id.to_string()),
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.total, 3);
            let page = host
                .query_objects(ObjectQuery {
                    only_selected: true,
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.total, 1);
            assert_eq!(page.objects[0].id, ids[0].to_string());

            // Staged additions and their own attribute edits join the same query.
            let added = host.add_point("gem three", [5., 0., 0.]).unwrap();
            let patched = host
                .set_object_attributes(
                    &added,
                    AttributesPatch {
                        color: Some("#ffffff".into()),
                        ..AttributesPatch::default()
                    },
                )
                .unwrap();
            assert_eq!(patched.color, Some([255, 255, 255, 255]));
            let page = host
                .query_objects(ObjectQuery {
                    name_contains: Some("gem".into()),
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.total, 3);
        });
    }

    #[test]
    fn one_edit_kind_per_object_and_locked_layers_are_refused() {
        let mut document = Document::new("conflicts");
        let entity = Entity::new("target", CoreGeometry::Point([0.; 3]), None);
        let id = entity.id;
        document.entities.insert(id, entity);

        let conflict = refusal(&document, &[], |host| {
            host.transform_object(id.to_string(), Transform::translation([1., 0., 0.]).0)?;
            host.set_object_attributes(
                id.to_string(),
                AttributesPatch {
                    locked: Some(true),
                    ..AttributesPatch::default()
                },
            )
            .map(|_| Value::Null)
        });
        assert_eq!(conflict.code, "conflict");

        let label_conflict = refusal(&document, &[], |host| {
            host.set_object_attributes(
                id.to_string(),
                AttributesPatch {
                    visible: Some(false),
                    ..AttributesPatch::default()
                },
            )?;
            host.rename_object(id.to_string(), "renamed")
                .map(|()| Value::Null)
        });
        assert_eq!(label_conflict.code, "conflict");

        let bad_color = refusal(&document, &[], |host| {
            host.set_object_attributes(
                id.to_string(),
                AttributesPatch {
                    color: Some("#zzzzzz".into()),
                    ..AttributesPatch::default()
                },
            )
            .map(|_| Value::Null)
        });
        assert_eq!(bad_color.code, "color");

        let over_page = refusal(&document, &[], |host| {
            host.query_objects(ObjectQuery {
                limit: MAX_PAGE + 1,
                ..ObjectQuery::default()
            })
            .map(|_| Value::Null)
        });
        assert_eq!(over_page.code, "budget");

        // A locked layer refuses attribute edits even to unlock an object on it.
        let mut locked = Document::new("locked layer");
        let mut layer = Layer::new("locked", [1, 2, 3, 255]);
        layer.locked = true;
        let layer_id = layer.id;
        locked.layers.insert(layer_id, layer);
        locked.layer_order.push(layer_id);
        let entity = Entity::new("locked", CoreGeometry::Point([0.; 3]), layer_id);
        let locked_id = entity.id;
        locked.entities.insert(locked_id, entity);
        let locked_layer = refusal(&locked, &[], |host| {
            host.set_object_attributes(
                locked_id.to_string(),
                AttributesPatch {
                    locked: Some(false),
                    ..AttributesPatch::default()
                },
            )
            .map(|_| Value::Null)
        });
        assert_eq!(locked_layer.code, "permission");
    }

    #[test]
    fn layer_create_update_current_and_assignment_are_one_readable_transaction() {
        let limits = Limits::default();
        let mut document = Document::new("layers");
        let base_layer = Layer::new("Base", [10, 20, 30, 255]);
        let base_layer_id = base_layer.id;
        document.layers.insert(base_layer_id, base_layer);
        document.layer_order.push(base_layer_id);
        document.current_layer = Some(base_layer_id);
        let spare = Layer::new("Spare", [40, 50, 60, 255]);
        let spare_id = spare.id;
        document.layers.insert(spare_id, spare);
        document.layer_order.push(spare_id);
        let entity = Entity::new("on base", CoreGeometry::Point([0.; 3]), base_layer_id);
        let entity_id = entity.id;
        document.entities.insert(entity_id, entity);

        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut created_id = String::new();
        with_host(&mut staged, |host| {
            created_id = host.layer_create("Gems", [1, 2, 3, 255], true).unwrap();
            let layers = host.layers().unwrap();
            assert_eq!(layers.len(), 3);
            let created = layers
                .iter()
                .find(|layer| layer.id == created_id)
                .expect("created layer is readable");
            assert_eq!(created.name, "Gems");
            assert_eq!(created.color, [1, 2, 3, 255]);
            assert!(created.current);
            assert_eq!(
                host.document_info().unwrap().current_layer.as_deref(),
                Some(created_id.as_str())
            );
            // Assignment clears the object colour, matching the application's rule.
            host.assign_objects_to_layer(vec![entity_id.to_string()], &created_id)
                .unwrap();
            let attributes = host.object_attributes(entity_id.to_string()).unwrap();
            assert_eq!(attributes.layer.as_deref(), Some(created_id.as_str()));
            assert_eq!(attributes.color, None);
            assert_eq!(attributes.attributes_revision, 2);
            // A spare layer's flags can change in the same command.
            host.layer_update(
                spare_id.to_string(),
                LayerUpdate {
                    name: Some("Spare renamed".into()),
                    visible: Some(false),
                    ..LayerUpdate::default()
                },
            )
            .unwrap();
            let layers = host.layers().unwrap();
            let spare = layers
                .iter()
                .find(|layer| layer.id == spare_id.to_string())
                .unwrap();
            assert_eq!(spare.name, "Spare renamed");
            assert!(!spare.visible);
            // Another created layer stays non-current, then the base layer is selected.
            let _other = host.layer_create("Other", [9, 9, 9, 255], false).unwrap();
            host.set_current_layer(Some(base_layer_id.to_string()))
                .unwrap();
            assert_eq!(
                host.document_info().unwrap().current_layer.as_deref(),
                Some(base_layer_id.to_string().as_str())
            );
        });
        let run = staged.finish("layers".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let updated = &coordinator.world().documents[&document.id];
        let created = LayerId(uuid::Uuid::parse_str(&created_id).unwrap());
        assert!(updated.layers.contains_key(&created));
        assert_eq!(updated.layers[&created].name, "Gems");
        assert_eq!(updated.layers[&created].color, [1, 2, 3, 255]);
        assert_eq!(updated.current_layer, Some(base_layer_id));
        assert_eq!(updated.entities[&entity_id].attributes.layer, Some(created));
        assert_eq!(updated.entities[&entity_id].attributes_revision, 2);
        assert_eq!(updated.layers[&spare_id].name, "Spare renamed");
        assert!(!updated.layers[&spare_id].visible);
        assert_eq!(updated.layers.len(), 4);
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);
    }

    #[test]
    fn a_created_layer_accepts_new_objects_in_the_same_command() {
        let limits = Limits::default();
        let document = Document::new("create then add");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut created = String::new();
        let mut added = String::new();
        with_host(&mut staged, |host| {
            created = host.layer_create("Gems", [1, 2, 3, 255], true).unwrap();
            added = host.add_point("gem", [1., 2., 3.]).unwrap();
            let attributes = host.object_attributes(&added).unwrap();
            assert_eq!(attributes.layer.as_deref(), Some(created.as_str()));
            assert!(attributes.effective_visible && attributes.effective_editable);
            let page = host
                .query_objects(ObjectQuery {
                    layer: Some(created.clone()),
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.total, 1);
        });
        let run = staged
            .finish("create then add".into(), Value::Null)
            .unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let layer_id = LayerId(uuid::Uuid::parse_str(&created).unwrap());
        let object_id = ObjectId(uuid::Uuid::parse_str(&added).unwrap());
        let updated = &coordinator.world().documents[&document.id];
        assert!(updated.layers.contains_key(&layer_id));
        assert_eq!(updated.current_layer, Some(layer_id));
        assert_eq!(
            updated.entities[&object_id].attributes.layer,
            Some(layer_id)
        );
        assert_eq!(updated.layers[&layer_id].name, "Gems");
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);
    }

    #[test]
    fn layer_and_assignment_refusals_name_their_rule() {
        let mut document = Document::new("refusals");
        let base_layer = Layer::new("Base", [10, 20, 30, 255]);
        let base_layer_id = base_layer.id;
        document.layers.insert(base_layer_id, base_layer);
        document.layer_order.push(base_layer_id);
        document.current_layer = Some(base_layer_id);
        let mut locked_layer = Layer::new("Locked", [1, 1, 1, 255]);
        locked_layer.locked = true;
        let locked_id = locked_layer.id;
        document.layers.insert(locked_id, locked_layer);
        document.layer_order.push(locked_id);
        let entity = Entity::new("on base", CoreGeometry::Point([0.; 3]), base_layer_id);
        let entity_id = entity.id;
        document.entities.insert(entity_id, entity);
        let locked_entity = Entity::new("on locked", CoreGeometry::Point([1.; 3]), locked_id);
        let locked_entity_id = locked_entity.id;
        document.entities.insert(locked_entity_id, locked_entity);

        // A created layer can receive output, but its proposed lock still applies.
        let add_into_locked_created = refusal(&document, &[], |host| {
            let layer = host.layer_create("Gems", [1, 2, 3, 255], true)?;
            host.layer_update(
                &layer,
                LayerUpdate {
                    locked: Some(true),
                    ..LayerUpdate::default()
                },
            )?;
            host.add_point("new", [0.; 3]).map(|_| Value::Null)
        });
        assert_eq!(add_into_locked_created.code, "permission");

        let bad_name = refusal(&document, &[], |host| {
            host.layer_create(" padded ", [1, 2, 3, 255], false)
                .map(|_| Value::Null)
        });
        assert_eq!(bad_name.code, "label");

        let unknown_current = refusal(&document, &[], |host| {
            host.set_current_layer(Some(uuid::Uuid::new_v4().to_string()))
                .map(|_| Value::Null)
        });
        assert_eq!(unknown_current.code, "layer");

        let unknown_assign = refusal(&document, &[], |host| {
            host.assign_objects_to_layer(
                vec![entity_id.to_string()],
                uuid::Uuid::new_v4().to_string(),
            )
            .map(|_| Value::Null)
        });
        assert_eq!(unknown_assign.code, "layer");

        let locked_update = refusal(&document, &[], |host| {
            host.layer_update(
                locked_id.to_string(),
                LayerUpdate {
                    visible: Some(false),
                    ..LayerUpdate::default()
                },
            )
            .map(|_| Value::Null)
        });
        assert_eq!(locked_update.code, "permission");

        let locked_source = refusal(&document, &[], |host| {
            host.assign_objects_to_layer(vec![locked_entity_id.to_string()], locked_id.to_string())
                .map(|_| Value::Null)
        });
        assert_eq!(locked_source.code, "permission");

        // Assignment is an attribute write, so it cannot be combined with a transform.
        let assign_conflict = refusal(&document, &[], |host| {
            host.assign_objects_to_layer(vec![entity_id.to_string()], base_layer_id.to_string())?;
            host.transform_object(
                entity_id.to_string(),
                Transform::translation([1., 0., 0.]).0,
            )
            .map(|()| Value::Null)
        });
        assert_eq!(assign_conflict.code, "conflict");
    }

    fn radius_spec() -> ParameterSpec {
        ParameterSpec {
            name: "radius".into(),
            domain: ParameterDomain::Number {
                dimension: Dimension::Length,
                min: Some(1.0),
                min_inclusive: true,
                max: Some(5.0),
                step: Some(0.5),
            },
            default: None,
            required: true,
        }
    }

    #[test]
    fn typed_parameters_apply_defaults_and_validate_ranges() {
        let limits = Limits::default();
        let document = Document::new("parameters");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let radius = radius_spec();
            assert_eq!(
                host.validate_parameter(
                    radius.clone(),
                    serde_json::json!({"kind": "length", "millimeters": 2.5})
                )
                .unwrap(),
                ParameterValue::Length { millimeters: 2.5 }
            );
            let sides = ParameterSpec {
                name: "sides".into(),
                domain: ParameterDomain::Number {
                    dimension: Dimension::Count,
                    min: Some(3.0),
                    min_inclusive: true,
                    max: Some(64.0),
                    step: None,
                },
                default: Some(ParameterValue::Count { value: 6 }),
                required: false,
            };
            let profile = ParameterSpec {
                name: "profile".into(),
                domain: ParameterDomain::Choice {
                    choices: vec!["round".into(), "flat".into()],
                },
                default: Some(ParameterValue::Choice {
                    value: "round".into(),
                }),
                required: false,
            };
            let applied = host
                .apply_parameters(
                    vec![radius, sides, profile],
                    BTreeMap::from([(
                        "radius".to_string(),
                        serde_json::json!({"kind": "length", "millimeters": 3.0}),
                    )]),
                )
                .unwrap();
            assert_eq!(
                applied["radius"],
                ParameterValue::Length { millimeters: 3.0 }
            );
            assert_eq!(applied["sides"], ParameterValue::Count { value: 6 });
            assert_eq!(
                applied["profile"],
                ParameterValue::Choice {
                    value: "round".into()
                }
            );
            // An angle value keeps its own unit through the same service.
            let angle = ParameterSpec {
                name: "draft".into(),
                domain: ParameterDomain::Number {
                    dimension: Dimension::Angle,
                    min: Some(0.0),
                    min_inclusive: true,
                    max: Some(45.0),
                    step: None,
                },
                default: None,
                required: true,
            };
            assert_eq!(
                host.validate_parameter(
                    angle,
                    serde_json::json!({"kind": "angle", "degrees": 12.5})
                )
                .unwrap(),
                ParameterValue::Angle { degrees: 12.5 }
            );
        });
    }

    #[test]
    fn parameter_refusals_name_the_unit_range_and_schema_rules() {
        let document = Document::new("parameter refusals");
        let refusal = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;

        // A bare number is not an implicit millimetre.
        assert_eq!(
            refusal(&|host| host
                .validate_parameter(radius_spec(), serde_json::json!(2.5))
                .map(|_| Value::Null)),
            "parameter"
        );
        // A length wire value cannot be read as an angle.
        assert_eq!(
            refusal(&|host| host
                .validate_parameter(
                    radius_spec(),
                    serde_json::json!({"kind": "angle", "millimeters": 2.5})
                )
                .map(|_| Value::Null)),
            "parameter"
        );
        // Above the declared maximum.
        assert_eq!(
            refusal(&|host| host
                .validate_parameter(
                    radius_spec(),
                    serde_json::json!({"kind": "length", "millimeters": 6.0})
                )
                .map(|_| Value::Null)),
            "parameter"
        );
        // Off the declared step grid (0.5 measured from 1.0).
        assert_eq!(
            refusal(&|host| host
                .validate_parameter(
                    radius_spec(),
                    serde_json::json!({"kind": "length", "millimeters": 2.3})
                )
                .map(|_| Value::Null)),
            "parameter"
        );
        // A count is an integer.
        let sides = ParameterSpec {
            name: "sides".into(),
            domain: ParameterDomain::Number {
                dimension: Dimension::Count,
                min: Some(3.0),
                min_inclusive: true,
                max: Some(64.0),
                step: None,
            },
            default: None,
            required: true,
        };
        assert_eq!(
            refusal(&|host| host
                .validate_parameter(
                    sides.clone(),
                    serde_json::json!({"kind": "count", "value": 2.5})
                )
                .map(|_| Value::Null)),
            "parameter"
        );
        // A required parameter without a value, and an unknown parameter name.
        assert_eq!(
            refusal(&|host| host
                .apply_parameters(vec![radius_spec()], BTreeMap::new())
                .map(|_| Value::Null)),
            "parameter"
        );
        assert_eq!(
            refusal(&|host| host
                .apply_parameters(
                    vec![radius_spec()],
                    BTreeMap::from([
                        (
                            "radius".to_string(),
                            serde_json::json!({"kind": "length", "millimeters": 2.0})
                        ),
                        (
                            "unknown".to_string(),
                            serde_json::json!({"kind": "length", "millimeters": 1.0})
                        ),
                    ])
                )
                .map(|_| Value::Null)),
            "parameter"
        );
        // The schema itself is bounded.
        assert_eq!(
            refusal(&|host| host
                .apply_parameters(vec![radius_spec(); MAX_PARAMETERS + 1], BTreeMap::new())
                .map(|_| Value::Null)),
            "budget"
        );
    }
    #[test]
    fn save_as_request_is_staged_once_and_cannot_be_combined_with_save() {
        let limits = Limits::default();
        let document = Document::new("save as");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        staged.set_project(ProjectState {
            path: Some(std::path::PathBuf::from("C:/tmp/a.om9")),
            dirty: false,
        });
        with_host(&mut staged, |host| {
            host.save_project_as().unwrap();
        });
        let run = staged.finish("save as".into(), Value::Null).unwrap();
        assert!(run.save_as);
        assert!(!run.save);

        let mut combined = Staged::new(&coordinator, base, &[], &limits).unwrap();
        combined.set_project(ProjectState {
            path: Some(std::path::PathBuf::from("C:/tmp/a.om9")),
            dirty: true,
        });
        with_host(&mut combined, |host| {
            host.save_project_as().unwrap();
            let error = host.save_project().unwrap_err();
            assert_eq!(error.code, "save");
        });
    }
    #[test]
    fn mesh_creation_validates_indices_and_publishes_one_object() {
        let limits = Limits::default();
        let document = Document::new("mesh");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut id = String::new();
        with_host(&mut staged, |host| {
            id = host
                .add_mesh(
                    "quad",
                    vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
                    vec![vec![0, 1, 2, 3]],
                )
                .unwrap();
            let page = host
                .query_objects(ObjectQuery {
                    geometry_kind: Some("Mesh".into()),
                    ..ObjectQuery::default()
                })
                .unwrap();
            assert_eq!(page.objects.len(), 1);
            assert_eq!(page.objects[0].geometry_kind, "Mesh");
        });
        let run = staged.finish("mesh".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let object = ObjectId(uuid::Uuid::parse_str(&id).unwrap());
        match &coordinator.world().documents[&document.id].entities[&object].geometry {
            CoreGeometry::Mesh(mesh) => {
                assert_eq!(mesh.vertices.len(), 4);
                assert_eq!(mesh.faces, vec![vec![0, 1, 2, 3]]);
            }
            other => panic!("expected a mesh, got {other:?}"),
        }
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .add_mesh(
                    "bad index",
                    vec![[0.; 3], [1., 0., 0.]],
                    vec![vec![0, 1, 2]]
                )
                .map(|_| Value::Null)),
            "mesh"
        );
    }
    #[test]
    fn framed_circle_creation_uses_the_supplied_orthonormal_frame() {
        let limits = Limits::default();
        let document = Document::with_template_tolerance("frame circle");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut id = String::new();
        with_host(&mut staged, |host| {
            id = host
                .add_circle_on_frame("XZ circle", [1., 2., 3.], [1., 0., 0.], [0., 0., 1.], 4.)
                .unwrap();
            let info = host.curve_info(&id).unwrap();
            assert_eq!(info.kind, "circle");
            assert_eq!(info.domain, Some([0., std::f64::consts::TAU]));
            let start = host.evaluate_curve(&id, vec![0.0], 0).unwrap();
            assert_eq!(start[0].point, [5., 2., 3.]);
            let quarter = host
                .evaluate_curve(&id, vec![std::f64::consts::FRAC_PI_2], 0)
                .unwrap();
            assert!((quarter[0].point[0] - 1.0).abs() < 1e-9);
            assert!((quarter[0].point[2] - 7.0).abs() < 1e-9);
            let semantics = host.curve_semantics(&id, None).unwrap();
            assert!(semantics.closed && semantics.planar);
        });
        let run = staged.finish("frame circle".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert_eq!(
            coordinator.world().documents[&document.id].entities.len(),
            1
        );
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .add_circle_on_frame("flat", [0.; 3], [1., 0., 0.], [0., 1., 0.], 0.0)
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_circle_on_frame("skew", [0.; 3], [2., 0., 0.], [0., 1., 0.], 4.0)
                .map(|_| Value::Null)),
            "curve"
        );
    }
    #[test]
    fn pickable_needs_effective_visible_and_selectable() {
        let limits = Limits::default();
        let locked = Layer::new("locked", [1, 1, 1, 255]);
        let layer_id = locked.id;

        let mut document = Document::new("pickable");
        let entity = Entity::new("p", CoreGeometry::Point([0.; 3]), None);
        let id = entity.id;
        let mut placed = entity.clone();
        placed.attributes.layer = Some(layer_id);
        document.entities.insert(id, placed);
        document.layers.insert(layer_id, locked.clone());
        document.layer_order.push(layer_id);
        document.current_layer = Some(layer_id);

        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            // Locked blocks editing, not picking.
            assert!(host.is_object_pickable(id.to_string()).unwrap());
        });
        let run = staged.finish("pickable".into(), Value::Null).unwrap();
        assert!(run.request.is_none());

        let mut hidden_doc = document.clone();
        hidden_doc.entities.get_mut(&id).unwrap().attributes.hidden = true;
        let (coordinator2, doc2) = coordinator_with(hidden_doc.clone(), limits);
        let base2 = &coordinator2.world().documents[&doc2];
        let mut staged2 = Staged::new(&coordinator2, base2, &[], &limits).unwrap();
        with_host(&mut staged2, |host| {
            assert!(!host.is_object_pickable(id.to_string()).unwrap());
        });
        let run2 = staged2.finish("hidden".into(), Value::Null).unwrap();
        assert!(run2.request.is_none());

        let mut unselectable_doc = document.clone();
        unselectable_doc
            .entities
            .get_mut(&id)
            .unwrap()
            .attributes
            .selectable = false;
        let (coordinator3, doc3) = coordinator_with(unselectable_doc.clone(), limits);
        let base3 = &coordinator3.world().documents[&doc3];
        let mut staged3 = Staged::new(&coordinator3, base3, &[], &limits).unwrap();
        with_host(&mut staged3, |host| {
            assert!(!host.is_object_pickable(id.to_string()).unwrap());
        });
        let run3 = staged3.finish("unselectable".into(), Value::Null).unwrap();
        assert!(run3.request.is_none());
    }
    #[test]
    fn object_provenance_reports_the_source_chain() {
        use om9_core::{
            geometry::Geometry,
            identity::{ImportNamespaceId, SourceId},
            model::Entity,
            resource::{Resource, ResourceKind},
            retained::{
                HostRef, ImportBundle, ImportMode, ImportNamespace, NativeRecordKey,
                NativeRecordKind, NativeUnits, ResolvedBy, RetainedSource, SourceRecord,
                SourceUnits,
            },
        };
        let limits = Limits::default();
        let document = Document::new("import");
        let resource = Resource::from_bytes(
            "ring.3dm",
            ResourceKind::RetainedThreeDm { archive_version: 5 },
            b"3dm payload",
            &limits,
        )
        .unwrap();
        let units = NativeUnits {
            native_enum: 0,
            name: "Millimeters".into(),
            declared_mm_per_unit: None,
            millimeters_per_unit: 1.0,
            resolved_by: ResolvedBy::Explicit,
            absolute_tolerance: 0.001,
            angular_tolerance_radians: 0.0,
            relative_tolerance: 0.0,
        };
        let source = RetainedSource {
            id: SourceId::new(),
            resource: resource.id,
            sha256: resource.sha256(),
            archive_version: 5,
            units: SourceUnits {
                model: units.clone(),
                page: units,
                definitions: Default::default(),
            },
        };
        let entity = Entity::new("Imported stone", Geometry::Point([1., 2., 3.]), None);
        let object = entity.id;
        let record = SourceRecord::bound_entity(
            NativeRecordKey {
                kind: NativeRecordKind::Geometry,
                id: uuid::Uuid::new_v4(),
                native_index: Some(3),
            },
            "mesh",
            HostRef::Object(object),
            &entity,
        )
        .unwrap();
        let namespace = ImportNamespace {
            id: ImportNamespaceId::new(),
            source: source.id,
            import_mode: Some(ImportMode::Preserve),
            records: vec![record],
        };
        let bundle = ImportBundle {
            resource,
            source,
            namespace,
            entities: vec![entity.clone()],
            layers: vec![],
            definitions: vec![],
            groups: vec![],
            exchange: Default::default(),
        };
        let document_id = document.id;
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let prepared = coordinator
            .prepare(TransactionRequest::new(
                "Import",
                vec![Mutation::Import {
                    document: document_id,
                    expected_revision: 1,
                    bundle: Box::new(bundle),
                }],
            ))
            .unwrap();
        coordinator.commit(prepared).unwrap();
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let proof = host
                .object_provenance(object.to_string())
                .unwrap()
                .expect("complete source proof");
            assert_eq!(proof.source_name, "ring.3dm");
            assert_eq!(proof.source_sha256.len(), 64);
            assert_eq!(proof.archive_version, 5);
            assert_eq!(proof.import_mode.as_deref(), Some("preserve"));
            assert_eq!(proof.kind, "geometry");
            assert_eq!(proof.original_index, Some(3));
            assert_eq!(proof.original_family, "mesh");
            assert_eq!(proof.disposition, "unchanged");
        });
        let run = staged.finish("provenance".into(), Value::Null).unwrap();
        assert!(run.request.is_none());

        let mut plain_doc = Document::new("plain");
        let plain = Entity::new("plain", Geometry::Point([0.; 3]), None);
        let plain_id = plain.id;
        plain_doc.entities.insert(plain_id, plain);
        let (plain_coordinator, plain_doc_id) = coordinator_with(plain_doc.clone(), limits);
        let base2 = &plain_coordinator.world().documents[&plain_doc_id];
        let mut staged2 = Staged::new(&plain_coordinator, base2, &[], &limits).unwrap();
        with_host(&mut staged2, |host| {
            assert!(
                host.object_provenance(plain_id.to_string())
                    .unwrap()
                    .is_none()
            );
        });
        let run2 = staged2.finish("plain".into(), Value::Null).unwrap();
        assert!(run2.request.is_none());
    }
    #[test]
    fn document_tolerance_write_validates_and_undoes() {
        let limits = Limits::default();
        let document = Document::new("tolerance write");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.set_document_tolerance(0.01, 1e-6, 1e-8).unwrap();
        });
        let run = staged.finish("tolerance".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        {
            let updated = &coordinator.world().documents[&document.id];
            let tolerance = updated.effective_tolerance().expect("tolerance");
            assert_eq!(tolerance.absolute_mm, 0.01);
        }
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);
    }
    #[test]
    fn single_layer_info_matches_the_list() {
        let limits = Limits::default();
        let mut document = Document::new("layer info");
        let layer = Layer::new("L", [1, 2, 3, 255]);
        let layer_id = layer.id;
        document.layers.insert(layer_id, layer);
        document.layer_order.push(layer_id);
        document.current_layer = Some(layer_id);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let info = host.layer_info(layer_id.to_string()).unwrap();
            let listed = host.layers().unwrap();
            assert_eq!(listed.len(), 1);
            assert_eq!(info, listed[0]);
            assert!(info.current);
        });
        let run = staged.finish("layer info".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert_eq!(
            refusal(&document, &[], |host| {
                host.layer_info(uuid::Uuid::new_v4().to_string())
                    .map(|_| Value::Null)
            })
            .code,
            "layer"
        );
    }
    #[test]
    fn group_delete_removes_only_the_record_and_undoes() {
        let limits = Limits::default();
        let mut document = Document::new("group delete");
        let member = Entity::new("m", CoreGeometry::Point([0.; 3]), None);
        let member_id = member.id;
        document.entities.insert(member_id, member);
        let group = om9_core::model::Group::new("Set", vec![member_id]);
        let group_id = group.id;
        document.groups.insert(group_id, group);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.delete_group(group_id.to_string()).unwrap();
            assert!(host.groups().unwrap().is_empty());
        });
        let run = staged.finish("delete".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        {
            let updated = &coordinator.world().documents[&document.id];
            assert!(updated.groups.is_empty());
            assert_eq!(updated.entities.len(), 1);
        }
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .delete_group(uuid::Uuid::new_v4().to_string())
                .map(|()| Value::Null)),
            "missing"
        );
    }
    #[test]
    fn group_create_stages_one_group_and_undoes() {
        let limits = Limits::default();
        let mut document = Document::new("group write");
        let member = Entity::new("m", CoreGeometry::Point([0.; 3]), None);
        let member_id = member.id;
        document.entities.insert(member_id, member);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut group_id = String::new();
        with_host(&mut staged, |host| {
            group_id = host
                .create_group("Set", vec![member_id.to_string()])
                .unwrap();
            let groups = host.groups().unwrap();
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].name, "Set");
        });
        let run = staged.finish("group".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        {
            let updated = &coordinator.world().documents[&document.id];
            assert_eq!(updated.groups.len(), 1);
            let group = updated
                .groups
                .values()
                .find(|group| group.id.to_string() == group_id)
                .expect("group");
            assert_eq!(group.members, vec![member_id]);
        }
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .create_group("", vec![member_id.to_string()])
                .map(|_| Value::Null)),
            "group"
        );
        assert_eq!(
            code(&|host| host
                .create_group("Set", vec![uuid::Uuid::new_v4().to_string()])
                .map(|_| Value::Null)),
            "missing"
        );
    }
    #[test]
    fn object_info_reads_one_record() {
        let limits = Limits::default();
        let mut document = Document::new("info");
        let entity = Entity::new("tag", CoreGeometry::Point([1., 2., 3.]), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let info = host.object_info(id.to_string()).unwrap();
            assert_eq!(info.id, id.to_string());
            assert_eq!(info.label, "tag");
            assert_eq!(info.geometry_kind, "Point");
            assert_eq!(info.placement[3], [0.0, 0.0, 0.0, 1.0]);
        });
        let run = staged.finish("info".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
    }
    #[test]
    fn object_placement_reads_base_and_staged_matrix() {
        let limits = Limits::default();
        let mut document = Document::new("placement read");
        let entity = Entity::new("p", CoreGeometry::Point([0.; 3]), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut matrix = [[0.; 4]; 4];
        matrix[0][0] = 1.;
        matrix[1][1] = 1.;
        matrix[2][2] = 1.;
        matrix[3][3] = 1.;
        matrix[0][3] = 7.;
        with_host(&mut staged, |host| {
            let identity = host.object_placement(id.to_string()).unwrap();
            assert_eq!(identity[0][3], 0.0);
            host.set_object_placement(id.to_string(), matrix).unwrap();
            let staged_matrix = host.object_placement(id.to_string()).unwrap();
            assert_eq!(staged_matrix, matrix);
        });
        let run = staged.finish("placement read".into(), Value::Null).unwrap();
        assert!(run.request.is_some());
    }
    #[test]
    fn clear_current_layer_clears_only_when_one_is_set() {
        let limits = Limits::default();
        let mut document = Document::new("clear");
        let layer = Layer::new("A", [1, 2, 3, 255]);
        let layer_id = layer.id;
        document.layers.insert(layer_id, layer);
        document.layer_order.push(layer_id);
        document.current_layer = Some(layer_id);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.clear_current_layer().unwrap();
        });
        let run = staged.finish("clear".into(), Value::Null).unwrap();
        assert!(run.request.is_some());
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert_eq!(
            coordinator.world().documents[&document.id].current_layer,
            None
        );
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        // Clearing an already-clear current layer stages nothing.
        let no_current = {
            let mut d = document.clone();
            d.current_layer = None;
            d
        };
        let (coordinator2, _) = coordinator_with(no_current.clone(), limits);
        let base2 = &coordinator2.world().documents[&no_current.id];
        let mut staged = Staged::new(&coordinator2, base2, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.clear_current_layer().unwrap();
        });
        let run = staged.finish("clear".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
    }
    #[test]
    fn block_definitions_report_member_counts() {
        let limits = Limits::default();
        let mut document = Document::new("blocks");
        let member = Entity::new("m", CoreGeometry::Point([0.; 3]), None);
        let definition = om9_core::model::Definition::new("Ring", vec![member]);
        let definition_id = definition.id;
        document.definitions.insert(definition_id, definition);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let definitions = host.block_definitions().unwrap();
            assert_eq!(definitions.len(), 1);
            assert_eq!(definitions[0].id, definition_id.to_string());
            assert_eq!(definitions[0].label, "Ring");
            assert_eq!(definitions[0].members, 1);
            assert_eq!(definitions[0].revision, 1);
        });
        let run = staged.finish("blocks".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
    }
    #[test]
    fn groups_list_reports_members_read_only() {
        let limits = Limits::default();
        let mut document = Document::new("groups");
        let point = Entity::new("p", CoreGeometry::Point([0.; 3]), None);
        let point_id = point.id;
        document.entities.insert(point_id, point);
        let group = om9_core::model::Group::new("Jewelry set", vec![point_id]);
        let group_id = group.id;
        document.groups.insert(group_id, group);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let groups = host.groups().unwrap();
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].id, group_id.to_string());
            assert_eq!(groups[0].name, "Jewelry set");
            assert_eq!(groups[0].members, vec![point_id.to_string()]);
            assert_eq!(groups[0].revision, 1);
        });
        let run = staged.finish("groups".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
    }
    #[test]
    fn mesh_area_sums_triangle_fans() {
        let limits = Limits::default();
        let mut document = Document::new("area");
        let mesh = Entity::new(
            "quad",
            CoreGeometry::Mesh(om9_core::geometry::Mesh {
                vertices: vec![[0.; 3], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
                faces: vec![vec![0, 1, 2, 3]],
                normals: Vec::new(),
                colors: Vec::new(),

                corner_uvs: Vec::new(),
            }),
            None,
        );
        let mesh_id = mesh.id;
        document.entities.insert(mesh_id, mesh);
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let area = host.measure_mesh_area(mesh_id.to_string()).unwrap();
            assert!((area.square_millimeters - 100.0).abs() < 1e-9);
        });
        let run = staged.finish("area".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert_eq!(
            refusal(&document, &[], |host| {
                host.measure_mesh_area(line_id.to_string())
                    .map(|_| Value::Null)
            })
            .code,
            "unsupported"
        );
    }
    #[test]
    fn layer_order_lists_base_and_created_layers() {
        let limits = Limits::default();
        let mut document = Document::new("order");
        let first = Layer::new("A", [1, 2, 3, 255]);
        let second = Layer::new("B", [4, 5, 6, 255]);
        let (first_id, second_id) = (first.id, second.id);
        document.layers.insert(first_id, first);
        document.layers.insert(second_id, second);
        document.layer_order.extend([first_id, second_id]);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let order = host.layer_order().unwrap();
            assert_eq!(order, vec![first_id.to_string(), second_id.to_string()]);
            let created = host.layer_create("C", [7, 8, 9, 255], false).unwrap();
            let order = host.layer_order().unwrap();
            assert_eq!(order.last().map(String::as_str), Some(created.as_str()));
            assert_eq!(order.len(), 3);
        });
        let run = staged.finish("order".into(), Value::Null).unwrap();
        assert!(run.request.is_some());
    }
    #[test]
    fn measurement_label_is_one_bounded_progress_message() {
        let limits = Limits::default();
        let document = Document::new("label");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.report_measurement("length 10.00 mm").unwrap();
        });
        let run = staged.finish("label".into(), Value::Null).unwrap();
        assert_eq!(run.messages.len(), 1);
        assert_eq!(run.messages[0].text, "length 10.00 mm");
        assert!(matches!(
            run.messages[0].level,
            om9_sdk::MessageLevel::Progress
        ));
        assert_eq!(
            refusal(&document, &[], |host| {
                host.report_measurement("").map(|()| Value::Null)
            })
            .code,
            "message"
        );
    }
    #[test]
    fn object_summary_counts_staged_state_by_geometry_kind() {
        let limits = Limits::default();
        let mut document = Document::new("summary");
        let point = Entity::new("point", CoreGeometry::Point([0.; 3]), None);
        document.entities.insert(point.id, point);
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let before = host.object_summary().unwrap();
            assert_eq!(before.total, 2);
            assert_eq!(before.by_kind["Point"], 1);
            assert_eq!(before.by_kind["Curve"], 1);
            let mesh = host
                .add_mesh(
                    "quad",
                    vec![[0.; 3], [1., 0., 0.], [1., 1., 0.]],
                    vec![vec![0, 1, 2]],
                )
                .unwrap();
            host.delete_object(line_id.to_string()).unwrap();
            let after = host.object_summary().unwrap();
            assert_eq!(after.total, 2);
            assert_eq!(after.by_kind["Mesh"], 1);
            assert_eq!(after.by_kind.get("Curve").copied().unwrap_or(0), 0);
            assert!(!mesh.is_empty());
        });
        let run = staged.finish("summary".into(), Value::Null).unwrap();
        assert!(run.request.is_some());
    }
    #[test]
    fn circle_radius_is_measured_or_refused() {
        let limits = Limits::default();
        let mut document = Document::new("radius");
        let circle = Entity::new(
            "circle",
            CoreGeometry::Curve(Curve::Circle {
                center: [1., 2., 3.],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 5.,
                domain: [0., std::f64::consts::TAU],
                angle_domain: Some([0., std::f64::consts::PI]),
            }),
            None,
        );
        let circle_id = circle.id;
        document.entities.insert(circle_id, circle);
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let radius = host.measure_curve_radius(circle_id.to_string()).unwrap();
            assert_eq!(radius.millimeters, 5.0);
        });
        let run = staged.finish("radius".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert_eq!(
            refusal(&document, &[], |host| {
                host.measure_curve_radius(line_id.to_string())
                    .map(|_| Value::Null)
            })
            .code,
            "unsupported"
        );
    }
    #[test]
    fn curve_length_is_a_sampled_measurement() {
        let limits = Limits::default();
        let mut document = Document::new("length");
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let circle = Entity::new(
            "circle",
            CoreGeometry::Curve(Curve::Circle {
                center: [0.; 3],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 5.,
                domain: [0., std::f64::consts::TAU],
                angle_domain: None,
            }),
            None,
        );
        let circle_id = circle.id;
        document.entities.insert(circle_id, circle);
        let point = Entity::new("point", CoreGeometry::Point([0.; 3]), None);
        let point_id = point.id;
        document.entities.insert(point_id, point);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let line = host.measure_curve_length(line_id.to_string()).unwrap();
            assert!((line.millimeters - 10.0).abs() < 1e-9);
            let circle = host.measure_curve_length(circle_id.to_string()).unwrap();
            let exact = 2.0 * std::f64::consts::PI * 5.0;
            assert!((circle.millimeters - exact).abs() / exact < 0.01);
        });
        let run = staged.finish("length".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert_eq!(
            refusal(&document, &[], |host| {
                host.measure_curve_length(point_id.to_string())
                    .map(|_| Value::Null)
            })
            .code,
            "unsupported"
        );
    }
    #[test]
    fn object_bounds_measure_points_curves_and_meshes() {
        let limits = Limits::default();
        let mut document = Document::new("bounds");
        let point = Entity::new("point", CoreGeometry::Point([2., 3., 4.]), None);
        let point_id = point.id;
        document.entities.insert(point_id, point);
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let mesh = Entity::new(
            "mesh",
            CoreGeometry::Mesh(om9_core::geometry::Mesh {
                vertices: vec![[0.; 3], [4., 0., 0.], [4., 2., 0.], [0., 2., 0.]],
                faces: vec![vec![0, 1, 2, 3]],
                normals: Vec::new(),
                colors: Vec::new(),

                corner_uvs: Vec::new(),
            }),
            None,
        );
        let mesh_id = mesh.id;
        document.entities.insert(mesh_id, mesh);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let bounds = host.object_bounds(point_id.to_string()).unwrap();
            assert_eq!(bounds.min, [2., 3., 4.]);
            assert_eq!(bounds.max, [2., 3., 4.]);
            assert_eq!(bounds.size, [0., 0., 0.]);
            assert_eq!(bounds.center, [2., 3., 4.]);
            let bounds = host.object_bounds(line_id.to_string()).unwrap();
            assert_eq!(bounds.min, [0., 0., 0.]);
            assert_eq!(bounds.max, [10., 0., 0.]);
            assert_eq!(bounds.size, [10., 0., 0.]);
            assert_eq!(bounds.center, [5., 0., 0.]);
            let bounds = host.object_bounds(mesh_id.to_string()).unwrap();
            assert_eq!(bounds.size, [4., 2., 0.]);
            assert_eq!(bounds.center, [2., 1., 0.]);
        });
        let run = staged.finish("bounds".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
    }
    #[test]
    fn curve_segments_address_polycurve_segments_for_evaluation() {
        let limits = Limits::default();
        let mut document = Document::new("segments");
        let rectangle = Entity::new(
            "rect",
            CoreGeometry::Curve(
                om9_core::curve_build::rectangle([0.; 3], [1., 0., 0.], [0., 1., 0.], 10., 5.)
                    .expect("rectangle"),
            ),
            None,
        );
        let id = rectangle.id;
        document.entities.insert(id, rectangle);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let segments = host.curve_segments(id.to_string()).unwrap();
            assert_eq!(segments.len(), 4);
            assert!(segments.iter().all(|s| s.kind == "line"));
            assert_eq!(segments[0].domain, Some([0., 1.]));
            let sample = host
                .evaluate_curve_segment(id.to_string(), 0, vec![0.5], 1)
                .unwrap();
            assert_eq!(sample[0].point, [5., 0., 0.]);
            assert_eq!(sample[0].derivatives, vec![[10., 0., 0.]]);
        });
        let run = staged.finish("segments".into(), Value::Null).unwrap();
        assert!(run.request.is_none());

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .evaluate_curve_segment(id.to_string(), 4, vec![0.5], 0)
                .map(|_| Value::Null)),
            "curve"
        );
    }
    #[test]
    fn last_change_reports_the_recorded_batch_read_only() {
        let limits = Limits::default();
        let document = Document::new("change feed");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            assert!(host.last_change().unwrap().is_none());
        });
        let run = staged.finish("change".into(), Value::Null).unwrap();
        assert!(run.request.is_none());

        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        staged.set_last_change(Some(ProjectChange {
            from_revision: 2,
            to_revision: 3,
            cause: "edit".into(),
            changes: vec![om9_sdk::ChangeEntry {
                kind: "entity_added".into(),
                id: Some("00000000-0000-4000-8000-000000000001".into()),
            }],
        }));
        with_host(&mut staged, |host| {
            let change = host.last_change().unwrap().expect("change");
            assert_eq!(change.from_revision, 2);
            assert_eq!(change.to_revision, 3);
            assert_eq!(change.changes.len(), 1);
            assert_eq!(change.changes[0].kind, "entity_added");
        });
        let run = staged.finish("change".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
    }
    #[test]
    fn placement_write_validates_the_affine_matrix_and_undoes() {
        let limits = Limits::default();
        let mut document = Document::new("placement");
        let entity = Entity::new("moved", CoreGeometry::Point([0.; 3]), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut matrix = [[0.; 4]; 4];
        matrix[0][0] = 1.;
        matrix[1][1] = 1.;
        matrix[2][2] = 1.;
        matrix[3][3] = 1.;
        matrix[0][3] = 10.;
        with_host(&mut staged, |host| {
            host.set_object_placement(id.to_string(), matrix).unwrap();
        });
        let run = staged.finish("placement".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert_eq!(
            coordinator.world().documents[&document.id].entities[&id]
                .placement
                .0,
            matrix
        );
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        let mut bad_row = matrix;
        bad_row[3] = [0., 0., 0., 0.];
        assert_eq!(
            code(&|host| host
                .set_object_placement(id.to_string(), bad_row)
                .map(|()| Value::Null)),
            "placement"
        );
        let mut singular = matrix;
        singular[0] = [0., 0., 0., 0.];
        assert_eq!(
            code(&|host| host
                .set_object_placement(id.to_string(), singular)
                .map(|()| Value::Null)),
            "placement"
        );

        // A created object carries its placement through the insert instead.
        let mut created = Staged::new(
            &coordinator,
            &coordinator.world().documents[&document.id],
            &[],
            &limits,
        )
        .unwrap();
        with_host(&mut created, |host| {
            let new_id = host.add_point("new", [0.; 3]).unwrap();
            let error = host.set_object_placement(new_id, matrix).unwrap_err();
            assert_eq!(error.code, "placement");
        });

        // One source write per object per command still applies.
        let mut conflict = Staged::new(
            &coordinator,
            &coordinator.world().documents[&document.id],
            &[],
            &limits,
        )
        .unwrap();
        with_host(&mut conflict, |host| {
            host.set_object_placement(id.to_string(), matrix).unwrap();
            let error = host
                .set_object_attributes(
                    id.to_string(),
                    AttributesPatch {
                        color: None,
                        visible: None,
                        hidden: Some(false),
                        selectable: Some(true),
                        locked: None,
                    },
                )
                .unwrap_err();
            assert_eq!(error.code, "conflict");
        });
    }
    #[test]
    fn object_extensions_round_trip_clear_and_undo() {
        let limits = Limits::default();
        let mut document = Document::new("extensions");
        let entity = Entity::new("tagged", CoreGeometry::Point([0.; 3]), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            assert!(host.object_extensions(id.to_string()).unwrap().is_empty());
            host.set_object_extension(
                id.to_string(),
                "org.openmatrix9.demo",
                1,
                "{\"weight_g\":4.2}",
            )
            .unwrap();
            let slots = host.object_extensions(id.to_string()).unwrap();
            assert_eq!(slots["org.openmatrix9.demo"].version, 1);
            assert_eq!(slots["org.openmatrix9.demo"].payload, "{\"weight_g\":4.2}");
        });
        let run = staged.finish("extension set".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        {
            let updated = &coordinator.world().documents[&document.id];
            let extension = &updated.entities[&id].extensions["org.openmatrix9.demo"];
            assert_eq!(extension.version, 1);
            assert_eq!(extension.payload, "{\"weight_g\":4.2}");
        }
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        // Clearing removes the slot and undo restores it.
        let mut set_again = Staged::new(
            &coordinator,
            &coordinator.world().documents[&document.id],
            &[],
            &limits,
        )
        .unwrap();
        with_host(&mut set_again, |host| {
            host.set_object_extension(id.to_string(), "org.openmatrix9.demo", 1, "{\"a\":1}")
                .unwrap();
        });
        let run = set_again
            .finish("extension set again".into(), Value::Null)
            .unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let with_slot = coordinator.world().documents[&document.id].clone();
        let mut clear = Staged::new(
            &coordinator,
            &coordinator.world().documents[&document.id],
            &[],
            &limits,
        )
        .unwrap();
        with_host(&mut clear, |host| {
            host.clear_object_extension(id.to_string(), "org.openmatrix9.demo")
                .unwrap();
            assert!(host.object_extensions(id.to_string()).unwrap().is_empty());
        });
        let run = clear.finish("extension clear".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert!(
            coordinator.world().documents[&document.id].entities[&id]
                .extensions
                .is_empty()
        );
        coordinator.undo().unwrap();
        assert_eq!(coordinator.world().documents[&document.id], with_slot);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .set_object_extension(id.to_string(), "slot", 1, "not json")
                .map(|()| Value::Null)),
            "extension"
        );
        assert_eq!(
            code(&|host| host
                .set_object_extension(id.to_string(), "slot", 1, "{}")
                .map(|()| Value::Null)),
            "extension"
        );
        assert_eq!(
            code(&|host| host
                .set_object_extension(id.to_string(), "slot", 0, "{\"a\":1}")
                .map(|()| Value::Null)),
            "extension"
        );
        assert_eq!(
            code(&|host| host
                .set_object_extension(id.to_string(), "s".repeat(65), 1, "{\"a\":1}")
                .map(|()| Value::Null)),
            "extension"
        );
        assert_eq!(
            code(&|host| host
                .set_object_extension(uuid::Uuid::new_v4().to_string(), "slot", 1, "{\"a\":1}")
                .map(|()| Value::Null)),
            "missing"
        );

        // One source write per object per command still applies.
        let mut conflict = Staged::new(
            &coordinator,
            &coordinator.world().documents[&document.id],
            &[],
            &limits,
        )
        .unwrap();
        with_host(&mut conflict, |host| {
            host.set_object_extension(id.to_string(), "slot", 1, "{\"a\":1}")
                .unwrap();
            let error = host
                .set_object_attributes(
                    id.to_string(),
                    AttributesPatch {
                        color: None,
                        visible: None,
                        hidden: Some(false),
                        selectable: Some(true),
                        locked: None,
                    },
                )
                .unwrap_err();
            assert_eq!(error.code, "conflict");
        });
    }
    #[test]
    fn project_state_is_host_owned_and_save_is_staged_after_commit() {
        let limits = Limits::default();
        let document = Document::new("project");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];

        // An untitled project has no path, is clean, and cannot be saved.
        let mut untitled = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut untitled, |host| {
            let info = host.project_state().unwrap();
            assert_eq!(info.path, None);
            assert!(!info.dirty);
        });
        assert_eq!(
            refusal(&document, &[], |host| {
                host.save_project().map(|()| Value::Null)
            })
            .code,
            "save"
        );

        // With a host snapshot the plugin reads the path/dirty state and stages a save.
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        staged.set_project(ProjectState {
            path: Some(std::path::PathBuf::from("C:/tmp/ring.om9")),
            dirty: true,
        });
        with_host(&mut staged, |host| {
            let info = host.project_state().unwrap();
            assert_eq!(info.path.as_deref(), Some("C:/tmp/ring.om9"));
            assert!(info.dirty);
            host.save_project().unwrap();
        });
        let run = staged.finish("project".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert!(run.save);
    }
    #[test]
    fn polygon_creation_closes_once_and_refuses_degenerate_input() {
        let limits = Limits::default();
        let document = Document::with_template_tolerance("polygon");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut open_id = String::new();
        let mut closed_id = String::new();
        with_host(&mut staged, |host| {
            open_id = host
                .add_polygon(
                    "square",
                    vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
                )
                .unwrap();
            closed_id = host
                .add_polygon(
                    "square closed by caller",
                    vec![
                        [0., 0., 0.],
                        [10., 0., 0.],
                        [10., 10., 0.],
                        [0., 10., 0.],
                        [0., 0., 0.],
                    ],
                )
                .unwrap();
            for id in [&open_id, &closed_id] {
                let info = host.curve_info(id).unwrap();
                assert_eq!(info.kind, "polycurve");
                assert_eq!(info.segments, 4);
                let semantics = host.curve_semantics(id, None).unwrap();
                assert!(semantics.closed && semantics.planar);
            }
        });
        let run = staged.finish("polygon".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        assert_eq!(
            coordinator.world().documents[&document.id].entities.len(),
            2
        );
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .add_polygon("two", vec![[0., 0., 0.], [1., 0., 0.]])
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_polygon(
                    "duplicate",
                    vec![[0., 0., 0.], [1., 0., 0.], [1., 0., 0.], [0., 1., 0.]]
                )
                .map(|_| Value::Null)),
            "curve"
        );
    }
    #[test]
    fn ellipse_and_rectangle_construction_use_the_owner_frames() {
        let limits = Limits::default();
        let document = Document::with_template_tolerance("frames");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut ellipse_id = String::new();
        let mut rectangle_id = String::new();
        with_host(&mut staged, |host| {
            ellipse_id = host
                .add_ellipse("ellipse", [0.; 3], [1., 0., 0.], [0., 1., 0.], 5.0, 2.0)
                .unwrap();
            let info = host.curve_info(&ellipse_id).unwrap();
            assert_eq!(info.kind, "nurbs");
            let sample = host.evaluate_curve(&ellipse_id, vec![0.25], 0).unwrap();
            assert!(sample[0].point[0].abs() < 1e-9);
            assert!((sample[0].point[1] - 2.0).abs() < 1e-9);
            let semantics = host.curve_semantics(&ellipse_id, None).unwrap();
            assert!(semantics.closed && semantics.planar);

            rectangle_id = host
                .add_rectangle("rect", [0.; 3], [1., 0., 0.], [0., 1., 0.], 10.0, 5.0)
                .unwrap();
            let info = host.curve_info(&rectangle_id).unwrap();
            assert_eq!(info.kind, "polycurve");
            assert_eq!(info.segments, 4);
            let semantics = host.curve_semantics(&rectangle_id, None).unwrap();
            assert!(semantics.closed && semantics.planar);
        });
        let run = staged.finish("frames".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let updated = &coordinator.world().documents[&document.id];
        assert_eq!(updated.entities.len(), 2);
        let ellipse = ObjectId(uuid::Uuid::parse_str(&ellipse_id).unwrap());
        assert!(matches!(
            updated.entities[&ellipse].geometry,
            CoreGeometry::Curve(Curve::Nurbs(_))
        ));
        let rectangle = ObjectId(uuid::Uuid::parse_str(&rectangle_id).unwrap());
        assert!(matches!(
            updated.entities[&rectangle].geometry,
            CoreGeometry::Curve(Curve::Polycurve { .. })
        ));
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .add_ellipse("frame", [0.; 3], [2., 0., 0.], [0., 1., 0.], 5.0, 2.0)
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_ellipse("radius", [0.; 3], [1., 0., 0.], [0., 1., 0.], 0.0, 2.0)
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_rectangle("size", [0.; 3], [1., 0., 0.], [0., 1., 0.], 10.0, 0.0)
                .map(|_| Value::Null)),
            "curve"
        );
    }
    #[test]
    fn measurement_services_use_the_analysis_rules() {
        let limits = Limits::default();
        let document = Document::new("measures");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let distance = host.measure_distance([0., 0., 0.], [3., 4., 0.]).unwrap();
            assert_eq!(distance.millimeters, 5.0);
            let angle = host
                .measure_angle([0., 0., 0.], [1., 0., 0.], [0., 0., 0.], [0., 1., 0.])
                .unwrap();
            assert!((angle.degrees - 90.0).abs() < 1e-12);
            let parallel = host
                .measure_angle([0., 0., 0.], [1., 0., 0.], [0., 0., 0.], [2., 0., 0.])
                .unwrap();
            assert!(parallel.degrees.abs() < 1e-12);
            let opposite = host
                .measure_angle([0., 0., 0.], [1., 0., 0.], [0., 0., 0.], [-1., 0., 0.])
                .unwrap();
            assert!((opposite.degrees - 180.0).abs() < 1e-9);
            let diagonal = host
                .measure_angle([0., 0., 0.], [1., 0., 0.], [0., 0., 0.], [1., 1., 0.])
                .unwrap();
            assert!((diagonal.degrees - 45.0).abs() < 1e-9);
        });
        let run = staged.finish("measure".into(), Value::Null).unwrap();
        assert!(run.request.is_none());

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        // A non-finite coordinate cannot cross the JSON ABI, so the admitted bound is
        // exercised with a finite value beyond it.
        assert_eq!(
            code(&|host| host
                .measure_distance([0.; 3], [1.0e10, 0., 0.])
                .map(|_| Value::Null)),
            "measure"
        );
        assert_eq!(
            code(&|host| host
                .measure_distance([0.; 3], [1.0e10, 0., 0.])
                .map(|_| Value::Null)),
            "measure"
        );
        assert_eq!(
            code(&|host| host
                .measure_angle([0.; 3], [0.; 3], [0.; 3], [1., 0., 0.])
                .map(|_| Value::Null)),
            "measure"
        );
    }
    #[test]
    fn control_point_nurbs_creation_validates_and_undoes_its_record() {
        let limits = Limits::default();
        let document = Document::new("nurbs build");
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut id = String::new();
        with_host(&mut staged, |host| {
            id = host
                .add_nurbs_curve(
                    "nurbs",
                    3,
                    vec![[0., 0., 0.], [1., 4., 0.], [3., 4., 0.], [4., 0., 0.]],
                    vec![1.; 4],
                    vec![0., 0., 0., 0., 1., 1., 1., 1.],
                )
                .unwrap();
            let definition = host.curve_definition(&id).unwrap();
            assert_eq!(definition.kind, "nurbs");
            assert_eq!(definition.degree, Some(3));
            assert_eq!(definition.domain, Some([0., 1.]));
            assert_eq!(definition.poles.as_ref().map(Vec::len), Some(4));
            let samples = host.evaluate_curve(&id, vec![0.0, 1.0], 0).unwrap();
            assert_eq!(samples[0].point, [0., 0., 0.]);
            assert_eq!(samples[1].point, [4., 0., 0.]);
        });
        let run = staged.finish("nurbs build".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let updated = &coordinator.world().documents[&document.id];
        let object = ObjectId(uuid::Uuid::parse_str(&id).unwrap());
        assert!(matches!(
            updated.entities[&object].geometry,
            CoreGeometry::Curve(Curve::Nurbs(_))
        ));
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .add_nurbs_curve(
                    "weights",
                    3,
                    vec![[0.; 3], [1., 4., 0.], [3., 4., 0.], [4., 0., 0.]],
                    vec![1.; 3],
                    vec![0., 0., 0., 0., 1., 1., 1., 1.]
                )
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_nurbs_curve(
                    "degree",
                    0,
                    vec![[0.; 3], [1., 0., 0.]],
                    vec![1.; 2],
                    vec![0., 1.]
                )
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_nurbs_curve(
                    "knots",
                    3,
                    vec![[0.; 3], [1., 4., 0.], [3., 4., 0.], [4., 0., 0.]],
                    vec![1.; 4],
                    vec![0., 0., 0., 0., 1., 1., 1.]
                )
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_nurbs_curve(
                    "weight value",
                    3,
                    vec![[0.; 3], [1., 4., 0.], [3., 4., 0.], [4., 0., 0.]],
                    vec![1., 1., -1., 1.],
                    vec![0., 0., 0., 0., 1., 1., 1., 1.]
                )
                .map(|_| Value::Null)),
            "curve"
        );
    }
    #[test]
    fn curve_construction_builds_one_transaction_output_on_the_current_layer() {
        let limits = Limits::default();
        let mut document = Document::new("curve build");
        let layer = Layer::new("Curves", [1, 2, 3, 255]);
        let layer_id = layer.id;
        document.layers.insert(layer_id, layer);
        document.layer_order.push(layer_id);
        document.current_layer = Some(layer_id);
        let (mut coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        let mut arc_id = String::new();
        let mut spline_id = String::new();
        with_host(&mut staged, |host| {
            arc_id = host
                .add_arc("arc", [5., 0., 0.], [0., 5., 0.], [-5., 0., 0.])
                .unwrap();
            spline_id = host
                .add_interpolated_curve(
                    "spline",
                    vec![[0., 0., 0.], [1., 5., 0.], [3., 5., 0.], [4., 0., 0.]],
                    3,
                    KnotMode::ChordLength,
                )
                .unwrap();
            let info = host.curve_info(&arc_id).unwrap();
            assert_eq!(info.kind, "circle");
            assert_eq!(info.domain, Some([0., 1.]));
            let midpoint = host.evaluate_curve(&arc_id, vec![0.5], 0).unwrap();
            assert!(midpoint[0].point[0].abs() < 1e-12);
            assert!((midpoint[0].point[1] - 5.0).abs() < 1e-12);
            let definition = host.curve_definition(&spline_id).unwrap();
            assert_eq!(definition.kind, "nurbs");
            assert_eq!(definition.degree, Some(3));
            assert_eq!(definition.poles.as_ref().map(Vec::len), Some(4));
            let samples = host.evaluate_curve(&spline_id, vec![0.0, 1.0], 0).unwrap();
            assert_eq!(samples[0].point, [0., 0., 0.]);
            assert_eq!(samples[1].point, [4., 0., 0.]);
            let attributes = host.object_attributes(&arc_id).unwrap();
            assert_eq!(attributes.layer, Some(layer_id.to_string()));
        });
        let run = staged.finish("curve build".into(), Value::Null).unwrap();
        let prepared = coordinator.prepare(run.request.unwrap()).unwrap();
        coordinator.commit(prepared).unwrap();
        let updated = &coordinator.world().documents[&document.id];
        assert_eq!(updated.entities.len(), 2);
        assert!(
            updated
                .entities
                .values()
                .all(|entity| entity.attributes.layer == Some(layer_id))
        );
        let arc = ObjectId(uuid::Uuid::parse_str(&arc_id).unwrap());
        assert!(matches!(
            updated.entities[&arc].geometry,
            CoreGeometry::Curve(Curve::Circle {
                angle_domain: Some(_),
                ..
            })
        ));
        let spline = ObjectId(uuid::Uuid::parse_str(&spline_id).unwrap());
        assert!(matches!(
            updated.entities[&spline].geometry,
            CoreGeometry::Curve(Curve::Nurbs(_))
        ));
        assert_eq!(coordinator.undo_len(), 1);
        coordinator.undo().unwrap();
        assert_eq!(*coordinator.world().documents[&document.id], document);

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host
                .add_arc("flat", [0., 0., 0.], [1., 0., 0.], [2., 0., 0.])
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_interpolated_curve(
                    "even",
                    vec![[0., 0., 0.], [1., 1., 0.], [2., 0., 0.], [3., 1., 0.]],
                    2,
                    KnotMode::Uniform
                )
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .add_arc(
                    "x".repeat(17 * 1024),
                    [0., 0., 0.],
                    [1., 1., 0.],
                    [2., 0., 0.]
                )
                .map(|_| Value::Null)),
            "budget"
        );
    }
    #[test]
    fn curve_definition_and_semantics_use_the_owner_rules() {
        let limits = Limits::default();
        let mut document = Document::with_template_tolerance("curve semantics");
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let circle = Entity::new(
            "circle",
            CoreGeometry::Curve(Curve::Circle {
                center: [0.; 3],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 5.,
                domain: [0., std::f64::consts::TAU],
                angle_domain: Some([0., std::f64::consts::FRAC_PI_2]),
            }),
            None,
        );
        let circle_id = circle.id;
        document.entities.insert(circle_id, circle);
        let closed = Entity::new(
            "closed",
            CoreGeometry::Curve(
                om9_core::curve_build::polyline(&[[0.; 3], [10., 0., 0.], [10., 10., 0.], [0.; 3]])
                    .expect("closed polyline"),
            ),
            None,
        );
        let closed_id = closed.id;
        document.entities.insert(closed_id, closed);
        let nurbs = Entity::new(
            "nurbs",
            CoreGeometry::Curve(Curve::Nurbs(om9_core::geometry::NurbsCurve {
                degree: 1,
                poles: vec![[0.; 3], [10., 0., 0.]],
                weights: vec![1., 1.],
                knots: vec![0., 0., 1., 1.],
                domain: [0., 1.],
                periodic: false,
                reversed: true,
                seam: Some(0.5),
            })),
            None,
        );
        let nurbs_id = nurbs.id;
        document.entities.insert(nurbs_id, nurbs);

        let point = Entity::new("point", CoreGeometry::Point([0.; 3]), None);
        let point_id = point.id;
        document.entities.insert(point_id, point);

        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let definition = host.curve_definition(nurbs_id.to_string()).unwrap();
            assert_eq!(definition.kind, "nurbs");
            assert_eq!(definition.degree, Some(1));
            assert_eq!(
                definition.poles.as_deref(),
                Some(&[[0., 0., 0.], [10., 0., 0.]][..])
            );
            assert_eq!(definition.weights.as_deref(), Some(&[1.0, 1.0][..]));
            assert_eq!(definition.knots.as_deref(), Some(&[0., 0., 1., 1.][..]));
            assert_eq!(definition.periodic, Some(false));
            assert!(definition.reversed);
            assert_eq!(definition.seam, Some(0.5));
            let line = host.curve_definition(line_id.to_string()).unwrap();
            assert_eq!(line.kind, "line");
            assert!(line.poles.is_none());
            let circle = host.curve_definition(circle_id.to_string()).unwrap();
            assert_eq!(circle.angle_domain, Some([0., std::f64::consts::FRAC_PI_2]));

            // The document states the template tolerance, so no default is invented.
            let semantics = host.curve_semantics(line_id.to_string(), None).unwrap();
            assert_eq!(semantics.kind, "line");
            assert!(!semantics.closed);
            assert!(semantics.planar);
            assert_eq!(semantics.tolerance_mm, 0.001);
            assert_eq!(semantics.endpoints, [[0., 0., 0.], [10., 0., 0.]]);
            // The quarter arc is open but planar; the closed polyline is closed and planar.
            let semantics = host.curve_semantics(circle_id.to_string(), None).unwrap();
            assert_eq!(semantics.kind, "circle");
            assert!(!semantics.closed);
            assert!(semantics.planar);
            let semantics = host.curve_semantics(closed_id.to_string(), None).unwrap();
            assert_eq!(semantics.kind, "polyline");
            assert!(semantics.closed && semantics.planar);
            // An explicit tolerance is reported back.
            let semantics = host
                .curve_semantics(closed_id.to_string(), Some(1.0e-6))
                .unwrap();
            assert!(semantics.closed);
            assert_eq!(semantics.tolerance_mm, 1.0e-6);
        });
        let run = staged
            .finish("curve definitions".into(), Value::Null)
            .unwrap();
        assert!(run.request.is_none());

        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        // A curve shorter than the stated tolerance is refused as degenerate.
        assert_eq!(
            code(&|host| host
                .curve_semantics(line_id.to_string(), Some(100.0))
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .curve_semantics(line_id.to_string(), Some(-1.0))
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .curve_definition(point_id.to_string())
                .map(|_| Value::Null)),
            "unsupported"
        );

        // A document that states no tolerance refuses the implicit measurement.
        let mut plain = Document::new("no tolerance");
        let entity = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let plain_id = entity.id;
        plain.entities.insert(plain_id, entity);
        assert_eq!(
            refusal(&plain, &[], |host| {
                host.curve_semantics(plain_id.to_string(), None)
                    .map(|_| Value::Null)
            })
            .code,
            "tolerance"
        );
    }
    #[test]
    fn curve_information_and_evaluation_are_object_local() {
        let limits = Limits::default();
        let mut document = Document::new("curves");
        let line = Entity::new(
            "line",
            CoreGeometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [10., 0., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        let line_id = line.id;
        document.entities.insert(line_id, line);
        let circle = Entity::new(
            "circle",
            CoreGeometry::Curve(Curve::Circle {
                center: [1., 2., 3.],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 5.,
                domain: [0., std::f64::consts::TAU],
                angle_domain: None,
            }),
            None,
        );
        let circle_id = circle.id;
        document.entities.insert(circle_id, circle);
        let nurbs = Entity::new(
            "nurbs",
            CoreGeometry::Curve(Curve::Nurbs(om9_core::geometry::NurbsCurve {
                degree: 1,
                poles: vec![[0.; 3], [10., 0., 0.]],
                weights: vec![1., 1.],
                knots: vec![0., 0., 1., 1.],
                domain: [0., 1.],
                periodic: false,
                reversed: false,
                seam: None,
            })),
            None,
        );
        let nurbs_id = nurbs.id;
        document.entities.insert(nurbs_id, nurbs);
        let point = Entity::new("point", CoreGeometry::Point([0.; 3]), None);
        let point_id = point.id;
        document.entities.insert(point_id, point);
        let polycurve = Entity::new(
            "polyline",
            CoreGeometry::Curve(
                om9_core::curve_build::polyline(&[[0.; 3], [1., 0., 0.], [1., 1., 0.]])
                    .expect("polyline"),
            ),
            None,
        );
        let polycurve_id = polycurve.id;
        document.entities.insert(polycurve_id, polycurve);

        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let info = host.curve_info(line_id.to_string()).unwrap();
            assert_eq!(info.kind, "line");
            assert_eq!(info.domain, Some([0.0, 10.0]));
            let samples = host
                .evaluate_curve(line_id.to_string(), vec![0.0, 5.0, 10.0], 1)
                .unwrap();
            assert_eq!(samples[1].point, [5., 0., 0.]);
            assert_eq!(samples[1].derivatives, vec![[1., 0., 0.]]);
            let right = host
                .evaluate_curve(circle_id.to_string(), vec![0.0], 1)
                .unwrap();
            assert_eq!(right[0].point, [6., 2., 3.]);
            assert_eq!(right[0].derivatives, vec![[0., 5., 0.]]);
            let info = host.curve_info(nurbs_id.to_string()).unwrap();
            assert_eq!(info.kind, "nurbs");
            assert_eq!(info.degree, Some(1));
            assert_eq!(info.poles, Some(2));
            assert_eq!(info.periodic, Some(false));
            let samples = host
                .evaluate_curve(nurbs_id.to_string(), vec![0.5], 0)
                .unwrap();
            assert_eq!(samples[0].point, [5., 0., 0.]);
            assert!(samples[0].derivatives.is_empty());
            let info = host.curve_info(polycurve_id.to_string()).unwrap();
            assert_eq!(info.kind, "polycurve");
            assert_eq!(info.domain, None);
            assert!(info.segments > 0);
        });
        let run = staged.finish("curves".into(), Value::Null).unwrap();
        assert!(run.request.is_none());

        // Refusals are typed and leave the document alone.
        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host.curve_info(point_id.to_string()).map(|_| Value::Null)),
            "unsupported"
        );
        assert_eq!(
            code(&|host| host
                .evaluate_curve(line_id.to_string(), vec![11.0], 0)
                .map(|_| Value::Null)),
            "curve"
        );
        assert_eq!(
            code(&|host| host
                .evaluate_curve(line_id.to_string(), vec![5.0], 4)
                .map(|_| Value::Null)),
            "unsupported"
        );
        assert_eq!(
            code(&|host| host
                .evaluate_curve(polycurve_id.to_string(), vec![0.0], 0)
                .map(|_| Value::Null)),
            "unsupported"
        );
        assert_eq!(
            code(&|host| host
                .evaluate_curve(line_id.to_string(), vec![1.0; MAX_CURVE_SAMPLES + 1], 0)
                .map(|_| Value::Null)),
            "budget"
        );
    }
    #[test]
    fn selection_is_staged_validated_and_read_back() {
        let limits = Limits::default();
        let mut document = Document::new("selection");
        let first = Entity::new("first", CoreGeometry::Point([0.; 3]), None);
        let second = Entity::new("second", CoreGeometry::Point([1.; 3]), None);
        let (first_id, second_id) = (first.id, second.id);
        document.entities.insert(first_id, first);
        document.entities.insert(second_id, second);
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let initial_selection = [first_id];
        let mut staged = Staged::new(&coordinator, base, &initial_selection, &limits).unwrap();
        with_host(&mut staged, |host| {
            assert_eq!(host.selected_objects().unwrap(), vec![first_id.to_string()]);
            host.set_selection(vec![second_id.to_string()]).unwrap();
            assert_eq!(
                host.selected_objects().unwrap(),
                vec![second_id.to_string()]
            );
            host.add_to_selection(vec![first_id.to_string()]).unwrap();
            let mut expected = vec![first_id.to_string(), second_id.to_string()];
            expected.sort();
            let mut actual = host.selected_objects().unwrap();
            actual.sort();
            assert_eq!(actual, expected);
            host.set_selection(Vec::new()).unwrap();
            assert!(host.selected_objects().unwrap().is_empty());
            host.add_to_selection(vec![second_id.to_string()]).unwrap();
        });
        let run = staged.finish("selection".into(), Value::Null).unwrap();
        // Selection is not document state, so no transaction is produced.
        assert!(run.request.is_none());
        assert_eq!(run.selection, Some(vec![second_id]));
    }

    #[test]
    fn selection_remove_and_toggle_follow_the_set_rules() {
        let limits = Limits::default();
        let mut document = Document::new("selection toggle");
        let first = Entity::new("first", CoreGeometry::Point([0.; 3]), None);
        let second = Entity::new("second", CoreGeometry::Point([1.; 3]), None);
        let (first_id, second_id) = (first.id, second.id);
        document.entities.insert(first_id, first);
        document.entities.insert(second_id, second);
        let initial_selection = [first_id];
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &initial_selection, &limits).unwrap();
        with_host(&mut staged, |host| {
            host.toggle_selection(vec![second_id.to_string()]).unwrap();
            let mut expected = vec![first_id.to_string(), second_id.to_string()];
            expected.sort();
            let mut actual = host.selected_objects().unwrap();
            actual.sort();
            assert_eq!(actual, expected);
            // A second toggle of a selected id removes it.
            host.toggle_selection(vec![first_id.to_string()]).unwrap();
            assert_eq!(
                host.selected_objects().unwrap(),
                vec![second_id.to_string()]
            );
            host.remove_from_selection(vec![second_id.to_string()])
                .unwrap();
            assert!(host.selected_objects().unwrap().is_empty());
            // Removing an id that is not selected is a no-op.
            host.remove_from_selection(vec![uuid::Uuid::new_v4().to_string()])
                .unwrap();
            assert!(host.selected_objects().unwrap().is_empty());
        });
        let run = staged.finish("toggle".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert_eq!(run.selection, Some(Vec::new()));
    }
    #[test]
    fn selection_refusals_name_the_id_permission_and_grammar_rules() {
        let mut document = Document::new("selection refusals");
        let mut locked = Entity::new("locked", CoreGeometry::Point([0.; 3]), None);
        locked.attributes.locked = true;
        let locked_id = locked.id;
        document.entities.insert(locked_id, locked);
        let visible = Entity::new("visible", CoreGeometry::Point([1.; 3]), None);
        let visible_id = visible.id;
        document.entities.insert(visible_id, visible);
        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;

        assert_eq!(
            code(&|host| host
                .set_selection(vec!["not-a-uuid".into()])
                .map(|()| Value::Null)),
            "id"
        );
        assert_eq!(
            code(&|host| host
                .set_selection(vec![uuid::Uuid::new_v4().to_string()])
                .map(|()| Value::Null)),
            "missing"
        );
        // A locked object is not selectable.
        assert_eq!(
            code(&|host| host
                .set_selection(vec![locked_id.to_string()])
                .map(|()| Value::Null)),
            "permission"
        );
        // Toggling a locked root on needs a selectable root; removing it does not.
        assert_eq!(
            code(&|host| host
                .toggle_selection(vec![locked_id.to_string()])
                .map(|()| Value::Null)),
            "permission"
        );
        assert_eq!(
            code(&|host| host
                .remove_from_selection(vec!["not-a-uuid".into()])
                .map(|()| Value::Null)),
            "id"
        );
        // Leaving the set never needs selectability: removing an unselected locked id
        // is an accepted no-op.
        let limits = Limits::default();
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.remove_from_selection(vec![locked_id.to_string()])
                .unwrap();
            assert!(host.selected_objects().unwrap().is_empty());
        });
        // Deleting first and then selecting the same object is refused.
        let deleted = refusal(&document, &[], |host| {
            host.delete_object(visible_id.to_string())?;
            host.set_selection(vec![visible_id.to_string()])
                .map(|()| Value::Null)
        });
        assert_eq!(deleted.code, "selection");
    }
    #[test]
    fn progress_and_warnings_are_staged_with_bounds_and_no_mutation() {
        let limits = Limits::default();
        let document = Document::new("messages");
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            host.progress("step one").unwrap();
            host.warning("careful").unwrap();
        });
        let run = staged.finish("feedback".into(), Value::Null).unwrap();
        assert!(run.request.is_none());
        assert_eq!(
            run.messages,
            vec![
                PluginMessage {
                    level: MessageLevel::Progress,
                    text: "step one".into()
                },
                PluginMessage {
                    level: MessageLevel::Warning,
                    text: "careful".into()
                },
            ]
        );
    }

    #[test]
    fn report_refusals_name_the_message_budget_and_grammar() {
        let document = Document::new("message refusals");
        let code = |f: &dyn Fn(&mut Host<'_>) -> Result<Value>| refusal(&document, &[], f).code;
        assert_eq!(
            code(&|host| host.progress("").map(|()| Value::Null)),
            "budget"
        );
        assert_eq!(
            code(&|host| host.progress("two\nlines").map(|()| Value::Null)),
            "message"
        );
        assert_eq!(
            code(&|host| host
                .progress("x".repeat(MAX_MESSAGE_BYTES + 1))
                .map(|()| Value::Null)),
            "budget"
        );
        assert_eq!(
            code(&|host| {
                for index in 0..MAX_MESSAGES {
                    host.progress(format!("report {index}"))?;
                }
                host.progress("overflow").map(|()| Value::Null)
            }),
            "budget"
        );
    }

    /// Build a three-node typed history: rail -> profile -> sweep.
    fn history_document() -> Document {
        use om9_core::{
            associative::FeaturePort,
            geometry::{Curve as CoreCurve, Geometry as CoreHistoryGeometry},
            history::{HistoryGraph, Recipe, ReplayEnvironment, Tolerance},
            request::{Selection, SourceBinding},
        };
        let mut document = Document::new("history");
        let mut add = |label: &str, geometry: CoreHistoryGeometry| {
            let entity = Entity::new(label, geometry, None);
            let id = entity.id;
            document.entities.insert(id, entity);
            id
        };
        let rail_curve = CoreHistoryGeometry::Curve(CoreCurve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        });
        let profile_curve = CoreHistoryGeometry::Curve(CoreCurve::Line {
            start: [0., 0., 0.],
            end: [0., 1., 0.],
            domain: [0., 1.],
        });
        let sweep_curve = CoreHistoryGeometry::Curve(CoreCurve::Line {
            start: [0., 0., 0.],
            end: [0., 0., 1.],
            domain: [0., 1.],
        });
        let out_rail = add("Rail", rail_curve);
        let out_profile = add("Profile", profile_curve);
        let out_sweep = add("Sweep", sweep_curve);
        let environment = ReplayEnvironment {
            command_version: 1,
            solver: "om9".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: om9_core::geometry::Transform::identity(),
            tolerance: Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap(),
        };
        let binding = |id, role: &str| SourceBinding {
            role: role.into(),
            source: document.stamp(id).unwrap(),
            selection: Selection::Whole,
        };
        let port =
            |role: &str| FeaturePort::geometry(role, &[om9_core::geometry::GeometryKind::Curve]);
        let mut rail = Recipe::new(
            document.id,
            "RingRail",
            vec![],
            vec![out_rail],
            environment.clone(),
        )
        .with_ports(vec![], vec![port("result")]);
        rail.parameters.insert(
            "diameter_mm".into(),
            om9_core::request::Value::length(om9_core::units::LengthMm::try_new(18.0).unwrap()),
        );
        let profile = Recipe::new(
            document.id,
            "RailProfile",
            vec![binding(out_rail, "rail")],
            vec![out_profile],
            environment.clone(),
        )
        .with_ports(vec![port("rail")], vec![port("result")]);
        let mut sweep = Recipe::new(
            document.id,
            "RailSweep",
            vec![binding(out_rail, "rail"), binding(out_profile, "profile")],
            vec![out_sweep],
            environment,
        )
        .with_ports(vec![port("rail"), port("profile")], vec![port("result")]);
        sweep.state = om9_core::history::EvaluationState::Dirty("source changed".into());
        let mut graph = HistoryGraph::default();
        for recipe in [rail, profile, sweep] {
            graph.recipes.insert(recipe.id, recipe);
        }
        document.history = graph;
        document
    }

    #[test]
    fn history_queries_report_typed_ports_states_provenance_and_plan() {
        let limits = Limits::default();
        let document = history_document();
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let list = host.history_list().unwrap();
            assert_eq!(list.len(), 3);
            assert_eq!(list[0].command, "RingRail");
            assert_eq!(list[0].state, "clean");
            assert_eq!(
                list[0].parameters["diameter_mm"],
                ParameterValue::Length { millimeters: 18.0 }
            );
            assert_eq!(list[1].inputs[0].role, "rail");
            assert_eq!(list[1].inputs[0].kind, "geometry:Curve");
            assert_eq!(list[2].state, "dirty");
            assert_eq!(list[2].state_reason.as_deref(), Some("source changed"));

            let dependencies = host.history_dependencies().unwrap();
            assert_eq!(dependencies.len(), 3);
            assert_eq!(
                dependencies[0].producer.as_deref(),
                Some(list[0].id.as_str())
            );
            assert_eq!(dependencies[0].role, "rail");

            let provenance = host.history_provenance(list[0].id.clone()).unwrap();
            assert_eq!(provenance.command, "RingRail");
            assert_eq!(provenance.command_version, 1);
            assert_eq!(provenance.unit, "mm");

            let plan = host.history_rebuild_plan().unwrap();
            assert_eq!(plan, vec![list[2].id.clone()]);

            let missing = host
                .history_feature(uuid::Uuid::new_v4().to_string())
                .unwrap_err();
            assert_eq!(missing.code, "missing");
        });
    }

    #[test]
    fn a_malformed_history_id_is_refused() {
        // The first rejected host call is sticky for the rest of the command, so this refusal
        // gets its own host.
        let limits = Limits::default();
        let document = history_document();
        let (coordinator, doc_id) = coordinator_with(document, limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let bad = host.history_feature("not-a-uuid").unwrap_err();
            assert_eq!(bad.code, "id");
        });
    }

    #[test]
    fn history_is_read_only_for_a_plugin() {
        // Every history request is a query: a plugin cannot forge a feature or change a
        // state through this surface, so the committed document is unchanged after a run.
        let limits = Limits::default();
        let document = history_document();
        let before = serde_json::to_vec(&document).unwrap();
        let (coordinator, doc_id) = coordinator_with(document.clone(), limits);
        let base = &coordinator.world().documents[&doc_id];
        let mut staged = Staged::new(&coordinator, base, &[], &limits).unwrap();
        with_host(&mut staged, |host| {
            let _ = host.history_list().unwrap();
            let _ = host.history_dependencies().unwrap();
            let _ = host.history_rebuild_plan().unwrap();
        });
        let run = staged.finish("HistoryQuery".into(), Value::Null).unwrap();
        // Read-only calls stage no object mutation.
        assert!(
            run.request
                .as_ref()
                .is_none_or(|request| request.mutations.is_empty())
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);
    }
}
