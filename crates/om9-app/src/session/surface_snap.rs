// SPDX-License-Identifier: LGPL-2.1-or-later
//! Refine B-rep display candidates only after checking their captured context.
use super::Controller;
use om9_core::{Error, Result};
use om9_scene::{
    camera::{Camera, Viewport},
    osnap::{SnapCandidate, SnapKind},
};
use std::sync::{Arc, Weak, atomic::{AtomicBool, Ordering}, mpsc};

/// Captured results are not usable until checked against the current view and document.
/// The caller must also validate the published viewport receipt and applicable
/// occlusion before using the point; this worker does not perform those checks.
pub struct SurfaceSnapReceipt {
    document: Weak<om9_core::model::Document>,
    original: Arc<Vec<SnapCandidate>>,
    refined: Vec<SnapCandidate>,
    camera: Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    selection_revision: u64,
    preview_revision: u64,
    _reservation: om9_core::transaction::ExternalMemoryReservation,
    cancellation: Arc<AtomicBool>,
}

impl SurfaceSnapReceipt {
    pub fn accept(self, controller: &Controller, camera: &Camera, viewport: Viewport, cursor: [f64; 2]) -> Result<Option<SnapCandidate>> {
        if self.cancellation.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let scene = controller.scene().ok_or(Error::Missing("surface snap scene"))?;
        if !self.document.ptr_eq(&Arc::downgrade(&scene.document)) {
            return Err(Error::Stale("surface snap document snapshot"));
        }
        if *camera != self.camera || viewport != self.viewport || cursor != self.cursor
            || controller.selection_state().revision != self.selection_revision
            || controller.preview_revision() != self.preview_revision {
            return Err(Error::Stale("surface snap view or cursor"));
        }
        for candidate in self.original.iter() {
            controller.validate_surface_candidate(candidate, camera, viewport)?;
        }
        Ok(om9_scene::osnap::choose(&self.refined))
    }
}

/// A single worker holding a registered document snapshot and caller reservation.
/// The caller supplies the full candidate set and a conservative native-work
/// reservation; candidate enumeration and occluders are outside this worker.
/// This foundation is not yet connected to UI scheduling or gesture release.
pub struct SurfaceSnapJob {
    document: Weak<om9_core::model::Document>,
    cancel: Arc<AtomicBool>,
    result: Option<mpsc::Receiver<Result<SurfaceSnapReceipt>>>,
    delivered: bool,
    candidates: Arc<Vec<SnapCandidate>>,
    camera: Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    selection_revision: u64,
    preview_revision: u64,
}

impl SurfaceSnapJob {
    fn validate_request(
        controller: &Controller,
        candidates: &Vec<SnapCandidate>,
        camera: &Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: &om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<()> {
        use om9_core::budget::memory::MemoryClass;
        if candidates.is_empty() || candidates.len() as u64 > controller.coordinator().limits().max_outputs
            || reservation.class() != MemoryClass::Kernel
            || !reservation.belongs_to(&controller.coordinator().external_memory_account()) {
            return Err(Error::Invalid("surface snap worker admission"));
        }
        // Retain the actual input allocation and make a result candidate for
        // each input. Each cloned source owns a copy of its instance path.
        // This is a Rust storage floor, not an upper bound on allocator or OCCT work.
        let overflow = || Error::Overflow("surface snap worker candidates");
        let slots = candidates.capacity().checked_add(candidates.len()).ok_or_else(overflow)?;
        let mut minimum = slots.checked_mul(std::mem::size_of::<SnapCandidate>()).ok_or_else(overflow)?;
        for candidate in candidates {
            if let Some(source) = &candidate.surface_source {
                let paths = source.path.capacity().checked_add(source.path.len()).ok_or_else(overflow)?;
                minimum = minimum.checked_add(paths.checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>()).ok_or_else(overflow)?).ok_or_else(overflow)?;
            }
        }
        let minimum = u64::try_from(minimum).map_err(|_| overflow())?;
        if reservation.bytes() < minimum {
            return Err(Error::Budget("surface snap worker candidates"));
        }
        for candidate in candidates {
            controller.validate_surface_candidate(candidate, camera, viewport)?;
            if candidate.surface_source.as_ref().is_none_or(|source| source.cursor != cursor) {
                return Err(Error::Stale("surface snap cursor"));
            }
        }
        Ok(())
    }

