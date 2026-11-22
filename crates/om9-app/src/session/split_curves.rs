// SPDX-License-Identifier: LGPL-2.1-or-later
use super::Controller;
use om9_addon_abi::session::{Geo, Input, OptionToken, Reply, Want};
use om9_core::{
    Error, Result,
    budget::memory::MemoryClass,
    geometry::Geometry,
    identity::{DocumentId, ObjectId},
    top11::split_network::{self, SplitMember},
    transaction::{Cancellation, ExternalMemoryReservation, Mutation, TransactionRequest},
};
use std::sync::mpsc;

pub(super) const CODE: &str = "tsSplitCurves";

struct Output {
    members: Vec<SplitMember>,
    _memory: ExternalMemoryReservation,
}
struct Job {
    rx: mpsc::Receiver<Result<Output>>,
    cancel: Cancellation,
    accept: bool,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

pub(super) struct SplitCurves {
    doc: DocumentId,
    epoch: u64,
    sources: Vec<ObjectId>,
    file: Option<f64>,
    use_file: bool,
    custom: Option<f64>,
    job: Option<Job>,
    ready: Option<Output>,
}
impl SplitCurves {
    fn capture(c: &Controller) -> Result<Option<Self>> {
        let mut sources = Vec::new();
        for id in c.selected_ordered() {
            let entity = &c.document().entities[&id];
            if !matches!(entity.geometry, Geometry::Curve(_)) {
                continue;
            }
            if !c.document().permissions_for_entity(entity, false)?.editable {
                return Err(Error::Permission("selected curve is locked"));
            }
            sources.push(id);
        }
        if sources.len() < 2 {
            return Ok(None);
        }
        let file = c
            .document()
            .tolerance()
            .map(|t| t.absolute_mm / c.document().page_unit.factor());
        Ok(Some(Self {
            doc: c.document,
            epoch: c.coordinator.epoch(),
            sources,
            file,
            use_file: file.is_some(),
            custom: file,
            job: None,
            ready: None,
        }))
    }
    pub(super) fn current(&self, c: &Controller) -> bool {
        self.doc == c.document && self.epoch == c.coordinator.epoch()
    }
    fn tolerance(&self) -> Option<f64> {
        if self.use_file {
            self.file
        } else {
            self.custom
        }
    }
    fn edit(&mut self, name: &str, value: Option<&str>) -> Result<()> {
        match name {
            "UseFileTolerance" => {
                let enabled = match value.map(str::to_ascii_lowercase).as_deref() {
                    None => !self.use_file,
                    Some("yes" | "true" | "on") => true,
                    Some("no" | "false" | "off") => false,
                    _ => return Err(Error::Invalid("Use File Tolerance expects Yes or No")),
                };
                if enabled && self.file.is_none() {
                    return Err(Error::Missing("document tolerance"));
                }
                self.use_file = enabled;
            }
            "Tolerance" if !self.use_file => {
                self.custom = Some(
                    value
                        .and_then(|v| v.parse::<f64>().ok())
                        .filter(|v| v.is_finite() && *v > 0.)
                        .ok_or(Error::Invalid("Tolerance must be positive"))?,
                );
            }
            _ => {
                return Err(Error::Invalid(
                    "turn off Use File Tolerance to set Tolerance",
                ));
            }
        }
        self.job = None;
        self.ready = None;
        Ok(())
    }
    fn start(&mut self, c: &Controller, accept: bool) -> Result<()> {
        let tolerance = self
            .tolerance()
            .ok_or(Error::Missing("enter a positive tolerance"))?;
        let held = c.scene_cache_usage()?;
        let peer = c.coordinator.capture_transaction_preparation_peer(held)?;
        let document = c.document;
        let limits = *c.coordinator.limits();
        let sources = self.sources.clone();
        let mut bytes = 0u64;
        for id in &sources {
            bytes =
                bytes.saturating_add(c.document().entities[id].geometry.validate(&limits)?.bytes);
        }
        let mut memory = c.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache,
            bytes.saturating_mul(8).saturating_add(1024 * 1024),
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            c.scene_cache_usage()?,
        )?;
        let cancel = Cancellation::default();
        let flag = cancel.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("om9-split-curves".into())
            .spawn(move || {
                let coordinator = peer.coordinator();
                let doc = &coordinator.world().documents[&document];
                let members = split_network::split_admitted(
                    sources.iter().map(|id| &doc.entities[id]),
                    tolerance,
                    &limits,
                    &|| flag.is_canceled(),
                    &mut |required| {
                        if required > memory.bytes() {
                            let extra = required - memory.bytes();
                            coordinator.grow_external_memory_with_held(
                                &mut memory,
                                extra,
                                om9_native_owner::physical_memory()
                                    .map(|s| (s.total_bytes, s.available_bytes)),
                                held,
                            )?;
                        }
                        Ok(())
                    },
                );
                let _ = tx.send(members.map(|members| Output {
                    members,
                    _memory: memory,
                }));
            })
            .map_err(|e| Error::Io(e.to_string()))?;
        self.job = Some(Job { rx, cancel, accept });
        Ok(())
    }
    fn reply(&self, c: &Controller) -> Reply {
        let mut reply = Reply::ask("Split curves at intersections", serde_json::Value::Null)
            .want(Want::Nothing);
        let mut tolerance = OptionToken::number("Tolerance", None);
        tolerance.value = self.tolerance().map(|n| n.to_string());
        reply.options = vec![
            OptionToken::toggle("UseFileTolerance", self.use_file),
            tolerance,
            OptionToken::action("Display"),
        ];
        reply
            .notes
            .push(("@when".into(), "Tolerance|UseFileTolerance=No".into()));
        if self.file.is_none() {
            reply.notes.push((
                "UseFileTolerance".into(),
                "Document has no file tolerance".into(),
            ));
        }
        if self.job.is_some() {
            reply
                .notes
                .push(("".into(), "Finding intersections…".into()));
        }
        if let Some(ready) = &self.ready {
            let k = c.document().page_unit.factor();
            for m in &ready.members {
                reply.preview.extend(m.points.iter().map(|p| Geo::Point {
                    point: p.map(|v| v * k),
                }));
            }
            if ready.members.is_empty() {
                reply
                    .notes
                    .push(("".into(), "No curves to split at this tolerance".into()));
            }
        }
        reply
    }
    fn apply(&mut self, c: &mut Controller) -> Result<Reply> {
        let output = self
            .ready
            .take()
            .ok_or(Error::Missing("Split Curves result"))?;
        let mut mutations = Vec::new();
        let mut selection = Vec::new();
        for m in output.members {
            let source = c.document().stamp(m.source)?;
            mutations.push(Mutation::Delete { source });
            let original = &c.document().entities[&m.source];
            for piece in m.pieces {
                let entity = super::top11::child_entity(original, Geometry::Curve(piece));
                selection.push(entity.id);
                mutations.push(Mutation::Insert {
                    document: c.document,
                    entity,
                });
            }
        }
        if !mutations.is_empty() {
            c.publish_request_with_selection(
                TransactionRequest::new("Split Curves", mutations),
                Some(&selection),
            )?;
        }
        Ok(Reply {
            done: true,
            messages: vec![format!("Split Curves: {} new segments", selection.len())],
            ..Default::default()
        })
    }
}
impl Controller {
    pub(super) fn start_split_curves(&mut self) -> Result<()> {
        self.start_interactive_session(CODE, "tsSplitCurves", "om9".into(), false, false)
    }
    pub(super) fn split_curves_step(&mut self, input: &Input) -> Result<Reply> {
        let opening = self.split_curves.is_none();
        if opening && matches!(input, Input::Start | Input::Enter) {
            self.split_curves = SplitCurves::capture(self)?;
        }
        let Some(mut state) = self.split_curves.take() else {
            return Ok(
                Reply::ask("Select curves to split, Enter", serde_json::Value::Null)
                    .want(Want::Objects),
            );
        };
        if !state.current(self) {
            self.cancel_addon_session();
            return Err(Error::Stale("Split Curves inputs changed"));
        }
        let result = (|| {
            match input {
                Input::Option { name, .. } if name == "Display" => {
                    if state.job.is_none() && state.ready.is_none() {
                        state.start(self, false)?;
                    }
                }
                Input::Option { name, value } => state.edit(name, value.as_deref())?,
                Input::Enter if !opening => {
                    if state.ready.is_some() {
                        return state.apply(self);
                    }
                    if let Some(job) = state.job.as_mut() {
                        job.accept = true;
                    } else {
                        state.start(self, true)?;
                    }
                }
                Input::Start | Input::Enter | Input::Selection => {}
                _ => {
                    return Err(Error::Invalid(
                        "choose a Split Curves option or press Enter",
                    ));
                }
            }
            Ok(state.reply(self))
        })();
        if !result.as_ref().is_ok_and(|r| r.done) {
            self.split_curves = Some(state);
        }
        result
    }
    pub fn split_curves_pending(&self) -> bool {
        self.split_curves.as_ref().is_some_and(|s| s.job.is_some())
    }
    pub fn poll_split_curves(&mut self) -> Option<Result<()>> {
        let state = self.split_curves.as_mut()?;
        let job = state.job.as_ref()?;
        let accept = job.accept;
        let result = match job.rx.try_recv() {
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(Error::Io("Split Curves worker disconnected".into()))
            }
            Ok(result) => result,
        };
        state.job = None;
        match result {
            Ok(output) => state.ready = Some(output),
            Err(error) => {
                let _ = self.session_step(Input::Selection);
                return Some(Err(error));
            }
        }
        Some(self.session_step(if accept {
            Input::Enter
        } else {
            Input::Selection
        }))
    }
}
