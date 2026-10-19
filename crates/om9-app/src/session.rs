// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Unit, finite},
    identity::DocumentId,
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use om9_scene::{camera::CPlane, scene::Scene, workspace::Workspace};
use std::{collections::VecDeque, sync::Arc};
pub mod contract;
pub mod gumball_numeric;
pub mod stress;
mod subobjects;
pub use subobjects::SelectedSubobject;
mod gumball;
mod files;
mod layer_dialogs;
mod layers;
pub const LINE: &str = "OM9-CURVE-002";
pub const POINT: &str = "OM9-CURVE-034";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationOrigin {
    Menu,
    Cmd,
    F6,
    Builder,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    Command,
    File(crate::file_lifecycle::FileOutcome),
}
/// Which step of an interactive command is waiting for the user
/// (OM9-L3-01-02-004). It is derived from the one session state, so the prompt,
/// the audit and the refusals cannot disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdPhase {
    /// No command is running.
    Idle,
    /// Waiting for the first point.
    AwaitPoint,
    /// Waiting for the end point of a two-point command.
    AwaitEnd,
}

/// The session facts one command run was started against (OM9-L3-01-02-003):
/// which document, which view, what was selected, and the revisions that make
/// the snapshot checkable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContext {
    pub document: DocumentId,
    pub view: om9_scene::workspace::ViewId,
    pub selection: Vec<om9_core::identity::ObjectId>,
    pub revision: u64,
    pub epoch: u64,
}

