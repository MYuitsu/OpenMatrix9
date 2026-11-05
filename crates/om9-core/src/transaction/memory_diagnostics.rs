// SPDX-License-Identifier: LGPL-2.1-or-later
//! Explicit, read-only managed-memory observation. Never called per frame.
use super::*;
use crate::budget::memory::MemoryClass;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MemoryProbe<T> {
    NotConfigured,
    Failed,
    Invalid,
    Measured { sample: T, sample_age_at_report_us: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{budget::memory::CommitMemory, geometry::Geometry, model::Entity};
    const MIB: u64 = 1024 * 1024;
    fn empty() -> TransactionCoordinator {
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap()
    }

    #[test]
    fn pending_and_materialized_are_not_double_counted_or_confused_with_undo_quota() {
        let mut coordinator = empty();
        let baseline = coordinator.retained_usage().unwrap().bytes;
        let lease = coordinator.reserve_external_memory_with_allocated(MemoryClass::Export, 5000, 2000, None, Usage::default()).unwrap();
        coordinator.set_physical_memory_sampler(|| Some((1024 * MIB, 256 * MIB + 4000)));
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 768 * MIB - 4000 }));
        let held = Usage { bytes: 100, ..Usage::default() };
        let report = coordinator.memory_diagnostics(Ok(held), Some(1000));
        assert_eq!(report.tracked_total_bytes, Some(baseline + 5100));
        assert_eq!(report.tracked_materialized_bytes, Some(baseline + 2100));
        assert_eq!(report.tracked_pending_bytes, Some(3000));
        assert_eq!(report.external_classes[5].reserved_bytes, 5000);
        assert_eq!(report.external_classes[5].materialized_bytes, 2000);
        assert_eq!(report.undo_policy.max_memory_mb, 1024);
        let assessment = report.request.unwrap();
        assert_eq!(assessment.fits_known_managed_inventory, Some(true));
        assert_eq!(assessment.fits_physical_sample, Some(true));
        assert_eq!(assessment.fits_commit_sample, Some(true));
        let larger = coordinator.memory_diagnostics(Ok(held), Some(1500)).request.unwrap();
        assert_eq!(larger.fits_physical_sample, Some(false), "physical and commit headroom must not be added");
        assert_eq!(larger.fits_commit_sample, Some(false));
        assert_eq!(lease.pending_bytes(), 3000);
        assert_eq!(coordinator.external_reservations.lock().unwrap().managed_ceiling, None, "observation must not tighten policy");
    }

    #[test]
    fn unknown_failed_and_invalid_probes_do_not_become_verified_headroom() {
        let mut coordinator = empty();
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(1));
        assert!(matches!(report.physical, MemoryProbe::NotConfigured));
        assert!(matches!(report.commit, MemoryProbe::NotConfigured));
        assert_eq!(report.request.unwrap().fits_commit_sample, None);
        coordinator.set_physical_memory_sampler(|| None);
        coordinator.set_commit_memory_sampler(|| None);
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(1));
        assert!(matches!(report.physical, MemoryProbe::Failed));
        assert!(matches!(report.commit, MemoryProbe::Failed));
        assert_eq!(report.request.unwrap().fits_physical_sample, None);
        coordinator.set_physical_memory_sampler(|| Some((0, 0)));
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1, used_bytes: 2 }));
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(1));
        assert!(matches!(report.physical, MemoryProbe::Invalid));
        assert!(matches!(report.commit, MemoryProbe::Invalid));
    }

    #[test]
    fn reports_overage_without_reclamation_or_mutating_expired_proofs() {
        struct NeverReclaim;
        impl OptionalMemoryCache for NeverReclaim {
            fn reclaim_unused_memory(&self) -> Result<u64> { panic!("diagnostic must not reclaim quota") }
            fn reclaim_optional_memory(&self, _: u64) -> Result<u64> { panic!("diagnostic must not evict cache") }
        }
        let coordinator = empty();
        let owner: Arc<dyn OptionalMemoryCache> = Arc::new(NeverReclaim);
        coordinator.register_optional_memory_cache(&owner, Usage::default()).unwrap();
        let baseline = coordinator.retained_usage().unwrap().bytes;
        let document = Arc::new(Document::new("expired diagnostic proof"));
        *coordinator.current_validation_cache.lock().unwrap() = Some((Arc::downgrade(&document), *coordinator.limits()));
        drop(document);
        coordinator.external_reservations.lock().unwrap().managed_ceiling = Some(1);
        let before = coordinator.snapshot();
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(1));
        let block = (std::mem::size_of::<Document>() + 2 * std::mem::size_of::<usize>()) as u64;
        assert_eq!(report.tracked_total_bytes, Some(baseline + block));
        assert_eq!(report.managed_overage_bytes, Some(baseline + block - 1));
        assert_eq!(report.managed_headroom_bytes, Some(0));
        assert_eq!(report.request.unwrap().fits_known_managed_inventory, Some(false));
        assert!(coordinator.current_validation_cache.lock().unwrap().is_some());
        assert_eq!(coordinator.external_reservations.lock().unwrap().managed_ceiling, Some(1));
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn storage_projection_matches_admission_and_keeps_history_and_cached_undo_inventory() {
        let limits = Limits::default();
        let mut document = Document::new("diagnostic history");
        let entity = Entity::new("point", Geometry::Point([1.,2.,3.]), None);
        let id = entity.id; document.entities.insert(id, entity);
        let document_id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let source = coordinator.world.documents[&document_id].stamp(id).unwrap();
        let change = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1.,0.,0.]),
        }])).unwrap();
        coordinator.commit(change).unwrap();
        let capture = coordinator.capture_tracked_document(document_id).unwrap();
        let expected = coordinator.retained_usage().unwrap().bytes;
        let before = coordinator.snapshot();
        let stats = coordinator.undo_memory_stats();
        let report = coordinator.memory_diagnostics(Ok(Usage { bytes: 1024, ..Usage::default() }), None);
        assert_eq!(report.tracked_total_bytes, Some(expected + 1024));
        assert_eq!(report.undo_cached_estimated_bytes, stats.estimated_bytes);
        assert_eq!(report.undo_capture_change_pending, stats.capture_change_pending);
        assert_eq!(coordinator.undo_memory_stats(), stats);
        assert_eq!(coordinator.snapshot(), before);
        drop(capture);
    }

    #[test]
    fn incomplete_caller_inventory_or_overflow_returns_unknown_totals() {
        let coordinator = empty();
        let report = coordinator.memory_diagnostics(Err(Error::Budget("caller scratch")), Some(1));
        assert_eq!(report.caller_held_bytes, None);
        assert_eq!(report.tracked_total_bytes, None);
        assert!(report.inventory_error.unwrap().contains("caller scratch"));
        assert_eq!(report.request.unwrap().fits_known_managed_inventory, None);
        let report = coordinator.memory_diagnostics(Ok(Usage { bytes: u64::MAX, ..Usage::default() }), Some(1));
        assert!(report.inventory_error.unwrap().contains("overflow"));
        assert_eq!(report.tracked_materialized_bytes, None);
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(u64::MAX));
        assert_eq!(report.request.unwrap().fits_known_managed_inventory, Some(false));
    }

    #[test]
    fn probe_ages_belong_to_the_report_and_existing_pending_overage_is_visible() {
        let mut coordinator = empty();
        let lease = coordinator.reserve_external_memory(MemoryClass::Kernel, 100, None).unwrap();
        coordinator.set_physical_memory_sampler(|| Some((1024 * MIB, 256 * MIB + 99)));
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 768 * MIB - 99 }));
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(0));
        if let MemoryProbe::Measured { sample, sample_age_at_report_us } = report.physical {
            assert_eq!(sample.additional_pending_headroom_bytes, Some(0));
            assert!(sample_age_at_report_us <= report.collection_duration_us);
        } else { panic!("valid physical sample must be measured") }
        let request = report.request.unwrap();
        assert_eq!(request.fits_physical_sample, Some(false));
        assert_eq!(request.fits_commit_sample, Some(false));
        assert_eq!(lease.pending_bytes(), 100);
    }
}
#[derive(Debug, Serialize)]
pub struct PhysicalObservation {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub reserve_bytes: u64,
    pub additional_pending_headroom_bytes: Option<u64>,
}
#[derive(Debug, Serialize)]
pub struct CommitObservation {
    pub limit_bytes: u64,
    pub used_bytes: u64,
    pub reserve_bytes: u64,
    pub additional_pending_headroom_bytes: Option<u64>,
}
#[derive(Debug, Serialize)]
pub struct ExternalClassObservation {
    pub class: &'static str,
    pub reserved_bytes: u64,
    pub materialized_bytes: u64,
    pub pending_bytes: u64,
}
#[derive(Debug, Serialize)]
pub struct PendingRequestObservation {
    pub additional_pending_bytes: u64,
    pub fits_known_managed_inventory: Option<bool>,
    pub fits_physical_sample: Option<bool>,
    pub fits_commit_sample: Option<bool>,
}
#[derive(Debug, Serialize)]
pub struct MemoryDiagnostics {
    pub schema_version: u32,
    pub coordinator_authority: SessionId,
    pub coordinator_epoch: u64,
    pub last_external_refusal: Option<memory_refusal::ExternalMemoryRefusal>,
    pub configured_limit_bytes: u64,
    pub learned_ram_ceiling_bytes: Option<u64>,
    pub effective_limit_bytes: u64,
    pub coordinator_retained_bytes: Option<u64>,
    pub external_documents_controls_and_reservations_bytes: Option<u64>,
    pub caller_held_bytes: Option<u64>,
    pub tracked_total_bytes: Option<u64>,
    pub tracked_materialized_bytes: Option<u64>,
    pub tracked_pending_bytes: Option<u64>,
    pub managed_headroom_bytes: Option<u64>,
    pub managed_overage_bytes: Option<u64>,
    pub inventory_error: Option<String>,
    pub external_classes: [ExternalClassObservation; 6],
    pub physical: MemoryProbe<PhysicalObservation>,
    pub commit: MemoryProbe<CommitObservation>,
    pub undo_policy: UndoPolicy,
    pub undo_ram_limit_mb: Option<u64>,
    pub undo_cached_estimated_bytes: u64,
    pub undo_cached_growth_since_inventory_bytes: u64,
    pub undo_capture_change_pending: bool,
    pub request: Option<PendingRequestObservation>,
    pub collection_duration_us: u64,
    pub not_measured: [&'static str; 4],
}
fn micros(started: Instant) -> u64 { started.elapsed().as_micros().min(u64::MAX as u128) as u64 }
fn fits_pending(pending: u64, additional: u64, free: u64, reserve: u64) -> bool {
    free >= reserve && pending.checked_add(additional).is_some_and(|bytes| bytes <= free - reserve)
}

impl TransactionCoordinator {
    /// Inspect known storage under the admission locks without applying or
    /// widening policy. `held` excludes registered owners; if that inventory
    /// fails, totals stay unknown while counters/probe statuses remain readable.
    /// An optional request
    /// models NEW pending bytes only; this does not reserve/admit an operation.
    pub fn memory_diagnostics(&self, held: Result<Usage>, additional_pending: Option<u64>) -> MemoryDiagnostics {
        let started = Instant::now();
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let physical_sample = reservations.physical_sampler.map(|sample| (sample(), Instant::now()));
        let commit_sample = reservations.commit_sampler.map(|sample| (sample(), Instant::now()));
        // Diagnostic inspection must still work when policy has tightened below
        // current retained usage. Relax only this storage projection, never the
        // account or a validation/admission path. Overflow checks remain active.
        let inspection = Limits { max_bytes: u64::MAX, max_objects: u64::MAX, max_nodes: u64::MAX,
            max_points: u64::MAX, max_history: u64::MAX, max_outputs: u64::MAX, ..self.limits };
        let held_bytes = held.as_ref().ok().map(|usage| usage.bytes);
        let pending_result = reservations.pending_bytes();
        let inventory = (|| -> Result<(u64, u64, u64)> {
            pending_result.as_ref().map_err(Clone::clone)?;
            let held = held?;
            let internal = self.retained_storage_usage(&self.identities, None, false, &inspection)?;
            let mut outside = self.external_storage_usage(None, &inspection, &external,
                &reservations.accounted_classes(0)?, None)?;
            // Unlike validation, observation leaves expired proofs untouched.
            let proof = self.current_validation_cache.lock().unwrap_or_else(|e| e.into_inner());
            if proof.as_ref().is_some_and(|(weak, _)| weak.strong_count() == 0
                && !external.iter().any(|owner| owner.ptr_eq(weak))) {
                outside.add(Usage { bytes: (std::mem::size_of::<Document>() + 2 * std::mem::size_of::<usize>()) as u64,
                    ..Usage::default() }, &inspection)?;
            }
            let total = internal.bytes.checked_add(outside.bytes).and_then(|n| n.checked_add(held.bytes))
                .ok_or(Error::Overflow("diagnostic managed bytes"))?;
            Ok((internal.bytes, outside.bytes, total))
        })();
        let (internal, outside, total, inventory_error) = match inventory {
            Ok((a,b,c)) => (Some(a), Some(b), Some(c), None),
            Err(error) => (None, None, None, Some(error.to_string())),
        };
        let pending = pending_result.ok();
        let effective = reservations.constrain(self.limits).max_bytes;
        let classes = [MemoryClass::Document, MemoryClass::Undo, MemoryClass::CpuCache,
            MemoryClass::Upload, MemoryClass::Kernel, MemoryClass::Export];
        let names = ["document", "undo", "cpu_cache", "upload", "kernel", "export"];
        let external_classes = std::array::from_fn(|index| ExternalClassObservation {
            class: names[index], reserved_bytes: reservations.by_class[classes[index].index()],
            pending_bytes: reservations.pending_by_class[classes[index].index()],
            materialized_bytes: reservations.by_class[classes[index].index()].saturating_sub(reservations.pending_by_class[classes[index].index()]),
        });
        let physical = match physical_sample {
            None => MemoryProbe::NotConfigured,
            Some((None, _)) => MemoryProbe::Failed,
            Some((Some((total, available)), when)) if total > 0 && available <= total => MemoryProbe::Measured {
                sample: PhysicalObservation { total_bytes: total, available_bytes: available,
                    reserve_bytes: (total / 10).max(256 * 1024 * 1024),
                    additional_pending_headroom_bytes: pending.and_then(|pending|
                        crate::budget::memory::available_for_pending(total, available, pending).ok()) },
                sample_age_at_report_us: micros(when),
            },
            Some(_) => MemoryProbe::Invalid,
        };
        let commit = match commit_sample {
            None => MemoryProbe::NotConfigured,
            Some((None, _)) => MemoryProbe::Failed,
            Some((Some(sample), when)) if sample.limit_bytes > 0 && sample.used_bytes <= sample.limit_bytes => MemoryProbe::Measured {
                sample: CommitObservation { limit_bytes: sample.limit_bytes, used_bytes: sample.used_bytes,
                    reserve_bytes: 256 * 1024 * 1024,
                    additional_pending_headroom_bytes: pending.and_then(|pending| sample.available_for_pending(pending).ok()) },
                sample_age_at_report_us: micros(when),
            },
            Some(_) => MemoryProbe::Invalid,
        };
        let request = additional_pending.map(|bytes| PendingRequestObservation {
            additional_pending_bytes: bytes,
            fits_known_managed_inventory: total.map(|total| total.checked_add(bytes).is_some_and(|next| next <= effective)),
            fits_physical_sample: match &physical {
                MemoryProbe::Measured { sample, .. } => pending.map(|pending| fits_pending(pending, bytes, sample.available_bytes, sample.reserve_bytes)),
                _ => None,
            },
            fits_commit_sample: match &commit {
                MemoryProbe::Measured { sample, .. } => pending.map(|pending| fits_pending(pending, bytes, sample.limit_bytes - sample.used_bytes, sample.reserve_bytes)),
                _ => None,
            },
        });
        let undo = self.undo_memory_stats();
        MemoryDiagnostics {
            schema_version: 1, configured_limit_bytes: self.limits.max_bytes,
            coordinator_authority: self.authority, coordinator_epoch: self.epoch,
            last_external_refusal: reservations.last_external_refusal.as_ref().map(memory_refusal::StoredExternalRefusal::snapshot),
            learned_ram_ceiling_bytes: reservations.managed_ceiling, effective_limit_bytes: effective,
            coordinator_retained_bytes: internal, external_documents_controls_and_reservations_bytes: outside,
            caller_held_bytes: held_bytes, tracked_total_bytes: total,
            tracked_materialized_bytes: total.zip(pending).and_then(|(n, pending)| n.checked_sub(pending)), tracked_pending_bytes: pending,
            managed_headroom_bytes: total.map(|n| effective.saturating_sub(n)), managed_overage_bytes: total.map(|n| n.saturating_sub(effective)),
            inventory_error, external_classes, physical, commit,
            undo_policy: self.undo_policy(), undo_ram_limit_mb: self.undo_ram_limit_mb(),
            undo_cached_estimated_bytes: undo.estimated_bytes, undo_cached_growth_since_inventory_bytes: undo.growth_since_inventory,
            undo_capture_change_pending: undo.capture_change_pending, request, collection_duration_us: micros(started),
            not_measured: ["native_allocator_overhead_and_peak", "process_working_set_and_private_commit",
                "unregistered_auxiliary_peer_and_retired_owners", "gpu_residency_and_driver_allocations"],
        }
    }
}
