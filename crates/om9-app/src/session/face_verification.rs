// SPDX-License-Identifier: LGPL-2.1-or-later
//! Background verification of selected faces against owned scene captures.
use super::{Controller, SelectedSubobject};
use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::Point3, identity::{LayerId, ObjectId, SourceStamp}, transaction::{Cancellation, ExternalMemoryReservation}, Error, Result};
use om9_scene::scene::Subelement;
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};

struct FaceWorkerPermit(Arc<AtomicBool>);
impl Drop for FaceWorkerPermit { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }

pub(super) fn choose_witness(display: &om9_core::brep::BrepDisplay, index: usize,
    cancel: Option<&Cancellation>) -> Result<[Point3; 3]> {
    let mut best: Option<(f64, [Point3; 3])> = None;
    for triangle in &display.triangles {
        if cancel.is_some_and(|token| token.is_canceled()) { return Err(Error::Canceled); }
        if triangle[3] as usize != index { continue; }
        let points = [*display.points.get(triangle[0] as usize).ok_or(Error::Stale("display triangle"))?,
            *display.points.get(triangle[1] as usize).ok_or(Error::Stale("display triangle"))?,
            *display.points.get(triangle[2] as usize).ok_or(Error::Stale("display triangle"))?];
        let a: [f64; 3] = std::array::from_fn(|i| points[1][i] - points[0][i]);
        let b: [f64; 3] = std::array::from_fn(|i| points[2][i] - points[0][i]);
        let cross = [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
        let area2 = cross.iter().map(|v| v*v).sum::<f64>();
        if !area2.is_finite() { return Err(Error::Invalid("display triangle area")); }
        if best.as_ref().is_none_or(|(old, _)| area2 > *old) { best = Some((area2, points)); }
    }
    best.map(|(_, points)| points).ok_or(Error::Stale("display face triangle"))
}

pub struct FaceVerificationJob {
    cancel: Cancellation,
    receiver: mpsc::Receiver<FaceWorkerEvent>,
    delivered: bool,
    authority:om9_core::identity::SessionId,
    epoch:u64,
    publication:uuid::Uuid,
    selection_revision:u64,
    root_selection_revision:u64,
    command_run:u64,
}

enum FaceWorkerEvent {
    Finished(Result<FaceVerificationReceipt>),
    Grow { additional: u64, reply: mpsc::SyncSender<Result<ExternalMemoryReservation>> },
}

fn grow_on_ui(sender: &mpsc::SyncSender<FaceWorkerEvent>, cancel: &Cancellation,
    reservation: &mut ExternalMemoryReservation, needed: u64) -> Result<()> {
    if needed <= reservation.bytes() { return Ok(()); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let additional = needed - reservation.bytes();
    let (reply, receiver) = mpsc::sync_channel(1);
    sender.send(FaceWorkerEvent::Grow { additional, reply }).map_err(|_| Error::Canceled)?;
    loop {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        match receiver.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(result) => { let mut extra = result?; return reservation.merge_pending_from(&mut extra); }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Canceled),
        }
    }
}

pub struct FaceVerificationReceipt {
    entries: Vec<FaceVerificationEntry>,
    selection_revision: u64,
    root_selection_revision: u64,
    command_run: u64,
    expected_feature: Option<&'static str>,
    requires_subselection: bool,
    isocurve_bounds: Option<[[f64; 2]; 2]>,
    isocurve_ray_hit: Option<([[f64; 2]; 2], Point3)>,
    isocurve_root_bounds: Option<(SourceStamp, uuid::Uuid, [[f64; 2]; 2])>,
    shell_output: Option<om9_core::brep::Brep>,
    shell_thickness: Option<f64>,
    extract_output: Option<ExtractOutputBatch>,
    extract_options: Option<ExtractOptions>,
    #[cfg(test)]
    shell_worker_thread: Option<std::thread::ThreadId>,
    #[cfg(test)]
    extract_worker_thread: Option<std::thread::ThreadId>,
    #[cfg(test)]
    isocurve_bounds_thread: Option<std::thread::ThreadId>,
    cancel: Cancellation,
    reservation: ExternalMemoryReservation,
}

fn register_worker_output(coordinator: &om9_core::transaction::TransactionCoordinator,
    output: om9_core::brep::Brep, reservation: &mut ExternalMemoryReservation,
    held: om9_core::budget::Usage) -> Result<(om9_core::brep::CanonicalBrep, ExternalMemoryReservation)> {
    let bytes = output.storage_usage(coordinator.limits())?.bytes;
    let allocated = reservation.bytes().checked_sub(reservation.pending_bytes())
        .ok_or(Error::Invalid("face worker output reservation"))?;
    if allocated < bytes { return Err(Error::Invalid("face worker output reservation")); }
    let mut source_guard = reservation.split_off(bytes)?;
    source_guard.set_allocated_bytes(bytes)?;
    let source = coordinator.register_reserved_brep_candidate(output, &mut source_guard, held)?;
    Ok((source, source_guard))
}

struct FaceVerificationEntry {
    capture: FaceCapture,
    selection: SelectedSubobject,
    index: usize,
    witness: Option<[Point3; 3]>,
}

enum FaceCapture {
    Source { source: om9_core::transaction::CapturedBrepSource, display: om9_core::brep::BrepDisplayPin,
        stamp: SourceStamp, publication: uuid::Uuid, output_deflection: f64 },
    Witness { stamp: SourceStamp, publication: uuid::Uuid },
}
enum IsocurveBoundsTarget {
    Ray(SelectedSubobject, om9_kernel::editing::IsocurveFaceRay),
    Root(SourceStamp),
}
impl FaceCapture {
    fn display(&self) -> Result<&om9_core::brep::BrepDisplay> {
        match self { Self::Source { display, .. } => Ok(display.display()),
            Self::Witness { .. } => Err(Error::Invalid("completed face capture")) }
    }
    fn source_lease(&self) -> Result<&om9_core::transaction::CapturedBrepSource> {
        match self { Self::Source { source, .. } => Ok(source), _ => Err(Error::Invalid("missing face source lease")) }
    }
    fn topology(&self) -> Result<&om9_core::brep::BrepTopology> {
        match self { Self::Source { source, .. } => Ok(source.topology()), Self::Witness { .. } => Err(Error::Invalid("completed face capture")) }
    }
    fn output_deflection(&self) -> Result<f64> {
        match self { Self::Source { output_deflection, .. } => Ok(*output_deflection), Self::Witness { .. } => Err(Error::Invalid("completed face capture")) }
    }
    fn stamp(&self) -> SourceStamp {
        match self { Self::Source { stamp, .. } | Self::Witness { stamp, .. } => *stamp }
    }
    fn publication_id(&self) -> uuid::Uuid {
        match self { Self::Source { publication, .. } | Self::Witness { publication, .. } => *publication }
    }
    fn release_source(&mut self) {
        if let Self::Source { stamp, publication, .. } = self {
            *self = Self::Witness { stamp: *stamp, publication: *publication };
        }
    }
    fn shared_source(&self) -> Option<Self> {
        match self {
            Self::Source { source, display, stamp, publication, output_deflection } => Some(Self::Source {
                source: source.clone(), display: display.clone(), stamp: *stamp, publication: *publication,
                output_deflection: *output_deflection,
            }),
            _ => None,
        }
    }
}

