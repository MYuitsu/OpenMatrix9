// SPDX-License-Identifier: LGPL-2.1-or-later
//! Live ownership of root selection storage; source captures own separate IDs.
use super::Controller;
use om9_core::{Error, Result, budget::{Usage, memory::MemoryClass}, identity::ObjectId,
    transaction::ExternalMemoryReservation};
use om9_scene::scene::Scene;
use std::collections::BTreeSet;

// Data must drop before its reservation on every rejected candidate.
pub(super) struct ReservedRootSelection {
    selection: BTreeSet<ObjectId>,
    order: Vec<ObjectId>,
    reservation: ExternalMemoryReservation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{budget::Limits, geometry::Geometry, model::{Document, Entity}};

    fn fixture() -> (Controller, Vec<ObjectId>) {
        let mut document = Document::with_template_tolerance("root selection memory");
        let mut ids = Vec::new();
        for index in 0..8 {
            let entity = Entity::new("point", Geometry::Point([index as f64, 0., 0.]), None);
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        (Controller::new(document, Limits::default()).unwrap(), ids)
    }

    #[test]
    fn root_selection_keeps_order_revisions_and_live_capacity_through_gestures() {
        let (mut controller, ids) = fixture();
        controller.select_root(Some(ids[3]), false).unwrap();
        let revision = controller.selection_revision;
        controller.select_root(Some(ids[3]), false).unwrap();
        assert_eq!(controller.selection_revision, revision + 1);
        controller.add_objects(&[ids[0], ids[0]]).unwrap();
        controller.select_root(Some(ids[2]), true).unwrap();
        assert_eq!(controller.selected_ordered(), vec![ids[3], ids[0], ids[2]]);
        assert_eq!(controller.selection_reservation.as_ref().unwrap().bytes(),
            storage_bytes(3, controller.selection_order.capacity()).unwrap());
        controller.toggle_selection(ids[0]).unwrap();
        controller.remove_objects(&[ids[2]]).unwrap();
        controller.toggle_selection(ids[1]).unwrap();
        assert_eq!(controller.selected_ordered(), vec![ids[3], ids[1]]);
        let revision = controller.selection_revision;
        controller.select_objects(&[ids[3], ids[1]]).unwrap();
        assert_eq!(controller.selection_revision, revision);
        controller.clear_selection();
        assert!(controller.selected().is_empty());
        let retained_order = Usage::array::<ObjectId>(controller.selection_order.capacity()).unwrap().bytes;
        assert_eq!(controller.selection_reservation.as_ref().unwrap().bytes(), retained_order);
        assert_eq!(controller.selection_reservation.as_ref().unwrap().pending_bytes(), 0);
        assert_eq!(controller.coordinator.undo_len(), 0);
        assert_eq!(controller.document().entities.len(), 8);
    }

    #[test]
    fn refused_replacement_keeps_selection_history_and_its_existing_owner() {
        let (mut controller, ids) = fixture();
        controller.select_objects(&[ids[0]]).unwrap();
        controller.submit("Point").unwrap(); controller.submit("20,0,0").unwrap(); controller.undo().unwrap();
        controller.select_objects(&[ids[0]]).unwrap();
        let spheres = controller.scene().unwrap().object_spheres().unwrap();
        let index = controller.scene().unwrap().spatial_index().unwrap();
        let before = controller.coordinator.snapshot();
        let revision = controller.selection_revision;
        let owner = controller.selection_reservation.as_ref().unwrap().bytes();
        let pressure = controller.coordinator.reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(matches!(controller.select_objects(&ids), Err(Error::Budget(_))));
        assert_eq!(controller.selected_ordered(), vec![ids[0]]);
        assert_eq!(controller.selection_revision, revision);
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.selection_reservation.as_ref().unwrap().bytes(), owner);
        assert_eq!(controller.selection_reservation.as_ref().unwrap().pending_bytes(), 0);
        drop(pressure);
        controller.select_objects(&ids).unwrap();
        assert_eq!(controller.selected_ordered(), ids);
        assert_eq!(controller.coordinator.redo_len(), 1);
        drop((spheres, index));
    }
}

fn storage_bytes(count: usize, capacity: usize) -> Result<u64> {
    // All empty sets published here are reset, so no empty root is retained.
    let tree = if count == 0 { 0 } else { Usage::btree::<ObjectId, ()>(count)?.bytes };
    tree.checked_add(Usage::array::<ObjectId>(capacity)?.bytes)
        .ok_or(Error::Overflow("selection storage"))
}

impl Controller {
    pub(super) fn root_selection_usage(&self) -> Result<Usage> {
        Ok(Usage { bytes: storage_bytes(self.selection.len(), self.selection_order.capacity())?, ..Usage::default() })
    }

