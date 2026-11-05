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
    camera::{CPlane, Camera, Viewport},
    scene::PickIntent,
    workspace::ViewId,
};
mod channel_cutters;
mod chrome;
mod clipboard;
mod command;
mod gem;
mod gem_cluster;
mod gem_cutters;
mod gem_edits;
mod gem_pave;
mod icon_resize_audit;
#[cfg(test)]
mod icon_size_audit;
mod layer_audit;
mod layer_dialog_audit;
mod layers;
mod panel_audit;
mod sidebar_palettes;
#[cfg(test)]
mod sidebar_scroll_audit;
mod typography;
pub use layers::LayerControl;
mod file_audit;
mod file_success_audit;
mod files;
mod undo_settings;
mod layer_dialogs;
mod startup_audit;
pub use files::FileControl;
pub use layer_dialogs::LayerDialogControl;
mod occurrence_audit;
mod object_info;
mod os_audit;
mod rotation_feedback;
mod selection_audit;
mod top11;
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
pub const PANEL_AUDIT_SCENARIOS: [&str; 8] = [
    "reorder",
    "float",
    "dock-right",
    "escape",
    "roll-up",
    "float-drop",
    "readback-recover",
    "object-info",
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
struct HoverBasis {
    document: std::sync::Weak<om9_core::model::Document>,
    camera: om9_scene::camera::Camera,
    epoch: u64,
    preview: u64,
    pointer: egui::Pos2,
    viewport: Viewport,
}
// Drop actual snapshot allocations before their source/buffer owners and the
// reservation, on normal completion, cancellation-by-drop and worker unwind.
struct HoverSnapshot {
    scene: om9_scene::scene::Scene,
    _document: om9_core::transaction::CapturedDocument,
    _buffers: std::sync::Arc<crate::session::SceneBufferLease>,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}
fn hover_snapshot_overhead() -> u64 {
    (std::mem::size_of::<HoverSnapshot>() - std::mem::size_of::<om9_scene::scene::Scene>()) as u64
}
impl HoverBasis {
    fn matches(&self, document: &std::sync::Weak<om9_core::model::Document>, camera: &om9_scene::camera::Camera,
        epoch: u64, preview: u64, viewport: Viewport, pointer: egui::Pos2) -> bool {
        self.document.ptr_eq(document) && self.camera == *camera && self.epoch == epoch
            && self.preview == preview && self.viewport == viewport && self.pointer == pointer
    }
}
type HoverJob = (HoverBasis, std::sync::mpsc::Receiver<Option<om9_core::identity::ObjectId>>);

pub struct MatrixApp {
    pub controller: Controller,
    live_mcp: Option<crate::mcp::live::LiveHost>,
    #[cfg(windows)]
    clipboard_shortcut_hook: Option<om9_native_owner::clipboard::ShortcutHook>,
    #[cfg(windows)]
    clipboard_hook_attempted: bool,
    object_info_cache: std::cell::RefCell<object_info::Cache>,
    object_info_async: object_info::AsyncCache,
    /// Project Manager rows, Job Bag slots and Master snapshot (L2-03-15).
    pub project_manager: crate::project_manager::ProjectManager,
    pub display_preferences: crate::display::DisplayPreferences,
    pub pictureframes: Vec<crate::pictureframe::PictureFrame>,
    pictureframe_textures: BTreeMap<[u8; 32], egui::TextureHandle>,
    pub project_row_rects: BTreeMap<crate::project_manager::ProjectId, Rect>,
    pub jobbag_slot_rects: BTreeMap<usize, Rect>,
    pub project_action_rects: BTreeMap<&'static str, Rect>,
    pub transcript: Transcript,
    features: Vec<Feature>,
    focus_cmd: bool,
    cmd_rect: Option<Rect>,
    /// Command-name suggestions for the current draft, shown below the command row.
    cmd_suggestions: Vec<String>,
    cmd_suggestion_index: usize,
    cmd_suggestions_query: String,
    cmd_suggestions_rect: Option<Rect>,
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
    /// File ▸ Export Selected is waiting for the native `.stl` save dialog.
    stl_request: bool,
    stl_profile_draft: Option<f64>,
    stl_angle_draft: f64,
    stl_selected_profile: Option<crate::session::StlExportProfile>,
    stl_capture: Option<crate::session::StlExportCapture>,
    /// Gumball settings popup opened from Info & Settings.
    gumball_settings_open: bool,
    undo_settings_draft: Option<om9_core::transaction::UndoPolicy>,
    undo_settings_path: Option<PathBuf>,
    /// Application-scoped Gumball settings and the last value written to it.
    gumball_settings_path: Option<PathBuf>,
    gumball_settings_saved: crate::session::gumball::GumballSettings,
    gumball_settings_error_reported: bool,
    /// Opt-in OS-mouse/GPU gumball acceptance (`OM9_GUMBALL_OS_AUDIT`).
    os_audit: Option<os_audit::GumballOsAudit>,
    file_ime_composing: bool,
    file_f6_rects: BTreeMap<crate::file_lifecycle::FileKind, Rect>,
    /// Window title owned by this document's path/name and dirty marker.
    pub window_title: String,
    /// F4 jewelry datum overlay (world 0,0,0 origin and ring centre).
    world_datum_visible: bool,
    /// Responsive icon scale derived from the logical screen size.
    screen_icon_scale: f32,
    /// Toolbar icon edge after the responsive scale is applied.
    toolbar_icon_edge: f32,
    toolbar_drawing: bool,
    current_ui_context: Option<egui::Context>,
    hover: Option<Point3>,
    hover_view: Option<ViewId>,
    /// The object-snap candidate under the cursor, and its transient SmartTrack anchors.
    snap: Option<om9_scene::osnap::SnapCandidate>,
    surface_snap_cache: Vec<(std::sync::Weak<Document>, om9_scene::osnap::SnapCandidate, om9_scene::osnap::SnapCandidate)>,
    snap_state: om9_scene::osnap::SnapState,
    smarttrack: om9_scene::osnap::SmartTrack,
    smarttrack_enabled: bool,
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
    ambiguous_difference_context: Option<(u64, bool)>,
    ambiguous_pipe_context: Option<u64>,
    ambiguous_extract_context: Option<(u64, ViewId, bool, Vec<om9_scene::scene::PickHit>)>,
    ambiguous_sphere_context: Option<(u64, ViewId, Vec<om9_scene::scene::PickHit>)>,
    ambiguous_slab_context: Option<(u64, ViewId, Vec<om9_scene::scene::PickHit>)>,
    /// The subobject picked with Ctrl+Shift+click, and which kind that gesture looks for.
    sub_object: Option<crate::session::SelectedSubobject>,
    sub_object_kind: om9_scene::scene::PickKind,
    /// Object under the pointer, and where/when it was computed, so hovering does not pick every frame.
    hover_object: Option<om9_core::identity::ObjectId>,
    hover_job: Option<HoverJob>,
    hover_picked_at: Option<HoverBasis>,
    hover_retry_at: Option<std::time::Instant>,
    pub audit: Option<UiAudit>,
    passes: u64,
    /// Smoothed frames per second, shown in the status line.
    fps: f32,
    adapter: String,
    pub f6_open: bool,
    pub f6_pinned: bool,
    pub f6_custom: crate::f6::CustomMenu,
    pub f6_providers: crate::f6::Providers,
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
    /// Menu buttons and their items, cleared every frame (the Layers panel clears
    /// the action map, so menus keep their own).
    pub menu_rects: std::collections::BTreeMap<&'static str, Rect>,
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
    /// One document scene shared by the four viewports; geometry is uploaded once, not once per view.
    shared_scene: crate::gpu_viewport::SharedResourcePool,
    /// Opt-in JSONL log of camera and gumball gestures (`OM9_INTERACTION_LOG`).
    interaction_log: Option<crate::interaction_log::InteractionLog>,
    /// Opt-in scripted camera gesture (`OM9_CAMERA_AUDIT=<frames>`): orbit, zoom, then pan the
    /// Perspective view and log every frame, so interaction cost is measured without OS input.
    camera_audit: Option<CameraAudit>,
    /// Window-owned navigation; cameras remain in workspace.
    navigation: crate::navigation::NavigationController,
    scheduler: crate::frame_scheduler::WindowFrameScheduler,
    nav_frame_token: Option<u64>,
    nav_logged_token: Option<u64>,
    navigation_audit: Option<crate::navigation_audit::NavigationAudit>,
    nav_raw_events: Option<Vec<egui::Event>>,
    nav_time_origin: Option<(f64, std::time::Instant)>,
    nav_sample_now: std::time::Instant,
    nav_render_due: bool,
    nav_received: Option<std::time::Instant>,
    nav_frame_started: std::time::Instant,
    nav_document: Option<om9_core::identity::DocumentId>,
    nav_actions: std::collections::BTreeMap<ViewId, &'static str>,
    nav_cameras: std::collections::BTreeMap<ViewId, om9_scene::camera::Camera>,
    nav_camera_sources: BTreeMap<ViewId, u64>,
    /// Scene shown when the current primary-button gesture began.
    primary_press_scene: Option<(ViewId, crate::gpu_viewport::RefinementKey, Camera, [f64; 2])>,
    /// Which gumball handles are shown when no modifier is held.
    gumball_mode: om9_scene::gumball_view::Mode,
    /// Gumball gesture in flight: the drag session, its handle and the last pointer position.
    gumball_session: Option<om9_core::gumball::Session>,
    gumball_handle: Option<om9_core::gumball::Handle>,
    gumball_last: Option<[f64; 2]>,
    gumball_basis: Option<GumballBasis>,
    /// Most recent Top 11 grip drag: the handle, its drag plane and the live point.
    grip_drag: Option<GripDrag>,
    selected_grips: Vec<(om9_core::top11::GripHandle, u64)>,
    /// The builder field a viewport handle drag is moving (`OM9-L3-03-11-005`).
    builder_handle_drag: Option<&'static str>,
    /// The profile library window of the active builder (`OM9-L3-03-11-007`).
    builder_profile_window: bool,
    /// The Matrix History report window (`OM9-L3-03-13-003`).
    history_window: bool,
    /// The builder style browser window (`OM9-L3-03-12-002`).
    builder_style_window: bool,
    /// The name typed into the style browser's save field (`OM9-L3-03-12-004`).
    builder_style_name: String,
    grip_cache: std::cell::RefCell<Option<GripCache>>,
    gpu_drawn_views: std::collections::BTreeSet<ViewId>,
    pub status: StatusSurface,
    /// Object Health Report panel visibility (OM9-L3-03-19-007).
    pub health_panel_open: bool,
    /// Feature-edge angle used by the mesh reducer.
    pub feature_angle_deg: f32,
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
    /// Scroll viewport and separately allocated slim bar (logical points).
    pub sidebar_content_rect: Option<Rect>,
    pub sidebar_scrollbar_rect: Option<Rect>,
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
#[derive(Clone)]
struct GripDrag {
    handle: om9_core::top11::GripHandle,
    source: om9_core::identity::SourceStamp,
    epoch: u64,
    selection: u64,
    view: ViewId,
    camera: Camera,
    workspace_camera: Camera,
    maximized: Option<ViewId>,
    viewport: Viewport,
    view_plane: CPlane,
    plane: CPlane,
    press_world: Point3,
    current: Point3,
}
/// Grip handles only change with the document, selection or grip mode; the overlay is
/// painted every frame, so recomputing a large surface edit grid each frame is wasteful.
struct GripCache {
    revision: u64,
    selection: Vec<om9_core::identity::ObjectId>,
    mode: om9_core::top11::GripMode,
    handles: Vec<om9_core::top11::GripHandle>,
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

/// Scripted camera gesture for interaction measurements (`OM9_CAMERA_AUDIT=<frames>`).
struct CameraAudit {
    remaining: usize,
    total: usize,
    frames: usize,
}

/// One CMD history line: the prompt as shown, then `: ` and the submitted value
/// (`First corner of base (...): 0`). Prompts are stored with varied trailing
/// punctuation, so the colon and its single separating space are normalized here.
fn prompt_echo(prompt: &str, value: &str) -> String {
    let prompt = prompt.trim_end();
    if value.is_empty() {
        return prompt.to_owned();
    }
    if prompt.ends_with(':') {
        format!("{prompt} {value}")
    } else {
        format!("{prompt}: {value}")
    }
}

// Reorder indices, never candidate data: the resolver restores input order
// before final depth choice. Root + complete member path separates instances.
fn surface_group_order(candidates: &[om9_scene::osnap::SnapCandidate]) -> Result<Vec<usize>> {
    let mut order = Vec::new();
    order.try_reserve_exact(candidates.len()).map_err(|_| Error::Budget("surface snap grouping"))?;
    order.extend(0..candidates.len());
    order.sort_unstable_by(|&a, &b| {
        let left = &candidates[a];
        let right = &candidates[b];
        left.root.cmp(&right.root).then_with(|| {
            left.surface_source.as_ref().map(|s| (&s.path, s.leaf))
                .cmp(&right.surface_source.as_ref().map(|s| (&s.path, s.leaf)))
        }).then(a.cmp(&b))
    });
    Ok(order)
}

impl MatrixApp {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let (limits, measured) = crate::document_limits_with_memory_status();
        let mut app = Self::from_controller(Controller::from_file(path, limits)?)?;
        app.report_memory_fallback(measured);
        Ok(app)
    }
    pub fn new(document: Document) -> Result<Self> {
        let document = mesh_stress_document(document);
        let (limits, measured) = crate::document_limits_with_memory_status();
        let mut app = Self::from_controller(Controller::new(document, limits)?)?;
        app.report_memory_fallback(measured);
        Ok(app)
    }
    fn report_memory_fallback(&mut self, measured: bool) {
        if !measured {
            let message = "Physical RAM unavailable; managed memory is limited to at most 256 MiB";
            self.transcript.append_history(message);
            self.status.info(message);
        }
    }
    /// Typed startup: validate the configuration, open the empty document and
    /// load the owned layout. No window exists yet, so a failure here is a
    /// document-stage failure the caller reports.
    pub fn start(
        config: &crate::startup::AppConfig,
    ) -> std::result::Result<Self, crate::startup::StartupFailure> {
        use crate::startup::{StartupFailure, StartupStage};
        config.validate()?;
        let mut app = Self::new(Document::with_template_tolerance(&config.document_name))
            .map_err(|error| StartupFailure::from_error(StartupStage::Document, &error))?;
        if let Some(path) = &config.layout_path {
            app.set_dock_layout_path(Some(path.clone()));
            app.load_dock_layout();
        }
        if let Some(path) = &config.gumball_settings_path {
            // A missing preference file is defaults; a corrupt one is reported in the
            // transcript but never blocks startup.
            if let Err(error) = app.load_gumball_settings(path) {
                app.transcript
                    .append_history(&format!("Gumball settings: {error}"));
            }
        }
        if let Some(path) = &config.layout_path {
            if let Err(error) = app.load_undo_settings(&path.with_file_name("undo.json")) {
                app.transcript.append_history(&format!("Undo settings: {error}"));
            }
        }
        Ok(app)
    }
    fn from_controller(controller: Controller) -> Result<Self> {
        let features = serde_json::from_str(include_str!("../assets/features.json"))
            .map_err(|e| Error::Io(e.to_string()))?;
        let dock = DockLayout::matrix_default();
        let regions = dock.resolve([1600., 1000.]);
        let project_name = controller.document().name.clone();
        let mut app = Self {
            live_mcp: None,
            object_info_cache: Default::default(),
            object_info_async: Default::default(),
            gpu_viewports: std::collections::BTreeMap::new(),
            gpu_state: None,
            shared_scene: Default::default(),
            interaction_log: crate::interaction_log::InteractionLog::from_env(),
            navigation: Default::default(),
            scheduler: crate::frame_scheduler::WindowFrameScheduler::from_env(),
            nav_frame_token: None,
            nav_logged_token: None,
            navigation_audit: crate::navigation_audit::NavigationAudit::from_env(),
            nav_raw_events: None,
            nav_time_origin: None,
            nav_sample_now: std::time::Instant::now(),
            nav_render_due: true,
            nav_received: None,
            nav_frame_started: std::time::Instant::now(),
            nav_document: None,
            nav_actions: Default::default(),
            nav_cameras: Default::default(),
            nav_camera_sources: Default::default(),
            camera_audit: std::env::var("OM9_CAMERA_AUDIT")
                .ok()
                .and_then(|value| value.trim().parse::<usize>().ok())
                .filter(|frames| *frames > 0)
                .map(|frames| CameraAudit {
                    remaining: frames,
                    total: frames,
                    frames: 0,
                }),
            project_manager: crate::project_manager::ProjectManager::new(project_name),
            display_preferences: crate::display::DisplayPreferences::default(),
            pictureframes: Vec::new(),
            pictureframe_textures: BTreeMap::new(),
            project_row_rects: BTreeMap::new(),
            jobbag_slot_rects: BTreeMap::new(),
            project_action_rects: BTreeMap::new(),
            controller,
            #[cfg(windows)]
            clipboard_shortcut_hook: None,
            #[cfg(windows)]
            clipboard_hook_attempted: false,
            gumball_mode: om9_scene::gumball_view::Mode::Move,
            primary_press_scene: None,
            gumball_session: None,
            gumball_handle: None,
            gumball_last: None,
            gumball_basis: None,
            grip_drag: None,
            selected_grips: Vec::new(),
            builder_handle_drag: None,
            builder_profile_window: false,
            history_window: false,
            builder_style_window: false,
            builder_style_name: String::new(),
            grip_cache: std::cell::RefCell::new(None),
            gpu_drawn_views: std::collections::BTreeSet::new(),
            transcript: Transcript::new(),
            features,
            focus_cmd: true,
            cmd_rect: None,
            cmd_suggestions_rect: None,
            cmd_suggestions: Vec::new(),
            cmd_suggestion_index: 0,
            cmd_suggestions_query: String::new(),
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
            stl_request: false,
            stl_profile_draft: None,
            stl_angle_draft: 0.3,
            stl_selected_profile: None,
            stl_capture: None,
            gumball_settings_open: false,
            undo_settings_draft: None,
            undo_settings_path: None,
            gumball_settings_path: None,
            gumball_settings_saved: crate::session::gumball::GumballSettings::default(),
            gumball_settings_error_reported: false,
            os_audit: None,
            file_ime_composing: false,
            file_f6_rects: BTreeMap::new(),
            window_title: String::new(),
            world_datum_visible: true,
            screen_icon_scale: 1.,
            toolbar_icon_edge: 22.,
            toolbar_drawing: false,
            current_ui_context: None,
            hover: None,
            hover_view: None,
            snap: None,
            snap_state: om9_scene::osnap::SnapState::default(),
            surface_snap_cache: Vec::new(),
            smarttrack: om9_scene::osnap::SmartTrack::new(),
            smarttrack_enabled: true,
            band_start: None,
            band_view: None,
            selection_frames: std::collections::BTreeMap::new(),
            ambiguous: Vec::new(),
            ambiguous_pos: None,
            ambiguous_epoch: 0,
            ambiguous_difference_context: None,
            ambiguous_pipe_context: None,
            ambiguous_extract_context: None,
            ambiguous_sphere_context: None,
            ambiguous_slab_context: None,
            sub_object: None,
            sub_object_kind: om9_scene::scene::PickKind::Face,
            hover_object: None,
            hover_job: None,
            hover_picked_at: None,
            hover_retry_at: None,
            audit: None,
            passes: 0,
            fps: 0.0,
            adapter: "headless-egui".into(),
            f6_open: false,
            f6_pinned: false,
            f6_custom: crate::f6::CustomMenu::default(),
            f6_providers: crate::f6::Providers {
                // M04 owns the ring builder backend (OM9-L2-03-11 / OM9-L2-04-02).
                m04_builder: true,
                // M05 gem assets, loader, builders, positioner and Gem Properties.
                m05_gem: true,
                ..Default::default()
            },
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
            menu_rects: std::collections::BTreeMap::new(),
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
            health_panel_open: false,
            feature_angle_deg: 30.0,
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
            sidebar_content_rect: None,
            sidebar_scrollbar_rect: None,
            audit_layout_mode: AuditLayoutMode::Default,
            audit_rail_width: None,
            builder_slot_rect: None,
            status_rect: None,
            status_message_rect: None,
            status_summary_rect: None,
            status_dismiss_rect: None,
            dock_panel_rects: BTreeMap::new(),
        };
        match app.controller.embedded_project_state() {
            Ok(Some(bytes)) => {
                let name = app.controller.document().name.clone();
                match crate::project_manager::ProjectManager::from_json(&bytes, &name) {
                    Ok(manager) => app.project_manager = manager,
                    Err(error) => app
                        .transcript
                        .append_history(&format!("project manager state: {error}")),
                }
            }
            Ok(None) => {}
            Err(error) => app
                .transcript
                .append_history(&format!("project manager state: {error}")),
        }
        for diagnostic in app.controller.file_diagnostics() {
            app.transcript.append_history(diagnostic);
        }
        if let Some(log) = app.interaction_log.as_mut() {
            let limits = app.controller.coordinator().limits();
            log.record(serde_json::json!({
                "event": "session_start",
                "document": app.controller.document().name,
                "objects": app.controller.document().entities.len(),
                "max_objects": limits.max_objects,
                "max_points": limits.max_points,
                "max_bytes": limits.max_bytes,
            }));
        }
        if let Some(audit) = os_audit::GumballOsAudit::from_env() {
            app.os_audit = Some(audit);
            app.os_audit_setup()?;
        }
        Ok(app)
    }
    /// Scripted orbit, zoom then pan of the Perspective view, one camera step per frame, with a JSONL
    /// record of the frame time and scene counters. It closes the window after the requested frames.
    fn camera_audit_frame(&mut self, ui: &mut egui::Ui) {
        let Some(audit) = self.camera_audit.as_mut() else {
            return;
        };
        if audit.remaining == 0 {
            return;
        }
        audit.remaining -= 1;
        audit.frames += 1;
        let frame = audit.frames;
        let total = audit.total.max(1);
        let done = audit.remaining == 0;
        let action = if frame * 3 <= total {
            "orbit"
        } else if frame * 3 <= total * 2 {
            "zoom"
        } else {
            "pan"
        };
        let id = ViewId::Perspective;
        let camera = self
            .controller
            .workspace
            .views
            .get_mut(&id)
            .and_then(|view| {
                let result = match action {
                    "orbit" => view.camera.orbit(0.01, 0.004),
                    "zoom" => view.camera.zoom(
                        std::env::var("OM9_AUDIT_ZOOM_FACTOR")
                            .ok()
                            .and_then(|value| value.trim().parse::<f64>().ok())
                            .unwrap_or(0.996),
                    ),
                    _ => view.camera.pan([0.01, 0.0]),
                };
                result.ok().map(|()| view.camera.clone())
            });
        let frame_ms = crate::interaction_log::frame_ms(ui.input(|input| input.stable_dt));
        let port = self.gpu_viewports.get(&id);
        let stats = port.and_then(|port| port.frame_stats());
        if let (Some(log), Some(camera)) = (self.interaction_log.as_mut(), camera.as_ref()) {
            log.record(serde_json::json!({
                "event": "camera_audit",
                "frame": frame,
                "action": action,
                "frame_ms": frame_ms,
                "revision": camera.revision,
                "eye": camera.eye,
                "target": camera.target,
                "scene_objects": stats.map(|stats| stats.objects),
                "visible": stats.map(|stats| stats.visible),
                "culled": stats.map(|stats| stats.culled),
                "draw_calls": stats.map(|stats| stats.draw_calls),
                "prepare_ms": port.map(|port| port.last_prepare_ms()),
                "draw_ms": port.map(|port| port.last_draw_ms()),
            }));
        }
        ui.ctx().request_repaint();
        if done {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    /// Serialize the Project Manager and display preferences into the open document before a save.
    pub fn stage_project_state(&mut self) -> Result<()> {
        if self.project_manager.has_persistent_state() {
            let bytes = self.project_manager.to_json()?;
            self.controller.store_project_state(&bytes)?;
        }
        let display = serde_json::to_vec(&self.display_preferences)
            .map_err(|_| Error::Invalid("display state"))?;
        self.controller.store_display_state(&display)?;
        let store = crate::pictureframe::PictureFrameStore {
            version: 1,
            frames: self
                .pictureframes
                .iter()
                .map(|frame| frame.to_record(true))
                .collect(),
        };
        self.controller
            .store_pictureframes(store.to_json()?.as_bytes())?;
        self.controller
            .store_f6_state(self.f6_custom.to_json()?.as_bytes())
    }
    /// Reload Project Manager and display state after New/Open/Master replacement.
    pub fn reload_project_state(&mut self) -> Result<()> {
        let name = self.controller.document().name.clone();
        self.project_manager = match self.controller.embedded_project_state()? {
            Some(bytes) => crate::project_manager::ProjectManager::from_json(&bytes, &name)?,
            None => crate::project_manager::ProjectManager::new(name),
        };
        self.display_preferences = match self.controller.embedded_display_state()? {
            Some(bytes) => {
                let text =
                    String::from_utf8(bytes).map_err(|_| Error::Invalid("display state text"))?;
                let (preferences, warning) = crate::display::DisplayPreferences::from_json(&text);
                if let Some(warning) = warning {
                    self.transcript.append_history(&warning);
                    self.status.info(&warning);
                }
                preferences
            }
            None => crate::display::DisplayPreferences::default(),
        };
        self.pictureframes = match self.controller.embedded_pictureframes()? {
            Some(bytes) => {
                let text = String::from_utf8(bytes)
                    .map_err(|_| Error::Invalid("picture frame state text"))?;
                let (frames, warnings) = crate::pictureframe::PictureFrameStore::from_json(&text)?;
                for warning in warnings {
                    self.transcript.append_history(&warning);
                    self.status.info(&warning);
                }
                frames
            }
            None => Vec::new(),
        };
        self.f6_custom = match self.controller.embedded_f6_state()? {
            Some(bytes) => {
                let text = String::from_utf8(bytes).map_err(|_| Error::Invalid("F6 state text"))?;
                match crate::f6::CustomMenu::from_json(&text) {
                    Ok(menu) => menu,
                    Err(error) => {
                        self.transcript
                            .append_history(&format!("F6 state: {error}"));
                        crate::f6::CustomMenu::default()
                    }
                }
            }
            None => crate::f6::CustomMenu::default(),
        };
        Ok(())
    }

    /// Import one PNG reference image into the active CPlane as an aspect-locked quad.
    /// Decoding runs before the frame is pushed, so a bad image never leaves an empty quad.
    pub fn import_picture_frame(
        &mut self,
        name: &str,
        bytes: Vec<u8>,
        origin: om9_core::geometry::Point3,
        width_mm: f64,
    ) -> Result<()> {
        let asset = crate::pictureframe::RasterAsset::from_png(name, bytes)?;
        let frame = crate::pictureframe::PictureFrame::import_png(
            asset,
            &self.controller.input_plane(),
            origin,
            width_mm,
        )?;
        self.pictureframes.push(frame);
        Ok(())
    }

    pub fn picture_frames(&self) -> &[crate::pictureframe::PictureFrame] {
        &self.pictureframes
    }
    /// Restore the captured Master document atomically and reload its project state.
    pub fn restore_master_live(&mut self) -> Result<()> {
        let document = self
            .project_manager
            .restore_master_document()
            .ok_or(Error::Missing("Master jobbag"))?;
        self.controller.restore_master_document(document)?;
        // Keep the live Project Manager rows/slots; the Master snapshot owns
        // geometry/layers, not the user's current project workspace.
        self.stage_project_state()
    }
    fn feedback(&mut self, result: Result<()>) {
        // A Measure read-out is the command's answer, so it is shown in the
        // Command Line instead of the generic "Ready" (OM9-L3-03-20-005).
        let measurement = self.controller.take_measurement_label();
        match result {
            Ok(()) => {
                if let Some(label) = measurement {
                    self.transcript.append_history(&label);
                    self.status.info(&label);
                } else if self.controller.is_running() {
                    let prompt = self.controller.prompt();
                    self.status
                        .info(&format!("{prompt} — type a value and press Enter"));
                } else {
                    self.status.complete_success();
                }
            }
            Err(error) => {
                let message = format!("{error}");
                self.transcript.append_history(&message);
                self.status.error(&message);
            }
        }
        // Matrix History switches and the report speak through the same status line.
        if let Some(notice) = self.controller.take_history_notice() {
            self.transcript.append_history(&notice);
            self.status.info(&notice);
        }
        if self.controller.take_history_window_request() {
            self.history_window = true;
        }
        self.transcript
            .set_prompt(&self.controller.prompt_with_options());
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
    fn apply_view_command(&mut self, name: &str) -> Option<Result<()>> {
        let command = Controller::view_command(name)?;
        let active = self.controller.workspace.active;
        let aspect = self
            .view_rects
            .get(&active)
            .map(|rect| (rect.width() / rect.height().max(1.0)).clamp(0.05, 20.0) as f64)
            .unwrap_or(1.6);
        Some(match command {
            crate::session::ZOOM_EXTENTS => self.controller.zoom_extents(aspect),
            crate::session::ZOOM_SELECTED => self.controller.zoom_selected(aspect),
            crate::session::VIEW_HOME => self.controller.view_home(),
            crate::session::VIEW_CENTER => self.controller.view_center(),
            _ => return None,
        })
    }

    /// The current F6 context resolved from the document selection and registry.
    pub fn f6_context(&self) -> crate::f6::Context {
        crate::f6::context(
            self.controller.document(),
            self.controller.selected(),
            self.f6_providers,
        )
    }

    fn invoke(&mut self, id: &str, origin: InvocationOrigin) {
        if crate::file_lifecycle::FileKind::from_feature(id).is_some_and(|kind| matches!(kind, crate::file_lifecycle::FileKind::New | crate::file_lifecycle::FileKind::Open)) {
            self.cancel_stl_export_options();
        }
        if self.open_undo_settings_command(id) { return; }
        if crate::file_lifecycle::FileKind::from_feature(id).is_some_and(|kind| {
            matches!(
                kind,
                crate::file_lifecycle::FileKind::Save | crate::file_lifecycle::FileKind::SaveAs
            )
        }) && let Err(error) = self.stage_project_state()
        {
            self.feedback(Err(error));
            return;
        }
        if let Some(result) = self.apply_view_command(id) {
            self.feedback(result);
            return;
        }
        // F6 "Builder": hand the selection to the M04 builder through the typed prefill.
        if id == "BuilderPrefill" {
            let result = self.builder_from_prefill();
            self.feedback(result);
            return;
        }

        let result = self.controller.dispatch_effect(id, origin);
        if result.is_ok() && self.controller.is_running() {
            // The first prompt belongs to the live row; history records the command that
            // started, exactly like the Matrix command line (`Command: Box`).
            let label = self.command_label(id);
            self.transcript.append_history(&format!("Command: {label}"));
        }
        if result.is_ok() && !self.f6_pinned {
            self.f6_open = false;
        }
        match result {
            Ok(crate::session::DispatchOutcome::File(outcome)) => {
                let ctx = self.current_ui_context.clone().expect("UI context");
                let _ = self.file_feedback(&ctx, Ok(outcome));
            }
            Ok(crate::session::DispatchOutcome::StlExport) => {
                self.begin_stl_export_options();
            }
            other => self.feedback(other.map(|_| ())),
        }
    }
    /// F6 "Builder" action: emit the typed prefill and open the mode the selection fits.
    fn builder_from_prefill(&mut self) -> Result<()> {
        use std::collections::BTreeSet;
        let selection: BTreeSet<_> = self.controller.selected().iter().copied().collect();
        let prefill = crate::f6::builder_prefill(
            &selection,
            [0.0; 3],
            self.controller.document().context_revision,
            self.f6_providers,
        )?;
        let kind = match prefill.selection.len() {
            0 => om9_core::builder::BuilderKind::RingRail,
            1 => om9_core::builder::BuilderKind::RailProfile,
            _ => om9_core::builder::BuilderKind::RailSweep,
        };
        self.controller.begin_builder(kind)
    }

    fn append_running_prompt(&mut self) {
        if self.controller.is_running() {
            let prompt = self.controller.prompt();
            self.transcript.append_history(&prompt);
        }
    }
    /// The command name recorded in the CMD history when a menu icon starts a command.
    fn command_label(&self, id: &str) -> String {
        self.features
            .iter()
            .find(|feature| feature.id == id)
            .or_else(|| {
                crate::icons::feature_for(id)
                    .and_then(|feature| self.features.iter().find(|row| row.id == feature))
            })
            .map_or_else(|| id.to_owned(), |feature| feature.name.clone())
    }
    fn submit_command_option(&mut self, option: crate::command_options::CommandOptionId) -> bool {
        use crate::command_options::{CommandOptionEffect, CommandOptionInterface};
        let prompt = self.controller.prompt().to_owned();
        let result = self.controller.activate_command_option(option);
        match result {
            Ok(CommandOptionEffect::Draft(prefix)) => {
                self.transcript.set_draft(&prefix);
                self.focus_cmd = true;
                false // Keep the newly staged draft after typed-name activation.
            }
            result => {
                self.transcript
                    .append_history(&prompt_echo(&prompt, option.name()));
                let result = result.map(|_| ());
                let success = result.is_ok();
                self.feedback(result);
                self.focus_cmd = true;
                success
            }
        }
    }
    fn submit_draft(&mut self) {
        use crate::command_options::CommandOptionInterface;
        let draft = self.transcript.draft().to_owned();
        if !self.controller.is_running() && crate::file_lifecycle::FileKind::from_command(&draft).is_some_and(|kind| matches!(kind, crate::file_lifecycle::FileKind::New | crate::file_lifecycle::FileKind::Open)) {
            self.cancel_stl_export_options();
        }
        if !self.controller.is_running() && self.open_undo_settings_command(&draft) {
            self.transcript.set_draft("");
            return;
        }
        if let Some(option) = self.controller.resolve_command_option(&draft) {
            if self.submit_command_option(option) {
                self.transcript.set_draft("");
            }
            return;
        }
        let prompt = self.controller.prompt();
        self.transcript
            .append_history(&prompt_echo(&prompt, &draft));

        if crate::file_lifecycle::FileKind::from_command(&draft).is_some_and(|kind| {
            matches!(
                kind,
                crate::file_lifecycle::FileKind::Save | crate::file_lifecycle::FileKind::SaveAs
            )
        }) && let Err(error) = self.stage_project_state()
        {
            self.feedback(Err(error));
            return;
        }
        if let Some(result) = self.apply_view_command(&draft) {
            if result.is_ok() {
                self.transcript.record_command(&draft);
                if let Some(command) = Controller::view_command(&draft) {
                    self.transcript.append_history(&format!("[{command}]"));
                }
                self.transcript.set_draft("");
            }
            self.feedback(result);
            return;
        }
        let result = self.controller.submit_effect(&draft);
        if result.is_ok() {
            self.transcript.record_command(&draft);
            // The transcript mirrors the Matrix command log: the typed command and its
            // prompt+value lines only. The canonical feature id stays in the document
            // history/receipts instead of adding a `[OM9-…]` line to every command.
            self.transcript.set_draft("");
        }
        match result {
            Ok(crate::session::DispatchOutcome::File(outcome)) => {
                let ctx = self.current_ui_context.clone().expect("UI context");
                let _ = self.file_feedback(&ctx, Ok(outcome));
            }
            Ok(crate::session::DispatchOutcome::StlExport) => {
                self.begin_stl_export_options();
            }
            Ok(crate::session::DispatchOutcome::Top11(outcome)) => {
                let message = outcome.message();
                self.status.info(&message);
                self.transcript.append_history(&message);
            }
            // A command the program does not know is reported with the words the
            // user typed: "unsupported unknown/unavailable command" alone leaves
            // the reader guessing which command was refused (OM9-L3-01-01-007).
            Err(Error::Unsupported(_)) if !draft.trim().is_empty() => {
                let name: String = draft.trim().chars().take(120).collect();
                let message = format!("unknown command: {name}");
                self.transcript.append_history(&message);
                self.status.error(&message);
                self.transcript
                    .set_prompt(&self.controller.prompt_with_options());
                self.focus_cmd = true;
            }
            other => self.feedback(other.map(|_| ())),
        }
    }
    /// Whether the command editor asked for keyboard focus on this frame (tests and audit).
    pub fn command_focus_requested(&self) -> bool {
        self.focus_cmd
    }

    /// Put keyboard focus back on the command editor (the normal state after a typed command).
    pub fn focus_command_editor(&mut self) {
        self.focus_cmd = true;
    }

    pub fn command_input_rect(&self) -> Option<Rect> {
        self.cmd_rect
    }
    /// The command-name suggestion list rectangle, for tests and hit reports.
    pub fn command_suggestions_rect(&self) -> Option<Rect> {
        self.cmd_suggestions_rect
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

    /// The active snap candidate, for the marker/tooltip tests and the audit receipt.
    pub fn snap_candidate(&self) -> Option<&om9_scene::osnap::SnapCandidate> {
        self.snap.as_ref()
    }

    /// Transient SmartTrack alignment lines, for tests and the audit receipt.
    pub fn smarttrack_guides(&self) -> &[om9_scene::osnap::SnapGuide] {
        self.smarttrack.guides()
    }

    /// Number of decoded picture-frame textures currently bound for painting.
    pub fn pictureframe_texture_count(&self) -> usize {
        self.pictureframe_textures.len()
    }

    pub fn snap_enabled(&self, kind: om9_scene::osnap::SnapKind) -> bool {
        self.snap_state.settings.is_enabled(kind)
    }

    /// Enable or disable one persistent snap kind.
    pub fn set_snap_enabled(&mut self, kind: om9_scene::osnap::SnapKind, enabled: bool) {
        self.snap_state.settings.set(kind, enabled);
        if !enabled && self.snap.as_ref().is_some_and(|snap| snap.kind == kind) {
            self.snap = None;
        }
        if !enabled && self.snap_state.settings.one_shot == Some(kind) {
            self.snap_state.settings.one_shot = None;
        }
    }

    /// Set a one-shot snap override for exactly the next accepted pick.
    pub fn set_snap_override(&mut self, kind: Option<om9_scene::osnap::SnapKind>) {
        self.snap_state.settings.one_shot = kind;
    }

    /// The transient one-shot snap kind, if any.
    pub fn snap_override(&self) -> Option<om9_scene::osnap::SnapKind> {
        self.snap_state.settings.one_shot
    }

    pub fn smarttrack_enabled(&self) -> bool {
        self.smarttrack_enabled
    }

    /// Enable or disable SmartTrack. Disabling also drops transient anchors so no ghost guide survives.
    pub fn set_smarttrack_enabled(&mut self, enabled: bool) {
        self.smarttrack_enabled = enabled;
        if !enabled {
            self.smarttrack.clear();
        }
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
        } else if let Some((run, view_id, hits)) = &self.ambiguous_slab_context {
            if self.controller.solid_slab_pick_context() == Some(*run) {
                hits.get(index)
                    .cloned()
                    .ok_or(Error::Invalid("Slab selection candidate"))
                    .and_then(|hit| {
                        let rect = self
                            .view_rects
                            .get(view_id)
                            .ok_or(Error::Stale("pick viewport"))?;
                        let viewport = om9_scene::camera::Viewport {
                            origin: [rect.min.x as f64, rect.min.y as f64],
                            size: [rect.width() as f64, rect.height() as f64],
                        };
                        let view = self
                            .controller
                            .workspace
                            .views
                            .get(view_id)
                            .ok_or(Error::Stale("pick view"))?;
                        hit.revalidate_in_view(
                            self.controller.coordinator(),
                            &view.camera,
                            viewport,
                        )?;
                        self.controller.solid_slab_pick_curve(hit.root)
                    })
            } else {
                Err(Error::Stale("Slab selection candidates"))
            }
        } else if let Some((run, view_id, hits)) = &self.ambiguous_sphere_context {
            if self.controller.solid_sphere_edge_pick_context() == Some(*run) {
                hits.get(index)
                    .cloned()
                    .ok_or(Error::Invalid("edge selection candidate"))
                    .and_then(|hit| {
                        let rect = self
                            .view_rects
                            .get(view_id)
                            .ok_or(Error::Stale("pick viewport"))?;
                        let viewport = om9_scene::camera::Viewport {
                            origin: [rect.min.x as f64, rect.min.y as f64],
                            size: [rect.width() as f64, rect.height() as f64],
                        };
                        let view = self
                            .controller
                            .workspace
                            .views
                            .get(view_id)
                            .ok_or(Error::Stale("pick view"))?;
                        hit.revalidate_in_view(
                            self.controller.coordinator(),
                            &view.camera,
                            viewport,
                        )?;
                        self.controller.solid_sphere_pick_brep_edge(
                            crate::session::SelectedSubobject::from_hit(&hit),
                            hit.world_point,
                        )
                    })
            } else {
                Err(Error::Stale("Sphere selection candidates"))
            }
        } else if let Some((run, view_id, remove, hits)) = &self.ambiguous_extract_context {
            if self.controller.solid_extract_pick_context() == Some(*run) {
                hits.get(index)
                    .cloned()
                    .ok_or(Error::Invalid("face selection candidate"))
                    .and_then(|hit| {
                        let rect = self
                            .view_rects
                            .get(view_id)
                            .ok_or(Error::Stale("pick viewport"))?;
                        let viewport = om9_scene::camera::Viewport {
                            origin: [rect.min.x as f64, rect.min.y as f64],
                            size: [rect.width() as f64, rect.height() as f64],
                        };
                        let view = self
                            .controller
                            .workspace
                            .views
                            .get(view_id)
                            .ok_or(Error::Stale("pick view"))?;
                        hit.revalidate_in_view(
                            self.controller.coordinator(),
                            &view.camera,
                            viewport,
                        )?;
                        let reference = crate::session::SelectedSubobject::from_hit(&hit);
                        if *remove {
                            self.controller.solid_extract_remove_face(reference)
                        } else {
                            self.controller.solid_extract_pick_face(reference)
                        }
                    })
            } else {
                Err(Error::Stale("Extract Surface selection candidates"))
            }
        } else if let Some(run) = self.ambiguous_pipe_context {
            if self.controller.solid_pipe_pick_context() == Some(run) {
                self.controller.solid_pipe_pick_curve(id)
            } else {
                Err(Error::Stale("Pipe selection candidates"))
            }
        } else if let Some(context) = self.ambiguous_difference_context {
            if self.controller.solid_boolean_pick_context() == Some(context) {
                self.controller.solid_boolean_pick(id, false)
            } else {
                Err(Error::Stale("Boolean selection candidates"))
            }
        } else {
            self.controller.select_objects(&[id])
        };
        self.ambiguous.clear();
        self.ambiguous_pos = None;
        self.ambiguous_difference_context = None;
        self.ambiguous_pipe_context = None;
        self.ambiguous_extract_context = None;
        self.ambiguous_sphere_context = None;
        self.ambiguous_slab_context = None;
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

    fn ambiguous_face_row(
        &self,
        index: usize,
        id: &om9_core::identity::ObjectId,
    ) -> (String, Option<[u8; 4]>) {
        let (label, colour) = self.ambiguous_row(id);
        if let Some((_, _, hits)) = &self.ambiguous_sphere_context {
            let edge = hits.get(index).and_then(|hit| {
                let om9_scene::scene::Subelement::BrepEdge(key) = hit.subset else {
                    return None;
                };
                self.controller
                    .document()
                    .entities
                    .get(id)
                    .and_then(|entity| match &entity.geometry {
                        om9_core::geometry::Geometry::Brep(brep) => {
                            brep.display.edges.iter().position(|item| *item == key)
                        }
                        _ => None,
                    })
            });
            return (
                format!(
                    "{label} · Edge{} · {}",
                    edge.map_or("?".to_owned(), |i| (i + 1).to_string()),
                    &id.0.to_string()[..8]
                ),
                colour,
            );
        }
        let Some((_, _, _, hits)) = &self.ambiguous_extract_context else {
            return (label, colour);
        };
        let face = hits.get(index).and_then(|hit| {
            let om9_scene::scene::Subelement::BrepFace(face) = hit.subset else {
                return None;
            };
            self.controller
                .document()
                .entities
                .get(id)
                .and_then(|entity| match &entity.geometry {
                    om9_core::geometry::Geometry::Brep(brep) => {
                        brep.display.faces.iter().position(|item| *item == face)
                    }
                    _ => None,
                })
        });
        (
            format!(
                "{label} · face {} · {}",
                face.map_or("?".to_owned(), |i| i.to_string()),
                &id.0.to_string()[..8]
            ),
            colour,
        )
    }

    /// The candidate labels, for tests and status reports.
    pub fn ambiguous_swatches(&self) -> Vec<Option<[u8; 4]>> {
        self.ambiguous
            .iter()
            .map(|id| self.ambiguous_row(id).1)
            .collect()
    }

    /// The candidate labels, for tests and status reports.
    pub fn ambiguous_labels(&self) -> Vec<String> {
        self.ambiguous
            .iter()
            .enumerate()
            .map(|(index, id)| self.ambiguous_face_row(index, id).0)
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
                let (label, colour) = self.ambiguous_face_row(index, id);
                (index, label, colour)
            })
            .collect();
        let mut chosen: Option<usize> = None;
        let area = egui::Area::new(egui::Id::new("om9-pick-popup"))
            .fixed_pos(position)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(if self.ambiguous_sphere_context.is_some() {
                        "Select edge"
                    } else if self.ambiguous_extract_context.is_some() {
                        "Select face"
                    } else {
                        "Select object"
                    });
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
            let is_pipe = self.ambiguous_pipe_context.is_some()
                || self.ambiguous_sphere_context.is_some()
                || self.ambiguous_slab_context.is_some();
            let result = self.choose_ambiguous(index);
            if is_pipe && result.is_ok() {
                self.append_running_prompt();
            }
            self.feedback(result);
        } else if ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !area.response.rect.contains(pos))
        }) {
            self.ambiguous.clear();
            self.ambiguous_pos = None;
            self.ambiguous_extract_context = None;
            self.ambiguous_sphere_context = None;
            self.ambiguous_slab_context = None;
            self.ambiguous_pipe_context = None;
        }
    }

    /// Decode each embedded reference image once into an egui/GPU texture. The key is the
    /// immutable SHA-256, so re-importing identical bytes reuses the same texture.
    fn sync_pictureframe_textures(&mut self, context: &egui::Context) {
        for frame in &self.pictureframes {
            if frame.asset.bytes.is_empty()
                || self.pictureframe_textures.contains_key(&frame.asset.sha256)
            {
                continue;
            }
            let Ok(decoded) = image::load_from_memory(&frame.asset.bytes) else {
                continue;
            };
            let rgba = decoded.to_rgba8();
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [rgba.width() as usize, rgba.height() as usize],
                rgba.as_raw(),
            );
            let texture = context.load_texture(
                format!("pictureframe-{}", frame.asset.sha_hex()),
                image,
                egui::TextureOptions::LINEAR,
            );
            self.pictureframe_textures
                .insert(frame.asset.sha256, texture);
        }
    }

    /// Paint visible reference quads with the imported image and its opacity. This is the
    /// viewport-side texture binding for Picture Frame; hidden frames are skipped, locked
    /// frames stay visible because lock is an edit/selection policy.
    fn paint_picture_frames(
        &self,
        painter: &egui::Painter,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        if self.pictureframes.is_empty() {
            return;
        }
        let painter = painter.with_clip_rect(rect);
        for frame in &self.pictureframes {
            if !frame.visible {
                continue;
            }
            let Some(texture) = self.pictureframe_textures.get(&frame.asset.sha256) else {
                continue;
            };
            let corners = frame.corners();
            let mut points = Vec::with_capacity(4);
            for corner in corners {
                let Ok(Some(projected)) = view.camera.project(corner, viewport) else {
                    points.clear();
                    break;
                };
                points.push(egui::pos2(
                    projected.screen[0] as f32,
                    projected.screen[1] as f32,
                ));
            }
            if points.len() != 4 {
                continue;
            }
            let alpha = (frame.opacity.clamp(0., 1.) * 255.).round() as u8;
            let color = egui::Color32::from_white_alpha(alpha);
            let mut mesh = egui::Mesh::with_texture(texture.id());
            for (position, uv) in points
                .into_iter()
                .zip([(0., 0.), (1., 0.), (1., 1.), (0., 1.)])
            {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: position,
                    uv: egui::pos2(uv.0, uv.1),
                    color,
                });
            }
            mesh.indices = vec![0, 1, 2, 0, 2, 3];
            painter.add(egui::Shape::mesh(mesh));
        }
    }

    /// Draw the transient snap marker and SmartTrack guides. The candidate comes from the
    /// core query; this paints it, it never runs a second snap solver.
    fn paint_osnap(
        &self,
        painter: &egui::Painter,
        id: ViewId,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
    ) {
        if self.smarttrack_enabled {
            for guide in self.smarttrack.guides() {
                let (Ok(Some(start)), Ok(Some(end))) = (
                    view.camera.project(guide.start, viewport),
                    view.camera.project(guide.end, viewport),
                ) else {
                    continue;
                };
                painter.line_segment(
                    [
                        egui::pos2(start.screen[0] as f32, start.screen[1] as f32),
                        egui::pos2(end.screen[0] as f32, end.screen[1] as f32),
                    ],
                    egui::Stroke::new(1.0, Color32::from_rgb(220, 190, 80)),
                );
            }
        }
        if id != self.controller.workspace.active {
            return;
        }
        if let Some(kind) = self.snap_state.settings.one_shot {
            painter.text(
                egui::pos2(
                    viewport.origin[0] as f32 + 12.0,
                    viewport.origin[1] as f32 + 30.0,
                ),
                egui::Align2::LEFT_TOP,
                format!("{} ONE-SHOT", kind.label()),
                egui::FontId::proportional(11.),
                Color32::from_rgb(255, 220, 90),
            );
        }
        let Some(snap) = &self.snap else {
            return;
        };
        let at = egui::pos2(snap.screen[0] as f32, snap.screen[1] as f32);
        let size = 10.0;
        painter.rect_stroke(
            egui::Rect::from_center_size(at, egui::vec2(size, size)),
            0.0,
            egui::Stroke::new(1.5, Color32::from_rgb(255, 220, 90)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            at + egui::vec2(size * 0.7, -size * 0.7),
            egui::Align2::LEFT_BOTTOM,
            snap.marker_label(),
            egui::FontId::proportional(11.),
            Color32::from_rgb(255, 220, 90),
        );
    }

    /// Admit the optional background hover only with a current physical-RAM sample.
    fn reserve_hover_snapshot(
        &self, physical: Option<(u64, u64)>,
    ) -> om9_core::Result<(om9_core::transaction::CapturedDocument, std::sync::Arc<crate::session::SceneBufferLease>, om9_core::transaction::ExternalMemoryReservation)> {
        let physical = physical.ok_or(om9_core::Error::Unsupported("physical RAM availability is unavailable"))?;
        let scene = self.controller.scene().ok_or(om9_core::Error::Missing("hover scene"))?;
        let coordinator = self.controller.coordinator();
        let capture = coordinator.capture_tracked_document(scene.document.id)?;
        if !capture.matches_weak(&std::sync::Arc::downgrade(&scene.document)) {
            return Err(om9_core::Error::Stale("hover scene document"));
        }
        // Render buffers are already shared with the viewport/current scene;
        // reserve the independent snapshot metadata and picking workspace.
        let buffers = self.controller.share_scene_buffer_owner_with_sample(scene, Some(physical))?;
        let bytes = scene.render_snapshot_metadata_estimate()?.bytes
            .checked_add(hover_snapshot_overhead())
            .ok_or(om9_core::Error::Overflow("hover snapshot accounting"))?
            .checked_add(scene.render_snapshot_pick_scratch_estimate()?.bytes)
            .ok_or(om9_core::Error::Overflow("hover snapshot estimate"))?;
        let held = self.controller.scene_cache_usage()?;
        let reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, Some(physical), held,
        )?;
        Ok((capture, buffers, reservation))
    }

    fn capture_hover_snapshot(&self, physical: Option<(u64, u64)>) -> om9_core::Result<HoverSnapshot> {
        let (capture, buffers, mut reservation) = self.reserve_hover_snapshot(physical)?;
        let scene = self.controller.scene().ok_or(om9_core::Error::Missing("hover scene"))?;
        let snapshot = scene.try_render_snapshot()?;
        let metadata = snapshot.metadata_storage_usage()?.bytes.checked_add(hover_snapshot_overhead())
            .ok_or(om9_core::Error::Overflow("hover snapshot metadata"))?;
        let scratch = snapshot.render_snapshot_pick_scratch_estimate()?.bytes;
        let required = metadata.checked_add(scratch).ok_or(om9_core::Error::Overflow("hover snapshot estimate"))?;
        if required > reservation.bytes() {
            let additional = required - reservation.bytes();
            let fresh = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
            self.controller.coordinator().grow_external_memory_with_held(&mut reservation, additional, fresh,
                self.controller.scene_cache_usage()?)?;
        }
        reservation.shrink_to(required)?;
        snapshot.transfer_memory_allowance(&mut reservation, self.controller.coordinator(), self.controller.scene_cache_usage()?)?;
        let mut snapshot = HoverSnapshot { scene: snapshot, _document: capture, _buffers: buffers, reservation };
        snapshot.reservation.mark_allocated(hover_snapshot_overhead())?;
        Ok(snapshot)
    }

    /// Reserve transient picking work even for a scene with few objects: one
    /// object can still contain a very large mesh.
    fn reserve_inline_hover_pick(
        &self, physical: Option<(u64, u64)>,
    ) -> om9_core::Result<om9_core::transaction::ExternalMemoryReservation> {
        let scene = self.controller.scene().ok_or(om9_core::Error::Missing("hover scene"))?;
        self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache,
            scene.nearest_pick_scratch_estimate()?.bytes.max(1), physical,
            self.controller.scene_cache_usage()?,
        )
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
            // Only the active viewport calls this method. A cached hit from a
            // different view is equally stale when this view has no pointer.
            self.hover_object = None;
            self.hover_picked_at = None;
            self.hover_retry_at = None;
            return;
        };
        let Some(scene) = self.controller.scene() else {
            self.hover_object = None;
            self.hover_picked_at = None;
            self.hover_retry_at = None;
            return;
        };
        let epoch = scene.epoch;
        let document = std::sync::Arc::downgrade(&scene.document);
        let preview = self.controller.preview_revision();
        let matches = |basis: &HoverBasis| basis.matches(&document, &view.camera, epoch, preview, viewport, pointer);
        let new_basis = || HoverBasis {
            document: document.clone(), camera: view.camera.clone(), epoch, preview, pointer, viewport,
        };
        if let Some(deadline) = self.hover_retry_at {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if !remaining.is_zero() { ui.ctx().request_repaint_after(remaining); }
        }
        // Picking iterates the scene, so it runs only when the pointer really moved or the view changed.
        let stale = self.hover_picked_at.as_ref().is_none_or(|basis| !matches(basis))
            || self.hover_retry_at.is_some_and(|at| std::time::Instant::now() >= at);
        if stale {
            // A pending worker must not keep painting a hit from an older input basis.
            self.hover_object = None;
        }
        let mut completed = false;
        if let Some((basis, receiver)) = self.hover_job.as_ref() {
            match receiver.try_recv() {
                Ok(hit) => {
                    let valid = matches(basis);
                    if valid {
                        self.hover_object = hit;
                        self.hover_picked_at = Some(new_basis());
                        completed = true;
                    } else {
                        self.hover_picked_at = None;
                        self.hover_object = None;
                    }
                    self.hover_job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.hover_job = None;
                    self.hover_picked_at = None;
                }
                _ => {}
            }
        }
        if stale && !completed && self.hover_job.is_none() {
            let tolerance = pick_tolerance(ui);
            if let Some(scene) = self.controller.scene() {
                if scene.objects.len() > 128 {
                    let physical = om9_native_owner::physical_memory()
                        .map(|sample| (sample.total_bytes, sample.available_bytes));
                    let admission = self.capture_hover_snapshot(physical);
                    if let Ok(snapshot) = admission {
                        self.hover_retry_at = None;
                        let camera = view.camera.clone();
                        let basis = new_basis();
                        let ctx = ui.ctx().clone();
                        let (tx, rx) = std::sync::mpsc::channel();
                        std::thread::spawn(move || {
                            let hit = snapshot
                                .scene
                                .pick(
                                    [pointer.x as f64, pointer.y as f64],
                                    &camera,
                                    viewport,
                                    tolerance,
                                    om9_scene::scene::PickIntent::Select,
                                )
                                .ok()
                                .and_then(|hit| hit.map(|h| h.root));
                            drop(snapshot);
                            let _ = tx.send(hit);
                            ctx.request_repaint();
                        });
                        self.hover_job = Some((basis, rx));
                    } else {
                        // Retry transient pressure at a bounded cadence even if
                        // the pointer remains still. Unsupported platforms do
                        // not repeatedly poll for a sample they cannot supply.
                        self.hover_picked_at = Some(new_basis());
                        if physical.is_some() || cfg!(windows) {
                            let delay = std::time::Duration::from_secs(1);
                            self.hover_retry_at = Some(std::time::Instant::now() + delay);
                            ui.ctx().request_repaint_after(delay);
                        } else {
                            self.hover_retry_at = None;
                        }
                    }
                    self.hover_object = None;
                } else {
                    let physical = om9_native_owner::physical_memory()
                        .map(|sample| (sample.total_bytes, sample.available_bytes));
                    match self.reserve_inline_hover_pick(physical) {
                        Ok(_reservation) => {
                            self.hover_retry_at = None;
                            self.hover_picked_at = Some(new_basis());
                            self.hover_object = scene
                                .pick(
                                    [pointer.x as f64, pointer.y as f64],
                                    &view.camera,
                                    viewport,
                                    tolerance,
                                    om9_scene::scene::PickIntent::Select,
                                )
                                .ok()
                                .and_then(|hit| hit.map(|h| h.root));
                        }
                        Err(_) => {
                            self.hover_object = None;
                            self.hover_picked_at = Some(new_basis());
                            let delay = std::time::Duration::from_secs(1);
                            self.hover_retry_at = Some(std::time::Instant::now() + delay);
                            ui.ctx().request_repaint_after(delay);
                        }
                    }
                }
            }
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
    /// The rect of a menu button or one of its items, as drawn this frame.
    pub fn menu_rect(&self, key: &str) -> Option<Rect> {
        self.menu_rects.get(key).copied()
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

    pub fn screen_icon_scale(&self) -> f32 {
        self.screen_icon_scale
    }

    pub fn toolbar_icon_edge(&self) -> f32 {
        self.toolbar_icon_edge
    }
    /// Fixed-column palettes also follow live window size. Keep their growth
    /// within the existing row capacity instead of changing column counts.
    fn palette_icon_scale(&self) -> f32 {
        0.9 + 0.1 * ((self.screen_icon_scale - 0.85) / 0.35).clamp(0., 1.)
    }

    /// The number of Undo steps (the adapter test checks that an edit is one step).
    pub fn undo_len(&self) -> usize {
        self.controller.coordinator().undo_len()
    }
    /// Edit one property of the single selected object through the same document
    /// command the editor uses (OM9-L3-03-14-007). The panel is the caller's job;
    /// this is the adapter that turns a field edit into one transaction.
    pub fn edit_selected_property(
        &mut self,
        edit: crate::session::PropertyEdit,
    ) -> om9_core::Result<()> {
        let selected: Vec<_> = self.controller.selected().iter().copied().collect();
        if selected.len() != 1 {
            return Err(om9_core::Error::Missing("single selection"));
        }
        let sources = self
            .controller
            .collect_object_sources(selected.into_iter())?;
        self.controller.edit_object_property(sources, edit)
    }
    /// Select exactly these objects (the palette and the Super Select tool need
    /// the same door the CMD path uses).
    pub fn select_objects(&mut self, ids: &[om9_core::identity::ObjectId]) -> om9_core::Result<()> {
        self.controller.select_objects(ids)
    }
    /// The Object Info line the receipt carries for a single selection: the same
    /// line the overlay paints. `None` when nothing (or more than one thing) is
    /// selected, so a receipt never claims information about a set.
    pub fn object_info_block(&self) -> Option<serde_json::Value> {
        self.controller
            .selected()
            .iter()
            .next()
            .filter(|_| self.controller.selected().len() == 1)
            .and_then(|id| {
                self.object_info_cache.borrow_mut().get(
                    self.controller.document(),
                    self.controller.coordinator().epoch(),
                    *id,
                    self.controller.coordinator().limits(),
                )
                    .ok()
            })
            .map(|info| {
                serde_json::json!({
                    "kind": info.kind,
                    "label": om9_core::object_info::label_text(&info),
                })
            })
    }
    /// The seven palette categories, in the order the Layers menu shows them
    /// (Matrix8 book 1, pages 54-57; OM9-L3-03-14-001).
    pub fn palette_categories(&self) -> [&'static str; 7] {
        om9_core::layers::LayerCategory::ALL.map(|category| category.label())
    }
    /// The category a palette slot belongs to, or its own label when it has none
    /// (slot 33, "Lights").
    pub fn palette_slot_label(&self, number: u8) -> Option<&'static str> {
        let slot = om9_core::layers::PresetSlot::new(number).ok()?;
        Some(slot.category_label())
    }
    pub fn main_menu_category(&self) -> usize {
        self.main_category
    }

    pub fn main_category_rect(&self, index: usize) -> Option<Rect> {
        self.main_category_rects.get(&index).copied()
    }
    /// Flush opt-in diagnostic output for an audit reader, outside the frame path.
    pub fn flush_interaction_log(&self) {
        if let Some(log) = &self.interaction_log {
            log.flush();
        }
    }

    /// Live navigation facts for the headless replay receipt: whether a gesture owns the pointer,
    /// how many camera poses are waiting to publish, and which gesture the last frame recorded.
    pub fn navigation_probe(&self) -> serde_json::Value {
        serde_json::json!({
            "gesture": self.navigation.gesture.is_some(),
            "pending_cameras": self.nav_cameras.len(),
            "actions": self.nav_actions.values().copied().collect::<Vec<_>>(),
            "render_due": self.nav_render_due,
        })
    }

    /// Delete is a global viewport key: handle it before widgets can consume it, so a focused
    /// command transcript or an open options popup cannot swallow it. A text draft keeps the key.
    fn delete_selection_from_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.controller.is_running()
            || self.controller.selected().is_empty()
            || !self.transcript.draft().is_empty()
            || self.controller.layer_dialog().is_some()
            || self.controller.file_operation().is_some()
            || self.audit.is_some()
        {
            return;
        }
        // A real text field keeps Delete for editing; every other focus owner still deletes.
        if ctx.egui_wants_keyboard_input()
            && ctx.memory(|memory| memory.focused())
                != Some(egui::Id::new("matrix-command-transcript"))
        {
            return;
        }
        let pressed = raw.events.iter().any(|event| {
            matches!(event, egui::Event::Key { key: Key::Delete, pressed: true, modifiers, .. }
                if !modifiers.any())
        });
        if !pressed {
            return;
        }
        raw.events.retain(|event| {
            !matches!(
                event,
                egui::Event::Key {
                    key: Key::Delete,
                    ..
                }
            )
        });
        if self.delete_selected_control_grip() {
            return;
        }
        let ids: Vec<_> = self.controller.selected().iter().copied().collect();
        let result = self.controller.delete_objects(&ids).map(|_| ());
        self.feedback(result);
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.stl_capture.as_ref().is_some_and(|capture| !capture.belongs_to_session(self.controller.document())) {
            self.cancel_stl_export_options();
            self.status.info("STL export cancelled after document change");
        }
        // The export dialog owns Escape; it must not also clear the selection
        // or cancel a viewport gesture in the underlying workspace.
        if self.stl_profile_draft.is_some()
            && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
        {
            self.cancel_stl_export_options();
            self.focus_cmd = true;
            self.status.info("STL export cancelled");
        }
        if self.controller.top11_grip_mode() != Some(om9_core::top11::GripMode::Control) {
            self.selected_grips.clear();
        }
        // Smoothed frame rate for the status line; a bad first dt never shows "inf fps".
        let dt = ui.input(|input| input.stable_dt);
        if dt.is_finite() && dt > 0.0 {
            let instant = 1.0 / dt;
            self.fps = if self.fps > 0.0 {
                self.fps * 0.9 + instant * 0.1
            } else {
                instant
            };
        }
        // The headless adapter has no raw_input_hook; preserve its events before CMD consumes Escape.
        if self.nav_frame_token != Some(ui.ctx().cumulative_frame_nr())
            && self.nav_raw_events.is_none()
        {
            self.nav_raw_events = Some(ui.input(|i| i.events.clone()));
        }
        if self.gpu_state.is_none() && self.nav_frame_token != Some(ui.ctx().cumulative_frame_nr())
        {
            self.nav_frame_started = std::time::Instant::now();
        }
        self.passes += 1;
        self.sync_pictureframe_textures(ui.ctx());
        if self
            .camera_audit
            .as_ref()
            .is_some_and(|audit| audit.remaining > 0)
        {
            self.camera_audit_frame(ui);
        }
        self.screen_icon_scale = responsive_icon_scale(ui.ctx().content_rect());
        if !self.viewport_menu_owns_frame
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F4))
        {
            if self.controller.phase() == crate::session::CmdPhase::Idle {
                self.world_datum_visible = !self.world_datum_visible;
            } else {
                let result = self.controller.submit_world_origin();
                self.feedback(result);
            }
        }
        if !self.viewport_menu_owns_frame
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F10))
            && !self.controller.is_running()
        {
            self.top11_action("ControlPoints");
        }
        if self.controller.is_running() && !self.focus_cmd {
            for (key, kind) in [
                (Key::E, om9_scene::osnap::SnapKind::End),
                (Key::M, om9_scene::osnap::SnapKind::Mid),
            ] {
                // Declared accelerators own their keys. Editors retain normal text.
                use crate::command_options::CommandOptionInterface;
                if ui.ctx().egui_wants_keyboard_input()
                    || self.controller.command_options().iter().any(|option| {
                        option.shortcut.is_some_and(|shortcut| {
                            key.name().eq_ignore_ascii_case(&shortcut.to_string())
                        })
                    })
                {
                    continue;
                }
                if ui.input_mut(|input| input.consume_key(Modifiers::NONE, key)) {
                    self.set_snap_override(Some(kind));
                    self.status.info(&format!("{} one-shot", kind.label()));
                }
            }
        }
        if !self.controller.is_running() {
            self.snap = None;
            self.smarttrack.clear();
        }
        self.gpu_drawn_views.clear();
        if ui.input(|input| !input.focused || input.key_pressed(Key::Escape)
            || input.pointer.button_pressed(egui::PointerButton::Primary)) {
            self.primary_press_scene = None;
        }
        let stale_gesture = self.gumball_basis.as_ref().is_some_and(|basis| {
            basis.epoch != self.controller.coordinator().epoch()
                || basis.selection != self.controller.selection_state().revision
                || (!ui.input(|i| i.focused) && self.os_audit.is_none())
        });
        if stale_gesture {
            self.cancel_gumball();
            self.status
                .info("Transform cancelled: selection, document or focus changed");
        }
        let stale_grip = self.grip_drag.as_ref().is_some_and(|drag| {
            drag.epoch != self.controller.coordinator().epoch()
                || drag.selection != self.controller.selection_state().revision
                || self.controller.document().resolve(&drag.source).is_err()
                || self.controller.top11_grip_mode() != Some(drag.handle.mode)
                || self.controller.workspace.active != drag.view
                || self.controller.workspace.maximized != drag.maximized
                || self
                    .controller
                    .workspace
                    .views
                    .get(&drag.view)
                    .is_none_or(|view| {
                        view.camera != drag.workspace_camera || view.cplane != drag.view_plane
                    })
                || !ui.input(|i| i.focused)
        });
        if stale_grip {
            self.grip_drag = None;
            self.status
                .info("Grip edit cancelled: selection, document, view or focus changed");
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
            self.snap = None;
            self.snap_state.cancel();
            self.smarttrack.clear();
            if self.ambiguous_pos.take().is_some() {
                self.ambiguous.clear();
                self.ambiguous_extract_context = None;
                self.ambiguous_sphere_context = None;
                self.ambiguous_slab_context = None;
                self.ambiguous_pipe_context = None;
                if !self.controller.is_running() {
                    ui.input_mut(|input| {
                        input.consume_key(Modifiers::NONE, Key::Escape);
                    });
                }
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
                // Escape ends a Top 11 prompt or grip display without touching the
                // selection, matching the manual's Esc rule for those commands.
                if self.controller.top11_grip_mode().is_some() || self.grip_drag.is_some() {
                    self.grip_drag = None;
                    self.selected_grips.clear();
                } else if !dragging && !self.controller.is_running() {
                    // No gesture in flight: Esc cancels the selection itself, which is the manual's rule.
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
            // The command editor normally keeps focus even while the user works
            // in a viewport. Its empty draft must not swallow document Undo.
            let editor = egui::Id::new("matrix-command-transcript");
            let focus = ctx.memory(|memory| memory.focused());
            let document_shortcut = !self.controller.is_running()
                && self.transcript.draft().is_empty()
                && (focus.is_none() || focus == Some(editor))
                && !egui::Popup::is_any_open(&ctx);
            if document_shortcut && ctx.input_mut(|input| input.consume_key(command, Key::Z)) {
                let result = self.controller.undo();
                self.feedback(result);
            }
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
                self.hover_view = None;
                self.transcript.set_draft("");
                self.transcript
                    .set_prompt(&self.controller.prompt_with_options());
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
            .frame(egui::Frame::new().fill(chrome::PANEL_BG).inner_margin(2))
            .show(ui, |ui| {
                self.sidebar_rect = Some(ui.max_rect());
                // Icons follow the OUTER width (inner content plus frame chrome).
                self.left_icon_edge = (dock_layout::rail_icon_edge(
                    ui.max_rect().width() + dock_layout::LEFT_RAIL_CHROME_PX,
                ) * self.screen_icon_scale)
                    .clamp(dock_layout::ICON_MIN_EDGE, dock_layout::ICON_MAX_EDGE)
                    .round();
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
            .frame(egui::Frame::new().fill(command::COMMAND_BG).inner_margin(3))
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
                .frame(egui::Frame::new().fill(chrome::PANEL_BG).inner_margin(3))
                .show(ui, |ui| {
                    self.right_dock_rect = Some(ui.max_rect());
                    self.column_icon_edge = (dock_layout::rail_icon_edge(
                        ui.max_rect().width() + dock_layout::LEFT_RAIL_CHROME_PX,
                    ) * self.screen_icon_scale)
                        .clamp(dock_layout::ICON_MIN_EDGE, dock_layout::ICON_MAX_EDGE)
                        .round();
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
                    .fill(Color32::from_rgb(8, 11, 15))
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
                let rect = ui
                    .available_rect_before_wrap()
                    .intersect(ui.clip_rect())
                    .intersect(ui.ctx().content_rect());
                let views = self.controller.workspace.visible_views();
                self.view_rects.clear();
                // The tiling lives in workbench_regions; the frame feeds it the
                // rectangle egui actually left for the canvas.
                let center = RegionRect::new(rect.min.x, rect.min.y, rect.width(), rect.height());
                match tile_views(center, &views) {
                    Ok(slots) => {
                        let mut panes = Vec::new();
                        for slot in slots {
                            let rect = Rect::from_min_size(
                                egui::pos2(slot.rect.x, slot.rect.y),
                                Vec2::new(slot.rect.width, slot.rect.height),
                            );
                            self.view_rects.insert(slot.view, rect);
                            let canvas = Rect::from_min_max(
                                egui::pos2(rect.min.x, rect.min.y + 18.),
                                rect.max,
                            );
                            let response = ui.interact(
                                canvas,
                                ui.id().with(("canvas", slot.view.title())),
                                Sense::click_and_drag(),
                            );
                            panes.push((slot.view, rect, response));
                        }
                        self.navigation_frame(ui, &panes);
                        let render_started = std::time::Instant::now();
                        for (id, rect, response) in panes {
                            self.viewport_ui(ui, id, rect, response);
                        }
                        self.navigation_frame_finished(ui, render_started);
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
            let stored = crate::f6::F6WindowPreference {
                x: stored.x,
                y: stored.y,
                width: stored.width,
                height: stored.height,
                pinned: self.f6_pinned,
            }
            .clamp(ctx.content_rect());
            let context = self.f6_context();
            let custom = self.f6_custom.context();
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
                    ui.separator();
                    ui.label(format!("{} actions", context.title));
                    for action in &context.actions {
                        let response = ui.add_enabled(
                            action.availability.enabled(),
                            egui::Button::new(&action.label),
                        );
                        let response = if let Some(reason) = action.availability.reason() {
                            response.on_hover_text(reason)
                        } else {
                            response
                        };
                        if action.availability.enabled() && response.clicked() {
                            self.invoke(&action.id, InvocationOrigin::F6);
                        }
                    }
                    if !custom.actions.is_empty() {
                        ui.separator();
                        ui.label("Custom");
                        for action in &custom.actions {
                            let response = ui.add_enabled(
                                action.availability.enabled(),
                                egui::Button::new(&action.label),
                            );
                            let response = if let Some(reason) = action.availability.reason() {
                                response.on_hover_text(reason)
                            } else {
                                response
                            };
                            if action.availability.enabled() && response.clicked() {
                                self.invoke(&action.id, InvocationOrigin::F6);
                            }
                        }
                    }
                    ui.separator();
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
        self.gumball_settings_ui(&ctx);
        self.undo_settings_ui(&ctx);
        self.stl_profile_ui(&ctx);
        self.persist_gumball_settings();
        self.os_audit_frame(&ctx);
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
    /// Route one event stream before painting any pane; layout retries cannot consume it again.
    fn navigation_frame(&mut self, ui: &mut egui::Ui, panes: &[(ViewId, Rect, egui::Response)]) {
        let token = ui.ctx().cumulative_frame_nr();
        if self.nav_frame_token == Some(token) {
            self.nav_render_due = false;
            return;
        }
        self.nav_frame_token = Some(token);
        self.nav_actions.clear();
        let input_time = ui.input(|i| i.time);
        let origin = *self
            .nav_time_origin
            .get_or_insert((input_time, std::time::Instant::now()));
        let now = if self.gpu_state.is_some() {
            std::time::Instant::now()
        } else {
            origin.1 + std::time::Duration::from_secs_f64((input_time - origin.0).clamp(0., 1e9))
        };
        self.nav_sample_now = now;
        let events = self
            .nav_raw_events
            .take()
            .unwrap_or_else(|| ui.input(|i| i.events.clone()));
        let document = self.controller.document().id;
        if self.nav_document != Some(document) {
            self.navigation = Default::default();
            self.nav_cameras.clear();
            self.nav_camera_sources.clear();
            self.scheduler.idle();
            self.nav_document = Some(document);
        }
        let blocked = events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                }
            )
        }) || self.viewport_menu_owns_frame
            || self.gumball_session.is_some()
            || self.controller.layer_dialog().is_some()
            || self.controller.file_operation().is_some()
            || egui::Popup::is_any_open(ui.ctx());
        if self.navigation_audit.is_some()
            && let Some(log) = self.interaction_log.as_mut()
        {
            log.record(serde_json::json!({"event":"navigation_route","focused":ui.input(|i|i.focused),"blocked":blocked,"events":events.len()}));
        }
        if blocked {
            self.navigation.cancel();
            self.nav_cameras.clear();
            self.nav_camera_sources.clear();
        }
        if self
            .navigation
            .gesture
            .is_some_and(|g| !panes.iter().any(|p| p.0 == g.view))
        {
            self.navigation.cancel();
        }
        self.navigation
            .views
            .retain(|id, _| panes.iter().any(|p| p.0 == *id));
        self.nav_cameras
            .retain(|id, _| panes.iter().any(|p| p.0 == *id));
        self.nav_camera_sources
            .retain(|id, _| panes.iter().any(|p| p.0 == *id));
        for (id, _, _) in panes {
            if let Some(v) = self.controller.workspace.views.get(id) {
                if self
                    .nav_camera_sources
                    .get(id)
                    .is_some_and(|source| *source != v.camera.revision)
                {
                    self.nav_cameras.remove(id);
                    self.nav_camera_sources.remove(id);
                    self.navigation.cancel();
                }
                if !self.nav_cameras.contains_key(id) {
                    self.navigation.sync(*id, &v.camera, now);
                }
            }
        }
        let speed = crate::navigation::finite_speed(
            std::env::var("OM9_NAV_SPEED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.),
        );
        let mut cursor = ui.input(|i| i.pointer.interact_pos());
        let mut shift = ui.input(|i| i.modifiers.shift);
        for event in events {
            self.navigation.input_seq = self.navigation.input_seq.wrapping_add(1);
            if matches!(
                &event,
                egui::Event::PointerMoved(_)
                    | egui::Event::PointerButton { .. }
                    | egui::Event::MouseWheel { .. }
                    | egui::Event::WindowFocused(_)
                    | egui::Event::Key {
                        key: Key::Escape,
                        ..
                    }
            ) && let Some(log) = self.interaction_log.as_mut()
            {
                log.record(serde_json::json!({"event":"navigation_input","frame_id":token,"input_seq":self.navigation.input_seq,"raw_event":format!("{event:?}")}));
            }
            match event {
                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers,
                } => {
                    cursor = Some(pos);
                    shift = modifiers.shift;
                    if !pressed && self.navigation.gesture.is_some_and(|g| g.button == button) {
                        let finish_curve = !blocked
                            && button == egui::PointerButton::Secondary
                            && !modifiers.any()
                            && self.navigation.gesture.is_some_and(|g| {
                                !g.moved
                                    && g.start.distance(pos) <= 4.0
                                    && g.curve_command_run
                                        == Some(self.controller.active_command_input_snapshot().run)
                                    && panes
                                        .iter()
                                        .any(|(id, rect, _)| *id == g.view && rect.contains(pos))
                            })
                            && self
                                .controller
                                .running_feature()
                                .is_some_and(|id| id.starts_with("OM9-CURVE-"));
                        self.navigation.end_drag();
                        if finish_curve {
                            self.submit_draft();
                        }
                    }
                    if pressed
                        && !blocked
                        && matches!(
                            button,
                            egui::PointerButton::Secondary | egui::PointerButton::Middle
                        )
                        && let Some((id, _, _)) = panes
                            .iter()
                            .find(|(_, r, _)| pos.y >= r.min.y + 18. && r.contains(pos))
                        && self.navigation.gesture.is_none()
                        && let Some(v) = self.controller.workspace.views.get_mut(id)
                    {
                        if let Some(displayed) = self
                            .gpu_viewports
                            .get(id)
                            .and_then(|p| p.displayed_camera())
                            .filter(|c| **c != v.camera)
                        {
                            let mut pose = displayed.clone();
                            pose.revision = v.camera.revision.wrapping_add(1);
                            v.camera = pose;
                        }
                        self.nav_cameras.remove(id);
                        self.nav_camera_sources.remove(id);
                        self.navigation.begin_drag(*id, button, pos, &v.camera, now);
                        if let Some(gesture) = self.navigation.gesture.as_mut() {
                            gesture.pan = shift;
                            if self
                                .controller
                                .running_feature()
                                .is_some_and(|id| id.starts_with("OM9-CURVE-"))
                            {
                                gesture.curve_command_run =
                                    Some(self.controller.active_command_input_snapshot().run);
                            }
                        }
                    }
                }
                egui::Event::PointerMoved(pos) => {
                    cursor = Some(pos);
                    if let Some(g) = self.navigation.gesture.as_mut() {
                        g.moved |= g.start.distance(pos) > 4.0;
                    }
                    if let Some(g) = self.navigation.gesture {
                        if let Some((_, rect, _)) = panes.iter().find(|p| p.0 == g.view) {
                            let vp = Viewport {
                                origin: [rect.min.x as f64, rect.min.y as f64],
                                size: [rect.width() as f64, rect.height() as f64],
                            };
                            if let Some(view) = self.controller.workspace.views.get(&g.view) {
                                let current = self.nav_cameras.get(&g.view).unwrap_or(&view.camera);
                                let pan = g.button == egui::PointerButton::Middle
                                    || g.pan
                                    || shift
                                    || matches!(
                                        view.camera.projection,
                                        om9_scene::camera::Projection::Orthographic { .. }
                                    );
                                if let Some(camera) = self.navigation.drag(
                                    current,
                                    vp,
                                    [(pos.x - g.last.x) as f64, (pos.y - g.last.y) as f64],
                                    pan,
                                    now,
                                ) {
                                    self.nav_camera_sources
                                        .entry(g.view)
                                        .or_insert(view.camera.revision);
                                    self.nav_cameras.insert(g.view, camera);
                                    self.nav_actions
                                        .insert(g.view, if pan { "pan" } else { "orbit" });
                                }
                            }
                        }
                        if let Some(g) = self.navigation.gesture.as_mut() {
                            g.last = pos;
                        }
                    }
                }
                egui::Event::MouseWheel {
                    unit,
                    delta,
                    modifiers,
                    phase: _,
                } if !blocked => {
                    if let Some(pos) = cursor
                        && let Some((id, rect, _response)) = panes
                            .iter()
                            .find(|(_, r, response)| r.contains(pos) && response.hovered())
                    {
                        let options = ui.ctx().options(|o| o.input_options);
                        let mut delta = match unit {
                            egui::MouseWheelUnit::Point => delta,
                            // Winit maps one Windows wheel notch to one Line unit.
                            egui::MouseWheelUnit::Line => delta * 40.,
                            egui::MouseWheelUnit::Page => delta * ui.ctx().content_rect().height(),
                        };
                        if modifiers.matches_any(options.horizontal_scroll_modifier)
                            && !modifiers.matches_any(options.vertical_scroll_modifier)
                        {
                            delta = Vec2::new(delta.x + delta.y, 0.);
                        }
                        if !modifiers.matches_any(options.horizontal_scroll_modifier)
                            && modifiers.matches_any(options.vertical_scroll_modifier)
                        {
                            delta = Vec2::new(0., delta.x + delta.y);
                        }
                        let vp = Viewport {
                            origin: [rect.min.x as f64, rect.min.y as f64],
                            size: [rect.width() as f64, rect.height() as f64],
                        };
                        if let Some(view) = self.controller.workspace.views.get(id) {
                            self.navigation.wheel_displayed(
                                *id,
                                &view.camera,
                                self.gpu_viewports
                                    .get(id)
                                    .and_then(|p| p.displayed_camera())
                                    .unwrap_or(&view.camera),
                                vp,
                                [pos.x as f64, pos.y as f64],
                                delta.y as f64,
                                now,
                                speed,
                            );
                            self.nav_actions.insert(*id, "zoom");
                        }
                    }
                }
                egui::Event::WindowFocused(false) => {
                    self.navigation.cancel();
                    self.nav_cameras.clear();
                    self.nav_camera_sources.clear();
                }
                _ => {}
            }
        }
        if !self.nav_actions.is_empty() && self.interaction_log.is_some() {
            // Timestamp only input that actually changed navigation, not unrelated UI events.
            self.nav_received = Some(self.nav_frame_started);
        }
        if self
            .navigation
            .gesture
            .is_some_and(|g| !ui.input(|i| i.pointer.button_down(g.button)))
        {
            self.navigation.end_drag();
        }
        self.nav_render_due = self.scheduler.due(now) || !self.navigation.needs_tick();
        if self.nav_render_due {
            for (id, camera) in std::mem::take(&mut self.nav_cameras) {
                if let Some(view) = self.controller.workspace.views.get_mut(&id)
                    && self.nav_camera_sources.remove(&id) == Some(view.camera.revision)
                {
                    view.camera = camera;
                    self.nav_actions.entry(id).or_insert("camera");
                }
            }
            for (id, _, _) in panes {
                if let Some(view) = self.controller.workspace.views.get(id)
                    && let Some(camera) = self.navigation.tick(*id, &view.camera, now)
                {
                    self.controller.workspace.views.get_mut(id).unwrap().camera = camera;
                    self.nav_actions.entry(*id).or_insert("zoom");
                }
            }
            if self.navigation.needs_tick() {
                self.scheduler.begin(now);
            } else {
                self.scheduler.idle();
            }
        }
    }
    fn navigation_frame_finished(&mut self, ui: &mut egui::Ui, started: std::time::Instant) {
        if let Some(state) = self.gpu_state.as_ref() {
            crate::gpu_viewport::submit_window(&self.shared_scene, state);
            for port in self.gpu_viewports.values_mut() {
                port.window_submitted(&state.queue);
            }
        }
        if self.nav_logged_token == self.nav_frame_token {
            return;
        }
        self.nav_logged_token = self.nav_frame_token;
        let moving = self.navigation.views.values().any(|s| {
            matches!(
                s.phase,
                crate::navigation::NavigationPhase::Dragging
                    | crate::navigation::NavigationPhase::Settling
            )
        });
        if self.nav_render_due && moving {
            let gpu_ms = self
                .gpu_viewports
                .values()
                .filter_map(|p| {
                    p.gpu_sample
                        .filter(|s| s.0 >= self.nav_frame_token.unwrap_or(0).saturating_sub(4))
                        .map(|s| s.1)
                })
                .fold(0., f64::max);
            self.scheduler.observe(
                self.nav_sample_now,
                self.nav_frame_started
                    .elapsed()
                    .max(std::time::Duration::from_secs_f64(gpu_ms / 1000.)),
            );
        }
        if !moving {
            self.scheduler.reset_quality_feedback();
        }
        if let Some(log) = self.interaction_log.as_mut() {
            if moving && self.nav_render_due {
                let counters: BTreeMap<_, _> = self.gpu_viewports.iter()
                    .map(|(id, port)| (id.title(), port.diagnostics()))
                    .collect();
                let targets = crate::gpu_viewport::window_target_diagnostics(&self.shared_scene, self.gpu_viewports.values());
                let overlays = crate::gpu_viewport::window_overlay_diagnostics(&self.shared_scene, self.gpu_viewports.values());
                log.record(serde_json::json!({"event":"viewport_counters","frame_id":self.nav_frame_token,"views":counters,"window_targets":targets,"window_overlays":overlays}));
            }
            for (&id, view) in &self.controller.workspace.views {
                let state = self.navigation.views.get(&id);
                let port = self.gpu_viewports.get(&id);
                if state.is_none_or(|s| s.phase == crate::navigation::NavigationPhase::Idle)
                    && !self.nav_actions.contains_key(&id)
                    && !port.is_some_and(|p| {
                        p.preparing()
                            || p.encoded_frame == self.nav_frame_token
                            || p.published_frame == self.nav_frame_token
                    })
                {
                    continue;
                }
                let stats = port.and_then(|p| p.frame_stats());
                log.record(serde_json::json!({"event":"viewport_prepare","frame_id":self.nav_frame_token,"view":id.title(),"preparing":port.map(|p|p.preparing()),"pending_geometry_bytes":port.map(|p|p.pending_geometry_bytes()),"error":port.and_then(|p|p.preparation_error())}));
                log.record(serde_json::json!({"event":"nav_frame","pixels_per_point":ui.ctx().pixels_per_point(),"zoom_factor":ui.ctx().zoom_factor(),"document_id":self.controller.document().id,"scene_epoch":self.controller.scene().map(|s|s.epoch),"selection_revision":self.controller.selection_state().revision,"physical_extent":self.view_rects.get(&id).map(|r|[(r.width()*ui.ctx().pixels_per_point()).round(),(r.height()*ui.ctx().pixels_per_point()).round()]),"encoded_frame_id":port.and_then(|p|p.encoded_frame),"published_frame_id":port.and_then(|p|p.published_frame),"final_complete":port.map(|p|p.final_complete()),"frame_id":self.nav_frame_token,"input_seq":self.navigation.input_seq,"view":id.title(),"active_view":self.controller.workspace.active.title(),"owner_view":self.navigation.gesture.map(|g|g.view.title()),"phase":state.map(|s|s.phase.name()),"render_due":self.nav_render_due,"interval_ms":self.scheduler.interval.map(|d|d.as_secs_f64()*1000.),"work_ms":self.nav_frame_started.elapsed().as_secs_f64()*1000.,"pane_work_ms":started.elapsed().as_secs_f64()*1000.,"hz":self.scheduler.hz(),"next_deadline_in_ms":self.scheduler.wait(std::time::Instant::now()).as_secs_f64()*1000.,"quality":self.scheduler.quality.index(),"revision":view.camera.revision,"displayed_revision":port.and_then(|p|p.displayed_camera().map(|c|c.revision)),"draw_counter_scope":"scene_mesh_batch","generation":self.navigation.generation(id),"prepare_ms":port.map(|p|p.last_prepare_ms()),"submit_cpu_ms":port.map(|p|p.last_draw_ms()),"gpu_ms":port.and_then(|p|p.gpu_sample.map(|s|s.1)),"gpu_frame_id":port.and_then(|p|p.gpu_sample.map(|s|s.0)),"receive_to_submit_ms":self.nav_received.map(|t|t.elapsed().as_secs_f64()*1000.),"refinement_ms":port.and_then(|p|p.refinement_ms()),"present_ms":null,"scene_objects":stats.map(|s|s.objects),"visible":stats.map(|s|s.visible),"draw_calls":stats.map(|s|s.draw_calls)}));
                if let Some(action) = self.nav_actions.get(&id) {
                    log.record(serde_json::json!({"event":"camera","frame_id":self.nav_frame_token,"action":action,"view":id.title(),"revision":view.camera.revision,"eye":view.camera.eye,"target":view.camera.target,"frame_ms":crate::interaction_log::frame_ms(ui.input(|i|i.stable_dt)),"dt_ms":self.scheduler.interval.map(|d|d.as_secs_f64()*1000.),"timing_source":"window_monotonic","prepare_ms":port.map(|p|p.last_prepare_ms()),"draw_ms":port.map(|p|p.last_draw_ms())}));
                }
            }
        }
        if self.nav_actions.keys().any(|id| {
            self.gpu_viewports.get(id).is_some_and(|p| p.encoded_frame == self.nav_frame_token)
        })
        {
            self.nav_received = None;
        }
        if self.navigation.needs_tick() {
            let now = if self.gpu_state.is_some() {
                std::time::Instant::now()
            } else {
                self.nav_sample_now + started.elapsed()
            };
            ui.ctx().request_repaint_after(self.scheduler.wait(now));
        } else {
            self.scheduler.idle();
        }
    }

    fn viewport_command_point(&self, point: Point3) -> Point3 {
        let point = self.controller.constrain_point(point).unwrap_or(point);
        let point = self
            .snap
            .as_ref()
            .map_or(point, |candidate| candidate.world);
        if self.controller.project() {
            self.controller.project_to_plane(point).unwrap_or(point)
        } else {
            point
        }
    }

    fn viewport_ui(&mut self, ui: &mut egui::Ui, id: ViewId, rect: Rect, response: egui::Response) {
        let primary_pressed = ui.input(|input| input.pointer.button_pressed(egui::PointerButton::Primary));
        let primary_press_origin = if primary_pressed {
            ui.input(|input| input.pointer.press_origin().or_else(|| input.events.iter().find_map(|event| {
                match event {
                    egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, .. } => Some(*pos),
                    _ => None,
                }
            })))
        } else { None };
        // egui hit testing uses last pass's widget order. The final pointer
        // position may already have left an overlapping same-layer control,
        // so response.hovered() alone cannot establish who owned the press.
        let press_unobstructed = primary_press_origin.is_some_and(|pos| {
            if !response.interact_rect.contains(pos) || !ui.clip_rect().contains(pos) || ui.ctx().layer_id_at(pos) != Some(ui.layer_id()) {
                return false;
            }
            let rects = ui.ctx().interactive_rects_last_pass();
            rects.iter().position(|candidate| *candidate == response.interact_rect)
                .is_some_and(|index| !rects[index + 1..].iter().any(|candidate| candidate.contains(pos)))
        });
        // The last pointer position can already be outside this pane when a
        // press and fast move arrive together. Only recover a possible Gumball
        // gesture when the original press was in the unobstructed pane.
        let fast_exit_press = !response.hovered() && !self.controller.selected().is_empty()
            && !self.controller.is_running() && !self.viewport_menu_owns_frame
            && !egui::Popup::is_any_open(ui.ctx()) && ui.input(|input| input.focused)
            && press_unobstructed;
        if self.gpu_state.is_some()
            && (response.hovered() || fast_exit_press)
            && press_unobstructed
            && primary_pressed
        {
            self.primary_press_scene = self.gpu_viewports.get(&id)
                .and_then(|port| {
                    let origin = primary_press_origin?;
                    Some((id, port.displayed_key()?, port.displayed_camera()?.clone(), [f64::from(origin.x), f64::from(origin.y)]))
                });
        }
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let Some(mut view) = self.controller.workspace.views.get(&id).cloned() else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, 0, Color32::BLACK);

        let gpu_drew = if let Some(state) = self.gpu_state.as_ref() {
            // Each pane retains its own colour/depth target and camera. The close-range batch mode is
            // shared by all panes, so the shared scene is not rebuilt once per viewport camera.
            let shared_scene = self.shared_scene.clone();
            let request = crate::gpu_viewport::ViewportRenderRequest {
                frame_id: self.nav_frame_token.unwrap_or(0),
                generation: self.navigation.generation(id),
                moving: self.navigation.moving(id),
                render_due: self.nav_render_due,
                quality: self.scheduler.quality,
            };
            let needs_inventory = self.gpu_viewports.get(&id).is_none_or(|port| {
                port.needs_target_inventory(rect, ui.ctx().pixels_per_point(), request)
            });
            let window_targets = needs_inventory.then(|| {
                let targets = crate::gpu_viewport::window_target_diagnostics(
                    &self.shared_scene, self.gpu_viewports.values(),
                )?.unique_bytes;
                targets.checked_add(crate::gpu_viewport::window_known_buffer_bytes(&self.shared_scene, self.gpu_viewports.values())?)
            });
            let close = false;
            let port = self
                .gpu_viewports
                .entry(id)
                .or_insert_with(|| crate::gpu_viewport::GpuViewport::new(state, shared_scene));
            if let Some(window_targets) = window_targets { port.set_window_known_gpu_bytes(window_targets); }
            // Hit testing uses the image under the primary press. Keep that
            // camera through release, but still render Gumball preview changes.
            let primary_gesture = ui.input(|input| {
                input.pointer.primary_down()
                    || input.pointer.button_released(egui::PointerButton::Primary)
            });
            if primary_gesture && let Some(camera) = port.displayed_camera() {
                if *camera != view.camera {
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
                }
                view.camera = camera.clone();
            }
            port.set_render_request(request);
            port.hold_publication_on_press(primary_gesture && self.gumball_session.is_none()
                && self.primary_press_scene.as_ref().is_some_and(|(press_id, _, _, _)| *press_id == id));
            port.set_display_mode(view.mode);
            let drew = port.show(
                ui,
                rect,
                state,
                &self.controller,
                crate::gpu_viewport::scene_camera(&view.camera),
                view.camera.revision,
                &view.camera,
                self.controller.scene(),
                view.cplane,
                self.controller.selected(),
                self.controller.selection_state().revision,
                self.controller.preview_revision(),
                self.controller.solid_boolean_two_hidden_sources(),
                close,
            );
            if port.final_complete() {
                self.navigation
                    .completed(id, self.navigation.generation(id));
            }
            if let Some(camera) = port.displayed_camera() {
                view.camera = camera.clone();
            }
            if let Some(error) = port.preparation_error() {
                painter.text(
                    rect.min + Vec2::new(8., 36.),
                    egui::Align2::LEFT_TOP,
                    error,
                    egui::FontId::proportional(13.),
                    Color32::LIGHT_RED,
                );
            } else if port.refinement_delayed() {
                painter.text(
                    rect.min + Vec2::new(8., 36.),
                    egui::Align2::LEFT_TOP,
                    "Đang hoàn thiện hình ảnh",
                    egui::FontId::proportional(13.),
                    Color32::LIGHT_GRAY,
                );
            }
            drew
        } else {
            false
        };
        // A current document hit is unsafe while the viewport is still showing
        // another scene revision. A captured gesture has its own stale guards.
        let primary_gesture = ui.input(|input| input.pointer.primary_down()
            || input.pointer.button_pressed(egui::PointerButton::Primary)
            || input.pointer.button_released(egui::PointerButton::Primary));
        let published_scene_current = self.gpu_state.is_none()
            || self.controller.scene().is_some_and(|scene| {
                self.gpu_viewports.get(&id).is_some_and(|port| {
                    port.displays_content(scene, self.controller.selection_state().revision,
                        self.controller.preview_revision(), view.cplane.revision,
                        self.controller.solid_boolean_two_hidden_sources(), view.mode)
                        && (!primary_gesture || self.primary_press_scene.as_ref().is_some_and(|(press_id, press_key, press_camera, _)| {
                            *press_id == id && port.displayed_key().is_some_and(|key| key.same_input_basis(*press_key))
                                && port.displayed_camera() == Some(press_camera)
                        }))
                })
            });
        // Opt-in diagnostic: correlate a primary gesture with the image and
        // camera that were eligible for input, even if routing rejects it.
        let primary_edge = ui.input(|input| input.pointer.button_pressed(egui::PointerButton::Primary)
            || input.pointer.button_released(egui::PointerButton::Primary));
        if self.interaction_log.is_some() && !self.controller.selected().is_empty()
            && (primary_edge || response.drag_started_by(egui::PointerButton::Primary)
                || response.drag_stopped_by(egui::PointerButton::Primary))
            && (response.hovered() || response.is_pointer_button_down_on() || self.gumball_session.is_some()) {
            let port = self.gpu_viewports.get(&id);
            let captured = self.primary_press_scene.as_ref();
            let content_matches = self.controller.scene().is_some_and(|scene| port.is_some_and(|p| {
                p.displays_content(scene, self.controller.selection_state().revision,
                    self.controller.preview_revision(), view.cplane.revision,
                    self.controller.solid_boolean_two_hidden_sources(), view.mode)
            }));
            let press_matches = captured.is_some_and(|(press_id, press_key, press_camera, _)| {
                *press_id == id && port.is_some_and(|p| p.displayed_key().is_some_and(|key| key.same_input_basis(*press_key))
                    && p.displayed_camera() == Some(press_camera))
            });
            if let Some(log) = self.interaction_log.as_mut() {
                log.record(serde_json::json!({"event":"gumball_route", "frame_id":self.nav_frame_token,
                    "input_seq":self.navigation.input_seq, "view":id.title(), "active_view":self.controller.workspace.active.title(),
                    "primary_pressed":ui.input(|i|i.pointer.button_pressed(egui::PointerButton::Primary)),
                    "primary_released":ui.input(|i|i.pointer.button_released(egui::PointerButton::Primary)),
                    "drag_started":response.drag_started_by(egui::PointerButton::Primary),
                    "drag_stopped":response.drag_stopped_by(egui::PointerButton::Primary),
                    "hovered":response.hovered(), "pointer":response.interact_pointer_pos().map(|p|[p.x,p.y]),
                    "press_origin":ui.input(|i|i.pointer.press_origin()).map(|p|[p.x,p.y]),
                    "scene_current":published_scene_current, "content_matches":content_matches,
                    "press_matches":press_matches, "gesture_active":self.gumball_session.is_some(),
                    "navigation_moving":self.navigation.moving(id), "selected":self.controller.selected().len(),
                    "camera_revision":self.controller.workspace.views.get(&id).map(|v|v.camera.revision),
                    "displayed_revision":port.and_then(|p|p.displayed_camera().map(|camera|camera.revision)),
                    "encoded_frame_id":port.and_then(|p|p.encoded_frame), "published_frame_id":port.and_then(|p|p.published_frame)}));
            }
        }
        if gpu_drew {
            self.gpu_drawn_views.insert(id);
            paint_fps(ui, rect);
        }
        if !gpu_drew {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                if self.gpu_viewports.get(&id).is_some_and(|p| p.preparing()) {
                    "Preparing viewport"
                } else {
                    "GPU viewport unavailable"
                },
                egui::FontId::proportional(13.),
                Color32::LIGHT_RED,
            );
        }
        self.paint_picture_frames(&painter, &view, viewport, rect);
        if self.controller.solid_boolean_two_hidden_sources().is_none() {
            self.selection_frame(ui, id, &view, viewport, rect);
            self.object_info_label(ui, &view, viewport, rect);
            self.subselection_overlay(ui, &view, viewport, rect);
            self.top11_grip_overlay(ui, &view, viewport, rect);
            if published_scene_current || self.grip_drag.is_some() {
                self.top11_grip_drag_ui(ui, &response, &view, viewport);
            }
            if published_scene_current || self.gumball_session.is_some() {
                self.gumball_ui(ui, &response, &view, viewport, rect);
            }
        }
        self.builder_handles_ui(ui, &response, &view, viewport, rect);
        if published_scene_current {
            self.selection_band(ui, &response, &view, viewport, rect);
        } else if self.band_view == Some(id) {
            self.band_start = None;
            self.band_view = None;
        }
        if self.controller.solid_boolean_two_hidden_sources().is_none()
            && id == self.controller.workspace.active
            && !self.navigation.moving(id)
            && published_scene_current
        {
            self.hover_frame(ui, &response, &view, viewport, rect);
            self.ambiguous_popup(ui);
        } else if id == self.controller.workspace.active {
            // No hover can be current while interaction is gated against the
            // published image or navigation is changing its input basis.
            self.hover_object = None;
            self.hover_picked_at = None;
            self.hover_retry_at = None;
        }
        if self.controller.solid_boolean_two_hidden_sources().is_none() {
            self.paint_osnap(&painter, id, &view, viewport);
        }
        if let Some((body, _mode)) = self.controller.solid_boolean_two_preview() {
            for &[a, b, _] in &body.display.segments {
                if let (Some(&a), Some(&b)) = (
                    body.display.points.get(a as usize),
                    body.display.points.get(b as usize),
                ) {
                    paint_line(
                        &painter,
                        &view.camera,
                        viewport,
                        a,
                        b,
                        [0, 200, 130, 255],
                        1.5,
                    );
                }
            }
        }
        if let Some((body, placement)) = self.controller.solid_fillet_preview() {
            for &[a, b, _] in &body.display.segments {
                if let (Some(&a), Some(&b)) = (
                    body.display.points.get(a as usize),
                    body.display.points.get(b as usize),
                ) && let (Ok(a), Ok(b)) = (placement.point(a), placement.point(b))
                {
                    paint_line(
                        &painter,
                        &view.camera,
                        viewport,
                        a,
                        b,
                        [0, 200, 130, 255],
                        1.5,
                    );
                }
            }
            for (_, world, radius, active) in self.controller.solid_fillet_handles() {
                if let Ok(Some(point)) = view.camera.project(world, viewport) {
                    let center = egui::Pos2::new(point.screen[0] as f32, point.screen[1] as f32);
                    if rect.contains(center) {
                        painter.circle_filled(
                            center,
                            5.0,
                            if active {
                                Color32::YELLOW
                            } else {
                                Color32::LIGHT_GREEN
                            },
                        );
                        if self.controller.solid_fillet_show_radius() {
                            painter.text(
                                center + Vec2::new(7.0, -7.0),
                                egui::Align2::LEFT_BOTTOM,
                                format!("{radius}"),
                                egui::FontId::proportional(11.0),
                                Color32::LIGHT_GREEN,
                            );
                        }
                    }
                }
            }
        }
        self.gem_preview_ui(ui, &view, viewport, rect);
        self.gem_cluster_preview_ui(ui, &view, viewport, rect);
        self.gem_pave_preview_ui(ui, &view, viewport, rect);
        self.gem_cutters_preview_ui(ui, &view, viewport, rect);
        self.row_cutters_preview_ui(ui, &view, viewport, rect);
        self.gem_map_overlay_ui(ui, &view, viewport, rect);
        // Annotation text is derived from the typed record, never parsed from line geometry.
        for entity in self.controller.document().entities.values() {
            if !entity.extensions.contains_key(om9_core::annotation::SLOT) {
                continue;
            }
            if !self
                .controller
                .document()
                .permissions(entity.id, false)
                .is_ok_and(|p| p.visible)
            {
                continue;
            }
            if let Ok(record) = om9_core::annotation::Annotation::read(entity)
                .and_then(|r| r.placed(entity.placement))
                && let Ok(layout) = record.layout(self.controller.coordinator().limits())
                && let Ok(Some(p)) = view.camera.project(record.text_point, viewport)
            {
                let top = std::array::from_fn(|i| {
                    record.text_point[i] + record.y[i] * record.style.height_mm
                });
                let height = view
                    .camera
                    .project(top, viewport)
                    .ok()
                    .flatten()
                    .map(|q| {
                        ((q.screen[0] - p.screen[0]).powi(2) + (q.screen[1] - p.screen[1]).powi(2))
                            .sqrt() as f32
                    })
                    .unwrap_or(13.)
                    .clamp(7., 96.);
                painter.text(
                    egui::pos2(p.screen[0] as f32, p.screen[1] as f32),
                    egui::Align2::CENTER_CENTER,
                    layout.text,
                    egui::FontId::proportional(height),
                    Color32::from_rgb(225, 235, 240),
                );
            }
        }
        if !self.viewport_menu_owns_frame && response.hovered() && let Some(pos) = response.hover_pos() {
            let previous_hover = (self.hover, self.hover_view);
            let active = self.controller.workspace.active;
            if self.controller.top11_transform_is_pending() {
                self.controller.workspace.active = id;
            }
            if self.controller.solid_cube_needs_corner() {
                self.hover = None;
                self.snap = None;
            }
            let world = view
                .camera
                .ray([pos.x as f64, pos.y as f64], viewport)
                .and_then(|ray| {
                    if let Some(point) = self.controller.box_height_ray_point(&ray) {
                        Ok(Some(point))
                    } else if self.controller.solid_cube_needs_corner() {
                        ray.intersect_plane(view.cplane)
                    } else {
                        ray.intersect_plane(
                            self.controller
                                .slab_offset_input_plane()
                                .or_else(|| self.controller.solid_cone_input_plane())
                                .unwrap_or_else(|| self.controller.input_plane()),
                        )
                    }
                });
            if let Ok(Some(world)) = world {
                let world = self.controller.constrain_point(world).unwrap_or(world);
                let mut snap = None;
                let mut unsupported_surface = false;
                if self.controller.is_running()
                    && let Some(scene) = self.controller.scene()
                {
                    let from = self.controller.relative_base().map(|point| point.xyz());
                    let settings = self.snap_state.settings;
                    let candidates = om9_scene::osnap::candidates(
                        scene,
                        &view.camera,
                        viewport,
                        [pos.x as f64, pos.y as f64],
                        settings,
                        from,
                    ).and_then(|mut candidates| {
                        if let Some(one_shot) = settings.one_shot {
                            candidates.retain(|candidate| candidate.kind == one_shot);
                        }
                        self.resolve_snap_candidates(candidates, &view.camera, viewport, view.mode, &[])
                    });
                    match candidates {
                      Ok(best) => {
                        if let Some(candidate) = best {
                            snap = Some(candidate);
                        } else if self.snap_state.settings.one_shot == Some(om9_scene::osnap::SnapKind::Surface) {
                            unsupported_surface = true;
                            self.status.info("Surface snap unavailable: no surface candidate under the pointer");
                        }
                      }
                      Err(error) => {
                          unsupported_surface = true;
                          let label = if settings.one_shot == Some(om9_scene::osnap::SnapKind::Surface) {
                              "Surface snap unavailable"
                          } else {
                              "Snap unavailable"
                          };
                          self.status.info(&format!("{label}: {error}"));
                      }
                    }
                }
                if self.smarttrack_enabled
                    && !unsupported_surface
                    && snap.is_none()
                    && !self.smarttrack.anchors().is_empty()
                {
                    let tolerance = world_per_pixel(&view.camera, viewport)
                        * self.snap_state.settings.radius_px;
                    snap = self.smarttrack.infer(world, tolerance);
                }
                if unsupported_surface {
                    self.hover = None;
                    self.snap = None;
                } else if let Some(candidate) = snap {
                    self.hover = Some(candidate.world);
                    self.snap = Some(candidate);
                } else {
                    self.hover = Some(world);
                    self.snap = None;
                }
                self.hover_view = Some(id);
            } else if self.controller.top11_transform_is_pending() {
                self.hover = None;
                self.hover_view = None;
                self.snap = None;
            }
            self.controller.workspace.active = active;
            if self.controller.is_running() && (self.hover, self.hover_view) != previous_hover {
                // Other viewports may already have painted this frame.
                ui.ctx().request_repaint();
            }
        }
        if let Some(point) = self.hover {
            if let Ok(Some((record, layout))) = self.controller.annotation_preview(point) {
                for [a, b] in layout.lines {
                    paint_line(
                        &painter,
                        &view.camera,
                        viewport,
                        a,
                        b,
                        [0, 140, 100, 255],
                        1.5,
                    );
                }
                if let Ok(Some(p)) = view.camera.project(record.text_point, viewport) {
                    let top = std::array::from_fn(|i| {
                        record.text_point[i] + record.y[i] * record.style.height_mm
                    });
                    let height = view
                        .camera
                        .project(top, viewport)
                        .ok()
                        .flatten()
                        .map(|q| {
                            ((q.screen[0] - p.screen[0]).powi(2)
                                + (q.screen[1] - p.screen[1]).powi(2))
                            .sqrt() as f32
                        })
                        .unwrap_or(13.)
                        .clamp(7., 96.);
                    painter.text(
                        egui::pos2(p.screen[0] as f32, p.screen[1] as f32),
                        egui::Align2::CENTER_CENTER,
                        layout.text,
                        egui::FontId::proportional(height),
                        Color32::from_rgb(130, 240, 175),
                    );
                }
            }
            if let Ok(Some((label, wire))) = self.controller.measure_preview(point) {
                for [a, b] in wire {
                    paint_line(
                        &painter,
                        &view.camera,
                        viewport,
                        a,
                        b,
                        [0, 140, 100, 255],
                        1.5,
                    );
                }
                if let Ok(Some(p)) = view.camera.project(point, viewport) {
                    painter.text(
                        egui::pos2(p.screen[0] as f32 + 12., p.screen[1] as f32 - 15.),
                        egui::Align2::LEFT_BOTTOM,
                        &label,
                        egui::FontId::proportional(13.),
                        Color32::from_rgb(130, 240, 175),
                    );
                }
            }
            let active = self.controller.workspace.active;
            if self.controller.top11_transform_is_pending()
                && let Some(hover_view) = self.hover_view
            {
                self.controller.workspace.active = hover_view;
            }
            let transform_preview = self
                .controller
                .transform_preview(self.viewport_command_point(point), self.transcript.draft());
            self.controller.workspace.active = active;
            if let Ok(wire) = transform_preview {
                for [a, b] in wire {
                    if a == b
                        && let Ok(Some(p)) = view.camera.project(a, viewport)
                    {
                        painter.circle_filled(
                            egui::pos2(p.screen[0] as f32, p.screen[1] as f32),
                            3.,
                            Color32::from_rgb(0, 200, 130),
                        );
                    }
                    paint_line(
                        &painter,
                        &view.camera,
                        viewport,
                        a,
                        b,
                        [0, 200, 130, 255],
                        1.5,
                    );
                }
            }
            if let Ok(Some((a, b))) = self.controller.preview(point) {
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
            for [a, b] in self.controller.solid_preview(point) {
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
            for marker in self.controller.solid_preview_markers() {
                if let Ok(Some(projected)) = view.camera.project(marker, viewport) {
                    painter.circle_filled(
                        egui::pos2(projected.screen[0] as f32, projected.screen[1] as f32),
                        3.5,
                        Color32::from_rgb(0, 200, 120),
                    );
                }
            }
            for [a, b] in self.controller.circle_preview(point) {
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
            for [a, b] in self.controller.rect_preview(point) {
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
            for [a, b] in self.controller.polygon_preview(point) {
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
            for [a, b] in self.controller.helix_preview(point) {
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
            for [a, b] in self.controller.spiral_preview(point) {
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
            for [a, b] in self.controller.polyline_preview(point) {
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
            for [a, b] in self.controller.interp_preview(point) {
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
            for [a, b] in self.controller.ellipse_preview(point) {
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
            for [a, b] in self.controller.arc_preview(point) {
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
            if response.clicked() && !published_scene_current {
                self.status.info("Viewport is updating; try selecting again");
            } else if response.clicked() {
                self.controller.workspace.active = id;
                if self.select_control_grip_at(
                    &view,
                    viewport,
                    [pos.x as f64, pos.y as f64],
                    ui.input(|input| input.modifiers.shift),
                ) {
                    self.focus_cmd = false;
                    return;
                }
                if self.controller.is_running() {
                    if self.controller.solid_boolean_two_preview_mode().is_some() {
                        let result = self.controller.submit("Next");
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                    } else if self.controller.solid_fillet_adding_handle() {
                        let mut handle_hits: Vec<_> =
                            if self.controller.solid_fillet_copying_to_target() {
                                self.controller
                                    .solid_fillet_handles()
                                    .into_iter()
                                    .filter_map(|(index, world, _, _)| {
                                        let projected =
                                            view.camera.project(world, viewport).ok().flatten()?;
                                        let distance = pos.distance(egui::Pos2::new(
                                            projected.screen[0] as f32,
                                            projected.screen[1] as f32,
                                        ));
                                        (distance <= 8.0).then_some((index, distance))
                                    })
                                    .collect()
                            } else {
                                Vec::new()
                            };
                        handle_hits.sort_by(|a, b| a.1.total_cmp(&b.1));
                        if !handle_hits.is_empty() {
                            let result = self.controller.solid_fillet_copy_to_hit(&handle_hits);
                            if result.is_ok() {
                                self.append_running_prompt();
                            }
                            self.feedback(result);
                            self.focus_cmd = false;
                            return;
                        }
                        let candidates = self.controller.scene().and_then(|scene| {
                            scene.pick_kind_all([pos.x as f64, pos.y as f64], &view.camera, viewport,
                                pick_tolerance(ui), PickIntent::Select, om9_scene::scene::PickKind::Edge)
                                .ok().map(|hits| hits.into_iter().filter(|hit| {
                                    hit.revalidate_in_view(self.controller.coordinator(), &view.camera, viewport).is_ok() &&
                                    self.controller.subselection().contains(&crate::session::SelectedSubobject::from_hit(hit))
                                }).collect::<Vec<_>>())
                        }).unwrap_or_default();
                        let result = match candidates.as_slice() {
                            [hit] => self.controller.solid_fillet_add_handle(
                                crate::session::SelectedSubobject::from_hit(hit),
                                hit.world_point,
                            ),
                            [] => Err(Error::Missing("selected fillet edge under the pointer")),
                            _ => Err(Error::Invalid("ambiguous fillet edges")),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_fillet_preview().is_some() {
                        let mut hits: Vec<_> = self
                            .controller
                            .solid_fillet_handles()
                            .into_iter()
                            .filter_map(|(index, world, _, _)| {
                                let projected =
                                    view.camera.project(world, viewport).ok().flatten()?;
                                let distance = pos.distance(egui::Pos2::new(
                                    projected.screen[0] as f32,
                                    projected.screen[1] as f32,
                                ));
                                (distance <= 8.0).then_some((index, distance))
                            })
                            .collect();
                        hits.sort_by(|a, b| a.1.total_cmp(&b.1));
                        let result = self.controller.solid_fillet_select_hit(&hits);
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = true;
                    } else if self.controller.solid_slab_needs_curve() {
                        let candidates = self
                            .controller
                            .scene()
                            .and_then(|scene| {
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
                                                hit.path.is_empty()
                                                    && hit
                                                        .revalidate_in_view(
                                                            self.controller.coordinator(),
                                                            &view.camera,
                                                            viewport,
                                                        )
                                                        .is_ok()
                                                    && matches!(
                                                        self.controller
                                                            .document()
                                                            .entities
                                                            .get(&hit.root)
                                                            .map(|e| &e.geometry),
                                                        Some(om9_core::geometry::Geometry::Curve(
                                                            _
                                                        ))
                                                    )
                                            })
                                            .collect::<Vec<_>>()
                                    })
                            })
                            .unwrap_or_default();
                        if candidates.len() > 1 {
                            self.ambiguous = candidates.iter().map(|hit| hit.root).collect();
                            self.ambiguous_pos = Some(pos);
                            self.ambiguous_epoch = self.controller.coordinator().epoch();
                            self.ambiguous_pipe_context = None;
                            self.ambiguous_difference_context = None;
                            self.ambiguous_extract_context = None;
                            self.ambiguous_sphere_context = None;
                            self.ambiguous_slab_context = self
                                .controller
                                .solid_slab_pick_context()
                                .map(|run| (run, id, candidates));
                            self.focus_cmd = true;
                            return;
                        }
                        let result = match candidates.as_slice() {
                            [hit] => self.controller.solid_slab_pick_curve(hit.root),
                            [] => Err(Error::Missing("curve under the pointer")),
                            _ => Err(Error::Invalid("ambiguous Slab curves")),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_sphere_brep_edge_stage() == 3 {
                        let candidates = self
                            .controller
                            .scene()
                            .and_then(|scene| {
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
                                                    && self
                                                        .controller
                                                        .solid_sphere_is_source_curve(hit)
                                            })
                                            .collect::<Vec<_>>()
                                    })
                            })
                            .unwrap_or_default();
                        let result = match candidates.as_slice() {
                            [hit] => self.controller.accept_world_point(hit.world_point),
                            [] => Err(Error::Missing("selected curve under the pointer")),
                            _ => Err(Error::Invalid("ambiguous selected curve")),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_sphere_brep_edge_stage() != 0 {
                        let candidates = self.controller.scene().and_then(|scene| {
                            scene.pick_kind_all([pos.x as f64, pos.y as f64], &view.camera, viewport,
                                pick_tolerance(ui), PickIntent::Select, om9_scene::scene::PickKind::Edge)
                                .ok().map(|hits| hits.into_iter().filter(|hit|
                                    matches!(hit.subset, om9_scene::scene::Subelement::BrepEdge(_)) &&
                                    hit.revalidate_in_view(self.controller.coordinator(), &view.camera, viewport).is_ok() &&
                                    (self.controller.solid_sphere_brep_edge_stage() == 1 ||
                                     self.controller.solid_sphere_is_source_edge(&crate::session::SelectedSubobject::from_hit(hit)))
                                ).collect::<Vec<_>>())
                        }).unwrap_or_default();
                        if candidates.len() > 1
                            && self.controller.solid_sphere_brep_edge_stage() == 1
                        {
                            self.ambiguous = candidates.iter().map(|hit| hit.root).collect();
                            self.ambiguous_pos = Some(pos);
                            self.ambiguous_epoch = self.controller.coordinator().epoch();
                            self.ambiguous_difference_context = None;
                            self.ambiguous_pipe_context = None;
                            self.ambiguous_extract_context = None;
                            self.ambiguous_sphere_context = self
                                .controller
                                .solid_sphere_edge_pick_context()
                                .map(|run| (run, id, candidates));
                            self.focus_cmd = true;
                            return;
                        }
                        let result = match candidates.as_slice() {
                            [hit] => self.controller.solid_sphere_pick_brep_edge(
                                crate::session::SelectedSubobject::from_hit(hit),
                                hit.world_point,
                            ),
                            [] if self.controller.solid_sphere_brep_edge_stage() == 1 => {
                                let curves = self.controller.scene().and_then(|scene| {
                                    scene.pick_all([pos.x as f64, pos.y as f64], &view.camera, viewport,
                                        pick_tolerance(ui), PickIntent::Select).ok().map(|hits| hits.into_iter().filter(|hit|
                                            hit.path.is_empty() && hit.revalidate_in_view(self.controller.coordinator(), &view.camera, viewport).is_ok() &&
                                            matches!(self.controller.document().entities.get(&hit.root).map(|e| &e.geometry), Some(om9_core::geometry::Geometry::Curve(_)))
                                        ).map(|hit| hit.root).fold(Vec::new(), |mut ids, id| { if !ids.contains(&id) { ids.push(id); } ids }))
                                }).unwrap_or_default();
                                match curves.as_slice() {
                                    [id] => self.controller.solid_sphere_pick_curve(*id),
                                    [] => {
                                        Err(Error::Missing("curve or B-Rep edge under the pointer"))
                                    }
                                    _ => Err(Error::Invalid(
                                        "ambiguous curves: isolate the intended object",
                                    )),
                                }
                            }
                            [] => Err(Error::Missing("B-Rep edge under the pointer")),
                            _ => Err(Error::Invalid(
                                "ambiguous edges: isolate the intended object",
                            )),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_fillet_needs_edge() {
                        let candidates = self
                            .controller
                            .scene()
                            .and_then(|scene| {
                                scene
                                    .pick_kind_all(
                                        [pos.x as f64, pos.y as f64],
                                        &view.camera,
                                        viewport,
                                        pick_tolerance(ui),
                                        PickIntent::Select,
                                        om9_scene::scene::PickKind::Edge,
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
                                            .collect::<Vec<_>>()
                                    })
                            })
                            .unwrap_or_default();
                        let result = match candidates.as_slice() {
                            [hit] => self.controller.solid_fillet_pick_edge(
                                crate::session::SelectedSubobject::from_hit(hit),
                            ),
                            [] => Err(Error::Missing("B-Rep edge under the pointer")),
                            _ => Err(Error::Invalid(
                                "ambiguous edges: isolate the intended object",
                            )),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_face_needs_pick() {
                        let remove = ui.input(|input| input.modifiers.ctrl);
                        let candidates = self
                            .controller
                            .scene()
                            .and_then(|scene| {
                                scene
                                    .pick_kind_all(
                                        [pos.x as f64, pos.y as f64],
                                        &view.camera,
                                        viewport,
                                        pick_tolerance(ui),
                                        PickIntent::Select,
                                        om9_scene::scene::PickKind::Face,
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
                                            .collect::<Vec<_>>()
                                    })
                            })
                            .unwrap_or_default();
                        if candidates.len() > 1 {
                            self.ambiguous = candidates.iter().map(|hit| hit.root).collect();
                            self.ambiguous_pos = Some(pos);
                            self.ambiguous_epoch = self.controller.coordinator().epoch();
                            self.ambiguous_difference_context = None;
                            self.ambiguous_pipe_context = None;
                            self.ambiguous_extract_context = self
                                .controller
                                .solid_extract_pick_context()
                                .map(|run| (run, id, remove, candidates));
                            self.focus_cmd = true;
                            return;
                        }
                        self.ambiguous.clear();
                        self.ambiguous_pos = None;
                        self.ambiguous_extract_context = None;
                        self.ambiguous_sphere_context = None;
                        self.ambiguous_slab_context = None;
                        let result = match candidates.first() {
                            Some(hit) => {
                                let reference = crate::session::SelectedSubobject::from_hit(hit);
                                if remove {
                                    self.controller.solid_extract_remove_face(reference)
                                } else {
                                    self.controller.solid_extract_pick_face(reference)
                                }
                            }
                            None => Err(Error::Missing("B-Rep face under the pointer")),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_pipe_needs_curve() {
                        if self.controller.solid_pipe_chain_selecting() {
                            let edges = self
                                .controller
                                .scene()
                                .and_then(|scene| {
                                    scene
                                        .pick_kind_all(
                                            [pos.x as f64, pos.y as f64],
                                            &view.camera,
                                            viewport,
                                            pick_tolerance(ui),
                                            PickIntent::Select,
                                            om9_scene::scene::PickKind::Edge,
                                        )
                                        .ok()
                                        .map(|hits| {
                                            hits.into_iter()
                                                .filter(|hit| {
                                                    matches!(
                                                        hit.subset,
                                                        om9_scene::scene::Subelement::BrepEdge(_)
                                                    ) && hit
                                                        .revalidate_in_view(
                                                            self.controller.coordinator(),
                                                            &view.camera,
                                                            viewport,
                                                        )
                                                        .is_ok()
                                                })
                                                .collect::<Vec<_>>()
                                        })
                                })
                                .unwrap_or_default();
                            if !edges.is_empty() {
                                let result = match edges.as_slice() {
                                    [hit] => self.controller.solid_pipe_pick_edge(
                                        crate::session::SelectedSubobject::from_hit(hit),
                                    ),
                                    _ => Err(Error::Invalid("ambiguous B-Rep edges")),
                                };
                                if result.is_ok() {
                                    self.append_running_prompt();
                                }
                                self.feedback(result);
                                self.focus_cmd = false;
                                return;
                            }
                        }
                        let candidates = self
                            .controller
                            .scene()
                            .and_then(|scene| {
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
                                                    && matches!(
                                                        self.controller
                                                            .document()
                                                            .entities
                                                            .get(&hit.root)
                                                            .map(|e| &e.geometry),
                                                        Some(om9_core::geometry::Geometry::Curve(
                                                            _
                                                        ))
                                                    )
                                            })
                                            .map(|hit| hit.root)
                                            .fold(Vec::new(), |mut ids, id| {
                                                if !ids.contains(&id) {
                                                    ids.push(id);
                                                }
                                                ids
                                            })
                                    })
                            })
                            .unwrap_or_default();
                        if candidates.len() > 1 {
                            self.ambiguous = candidates;
                            self.ambiguous_pos = Some(pos);
                            self.ambiguous_epoch = self.controller.coordinator().epoch();
                            self.ambiguous_pipe_context = self.controller.solid_pipe_pick_context();
                            self.ambiguous_difference_context = None;
                            self.ambiguous_extract_context = None;
                            self.ambiguous_sphere_context = None;
                            self.ambiguous_slab_context = None;
                            self.focus_cmd = true;
                            return;
                        }
                        self.ambiguous.clear();
                        self.ambiguous_pos = None;
                        self.ambiguous_pipe_context = None;
                        self.ambiguous_difference_context = None;
                        self.ambiguous_extract_context = None;
                        self.ambiguous_sphere_context = None;
                        self.ambiguous_slab_context = None;
                        let result = match candidates.len() {
                            0 => Err(Error::Missing("curve under the pointer")),
                            _ => self.controller.solid_pipe_pick_curve(candidates[0]),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                    } else if self.controller.solid_boolean_needs_object() {
                        let candidates = self
                            .controller
                            .scene()
                            .and_then(|scene| {
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
                            })
                            .unwrap_or_default();
                        let ctrl = ui.input(|input| input.modifiers.ctrl);
                        if candidates.len() > 1 && !ctrl {
                            self.ambiguous = candidates;
                            self.ambiguous_pos = Some(pos);
                            self.ambiguous_epoch = self.controller.coordinator().epoch();
                            self.ambiguous_difference_context =
                                self.controller.solid_boolean_pick_context();
                            self.ambiguous_pipe_context = None;
                            self.ambiguous_extract_context = None;
                            self.ambiguous_sphere_context = None;
                            self.ambiguous_slab_context = None;
                            self.focus_cmd = true;
                            return;
                        }
                        self.ambiguous.clear();
                        self.ambiguous_pos = None;
                        self.ambiguous_difference_context = None;
                        self.ambiguous_pipe_context = None;
                        self.ambiguous_extract_context = None;
                        self.ambiguous_sphere_context = None;
                        self.ambiguous_slab_context = None;
                        let picked = candidates.first().copied();
                        let result = match picked {
                            Some(id) => self.controller.solid_boolean_pick(id, ctrl),
                            None => Err(Error::Missing("B-Rep solid under the pointer")),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                        if let Some(focus) = ui.ctx().memory(|memory| memory.focused()) {
                            ui.ctx().memory_mut(|memory| memory.surrender_focus(focus));
                        }
                    } else if self.controller.top11_needs_object() {
                        let picked = self.controller.scene().and_then(|scene| {
                            scene
                                .pick_all(
                                    [pos.x as f64, pos.y as f64],
                                    &view.camera,
                                    viewport,
                                    pick_tolerance(ui),
                                    PickIntent::Select,
                                )
                                .ok()
                                .and_then(|hits| hits.into_iter().next().map(|hit| hit.root))
                        });
                        let result = match picked {
                            Some(object) if ui.input(|input| input.modifiers.ctrl) => {
                                self.controller.top11_remove_object(object)
                            }
                            Some(object) => self.controller.top11_accept_object(object),
                            None => Err(Error::Missing("cutting curve under the pointer")),
                        };
                        match result {
                            Ok(result) => {
                                self.status.info(&result.message());
                            }
                            Err(error) => {
                                self.feedback(Err(error));
                            }
                        }
                        self.focus_cmd = false;
                        if let Some(focus) = ui.ctx().memory(|memory| memory.focused()) {
                            ui.ctx().memory_mut(|memory| memory.surrender_focus(focus));
                        }
                    } else if let Some(target) = self.controller.top11_split_iso_target() {
                        let hit = self.controller.scene().and_then(|scene| {
                            scene
                                .pick_all(
                                    [pos.x as f64, pos.y as f64],
                                    &view.camera,
                                    viewport,
                                    pick_tolerance(ui),
                                    PickIntent::Select,
                                )
                                .ok()
                                .and_then(|hits| hits.into_iter().find(|hit| hit.root == target))
                        });
                        let result = match hit {
                            Some(hit) => view
                                .camera
                                .ray([pos.x as f64, pos.y as f64], viewport)
                                .and_then(|ray| {
                                    self.controller.top11_split_iso_accept_viewport_pick(
                                        hit.root,
                                        hit.world_point,
                                        ray,
                                    )
                                }),
                            None => Err(Error::Missing("split surface under the pointer")),
                        };
                        if result.is_ok() {
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        self.focus_cmd = false;
                        if let Some(focus) = ui.ctx().memory(|memory| memory.focused()) {
                            ui.ctx().memory_mut(|memory| memory.surrender_focus(focus));
                        }
                    } else if self.controller.top11_trim_many_ready() {
                        let hit = self.controller.scene().and_then(|scene| {
                            scene
                                .pick_all(
                                    [pos.x as f64, pos.y as f64],
                                    &view.camera,
                                    viewport,
                                    pick_tolerance(ui),
                                    PickIntent::Select,
                                )
                                .ok()
                                .and_then(|hits| {
                                    let mut hits = hits.into_iter().filter(|hit| {
                                        self.controller.top11_trim_many_has_target(hit.root)
                                    });
                                    let first = hits.next()?;
                                    let first_is_face = matches!(
                                        self.controller
                                            .document()
                                            .entities
                                            .get(&first.root)
                                            .map(|e| &e.geometry),
                                        Some(
                                            om9_core::geometry::Geometry::Brep(_)
                                                | om9_core::geometry::Geometry::Surface(_)
                                        )
                                    );
                                    if first_is_face {
                                        let tolerance = self
                                            .controller
                                            .document()
                                            .effective_tolerance()
                                            .ok()
                                            .map(|t| t.absolute_mm)
                                            .unwrap_or(0.0);
                                        if let Some(curve) = hits.find(|hit| {
                                            (hit.depth - first.depth).abs() <= tolerance
                                                && matches!(
                                                    self.controller
                                                        .document()
                                                        .entities
                                                        .get(&hit.root)
                                                        .map(|e| &e.geometry),
                                                    Some(om9_core::geometry::Geometry::Curve(_))
                                                )
                                        }) {
                                            return Some(curve);
                                        }
                                    }
                                    Some(first)
                                })
                        });
                        match hit {
                            Some(hit) => match view
                                .camera
                                .ray([pos.x as f64, pos.y as f64], viewport)
                                .and_then(|ray| {
                                    self.controller.top11_trim_many_accept_viewport_pick(
                                        hit.root,
                                        hit.world_point,
                                        ray,
                                    )
                                }) {
                                Ok(result) => {
                                    self.status.info(&result.message());
                                    self.append_running_prompt();
                                }
                                Err(error) => self.feedback(Err(error)),
                            },
                            None => self.feedback(Err(Error::Missing(
                                "remaining trim target under the pointer",
                            ))),
                        }
                        self.focus_cmd = false;
                        if let Some(focus) = ui.ctx().memory(|memory| memory.focused()) {
                            ui.ctx().memory_mut(|memory| memory.surrender_focus(focus));
                        }
                    } else if let Some(point) = self.hover {
                        let anchor = self.snap.clone();
                        let point = self.viewport_command_point(point);
                        // Matrix Rectangle: Shift while picking a corner makes it square.
                        if self.controller.rect_running() {
                            let shift = ui.input(|input| input.modifiers.shift);
                            let _ = self.controller.set_rect_square(shift);
                        }
                        let result = self.controller.accept_world_point(point);
                        if result.is_ok() {
                            self.snap_state.accept();
                            if let Some(candidate) = anchor {
                                let plane = self.controller.input_plane();
                                let _ = self.smarttrack.push(&candidate, plane);
                            }
                            self.append_running_prompt();
                        }
                        self.feedback(result);
                        // After a viewport pick the keyboard belongs to the viewport, so the
                        // End/Mid one-shot shortcuts can be typed without entering command text.
                        self.focus_cmd = false;
                        if let Some(focus) = ui.ctx().memory(|memory| memory.focused()) {
                            ui.ctx().memory_mut(|memory| memory.surrender_focus(focus));
                        }
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
                        self.ambiguous_difference_context = None;
                        self.ambiguous_pipe_context = None;
                        self.ambiguous_extract_context = None;
                        self.ambiguous_sphere_context = None;
                        self.ambiguous_slab_context = None;
                        self.focus_cmd = true;
                        return;
                    }
                    self.ambiguous.clear();
                    self.ambiguous_pos = None;
                    // `pick_all` already returned nearest-first, revalidated roots above.
                    // Reuse it instead of intersecting the same mesh a second time.
                    let picked = candidates.as_ref().and_then(|list| list.first().copied());
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
    /// The Object Info label beside a single selected object (Matrix8 book 1,
    /// pages 32-34: a quick pop up label beside a selected object, drawn in the
    /// layer colour, showing the type, the size and the validity).
    fn object_info_label(
        &mut self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        use om9_scene::scene::RenderBuffers;
        if self.controller.selected().len() != 1 {
            return;
        }
        let id = *self
            .controller
            .selected()
            .iter()
            .next()
            .expect("one selected");
        let Some(document) = self.controller.coordinator().world().documents
            .get(&self.controller.document().id).cloned() else {
            return;
        };
        let epoch = self.controller.coordinator().epoch();
        let limits = self.controller.coordinator().limits();
        let info = if document.entities.get(&id).is_some_and(|entity| {
            matches!(entity.geometry, om9_core::geometry::Geometry::Brep(_))
        }) {
            self.object_info_async.get(document, epoch, id, limits, ui.ctx())
        } else {
            self.object_info_cache.borrow_mut().get(&document, epoch, id, limits).ok()
        };
        let Some(info) = info else {
            return;
        };
        let text = om9_core::object_info::label_text(&info);
        let Some(scene) = self.controller.scene() else {
            return;
        };
        let Some(object) = scene.objects.iter().find(|entry| entry.root == id) else {
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
        let mut sum = [0.0_f64; 3];
        let mut count = 0.0_f64;
        for point in points.iter().take(256) {
            if let Ok(world) = transform.point(*point) {
                for axis in 0..3 {
                    sum[axis] += world[axis];
                }
                count += 1.0;
            }
        }
        if count == 0.0 {
            return;
        }
        let centre = [sum[0] / count, sum[1] / count, sum[2] / count];
        let Ok(Some(projected)) = view.camera.project(centre, viewport) else {
            return;
        };
        let pos = egui::pos2(
            projected.screen[0] as f32 + 10.0,
            projected.screen[1] as f32 - 10.0,
        );
        let color = info
            .color
            .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
            .unwrap_or(Color32::WHITE);
        ui.painter().with_clip_rect(rect).text(
            pos,
            egui::Align2::LEFT_BOTTOM,
            &text,
            egui::FontId::proportional(9.0),
            color,
        );
    }

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
        if self.gumball_session.is_some() || self.grip_drag.is_some() {
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

    /// Builder viewport handles (`OM9-L3-03-11-005`/`OM9-L3-04-02-009`): one square per
    /// handle at its preview position. Dragging along the handle axis edits the same field
    /// the panel edits, so a drag and a typed value can never disagree.
    fn builder_handles_ui(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        let handles = self.controller.builder_handles();
        if handles.is_empty() {
            self.builder_handle_drag = None;
            return;
        }
        let painter = ui.painter().with_clip_rect(rect);
        let pointer = ui.input(|input| input.pointer.hover_pos());
        let mut projected = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Ok(Some(point)) = view.camera.project(handle.position, viewport) {
                projected.push((
                    handle,
                    egui::pos2(point.screen[0] as f32, point.screen[1] as f32),
                ));
            }
        }
        let nearest = pointer.and_then(|pointer| {
            projected
                .iter()
                .map(|(handle, position)| (handle.field, *position, pointer.distance(*position)))
                .filter(|(_, _, distance)| *distance <= 11.0)
                .min_by(|a, b| a.2.total_cmp(&b.2))
        });
        if response.drag_started() {
            self.builder_handle_drag = nearest.map(|(field, _, _)| field);
        }
        let mut adjusted: Option<Result<()>> = None;
        if let Some(field) = self.builder_handle_drag {
            if response.dragged()
                && let Some((handle, position)) =
                    projected.iter().find(|(handle, _)| handle.field == field)
                && let Ok(Some(plus)) = view.camera.project(
                    [
                        handle.position[0] + handle.axis[0],
                        handle.position[1] + handle.axis[1],
                        handle.position[2] + handle.axis[2],
                    ],
                    viewport,
                )
            {
                let axis = egui::vec2(
                    plus.screen[0] as f32 - position.x,
                    plus.screen[1] as f32 - position.y,
                );
                let length = axis.length();
                if length > 0.5 {
                    let delta = ui.input(|input| input.pointer.delta());
                    let millimetres = (delta.x * axis.x + delta.y * axis.y) as f64 / length as f64;
                    let millimetres = millimetres * handle.scale;
                    if millimetres.abs() > 0.0 {
                        adjusted = Some(self.controller.builder_adjust_field(field, millimetres));
                    }
                }
            }
            if response.drag_stopped() {
                self.builder_handle_drag = None;
            }
        }
        if let Some(result) = adjusted {
            self.feedback(result);
        }
        for (handle, position) in &projected {
            let active = self.builder_handle_drag == Some(handle.field);
            let colour = if active {
                Color32::from_rgb(255, 214, 102)
            } else {
                Color32::from_rgb(45, 212, 191)
            };
            let square = Rect::from_center_size(*position, Vec2::splat(9.0));
            painter.rect_filled(square, 1.0, Color32::from_black_alpha(120));
            painter.rect_stroke(
                square,
                1.0,
                Stroke::new(1.5, colour),
                egui::StrokeKind::Outside,
            );
            painter.text(
                *position + Vec2::new(8.0, -8.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{} {:.2}", handle.label, handle.value),
                egui::FontId::proportional(11.0),
                colour,
            );
        }
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

        if self.grip_drag.is_some() {
            return;
        }
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
        // egui can receive the press and initial motion in one frame and the
        // release in the next without emitting drag_started/drag_stopped. Use
        // only the press bound to the still-published image in this viewport.
        let short_drag = if self.gumball_session.is_none() && !self.controller.is_running()
            && self.controller.workspace.active == view.id
            && !self.viewport_menu_owns_frame && !egui::Popup::is_any_open(ui.ctx())
            && !response.drag_stopped_by(egui::PointerButton::Primary)
            && ui.input(|input| input.focused && input.pointer.button_released(egui::PointerButton::Primary)) {
            self.primary_press_scene.as_ref().and_then(|(id, _, camera, start)| {
                let end = ui.input(|input| input.pointer.latest_pos())?;
                let end = [f64::from(end.x), f64::from(end.y)];
                (*id == view.id && camera == &view.camera && (end[0] - start[0]).hypot(end[1] - start[1]) > 8.)
                    .then_some((*start, end))
            })
        } else { None };
        let Some(pointer) = response.interact_pointer_pos().or_else(|| short_drag.map(|(_, end)| egui::pos2(end[0] as f32, end[1] as f32))) else {
            return;
        };
        let pointer = [f64::from(pointer.x), f64::from(pointer.y)];
        if response.drag_started_by(egui::PointerButton::Primary) || short_drag.is_some() {
            let start = short_drag.map(|(start, _)| start).unwrap_or_else(|| ui
                .input(|input| input.pointer.press_origin())
                .map(|p| [f64::from(p.x), f64::from(p.y)])
                .unwrap_or(pointer));
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
            if let Some(log) = self.interaction_log.as_mut() {
                log.record(serde_json::json!({"event":"gumball_hit", "frame_id":self.nav_frame_token,
                    "view":view.id.title(), "start":start, "handle":hit.map(|handle|format!("{handle:?}")),
                    "camera_revision":view.camera.revision}));
            }
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
                || response.drag_stopped_by(egui::PointerButton::Primary) || short_drag.is_some())
        {
            let snapped = match self.gumball_snap_candidate(&view.camera, viewport, pointer) {
                Ok(candidate) => candidate,
                Err(error) => {
                    self.snap = None;
                    self.cancel_gumball();
                    self.feedback(Err(error));
                    return;
                }
            };
            if (response.drag_stopped_by(egui::PointerButton::Primary) || short_drag.is_some())
                && self.controller.gumball_snappy()
                && self.snap_state.settings.one_shot == Some(om9_scene::osnap::SnapKind::Surface)
                && snapped.is_none()
            {
                self.snap = None;
                self.cancel_gumball();
                self.feedback(Err(Error::Missing("surface snap candidate")));
                return;
            }
            let result =
                self.update_gumball(&view.camera, viewport, pointer, snapped, modifiers.alt);
            if let Err(error) = result {
                self.cancel_gumball();
                self.feedback(Err(error));
                return;
            }
            let action = match self.gumball_handle {
                Some(Handle::Rotate(_)) => "rotate",
                Some(Handle::Scale(_)) => "scale",
                Some(Handle::MoveAxis(_)) => "move",
                _ => "move",
            };
            let frame_ms = crate::interaction_log::frame_ms(ui.input(|input| input.stable_dt));
            let selected = self.controller.selected().len();
            if let Some(log) = self.interaction_log.as_mut() {
                log.record(serde_json::json!({
                    "event": "gumball",
                    "view": view.id.title(),
                    "action": action,
                    "frame_ms": frame_ms,
                    "selected": selected,
                }));
            }
            ui.ctx().request_repaint();
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) || short_drag.is_some() {
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
                if let Some(log) = self.interaction_log.as_mut() {
                    log.record(serde_json::json!({"event":"gumball_commit", "frame_id":self.nav_frame_token,
                        "view":view.id.title(), "success":result.is_ok(), "error":result.as_ref().err().map(ToString::to_string)}));
                }
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

    #[cfg(test)]
    fn refine_snap_candidate(
        &mut self,
        candidate: om9_scene::osnap::SnapCandidate,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<om9_scene::osnap::SnapCandidate> {
        self.try_refine_snap_candidate(candidate, camera, viewport)?
            .ok_or(Error::Missing("exact surface snap intersection"))
    }

    fn try_refine_snap_candidate(
        &mut self,
        candidate: om9_scene::osnap::SnapCandidate,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<Option<om9_scene::osnap::SnapCandidate>> {
        if candidate.kind != om9_scene::osnap::SnapKind::Surface {
            return Ok(Some(candidate));
        }
        self.controller.validate_surface_candidate(&candidate, camera, viewport)?;
        let document = std::sync::Arc::downgrade(&self.controller.scene().ok_or(Error::Missing("surface snap scene"))?.document);
        if let Some((_, _, output)) = self.surface_snap_cache.iter().find(|(source, input, _)| source.ptr_eq(&document) && input == &candidate) {
            return Ok(Some(output.clone()));
        }
        let Some(refined) = self.controller.try_refine_surface_snap(&candidate, camera, viewport)? else { return Ok(None) };
        if self.surface_snap_cache.len() >= 64 {
            self.surface_snap_cache.clear();
        }
        self.surface_snap_cache.push((document, candidate, refined.clone()));
        Ok(Some(refined))
    }

    fn resolve_snap_candidates(
        &mut self,
        candidates: Vec<om9_scene::osnap::SnapCandidate>,
        camera: &Camera,
        viewport: Viewport,
        mode: om9_scene::workspace::DisplayMode,
        excluded: &[om9_core::identity::ObjectId],
    ) -> Result<Option<om9_scene::osnap::SnapCandidate>> {
        let Some(first) = om9_scene::osnap::choose(&candidates) else { return Ok(None) };
        if first.kind != om9_scene::osnap::SnapKind::Surface {
            return Ok(Some(first));
        }
        let mut candidates = candidates;
        candidates.retain(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface);
        let order = surface_group_order(&candidates)?;
        let mut refined = Vec::new();
        refined.try_reserve_exact(candidates.len()).map_err(|_| Error::Budget("surface snap results"))?;
        let mut index = 0;
        while index < order.len() {
            let mut end = index + 1;
            while end < order.len() && end - index < 8
                && Controller::can_group_surface_candidates(&candidates[order[index]], &candidates[order[end]]) {
                end += 1;
            }
            let mut group = [&candidates[order[index]]; 8];
            for (slot, &position) in group.iter_mut().zip(&order[index..end]) { *slot = &candidates[position]; }
            let group = &group[..end-index];
            let document = std::sync::Arc::downgrade(&self.controller.scene().ok_or(Error::Missing("surface snap scene"))?.document);
            let has_cached = group.iter().any(|candidate| self.surface_snap_cache.iter()
                .any(|(source, input, _)| source.ptr_eq(&document) && input == *candidate));
            if group.len() > 1 && !has_cached {
                let results = self.controller.refine_surface_snap_group(group, camera, viewport)?;
                for ((&position, candidate), result) in order[index..end].iter().zip(group).zip(results) {
                    let Some(result) = result else { continue };
                    if self.surface_snap_cache.len() >= 64 { self.surface_snap_cache.clear(); }
                    self.surface_snap_cache.push((document.clone(), (*candidate).clone(), result.clone()));
                    refined.push((position, result));
                }
            } else {
                for (&position, candidate) in order[index..end].iter().zip(group) {
                    if let Some(result) = self.try_refine_snap_candidate((*candidate).clone(), camera, viewport)? {
                        refined.push((position, result));
                    }
                }
            }
            index = end;
        }
        refined.sort_unstable_by_key(|(position, _)| *position);
        let best = om9_scene::osnap::choose_iter(refined.iter().map(|(_, candidate)| candidate));
        if mode == om9_scene::workspace::DisplayMode::Shaded
            && let Some(candidate) = &best
        {
            let cursor = candidate.surface_source.as_ref().ok_or(Error::Missing("surface source"))?.cursor;
            let scene = self.controller.scene().ok_or(Error::Missing("surface snap scene"))?;
            let tolerance = self.controller.document().effective_tolerance()?.absolute_mm;
            if om9_scene::osnap::visible_display_occludes(scene, camera, viewport, cursor, candidate.world, tolerance, excluded, candidate.surface_source.as_ref())? {
                return Err(Error::Unsupported("surface snap is occluded by shaded geometry"));
            }
        }
        Ok(best)
    }

    /// Snappy mode feeds the active object-snap candidate into the drag. The objects
    /// being dragged are excluded so a handle cannot snap to geometry moving with it.
    fn gumball_snap_candidate(
        &mut self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
    ) -> Result<Option<om9_scene::osnap::SnapCandidate>> {
        if !self.controller.gumball_snappy() {
            return Ok(None);
        }
        let selected: Vec<_> = self.controller.selected().iter().copied().collect();
        let Some(scene) = self.controller.scene() else { return Ok(None) };
        let settings = self.snap_state.settings;
        let mut list =
            om9_scene::osnap::candidates(scene, camera, viewport, pointer, settings, None)?;
        list.retain(|candidate| {
            candidate.root.is_none_or(|root| !selected.contains(&root))
                && settings.one_shot.is_none_or(|kind| candidate.kind == kind)
        });
        let active = self.controller.workspace.active;
        let mode = if self.gpu_state.is_some() {
            self.gpu_viewports.get(&active).and_then(|port| port.displayed_mode())
                .ok_or(Error::Stale("surface snap displayed image"))?
        } else {
            self.controller.workspace.views[&active].mode
        };
        let Some(candidate) = self.resolve_snap_candidates(list, camera, viewport, mode, &selected)? else { return Ok(None) };
        self.snap = Some(candidate.clone());
        Ok(Some(candidate))
    }

    fn update_gumball(
        &mut self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
        snapped: Option<om9_scene::osnap::SnapCandidate>,
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
        // A 3D snap moves the whole frame to the snapped point for a move handle;
        // the handle constraint still projects it for an axis or plane handle.
        if let Some(candidate) = &snapped
            && matches!(
                handle,
                om9_core::gumball::Handle::MoveAxis(_)
                    | om9_core::gumball::Handle::MovePlane(_)
                    | om9_core::gumball::Handle::MoveFree
            )
        {
            let delta: [f64; 3] =
                std::array::from_fn(|i| candidate.world[i] - session.frame().origin[i]);
            session.reset();
            session.drag(handle, om9_core::gumball::Drag::WorldDelta(delta))?;
            let delta = session.accumulated();
            self.gumball_last = Some(pointer);
            return self.controller.preview_gumball(delta);
        }
        let view = om9_scene::gumball_view::GumballView::new(*session.frame(), self.gumball_mode);
        let mut input = snapped
            .as_ref()
            .map_or(pointer, |candidate| candidate.screen);
        // Drag strength scales pointer-driven motion only; saved pivots such as the
        // start point and snapped markers stay exact.
        if snapped.is_none() {
            let strength = self.controller.gumball_drag_strength();
            if strength != 1.0 {
                input = [
                    basis.start[0] + (input[0] - basis.start[0]) * strength,
                    basis.start[1] + (input[1] - basis.start[1]) * strength,
                ];
            }
        }
        if let Some(drag) = view.drag(&basis.camera, basis.viewport, handle, basis.start, input)? {
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
            "sidebar_content_viewport_px": self.sidebar_content_rect.map(|r| [r.left(), r.top(), r.width(), r.height()]),
            "sidebar_scrollbar_px": self.sidebar_scrollbar_rect.map(|r| [r.left(), r.top(), r.width(), r.height()]),
            "sidebar_scroll_offset": self.sidebar_scroll,
            "section_title_bars_px": title_bars,
            "layers_rect_px": layers_rect,
            "right_dock_outer_px": self.right_dock_outer_px,
            "command_line_px": self.regions.command_line,
            "status_bar_px": self.regions.status_bar,
            "object_info": self.object_info_block(),
            "palette": {
                "categories": self.palette_categories(),
                "slots": (1..=33u8)
                    .filter_map(|number| self.palette_slot_label(number).map(|label| (number, label)))
                    .collect::<Vec<_>>(),
            },
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
        self.audit_icon_resize(ctx);
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
/// Responsive icon scale: 1.0 at the 1280x900 design size, gently shrinking on
/// smaller screens and growing on larger ones. Bounded so the 67-pt toolbar and
/// the 240..460-pt rail keep their documented allocations.
fn responsive_icon_scale(rect: Rect) -> f32 {
    if !rect.width().is_finite()
        || !rect.height().is_finite()
        || rect.width() <= 0.
        || rect.height() <= 0.
    {
        return 1.;
    }
    (rect.width() / 1280.)
        .min(rect.height() / 900.)
        .clamp(0.85, 1.2)
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
impl MatrixApp {
    /// Separate dock-style Object Health Report window (not a status line).
    fn health_panel_ui(&mut self, ctx: &egui::Context) {
        if !self.health_panel_open {
            return;
        }
        let entries = self.controller.health_entries();
        let summary = self.controller.health_summary();
        let mut open = true;
        egui::Window::new("Object Health Report")
            .id(egui::Id::new("om9-object-health-report"))
            .open(&mut open)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(summary);
                ui.horizontal(|ui| {
                    ui.label("Feature angle");
                    ui.add(egui::Slider::new(&mut self.feature_angle_deg, 0.0..=180.0).suffix("°"));
                    if ui.button("Reduce Selected").clicked() {
                        match self
                            .controller
                            .reduce_mesh_selected_with_angle(self.feature_angle_deg as f64)
                        {
                            Ok(count) => {
                                self.status.info(&format!("{count} meshes reduced"));
                            }
                            Err(error) => {
                                self.status.error(&error.to_string());
                            }
                        }
                    }
                });
                ui.separator();
                egui::Grid::new("om9-health-grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Object");
                        ui.strong("Kind");
                        ui.strong("Valid");
                        ui.strong("Naked");
                        ui.strong("Non-manifold");
                        ui.strong("Duplicate");
                        ui.end_row();
                        for entry in &entries {
                            ui.label(&entry.label);
                            ui.label(entry.kind);
                            ui.label(if entry.valid { "yes" } else { "no" });
                            ui.label(entry.naked_edges.to_string());
                            ui.label(entry.non_manifold_edges.to_string());
                            ui.label(
                                entry
                                    .duplicate_of
                                    .map(|id| id.to_string())
                                    .unwrap_or_default(),
                            );
                            ui.end_row();
                        }
                    });
            });
        self.health_panel_open = open;
    }
}
impl eframe::App for MatrixApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.live_mcp.as_mut().is_some_and(|host| host.service(&mut self.controller)) {
            self.feedback(Ok(()));
            ctx.request_repaint();
        }
        self.service_window_close(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.attach_clipboard_shortcuts(frame);
        self.show(ui);
        self.health_panel_ui(ui.ctx());
        self.gem_window_ui(ui.ctx());
        self.gem_cluster_window_ui(ui.ctx());
        self.gem_pave_window_ui(ui.ctx());
        self.gem_cutters_window_ui(ui.ctx());
        self.row_cutters_window_ui(ui.ctx());
        self.gem_edits_window_ui(ui.ctx());
        // Enter in the viewport finishes the running point command (Matrix/Rhino).
        if self.controller.is_running()
            && ui.ctx().memory(|memory| memory.focused().is_none())
            && ui
                .ctx()
                .input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter))
        {
            self.submit_draft();
        }
        // Delete removes the selected objects when no text editor owns the keyboard.
        if !self.controller.is_running()
            && !self.controller.selected().is_empty()
            && ui.ctx().memory(|memory| memory.focused().is_none())
            && ui
                .ctx()
                .input_mut(|input| input.consume_key(Modifiers::NONE, Key::Delete))
        {
            if !self.delete_selected_control_grip() {
                let ids: Vec<_> = self.controller.selected().iter().copied().collect();
                let result = self.controller.delete_objects(&ids).map(|_| ());
                self.feedback(result);
            }
        }
        self.service_file_close_probe(ui.ctx(), frame);
        self.service_native_file_picker(ui.ctx(), frame);
        self.service_three_dm_picker(ui.ctx(), frame);
        self.service_stl_picker(ui.ctx(), frame);
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.nav_frame_started = std::time::Instant::now();
        self.clipboard_shortcuts_from_input(ctx, raw);
        self.delete_selection_from_input(ctx, raw);
        let mut audit_input = || {
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
                        range
                            == CCursorRange::one(CCursor::new(
                                self.transcript.text().chars().count(),
                            ))
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
        };
        audit_input();
        if let Some(audit) = self.navigation_audit.as_mut() {
            let center = self
                .view_rects
                .get(&ViewId::Perspective)
                .map(|r| r.center());
            let ready = !self.gpu_viewports.is_empty()
                && self.gpu_viewports.values().all(|p| p.final_complete())
                && !self
                    .navigation
                    .views
                    .values()
                    .any(|s| matches!(s.phase, crate::navigation::NavigationPhase::Settling));
            if let Some(log) = self.interaction_log.as_mut() {
                log.record(serde_json::json!({"event":"navigation_audit_state","ready":ready,"center":center.map(|p|[p.x,p.y]),"final":self.gpu_viewports.iter().map(|(id,p)|(id.title(),p.final_complete())).collect::<BTreeMap<_,_>>()}));
            }
            audit.input(ctx, raw, center, ready, self.scheduler.hz());
        }
        self.nav_raw_events = Some(raw.events.clone());
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
    let navigation_zoom = if app.navigation_audit.is_some() {
        std::env::var("OM9_NAVIGATION_AUDIT_DPI")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite() && (1.0..=1.5).contains(v))
    } else {
        None
    };
    if app.navigation_audit.is_some()
        && std::env::var("OM9_NAVIGATION_AUDIT_VIEWS").is_ok_and(|v| v == "1")
    {
        app.controller.workspace.maximized = Some(ViewId::Perspective);
    }
    let mut surface = eframe::egui_wgpu::SurfaceConfig::LOW_LATENCY;
    if std::env::var("OM9_FRAME_LATENCY").is_ok_and(|v| v == "2") {
        surface.desired_maximum_frame_latency = Some(2);
    }
    let mut wgpu_options =
        eframe::egui_wgpu::WgpuConfiguration::default().with_surface_config(surface);
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut wgpu_options.wgpu_setup {
        let original = setup.device_descriptor.clone();
        setup.device_descriptor = std::sync::Arc::new(move |adapter| {
            let mut descriptor = original(adapter);
            descriptor.required_features |=
                adapter.features() & eframe::wgpu::Features::TIMESTAMP_QUERY;
            descriptor
        });
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(config.window)
            .with_maximized(config.maximized && app.audit.is_none())
            .with_min_inner_size(config.builder_min_window())
            .with_title(&title)
            .with_icon(app_icon()),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options,
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            if let Some(zoom) = audit_zoom {
                cc.egui_ctx.set_zoom_factor(zoom);
            }
            if let Some(zoom)=navigation_zoom {cc.egui_ctx.set_zoom_factor(zoom);}
            app.configure_style(&cc.egui_ctx);
            match crate::mcp::live::LiveHost::start(cc.egui_ctx.clone()) {
                Ok(host) => app.live_mcp = Some(host),
                Err(error) => app.transcript.append_history(&format!("MCP live: {error}")),
            }
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
            if let Some(log)=app.interaction_log.as_mut(){log.record(serde_json::json!({"event":"gpu_configuration","adapter":format!("{info:?}"),"timestamp_queries":state.device.features().contains(eframe::wgpu::Features::TIMESTAMP_QUERY),"frame_latency":surface.desired_maximum_frame_latency,"present_mode":format!("{:?}",surface.present_mode),"pixels_per_point":cc.egui_ctx.pixels_per_point(),"zoom_factor":cc.egui_ctx.zoom_factor(),"dpi_is_simulated":navigation_zoom.is_some()}));}
            #[cfg(windows)]
            if let Some(log) = app.interaction_log.as_mut() {
                let event = match om9_native_owner::gpu_memory::dx12_adapter_memory(&state.adapter) {
                    Some(sample) => serde_json::json!({"event":"gpu_memory_sample","query_succeeded":true,"source":"dxgi_selected_dx12_adapter","luid_low":sample.luid_low,"luid_high":sample.luid_high,"vendor_id":sample.vendor_id,"device_id":sample.device_id,"dedicated_video_bytes":sample.dedicated_video_bytes,"shared_system_bytes":sample.shared_system_bytes,"local_budget_bytes":sample.local_budget_bytes,"local_usage_bytes":sample.local_usage_bytes,"nonlocal_budget_bytes":sample.nonlocal_budget_bytes,"nonlocal_usage_bytes":sample.nonlocal_usage_bytes,"budget_preview":crate::gpu_budget::preview_dxgi(sample, info.device_type)}),
                    None => match (om9_native_owner::gpu_memory::vulkan_dxgi_adapter_memory(&state.adapter, &state.device), om9_native_owner::gpu_memory::vulkan_device_memory(&state.adapter, &state.device)) {
                        (Some(sample), _) => serde_json::json!({"event":"gpu_memory_sample","query_succeeded":true,"source":"dxgi_matched_vulkan_luid","luid_low":sample.luid_low,"luid_high":sample.luid_high,"vendor_id":sample.vendor_id,"device_id":sample.device_id,"dedicated_video_bytes":sample.dedicated_video_bytes,"local_budget_bytes":sample.local_budget_bytes,"local_usage_bytes":sample.local_usage_bytes,"budget_preview":crate::gpu_budget::preview_dxgi(sample, info.device_type)}),
                        (None, Some(heaps)) => serde_json::json!({"event":"gpu_memory_sample","query_succeeded":true,"source":"vulkan_selected_device_memory_budget","budget_preview":crate::gpu_budget::preview_vulkan(&heaps, info.device_type),"heaps":heaps.iter().map(|heap| serde_json::json!({"size_bytes":heap.size_bytes,"budget_bytes":heap.budget_bytes,"usage_bytes":heap.usage_bytes,"device_local":heap.device_local,"host_visible":heap.host_visible})).collect::<Vec<_>>() }),
                        (None, None) => serde_json::json!({"event":"gpu_memory_sample","query_succeeded":false,"reason":"backend or memory-budget query unsupported","budget_preview":crate::gpu_budget::unverified_preview(info.device_type)}),
                    },
                };
                log.record(event);
            }
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

            corner_uvs: Vec::new(),
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
    fn unknown_physical_ram_is_reported_in_status_and_transcript() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("memory fallback")).unwrap();
        app.report_memory_fallback(false);
        assert!(app.status.message().contains("at most 256 MiB"));
        assert!(app.transcript.history().contains("at most 256 MiB"));
    }

    #[test]
    fn delayed_hover_receipt_rejects_each_changed_basis() {
        use super::HoverBasis;
        use om9_scene::camera::{Camera, Viewport};
        use std::sync::{Arc, mpsc};
        let document = Arc::new(Document::new("hover receipt"));
        let document_weak = Arc::downgrade(&document);
        let camera = Camera::orthographic([0., 0., 10.], [0.; 3], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
        let pointer = egui::pos2(200., 200.);
        let basis = HoverBasis { document: document_weak.clone(), camera: camera.clone(), epoch: 7, preview: 3, pointer, viewport };
        let (send, receive) = mpsc::channel();
        let target = om9_core::model::Entity::new("target", om9_core::geometry::Geometry::Point([0.; 3]), None).id;
        // The result is delivered only after the receipt's source has changed.
        send.send(Some(target)).unwrap();
        assert_eq!(receive.recv().unwrap(), Some(target));
        assert!(basis.matches(&document_weak, &camera, 7, 3, viewport, pointer));
        assert!(!basis.matches(&document_weak, &camera, 7, 4, viewport, pointer));
        assert!(!basis.matches(&document_weak, &camera, 8, 3, viewport, pointer));
        let replacement = Arc::new((*document).clone());
        assert_eq!(replacement.id, document.id);
        assert!(!basis.matches(&Arc::downgrade(&replacement), &camera, 7, 3, viewport, pointer));
        let mut moved_camera = camera.clone();
        moved_camera.eye[0] += 1.;
        assert_eq!(moved_camera.revision, camera.revision);
        assert!(!basis.matches(&document_weak, &moved_camera, 7, 3, viewport, pointer));
        let resized = Viewport { origin: viewport.origin, size: [401., 400.] };
        assert!(!basis.matches(&document_weak, &camera, 7, 3, resized, pointer));
        assert!(!basis.matches(&document_weak, &camera, 7, 3, viewport, pointer + egui::vec2(4., 0.)));
        assert!(!basis.matches(&document_weak, &camera, 7, 3, viewport, pointer + egui::vec2(1., 0.)));
    }

    #[test]
    fn pending_hover_does_not_keep_a_hit_after_pointer_moves() {
        use super::HoverBasis;
        use egui::{Event, Pos2, Rect, Vec2};
        use om9_core::{geometry::Geometry, model::Entity};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut document = Document::new("pending hover pointer movement");
        let entity = Entity::new("point", Geometry::Point([0.; 3]), None);
        let target = entity.id;
        document.entities.insert(target, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let ctx = egui::Context::default();
        let frame = |app: &mut MatrixApp, events: Vec<Event>| {
            let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))), events, ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| app.show(ui));
            out.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let rect = app.view_rects[&ViewId::Perspective];
        let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
        let camera = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
        let at = camera.project([0.; 3], viewport).unwrap().unwrap().screen;
        let pointer = Pos2::new(at[0] as f32, at[1] as f32);
        frame(&mut app, vec![Event::PointerMoved(pointer)]);
        assert_eq!(app.hovered_object(), Some(target));
        let scene = app.controller.scene().unwrap();
        let basis = HoverBasis { document: std::sync::Arc::downgrade(&scene.document), camera, epoch: scene.epoch,
            preview: app.controller.preview_revision(), pointer, viewport };
        let (send, receive) = std::sync::mpsc::channel();
        app.hover_job = Some((basis, receive));
        let moved = pointer + Vec2::new(40., 0.);
        assert!(rect.contains(moved));
        frame(&mut app, vec![Event::PointerMoved(moved)]);
        assert_eq!(app.hovered_object(), None, "an old hit must not remain highlighted while the worker is pending");
        assert!(app.hover_job.is_some(), "fixture must keep the old worker pending");
        send.send(Some(target)).unwrap();
        frame(&mut app, vec![]);
        assert_eq!(app.hovered_object(), None, "the old result must not restore the highlight");
    }

    #[test]
    #[cfg(windows)]
    fn stationary_hover_retries_after_temporary_memory_pressure() {
        use egui::{Event, Pos2, Rect, Vec2};
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut document = Document::new("hover retry");
        let target = Entity::new("target", Geometry::Point([0.; 3]), None);
        let id = target.id;
        document.entities.insert(id, target);
        for index in 0..129 {
            let point = Entity::new("far", Geometry::Point([1000. + index as f64, 1000., 0.]), None);
            document.entities.insert(point.id, point);
        }
        // Exercise managed pressure without assuming the whole machine's 60%
        // RAM ceiling also fits its currently available system commit.
        let limits = om9_core::budget::Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() };
        let mut app = MatrixApp::from_controller(crate::session::Controller::new(document, limits).unwrap()).unwrap();
        let ctx = egui::Context::default();
        let frame = |app: &mut MatrixApp, events: Vec<Event>| {
            let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))), events, ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| app.show(ui));
            out.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let rect = app.view_rects[&ViewId::Perspective];
        let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
        let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
        let screen = camera.project([0.; 3], viewport).unwrap().unwrap().screen;
        let pointer = Pos2::new(screen[0] as f32, screen[1] as f32);
        let coordinator = app.controller.coordinator();
        let held = app.controller.scene_cache_usage().unwrap();
        let mut reservation = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
        reservation.shrink_to(reservation.bytes() - 1024).unwrap();
        frame(&mut app, vec![Event::PointerMoved(pointer)]);
        assert_eq!(app.hovered_object(), None);
        assert!(app.hover_retry_at.is_some(), "refusal must schedule a bounded retry");
        let repaint_delays = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded = repaint_delays.clone();
        ctx.set_request_repaint_callback(move |info| recorded.lock().unwrap().push(info.delay));
        app.hover_retry_at = Some(std::time::Instant::now() + std::time::Duration::from_millis(100));
        for _ in 0..3 { frame(&mut app, vec![]); }
        let delays = repaint_delays.lock().unwrap().clone();
        assert!(delays.iter().any(|delay| *delay > std::time::Duration::ZERO
            && *delay <= std::time::Duration::from_millis(100)), "an intervening frame must renew the delayed retry repaint: {delays:?}");
        frame(&mut app, vec![]);
        assert!(app.hover_job.is_none(), "no retry is launched before its deadline");
        drop(reservation);
        app.hover_retry_at = Some(std::time::Instant::now());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.hovered_object() != Some(id) {
            frame(&mut app, vec![]);
            assert!(std::time::Instant::now() < deadline, "stationary hover did not recover");
            std::thread::yield_now();
        }
        assert!(app.hover_retry_at.is_none());
        let scene = app.controller.scene().unwrap();
        let owner_bytes = std::mem::size_of::<crate::session::SceneBufferLease>() as u64
            + 4 * std::mem::size_of::<usize>() as u64;
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache),
            scene.shared_buffer_bytes() + owner_bytes + scene.memory_allowance_bytes());
    }

    #[test]
    #[cfg(windows)]
    fn inline_hover_retries_after_shared_memory_pressure() {
        use egui::{Event, Pos2, Rect, Vec2};
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut document = Document::new("inline hover pressure");
        let point = Entity::new("target", Geometry::Point([0.; 3]), None);
        let id = point.id;
        document.entities.insert(id, point);
        let mut app = MatrixApp::new(document).unwrap();
        let ctx = egui::Context::default();
        let frame = |app: &mut MatrixApp, events: Vec<Event>| {
            let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))), events, ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| app.show(ui));
            out.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let rect = app.view_rects[&ViewId::Perspective];
        let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
        let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
        let screen = camera.project([0.; 3], viewport).unwrap().unwrap().screen;
        let pointer = Pos2::new(screen[0] as f32, screen[1] as f32);
        let held = app.controller.scene_cache_usage().unwrap();
        let baseline_buffers = app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, held,
        ).unwrap();
        frame(&mut app, vec![Event::PointerMoved(pointer)]);
        assert_eq!(app.hovered_object(), None);
        assert!(app.hover_retry_at.is_some());
        drop(pressure);
        app.hover_retry_at = Some(std::time::Instant::now());
        frame(&mut app, vec![]);
        assert_eq!(app.hovered_object(), Some(id));
        assert!(app.hover_retry_at.is_none());
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline_buffers);
    }

    #[test]
    fn inline_hover_reserves_single_mesh_workspace_and_releases_it() {
        use om9_core::{budget::memory::MemoryClass, geometry::{Geometry, Mesh}, model::Entity};
        let mut document = Document::new("single mesh hover");
        let vertices = (0..4096).map(|i| [i as f64, 0., 0.]).collect();
        let mesh = Mesh { vertices, faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        document.entities.insert(entity.id, entity);
        let app = MatrixApp::new(document).unwrap();
        let coordinator = app.controller.coordinator();
        let before = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let reservation = app.reserve_inline_hover_pick(None).unwrap();
        assert!(reservation.bytes() >= (4096 * std::mem::size_of::<Option<[f64; 3]>>()) as u64);
        assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) >= before + reservation.bytes());
        drop(reservation);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), before);
    }

    #[test]
    fn hover_snapshot_requires_physical_ram_sample_and_releases_reservation() {
        use om9_core::budget::memory::MemoryClass;
        let mut document = Document::new("hover memory sample");
        let point = om9_core::model::Entity::new("point", om9_core::geometry::Geometry::Point([0.; 3]), None);
        document.entities.insert(point.id, point);
        let app = MatrixApp::new(document).unwrap();
        let document = app.controller.document().clone();
        let coordinator = app.controller.coordinator();
        let before = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        assert!(matches!(app.reserve_hover_snapshot(None), Err(om9_core::Error::Unsupported(_))));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), before);
        assert_eq!(app.controller.document(), &document);
        let mib = 1024 * 1024;
        assert!(matches!(app.reserve_hover_snapshot(Some((1024 * mib, 1))), Err(om9_core::Error::Budget(_))));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache),
            before - app.controller.scene().unwrap().picking_arena_bytes().unwrap(),
            "failed physical admission may release unused cache quota, not source/metadata");
        let (capture, buffers, reservation) = app.reserve_hover_snapshot(Some((1024 * mib, 1024 * mib))).unwrap();
        assert!(reservation.physical_sampled());
        assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) > before);
        let live_buffer_charge = coordinator.external_reserved_bytes(MemoryClass::CpuCache) - reservation.bytes();
        drop((capture, buffers, reservation));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), live_buffer_charge);
        assert_eq!(app.controller.document(), &document);
    }

    #[test]
    fn hover_metadata_materializes_but_pick_scratch_stays_pending_across_new() {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity};
        let mut document = Document::new("hover metadata phases");
        let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(point.id, point);
        let mut app = MatrixApp::new(document).unwrap();
        app.controller.scene().unwrap().spatial_index().unwrap();
        let baseline = app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let old_main = app.controller.scene().unwrap().memory_allowance_bytes();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let snapshot = app.capture_hover_snapshot(physical).unwrap();
        let metadata = super::hover_snapshot_overhead();
        let scratch = snapshot.scene.nearest_pick_scratch_estimate().unwrap().bytes;
        assert_eq!(snapshot.reservation.bytes(), metadata + scratch);
        assert_eq!(snapshot.reservation.pending_bytes(), scratch);
        let worker = snapshot.scene.memory_allowance_bytes() + metadata + scratch;
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline + worker);
        assert_eq!(app.controller.coordinator().external_pending_bytes(MemoryClass::CpuCache), scratch + snapshot.scene.picking_arena_bytes().unwrap());
        app.controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache),
            baseline - old_main + app.controller.scene().unwrap().memory_allowance_bytes() + worker);
        drop(snapshot);
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), app.controller.scene().unwrap().memory_allowance_bytes());
        assert_eq!(app.controller.coordinator().external_pending_bytes(MemoryClass::CpuCache), app.controller.scene().unwrap().picking_arena_bytes().unwrap());
    }

    #[test]
    fn hover_shares_viewport_buffers_under_pressure_and_across_new_document() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Mesh}, model::{Document, Entity}};
        let mut document = Document::with_template_tolerance("shared hover buffers");
        let mut vertices = vec![[0., 0., 0.]; 40_000];
        vertices[1] = [1., 0., 0.];
        vertices[2] = [0., 1., 0.];
        let mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
            vertices, faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        document.entities.insert(mesh.id, mesh);
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut app = MatrixApp::from_controller(crate::session::Controller::new(document, limits).unwrap()).unwrap();
        let mib = 1024 * 1024;
        let physical = (1024 * mib, 1024 * mib);
        let scene = app.controller.scene().unwrap();
        let buffer_bytes = scene.shared_buffer_bytes();
        let metadata = scene.cache_usage().unwrap().bytes - buffer_bytes;
        let scratch = scene.nearest_pick_scratch_estimate().unwrap().bytes;
        let viewport_buffers = app.controller.share_scene_buffer_owner_with_sample(scene, Some(physical)).unwrap();
        let held = app.controller.scene_cache_usage().unwrap();
        let mut pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, held,
        ).unwrap();
        let slack = metadata + scratch + buffer_bytes / 2;
        assert!(slack < metadata + scratch + buffer_bytes);
        pressure.shrink_to(pressure.bytes() - slack).unwrap();
        let (capture, hover_buffers, reservation) = app.reserve_hover_snapshot(Some(physical)).unwrap();
        assert!(std::sync::Arc::ptr_eq(&viewport_buffers, &hover_buffers));
        let snapshot = app.controller.scene().unwrap().try_render_snapshot().unwrap();
        drop(viewport_buffers);
        drop(pressure);
        app.controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(app.controller.document().entities.is_empty());
        assert!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) >= buffer_bytes);
        drop(snapshot);
        drop((capture, hover_buffers, reservation));
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), app.controller.scene().unwrap().memory_allowance_bytes());
    }

    #[test]
    fn delayed_hover_worker_result_is_not_published_after_preview_moves() {
        use super::HoverBasis;
        use egui::{Event, Pos2, Rect, Vec2};
        use om9_core::{geometry::{Geometry, Transform}, model::Entity};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut document = Document::new("delayed hover UI");
        let entity = Entity::new("point", Geometry::Point([0.; 3]), None);
        let target = entity.id;
        document.entities.insert(target, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let ctx = egui::Context::default();
        let frame = |app: &mut MatrixApp, events: Vec<Event>| {
            let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))), events, ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| app.show(ui));
            out.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let rect = app.view_rects[&ViewId::Perspective];
        let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
        let camera = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
        let at = camera.project([0.; 3], viewport).unwrap().unwrap().screen;
        let pointer = Pos2::new(at[0] as f32, at[1] as f32);
        frame(&mut app, vec![Event::PointerMoved(pointer)]);
        assert_eq!(app.hovered_object(), Some(target));
        let scene = app.controller.scene().unwrap();
        let basis = HoverBasis { document: std::sync::Arc::downgrade(&scene.document), camera, epoch: scene.epoch,
            preview: app.controller.preview_revision(), pointer, viewport };
        let (send, receive) = std::sync::mpsc::channel();
        app.hover_job = Some((basis, receive));
        app.controller.select_objects(&[target]).unwrap();
        app.controller.preview_gumball(Transform::translation([1000., 0., 0.])).unwrap();
        send.send(Some(target)).unwrap();
        frame(&mut app, vec![]);
        assert_eq!(app.hovered_object(), None, "the old worker receipt must not publish");
        assert!(app.hover_job.is_none());
    }

    #[test]
    fn delayed_hover_worker_result_is_not_published_after_document_replacement() {
        use super::HoverBasis;
        use egui::{Event, Pos2, Rect, Vec2};
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut document = Document::new("replaced hover UI");
        let entity = Entity::new("point", Geometry::Point([0.; 3]), None);
        let target = entity.id;
        document.entities.insert(target, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let ctx = egui::Context::default();
        let frame = |app: &mut MatrixApp, events: Vec<Event>| {
            let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))), events, ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| app.show(ui));
            out.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let rect = app.view_rects[&ViewId::Perspective];
        let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
        let camera = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
        let at = camera.project([0.; 3], viewport).unwrap().unwrap().screen;
        let pointer = Pos2::new(at[0] as f32, at[1] as f32);
        frame(&mut app, vec![Event::PointerMoved(pointer)]);
        assert_eq!(app.hovered_object(), Some(target));
        let scene = app.controller.scene().unwrap();
        let basis = HoverBasis { document: std::sync::Arc::downgrade(&scene.document), camera, epoch: scene.epoch,
            preview: app.controller.preview_revision(), pointer, viewport };
        let (send, receive) = std::sync::mpsc::channel();
        app.hover_job = Some((basis, receive));
        let mut replacement = app.controller.document().clone();
        replacement.entities.clear();
        app.controller = crate::session::Controller::new(replacement, Limits::default()).unwrap();
        send.send(Some(target)).unwrap();
        frame(&mut app, vec![]);
        assert_eq!(app.hovered_object(), None, "receipt for the old allocation must not publish");
        assert!(app.hover_job.is_none());
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn cached_surface_snap_rejects_a_changed_camera() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}};
        let limits = Limits::default();
        let mut document = Document::with_template_tolerance("cached surface context");
        let entity = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = [300., 300.];
        let candidate = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().find(|hit| hit.kind == SnapKind::Surface && hit.root == Some(id)).unwrap();
        let first = app.refine_snap_candidate(candidate.clone(), &camera, viewport).unwrap();
        assert!((first.world[2] - 3.).abs() < 1e-6);
        assert_eq!(app.refine_snap_candidate(candidate.clone(), &camera, viewport).unwrap(), first);
        assert_eq!(app.surface_snap_cache.len(), 1);
        // A newly captured document can reuse IDs and revisions; its allocation
        // must not reuse refinement cached for the previous captured source.
        let replacement = app.controller.document().clone();
        app.controller = crate::session::Controller::new(replacement, limits).unwrap();
        assert_eq!(app.refine_snap_candidate(candidate.clone(), &camera, viewport).unwrap(), first);
        assert_eq!(app.surface_snap_cache.len(), 2);
        let mut moved = camera.clone();
        moved.pan([1., 0.]).unwrap();
        assert!(app.refine_snap_candidate(candidate, &moved, viewport).is_err(), "a cached point must still validate the current camera");
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn multi_face_surface_snap_keeps_front_hit_and_cache_context() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let brep = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("multi-face UI snap");
        let entity = Entity::new("box", Geometry::Brep(brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([5., 5., 30.], [5., 5., 0.], [0., 1., 0.], 15.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = camera.project([4., 4., 10.], viewport).unwrap().unwrap().screen;
        let candidates: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().filter(|hit| hit.kind == SnapKind::Surface && hit.root == Some(id)).collect();
        assert_eq!(candidates.len(), 2);
        let hit = app.resolve_snap_candidates(candidates.clone(), &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
        assert!((hit.world[2] - 10.).abs() < 1e-7);
        assert_eq!(app.surface_snap_cache.len(), 2);
        app.surface_snap_cache.pop();
        let cached = app.resolve_snap_candidates(candidates.clone(), &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
        assert_eq!(cached, hit, "a partially cached group must match the fresh result");
        let mut moved = camera.clone(); moved.pan([1., 0.]).unwrap();
        assert!(app.resolve_snap_candidates(candidates, &moved, viewport, DisplayMode::Wireframe, &[]).is_err());
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn surface_candidates_with_one_exact_miss_preserve_other_exact_hits() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let brep = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let faces = brep.display.faces.clone();
        let mut document = Document::with_template_tolerance("candidate miss");
        let entity = Entity::new("box", Geometry::Brep(brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([5., 5., 30.], [5., 5., 0.], [0., 1., 0.], 15.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = camera.project([4., 4., 10.], viewport).unwrap().unwrap().screen;
        let mut candidates: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().filter(|hit| hit.kind == SnapKind::Surface && hit.root == Some(id)).collect();
        assert_eq!(candidates.len(), 2);
        let visible_faces: Vec<_> = candidates.iter().map(|c| c.surface_source.as_ref().unwrap().face.unwrap()).collect();
        let missing_face = *faces.iter().find(|face| !visible_faces.contains(face)).unwrap();
        let mut false_candidate = candidates[0].clone();
        false_candidate.surface_source.as_mut().unwrap().face = Some(missing_face);
        assert!(app.resolve_snap_candidates(vec![false_candidate.clone()], &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().is_none());
        for index in 0..=candidates.len() {
            app.surface_snap_cache.clear();
            let mut mixed = candidates.clone();
            mixed.insert(index, false_candidate.clone());
            for _ in 0..2 {
                let hit = app.resolve_snap_candidates(mixed.clone(), &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
                assert!((hit.world[2] - 10.).abs() < 1e-7);
                assert_ne!(hit.surface_source.unwrap().face, Some(missing_face));
            }
            assert_eq!(app.surface_snap_cache.len(), 2, "misses must not be cached as hits");
        }
        let mut invalid = false_candidate;
        invalid.surface_source.as_mut().unwrap().face = Some(uuid::Uuid::new_v4());
        candidates.insert(0, invalid);
        assert!(app.resolve_snap_candidates(candidates, &camera, viewport, DisplayMode::Wireframe, &[]).is_err(), "unknown faces are errors, not misses");
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn nurbs_exact_miss_does_not_discard_a_valid_brep_surface_hit() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let sphere = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        let sphere_id = sphere.id;
        let plane = om9_core::surface_kernel::plane([5., -1., 5.], [1., 0., 0.], [0., 1., 0.], 2., 2., &limits).unwrap();
        let plane = Entity::new("plane", Geometry::Surface(plane), None);
        let plane_id = plane.id;
        let ray_origin = [0., 0., 20.];
        let ray_direction = [0., 0., -1.];
        assert!(om9_kernel::measurement::try_surface_ray_hit(&plane.geometry, plane.placement, ray_origin, ray_direction, ray_origin, 0.001, &limits).unwrap().is_none());
        assert!(om9_kernel::measurement::surface_ray_hit(&plane.geometry, plane.placement, ray_origin, ray_direction, ray_origin, 0.001, &limits).is_err(), "existing entrypoint must preserve error-on-miss");
        assert!(om9_kernel::measurement::try_surface_ray_hit(&plane.geometry, plane.placement, ray_origin, [0.; 3], ray_origin, 0.001, &limits).is_err(), "invalid rays must not become misses");
        let mut document = Document::with_template_tolerance("NURBS exact miss");
        document.entities.insert(sphere_id, sphere);
        document.entities.insert(plane_id, plane);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 20.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = camera.project([0., 0., 3.], viewport).unwrap().unwrap().screen;
        let plane_cursor = camera.project([6., 0., 5.], viewport).unwrap().unwrap().screen;
        let mut false_candidate = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, plane_cursor, Default::default(), None).unwrap()
            .into_iter().find(|hit| hit.kind == SnapKind::Surface && hit.root == Some(plane_id)).unwrap();
        false_candidate.surface_source.as_mut().unwrap().cursor = cursor;
        false_candidate.screen = cursor;
        false_candidate.distance_px = 0.;
        let target = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().find(|hit| hit.kind == SnapKind::Surface && hit.root == Some(sphere_id)).unwrap();
        for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
            assert!(app.resolve_snap_candidates(vec![false_candidate.clone()], &camera, viewport, mode, &[]).unwrap().is_none());
            for candidates in [vec![false_candidate.clone(), target.clone()], vec![target.clone(), false_candidate.clone()]] {
                let hit = app.resolve_snap_candidates(candidates, &camera, viewport, mode, &[]).unwrap().unwrap();
                assert_eq!(hit.root, Some(sphere_id));
                assert!((hit.world[2] - 3.).abs() < 1e-7);
            }
            assert!(app.surface_snap_cache.iter().all(|(_, input, _)| input.root != Some(plane_id)), "misses must not be cached as hits");
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn a_real_nurbs_display_hit_beyond_far_plane_does_not_hide_a_front_sphere() {
        use om9_core::{budget::Limits, geometry::{Geometry, NurbsSurface}, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let surface = NurbsSurface {
            degree: [2, 1], size: [3, 2],
            poles: vec![[-5., -5., 0.], [-5., 5., 0.], [0., -5., -4.], [0., 5., -4.], [5., -5., 0.], [5., 5., 0.]],
            weights: vec![1.; 6], knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 1., 1.]],
            domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
        };
        let surface = Entity::new("curved NURBS", Geometry::Surface(surface), None);
        let surface_id = surface.id;
        let sphere = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        let sphere_id = sphere.id;
        let mut document = Document::with_template_tolerance("far clipped exact surface");
        document.entities.insert(surface_id, surface);
        document.entities.insert(sphere_id, sphere);
        let mut app = MatrixApp::new(document).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cameras = [
            Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 20.).unwrap(),
            Camera::perspective([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 0.8).unwrap(),
        ];
        for mut camera in cameras {
            let mut found = None;
            for i in 1..280 {
                let cursor = camera.project([i as f64 * 0.01, 0.23, 0.], viewport).unwrap().unwrap().screen;
                let candidates = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
                if let Some(display) = candidates.iter().find(|hit| hit.kind == SnapKind::Surface && hit.root == Some(surface_id)) {
                    let exact = app.controller.refine_surface_snap(display, &camera, viewport).unwrap();
                    let analytic_z = -2. * (1. - exact.world[0] * exact.world[0] / 25.);
                    assert!((exact.world[2] - analytic_z).abs() < 1e-7, "OCCT must agree with the analytic surface");
                    let display_depth = camera.project(display.world, viewport).unwrap().unwrap().depth;
                    let exact_depth = camera.project(exact.world, viewport).unwrap().unwrap().depth;
                    if exact_depth - display_depth > 1e-5 {
                        found = Some((cursor, display_depth, exact_depth));
                        break;
                    }
                }
            }
            let (cursor, display_depth, exact_depth) = found.expect("fixture must expose actual display approximation");
            camera.far_mm = (display_depth + exact_depth) / 2.;
            let candidates: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
                .into_iter().filter(|hit| hit.kind == SnapKind::Surface).collect();
            assert!(candidates.iter().any(|hit| hit.root == Some(surface_id)));
            assert!(candidates.iter().any(|hit| hit.root == Some(sphere_id)));
            let clipped = candidates.iter().find(|hit| hit.root == Some(surface_id)).unwrap().clone();
            assert!(app.resolve_snap_candidates(vec![clipped], &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().is_none());
            for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
                let hit = app.resolve_snap_candidates(candidates.clone(), &camera, viewport, mode, &[]).unwrap().unwrap();
                assert_eq!(hit.root, Some(sphere_id));
                assert!(hit.world[2] > 0.);
            }
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn clipped_face_in_brep_group_does_not_discard_a_visible_face() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let mut document = Document::with_template_tolerance("clipped B-rep face");
        let entity = Entity::new("box", Geometry::Brep(om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap()), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let mut camera = Camera::orthographic([5., 5., 30.], [5., 5., 0.], [0., 1., 0.], 15.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = camera.project([4., 4., 10.], viewport).unwrap().unwrap().screen;
        let original: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().filter(|hit| hit.kind == SnapKind::Surface && hit.root == Some(id)).collect();
        assert_eq!(original.len(), 2);
        let back_face = original.iter().find(|hit| hit.world[2].abs() < 1e-7).unwrap().surface_source.as_ref().unwrap().face;
        camera.far_mm = 25.;
        let front = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().find(|hit| hit.kind == SnapKind::Surface && hit.root == Some(id)).unwrap();
        let mut clipped = front.clone();
        clipped.surface_source.as_mut().unwrap().face = back_face;
        assert!(app.resolve_snap_candidates(vec![clipped.clone()], &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().is_none());
        for candidates in [vec![clipped.clone(), front.clone()], vec![front.clone(), clipped.clone()]] {
            let hit = app.resolve_snap_candidates(candidates, &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
            assert!((hit.world[2] - 10.).abs() < 1e-7);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn interleaved_box_faces_group_by_instance_without_changing_front_hit() {
        use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let brep = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("interleaved UI snap");
        let back = Entity::new("back", Geometry::Brep(brep.clone()), None);
        let back_id = back.id;
        let mut front = Entity::new("front", Geometry::Brep(brep), None);
        front.placement = Transform::translation([0., 0., 5.]);
        let front_id = front.id;
        document.entities.insert(back_id, back);
        document.entities.insert(front_id, front);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([5., 5., 30.], [5., 5., 0.], [0., 1., 0.], 15.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = camera.project([4., 4., 15.], viewport).unwrap().unwrap().screen;
        let candidates: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().filter(|hit| hit.kind == SnapKind::Surface).collect();
        assert_eq!(candidates.iter().map(|c| c.root.unwrap()).collect::<Vec<_>>(), vec![front_id, back_id, front_id, back_id]);
        let order = super::surface_group_order(&candidates).unwrap();
        assert!(crate::session::Controller::can_group_surface_candidates(&candidates[order[0]], &candidates[order[1]]));
        assert!(crate::session::Controller::can_group_surface_candidates(&candidates[order[2]], &candidates[order[3]]));
        let hit = app.resolve_snap_candidates(candidates, &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
        assert_eq!(hit.root, Some(front_id));
        assert!((hit.world[2] - 15.).abs() < 1e-7);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn grouped_surface_snap_separates_members_under_one_instance_root() {
        use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::{Definition, Entity}};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let brep = om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap();
        let back = Entity::new("back member", Geometry::Brep(brep.clone()), None);
        let mut front = Entity::new("front member", Geometry::Brep(brep), None);
        front.placement = Transform::translation([0., 0., 5.]);
        let front_leaf = front.id;
        let definition = Definition::new("two boxes", vec![back, front]);
        let definition_id = definition.id;
        let root = Entity::new("block", Geometry::Instance(definition_id), None);
        let root_id = root.id;
        let mut document = Document::with_template_tolerance("member paths");
        document.definitions.insert(definition_id, definition);
        document.entities.insert(root_id, root);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([5., 5., 30.], [5., 5., 0.], [0., 1., 0.], 15.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let cursor = camera.project([4., 4., 15.], viewport).unwrap().unwrap().screen;
        let candidates: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().filter(|hit| hit.kind == SnapKind::Surface).collect();
        assert_eq!(candidates.len(), 4);
        assert!(candidates.iter().all(|hit| hit.root == Some(root_id)));
        let order = super::surface_group_order(&candidates).unwrap();
        assert!(crate::session::Controller::can_group_surface_candidates(&candidates[order[0]], &candidates[order[1]]));
        assert!(!crate::session::Controller::can_group_surface_candidates(&candidates[order[1]], &candidates[order[2]]));
        assert!(crate::session::Controller::can_group_surface_candidates(&candidates[order[2]], &candidates[order[3]]));
        let hit = app.resolve_snap_candidates(candidates, &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
        assert_eq!(hit.surface_source.unwrap().leaf, front_leaf);
        assert!((hit.world[2] - 15.).abs() < 1e-7);
    }

    #[cfg(all(feature = "occt-brep", windows))]
    #[test]
    #[ignore = "isolated release Surface UI diagnostic"]
    fn surface_ui_grouped_release_probe() {
        use om9_core::{budget::Limits, geometry::Geometry, model::Entity};
        use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}, workspace::DisplayMode};
        let limits = Limits::default();
        let path = std::env::var_os("OM9_SURFACE_BREP_PATH").expect("prepared 16-hole source");
        let brep = om9_kernel::brep::read_brep(&std::fs::read_to_string(path).unwrap(), 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("Surface UI release probe");
        let entity = Entity::new("16-hole block", Geometry::Brep(brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let camera = Camera::orthographic([0., 0., 30.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let candidates: Vec<_> = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, [300., 300.], Default::default(), None).unwrap()
            .into_iter().filter(|candidate| candidate.kind == SnapKind::Surface && candidate.root == Some(id)).collect();
        assert_eq!(candidates.len(), 2);
        let (mut baseline, mut grouped) = (Vec::new(), Vec::new());
        for sample in 0..33 {
            app.surface_snap_cache.clear();
            let started = std::time::Instant::now();
            let single: Vec<_> = candidates.iter().cloned().map(|candidate| app.refine_snap_candidate(candidate, &camera, viewport).unwrap()).collect();
            let baseline_ms = started.elapsed().as_secs_f64() * 1000.;
            app.surface_snap_cache.clear();
            let started = std::time::Instant::now();
            let group = app.resolve_snap_candidates(candidates.clone(), &camera, viewport, DisplayMode::Wireframe, &[]).unwrap().unwrap();
            let grouped_ms = started.elapsed().as_secs_f64() * 1000.;
            let expected = osnap::choose(&single).unwrap();
            assert_eq!(group.root, expected.root);
            assert!(group.world.iter().zip(expected.world).all(|(a,b)| (a-b).abs() < 1e-7));
            assert!((group.world[2]-10.).abs() < 1e-6);
            if sample >= 3 { baseline.push(baseline_ms); grouped.push(grouped_ms); }
        }
        baseline.sort_by(f64::total_cmp); grouped.sort_by(f64::total_cmp);
        // Nearest-rank p95/p99 on exactly 30 measured samples.
        println!("SURFACE_UI candidates=2 samples=30 baseline_p50_ms={:.3} baseline_p95_ms={:.3} baseline_p99_ms={:.3} grouped_p50_ms={:.3} grouped_p95_ms={:.3} grouped_p99_ms={:.3}",
            baseline[14], baseline[28], baseline[29], grouped[14], grouped[28], grouped[29]);
    }

    #[test]
    fn interactive_start_initializes_document_tolerance() {
        let config = crate::startup::AppConfig::default();
        let app = MatrixApp::start(&config).unwrap();
        assert_eq!(
            app.controller.document().effective_tolerance().unwrap(),
            om9_core::history::Tolerance::om9_template()
        );
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_second_click_does_not_jump_before_pointer_moves() {
        use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut app=MatrixApp::new(Document::new("box height transition")).unwrap();
        app.controller.workspace.active=ViewId::Perspective;
        app.controller.submit("Box").unwrap();app.controller.accept_world_point([0.,0.,0.]).unwrap();
        let ctx=egui::Context::default();
        let frame=|app:&mut MatrixApp,events:Vec<Event>| {
            let mut raw=egui::RawInput {screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(1280.,720.))),events,..Default::default()};
            eframe::App::raw_input_hook(app,&ctx,&mut raw);
            let mut out=ctx.run_ui(raw,|ui|app.show(ui));out.textures_delta.clear();
        };
        for _ in 0..4 {frame(&mut app,vec![]);}
        let rect=app.view_rects[&ViewId::Perspective];let view=app.controller.workspace.views[&ViewId::Perspective].clone();
        let viewport=Viewport {origin:[rect.min.x as f64,rect.min.y as f64],size:[rect.width() as f64,rect.height() as f64]};
        let projected=view.camera.project([4.,3.,0.],viewport).unwrap().unwrap().screen;
        let pointer=Pos2::new(projected[0] as f32,projected[1] as f32);assert!(rect.contains(pointer));
        frame(&mut app,vec![Event::PointerMoved(pointer)]);
        frame(&mut app,vec![Event::PointerButton {pos:pointer,button:PointerButton::Primary,pressed:true,modifiers:Modifiers::NONE}]);
        frame(&mut app,vec![Event::PointerButton {pos:pointer,button:PointerButton::Primary,pressed:false,modifiers:Modifiers::NONE}]);
        frame(&mut app,vec![]);
        assert!(app.controller.prompt().to_ascii_lowercase().contains("height"));
        let hover=app.hover.expect("height pointer");assert!(hover[2].abs()<1e-4,"stationary cursor produced {hover:?}");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.solid_preview(hover).len(),4,"the base stays visible at zero height");
        for height in [5.,-5.] {
            let projected=view.camera.project([4.,3.,height],viewport).unwrap().unwrap().screen;
            let pointer=Pos2::new(projected[0] as f32,projected[1] as f32);
            frame(&mut app,vec![Event::PointerMoved(pointer)]);
            let hover=app.hover.expect("height pointer");
            assert!((hover[2]-height).abs()<1e-3,"height {height} produced {hover:?}");
            assert_eq!(app.controller.solid_preview(hover).len(),12);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn box_preview_uses_the_current_pointer_on_the_same_frame() {
        use egui::{Event, Pos2, Rect, Vec2};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        let mut app=MatrixApp::new(Document::new("box pointer preview")).unwrap();
        app.controller.submit("Box").unwrap();app.controller.accept_world_point([0.,0.,0.]).unwrap();
        let ctx=egui::Context::default();
        let frame=|app:&mut MatrixApp, events:Vec<Event>| {
            let mut raw=egui::RawInput {screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(1280.,720.))),events,..Default::default()};
            eframe::App::raw_input_hook(app,&ctx,&mut raw);
            let mut output=ctx.run_ui(raw,|ui|app.show(ui));
            output.textures_delta.clear();
            output
        };
        for _ in 0..4 {frame(&mut app,vec![]);}
        for id in [ViewId::LookingDown,ViewId::Perspective] {
        let rect=app.view_rects[&id];
        let pointer=rect.center()+Vec2::new(55.,35.);
        let output=frame(&mut app,vec![Event::PointerMoved(pointer)]);
        assert_eq!(app.hover_view,Some(id));
        let point=app.hover.expect("current hover point");
        let wire=app.controller.solid_preview(point);assert_eq!(wire.len(),4);
        let view=&app.controller.workspace.views[&id];
        let viewport=Viewport {origin:[rect.min.x as f64,rect.min.y as f64],size:[rect.width() as f64,rect.height() as f64]};
        for [a,b] in wire {
            let a=view.camera.project(a,viewport).unwrap().unwrap().screen;
            let b=view.camera.project(b,viewport).unwrap().unwrap().screen;
            let expected=[Pos2::new(a[0] as f32,a[1] as f32),Pos2::new(b[0] as f32,b[1] as f32)];
            assert!(output.shapes.iter().any(|s| match &s.shape {
                egui::Shape::LineSegment {points,stroke} => *points == expected && stroke.color == egui::Color32::from_rgb(0,130,85),
                _ => false,
            }),"the current pointer's preview edge must appear in the same frame");
        }
        }
    }

    #[test]
    fn ending_duplicate_without_copy_shows_no_change_in_command_history() {
        let mut app = MatrixApp::new(Document::new("no-change feedback")).unwrap();
        app.controller.submit("Line").unwrap();
        app.controller.submit("0,0,0").unwrap();
        app.controller.submit("10,0,0").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Duplicate").unwrap();
        app.transcript.set_draft("");
        app.submit_draft();
        assert_eq!(app.status.message(), "Duplicate: no changes");
        assert!(app.transcript.history().contains("Duplicate: no changes"));
        assert!(!app.controller.is_running());
    }

    #[test]
    fn typed_command_offers_available_suggestions_and_enter_completes() {
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("suggestions")).unwrap();
        let frame = |app: &mut MatrixApp, events: Vec<egui::Event>| {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(1280., 720.),
                    )),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            output.textures_delta.clear();
        };
        app.focus_cmd = true;
        app.transcript.set_draft("Bo");
        frame(&mut app, Vec::new());
        frame(&mut app, Vec::new());
        assert!(
            app.cmd_suggestions.iter().any(|name| name == "Box"),
            "{:?}",
            app.cmd_suggestions
        );
        assert!(
            app.cmd_suggestions.iter().all(|name| app
                .features
                .iter()
                .any(|feature| feature.command.as_deref() == Some(name))),
            "only catalogued commands may be suggested: {:?}",
            app.cmd_suggestions
        );
        assert_eq!(app.cmd_suggestions.first().map(String::as_str), Some("Box"));
        frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.transcript.draft(), "Box ");
        app.focus_cmd = true;
        app.transcript.set_draft("Box");
        frame(&mut app, Vec::new());
        frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(
            app.transcript.history().contains("Box"),
            "an exact command still submits rather than completing itself: {:?}",
            app.transcript.history()
        );
    }

    #[test]
    fn gumball_settings_persist_and_reload() {
        use crate::session::GumballAlignment;
        let directory =
            std::env::temp_dir().join(format!("om9-gumball-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("gumball.json");
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("persist")).unwrap();
        app.load_gumball_settings(&path).unwrap();
        app.controller.submit("Gumball Snappy").unwrap();
        app.controller.submit("Gumball DragStrength=50").unwrap();
        app.controller.submit("Gumball Object").unwrap();
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
        assert!(path.exists(), "a settings change must be written");

        let mut restored = MatrixApp::new(Document::new("restored")).unwrap();
        restored.load_gumball_settings(&path).unwrap();
        assert!(restored.controller.gumball_snappy());
        assert!((restored.controller.gumball_drag_strength() - 0.5).abs() < 1e-9);
        assert_eq!(
            restored.controller.gumball_alignment(),
            GumballAlignment::Object
        );
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn gumball_settings_window_renders_and_stays_open() {
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("gumball settings")).unwrap();
        app.gumball_settings_open = true;
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
        assert!(app.gumball_settings_open);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn export_selected_menu_and_cmd_request_the_stl_dialog() {
        for menu in [true, false] {
            let mut app = MatrixApp::new(Document::with_template_tolerance("stl request")).unwrap();
            app.controller.submit("Box 10 10 10").unwrap();
            let id = *app.controller.document().entities.keys().next().unwrap();
            app.controller.select_objects(&[id]).unwrap();
            if menu {
                app.invoke("OM9-FILE-001", crate::session::InvocationOrigin::Menu);
            } else {
                app.transcript.set_draft("ExportSTL");
                app.submit_draft();
            }
            assert!(app.stl_profile_draft.is_some(), "menu={menu}");
            assert!(!app.stl_request, "picker must wait for quality confirmation");
            assert_eq!(app.status.message(), "Choose STL export quality");
            assert!(!app.controller.is_running());
        }
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_quality_dialog_rejects_invalid_value_accepts_selected_profile_and_cancels() {
        fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
            let mut output = ctx.run_ui(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900., 600.))),
                focused: true, events, ..Default::default()
            }, |ui| app.stl_profile_ui(ui.ctx()));
            output.textures_delta.clear();
        }
        fn click(app: &mut MatrixApp, ctx: &egui::Context, pos: egui::Pos2) {
            for pressed in [true, false] {
                frame(app, ctx, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
                    pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
                }]);
            }
        }
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::with_template_tolerance("stl profile")).unwrap();
        app.controller.submit("Box 10 10 10").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        let before = app.controller.coordinator().snapshot();
        app.invoke("OM9-FILE-001", crate::session::InvocationOrigin::Menu);
        assert_eq!(app.stl_profile_draft, Some(0.001));
        app.stl_profile_draft = Some(0.0);
        for _ in 0..3 { frame(&mut app, &ctx, vec![]); }
        let rect = ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("stl-profile-accept-rect"))).unwrap();
        click(&mut app, &ctx, rect.center());
        assert!(!app.stl_request);
        app.stl_profile_draft = Some(0.02);
        app.stl_angle_draft = 0.0;
        frame(&mut app, &ctx, vec![]);
        click(&mut app, &ctx, rect.center());
        assert!(!app.stl_request);
        app.stl_angle_draft = 0.15;
        frame(&mut app, &ctx, vec![]);
        click(&mut app, &ctx, rect.center());
        assert!(app.stl_request);
        assert_eq!(app.stl_selected_profile.unwrap().linear_tolerance_mm, 0.02);
        assert_eq!(app.stl_selected_profile.unwrap().angular_tolerance_rad, 0.15);
        assert_eq!(app.controller.coordinator().snapshot(), before);

        app.cancel_stl_export_options();
        app.begin_stl_export_options();
        frame(&mut app, &ctx, vec![egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }]);
        assert!(app.stl_profile_draft.is_none());
        assert!(!app.stl_request);
        assert!(app.stl_capture.is_none());
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }

    #[test]
    fn stl_capture_survives_edits_but_not_reopening_the_same_document() {
        use crate::file_lifecycle::{FileKind, FileOutcome};
        use om9_core::{geometry::{Geometry, Mesh}, model::Entity};
        let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = Document::with_template_tolerance("same document reopen");
        let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let path = std::env::temp_dir().join(format!("om9-stl-same-open-{}.om9", uuid::Uuid::new_v4()));
        assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
        let token = app.controller.file_operation().unwrap().token();
        assert_eq!(app.controller.accept_file_path(token, &path).unwrap(), FileOutcome::Saved);
        app.controller.select_objects(&[id]).unwrap();
        app.begin_stl_export_options();
        assert!(app.stl_capture.is_some());
        let direct_capture = app.controller.capture_stl_selected().unwrap();
        let detached_capture = app.controller.capture_stl_selected().unwrap();
        let doc_id = app.controller.document().id;
        let owner = app.controller.document().owner_session;
        let ctx = egui::Context::default();
        let frame = |app: &mut MatrixApp| {
            let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900., 600.))), ..Default::default() }, |ui| app.show(ui));
            output.textures_delta.clear();
        };
        app.controller.submit("Point").unwrap();
        app.controller.submit("2,3,4").unwrap();
        frame(&mut app);
        assert!(app.stl_capture.is_some(), "ordinary edits must retain the captured export");
        app.controller.undo().unwrap();
        frame(&mut app);
        assert!(app.stl_capture.is_some(), "undo must retain the captured export");
        app.controller.redo().unwrap();
        frame(&mut app);
        assert!(app.stl_capture.is_some(), "redo must retain the captured export");
        app.controller.undo().unwrap();
        assert_eq!(app.controller.begin_file_operation(FileKind::Open).unwrap(), FileOutcome::Pending);
        let token = app.controller.file_operation().unwrap().token();
        assert_eq!(app.controller.accept_file_path(token, &path).unwrap(), FileOutcome::Replaced);
        assert_eq!(app.controller.document().id, doc_id);
        assert_ne!(app.controller.document().owner_session, owner);
        let destination = path.with_extension("stl");
        std::fs::write(&destination, b"previous export").unwrap();
        let result = app.controller.export_captured_stl(direct_capture, &destination,
            crate::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 });
        assert!(matches!(result, Err(om9_core::Error::Stale("STL export document session"))));
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous export");
        let result = detached_capture.export_with_profile(&destination,
            crate::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 });
        assert!(matches!(result, Err(om9_core::Error::Canceled)));
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous export");
        std::fs::remove_file(destination).unwrap();
        frame(&mut app);
        assert!(app.stl_capture.is_none());
        assert!(app.stl_profile_draft.is_none());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_menu_captures_selection_at_invocation_and_new_cancels_it() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("capture at invocation")).unwrap();
        app.controller.submit("Box 10 10 10").unwrap();
        let first = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[first]).unwrap();
        app.invoke("OM9-FILE-001", crate::session::InvocationOrigin::Menu);
        let epoch = app.stl_capture.as_ref().unwrap().source_epoch();
        app.controller.submit("Box 20 20 20").unwrap();
        let second = *app.controller.document().entities.keys().find(|id| **id != first).unwrap();
        app.controller.select_objects(&[second]).unwrap();
        app.controller.delete_objects(&[first]).unwrap();
        let capture = app.stl_capture.take().unwrap();
        let path = std::env::temp_dir().join(format!("om9-ui-capture-{}.stl", uuid::Uuid::new_v4()));
        let receipt = capture.export_with_profile(&path, crate::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }).unwrap();
        assert_eq!(receipt.source_epoch, epoch);
        assert_eq!(receipt.triangles, 12);
        let bytes = std::fs::read(&path).unwrap();
        let max_x = bytes[84..].chunks_exact(50).flat_map(|face| [12, 24, 36].map(|offset| f32::from_le_bytes(face[offset..offset + 4].try_into().unwrap()))).fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(max_x, 10.0);
        std::fs::remove_file(path).unwrap();
        app.cancel_stl_export_options();
        app.begin_stl_export_options();
        assert!(app.stl_capture.is_some());
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900., 600.))), ..Default::default() }, |ui| app.show(ui));
        output.textures_delta.clear();
        app.invoke("OM9-FILE-002", crate::session::InvocationOrigin::Menu);
        assert!(app.stl_capture.is_none());
        assert!(app.stl_profile_draft.is_none());
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_quality_escape_through_full_ui_preserves_selection() {
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::with_template_tolerance("stl cancel")).unwrap();
        app.controller.submit("Box 10 10 10").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        let before = app.controller.coordinator().snapshot();
        app.begin_stl_export_options();
        let mut output = ctx.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900., 600.))),
            focused: true,
            events: vec![egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }],
            ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
        assert!(app.stl_profile_draft.is_none());
        assert!(!app.stl_request);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert!(app.controller.selected().contains(&id));
    }

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
#[path = "ui/gumball_gpu_tests.rs"]
mod gumball_gpu_tests;

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

