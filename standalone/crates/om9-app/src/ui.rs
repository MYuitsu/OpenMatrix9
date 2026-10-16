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
    grid::Grid,
    scene::{PickIntent, RenderBuffers},
    workspace::ViewId,
};
mod chrome;
mod command;
mod layer_audit;
mod layer_dialog_audit;
mod layers;
pub use layers::LayerControl;
mod file_audit;
mod file_success_audit;
mod files;
mod layer_dialogs;
mod startup_audit;
pub use files::FileControl;
pub use layer_dialogs::LayerDialogControl;
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
}
/// Adapter for the `CommandInputHost` seam toward `OM9-L2-01-02`. The CMD frame
/// (`MatrixApp::transcript` plus `command_ui`) is built into every frame, so it
/// is always attached. This says nothing about command dispatch or the Undo
/// journal, which that module does not provide yet.
struct AttachedCmdFrame;
impl CommandInputHost for AttachedCmdFrame {
    fn command_input_ready(&self) -> bool {
        true
    }
}
pub struct MatrixApp {
    pub controller: Controller,
    pub transcript: Transcript,
    features: Vec<Feature>,
    focus_cmd: bool,
    cmd_rect: Option<Rect>,
    cmd_history_rect: Option<Rect>,
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
    file_ime_composing: bool,
    file_f6_rects: BTreeMap<crate::file_lifecycle::FileKind, Rect>,
    /// Window title owned by this document's path/name and dirty marker.
    pub window_title: String,
    current_ui_context: Option<egui::Context>,
    hover: Option<Point3>,
    pub audit: Option<UiAudit>,
    passes: u64,
    adapter: String,
    pub f6_open: bool,
    pub f6_pinned: bool,
    pub sidebar_rect: Option<Rect>,
    pub toolbar_rect: Option<Rect>,
    icons: crate::icons::Icons,
    main_category: usize,
    toolbar_tab: usize,
    style_ready: bool,
    /// Live rectangles of the Matrix menu title bars, keyed by section key.
    pub section_rects: BTreeMap<&'static str, Rect>,
    section_drag: Option<SectionId>,
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
    pub status: StatusSurface,
    pub regions: Regions,
    /// Last validated workbench region plan (L3-002). Kept when a later frame
    /// cannot produce one, so the shell never falls back to an unchecked layout.
    pub region_plan: Option<RegionPlan>,
    /// Why the current frame has no fresh plan; `None` when the plan is current.
    pub region_error: Option<RegionError>,
    /// Status text this module raised, so only it clears it again.
    region_status: Option<String>,
    pub right_dock_rect: Option<Rect>,
    pub builder_slot_rect: Option<Rect>,
    pub status_rect: Option<Rect>,
    pub dock_panel_rects: BTreeMap<&'static str, Rect>,
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
    viewport_views: Option<BTreeMap<ViewId, om9_scene::workspace::ViewState>>,
    viewport_popup_ready_at: Option<f64>,
    viewport_replay: Vec<serde_json::Value>,
    viewport_idle_step: Option<usize>,
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
            viewport_views: None,
            viewport_popup_ready_at: None,
            viewport_replay: Vec::new(),
            viewport_idle_step: None,
        }
    }
}
impl MatrixApp {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        Self::from_controller(Controller::from_file(path, Limits::default())?)
    }
    pub fn new(document: Document) -> Result<Self> {
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
            controller,
            transcript: Transcript::new(),
            features,
            focus_cmd: true,
            cmd_rect: None,
            cmd_history_rect: None,
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
            file_ime_composing: false,
            file_f6_rects: BTreeMap::new(),
            window_title: String::new(),
            current_ui_context: None,
            hover: None,
            audit: None,
            passes: 0,
            adapter: "headless-egui".into(),
            f6_open: false,
            f6_pinned: false,
            sidebar_rect: None,
            toolbar_rect: None,
            icons: crate::icons::Icons::new(),
            main_category: 0,
            toolbar_tab: 0,
            style_ready: false,
            section_rects: BTreeMap::new(),
            section_drag: None,
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
            right_dock_rect: None,
            builder_slot_rect: None,
            status_rect: None,
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
    /// Reorder a menu section inside its own column (documented title-bar drag).
    pub fn move_menu_section(&mut self, id: SectionId, delta: i32) -> bool {
        match self.dock.move_section(id, delta) {
            Ok((section, index)) => {
                self.dock_dirty = true;
                self.status.info(&format!(
                    "{} moved to menu position {}",
                    section.title(),
                    index + 1
                ));
                true
            }
            Err(error) => {
                self.status.warning(&format!("{error}"));
                false
            }
        }
    }
    /// Dock a menu to the left column or to the far-right mirror column.
    pub fn dock_menu_section(&mut self, id: SectionId, side: DockSide) -> bool {
        match self.dock.dock_section(id, side) {
            Ok(()) => {
                self.dock_dirty = true;
                self.status.info(&format!(
                    "{} docked {}",
                    id.title(),
                    match side {
                        DockSide::Right => "to the right menu column",
                        _ => "in the left menu column",
                    }
                ));
                true
            }
            Err(error) => {
                self.status.warning(&format!("{error}"));
                false
            }
        }
    }
    /// Float a menu over the model at an owned rectangle.
    pub fn float_menu_section(&mut self, id: SectionId, rect: FloatRect) -> bool {
        if !matches!(self.dock.section(id).dock, DockSide::Floating) {
            self.dock.float_section(id, rect);
            self.dock_dirty = true;
            self.status
                .info(&format!("{} floated over the model", id.title()));
            true
        } else {
            false
        }
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
        self.status.info("Menus reset to the Matrix default");
    }
    /// View ▸ Matrix Menus ▸ Reload Menus: load the configuration the program
    /// opens with, falling back to the factory default.
    pub fn reload_menus(&mut self) {
        if self.dock_path.is_none() {
            self.dock.reset_to_default();
            self.dock_dirty = false;
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
        let prompt = self.controller.prompt();
        self.transcript.append_history(&format!("{prompt}{draft}"));
        let result = self.controller.submit_effect(&draft);
        if result.is_ok() {
            self.transcript.record_command(&draft);
            self.transcript.set_draft("");
        }
        match result {
            Ok(crate::session::DispatchOutcome::File(outcome)) => {
                let ctx = self.current_ui_context.clone().expect("UI context");
                let _ = self.file_feedback(&ctx, Ok(outcome));
            }
            other => self.feedback(other.map(|_| ())),
        }
    }
    pub fn command_input_rect(&self) -> Option<Rect> {
        self.cmd_rect
    }
    pub fn command_history_rect(&self) -> Option<Rect> {
        self.cmd_history_rect
    }
    pub fn selected_objects(&self) -> &std::collections::BTreeSet<om9_core::identity::ObjectId> {
        self.controller.selected()
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
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.passes += 1;
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
            if let Some(token) = self
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
        ui.set_style(ctx.style_of(egui::Theme::Light));
        let viewport = ui.max_rect().size();
        self.refresh_region_plan([viewport.x, viewport.y]);
        // Floating menus are only pulled back inside a window the region check
        // accepts; a window below the minimum must not rewrite stored layout.
        if self.region_error.is_none() {
            self.sync_dock_window([viewport.x, viewport.y]);
        }
        self.regions = self.dock.resolve([viewport.x, viewport.y]);
        let regions = self.regions;
        self.dock_panel_rects.clear();
        self.section_rects.clear();
        self.builder_slot_rect = None;
        egui::Panel::top("menu")
            .exact_size(21.)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(232, 232, 230))
                    .inner_margin(2),
            )
            .show(ui, |ui| {
                if self.viewport_menu_owns_frame {
                    ui.disable();
                }
                self.menu_ui(ui);
            });
        egui::Panel::left("matrix-sidebar")
            .default_size(regions.left_rail)
            .min_size(dock_layout::LEFT_RAIL_MIN)
            .max_size(dock_layout::LEFT_RAIL_MAX)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_gray(82))
                    .inner_margin(2),
            )
            .show(ui, |ui| {
                self.sidebar_rect = Some(ui.max_rect());
                self.sync_panel_size(PanelId::LeftRail, ui.max_rect().width());
                if self.viewport_menu_owns_frame {
                    ui.disable();
                }
                self.sidebar_ui(ui);
            });
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
                    .fill(Color32::from_rgb(203, 214, 213))
                    .inner_margin(1),
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
            egui::Panel::right("matrix-right-dock")
                .default_size(regions.right_dock)
                .min_size(dock_layout::RIGHT_DOCK_MIN)
                .max_size(dock_layout::RIGHT_DOCK_MAX)
                .resizable(true)
                .frame(
                    egui::Frame::new()
                        .fill(Color32::from_gray(70))
                        .inner_margin(2),
                )
                .show(ui, |ui| {
                    self.right_dock_rect = Some(ui.max_rect());
                    self.sync_panel_size(PanelId::RightDock, ui.max_rect().width());
                    if self.viewport_menu_owns_frame {
                        ui.disable();
                    }
                    self.right_dock_ui(ui);
                });
        } else {
            self.right_dock_rect = None;
        }
        // Global status surface (L3-007): always visible, never owns Escape.
        egui::Panel::bottom("matrix-status")
            .exact_size(dock_layout::STATUS_BAR_HEIGHT)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_gray(96))
                    .inner_margin(2),
            )
            .show(ui, |ui| {
                self.status_rect = Some(ui.max_rect());
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
        if !ctx.input(|input| input.pointer.any_down()) {
            if let Some(pointer) = ctx.input(|input| input.pointer.interact_pos()) {
                self.settle_section_drag(pointer);
            }
            self.section_drag = None;
        }
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
    fn viewport_ui(&mut self, ui: &mut egui::Ui, id: ViewId, rect: Rect) {
        let Some(view) = self.controller.workspace.views.get(&id).cloned() else {
            return;
        };
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, 0, Color32::BLACK);
        if let Ok(grid) = Grid::new(view.cplane) {
            for line in grid.lines {
                let start = if line.axis {
                    paint_line(
                        &painter,
                        &view.camera,
                        viewport,
                        line.start,
                        view.cplane.origin,
                        [97, 97, 97, 255],
                        1.,
                    );
                    view.cplane.origin
                } else {
                    line.start
                };
                paint_line(
                    &painter,
                    &view.camera,
                    viewport,
                    start,
                    line.end,
                    line.color,
                    1.,
                );
            }
        }
        if let Some(scene) = self.controller.scene() {
            for object in &scene.objects {
                if !object.visible {
                    continue;
                }
                let Ok(transform) = object.display_transform() else {
                    continue;
                };
                let color = if self.controller.selected().contains(&object.root) {
                    [255, 220, 0, 255]
                } else {
                    object.color
                };
                match object.buffers.as_ref() {
                    RenderBuffers::Points { points, .. } => {
                        for point in points {
                            if let Ok(point) = transform.point(*point)
                                && let Ok(Some(p)) = view.camera.project(point, viewport)
                            {
                                painter.circle_filled(
                                    egui::pos2(p.screen[0] as f32, p.screen[1] as f32),
                                    3.,
                                    rgba(color),
                                );
                            }
                        }
                    }
                    RenderBuffers::Curve { points, .. } => {
                        for segment in points.windows(2) {
                            if let (Ok(a), Ok(b)) =
                                (transform.point(segment[0]), transform.point(segment[1]))
                            {
                                paint_line(&painter, &view.camera, viewport, a, b, color, 1.3);
                            }
                        }
                    }
                    RenderBuffers::Mesh {
                        vertices, edges, ..
                    } => {
                        for edge in edges {
                            if let (Ok(a), Ok(b)) = (
                                transform.point(vertices[edge[0] as usize]),
                                transform.point(vertices[edge[1] as usize]),
                            ) {
                                paint_line(&painter, &view.camera, viewport, a, b, color, 1.);
                            }
                        }
                    }
                }
            }
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
        paint_view_marker(&painter, rect, id);
        let canvas = Rect::from_min_max(egui::pos2(rect.min.x, rect.min.y + 18.), rect.max);
        let response = ui.interact(canvas, ui.id().with(("canvas", id.title())), Sense::click());
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
                    let picked = self.controller.scene().and_then(|scene| {
                        scene
                            .pick(
                                [pos.x as f64, pos.y as f64],
                                &view.camera,
                                viewport,
                                6.,
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
                    let result = self
                        .controller
                        .select_root(picked, ui.input(|i| i.modifiers.shift));
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
    fn audit_receipt(&mut self, ctx: &egui::Context) {
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
            // order, menus docked to the right, the reserved Builder area and the
            // always-visible status surface.
            let left_sections: Vec<&str> = self
                .dock
                .left_sections()
                .into_iter()
                .map(SectionId::key)
                .collect();
            let right_sections: Vec<&str> = self
                .dock
                .right_sections()
                .into_iter()
                .map(SectionId::key)
                .collect();
            let floating_sections: Vec<&str> = self
                .dock
                .floating_sections()
                .into_iter()
                .map(SectionId::key)
                .collect();
            receipt["dock"] = serde_json::json!({
                "left_sections": left_sections,
                "right_sections": right_sections,
                "floating_sections": floating_sections,
                "right_dock_visible": self.regions.right_dock_visible,
                "builder_slot_visible": self.builder_slot_rect.is_some()
                    || self
                        .dock
                        .left_sections()
                        .contains(&SectionId::BuilderSlot),
                "left_rail_px": self.regions.left_rail,
                "command_line_px": self.regions.command_line,
                "status_bar_px": self.regions.status_bar,
                "status_kind": self.status.kind().label(),
                "status_message": self.status.message(),
                "diagnostics": self.dock_diagnostics,
            });
            // Workbench docking regions (OM9-L3-01-01-002): the validated plan
            // for the audited window, or the explicit reason there is none.
            receipt["regions"] = serde_json::json!({
                "window": self.dock_window,
                "plan": self.region_plan,
                "error": self.region_error.as_ref().map(ToString::to_string),
                "sidebar_px": self.sidebar_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "right_dock_px": self.right_dock_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
                "command_input_px": self.cmd_rect.map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
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
fn rgba(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
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
            Stroke::new(width, rgba(color)),
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
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
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
        // Explicit opt-in own-app event replay through the normal egui input
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
    // Audit-only: show the optional right dock without touching the stored
    // layout (nothing is saved unless the user runs Set as Default).
    if app.audit.is_some() && std::env::var("OM9_UI_AUDIT_RIGHT_DOCK").is_ok_and(|v| v == "1") {
        app.dock_menu_section(SectionId::Layers, DockSide::Right);
    }
    let title = config.window_title().to_owned();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(config.window)
            .with_maximized(config.maximized && app.audit.is_none())
            .with_min_inner_size(config.min_window)
            .with_title(&title)
            .with_icon(egui::IconData::default()),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            app.configure_style(&cc.egui_ctx);
            // L3-001: an unavailable hardware adapter is explained on the status
            // surface instead of silently rendering with a fallback identity.
            let reported = cc
                .wgpu_render_state
                .as_ref()
                .map(|render| format!("{:?}", render.adapter.get_info()));
            let (adapter, note) = crate::startup::adapter_note(reported.as_deref());
            app.adapter = adapter;
            if let Some(note) = note {
                app.status.warning(&note);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| Error::Io(e.to_string()))
}
#[cfg(test)]
mod tests {
    use super::parse_window_size;

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
