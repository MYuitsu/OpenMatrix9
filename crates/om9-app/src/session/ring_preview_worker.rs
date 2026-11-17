// SPDX-License-Identifier: LGPL-2.1-or-later
//! Cancellable ring feature worker. Its result retains the memory lease until dropped.
use om9_core::{
    Error, Result,
    associative::ProducedFeature,
    budget::{Limits, Usage},
    builder::PreparedBuilderRefresh,
    ring_builder::RingBuilderKind,
    transaction::{Cancellation, ExternalMemoryReservation},
};
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};

struct WorkerPermit(Arc<AtomicBool>);
impl Drop for WorkerPermit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

enum Event {
    Grow { additional: u64, reply: mpsc::SyncSender<Result<ExternalMemoryReservation>> },
    Finished(Result<RingPreviewProduct>),
}

/// Geometry and its lease cannot be separated by this staging API.
pub(super) struct RingPreviewProduct {
    prepared: PreparedBuilderRefresh,
    _feature: ProducedFeature,
    _reservation: ExternalMemoryReservation,
}
impl RingPreviewProduct {
    fn matches(&self, session: &om9_core::builder::BuilderSession, document: &om9_core::model::Document) -> bool {
        self.prepared.matches(session, document)
    }
    #[cfg(test)]
    fn prepared(&self) -> &PreparedBuilderRefresh { &self.prepared }
    #[cfg(test)]
    fn feature(&self) -> &ProducedFeature { &self._feature }
    #[cfg(test)]
    fn reserved_bytes(&self) -> u64 { self._reservation.bytes() }
}

pub(super) struct RingPreviewJob {
    receiver: mpsc::Receiver<Event>,
    cancel: Cancellation,
    delivered: bool,
}
impl Drop for RingPreviewJob {
    fn drop(&mut self) { if !self.delivered { self.cancel.cancel(); } }
}

fn grow(sender: &mpsc::SyncSender<Event>, cancel: &Cancellation,
    reservation: &mut ExternalMemoryReservation, needed: u64) -> Result<()> {
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if needed <= reservation.bytes() { return Ok(()); }
    let (reply, receiver) = mpsc::sync_channel(1);
    sender.send(Event::Grow { additional: needed - reservation.bytes(), reply }).map_err(|_| Error::Canceled)?;
    loop {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        match receiver.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(extra) => {
                return reservation.merge_pending_from(&mut extra?);
            },
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Canceled),
        }
    }
}

impl RingPreviewJob {
    /// The caller must reserve and account for the prepared inputless request before spawn.
    pub(super) fn spawn(prepared: PreparedBuilderRefresh, kind: RingBuilderKind, limits: Limits,
        mut reservation: ExternalMemoryReservation, active: Arc<AtomicBool>) -> Result<Self> {
        if reservation.class() != om9_core::budget::memory::MemoryClass::Kernel {
            return Err(Error::Invalid("ring worker reservation class"));
        }
        if prepared.request().command != kind.command() || prepared.request().outputs.len() != 1 {
            return Err(Error::Invalid("ring worker request kind"));
        }
        let request_bytes = prepared.inputless_storage_usage(&limits)?.bytes;
        if request_bytes > reservation.bytes() { return Err(Error::Budget("ring worker request")); }
        reservation.set_allocated_bytes(request_bytes)?;
        reservation.recheck_pending()?;
        let worker_limits = limits.remaining_after(Usage { bytes: request_bytes, ..Usage::default() })?;
        if active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return Err(Error::Busy);
        }
        let permit = WorkerPermit(active);
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-ring-preview".into()).spawn(move || {
            let _permit = permit;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let producer = om9_kernel::associative::RingFamilyBrepProducer { kind };
                let feature = producer.produce_feature_cancellable(prepared.request(), &worker_limits, |live, pending| {
                    let live = request_bytes.checked_add(live).ok_or(Error::Overflow("ring worker live"))?;
                    let needed = live.checked_add(pending).ok_or(Error::Overflow("ring worker pending"))?.max(1);
                    grow(&sender, &worker_cancel, &mut reservation, needed)?;
                    reservation.set_allocated_bytes(live)?;
                    reservation.recheck_pending()
                }, || worker_cancel.is_canceled())?;
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                let retained = reservation.bytes().checked_sub(reservation.pending_bytes())
                    .ok_or(Error::Invalid("ring worker reservation"))?.max(1);
                reservation.shrink_to(retained)?;
                Ok(RingPreviewProduct { prepared, _feature: feature, _reservation: reservation })
            })).unwrap_or_else(|_| Err(Error::Io("ring preview worker panicked".into())));
            let _ = sender.send(Event::Finished(result));
        }).map_err(|error| Error::Io(format!("ring preview worker start: {error}")))?;
        Ok(Self { receiver, cancel, delivered: false })
    }

    pub(super) fn cancel(&self) { self.cancel.cancel(); }

    /// Poll on the controller thread; `reserve` rechecks current document and scene usage.
    pub(super) fn poll(&mut self, mut reserve: impl FnMut(u64) -> Result<ExternalMemoryReservation>)
        -> Option<Result<RingPreviewProduct>> {
        match self.receiver.try_recv() {
            Ok(Event::Grow { additional, reply }) => {
                let batch = additional.max(64 * 1024);
                let result = match reserve(batch) {
                    Err(Error::Budget(_)) if batch > additional => reserve(additional),
                    other => other,
                };
                let _ = reply.send(result);
                None
            }
            Ok(Event::Finished(result)) => {
                self.delivered = true;
                if self.cancel.is_canceled() { drop(result); Some(Err(Error::Canceled)) } else { Some(result) }
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.delivered = true;
                Some(Err(Error::Io("ring preview worker exited".into())))
            }
        }
    }
}

