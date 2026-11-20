// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned-process BRep restore preparation. This is not a project archive codec.
//!
//! The caller supplies an executable fingerprint from trusted configuration,
//! accounts its source inputs, and must retain the returned owner until the
//! candidate is transferred into a transaction. No document is published here.
use om9_core::{
    Error, Result,
    brep::{Brep, BrepTopology},
    budget::{Limits, Usage, memory::MemoryClass},
    transaction::{Cancellation, ExternalMemoryReservation, Mutation, OperationContext, PreparedTransaction, TransactionCoordinator, TransactionRequest},
};
use om9_native_owner::{NativeProgram, ProcessLimits, VerifiedFile};
use sha2::{Digest, Sha256};
use std::{io::{self, Write}, path::{Path, PathBuf}};
use uuid::Uuid;

pub(crate) fn is_brep_archive(input: &[u8]) -> bool {
    input.len() >= 12 && &input[..8] == b"OM9\0\r\n\x1a\n"
        && matches!(u32::from_le_bytes(input[8..12].try_into().expect("header checked")), 6 | 7)
}
fn is_mixed_archive(input: &[u8]) -> bool {
    is_brep_archive(input) && u32::from_le_bytes(input[8..12].try_into().expect("header checked")) == 7
}
fn has_native_brep(document: &om9_core::model::Document) -> bool {
    document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot))
        .any(|geometry| matches!(geometry, om9_core::geometry::Geometry::NativeBrep(_)))
}

pub struct BrepRestoreHost { program: NativeProgram, process: ProcessLimits }

/// Isolated display reconstruction for an opaque, retained native 3DM source.
/// The fingerprint is trusted configuration; the payload remains the source.
pub struct NativeBrepRestoreHost { program: NativeProgram, process: ProcessLimits }

/// A derived display and its accounted owner. It is not published or a proof
/// that the opaque source has editable topology.
pub struct ReservedNativeBrepDisplay {
    source: om9_core::geometry::NativeBrepSource,
    display: om9_core::geometry::NativeBrepRepresentation,
    reservation: ExternalMemoryReservation,
    context: OperationContext,
}

impl ReservedNativeBrepDisplay {
    pub fn source(&self) -> &om9_core::geometry::NativeBrepSource { &self.source }
    pub fn display(&self) -> &om9_core::geometry::NativeBrepRepresentation { &self.display }
    pub fn reserved_bytes(&self) -> u64 { self.reservation.bytes() }
    pub fn into_parts(self, coordinator: &TransactionCoordinator)
        -> Result<(om9_core::geometry::NativeBrepSource, om9_core::geometry::NativeBrepRepresentation, ExternalMemoryReservation)> {
        if coordinator.capture_context(None)?.0 != self.context { return Err(Error::Stale("native BRep display context")); }
        Ok((self.source, self.display, self.reservation))
    }

    fn register(self, coordinator: &TransactionCoordinator, held: Usage) -> Result<()> {
        let (source, mut display, mut reservation) = self.into_parts(coordinator)?;
        let store = coordinator.representation_store();
        if store.lock().unwrap_or_else(|e| e.into_inner()).get_native(&source).is_ok() { return Ok(()); }
        let display_bytes = display.display_bytes()?;
        let owner_bytes = (std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>() + 2 * std::mem::size_of::<usize>()) as u64;
        let store_bytes = Usage::btree::<om9_core::geometry::NativeBrepSourceIdentity, om9_core::geometry::NativeBrepRepresentation>(1)?.bytes
            .checked_add((std::mem::size_of::<Vec<u8>>() + 2 * std::mem::size_of::<usize>()) as u64)
            .ok_or(Error::Overflow("native BRep store"))?;
        let total = display_bytes.checked_add(owner_bytes).and_then(|n| n.checked_add(store_bytes))
            .ok_or(Error::Overflow("native BRep display owner"))?.max(1);
        if total > reservation.bytes() {
            let extra = total - reservation.bytes();
            coordinator.grow_external_memory_with_held(&mut reservation, extra,
                om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), held)?;
        }
        reservation.set_allocated_bytes(display_bytes)?;
        let owner = std::sync::Arc::new(std::sync::Mutex::new(reservation));
        owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(display_bytes + owner_bytes)?;
        display.bind_owner(&source, owner.clone(), &coordinator.external_memory_account(), coordinator.limits())?;
        let mut store = store.lock().unwrap_or_else(|e| e.into_inner());
        if store.get_native(&source).is_ok() { return Ok(()); }
        store.insert_native(&source, display, coordinator.limits())?;
        owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(total)?;
        Ok(())
    }
}

/// Verified staging file tied to the originating coordinator context. The
/// originating owner supplies its current coordinator at publication time.
pub struct PreparedBrepSave {
    file: om9_archive::canonical_brep::PreparedArchive,
    context: OperationContext,
}

/// Encoded, native-verified v6 archive and the receipt for its byte buffer.
pub struct ReservedBrepArchive {
    bytes: Vec<u8>,
    reservation: ExternalMemoryReservation,
}
impl ReservedBrepArchive {
    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub fn into_parts(self) -> (Vec<u8>, ExternalMemoryReservation) { (self.bytes, self.reservation) }
}
impl PreparedBrepSave {
    pub fn publish(self, coordinator: &TransactionCoordinator, cancel: &Cancellation) -> Result<()> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if coordinator.capture_context(None)?.0 != self.context { return Err(Error::Stale("BRep save context")); }
        self.file.publish(cancel)
    }
}

/// Detached structural document and all unique source candidates. The input
/// archive is borrowed, and the owners retain every admission through staging.
pub struct ReservedBrepDocument<'a> {
    document: om9_archive::canonical_brep::BrepDocumentCandidate<'a>,
    candidates: Vec<([u8; 16], ReservedBrepCandidate)>,
    _document_reservation: ExternalMemoryReservation,
    _vector_reservation: ExternalMemoryReservation,
}

