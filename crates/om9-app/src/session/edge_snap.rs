// SPDX-License-Identifier: LGPL-2.1-or-later
//! Accounted, source-only extraction for B-Rep edge snapping.
//!
//! This worker does not publish candidates; callers must validate the receipt
//! against the current scene before evaluating the retained curves.
use super::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};
use om9_core::budget::memory::MemoryClass;
use om9_core::geometry::Transform;
use om9_core::transaction::ExternalMemoryReservation;
use om9_scene::osnap::SnapSource;

enum EdgeSource {
    Brep(om9_core::transaction::CapturedBrepSource),
    Native(om9_core::transaction::CapturedNativeBrepSource),
    Document(om9_core::transaction::CapturedDocument),
}

fn retained_curve_bytes(curves: &Vec<Curve>, limits: &om9_core::budget::Limits) -> Result<u64> {
    let mut bytes = (curves.capacity() as u64).checked_mul(std::mem::size_of::<Curve>() as u64)
        .ok_or(Error::Overflow("edge snap curves"))?;
    for curve in curves {
        bytes = bytes.checked_add(curve.validate(limits)?.bytes).ok_or(Error::Overflow("edge snap curves"))?;
    }
    Ok(bytes)
}

fn copy_worker_curve_parts(curve: &Curve, output: &mut Vec<Curve>, live: &mut u64,
    limits: &om9_core::budget::Limits, depth: usize, admit: &mut impl FnMut(u64, u64, u64) -> Result<()>) -> Result<()> {
    if depth > limits.max_depth { return Err(Error::Budget("curve snap source depth")); }
    match curve {
        Curve::Polycurve { segments, .. } => {
            for segment in segments { copy_worker_curve_parts(segment, output, live, limits, depth + 1, admit)?; }
        }
        Curve::Nurbs(n) => {
            n.validate(limits)?;
            let planned = n.poles.len().checked_mul(std::mem::size_of::<Point3>())
                .and_then(|bytes| bytes.checked_add(n.weights.len().checked_mul(std::mem::size_of::<f64>())?))
                .and_then(|bytes| bytes.checked_add(n.knots.len().checked_mul(std::mem::size_of::<f64>())?))
                .and_then(|bytes| bytes.checked_add(output.len().saturating_add(1).checked_mul(std::mem::size_of::<Curve>())?.saturating_mul(2)))
                .ok_or(Error::Overflow("curve snap source"))?;
            admit(*live, 0, u64::try_from(planned).map_err(|_| Error::Overflow("curve snap source"))?.saturating_mul(2))?;
            let (mut poles, mut weights, mut knots) = (Vec::new(), Vec::new(), Vec::new());
            poles.try_reserve_exact(n.poles.len()).map_err(|_| Error::Budget("curve snap poles"))?;
            weights.try_reserve_exact(n.weights.len()).map_err(|_| Error::Budget("curve snap weights"))?;
            knots.try_reserve_exact(n.knots.len()).map_err(|_| Error::Budget("curve snap knots"))?;
            poles.extend_from_slice(&n.poles); weights.extend_from_slice(&n.weights); knots.extend_from_slice(&n.knots);
            let old_capacity = output.capacity();
            output.try_reserve(1).map_err(|_| Error::Budget("curve snap sources"))?;
            let slots = output.capacity().saturating_sub(old_capacity).checked_mul(std::mem::size_of::<Curve>())
                .ok_or(Error::Overflow("curve snap sources"))?;
            let arrays = poles.capacity().checked_mul(std::mem::size_of::<Point3>())
                .and_then(|bytes| bytes.checked_add(weights.capacity().checked_mul(std::mem::size_of::<f64>())?))
                .and_then(|bytes| bytes.checked_add(knots.capacity().checked_mul(std::mem::size_of::<f64>())?))
                .and_then(|bytes| bytes.checked_add(slots)).ok_or(Error::Overflow("curve snap source"))?;
            *live = live.checked_add(u64::try_from(arrays).map_err(|_| Error::Overflow("curve snap source"))?)
                .ok_or(Error::Overflow("curve snap source"))?;
            admit(*live, 0, 0)?;
            output.push(Curve::Nurbs(om9_core::geometry::NurbsCurve { degree: n.degree, poles, weights, knots,
                domain: n.domain, periodic: n.periodic, reversed: n.reversed, seam: n.seam }));
        }
        Curve::Circle { .. } => {
            curve.validate(limits)?;
            let pending = u64::try_from(std::mem::size_of::<Curve>()).map_err(|_| Error::Overflow("curve snap source"))?
                .checked_mul(2).ok_or(Error::Overflow("curve snap source"))?;
            admit(*live, 0, pending)?;
            let old_capacity = output.capacity();
            output.try_reserve(1).map_err(|_| Error::Budget("curve snap sources"))?;
            let slots = output.capacity().saturating_sub(old_capacity).checked_mul(std::mem::size_of::<Curve>())
                .ok_or(Error::Overflow("curve snap sources"))?;
            *live = live.checked_add(u64::try_from(slots).map_err(|_| Error::Overflow("curve snap sources"))?)
                .ok_or(Error::Overflow("curve snap sources"))?;
            admit(*live, 0, 0)?;
            output.push(curve.clone());
        }
        _ => {}
    }
    Ok(())
}

struct EdgeBoundsReceipt {
    source: SnapSource,
    local: Option<[f64; 6]>,
    tolerance: f64,
    _reservation: ExternalMemoryReservation,
}

impl EdgeBoundsReceipt {
    fn validate(&self, controller: &Controller) -> Result<()> {
        if self.tolerance.to_bits() != controller.document().effective_tolerance()?.absolute_mm.to_bits() {
            return Err(Error::Stale("edge bounds tolerance"));
        }
        self.source.validate(controller.scene().ok_or(Error::Missing("edge bounds scene"))?)
    }