#[derive(Debug, Clone)]
struct CommandSession {
    feature: &'static str,
    context: CommandContext,
    plane: CPlane,
    unit: Unit,
    start: Option<Point3>,
    both_sides: bool,
    /// `Length=…` fixes the end point's distance from the start; the click then
    /// only chooses the direction (OM9-L3-01-02-005).
    length_mm: Option<f64>,
}
pub struct Controller {
    coordinator: TransactionCoordinator,
    /// Gumball placement: an explicit origin (relocate) and the alignment of its axes.
    gumball_origin: Option<om9_core::geometry::Point3>,
    gumball_alignment: crate::session::gumball::GumballAlignment,
    /// Faces, edges and vertices picked with Ctrl+Shift+click, in pick order.
    subselection: Vec<SelectedSubobject>,
    sub_revision: u64,
    /// Bumped while a gumball drag previews, so every viewport redraws the moving object.
    preview_revision: u64,
    document: DocumentId,
    scene: Option<Scene>,
    session: Option<CommandSession>,
    recent: VecDeque<String>,
    repeat: Option<String>,
    view_menu_generation: u64,
    selection: std::collections::BTreeSet<om9_core::identity::ObjectId>,
    /// Bumped whenever the selection set changes, so a published selection
    /// state has a revision a caller can compare.
    selection_revision: u64,
    layer_dialog: Option<crate::layer_dialog::LayerDialog>,
    layer_dialog_serial: u64,
    file: crate::file_lifecycle::FileState,
    /// The ordered run events of this session (OM9-L3-01-02-010).
    pub events: crate::command_events::EventLog,
    /// Document change observers (OM9-L3-01-03-011). They receive an owned batch
    /// after the document is final and cannot reach the controller.
    observers: Vec<Box<dyn contract::DocumentObserver>>,
    observer_diagnostics: Vec<contract::ObserverDiagnostic>,
    changes: VecDeque<om9_core::changes::ChangeBatch>,
    /// The revision the change batches are diffed against. It is the world's own
    /// `Arc`, so recording costs an `Arc` clone, never a document copy.
    published: Option<std::sync::Arc<Document>>,
    pub workspace: Workspace,
}
impl Controller {
    pub fn capture_view_menu(
        &self,
        id: om9_scene::workspace::ViewId,
    ) -> Result<crate::viewport_menu::ViewMenuCapture> {
        let (context, _) = self.coordinator.capture_context(None)?;
        let view = self
            .workspace
            .views
            .get(&id)
            .ok_or(Error::Missing("viewport menu target"))?;
        if view.id != id {
            return Err(Error::Invalid("viewport menu key/identity"));
        }
        Ok(crate::viewport_menu::ViewMenuCapture::new(
            context,
            self.document(),
            view,
            &self.workspace,
            self.view_menu_generation,
        ))
    }
    pub fn validate_view_menu(
        &self,
        capture: &crate::viewport_menu::ViewMenuCapture,
    ) -> Result<()> {
        let (context, _) = self.coordinator.capture_context(None)?;
        capture.validate(
            context,
            self.document(),
            &self.workspace,
            self.view_menu_generation,
        )
    }
    pub fn apply_view_menu(
        &mut self,
        capture: &crate::viewport_menu::ViewMenuCapture,
        action: crate::viewport_menu::MenuAction,
    ) -> Result<()> {
        self.validate_view_menu(capture)?;
        action.availability()?;
        let generation = self
            .view_menu_generation
            .checked_add(1)
            .ok_or(Error::Overflow("viewport menu generation"))?;
        match action {
            crate::viewport_menu::MenuAction::Maximize => {
                self.workspace.toggle_maximize(capture.target())
            }
            crate::viewport_menu::MenuAction::Preset(
                crate::viewport_menu::DisplayPreset::Wireframe,
            ) => {
                let view = self
                    .workspace
                    .views
                    .get_mut(&capture.target())
                    .ok_or(Error::Missing("viewport menu target"))?;
                if view.mode != om9_scene::workspace::DisplayMode::Wireframe {
                    let revision = view
                        .revision
                        .checked_add(1)
                        .ok_or(Error::Overflow("view revision"))?;
                    view.mode = om9_scene::workspace::DisplayMode::Wireframe;
                    view.revision = revision;
                }
                Ok(())
            }
            _ => unreachable!("availability admitted only implemented actions"),
        }?;
        self.view_menu_generation = generation;
        Ok(())
    }
    pub fn new(document: Document, limits: Limits) -> Result<Self> {
        Self::new_with_held(document, limits, om9_core::budget::Usage::default())
    }
    fn new_with_held(
        document: Document,
        limits: Limits,
        held: om9_core::budget::Usage,
    ) -> Result<Self> {
        let mut held = held;
        held.add(Workspace::startup_usage()?, &limits)?;
        let id = document.id;
        let remaining = limits.remaining_after(held)?;
        let mut peak = held;
        peak.add(document.validate(&remaining)?, &limits)?;
        peak.add(
            om9_core::budget::Usage::btree::<DocumentId, Arc<Document>>(1)?,
            &limits,
        )?;
        peak.add(om9_core::budget::Usage::array::<Document>(1)?, &limits)?;
        let file = crate::file_lifecycle::FileState::initialized(&document)?;
        let coordinator = TransactionCoordinator::new_with_held(
            World::new(vec![document], &remaining)?,
            limits,
            held,
        )?;
        let scene = Scene::capture_in_world(&coordinator, id, held)?;
        Ok(Self {
            coordinator,
            gumball_origin: None,
            gumball_alignment: crate::session::gumball::GumballAlignment::World,
            subselection: Vec::new(),
            sub_revision: 1,
            preview_revision: 1,
            document: id,
            scene: Some(scene),
            session: None,
            recent: VecDeque::new(),
            repeat: None,
            view_menu_generation: 0,
            selection: std::collections::BTreeSet::new(),
            selection_revision: 1,
            layer_dialog: None,
            layer_dialog_serial: 0,
            file,
            events: crate::command_events::EventLog::new(),
            observers: Vec::new(),
            observer_diagnostics: Vec::new(),
            changes: VecDeque::new(),
            published: None,
            workspace: Workspace::new(),
        })
    }
    pub fn coordinator(&self) -> &TransactionCoordinator {
        &self.coordinator
    }
    pub fn document(&self) -> &Document {
        &self.coordinator.world().documents[&self.document]
    }
    pub fn scene(&self) -> Option<&Scene> {
        self.scene.as_ref()
    }
    /// The command an empty Enter would run again: the last completed command
    /// that may be repeated (OM9-L3-01-02-009). Only the interactive geometry
    /// commands are repeatable; a file or destructive command is never repeated
    /// silently, and never enters the repeat slot.
    pub fn repeatable_last(&self) -> Option<&str> {
        self.repeat
            .as_deref()
            .filter(|feature| Self::is_repeatable(feature))
    }
    /// Whether a command may be started again by an empty Enter. A command is
    /// named the way the registry names it: its stable feature id, its command
    /// id or any alias (OM9-L3-01-02-001), so `Line`, `line`, `Ln` and the
    /// feature id all answer the same way.
    pub fn is_repeatable(feature: &str) -> bool {
        let wanted = feature.trim();
        let line = Self::line_descriptor();
        line.id.eq_ignore_ascii_case(wanted)
            || line.feature.eq_ignore_ascii_case(wanted)
            || line
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(wanted))
            // POINT has no descriptor yet; its own item adds one.
            || "point".eq_ignore_ascii_case(wanted)
            || POINT.eq_ignore_ascii_case(wanted)
    }
    /// The repeat slot as stored, for the audit: it is either empty or a
    /// repeatable feature.
    pub fn repeat_slot(&self) -> Option<&str> {
        self.repeat.as_deref()
    }
    pub fn recent(&self) -> &VecDeque<String> {
        &self.recent
    }
    pub fn is_running(&self) -> bool {
        // A command whose document, view, revision or epoch is gone is NOT
        // running: closing or replacing the active document cancels it instead of
        // leaving a prompt that can only fail later (OM9-L3-01-02-003).
        match &self.session {
            None => false,
            Some(s) => self.validate_context(&s.context).is_ok(),
        }
    }
    /// Drop a session whose context is gone. Returns true when one was dropped,
    /// so a caller (the window, the command line) can report the cancellation.
    pub fn cancel_if_stale(&mut self) -> bool {
        if self.session.is_some() && !self.is_running() {
            self.session = None;
            return true;
        }
        false
    }
    pub fn availability(feature: &str) -> Result<()> {
        if crate::file_lifecycle::FileKind::from_feature(feature).is_some() {
            return Ok(());
        }
        match feature {
            LINE | POINT | "MeshStress" => Ok(()),
            _ => Err(Error::Unsupported(
                "command has no accepted standalone operation",
            )),
        }
    }
    /// The descriptor of the geometry commands this session runs: the option names
    /// the parser checks against (OM9-L3-01-02-005). The registry (OM9-L3-01-02-001)
    /// will own the catalogue; the session reads the same shape meanwhile.
    pub fn line_descriptor() -> om9_core::capability::Operation {
        om9_core::capability::Operation {
            id: "line".to_owned(),
            feature: LINE.to_owned(),
            aliases: vec!["Line".into(), "Ln".into()],
            prompt: "Start of line (BothSides=No): ".into(),
            input_types: vec![
                om9_core::geometry::GeometryKind::Point,
                om9_core::geometry::GeometryKind::Point,
            ],
            requires_input: true,
            output: vec![om9_core::geometry::GeometryKind::Curve],
            options: vec!["BothSides".into(), "Length".into()],
            preconditions: om9_core::capability::Preconditions {
                needs_document: true,
                needs_selection: false,
                needs_active_view: true,
            },
            backend: om9_core::capability::Backend::Rust {
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            prepare: true,
            preview: true,
            commit: true,
            receipts: Vec::new(),
        }
    }
    pub fn dispatch(&mut self, feature: &str, _origin: InvocationOrigin) -> Result<()> {
        self.dispatch_effect(feature, _origin).map(|_| ())
    }
    pub fn dispatch_effect(
        &mut self,
        feature: &str,
        origin: InvocationOrigin,
    ) -> Result<DispatchOutcome> {
        if let Some(kind) = crate::file_lifecycle::FileKind::from_feature(feature) {
            return self.begin_file_operation(kind).map(DispatchOutcome::File);
        }
        self.dispatch_geometry(feature, origin)
            .map(|_| DispatchOutcome::Command)
    }
    fn dispatch_geometry(&mut self, feature: &str, _origin: InvocationOrigin) -> Result<()> {
        Self::availability(feature)?;
        if self.is_running() {
            return Err(Error::Busy);
        }
        let feature = match feature {
            LINE => LINE,
            POINT => POINT,
            "MeshStress" => {
                // Menu or search invocation: the same command as typing `MeshStress` in the line.
                self.insert_stress_meshes(self::stress::DEFAULT_STRESS_MESHES)?;
                return Ok(());
            }
            _ => unreachable!(),
        };
        let view = &self.workspace.views[&self.workspace.active];
        let context = CommandContext {
            document: self.document,
            view: self.workspace.active,
            selection: self.selection.iter().copied().collect(),
            revision: self.document().context_revision,
            epoch: self.coordinator.epoch(),
        };
        self.session = Some(CommandSession {
            feature,
            context,
            plane: view.cplane,
            unit: self.document().page_unit,
            start: None,
            both_sides: false,
            length_mm: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "started",
            None,
        );
        Ok(())
    }
    /// The session facts a command would start against right now.
    pub fn context(&self) -> CommandContext {
        CommandContext {
            document: self.document,
            view: self.workspace.active,
            selection: self.selection.iter().copied().collect(),
            revision: self.document().context_revision,
            epoch: self.coordinator.epoch(),
        }
    }
    /// Refuse a snapshot that no longer describes the session: another document,
    /// another active view, another revision or another publication epoch.
    pub fn validate_context(&self, context: &CommandContext) -> Result<()> {
        if context.document != self.document
            || !self.workspace.views.contains_key(&context.view)
            || context.revision != self.document().context_revision
            || context.epoch != self.coordinator.epoch()
        {
            return Err(Error::Stale("command document/view context"));
        }
        Ok(())
    }
    /// Which step of the running command is waiting for input. A stale session
    /// reads as Idle, exactly as `is_running` reports it.
    pub fn phase(&self) -> CmdPhase {
        if !self.is_running() {
            return CmdPhase::Idle;
        }
        match &self.session {
            None => CmdPhase::Idle,
            Some(s) if s.feature == POINT => CmdPhase::AwaitPoint,
            Some(s) if s.start.is_none() => CmdPhase::AwaitPoint,
            Some(_) => CmdPhase::AwaitEnd,
        }
    }
    pub fn prompt(&self) -> &'static str {
        // A session whose context is gone asks for nothing: the prompt and the
        // phase machine must never disagree (OM9-L3-01-02-004 N5).
        if !self.is_running() {
            return "Command: ";
        }
        match &self.session {
            None => "Command: ",
            Some(s) if s.feature == POINT => "Location of point: ",
            Some(s) => match (s.start.is_some(), s.both_sides) {
                (false, false) => "Start of line (BothSides=No): ",
                (false, true) => "Start of line (BothSides=Yes): ",
                (true, false) => "End of line (BothSides=No): ",
                (true, true) => "End of line (BothSides=Yes): ",
            },
        }
    }
    fn validate_session(&self) -> Result<&CommandSession> {
        let s = self
            .session
            .as_ref()
            .ok_or(Error::Missing("active command"))?;
        self.validate_context(&s.context)?;
        Ok(s)
    }
    pub fn set_cplane(&mut self, plane: CPlane) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        plane.validate()?;
        self.workspace
            .views
            .get_mut(&self.workspace.active)
            .expect("active view")
            .cplane = plane;
        Ok(())
    }
    pub fn cancel(&mut self) {
        if let Some(s) = self.session.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                s.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
    }
    /// Record a refusal that happened inside a running command (a bad option, a
    /// degenerate point): the run continues, and the log says what was refused.
    pub fn warn(&mut self, detail: &str) {
        let Some(feature) = self.session.as_ref().map(|s| s.feature) else {
            return;
        };
        self.events.emit(
            crate::command_events::EventKind::Warn,
            feature,
            detail,
            None,
        );
    }
    pub fn submit(&mut self, text: &str) -> Result<()> {
        self.submit_effect(text).map(|_| ())
    }
    pub fn submit_effect(&mut self, text: &str) -> Result<DispatchOutcome> {
        if text.len() > 4096 {
            return Err(Error::Budget("command input"));
        }
        if let Some(kind) = crate::file_lifecycle::FileKind::from_command(text.trim()) {
            return self.dispatch_effect(kind.feature_id(), InvocationOrigin::Cmd);
        }
        self.submit_geometry_text(text)
            .map(|_| DispatchOutcome::Command)
    }
    fn submit_geometry_text(&mut self, text: &str) -> Result<()> {
        if text.len() > 4096 {
            return Err(Error::Budget("command input"));
        }
        let text = text.trim();
        if let Some(command) = parse_gumball_placement(text) {
            match command {
                GumballPlacement::Origin(origin) => self.relocate_gumball(Some(origin)),
                GumballPlacement::Reset => self.relocate_gumball(None),
                GumballPlacement::World => self.set_gumball_alignment(
                    self::gumball::GumballAlignment::World,
                ),
                GumballPlacement::Object => self.set_gumball_alignment(
                    self::gumball::GumballAlignment::Object,
                ),
                GumballPlacement::CPlane => self.set_gumball_alignment(
                    self::gumball::GumballAlignment::CPlane,
                ),
            }
            self.events.emit(
                crate::command_events::EventKind::Finished,
                "Gumball",
                "gumball placement",
                None,
            );
            return Ok(());
        }
        if let Some(edit) = self::gumball_numeric::parse(text) {
            use self::gumball_numeric::NumericEdit;
            let result = match edit {
                NumericEdit::Move(delta) => self.numeric_move(delta),
                NumericEdit::Rotate(axis, degrees) => self.numeric_rotate(axis, degrees),
                NumericEdit::Scale(axis, factor) => self.numeric_scale(axis, factor),
            };
            let moved = result?;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                "Gumball",
                &format!("applied to {moved} objects"),
                None,
            );
            return Ok(());
        }
        if let Some(count) = parse_mesh_stress(text) {
            let created = self.insert_stress_meshes(count)?;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                "MeshStress",
                &format!("created {created} meshes"),
                None,
            );
            return Ok(());
        }
        if let Some(choice) = crate::layer_command::parse(text, self.document())? {
            return self.set_current_layer(choice);
        }
        if !self.is_running() {
            if text.is_empty() {
                let repeat = self
                    .repeatable_last()
                    .map(str::to_owned)
                    .ok_or(Error::Missing("previous repeatable command"))?;
                return self.dispatch(&repeat, InvocationOrigin::Cmd);
            }
            return match text.to_ascii_lowercase().as_str() {
                "line" => self.dispatch(LINE, InvocationOrigin::Cmd),
                "point" => self.dispatch(POINT, InvocationOrigin::Cmd),
                "undo" => self.undo(),
                "redo" => self.redo(),
                _ => Err(Error::Unsupported("unknown/unavailable command")),
            };
        }
        let s = self.validate_session()?.clone();
        if text.is_empty() {
            return Err(Error::Invalid("point required"));
        }
        if let Some(parsed) = crate::option_parser::parse_option(
            self.document().page_unit,
            &Self::line_descriptor(),
            text,
        )? {
            if s.feature != LINE {
                // A command that declares no options refuses every assignment, by
                // name: POINT has none.
                return Err(Error::Unsupported("command option"));
            }
            match parsed.value {
                crate::option_parser::OptionValue::Flag(both_sides) => {
                    self.session.as_mut().expect("session").both_sides = both_sides;
                }
                crate::option_parser::OptionValue::Number { value_mm, .. } => {
                    // `Length` fixes the end point's distance from the start; the
                    // click then chooses the direction (OM9-L3-01-02-005).
                    self.session.as_mut().expect("session").length_mm = Some(value_mm);
                }
            }
            return Ok(());
        }
        // Typed local input: the grammar, prompts and error labels are unchanged, but
        // the point only becomes world through the typed frame (OM9-L3-01-08-007).
        let local = om9_core::coords::parse_local(text, s.unit)?;
        let world = om9_core::coords::resolve(
            om9_core::coords::CoordinateInput::Local(local),
            Some(&s.plane),
        )?;
        self.accept_world_point(world.xyz())
    }
    pub fn accept_world_point(&mut self, point: Point3) -> Result<()> {
        finite(point)?;
        let s = self.validate_session()?.clone();
        let geometry = if s.feature == POINT {
            Geometry::Point(point)
        } else {
            let Some(center) = s.start else {
                self.session.as_mut().expect("session").start = Some(point);
                self.events.emit(
                    crate::command_events::EventKind::Progress,
                    s.feature,
                    "first point",
                    None,
                );
                return Ok(());
            };
            let start = if s.both_sides {
                std::array::from_fn(|i| 2. * center[i] - point[i])
            } else {
                center
            };
            finite(start)?;
            // A fixed length keeps the clicked direction and scales it.
            let end = match s.length_mm {
                None => point,
                Some(fixed) => {
                    let d = [
                        point[0] - start[0],
                        point[1] - start[1],
                        point[2] - start[2],
                    ];
                    let span = d[0].hypot(d[1]).hypot(d[2]);
                    if !span.is_finite() || span <= 0. {
                        return Err(Error::Invalid("direction for the fixed length"));
                    }
                    let scale = fixed / span;
                    let end = std::array::from_fn(|i| start[i] + d[i] * scale);
                    finite(end)?;
                    end
                }
            };
            let length = (end[0] - start[0])
                .hypot(end[1] - start[1])
                .hypot(end[2] - start[2]);
            if !length.is_finite() || length <= 0. {
                return Err(Error::Invalid("nonzero finite line length"));
            }
            Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0., length],
            })
        };
        let name = if s.feature == POINT { "Point" } else { "Line" };
        let mut entity = Entity::new(name, geometry, self.document().current_layer);
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        let added = entity.id;
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let old_usage = self.scene_cache_usage()?;
        let mut held = old_usage;
        held.add(
            old_scene.update_usage(Some(&entity))?,
            self.coordinator.limits(),
        )?;
        let prepared = self.coordinator.prepare_with_held_usage(
            TransactionRequest::new(
                name,
                vec![Mutation::Insert {
                    document: self.document,
                    entity,
                }],
            ),
            held,
        )?;
        let epoch = self
            .coordinator
            .epoch()
            .checked_add(1)
            .ok_or(Error::Overflow("scene publication epoch"))?;
        let scene = old_scene.update_owned(
            prepared.candidate_world().documents[&self.document].clone(),
            epoch,
            Some(added),
            self.coordinator.limits(),
        )?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.scene = Some(scene);
            self.recent.push_back(s.feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(s.feature.into());
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        let outcome = if receipt.published {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            s.feature,
            "finished",
            Some(outcome),
        );
        self.session = None;
        Ok(())
    }
    fn refresh_scene(&mut self) -> Result<()> {
        // On a cache failure after Undo/Redo, never present the prior world as
        // current. The document action has succeeded; the display is unavailable.
        self.scene = None;
        let held = self.auxiliary_usage()?;
        let scene = Scene::capture_in_world(&self.coordinator, self.document, held)?;
        self.scene = Some(scene);
        Ok(())
    }
    pub fn undo(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        self.coordinator.undo()?;
        self.refresh_scene()?;
        self.sync_published(om9_core::changes::ChangeCause::Undo);
        Ok(())
    }
    pub fn redo(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        self.coordinator.redo()?;
        self.refresh_scene()?;
        self.sync_published(om9_core::changes::ChangeCause::Redo);
        Ok(())
    }
    pub fn preview(&self, point: Point3) -> Result<Option<(Point3, Point3)>> {
        finite(point)?;
        let Some(s) = self.session.as_ref() else {
            return Ok(None);
        };
        self.validate_session()?;
        let Some(start) = s.start else {
            return Ok(None);
        };
        let start = if s.both_sides {
            std::array::from_fn(|i| 2. * start[i] - point[i])
        } else {
            start
        };
        finite(start)?;
        Ok(Some((start, point)))
    }
    pub fn input_plane(&self) -> CPlane {
        self.session
            .as_ref()
            .map_or(self.workspace.views[&self.workspace.active].cplane, |s| {
                s.plane
            })
    }
}

