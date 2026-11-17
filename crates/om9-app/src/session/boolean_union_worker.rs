//! Source-only union preparation. Publication remains a controller transaction.
use super::Controller;
use om9_core::{
    brep::Brep,
    budget::{Usage, memory::MemoryClass},
    geometry::{Geometry, Transform},
    identity::{LayerId, SessionId, SourceStamp},
    model::Entity,
    transaction::{Cancellation, CapturedBrepSource, ExternalMemoryReservation, Mutation},
    Error, Result,
};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};

enum Event {
    Grow { additional: u64, reply: mpsc::SyncSender<Result<ExternalMemoryReservation>> },
    Finished(Result<Output>),
}
enum Bodies { One(Brep), Many(Vec<Brep>), Empty }
struct Output {
    bodies: Bodies,
    proof: Option<om9_kernel::brep::TopologyAttestation>,
    prepared: Option<(om9_scene::scene::PreparedBrepBuffers, u64)>,
    reservation: ExternalMemoryReservation,
}
struct Permit(Arc<AtomicBool>);
impl Drop for Permit { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }
mod mesh_preview;

pub(super) struct UnionJob {
    receiver: mpsc::Receiver<Event>, cancel: Cancellation,
    authority: SessionId, epoch: u64, publication: uuid::Uuid, command_run: u64,
    selection_revision: u64, layer: Option<LayerId>, stamps: Vec<SourceStamp>,
    placement: Option<Transform>, delete_input: bool, feature: &'static str, target_count: usize,
    preview_mode: Option<usize>, previous_mode: Option<usize>, old_unreserved: Usage,
    batch_capture: bool,
    mesh_preview: Option<mesh_preview::PreviewJob>,
    waiting_output: Option<Result<Output>>,
}
impl Drop for UnionJob { fn drop(&mut self) { self.cancel.cancel(); } }

fn grow(sender: &mpsc::SyncSender<Event>, cancel: &Cancellation,
    reservation: &mut ExternalMemoryReservation, needed: u64) -> Result<()> {
    if needed <= reservation.bytes() { return Ok(()); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let (reply, receiver) = mpsc::sync_channel(1);
    sender.send(Event::Grow { additional: needed - reservation.bytes(), reply }).map_err(|_| Error::Canceled)?;
    loop {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        match receiver.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(result) => { let mut extra = result?; return reservation.merge_pending_from(&mut extra); }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Canceled),
        }
    }
}

fn subtract(a: Usage, b: Usage) -> Result<Usage> {
    Ok(Usage {
        bytes: a.bytes.checked_sub(b.bytes).ok_or(Error::Overflow("split accounting"))?,
        objects: a.objects.checked_sub(b.objects).ok_or(Error::Overflow("split accounting"))?,
        nodes: a.nodes.checked_sub(b.nodes).ok_or(Error::Overflow("split accounting"))?,
        points: a.points.checked_sub(b.points).ok_or(Error::Overflow("split accounting"))?,
        history: a.history.checked_sub(b.history).ok_or(Error::Overflow("split accounting"))?,
        outputs: a.outputs.checked_sub(b.outputs).ok_or(Error::Overflow("split accounting"))?,
    })
}

fn prepare_split(
    inputs: &[(CapturedBrepSource, Transform)], target_count: usize, deflection: f64,
    limits: &om9_core::budget::Limits, cancel: &Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>,
) -> Result<Vec<Brep>> {
    let mut metadata = Usage::array::<Usage>(inputs.len())?;
    metadata.add(Usage::array::<Brep>(inputs.len())?, limits)?;
    let source_bytes = inputs.iter().try_fold(0u64, |sum, (source, _)|
        sum.checked_add(source.canonical_storage_bytes()?).ok_or(Error::Overflow("split sources")))?;
    let requested = source_bytes.checked_add(metadata.bytes).ok_or(Error::Overflow("split input metadata"))?;
    limits.remaining_after(Usage { bytes: requested, ..Usage::default() })?;
    admit(source_bytes, metadata.bytes)?;
    let mut usages = Vec::new();
    usages.try_reserve_exact(inputs.len()).map_err(|_| Error::Budget("split input inventory"))?;
    let mut bodies: Vec<Brep> = Vec::new();
    bodies.try_reserve_exact(inputs.len()).map_err(|_| Error::Budget("split inputs"))?;
    metadata = Usage::array::<Usage>(usages.capacity())?;
    metadata.add(Usage::array::<Brep>(bodies.capacity())?, limits)?;
    let actual = source_bytes.checked_add(metadata.bytes).ok_or(Error::Overflow("split input metadata"))?;
    limits.remaining_after(Usage { bytes: actual, ..Usage::default() })?;
    admit(source_bytes, metadata.bytes)?;
    let mut held = metadata;
    let mut remaining_source_bytes = source_bytes;
    for (source, _) in inputs {
        remaining_source_bytes = remaining_source_bytes.checked_sub(source.canonical_storage_bytes()?)
            .ok_or(Error::Overflow("split source inventory"))?;
        let mut validation_other = held;
        validation_other.add(Usage { bytes: remaining_source_bytes, ..Usage::default() }, limits)?;
        let reduced = limits.remaining_after(validation_other)?;
        let usage = source.validate_canonical_admitted(&reduced, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let total = held.bytes.checked_add(live).and_then(|n| n.checked_add(remaining_source_bytes))
                .ok_or(Error::Overflow("split source admission"))?;
            admit(total, pending)
        })?;
        held.add(usage, limits)?;
        usages.push(usage);
    }
    let source_and_metadata = held;
    for (index, (source, transform)) in inputs.iter().enumerate() {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if let Some(previous) = (0..index).find(|&i| inputs[i].1 == *transform && inputs[i].0.same_canonical_allocations(source)) {
            let mut next = held;
            next.add(bodies[previous].storage_usage(limits)?, limits)?;
            admit(next.bytes, 0)?;
            bodies.push(bodies[previous].clone());
            held = next;
            continue;
        }
        let outside = subtract(held, usages[index])?;
        let reduced = limits.remaining_after(outside)?;
        let body = om9_kernel::transform_ops::transform_captured_source_cancellable(source, transform,
            deflection, &reduced, cancel, |live, pending| {
                admit(outside.bytes.checked_add(live).ok_or(Error::Overflow("split placement admission"))?, pending)
            })?;
        held.add(body.storage_usage(&limits.remaining_after(held)?)?, limits)?;
        bodies.push(body);
    }
    let reduced = limits.remaining_after(source_and_metadata)?;
    let pieces = om9_kernel::editing::split_solid_sets_cancellable_admitted(&bodies[..target_count],
        &bodies[target_count..], deflection, &reduced, cancel, |live, pending| {
            admit(source_and_metadata.bytes.checked_add(live).ok_or(Error::Overflow("split group admission"))?, pending)
        })?;
    if pieces.is_empty() { return Err(Error::Invalid("Boolean Split produced no pieces")); }
    Ok(pieces)
}