    fn may_hit(&self, controller: &Controller, camera: &om9_scene::camera::Camera,
        viewport: om9_scene::camera::Viewport, cursor: [f64; 2], radius: f64) -> Result<bool> {
        self.validate(controller)?;
        let Some(bounds) = self.local else { return Ok(true) };
        let scene = controller.scene().ok_or(Error::Missing("edge bounds scene"))?;
        let object = scene.objects.iter().find(|object| object.root == self.source.root
            && object.path == self.source.path && object.leaf == self.source.leaf && object.source == self.source.stamp)
            .ok_or(Error::Stale("edge bounds occurrence"))?;
        let transform = object.display_transform()?;
        let magnitude = bounds.iter().map(|value| value.abs()).fold(1f64, f64::max);
        let margin = self.tolerance.max(magnitude * 1e-12);
        if !margin.is_finite() { return Ok(true); }
        let mut corners = [[0.; 3]; 8];
        for (index, corner) in corners.iter_mut().enumerate() {
            let local = std::array::from_fn(|axis| if index & (1 << axis) == 0 {
                bounds[axis] - margin
            } else { bounds[axis + 3] + margin });
            let Ok(world) = transform.point(local) else { return Ok(true) };
            *corner = world;
        }
        let projected = match camera.projector(viewport)?.project_box(corners) {
            Ok(Some(bounds)) => bounds,
            Ok(None) | Err(_) => return Ok(true),
        };
        let radius = radius + 1e-7;
        Ok((0..2).all(|axis| cursor[axis] >= projected[0][axis] - radius
            && cursor[axis] <= projected[1][axis] + radius))
    }
}

struct EdgeBoundsJob {
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Result<EdgeBoundsReceipt>>>,
    delivered: bool,
}

impl EdgeBoundsJob {
    fn start(controller: &Controller, source: SnapSource, mut reservation: ExternalMemoryReservation,
        permit: om9_core::transaction::ExternalSurfaceSnapPermit) -> Result<Self> {
        if reservation.class() != MemoryClass::Kernel || !reservation.belongs_to(&controller.coordinator().external_memory_account()) {
            return Err(Error::Invalid("edge bounds admission"));
        }
        let scene = controller.scene().ok_or(Error::Missing("edge bounds scene"))?;
        source.validate(scene)?;
        let object = scene.objects.iter().find(|object| object.root == source.root && object.path == source.path
            && object.leaf == source.leaf && object.source == source.stamp).ok_or(Error::Stale("edge bounds occurrence"))?;
        let entity = scene.entity_for(&scene.document, object)?;
        let captured = match &entity.geometry {
            Geometry::Brep(_) => EdgeSource::Brep(scene.capture_brep_source_for(controller.coordinator(), object)?),
            Geometry::NativeBrep(_) => EdgeSource::Native(scene.capture_native_brep_source_for(controller.coordinator(), object)?),
            Geometry::Surface(_) | Geometry::Curve(_) => {
                let current = controller.coordinator().world().documents.get(&controller.document().id)
                    .ok_or(Error::Missing("edge bounds document"))?;
                if !std::ptr::eq(scene.document.as_ref(), current.as_ref()) { return Err(Error::Stale("edge bounds document")); }
                EdgeSource::Document(controller.coordinator().capture_tracked_document(controller.document().id)?)
            }
            _ => return Err(Error::Invalid("edge bounds source")),
        };
        let baseline = u64::try_from(source.path.capacity()).map_err(|_| Error::Overflow("edge bounds metadata"))?
            .checked_mul(32).and_then(|bytes| bytes.checked_add(64 * 1024)).ok_or(Error::Overflow("edge bounds metadata"))?;
        if reservation.pending_bytes() < baseline { return Err(Error::Budget("edge bounds metadata")); }
        reservation.set_allocated_bytes(baseline)?;
        reservation.recheck_pending()?;
        let limits = *controller.coordinator().limits();
        let tolerance = controller.document().effective_tolerance()?.absolute_mm;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-edge-bounds".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<EdgeBoundsReceipt> {
                let mut admit = |live: u64, pending: u64| -> Result<()> {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    let allocated = baseline.checked_add(live).ok_or(Error::Overflow("edge bounds admission"))?;
                    if allocated.checked_add(pending).ok_or(Error::Overflow("edge bounds admission"))? > reservation.bytes() {
                        return Err(Error::Budget("edge bounds admission"));
                    }
                    reservation.set_allocated_bytes(allocated)?;
                    reservation.recheck_pending()
                };
                let local = match &captured {
                    EdgeSource::Brep(brep) => om9_kernel::brep::world_bounds_source(brep.source(), &Transform::identity(), &limits).ok(),
                    EdgeSource::Native(native) => {
                        let text = om9_kernel::brep::from_3dm_source_typed_admitted(native.payload(), tolerance, &limits, &mut admit)?;
                        om9_kernel::brep::world_bounds_source(&text, &Transform::identity(), &limits).ok()
                    }
                    EdgeSource::Document(document) => {
                        let entity = super::intersection_snap::document_entity(document, &source)?;
                        if let Geometry::Surface(surface) = &entity.geometry {
                          if surface.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) { None } else {
                            let mut bounds = [f64::INFINITY, f64::INFINITY, f64::INFINITY,
                                f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
                            for point in &surface.poles { for axis in 0..3 {
                                bounds[axis] = bounds[axis].min(point[axis]); bounds[axis + 3] = bounds[axis + 3].max(point[axis]);
                            }}
                            bounds.iter().all(|value| value.is_finite()).then_some(bounds)
                          }
                        } else { None }
                    }
                };
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                reservation.set_allocated_bytes(baseline)?;
                reservation.shrink_to(baseline)?;
                Ok(EdgeBoundsReceipt { source, local, tolerance, _reservation: reservation })
            })).unwrap_or_else(|_| Err(Error::Io("edge bounds worker panicked".into())));
            drop(permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("edge bounds worker: {error}")))?;
        Ok(Self { cancel, receiver: Some(receiver), delivered: false })
    }
    fn cancel(&self) { self.cancel.store(true, Ordering::Release); }
    fn try_result(&mut self) -> Option<Result<EdgeBoundsReceipt>> {
        match self.receiver.as_ref()?.try_recv() {
            Ok(result) => { self.receiver = None; self.delivered = true; Some(result) },
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => { self.receiver = None; Some(Err(Error::Io("edge bounds worker disconnected".into()))) },
        }
    }
}
impl Drop for EdgeBoundsJob { fn drop(&mut self) { if !self.delivered { self.cancel(); } } }