/// Structurally decoded v7 document whose canonical packets and every native
/// payload have been checked in their respective contained workers. It is
/// detached and has no scene bindings; callers cannot publish it directly.
pub struct ReservedMixedBrepDocument<'a> {
    canonical: ReservedBrepDocument<'a>,
    native: Vec<ReservedNativeBrepDisplay>,
    _native_vectors: ExternalMemoryReservation,
}
impl ReservedMixedBrepDocument<'_> {
    pub fn document(&self) -> &om9_core::model::Document { self.canonical.document() }
    pub fn native_display_count(&self) -> usize { self.native.len() }

    pub(crate) fn into_unbound_validation_document(self) -> (ExternalMemoryReservation, om9_core::model::Document) {
        let Self { canonical, native, _native_vectors } = self;
        drop((native, _native_vectors));
        canonical.into_unbound_validation_document()
    }

    /// Register the verified displays for this exact detached document. The
    /// returned document owns source payloads while its scene is prepared.
    pub fn bind(self, coordinator: &TransactionCoordinator, held: Usage, cancel: &Cancellation) -> Result<BoundBrepDocument> {
        let result = (|| {
            let Self { canonical, native, _native_vectors } = self;
            for display in native {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                display.register(coordinator, held)?;
            }
            drop(_native_vectors);
            canonical.bind_inner(coordinator, held, cancel, true)
        })();
        if result.is_err() {
            coordinator.prune_orphaned_brep_representations();
            coordinator.prune_orphaned_native_representations();
        }
        result
    }
}
impl ReservedBrepDocument<'_> {
    pub fn document(&self) -> &om9_core::model::Document { self.document.document() }
    pub fn candidates(&self) -> &[([u8; 16], ReservedBrepCandidate)] { &self.candidates }

    /// Transfer a native-validated structural document for metadata and root
    /// checks only. It has no registered display owners and must not be
    /// published as a scene or inserted into a document.
    pub(crate) fn into_unbound_validation_document(self) -> (ExternalMemoryReservation, om9_core::model::Document) {
        let Self { document, candidates, _document_reservation, _vector_reservation } = self;
        drop((candidates, _vector_reservation));
        (_document_reservation, document.into_document())
    }

    /// Bind each restored source to all matching root, definition and result
    /// occurrences in a detached document. The returned guards must survive
    /// until a later transaction or document replacement takes ownership.
    /// This does not restore a native topology proof or publish a scene.
    pub fn bind(self, coordinator: &TransactionCoordinator, held: Usage, cancel: &Cancellation) -> Result<BoundBrepDocument> {
        self.bind_inner(coordinator, held, cancel, false)
    }

    fn bind_inner(self, coordinator: &TransactionCoordinator, held: Usage, cancel: &Cancellation, allow_native: bool) -> Result<BoundBrepDocument> {
        let result = (|| {
            use om9_core::{brep::CanonicalBrep, geometry::Geometry};
            let Self { document, candidates, _document_reservation: document_owner, _vector_reservation: original_vectors } = self;
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let count = candidates.len();
            let planned = Usage::btree::<[u8; 16], (CanonicalBrep, bool)>(count)?.bytes
                .checked_add(Usage::array::<ExternalMemoryReservation>(count)?.bytes)
                .ok_or(Error::Overflow("BRep document binding"))?.max(1);
            let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            let mut owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, planned, physical(), held)?;
            let mut guards = Vec::new();
            guards.try_reserve_exact(count).map_err(|_| Error::Budget("BRep document binding"))?;
            let extra = Usage::array::<ExternalMemoryReservation>(guards.capacity().saturating_sub(count))?.bytes;
            let actual = planned.checked_add(extra).ok_or(Error::Overflow("BRep document binding"))?;
            if actual > owner.bytes() {
                let extra = actual - owner.bytes();
                coordinator.grow_external_memory_with_held(&mut owner, extra, physical(), held)?;
            }
            owner.set_allocated_bytes(actual)?;
            let mut sources = std::collections::BTreeMap::<[u8; 16], (CanonicalBrep, bool)>::new();
            for (id, candidate) in candidates {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                let (source, guard) = candidate.register(coordinator, held)?;
                if sources.insert(id, (source, false)).is_some() { return Err(Error::Invalid("BRep document binding identity")); }
                guards.push(guard);
            }
            drop(original_vectors);
            let mut document = document.into_document();
            let mut replace = |geometry: &mut Geometry| -> Result<()> {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                if let Geometry::Brep(original) = geometry {
                    let binding = original.canonical_fingerprints().1;
                    let mut id = [0u8; 16]; id.copy_from_slice(&binding[..16]);
                    let (source, used) = sources.get_mut(&id).ok_or(Error::Invalid("BRep document binding coverage"))?;
                    if source.source() != original.source() { return Err(Error::Invalid("BRep document binding source")); }
                    *used = true;
                    *geometry = Geometry::Brep(source.clone());
                } else if let Geometry::NativeBrep(source) = geometry {
                    if !allow_native { return Err(Error::Unsupported("native BRep packet disposition")); }
                    let store = coordinator.representation_store();
                    if store.lock().unwrap_or_else(|e| e.into_inner()).get_native(source).is_err() {
                        return Err(Error::Missing("native BRep document display"));
                    }
                }
                Ok(())
            };
            for entity in document.entities.values_mut() { replace(&mut entity.geometry)?; }
            for definition in document.definitions.values_mut() {
                for entity in &mut definition.members { replace(&mut entity.geometry)?; }
            }
            for result in document.results.values_mut() { replace(&mut result.geometry_snapshot)?; }
            drop(replace);
            if sources.values().any(|(_, used)| !used) { return Err(Error::Invalid("BRep document binding coverage")); }
            drop(sources);
            let scratch_bytes = document.validation_scratch_usage(coordinator.limits())?.bytes.max(1);
            let scratch = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, scratch_bytes, physical(), held)?;
            document.validate(coordinator.limits())?;
            if cancel.is_canceled() { return Err(Error::Canceled); }
            drop(scratch);
            let retained = Usage::array::<ExternalMemoryReservation>(guards.capacity())?.bytes.max(1);
            owner.set_allocated_bytes(retained)?;
            owner.shrink_to(retained)?;
            let context = coordinator.capture_context(None)?.0;
            Ok(BoundBrepDocument { document, _guards: guards, _document_reservation: document_owner, _vector_reservation: owner, context })
        })();
        if result.is_err() {
            coordinator.prune_orphaned_brep_representations();
            coordinator.prune_orphaned_native_representations();
        }
        result
    }
}

/// Detached document with registered source/display owners and retained guards.
pub struct BoundBrepDocument {
    document: om9_core::model::Document,
    _guards: Vec<ExternalMemoryReservation>,
    _document_reservation: ExternalMemoryReservation,
    _vector_reservation: ExternalMemoryReservation,
    context: OperationContext,
}
impl BoundBrepDocument {
    pub fn document(&self) -> &om9_core::model::Document { &self.document }
    pub(crate) fn document_mut(&mut self) -> &mut om9_core::model::Document { &mut self.document }

    pub(crate) fn into_replacement_parts(self) -> (om9_core::model::Document, BoundBrepOwners, OperationContext) {
        let Self { document, _guards, _document_reservation, _vector_reservation, context } = self;
        (document, BoundBrepOwners { _guards, _document_reservation, _vector_reservation }, context)
    }

    /// Build a detached scene while the document and source reservations stay
    /// live. The caller must account other inputs in `held`; this does not
    /// publish the document, scene or an exact native topology proof.
    pub fn prepare_scene(self, coordinator: &TransactionCoordinator, held: Usage, cancel: &Cancellation) -> Result<BoundBrepScene> {
        let result = (|| {
            let Self { document, _guards: guards, _document_reservation: document_owner, _vector_reservation: vectors, context } = self;
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep document scene context")); }
            let limits = *coordinator.limits();
            let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            let arc_bytes = (std::mem::size_of::<om9_core::model::Document>() + 2 * std::mem::size_of::<usize>()) as u64;
            let mut arc_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, arc_bytes, physical(), held)?;
            let scratch_bytes = om9_scene::scene::Scene::candidate_preflight_scratch(&document, &limits)?.bytes.max(1);
            let scratch = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, scratch_bytes, physical(), held)?;
            let mut scene_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            let document = std::sync::Arc::new(document);
            arc_owner.set_allocated_bytes(arc_bytes)?;
            let scene = om9_scene::scene::Scene::capture_with_peak_and_representations(document, coordinator.epoch(), &limits,
                coordinator.representation_store(), |peak| {
                    if cancel.is_canceled() { return Err(Error::Canceled); }
                    let needed = peak.max(1);
                    if needed > scene_owner.bytes() {
                        let extra = needed - scene_owner.bytes();
                        coordinator.grow_external_memory_with_held(&mut scene_owner, extra, physical(), held)?;
                    }
                    scene_owner.recheck_pending()
                })?;
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep document scene context")); }
            let retained = scene.cache_usage()?.bytes.max(1);
            if retained > scene_owner.bytes() {
                let extra = retained - scene_owner.bytes();
                coordinator.grow_external_memory_with_held(&mut scene_owner, extra, physical(), held)?;
            }
            scene_owner.set_allocated_bytes(retained)?;
            scene_owner.shrink_to(retained)?;
            drop(scratch);
            Ok(BoundBrepScene { scene, _guards: guards, _document_reservation: document_owner,
                _vector_reservation: vectors, _scene_reservation: scene_owner, _arc_reservation: arc_owner })
        })();
        if result.is_err() {
            coordinator.prune_orphaned_brep_representations();
            coordinator.prune_orphaned_native_representations();
        }
        result
    }
}

pub(crate) struct BoundBrepOwners {
    _guards: Vec<ExternalMemoryReservation>,
    _document_reservation: ExternalMemoryReservation,
    _vector_reservation: ExternalMemoryReservation,
}

pub struct BoundBrepScene {
    scene: om9_scene::scene::Scene,
    _guards: Vec<ExternalMemoryReservation>,
    _document_reservation: ExternalMemoryReservation,
    _vector_reservation: ExternalMemoryReservation,
    _scene_reservation: ExternalMemoryReservation,
    _arc_reservation: ExternalMemoryReservation,
}
impl BoundBrepScene {
    pub fn scene(&self) -> &om9_scene::scene::Scene { &self.scene }
}

/// Owns the returned candidate's admission until coordinator registration.
pub struct ReservedBrepCandidate { candidate: Brep, reservation: ExternalMemoryReservation, context: OperationContext }
impl ReservedBrepCandidate {
    pub fn candidate(&self) -> &Brep { &self.candidate }
    pub fn reserved_bytes(&self) -> u64 { self.reservation.bytes() }

    /// Transfer this candidate into the coordinator's representation store.
    /// The returned source reservation must remain live until a prepared
    /// transaction takes it; registration alone does not publish a document.
    pub fn register(self, coordinator: &TransactionCoordinator, held: Usage)
        -> Result<(om9_core::brep::CanonicalBrep, ExternalMemoryReservation)> {
        if coordinator.capture_context(None)?.0 != self.context { return Err(Error::Stale("BRep worker context")); }
        let Self { candidate, mut reservation, .. } = self;
        let bytes = candidate.storage_usage(coordinator.limits())?.bytes;
        reservation.set_allocated_bytes(bytes)?;
        reservation.shrink_to(bytes)?;
        let source = coordinator.register_reserved_brep_candidate(candidate, &mut reservation, held)?;
        Ok((source, reservation))
    }
}