impl UnionJob {
    fn start(controller: &Controller, feature: &'static str, targets: &[SourceStamp], delete_input: bool,
        preview_mode: Option<usize>, previous_mode: Option<usize>, old_unreserved: Usage, active: Arc<AtomicBool>) -> Result<Self> {
        let profile = std::env::var_os("OM9_BOOLEAN_REQUEST_PROFILE").map(|_| std::time::Instant::now());
        let checkpoint = |phase: &str| {
            if let Some(started) = profile { eprintln!("boolean_request {phase} cumulative_ms={:.3}", started.elapsed().as_secs_f64()*1000.); }
        };
        let mut selected = controller.selected_ordered();
        let target_count = targets.len();
        if feature == "OM9-SOLID-003" || feature == "OM9-SOLID-004" {
            if feature == "OM9-SOLID-004" && selected.len() != 2 { return Err(Error::Missing("select exactly two B-Rep solids")); }
            if selected.len() < 2 { return Err(Error::Missing("select at least two B-Rep solids")); }
        } else {
            if targets.is_empty() || selected.is_empty() { return Err(Error::Missing("select the second Boolean set")); }
            for target in targets {
                if controller.document().stamp(target.object).ok().as_ref() != Some(target) { return Err(Error::Stale("Boolean first set")); }
                if feature != "SolidBooleanSplit" && selected.contains(&target.object) { return Err(Error::Invalid("an object cannot belong to both Boolean sets")); }
            }
            let mut all = Vec::new();
            all.try_reserve_exact(targets.len().checked_add(selected.len()).ok_or(Error::Overflow("Boolean inputs"))?)
                .map_err(|_| Error::Budget("Boolean inputs"))?;
            all.extend(targets.iter().map(|stamp| stamp.object));
            all.append(&mut selected);
            selected = all;
        }
        let placement = controller.common_boolean_affine(selected.iter().copied())?;
        checkpoint("selection_placement");
        let mut metadata = Usage::array::<(CapturedBrepSource, Transform)>(selected.len())?;
        metadata.add(Usage::array::<CapturedBrepSource>(selected.len())?, controller.coordinator().limits())?;
        metadata.add(Usage::array::<SourceStamp>(selected.len())?, controller.coordinator().limits())?;
        metadata.add(Usage::array::<om9_core::identity::ObjectId>(selected.capacity())?, controller.coordinator().limits())?;
        let mut scene_held = controller.scene_cache_usage()?;
        scene_held.add(old_unreserved, controller.coordinator().limits())?;
        let mut reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,
            metadata.bytes.max(1), om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            scene_held)?;
        reservation.recheck_pending()?;
        checkpoint("initial_admission");
        let mut inputs = Vec::new();
        inputs.try_reserve_exact(selected.len()).map_err(|_| Error::Budget("Boolean inputs"))?;
        let mut stamps = Vec::new();
        stamps.try_reserve_exact(selected.len()).map_err(|_| Error::Budget("Boolean stamps"))?;
        let mut captures = Vec::new();
        captures.try_reserve_exact(selected.len()).map_err(|_| Error::Budget("Boolean source batch"))?;
        let mut actual = Usage::array::<(CapturedBrepSource, Transform)>(inputs.capacity())?;
        actual.add(Usage::array::<SourceStamp>(stamps.capacity())?, controller.coordinator().limits())?;
        actual.add(Usage::array::<CapturedBrepSource>(captures.capacity())?, controller.coordinator().limits())?;
        actual.add(Usage::array::<om9_core::identity::ObjectId>(selected.capacity())?, controller.coordinator().limits())?;
        if actual.bytes > reservation.bytes() { return Err(Error::Budget("Boolean input metadata")); }
        reservation.set_allocated_bytes(actual.bytes)?;
        controller.solid_brep(selected[0])?;
        let scene = controller.scene().ok_or(Error::Missing("Boolean scene"))?;
        let first_stamp = controller.document().stamp(selected[0])?;
        let deflection = controller.viewport_detail.linear_mm();
        let angular = controller.viewport_detail.angular();
        let _ = first_stamp;
        // The existing batch API admits every source lease atomically against
        // the same immutable document. Repeated single captures otherwise
        // rescan the retained document/history owners once per operand.
        // Preserve the former accepted input counts when the batch API's own
        // count limit is lower than the command's valid operand count.
        let batch_capture = std::env::var_os("OM9_BOOLEAN_REQUEST_BASELINE").is_none()
            && selected.len() as u64 <= controller.coordinator().limits().max_outputs;
        if batch_capture {
            for id in &selected { controller.solid_brep(*id)?; }
            controller.coordinator().capture_brep_sources_into(controller.document().id, &selected, &mut captures)?;
        }
        checkpoint(if batch_capture {"batch_sources"} else {"serial_sources_begin"});
        let mut captures = captures.into_iter();
        let mut capture_ms = 0.;
        for id in selected {
            controller.solid_brep(id)?;
            let entity = controller.document().entities.get(&id).ok_or(Error::Missing("Boolean input"))?;
            let transform = if let Some(frame) = placement {
                Controller::boolean_relative_rigid(frame, entity.placement).ok_or(Error::Invalid("Boolean shared placement"))?
            } else {
                if !om9_kernel::transform_ops::is_exact_occt_rigid(&entity.placement)
                    .map_err(|e| Error::Io(format!("Boolean placement: {e}")))? {
                    return Err(Error::Unsupported("Boolean requires a rigid B-Rep placement; scaled or reflected inputs are not yet supported"));
                }
                entity.placement
            };
            let stamp = controller.document().stamp(id)?;
            let capture_started = profile.map(|_|std::time::Instant::now());
            let source = if batch_capture {
                captures.next().ok_or(Error::Missing("Boolean batch source"))?
            } else { controller.coordinator().capture_brep_source(&stamp)? };
            if let Some(started)=capture_started { capture_ms += started.elapsed().as_secs_f64()*1000.; }
            inputs.push((source, transform));
            stamps.push(stamp);
        }
        drop(captures);
        // The scratch capacity stays conservatively admitted through handoff,
        // just like the pre-existing selected-ID capacity in this reservation.
        if profile.is_some() { eprintln!("boolean_request batch={batch_capture} inputs={} serial_capture_ms={capture_ms:.3}",inputs.len()); }
        checkpoint("inputs_ready");
        if active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() { return Err(Error::Busy); }
        let permit = Permit(active);
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let mesh_preview = if feature == "OM9-SOLID-003" {
            Some(mesh_preview::PreviewJob::start(controller, &stamps, cancel.clone())
                .unwrap_or_else(mesh_preview::PreviewJob::failed))
        } else { None };
        let limits = *controller.coordinator().limits();
        let prepare_wires = std::env::var_os("OM9_BOOLEAN_COMMIT_BASELINE").is_none();
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-boolean-union".into()).spawn(move || {
            let result = (|| {
                let mut admit = |live: u64, pending: u64| {
                    if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                    let live = actual.bytes.checked_add(live).ok_or(Error::Overflow("Boolean worker admission"))?;
                    let needed = live.checked_add(pending).ok_or(Error::Overflow("Boolean worker admission"))?.max(1);
                    grow(&sender, &worker_cancel, &mut reservation, needed)?;
                    reservation.set_allocated_bytes(live)?;
                    reservation.recheck_pending()
                };
                let mut union_proof = None;
                let mut bodies = if let Some(mode) = preview_mode {
                    let ((a, ta), (b, tb)) = (&inputs[0], &inputs[1]);
                    let outcome = match mode {
                        0 => om9_kernel::editing::boolean_placed_captured_sources_cancellable(a, ta, b, tb,
                            om9_kernel::editing::BooleanOp::Union, deflection, true, &limits, &worker_cancel, &mut admit)?,
                        1 => om9_kernel::editing::boolean_placed_captured_sources_cancellable(a, ta, b, tb,
                            om9_kernel::editing::BooleanOp::Difference, deflection, false, &limits, &worker_cancel, &mut admit)?,
                        2 => om9_kernel::editing::boolean_placed_captured_sources_cancellable(b, tb, a, ta,
                            om9_kernel::editing::BooleanOp::Difference, deflection, false, &limits, &worker_cancel, &mut admit)?,
                        3 => om9_kernel::editing::boolean_placed_captured_sources_cancellable(a, ta, b, tb,
                            om9_kernel::editing::BooleanOp::Intersection, deflection, false, &limits, &worker_cancel, &mut admit)?,
                        4 => om9_kernel::editing::boolean_xor_placed_captured_sources_cancellable(a, ta, b, tb,
                            deflection, &limits, &worker_cancel, &mut admit)?,
                        _ => return Err(Error::Invalid("Boolean preview mode")),
                    };
                    match outcome { om9_kernel::editing::BooleanOutcome::Body(body) => Bodies::One(body),
                        om9_kernel::editing::BooleanOutcome::Empty => Bodies::Empty }
                } else if feature == "OM9-SOLID-003" {
                    let source = om9_kernel::editing::boolean_union_captured_source_cancellable(
                        &inputs, &limits, &worker_cancel, &mut admit)?;
                    let outside = source.capacity() as u64;
                    let remaining = limits.remaining_after(Usage { bytes: outside, ..Usage::default() })?;
                    let captured = om9_kernel::brep::capture_source_topology(&source, deflection, angular,
                        &remaining, |live, pending| admit(outside.checked_add(live)
                            .ok_or(Error::Overflow("Union final representation"))?, pending))
                        .map_err(|e| Error::Io(e.to_string()))?;
                    let (body, proof) = captured.into_parts();
                    union_proof = Some(proof);
                    Bodies::One(body)
                } else if feature == "SolidBooleanSplit" {
                    Bodies::Many(prepare_split(&inputs, target_count, deflection, &limits, &worker_cancel, &mut admit)?)
                } else {
                    let op = if feature == "OM9-SOLID-001" { om9_kernel::editing::BooleanOp::Difference }
                        else { om9_kernel::editing::BooleanOp::Intersection };
                    match om9_kernel::editing::boolean_binary_captured_groups_cancellable(&inputs[..target_count],
                        &inputs[target_count..], op, deflection, &limits, &worker_cancel, &mut admit)? {
                        om9_kernel::editing::BooleanOutcome::Body(body) => Bodies::One(body),
                        om9_kernel::editing::BooleanOutcome::Empty => return Err(Error::Invalid("boolean produced an empty body")),
                    }
                };
                // Capture the output's source topology before it becomes a document object.
                // This also supplies the requested angular profile for the viewport.
                let proof = if union_proof.is_some() { union_proof } else if let Bodies::One(body) = &bodies {
                    let outside = body.storage_usage(&limits)?;
                    let captured = om9_kernel::brep::capture_source_topology(body.source(), deflection, angular,
                        &limits.remaining_after(outside)?, |live, pending| {
                            admit(outside.bytes.checked_add(live).ok_or(Error::Overflow("Boolean display capture"))?, pending)
                        }).map_err(|e| Error::Io(format!("Boolean display capture: {e}")))?;
                    let (body, proof) = captured.into_parts();
                    bodies = Bodies::One(body);
                    Some(proof)
                } else { None };
                // Build source-derived interior wires/render buffers before
                // returning a committed-body candidate to the UI. The receipt
                // binds the exact display identity and retains no source Arc.
                let prepared = if prepare_wires && preview_mode.is_none() && let Bodies::One(body) = &bodies {
                    let mut outside = body.storage_usage(&limits)?;
                    if let Some(proof) = &proof { outside.add(Usage { bytes: proof.storage_bytes()?, ..Usage::default() }, &limits)?; }
                    let (buffers, usage) = super::topology_remesh::prepare_source_buffers(body.source(), body,
                        &limits.remaining_after(outside)?, |live, pending| {
                            admit(outside.bytes.checked_add(live).ok_or(Error::Overflow("Boolean wire admission"))?, pending)
                        })?;
                    Some((buffers, usage.bytes))
                } else { None };
                drop(admit);
                drop(inputs);
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                let mut retained = Usage { bytes: actual.bytes, ..Usage::default() };
                match &bodies {
                    Bodies::One(body) => retained.add(body.storage_usage(&limits)?, &limits)?,
                    Bodies::Many(pieces) => {
                        retained.add(Usage::array::<Brep>(pieces.capacity())?, &limits)?;
                        for piece in pieces { retained.add(piece.storage_usage(&limits)?, &limits)?; }
                    }
                    Bodies::Empty => (),
                }
                if let Some(proof) = &proof { retained.add(Usage { bytes: proof.storage_bytes()?, ..Usage::default() }, &limits)?; }
                if let Some((_, bytes)) = &prepared { retained.add(Usage { bytes: *bytes, ..Usage::default() }, &limits)?; }
                let retained = retained.bytes.max(1);
                reservation.set_allocated_bytes(retained)?;
                reservation.shrink_to(retained)?;
                Ok(Output { bodies, proof, prepared, reservation })
            })();
            drop(permit);
            let _ = sender.send(Event::Finished(result));
        }).map_err(|e| Error::Io(format!("Boolean worker start: {e}")))?;
        checkpoint("worker_spawned");
        Ok(Self { receiver, cancel, authority: controller.coordinator().authority(),
            epoch: controller.coordinator().epoch(), publication: scene.publication_id(), command_run: controller.command_run,
            selection_revision: controller.selection_state().revision, layer: controller.document().current_layer,
            stamps, placement, delete_input, feature, target_count, preview_mode, previous_mode, old_unreserved, batch_capture, mesh_preview, waiting_output: None })
    }

    fn current(&self, controller: &Controller) -> bool {
        self.authority == controller.coordinator().authority() && self.epoch == controller.coordinator().epoch()
            && self.command_run == controller.command_run && self.selection_revision == controller.selection_state().revision
            && self.layer == controller.document().current_layer
            && controller.scene().is_some_and(|s| s.publication_id() == self.publication)
            && controller.solid_pending.as_ref().is_some_and(|p| p.feature == self.feature && p.difference_delete_input == self.delete_input
                && (self.target_count == 0 || p.difference_targets.as_deref() == Some(&self.stamps[..self.target_count]))
                && (self.preview_mode.is_none() || p.boolean_two_preview.as_ref().map(|v| v.mode) == self.previous_mode))
            && self.stamps.iter().all(|stamp| controller.document().stamp(stamp.object).ok().as_ref() == Some(stamp))
    }
    fn poll(&mut self, controller: &Controller) -> Option<Result<Output>> {
        if self.cancel.is_canceled() { return Some(Err(Error::Canceled)); }
        if !self.current(controller) { self.cancel.cancel(); return Some(Err(Error::Stale("Boolean worker context"))); }
        if let Some(preview) = &mut self.mesh_preview { preview.poll(); }
        if let Some(output) = self.waiting_output.take() { return Some(output); }
        match self.receiver.try_recv() {
            Ok(Event::Grow { additional, reply }) => {
                let result = controller.scene_cache_usage().and_then(|held| {
                    let mut held = held;
                    held.add(self.old_unreserved, controller.coordinator().limits())?;
                    let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                    let batch = additional.max(64 * 1024);
                    match controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, batch, physical, held) {
                        Err(Error::Budget(_)) if batch > additional => controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, additional, physical, held),
                        result => result,
                    }
                });
                let _ = reply.send(result);
                None
            }
            Ok(Event::Finished(result)) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Error::Io("Boolean worker exited".into()))),
        }
    }
}

