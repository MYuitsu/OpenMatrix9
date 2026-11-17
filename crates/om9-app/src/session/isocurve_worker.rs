// SPDX-License-Identifier: LGPL-2.1-or-later
//! Source-only Isocurve construction. Publication remains with the controller.
use super::{Controller, Curve, Error, Geometry, Result};
use om9_core::{budget::{memory::MemoryClass, Limits, Usage}, identity::{SessionId, SourceStamp},
    surface_kernel::SurfaceIso, transaction::{Cancellation, CapturedGeometrySource, ExternalMemoryReservation}};
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};

enum Event {
    Grow { additional: u64, reply: mpsc::SyncSender<Result<ExternalMemoryReservation>> },
    Finished(Result<Output>),
}
pub(super) struct Output {
    pub(super) curves: Vec<Curve>,
    pub(super) reservation: ExternalMemoryReservation,
    #[cfg(test)] pub(super) thread: std::thread::ThreadId,
}
struct Permit(Arc<AtomicBool>);
impl Drop for Permit { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }

pub(super) struct IsocurveJob {
    receiver: mpsc::Receiver<Event>, cancel: Cancellation, delivered: bool,
    authority: SessionId, epoch: u64, publication: uuid::Uuid,
    key: (u64, SourceStamp, SurfaceIso, u64), selection_revision: u64, sub_revision: u64,
    command_run: u64, layer: Option<om9_core::identity::LayerId>,
}
impl Drop for IsocurveJob { fn drop(&mut self) { if !self.delivered { self.cancel.cancel(); } } }

fn grow(sender: &mpsc::SyncSender<Event>, cancel: &Cancellation,
    reservation: &mut ExternalMemoryReservation, needed: u64) -> Result<()> {
    if needed <= reservation.bytes() { return Ok(()); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let (reply, receiver) = mpsc::sync_channel(1);
    sender.send(Event::Grow { additional: needed-reservation.bytes(), reply }).map_err(|_| Error::Canceled)?;
    loop {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        match receiver.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(result) => { let mut extra=result?; return reservation.merge_pending_from(&mut extra); }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Canceled),
        }
    }
}