#[derive(Default)]
pub(crate) struct EdgeBoundsCache {
    active: Option<(SnapSource, EdgeBoundsJob)>,
    ready: Vec<EdgeBoundsReceipt>,
    obsolete: bool,
}
impl EdgeBoundsCache {
    pub fn request(&mut self, controller: &Controller, source: SnapSource, reservation: ExternalMemoryReservation) -> Result<()> {
        self.ready.retain(|receipt| receipt.validate(controller).is_ok());
        if self.ready.iter().any(|receipt| receipt.source == source)
            || !self.obsolete && self.active.as_ref().is_some_and(|(active, _)| *active == source) { return Ok(()); }
        if self.active.is_some() { return Err(Error::Busy); }
        if self.ready.len() as u64 >= controller.coordinator().limits().max_outputs { return Err(Error::Budget("edge bounds cache")); }
        self.ready.try_reserve(1).map_err(|_| Error::Budget("edge bounds cache"))?;
        let permit = match controller.coordinator().external_memory_account().try_acquire_surface_snap() {
            Ok(permit) => permit,
            Err(Error::Invalid("surface snap already active")) => return Ok(()),
            Err(error) => return Err(error),
        };
        let job = EdgeBoundsJob::start(controller, source.clone(), reservation, permit)?;
        self.active = Some((source, job));
        self.obsolete = false;
        Ok(())
    }
    pub fn poll(&mut self, controller: &Controller) -> Result<()> {
        if let Some(result) = self.active.as_mut().and_then(|(_, job)| job.try_result()) {
            self.active = None;
            if !self.obsolete {
                let receipt = result?;
                receipt.validate(controller)?;
                self.ready.push(receipt);
            }
            self.obsolete = false;
        }
        Ok(())
    }
    pub fn may_hit(&self, controller: &Controller, source: &SnapSource, camera: &om9_scene::camera::Camera,
        viewport: om9_scene::camera::Viewport, cursor: [f64; 2], radius: f64) -> Result<Option<bool>> {
        self.ready.iter().find(|receipt| &receipt.source == source && receipt.validate(controller).is_ok())
            .map(|receipt| receipt.may_hit(controller, camera, viewport, cursor, radius)).transpose()
    }
    pub fn is_busy(&self) -> bool { self.active.is_some() }
    pub fn cancel(&mut self) {
        self.ready.clear();
        if let Some((_, active)) = &self.active { active.cancel(); self.obsolete = true; }
    }
}

pub(crate) struct EdgeSnapSourceReceipt {
    pub source: SnapSource,
    pub curves: Vec<om9_core::geometry::Curve>,
    pub centers: Vec<Point3>,
    center_error: Option<Error>,
    pub vertices: Vec<Point3>,
    vertex_error: Option<Error>,
    tolerance: f64,
    trim_placement: Option<Transform>,
    cancel: Arc<AtomicBool>,
    // The curves and source identity must be dropped before their accounting.
    _reservation: ExternalMemoryReservation,
}

impl EdgeSnapSourceReceipt {
    pub fn validate(&self, controller: &Controller) -> Result<()> {
        if self.cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        if controller.document().effective_tolerance()?.absolute_mm.to_bits() != self.tolerance.to_bits() {
            return Err(Error::Stale("edge snap source tolerance"));
        }
        let scene = controller.scene().ok_or(Error::Missing("edge snap scene"))?;
        self.source.validate(scene)?;
        if let Some(placement) = self.trim_placement {
            let object = scene.objects.iter().find(|object| object.root == self.source.root
                && object.path == self.source.path && object.leaf == self.source.leaf && object.source == self.source.stamp)
                .ok_or(Error::Stale("trim edge occurrence"))?;
            if object.display_transform()? != placement { return Err(Error::Stale("trim edge placement")); }
        }
        Ok(())
    }

    /// Project the exact rational-curve control hull. Unknown or clipped bounds
    /// remain eligible; a display tessellation is never used to reject a source.
    pub fn may_hit(&self, controller: &Controller, camera: &om9_scene::camera::Camera,
        viewport: om9_scene::camera::Viewport, cursor: [f64; 2], radius: f64) -> Result<bool> {
        self.validate(controller)?;
        if self.vertex_error.is_some() { return Ok(true); }
        let scene = controller.scene().ok_or(Error::Missing("edge snap scene"))?;
        let object = scene.objects.iter().find(|object| object.root == self.source.root
            && object.path == self.source.path && object.leaf == self.source.leaf && object.source == self.source.stamp)
            .ok_or(Error::Stale("edge snap occurrence"))?;
        let transform = object.display_transform()?;
        let mut low = [f64::INFINITY; 3];
        let mut high = [f64::NEG_INFINITY; 3];
        for &vertex in &self.vertices {
            let Ok(point) = transform.point(vertex) else { return Ok(true) };
            for axis in 0..3 { low[axis] = low[axis].min(point[axis]); high[axis] = high[axis].max(point[axis]); }
        }
        for curve in &self.curves {
            let om9_core::geometry::Curve::Nurbs(curve) = curve else { return Ok(true) };
            if curve.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) { return Ok(true); }
            for &pole in &curve.poles {
                let Ok(point) = transform.point(pole) else { return Ok(true) };
                for axis in 0..3 { low[axis] = low[axis].min(point[axis]); high[axis] = high[axis].max(point[axis]); }
            }
        }
        if low.iter().chain(high.iter()).any(|value| !value.is_finite()) { return Ok(true); }
        let mut corners = [[0.; 3]; 8];
        for (index, corner) in corners.iter_mut().enumerate() {
            *corner = std::array::from_fn(|axis| if index & (1 << axis) == 0 { low[axis] } else { high[axis] });
        }
        let bounds = match camera.projector(viewport)?.project_box(corners) {
            Ok(Some(bounds)) => bounds,
            // The box can enclose the camera/frustum without its edges entering
            // the clipping slab; unknown is never treated as a miss.
            Ok(None) | Err(_) => return Ok(true),
        };
        let padding = radius + 1e-7;
        Ok((0..2).all(|axis| cursor[axis] >= bounds[0][axis] - padding && cursor[axis] <= bounds[1][axis] + padding))
    }

}

