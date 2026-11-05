// SPDX-License-Identifier: LGPL-2.1-or-later
//! Source-checked subobject selection, including complete nested occurrence paths.
use super::Controller;
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::{ObjectId, SourceStamp},
};
use om9_scene::scene::{PickHit, Scene, SceneObject, Subelement};

/// A member path is part of identity: shared topology in two occurrences is two selectable parts.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedSubobject {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub subset: Subelement,
    pub source: SourceStamp,
    pub world_epoch: u64,
}

// Input paths and candidate slots must be dropped before either lease on error.
struct ReservedSubselection {
    reference: SelectedSubobject,
    slots: Vec<SelectedSubobject>,
    reservation: om9_core::transaction::ExternalMemoryReservation,
    extra: Option<om9_core::transaction::ExternalMemoryReservation>,
}
impl SelectedSubobject {
    pub fn from_hit(hit: &PickHit) -> Self {
        Self {
            root: hit.root,
            path: hit.path.clone(),
            subset: hit.subset.clone(),
            source: hit.source,
            world_epoch: hit.world_epoch,
        }
    }
    fn same_key(&self, other: &Self) -> bool {
        self.root == other.root && self.path == other.path && self.subset == other.subset
    }
    fn validate<'a>(&self, scene: &'a Scene, epoch: u64) -> Result<&'a SceneObject> {
        if self.world_epoch != epoch || scene.epoch != epoch {
            return Err(Error::Stale("subobject publication"));
        }
        scene.document.resolve(&self.source)?;
        let object = scene
            .objects
            .iter()
            .find(|object| object.root == self.root && object.path == self.path)
            .ok_or(Error::Missing("subobject occurrence path"))?;
        if object.source != self.source {
            return Err(Error::Stale("subobject source"));
        }
        if !object.visible || !object.selectable || !object.has_pick_geometry() {
            return Err(Error::Permission("subobject owner"));
        }
        validate_subset(object, &scene.entity_for(object)?.geometry, &self.subset)?;
        // Resolve the definition path, never a nearby or first matching member.
        if scene.entity_for(object)?.id != object.leaf {
            return Err(Error::Stale("subobject leaf"));
        }
        Ok(object)
    }
}

