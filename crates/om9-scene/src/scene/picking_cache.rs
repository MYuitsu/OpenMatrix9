// SPDX-License-Identifier: LGPL-2.1-or-later
//! Reclaimable picking data. A reader pins the data and its allowance together.
use super::*;
use crate::spatial::SpatialIndex;
use om9_core::transaction::{ExternalMemoryReservation, OptionalMemoryCache};
use std::{ops::Deref, sync::Mutex};

struct PickingCache {
    spheres: Vec<([f64; 3], f64)>,
    index: OnceLock<SpatialIndex>,
    spare_index: Mutex<Option<SpatialIndex>>,
    // Last: storage dies before the allowance.
    reservation: Option<ExternalMemoryReservation>,
}
impl PickingCache {
    fn allocate(objects: usize, reservation: Option<ExternalMemoryReservation>) -> Result<Self> {
        Ok(Self { spheres: display_vector(objects, "picking spheres allocation")?, index: OnceLock::new(),
            spare_index: Mutex::new(Some(SpatialIndex::try_with_capacity(objects)?)), reservation })
    }
    fn index(&self) -> Result<&SpatialIndex> {
        if let Some(index) = self.index.get() { return Ok(index); }
        let mut spare = self.spare_index.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(index) = self.index.get() { return Ok(index); }
        let index = spare.as_mut().ok_or(Error::Missing("picking arena"))?;
        index.rebuild_from_spheres(&self.spheres)?;
        let _ = self.index.set(spare.take().expect("picking arena exists"));
        Ok(self.index.get().expect("picking index published"))
    }
    fn reset(&mut self) {
        if let Some(index) = self.index.take() { *self.spare_index.get_mut().unwrap_or_else(|e| e.into_inner()) = Some(index); }
        self.spheres.clear();
    }
    fn owner_bytes() -> u64 { (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64 }
    fn storage(&self) -> Result<u64> {
        let spare = self.spare_index.lock().unwrap_or_else(|e| e.into_inner());
        let index = self.index.get().or(spare.as_ref()).ok_or(Error::Missing("picking arena"))?;
        Usage::array::<([f64; 3], f64)>(self.spheres.capacity())?.bytes.checked_add(index.storage_usage()?.bytes)
            .and_then(|n| n.checked_add(Self::owner_bytes())).ok_or(Error::Overflow("picking storage"))
    }
}

#[derive(Clone)]
pub struct ObjectSpheres(Arc<PickingCache>);
impl std::fmt::Debug for ObjectSpheres {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("ObjectSpheres").field("objects", &self.0.spheres.len()).finish() }
}
impl Deref for ObjectSpheres {
    type Target = [([f64; 3], f64)];
    fn deref(&self) -> &Self::Target { &self.0.spheres }
}
#[derive(Clone)]
pub struct PickingIndex(Arc<PickingCache>);
impl std::fmt::Debug for PickingIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("PickingIndex").field("objects", &self.0.spheres.len()).finish() }
}
impl Deref for PickingIndex {
    type Target = SpatialIndex;
    fn deref(&self) -> &Self::Target { self.0.index.get().expect("built picking index") }
}