/// Prepare an atomic batch of restored roots. Each entity must carry a point
/// placeholder; its geometry is replaced by the corresponding candidate.
/// Inputs and all other live caller allocations must be included in `held`;
/// candidates retain their own reservations. Nothing is published here.
pub fn prepare_brep_insertions(coordinator: &TransactionCoordinator, document: om9_core::identity::DocumentId,
    label: &str, inputs: Vec<(ReservedBrepCandidate, om9_core::model::Entity)>, held: Usage)
    -> Result<PreparedTransaction> {
    let result = (|| {
        if inputs.is_empty() { return Err(Error::Invalid("BRep insertion batch")); }
        if label.len() > coordinator.limits().max_name_bytes { return Err(Error::Budget("BRep insertion label")); }
        let count = inputs.len();
        if count as u64 > coordinator.limits().max_nodes { return Err(Error::Budget("BRep insertion batch")); }
        let requested = Usage::array::<Mutation>(count)?.bytes
            .checked_add(Usage::array::<ExternalMemoryReservation>(count)?.bytes)
            .and_then(|bytes| bytes.checked_add(label.len() as u64))
            .ok_or(Error::Overflow("BRep insertion batch"))?;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut vectors = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, requested.max(1), physical(), held)?;
        let mut mutations = Vec::new();
        let mut guards = Vec::new();
        mutations.try_reserve_exact(count).map_err(|_| Error::Budget("BRep insertion batch"))?;
        guards.try_reserve_exact(count).map_err(|_| Error::Budget("BRep insertion batch"))?;
        let actual = Usage::array::<Mutation>(mutations.capacity())?.bytes
            .checked_add(Usage::array::<ExternalMemoryReservation>(guards.capacity())?.bytes)
            .and_then(|bytes| bytes.checked_add(label.len() as u64))
            .ok_or(Error::Overflow("BRep insertion batch"))?;
        if actual > vectors.bytes() {
            let extra = actual - vectors.bytes();
            coordinator.grow_external_memory_with_held(&mut vectors, extra, physical(), held)?;
        }
        vectors.set_allocated_bytes(actual)?;
        for (candidate, mut entity) in inputs {
            if !matches!(entity.geometry, om9_core::geometry::Geometry::Point(_)) {
                return Err(Error::Invalid("BRep insertion placeholder"));
            }
            let (source, guard) = candidate.register(coordinator, held)?;
            entity.geometry = om9_core::geometry::Geometry::Brep(source);
            mutations.push(Mutation::Insert { document, entity });
            guards.push(guard);
        }
        let prepared = coordinator.prepare_with_reserved_brep_mutations(TransactionRequest::new(label, mutations), held, &mut guards)?;
        drop((guards, vectors));
        Ok(prepared)
    })();
    if result.is_err() { coordinator.prune_orphaned_brep_representations(); }
    result
}

struct StagedInput(PathBuf);
impl Drop for StagedInput {
    fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); }
}

struct RequestWriter<'a> {
    file: std::fs::File, hash: Sha256, bytes: u64, limit: u64,
    cancel: &'a Cancellation, error: Option<Error>,
}
impl Write for RequestWriter<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.cancel.is_canceled() { self.error = Some(Error::Canceled); return Err(io::Error::other("canceled")); }
        let next = self.bytes.checked_add(input.len() as u64).filter(|n| *n <= self.limit);
        let Some(next) = next else { self.error = Some(Error::Budget("BRep worker input")); return Err(io::Error::other("input budget")); };
        self.file.write_all(input)?;
        self.hash.update(input);
        self.bytes = next;
        Ok(input.len())
    }
    fn flush(&mut self) -> io::Result<()> { self.file.flush() }
}

impl NativeBrepRestoreHost {
    /// A packaged application embeds the worker digest at build time.
    pub fn admit_packaged(process: ProcessLimits) -> Result<Self> {
        let mut executable = std::env::current_exe().map_err(|error| Error::Io(error.to_string()))?;
        #[cfg(feature = "test-fixtures")]
        if executable.parent().and_then(|path| path.file_name()).is_some_and(|name| name == "deps") {
            executable = executable.parent().and_then(|path| path.parent()).ok_or(Error::Invalid("application executable directory"))?.join("om9.exe");
        }
        let fingerprint = parse_worker_fingerprint(option_env!("OM9_NATIVE_BREP_WORKER_SHA256")
            .ok_or(Error::Unsupported("packaged native BRep worker fingerprint"))?)?;
        let directory = executable.parent().ok_or(Error::Invalid("application executable directory"))?;
        #[cfg(windows)]
        let worker = directory.join("om9-native-brep-worker.exe");
        #[cfg(not(windows))]
        let worker = directory.join("om9-native-brep-worker");
        Self::admit(&worker, fingerprint, process)
    }

    /// Admit the worker using a fingerprint supplied by trusted configuration.
    pub fn admit(path: &Path, fingerprint: [u8; 32], process: ProcessLimits) -> Result<Self> {
        process.validate().map_err(|error| Error::Io(format!("native BRep worker limits: {error:?}")))?;
        if process.native_committed_bytes > 4 * 1024 * 1024 * 1024 { return Err(Error::Budget("native BRep worker memory")); }
        let program = NativeProgram::admit(path, fingerprint)
            .map_err(|error| Error::Io(format!("native BRep worker admission: {error:?}")))?;
        Ok(Self { program, process })
    }

    /// Restore a display from the exact caller-owned payload in a contained
    /// process. The source allocation and other live inputs must be in `held`.
    pub fn restore_display(&self, coordinator: &TransactionCoordinator, source: &om9_core::geometry::NativeBrepSource,
        tolerance: f64, deflection: f64, angular: f64, held: Usage, cancel: &Cancellation) -> Result<ReservedNativeBrepDisplay> {
        use om9_core::geometry::{Mesh, NativeBrepRepresentation, Point3};
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let limits = *coordinator.limits();
        source.validate(&limits)?;
        let context = coordinator.capture_context(None)?.0;
        let input_limit = limits.max_bytes.min(self.process.native_committed_bytes / 2).min(1024 * 1024 * 1024);
        if input_limit == 0 { return Err(Error::Budget("native BRep worker input")); }
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let process_bytes = self.process.reservation_bytes().map_err(|error| Error::Io(format!("native BRep worker budget: {error:?}")))?;
        let process_guard = coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, process_bytes, physical(), held)?;
        let path = std::env::temp_dir().join(format!("om9-native-brep-{}.json", Uuid::new_v4()));
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path)
            .map_err(|error| Error::Io(error.to_string()))?;
        let staging = StagedInput(path);
        let mut writer = RequestWriter { file, hash: Sha256::new(), bytes: 0, limit: input_limit, cancel, error: None };
        if let Err(error) = serde_json::to_writer(&mut writer, &(1u32, source.payload(), tolerance, deflection, angular)) {
            return Err(writer.error.take().unwrap_or_else(|| Error::Io(error.to_string())));
        }
        writer.flush().map_err(|error| Error::Io(error.to_string()))?;
        let expected: [u8; 32] = writer.hash.clone().finalize().into();
        drop(writer);
        let verified = VerifiedFile::open(&staging.0, input_limit, cancel)
            .map_err(|error| if cancel.is_canceled() { Error::Canceled } else { Error::Io(format!("native BRep worker input: {error:?}")) })?;
        if verified.fingerprint() != expected { return Err(Error::Stale("native BRep worker input changed")); }
        let hash = expected.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let args = [staging.0.as_os_str().to_owned(), hash.into(), input_limit.to_string().into(),
            self.process.native_committed_bytes.to_string().into()];
        let output = self.program.run(&args, &self.process, cancel).map_err(|error| {
            if cancel.is_canceled() { Error::Canceled } else { Error::Io(format!("native BRep worker: {error:?}")) }
        })?;
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if !output.stats.exited || output.stats.active_processes != 0 { return Err(Error::Invalid("native BRep worker lifetime")); }
        let mut decode = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let estimated = om9_archive::admit_json_usage_with_admission(&output.stdout, &limits, |needed| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if needed > decode.bytes() {
                let extra = needed - decode.bytes();
                coordinator.grow_external_memory_with_held(&mut decode, extra, physical(), held)?;
            }
            decode.recheck_pending()
        })?;
        let candidate: Brep = serde_json::from_slice(&output.stdout).map_err(|_| Error::Invalid("native BRep worker response"))?;
        let actual = candidate.storage_usage(&limits)?.bytes;
        if actual > estimated.bytes || actual > decode.bytes() { return Err(Error::Budget("native BRep worker response")); }
        candidate.validate_admitted(&limits, &mut |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = live.checked_add(pending).ok_or(Error::Overflow("native BRep worker validation"))?;
            if needed > decode.bytes() {
                let extra = needed - decode.bytes();
                coordinator.grow_external_memory_with_held(&mut decode, extra, physical(), held)?;
            }
            decode.set_allocated_bytes(live)?;
            decode.recheck_pending()
        })?;
        let geometry = candidate.display();
        if geometry.points.is_empty() || geometry.triangles.is_empty() { return Err(Error::Unsupported("native BRep rebuild has no display")); }
        let planned = Usage::array::<Point3>(geometry.points.len())?.bytes
            .checked_add(Usage::array::<Vec<u32>>(geometry.triangles.len())?.bytes)
            .and_then(|n| n.checked_add(Usage::array::<u32>(3).ok()?.bytes.checked_mul(geometry.triangles.len() as u64)?))
            .and_then(|n| n.checked_add((std::mem::size_of::<Mesh>() + 2 * std::mem::size_of::<usize>()) as u64))
            .ok_or(Error::Overflow("native BRep display"))?;
        let mut display_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, planned.max(1), physical(), held)?;
        let mut vertices = Vec::new();
        vertices.try_reserve_exact(geometry.points.len()).map_err(|_| Error::Budget("native BRep display vertices"))?;
        vertices.extend_from_slice(&geometry.points);
        let mut faces = Vec::new();
        faces.try_reserve_exact(geometry.triangles.len()).map_err(|_| Error::Budget("native BRep display faces"))?;
        for triangle in &geometry.triangles {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let mut face = Vec::new();
            face.try_reserve_exact(3).map_err(|_| Error::Budget("native BRep display face"))?;
            face.extend_from_slice(&triangle[..3]);
            faces.push(face);
        }
        let mesh = Mesh { vertices, faces, normals: Vec::new(), colors: Vec::new(), corner_uvs: Vec::new() };
        let display = NativeBrepRepresentation::from_source_mesh(source, mesh, &limits)?;
        let actual = display.display_bytes()?;
        if actual > display_owner.bytes() {
            let extra = actual - display_owner.bytes();
            coordinator.grow_external_memory_with_held(&mut display_owner, extra, physical(), held)?;
        }
        display_owner.set_allocated_bytes(actual)?;
        display_owner.shrink_to(actual)?;
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("native BRep worker context")); }
        drop((candidate, decode, output, verified, staging, process_guard));
        Ok(ReservedNativeBrepDisplay { source: source.clone(), display, reservation: display_owner, context })
    }
}

