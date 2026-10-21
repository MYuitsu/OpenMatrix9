// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Geometry, Transform, Unit},
    identity::*,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
    pub layer: Option<LayerId>,
    pub visible: bool,
    #[serde(default, skip_serializing_if = "not_hidden")]
    pub hidden: bool,
    pub selectable: bool,
    pub locked: bool,
    pub color: Option<[u8; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_pattern: Option<LinePatternId>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub line_pattern_source: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<MaterialId>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub material_source: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_color: Option<[u8; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_weight_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_color_source: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_weight_source: Option<u32>,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub display_order: i32,
    #[serde(default = "model_space", skip_serializing_if = "is_model_space")]
    pub space: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport_id: Option<ViewportId>,
}
fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}
fn model_space() -> u32 {
    1
}
fn is_model_space(value: &u32) -> bool {
    *value == 1
}
fn not_hidden(value: &bool) -> bool {
    !*value
}
fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub id: ObjectId,
    pub label: String,
    pub generation: u64,
    pub geometry_revision: u64,
    pub attributes_revision: u64,
    pub geometry: Geometry,
    pub placement: Transform,
    pub attributes: Attributes,
    /// Versioned jewellery semantic slots (OM9-L3-01-03-009). Optional and
    /// additive: an entity without extensions stays byte-identical on save.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_named_slot_map"
    )]
    pub extensions: BTreeMap<String, Extension>,
}
impl Entity {
    pub fn new(
        label: impl Into<String>,
        geometry: Geometry,
        layer: impl Into<Option<LayerId>>,
    ) -> Self {
        let layer = layer.into();
        let color = if layer.is_none() && matches!(geometry, Geometry::Curve(_)) {
            Some([0, 130, 85, 255])
        } else {
            None
        };
        Self {
            id: ObjectId::new(),
            label: label.into(),
            generation: 1,
            geometry_revision: 1,
            attributes_revision: 1,
            geometry,
            placement: Transform::identity(),
            attributes: Attributes {
                layer,
                visible: true,
                hidden: false,
                selectable: true,
                locked: false,
                color,
                line_pattern: None,
                line_pattern_source: 0,
                material: None,
                material_source: 0,
                plot_color: None,
                plot_weight_mm: None,
                plot_color_source: None,
                plot_weight_source: None,
                display_order: 0,
                space: 1,
                viewport_id: None,
            },
            extensions: BTreeMap::new(),
        }
    }
    pub fn extension(&self, slot: &str) -> Option<&Extension> {
        self.extensions.get(slot)
    }
    /// The geometry revision plus its owning object is the display handle a
    /// renderer or picker caches; it is typed so a retired object can never be
    /// read as a live one.
    pub fn geometry_handle(&self) -> GeometryHandle {
        GeometryHandle {
            object: self.id,
            generation: self.generation,
            geometry_revision: self.geometry_revision,
        }
    }
}

/// Extension slots are namespaced names; the name is the persistence key and
/// the identity of the slot. Known slots keep their documented meaning, unknown
/// slots are preserved verbatim instead of being dropped on load.
pub const GEM_EXTENSION_SLOT: &str = "om9.gem";
pub const RAIL_EXTENSION_SLOT: &str = "om9.rail";
pub const HEAD_EXTENSION_SLOT: &str = "om9.head";
pub const CUTTER_EXTENSION_SLOT: &str = "om9.cutter";
/// Trimming loops over the entity's surface (OM9-L2-02-01 L3-005). The surface itself
/// stays the entity geometry; the slot constrains the visible face.
pub const TRIM_SURFACE_EXTENSION_SLOT: &str = "om9.trim.surface";
/// Lightweight extrusion parameters (OM9-L2-02-01 L3-008); the profile stays the geometry.
pub const EXTRUSION_EXTENSION_SLOT: &str = "om9.extrusion";
pub fn extension_slot_known(slot: &str) -> bool {
    matches!(
        slot,
        GEM_EXTENSION_SLOT
            | RAIL_EXTENSION_SLOT
            | HEAD_EXTENSION_SLOT
            | CUTTER_EXTENSION_SLOT
            | TRIM_SURFACE_EXTENSION_SLOT
            | EXTRUSION_EXTENSION_SLOT
    )
}
/// A slot name is a lowercase namespace plus a name, separated by a dot. A bare
/// word such as `"Gem"` is not a slot and must not be stored as one.
pub fn slot_name_valid(slot: &str) -> bool {
    let mut segments = slot.split('.');
    let Some(namespace) = segments.next() else {
        return false;
    };
    if namespace.is_empty()
        || !namespace
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        || !namespace
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
    {
        return false;
    }
    let mut count = 1;
    for segment in segments {
        if segment.is_empty()
            || !segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return false;
        }
        count += 1;
    }
    count >= 2
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extension {
    pub version: u32,
    /// Verbatim JSON object text. Unknown member names inside the payload are
    /// preserved because the record stores text, not a narrowed struct.
    pub payload: String,
}
impl Extension {
    pub fn new(version: u32, payload: impl Into<String>) -> Result<Self> {
        let extension = Self {
            version,
            payload: payload.into(),
        };
        extension.check_shape()?;
        Ok(extension)
    }
    fn check_shape(&self) -> Result<()> {
        if self.version == 0 {
            return Err(Error::Invalid("extension version"));
        }
        let object: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&self.payload).map_err(|_| Error::Invalid("extension payload"))?;
        if object.is_empty() {
            return Err(Error::Invalid("empty extension payload"));
        }
        Ok(())
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        self.check_shape()?;
        if self.payload.len() > l.max_name_bytes {
            return Err(Error::Budget("extension payload"));
        }
        Ok(Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.payload.capacity() as u64,
            nodes: 1,
            ..Usage::default()
        })
    }
}

