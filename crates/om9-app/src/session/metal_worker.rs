// SPDX-License-Identifier: LGPL-2.1-or-later
//! Cancellable source measurement off the UI thread; publication stays on the controller.
use super::{Controller, metal::{self, MetalCatalogReport, MetalWeightReport}};
use om9_core::{Error, Result, budget::{Usage, memory::MemoryClass}, identity::{DocumentId, MaterialId, ObjectId},
    metal_weight::{MassPart, MetalMaterialRegistry}, transaction::{Cancellation, CapturedDocument, ExternalMemoryReservation}};
use std::{collections::BTreeMap, sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc}};

// This is a conservative initial reservation, not a bound on OCCT's opaque allocations.
const NATIVE_START: u64 = 64 * 1024 * 1024;
type Bindings = BTreeMap<(DocumentId, MaterialId), String>;

#[derive(Clone, PartialEq, Eq)]
enum Mode { ObjectMaterials, One(String), Catalog(bool, bool) }
impl Mode {
    fn current(c: &Controller) -> Self {
        if c.metal_panel_object_materials { Self::ObjectMaterials }
        else if let Some(id) = &c.metal_panel_material { Self::One(id.clone()) }
        else { Self::Catalog(c.metal_panel_reference, c.metal_panel_user) }
    }
}
struct Inputs {
    document: CapturedDocument,
    ids: Vec<ObjectId>,
    registry: MetalMaterialRegistry,
    bindings: Bindings,
    mode: Mode,
    limits: om9_core::budget::Limits,
    epoch: u64,
    selection_revision: u64,
    generation: u64,
}
enum Output { Report(MetalWeightReport), Catalog(MetalCatalogReport) }
impl Output {
    fn retained_bytes(&self) -> Result<u64> {
        let mut bytes = 4096u64;
        let mut add = |value: usize| -> Result<()> {
            bytes = bytes.checked_add(value as u64).ok_or(Error::Overflow("metal report memory"))?;
            Ok(())
        };
        match self {
            Self::Report(report) => {
                add(report.text.capacity().saturating_mul(4))?;
                add(report.material_id.capacity())?;
                add(report.parts.capacity().saturating_mul(std::mem::size_of::<metal::MetalPartReport>()))?;
                for part in &report.parts { add(part.label.capacity())?; add(part.material_id.capacity())?; add(128)?; }
            }
            Self::Catalog(report) => {
                add(report.text.capacity().saturating_mul(4))?;
                add(report.estimates.capacity().saturating_mul(std::mem::size_of::<(String, om9_core::metal_weight::MassBreakdown)>()))?;
                for (name, breakdown) in &report.estimates {
                    add(name.capacity())?;
                    add(128)?;
                    add(breakdown.entries.capacity().saturating_mul(std::mem::size_of::<om9_core::metal_weight::MassEntry>()))?;
                    for entry in &breakdown.entries { add(entry.key.capacity())?; add(entry.material_id.capacity())?; add(128)?; }
                }
            }
        }
        Ok(bytes)
    }
}
enum Event {
    Grow { bytes: u64, reply: mpsc::SyncSender<Result<ExternalMemoryReservation>> },
    Finished(Result<Output>, ExternalMemoryReservation),
}
struct Permit(Arc<AtomicBool>);
impl Drop for Permit { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }
pub(super) struct MetalJob {
    input: Arc<Inputs>, cancel: Cancellation, receiver: mpsc::Receiver<Event>,
}
impl Drop for MetalJob { fn drop(&mut self) { self.cancel.cancel(); } }

fn grow(sender: &mpsc::SyncSender<Event>, cancel: &Cancellation, reservation: &mut ExternalMemoryReservation, need: u64) -> Result<()> {
    if need <= reservation.bytes() { return Ok(()); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let (reply, receiver) = mpsc::sync_channel(1);
    sender.send(Event::Grow { bytes: need - reservation.bytes(), reply }).map_err(|_| Error::Canceled)?;
    loop {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        match receiver.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(result) => { let mut extra = result?; return reservation.merge_pending_from(&mut extra); }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Canceled),
        }
    }
}

