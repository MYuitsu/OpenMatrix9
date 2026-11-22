// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix `Tools > Metal Weights` (`OM9-L2-04-19`).
//!
//! The command is read-only: it measures the selected closed B-Reps, multiplies each volume
//! by its material density and reports grams plus pennyweights. The document, selection and
//! Undo journal are never touched, so a refused open shell or missing material leaves the
//! previous valid report intact.
use super::measure::{MeasureUnit, Measurement};
use super::*;
#[cfg(feature = "occt-brep")]
use om9_core::geometry::Transform;
use om9_core::metal_weight::{
    DEFAULT_MATERIAL_ID, MassBreakdown, MassPart, MetalMaterialRegistry,
};
#[cfg(feature = "occt-brep")]
use om9_core::metal_weight::{SolidMeasure, closed_solid_volume_mm3};
use om9_core::model::Entity;
use om9_core::identity::MaterialId;

pub(crate) fn effective_render_material(
    document: &om9_core::model::Document,
    entity: &Entity,
    parent_material: Option<MaterialId>,
    has_parent: bool,
) -> Result<Option<MaterialId>> {
    match entity.attributes.material_source {
        1 => Ok(entity.attributes.material),
        3 if has_parent => Ok(parent_material),
        0 | 3 => match entity.attributes.layer {
            Some(layer) => Ok(document.layers.get(&layer).ok_or(Error::Missing("metal object layer"))?.material),
            None => Ok(None),
        },
        _ => Err(Error::Unsupported("metal material source")),
    }
}

/// Measure every solid occurrence under one selected root. Placements scale source
/// volume by the absolute affine determinant; no display tessellation is used.
#[cfg(feature = "occt-brep")]
pub fn measure_metal_source_volume(
    document: &om9_core::model::Document,
    root: &Entity,
    limits: &om9_core::budget::Limits,
) -> Result<f64> {
    measure_metal_source_volume_cancellable_admitted(document, root, limits, &Default::default(), &mut |_, _| Ok(()))
}

/// Walk source occurrences with cancellation and validation-scratch admission.
/// The OCCT measurement itself cannot yet be interrupted or fully accounted.
#[cfg(feature = "occt-brep")]
pub fn measure_metal_source_volume_cancellable_admitted(
    document: &om9_core::model::Document,
    root: &Entity,
    limits: &om9_core::budget::Limits,
    cancel: &om9_core::transaction::Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>,
) -> Result<f64> {
    measure_metal_source_volume_with_count(document, root, limits, cancel, admit, &mut 0)
}

#[cfg(feature = "occt-brep")]
pub(super) fn measure_metal_source_volume_with_count(
    document: &om9_core::model::Document,
    root: &Entity,
    limits: &om9_core::budget::Limits,
    cancel: &om9_core::transaction::Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    count: &mut u64,
) -> Result<f64> {
    struct Ancestor<'a> {
        id: om9_core::identity::DefinitionId,
        parent: Option<&'a Ancestor<'a>>,
    }
    fn visit(
        document: &om9_core::model::Document,
        entity: &Entity,
        parent: Transform,
        limits: &om9_core::budget::Limits,
        ancestors: Option<&Ancestor<'_>>,
        depth: usize,
        count: &mut u64,
        cancel: &om9_core::transaction::Cancellation,
        admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    ) -> Result<f64> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        *count = count.checked_add(1).ok_or(Error::Overflow("metal weight occurrences"))?;
        if *count > limits.max_nodes { return Err(Error::Budget("metal weight occurrences")); }
        let placement = parent.compose(entity.placement)?;
        if let Geometry::Instance(id) = entity.geometry {
            if depth >= limits.max_depth { return Err(Error::Budget("metal weight instance depth")); }
            let mut cursor = ancestors;
            while let Some(ancestor) = cursor {
                if ancestor.id == id { return Err(Error::Invalid("metal weight instance cycle")); }
                cursor = ancestor.parent;
            }
            let definition = document.definitions.get(&id).ok_or(Error::Missing("metal weight instance definition"))?;
            if definition.members.is_empty() { return Err(Error::Invalid("metal weight empty instance")); }
            let ancestor = Ancestor { id, parent: ancestors };
            let mut total = 0.0_f64;
            for member in &definition.members {
                total += visit(document, member, placement, limits, Some(&ancestor), depth + 1, count, cancel, admit)?;
                if !total.is_finite() { return Err(Error::Overflow("metal weight volume")); }
            }
            return Ok(total);
        }
        measure_metal_leaf_admitted(entity, placement, limits, cancel, admit)
    }
    visit(document, root, Transform::identity(), limits, None, 0, count, cancel, admit)
}