impl BrepRestoreHost {
    /// Restore a supported BRep archive and bind its source representations.
    pub(crate) fn restore_bound_archive(&self, coordinator: &TransactionCoordinator, input: &[u8],
        input_accounted: bool, held: Usage, cancel: &Cancellation) -> Result<BoundBrepDocument> {
        if is_mixed_archive(input) {
            let native = NativeBrepRestoreHost::admit_packaged(ProcessLimits::default())?;
            self.restore_mixed_document(&native, coordinator, input, input_accounted,
                om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?.bind(coordinator, held, cancel)
        } else {
            self.restore_document(coordinator, input, input_accounted,
                om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?.bind(coordinator, held, cancel)
        }
    }

    pub(crate) fn validate_unbound_archive(&self, coordinator: &TransactionCoordinator, input: &[u8],
        input_accounted: bool, held: Usage, cancel: &Cancellation)
        -> Result<(ExternalMemoryReservation, om9_core::model::Document)> {
        if is_mixed_archive(input) {
            let native = NativeBrepRestoreHost::admit_packaged(ProcessLimits::default())?;
            Ok(self.restore_mixed_document(&native, coordinator, input, input_accounted,
                om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?.into_unbound_validation_document())
        } else {
            Ok(self.restore_document(coordinator, input, input_accounted,
                om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?.into_unbound_validation_document())
        }
    }
    /// Reuse immutable, previously verified archive packets for unchanged
    /// canonical sources. The input owner must remain charged and alive; only
    /// packets still present in the current document are carried forward.
    pub fn prepare_document_candidate_from_archive(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document, input: &[u8], held: Usage,
        cancel: &Cancellation) -> Result<PreparedBrepSave> {
        use om9_core::geometry::Geometry;
        if is_mixed_archive(input) || has_native_brep(document) {
            return self.capture_and_prepare_document_candidate_with_archive(coordinator, path, document, &[], Some(input), held, cancel);
        }
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let limits = *coordinator.limits();
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut document_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let decoded = om9_archive::canonical_brep::decode_document_candidate_with_admission(input, &limits, |peak| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = peak.saturating_sub(input.len() as u64).max(1);
            if needed > document_owner.bytes() {
                let extra = needed - document_owner.bytes();
                coordinator.grow_external_memory_with_held(&mut document_owner, extra, physical(), held)?;
            }
            document_owner.recheck_pending()
        })?;
        let actual = decoded.owned_storage_bytes(&limits)?.max(1);
        if actual > document_owner.bytes() {
            let extra = actual - document_owner.bytes();
            coordinator.grow_external_memory_with_held(&mut document_owner, extra, physical(), held)?;
        }
        document_owner.set_allocated_bytes(actual)?;
        document_owner.shrink_to(actual)?;
        let geometry = || document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
            .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot));
        let count = geometry().filter(|item| matches!(item, Geometry::Brep(_) | Geometry::NativeBrep(_))).count();
        if count == 0 || count as u64 > limits.max_nodes { return Err(Error::Invalid("BRep save sources")); }
        let planned = Usage::btree::<[u8; 32], ()>(count)?.bytes
            .checked_add(Usage::array::<om9_archive::canonical_brep::ReservedBrepPacket<'_>>(decoded.packets().len())?.bytes)
            .ok_or(Error::Overflow("BRep cached packet metadata"))?.max(1);
        let mut metadata = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, planned, physical(), held)?;
        let mut wanted = std::collections::BTreeSet::new();
        for item in geometry() {
            match item {
                Geometry::Brep(source) => { wanted.insert(source.canonical_fingerprints().1); }
                Geometry::NativeBrep(_) => return Err(Error::Unsupported("native BRep packet disposition")),
                _ => {}
            }
        }
        let mut packets = Vec::new();
        packets.try_reserve_exact(decoded.packets().len()).map_err(|_| Error::Budget("BRep cached packet metadata"))?;
        let actual = Usage::btree::<[u8; 32], ()>(count)?.bytes
            .checked_add(Usage::array::<om9_archive::canonical_brep::ReservedBrepPacket<'_>>(packets.capacity())?.bytes)
            .ok_or(Error::Overflow("BRep cached packet metadata"))?.max(1);
        if actual > metadata.bytes() {
            let extra = actual - metadata.bytes();
            coordinator.grow_external_memory_with_held(&mut metadata, extra, physical(), held)?;
        }
        metadata.set_allocated_bytes(actual)?;
        for (_, bytes) in decoded.packets() {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            let packet = om9_archive::canonical_brep::decode_packet_growing(bytes, &limits,
                &coordinator.external_memory_account(), owner,
                |owner, extra| coordinator.grow_external_memory_with_held(owner, extra, physical(), held), || cancel.is_canceled())?;
            if wanted.remove(&packet.record().record().binding_fingerprint()) { packets.push(packet); }
        }
        if !wanted.is_empty() { return Err(Error::Missing("BRep cached source packet")); }
        self.prepare_document_candidate(coordinator, path, document, &packets, held, cancel)
    }

    /// Capture packets from current, account-owned topology proofs and save a
    /// document atomically. Every unique BRep in roots, definitions and
    /// results must be covered; the encoder rejects missing or extra sources.
    /// Capture invokes OCCT in the calling worker thread: native allocations
    /// and time spent inside each OCCT call are not bounded by Rust admission.
    /// Callers must keep document and source owners accounted and off the UI
    /// thread, and must revalidate their own operation before publication.
    pub fn capture_and_save_document_candidate(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document,
        sources: &[om9_kernel::brep::RegisteredTopologySource], held: Usage,
        cancel: &Cancellation) -> Result<()> {
        self.capture_and_prepare_document_candidate(coordinator, path, document, sources, held, cancel)?.publish(coordinator, cancel)
    }

    /// Prepare a verified staging file for publication by the original owner
    /// after it revalidates its live document and operation context.
    pub fn capture_and_prepare_document_candidate(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document,
        sources: &[om9_kernel::brep::RegisteredTopologySource], held: Usage,
        cancel: &Cancellation) -> Result<PreparedBrepSave> {
        self.capture_and_prepare_document_candidate_with_archive(coordinator, path, document, sources, None, held, cancel)
    }

    /// Combine verified packets from a previously opened archive with newly
    /// attested sources, retaining only packets needed by this document.
    pub fn capture_and_prepare_document_candidate_with_archive(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document,
        sources: &[om9_kernel::brep::RegisteredTopologySource], cached: Option<&[u8]>, held: Usage,
        cancel: &Cancellation) -> Result<PreparedBrepSave> {
        self.capture_and_prepare_document_candidate_with_archives(coordinator, path, document, sources, cached, held, cancel)
    }

    pub(crate) fn capture_and_prepare_document_candidate_with_archives<'a>(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document, sources: &[om9_kernel::brep::RegisteredTopologySource],
        cached: impl IntoIterator<Item = &'a [u8]>, held: Usage, cancel: &Cancellation) -> Result<PreparedBrepSave> {
        self.with_captured_packets(coordinator, document, sources, cached, held, cancel, |packets| {
            if has_native_brep(document) {
                let native = NativeBrepRestoreHost::admit_packaged(ProcessLimits::default())?;
                self.prepare_mixed_document_candidate(&native, coordinator, path, document, packets, held, cancel)
            } else { self.prepare_document_candidate(coordinator, path, document, packets, held, cancel) }
        })
    }

    /// Build an independently owned v6 archive for a detached selection. The
    /// caller retains and revalidates its selection and project operation.
    pub fn capture_and_encode_document_with_archive(&self, coordinator: &TransactionCoordinator,
        document: &om9_core::model::Document, sources: &[om9_kernel::brep::RegisteredTopologySource],
        cached: Option<&[u8]>, held: Usage, cancel: &Cancellation) -> Result<ReservedBrepArchive> {
        self.capture_and_encode_document_with_archives(coordinator, document, sources, cached, held, cancel)
    }

    pub(crate) fn capture_and_encode_document_with_archives<'a>(&self, coordinator: &TransactionCoordinator,
        document: &om9_core::model::Document, sources: &[om9_kernel::brep::RegisteredTopologySource],
        cached: impl IntoIterator<Item = &'a [u8]>, held: Usage, cancel: &Cancellation) -> Result<ReservedBrepArchive> {
        self.with_captured_packets(coordinator, document, sources, cached, held, cancel, |packets| {
            if has_native_brep(document) {
                let native = NativeBrepRestoreHost::admit_packaged(ProcessLimits::default())?;
                self.encode_verified_mixed_document_candidate(&native, coordinator, document, packets, held, cancel)
            } else { self.encode_verified_document_candidate(coordinator, document, packets, held, cancel) }
        })
    }