fn estimate_metadata(c: &Controller, ids: &[ObjectId]) -> Result<u64> {
    let mut bytes = Usage::array::<ObjectId>(ids.len())?.bytes.saturating_add(4096);
    for m in c.metal_registry.iter() {
        let size = m.id.len().checked_add(m.name.len()).and_then(|n| n.checked_add(m.provenance.len()))
            .and_then(|n| n.checked_add(512)).ok_or(Error::Overflow("metal worker metadata"))? as u64;
        bytes = bytes.checked_add(size).ok_or(Error::Overflow("metal worker metadata"))?;
    }
    for value in c.metal_material_bindings.values() {
        bytes = bytes.checked_add(value.len() as u64 + 256).ok_or(Error::Overflow("metal worker metadata"))?;
    }
    Ok(bytes)
}

impl MetalJob {
    fn start(c: &Controller) -> Result<Self> {
        let ids = c.selected_ordered();
        if ids.is_empty() { return Err(Error::Missing("metal weight selection")); }
        let metadata = estimate_metadata(c, &ids)?;
        let reserved = metadata.checked_add(NATIVE_START).ok_or(Error::Overflow("metal worker memory"))?;
        let held = c.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = c.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, reserved, physical, held)?;
        reservation.set_allocated_bytes(metadata)?;
        let document = c.coordinator.capture_tracked_document(c.document)?;
        let input = Arc::new(Inputs { document, ids, registry: c.metal_registry.clone(),
            bindings: c.metal_material_bindings.clone(), mode: Mode::current(c), limits: *c.coordinator.limits(),
            epoch: c.coordinator.epoch(), selection_revision: c.selection_revision,
            generation: c.metal_request_generation });
        if c.metal_worker_active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() { return Err(Error::Busy); }
        let permit = Permit(c.metal_worker_active.clone());
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_input = input.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-metal-weights".into()).spawn(move || {
            let mut admit = |live: u64, pending: u64| -> Result<()> {
                if worker_cancel.is_canceled() { return Err(Error::Canceled); }
                let required = metadata.checked_add(NATIVE_START).and_then(|n| n.checked_add(live))
                    .and_then(|n| n.checked_add(pending)).ok_or(Error::Overflow("metal worker memory"))?;
                grow(&sender, &worker_cancel, &mut reservation, required)?;
                reservation.recheck_pending()
            };
            let result = compute(&worker_input, &worker_cancel, &mut admit).and_then(|output| {
                admit(output.retained_bytes()?, 0)?;
                Ok(output)
            });
            drop(admit);
            drop(permit);
            if !worker_cancel.is_canceled() { let _ = sender.send(Event::Finished(result, reservation)); }
        }).map_err(|error| Error::Io(error.to_string()))?;
        Ok(Self { input, cancel, receiver })
    }

    fn current(&self, c: &Controller) -> bool {
        c.metal_panel_open && c.coordinator.epoch() == self.input.epoch
            && c.document == self.input.document.id && c.selection_revision == self.input.selection_revision
            && c.metal_request_generation == self.input.generation
            && c.metal_registry == self.input.registry && c.metal_material_bindings == self.input.bindings
            && Mode::current(c) == self.input.mode
    }

    fn poll(&mut self, c: &Controller) -> Option<Result<(Output, ExternalMemoryReservation)>> {
        if !self.current(c) { self.cancel.cancel(); return Some(Err(Error::Stale("metal weight context"))); }
        if self.cancel.is_canceled() { return Some(Err(Error::Canceled)); }
        match self.receiver.try_recv() {
            Ok(Event::Grow { bytes, reply }) => {
                let result = c.scene_cache_usage().and_then(|held| {
                    let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                    c.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, bytes, physical, held)
                });
                let _ = reply.send(result);
                None
            }
            Ok(Event::Finished(result, reservation)) => Some(result.map(|output| (output, reservation))),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Error::Io("metal worker exited".into()))),
        }
    }
}