#[cfg(feature = "occt-brep")]
fn measure_metal_leaf_admitted(entity: &Entity, placement: Transform, limits: &om9_core::budget::Limits,
    cancel: &om9_core::transaction::Cancellation, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<f64> {
    let Geometry::Brep(brep) = &entity.geometry else {
        return Err(Error::Invalid("metal weights need closed BRep solids"));
    };
    let measure = om9_kernel::creation::measure_canonical_cancellable_admitted(brep, limits, cancel, admit)
        .map_err(|error| match error { Error::Io(_) => Error::Invalid("metal weight geometry"), other => other })?;
    if !measure.closed || measure.solids == 0 {
        return Err(Error::Invalid("metal weight requires a closed solid"));
    }
    let m = placement.0;
    let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    closed_solid_volume_mm3(SolidMeasure {
        volume: measure.volume_mm3.abs() * determinant.abs(), unit: Unit::Millimeter,
        closed: measure.closed, solids: measure.solids,
    })
}

/// Matrix Tools > Metal Weights icon (`BuilderMetalWeights`).
pub const METAL_WEIGHTS: &str = "OM9-TOOLS-006";
/// The same command typed at the Command Line.
pub const METAL_WEIGHTS_COMMAND: &str = "MetalWeights";

#[derive(Debug, Clone, PartialEq)]
pub struct MetalPartReport {
    pub object: om9_core::identity::ObjectId,
    pub label: String,
    pub material_id: String,
    pub density_g_cm3: f64,
    pub volume_mm3: f64,
    pub grams: f64,
    pub dwt: f64,
}

/// The report the Command Line, status line and tests read. It is plain data; it holds no
/// kernel handle and cannot mutate the document.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalWeightReport {
    pub material_id: String,
    pub density_g_cm3: f64,
    pub volume_mm3: f64,
    pub grams: f64,
    pub dwt: f64,
    pub uncertainty_grams: f64,
    pub parts: Vec<MetalPartReport>,
    pub text: String,
}

/// Estimates for the same selected solids under every available registry material.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalCatalogReport {
    pub estimates: Vec<(String, MassBreakdown)>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetalCommand {
    /// `material=<id>` from the command line; the default material otherwise.
    pub material: Option<String>,
    pub all: bool,
    pub calculate: bool,
}

pub fn handles_command(feature: &str) -> bool {
    [METAL_WEIGHTS, "BuilderMetalWeights", "MetalWeights", "gvMetalWeights"]
        .iter().any(|name| feature.eq_ignore_ascii_case(name))
}

