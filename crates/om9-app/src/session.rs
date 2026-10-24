// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Unit, finite},
    history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
    identity::DocumentId,
    model::{Document, Entity},
    request::{Selection, SourceBinding},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use om9_scene::{
    camera::CPlane,
    scene::{RenderBuffers, Scene},
    workspace::Workspace,
};
use std::{collections::VecDeque, sync::Arc};
mod circle;
mod angle_isocurve;
mod apply_crv;
mod arc;
mod arc_blend;
mod blend;
mod divide;
mod dup_border;
mod ellipse;
mod extend_curve;
mod extrude_surface;
mod fillet_corners;
mod fit;
mod offset;
mod offset_normal;
mod offset_on_surface;
mod project;
mod pull;
mod continue_curve;
mod cross_section;
mod curve_boolean;
mod curve_soft_edit;
mod corner_ops;
mod continue_interp;
mod control_curve;
mod crv2view;
mod crv_ends;
mod crv_seam;
mod heightfield;
mod helix;
mod spiral;
mod interp;
mod interp_on_surface;
mod mesh_curve;
mod through_points;
mod uv_curves;
mod wireframe;
mod tween_curves;
mod to_lines;
mod intersect;
mod intersect_two_sets;
mod isocurve;
mod polygon;
mod polyline;
mod polyline_on_surface;
mod rebuild;
mod revolve;
mod section;
mod silhouette;
mod sketch;
mod sketch_on_surface;
mod rect;
pub mod contract;
pub mod gumball_numeric;
pub mod stress;
mod subobjects;
pub use subobjects::SelectedSubobject;
mod display_state;
mod f6_state;
mod files;
pub use files::format_import_summary;
mod gumball;
mod jobbag;
mod layer_dialogs;
mod layers;
mod pictureframe_state;
mod plugins;
mod project_state;
mod properties;
mod solid;
mod surface_edit;
mod top11;
pub use properties::{FIELDS, FieldState, PropertyEdit, PropertyField};
pub const LINE: &str = "OM9-CURVE-002";
pub const ZOOM_EXTENTS: &str = "ViewZoomZoomExtents";
pub const ZOOM_SELECTED: &str = "ViewZoomZoomSelected";
pub const VIEW_HOME: &str = "ViewHome";
pub const VIEW_CENTER: &str = "ViewCenter";
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
    /// Points already collected by the three-corner plane command.
    points: Vec<Point3>,
}

/// A selected-surface command waiting for one numeric distance.
#[derive(Debug, Clone)]
struct SurfacePending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A profile/rail command waiting for an axis origin and direction.
#[derive(Debug, Clone)]
struct CurvePairPending {
    feature: &'static str,
    profile: Curve,
    rail: Curve,
}

/// A two-surface command waiting for source/target edge names.
#[derive(Debug, Clone)]
struct SurfacePairPending {
    feature: &'static str,
    source: om9_core::geometry::NurbsSurface,
    target: om9_core::geometry::NurbsSurface,
}

/// A seam command waiting for an axis and tolerance.
#[derive(Debug, Clone)]
struct SeamPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A soft-edit command waiting for UV, radius and delta.
#[derive(Debug, Clone)]
struct SoftEditPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A tween command waiting for a fraction between two surfaces.
#[derive(Debug, Clone)]
struct TweenPending {
    feature: &'static str,
    first: om9_core::geometry::NurbsSurface,
    second: om9_core::geometry::NurbsSurface,
}

/// An extend command waiting for an edge and distance.
#[derive(Debug, Clone)]
struct ExtendPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A split command waiting for an axis and parameter.
#[derive(Debug, Clone)]
struct SplitPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A tangent command waiting for an edge and direction.
#[derive(Debug, Clone)]
struct TangentPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// An extrude-all command waiting for a height.
#[derive(Debug, Clone)]
struct ExtrudeAllPending {
    feature: &'static str,
    curves: Vec<Curve>,
}

/// A variable-offset command waiting for two distances and an axis.
#[derive(Debug, Clone)]
struct VarOffsetPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A shrink command waiting for a UV sub-domain.
#[derive(Debug, Clone)]
struct ShrinkPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A variable-blend command waiting for two tangent scales.
#[derive(Debug, Clone)]
struct VarBlendPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
}

/// A chamfer command waiting for a setback.
#[derive(Debug, Clone)]
struct ChamferPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
}

/// A fillet command waiting for one or two radii.
#[derive(Debug, Clone)]
struct FilletPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
}

/// A variable chamfer command waiting for two setbacks.
#[derive(Debug, Clone)]
struct VarChamferPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
}

/// A Sweep History command waiting for Closed/Flip/Simple options.
#[derive(Debug, Clone)]
struct HistoryOptionsPending {
    feature: &'static str,
    curves: Vec<Curve>,
    ids: Vec<om9_core::identity::ObjectId>,
    rail_count: usize,
}