fn capture_source_face(controller: &Controller, scene: &om9_scene::scene::Scene, stamp: &SourceStamp) -> Result<FaceCapture> {
    let current = controller.coordinator().world().documents.get(&stamp.document).ok_or(Error::Stale("face source document"))?;
    if !Arc::ptr_eq(current, &scene.document) { return Err(Error::Stale("face source scene")); }
    let source = controller.coordinator().capture_brep_source(stamp)?;
    let output_deflection = scene.root_brep_output_deflection(stamp)?;
    let display = match scene.root_brep_display_pin(stamp)? {
        Some(pin) => pin,
        None => controller.coordinator().capture_brep_display(stamp, controller.scene_cache_usage()?,
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes)))?,
    };
    Ok(FaceCapture::Source { source, display, stamp: *stamp, publication: scene.publication_id(), output_deflection })
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ExtractOptions { pub copy: bool, pub input_layer: bool, pub current_layer: Option<LayerId> }

struct ExtractOutput { root: ObjectId, face: om9_core::brep::Brep }
enum ExtractRemainder { Keep, Delete, Replace(om9_core::brep::Brep) }
struct ExtractGroup { source: SourceStamp, remainder: ExtractRemainder }
struct ExtractOutputBatch { outputs: Vec<ExtractOutput>, groups: Vec<ExtractGroup> }

pub(super) fn admitted_extract_snapshot(
    cancel: &Cancellation, reservation: &mut ExternalMemoryReservation, metadata: u64, retained: &mut u64,
    limits: &Limits, grow: &mut dyn FnMut(&mut ExternalMemoryReservation, u64) -> Result<()>,
    operation: impl FnOnce(&mut dyn FnMut(u64, u64) -> Result<()>) -> std::result::Result<om9_core::brep::Brep, om9_kernel::editing::EditingError>,
) -> Result<om9_core::brep::Brep> {
    let mut refusal = None;
    let mut admit = |live: u64, pending: u64| {
        let result = (|| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let live = metadata.checked_add(*retained).and_then(|n| n.checked_add(live)).ok_or(Error::Overflow("extract worker memory"))?;
            let needed = live.checked_add(pending).ok_or(Error::Overflow("extract worker memory"))?;
            grow(reservation, needed)?;
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        })();
        if let Err(error) = &result { refusal = Some(error.clone()); }
        result
    };
    let result = operation(&mut admit);
    drop(admit);
    let output = result.map_err(|error| refusal.unwrap_or_else(|| Error::Io(format!("extract surface: {error}"))))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    *retained = retained.checked_add(output.storage_usage(limits)?.bytes).ok_or(Error::Overflow("extract worker output"))?;
    let live = metadata.checked_add(*retained).ok_or(Error::Overflow("extract worker output"))?;
    if live > reservation.bytes() { return Err(Error::Budget("extract worker output")); }
    reservation.set_allocated_bytes(live)?;
    Ok(output)
}

fn admitted_extract_query<T>(cancel: &Cancellation, reservation: &mut ExternalMemoryReservation, metadata: u64, retained: u64,
    grow: &mut dyn FnMut(&mut ExternalMemoryReservation, u64) -> Result<()>,
    operation: impl FnOnce(&mut dyn FnMut(u64, u64) -> Result<()>) -> std::result::Result<T, om9_kernel::editing::EditingError>,
) -> Result<T> {
    let mut refusal = None;
    let mut admit = |live: u64, pending: u64| {
        let result = (|| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let live = metadata.checked_add(retained).and_then(|n| n.checked_add(live)).ok_or(Error::Overflow("extract worker memory"))?;
            let needed = live.checked_add(pending).ok_or(Error::Overflow("extract worker memory"))?;
            grow(reservation, needed)?; reservation.set_allocated_bytes(live)?; reservation.recheck_pending()
        })();
        if let Err(error) = &result { refusal = Some(error.clone()); }
        result
    };
    let result = operation(&mut admit);
    result.map_err(|error| refusal.unwrap_or_else(|| Error::Io(format!("extract surface source: {error}"))))
}

