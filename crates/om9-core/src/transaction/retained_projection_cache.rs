// SPDX-License-Identifier: LGPL-2.1-or-later
//! Optional admitted memoization of an exact immutable owner set. Reservations
//! and journal metadata are always projected afresh; this is not a second quota.
use super::*;

pub(super) struct ProjectionRoot {
    data: Mutex<Option<RetainedProjection>>,
    // Last: registered metadata remains charged until all cache roots retire.
    header: Mutex<ExternalMemoryReservation>,
}
impl std::fmt::Debug for ProjectionRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("ProjectionRoot") }
}
pub(super) struct ProjectionReclaimer {
    registry: Arc<Mutex<ExternalReservations>>,
    root: Arc<ProjectionRoot>,
}
impl std::fmt::Debug for ProjectionReclaimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("ProjectionReclaimer") }
}
impl OptionalMemoryCache for ProjectionReclaimer {
    fn reclaim_optional_memory(&self, requested: u64) -> Result<u64> {
        if requested == 0 { return Ok(0); }
        let retired = self.root.data.lock().unwrap_or_else(|e|e.into_inner()).take();
        let bytes = retired.as_ref().map_or(0, |p|p.charge.bytes);
        drop(retired);
        Ok(bytes)
    }
}
impl Drop for ProjectionReclaimer {
    fn drop(&mut self) {
        // Match the existing topology-cache lifecycle: remove expired Weak
        // registrations before their owner's header reservation is released.
        let mut account = self.registry.lock().unwrap_or_else(|e|e.into_inner());
        account.optional_caches.retain(|entry|entry.owner.strong_count() != 0);
        if account.optional_caches.is_empty() { account.optional_caches = Vec::new(); }
    }
}

#[derive(Debug)]
struct ProjectionCharge { owner: Arc<AtomicU64>, bytes: u64 }
impl ProjectionCharge {
    fn new(owner: &Arc<AtomicU64>, bytes: u64) -> Result<Self> {
        owner.fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| v.checked_add(bytes))
            .map_err(|_| Error::Overflow("projection account"))?;
        Ok(Self { owner: owner.clone(), bytes })
    }
}
impl Drop for ProjectionCharge {
    fn drop(&mut self) {
        let old = self.owner.fetch_sub(self.bytes, Ordering::AcqRel);
        debug_assert!(old >= self.bytes);
    }
}

#[derive(Debug)]
pub(super) struct RetainedProjection {
    internal: Vec<Weak<Document>>,
    external: Vec<(Weak<Document>, bool)>,
    external_capacity: usize,
    excluded: Option<Weak<Document>>,
    generation_owner: Arc<AtomicU64>,
    generation: u64,
    limits: Limits,
    documents: Usage,
    metadata_nodes: u64,
    external_bytes: u64,
    external_maxima: Usage,
    charge: Usage,
    // Last: memory and Weak control blocks are released before the shared
    // account charge. Admission peers observe this owner through their account.
    account: ProjectionCharge,
}

pub(super) fn reserved_bytes(classes: &AccountedClasses<'_>) -> Result<u64> {
    classes.bytes.iter().try_fold(0u64, |sum, n| sum.checked_add(*n).ok_or(Error::Overflow("projection reservations")))?
        .checked_sub(classes.cache_snapshot).and_then(|v|v.checked_add(classes.cache_bytes.load(Ordering::Acquire)))
        .ok_or(Error::Overflow("projection reservation snapshot"))
}

