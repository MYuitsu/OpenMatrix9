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
mod angle_isocurve;
pub mod annotation;
mod apply_crv;
mod arc;
mod arc_blend;
mod blend;
pub mod builder;
mod circle;
mod command_options;
mod command_line;
mod dispatch_geometry;
mod point_input;
mod text_input;
mod text_input_surface;
#[cfg(test)]
mod command_options_tests;
#[cfg(all(test, feature = "occt-brep"))]
mod solid_input_regressions;
mod continue_curve;
mod continue_interp;
pub mod contract;
mod control_curve;
mod corner_ops;
mod cross_section;
mod crv2view;
mod crv_ends;
mod crv_seam;
mod curve_boolean;
mod curve_soft_edit;
mod divide;
mod dup_border;
mod ellipse;
mod extend_curve;
mod extrude_surface;
mod fillet_corners;
mod fit;
pub mod gem;
pub mod gem_cluster;
pub mod channel_cutters;
pub mod gem_cutters;
pub mod gem_edits;
pub mod gem_pave;
pub mod gem_placement;
pub mod gumball_numeric;
pub mod head_commands;
mod heightfield;
mod helix;
pub mod history;
mod interp;
mod interp_on_surface;
mod intersect;
mod intersect_two_sets;
mod isocurve;
pub mod measure;
mod mesh_curve;
pub mod metal;
mod offset;
mod offset_normal;
mod offset_on_surface;
mod polygon;
mod polyline;
mod polyline_on_surface;
mod project;
mod pull;
mod rebuild;
mod rect;
mod registry;
mod refit_srf;
mod revolve;
pub mod ring_tools;
mod section;
pub mod settings_commands;
mod silhouette;
mod sketch;
mod sketch_on_surface;
mod spiral;
pub mod stress;
mod subobjects;
mod through_points;
mod to_lines;
pub mod transform;
mod tween_curves;
mod uv_curves;
mod wireframe;
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
mod solid_pending;
mod solid_geometry;
mod solid_pipe;
mod solid_boolean;
mod solid_cone;
mod solid_pyramid;
mod solid_text_stages;
mod surface_commands;
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

/// One read-only view of the active command input: owner, run generation,
/// phase and the option snapshots derived from the same owner state.
#[derive(Debug, Clone)]
pub struct CommandInputSnapshot {
    pub owner: Option<&'static str>,
    pub run: u64,
    pub phase: CmdPhase,
    pub options: Vec<crate::command_options::CommandOption>,
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
    source_edge: Option<om9_core::surface_ops::SurfaceEdge>,
    target_edge: Option<om9_core::surface_ops::SurfaceEdge>,
    /// Match continuity; Position (G0) is the documented default.
    continuity: om9_core::surface_ops::SurfaceContinuity,
}

/// A seam command waiting for an axis and tolerance.
#[derive(Debug, Clone)]
struct SeamPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    axis: Option<om9_core::surface_ops::SurfaceIso>,
    tolerance_mm: Option<f64>,
}

/// A soft-edit command waiting for its base point, move vector and documented options.
#[derive(Debug, Clone)]
struct SoftEditPending {
    feature: &'static str,
    id: om9_core::identity::ObjectId,
    surface: om9_core::geometry::NurbsSurface,
    /// `U`,`V` base point in the surface parameter domain.
    base: [Option<f64>; 2],
    /// `Dx`,`Dy`,`Dz` move vector; Matrix's "pick a point to move to" minus the base.
    delta: [Option<f64>; 3],
    u_distance: f64,
    v_distance: f64,
    copy: bool,
    fix_edges: bool,
    constraint: self::surface_edit::SoftEditConstraint,
}

impl SoftEditPending {
    fn base_complete(&self) -> bool {
        self.base.iter().all(Option::is_some)
    }

    fn delta_complete(&self) -> bool {
        self.delta.iter().all(Option::is_some)
    }

    fn base(&self) -> [f64; 2] {
        [self.base[0].expect("base u"), self.base[1].expect("base v")]
    }

    fn delta(&self) -> [f64; 3] {
        [
            self.delta[0].expect("delta x"),
            self.delta[1].expect("delta y"),
            self.delta[2].expect("delta z"),
        ]
    }
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
    edge: Option<om9_core::surface_ops::SurfaceEdge>,
    distance_mm: Option<f64>,
}

