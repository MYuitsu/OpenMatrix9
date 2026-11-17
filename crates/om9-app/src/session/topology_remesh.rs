// SPDX-License-Identifier: LGPL-2.1-or-later
//! Private remesh worker. A completed candidate is not a published scene.
use super::{Controller, PreparedEditWire};
use om9_core::{Error, Result, budget::memory::MemoryClass,
    transaction::ExternalMemoryReservation};
use om9_kernel::brep::{CapturedTopologySnapshot, SharedTopologyProof, capture_source_topology};
use std::sync::{Arc, Weak, atomic::{AtomicBool, Ordering}, mpsc};

mod viewport_detail;
pub(super) use viewport_detail::DetailChange;

struct HistoryRebuild {
    evidence_stamp: om9_core::identity::SourceStamp,
    source: om9_core::brep::CanonicalBrep,
    proof: Arc<SharedTopologyProof>,
    profile: Option<om9_core::brep::BrepRepresentationProfile>,
}

pub(super) struct HistoryRebuilds {
    jobs: Vec<HistoryRebuild>,
    native_jobs: Vec<om9_core::geometry::NativeBrepSource>,
    native_tolerance: Option<f64>,
    _metadata: ExternalMemoryReservation,
    cache_guard: Option<super::topology_cache::HistoryProofCacheGuard>,
}
struct HistoryWire {
    stamp: om9_core::identity::SourceStamp,
    candidate: om9_core::brep::Brep,
    buffers: om9_scene::scene::PreparedBrepBuffers,
    reservation: ExternalMemoryReservation,
    profile: [u64; 2],
}
pub(super) struct HistoryWireBatch {
    _cache_guard: Option<super::topology_cache::HistoryProofCacheGuard>,
}
struct HistoryWireOwner {
    _reservation: ExternalMemoryReservation,
}


impl Controller {
    pub(super) fn prepare_history_rebuilds(&self, candidate: &om9_core::transaction::PreparedHistory,
        held: om9_core::budget::Usage) -> Result<HistoryRebuilds> {
        use om9_core::{budget::Usage, geometry::Geometry};
        let cache_guard = self.topology_cache.as_ref().map(|cache| cache.pin_for_history()).transpose()?;
        let document = candidate.candidate_world().documents.get(&self.document)
            .ok_or(Error::Missing("history scene document"))?;
        let count = document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
            .filter(|entity| matches!(entity.geometry, Geometry::Brep(_))).count();
        let native_count = document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
            .filter(|entity| matches!(entity.geometry, Geometry::NativeBrep(_))).count();
        // Only one of these sets lives at a time; reserve the larger bound.
        let source_set_bytes = Usage::btree::<om9_core::brep::BrepSourceIdentity, ()>(count)?.bytes
            .max(Usage::btree::<om9_core::geometry::NativeBrepSourceIdentity, ()>(native_count)?.bytes);
        let bytes = Usage::array::<HistoryRebuild>(count)?.bytes
            .checked_add(Usage::array::<om9_core::geometry::NativeBrepSource>(native_count)?.bytes)
            .and_then(|n| n.checked_add(source_set_bytes))
            .ok_or(Error::Overflow("history rebuild metadata"))?.max(1);
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut metadata = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
        let mut jobs = Vec::new();
        jobs.try_reserve_exact(count).map_err(|_| Error::Budget("history rebuild metadata"))?;
        let mut native_jobs = Vec::new();
        native_jobs.try_reserve_exact(native_count).map_err(|_| Error::Budget("native history rebuild metadata"))?;
        let actual = Usage::array::<HistoryRebuild>(jobs.capacity())?.bytes
            .checked_add(Usage::array::<om9_core::geometry::NativeBrepSource>(native_jobs.capacity())?.bytes)
            .and_then(|n| n.checked_add(source_set_bytes))
            .ok_or(Error::Overflow("history rebuild metadata"))?.max(1);
        if actual > metadata.bytes() {
            self.coordinator.grow_external_memory_with_held(&mut metadata, actual - bytes, physical, held)?;
        }
        metadata.set_allocated_bytes(actual)?;
        let mut queued_sources = std::collections::BTreeSet::new();
        let store = self.coordinator.representation_store();
        for entity in document.entities.values() {
            let Geometry::Brep(source) = &entity.geometry else { continue; };
            if queued_sources.contains(&source.identity()) { continue; }
            let stamp = document.stamp(entity.id)?;
            if store.lock().unwrap_or_else(|e| e.into_inner()).get(source).is_ok() { continue; }
            let proof = self.coordinator.history_candidate_evidence_for::<SharedTopologyProof>(candidate, &stamp)
                .or_else(|| self.topology_cache.as_ref().and_then(|cache| cache.find_source(Some(entity.id), document)))
                .or_else(|| self.coordinator.history_candidate_shared_evidence_for::<SharedTopologyProof>(candidate, self.document, source)
                    .filter(|proof| proof.shares_canonical_allocations(source)))
                .or_else(|| self.topology_cache.as_ref().and_then(|cache| cache.find_shared_source(document, source)))
                .ok_or(Error::Missing("history BRep topology proof"))?;
            let profile = self.topology_cache.as_ref().and_then(|cache| cache.rebuild_profile(source.identity()));
            jobs.push(HistoryRebuild { evidence_stamp: proof.stamp(), source: source.clone(), proof, profile });
            queued_sources.insert(source.identity());
        }
        for entity in document.definitions.values().flat_map(|definition| &definition.members) {
            let Geometry::Brep(source) = &entity.geometry else { continue; };
            if queued_sources.contains(&source.identity()) { continue; }
            if store.lock().unwrap_or_else(|e| e.into_inner()).get(source).is_ok() { continue; }
            let proof = self.coordinator.history_candidate_shared_evidence_for::<SharedTopologyProof>(candidate, self.document, source)
                .filter(|proof| proof.shares_canonical_allocations(source))
                .or_else(|| self.topology_cache.as_ref().and_then(|cache| cache.find_shared_source(document, source)));
            // A definition may be unused. If it lacks shared proof, leave its
            // cache miss for scene preparation to refuse only if encountered.
            if let Some(proof) = proof {
                let profile = self.topology_cache.as_ref().and_then(|cache| cache.rebuild_profile(source.identity()));
                jobs.push(HistoryRebuild { evidence_stamp: proof.stamp(), source: source.clone(), proof, profile });
                queued_sources.insert(source.identity());
            }
        }
        drop(queued_sources);
        let mut queued_native = std::collections::BTreeSet::new();
        for entity in document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members)) {
            let Geometry::NativeBrep(source) = &entity.geometry else { continue; };
            if queued_native.contains(&source.identity()) || store.lock().unwrap_or_else(|e| e.into_inner()).get_native(source).is_ok() { continue; }
            native_jobs.push(source.clone());
            queued_native.insert(source.identity());
        }
        drop(queued_native);
        let retained_metadata = actual.checked_sub(source_set_bytes).ok_or(Error::Overflow("history rebuild metadata"))?.max(1);
        metadata.set_allocated_bytes(retained_metadata)?;
        metadata.shrink_to(retained_metadata)?;
        let native_tolerance = if native_jobs.is_empty() { None } else { Some(document.effective_tolerance()?.absolute_mm) };
        Ok(HistoryRebuilds { jobs, native_jobs, native_tolerance, _metadata: metadata, cache_guard })
    }
}

impl HistoryRebuilds {
    pub(super) fn run(self, coordinator: &om9_core::transaction::TransactionCoordinator,
        prepared: &mut om9_core::transaction::PreparedHistory,
        held: om9_core::budget::Usage, cancel: &AtomicBool) -> Result<HistoryWireBatch> {
        use om9_core::budget::Usage;
        let limits = *coordinator.limits();
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let pending_count = self.jobs.len().checked_add(self.native_jobs.len()).ok_or(Error::Overflow("history rebuild count"))?;
        if pending_count != 0 {
            coordinator.begin_pending_history_brep_representations(prepared, pending_count, held)?;
        }
        for source in &self.native_jobs {
            if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
            let store = coordinator.representation_store();
            if store.lock().unwrap_or_else(|e| e.into_inner()).get_native(source).is_ok() { continue; }
            let mut reservation = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
            let check = || if cancel.load(Ordering::Acquire) { Err(Error::Canceled) } else { Ok(()) };
            let mut display = super::files::rebuild_native_display(coordinator, source,
                self.native_tolerance.ok_or(Error::Missing("native history tolerance"))?, &mut reservation, held, &check)?;
            let display_bytes = display.display_bytes()?;
            let owner_bytes = (std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>() + 2 * std::mem::size_of::<usize>()) as u64;
            let store_bytes = Usage::btree::<om9_core::geometry::NativeBrepSourceIdentity, om9_core::geometry::NativeBrepRepresentation>(1)?.bytes
                .checked_add((std::mem::size_of::<Vec<u8>>() + 2 * std::mem::size_of::<usize>()) as u64)
                .ok_or(Error::Overflow("native history store"))?;
            let total = display_bytes.checked_add(owner_bytes).and_then(|n| n.checked_add(store_bytes))
                .ok_or(Error::Overflow("native history display"))?.max(1);
            if total > reservation.bytes() {
                let extra = total - reservation.bytes();
                coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(display_bytes)?;
            let owner = Arc::new(std::sync::Mutex::new(reservation));
            owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(display_bytes + owner_bytes)?;
            display.bind_owner(source, owner.clone(), &coordinator.external_memory_account(), &limits)?;
            check()?;
            prepared.insert_pending_native_brep_representation(source, display, &limits)?;
            let mut owner = owner.lock().unwrap_or_else(|e| e.into_inner());
            owner.set_allocated_bytes(total)?;
            owner.shrink_to(total)?;
        }
        for job in &self.jobs {
            if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
            let store = coordinator.representation_store();
            if store.lock().unwrap_or_else(|e| e.into_inner()).get(&job.source).is_ok() { continue; }
            let _permit = job.proof.acquire_worker()?;
            let mut reservation = coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical(), held)?;
            let refusal = std::cell::RefCell::new(None);
            let mut admit = |live: u64, pending: u64| -> Result<()> {
                let result = (|| {
                    if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    let needed = live.checked_add(pending).ok_or(Error::Overflow("history rebuild reservation"))?.max(1);
                    if needed > reservation.bytes() {
                        let extra = needed - reservation.bytes();
                        coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
                    }
                    reservation.set_allocated_bytes(live)?;
                    reservation.recheck_pending()
                })();
                if let Err(error) = &result { *refusal.borrow_mut() = Some(error.clone()); }
                result
            };
            let source = job.proof.bind_history_candidate(coordinator, &job.evidence_stamp, &job.source, &limits, &mut admit)?;
            let (deflection, angular) = match job.profile {
                Some(om9_core::brep::BrepRepresentationProfile::Certified { linear, angular, adapter_version })
                    if adapter_version == om9_kernel::brep::OCCT_REPRESENTATION_ADAPTER_VERSION => {
                    let profile = (f64::from_bits(linear), f64::from_bits(angular));
                    if validate_remesh_profile(profile.0, profile.1).is_ok() { profile }
                    else { (om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3) }
                }
                _ => (om9_kernel::creation::DEFAULT_DEFLECTION_MM, 0.3),
            };
            let candidate = capture_source_topology(source.source(), deflection,
                angular, &limits, &mut admit).map_err(|error| refusal.borrow_mut().take()
                    .unwrap_or_else(|| Error::Io(format!("history BRep rebuild: {error}"))))?;
            let mut representation = candidate.into_representation_for_source(&source, &job.source, &limits, &mut admit)?;
            representation.set_certified_profile(angular, om9_kernel::brep::OCCT_REPRESENTATION_ADAPTER_VERSION)?;
            drop(admit);
            if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
            let display_bytes = representation.display_allocation_bytes()?;
            let owner_bytes = (std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>()
                + 2 * std::mem::size_of::<usize>()) as u64;
            let store_bytes = Usage::btree::<(om9_core::brep::BrepSourceIdentity, om9_core::brep::BrepRepresentationProfile), om9_core::brep::BrepRepresentation>(1)?.bytes
                .checked_add(Usage::btree::<om9_core::brep::BrepSourceIdentity, om9_core::brep::BrepRepresentationProfile>(1)?.bytes)
                .and_then(|n| n.checked_add(om9_core::brep::BrepRepresentation::source_receipt_bytes()))
                .ok_or(Error::Overflow("history BRep store metadata"))?;
            let retained = display_bytes.checked_add(owner_bytes).and_then(|n| n.checked_add(store_bytes))
                .ok_or(Error::Overflow("history display owner"))?.max(1);
            if retained > reservation.bytes() {
                let extra = retained - reservation.bytes();
                coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
            }
            reservation.set_allocated_bytes(display_bytes)?;
            let owner = Arc::new(std::sync::Mutex::new(reservation));
            owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(display_bytes + owner_bytes)?;
            representation.bind_display_owner(&job.source, owner.clone(), &coordinator.external_memory_account(), &limits)?;
            if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
            prepared.insert_pending_brep_representation(&job.source, representation, &limits, coordinator, held)?;
            let mut owner = owner.lock().unwrap_or_else(|e| e.into_inner());
            owner.set_allocated_bytes(retained)?;
            owner.shrink_to(retained)?;
        }
        Ok(HistoryWireBatch { _cache_guard: self.cache_guard })
    }
}

