// SPDX-License-Identifier: LGPL-2.1-or-later
//! Worker for source edits and their dependent Join History results.
//! Join History creation routes remain disabled while broader propagation is incomplete.
use super::{Controller, ReservedCandidateScene};
use om9_core::{Error, Result, budget::{Usage, memory::MemoryClass}, transaction::{Cancellation,
    ExternalMemoryReservation, PreparedTransaction, TransactionPreparationPeer, TransactionRequest}};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};
use std::collections::BTreeSet;
use om9_core::{geometry::{Geometry, Point3}, identity::ObjectId, top11, transaction::Mutation};

enum WorkerRequest {
    Ready(TransactionRequest),
    RingRail(TransactionRequest),
    Mirror { ids: BTreeSet<ObjectId>, brep_deflections: Vec<(ObjectId, f64)>, origin: Point3, normal: Point3, allowance: u64 },
}

struct Owners {
    _scene: ExternalMemoryReservation,
    _request: ExternalMemoryReservation,
    _native_guards: Vec<ExternalMemoryReservation>,
    _native_guard_storage: Option<ExternalMemoryReservation>,
}
struct Permit(Arc<AtomicBool>);
impl Drop for Permit { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }
struct Output { value: Option<OutputValue> }
struct OutputValue {
    prepared: PreparedTransaction,
    candidate: ReservedCandidateScene,
    _input: ExternalMemoryReservation,
    _draft: ExternalMemoryReservation,
    _owners: Owners,
    staging: ExternalMemoryReservation,
    peer: TransactionPreparationPeer,
    _permit: Permit,
}
impl Drop for Output {
 fn drop(&mut self) {
    let Some(OutputValue { prepared, candidate, _input, _draft, _owners, staging, peer, _permit }) = self.value.take() else { return };
    drop((prepared, candidate, _input, _draft, _owners, staging));
    peer.coordinator().prune_orphaned_brep_representations();
    peer.coordinator().prune_orphaned_native_representations();
    drop((peer, _permit));
 }
}
enum Event { SceneStarted, Finished(Result<Output>) }

pub(super) struct JoinHistoryJob {
    cancel: Cancellation,
    receiver: mpsc::Receiver<Event>,
    epoch: u64,
    scene: uuid::Uuid,
    selection: u64,
    sub_revision: u64,
    scene_preparing: bool,
    trim_resume: Option<TrimResume>,
    completion_command: Option<CompletionCommand>,
    ring_rail: Option<(u64, u64)>,
}
enum CompletionCommand {
    Static(&'static str),
    Addon { code: String, _owner: ExternalMemoryReservation },
}
impl CompletionCommand {
    fn name(&self) -> &str { match self { Self::Static(name) => name, Self::Addon { code, .. } => code } }
}
struct TrimResume {
    before: super::top11::Top11Pending,
    after: super::top11::Top11Pending,
    _owner: ExternalMemoryReservation,
}
impl Drop for JoinHistoryJob { fn drop(&mut self) { self.cancel.cancel(); } }

impl Controller {
    pub(crate) fn enable_background_join_history(&mut self) { self.background_join_history = true; }

    pub(super) fn source_edits_update_join_history(&self, ids: &[om9_core::identity::ObjectId]) -> bool {
        self.background_join_history && self.document().history.recipes.values().any(|recipe|
            recipe.command == om9_core::producers::JOIN_HISTORY_COMMAND && recipe.policy.update
                && recipe.state != om9_core::history::EvaluationState::Detached
                && recipe.inputs.iter().any(|input| ids.contains(&input.source.object)))
    }

    pub(super) fn source_mutations_update_join_history(&self, mutations: &[om9_core::transaction::Mutation]) -> bool {
        use om9_core::transaction::Mutation;
        self.background_join_history && mutations.iter().all(|mutation|
            matches!(mutation, Mutation::Geometry { .. } | Mutation::Placement { .. } | Mutation::Insert { .. } | Mutation::Extensions { .. }))
            && self.document().history.recipes.values().any(|recipe|
                recipe.command == om9_core::producers::JOIN_HISTORY_COMMAND && recipe.policy.update
                    && recipe.state != om9_core::history::EvaluationState::Detached
                    && recipe.inputs.iter().any(|input| mutations.iter().any(|mutation| match mutation {
                        Mutation::Geometry { source, .. } | Mutation::Placement { source, .. } => source.object == input.source.object,
                        _ => false,
                    })))
    }
    /// Schedule a source edit and its Join History dependents.
    #[doc(hidden)]
    pub fn request_join_history_edit_worker(&mut self, request: TransactionRequest) -> Result<()> {
        self.request_join_history_edit_worker_with_guards(request, Vec::new(), None)
    }