    fn with_captured_packets<'a, T>(&self, coordinator: &TransactionCoordinator,
        document: &om9_core::model::Document, sources: &[om9_kernel::brep::RegisteredTopologySource],
        cached: impl IntoIterator<Item = &'a [u8]>, held: Usage, cancel: &Cancellation,
        finish: impl FnOnce(&[om9_archive::canonical_brep::ReservedBrepPacket<'_>]) -> Result<T>) -> Result<T> {
        use om9_core::geometry::Geometry;
        struct Envelope { bytes: Vec<u8>, _owner: ExternalMemoryReservation }
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let limits = *coordinator.limits();
        let geometry = || document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
            .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot));
        let count = geometry().filter(|item| matches!(item, Geometry::Brep(_) | Geometry::NativeBrep(_))).count();
        if count == 0 || count as u64 > limits.max_nodes || sources.len() > count { return Err(Error::Invalid("BRep save sources")); }
        let context = coordinator.capture_context(None)?.0;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let planned = Usage::array::<Envelope>(count)?.bytes
            .checked_add(Usage::array::<om9_archive::canonical_brep::ReservedBrepPacket<'_>>(count)?.bytes)
            .and_then(|n| n.checked_add(Usage::btree::<[u8; 32], ()>(count).ok()?.bytes))
            .ok_or(Error::Overflow("BRep save vectors"))?.max(1);
        let mut vectors = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, planned, physical(), held)?;
        let mut envelopes = Vec::new();
        envelopes.try_reserve_exact(count).map_err(|_| Error::Budget("BRep save vectors"))?;
        let mut packets = Vec::new();
        packets.try_reserve_exact(count).map_err(|_| Error::Budget("BRep save vectors"))?;
        let actual = Usage::array::<Envelope>(envelopes.capacity())?.bytes
            .checked_add(Usage::array::<om9_archive::canonical_brep::ReservedBrepPacket<'_>>(packets.capacity())?.bytes)
            .and_then(|n| n.checked_add(Usage::btree::<[u8; 32], ()>(count).ok()?.bytes))
            .ok_or(Error::Overflow("BRep save vectors"))?.max(1);
        if actual > vectors.bytes() {
            let extra = actual - vectors.bytes();
            coordinator.grow_external_memory_with_held(&mut vectors, extra, physical(), held)?;
        }
        vectors.set_allocated_bytes(actual)?;
        let mut wanted = std::collections::BTreeSet::new();
        for item in geometry() {
            match item {
                Geometry::Brep(source) => { wanted.insert(source.canonical_fingerprints().1); }
                Geometry::NativeBrep(_) => {},
                _ => {}
            }
        }
        for input in cached {
            let mut cached_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            let mut admit_cached = |peak: u64| {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                let needed = peak.saturating_sub(input.len() as u64).max(1);
                if needed > cached_owner.bytes() {
                    let extra = needed - cached_owner.bytes();
                    coordinator.grow_external_memory_with_held(&mut cached_owner, extra, physical(), held)?;
                }
                cached_owner.recheck_pending()
            };
            let decoded = if is_mixed_archive(input) {
                om9_archive::canonical_brep::decode_mixed_document_candidate_with_admission(input, &limits, &mut admit_cached)?
            } else {
                om9_archive::canonical_brep::decode_document_candidate_with_admission(input, &limits, &mut admit_cached)?
            };
            let retained = decoded.owned_storage_bytes(&limits)?.max(1);
            if retained > cached_owner.bytes() {
                let extra = retained - cached_owner.bytes();
                coordinator.grow_external_memory_with_held(&mut cached_owner, extra, physical(), held)?;
            }
            cached_owner.set_allocated_bytes(retained)?;
            cached_owner.shrink_to(retained)?;
            for (_, bytes) in decoded.packets() {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                let receipt = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
                let packet = om9_archive::canonical_brep::decode_packet_growing(bytes, &limits,
                    &coordinator.external_memory_account(), receipt,
                    |owner, extra| coordinator.grow_external_memory_with_held(owner, extra, physical(), held), || cancel.is_canceled())?;
                if wanted.remove(&packet.record().record().binding_fingerprint()) {
                    let requested = (bytes.len() as u64).max(1);
                    let mut owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, requested, physical(), held)?;
                    let mut copy = Vec::new();
                    copy.try_reserve_exact(bytes.len()).map_err(|_| Error::Budget("BRep cached packet"))?;
                    if copy.capacity() as u64 > owner.bytes() {
                        let extra = copy.capacity() as u64 - owner.bytes();
                        coordinator.grow_external_memory_with_held(&mut owner, extra, physical(), held)?;
                    }
                    owner.set_allocated_bytes(copy.capacity() as u64)?;
                    for chunk in bytes.chunks(1024 * 1024) {
                        if cancel.is_canceled() { return Err(Error::Canceled); }
                        copy.extend_from_slice(chunk);
                    }
                    owner.shrink_to((copy.capacity() as u64).max(1))?;
                    envelopes.push(Envelope { bytes: copy, _owner: owner });
                }
            }
        }
        for source in sources {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            source.proof().verify_unchanged_source(coordinator)?;
            let mut digest = Sha256::new();
            for chunk in source.source().as_bytes().chunks(1024 * 1024) {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                digest.update(chunk);
            }
            let binding = om9_core::brep::source_topology_fingerprint(digest.finalize().into(), source.topology());
            if !wanted.remove(&binding) { continue; }
            let mut capture_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            let named = source.capture_named_source(om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, &limits,
                |live, pending| {
                    if cancel.is_canceled() { return Err(Error::Canceled); }
                    let needed = live.checked_add(pending).ok_or(Error::Overflow("BRep capture admission"))?.max(1);
                    if needed > capture_owner.bytes() {
                        let extra = needed - capture_owner.bytes();
                        coordinator.grow_external_memory_with_held(&mut capture_owner, extra, physical(), held)?;
                    }
                    capture_owner.set_allocated_bytes(live)?;
                    capture_owner.recheck_pending()
                }, || cancel.is_canceled())?;
            let named_bytes = named.storage_bytes()?.max(1);
            if named_bytes > capture_owner.bytes() { return Err(Error::Budget("BRep captured source accounting")); }
            capture_owner.set_allocated_bytes(named_bytes)?;
            capture_owner.shrink_to(named_bytes)?;
            let mut envelope_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            let bytes = om9_archive::canonical_brep::encode_packet_with_admission(&named.canonical_record(), named.shape_set(), &limits,
                |live, pending| {
                    if cancel.is_canceled() { return Err(Error::Canceled); }
                    let needed = live.checked_add(pending).ok_or(Error::Overflow("BRep envelope admission"))?.max(1);
                    if needed > envelope_owner.bytes() {
                        let extra = needed - envelope_owner.bytes();
                        coordinator.grow_external_memory_with_held(&mut envelope_owner, extra, physical(), held)?;
                    }
                    envelope_owner.set_allocated_bytes(live)?;
                    envelope_owner.recheck_pending()
                })?;
            drop(named);
            drop(capture_owner);
            let retained = (bytes.capacity() as u64).max(1);
            envelope_owner.set_allocated_bytes(retained)?;
            envelope_owner.shrink_to(retained)?;
            source.proof().verify_unchanged_source(coordinator)?;
            envelopes.push(Envelope { bytes, _owner: envelope_owner });
        }
        if !wanted.is_empty() { return Err(Error::Missing("BRep save source packet")); }
        for envelope in &envelopes {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            packets.push(om9_archive::canonical_brep::decode_packet_growing(&envelope.bytes, &limits,
                &coordinator.external_memory_account(), owner,
                |owner, extra| coordinator.grow_external_memory_with_held(owner, extra, physical(), held), || cancel.is_canceled())?);
        }
        if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep capture context")); }
        for source in sources { source.proof().verify_unchanged_source(coordinator)?; }
        finish(&packets)
    }

    /// Admit the worker shipped beside the executable using the fingerprint
    /// embedded by the build, never one supplied by an archive. This pins the
    /// worker image only; its native dependencies are not authenticated here.
    pub fn admit_packaged(process: ProcessLimits) -> Result<Self> {
        let mut executable = std::env::current_exe().map_err(|error| Error::Io(error.to_string()))?;
        // Integration test executables live in target/debug/deps while the
        // packaged worker and app live in target/debug.
        #[cfg(feature = "test-fixtures")]
        if executable.parent().and_then(|path| path.file_name()).is_some_and(|name| name == "deps") {
            executable = executable.parent().and_then(|path| path.parent()).ok_or(Error::Invalid("application executable directory"))?.join("om9.exe");
        }
        Self::admit_packaged_at(&executable, option_env!("OM9_BREP_WORKER_SHA256"), process)
    }

    fn admit_packaged_at(executable: &Path, raw: Option<&str>, process: ProcessLimits) -> Result<Self> {
        let raw = raw.ok_or(Error::Unsupported("packaged BRep worker fingerprint"))?;
        let fingerprint = parse_worker_fingerprint(raw)?;
        let directory = executable.parent().ok_or(Error::Invalid("application executable directory"))?;
        #[cfg(windows)]
        let worker = directory.join("om9-brep-restore-worker.exe");
        #[cfg(not(windows))]
        let worker = directory.join("om9-brep-restore-worker");
        Self::admit(&worker, fingerprint, process)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_admit_packaged_at(executable: &Path, raw: Option<&str>, process: ProcessLimits) -> Result<Self> {
        Self::admit_packaged_at(executable, raw, process)
    }
    /// The fingerprint must come from the application layout or other trusted
    /// configuration, never from the input document.
    pub fn admit(path: &Path, fingerprint: [u8; 32], process: ProcessLimits) -> Result<Self> {
        process.validate().map_err(|error| Error::Io(format!("BRep worker limits: {error:?}")))?;
        if process.native_committed_bytes > 4 * 1024 * 1024 * 1024 {
            return Err(Error::Budget("BRep worker memory"));
        }
        let program = NativeProgram::admit(path, fingerprint)
            .map_err(|error| Error::Io(format!("BRep worker admission: {error:?}")))?;
        Ok(Self { program, process })
    }

    /// Restore an already admitted source record. The caller separately owns
    /// and accounts for the packet and all other live input in `held`; the
    /// record's reservation stays charged through the worker call. The packet
    /// is untrusted and is still checked against the source by the worker.
    pub fn restore_record(&self, coordinator: &TransactionCoordinator,
        record: &om9_archive::canonical_brep::ReservedCanonicalBrep, packet: &[u8],
        deflection: f64, angular: f64, held: Usage, cancel: &Cancellation) -> Result<ReservedBrepCandidate> {
        if !record.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Invalid("BRep record account"));
        }
        self.restore(coordinator, record.record().source(), record.record().topology(), packet,
            deflection, angular, held, cancel)
    }

    /// The envelope and record stay borrowed and accounted for this call;
    /// worker validation remains mandatory for the native payload.
    pub fn restore_packet(&self, coordinator: &TransactionCoordinator,
        packet: &om9_archive::canonical_brep::ReservedBrepPacket<'_>, deflection: f64, angular: f64,
        held: Usage, cancel: &Cancellation) -> Result<ReservedBrepCandidate> {
        self.restore_record(coordinator, packet.record(), packet.packet(), deflection, angular, held, cancel)
    }

    /// Encode and verify an in-memory candidate without publishing a file.
    /// All source and packet allocations remain caller-owned and accounted.
    pub fn encode_verified_document_candidate(&self, coordinator: &TransactionCoordinator,
        document: &om9_core::model::Document, packets: &[om9_archive::canonical_brep::ReservedBrepPacket<'_>],
        held: Usage, cancel: &Cancellation) -> Result<ReservedBrepArchive> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let limits = *coordinator.limits();
        let context = coordinator.capture_context(None)?.0;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut reservation = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let bytes = om9_archive::canonical_brep::encode_document_candidate(document, packets,
            &coordinator.external_memory_account(), &limits, |live, pending| {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                let needed = live.checked_add(pending).ok_or(Error::Overflow("BRep archive admission"))?.max(1);
                if needed > reservation.bytes() {
                    let extra = needed - reservation.bytes();
                    coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
                }
                reservation.set_allocated_bytes(live)?;
                reservation.recheck_pending()
            })?;
        let actual = (bytes.capacity() as u64).max(1);
        if actual > reservation.bytes() { return Err(Error::Budget("BRep archive accounting")); }
        reservation.set_allocated_bytes(actual)?;
        reservation.shrink_to(actual)?;
        let restored = self.restore_document(coordinator, &bytes, true,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?;
        if restored.document() != document { return Err(Error::Invalid("BRep archive document verification")); }
        drop(restored);
        if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep archive context")); }
        Ok(ReservedBrepArchive { bytes, reservation })
    }

    /// Encode a v7 mixed-source candidate and verify the independently decoded
    /// canonical and native payloads before returning its accounted bytes.
    pub fn encode_verified_mixed_document_candidate(&self, native_host: &NativeBrepRestoreHost,
        coordinator: &TransactionCoordinator, document: &om9_core::model::Document,
        packets: &[om9_archive::canonical_brep::ReservedBrepPacket<'_>], held: Usage,
        cancel: &Cancellation) -> Result<ReservedBrepArchive> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let limits = *coordinator.limits();
        let context = coordinator.capture_context(None)?.0;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut reservation = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let bytes = om9_archive::canonical_brep::encode_mixed_document_candidate(document, packets,
            &coordinator.external_memory_account(), &limits, |live, pending| {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                let needed = live.checked_add(pending).ok_or(Error::Overflow("mixed BRep archive admission"))?.max(1);
                if needed > reservation.bytes() {
                    let extra = needed - reservation.bytes();
                    coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
                }
                reservation.set_allocated_bytes(live)?;
                reservation.recheck_pending()
            })?;
        let actual = (bytes.capacity() as u64).max(1);
        if actual > reservation.bytes() { return Err(Error::Budget("mixed BRep archive accounting")); }
        reservation.set_allocated_bytes(actual)?;
        reservation.shrink_to(actual)?;
        let restored = self.restore_mixed_document(native_host, coordinator, &bytes, true,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?;
        if restored.document() != document { return Err(Error::Invalid("mixed BRep archive verification")); }
        drop(restored);
        if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("mixed BRep archive context")); }
        Ok(ReservedBrepArchive { bytes, reservation })
    }

    /// Verify again after staging without replacing the destination. The
    /// original operation must still be checked before the caller publishes.
    pub fn prepare_mixed_document_candidate(&self, native_host: &NativeBrepRestoreHost,
        coordinator: &TransactionCoordinator, path: &Path, document: &om9_core::model::Document,
        packets: &[om9_archive::canonical_brep::ReservedBrepPacket<'_>], held: Usage,
        cancel: &Cancellation) -> Result<PreparedBrepSave> {
        let context = coordinator.capture_context(None)?.0;
        let encoded = self.encode_verified_mixed_document_candidate(native_host, coordinator, document, packets, held, cancel)?;
        let limits = *coordinator.limits();
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let mut admit = |live: u64, pending: u64| -> Result<()> {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = live.checked_add(pending).ok_or(Error::Overflow("mixed BRep staging admission"))?.max(1);
            if needed > owner.bytes() {
                let extra = needed - owner.bytes();
                coordinator.grow_external_memory_with_held(&mut owner, extra, physical(), held)?;
            }
            owner.set_allocated_bytes(live)?;
            owner.recheck_pending()
        };
        let file = om9_archive::canonical_brep::prepare_verified_mixed_document_candidate(path, &encoded.bytes, &limits,
            cancel, &mut admit, |input| {
                if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("mixed BRep save context")); }
                let restored = self.restore_mixed_document(native_host, coordinator, input, true,
                    om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?;
                if restored.document() != document { return Err(Error::Invalid("mixed BRep saved document verification")); }
                if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("mixed BRep save context")); }
                Ok(())
            })?;
        Ok(PreparedBrepSave { file, context })
    }

    /// Encode and atomically publish a v6 document from already admitted,
    /// source-bound packets. Every packet is restored in the isolated worker
    /// before staging and again from the staging readback. The caller keeps the
    /// document, packets and their existing reservations alive throughout.
    /// This does not capture packets or connect the normal Save/Job Bag paths.
    pub fn save_document_candidate(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document,
        packets: &[om9_archive::canonical_brep::ReservedBrepPacket<'_>], held: Usage,
        cancel: &Cancellation) -> Result<()> {
        self.prepare_document_candidate(coordinator, path, document, packets, held, cancel)?.publish(coordinator, cancel)
    }

    /// Prepare the native-verified staging file without replacing the target.
    pub fn prepare_document_candidate(&self, coordinator: &TransactionCoordinator,
        path: &Path, document: &om9_core::model::Document,
        packets: &[om9_archive::canonical_brep::ReservedBrepPacket<'_>], held: Usage,
        cancel: &Cancellation) -> Result<PreparedBrepSave> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let limits = *coordinator.limits();
        let context = coordinator.capture_context(None)?.0;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let mut admit = |live: u64, pending: u64| -> Result<()> {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = live.checked_add(pending).ok_or(Error::Overflow("BRep save admission"))?.max(1);
            if needed > owner.bytes() {
                let extra = needed - owner.bytes();
                coordinator.grow_external_memory_with_held(&mut owner, extra, physical(), held)?;
            }
            owner.set_allocated_bytes(live)?;
            owner.recheck_pending()
        };
        let bytes = om9_archive::canonical_brep::encode_document_candidate(document, packets,
            &coordinator.external_memory_account(), &limits, &mut admit)?;
        admit(bytes.capacity() as u64, 0)?;
        let file = om9_archive::canonical_brep::prepare_verified_document_candidate(path, &bytes, &limits, cancel,
            &mut admit, |input| {
                if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep save context")); }
                let restored = self.restore_document(coordinator, input, true,
                    om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3, held, cancel)?;
                if restored.document() != document { return Err(Error::Invalid("BRep saved document verification")); }
                if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep save context")); }
                Ok(())
            })?;
        Ok(PreparedBrepSave { file, context })
    }

    /// Restore all unique BRep sources from an experimental document candidate.
    /// The caller keeps the archive alive and accounts other input in `held`.
    /// No document, display, or native topology proof is published here.
    pub fn restore_document<'a>(&self, coordinator: &TransactionCoordinator, input: &'a [u8],
        input_accounted: bool, deflection: f64, angular: f64, held: Usage, cancel: &Cancellation)
        -> Result<ReservedBrepDocument<'a>> {
        self.restore_document_inner(coordinator, input, input_accounted, deflection, angular, held, cancel, false)
    }

    /// Verify all v7 payloads and canonical packets without publishing or
    /// registering a scene. Both worker executables are pinned independently.
    pub fn restore_mixed_document<'a>(&self, native_host: &NativeBrepRestoreHost,
        coordinator: &TransactionCoordinator, input: &'a [u8], input_accounted: bool,
        deflection: f64, angular: f64, held: Usage, cancel: &Cancellation) -> Result<ReservedMixedBrepDocument<'a>> {
        use om9_core::geometry::Geometry;
        let context = coordinator.capture_context(None)?.0;
        let canonical = self.restore_document_inner(coordinator, input, input_accounted, deflection, angular, held, cancel, true)?;
        let document = canonical.document();
        let geometry = || document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
            .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot));
        let count = geometry().filter(|item| matches!(item, Geometry::NativeBrep(_))).count();
        if count == 0 || count as u64 > coordinator.limits().max_nodes { return Err(Error::Invalid("native BRep document coverage")); }
        let bytes = Usage::array::<ReservedNativeBrepDisplay>(count)?.bytes.max(1);
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut vectors = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical(), held)?;
        let mut native = Vec::new();
        native.try_reserve_exact(count).map_err(|_| Error::Budget("native BRep document vector"))?;
        let actual = Usage::array::<ReservedNativeBrepDisplay>(native.capacity())?.bytes.max(1);
        if actual > vectors.bytes() {
            let extra = actual - vectors.bytes();
            coordinator.grow_external_memory_with_held(&mut vectors, extra, physical(), held)?;
        }
        vectors.set_allocated_bytes(actual)?;
        let tolerance = document.effective_tolerance()?.absolute_mm;
        for item in geometry() {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if let Geometry::NativeBrep(source) = item {
                native.push(native_host.restore_display(coordinator, source, tolerance, deflection, angular, held, cancel)?);
            }
        }
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("mixed BRep document context")); }
        Ok(ReservedMixedBrepDocument { canonical, native, _native_vectors: vectors })
    }

    fn restore_document_inner<'a>(&self, coordinator: &TransactionCoordinator, input: &'a [u8],
        input_accounted: bool, deflection: f64, angular: f64, held: Usage, cancel: &Cancellation, mixed: bool)
        -> Result<ReservedBrepDocument<'a>> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let context = coordinator.capture_context(None)?.0;
        let limits = *coordinator.limits();
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut document_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let mut admit_document = |peak: u64| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = if input_accounted { peak.saturating_sub(input.len() as u64) } else { peak }.max(1);
            if needed > document_owner.bytes() {
                let extra = needed - document_owner.bytes();
                coordinator.grow_external_memory_with_held(&mut document_owner, extra, physical(), held)?;
            }
            document_owner.recheck_pending()
        };
        let document = if mixed {
            om9_archive::canonical_brep::decode_mixed_document_candidate_with_admission(input, &limits, &mut admit_document)?
        } else {
            om9_archive::canonical_brep::decode_document_candidate_with_admission(input, &limits, &mut admit_document)?
        };
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let actual = document.owned_storage_bytes(&limits)?.checked_add(if input_accounted { 0 } else { input.len() as u64 })
            .ok_or(Error::Overflow("BRep document owner"))?.max(1);
        if actual > document_owner.bytes() {
            let extra = actual - document_owner.bytes();
            coordinator.grow_external_memory_with_held(&mut document_owner, extra, physical(), held)?;
        }
        document_owner.set_allocated_bytes(actual)?;
        document_owner.shrink_to(actual)?;
        let count = document.packets().len();
        let planned = Usage::array::<om9_archive::canonical_brep::ReservedBrepPacket<'_>>(count)?.bytes
            .checked_add(Usage::array::<([u8; 16], ReservedBrepCandidate)>(count)?.bytes)
            .ok_or(Error::Overflow("BRep document vectors"))?.max(1);
        let mut vectors = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, planned, physical(), held)?;
        let mut packets = Vec::new();
        let mut candidates = Vec::new();
        packets.try_reserve_exact(count).map_err(|_| Error::Budget("BRep document packets"))?;
        candidates.try_reserve_exact(count).map_err(|_| Error::Budget("BRep document candidates"))?;
        let actual_vectors = Usage::array::<om9_archive::canonical_brep::ReservedBrepPacket<'_>>(packets.capacity())?.bytes
            .checked_add(Usage::array::<([u8; 16], ReservedBrepCandidate)>(candidates.capacity())?.bytes)
            .ok_or(Error::Overflow("BRep document vectors"))?.max(1);
        if actual_vectors > vectors.bytes() {
            let extra = actual_vectors - vectors.bytes();
            coordinator.grow_external_memory_with_held(&mut vectors, extra, physical(), held)?;
        }
        vectors.set_allocated_bytes(actual_vectors)?;
        for (_, bytes) in document.packets() {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            packets.push(om9_archive::canonical_brep::decode_packet_growing(bytes, &limits,
                &coordinator.external_memory_account(), owner,
                |owner, extra| coordinator.grow_external_memory_with_held(owner, extra, physical(), held), || cancel.is_canceled())?);
        }
        let mut scratch = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        document.validate_packets(&packets, &coordinator.external_memory_account(), &limits, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = live.checked_add(pending).ok_or(Error::Overflow("BRep document validation"))?.max(1);
            if needed > scratch.bytes() {
                let extra = needed - scratch.bytes();
                coordinator.grow_external_memory_with_held(&mut scratch, extra, physical(), held)?;
            }
            scratch.set_allocated_bytes(live)?;
            scratch.recheck_pending()
        })?;
        drop(scratch);
        for ((id, _), packet) in document.packets().iter().zip(&packets) {
            candidates.push((*id, self.restore_packet(coordinator, packet, deflection, angular, held, cancel)?));
        }
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep document worker context")); }
        drop(packets);
        // The retained vector receipt can shrink after the temporary packet
        // vector has been dropped, without releasing the candidates' owners.
        let retained = Usage::array::<([u8; 16], ReservedBrepCandidate)>(candidates.capacity())?.bytes.max(1);
        vectors.set_allocated_bytes(retained)?;
        vectors.shrink_to(retained)?;
        Ok(ReservedBrepDocument { document, candidates, _document_reservation: document_owner, _vector_reservation: vectors })
    }

    /// Prepare one detached candidate. Native parsing is contained, but this
    /// does not make the worker a security sandbox or prove native memory safety.
    pub fn restore(&self, coordinator: &TransactionCoordinator, source: &str, topology: &BrepTopology,
        packet: &[u8], deflection: f64, angular: f64, held: Usage, cancel: &Cancellation) -> Result<ReservedBrepCandidate> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let context = coordinator.capture_context(None)?.0;
        let limits = *coordinator.limits();
        let input_limit = limits.max_bytes.min(self.process.native_committed_bytes / 2).min(1024 * 1024 * 1024);
        if input_limit == 0 { return Err(Error::Budget("BRep worker input")); }
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let process_bytes = self.process.reservation_bytes().map_err(|error| Error::Io(format!("BRep worker budget: {error:?}")))?;
        let process_guard = coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, process_bytes, physical(), held)?;
        let path = std::env::temp_dir().join(format!("om9-brep-restore-{}.json", Uuid::new_v4()));
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path)
            .map_err(|error| Error::Io(error.to_string()))?;
        let staging = StagedInput(path);
        let mut writer = RequestWriter { file, hash: Sha256::new(), bytes: 0, limit: input_limit, cancel, error: None };
        if let Err(error) = serde_json::to_writer(&mut writer, &(1u32, source, topology, packet, deflection, angular)) {
            return Err(writer.error.take().unwrap_or_else(|| Error::Io(error.to_string())));
        }
        writer.flush().map_err(|error| Error::Io(error.to_string()))?;
        let expected: [u8; 32] = writer.hash.clone().finalize().into();
        drop(writer);
        let verified = VerifiedFile::open(&staging.0, input_limit, cancel)
            .map_err(|error| if cancel.is_canceled() { Error::Canceled } else { Error::Io(format!("BRep worker input: {error:?}")) })?;
        if verified.fingerprint() != expected { return Err(Error::Stale("BRep worker input changed")); }
        let hash = expected.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let args = [staging.0.as_os_str().to_owned(), hash.into(), input_limit.to_string().into(),
            self.process.native_committed_bytes.to_string().into()];
        let output = self.program.run(&args, &self.process, cancel).map_err(|error| {
            if cancel.is_canceled() { Error::Canceled } else { Error::Io(format!("BRep worker: {error:?}")) }
        })?;
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if !output.stats.exited || output.stats.active_processes != 0 { return Err(Error::Invalid("BRep worker lifetime")); }
        let mut decode = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
        let estimated = om9_archive::admit_json_usage_with_admission(&output.stdout, &limits, |needed| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if needed > decode.bytes() {
                let extra = needed - decode.bytes();
                coordinator.grow_external_memory_with_held(&mut decode, extra, physical(), held)?;
            }
            decode.recheck_pending()
        })?;
        let candidate: Brep = serde_json::from_slice(&output.stdout).map_err(|_| Error::Invalid("BRep worker response"))?;
        let actual = candidate.storage_usage(&limits)?.bytes;
        if actual > estimated.bytes || actual > decode.bytes() { return Err(Error::Budget("BRep worker response")); }
        candidate.validate_admitted(&limits, &mut |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let needed = live.checked_add(pending).ok_or(Error::Overflow("BRep worker validation"))?;
            if needed > decode.bytes() {
                let extra = needed - decode.bytes();
                coordinator.grow_external_memory_with_held(&mut decode, extra, physical(), held)?;
            }
            decode.set_allocated_bytes(live)?;
            decode.recheck_pending()
        })?;
        if candidate.source() != source { return Err(Error::Invalid("BRep worker source mismatch")); }
        validate_topology(topology, candidate.topology(), &limits, coordinator, &mut decode, held, cancel)?;
        drop((output, verified, staging, process_guard));
        decode.set_allocated_bytes(actual)?;
        Ok(ReservedBrepCandidate { candidate, reservation: decode, context })
    }
}