fn build_extract_batch(entries: &[FaceVerificationEntry], options: ExtractOptions, single_numeric: bool,
    cancel: &Cancellation, reservation: &mut ExternalMemoryReservation, minimum: usize, limits: &Limits, tolerance: f64,
    grow: &mut dyn FnMut(&mut ExternalMemoryReservation, u64) -> Result<()>) -> Result<ExtractOutputBatch> {
    let count = entries.len();
    let extra = std::mem::size_of::<ExtractOutput>().checked_add(std::mem::size_of::<ExtractGroup>())
        .and_then(|n| n.checked_add(std::mem::size_of::<usize>()))
        .and_then(|n| n.checked_add(std::mem::size_of::<[Point3; 3]>()))
        .and_then(|n| n.checked_mul(count)).ok_or(Error::Overflow("extract worker metadata"))?;
    grow(reservation, minimum.checked_add(extra).ok_or(Error::Overflow("extract worker metadata"))? as u64)?;
    let mut outputs = Vec::new(); let mut groups = Vec::new();
    let mut indices = Vec::new(); let mut triangles = Vec::new();
    outputs.try_reserve_exact(count).map_err(|_| Error::Budget("extract outputs"))?;
    groups.try_reserve_exact(count).map_err(|_| Error::Budget("extract groups"))?;
    indices.try_reserve_exact(count).map_err(|_| Error::Budget("extract indices"))?;
    triangles.try_reserve_exact(count).map_err(|_| Error::Budget("extract triangles"))?;
    let metadata = minimum.checked_add(outputs.capacity().checked_mul(std::mem::size_of::<ExtractOutput>()).ok_or(Error::Overflow("extract metadata"))?)
        .and_then(|n| n.checked_add(groups.capacity().checked_mul(std::mem::size_of::<ExtractGroup>())?))
        .and_then(|n| n.checked_add(indices.capacity().checked_mul(std::mem::size_of::<usize>())?))
        .and_then(|n| n.checked_add(triangles.capacity().checked_mul(std::mem::size_of::<[Point3; 3]>())?))
        .ok_or(Error::Overflow("extract metadata"))? as u64;
    grow(reservation, metadata)?;
    let mut retained = 0u64;
    let mut previous_root = None;
    while let Some(first) = entries.iter().filter(|entry| previous_root.is_none_or(|root| entry.selection.root > root))
        .min_by_key(|entry| entry.selection.root) {
        previous_root = Some(first.selection.root);
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let source = first.capture.source_lease()?;
        let witness_deflection = first.capture.display()?.deflection_mm;
        let output_deflection = first.capture.output_deflection()?;
        indices.clear(); triangles.clear();
        for entry in entries.iter().filter(|entry| entry.selection.root == first.selection.root) {
            if entry.selection.source != first.selection.source { return Err(Error::Stale("extract source")); }
            let Subelement::BrepFace(key) = entry.selection.subset else { return Err(Error::Invalid("extract face")); };
            indices.push(source.topology().faces.iter().position(|face| *face == key).ok_or(Error::Stale("extract face"))?);
            triangles.push(entry.witness.ok_or(Error::Stale("extract witness"))?);
        }
        admitted_extract_query(cancel, reservation, metadata, retained, grow, |admit|
            om9_kernel::editing::verify_captured_face_triangles_admitted(source, &triangles, tolerance, witness_deflection, limits, admit))?;
        for (&index, &triangle) in indices.iter().zip(&triangles) {
            let face = admitted_extract_snapshot(cancel, reservation, metadata, &mut retained, limits, grow, |admit|
                om9_kernel::editing::extract_captured_face_admitted(source, index, triangle, tolerance,
                    witness_deflection, output_deflection, limits, admit))?;
            outputs.push(ExtractOutput { root: first.selection.root, face });
        }
        let remainder = if options.copy { ExtractRemainder::Keep } else {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let faces = admitted_extract_query(cancel, reservation, metadata, retained, grow, |admit|
                om9_kernel::editing::captured_source_face_count_admitted(source, limits, admit))?;
            if indices.len() == faces { ExtractRemainder::Delete } else {
                let body = admitted_extract_snapshot(cancel, reservation, metadata, &mut retained, limits, grow, |admit| {
                    if single_numeric {
                        om9_kernel::editing::remove_captured_face_admitted(source, indices[0], triangles[0], tolerance,
                            witness_deflection, output_deflection, limits, admit)
                    } else {
                        om9_kernel::editing::remove_captured_faces_admitted(source, &indices, &triangles, tolerance,
                            witness_deflection, output_deflection, limits, admit)
                    }
                })?;
                ExtractRemainder::Replace(body)
            }
        };
        groups.push(ExtractGroup { source: first.selection.source, remainder });
    }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    reservation.set_allocated_bytes(metadata.checked_add(retained).ok_or(Error::Overflow("extract worker output"))?)?;
    Ok(ExtractOutputBatch { outputs, groups })
}