fn prepare_history_wire(coordinator: &om9_core::transaction::TransactionCoordinator,
    stamp: om9_core::identity::SourceStamp, source: &om9_core::brep::CanonicalBrep,
    held: om9_core::budget::Usage, cancel: &AtomicBool) -> Result<HistoryWire> {
    let limits = *coordinator.limits();
    let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
    if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
    let representation = coordinator.brep_representation(source)?;
    let profile = match representation.profile() {
        om9_core::brep::BrepRepresentationProfile::Certified { linear, angular, .. } => [linear, angular],
        om9_core::brep::BrepRepresentationProfile::Unspecified { .. } => [0; 2],
    };
    let mut reservation = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical(), held)?;
    let baseline = source.storage_usage(&limits)?.bytes.checked_add(representation.display_allocation_bytes()?)
        .ok_or(Error::Overflow("history wire validation"))?;
    let candidate = representation.materialize_admitted(source, &limits, &mut |live, pending| {
        if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let extra_live = live.checked_sub(baseline).ok_or(Error::Invalid("history wire validation accounting"))?;
        let needed = extra_live.checked_add(pending).ok_or(Error::Overflow("history wire validation"))?.max(1);
        if needed > reservation.bytes() {
            let extra = needed - reservation.bytes();
            coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
        }
        reservation.set_allocated_bytes(extra_live)?;
        reservation.recheck_pending()
    })?;
    reservation.set_allocated_bytes(0)?;
    let (buffers, actual) = prepare_source_buffers(source.source(), &candidate, &limits, |live, pending| {
        if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let needed = live.checked_add(pending).ok_or(Error::Overflow("history wire reservation"))?.max(1);
        if needed > reservation.bytes() {
            let extra = needed - reservation.bytes();
            coordinator.grow_external_memory_with_held(&mut reservation, extra, physical(), held)?;
        }
        reservation.set_allocated_bytes(live)?;
        reservation.recheck_pending()
    })?;
    reservation.set_allocated_bytes(actual.bytes)?;
    reservation.shrink_to(actual.bytes.max(1))?;
    Ok(HistoryWire { stamp, candidate, buffers, reservation, profile })
}

pub(super) fn apply_prepared_edit_wire(coordinator: &om9_core::transaction::TransactionCoordinator,
    mut built: super::ReservedCandidateScene, mut wire: PreparedEditWire, held: om9_core::budget::Usage)
    -> Result<super::ReservedCandidateScene> {
    let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
    let estimate = built.scene.render_snapshot_metadata_estimate()?.bytes
        .checked_add(built.scene.picking_arena_bytes()?).ok_or(Error::Overflow("prepared wire metadata"))?;
    let mut metadata = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, estimate.max(1), physical(), held)?;
    let owner_bytes = (std::mem::size_of::<HistoryWireOwner>() + 2*std::mem::size_of::<usize>()) as u64;
    coordinator.grow_external_memory_with_held(&mut wire.reservation, owner_bytes, physical(), held)?;
    wire.reservation.set_allocated_bytes(wire.reservation.bytes())?;
    let stamp = built.scene.document.stamp(wire.object)?;
    let lease: Arc<dyn Send + Sync> = Arc::new(HistoryWireOwner { _reservation: wire.reservation });
    // Retain a caller-side lease until rejected buffer arguments have dropped.
    let scene = built.scene.try_install_prepared_brep_wires(&stamp, wire.buffers, lease.clone())?;
    if scene.managed_memory_allowance()? > metadata.bytes() { return Err(Error::Budget("prepared wire metadata")); }
    scene.transfer_memory_allowance(&mut metadata, coordinator, held)?;
    let old_scene = std::mem::replace(&mut built.scene, scene);
    let old_reservation = std::mem::replace(&mut built.reservation, metadata);
    drop(old_scene); drop(old_reservation);
    let shared = built.scene.shared_buffer_bytes();
    if shared == 0 { built.buffer_owner = None; }
    else if let Some(owner) = built.buffer_owner.as_mut() { owner.bytes = shared; }
    else { return Err(Error::Invalid("prepared shared buffer owner")); }
    Ok(built)
}

/// Add source-derived wires to an unpublished Open scene. The archive
/// reservation retains the original scene's shared buffers until replacement;
/// each new wire buffer has its own owner and the temporary scene snapshot has
/// a separate reservation while both scene versions coexist.
pub(super) fn prepare_open_scene_wires(initial: &mut super::PreparedInitialScene,
    archive: &mut ExternalMemoryReservation, held: om9_core::budget::Usage,
    cancel: &AtomicBool) -> Result<()> {
    use om9_core::{budget::Usage, geometry::Geometry};
    let coordinator = &initial.coordinator;
    let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
    for index in 0..initial.scene.objects.len() {
        if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let object = &initial.scene.objects[index];
        let Geometry::Brep(source) = &initial.scene.entity_for(&initial.scene.document, object)?.geometry else { continue; };
        if initial.scene.brep_occurrence_has_interior_wires(object)? { continue; }
        let source = source.clone();
        let stamp = object.source;
        let path_bytes = Usage::array::<om9_core::identity::ObjectId>(object.path.len())?.bytes.max(1);
        let mut path_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, path_bytes, physical(), held)?;
        let mut path = Vec::new();
        path.try_reserve_exact(object.path.len()).map_err(|_| Error::Budget("Open wire occurrence path"))?;
        let actual_path = Usage::array::<om9_core::identity::ObjectId>(path.capacity())?.bytes.max(1);
        if actual_path > path_owner.bytes() {
            coordinator.grow_external_memory_with_held(&mut path_owner, actual_path - path_bytes, physical(), held)?;
        }
        path.extend_from_slice(&object.path);
        path_owner.set_allocated_bytes(actual_path)?;
        let mut wire = prepare_history_wire(coordinator, stamp, &source, held, cancel)?;
        let owner_bytes = (std::mem::size_of::<HistoryWireOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
        let topology_charge = if wire.candidate.topology_owner_covers_allocation(&coordinator.external_memory_account())? {
            0
        } else { wire.candidate.topology_allocation_bytes()? };
        let topology_bytes = topology_charge.checked_add(owner_bytes)
            .ok_or(Error::Overflow("Open wire topology owner"))?;
        let overhead = topology_bytes.checked_add(owner_bytes).ok_or(Error::Overflow("Open wire owner"))?;
        coordinator.grow_external_memory_with_held(&mut wire.reservation, overhead, physical(), held)?;
        wire.reservation.set_allocated_bytes(wire.reservation.bytes())?;
        let topology_reservation = wire.reservation.split_off(topology_bytes)?;
        let candidate = wire.candidate;
        let display_lease: Arc<dyn std::any::Any + Send + Sync> = Arc::new(HistoryWireOwner { _reservation: topology_reservation });
        let buffer_lease: Arc<dyn Send + Sync> = Arc::new(HistoryWireOwner { _reservation: wire.reservation });
        let profile = wire.profile;
        let old_metadata = initial.scene.managed_memory_allowance()?;
        let estimate = initial.scene.render_snapshot_metadata_estimate()?.bytes
            .checked_add(initial.scene.picking_arena_bytes()?).ok_or(Error::Overflow("Open wire scene metadata"))?;
        let _snapshot = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, estimate.max(1), physical(), held)?;
        let scene = initial.scene.try_replace_brep_occurrence_display(&wire.stamp, &path, &candidate,
            wire.buffers, profile, display_lease, buffer_lease)?;
        let new_metadata = scene.managed_memory_allowance()?;
        if new_metadata > estimate { return Err(Error::Budget("Open wire scene metadata")); }
        if new_metadata > old_metadata {
            coordinator.grow_external_memory_with_held(archive, new_metadata - old_metadata, physical(), held)?;
        }
        initial.scene = scene;
    }
    Ok(())
}

impl HistoryWireBatch {
    pub(super) fn apply(self, coordinator: &om9_core::transaction::TransactionCoordinator,
        built: super::ReservedCandidateScene, held: om9_core::budget::Usage,
        cancel: &AtomicBool) -> Result<super::ReservedCandidateScene> {
        let result = prepare_missing_scene_wires_cancellable(coordinator, built, held, cancel);
        drop(self);
        result
    }
}

/// Complete an edit candidate before its transaction is committed. Existing
/// occurrence buffers are reused, including across placement-only updates.
pub(super) fn prepare_missing_scene_wires(coordinator: &om9_core::transaction::TransactionCoordinator,
    built: super::ReservedCandidateScene, held: om9_core::budget::Usage) -> Result<super::ReservedCandidateScene> {
    prepare_missing_scene_wires_cancellable(coordinator, built, held, &AtomicBool::new(false))
}
fn prepare_missing_scene_wires_cancellable(coordinator: &om9_core::transaction::TransactionCoordinator,
    mut built: super::ReservedCandidateScene, held: om9_core::budget::Usage,
    cancel: &AtomicBool) -> Result<super::ReservedCandidateScene> {
    use om9_core::{budget::Usage, geometry::Geometry};
    let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
    for index in 0..built.scene.objects.len() {
        if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let object = &built.scene.objects[index];
        let Geometry::Brep(source) = &built.scene.entity_for(&built.scene.document, object)?.geometry else { continue; };
        if built.scene.brep_occurrence_has_interior_wires(object)? { continue; }
        let source = source.clone();
        let stamp = object.source;
        let bytes = Usage::array::<om9_core::identity::ObjectId>(object.path.len())?.bytes.max(1);
        let mut path_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical(), held)?;
        let mut path = Vec::new();
        path.try_reserve_exact(object.path.len()).map_err(|_| Error::Budget("edit wire occurrence path"))?;
        let actual = Usage::array::<om9_core::identity::ObjectId>(path.capacity())?.bytes.max(1);
        if actual > path_owner.bytes() {
            coordinator.grow_external_memory_with_held(&mut path_owner, actual - bytes, physical(), held)?;
        }
        path.extend_from_slice(&object.path);
        path_owner.set_allocated_bytes(actual)?;
        let wire = prepare_history_wire(coordinator, stamp, &source, held, cancel)?;
        built = apply_history_wire(coordinator, built, wire, &path, held, cancel)?;
    }
    Ok(built)
}

fn apply_history_wire(coordinator: &om9_core::transaction::TransactionCoordinator,
    mut built: super::ReservedCandidateScene, mut wire: HistoryWire, path: &[om9_core::identity::ObjectId],
    held: om9_core::budget::Usage, cancel: &AtomicBool) -> Result<super::ReservedCandidateScene> {
    if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
    let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
    let estimate = built.scene.render_snapshot_metadata_estimate()?.bytes
        .checked_add(built.scene.picking_arena_bytes()?).ok_or(Error::Overflow("history wire scene metadata"))?;
    let mut metadata = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, estimate.max(1), physical(), held)?;
    let owner_bytes = (std::mem::size_of::<HistoryWireOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
    let topology_charge = if wire.candidate.topology_owner_covers_allocation(&coordinator.external_memory_account())? {
        0
    } else { wire.candidate.topology_allocation_bytes()? };
    let topology_bytes = topology_charge.checked_add(owner_bytes)
        .ok_or(Error::Overflow("history wire topology owner"))?;
    let overhead = topology_bytes.checked_add(owner_bytes).ok_or(Error::Overflow("history wire owner"))?;
    coordinator.grow_external_memory_with_held(&mut wire.reservation, overhead, physical(), held)?;
    wire.reservation.set_allocated_bytes(wire.reservation.bytes())?;
    let topology_reservation = wire.reservation.split_off(topology_bytes)?;
    let candidate = wire.candidate;
    let display_lease: Arc<dyn std::any::Any + Send + Sync> = Arc::new(HistoryWireOwner { _reservation: topology_reservation });
    let buffer_lease: Arc<dyn Send + Sync> = Arc::new(HistoryWireOwner { _reservation: wire.reservation });
    let profile = wire.profile;
    let scene = built.scene.try_replace_brep_occurrence_display(&wire.stamp, path, &candidate,
        wire.buffers, profile, display_lease, buffer_lease)?;
    if scene.managed_memory_allowance()? > metadata.bytes() { return Err(Error::Budget("history wire scene metadata")); }
    scene.transfer_memory_allowance(&mut metadata, coordinator, held)?;
    let old_scene = std::mem::replace(&mut built.scene, scene);
    let old_reservation = std::mem::replace(&mut built.reservation, metadata);
    drop(old_scene);
    drop(old_reservation);
    let shared = built.scene.shared_buffer_bytes();
    if shared == 0 { built.buffer_owner = None; }
    else if let Some(owner) = built.buffer_owner.as_mut() { owner.bytes = shared; }
    else { return Err(Error::Invalid("history shared buffer owner")); }
    Ok(built)
}

// Build the same source-derived wires for each B-rep representation. The
// admission callback covers curves and sampled lines until the final buffers
// have been copied, including their overlapping allocations.
pub(super) fn prepare_source_buffers(source: &str, candidate: &om9_core::brep::Brep,
    limits: &om9_core::budget::Limits, mut admit: impl FnMut(u64, u64) -> Result<()>)
    -> Result<(om9_scene::scene::PreparedBrepBuffers, om9_core::budget::Usage)> {
    use om9_core::budget::Usage;
    // The face wires and the candidate's edge/triangle buffers must describe
    // the same source. Native topology attestation is a separate requirement.
    if source != candidate.source() { return Err(Error::Stale("source wire candidate")); }
    let refusal = std::cell::RefCell::new(None);
    let curves = om9_kernel::editing::default_face_isocurves_from_source_admitted(source, limits, |live, pending| {
        let result = admit(live, pending);
        if let Err(error) = &result { *refusal.borrow_mut() = Some(error.clone()); }
        result
    }).map_err(|error| refusal.into_inner().unwrap_or_else(|| Error::Io(format!("source face wires: {error}"))))?;
    let curve_bytes = curves.iter().try_fold(Usage::array::<om9_core::geometry::Curve>(curves.capacity())?.bytes,
        |sum, curve| sum.checked_add(curve.validate(limits)?.bytes).ok_or(Error::Overflow("source wire curves")))?;
    let (lines, line_bytes) = om9_scene::scene::PreparedBrepBuffers::sample_interior_curves(&curves, limits,
        |live, pending| admit(curve_bytes.checked_add(live).ok_or(Error::Overflow("source wire reservation"))?, pending))?;
    let wire_bytes = curve_bytes.checked_add(line_bytes).ok_or(Error::Overflow("source wire storage"))?;
    let (buffers, _) = om9_scene::scene::Scene::prepare_brep_buffers(candidate, limits,
        |live, pending| admit(wire_bytes.checked_add(live).ok_or(Error::Overflow("source wire reservation"))?, pending))?;
    buffers.with_interior_wires(&lines, limits,
        |live, pending| admit(wire_bytes.checked_add(live).ok_or(Error::Overflow("source wire reservation"))?, pending))
}

