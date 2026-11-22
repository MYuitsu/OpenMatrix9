// SPDX-License-Identifier: LGPL-2.1-or-later
//! Host worker for mesh Boolean Builder preview and final calculation.
use super::Controller;
use om9_addon_abi::session::{Event, Geo, Reply, Replacement, Selected, SelectedMesh};
use om9_core::{Error, Result, budget::memory::MemoryClass, geometry::Mesh,
    mesh_csg::{BooleanOp, CsgOptions, mesh_boolean_batch_cancellable},
    transaction::{Cancellation, ExternalMemoryReservation}};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};

struct Permit(Arc<AtomicBool>);
impl Drop for Permit { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }

pub(super) struct BooleanBuilderJob {
    receiver: mpsc::Receiver<Result<(Reply, ExternalMemoryReservation)>>,
    cancel: Cancellation,
    authority: om9_core::identity::SessionId,
    epoch: u64,
    scene: uuid::Uuid,
    selection_revision: u64,
    command_run: u64,
    command: String,
    builder_revision: Option<u64>,
    preview: bool,
    commit_requested: bool,
}
impl Drop for BooleanBuilderJob { fn drop(&mut self) { self.cancel.cancel(); } }
impl BooleanBuilderJob {
    fn current(&self, c: &Controller) -> bool {
        let Some(session) = c.addon_session.as_ref() else { return false };
        c.coordinator.authority() == self.authority && c.coordinator.epoch() == self.epoch
            && c.scene.as_ref().is_some_and(|scene| scene.publication_id() == self.scene)
            && c.selection_state().revision == self.selection_revision && c.command_run == self.command_run
            && c.addon_builder.as_ref().map(|builder| builder.revision) == self.builder_revision
            && session.code == self.command
            && session.boolean_sources.values().chain(session.boolean_preview_sources.values()).all(|stamp| c.document().resolve(stamp).is_ok())
            && c.boolean_definitions_current(&session.boolean_definitions)
            && c.boolean_definitions_current(&session.boolean_preview_definitions)
            && session.boolean_source_tolerance.is_some_and(|t| c.document().effective_tolerance().is_ok_and(|current| current.absolute_mm == t))
    }
}

fn value_bytes(value: &serde_json::Value) -> Result<u64> {
    use serde_json::Value;
    let nested = match value {
        Value::Array(items) => items.iter().try_fold(0u64, |sum, item| sum.checked_add(value_bytes(item)?).ok_or(Error::Overflow("Boolean event")))?,
        Value::Object(items) => items.iter().try_fold(0u64, |sum, (key, item)| {
            let child = value_bytes(item)?;
            sum.checked_add(key.len() as u64).and_then(|n| n.checked_add(64))
                .and_then(|n| n.checked_add(child)).ok_or(Error::Overflow("Boolean event"))
        })?,
        Value::String(value) => value.len() as u64,
        _ => 0,
    };
    nested.checked_add(32).ok_or(Error::Overflow("Boolean event"))
}