fn prepare(source: CapturedGeometrySource, along_u: bool, value: f64,
    witness: Option<om9_kernel::editing::IsocurveFaceWitness>,
    ray: Option<om9_kernel::editing::IsocurveFaceRay>, tolerance: f64, limits: Limits,
    mut reservation: ExternalMemoryReservation, cancel: Cancellation, sender: &mpsc::SyncSender<Event>) -> Result<Output> {
    let mut admit=|live:u64,pending:u64| {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let needed=live.checked_add(pending).ok_or(Error::Overflow("isocurve worker memory"))?.max(1);
        grow(sender,&cancel,&mut reservation,needed)?;
        reservation.set_allocated_bytes(live)?;
        reservation.recheck_pending()
    };
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let curves=match source {
        CapturedGeometrySource::Brep(source)=>{
            if let Some(ray)=ray {
                om9_kernel::editing::ray_face_isocurve_from_captured_source_cancellable(
                    &source,along_u,value,ray,&limits,&cancel,&mut admit)?
            } else {
                om9_kernel::editing::face_isocurve_from_captured_source_cancellable(
                    &source,along_u,value,witness,&limits,&cancel,&mut admit)?
            }
        }
        CapturedGeometrySource::NativeBrep(source)=>{
            let converted=om9_kernel::brep::from_3dm_source_typed_admitted(
                source.payload(),tolerance,&limits,&mut admit)?;
            drop(source);
            let held=u64::try_from(converted.capacity()).map_err(|_|Error::Overflow("isocurve native source"))?;
            admit(held,0)?;
            let mut refusal=None;
            let mut admit_curves=|live:u64,pending:u64| {
                let result=held.checked_add(live).ok_or(Error::Overflow("isocurve native outputs"))
                    .and_then(|total|admit(total,pending));
                if let Err(error)=&result {refusal=Some(error.clone());}
                result
            };
            let result=if let Some(ray)=ray {
                om9_kernel::editing::ray_face_isocurve_from_source_admitted(
                    &converted,along_u,value,ray,&limits,&mut admit_curves)
            } else {
                om9_kernel::editing::face_isocurve_from_source_admitted(
                    &converted,along_u,value,witness,&limits,&mut admit_curves)
            };
            result.map_err(|error|refusal.unwrap_or_else(||match error {
                om9_kernel::editing::EditingError::Budget(_)=>Error::Budget("bytes"),
                other=>Error::Io(format!("ExtractIsocurve: {other}")),
            }))?
        }
    };
    let bytes=curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,|sum,curve|
        sum.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("isocurve worker outputs")))?;
    admit(bytes,0)?;
    drop(admit);
    if cancel.is_canceled() { return Err(Error::Canceled); }
    reservation.shrink_to(bytes.max(1))?;
    Ok(Output { curves, reservation, #[cfg(test)] thread:std::thread::current().id() })
}

impl IsocurveJob {
    pub(super) fn start(controller:&Controller, active:Arc<AtomicBool>) -> Result<Self> {
        let pending=controller.isocurve_pending.as_ref().ok_or(Error::Missing("isocurve session"))?;
        if pending.surface.is_some() { return Err(Error::Unsupported("isocurve worker source")); }
        let key=controller.isocurve_recovery_key().ok_or(Error::Stale("isocurve worker context"))?;
        let entity=controller.document().resolve(&pending.source)?;
        if entity.placement!=pending.placement {return Err(Error::Stale("isocurve worker source"));}
        if let Some(reservation)=&pending._surface_reservation { reservation.recheck_pending()?; }
        let scene=controller.scene().ok_or(Error::Missing("isocurve scene"))?;
        let publication=scene.publication_id();
        let source=match &entity.geometry {
            Geometry::Brep(_)=>CapturedGeometrySource::Brep(controller.coordinator().capture_brep_source(&pending.source)?),
            Geometry::NativeBrep(_)=>CapturedGeometrySource::NativeBrep(controller.coordinator().capture_native_brep_source(&pending.source)?),
            _=>return Err(Error::Stale("isocurve worker source")),
        };
        let tolerance=controller.document().effective_tolerance()?.absolute_mm;
        let limits=*controller.coordinator().limits();
        let reservation=controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,1,
            om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),controller.scene_cache_usage()?)?;
        if active.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err(Error::Busy);}
        let permit=Permit(active);let cancel=Cancellation::default();let worker_cancel=cancel.clone();
        let (sender,receiver)=mpsc::sync_channel(1);
        let along_u=pending.iso==SurfaceIso::U;let value=pending.value;
        let witness=pending.face.as_ref().map(|(_,w,_)|*w);
        let ray=pending.native_face.as_ref().map(|pick|pick.ray);
        std::thread::Builder::new().name("om9-isocurve-build".into()).spawn(move || {
            let result=prepare(source,along_u,value,witness,ray,tolerance,limits,reservation,worker_cancel,&sender);
            drop(permit);let _=sender.send(Event::Finished(result));
        }).map_err(|error|Error::Io(format!("isocurve worker start: {error}")))?;
        Ok(Self {receiver,cancel,delivered:false,authority:controller.coordinator().authority(),
            epoch:controller.coordinator().epoch(),publication,key,selection_revision:controller.selection_state().revision,
            sub_revision:controller.sub_revision(),command_run:controller.command_run,layer:controller.document().current_layer})
    }

    pub(super) fn current(&self,c:&Controller)->bool {
        self.authority==c.coordinator().authority() && self.epoch==c.coordinator().epoch()
            && self.command_run==c.command_run && self.selection_revision==c.selection_state().revision
            && self.sub_revision==c.sub_revision() && c.document().current_layer==self.layer
            && c.isocurve_recovery_key()==Some(self.key)
            && c.scene().is_some_and(|scene|scene.publication_id()==self.publication)
    }
    pub(super) fn cancel(&self) { self.cancel.cancel(); }
    pub(super) fn poll(&mut self,c:&Controller)->Option<Result<Output>> {
        if self.cancel.is_canceled() {return Some(Err(Error::Canceled));}
        if !self.current(c) {self.cancel();return Some(Err(Error::Stale("isocurve worker context")));}
        match self.receiver.try_recv() {
            Ok(Event::Grow {additional,reply})=>{
                let result=c.scene_cache_usage().and_then(|held| {
                    let physical=om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes));
                    let batch=additional.max(64*1024);
                    match c.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,batch,physical,held) {
                        Err(Error::Budget(_)) if batch>additional=>c.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,additional,physical,held),
                        result=>result,
                    }
                });
                let _=reply.send(result);None
            },
            Ok(Event::Finished(result))=>{self.delivered=true;Some(result)},
            Err(mpsc::TryRecvError::Empty)=>None,
            Err(mpsc::TryRecvError::Disconnected)=>Some(Err(Error::Io("isocurve worker exited".into()))),
        }
    }
}

impl Controller {
    pub(crate) fn enable_background_isocurve_build(&mut self) { self.background_isocurve_build_enabled=true; }
    /// Start explicit background construction for the current B-rep
    /// Isocurve session. The active session is kept intact until publication.
    pub fn request_isocurve_build_worker(&mut self) -> Result<()> {
        if self.isocurve_build_worker.is_some() { return Err(Error::Busy); }
        let job=IsocurveJob::start(self,Arc::clone(&self.isocurve_build_active))?;
        self.isocurve_build_worker=Some(job);
        Ok(())
    }

    pub fn isocurve_build_worker_pending(&self) -> bool { self.isocurve_build_worker.is_some() }
    pub fn isocurve_build_worker_active(&self) -> bool { self.isocurve_build_active.load(Ordering::Acquire) }
    pub fn cancel_isocurve_build_worker(&mut self) { drop(self.isocurve_build_worker.take()); }

    /// Poll without blocking; the caller must separately guard its CMD draft.
    /// Output and its charge remain live until the atomic transaction returns.
    pub fn poll_isocurve_build_worker(&mut self) -> Option<Result<()>> {
        let mut job=self.isocurve_build_worker.take()?;
        match job.poll(self) {
            None=>{self.isocurve_build_worker=Some(job);None}
            Some(result)=>Some(result.and_then(|output| {
                if !job.current(self) {return Err(Error::Stale("isocurve worker context"));}
                #[cfg(test)]
                assert_ne!(output.thread,std::thread::current().id(),"isocurve must be built on its worker");
                let pending=self.isocurve_pending.take().ok_or(Error::Stale("isocurve session"))?;
                let Output {curves,reservation,..}=output;
                let result=self.commit_placed_geometries(super::isocurve::ISOCURVE,"ExtractIsocurve",
                    curves.into_iter().map(|curve|("ExtractIsocurve",Geometry::Curve(curve),pending.placement)));
                drop(reservation);
                if result.is_err() {self.isocurve_pending=Some(pending);}
                result
            })),
        }
    }
}