    pub fn start(
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<Self> {
        Self::validate_request(controller, &candidates, &camera, viewport, cursor, &reservation)?;
        let permit = controller.coordinator().external_memory_account().try_acquire_surface_snap()?;
        Self::start_with_permit(controller, candidates, camera, viewport, cursor, reservation, permit)
    }

    fn start_with_permit(
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
        permit: om9_core::transaction::ExternalSurfaceSnapPermit,
    ) -> Result<Self> {
        Self::validate_request(controller, &candidates, &camera, viewport, cursor, &reservation)?;
        let source_document = Arc::downgrade(&controller.scene().ok_or(Error::Missing("surface snap scene"))?.document);
        let worker_source_document = source_document.clone();
        let document = controller.coordinator().capture_tracked_document(controller.document().id)?;
        if !document.matches_weak(&source_document) {
            return Err(Error::Stale("surface snap scene document"));
        }
        let limits = *controller.coordinator().limits();
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        let candidates = Arc::new(candidates);
        let worker_candidates = Arc::clone(&candidates);
        let worker_camera = camera.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-surface-snap".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut refined = Vec::new();
                refined.try_reserve_exact(worker_candidates.len()).map_err(|_| Error::Budget("surface snap results"))?;
                let mut index = 0;
                while index < worker_candidates.len() {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    let mut end = index + 1;
                    while end < worker_candidates.len() && end - index < 8
                        && same_batch_source(&worker_candidates[index], &worker_candidates[end]) {
                        end += 1;
                    }
                    if end - index > 1 {
                        let mut group = [&worker_candidates[index]; 8];
                        for (slot, candidate) in group.iter_mut().zip(&worker_candidates[index..end]) { *slot = candidate; }
                        refined.extend(refine_captured_group(&document, &group[..end-index], &worker_camera, viewport, &limits)?.into_iter().flatten());
                    } else {
                        if let Some(candidate) = try_refine_captured_surface(&document, &worker_candidates[index], &worker_camera, viewport, &limits)? {
                            refined.push(candidate);
                        }
                    }
                    index = end;
                }
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                Ok(SurfaceSnapReceipt { document: worker_source_document, original: worker_candidates, refined, camera: worker_camera, viewport, cursor, selection_revision, preview_revision, _reservation: reservation, cancellation: Arc::clone(&worker_cancel) })
            })).unwrap_or_else(|_| Err(Error::Io("surface snap worker panicked".into())));
            // All native calls have finished. Release before notifying the
            // scheduler, so its next job can acquire the slot immediately.
            drop(permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("surface snap worker start: {error}")))?;
        Ok(Self { document: source_document, cancel, result: Some(receiver), delivered: false, candidates, camera, viewport, cursor, selection_revision, preview_revision })
    }

    pub fn cancel(&self) { self.cancel.store(true, Ordering::Release); }
    pub fn try_result(&mut self) -> Option<Result<SurfaceSnapReceipt>> {
        let receiver = self.result.as_ref()?;
        match receiver.try_recv() {
            Ok(result) => { self.result = None; self.delivered = true; Some(result) }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.result = None;
                Some(Err(Error::Io("surface snap worker exited without a result".into())))
            }
        }
    }
}

impl Drop for SurfaceSnapJob {
    fn drop(&mut self) { if !self.delivered { self.cancel(); } }
}

struct PendingSurfaceSnap {
    document: Weak<om9_core::model::Document>,
    candidates: Vec<SnapCandidate>,
    camera: Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    selection_revision: u64,
    preview_revision: u64,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}

/// Bounded latest-input scheduling; UI integration and memory sizing remain separate.
#[derive(Default)]
pub struct SurfaceSnapScheduler {
    active: Option<SurfaceSnapJob>,
    obsolete: bool,
    pending: Option<PendingSurfaceSnap>,
}