/// Parses `MeshStress` / `MeshStress 2000` into a count.
fn parse_mesh_stress(text: &str) -> Option<usize> {
    let mut words = text.split_whitespace();
    let command = words.next()?;
    if !command.eq_ignore_ascii_case("MeshStress") {
        return None;
    }
    let count = match words.next() {
        Some(value) => value.parse().ok()?,
        None => self::stress::DEFAULT_STRESS_MESHES,
    };
    Some(count)
}

/// Where the gumball sits and which axes it uses.
enum GumballPlacement {
    Origin(om9_core::geometry::Point3),
    Reset,
    World,
    CPlane,
    Object,
}

fn parse_gumball_placement(text: &str) -> Option<GumballPlacement> {
    let mut words = text.split_whitespace();
    let command = words.next()?.to_ascii_lowercase();
    match command.as_str() {
        "gumballreset" => Some(GumballPlacement::Reset),
        "gumballworld" => Some(GumballPlacement::World),
        "gumballobject" => Some(GumballPlacement::Object),
        "gumballcplane" => Some(GumballPlacement::CPlane),
        "gumballorigin" => {
            let argument = words.next()?;
            let mut parts = argument.split(',');
            let x = parts.next()?.parse().ok()?;
            let y = parts.next()?.parse().ok()?;
            let z = parts.next()?.parse().ok()?;
            if parts.next().is_some() {
                return None;
            }
            Some(GumballPlacement::Origin([x, y, z]))
        }
        _ => None,
    }
}

