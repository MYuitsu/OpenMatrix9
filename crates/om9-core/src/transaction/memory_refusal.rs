// SPDX-License-Identifier: LGPL-2.1-or-later
//! One final external-admission refusal per shared account; no heap log/history.
use super::*;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct ExternalMemoryRefusal {
    pub stage: &'static str,
    pub error_kind: &'static str,
    pub error_context: &'static str,
    pub class: &'static str,
    pub coordinator_authority: SessionId,
    pub coordinator_epoch: u64,
    pub requested_bytes: u64,
    pub requested_allocated_bytes: u64,
    pub requested_pending_bytes: u64,
    pub caller_held_bytes: u64,
    pub prior_external_reserved_bytes: Option<u64>,
    pub prior_pending_bytes: Option<u64>,
    pub configured_limit_bytes: u64,
    pub learned_ram_ceiling_bytes: Option<u64>,
    pub effective_limit_bytes: u64,
    pub physical_input_provided: bool,
    pub physical_probe_configured: bool,
    pub physical_new_pending_headroom_bytes: Option<u64>,
    pub commit_probe_configured: bool,
    pub commit_was_checked: bool,
    pub commit_new_pending_headroom_bytes: Option<u64>,
    /// Time since the final failed attempt, not since a later probe/report.
    pub age_at_report_us: u64,
}
#[derive(Clone)]
pub(super) struct StoredExternalRefusal {
    pub(super) observed: Instant,
    pub(super) report: ExternalMemoryRefusal,
}
fn cause(error: &Error) -> Option<(&'static str, &'static str)> {
    match error {
        Error::Budget(context) => Some(("budget", *context)),
        Error::Overflow(context) => Some(("overflow", *context)),
        Error::Unsupported(context @ ("physical RAM availability is unavailable" | "system commit availability is unavailable")) => Some(("probe_unavailable", *context)),
        Error::Invalid(context @ ("physical RAM sample" | "system commit sample")) => Some(("invalid_probe", *context)),
        _ => None,
    }
}
pub(super) fn class_name(class: crate::budget::memory::MemoryClass) -> &'static str {
    use crate::budget::memory::MemoryClass::*;
    match class { Document => "document", Undo => "undo", CpuCache => "cpu_cache", Upload => "upload", Kernel => "kernel", Export => "export" }
}
impl StoredExternalRefusal {
    pub(super) fn snapshot(&self) -> ExternalMemoryRefusal {
        let mut report = self.report.clone();
        report.age_at_report_us = self.observed.elapsed().as_micros().min(u64::MAX as u128) as u64;
        report
    }
    pub(super) fn matches(&self, error: &Error) -> bool {
        cause(error).is_some_and(|(kind, context)| self.report.error_kind == kind && self.report.error_context == context)
    }
}
impl TransactionCoordinator {
    /// Cheap last-refusal projection: no storage walk, OS probes or mutation.
    /// A successful retry leaves prior historical refusals intact.
    pub fn last_external_memory_refusal(&self) -> Option<ExternalMemoryRefusal> {
        self.external_reservations.lock().unwrap_or_else(|e| e.into_inner())
            .last_external_refusal.as_ref().map(StoredExternalRefusal::snapshot)
    }
    pub(super) fn store_external_refusal(&self, refusal: StoredExternalRefusal) {
        let mut account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        // Concurrent callers may publish their result in a different order
        // from the locked admission attempts. Never replace a newer event.
        if account.last_external_refusal.as_ref().is_none_or(|old| old.observed <= refusal.observed) {
            account.last_external_refusal = Some(refusal);
        }
    }
    pub(super) fn refusal_attempt(&self, class: crate::budget::memory::MemoryClass,
        bytes: u64, allocated: u64, physical_provided: bool, held: Usage,
        account: &ExternalReservations) -> ExternalMemoryRefusal {
        ExternalMemoryRefusal {
            stage: "external_reservation", error_kind: "pending", error_context: "pending",
            class: class_name(class), coordinator_authority: self.authority, coordinator_epoch: self.epoch,
            requested_bytes: bytes, requested_allocated_bytes: allocated, requested_pending_bytes: bytes - allocated,
            caller_held_bytes: held.bytes,
            prior_external_reserved_bytes: account.by_class.iter().try_fold(0u64, |n, bytes| n.checked_add(*bytes)),
            prior_pending_bytes: account.pending_bytes().ok(), configured_limit_bytes: self.limits.max_bytes,
            learned_ram_ceiling_bytes: account.managed_ceiling, effective_limit_bytes: account.constrain(self.limits).max_bytes,
            physical_input_provided: physical_provided, physical_probe_configured: account.physical_sampler.is_some(),
            physical_new_pending_headroom_bytes: None, commit_probe_configured: account.commit_sampler.is_some(),
            commit_was_checked: false, commit_new_pending_headroom_bytes: None, age_at_report_us: 0,
        }
    }
}
impl ExternalMemoryRefusal {
    pub(super) fn finish(mut self, error: &Error, account: &ExternalReservations, limits: Limits) -> Option<StoredExternalRefusal> {
        let (kind, context) = cause(error)?;
        self.error_kind = kind;
        self.error_context = context;
        self.learned_ram_ceiling_bytes = account.managed_ceiling;
        self.effective_limit_bytes = account.constrain(limits).max_bytes;
        Some(StoredExternalRefusal { observed: Instant::now(), report: self })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::memory::{CommitMemory, MemoryClass};
    const MIB: u64 = 1024 * 1024;
    fn empty() -> TransactionCoordinator {
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap()
    }
    #[test]
    fn physical_refusal_uses_attempt_headroom_and_cheap_read_does_not_probe_again() {
        let mut coordinator = empty();
        coordinator.set_physical_memory_sampler(|| Some((1024 * MIB, 256 * MIB + 123)));
        coordinator.set_commit_memory_sampler(|| panic!("physical failure must precede commit probe"));
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Export, 124, Some((1024 * MIB, 1024 * MIB))), Err(Error::Budget("available physical RAM"))));
        coordinator.set_physical_memory_sampler(|| panic!("cheap refusal query must not probe"));
        let refusal = coordinator.last_external_memory_refusal().unwrap();
        assert_eq!(refusal.stage, "external_reservation");
        assert_eq!(refusal.class, "export");
        assert_eq!(refusal.error_context, "available physical RAM");
        assert_eq!(refusal.requested_pending_bytes, 124);
        assert_eq!(refusal.physical_new_pending_headroom_bytes, Some(123));
        assert!(!refusal.commit_was_checked);
        assert_eq!(refusal.commit_new_pending_headroom_bytes, None);
        assert_eq!(refusal.prior_external_reserved_bytes, Some(0));
        assert_eq!(refusal.coordinator_authority, coordinator.authority());
        assert_eq!(coordinator.snapshot(), before);
    }
    #[test]
    fn historical_commit_refusal_is_distinct_from_fresh_report_and_success() {
        let mut coordinator = empty();
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 768 * MIB - 123 }));
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 124, None), Err(Error::Budget("available system commit"))));
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 0 }));
        let lease = coordinator.reserve_external_memory(MemoryClass::CpuCache, 1, None).unwrap();
        let report = coordinator.memory_diagnostics(Ok(Usage::default()), Some(1));
        let old = report.last_external_refusal.unwrap();
        assert_eq!(old.class, "kernel");
        assert_eq!(old.requested_bytes, 124);
        assert_eq!(old.commit_new_pending_headroom_bytes, Some(123));
        assert!(old.commit_was_checked);
        if let memory_diagnostics::MemoryProbe::Measured { sample, .. } = report.commit {
            assert!(sample.additional_pending_headroom_bytes.unwrap() > 123);
        } else { panic!("fresh commit must be available") }
        assert_eq!(lease.bytes(), 1);
    }
    #[test]
    fn failed_growth_preserves_lease_and_older_publication_cannot_replace_newer_event() {
        let coordinator = empty();
        let mut lease = coordinator.reserve_external_memory(MemoryClass::CpuCache, 100, None).unwrap();
        let ceiling = coordinator.retained_usage().unwrap().bytes + 16 * 1024;
        coordinator.external_reservations.lock().unwrap().managed_ceiling = Some(ceiling);
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.grow_external_memory_with_held(&mut lease, 32 * 1024, None, Usage::default()), Err(Error::Budget("managed RAM"))));
        let old = coordinator.external_reservations.lock().unwrap().last_external_refusal.clone().unwrap();
        assert_eq!(old.report.prior_pending_bytes, Some(100));
        assert_eq!(old.report.requested_bytes, 32 * 1024);
        assert_eq!(old.report.effective_limit_bytes, ceiling);
        assert_eq!(lease.bytes(), 100);
        assert_eq!(lease.pending_bytes(), 100);
        assert!(coordinator.reserve_external_memory(MemoryClass::Export, MIB, None).is_err());
        coordinator.store_external_refusal(old);
        let newer = coordinator.last_external_memory_refusal().unwrap();
        assert_eq!(newer.class, "export");
        assert_eq!(newer.requested_bytes, MIB);
        assert_eq!(coordinator.snapshot(), before);
    }
    #[test]
    fn unavailable_or_invalid_probe_is_not_reported_as_verified_headroom() {
        let mut coordinator = empty();
        coordinator.set_physical_memory_sampler(|| None);
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Upload, 1, Some((1024 * MIB, 1024 * MIB))), Err(Error::Unsupported(_))));
        let refusal = coordinator.last_external_memory_refusal().unwrap();
        assert_eq!(refusal.error_kind, "probe_unavailable");
        assert_eq!(refusal.physical_new_pending_headroom_bytes, None);
        coordinator.set_physical_memory_sampler(|| Some((0,0)));
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Upload, 1, Some((1024 * MIB, 1024 * MIB))), Err(Error::Invalid(_))));
        assert_eq!(coordinator.last_external_memory_refusal().unwrap().error_kind, "invalid_probe");
    }
    #[test]
    fn invalid_external_request_is_not_a_memory_pressure_event() {
        let coordinator = empty();
        assert!(coordinator.reserve_external_memory(MemoryClass::Document, 1, None).is_err());
        assert!(coordinator.reserve_external_memory(MemoryClass::Export, 0, None).is_err());
        assert!(coordinator.reserve_external_memory_with_allocated(MemoryClass::Export, 1, 2, None, Usage::default()).is_err());
        assert!(coordinator.last_external_memory_refusal().is_none());
    }
}