pub(super) fn calculate(event: &Event, cancel: &Cancellation, preview: bool, prepared: Option<Vec<(String, SelectedMesh)>>) -> Result<Reply> {
    let objects = match prepared {
        Some(meshes) => meshes,
        None => serde_json::from_value(event.state.get("objects").cloned().ok_or(Error::Missing("Boolean objects"))?)
            .map_err(|_| Error::Invalid("Boolean objects"))?,
    };
    if objects.is_empty() { return Err(Error::Missing("Boolean objects")); }
    let first: std::collections::BTreeSet<_> = objects.iter().map(|(id, _)| id.as_str()).collect();
    let mut cutters: Vec<_> = event.selection.iter().filter(|s| !first.contains(s.id.as_str())).collect();
    cutters.sort_by(|a,b| a.id.cmp(&b.id));
    if cutters.is_empty() { return Err(Error::Missing("Boolean cutters")); }
    let mesh_op = super::mesh_boolean::MeshBoolean::from_name(&event.command);
    if cutters.iter().any(|s| (mesh_op.is_none() && (s.kind != "mesh" || s.gem.is_some())) || s.mesh.as_ref().is_none_or(|m| m.triangles.is_empty())) {
        return Err(Error::Invalid("Boolean cutter source mesh"));
    }
    let options = CsgOptions { tolerance: event.number("_om9_document_tolerance_mm", CsgOptions::default().tolerance), ..CsgOptions::default() };
    let to_core = |mesh: &SelectedMesh| -> Result<Mesh> {
        if mesh.triangles.len() > options.max_triangles { return Err(Error::Budget("Boolean mesh triangles")); }
        let mut vertices = Vec::new();
        vertices.try_reserve_exact(mesh.vertices.len()).map_err(|_| Error::Budget("Boolean mesh vertices"))?;
        vertices.extend_from_slice(&mesh.vertices);
        let mut faces = Vec::new();
        faces.try_reserve_exact(mesh.triangles.len()).map_err(|_| Error::Budget("Boolean mesh faces"))?;
        for triangle in &mesh.triangles { faces.push(triangle.to_vec()); }
        Ok(Mesh { vertices, faces, normals: vec![], colors: vec![], corner_uvs: vec![] })
    };
    let inputs = objects.iter().map(|(_, mesh)| to_core(mesh)).collect::<Result<Vec<_>>>()?;
    let tools = cutters.iter().map(|selected| to_core(selected.mesh.as_ref().expect("validated mesh"))).collect::<Result<Vec<_>>>()?;
    let op = match event.text("bb_mode", "difference") {
        "union" => BooleanOp::Union, "intersection" => BooleanOp::Intersection, _ => BooleanOp::Difference,
    };
    let meshes = mesh_boolean_batch_cancellable(&inputs, &tools, op, options, cancel)?;
    let mut lines = Vec::new();
    if preview {
        for mesh in &meshes {
            let mut seen = std::collections::BTreeSet::new();
            for face in &mesh.faces {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                if face.len() != 3 { return Err(Error::Invalid("Boolean preview face")); }
                for i in 0..3 {
                    let (a,b) = (face[i], face[(i+1)%3]);
                    if seen.insert((a.min(b), a.max(b))) {
                        lines.push(Geo::Line { start: mesh.vertices[a as usize], end: mesh.vertices[b as usize] });
                    }
                }
            }
        }
    }
    let mut reply = Reply::finish(Vec::new());
    let mut volume = 0.0;
    for ((id, _), mesh) in objects.into_iter().zip(meshes) {
        let triangles = mesh.faces.iter().map(|face| <[u32; 3]>::try_from(face.as_slice()).map_err(|_| Error::Invalid("Boolean output face")))
            .collect::<Result<Vec<_>>>()?;
        let origin = mesh.vertices.first().copied().unwrap_or([0.; 3]);
        for triangle in &triangles {
            let [a,b,c] = triangle.map(|index| om9_addon_abi::session::sub(mesh.vertices[index as usize], origin));
            volume += om9_addon_abi::session::dot(a, om9_addon_abi::session::cross(b,c)) / 6.0;
        }
        let geo = Geo::Mesh { name: mesh_op.map_or("Boolean", |op| op.label()).into(), vertices: mesh.vertices, triangles, meta: None, layer: None };
        if mesh_op.is_some() && event.flag("bb_keep_cutters", false) {
            reply.create.push(geo);
        } else {
            reply.replace.push(Replacement { id, geo });
        }
    }
    if !event.flag("bb_keep_cutters", false) { reply.delete = cutters.iter().map(|s| s.id.clone()).collect(); }
    if preview {
        reply.prompt = format!("Boolean Builder: {} objects × {} cutters. Mode in the builder, Enter", reply.replace.len(), cutters.len());
        reply.preview = lines;
    }
    let message = format!("{}: {} objects, {} cutters, volume {} mm³", mesh_op.map_or("Boolean Builder", |op| op.label()), reply.replace.len() + reply.create.len(), cutters.len(), om9_addon_abi::session::fmt(volume));
    Ok(reply.say(message))
}