/// A private source-derived display. It does not attest the old v4 UUIDs and
/// cannot be published without a separate checked scene/lifetime transition.
pub struct DupEdgeSourceViewReceipt {
    candidate: CapturedTopologySnapshot,
    prepared: Option<WorkerBuffers>,
    source: om9_core::transaction::CapturedGeometrySource,
    stamp: om9_core::identity::SourceStamp,
    session_id: uuid::Uuid,
    selection_revision: u64,
    root_selection_revision: u64,
    document: Weak<om9_core::model::Document>,
    scene_publication: uuid::Uuid,
    cancel: Arc<AtomicBool>,
    _kernel: ExternalMemoryReservation,
}

pub struct DupEdgeSourceViewJob {
    root: om9_core::identity::ObjectId,
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Result<DupEdgeSourceViewReceipt>>>,
    delivered: bool,
    recovery_key: Option<(uuid::Uuid,u64,u64,uuid::Uuid)>,
}

struct DupEdgeSourceViewPermit(Arc<AtomicBool>);
impl Drop for DupEdgeSourceViewPermit {
    fn drop(&mut self) { self.0.store(false, Ordering::Release); }
}

impl DupEdgeSourceViewJob {
    pub fn cancel(&self) { self.cancel.store(true, Ordering::Release); }
    pub fn try_result(&mut self) -> Option<Result<DupEdgeSourceViewReceipt>> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(result) => { self.receiver = None; self.delivered = true; Some(result) }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                Some(Err(Error::Io("DupEdge source-view worker exited without a result".into())))
            }
        }
    }
}
impl Drop for DupEdgeSourceViewJob {
    fn drop(&mut self) { if !self.delivered { self.cancel(); } }
}

impl DupEdgeSourceViewReceipt {
    /// The result is usable only while the originating command, document,
    /// object and scene publication are still current.
    pub fn validate_current(&self, controller: &Controller) -> Result<()> {
        if self.cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        if !controller.dupedge_needs_pick() || controller.dupedge_session_id != self.session_id
            || controller.sub_revision() != self.selection_revision
            || controller.selection_state().revision != self.root_selection_revision {
            return Err(Error::Stale("DupEdge source-view command"));
        }
        if self.prepared.as_ref().is_some_and(|prepared|
            !prepared.reservation.belongs_to(&controller.coordinator.external_memory_account()))
            || !self._kernel.belongs_to(&controller.coordinator.external_memory_account()) {
            return Err(Error::Stale("DupEdge source-view reservation"));
        }
        let scene = controller.scene().ok_or(Error::Missing("DupEdge source-view scene"))?;
        if !self.document.ptr_eq(&Arc::downgrade(&scene.document)) || scene.publication_id() != self.scene_publication {
            return Err(Error::Stale("DupEdge source-view scene"));
        }
        let entity = scene.document.resolve(&self.stamp)?;
        let mut objects = scene.objects.iter().filter(|object| object.root == self.stamp.object && object.path.is_empty());
        let object = objects.next().ok_or(Error::Missing("DupEdge source-view root"))?;
        if objects.next().is_some() || object.source != self.stamp || !object.visible || !object.selectable
            || !object.has_pick_geometry() || scene.entity_for(controller.document(), object)?.id != self.stamp.object {
            return Err(Error::Stale("DupEdge source-view root"));
        }
        let matches_source = match (&self.source,&entity.geometry) {
            (om9_core::transaction::CapturedGeometrySource::Brep(source),om9_core::geometry::Geometry::Brep(current)) =>
                source.belongs_to(&controller.coordinator) && current.source()==source.source()
                    && self.candidate.brep().source()==source.source(),
            (om9_core::transaction::CapturedGeometrySource::NativeBrep(source),om9_core::geometry::Geometry::NativeBrep(current)) =>
                source.belongs_to(&controller.coordinator) && current.crc==source.crc() && current.payload.as_ref()==source.payload(),
            _ => false,
        };
        if !matches_source {
            return Err(Error::Stale("DupEdge source-view source"));
        }
        Ok(())
    }
    #[cfg(test)]
    fn candidate(&self) -> &om9_core::brep::Brep { self.candidate.brep() }

    /// Prepare but do not publish a temporary source-derived scene. Its owner
    /// keeps both worker reservations and any shared original buffers alive.
    pub fn prepare_scene(mut self, controller: &Controller, metadata_budget: u64) -> Result<PreparedDupEdgeSourceScene> {
        self.validate_current(controller)?;
        let held = controller.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut metadata = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache, metadata_budget.max(1), physical(), held)?;
        let current = controller.scene().ok_or(Error::Missing("DupEdge source-view scene"))?;
        let original_buffers = controller.share_scene_buffer_owner(current)?;
        let held = controller.scene_cache_usage()?;
        let estimate = current.render_snapshot_metadata_estimate()?.bytes.checked_add(current.picking_arena_bytes()?)
            .ok_or(Error::Overflow("DupEdge source-view scene metadata"))?;
        metadata.recheck_pending()?;
        if estimate > metadata.bytes() { return Err(Error::Budget("DupEdge source-view scene metadata")); }
        let mut prepared = self.prepared.take().ok_or(Error::Invalid("DupEdge source-view buffers"))?;
        let display_overhead = (std::mem::size_of::<DupEdgeSourceViewReceipt>() + 2 * std::mem::size_of::<usize>()) as u64;
        let buffer_overhead = (std::mem::size_of::<RemeshBufferOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
        controller.coordinator.grow_external_memory_with_held(&mut self._kernel, display_overhead, physical(), held)?;
        controller.coordinator.grow_external_memory_with_held(&mut prepared.reservation, buffer_overhead, physical(), held)?;
        let allocated = self._kernel.bytes().checked_sub(self._kernel.pending_bytes())
            .and_then(|bytes| bytes.checked_add(display_overhead))
            .ok_or(Error::Overflow("DupEdge source-view owner"))?;
        self._kernel.set_allocated_bytes(allocated)?;
        prepared.reservation.set_allocated_bytes(prepared.reservation.bytes())?;
        let buffer_owner = Arc::new(RemeshBufferOwner { _buffer_reservation: prepared.reservation });
        let owner = Arc::new(self);
        let display_lease: Arc<dyn std::any::Any + Send + Sync> = owner.clone();
        let buffer_lease: Arc<dyn Send + Sync> = buffer_owner;
        let profile=[owner.candidate.brep().display().deflection_mm.to_bits(),0.3f64.to_bits()];
        let scene = match &owner.source {
            om9_core::transaction::CapturedGeometrySource::Brep(_) => current.try_replace_root_brep_display(
                &owner.stamp,owner.candidate.brep(),prepared.buffers,profile,display_lease,buffer_lease)?,
            om9_core::transaction::CapturedGeometrySource::NativeBrep(source) => current.try_replace_root_native_brep_display(
                &owner.stamp,owner.candidate.brep(),prepared.buffers,profile,display_lease,buffer_lease,source)?,
        };
        let original_buffers = (scene.shared_buffer_bytes() != 0).then_some(original_buffers);
        if scene.managed_memory_allowance()? > metadata.bytes() { return Err(Error::Budget("DupEdge source-view scene metadata")); }
        scene.transfer_memory_allowance(&mut metadata, &controller.coordinator, held)?;
        owner.validate_current(controller)?;
        Ok(PreparedDupEdgeSourceScene { _scene: scene, owner, _original_buffers: original_buffers, _construction_reservation: metadata })
    }
}

/// A prepared source-derived scene, valid only for its originating DupEdge session.
pub struct PreparedDupEdgeSourceScene {
    _scene: om9_scene::scene::Scene,
    owner: Arc<DupEdgeSourceViewReceipt>,
    _original_buffers: Option<Arc<super::SceneBufferLease>>,
    _construction_reservation: ExternalMemoryReservation,
}
impl PreparedDupEdgeSourceScene {
    pub fn validate_current(&self, controller: &Controller) -> Result<()> { self.owner.validate_current(controller) }
    pub fn publish(self, controller: &mut Controller) -> Result<()> {
        self.validate_current(controller)?;
        if controller.dupedge_scene_backup.is_some() { return Err(Error::Busy); }
        let current = controller.scene().ok_or(Error::Missing("DupEdge source-view scene"))?;
        let mut old_usage = current.cache_held_usage()?;
        old_usage.bytes = old_usage.bytes.checked_sub(controller.registered_scene_buffer_bytes(current))
            .ok_or(Error::Invalid("DupEdge backup accounting"))?;
        let held = controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache, old_usage.bytes.max(1), physical, held)?;
        reservation.set_allocated_bytes(reservation.bytes())?;
        self.validate_current(controller)?;
        let PreparedDupEdgeSourceScene { _scene: scene, owner, _original_buffers, _construction_reservation } = self;
        let publication = scene.publication_id();
        let buffer_owner = _original_buffers.map(|lease| super::SceneBufferOwner {
            epoch: scene.epoch, document: Arc::downgrade(&scene.document),
            bytes: scene.shared_buffer_bytes(), lease,
        });
        if let Some(buffer_owner) = &buffer_owner { scene.bind_shared_buffer_owner(buffer_owner.lease.clone()); }
        drop(controller.topology_remesh_job.take());
        let old_scene = controller.scene.replace(scene).expect("validated source-view scene");
        let old_owner = std::mem::replace(&mut *controller.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()), buffer_owner);
        controller.dupedge_scene_backup = Some(super::DupEdgeSceneBackup {
            scene: old_scene, owner: old_owner, preview_publication: publication, reservation,
        });
        controller.clear_subselection();
        drop(owner);
        drop(_construction_reservation);
        Ok(())
    }
    #[cfg(test)]
    fn scene(&self) -> &om9_scene::scene::Scene { &self._scene }
}

impl Controller {
    pub fn dupedge_source_view_pending(&self) -> bool { self.dupedge_source_view_job.is_some() }
    pub(crate) fn dupedge_source_view_pending_root(&self) -> Option<om9_core::identity::ObjectId> {self.dupedge_source_view_job.as_ref().map(|job|job.root)}
    pub(crate) fn cancel_dupedge_source_view(&mut self) {drop(self.dupedge_source_view_job.take());}
    #[cfg(test)]
    pub(crate) fn dupedge_source_view_worker_active(&self) -> bool {
        self.dupedge_source_view_active.load(Ordering::Acquire)
    }
    #[cfg(test)]
    pub(crate) fn inject_dupedge_source_view_error_for_test(&mut self,error:Error)->Result<()> {
        let old=self.dupedge_source_view_job.take().ok_or(Error::Missing("DupEdge source-view job"))?;
        let key=old.recovery_key;let root=old.root;drop(old);
        let (sender,receiver)=mpsc::sync_channel(1);
        sender.send(Err(error)).map_err(|_|Error::Missing("DupEdge test receiver"))?;
        self.dupedge_source_view_job=Some(DupEdgeSourceViewJob {root,cancel:Arc::new(AtomicBool::new(false)),
            receiver:Some(receiver),delivered:false,recovery_key:key});Ok(())
    }

    pub(crate) fn request_dupedge_source_view(&mut self, object: om9_core::identity::ObjectId) -> Result<()> {
        if self.dupedge_source_view_job.is_some() { return Err(Error::Busy); }
        let max = self.coordinator.limits().max_bytes;
        // Fixed reservations bound managed worker memory; large sources can be refused.
        let kernel = (max / 16).min(16 * 1024 * 1024).max(1);
        let buffers = (max / 32).min(8 * 1024 * 1024).max(1);
        self.dupedge_source_view_job = Some(self.start_dupedge_source_view(object, kernel, buffers)?);
        Ok(())
    }

    pub fn poll_dupedge_source_view(&mut self) -> Option<Result<()>> {
        let mut job = self.dupedge_source_view_job.take()?;
        let Some(result) = job.try_result() else {
            self.dupedge_source_view_job = Some(job);
            return None;
        };
        let metadata = (self.coordinator.limits().max_bytes / 32).min(8 * 1024 * 1024).max(1);
        let result=result.and_then(|receipt| receipt.prepare_scene(self, metadata)?.publish(self));
        if matches!(&result,Err(error) if super::wireframe::source_view_memory_refusal(error))
            && (job.recovery_key.is_none() || self.dupedge_source_view_context_key()!=job.recovery_key) {
            return Some(Err(Error::Stale("DupEdge source-view recovery context")));
        }
        Some(result)
    }

    /// Restore only the exact temporary publication; a newer scene wins.
    pub(super) fn restore_dupedge_scene(&mut self) {
        let Some(backup) = self.dupedge_scene_backup.take() else { return; };
        let valid = self.scene.as_ref().is_some_and(|current|
            current.publication_id() == backup.preview_publication
                && current.epoch == self.coordinator.epoch()
                && Arc::ptr_eq(&current.document, &backup.scene.document)
                && std::ptr::eq(current.document.as_ref(), self.document()));
        if valid {
            let old_scene = self.scene.replace(backup.scene);
            let old_owner = std::mem::replace(&mut *self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()), backup.owner);
            drop(old_scene);
            drop(old_owner);
        }
    }

    pub(super) fn commit_dupedge_source_geometries(&mut self,
        items: Vec<(&'static str, om9_core::geometry::Geometry, om9_core::geometry::Transform)>) -> Result<()> {
        let Some(backup) = self.dupedge_scene_backup.take() else {
            return self.commit_placed_geometries("OM9-CURVE-040", "DupEdge", items.into_iter());
        };
        let current = self.scene.as_ref().ok_or(Error::Missing("DupEdge source-view scene"))?;
        if current.publication_id() != backup.preview_publication || current.epoch != self.coordinator.epoch()
            || !Arc::ptr_eq(&current.document, &backup.scene.document)
            || !std::ptr::eq(current.document.as_ref(), self.document()) {
            return Err(Error::Stale("DupEdge source-view scene"));
        }
        let original_publication = backup.scene.publication_id();
        let preview = self.scene.replace(backup.scene).expect("validated source-view scene");
        let preview_owner = std::mem::replace(&mut *self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()), backup.owner);
        let result = self.commit_placed_geometries("OM9-CURVE-040", "DupEdge", items.into_iter());
        let original_still_current = self.scene.as_ref().is_some_and(|scene|
            scene.publication_id() == original_publication && scene.epoch == self.coordinator.epoch()
                && std::ptr::eq(scene.document.as_ref(), self.document()));
        if original_still_current {
            let scene = self.scene.replace(preview).expect("original scene still current");
            let owner = std::mem::replace(&mut *self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()), preview_owner);
            self.dupedge_scene_backup = Some(super::DupEdgeSceneBackup {
                scene, owner, preview_publication: backup.preview_publication, reservation: backup.reservation,
            });
            return match result { Ok(()) => Err(Error::Invalid("DupEdge commit was not published")), Err(error) => Err(error) };
        }
        result
    }
}

