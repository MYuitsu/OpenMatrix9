// SPDX-License-Identifier: LGPL-2.1-or-later
//! Reclaimable, account-owned in-process topology proofs.
use super::*;
use om9_core::{budget::{Usage, memory::MemoryClass}, identity::ObjectId,
    transaction::{ExternalMemoryReservation, OptionalMemoryCache}};
use om9_kernel::brep::SharedTopologyProof;
use std::sync::{Mutex, TryLockError};

struct Node {
    proof: Option<Arc<SharedTopologyProof>>,
    rebuild_pin: bool,
    rebuild_profile: Option<om9_core::brep::BrepRepresentationProfile>,
    last_used: u64,
    next: Option<Box<Node>>,
    reservation: ExternalMemoryReservation,
}
#[derive(Default)]
struct State { primary: Option<Arc<SharedTopologyProof>>, primary_rebuild_pin: bool,
    primary_rebuild_profile: Option<om9_core::brep::BrepRepresentationProfile>, primary_last_used: u64, clock: u64,
    current_document: std::sync::Weak<Document>, extras: Option<Box<Node>> }
impl State {
    fn tick(&mut self) -> u64 {
        if self.clock == u64::MAX {
            self.primary_last_used = 0;
            let mut cursor = self.extras.as_deref_mut();
            while let Some(node) = cursor { node.last_used = 0; cursor = node.next.as_deref_mut(); }
            self.clock = 0;
        }
        self.clock += 1;
        self.clock
    }
    fn find_matching(&mut self, matches: impl Fn(&SharedTopologyProof) -> bool) -> Option<Arc<SharedTopologyProof>> {
        let now = self.tick();
        if let Some(proof) = &self.primary {
            if matches(proof) { self.primary_last_used = now; return Some(proof.clone()); }
        }
        let mut cursor = self.extras.as_deref_mut();
        while let Some(node) = cursor {
            let proof = node.proof.as_ref().expect("live proof node");
            if matches(proof) { node.last_used = now; return Some(proof.clone()); }
            cursor = node.next.as_deref_mut();
        }
        None
    }
    fn source_last_used(&self, id: om9_core::brep::BrepSourceIdentity) -> u64 {
        let mut last = if self.primary.as_ref().is_some_and(|proof| proof.source_identity() == id) { self.primary_last_used } else { 0 };
        let mut cursor = self.extras.as_deref();
        while let Some(node) = cursor {
            if node.proof.as_ref().expect("live proof node").source_identity() == id { last = last.max(node.last_used); }
            cursor = node.next.as_deref();
        }
        last
    }
}
impl Drop for State {
    fn drop(&mut self) {
        let mut next = self.extras.take();
        while let Some(mut node) = next { next = node.next.take(); }
    }
}
pub(super) struct TopologyProofCache {
    state: Mutex<State>,
    history_users: std::sync::atomic::AtomicUsize,
    peer: TransactionCoordinator,
    // Last: proof entries and their account peer drop before this owner charge.
    _reservation: ExternalMemoryReservation,
}
impl Drop for TopologyProofCache {
    fn drop(&mut self) { self.peer.prune_optional_cache_index(); }
}
pub(super) struct HistoryProofCacheGuard(Arc<TopologyProofCache>);
impl Drop for HistoryProofCacheGuard {
    fn drop(&mut self) { self.0.history_users.fetch_sub(1, std::sync::atomic::Ordering::AcqRel); }
}
impl TopologyProofCache {
    pub(super) fn new(coordinator: &TransactionCoordinator, document: &Arc<Document>, held: Usage,
        physical: Option<(u64, u64)>) -> Result<Arc<Self>> {
        let bytes = (std::mem::size_of::<Self>() + std::mem::size_of::<Document>()
            + 4 * std::mem::size_of::<usize>()) as u64;
        let reservation = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
        let peer = coordinator.memory_admission_peer_with_held(held)?;
        let state = State { primary: None, primary_rebuild_pin: false, primary_rebuild_profile: None, primary_last_used: 0, clock: 0,
            current_document: Arc::downgrade(document), extras: None };
        let mut cache = Arc::new(Self { state: Mutex::new(state), history_users: std::sync::atomic::AtomicUsize::new(0), peer, _reservation: reservation });
        Arc::get_mut(&mut cache).expect("new topology cache")._reservation.set_allocated_bytes(bytes)?;
        let registered: Arc<dyn OptionalMemoryCache> = cache.clone();
        coordinator.register_optional_memory_cache(&registered, held)?;
        Ok(cache)
    }
    pub(super) fn pin_for_history(self: &Arc<Self>) -> Result<HistoryProofCacheGuard> {
        let _state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        self.history_users.fetch_update(std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire,
            |count| count.checked_add(1)).map_err(|_| Error::Overflow("history proof cache users"))?;
        Ok(HistoryProofCacheGuard(self.clone()))
    }
    pub(super) fn find(&self, object: Option<ObjectId>, document: &Document) -> Option<Arc<SharedTopologyProof>> {
        self.find_for_kind(object, document, false)
    }
    pub(super) fn find_source(&self, object: Option<ObjectId>, document: &Document) -> Option<Arc<SharedTopologyProof>> {
        self.find_for_kind(object, document, true)
    }
    pub(super) fn rebuild_profile(&self, source: om9_core::brep::BrepSourceIdentity)
        -> Option<om9_core::brep::BrepRepresentationProfile> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.primary_rebuild_pin && state.primary.as_ref().is_some_and(|proof| proof.source_identity() == source) {
            return state.primary_rebuild_profile;
        }
        let mut cursor = state.extras.as_deref();
        while let Some(node) = cursor {
            if node.rebuild_pin && node.proof.as_ref().is_some_and(|proof| proof.source_identity() == source) {
                return node.rebuild_profile;
            }
            cursor = node.next.as_deref();
        }
        None
    }
    /// A definition member may reuse an existing proof only for the exact
    /// immutable source and topology allocations attested by that proof.
    pub(super) fn find_shared_source(&self, document: &Document,
        source: &om9_core::brep::CanonicalBrep) -> Option<Arc<SharedTopologyProof>> {
        let matches = |proof: &SharedTopologyProof| {
            let stamp = proof.stamp();
            stamp.document == document.id && stamp.owner_session == document.owner_session
                && stamp.session_generation == document.session_generation
                && proof.shares_canonical_allocations(source)
        };
        self.state.lock().unwrap_or_else(|e| e.into_inner()).find_matching(matches)
    }
    fn find_for_kind(&self, object: Option<ObjectId>, document: &Document, source_only: bool) -> Option<Arc<SharedTopologyProof>> {
        let matches = |proof: &SharedTopologyProof| {
            let stamp = proof.stamp();
            (object.is_none() || object == Some(stamp.object))
                && document.stamp(stamp.object).is_ok_and(|now| now.document == stamp.document
                    && now.owner_session == stamp.owner_session && now.session_generation == stamp.session_generation
                    && now.generation == stamp.generation)
                && document.entities.get(&stamp.object).is_some_and(|entity|
                    matches!(&entity.geometry, Geometry::Brep(brep) if if source_only { proof.may_candidate_source(brep) } else { proof.may_candidate_source(brep) }))
        };
        self.state.lock().unwrap_or_else(|e| e.into_inner()).find_matching(matches)
    }
    pub(super) fn insert(&self, coordinator: &TransactionCoordinator, proof: Arc<SharedTopologyProof>, held: Usage,
        physical: Option<(u64, u64)>) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let now = state.tick();
        if state.primary.is_none() { state.primary_rebuild_pin = false; state.primary_rebuild_profile = None; }
        if state.primary.as_ref().is_none_or(|current| current.same_snapshot(&proof)
            && (!state.primary_rebuild_pin || current.same_source_allocations(&proof))) {
            let old = state.primary.replace(proof); state.primary_last_used = now;
            drop(state); drop(old); return Ok(());
        }
        let mut cursor = &mut state.extras;
        while let Some(node) = cursor {
            if node.proof.as_ref().expect("live proof node").same_snapshot(&proof)
                && (!node.rebuild_pin || node.proof.as_ref().expect("live proof node").same_source_allocations(&proof)) {
                let old = node.proof.replace(proof); node.last_used = now;
                drop(state); drop(old); return Ok(());
            }
            cursor = &mut node.next;
        }
        // Admission can reclaim this cache. Do not hold its lock while the
        // coordinator calls optional-cache owners under memory pressure.
        drop(state);
        let bytes = std::mem::size_of::<Node>() as u64;
        let reservation = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let now = state.tick();
        if state.primary.is_none() { state.primary_rebuild_pin = false; state.primary_rebuild_profile = None; }
        // Another insertion or reclamation may have changed the slots while
        // admission ran. Recheck before allocating or replacing an entry.
        if state.primary.as_ref().is_none_or(|current| current.same_snapshot(&proof)
            && (!state.primary_rebuild_pin || current.same_source_allocations(&proof))) {
            let old = state.primary.replace(proof); state.primary_last_used = now;
            drop(state); drop(old); return Ok(());
        }
        let mut cursor = &mut state.extras;
        while let Some(node) = cursor {
            if node.proof.as_ref().expect("live proof node").same_snapshot(&proof)
                && (!node.rebuild_pin || node.proof.as_ref().expect("live proof node").same_source_allocations(&proof)) {
                let old = node.proof.replace(proof); node.last_used = now;
                drop(state); drop(old); return Ok(());
            }
            cursor = &mut node.next;
        }
        reservation.recheck_pending()?;
        let mut node = Box::new(Node { proof: Some(proof), rebuild_pin: false, rebuild_profile: None, last_used: now, next: state.extras.take(), reservation });
        if let Err(error) = node.reservation.set_allocated_bytes(bytes) {
            state.extras = node.next.take();
            drop(state);
            drop(node);
            return Err(error);
        }
        state.extras = Some(node);
        Ok(())
    }
    pub(super) fn retain(&self, document: &Arc<Document>) {
        let eligible = |proof: &SharedTopologyProof| {
            let old = proof.stamp();
            if document.id != old.document || document.owner_session != old.owner_session
                || document.session_generation != old.session_generation { return false; }
            // An equivalent source may still be retained by a different
            // history document. Keep the optional proof until pressure can
            // reclaim it; lookup and use still check the current document.
            true
        };
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.current_document = Arc::downgrade(document);
        let retired_primary = if state.primary.as_ref().is_some_and(|proof| !eligible(proof)) {
            state.primary_rebuild_pin = false;
            state.primary_rebuild_profile = None;
            state.primary.take()
        } else { None };
        let mut retired_nodes = None;
        let mut cursor = &mut state.extras;
        while cursor.is_some() {
            if eligible(cursor.as_ref().expect("proof node").proof.as_ref().expect("live proof node")) {
                cursor = &mut cursor.as_mut().expect("proof node").next;
            } else {
                let mut node = cursor.take().expect("proof node");
                *cursor = node.next.take();
                node.next = retired_nodes.take();
                retired_nodes = Some(node);
            }
        }
        // A published current source with a representation no longer needs a
        // rebuild-only pin. Its proof remains an optional cache entry; if it is
        // reclaimed, the representation cannot be evicted without new proof.
        let store = self.peer.representation_store();
        if let Ok(store) = store.try_lock() {
            let is_current = |id| document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
                .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot))
                .any(|geometry| matches!(geometry, Geometry::Brep(source) if source.identity() == id));
            if state.primary_rebuild_pin && state.primary.as_ref().is_some_and(|proof|
                store.preferred_profile(proof.source_identity()).is_some() && is_current(proof.source_identity())) {
                state.primary_rebuild_pin = false;
                state.primary_rebuild_profile = None;
            }
            let mut cursor = state.extras.as_deref_mut();
            while let Some(node) = cursor {
                if node.rebuild_pin && node.proof.as_ref().is_some_and(|proof|
                    store.preferred_profile(proof.source_identity()).is_some() && is_current(proof.source_identity())) {
                    node.rebuild_pin = false;
                    node.rebuild_profile = None;
                }
                cursor = node.next.as_deref_mut();
            }
        }
        drop(state);
        drop(retired_primary);
        while let Some(mut node) = retired_nodes {
            retired_nodes = node.next.take();
            drop(node);
        }
    }
}
impl OptionalMemoryCache for TopologyProofCache {
    fn reclaim_optional_memory(&self, requested: u64) -> Result<u64> {
        if requested == 0 { return Ok(0); }
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Ok(0),
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
        };
        if self.history_users.load(std::sync::atomic::Ordering::Acquire) != 0 { return Ok(0); }
        // Keep the source-only proof while any document/history owner can
        // need it, and retire a separately charged display only when no scene
        // or worker pins that display. Do not hold either lock while dropping
        // the released reservation.
        let store = self.peer.representation_store();
        if let Ok(mut store) = store.try_lock() {
            let account = self.peer.external_memory_account();
            let current = state.current_document.upgrade();
            let is_current = |id: om9_core::brep::BrepSourceIdentity| {
                current.as_ref().is_none_or(|document| {
                    document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
                        .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot))
                        .any(|geometry| matches!(geometry, Geometry::Brep(source) if source.identity() == id))
                })
            };
            let mut oldest = None;
            let mut consider = |proof: &SharedTopologyProof| {
                let id = proof.source_identity();
                if proof.source_strong_count() > 1 && !is_current(id) && store.has_rebuildable_unpinned(id, &account) {
                    let used = state.source_last_used(id);
                    if oldest.is_none_or(|(previous, _)| used < previous) { oldest = Some((used, id)); }
                }
            };
            if let Some(proof) = &state.primary { consider(proof); }
            let mut cursor = state.extras.as_deref();
            while let Some(node) = cursor {
                consider(node.proof.as_ref().expect("live proof node"));
                cursor = node.next.as_deref();
            }
            let retired = oldest.and_then(|(_, id)| {
                let profile = store.preferred_profile(id);
                store.take_rebuildable_unpinned(id, &account).map(|(entry, bytes)| (entry, bytes, id, profile))
            });
            drop(store);
            if let Some((representation, bytes, id, profile)) = retired {
                let primary_matches = state.primary.as_ref().is_some_and(|proof| proof.source_identity() == id);
                let mut pinned = primary_matches && state.primary_rebuild_pin;
                if pinned && state.primary_rebuild_profile.is_none() { state.primary_rebuild_profile = profile; }
                let mut cursor = state.extras.as_deref_mut();
                while let Some(node) = cursor {
                    if node.rebuild_pin && node.proof.as_ref().expect("live proof node").source_identity() == id {
                        if node.rebuild_profile.is_none() { node.rebuild_profile = profile; }
                        pinned = true;
                    }
                    cursor = node.next.as_deref_mut();
                }
                if !pinned {
                    if primary_matches { state.primary_rebuild_pin = true; state.primary_rebuild_profile = profile; }
                    else {
                        let mut cursor = state.extras.as_deref_mut();
                        while let Some(node) = cursor {
                            if node.proof.as_ref().expect("live proof node").source_identity() == id {
                                node.rebuild_pin = true;
                                node.rebuild_profile = profile;
                                break;
                            }
                            cursor = node.next.as_deref_mut();
                        }
                    }
                }
                drop(state);
                drop(representation);
                return Ok(bytes);
            }
        }
        if state.primary.as_ref().is_some_and(|proof| Arc::strong_count(proof) == 1
            && (!state.primary_rebuild_pin || proof.source_strong_count() == 1)) {
            let proof = state.primary.take().expect("primary proof");
            match Arc::try_unwrap(proof) {
                Ok(proof) => {
                    state.primary_rebuild_pin = false;
                    state.primary_rebuild_profile = None;
                    drop(state);
                    let bytes = proof.reserved_bytes()?;
                    drop(proof);
                    return Ok(bytes);
                }
                Err(proof) => state.primary = Some(proof),
            }
        }
        let mut cursor = &mut state.extras;
        while cursor.is_some() {
            if cursor.as_ref().is_some_and(|node| node.proof.as_ref().is_some_and(|proof|
                Arc::strong_count(proof) == 1 && (!node.rebuild_pin || proof.source_strong_count() == 1))) {
                let mut node = cursor.take().expect("proof node");
                let proof = node.proof.take().expect("live proof node");
                match Arc::try_unwrap(proof) {
                    Ok(proof) => {
                        *cursor = node.next.take();
                        drop(state);
                        let bytes = proof.reserved_bytes()?.checked_add(node.reservation.bytes()).ok_or(Error::Overflow("topology cache reclaimed"))?;
                        drop(proof);
                        drop(node);
                        return Ok(bytes);
                    }
                    Err(proof) => { node.proof = Some(proof); *cursor = Some(node); }
                }
            }
            cursor = &mut cursor.as_mut().expect("proof node").next;
        }
        Ok(0)
    }
}
