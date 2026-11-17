// SPDX-License-Identifier: LGPL-2.1-or-later
//! Soft journal retention policy, independent of document/recipe limits.
use super::*;
use serde::{Deserialize, Serialize};

pub const INVENTORY_INTERVAL_BYTES: u64 = 100 * 1024 * 1024;

// A later journal entry may still restore the same immutable source even when
// the insertion entry is trimmed. Moving an existing owner needs no allocation;
// entries with other evidence remain untouched.
fn document_references_evidence(document: &Document, evidence: &UndoEvidence) -> bool {
    let Ok(stamp) = document.stamp(evidence.object) else { return false; };
    if stamp.generation != evidence.generation || stamp.owner_session != evidence.owner_session
        || stamp.session_generation != evidence.session_generation { return false; }
    let Ok(entity) = document.resolve(&stamp) else { return false; };
    let Geometry::Brep(brep) = &entity.geometry else { return false; };
    brep.source_allocation_key() == evidence.source_key
        && brep.topology_allocation_key() == evidence.topology_key
}
fn entry_references_evidence(entry: &UndoEntry, evidence: &UndoEvidence) -> bool {
    entry.before.get(&evidence.document).into_iter()
        .chain(entry.after.get(&evidence.document))
        .any(|document| document_references_evidence(document, evidence))
}