#[cfg(test)]
mod dupedge_source_view_tests {
    use super::*;
    use om9_core::{budget::Limits, geometry::Geometry, model::{Document, Entity}};

    fn fixture() -> (Controller, om9_core::identity::ObjectId) {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
        let mut document = Document::new("source-view worker");
        let entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("10,10,10").unwrap(); controller.undo().unwrap();
        controller.submit("DupEdge").unwrap();
        (controller, id)
    }

    #[test]
    fn native_source_view_worker_is_private_and_rejects_stale_session() {
        use om9_core::{budget::Limits,geometry::Geometry,model::{Document,Entity}};
        let limits=Limits::default();
        let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        let mut doc=Document::with_template_tolerance("native worker");
        let entity=Entity::new("box",Geometry::NativeBrep(native),None);let id=entity.id;doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("10,10,10").unwrap();c.undo().unwrap();
        c.submit("DupEdge").unwrap();let before=c.coordinator().snapshot();
        let receipt=wait(c.start_dupedge_source_view(id,16*1024*1024,8*1024*1024).unwrap()).unwrap();
        assert!(receipt.validate_current(&c).is_ok());assert_eq!(receipt.candidate().topology().edges.len(),12);
        assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
        let scene=receipt.prepare_scene(&c,8*1024*1024).unwrap();
        scene.publish(&mut c).unwrap();
        let stamp=c.scene().unwrap().document.stamp(id).unwrap();
        let display=c.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        assert_eq!(display.topology.edges.len(),12);
        let object=c.scene().unwrap().objects.iter().find(|object|object.root==id).unwrap();
        let om9_scene::scene::RenderBuffers::Mesh {vertices,edges,..}=object.buffers.as_ref() else {panic!("native BRep buffers")};
        assert!(vertices.len()>display.points.len() && edges.len()>display.segments.len());
        let camera=om9_scene::camera::Camera::orthographic([2.,-10.,50.],[2.,0.,5.],[0.,1.,0.],10.).unwrap();
        let viewport=om9_scene::camera::Viewport {origin:[0.,0.],size:[400.,400.]};
        let top=om9_scene::camera::Camera::orthographic([2.,2.5,20.],[2.,2.5,0.],[0.,1.,0.],10.).unwrap();
        let center=top.project([2.,2.5,6.],viewport).unwrap().unwrap().screen;
        assert!(c.scene().unwrap().pick_kind(center,&top,viewport,1.,om9_scene::scene::PickIntent::Select,
            om9_scene::scene::PickKind::Edge).unwrap().is_none());
        let screen=camera.project([2.,0.,6.],viewport).unwrap().unwrap().screen;
        assert!(!c.scene().unwrap().pick_edge_candidates(screen,&camera,viewport,6.,om9_scene::scene::PickIntent::Select).unwrap().is_empty());
        c.cancel();assert!(c.scene().unwrap().root_brep_pick_display(&stamp).is_err());
        assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
        c.submit("DupEdge").unwrap();
        let stale=wait(c.start_dupedge_source_view(id,16*1024*1024,8*1024*1024).unwrap()).unwrap();
        c.cancel();assert!(stale.validate_current(&c).is_err());assert_eq!(c.coordinator().snapshot(),before);
        c.submit("DupEdge").unwrap();let job=c.start_dupedge_source_view(id,16*1024*1024,8*1024*1024).unwrap();
        job.cancel();match wait(job) {Err(Error::Canceled)=>{},Ok(receipt)=>assert!(matches!(receipt.validate_current(&c),Err(Error::Canceled))),other=>panic!("unexpected cancel: {}",if other.is_err(){"error"}else{"success"})};
        c.cancel();assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
    }

    #[test]
    fn native_source_view_worker_refuses_bad_payload_and_small_budget_without_changing_history() {
        use om9_core::{budget::Limits,geometry::Geometry,model::{Document,Entity}};
        let limits=Limits::default();
        let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        for bad in [false,true] {
            let mut source=native.clone();if bad {source.payload=vec![1,2,3,4].into();}
            let mut doc=Document::with_template_tolerance("native worker failure");
            let entity=Entity::new("box",Geometry::NativeBrep(source),None);let id=entity.id;doc.entities.insert(id,entity);
            let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("10,10,10").unwrap();c.undo().unwrap();
            c.submit("DupEdge").unwrap();let before=c.coordinator().snapshot();let publication=c.scene().unwrap().publication_id();
            let job=c.start_dupedge_source_view(id,if bad {16*1024*1024} else {1},8*1024*1024).unwrap();
            assert!(wait(job).is_err());assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
            assert_eq!(c.scene().unwrap().publication_id(),publication);
        }
    }