impl Controller {
    #[doc(hidden)]
    pub fn boolean_mesh_preview_failure(&self) -> Option<&str> {
        self.boolean_union_worker.as_ref()?.mesh_preview.as_ref()?.error.as_deref()
    }
    /// Continue admission/preview while navigating; defer document publication
    /// until the existing UI gesture guard permits it.
    #[doc(hidden)]
    pub fn service_boolean_union_worker(&mut self) {
        if let Some(mut job) = self.boolean_union_worker.take() {
            if let Some(preview) = &mut job.mesh_preview { preview.poll(); }
            if job.waiting_output.is_none() { job.waiting_output = job.poll(self); }
            self.boolean_union_worker = Some(job);
        }
    }
    /// Transient render data only: cannot be queried as a CAD object, saved or committed.
    #[doc(hidden)]
    pub fn boolean_mesh_preview(&self) -> Option<(&om9_scene::scene::SceneBuffers, &[SourceStamp], uuid::Uuid)> {
        let job = self.boolean_union_worker.as_ref()?;
        if !job.current(self) || job.cancel.is_canceled() { return None; }
        let output = job.mesh_preview.as_ref()?.output.as_ref()?;
        Some((&output.buffers, &job.stamps, output.id))
    }
    #[doc(hidden)]
    pub fn boolean_mesh_preview_diagnostics(&self) -> Option<(uuid::Uuid, f64, usize)> {
        let output = self.boolean_union_worker.as_ref()?.mesh_preview.as_ref()?.output.as_ref()?;
        Some((output.id, output.compute_ms, output.triangles))
    }
    pub(super) fn own_brep_preview(&self, mut candidate: Brep, mut reservation: ExternalMemoryReservation) -> Result<Brep> {
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let owner_bytes = (std::mem::size_of::<std::sync::Mutex<ExternalMemoryReservation>>()
            + 2 * std::mem::size_of::<usize>()) as u64;
        let store_bytes = Usage::btree::<(om9_core::brep::BrepSourceIdentity, om9_core::brep::BrepRepresentationProfile), om9_core::brep::BrepRepresentation>(1)?.bytes
            .checked_add(Usage::btree::<om9_core::brep::BrepSourceIdentity, om9_core::brep::BrepRepresentationProfile>(1)?.bytes)
            .and_then(|n| n.checked_add(om9_core::brep::BrepRepresentation::source_receipt_bytes()))
            .ok_or(Error::Overflow("BRep preview store"))?;
        let overhead = owner_bytes.checked_mul(3).and_then(|n| n.checked_add(store_bytes))
            .ok_or(Error::Overflow("BRep preview owners"))?;
        self.coordinator.grow_external_memory_with_held(&mut reservation, overhead, physical, held)?;
        reservation.set_allocated_bytes(reservation.bytes())?;
        let display_total = candidate.display_allocation_bytes()?.checked_add(owner_bytes).and_then(|n| n.checked_add(store_bytes))
            .ok_or(Error::Overflow("BRep preview display"))?;
        let topology_total = candidate.topology_allocation_bytes()?.checked_add(owner_bytes).ok_or(Error::Overflow("BRep preview topology"))?;
        let source_total = candidate.source_allocation_bytes()?.checked_add(owner_bytes).ok_or(Error::Overflow("BRep preview source"))?;
        let display = reservation.split_off(display_total)?;
        let topology = reservation.split_off(topology_total)?;
        reservation.shrink_to(source_total)?;
        let source = Arc::new(std::sync::Mutex::new(reservation));
        let topology = Arc::new(std::sync::Mutex::new(topology));
        let display = Arc::new(std::sync::Mutex::new(display));
        candidate.bind_split_allocation_owners(source, topology, display)?;
        Ok(candidate)
    }
    pub(crate) fn enable_background_boolean_union(&mut self) { self.background_boolean_union_enabled = true; }
    pub(super) fn request_boolean_union_worker(&mut self, delete_input: bool) -> Result<()> {
        if self.boolean_union_worker.is_some() { return Err(Error::Busy); }
        self.boolean_union_worker = Some(UnionJob::start(self, "OM9-SOLID-003", &[], delete_input, None, None, Usage::default(), Arc::clone(&self.boolean_union_active))?);
        Ok(())
    }
    pub(super) fn request_boolean_binary_worker(&mut self, feature: &'static str, targets: &[SourceStamp], delete_input: bool) -> Result<()> {
        if self.boolean_union_worker.is_some() { return Err(Error::Busy); }
        if !matches!(feature, "OM9-SOLID-001" | "OM9-SOLID-002") { return Err(Error::Invalid("Boolean operation")); }
        self.boolean_union_worker = Some(UnionJob::start(self, feature, targets, delete_input, None, None, Usage::default(), Arc::clone(&self.boolean_union_active))?);
        Ok(())
    }
    pub(super) fn request_boolean_split_worker(&mut self, targets: &[SourceStamp], delete_input: bool) -> Result<()> {
        if self.boolean_union_worker.is_some() { return Err(Error::Busy); }
        self.boolean_union_worker = Some(UnionJob::start(self, "SolidBooleanSplit", targets, delete_input, None, None, Usage::default(), Arc::clone(&self.boolean_union_active))?);
        Ok(())
    }
    pub(super) fn request_boolean_two_preview_worker(&mut self, mode: usize, pending: &super::solid::SolidPending) -> Result<()> {
        if self.boolean_union_worker.is_some() { return Err(Error::Busy); }
        let previous_mode = pending.boolean_two_preview.as_ref().map(|preview| preview.mode);
        let old_unreserved = pending.boolean_two_preview.as_ref().filter(|p| !p.is_accounted())
            .map_or(Usage::default(), |p| p.usage);
        self.boolean_union_worker = Some(UnionJob::start(self, "OM9-SOLID-004", &[], pending.difference_delete_input,
            Some(mode), previous_mode, old_unreserved, Arc::clone(&self.boolean_union_active))?);
        Ok(())
    }
    pub fn boolean_union_worker_pending(&self) -> bool { self.boolean_union_worker.is_some() }
    /// Runtime witness for request-path diagnostics, without retaining inputs.
    #[doc(hidden)]
    pub fn boolean_union_request_capture(&self) -> Option<(bool,usize)> {
        self.boolean_union_worker.as_ref().map(|job|(job.batch_capture,job.stamps.len()))
    }
    pub fn cancel_boolean_union_worker(&mut self) { drop(self.boolean_union_worker.take()); }
    pub fn poll_boolean_union_worker(&mut self) -> Option<Result<()>> {
        let mut job = self.boolean_union_worker.take()?;
        match job.poll(self) {
            None => { self.boolean_union_worker = Some(job); None }
            Some(result) => Some(result.and_then(|output| {
                if !job.current(self) { return Err(Error::Stale("Boolean worker context")); }
                if let Some(mode) = job.preview_mode {
                    let Output { bodies, proof, prepared: _, mut reservation } = output;
                    let topology_proof = proof.map(|proof| {
                        let bytes = proof.storage_bytes()?;
                        let kernel_owner = reservation.split_off(bytes)?;
                        let held = self.scene_cache_usage()?;
                        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                        let mut owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
                        owner.set_allocated_bytes(bytes)?;
                        drop(kernel_owner);
                        Ok::<_, Error>(std::sync::Mutex::new(Some((proof, owner))))
                    }).transpose()?;
                    let body = match bodies { Bodies::One(body) => Some(body), Bodies::Empty => None,
                        Bodies::Many(_) => return Err(Error::Invalid("Boolean preview output")) };
                    let usage = match &body { Some(body) => body.storage_usage(self.coordinator.limits())?, None => Usage::default() };
                    if let Some(candidate) = body {
                        let body = Some(self.own_brep_preview(candidate, reservation)?);
                        let sources: [SourceStamp; 2] = job.stamps.as_slice().try_into().map_err(|_| Error::Invalid("Boolean preview sources"))?;
                        let pending = self.solid_pending.as_mut().ok_or(Error::Stale("Boolean command"))?;
                        pending.boolean_two_preview = Some(super::solid::BooleanTwoPreview { mode, sources, body,
                            placement: job.placement.unwrap_or_else(Transform::identity), usage, reservation: None, topology_proof });
                        return Ok(());
                    }
                    let sources: [SourceStamp; 2] = job.stamps.as_slice().try_into().map_err(|_| Error::Invalid("Boolean preview sources"))?;
                    let pending = self.solid_pending.as_mut().ok_or(Error::Stale("Boolean command"))?;
                    pending.boolean_two_preview = Some(super::solid::BooleanTwoPreview { mode, sources, body: None,
                        placement: job.placement.unwrap_or_else(Transform::identity), usage, reservation: Some(reservation), topology_proof });
                    return Ok(());
                }
                let pending = self.solid_pending.take().ok_or(Error::Stale("Boolean command"))?;
                let Output { bodies, proof: _, prepared, mut reservation } = output;
                let name = match job.feature { "OM9-SOLID-001" => "Boolean Difference", "OM9-SOLID-002" => "Boolean Intersection", "SolidBooleanSplit" => "Boolean Split", _ => "Boolean Union" };
                let profile = std::env::var_os("OM9_COMMIT_PROFILE").map(|_| std::time::Instant::now());
                let result = (|| {
                    let output_count = match &bodies { Bodies::One(_) => 1, Bodies::Many(pieces) => pieces.len(), Bodies::Empty => return Err(Error::Invalid("Boolean output")) };
                    let held = self.scene_cache_usage()?;
                    let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                    let guard_bytes = Usage::array::<ExternalMemoryReservation>(output_count)?.bytes.max(1);
                    let mut guard_reservation = self.coordinator.reserve_external_memory_with_held(
                        om9_core::budget::memory::MemoryClass::CpuCache, guard_bytes, physical, held)?;
                    let mut source_guards = Vec::new();
                    source_guards.try_reserve_exact(output_count).map_err(|_| Error::Budget("Boolean source guards"))?;
                    let actual = Usage::array::<ExternalMemoryReservation>(source_guards.capacity())?.bytes.max(1);
                    if actual > guard_reservation.bytes() {
                        self.coordinator.grow_external_memory_with_held(&mut guard_reservation, actual - guard_bytes, physical, held)?;
                    }
                    guard_reservation.set_allocated_bytes(actual)?;
                    let mut mutations = Vec::new();
                    mutations.try_reserve_exact(job.stamps.len().checked_add(output_count).ok_or(Error::Overflow("Boolean mutations"))?)
                        .map_err(|_| Error::Budget("Boolean mutations"))?;
                    let mut outputs = Vec::new();
                    outputs.try_reserve_exact(output_count).map_err(|_| Error::Budget("Boolean outputs"))?;
                    for (index, source) in job.stamps.iter().enumerate() {
                        if (job.delete_input || (job.feature == "OM9-SOLID-001" && index < job.target_count))
                            && !job.stamps[..index].contains(source) {
                            mutations.push(Mutation::Delete { source: *source });
                        }
                    }
                    let mut push_body = |body: Brep| -> Result<()> {
                        let bytes = body.storage_usage(self.coordinator.limits())?.bytes;
                        let mut source_guard = reservation.split_off(bytes)?;
                        let source = self.coordinator.register_reserved_brep_candidate(body, &mut source_guard, held)?;
                        source_guards.push(source_guard);
                        let mut entity = Entity::new(name, Geometry::Brep(source), job.layer);
                        if let Some(placement) = job.placement { entity.placement = placement; }
                        outputs.push(entity.id);
                        mutations.push(Mutation::Insert { document: self.document, entity });
                        Ok(())
                    };
                    match bodies { Bodies::One(body) => push_body(body)?, Bodies::Many(pieces) => {
                        for body in pieces { push_body(body)?; }
                    }, Bodies::Empty => unreachable!("empty result rejected above") }
                    if let Some(start) = profile { eprintln!("OM9_COMMIT_PROFILE register_output_ms={:.3}", start.elapsed().as_secs_f64()*1000.); }
                    if let Some((buffers, bytes)) = prepared {
                        if outputs.len()!=1 { return Err(Error::Invalid("Boolean prepared wire output")); }
                        let kernel_wire = reservation.split_off(bytes.max(1))?;
                        let wire_owner = (|| {
                            let mut owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
                                bytes.max(1), physical, held)?;
                            owner.set_allocated_bytes(bytes)?;
                            Ok::<_, Error>(owner)
                        })();
                        let wire_owner = match wire_owner {
                            Ok(owner) => owner,
                            Err(error) => { drop(buffers); drop(kernel_wire); return Err(error); }
                        };
                        drop(kernel_wire);
                        let wire = super::PreparedEditWire { object: outputs[0], buffers, reservation: wire_owner };
                        self.commit_solid_result_with_prepared_wire(job.feature, name, mutations, Some(outputs), &mut source_guards, wire)
                    } else {
                        self.commit_solid_result_with_reserved_sources(job.feature, name, mutations, Some(outputs), &mut source_guards)
                    }
                })();
                drop(reservation);
                if result.is_err() {
                    self.coordinator.prune_orphaned_brep_representations();
                    self.solid_pending = Some(pending);
                }
                result
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{budget::Limits, model::Document};

    fn fixture() -> (Controller, Vec<om9_core::identity::ObjectId>) {
        let limits = Limits::default();
        let shape = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("worker union");
        let mut ids = Vec::new();
        for x in [0., 5., 10.] {
            let mut entity = Entity::new("box", Geometry::Brep(shape.clone()), None);
            entity.placement = Transform::translation([x, 0., 0.]);
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let mut controller = Controller::new(document, limits).unwrap();
        controller.enable_background_boolean_union();
        (controller, ids)
    }
    fn begin(controller: &mut Controller, ids: &[om9_core::identity::ObjectId]) {
        controller.select_objects(ids).unwrap();
        controller.submit("BooleanUnion").unwrap();
        controller.submit("DeleteInput=Yes").unwrap();
        controller.submit("").unwrap();
        assert!(controller.boolean_union_worker_pending());
    }
    fn finish(controller: &mut Controller) -> Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_boolean_union_worker() { return result; }
            assert!(std::time::Instant::now() < deadline, "Boolean worker timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    #[test]
    fn product_union_worker_commits_three_placements_and_undo_redo() {
        let (mut controller, ids) = fixture();
        let before = controller.document().clone();
        begin(&mut controller, &ids);
        assert_eq!(controller.document(), &before);
        finish(&mut controller).unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        let result = controller.document().entities.values().next().unwrap();
        let Geometry::Brep(body) = &result.geometry else { panic!("union result") };
        let volume = om9_kernel::creation::measure(body, controller.coordinator().limits()).unwrap().volume_mm3;
        assert!((volume - 2000.).abs() < 1e-6, "{volume}");
        assert_eq!(controller.coordinator().undo_len(), 1);
        let after = controller.document().clone();
        controller.undo().unwrap(); assert_eq!(controller.document(), &before);
        controller.redo().unwrap(); assert_eq!(controller.document(), &after);
    }
    #[test]
    fn product_union_worker_rejects_changed_selection_and_cancel() {
        for canceled in [false, true] {
            let (mut controller, ids) = fixture();
            let before = controller.document().clone();
            begin(&mut controller, &ids);
            if canceled {
                controller.cancel();
                assert!(!controller.boolean_union_worker_pending());
            } else {
                controller.select_objects(&ids[..2]).unwrap();
                assert!(matches!(controller.poll_boolean_union_worker(), Some(Err(Error::Stale(_)))));
            }
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
        }
    }
    #[test]
    fn product_union_worker_refuses_memory_without_changing_history() {
        let (mut controller, ids) = fixture();
        let before = controller.document().clone();
        begin(&mut controller, &ids);
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(matches!(finish(&mut controller), Err(Error::Budget(_))));
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.coordinator().undo_len(), 0);
        drop(pressure);
    }
    #[test]
    fn product_binary_worker_preserves_group_and_delete_semantics() {
        for (command, difference, delete) in [("BooleanDifference", true, false), ("BooleanDifference", true, true),
            ("BooleanIntersection", false, false), ("BooleanIntersection", false, true)] {
            let (mut controller, ids) = fixture();
            let before = controller.document().clone();
            controller.select_objects(&ids[..2]).unwrap();
            controller.submit(command).unwrap();
            controller.submit(if delete { "DeleteInput=Yes" } else { "DeleteInput=No" }).unwrap();
            controller.submit("").unwrap();
            controller.select_objects(&ids[2..]).unwrap();
            controller.submit("").unwrap();
            assert!(controller.boolean_union_worker_pending());
            assert_eq!(controller.document(), &before);
            finish(&mut controller).unwrap();
            let output = controller.document().entities.iter().find(|(id, _)| !ids.contains(id)).unwrap().1;
            let Geometry::Brep(body) = &output.geometry else { panic!("binary result") };
            let volume = om9_kernel::creation::measure(body, controller.coordinator().limits()).unwrap().volume_mm3;
            let expected = if difference { 1000. } else { 500. };
            assert!((volume - expected).abs() < 1e-6, "{command}: {volume}");
            for (index, id) in ids.iter().enumerate() {
                assert_eq!(controller.document().entities.contains_key(id), !delete && !(difference && index < 2));
            }
            let after = controller.document().clone();
            controller.undo().unwrap(); assert_eq!(controller.document(), &before);
            controller.redo().unwrap(); assert_eq!(controller.document(), &after);
        }
    }
    #[test]
    fn product_binary_worker_empty_result_and_stale_selection_preserve_document() {
        for stale in [false, true] {
            let (mut controller, ids) = fixture();
            let before = controller.document().clone();
            controller.select_objects(&ids[..1]).unwrap();
            controller.submit("BooleanIntersection").unwrap();
            controller.submit("").unwrap();
            controller.select_objects(&ids[2..]).unwrap();
            controller.submit("").unwrap();
            if stale { controller.select_objects(&ids[1..2]).unwrap(); }
            let result = finish(&mut controller);
            if stale { assert!(matches!(result, Err(Error::Stale(_)))); }
            else { assert!(matches!(result, Err(Error::Invalid(_)))); }
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
        }
    }
    #[test]
    fn product_difference_worker_handles_multiple_disjoint_cutters() {
        let limits = Limits::default();
        let target = om9_kernel::brep::box_brep([30., 10., 10.], 0.1, &limits).unwrap();
        let cutter = om9_kernel::brep::box_brep([2., 2., 2.], 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("disjoint tools");
        let target = Entity::new("target", Geometry::Brep(target), None);
        let target_id = target.id;
        document.entities.insert(target.id, target);
        let mut tools = Vec::new();
        for x in [2., 22.] {
            let mut entity = Entity::new("tool", Geometry::Brep(cutter.clone()), None);
            entity.placement = Transform::translation([x, 2., 2.]);
            tools.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let mut controller = Controller::new(document, limits).unwrap();
        controller.enable_background_boolean_union();
        controller.select_objects(&[target_id]).unwrap();
        controller.submit("BooleanDifference").unwrap(); controller.submit("").unwrap();
        controller.select_objects(&tools).unwrap(); controller.submit("").unwrap();
        finish(&mut controller).unwrap();
        let output = controller.document().entities.values().find(|e| e.label == "Boolean Difference").unwrap();
        let Geometry::Brep(body) = &output.geometry else { panic!("difference") };
        let volume = om9_kernel::creation::measure(body, &limits).unwrap().volume_mm3;
        assert!((volume - 2984.).abs() < 1e-6, "{volume}");
    }
    #[test]
    fn product_binary_worker_refuses_memory_before_commit() {
        let (mut controller, ids) = fixture();
        let before = controller.document().clone();
        controller.select_objects(&ids[..1]).unwrap();
        controller.submit("BooleanDifference").unwrap(); controller.submit("").unwrap();
        controller.select_objects(&ids[1..]).unwrap(); controller.submit("").unwrap();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(matches!(finish(&mut controller), Err(Error::Budget(_))));
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.coordinator().undo_len(), 0);
        drop(pressure);
    }
    #[test]
    fn product_split_worker_handles_overlapping_sets_and_undo_redo() {
        for delete in [false, true] {
            let (mut controller, ids) = fixture();
            let before = controller.document().clone();
            controller.select_objects(&ids[..2]).unwrap();
            controller.submit("BooleanSplit").unwrap();
            controller.submit(if delete { "DeleteInput=Yes" } else { "DeleteInput=No" }).unwrap();
            controller.submit("").unwrap();
            controller.select_objects(&ids[1..]).unwrap();
            controller.submit("").unwrap();
            assert!(controller.boolean_union_worker_pending());
            assert_eq!(controller.document(), &before);
            finish(&mut controller).unwrap();
            let output: Vec<_> = controller.document().entities.values().filter(|e| e.label == "Boolean Split").collect();
            assert_eq!(output.len(), 3);
            let mut volume = 0.;
            for entity in output {
                let Geometry::Brep(body) = &entity.geometry else { panic!("split result") };
                let measure = om9_kernel::creation::measure(body, controller.coordinator().limits()).unwrap();
                assert_eq!(measure.solids, 1);
                volume += measure.volume_mm3;
            }
            assert!((volume - 1500.).abs() < 1e-6, "{volume}");
            for id in &ids { assert_eq!(controller.document().entities.contains_key(id), !delete); }
            let after = controller.document().clone();
            controller.undo().unwrap(); assert_eq!(controller.document(), &before);
            controller.redo().unwrap(); assert_eq!(controller.document(), &after);
        }
    }
    #[test]
    fn product_split_worker_rejects_stale_and_memory_pressure() {
        for stale in [false, true] {
            let (mut controller, ids) = fixture();
            let before = controller.document().clone();
            controller.select_objects(&ids[..1]).unwrap();
            controller.submit("BooleanSplit").unwrap(); controller.submit("").unwrap();
            controller.select_objects(&ids[1..]).unwrap(); controller.submit("").unwrap();
            if stale {
                controller.select_objects(&ids[1..2]).unwrap();
                assert!(matches!(finish(&mut controller), Err(Error::Stale(_))));
            } else {
                let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
                    MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
                assert!(matches!(finish(&mut controller), Err(Error::Budget(_))));
                drop(pressure);
            }
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
        }
    }
    #[test]
    fn product_split_worker_preserves_shared_affine_frame() {
        let limits = Limits::default();
        let shape = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let frame = Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]);
        let mut document = Document::with_template_tolerance("split shared affine");
        let mut ids = Vec::new();
        for delta in [0., 5.] {
            let mut entity = Entity::new("box", Geometry::Brep(shape.clone()), None);
            entity.placement = frame.compose(Transform::translation([delta, 0., 0.])).unwrap();
            ids.push(entity.id); document.entities.insert(entity.id, entity);
        }
        let mut controller = Controller::new(document, limits).unwrap();
        controller.enable_background_boolean_union();
        controller.select_objects(&ids[..1]).unwrap(); controller.submit("BooleanSplit").unwrap();
        controller.submit("DeleteInput=Yes").unwrap(); controller.submit("").unwrap();
        controller.select_objects(&ids[1..]).unwrap(); controller.submit("").unwrap();
        finish(&mut controller).unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        for entity in controller.document().entities.values() {
            assert_eq!(entity.placement, frame);
            let Geometry::Brep(body) = &entity.geometry else { panic!("split output") };
            assert!((om9_kernel::creation::measure(body, &limits).unwrap().volume_mm3 - 500.).abs() < 1e-6);
        }
    }
    #[test]
    fn product_split_worker_explicit_cancel_preserves_document() {
        let (mut controller, ids) = fixture();
        let before = controller.document().clone();
        controller.select_objects(&ids[..1]).unwrap(); controller.submit("BooleanSplit").unwrap();
        controller.submit("").unwrap(); controller.select_objects(&ids[1..]).unwrap(); controller.submit("").unwrap();
        assert!(controller.boolean_union_worker_pending());
        controller.cancel();
        assert!(!controller.boolean_union_worker_pending());
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.coordinator().undo_len(), 0);
    }
    #[test]
    fn product_boolean_two_worker_cycles_all_modes_and_commits() {
        let (mut controller, ids) = fixture();
        controller.select_objects(&ids[..2]).unwrap();
        let before = controller.document().clone();
        controller.submit("Boolean2Objects").unwrap(); controller.submit("").unwrap();
        assert!(controller.boolean_union_worker_pending());
        finish(&mut controller).unwrap();
        for (name, expected) in [("Union", 1500.), ("Difference", 500.), ("ReverseDifference", 500.),
            ("Intersection", 500.), ("InversionIntersection", 1000.)] {
            let (body, actual) = controller.solid_boolean_two_preview().unwrap();
            assert_eq!(actual, name);
            let measurement = om9_kernel::creation::measure(body, controller.coordinator().limits()).unwrap();
            let volume = measurement.volume_mm3;
            assert!((volume - expected).abs() < 1e-6, "{name}: {volume}");
            if name == "Difference" { assert!((measurement.bounds_mm[0] - 0.).abs() < 1e-6 && (measurement.bounds_mm[3] - 5.).abs() < 1e-6); }
            if name == "ReverseDifference" { assert!((measurement.bounds_mm[0] - 10.).abs() < 1e-6 && (measurement.bounds_mm[3] - 15.).abs() < 1e-6); }
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
            controller.submit("Next").unwrap();
            assert_eq!(controller.solid_boolean_two_preview().unwrap().1, name, "old preview remains while worker runs");
            finish(&mut controller).unwrap();
        }
        assert_eq!(controller.solid_boolean_two_preview().unwrap().1, "Union");
        controller.submit("").unwrap();
        let after = controller.document().clone();
        assert_eq!(controller.coordinator().undo_len(), 1);
        controller.undo().unwrap(); assert_eq!(controller.document(), &before);
        controller.redo().unwrap(); assert_eq!(controller.document(), &after);
    }
    #[test]
    fn product_boolean_two_worker_keeps_preview_on_stale_or_pressure() {
        for stale in [false, true] {
            let (mut controller, ids) = fixture();
            controller.select_objects(&ids[..2]).unwrap();
            let before = controller.document().clone();
            controller.submit("Boolean2Objects").unwrap(); controller.submit("").unwrap(); finish(&mut controller).unwrap();
            let prior = controller.solid_boolean_two_preview().unwrap().0.display_fingerprint();
            controller.submit("Next").unwrap();
            if stale {
                controller.select_objects(&ids[1..]).unwrap();
                assert!(matches!(finish(&mut controller), Err(Error::Stale(_))));
            } else {
                let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
                    MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
                assert!(matches!(finish(&mut controller), Err(Error::Budget(_))));
                drop(pressure);
            }
            let (body, mode) = controller.solid_boolean_two_preview().unwrap();
            assert_eq!(mode, "Union"); assert_eq!(body.display_fingerprint(), prior);
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
        }
    }
    #[test]
    fn product_boolean_two_worker_empty_xor_preserves_sources_and_releases_reservation() {
        let limits = Limits::default();
        let shape = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("Boolean two empty");
        let a = Entity::new("a", Geometry::Brep(shape.clone()), None);
        let b = Entity::new("b", Geometry::Brep(shape), None);
        let ids = [a.id, b.id]; document.entities.insert(a.id, a); document.entities.insert(b.id, b);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.enable_background_boolean_union();
        controller.select_objects(&ids).unwrap();
        let before = controller.document().clone();
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        controller.submit("Boolean2Objects").unwrap(); controller.submit("X").unwrap();
        finish(&mut controller).unwrap();
        assert_eq!(controller.solid_boolean_two_preview_mode(), Some("InversionIntersection"));
        assert!(controller.solid_boolean_two_preview().is_none());
        assert!(controller.solid_boolean_two_hidden_sources().is_some());
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > baseline);
        assert!(controller.submit("").is_err());
        assert_eq!(controller.document(), &before);
        controller.cancel();
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), baseline);
        assert_eq!(controller.coordinator().undo_len(), 0);
    }
}
