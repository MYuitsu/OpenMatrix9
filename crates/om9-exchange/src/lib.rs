// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned operation-scoped exchange requests. No archive selects a program.
#![forbid(unsafe_code)]
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    identity::{DocumentId, ObjectId, SourceStamp},
    model::Document,
    transaction::{OperationContext, TransactionCoordinator},
};
use serde::Serialize;
use std::{
    io::{self, Write},
    sync::Arc,
};
mod export_document;
pub mod import_capture;
mod inspection;
mod staged;
pub mod stl;
pub mod three_dm;
pub mod three_dm_clipboard;
pub use import_capture::ImportCapture;
mod current_overlay;
mod dependency_cache;
mod output_identity;
mod source_preparation;
pub use source_preparation::{
    PreparedArchiveSpan, PreparedNativeUnits, PreparedSourceRecord, PreservationFlags,
    SourcePreparation, WireIdentity,
};
mod merge_inputs;
pub use merge_inputs::PreparedMergeInputs;
pub mod retained_cold;
mod retained_packet;
pub use retained_cold::{ColdRetainedProof, ExpectedRetainedOutput};
#[cfg(windows)]
mod retained_stage;
#[cfg(windows)]
pub use retained_stage::StagedRetainedRequest;
#[cfg(windows)]
mod retained_write;
#[cfg(windows)]
pub use retained_write::{
    PrivateRetainedFile, RetainedExportLosses, RetainedExportReceipt, RetainedWriteReceipt,
    ValidatedRetainedFile,
};
mod wire_output;
pub use output_identity::{NativeIdentity, OutputPlan, ScopedNativeKey};
pub use wire_output::{OutputWireIdentity, WireOutputPlan};
mod import_dto;
mod import_graph;
mod imported;
pub use current_overlay::{
    ChangedFields, CurrentOverlay, HostValue, NamespaceOverlay, RecordAction, RecordOverlay,
};
pub use imported::{
    BrepBridge, BrepExport, BrepMode, ImportMode, ImportOptions, ImportReceipt,
    PreparedNativeImport, UnitResolution,
};
pub use staged::{ExportReceipt, NativeExchange, ValidatedExport};
pub struct ExportCapture {
    document: Arc<Document>,
    converted: Option<Arc<Document>>,
    // Holds the selected projection's conservative clone allowance while it
    // can be mutated and for as long as the resulting capture is alive.
    projection_reservation: Option<om9_core::transaction::ExternalMemoryReservation>,
    context: OperationContext,
    sources: Vec<SourceStamp>,
}
#[derive(Serialize)]
struct Packet<'a> {
    schema_version: u32,
    mode: &'static str,
    document: ExportDocument<'a>,
}
#[derive(Serialize)]
struct ExportDocument<'a> {
    id: DocumentId,
    name: &'a str,
    page_unit: om9_core::geometry::Unit,
    tolerance: Option<&'a om9_core::history::Tolerance>,
    current_layer: Option<om9_core::identity::LayerId>,
    layer_order: &'a [om9_core::identity::LayerId],
    layers: export_document::Layers<'a>,
    entities: export_document::Entities<'a>,
    definitions: export_document::Definitions<'a>,
    groups: export_document::Groups<'a>,
    exchange: export_document::Exchange<'a>,
}
struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
    failed: bool,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.limit)
        {
            self.failed = true;
            return Err(io::Error::other("exchange packet budget"));
        }
        self.bytes
            .try_reserve_exact(bytes.len())
            .map_err(|_| io::Error::other("exchange packet allocation"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn convert_brep_geometry(
    geometry: &mut om9_core::geometry::Geometry,
    bridge: Option<&dyn crate::BrepExport>,
    tolerance: Option<&om9_core::history::Tolerance>,
    limits: &Limits,
) -> Result<bool> {
    let om9_core::geometry::Geometry::Brep(brep) = geometry else {
        return Ok(false);
    };
    let Some(bridge) = bridge else {
        return Err(Error::Missing("BRep export bridge"));
    };
    let tolerance = tolerance.ok_or(Error::Missing("document tolerance"))?;
    tolerance.validate()?;
    *geometry = om9_core::geometry::Geometry::NativeBrep(bridge.export(brep, tolerance.absolute_mm, limits)?);
    Ok(true)
}
impl ExportCapture {
    pub fn all(coordinator: &TransactionCoordinator, id: DocumentId) -> Result<Self> {
        let (context, world) = coordinator.capture_context(None)?;
        let document = world
            .documents
            .get(&id)
            .ok_or(Error::Missing("export document"))?;
        // The capture can outlive the current world and its Undo history.
        // Register this exact immutable allocation before returning it so
        // shared admission continues to count it until the last owner drops.
        coordinator.register_external_candidate(document)?;
        let mut usage = coordinator.retained_usage()?;
        usage.add(
            Usage::array::<SourceStamp>(document.entities.len())?,
            coordinator.limits(),
        )?;
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(document.entities.len())
            .map_err(|_| Error::Budget("export source stamps"))?;
        for id in document.entities.keys() {
            sources.push(document.stamp(*id)?);
        }
        Ok(Self {
            document: document.clone(),
            converted: None,
            projection_reservation: None,
            context,
            sources,
        })
    }
    /// Capture only selected top-level objects and their nested block definitions.
    /// The original snapshot remains the authority for stale-publication checks.
    pub fn selected(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        selected: &[ObjectId],
    ) -> Result<Self> {
        use std::collections::BTreeSet;
        if selected.is_empty() {
            return Err(Error::Invalid("empty export selection"));
        }
        let mut capture = Self::all(coordinator, id)?;
        let ids: BTreeSet<_> = selected.iter().copied().collect();
        if ids.len() != selected.len() {
            return Err(Error::Invalid("duplicate export selection"));
        }
        for object in &ids {
            capture.document.stamp(*object)?;
        }
        // Account for the private copy before cloning the shared document.
        let mut held = coordinator.retained_usage()?;
        let copied = capture.document.clone_owned_bytes_estimate(coordinator.limits())?;
        // The private projection is a separately validated document. Charge
        // both live copies' bytes without summing their structural point quota.
        held.add(Usage { bytes: copied, ..Usage::default() }, coordinator.limits())?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache,
            copied.max(1), physical, Usage::array::<SourceStamp>(capture.sources.capacity())?,
        )?;
        let mut output = (*capture.document).clone();
        output.entities.retain(|id, _| ids.contains(id));
        let mut definitions = BTreeSet::new();
        let mut pending = Vec::new();
        for entity in output.entities.values() {
            if let om9_core::geometry::Geometry::Instance(id) = entity.geometry {
                pending.push(id);
            }
        }
        while let Some(id) = pending.pop() {
            if definitions.insert(id) {
                let definition = output.definitions.get(&id).ok_or(Error::Missing("block definition"))?;
                for member in &definition.members {
                    if let om9_core::geometry::Geometry::Instance(child) = member.geometry {
                        pending.push(child);
                    }
                }
            }
        }
        output.definitions.retain(|id, _| definitions.contains(id));
        output.groups.retain(|_, group| {
            group.members.retain(|id| ids.contains(id));
            !group.members.is_empty()
        });
        output.history = Default::default();
        output.results.clear();
        output.features.clear();
        output.retained_sources.clear();
        output.import_namespaces.clear();
        output.validate(coordinator.limits())?;
        capture.sources.retain(|stamp| ids.contains(&stamp.object));
        capture.converted = Some(Arc::new(output));
        capture.projection_reservation = Some(reservation);
        Ok(capture)
    }
    pub fn document(&self) -> &Document {
        self.converted.as_deref().unwrap_or(&self.document)
    }
    /// Preserve selected OM9 feature ownership and complete clean history;
    /// 3DM export continues to use the geometry-only projection from `selected`.
    pub fn selected_om9(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        selected: &[ObjectId],
    ) -> Result<Self> {
        let mut capture = Self::selected(coordinator, id, selected)?;
        let output = Arc::get_mut(
            capture
                .converted
                .as_mut()
                .ok_or(Error::Missing("clipboard projection"))?,
        )
        .ok_or(Error::Invalid("shared clipboard projection"))?;
        for feature in capture.document.features.values() {
            if feature.outputs.iter().any(|id| output.entities.contains_key(id)) {
                if feature.outputs.iter().any(|id| !output.entities.contains_key(id)) {
                    return Err(Error::Unsupported(
                        "clipboard requires all feature outputs in the selection",
                    ));
                }
                output.features.insert(feature.id, feature.clone());
            }
        }
        for recipe in capture.document.history.recipes.values() {
            if !recipe.outputs.iter().any(|id| output.entities.contains_key(id)) {
                continue;
            }
            if recipe.state != om9_core::history::EvaluationState::Clean
                || recipe.outputs.iter().any(|id| !output.entities.contains_key(id))
                || recipe.inputs.iter().any(|input| {
                    !output.entities.contains_key(&input.source.object)
                })
                || capture.document.history.references.values().any(|r| r.recipe == recipe.id)
            {
                return Err(Error::Unsupported("clipboard history dependencies or state"));
            }
            output.history.recipes.insert(recipe.id, recipe.clone());
        }
        for (id, result) in &capture.document.results {
            if !output.entities.contains_key(id) {
                continue;
            }
            if result.status != om9_core::result::ResultStatus::Current
                || result.recipe.is_none_or(|recipe| !output.history.recipes.contains_key(&recipe))
                || result.source_stamps.iter().any(|stamp| {
                    !output.entities.contains_key(&stamp.object)
                })
            {
                return Err(Error::Unsupported("clipboard result dependencies or state"));
            }
            output.results.insert(*id, result.clone());
        }
        output.validate(coordinator.limits())?;
        Ok(capture)
    }
    /// Build a private export-only document with editable BReps replaced by
    /// native payloads. The shared document snapshot is never mutated, so the
    /// later publication revalidation still sees the original Arc.
    fn reserve_brep_projection(&mut self, coordinator: &TransactionCoordinator) -> Result<()> {
        if self.converted.is_some() || !self.document.entities.values()
            .chain(self.document.definitions.values().flat_map(|definition| &definition.members))
            .any(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Brep(_)))
        {
            return Ok(());
        }
        let bytes = self.document.clone_owned_bytes_estimate(coordinator.limits())?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes.max(1), physical,
            Usage::array::<SourceStamp>(self.sources.capacity())?,
        )?;
        self.projection_reservation = Some(reservation);
        Ok(())
    }

    fn convert_breps(
        &mut self,
        bridge: Option<&dyn crate::BrepExport>,
        limits: &om9_core::budget::Limits,
    ) -> Result<()> {
        if self.converted.is_none()
            && !self.document.entities.values()
                .chain(self.document.definitions.values().flat_map(|definition| &definition.members))
                .any(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Brep(_)))
        {
            // The original immutable snapshot is already suitable for export.
            return Ok(());
        }
        let tolerance = self.document.tolerance().cloned();
        let selected = self.converted.take();
        let had_projection = selected.is_some();
        let mut document = match selected {
            Some(document) => Arc::try_unwrap(document)
                .map_err(|_| Error::Invalid("shared export projection"))?,
            None => (*self.document).clone(),
        };
        let mut converted = 0usize;
        for entity in document.entities.values_mut() {
            if convert_brep_geometry(&mut entity.geometry, bridge, tolerance.as_ref(), limits)? {
                converted = converted
                    .checked_add(1)
                    .ok_or(Error::Overflow("export editable BRep count"))?;
            }
        }
        for definition in document.definitions.values_mut() {
            for member in &mut definition.members {
                if convert_brep_geometry(&mut member.geometry, bridge, tolerance.as_ref(), limits)? {
                    converted = converted
                        .checked_add(1)
                        .ok_or(Error::Overflow("export editable BRep count"))?;
                }
            }
        }
        if converted > 0 {
            // This private projection replaces B-rep representations. Keep
            // retained provenance coherent without touching the source model.
            document.reconcile_derived_export_bindings(limits)?;
        }
        if converted > 0 || had_projection {
            self.converted = Some(Arc::new(document));
        }
        Ok(())
    }
    pub fn revalidate(&self, coordinator: &TransactionCoordinator) -> Result<()> {
        if coordinator.authority() != self.context.authority() {
            return Err(Error::WrongOwner);
        }
        let (context, world) = coordinator.capture_context(None)?;
        if context != self.context {
            return Err(Error::Stale("export publication context"));
        }
        let current = world
            .documents
            .get(&self.document.id)
            .ok_or(Error::Missing("export owner"))?;
        if !Arc::ptr_eq(current, &self.document) {
            return Err(Error::Stale("export document snapshot"));
        }
        for stamp in &self.sources {
            current.resolve(stamp)?;
        }
        Ok(())
    }
    pub fn encode(&self, limits: &Limits) -> Result<Vec<u8>> {
        let mut usage = self.document().validate(limits)?;
        usage.add(Usage::array::<SourceStamp>(self.sources.len())?, limits)?;
        let remaining = limits
            .max_bytes
            .checked_sub(usage.bytes)
            .ok_or(Error::Budget("export serialization peak"))?;
        // Reserve transient old/new Vec storage together with the immutable
        // source. Native DOM/geometry memory is separately capped by its owner.
        let limit = usize::try_from((remaining / 2).min(16 * 1024 * 1024))
            .map_err(|_| Error::Overflow("exchange packet address space"))?;
        self.encode_up_to(limit)
    }
    fn packet(&self) -> Result<Packet<'_>> {
        let d = self.document();
        if let Some(tolerance) = d.tolerance() {
            tolerance.validate()?;
            if tolerance.angular_rad <= 0. || tolerance.angular_rad > std::f64::consts::PI
                || tolerance.relative <= 0. || tolerance.relative >= 1. {
                return Err(Error::Unsupported("3DM model tolerance cannot represent document settings"));
            }
        }
        let document = ExportDocument {
            id: d.id,
            name: &d.name,
            page_unit: d.page_unit,
            tolerance: d.tolerance(),
            current_layer: d.current_layer,
            layer_order: &d.layer_order,
            layers: export_document::Layers(&d.layers),
            entities: export_document::Entities(&d.entities),
            definitions: export_document::Definitions(&d.definitions),
            groups: export_document::Groups(&d.groups),
            exchange: export_document::Exchange(&d.exchange),
        };
        Ok(Packet { schema_version: 1, mode: "geometry_only_current", document })
    }
    fn encode_up_to(&self, limit: usize) -> Result<Vec<u8>> {
        let mut writer = BoundedWriter {
            bytes: vec![],
            limit,
            failed: false,
        };
        if let Err(error) = serde_json::to_writer(&mut writer, &self.packet()?) {
            return Err(if writer.failed {
                Error::Budget("exchange packet bytes")
            } else {
                Error::Io(error.to_string())
            });
        }
        Ok(writer.bytes)
    }

}