/// Parse `MetalWeights`, `Metal Weights` and optional `material=<id>`.
///
/// It returns `Ok(None)` for a different command, so the shared dispatcher can fall through.
/// An unknown option is a typed refusal instead of a silently ignored token.
pub fn parse_command(text: &str) -> Result<Option<MetalCommand>> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    let Some(alias) = ["buildermetalweights", "gvmetalweights", "metalweights", "metal weights", "om9-tools-006"]
        .into_iter().find(|alias| lower == *alias || lower.strip_prefix(alias).is_some_and(|rest| rest.starts_with(char::is_whitespace)))
    else { return Ok(None) };
    let rest = &trimmed[alias.len()..];
    let mut material = None;
    let mut all = false;
    let mut calculate = false;
    for token in rest.split_whitespace() {
        if let Some((key, value)) = token.split_once('=')
            && key.eq_ignore_ascii_case("material")
        {
            if value.trim().is_empty() {
                return Err(Error::Missing("metal material id"));
            }
            if material.replace(value.trim().to_owned()).is_some() {
                return Err(Error::Invalid("duplicate metal material option"));
            }
        } else if token.eq_ignore_ascii_case("all") {
            if all { return Err(Error::Invalid("duplicate metal catalog option")); }
            all = true;
        } else if token.eq_ignore_ascii_case("calculate") {
            if calculate { return Err(Error::Invalid("duplicate metal calculate option")); }
            calculate = true;
        } else {
            return Err(Error::Invalid("metal weights option"));
        }
    }
    if all && material.is_some() { return Err(Error::Invalid("conflicting metal material options")); }
    Ok(Some(MetalCommand { material, all, calculate }))
}