/// Sweep 2 Add Slash: two alignment points on the two rails.
#[derive(Debug, Clone)]
struct SlashPending {
    feature: &'static str,
    curves: Vec<Curve>,
    ids: Vec<om9_core::identity::ObjectId>,
    rail_count: usize,
    closed: bool,
    flip: bool,
    points: Vec<Point3>,
}
pub struct Controller {
    coordinator: TransactionCoordinator,
    plugins: Vec<std::rc::Rc<om9_plugin_host::LoadedPlugin>>,
    /// The last accepted world point, for `@x,y` relative entry.
    last_point: Option<om9_core::coords::WorldPoint>,
    /// Cursor constraints applied to viewport picks, not to typed coordinates.
    ortho: bool,
    planar: bool,
    project: bool,
    /// Gumball placement: an explicit origin (relocate) and the alignment of its axes.
    gumball_origin: Option<om9_core::geometry::Point3>,
    gumball_alignment: crate::session::gumball::GumballAlignment,
    /// Pending Top 11 point command (OM9-L2-03-08).
    top11_pending: Option<self::top11::Top11Pending>,
    solid_pending: Option<solid::SolidPending>,
    circle_pending: Option<self::circle::CirclePending>,
    circle_rebuild: Option<(bool, usize, usize)>,
    rect_pending: Option<self::rect::RectPending>,
    polyline_pending: Option<self::polyline::PolylinePending>,
    interp_pending: Option<self::interp::InterpPending>,
    ellipse_pending: Option<self::ellipse::EllipsePending>,
    arc_pending: Option<self::arc::ArcPending>,
    rebuild_pending: Option<self::rebuild::RebuildPending>,
    fit_pending: Option<self::fit::FitPending>,
    offset_curve_pending: Option<self::offset::OffsetPending>,
    project_pending: Option<self::project::ProjectPending>,
    blend_pending: Option<self::blend::BlendPending>,
    isocurve_pending: Option<self::isocurve::IsoPending>,
    pull_pending: Option<self::pull::PullPending>,
    intersect_pending: Option<self::intersect::IntersectPending>,
    crv2view_pending: Option<self::crv2view::Crv2ViewPending>,
    crv_seam_pending: Option<self::crv_seam::CrvSeamPending>,
    control_curve_pending: Option<self::control_curve::ControlCurvePending>,
    offset_on_surface_pending: Option<self::offset_on_surface::OffsetOnSurfacePending>,
    offset_normal_pending: Option<self::offset_normal::OffsetNormalPending>,
    corner_pending: Option<self::corner_ops::CornerPending>,
    cross_section_pending: Option<self::cross_section::CrossSectionPending>,
    fillet_corners_pending: Option<self::fillet_corners::FilletCornersPending>,
    continue_curve_pending: Option<self::continue_curve::ContinueCurvePending>,
    through_points_pending: Option<self::through_points::ThroughPointsPending>,
    polygon_pending: Option<self::polygon::PolygonPending>,
    helix_pending: Option<self::helix::HelixPending>,
    spiral_pending: Option<self::spiral::SpiralPending>,
    divide_pending: Option<self::divide::DividePending>,
    to_lines_pending: Option<self::to_lines::ToLinesPending>,
    extend_curve_pending: Option<self::extend_curve::ExtendPending>,
    tween_curves_pending: Option<self::tween_curves::TweenCurvesPending>,
    continue_interp_pending: Option<self::continue_interp::ContinueInterpPending>,
    arc_blend_pending: Option<self::arc_blend::ArcBlendPending>,
    /// Pending numeric distance for Offset/Extrude Normal.
    surface_pending: Option<SurfacePending>,
    curve_pair_pending: Option<CurvePairPending>,
    surface_pair_pending: Option<SurfacePairPending>,
    seam_pending: Option<SeamPending>,
    soft_edit_pending: Option<SoftEditPending>,
    curve_soft_edit_pending: Option<self::curve_soft_edit::CurveSoftEditPending>,
    angle_isocurve_pending: Option<self::angle_isocurve::AngleIsocurvePending>,
    extrude_surface_pending: Option<self::extrude_surface::ExtrudePending>,
    heightfield_pending: Option<self::heightfield::HeightfieldPending>,
    tween_pending: Option<TweenPending>,
    extend_pending: Option<ExtendPending>,
    split_pending: Option<SplitPending>,
    tangent_pending: Option<TangentPending>,
    extrude_all_pending: Option<ExtrudeAllPending>,
    variable_offset_pending: Option<VarOffsetPending>,
    shrink_pending: Option<ShrinkPending>,
    variable_blend_pending: Option<VarBlendPending>,
    chamfer_pending: Option<ChamferPending>,
    fillet_pending: Option<FilletPending>,
    variable_chamfer_pending: Option<VarChamferPending>,
    history_options_pending: Option<HistoryOptionsPending>,
    slash_pending: Option<SlashPending>,
    /// Active edit/control point display for the selected curves.
    top11_grip: Option<om9_core::top11::GripMode>,
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
    /// Selection in pick order, filtered by the live set. Commands that combine
    /// multiple curves/surfaces use this order; stale entries are ignored.
    selection_order: Vec<om9_core::identity::ObjectId>,
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
    /// The batch recorded for the last plugin command; it is the feed a following
    /// plugin command reads, independent of the observer baseline.
    plugin_change: Option<om9_core::changes::ChangeBatch>,
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
            )
            | crate::viewport_menu::MenuAction::Preset(
                crate::viewport_menu::DisplayPreset::Shaded,
            ) => {
                let mode = match action {
                    crate::viewport_menu::MenuAction::Preset(
                        crate::viewport_menu::DisplayPreset::Shaded,
                    ) => om9_scene::workspace::DisplayMode::Shaded,
                    _ => om9_scene::workspace::DisplayMode::Wireframe,
                };
                let view = self
                    .workspace
                    .views
                    .get_mut(&capture.target())
                    .ok_or(Error::Missing("viewport menu target"))?;
                if view.mode != mode {
                    let revision = view
                        .revision
                        .checked_add(1)
                        .ok_or(Error::Overflow("view revision"))?;
                    view.mode = mode;
                    view.revision = revision;
                }
                Ok(())
            }
            crate::viewport_menu::MenuAction::SetCPlane(preset) => {
                let view = self
                    .workspace
                    .views
                    .get_mut(&capture.target())
                    .ok_or(Error::Missing("viewport menu target"))?;
                let revision = view
                    .cplane
                    .revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("CPlane revision"))?;
                view.cplane = CPlane {
                    revision,
                    ..CPlane::preset(preset)
                };
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
            plugins: Vec::new(),
            last_point: None,
            ortho: false,
            planar: false,
            project: false,
            gumball_origin: None,
            gumball_alignment: crate::session::gumball::GumballAlignment::World,
            top11_pending: None,
            solid_pending: None,
            circle_pending: None,
            circle_rebuild: None,
            rect_pending: None,
            polyline_pending: None,
            interp_pending: None,
            ellipse_pending: None,
            arc_pending: None,
            rebuild_pending: None,
            fit_pending: None,
            offset_curve_pending: None,
            project_pending: None,
            blend_pending: None,
            isocurve_pending: None,
            pull_pending: None,
            intersect_pending: None,
            crv2view_pending: None,
            crv_seam_pending: None,
            control_curve_pending: None,
            offset_on_surface_pending: None,
            offset_normal_pending: None,
            corner_pending: None,
            cross_section_pending: None,
            fillet_corners_pending: None,
            continue_curve_pending: None,
            through_points_pending: None,
            polygon_pending: None,
            helix_pending: None,
            spiral_pending: None,
            divide_pending: None,
            to_lines_pending: None,
            extend_curve_pending: None,
            tween_curves_pending: None,
            continue_interp_pending: None,
            arc_blend_pending: None,
            surface_pending: None,
            curve_pair_pending: None,
            surface_pair_pending: None,
            seam_pending: None,
            soft_edit_pending: None,
            curve_soft_edit_pending: None,
            angle_isocurve_pending: None,
            extrude_surface_pending: None,
            heightfield_pending: None,
            tween_pending: None,
            extend_pending: None,
            split_pending: None,
            tangent_pending: None,
            extrude_all_pending: None,
            variable_offset_pending: None,
            shrink_pending: None,
            variable_blend_pending: None,
            chamfer_pending: None,
            fillet_pending: None,
            variable_chamfer_pending: None,
            history_options_pending: None,
            slash_pending: None,
            top11_grip: None,
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
            selection_order: Vec::new(),
            selection_revision: 1,
            layer_dialog: None,
            layer_dialog_serial: 0,
            file,
            events: crate::command_events::EventLog::new(),
            observers: Vec::new(),
            observer_diagnostics: Vec::new(),
            changes: VecDeque::new(),
            plugin_change: None,
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
    /// Resolves a typed command name through the declared aliases of the implemented commands.
    /// Registry aliases such as `Ln` are part of the record, so typing them must reach the same command.
    pub fn resolve_alias(text: &str) -> Option<&'static str> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("undo") {
            return Some("undo");
        }
        if text.eq_ignore_ascii_case("redo") {
            return Some("redo");
        }
        if let Some(command) = Self::view_command(text) {
            return Some(command);
        }
        if text.eq_ignore_ascii_case("circle") || text.eq_ignore_ascii_case(self::circle::CIRCLE) {
            return Some(self::circle::CIRCLE);
        }
        if text.eq_ignore_ascii_case("rectangle")
            || text.eq_ignore_ascii_case(self::rect::RECTANGLE)
        {
            return Some(self::rect::RECTANGLE);
        }
        if text.eq_ignore_ascii_case("polyline")
            || text.eq_ignore_ascii_case(self::polyline::POLYLINE)
        {
            return Some(self::polyline::POLYLINE);
        }
        if text.eq_ignore_ascii_case("interpcrv")
            || text.eq_ignore_ascii_case(self::interp::INTERP)
        {
            return Some(self::interp::INTERP);
        }
        if text.eq_ignore_ascii_case("ellipse")
            || text.eq_ignore_ascii_case(self::ellipse::ELLIPSE)
        {
            return Some(self::ellipse::ELLIPSE);
        }
        if text.eq_ignore_ascii_case("arc") || text.eq_ignore_ascii_case(self::arc::ARC) {
            return Some(self::arc::ARC);
        }
        if text.eq_ignore_ascii_case("arc_startpoint")
            || text.eq_ignore_ascii_case(self::arc::ARC_START)
        {
            return Some(self::arc::ARC_START);
        }
        if text.eq_ignore_ascii_case("rebuild")
            || text.eq_ignore_ascii_case(self::rebuild::REBUILD)
        {
            return Some(self::rebuild::REBUILD);
        }
        if text.eq_ignore_ascii_case("fitcrv") || text.eq_ignore_ascii_case(self::fit::FIT) {
            return Some(self::fit::FIT);
        }
        if text.eq_ignore_ascii_case("offset") || text.eq_ignore_ascii_case(self::offset::OFFSET) {
            return Some(self::offset::OFFSET);
        }
        if text.eq_ignore_ascii_case("project")
            || text.eq_ignore_ascii_case(self::project::PROJECT)
        {
            return Some(self::project::PROJECT);
        }
        if text.eq_ignore_ascii_case("blend") || text.eq_ignore_ascii_case(self::blend::BLEND) {
            return Some(self::blend::BLEND);
        }
        if text.eq_ignore_ascii_case("blendcrv")
            || text.eq_ignore_ascii_case(self::blend::BLEND_CRV)
        {
            return Some(self::blend::BLEND_CRV);
        }
        if text.eq_ignore_ascii_case("match") || text.eq_ignore_ascii_case(self::blend::MATCH) {
            return Some(self::blend::MATCH);
        }
        if text.eq_ignore_ascii_case("extractisocurve")
            || text.eq_ignore_ascii_case(self::isocurve::ISOCURVE)
        {
            return Some(self::isocurve::ISOCURVE);
        }
        if text.eq_ignore_ascii_case("pull") || text.eq_ignore_ascii_case(self::pull::PULL) {
            return Some(self::pull::PULL);
        }
        if text.eq_ignore_ascii_case("intersect")
            || text.eq_ignore_ascii_case(self::intersect::INTERSECT)
        {
            return Some(self::intersect::INTERSECT);
        }
        if text.eq_ignore_ascii_case("intersecttwosets")
            || text.eq_ignore_ascii_case(self::intersect_two_sets::INTERSECT_TWO_SETS)
        {
            return Some(self::intersect_two_sets::INTERSECT_TWO_SETS);
        }
        if text.eq_ignore_ascii_case("crv2view")
            || text.eq_ignore_ascii_case(self::crv2view::CRV2VIEW)
        {
            return Some(self::crv2view::CRV2VIEW);
        }
        if text.eq_ignore_ascii_case("curvethroughpt")
            || text.eq_ignore_ascii_case(self::through_points::THROUGH_POINTS)
        {
            return Some(self::through_points::THROUGH_POINTS);
        }
        if text.eq_ignore_ascii_case("polygon")
            || text.eq_ignore_ascii_case(self::polygon::POLYGON)
        {
            return Some(self::polygon::POLYGON);
        }
        if text.eq_ignore_ascii_case("helix") || text.eq_ignore_ascii_case(self::helix::HELIX) {
            return Some(self::helix::HELIX);
        }
        if text.eq_ignore_ascii_case("spiral") || text.eq_ignore_ascii_case(self::spiral::SPIRAL) {
            return Some(self::spiral::SPIRAL);
        }
        if text.eq_ignore_ascii_case("divide") || text.eq_ignore_ascii_case(self::divide::DIVIDE) {
            return Some(self::divide::DIVIDE);
        }
        if text.eq_ignore_ascii_case("convert") || text.eq_ignore_ascii_case(self::to_lines::TO_LINES)
        {
            return Some(self::to_lines::TO_LINES);
        }
        if text.eq_ignore_ascii_case("extend")
            || text.eq_ignore_ascii_case(self::extend_curve::EXTEND)
        {
            return Some(self::extend_curve::EXTEND);
        }
        if text.eq_ignore_ascii_case("tweencurves")
            || text.eq_ignore_ascii_case(self::tween_curves::TWEEN_CURVES)
        {
            return Some(self::tween_curves::TWEEN_CURVES);
        }
        if text.eq_ignore_ascii_case("continueinterpcrv")
            || text.eq_ignore_ascii_case(self::continue_interp::CONTINUE_INTERP)
        {
            return Some(self::continue_interp::CONTINUE_INTERP);
        }
        if text.eq_ignore_ascii_case("arcblend")
            || text.eq_ignore_ascii_case(self::arc_blend::ARC_BLEND)
        {
            return Some(self::arc_blend::ARC_BLEND);
        }
        if text.eq_ignore_ascii_case("dupedge")
            || text.eq_ignore_ascii_case(self::wireframe::DUP_EDGE)
        {
            return Some(self::wireframe::DUP_EDGE);
        }
        if text.eq_ignore_ascii_case("extractwire")
            || text.eq_ignore_ascii_case(self::wireframe::EXTRACT_WIRE)
        {
            return Some(self::wireframe::EXTRACT_WIRE);
        }
        if text.eq_ignore_ascii_case("crvstart")
            || text.eq_ignore_ascii_case(self::crv_ends::CRV_START)
        {
            return Some(self::crv_ends::CRV_START);
        }
        if text.eq_ignore_ascii_case("crvend") || text.eq_ignore_ascii_case(self::crv_ends::CRV_END)
        {
            return Some(self::crv_ends::CRV_END);
        }
        if text.eq_ignore_ascii_case("interpcrvonsrf")
            || text.eq_ignore_ascii_case(self::interp_on_surface::INTERP_ON_SURFACE)
        {
            return Some(self::interp_on_surface::INTERP_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("section") || text.eq_ignore_ascii_case(self::section::SECTION) {
            return Some(self::section::SECTION);
        }
        if text.eq_ignore_ascii_case("revolve") || text.eq_ignore_ascii_case(self::revolve::REVOLVE)
        {
            return Some(self::revolve::REVOLVE);
        }
        if text.eq_ignore_ascii_case("curveboolean")
            || text.eq_ignore_ascii_case(self::curve_boolean::CURVE_BOOLEAN)
        {
            return Some(self::curve_boolean::CURVE_BOOLEAN);
        }
        if text.eq_ignore_ascii_case("loft") {
            return Some(self::surface_edit::LOFT);
        }
        if text.eq_ignore_ascii_case("sweep1") {
            return Some(self::surface_edit::SWEEP1);
        }
        if text.eq_ignore_ascii_case("sweep2") {
            return Some(self::surface_edit::SWEEP2);
        }
        if text.eq_ignore_ascii_case("networksrf") {
            return Some(self::surface_edit::NETWORK);
        }
        if text.eq_ignore_ascii_case("patch") {
            return Some(self::surface_edit::PATCH);
        }
        if text.eq_ignore_ascii_case("blendsrf") {
            return Some(self::surface_edit::BLEND);
        }
        if text.eq_ignore_ascii_case("seamsrf") || text.eq_ignore_ascii_case("seam") {
            return Some(self::surface_edit::SEAM);
        }
        if text.eq_ignore_ascii_case("mergesrf") {
            return Some(self::surface_edit::MERGE);
        }
        if text.eq_ignore_ascii_case("matchsrf") {
            return Some(self::surface_edit::MATCH);
        }
        if text.eq_ignore_ascii_case("tweensurfaces") || text.eq_ignore_ascii_case("tween") {
            return Some(self::surface_edit::TWEEN);
        }
        if text.eq_ignore_ascii_case("railrevolve") {
            return Some(self::surface_edit::RAIL_REVOLVE);
        }
        if text.eq_ignore_ascii_case("extendsrf") {
            return Some(self::surface_edit::EXTEND);
        }
        if text.eq_ignore_ascii_case("shrinksrf") {
            return Some(self::surface_edit::SHRINK);
        }
        if text.eq_ignore_ascii_case("unrollsrf") {
            return Some(self::surface_edit::UNROLL);
        }
        if text.eq_ignore_ascii_case("offsetsrf") {
            return Some(self::surface_edit::OFFSET);
        }
        if text.eq_ignore_ascii_case("planarsrf") {
            return Some(self::surface_edit::PLANAR_CURVES);
        }
        if text.eq_ignore_ascii_case("edgesrf") {
            return Some(self::surface_edit::EDGE_CURVES);
        }
        if text.eq_ignore_ascii_case("heightfield")
            || text.eq_ignore_ascii_case(self::heightfield::HEIGHTFIELD)
        {
            return Some(self::heightfield::HEIGHTFIELD);
        }
        if text.eq_ignore_ascii_case("extrudesurface")
            || text.eq_ignore_ascii_case(self::extrude_surface::EXTRUDE)
        {
            return Some(self::extrude_surface::EXTRUDE);
        }
        if text.eq_ignore_ascii_case("angleisocurve")
            || text.eq_ignore_ascii_case(self::angle_isocurve::ANGLE_ISOCURVE)
        {
            return Some(self::angle_isocurve::ANGLE_ISOCURVE);
        }
        if text.eq_ignore_ascii_case("curvesoftedit")
            || text.eq_ignore_ascii_case(self::curve_soft_edit::CURVE_SOFT_EDIT)
        {
            return Some(self::curve_soft_edit::CURVE_SOFT_EDIT);
        }
        if text.eq_ignore_ascii_case("crosssectionprofiles")
            || text.eq_ignore_ascii_case(self::cross_section::CROSS_SECTION)
        {
            return Some(self::cross_section::CROSS_SECTION);
        }
        if text.eq_ignore_ascii_case("filletcorners")
            || text.eq_ignore_ascii_case(self::fillet_corners::FILLET_CORNERS)
        {
            return Some(self::fillet_corners::FILLET_CORNERS);
        }
        if text.eq_ignore_ascii_case("silhouette")
            || text.eq_ignore_ascii_case(self::silhouette::SILHOUETTE)
        {
            return Some(self::silhouette::SILHOUETTE);
        }
        if text.eq_ignore_ascii_case("mesh") || text.eq_ignore_ascii_case(self::mesh_curve::MESH_CURVE)
        {
            return Some(self::mesh_curve::MESH_CURVE);
        }
        if text.eq_ignore_ascii_case("createuvcrv")
            || text.eq_ignore_ascii_case(self::uv_curves::CREATE_UV)
        {
            return Some(self::uv_curves::CREATE_UV);
        }
        if text.eq_ignore_ascii_case("applycrv")
            || text.eq_ignore_ascii_case(self::apply_crv::APPLY_CRV)
        {
            return Some(self::apply_crv::APPLY_CRV);
        }
        if text.eq_ignore_ascii_case("offsetcrvonsrf")
            || text.eq_ignore_ascii_case(self::offset_on_surface::OFFSET_ON_SURFACE)
        {
            return Some(self::offset_on_surface::OFFSET_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("offset normal")
            || text.eq_ignore_ascii_case("offsetnormal")
            || text.eq_ignore_ascii_case(self::offset_normal::OFFSET_NORMAL)
        {
            return Some(self::offset_normal::OFFSET_NORMAL);
        }
        if text.eq_ignore_ascii_case("curvefillet")
            || text.eq_ignore_ascii_case(self::corner_ops::FILLET)
        {
            return Some(self::corner_ops::FILLET);
        }
        if text.eq_ignore_ascii_case("curvechamfer")
            || text.eq_ignore_ascii_case(self::corner_ops::CHAMFER)
        {
            return Some(self::corner_ops::CHAMFER);
        }
        if text.eq_ignore_ascii_case("crvseam")
            || text.eq_ignore_ascii_case(self::crv_seam::CRV_SEAM)
        {
            return Some(self::crv_seam::CRV_SEAM);
        }
        if text.eq_ignore_ascii_case("curve")
            || text.eq_ignore_ascii_case(self::control_curve::CONTROL_CURVE)
        {
            return Some(self::control_curve::CONTROL_CURVE);
        }
        if text.eq_ignore_ascii_case("continuecurve")
            || text.eq_ignore_ascii_case(self::continue_curve::CONTINUE_CURVE)
        {
            return Some(self::continue_curve::CONTINUE_CURVE);
        }
        if text.eq_ignore_ascii_case("polylineonsrf")
            || text.eq_ignore_ascii_case(self::polyline_on_surface::POLYLINE_ON_SURFACE)
        {
            return Some(self::polyline_on_surface::POLYLINE_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("sketch") || text.eq_ignore_ascii_case(self::sketch::SKETCH) {
            return Some(self::sketch::SKETCH);
        }
        if text.eq_ignore_ascii_case("sketchonsrf")
            || text.eq_ignore_ascii_case(self::sketch_on_surface::SKETCH_ON_SURFACE)
        {
            return Some(self::sketch_on_surface::SKETCH_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("dupborder")
            || text.eq_ignore_ascii_case(self::dup_border::DUP_BORDER)
        {
            return Some(self::dup_border::DUP_BORDER);
        }
        for descriptor in [Self::line_descriptor(), Self::point_descriptor()] {
            if text.eq_ignore_ascii_case(&descriptor.feature)
                || descriptor
                    .aliases
                    .iter()
                    .any(|alias| text.eq_ignore_ascii_case(alias))
            {
                return Some(if descriptor.feature == LINE {
                    LINE
                } else {
                    POINT
                });
            }
        }
        None
    }

    /// Canonical immediate view commands accepted by the command line.
    pub fn view_command(text: &str) -> Option<&'static str> {
        let text = text.trim();
        let aliases: &[&str] = match text {
            t if t.eq_ignore_ascii_case("ZoomExtents")
                || t.eq_ignore_ascii_case("ZE")
                || t.eq_ignore_ascii_case("ViewZoomZoomExtents") =>
            {
                &["ViewZoomZoomExtents"]
            }
            t if t.eq_ignore_ascii_case("ZoomSelected")
                || t.eq_ignore_ascii_case("ZS")
                || t.eq_ignore_ascii_case("ViewZoomZoomSelected") =>
            {
                &["ViewZoomZoomSelected"]
            }
            t if t.eq_ignore_ascii_case("Home") || t.eq_ignore_ascii_case("ViewHome") => {
                &["ViewHome"]
            }
            t if t.eq_ignore_ascii_case("Center") || t.eq_ignore_ascii_case("ViewCenter") => {
                &["ViewCenter"]
            }
            _ => return None,
        };
        aliases.first().copied()
    }

    /// Whether the running line command draws both sides of its start point.
    pub fn both_sides(&self) -> Option<bool> {
        self.session.as_ref().map(|session| session.both_sides)
    }

    /// Sets the BothSides option through the same parser path a typed assignment uses.
    pub fn set_both_sides(&mut self, value: bool) -> Result<()> {
        if self.phase() == CmdPhase::Idle {
            return Err(Error::Missing("active command"));
        }
        self.submit_geometry_text(&format!("BothSides={}", if value { "Yes" } else { "No" }))
    }

    /// The canonical feature id the command line should echo: the running command when one is active,
    /// otherwise the repeatable command an empty Enter would run.
    pub fn active_feature(&self) -> Option<&str> {
        self.session
            .as_ref()
            .map(|session| session.feature)
            .or_else(|| self.repeatable_last())
    }
    pub fn repeat_slot(&self) -> Option<&str> {
        self.repeat.as_deref()
    }
    pub fn recent(&self) -> &VecDeque<String> {
        &self.recent
    }
    pub fn is_running(&self) -> bool {
        if self.solid_pending.is_some()
            || self.circle_pending.is_some()
            || self.rect_pending.is_some()
            || self.polyline_pending.is_some()
            || self.interp_pending.is_some()
            || self.ellipse_pending.is_some()
            || self.arc_pending.is_some()
            || self.rebuild_pending.is_some()
            || self.fit_pending.is_some()
            || self.offset_curve_pending.is_some()
            || self.project_pending.is_some()
            || self.blend_pending.is_some()
            || self.isocurve_pending.is_some()
            || self.intersect_pending.is_some()
            || self.crv2view_pending.is_some()
            || self.through_points_pending.is_some()
            || self.polygon_pending.is_some()
            || self.helix_pending.is_some()
            || self.spiral_pending.is_some()
            || self.divide_pending.is_some()
            || self.to_lines_pending.is_some()
            || self.extend_curve_pending.is_some()
            || self.tween_curves_pending.is_some()
            || self.continue_interp_pending.is_some()
            || self.arc_blend_pending.is_some()
        {
            return true;
        }
        if self.top11_pending.is_some()
            || self.surface_pending.is_some()
            || self.curve_pair_pending.is_some()
            || self.surface_pair_pending.is_some()
            || self.seam_pending.is_some()
            || self.soft_edit_pending.is_some()
            || self.tween_pending.is_some()
            || self.extend_pending.is_some()
            || self.split_pending.is_some()
            || self.tangent_pending.is_some()
            || self.extrude_all_pending.is_some()
            || self.variable_offset_pending.is_some()
            || self.shrink_pending.is_some()
            || self.variable_blend_pending.is_some()
            || self.chamfer_pending.is_some()
            || self.fillet_pending.is_some()
            || self.variable_chamfer_pending.is_some()
            || self.history_options_pending.is_some()
            || self.slash_pending.is_some()
        {
            return true;
        }
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
        if solid::mode_command(feature).is_some() {
            return Ok(());
        }
        if solid::handles(feature) {
            return crate::solid_menu::by_feature(feature)
                .filter(|binding| binding.enabled())
                .map(|_| ())
                .ok_or(Error::Unsupported(
                    crate::solid_menu::by_feature(feature)
                        .map_or("solid command is not catalogued", |binding| binding.reason),
                ));
        }
        if surface_edit::handles(feature)
            || surface_edit::handles_numeric(feature)
            || surface_edit::handles_curves(feature)
            || surface_edit::handles_history(feature)
            || surface_edit::handles_sweep4(feature)
            || surface_edit::handles_four_rail_profile(feature)
            || surface_edit::handles_surface_pair(feature)
            || surface_edit::handles_seam(feature)
            || surface_edit::handles_soft_edit(feature)
            || surface_edit::handles_tween(feature)
            || surface_edit::handles_extend(feature)
            || surface_edit::handles_split(feature)
            || surface_edit::handles_drape(feature)
            || surface_edit::handles_tangent(feature)
            || surface_edit::handles_extrude_all(feature)
            || surface_edit::handles_planar(feature)
            || surface_edit::handles_variable_offset(feature)
            || surface_edit::handles_shrink(feature)
            || surface_edit::handles_variable_blend(feature)
            || surface_edit::handles_untrim(feature)
            || surface_edit::handles_chamfer(feature)
            || surface_edit::handles_fillet(feature)
            || surface_edit::handles_variable_chamfer(feature)
            || surface_edit::handles_sweep_multi(feature)
            || surface_edit::handles_merge(feature)
            || surface_edit::plane_like(feature)
        {
            return Ok(());
        }
        match feature {
            LINE | POINT | self::circle::CIRCLE | self::rect::RECTANGLE
            | self::polyline::POLYLINE | self::interp::INTERP | self::ellipse::ELLIPSE
            | self::arc::ARC | self::arc::ARC_START | self::rebuild::REBUILD
            | self::fit::FIT | self::offset::OFFSET | self::project::PROJECT
            | self::blend::BLEND | self::blend::BLEND_CRV | self::blend::MATCH
            | self::isocurve::ISOCURVE
            | self::pull::PULL
            | self::interp_on_surface::INTERP_ON_SURFACE
            | self::polyline_on_surface::POLYLINE_ON_SURFACE
            | self::sketch_on_surface::SKETCH_ON_SURFACE
            | self::intersect::INTERSECT | self::intersect_two_sets::INTERSECT_TWO_SETS
            | self::crv2view::CRV2VIEW | self::through_points::THROUGH_POINTS
            | self::polygon::POLYGON | self::helix::HELIX | self::spiral::SPIRAL
            | self::divide::DIVIDE | self::crv_ends::CRV_START | self::crv_ends::CRV_END
            | self::dup_border::DUP_BORDER | self::to_lines::TO_LINES
            | self::extend_curve::EXTEND | self::tween_curves::TWEEN_CURVES
            | self::continue_interp::CONTINUE_INTERP | self::arc_blend::ARC_BLEND
            | self::wireframe::DUP_EDGE | self::wireframe::EXTRACT_WIRE
            | self::section::SECTION | self::sketch::SKETCH | self::revolve::REVOLVE
            | self::curve_boolean::CURVE_BOOLEAN | self::cross_section::CROSS_SECTION
            | self::fillet_corners::FILLET_CORNERS | self::silhouette::SILHOUETTE
            | self::curve_soft_edit::CURVE_SOFT_EDIT | self::angle_isocurve::ANGLE_ISOCURVE
            | self::extrude_surface::EXTRUDE | self::heightfield::HEIGHTFIELD
            | self::mesh_curve::MESH_CURVE | self::uv_curves::CREATE_UV | self::apply_crv::APPLY_CRV
            | self::offset_on_surface::OFFSET_ON_SURFACE | self::offset_normal::OFFSET_NORMAL
            | self::corner_ops::FILLET | self::corner_ops::CHAMFER
            | self::crv_seam::CRV_SEAM
            | self::control_curve::CONTROL_CURVE | self::continue_curve::CONTINUE_CURVE
            | "MeshStress" | ZOOM_EXTENTS | ZOOM_SELECTED
            | VIEW_HOME | VIEW_CENTER => Ok(()),
            _ => Err(Error::Unsupported(
                "command has no accepted standalone operation",
            )),
        }
    }
    /// The descriptor of the geometry commands this session runs: the option names
    /// the parser checks against (OM9-L3-01-02-005). The registry (OM9-L3-01-02-001)
    /// will own the catalogue; the session reads the same shape meanwhile.
    pub fn point_descriptor() -> om9_core::capability::Operation {
        om9_core::capability::Operation {
            id: "point".to_owned(),
            feature: POINT.to_owned(),
            aliases: vec!["Point".into(), "Pt".into()],
            prompt: "Location of point: ".into(),
            input_types: vec![om9_core::geometry::GeometryKind::Point],
            requires_input: true,
            output: vec![om9_core::geometry::GeometryKind::Point],
            options: Vec::new(),
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
        if let Some((target, mode)) = solid::mode_command(feature) {
            self.dispatch_solid(target)?;
            if let Some(result) = self.submit_solid_text(&format!("Mode={mode}")) {
                result?;
            }
            return Ok(DispatchOutcome::Command);
        }
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
            ZOOM_EXTENTS => {
                self.zoom_extents(1.6)?;
                return Ok(());
            }
            ZOOM_SELECTED => {
                self.zoom_selected(1.6)?;
                return Ok(());
            }
            VIEW_HOME => {
                self.view_home()?;
                return Ok(());
            }
            VIEW_CENTER => {
                self.view_center()?;
                return Ok(());
            }
            LINE => LINE,
            POINT => POINT,
            "Circle" | self::circle::CIRCLE => {
                return self.dispatch_circle();
            }
            "Rectangle" | self::rect::RECTANGLE => {
                return self.dispatch_rect();
            }
            "Polyline" | self::polyline::POLYLINE => {
                return self.dispatch_polyline();
            }
            "InterpCrv" | self::interp::INTERP => {
                return self.dispatch_interp();
            }
            "Ellipse" | self::ellipse::ELLIPSE => {
                return self.dispatch_ellipse();
            }
            "Arc" | self::arc::ARC => {
                return self.dispatch_arc();
            }
            "Arc_StartPoint" | self::arc::ARC_START => {
                return self.dispatch_arc_start();
            }
            "Rebuild" | self::rebuild::REBUILD => {
                return self.dispatch_rebuild();
            }
            "FitCrv" | self::fit::FIT => {
                return self.dispatch_fit();
            }
            "Offset" | self::offset::OFFSET => {
                return self.dispatch_offset_curve();
            }
            "Project" | self::project::PROJECT => {
                return self.dispatch_project();
            }
            "Blend" | self::blend::BLEND => {
                return self.dispatch_blend();
            }
            "BlendCrv" | self::blend::BLEND_CRV => {
                return self.dispatch_blend();
            }
            "Match" | self::blend::MATCH => {
                return self.dispatch_match();
            }
            "ExtractIsocurve" | self::isocurve::ISOCURVE => {
                return self.dispatch_isocurve();
            }
            "Pull" | self::pull::PULL => {
                return self.dispatch_pull();
            }
            "Intersect" | self::intersect::INTERSECT => {
                return self.dispatch_intersect();
            }
            "IntersectTwoSets" | self::intersect_two_sets::INTERSECT_TWO_SETS => {
                return self.dispatch_intersect_two_sets();
            }
            "Crv2View" | self::crv2view::CRV2VIEW => {
                return self.dispatch_crv2view();
            }
            "CurveThroughPt" | self::through_points::THROUGH_POINTS => {
                return self.dispatch_through_points();
            }
            "Polygon" | self::polygon::POLYGON => {
                return self.dispatch_polygon();
            }
            "Helix" | self::helix::HELIX => {
                return self.dispatch_helix();
            }
            "Spiral" | self::spiral::SPIRAL => {
                return self.dispatch_spiral();
            }
            "Divide" | self::divide::DIVIDE => {
                return self.dispatch_divide();
            }
            "CrvStart" | self::crv_ends::CRV_START => {
                return self.dispatch_crv_mark(false);
            }
            "CrvEnd" | self::crv_ends::CRV_END => {
                return self.dispatch_crv_mark(true);
            }
            "DupBorder" | self::dup_border::DUP_BORDER => {
                return self.dispatch_dup_border();
            }
            "Convert" | self::to_lines::TO_LINES => {
                return self.dispatch_to_lines();
            }
            "Extend" | self::extend_curve::EXTEND => {
                return self.dispatch_extend_curve();
            }
            "TweenCurves" | self::tween_curves::TWEEN_CURVES => {
                return self.dispatch_tween_curves();
            }
            "ContinueinterpCrv" | self::continue_interp::CONTINUE_INTERP => {
                return self.dispatch_continue_interp();
            }
            "ArcBlend" | self::arc_blend::ARC_BLEND => {
                return self.dispatch_arc_blend();
            }
            "InterpCrvOnSrf" | self::interp_on_surface::INTERP_ON_SURFACE => {
                return self.dispatch_interp_on_surface();
            }
            "Section" | self::section::SECTION => {
                return self.dispatch_section();
            }
            "Revolve" | self::revolve::REVOLVE => {
                return self.dispatch_revolve();
            }
            "CurveBoolean" | self::curve_boolean::CURVE_BOOLEAN => {
                return self.dispatch_curve_boolean();
            }
            "Heightfield" | self::heightfield::HEIGHTFIELD => {
                return self.dispatch_heightfield();
            }
            "ExtrudeSurface" | self::extrude_surface::EXTRUDE => {
                return self.dispatch_extrude_surface();
            }
            "AngleIsocurve" | self::angle_isocurve::ANGLE_ISOCURVE => {
                return self.dispatch_angle_isocurve();
            }
            "CurveSoftEdit" | self::curve_soft_edit::CURVE_SOFT_EDIT => {
                return self.dispatch_curve_soft_edit();
            }
            "CrossSectionProfiles" | self::cross_section::CROSS_SECTION => {
                return self.dispatch_cross_section();
            }
            "FilletCorners" | self::fillet_corners::FILLET_CORNERS => {
                return self.dispatch_fillet_corners();
            }
            "Silhouette" | self::silhouette::SILHOUETTE => {
                return self.dispatch_silhouette();
            }
            "Mesh" | self::mesh_curve::MESH_CURVE => {
                return self.dispatch_mesh_curve();
            }
            "CreateUVCrv" | self::uv_curves::CREATE_UV => {
                return self.dispatch_create_uv();
            }
            "ApplyCrv" | self::apply_crv::APPLY_CRV => {
                return self.dispatch_apply_crv();
            }
            "OffsetCrvOnSrf" | self::offset_on_surface::OFFSET_ON_SURFACE => {
                return self.dispatch_offset_on_surface();
            }
            "Offset Normal" | self::offset_normal::OFFSET_NORMAL => {
                return self.dispatch_offset_normal();
            }
            "CurveFillet" | self::corner_ops::FILLET => {
                return self.dispatch_corner(self::corner_ops::FILLET);
            }
            "CurveChamfer" | self::corner_ops::CHAMFER => {
                return self.dispatch_corner(self::corner_ops::CHAMFER);
            }
            "CrvSeam" | self::crv_seam::CRV_SEAM => {
                return self.dispatch_crv_seam();
            }
            "Curve" | self::control_curve::CONTROL_CURVE => {
                return self.dispatch_control_curve();
            }
            "ContinueCurve" | self::continue_curve::CONTINUE_CURVE => {
                return self.dispatch_continue_curve();
            }
            "PolylineOnSrf" | self::polyline_on_surface::POLYLINE_ON_SURFACE => {
                return self.dispatch_polyline_on_surface();
            }
            "Sketch" | self::sketch::SKETCH => {
                return self.dispatch_sketch();
            }
            "SketchOnSrf" | self::sketch_on_surface::SKETCH_ON_SURFACE => {
                return self.dispatch_sketch_on_surface();
            }
            "DupEdge" | self::wireframe::DUP_EDGE => {
                return self.dispatch_wireframe(self::wireframe::DUP_EDGE);
            }
            "ExtractWire" | self::wireframe::EXTRACT_WIRE => {
                return self.dispatch_wireframe(self::wireframe::EXTRACT_WIRE);
            }
            "MeshStress" => {
                // Menu or search invocation: the same command as typing `MeshStress` in the line.
                self.insert_stress_meshes(self::stress::DEFAULT_STRESS_MESHES)?;
                return Ok(());
            }
            _ if solid::handles(feature) => {
                self.dispatch_solid(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles(feature) => {
                let geometry = surface_edit::apply(
                    feature,
                    self.document(),
                    &self.selection,
                    self.coordinator.limits(),
                )?;
                self.events.emit(
                    crate::command_events::EventKind::Started,
                    surface_edit::static_feature(feature),
                    "selected surface",
                    None,
                );
                self.commit_geometry(
                    surface_edit::static_feature(feature),
                    surface_edit::label(feature),
                    geometry,
                )?;
                return Ok(());
            }
            _ if surface_edit::handles_numeric(feature) => {
                self.dispatch_surface_distance(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_drape(feature) => {
                self.dispatch_drape(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_fillet(feature) => {
                self.dispatch_fillet(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_variable_chamfer(feature) => {
                self.dispatch_variable_chamfer(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_chamfer(feature) => {
                self.dispatch_chamfer(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_untrim(feature) => {
                self.dispatch_untrim(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_variable_blend(feature) => {
                self.dispatch_variable_blend(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_shrink(feature) => {
                self.dispatch_shrink(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_variable_offset(feature) => {
                self.dispatch_variable_offset(feature)?;
                return Ok(());
            }
            "EdgeSrf" => {
                let geometry = surface_edit::apply_curves(
                    self::surface_edit::EDGE_CURVES,
                    self.document(),
                    self.selected_ordered().as_slice(),
                    self.coordinator.limits(),
                )?;
                self.commit_geometry(
                    self::surface_edit::EDGE_CURVES,
                    "EdgeSrf",
                    geometry,
                )?;
                return Ok(());
            }
            "PlanarSrf" => {
                self.dispatch_planar_curves(self::surface_edit::PLANAR_CURVES)?;
                return Ok(());
            }
            _ if surface_edit::handles_planar(feature) => {
                self.dispatch_planar_curves(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_extrude_all(feature) => {
                self.dispatch_extrude_all(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_tangent(feature) => {
                self.dispatch_tangent(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_split(feature) => {
                self.dispatch_split(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_extend(feature) => {
                self.dispatch_extend(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_tween(feature) => {
                self.dispatch_tween(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_soft_edit(feature) => {
                self.dispatch_soft_edit(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_seam(feature) => {
                self.dispatch_seam(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_surface_pair(feature) => {
                self.dispatch_surface_pair(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_merge(feature) => {
                self.dispatch_merge(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_sweep_multi(feature) => {
                self.dispatch_sweep_multi(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_curve_pair(feature) => {
                self.dispatch_curve_pair(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_four_rail_profile(feature) => {
                self.dispatch_four_rail_profile(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_sweep4(feature) => {
                self.dispatch_sweep4(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_history(feature) => {
                self.dispatch_sweep_history(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_curves(feature) => {
                let geometry = surface_edit::apply_curves(
                    feature,
                    self.document(),
                    &self.selected_ordered(),
                    self.coordinator.limits(),
                )?;
                self.events.emit(
                    crate::command_events::EventKind::Started,
                    surface_edit::static_feature(feature),
                    "selected curves",
                    None,
                );
                self.commit_geometry(
                    surface_edit::static_feature(feature),
                    surface_edit::label(feature),
                    geometry,
                )?;
                return Ok(());
            }
            surface_edit::PLANE_CORNERS | surface_edit::CUTTING_PLANE => {
                surface_edit::static_feature(feature)
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
            points: Vec::new(),
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
        if self.top11_pending.is_some() {
            return CmdPhase::AwaitPoint;
        }
        if self.solid_pending.is_some()
            || self.circle_pending.is_some()
            || self.rect_pending.is_some()
            || self.polyline_pending.is_some()
        {
            return CmdPhase::AwaitPoint;
        }
        if self.surface_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.curve_pair_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.surface_pair_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.seam_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.soft_edit_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.tween_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.extend_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.split_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.tangent_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.extrude_all_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.variable_offset_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.shrink_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.variable_blend_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.chamfer_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.fillet_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.variable_chamfer_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.history_options_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.slash_pending.is_some() {
            return CmdPhase::AwaitPoint;
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
        if let Some(pending) = &self.solid_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.circle_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.rect_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.polyline_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.interp_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.ellipse_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.arc_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.rebuild_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.fit_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.offset_curve_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.project_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.blend_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.isocurve_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.pull_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.intersect_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.crv2view_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.crv_seam_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.control_curve_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.offset_on_surface_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.offset_normal_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.corner_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.cross_section_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.fillet_corners_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.continue_curve_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.through_points_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.polygon_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.helix_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.spiral_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.divide_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.to_lines_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.extend_curve_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.tween_curves_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.continue_interp_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.arc_blend_pending {
            return pending.prompt();
        }
        if let Some(prompt) = self.top11_prompt() {
            return prompt;
        }
        if let Some(pending) = &self.surface_pending {
            return if pending.feature == surface_edit::OFFSET {
                "Offset distance: "
            } else {
                "Extrude distance: "
            };
        }
        if self.curve_pair_pending.is_some() {
            return "Rail revolve axis origin,direction: ";
        }
        if self.surface_pair_pending.is_some() {
            return "Match edges (source,target): ";
        }
        if self.seam_pending.is_some() {
            return "Seam axis,tolerance (e.g. U,0.01): ";
        }
        if self.soft_edit_pending.is_some() {
            return "Soft edit u,v,radius_u,radius_v,dx,dy,dz: ";
        }
        if let Some(pending) = &self.curve_soft_edit_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.angle_isocurve_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.extrude_surface_pending {
            return pending.prompt();
        }
        if let Some(pending) = &self.heightfield_pending {
            return pending.prompt();
        }
        if self.tween_pending.is_some() {
            return "Tween fraction (0..1): ";
        }
        if self.extend_pending.is_some() {
            return "Extend edge,distance (e.g. U1,5): ";
        }
        if self.split_pending.is_some() {
            return "Split axis,parameter (e.g. U,0.5): ";
        }
        if self.tangent_pending.is_some() {
            return "Tangent edge,dx,dy,dz: ";
        }
        if self.extrude_all_pending.is_some() {
            return "Extrude all height: ";
        }
        if self.variable_offset_pending.is_some() {
            return "Distances start,end,axis (e.g. 1,5,U): ";
        }
        if self.shrink_pending.is_some() {
            return "Shrink u0,u1,v0,v1: ";
        }
        if self.variable_blend_pending.is_some() {
            return "Tangent scales start,end (e.g. 1,2): ";
        }
        if self.chamfer_pending.is_some() {
            return "Chamfer setback: ";
        }
        if let Some(pending) = &self.fillet_pending {
            return if pending.feature == surface_edit::VARIABLE_FILLET {
                "Fillet radii start,end: "
            } else {
                "Fillet radius: "
            };
        }
        if self.variable_chamfer_pending.is_some() {
            return "Chamfer setbacks start,end: ";
        }
        if self.history_options_pending.is_some() {
            return "Sweep options closed,flip,simple,slash (Enter=open): ";
        }
        if let Some(pending) = &self.slash_pending {
            return if pending.points.is_empty() {
                "Slash point on first rail: "
            } else {
                "Slash point on second rail: "
            };
        }
        match &self.session {
            None => "Command: ",
            Some(s) if s.feature == POINT => "Location of point: ",
            Some(s) if surface_edit::plane_like(s.feature) => match s.points.len() {
                0 => "First corner: ",
                1 => "Second corner: ",
                _ => "Third corner: ",
            },
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
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .expect("active view");
        let revision = view
            .cplane
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("CPlane revision"))?;
        view.cplane = CPlane { revision, ..plane };
        Ok(())
    }
    /// Apply one of the world construction-plane presets to the active view.
    pub fn set_cplane_preset(&mut self, preset: om9_scene::camera::CPlanePreset) -> Result<()> {
        self.set_cplane(CPlane::preset(preset))
    }
    /// Build a plane from three points through the validated frame constructor, then
    /// assign it atomically. Degenerate/collinear points leave the current plane alone.
    pub fn set_cplane_from_three_points(
        &mut self,
        origin: Point3,
        x_point: Point3,
        y_point: Point3,
    ) -> Result<()> {
        let plane = CPlane::from_three_points(origin, x_point, y_point)?;
        self.set_cplane(plane)
    }
    /// Move and rotate the active plane without touching world geometry.
    pub fn set_cplane_origin_rotation(&mut self, origin: Point3, radians: f64) -> Result<()> {
        let plane = self.input_plane().with_origin(origin)?.rotated(radians)?;
        self.set_cplane(plane)
    }

    pub fn zoom_extents(&mut self, aspect: f64) -> Result<()> {
        let (center, radius) = self.world_bounds(false)?;
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera.fit_bounds(center, radius, aspect, 1.1)
    }
    pub fn zoom_selected(&mut self, aspect: f64) -> Result<()> {
        if self.selection.is_empty() {
            return Err(Error::Missing("selection bounds"));
        }
        let (center, radius) = self.world_bounds(true)?;
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera.fit_bounds(center, radius, aspect, 1.1)
    }
    /// Restore the active view's startup camera; CPlane, layers and document are untouched.
    pub fn view_home(&mut self) -> Result<()> {
        let id = self.workspace.active;
        let camera = Workspace::default_camera(id);
        let view = self
            .workspace
            .views
            .get_mut(&id)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera = camera;
        Ok(())
    }
    /// Move the active view target to the world origin without changing rotation or zoom.
    pub fn view_center(&mut self) -> Result<()> {
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera.center_at([0.; 3])
    }
    fn world_bounds(&self, selected: bool) -> Result<(Point3, f64)> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("navigation scene"))?;
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut found = false;
        for object in &scene.objects {
            if !object.visible || (selected && !self.selection.contains(&object.root)) {
                continue;
            }
            let transform = object.display_transform()?;
            let mut include = |point: Point3| -> Result<()> {
                let world = transform.point(point)?;
                for axis in 0..3 {
                    min[axis] = min[axis].min(world[axis]);
                    max[axis] = max[axis].max(world[axis]);
                }
                found = true;
                Ok(())
            };
            match object.buffers.as_ref() {
                RenderBuffers::Points { points, .. } | RenderBuffers::Curve { points, .. } => {
                    for point in points {
                        include(*point)?;
                    }
                }
                RenderBuffers::Mesh { vertices, .. } => {
                    for point in vertices {
                        include(*point)?;
                    }
                }
            }
        }
        if !found {
            return Err(Error::Missing("fit bounds"));
        }
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let diagonal = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        let radius =
            (diagonal[0] * diagonal[0] + diagonal[1] * diagonal[1] + diagonal[2] * diagonal[2])
                .sqrt()
                * 0.5;
        // A single point has zero extent; keep the fit finite instead of dividing by zero.
        Ok((center, radius.max(0.001)))
    }
    pub fn set_ortho(&mut self, enabled: bool) {
        self.ortho = enabled;
    }
    pub fn set_planar(&mut self, enabled: bool) {
        self.planar = enabled;
    }
    pub fn set_project(&mut self, enabled: bool) {
        self.project = enabled;
    }
    pub fn ortho(&self) -> bool {
        self.ortho
    }
    pub fn planar(&self) -> bool {
        self.planar
    }
    pub fn project(&self) -> bool {
        self.project
    }
    /// Previous accepted point in the active command, or the last accepted point.
    pub fn relative_base(&self) -> Option<om9_core::coords::WorldPoint> {
        self.session
            .as_ref()
            .and_then(|session| session.start)
            .and_then(|point| om9_core::coords::WorldPoint::try_new(point).ok())
            .or(self.last_point)
    }
    /// Insert the world origin through the ordinary typed point path (Matrix F4).
    /// Refuses outside a point prompt, so a shortcut can never create geometry from idle.
    pub fn submit_world_origin(&mut self) -> Result<()> {
        if self.phase() == CmdPhase::Idle {
            return Err(Error::Missing("point prompt"));
        }
        self.submit_geometry_text("W0,0,0")
    }
    /// Apply the active Ortho/Planar cursor constraints to a world-space pick.
    pub fn constrain_point(&self, point: Point3) -> Result<Point3> {
        finite(point)?;
        let plane = self.input_plane();
        if !self.ortho && !self.planar && !self.project {
            return Ok(point);
        }
        let world = om9_core::coords::WorldPoint::try_new(point)?;
        let local = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, world)?.xyz();
        let mut constrained = local;
        if self.planar || self.project {
            constrained[2] = 0.;
        }
        if self.ortho
            && let Some(base) = self.relative_base()
        {
            let base = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, base)?.xyz();
            let mut delta = [constrained[0] - base[0], constrained[1] - base[1], 0.];
            if delta[0].abs() >= delta[1].abs() {
                delta[1] = 0.;
            } else {
                delta[0] = 0.;
            }
            constrained = [base[0] + delta[0], base[1] + delta[1], 0.];
        }
        plane.point(constrained)
    }
    /// Project an arbitrary world point onto the active CPlane's local Z=0.
    /// This is the Project snap; it never changes the CPlane itself.
    pub fn project_to_plane(&self, point: Point3) -> Result<Point3> {
        finite(point)?;
        let plane = self.input_plane();
        let world = om9_core::coords::WorldPoint::try_new(point)?;
        let local = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, world)?.xyz();
        plane.point([local[0], local[1], 0.])
    }
    pub fn cancel(&mut self) {
        self.solid_pending = None;
        self.circle_pending = None;
        self.rect_pending = None;
        self.polyline_pending = None;
        self.interp_pending = None;
        self.ellipse_pending = None;
        self.arc_pending = None;
        self.rebuild_pending = None;
        self.fit_pending = None;
        self.offset_curve_pending = None;
        self.project_pending = None;
        self.blend_pending = None;
        self.isocurve_pending = None;
        self.pull_pending = None;
        self.intersect_pending = None;
        self.crv2view_pending = None;
        self.crv_seam_pending = None;
        self.control_curve_pending = None;
        self.offset_on_surface_pending = None;
        self.offset_normal_pending = None;
        self.corner_pending = None;
        self.cross_section_pending = None;
        self.fillet_corners_pending = None;
        self.soft_edit_pending = None;
        self.curve_soft_edit_pending = None;
        self.angle_isocurve_pending = None;
        self.extrude_surface_pending = None;
        self.heightfield_pending = None;
        self.continue_curve_pending = None;
        self.through_points_pending = None;
        self.polygon_pending = None;
        self.helix_pending = None;
        self.spiral_pending = None;
        self.divide_pending = None;
        self.to_lines_pending = None;
        self.extend_curve_pending = None;
        self.tween_curves_pending = None;
        self.continue_interp_pending = None;
        self.arc_blend_pending = None;
        if let Some(pending) = self.top11_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.action().command(),
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        self.top11_grip = None;
        if let Some(pending) = self.surface_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.curve_pair_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.surface_pair_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.seam_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.soft_edit_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.tween_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.extend_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.split_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.tangent_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.extrude_all_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.variable_offset_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.shrink_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.variable_blend_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.chamfer_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.fillet_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.variable_chamfer_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.history_options_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.slash_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
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
        if text.trim().is_empty()
            && let Some(result) = self.top11_finish()
        {
            result?;
            return Ok(DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_plugin_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(kind) = crate::file_lifecycle::FileKind::from_command(text.trim()) {
            return self.dispatch_effect(kind.feature_id(), InvocationOrigin::Cmd);
        }
        if let Some(result) = self.submit_circle_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_rect_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_polyline_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_interp_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_ellipse_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_arc_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_rebuild_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_fit_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_offset_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_project_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_blend_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_isocurve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_pull_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_intersect_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_crv2view_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_through_points_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_crv_seam_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_control_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_continue_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_offset_on_surface_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_offset_normal_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_corner_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_curve_soft_edit_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_angle_isocurve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_extrude_surface_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_heightfield_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_cross_section_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_fillet_corners_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_polygon_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_helix_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_spiral_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_divide_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_to_lines_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_extend_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_tween_curves_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_continue_interp_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_arc_blend_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_solid_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.top11_submit(text.trim()) {
            result?;
            return Ok(DispatchOutcome::Command);
        }
        self.submit_geometry_text(text)
            .map(|_| DispatchOutcome::Command)
    }
    fn submit_geometry_text(&mut self, text: &str) -> Result<()> {
        if text.len() > 4096 {
            return Err(Error::Budget("command input"));
        }
        let text = text.trim();
        if self.history_options_pending.is_some() {
            let pending = self
                .history_options_pending
                .as_ref()
                .expect("history options")
                .clone();
            let mut closed = false;
            let mut flip = false;
            let mut slash = false;
            if !text.is_empty() {
                for token in text.to_ascii_lowercase().split([',', ' ']) {
                    match token.trim() {
                        "closed" | "yes" => closed = true,
                        "flip" => flip = true,
                        "slash" => slash = true,
                        "open" | "no" | "simple" => {}
                        "" => {}
                        _ => return Err(Error::Invalid("sweep option")),
                    }
                }
            }
            if slash {
                if pending.rail_count != 2 {
                    return Err(Error::Invalid("slash needs two rails"));
                }
                self.history_options_pending = None;
                self.slash_pending = Some(SlashPending {
                    feature: pending.feature,
                    curves: pending.curves,
                    ids: pending.ids,
                    rail_count: pending.rail_count,
                    closed,
                    flip,
                    points: Vec::new(),
                });
                return Ok(());
            }
            let brep = self.build_sweep_history(
                pending.feature,
                &pending.curves,
                pending.rail_count,
                closed,
                flip,
                None,
            )?;
            let entity = Entity::new(
                surface_edit::label(pending.feature),
                Geometry::Brep(brep),
                self.document().current_layer,
            );
            let mut inputs = Vec::with_capacity(pending.ids.len());
            for (index, id) in pending.ids.iter().enumerate() {
                inputs.push(SourceBinding {
                    role: if index < pending.rail_count {
                        "rail"
                    } else {
                        "profile"
                    }
                    .into(),
                    source: self.document().stamp(*id)?,
                    selection: Selection::Whole,
                });
            }
            self.history_options_pending = None;
            return self.commit_entity_with_history(
                pending.feature,
                surface_edit::label(pending.feature),
                entity,
                if pending.feature == surface_edit::SWEEP2_HISTORY {
                    "Sweep2History"
                } else {
                    "Sweep1History"
                },
                inputs,
            );
        }
        if self.fillet_pending.is_some() {
            let pending = self.fillet_pending.as_ref().expect("fillet").clone();
            let (start, end) = if pending.feature == surface_edit::VARIABLE_FILLET {
                let values: Vec<f64> = text
                    .split(',')
                    .map(|value| value.trim().parse::<f64>())
                    .collect::<std::result::Result<_, _>>()
                    .map_err(|_| Error::Invalid("fillet radii"))?;
                if values.len() != 2 {
                    return Err(Error::Invalid("fillet radii"));
                }
                (values[0], values[1])
            } else {
                let radius: f64 = text.parse().map_err(|_| Error::Invalid("fillet radius"))?;
                (radius, radius)
            };
            let geometry = surface_edit::apply_fillet(
                &pending.first,
                &pending.second,
                start,
                end,
                self.coordinator.limits(),
            )?;
            self.fillet_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.variable_chamfer_pending.is_some() {
            let values: Vec<f64> = text
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("chamfer setbacks"))?;
            if values.len() != 2 {
                return Err(Error::Invalid("chamfer setbacks"));
            }
            let pending = self
                .variable_chamfer_pending
                .as_ref()
                .expect("variable chamfer")
                .clone();
            let geometry = surface_edit::apply_variable_chamfer(
                &pending.first,
                &pending.second,
                values[0],
                values[1],
                self.coordinator.limits(),
            )?;
            self.variable_chamfer_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.chamfer_pending.is_some() {
            let setback: f64 = text
                .parse()
                .map_err(|_| Error::Invalid("chamfer setback"))?;
            if !setback.is_finite() || setback <= 0.0 {
                return Err(Error::Invalid("chamfer setback"));
            }
            let pending = self.chamfer_pending.as_ref().expect("chamfer").clone();
            let geometry = surface_edit::apply_chamfer(
                &pending.first,
                &pending.second,
                setback,
                self.coordinator.limits(),
            )?;
            self.chamfer_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.variable_blend_pending.is_some() {
            let values: Vec<f64> = text
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("variable blend scale"))?;
            if values.len() != 2 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("variable blend scale"));
            }
            let pending = self
                .variable_blend_pending
                .as_ref()
                .expect("variable blend")
                .clone();
            let geometry = surface_edit::apply_variable_blend(
                &pending.first,
                &pending.second,
                values[0],
                values[1],
                self.coordinator.limits(),
            )?;
            self.variable_blend_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.shrink_pending.is_some() {
            let values: Vec<f64> = text
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("shrink domain"))?;
            if values.len() != 4 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("shrink domain"));
            }
            let pending = self.shrink_pending.as_ref().expect("shrink").clone();
            let geometry = surface_edit::apply_shrink(
                &pending.surface,
                [values[0], values[1]],
                [values[2], values[3]],
                self.coordinator.limits(),
            )?;
            self.shrink_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.variable_offset_pending.is_some() {
            let mut parts = text.split(',');
            let start = parts
                .next()
                .ok_or(Error::Invalid("variable offset start"))?
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Invalid("variable offset start"))?;
            let end = parts
                .next()
                .ok_or(Error::Invalid("variable offset end"))?
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Invalid("variable offset end"))?;
            let axis = parts
                .next()
                .ok_or(Error::Invalid("variable offset axis"))
                .and_then(surface_edit::parse_iso)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("variable offset input"));
            }
            let pending = self
                .variable_offset_pending
                .as_ref()
                .expect("variable offset")
                .clone();
            let geometry = surface_edit::apply_variable_offset(
                &pending.surface,
                start,
                end,
                axis,
                self.coordinator.limits(),
            )?;
            self.variable_offset_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.extrude_all_pending.is_some() {
            let height: f64 = text.parse().map_err(|_| Error::Invalid("extrude height"))?;
            if !height.is_finite() || height == 0.0 {
                return Err(Error::Invalid("extrude height"));
            }
            let pending = self
                .extrude_all_pending
                .as_ref()
                .expect("extrude all")
                .clone();
            let geometries = surface_edit::apply_extrude_all(
                &pending.curves,
                height,
                self.coordinator.limits(),
            )?;
            self.extrude_all_pending = None;
            let items = geometries
                .into_iter()
                .map(|geometry| (surface_edit::label(pending.feature), geometry))
                .collect();
            return self.commit_geometries(
                pending.feature,
                surface_edit::label(pending.feature),
                items,
            );
        }
        if self.tangent_pending.is_some() {
            let mut parts = text.split(',');
            let edge = parts
                .next()
                .ok_or(Error::Invalid("surface tangent edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            let values: Vec<f64> = parts
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("surface tangent direction"))?;
            if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("surface tangent direction"));
            }
            let pending = self.tangent_pending.as_ref().expect("tangent").clone();
            let geometry = surface_edit::apply_tangent(
                &pending.surface,
                edge,
                [values[0], values[1], values[2]],
                self.coordinator.limits(),
            )?;
            self.tangent_pending = None;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.split_pending.is_some() {
            let mut parts = text.split(',');
            let axis = parts
                .next()
                .ok_or(Error::Invalid("surface split axis"))
                .and_then(surface_edit::parse_iso)?;
            let parameter = parts
                .next()
                .ok_or(Error::Invalid("surface split parameter"))?
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Invalid("surface split parameter"))?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface split input"));
            }
            let pending = self.split_pending.as_ref().expect("split").clone();
            let geometries = surface_edit::apply_split(
                &pending.surface,
                axis,
                parameter,
                self.coordinator.limits(),
            )?;
            self.split_pending = None;
            self.events.emit(
                crate::command_events::EventKind::Started,
                pending.feature,
                "surface split",
                None,
            );
            let items = geometries
                .into_iter()
                .map(|geometry| (surface_edit::label(pending.feature), geometry))
                .collect();
            return self.commit_geometries(
                pending.feature,
                surface_edit::label(pending.feature),
                items,
            );
        }
        if self.extend_pending.is_some() {
            let mut parts = text.split(',');
            let edge = parts
                .next()
                .ok_or(Error::Invalid("surface extend edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            let distance = parts
                .next()
                .ok_or(Error::Invalid("surface extend distance"))?
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Invalid("surface extend distance"))?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface extend input"));
            }
            let pending = self.extend_pending.take().expect("extend");
            let geometry = surface_edit::apply_extend(
                &pending.surface,
                edge,
                distance,
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.tween_pending.is_some() {
            let fraction: f64 = text.parse().map_err(|_| Error::Invalid("tween fraction"))?;
            if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                return Err(Error::Invalid("tween fraction"));
            }
            let pending = self.tween_pending.take().expect("tween");
            let geometry = surface_edit::apply_tween(
                &pending.first,
                &pending.second,
                fraction,
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.soft_edit_pending.is_some() {
            let values: Vec<f64> = text
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("soft edit input"))?;
            if values.len() != 7 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("soft edit input"));
            }
            let pending = self.soft_edit_pending.take().expect("soft edit");
            let geometry = surface_edit::apply_soft_edit(
                &pending.surface,
                [values[0], values[1]],
                [values[2], values[3]],
                [values[4], values[5], values[6]],
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.seam_pending.is_some() {
            let mut parts = text.split(',');
            let axis = parts
                .next()
                .ok_or(Error::Invalid("surface seam axis"))
                .and_then(surface_edit::parse_iso)?;
            let tolerance = parts
                .next()
                .ok_or(Error::Invalid("surface seam tolerance"))?
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Invalid("surface seam tolerance"))?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface seam input"));
            }
            let pending = self.seam_pending.take().expect("seam");
            let geometry = surface_edit::apply_seam(
                &pending.surface,
                axis,
                tolerance,
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.surface_pair_pending.is_some() {
            let mut parts = text.split(',');
            let source = parts
                .next()
                .ok_or(Error::Invalid("surface source edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            let target = parts
                .next()
                .ok_or(Error::Invalid("surface target edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface match edges"));
            }
            let pending = self.surface_pair_pending.take().expect("surface pair");
            let geometry = surface_edit::apply_match(
                &pending.source,
                &pending.target,
                source,
                target,
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.curve_pair_pending.is_some() {
            let values: Vec<f64> = text
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("rail revolve axis"))?;
            if values.len() != 6 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("rail revolve axis"));
            }
            let pending = self.curve_pair_pending.take().expect("curve pair");
            let geometry = surface_edit::apply_rail_revolve(
                &pending.profile,
                &pending.rail,
                [values[0], values[1], values[2]],
                [values[3], values[4], values[5]],
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if self.surface_pending.is_some() {
            let distance: f64 = text
                .parse()
                .map_err(|_| Error::Invalid("surface distance"))?;
            if !distance.is_finite() || distance == 0.0 {
                return Err(Error::Invalid("surface distance"));
            }
            let pending = self.surface_pending.take().expect("surface pending");
            let geometry = surface_edit::apply_numeric(
                pending.feature,
                &pending.surface,
                distance,
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        if let Some(command) = parse_gumball_placement(text) {
            match command {
                GumballPlacement::Origin(origin) => self.relocate_gumball(Some(origin)),
                GumballPlacement::Reset => self.relocate_gumball(None),
                GumballPlacement::World => {
                    self.set_gumball_alignment(self::gumball::GumballAlignment::World)
                }
                GumballPlacement::Object => {
                    self.set_gumball_alignment(self::gumball::GumballAlignment::Object)
                }
                GumballPlacement::CPlane => {
                    self.set_gumball_alignment(self::gumball::GumballAlignment::CPlane)
                }
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
            return match Self::resolve_alias(text) {
                Some(LINE) => self.dispatch(LINE, InvocationOrigin::Cmd),
                Some(POINT) => self.dispatch(POINT, InvocationOrigin::Cmd),
                Some(self::circle::CIRCLE) => self.dispatch(self::circle::CIRCLE, InvocationOrigin::Cmd),
                Some(self::rect::RECTANGLE) => {
                    self.dispatch(self::rect::RECTANGLE, InvocationOrigin::Cmd)
                }
                Some(self::polyline::POLYLINE) => {
                    self.dispatch(self::polyline::POLYLINE, InvocationOrigin::Cmd)
                }
                Some(self::interp::INTERP) => {
                    self.dispatch(self::interp::INTERP, InvocationOrigin::Cmd)
                }
                Some(self::ellipse::ELLIPSE) => {
                    self.dispatch(self::ellipse::ELLIPSE, InvocationOrigin::Cmd)
                }
                Some(self::arc::ARC) => self.dispatch(self::arc::ARC, InvocationOrigin::Cmd),
                Some(self::arc::ARC_START) => {
                    self.dispatch(self::arc::ARC_START, InvocationOrigin::Cmd)
                }
                Some(self::rebuild::REBUILD) => {
                    self.dispatch(self::rebuild::REBUILD, InvocationOrigin::Cmd)
                }
                Some(self::fit::FIT) => self.dispatch(self::fit::FIT, InvocationOrigin::Cmd),
                Some(self::offset::OFFSET) => {
                    self.dispatch(self::offset::OFFSET, InvocationOrigin::Cmd)
                }
                Some(self::project::PROJECT) => {
                    self.dispatch(self::project::PROJECT, InvocationOrigin::Cmd)
                }
                Some(self::blend::BLEND) => self.dispatch(self::blend::BLEND, InvocationOrigin::Cmd),
                Some(self::blend::BLEND_CRV) => {
                    self.dispatch(self::blend::BLEND_CRV, InvocationOrigin::Cmd)
                }
                Some(self::blend::MATCH) => self.dispatch(self::blend::MATCH, InvocationOrigin::Cmd),
                Some(self::isocurve::ISOCURVE) => {
                    self.dispatch(self::isocurve::ISOCURVE, InvocationOrigin::Cmd)
                }
                Some(self::pull::PULL) => self.dispatch(self::pull::PULL, InvocationOrigin::Cmd),
                Some(self::intersect::INTERSECT) => {
                    self.dispatch(self::intersect::INTERSECT, InvocationOrigin::Cmd)
                }
                Some(self::intersect_two_sets::INTERSECT_TWO_SETS) => {
                    self.dispatch(
                        self::intersect_two_sets::INTERSECT_TWO_SETS,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::crv2view::CRV2VIEW) => {
                    self.dispatch(self::crv2view::CRV2VIEW, InvocationOrigin::Cmd)
                }
                Some(self::through_points::THROUGH_POINTS) => {
                    self.dispatch(self::through_points::THROUGH_POINTS, InvocationOrigin::Cmd)
                }
                Some(self::polygon::POLYGON) => {
                    self.dispatch(self::polygon::POLYGON, InvocationOrigin::Cmd)
                }
                Some(self::helix::HELIX) => self.dispatch(self::helix::HELIX, InvocationOrigin::Cmd),
                Some(self::spiral::SPIRAL) => {
                    self.dispatch(self::spiral::SPIRAL, InvocationOrigin::Cmd)
                }
                Some(self::divide::DIVIDE) => {
                    self.dispatch(self::divide::DIVIDE, InvocationOrigin::Cmd)
                }
                Some(self::to_lines::TO_LINES) => {
                    self.dispatch(self::to_lines::TO_LINES, InvocationOrigin::Cmd)
                }
                Some(self::extend_curve::EXTEND) => {
                    self.dispatch(self::extend_curve::EXTEND, InvocationOrigin::Cmd)
                }
                Some(self::tween_curves::TWEEN_CURVES) => {
                    self.dispatch(self::tween_curves::TWEEN_CURVES, InvocationOrigin::Cmd)
                }
                Some(self::continue_interp::CONTINUE_INTERP) => {
                    self.dispatch(self::continue_interp::CONTINUE_INTERP, InvocationOrigin::Cmd)
                }
                Some(self::arc_blend::ARC_BLEND) => {
                    self.dispatch(self::arc_blend::ARC_BLEND, InvocationOrigin::Cmd)
                }
                Some(self::section::SECTION) => {
                    self.dispatch(self::section::SECTION, InvocationOrigin::Cmd)
                }
                Some(self::revolve::REVOLVE) => {
                    self.dispatch(self::revolve::REVOLVE, InvocationOrigin::Cmd)
                }
                Some(self::curve_boolean::CURVE_BOOLEAN) => {
                    self.dispatch(self::curve_boolean::CURVE_BOOLEAN, InvocationOrigin::Cmd)
                }
                Some(self::heightfield::HEIGHTFIELD) => {
                    self.dispatch(self::heightfield::HEIGHTFIELD, InvocationOrigin::Cmd)
                }
                Some(self::extrude_surface::EXTRUDE) => {
                    self.dispatch(self::extrude_surface::EXTRUDE, InvocationOrigin::Cmd)
                }
                Some(self::angle_isocurve::ANGLE_ISOCURVE) => {
                    self.dispatch(
                        self::angle_isocurve::ANGLE_ISOCURVE,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::curve_soft_edit::CURVE_SOFT_EDIT) => {
                    self.dispatch(
                        self::curve_soft_edit::CURVE_SOFT_EDIT,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::cross_section::CROSS_SECTION) => {
                    self.dispatch(self::cross_section::CROSS_SECTION, InvocationOrigin::Cmd)
                }
                Some(self::fillet_corners::FILLET_CORNERS) => {
                    self.dispatch(self::fillet_corners::FILLET_CORNERS, InvocationOrigin::Cmd)
                }
                Some(self::silhouette::SILHOUETTE) => {
                    self.dispatch(self::silhouette::SILHOUETTE, InvocationOrigin::Cmd)
                }
                Some(self::mesh_curve::MESH_CURVE) => {
                    self.dispatch(self::mesh_curve::MESH_CURVE, InvocationOrigin::Cmd)
                }
                Some(self::uv_curves::CREATE_UV) => {
                    self.dispatch(self::uv_curves::CREATE_UV, InvocationOrigin::Cmd)
                }
                Some(self::apply_crv::APPLY_CRV) => {
                    self.dispatch(self::apply_crv::APPLY_CRV, InvocationOrigin::Cmd)
                }
                Some(self::offset_normal::OFFSET_NORMAL) => {
                    self.dispatch(self::offset_normal::OFFSET_NORMAL, InvocationOrigin::Cmd)
                }
                Some(self::corner_ops::FILLET) => {
                    self.dispatch(self::corner_ops::FILLET, InvocationOrigin::Cmd)
                }
                Some(self::corner_ops::CHAMFER) => {
                    self.dispatch(self::corner_ops::CHAMFER, InvocationOrigin::Cmd)
                }
                Some(self::offset_on_surface::OFFSET_ON_SURFACE) => {
                    self.dispatch(
                        self::offset_on_surface::OFFSET_ON_SURFACE,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::crv_seam::CRV_SEAM) => {
                    self.dispatch(self::crv_seam::CRV_SEAM, InvocationOrigin::Cmd)
                }
                Some(self::control_curve::CONTROL_CURVE) => {
                    self.dispatch(self::control_curve::CONTROL_CURVE, InvocationOrigin::Cmd)
                }
                Some(self::continue_curve::CONTINUE_CURVE) => {
                    self.dispatch(self::continue_curve::CONTINUE_CURVE, InvocationOrigin::Cmd)
                }
                Some(self::sketch::SKETCH) => {
                    self.dispatch(self::sketch::SKETCH, InvocationOrigin::Cmd)
                }
                Some(self::sketch_on_surface::SKETCH_ON_SURFACE) => {
                    self.dispatch(
                        self::sketch_on_surface::SKETCH_ON_SURFACE,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::polyline_on_surface::POLYLINE_ON_SURFACE) => {
                    self.dispatch(
                        self::polyline_on_surface::POLYLINE_ON_SURFACE,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::interp_on_surface::INTERP_ON_SURFACE) => {
                    self.dispatch(
                        self::interp_on_surface::INTERP_ON_SURFACE,
                        InvocationOrigin::Cmd,
                    )
                }
                Some(self::wireframe::DUP_EDGE) => {
                    self.dispatch(self::wireframe::DUP_EDGE, InvocationOrigin::Cmd)
                }
                Some(self::wireframe::EXTRACT_WIRE) => {
                    self.dispatch(self::wireframe::EXTRACT_WIRE, InvocationOrigin::Cmd)
                }
                Some(self::crv_ends::CRV_START) => {
                    self.dispatch(self::crv_ends::CRV_START, InvocationOrigin::Cmd)
                }
                Some(self::crv_ends::CRV_END) => {
                    self.dispatch(self::crv_ends::CRV_END, InvocationOrigin::Cmd)
                }
                Some(self::dup_border::DUP_BORDER) => {
                    self.dispatch(self::dup_border::DUP_BORDER, InvocationOrigin::Cmd)
                }
                Some(command)
                    if matches!(
                        command,
                        ZOOM_EXTENTS | ZOOM_SELECTED | VIEW_HOME | VIEW_CENTER
                    ) =>
                {
                    self.dispatch(command, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_planar(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_curves(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_numeric(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_extend(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_shrink(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_tween(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_merge(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_seam(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_surface_pair(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some("undo") => self.undo(),
                Some("redo") => self.redo(),
                _ => Err(Error::Unsupported("unknown/unavailable command")),
            };
        }
        let s = self.validate_session()?.clone();
        if text.is_empty() {
            return Err(Error::Invalid("point required"));
        }
        // Rhino p22: an option can be activated by typing its underlined letter. BothSides is the only flag
        // the implemented commands declare, so `B` toggles it through the same assignment path.
        if s.feature == LINE && text.chars().count() == 1 && text.eq_ignore_ascii_case("b") {
            self.set_both_sides(!s.both_sides)?;
            return Ok(());
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
        // Typed world/local/relative input: the grammar, prompts and error labels are
        // unchanged, but a local point only becomes world through the captured frame.
        let input = om9_core::coords::parse_input(text, s.unit)?;
        let world =
            om9_core::coords::resolve_with_base(input, Some(&s.plane), self.relative_base())?;
        self.accept_world_point(world.xyz())
    }
    pub fn accept_world_point(&mut self, point: Point3) -> Result<()> {
        finite(point)?;
        if self.circle_pending.is_some() {
            return self.accept_circle_point(point);
        }
        if self.control_curve_pending.is_some() {
            return self.accept_control_curve_point(point);
        }
        if self.continue_curve_pending.is_some() {
            return self.accept_continue_curve_point(point);
        }
        if self.rect_pending.is_some() {
            return self.accept_rect_point(point);
        }
        if self.polyline_pending.is_some() {
            return self.accept_polyline_point(point);
        }
        if self.interp_pending.is_some() {
            return self.accept_interp_point(point);
        }
        if self.polygon_pending.is_some() {
            return self.accept_polygon_point(point);
        }
        if self.continue_interp_pending.is_some() {
            return self.accept_continue_interp_point(point);
        }
        if self.helix_pending.is_some() {
            return self.accept_helix_point(point);
        }
        if self.spiral_pending.is_some() {
            return self.accept_spiral_point(point);
        }
        if self.ellipse_pending.is_some() {
            return self.accept_ellipse_point(point);
        }
        if self.arc_pending.is_some() {
            return self.accept_arc_point(point);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.sphere_mode)
        {
            return self.accept_sphere_point(point);
        }
        if self.solid_pending.as_ref().is_some_and(|pending| {
            pending.box_mode
                && pending.box_kind == "Center"
                && pending.points.len() == 1
                && pending.box_length > 0.0
                && pending.box_width.is_some()
        }) {
            let pending = self.solid_pending.take().expect("center box pending");
            let center = pending.points[0];
            let height = (point[2] - center[2]).abs();
            if height <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("box height must be positive"));
            }
            return self.solid_box_dimensions(
                center,
                pending.box_length,
                pending.box_width.unwrap_or(pending.box_length),
                height,
            );
        }
        if let Some(pending) = self.solid_pending.as_mut()
            && pending.box_mode
            && pending.points.len() < 2
        {
            pending.points.push(point);
            return Ok(());
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.box_mode && pending.points.len() >= 2)
        {
            let pending = self.solid_pending.take().expect("box pending");
            if matches!(pending.box_kind, "ThreePoint" | "Vertical") && pending.box_width.is_none()
            {
                self.solid_pending = Some(pending);
                return Err(Error::Missing("enter the box width first"));
            }
            let first = pending.points[0];
            let second = pending.points[1];
            let height = (point[2] - first[2]).abs();
            if height <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("box height must be positive"));
            }
            return self.solid_box_points(
                first,
                second,
                height,
                pending.box_kind,
                pending.box_width,
            );
        }
        if self.slash_pending.is_some() {
            let mut pending = self.slash_pending.as_ref().expect("slash").clone();
            pending.points.push(point);
            if pending.points.len() < 2 {
                self.slash_pending = Some(pending);
                self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
                return Ok(());
            }
            let slash = [pending.points[0], pending.points[1]];
            self.slash_pending = None;
            let brep = self.build_sweep_history(
                pending.feature,
                &pending.curves,
                pending.rail_count,
                pending.closed,
                pending.flip,
                Some(slash),
            )?;
            let entity = Entity::new(
                surface_edit::label(pending.feature),
                Geometry::Brep(brep),
                self.document().current_layer,
            );
            let mut inputs = Vec::with_capacity(pending.ids.len());
            for (index, id) in pending.ids.iter().enumerate() {
                inputs.push(SourceBinding {
                    role: if index < pending.rail_count {
                        "rail"
                    } else {
                        "profile"
                    }
                    .into(),
                    source: self.document().stamp(*id)?,
                    selection: Selection::Whole,
                });
            }
            return self.commit_entity_with_history(
                pending.feature,
                surface_edit::label(pending.feature),
                entity,
                "Sweep2History",
                inputs,
            );
        }
        if self.top11_pending.is_some() {
            self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
            self.top11_accept_point(point)?;
            return Ok(());
        }
        let s = self.validate_session()?.clone();
        self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
        if surface_edit::plane_like(s.feature) {
            let collected = {
                let session = self.session.as_mut().expect("session");
                session.points.push(point);
                session.points.clone()
            };
            if collected.len() < 3 {
                self.events.emit(
                    crate::command_events::EventKind::Progress,
                    s.feature,
                    "corner",
                    None,
                );
                return Ok(());
            }
            let corners = [collected[0], collected[1], collected[2]];
            let surface = surface_edit::plane_from_corners(corners, self.coordinator.limits())?;
            return self.commit_geometry(
                s.feature,
                surface_edit::label(s.feature),
                Geometry::Surface(surface),
            );
        }
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
        self.commit_geometry(s.feature, name, geometry)
    }
    /// Start a fillet command that waits for one or two radii.
    fn dispatch_fillet(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let (first, second) = self.selected_curve_pair()?;
        self.fillet_pending = Some(FilletPending {
            feature,
            first,
            second,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "fillet strip",
            None,
        );
        Ok(())
    }
    /// Start a variable-chamfer command that waits for two setbacks.
    fn dispatch_variable_chamfer(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let (first, second) = self.selected_curve_pair()?;
        self.variable_chamfer_pending = Some(VarChamferPending {
            feature,
            first,
            second,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "variable chamfer strip",
            None,
        );
        Ok(())
    }
    fn selected_curve_pair(&self) -> Result<(Curve, Curve)> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        Ok((curves[0].clone(), curves[1].clone()))
    }
    /// Start a chamfer command that waits for a setback.
    fn dispatch_chamfer(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.chamfer_pending = Some(ChamferPending {
            feature,
            first: curves[0].clone(),
            second: curves[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "chamfer strip",
            None,
        );
        Ok(())
    }
    /// Remove one trim extension and commit the untrimmed copy.
    fn dispatch_untrim(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        if !matches!(entity.geometry, Geometry::Surface(_)) {
            return Err(Error::Invalid("surface selection"));
        }
        let untrimmed = surface_edit::untrim_entity(entity, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "untrim surface",
            None,
        );
        self.commit_entity(feature, surface_edit::label(feature), untrimmed)
    }
    /// Start a variable-blend command that waits for two tangent scales.
    fn dispatch_variable_blend(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.variable_blend_pending = Some(VarBlendPending {
            feature,
            first: curves[0].clone(),
            second: curves[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "variable blend",
            None,
        );
        Ok(())
    }
    /// Start a shrink command that waits for a UV sub-domain.
    fn dispatch_shrink(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.shrink_pending = Some(ShrinkPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "shrink surface",
            None,
        );
        Ok(())
    }
    /// Start a variable-offset command that waits for two distances and an axis.
    fn dispatch_variable_offset(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.variable_offset_pending = Some(VarOffsetPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "variable offset",
            None,
        );
        Ok(())
    }
    /// Build a planar surface from selected coplanar curves.
    fn dispatch_planar_curves(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        if curves.is_empty() {
            return Err(Error::Invalid("planar curve count"));
        }
        let geometry = surface_edit::apply_planar_curves(&curves, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "planar curves",
            None,
        );
        self.commit_geometry(feature, surface_edit::label(feature), geometry)
    }
    /// Start an extrude-all command that waits for a height.
    fn dispatch_extrude_all(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        if curves.is_empty() {
            return Err(Error::Invalid("curve selection"));
        }
        self.extrude_all_pending = Some(ExtrudeAllPending { feature, curves });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "extrude all curves",
            None,
        );
        Ok(())
    }
    /// Start a tangent command that waits for an edge and direction.
    fn dispatch_tangent(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.tangent_pending = Some(TangentPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface tangent",
            None,
        );
        Ok(())
    }
    /// Drape a surface over selected point entities.
    fn dispatch_drape(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        let mut points = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected point"))?;
            let Geometry::Point(point) = entity.geometry else {
                return Err(Error::Invalid("point selection"));
            };
            points.push(point);
        }
        if points.len() < 3 {
            return Err(Error::Invalid("drape point count"));
        }
        let geometry = surface_edit::apply_drape(&points, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "drape points",
            None,
        );
        self.commit_geometry(feature, surface_edit::label(feature), geometry)
    }
    /// Split a surface at an interior isocurve.
    fn dispatch_split(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.split_pending = Some(SplitPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface split",
            None,
        );
        Ok(())
    }
    /// Start an extend command that waits for an edge and distance.
    fn dispatch_extend(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.extend_pending = Some(ExtendPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface extend",
            None,
        );
        Ok(())
    }
    /// Start a tween command that waits for a fraction.
    fn dispatch_tween(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two surface selection"));
        }
        let mut surfaces = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected surface"))?;
            let Geometry::Surface(surface) = &entity.geometry else {
                return Err(Error::Invalid("surface selection"));
            };
            surfaces.push(surface.clone());
        }
        self.tween_pending = Some(TweenPending {
            feature,
            first: surfaces[0].clone(),
            second: surfaces[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface tween",
            None,
        );
        Ok(())
    }
    /// Start a soft-edit command that waits for UV, radius and delta.
    fn dispatch_soft_edit(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.soft_edit_pending = Some(SoftEditPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface soft edit",
            None,
        );
        Ok(())
    }
    /// Start a seam command that waits for its axis and tolerance.
    fn dispatch_seam(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.seam_pending = Some(SeamPending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface seam",
            None,
        );
        Ok(())
    }
    /// Start a two-surface command that waits for edge names.
    fn dispatch_surface_pair(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two surface selection"));
        }
        let mut surfaces = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected surface"))?;
            let Geometry::Surface(surface) = &entity.geometry else {
                return Err(Error::Invalid("surface selection"));
            };
            surfaces.push(surface.clone());
        }
        self.surface_pair_pending = Some(SurfacePairPending {
            feature,
            source: surfaces[0].clone(),
            target: surfaces[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface match edges",
            None,
        );
        Ok(())
    }
    /// Attach FourRailProfile metadata to selected profile curves.
    fn dispatch_four_rail_profile(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() < 5 {
            return Err(Error::Invalid("four rails and a profile"));
        }
        let rails: [om9_core::identity::ObjectId; 4] = ordered[..4]
            .try_into()
            .map_err(|_| Error::Invalid("four rails"))?;
        let mut rail_points = Vec::with_capacity(4);
        for id in &rails {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("four rail"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("four rail curve"));
            };
            rail_points.push(
                om9_core::surface_kernel::sample_curve(curve, 33, self.coordinator.limits())
                    .map_err(|_| Error::Unsupported("four rail curve"))?,
            );
        }
        let first_profile = self
            .document()
            .entities
            .get(&ordered[4])
            .ok_or(Error::Missing("four rail profile"))?;
        let Geometry::Curve(first_curve) = &first_profile.geometry else {
            return Err(Error::Invalid("profile curve"));
        };
        let first_points =
            om9_core::surface_kernel::sample_curve(first_curve, 64, self.coordinator.limits())
                .map_err(|_| Error::Unsupported("four rail profile curve"))?;
        let (_, rail_params) = rail_intersections(
            first_curve,
            &first_points,
            &rail_points,
            self.coordinator.limits(),
        )
        .ok_or(Error::Invalid("four rail profile intersections"))?;
        let mut rail_order = [0usize, 1, 2, 3];
        rail_order.sort_by(|left, right| rail_params[*left].total_cmp(&rail_params[*right]));
        let rails = [
            rails[rail_order[0]],
            rails[rail_order[1]],
            rails[rail_order[2]],
            rails[rail_order[3]],
        ];
        let mut mutations = Vec::with_capacity(ordered.len() - 4);
        for id in &ordered[4..] {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected profile"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("profile curve"));
            };
            let points =
                om9_core::surface_kernel::sample_curve(curve, 16, self.coordinator.limits())
                    .map_err(|_| Error::Unsupported("four rail profile curve"))?;
            let mut chord = None;
            let mut normal = None;
            'outer: for i in 1..points.len() {
                let first = [
                    points[i][0] - points[0][0],
                    points[i][1] - points[0][1],
                    points[i][2] - points[0][2],
                ];
                for j in (i + 1)..points.len() {
                    let second = [
                        points[j][0] - points[0][0],
                        points[j][1] - points[0][1],
                        points[j][2] - points[0][2],
                    ];
                    let candidate = [
                        first[1] * second[2] - first[2] * second[1],
                        first[2] * second[0] - first[0] * second[2],
                        first[0] * second[1] - first[1] * second[0],
                    ];
                    let length = candidate[0].hypot(candidate[1]).hypot(candidate[2]);
                    if length > 1.0e-9 {
                        chord = Some(first);
                        normal = Some([
                            candidate[0] / length,
                            candidate[1] / length,
                            candidate[2] / length,
                        ]);
                        break 'outer;
                    }
                }
            }
            let (chord, normal) = match (chord, normal) {
                (Some(chord), Some(normal)) => (chord, normal),
                _ => return Err(Error::Invalid("four rail profile plane")),
            };
            let chord_len = chord[0].hypot(chord[1]).hypot(chord[2]);
            let metadata = om9_core::rhino_semantics::FourRailProfile {
                rails,
                origin: points[0],
                normal,
                x_axis: [
                    chord[0] / chord_len,
                    chord[1] / chord_len,
                    chord[2] / chord_len,
                ],
            };
            let mut extensions = entity.extensions.clone();
            extensions.insert(
                om9_core::model::FOUR_RAIL_PROFILE_EXTENSION_SLOT.to_string(),
                metadata.to_extension()?,
            );
            mutations.push(Mutation::Extensions {
                source: self.document().stamp(*id)?,
                extensions,
            });
        }
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "four rail profile",
            None,
        );
        let prepared = self
            .coordinator
            .prepare(TransactionRequest::new("FourRailProfile", mutations))?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "four rail profile",
            Some(if receipt.published {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            }),
        );
        Ok(())
    }
    #[cfg(feature = "occt-brep")]
    fn build_sweep4(&self, curves: &[Curve]) -> Result<om9_core::brep::Brep> {
        use om9_core::surface_kernel;
        if curves.len() < 6 {
            return Err(Error::Invalid("four rails and two profiles"));
        }
        let limits = self.coordinator.limits();
        let mut rails = Vec::with_capacity(4);
        for curve in &curves[..4] {
            rails.push(
                surface_kernel::sample_curve(curve, 33, limits)
                    .map_err(|_| Error::Unsupported("Sweep 4 rail"))?,
            );
        }
        let mut profiles = Vec::with_capacity(curves.len() - 4);
        for curve in &curves[4..] {
            profiles.push(
                surface_kernel::sample_curve(curve, 64, limits)
                    .map_err(|_| Error::Unsupported("Sweep 4 profile"))?,
            );
        }
        let (_, first_params) = rail_intersections(&curves[4], &profiles[0], &rails, limits)
            .ok_or(Error::Invalid("four rail profile intersections"))?;
        let mut order = [0usize, 1, 2, 3];
        order.sort_by(|left, right| first_params[*left].total_cmp(&first_params[*right]));
        let ordered_rails: Vec<Vec<Point3>> =
            order.iter().map(|index| rails[*index].clone()).collect();
        let mut entries: Vec<(f64, [f64; 4], Vec<Point3>)> = Vec::with_capacity(profiles.len());
        for (index, profile) in profiles.into_iter().enumerate() {
            let (indices, _) = rail_intersections(&curves[4 + index], &profile, &rails, limits)
                .ok_or(Error::Invalid("four rail profile intersections"))?;
            let mut ordered = [0.0f64; 4];
            for (slot, rail) in order.iter().enumerate() {
                ordered[slot] = indices[*rail];
            }
            for left in 0..4 {
                for right in (left + 1)..4 {
                    if (ordered[left] - ordered[right]).abs() <= 1.0e-6 {
                        return Err(Error::Invalid("four rail profile intersections"));
                    }
                }
            }
            entries.push((ordered[0], ordered, profile));
        }
        entries.sort_by(|left, right| left.0.total_cmp(&right.0));
        let mut sections: [Vec<Vec<Point3>>; 4] = Default::default();
        for (_, indices, profile) in entries {
            for quadrant in 0..4 {
                let start = indices[quadrant];
                let end = indices[(quadrant + 1) % 4];
                sections[quadrant].push(profile_segment(&profile, start, end));
            }
        }
        om9_kernel::creation::four_rail_sweep(
            &ordered_rails
                .try_into()
                .map_err(|_| Error::Invalid("four rails"))?,
            &sections,
            1.0e-3,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|_| Error::Unsupported("Sweep 4 builder"))
    }

    #[cfg(feature = "occt-brep")]
    fn dispatch_sweep4(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let mut ordered = self.selected_ordered();
        if ordered.len() < 6 {
            // Profiles created by Four Rail Profile carry their four rail ids;
            // selecting only those profiles is enough to resolve the rails.
            let mut rails: Option<[om9_core::identity::ObjectId; 4]> = None;
            let mut profiles = Vec::new();
            for id in &ordered {
                let entity = self
                    .document()
                    .entities
                    .get(id)
                    .ok_or(Error::Missing("selected profile"))?;
                let metadata = om9_core::rhino_semantics::four_rail_profile_of(
                    entity,
                    self.coordinator.limits(),
                )?
                .ok_or(Error::Invalid("profile has no FourRailProfile metadata"))?;
                if let Some(existing) = rails {
                    if existing != metadata.rails {
                        return Err(Error::Invalid("profiles use different four rails"));
                    }
                } else {
                    rails = Some(metadata.rails);
                }
                profiles.push(*id);
            }
            let Some(rails) = rails else {
                return Err(Error::Invalid("four rails and two profiles"));
            };
            ordered = rails.to_vec();
            ordered.extend(profiles);
        }
        if ordered.len() < 6 {
            return Err(Error::Invalid("four rails and two profiles"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in &ordered {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        let brep = self.build_sweep4(&curves)?;
        let entity = Entity::new(
            surface_edit::label(feature),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let mut inputs = Vec::with_capacity(ordered.len());
        for (index, id) in ordered.iter().enumerate() {
            inputs.push(SourceBinding {
                role: if index < 4 { "rail" } else { "profile" }.into(),
                source: self.document().stamp(*id)?,
                selection: Selection::Whole,
            });
        }
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "four rail sweep",
            None,
        );
        self.commit_entity_with_history(
            feature,
            surface_edit::label(feature),
            entity,
            "Sweep4",
            inputs,
        )
    }

    #[cfg(not(feature = "occt-brep"))]
    fn dispatch_sweep4(&mut self, _feature: &str) -> Result<()> {
        Err(Error::Unsupported("Sweep 4 requires OCCT"))
    }
    /// Multi-profile Sweep 1/2 History through the OCCT pipe-shell adapter.
    #[cfg(feature = "occt-brep")]
    fn dispatch_sweep_history(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let rail_count = if feature == surface_edit::SWEEP2_HISTORY {
            2
        } else {
            1
        };
        let ordered = self.selected_ordered();
        if ordered.len() <= rail_count {
            return Err(Error::Invalid("rail and profile selection"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in &ordered {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.history_options_pending = Some(HistoryOptionsPending {
            feature,
            curves,
            ids: ordered,
            rail_count,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "multi-profile sweep options",
            None,
        );
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    fn build_sweep_history(
        &self,
        _feature: &'static str,
        curves: &[Curve],
        rail_count: usize,
        closed: bool,
        flip: bool,
        slash: Option<[Point3; 2]>,
    ) -> Result<om9_core::brep::Brep> {
        use om9_core::surface_kernel;
        let limits = self.coordinator.limits();
        let mut rails: Vec<Vec<Point3>> = Vec::with_capacity(rail_count);
        for curve in &curves[..rail_count] {
            rails.push(
                surface_kernel::sample_curve(curve, 33, limits)
                    .map_err(|_| Error::Unsupported("Sweep history builder"))?,
            );
        }
        let mut profiles: Vec<Vec<Point3>> = Vec::with_capacity(curves.len() - rail_count);
        for curve in &curves[rail_count..] {
            profiles.push(
                surface_kernel::sample_curve(curve, 24, limits)
                    .map_err(|_| Error::Unsupported("Sweep history builder"))?,
            );
        }
        if flip {
            profiles.reverse();
            for profile in &mut profiles {
                profile.reverse();
            }
        }
        if let Some([a, b]) = slash {
            let section = slash_alignment_section(&profiles, &rails, a, b)?;
            profiles.push(section);
        }
        if closed {
            let first = rails[0][0];
            let last = rails[0][rails[0].len() - 1];
            let dx = first[0] - last[0];
            let dy = first[1] - last[1];
            let dz = first[2] - last[2];
            if dx * dx + dy * dy + dz * dz > 1.0e-12 {
                return Err(Error::Invalid("closed sweep needs a closed rail"));
            }
            if profiles.len() > 1 {
                let first_profile = profiles[0].clone();
                profiles.push(first_profile);
            }
        }
        om9_kernel::creation::pipe_shell(
            &rails[0],
            rails.get(1).map(Vec::as_slice),
            &profiles,
            false,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|_| Error::Unsupported("Sweep history builder"))
    }

    #[cfg(not(feature = "occt-brep"))]
    fn build_sweep_history(
        &self,
        _feature: &'static str,
        _curves: &[Curve],
        _rail_count: usize,
        _closed: bool,
        _flip: bool,
        _slash: Option<[Point3; 2]>,
    ) -> Result<om9_core::brep::Brep> {
        Err(Error::Unsupported("Sweep History requires OCCT"))
    }

    #[cfg(not(feature = "occt-brep"))]
    fn dispatch_sweep_history(&mut self, _feature: &str) -> Result<()> {
        Err(Error::Unsupported("Sweep History requires OCCT"))
    }
    /// Merge two selected surfaces sharing a boundary.
    fn dispatch_merge(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two surface selection"));
        }
        let mut surfaces = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected surface"))?;
            let Geometry::Surface(surface) = &entity.geometry else {
                return Err(Error::Invalid("surface selection"));
            };
            surfaces.push(surface.clone());
        }
        let geometry =
            surface_edit::apply_merge(&surfaces[0], &surfaces[1], self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "merge surfaces",
            None,
        );
        self.commit_geometry(feature, surface_edit::label(feature), geometry)
    }
    /// Sweep all profiles after the first selected curve along that rail.
    fn dispatch_sweep_multi(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() < 2 {
            return Err(Error::Invalid("rail and profile selection"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        let rail = curves.remove(0);
        let surfaces = surface_edit::apply_sweep_multi(&rail, &curves, self.coordinator.limits())?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "sweep multi",
            None,
        );
        let items = surfaces
            .into_iter()
            .map(|surface| (surface_edit::label(feature), surface))
            .collect();
        self.commit_geometries(feature, surface_edit::label(feature), items)
    }
    /// Start a profile/rail command that waits for its axis.
    fn dispatch_curve_pair(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("profile and rail selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.curve_pair_pending = Some(CurvePairPending {
            feature,
            profile: curves[0].clone(),
            rail: curves[1].clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "rail revolve axis",
            None,
        );
        Ok(())
    }
    /// Start a selected-surface command that waits for one numeric distance.
    fn dispatch_surface_distance(&mut self, feature: &str) -> Result<()> {
        let feature = surface_edit::static_feature(feature);
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.surface_pending = Some(SurfacePending {
            feature,
            surface: surface.clone(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "surface distance",
            None,
        );
        Ok(())
    }
    /// Insert one authored geometry item and publish it. Shared by the interactive point
    /// commands and the immediate selected-surface edit commands (`Unroll`, `Rebuild`).
    fn commit_geometry(
        &mut self,
        feature: &'static str,
        name: &'static str,
        geometry: Geometry,
    ) -> Result<()> {
        self.commit_geometries(feature, name, vec![(name, geometry)])
    }
    /// Insert one already-built entity, preserving its extensions.
    fn commit_entity(
        &mut self,
        feature: &'static str,
        name: &'static str,
        mut entity: Entity,
    ) -> Result<()> {
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            old_scene.update_usage(Some(&entity))?,
            self.coordinator.limits(),
        )?;
        let added = entity.id;
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
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        let outcome = if receipt.published {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "finished",
            Some(outcome),
        );
        self.session = None;
        Ok(())
    }
    /// Insert one already-built entity plus its history recipe in one transaction.
    fn commit_entity_with_history(
        &mut self,
        feature: &'static str,
        name: &'static str,
        mut entity: Entity,
        command: &str,
        inputs: Vec<SourceBinding>,
    ) -> Result<()> {
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        let output = entity.id;
        let tolerance = self
            .document()
            .tolerance()
            .cloned()
            .unwrap_or_else(Tolerance::om9_template);
        let mut recipe = Recipe::new(
            self.document,
            command,
            inputs,
            vec![output],
            ReplayEnvironment {
                command_version: 1,
                solver: "om9-app".into(),
                solver_version: env!("CARGO_PKG_VERSION").into(),
                schema_version: 1,
                length_unit: Unit::Millimeter,
                frame: om9_core::geometry::Transform::identity(),
                tolerance,
            },
        );
        recipe.state = EvaluationState::Pending;
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            old_scene.update_usage(Some(&entity))?,
            self.coordinator.limits(),
        )?;
        let expected_revision = self.document().revision;
        let prepared = self.coordinator.prepare_with_held_usage(
            TransactionRequest::new(
                name,
                vec![
                    Mutation::Insert {
                        document: self.document,
                        entity,
                    },
                    Mutation::History {
                        document: self.document,
                        expected_revision,
                        recipe: Box::new(recipe),
                    },
                ],
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
            Some(output),
            self.coordinator.limits(),
        )?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.scene = Some(scene);
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        let outcome = if receipt.published {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "finished",
            Some(outcome),
        );
        self.session = None;
        Ok(())
    }
    fn commit_geometries(
        &mut self,
        feature: &'static str,
        name: &'static str,
        items: Vec<(&'static str, Geometry)>,
    ) -> Result<()> {
        if items.is_empty() {
            return Err(Error::Invalid("empty geometry batch"));
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let mut mutations = Vec::with_capacity(items.len());
        let mut added = None;
        for (label, geometry) in items {
            let mut entity = Entity::new(label, geometry, self.document().current_layer);
            if entity.attributes.layer.is_some() {
                entity.attributes.color = None;
            }
            held.add(
                old_scene.update_usage(Some(&entity))?,
                self.coordinator.limits(),
            )?;
            added = Some(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        let count = mutations.len();
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        if count > 1 {
            let receipt = self.coordinator.commit(prepared)?;
            if receipt.published {
                self.refresh_scene()?;
                self.recent.push_back(feature.into());
                while self.recent.len() > 9 {
                    self.recent.pop_front();
                }
                self.repeat = Some(feature.into());
                self.sync_published(om9_core::changes::ChangeCause::Edit);
            }
            let outcome = if receipt.published {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            };
            self.events.emit(
                crate::command_events::EventKind::Finished,
                feature,
                "finished",
                Some(outcome),
            );
            self.session = None;
            return Ok(());
        }
        let epoch = self
            .coordinator
            .epoch()
            .checked_add(1)
            .ok_or(Error::Overflow("scene publication epoch"))?;
        let scene = old_scene.update_owned(
            prepared.candidate_world().documents[&self.document].clone(),
            epoch,
            added,
            self.coordinator.limits(),
        )?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.scene = Some(scene);
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        let outcome = if receipt.published {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
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
    /// Health entries for every object in the current document.
    pub fn health_entries(&self) -> Vec<crate::utilities::HealthEntry> {
        crate::utilities::health_entries(self.document(), self.coordinator.limits())
    }

    pub fn health_summary(&self) -> String {
        crate::utilities::summary(&self.health_entries())
    }

    /// Select every invalid object and return the count.
    pub fn select_bad_objects(&mut self) -> Result<usize> {
        let entries = self.health_entries();
        let ids = crate::utilities::bad_object_ids(&entries);
        self.select_objects(&ids)?;
        Ok(ids.len())
    }

    /// Select every object with naked edges and return the count.
    pub fn select_naked_edges(&mut self) -> Result<usize> {
        let entries = self.health_entries();
        let ids = crate::utilities::naked_edge_ids(&entries);
        self.select_objects(&ids)?;
        Ok(ids.len())
    }

    /// Center the selected objects on the world origin.
    pub fn center_selected(&mut self) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("center selection"));
        }
        let mut bounds: Option<om9_core::utilities::Bounds> = None;
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("center object"))?;
            if let Some(local) = om9_core::utilities::bounds(&entity.geometry) {
                bounds = Some(match bounds {
                    Some(mut current) => {
                        for axis in 0..3 {
                            current.min[axis] = current.min[axis].min(local.min[axis]);
                            current.max[axis] = current.max[axis].max(local.max[axis]);
                        }
                        current
                    }
                    None => local,
                });
            }
        }
        let bounds = bounds.ok_or(Error::Invalid("center bounds"))?;
        let center = bounds.center();
        let delta =
            om9_core::geometry::Transform::translation([-center[0], -center[1], -center[2]]);
        let mut mutations = Vec::with_capacity(ids.len());
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("center object"))?;
            mutations.push(Mutation::Placement {
                source: self.document().stamp(id)?,
                placement: delta.compose(entity.placement)?,
            });
        }
        self.commit_utility_mutations("CenterObject", mutations)
    }

    /// Group the current selection.
    pub fn group_selection(&mut self) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.len() < 2 {
            return Err(Error::Invalid("group selection"));
        }
        let group = om9_core::model::Group::new("Group", ids);
        let mutation = Mutation::GroupCreate {
            document: self.document,
            expected_revision: self.document().revision,
            group: Box::new(group),
        };
        self.commit_utility_mutations("Group", vec![mutation])
    }

    /// Remove every group that contains a selected object.
    pub fn ungroup_selection(&mut self) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("ungroup selection"));
        }
        let mut groups = std::collections::BTreeSet::new();
        for id in &ids {
            groups.extend(self.document().groups_of(*id));
        }
        if groups.is_empty() {
            return Err(Error::Invalid("selection is not grouped"));
        }
        let mut mutations = Vec::with_capacity(groups.len());
        for group in groups {
            mutations.push(Mutation::GroupDelete {
                document: self.document,
                expected_revision: self.document().revision,
                group,
            });
        }
        self.commit_utility_mutations("Ungroup", mutations)
    }

    /// Tessellate selected NURBS surfaces into editable display meshes.
    pub fn mesh_from_nurbs_selected(&mut self) -> Result<usize> {
        let ids = self.selected_ordered();
        let mut items = Vec::new();
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("mesh surface"))?;
            if let Geometry::Surface(surface) = &entity.geometry {
                let mesh = om9_core::utilities::mesh_from_surface(
                    surface,
                    16,
                    16,
                    self.coordinator.limits(),
                )?;
                items.push((surface, mesh));
            }
        }
        if items.is_empty() {
            return Err(Error::Invalid("no surface selection"));
        }
        let count = items.len();
        let mut geometries = Vec::with_capacity(count);
        for (_, mesh) in items {
            geometries.push(("Mesh From NURBS", Geometry::Mesh(mesh)));
        }
        self.commit_geometries("MeshFromNurbs", "Mesh From NURBS", geometries)?;
        Ok(count)
    }

    /// Reduce selected meshes to roughly half their face count.
    pub fn reduce_mesh_selected(&mut self) -> Result<usize> {
        self.reduce_mesh_selected_with_angle(30.0)
    }

    /// Reduce selected meshes with a caller-provided dihedral feature angle.
    pub fn reduce_mesh_selected_with_angle(&mut self, angle_deg: f64) -> Result<usize> {
        if !angle_deg.is_finite() || !(0.0..=180.0).contains(&angle_deg) {
            return Err(Error::Invalid("feature angle"));
        }
        let ids = self.selected_ordered();
        let mut mutations = Vec::new();
        let mut count = 0usize;
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("mesh reduce"))?;
            let Geometry::Mesh(mesh) = &entity.geometry else {
                continue;
            };
            let target = (mesh.faces.len() / 2).max(1);
            let (reduced, _) =
                om9_core::mesh_repair::reduce_mesh_feature_aware(mesh, target, true, angle_deg)?;
            let mut extensions = entity.extensions.clone();
            if let Some(uv) =
                om9_core::rhino_semantics::mesh_uv_of(entity, self.coordinator.limits())?
                && uv.uvs.len() == mesh.vertices.len()
            {
                let mapped: Vec<[f64; 2]> = reduced
                    .vertices
                    .iter()
                    .map(|point| {
                        let mut best = 0usize;
                        let mut distance = f64::INFINITY;
                        for (index, old) in mesh.vertices.iter().enumerate() {
                            let dx = point[0] - old[0];
                            let dy = point[1] - old[1];
                            let dz = point[2] - old[2];
                            let candidate = dx * dx + dy * dy + dz * dz;
                            if candidate < distance {
                                distance = candidate;
                                best = index;
                            }
                        }
                        uv.uvs.get(best).copied().unwrap_or([0.0, 0.0])
                    })
                    .collect();
                let mut corner_map = vec![[0.0f64, 0.0]; mesh.vertices.len()];
                if uv.corners.len() == mesh.faces.iter().map(Vec::len).sum::<usize>() {
                    let mut offset = 0usize;
                    for face in &mesh.faces {
                        for index in face {
                            corner_map[*index as usize] = uv.corners[offset];
                            offset += 1;
                        }
                    }
                }
                let corners: Vec<[f64; 2]> = reduced
                    .faces
                    .iter()
                    .flat_map(|face| {
                        face.iter().map(|index| {
                            let point = reduced.vertices[*index as usize];
                            let mut best = 0usize;
                            let mut distance = f64::INFINITY;
                            for (old_index, old) in mesh.vertices.iter().enumerate() {
                                let dx = point[0] - old[0];
                                let dy = point[1] - old[1];
                                let dz = point[2] - old[2];
                                let candidate = dx * dx + dy * dy + dz * dz;
                                if candidate < distance {
                                    distance = candidate;
                                    best = old_index;
                                }
                            }
                            if uv.corners.is_empty() {
                                uv.uvs.get(best).copied().unwrap_or([0.0, 0.0])
                            } else {
                                corner_map[best]
                            }
                        })
                    })
                    .collect();
                let preserved = om9_core::rhino_semantics::MeshUv {
                    uvs: mapped,
                    corners,
                    material: uv.material,
                    pbr: uv.pbr,
                };
                extensions.insert(
                    om9_core::model::MESH_UV_EXTENSION_SLOT.to_string(),
                    preserved.to_extension()?,
                );
            }
            mutations.push(Mutation::Geometry {
                source: self.document().stamp(*id)?,
                geometry: Geometry::Mesh(reduced),
            });
            if extensions != entity.extensions {
                mutations.push(Mutation::Extensions {
                    source: self.document().stamp(*id)?,
                    extensions,
                });
            }
            count += 1;
        }
        if count == 0 {
            return Err(Error::Invalid("no mesh selection"));
        }
        self.commit_utility_mutations("MeshReduce", mutations)?;
        Ok(count)
    }

    fn commit_utility_mutations(
        &mut self,
        label: &'static str,
        mutations: Vec<Mutation>,
    ) -> Result<()> {
        let prepared = self
            .coordinator
            .prepare(TransactionRequest::new(label, mutations))?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }
    pub fn undo(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let before = self.document().clone();
        self.coordinator.undo()?;
        self.refresh_scene()?;
        self.record_history_change(&before, om9_core::changes::ChangeCause::Undo);
        self.sync_published(om9_core::changes::ChangeCause::Undo);
        Ok(())
    }
    /// Re-run one recorded Sweep 1/2 History recipe and update its output.
    #[cfg(feature = "occt-brep")]
    pub fn replay_history(&mut self, recipe_id: om9_core::identity::HistoryId) -> Result<()> {
        use om9_core::surface_kernel;
        let recipe = self
            .document()
            .history
            .recipes
            .get(&recipe_id)
            .cloned()
            .ok_or(Error::Missing("history recipe"))?;
        if recipe.outputs.len() != 1 {
            return Err(Error::Invalid("history recipe outputs"));
        }
        let mut curves = Vec::with_capacity(recipe.inputs.len());
        for input in &recipe.inputs {
            let entity = self
                .document()
                .entities
                .get(&input.source.object)
                .ok_or(Error::Missing("history input"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("history curve input"));
            };
            curves.push(curve.clone());
        }
        let brep = if recipe.command == "Sweep4" {
            self.build_sweep4(&curves)?
        } else {
            let rail_count = match recipe.command.as_str() {
                "Sweep1History" => 1,
                "Sweep2History" => 2,
                _ => return Err(Error::Unsupported("history replay command")),
            };
            if curves.len() <= rail_count {
                return Err(Error::Invalid("history recipe inputs"));
            }
            let limits = self.coordinator.limits();
            let mut rails: Vec<Vec<Point3>> = Vec::with_capacity(rail_count);
            for curve in &curves[..rail_count] {
                rails.push(
                    surface_kernel::sample_curve(curve, 33, limits)
                        .map_err(|_| Error::Unsupported("history replay rail"))?,
                );
            }
            let mut profiles: Vec<Vec<Point3>> = Vec::with_capacity(curves.len() - rail_count);
            for curve in &curves[rail_count..] {
                profiles.push(
                    surface_kernel::sample_curve(curve, 24, limits)
                        .map_err(|_| Error::Unsupported("history replay profile"))?,
                );
            }
            om9_kernel::creation::pipe_shell(
                &rails[0],
                rails.get(1).map(Vec::as_slice),
                &profiles,
                false,
                om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                limits,
            )
            .map_err(|_| Error::Unsupported("history replay builder"))?
        };
        let output = recipe.outputs[0];
        let source = self.document().stamp(output)?;
        let prepared = self.coordinator.prepare(TransactionRequest::new(
            "ReplayHistory",
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Brep(brep),
            }],
        ))?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.recent.push_back("ReplayHistory".into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let before = self.document().clone();
        self.coordinator.redo()?;
        self.refresh_scene()?;
        self.record_history_change(&before, om9_core::changes::ChangeCause::Redo);
        self.sync_published(om9_core::changes::ChangeCause::Redo);
        Ok(())
    }
    /// Record a host Undo/Redo batch in the plugin change feed so a following plugin
    /// command sees the last committed document change, not only plugin commands.
    fn record_history_change(
        &mut self,
        before: &om9_core::model::Document,
        cause: om9_core::changes::ChangeCause,
    ) {
        if let Some(batch) = om9_core::changes::ChangeBatch::between(before, self.document(), cause)
            .ok()
            .flatten()
        {
            self.plugin_change = Some(batch);
        }
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

/// Resample a profile to `count` points by arc length.
fn resample_profile(points: &[Point3], count: usize) -> Vec<Point3> {
    if points.len() < 2 || count < 2 {
        return points.to_vec();
    }
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    let mut total = 0.0;
    for pair in points.windows(2) {
        let dx = pair[1][0] - pair[0][0];
        let dy = pair[1][1] - pair[0][1];
        let dz = pair[1][2] - pair[0][2];
        total += dx.hypot(dy).hypot(dz);
        lengths.push(total);
    }
    if total <= 1.0e-12 {
        return points.to_vec();
    }
    let mut result = Vec::with_capacity(count);
    let mut segment = 0usize;
    for index in 0..count {
        let target = total * index as f64 / (count - 1) as f64;
        while segment + 2 < lengths.len() && lengths[segment + 1] < target {
            segment += 1;
        }
        let span = lengths[segment + 1] - lengths[segment];
        let t = if span <= 1.0e-12 {
            0.0
        } else {
            (target - lengths[segment]) / span
        };
        result.push([
            points[segment][0] + (points[segment + 1][0] - points[segment][0]) * t,
            points[segment][1] + (points[segment + 1][1] - points[segment][1]) * t,
            points[segment][2] + (points[segment + 1][2] - points[segment][2]) * t,
        ]);
    }
    result
}

/// Build the Add Slash alignment section by interpolating the two nearest
/// profiles and mapping the result onto the two picked rail points.
fn slash_alignment_section(
    profiles: &[Vec<Point3>],
    rails: &[Vec<Point3>],
    a: Point3,
    b: Point3,
) -> Result<Vec<Point3>> {
    if profiles.is_empty() {
        return Err(Error::Invalid("slash needs a profile"));
    }
    let count = 32usize;
    let slash_param0 = rails
        .first()
        .and_then(|rail| nearest_rail_parameter(rail, a))
        .unwrap_or(0.0);
    let slash_param1 = rails
        .get(1)
        .and_then(|rail| nearest_rail_parameter(rail, b))
        .unwrap_or(1.0);
    let mut candidates: Vec<(f64, Vec<Point3>)> = profiles
        .iter()
        .map(|profile| {
            let resampled = resample_profile(profile, count);
            let first = resampled[0];
            let last = resampled[resampled.len() - 1];
            let forward = (first[0] - a[0]).hypot((first[1] - a[1]).hypot(first[2] - a[2]))
                + (last[0] - b[0]).hypot((last[1] - b[1]).hypot(last[2] - b[2]));
            let reverse = (last[0] - a[0]).hypot((last[1] - a[1]).hypot(last[2] - a[2]))
                + (first[0] - b[0]).hypot((first[1] - b[1]).hypot(first[2] - b[2]));
            let oriented = if reverse < forward {
                resampled.into_iter().rev().collect()
            } else {
                resampled
            };
            let score = profile_plane(&oriented)
                .and_then(|(origin, normal)| {
                    let first_rail = rails.first()?;
                    let second_rail = rails.get(1)?;
                    let (param0, _) = rail_plane_intersection(first_rail, origin, normal)?;
                    let (param1, _) = rail_plane_intersection(second_rail, origin, normal)?;
                    Some((param0 - slash_param0).powi(2) + (param1 - slash_param1).powi(2))
                })
                .unwrap_or(forward.min(reverse));
            (score, oriented)
        })
        .collect();
    candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
    let first = candidates[0].1.clone();
    let second = if candidates.len() > 1 {
        candidates[1].1.clone()
    } else {
        first.clone()
    };
    let midpoint = |points: &[Point3]| {
        let last = points[points.len() - 1];
        [
            (points[0][0] + last[0]) / 2.0,
            (points[0][1] + last[1]) / 2.0,
            (points[0][2] + last[2]) / 2.0,
        ]
    };
    let m0 = midpoint(&first);
    let m1 = midpoint(&second);
    let slash = [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ];
    let span = [m1[0] - m0[0], m1[1] - m0[1], m1[2] - m0[2]];
    let span2 = span[0] * span[0] + span[1] * span[1] + span[2] * span[2];
    let t = if span2 <= 1.0e-12 {
        0.0
    } else {
        (((slash[0] - m0[0]) * span[0]
            + (slash[1] - m0[1]) * span[1]
            + (slash[2] - m0[2]) * span[2])
            / span2)
            .clamp(0.0, 1.0)
    };
    let mut shape: Vec<Point3> = first
        .iter()
        .zip(&second)
        .map(|(left, right)| {
            [
                left[0] + (right[0] - left[0]) * t,
                left[1] + (right[1] - left[1]) * t,
                left[2] + (right[2] - left[2]) * t,
            ]
        })
        .collect();
    let q0 = shape[0];
    let q1 = shape[shape.len() - 1];
    let from = [q1[0] - q0[0], q1[1] - q0[1], q1[2] - q0[2]];
    let to = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let from_len = from[0].hypot(from[1]).hypot(from[2]);
    let to_len = to[0].hypot(to[1]).hypot(to[2]);
    if from_len <= 1.0e-12 {
        return Ok(shape);
    }
    let scale = if to_len <= 1.0e-12 {
        1.0
    } else {
        to_len / from_len
    };
    let axis = [
        from[1] * to[2] - from[2] * to[1],
        from[2] * to[0] - from[0] * to[2],
        from[0] * to[1] - from[1] * to[0],
    ];
    let axis_len = axis[0].hypot(axis[1]).hypot(axis[2]);
    let cos = (from[0] * to[0] + from[1] * to[1] + from[2] * to[2]) / (from_len * to_len);
    let sin = axis_len / (from_len * to_len);
    let axis = if axis_len <= 1.0e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len]
    };
    for point in &mut shape {
        let v = [
            (point[0] - q0[0]) * scale,
            (point[1] - q0[1]) * scale,
            (point[2] - q0[2]) * scale,
        ];
        let cross = [
            axis[1] * v[2] - axis[2] * v[1],
            axis[2] * v[0] - axis[0] * v[2],
            axis[0] * v[1] - axis[1] * v[0],
        ];
        let dot = axis[0] * v[0] + axis[1] * v[1] + axis[2] * v[2];
        *point = [
            a[0] + v[0] * cos + cross[0] * sin + axis[0] * dot * (1.0 - cos),
            a[1] + v[1] * cos + cross[1] * sin + axis[1] * dot * (1.0 - cos),
            a[2] + v[2] * cos + cross[2] * sin + axis[2] * dot * (1.0 - cos),
        ];
    }
    Ok(shape)
}

/// Best-fit plane of a sampled profile: first non-collinear triple.
fn profile_plane(points: &[Point3]) -> Option<([f64; 3], [f64; 3])> {
    if points.len() < 3 {
        return None;
    }
    let origin = points[0];
    for i in 1..points.len() {
        let first = [
            points[i][0] - origin[0],
            points[i][1] - origin[1],
            points[i][2] - origin[2],
        ];
        for other in &points[(i + 1)..] {
            let second = [
                other[0] - origin[0],
                other[1] - origin[1],
                other[2] - origin[2],
            ];
            let normal = [
                first[1] * second[2] - first[2] * second[1],
                first[2] * second[0] - first[0] * second[2],
                first[0] * second[1] - first[1] * second[0],
            ];
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            if length > 1.0e-9 {
                return Some((
                    origin,
                    [normal[0] / length, normal[1] / length, normal[2] / length],
                ));
            }
        }
    }
    None
}

fn nearest_rail_parameter(rail: &[Point3], point: Point3) -> Option<f64> {
    if rail.len() < 2 {
        return None;
    }
    let mut best = 0usize;
    let mut distance = f64::INFINITY;
    for (index, station) in rail.iter().enumerate() {
        let dx = station[0] - point[0];
        let dy = station[1] - point[1];
        let dz = station[2] - point[2];
        let value = dx * dx + dy * dy + dz * dz;
        if value < distance {
            distance = value;
            best = index;
        }
    }
    distance
        .is_finite()
        .then_some(best as f64 / (rail.len() - 1).max(1) as f64)
}
/// Intersect one sampled rail polyline with a plane; returns the normalized
/// rail parameter and the intersection point.
fn rail_plane_intersection(
    rail: &[Point3],
    origin: [f64; 3],
    normal: [f64; 3],
) -> Option<(f64, Point3)> {
    if rail.len() < 2 {
        return None;
    }
    let distance = |point: Point3| {
        (point[0] - origin[0]) * normal[0]
            + (point[1] - origin[1]) * normal[1]
            + (point[2] - origin[2]) * normal[2]
    };
    let mut segments: Vec<(usize, Point3, Point3)> = rail
        .windows(2)
        .enumerate()
        .map(|(index, pair)| (index, pair[0], pair[1]))
        .collect();
    if rail[0][0] != rail[rail.len() - 1][0]
        || rail[0][1] != rail[rail.len() - 1][1]
        || rail[0][2] != rail[rail.len() - 1][2]
    {
        // open rail; no wrap segment
    } else {
        segments.pop();
    }
    for (index, start, end) in segments {
        let d0 = distance(start);
        let d1 = distance(end);
        let span = d1 - d0;
        if d0 * d1 <= 0.0 && span.abs() > 1.0e-12 {
            let t = (d0 / -span).clamp(0.0, 1.0);
            let point = [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
                start[2] + (end[2] - start[2]) * t,
            ];
            let parameter = (index as f64 + t) / (rail.len() - 1).max(1) as f64;
            return Some((parameter, point));
        }
    }
    None
}

/// Numerically intersect a line through `origin`/`point` with the exact NURBS
/// curve. Returns the curve fraction at the closest approach.
fn line_curve_parameter(
    curve: &Curve,
    origin: Point3,
    point: Point3,
    limits: &Limits,
) -> Result<f64> {
    let direction = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let length2 =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if length2 <= 1.0e-18 {
        return Err(Error::Invalid("line curve direction"));
    }
    #[cfg(feature = "occt-brep")]
    if let Ok(fraction) = om9_kernel::intersect::line_curve_fraction(curve, origin, point, limits) {
        return Ok(fraction);
    }
    let distance = |fraction: f64| -> Result<f64> {
        let evaluated = om9_core::evaluation::evaluate_curve(
            curve,
            om9_core::evaluation::Parameter::Fraction(fraction.clamp(0.0, 1.0)),
            0,
            om9_core::geometry::Transform::identity(),
            limits,
        )?;
        let value = *evaluated
            .first()
            .ok_or(Error::Invalid("line curve point"))?;
        let offset = [
            value[0] - origin[0],
            value[1] - origin[1],
            value[2] - origin[2],
        ];
        let t = (offset[0] * direction[0] + offset[1] * direction[1] + offset[2] * direction[2])
            / length2;
        let projected = [
            origin[0] + direction[0] * t,
            origin[1] + direction[1] * t,
            origin[2] + direction[2] * t,
        ];
        Ok((value[0] - projected[0])
            .hypot(value[1] - projected[1])
            .hypot(value[2] - projected[2]))
    };
    let samples = 256usize;
    let mut best_fraction = 0.0;
    let mut best_distance = f64::INFINITY;
    for index in 0..=samples {
        let fraction = index as f64 / samples as f64;
        let value = distance(fraction)?;
        if value < best_distance {
            best_distance = value;
            best_fraction = fraction;
        }
    }
    let mut left = (best_fraction - 1.0 / samples as f64).max(0.0);
    let mut right = (best_fraction + 1.0 / samples as f64).min(1.0);
    for _ in 0..64 {
        let first = left + (right - left) / 3.0;
        let second = right - (right - left) / 3.0;
        if distance(first)? < distance(second)? {
            right = second;
        } else {
            left = first;
        }
    }
    let fraction = (left + right) / 2.0;
    if distance(fraction)? <= distance(best_fraction)? {
        Ok(fraction)
    } else {
        Ok(best_fraction)
    }
}

/// Intersect the line from the profile-plane origin to a rail point with the
/// sampled profile polyline; returns the normalized profile parameter.
fn line_profile_parameter(profile: &[Point3], origin: Point3, point: Point3) -> Option<f64> {
    if profile.len() < 2 {
        return None;
    }
    let e = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let c = e[0] * e[0] + e[1] * e[1] + e[2] * e[2];
    if c <= 1.0e-18 {
        return None;
    }
    let mut best: Option<(f64, f64, f64)> = None;
    for (index, pair) in profile.windows(2).enumerate() {
        let start = pair[0];
        let d = [
            pair[1][0] - start[0],
            pair[1][1] - start[1],
            pair[1][2] - start[2],
        ];
        let a = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if a <= 1.0e-18 {
            continue;
        }
        let w = [
            start[0] - origin[0],
            start[1] - origin[1],
            start[2] - origin[2],
        ];
        let b = d[0] * e[0] + d[1] * e[1] + d[2] * e[2];
        let d0 = d[0] * w[0] + d[1] * w[1] + d[2] * w[2];
        let e0 = e[0] * w[0] + e[1] * w[1] + e[2] * w[2];
        let denom = a * c - b * b;
        let (segment_t, line_t) = if denom.abs() > 1.0e-18 {
            let s = (b * e0 - c * d0) / denom;
            let t = (a * e0 - b * d0) / denom;
            (s.clamp(0.0, 1.0), t)
        } else {
            (0.0, e0 / c)
        };
        if !(0.0..=1.0).contains(&line_t) {
            continue;
        }
        let candidate = [
            start[0] + d[0] * segment_t,
            start[1] + d[1] * segment_t,
            start[2] + d[2] * segment_t,
        ];
        let line_point = [
            origin[0] + e[0] * line_t,
            origin[1] + e[1] * line_t,
            origin[2] + e[2] * line_t,
        ];
        let distance = (candidate[0] - line_point[0])
            .hypot(candidate[1] - line_point[1])
            .hypot(candidate[2] - line_point[2]);
        let parameter = (index as f64 + segment_t) / (profile.len() - 1) as f64;
        let point_distance = (candidate[0] - point[0])
            .hypot(candidate[1] - point[1])
            .hypot(candidate[2] - point[2]);
        if best.is_none_or(|(best_distance, best_point, _)| {
            distance < best_distance - 1.0e-12
                || ((distance - best_distance).abs() <= 1.0e-12 && point_distance < best_point)
        }) {
            best = Some((distance, point_distance, parameter));
        }
    }
    best.filter(|(distance, _, _)| *distance <= 1.0e-4)
        .map(|(_, _, parameter)| parameter)
}

fn profile_point_at(profile: &[Point3], parameter: f64) -> Point3 {
    let scaled = parameter.clamp(0.0, 1.0) * (profile.len() - 1) as f64;
    let index = (scaled.floor() as usize).min(profile.len() - 1);
    let next = (index + 1).min(profile.len() - 1);
    let t = scaled - index as f64;
    [
        profile[index][0] + (profile[next][0] - profile[index][0]) * t,
        profile[index][1] + (profile[next][1] - profile[index][1]) * t,
        profile[index][2] + (profile[next][2] - profile[index][2]) * t,
    ]
}

fn profile_segment(profile: &[Point3], start: f64, end: f64) -> Vec<Point3> {
    let mut result = vec![profile_point_at(profile, start)];
    let last = profile.len() - 1;
    if start <= end {
        let first = (start * last as f64).ceil() as usize;
        let stop = (end * last as f64).floor() as usize;
        for point in &profile[first..=stop.min(last)] {
            result.push(*point);
        }
    } else {
        let first = (start * last as f64).ceil() as usize;
        for point in &profile[first..=last] {
            result.push(*point);
        }
        let stop = (end * last as f64).floor() as usize;
        for point in &profile[..=stop] {
            result.push(*point);
        }
    }
    result.push(profile_point_at(profile, end));
    result.dedup_by(|left, right| {
        (left[0] - right[0])
            .hypot(left[1] - right[1])
            .hypot(left[2] - right[2])
            <= 1.0e-12
    });
    result
}
fn nearest_profile_index(profile: &[Point3], point: Point3) -> Option<usize> {
    let mut best = 0usize;
    let mut distance = f64::INFINITY;
    for (index, candidate) in profile.iter().enumerate() {
        let dx = candidate[0] - point[0];
        let dy = candidate[1] - point[1];
        let dz = candidate[2] - point[2];
        let value = dx * dx + dy * dy + dz * dz;
        if value < distance {
            distance = value;
            best = index;
        }
    }
    distance.is_finite().then_some(best)
}

/// Profile/rail intersection indices and rail parameters (original rail order).
fn rail_intersections(
    curve: &Curve,
    profile: &[Point3],
    rails: &[Vec<Point3>],
    limits: &Limits,
) -> Option<([f64; 4], [f64; 4])> {
    let (origin, normal) = profile_plane(profile)?;
    let mut profile_params = [0.0f64; 4];
    let mut rail_params = [0.0f64; 4];
    for (index, rail) in rails.iter().enumerate().take(4) {
        let (parameter, point) = rail_plane_intersection(rail, origin, normal)?;
        profile_params[index] = line_curve_parameter(curve, origin, point, limits)
            .ok()
            .or_else(|| line_profile_parameter(profile, origin, point))
            .unwrap_or_else(|| {
                nearest_profile_index(profile, point)
                    .map(|index| index as f64 / (profile.len() - 1).max(1) as f64)
                    .unwrap_or(0.0)
            });
        rail_params[index] = parameter;
    }
    Some((profile_params, rail_params))
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

#[cfg(test)]
mod surface_command_tests {
    use super::*;
    use om9_core::surface_kernel;

    fn surface_controller() -> (Controller, om9_core::identity::ObjectId) {
        let limits = Limits::default();
        let surface = surface_kernel::plane(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            10.0,
            10.0,
            &limits,
        )
        .unwrap();
        let mut document = Document::new("surface command");
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        let source = entity.id;
        document.entities.insert(source, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.selection.insert(source);
        (controller, source)
    }

    #[test]
    fn rebuild_and_unroll_commit_one_surface_and_one_undo_entry() {
        let (mut controller, source) = surface_controller();
        controller
            .dispatch("OM9-SURFACE-006", InvocationOrigin::Menu)
            .expect("rebuild");
        assert_eq!(controller.document().entities.len(), 2);
        assert_eq!(controller.coordinator().undo_len(), 1);
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);

        controller
            .dispatch("OM9-SURFACE-134", InvocationOrigin::Menu)
            .expect("unroll");
        assert_eq!(controller.document().entities.len(), 2);
        let added = controller
            .document()
            .entities
            .values()
            .find(|entity| entity.id != source)
            .expect("surface");
        assert!(matches!(added.geometry, Geometry::Surface(_)));
        assert_eq!(controller.coordinator().undo_len(), 1);
    }
}