pub(super) struct RingPreviewRequest {
    job: RingPreviewJob,
    authority: om9_core::identity::SessionId,
    epoch: u64,
    publication: uuid::Uuid,
    document: om9_core::identity::DocumentId,
    document_revision: u64,
    context_revision: u64,
    recipe: om9_core::identity::HistoryId,
    builder_revision: u64,
}

pub(super) struct RingPreviewCandidate {
    request: RingPreviewRequest,
    product: RingPreviewProduct,
}

impl RingPreviewRequest {
    fn current(&self, controller: &super::Controller) -> bool {
        let document = controller.document();
        self.authority == controller.coordinator.authority()
            && self.epoch == controller.coordinator.epoch()
            && self.document == document.id
            && self.document_revision == document.revision
            && self.context_revision == document.context_revision
            && controller.scene().is_some_and(|scene| scene.publication_id() == self.publication)
            && controller.builder.as_ref().is_some_and(|session|
                session.recipe() == self.recipe && session.revision() == self.builder_revision)
    }
}

fn held_with_builder_previews(controller: &super::Controller) -> Result<Usage> {
    use om9_core::{associative::Subshape, builder::BuilderHandle, geometry::Geometry};
    let limits = controller.coordinator.limits();
    let mut held = controller.scene_cache_usage()?;
    for session in controller.builder.iter().chain(controller.builder_saved.values()) {
        let Some(preview) = session.preview() else { continue; };
        let Geometry::Brep(brep) = &preview.geometry else { continue; };
        let mut preview_usage = brep.storage_usage(limits)?;
        preview_usage.add(Usage::array::<Subshape>(preview.subshapes.capacity())?, limits)?;
        preview_usage.add(Usage::array::<BuilderHandle>(preview.handles.capacity())?, limits)?;
        let names = preview.subshapes.iter().try_fold(preview.summary.capacity() as u64, |sum, shape|
            sum.checked_add(shape.name.capacity() as u64).ok_or(Error::Overflow("builder preview names")))?;
        preview_usage.add(Usage { bytes: names, ..Usage::default() }, limits)?;
        let covered = brep.allocation_covered_by(&controller.coordinator.external_memory_account(), preview_usage.bytes);
        if !covered { held.add(preview_usage, limits)?; }
    }
    Ok(held)
}

impl super::Controller {
    pub(crate) fn enable_background_ring_preview(&mut self) { self.background_ring_preview_enabled = true; }

    pub(super) fn queue_background_ring_preview(&mut self) -> Result<()> {
        self.cancel_ring_preview_candidate();
        self.builder.as_mut().ok_or(Error::Missing("builder session"))?.clear_preview();
        self.ring_preview_refresh_queued = true;
        self.start_queued_ring_preview()
    }