// A document's own storage has already deduplicated its aliases. Return true
// only on the first occurrence in this document when an earlier owner exists.
pub(super) fn retained_before_document(index: &mut BTreeMap<usize, Option<usize>>, key: usize, document: usize) -> bool {
    index.insert(key, Some(document)).is_some_and(|prior| prior != Some(document))
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UndoMemoryStats {
    pub estimated_bytes: u64,
    pub growth_since_inventory: u64,
    pub full_inventories: u64,
    /// Capture attribution refreshes avoided with an admitted immutable owner
    /// byte bound below the soft quota. Statistics remain marked stale.
    pub deferred_attribution_refreshes: u64,
    /// A tracked capture changed, evidence sharing may have changed, or the
    /// last inventory refresh failed.
    pub capture_change_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndoPolicy {
    pub minimum_steps: usize,
    pub max_memory_mb: u64,
}
impl Default for UndoPolicy {
    fn default() -> Self {
        Self {
            minimum_steps: 1,
            max_memory_mb: 1024,
        }
    }
}
impl UndoPolicy {
    pub fn validate(self) -> Result<()> {
        if self.minimum_steps == 0 || self.minimum_steps > 1_000_000 {
            return Err(Error::Invalid("minimum Undo steps"));
        }
        if self.max_memory_mb == 0 || self.max_memory_mb.checked_mul(1024 * 1024).is_none() {
            return Err(Error::Invalid("Undo memory budget"));
        }
        Ok(())
    }
}
impl TransactionCoordinator {
    /// Bound and admit the temporary indexes before the inventory allocates.
    /// The shared account lock remains held throughout inventory, so no peer
    /// can spend the same available headroom until the indexes are dropped.
    fn admit_undo_inventory_scratch(&self, external: &Vec<Weak<Document>>, reservations: &ExternalReservations, held: Usage) -> Result<()> {
        let mut docs = 0usize;
        let mut resources = 0usize;
        let mut breps = 0usize;
        let mut native = 0usize;
        let mut max_breps = 0usize;
        let mut max_native = 0usize;
        let mut count = |document: &Document| -> Result<()> {
            docs = docs.checked_add(1).ok_or(Error::Overflow("Undo inventory documents"))?;
            resources = resources.checked_add(document.resources.len()).ok_or(Error::Overflow("Undo inventory resources"))?;
            let document_breps = retained_breps(document).count();
            let document_native = retained_native_breps(document).count();
            breps = breps.checked_add(document_breps).ok_or(Error::Overflow("Undo inventory BReps"))?;
            native = native.checked_add(document_native).ok_or(Error::Overflow("Undo inventory native BReps"))?;
            max_breps = max_breps.max(document_breps);
            max_native = max_native.max(document_native);
            Ok(())
        };
        for document in self.saved_worlds().flat_map(|world| world.documents.values()) { count(document)?; }
        for capture in reservations.capture_leases.iter().filter_map(Weak::upgrade) { count(&capture.document)?; }
        for document in self.undo.iter().chain(&self.redo).flat_map(|entry| entry.before.values().chain(entry.after.values())) {
            count(document)?;
        }
        let sources = reservations.brep_source_leases.len();
        let brep_source_slots = breps.checked_add(sources).ok_or(Error::Overflow("Undo inventory BRep sources"))?;
        let brep_display_slots = breps.checked_add(reservations.brep_display_leases.len()).ok_or(Error::Overflow("Undo inventory BRep displays"))?;
        let brep_topology_slots = brep_source_slots.checked_add(reservations.brep_display_leases.len()).ok_or(Error::Overflow("Undo inventory BRep topologies"))?;
        let native_source_slots = native.checked_add(sources).ok_or(Error::Overflow("Undo inventory native sources"))?;
        let mut scratch = Usage::btree::<usize, ()>(docs)?;
        let unlimited = Limits { max_bytes: u64::MAX, ..Limits::default() };
        for usage in [Usage::btree::<(usize, usize), ()>(resources)?,
            Usage::btree::<usize, Option<usize>>(brep_source_slots)?, Usage::btree::<usize, Option<usize>>(brep_display_slots)?,
            Usage::btree::<usize, Option<usize>>(brep_topology_slots)?, Usage::btree::<usize, Option<usize>>(native_source_slots)?,
            Usage::btree::<usize, Option<usize>>(native)?] {
            scratch.add(usage, &unlimited)?;
        }
        if self.undo_evidence_count != 0 {
            scratch.add(Usage::btree::<usize, (&Arc<dyn std::any::Any + Send + Sync>, u64, usize, usize, bool)>(self.undo_evidence_count)?, &unlimited)?;
            scratch.add(Usage::btree::<usize, (&Arc<CapturedBrepSourceInner>, usize, usize)>(self.undo_evidence_count)?, &unlimited)?;
        }
        scratch.add(crate::model::BrepStorageIndex::scratch_usage(max_breps, max_native, &unlimited)?, &unlimited)?;
        let pending = reservations.pending_bytes()?;
        let mut limits = reservations.constrain(self.limits);
        if let Some(sample) = reservations.physical_sampler {
            let (total, available) = sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
            let available = crate::budget::memory::available_for_pending(total, available, pending)?;
            if scratch.bytes > available { return Err(Error::Budget("available physical RAM")); }
            limits.max_bytes = limits.max_bytes.min((u128::from(total) * 3 / 5) as u64);
        }
        if reservations.commit_available(pending)?.is_some_and(|available| scratch.bytes > available) {
            return Err(Error::Budget("available system commit"));
        }
        let remaining = limits.remaining_after(scratch)?.remaining_after(held)?;
        self.validate_retained_owners_with_external(&self.identities, None, false, &remaining,
            external, &reservations.accounted_classes(0)?)?;
        Ok(())
    }

    pub fn undo_policy(&self) -> UndoPolicy {
        self.undo_policy
    }

    pub fn undo_ram_limit_mb(&self) -> Option<u64> {
        self.undo_ram_limit_mb
    }

    /// Hardware is supplied by the platform owner; portable core owns the 50% rule.
    pub fn set_undo_ram_capacity(&mut self, total_bytes: u64) -> Result<()> {
        self.available()?;
        if self.scope.is_some() {
            return Err(Error::Busy);
        }
        let max_mb = total_bytes / 2 / (1024 * 1024);
        if max_mb == 0 {
            return Err(Error::Invalid("physical RAM capacity"));
        }
        self.refresh_undo_memory_inventory()?;
        self.undo_ram_limit_mb = Some(max_mb);
        self.undo_policy.max_memory_mb = self.undo_policy.max_memory_mb.min(max_mb);
        self.trim_undo_memory();
        Ok(())
    }

    pub fn undo_memory_stats(&self) -> UndoMemoryStats {
        let current = self.capture_generation.load(Ordering::Acquire);
        let mut stats = self.undo_memory_stats;
        stats.capture_change_pending = current == u64::MAX || current != self.undo_capture_generation || self.undo_evidence_count != 0;
        stats
    }

    /// Explicit user action; current document and Redo remain intact.
    pub fn discard_oldest_undo(&mut self) -> Result<bool> {
        self.available()?;
        if self.scope.is_some() { return Err(Error::Busy); }
        if self.undo.is_empty() { return Ok(false); }
        let mut removed = self.undo.remove(0);
        self.rehome_history_evidence(&mut removed);
        if let Some(evidence) = removed.evidence.take() {
            let bytes = evidence.bytes.checked_add(evidence.source_upper).expect("Undo evidence upper bound");
            self.undo_evidence_upper = self.undo_evidence_upper.checked_sub(bytes).expect("Undo evidence byte count");
            self.undo_evidence_count = self.undo_evidence_count.checked_sub(1).expect("Undo evidence owner count");
            drop(evidence);
        }
        drop(removed);
        self.prune_overflow_evidence_unlocked();
        self.release_empty_history_evidence_capacity();
        // The old estimate is conservative until a fresh inventory can be made.
        self.undo_capture_generation = u64::MAX;
        Ok(true)
    }

    /// Application preference, not a document edit. Redo and source identities stay valid.
    pub fn set_undo_policy(&mut self, policy: UndoPolicy) -> Result<()> {
        self.available()?;
        if self.scope.is_some() {
            return Err(Error::Busy);
        }
        policy.validate()?;
        if self
            .undo_ram_limit_mb
            .is_some_and(|limit| policy.max_memory_mb > limit)
        {
            return Err(Error::Invalid("Undo budget exceeds 50% of physical RAM"));
        }
        self.refresh_undo_memory_inventory()?;
        self.undo_policy = policy;
        self.trim_undo_memory();
        Ok(())
    }

    /// Estimated owned journal bytes, not process RSS. Shared document Arcs and
    /// embedded resource payloads and immutable B-rep sources are counted once;
    /// owners also needed by live documents/scopes are excluded. Existing hard
    /// admission limits still apply.
    pub fn undo_memory_bytes(&self) -> Result<u64> {
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        self.undo_memory_bytes_with_reservations(&external, &reservations).map(|(bytes, _)| bytes)
    }

    fn undo_memory_bytes_with_reservations(&self, external: &Vec<Weak<Document>>, reservations: &ExternalReservations) -> Result<(u64, u64)> {
        self.undo_memory_bytes_with_guard(external, reservations, || {})
    }

    pub(super) fn undo_memory_bytes_with_guard(&self, external: &Vec<Weak<Document>>,
        reservations: &ExternalReservations, guard: impl FnOnce()) -> Result<(u64, u64)> {
        self.undo_memory_bytes_with_held_and_guard(external, reservations, Usage::default(), guard)
    }

    fn undo_memory_bytes_with_held_and_guard(&self, external: &Vec<Weak<Document>>,
        reservations: &ExternalReservations, held: Usage, guard: impl FnOnce()) -> Result<(u64, u64)> {
        let generation = reservations.capture_generation.load(Ordering::Acquire);
        if generation == u64::MAX { return Err(Error::Busy); }
        self.admit_undo_inventory_scratch(external, reservations, held)?;
        let mut documents = BTreeSet::new();
        let mut payloads = BTreeSet::new();
        let mut brep_sources = BTreeMap::new();
        let mut brep_topologies = BTreeMap::new();
        let mut native_payloads = BTreeMap::new();
        let mut evidence = BTreeMap::<usize, (&Arc<dyn std::any::Any + Send + Sync>, u64, usize, usize, bool)>::new();
        let mut history_sources = BTreeMap::<usize, (&Arc<CapturedBrepSourceInner>, usize, usize)>::new();
        for item in self.undo.iter().chain(&self.redo).filter_map(|entry| entry.evidence.as_ref())
            .chain(self.overflow_evidence.entries.iter()) {
            if let Some(source) = &item.source {
                let key = Arc::as_ptr(&source.0) as usize;
                let record = history_sources.entry(key).or_insert((&source.0, 0, Arc::strong_count(&source.0)));
                record.1 = record.1.checked_add(1).ok_or(Error::Overflow("history source owners"))?;
            }
        }
        let mut local = crate::model::BrepStorageIndex::default();
        for source in reservations.brep_source_leases.iter().filter_map(Weak::upgrade) {
            if let Some((_, history_count, initial)) = history_sources.get(&(Arc::as_ptr(&source) as usize)) {
                let count = Arc::strong_count(&source).checked_sub(1).ok_or(Error::Invalid("history source owners"))?;
                if count != *initial { return Err(Error::Busy); }
                if count < *history_count { return Err(Error::Invalid("history source owners")); }
                if count == *history_count { continue; }
            }
            match &source.source {
                CapturedBrepSourceData::Occt(source) => {
                    brep_sources.insert(source.source_key(), None);
                    brep_topologies.insert(source.topology_key(), None);
                }
                CapturedBrepSourceData::Native(source) => {
                    native_payloads.insert(source.allocation_key(), None);
                }
            }
        }
        for display in reservations.brep_display_leases.iter().filter_map(Weak::upgrade) {
            brep_topologies.insert(display.topology_key, None);
        }
        for document in self.saved_worlds().flat_map(|w| w.documents.values()) {
            documents.insert(Arc::as_ptr(document) as usize);
            for brep in retained_breps(document) {
                brep_sources.insert(brep.source_allocation_key(), None);
                brep_topologies.insert(brep.topology_allocation_key(), None);
            }
            for brep in retained_native_breps(document) {
                native_payloads.insert(brep.payload_allocation_key(), None);
            }
            for resource in document.resources.values() {
                if let Ok(bytes) = resource.payload() {
                    payloads.insert((bytes.as_ptr() as usize, bytes.len()));
                }
            }
        }
        // A tracked capture is a distinct live owner even when the same
        // document is no longer in the active world. Do not infer this from
        // the weak document registry: Undo may itself keep that Arc alive.
        for capture in reservations.capture_leases.iter().filter_map(Weak::upgrade) {
            let document = &capture.document;
            documents.insert(Arc::as_ptr(document) as usize);
            for brep in retained_breps(document) {
                brep_sources.insert(brep.source_allocation_key(), None);
                brep_topologies.insert(brep.topology_allocation_key(), None);
            }
            for brep in retained_native_breps(document) {
                native_payloads.insert(brep.payload_allocation_key(), None);
            }
            for resource in document.resources.values() {
                if let Ok(bytes) = resource.payload() {
                    payloads.insert((bytes.as_ptr() as usize, bytes.len()));
                }
            }
        }
        let mut total =
            Usage::array::<UndoEntry>(self.undo.capacity().saturating_add(self.redo.capacity()))?
                .bytes;
        total = total.checked_add(Usage::array::<UndoEvidence>(self.overflow_evidence.entries.capacity())?.bytes)
            .ok_or(Error::Overflow("Undo evidence storage"))?;
        let mut add = |bytes: u64| -> Result<()> {
            total = total
                .checked_add(bytes)
                .ok_or(Error::Overflow("Undo memory"))?;
            Ok(())
        };
        for entry in self.undo.iter().chain(&self.redo) {
            add(entry.storage_usage(&self.limits)?.bytes)?;
            if let Some(item) = &entry.evidence {
                let key = Arc::as_ptr(&item.owner) as *const () as usize;
                let record = evidence.entry(key).or_insert((&item.owner, item.bytes, 0, Arc::strong_count(&item.owner), false));
                if record.1 != item.bytes { return Err(Error::Invalid("history evidence accounting")); }
                record.2 = record.2.checked_add(1).ok_or(Error::Overflow("history evidence owners"))?;
                record.4 |= self.world.documents.get(&item.document).is_some_and(|doc| document_references_evidence(doc, item));
            }
            for document in entry.before.values().chain(entry.after.values()) {
                let document_key = Arc::as_ptr(document) as usize;
                if !documents.insert(document_key) {
                    continue;
                }
                let mut bytes = document.storage_usage_indexed(&self.limits, &mut local)?.bytes;
                // Document storage has already deduplicated aliases in one document.
                for brep in retained_native_breps(document) {
                    let key = brep.payload_allocation_key();
                    if retained_before_document(&mut native_payloads, key, document_key) {
                        bytes = bytes.checked_sub(brep.payload_allocation_bytes()?)
                            .ok_or(Error::Overflow("Undo shared native BRep payload accounting"))?;
                    }
                }
                for brep in retained_breps(document) {
                    let key = brep.source_allocation_key();
                    if retained_before_document(&mut brep_sources, key, document_key) {
                        bytes = bytes.checked_sub(brep.source_allocation_bytes()?)
                            .ok_or(Error::Overflow("Undo shared BRep source accounting"))?;
                    }
                }
                for brep in retained_breps(document) {
                    let key = brep.topology_allocation_key();
                    if retained_before_document(&mut brep_topologies, key, document_key) {
                        bytes = bytes.checked_sub(brep.topology_allocation_bytes()?)
                            .ok_or(Error::Overflow("Undo shared BRep topology accounting"))?;
                    }
                }
                for resource in document.resources.values() {
                    if let Ok(payload) = resource.payload()
                        && !payloads.insert((payload.as_ptr() as usize, payload.len()))
                    {
                        bytes = bytes
                            .checked_sub(payload.len() as u64)
                            .ok_or(Error::Overflow("Undo shared resource accounting"))?;
                    }
                }
                add(bytes)?;
            }
        }
        for item in &self.overflow_evidence.entries {
            let key = Arc::as_ptr(&item.owner) as *const () as usize;
            let record = evidence.entry(key).or_insert((&item.owner, item.bytes, 0, Arc::strong_count(&item.owner), false));
            if record.1 != item.bytes { return Err(Error::Invalid("history evidence accounting")); }
            record.2 = record.2.checked_add(1).ok_or(Error::Overflow("history evidence owners"))?;
            record.4 |= self.world.documents.get(&item.document).is_some_and(|doc| document_references_evidence(doc, item));
        }
        // A source-only history representation may own the lease without any
        // full BRep document retaining those allocations. Charge such source
        // and topology once unless a current/external owner already did so.
        for (lease, _, _) in history_sources.values() {
            if let CapturedBrepSourceData::Occt(source) = &lease.source {
                if !brep_sources.contains_key(&source.source_key()) {
                    add(source.source_bytes()?)?;
                    brep_sources.insert(source.source_key(), None);
                }
                if !brep_topologies.contains_key(&source.topology_key()) {
                    add(source.topology_bytes()?)?;
                    brep_topologies.insert(source.topology_key(), None);
                }
            }
        }
        for (owner, bytes, history_count, initial_count, current) in evidence.values() {
            let count = Arc::strong_count(owner);
            if count != *initial_count { return Err(Error::Busy); }
            if count < *history_count { return Err(Error::Invalid("history evidence owners")); }
            if !current && count == *history_count { add(*bytes)?; }
        }
        guard();
        for (owner, _, initial) in history_sources.values() {
            if Arc::strong_count(owner) != *initial { return Err(Error::Busy); }
        }
        if reservations.capture_generation.load(Ordering::Acquire) != generation {
            return Err(Error::Busy);
        }
        Ok((total, generation))
    }

    pub(super) fn refresh_undo_memory_inventory(&mut self) -> Result<()> {
        let external_registry = Arc::clone(&self.external_documents);
        let registry = Arc::clone(&self.external_reservations);
        let external = external_registry.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = registry.lock().unwrap_or_else(|e| e.into_inner());
        self.refresh_undo_memory_inventory_with_reservations(&external, &reservations, Usage::default())
    }

    pub(super) fn refresh_undo_memory_inventory_with_reservations(&mut self, external: &Vec<Weak<Document>>, reservations: &ExternalReservations, held: Usage) -> Result<()> {
        let profile=std::env::var_os("OM9_COMMIT_PROFILE").map(|_|std::time::Instant::now());
        let (bytes, generation) = match self.undo_memory_bytes_with_held_and_guard(external, reservations, held, || {}) {
            Ok(result) => result,
            Err(error) => {
                if let Some(start)=profile {eprintln!("OM9_COMMIT_PROFILE undo_inventory_ms={:.3} ok=false undo={} redo={} thread={:?}",start.elapsed().as_secs_f64()*1000.,self.undo.len(),self.redo.len(),std::thread::current().id());}
                // A failed refresh must be retried before a cached estimate can
                // drive soft-quota trimming, even if no capture generation changed.
                self.undo_capture_generation = u64::MAX;
                return Err(error);
            }
        };
        self.undo_memory_stats.estimated_bytes = bytes;
        self.undo_memory_stats.growth_since_inventory = 0;
        self.undo_memory_stats.full_inventories =
            self.undo_memory_stats.full_inventories.saturating_add(1);
        self.undo_capture_generation = generation;
        if let Some(start)=profile {eprintln!("OM9_COMMIT_PROFILE undo_inventory_ms={:.3} ok=true undo={} redo={} bytes={bytes} thread={:?}",start.elapsed().as_secs_f64()*1000.,self.undo.len(),self.redo.len(),std::thread::current().id());}
        Ok(())
    }

    /// Charge a conservative bound for the newly retained baseline only, rather
    /// than traversing the whole journal on every accepted command. The vector
    /// allowance dominates its capacity growth; full inventories deduplicate it.
    pub(super) fn note_undo_memory_growth(&mut self) {
        let upper = (|| -> Result<u64> {
            let Some(entry) = self.undo.last() else {
                return Ok(0);
            };
            let mut bytes = entry.storage_usage(&self.limits)?.bytes;
            for document in entry.before.values() {
                bytes = bytes.saturating_add(self.retained_storage_receipt(document, None, &self.limits,
                    || document.storage_usage(&self.limits))?.bytes);
            }
            bytes = bytes.saturating_add(
                Usage::array::<UndoEntry>(
                    self.undo.capacity().saturating_add(self.redo.capacity()),
                )?
                .bytes,
            );
            Ok(bytes)
        })()
        .unwrap_or(u64::MAX);
        self.undo_memory_stats.estimated_bytes =
            self.undo_memory_stats.estimated_bytes.saturating_add(upper);
        self.undo_memory_stats.growth_since_inventory = self
            .undo_memory_stats
            .growth_since_inventory
            .saturating_add(upper);
    }

    pub(super) fn trim_undo_memory(&mut self) {
        self.trim_undo_memory_with_held(Usage::default());
    }

    pub(super) fn trim_undo_memory_with_held(&mut self, held: Usage) {
        let mut deferred = match self.prepare_deferred_undo_evidence(self.undo_evidence_count, held) {
            Ok(value) => value,
            Err(_) => { self.undo_capture_generation = u64::MAX; return; }
        };
        let external_registry = Arc::clone(&self.external_documents);
        let registry = Arc::clone(&self.external_reservations);
        let external = external_registry.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = registry.lock().unwrap_or_else(|e| e.into_inner());
        self.trim_undo_memory_with_reservations(&external, &reservations, &mut deferred, held);
        drop(reservations);
        drop(external);
        drop(deferred);
        self.release_empty_history_evidence_capacity();
    }

    pub(super) fn prune_overflow_evidence(&mut self, deferred: &mut DeferredUndoEvidence) {
        let mut index = 0;
        while index < self.overflow_evidence.entries.len() {
            let evidence = &self.overflow_evidence.entries[index];
            if self.world.documents.get(&evidence.document).is_some_and(|doc| document_references_evidence(doc, evidence))
                || self.undo.iter().chain(&self.redo).any(|entry| entry_references_evidence(entry, evidence)) {
                index += 1;
            } else {
                let evidence = self.overflow_evidence.entries.swap_remove(index);
                deferred.take_item(evidence, &mut self.undo_evidence_upper, &mut self.undo_evidence_count);
                self.undo_capture_generation = u64::MAX;
            }
        }
    }

    /// Document close does not hold the shared account lock, so obsolete
    /// owners can be released directly without temporary deferred storage.
    pub(super) fn prune_overflow_evidence_unlocked(&mut self) {
        let mut index = 0;
        while index < self.overflow_evidence.entries.len() {
            let evidence = &self.overflow_evidence.entries[index];
            if self.world.documents.get(&evidence.document).is_some_and(|doc| document_references_evidence(doc, evidence))
                || self.undo.iter().chain(&self.redo).any(|entry| entry_references_evidence(entry, evidence)) {
                index += 1;
            } else {
                let evidence = self.overflow_evidence.entries.swap_remove(index);
                self.undo_evidence_upper -= evidence.bytes + evidence.source_upper;
                self.undo_evidence_count -= 1;
                drop(evidence);
            }
        }
    }

    pub(super) fn rehome_history_evidence(&mut self, removed: &mut UndoEntry) {
        let Some(evidence) = removed.evidence.as_ref() else { return; };
        if let Some(target) = self.undo.iter_mut().chain(&mut self.redo)
            .find(|entry| entry.evidence.is_none() && entry_references_evidence(entry, evidence)) {
            target.evidence = removed.evidence.take();
        } else if self.undo.iter().chain(&self.redo).any(|entry| entry_references_evidence(entry, evidence))
            || self.world.documents.get(&evidence.document).is_some_and(|doc| document_references_evidence(doc, evidence)) {
            assert!(self.overflow_evidence.entries.len() < self.overflow_evidence.entries.capacity(), "history evidence capacity");
            self.overflow_evidence.entries.push(removed.evidence.take().expect("Undo evidence"));
        }
    }

    pub(super) fn rehome_current_evidence(&mut self, removed: &mut UndoEntry) {
        let Some(evidence) = removed.evidence.as_ref() else { return; };
        if self.world.documents.get(&evidence.document).is_some_and(|doc| document_references_evidence(doc, evidence)) {
            assert!(self.overflow_evidence.entries.len() < self.overflow_evidence.entries.capacity(), "history evidence capacity");
            self.overflow_evidence.entries.push(removed.evidence.take().expect("Undo evidence"));
        }
    }

    pub(super) fn trim_undo_memory_with_reservations(&mut self, external: &Vec<Weak<Document>>, reservations: &ExternalReservations,
        deferred: &mut DeferredUndoEvidence, held: Usage) {
        let budget = self.undo_policy.max_memory_mb.saturating_mul(1024 * 1024);
        let generation = reservations.capture_generation.load(Ordering::Acquire);
        // Attribution changes can only move bytes between the live and history
        // subsets. A cached total for all immutable internal documents plus all
        // evidence bounds the latter without inspecting external readers. Keep
        // the 100 MiB cadence and refresh-before-trim behavior unchanged.
        if generation != u64::MAX && generation != self.undo_capture_generation
            && self.undo_memory_stats.growth_since_inventory < INVENTORY_INTERVAL_BYTES
            && self.undo_memory_stats.estimated_bytes.saturating_add(self.undo_evidence_upper) <= budget
        {
            let upper = (|| -> Result<Option<u64>> {
                let Some(bytes) = self.cached_journal_document_byte_upper()? else { return Ok(None); };
                let overflow = Usage::array::<UndoEvidence>(self.overflow_evidence.entries.capacity())?.bytes;
                Ok(bytes.checked_add(overflow).and_then(|bytes| bytes.checked_add(self.undo_evidence_upper)))
            })();
            if let Ok(Some(upper)) = upper && upper <= budget {
                #[cfg(debug_assertions)]
                if let Ok((exact, _)) = self.undo_memory_bytes_with_held_and_guard(external, reservations, held, || {}) {
                    debug_assert!(exact <= upper, "journal byte upper bound understated independent inventory: {exact} > {upper}");
                }
                self.undo_memory_stats.deferred_attribution_refreshes =
                    self.undo_memory_stats.deferred_attribution_refreshes.saturating_add(1);
                return;
            }
        }
        if (generation == u64::MAX || generation != self.undo_capture_generation
            || self.undo_memory_stats.growth_since_inventory >= INVENTORY_INTERVAL_BYTES
            || self.undo_memory_stats.estimated_bytes.saturating_add(self.undo_evidence_upper) > budget)
            && self.refresh_undo_memory_inventory_with_reservations(external, reservations, held).is_err()
        {
            return;
        }
        // Preserve the minimum even if one step alone exceeds the soft budget.
        // Do not evict anything when inventory fails; admission errors remain explicit.
        while self.undo.len() > self.undo_policy.minimum_steps {
            if self.undo_memory_stats.estimated_bytes > budget {
                let mut removed = self.undo.remove(0);
                self.rehome_history_evidence(&mut removed);
                deferred.take(&mut removed, &mut self.undo_evidence_upper, &mut self.undo_evidence_count);
                drop(removed);
                self.prune_overflow_evidence(deferred);
                self.undo.shrink_to_fit();
                if self.refresh_undo_memory_inventory_with_reservations(external, reservations, held).is_err() {
                    return;
                }
            } else {
                break;
            }
        }
    }
}