impl Controller {
    fn root_subobject(&self, root: ObjectId, subset: Subelement) -> Result<SelectedSubobject> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("subobject scene"))?;
        let object = scene
            .objects
            .iter()
            .find(|o| o.root == root && o.path.is_empty())
            .ok_or(Error::Missing(
                "subobject root geometry; occurrence path required",
            ))?;
        Ok(SelectedSubobject {
            root,
            path: Vec::new(),
            subset,
            source: object.source,
            world_epoch: scene.epoch,
        })
    }
    /// Root-only compatibility door; instances must provide the path from their actual hit.
    pub fn select_subobject(&mut self, root: ObjectId, subset: Subelement) -> Result<()> {
        let reference = self.root_subobject(root, subset)?;
        self.select_subobject_ref(reference)
    }
    pub fn toggle_subobject(&mut self, root: ObjectId, subset: Subelement) -> Result<()> {
        let reference = self.root_subobject(root, subset)?;
        self.toggle_subobject_ref(reference)
    }
    pub fn select_subobject_ref(&mut self, reference: SelectedSubobject) -> Result<()> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("subobject scene"))?;
        reference.validate(scene, self.coordinator.epoch())?;
        if self.subselection.iter().any(|old| old.same_key(&reference)) {
            return Ok(());
        }
        self.append_subselection_owned(reference)?;
        self.sub_revision = self.sub_revision.saturating_add(1);
        Ok(())
    }
    pub fn toggle_subobject_ref(&mut self, reference: SelectedSubobject) -> Result<()> {
        reference.validate(
            self.scene
                .as_ref()
                .ok_or(Error::Missing("subobject scene"))?,
            self.coordinator.epoch(),
        )?;
        if let Some(index) = self
            .subselection
            .iter()
            .position(|old| old.same_key(&reference))
        {
            self.subselection.remove(index);
            self.release_removed_subselection();
            self.sub_revision = self.sub_revision.saturating_add(1);
            Ok(())
        } else {
            self.select_subobject_ref(reference)
        }
    }
    pub fn clear_subselection(&mut self) {
        if !self.subselection.is_empty() {
            self.subselection.clear();
            self.release_removed_subselection();
            self.sub_revision = self.sub_revision.saturating_add(1);
        }
    }
    pub fn subselection(&self) -> &[SelectedSubobject] {
        &self.subselection
    }
    pub fn sub_revision(&self) -> u64 {
        self.sub_revision
    }
    pub fn prune_subselection(&mut self) {
        let Some(scene) = self.scene.as_ref() else {
            self.clear_subselection();
            return;
        };
        let before = self.subselection.len();
        self.subselection
            .retain(|key| key.validate(scene, self.coordinator.epoch()).is_ok());
        if before != self.subselection.len() {
            self.release_removed_subselection();
            self.sub_revision = self.sub_revision.saturating_add(1);
        }
    }

    pub(super) fn subselection_storage_usage(&self) -> Result<Usage> {
        let mut bytes = Usage::array::<SelectedSubobject>(self.subselection.capacity())?.bytes;
        for selected in &self.subselection {
            bytes = bytes.checked_add(Usage::array::<ObjectId>(selected.path.capacity())?.bytes)
                .ok_or(Error::Overflow("subselection storage"))?;
        }
        Ok(Usage { bytes, ..Usage::default() })
    }

    fn append_subselection_owned(&mut self, reference: SelectedSubobject) -> Result<()> {
        use om9_core::budget::memory::MemoryClass;
        let path_bytes = Usage::array::<ObjectId>(reference.path.capacity())?.bytes;
        let needed = self.subselection.len().checked_add(1).ok_or(Error::Overflow("subselection length"))?;
        let resize = needed > self.subselection.capacity();
        // An empty path in a spare slot creates no allocation or new quota.
        if !resize && path_bytes == 0 {
            self.subselection.push(reference);
            return Ok(());
        }
        let capacity = if resize {
            self.subselection.capacity().checked_mul(2).ok_or(Error::Overflow("subselection capacity"))?.max(needed).max(4)
        } else { 0 };
        let slot_bytes = Usage::array::<SelectedSubobject>(capacity)?.bytes;
        let bytes = slot_bytes.checked_add(path_bytes).ok_or(Error::Overflow("subselection storage"))?;
        let reservation = self.coordinator.reserve_external_memory_with_allocated(
            MemoryClass::CpuCache, bytes, path_bytes,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), self.scene_cache_usage()?)?;
        let mut next = ReservedSubselection { reference, slots: Vec::new(), reservation, extra: None };
        if !resize {
            let current = self.subselection_reservation.as_mut().ok_or(Error::Invalid("subselection storage owner"))?;
            current.transfer_allocated_from(&mut next.reservation, path_bytes)?;
            self.subselection.push(next.reference);
            return Ok(());
        }

        next.slots.try_reserve_exact(capacity).map_err(|_| Error::Budget("subobject selection"))?;
        let actual_slots = Usage::array::<SelectedSubobject>(next.slots.capacity())?.bytes;
        let actual = actual_slots.checked_add(path_bytes).ok_or(Error::Overflow("subselection storage"))?;
        next.reservation.set_allocated_bytes(actual.min(next.reservation.bytes()))?;
        if actual > next.reservation.bytes() {
            let extra = actual - next.reservation.bytes();
            // The allocator has already produced these extra slots. Admit them
            // as materialized, not as a second physical request for old data.
            next.extra = Some(self.coordinator.reserve_external_memory_with_allocated(
                MemoryClass::CpuCache, extra, extra,
                om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), self.scene_cache_usage()?)?);
            next.reservation.transfer_allocated_from(next.extra.as_mut().expect("extra capacity owner"), extra)?;
            next.extra = None;
        }
        let old_slots = Usage::array::<SelectedSubobject>(self.subselection.capacity())?.bytes;
        if let Some(previous) = self.subselection_reservation.as_mut() {
            let paths = previous.bytes().checked_sub(old_slots).ok_or(Error::Invalid("subselection path accounting"))?;
            next.reservation.transfer_allocated_from(previous, paths)?;
        }
        // No fallible work after transferring paths: move them, never clone.
        next.slots.append(&mut self.subselection);
        next.slots.push(next.reference);
        let old_slots = std::mem::replace(&mut self.subselection, next.slots);
        let old_reservation = self.subselection_reservation.replace(next.reservation);
        drop(old_slots);
        drop(old_reservation);
        Ok(())
    }

    fn release_removed_subselection(&mut self) {
        let bytes = self.subselection_storage_usage().expect("subselection removal cannot overflow previous storage").bytes;
        if let Some(reservation) = self.subselection_reservation.as_mut() {
            reservation.set_allocated_bytes(bytes).expect("subselection removal stays within its owner");
            reservation.shrink_to(bytes).expect("subselection removal only releases storage");
        }
    }
}