    fn start_queued_ring_preview(&mut self) -> Result<()> {
        if !self.ring_preview_refresh_queued || self.ring_preview_active.load(Ordering::Acquire) { return Ok(()); }
        match self.request_ring_preview_candidate() {
            Ok(()) => { self.ring_preview_refresh_queued = false; Ok(()) }
            Err(error) => { self.ring_preview_refresh_queued = false; Err(error) }
        }
    }

    pub fn background_ring_preview_pending(&self) -> bool {
        self.ring_preview_refresh_queued || self.ring_preview_request.is_some()
    }
    pub fn background_ring_apply_queued(&self) -> bool { self.ring_preview_apply_queued }

    /// Nonblocking UI poll. Canceled workers retire before the latest edit is started.
    pub fn poll_background_ring_preview(&mut self) -> Option<Result<()>> {
        if let Err(error) = self.start_queued_ring_preview() {
            self.ring_preview_apply_queued = false;
            return Some(Err(error));
        }
        let result = self.poll_ring_preview_candidate()?;
        Some(match result.and_then(|()| self.publish_ring_preview_candidate()) {
            Ok(()) if self.ring_preview_apply_queued => {
                self.ring_preview_apply_queued = false;
                self.builder_apply()
            }
            Ok(()) => Ok(()),
            Err(error) => { self.ring_preview_apply_queued = false; Err(error) }
        })
    }