    pub(super) fn request_ring_rail_history_worker(&mut self, request: TransactionRequest) -> Result<()> {
        let revision = self.builder_revision().ok_or(Error::Missing("Ring Rail builder"))?;
        let run = self.command_run;
        self.request_join_history_worker(WorkerRequest::RingRail(request), Vec::new(), None, None)?;
        let job = self.join_history_worker.as_mut().expect("scheduled Ring Rail");
        job.ring_rail = Some((run, revision));
        job.completion_command = Some(CompletionCommand::Static("RingRail"));
        Ok(())
    }

    pub(super) fn ring_rail_history_pending(&self) -> bool {
        self.join_history_worker.as_ref().is_some_and(|job| job.ring_rail.is_some())
    }

    fn finish_ring_rail_history(&mut self) {
        self.builder = None;
        self.ring_rail_replace = None;
        self.builder_saved.clear();
        self.applied_style = None;
    }

    pub(super) fn request_join_history_mirror(&mut self, ids: &BTreeSet<ObjectId>, origin: Point3, normal: Point3) -> Result<()> {
        let limits = self.coordinator.limits();
        let allowance = ids.iter().try_fold(4096u64, |bytes, id| {
            let geometry = &self.document().entities.get(id).ok_or(Error::Missing("object"))?.geometry;
            let size = geometry.validate(limits)?.bytes;
            bytes.checked_add(size.checked_mul(4).ok_or(Error::Overflow("mirror worker input"))?)
                .and_then(|n| n.checked_add(1024)).ok_or(Error::Overflow("mirror worker input"))
        })?;
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, allowance, physical, held)?;
        let mut brep_deflections = Vec::new();
        for id in ids {
            if matches!(self.document().entities.get(id).map(|entity| &entity.geometry), Some(Geometry::Brep(_))) {
                #[cfg(feature = "occt-brep")]
                brep_deflections.push((*id, self.solid_output_deflection(*id)?));
                #[cfg(not(feature = "occt-brep"))]
                return Err(Error::Unsupported("BRep mirror needs the OCCT kernel feature"));
            }
        }
        self.request_join_history_worker(WorkerRequest::Mirror { ids: ids.clone(), brep_deflections, origin, normal, allowance }, Vec::new(), None, Some(owner))
    }

    pub(super) fn request_join_history_edit_worker_with_guards(&mut self, request: TransactionRequest,
        guards: Vec<ExternalMemoryReservation>, guard_storage: Option<ExternalMemoryReservation>) -> Result<()> {
        self.request_join_history_worker(WorkerRequest::Ready(request), guards, guard_storage, None)
    }