fn validate_subset(
    object: &om9_scene::scene::SceneObject,
    geometry: &om9_core::geometry::Geometry,
    subset: &Subelement,
) -> Result<()> {
    use om9_scene::scene::RenderBuffers;
    if let om9_core::geometry::Geometry::Brep(shape) = geometry {
        let valid = match subset {
            Subelement::BrepFace(id) => shape.display.faces.contains(id),
            Subelement::BrepEdge(id) => shape.display.edges.contains(id),
            Subelement::BrepVertex(id) => shape.display.vertex_ids.contains(id),
            _ => false,
        };
        return if valid {
            Ok(())
        } else {
            Err(Error::Stale("BRep topology identity"))
        };
    }
    let valid = match (subset, object.buffers.as_ref()) {
        (Subelement::MeshVertex(index), RenderBuffers::Mesh { vertices, .. }) => {
            (*index as usize) < vertices.len()
        }
        (Subelement::MeshFace(face), RenderBuffers::Mesh { triangles, .. }) => {
            triangles.iter().any(|(_, f)| f == face)
        }
        (Subelement::MeshEdge { a, b }, RenderBuffers::Mesh { edges, .. }) => {
            a < b
                && edges
                    .iter()
                    .any(|[x, y]| (*x == *a && *y == *b) || (*x == *b && *y == *a))
        }
        (Subelement::CurveParameter(t), RenderBuffers::Curve { parameters, .. }) => {
            t.is_finite()
                && parameters
                    .first()
                    .zip(parameters.last())
                    .is_some_and(|(a, b)| t >= a && t <= b)
        }
        (Subelement::CloudPoint(index), RenderBuffers::Points { native_indices, .. }) => {
            native_indices.contains(index)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::Stale("subobject topology key"))
    }
}