    /// Start an explicitly polled candidate. The active preview and document stay unchanged.
    pub fn request_ring_preview_candidate(&mut self) -> Result<()> {
        if self.ring_preview_request.is_some() || self.ring_preview_active.load(Ordering::Acquire) {
            return Err(Error::Busy);
        }
        let session = self.builder.as_ref().ok_or(Error::Missing("builder session"))?;
        let kind = session.kind().ring_family().ok_or(Error::Invalid("ring builder"))?;
        let recipe = session.recipe();
        let builder_revision = session.revision();
        let document = self.document();
        let document_id = document.id;
        let document_revision = document.revision;
        let context_revision = document.context_revision;
        let publication = self.scene().ok_or(Error::Missing("builder scene"))?.publication_id();
        let held = held_with_builder_previews(self)?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        self.ring_preview_candidate = None;
        let mut reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel, 64 * 1024, physical, held)?;
        let limits = *self.coordinator.limits();
        let document = &self.coordinator.world().documents[&self.document];
        let prepared = self.builder.as_mut().ok_or(Error::Missing("builder session"))?.prepare_refresh(document)?;
        reservation.set_allocated_bytes(prepared.inputless_storage_usage(&limits)?.bytes)?;
        let job = RingPreviewJob::spawn(prepared, kind, limits, reservation, Arc::clone(&self.ring_preview_active))?;
        self.ring_preview_request = Some(RingPreviewRequest { job, authority: self.coordinator.authority(),
            epoch: self.coordinator.epoch(), publication, document: document_id, document_revision,
            context_revision, recipe, builder_revision });
        Ok(())
    }

    pub fn ring_preview_candidate_pending(&self) -> bool { self.ring_preview_request.is_some() }
    pub fn ring_preview_candidate_ready(&self) -> bool {
        self.ring_preview_candidate.as_ref().is_some_and(|candidate|
            candidate.request.current(self) && self.builder.as_ref().is_some_and(|session|
                candidate.product.matches(session, self.document())))
    }
    pub fn cancel_ring_preview_candidate(&mut self) {
        self.ring_preview_refresh_queued = false;
        self.ring_preview_apply_queued = false;
        if let Some(request) = self.ring_preview_request.take() { request.job.cancel(); }
        self.ring_preview_candidate = None;
    }

    /// Transfer a still-current candidate into the Builder preview. Its BRep retains the
    /// reservation through clones, source transfers and display pins.
    pub fn publish_ring_preview_candidate(&mut self) -> Result<()> {
        use om9_core::{budget::memory::MemoryClass, builder::BuilderHandle, geometry::Geometry};
        let candidate = self.ring_preview_candidate.take().ok_or(Error::Missing("ring preview candidate"))?;
        if !candidate.request.current(self) || !self.builder.as_ref().is_some_and(|session|
            candidate.product.matches(session, self.document())) {
            return Err(Error::Stale("ring preview context"));
        }
        let held = held_with_builder_previews(self)?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut extra = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 64 * 1024, physical, held)?;
        let RingPreviewProduct { prepared, _feature: mut feature, _reservation: mut reservation } = candidate.product;
        let output = feature.outputs.first_mut().ok_or(Error::Invalid("ring preview output"))?;
        if !matches!(&output.geometry, Geometry::Brep(_)) { return Err(Error::Invalid("ring preview BRep")); }
        let mut brep = feature.brep_candidates.pop().ok_or(Error::Missing("ring preview native candidate"))?;
        let display_bytes=brep.display_allocation_bytes()?;
        let topology_bytes=brep.topology_allocation_bytes()?;
        if reservation.pending_bytes()!=0 {return Err(Error::Invalid("ring preview pending output"));}
        let required=display_bytes.checked_add(topology_bytes).and_then(|n|n.checked_add(brep.source_allocation_bytes().ok()?))
            .ok_or(Error::Overflow("ring preview split"))?;
        if required>reservation.bytes() {return Err(Error::Invalid("ring preview split accounting"));}
        let owner_bytes=(std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>() + 2*std::mem::size_of::<usize>()) as u64;
        let mut display_reservation=extra.split_off(owner_bytes)?;
        display_reservation.set_allocated_bytes(owner_bytes)?;
        display_reservation.transfer_allocated_from(&mut reservation,display_bytes)?;
        let mut topology_reservation=extra.split_off(owner_bytes)?;
        topology_reservation.set_allocated_bytes(owner_bytes)?;
        topology_reservation.transfer_allocated_from(&mut reservation,topology_bytes)?;
        let display_owner=Arc::new(std::sync::Mutex::new(display_reservation));
        let topology_owner=Arc::new(std::sync::Mutex::new(topology_reservation));
        let previous_live = reservation.bytes().checked_sub(reservation.pending_bytes())
            .ok_or(Error::Invalid("ring preview reservation"))?;
        let owner = Arc::new(std::sync::Mutex::new(reservation));
        if let Err(error)=brep.bind_split_allocation_owners(owner.clone(),topology_owner.clone(),display_owner.clone()) {
            drop(feature);drop(display_owner);drop(topology_owner);drop(owner);return Err(error);
        }
        output.geometry = Geometry::Brep(self.coordinator.register_brep_candidate(brep)?);
        drop(display_owner);drop(topology_owner);
        {
            let mut guard=owner.lock().unwrap_or_else(|error|error.into_inner());
            guard.merge_pending_from(&mut extra)?;guard.recheck_pending()?;
        }
        let document = &self.coordinator.world().documents[&self.document];
        let session = self.builder.as_mut().ok_or(Error::Missing("builder session"))?;
        session.finish_refresh(document, prepared, feature)?;
        let preview = session.preview().ok_or(Error::Missing("ring preview"))?;
        let extra_live = Usage::array::<BuilderHandle>(preview.handles.capacity())?.bytes
            .checked_add(preview.summary.capacity() as u64)
            .and_then(|v| v.checked_add(owner_bytes))
            .ok_or(Error::Overflow("ring preview owner"));
        let result = extra_live.and_then(|extra_live| {
            let live = previous_live.checked_add(extra_live).ok_or(Error::Overflow("ring preview owner"))?;
            let mut guard = owner.lock().unwrap_or_else(|error| error.into_inner());
            guard.set_allocated_bytes(live)?;
            guard.shrink_to(live)
        });
        if let Err(error) = result { session.clear_preview(); return Err(error); }
        Ok(())
    }

    /// Service growth requests and retain only a candidate matching the current scene and session.
    pub fn poll_ring_preview_candidate(&mut self) -> Option<Result<()>> {
        if self.ring_preview_candidate.is_some() && !self.ring_preview_candidate_ready() {
            self.ring_preview_candidate = None;
        }
        let mut request = self.ring_preview_request.take()?;
        if !request.current(self) { request.job.cancel(); return Some(Err(Error::Stale("ring preview context"))); }
        let result = request.job.poll(|bytes| {
            let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
            self.coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::Kernel, bytes, physical, held_with_builder_previews(self)?)
        });
        match result {
            None => { self.ring_preview_request = Some(request); None }
            Some(result) => Some(result.and_then(|product| {
                if !request.current(self) || !self.builder.as_ref().is_some_and(|session| product.matches(session, self.document())) {
                    return Err(Error::Stale("ring preview context"));
                }
                self.ring_preview_candidate = Some(RingPreviewCandidate { request, product });
                Ok(())
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Controller;
    use om9_core::{budget::memory::MemoryClass, builder::{BuilderKind, BuilderSession}, model::Document};

    fn fixture() -> (Controller, RingPreviewJob) {
        let limits = Limits::default();
        let controller = Controller::new(Document::new("ring worker"), limits).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let mut reservation = controller.coordinator().reserve_external_memory_with_held(
            MemoryClass::Kernel, 64 * 1024, None, held).unwrap();
        reservation.recheck_pending().unwrap();
        let mut session = BuilderSession::new(BuilderKind::SignetRing, controller.document(), &[]).unwrap();
        let request = session.prepare_refresh(controller.document()).unwrap();
        reservation.set_allocated_bytes(request.inputless_storage_usage(&limits).unwrap().bytes).unwrap();
        let job = RingPreviewJob::spawn(request, RingBuilderKind::Signet, limits, reservation, Arc::new(AtomicBool::new(false))).unwrap();
        (controller, job)
    }

    fn wait(controller: &Controller, job: &mut RingPreviewJob) -> Result<RingPreviewProduct> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = job.poll(|bytes| controller.coordinator().reserve_external_memory_with_held(
                MemoryClass::Kernel, bytes, None, controller.scene_cache_usage()?)) {
                return result;
            }
            assert!(std::time::Instant::now() < deadline, "ring worker timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    #[test]
    fn worker_result_keeps_its_reservation_until_dropped() {
        let (controller, mut job) = fixture();
        let product = wait(&controller, &mut job).unwrap();
        assert_eq!(product.feature().outputs.len(), 1);
        assert_eq!(product.prepared().request().outputs.len(), 1);
        assert!(product.reserved_bytes() > 0);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), product.reserved_bytes());
        drop(job);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), product.reserved_bytes());
        drop(product);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn cancellation_releases_the_worker_reservation() {
        let (controller, mut job) = fixture();
        job.cancel();
        assert!(matches!(wait(&controller, &mut job), Err(Error::Canceled)));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn refused_growth_drops_staged_data_and_reservation() {
        let (controller, mut job) = fixture();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let result = loop {
            if let Some(result) = job.poll(|_| Err(Error::Budget("test ring growth"))) { break result; }
            assert!(std::time::Instant::now() < deadline, "ring worker timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        assert!(matches!(result, Err(Error::Budget("test ring growth"))));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn growth_falls_back_to_exact_size_when_batch_is_refused() {
        let (controller, mut running) = fixture();
        running.cancel();
        assert!(matches!(wait(&controller, &mut running), Err(Error::Canceled)));
        let (sender, receiver) = mpsc::sync_channel(1);
        let (reply, answer) = mpsc::sync_channel(1);
        assert!(sender.send(Event::Grow { additional: 1024, reply }).is_ok());
        let mut job = RingPreviewJob { receiver, cancel: Cancellation::default(), delivered: false };
        let mut attempts = Vec::new();
        assert!(job.poll(|bytes| {
            attempts.push(bytes);
            if bytes > 1024 { Err(Error::Budget("test batch")) } else {
                controller.coordinator().reserve_external_memory_with_held(
                    MemoryClass::Kernel, bytes, None, controller.scene_cache_usage()?)
            }
        }).is_none());
        assert_eq!(attempts, [64 * 1024, 1024]);
        drop(answer.recv().unwrap().unwrap());
    }

    #[test]
    fn cancellation_rejects_a_result_already_waiting_in_the_channel() {
        let (controller, mut running) = fixture();
        let product = wait(&controller, &mut running).unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        assert!(sender.send(Event::Finished(Ok(product))).is_ok());
        let mut job = RingPreviewJob { receiver, cancel: Cancellation::default(), delivered: false };
        job.cancel();
        assert!(matches!(job.poll(|_| unreachable!()), Some(Err(Error::Canceled))));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn mismatched_ring_kind_is_refused_before_spawning() {
        let limits = Limits::default();
        let controller = Controller::new(Document::new("wrong ring kind"), limits).unwrap();
        let reservation = controller.coordinator().reserve_external_memory_with_held(
            MemoryClass::Kernel, 64 * 1024, None, controller.scene_cache_usage().unwrap()).unwrap();
        let mut session = BuilderSession::new(BuilderKind::SignetRing, controller.document(), &[]).unwrap();
        let request = session.prepare_refresh(controller.document()).unwrap();
        assert!(matches!(RingPreviewJob::spawn(request, RingBuilderKind::Award, limits, reservation, Arc::new(AtomicBool::new(false))),
            Err(Error::Invalid("ring worker request kind"))));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    fn poll_candidate(controller: &mut Controller) -> Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_ring_preview_candidate() { return result; }
            assert!(std::time::Instant::now() < deadline, "candidate timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    #[test]
    fn controller_retains_only_current_candidate_without_changing_document() {
        let mut controller = Controller::new(Document::new("candidate"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        let before = controller.document().clone();
        controller.request_ring_preview_candidate().unwrap();
        assert!(matches!(controller.request_ring_preview_candidate(), Err(Error::Busy)));
        poll_candidate(&mut controller).unwrap();
        assert!(controller.ring_preview_candidate_ready());
        assert_eq!(controller.document(), &before);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > 0);
        controller.builder.as_mut().unwrap().adjust_field("width_mm", 0.1).unwrap();
        assert!(!controller.ring_preview_candidate_ready());
        assert!(controller.poll_ring_preview_candidate().is_none());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn stale_builder_result_is_canceled_and_never_retained() {
        let mut controller = Controller::new(Document::new("stale candidate"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        controller.builder.as_mut().unwrap().adjust_field("width_mm", 0.1).unwrap();
        assert!(matches!(controller.poll_ring_preview_candidate(), Some(Err(Error::Stale(_)))));
        assert!(!controller.ring_preview_candidate_ready());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while controller.ring_preview_active.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline, "canceled worker did not exit");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn closing_builder_cancels_its_running_candidate() {
        let mut controller = Controller::new(Document::new("closing builder"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        assert!(controller.builder_cancel());
        assert!(!controller.ring_preview_candidate_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while controller.ring_preview_active.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline, "canceled worker did not exit");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
        assert!(controller.document().entities.is_empty());
    }

    #[test]
    fn published_candidate_lease_follows_a_geometry_clone() {
        let mut controller = Controller::new(Document::new("candidate lease"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();
        controller.publish_ring_preview_candidate().unwrap();
        let escaped = controller.builder().unwrap().preview().unwrap().geometry.clone();
        assert!(matches!(&escaped, om9_core::geometry::Geometry::Brep(brep) if brep.has_allocation_owner()));
        let retained = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        assert!(retained > 0);
        let entity = om9_core::model::Entity::new("transient", escaped.clone(), None);
        let request = om9_core::transaction::TransactionRequest::new("insert transient", vec![
            om9_core::transaction::Mutation::Insert { document: controller.document().id, entity },
        ]);
        assert!(matches!(controller.coordinator().prepare(request), Err(Error::Unsupported("transient leased BRep geometry"))));
        assert!(controller.builder_cancel());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), retained);
        drop(escaped);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn published_candidate_source_releases_display_reservation_after_last_display() {
        use om9_core::geometry::Geometry;
        let mut controller=Controller::new(Document::new("split candidate source"),Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();controller.publish_ring_preview_candidate().unwrap();
        let Geometry::Brep(brep)=controller.builder().unwrap().preview().unwrap().geometry.clone() else {panic!("BRep")};
        let display_bytes=brep.display_allocation_bytes().unwrap();let live=brep.display_liveness();
        let owner_bytes=(std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>() + 2*std::mem::size_of::<usize>()) as u64;
        let before=controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        assert!(controller.builder_cancel());assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),before);
        let source=brep.into_owned_source();assert!(!live.is_alive());
        assert_eq!(before-controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),display_bytes+owner_bytes);
        assert!(!source.source().is_empty());drop(source);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),0);
    }

    #[test]
    fn published_candidate_display_pin_releases_source_reservation() {
        use om9_core::geometry::Geometry;
        let mut controller=Controller::new(Document::new("split candidate display"),Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();controller.publish_ring_preview_candidate().unwrap();
        let Geometry::Brep(brep)=controller.builder().unwrap().preview().unwrap().geometry.clone() else {panic!("BRep")};
        let display_bytes=brep.display_allocation_bytes().unwrap();let topology_bytes=brep.topology_allocation_bytes().unwrap();
        let owner_bytes=(std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>() + 2*std::mem::size_of::<usize>()) as u64;
        let source_live=brep.source_liveness();let display_live=brep.display_liveness();let pin=brep.pin_display(Arc::new(()));
        assert!(controller.builder_cancel());drop(brep);
        assert!(!source_live.is_alive());assert!(display_live.is_alive());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),display_bytes+topology_bytes+2*owner_bytes);
        drop(pin);assert!(!display_live.is_alive());assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),0);
    }

    #[test]
    fn leased_candidate_applies_with_undo_and_external_clone() {
        let mut controller = Controller::new(Document::new("candidate apply"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();
        controller.publish_ring_preview_candidate().unwrap();
        let external = controller.builder().unwrap().preview().unwrap().geometry.clone();
        controller.builder_apply().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        assert!(controller.document().entities.values().all(|entity|
            matches!(&entity.geometry, om9_core::geometry::Geometry::Brep(brep) if !brep.has_allocation_owner())));
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > 0);
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        drop(external);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn leased_candidate_updates_a_reopened_history_feature() {
        let mut controller = Controller::new(Document::new("candidate history"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.builder_apply().unwrap();
        let id = *controller.document().entities.keys().next().unwrap();
        let revision = controller.document().entities[&id].geometry_revision;
        controller.begin_builder_from_history(id).unwrap();
        controller.builder_adjust_field("width_mm", 0.1).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();
        controller.publish_ring_preview_candidate().unwrap();
        controller.builder_apply().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        assert_eq!(controller.document().entities[&id].geometry_revision, revision + 1);
        controller.undo().unwrap();
        assert_eq!(controller.document().entities[&id].geometry_revision, revision);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities[&id].geometry_revision, revision + 1);
    }

    #[test]
    fn changed_scene_refuses_leased_apply_and_keeps_the_preview() {
        use om9_core::{geometry::Geometry, model::Entity, transaction::{Mutation, TransactionRequest}};
        let mut controller = Controller::new(Document::new("stale apply"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();
        controller.publish_ring_preview_candidate().unwrap();
        let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        let request = TransactionRequest::new("change", vec![Mutation::Insert { document: controller.document().id, entity }]);
        let prepared = controller.coordinator.prepare(request).unwrap();
        controller.coordinator.commit(prepared).unwrap();
        assert!(matches!(controller.builder_apply(), Err(Error::Stale("command scene"))));
        assert!(controller.builder().unwrap().preview().is_some());
        assert_eq!(controller.document().entities.len(), 1);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > 0);
    }

    #[test]
    fn leased_candidate_respects_history_record_off() {
        let mut controller = Controller::new(Document::new("candidate without history"), Limits::default()).unwrap();
        controller.set_history_record(false).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();
        controller.publish_ring_preview_candidate().unwrap();
        controller.builder_apply().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        assert!(controller.document().history.recipes.is_empty());
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn discarded_staging_keeps_leased_preview_available_for_retry() {
        let mut controller = Controller::new(Document::new("discarded stage"), Limits::default()).unwrap();
        controller.begin_builder(BuilderKind::SignetRing).unwrap();
        controller.request_ring_preview_candidate().unwrap();
        poll_candidate(&mut controller).unwrap();
        controller.publish_ring_preview_candidate().unwrap();
        let prepared = controller.coordinator.prepare_leased_builder(controller.document().id,
            controller.builder().unwrap(), true, true, controller.scene_cache_usage().unwrap()).unwrap();
        drop(prepared);
        assert!(controller.document().entities.is_empty());
        assert!(controller.builder().unwrap().preview().is_some());
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > 0);
        controller.builder_apply().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
    }
}