impl FaceVerificationJob {
    fn start_isocurve_bounds(controller: &Controller, target:IsocurveBoundsTarget,
        mut reservation: ExternalMemoryReservation) -> Result<Self> {
        use om9_core::{geometry::Geometry, transaction::CapturedGeometrySource};
        let scene=controller.scene().ok_or(Error::Missing("isocurve ray scene"))?;
        let stamp=match &target {
            IsocurveBoundsTarget::Ray(selection,_)=>{
                selection.validate(scene,controller.coordinator().epoch())?;
                if !controller.subselection().contains(selection) {return Err(Error::Stale("isocurve ray selection"));}
                selection.source
            }
            IsocurveBoundsTarget::Root(stamp)=>{
                if !controller.selection.contains(&stamp.object) || !controller.subselection().is_empty() {
                    return Err(Error::Stale("isocurve root selection"));
                }
                *stamp
            }
        };
        let entity=controller.document().resolve(&stamp)?;
        let source=match &entity.geometry {
            Geometry::Brep(_)=>CapturedGeometrySource::Brep(controller.coordinator().capture_brep_source(&stamp)?),
            Geometry::NativeBrep(_)=>CapturedGeometrySource::NativeBrep(controller.coordinator().capture_native_brep_source(&stamp)?),
            _=>return Err(Error::Stale("isocurve ray source")),
        };
        let tolerance=controller.document().effective_tolerance()?.absolute_mm;
        let limits=*controller.coordinator().limits();
        let selection_revision=controller.sub_revision();let root_selection_revision=controller.selection_state().revision;
        let command_run=controller.command_run;let publication=scene.publication_id();
        if controller.face_verification_active.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err(Error::Busy);}
        let permit=FaceWorkerPermit(Arc::clone(&controller.face_verification_active));
        let cancel=Cancellation::default();let worker_cancel=cancel.clone();let (sender,receiver)=mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-isocurve-ray".into()).spawn(move || {
            let result=(|| {
                let metadata=(std::mem::size_of::<FaceVerificationReceipt>()+std::mem::size_of::<FaceVerificationEntry>()) as u64;
                grow_on_ui(&sender,&worker_cancel,&mut reservation,metadata)?;
                let mut admit=|live:u64,pending:u64| {
                    if worker_cancel.is_canceled(){return Err(Error::Canceled);}
                    let live=live.checked_add(metadata).ok_or(Error::Overflow("isocurve ray metadata"))?;
                    let needed=live.checked_add(pending).ok_or(Error::Overflow("isocurve ray admission"))?.max(1);
                    grow_on_ui(&sender,&worker_cancel,&mut reservation,needed)?;
                    reservation.set_allocated_bytes(live)?;reservation.recheck_pending()
                };
                let native_source=match &source {
                    CapturedGeometrySource::NativeBrep(native)=>Some(om9_kernel::brep::from_3dm_source_typed_admitted(
                        native.payload(),tolerance,&limits,&mut admit)?),
                    CapturedGeometrySource::Brep(_)=>None,
                };
                let (text,retained)=match (&source,&native_source) {
                    (CapturedGeometrySource::Brep(brep),_)=>{
                        let usage=brep.validate_canonical_admitted(&limits,&mut admit)?;
                        (brep.source(),usage.bytes)
                    }
                    (CapturedGeometrySource::NativeBrep(_),Some(native_source))=>{
                        (native_source.as_str(),u64::try_from(native_source.capacity()).map_err(|_|Error::Overflow("isocurve ray source"))?)
                    }
                    _=>return Err(Error::Stale("isocurve ray source")),
                };
                let scratch=(text.len() as u64).checked_mul(4).and_then(|n|n.checked_add(4*1024*1024))
                    .ok_or(Error::Overflow("isocurve ray scratch"))?;
                admit(retained,scratch)?;
                let (hit,root_bounds)=match &target {
                    IsocurveBoundsTarget::Ray(_,ray)=>(Some(om9_kernel::editing::ray_face_isocurve_hit_with_witness_from_source(text,*ray,&limits)
                        .map_err(|error|Error::Io(format!("ExtractIsocurve: {error}")))?),None),
                    IsocurveBoundsTarget::Root(stamp)=>(None,Some((*stamp,publication,
                        om9_kernel::editing::face_isocurve_bounds_from_source(text,None,&limits)
                            .map_err(|error|Error::Io(format!("ExtractIsocurve: {error}")))?))),
                };
                if worker_cancel.is_canceled(){return Err(Error::Canceled);}
                drop(source);drop(native_source);drop(admit);
                let mut entries=Vec::new();
                if let IsocurveBoundsTarget::Ray(selection,_)=target {
                    entries.try_reserve_exact(1).map_err(|_|Error::Budget("isocurve ray metadata"))?;
                    entries.push(FaceVerificationEntry {capture:FaceCapture::Witness {stamp:selection.source,publication},selection,index:0,witness:None});
                }
                let retained=(std::mem::size_of::<FaceVerificationReceipt>() as u64).checked_add(
                    (entries.capacity() as u64).checked_mul(std::mem::size_of::<FaceVerificationEntry>() as u64)
                    .ok_or(Error::Overflow("isocurve ray metadata"))?).ok_or(Error::Overflow("isocurve ray metadata"))?;
                grow_on_ui(&sender,&worker_cancel,&mut reservation,retained)?;
                reservation.set_allocated_bytes(retained)?;reservation.shrink_to(retained.max(1))?;
                Ok(FaceVerificationReceipt {entries,selection_revision,root_selection_revision,command_run,
                    expected_feature:Some(super::isocurve::ISOCURVE),requires_subselection:hit.is_some(),isocurve_bounds:None,
                    isocurve_ray_hit:hit,isocurve_root_bounds:root_bounds,shell_output:None,shell_thickness:None,extract_output:None,extract_options:None,
                    #[cfg(test)] shell_worker_thread:None,#[cfg(test)] extract_worker_thread:None,
                    #[cfg(test)] isocurve_bounds_thread:Some(std::thread::current().id()),cancel:worker_cancel,reservation})
            })();
            drop(permit);let _=sender.send(FaceWorkerEvent::Finished(result));
        }).map_err(|error|Error::Io(format!("isocurve ray worker: {error}")))?;
        Ok(Self {cancel,receiver,delivered:false,authority:controller.coordinator().authority(),epoch:controller.coordinator().epoch(),
            publication,selection_revision,root_selection_revision,command_run})
    }

    pub fn start(controller: &Controller, selection: SelectedSubobject,
        reservation: ExternalMemoryReservation) -> Result<Self> {
        Self::start_inner(controller, std::slice::from_ref(&selection), reservation, true, None, None, None)
    }

    pub(super) fn start_inner(controller: &Controller, selections: &[SelectedSubobject],
        reservation: ExternalMemoryReservation, requires_subselection: bool, expected_feature: Option<&'static str>, shell_thickness: Option<f64>, extract_options: Option<ExtractOptions>) -> Result<Self> {
        let minimum = selections.len().checked_mul(std::mem::size_of::<FaceVerificationEntry>())
            .and_then(|n| n.checked_add(std::mem::size_of::<FaceVerificationReceipt>()))
            .and_then(|n| n.checked_add(if shell_thickness.is_some() { selections.len().checked_mul(std::mem::size_of::<usize>() + std::mem::size_of::<[Point3; 3]>())? } else { 0 }))
            .ok_or(Error::Overflow("face verification metadata"))?;
        if reservation.class() != MemoryClass::Kernel || !reservation.belongs_to(&controller.coordinator().external_memory_account())
            || selections.is_empty() || selections.len() as u64 > controller.coordinator().limits().max_outputs
            || minimum as u64 > reservation.bytes() {
            return Err(Error::Invalid("face verification reservation"));
        }
        let scene = controller.scene().ok_or(Error::Missing("face verification scene"))?;
        let mut entries: Vec<FaceVerificationEntry> = Vec::new();
        entries.try_reserve_exact(selections.len()).map_err(|_| Error::Budget("face verification metadata"))?;
        for (ordinal, selection) in selections.iter().enumerate() {
            selection.validate(scene, controller.coordinator().epoch())?;
            let isocurve_subselection_only=expected_feature==Some(super::isocurve::ISOCURVE)
                && requires_subselection && controller.selection.is_empty();
            if !selection.path.is_empty() || (!controller.selection.contains(&selection.root) && !isocurve_subselection_only)
                || (requires_subselection && !controller.subselection().contains(selection)) {
                return Err(Error::Stale("face verification selection"));
            }
            if selections[..ordinal].contains(selection) { return Err(Error::Invalid("duplicate face verification selection")); }
            let Subelement::BrepFace(face) = selection.subset else { return Err(Error::Invalid("face verification selection")); };
            let capture = if let Some(capture) = entries.iter().find(|entry| entry.selection.source == selection.source)
                .and_then(|entry| entry.capture.shared_source()) { capture }
            else { capture_source_face(controller, scene, &selection.source)? };
            let index = capture.display()?.topology.faces.iter().position(|id| *id == face).ok_or(Error::Stale("face verification face"))?;
            entries.push(FaceVerificationEntry { capture, selection: selection.clone(), index, witness: None });
        }
        let entry_extra = entries.capacity().saturating_sub(selections.len())
            .checked_mul(std::mem::size_of::<FaceVerificationEntry>()).ok_or(Error::Overflow("face verification metadata"))?;
        let minimum = minimum.checked_add(entry_extra).ok_or(Error::Overflow("face verification metadata"))?;
        if minimum as u64 > reservation.bytes() { return Err(Error::Budget("face verification metadata")); }
        let tolerance = controller.document().tolerance().map_or(1e-6, |value| value.absolute_mm.min(1e-6));
        let limits: Limits = *controller.coordinator().limits();
        let selection_revision = controller.sub_revision();
        let root_selection_revision = controller.selection_state().revision;
        let command_run = controller.command_run;
        if controller.face_verification_active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return Err(Error::Busy);
        }
        let permit = FaceWorkerPermit(Arc::clone(&controller.face_verification_active));
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-face-verification".into()).spawn(move || {
            let result = (|| {
                let mut reservation = reservation;
                let mut grow = |reservation: &mut ExternalMemoryReservation, needed| grow_on_ui(&sender, &worker_cancel, reservation, needed);
                for entry in &mut entries {
                    let display = entry.capture.display()?;
                    let witness = choose_witness(display, entry.index, Some(&worker_cancel))?;
                    match &entry.capture {
                        FaceCapture::Source { source, .. } => om9_kernel::editing::verify_captured_face_display_cancellable(source, display,
                            entry.index, witness, tolerance, &limits, &worker_cancel, |live, pending| {
                                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                                let live = (minimum as u64).checked_add(live).ok_or(Error::Overflow("face worker memory"))?;
                                let needed = live.checked_add(pending).ok_or(Error::Overflow("face worker memory"))?;
                                grow(&mut reservation, needed)?; reservation.set_allocated_bytes(live)?; reservation.recheck_pending()
                            })?,
                        FaceCapture::Witness { .. } => return Err(Error::Invalid("completed face capture")),
                    }
                    entry.witness = Some(witness);
                }
                let isocurve_bounds = if expected_feature == Some(super::isocurve::ISOCURVE) {
                    let entry = entries.first().ok_or(Error::Missing("isocurve face"))?;
                    if entries.len() != 1 { return Err(Error::Invalid("isocurve face count")); }
                    let source = entry.capture.source_lease()?;
                    let display = entry.capture.display()?;
                    let witness = om9_kernel::editing::IsocurveFaceWitness {
                        triangle: entry.witness.ok_or(Error::Stale("isocurve face witness"))?,
                        deflection: display.deflection_mm, tolerance,
                    };
                    let live = source.canonical_storage_bytes()?.checked_add(minimum as u64)
                        .ok_or(Error::Overflow("isocurve bounds admission"))?;
                    let needed = (source.source().len() as u64).checked_mul(4)
                        .and_then(|n| n.checked_add(4 * 1024 * 1024))
                        .and_then(|n| n.checked_add(live))
                        .ok_or(Error::Overflow("isocurve bounds admission"))?;
                    grow(&mut reservation, needed)?;
                    reservation.set_allocated_bytes(live)?;
                    reservation.recheck_pending()?;
                    if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                    let bounds = om9_kernel::editing::face_isocurve_bounds_from_source(source.source(), Some(witness), &limits)
                        .map_err(|error| Error::Io(format!("ExtractIsocurve: {error}")))?;
                    if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                    Some(bounds)
                } else { None };
                let shell_output = if let Some(thickness) = shell_thickness {
                    let first = entries.first().ok_or(Error::Missing("shell face"))?;
                    let mut indices = Vec::new();
                    let mut witnesses = Vec::new();
                    indices.try_reserve_exact(entries.len()).map_err(|_| Error::Budget("shell face indices"))?;
                    witnesses.try_reserve_exact(entries.len()).map_err(|_| Error::Budget("shell face witnesses"))?;
                    let extra = indices.capacity().saturating_sub(entries.len()).checked_mul(std::mem::size_of::<usize>())
                        .and_then(|n| witnesses.capacity().saturating_sub(entries.len()).checked_mul(std::mem::size_of::<[Point3; 3]>())?.checked_add(n))
                        .ok_or(Error::Overflow("shell worker metadata"))?;
                    let metadata = minimum.checked_add(extra).ok_or(Error::Overflow("shell worker metadata"))? as u64;
                    grow(&mut reservation, metadata)?;
                    for entry in &entries {
                        if entry.selection.root != first.selection.root || entry.selection.source != first.selection.source {
                            return Err(Error::Stale("shell face selection"));
                        }
                        let Subelement::BrepFace(key) = entry.selection.subset else { return Err(Error::Invalid("shell face")); };
                        indices.push(first.capture.topology()?.faces.iter().position(|face| *face == key).ok_or(Error::Stale("shell source face"))?);
                        witnesses.push(entry.witness.ok_or(Error::Stale("shell face witness"))?);
                    }
                    let refusal = std::cell::RefCell::new(None);
                    let mut admit = |live: u64, pending: u64| {
                            let result = (|| {
                                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                                let live = live.checked_add(metadata).ok_or(Error::Overflow("shell worker memory"))?;
                                let needed = live.checked_add(pending).ok_or(Error::Overflow("shell worker memory"))?;
                                grow(&mut reservation, needed)?;
                                reservation.set_allocated_bytes(live)?;
                                reservation.recheck_pending()
                            })();
                            if let Err(error) = &result { *refusal.borrow_mut() = Some(error.clone()); }
                            result
                        };
                    let output = match &first.capture {
                        FaceCapture::Source { source, .. } => om9_kernel::editing::shell_captured_faces_with_deflection_admitted(source, &indices, &witnesses, tolerance,
                            first.capture.display()?.deflection_mm, thickness, first.capture.output_deflection()?, &limits, &mut admit),
                        FaceCapture::Witness { .. } => return Err(Error::Invalid("completed face capture")),
                    }.map_err(|error| refusal.into_inner().unwrap_or_else(|| Error::Io(format!("shell: {error}"))))?;
                    if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                    let retained = output.storage_usage(&limits)?.bytes.checked_add(metadata).ok_or(Error::Overflow("shell worker output"))?;
                    if retained > reservation.bytes() { return Err(Error::Budget("shell worker output")); }
                    reservation.set_allocated_bytes(retained)?;
                    Some(output)
                } else { None };
                let extract_output = if let Some(options) = extract_options {
                    Some(build_extract_batch(&entries, options, !requires_subselection, &worker_cancel, &mut reservation, minimum, &limits, tolerance, &mut grow)?)
                } else { None };
                for entry in &mut entries { entry.capture.release_source(); }
                if shell_output.is_none() && extract_output.is_none() { reservation.set_allocated_bytes(minimum as u64)?; }
                let retained = reservation.bytes().checked_sub(reservation.pending_bytes()).ok_or(Error::Overflow("face worker retained"))?;
                reservation.shrink_to(retained.max(1))?;
                Ok(FaceVerificationReceipt { entries, selection_revision, root_selection_revision, command_run, expected_feature, requires_subselection,
                    isocurve_bounds, isocurve_ray_hit:None, isocurve_root_bounds:None, shell_output, shell_thickness, extract_output, extract_options, #[cfg(test)] shell_worker_thread: shell_thickness.map(|_| std::thread::current().id()),
                    #[cfg(test)] extract_worker_thread: extract_options.map(|_| std::thread::current().id()),
                    #[cfg(test)] isocurve_bounds_thread: isocurve_bounds.map(|_| std::thread::current().id()),
                    cancel: worker_cancel, reservation })
            })();
            drop(permit);
            let _ = sender.send(FaceWorkerEvent::Finished(result));
        }).map_err(|error| Error::Io(format!("face verification thread: {error}")))?;
        Ok(Self { cancel, receiver, delivered: false,authority:controller.coordinator().authority(),
            epoch:controller.coordinator().epoch(),publication:scene.publication_id(),selection_revision,root_selection_revision,command_run })
    }
    pub fn cancel(&self) { self.cancel.cancel(); }
    fn context_is_current(&self,controller:&Controller)->bool {
        self.authority==controller.coordinator().authority() && self.epoch==controller.coordinator().epoch()
            && self.selection_revision==controller.sub_revision() && self.root_selection_revision==controller.selection_state().revision
            && self.command_run==controller.command_run
            && controller.scene().is_some_and(|scene|scene.publication_id()==self.publication)
    }
    #[cfg(test)]
    pub(super) fn take_growth_for_test(&self) -> Option<mpsc::SyncSender<Result<ExternalMemoryReservation>>> {
        match self.receiver.try_recv() {
            Ok(FaceWorkerEvent::Grow { reply, .. }) => Some(reply),
            Err(mpsc::TryRecvError::Empty) => None,
            _ => panic!("expected a pending growth request"),
        }
    }
    pub fn try_result(&mut self) -> Option<Result<FaceVerificationReceipt>> {
        match self.receiver.try_recv() {
            Ok(FaceWorkerEvent::Finished(result)) => { self.delivered = true; Some(result) },
            Ok(FaceWorkerEvent::Grow { reply, .. }) => {
                let _ = reply.send(Err(Error::Invalid("face worker growth requires controller polling")));
                None
            },
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Error::Io("face verification thread exited".into()))),
        }
    }
}
impl Drop for FaceVerificationJob { fn drop(&mut self) { if !self.delivered { self.cancel.cancel(); } } }