pub(crate) struct EdgeSnapSourceJob {
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Result<EdgeSnapSourceReceipt>>>,
    delivered: bool,
}

impl EdgeSnapSourceJob {
    fn start(controller: &Controller, source: SnapSource, mut reservation: ExternalMemoryReservation,
        permit: om9_core::transaction::ExternalSurfaceSnapPermit) -> Result<Self> {
        if reservation.class() != MemoryClass::Kernel
            || !reservation.belongs_to(&controller.coordinator().external_memory_account()) {
            return Err(Error::Invalid("edge snap worker admission"));
        }
        let scene = controller.scene().ok_or(Error::Missing("edge snap scene"))?;
        source.validate(scene)?;
        let object = scene.objects.iter().find(|object| object.root == source.root && object.path == source.path
            && object.leaf == source.leaf && object.source == source.stamp)
            .ok_or(Error::Stale("edge snap occurrence"))?;
        let entity = scene.entity_for(&scene.document, object)?;
        let placement = object.display_transform()?;
        let captured = match &entity.geometry {
            om9_core::geometry::Geometry::Brep(_) => EdgeSource::Brep(scene.capture_brep_source_for(controller.coordinator(), object)?),
            om9_core::geometry::Geometry::NativeBrep(_) => EdgeSource::Native(scene.capture_native_brep_source_for(controller.coordinator(), object)?),
            om9_core::geometry::Geometry::Surface(_) | om9_core::geometry::Geometry::Curve(_) => {
                let current = controller.coordinator().world().documents.get(&controller.document().id)
                    .ok_or(Error::Missing("trim edge document"))?;
                if !std::ptr::eq(scene.document.as_ref(), current.as_ref()) { return Err(Error::Stale("trim edge document")); }
                EdgeSource::Document(controller.coordinator().capture_tracked_document(controller.document().id)?)
            }
            _ => return Err(Error::Invalid("edge snap source type")),
        };
        let trim_placement = (matches!(entity.geometry, om9_core::geometry::Geometry::Surface(_))
            && entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)).then_some(placement);
        let path_bytes = source.path.capacity().checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>())
            .ok_or(Error::Overflow("edge snap metadata"))?;
        let baseline = u64::try_from(path_bytes).map_err(|_| Error::Overflow("edge snap metadata"))?
            .checked_mul(2).ok_or(Error::Overflow("edge snap metadata"))?
            .checked_add(64 * 1024).ok_or(Error::Overflow("edge snap metadata"))?;
        if reservation.pending_bytes() < baseline { return Err(Error::Budget("edge snap metadata")); }
        reservation.recheck_pending()?;
        reservation.set_allocated_bytes(baseline)?;
        let limits = *controller.coordinator().limits();
        let tolerance = controller.document().effective_tolerance()?.absolute_mm;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-edge-snap-source".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<EdgeSnapSourceReceipt> {
                let mut admit = |base: u64, live: u64, pending: u64| -> Result<()> {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    let allocated = baseline.checked_add(base).and_then(|n| n.checked_add(live))
                        .ok_or(Error::Overflow("edge snap admission"))?;
                    let required = allocated.checked_add(pending).ok_or(Error::Overflow("edge snap admission"))?;
                    if required > reservation.bytes() { return Err(Error::Budget("edge snap admission")); }
                    reservation.set_allocated_bytes(allocated)?;
                    reservation.recheck_pending()
                };
                let mut centers = Vec::new();
                let mut center_error = None;
                let mut vertices = Vec::new();
                let mut vertex_error = None;
                let curves = match &captured {
                    EdgeSource::Brep(brep) => {
                        let curves = om9_kernel::editing::snap_edges_from_source_admitted(brep.source(), &limits,
                            |live, pending| admit(0, live, pending)).map_err(|error| Error::Io(format!("edge snap source: {error}")))?;
                        let curve_bytes = retained_curve_bytes(&curves, &limits)?;
                        match om9_kernel::editing::circular_edge_centers_from_source_admitted(brep.source(), &limits,
                            |live, pending| admit(curve_bytes, live, pending)) {
                            Ok(value) => centers = value,
                            Err(error) => { if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                                center_error = Some(Error::Io(format!("edge snap centers: {error}"))); admit(curve_bytes, 0, 0)?; }
                        }
                        let center_bytes = (centers.capacity() as u64).checked_mul(std::mem::size_of::<Point3>() as u64)
                            .and_then(|n| n.checked_add(match &center_error { Some(Error::Io(message)) => message.capacity() as u64, _ => 0 }))
                            .ok_or(Error::Overflow("edge snap centers"))?;
                        let base = curve_bytes.checked_add(center_bytes).ok_or(Error::Overflow("edge snap vertices"))?;
                        match om9_kernel::editing::vertices_from_source_admitted(brep.source(), &limits,
                            |live, pending| admit(base, live, pending)) {
                            Ok(value) => vertices = value,
                            Err(error) => { if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                                vertex_error = Some(Error::Io(format!("edge snap vertices: {error}"))); admit(base, 0, 0)?; }
                        }
                        Ok(curves)
                    }
                    EdgeSource::Native(native) => {
                        let text = om9_kernel::brep::from_3dm_source_typed_admitted(native.payload(), tolerance, &limits,
                            |live, pending| admit(0, live, pending))?;
                        let text_bytes = u64::try_from(text.capacity()).map_err(|_| Error::Overflow("edge snap source"))?;
                        let curves = om9_kernel::editing::snap_edges_from_source_admitted(&text, &limits,
                            |live, pending| admit(text_bytes, live, pending)).map_err(|error| Error::Io(format!("edge snap source: {error}")))?;
                        let curve_bytes = retained_curve_bytes(&curves, &limits)?;
                        let base = text_bytes.checked_add(curve_bytes).ok_or(Error::Overflow("edge snap centers"))?;
                        match om9_kernel::editing::circular_edge_centers_from_source_admitted(&text, &limits,
                            |live, pending| admit(base, live, pending)) {
                            Ok(value) => centers = value,
                            Err(error) => { if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                                center_error = Some(Error::Io(format!("edge snap centers: {error}"))); admit(base, 0, 0)?; }
                        }
                        let center_bytes = (centers.capacity() as u64).checked_mul(std::mem::size_of::<Point3>() as u64)
                            .and_then(|n| n.checked_add(match &center_error { Some(Error::Io(message)) => message.capacity() as u64, _ => 0 }))
                            .ok_or(Error::Overflow("edge snap centers"))?;
                        let base = base.checked_add(center_bytes).ok_or(Error::Overflow("edge snap vertices"))?;
                        match om9_kernel::editing::vertices_from_source_admitted(&text, &limits,
                            |live, pending| admit(base, live, pending)) {
                            Ok(value) => vertices = value,
                            Err(error) => { if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                                vertex_error = Some(Error::Io(format!("edge snap vertices: {error}"))); admit(base, 0, 0)?; }
                        }
                        Ok(curves)
                    },
                    EdgeSource::Document(document) => {
                        let entity = super::intersection_snap::document_entity(document, &source)?;
                        if let Geometry::Curve(curve) = &entity.geometry {
                            let mut curves = Vec::new();
                            let mut live = 0u64;
                            copy_worker_curve_parts(curve, &mut curves, &mut live, &limits, 0, &mut admit)?;
                            Ok(curves)
                        } else if let Geometry::Surface(surface) = &entity.geometry
                            && !entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                            use om9_core::surface_kernel::SurfaceIso;
                            let mut curves = Vec::new();
                            let mut live = 0u64;
                            for (iso, fixed_axis) in [(SurfaceIso::U, 1usize), (SurfaceIso::V, 0usize)] {
                                for side in 0..if surface.periodic[fixed_axis] { 1 } else { 2 } {
                                    let planned = surface.size[0].checked_add(surface.size[1])
                                        .and_then(|n| n.checked_add(surface.knots[0].len()))
                                        .and_then(|n| n.checked_add(surface.knots[1].len()))
                                        .and_then(|n| n.checked_mul(128)).and_then(|n| n.checked_add(65536))
                                        .ok_or(Error::Overflow("surface snap boundary"))?;
                                    admit(live, 0, u64::try_from(planned).map_err(|_| Error::Overflow("surface snap boundary"))?)?;
                                    let curve = Curve::Nurbs(om9_core::surface_kernel::isocurve(surface, iso,
                                        surface.domain[fixed_axis][side], &limits)?);
                                    live = live.checked_add(curve.validate(&limits)?.bytes)
                                        .and_then(|n| n.checked_add(std::mem::size_of::<Curve>() as u64 * 2))
                                        .ok_or(Error::Overflow("surface snap boundary"))?;
                                    admit(live, 0, 0)?;
                                    curves.try_reserve(1).map_err(|_| Error::Budget("surface snap boundaries"))?;
                                    curves.push(curve);
                                }
                            }
                            Ok(curves)
                        } else {
                        let retained = std::cell::Cell::new(0u64);
                        let mut items = Vec::new();
                        super::wireframe::append_trimmed_surface_edges_at(entity, placement, &mut items, "ExtractWire",
                            &limits, tolerance, &retained, &mut |live, pending| {
                                admit(retained.get(), live, pending)
                            })?;
                        let slots = items.len().checked_mul(std::mem::size_of::<om9_core::geometry::Curve>())
                            .and_then(|bytes| bytes.checked_mul(2)).ok_or(Error::Overflow("trim edge curves"))?;
                        admit(retained.get(), 0, u64::try_from(slots).map_err(|_| Error::Overflow("trim edge curves"))?)?;
                        let mut curves = Vec::new();
                        curves.try_reserve_exact(items.len()).map_err(|_| Error::Budget("trim edge curves"))?;
                        for (_, geometry, _) in items {
                            let om9_core::geometry::Geometry::Curve(curve) = geometry else {
                                return Err(Error::Invalid("trim edge curve"));
                            };
                            curves.push(curve);
                        }
                        Ok(curves)
                        }
                    },
                }.map_err(|error: Error| if worker_cancel.load(Ordering::Acquire) { Error::Canceled }
                    else { Error::Io(format!("edge snap source: {error}")) })?;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let mut bytes = curves.capacity().checked_mul(std::mem::size_of::<om9_core::geometry::Curve>())
                    .ok_or(Error::Overflow("edge snap curves"))?;
                for curve in &curves {
                    let owned = usize::try_from(curve.validate(&limits)?.bytes).map_err(|_| Error::Overflow("edge snap curves"))?;
                    bytes = bytes.checked_add(owned).ok_or(Error::Overflow("edge snap curves"))?;
                }
                bytes = bytes.checked_add(centers.capacity().checked_mul(std::mem::size_of::<Point3>())
                    .ok_or(Error::Overflow("edge snap centers"))?).ok_or(Error::Overflow("edge snap centers"))?;
                if let Some(Error::Io(message)) = &center_error {
                    bytes = bytes.checked_add(message.capacity()).ok_or(Error::Overflow("edge snap center error"))?;
                }
                bytes = bytes.checked_add(vertices.capacity().checked_mul(std::mem::size_of::<Point3>())
                    .ok_or(Error::Overflow("edge snap vertices"))?).ok_or(Error::Overflow("edge snap vertices"))?;
                if let Some(Error::Io(message)) = &vertex_error {
                    bytes = bytes.checked_add(message.capacity()).ok_or(Error::Overflow("edge snap vertex error"))?;
                }
                let retained = baseline.checked_add(u64::try_from(bytes).map_err(|_| Error::Overflow("edge snap curves"))?)
                    .ok_or(Error::Overflow("edge snap curves"))?;
                reservation.set_allocated_bytes(retained)?;
                reservation.shrink_to(retained)?;
                Ok(EdgeSnapSourceReceipt { source, curves, centers, center_error, vertices, vertex_error, tolerance, trim_placement,
                    cancel: Arc::clone(&worker_cancel), _reservation: reservation })
            })).unwrap_or_else(|_| Err(Error::Io("edge snap source worker panicked".into())));
            drop(permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("edge snap worker: {error}")))?;
        Ok(Self { cancel, receiver: Some(receiver), delivered: false })
    }

    pub fn cancel(&self) { self.cancel.store(true, Ordering::Release); }

    pub fn try_result(&mut self) -> Option<Result<EdgeSnapSourceReceipt>> {
        match self.receiver.as_ref()?.try_recv() {
            Ok(result) => { self.receiver = None; self.delivered = true; Some(result) },
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                Some(Err(Error::Io("edge snap worker disconnected".into())))
            },
        }
    }
}