impl SurfaceSnapScheduler {
    pub fn submit(
        &mut self,
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<()> {
        SurfaceSnapJob::validate_request(controller, &candidates, &camera, viewport, cursor, &reservation)?;
        let document = Arc::downgrade(&controller.scene().ok_or(Error::Missing("surface snap scene"))?.document);
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        if !self.obsolete && self.pending.is_none() && self.active.as_ref().is_some_and(|active| {
            active.document.ptr_eq(&document) && active.candidates.as_slice() == candidates.as_slice() && active.camera == camera
                && active.viewport == viewport && active.cursor == cursor
                && active.selection_revision == selection_revision && active.preview_revision == preview_revision
        }) {
            return Ok(());
        }
        if self.pending.as_ref().is_some_and(|pending| {
            pending.document.ptr_eq(&document) && pending.candidates == candidates && pending.camera == camera
                && pending.viewport == viewport && pending.cursor == cursor
                && pending.selection_revision == selection_revision && pending.preview_revision == preview_revision
        }) {
            return Ok(());
        }
        let request = PendingSurfaceSnap {
            document, candidates, camera, viewport, cursor,
            selection_revision, preview_revision, reservation,
        };
        if let Some(active) = &self.active {
            active.cancel();
            self.obsolete = true;
            self.pending = Some(request);
        } else {
            self.pending = Some(request);
            self.obsolete = false;
            self.try_launch_pending(controller)?;
        }
        Ok(())
    }

    /// Cancel queued work immediately. An in-flight native call may still finish;
    /// poll until it has exited before starting another request.
    pub fn cancel(&mut self) {
        self.pending = None;
        if let Some(active) = &self.active {
            active.cancel();
            self.obsolete = true;
        }
    }

    pub fn is_busy(&self) -> bool { self.active.is_some() || self.pending.is_some() }

    fn try_launch_pending(&mut self, controller: &Controller) -> Result<()> {
        let Some(request) = self.pending.as_ref() else { return Ok(()) };
        if controller.scene().is_none_or(|scene| !request.document.ptr_eq(&Arc::downgrade(&scene.document)))
            || request.selection_revision != controller.selection_state().revision
            || request.preview_revision != controller.preview_revision() {
            self.pending = None;
            return Err(Error::Stale("pending surface snap context"));
        }
        let permit = match controller.coordinator().external_memory_account().try_acquire_surface_snap() {
            Ok(permit) => permit,
            Err(Error::Invalid("surface snap already active")) => return Ok(()),
            Err(error) => return Err(error),
        };
        let request = self.pending.take().expect("pending request checked above");
        self.active = Some(SurfaceSnapJob::start_with_permit(controller, request.candidates, request.camera,
            request.viewport, request.cursor, request.reservation, permit)?);
        Ok(())
    }

    /// Nonblocking. The caller must continue polling while busy and separately
    /// validate the published frame and occlusion before using a receipt.
    pub fn poll(&mut self, controller: &Controller) -> Option<Result<SurfaceSnapReceipt>> {
        let mut completed = None;
        if let Some(result) = self.active.as_mut().and_then(SurfaceSnapJob::try_result) {
            self.active = None;
            if !self.obsolete { completed = Some(result); }
            self.obsolete = false;
        }
        if self.active.is_none() && let Err(error) = self.try_launch_pending(controller) {
            return Some(Err(error));
        }
        completed
    }
}

fn same_batch_source(a: &SnapCandidate, b: &SnapCandidate) -> bool {
    let (Some(a_source), Some(b_source)) = (&a.surface_source, &b.surface_source) else { return false };
    a.kind == SnapKind::Surface && b.kind == SnapKind::Surface && a.root == b.root
        && a_source.face.is_some() && b_source.face.is_some()
        && a_source.source == b_source.source && a_source.path == b_source.path
        && a_source.leaf == b_source.leaf && a_source.scene_epoch == b_source.scene_epoch
        && a_source.placement == b_source.placement && a_source.camera == b_source.camera
        && a_source.viewport == b_source.viewport && a_source.cursor == b_source.cursor
}

fn captured_entity<'a>(document: &'a om9_core::model::Document, candidate: &SnapCandidate) -> Result<&'a om9_core::model::Entity> {
    let source = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
    document.resolve(&source.source)?;
    let mut entity = document.entities.get(&candidate.root.ok_or(Error::Missing("surface snap root"))?)
        .ok_or(Error::Missing("surface snap root"))?;
    for id in &source.path {
        let om9_core::geometry::Geometry::Instance(definition) = entity.geometry else { return Err(Error::Invalid("surface snap instance path")) };
        entity = document.definitions.get(&definition).ok_or(Error::Missing("surface snap definition"))?
            .members.iter().find(|member| member.id == *id).ok_or(Error::Missing("surface snap member"))?;
    }
    if entity.id != source.leaf { return Err(Error::Stale("surface snap leaf")); }
    Ok(entity)
}