    fn request_join_history_worker(&mut self, request: WorkerRequest,
        guards: Vec<ExternalMemoryReservation>, guard_storage: Option<ExternalMemoryReservation>,
        request_owner: Option<ExternalMemoryReservation>) -> Result<()> {
        let ring_rail = matches!(&request, WorkerRequest::RingRail(_));
        if self.is_running() && (!ring_rail || self.join_history_worker_pending()
            || self.builder.as_ref().is_none_or(|builder| builder.kind() != om9_core::builder::BuilderKind::RingRail)) {
            return Err(Error::Busy);
        }
        if self.join_history_worker_active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return Err(Error::Busy);
        }
        let permit = Permit(self.join_history_worker_active.clone());
        let scene = self.scene.as_ref().ok_or(Error::Missing("history edit scene"))?.publication_id();
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let bytes = held.bytes.checked_add(4096).ok_or(Error::Overflow("history edit scene owner"))?;
        let scene_owner = self.coordinator.reserve_external_memory_with_allocated(
            MemoryClass::CpuCache, bytes, held.bytes, physical, Usage::default())?;
        let limits = *self.coordinator.limits();
        let input_bytes = match &request { WorkerRequest::Ready(request) | WorkerRequest::RingRail(request) => request.worker_input_bytes(&limits)?,
            WorkerRequest::Mirror { allowance, .. } => *allowance };
        let request_owner = match request_owner { Some(owner) => owner, None => self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            input_bytes, physical, held)? };
        let peer = self.coordinator.capture_transaction_preparation_peer(held)?;
        let epoch = self.coordinator.epoch();
        let next_epoch = epoch.checked_add(1).ok_or(Error::Overflow("history edit scene epoch"))?;
        let document = self.document;
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut owners = Owners { _scene: scene_owner, _request: request_owner,
            _native_guards: guards, _native_guard_storage: guard_storage };
        std::thread::Builder::new().name("om9-join-history-edit".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                let coordinator = peer.coordinator();
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                let request = match request {
                    WorkerRequest::Ready(request) | WorkerRequest::RingRail(request) => request,
                    WorkerRequest::Mirror { mut ids, brep_deflections, origin, normal, .. } => {
                        let doc = coordinator.world().documents.get(&document).ok_or(Error::Missing("history edit document"))?;
                        for (id, _) in &brep_deflections { ids.remove(id); }
                        let outcome = top11::mirror_entities(doc, &ids, origin, normal, false, &limits)?;
                        if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                        let mut mutations = Vec::with_capacity(outcome.placements.len() + outcome.replacements.len());
                        for (id, placement) in outcome.placements { mutations.push(Mutation::Placement { source: doc.stamp(id)?, placement }); }
                        for (id, geometry) in outcome.replacements { mutations.push(Mutation::Geometry { source: doc.stamp(id)?, geometry }); }
                        #[cfg(feature = "occt-brep")]
                        if !brep_deflections.is_empty() {
                            let bytes = Usage::array::<ExternalMemoryReservation>(brep_deflections.len())?.bytes.max(1);
                            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                            let mut storage = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, Usage::default())?;
                            owners._native_guards.try_reserve_exact(brep_deflections.len()).map_err(|_| Error::Budget("mirror source guards"))?;
                            let actual = Usage::array::<ExternalMemoryReservation>(owners._native_guards.capacity())?.bytes.max(1);
                            if actual > bytes { coordinator.grow_external_memory_with_held(&mut storage, actual - bytes, physical, Usage::default())?; }
                            storage.set_allocated_bytes(actual)?;
                            owners._native_guard_storage = Some(storage);
                            let reflection = om9_kernel::transform_ops::mirror(origin, normal).map_err(|e| Error::Io(format!("BRep mirror: {e}")))?;
                            for (id, deflection) in brep_deflections {
                                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                                let entity = doc.entities.get(&id).ok_or(Error::Missing("BRep"))?;
                                let inverse = om9_kernel::transform_ops::inverse(&entity.placement).map_err(|e| Error::Io(format!("BRep mirror: {e}")))?;
                                let reflected = om9_kernel::transform_ops::compose(&reflection, &entity.placement).map_err(|e| Error::Io(format!("BRep mirror: {e}")))?;
                                let local = om9_kernel::transform_ops::compose(&inverse, &reflected).map_err(|e| Error::Io(format!("BRep mirror: {e}")))?;
                                let source = coordinator.capture_brep_source(&doc.stamp(id)?)?;
                                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                                let mut guard = coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, physical, Usage::default())?;
                                let body = om9_kernel::transform_ops::transform_captured_source_cancellable(&source, &local, deflection, &limits, &worker_cancel, |live, pending| {
                                    let needed = live.checked_add(pending).ok_or(Error::Overflow("mirror native admission"))?.max(1);
                                    if needed > guard.bytes() {
                                        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                                        let extra = needed - guard.bytes();
                                        coordinator.grow_external_memory_with_held(&mut guard, extra, physical, Usage::default())?;
                                    }
                                    guard.set_allocated_bytes(live)?;
                                    guard.recheck_pending()
                                })?;
                                drop(source);
                                let bytes = body.storage_usage(&limits)?.bytes;
                                guard.set_allocated_bytes(bytes)?;
                                guard.shrink_to(bytes)?;
                                let source = coordinator.register_reserved_brep_candidate(body, &mut guard, Usage::default())?;
                                owners._native_guards.push(guard);
                                mutations.push(Mutation::Geometry { source: doc.stamp(id)?, geometry: Geometry::Brep(source) });
                            }
                        }
                        TransactionRequest::new("Mirror", mutations)
                    }
                };
                if request.worker_input_bytes(&limits)? > owners._request.bytes() { return Err(Error::Budget("mirror worker input")); }
                let edit = if ring_rail { coordinator.prepare_dependent_history_edit(request, Usage::default())? }
                    else { coordinator.prepare_join_history_edit(request, Usage::default())? };
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                let draft = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical, Usage::default())?;
                #[cfg(feature = "occt-brep")]
                let registry = om9_kernel::associative::history_registry()?;
                #[cfg(not(feature = "occt-brep"))]
                let registry = om9_core::producers::reference_registry()?;
                let (request, input, draft, mut candidates) = edit.assemble_registered_with_admission(&registry, &limits, &worker_cancel, draft, |reservation, required| {
                    if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                    let extra = required.saturating_sub(reservation.bytes());
                    if extra == 0 { return Ok(()); }
                    let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                    coordinator.grow_external_memory_with_held(reservation, extra, physical, Usage::default())
                })?;
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                let mut prepared = coordinator.prepare_with_held_usage(request, Usage::default())?;
                Controller::stage_builder_representations(coordinator, &mut prepared, &mut candidates, Usage::default())?;
                drop(candidates);
                coordinator.register_prepared_document(&prepared, document)?;
                let staging_bytes = prepared.worker_staging_allowance_bytes().max(1);
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                let staging = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
                    staging_bytes, physical, Usage::default())?;
                coordinator.reserve_prepared_held_usage(&mut prepared, Usage::default())?;
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                sender.send(Event::SceneStarted).map_err(|_| Error::Canceled)?;
                let candidate = Controller::prepare_history_scene(coordinator, prepared.candidate_world(), next_epoch,
                    document, Usage::default(), Usage::default(), None)?;
                #[cfg(feature = "occt-brep")]
                let candidate = super::topology_remesh::prepare_missing_scene_wires(coordinator, candidate, Usage::default())?;
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                Ok((prepared, candidate, input, draft, staging))
            })).unwrap_or_else(|_| Err(Error::Io("Join History worker panicked".into())));
            if result.is_err() {
                peer.coordinator().prune_orphaned_brep_representations();
                peer.coordinator().prune_orphaned_native_representations();
            }
            let result = result.map(|(prepared, candidate, input, draft, staging)| Output { value: Some(OutputValue {
                    prepared, candidate, _input: input, _draft: draft, _owners: owners, staging, peer, _permit: permit,
                }) });
            let _ = sender.send(Event::Finished(result));
        }).map_err(|error| Error::Io(format!("Join History worker start: {error}")))?;
        self.join_history_worker = Some(JoinHistoryJob { cancel, receiver, epoch, scene,
            selection: self.selection_state().revision, sub_revision: self.sub_revision(), scene_preparing: false, trim_resume: None, completion_command: None, ring_rail: None });
        Ok(())
    }

    pub(super) fn resume_trim_after_history(&mut self, before: super::top11::Top11Pending,
        after: super::top11::Top11Pending, owner: ExternalMemoryReservation) {
        self.join_history_worker.as_mut().expect("scheduled History worker").trim_resume = Some(TrimResume { before, after, _owner: owner });
    }

    pub(super) fn mark_history_completion_command(&mut self, command: &'static str) {
        self.join_history_worker.as_mut().expect("scheduled History worker").completion_command = Some(CompletionCommand::Static(command));
    }

    pub(super) fn reserve_addon_history_completion(&self, code: &str) -> Result<(String, ExternalMemoryReservation)> {
        let bytes = (code.len() as u64).checked_add(std::mem::size_of::<String>() as u64)
            .ok_or(Error::Overflow("add-on History completion"))?.max(1);
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, self.scene_cache_usage()?)?;
        let mut copy = String::new();
        copy.try_reserve_exact(code.len()).map_err(|_| Error::Budget("add-on History completion"))?;
        copy.push_str(code);
        let actual = (copy.capacity() as u64).checked_add(std::mem::size_of::<String>() as u64).ok_or(Error::Overflow("add-on History completion"))?;
        if actual > owner.bytes() { self.coordinator.grow_external_memory_with_held(&mut owner, actual - bytes, physical, self.scene_cache_usage()?)?; }
        owner.set_allocated_bytes(actual)?;
        Ok((copy, owner))
    }

    pub(super) fn mark_addon_history_completion(&mut self, code: String, owner: ExternalMemoryReservation) {
        self.join_history_worker.as_mut().expect("scheduled History worker").completion_command = Some(CompletionCommand::Addon { code, _owner: owner });
    }

    #[doc(hidden)]
    pub fn join_history_worker_pending(&self) -> bool { self.join_history_worker.is_some() }
    #[doc(hidden)]
    pub fn join_history_scene_pending(&self) -> bool {
        self.join_history_worker.as_ref().is_some_and(|job| job.scene_preparing)
    }
    #[doc(hidden)]
    pub fn cancel_join_history_worker(&mut self) {
        if let Some(mut job) = self.join_history_worker.take() {
            if job.ring_rail.is_some() { self.finish_ring_rail_history(); }
            if let Some(resume) = job.trim_resume.take() && self.top11_pending.is_none() { self.top11_pending = Some(resume.before); }
            if let Some(command) = &job.completion_command {
                self.events.emit(crate::command_events::EventKind::Finished, command.name(), "History update canceled", Some(crate::command_events::EventOutcome::Cancelled));
            }
        }
    }

    /// The primary coordinator rechecks authority, epoch and retained state at
    /// publication; stale or canceled private candidates are simply discarded.
    #[doc(hidden)]
    pub fn poll_join_history_worker(&mut self) -> Option<Result<()>> {
        let mut job = self.join_history_worker.take()?;
        let result = match job.receiver.try_recv() {
            Ok(Event::SceneStarted) => {
                job.scene_preparing = true;
                self.join_history_worker = Some(job);
                return None;
            }
            Ok(Event::Finished(result)) => result,
            Err(mpsc::TryRecvError::Empty) => { self.join_history_worker = Some(job); return None; }
            Err(mpsc::TryRecvError::Disconnected) => Err(Error::Io("Join History worker exited".into())),
        };
        let result = result.and_then(|mut output| {
            if job.cancel.is_canceled() { return Err(Error::Canceled); }
            if let Some((run, revision)) = job.ring_rail
                && (self.command_run != run || self.builder_revision() != Some(revision)) {
                return Err(Error::Stale("Ring Rail History command context"));
            }
            if job.trim_resume.is_some() && self.top11_pending.is_some() { return Err(Error::Stale("Trim History command context")); }
            if self.coordinator.epoch() != job.epoch || self.scene.as_ref().is_none_or(|scene| scene.publication_id() != job.scene)
                || self.selection_state().revision != job.selection || self.sub_revision() != job.sub_revision {
                return Err(Error::Stale("Join History worker context"));
            }
            let held = match self.scene_cache_usage() {
                Ok(held) => held,
                Err(error) => return Err(error),
            };
            if let Err(error) = self.coordinator.reserve_prepared_held_usage(&mut output.value.as_mut().expect("worker output").prepared, held) {
                return Err(error);
            }
            let OutputValue { prepared, candidate, _input, _draft, _owners, staging, peer, _permit } = output.value.take().expect("worker output");
            drop(staging);
            drop(peer);
            let receipt = match self.coordinator.commit(prepared) {
                Ok(receipt) => receipt,
                Err(error) => {
                    drop((candidate, _input, _draft, _owners, _permit));
                    self.coordinator.prune_orphaned_brep_representations();
                    self.coordinator.prune_orphaned_native_representations();
                    return Err(error);
                }
            };
            let ReservedCandidateScene { scene, buffer_owner, reservation } = candidate;
            if receipt.published { self.publish_owned_scene(scene, buffer_owner); self.sync_published(om9_core::changes::ChangeCause::Edit); }
            drop(reservation);
            Ok(())
        });
        if job.ring_rail.is_some() { self.finish_ring_rail_history(); }
        if let Some(resume) = job.trim_resume.take() && self.top11_pending.is_none() {
            self.top11_pending = Some(if result.is_ok() { resume.after } else { resume.before });
        }
        if let Some(command) = &job.completion_command {
            let (detail, outcome) = if result.is_ok() { ("History update committed", crate::command_events::EventOutcome::Committed) }
                else { ("History update refused", crate::command_events::EventOutcome::Refused) };
            self.events.emit(crate::command_events::EventKind::Finished, command.name(), detail, Some(outcome));
        }
        Some(result)
    }
}