fn prepare_selection(document: &om9_core::model::Document, roots: &[om9_core::identity::ObjectId],
    k: f64, tolerance: f64, max_depth: usize, cancel: &Cancellation) -> Result<Vec<Selected>> {
    use super::addon_session::{boolean_mesh_source, boolean_source_mesh};
    use om9_core::model::GEM_EXTENSION_SLOT;
    let mut selected = Vec::with_capacity(roots.len());
    let mut triangles = 0usize;
    for id in roots {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let entity = document.entities.get(id).ok_or(Error::Missing("Boolean source"))?;
        let gem = entity.extension(GEM_EXTENSION_SLOT).and_then(|x| serde_json::from_str(&x.payload).ok());
        let mut item = Selected { id: id.to_string(), name: entity.label.clone(), gem, ..Selected::default() };
        if let Some((mesh, placement, _)) = boolean_mesh_source(document, entity, max_depth)? {
            let mesh = boolean_source_mesh(mesh, &placement, k, tolerance, Some(cancel))?;
            triangles = triangles.checked_add(mesh.triangles.len()).ok_or(Error::Overflow("Boolean selection triangles"))?;
            if triangles > 2_000_000 { return Err(Error::Budget("Boolean selection triangles")); }
            item.kind = "mesh".into();
            item.mesh = Some(mesh);
        }
        selected.push(item);
    }
    Ok(selected)
}

impl Controller {
    pub(crate) fn enable_background_boolean_builder(&mut self) { self.background_boolean_builder = true; }
    #[cfg(feature = "test-fixtures")]
    #[doc(hidden)]
    pub fn test_enable_background_boolean_builder(&mut self) { self.enable_background_boolean_builder(); }
    pub fn boolean_builder_worker_pending(&self) -> bool { self.boolean_builder_worker.is_some() }
    pub fn boolean_builder_worker_preview(&self) -> bool { self.boolean_builder_worker.as_ref().is_some_and(|job| job.preview && !job.commit_requested) }
    pub fn boolean_builder_worker_promoted(&self) -> bool { self.boolean_builder_worker.as_ref().is_some_and(|job| job.preview && job.commit_requested) }
    pub fn promote_boolean_builder_preview(&mut self) -> Result<()> {
        let job = self.boolean_builder_worker.as_mut().ok_or(Error::Missing("Boolean preview worker"))?;
        if !job.preview { return Err(Error::Busy); }
        job.commit_requested = true;
        Ok(())
    }
    pub fn cancel_boolean_builder_worker(&mut self) { drop(self.boolean_builder_worker.take()); }
    pub fn cancel_boolean_builder_worker_for_input(&mut self) {
        let preview = self.boolean_builder_worker_preview();
        self.cancel_boolean_builder_worker();
        if preview { self.invalidate_boolean_preview(); }
    }
    pub fn boolean_builder_worker_retiring(&self) -> bool {
        self.boolean_builder_worker.is_none() && self.boolean_builder_active.load(Ordering::Acquire)
    }