fn refine_captured_group(
    document: &om9_core::model::Document, candidates: &[&SnapCandidate], camera: &Camera,
    viewport: Viewport, limits: &om9_core::budget::Limits,
) -> Result<[Option<SnapCandidate>; 8]> {
    if candidates.is_empty() || candidates.len() > 8 { return Err(Error::Invalid("surface snap group")); }
    let mut refined: [Option<SnapCandidate>; 8] = std::array::from_fn(|_| None);
    #[cfg(feature = "occt-brep")]
    {
        let first = candidates.first().ok_or(Error::Invalid("empty surface snap group"))?;
        let source = first.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
        let entity = captured_entity(document, first)?;
        if let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry {
            let mut faces = [uuid::Uuid::nil(); 8];
            for (index, candidate) in candidates.iter().enumerate() {
                if index >= faces.len() || !same_batch_source(first, candidate) { return Err(Error::Invalid("surface snap group")); }
                faces[index] = candidate.surface_source.as_ref().and_then(|s| s.face).ok_or(Error::Missing("surface snap face"))?;
            }
            let ray = camera.visible_ray(source.cursor, viewport)?;
            let tolerance = document.effective_tolerance()?.absolute_mm;
            let points = om9_kernel::measurement::try_brep_face_ray_hits_fixed(brep, &faces[..candidates.len()], source.placement,
                ray.origin, ray.direction, tolerance, limits).map_err(|error| Error::Io(format!("surface snap refinement: {error}")))?;
            if let Some(points) = points {
                for (index, (candidate, &point)) in candidates.iter().zip(points.as_slice()).enumerate() {
                    let Some(projected) = camera.project(point, viewport)? else { continue };
                    let mut result = (*candidate).clone();
                    result.world = point;
                    result.screen = projected.screen;
                    result.distance_px = (projected.screen[0] - source.cursor[0]).hypot(projected.screen[1] - source.cursor[1]);
                    result.normal = None;
                    refined[index] = Some(result);
                }
                return Ok(refined);
            }
            // A display triangle can hit where its exact face does not. Retry
            // individually only for this geometric miss, preserving each slot.
            for (index, candidate) in candidates.iter().enumerate() {
                refined[index] = try_refine_captured_surface(document, candidate, camera, viewport, limits)?;
            }
            return Ok(refined);
        }
    }
    for (index, candidate) in candidates.iter().enumerate() {
        refined[index] = try_refine_captured_surface(document, candidate, camera, viewport, limits)?;
    }
    Ok(refined)
}

fn refine_captured_surface(
    document: &om9_core::model::Document,
    candidate: &SnapCandidate,
    camera: &Camera,
    viewport: Viewport,
    limits: &om9_core::budget::Limits,
) -> Result<SnapCandidate> {
    try_refine_captured_surface(document, candidate, camera, viewport, limits)?
        .ok_or(Error::Missing("visible exact surface snap intersection"))
}