fn compute(input: &Inputs, cancel: &Cancellation, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Output> {
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if input.mode == Mode::ObjectMaterials {
        let (parts, objects) = Controller::collect_object_metal_parts(&input.document, &input.ids, &input.limits,
            &input.bindings, &input.registry, cancel, admit)?;
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let report = Controller::build_metal_parts_report(parts, objects, &input.registry)?;
        if cancel.is_canceled() { return Err(Error::Canceled); }
        return Ok(Output::Report(report));
    }
    let material_count = match &input.mode {
        Mode::One(id) => { input.registry.get(id)?; 1 },
        Mode::Catalog(reference, user) => input.registry.iter().filter(|m|
            if input.registry.is_reference(&m.id) { *reference } else { *user }).count(),
        Mode::ObjectMaterials => unreachable!(),
    };
    if material_count == 0 { return Err(Error::Missing("metal material density")); }
    let count = input.ids.len().checked_mul(material_count).ok_or(Error::Overflow("metal catalog entries"))?;
    if count as u64 > input.limits.max_outputs { return Err(Error::Budget("metal catalog entries")); }
    let max_id = input.registry.iter().map(|m| m.id.len()).max().unwrap_or(0) as u64;
    let mut report_bytes = 4096u64;
    for id in &input.ids {
        let entity = input.document.entities.get(id).ok_or(Error::Missing("metal weight object"))?;
        let per = (entity.label.len() as u64).checked_add(max_id).and_then(|v| v.checked_add(128))
            .and_then(|v| v.checked_mul(8)).and_then(|v| v.checked_add(2048)).ok_or(Error::Overflow("metal worker report"))?;
        report_bytes = report_bytes.checked_add(per.checked_mul(material_count as u64).ok_or(Error::Overflow("metal worker report"))?)
            .ok_or(Error::Overflow("metal worker report"))?;
    }
    admit(0, report_bytes)?;
    let mut volumes = Vec::new();
    volumes.try_reserve_exact(input.ids.len()).map_err(|_| Error::Budget("metal worker volumes"))?;
    let mut occurrences = 0;
    for id in &input.ids {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let entity = input.document.entities.get(id).ok_or(Error::Missing("metal weight object"))?;
        let mut validation = |live: u64, pending: u64| admit(report_bytes.checked_add(live).ok_or(Error::Overflow("metal worker memory"))?, pending);
        let volume = metal::measure_metal_source_volume_with_count(&input.document, entity, &input.limits, cancel, &mut validation, &mut occurrences)?;
        volumes.push((*id, metal::entity_label(entity, id), volume));
    }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let output = match &input.mode {
        Mode::One(id) => {
            let parts = volumes.iter().map(|(object, label, volume)| MassPart::new(format!("{label} [{object}]"), *volume, id.clone())).collect();
            let objects = volumes.into_iter().map(|(object, label, _)| (object, label)).collect();
            Output::Report(Controller::build_metal_parts_report(parts, objects, &input.registry)?)
        }
        Mode::Catalog(reference, user) => {
            let values = volumes.into_iter().map(|(id, label, volume)| (format!("{label} [{id}]"), volume)).collect();
            Output::Catalog(Controller::build_metal_catalog(values, *reference, *user, &input.registry, &input.limits)?)
        }
        Mode::ObjectMaterials => unreachable!(),
    };
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(output)
}

impl Controller {
    pub fn enable_background_metal_weights(&mut self) { self.background_metal = true; }
    pub fn metal_worker_pending(&self) -> bool { self.metal_worker.is_some() }
    pub fn metal_worker_active(&self) -> bool { self.metal_worker_active.load(Ordering::Acquire) }
    pub fn cancel_metal_worker(&mut self) { drop(self.metal_worker.take()); }
    pub(super) fn start_metal_worker(&mut self) -> Result<()> {
        if self.metal_worker.is_some() || self.metal_worker_active() { return Err(Error::Busy); }
        self.metal_request_generation = self.metal_request_generation.saturating_add(1);
        self.metal_worker = Some(MetalJob::start(self)?);
        Ok(())
    }
    pub fn poll_metal_worker(&mut self) -> Option<Result<()>> {
        let mut job = self.metal_worker.take()?;
        match job.poll(self) {
            None => { self.metal_worker = Some(job); None }
            Some(result) => Some(result.and_then(|(output, mut reservation)| {
                let bytes = output.retained_bytes()?.max(1);
                reservation.set_allocated_bytes(bytes)?;
                reservation.shrink_to(bytes)?;
                let held = self.scene_cache_usage()?;
                let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                let mut owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held)?;
                owner.set_allocated_bytes(bytes)?;
                drop(reservation);
                match output {
                    Output::Report(report) => { self.install_metal_report(report); self.metal_report_owner = Some(owner); }
                    Output::Catalog(report) => { self.install_metal_catalog(report); self.metal_catalog_owner = Some(owner); }
                }
                Ok(())
            })),
        }
    }
}
