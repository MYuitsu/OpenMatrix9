// SPDX-License-Identifier: LGPL-2.1-or-later
//! Optional derived owners. Reclamation never runs under the account lock.
use super::*;

/// Return only bytes whose allocation/allowance actually stopped being owned.
/// Implementations must not block on an active reader or alter source/history.
pub trait OptionalMemoryCache: Send + Sync {
    /// Return unused allowance before retiring materialized data. Default
    /// owners have no separately reclaimable pending quota.
    fn reclaim_unused_memory(&self) -> Result<u64> { Ok(0) }
    fn reclaim_optional_memory(&self, requested: u64) -> Result<u64>;
}
pub(super) struct OptionalCacheEntry {
    sequence: u64,
    pub(super) owner: Weak<dyn OptionalMemoryCache>,
}
fn reclaimable(error: &Error) -> bool {
    matches!(error, Error::Budget("bytes" | "managed RAM" | "available physical RAM" | "available system commit" | "external memory headroom"))
}
impl TransactionCoordinator {
    /// Mark an actual interactive cache read. The existing weak index doubles
    /// as its recency queue; touching its newest entry is allocation-free O(1).
    pub fn touch_optional_memory_cache(&self, owner: &Arc<dyn OptionalMemoryCache>) -> Result<()> {
        let weak = Arc::downgrade(owner);
        let mut account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let Some(index) = account.optional_caches.iter().rposition(|entry| entry.owner.ptr_eq(&weak)) else { return Ok(()); };
        if index + 1 == account.optional_caches.len() { return Ok(()); }
        let sequence = account.optional_cache_sequence.checked_add(1).ok_or(Error::Overflow("optional cache sequence"))?;
        let mut entry = account.optional_caches.remove(index);
        entry.sequence = sequence;
        account.optional_caches.push(entry);
        account.optional_cache_sequence = sequence;
        Ok(())
    }
    pub fn recheck_external_memory_with_reclamation(&self, reservation: &ExternalMemoryReservation) -> Result<()> {
        if !Arc::ptr_eq(&self.external_reservations, &reservation.registry) {
            return Err(Error::Invalid("external memory reservation account"));
        }
        self.with_optional_cache_reclamation(1, || reservation.recheck_pending())
    }
    /// Remove expired weak owners before their Arc allocation can linger as
    /// uncharged registry metadata. Live cache data is never touched here.
    pub fn prune_optional_cache_index(&self) {
        let mut account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        account.optional_caches.retain(|entry| entry.owner.strong_count() != 0);
        if account.optional_caches.is_empty() { account.optional_caches = Vec::new(); }
    }
    pub fn physical_memory_sample(&self) -> Result<Option<(u64, u64)>> {
        let sampler = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).physical_sampler;
        sampler.map(|sample| sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))).transpose()
    }
    /// Empty admission peer sharing existing owner/policy registries. No new
    /// document, journal or identity allocation is made by this constructor.
    pub fn memory_admission_peer(&self) -> Result<Self> {
        self.memory_admission_peer_with_held(Usage::default())
    }
    /// Keep caller-private materialized bytes in the account while peer/cache
    /// metadata is allocated. The caller retains its usual lifetime obligation
    /// for those bytes after this startup operation completes.
    pub fn memory_admission_peer_with_held(&self, held: Usage) -> Result<Self> {
        self.available()?;
        let _startup = if held.bytes != 0 {
            Some(self.reserve_external_memory_with_allocated(crate::budget::memory::MemoryClass::CpuCache,
                held.bytes, held.bytes, None, Usage { bytes: 0, ..held })?)
        } else {
            if held != Usage::default() { self.retained_usage_with_held_usage(held)?; }
            None
        };
        if !self.memory_peer_initialized.load(Ordering::Acquire) {
            self.register_retained_documents()?;
            self.memory_peer_initialized.store(true, Ordering::Release);
        }
        let identities = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).empty_peer_identities.clone();
        let mut peer = Self {
            world: World { documents: BTreeMap::new() }, limits: self.limits,
            physical_memory_sampler: self.physical_memory_sampler, authority: self.authority, epoch: 1,
            scope_generation: 0, undo: Vec::new(), redo: Vec::new(), scope: None, barrier: false, history_pending: None,
            benchmark_no_undo: false, undo_policy: UndoPolicy::default(), undo_ram_limit_mb: None,
            undo_memory_stats: UndoMemoryStats::default(), undo_capture_generation: 0,
            undo_evidence_upper: 0, undo_evidence_count: 0,
            overflow_evidence: DeferredUndoEvidence { entries: Vec::new(), _reservation: None },
            shared_peer_identities: Some(identities.clone()), identities,
            current_validation_cache: Mutex::new(None), external_documents: self.external_documents.clone(),
            current_storage_cache: Mutex::new([None, None]),
            retained_projection: None,
            projection_reclaimer: None,
            projection_validation: Mutex::new(()),
            storage_cache_hits: AtomicU64::new(0), storage_cache_misses: AtomicU64::new(0),
            representations: self.representations.clone(),
            external_reservations: self.external_reservations.clone(), capture_generation: self.capture_generation.clone(),
            external_export_active: self.external_export_active.clone(),
            memory_peer_initialized: AtomicBool::new(true),
        };
        if let Some((root, owner)) = Self::make_projection_root(self, Usage::default()) {
            peer.retained_projection = Some(root);
            peer.projection_reclaimer = Some(owner);
        }
        Ok(peer)
    }
    /// Register a weak callback. The index Vec capacity belongs to the account
    /// and is charged by accounted_classes, including old/new growth overlap.
    pub fn register_optional_memory_cache(&self, owner: &Arc<dyn OptionalMemoryCache>, held: Usage) -> Result<()> {
        let result = self.with_optional_cache_reclamation(1, || self.try_register_optional_memory_cache(owner, held));
        match result {
            Err(error) if reclaimable(&error) => {
                if owner.reclaim_optional_memory(1)? == 0 { return Err(error); }
                self.try_register_optional_memory_cache(owner, held)
            }
            other => other,
        }
    }
    fn try_register_optional_memory_cache(&self, owner: &Arc<dyn OptionalMemoryCache>, held: Usage) -> Result<()> {
        self.available()?;
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let mut account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        account.optional_caches.retain(|entry| entry.owner.strong_count() != 0);
        let weak = Arc::downgrade(owner);
        if account.optional_caches.iter().any(|entry| entry.owner.ptr_eq(&weak)) { return Ok(()); }
        let sequence = account.optional_cache_sequence.checked_add(1).ok_or(Error::Overflow("optional cache sequence"))?;
        let entry = OptionalCacheEntry { sequence, owner: weak };
        if account.optional_caches.len() == account.optional_caches.capacity() {
            let needed = account.optional_caches.len().checked_add(1).ok_or(Error::Overflow("optional cache index"))?;
            let requested = needed.max(account.optional_caches.capacity().saturating_mul(2));
            let bytes = Usage::array::<OptionalCacheEntry>(requested)?.bytes;
            let pending = account.pending_bytes()?;
            if let Some(sample) = account.physical_sampler {
                let (total, available) = sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
                let ceiling = (u128::from(total) * 3 / 5) as u64;
                account.managed_ceiling = Some(account.managed_ceiling.map_or(ceiling, |old| old.min(ceiling)));
                if bytes > crate::budget::memory::available_for_pending(total, available, pending)? {
                    return Err(Error::Budget("available physical RAM"));
                }
            }
            if account.commit_available(pending)?.is_some_and(|available| bytes > available) {
                return Err(Error::Budget("available system commit"));
            }
            let limits = account.constrain(self.limits).remaining_after(held)?;
            let mut classes = account.accounted_classes(0)?;
            let index = crate::budget::memory::MemoryClass::Document.index();
            classes[index] = classes[index].checked_add(bytes).ok_or(Error::Overflow("optional cache index"))?;
            self.validate_retained_owners_with_external(&self.identities, None, false, &limits, &external, &classes)?;
            let mut next = Vec::new();
            next.try_reserve_exact(requested).map_err(|_| Error::Budget("optional cache index"))?;
            let actual = Usage::array::<OptionalCacheEntry>(next.capacity())?.bytes;
            classes[index] = classes[index].checked_sub(bytes).and_then(|n| n.checked_add(actual))
                .ok_or(Error::Overflow("optional cache index"))?;
            self.validate_retained_owners_with_external(&self.identities, None, false, &limits, &external, &classes)?;
            next.extend(account.optional_caches.drain(..));
            next.push(entry);
            account.optional_caches = next;
        } else { account.optional_caches.push(entry); }
        account.optional_cache_sequence = sequence;
        Ok(())
    }
    /// Unused quotas first, then least-recently-read arenas. Each phase is
    /// bounded by its initial watermark. Newly touched/registered entries are
    /// deferred; no temporary callback vector or second owner ledger is made.
    pub fn reclaim_optional_memory(&self, requested: u64) -> Result<u64> {
        let (maximum, index_capacity) = {
            let account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
            (account.optional_cache_sequence, account.optional_caches.capacity())
        };
        let mut released = 0u64;
        for unused_only in [true, false] {
            let mut cursor = 0;
            while released < requested {
                let next = {
                    let account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
                    account.optional_caches.iter().find(|entry| entry.sequence > cursor && entry.sequence <= maximum)
                        .map(|entry| (entry.sequence, entry.owner.clone()))
                };
                let Some((sequence, owner)) = next else { break; };
                cursor = sequence;
                if let Some(owner) = owner.upgrade() {
                    let bytes = if unused_only { owner.reclaim_unused_memory()? }
                        else { owner.reclaim_optional_memory(requested - released)? };
                    released = released.checked_add(bytes).ok_or(Error::Overflow("optional memory released"))?;
                }
            }
        }
        let mut account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        account.optional_caches.retain(|entry| entry.owner.strong_count() != 0);
        if account.optional_caches.is_empty() { account.optional_caches = Vec::new(); }
        released.checked_add(Usage::array::<OptionalCacheEntry>(index_capacity.saturating_sub(account.optional_caches.capacity()))?.bytes)
            .ok_or(Error::Overflow("optional index released"))
    }
    pub(super) fn with_optional_cache_reclamation<T>(&self, _requested: u64, mut operation: impl FnMut() -> Result<T>) -> Result<T> {
        enum Orphan {
            Occt(crate::brep::BrepRepresentation),
            Native(crate::geometry::NativeBrepRepresentation),
        }
        let mut orphan_budget = self.representations.try_lock().map_or(0, |store| store.entry_count());
        let maximum = {
            let account = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
            let index = Usage::array::<OptionalCacheEntry>(account.optional_caches.capacity())?.bytes;
            account.by_class.iter().fold(u128::from(index) + u128::from(account.projection_cache_bytes.load(Ordering::Acquire)), |sum, bytes| sum + u128::from(*bytes))
        };
        let mut released = 0u128;
        // Retry after each indivisible entry, stopping as soon as admission
        // succeeds. Bound concurrent cache churn by the initially held bytes.
        loop {
            match operation() {
                Err(error) if reclaimable(&error) => {
                    // Source-dead entries require no reconstruction proof. Pull
                    // one out under the store lock, then release its owners
                    // after unlocking before rechecking admission.
                    if orphan_budget != 0 {
                        let retired = match self.representations.try_lock() {
                            Ok(mut store) => store.take_orphaned().map(Orphan::Occt)
                                .or_else(|| store.take_orphaned_native().map(Orphan::Native)),
                            Err(std::sync::TryLockError::Poisoned(error)) => {
                                let mut store = error.into_inner();
                                store.take_orphaned().map(Orphan::Occt)
                                    .or_else(|| store.take_orphaned_native().map(Orphan::Native))
                            }
                            Err(std::sync::TryLockError::WouldBlock) => None,
                        };
                        if let Some(retired) = retired {
                            orphan_budget -= 1;
                            match retired { Orphan::Occt(value) => drop(value), Orphan::Native(value) => drop(value) }
                            continue;
                        }
                        orphan_budget = 0;
                    }
                    if released >= maximum { return Err(error); }
                    let bytes = self.reclaim_optional_memory(1)?;
                    if bytes == 0 { return Err(error); }
                    released += u128::from(bytes);
                }
                result => return result,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::memory::MemoryClass;

    struct SegmentedCache(Mutex<Vec<ExternalMemoryReservation>>);
    impl OptionalMemoryCache for SegmentedCache {
        fn reclaim_optional_memory(&self, _: u64) -> Result<u64> {
            let entry = self.0.lock().unwrap().pop();
            let bytes = entry.as_ref().map_or(0, ExternalMemoryReservation::bytes);
            drop(entry);
            Ok(bytes)
        }
    }

    #[test]
    fn admission_reclaims_multiple_entries_from_one_owner() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 6 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        let entries = (0..4).map(|_| coordinator.reserve_external_memory(MemoryClass::CpuCache, MIB, None).unwrap()).collect();
        let cache = Arc::new(SegmentedCache(Mutex::new(entries)));
        let owner: Arc<dyn OptionalMemoryCache> = cache.clone();
        coordinator.register_optional_memory_cache(&owner, Usage::default()).unwrap();

        let allocation = coordinator.reserve_external_memory(MemoryClass::Kernel, 4 * MIB, None).unwrap();
        assert_eq!(cache.0.lock().unwrap().len(), 1, "stop after enough entries are released");
        assert_eq!(allocation.bytes(), 4 * MIB);
    }

    #[test]
    fn exhausted_cache_refuses_without_losing_an_existing_reservation() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 6 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        let retained = coordinator.reserve_external_memory(MemoryClass::Export, MIB, None).unwrap();
        let entries = (0..3).map(|_| coordinator.reserve_external_memory(MemoryClass::CpuCache, MIB, None).unwrap()).collect();
        let cache = Arc::new(SegmentedCache(Mutex::new(entries)));
        let owner: Arc<dyn OptionalMemoryCache> = cache.clone();
        coordinator.register_optional_memory_cache(&owner, Usage::default()).unwrap();

        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 6 * MIB, None), Err(Error::Budget("bytes"))));
        assert!(cache.0.lock().unwrap().is_empty());
        assert_eq!(retained.bytes(), MIB);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), MIB);
    }
}