impl Drop for EdgeSnapSourceJob {
    fn drop(&mut self) { if !self.delivered { self.cancel(); } }
}

#[derive(Clone, PartialEq)]
pub(crate) struct EdgeSnapViewInput {
    pub camera: om9_scene::camera::Camera,
    pub viewport: om9_scene::camera::Viewport,
    pub cursor: [f64; 2],
    pub settings: om9_scene::osnap::SnapSettings,
    pub from: Option<om9_core::geometry::Point3>,
    pub cplane: om9_scene::camera::CPlane,
    pub mode: om9_scene::workspace::DisplayMode,
    pub guides: [Option<om9_scene::osnap::SnapGuide>; om9_scene::osnap::MAX_SMARTTRACK_GUIDES],
}

pub(crate) struct EdgeSnapViewReceipt {
    source: Arc<EdgeSnapSourceReceipt>,
    input: EdgeSnapViewInput,
    tolerance: f64,
    selection_revision: u64,
    preview_revision: u64,
    candidates: Vec<om9_scene::osnap::SnapCandidate>,
    cancel: Arc<AtomicBool>,
    _reservation: ExternalMemoryReservation,
}

impl EdgeSnapViewReceipt {
    pub fn resolve<'a>(&'a self, controller: &Controller, input: &EdgeSnapViewInput)
        -> Result<&'a [om9_scene::osnap::SnapCandidate]> {
        if self.cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        if &self.input != input { return Err(Error::Stale("edge snap view")); }
        self.source.validate(controller)?;
        if controller.selection_state().revision != self.selection_revision || controller.preview_revision() != self.preview_revision {
            return Err(Error::Stale("edge snap display context"));
        }
        if controller.document().effective_tolerance()?.absolute_mm.to_bits() != self.tolerance.to_bits() {
            return Err(Error::Stale("edge snap tolerance"));
        }
        Ok(&self.candidates)
    }
}