impl Controller {
    /// Insert fixture definitions through the same admitted transaction and scene path.
    #[cfg(feature = "test-fixtures")]
    #[doc(hidden)]
    pub fn test_insert_metal_definitions(
        &mut self,
        definitions: Vec<om9_core::model::Definition>,
        roots: Vec<Entity>,
    ) -> Result<()> {
        let revision = self.document().revision;
        let mut mutations: Vec<_> = definitions.into_iter().map(|definition| Mutation::DefinitionInsert {
            document: self.document,
            expected_revision: revision,
            definition: Box::new(definition),
            instances: Vec::new(),
        }).collect();
        mutations.extend(roots.into_iter().map(|entity| Mutation::Insert { document: self.document, entity }));
        let held = self.scene_cache_usage()?;
        let prepared = self.coordinator.prepare_with_held_usage(TransactionRequest::new("metal fixture", mutations), held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if !receipt.published { return Err(Error::Invalid("metal fixture not published")); }
        self.sync_published(om9_core::changes::ChangeCause::Edit);
        Ok(())
    }

    /// The built-in reference materials plus any user materials this session has added.
    pub fn metal_registry(&self) -> &MetalMaterialRegistry {
        &self.metal_registry
    }

    pub fn metal_registry_mut(&mut self) -> &mut MetalMaterialRegistry {
        &mut self.metal_registry
    }

    /// Replace the user catalog only after the complete candidate validates.
    pub fn apply_user_metal_catalog(&mut self, catalog: &super::metal_catalog::UserMetalCatalog) -> Result<()> {
        catalog.validate()?;
        let mut registry = MetalMaterialRegistry::builtin();
        for material in &catalog.materials { registry.insert(material.clone())?; }
        self.metal_registry = registry;
        Ok(())
    }

    /// Restore the catalog and all document-scoped bindings as one validated state.
    pub fn restore_user_metal_catalog(&mut self, catalog: &super::metal_catalog::UserMetalCatalog) -> Result<()> {
        catalog.validate()?;
        if catalog.bindings.len() as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("metal material bindings"));
        }
        let mut registry = MetalMaterialRegistry::builtin();
        for material in &catalog.materials { registry.insert(material.clone())?; }
        let mut bindings = std::collections::BTreeMap::new();
        for binding in &catalog.bindings {
            let material = registry.get(&binding.density_id)?;
            if material.id.len() > self.coordinator.limits().max_name_bytes {
                return Err(Error::Budget("metal material binding id"));
            }
            bindings.insert((binding.document, binding.render_material), material.id.clone());
        }
        self.metal_registry = registry;
        self.metal_material_bindings = bindings;
        Ok(())
    }

    /// The last successful report. A later refusal does not clear it.
    pub fn metal_report(&self) -> Option<&MetalWeightReport> {
        self.last_metal_report.as_ref()
    }

    pub fn metal_catalog_report(&self) -> Option<&MetalCatalogReport> {
        self.last_metal_catalog_report.as_ref()
    }

    pub fn metal_panel_open(&self) -> bool { self.metal_panel_open }

    pub fn close_metal_panel(&mut self) {
        #[cfg(feature = "occt-brep")]
        self.cancel_metal_worker();
        self.metal_panel_open = false;
    }

    pub fn metal_panel_material(&self) -> Option<&str> { self.metal_panel_material.as_deref() }

    pub fn metal_panel_object_materials(&self) -> bool { self.metal_panel_object_materials }

    pub fn set_metal_panel_object_materials(&mut self, enabled: bool) {
        self.metal_panel_object_materials = enabled;
    }

    pub fn metal_panel_filters(&self) -> (bool, bool) { (self.metal_panel_reference, self.metal_panel_user) }

    pub fn set_metal_panel_filters(&mut self, reference: bool, user: bool) {
        self.metal_panel_reference = reference;
        self.metal_panel_user = user;
    }

    /// Check a binding before making a session or persisted change.
    pub fn validate_metal_material_binding(&self, render: om9_core::identity::MaterialId, density: Option<&str>) -> Result<()> {
        if !self.document().exchange.materials.contains_key(&render) { return Err(Error::Missing("render material")); }
        let key = (self.document, render);
        if let Some(id) = density {
            let material = self.metal_registry.get(id)?;
            if material.id.len() > self.coordinator.limits().max_name_bytes {
                return Err(Error::Budget("metal material binding id"));
            }
            if !self.metal_material_bindings.contains_key(&key)
                && self.metal_material_bindings.len() as u64 >= self.coordinator.limits().max_outputs {
                return Err(Error::Budget("metal material bindings"));
            }
        }
        Ok(())
    }

    /// Session association. Render colors and names never imply a density.
    pub fn bind_metal_material(&mut self, render: om9_core::identity::MaterialId, density: Option<&str>) -> Result<()> {
        self.validate_metal_material_binding(render, density)?;
        let key = (self.document, render);
        if let Some(id) = density { self.metal_material_bindings.insert(key, self.metal_registry.get(id)?.id.clone()); }
        else { self.metal_material_bindings.remove(&key); }
        Ok(())
    }

    pub fn bound_metal_material(&self, render: om9_core::identity::MaterialId) -> Option<&str> {
        self.metal_material_bindings.get(&(self.document, render)).map(String::as_str)
    }

    pub fn metal_density_in_use(&self, id: &str) -> bool {
        self.metal_material_bindings.values().any(|value| value.trim().eq_ignore_ascii_case(id.trim()))
    }

    pub fn set_metal_panel_material(&mut self, material: Option<&str>) -> Result<()> {
        if let Some(id) = material { self.metal_registry.get(id)?; }
        self.metal_panel_material = material.map(str::to_owned);
        Ok(())
    }

    pub fn calculate_metal_panel(&mut self) -> Result<()> {
        #[cfg(feature = "occt-brep")]
        if self.background_metal { return self.start_metal_worker(); }
        if self.metal_panel_object_materials { return self.calculate_object_metal_weights().map(|_| ()); }
        let material = self.metal_panel_material.clone();
        if let Some(id) = material { self.calculate_metal_weights(Some(&id)).map(|_| ()) }
        else { self.calculate_metal_weights_filtered(self.metal_panel_reference, self.metal_panel_user).map(|_| ()) }
    }

    /// Resolve object/layer/parent material at every occurrence; any missing
    /// material or density rejects the entire report before publication.
    #[cfg(feature = "occt-brep")]
    pub fn calculate_object_metal_weights(&mut self) -> Result<MetalWeightReport> {
        let ids = self.selected_ordered();
        let cancellation = om9_core::transaction::Cancellation::default();
        let mut admit = |_, _| Ok(());
        let (parts, objects) = Self::collect_object_metal_parts(self.document(), &ids, self.coordinator.limits(),
            &self.metal_material_bindings, &self.metal_registry, &cancellation, &mut admit)?;
        self.publish_metal_parts(parts, objects)
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn collect_object_metal_parts(
        document: &om9_core::model::Document,
        ids: &[om9_core::identity::ObjectId],
        limits: &om9_core::budget::Limits,
        bindings: &std::collections::BTreeMap<(om9_core::identity::DocumentId, om9_core::identity::MaterialId), String>,
        registry: &MetalMaterialRegistry,
        cancellation: &om9_core::transaction::Cancellation,
        admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    ) -> Result<(Vec<MassPart>, Vec<(om9_core::identity::ObjectId, String)>)> {
        struct Walker<'a> {
            doc: &'a om9_core::model::Document,
            limits: &'a om9_core::budget::Limits,
            bindings: &'a std::collections::BTreeMap<(om9_core::identity::DocumentId, om9_core::identity::MaterialId), String>,
            registry: &'a MetalMaterialRegistry,
            count: u64,
            retained: u64,
            definitions: Vec<om9_core::identity::DefinitionId>,
            path: Vec<om9_core::identity::ObjectId>,
            parts: Vec<MassPart>,
            objects: Vec<(om9_core::identity::ObjectId, String)>,
            cancel: &'a om9_core::transaction::Cancellation,
            admit: &'a mut dyn FnMut(u64, u64) -> Result<()>,
        }
        impl Walker<'_> {
            fn visit(&mut self, root: om9_core::identity::ObjectId, e: &Entity, parent: Transform,
                parent_material: Option<om9_core::identity::MaterialId>, has_parent: bool) -> Result<()> {
                if self.cancel.is_canceled() { return Err(Error::Canceled); }
                self.count = self.count.checked_add(1).ok_or(Error::Overflow("metal weight occurrences"))?;
                if self.count > self.limits.max_nodes { return Err(Error::Budget("metal weight occurrences")); }
                let effective = effective_render_material(self.doc, e, parent_material, has_parent)?;
                let placement = parent.compose(e.placement)?;
                self.path.try_reserve(1).map_err(|_| Error::Budget("metal occurrence path"))?;
                self.path.push(e.id);
                if let Geometry::Instance(id) = e.geometry {
                    if self.definitions.len() >= self.limits.max_depth { return Err(Error::Budget("metal weight instance depth")); }
                    if self.definitions.contains(&id) { return Err(Error::Invalid("metal weight instance cycle")); }
                    let definition = self.doc.definitions.get(&id).ok_or(Error::Missing("metal weight instance definition"))?;
                    if definition.members.is_empty() { return Err(Error::Invalid("metal weight empty instance")); }
                    self.definitions.try_reserve(1).map_err(|_| Error::Budget("metal instance path"))?;
                    self.definitions.push(id);
                    for member in &definition.members { self.visit(root, member, placement, effective, true)?; }
                    self.definitions.pop();
                } else {
                    if self.parts.len() as u64 >= self.limits.max_outputs { return Err(Error::Budget("metal weight parts")); }
                    let material = effective.ok_or(Error::Missing("metal object material"))?;
                    if !self.doc.exchange.materials.contains_key(&material) { return Err(Error::Missing("render material")); }
                    let id = self.bindings.get(&(self.doc.id, material)).ok_or(Error::Missing("metal material density binding"))?;
                    self.registry.get(id)?;
                    let retained = self.retained;
                    let mut admission = |live: u64, pending: u64| {
                        (self.admit)(retained.checked_add(live).ok_or(Error::Overflow("metal worker memory"))?, pending)
                    };
                    let volume = measure_metal_leaf_admitted(e, placement, self.limits, self.cancel, &mut admission)?;
                    let label_bound = self.doc.entities[&root].label.len().checked_add(e.label.len())
                        .and_then(|v| v.checked_add(self.path.len().checked_mul(37)?))
                        .and_then(|v| v.checked_add(128)).ok_or(Error::Overflow("metal worker memory"))? as u64;
                    let reserve = label_bound.checked_add(id.len() as u64).and_then(|v| v.checked_mul(8))
                        .and_then(|v| v.checked_add(2048)).ok_or(Error::Overflow("metal worker memory"))?;
                    (self.admit)(self.retained, reserve)?;
                    self.retained = self.retained.checked_add(reserve).ok_or(Error::Overflow("metal worker memory"))?;
                    let path = self.path.iter().map(ToString::to_string).collect::<Vec<_>>().join("/");
                    let label = format!("{} / {} [{path}]", entity_label(&self.doc.entities[&root], &root), entity_label(e, &e.id));
                    self.parts.try_reserve(1).map_err(|_| Error::Budget("metal weight parts"))?;
                    self.objects.try_reserve(1).map_err(|_| Error::Budget("metal weight parts"))?;
                    self.parts.push(MassPart::new(label.clone(), volume, id.clone()));
                    self.objects.push((root, label));
                }
                self.path.pop();
                Ok(())
            }
        }
        if ids.is_empty() { return Err(Error::Missing("metal weight selection")); }
        let mut walker = Walker { doc: document, limits, bindings, registry,
            count: 0, retained: 0, definitions: Vec::new(), path: Vec::new(), parts: Vec::new(), objects: Vec::new(),
            cancel: cancellation, admit };
        for &id in ids {
            let entity = walker.doc.entities.get(&id).ok_or(Error::Missing("metal weight object"))?;
            walker.visit(id, entity, Transform::identity(), None, false)?;
        }
        Ok((walker.parts, walker.objects))
    }

    #[cfg(not(feature = "occt-brep"))]
    pub fn calculate_object_metal_weights(&mut self) -> Result<MetalWeightReport> {
        Err(Error::Unsupported("metal weights require OCCT adapter"))
    }

    /// Compare all registry densities for the selected solids, measuring each source once.
    pub fn calculate_metal_weights_catalog(&mut self) -> Result<MetalCatalogReport> {
        self.calculate_metal_weights_filtered(true, true)
    }

    pub fn calculate_metal_weights_filtered(&mut self, reference: bool, user: bool) -> Result<MetalCatalogReport> {
        let ids = self.selected_ordered();
        if ids.is_empty() { return Err(Error::Missing("metal weight selection")); }
        let selected = |material: &&om9_core::metal_weight::MetalMaterial| {
            if self.metal_registry.is_reference(&material.id) { reference } else { user }
        };
        let material_count = self.metal_registry.iter().filter(selected).count();
        let count = ids.len().checked_mul(material_count).ok_or(Error::Overflow("metal catalog entries"))?;
        if count as u64 > self.coordinator.limits().max_outputs { return Err(Error::Budget("metal catalog entries")); }
        if material_count == 0 { return Err(Error::Missing("metal material density")); }
        let mut volumes = Vec::new();
        volumes.try_reserve_exact(ids.len()).map_err(|_| Error::Budget("metal catalog bodies"))?;
        for id in &ids {
            let entity = self.document().entities.get(id).ok_or(Error::Missing("metal weight object"))?;
            volumes.push((format!("{} [{id}]", entity_label(entity, id)), self.measure_metal_entity(entity)?));
        }
        self.publish_metal_catalog(volumes, reference, user)
    }

    pub(super) fn publish_metal_catalog(&mut self, volumes: Vec<(String, f64)>, reference: bool, user: bool) -> Result<MetalCatalogReport> {
        let report = Self::build_metal_catalog(volumes, reference, user, &self.metal_registry, self.coordinator.limits())?;
        self.install_metal_catalog(report.clone());
        Ok(report)
    }

    pub(super) fn build_metal_catalog(volumes: Vec<(String, f64)>, reference: bool, user: bool,
        registry: &MetalMaterialRegistry, limits: &om9_core::budget::Limits) -> Result<MetalCatalogReport> {
        let selected = |material: &&om9_core::metal_weight::MetalMaterial| {
            if registry.is_reference(&material.id) { reference } else { user }
        };
        let material_count = registry.iter().filter(selected).count();
        let count = volumes.len().checked_mul(material_count).ok_or(Error::Overflow("metal catalog entries"))?;
        if count as u64 > limits.max_outputs { return Err(Error::Budget("metal catalog entries")); }
        if material_count == 0 { return Err(Error::Missing("metal material density")); }
        let mut estimates = Vec::new();
        estimates.try_reserve_exact(material_count).map_err(|_| Error::Budget("metal catalog entries"))?;
        let mut text = String::from("Metal Weights:");
        for material in registry.iter().filter(selected) {
            let parts: Vec<_> = volumes.iter().map(|(key, volume)| MassPart::new(key.clone(), *volume, material.id.clone())).collect();
            let breakdown = MassBreakdown::compute(&parts, registry)?;
            text.push_str(&format!(" | {} ({}): {} g · {} dwt", material.name, material.id,
                Measurement::format_value(breakdown.total_grams), Measurement::format_value(breakdown.total_dwt)));
            if text.len() as u64 > limits.max_bytes { return Err(Error::Budget("metal catalog report")); }
            estimates.push((material.name.clone(), breakdown));
        }
        Ok(MetalCatalogReport { estimates, text })
    }

    pub(super) fn install_metal_catalog(&mut self, report: MetalCatalogReport) {
        self.metal_request_generation = self.metal_request_generation.saturating_add(1);
        #[cfg(feature = "occt-brep")]
        { self.metal_catalog_owner = None; }
        self.report_measurement(Measurement::with_text(METAL_WEIGHTS_COMMAND, report.estimates.len() as f64,
            MeasureUnit::Count, report.text.clone()));
        self.last_metal_catalog_report = Some(report);
    }

    /// Calculate the report for the current selection using one material.
    pub fn calculate_metal_weights(&mut self, material: Option<&str>) -> Result<MetalWeightReport> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("metal weight selection"));
        }
        let material = material.unwrap_or(DEFAULT_MATERIAL_ID).to_owned();
        let assignments: Vec<_> = ids.iter().map(|id| (*id, material.as_str())).collect();
        self.calculate_metal_weights_with(&assignments)
    }

    /// Calculate a per-object report. `assignments` must name each selected object exactly
    /// once; a missing or extra id is refused so a report can never silently omit a body.
    pub fn calculate_metal_weights_with(
        &mut self,
        assignments: &[(om9_core::identity::ObjectId, &str)],
    ) -> Result<MetalWeightReport> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("metal weight selection"));
        }
        if assignments.len() != ids.len() {
            return Err(Error::Invalid("metal weight material assignment"));
        }
        let mut assigned = std::collections::BTreeMap::new();
        for (id, material) in assignments {
            if !ids.contains(id) {
                return Err(Error::Invalid("metal weight material assignment"));
            }
            if assigned.insert(*id, (*material).to_owned()).is_some() {
                return Err(Error::Invalid("duplicate metal body"));
            }
        }

        let mut parts = Vec::with_capacity(ids.len());
        let mut objects = Vec::with_capacity(ids.len());
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("metal weight object"))?;
            let volume = self.measure_metal_entity(entity)?;
            let material = assigned
                .remove(id)
                .ok_or(Error::Missing("metal material assignment"))?;
            let label = entity_label(entity, id);
            parts.push(MassPart::new(format!("{label} [{id}]"), volume, material));
            objects.push((*id, label));
        }
        self.publish_metal_parts(parts, objects)
    }

    pub(super) fn publish_metal_parts(&mut self, parts: Vec<MassPart>, objects: Vec<(om9_core::identity::ObjectId, String)>) -> Result<MetalWeightReport> {
        let report = Self::build_metal_parts_report(parts, objects, &self.metal_registry)?;
        self.install_metal_report(report.clone());
        Ok(report)
    }

    pub(super) fn build_metal_parts_report(parts: Vec<MassPart>, objects: Vec<(om9_core::identity::ObjectId, String)>,
        registry: &MetalMaterialRegistry) -> Result<MetalWeightReport> {
        if parts.len() != objects.len() { return Err(Error::Invalid("metal weight report parts")); }
        let breakdown = MassBreakdown::compute(&parts, registry)?;
        let mixed = breakdown
            .entries
            .iter()
            .map(|entry| entry.material_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            > 1;
        let first_material = registry
            .get(&breakdown.entries[0].material_id)
            .cloned()
            .map_err(|_| Error::Missing("metal material density"))?;
        let material_name = if mixed {
            "Mixed materials".to_owned()
        } else {
            first_material.name.clone()
        };
        let material_id = if mixed {
            "mixed".to_owned()
        } else {
            first_material.id.clone()
        };
        let mut report_parts = Vec::with_capacity(objects.len());
        for ((id, label), entry) in objects.iter().zip(breakdown.entries.iter()) {
            report_parts.push(MetalPartReport {
                object: *id,
                label: label.clone(),
                material_id: entry.material_id.clone(),
                density_g_cm3: entry.density_g_cm3,
                volume_mm3: entry.volume_mm3,
                grams: entry.grams,
                dwt: entry.dwt,
            });
        }
        let text = report_text(
            &material_name,
            &material_id,
            if mixed {
                0.0
            } else {
                first_material.density_g_cm3
            },
            breakdown.total_volume_mm3,
            breakdown.total_grams,
            breakdown.total_dwt,
            breakdown.total_uncertainty_grams,
            &report_parts,
        );
        let report = MetalWeightReport {
            material_id,
            density_g_cm3: if mixed {
                0.0
            } else {
                first_material.density_g_cm3
            },
            volume_mm3: breakdown.total_volume_mm3,
            grams: breakdown.total_grams,
            dwt: breakdown.total_dwt,
            uncertainty_grams: breakdown.total_uncertainty_grams,
            parts: report_parts,
            text,
        };
        Ok(report)
    }

    pub(super) fn install_metal_report(&mut self, report: MetalWeightReport) {
        self.metal_request_generation = self.metal_request_generation.saturating_add(1);
        #[cfg(feature = "occt-brep")]
        { self.metal_report_owner = None; }
        self.report_measurement(Measurement::with_text(
            METAL_WEIGHTS_COMMAND,
            report.grams,
            MeasureUnit::Gram,
            report.text.clone(),
        ));
        self.last_metal_report = Some(report);
    }

    pub fn dispatch_metal_tool(&mut self, feature: &str) -> Result<()> {
        if !handles_command(feature) {
            return Err(Error::Unsupported("metal weights command"));
        }
        #[cfg(feature = "occt-brep")]
        self.cancel_metal_worker();
        self.metal_panel_open = true;
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    fn measure_metal_entity(&self, entity: &Entity) -> Result<f64> {
        measure_metal_source_volume(self.document(), entity, self.coordinator.limits())
    }

    #[cfg(not(feature = "occt-brep"))]
    fn measure_metal_entity(&self, _entity: &Entity) -> Result<f64> {
        Err(Error::Unsupported("metal weights require OCCT adapter"))
    }
}

pub(super) fn entity_label(entity: &Entity, id: &om9_core::identity::ObjectId) -> String {
    if entity.label.trim().is_empty() {
        id.to_string()
    } else {
        entity.label.clone()
    }
}

#[allow(clippy::too_many_arguments)]
fn report_text(
    material_name: &str,
    material_id: &str,
    density: f64,
    volume: f64,
    grams: f64,
    dwt: f64,
    uncertainty: f64,
    parts: &[MetalPartReport],
) -> String {
    let mut text = format!(
        "Metal Weights: {} ({}) — {} mm³ · {} g · {} dwt",
        material_name,
        material_id,
        Measurement::format_value(volume),
        Measurement::format_value(grams),
        Measurement::format_value(dwt),
    );
    if density > 0.0 {
        text.push_str(&format!(
            " · density {} g/cm³",
            Measurement::format_value(density)
        ));
    }
    if uncertainty > 0.0 {
        text.push_str(&format!(" · ±{} g", Measurement::format_value(uncertainty)));
    }
    if parts.len() > 1 {
        for part in parts {
            text.push_str(&format!(
                " | {} [{}]: {} g ({} dwt)",
                part.label,
                part.material_id,
                Measurement::format_value(part.grams),
                Measurement::format_value(part.dwt)
            ));
        }
    }
    text
}