/// A split command waiting for an axis and parameter.
#[derive(Debug, Clone)]
struct SplitPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    axis: Option<om9_core::surface_ops::SurfaceIso>,
    parameter: Option<f64>,
}

/// A tangent command waiting for an edge and direction.
#[derive(Debug, Clone)]
struct TangentPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    edge: Option<om9_core::surface_ops::SurfaceEdge>,
    direction: Option<om9_core::geometry::Point3>,
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
    start: Option<f64>,
    end: Option<f64>,
    axis: Option<om9_core::surface_ops::SurfaceIso>,
}

/// A shrink command waiting for a UV sub-domain.
#[derive(Debug, Clone)]
struct ShrinkPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    u0: Option<f64>,
    u1: Option<f64>,
    v0: Option<f64>,
    v1: Option<f64>,
}

/// A variable-blend command waiting for two tangent scales.
#[derive(Debug, Clone)]
struct VarBlendPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
    start_scale: Option<f64>,
    end_scale: Option<f64>,
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
    start_radius: Option<f64>,
    end_radius: Option<f64>,
}

/// A variable chamfer command waiting for two setbacks.
#[derive(Debug, Clone)]
struct VarChamferPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
    start_setback: Option<f64>,
    end_setback: Option<f64>,
}

/// A Sweep History command waiting for Closed/Flip/Simple options.
#[derive(Debug, Clone)]
struct HistoryOptionsPending {
    feature: &'static str,
    curves: Vec<Curve>,
    ids: Vec<om9_core::identity::ObjectId>,
    rail_count: usize,
    closed: bool,
    flip: bool,
    /// `Closed` is only offered when every rail is closed and at least two profiles
    /// are selected (Matrix p183/p186); otherwise it starts `No` and is not offered.
    closed_available: bool,
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
    /// Monotonic command-start generation used to reject stale option IDs.
    command_run: u64,
    invoked_feature: Option<String>,
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
    builder: Option<om9_core::builder::BuilderSession>,
    /// Open gem tool (loader, builders, positioner); its preview owns nothing in the document.
    gem_session: Option<self::gem::GemSession>,
    /// Open F6 Gem Properties editor.
    gem_editor: Option<self::gem::GemPropertiesEditor>,
    /// Open gem-on-curve/surface placement tool (`OM9-L2-04-10`). Its preview owns nothing in
    /// the document; Apply commits one transaction.
    gem_placement: Option<self::gem_placement::GemPlacementSession>,
    /// Open halo/cluster/baguette builder (`OM9-L2-04-11`). Its preview owns nothing in the
    /// document; Apply commits one transaction.
    gem_cluster: Option<self::gem_cluster::GemClusterSession>,
    /// Open gem edit/report tool (`OM9-L2-04-13`). A mutating tool previews every stone and
    /// commits ONE transaction on Apply; the report tools never touch the document.
    gem_edits: Option<self::gem_edits::GemEditsSession>,
    /// Open pavé/azure/prong/sphere/springs builder (`OM9-L2-04-12`). Its preview owns
    /// nothing in the document; Apply commits one transaction.
    gem_pave: Option<self::gem_pave::GemPaveSession>,
    /// Open Gem Cutter / Gem Cutter Library builder (`OM9-L2-04-16`). Its preview owns nothing
    /// in the document; Enter commits one transaction (the Interactive Boolean cut plus the
    /// cutter bodies under the Preserve/Remove answer).
    gem_cutters: Option<self::gem_cutters::GemCutterSession>,
    /// Open Channel / Micro Prong / Bright Cut builder (`OM9-L2-04-17`). One staged session
    /// for the four row commands; Enter commits the Boolean cuts and the cutter bodies.
    row_cutters: Option<self::channel_cutters::RowCutterSession>,
    /// Inactive builder tabs, each keeping its own parameters while another tab is active.
    builder_saved: std::collections::BTreeMap<
        om9_core::builder::BuilderKind,
        om9_core::builder::BuilderSession,
    >,
    /// Builder styles: the built-in library plus the user's own (`OM9-L2-03-12`).
    style_library: om9_core::builder_style::StyleLibrary,
    /// Where user styles persist. `OM9_STYLE_LIBRARY` names the file; without it the user
    /// list lives for the session only.
    style_path: Option<std::path::PathBuf>,
    /// The last style-load problem, shown once in the panel instead of being swallowed.
    style_error: Option<String>,
    /// The style the open session last applied, for the browser's selected state.
    applied_style: Option<String>,
    /// Pavé styles (`OM9-L3-04-12-001`): built-ins plus the user's own snapshots.
    pave_style_library: om9_core::gem_pave::PaveStyleLibrary,
    /// Where user pavé styles persist (`OM9_PAVE_STYLE_LIBRARY`); without it they live for
    /// this session only.
    pave_style_path: Option<std::path::PathBuf>,
    /// Last pavé style problem, shown once in the builder window.
    pave_style_error: Option<String>,
    history_cache: om9_core::associative::RecomputeCache,
    /// Matrix History recording policy for recipes created from now on (`OM9-L3-03-13-001`).
    history_record: bool,
    /// Associative recompute policy (`OM9-L3-03-13-002`).
    history_update: bool,
    /// True while a rebuild owns the document; close/open refuses to race it (`-006`).
    history_rebuilding: bool,
    /// One-shot message for the UI status line after a history operation.
    history_notice: Option<String>,
    /// The command line asked for the history report window.
    history_window_requested: bool,
    fit_pending: Option<self::fit::FitPending>,
    refit_srf_pending: Option<self::refit_srf::RefitSrfPending>,
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
    /// Active read-out of the Measure menu (OM9-L2-03-20). It is read-only: the
    /// document and the Undo journal never change.
    measure_pending: Option<self::measure::MeasurePending>,
    annotation_pending: Option<self::annotation::Pending>,
    annotation_style: om9_core::annotation::Style,
    transform_pending: Option<self::transform::TransformPending>,
    /// The last published read-out, tagged with the selection it measured.
    last_measurement: Option<self::measure::MeasurementLabel>,
    /// Metal material registry and the last valid Metal Weights report (`OM9-L2-04-19`).
    metal_registry: om9_core::metal_weight::MetalMaterialRegistry,
    last_metal_report: Option<self::metal::MetalWeightReport>,
    /// Staged head/bezel command (`OM9-L2-04-14`).
    head_pending: Option<self::head_commands::HeadPending>,
    head_sequence: u64,
    /// Staged setting-parts command (`OM9-L2-04-15`) and the features it already committed.
    settings_pending: Option<self::settings_commands::SettingsPending>,
    settings_volumes: Vec<(String, f64)>,
    /// Settings features committed as cutters; they remove metal and stay out of the mass.
    pub(crate) settings_cutters: Vec<String>,
    /// Volume each committed cutter removes, so the mass read-out can report the net metal.
    pub(crate) settings_cutter_volumes: Vec<(String, f64)>,
    /// User (Bezier) profiles saved in this session; the profile library is editable here.
    pub(crate) head_profiles: Vec<om9_core::settings::head::Profile>,
    /// The last committed channel border, reused by Metal Piece's optional border input.
    pub(crate) settings_border: Option<om9_core::settings::channel_border::ChannelBorder>,
    settings_sequence: u64,
    last_settings_mass: Option<self::settings_commands::SettingsMassReport>,
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
            command_run: 0,
            invoked_feature: None,
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
            builder: None,
            gem_session: None,
            gem_editor: None,
            gem_placement: None,
            gem_cluster: None,
            gem_edits: None,
            gem_pave: None,
            gem_cutters: None,
            row_cutters: None,
            builder_saved: std::collections::BTreeMap::new(),
            style_library: {
                let mut library = om9_core::builder_style::StyleLibrary::builtin();
                if let Some(path) = std::env::var_os("OM9_STYLE_LIBRARY") {
                    // A broken user file never hides the built-in library; the error is kept
                    // for the panel to show once.
                    let _ = library.load_from(&std::path::PathBuf::from(path));
                }
                library
            },
            style_path: std::env::var_os("OM9_STYLE_LIBRARY").map(std::path::PathBuf::from),
            style_error: None,
            applied_style: None,
            pave_style_library: {
                let mut library = om9_core::gem_pave::PaveStyleLibrary::builtin();
                if let Some(path) = std::env::var_os("OM9_PAVE_STYLE_LIBRARY")
                    && let Ok(loaded) = om9_core::gem_pave::PaveStyleLibrary::load_user(
                        &std::path::PathBuf::from(path),
                    )
                {
                    library = loaded;
                }
                library
            },
            pave_style_path: std::env::var_os("OM9_PAVE_STYLE_LIBRARY")
                .map(std::path::PathBuf::from),
            pave_style_error: None,
            history_cache: om9_core::associative::RecomputeCache::default(),
            history_record: true,
            history_update: true,
            history_rebuilding: false,
            history_notice: None,
            history_window_requested: false,
            fit_pending: None,
            refit_srf_pending: None,
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
            measure_pending: None,
            annotation_pending: None,
            annotation_style: om9_core::annotation::Style::default(),
            transform_pending: None,
            last_measurement: None,
            metal_registry: om9_core::metal_weight::MetalMaterialRegistry::builtin(),
            last_metal_report: None,
            head_pending: None,
            head_sequence: 0,
            settings_pending: None,
            settings_volumes: Vec::new(),
            settings_cutters: Vec::new(),
            settings_cutter_volumes: Vec::new(),
            head_profiles: Vec::new(),
            settings_border: None,
            settings_sequence: 0,
            last_settings_mass: None,
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
    /// The ordered command events of this session, for text readers and audits.
    pub fn events(&self) -> &crate::command_events::EventLog {
        &self.events
    }
    /// The four workspace viewport cameras in stable title order, for headless rendering.
    pub fn view_cameras(
        &self,
    ) -> Vec<(
        &'static str,
        om9_scene::camera::Camera,
        om9_scene::camera::CPlane,
    )> {
        let mut views: Vec<_> = self
            .workspace
            .views
            .values()
            .map(|view| (view.id.title(), view.camera.clone(), view.cplane.clone()))
            .collect();
        views.sort_by_key(|(title, _, _)| *title);
        views
    }
    /// The active workspace view title, for headless input replay receipts.
    pub fn active_view_title(&self) -> &'static str {
        self.workspace
            .views
            .get(&self.workspace.active)
            .map(|view| view.id.title())
            .unwrap_or("")
    }
    /// Makes the named workspace view active, for headless input replay.
    pub fn activate_view(&mut self, title: &str) -> Result<()> {
        let id = self
            .workspace
            .views
            .values()
            .find(|view| view.id.title().eq_ignore_ascii_case(title))
            .map(|view| view.id)
            .ok_or(Error::Missing("workspace view"))?;
        self.workspace.active = id;
        Ok(())
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
            .or_else(|| self.solid_pending.as_ref().map(|pending| pending.feature))
            .or_else(|| self.measure_pending.as_ref().map(|pending| pending.feature))
            .or_else(|| {
                self.annotation_pending
                    .as_ref()
                    .map(|pending| pending.feature)
            })
            .or_else(|| {
                self.transform_pending
                    .as_ref()
                    .map(|pending| pending.feature)
            })
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
            || self.refit_srf_pending.is_some()
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
            || self.control_curve_pending.is_some()
            || self.continue_curve_pending.is_some()
            || self.angle_isocurve_pending.is_some()
            || self.corner_pending.is_some()
            || self.cross_section_pending.is_some()
            || self.crv_seam_pending.is_some()
            || self.curve_soft_edit_pending.is_some()
            || self.fillet_corners_pending.is_some()
            || self.offset_normal_pending.is_some()
            || self.offset_on_surface_pending.is_some()
            || self.pull_pending.is_some()
            || self.extrude_surface_pending.is_some()
            || self.heightfield_pending.is_some()
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
            || self.measure_pending.is_some()
            || self.annotation_pending.is_some()
            || self.transform_pending.is_some()
            || self.builder.is_some()
            || self.head_pending.is_some()
            || self.settings_pending.is_some()
            || self.gem_session.is_some()
            || self.gem_editor.is_some()
            || self.gem_placement.is_some()
            || self.gem_cluster.is_some()
            || self.gem_edits.is_some()
            || self.gem_pave.is_some()
            || self.gem_cutters.is_some()
            || self.row_cutters.is_some()
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
    /// Feature that started the current live command, without falling back to repeat.
    pub fn running_feature(&self) -> Option<&str> {
        self.is_running()
            .then_some(self.invoked_feature.as_deref())
            .flatten()
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
        if self::gem::handles(feature) {
            return Ok(());
        }
        // The placement family (`OM9-L2-04-10`) is a normal command; its selection is
        // validated when the tool opens, so availability never depends on the current pick.
        if self::gem_placement::handles(feature)
            || self::gem_placement::command_name(feature).is_some()
        {
            return Ok(());
        }
        // The halo/cluster/baguette family (`OM9-L2-04-11`) validates its selection when it
        // opens, exactly like the placement family.
        if self::gem_cluster::handles(feature) || self::gem_cluster::command_name(feature).is_some()
        {
            return Ok(());
        }
        // The gem edit/report family (`OM9-L2-04-13`) validates its selection when it opens.
        if self::gem_edits::handles(feature) || self::gem_edits::command_name(feature).is_some() {
            return Ok(());
        }
        // The pavé/azure/prong family (`OM9-L2-04-12`) validates its selection when it opens.
        if self::gem_pave::PaveTool::handles(feature)
            || self::gem_pave::PaveTool::command_name(feature).is_some()
        {
            return Ok(());
        }
        // The cutter family (`OM9-L2-04-16`) validates its gem selection when it opens.
        if self::gem_cutters::CutterTool::handles(feature)
            || self::gem_cutters::CutterTool::command_name(feature).is_some()
        {
            return Ok(());
        }
        if self::channel_cutters::RowTool::handles(feature)
            || self::channel_cutters::RowTool::command_name(feature).is_some()
        {
            return Ok(());
        }
        if self::annotation::handles(feature) {
            return Ok(());
        }
        if self::metal::handles_command(feature) {
            return Ok(());
        }
        if self::head_commands::handles_command(feature) {
            return Ok(());
        }
        if self::settings_commands::handles_command(feature) {
            return Ok(());
        }
        if self::ring_tools::handles_command(feature) {
            return Ok(());
        }
        if crate::file_lifecycle::FileKind::from_feature(feature).is_some() {
            return Ok(());
        }
        if let Some((target, _)) = solid::mode_command(feature) {
            return Self::availability(target);
        }
        if matches!(feature, "SolidBooleanSplit" | "SolidCreateSolid" | "SolidExtractSurfaceCopy" | "SolidPipeChain" | "SolidGVDMultiPipe" | "OM9-SOLID-021") {
            return if cfg!(feature = "occt-brep") { Ok(()) } else { Err(Error::Unsupported("Solid command requires OCCT")) };
        }
        if solid::handles(feature) {
            if !cfg!(feature = "occt-brep") { return Err(Error::Unsupported("Solid command requires OCCT")); }
            return crate::solid_menu::by_feature(feature)
                .filter(|binding| binding.enabled())
                .map(|_| ())
                .ok_or(Error::Unsupported(
                    crate::solid_menu::by_feature(feature)
                        .map_or("solid command is not catalogued", |binding| binding.reason),
                ));
        }
        if surface_edit::handles_command(feature) {
            return Ok(());
        }
        if self::transform::handles(feature) {
            return Ok(());
        }
        if let Some(reason) = crate::transform_menu::unavailable_reason(feature) {
            return Err(Error::Unsupported(reason));
        }
        match feature {
            LINE
            | POINT
            | self::circle::CIRCLE
            | self::rect::RECTANGLE
            | self::polyline::POLYLINE
            | self::interp::INTERP
            | self::ellipse::ELLIPSE
            | self::arc::ARC
            | self::arc::ARC_START
            | self::rebuild::REBUILD
            | self::fit::FIT
            | self::refit_srf::REFIT
            | self::offset::OFFSET
            | self::project::PROJECT
            | self::blend::BLEND
            | self::blend::BLEND_CRV
            | self::blend::MATCH
            | self::isocurve::ISOCURVE
            | self::pull::PULL
            | self::interp_on_surface::INTERP_ON_SURFACE
            | self::polyline_on_surface::POLYLINE_ON_SURFACE
            | self::sketch_on_surface::SKETCH_ON_SURFACE
            | self::intersect::INTERSECT
            | self::intersect_two_sets::INTERSECT_TWO_SETS
            | self::crv2view::CRV2VIEW
            | self::crv2view::CRV2VIEW_HISTORY
            | self::through_points::THROUGH_POINTS
            | self::polygon::POLYGON
            | self::helix::HELIX
            | self::spiral::SPIRAL
            | self::divide::DIVIDE
            | self::crv_ends::CRV_START
            | self::crv_ends::CRV_END
            | self::dup_border::DUP_BORDER
            | self::to_lines::TO_LINES
            | self::extend_curve::EXTEND
            | self::tween_curves::TWEEN_CURVES
            | self::continue_interp::CONTINUE_INTERP
            | self::arc_blend::ARC_BLEND
            | self::wireframe::DUP_EDGE
            | self::wireframe::EXTRACT_WIRE
            | self::section::SECTION
            | self::sketch::SKETCH
            | self::revolve::REVOLVE
            | self::curve_boolean::CURVE_BOOLEAN
            | self::cross_section::CROSS_SECTION
            | self::fillet_corners::FILLET_CORNERS
            | self::silhouette::SILHOUETTE
            | self::curve_soft_edit::CURVE_SOFT_EDIT
            | self::angle_isocurve::ANGLE_ISOCURVE
            | self::extrude_surface::EXTRUDE
            | self::heightfield::HEIGHTFIELD
            | self::mesh_curve::MESH_CURVE
            | self::uv_curves::CREATE_UV
            | self::apply_crv::APPLY_CRV
            | self::offset_on_surface::OFFSET_ON_SURFACE
            | self::offset_normal::OFFSET_NORMAL
            | self::corner_ops::FILLET
            | self::corner_ops::CHAMFER
            | self::crv_seam::CRV_SEAM
            | self::control_curve::CONTROL_CURVE
            | self::continue_curve::CONTINUE_CURVE
            | "MeshStress"
            | self::measure::DISTANCE
            | self::measure::ANGLE
            | self::measure::LENGTH
            | self::measure::RADIUS
            | self::measure::BOUNDING_BOX
            | self::measure::OBJECT_SUMMARY
            | self::measure::AREA
            | self::measure::VOLUME
            | ZOOM_EXTENTS
            | ZOOM_SELECTED
            | VIEW_HOME
            | VIEW_CENTER => Ok(()),
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
        if !self.is_running() {
            self.invoked_feature = None;
        }
        self.command_run = self.command_run.wrapping_add(1);
        // A new command invalidates the previous read-out: the label shown in the
        // Command Line always belongs to the run the user just started
        // (OM9-L3-03-20-005).
        self.last_measurement = None;
        if let Some(result) = self.dispatch_history_feature(feature) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some((target, mode)) = solid::mode_command(feature) {
            if self.is_running() {
                return Err(Error::Busy);
            }
            Self::availability(target)?;
            self.dispatch_solid(target)?;
            if let Some(result) = self.submit_solid_text(&format!("Mode={mode}")) {
                result?;
            }
            return Ok(DispatchOutcome::Command);
        }
        if let Some(kind) = crate::file_lifecycle::FileKind::from_feature(feature) {
            return self.begin_file_operation(kind).map(DispatchOutcome::File);
        }
        let result = self.dispatch_geometry(feature, origin);
        if result.is_ok() {
            self.invoked_feature = self.is_running().then(|| feature.to_owned());
        }
        result.map(|_| DispatchOutcome::Command)
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
        match &self.session {
            None => CmdPhase::Idle,
            Some(session) if session.start.is_some() => CmdPhase::AwaitEnd,
            Some(_) => CmdPhase::AwaitPoint,
        }
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
        if self.solid_face_needs_pick() || self.solid_pending.as_ref().is_some_and(|p| p.feature == "OM9-SOLID-029" || p.pipe_chain) || self.solid_fillet_needs_edge() || self.solid_fillet_preview().is_some() {
            self.clear_subselection();
        }
        self.builder_cancel();
        self.head_pending = None;
        self.settings_pending = None;
        self.gem_cancel();
        self.gem_placement_cancel();
        self.gem_cluster_cancel();
        self.gem_edits_cancel();
        self.gem_pave_cancel();
        self.gem_cutter_cancel();
        self.row_cutter_cancel();
        self.gem_editor = None;
        self.solid_pending = None;
        self.circle_pending = None;
        self.rect_pending = None;
        self.polyline_pending = None;
        self.interp_pending = None;
        self.ellipse_pending = None;
        self.arc_pending = None;
        self.rebuild_pending = None;
        self.fit_pending = None;
        self.refit_srf_pending = None;
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
        if let Some(pending) = self.measure_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.annotation_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.transform_pending.take() {
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
        if !self.is_running() {
            self.invoked_feature = None;
        }
        if text.len() > 4096 {
            return Err(Error::Budget("command input"));
        }
        // An open builder owns the command line: a field, Apply, Reset or Cancel.
        if let Some(result) = self.builder_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open head/bezel builder owns the command line too (`OM9-L2-04-14`).
        if let Some(result) = self.submit_head_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open gem tool owns the command line: an option, Apply or Cancel.
        if let Some(result) = self.gem_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open placement tool owns the command line the same way (`OM9-L2-04-10`).
        if let Some(result) = self.gem_placement_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open halo/cluster/baguette builder owns the command line too (`OM9-L2-04-11`).
        if let Some(result) = self.gem_cluster_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open gem edit/report tool owns the command line too (`OM9-L2-04-13`).
        if let Some(result) = self.gem_edits_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open pavé/azure/prong builder owns the command line too (`OM9-L2-04-12`).
        if let Some(result) = self.gem_pave_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open Gem Cutter / Library builder owns the command line too (`OM9-L2-04-16`).
        if let Some(result) = self.gem_cutter_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // The Channel / Micro Prong / Bright Cut builders own it too (`OM9-L2-04-17`).
        if let Some(result) = self.row_cutter_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // Matrix History switches/report live on the same command line (`-006`).
        if let Some(result) = self.submit_history_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(command) = self::metal::parse_command(text)? {
            return self
                .calculate_metal_weights(command.material.as_deref())
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(material) = self::settings_commands::parse_mass_command(text)? {
            return self
                .calculate_settings_mass(material.as_deref())
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(feature) = self::ring_tools::command_name(text) {
            return self
                .dispatch_ring_tool(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // Typed placement commands (`gvGemOnCrv`, `gvGemFlow`, ...) reach the same handler as
        // the palette icons (`OM9-L2-04-10`).
        if let Some(feature) = self::gem_placement::command_name(text) {
            return self
                .dispatch_gem_placement(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // Typed halo/cluster/baguette commands (`gvHaloBuilder`, `gvGemsBetween2Crvs`, ...)
        // reach the same handler as the palette icons (`OM9-L2-04-11`).
        if let Some(feature) = self::gem_cluster::command_name(text) {
            return self
                .dispatch_gem_cluster(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // Typed pavé/azure/prong commands (`gvPaveDialog`, `gvPaveSphere`, ...).
        if let Some(feature) = self::gem_pave::PaveTool::command_name(text) {
            return self
                .dispatch_gem_pave(feature)
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(feature) = self::gem::command_name(text) {
            return self.dispatch_gem(feature).map(|_| DispatchOutcome::Command);
        }
        // Typed cutter commands (`gvAzureCutter`, `gvGemCutterLibrary`) reach the same handler
        // as the palette icons (`OM9-L2-04-16`).
        if let Some(feature) = self::gem_cutters::CutterTool::command_name(text) {
            return self
                .dispatch_gem_cutter(feature)
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(feature) = self::channel_cutters::RowTool::command_name(text) {
            return self
                .dispatch_row_cutter(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // The three builder command names start their mode from the command line too.
        if let Some(kind) = self::builder::builder_command(text) {
            return self.begin_builder(kind).map(|_| DispatchOutcome::Command);
        }
        let canonical = self.canonical_option_assignment(text);
        let text = canonical.as_deref().unwrap_or(text);
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
        if let Some(result) = self.submit_settings_text(text.trim()) {
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
        if let Some(result) = self.submit_refit_srf_text(text.trim()) {
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
        recipe.policy.record = self.history_record;
        recipe.policy.update = self.history_update;
        let record = self.history_record;
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
        let mut mutations = vec![Mutation::Insert {
            document: self.document,
            entity,
        }];
        if record {
            mutations.push(Mutation::History {
                document: self.document,
                expected_revision,
                recipe: Box::new(recipe),
                published: vec![],
            });
        }
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
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
        if receipt.published || !record {
            self.scene = Some(scene);
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            if record {
                self.sync_published(om9_core::changes::ChangeCause::Edit);
            }
        }
        let outcome = if receipt.published || !record {
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
        let geometry = if recipe.command == "Crv2ViewHistory" {
            if curves.len() != 2 {
                return Err(Error::Invalid("history recipe inputs"));
            }
            let curve = self::crv2view::build_from_two_views(
                &curves[0],
                &curves[1],
                self.coordinator.limits(),
            )?;
            Geometry::Curve(curve)
        } else if recipe.command == "Sweep4" {
            Geometry::Brep(self.build_sweep4(&curves)?)
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
            Geometry::Brep(
                om9_kernel::creation::pipe_shell(
                    &rails[0],
                    rails.get(1).map(Vec::as_slice),
                    &profiles,
                    false,
                    om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                    limits,
                )
                .map_err(|_| Error::Unsupported("history replay builder"))?,
            )
        };
        let output = recipe.outputs[0];
        let source = self.document().stamp(output)?;
        let prepared = self.coordinator.prepare(TransactionRequest::new(
            "ReplayHistory",
            vec![Mutation::Geometry { source, geometry }],
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
fn parse_soft_value(text: &str) -> Result<f64> {
    let value: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("soft edit input"))?;
    if !value.is_finite() {
        return Err(Error::Invalid("soft edit input"));
    }
    Ok(value)
}
fn parse_soft_distance(text: &str) -> Result<f64> {
    let value = parse_soft_value(text)?;
    if value <= 0.0 {
        return Err(Error::Invalid("soft edit distance"));
    }
    Ok(value)
}
fn parse_soft_flag(text: &str) -> Result<bool> {
    match text.trim().to_ascii_lowercase().as_str() {
        "yes" | "y" | "true" | "on" => Ok(true),
        "no" | "n" | "false" | "off" => Ok(false),
        _ => Err(Error::Invalid("soft edit flag")),
    }
}
fn parse_shrink_value(text: &str) -> Result<f64> {
    let value: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("shrink domain"))?;
    if !value.is_finite() {
        return Err(Error::Invalid("shrink domain"));
    }
    Ok(value)
}
fn parse_offset_distance(text: &str) -> Result<f64> {
    let distance: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("variable offset distance"))?;
    if !distance.is_finite() {
        return Err(Error::Invalid("variable offset distance"));
    }
    Ok(distance)
}
fn parse_blend_scale(text: &str) -> Result<f64> {
    let scale: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("variable blend scale"))?;
    if !scale.is_finite() {
        return Err(Error::Invalid("variable blend scale"));
    }
    Ok(scale)
}
fn parse_chamfer_setback(text: &str) -> Result<f64> {
    let setback: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("chamfer setbacks"))?;
    if !setback.is_finite() || setback <= 0.0 {
        return Err(Error::Invalid("chamfer setbacks"));
    }
    Ok(setback)
}
fn parse_fillet_radius(text: &str) -> Result<f64> {
    let radius: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("fillet radius"))?;
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Error::Invalid("fillet radius"));
    }
    Ok(radius)
}
fn parse_tangent_direction(text: &str) -> Result<[f64; 3]> {
    let values: Vec<f64> = text
        .split(',')
        .map(|value| value.trim().parse::<f64>())
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| Error::Invalid("surface tangent direction"))?;
    if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Invalid("surface tangent direction"));
    }
    Ok([values[0], values[1], values[2]])
}
fn parse_split_parameter(text: &str) -> Result<f64> {
    let parameter: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("surface split parameter"))?;
    if !parameter.is_finite() || parameter <= 0.0 || parameter >= 1.0 {
        return Err(Error::Invalid("surface split parameter"));
    }
    Ok(parameter)
}
fn parse_extend_distance(text: &str) -> Result<f64> {
    let distance: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("surface extend distance"))?;
    if !distance.is_finite() {
        return Err(Error::Invalid("surface extend distance"));
    }
    Ok(distance)
}
fn parse_seam_tolerance(text: &str) -> Result<f64> {
    let tolerance: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("surface seam tolerance"))?;
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(Error::Invalid("surface seam tolerance"));
    }
    Ok(tolerance)
}
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