impl FaceVerificationReceipt {
    pub(super) fn isocurve_root_bounds_for(&self,controller:&Controller,stamp:SourceStamp)->Result<[[f64;2];2]> {
        self.validate(controller)?;
        let (source,_,bounds)=self.isocurve_root_bounds.ok_or(Error::Missing("isocurve root bounds"))?;
        if source!=stamp {return Err(Error::Stale("isocurve root source"));}
        Ok(bounds)
    }
    pub(super) fn isocurve_ray_hit_for(&self,controller:&Controller,selection:&SelectedSubobject)->Result<([[f64;2];2],Point3)> {
        self.validate(controller)?;
        if self.entries.len()!=1 || &self.entries[0].selection!=selection {return Err(Error::Stale("isocurve ray target"));}
        self.isocurve_ray_hit.ok_or(Error::Missing("isocurve ray hit"))
    }
    pub(super) fn isocurve_bounds_for(&self, controller: &Controller, selection: &SelectedSubobject) -> Result<[[f64; 2]; 2]> {
        if self.expected_feature != Some(super::isocurve::ISOCURVE) { return Err(Error::Stale("isocurve bounds receipt")); }
        self.accept_for(controller, selection)?;
        self.isocurve_bounds.ok_or(Error::Missing("isocurve bounds receipt"))
    }
    #[cfg(test)]
    pub(super) fn isocurve_bounds_thread_for_test(&self) -> Option<std::thread::ThreadId> { self.isocurve_bounds_thread }
    #[cfg(test)]
    pub(super) fn shell_output_for_test(&self) -> Option<&om9_core::brep::Brep> { self.shell_output.as_ref() }
    #[cfg(test)]
    pub(super) fn extract_summary_for_test(&self) -> Option<(Vec<&om9_core::brep::Brep>, usize, usize, usize)> {
        let batch = self.extract_output.as_ref()?;
        Some((batch.outputs.iter().map(|output| &output.face).collect(),
            batch.groups.iter().filter(|group| matches!(group.remainder, ExtractRemainder::Keep)).count(),
            batch.groups.iter().filter(|group| matches!(group.remainder, ExtractRemainder::Delete)).count(),
            batch.groups.iter().filter(|group| matches!(group.remainder, ExtractRemainder::Replace(_))).count()))
    }
    pub(super) fn accept_for(&self, controller: &Controller, selection: &SelectedSubobject) -> Result<[Point3; 3]> {
        self.validate(controller)?;
        self.entries.iter().find(|entry| &entry.selection == selection)
            .and_then(|entry| entry.witness).ok_or(Error::Stale("face verification target"))
    }
    /// A receipt is only proof for the still-current publication and selection.
    pub fn accept(&self, controller: &Controller) -> Result<[Point3; 3]> {
        self.validate(controller)?;
        if self.entries.len() != 1 { return Err(Error::Invalid("face verification single receipt")); }
        self.entries[0].witness.ok_or(Error::Stale("face verification witness"))
    }
    fn validate(&self, controller: &Controller) -> Result<()> {
        if self.cancel.is_canceled() { return Err(Error::Canceled); }
        if !self.reservation.belongs_to(&controller.coordinator().external_memory_account())
            || self.selection_revision != controller.sub_revision() || self.root_selection_revision != controller.selection_state().revision
            || self.command_run != controller.command_run {
            return Err(Error::Stale("face verification selection"));
        }
        let scene = controller.scene().ok_or(Error::Missing("face verification scene"))?;
        if let Some((stamp,publication,_))=self.isocurve_root_bounds {
            if scene.publication_id()!=publication || !controller.selection.contains(&stamp.object)
                || !controller.subselection().is_empty() || controller.document().resolve(&stamp).is_err() {
                return Err(Error::Stale("isocurve root publication"));
            }
        }
        for entry in &self.entries {
            let isocurve_subselection_only=self.expected_feature==Some(super::isocurve::ISOCURVE)
                && self.requires_subselection && controller.selection.is_empty();
            if (!controller.selection.contains(&entry.selection.root) && !isocurve_subselection_only)
                || (self.requires_subselection && !controller.subselection().contains(&entry.selection))
                || scene.publication_id() != entry.capture.publication_id() || entry.capture.stamp() != entry.selection.source {
                return Err(Error::Stale("face verification publication"));
            }
            entry.selection.validate(scene, controller.coordinator().epoch())?;
        }
        Ok(())
    }
}