pub(crate) struct EdgeSnapViewJob {
    cancel: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Result<EdgeSnapViewReceipt>>>,
    delivered: bool,
}

impl EdgeSnapViewJob {
    pub fn required_bytes(source: &EdgeSnapSourceReceipt, input: &EdgeSnapViewInput,
        limits: &om9_core::budget::Limits) -> Result<u64> {
        let kinds = input.settings;
        let directions = usize::from(kinds.is_effective(om9_scene::osnap::SnapKind::Tangent)
            || kinds.is_effective(om9_scene::osnap::SnapKind::Perpendicular)) * 256;
        let per_curve = usize::from(kinds.is_effective(om9_scene::osnap::SnapKind::End)) * 2
            + usize::from(kinds.is_effective(om9_scene::osnap::SnapKind::Mid))
            + usize::from(kinds.is_effective(om9_scene::osnap::SnapKind::Near))
            + usize::from(kinds.is_effective(om9_scene::osnap::SnapKind::Quadrant)) * 4 + directions
            // Line candidates are filtered after their shared solver returns.
            + 4;
        let count = source.curves.len().checked_mul(per_curve)
            .and_then(|count| count.checked_add(if kinds.is_effective(om9_scene::osnap::SnapKind::Center) { source.centers.len() } else { 0 }))
            .and_then(|count| count.checked_add(if kinds.is_effective(om9_scene::osnap::SnapKind::End) { source.vertices.len() } else { 0 }))
            .ok_or(Error::Overflow("edge snap evaluation"))?;
        let count = count.min(usize::try_from(limits.max_outputs).unwrap_or(usize::MAX).saturating_add(per_curve));
        let candidate_bytes = std::mem::size_of::<om9_scene::osnap::SnapCandidate>()
            .checked_add(source.source.path.len().checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>())
                .ok_or(Error::Overflow("edge snap evaluation"))?)
            .ok_or(Error::Overflow("edge snap evaluation"))?;
        let mut curve_scratch = 0u64;
        for curve in &source.curves {
            curve_scratch = curve_scratch.max(curve.validate(limits)?.bytes.checked_mul(8)
                .ok_or(Error::Overflow("edge snap curve scratch"))?);
        }
        let endpoint_witness_bytes = if kinds.is_effective(om9_scene::osnap::SnapKind::End) {
            om9_scene::osnap::END_DIRECTION_SCRATCH_BYTES
        } else { 0 };
        let required = count.checked_mul(candidate_bytes).and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(endpoint_witness_bytes))
            .and_then(|n| n.checked_add(2 * 1024 * 1024)).ok_or(Error::Overflow("edge snap evaluation"))?;
        u64::try_from(required).map_err(|_| Error::Overflow("edge snap evaluation"))?
            .checked_add(curve_scratch).ok_or(Error::Overflow("edge snap evaluation"))
    }

    pub fn start(controller: &Controller, source: Arc<EdgeSnapSourceReceipt>, input: EdgeSnapViewInput,
        mut reservation: ExternalMemoryReservation) -> Result<Self> {
        if reservation.class() != MemoryClass::CpuCache
            || !reservation.belongs_to(&controller.coordinator().external_memory_account()) {
            return Err(Error::Invalid("edge snap evaluation admission"));
        }
        source.validate(controller)?;
        let scene = controller.scene().ok_or(Error::Missing("edge snap scene"))?;
        let object = scene.objects.iter().find(|object| object.root == source.source.root
            && object.path == source.source.path && object.leaf == source.source.leaf && object.source == source.source.stamp)
            .ok_or(Error::Stale("edge snap occurrence"))?;
        let transform = object.display_transform()?;
        let tolerance = controller.document().effective_tolerance()?.absolute_mm;
        let limits = *controller.coordinator().limits();
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        let required = Self::required_bytes(&source, &input, &limits)?;
        if reservation.pending_bytes() < required { return Err(Error::Budget("edge snap evaluation")); }
        reservation.recheck_pending()?;
        let occlusion = if input.mode == om9_scene::workspace::DisplayMode::Shaded {
            Some(controller.capture_surface_occlusion()?.ok_or(Error::Unsupported("edge snap occlusion worker"))?)
        } else { None };
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-edge-snap-view".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<EdgeSnapViewReceipt> {
                reservation.recheck_pending()?;
                if input.settings.is_effective(om9_scene::osnap::SnapKind::End)
                    && let Some(error) = &source.vertex_error
                {
                    return Err(error.clone());
                }
                let mut candidates = om9_scene::osnap::source_curve_candidates(&source.curves, &source.vertices, &source.source, transform,
                    &input.camera, input.viewport, input.cursor, input.settings, input.from, &limits,
                    input.cplane, tolerance, || worker_cancel.load(Ordering::Acquire) || source.cancel.load(Ordering::Acquire))?;
                if input.settings.is_effective(om9_scene::osnap::SnapKind::Center) {
                    if let Some(error) = &source.center_error { return Err(error.clone()); }
                    let centers = om9_scene::osnap::source_point_candidates(&source.centers, om9_scene::osnap::SnapKind::Center, &source.source, transform,
                        &input.camera, input.viewport, input.cursor, input.settings.radius_px,
                        limits.max_outputs.saturating_sub(candidates.len() as u64),
                        || worker_cancel.load(Ordering::Acquire) || source.cancel.load(Ordering::Acquire))?;
                    if candidates.len().saturating_add(centers.len()) as u64 > limits.max_outputs {
                        return Err(Error::Budget("edge snap candidates"));
                    }
                    candidates.try_reserve(centers.len()).map_err(|_| Error::Budget("edge snap candidates"))?;
                    candidates.extend(centers);
                }
                if let Some(occlusion) = &occlusion {
                    let mut failure = None;
                    candidates.retain(|candidate| {
                        if failure.is_some() { return false; }
                        if worker_cancel.load(Ordering::Acquire) { failure = Some(Error::Canceled); return false; }
                        match occlusion.edge_occluded(&input.camera, input.viewport, candidate) {
                            Ok(hidden) => !hidden,
                            Err(error) => { failure = Some(error); false }
                        }
                    });
                    if let Some(error) = failure { return Err(error); }
                }
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let mut bytes = candidates.capacity().checked_mul(std::mem::size_of::<om9_scene::osnap::SnapCandidate>())
                    .ok_or(Error::Overflow("edge snap evaluation receipt"))?;
                for candidate in &candidates {
                    if let Some(identity) = &candidate.source {
                        bytes = bytes.checked_add(identity.path.capacity().checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>())
                            .ok_or(Error::Overflow("edge snap evaluation receipt"))?).ok_or(Error::Overflow("edge snap evaluation receipt"))?;
                    }
                }
                let retained = u64::try_from(bytes).map_err(|_| Error::Overflow("edge snap evaluation receipt"))?
                    .checked_add(64 * 1024).ok_or(Error::Overflow("edge snap evaluation receipt"))?;
                if retained > reservation.bytes() { return Err(Error::Budget("edge snap evaluation receipt")); }
                reservation.set_allocated_bytes(retained)?;
                reservation.shrink_to(retained)?;
                Ok(EdgeSnapViewReceipt { source, input, tolerance, selection_revision, preview_revision,
                    candidates, cancel: Arc::clone(&worker_cancel), _reservation: reservation })
            })).unwrap_or_else(|_| Err(Error::Io("edge snap view worker panicked".into())));
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("edge snap view worker: {error}")))?;
        Ok(Self { cancel, receiver: Some(receiver), delivered: false })
    }
    pub fn cancel(&self) { self.cancel.store(true, Ordering::Release); }
    pub fn try_result(&mut self) -> Option<Result<EdgeSnapViewReceipt>> {
        match self.receiver.as_ref()?.try_recv() {
            Ok(result) => { self.receiver = None; self.delivered = true; Some(result) },
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                Some(Err(Error::Io("edge snap view worker disconnected".into())))
            },
        }
    }
}
impl Drop for EdgeSnapViewJob {
    fn drop(&mut self) { if !self.delivered { self.cancel(); } }
}