    #[test]
    fn native_source_view_at_extreme_zoom_refuses_offscreen_source_without_loop() {
        use om9_core::{budget::Limits,geometry::Geometry,model::{Document,Entity}};
        use om9_scene::{camera::{Camera,Viewport},scene::Subelement};
        let limits=Limits::default();let shape=om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        let mut doc=Document::with_template_tolerance("native zoom");
        let entity=Entity::new("cylinder",Geometry::NativeBrep(native),None);let id=entity.id;doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("10,10,10").unwrap();c.undo().unwrap();c.submit("DupEdge").unwrap();
        let receipt=wait(c.start_dupedge_source_view(id,16*1024*1024,8*1024*1024).unwrap()).unwrap();receipt.prepare_scene(&c,8*1024*1024).unwrap().publish(&mut c).unwrap();
        let stamp=c.document().stamp(id).unwrap();let display=c.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        let s=display.segments.iter().find(|s| {let a=display.points[s[0] as usize];let b=display.points[s[1] as usize];
            (a[2]-5.).abs()<1e-8&&(b[2]-5.).abs()<1e-8&&(a[0]-b[0]).hypot(a[1]-b[1])>0.01}).unwrap();
        let a=display.points[s[0] as usize];let b=display.points[s[1] as usize];let edge=display.topology.edges[s[2] as usize];
        let point=std::array::from_fn(|i|(a[i]+b[i])*0.5);
        let camera=Camera::orthographic([point[0],point[1],50.],[point[0],point[1],5.],[0.,1.,0.],0.001).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[800.,800.]};let projected=camera.project(point,viewport).unwrap().unwrap();
        let selected=super::super::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(edge),source:stamp,world_epoch:c.coordinator().epoch()};
        c.dupedge_pick_with_point(selected,point,0.01,&camera,viewport,projected.screen,6.,projected.depth).unwrap();
        let before=c.coordinator().snapshot();let (result,refresh)=c.probe_dupedge_source_picks_for_test();
        assert!(result.is_err()&&!refresh);assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
        assert!(c.submit("").is_err());assert!(!c.dupedge_source_view_pending());assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
        c.cancel();assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
    }

    #[test]
    fn native_source_view_memory_refusal_reports_budget_until_context_changes() {
        use om9_core::{budget::Limits,geometry::Geometry,model::{Document,Entity}};
        let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        for change in [false,true] {
            let mut doc=Document::with_template_tolerance("native source-view admission");
            let entity=Entity::new("box",Geometry::NativeBrep(native.clone()),None);let id=entity.id;doc.entities.insert(id,entity);
            let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("10,10,10").unwrap();c.undo().unwrap();c.submit("DupEdge").unwrap();
            let before=c.coordinator().snapshot();c.request_dupedge_source_view(id).unwrap();
            c.inject_dupedge_source_view_error_for_test(Error::Budget("bytes")).unwrap();
            if change {c.select_objects(&[id]).unwrap();}
            let result=c.poll_dupedge_source_view().unwrap();
            if change {assert!(matches!(result,Err(Error::Stale(_))),"{result:?}");}
            else {assert!(matches!(result,Err(Error::Budget("bytes"))),"{result:?}");}
            assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
        }
    }
    fn wait(mut job: DupEdgeSourceViewJob) -> Result<DupEdgeSourceViewReceipt> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if let Some(result) = job.try_result() { return result; }
            assert!(std::time::Instant::now() < deadline, "source-view worker timed out");
            std::thread::yield_now();
        }
    }
    #[test]
    fn source_view_publishes_face_wires_without_picking_them_as_edges() {
        use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind, RenderBuffers}};
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let scene = controller.scene().unwrap();
        let stamp = scene.document.stamp(id).unwrap();
        let display = scene.root_brep_pick_display(&stamp).unwrap();
        let object = scene.objects.iter().find(|object| object.root == id).unwrap();
        let RenderBuffers::Mesh { vertices, edges, .. } = object.buffers.as_ref() else { panic!("BRep buffers") };
        assert!(vertices.len() > display.points.len());
        assert!(edges.len() > display.segments.len());
        for point in &vertices[display.points.len()..] {
            assert!(point[0] >= -1e-7 && point[0] <= 4. + 1e-7
                && point[1] >= -1e-7 && point[1] <= 5. + 1e-7
                && point[2] >= -1e-7 && point[2] <= 6. + 1e-7);
            assert!(point[0].abs() < 1e-7 || (point[0]-4.).abs() < 1e-7
                || point[1].abs() < 1e-7 || (point[1]-5.).abs() < 1e-7
                || point[2].abs() < 1e-7 || (point[2]-6.).abs() < 1e-7);
        }
        let camera = Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let center = camera.project([2., 2.5, 6.], viewport).unwrap().unwrap().screen;
        assert!(scene.pick_kind(center, &camera, viewport, 1., PickIntent::Select, PickKind::Edge).unwrap().is_none());
        controller.cancel();
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }
    #[test]
    fn source_view_worker_retains_private_output_and_refuses_stale_scene() {
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let old_publication = controller.scene().unwrap().publication_id();
        let old = match &controller.document().entities[&id].geometry { Geometry::Brep(brep) => brep.clone(), _ => unreachable!() };
        let kernel_before = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let buffer_before = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.validate_current(&controller).unwrap();
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > kernel_before);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) > buffer_before);
        assert_eq!(receipt.candidate().source(), old.source());
        assert_ne!(receipt.candidate().topology().edges, old.topology().edges);
        assert_eq!(controller.scene().unwrap().publication_id(), old_publication);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert!(receipt.validate_current(&controller).is_err());
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        drop(receipt);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), kernel_before);
    }
    #[test]
    fn source_view_worker_rejects_low_budget_and_cancelled_command() {
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        controller.dupedge_source_view_active.store(true, Ordering::Release);
        assert!(matches!(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024), Err(Error::Busy)));
        controller.dupedge_source_view_active.store(false, Ordering::Release);
        let result = wait(controller.start_dupedge_source_view(id, 1, 1).unwrap());
        assert!(result.is_err());
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        let publication = controller.scene().unwrap().publication_id();
        let kernel_before = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let buffer_before = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let result = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 1).unwrap());
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), kernel_before);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), buffer_before);
        assert_eq!(controller.coordinator().snapshot(), before);
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        controller.cancel();
        assert!(receipt.validate_current(&controller).is_err());
        controller.submit("DupEdge").unwrap();
        assert!(receipt.validate_current(&controller).is_err(), "a later DupEdge session cannot reuse an earlier worker result");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }
    #[test]
    fn source_view_worker_cancel_never_authorizes_output_and_releases_budget() {
        let (controller, id) = fixture();
        let kernel_before = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let buffer_before = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let job = controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        job.cancel();
        if let Ok(receipt) = wait(job) {
            assert!(receipt.validate_current(&controller).is_err());
            drop(receipt);
        }
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), kernel_before);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), buffer_before);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }
    #[test]
    fn source_view_worker_rejects_changed_subselection() {
        let (mut controller, id) = fixture();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        let edge = match &controller.document().entities[&id].geometry {
            Geometry::Brep(brep) => brep.topology().edges[0], _ => unreachable!(),
        };
        controller.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
        assert!(receipt.validate_current(&controller).is_err(), "changed edge selection invalidates worker result");
        assert_eq!(controller.coordinator().redo_len(), 1);
    }

    #[test]
    fn source_view_worker_rejects_changed_root_selection() {
        let (mut controller,id)=fixture();let before=controller.coordinator().snapshot();
        let receipt=wait(controller.start_dupedge_source_view(id,16*1024*1024,8*1024*1024).unwrap()).unwrap();
        controller.select_objects(&[id]).unwrap();
        assert!(matches!(receipt.validate_current(&controller),Err(Error::Stale(_))));
        assert_eq!(controller.coordinator().snapshot(),before);assert_eq!(controller.coordinator().redo_len(),1);
    }

    #[test]
    fn source_view_memory_error_refuses_changed_recovery_context() {
        use om9_scene::scene::Subelement;
        for changed in 0..3 {
            let (mut controller,id)=fixture();
            let edges=match &controller.document().entities[&id].geometry {Geometry::Brep(brep)=>brep.topology().edges.clone(),_=>unreachable!()};
            controller.select_subobject(id,Subelement::BrepEdge(edges[0])).unwrap();
            let key=controller.dupedge_recovery_key().unwrap();let (sender,receiver)=mpsc::sync_channel(1);
            sender.send(Err(Error::Budget("bytes"))).unwrap();
            controller.dupedge_source_view_job=Some(DupEdgeSourceViewJob {root:id,cancel:Arc::new(AtomicBool::new(false)),
                receiver:Some(receiver),delivered:false,recovery_key:Some(key)});
            if changed==1 {controller.select_subobject(id,Subelement::BrepEdge(edges[1])).unwrap();}
            if changed==2 {controller.select_objects(&[id]).unwrap();}
            let before=controller.coordinator().snapshot();let error=controller.poll_dupedge_source_view().unwrap().unwrap_err();
            if changed!=0 {assert!(matches!(error,Error::Stale(_)));}else{assert!(matches!(error,Error::Budget("bytes")));}
            assert_eq!(controller.coordinator().snapshot(),before);assert_eq!(controller.coordinator().redo_len(),1);
        }
    }
    #[test]
    fn scheduled_source_view_refuses_selection_changed_during_worker() {
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let scene = controller.scene().unwrap().publication_id();
        controller.request_dupedge_source_view(id).unwrap();
        let edge = match &controller.document().entities[&id].geometry {
            Geometry::Brep(brep) => brep.topology().edges[0], _ => unreachable!(),
        };
        controller.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        let error = loop {
            if let Some(result) = controller.poll_dupedge_source_view() { break result.unwrap_err(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        };
        assert!(matches!(error, Error::Stale(_)));
        assert_eq!(controller.scene().unwrap().publication_id(), scene);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.subselection().len(), 1);
    }
    #[test]
    fn cancelled_source_view_cannot_replace_new_document() {
        use crate::file_lifecycle::{FileKind, FileOutcome, UnsavedDecision};
        let (mut controller, id) = fixture();
        controller.request_dupedge_source_view(id).unwrap();
        let worker = Arc::clone(&controller.dupedge_source_view_active);
        controller.cancel();
        let outcome = controller.begin_file_operation(FileKind::New).unwrap();
        if outcome == FileOutcome::Pending {
            let token = controller.file_operation().unwrap().token();
            assert_eq!(controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap(), FileOutcome::Replaced);
        } else { assert_eq!(outcome, FileOutcome::Replaced); }
        let document = controller.document().id;
        let publication = controller.scene().unwrap().publication_id();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while worker.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline, "cancelled worker did not finish");
            std::thread::yield_now();
        }
        assert_eq!(controller.document().id, document);
        assert!(controller.document().entities.is_empty());
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert!(!controller.dupedge_source_view_pending());
    }
    #[test]
    fn source_view_prepares_private_scene_and_refuses_stale_publication() {
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let old_publication = controller.scene().unwrap().publication_id();
        let kernel_before = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        let original_ids = match &controller.document().entities[&id].geometry {
            Geometry::Brep(brep) => brep.topology().edges.clone(), _ => unreachable!(),
        };
        let prepared = receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        prepared.validate_current(&controller).unwrap();
        assert_ne!(prepared.scene().publication_id(), old_publication);
        let stamp = controller.document().stamp(id).unwrap();
        let view = prepared.scene().root_brep_pick_display(&stamp).unwrap();
        assert_ne!(view.topology.edges, original_ids);
        assert_eq!(controller.scene().unwrap().publication_id(), old_publication);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > kernel_before);
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert!(prepared.validate_current(&controller).is_err());
        drop(prepared);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), kernel_before);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }
    #[test]
    fn source_view_private_scene_low_budget_preserves_current_scene() {
        let (controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let old_publication = controller.scene().unwrap().publication_id();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        assert!(receipt.prepare_scene(&controller, 1).is_err());
        assert_eq!(controller.scene().unwrap().publication_id(), old_publication);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }
    #[test]
    fn source_view_scene_snapshot_keeps_worker_reservations_alive() {
        let (controller, id) = fixture();
        let kernel_before = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        let prepared = receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache, 8 * 1024 * 1024, physical, held).unwrap();
        let snapshot = prepared.scene().try_render_snapshot().unwrap();
        snapshot.transfer_memory_allowance(&mut reservation, &controller.coordinator, held).unwrap();
        drop(reservation);
        let owner = Arc::downgrade(&prepared.owner);
        drop(prepared);
        assert!(owner.upgrade().is_some(), "scene snapshot must retain the display owner");
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > kernel_before);
        drop(snapshot);
        assert!(owner.upgrade().is_none(), "last scene snapshot must release the display owner");
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), kernel_before);
    }
    #[test]
    fn source_view_publication_cancel_restores_original_without_history_change() {
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let original = controller.scene().unwrap().publication_id();
        let kernel_before = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_ne!(controller.scene().unwrap().publication_id(), original);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        controller.cancel();
        assert_eq!(controller.scene().unwrap().publication_id(), original);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), kernel_before);
    }
    #[test]
    fn source_view_commit_refusal_retries_and_success_uses_original_scene() {
        let (mut controller, id) = fixture();
        let before = controller.coordinator().snapshot();
        let entity_count = controller.document().entities.len();
        let original = controller.scene().unwrap().publication_id();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let preview = controller.scene().unwrap().publication_id();
        assert!(controller.commit_dupedge_source_geometries(Vec::new()).is_err());
        assert_eq!(controller.scene().unwrap().publication_id(), preview);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(controller.commit_dupedge_source_geometries(vec![("DupEdge", Geometry::Point([1., 2., 3.]), om9_core::geometry::Transform::identity())]).is_err());
        assert_eq!(controller.scene().unwrap().publication_id(), preview);
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        drop(pressure);
        controller.commit_dupedge_source_geometries(vec![("DupEdge", Geometry::Point([1., 2., 3.]), om9_core::geometry::Transform::identity())]).unwrap();
        assert!(controller.dupedge_scene_backup.is_none());
        assert_ne!(controller.scene().unwrap().publication_id(), preview);
        assert_ne!(controller.scene().unwrap().publication_id(), original);
        let stamp = controller.document().stamp(id).unwrap();
        let ids = match &controller.document().entities[&id].geometry { Geometry::Brep(brep) => &brep.topology().edges, _ => unreachable!() };
        assert_eq!(&controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.edges, ids);
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.len(), entity_count);
        controller.redo().unwrap();
    }
    #[test]
    fn source_view_cancel_does_not_replace_newer_scene() {
        let (mut controller, id) = fixture();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        let newer = controller.scene().unwrap().publication_id();
        controller.cancel();
        assert_eq!(controller.scene().unwrap().publication_id(), newer);
    }
    #[test]
    fn property_edit_during_source_view_does_not_leak_temporary_edges() {
        let (mut controller, id) = fixture();
        let old_ids = match &controller.document().entities[&id].geometry {
            Geometry::Brep(brep) => brep.topology().edges.clone(), _ => unreachable!(),
        };
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        assert_ne!(controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.edges, old_ids);
        let before = controller.coordinator().snapshot();
        let preview = controller.scene().unwrap().publication_id();
        let error = controller.edit_object_property(vec![stamp], super::super::properties::PropertyEdit::Label("renamed".into())).unwrap_err();
        assert!(error.to_string().contains("finish or cancel DupEdge"));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.scene().unwrap().publication_id(), preview);
        assert!(controller.dupedge_needs_pick());
        controller.cancel();
        let stamp = controller.document().stamp(id).unwrap();
        assert_eq!(controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.edges, old_ids,
            "cancel must restore the original edge identities");
        controller.edit_object_property(vec![stamp], super::super::properties::PropertyEdit::Label("renamed".into())).unwrap();
        controller.undo().unwrap();
        assert_eq!(controller.document().entities[&id].label, "box");
        controller.redo().unwrap();
        assert_eq!(controller.document().entities[&id].label, "renamed");
    }
    #[test]
    fn source_view_publication_cancels_pending_topology_remesh() {
        let (mut controller, id) = fixture();
        controller.cancel();
        controller.insert_attested_sphere_for_test().unwrap();
        let sphere = *controller.document().entities.keys().find(|&&other| other != id).unwrap();
        controller.submit("DupEdge").unwrap();
        controller.request_topology_remesh_for(sphere, 0.05, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert!(!controller.topology_remesh_pending(), "older remesh must not publish a scene containing a temporary edge view");
        controller.cancel();
    }
    #[test]
    fn source_view_published_edges_complete_dupedge_and_undo() {
        use om9_scene::{camera::{Camera, Viewport}, scene::Subelement};
        let (mut controller, id) = fixture();
        let receipt = wait(controller.start_dupedge_source_view(id, 16 * 1024 * 1024, 8 * 1024 * 1024).unwrap()).unwrap();
        let shape = receipt.candidate().clone();
        receipt.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let camera = Camera::orthographic([10., -10., 14.], [2., 2., 3.], [0., 0., 1.], 30.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let mut expected = Vec::new();
        for s in shape.display().segments.iter().filter(|s| {
            let a = shape.display().points[s[0] as usize]; let b = shape.display().points[s[1] as usize];
            (a[2] - 6.).abs() < 1e-8 && (b[2] - 6.).abs() < 1e-8
        }).take(2) {
            let a = shape.display().points[s[0] as usize]; let b = shape.display().points[s[1] as usize];
            let point = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
            let projected = camera.project(point, viewport).unwrap().unwrap();
            let reference = super::super::SelectedSubobject { root: id, path: vec![],
                subset: Subelement::BrepEdge(shape.topology().edges[s[2] as usize]),
                source: controller.document().stamp(id).unwrap(), world_epoch: controller.coordinator().epoch() };
            controller.dupedge_pick_with_point(reference, point, 0.01, &camera, viewport,
                projected.screen, 6., projected.depth).unwrap();
            expected.push([a, b]);
        }
        assert_eq!(expected.len(), 2);
        let before = controller.document().entities.len();
        let snapshot = controller.coordinator().snapshot();
        let preview = controller.scene().unwrap().publication_id();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(controller.submit("").is_err());
        assert_eq!(controller.coordinator().snapshot(), snapshot);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.scene().unwrap().publication_id(), preview);
        assert_eq!(controller.dupedge_temporary_picks().count(), 2);
        assert!(controller.dupedge_needs_pick());
        drop(pressure);
        controller.submit("").unwrap();
        assert!(controller.dupedge_scene_backup.is_none());
        let curves: Vec<_> = controller.document().entities.values().filter(|entity| entity.id != id).collect();
        assert_eq!(curves.len(), 2);
        for pair in expected {
            assert!(curves.iter().any(|entity| {
                let Geometry::Curve(curve) = &entity.geometry else { return false };
                let endpoints: Vec<_> = [0., 1.].into_iter().map(|t| om9_core::evaluation::evaluate_curve(curve,
                    om9_core::evaluation::Parameter::Fraction(t), 0, om9_core::geometry::Transform::identity(),
                    controller.coordinator().limits()).unwrap()[0]).collect();
                pair.iter().all(|point| endpoints.iter().any(|end| (0..3).all(|i| (point[i] - end[i]).abs() < 1e-8)))
            }));
        }
        controller.undo().unwrap(); assert_eq!(controller.document().entities.len(), before);
        controller.redo().unwrap(); assert_eq!(controller.document().entities.len(), before + 2);
    }

    #[test]
    fn source_view_affine_edge_commits_with_original_placement() {
        use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
        use om9_scene::{camera::{Camera, Viewport}, scene::Subelement};
        let limits=Limits::default();
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box{x_mm:4.,y_mm:5.,z_mm:6.},0.01,&limits).unwrap();
        let placement=Transform([[2.,0.5,0.,3.],[0.,1.,0.,-2.],[0.,0.,1.,1.],[0.,0.,0.,1.]]);
        let mut entity=Entity::new("box",Geometry::Brep(shape),None); entity.placement=placement; let id=entity.id;
        let mut doc=Document::new("affine temporary view"); doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap(); c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.submit("DupEdge").unwrap();
        let receipt=wait(c.start_dupedge_source_view(id,16*1024*1024,8*1024*1024).unwrap()).unwrap();
        let shape=receipt.candidate().clone(); receipt.prepare_scene(&c,8*1024*1024).unwrap().publish(&mut c).unwrap();
        let segment=shape.display().segments.iter().find(|s| {
            let a=shape.display().points[s[0] as usize]; let b=shape.display().points[s[1] as usize];
            (a[2]-6.).abs()<1e-8 && (b[2]-6.).abs()<1e-8
        }).unwrap();
        let a=shape.display().points[segment[0] as usize]; let b=shape.display().points[segment[1] as usize];
        let world=placement.point(std::array::from_fn(|i|(a[i]+b[i])*0.5)).unwrap();
        let camera=Camera::orthographic([15.,-15.,20.],[4.,2.,3.],[0.,0.,1.],40.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]}; let projected=camera.project(world,viewport).unwrap().unwrap();
        let reference=super::super::SelectedSubobject {root:id,path:vec![],subset:Subelement::BrepEdge(shape.topology().edges[segment[2] as usize]),
            source:c.document().stamp(id).unwrap(),world_epoch:c.coordinator().epoch()};
        c.dupedge_pick_with_point(reference,world,0.01,&camera,viewport,projected.screen,6.,projected.depth).unwrap();
        let before=c.document().entities.clone(); c.submit("").unwrap(); assert!(c.dupedge_scene_backup.is_none());
        let output=c.document().entities.values().find(|e|e.label=="DupEdge").unwrap(); assert_eq!(output.placement,placement);
        let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
        let ends: Vec<_>=[0.,1.].into_iter().map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]).collect();
        for p in [placement.point(a).unwrap(),placement.point(b).unwrap()] {
            assert!(ends.iter().any(|q|(0..3).all(|i|(p[i]-q[i]).abs()<1e-8)));
        }
        let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