    pub(super) fn request_boolean_builder_worker(&mut self, mut event: Event, preview: bool,
        roots: Option<Vec<om9_core::identity::ObjectId>>, k: f64) -> Result<()> {
        if self.boolean_builder_worker.is_some() || self.boolean_builder_active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return Err(Error::Busy);
        }
        let permit = Permit(self.boolean_builder_active.clone());
        let scene = self.scene.as_ref().ok_or(Error::Missing("Boolean scene"))?.publication_id();
        let held = self.scene_cache_usage()?;
        let document = self.coordinator.capture_tracked_document(self.document)?;
        let max_depth = self.coordinator.limits().max_depth;
        let tolerance = document.effective_tolerance()?.absolute_mm;
        let limits = *self.coordinator.limits();
        let operands = if roots.is_none() { Some(super::mesh_boolean::Operands::read(&event)?) } else { None };
        let selection_bytes = roots.iter().flatten().chain(operands.iter().flat_map(|op| op.ids())).try_fold(0u64, |sum, id| {
            let entity = document.entities.get(id).ok_or(Error::Missing("Boolean source"))?;
            let bytes = if let Some((mesh, _, _)) = super::addon_session::boolean_mesh_source(&document, entity, max_depth)? {
                let faces = mesh.faces.iter().try_fold(0u64, |n, face| n.checked_add(face.len().saturating_sub(2) as u64).ok_or(Error::Overflow("Boolean input")))?;
                (mesh.vertices.len() as u64).saturating_mul(24).saturating_add(faces.saturating_mul(12))
            } else if operands.is_some() {
                use om9_core::geometry::Geometry;
                let source_bytes = match &entity.geometry {
                    Geometry::Brep(b) => b.source().len() as u64,
                    Geometry::NativeBrep(b) => b.storage_bytes()?,
                    Geometry::Surface(s) => (s.poles.len() as u64).saturating_mul(32),
                    _ => return Err(Error::Invalid("select a mesh, surface or polysurface")),
                };
                source_bytes.saturating_add((CsgOptions::default().max_triangles as u64).saturating_mul(84))
            } else { 0 };
            sum.checked_add(bytes).ok_or(Error::Overflow("Boolean input"))
        })?;
        // This is an admission allowance, not a proved CSG peak bound.
        let bytes = value_bytes(&event.state)?.checked_add(selection_bytes).and_then(|n| n.checked_mul(8))
            .and_then(|n| n.checked_add(32 * 1024 * 1024)).ok_or(Error::Overflow("Boolean worker memory"))?;
        let owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes,
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes)), held)?;
        let cancel = Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        let job = BooleanBuilderJob { receiver, cancel, authority: self.coordinator.authority(), epoch: self.coordinator.epoch(),
            scene, selection_revision: self.selection_state().revision, command_run: self.command_run,
            command: event.command.clone(), builder_revision: self.addon_builder.as_ref().map(|builder| builder.revision), preview, commit_requested: false };
        std::thread::Builder::new().name("om9-boolean-builder".into()).spawn(move || {
            let _permit = permit;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(operands) = operands {
                    return operands.compute(&document, event, &limits, &thread_cancel);
                }
                event.selection = prepare_selection(&document, roots.as_deref().unwrap_or_default(), k, tolerance, max_depth, &thread_cancel)?;
                calculate(&event, &thread_cancel, preview, None)
            }))
                .unwrap_or(Err(Error::Io("Boolean worker panicked".into())))
                .map(|reply| (reply, owner));
            let _ = sender.send(result);
        }).map_err(|e| Error::Io(format!("Boolean worker: {e}")))?;
        self.boolean_builder_worker = Some(job);
        Ok(())
    }

    pub fn poll_boolean_builder_worker(&mut self) -> Option<Result<()>> {
        let job = self.boolean_builder_worker.take()?;
        if !job.current(self) {
            self.invalidate_boolean_preview();
            return Some(Err(Error::Stale("Boolean worker context")));
        }
        let output = match job.receiver.try_recv() {
            Err(mpsc::TryRecvError::Empty) => { self.boolean_builder_worker = Some(job); return None; },
            Err(mpsc::TryRecvError::Disconnected) => return Some(Err(Error::Io("Boolean worker disconnected".into()))),
            Ok(result) => result,
        };
        Some(output.and_then(|(mut reply, _owner)| {
            if job.preview && !job.commit_requested { return self.accept_boolean_worker_preview(reply); }
            // A promoted preview no longer needs wire geometry during publication.
            reply.preview = Vec::new();
            let session = self.addon_session.as_ref().ok_or(Error::Stale("Boolean session"))?;
            let (code, label, seen) = (session.code.clone(), session.label.clone(), session.seen.clone());
            let created = self.commit_session_result(&code, &label, &reply, &seen)?;
            self.addon_messages.extend(reply.messages.iter().map(|message| self.tr.text(message)));
            self.addon_session = None;
            if super::mesh_boolean::MeshBoolean::from_name(&code).is_some() { self.addon_last = Some(code.clone()); }
            if let Some(builder) = self.addon_builder.take_if(|builder| builder.interactive && builder.code == code) {
                self.store_builder_values(&builder);
            }
            if self.join_history_worker_pending() {
                self.events.emit(crate::command_events::EventKind::Progress, &code, "updating History", None);
            } else {
                self.events.emit(crate::command_events::EventKind::Finished, &code, &format!("created {created} objects"),
                    Some(crate::command_events::EventOutcome::Committed));
            }
            Ok(())
        }))
    }
}