impl TransactionCoordinator {
    /// Payload totals count unique immutable documents. Publication can replace
    /// a world alias while the journal already owns both document versions.
    /// Repetition/order may change; every unmatched allocation must still occur
    /// on the other side. Weak keys prevent address reuse during this proof.
    fn same_retained_document_set(&self, stored: &[Weak<Document>]) -> bool {
        if stored.len() != self.retained_documents(None).count() { return false; }
        stored.iter().zip(self.retained_documents(None)).all(|(old, current)| {
            old.as_ptr() == Arc::as_ptr(current)
                || (stored.iter().any(|key| key.as_ptr() == Arc::as_ptr(current))
                    && self.retained_documents(None).any(|doc| Arc::as_ptr(doc) == old.as_ptr()))
        })
    }
    /// No allocation, admission decision or external-attribution estimate.
    /// Includes all internal documents, so it bounds their history-only subset
    /// even after the last external capture drops. Fresh metadata is separate.
    pub(super) fn cached_journal_document_byte_upper(&self) -> Result<Option<u64>> {
        let Some(root) = &self.retained_projection else { return Ok(None); };
        let bytes = root.data.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
            .filter(|p| storage_limits_cover(&p.limits, &self.limits) && self.same_retained_document_set(&p.internal))
            .map(|p| p.documents.bytes);
        bytes.map(|bytes| self.retained_metadata_usage(&self.identities, None, false, &self.limits)?.bytes
            .checked_add(bytes).ok_or(Error::Overflow("journal document upper bound"))).transpose()
    }
    pub(super) fn make_projection_root(parent: &Self, held: Usage) -> Option<(Arc<ProjectionRoot>, Arc<ProjectionReclaimer>)> {
        let bytes = (std::mem::size_of::<ProjectionRoot>() + std::mem::size_of::<ProjectionReclaimer>()
            + 4 * std::mem::size_of::<usize>()) as u64;
        let physical = parent.physical_memory_sample().ok()?;
        let header = parent.reserve_external_memory_with_held(crate::budget::memory::MemoryClass::CpuCache, bytes, physical, held).ok()?;
        let root = Arc::new(ProjectionRoot { data: Mutex::new(None), header: Mutex::new(header) });
        let owner = Arc::new(ProjectionReclaimer { registry: parent.external_reservations.clone(), root: root.clone() });
        root.header.lock().unwrap_or_else(|e|e.into_inner()).set_allocated_bytes(bytes).ok()?;
        let registered: Arc<dyn OptionalMemoryCache> = owner.clone();
        parent.register_optional_memory_cache(&registered, held).ok()?;
        Some((root, owner))
    }
    pub(super) fn cached_retained_projection(
        &self, identities: &IdentityLedger, entry: Option<&UndoEntry>, recording: bool,
        limits: &Limits, external: &Vec<Weak<Document>>, classes: &AccountedClasses<'_>,
        excluded: Option<&Arc<Document>>,
    ) -> Result<Option<Usage>> {
        let Some(root) = &self.retained_projection else { return Ok(None); };
        let generation = self.capture_generation.load(Ordering::Acquire);
        let hit = {
            let cache = root.data.lock().unwrap_or_else(|e| e.into_inner());
            cache.as_ref().filter(|p| {
                generation != u64::MAX && p.generation == generation
                    && Arc::ptr_eq(&p.generation_owner, &self.capture_generation)
                    && storage_limits_cover(&p.limits, limits)
                    && p.external_capacity == external.capacity()
                    && p.internal.len() == self.retained_documents(entry).count()
                    && p.internal.iter().zip(self.retained_documents(entry)).all(|(a,b)| a.as_ptr() == Arc::as_ptr(b))
                    && p.external.len() == external.len()
                    && p.external.iter().zip(external).all(|((a,live),b)| a.ptr_eq(b) && *live == (b.strong_count() != 0))
                    && match (&p.excluded, excluded) {
                        (None, None) => true,
                        (Some(a), Some(b)) => a.as_ptr() == Arc::as_ptr(b),
                        _ => false,
                    }
            }).map(|p| (p.documents, p.external_bytes, p.metadata_nodes, p.external_maxima))
        };
        if let Some((documents, external_bytes, metadata_nodes, external_maxima)) = hit {
            let projected = (|| -> Result<Usage> {
                external_maxima.check(limits)?;
                let mut usage = self.retained_metadata_usage(identities, entry, recording, limits)?;
                // Retained node counters use max(metadata, per-document), not
                // subtraction. An identity-ledger change needs a fresh maximum.
                if usage.nodes != metadata_nodes { return Err(Error::Busy); }
                add_retained_document_usage(&mut usage, documents, limits)?;
                usage.add(Usage { bytes: external_bytes.checked_add(reserved_bytes(classes)?)
                    .ok_or(Error::Overflow("projection external bytes"))?, ..Usage::default() }, limits)?;
                Ok(usage)
            })();
            if let Ok(usage) = projected {
              if self.capture_generation.load(Ordering::Acquire) == generation {
                return Ok(Some(usage));
              }
            }
        }
        // A changed external set does not invalidate the internal document
        // projection. The caller may reuse it before retiring this index.
        Ok(None)
    }

    pub(super) fn cached_internal_projection(&self, identities: &IdentityLedger,
        entry: Option<&UndoEntry>, recording: bool, limits: &Limits) -> Result<Option<Usage>> {
        let Some(root) = &self.retained_projection else { return Ok(None); };
        let stored = {
            let cache = root.data.lock().unwrap_or_else(|e| e.into_inner());
            cache.as_ref().filter(|p| storage_limits_cover(&p.limits, limits)
                && p.internal.len() == self.retained_documents(entry).count()
                && p.internal.iter().zip(self.retained_documents(entry)).all(|(a,b)|a.as_ptr() == Arc::as_ptr(b)))
                .map(|p| (p.documents, p.metadata_nodes))
        };
        if let Some((documents, metadata_nodes)) = stored {
            let projected = (|| -> Result<Usage> {
                let mut usage = self.retained_metadata_usage(identities, entry, recording, limits)?;
                if usage.nodes < metadata_nodes { return Err(Error::Busy); }
                add_retained_document_usage(&mut usage, documents, limits)?;
                Ok(usage)
            })();
            if let Ok(usage) = projected { return Ok(Some(usage)); }
        }
        Ok(None)
    }

