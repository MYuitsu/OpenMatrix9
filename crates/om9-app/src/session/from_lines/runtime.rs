// SPDX-License-Identifier: LGPL-2.1-or-later
use super::CageRecord;
use super::{CODE, prepare};
use crate::session::Controller;
use om9_addon_abi::session::{Input, OptionToken, Reply, Want};
use om9_addon_geom::subd::{
    Edge,
    from_lines::{FaceLayout, LineCage, NetworkFace},
};
use om9_core::{
    Error, Result,
    budget::{Limits, memory::MemoryClass},
    geometry::{Geometry, Mesh},
    identity::{DocumentId, ObjectId},
    model::{Entity, Extension, TSPLINE_FROM_LINES_SLOT},
    transaction::{Cancellation, ExternalMemoryReservation, Mutation, TransactionRequest},
};
use std::sync::{Arc, mpsc};

struct Preview {
    layout: FaceLayout,
    _memory: ExternalMemoryReservation,
}
enum Operation {
    Prepare,
    Choices(Edge),
    Remove(usize),
    Add(NetworkFace),
    Finish,
}
enum Outcome {
    Layout(FaceLayout),
    Choices(Vec<NetworkFace>),
    Product(Entity),
}
struct Job {
    rx: mpsc::Receiver<Result<(Outcome, ExternalMemoryReservation)>>,
    cancel: Cancellation,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
pub(in crate::session) struct FromLines {
    document: DocumentId,
    epoch: u64,
    sources: Vec<ObjectId>,
    file: Option<f64>,
    custom: Option<f64>,
    use_file: bool,
    auto: usize,
    manual: usize,
    snapping: bool,
    smooth: bool,
    preview: Option<Arc<Preview>>,
    choices: Vec<NetworkFace>,
    choices_memory: Option<ExternalMemoryReservation>,
    job: Option<Job>,
    finished: bool,
}
impl FromLines {
    pub(in crate::session) fn current(&self, c: &Controller) -> bool {
        c.document == self.document && c.coordinator.epoch() == self.epoch
    }
    fn capture(c: &Controller) -> Result<Option<Self>> {
        let sources = c.selected_ordered();
        if sources.is_empty() {
            return Ok(None);
        }
        if sources
            .iter()
            .any(|id| !matches!(c.document().entities[id].geometry, Geometry::Curve(_)))
        {
            return Err(Error::Invalid("Select lines or polylines"));
        }
        let file = c
            .document()
            .tolerance()
            .map(|t| t.absolute_mm / c.document().page_unit.factor());
        Ok(Some(Self {
            document: c.document,
            epoch: c.coordinator.epoch(),
            sources,
            file,
            custom: file,
            use_file: file.is_some(),
            auto: 4,
            manual: 4,
            snapping: false,
            smooth: true,
            preview: None,
            choices: Vec::new(),
            choices_memory: None,
            job: None,
            finished: false,
        }))
    }
    fn tolerance(&self) -> Option<f64> {
        if self.use_file {
            self.file
        } else {
            self.custom
        }
    }
    fn clear_choices(&mut self) {
        self.choices.clear();
        self.choices_memory = None;
    }
    fn launch(&mut self, c: &Controller, op: Operation) -> Result<()> {
        if !self.current(c) {
            return Err(Error::Stale("From Lines inputs changed"));
        }
        if self.job.is_some() {
            return Err(Error::Busy);
        }
        let tolerance = self
            .tolerance()
            .ok_or(Error::Missing("enter a positive tolerance"))?;
        let prior = self.preview.clone();
        if !matches!(op, Operation::Prepare) && prior.is_none() {
            return Err(Error::Missing("From Lines preview"));
        }
        let limits = *c.coordinator.limits();
        let held = c.scene_cache_usage()?;
        let peer = c.coordinator.capture_transaction_preparation_peer(held)?;
        let doc_id = self.document;
        let source_ids = self.sources.clone();
        let mut base = 128 * 1024u64;
        for id in &source_ids {
            let usage = c.document().entities[id].geometry.validate(&limits)?;
            base = base.saturating_add(usage.bytes.saturating_mul(12));
        }
        // The same wire can outlive an old layout when faces are edited.
        // Re-admit its storage for the new preview before releasing the old one.
        if let Some(old) = &prior {
            base = base
                .saturating_add((old.layout.network().vertices.len() as u64).saturating_mul(192));
        }
        let physical =
            || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = c.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache,
            base,
            physical(),
            held,
        )?;
        let (auto, manual, smooth) = (self.auto, self.manual, self.smooth);
        let cancel = Cancellation::default();
        let flag = cancel.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("om9-from-lines".into())
            .spawn(move || {
                let coordinator = peer.coordinator();
                let snapshot = &coordinator.world().documents[&doc_id];
                let mut units = 0u64;
                let mut spend = |n: usize| -> Result<()> {
                    if flag.is_canceled() {
                        return Err(Error::Canceled);
                    }
                    units = units.saturating_add(n as u64);
                    if units > limits.max_nodes.saturating_mul(32) {
                        return Err(Error::Budget("From Lines face search"));
                    }
                    // Charge candidate copies as the walk expands; grow in chunks to avoid
                    // querying physical RAM at every edge. Each accepted cycle charges its length.
                    let wanted = base.saturating_add(units.saturating_mul(8));
                    if wanted > reservation.bytes() {
                        let growth = (wanted - reservation.bytes()).max(256 * 1024);
                        coordinator.grow_external_memory_with_held(
                            &mut reservation,
                            growth,
                            physical(),
                            held,
                        )?;
                    }
                    Ok(())
                };
                let result = match op {
                    Operation::Prepare => {
                        prepare(snapshot, &source_ids, tolerance, auto, &limits, &mut spend)
                            .map(Outcome::Layout)
                    }
                    op => {
                        let layout = &prior.as_ref().unwrap().layout;
                        let mut work_error = None;
                        let mut check = |n| {
                            spend(n).map_err(|e| {
                                work_error = Some(e);
                                "From Lines preparation interrupted"
                            })
                        };
                        let result = (|| match op {
                            Operation::Choices(e) => {
                                layout.choices(e, manual, &mut check).map(Outcome::Choices)
                            }
                            Operation::Finish => {
                                product(layout, smooth, snapshot.current_layer, &limits, &mut check)
                                    .map(Outcome::Product)
                            }
                            Operation::Remove(_) | Operation::Add(_) => {
                                let mut faces = layout.faces().to_vec();
                                match op {
                                    Operation::Remove(i) if i < faces.len() => {
                                        faces.remove(i);
                                    }
                                    Operation::Add(face) => faces.push(face),
                                    _ => return Err("unknown preview face"),
                                }
                                FaceLayout::new(layout.network().clone(), faces, &mut check)
                                    .map(Outcome::Layout)
                            }
                            Operation::Prepare => unreachable!(),
                        })();
                        result.map_err(|s| work_error.unwrap_or(Error::Invalid(s)))
                    }
                };
                let _ = tx.send(result.map(|o| (o, reservation)));
            })
            .map_err(|e| Error::Io(e.to_string()))?;
        self.job = Some(Job { rx, cancel });
        self.clear_choices();
        Ok(())
    }
    fn reply(&self) -> Reply {
        let prompt = if self.job.is_some() {
            "Preparing From Lines"
        } else if !self.choices.is_empty() {
            "Choose a face"
        } else {
            "Click a face to remove it or an edge to add a face. Enter to finish"
        };
        let mut r = Reply::ask(prompt, serde_json::Value::Null).want(Want::Nothing);
        let mut tol = OptionToken::number("Tolerance", None);
        tol.value = self.tolerance().map(|v| v.to_string());
        r.options = vec![
            OptionToken::toggle("LineSnapping", self.snapping),
            OptionToken::toggle("UseFileTolerance", self.use_file),
            tol,
            OptionToken::number("MaxAutoFace", Some(self.auto as f64)),
            OptionToken::number("MaxManualFace", Some(self.manual as f64)),
            OptionToken::toggle("Smooth", self.smooth),
        ];
        r
    }
}
fn product(
    layout: &FaceLayout,
    smooth: bool,
    layer: Option<om9_core::identity::LayerId>,
    limits: &Limits,
    work: &mut impl FnMut(usize) -> std::result::Result<(), &'static str>,
) -> std::result::Result<Entity, &'static str> {
    let source = layout.cage(work)?;
    let (mesh, ext) = render_record(source, smooth, limits, work)?;
    let mut e = Entity::new("T-Spline From Lines", Geometry::Mesh(mesh), layer);
    if layer.is_some() {
        e.attributes.color = None;
    }
    e.extensions.insert(TSPLINE_FROM_LINES_SLOT.into(), ext);
    Ok(e)
}
fn render_record(
    source: LineCage,
    smooth: bool,
    limits: &Limits,
    work: &mut impl FnMut(usize) -> std::result::Result<(), &'static str>,
) -> std::result::Result<(Mesh, Extension), &'static str> {
    let display = source.display(usize::from(smooth), work)?;
    let (control, polygon_face_ids) = source.into_parts();
    let (display, display_face_ids) = display.into_parts();
    let record = CageRecord {
        vertices: control.verts,
        polygons: control.faces,
        polygon_face_ids,
        display_face_ids,
        smooth,
    };
    let mesh = Mesh {
        vertices: display.verts,
        faces: display.faces,
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    mesh.validate(limits)
        .map_err(|_| "invalid From Lines display mesh or document limit")?;
    let data = serde_json::to_string(&record).map_err(|_| "From Lines cage serialization")?;
    if data.len() as u64 > limits.max_bytes {
        return Err("From Lines cage payload budget");
    }
    Ok((
        mesh,
        Extension::new(1, data).map_err(|_| "From Lines control cage")?,
    ))
}