/// Latest-view scheduling keeps at most one evaluator running while allowing
/// the UI to replace the queued pointer request.
#[derive(Default)]
pub(crate) struct EdgeSnapViewScheduler {
    active: Option<(Arc<EdgeSnapSourceReceipt>, EdgeSnapViewInput, EdgeSnapViewJob)>,
    pending: Option<(Arc<EdgeSnapSourceReceipt>, EdgeSnapViewInput, ExternalMemoryReservation)>,
    ready: Vec<EdgeSnapViewReceipt>,
    obsolete: bool,
}

impl EdgeSnapViewScheduler {
    pub fn matches_active(&self, source: &Arc<EdgeSnapSourceReceipt>, input: &EdgeSnapViewInput) -> bool {
        !self.obsolete && self.active.as_ref().is_some_and(|(old, old_input, _)| Arc::ptr_eq(old, source) && old_input == input)
            || self.pending.as_ref().is_some_and(|(old, old_input, _)| Arc::ptr_eq(old, source) && old_input == input)
    }
    pub fn request(&mut self, controller: &Controller, source: Arc<EdgeSnapSourceReceipt>,
        input: EdgeSnapViewInput, reservation: ExternalMemoryReservation) -> Result<()> {
        source.validate(controller)?;
        self.ready.retain(|ready| ready.resolve(controller, &input).is_ok());
        if self.ready.iter().any(|ready| Arc::ptr_eq(&ready.source, &source))
            || self.pending.as_ref().is_some_and(|(old, old_input, _)| Arc::ptr_eq(old, &source) && *old_input == input)
            || !self.obsolete && self.pending.is_none() && self.active.as_ref()
                .is_some_and(|(old, old_input, _)| Arc::ptr_eq(old, &source) && *old_input == input) {
            return Ok(());
        }
        if self.ready.len() as u64 >= controller.coordinator().limits().max_outputs { return Err(Error::Budget("edge snap view cache")); }
        self.ready.try_reserve(1).map_err(|_| Error::Budget("edge snap view cache"))?;
        self.pending = Some((source, input, reservation));
        if let Some((_, _, job)) = &self.active {
            job.cancel();
            self.obsolete = true;
        } else {
            self.launch(controller)?;
        }
        Ok(())
    }