    pub(super) fn retire_projection(&self, current_account: &Arc<AtomicU64>) -> Usage {
        let Some(root) = &self.retained_projection else { return Usage::default(); };
        let old = root.data.lock().unwrap_or_else(|e| e.into_inner()).take();
        let charge = old.as_ref().filter(|p|Arc::ptr_eq(&p.account.owner, current_account))
            .map_or(Usage::default(), |p| p.charge);
        drop(old);
        charge
    }

    pub(super) fn remember_retained_projection(
        &self, identities: &IdentityLedger, entry: Option<&UndoEntry>, recording: bool,
        limits: &Limits, external: &Vec<Weak<Document>>, classes: &AccountedClasses<'_>,
        excluded: Option<&Arc<Document>>, external_bytes: u64, internal: Usage, total: &mut Usage,
        generation: u64, external_maxima: Usage,
    ) -> Result<()> {
        let Some(root) = &self.retained_projection else { return Ok(()); };
        if generation == u64::MAX || self.capture_generation.load(Ordering::Acquire) != generation { return Ok(()); }
        let metadata = self.retained_metadata_usage(identities, entry, recording, limits)?;
        let documents = Usage {
            bytes: internal.bytes.checked_sub(metadata.bytes).ok_or(Error::Overflow("projection metadata"))?,
            objects: internal.objects.checked_sub(metadata.objects).ok_or(Error::Overflow("projection objects"))?,
            nodes: internal.nodes, points: internal.points,
            history: internal.history.checked_sub(metadata.history).ok_or(Error::Overflow("projection history"))?,
            outputs: internal.outputs.checked_sub(metadata.outputs).ok_or(Error::Overflow("projection outputs"))?,
        };
        let count = self.retained_documents(entry).count();
        let cache_limits = Limits { max_bytes: u64::MAX, ..*limits };
        // Admit the index and its weak-only Arc block upper bound before any
        // allocation. Count aliases conservatively, including the excluded Arc.
        let mut charge = Usage::array::<Weak<Document>>(count)?;
        charge.add(Usage::array::<(Weak<Document>, bool)>(external.len())?, &cache_limits)?;
        let blocks = count.checked_add(external.len()).and_then(|v| v.checked_add(1))
            .ok_or(Error::Overflow("projection weak blocks"))?;
        charge.add(Usage::array::<Document>(blocks)?, &cache_limits)?;
        charge.add(Usage::array::<usize>(blocks.checked_mul(2).and_then(|v|v.checked_add(4))
            .ok_or(Error::Overflow("projection controls"))?)?, &cache_limits)?;
        // Cache is optional. Failure to admit it preserves the already proved
        // operation; it never relaxes the actual document/reservation budget.
        let mut with_cache = *total;
        if with_cache.add(charge, limits).is_err() { return Ok(()); }
        let pending = classes.pending.checked_add(charge.bytes).ok_or(Error::Overflow("projection pending peak"))?;
        if let Some(sample) = classes.physical_sampler {
            let Some((total, available)) = sample() else { return Ok(()); };
            if crate::budget::memory::available_for_pending(total, available, pending).is_err() { return Ok(()); }
        }
        if let Some(sample) = classes.commit_sampler {
            let Some(commit) = sample() else { return Ok(()); };
            if commit.available_for_pending(pending).is_err() { return Ok(()); }
        }
        // Every positive change runs under the shared account lock. Until it
        // is released, no peer can observe a charge before materialization.
        let account = ProjectionCharge::new(classes.cache_bytes, charge.bytes)?;
        let mut internal_keys = Vec::new();
        let mut external_keys = Vec::new();
        if internal_keys.try_reserve_exact(count).is_err() || external_keys.try_reserve_exact(external.len()).is_err() {
            return Ok(());
        }
        internal_keys.extend(self.retained_documents(entry).map(Arc::downgrade));
        external_keys.extend(external.iter().map(|w| (w.clone(), w.strong_count() != 0)));
        let cache = RetainedProjection {
            internal: internal_keys, external: external_keys, external_capacity: external.capacity(),
            excluded: excluded.map(Arc::downgrade), generation_owner: self.capture_generation.clone(),
            generation, limits: *limits, documents, metadata_nodes: metadata.nodes,
            external_bytes, charge, external_maxima, account,
        };
        // Raw external Arc captures can expire without a tracked lease drop.
        // Do not memoize that transition: a live Weak cannot resurrect, so
        // all-live here proves these external payloads stayed live during the
        // inventory. Subsequent expiry invalidates the live-state signature.
        if self.capture_generation.load(Ordering::Acquire) != generation
            || external.iter().any(|w| w.strong_count() == 0) { return Ok(()); }
        *root.data.lock().unwrap_or_else(|e| e.into_inner()) = Some(cache);
        *total = with_cache;
        Ok(())
    }
}