    pub(super) fn reserve_root_selection(&self, count: usize, capacity: usize) -> Result<ExternalMemoryReservation> {
        self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            storage_bytes(count, capacity)?.max(1),
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            self.scene_cache_usage()?)
    }

    fn fit_selection_capacity(&self, lease: &mut ExternalMemoryReservation, count: usize, capacity: usize) -> Result<()> {
        let bytes = storage_bytes(count, capacity)?;
        if bytes > lease.bytes() {
            let additional = bytes - lease.bytes();
            self.coordinator.grow_external_memory_with_held(lease, additional,
                om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), self.scene_cache_usage()?)?;
        }
        Ok(())
    }

    pub(super) fn prepare_root_selection(&self, scene: &Scene, ids: &[ObjectId], sorted: bool) -> Result<ReservedRootSelection> {
        if ids.len() as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("root selection count"));
        }
        // Validate first without constructing an unadmitted BTreeSet.
        for id in ids {
            if id.is_nil() || !scene.objects.iter().any(|o| o.root == *id && o.selectable && o.has_pick_geometry()) {
                return Err(Error::Permission("selected root"));
            }
        }
        let reservation = self.reserve_root_selection(ids.len(), ids.len())?;
        let mut next = ReservedRootSelection { selection: BTreeSet::new(), order: Vec::new(), reservation };
        next.order.try_reserve_exact(ids.len()).map_err(|_| Error::Budget("selection order"))?;
        self.fit_selection_capacity(&mut next.reservation, ids.len(), next.order.capacity())?;
        for id in ids {
            if next.selection.insert(*id) { next.order.push(*id); }
        }
        if sorted { next.order.sort_unstable(); }
        let bytes = storage_bytes(next.selection.len(), next.order.capacity())?;
        next.reservation.set_allocated_bytes(bytes)?;
        next.reservation.shrink_to(bytes)?;
        Ok(next)
    }

    pub(super) fn publish_root_selection(&mut self, next: ReservedRootSelection) {
        let ReservedRootSelection { selection, order, reservation } = next;
        if selection != self.selection || order != self.selection_order {
            self.selection_revision = self.selection_revision.saturating_add(1);
        }
        let old_selection = std::mem::replace(&mut self.selection, selection);
        let old_order = std::mem::replace(&mut self.selection_order, order);
        let old_reservation = self.selection_reservation.replace(reservation);
        drop(old_selection);
        drop(old_order);
        drop(old_reservation);
    }

    pub(super) fn prepare_root_selection_ordered(&self, order: Vec<ObjectId>, reservation: ExternalMemoryReservation)
        -> Result<ReservedRootSelection> {
        let mut next = ReservedRootSelection { selection: BTreeSet::new(), order, reservation };
        self.fit_selection_capacity(&mut next.reservation, next.order.len(), next.order.capacity())?;
        next.selection.extend(next.order.iter().copied());
        let bytes = storage_bytes(next.selection.len(), next.order.capacity())?;
        next.reservation.set_allocated_bytes(bytes)?;
        next.reservation.shrink_to(bytes)?;
        Ok(next)
    }

    /// Extend the existing tree. Only a full order Vec requires a replacement;
    /// Shift-click does not clone the complete selection tree.
    pub(super) fn append_root_selection(&mut self, ids: &[ObjectId]) -> Result<()> {
        let additional = ids.iter().filter(|id| !self.selection.contains(*id)).count();
        if additional == 0 { return Ok(()); }
        let count = self.selection.len().checked_add(additional).ok_or(Error::Overflow("selection count"))?;
        let capacity = self.selection_order.len().checked_add(additional)
            .ok_or(Error::Overflow("selection order"))?.max(self.selection_order.capacity());
        // The old owner stays charged through potential tree splits and Vec
        // replacement. This is a conservative BTree allowance, not RSS.
        let mut reservation = self.reserve_root_selection(count, capacity)?;
        let mut replacement = if capacity > self.selection_order.capacity() {
            let mut order = Vec::new();
            order.try_reserve_exact(capacity).map_err(|_| Error::Budget("selection order"))?;
            self.fit_selection_capacity(&mut reservation, count, order.capacity())?;
            order.extend_from_slice(&self.selection_order);
            Some(order)
        } else { None };
        let order = replacement.as_mut().unwrap_or(&mut self.selection_order);
        for id in ids {
            if self.selection.insert(*id) { order.push(*id); }
        }
        let old_order = replacement.map(|order| std::mem::replace(&mut self.selection_order, order));
        let old_reservation = self.selection_reservation.replace(reservation);
        // Keep old Vec storage before its old lease even on an internal
        // bookkeeping error; the new tree is already covered by the new owner.
        let _old_storage = (old_order, old_reservation);
        let bytes = self.root_selection_usage()?.bytes;
        // Only bookkeeping remains after insertion; the complete bound was
        // admitted before changing either collection.
        let reservation = self.selection_reservation.as_mut().expect("published selection owner");
        reservation.set_allocated_bytes(bytes)?;
        reservation.shrink_to(bytes)?;
        Ok(())
    }

    /// Removal allocates nothing. The Vec's retained capacity stays live.
    pub(super) fn release_removed_root_selection(&mut self) {
        if self.selection.is_empty() { self.selection = BTreeSet::new(); }
        let bytes = self.root_selection_usage().expect("selection removal cannot overflow its previous storage").bytes;
        if let Some(reservation) = self.selection_reservation.as_mut() {
            reservation.set_allocated_bytes(bytes).expect("selection removal stays within its admitted owner");
            reservation.shrink_to(bytes).expect("selection removal only releases memory");
        }
    }
}
