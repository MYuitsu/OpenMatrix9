// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    dock_layout::{self, DockLayout, DockSide, FloatRect, PanelId, Regions, SectionId},
    session::{Controller, InvocationOrigin, LINE, POINT},
    status_surface::{StatusKind, StatusSurface},
    transcript::Transcript,
    workbench_regions::{
        CommandInputHost, RegionError, RegionPlan, RegionRect, WindowSize, compute_regions_for,
        tile_views,
    },
};
use eframe::egui::{
    self, Color32, Key, Modifiers, Rect, Sense, Stroke, TextBuffer, TextEdit, Vec2,
    text::{CCursor, CCursorRange, CharIndex},
};
use om9_core::{Error, Result, budget::Limits, geometry::Point3, model::Document};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::PickIntent,
    workspace::ViewId,
};
mod chrome;
mod command;
mod layer_audit;
mod layer_dialog_audit;
mod layers;
mod panel_audit;
pub use layers::LayerControl;
mod file_audit;
mod file_success_audit;
mod files;
mod layer_dialogs;
mod startup_audit;
pub use files::FileControl;
pub use layer_dialogs::LayerDialogControl;
mod occurrence_audit;
mod rotation_feedback;
mod selection_audit;
mod viewport_audit;
mod viewport_menu;
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};
impl TextBuffer for Transcript {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.text()
    }
    fn insert_text(&mut self, text: &str, index: CharIndex) -> usize {
        self.insert(text, index.0)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<CharIndex>) {
        self.delete(range.start.0..range.end.0);
    }
    fn clear(&mut self) {
        self.set_draft("");
    }
    fn replace_with(&mut self, text: &str) {
        let prefix_bytes = self
            .text()
            .char_indices()
            .nth(self.protected_chars())
            .map_or(self.text().len(), |(i, _)| i);
        if let Some(draft) = text.strip_prefix(&self.text()[..prefix_bytes]) {
            let draft = draft.to_owned();
            self.set_draft(&draft);
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub command: Option<String>,
    pub module: String,
    /// Optional command bound to the icon's secondary click. Missing means the
    /// slot is inert; it must never repeat the primary action.
    #[serde(default)]
    pub right_click: Option<String>,
}
/// Adapter for the `CommandInputHost` seam toward `OM9-L2-01-02`. The CMD frame
/// (`MatrixApp::transcript` plus `command_ui`) is built into every frame, so it
/// is always attached. This says nothing about command dispatch or the Undo
/// journal, which that module does not provide yet. BLOCKED_DEPENDENCY: when
/// `OM9-L2-01-02` lands, replace this constant-true adapter with the real CMD host query.
struct AttachedCmdFrame;
impl CommandInputHost for AttachedCmdFrame {
    fn command_input_ready(&self) -> bool {
        true
    }
}
/// Which menu column `icon` is drawing into, so only column icons are recorded.
#[derive(Clone, Copy, PartialEq, Eq)]
enum IconColumn {
    None,
    Left,
    Right,
}
/// Audit-only layout mode (`OM9_UI_AUDIT_LAYOUT_MODE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditLayoutMode {
    /// Nothing docked right: the Matrix9 parity layout, used for every parity capture.
    Default,
    /// `RIGHT_DOCK_FIXTURE` docked right, as a user would after dragging it there.
    UserDockedRight,
}
impl AuditLayoutMode {
    pub fn key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::UserDockedRight => "user-docked-right",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Matrix-parity capture",
            Self::UserDockedRight => {
                "user-initiated OM9-extension layout, NOT a Matrix9 parity capture"
            }
        }
    }
}
/// Parse `OM9_UI_AUDIT_LAYOUT_MODE`: unset or `default`, or `user-docked-right`.
/// Anything else is an error, never a silent default.
pub fn parse_audit_layout_mode(
    value: Option<&str>,
) -> std::result::Result<AuditLayoutMode, &'static str> {
    match value {
        None | Some("default") => Ok(AuditLayoutMode::Default),
        Some("user-docked-right") => Ok(AuditLayoutMode::UserDockedRight),
        Some(_) => Err("OM9_UI_AUDIT_LAYOUT_MODE must be default or user-docked-right"),
    }
}
/// Parse `OM9_UI_AUDIT_RAIL_WIDTH`: finite and inside the rail's own bounds.
pub fn parse_audit_rail_width(text: &str) -> Option<f32> {
    let width = text.trim().parse::<f32>().ok()?;
    (width.is_finite()
        && (dock_layout::LEFT_RAIL_MIN..=dock_layout::LEFT_RAIL_MAX).contains(&width))
    .then_some(width)
}
/// The scripted menu-drag scenarios of `OM9_UI_AUDIT_PANELS` (L3-003).
pub const PANEL_AUDIT_SCENARIOS: [&str; 7] = [
    "reorder",
    "float",
    "dock-right",
    "escape",
    "roll-up",
    "float-drop",
    "readback-recover",
];
/// Parse `OM9_UI_AUDIT_PANELS`: unset, or one of the documented scenarios.
pub fn parse_audit_panels(text: &str) -> Option<String> {
    let scenario = text.trim().to_ascii_lowercase();
    PANEL_AUDIT_SCENARIOS
        .contains(&scenario.as_str())
        .then_some(scenario)
}
fn panel_audit_scenario() -> Option<String> {
    // An unusable value never reaches here: `launch_with` reports it before a
    // window exists, exactly like OM9_UI_AUDIT_LAYOUT_MODE.
    std::env::var("OM9_UI_AUDIT_PANELS")
        .ok()
        .and_then(|text| parse_audit_panels(&text))
}
pub struct MatrixApp {
    pub controller: Controller,
    pub transcript: Transcript,
    features: Vec<Feature>,
    focus_cmd: bool,
    cmd_rect: Option<Rect>,
    cmd_history_rect: Option<Rect>,
    /// Screen rectangles of the clickable command option tokens, for tests and hit reports.
    cmd_token_rects: std::collections::BTreeMap<String, Rect>,
    cmd_horizontal_offset: f32,
    cmd_ime_composing: bool,
    pub view_rects: BTreeMap<ViewId, Rect>,
    advanced_layers: bool,
    layer_rects: BTreeMap<(u8, LayerControl), Rect>,
    layer_action_rects: BTreeMap<&'static str, Rect>,
    layer_dialog_rects: BTreeMap<LayerDialogControl, Rect>,
    layer_dialog_focused: Option<crate::layer_dialog::LayerDialogToken>,
    layer_dialog_input_token: Option<crate::layer_dialog::LayerDialogToken>,
    layer_color_inputs: [String; 3],
    layer_dialog_error: String,
    layer_dialog_ime_composing: bool,
    file_controls: BTreeMap<FileControl, Rect>,
    file_toolbar_rects: BTreeMap<crate::file_lifecycle::FileKind, Rect>,
    file_menu_rect: Option<Rect>,
    file_menu_items: BTreeMap<crate::file_lifecycle::FileKind, Rect>,
    file_picker_request: Option<crate::file_lifecycle::FileToken>,
    file_picker_attempted: Option<crate::file_lifecycle::FileToken>,
    file_prompt_token: Option<crate::file_lifecycle::FileToken>,
    file_error: String,
    /// Pending Rhino 3DM picker: `Some(true)` export, `Some(false)` import.
    three_dm_request: Option<bool>,
    file_ime_composing: bool,
    file_f6_rects: BTreeMap<crate::file_lifecycle::FileKind, Rect>,
    /// Window title owned by this document's path/name and dirty marker.
    pub window_title: String,
    /// F4 jewelry datum overlay (world 0,0,0 origin and ring centre).
    world_datum_visible: bool,
    current_ui_context: Option<egui::Context>,
    hover: Option<Point3>,
    /// Rubber-band selection: where the drag started, in screen points.
    band_start: Option<egui::Pos2>,
    band_view: Option<ViewId>,
    /// Selection frame per viewport, keyed by (selection revision, camera revision, size): projecting every
    /// selected vertex each frame is wasted work while nothing changes.
    selection_frames: std::collections::BTreeMap<ViewId, SelectionFrame>,
    /// Objects under the last click when more than one was there: the disambiguation popup lists them.
    ambiguous: Vec<om9_core::identity::ObjectId>,
    ambiguous_pos: Option<egui::Pos2>,
    ambiguous_epoch: u64,
    /// The subobject picked with Ctrl+Shift+click, and which kind that gesture looks for.
    sub_object: Option<crate::session::SelectedSubobject>,
    sub_object_kind: om9_scene::scene::PickKind,
    /// Object under the pointer, and where/when it was computed, so hovering does not pick every frame.
    hover_object: Option<om9_core::identity::ObjectId>,
    hover_picked_at: Option<(egui::Pos2, u64, u64)>,
    pub audit: Option<UiAudit>,
    passes: u64,
    adapter: String,
    pub f6_open: bool,
    pub f6_pinned: bool,
    pub sidebar_rect: Option<Rect>,
    pub toolbar_rect: Option<Rect>,
    icons: crate::icons::Icons,
    main_category: usize,
    main_category_rects: BTreeMap<usize, Rect>,
    toolbar_tab: usize,
    style_ready: bool,
    /// Live rectangles of the Matrix menu title bars, keyed by section key.
    pub section_rects: BTreeMap<&'static str, Rect>,
    /// The menu currently being dragged, with the last finite pointer position
    /// (L3-003). Menu placement has exactly one mutation path:
    /// [`panel_audit::MatrixApp::apply_panel_op`].
    section_drag: Option<panel_audit::SectionDrag>,
    /// The preview the user sees while dragging a title bar.
    pub drop_preview: Option<panel_audit::DropPreview>,
    /// The work area of the current frame; `None` while the window is unusable.
    pub work_area: Option<crate::panel_ops::WorkArea>,
    /// The last placement operation, for the audit receipt.
    pub last_panel_op: Option<panel_audit::PanelOpRecord>,
    /// Menus whose closed-bar controls rendered this frame.
    pub closed_bar_controls: std::collections::BTreeSet<&'static str>,
    /// A floated menu needs its rectangle read back into the work area.
    float_resync: bool,
    /// Title bars rendered this frame, with the column they belong to.
    panel_bars: Vec<(SectionId, crate::panel_ops::ColumnSide, Rect)>,
    /// Bottom of the left column content this frame (drop indicator end).
    left_content_bottom: f32,
    /// Bottom of the right column content this frame.
    right_content_bottom: f32,
    sidebar_scroll: f32,
    view_menu_capture: Option<(egui::Id, crate::viewport_menu::ViewMenuCapture)>,
    viewport_menu_owns_frame: bool,
    viewport_menu_pointer_guard: bool,
    viewport_title_arrows: BTreeMap<ViewId, Rect>,
    /// Matrix dock layout owned by L2-01-01 (panels, sizes, visibility, floats).
    pub dock: DockLayout,
    dock_path: Option<PathBuf>,
    dock_dirty: bool,
    dock_window: [f32; 2],
    /// Layout warnings that must stay visible to the user (unknown version,
    /// removed panel id, panel recovered from outside the window).
    pub dock_diagnostics: Vec<String>,
    /// Always-visible status/error line (L3-007).
    /// GPU viewport surface, created on first use from eframe's render state.
    /// One GPU target per viewport, so all four render their own camera instead of sharing pixels.
    gpu_viewports: std::collections::BTreeMap<ViewId, crate::gpu_viewport::GpuViewport>,
    gpu_state: Option<eframe::egui_wgpu::RenderState>,
    /// Which gumball handles are shown when no modifier is held.
    gumball_mode: om9_scene::gumball_view::Mode,
    /// Gumball gesture in flight: the drag session, its handle and the last pointer position.
    gumball_session: Option<om9_core::gumball::Session>,
    gumball_handle: Option<om9_core::gumball::Handle>,
    gumball_last: Option<[f64; 2]>,
    gumball_basis: Option<GumballBasis>,
    gpu_drawn_views: std::collections::BTreeSet<ViewId>,
    pub status: StatusSurface,
    pub regions: Regions,
    /// Last validated workbench region plan (L3-002). Kept when a later frame
    /// cannot produce one, so the shell never falls back to an unchecked layout.
    pub region_plan: Option<RegionPlan>,
    /// Why the current frame has no fresh plan; `None` when the plan is current.
    pub region_error: Option<RegionError>,
    /// Status text this module raised, so only it clears it again.
    region_status: Option<String>,
    /// Minimum inner size last sent to the window (logical points).
    min_window_sent: Option<[f32; 2]>,
    /// Monitor size (logical points) the last minimum was derived from.
    min_window_monitor: Option<[f32; 2]>,
    /// Where `min_window_monitor` came from: `viewport`, `audit-simulated`, `unknown`.
    min_window_source: &'static str,
    /// Audit-only simulated monitor in physical pixels (`OM9_UI_AUDIT_MONITOR`).
    audit_monitor_px: Option<[f32; 2]>,
    pub right_dock_rect: Option<Rect>,
    /// OUTER width of the left rail panel last frame (frame margins included).
    pub left_rail_outer_px: f32,
    /// OUTER width of the right dock panel last frame, 0 while it is hidden.
    pub right_dock_outer_px: f32,
    /// Icon box edge the sidebar/right-dock icon calls use while a column is
    /// being drawn; reset to ICON_BASE_EDGE afterwards so toolbars stay at 22.
    column_icon_edge: f32,
    icon_column: IconColumn,
    /// Edge used in the left column last frame (`rail_icon_edge` of its width).
    pub left_icon_edge: f32,
    /// Rectangles of the icons drawn in the left column last frame.
    pub rail_icon_rects: Vec<Rect>,
    /// Rectangles of the icons drawn in the right dock last frame.
    pub right_dock_icon_rects: Vec<Rect>,
    /// Size of the scrollable left stack content last frame.
    pub sidebar_content_size: Option<Vec2>,
    audit_layout_mode: AuditLayoutMode,
    audit_rail_width: Option<f32>,
    pub builder_slot_rect: Option<Rect>,
    pub status_rect: Option<Rect>,
    /// Rect of the status message label, recorded by `status_bar_ui`.
    pub status_message_rect: Option<Rect>,
    /// Rect of the dock summary label, recorded by `status_bar_ui`.
    pub status_summary_rect: Option<Rect>,
    /// Rect of the Dismiss button (only while a failure is on screen).
    pub status_dismiss_rect: Option<Rect>,
    pub dock_panel_rects: BTreeMap<&'static str, Rect>,
}

struct GumballBasis {
    view: ViewId,
    camera: Camera,
    viewport: Viewport,
    start: [f64; 2],
    epoch: u64,
    selection: u64,
    alt_held: bool,
}
pub struct UiAudit {
    pub directory: PathBuf,
    step: usize,
    pub checks: Vec<String>,
    awaiting_screenshot: bool,
    receipt_written: bool,
    hold: bool,
    started: std::time::Instant,
    viewport_menu: bool,
    selection: bool,
    occurrences: bool,
    selection_points: Option<[[f64; 2]; 2]>,
    selection_report: Vec<serde_json::Value>,
    layers: bool,
    layer_dialogs: bool,
    native_focus_wait_started: Option<std::time::Instant>,
    files: bool,
    file_success: bool,
    file_success_wait_started: Option<std::time::Instant>,
    file_success_document: Option<om9_core::identity::DocumentId>,
    file_success_original_sha: Option<String>,
    file_success_seed_sha: Option<String>,
    startup_file: bool,
    file_close: bool,
    file_close_post_pending: bool,
    file_close_events: usize,
    file_picker_receipts: Vec<serde_json::Value>,
    dialog_capture_ready_at: Option<std::time::Instant>,
    dialog_capture_pending: Option<&'static str>,
    dialog_capture_step: Option<usize>,
    dialog_capture_metrics: Vec<serde_json::Value>,
    layer_current: Option<om9_core::identity::LayerId>,
    layer_clicks: Vec<serde_json::Value>,
    /// Scripted menu-drag scenario (`OM9_UI_AUDIT_PANELS`): reorder, float,
    /// dock-right or escape. The panel operations are replayed in a real window.
    panels: Option<String>,
    panel_checks: Vec<serde_json::Value>,
    panel_drag_observed: Option<serde_json::Value>,
    /// Where the scripted drag goes, fixed on the first frame that can see the
    /// live rectangles: the target must not move while the script runs.
    panel_target: Option<[f32; 2]>,
    viewport_views: Option<BTreeMap<ViewId, om9_scene::workspace::ViewState>>,
    viewport_popup_ready_at: Option<f64>,
    viewport_replay: Vec<serde_json::Value>,
    viewport_idle_step: Option<usize>,
    /// The default scripted CMD input starts only once the CMD field has focus.
    cmd_focus_ready: bool,
    cmd_focus_failed: bool,
    cmd_focus_waits: u32,
}
impl UiAudit {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            step: 0,
            checks: vec![],
            awaiting_screenshot: false,
            receipt_written: false,
            hold: std::env::var("OM9_UI_AUDIT_HOLD").is_ok_and(|s| s == "1"),
            started: std::time::Instant::now(),
            viewport_menu: std::env::var("OM9_UI_AUDIT_VIEWPORT_MENU").is_ok_and(|s| s == "1"),
            selection: std::env::var("OM9_UI_AUDIT_SELECTION").is_ok_and(|s| s == "1")
                || std::env::var("OM9_UI_AUDIT_OCCURRENCES").is_ok_and(|s| s == "1"),
            occurrences: std::env::var("OM9_UI_AUDIT_OCCURRENCES").is_ok_and(|s| s == "1"),
            selection_points: None,
            selection_report: Vec::new(),
            layers: std::env::var("OM9_UI_AUDIT_LAYERS").is_ok_and(|s| s == "1"),
            layer_dialogs: std::env::var("OM9_UI_AUDIT_LAYER_DIALOGS").is_ok_and(|s| s == "1"),
            native_focus_wait_started: None,
            files: std::env::var("OM9_UI_AUDIT_FILES").is_ok_and(|s| s == "1"),
            file_success: std::env::var("OM9_UI_AUDIT_FILE_SUCCESS").is_ok_and(|s| s == "1"),
            file_success_wait_started: None,
            file_success_document: None,
            file_success_original_sha: None,
            file_success_seed_sha: None,
            startup_file: std::env::var_os("OM9_UI_AUDIT_STARTUP_DIRECTORY").is_some(),
            file_close: std::env::var("OM9_UI_AUDIT_FILE_CLOSE").is_ok_and(|s| s == "1"),
            file_close_post_pending: false,
            file_close_events: 0,
            file_picker_receipts: Vec::new(),
            dialog_capture_ready_at: None,
            dialog_capture_pending: None,
            dialog_capture_step: None,
            dialog_capture_metrics: Vec::new(),
            layer_current: None,
            layer_clicks: Vec::new(),
            panels: panel_audit_scenario(),
            panel_checks: Vec::new(),
            panel_drag_observed: None,
            panel_target: None,
            viewport_views: None,
            viewport_popup_ready_at: None,
            viewport_replay: Vec::new(),
            viewport_idle_step: None,
            cmd_focus_ready: false,
            cmd_focus_failed: false,
            cmd_focus_waits: 0,
        }
    }
}
/// A cached selection frame: the keys it was computed for and the rectangle itself.
type SelectionFrame = ((u64, u64, u64, Camera, Rect), Option<Rect>);

