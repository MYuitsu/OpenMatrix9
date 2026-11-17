//! Batch profile replacement: all candidates validate against the same old scene.
use super::*;
use crate::display::ViewportDetail;
use om9_core::{budget::Usage, geometry::Geometry, identity::ObjectId};

pub(crate) struct DetailChange {
    detail: ViewportDetail,
    publication: uuid::Uuid,
    objects: Vec<ObjectId>,
    job: Option<TopologyRemeshJob>,
    prepared: Vec<PreparedTopologyScene>,
    _metadata: ExternalMemoryReservation,
}
const KERNEL_BYTES: u64 = 64 * 1024 * 1024;
const BUFFER_BYTES: u64 = 32 * 1024 * 1024;
const SCENE_BYTES: u64 = 16 * 1024 * 1024;

impl Controller {
    pub fn viewport_detail_pending(&self) -> bool { self.viewport_detail_change.is_some() }
    pub fn cancel_viewport_detail(&mut self) { drop(self.viewport_detail_change.take()); }
    pub fn request_viewport_detail(&mut self, detail: ViewportDetail) -> Result<()> {
        if self.viewport_detail_change.is_some() || self.topology_remesh_pending()
            || self.is_running() || self.boolean_union_worker_pending() { return Err(Error::Busy); }
        let count = self.document().entities.values().filter(|e| matches!(e.geometry, Geometry::Brep(_))).count();
        let bytes = Usage::array::<ObjectId>(count)?.bytes.checked_add(Usage::array::<PreparedTopologyScene>(count)?.bytes)
            .ok_or(Error::Overflow("viewport detail metadata"))?.max(1);
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut metadata = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
        let mut objects = Vec::new();
        objects.try_reserve_exact(count).map_err(|_| Error::Budget("viewport detail objects"))?;
        let mut prepared = Vec::new();
        prepared.try_reserve_exact(count).map_err(|_| Error::Budget("viewport detail scenes"))?;
        for e in self.document().entities.values() {
            if matches!(e.geometry, Geometry::Brep(_)) {
                let proof = self.topology_source_proof_for(Some(e.id)).ok_or(Error::Unsupported("display profile requires attested B-Rep topology"))?;
                if validate_proof(self, &proof).is_err() {
                    // Inserting the second sphere changes the document context,
                    // but not the first sphere's immutable source allocation.
                    let _ = proof.rebind_shared_source(&self.coordinator)?;
                }
                objects.push(e.id);
            }
        }
        let actual = Usage::array::<ObjectId>(objects.capacity())?.bytes
            .checked_add(Usage::array::<PreparedTopologyScene>(prepared.capacity())?.bytes).ok_or(Error::Overflow("viewport detail metadata"))?.max(1);
        if actual > metadata.bytes() { return Err(Error::Budget("viewport detail metadata")); }
        metadata.set_allocated_bytes(actual)?;
        if objects.is_empty() { self.viewport_detail = detail; return Ok(()); }
        let publication = self.scene().ok_or(Error::Missing("viewport detail scene"))?.publication_id();
        self.viewport_detail_change = Some(DetailChange { detail, publication, objects, job: None, prepared, _metadata: metadata });
        Ok(())
    }
    pub fn poll_viewport_detail(&mut self) -> Option<Result<ViewportDetail>> {
        let mut change = self.viewport_detail_change.take()?;
        let result = change.poll(self);
        match result {
            Ok(Some(detail)) => Some(Ok(detail)),
            Ok(None) => { self.viewport_detail_change = Some(change); None },
            Err(error) => Some(Err(error)),
        }
    }
}
impl DetailChange {
    fn poll(&mut self, controller: &mut Controller) -> Result<Option<ViewportDetail>> {
        if controller.scene().ok_or(Error::Missing("viewport detail scene"))?.publication_id() != self.publication {
            return Err(Error::Stale("viewport detail scene"));
        }
        for prepared in &self.prepared { prepared.validate_current(controller)?; }
        if let Some(job) = &mut self.job {
            let Some(receipt) = job.try_result() else { return Ok(None); };
            let prepared = receipt?.prepare_display(controller, BUFFER_BYTES)?.prepare_scene(controller, SCENE_BYTES)?;
            self.prepared.push(prepared);
            self.job = None;
        }
        if self.prepared.len() < self.objects.len() {
            let id = self.objects[self.prepared.len()];
            let kernel = controller.reserve_remesh_budget(MemoryClass::Kernel, KERNEL_BYTES, id)?;
            let display = controller.reserve_remesh_budget(MemoryClass::CpuCache, BUFFER_BYTES, id)?;
            self.job = Some(controller.start_topology_remesh_inner(self.detail.linear_mm(), self.detail.angular(), kernel, Some(display), Some(id))?);
            return Ok(None);
        }
        let held = controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, SCENE_BYTES, physical, held)?;
        // The first prepared scene already includes the unchanged ordinary-buffer owner.
        let first = self.prepared.first().ok_or(Error::Missing("viewport detail scene"))?;
        let stamp = first.owner._proof.stamp();
        let representation = first.scene.root_brep_representation(&stamp)?;
        let mut scene = controller.scene().ok_or(Error::Missing("viewport detail scene"))?.try_replace_root_brep_representation(&representation)?;
        for prepared in self.prepared.iter().skip(1) {
            let representation = prepared.scene.root_brep_representation(&prepared.owner._proof.stamp())?;
            scene = scene.try_replace_root_brep_representation(&representation)?;
        }
        for prepared in &self.prepared { prepared.validate_current(controller)?; }
        if scene.managed_memory_allowance()? > reservation.bytes() { return Err(Error::Budget("viewport detail scene")); }
        scene.transfer_memory_allowance(&mut reservation, &controller.coordinator, held)?;
        let original_buffers = first.original_buffers.clone();
        let owner = original_buffers.filter(|_| scene.shared_buffer_bytes() != 0).map(|lease| super::super::SceneBufferOwner {
            epoch: scene.epoch, document: Arc::downgrade(&scene.document), bytes: scene.shared_buffer_bytes(), lease,
        });
        controller.publish_owned_scene(scene, owner);
        for prepared in self.prepared.drain(..) {
            let proof = prepared.owner._proof.clone();
            let PreparedTopologyScene { owner, receipt, display_bytes, .. } = prepared;
            drop(receipt);
            if let Ok(mut guard) = owner.kernel_reservation.lock() {
                let _ = guard.set_allocated_bytes(display_bytes);
                let _ = guard.shrink_to(display_bytes);
            }
            let _ = controller.retain_current_topology_proof(proof);
        }
        controller.viewport_detail = self.detail;
        Ok(Some(self.detail))
    }
}
