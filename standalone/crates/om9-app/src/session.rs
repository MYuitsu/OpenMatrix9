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
#[derive(Debug, Clone)]
struct CommandSession {
    feature: &'static str,
    epoch: u64,
    context: u64,
    plane: CPlane,
    unit: Unit,
    start: Option<Point3>,
    both_sides: bool,
}
pub struct Controller {
    coordinator: TransactionCoordinator,
    document: DocumentId,
    scene: Option<Scene>,
    session: Option<CommandSession>,
    recent: VecDeque<String>,
    repeat: Option<String>,
    view_menu_generation: u64,
    selection: std::collections::BTreeSet<om9_core::identity::ObjectId>,
    layer_dialog: Option<crate::layer_dialog::LayerDialog>,
    layer_dialog_serial: u64,
    file: crate::file_lifecycle::FileState,
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
            document: id,
            scene: Some(scene),
            session: None,
            recent: VecDeque::new(),
            repeat: None,
            view_menu_generation: 0,
            selection: std::collections::BTreeSet::new(),
            layer_dialog: None,
            layer_dialog_serial: 0,
            file,
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
    pub fn recent(&self) -> &VecDeque<String> {
        &self.recent
    }
    pub fn is_running(&self) -> bool {
        self.session.is_some()
    }
    pub fn availability(feature: &str) -> Result<()> {
        if crate::file_lifecycle::FileKind::from_feature(feature).is_some() {
            return Ok(());
        }
        match feature {
            LINE | POINT => Ok(()),
            _ => Err(Error::Unsupported(
                "command has no accepted standalone operation",
            )),
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
            _ => unreachable!(),
        };
        let view = &self.workspace.views[&self.workspace.active];
        self.session = Some(CommandSession {
            feature,
            epoch: self.coordinator.epoch(),
            context: self.document().context_revision,
            plane: view.cplane,
            unit: self.document().page_unit,
            start: None,
            both_sides: false,
        });
        Ok(())
    }
    pub fn prompt(&self) -> &'static str {
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
        if s.epoch != self.coordinator.epoch() || s.context != self.document().context_revision {
            return Err(Error::Stale("command document/context"));
        }
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
        self.session = None;
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
        if let Some(choice) = crate::layer_command::parse(text, self.document())? {
            return self.set_current_layer(choice);
        }
        if !self.is_running() {
            if text.is_empty() {
                let repeat = self
                    .repeat
                    .clone()
                    .ok_or(Error::Missing("previous command"))?;
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
        if let Some((key, value)) = text.split_once('=') {
            if !key.eq_ignore_ascii_case("BothSides") || s.feature != LINE {
                return Err(Error::Unsupported("command option"));
            }
            let value = match value.to_ascii_lowercase().as_str() {
                "yes" => true,
                "no" => false,
                _ => return Err(Error::Invalid("BothSides Yes/No")),
            };
            self.session.as_mut().expect("session").both_sides = value;
            return Ok(());
        }
        let coordinates: Vec<_> = text.split(',').collect();
        if !(2..=3).contains(&coordinates.len()) {
            return Err(Error::Invalid("x,y[,z] coordinate"));
        }
        let mut p = [0.; 3];
        for (i, value) in coordinates.iter().enumerate() {
            let value = value
                .trim()
                .parse()
                .map_err(|_| Error::Invalid("coordinate number"))?;
            p[i] = s.unit.length_to_mm(value)?;
        }
        self.accept_world_point(s.plane.point(p)?)
    }
    pub fn accept_world_point(&mut self, point: Point3) -> Result<()> {
        finite(point)?;
        let s = self.validate_session()?.clone();
        let geometry = if s.feature == POINT {
            Geometry::Point(point)
        } else {
            let Some(center) = s.start else {
                self.session.as_mut().expect("session").start = Some(point);
                return Ok(());
            };
            let start = if s.both_sides {
                std::array::from_fn(|i| 2. * center[i] - point[i])
            } else {
                center
            };
            finite(start)?;
            let length = (point[0] - start[0])
                .hypot(point[1] - start[1])
                .hypot(point[2] - start[2]);
            if !length.is_finite() || length <= 0. {
                return Err(Error::Invalid("nonzero finite line length"));
            }
            Geometry::Curve(Curve::Line {
                start,
                end: point,
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
        }
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
        self.refresh_scene()
    }
    pub fn redo(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        self.coordinator.redo()?;
        self.refresh_scene()
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