use crate::session::context_presets::toggle;

/// Resolve display picks to logical control faces before changing either geometry or metadata.
pub(in crate::session) fn face_subset(
    entity: &Entity,
    picks: &std::collections::BTreeSet<u32>,
    side: om9_core::top11::MeshFaceSide,
    limits: &Limits,
) -> Result<Option<(Mesh, Extension)>> {
    use om9_addon_geom::subd::Poly;
    use om9_core::top11::MeshFaceSide;
    use std::collections::BTreeSet;
    let record = CageRecord::read(entity)?;
    let Geometry::Mesh(current) = &entity.geometry else {
        return Err(Error::Invalid("From Lines mesh"));
    };
    if !current.corner_uvs.is_empty()
        || !current.colors.is_empty()
        || !current.normals.is_empty()
        || entity
            .extension(om9_core::model::MESH_UV_EXTENSION_SLOT)
            .is_some()
    {
        return Err(Error::Unsupported(
            "From Lines face edit with mesh channels",
        ));
    }
    let selected = picks
        .iter()
        .map(|&i| {
            record
                .display_face_ids
                .get(i as usize)
                .copied()
                .ok_or(Error::Stale("From Lines display face"))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let mut remaining_work = limits.max_nodes.saturating_mul(32);
    let mut work = |n: usize| {
        remaining_work = remaining_work
            .checked_sub(n as u64)
            .ok_or("From Lines face edit budget")?;
        Ok(())
    };
    let source = LineCage::from_polygons(
        Poly {
            verts: record.vertices,
            faces: record.polygons,
        },
        record.polygon_face_ids,
    )
    .map_err(Error::Invalid)?;
    let baseline = source
        .display(usize::from(record.smooth), &mut work)
        .map_err(Error::Invalid)?;
    if baseline.poly().verts != current.vertices
        || baseline.poly().faces != current.faces
        || baseline.face_ids() != record.display_face_ids
    {
        return Err(Error::Stale("From Lines display geometry changed"));
    }
    drop(baseline);
    let (poly, ids) = source.into_parts();
    let (faces, ids): (Vec<_>, Vec<_>) = poly
        .faces
        .into_iter()
        .zip(ids)
        .filter(|(_, id)| selected.contains(id) == (side == MeshFaceSide::Selected))
        .unzip();
    if faces.is_empty() {
        return Ok(None);
    }
    let subset = LineCage::from_polygons(
        Poly {
            verts: poly.verts,
            faces,
        }
        .compacted(),
        ids,
    )
    .map_err(Error::Invalid)?;
    render_record(subset, record.smooth, limits, &mut work)
        .map(Some)
        .map_err(Error::Invalid)
}
impl Controller {
    pub(in crate::session) fn start_from_lines(&mut self) -> Result<()> {
        self.start_interactive_session(CODE, "From Lines", "om9".into(), false, false)
    }
    pub(in crate::session) fn from_lines_step(&mut self, input: &Input) -> Result<Reply> {
        let opening = self.from_lines.is_none();
        if opening && matches!(input, Input::Start | Input::Enter) {
            self.from_lines = FromLines::capture(self)?;
        }
        let Some(mut state) = self.from_lines.take() else {
            return Ok(
                Reply::ask("Select lines or polylines, Enter", serde_json::Value::Null)
                    .want(Want::Objects),
            );
        };
        if state.finished {
            return Ok(Reply {
                done: true,
                ..Default::default()
            });
        }
        if !state.current(self) {
            self.cancel_addon_session();
            return Err(Error::Stale("From Lines inputs changed"));
        }
        let result = (|| {
            let mut rebuild = opening;
            match input {
                Input::Start | Input::Selection => {}
                Input::Enter if opening => {}
                Input::Enter if !state.choices.is_empty() => {
                    return Err(Error::Invalid("choose a From Lines face first"));
                }
                Input::Enter => {
                    if state.preview.is_some() {
                        state.launch(self, Operation::Finish)?;
                    } else if state.job.is_none() {
                        rebuild = true;
                    }
                }
                Input::Option { name, value } => {
                    let value = value.as_deref();
                    match name.as_str() {
                        "UseFileTolerance" => {
                            let on = toggle(state.use_file, value)?;
                            if on && state.file.is_none() {
                                return Err(Error::Missing("file tolerance"));
                            }
                            state.use_file = on;
                            rebuild = true;
                        }
                        "Tolerance" if !state.use_file => {
                            state.custom = Some(
                                value
                                    .and_then(|v| v.parse().ok())
                                    .filter(|v: &f64| v.is_finite() && *v > 0.)
                                    .ok_or(Error::Invalid("Tolerance must be positive"))?,
                            );
                            rebuild = true;
                        }
                        "MaxAutoFace" | "MaxManualFace" => {
                            let n = value
                                .and_then(|s| s.parse::<usize>().ok())
                                .ok_or(Error::Invalid("enter a nonnegative edge count"))?;
                            if name == "MaxAutoFace" {
                                state.auto = n;
                                rebuild = true;
                            } else {
                                if state.job.is_some() {
                                    return Err(Error::Busy);
                                }
                                state.manual = n;
                                state.clear_choices();
                            }
                        }
                        "LineSnapping" => state.snapping = toggle(state.snapping, value)?,
                        "Smooth" => {
                            if state.job.is_some() {
                                return Err(Error::Busy);
                            }
                            state.smooth = toggle(state.smooth, value)?;
                        }
                        _ => return Err(Error::Invalid("From Lines option")),
                    }
                }
                _ => return Err(Error::Invalid("click an edge or face in the preview")),
            }
            if rebuild {
                state.job = None;
                state.preview = None;
                state.clear_choices();
                if state.tolerance().is_some() {
                    state.launch(self, Operation::Prepare)?;
                }
            }
            Ok(state.reply())
        })();
        self.from_lines = Some(state);
        result
    }
    pub fn from_lines_pending(&self) -> bool {
        self.from_lines.as_ref().is_some_and(|s| s.job.is_some())
    }
    pub fn from_lines_layout(&self) -> Option<&FaceLayout> {
        Some(&self.from_lines.as_ref()?.preview.as_ref()?.layout)
    }
    pub fn from_lines_snap_marks(&self) -> bool {
        self.from_lines.as_ref().is_some_and(|s| s.snapping)
    }
    pub fn from_lines_choices(&self) -> &[NetworkFace] {
        self.from_lines
            .as_ref()
            .map_or(&[], |s| s.choices.as_slice())
    }
    fn from_lines_operation(&mut self, op: Operation) -> Result<()> {
        let mut state = self
            .from_lines
            .take()
            .ok_or(Error::Missing("From Lines session"))?;
        let result = state.launch(self, op);
        self.from_lines = Some(state);
        result?;
        self.session_step(Input::Selection)
    }
    pub fn from_lines_edge(&mut self, edge: Edge) -> Result<()> {
        self.from_lines_operation(Operation::Choices(edge))
    }
    pub fn from_lines_remove(&mut self, index: usize) -> Result<()> {
        self.from_lines_operation(Operation::Remove(index))
    }
    pub fn from_lines_add(&mut self, index: usize) -> Result<()> {
        let face = self
            .from_lines_choices()
            .get(index)
            .ok_or(Error::Missing("From Lines candidate"))?
            .clone();
        self.from_lines_operation(Operation::Add(face))
    }
    pub fn from_lines_dismiss_choices(&mut self) -> Result<()> {
        if let Some(s) = self.from_lines.as_mut() {
            s.clear_choices();
        }
        self.session_step(Input::Selection)
    }
    pub fn poll_from_lines(&mut self) -> Option<Result<()>> {
        let state = self.from_lines.as_ref()?;
        let result = match state.job.as_ref()?.rx.try_recv() {
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(Error::Io("From Lines worker disconnected".into()))
            }
            Ok(output) => output,
        };
        if !state.current(self) {
            self.cancel_addon_session();
            return Some(Err(Error::Stale("From Lines inputs changed")));
        }
        let state = self.from_lines.as_mut()?;
        state.job = None;
        match result {
            Ok((Outcome::Layout(layout), memory)) => {
                state.preview = Some(Arc::new(Preview {
                    layout,
                    _memory: memory,
                }))
            }
            Ok((Outcome::Choices(choices), memory)) => {
                if choices.is_empty() {
                    self.addon_messages.push(format!(
                        "No candidate faces for this edge at MaxManualFace={}",
                        state.manual
                    ));
                }
                state.choices = choices;
                state.choices_memory = Some(memory);
            }
            Ok((Outcome::Product(entity), _memory)) => {
                let id = entity.id;
                let tx = TransactionRequest::new(
                    "From Lines",
                    vec![Mutation::Insert {
                        document: self.document,
                        entity,
                    }],
                );
                if let Err(e) = self.publish_request_with_selection(tx, Some(&[id])) {
                    return Some(Err(e));
                }
                if let Some(state) = self.from_lines.as_mut() {
                    state.finished = true;
                }
            }
            Err(error) => {
                let _ = self.session_step(Input::Selection);
                return Some(Err(error));
            }
        }
        Some(self.session_step(Input::Selection))
    }
}