impl Controller {
    pub(super) fn request_isocurve_ray_verification(&mut self,selection:SelectedSubobject,
        ray:om9_kernel::editing::IsocurveFaceRay)->Result<()> {
        if self.face_verification_job.is_some(){return Err(Error::Busy);}
        let held=self.scene_cache_usage()?;
        let physical=om9_native_owner::physical_memory().map(|sample|(sample.total_bytes,sample.available_bytes));
        let reservation=self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,1,physical,held)?;
        let job=FaceVerificationJob::start_isocurve_bounds(self,IsocurveBoundsTarget::Ray(selection,ray),reservation)?;
        self.face_verification_input=Some(super::isocurve::ISOCURVE.to_owned());self.face_verification_job=Some(job);Ok(())
    }
    pub(super) fn request_isocurve_root_verification(&mut self,stamp:SourceStamp)->Result<()> {
        if self.face_verification_job.is_some(){return Err(Error::Busy);}
        let held=self.scene_cache_usage()?;
        let physical=om9_native_owner::physical_memory().map(|sample|(sample.total_bytes,sample.available_bytes));
        let reservation=self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,1,physical,held)?;
        let job=FaceVerificationJob::start_isocurve_bounds(self,IsocurveBoundsTarget::Root(stamp),reservation)?;
        self.face_verification_input=Some(super::isocurve::ISOCURVE.to_owned());self.face_verification_job=Some(job);Ok(())
    }
    pub fn face_verification_pending(&self) -> bool { self.face_verification_job.is_some() }

    #[cfg(test)]
    pub(crate) fn face_verification_worker_active_for_test(&self) -> bool {
        self.face_verification_active.load(Ordering::Acquire)
    }

    pub fn isocurve_face_verification_pending(&self)->bool {
        self.face_verification_job.is_some() && self.face_verification_input.as_deref()==Some(super::isocurve::ISOCURVE)
    }
    pub fn cancel_isocurve_face_verification(&mut self) {
        if self.isocurve_face_verification_pending() {
            drop(self.face_verification_job.take());drop(self.face_verification_input.take());
        }
    }

    pub(super) fn request_face_verification(&mut self, selections: &[SelectedSubobject], input: &str, requires_subselection: bool, feature: &'static str, shell_thickness: Option<f64>, extract_options: Option<ExtractOptions>) -> Result<()> {
        if self.face_verification_job.is_some() { return Err(Error::Busy); }
        let metadata = selections.len().checked_mul(std::mem::size_of::<FaceVerificationEntry>()
            + if shell_thickness.is_some() { std::mem::size_of::<usize>() + std::mem::size_of::<[Point3; 3]>() } else { 0 })
            .and_then(|n| n.checked_add(std::mem::size_of::<FaceVerificationReceipt>()))
            .ok_or(Error::Overflow("face verification metadata"))? as u64;
        let bytes = (self.coordinator().limits().max_bytes / 16).min(16 * 1024 * 1024).max(metadata).max(1);
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, bytes, physical, held)?;
        let job = FaceVerificationJob::start_inner(self, selections, reservation, requires_subselection, Some(feature), shell_thickness, extract_options)?;
        self.face_verification_input = Some(input.to_owned());
        self.face_verification_job = Some(job);
        Ok(())
    }

    /// Poll from the UI without blocking and resume the still-current command.
    pub fn poll_face_verification(&mut self) -> Option<Result<()>> {
        let mut job = self.face_verification_job.take()?;
        if self.face_verification_input.as_deref()==Some(super::isocurve::ISOCURVE) && !job.context_is_current(self) {
            self.face_verification_input.take();job.cancel();return Some(Err(Error::Stale("isocurve face verification")));
        }
        let result = match job.receiver.try_recv() {
            Ok(FaceWorkerEvent::Finished(result)) => { job.delivered = true; result }
            Ok(FaceWorkerEvent::Grow { additional, reply }) => {
                let result = self.scene_cache_usage().and_then(|held| {
                    let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                    let batch = additional.max(64 * 1024);
                    match self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, batch, physical, held) {
                        Err(Error::Budget(_)) if batch > additional => self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, additional, physical, held),
                        result => result,
                    }
                });
                let _ = reply.send(result);
                self.face_verification_job = Some(job);
                return None;
            }
            Err(mpsc::TryRecvError::Empty) => { self.face_verification_job = Some(job); return None; }
            Err(mpsc::TryRecvError::Disconnected) => Err(Error::Io("face verification thread exited".into())),
        };
        let input = self.face_verification_input.take().unwrap_or_default();
        let result = result.and_then(|mut receipt| {
            if receipt.expected_feature==Some(super::isocurve::ISOCURVE) {
                if receipt.isocurve_ray_hit.is_some() || receipt.isocurve_root_bounds.is_some(){receipt.validate(self)?;} else {receipt.accept(self)?;}
                self.face_verification_receipt=Some(receipt);
                let result=self.dispatch_isocurve();
                drop(self.face_verification_receipt.take());
                return result;
            }
            let pending = self.solid_pending.as_ref().ok_or(Error::Stale("face verification command"))?;
            if receipt.expected_feature != Some(pending.feature) {
                return Err(Error::Stale("face verification command"));
            }
            if input.is_empty() {
                if pending.extract_face_picks.len() != receipt.entries.len() { return Err(Error::Stale("face verification command")); }
                for selection in &pending.extract_face_picks { receipt.accept_for(self, selection)?; }
            } else { receipt.accept(self)?; }
            if receipt.expected_feature == Some("OM9-SOLID-029") {
                #[cfg(test)]
                assert!(receipt.shell_worker_thread.is_some_and(|thread| thread != std::thread::current().id()), "Shell must be built on the worker thread");
                let thickness = receipt.shell_thickness.ok_or(Error::Stale("shell thickness"))?;
                let current = if input.is_empty() { pending.shell_thickness }
                    else { input.split_whitespace().nth(1).and_then(|word| word.parse::<f64>().ok()) };
                let current_mm = current.and_then(|value| self.document().page_unit.length_to_mm(value).ok());
                if current_mm != Some(thickness) { return Err(Error::Stale("shell thickness")); }
                let source = receipt.entries.first().ok_or(Error::Stale("shell source"))?.selection.source;
                let root = receipt.entries[0].selection.root;
                let output = receipt.shell_output.take().ok_or(Error::Stale("shell output"))?;
                receipt.entries.clear();
                let held = self.scene_cache_usage()?;
                let (output, mut source_guard) = register_worker_output(&self.coordinator, output, &mut receipt.reservation, held)?;
                let pending = self.solid_pending.take();
                let revision = self.document().revision;
                let result = self.commit_solid_result_with_reserved_sources("OM9-SOLID-029", "Shell", vec![super::Mutation::Geometry {
                    source, geometry: super::Geometry::Brep(output),
                }], Some(vec![root]), std::slice::from_mut(&mut source_guard));
                if result.is_err() && self.document().revision == revision { self.solid_pending = pending; }
                else { self.clear_subselection(); }
                if result.is_err() { self.coordinator.prune_orphaned_brep_representations(); }
                return result;
            }
            if receipt.expected_feature == Some("OM9-SOLID-006") {
                #[cfg(test)]
                assert!(receipt.extract_worker_thread.is_some_and(|thread| thread != std::thread::current().id()), "Extract Surface must be built on the worker thread");
                let options = receipt.extract_options.ok_or(Error::Stale("extract options"))?;
                if options.copy != pending.extract_copy || options.input_layer != pending.extract_input_layer
                    || (!options.input_layer && options.current_layer != self.document().current_layer) {
                    return Err(Error::Stale("extract options"));
                }
                let batch = receipt.extract_output.take().ok_or(Error::Stale("extract output"))?;
                let count = batch.outputs.len();
                let mutation_count = count.checked_add(batch.groups.len()).ok_or(Error::Overflow("extract mutations"))?;
                let scratch = mutation_count.checked_mul(std::mem::size_of::<super::Mutation>())
                    .and_then(|n| n.checked_add(count.checked_mul(std::mem::size_of::<ObjectId>() + "Extract Surface".len())?))
                    .and_then(|n| n.checked_add(mutation_count.checked_mul(std::mem::size_of::<ExternalMemoryReservation>())?))
                    .ok_or(Error::Overflow("extract mutations"))? as u64;
                let held = self.scene_cache_usage()?;
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                let mut scratch_owner = self.coordinator().reserve_external_memory_with_held(MemoryClass::CpuCache, scratch.max(1), physical, held)?;
                let mut mutations = Vec::new(); let mut ids = Vec::new(); let mut source_guards = Vec::new();
                mutations.try_reserve_exact(mutation_count).map_err(|_| Error::Budget("extract mutations"))?;
                ids.try_reserve_exact(count).map_err(|_| Error::Budget("extract outputs"))?;
                source_guards.try_reserve_exact(mutation_count).map_err(|_| Error::Budget("extract source guards"))?;
                let actual = mutations.capacity().checked_mul(std::mem::size_of::<super::Mutation>())
                    .and_then(|n| n.checked_add(ids.capacity().checked_mul(std::mem::size_of::<ObjectId>())?))
                    .and_then(|n| n.checked_add(source_guards.capacity().checked_mul(std::mem::size_of::<ExternalMemoryReservation>())?))
                    .and_then(|n| n.checked_add(count.checked_mul("Extract Surface".len())?))
                    .ok_or(Error::Overflow("extract mutations"))? as u64;
                if actual > scratch_owner.bytes() {
                    self.coordinator.grow_external_memory_with_held(&mut scratch_owner, actual - scratch, physical, held)?;
                }
                for output in batch.outputs {
                    let source = self.document().entities.get(&output.root).ok_or(Error::Stale("extract source"))?;
                    let layer = if options.input_layer { source.attributes.layer } else { options.current_layer };
                    let (face, guard) = register_worker_output(&self.coordinator, output.face, &mut receipt.reservation, held)?;
                    source_guards.push(guard);
                    let mut entity = super::Entity::new("Extract Surface", super::Geometry::Brep(face), layer);
                    entity.placement = source.placement;
                    ids.push(entity.id);
                    mutations.push(super::Mutation::Insert { document: self.document, entity });
                }
                for group in batch.groups {
                    match group.remainder {
                        ExtractRemainder::Keep => (),
                        ExtractRemainder::Delete => mutations.push(super::Mutation::Delete { source: group.source }),
                        ExtractRemainder::Replace(body) => {
                            let (source, guard) = register_worker_output(&self.coordinator, body, &mut receipt.reservation, held)?;
                            source_guards.push(guard);
                            mutations.push(super::Mutation::Geometry { source: group.source, geometry: super::Geometry::Brep(source) });
                        }
                    }
                }
                // The numeric path historically removes or replaces its source first.
                if !receipt.requires_subselection {
                    let source_mutations = mutations.len() - count;
                    mutations.rotate_right(source_mutations);
                    let replacement_guards = source_guards.len() - count;
                    source_guards.rotate_right(replacement_guards);
                }
                receipt.entries.clear();
                let pending = self.solid_pending.take();
                let revision = self.document().revision;
                let result = if source_guards.is_empty() {
                    self.commit_solid_result("OM9-SOLID-006", "Extract Surface", mutations, Some(ids))
                } else {
                    self.commit_solid_result_with_reserved_sources("OM9-SOLID-006", "Extract Surface", mutations, Some(ids), &mut source_guards)
                };
                if result.is_err() && self.document().revision == revision { self.solid_pending = pending; }
                else { self.clear_subselection(); }
                if result.is_err() { self.coordinator.prune_orphaned_brep_representations(); }
                return result;
            }
            self.face_verification_receipt = Some(receipt);
            let result = self.submit(&input);
            drop(self.face_verification_receipt.take());
            result
        });
        Some(result)
    }
}