impl Controller {
    /// Deletes objects in one transaction. The selection and the sub-selection drop the removed ids, so no
    /// stale key is left behind.
    pub fn delete_objects(&mut self, ids: &[ObjectId]) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        if self.scene.is_none() {
            return Err(Error::Missing("delete scene"));
        }
        let mut mutations = Vec::with_capacity(ids.len());
        for id in ids {
            if !self.document().entities.contains_key(id) {
                return Err(Error::Missing("delete object"));
            }
            mutations.push(om9_core::transaction::Mutation::Delete {
                source: self.document().stamp(*id)?,
            });
        }
        let held = self.scene_cache_usage()?;
        let prepared = self.coordinator.prepare_with_held_usage(
            om9_core::transaction::TransactionRequest::new("Delete", mutations),
            held,
        )?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            for id in ids {
                self.selection.remove(id);
            }
            self.selection_order.retain(|id| self.selection.contains(id));
            self.release_removed_root_selection();
            self.prune_subselection();
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(ids.len())
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::{Definition, Document, Entity}};

    fn nested_fixture() -> (Controller, ObjectId) {
        let mut document = Document::with_template_tolerance("nested subselection memory");
        let leaf = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let inner = Definition::new("inner", vec![leaf]);
        let inner_id = inner.id;
        document.definitions.insert(inner_id, inner);
        let member = Entity::new("nested", Geometry::Instance(inner_id), None);
        let outer = Definition::new("outer", vec![member]);
        let outer_id = outer.id;
        document.definitions.insert(outer_id, outer);
        let occurrence = Entity::new("occurrence", Geometry::Instance(outer_id), None);
        let root = occurrence.id;
        document.entities.insert(root, occurrence);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[root]).unwrap();
        (controller, root)
    }

    fn reference(controller: &Controller, root: ObjectId, subset: Subelement, capacity: usize) -> SelectedSubobject {
        let scene = controller.scene().unwrap();
        let object = scene.objects.iter().find(|o| o.root == root).unwrap();
        assert_eq!(object.path.len(), 2);
        let mut path = Vec::with_capacity(capacity);
        path.extend_from_slice(&object.path);
        SelectedSubobject { root, path, subset, source: object.source, world_epoch: scene.epoch }
    }

    #[test]
    fn detached_capture_observes_subselection_slots_and_nested_path_capacity() {
        let (mut controller, root) = nested_fixture();
        let capture = controller.capture_stl_selected().unwrap();
        let epoch = capture.source_epoch();
        let peer = controller.coordinator.memory_admission_peer().unwrap();
        let (request, requested) = std::sync::mpsc::sync_channel(1);
        let (response, observed) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            for _ in 0..5 {
                requested.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
                response.send((peer.retained_usage().unwrap().bytes, peer.external_reserved_bytes(MemoryClass::CpuCache))).unwrap();
            }
            capture.source_epoch()
        });
        let observe = || {
            request.send(()).unwrap();
            observed.recv_timeout(std::time::Duration::from_secs(10)).unwrap()
        };
        let initial = observe().0;
        let held = controller.auxiliary_usage().unwrap();
        let mut path_pointers = Vec::new();
        for subset in [Subelement::MeshFace(0), Subelement::MeshVertex(0), Subelement::MeshVertex(1),
            Subelement::MeshVertex(2), Subelement::MeshEdge { a: 0, b: 1 }] {
            let selected = reference(&controller, root, subset, 4096);
            path_pointers.push(selected.path.as_ptr());
            controller.select_subobject_ref(selected).unwrap();
        }
        assert_eq!(controller.subselection.iter().map(|s| s.path.as_ptr()).collect::<Vec<_>>(), path_pointers,
            "slot growth must move path owners without cloning their buffers");
        let slots = Usage::array::<SelectedSubobject>(controller.subselection.capacity()).unwrap().bytes;
        let paths: u64 = controller.subselection.iter().map(|s| Usage::array::<ObjectId>(s.path.capacity()).unwrap().bytes).sum();
        assert_eq!(observe().0 - initial, slots + paths,
            "detached admission must see actual live slot and nested-path capacities");
        assert_eq!(controller.auxiliary_usage().unwrap(), held, "do not charge registered subobjects in captured auxiliary too");
        controller.toggle_subobject_ref(reference(&controller, root, Subelement::MeshFace(0), 2)).unwrap();
        assert_eq!(observe().0 - initial, slots + paths - Usage::array::<ObjectId>(4096).unwrap().bytes);
        controller.clear_subselection();
        assert_eq!(observe().0 - initial, slots, "clear drops paths but retains allocated slot capacity");
        drop(controller);
        assert_eq!(observe().1, 0, "capture must not keep controller selection allocations alive");
        assert_eq!(worker.join().unwrap(), epoch);
    }

    #[test]
    fn spare_slots_admit_existing_paths_without_requesting_their_physical_ram_again() {
        fn no_pending_headroom() -> Option<(u64, u64)> { Some((1024 * 1024 * 1024, 256 * 1024 * 1024)) }
        let (mut controller, root) = nested_fixture();
        controller.select_subobject_ref(reference(&controller, root, Subelement::MeshFace(0), 2)).unwrap();
        let slots = controller.subselection.as_ptr();
        let capacity = controller.subselection.capacity();
        let before = controller.subselection_reservation.as_ref().unwrap().bytes();
        controller.coordinator.set_physical_memory_sampler(no_pending_headroom);
        let selected = reference(&controller, root, Subelement::MeshVertex(0), 4096);
        let path = selected.path.as_ptr();
        controller.select_subobject_ref(selected).unwrap();
        assert_eq!(controller.subselection.as_ptr(), slots);
        assert_eq!(controller.subselection.capacity(), capacity);
        assert_eq!(controller.subselection[1].path.as_ptr(), path);
        assert_eq!(controller.subselection_reservation.as_ref().unwrap().bytes(),
            before + Usage::array::<ObjectId>(4096).unwrap().bytes);
        assert_eq!(controller.subselection_reservation.as_ref().unwrap().pending_bytes(), 0);
        assert_eq!(controller.coordinator.undo_len(), 0);
    }

    #[test]
    fn rejected_slot_growth_preserves_subselection_paths_revision_and_history() {
        let (mut controller, root) = nested_fixture();
        for subset in [Subelement::MeshFace(0), Subelement::MeshVertex(0), Subelement::MeshVertex(1), Subelement::MeshVertex(2)] {
            controller.select_subobject_ref(reference(&controller, root, subset, 2)).unwrap();
        }
        let before = controller.subselection.clone();
        let slots = controller.subselection.as_ptr();
        let paths = controller.subselection.iter().map(|s| s.path.as_ptr()).collect::<Vec<_>>();
        let revision = controller.sub_revision;
        let owner = controller.subselection_reservation.as_ref().unwrap().bytes();
        let spheres = controller.scene().unwrap().object_spheres().unwrap();
        let index = controller.scene().unwrap().spatial_index().unwrap();
        let document = controller.coordinator.snapshot();
        let pressure = controller.coordinator.reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
        let selected = reference(&controller, root, Subelement::MeshEdge { a: 0, b: 1 }, 4096);
        assert!(matches!(controller.select_subobject_ref(selected), Err(Error::Budget(_))));
        assert_eq!(controller.subselection, before);
        assert_eq!(controller.subselection.as_ptr(), slots);
        assert_eq!(controller.subselection.iter().map(|s| s.path.as_ptr()).collect::<Vec<_>>(), paths);
        assert_eq!(controller.sub_revision, revision);
        assert_eq!(controller.coordinator.snapshot(), document);
        assert_eq!(controller.subselection_reservation.as_ref().unwrap().bytes(), owner);
        assert_eq!(controller.subselection_reservation.as_ref().unwrap().pending_bytes(), 0);
        drop(pressure);
        controller.select_subobject_ref(reference(&controller, root, Subelement::MeshEdge { a: 0, b: 1 }, 4096)).unwrap();
        assert_eq!(controller.subselection.len(), 5);
        assert_eq!(controller.subselection[..4].iter().map(|s| s.path.as_ptr()).collect::<Vec<_>>(), paths);
        drop((spheres, index));
    }
}