    fn launch(&mut self, controller: &Controller) -> Result<()> {
        if self.active.is_some() { return Ok(()); }
        let Some((source, input, reservation)) = self.pending.take() else { return Ok(()) };
        let job = EdgeSnapViewJob::start(controller, Arc::clone(&source), input.clone(), reservation)?;
        self.active = Some((source, input, job));
        self.obsolete = false;
        Ok(())
    }

    pub fn poll(&mut self, controller: &Controller) -> Result<()> {
        let finished = self.active.as_mut().and_then(|(_, _, job)| job.try_result());
        if let Some(result) = finished {
            self.active = None;
            if !self.obsolete {
                let receipt = result?;
                receipt.source.validate(controller)?;
                self.ready.push(receipt);
            }
            self.obsolete = false;
        }
        self.launch(controller)
    }

    pub fn ready<'a>(&'a self, controller: &Controller, source: &Arc<EdgeSnapSourceReceipt>,
        input: &EdgeSnapViewInput) -> Option<&'a [om9_scene::osnap::SnapCandidate]> {
        self.ready.iter().find(|receipt| Arc::ptr_eq(&receipt.source, source))?
            .resolve(controller, input).ok()
    }

    pub fn cancel(&mut self) {
        self.pending = None;
        self.ready.clear();
        if let Some((_, _, job)) = &self.active { job.cancel(); self.obsolete = true; }
    }
    pub fn is_busy(&self) -> bool { self.active.is_some() || self.pending.is_some() }
}

/// One active extraction and a bounded set of retained occurrences. An obsolete native call
/// must exit before the newest queued request is launched.
#[derive(Default)]
pub(crate) struct EdgeSnapSourceCache {
    active: Option<(SnapSource, EdgeSnapSourceJob)>,
    pending: Option<(SnapSource, ExternalMemoryReservation)>,
    ready: Vec<Arc<EdgeSnapSourceReceipt>>,
    obsolete: bool,
}

impl EdgeSnapSourceCache {
    pub fn request(&mut self, controller: &Controller, source: SnapSource,
        reservation: ExternalMemoryReservation) -> Result<()> {
        source.validate(controller.scene().ok_or(Error::Missing("edge snap scene"))?)?;
        self.ready.retain(|ready| ready.validate(controller).is_ok());
        if self.ready.iter().any(|ready| ready.source == source)
            || self.pending.as_ref().is_some_and(|(pending, _)| *pending == source)
            || !self.obsolete && self.pending.is_none()
                && self.active.as_ref().is_some_and(|(active, _)| *active == source) {
            return Ok(());
        }
        if self.ready.len() as u64 >= controller.coordinator().limits().max_outputs { return Err(Error::Budget("edge snap source cache")); }
        self.ready.try_reserve(1).map_err(|_| Error::Budget("edge snap source cache"))?;
        self.pending = Some((source, reservation));
        if let Some((_, active)) = &self.active {
            active.cancel();
            self.obsolete = true;
        } else {
            self.launch(controller)?;
        }
        Ok(())
    }

    fn launch(&mut self, controller: &Controller) -> Result<()> {
        if self.active.is_some() { return Ok(()); }
        if self.pending.is_none() { return Ok(()); }
        // Share the surface worker's slot; keep the latest request queued while busy.
        let permit = match controller.coordinator().external_memory_account().try_acquire_surface_snap() {
            Ok(permit) => permit,
            Err(Error::Invalid("surface snap already active")) => return Ok(()),
            Err(error) => return Err(error),
        };
        let Some((source, reservation)) = self.pending.take() else { return Ok(()) };
        let job = EdgeSnapSourceJob::start(controller, source.clone(), reservation, permit)?;
        self.active = Some((source, job));
        self.obsolete = false;
        Ok(())
    }

    pub fn poll(&mut self, controller: &Controller) -> Result<()> {
        let finished = self.active.as_mut().and_then(|(_, job)| job.try_result());
        if let Some(result) = finished {
            self.active = None;
            if !self.obsolete {
                let receipt = result?;
                receipt.validate(controller)?;
                self.ready.push(Arc::new(receipt));
            }
            self.obsolete = false;
        }
        self.launch(controller)
    }

    pub fn ready(&self, controller: &Controller, source: &SnapSource) -> Option<Arc<EdgeSnapSourceReceipt>> {
        self.ready.iter().find(|receipt| &receipt.source == source && receipt.validate(controller).is_ok()).cloned()
    }

    pub fn cancel(&mut self) {
        self.pending = None;
        self.ready.clear();
        if let Some((_, job)) = &self.active { job.cancel(); self.obsolete = true; }
    }

    pub fn is_busy(&self) -> bool { self.active.is_some() || self.pending.is_some() }
}