impl MatrixApp {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        Self::from_controller(Controller::from_file(path, Limits::default())?)
    }
    pub fn new(document: Document) -> Result<Self> {
        let document = mesh_stress_document(document);
        Self::from_controller(Controller::new(document, Limits::default())?)
    }
    /// Typed startup: validate the configuration, open the empty document and
    /// load the owned layout. No window exists yet, so a failure here is a
    /// document-stage failure the caller reports.
    pub fn start(
        config: &crate::startup::AppConfig,
    ) -> std::result::Result<Self, crate::startup::StartupFailure> {
        use crate::startup::{StartupFailure, StartupStage};
        config.validate()?;
        let mut app = Self::new(Document::new(&config.document_name))
            .map_err(|error| StartupFailure::from_error(StartupStage::Document, &error))?;
        if let Some(path) = &config.layout_path {
            app.set_dock_layout_path(Some(path.clone()));
            app.load_dock_layout();
        }
        Ok(app)
    }
    fn from_controller(controller: Controller) -> Result<Self> {
        let features = serde_json::from_str(include_str!("../assets/features.json"))
            .map_err(|e| Error::Io(e.to_string()))?;
        let dock = DockLayout::matrix_default();
        let regions = dock.resolve([1600., 1000.]);
        let mut app = Self {
            gpu_viewports: std::collections::BTreeMap::new(),
            gpu_state: None,
            controller,
            gumball_mode: om9_scene::gumball_view::Mode::Move,
            gumball_session: None,
            gumball_handle: None,
            gumball_last: None,
            gumball_basis: None,
            gpu_drawn_views: std::collections::BTreeSet::new(),
            transcript: Transcript::new(),
            features,
            focus_cmd: true,
            cmd_rect: None,
            cmd_history_rect: None,
            cmd_token_rects: std::collections::BTreeMap::new(),
            cmd_horizontal_offset: 0.,
            cmd_ime_composing: false,
            view_rects: BTreeMap::new(),
            advanced_layers: false,
            layer_rects: BTreeMap::new(),
            layer_action_rects: BTreeMap::new(),
            layer_dialog_rects: BTreeMap::new(),
            layer_dialog_focused: None,
            layer_dialog_input_token: None,
            layer_color_inputs: std::array::from_fn(|_| String::with_capacity(12)),
            layer_dialog_error: String::new(),
            layer_dialog_ime_composing: false,
            file_controls: BTreeMap::new(),
            file_toolbar_rects: BTreeMap::new(),
            file_menu_rect: None,
            file_menu_items: BTreeMap::new(),
            file_picker_request: None,
            file_picker_attempted: None,
            file_prompt_token: None,
            file_error: String::with_capacity(512),
            three_dm_request: None,
            file_ime_composing: false,
            file_f6_rects: BTreeMap::new(),
            window_title: String::new(),
            world_datum_visible: true,
            current_ui_context: None,
            hover: None,
            band_start: None,
            band_view: None,
            selection_frames: std::collections::BTreeMap::new(),
            ambiguous: Vec::new(),
            ambiguous_pos: None,
            ambiguous_epoch: 0,
            sub_object: None,
            sub_object_kind: om9_scene::scene::PickKind::Face,
            hover_object: None,
            hover_picked_at: None,
            audit: None,
            passes: 0,
            adapter: "headless-egui".into(),
            f6_open: false,
            f6_pinned: false,
            sidebar_rect: None,
            toolbar_rect: None,
            icons: crate::icons::Icons::new(),
            main_category: 0,
            main_category_rects: BTreeMap::new(),
            toolbar_tab: 0,
            style_ready: false,
            section_rects: BTreeMap::new(),
            section_drag: None,
            drop_preview: None,
            work_area: None,
            last_panel_op: None,
            closed_bar_controls: std::collections::BTreeSet::new(),
            float_resync: false,
            panel_bars: Vec::new(),
            left_content_bottom: 0.,
            right_content_bottom: 0.,
            sidebar_scroll: 0.,
            view_menu_capture: None,
            viewport_menu_owns_frame: false,
            viewport_menu_pointer_guard: false,
            viewport_title_arrows: BTreeMap::new(),
            dock,
            dock_path: None,
            dock_dirty: false,
            dock_window: [0., 0.],
            dock_diagnostics: Vec::new(),
            status: StatusSurface::new(),
            regions,
            region_plan: None,
            region_error: None,
            region_status: None,
            min_window_sent: None,
            min_window_monitor: None,
            min_window_source: "unknown",
            audit_monitor_px: None,
            right_dock_rect: None,
            left_rail_outer_px: 0.,
            right_dock_outer_px: 0.,
            column_icon_edge: dock_layout::ICON_BASE_EDGE,
            icon_column: IconColumn::None,
            left_icon_edge: dock_layout::ICON_BASE_EDGE,
            rail_icon_rects: Vec::new(),
            right_dock_icon_rects: Vec::new(),
            sidebar_content_size: None,
            audit_layout_mode: AuditLayoutMode::Default,
            audit_rail_width: None,
            builder_slot_rect: None,
            status_rect: None,
            status_message_rect: None,
            status_summary_rect: None,
            status_dismiss_rect: None,
            dock_panel_rects: BTreeMap::new(),
        };
        for diagnostic in app.controller.file_diagnostics() {
            app.transcript.append_history(diagnostic);
        }
        Ok(app)
    }
    fn feedback(&mut self, result: Result<()>) {
        match result {
            Ok(()) => {
                self.status.complete_success();
            }
            Err(error) => {
                let message = format!("{error}");
                self.transcript.append_history(&message);
                self.status.error(&message);
            }
        }
        self.transcript.set_prompt(self.controller.prompt());
        self.focus_cmd = true;
    }
    /// Attach the application-scoped layout store. The real app uses
    /// `DockLayout::default_store_path`; tests inject an owned directory.
    pub fn set_dock_layout_path(&mut self, path: Option<PathBuf>) {
        self.dock_path = path;
    }
    pub fn dock_layout_path(&self) -> Option<&std::path::Path> {
        self.dock_path.as_deref()
    }
    pub fn dock_panel_rect(&self, key: &str) -> Option<Rect> {
        self.dock_panel_rects.get(key).copied()
    }
    pub fn status_line(&self) -> String {
        self.status.line()
    }
    /// One-line summary of the current menu placement, used by tests and by the
    /// status bar.
    pub fn dock_summary(&self) -> String {
        format!(
            "left {}px, command {}px, right {}px · {}",
            self.regions.left_rail,
            self.regions.command_line,
            self.regions.right_dock,
            self.menu_summary()
        )
    }
    /// Total load of the persisted layout. A missing file is the Matrix
    /// default; an unusable file is reported and does not fail startup.
    pub fn load_dock_layout(&mut self) -> Vec<String> {
        let Some(path) = self.dock_path.clone() else {
            return Vec::new();
        };
        let (mut layout, diagnostics) = DockLayout::load(&path, &Limits::default());
        let mut reported = diagnostics;
        if let [width, height] = self.dock_window
            && width > 0.
            && height > 0.
        {
            reported.extend(layout.clamp_floats([width, height]));
        }
        self.dock = layout;
        self.dock_dirty = false;
        self.cancel_section_drag(crate::panel_ops::CancelReason::LayoutChanged);
        self.note_dock_diagnostics(&reported);
        reported
    }
    /// Persist the layout when the process owns a store path and the layout
    /// changed. A failed write stays a visible warning, never a document error.
    pub fn save_dock_layout(&mut self) -> Result<()> {
        if !self.dock_dirty {
            return Ok(());
        }
        let Some(path) = self.dock_path.clone() else {
            return Ok(());
        };
        match self.dock.save(&path) {
            Ok(()) => {
                self.dock_dirty = false;
                Ok(())
            }
            Err(error) => {
                let message = format!("layout not saved ({error})");
                self.status.warning(&message);
                Err(error)
            }
        }
    }
    /// Restore the Matrix baseline layout. Document, undo journal, pending
    /// command draft and selection are untouched: only panel geometry resets.
    pub fn reset_dock_layout(&mut self, ctx: Option<&egui::Context>) {
        if let Some(ctx) = ctx {
            // egui remembers panel/window rectangles in its own memory; a
            // layout reset must drop exactly that geometry.
            ctx.memory_mut(|memory| memory.reset_areas());
        }
        self.dock.reset_to_default();
        self.dock_dirty = true;
        self.cancel_section_drag(crate::panel_ops::CancelReason::LayoutChanged);
        self.status.info("Layout reset to the Matrix default");
    }
    pub fn toggle_right_dock(&mut self) {
        let visible = self.dock.state(PanelId::RightDock).visible;
        self.dock.set_visible(PanelId::RightDock, !visible);
        self.dock_dirty = true;
        self.status.info(if visible {
            "Right menu column closed"
        } else {
            "Right menu column opened"
        });
    }
    /// View ▸ Matrix Menus ▸ Set as Default: keep the current configuration. The
    /// documented Matrix commands are explicit, so live menu changes are session
    /// state until the user keeps them.
    pub fn set_menus_as_default(&mut self) {
        self.dock_dirty = true;
        if self.save_dock_layout().is_ok() {
            self.status.info("Menu configuration set as default");
        }
    }
    /// View ▸ Matrix Menus ▸ Reset Menus: back to the documented default mode in
    /// this session, without overwriting the configuration the program opens with.
    pub fn reset_menus(&mut self, ctx: Option<&egui::Context>) {
        if let Some(ctx) = ctx {
            // egui remembers panel/window rectangles in its own memory; a reset
            // must drop exactly that geometry.
            ctx.memory_mut(|memory| memory.reset_areas());
        }
        self.dock.reset_to_default();
        self.dock_dirty = false;
        self.cancel_section_drag(crate::panel_ops::CancelReason::LayoutChanged);
        self.status.info("Menus reset to the Matrix default");
    }
    /// View ▸ Matrix Menus ▸ Reload Menus: load the configuration the program
    /// opens with, falling back to the factory default.
    pub fn reload_menus(&mut self) {
        if self.dock_path.is_none() {
            self.dock.reset_to_default();
            self.dock_dirty = false;
            self.cancel_section_drag(crate::panel_ops::CancelReason::LayoutChanged);
            self.status.info("Menus reloaded from the factory default");
            return;
        }
        let diagnostics = self.load_dock_layout();
        self.dock_dirty = false;
        let fallback = diagnostics
            .iter()
            .any(|line| line.contains("Matrix default") && line.contains("layout"));
        self.status.info(if fallback {
            "Menus reloaded from the factory default"
        } else {
            "Menus reloaded from the saved configuration"
        });
    }
    /// True while the session layout differs from the kept configuration.
    pub fn menus_have_unsaved_changes(&self) -> bool {
        self.dock_dirty
    }
    pub fn section_rect(&self, key: &str) -> Option<Rect> {
        self.section_rects.get(key).copied()
    }
    /// One-line summary of the current menu placement for tests and the status bar.
    pub fn menu_summary(&self) -> String {
        let names = |ids: Vec<SectionId>| -> String {
            ids.into_iter()
                .map(SectionId::title)
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "left [{}] · right [{}]",
            names(self.dock.left_sections()),
            names(self.dock.right_sections())
        )
    }
    fn note_dock_diagnostics(&mut self, diagnostics: &[String]) {
        for diagnostic in diagnostics {
            self.transcript.append_history(diagnostic);
        }
        if let Some(last) = diagnostics.last() {
            self.status.warning(last);
        }
        self.dock_diagnostics.extend(diagnostics.iter().cloned());
    }
    /// Keep the real window at or above the supported minimum (1280x720 logical
    /// points), capped to the usable screen when the screen is smaller in
    /// logical points (for example 1366x768 at 125% or 150% DPI). The monitor
    /// is only known once the window exists, so this runs every frame and sends
    /// the viewport command on the first frame and whenever the value changes
    /// (another monitor, another DPI).
    fn enforce_min_window(&mut self, ctx: &egui::Context) {
        let (monitor, source) = match self.audit_monitor_px {
            Some(px) => {
                let scale = ctx.pixels_per_point();
                let points =
                    (scale.is_finite() && scale > 0.).then(|| [px[0] / scale, px[1] / scale]);
                (points, "audit-simulated")
            }
            None => {
                // Not `i.viewport()`: that panics when a backend sends no info.
                let points = ctx.input(|i| {
                    i.raw
                        .viewports
                        .get(&i.raw.viewport_id)
                        .and_then(|info| info.monitor_size)
                });
                (
                    points.map(|size| [size.x, size.y]),
                    if points.is_some() {
                        "viewport"
                    } else {
                        "unknown"
                    },
                )
            }
        };
        let wanted = crate::startup::effective_min_size(monitor);
        self.min_window_monitor = monitor;
        self.min_window_source = source;
        if self.min_window_sent != Some(wanted) {
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::Vec2::new(
                wanted[0], wanted[1],
            )));
            self.min_window_sent = Some(wanted);
        }
    }
    /// Validate the workbench regions for this frame (OM9-L3-01-01-002).
    /// A usable plan replaces the previous one; a failure keeps the last good
    /// plan, changes no layout or scene state and is put on the status line.
    fn refresh_region_plan(&mut self, window: [f32; 2]) {
        match compute_regions_for(
            &self.dock,
            WindowSize::from(window),
            &self.controller.workspace,
            &AttachedCmdFrame,
        ) {
            Ok(plan) => {
                self.region_error = None;
                self.region_plan = Some(plan);
            }
            Err(error) => self.region_error = Some(error),
        }
        self.announce_regions();
    }
    /// Show what the region check wants the user to know: an error while there
    /// is no usable plan, or a notice for a defined fallback (right dock
    /// yielded, left rail narrowed, command input kept). Only text raised here
    /// is cleared here, and a notice never hides an unacknowledged failure.
    fn announce_regions(&mut self) {
        let desired = match (&self.region_error, &self.region_plan) {
            (Some(error), _) => Some((StatusKind::Error, format!("Layout: {error}"))),
            (None, Some(plan)) if !plan.notes.is_empty() => {
                let notes: Vec<String> = plan.notes.iter().map(ToString::to_string).collect();
                Some((StatusKind::Info, format!("Layout: {}", notes.join("; "))))
            }
            _ => None,
        };
        match desired {
            Some((kind, message)) => {
                if self.region_status.as_deref() == Some(message.as_str()) {
                    return;
                }
                if kind != StatusKind::Error && self.status.kind().is_failure() {
                    return;
                }
                // The CMD history keeps the notice even if a later status
                // message (for example a floating-menu warning) replaces it.
                self.transcript.append_history(&message);
                self.status.report(kind, &message);
                self.region_status = Some(message);
            }
            None => {
                if let Some(previous) = self.region_status.take()
                    && self.status.message() == previous
                {
                    self.status.acknowledge();
                }
            }
        }
    }
    fn sync_dock_window(&mut self, window: [f32; 2]) {
        if self.dock_window == window {
            return;
        }
        self.dock_window = window;
        let diagnostics = self.dock.clamp_floats(window);
        if !diagnostics.is_empty() {
            self.dock_dirty = true;
        }
        self.note_dock_diagnostics(&diagnostics);
    }
    fn invoke(&mut self, id: &str, origin: InvocationOrigin) {
        let result = self.controller.dispatch_effect(id, origin);
        if result.is_ok() && !self.f6_pinned {
            self.f6_open = false;
        }
        match result {
            Ok(crate::session::DispatchOutcome::File(outcome)) => {
                let ctx = self.current_ui_context.clone().expect("UI context");
                let _ = self.file_feedback(&ctx, Ok(outcome));
            }
            other => self.feedback(other.map(|_| ())),
        }
    }
    fn submit_draft(&mut self) {
        let draft = self.transcript.draft().to_owned();
        // The underlined letter of the Length option starts its typed assignment, exactly as p22 describes.
        if self.controller.is_running() && draft.eq_ignore_ascii_case("l") {
            self.transcript.set_draft("Length=");
            self.focus_cmd = true;
            return;
        }
        let prompt = self.controller.prompt();
        self.transcript.append_history(&format!("{prompt}{draft}"));
        let result = self.controller.submit_effect(&draft);
        if result.is_ok() {
            self.transcript.record_command(&draft);
            // Echo the canonical feature id next to the typed alias (OM9-L3-03-10-001): the reader sees what
            // the command actually became, not only the words they typed.
            if let Some(feature) = self.controller.active_feature() {
                self.transcript.append_history(&format!("[{feature}]"));
            }
            self.transcript.set_draft("");
        }
        match result {
            Ok(crate::session::DispatchOutcome::File(outcome)) => {
                let ctx = self.current_ui_context.clone().expect("UI context");
                let _ = self.file_feedback(&ctx, Ok(outcome));
            }
            // A command the program does not know is reported with the words the
            // user typed: "unsupported unknown/unavailable command" alone leaves
            // the reader guessing which command was refused (OM9-L3-01-01-007).
            Err(Error::Unsupported(_)) if !draft.trim().is_empty() => {
                let name: String = draft.trim().chars().take(120).collect();
                let message = format!("unknown command: {name}");
                self.transcript.append_history(&message);
                self.status.error(&message);
                self.transcript.set_prompt(self.controller.prompt());
                self.focus_cmd = true;
            }
            other => self.feedback(other.map(|_| ())),
        }
    }
    pub fn command_input_rect(&self) -> Option<Rect> {
        self.cmd_rect
    }
    /// The screen rectangle of one clickable option token, when the running command declares it.
    pub fn command_option_token_rect(&self, name: &str) -> Option<Rect> {
        self.cmd_token_rects.get(name).copied()
    }
    pub fn command_history_rect(&self) -> Option<Rect> {
        self.cmd_history_rect
    }
    pub fn selected_objects(&self) -> &std::collections::BTreeSet<om9_core::identity::ObjectId> {
        self.controller.selected()
    }
    /// The gumball mode the viewport keeps when no modifier is held.
    pub fn set_gumball_mode(&mut self, mode: om9_scene::gumball_view::Mode) {
        self.gumball_mode = mode;
    }

    pub fn gumball_mode(&self) -> om9_scene::gumball_view::Mode {
        self.gumball_mode
    }

    /// The object under the pointer, for tests and status lines.
    pub fn hovered_object(&self) -> Option<om9_core::identity::ObjectId> {
        self.hover_object
    }

    /// Whether a selection marquee is in flight, for tests.
    pub fn band_active(&self) -> bool {
        self.band_start.is_some()
    }

    /// Screen positions of the gumball's scale boxes, computed with the same size rule the painter uses.
    ///
    /// Tests press these points, so a layout change cannot leave a test aiming at a stale coordinate.
    pub fn gumball_scale_boxes(
        &self,
        view: ViewId,
    ) -> Vec<(om9_core::gumball::Handle, egui::Pos2)> {
        let Some(frame) = self.controller.gumball_pivot() else {
            return Vec::new();
        };
        let Some(rect) = self.view_rects.get(&view).copied() else {
            return Vec::new();
        };
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let Some(camera) = self
            .controller
            .workspace
            .views
            .get(&view)
            .map(|view| view.camera.clone())
        else {
            return Vec::new();
        };
        let size = (world_per_pixel(&camera, viewport) * 90.0).max(1e-6);
        let gumball = om9_scene::gumball_view::GumballView::with_size(
            frame,
            om9_scene::gumball_view::Mode::Scale,
            size,
        );
        gumball
            .visible_segments(&camera, viewport)
            .into_iter()
            .filter(|segment| matches!(segment.handle, om9_core::gumball::Handle::Scale(_)))
            .filter_map(|segment| {
                let point = gumball
                    .screen_point(&camera, viewport, segment.end)
                    .ok()
                    .flatten()?;
                Some((segment.handle, egui::pos2(point[0] as f32, point[1] as f32)))
            })
            .collect()
    }

    /// The subobject last picked with Ctrl+Shift+click.
    pub fn sub_object(&self) -> Option<crate::session::SelectedSubobject> {
        self.sub_object
            .clone()
            .filter(|key| self.controller.subselection().contains(key))
    }

    /// Chooses which subobject kind Ctrl+Shift+click looks for (1 = face, 2 = edge, 3 = vertex).
    pub fn set_sub_object_kind(&mut self, kind: om9_scene::scene::PickKind) {
        self.sub_object_kind = kind;
    }

    /// The subobject kind in use.
    pub fn sub_object_kind(&self) -> om9_scene::scene::PickKind {
        self.sub_object_kind
    }

    /// The candidates the last click could not decide between, nearest first.
    pub fn ambiguous_candidates(&self) -> &[om9_core::identity::ObjectId] {
        &self.ambiguous
    }

    /// Chooses one candidate from the disambiguation popup and closes it.
    pub fn choose_ambiguous(&mut self, index: usize) -> Result<()> {
        let Some(id) = self.ambiguous.get(index).copied() else {
            return Err(Error::Invalid("selection candidate"));
        };
        let result = if self.ambiguous_epoch != self.controller.coordinator().epoch() {
            Err(Error::Stale("selection candidates"))
        } else {
            self.controller.select_objects(&[id])
        };
        self.ambiguous.clear();
        self.ambiguous_pos = None;
        result
    }

    /// The popup itself: one row per object under the click.
    /// The label, geometry type and layer colour shown for one ambiguity candidate. Matrix9 shows the object
    /// type and a layer swatch, so the user can tell two overlapping objects apart before choosing.
    fn ambiguous_row(&self, id: &om9_core::identity::ObjectId) -> (String, Option<[u8; 4]>) {
        let document = self.controller.document();
        let (name, kind) = document
            .entities
            .get(id)
            .map(|entity| (entity.label.clone(), entity.geometry.kind().tag()))
            .unwrap_or_else(|| ("Missing".to_owned(), "?"));
        let colour = document
            .display_attributes(*id)
            .ok()
            .and_then(|attributes| attributes.color);
        (format!("{name} [{kind}]"), colour)
    }

    /// The candidate labels, for tests and status reports.
    pub fn ambiguous_labels(&self) -> Vec<String> {
        self.ambiguous
            .iter()
            .map(|id| self.ambiguous_row(id).0)
            .collect()
    }

    fn ambiguous_popup(&mut self, ui: &mut egui::Ui) {
        let Some(position) = self.ambiguous_pos else {
            return;
        };
        let rows: Vec<(usize, String, Option<[u8; 4]>)> = self
            .ambiguous
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let (label, colour) = self.ambiguous_row(id);
                (index, label, colour)
            })
            .collect();
        let mut chosen: Option<usize> = None;
        let area = egui::Area::new(egui::Id::new("om9-pick-popup"))
            .fixed_pos(position)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label("Select object");
                    for (index, label, colour) in &rows {
                        let row = ui.horizontal(|ui| {
                            if let Some(colour) = colour {
                                let (swatch, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    swatch,
                                    1.0,
                                    Color32::from_rgb(colour[0], colour[1], colour[2]),
                                );
                                ui.painter().rect_stroke(
                                    swatch,
                                    1.0,
                                    Stroke::new(1.0, Color32::from_gray(90)),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            ui.button(label.clone())
                        });
                        if row.inner.clicked() {
                            chosen = Some(*index);
                        }
                    }
                });
            });
        if let Some(index) = chosen {
            let result = self.choose_ambiguous(index);
            self.feedback(result);
        } else if ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !area.response.rect.contains(pos))
        }) {
            self.ambiguous.clear();
            self.ambiguous_pos = None;
        }
    }

    /// Highlights the object under the pointer with a white frame, the way Matrix9 previews a pick.
    fn hover_frame(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        use om9_scene::scene::RenderBuffers;
        let Some(pointer) = response.hover_pos() else {
            if self
                .hover_picked_at
                .is_some_and(|(last, _, _)| rect.contains(last))
            {
                self.hover_object = None;
                self.hover_picked_at = None;
            }
            return;
        };
        let epoch = self
            .controller
            .scene()
            .map(|scene| scene.epoch)
            .unwrap_or(0);
        // Picking iterates the scene, so it runs only when the pointer really moved or the view changed.
        let stale = match self.hover_picked_at {
            Some((last, camera, last_epoch)) => {
                last.distance(pointer) > 3.0
                    || camera != view.camera.revision
                    || last_epoch != epoch
            }
            None => true,
        };
        if stale {
            self.hover_picked_at = Some((pointer, view.camera.revision, epoch));
            self.hover_object = self.controller.scene().and_then(|scene| {
                scene
                    .pick_all(
                        [f64::from(pointer.x), f64::from(pointer.y)],
                        &view.camera,
                        viewport,
                        pick_tolerance(ui),
                        om9_scene::scene::PickIntent::Select,
                    )
                    .ok()
                    .and_then(|hits| hits.first().map(|hit| hit.root))
            });
        }
        let Some(hovered) = self.hover_object else {
            return;
        };
        if self.controller.selected().contains(&hovered) {
            // Already highlighted by the selection frame; a second frame would be noise.
            return;
        }
        let Some(scene) = self.controller.scene() else {
            return;
        };
        let object = scene.objects.iter().find(|object| object.root == hovered);
        let Some(object) = object else {
            return;
        };
        let Ok(transform) = object.display_transform() else {
            return;
        };
        let points: &[[f64; 3]] = match object.buffers.as_ref() {
            RenderBuffers::Points { points, .. } => points,
            RenderBuffers::Curve { points, .. } => points,
            RenderBuffers::Mesh { vertices, .. } => vertices,
        };
        let (mut min, mut max, mut found) = (
            egui::pos2(f32::MAX, f32::MAX),
            egui::pos2(f32::MIN, f32::MIN),
            false,
        );
        for point in points {
            let Ok(point) = transform.point(*point) else {
                continue;
            };
            if let Ok(Some(projected)) = view.camera.project(point, viewport) {
                let screen = egui::pos2(projected.screen[0] as f32, projected.screen[1] as f32);
                min = min.min(screen);
                max = max.max(screen);
                found = true;
            }
        }
        if !found {
            return;
        }
        // A margin, so a single point still reads as a box instead of a zero-size rectangle.
        let frame = Rect::from_two_pos(min, max)
            .expand(4.0)
            .intersect(rect.shrink(1.0));
        ui.painter().with_clip_rect(rect).rect_stroke(
            frame,
            0.0,
            Stroke::new(1.0, Color32::from_rgb(255, 255, 255)),
            egui::StrokeKind::Inside,
        );
    }

    pub fn layer_control_rect(&self, slot: u8, control: LayerControl) -> Option<Rect> {
        self.layer_rects.get(&(slot, control)).copied()
    }
    pub fn layer_action_rect(&self, action: &str) -> Option<Rect> {
        self.layer_action_rects.get(action).copied()
    }
    pub fn layer_dialog_control_rect(&self, control: LayerDialogControl) -> Option<Rect> {
        self.layer_dialog_rects.get(&control).copied()
    }
    pub fn world_datum_visible(&self) -> bool {
        self.world_datum_visible
    }

    pub fn main_menu_category(&self) -> usize {
        self.main_category
    }

    pub fn main_category_rect(&self, index: usize) -> Option<Rect> {
        self.main_category_rects.get(&index).copied()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.passes += 1;
        if !self.viewport_menu_owns_frame
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F4))
        {
            self.world_datum_visible = !self.world_datum_visible;
        }
        self.gpu_drawn_views.clear();
        let stale_gesture = self.gumball_basis.as_ref().is_some_and(|basis| {
            basis.epoch != self.controller.coordinator().epoch()
                || basis.selection != self.controller.selection_state().revision
                || !ui.input(|i| i.focused)
        });
        if stale_gesture {
            self.cancel_gumball();
            self.status
                .info("Transform cancelled: selection, document or focus changed");
        }
        let (shift_held, ctrl_held) = ctrl_or_command(ui);
        if shift_held && ctrl_held {
            // Rhino: Ctrl+Shift+click picks a sub-part; 1/2/3 choose face, edge or vertex.
            for (key, kind) in [
                (Key::Num1, om9_scene::scene::PickKind::Face),
                (Key::Num2, om9_scene::scene::PickKind::Edge),
                (Key::Num3, om9_scene::scene::PickKind::Vertex),
            ] {
                if ui.input(|input| input.key_pressed(key)) {
                    self.sub_object_kind = kind;
                }
            }
        }
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            if self.ambiguous_pos.take().is_some() {
                self.ambiguous.clear();
                ui.input_mut(|input| {
                    input.consume_key(Modifiers::NONE, Key::Escape);
                });
            } else {
                let dragging = self.gumball_session.is_some() || self.band_start.is_some();
                // Esc drops the gesture: nothing is committed, the selection stays as it was.
                if self.gumball_session.take().is_some() {
                    self.gumball_handle = None;
                    self.gumball_last = None;
                    self.gumball_basis = None;
                    self.controller.clear_gumball_preview();
                    self.status.info("Transform cancelled");
                }
                self.band_start = None;
                self.band_view = None;
                // No gesture in flight: Esc cancels the selection itself, which is the manual's rule.
                if !dragging {
                    self.controller.clear_selection();
                }
            }
        }
        // Press O to compare the cluster culler with and without depth occlusion. The overlay keeps the
        // state visible, so a report can say which mode it was.
        if ui.input(|input| input.key_pressed(Key::O)) {
            {
                let mut any = false;
                let mut enabled = false;
                for port in self.gpu_viewports.values_mut() {
                    if let Some(cluster) = port.cluster_mut() {
                        any = true;
                        enabled = !cluster.occluding();
                        cluster.set_occlusion(enabled);
                    }
                }
                if any {
                    self.status.info(if enabled {
                        "Occlusion on"
                    } else {
                        "Occlusion off"
                    });
                }
            }
        }
        let ctx = ui.ctx().clone();
        self.current_ui_context = Some(ctx.clone());
        if self.service_window_close(&ctx) {
            return;
        }
        let viewport_popup_open =
            self.view_menu_capture.is_some() && egui::Popup::is_any_open(&ctx);
        let layer_dialog_open = self.controller.layer_dialog().is_some();
        let file_open = self.controller.file_operation().is_some();
        self.viewport_menu_owns_frame = viewport_popup_open
            || layer_dialog_open
            || file_open
            || self.viewport_menu_pointer_guard;
        if (viewport_popup_open || layer_dialog_open || file_open)
            && ctx.input(|i| i.pointer.any_down())
        {
            self.viewport_menu_pointer_guard = true;
        }
        if let Some(token) = self
            .controller
            .file_operation()
            .map(|operation| operation.token())
            && let Err(error) = self.controller.validate_file_operation(token)
        {
            let _ = self.controller.cancel_file_operation(token);
            self.file_picker_request = None;
            self.feedback(Err(error));
        }
        if let Some(token) = self.controller.layer_dialog().map(|d| d.token())
            && let Err(error) = self.controller.validate_layer_dialog(token)
        {
            let _ = self.controller.cancel_layer_dialog(token);
            self.layer_dialog_focused = None;
            self.layer_dialog_ime_composing = false;
            self.feedback(Err(error));
        }
        if let Some((id, capture)) = &self.view_menu_capture
            && self.controller.validate_view_menu(capture).is_err()
        {
            egui::Popup::close_id(&ctx, *id);
            self.view_menu_capture = None;
            self.focus_cmd = true;
        }
        if ctx.memory(|m| m.focused().is_none())
            && ctx.input(|i| {
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
            })
        {
            self.focus_cmd = true;
        }
        let ime_pending = ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))));
        // Composition follows the OS lifecycle across focus changes. Opening a
        // File prompt must not turn a CMD candidate-confirm Enter into Save.
        ctx.input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                        self.file_ime_composing = !text.is_empty()
                    }
                    egui::Event::Ime(egui::ImeEvent::Commit(_))
                    | egui::Event::WindowFocused(false) => self.file_ime_composing = false,
                    _ => {}
                }
            }
        });
        if !self.viewport_menu_owns_frame && !ime_pending && !self.cmd_ime_composing {
            let command = Modifiers {
                ctrl: true,
                command: true,
                ..Modifiers::NONE
            };
            let shift = Modifiers {
                shift: true,
                ..command
            };
            let kind = ctx.input_mut(|input| {
                if input.consume_key(shift, Key::S) {
                    Some(crate::file_lifecycle::FileKind::SaveAs)
                } else if input.consume_key(command, Key::S) {
                    Some(crate::file_lifecycle::FileKind::Save)
                } else if input.consume_key(command, Key::O) {
                    Some(crate::file_lifecycle::FileKind::Open)
                } else if input.consume_key(command, Key::N) {
                    Some(crate::file_lifecycle::FileKind::New)
                } else {
                    None
                }
            });
            if let Some(kind) = kind {
                self.invoke(kind.feature_id(), InvocationOrigin::Menu);
            }
        }
        let escape = !self.cmd_ime_composing
            && !self.layer_dialog_ime_composing
            && !self.file_ime_composing
            && !ime_pending
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if escape {
            // A menu drag in progress is cancelled first and does not fall
            // through: the command draft and the running command stay untouched.
            if self.section_drag().is_some() {
                self.cancel_section_drag(crate::panel_ops::CancelReason::Escape);
            } else if let Some(token) = self
                .controller
                .file_operation()
                .map(|operation| operation.token())
            {
                let result = self.controller.cancel_file_operation(token);
                self.file_picker_request = None;
                self.feedback(result);
            } else if let Some(token) = self.controller.layer_dialog().map(|d| d.token()) {
                let result = self.controller.cancel_layer_dialog(token);
                self.layer_dialog_focused = None;
                self.layer_dialog_ime_composing = false;
                self.feedback(result);
            } else if self.viewport_menu_owns_frame || egui::Popup::is_any_open(&ctx) {
                egui::Popup::close_all(&ctx);
                self.view_menu_capture = None;
                self.focus_cmd = true;
            } else {
                if !self.f6_pinned {
                    self.f6_open = false;
                }
                self.controller.cancel();
                self.hover = None;
                self.transcript.set_draft("");
                self.transcript.set_prompt(self.controller.prompt());
                self.focus_cmd = true;
            }
        }
        self.configure_style(&ctx);
        self.enforce_min_window(&ctx);
        ui.set_style(ctx.style_of(egui::Theme::Light));
        let viewport = ui.max_rect().size();
        self.refresh_region_plan([viewport.x, viewport.y]);
        // Floating menus are only pulled back inside a window the region check
        // accepts; a window below the minimum must not rewrite stored layout.
        if self.region_error.is_none() {
            self.sync_dock_window([viewport.x, viewport.y]);
        }
        self.regions = self.dock.resolve([viewport.x, viewport.y]);
        self.update_work_area(viewport);
        let regions = self.regions;
        self.dock_panel_rects.clear();
        self.section_rects.clear();
        self.builder_slot_rect = None;
        self.panel_bars.clear();
        self.closed_bar_controls.clear();
        self.left_content_bottom = 0.;
        self.right_content_bottom = 0.;
        egui::Panel::top("menu")
            .exact_size(21.)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(248, 250, 252))
                    .inner_margin(3),
            )
            .show(ui, |ui| {
                if self.viewport_menu_owns_frame {
                    ui.disable();
                }
                self.menu_ui(ui);
            });
        let left_panel = egui::Panel::left("matrix-sidebar")
            .default_size(regions.left_rail)
            .min_size(dock_layout::LEFT_RAIL_MIN)
            .max_size(dock_layout::LEFT_RAIL_MAX)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(20, 29, 40))
                    .inner_margin(2),
            )
            .show(ui, |ui| {
                self.sidebar_rect = Some(ui.max_rect());
                // Icons follow the OUTER width (inner content plus frame chrome).
                self.left_icon_edge = dock_layout::rail_icon_edge(
                    ui.max_rect().width() + dock_layout::LEFT_RAIL_CHROME_PX,
                );
                self.column_icon_edge = self.left_icon_edge;
                self.icon_column = IconColumn::Left;
                if self.viewport_menu_owns_frame {
                    ui.disable();
                }
                self.sidebar_ui(ui);
                self.icon_column = IconColumn::None;
                self.column_icon_edge = dock_layout::ICON_BASE_EDGE;
            });
        // The stored rail width is the OUTER panel width (what the manual and the
        // user mean by 300), not the inner content width.
        self.left_rail_outer_px = left_panel.response.rect.width();
        self.sync_panel_size(PanelId::LeftRail, self.left_rail_outer_px);
        egui::Panel::top("command")
            .default_size(regions.command_line)
            .min_size(dock_layout::COMMAND_MIN)
            .max_size(dock_layout::COMMAND_MAX)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(130, 180, 140))
                    .inner_margin(3),
            )
            .show(ui, |ui| {
                ui.set_min_height(ui.available_height());
                self.sync_panel_size(PanelId::CommandLine, ui.max_rect().height());
                self.command_ui(ui);
            });
        egui::Panel::top("matrix-toolbars")
            .exact_size(67.)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(231, 238, 245))
                    .inner_margin(2),
            )
            .show(ui, |ui| {
                if self.viewport_menu_owns_frame {
                    ui.disable();
                }
                self.toolbars_ui(ui);
            });
        // Far-right menu column (documented ~300-pixel mirror of the left one).
        // It only appears when a menu is docked there and yields on a narrow
        // window instead of overlapping the command line or the canvas.
        if regions.right_dock_visible {
            let right_panel = egui::Panel::right("matrix-right-dock")
                .default_size(regions.right_dock)
                .min_size(dock_layout::RIGHT_DOCK_MIN)
                .max_size(dock_layout::RIGHT_DOCK_MAX)
                .resizable(true)
                .frame(
                    egui::Frame::new()
                        .fill(Color32::from_rgb(27, 39, 53))
                        .inner_margin(3),
                )
                .show(ui, |ui| {
                    self.right_dock_rect = Some(ui.max_rect());
                    self.column_icon_edge = dock_layout::rail_icon_edge(
                        ui.max_rect().width() + dock_layout::LEFT_RAIL_CHROME_PX,
                    );
                    self.icon_column = IconColumn::Right;
                    if self.viewport_menu_owns_frame {
                        ui.disable();
                    }
                    self.right_dock_ui(ui);
                    self.icon_column = IconColumn::None;
                    self.column_icon_edge = dock_layout::ICON_BASE_EDGE;
                });
            self.right_dock_outer_px = right_panel.response.rect.width();
            self.sync_panel_size(PanelId::RightDock, self.right_dock_outer_px);
        } else {
            self.right_dock_rect = None;
            self.right_dock_outer_px = 0.;
            self.right_dock_icon_rects.clear();
        }
        // Global status surface (L3-007): always visible, never owns Escape.
        egui::Panel::bottom("matrix-status")
            .exact_size(dock_layout::STATUS_BAR_HEIGHT)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(17, 24, 39))
                    .inner_margin(2),
            )
            .show(ui, |ui| {
                self.status_rect = Some(ui.max_rect());
                self.status_message_rect = None;
                self.status_summary_rect = None;
                self.status_dismiss_rect = None;
                self.status_bar_ui(ui);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::BLACK).inner_margin(0))
            .show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                let views = self.controller.workspace.visible_views();
                self.view_rects.clear();
                // The tiling lives in workbench_regions; the frame feeds it the
                // rectangle egui actually left for the canvas.
                let center = RegionRect::new(rect.min.x, rect.min.y, rect.width(), rect.height());
                match tile_views(center, &views) {
                    Ok(slots) => {
                        for slot in slots {
                            let view_rect = Rect::from_min_size(
                                egui::pos2(slot.rect.x, slot.rect.y),
                                Vec2::new(slot.rect.width, slot.rect.height),
                            );
                            self.view_rects.insert(slot.view, view_rect);
                            self.viewport_ui(ui, slot.view, view_rect);
                        }
                    }
                    Err(error) => {
                        // Not drawn, and said so on the status line.
                        if self.region_error.is_none() {
                            self.region_error = Some(error);
                        }
                        self.announce_regions();
                    }
                }
                ui.allocate_rect(rect, Sense::hover());
            });
        if !self.viewport_menu_owns_frame
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F6))
        {
            self.f6_open = !self.f6_open;
        }
        self.file_f6_rects.clear();
        if self.f6_open {
            let stored = self
                .dock
                .state(PanelId::CommandSearch)
                .float
                .unwrap_or_else(FloatRect::command_search_default);
            let mut open = self.f6_open;
            let window = egui::Window::new("F6")
                // A window named only by its title takes its identity from that
                // title, so a translated or renamed title would move it (adapted
                // from the pinned QymCAD window-identity guard).
                .id(egui::Id::new("om9-command-search"))
                .default_pos([stored.x, stored.y])
                .default_size([stored.width, stored.height])
                .open(&mut open)
                .show(&ctx, |ui| {
                    if self.viewport_menu_owns_frame {
                        ui.disable();
                    }
                    ui.checkbox(&mut self.f6_pinned, "Pin");
                    if ui.button("Line").clicked() {
                        self.invoke(LINE, InvocationOrigin::F6);
                    }
                    if ui.button("Point").clicked() {
                        self.invoke(POINT, InvocationOrigin::F6);
                    }
                    for kind in crate::file_lifecycle::FileKind::ALL {
                        let response = ui.button(kind.label());
                        self.file_f6_rects.insert(kind, response.rect);
                        if response.clicked() {
                            self.invoke(kind.feature_id(), InvocationOrigin::F6);
                        }
                    }
                });
            if let Some(window) = window {
                let rect = window.response.rect;
                if (rect.min.x - stored.x).abs() > 0.5
                    || (rect.min.y - stored.y).abs() > 0.5
                    || (rect.width() - stored.width).abs() > 0.5
                    || (rect.height() - stored.height).abs() > 0.5
                {
                    self.dock.set_float(
                        PanelId::CommandSearch,
                        Some(FloatRect {
                            x: rect.min.x,
                            y: rect.min.y,
                            width: rect.width(),
                            height: rect.height(),
                        }),
                    );
                    self.dock_dirty = true;
                }
            }
            self.f6_open = self.f6_open && open;
        }
        // Menus dragged out over the model float as owned windows; their
        // rectangle is part of the layout, so it survives the session.
        self.float_menu_windows(&ctx);
        // Frame-level recovery of floating menus, then the drag/preview state of
        // the single door (L3-003).
        self.resync_floats();
        let zones = self.panel_drop_zones(
            ui.max_rect(),
            &self.panel_bars.clone(),
            self.left_content_bottom,
            self.right_content_bottom,
        );
        self.panel_drag_frame(&ctx, zones);
        if let Some((id, _)) = &self.view_menu_capture
            && !egui::Popup::is_id_open(&ctx, *id)
        {
            self.view_menu_capture = None;
            self.focus_cmd = true;
        }
        self.layer_dialog_ui(&ctx);
        self.file_ui(&ctx);
        self.update_file_title(&ctx);
        if self.focus_cmd
            && !egui::Popup::is_any_open(&ctx)
            && self.controller.layer_dialog().is_none()
            && self.controller.file_operation().is_none()
        {
            ctx.memory_mut(|m| m.request_focus(egui::Id::new("matrix-command-transcript")));
        }
        self.audit_receipt(&ctx);
        if !ctx.input(|i| i.pointer.any_down()) {
            self.viewport_menu_pointer_guard = false;
        }
    }
    /// Orbit, pan and zoom for one viewport.
    ///
    /// Right-drag orbits the camera around its target, right-drag with Shift (or the middle button) pans,
    /// and the wheel dollies. Every accepted change lands on the workspace camera, so it persists and the
    /// frame drawn below already uses it.
    fn viewport_camera_input(
        &mut self,
        ui: &mut egui::Ui,
        id: ViewId,
        response: &egui::Response,
        viewport: Viewport,
    ) {
        let (shift, wheel) = ui.input(|input| (input.modifiers.shift, input.smooth_scroll_delta.y));
        let delta = response.drag_delta();
        // In an orthographic view a turntable drag has no useful degree of freedom (the elevation is
        // already at a pole), so the right button pans there, which is what a 2D view expects. Only the
        // perspective view orbits.
        let orthographic = matches!(
            self.controller
                .workspace
                .views
                .get(&id)
                .map(|view| view.camera.projection),
            Some(om9_scene::camera::Projection::Orthographic { .. })
        );
        let orbit_drag =
            response.dragged_by(egui::PointerButton::Secondary) && !shift && !orthographic;
        let pan_drag = response.dragged_by(egui::PointerButton::Middle)
            || (response.dragged_by(egui::PointerButton::Secondary) && (shift || orthographic));
        let wheel_over = response.hovered() && wheel != 0.0;
        if !orbit_drag && !pan_drag && !wheel_over {
            return;
        }
        let Some(current) = self
            .controller
            .workspace
            .views
            .get(&id)
            .map(|view| view.camera.clone())
        else {
            return;
        };
        let mut camera = current;
        if orbit_drag && delta != Vec2::ZERO {
            if camera
                .orbit(delta.x as f64 * 0.01, -(delta.y as f64) * 0.01)
                .is_err()
            {
                return;
            }
        } else if pan_drag && delta != Vec2::ZERO {
            let per_pixel = world_per_pixel(&camera, viewport);
            if camera
                .pan([-delta.x as f64 * per_pixel, delta.y as f64 * per_pixel])
                .is_err()
            {
                return;
            }
        } else if wheel_over {
            if camera.zoom(0.9_f64.powf(wheel as f64 / 40.0)).is_err() {
                return;
            }
        } else {
            return;
        }
        if let Some(state) = self.controller.workspace.views.get_mut(&id) {
            state.camera = camera;
        }
        ui.ctx().request_repaint();
    }

    fn viewport_ui(&mut self, ui: &mut egui::Ui, id: ViewId, rect: Rect) {
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        // The camera is updated before anything is painted, so this frame already shows the new view.
        let canvas = Rect::from_min_max(egui::pos2(rect.min.x, rect.min.y + 18.), rect.max);
        let response = ui.interact(
            canvas,
            ui.id().with(("canvas", id.title())),
            Sense::click_and_drag(),
        );
        self.viewport_camera_input(ui, id, &response, viewport);
        let Some(view) = self.controller.workspace.views.get(&id).cloned() else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, 0, Color32::BLACK);

        let gpu_drew = if let Some(state) = self.gpu_state.as_ref() {
            // Each pane retains its own colour/depth target and camera.
            let port = self
                .gpu_viewports
                .entry(id)
                .or_insert_with(|| crate::gpu_viewport::GpuViewport::new(state));
            port.show(
                ui,
                rect,
                state,
                crate::gpu_viewport::scene_camera(&view.camera),
                view.camera.revision,
                &view.camera,
                self.controller.scene(),
                view.cplane,
                self.controller.selected(),
                self.controller.selection_state().revision,
                self.controller.preview_revision(),
            )
        } else {
            false
        };
        if gpu_drew {
            self.gpu_drawn_views.insert(id);
            paint_fps(ui, rect);
        }
        if !gpu_drew {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "GPU viewport unavailable",
                egui::FontId::proportional(13.),
                Color32::LIGHT_RED,
            );
        }
        self.selection_frame(ui, id, &view, viewport, rect);
        self.subselection_overlay(ui, &view, viewport, rect);
        self.gumball_ui(ui, &response, &view, viewport, rect);
        self.selection_band(ui, &response, &view, viewport, rect);
        if id == self.controller.workspace.active {
            self.hover_frame(ui, &response, &view, viewport, rect);
            self.ambiguous_popup(ui);
        }
        if let Some(point) = self.hover
            && let Ok(Some((a, b))) = self.controller.preview(point)
        {
            paint_line(
                &painter,
                &view.camera,
                viewport,
                a,
                b,
                [0, 130, 85, 255],
                1.5,
            );
        }
        let title_width = ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(
                    id.title().into(),
                    egui::FontId::proportional(11.),
                    Color32::BLACK,
                )
                .size()
                .x
        }) + 8.;
        let title = Rect::from_min_size(rect.min, Vec2::new(title_width, 18.));
        let title_response =
            ui.interact(title, ui.id().with(("title", id.title())), Sense::click());
        let active = self.controller.workspace.active == id;
        let title_color = if active {
            Color32::from_rgb(130, 180, 140)
        } else {
            Color32::from_rgb(192, 204, 211)
        };
        painter.rect_filled(title, 0, title_color);
        painter.text(
            title.min + Vec2::new(4., 3.),
            egui::Align2::LEFT_TOP,
            id.title(),
            egui::FontId::proportional(11.),
            Color32::from_rgb(43, 62, 77),
        );
        let arrow_rect = Rect::from_min_size(title.right_top(), Vec2::new(12., 18.));
        self.viewport_title_arrows.insert(id, arrow_rect);
        let arrow_response = ui.interact(
            arrow_rect,
            ui.id().with(("view-dropdown", id.title())),
            Sense::click(),
        );
        painter.rect_filled(arrow_rect, 0, title_color);
        let arrow = arrow_rect.center();
        painter.add(egui::Shape::convex_polygon(
            vec![
                arrow + Vec2::new(-3., -1.),
                arrow + Vec2::new(3., -1.),
                arrow + Vec2::new(0., 2.),
            ],
            Color32::BLACK,
            Stroke::NONE,
        ));
        if title_response.clicked() || arrow_response.clicked() {
            self.controller.workspace.active = id;
            self.focus_cmd = true;
        }
        if title_response.double_clicked() {
            let r = self.controller.workspace.toggle_maximize(id);
            self.feedback(r);
        }
        self.viewport_title_popup(ui, id, &arrow_response);
        paint_axis_triad(&painter, &view.camera, viewport, rect);
        if self.world_datum_visible {
            paint_world_datum(&painter, &view.camera, viewport, rect);
        }
        paint_view_marker(&painter, rect, id);
        if !self.viewport_menu_owns_frame
            && response.hovered()
            && let Some(pos) = response.hover_pos()
        {
            let world = view
                .camera
                .ray([pos.x as f64, pos.y as f64], viewport)
                .and_then(|ray| ray.intersect_plane(self.controller.input_plane()));
            if let Ok(world) = world {
                self.hover = world;
            }
            if response.clicked() {
                self.controller.workspace.active = id;
                if self.controller.is_running() {
                    if let Some(point) = self.hover {
                        let result = self.controller.accept_world_point(point);
                        self.feedback(result);
                    }
                } else {
                    let candidates = self.controller.scene().and_then(|scene| {
                        scene
                            .pick_all(
                                [pos.x as f64, pos.y as f64],
                                &view.camera,
                                viewport,
                                pick_tolerance(ui),
                                PickIntent::Select,
                            )
                            .ok()
                            .map(|hits| {
                                hits.into_iter()
                                    .filter(|hit| {
                                        hit.revalidate_in_view(
                                            self.controller.coordinator(),
                                            &view.camera,
                                            viewport,
                                        )
                                        .is_ok()
                                    })
                                    .map(|hit| hit.root)
                                    .collect::<Vec<_>>()
                            })
                    });
                    // Several objects under the pointer: Matrix9 asks which one, instead of guessing.
                    if let Some(list) = candidates.as_ref()
                        && list.len() > 1
                        && {
                            let (shift, ctrl) = ctrl_or_command(ui);
                            !shift && !ctrl
                        }
                    {
                        self.ambiguous = list.clone();
                        self.ambiguous_pos = Some(pos);
                        self.ambiguous_epoch = self.controller.coordinator().epoch();
                        self.focus_cmd = true;
                        return;
                    }
                    self.ambiguous.clear();
                    self.ambiguous_pos = None;
                    let picked = self.controller.scene().and_then(|scene| {
                        scene
                            .pick(
                                [pos.x as f64, pos.y as f64],
                                &view.camera,
                                viewport,
                                pick_tolerance(ui),
                                PickIntent::Select,
                            )
                            .ok()
                            .flatten()
                            .filter(|p| {
                                p.revalidate_in_view(
                                    self.controller.coordinator(),
                                    &view.camera,
                                    viewport,
                                )
                                .is_ok()
                            })
                            .map(|p| p.root)
                    });
                    let (shift, ctrl) = ctrl_or_command(ui);
                    if shift && ctrl {
                        // Sub-object selection: pick the face, edge or vertex under the pointer and keep its
                        // persistent key next to the object it belongs to.
                        let kind = self.sub_object_kind;
                        let hit = self.controller.scene().and_then(|scene| {
                            scene
                                .pick_kind(
                                    [pos.x as f64, pos.y as f64],
                                    &view.camera,
                                    viewport,
                                    pick_tolerance(ui),
                                    PickIntent::Select,
                                    kind,
                                )
                                .ok()
                                .flatten()
                        });
                        if let Some(hit) = hit {
                            let root = hit.root;
                            let subset = hit.subset.clone();
                            if let Err(error) = self.controller.select_objects(&[root]) {
                                self.feedback(Err(error));
                                return;
                            }
                            // Keep the exact hit path; no root-only member lookup is allowed.
                            if let Err(error) = hit.revalidate_in_view(
                                self.controller.coordinator(),
                                &view.camera,
                                viewport,
                            ) {
                                self.feedback(Err(error));
                                return;
                            }
                            let reference = crate::session::SelectedSubobject::from_hit(&hit);
                            if let Err(error) =
                                self.controller.toggle_subobject_ref(reference.clone())
                            {
                                self.feedback(Err(error));
                                return;
                            }
                            self.sub_object = self
                                .controller
                                .subselection()
                                .contains(&reference)
                                .then_some(reference);
                            let message = match subset {
                                om9_scene::scene::Subelement::MeshFace(face) => {
                                    format!("Face {face}")
                                }
                                om9_scene::scene::Subelement::MeshEdge { a, b } => {
                                    format!("Edge {a}-{b}")
                                }
                                om9_scene::scene::Subelement::MeshVertex(vertex) => {
                                    format!("Vertex {vertex}")
                                }
                                other => format!("{other:?}"),
                            };
                            self.status.info(&message);
                        }
                        return;
                    }
                    self.controller.clear_subselection();
                    self.sub_object = None;
                    let result = match (picked, ctrl) {
                        (Some(id), true) => self.controller.toggle_selection(id),
                        _ => self.controller.select_root(picked, shift),
                    };
                    self.feedback(result);
                    self.focus_cmd = true;
                }
            }
        }
        painter.rect_stroke(
            rect.shrink(0.5),
            0,
            Stroke::new(1., Color32::from_gray(170)),
            egui::StrokeKind::Inside,
        );
    }
    /// Draws a white frame around the selected objects, the way Matrix9 shows that a set is selected.
    ///
    /// The frame is the screen bounding box of every projected vertex, so it grows with the selection and
    /// costs one pass over the selected geometry per viewport, not over the whole scene.
    fn selection_frame(
        &mut self,
        ui: &mut egui::Ui,
        id: ViewId,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        use om9_scene::scene::RenderBuffers;
        // The frame marks a *set*: one selected object already reads as selected through its highlight and the
        // gumball, and a box around it only gets in the way.
        if self.controller.selected().len() < 2 {
            self.selection_frames.remove(&id);
            return;
        }
        let key = (
            self.controller.selection_state().revision,
            self.controller.coordinator().epoch(),
            self.controller.preview_revision(),
            view.camera.clone(),
            rect,
        );
        if let Some((old, cached)) = self.selection_frames.get(&id)
            && *old == key
        {
            if let Some(frame) = cached {
                ui.painter().with_clip_rect(rect).rect_stroke(
                    *frame,
                    0.0,
                    Stroke::new(1.0, Color32::WHITE),
                    egui::StrokeKind::Inside,
                );
            }
            return;
        }
        let mut min = egui::pos2(f32::MAX, f32::MAX);
        let mut max = egui::pos2(f32::MIN, f32::MIN);
        let mut found = false;
        if let Some(scene) = self.controller.scene() {
            for object in &scene.objects {
                if !self.controller.selected().contains(&object.root) {
                    continue;
                }
                let Ok(transform) = object.display_transform() else {
                    continue;
                };
                let points: &[[f64; 3]] = match object.buffers.as_ref() {
                    RenderBuffers::Points { points, .. } => points,
                    RenderBuffers::Curve { points, .. } => points,
                    RenderBuffers::Mesh { vertices, .. } => vertices,
                };
                for point in points {
                    let Ok(point) = transform.point(*point) else {
                        continue;
                    };
                    if let Ok(Some(projected)) = view.camera.project(point, viewport) {
                        let screen =
                            egui::pos2(projected.screen[0] as f32, projected.screen[1] as f32);
                        min = min.min(screen);
                        max = max.max(screen);
                        found = true;
                    }
                }
            }
        }
        if !found {
            self.selection_frames.insert(id, (key, None));
            return;
        }
        // A small margin so a flat selection (two points on one line) still reads as a box.
        let frame = Rect::from_two_pos(min, max)
            .expand(3.0)
            .intersect(rect.shrink(1.0));
        if frame.width() <= 3.0 || frame.height() <= 3.0 {
            self.selection_frames.insert(id, (key, None));
            return;
        }
        self.selection_frames.insert(id, (key, Some(frame)));
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_stroke(
            frame,
            0.0,
            Stroke::new(1.0, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }

    /// Drag a rectangle in empty space to select every object it touches (the Matrix9 rubber band).
    fn selection_band(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        let painter = ui.painter().with_clip_rect(rect);
        if self.gumball_session.is_some() {
            self.band_start = None;
            return;
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            // egui reports the drag only after the movement threshold; the band must start where the
            // button went down or the first few pixels are lost.
            let origin = ui
                .input(|input| input.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos());
            if let Some(origin) = origin {
                self.band_start = Some(origin);
                self.band_view = Some(self.controller.workspace.active);
            }
        } else if response.dragged_by(egui::PointerButton::Primary) {
            if let (Some(start), Some(pointer)) = (self.band_start, response.interact_pointer_pos())
            {
                // Matrix9 draws the band as a plain white rectangle, not a filled selection box.
                let band = Rect::from_two_pos(start, pointer);
                painter.rect_stroke(
                    band,
                    0.0,
                    Stroke::new(1.0, Color32::WHITE),
                    egui::StrokeKind::Inside,
                );
            }
        } else if response.drag_stopped_by(egui::PointerButton::Primary) {
            if let (Some(start), Some(pointer)) = (self.band_start, response.interact_pointer_pos())
            {
                let band = Rect::from_two_pos(start, pointer);
                if band.width() > 3.0 || band.height() > 3.0 {
                    // Matrix9: dragging left to right selects objects fully inside the box, right to left
                    // selects anything the box touches.
                    let window = pointer.x >= start.x;
                    let picked = self.objects_in_band(&view.camera, viewport, band, window);
                    // Rhino: Shift adds to the selection, Ctrl removes from it, plain replaces it.
                    let (shift, ctrl) = ctrl_or_command(ui);

                    let result = if shift {
                        self.controller.add_objects(&picked)
                    } else if ctrl {
                        self.controller.remove_objects(&picked)
                    } else {
                        self.controller.select_objects(&picked)
                    };
                    self.feedback(result);
                }
            }
            self.band_start = None;
            self.band_view = None;
        }
    }

    /// Draws the picked faces, edges and vertices in green, so Ctrl+Shift+click has visible feedback.
    fn subselection_overlay(
        &mut self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        use om9_scene::scene::{RenderBuffers, Subelement};
        if self.controller.subselection().is_empty() {
            return;
        }
        let picked = self.controller.subselection().to_vec();
        let Some(scene) = self.controller.scene() else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        let green = Color32::from_rgb(90, 220, 130);
        for reference in &picked {
            let subset = &reference.subset;
            let Some(object) = scene
                .objects
                .iter()
                .find(|object| object.root == reference.root && object.path == reference.path)
            else {
                continue;
            };
            let Ok(transform) = object.display_transform() else {
                continue;
            };
            let project = |point: [f64; 3]| -> Option<egui::Pos2> {
                let world = transform.point(point).ok()?;
                let projected = view.camera.project(world, viewport).ok()??;
                Some(egui::pos2(
                    projected.screen[0] as f32,
                    projected.screen[1] as f32,
                ))
            };
            if let Ok(entity) = scene.entity_for(object)
                && let om9_core::geometry::Geometry::Brep(shape) = &entity.geometry
            {
                let display = &shape.display;
                if let Subelement::BrepVertex(id) = subset {
                    if let Some(index) = display.vertex_ids.iter().position(|key| key == id)
                        && let Some(point) =
                            project(display.points[display.vertices[index] as usize])
                    {
                        painter.circle_filled(point, 4., green);
                    }
                } else {
                    let edge = if let Subelement::BrepEdge(id) = subset {
                        display.edges.iter().position(|key| key == id)
                    } else {
                        None
                    };
                    let face = if let Subelement::BrepFace(id) = subset {
                        display.faces.iter().position(|key| key == id)
                    } else {
                        None
                    };
                    for [a, b, e] in &display.segments {
                        if (edge == Some(*e as usize)
                            || face.is_some_and(|f| display.face_edges.contains(&[f as u32, *e])))
                            && let (Some(a), Some(b)) = (
                                project(display.points[*a as usize]),
                                project(display.points[*b as usize]),
                            )
                        {
                            painter.line_segment([a, b], Stroke::new(2., green));
                        }
                    }
                }
                continue;
            }
            match (subset, object.buffers.as_ref()) {
                (
                    Subelement::MeshFace(face),
                    RenderBuffers::Mesh {
                        vertices,
                        triangles,
                        ..
                    },
                ) => {
                    // Outline of the polygon: every triangle that belongs to the face contributes its
                    // corners, so a quad reads as a quad, not as two triangles.
                    let mut outline: Vec<egui::Pos2> = Vec::new();
                    if let Ok(entity) = scene.entity_for(object)
                        && let om9_core::geometry::Geometry::Mesh(mesh) = &entity.geometry
                        && let Some(indices) = mesh.faces.get(*face as usize)
                    {
                        for index in indices {
                            if let Some(point) =
                                vertices.get(*index as usize).and_then(|p| project(*p))
                            {
                                outline.push(point);
                            }
                        }
                    }
                    let _ = triangles;
                    if outline.len() >= 3 {
                        outline.push(outline[0]);
                        painter.add(egui::Shape::line(outline, Stroke::new(2.0, green)));
                    }
                }
                (Subelement::MeshEdge { a, b }, RenderBuffers::Mesh { vertices, .. }) => {
                    if let (Some(start), Some(end)) = (
                        project(vertices[*a as usize]),
                        project(vertices[*b as usize]),
                    ) {
                        painter.line_segment([start, end], Stroke::new(2.0, green));
                    }
                }
                (Subelement::MeshVertex(vertex), RenderBuffers::Mesh { vertices, .. }) => {
                    if let Some(point) = project(vertices[*vertex as usize]) {
                        painter.circle_filled(point, 4.0, green);
                    }
                }
                (
                    Subelement::CurveParameter(parameter),
                    RenderBuffers::Curve { points, parameters },
                ) => {
                    if let Some(index) = parameters
                        .iter()
                        .position(|value| (value - parameter).abs() < 1e-9)
                        && let Some(point) = points.get(index).and_then(|point| project(*point))
                    {
                        painter.circle_filled(point, 4.0, green);
                    }
                }
                (
                    Subelement::CloudPoint(native),
                    RenderBuffers::Points {
                        points,
                        native_indices,
                    },
                ) => {
                    if let Some(index) = native_indices.iter().position(|value| value == native)
                        && let Some(point) = points.get(index).and_then(|point| project(*point))
                    {
                        painter.circle_filled(point, 4.0, green);
                    }
                }
                _ => {}
            }
        }
    }

    /// Query displayed primitives, including edges crossing a band without any vertex inside.
    fn objects_in_band(
        &self,
        camera: &om9_scene::camera::Camera,
        viewport: Viewport,
        band: Rect,
        window: bool,
    ) -> Vec<om9_core::identity::ObjectId> {
        self.controller
            .scene()
            .and_then(|scene| {
                scene
                    .select_band(
                        camera,
                        viewport,
                        om9_scene::selection::SelectionRect {
                            min: [band.min.x as f64, band.min.y as f64],
                            max: [band.max.x as f64, band.max.y as f64],
                        },
                        window,
                    )
                    .ok()
            })
            .unwrap_or_default()
    }

    fn gumball_ui(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        use om9_core::gumball::Handle;
        use om9_scene::gumball_view::{GumballView, Mode};

        let modifiers = ui.input(|i| i.modifiers);
        // The stored mode breaks ties between overlapping handles. Alt toggles copy during a gesture.
        let primary = self.gumball_mode;
        let Some(frame) = self.controller.gumball_pivot() else {
            if self.gumball_session.is_some() {
                self.cancel_gumball();
            }
            return;
        };
        // Roughly ninety pixels of handle, independent of zoom and of the projection.
        let size = (world_per_pixel(&view.camera, viewport) * 90.0).max(1e-6);
        // Matrix9 shows move, rotate and scale on one gumball: draw all three groups, and let the pointer
        // pick the nearest handle at the press position.
        let mut views = vec![
            GumballView::with_size(frame, Mode::Move, size),
            GumballView::with_size(frame, Mode::Rotate, size),
            GumballView::with_size(frame, Mode::Scale, size),
        ];
        match primary {
            Mode::Rotate => views.swap(0, 1),
            Mode::Scale => views.swap(0, 2),
            Mode::Move => {}
        }
        let painter = ui.painter().with_clip_rect(rect);

        let colour = gumball_colour;
        let to_pos = |point: [f64; 2]| egui::pos2(point[0] as f32, point[1] as f32);
        // Arcs first, arrows over them, boxes last: the rotation rings sit behind the arrow cluster.
        let mut draw_order: Vec<om9_scene::gumball_view::GumballView> =
            Vec::with_capacity(views.len());
        for mode in [Mode::Rotate, Mode::Scale, Mode::Move] {
            if let Some(view) = views.iter().find(|candidate| candidate.mode == mode) {
                draw_order.push(*view);
            }
        }
        for active in &draw_order {
            // Only the handles this view can show: an axis pointing at the camera would draw its box on the
            // pivot, which is exactly the pile of squares a single view must not show.
            let segments = active.visible_segments(&view.camera, viewport);
            // One pass per handle: arrows, arcs and boxes are drawn differently.
            for handle in active.handles() {
                let mut points: Vec<egui::Pos2> = Vec::new();
                for segment in segments.iter().filter(|segment| segment.handle == handle) {
                    if let (Ok(Some(start)), Ok(Some(end))) = (
                        active.screen_point(&view.camera, viewport, segment.start),
                        active.screen_point(&view.camera, viewport, segment.end),
                    ) {
                        if points.is_empty() {
                            points.push(to_pos(start));
                        }
                        points.push(to_pos(end));
                    }
                }
                if points.is_empty() {
                    continue;
                }
                let tint = colour(handle);
                match handle {
                    Handle::Rotate(_) => {
                        // A smooth arc rather than forty separate chords.
                        painter.add(egui::Shape::line(points, Stroke::new(2.0, tint)));
                    }
                    Handle::MoveAxis(_) => {
                        let (start, tip) = (points[0], *points.last().expect("tip"));
                        painter.line_segment([start, tip], Stroke::new(2.0, tint));
                        // Arrow head, so the handle reads as "push this way".
                        let direction = tip - start;
                        if direction.length() > 1.0 {
                            let unit = direction / direction.length();
                            let side = egui::vec2(-unit.y, unit.x) * 5.0;
                            painter.add(egui::Shape::convex_polygon(
                                vec![tip, tip - unit * 13.0 + side, tip - unit * 13.0 - side],
                                tint,
                                Stroke::NONE,
                            ));
                        }
                    }
                    Handle::Scale(_) => {
                        let (start, tip) = (points[0], *points.last().expect("tip"));
                        // Dotted leader crossing the pivot to a hollow box on the far side, as drawn in the
                        // Matrix 8 manual.
                        let dashes = dashed_line(start, tip, 4.0, 4.0);
                        for [a, b] in dashes {
                            painter.line_segment([a, b], Stroke::new(1.0, tint));
                        }
                        painter.rect_stroke(
                            Rect::from_center_size(tip, egui::vec2(8.0, 8.0)),
                            0.0,
                            Stroke::new(1.5, tint),
                            egui::StrokeKind::Inside,
                        );
                    }
                    Handle::MovePlane(plane) => {
                        // The "4" icon of the manual: a small grid standing for the two-axis plane, drawn in
                        // the colours of the two axes it moves along ("the two axises are indicated by their
                        // color").
                        let (first, second) = plane_colours(plane);
                        let icon = Rect::from_center_size(points[0], egui::vec2(9.0, 9.0));
                        painter.rect_stroke(
                            icon,
                            0.0,
                            Stroke::new(1.0, first),
                            egui::StrokeKind::Inside,
                        );
                        painter.line_segment(
                            [
                                egui::pos2(icon.center().x, icon.top()),
                                egui::pos2(icon.center().x, icon.bottom()),
                            ],
                            Stroke::new(1.0, second),
                        );
                        painter.line_segment(
                            [
                                egui::pos2(icon.left(), icon.center().y),
                                egui::pos2(icon.right(), icon.center().y),
                            ],
                            Stroke::new(1.0, second),
                        );
                    }
                    Handle::MoveFree => {}
                }
            }
        }
        // The centre of the gumball, as in the manual.
        if let Ok(Some(centre)) = active_centre(&views, &view.camera, viewport) {
            // "5 = Free Move Origin": a hollow square on the pivot with a dot inside.
            let origin = Rect::from_center_size(centre, egui::vec2(7.0, 7.0));
            painter.rect_filled(origin, 0.0, Color32::from_rgb(30, 30, 30));
            painter.rect_stroke(
                origin,
                0.0,
                Stroke::new(1.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );
            painter.circle_filled(centre, 1.6, Color32::WHITE);
        }
        self.paint_rotation_feedback(&painter, &view.camera, viewport, size);
        let Some(pointer) = response.interact_pointer_pos() else {
            return;
        };
        let pointer = [f64::from(pointer.x), f64::from(pointer.y)];
        if response.drag_started_by(egui::PointerButton::Primary) {
            let start = ui
                .input(|input| input.pointer.press_origin())
                .map(|p| [f64::from(p.x), f64::from(p.y)])
                .unwrap_or(pointer);
            let box_hit = views.iter().find_map(|group| {
                group
                    .box_at(&view.camera, viewport, start, 8.0)
                    .ok()
                    .flatten()
            });
            let hit = box_hit.or_else(|| {
                views
                    .iter()
                    .filter_map(|group| {
                        group
                            .nearest_in_view(&view.camera, viewport, start, 8.0)
                            .ok()
                            .flatten()
                    })
                    .min_by(|left, right| left.0.total_cmp(&right.0))
                    .map(|(_, handle)| handle)
            });
            if let Some(handle) = hit {
                let targets = self.controller.gumball_targets();
                match om9_core::gumball::Session::begin(frame, targets, modifiers.alt) {
                    Ok(session) => {
                        self.gumball_session = Some(session);
                        self.gumball_handle = Some(handle);
                        self.gumball_last = Some(start);
                        self.gumball_basis = Some(GumballBasis {
                            view: self.controller.workspace.active,
                            camera: view.camera.clone(),
                            viewport,
                            start,
                            epoch: self.controller.coordinator().epoch(),
                            selection: self.controller.selection_state().revision,
                            alt_held: modifiers.alt,
                        });
                    }
                    Err(error) => self.feedback(Err(error)),
                }
            }
        }
        if self.gumball_session.is_some()
            && (response.dragged_by(egui::PointerButton::Primary)
                || response.drag_stopped_by(egui::PointerButton::Primary))
        {
            let result = self.update_gumball(&view.camera, viewport, pointer, modifiers.alt);
            if let Err(error) = result {
                self.cancel_gumball();
                self.feedback(Err(error));
                return;
            }
            ui.ctx().request_repaint();
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(session) = self.gumball_session.take() {
                let name = match self.gumball_handle {
                    Some(Handle::Rotate(_)) => "Rotate",
                    Some(Handle::Scale(_)) => "Scale",
                    _ => "Move",
                };
                let result = if session.is_copy() {
                    self.controller.commit_gumball_copy(&session, name)
                } else {
                    self.controller.commit_gumball(&session, name)
                };
                self.controller.clear_gumball_preview();
                self.feedback(result);
            }
            self.gumball_handle = None;
            self.gumball_last = None;
            self.gumball_basis = None;
        }
    }

    fn cancel_gumball(&mut self) {
        self.gumball_session = None;
        self.gumball_handle = None;
        self.gumball_last = None;
        self.gumball_basis = None;
        self.controller.clear_gumball_preview();
    }

    fn update_gumball(
        &mut self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
        alt: bool,
    ) -> Result<()> {
        let basis = self
            .gumball_basis
            .as_mut()
            .ok_or(Error::Missing("gumball basis"))?;
        if basis.camera != *camera
            || basis.viewport != viewport
            || basis.view != self.controller.workspace.active
            || basis.epoch != self.controller.coordinator().epoch()
            || basis.selection != self.controller.selection_state().revision
        {
            return Err(Error::Stale("gumball gesture context"));
        }
        let session = self
            .gumball_session
            .as_mut()
            .ok_or(Error::Missing("gumball session"))?;
        if alt && !basis.alt_held {
            session.set_copy(!session.is_copy());
        }
        basis.alt_held = alt;
        let handle = self
            .gumball_handle
            .ok_or(Error::Missing("gumball handle"))?;
        let view = om9_scene::gumball_view::GumballView::new(*session.frame(), self.gumball_mode);
        if let Some(drag) =
            view.drag(&basis.camera, basis.viewport, handle, basis.start, pointer)?
        {
            session.reset();
            session.drag(handle, drag)?;
        }
        let delta = session.accumulated();
        self.gumball_last = Some(pointer);
        self.controller.preview_gumball(delta)
    }

    /// The sector in the reference clip shows the current angle, without leaving copies behind.
    fn paint_rotation_feedback(
        &self,
        painter: &egui::Painter,
        camera: &Camera,
        viewport: Viewport,
        size: f64,
    ) {
        use om9_core::gumball::{Axis, Drag, Handle, Plane};
        let (Some(session), Some(Handle::Rotate(axis)), Some(basis), Some(last)) = (
            &self.gumball_session,
            self.gumball_handle,
            &self.gumball_basis,
            self.gumball_last,
        ) else {
            return;
        };
        let source = *session.frame();
        let query = om9_scene::gumball_view::GumballView::new(source, self.gumball_mode);
        let Ok(Some(Drag::Angle(angle))) = query.drag(
            &basis.camera,
            basis.viewport,
            Handle::Rotate(axis),
            basis.start,
            last,
        ) else {
            return;
        };
        let plane = match axis {
            Axis::X => Plane::YZ,
            Axis::Y => Plane::ZX,
            Axis::Z => Plane::XY,
        };
        let (x, y) = source.plane_basis(plane);
        let Ok(Some(start)) = basis
            .camera
            .ray(basis.start, basis.viewport)
            .and_then(|ray| {
                ray.intersect_plane(om9_scene::camera::CPlane::new(source.origin, x, y)?)
            })
        else {
            return;
        };
        let v = std::array::from_fn::<_, 3, _>(|i| start[i] - source.origin[i]);
        let length = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if length <= 1e-12 {
            return;
        }
        let start = std::array::from_fn(|i| source.origin[i] + v[i] / length * size * 0.6);
        let project = |p| {
            camera
                .project(p, viewport)
                .ok()
                .flatten()
                .map(|p| egui::pos2(p.screen[0] as f32, p.screen[1] as f32))
        };
        let Some(centre) = project(source.origin) else {
            return;
        };
        let mut sector = vec![centre];
        for i in 0..=48 {
            if let Ok(p) = om9_core::gumball::rotation_about(
                source.origin,
                source.axis(axis),
                angle * i as f64 / 48.,
            )
            .and_then(|t| t.point(start))
                && let Some(p) = project(p)
            {
                sector.push(p);
            }
        }
        rotation_feedback::paint_sector(painter, &sector);
    }

    /// The `dock` part of the audit receipt: documented left order, right dock
    /// state and the left-stack evidence (title bars, scroll, widths, icons).
    fn dock_receipt(&self, _ctx: &egui::Context) -> serde_json::Value {
        let keys = |ids: Vec<SectionId>| -> Vec<&'static str> {
            ids.into_iter().map(SectionId::key).collect()
        };
        let left_sections = keys(self.dock.left_sections());
        let right_sections = keys(self.dock.right_sections());
        let floating_sections = keys(self.dock.floating_sections());
        let bar = |key: &str, rect: &Rect| {
            let column = SectionId::from_key(key)
                .map(|id| match self.dock.section(id).dock {
                    DockSide::Left => "left",
                    DockSide::Right => "right",
                    DockSide::Floating => "floating",
                    DockSide::Top => "top",
                    DockSide::Bottom => "bottom",
                })
                .unwrap_or("unknown");
            serde_json::json!({
                "x": rect.min.x, "y": rect.min.y, "w": rect.width(), "h": rect.height(),
                "column": column,
            })
        };
        let title_bars: BTreeMap<&str, serde_json::Value> = self
            .section_rects
            .iter()
            .map(|(key, rect)| (*key, bar(key, rect)))
            .collect();
        let layers_rect = self
            .section_rects
            .get(SectionId::Layers.key())
            .map(|rect| bar(SectionId::Layers.key(), rect));
        // Every recorded icon must sit inside the sidebar's horizontal extent.
        let icons_inside = self.sidebar_rect.is_some_and(|sidebar| {
            self.rail_icon_rects
                .iter()
                .all(|r| r.min.x >= sidebar.min.x - 0.5 && r.max.x <= sidebar.max.x + 0.5)
        });
        serde_json::json!({
            "left_sections": left_sections,
            "right_sections": right_sections,
            "floating_sections": floating_sections,
            "right_dock_visible": self.regions.right_dock_visible,
            "right_dock_sections": right_sections,
            "builder_slot_visible": self.builder_slot_rect.is_some()
                || self.dock.left_sections().contains(&SectionId::BuilderSlot),
            "left_rail_px": self.regions.left_rail,
            "left_rail_outer_px": self.left_rail_outer_px,
            "left_rail_inner_px": self.sidebar_rect.map(|r| r.width()),
            "left_icon_edge_px": self.left_icon_edge,
            "left_icon_count": self.rail_icon_rects.len(),
            "left_icon_rects_in_sidebar": icons_inside,
            "left_stack_content_px": self.sidebar_content_size.map(|s| [s.x, s.y]),
            "sidebar_scroll_offset": self.sidebar_scroll,
            "section_title_bars_px": title_bars,
            "layers_rect_px": layers_rect,
            "right_dock_outer_px": self.right_dock_outer_px,
            "command_line_px": self.regions.command_line,
            "status_bar_px": self.regions.status_bar,
            "status_kind": self.status.kind().label(),
            "status_message": self.status.message(),
            "status_tail": self.status.tail(),
            "diagnostics": self.dock_diagnostics,
            "panels": {
                "scenario": self.audit.as_ref().and_then(|audit| audit.panels.clone()),
                "checks": self.audit.as_ref().map(|audit| audit.panel_checks.clone()),
                "drag_in_progress": self.section_drag().is_some(),
                "work_area": self.work_area.map(|work| [work.x, work.y, work.width, work.height]),
                "last_op": self.last_panel_op.as_ref().map(|record| serde_json::json!({
                    "op": record.op,
                    "ok": record.ok,
                    "outcome": record.report.as_ref().map(|report| &report.outcome),
                    "notes": record.report.as_ref().map(|report| &report.notes),
                    "error": record.error,
                })),
                "drop_preview": self.drop_preview.map(|preview| serde_json::json!({
                    "section": preview.section.key(),
                    "target": preview.target,
                    "indicator": preview.indicator,
                })),
            },
        })
    }
    fn audit_receipt(&mut self, ctx: &egui::Context) {
        let dock_receipt = if self
            .audit
            .as_ref()
            .is_some_and(|audit| audit.awaiting_screenshot && !audit.receipt_written)
        {
            Some(self.dock_receipt(ctx))
        } else {
            None
        };
        let Some(audit) = &mut self.audit else {
            return;
        };
        if audit.receipt_written {
            if !audit.hold
                || audit.directory.join("release").exists()
                || audit.started.elapsed().as_secs() > 60
            {
                self.audit_window_close(ctx);
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
            return;
        }
        if audit.layer_dialogs {
            ctx.input(|input| {
                for event in &input.events {
                    if let egui::Event::Screenshot {
                        image, user_data, ..
                    } = event
                    {
                        let tag = user_data
                            .data
                            .as_ref()
                            .and_then(|data| data.downcast_ref::<&'static str>())
                            .copied();
                        if let Some(tag @ ("rename" | "color" | "selective-show")) = tag {
                            let started=std::time::Instant::now();
                            let pixels: Vec<u8> = image
                                .pixels
                                .iter()
                                .flat_map(|color| color.to_array())
                                .collect();
                            let result = image::save_buffer(
                                audit.directory.join(format!("dialog-{tag}.png")),
                                &pixels,
                                image.size[0] as u32,
                                image.size[1] as u32,
                                image::ColorType::Rgba8,
                            );
                            audit.dialog_capture_metrics.push(serde_json::json!({"dialog":tag,
                                "saved":result.is_ok(),"encoding_ms":started.elapsed().as_millis()}));
                            if audit.dialog_capture_pending==Some(tag) { audit.dialog_capture_pending=None; }
                        }
                    }
                }
            });
        }
        if audit.selection {
            ctx.input(|input| {
                for event in &input.events {
                    if let egui::Event::Screenshot {
                        image, user_data, ..
                    } = event
                        && let Some(tag) = user_data
                            .data
                            .as_ref()
                            .and_then(|data| data.downcast_ref::<&'static str>())
                        && matches!(
                            *tag,
                            "m02-move" | "m02-scale" | "m02-rotate" | "m02-mesh" | "m02-occurrence"
                        )
                    {
                        let pixels: Vec<u8> =
                            image.pixels.iter().flat_map(|c| c.to_array()).collect();
                        let _ = image::save_buffer(
                            audit.directory.join(format!("{tag}.png")),
                            &pixels,
                            image.size[0] as u32,
                            image.size[1] as u32,
                            image::ColorType::Rgba8,
                        );
                    }
                }
            });
        }
        if !audit.awaiting_screenshot {
            return;
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Screenshot {
                    image, user_data, ..
                } = e
                {
                    if user_data.data.is_some() {
                        return None;
                    }
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(image) = image {
            let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
            let path = audit.directory.join("workspace.png");
            let result = image::save_buffer(
                &path,
                &pixels,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ColorType::Rgba8,
            );
            let mut receipt = serde_json::json!({"scope":"initial_standalone_ui_slice","adapter":self.adapter,"passes":self.passes,"objects":self.controller.document().entities.len(),"document":self.controller.document(),"undo":self.controller.coordinator().undo_len(),"checks":audit.checks,"screenshot":path,"screenshot_saved":result.is_ok(),"view_rects":self.view_rects.iter().map(|(id,r)|(id.title(),[r.min.x,r.min.y,r.width(),r.height()])).collect::<BTreeMap<_,_>>(),"cmd":self.transcript.text(),"limitations":["full Matrix9 UI acceptance pending","CAD native adapters pending","GPU geometry caching and performance acceptance pending","DPI/multi-monitor acceptance pending"]});
            // Menu/docking evidence for OM9-L2-01-01: the documented left column
            // order, menus docked to the right, the reserved Builder area, the
            // left-stack geometry and the always-visible status surface.
            receipt["dock"] = dock_receipt.unwrap_or(serde_json::Value::Null);
            // Menu placement (OM9-L3-01-01-003): the scripted drag scenario, the
            // preview that the release applied, the last operation and the
            // resulting columns.
            receipt["panels"] = receipt["dock"]["panels"].clone();
            if audit.selection {
                receipt["scope"] = serde_json::json!("M02 OM9-L2-01-07 native widget replay");
                receipt["selection_replay"] = serde_json::json!(audit.selection_report);
                receipt["human_approval"] = serde_json::Value::Null;
                receipt["scenario"] = serde_json::json!(if audit.occurrences {
                    "occurrences"
                } else {
                    "selection"
                });
                receipt["gpu_scene_views"] = serde_json::json!(
                    self.gpu_drawn_views
                        .iter()
                        .map(|id| id.title())
                        .collect::<Vec<_>>()
                );
            }
            receipt["audit"] = serde_json::json!({
                "layout_mode": self.audit_layout_mode.key(),
                "mode_label": self.audit_layout_mode.label(),
                "rail_width_override": self.audit_rail_width,
            });
            // Workbench docking regions (OM9-L3-01-01-002): the validated plan
            // for the audited window, or the explicit reason there is none.
            receipt["regions"] = serde_json::json!({
                "window": self.dock_window,
                "zoom_factor": ctx.zoom_factor(),
                "pixels_per_point": ctx.pixels_per_point(),
                "min_window": {
                    "sent": self.min_window_sent,
                    "monitor_points": self.min_window_monitor,
                    "source": self.min_window_source,
                },
                "screen_points": [ctx.content_rect().width(), ctx.content_rect().height()],
                "plan": self.region_plan,
                "error": self.region_error.as_ref().map(ToString::to_string),
                "sidebar_px": self.sidebar_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "right_dock_px": self.right_dock_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "command_input_px": self.cmd_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "status_message_px": self.status_message_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "status_summary_px": self.status_summary_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "status_text_overlap": match (self.status_message_rect, self.status_summary_rect) {
                    (Some(message), Some(summary)) => message.max.x > summary.min.x + 0.5,
                    _ => false,
                },
            });
            if audit.viewport_menu {
                receipt["scope"] = serde_json::json!("viewport_title_menu_slice");
                receipt["viewport_replay"] = serde_json::json!(audit.viewport_replay);
                receipt["viewport_popup_open"] = serde_json::json!(egui::Popup::is_any_open(ctx));
            }
            if audit.layers {
                receipt["scope"] = serde_json::json!("owned_layers_widgets_current_cmd_slice");
                receipt["layer_clicks"] = serde_json::json!(audit.layer_clicks);
            }
            if audit.layer_dialogs {
                receipt["scope"] = serde_json::json!("owned_layer_dialogs_slice");
                receipt["modal_open"] = serde_json::json!(self.controller.layer_dialog().is_some());
                receipt["dialog_replay"] = serde_json::json!(audit.viewport_replay);
                receipt["dialog_clicks"] = serde_json::json!(audit.layer_clicks);
                receipt["dialog_capture_metrics"] = serde_json::json!(audit.dialog_capture_metrics);
            }
            if audit.files {
                receipt["scope"] = serde_json::json!("owned_file_native_cancel_slice");
                receipt["file_picker_receipts"] = serde_json::json!(audit.file_picker_receipts);
                receipt["file_operation_open"] =
                    serde_json::json!(self.controller.file_operation().is_some());
                receipt["active_file_path"] = serde_json::json!(self.controller.active_file_path());
                receipt["draft"] = serde_json::json!(self.transcript.draft());
                receipt["file_replay"] = serde_json::json!(audit.viewport_replay);
                receipt["native_close_events"] = serde_json::json!(audit.file_close_events);
                if audit.file_close {
                    receipt["scope"] = serde_json::json!("owned_file_native_close_cancel_slice");
                }
                if audit.file_success {
                    receipt["scope"] = serde_json::json!("owned_file_native_selection_slice");
                    receipt["saved_document_id"] = serde_json::json!(audit.file_success_document);
                    receipt["original_saved_archive_sha256"] =
                        serde_json::json!(audit.file_success_original_sha);
                    receipt["seeded_overwrite_archive_sha256"] =
                        serde_json::json!(audit.file_success_seed_sha);
                }
            }
            if audit.startup_file {
                receipt["scope"] = serde_json::json!("owned_native_file_startup_slice");
                receipt["active_file_path"] = serde_json::json!(self.controller.active_file_path());
                receipt["file_diagnostics"] = serde_json::json!(self.controller.file_diagnostics());
                receipt["window_title"] = serde_json::json!(self.window_title);
                let mut report = Vec::new();
                if crate::report::write(
                    &mut report,
                    self.controller.document(),
                    self.controller.file_diagnostics(),
                    self.controller.coordinator().limits(),
                )
                .is_ok()
                {
                    receipt["startup_persistent_report"] =
                        serde_json::from_slice(&report).expect("bounded report JSON");
                } else {
                    receipt["checks"]
                        .as_array_mut()
                        .expect("checks")
                        .push(serde_json::json!("FAILED startup persistent report"));
                }
            }
            let _ = std::fs::write(
                audit.directory.join("ui-receipt.json"),
                serde_json::to_vec_pretty(&receipt).expect("receipt"),
            );
            audit.receipt_written = true;
            if !audit.hold {
                self.audit_window_close(ctx);
            } else {
                ctx.request_repaint();
            }
        } else {
            ctx.request_repaint();
        }
    }
}
/// F4 jewelry datum: world (0,0,0) and the ring centre, drawn as a small
/// crosshair plus ring. It is display-only and never moves any datum coordinate.
fn paint_world_datum(painter: &egui::Painter, camera: &Camera, viewport: Viewport, rect: Rect) {
    let Ok(Some(origin)) = camera.project([0.; 3], viewport) else {
        return;
    };
    let center = egui::pos2(origin.screen[0] as f32, origin.screen[1] as f32);
    if !rect.contains(center) {
        return;
    }
    let color = Color32::from_rgb(255, 170, 40);
    let ring: Vec<_> = (0..24)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 24.;
            center + Vec2::new(6. * a.cos(), 6. * a.sin())
        })
        .collect();
    painter.add(egui::Shape::closed_line(ring, Stroke::new(1.2, color)));
    painter.line_segment(
        [center - Vec2::new(11., 0.), center + Vec2::new(11., 0.)],
        Stroke::new(1., color),
    );
    painter.line_segment(
        [center - Vec2::new(0., 11.), center + Vec2::new(0., 11.)],
        Stroke::new(1., color),
    );
    painter.circle_filled(center, 1.6, color);
}

fn paint_axis_triad(painter: &egui::Painter, camera: &Camera, viewport: Viewport, rect: Rect) {
    let anchor = egui::pos2(rect.left() + 30., rect.bottom() - 28.);
    let Ok(Some(origin)) = camera.project([0.; 3], viewport) else {
        return;
    };
    for (p, label, color) in [
        ([1., 0., 0.], "x", Color32::RED),
        ([0., 1., 0.], "y", Color32::from_rgb(0, 190, 210)),
        ([0., 0., 1.], "z", Color32::from_rgb(235, 205, 0)),
    ] {
        let Ok(Some(endpoint)) = camera.project(p, viewport) else {
            continue;
        };
        let direction = Vec2::new(
            (endpoint.screen[0] - origin.screen[0]) as f32,
            (endpoint.screen[1] - origin.screen[1]) as f32,
        );
        if direction.length_sq() < 1e-6 {
            continue;
        }
        let endpoint = anchor + direction.normalized() * 24.;
        painter.line_segment([anchor, endpoint], Stroke::new(1., color));
        painter.text(
            endpoint,
            egui::Align2::LEFT_BOTTOM,
            label,
            egui::FontId::proportional(10.),
            color,
        );
    }
}
fn paint_view_marker(painter: &egui::Painter, rect: Rect, id: ViewId) {
    let center = rect.right_top() + Vec2::new(-29., 31.);
    let (squash, angle): (f32, f32) = match id {
        ViewId::LookingDown => (0.28, 0.),
        ViewId::ThroughFinger => (1., 0.),
        ViewId::SideView => (0.22, -std::f32::consts::FRAC_PI_2),
        ViewId::Perspective => (0.7, -0.7),
    };
    let ring: Vec<_> = (0..32)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 32.;
            let x = 10. * a.cos();
            let y = 10. * squash * a.sin();
            center
                + Vec2::new(
                    x * angle.cos() - y * angle.sin(),
                    x * angle.sin() + y * angle.cos(),
                )
        })
        .collect();
    painter.add(egui::Shape::closed_line(
        ring,
        Stroke::new(1.2, Color32::from_rgb(245, 193, 0)),
    ));
    let gem = center + Vec2::new(0., -12. * squash);
    painter.add(egui::Shape::closed_line(
        vec![
            gem + Vec2::new(-4., 0.),
            gem + Vec2::new(-2., -3.),
            gem + Vec2::new(2., -3.),
            gem + Vec2::new(4., 0.),
            gem + Vec2::new(0., 4.),
        ],
        Stroke::new(0.8, Color32::from_rgb(61, 144, 230)),
    ));
}
/// World units per screen pixel at the target distance, so a pan follows the cursor's scale.
fn world_per_pixel(camera: &Camera, viewport: Viewport) -> f64 {
    let height = viewport.size[1].max(1.0);
    match camera.projection {
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => {
            let offset = [
                camera.eye[0] - camera.target[0],
                camera.eye[1] - camera.target[1],
                camera.eye[2] - camera.target[2],
            ];
            let distance =
                (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
            2.0 * (vertical_fov_rad * 0.5).tan() * distance / height
        }
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => {
            vertical_span_mm / height
        }
    }
}

fn paint_line(
    painter: &egui::Painter,
    camera: &Camera,
    viewport: Viewport,
    a: Point3,
    b: Point3,
    color: [u8; 4],
    width: f32,
) {
    if let (Ok(Some(a)), Ok(Some(b))) = (camera.project(a, viewport), camera.project(b, viewport)) {
        painter.line_segment(
            [
                egui::pos2(a.screen[0] as f32, a.screen[1] as f32),
                egui::pos2(b.screen[0] as f32, b.screen[1] as f32),
            ],
            Stroke::new(
                width,
                Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]),
            ),
        );
    }
}
impl eframe::App for MatrixApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.service_window_close(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.show(ui);
        self.service_file_close_probe(ui.ctx(), frame);
        self.service_native_file_picker(ui.ctx(), frame);
        self.service_three_dm_picker(ui.ctx(), frame);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.audit.as_ref().is_some_and(|audit| audit.occurrences) && self.passes >= 2 {
            self.occurrence_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|audit| audit.selection) && self.passes >= 2 {
            self.selection_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|audit| audit.startup_file) && self.passes >= 2 {
            self.startup_file_audit_input(ctx);
            return;
        }
        if self.audit.as_ref().is_some_and(|audit| audit.file_success) && self.passes >= 2 {
            self.file_success_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|audit| audit.files) && self.passes >= 2 {
            self.file_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|a| a.layer_dialogs) && self.passes >= 2 {
            self.layer_dialog_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|a| a.layers) && self.passes >= 2 {
            self.layer_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|a| a.panels.is_some()) && self.passes >= 2 {
            self.panel_audit_input(ctx, raw);
            return;
        }
        if self.audit.as_ref().is_some_and(|a| a.viewport_menu) && self.passes >= 2 {
            self.viewport_menu_audit_input(ctx, raw);
            return;
        }
        let Some(audit) = &mut self.audit else {
            return;
        };
        if self.passes < 2 {
            ctx.request_repaint();
            return;
        }
        // The scripted text goes to the focused CMD field at its cursor, so a
        // step must not run before the field has focus with the cursor at the end
        // of the transcript. Two races drop typed text otherwise: an early `Text`
        // event before focus, and a history line appended in the same frame
        // (for example a layout diagnostic), which leaves the stored cursor
        // inside the protected history so `Transcript::insert` refuses the text.
        // `focus_cmd` makes the CMD frame put the cursor back at the end. A field
        // that never gets there is reported and the script continues, so the
        // failure shows instead of hanging.
        if audit.step <= 14 && !audit.cmd_focus_failed {
            let id = egui::Id::new("matrix-command-transcript");
            let at_end = TextEdit::load_state(ctx, id)
                .and_then(|state| state.cursor.char_range())
                .is_some_and(|range| {
                    range == CCursorRange::one(CCursor::new(self.transcript.text().chars().count()))
                });
            if ctx.memory(|m| m.has_focus(id)) && at_end {
                audit.cmd_focus_waits = 0;
                if !audit.cmd_focus_ready {
                    audit.cmd_focus_ready = true;
                    audit.checks.push("audit CMD focus ready".into());
                }
            } else if audit.cmd_focus_waits < AUDIT_CMD_FOCUS_MAX_PASSES {
                audit.cmd_focus_waits += 1;
                self.focus_cmd = true;
                ctx.request_repaint();
                return;
            } else {
                audit.cmd_focus_failed = true;
                audit.checks.push("FAILED audit CMD focus".into());
            }
        } // Explicit opt-in own-app event replay through the normal egui input
        // path. It never controls another process or desktop input devices.
        let step = audit.step;
        audit.step += 1;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        match step {
            0 => {
                raw.events.push(egui::Event::Text("Line".into()));
            }
            1 => raw.events.push(key(Key::Enter)),
            2 => raw.events.push(egui::Event::Text("0,0,0".into())),
            3 => raw.events.push(key(Key::Enter)),
            4 => raw.events.push(egui::Event::Text("12,6,0".into())),
            5 => raw.events.push(key(Key::Enter)),
            6 => {
                if self.controller.document().entities.len() == 1
                    && self.controller.coordinator().undo_len() == 1
                {
                    audit.checks.push("CMD line accepted once".into());
                }
                raw.events.push(egui::Event::Text("Undo".into()));
            }
            7 => raw.events.push(key(Key::Enter)),
            8 => {
                if self.controller.document().entities.is_empty()
                    && self.controller.coordinator().redo_len() == 1
                {
                    audit.checks.push("actual CMD Undo".into());
                }
                raw.events.push(egui::Event::Text("Redo".into()));
            }
            9 => raw.events.push(key(Key::Enter)),
            10 => {
                if self.controller.document().entities.len() == 1 {
                    audit.checks.push("actual CMD Redo".into());
                }
                raw.events.push(egui::Event::Text("Line".into()));
            }
            11 => raw.events.push(key(Key::Enter)),
            12 => raw.events.push(egui::Event::Text("2,2,0".into())),
            13 => raw.events.push(key(Key::Enter)),
            14 => raw.events.push(key(Key::Escape)),
            15 => {
                if self.controller.document().entities.len() == 1
                    && !self.controller.is_running()
                    && self.controller.coordinator().undo_len() == 1
                {
                    audit
                        .checks
                        .push("Esc canceled draft without publication".into());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                audit.awaiting_screenshot = true;
            }
            _ => {}
        }
        ctx.request_repaint();
    }
}
/// Passes the audit script waits for the CMD field to take focus.
const AUDIT_CMD_FOCUS_MAX_PASSES: u32 = 30;
pub fn launch(document: Document, audit: Option<PathBuf>) -> Result<()> {
    launch_classified(document, audit).map_err(|failure| Error::Io(failure.line()))
}
pub fn launch_file(path: &std::path::Path) -> Result<()> {
    launch_file_classified(path).map_err(|failure| Error::Io(failure.line()))
}
/// Launch with an explicit failure stage so a start that cannot restore the
/// document is never reported as a renderer fault and vice versa (L3-001).
pub fn launch_classified(
    document: Document,
    audit: Option<PathBuf>,
) -> std::result::Result<(), crate::startup::StartupFailure> {
    use crate::startup::{AppConfig, StartupFailure, StartupStage};
    let mut config = AppConfig {
        document_name: document.name.clone(),
        audit_dir: audit,
        ..AppConfig::interactive("Untitled", None)
    };
    // Audit-only window size (L3-002 runtime evidence at a narrow width). An
    // unusable value is a visible startup failure, never ignored.
    if config.audit_dir.is_some()
        && let Some(text) = std::env::var_os("OM9_UI_AUDIT_WINDOW")
    {
        config.window = text.to_str().and_then(parse_window_size).ok_or_else(|| {
            StartupFailure::new(
                StartupStage::Renderer,
                "OM9_UI_AUDIT_WINDOW must look like 800x600",
            )
        })?;
        config.maximized = false;
    }
    // Audit-only: the window is given in physical pixels at the native scale,
    // so the accepted minimum follows the simulated zoom and monitor.
    if config.audit_dir.is_some() {
        let fail = |detail: &str| StartupFailure::new(StartupStage::Renderer, detail);
        let zoom = audit_zoom_from_env().map_err(fail)?;
        let monitor = audit_monitor_from_env().map_err(fail)?;
        config.min_window = audit_min_window(monitor, zoom);
    }
    let app = MatrixApp::start(&config)?;
    launch_with(app, &config)
        .map_err(|error| StartupFailure::from_error(StartupStage::Renderer, &error))
}
/// Parse `WIDTHxHEIGHT` (logical pixels, finite and positive).
fn parse_window_size(text: &str) -> Option<[f32; 2]> {
    let (width, height) = text.split_once('x')?;
    let size = [
        width.trim().parse::<f32>().ok()?,
        height.trim().parse::<f32>().ok()?,
    ];
    size.iter()
        .all(|v| v.is_finite() && *v > 0.)
        .then_some(size)
}
/// Parse the audit-only display-scale simulation: finite, 0.5 to 4.0 inclusive.
fn parse_zoom_factor(text: &str) -> Option<f32> {
    let zoom = text.trim().parse::<f32>().ok()?;
    (zoom.is_finite() && (0.5..=4.0).contains(&zoom)).then_some(zoom)
}
/// `OM9_UI_AUDIT_ZOOM`: an unusable value is a visible failure, never ignored.
/// Callers read this only for an audit run.
fn audit_zoom_from_env() -> std::result::Result<Option<f32>, &'static str> {
    match std::env::var_os("OM9_UI_AUDIT_ZOOM") {
        Some(text) => text
            .to_str()
            .and_then(parse_zoom_factor)
            .map(Some)
            .ok_or("OM9_UI_AUDIT_ZOOM must be a number from 0.5 to 4.0"),
        None => Ok(None),
    }
}
/// `OM9_UI_AUDIT_MONITOR=WxH`: a simulated monitor in physical pixels. Callers
/// read this only for an audit run.
fn audit_monitor_from_env() -> std::result::Result<Option<[f32; 2]>, &'static str> {
    match std::env::var_os("OM9_UI_AUDIT_MONITOR") {
        Some(text) => text
            .to_str()
            .and_then(parse_window_size)
            .map(Some)
            .ok_or("OM9_UI_AUDIT_MONITOR must look like 1366x768"),
        None => Ok(None),
    }
}
/// Smallest window (physical pixels at the native scale) an audit run accepts:
/// the effective minimum for the simulated monitor, scaled by the simulated zoom.
fn audit_min_window(monitor_px: Option<[f32; 2]>, zoom: Option<f32>) -> [f32; 2] {
    let scale = zoom.unwrap_or(1.);
    let monitor_points = monitor_px.map(|px| [px[0] / scale, px[1] / scale]);
    let minimum = crate::startup::effective_min_size(monitor_points);
    [minimum[0] * scale, minimum[1] * scale]
}
pub fn launch_file_classified(
    path: &std::path::Path,
) -> std::result::Result<(), crate::startup::StartupFailure> {
    use crate::startup::{AppConfig, StartupFailure, StartupStage};
    // Explicit own-app audit retains the normal --ui path constructor and
    // native window. No synthetic startup document or input events are used.
    let audit = std::env::var_os("OM9_UI_AUDIT_STARTUP_DIRECTORY").map(PathBuf::from);
    let mut config = AppConfig::interactive("Untitled", audit);
    let app = MatrixApp::from_file(path)
        .map_err(|error| StartupFailure::from_error(StartupStage::Document, &error))?;
    config.document_name = app.controller.document().name.clone();
    launch_with(app, &config)
        .map_err(|error| StartupFailure::from_error(StartupStage::Renderer, &error))
}
/// Owned startup diagnostic: the native dialog is shown only when the caller
/// decides to, and never for audits or automation.
pub fn report_startup_failure(failure: &crate::startup::StartupFailure) -> bool {
    if !crate::startup::should_report_dialog(
        std::env::var(crate::startup::StartupFailure::DIALOG_ENV)
            .ok()
            .as_deref(),
    ) {
        return false;
    }
    om9_native_owner::report_startup_failure("OpenMatrix9", &failure.line())
}
/// Run one window from a validated configuration: title, size, minimum size,
/// audit directory and adapter explanation all come from the record.
fn launch_with(mut app: MatrixApp, config: &crate::startup::AppConfig) -> Result<()> {
    if let Some(directory) = &config.audit_dir {
        std::fs::create_dir_all(directory).map_err(|e| Error::Io(e.to_string()))?;
    }
    app.audit = config.audit_dir.clone().map(UiAudit::new);
    if app.dock_layout_path().is_none()
        && let Some(path) = &config.layout_path
    {
        app.set_dock_layout_path(Some(path.clone()));
        app.load_dock_layout();
    }
    // Audit-only layout mode, applied without touching the stored layout
    // (nothing is saved unless the user runs Set as Default). The default docks
    // nothing right, like the Matrix9 screenshot. A bad value or the removed
    // OM9_UI_AUDIT_RIGHT_DOCK is a visible startup failure, never ignored.
    if app.audit.is_some() {
        if std::env::var_os("OM9_UI_AUDIT_RIGHT_DOCK").is_some() {
            return Err(Error::Io(
                "OM9_UI_AUDIT_RIGHT_DOCK was replaced by OM9_UI_AUDIT_LAYOUT_MODE=user-docked-right"
                    .into(),
            ));
        }
        let mode_text = std::env::var_os("OM9_UI_AUDIT_LAYOUT_MODE");
        let mode = match &mode_text {
            None => parse_audit_layout_mode(None),
            Some(text) => parse_audit_layout_mode(text.to_str()),
        }
        .map_err(|detail| Error::Io(detail.into()))?;
        app.audit_layout_mode = mode;
        if mode == AuditLayoutMode::UserDockedRight {
            app.dock_menu_section(dock_layout::RIGHT_DOCK_FIXTURE, DockSide::Right);
        }
        // The scripted menu-drag scenario: an unusable value is a visible
        // startup failure, never a silent no-op.
        if let Some(text) = std::env::var_os("OM9_UI_AUDIT_PANELS") {
            let scenario = text.to_str().and_then(parse_audit_panels).ok_or_else(|| {
                Error::Io(format!(
                    "OM9_UI_AUDIT_PANELS must be one of {}",
                    PANEL_AUDIT_SCENARIOS.join(", ")
                ))
            })?;
            app.audit.as_mut().expect("audit").panels = Some(scenario);
        }
        if let Some(text) = std::env::var_os("OM9_UI_AUDIT_RAIL_WIDTH") {
            let width = text
                .to_str()
                .and_then(parse_audit_rail_width)
                .ok_or_else(|| {
                    Error::Io(format!(
                        "OM9_UI_AUDIT_RAIL_WIDTH must be a number from {} to {}",
                        dock_layout::LEFT_RAIL_MIN,
                        dock_layout::LEFT_RAIL_MAX
                    ))
                })?;
            // Applied before the first frame and never saved (no dock_dirty).
            app.dock.set_size(PanelId::LeftRail, width);
            app.audit_rail_width = Some(width);
        }
    }
    // Audit-only: simulate a display scale by zooming egui and a monitor size.
    // An unusable value is a visible startup failure, never ignored, and a
    // normal start never reads either variable.
    let audit_zoom = if app.audit.is_some() {
        audit_zoom_from_env().map_err(|detail| Error::Io(detail.into()))?
    } else {
        None
    };
    if app.audit.is_some() {
        app.audit_monitor_px =
            audit_monitor_from_env().map_err(|detail| Error::Io(detail.into()))?;
    }
    let title = config.window_title().to_owned();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(config.window)
            .with_maximized(config.maximized && app.audit.is_none())
            .with_min_inner_size(config.builder_min_window())
            .with_title(&title)
            .with_icon(egui::IconData::default()),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            if let Some(zoom) = audit_zoom {
                cc.egui_ctx.set_zoom_factor(zoom);
            }
            app.configure_style(&cc.egui_ctx);
            let state = cc
                .wgpu_render_state
                .as_ref()
                .ok_or_else(|| std::io::Error::other("A hardware GPU adapter is required"))?;
            let info = state.adapter.get_info();
            app.adapter = crate::startup::gpu_adapter_identity(Some((
                &format!("{info:?}"),
                info.device_type,
            )))?;
            app.gpu_state = Some(state.clone());
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| Error::Io(e.to_string()))
}