/// Window/taskbar icon authored in the repository (`launcher/OpenMatrix9.ico`).
fn app_icon() -> egui::IconData {
    const ICON: &[u8] = include_bytes!("../../../launcher/OpenMatrix9.ico");
    let png = ico_largest_png(ICON)
        .and_then(|bytes| image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok());
    match png {
        Some(image) => {
            let image = image.to_rgba8();
            let (width, height) = image.dimensions();
            egui::IconData {
                rgba: image.into_raw(),
                width,
                height,
            }
        }
        None => egui::IconData::default(),
    }
}

/// The authored ICO stores every size as PNG. Pick the largest and decode it directly:
/// the image crate's ICO wrapper refuses RGB PNG entries even though the pixels are valid.
fn ico_largest_png(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.len() < 6 || bytes[0..4] != [0, 0, 1, 0] {
        return None;
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    let mut best: Option<(u32, &[u8])> = None;
    for index in 0..count {
        let entry = 6 + index * 16;
        if entry + 16 > bytes.len() {
            break;
        }
        let width = if bytes[entry] == 0 {
            256
        } else {
            u32::from(bytes[entry])
        };
        let height = if bytes[entry + 1] == 0 {
            256
        } else {
            u32::from(bytes[entry + 1])
        };
        let size = u32::from_le_bytes(bytes[entry + 8..entry + 12].try_into().ok()?) as usize;
        let offset = u32::from_le_bytes(bytes[entry + 12..entry + 16].try_into().ok()?) as usize;
        let end = offset.checked_add(size)?;
        let data = bytes.get(offset..end)?;
        if data.starts_with(b"\x89PNG\r\n\x1a\n")
            && best.is_none_or(|(area, _)| width * height > area)
        {
            best = Some((width * height, data));
        }
    }
    best.map(|(_, data)| data)
}

#[cfg(test)]
mod icon_tests {
    #[test]
    fn window_icon_is_decoded_from_the_authored_launcher_asset() {
        let icon = super::app_icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
        assert!(
            icon.rgba
                .chunks(4)
                .any(|pixel| pixel.get(3).is_some_and(|alpha| *alpha != 0))
        );
    }
}

#[cfg(test)]
mod delete_key_tests {
    use super::*;
    use om9_core::geometry::{Curve, Geometry};
    use om9_core::model::Entity;

    #[test]
    fn delete_from_raw_input_removes_the_selection() {
        let mut document = Document::new("delete key");
        let line = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        let id = line.id;
        document.entities.insert(id, line);
        let mut app = MatrixApp::new(document).expect("app");
        app.controller.select_objects(&[id]).expect("selection");
        let context = egui::Context::default();
        let mut raw = egui::RawInput {
            events: vec![egui::Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        app.delete_selection_from_input(&context, &mut raw);
        assert!(
            app.controller.document().entities.is_empty(),
            "Delete removes the object"
        );
        assert!(raw.events.is_empty(), "the handled Delete is consumed");
    }

}