impl Controller {
    /// Begin a private source-derived view for one root while DupEdge is active.
    /// The caller owns polling and must separately implement admitted publication.
    pub fn start_dupedge_source_view(&self, object: om9_core::identity::ObjectId,
        kernel_budget: u64, buffer_budget: u64) -> Result<DupEdgeSourceViewJob> {
        if !self.dupedge_needs_pick() { return Err(Error::Missing("DupEdge selection session")); }
        if kernel_budget == 0 || buffer_budget == 0 { return Err(Error::Invalid("DupEdge source-view budget")); }
        self.dupedge_source_view_active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::Busy)?;
        let permit = DupEdgeSourceViewPermit(Arc::clone(&self.dupedge_source_view_active));
        let scene = self.scene().ok_or(Error::Missing("DupEdge source-view scene"))?;
        let stamp = scene.document.stamp(object)?;
        let entity = scene.document.resolve(&stamp)?;
        let mut roots = scene.objects.iter().filter(|entry| entry.root == object && entry.path.is_empty());
        let root = roots.next().ok_or(Error::Missing("DupEdge source-view root"))?;
        if roots.next().is_some() || root.source != stamp || !root.visible || !root.selectable
            || !root.has_pick_geometry() || scene.entity_for(self.document(), root)?.id != object {
            return Err(Error::Stale("DupEdge source-view root"));
        }
        // A replacement view must improve a coarse display as well as refresh identities.
        let (deflection,source) = match &entity.geometry {
            om9_core::geometry::Geometry::Brep(_) => (scene.root_brep_pick_display(&stamp)?.deflection_mm.min(0.01),
                om9_core::transaction::CapturedGeometrySource::Brep(self.coordinator.capture_brep_source(&stamp)?)),
            om9_core::geometry::Geometry::NativeBrep(_) => (0.01,
                om9_core::transaction::CapturedGeometrySource::NativeBrep(self.coordinator.capture_native_brep_source(&stamp)?)),
            _ => return Err(Error::Invalid("DupEdge source-view B-Rep")),
        };
        validate_remesh_profile(deflection, 0.3)?;
        let document = Arc::downgrade(&scene.document);
        let current = self.coordinator.world().documents.get(&self.document).ok_or(Error::Missing("DupEdge document"))?;
        if !document.ptr_eq(&Arc::downgrade(current)) { return Err(Error::Stale("DupEdge source-view document")); }
        let scene_publication = scene.publication_id();
        let session_id = self.dupedge_session_id;
        let selection_revision = self.sub_revision();
        let root_selection_revision = self.selection_state().revision;
        let tolerance = scene.document.tolerance().map_or(1e-6,|t|t.absolute_mm.min(1e-6));
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        if physical.is_none() { return Err(Error::Unsupported("physical RAM availability is unavailable")); }
        let mut kernel = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, kernel_budget, physical, held)?;
        let mut buffer = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, buffer_budget, physical, held)?;
        let limits = *self.coordinator.limits();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-dupedge-source-view".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<DupEdgeSourceViewReceipt> {
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let refusal = std::cell::RefCell::new(None);
                let mut admit = |live: u64, pending: u64| -> Result<()> {
                    let result = (|| {
                        if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                        if live.checked_add(pending).ok_or(Error::Overflow("DupEdge source-view budget"))? > kernel.bytes() {
                            return Err(Error::Budget("DupEdge source-view budget"));
                        }
                        kernel.set_allocated_bytes(live)?;
                        kernel.recheck_pending()
                    })();
                    if let Err(error) = &result { *refusal.borrow_mut() = Some(error.clone()); }
                    result
                };
                let native_source = match &source {
                    om9_core::transaction::CapturedGeometrySource::NativeBrep(native) =>
                        Some(om9_kernel::brep::from_3dm_source_typed_admitted(native.payload(),tolerance,&limits,&mut admit)?),
                    _ => None,
                };
                let source_text = match &source {
                    om9_core::transaction::CapturedGeometrySource::Brep(brep) => brep.source(),
                    om9_core::transaction::CapturedGeometrySource::NativeBrep(_) => native_source.as_deref().expect("native source"),
                };
                let source_bytes = native_source.as_ref().map_or(0,|s|s.capacity() as u64);
                let candidate = capture_source_topology(source_text, deflection, 0.3, &limits, |live,pending| {
                    admit(source_bytes.checked_add(live).ok_or(Error::Overflow("DupEdge native source"))?,pending)
                })
                    .map_err(|error| refusal.borrow_mut().take().unwrap_or_else(|| Error::Io(format!("DupEdge source-view: {error}"))))?;
                drop(admit);
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let (buffers, actual) = prepare_source_buffers(source_text, candidate.brep(), &limits, |live, pending| {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    if live.checked_add(pending).ok_or(Error::Overflow("DupEdge source-view buffers"))? > buffer.bytes() {
                        return Err(Error::Budget("DupEdge source-view buffers"));
                    }
                    buffer.set_allocated_bytes(live)?;
                    buffer.recheck_pending()
                })?;
                drop(native_source);
                buffer.set_allocated_bytes(actual.bytes)?;
                buffer.shrink_to(actual.bytes.max(1))?;
                let retained = candidate.manifest_bytes()?.checked_add(candidate.brep().storage_usage(&limits)?.bytes)
                    .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Result<DupEdgeSourceViewReceipt>>() as u64))
                    .ok_or(Error::Overflow("DupEdge source-view retained bytes"))?.max(1);
                if retained > kernel.bytes() { return Err(Error::Budget("DupEdge source-view budget")); }
                kernel.set_allocated_bytes(retained)?;
                kernel.shrink_to(retained)?;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                Ok(DupEdgeSourceViewReceipt { candidate, prepared: Some(WorkerBuffers { buffers, reservation: buffer,
                    #[cfg(test)] thread: std::thread::current().id() }), source, stamp, session_id, selection_revision, root_selection_revision, document, scene_publication,
                    cancel: Arc::clone(&worker_cancel), _kernel: kernel })
            })).unwrap_or_else(|_| Err(Error::Io("DupEdge source-view worker panicked".into())));
            drop(permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("DupEdge source-view worker start: {error}")))?;
        Ok(DupEdgeSourceViewJob { root:object, cancel, receiver: Some(receiver), delivered: false, recovery_key:self.dupedge_source_view_context_key() })
    }
}

/// Worker output remains charged until dropped. Validation is required again
/// at publication; this receipt does not itself alter a document or scene.
pub struct TopologyRemeshReceipt {
    candidate: CapturedTopologySnapshot,
    proof: Arc<SharedTopologyProof>,
    rebind: Option<om9_kernel::brep::ValidatedSourceRebind>,
    #[cfg(test)]
    rebind_thread: Option<std::thread::ThreadId>,
    prepared: Option<WorkerBuffers>,
    document: Weak<om9_core::model::Document>,
    scene_publication: uuid::Uuid,
    profile: [u64; 2],
    cancel: Arc<AtomicBool>,
    _reservation: Option<ExternalMemoryReservation>,
    // After transfer this keeps the kernel charge alive until `candidate`
    // drops, including every early-error path while preparing the scene.
    _owner: Option<Arc<RemeshDisplayOwner>>,
}
struct WorkerBuffers {
    buffers: om9_scene::scene::PreparedBrepBuffers,
    reservation: ExternalMemoryReservation,
    #[cfg(test)]
    thread: std::thread::ThreadId,
}

/// Private prepared buffers and their two reservations. Buffers drop before
/// the candidate and before the reservation covering the new buffer vectors.
pub(crate) struct PreparedTopologyDisplay {
    buffers: om9_scene::scene::PreparedBrepBuffers,
    receipt: TopologyRemeshReceipt,
    _buffer_reservation: ExternalMemoryReservation,
}
/// Private scene and its retained owners. It must not escape as an independent
/// scene until publication transfers these owners to every snapshot consumer.
pub(crate) struct PreparedTopologyScene {
    scene: om9_scene::scene::Scene,
    receipt: TopologyRemeshReceipt,
    owner: Arc<RemeshDisplayOwner>,
    display_bytes: u64,
    original_buffers: Option<Arc<super::SceneBufferLease>>,
    _construction_reservation: ExternalMemoryReservation,
}
struct RemeshDisplayOwner {
    _proof: Arc<SharedTopologyProof>,
    kernel_reservation: std::sync::Mutex<ExternalMemoryReservation>,
}
struct RemeshBufferOwner {
    _buffer_reservation: ExternalMemoryReservation,
}

struct CachedRemeshRepresentation {
    representation: om9_scene::scene::RootBrepRepresentation,
    proof: Arc<SharedTopologyProof>,
    reclaim_bytes: u64,
}

