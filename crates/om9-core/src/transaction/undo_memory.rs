// SPDX-License-Identifier: LGPL-2.1-or-later
//! Soft journal retention policy, independent of document/recipe limits.
use super::*;
use serde::{Deserialize, Serialize};

pub const INVENTORY_INTERVAL_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UndoMemoryStats {
    pub estimated_bytes: u64,
    pub growth_since_inventory: u64,
    pub full_inventories: u64,
    /// A tracked capture was created or finally dropped since this inventory.
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
        stats.capture_change_pending = current == u64::MAX || current != self.undo_capture_generation;
        stats
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
        let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        self.undo_memory_bytes_with_reservations(&reservations).map(|(bytes, _)| bytes)
    }

    fn undo_memory_bytes_with_reservations(&self, reservations: &ExternalReservations) -> Result<(u64, u64)> {
        self.undo_memory_bytes_with_guard(reservations, || {})
    }

    pub(super) fn undo_memory_bytes_with_guard(&self, reservations: &ExternalReservations, guard: impl FnOnce()) -> Result<(u64, u64)> {
        let generation = reservations.capture_generation.load(Ordering::Acquire);
        if generation == u64::MAX { return Err(Error::Busy); }
        let mut documents = BTreeSet::new();
        let mut payloads = BTreeSet::new();
        let mut brep_sources = BTreeSet::new();
        let mut brep_displays = BTreeSet::new();
        let mut native_payloads = BTreeSet::new();
        for document in self.saved_worlds().flat_map(|w| w.documents.values()) {
            documents.insert(Arc::as_ptr(document) as usize);
            for brep in retained_breps(document) {
                brep_sources.insert(brep.source_allocation_key());
                brep_displays.insert(brep.display_allocation_key());
            }
            for brep in retained_native_breps(document) {
                native_payloads.insert(brep.payload_allocation_key());
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
                brep_sources.insert(brep.source_allocation_key());
                brep_displays.insert(brep.display_allocation_key());
            }
            for brep in retained_native_breps(document) {
                native_payloads.insert(brep.payload_allocation_key());
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
        let mut add = |bytes: u64| -> Result<()> {
            total = total
                .checked_add(bytes)
                .ok_or(Error::Overflow("Undo memory"))?;
            Ok(())
        };
        for entry in self.undo.iter().chain(&self.redo) {
            add(entry.storage_usage(&self.limits)?.bytes)?;
            for document in entry.before.values().chain(entry.after.values()) {
                if !documents.insert(Arc::as_ptr(document) as usize) {
                    continue;
                }
                let mut bytes = document.storage_usage(&self.limits)?.bytes;
                // Document storage has already deduplicated aliases in one document.
                for (index, brep) in retained_native_breps(document).enumerate() {
                    let key = brep.payload_allocation_key();
                    if retained_native_breps(document).take(index).any(|prior| prior.payload_allocation_key() == key) {
                        continue;
                    }
                    if !native_payloads.insert(key) {
                        bytes = bytes.checked_sub(brep.payload_allocation_bytes()?)
                            .ok_or(Error::Overflow("Undo shared native BRep payload accounting"))?;
                    }
                }
                for (index, brep) in retained_breps(document).enumerate() {
                    let key = brep.source_allocation_key();
                    // Document storage already deduplicates aliases within the
                    // same document; subtract at most once per source here.
                    if retained_breps(document).take(index).any(|prior| prior.source_allocation_key() == key) {
                        continue;
                    }
                    if !brep_sources.insert(key) {
                        bytes = bytes.checked_sub(brep.source_allocation_bytes()?)
                            .ok_or(Error::Overflow("Undo shared BRep source accounting"))?;
                    }
                }
                for (index, brep) in retained_breps(document).enumerate() {
                    let key = brep.display_allocation_key();
                    if retained_breps(document).take(index).any(|prior| prior.display_allocation_key() == key) { continue; }
                    if !brep_displays.insert(key) {
                        bytes = bytes.checked_sub(brep.display_allocation_bytes()?)
                            .ok_or(Error::Overflow("Undo shared BRep display accounting"))?;
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
        guard();
        if reservations.capture_generation.load(Ordering::Acquire) != generation {
            return Err(Error::Busy);
        }
        Ok((total, generation))
    }

    pub(super) fn refresh_undo_memory_inventory(&mut self) -> Result<()> {
        let registry = Arc::clone(&self.external_reservations);
        let reservations = registry.lock().unwrap_or_else(|e| e.into_inner());
        self.refresh_undo_memory_inventory_with_reservations(&reservations)
    }

    pub(super) fn refresh_undo_memory_inventory_with_reservations(&mut self, reservations: &ExternalReservations) -> Result<()> {
        let (bytes, generation) = self.undo_memory_bytes_with_reservations(reservations)?;
        self.undo_memory_stats.estimated_bytes = bytes;
        self.undo_memory_stats.growth_since_inventory = 0;
        self.undo_memory_stats.full_inventories =
            self.undo_memory_stats.full_inventories.saturating_add(1);
        self.undo_capture_generation = generation;
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
                bytes = bytes.saturating_add(document.storage_usage(&self.limits)?.bytes);
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
        let registry = Arc::clone(&self.external_reservations);
        let reservations = registry.lock().unwrap_or_else(|e| e.into_inner());
        self.trim_undo_memory_with_reservations(&reservations);
    }

    pub(super) fn trim_undo_memory_with_reservations(&mut self, reservations: &ExternalReservations) {
        let budget = self.undo_policy.max_memory_mb.saturating_mul(1024 * 1024);
        let generation = reservations.capture_generation.load(Ordering::Acquire);
        if (generation == u64::MAX || generation != self.undo_capture_generation
            || self.undo_memory_stats.growth_since_inventory >= INVENTORY_INTERVAL_BYTES
            || self.undo_memory_stats.estimated_bytes > budget)
            && self.refresh_undo_memory_inventory_with_reservations(reservations).is_err()
        {
            return;
        }
        // Preserve the minimum even if one step alone exceeds the soft budget.
        // Do not evict anything when inventory fails; admission errors remain explicit.
        while self.undo.len() > self.undo_policy.minimum_steps {
            if self.undo_memory_stats.estimated_bytes > budget {
                self.undo.remove(0);
                self.undo.shrink_to_fit();
                if self.refresh_undo_memory_inventory_with_reservations(reservations).is_err() {
                    return;
                }
            } else {
                break;
            }
        }
    }
}