#[derive(Default)]
pub(super) struct SceneMemory {
    cache: Option<Arc<PickingCache>>,
    valid: bool,
    peer: Option<TransactionCoordinator>,
    metadata_bytes: u64,
    pub(super) reservation: Option<ExternalMemoryReservation>,
    managed: bool,
}
#[derive(Default)]
pub(super) struct SceneMemoryOwner(Mutex<SceneMemory>);
impl Deref for SceneMemoryOwner {
    type Target = Mutex<SceneMemory>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl Drop for SceneMemoryOwner {
    fn drop(&mut self) {
        // Weak index entries otherwise keep this Arc allocation after its
        // metadata lease is returned. Data/lease fields still exist here.
        let state = self.0.get_mut().unwrap_or_else(|e| e.into_inner());
        if let Some(peer) = state.peer.as_ref() { peer.prune_optional_cache_index(); }
    }
}
impl SceneMemory {
    fn unregistered_cache_bytes(&self) -> Result<u64> {
        if let Some(cache) = self.cache.as_ref().filter(|cache| cache.reservation.is_none()) {
            if Arc::strong_count(cache) != 1 { return Err(Error::Busy); }
            return cache.storage();
        }
        Ok(0)
    }
    fn bind_existing_cache(&mut self, arena: u64) -> Result<()> {
        if let Some(cache) = self.cache.as_mut().filter(|cache| cache.reservation.is_none()) {
            let cache = Arc::get_mut(cache).ok_or(Error::Busy)?;
            let mut reservation = self.reservation.as_mut().ok_or(Error::Missing("cache metadata owner"))?.split_off(arena)?;
            reservation.set_allocated_bytes(cache.storage()?)?;
            cache.reservation = Some(reservation);
        }
        Ok(())
    }
}
impl SceneMemoryOwner {
    fn reclaim(&self, unused_only: bool) -> Result<u64> {
        let Ok(mut state) = self.try_lock() else { return Ok(0); };
        let mut released = 0u64;
        if !unused_only && state.cache.as_ref().is_some_and(|cache| Arc::strong_count(cache) == 1) {
            released = state.cache.as_ref().and_then(|c| c.reservation.as_ref()).map_or(0, ExternalMemoryReservation::bytes);
            drop(state.cache.take());
            state.valid = false;
        }
        let metadata = state.metadata_bytes;
        if let Some(reservation) = state.reservation.as_mut() {
            let pending = reservation.bytes().checked_sub(metadata).ok_or(Error::Invalid("cache metadata charge"))?;
            reservation.shrink_to(metadata)?;
            released = released.checked_add(pending).ok_or(Error::Overflow("optional cache released"))?;
        }
        Ok(released)
    }
}
impl OptionalMemoryCache for SceneMemoryOwner {
    fn reclaim_unused_memory(&self) -> Result<u64> { self.reclaim(true) }
    fn reclaim_optional_memory(&self, requested: u64) -> Result<u64> {
        if requested == 0 { return Ok(0); }
        self.reclaim(false)
    }
}

impl Scene {
    pub fn picking_arena_bytes(&self) -> Result<u64> {
        SpatialIndex::allocation_usage(self.objects.len())?.bytes
            .checked_add(Usage::array::<([f64; 3], f64)>(self.objects.len())?.bytes)
            .and_then(|n| n.checked_add(PickingCache::owner_bytes())).ok_or(Error::Overflow("picking arena"))
    }
    pub fn managed_memory_allowance(&self) -> Result<u64> {
        self.metadata_storage_usage()?.bytes.checked_add(self.picking_arena_bytes()?).ok_or(Error::Overflow("scene memory allowance"))
    }
    pub fn transfer_memory_allowance(&self, construction: &mut ExternalMemoryReservation,
        coordinator: &TransactionCoordinator, held: Usage) -> Result<()> {
        let peer = coordinator.memory_admission_peer()?;
        let mut state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if state.reservation.is_some() { return Err(Error::Invalid("scene memory already registered")); }
        let metadata = self.metadata_storage_usage()?.bytes;
        let existing = state.unregistered_cache_bytes()?;
        let mut reservation = construction.split_off(self.managed_memory_allowance()?)?;
        reservation.set_allocated_bytes(metadata.checked_add(existing).ok_or(Error::Overflow("existing cache allocation"))?)?;
        state.reservation = Some(reservation);
        state.metadata_bytes = metadata;
        state.peer = Some(peer);
        state.managed = true;
        state.bind_existing_cache(self.picking_arena_bytes()?)?;
        drop(state);
        let owner: Arc<dyn OptionalMemoryCache> = self.memory.clone();
        coordinator.register_optional_memory_cache(&owner, held)
    }
    pub fn reserve_memory_allowance(&self, coordinator: &TransactionCoordinator, held: Usage,
        physical: Option<(u64, u64)>) -> Result<()> {
        let peer = coordinator.memory_admission_peer()?;
        let mut state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        let metadata = self.metadata_storage_usage()?.bytes;
        let existing = state.unregistered_cache_bytes()?;
        let allocated = metadata.checked_add(existing).ok_or(Error::Overflow("existing cache allocation"))?;
        let resident = state.cache.as_ref().and_then(|c| c.reservation.as_ref()).map_or(0, ExternalMemoryReservation::bytes);
        let quota = self.managed_memory_allowance()?.checked_sub(resident).ok_or(Error::Invalid("cache allowance"))?;
        if let Some(reservation) = state.reservation.as_mut() {
            if quota > reservation.bytes() {
                let additional = quota - reservation.bytes();
                coordinator.grow_external_memory_with_held(reservation, additional, physical, held)?;
            }
            reservation.set_allocated_bytes(allocated)?;
        } else {
            state.reservation = Some(coordinator.reserve_external_memory_with_allocated(
                om9_core::budget::memory::MemoryClass::CpuCache, quota, allocated, physical, held)?);
        }
        state.peer = Some(peer);
        state.metadata_bytes = metadata;
        state.managed = true;
        state.bind_existing_cache(self.picking_arena_bytes()?)?;
        drop(state);
        let owner: Arc<dyn OptionalMemoryCache> = self.memory.clone();
        coordinator.register_optional_memory_cache(&owner, held)
    }
    pub(super) fn memory_usage(&self, metadata: u64) -> Result<(u64, u64)> {
        let state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        let picking = state.cache.as_ref().map_or(Ok(0), |c| c.storage())?;
        let mut registered = state.reservation.as_ref().map_or(0, |r| r.bytes() - r.pending_bytes());
        if let Some(reservation) = state.cache.as_ref().and_then(|c| c.reservation.as_ref()) {
            registered = registered.checked_add(reservation.bytes() - reservation.pending_bytes()).ok_or(Error::Overflow("cache ownership"))?;
        }
        Ok((picking, registered.min(metadata.checked_add(picking).ok_or(Error::Overflow("scene memory"))?)))
    }
    pub fn registered_memory_bytes(&self) -> Result<u64> { self.memory_usage(self.metadata_storage_usage()?.bytes).map(|(_, n)| n) }
    pub fn memory_allowance_bytes(&self) -> u64 {
        let state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        state.reservation.as_ref().map_or(0, ExternalMemoryReservation::bytes)
            + state.cache.as_ref().and_then(|c| c.reservation.as_ref()).map_or(0, ExternalMemoryReservation::bytes)
    }
    fn picking_cache(&self) -> Result<Arc<PickingCache>> {
        let mut state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if state.valid {
            self.touch_picking_cache(&state)?;
            return state.cache.as_ref().cloned().ok_or(Error::Missing("picking cache"));
        }
        if state.cache.as_ref().is_some_and(|c| Arc::strong_count(c) > 1) { drop(state.cache.take()); }
        if state.cache.is_none() {
            let arena = self.picking_arena_bytes()?;
            let metadata = self.metadata_storage_usage()?.bytes;
            let reservation = if state.managed {
                let SceneMemory { peer, reservation, .. } = &mut *state;
                let peer = peer.as_ref().ok_or(Error::Missing("cache admission peer"))?;
                let reservation = reservation.as_mut().ok_or(Error::Missing("scene metadata reservation"))?;
                let required = metadata.checked_add(arena).ok_or(Error::Overflow("picking admission"))?;
                if required > reservation.bytes() {
                    let additional = required - reservation.bytes();
                    peer.grow_external_memory_with_held(reservation, additional, peer.physical_memory_sample()?, Usage::default())?;
                }
                peer.recheck_external_memory_with_reclamation(reservation)?;
                Some(reservation.split_off(arena)?)
            } else {
                Usage { bytes: metadata.checked_add(arena).and_then(|n| n.checked_add(self.shared_buffer_bytes()))
                    .ok_or(Error::Overflow("picking local allowance"))?, ..Usage::default() }.check(&self.limits)?;
                None
            };
            let mut cache = PickingCache::allocate(self.objects.len(), reservation)?;
            let actual = cache.storage()?;
            if let Some(reservation) = cache.reservation.as_mut() {
                if actual > reservation.bytes() { return Err(Error::Budget("picking actual capacity")); }
                reservation.set_allocated_bytes(actual)?;
            }
            state.cache = Some(Arc::new(cache));
        }
        let cache = Arc::get_mut(state.cache.as_mut().expect("picking arena exists")).ok_or(Error::Busy)?;
        if cache.spheres.capacity() < self.objects.len() { return Err(Error::Budget("picking spheres capacity")); }
        cache.spheres.clear();
        let mut last: Option<(&Arc<RenderBuffers>, Option<([f64; 3], [f64; 3])>)> = None;
        for object in &self.objects {
            let local = match &last {
                Some((buffers, bounds)) if Arc::ptr_eq(buffers, &object.buffers) => *bounds,
                _ => { let bounds = object.local_bounds(); last = Some((&object.buffers, bounds)); bounds }
            };
            cache.spheres.push(object.sphere_bounds_from_local(local).unwrap_or(([0.; 3], 0.)));
        }
        state.valid = true;
        self.touch_picking_cache(&state)?;
        Ok(state.cache.as_ref().expect("picking cache published").clone())
    }
    fn touch_picking_cache(&self, state: &SceneMemory) -> Result<()> {
        if let Some(peer) = state.peer.as_ref() {
            let owner: Arc<dyn OptionalMemoryCache> = self.memory.clone();
            peer.touch_optional_memory_cache(&owner)?;
        }
        Ok(())
    }
    pub fn spatial_index(&self) -> Result<PickingIndex> { let cache = self.picking_cache()?; cache.index()?; Ok(PickingIndex(cache)) }
    pub fn object_spheres(&self) -> Result<ObjectSpheres> { self.picking_cache().map(ObjectSpheres) }
    /// Optional render bounds: no allocation, admission, reclamation or recency
    /// promotion. A cold/busy/invalid cache means conservative unculled drawing.
    pub fn cached_object_spheres(&self) -> Option<ObjectSpheres> {
        let state = self.memory.try_lock().ok()?;
        if !state.valid { return None; }
        state.cache.as_ref().cloned().map(ObjectSpheres)
    }
    pub fn invalidate_picking_bounds(&mut self) {
        let mut state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(cache) = state.cache.as_mut().and_then(Arc::get_mut) { cache.reset(); }
        state.valid = false;
    }
    /// Existing read handles retain data/allowance; a subsequent read admits
    /// its replacement through the shared account before allocation.
    pub fn evict_picking_cache(&mut self) -> Result<()> {
        let mut state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        drop(state.cache.take());
        state.valid = false;
        let metadata = state.metadata_bytes;
        if let Some(reservation) = state.reservation.as_mut() { reservation.shrink_to(metadata)?; }
        Ok(())
    }
}