// Fixed metadata storage keeps this transitional cache bounded and fully
// charged. Entries hold no document; display and buffers own their reservations.
const REMESH_CACHE_SLOTS: usize = 4;
pub(super) struct RemeshRepresentationCache {
    entries: std::sync::Mutex<[Option<CachedRemeshRepresentation>; REMESH_CACHE_SLOTS]>,
    peer: om9_core::transaction::TransactionCoordinator,
    _reservation: ExternalMemoryReservation,
}
impl Drop for RemeshRepresentationCache {
    fn drop(&mut self) { self.peer.prune_optional_cache_index(); }
}
impl RemeshRepresentationCache {
    fn new(controller: &Controller) -> Result<Arc<Self>> {
        let bytes = (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64;
        let held = controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes,s.available_bytes));
        let reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
        let peer = controller.coordinator.memory_admission_peer_with_held(held)?;
        let mut cache = Arc::new(Self { entries: std::sync::Mutex::new(std::array::from_fn(|_| None)), peer, _reservation: reservation });
        Arc::get_mut(&mut cache).expect("new representation cache")._reservation.set_allocated_bytes(bytes)?;
        let owner: Arc<dyn om9_core::transaction::OptionalMemoryCache> = cache.clone();
        controller.coordinator.register_optional_memory_cache(&owner, held)?;
        Ok(cache)
    }
    fn remember(&self, representation: om9_scene::scene::RootBrepRepresentation, proof: Arc<SharedTopologyProof>) -> Result<()> {
        let display_owner = (std::mem::size_of::<RemeshDisplayOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
        let buffer_owner = (std::mem::size_of::<RemeshBufferOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
        let bytes = representation.display_bytes()?.checked_add(display_owner)
            .and_then(|n| n.checked_add(representation.buffer_bytes()))
            .and_then(|n| n.checked_add(buffer_owner)).ok_or(Error::Overflow("remesh representation cache"))?;
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(index) = entries.iter().position(|entry| entry.as_ref().is_some_and(|item|
            item.representation.stamp() == representation.stamp() && item.representation.profile() == representation.profile()
                && item.proof.same_snapshot(&proof))) {
            // The same proven source/profile may already be pinned by a scene.
            // Keep its existing owner instead of inserting a duplicate pin.
            let existing = remove_cache_entry(&mut entries, index);
            let end = entries.iter().position(Option::is_none).expect("removed cache slot");
            entries[end] = existing;
            return Ok(());
        }
        let retired = if entries.iter().all(Option::is_some) {
            let index = entries.iter().position(|entry| entry.as_ref().is_some_and(|item| item.representation.is_exclusive()))
                .ok_or(Error::Busy)?;
            remove_cache_entry(&mut entries, index)
        } else { None };
        let end = entries.iter().position(Option::is_none).expect("available cache slot");
        entries[end] = Some(CachedRemeshRepresentation {
            representation, proof, reclaim_bytes: bytes,
        });
        drop(entries);
        drop(retired);
        Ok(())
    }
    fn find(&self, proof: &SharedTopologyProof, profile: [u64; 2]) -> Option<om9_scene::scene::RootBrepRepresentation> {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let index = entries.iter().position(|entry| entry.as_ref().is_some_and(|item|
            item.representation.stamp() == proof.stamp() && item.representation.profile() == profile
                && item.proof.same_snapshot(proof)))?;
        let entry = remove_cache_entry(&mut entries, index);
        let result = entry.as_ref()?.representation.clone();
        let end = entries.iter().position(Option::is_none).expect("removed cache slot");
        entries[end] = entry;
        Some(result)
    }
    fn find_rebound(&self, proof: &Arc<SharedTopologyProof>, profile: [u64; 2]) -> Option<om9_scene::scene::RootBrepRepresentation> {
        let stamp=proof.stamp();
        let mut entries=self.entries.lock().unwrap_or_else(|e|e.into_inner());
        let index=entries.iter().position(|entry|entry.as_ref().is_some_and(|item| {
            let old=item.representation.stamp();
            Arc::ptr_eq(&item.proof,proof) && item.representation.profile()==profile
                && old.document==stamp.document && old.owner_session==stamp.owner_session
                && old.session_generation==stamp.session_generation && old.object==stamp.object
                && old.generation==stamp.generation
        }))?;
        let entry=remove_cache_entry(&mut entries,index);
        let mut result=entry.as_ref()?.representation.clone();
        let valid=result.retarget_same_object(stamp).is_ok();
        let end=entries.iter().position(Option::is_none).expect("removed cache slot");entries[end]=entry;
        valid.then_some(result)
    }
}
// Entries are kept in oldest-to-newest order without heap allocation.
fn remove_cache_entry(entries: &mut [Option<CachedRemeshRepresentation>; REMESH_CACHE_SLOTS], index: usize)
    -> Option<CachedRemeshRepresentation> {
    let removed = entries[index].take();
    for i in index..REMESH_CACHE_SLOTS - 1 { entries[i] = entries[i + 1].take(); }
    removed
}
impl om9_core::transaction::OptionalMemoryCache for RemeshRepresentationCache {
    fn reclaim_optional_memory(&self, requested: u64) -> Result<u64> {
        if requested == 0 { return Ok(0); }
        let mut entries = match self.entries.try_lock() {
            Ok(entry) => entry,
            Err(std::sync::TryLockError::WouldBlock) => return Ok(0),
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
        };
        let mut retired: [Option<CachedRemeshRepresentation>; REMESH_CACHE_SLOTS] = std::array::from_fn(|_| None);
        let mut count = 0usize;
        let mut selected = 0u64;
        while selected < requested && count < REMESH_CACHE_SLOTS {
            let Some(index) = entries.iter().position(|entry| entry.as_ref().is_some_and(|item| item.representation.is_exclusive())) else { break; };
            let entry = remove_cache_entry(&mut entries, index).expect("exclusive representation");
            selected = selected.saturating_add(entry.reclaim_bytes);
            retired[count] = Some(entry);
            count += 1;
        }
        drop(entries);
        let mut released = 0u64;
        for entry in retired.into_iter().flatten() {
            released = released.checked_add(entry.reclaim_bytes).ok_or(Error::Overflow("remesh cache reclamation"))?;
            drop(entry);
        }
        Ok(released)
    }
}

impl Controller {
    fn remember_remesh_representation(&mut self, representation: om9_scene::scene::RootBrepRepresentation,
        proof: Arc<SharedTopologyProof>) -> Result<()> {
        if self.remesh_representation_cache.is_none() {
            self.remesh_representation_cache = Some(RemeshRepresentationCache::new(self)?);
        }
        self.remesh_representation_cache.as_ref().expect("representation cache").remember(representation, proof)
    }
}
impl PreparedTopologyScene {
    pub(crate) fn validate_current(&self, controller: &Controller) -> Result<()> {
        self.receipt.validate_current(controller)
    }
    /// Atomically replace the CPU scene after revalidating the original scene.
    /// GPU publication and UI scheduling remain separate.
    pub(crate) fn publish(self, controller: &mut Controller) -> Result<()> {
        self.validate_current(controller)?;
        if controller.dupedge_scene_backup.is_some() { return Err(Error::Busy); }
        let PreparedTopologyScene { scene, owner, receipt, display_bytes, original_buffers, _construction_reservation } = self;
        let proof = Arc::clone(&owner._proof);
        let previous = controller.scene().and_then(|current| current.root_brep_representation(&proof.stamp()).ok())
            .filter(|representation| representation.profile() != receipt.profile && representation.display_owner::<RemeshDisplayOwner>()
                .is_some_and(|owner| owner._proof.same_snapshot(&proof)));
        // The private scene now owns the validated display. Release the
        // candidate source and descriptors before releasing their charge.
        drop(receipt);
        {
            let mut reservation = owner.kernel_reservation.lock().unwrap_or_else(|e| e.into_inner());
            reservation.set_allocated_bytes(display_bytes)?;
            reservation.shrink_to(display_bytes)?;
        }
        let buffer_owner = original_buffers.map(|lease| super::SceneBufferOwner {
            epoch: scene.epoch,
            document: Arc::downgrade(&scene.document),
            bytes: scene.shared_buffer_bytes(),
            lease,
        });
        controller.publish_owned_scene(scene, buffer_owner);
        drop(owner);
        drop(_construction_reservation);
        if let Some(previous) = previous { let _ = controller.remember_remesh_representation(previous, Arc::clone(&proof)); }
        let _ = controller.retain_current_topology_proof(proof);
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn scene(&self) -> &om9_scene::scene::Scene { &self.scene }
}
impl PreparedTopologyDisplay {
    pub(crate) fn validate_current(&self, controller: &Controller) -> Result<()> {
        self.receipt.validate_current(controller)
    }
    pub(crate) fn prepare_scene(self, controller: &Controller, metadata_budget: u64) -> Result<PreparedTopologyScene> {
        self.validate_current(controller)?;
        let held = controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache, metadata_budget.max(1), physical, held)?;
        self.prepare_scene_reserved(controller, reservation)
    }
    fn prepare_scene_reserved(self, controller: &Controller, mut reservation: ExternalMemoryReservation) -> Result<PreparedTopologyScene> {
        self.validate_current(controller)?;
        if reservation.class() != MemoryClass::CpuCache || reservation.pending_bytes() != reservation.bytes()
            || !reservation.belongs_to(&controller.coordinator.external_memory_account()) || !reservation.physical_sampled() {
            return Err(Error::Invalid("remesh scene reservation"));
        }
        let current = controller.scene().ok_or(Error::Missing("remesh scene"))?;
        let original_buffers = controller.share_scene_buffer_owner(current)?;
        let held = controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        reservation.recheck_pending()?;
        let estimate = current.render_snapshot_metadata_estimate()?.bytes.checked_add(current.picking_arena_bytes()?)
            .ok_or(Error::Overflow("remesh scene metadata"))?;
        if estimate > reservation.bytes() { return Err(Error::Budget("remesh scene metadata")); }
        let PreparedTopologyDisplay { buffers, mut receipt, mut _buffer_reservation } = self;
        let display_overhead = (std::mem::size_of::<RemeshDisplayOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
        let display_bytes = match receipt.candidate.brep().display_storage_bytes()
            .and_then(|bytes| bytes.checked_add(display_overhead).ok_or(Error::Overflow("remesh display owner"))) {
            Ok(bytes) => bytes.max(1),
            Err(error) => { drop(buffers); drop(receipt); drop(_buffer_reservation); return Err(error); }
        };
        let overhead = (std::mem::size_of::<RemeshBufferOwner>() + 2 * std::mem::size_of::<usize>()) as u64;
        if let Err(error) = controller.coordinator.grow_external_memory_with_held(&mut _buffer_reservation, overhead, physical, held) {
            drop(buffers); drop(receipt); drop(_buffer_reservation); return Err(error);
        }
        let growth = match receipt._reservation.as_mut() {
            Some(kernel) => controller.coordinator.grow_external_memory_with_held(kernel, display_overhead, physical, held),
            None => Err(Error::Invalid("remesh candidate reservation")),
        };
        if let Err(error) = growth {
            drop(buffers); drop(receipt); drop(_buffer_reservation); return Err(error);
        }
        let Some(kernel_reservation) = receipt._reservation.take() else {
            drop(buffers); drop(receipt); drop(_buffer_reservation);
            return Err(Error::Invalid("remesh candidate reservation"));
        };
        let owner = Arc::new(RemeshDisplayOwner { _proof: Arc::clone(&receipt.proof),
            kernel_reservation: std::sync::Mutex::new(kernel_reservation) });
        receipt._owner = Some(Arc::clone(&owner));
        let mut buffer_owner = Arc::new(RemeshBufferOwner { _buffer_reservation });
        let reservation_owner = &mut Arc::get_mut(&mut buffer_owner).expect("new remesh buffer owner")._buffer_reservation;
        if let Err(error) = reservation_owner.set_allocated_bytes(reservation_owner.bytes()) {
            drop(buffers);
            drop(receipt);
            drop(owner);
            drop(buffer_owner);
            return Err(error);
        }
        let display_lease: Arc<dyn std::any::Any + Send + Sync> = owner.clone();
        let buffer_lease: Arc<dyn Send + Sync> = buffer_owner;
        let scene = current.try_replace_root_brep_display(&receipt.proof.stamp(), receipt.candidate.brep(), buffers,
            receipt.profile, display_lease, buffer_lease)?;
        // Each old buffer pins its own lease. Only the new scene's ordinary
        // buffer owner needs the aggregate reference; remeshed buffers must
        // not keep it after the last ordinary buffer is replaced later.
        let original_buffers = (scene.shared_buffer_bytes() != 0).then_some(original_buffers);
        if scene.managed_memory_allowance()? > reservation.bytes() { return Err(Error::Budget("remesh scene metadata")); }
        scene.transfer_memory_allowance(&mut reservation, &controller.coordinator, held)?;
        receipt.validate_current(controller)?;
        Ok(PreparedTopologyScene { scene, owner, receipt, display_bytes, original_buffers, _construction_reservation: reservation })
    }
    #[cfg(test)]
    pub(crate) fn buffers(&self) -> &om9_scene::scene::RenderBuffers { self.buffers.as_ref() }
    #[cfg(test)]
    pub(crate) fn candidate<'a>(&'a self, controller: &'a Controller) -> Result<&'a om9_core::brep::Brep> {
        self.receipt.candidate(controller)
    }
}
impl TopologyRemeshReceipt {
    #[cfg(test)]
    pub(crate) fn rebind_thread(&self) -> Option<std::thread::ThreadId> { self.rebind_thread }
    #[cfg(test)]
    pub(crate) fn retained_kernel_bytes(&self) -> Result<(u64, u64)> {
        let actual = self.candidate.manifest_bytes()?
            .checked_add(self.candidate.brep().storage_usage(&om9_core::budget::Limits::default())?.bytes)
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Result<TopologyRemeshReceipt>>() as u64))
            .ok_or(Error::Overflow("topology remesh retained bytes"))?;
        Ok((self._reservation.as_ref().ok_or(Error::Invalid("remesh candidate reservation"))?.bytes(), actual))
    }
    pub(crate) fn prepare_display(mut self, controller: &Controller, buffer_budget: u64) -> Result<PreparedTopologyDisplay> {
        self.validate_current(controller)?;
        if let Some(rebind) = self.rebind.take() { drop(rebind.publish(&controller.coordinator)?); }
        self.validate_current(controller)?;
        if let Some(prepared) = self.prepared.take() {
            if prepared.reservation.bytes() > buffer_budget { return Err(Error::Budget("remesh display buffer")); }
            return Ok(PreparedTopologyDisplay { buffers: prepared.buffers, receipt: self, _buffer_reservation: prepared.reservation });
        }
        let held = controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache, buffer_budget.max(1), physical, held)?;
        let (buffers, actual) = prepare_source_buffers(self.candidate.brep().source(), self.candidate.brep(), controller.coordinator.limits(), |live, pending| {
            if live.checked_add(pending).ok_or(Error::Overflow("remesh display buffer"))? > reservation.bytes() {
                return Err(Error::Budget("remesh display buffer"));
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        })?;
        reservation.set_allocated_bytes(actual.bytes)?;
        reservation.shrink_to(actual.bytes.max(1))?;
        self.validate_current(controller)?;
        Ok(PreparedTopologyDisplay { buffers, receipt: self, _buffer_reservation: reservation })
    }
    #[cfg(test)]
    pub(crate) fn prepared_on_thread(&self) -> Option<std::thread::ThreadId> {
        self.prepared.as_ref().map(|prepared| prepared.thread)
    }
    // Until publication transfers its reservation, do not expose a clonable
    // BRep to callers: it could outlive this receipt and its memory charge.
    #[cfg(test)]
    pub(crate) fn candidate<'a>(&'a self, controller: &'a Controller) -> Result<&'a om9_core::brep::Brep> {
        self.validate_current(controller)?;
        Ok(self.candidate.brep())
    }
    pub fn validate_current(&self, controller: &Controller) -> Result<()> {
        if self.cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let scene = controller.scene().ok_or(Error::Missing("topology remesh scene"))?;
        if !self.document.ptr_eq(&Arc::downgrade(&scene.document)) {
            return Err(Error::Stale("topology remesh document"));
        }
        if scene.publication_id() != self.scene_publication {
            return Err(Error::Stale("topology remesh scene"));
        }
        if let Some(rebind) = &self.rebind { rebind.validate_current(&controller.coordinator)?; }
        else { validate_proof(controller, &self.proof)?; }
        if self.cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        Ok(())
    }
}

/// At most one worker can run for one proof; the worker holds its reservation.
pub struct TopologyRemeshJob {
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Result<TopologyRemeshReceipt>>>,
    delivered: bool,
}
enum WorkerSource {
    Ready(om9_kernel::brep::RegisteredTopologySource),
    Pending(om9_kernel::brep::PendingSourceRebind),
}
pub(super) enum PendingTopologyRemesh {
    Worker { job: TopologyRemeshJob, display_budget: u64, scene_reservation: ExternalMemoryReservation },
    Cached { proof: Arc<SharedTopologyProof>, document: Weak<om9_core::model::Document>,
        scene_publication: uuid::Uuid, profile: [u64; 2] },
    Stored { proof: Arc<SharedTopologyProof>, document: Weak<om9_core::model::Document>,
        scene_publication: uuid::Uuid, representation: om9_scene::scene::RootBrepRepresentation,
        scene_reservation: ExternalMemoryReservation },
}
impl TopologyRemeshJob {
    pub fn cancel(&self) { self.cancel.store(true, Ordering::Release); }
    pub fn try_result(&mut self) -> Option<Result<TopologyRemeshReceipt>> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(result) => { self.receiver = None; self.delivered = true; Some(result) }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                Some(Err(Error::Io("topology remesh worker exited without a result".into())))
            }
        }
    }
}
impl Drop for TopologyRemeshJob {
    fn drop(&mut self) { if !self.delivered { self.cancel(); } }
}

fn validate_proof(controller: &Controller, proof: &SharedTopologyProof) -> Result<()> {
    proof.verify_unchanged_source(&controller.coordinator)?;
    Ok(())
}

fn published_remesh_matches(scene: &om9_scene::scene::Scene, proof: &SharedTopologyProof, profile: [u64; 2]) -> bool {
    scene.has_root_remesh_profile(&proof.stamp(), profile)
        && scene.root_brep_representation(&proof.stamp()).is_ok_and(|representation|
            representation.display_owner::<RemeshDisplayOwner>().is_some_and(|owner| owner._proof.same_snapshot(proof)))
}

pub(super) fn capture_proof_source(controller: &Controller, proof: Arc<SharedTopologyProof>)
    -> Result<om9_kernel::brep::RegisteredTopologySource> {
    match controller.coordinator.current_evidence_source(&proof.stamp()) {
        Some(source) => proof.bind_captured_source(&controller.coordinator, source),
        None => proof.capture_source(&controller.coordinator),
    }
}

fn validate_remesh_profile(deflection: f64, angular: f64) -> Result<()> {
    if !deflection.is_finite() || deflection <= 0. || !angular.is_finite()
        || angular <= 0. || angular > std::f64::consts::PI {
        return Err(Error::Invalid("topology remesh profile"));
    }
    Ok(())
}