fn mesh_stress_document(mut document: Document) -> Document {
    use om9_core::geometry::{Geometry, Mesh};
    use om9_core::model::Entity;
    let count = match std::env::var("OM9_MESH_STRESS") {
        Ok(value) => value.trim().parse::<usize>().unwrap_or(0),
        Err(_) => 0,
    };
    if count == 0 {
        return document;
    }
    let sphere = om9_render::shapes::sphere(24, 12, 0.6);
    let side = (count as f32).cbrt().ceil().max(1.0) as i32;
    for index in 0..count {
        let x = (index as i32 % side) as f64 - f64::from(side) * 0.5;
        let y = ((index as i32 / side) % side) as f64 - f64::from(side) * 0.5;
        let z = (index as i32 / (side * side)) as f64 - f64::from(side) * 0.5;
        let mesh = Mesh {
            vertices: sphere
                .positions
                .iter()
                .map(|p| [p[0] as f64, p[1] as f64, p[2] as f64])
                .collect(),
            faces: sphere
                .indices
                .chunks(3)
                .map(|corners| corners.to_vec())
                .collect(),
            normals: Vec::new(),
            colors: Vec::new(),
        };
        let mut entity = Entity::new("Mesh", Geometry::Mesh(mesh), document.current_layer);
        entity.placement = om9_core::geometry::Transform::translation([x * 1.6, y * 1.6, z * 1.6]);
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

/// Draws the frame rate in the viewport corner when `OM9_GPU_FPS` is set.
/// the image cannot cover the text.
fn paint_fps(ui: &egui::Ui, rect: Rect) {
    if std::env::var("OM9_GPU_FPS").is_err() {
        return;
    }
    let frame_seconds = ui.input(|input| input.stable_dt).max(1e-4);
    ui.painter().text(
        rect.left_top() + Vec2::new(8.0, 24.0),
        egui::Align2::LEFT_TOP,
        format!(
            "{:.0} fps / {:.1} ms",
            1.0 / frame_seconds,
            frame_seconds * 1000.0
        ),
        egui::FontId::monospace(12.0),
        Color32::from_rgb(160, 255, 160),
    );
}

/// Colour of one gumball handle. Matrix 8 manual, "The Gumball": Move X is the red arrow, Move Y the blue
/// arrow, Move Z the yellow arrow; arcs and scale boxes carry the same axis colour so the axis is readable
/// from the handle alone.
fn gumball_colour(handle: om9_core::gumball::Handle) -> Color32 {
    use om9_core::gumball::Handle;
    match handle {
        Handle::MoveAxis(axis) | Handle::Scale(axis) | Handle::Rotate(axis) => axis_colour(axis),
        Handle::MovePlane(plane) => plane_colours(plane).0,
        Handle::MoveFree => Color32::from_rgb(245, 245, 245),
    }
}

/// The colour of one world axis.
fn axis_colour(axis: om9_core::gumball::Axis) -> Color32 {
    use om9_core::gumball::Axis;
    match axis {
        Axis::X => Color32::from_rgb(205, 60, 50),
        Axis::Y => Color32::from_rgb(60, 150, 200),
        Axis::Z => Color32::from_rgb(235, 180, 50),
    }
}

/// The two axis colours of a plane handle, first for the border and second for the inner lines.
fn plane_colours(plane: om9_core::gumball::Plane) -> (Color32, Color32) {
    use om9_core::gumball::{Axis, Plane};
    let (first, second) = match plane {
        Plane::XY => (Axis::X, Axis::Y),
        Plane::YZ => (Axis::Y, Axis::Z),
        Plane::ZX => (Axis::Z, Axis::X),
    };
    (axis_colour(first), axis_colour(second))
}

/// Pick tolerance in screen pixels for the current scale factor: six points, so a high-DPI screen keeps the
/// same physical tolerance instead of becoming harder to click.
fn pick_tolerance(ui: &egui::Ui) -> f64 {
    // Camera, viewport and pointer positions are all in logical points. Egui applies DPI once.
    let _ = ui;
    6.0
}

/// Shift and Ctrl, where Ctrl is accepted as either the physical key or egui's logical command.
fn ctrl_or_command(ui: &egui::Ui) -> (bool, bool) {
    ui.input(|input| {
        (
            input.modifiers.shift,
            input.modifiers.ctrl || input.modifiers.command,
        )
    })
}

/// The gumball's centre in screen space, taken from the first view that can project it.
fn active_centre(
    views: &[om9_scene::gumball_view::GumballView],
    camera: &om9_scene::camera::Camera,
    viewport: Viewport,
) -> om9_core::Result<Option<egui::Pos2>> {
    for view in views {
        if let Some(point) = view.screen_point(camera, viewport, view.frame.origin)? {
            return Ok(Some(egui::pos2(point[0] as f32, point[1] as f32)));
        }
    }
    Ok(None)
}

/// Splits a screen-space line into dash segments of `dash` pixels separated by `gap` pixels.
fn dashed_line(from: egui::Pos2, to: egui::Pos2, dash: f32, gap: f32) -> Vec<[egui::Pos2; 2]> {
    let total = from.distance(to);
    if total <= f32::EPSILON {
        return Vec::new();
    }
    let unit = (to - from) / total;
    let mut dashes = Vec::new();
    let mut position = 0.0f32;
    while position < total {
        let end = (position + dash).min(total);
        dashes.push([from + unit * position, from + unit * end]);
        position = end + gap;
    }
    dashes
}

#[cfg(test)]
mod tests {
    use super::{
        AuditLayoutMode, MatrixApp, audit_min_window, parse_audit_layout_mode,
        parse_audit_rail_width, parse_window_size, parse_zoom_factor,
    };
    use crate::dock_layout::SectionId;
    use eframe::egui;
    use om9_core::model::Document;

    #[test]
    fn audit_layout_mode_parses_only_the_two_known_modes() {
        assert_eq!(parse_audit_layout_mode(None), Ok(AuditLayoutMode::Default));
        assert_eq!(
            parse_audit_layout_mode(Some("default")),
            Ok(AuditLayoutMode::Default)
        );
        assert_eq!(
            parse_audit_layout_mode(Some("user-docked-right")),
            Ok(AuditLayoutMode::UserDockedRight)
        );
        for bad in ["", "Default", "right", "1", "user-docked-right ", "layers"] {
            assert!(parse_audit_layout_mode(Some(bad)).is_err(), "{bad:?}");
        }
        assert_ne!(
            AuditLayoutMode::Default.label(),
            AuditLayoutMode::UserDockedRight.label()
        );
    }

    #[test]
    fn audit_rail_width_is_finite_and_inside_the_rail_bounds() {
        assert_eq!(parse_audit_rail_width("240"), Some(240.));
        assert_eq!(parse_audit_rail_width(" 460 "), Some(460.));
        for bad in ["", "239.9", "460.1", "NaN", "inf", "-300", "abc", "3e2x"] {
            assert_eq!(parse_audit_rail_width(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn dock_receipt_contains_the_left_stack_evidence() {
        let mut app = MatrixApp::new(Document::new("receipt")).unwrap();
        let context = egui::Context::default();
        for _ in 0..3 {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(1280., 720.),
                    )),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            output.textures_delta.clear();
        }
        let dock = app.dock_receipt(&context);
        for key in [
            "section_title_bars_px",
            "layers_rect_px",
            "sidebar_scroll_offset",
            "left_rail_outer_px",
            "left_rail_inner_px",
            "left_icon_edge_px",
            "left_icon_rects_in_sidebar",
            "left_stack_content_px",
            "right_dock_visible",
            "right_dock_sections",
        ] {
            assert!(dock.get(key).is_some(), "missing {key}");
        }
        assert_eq!(dock["right_dock_visible"], false);
        assert_eq!(dock["right_dock_sections"], serde_json::json!([]));
        assert_eq!(
            dock["left_sections"],
            serde_json::json!([
                "icon-history",
                "main-menu",
                "display",
                "snaps",
                "info-settings",
                "layers",
                "projects",
                "builder-slot"
            ])
        );
        let y = |key: &str| dock["section_title_bars_px"][key]["y"].as_f64().unwrap();
        assert!(y("info-settings") < y("layers") && y("layers") < y("projects"));
        assert_eq!(dock["layers_rect_px"]["column"], "left");
        assert_eq!(dock["left_icon_rects_in_sidebar"], true);
        assert_eq!(dock["sidebar_scroll_offset"].as_f64(), Some(0.));
        let outer = dock["left_rail_outer_px"].as_f64().unwrap();
        let inner = dock["left_rail_inner_px"].as_f64().unwrap();
        assert!((outer - 300.).abs() < 0.5, "{outer}");
        assert!(
            (outer - inner - f64::from(crate::dock_layout::LEFT_RAIL_CHROME_PX)).abs() < 0.5,
            "{outer} {inner}"
        );
        // Layers never appears in a right column of the receipt.
        assert!(!dock["right_sections"].to_string().contains("layers"));
        assert_eq!(SectionId::Layers.key(), "layers");
    }

    #[test]
    fn audit_min_window_follows_the_simulated_monitor_and_zoom() {
        let close = |actual: [f32; 2], expected: [f32; 2]| {
            (actual[0] - expected[0]).abs() < 0.1 && (actual[1] - expected[1]).abs() < 0.1
        };
        // No simulation: the supported minimum, scaled by the zoom.
        assert_eq!(audit_min_window(None, None), [1280., 720.]);
        assert!(close(audit_min_window(None, Some(1.5)), [1920., 1080.]));
        // A 1366x768 monitor at 100%, 125% and 150% (physical pixels).
        let monitor = Some([1366., 768.]);
        assert!(close(audit_min_window(monitor, None), [1280., 688.]));
        assert!(close(audit_min_window(monitor, Some(1.25)), [1346., 668.]));
        assert!(close(audit_min_window(monitor, Some(1.5)), [1342., 648.]));
    }

    #[test]
    fn audit_zoom_parses_only_finite_values_in_range() {
        assert_eq!(parse_zoom_factor("1.5"), Some(1.5));
        assert_eq!(parse_zoom_factor(" 2 "), Some(2.));
        assert_eq!(parse_zoom_factor("0.5"), Some(0.5));
        assert_eq!(parse_zoom_factor("4.0"), Some(4.));
        for bad in [
            "", "abc", "0", "-1", "0.49", "4.01", "NaN", "inf", "-inf", "1,5",
        ] {
            assert_eq!(parse_zoom_factor(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn audit_window_size_parses_only_positive_finite_pairs() {
        assert_eq!(parse_window_size("800x600"), Some([800., 600.]));
        assert_eq!(parse_window_size(" 1600 x 1000 "), Some([1600., 1000.]));
        for bad in [
            "", "800", "800x", "x600", "0x600", "-5x600", "NaNx600", "infx600", "axb",
        ] {
            assert_eq!(parse_window_size(bad), None, "{bad:?}");
        }
    }
}

#[cfg(test)]
mod gumball_colour_tests {
    use super::{axis_colour, gumball_colour, plane_colours};
    use om9_core::gumball::{Axis, Handle, Plane};

    #[test]
    fn the_three_axes_have_three_different_colours() {
        let (x, y, z) = (
            axis_colour(Axis::X),
            axis_colour(Axis::Y),
            axis_colour(Axis::Z),
        );
        assert_ne!(x, y);
        assert_ne!(y, z);
        assert_ne!(x, z);
        // Red, blue, yellow, as the manual names them.
        assert!(x.r() > 150 && x.g() < 100, "X is red: {x:?}");
        assert!(y.b() > 150 && y.r() < 120, "Y is blue: {y:?}");
        assert!(
            z.r() > 180 && z.g() > 120 && z.b() < 100,
            "Z is yellow: {z:?}"
        );
    }

    #[test]
    fn every_scale_box_carries_its_own_axis_colour() {
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            assert_eq!(gumball_colour(Handle::Scale(axis)), axis_colour(axis));
            assert_eq!(gumball_colour(Handle::MoveAxis(axis)), axis_colour(axis));
            assert_eq!(gumball_colour(Handle::Rotate(axis)), axis_colour(axis));
        }
        assert_ne!(
            gumball_colour(Handle::Scale(Axis::X)),
            gumball_colour(Handle::Scale(Axis::Z))
        );
    }

    #[test]
    fn a_plane_icon_shows_its_two_axes() {
        assert_eq!(
            plane_colours(Plane::XY),
            (axis_colour(Axis::X), axis_colour(Axis::Y))
        );
        assert_eq!(
            plane_colours(Plane::YZ),
            (axis_colour(Axis::Y), axis_colour(Axis::Z))
        );
        assert_eq!(
            plane_colours(Plane::ZX),
            (axis_colour(Axis::Z), axis_colour(Axis::X))
        );
    }
}