/// Cached display identity for one object: name, colour and the visibility and
/// lock flags after the owning layer chain is applied.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayAttributes {
    pub object: ObjectId,
    pub name: String,
    pub color: Option<[u8; 4]>,
    pub visible: bool,
    pub selectable: bool,
    pub editable: bool,
    pub layer: Option<LayerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryHandle {
    pub object: ObjectId,
    pub generation: u64,
    pub geometry_revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FeatureKind {
    Annotation,
    Dimension,
    Construction,
    Semantic,
}
impl FeatureKind {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Annotation => "annotation",
            Self::Dimension => "dimension",
            Self::Construction => "construction",
            Self::Semantic => "semantic",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag {
            "annotation" => Ok(Self::Annotation),
            "dimension" => Ok(Self::Dimension),
            "construction" => Ok(Self::Construction),
            "semantic" => Ok(Self::Semantic),
            _ => Err(Error::Unsupported("feature kind")),
        }
    }
}

/// A non-geometric document feature. It owns a stable `FeatureId` for its whole
/// lifetime and never reuses the ID of a deleted feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    pub id: FeatureId,
    pub label: String,
    pub kind: FeatureKind,
    pub outputs: Vec<ObjectId>,
    pub revision: u64,
}
impl Feature {
    pub fn new(label: impl Into<String>, kind: FeatureKind, outputs: Vec<ObjectId>) -> Self {
        Self {
            id: FeatureId::new(),
            label: label.into(),
            kind,
            outputs,
            revision: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<ObjectId>,
    pub revision: u64,
}
impl Group {
    pub fn new(name: impl Into<String>, members: Vec<ObjectId>) -> Self {
        Self {
            id: GroupId::new(),
            name: name.into(),
            members,
            revision: 1,
        }
    }
    pub fn contains(&self, id: ObjectId) -> bool {
        self.members.contains(&id)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: DefinitionId,
    pub label: String,
    pub revision: u64,
    pub members: Vec<Entity>,
}
impl Definition {
    pub fn new(label: impl Into<String>, members: Vec<Entity>) -> Self {
        Self {
            id: DefinitionId::new(),
            label: label.into(),
            revision: 1,
            members,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub parent: Option<LayerId>,
    pub visible: bool,
    pub selectable: bool,
    pub locked: bool,
    pub color: [u8; 4],
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_pattern: Option<LinePatternId>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub line_pattern_source: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<MaterialId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_color: Option<[u8; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_weight_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_slot: Option<crate::layers::PresetSlot>,
}
impl Layer {
    pub fn new(name: impl Into<String>, color: [u8; 4]) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            parent: None,
            visible: true,
            selectable: true,
            locked: false,
            color,
            revision: 1,
            line_pattern: None,
            line_pattern_source: 0,
            material: None,
            plot_color: None,
            plot_weight_mm: None,
            preset_slot: None,
        }
    }
    pub fn preset(slot: crate::layers::PresetSlot) -> Self {
        let mut layer = Self::new(slot.name(), slot.color());
        layer.preset_slot = Some(slot);
        layer
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    pub visible: bool,
    pub selectable: bool,
    pub editable: bool,
    pub reference_snappable: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: DocumentId,
    pub name: String,
    pub epoch: u64,
    pub session_generation: u64,
    #[serde(default = "SessionId::legacy_nil")]
    pub owner_session: SessionId,
    pub revision: u64,
    pub context_revision: u64,
    pub page_unit: Unit,
    /// One document tolerance policy (OM9-L3-01-08-003). `None` means the document
    /// states no tolerance: no default is invented, so readers must ask explicitly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<crate::history::Tolerance>,
    pub current_layer: Option<LayerId>,
    pub layer_order: Vec<LayerId>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub layers: BTreeMap<LayerId, Layer>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub entities: BTreeMap<ObjectId, Entity>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub definitions: BTreeMap<DefinitionId, Definition>,
    pub history: crate::history::HistoryGraph,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub results: BTreeMap<ObjectId, crate::result::PublishedResult>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub resources: BTreeMap<ResourceId, crate::resource::Resource>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub retained_sources: BTreeMap<SourceId, crate::retained::RetainedSource>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub import_namespaces: BTreeMap<ImportNamespaceId, crate::retained::ImportNamespace>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub groups: BTreeMap<GroupId, Group>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub features: BTreeMap<FeatureId, Feature>,
    #[serde(
        default,
        skip_serializing_if = "crate::exchange::ExchangeMetadata::is_empty"
    )]
    pub exchange: crate::exchange::ExchangeMetadata,
}
struct DefinitionPath<'a> {
    id: DefinitionId,
    parent: Option<&'a DefinitionPath<'a>>,
}
impl Document {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: DocumentId::new(),
            name: name.into(),
            epoch: 1,
            session_generation: 1,
            owner_session: SessionId::new(),
            revision: 1,
            context_revision: 1,
            page_unit: Unit::Millimeter,
            tolerance: None,
            current_layer: None,
            layer_order: vec![],
            layers: BTreeMap::new(),
            entities: BTreeMap::new(),
            definitions: BTreeMap::new(),
            history: crate::history::HistoryGraph::default(),
            results: BTreeMap::new(),
            resources: BTreeMap::new(),
            retained_sources: BTreeMap::new(),
            import_namespaces: BTreeMap::new(),
            groups: BTreeMap::new(),
            features: BTreeMap::new(),
            exchange: crate::exchange::ExchangeMetadata::default(),
        }
    }
    /// A new document that starts with the OM9 template tolerance policy. The policy
    /// is written into the document explicitly (never a hidden fallback), so a saved
    /// and reopened document still states the tolerance it was created with.
    pub fn with_template_tolerance(name: impl Into<String>) -> Self {
        let mut document = Self::new(name);
        document.tolerance = Some(crate::history::Tolerance::om9_template());
        document
    }
    /// The document's own tolerance policy, if it states one (OM9-L3-01-08-003).
    pub fn tolerance(&self) -> Option<&crate::history::Tolerance> {
        self.tolerance.as_ref()
    }
    /// The tolerance a reader must use. A document that states none is `Missing`,
    /// never an invented default.
    pub fn effective_tolerance(&self) -> Result<crate::history::Tolerance> {
        let tolerance = self
            .tolerance
            .as_ref()
            .ok_or(Error::Missing("document tolerance"))?;
        tolerance.validate()?;
        Ok(tolerance.clone())
    }
    pub fn stamp(&self, id: ObjectId) -> Result<SourceStamp> {
        let e = self.entities.get(&id).ok_or(Error::Missing("object"))?;
        Ok(SourceStamp {
            document: self.id,
            document_epoch: self.epoch,
            session_generation: self.session_generation,
            owner_session: self.owner_session,
            context_revision: self.context_revision,
            object: e.id,
            generation: e.generation,
            geometry_revision: e.geometry_revision,
            attributes_revision: e.attributes_revision,
        })
    }
    pub fn resolve(&self, stamp: &SourceStamp) -> Result<&Entity> {
        if stamp.document != self.id || &self.stamp(stamp.object)? != stamp {
            return Err(Error::Stale("source stamp"));
        }
        self.entities
            .get(&stamp.object)
            .ok_or(Error::Missing("object"))
    }
    /// Fresh runtime ownership even when two opens start with identical stored
    /// counters. Rebind only complete stamps proved current before renewal;
    /// stale/unresolved references never become valid merely by changing owner.
    pub fn renew_session(&mut self, limits: &Limits) -> Result<()> {
        let next_session = self
            .session_generation
            .checked_add(1)
            .ok_or(Error::Overflow("session generation"))?;
        let next_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(Error::Overflow("document epoch"))?;
        let mut usage = self.validate(limits)?;
        usage.add(Usage::array::<SourceStamp>(self.entities.len())?, limits)?;
        let mut current = Vec::new();
        current
            .try_reserve_exact(self.entities.len())
            .map_err(|_| Error::Budget("session stamp capture"))?;
        for id in self.entities.keys() {
            current.push(self.stamp(*id)?);
        }
        current.sort_unstable();
        let owner = SessionId::new();
        let rebind = |stamp: &mut SourceStamp| {
            if current.binary_search(stamp).is_ok() {
                stamp.session_generation = next_session;
                stamp.document_epoch = next_epoch;
                stamp.owner_session = owner;
            }
        };
        for recipe in self.history.recipes.values_mut() {
            for binding in &mut recipe.inputs {
                rebind(&mut binding.source);
            }
        }
        for result in self.results.values_mut() {
            for stamp in &mut result.source_stamps {
                rebind(stamp);
            }
        }
        self.session_generation = next_session;
        self.epoch = next_epoch;
        self.owner_session = owner;
        Ok(())
    }
    pub(crate) fn layer_permissions(
        &self,
        id: Option<LayerId>,
        snap_locked: bool,
    ) -> Result<Permissions> {
        let mut p = Permissions {
            visible: true,
            selectable: true,
            editable: true,
            reference_snappable: true,
        };
        let mut current = id;
        let mut visited = BTreeSet::new();
        while let Some(id) = current {
            if !visited.insert(id) {
                return Err(Error::Invalid("layer cycle"));
            }
            let l = self.layers.get(&id).ok_or(Error::Missing("layer"))?;
            p.visible &= l.visible;
            p.selectable &= l.selectable && !l.locked;
            p.editable &= !l.locked;
            p.reference_snappable &= !l.locked || snap_locked;
            current = l.parent;
        }
        p.selectable &= p.visible;
        // A layer Off can still receive new output. Entity editing/picking
        // separately masks visibility; lock alone denies layer writes.
        p.reference_snappable &= p.visible;
        Ok(p)
    }
    pub fn permissions(&self, id: ObjectId, snap_locked: bool) -> Result<Permissions> {
        let e = self.entities.get(&id).ok_or(Error::Missing("object"))?;
        self.permissions_for_entity(e, snap_locked)
    }
    pub fn permissions_for_entity(&self, e: &Entity, snap_locked: bool) -> Result<Permissions> {
        let mut p = self.layer_permissions(e.attributes.layer, snap_locked)?;
        p.visible &= e.attributes.visible && !e.attributes.hidden;
        p.selectable &= p.visible && e.attributes.selectable && !e.attributes.locked;
        p.editable &= p.visible && !e.attributes.locked;
        p.reference_snappable &= p.visible && (!e.attributes.locked || snap_locked);
        Ok(p)
    }
    /// Display identity of one object: name, colour and the effective
    /// visibility/selectability/editing flags after the whole layer chain.
    pub fn display_attributes(&self, id: ObjectId) -> Result<DisplayAttributes> {
        let e = self.entities.get(&id).ok_or(Error::Missing("object"))?;
        let p = self.permissions_for_entity(e, false)?;
        Ok(DisplayAttributes {
            object: e.id,
            name: e.label.clone(),
            color: e.attributes.color.or_else(|| {
                e.attributes
                    .layer
                    .and_then(|l| self.layers.get(&l))
                    .map(|l| l.color)
            }),
            visible: p.visible,
            selectable: p.selectable,
            editable: p.editable,
            layer: e.attributes.layer,
        })
    }
    pub fn geometry_handle(&self, id: ObjectId) -> Result<GeometryHandle> {
        Ok(self
            .entities
            .get(&id)
            .ok_or(Error::Missing("object"))?
            .geometry_handle())
    }
    /// A handle that no longer matches the live object is `Stale`, never a live
    /// object: a retired or edited object must not be drawn or serialised as if
    /// the cached handle were still current.
    pub fn resolve_handle(&self, handle: &GeometryHandle) -> Result<&Entity> {
        let e = self
            .entities
            .get(&handle.object)
            .ok_or(Error::Stale("geometry handle"))?;
        if e.geometry_handle() != *handle {
            return Err(Error::Stale("geometry handle"));
        }
        Ok(e)
    }
    pub fn geometry_kind(&self, id: ObjectId) -> Result<crate::geometry::GeometryKind> {
        Ok(self
            .entities
            .get(&id)
            .ok_or(Error::Missing("object"))?
            .geometry
            .kind())
    }
    pub fn feature(&self, id: FeatureId) -> Result<&Feature> {
        self.features.get(&id).ok_or(Error::Missing("feature"))
    }
    pub fn group(&self, id: GroupId) -> Result<&Group> {
        self.groups.get(&id).ok_or(Error::Missing("group"))
    }
    /// Groups are a membership overlay; they never merge geometric identity, so
    /// this reports every group that currently lists the object.
    pub fn groups_of(&self, id: ObjectId) -> Vec<GroupId> {
        self.groups
            .values()
            .filter(|g| g.contains(id))
            .map(|g| g.id)
            .collect()
    }
    pub fn extension(&self, id: ObjectId, slot: &str) -> Result<Option<&Extension>> {
        Ok(self
            .entities
            .get(&id)
            .ok_or(Error::Missing("object"))?
            .extension(slot))
    }
    /// Slots that carry data but have no meaning in this build. They are
    /// reported rather than dropped so an unknown jewellery record survives.
    pub fn unknown_extension_slots(&self) -> BTreeSet<String> {
        let mut slots = BTreeSet::new();
        for entity in self.entities.values() {
            for slot in entity.extensions.keys() {
                if !extension_slot_known(slot) {
                    slots.insert(slot.clone());
                }
            }
        }
        slots
    }
    /// Build one occurrence of a definition. The definition keeps the body
    /// topology; the occurrence only stores a placement and a layer.
    pub fn instance(
        &self,
        definition: DefinitionId,
        placement: Transform,
        layer: Option<LayerId>,
    ) -> Result<Entity> {
        if !self.definitions.contains_key(&definition) {
            return Err(Error::Missing("instance definition"));
        }
        placement.validate()?;
        let label = self
            .definitions
            .get(&definition)
            .map(|d| d.label.clone())
            .unwrap_or_default();
        let mut entity = Entity::new(label, Geometry::Instance(definition), layer);
        entity.placement = placement;
        Ok(entity)
    }
    /// Membership is an overlay on top of stable object identity: removing an
    /// object drops it from every group, deletes a group that just lost its last
    /// member, and drops features that no longer have any output. The object's
    /// geometry and the surviving objects' identities are untouched.
    pub(crate) fn prune_membership(&mut self, removed: ObjectId) -> Result<()> {
        let mut emptied = Vec::new();
        for (id, group) in self.groups.iter_mut() {
            if group.members.contains(&removed) {
                group.members.retain(|member| *member != removed);
                group.revision = group
                    .revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("group revision"))?;
                if group.members.is_empty() {
                    emptied.push(*id);
                }
            }
        }
        for id in emptied {
            self.groups.remove(&id);
        }
        self.features
            .retain(|_, feature| !feature.outputs.contains(&removed));
        Ok(())
    }
    fn validate_entity(
        &self,
        e: &Entity,
        l: &Limits,
        total: &mut Usage,
        ids: &mut BTreeSet<ObjectId>,
    ) -> Result<()> {
        if e.id.is_nil()
            || !ids.insert(e.id)
            || e.generation == 0
            || e.geometry_revision == 0
            || e.attributes_revision == 0
        {
            return Err(Error::Invalid("entity identity/revision"));
        }
        if !e.extensions.is_empty() {
            total.add(Usage::btree::<String, Extension>(e.extensions.len())?, l)?;
        }
        for (slot, extension) in &e.extensions {
            if !slot_name_valid(slot) {
                return Err(Error::Invalid("extension slot"));
            }
            total.add(
                Usage {
                    bytes: slot.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(extension.validate(l)?, l)?;
            crate::rhino_semantics::validate_extension(slot, &extension.payload, l)?;
        }
        name(&e.label, l)?;
        e.placement.validate()?;
        if e.attributes
            .layer
            .is_some_and(|id| !self.layers.contains_key(&id))
        {
            return Err(Error::Missing("entity layer"));
        }
        total.add(
            Usage {
                bytes: e.label.capacity() as u64,
                objects: 1,
                ..Usage::default()
            },
            l,
        )?;
        total.add(e.geometry.validate(l)?, l)?;
        if let Geometry::Instance(id) = e.geometry
            && !self.definitions.contains_key(&id)
        {
            return Err(Error::Missing("instance definition"));
        }
        Ok(())
    }
    fn visit_definition(
        &self,
        id: DefinitionId,
        l: &Limits,
        depth: usize,
        active: &mut BTreeSet<DefinitionId>,
        expanded: &mut Usage,
    ) -> Result<()> {
        if depth > l.max_depth {
            return Err(Error::Budget("instance depth"));
        }
        if !active.insert(id) {
            return Err(Error::Invalid("definition cycle"));
        }
        let def = self
            .definitions
            .get(&id)
            .ok_or(Error::Missing("definition"))?;
        for e in &def.members {
            expanded.add(
                Usage {
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            if let Geometry::Instance(child) = e.geometry {
                self.visit_definition(child, l, depth + 1, active, expanded)?;
            }
        }
        active.remove(&id);
        Ok(())
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        self.validate_internal(l, true)
    }
    /// Archive preflight only: validate typed metadata and charge prospective
    /// embedded payload storage before hydration. Runtime owners use validate.
    pub fn validate_manifest(&self, l: &Limits) -> Result<Usage> {
        self.validate_internal(l, false)
    }
    /// Nonallocating inventory of immutable owned storage. Cross-document
    /// callers reserve every peer before any identity/graph validation scratch.
    pub(crate) fn storage_usage(&self, l: &Limits) -> Result<Usage> {
        self.storage_usage_internal(l, true)
    }
    fn storage_usage_internal(&self, l: &Limits, payloads: bool) -> Result<Usage> {
        let mut total = Usage::array::<Self>(1)?;
        total.add(
            Usage {
                bytes: self.name.capacity() as u64,
                ..Usage::default()
            },
            l,
        )?;
        for storage in [
            Usage::array::<LayerId>(self.layer_order.capacity())?,
            Usage::btree::<ObjectId, Entity>(self.entities.len())?,
            Usage::btree::<LayerId, Layer>(self.layers.len())?,
            Usage::btree::<DefinitionId, Definition>(self.definitions.len())?,
            Usage::btree::<ObjectId, crate::result::PublishedResult>(self.results.len())?,
            Usage::btree::<ResourceId, crate::resource::Resource>(self.resources.len())?,
        ] {
            total.add(storage, l)?;
        }
        // An empty optional table is never serialised and owns no node, so it
        // charges nothing: a document without groups/features/extensions keeps
        // exactly the budget profile it had before those fields existed.
        if !self.groups.is_empty() {
            total.add(Usage::btree::<GroupId, Group>(self.groups.len())?, l)?;
        }
        if !self.features.is_empty() {
            total.add(Usage::btree::<FeatureId, Feature>(self.features.len())?, l)?;
        }
        for group in self.groups.values() {
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Group>() as u64 + group.name.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<ObjectId>(group.members.capacity())?, l)?;
        }
        for feature in self.features.values() {
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Feature>() as u64 + feature.label.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<ObjectId>(feature.outputs.capacity())?, l)?;
        }
        if !self.exchange.is_empty() {
            total.add(self.exchange.validate(l)?, l)?;
        }
        for resource in self.resources.values() {
            total.add(
                if payloads {
                    resource.validate(l)?
                } else {
                    resource.manifest_usage(l)?
                },
                l,
            )?;
        }
        for layer in self.layers.values() {
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Layer>() as u64 + layer.name.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
        }
        for definition in self.definitions.values() {
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Definition>() as u64
                        + definition.label.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<Entity>(definition.members.capacity())?, l)?;
        }
        for entity in self
            .entities
            .values()
            .chain(self.definitions.values().flat_map(|d| &d.members))
        {
            total.add(
                Usage {
                    bytes: entity.label.capacity() as u64,
                    objects: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(entity.geometry.validate(l)?, l)?;
            if !entity.extensions.is_empty() {
                total.add(
                    Usage::btree::<String, Extension>(entity.extensions.len())?,
                    l,
                )?;
            }
            for (slot, extension) in &entity.extensions {
                total.add(
                    Usage {
                        bytes: slot.capacity() as u64,
                        ..Usage::default()
                    },
                    l,
                )?;
                total.add(extension.validate(l)?, l)?;
            }
        }
        let mut expanded = Usage::default();
        for id in self.definitions.keys() {
            self.definition_storage_nodes(*id, l, 0, None, &mut expanded)?;
        }
        total.add(expanded, l)?;
        total.add(
            Usage::btree::<HistoryId, crate::history::Recipe>(self.history.recipes.len())?,
            l,
        )?;
        for recipe in self.history.recipes.values() {
            total.add(Self::recipe_storage_usage(recipe, l)?, l)?;
        }
        total.add(self.retained_owned_usage(l)?, l)?;
        for (id, result) in &self.results {
            total.add(result.validate(*id, self, l)?, l)?;
        }
        Ok(total)
    }
    pub(crate) fn recipe_storage_usage(
        recipe: &crate::history::Recipe,
        l: &Limits,
    ) -> Result<Usage> {
        let mut total = Usage::default();
        total.add(
            Usage {
                history: 1,
                ..Usage::array::<crate::history::Recipe>(1)?
            },
            l,
        )?;
        for text in [
            &recipe.command,
            &recipe.environment.solver,
            &recipe.environment.solver_version,
        ] {
            total.add(
                Usage {
                    bytes: text.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        total.add(
            Usage::array::<crate::request::SourceBinding>(recipe.inputs.capacity())?,
            l,
        )?;
        total.add(Usage::array::<ObjectId>(recipe.outputs.capacity())?, l)?;
        for input in &recipe.inputs {
            total.add(input.owned_usage(l)?, l)?;
        }
        total.add(crate::request::options_usage(&recipe.parameters, l)?, l)?;
        if let crate::history::EvaluationState::Dirty(text)
        | crate::history::EvaluationState::Unresolved(text)
        | crate::history::EvaluationState::Failed(text) = &recipe.state
        {
            total.add(
                Usage {
                    bytes: text.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }

        Ok(total)
    }
    fn definition_storage_nodes(
        &self,
        id: DefinitionId,
        l: &Limits,
        depth: usize,
        parent: Option<&DefinitionPath<'_>>,
        expanded: &mut Usage,
    ) -> Result<()> {
        if depth > l.max_depth {
            return Err(Error::Budget("instance depth"));
        }
        let mut ancestor = parent;
        while let Some(path) = ancestor {
            if path.id == id {
                return Err(Error::Invalid("definition cycle"));
            }
            ancestor = path.parent;
        }
        let path = DefinitionPath { id, parent };
        let definition = self
            .definitions
            .get(&id)
            .ok_or(Error::Missing("definition"))?;
        for entity in &definition.members {
            expanded.add(
                Usage {
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            if let Geometry::Instance(child) = entity.geometry {
                self.definition_storage_nodes(child, l, depth + 1, Some(&path), expanded)?;
            }
        }
        Ok(())
    }
    fn validation_scratch_usage(&self, l: &Limits) -> Result<Usage> {
        let objects = self
            .definitions
            .values()
            .try_fold(self.entities.len(), |n, d| {
                n.checked_add(d.members.len())
                    .ok_or(Error::Overflow("validation entities"))
            })?;
        let outputs = self
            .history
            .recipes
            .values()
            .map(|r| r.outputs.len())
            .max()
            .unwrap_or(0);
        let mut scratch = Usage::default();
        for storage in [
            Usage::btree::<LayerId, ()>(self.layer_order.len())?,
            Usage::btree::<ObjectId, ()>(objects)?,
            Usage::btree::<LayerId, ()>(self.layers.len())?,
            Usage::btree::<DefinitionId, ()>(
                self.definitions.len().min(l.max_depth.saturating_add(1)),
            )?,
            Usage::btree::<ObjectId, ()>(outputs)?,
            self.retained_validation_scratch_usage(l)?,
            self.history.execution_scratch_usage()?,
        ] {
            scratch.add(storage, l)?;
        }
        Ok(scratch)
    }
    fn validate_internal(&self, l: &Limits, require_resource_payloads: bool) -> Result<Usage> {
        // No validation set or graph traversal is allocated until owned storage
        // and its entire conservative scratch profile fit together.
        let storage = self.storage_usage_internal(l, require_resource_payloads)?;
        let mut admitted = storage;
        admitted.add(self.validation_scratch_usage(l)?, l)?;
        if self.id.is_nil()
            || [
                self.epoch,
                self.session_generation,
                self.revision,
                self.context_revision,
            ]
            .contains(&0)
        {
            return Err(Error::Invalid("document identity/revision"));
        }
        name(&self.name, l)?;
        if let Some(tolerance) = &self.tolerance {
            tolerance.validate()?;
        }
        let mut total = Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
            ..Usage::default()
        };
        total.add(Usage::array::<LayerId>(self.layer_order.capacity())?, l)?;
        total.add(Usage::btree::<ObjectId, Entity>(self.entities.len())?, l)?;
        total.add(Usage::btree::<LayerId, Layer>(self.layers.len())?, l)?;
        total.add(
            Usage::btree::<DefinitionId, Definition>(self.definitions.len())?,
            l,
        )?;
        total.add(
            Usage::btree::<ObjectId, crate::result::PublishedResult>(self.results.len())?,
            l,
        )?;
        total.add(
            Usage::btree::<ResourceId, crate::resource::Resource>(self.resources.len())?,
            l,
        )?;
        if !self.groups.is_empty() {
            total.add(Usage::btree::<GroupId, Group>(self.groups.len())?, l)?;
        }
        if !self.features.is_empty() {
            total.add(Usage::btree::<FeatureId, Feature>(self.features.len())?, l)?;
        }
        if !self.exchange.is_empty() {
            total.add(self.exchange.validate(l)?, l)?;
        }
        for (id, group) in &self.groups {
            if id.is_nil() || *id != group.id || group.revision == 0 {
                return Err(Error::Invalid("group ID/revision"));
            }
            name(&group.name, l)?;
            if group.members.is_empty() {
                return Err(Error::Invalid("empty group"));
            }
            for (index, member) in group.members.iter().enumerate() {
                if member.is_nil() || !self.entities.contains_key(member) {
                    return Err(Error::Missing("group member"));
                }
                // One object may join several groups, but never twice in the
                // same group. The scan keeps validation allocation-free.
                if group.members[..index].contains(member) {
                    return Err(Error::Invalid("duplicate group member"));
                }
            }
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Group>() as u64 + group.name.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<ObjectId>(group.members.capacity())?, l)?;
        }
        for (id, feature) in &self.features {
            if id.is_nil() || *id != feature.id || feature.revision == 0 {
                return Err(Error::Invalid("feature ID/revision"));
            }
            name(&feature.label, l)?;
            for output in &feature.outputs {
                if output.is_nil() || !self.entities.contains_key(output) {
                    return Err(Error::Missing("feature output"));
                }
            }
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Feature>() as u64 + feature.label.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<ObjectId>(feature.outputs.capacity())?, l)?;
        }
        for (id, resource) in &self.resources {
            if *id != resource.id {
                return Err(Error::Invalid("resource map key"));
            }
            total.add(
                if require_resource_payloads {
                    resource.validate(l)?
                } else {
                    resource.manifest_usage(l)?
                },
                l,
            )?;
        }
        let mut order = BTreeSet::new();
        for id in &self.layer_order {
            if !order.insert(id) || !self.layers.contains_key(id) {
                return Err(Error::Invalid("layer order"));
            }
        }
        if order.len() != self.layers.len()
            || self.current_layer.is_some_and(|id| !order.contains(&id))
        {
            return Err(Error::Invalid("current layer/order"));
        }
        let mut preset_slots = [false; 33];
        for (id, layer) in &self.layers {
            if let Some(slot) = layer.preset_slot {
                if !crate::layers::valid_label(slot, &layer.name) {
                    return Err(Error::Invalid("OM9 preset label"));
                }
                if !slot.is_custom() && layer.color != slot.color() {
                    return Err(Error::Invalid("OM9 basic preset palette"));
                }
                // Generic imported layers may share display labels. An OM9
                // Advanced custom label must be unambiguous even after restore
                // or a batch that changes a generic layer's name.
                if slot.is_custom()
                    && layer.name != slot.name()
                    && self.layers.values().any(|other| {
                        other.id != layer.id && crate::layers::same_name(&layer.name, &other.name)
                    })
                {
                    return Err(Error::Invalid("duplicate OM9 custom layer label"));
                }
                let seen = &mut preset_slots[(slot.number() - 1) as usize];
                if *seen {
                    return Err(Error::Invalid("duplicate OM9 preset slot"));
                }
                *seen = true;
            }
            if id.is_nil() || *id != layer.id || layer.revision == 0 {
                return Err(Error::Invalid("layer ID/revision"));
            }
            name(&layer.name, l)?;
            self.layer_permissions(Some(*id), false)?;
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Layer>() as u64 + layer.name.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
        }
        let mut ids = BTreeSet::new();
        for (id, e) in &self.entities {
            if *id != e.id {
                return Err(Error::Invalid("entity key"));
            }
            self.validate_entity(e, l, &mut total, &mut ids)?;
        }
        for (id, def) in &self.definitions {
            if id.is_nil() || *id != def.id || def.revision == 0 {
                return Err(Error::Invalid("definition identity"));
            }
            name(&def.label, l)?;
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Definition>() as u64 + def.label.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<Entity>(def.members.capacity())?, l)?;
            for e in &def.members {
                self.validate_entity(e, l, &mut total, &mut ids)?;
            }
        }
        let mut expanded = Usage::default();
        // Validate orphan definitions too, since they are persistent masters.
        for id in self.definitions.keys() {
            self.visit_definition(*id, l, 0, &mut BTreeSet::new(), &mut expanded)?;
        }
        total.add(expanded, l)?;
        total.add(self.history.owned_usage(self, l)?, l)?;
        total.add(self.retained_owned_usage(l)?, l)?;
        for (id, result) in &self.results {
            total.add(result.validate(*id, self, l)?, l)?;
        }
        total.check(l)?;
        let mut verification_held = total;
        verification_held.add(Usage::btree::<ObjectId, ()>(ids.len())?, l)?;
        verification_held.add(Usage::btree::<LayerId, ()>(order.len())?, l)?;
        self.validate_retained(l, verification_held)?;
        self.history.execution_order_bounded(l, verification_held)?;
        debug_assert_eq!(total, storage);
        Ok(total)
    }
    pub fn member_transform(&self, root: ObjectId, path: &[ObjectId]) -> Result<Transform> {
        let mut entity = self
            .entities
            .get(&root)
            .ok_or(Error::Missing("instance root"))?;
        let mut matrix = entity.placement;
        let mut visited = BTreeSet::new();
        for id in path {
            if !visited.insert(*id) {
                return Err(Error::Invalid("duplicate occurrence path"));
            }
            let Geometry::Instance(def) = entity.geometry else {
                return Err(Error::Invalid("non-instance path"));
            };
            let definition = self
                .definitions
                .get(&def)
                .ok_or(Error::Missing("definition"))?;
            entity = definition
                .members
                .iter()
                .find(|e| e.id == *id)
                .ok_or(Error::Missing("instance member"))?;
            matrix = matrix.compose(entity.placement)?;
        }
        Ok(matrix)
    }
    pub(crate) fn invalidate_dependents(&mut self, changed: ObjectId, missing: bool) -> Result<()> {
        let affected = self.history.invalidate(changed, missing)?;
        for id in affected {
            if let Some(result) = self.results.get_mut(&id) {
                result.status = crate::result::ResultStatus::Stale;
            }
        }
        Ok(())
    }
    pub(crate) fn invalidate_results(&mut self, changed: ObjectId, missing: bool) -> Result<()> {
        self.invalidate_dependents(changed, missing)?;
        if let Some(result) = self.results.get_mut(&changed) {
            result.status = crate::result::ResultStatus::Stale;
        }
        if let Some(owner) = self.history.owner(changed) {
            let recipe = self
                .history
                .recipes
                .get_mut(&owner)
                .ok_or(Error::Missing("recipe owner"))?;
            recipe.revision = recipe
                .revision
                .checked_add(1)
                .ok_or(Error::Overflow("recipe revision"))?;
            recipe.state = if missing {
                crate::history::EvaluationState::Unresolved(
                    "output was deleted or transferred".into(),
                )
            } else {
                crate::history::EvaluationState::Dirty("output geometry edited".into())
            };
        }
        Ok(())
    }
}