fn parse_worker_fingerprint(raw: &str) -> Result<[u8; 32]> {
    if raw.len() != 64 || !raw.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(Error::Invalid("packaged BRep worker fingerprint"));
    }
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&raw[index * 2..index * 2 + 2], 16)
            .map_err(|_| Error::Invalid("packaged BRep worker fingerprint"))?;
    }
    Ok(digest)
}

fn validate_topology(expected: &BrepTopology, actual: &BrepTopology, limits: &Limits,
    coordinator: &TransactionCoordinator, reservation: &mut ExternalMemoryReservation, held: Usage, cancel: &Cancellation) -> Result<()> {
    let ids = expected.faces.len().checked_add(expected.edges.len()).and_then(|n| n.checked_add(expected.vertex_ids.len()))
        .ok_or(Error::Overflow("BRep worker topology"))?;
    if ids as u64 > limits.max_nodes || expected.face_edges.len() != actual.face_edges.len() { return Err(Error::Invalid("BRep worker topology")); }
    let scratch = Usage::array::<Uuid>(ids.checked_mul(2).ok_or(Error::Overflow("BRep worker topology"))?)?.bytes
        .checked_add(Usage::array::<(Uuid, Uuid)>(expected.face_edges.len().checked_mul(2).ok_or(Error::Overflow("BRep worker topology"))?)?.bytes)
        .ok_or(Error::Overflow("BRep worker topology"))?;
    let baseline = reservation.bytes();
    let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
    coordinator.grow_external_memory_with_held(reservation, scratch, physical, held)?;
    reservation.recheck_pending()?;
    for (a, b) in [(&expected.faces, &actual.faces), (&expected.edges, &actual.edges), (&expected.vertex_ids, &actual.vertex_ids)] {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if a.len() != b.len() { return Err(Error::Invalid("BRep worker topology")); }
        let mut left = Vec::new(); let mut right = Vec::new();
        left.try_reserve_exact(a.len()).map_err(|_| Error::Budget("BRep worker topology"))?;
        right.try_reserve_exact(b.len()).map_err(|_| Error::Budget("BRep worker topology"))?;
        left.extend_from_slice(a); right.extend_from_slice(b);
        left.sort_unstable(); right.sort_unstable();
        if left != right { return Err(Error::Invalid("BRep worker topology")); }
    }
    let pairs = |topology: &BrepTopology| -> Result<Vec<(Uuid, Uuid)>> {
        let mut out = Vec::new();
        out.try_reserve_exact(topology.face_edges.len()).map_err(|_| Error::Budget("BRep worker topology"))?;
        for (index, [face, edge]) in topology.face_edges.iter().enumerate() {
            if index % 1024 == 0 && cancel.is_canceled() { return Err(Error::Canceled); }
            out.push((*topology.faces.get(*face as usize).ok_or(Error::Invalid("BRep worker face"))?,
                *topology.edges.get(*edge as usize).ok_or(Error::Invalid("BRep worker edge"))?));
        }
        out.sort_unstable();
        Ok(out)
    };
    if pairs(expected)? != pairs(actual)? { return Err(Error::Invalid("BRep worker adjacency")); }
    reservation.shrink_to(baseline)?;
    Ok(())
}