impl Controller {
    /// Schedule one opt-in remesh. Budgets are fixed before the worker starts;
    /// the result must be polled and is discarded on scene replacement.
    pub fn request_topology_remesh(&mut self, deflection: f64, angular: f64,
        kernel_budget: u64, display_budget: u64, scene_budget: u64) -> Result<()> {
        self.request_topology_remesh_inner(None, deflection, angular, kernel_budget, display_budget, scene_budget)
    }
    /// Request a remesh for one explicitly identified attested root.
    pub fn request_topology_remesh_for(&mut self, object: om9_core::identity::ObjectId, deflection: f64, angular: f64,
        kernel_budget: u64, display_budget: u64, scene_budget: u64) -> Result<()> {
        self.request_topology_remesh_inner(Some(object), deflection, angular, kernel_budget, display_budget, scene_budget)
    }
    fn request_topology_remesh_inner(&mut self, object: Option<om9_core::identity::ObjectId>, deflection: f64, angular: f64,
        kernel_budget: u64, display_budget: u64, scene_budget: u64) -> Result<()> {
        if self.topology_remesh_job.is_some() { return Err(Error::Busy); }
        if kernel_budget == 0 || display_budget == 0 || scene_budget == 0 {
            return Err(Error::Invalid("topology remesh budgets"));
        }
        validate_remesh_profile(deflection, angular)?;
        let target_pin = self.topology_source_proof_for(object).ok_or(Error::Missing("topology proof"))?;
        if validate_proof(self,&target_pin).is_err() {
            // Constant-time identity rebind only. Independent canonical owners
            // still use the existing validating worker path.
            let _=target_pin.rebind_shared_source(&self.coordinator);
        }
        let target = target_pin.stamp().object;
        let profile = [deflection.to_bits(), angular.to_bits()];
        let cached = if validate_proof(self, &target_pin).is_ok() {
            self.scene().and_then(|scene| {
                self.coordinator.world().documents.get(&self.document).filter(|document|
                    Arc::ptr_eq(&scene.document, document)
                        && published_remesh_matches(scene, &target_pin, profile))
                    .map(|_| (Arc::downgrade(&scene.document), scene.publication_id()))
            })
        } else { None };
        if let Some((document, scene_publication)) = cached {
            self.topology_remesh_job = Some(PendingTopologyRemesh::Cached {
                proof: target_pin, document, scene_publication, profile,
            });
            return Ok(());
        }
        if validate_proof(self, &target_pin).is_ok() && self.dupedge_scene_backup.is_none() {
            let stored = self.remesh_representation_cache.as_ref().and_then(|cache|
                cache.find(&target_pin, profile).or_else(||cache.find_rebound(&target_pin,profile)));
            if let Some(representation) = stored {
                let scene = self.scene().ok_or(Error::Missing("topology remesh scene"))?;
                if !self.coordinator.world().documents.get(&self.document).is_some_and(|document| Arc::ptr_eq(&scene.document, document)) {
                    return Err(Error::Stale("topology remesh document"));
                }
                let document = Arc::downgrade(&scene.document);
                let scene_publication = scene.publication_id();
                let scene_reservation = self.reserve_remesh_budget(MemoryClass::CpuCache, scene_budget, target)?;
                if let Some(cache) = &self.remesh_representation_cache {
                    let owner: Arc<dyn om9_core::transaction::OptionalMemoryCache> = cache.clone();
                    let _ = self.coordinator.touch_optional_memory_cache(&owner);
                }
                self.topology_remesh_job = Some(PendingTopologyRemesh::Stored {
                    proof: target_pin, document, scene_publication, representation, scene_reservation,
                });
                return Ok(());
            }
        }
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        if physical.is_none() { return Err(Error::Unsupported("physical RAM availability is unavailable")); }
        let kernel = self.reserve_remesh_budget(MemoryClass::Kernel, kernel_budget, target)?;
        let display = self.reserve_remesh_budget(MemoryClass::CpuCache, display_budget, target)?;
        let scene_reservation = self.reserve_remesh_budget(MemoryClass::CpuCache, scene_budget, target)?;
        let job = self.start_topology_remesh_inner(deflection, angular, kernel, Some(display), object)?;
        drop(target_pin);
        self.topology_remesh_job = Some(PendingTopologyRemesh::Worker { job, display_budget, scene_reservation });
        Ok(())
    }
    pub(super) fn reserve_remesh_budget(&mut self, class: MemoryClass, bytes: u64,
        target: om9_core::identity::ObjectId) -> Result<ExternalMemoryReservation> {
        let _target_pin = self.topology_source_proof_for(Some(target)).ok_or(Error::Missing("topology proof"))?;
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        self.coordinator.reserve_external_memory_with_held(class, bytes, physical, held)
    }
    pub fn topology_remesh_pending(&self) -> bool { self.topology_remesh_job.is_some() }
    pub fn cancel_topology_remesh(&mut self) { drop(self.topology_remesh_job.take()); }
    /// Nonblocking while the worker is running. Final validation and scene
    /// metadata construction still execute on the calling thread.
    pub fn poll_topology_remesh(&mut self) -> Option<Result<()>> {
        match self.topology_remesh_job.take()? {
            PendingTopologyRemesh::Cached { proof, document, scene_publication, profile } => Some((|| {
                let scene = self.scene().ok_or(Error::Missing("topology remesh scene"))?;
                if !document.ptr_eq(&Arc::downgrade(&scene.document)) || scene.publication_id() != scene_publication {
                    return Err(Error::Stale("topology remesh scene"));
                }
                validate_proof(self, &proof)?;
                if !published_remesh_matches(scene, &proof, profile) {
                    return Err(Error::Stale("topology remesh display"));
                }
                let _ = self.retain_current_topology_proof(proof);
                Ok(())
            })()),
            PendingTopologyRemesh::Stored { proof, document, scene_publication, representation, mut scene_reservation } => Some((|| {
                let current = self.scene().ok_or(Error::Missing("topology remesh scene"))?;
                if !document.ptr_eq(&Arc::downgrade(&current.document)) || current.publication_id() != scene_publication {
                    return Err(Error::Stale("topology remesh scene"));
                }
                if !self.coordinator.world().documents.get(&self.document).is_some_and(|document| Arc::ptr_eq(&current.document, document)) {
                    return Err(Error::Stale("topology remesh document"));
                }
                if self.dupedge_scene_backup.is_some() { return Err(Error::Busy); }
                validate_proof(self, &proof)?;
                if representation.stamp() != proof.stamp() { return Err(Error::Stale("remesh representation source")); }
                let original_buffers = self.share_scene_buffer_owner(current)?;
                let held = self.scene_cache_usage()?;
                scene_reservation.recheck_pending()?;
                let estimate = current.render_snapshot_metadata_estimate()?.bytes.checked_add(current.picking_arena_bytes()?)
                    .ok_or(Error::Overflow("remesh scene metadata"))?;
                if estimate > scene_reservation.bytes() { return Err(Error::Budget("remesh scene metadata")); }
                let previous = current.root_brep_representation(&proof.stamp()).ok()
                    .filter(|representation| representation.display_owner::<RemeshDisplayOwner>()
                        .is_some_and(|owner| owner._proof.same_snapshot(&proof)));
                let scene = current.try_replace_root_brep_representation(&representation)?;
                let original_buffers = (scene.shared_buffer_bytes() != 0).then_some(original_buffers);
                if scene.managed_memory_allowance()? > scene_reservation.bytes() { return Err(Error::Budget("remesh scene metadata")); }
                scene.transfer_memory_allowance(&mut scene_reservation, &self.coordinator, held)?;
                validate_proof(self, &proof)?;
                let current = self.scene().ok_or(Error::Missing("topology remesh scene"))?;
                if !document.ptr_eq(&Arc::downgrade(&current.document)) || current.publication_id() != scene_publication {
                    return Err(Error::Stale("topology remesh scene"));
                }
                let owner = original_buffers.map(|lease| super::SceneBufferOwner {
                    epoch: scene.epoch, document: Arc::downgrade(&scene.document), bytes: scene.shared_buffer_bytes(), lease,
                });
                self.publish_owned_scene(scene, owner);
                drop(representation);
                drop(scene_reservation);
                if let Some(previous) = previous { let _ = self.remember_remesh_representation(previous, Arc::clone(&proof)); }
                let _ = self.retain_current_topology_proof(proof);
                Ok(())
            })()),
            PendingTopologyRemesh::Worker { mut job, display_budget, scene_reservation } => {
                let Some(result) = job.try_result() else {
                    self.topology_remesh_job = Some(PendingTopologyRemesh::Worker { job, display_budget, scene_reservation });
                    return None;
                };
                Some(result.and_then(|receipt| {
                    receipt.prepare_display(self, display_budget)?
                        .prepare_scene_reserved(self, scene_reservation)?
                        .publish(self)
                }))
            }
        }
    }
    /// Start one private remesh with a caller-reserved upper bound. The fixed
    /// reservation prevents worker growth from bypassing live UI admission.
    /// Native OCCT internals remain outside the managed byte ceiling; cancel
    /// waits for a currently executing native call to return.
    pub fn start_topology_remesh(&mut self, deflection: f64, angular: f64,
        reservation: ExternalMemoryReservation) -> Result<TopologyRemeshJob> {
        self.start_topology_remesh_inner(deflection, angular, reservation, None, None)
    }
    /// Pre-reserve render-buffer storage so geometry and its CPU display buffers
    /// are both prepared on the worker, without waiting on the UI thread.
    pub fn start_topology_remesh_with_display(&mut self, deflection: f64, angular: f64,
        reservation: ExternalMemoryReservation, display_reservation: ExternalMemoryReservation) -> Result<TopologyRemeshJob> {
        self.start_topology_remesh_inner(deflection, angular, reservation, Some(display_reservation), None)
    }
    fn start_topology_remesh_inner(&mut self, deflection: f64, angular: f64,
        reservation: ExternalMemoryReservation, display_reservation: Option<ExternalMemoryReservation>,
        object: Option<om9_core::identity::ObjectId>) -> Result<TopologyRemeshJob> {
        validate_remesh_profile(deflection, angular)?;
        if reservation.class() != MemoryClass::Kernel || reservation.pending_bytes() != reservation.bytes()
            || !reservation.belongs_to(&self.coordinator.external_memory_account()) || !reservation.physical_sampled() {
            return Err(Error::Invalid("topology remesh reservation"));
        }
        if display_reservation.as_ref().is_some_and(|owner| owner.class() != MemoryClass::CpuCache
            || owner.pending_bytes() != owner.bytes() || !owner.physical_sampled()
            || !owner.belongs_to(&self.coordinator.external_memory_account())) {
            return Err(Error::Invalid("topology remesh display reservation"));
        }
        let proof = self.topology_source_proof_for(object).ok_or(Error::Missing("topology proof"))?;
        let mut needs_rebind = false;
        if let Err(error) = validate_proof(self, &proof) {
            if !matches!(error, Error::Stale(_)) { return Err(error); }
            needs_rebind = !proof.rebind_shared_source(&self.coordinator)?;
        }
        let scene = self.scene().ok_or(Error::Missing("topology remesh scene"))?;
        let document = Arc::downgrade(&scene.document);
        let scene_publication = scene.publication_id();
        let current = self.coordinator.world().documents.get(&self.document).ok_or(Error::Missing("topology remesh document"))?;
        if !document.ptr_eq(&Arc::downgrade(current)) { return Err(Error::Stale("topology remesh document")); }
        let worker_permit = proof.acquire_worker()?;
        let source = if needs_rebind { WorkerSource::Pending(proof.prepare_source_rebind(&self.coordinator)?) }
            else { WorkerSource::Ready(capture_proof_source(self, proof)?) };
        let limits = *self.coordinator.limits();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-topology-remesh".into()).spawn(move || {
            let _worker_permit = worker_permit;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<TopologyRemeshReceipt> {
                let mut reservation = reservation;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let refusal = std::cell::RefCell::new(None);
                let mut account = |live: u64, pending: u64| -> Result<()> {
                    let result = (|| {
                        if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                        let needed = live.checked_add(pending).ok_or(Error::Overflow("topology remesh reservation"))?;
                        if needed > reservation.bytes() { return Err(Error::Budget("topology remesh reservation")); }
                        reservation.set_allocated_bytes(live)?;
                        reservation.recheck_pending()
                    })();
                    if let Err(error) = &result { *refusal.borrow_mut() = Some(error.clone()); }
                    result
                };
                #[cfg(test)]
                let rebind_thread = matches!(&source, WorkerSource::Pending(_)).then(|| std::thread::current().id());
                let (ready, rebind) = match source {
                    WorkerSource::Ready(source) => (Some(source), None),
                    WorkerSource::Pending(source) => (None, Some(source.validate(&limits, &mut account)?)),
                };
                let source = ready.as_ref().or_else(|| rebind.as_ref().map(|item| item.source())).expect("worker source");
                let candidate = capture_source_topology(source.source(), deflection, angular, &limits, &mut account)
                    .map_err(|error| refusal.borrow_mut().take().unwrap_or_else(|| Error::Io(format!("topology remesh: {error}"))))?;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let candidate = candidate.into_rebound_for_source(&source, &limits, &mut account)?;
                drop(account);
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let prepared = if let Some(mut display_reservation) = display_reservation {
                    let (buffers, actual) = prepare_source_buffers(source.source(), candidate.brep(), &limits, |live, pending| {
                            if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                            if live.checked_add(pending).ok_or(Error::Overflow("remesh wire reservation"))? > display_reservation.bytes() {
                                return Err(Error::Budget("remesh display buffer"));
                            }
                            display_reservation.set_allocated_bytes(live)?;
                            display_reservation.recheck_pending()
                    })?;
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    display_reservation.set_allocated_bytes(actual.bytes)?;
                    display_reservation.shrink_to(actual.bytes.max(1))?;
                    Some(WorkerBuffers { buffers, reservation: display_reservation,
                        #[cfg(test)] thread: std::thread::current().id() })
                } else { None };
                // Keep the candidate and receipt envelope charged for their
                // lifetime; proof and prepared buffers have separate leases.
                let retained = candidate.manifest_bytes()?
                    .checked_add(candidate.brep().storage_usage(&limits)?.bytes)
                    .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Result<TopologyRemeshReceipt>>() as u64))
                    .ok_or(Error::Overflow("topology remesh retained bytes"))?;
                if retained > reservation.bytes() { return Err(Error::Budget("topology remesh reservation")); }
                reservation.set_allocated_bytes(retained)?;
                reservation.shrink_to(retained)?;
                let proof = match ready { Some(source) => source.into_proof(), None => rebind.as_ref().expect("validated rebind").proof_handle() };
                Ok(TopologyRemeshReceipt { candidate, proof, rebind, prepared, document, scene_publication,
                    #[cfg(test)] rebind_thread,
                    profile: [deflection.to_bits(), angular.to_bits()], cancel: Arc::clone(&worker_cancel),
                    _reservation: Some(reservation), _owner: None })
            })).unwrap_or_else(|_| Err(Error::Io("topology remesh worker panicked".into())));
            drop(_worker_permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("topology remesh worker start: {error}")))?;
        Ok(TopologyRemeshJob { cancel, receiver: Some(receiver), delivered: false })
    }
}