#[cfg(test)]
mod brep_export_tolerance_tests {
    use super::*;
    use std::cell::Cell;
    use om9_core::{brep::{Brep, BrepDisplay}, geometry::{Geometry, Mesh, NativeBrep}, history::Tolerance, model::{Document, Entity}, transaction::World};

    struct RecordingBridge(Cell<Option<f64>>);
    impl BrepExport for RecordingBridge {
        fn export(&self, _: &Brep, tolerance_mm: f64, _: &Limits) -> Result<NativeBrep> {
            self.0.set(Some(tolerance_mm));
            Ok(NativeBrep { display: Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }, payload: vec![1].into(), crc: 0 })
        }
    }

    fn fixture() -> Brep {
        Brep::from_native("CASCADE Topology fixture".into(), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            triangles: vec![[0,1,2,0]], segments: vec![], vertices: vec![],
            faces: vec![uuid::Uuid::new_v4()], edges: vec![], vertex_ids: vec![], face_edges: vec![],
        }, &Limits::default()).unwrap()
    }

    #[test]
    fn geometry_without_breps_uses_the_original_immutable_capture() {
        let limits = Limits::default();
        let mut document = Document::new("point export");
        let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(point.id, point);
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let mut capture = ExportCapture::all(&coordinator, id).unwrap();
        capture.convert_breps(None, &limits).unwrap();
        assert!(capture.converted.is_none());
        assert!(Arc::ptr_eq(&capture.document, &coordinator.world().documents[&id]));
    }

    #[test]
    fn brep_projection_clone_is_reserved_before_conversion_and_until_drop() {
        use om9_core::{budget::memory::MemoryClass, geometry::Cloud};
        struct CheckingBridge<'a>(&'a TransactionCoordinator);
        impl BrepExport for CheckingBridge<'_> {
            fn export(&self, _: &Brep, _: f64, _: &Limits) -> Result<NativeBrep> {
                assert!(self.0.external_reserved_bytes(MemoryClass::CpuCache) >= 1_000_000);
                Ok(NativeBrep { display: Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }, payload: vec![1].into(), crc: 0 })
            }
        }
        let limits = Limits { max_bytes: 8 * 1024 * 1024, ..Limits::default() };
        let mut document = Document::with_template_tolerance("conversion pressure");
        let brep = Entity::new("brep", Geometry::Brep(fixture()), None);
        document.entities.insert(brep.id, brep);
        let cloud = Entity::new("cloud", Geometry::Cloud(Cloud { points: vec![[1., 2., 3.]; 50_000], normals: vec![], colors: vec![], hidden: vec![] }), None);
        document.entities.insert(cloud.id, cloud);
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let mut capture = ExportCapture::all(&coordinator, id).unwrap();
        let competing = coordinator.reserve_external_memory(MemoryClass::Export, 6 * 1024 * 1024, None).unwrap();
        assert!(capture.reserve_brep_projection(&coordinator).is_err());
        assert!(capture.converted.is_none());
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        drop(competing);
        capture.reserve_brep_projection(&coordinator).unwrap();
        capture.convert_breps(Some(&CheckingBridge(&coordinator)), &limits).unwrap();
        assert!(capture.converted.is_some());
        assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) >= 1_000_000);
        drop(capture);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn conversion_uses_captured_document_tolerance_and_rejects_missing() {
        let limits = Limits::default();
        for value in [0.02, 0.00001] {
            let mut document = Document::new("non-default tolerance");
            document.tolerance = Some(Tolerance::try_new(value, 0.01, 0.1).unwrap());
            let entity = Entity::new("brep", Geometry::Brep(fixture()), None);
            let id = entity.id;
            document.entities.insert(id, entity);
            let document_id = document.id;
            let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
            let mut capture = ExportCapture::all(&coordinator, document_id).unwrap();
            let bridge = RecordingBridge(Cell::new(None));
            capture.convert_breps(Some(&bridge), &limits).unwrap();
            assert_eq!(bridge.0.get(), Some(value));
            assert!(matches!(capture.document().entities[&id].geometry, Geometry::NativeBrep(_)));
            assert!(matches!(coordinator.world().documents[&document_id].entities[&id].geometry, Geometry::Brep(_)));
        }
        let mut geometry = Geometry::Brep(fixture());
        let bridge = RecordingBridge(Cell::new(None));
        assert!(matches!(convert_brep_geometry(&mut geometry, Some(&bridge), None, &limits), Err(Error::Missing("document tolerance"))));
        assert!(bridge.0.get().is_none());
        assert!(matches!(geometry, Geometry::Brep(_)));
    }
}