fn try_refine_captured_surface(
    document: &om9_core::model::Document,
    candidate: &SnapCandidate,
    camera: &Camera,
    viewport: Viewport,
    limits: &om9_core::budget::Limits,
) -> Result<Option<SnapCandidate>> {
    let source = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
    let entity = captured_entity(document, candidate)?;
    #[cfg(feature = "occt-brep")]
    {
        let ray = camera.visible_ray(source.cursor, viewport)?;
        let tolerance = document.effective_tolerance()?.absolute_mm;
        let point = match &entity.geometry {
            om9_core::geometry::Geometry::Brep(brep) => om9_kernel::measurement::try_brep_face_ray_hit(
                brep, source.face.ok_or(Error::Missing("surface snap face"))?, source.placement,
                ray.origin, ray.direction, candidate.world, tolerance, limits),
            geometry @ om9_core::geometry::Geometry::Surface(_) => om9_kernel::measurement::try_surface_ray_hit(
                geometry, source.placement, ray.origin, ray.direction, ray.origin, tolerance, limits),
            _ => return Err(Error::Unsupported("exact surface snap for this geometry")),
        }.map_err(|error| Error::Io(format!("surface snap refinement: {error}")))?;
        let Some(point) = point else { return Ok(None) };
        let Some(projected) = camera.project(point, viewport)? else { return Ok(None) };
        let mut result = candidate.clone();
        result.world = point;
        result.screen = projected.screen;
        result.distance_px = (projected.screen[0] - source.cursor[0]).hypot(projected.screen[1] - source.cursor[1]);
        result.normal = None;
        Ok(Some(result))
    }
    #[cfg(not(feature = "occt-brep"))]
    { let _ = (camera, viewport, limits, source, entity); Err(Error::Unsupported("surface snap requires OCCT")) }
}

impl Controller {
    pub(crate) fn can_group_surface_candidates(a: &SnapCandidate, b: &SnapCandidate) -> bool {
        same_batch_source(a, b)
    }

    pub(crate) fn refine_surface_snap_group(
        &self, candidates: &[&SnapCandidate], camera: &Camera, viewport: Viewport,
    ) -> Result<[Option<SnapCandidate>; 8]> {
        if candidates.is_empty() || candidates.len() > 8
            || candidates.iter().any(|candidate| !same_batch_source(&candidates[0], candidate)) {
            return Err(Error::Invalid("surface snap group"));
        }
        for candidate in candidates { self.validate_surface_candidate(candidate, camera, viewport)?; }
        refine_captured_group(self.document(), candidates, camera, viewport, self.coordinator().limits())
    }

    pub(crate) fn validate_surface_candidate<'a>(
        &'a self,
        candidate: &SnapCandidate,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<&'a om9_scene::scene::SceneObject> {
        if candidate.kind != SnapKind::Surface {
            return Err(Error::Invalid("surface snap candidate required"));
        }
        let source = candidate
            .surface_source
            .as_ref()
            .ok_or(Error::Unsupported("surface snap has no source binding"))?;
        if &source.camera != camera
            || source.viewport != viewport
            || source.scene_epoch != self.coordinator().epoch()
        {
            return Err(Error::Stale("surface snap camera or scene"));
        }
        let root = candidate.root.ok_or(Error::Invalid("surface snap root"))?;
        if source.source.object != root || self.document().resolve(&source.source).is_err() {
            return Err(Error::Stale("surface snap source"));
        }
        let scene = self.scene().ok_or(Error::Missing("surface snap scene"))?;
        if scene.epoch != source.scene_epoch {
            return Err(Error::Stale("surface snap scene"));
        }
        let object = scene
            .objects
            .iter()
            .find(|object| {
                object.root == root
                    && object.path == source.path
                    && object.leaf == source.leaf
                    && object.source == source.source
            })
            .ok_or(Error::Stale("surface snap instance"))?;
        let placement = object.display_transform()?;
        if placement != source.placement {
            return Err(Error::Stale("surface snap placement"));
        }
        if !matches!(&scene.entity_for(object)?.geometry, om9_core::geometry::Geometry::Surface(_)) && source.face.is_none() {
            return Err(Error::Unsupported("surface snap face binding"));
        }
        Ok(object)
    }

    /// Resolve a surface candidate against its canonical geometry and source binding.
    pub fn refine_surface_snap(
        &self,
        candidate: &SnapCandidate,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<SnapCandidate> {
        self.validate_surface_candidate(candidate, camera, viewport)?;
        refine_captured_surface(self.document(), candidate, camera, viewport, self.coordinator().limits())
    }
    pub(crate) fn try_refine_surface_snap(
        &self, candidate: &SnapCandidate, camera: &Camera, viewport: Viewport,
    ) -> Result<Option<SnapCandidate>> {
        self.validate_surface_candidate(candidate, camera, viewport)?;
        try_refine_captured_surface(self.document(), candidate, camera, viewport, self.coordinator().limits())
    }
}