impl Controller {
    /// Applies a transient transform to the selected objects for the duration of a drag.
    ///
    /// The document is untouched: only `SceneObject::preview` changes, which every viewport reads through
    /// `display_transform`, so all four views show the object moving. The revision lets the GPU viewports know
    /// their cached image is stale.
    pub fn preview_gumball(&mut self, delta: om9_core::geometry::Transform) -> Result<()> {
        delta.validate()?;
        let Some(scene) = self.scene.as_mut() else {
            return Err(Error::Missing("preview scene"));
        };
        let selection = &self.selection;
        for object in scene.objects.iter_mut() {
            if selection.contains(&object.root) && object.visible && object.editable {
                object.preview = delta;
            } else {
                object.preview = om9_core::geometry::Transform::identity();
            }
        }
        self.preview_revision = self.preview_revision.saturating_add(1);
        Ok(())
    }

    /// Drops the transient transform, leaving the document as it was before the drag.
    pub fn clear_gumball_preview(&mut self) {
        if let Some(scene) = self.scene.as_mut() {
            for object in scene.objects.iter_mut() {
                object.preview = om9_core::geometry::Transform::identity();
            }
        }
        self.preview_revision = self.preview_revision.saturating_add(1);
    }

    /// Revision of the transient transform, for viewport caches.
    pub fn preview_revision(&self) -> u64 {
        self.preview_revision
    }
}
