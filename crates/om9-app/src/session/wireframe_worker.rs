// SPDX-License-Identifier: LGPL-2.1-or-later
//! Private source-only ExtractWireframe worker; publication stays on the controller.
use super::{Controller,Error,Result,Geometry,Curve,wireframe};
use om9_core::{budget::{Limits,Usage,memory::MemoryClass},geometry::Transform,
    identity::{SourceStamp,SessionId},transaction::{Cancellation,ExternalMemoryReservation}};
use std::sync::{mpsc,Arc,atomic::{AtomicBool,Ordering}};

type Item=(&'static str,Geometry,Transform);
enum Event {
    Grow { additional:u64,reply:mpsc::SyncSender<Result<ExternalMemoryReservation>> },
    Finished(Result<Output>),
}
struct Output { items:Vec<Item>, _reservation:ExternalMemoryReservation,
    #[cfg(test)] worker_thread:std::thread::ThreadId }
struct WorkerPermit(Arc<AtomicBool>);
impl Drop for WorkerPermit {fn drop(&mut self){self.0.store(false,Ordering::Release);}}

pub(super) struct WireframeJob {
    receiver:mpsc::Receiver<Event>, cancel:Cancellation, delivered:bool,
    authority:SessionId, epoch:u64, publication:uuid::Uuid, stamp:SourceStamp,
    selection_revision:u64, command_run:u64, layer:Option<om9_core::identity::LayerId>,
}
impl Drop for WireframeJob { fn drop(&mut self) {if !self.delivered {self.cancel.cancel();}} }

fn grow(sender:&mpsc::SyncSender<Event>,cancel:&Cancellation,reservation:&mut ExternalMemoryReservation,needed:u64)->Result<()> {
    if needed<=reservation.bytes() {return Ok(());}
    if cancel.is_canceled() {return Err(Error::Canceled);}
    let (reply,receiver)=mpsc::sync_channel(1);
    sender.send(Event::Grow {additional:needed-reservation.bytes(),reply}).map_err(|_|Error::Canceled)?;
    loop {
        if cancel.is_canceled() {return Err(Error::Canceled);}
        match receiver.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(result)=>{let mut extra=result?;return reservation.merge_pending_from(&mut extra);}
            Err(mpsc::RecvTimeoutError::Timeout)=>(),
            Err(mpsc::RecvTimeoutError::Disconnected)=>return Err(Error::Canceled),
        }
    }
}

fn prepare(source:om9_core::transaction::CapturedBrepSource,placement:Transform,
    limits:Limits,mut reservation:ExternalMemoryReservation,cancel:Cancellation,sender:&mpsc::SyncSender<Event>)->Result<Output> {
    let mut admit=|live:u64,pending:u64| -> Result<()> {
        if cancel.is_canceled() {return Err(Error::Canceled);}
        let needed=live.checked_add(pending).ok_or(Error::Overflow("ExtractWire worker memory"))?.max(1);
        grow(sender,&cancel,&mut reservation,needed)?;
        reservation.set_allocated_bytes(live)?;
        reservation.recheck_pending()
    };
    source.validate_canonical_admitted(&limits,&mut admit)?;
    let curves=wireframe::source_wires_with_admission(source.source(),&limits,&mut admit)?;
    let curve_bytes=curves.iter().try_fold(Usage::array::<Curve>(curves.capacity())?.bytes,|sum,curve|
        sum.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("ExtractWire curves")))?;
    let slots=Usage::array::<Item>(curves.len())?.bytes;
    let output_bytes=curve_bytes.checked_mul(2).and_then(|n|n.checked_add(slots))
        .ok_or(Error::Overflow("ExtractWire placement"))?;
    admit(curve_bytes,output_bytes.checked_sub(curve_bytes).ok_or(Error::Overflow("ExtractWire placement"))?)?;
    let mut items=Vec::new();wireframe::reserve_wire(&mut items,curves.len())?;
    let item_slots=Usage::array::<Item>(items.capacity())?.bytes;
    admit(curve_bytes.checked_add(item_slots).ok_or(Error::Overflow("ExtractWire placement"))?,curve_bytes)?;
    let rigid=(0..3).all(|a|(0..3).all(|b| {
        let dot:f64=(0..3).map(|i|placement.0[i][a]*placement.0[i][b]).sum();
        dot.is_finite() && (dot-if a==b {1.} else {0.}).abs()<=1e-10
    }));
    for curve in curves {
        if cancel.is_canceled() {return Err(Error::Canceled);}
        let (curve,placement)=if rigid {(om9_core::top11::bake_curve(&curve,placement)?,Transform::identity())} else {(curve,placement)};
        items.push(("ExtractWire",Geometry::Curve(curve),placement));
    }
    let actual=items.iter().try_fold(item_slots,|sum,(_,geometry,_)| {
        let Geometry::Curve(curve)=geometry else {return Err(Error::Invalid("ExtractWire output"));};
        sum.checked_add(curve.validate(&limits)?.bytes).ok_or(Error::Overflow("ExtractWire output"))
    })?;
    admit(actual,0)?;
    drop(admit);drop(source);
    if cancel.is_canceled() {return Err(Error::Canceled);}
    reservation.shrink_to(actual.max(1))?;
    Ok(Output {items,_reservation:reservation,#[cfg(test)] worker_thread:std::thread::current().id()})
}

impl WireframeJob {
    fn current(&self,c:&Controller)->bool {
        self.authority==c.coordinator.authority() && self.epoch==c.coordinator.epoch()
            && self.command_run==c.command_run && self.selection_revision==c.selection_state().revision
            && c.selection.len()==1 && c.selection.contains(&self.stamp.object)
            && c.document().current_layer==self.layer && c.document().stamp(self.stamp.object).ok()==Some(self.stamp)
            && c.scene().is_some_and(|scene|scene.publication_id()==self.publication)
    }
}

impl Controller {
    pub(crate) fn enable_background_brep_wireframe(&mut self) {self.background_wireframe_enabled=true;}
    /// Start an explicitly polled source-only worker for one selected root BRep.
    /// Until it completes the published document and scene remain unchanged.
    pub fn request_brep_wireframe_worker(&mut self)->Result<()> {
        if self.is_running() {return Err(Error::Busy);}
        if self.selection.len()!=1 {return Err(Error::Invalid("ExtractWireframe requires one selected B-Rep"));}
        let root=*self.selection.iter().next().expect("one selection");
        let document=self.document();
        let entity=document.entities.get(&root).ok_or(Error::Missing("edge extraction source"))?;
        let permissions=document.permissions(root,false)?;
        if !permissions.visible || !permissions.selectable {return Err(Error::Invalid("ExtractWireframe source is not selectable"));}
        if !matches!(entity.geometry,Geometry::Brep(_)) {return Err(Error::Invalid("edge extraction B-Rep"));}
        let placement=entity.placement;placement.validate()?;
        let stamp=document.stamp(root)?;let layer=document.current_layer;
        let scene=self.scene().ok_or(Error::Missing("ExtractWireframe scene"))?;
        if scene.epoch!=self.coordinator.epoch() {return Err(Error::Stale("ExtractWireframe selection"));}
        wireframe::root_scene_has_interior_wires(scene,&stamp)?;
        let publication=scene.publication_id();
        let source=self.coordinator.capture_brep_source(&stamp)?;
        let limits=*self.coordinator.limits();
        let held=self.scene_cache_usage()?;
        let physical=om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes));
        let reservation=self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,1,physical,held)?;
        if self.wireframe_worker_active.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err() {
            return Err(Error::Busy);
        }
        let permit=WorkerPermit(Arc::clone(&self.wireframe_worker_active));
        let cancel=Cancellation::default();let worker_cancel=cancel.clone();
        let (sender,receiver)=mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-extract-wire".into()).spawn(move || {
            let _permit=permit;
            let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
                prepare(source,placement,limits,reservation,worker_cancel,&sender)))
                .unwrap_or_else(|_|Err(Error::Io("ExtractWire worker panicked".into())));
            let _=sender.send(Event::Finished(result));
        }).map_err(|error|Error::Io(format!("ExtractWire worker start: {error}")))?;
        self.wireframe_worker=Some(WireframeJob {receiver,cancel,delivered:false,authority:self.coordinator.authority(),
            epoch:self.coordinator.epoch(),publication,stamp,selection_revision:self.selection_state().revision,
            command_run:self.command_run,layer});
        Ok(())
    }
    pub fn brep_wireframe_worker_pending(&self)->bool {self.wireframe_worker.is_some()}
    pub fn brep_wireframe_worker_active(&self)->bool {self.wireframe_worker_active.load(Ordering::Acquire)}
    /// Nonblocking poll; only a still-current result may commit.
    pub fn poll_brep_wireframe_worker(&mut self)->Option<Result<()>> {
        let mut job=self.wireframe_worker.take()?;
        if !job.current(self) {job.cancel.cancel();return Some(Err(Error::Stale("ExtractWire worker context")));}
        match job.receiver.try_recv() {
            Ok(Event::Grow {additional,reply})=>{
                let result=self.scene_cache_usage().and_then(|held| {
                    let physical=om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes));
                    let batch=additional.max(64*1024);
                    match self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,batch,physical,held) {
                        Err(Error::Budget(_)) if batch>additional=>self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel,additional,physical,held),
                        result=>result,
                    }
                });
                let _=reply.send(result);self.wireframe_worker=Some(job);None
            },
            Ok(Event::Finished(result))=>{
                job.delivered=true;
                Some(result.and_then(|output| {
                    if !job.current(self) {return Err(Error::Stale("ExtractWire worker context"));}
                    #[cfg(test)]
                    assert_ne!(output.worker_thread,std::thread::current().id(),"ExtractWire work must run on the worker thread");
                    let Output {items,_reservation,..}=output;
                    let result=self.commit_placed_geometries(wireframe::EXTRACT_WIRE,"ExtractWire",items.into_iter());
                    drop(_reservation);result
                }))
            },
            Err(mpsc::TryRecvError::Empty)=>{self.wireframe_worker=Some(job);None},
            Err(mpsc::TryRecvError::Disconnected)=>Some(Err(Error::Io("ExtractWire worker exited".into()))),
        }
    }
}
