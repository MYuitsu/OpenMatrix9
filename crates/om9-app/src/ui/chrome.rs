// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compact Matrix workspace chrome. Buttons keep the shared command dispatcher;
//! decorative icon identity is separate from accepted operation availability.
use super::*;
use egui::{Align2, FontId, RichText};

const GREEN: Color32 = Color32::from_rgb(130, 180, 140);
pub(super) const PANEL_BG: Color32 = super::command::COMMAND_BG;
const PANEL_ALT: Color32 = Color32::from_rgb(0, 81, 38);
const PANEL_EDGE: Color32 = Color32::from_rgb(35, 107, 72);
pub(super) const PANEL_INK: Color32 = Color32::from_rgb(255, 244, 214);
pub(super) const PANEL_MUTED: Color32 = Color32::from_rgb(205, 226, 210);
const PANEL_TITLE: Color32 = Color32::from_rgb(253, 216, 128);
const ACCENT: Color32 = Color32::from_rgb(45, 212, 191);
const SELECTED_CARD: Color32 = Color32::from_rgb(18, 58, 42);
const CATEGORY_COLORS: [Color32; 12] = [
    Color32::from_rgb(201, 173, 46),  // Curve: sampled from Matrix8 p14
    Color32::from_rgb(71, 186, 143),  // Surface
    Color32::from_rgb(133, 88, 164),  // Solid
    Color32::from_rgb(239, 71, 35),   // Transform
    Color32::from_rgb(123, 194, 70),  // SubD/TSpline
    Color32::from_rgb(211, 211, 45),  // Art
    Color32::from_rgb(212, 34, 46),   // Builder
    Color32::from_rgb(107, 102, 174), // Tools
    Color32::from_rgb(74, 130, 195),  // Gems
    Color32::from_rgb(166, 34, 142),  // Setting
    Color32::from_rgb(244, 116, 33),  // Cutters
    Color32::from_rgb(31, 185, 184),  // Render
];
const TOP_MENUS: [&str; 5] = ["File", "View", "Utilities", "Measure", "Custom"];
const CATEGORIES: [&str; 12] = [
    "Curve",
    "Surface",
    "Solid",
    "Transform",
    "SubD",
    "Art",
    "Builder",
    "Tools",
    "Gems",
    "Setting",
    "Cutters",
    "Render",
];
const TABS: [&str; 21] = [
    "Standard",
    "CPlanes",
    "Set View",
    "Display",
    "Select",
    "Viewport Layout",
    "Visibility",
    "Transform",
    "Curve Tools",
    "Surface Tools",
    "Solid Tools",
    "Mesh Tools",
    "Render Tools",
    "Drafting",
    "SubD",
    "New in V5",
    "Emboss",
    "Sculpt",
    "Tools",
    "Settings",
    "Top 11",
];

fn apply_dock_theme(ui: &mut egui::Ui) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(PANEL_INK);
    visuals.weak_text_color = Some(PANEL_MUTED);
    visuals.disabled_alpha = 0.78;
    visuals.panel_fill = PANEL_BG;
    visuals.hyperlink_color = PANEL_TITLE;
    visuals.selection.bg_fill = PANEL_EDGE;
    visuals.selection.stroke = Stroke::new(1., PANEL_INK);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.fg_stroke = Stroke::new(1., PANEL_INK);
    }
    visuals.widgets.inactive.bg_fill = PANEL_ALT;
    visuals.widgets.inactive.weak_bg_fill = PANEL_ALT;
    visuals.widgets.hovered.bg_fill = PANEL_EDGE;
    visuals.widgets.hovered.weak_bg_fill = PANEL_EDGE;
    visuals.widgets.active.bg_fill = PANEL_EDGE;
    visuals.widgets.active.weak_bg_fill = PANEL_EDGE;
}

impl MatrixApp {
    pub(super) fn configure_style(&mut self, ctx: &egui::Context) {
        if self.style_ready {
            return;
        }
        self.style_ready = true;
        super::typography::configure_fonts(ctx);
        let mut style = (*ctx.style_of(egui::Theme::Light)).clone();
        style.visuals = egui::Visuals::light();
        style.spacing.item_spacing = Vec2::new(2., 2.);
        style.spacing.button_padding = Vec2::new(3., 1.);
        style.spacing.interact_size = Vec2::new(18., 17.);
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(11.));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(11.));
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(10.));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, FontId::monospace(12.));
        style.visuals.widgets.inactive.corner_radius = 4.into();
        style.visuals.widgets.hovered.corner_radius = 4.into();
        style.visuals.widgets.active.corner_radius = 4.into();
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(238, 243, 248);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(220, 235, 250);
        style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(199, 232, 227);
        style.visuals.text_options.font_hinting = true;
        style.visuals.text_options.subpixel_binning = true;
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.selection.stroke = Stroke::new(1., Color32::from_rgb(15, 23, 42));
        style.visuals.window_corner_radius = 8.into();
        ctx.set_theme(egui::ThemePreference::Light);
        ctx.set_style_of(egui::Theme::Light, style);
    }
    pub(super) fn menu_ui(&mut self, ui: &mut egui::Ui) {
        self.menu_rects.clear();
        egui::MenuBar::new().ui(ui, |ui| {
            for name in [
                "File",
                "Edit",
                "View",
                "Curve",
                "Surface",
                "Solid",
                "Mesh",
                "Dimension",
                "Transform",
                "Tools",
                "Analyze",
                "Render",
                "Help",
            ] {
                let menu=ui.menu_button(name, |ui| match name {
                    "Dimension"=>{
                        for (label,feature) in ["Horizontal Dimension","Vertical Dimension","Angle Dimension","Rotated Dimension","Aligned Dimension","Diameter Dimension","Radial Dimension","Leader","Text Block","Edit Dimension","Edit Text","Dimension Options","Recenter Dimension Text"].into_iter().zip(crate::session::annotation::FEATURES){
                            if ui.button(label).clicked(){self.invoke(feature,InvocationOrigin::Menu);ui.close();}
                        }
                    }
                    "Analyze" => {
                        for (label,feature) in [("Area",crate::session::measure::AREA),("Volume",crate::session::measure::VOLUME),("Object Summary",crate::session::measure::OBJECT_SUMMARY)] {
                            if ui.button(label).clicked(){self.invoke(feature,InvocationOrigin::Menu);ui.close();}
                        }
                    }
                    "Tools" => {
                        // Super Select (OM9-L3-03-14-006): typed filters, the manual's tool.
                        for (label, key, filter) in [
                            ("Super Select: Gems", "super-gems", crate::superselect::SelectFilter::Gems),
                            ("Super Select: Bad geometry", "super-bad", crate::superselect::SelectFilter::BadGeometry),
                            ("Super Select: Open surfaces", "super-open", crate::superselect::SelectFilter::OpenGeometry),
                        ] {
                            let response = ui.button(label);
                            self.menu_rects.insert(key, response.rect);
                            if response.clicked() {
                                let count = self.controller.apply_superselect(filter);
                                self.status.info(&format!("{label}: {count} selected"));
                                self.focus_cmd = true;
                                ui.close();
                            }
                        }
                        ui.separator();
                        let has_host = self.controller.addon_host().is_some();
                        if ui
                            .add_enabled(has_host, egui::Button::new("Add-ons…"))
                            .on_hover_text("Install, enable and configure OM9 add-ons")
                            .clicked()
                        {
                            self.addons_ui.open = true;
                            ui.close();
                        }
                        if ui
                            .add_enabled(has_host, egui::Button::new("Install Add-on from Disk…"))
                            .on_hover_text("Choose an OM9 add-on .zip (or drag it onto the window)")
                            .clicked()
                        {
                            self.addons_ui.install_request = true;
                            ui.close();
                        }
                        // Four Rail Profile (OM9-TOOLS-004): tag profiles with the
                        // four selected rails so Sweep 4 can resolve them later.
                        if ui.button("Four Rail Profile").clicked() {
                            self.invoke("OM9-TOOLS-004", InvocationOrigin::Menu);
                            ui.close();
                        }
                        if ui.button("Object Health Report").clicked() {
                            let summary = self.controller.health_summary();
                            self.status.info(&summary);
                            ui.close();
                        }
                        if ui.button("Select Bad Objects").clicked() {
                            match self.controller.select_bad_objects() {
                                Ok(count) => self.status.info(&format!("{count} bad objects selected")),
                                Err(error) => self.status.error(&error.to_string()),
                            };
                            ui.close();
                        }
                        if ui.button("Detect Naked Edges").clicked() {
                            match self.controller.select_naked_edges() {
                                Ok(count) => self.status.info(&format!("{count} open objects selected")),
                                Err(error) => self.status.error(&error.to_string()),
                            };
                            ui.close();
                        }
                    }
                    "File" => {
                        for kind in crate::file_lifecycle::FileKind::ALL {
                            let label=if kind==crate::file_lifecycle::FileKind::New {kind.label().to_owned()} else {format!("{}…",kind.label())};
                            let response=ui.button(label).on_hover_text("Native OM9 document (.om9)");
                            self.file_menu_items.insert(kind,response.rect);
                            if response.clicked() {self.invoke(kind.feature_id(),InvocationOrigin::Menu);ui.close();}
                        }
                        for (label, export) in [("Import 3DM…", false), ("Export 3DM…", true)] {
                            let response = ui
                                .add_enabled(
                                    !self.controller.is_running(),
                                    egui::Button::new(label),
                                )
                                .on_hover_text(
                                    "Native Rhino 3DM reader/writer (requires the admitted OM9 exchange executable)",
                                );
                            if response.clicked() {
                                self.three_dm_request = Some(export);
                                ui.close();
                            }
                        }
                        let stl = ui
                            .add_enabled(
                                !self.controller.is_running(),
                                egui::Button::new("Export Selected STL…"),
                            )
                            .on_hover_text(
                                "Selected meshes and editable B-Rep solids, binary STL in millimetres",
                            );
                        if stl.clicked() {
                            self.invoke("OM9-FILE-001", InvocationOrigin::Menu);
                            ui.close();
                        }
                    }
                    "Edit" => {
                        let can_copy = !self.controller.is_running() && !self.controller.selected().is_empty();
                        for (label, action, enabled) in [
                            ("Copy\tCtrl+C", super::clipboard::ClipboardAction::Copy, can_copy),
                            ("Copy as 3DM\tCtrl+Shift+C", super::clipboard::ClipboardAction::Copy3dm, can_copy),
                            ("Paste\tCtrl+V", super::clipboard::ClipboardAction::Paste, !self.controller.is_running()),
                        ] {
                            if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                                self.clipboard_action(action);
                                ui.close();
                            }
                        }
                        ui.separator();
                        if ui
                            .add_enabled(
                                self.controller.coordinator().undo_len() > 0
                                    && !self.controller.is_running(),
                                egui::Button::new("Undo"),
                            )
                            .clicked()
                        {
                            let r = self.controller.undo();
                            self.feedback(r);
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                self.controller.coordinator().redo_len() > 0
                                    && !self.controller.is_running(),
                                egui::Button::new("Redo"),
                            )
                            .clicked()
                        {
                            let r = self.controller.redo();
                            self.feedback(r);
                            ui.close();
                        }
                        // The property editor (OM9-L3-03-14-007): the six editable
                        // fields for the single selected object, applied through the
                        // adapter so each toggle is one transaction.
                        let selected: Vec<_> = self.controller.selected().iter().copied().collect();
                        if selected.len() == 1 {
                            let id = selected[0];
                            let attributes = self.controller.document().entities[&id].attributes.clone();
                            ui.separator();
                            for (key, label, edit) in [
                                (
                                    "prop-visible",
                                    format!("Visible: {}", if attributes.visible { "on" } else { "off" }),
                                    crate::session::PropertyEdit::Visible(!attributes.visible),
                                ),
                                (
                                    "prop-hidden",
                                    format!("Hidden: {}", if attributes.hidden { "on" } else { "off" }),
                                    crate::session::PropertyEdit::Hidden(!attributes.hidden),
                                ),
                                (
                                    "prop-selectable",
                                    format!("Selectable: {}", if attributes.selectable { "on" } else { "off" }),
                                    crate::session::PropertyEdit::Selectable(!attributes.selectable),
                                ),
                                (
                                    "prop-locked",
                                    format!("Locked: {}", if attributes.locked { "on" } else { "off" }),
                                    crate::session::PropertyEdit::Locked(!attributes.locked),
                                ),
                                (
                                    "prop-color-red",
                                    "Colour: red".to_owned(),
                                    crate::session::PropertyEdit::Color(Some([220, 40, 40, 255])),
                                ),
                                (
                                    "prop-color-default",
                                    "Colour: layer".to_owned(),
                                    crate::session::PropertyEdit::Color(None),
                                ),
                            ] {
                                let response = ui.button(label);
                                self.menu_rects.insert(key, response.rect);
                                if response.clicked() {
                                    let outcome = self.edit_selected_property(edit);
                                    self.feedback(outcome);
                                    ui.close();
                                }
                            }
                        }
                    }
                    "Curve" => {
                        egui::ScrollArea::vertical().max_height(620.0).show(ui, |ui| {
                            for binding in crate::curve_menu::BINDINGS {
                                let response = ui.add_enabled(
                                    binding.enabled(),
                                    egui::Button::new(binding.label),
                                );
                                if !binding.enabled() {
                                    response.clone().on_hover_text(binding.disabled_reason);
                                }
                                if response.clicked() {
                                    self.invoke(binding.feature, InvocationOrigin::Menu);
                                    ui.close();
                                }
                            }
                        });
                    }
                    "View" => {
                        for id in [
                            ViewId::Perspective,
                            ViewId::ThroughFinger,
                            ViewId::SideView,
                            ViewId::LookingDown,
                        ] {
                            if ui.button(id.title()).clicked() {
                                self.activate_view(id);
                                ui.close();
                            }
                        }
                        ui.separator();
                        // Documented menu configuration commands.
                        ui.menu_button("Matrix Menus", |ui| {
                            if ui
                                .button("Set as Default")
                                .on_hover_text("Keep this menu configuration for the next start")
                                .clicked()
                            {
                                self.set_menus_as_default();
                                ui.close();
                            }
                            if ui
                                .button("Reset Menus")
                                .on_hover_text("Return to the documented Matrix default mode")
                                .clicked()
                            {
                                let ctx = self.current_ui_context.clone();
                                self.reset_menus(ctx.as_ref());
                                ui.close();
                            }
                            if ui
                                .button("Reload Menus")
                                .on_hover_text("Load the configuration the program opens with")
                                .clicked()
                            {
                                self.reload_menus();
                                ui.close();
                            }
                        });
                        let right_column = !self.dock.right_sections().is_empty();
                        if ui
                            .button(if right_column {
                                "Close Right Menu Column"
                            } else {
                                "Open Right Menu Column"
                            })
                            .on_hover_text("Mirror column for menus docked to the right")
                            .clicked()
                        {
                            self.toggle_right_dock();
                            ui.close();
                        }
                    }
                    "Help" => {
                        if ui.button("Command help").clicked() {
                            self.transcript.append_history(
                                "Commands: Line, Point, Undo, Redo. Enter accepts input; Esc cancels the draft. F6 opens command search.",
                            );
                            self.focus_cmd = true;
                            ui.close();
                        }
                        if ui.button("About OpenMatrix9").clicked() {
                            self.transcript.append_history(concat!("OpenMatrix9 ", env!("CARGO_PKG_VERSION"), " — standalone modeling workspace."));
                            self.focus_cmd = true;
                            ui.close();
                        }
                    }
                    _ => {
                        egui::ScrollArea::vertical()
                            .max_height(480.)
                            .show(ui, |ui| {
                                let items: Vec<_> = self
                                    .features
                                    .iter()
                                    .filter(|f| feature_category(f) == name)
                                    .cloned()
                                    .collect();
                                for f in items {
                                    let available = self.controller.is_available(&f.id);
                                    if ui
                                        .add_enabled(available, egui::Button::new(&f.name))
                                        .on_hover_text(if available {
                                            f.id.clone()
                                        } else {
                                            format!("{}: operation unavailable", f.id)
                                        })
                                        .clicked()
                                    {
                                        self.invoke(&f.id, InvocationOrigin::Menu);
                                        ui.close();
                                    }
                                }
                            });
                    }
                });
                if name=="File" {self.file_menu_rect=Some(menu.response.rect);}
                self.menu_rects.insert(name, menu.response.rect);
            }
        });
    }
    fn activate_view(&mut self, id: ViewId) {
        self.controller.workspace.active = id;
        if self.controller.workspace.maximized.is_some() {
            self.controller.workspace.maximized = Some(id);
        }
        // A view switch retains pending numeric input and returns subsequent
        // keyboard input to CMD instead of leaving focus on a view button.
        self.focus_cmd = true;
    }
    /// Title bar of one Matrix menu section. Left-click rolls it up, dragging
    /// reorders it inside its column, docks it to the right mirror column or
    /// floats it over the model — the documented Matrix menu behavior.
    fn section(&mut self, ui: &mut egui::Ui, id: SectionId) -> bool {
        let state = self.dock.section(id);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 14.),
            Sense::click_and_drag(),
        );
        let accent = section_accent(id);
        ui.painter().rect_filled(rect, 4, PANEL_ALT);
        ui.painter().rect_filled(
            Rect::from_min_size(rect.left_top(), Vec2::new(3., rect.height())),
            2,
            accent,
        );
        ui.painter().text(
            rect.left_center() + Vec2::new(8., 0.),
            Align2::LEFT_CENTER,
            id.bar_title(),
            FontId::proportional(11.),
            PANEL_TITLE,
        );
        // The reserved BUILDER area carries no close box (design review NIT 7):
        // it cannot be rolled up, so a +/− would promise something it refuses.
        // A click still reports why nothing happens (fixture N21).
        if id.is_movable() {
            ui.painter().text(
                rect.right_center() - Vec2::new(6., 0.),
                Align2::CENTER_CENTER,
                if state.collapsed { "+" } else { "−" },
                FontId::proportional(12.),
                category_text(accent),
            );
        }
        self.section_rects.insert(id.key(), rect);
        self.panel_bars
            .push((id, crate::panel_ops::ColumnSide::Left, rect));
        if response.clicked() {
            self.toggle_menu_section(id);
        }
        if response.drag_started() {
            self.begin_section_drag(id);
        }
        let collapsed = self.dock.section(id).collapsed;
        if collapsed && id.keeps_controls_while_collapsed() {
            self.closed_bar_controls.insert(id.key());
            self.section_bar_controls(ui, id);
        }
        !collapsed
    }
    /// Snaps and Layers keep important controls on their closed title bar.
    fn section_bar_controls(&mut self, ui: &mut egui::Ui, id: SectionId) {
        ui.horizontal_wrapped(|ui| match id {
            SectionId::Snaps => {
                for name in ["GridSnap", "Ortho", "Planar", "Project"] {
                    let enabled = matches!(name, "Ortho" | "Planar" | "Project");
                    let active = match name {
                        "Ortho" => self.controller.ortho(),
                        "Planar" => self.controller.planar(),
                        "Project" => self.controller.project(),
                        _ => false,
                    };
                    let response = if enabled {
                        self.icon_toggle(ui, name, active, name)
                    } else {
                        self.icon(ui, name, false, name)
                    };
                    if enabled && response.clicked() {
                        match name {
                            "Ortho" => {
                                let enabled = !self.controller.ortho();
                                self.controller.set_ortho(enabled);
                                self.status
                                    .info(if enabled { "Ortho on" } else { "Ortho off" });
                            }
                            "Planar" => {
                                let enabled = !self.controller.planar();
                                self.controller.set_planar(enabled);
                                self.status
                                    .info(if enabled { "Planar on" } else { "Planar off" });
                            }
                            "Project" => {
                                let enabled = !self.controller.project();
                                self.controller.set_project(enabled);
                                self.status.info(if enabled {
                                    "Project on"
                                } else {
                                    "Project off"
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            SectionId::Layers => {
                if let Some(current) = self
                    .controller
                    .document()
                    .current_layer
                    .and_then(|layer| self.controller.document().layers.get(&layer))
                {
                    ui.label(RichText::new(current.name.clone()).size(10.));
                }
            }
            _ => {}
        });
    }
    fn icon(&mut self, ui: &mut egui::Ui, name: &str, enabled: bool, help: &str) -> egui::Response {
        self.icon_slot(ui, name, enabled, help, None)
    }

    fn top11_icon(&mut self, ui: &mut egui::Ui, name: &str) -> egui::Response {
        let action = om9_core::top11::Top11Action::from_icon(name).expect("Top 11 icon");
        let state = self
            .controller
            .top11_availability()
            .into_iter()
            .find(|state| state.action == action)
            .expect("Top 11 state");
        let tooltip = if state.enabled {
            action.label().to_string()
        } else {
            format!("{}: {}", action.label(), state.reason)
        };
        let response = self.icon(ui, name, state.enabled, &tooltip);
        if response.clicked() {
            self.top11_action(name);
        }
        response
    }

    fn subd_flyout(&mut self, ui: &mut egui::Ui, name: &str) {
        let response = self.icon(ui, name, true, name);
        egui::Popup::menu(&response).width(260.).show(|ui| {
            ui.set_max_height(360.);
            egui::ScrollArea::vertical()
                .id_salt(("subd-flyout", name))
                .max_height(350.)
                .show(ui, |ui| {
                    for icon in subd_flyout_items(name) {
                        ui.horizontal(|ui| {
                            self.feature_icon(ui, "", icon);
                            ui.label(subd_label(icon));
                        });
                    }
                });
        });
    }

    /// A clickable icon whose active state is drawn as a bright frame instead of
    /// being disabled: snap toggles must stay clickable while they are off.
    pub(super) fn icon_toggle(
        &mut self,
        ui: &mut egui::Ui,
        name: &str,
        active: bool,
        help: &str,
    ) -> egui::Response {
        let response = self.icon_slot(ui, name, true, help, None);
        if active {
            ui.painter().rect_stroke(
                response.rect.shrink(1.5),
                4,
                Stroke::new(1.5, Color32::from_rgb(90, 190, 245)),
                egui::StrokeKind::Inside,
            );
        }
        response
    }
    pub(super) fn icon_slot(
        &mut self,
        ui: &mut egui::Ui,
        name: &str,
        enabled: bool,
        help: &str,
        right_click: Option<&str>,
    ) -> egui::Response {
        // Menu columns draw at the width-driven edge; every other caller (the
        // toolbars) keeps the responsive base because `column_icon_edge` is reset after
        // each column.
        let (rect, response) = ui.allocate_exact_size(
            Vec2::splat(self.column_icon_edge),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let fill = if !enabled {
            PANEL_BG
        } else if response.hovered() {
            Color32::from_rgb(38, 62, 78)
        } else {
            Color32::from_rgb(24, 36, 48)
        };
        ui.painter().rect_filled(rect, 4, fill);
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            4,
            Stroke::new(
                1.,
                if enabled {
                    Color32::from_rgb(72, 99, 121)
                } else {
                    Color32::from_rgb(49, 65, 81)
                },
            ),
            egui::StrokeKind::Inside,
        );
        // The icon fills more of its existing slot; no extra rows or dock
        // height are needed. A lit edge separates each card from the menu.
        ui.painter().line_segment(
            [
                rect.left_top() + Vec2::new(4., 1.5),
                rect.right_top() + Vec2::new(-4., 1.5),
            ],
            Stroke::new(1., Color32::from_white_alpha(if enabled { 65 } else { 28 })),
        );
        let inset = if self.toolbar_drawing { 1.125 } else { 1.5 };
        self.icons.paint(ui, rect.shrink(inset), name);
        match self.icon_column {
            IconColumn::Left => self.rail_icon_rects.push(rect),
            IconColumn::Right => self.right_dock_icon_rects.push(rect),
            IconColumn::None => {}
        }
        let mut tooltip = if enabled {
            help.to_owned()
        } else {
            format!("{help} — operation unavailable")
        };
        if enabled && let Some(command) = right_click {
            tooltip.push_str(&format!("\nRight-click: {command}"));
        }
        let response = response.on_hover_text(tooltip);
        if enabled
            && let Some(command) = right_click
            && response.secondary_clicked()
        {
            self.invoke(command, InvocationOrigin::Menu);
        }
        response
    }
    fn feature_icon(&mut self, ui: &mut egui::Ui, id: &str, name: &str) {
        if name == "RhinoOptions" {
            if self.icon(ui, name, true, "Options: General / Undo").clicked() {
                self.invoke(name, InvocationOrigin::Menu);
            }
            return;
        }
        if matches!(name, "SolidBooleanSplit" | "SolidCreateSolid") {
            let enabled = Controller::availability(name).is_ok();
            let label = if name == "SolidBooleanSplit" {
                "Boolean Split"
            } else {
                "Create Solid"
            };
            if self.icon(ui, name, enabled, label).clicked() {
                self.invoke(name, InvocationOrigin::Menu);
            }
            return;
        }
        let catalog_reason = crate::curve_menu::by_icon(name)
            .map(|binding| binding.disabled_reason)
            .or_else(|| crate::surface_menu::by_icon(name).map(|binding| binding.disabled_reason));
        let fallback = self
            .features
            .iter()
            .find(|f| crate::icons::icon_name(&f.id) == Some(name))
            .map(|f| f.id.clone())
            .or_else(|| (id == LINE || id == POINT).then(|| id.to_owned()));
        // An enabled add-on that provides this icon's command takes it over.
        let target = self.controller.icon_target(name, fallback.as_deref());
        let actual = target.as_ref().map(|t| t.command.clone());
        let feature = actual
            .as_ref()
            .and_then(|id| self.features.iter().find(|f| &f.id == id).cloned());
        if let (Some(actual), Some(target)) = (actual, target) {
            let enabled = target.enabled;
            let help = if enabled {
                if actual == "OM9-FILE-001" {
                    "Export Selected STL…".to_owned()
                } else {
                    feature
                        .as_ref()
                        .map_or(
                            if name == "SolidGVDMultiPipe" {
                                "Solid Multi Pipe"
                            } else {
                                name
                            },
                            |f| f.name.as_str(),
                        )
                        .to_owned()
                }
            } else {
                catalog_reason
                    .filter(|reason| !reason.is_empty())
                    .unwrap_or("operation unavailable")
                    .to_owned()
            };
            let right = if name == "SolidExtractSurface" {
                Some("SolidExtractSurfaceCopy")
            } else if name == "SolidPipe" {
                Some("SolidPipeChain")
            } else {
                feature
                    .as_ref()
                    .and_then(|f| f.right_click.as_deref())
                    .filter(|command| Controller::availability(command).is_ok())
            };
            if self.icon_slot(ui, name, enabled, &help, right).clicked() {
                self.invoke(&actual, InvocationOrigin::Menu);
            }
        } else {
            self.icon(ui, name, false, name);
        }
    }
    pub fn top_menu_rect(&self, index: usize) -> Option<Rect> {
        self.current_ui_context.as_ref().and_then(|context| {
            context.memory(|memory| {
                memory
                    .data
                    .get_temp::<Rect>(egui::Id::new(("main-top-menu-rect", index)))
            })
        })
    }
    pub fn top_menu_selected(&self) -> Option<usize> {
        self.current_ui_context.as_ref().and_then(|context| {
            context.memory(|memory| {
                memory
                    .data
                    .get_temp::<Option<usize>>(egui::Id::new("main-top-menu"))
                    .flatten()
            })
        })
    }
    pub(super) fn sidebar_ui(&mut self, ui: &mut egui::Ui) {
        apply_dock_theme(ui);
        // Reserve a narrow gutter: a floating bar covers the far-right menu
        // page buttons, Grid Snap and layer controls on narrow docks.
        ui.spacing_mut().scroll = egui::style::ScrollStyle {
            bar_width: 4.,
            bar_inner_margin: 2.,
            bar_outer_margin: 0.,
            ..egui::style::ScrollStyle::solid()
        };
        // The column order, visibility and roll-up state come from the typed
        // layout, so the documented Matrix order survives a session.
        let sections = self.dock.left_sections();
        let scroll = std::mem::take(&mut self.sidebar_scroll);
        self.rail_icon_rects.clear();
        let scrolled = egui::ScrollArea::vertical()
            .id_salt("matrix-sidebar-sections")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .vertical_scroll_offset(scroll)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for id in sections {
                    if !self.dock.section(id).visible {
                        continue;
                    }
                    if self.section(ui, id) {
                        self.sidebar_section_ui(ui, id);
                    }
                }
            });
        self.sidebar_content_size = Some(scrolled.content_size);
        self.sidebar_content_rect = Some(scrolled.inner_rect);
        self.sidebar_scrollbar_rect = Some(Rect::from_min_size(
            scrolled.inner_rect.right_top() + Vec2::new(2., 0.),
            Vec2::new(4., scrolled.inner_rect.height()),
        ));
        self.left_content_bottom = scrolled.inner_rect.bottom();
        self.sidebar_scroll = scrolled.state.offset.y;
    }
    /// Body of one Matrix menu section. Builders open in the reserved area below
    /// Projects; the slot itself never owns a builder model.
    fn sidebar_section_ui(&mut self, ui: &mut egui::Ui, id: SectionId) {
        match id {
            SectionId::IconHistory => {
                ui.horizontal_wrapped(|ui| {
                    let recent: Vec<_> = self.controller.recent().iter().cloned().collect();
                    for id in recent {
                        let name = crate::icons::icon_name(&id).unwrap_or("Unavailable");
                        self.feature_icon(ui, &id, name);
                    }
                    if self
                        .icon(
                            ui,
                            "Undo",
                            self.controller.coordinator().undo_len() > 0
                                && !self.controller.is_running(),
                            "Undo",
                        )
                        .clicked()
                    {
                        let r = self.controller.undo();
                        self.feedback(r);
                    }
                    if self
                        .icon(
                            ui,
                            "Redo",
                            self.controller.coordinator().redo_len() > 0
                                && !self.controller.is_running(),
                            "Redo",
                        )
                        .clicked()
                    {
                        let r = self.controller.redo();
                        self.feedback(r);
                    }
                });
            }
            SectionId::MainMenu => {
                let top_id = egui::Id::new("main-top-menu");
                let mut top = ui
                    .ctx()
                    .memory(|memory| memory.data.get_temp::<Option<usize>>(top_id).flatten());
                let top_gap = 1.0;
                let top_width = ((ui.available_width() - top_gap * (TOP_MENUS.len() - 1) as f32)
                    / TOP_MENUS.len() as f32)
                    .floor();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = top_gap;
                    for (index, label) in TOP_MENUS.iter().enumerate() {
                        let selected = top == Some(index);
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::new(top_width, 18.), Sense::click());
                        let shown = self.controller.tr().menu(label);
                        let response = response.on_hover_text(shown.clone());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 4, Color32::from_rgb(19, 27, 36));
                        if selected {
                            painter.rect_filled(rect, 4, PANEL_ALT);
                        }
                        painter.rect_stroke(
                            rect.shrink(0.5),
                            4,
                            Stroke::new(1., if selected { PANEL_EDGE } else { PANEL_TITLE }),
                            egui::StrokeKind::Inside,
                        );
                        painter.text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            &shown,
                            FontId::proportional(11.),
                            PANEL_INK,
                        );
                        ui.ctx().memory_mut(|memory| {
                            memory
                                .data
                                .insert_temp(egui::Id::new(("main-top-menu-rect", index)), rect);
                        });
                        if response.clicked() {
                            top = Some(index);
                            self.focus_cmd = true;
                        }
                    }
                });
                self.main_category_rects.clear();
                let columns = if ui.available_width() < 280. { 5 } else { 6 };
                let gap = 1.0;
                let card_width =
                    ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32).floor();
                for (row, categories) in CATEGORIES.chunks(columns).enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(gap, 2.);
                        for (column, category) in categories.iter().enumerate() {
                            let i = row * columns + column;
                            let accent = CATEGORY_COLORS[i];
                            let selected = self.main_category == i;
                            let font = super::typography::semibold(
                                ui.ctx(),
                                if card_width < 48. { 9.5 } else { 10.5 },
                            );
                            let text_color = category_text(accent);
                            let shown = self.controller.tr().menu(category);
                            let galley = ui.painter().layout_no_wrap(
                                shown.clone(),
                                font.clone(),
                                text_color,
                            );
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::new(card_width, 18.), Sense::click());
                            let response = response.on_hover_text(shown);
                            let painter = ui.painter();
                            // Keep the raised card separate from the brighter label.
                            painter.rect_filled(
                                rect.translate(Vec2::new(0., 2.)),
                                5,
                                Color32::from_rgba_unmultiplied(
                                    0,
                                    0,
                                    0,
                                    if selected { 140 } else { 95 },
                                ),
                            );
                            painter.rect_filled(
                                rect,
                                5,
                                if selected {
                                    SELECTED_CARD
                                } else {
                                    Color32::from_rgb(19, 27, 36)
                                },
                            );
                            painter.line_segment(
                                [
                                    egui::pos2(rect.left() + 5., rect.top() + 1.),
                                    egui::pos2(rect.right() - 5., rect.top() + 1.),
                                ],
                                Stroke::new(1., Color32::from_rgba_unmultiplied(255, 255, 255, 34)),
                            );
                            painter.rect_stroke(
                                rect.shrink(0.5),
                                5,
                                Stroke::new(1., if selected { accent } else { PANEL_EDGE }),
                                egui::StrokeKind::Inside,
                            );
                            let text_pos = egui::pos2(
                                rect.center().x - galley.size().x / 2.,
                                rect.center().y - galley.size().y / 2.,
                            );
                            // A real semibold face keeps one antialiased glyph pass.
                            painter.galley(text_pos, galley, text_color);
                            self.main_category_rects.insert(i, rect);
                            if response.clicked() {
                                self.main_category = i;
                                top = None;
                                self.focus_cmd = true;
                            }
                        }
                    });
                }
                let category = CATEGORIES[self.main_category];
                let palette_name = top.map_or(category, |index| TOP_MENUS[index]);
                let palette = if top.is_some() {
                    crate::menu_catalog::category(palette_name).unwrap_or(&[])
                } else {
                    category_palette(category)
                };
                // Reference menus use ten icons per row, two rows per page and
                // a fixed page column on the far right. The page column never
                // follows the icon content: it is anchored right-to-left.
                const PAGE_COLUMNS: usize = 10;
                const PAGE_ROWS: usize = 2;
                let page_id = egui::Id::new(("main-menu-page", palette_name));
                let mut page = ui
                    .ctx()
                    .memory(|memory| memory.data.get_temp::<usize>(page_id).unwrap_or(0));
                let reserve = if ui.available_width() >= 280. {
                    24.
                } else {
                    32.
                };
                let page_width = (ui.available_width() - reserve).max(80.);
                let gap = 1.0;
                let fit_capacity = ((page_width - gap * (PAGE_COLUMNS - 1) as f32)
                    / PAGE_COLUMNS as f32)
                    .floor()
                    .max(16.);
                // Scale the actual fitted slot too: scaling only the generic
                // dock edge stalls once ten columns reach their width limit.
                let fit_edge = (fit_capacity * self.palette_icon_scale())
                    .round()
                    .max(18.)
                    .min(fit_capacity)
                    .min(self.column_icon_edge);
                let saved_edge = self.column_icon_edge;
                self.column_icon_edge = fit_edge;
                let page_size = PAGE_COLUMNS * PAGE_ROWS;
                let pages = palette.len().div_ceil(page_size).max(1);
                page = page.min(pages - 1);
                let start = page * page_size;
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(page_width, fit_edge * PAGE_ROWS as f32 + gap),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            for row in 0..PAGE_ROWS {
                                ui.horizontal(|ui| {
                                    for column in 0..PAGE_COLUMNS {
                                        let index = start + row * PAGE_COLUMNS + column;
                                        if let Some(name) = palette.get(index) {
                                            self.feature_icon(ui, "", name);
                                        }
                                    }
                                });
                            }
                        },
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.vertical(|ui| {
                            for index in 0..pages {
                                if menu_page_button(ui, index, page == index).clicked() {
                                    page = index;
                                }
                            }
                        });
                    });
                });
                self.column_icon_edge = saved_edge;
                ui.ctx()
                    .memory_mut(|memory| memory.data.insert_temp(page_id, page));
                if palette.is_empty() {
                    ui.label(format!("{palette_name} menu has no pinned commands"));
                }
                // Matrix keeps Top 11 immediately below the main menu, above
                // DISPLAY. Fit all eleven buttons on one row at the current rail width.
                let saved_edge = self.column_icon_edge;
                let gap = 1.;
                self.column_icon_edge = ((ui.available_width() - 10. * gap) / 11.)
                    .max(0.)
                    .min(saved_edge);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for name in toolbar_palette("Top 11") {
                        let response = self.top11_icon(ui, name);
                        ui.ctx().memory_mut(|memory| {
                            memory
                                .data
                                .insert_temp(egui::Id::new(("top11-sidebar", name)), response.rect);
                        });
                    }
                });
                self.column_icon_edge = saved_edge;
                ui.ctx()
                    .memory_mut(|memory| memory.data.insert_temp(top_id, top));
            }
            SectionId::Display | SectionId::Snaps | SectionId::InfoSettings => {
                self.fixed_sidebar_palette(ui, id);
            }
            SectionId::Layers => {
                self.layers_ui(ui);
            }
            SectionId::Projects => {
                self.projects_ui(ui);
            }

            SectionId::BuilderSlot => {
                self.builder_slot_ui(ui);
            }
        }
    }
    /// PROJECTS: the Project Manager area of the left column.
    fn projects_ui(&mut self, ui: &mut egui::Ui) {
        self.project_row_rects.clear();
        self.jobbag_slot_rects.clear();
        self.project_action_rects.clear();
        let active_id = self.project_manager.active().id;
        let active_index = self.project_manager.active_index();
        let rows: Vec<(crate::project_manager::ProjectId, String)> = self
            .project_manager
            .rows()
            .iter()
            .map(|row| (row.id, row.name.clone()))
            .collect();
        ui.horizontal_wrapped(|ui| {
            for (index, (id, name)) in rows.into_iter().enumerate() {
                let response = ui.selectable_label(index == active_index, &name);
                self.project_row_rects.insert(id, response.rect);
                if response.clicked()
                    && let Err(error) = self.project_manager.select(id)
                {
                    self.feedback(Err(error));
                }
            }
            let add = ui.small_button("+");
            self.project_action_rects.insert("new-project", add.rect);
            if add.on_hover_text("Create a project row").clicked() {
                let name = format!("Project {}", self.project_manager.rows().len() + 1);
                let revision = self.controller.document().revision;
                let result = self
                    .project_manager
                    .create_project(&name, revision)
                    .map(|_| ());
                self.feedback(result);
            }
        });
        let active = self.project_manager.active();
        let mut name = active.name.clone();
        let mut notes = active.notes.clone();
        let mut customer = active.customer.clone();
        ui.horizontal(|ui| {
            ui.label("Name");
            let response = ui.add(
                egui::TextEdit::singleline(&mut name)
                    .desired_width(104.)
                    .id_salt("project-name"),
            );
            if response.changed()
                && let Err(error) = self.project_manager.rename_project(active_id, &name)
            {
                self.feedback(Err(error));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Customer");
            let response = ui.add(
                egui::TextEdit::singleline(&mut customer)
                    .desired_width(104.)
                    .id_salt("project-customer"),
            );
            if response.changed()
                && let Err(error) = self.project_manager.set_customer(&customer)
            {
                self.feedback(Err(error));
            }
        });
        let notes_response = ui.add(
            egui::TextEdit::singleline(&mut notes)
                .hint_text("Project notes")
                .desired_width(ui.available_width().min(220.))
                .id_salt("project-notes"),
        );
        if notes_response.changed()
            && let Err(error) = self.project_manager.set_notes(&notes)
        {
            self.feedback(Err(error));
        }
        type SlotRow = (usize, Option<(String, usize, String, u64)>, bool);
        let slots: Vec<SlotRow> = (0..3)
            .map(|slot| {
                let info = self
                    .project_manager
                    .active()
                    .slot(slot)
                    .and_then(|slot| slot.snapshot.as_ref())
                    .map(|snapshot| {
                        (
                            snapshot.thumbnail.label.clone(),
                            snapshot.thumbnail.count,
                            snapshot.thumbnail.kind.clone(),
                            snapshot.thumbnail.source_revision,
                        )
                    });
                let available = self
                    .project_manager
                    .slot_available(slot, self.controller.document());
                (slot, info, available)
            })
            .collect();
        for (slot, info, available) in slots {
            ui.horizontal(|ui| {
                let (rect, response) = ui.allocate_exact_size(Vec2::new(76., 44.), Sense::click());
                self.jobbag_slot_rects.insert(slot, rect);
                let fill = if available {
                    Color32::from_rgb(22, 58, 42)
                } else {
                    Color32::from_rgb(28, 34, 32)
                };
                ui.painter().rect_filled(rect, 4, fill);
                bevel(ui.painter(), rect);
                ui.painter().rect_stroke(
                    rect.shrink(0.5),
                    4,
                    egui::Stroke::new(1., PANEL_EDGE),
                    egui::StrokeKind::Inside,
                );
                if let Some((label, count, kind, revision)) = info.as_ref() {
                    self.icons.paint(ui, rect.shrink(7.), "Project");
                    ui.painter().text(
                        rect.left_top() + Vec2::new(3., 2.),
                        Align2::LEFT_TOP,
                        format!("{count} {kind}"),
                        FontId::proportional(9.),
                        PANEL_INK,
                    );
                    ui.painter().text(
                        rect.left_bottom() + Vec2::new(3., -2.),
                        Align2::LEFT_BOTTOM,
                        format!("{label} r{revision}"),
                        FontId::proportional(9.),
                        PANEL_TITLE,
                    );
                    response
                        .clone()
                        .on_hover_text("Jobbag slot; click Restore below");
                } else {
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "Empty slot",
                        FontId::proportional(9.),
                        PANEL_TITLE,
                    );
                }
                ui.vertical(|ui| {
                    if ui.small_button(format!("Capture {}", slot + 1)).clicked() {
                        let selected = self.controller.jobbag_selected_entities();
                        let result = match selected {
                            Ok(entities) => self
                                .project_manager
                                .capture(slot, entities, self.controller.document())
                                .map(|_| ()),
                            Err(error) => Err(error),
                        };
                        self.feedback(result);
                    }
                    if available && ui.small_button("Restore").clicked() {
                        let result = (|| {
                            let entities = self.project_manager.restore_entities(slot)?;
                            self.controller.insert_jobbag_entities(entities)?;
                            Ok(())
                        })();
                        self.feedback(result);
                    }
                });
            });
        }
        ui.horizontal(|ui| {
            let capture = ui.small_button("Capture Master");
            self.project_action_rects
                .insert("capture-master", capture.rect);
            if capture.clicked() {
                let result = self
                    .project_manager
                    .capture_master(self.controller.document())
                    .map(|_| ());
                self.feedback(result);
            }
            if let Some(master) = self.project_manager.master_document() {
                ui.label(format!(
                    "Master: {} objects, {} layers r{}",
                    master.entities.len(),
                    master.layer_order.len(),
                    master.revision
                ));
            }
        });
        ui.horizontal(|ui| {
            let restore = ui.add_enabled(
                self.project_manager.master_document().is_some(),
                egui::Button::new("Restore Master"),
            );
            self.project_action_rects
                .insert("restore-master", restore.rect);
            if restore.clicked() {
                let result = self.restore_master_live();
                self.feedback(result);
            }
        });
    }
    /// BUILDERS: the builder panel in the space below the Project Manager. The tab row
    /// switches modes and each tab keeps its own parameters; the active tab paints its typed
    /// fields, the preview summary, Apply/Reset/Cancel, the profile library button and the
    /// handle read-outs.
    fn builder_slot_ui(&mut self, ui: &mut egui::Ui) {
        if self.builder_panel_ui(ui) {
            self.builder_slot_rect = Some(ui.min_rect());
            return;
        }
        self.builder_slot_rect = Some(ui.available_rect_before_wrap());
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("BUILDERS").strong());
            for (kind, active) in self.controller.builder_tabs() {
                let response = ui
                    .add(
                        egui::Button::new(kind.title())
                            .selected(active)
                            .min_size(Vec2::new(0., 18.)),
                    )
                    .on_hover_text(match kind {
                        om9_core::builder::BuilderKind::RingRail => {
                            "Ring rail from a finger diameter"
                        }
                        om9_core::builder::BuilderKind::RailProfile => {
                            "Profile on the selected rail"
                        }
                        om9_core::builder::BuilderKind::RailSweep => {
                            "Sweep the selected profile along the rail"
                        }
                        om9_core::builder::BuilderKind::OutsideRail => {
                            "Offset rail outside the selected rail"
                        }
                        om9_core::builder::BuilderKind::RingResizer => {
                            "Resize a ring rail to a new diameter"
                        }
                        _ => "Ring builder mode",
                    });
                if response.clicked() && !active {
                    let result = self.controller.switch_builder(kind);
                    self.feedback(result);
                }
            }
        });
        let fields = self.controller.builder_field_views();
        if fields.is_empty() {
            ui.label("Select a rail curve and choose a builder tab");
            return;
        }
        if let Some(summary) = self.controller.builder_summary() {
            ui.label(summary);
        }
        if !self.controller.builder_editable() {
            ui.label(
                egui::RichText::new("Read-only: the builder input is on a locked layer")
                    .color(Color32::from_rgb(235, 170, 80)),
            );
        }
        // A read-only session paints its values but no enabled edit control, so a typed edit
        // cannot even be attempted (`OM9-L3-03-11-005`).
        let editable = self.controller.builder_editable();
        for field in fields {
            if let Some((selected, choices)) = field.choice {
                ui.horizontal_wrapped(|ui| {
                    ui.label(field.label);
                    let mut chosen = selected.clone();
                    egui::ComboBox::from_id_salt(("builder-choice", field.name))
                        .selected_text(chosen.clone())
                        .show_ui(ui, |ui| {
                            for choice in &choices {
                                ui.selectable_value(&mut chosen, choice.clone(), choice);
                            }
                        });
                    if chosen != selected {
                        let result = self.controller.builder_set_field(
                            field.name,
                            om9_core::request::Value::Choice(chosen),
                        );
                        self.feedback(result);
                    }
                    if field.name == "preset" && ui.button("Library…").clicked() {
                        self.builder_profile_window = true;
                    }
                });
            } else if let Some(mut value) = field.boolean {
                if ui
                    .add_enabled(editable, egui::Checkbox::new(&mut value, field.label))
                    .changed()
                {
                    let result = self
                        .controller
                        .builder_set_field(field.name, om9_core::request::Value::Boolean(value));
                    self.feedback(result);
                }
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut value = field.value;
                    let revision = self.controller.builder_revision();
                    let step = field.step.unwrap_or(0.1);
                    let suffix = match field.dimension {
                        om9_core::request::Dimension::Length => " mm",
                        om9_core::request::Dimension::Angle => " deg",
                        om9_core::request::Dimension::Ratio => "",
                        om9_core::request::Dimension::Count => "",
                    };
                    let response = ui.add_enabled(
                        editable,
                        egui::DragValue::new(&mut value)
                            .speed(step)
                            .range(field.min..=field.max)
                            .suffix(suffix),
                    );
                    ui.label(field.label);
                    if response.changed() {
                        // A panel edit is refused if a handle drag already moved the same
                        // field since this widget was painted (`OM9-L3-03-11-006`).
                        let editing = om9_core::request::Value::Quantity {
                            value,
                            dimension: field.dimension,
                        };
                        let result = match revision {
                            Some(revision) => self
                                .controller
                                .builder_set_field_at(field.name, editing, revision),
                            None => self.controller.builder_set_field(field.name, editing),
                        };
                        self.feedback(result);
                    }
                    // A slider moves in whole steps of the field's own step, so 2.0 -> 2.5
                    // lands on 2.5 mm in the widget, the model and the preview
                    // (`OM9-L3-03-11-003`).
                    if let (Some(ratio), Some(_step)) = (field.ratio, field.step) {
                        let span = (field.max - field.min).max(1.0e-9);
                        let mut ratio = ratio;
                        let response = ui.add_enabled(
                            editable,
                            egui::Slider::new(&mut ratio, 0.0..=1.0)
                                .step_by(step / span)
                                .show_value(false),
                        );
                        if response.changed() {
                            let result = self.controller.builder_set_slider(field.name, ratio);
                            self.feedback(result);
                        }
                    }
                });
            }
        }
        ui.horizontal_wrapped(|ui| {
            let apply = ui.add_enabled(editable, egui::Button::new("Apply"));
            #[cfg(test)]
            ui.ctx().memory_mut(|memory| {
                memory
                    .data
                    .insert_temp(egui::Id::new("builder-apply-test"), apply.rect)
            });
            if apply.clicked() {
                let result = self.controller.builder_apply();
                self.feedback(result);
            }
            if ui
                .add_enabled(editable, egui::Button::new("Reset"))
                .clicked()
            {
                let result = self.controller.builder_reset();
                self.feedback(result);
            }
            if ui.button("Cancel").clicked() {
                self.controller.builder_cancel();
            }
            if ui.button("Styles…").clicked() {
                self.builder_style_window = true;
            }
        });
        if let Some(applied) = self.controller.builder_applied_style() {
            ui.label(format!("style: {applied}"));
        }
        ui.horizontal_wrapped(|ui| {
            for handle in self.controller.builder_handles() {
                ui.label(format!("{} {:.2} mm", handle.label, handle.value));
            }
        });
        self.builder_profile_window_ui(ui);
        self.history_window_ui(ui);
        self.builder_style_window_ui(ui);
    }

    /// The Matrix History report (`OM9-L3-03-13-003`, `-004`, `-005`): the recorded
    /// features, their state, the broken rows, and per-row Edit/Clear actions.
    fn history_window_ui(&mut self, ui: &mut egui::Ui) {
        if !self.history_window {
            return;
        }
        let entries = self.controller.history_entries();
        let mut record = self.controller.history_record_enabled();
        let mut update = self.controller.history_update_enabled();
        let mut chosen_record: Option<bool> = None;
        let mut chosen_update: Option<bool> = None;
        let mut edit: Option<om9_core::identity::ObjectId> = None;
        let mut clear: Option<om9_core::identity::ObjectId> = None;
        let mut open = true;
        egui::Window::new("History")
            .id(egui::Id::new("om9-history-report"))
            .open(&mut open)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut record, "Record").changed() {
                        chosen_record = Some(record);
                    }
                    if ui.checkbox(&mut update, "Update").changed() {
                        chosen_update = Some(update);
                    }
                    ui.label(format!("{} features", entries.len()));
                });
                if entries.is_empty() {
                    ui.label("No recorded feature in this document");
                } else {
                    ui.label(
                        "Edit rebuilds the selected feature from its recorded recipe; Clear detaches \
                         its history but keeps the geometry.",
                    );
                }
                egui::Grid::new("om9-history-grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Feature");
                        ui.label("Recipe");
                        ui.label("State");
                        ui.label("Problem");
                        ui.label("");
                        ui.end_row();
                        for entry in &entries {
                            ui.label(&entry.command);
                            ui.label(entry.recipe.to_string());
                            ui.label(format!("{:?}", entry.state));
                            ui.label(&entry.detail);
                            ui.horizontal(|ui| {
                                if ui.small_button("Edit").clicked() {
                                    edit = entry.outputs.first().copied();
                                }
                                if ui.small_button("Clear").clicked() {
                                    clear = entry.outputs.first().copied();
                                }
                            });
                            ui.end_row();
                        }
                    });
            });
        self.history_window = open;
        if let Some(value) = chosen_record {
            let result = self.controller.set_history_record(value).map(|_| ());
            self.feedback(result);
        }
        if let Some(value) = chosen_update {
            let result = self.controller.set_history_update(value).map(|_| ());
            self.feedback(result);
        }
        if let Some(object) = edit {
            let result = self
                .controller
                .begin_builder_from_history(object)
                .map(|_| ());
            self.feedback(result);
        }
        if let Some(object) = clear {
            let result = self.controller.clear_object_history(object).map(|_| ());
            self.feedback(result);
        }
    }

    /// The builder style browser (`OM9-L3-03-12-002` … `-006`).
    ///
    /// One row per style: a thumbnail (or its placeholder), the name, an Apply that loads the
    /// parameters into the preview only, and a save field that snapshots the current session
    /// under a new user name. An entry for another mode stays visible but disabled, so the
    /// list explains a refusal instead of hiding it.
    fn builder_style_window_ui(&mut self, ui: &mut egui::Ui) {
        if !self.builder_style_window {
            return;
        }
        let rows = self.controller.builder_styles();
        let let_editable = self.controller.builder_editable();
        let source = self
            .controller
            .builder_style_path()
            .map(|path| path.display().to_string());
        let mut chosen: Option<String> = None;
        let mut saved: Option<String> = None;
        let mut matched = false;
        let mut open = true;
        egui::Window::new("Builder Styles")
            .id(egui::Id::new("om9-builder-styles"))
            .open(&mut open)
            .resizable(false)
            .default_width(260.0)
            .show(ui.ctx(), |ui| {
                for row in &rows {
                    ui.horizontal(|ui| {
                        // A missing or corrupt thumbnail paints the placeholder square with
                        // its fallback icon; the row is never dropped for it.
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(14., 14.), egui::Sense::hover());
                        let fill = if row.fallback {
                            Color32::from_gray(205)
                        } else {
                            Color32::from_rgb(214, 238, 228)
                        };
                        ui.painter().rect_filled(rect, 2.0, fill);
                        let label = if row.applied {
                            format!("{} *", row.name)
                        } else {
                            row.name.clone()
                        };
                        let hover = if row.enabled {
                            format!("Load {} into the preview", row.name)
                        } else {
                            format!("{} is not compatible with this builder", row.name)
                        };
                        let button = egui::Button::new(label).selected(row.applied);
                        if ui
                            .add_enabled(let_editable && row.enabled, button)
                            .on_hover_text(hover)
                            .clicked()
                        {
                            chosen = Some(row.name.clone());
                        }
                        if row.builtin {
                            ui.small("-");
                        } else {
                            ui.small("user");
                        }
                    });
                }
                if rows.is_empty() {
                    ui.label("Open a builder to list its styles");
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Save as");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.builder_style_name)
                            .desired_width(120.0)
                            .hint_text("Classic 2.5"),
                    );
                    if ui
                        .add_enabled(let_editable, egui::Button::new("Save"))
                        .clicked()
                    {
                        saved = Some(self.builder_style_name.clone());
                    }
                });
                if ui
                    .button("Match selected attributes")
                    .on_hover_text("Copy width/height/material from the selected design entity")
                    .clicked()
                {
                    matched = true;
                }
                match &source {
                    Some(path) => {
                        ui.label(format!("user file: {path}"));
                    }
                    None => {
                        ui.label("user styles: this session (set OM9_STYLE_LIBRARY to persist)");
                    }
                }
                if let Some(error) = self.controller.builder_style_error() {
                    ui.colored_label(Color32::from_rgb(200, 120, 40), error);
                }
            });
        if let Some(name) = chosen {
            let result = self.controller.builder_apply_style(&name);
            self.feedback(result.map(|_| ()));
        }
        if let Some(name) = saved
            && !name.trim().is_empty()
        {
            let result = self.controller.builder_save_style(&name);
            match result {
                Ok(summary) => {
                    self.builder_style_name.clear();
                    self.feedback(Ok(()));
                    self.status.info(&summary);
                }
                Err(error) => {
                    self.feedback(Err(error));
                }
            }
        }
        if matched {
            let result = self.controller.builder_match_attributes();
            match result {
                Ok(summary) => {
                    self.feedback(Ok(()));
                    self.status.info(&summary);
                }
                Err(error) => {
                    self.feedback(Err(error));
                }
            }
        }
        self.builder_style_window = open;
    }

    /// The profile library window: one row per preset with the selected state and a Remove
    /// action. Choosing only changes the builder preview; the document is untouched.
    fn builder_profile_window_ui(&mut self, ui: &mut egui::Ui) {
        if !self.builder_profile_window {
            return;
        }
        let presets = self.controller.builder_presets();
        let selected = self.controller.builder_selected_preset();
        let mut chosen: Option<String> = None;
        let mut removed: Option<String> = None;
        let mut open = true;
        egui::Window::new("Profile Library")
            .id(egui::Id::new("om9-builder-profile-library"))
            .open(&mut open)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                for preset in &presets {
                    ui.horizontal(|ui| {
                        let is_selected = selected == Some(*preset);
                        if ui
                            .add(egui::Button::new(preset.label()).selected(is_selected))
                            .clicked()
                        {
                            chosen = Some(preset.name().to_string());
                        }
                        if ui.small_button("Remove").clicked() {
                            removed = Some(preset.name().to_string());
                        }
                    });
                }
                if presets.is_empty() {
                    ui.label("No profile left in this library");
                }
            });
        if let Some(name) = chosen {
            let result = self.controller.builder_set_preset(&name);
            self.feedback(result);
        }
        if let Some(name) = removed {
            let result = self.controller.builder_remove_preset(&name);
            self.feedback(result);
        }
        self.builder_profile_window = open;
    }
    pub(super) fn toolbars_ui(&mut self, ui: &mut egui::Ui) {
        self.toolbar_icon_edge = (dock_layout::ICON_BASE_EDGE * self.screen_icon_scale)
            .clamp(dock_layout::ICON_MIN_EDGE, 32.)
            .round()
            * 0.75;
        self.column_icon_edge = self.toolbar_icon_edge;
        self.file_toolbar_rects.clear();
        self.toolbar_rect = Some(ui.max_rect());
        egui::ScrollArea::horizontal()
            .id_salt("standard-toolbar-tabs")
            .max_height(18.)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (i, name) in TABS.iter().enumerate() {
                        if ui
                            .add(
                                egui::Button::new(*name)
                                    .selected(self.toolbar_tab == i)
                                    .min_size(Vec2::new(0., 16.)),
                            )
                            .clicked()
                        {
                            self.toolbar_tab = i;
                            self.focus_cmd = true;
                        }
                    }
                });
            });
        let row = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 26.));
        gradient(
            ui.painter(),
            row,
            Color32::from_rgb(226, 232, 240),
            Color32::from_rgb(248, 250, 252),
        );
        let tab = TABS[self.toolbar_tab];
        self.toolbar_drawing = true;
        egui::ScrollArea::horizontal()
            .id_salt(("toolbar-palette", tab))
            .max_height(self.toolbar_icon_edge + 2.)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(self.toolbar_icon_edge);
                    let palette = toolbar_palette(tab);
                    if palette.is_empty() {
                        ui.weak(format!("{tab} tools unavailable"));
                    }
                    for name in palette {
                        if matches!(
                            (tab, *name),
                            ("Emboss", "Emboss Resources")
                                | ("Sculpt", "Sculpt Brush")
                                | ("SubD", "SubD Primitives")
                                | ("SubD", "SubD Hotkeys Settings")
                                | ("SubD", "SubD Elements")
                                | ("Visibility", "Visibility Lock Objects")
                                | ("Visibility", "Visibility Invert and Hide Control Points")
                                | ("Visibility", "Visibility Add Clipping Plane")
                                | ("Visibility", "Visibility Hide A")
                                | ("Display", "Display Render Mesh Settings")
                        ) {
                            ui.add_space(6.);
                        }
                        #[cfg(test)]
                        {
                            let rect = Rect::from_min_size(
                                ui.next_widget_position(),
                                Vec2::splat(self.column_icon_edge),
                            );
                            ui.ctx().memory_mut(|memory| {
                                memory
                                    .data
                                    .insert_temp(egui::Id::new(("toolbar-test", tab, name)), rect);
                            });
                        }
                        match *name {
                            "Standard Copy" | "Standard Paste" if tab == "Standard" => {
                                let copying = *name == "Standard Copy";
                                let enabled = !self.controller.is_running()
                                    && (!copying || !self.controller.selected().is_empty());
                                if self.icon(ui, name, enabled, if copying { "Copy OM9 objects" } else { "Paste objects" }).clicked() {
                                    self.clipboard_action(if copying { super::clipboard::ClipboardAction::Copy } else { super::clipboard::ClipboardAction::Paste });
                                }
                            }
                            "New" | "Open" | "Save" => {
                                let kind = crate::file_lifecycle::FileKind::from_command(name)
                                    .expect("File control");
                                let response =
                                    self.icon(ui, name, true, "Native OM9 document (.om9)");
                                self.file_toolbar_rects.insert(kind, response.rect);
                                if response.clicked() {
                                    self.invoke(kind.feature_id(), InvocationOrigin::Menu);
                                }
                            }
                            "Undo" | "Redo" => {
                                let undo = *name == "Undo";
                                let available = !self.controller.is_running()
                                    && if undo {
                                        self.controller.coordinator().undo_len() > 0
                                    } else {
                                        self.controller.coordinator().redo_len() > 0
                                    };
                                if self.icon(ui, name, available, name).clicked() {
                                    let result = if undo {
                                        self.controller.undo()
                                    } else {
                                        self.controller.redo()
                                    };
                                    self.feedback(result);
                                }
                            }
                            "Line" | "Point" if tab == "Standard" || tab == "Curve Tools" => {
                                self.feature_icon(
                                    ui,
                                    if *name == "Line" { LINE } else { POINT },
                                    name,
                                );
                            }
                            "Wireframe" | "Shaded" if tab == "Display" => {
                                use crate::viewport_menu::{DisplayPreset, MenuAction};
                                let enabled = !self.controller.is_running();
                                if self.icon(ui, name, enabled, name).clicked() {
                                    let action = MenuAction::Preset(if *name == "Wireframe" {
                                        DisplayPreset::Wireframe
                                    } else {
                                        DisplayPreset::Shaded
                                    });
                                    let result = self
                                        .controller
                                        .capture_view_menu(self.controller.workspace.active)
                                        .and_then(|capture| {
                                            self.controller.apply_view_menu(&capture, action)
                                        });
                                    self.feedback(result);
                                }
                            }
                            // Toolbar slots (Move, Rotate, Mirror …) run the installed
                            // add-on that provides the command, ahead of OM9's own.
                            _ if self.controller.icon_addon(name).is_some() => {
                                self.feature_icon(ui, "", name);
                            }
                            _ if tab == "Top 11" => {
                                self.top11_icon(ui, name);
                            }
                            _ if tab == "SubD" && subd_flyout_items(name).next().is_some() => {
                                self.subd_flyout(ui, name);
                            }
                            _ => {
                                self.feature_icon(ui, "", name);
                            }
                        }
                    }
                });
            });
        self.toolbar_drawing = false;
        egui::ScrollArea::horizontal()
            .id_salt("workspace-view-tabs")
            .max_height(19.)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for id in [
                        ViewId::Perspective,
                        ViewId::ThroughFinger,
                        ViewId::SideView,
                        ViewId::LookingDown,
                    ] {
                        let active = self.controller.workspace.active == id;
                        if ui
                            .add(
                                egui::Button::new(id.title())
                                    .fill(if active {
                                        GREEN
                                    } else {
                                        Color32::from_rgb(197, 208, 213)
                                    })
                                    .min_size(Vec2::new(0., 17.)),
                            )
                            .clicked()
                        {
                            self.activate_view(id);
                        }
                    }
                });
            });
    }
    /// Mirror a user-resized dock panel into the typed layout so the size can be
    /// persisted. Only a real change marks the layout dirty, so a clamped frame
    /// cannot loop.
    pub(super) fn sync_panel_size(&mut self, id: PanelId, size: f32) {
        // While the region check has no usable plan (window below the minimum),
        // the panels are squeezed by the window and must not rewrite the stored
        // sizes: the layout stays exactly as it was.
        if self.region_error.is_some() {
            return;
        }
        let before = self.dock.state(id).size;
        self.dock.set_size(id, size);
        if (self.dock.state(id).size - before).abs() > 0.5 {
            self.dock_dirty = true;
        }
    }
    /// Far-right menu column: a ~300-pixel mirror of the left one holding the
    /// menus the user docked there. Each menu keeps the same title bar behavior
    /// (roll up, reorder, re-dock, float).
    pub(super) fn right_dock_ui(&mut self, ui: &mut egui::Ui) {
        apply_dock_theme(ui);
        ui.set_min_width(ui.available_width());
        self.right_dock_icon_rects.clear();
        let sections = self.dock.right_sections();
        if sections.is_empty() {
            ui.label(
                RichText::new("Drag a menu title bar here to dock it")
                    .size(10.)
                    .color(PANEL_MUTED),
            );
            return;
        }
        for (index, id) in sections.iter().enumerate() {
            let state = self.dock.section(*id);
            let mut move_by: Option<i32> = None;
            let mut toggle = false;
            let mut dock_left = false;
            let mut float = false;
            let mut drag_started = false;
            ui.horizontal(|ui| {
                let arrow = if state.collapsed { "+" } else { "−" };
                let response = ui
                    .add(
                        egui::Button::selectable(
                            !state.collapsed,
                            RichText::new(format!("{arrow} {}", id.bar_title())).color(PANEL_TITLE),
                        )
                        .sense(Sense::click_and_drag()),
                    )
                    .on_hover_text("Collapse or expand this menu; drag it to move it");
                self.section_rects.insert(id.key(), response.rect);
                self.panel_bars
                    .push((*id, crate::panel_ops::ColumnSide::Right, response.rect));
                toggle = response.clicked();
                drag_started = response.drag_started();
                let up = ui
                    .add_enabled(index > 0, egui::Button::new("▲").small())
                    .on_hover_text("Move this menu up");
                if up.clicked() {
                    move_by = Some(-1);
                }
                let down = ui
                    .add_enabled(index + 1 < sections.len(), egui::Button::new("▼").small())
                    .on_hover_text("Move this menu down");
                if down.clicked() {
                    move_by = Some(1);
                }
                dock_left = ui
                    .small_button("◀")
                    .on_hover_text("Dock this menu back in the left column")
                    .clicked();
                float = ui
                    .small_button("⧉")
                    .on_hover_text("Float this menu over the model")
                    .clicked();
            });
            if toggle {
                self.toggle_menu_section(*id);
            }
            if drag_started {
                self.begin_section_drag(*id);
            }
            if let Some(delta) = move_by {
                self.move_menu_section(*id, delta);
            }
            if dock_left {
                self.dock_menu_section(*id, DockSide::Left);
                continue;
            }
            if float {
                let pointer = ui.max_rect().center() + Vec2::new(-40. * index as f32, 40.);
                self.float_menu_section(
                    *id,
                    FloatRect {
                        x: pointer.x - crate::panel_ops::FLOAT_DROP_GRAB[0],
                        y: pointer.y - crate::panel_ops::FLOAT_DROP_GRAB[1],
                        width: crate::panel_ops::FLOAT_DROP_SIZE[0],
                        height: crate::panel_ops::FLOAT_DROP_SIZE[1],
                    },
                );
                continue;
            }
            if self.dock.section(*id).collapsed {
                continue;
            }
            self.sidebar_section_ui(ui, *id);
        }
        self.right_content_bottom = ui.min_rect().bottom();
    }
    /// Floating menus live in owned windows whose rectangle is layout state.
    pub(super) fn float_menu_windows(&mut self, ctx: &egui::Context) {
        for id in self.dock.floating_sections() {
            let state = self.dock.section(id);
            if !state.visible {
                continue;
            }
            let stored = state
                .float
                .unwrap_or_else(FloatRect::command_search_default);
            let mut open = true;
            let mut dock_left = false;
            let mut dock_right = false;
            let work = self.work_area.map(|area| {
                egui::Rect::from_min_size(
                    egui::pos2(area.x, area.y),
                    Vec2::new(area.width, area.height),
                )
            });
            // The framework uses a pale active window header. Keep its title
            // dark, with a cream inactive header for the same clear contrast.
            let mut builder = egui::Window::new(RichText::new(id.title()).color(PANEL_BG))
                .id(egui::Id::new(("om9-float-menu", id.key())))
                .default_pos([stored.x, stored.y])
                .default_size([stored.width, stored.height])
                .frame(
                    egui::Frame::window(&ctx.style_of(egui::Theme::Light))
                        .fill(PANEL_BG)
                        .stroke(Stroke::new(1., PANEL_EDGE)),
                )
                .title_frame(
                    egui::Frame::window(&ctx.style_of(egui::Theme::Light))
                        .fill(PANEL_INK)
                        .stroke(Stroke::new(1., PANEL_EDGE)),
                )
                .open(&mut open);
            if let Some(work) = work {
                builder = builder.constrain_to(work);
            }
            let window = builder.show(ctx, |ui| {
                apply_dock_theme(ui);
                ui.horizontal(|ui| {
                    dock_left = ui
                        .small_button("◀ Dock left")
                        .on_hover_text("Return this menu to the left column")
                        .clicked();
                    dock_right = ui
                        .small_button("Dock right ▶")
                        .on_hover_text("Dock this menu to the right mirror column")
                        .clicked();
                });
                self.sidebar_section_ui(ui, id);
            });
            if let Some(window) = window {
                let rect = window.response.rect;
                if (rect.min.x - stored.x).abs() > 0.5
                    || (rect.min.y - stored.y).abs() > 0.5
                    || (rect.width() - stored.width).abs() > 0.5
                    || (rect.height() - stored.height).abs() > 0.5
                {
                    // The readback goes through the same door as every other
                    // placement operation (recovery included).
                    self.readback_float(
                        id,
                        FloatRect {
                            x: rect.min.x,
                            y: rect.min.y,
                            width: rect.width(),
                            height: rect.height(),
                        },
                    );
                }
            }
            if dock_left {
                self.dock_menu_section(id, DockSide::Left);
            } else if dock_right {
                self.dock_menu_section(id, DockSide::Right);
            } else if !open {
                // Closing a floating menu returns it to the documented home
                // column instead of losing it.
                self.close_float_menu(id);
            }
        }
    }
    /// Always-visible status/error line. It shows the current state, keeps an
    /// unacknowledged failure on screen and never consumes Escape.
    pub(super) fn status_bar_ui(&mut self, ui: &mut egui::Ui) {
        let (kind, message) = (self.status.kind(), self.status.message().to_owned());
        let color = match kind {
            StatusKind::Ready => Color32::from_rgb(206, 228, 206),
            StatusKind::Info => Color32::WHITE,
            StatusKind::Warning => Color32::from_rgb(255, 226, 150),
            StatusKind::Error => Color32::from_rgb(255, 168, 158),
        };
        let summary = self.dock_summary();
        // Width budget, left to right: message (priority), summary (elided to
        // what is left), Dismiss (fixed, only for a failure). Both labels are
        // truncated to their allotted width and keep the full text on hover.
        const MIN_SUMMARY_WIDTH: f32 = 48.;
        let row = ui.available_rect_before_wrap();
        let spacing = ui.spacing().item_spacing.x;
        let mut right_edge = row.max.x;
        if kind.is_failure() {
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(row)
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
                |ui| {
                    let dismiss = ui.small_button("Dismiss");
                    right_edge = dismiss.rect.min.x - spacing;
                    self.status_dismiss_rect = Some(dismiss.rect);
                    if dismiss.clicked() {
                        self.status.acknowledge();
                    }
                },
            );
        }
        let message_max_x = (right_edge - MIN_SUMMARY_WIDTH).max(row.min.x);
        let message_text = format!("{}: {message}", kind.label());
        // The line shows the current state; hovering it shows the recent ones, so
        // a diagnostic the user did not read at the moment it appeared is not lost
        // (OM9-L3-01-01-007: status_tail).
        let hover_text = {
            let tail = self.status.tail();
            let mut text = message_text.clone();
            if tail.len() > 1 {
                text.push_str("\n\nrecent:");
                for line in tail.iter().take(tail.len().saturating_sub(1)).rev() {
                    text.push('\n');
                    text.push_str(line);
                }
            }
            text
        };
        let message_rect = Rect::from_min_max(row.min, egui::pos2(message_max_x, row.max.y));
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(message_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                let response = ui
                    .add(
                        egui::Label::new(RichText::new(&message_text).color(color).size(11.))
                            .truncate(),
                    )
                    .on_hover_text(&hover_text);
                self.status_message_rect = Some(response.rect);
            },
        );
        let message_end = self
            .status_message_rect
            .map_or(message_max_x, |rect| rect.max.x);
        let summary_rect = Rect::from_min_max(
            egui::pos2((message_end + spacing).min(right_edge), row.min.y),
            egui::pos2(right_edge.max(row.min.x), row.max.y),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(summary_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
            |ui| {
                let response = ui
                    .add(
                        egui::Label::new(
                            RichText::new(&summary)
                                .color(Color32::from_gray(205))
                                .size(10.),
                        )
                        .truncate(),
                    )
                    .on_hover_text(&summary);
                self.status_summary_rect = Some(response.rect);
            },
        );
    }
}
fn category_palette(category: &str) -> &'static [&'static str] {
    crate::menu_catalog::category(category).unwrap_or(&[])
}
fn subd_flyout_items(name: &str) -> impl Iterator<Item = &'static &'static str> {
    let items: &'static [&'static str] = match name {
        "SubD Primitives" => &[
            "ClayooPrimitivesPlane",
            "ClayooPrimitivesBox",
            "ClayooPrimitivesTorus",
            "ClayooPrimitivesSphere",
            "ClayooPrimitivesCylinder",
            "ClayooPrimitivesCone",
        ],
        "SubD Creation" => &[
            "ClayooCreationAppendFace",
            "ClayooCreationFromCurve",
            "ClayooCreationFrom2Curves",
            "ClayooCreationLoft",
            "ClayooCreationSweep1Rail",
            "ClayooCreationSweep2Rails",
            "ClayooCreationPipe",
            "ClayooCreationRevolve",
            "ClayooCreateIsocurves",
            "ClayooCreationRing",
            "ClayooCreationSignetRing",
            "ClayooCreationClayooBezel",
            "ClayooCreationFromSurface",
            "ClayooCreationFromMesh",
            "ClayooCreationFromTSplines",
            "ClayooCreationToNurbs",
            "ClayooCreationClayooLibrary",
        ],
        "SubD Edition" => &[
            "ClayooEditDivide",
            "ClayooEditSplitSides",
            "ClayooEditMatch",
            "ClayooEditExtract",
            "ClayooEditOffset",
            "ClayooEditInset",
            "ClayooEditExtrude",
            "ClayooEditWeld",
            "ClayooEditCollapse",
            "ClayooEditBridge",
            "ClayooEditFill",
            "ClayooEditShell",
            "ClayooEditKnife",
            "ClayooEditWrap",
            "ClayooEditSymmetry",
            "ClayooEditFlipNormals",
            "ClayooEditCrease",
            "ClayooEditProjectToPlane",
            "ClayooEditSubdivisionLevel",
            "ClayooEditItem",
        ],
        "SubD Selection" => &[
            "ClayooSelectionSelectAll",
            "ClayooSelectionSelectNone",
            "ClayooSelectionInvertSelection",
            "ClayooSelectionSelectCreased",
            "ClayooSelectionSelectNaked",
            "ClayooSelectionSelectCoplanar",
            "ClayooSelectionPaintSelection",
            "ClayooSelectionGrowSelection",
            "ClayooSelectionShrink",
            "ClayooSelectUV",
            "ClayooSelectionSelectRing",
            "ClayooSelectionSelectLoop",
            "ClayooSelectionSetPlane",
            "ClayooSelectionResetPlane",
            "ClayooSelectionSelectionSets",
        ],
        _ => &[],
    };
    items.iter()
}
fn subd_label(icon: &str) -> String {
    let name = icon
        .strip_prefix("ClayooPrimitives")
        .or_else(|| icon.strip_prefix("ClayooCreation"))
        .or_else(|| icon.strip_prefix("ClayooSelection"))
        .or_else(|| icon.strip_prefix("ClayooEdit"))
        .or_else(|| icon.strip_prefix("Clayoo"))
        .unwrap_or(icon);
    let mut result = String::new();
    for ch in name.chars() {
        if ch.is_ascii_uppercase() && !result.is_empty() {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}
fn toolbar_palette(tab: &str) -> &'static [&'static str] {
    match tab {
        "Standard" => &[
            "New",
            "Open",
            "Save",
            "FilePrint",
            "Standard Document Properties",
            "Standard Cut",
            "Standard Copy",
            "Standard Paste",
            "Undo",
            "Standard Pan",
            "Standard Rotate View",
            "ViewZoomZoomDynamic",
            "ViewZoomZoomWindow",
            "Fit",
            "ViewZoomZoomSelected",
            "Standard Undo View",
            "Standard Four Views",
            "Standard Right View",
            "Standard CPlane Origin",
            "Standard Object Snaps",
            "Standard Select All",
            "Standard Hide Objects",
            "Standard Lock Objects",
            "Layers",
            "ObjectProperties",
            "Standard Shade",
            "Shaded",
            "Standard Render",
            "Standard Spotlight",
            "RhinoOptions",
            "Standard Linear Dimension",
            "Standard Help",
            "Redo",
            "Line",
            "Point",
            "Circle",
            "Rectangle",
        ],
        "Set View" => &[
            "ViewZoomZoomDynamic",
            "Fit",
            "ViewZoomZoomWindow",
            "ViewZoomZoomSelected",
            "ViewZoomZoom1To1",
            "ViewManager",
        ],
        "Display" => &[
            "Wireframe",
            "Shaded",
            "Standard Shade",
            "Display Rendered Viewport",
            "Dial_Ghosted",
            "Display X-Ray Viewport",
            "Dial_Tech Shade",
            "Display Pen Viewport",
            "Display Artistic Viewport",
            "Display Render Mesh Settings",
            "Display Flat Shade",
            "ShadeSelectedOnlyON",
            "Display Object Shading",
            "ViewCaptureToFile",
            "Visibility Add Clipping Plane",
            "Visibility Disable Clipping Plane",
            "Display Panel",
        ],
        "Visibility" => &[
            "Visibility Hide Objects",
            "Visibility Show Objects",
            "Visibility Show Selected Objects",
            "Visibility Invert Selection and Hide",
            "Visibility Swap Hidden Objects",
            "Visibility Lock Objects",
            "Visibility Unlock Objects",
            "Visibility Unlock Selected Objects",
            "Visibility Invert Selection and Lock",
            "Visibility Swap Locked Objects",
            "Visibility Invert and Hide Control Points",
            "Visibility Hide Control Points",
            "Visibility Add Clipping Plane",
            "Visibility Disable Clipping Plane",
            "Visibility Hide A",
            "Visibility Hide B",
            "Visibility Hide C",
            "Visibility Hide Custom",
        ],
        "Transform" => category_palette("Transform"),
        "Curve Tools" => category_palette("Curve"),
        "Surface Tools" => category_palette("Surface"),
        "Solid Tools" => category_palette("Solid"),
        "Mesh Tools" => &[
            "MeshMeshRepairToolsAlignMeshVertices",
            "MeshMeshRepairToolsFillHole",
            "MeshMeshRepairToolsFillHoles",
            "MeshApplyMeshUVN",
            "MeshMeshEditToolsCollapseReduceVertexCount",
            "MeshMeshEditToolsExtractRenderMesh",
            "MeshFromNURBSObject",
        ],
        "Render Tools" => category_palette("Render"),
        "Drafting" => &[
            "OthersMeasureDimHorizontal",
            "OthersMeasureDimVertical",
            "DimensionAlignedDimension",
            "DimensionRotatedDimension",
            "DimensionAngleDimension",
            "DimensionRadialDimension",
            "DimensionDiameterDimension",
            "DimensionTextBlock",
            "DimensionLeader",
            "DimensionRecenterDimensionText",
            "DimensionMake2DDrawing",
        ],
        "SubD" => &[
            "SubD Edit Mode",
            "SubD Primitives",
            "SubD Creation",
            "SubD Edition",
            "SubD Selection",
            "SubD Hotkeys Settings",
            "SubD Settings",
            "SubD Help",
            "SubD Report Feedback",
            "SubD Elements",
        ],
        "Emboss" => &[
            "Clay Emboss",
            "Emboss Resources",
            "Emboss Texture Creator",
            "Emboss Raster to Vector",
            "Emboss Text on Curve",
            "Emboss Decimator",
        ],
        "Sculpt" => &[
            "Sculpt",
            "Sculpt Brush",
            "Sculpt Crease",
            "Sculpt Flatten",
            "Sculpt Inflate",
            "Sculpt Pinch",
            "Sculpt Smooth",
            "Sculpt Twist",
            "Sculpt Drag",
        ],
        "Tools" => category_palette("Tools"),
        "Settings" => category_palette("Setting"),
        "Top 11" => &[
            "Duplicate",
            "EditPoints",
            "ControlPoints",
            "Mirror",
            "Move",
            "Rotate",
            "Explode",
            "Join",
            "Split",
            "Trim",
            "RingRail",
        ],
        _ => &[],
    }
}
fn feature_category(f: &Feature) -> &str {
    if f.id.starts_with("OM9-CURVE-") {
        "Curve"
    } else if f.id.starts_with("OM9-SURFACE-") {
        "Surface"
    } else if f.id.starts_with("OM9-SOLID-") {
        "Solid"
    } else if f.id.starts_with("OM9-TRANSFORM-") {
        "Transform"
    } else if f.id.starts_with("OM9-MESH-") {
        "Mesh"
    } else if f.id.starts_with("OM9-RENDER-") {
        "Render"
    } else if f.id.starts_with("OM9-INFO-") {
        "Analyze"
    } else {
        "Tools"
    }
}
fn menu_page_button(ui: &mut egui::Ui, index: usize, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(22., 14.), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, 2, Color32::from_rgb(0, 24, 12));
    let inner = rect.shrink(1.5);
    let (top, bottom) = if selected {
        (
            Color32::from_rgb(52, 164, 96),
            Color32::from_rgb(21, 105, 58),
        )
    } else if response.hovered() {
        (Color32::from_rgb(0, 58, 30), Color32::from_rgb(0, 42, 22))
    } else {
        (Color32::from_rgb(0, 45, 22), Color32::from_rgb(0, 30, 15))
    };
    gradient(painter, inner, top, bottom);
    painter.rect_stroke(
        inner,
        2,
        Stroke::new(
            1.,
            if selected {
                Color32::from_rgb(190, 245, 210)
            } else {
                Color32::from_rgb(80, 150, 105)
            },
        ),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            egui::pos2(inner.left() + 2., inner.top() + 1.),
            egui::pos2(inner.right() - 2., inner.top() + 1.),
        ],
        Stroke::new(1., Color32::from_rgba_unmultiplied(255, 255, 255, 165)),
    );
    painter.line_segment(
        [
            egui::pos2(inner.left() + 1., inner.top() + 2.),
            egui::pos2(inner.left() + 1., inner.bottom() - 2.),
        ],
        Stroke::new(1., Color32::from_rgba_unmultiplied(255, 255, 255, 120)),
    );
    if selected {
        painter.line_segment(
            [
                egui::pos2(inner.center().x, inner.center().y - 4.),
                egui::pos2(inner.center().x, inner.center().y + 4.),
            ],
            Stroke::new(2., Color32::from_rgb(4, 38, 20)),
        );
    } else {
        painter.circle_stroke(
            inner.center(),
            2.6,
            Stroke::new(1., Color32::from_rgb(143, 227, 174)),
        );
    }
    response.on_hover_text(format!("Page {}", index + 1))
}

fn category_text(accent: Color32) -> Color32 {
    let luminance =
        (0.2126 * accent.r() as f32 + 0.7152 * accent.g() as f32 + 0.0722 * accent.b() as f32)
            / 255.;
    let mix = if luminance < 0.42 { 0.65 } else { 0.40 };
    let channel = |value: u8| (value as f32 * (1. - mix) + 255. * mix) as u8;
    Color32::from_rgb(
        channel(accent.r()),
        channel(accent.g()),
        channel(accent.b()),
    )
}

fn section_accent(id: SectionId) -> Color32 {
    match id {
        SectionId::IconHistory => CATEGORY_COLORS[0],
        SectionId::MainMenu => CATEGORY_COLORS[1],
        SectionId::Display => CATEGORY_COLORS[4],
        SectionId::Snaps => CATEGORY_COLORS[2],
        SectionId::InfoSettings => CATEGORY_COLORS[3],
        SectionId::Layers => CATEGORY_COLORS[5],
        SectionId::Projects => CATEGORY_COLORS[0],
        SectionId::BuilderSlot => CATEGORY_COLORS[3],
    }
}

fn gradient(p: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    for i in 0..12 {
        let t = i as f32 / 11.;
        let c = Color32::from_rgb(
            (top.r() as f32 * (1. - t) + bottom.r() as f32 * t) as u8,
            (top.g() as f32 * (1. - t) + bottom.g() as f32 * t) as u8,
            (top.b() as f32 * (1. - t) + bottom.b() as f32 * t) as u8,
        );
        let y = rect.top() + rect.height() * i as f32 / 12.;
        p.rect_filled(
            Rect::from_min_max(
                egui::pos2(rect.left(), y),
                egui::pos2(
                    rect.right(),
                    rect.top() + rect.height() * (i + 1) as f32 / 12.,
                ),
            ),
            0,
            c,
        );
    }
}
fn bevel(p: &egui::Painter, rect: Rect) {
    p.line_segment(
        [rect.left_bottom(), rect.left_top()],
        Stroke::new(1., Color32::from_gray(175)),
    );
    p.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1., Color32::from_gray(175)),
    );
    p.line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1., Color32::from_gray(38)),
    );
    p.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1., Color32::from_gray(38)),
    );
}
#[cfg(test)]
mod chrome_tests {
    use super::*;

    fn narrow_frame(app: &mut MatrixApp, sidebar: bool) -> egui::FullOutput {
        let context = egui::Context::default();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(240., 900.))),
                ..Default::default()
            },
            |ui| {
                app.configure_style(&context);
                ui.set_style(context.style_of(egui::Theme::Light));
                let right = ui.max_rect().right();
                if sidebar {
                    app.sidebar_ui(ui);
                } else {
                    app.toolbars_ui(ui);
                }
                assert!(
                    ui.min_rect().right() <= right + 0.5,
                    "chrome escaped narrow allocation: {:?}",
                    ui.min_rect()
                );
            },
        );
        output.textures_delta.clear();
        output
    }

    #[test]
    fn sidebar_wrapping_and_toolbar_scrolling_keep_a_240px_allocation() {
        let mut app = MatrixApp::new(Document::new("narrow chrome")).unwrap();
        narrow_frame(&mut app, true);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Curve Tools").unwrap();
        narrow_frame(&mut app, false);
        assert!(!app.controller.is_running());
        assert!(app.controller.document().entities.is_empty());
    }

    fn window_frame(
        app: &mut MatrixApp,
        context: &egui::Context,
        size: Vec2,
        events: Vec<egui::Event>,
    ) {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }

    #[test]
    fn native_brep_top11_buttons_are_visible_but_not_clickable() {
        use om9_core::{
            geometry::{Geometry, Mesh, NativeBrep},
            model::Entity,
        };
        let entity = Entity::new(
            "native",
            Geometry::NativeBrep(NativeBrep {
                display: Mesh {
                    vertices: vec![[0., 0., 0.], [2., 0., 0.], [0., 2., 0.]],
                    faces: vec![vec![0, 1, 2]],
                    normals: vec![],
                    colors: vec![],
                    corner_uvs: vec![],
                },
                payload: vec![1, 2, 3].into(),
                crc: 0,
            }),
            None,
        );
        let id = entity.id;
        let mut doc = Document::new("native toolbar");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        let context = egui::Context::default();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(600., 400.))),
                ..Default::default()
            },
            |ui| {
                let split = app.top11_icon(ui, "Split");
                assert!(split.rect.is_positive());
                assert_eq!(split.sense, egui::Sense::hover());
                let explode = app.top11_icon(ui, "Explode");
                assert!(explode.rect.is_positive());
                assert_eq!(
                    explode.sense,
                    if cfg!(feature = "occt-brep") {
                        egui::Sense::click()
                    } else {
                        egui::Sense::hover()
                    }
                );
                let mirror = app.top11_icon(ui, "Mirror");
                assert_eq!(mirror.sense, egui::Sense::click());
            },
        );
        output.textures_delta.clear();
    }

    #[test]
    fn top11_transform_preview_is_painted_by_egui() {
        use om9_core::{geometry::Geometry, model::Entity, top11::Top11Action};
        for action in [Top11Action::Mirror, Top11Action::Rotate] {
            let source = Entity::new("point", Geometry::Point([2., 1., 0.]), None);
            let id = source.id;
            let mut doc = Document::new("preview paint");
            doc.entities.insert(id, source);
            let mut app = MatrixApp::new(doc).unwrap();
            let context = egui::Context::default();
            window_frame(&mut app, &context, Vec2::new(1400., 900.), vec![]);
            app.controller.select_objects(&[id]).unwrap();
            app.controller.top11_invoke(action).unwrap();
            app.controller.accept_world_point([0.; 3]).unwrap();
            if action == Top11Action::Rotate {
                app.controller.accept_world_point([1., 0., 0.]).unwrap();
            }
            app.hover = Some([0., 3., 0.]);
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1400., 900.),
                    )),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            let painted = output.shapes.iter().any(|shape| {
                matches!(&shape.shape,
                egui::Shape::Circle(circle) if circle.fill == egui::Color32::from_rgb(0, 200, 130))
            });
            output.textures_delta.clear();
            assert!(painted, "missing {action:?} preview paint");
        }
    }

    #[test]
    fn top11_move_preview_uses_projected_snap_point() {
        use om9_core::{geometry::Geometry, model::Entity, top11::Top11Action};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        for project in [false, true] {
            for [x, y] in [[3., 2.], [-3., 2.], [3., -2.], [-3., -2.]] {
                let entity = Entity::new("source", Geometry::Point([-4., -4., 0.]), None);
                let id = entity.id;
                let target = Entity::new("snap", Geometry::Point([x, y, 5.]), None);
                let target_id = target.id;
                let mut doc = Document::new("projected preview");
                doc.entities.insert(id, entity);
                doc.entities.insert(target_id, target);
                let mut app = MatrixApp::new(doc).unwrap();
                app.controller.select_objects(&[id]).unwrap();
                app.controller.workspace.active = ViewId::LookingDown;
                app.controller.set_project(project);
                app.controller.top11_invoke(Top11Action::Move).unwrap();
                app.controller.accept_world_point([0.; 3]).unwrap();
                let context = egui::Context::default();
                let size = Vec2::new(1400., 900.);
                for _ in 0..3 {
                    window_frame(&mut app, &context, size, vec![]);
                }
                let top_rect = app.view_rects[&ViewId::LookingDown];
                let top_viewport = Viewport {
                    origin: [top_rect.min.x as f64, top_rect.min.y as f64],
                    size: [top_rect.width() as f64, top_rect.height() as f64],
                };
                let top_camera = app
                    .controller
                    .view_cameras()
                    .into_iter()
                    .find(|(title, _, _)| *title == "Looking Down")
                    .unwrap()
                    .1;
                let cursor = top_camera
                    .project([x, y, 5.], top_viewport)
                    .unwrap()
                    .unwrap()
                    .screen;
                let cursor = egui::pos2(cursor[0] as f32, cursor[1] as f32);
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(cursor)],
                );
                assert_eq!(app.snap.as_ref().and_then(|s| s.root), Some(target_id));
                let rect = app.view_rects[&ViewId::ThroughFinger];
                let viewport = Viewport {
                    origin: [rect.min.x as f64, rect.min.y as f64],
                    size: [rect.width() as f64, rect.height() as f64],
                };
                let camera = app
                    .controller
                    .view_cameras()
                    .into_iter()
                    .find(|(title, _, _)| *title == "Through Finger")
                    .unwrap()
                    .1;
                let expected = camera
                    .project([x - 4., y - 4., if project { 0. } else { 5. }], viewport)
                    .unwrap()
                    .unwrap()
                    .screen;
                let mut output = context.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ui| app.show(ui),
                );
                let painted = output.shapes.iter().any(|shape| {
                    matches!(&shape.shape, egui::Shape::Circle(circle)
                if circle.fill == egui::Color32::from_rgb(0, 200, 130)
                    && (circle.center.x as f64 - expected[0]).abs() < 0.1
                    && (circle.center.y as f64 - expected[1]).abs() < 0.1)
                });
                output.textures_delta.clear();
                assert!(
                    painted,
                    "preview is not at projected snap point, Project={project}"
                );
                window_frame(&mut app, &context, size, pointer(cursor, true));
                window_frame(&mut app, &context, size, pointer(cursor, false));
                let entity = &app.controller.document().entities[&id];
                let Geometry::Point(point) = entity.geometry else {
                    panic!("point")
                };
                let actual = entity.placement.point(point).unwrap();
                let wanted = [x - 4., y - 4., if project { 0. } else { 5. }];
                assert!(
                    (0..3).all(|i| (actual[i] - wanted[i]).abs() < 1e-8),
                    "Project={project}: {actual:?}"
                );
            }
        }
    }

    #[test]
    fn top11_move_uses_hovered_view_cplane_before_and_after_click() {
        use om9_core::{geometry::Geometry, model::Entity, top11::Top11Action};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        for action in [Top11Action::Move, Top11Action::Duplicate] {
            for [x, z] in [[3., 2.], [-3., 2.], [3., -2.], [-3., -2.]] {
                let entity = Entity::new("source", Geometry::Point([0.; 3]), None);
                let id = entity.id;
                let mut doc = Document::new("inactive viewport move");
                doc.entities.insert(id, entity);
                let mut app = MatrixApp::new(doc).unwrap();
                app.controller.workspace.active = ViewId::LookingDown;
                app.controller.select_objects(&[id]).unwrap();
                app.controller.top11_invoke(action).unwrap();
                app.controller.accept_world_point([0.; 3]).unwrap();
                app.snap_state.settings.enabled = 0;
                let context = egui::Context::default();
                let size = Vec2::new(1400., 900.);
                for _ in 0..3 {
                    window_frame(&mut app, &context, size, vec![]);
                }
                let rect = app.view_rects[&ViewId::ThroughFinger];
                let viewport = Viewport {
                    origin: [rect.min.x as f64, rect.min.y as f64],
                    size: [rect.width() as f64, rect.height() as f64],
                };
                let camera = app
                    .controller
                    .view_cameras()
                    .into_iter()
                    .find(|(title, _, _)| *title == "Through Finger")
                    .unwrap()
                    .1;
                let target = [x, 0., z];
                let p = camera.project(target, viewport).unwrap().unwrap().screen;
                let cursor = egui::pos2(p[0] as f32, p[1] as f32);
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(cursor)],
                );
                let hover = app
                    .hover
                    .expect("hover point on the pointed viewport CPlane");
                assert!(
                    (0..3).all(|i| (hover[i] - target[i]).abs() < 1e-4),
                    "hover={hover:?}, target={target:?}"
                );
                assert_eq!(app.controller.workspace.active, ViewId::LookingDown);
                let mut output = context.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ui| app.show(ui),
                );
                let painted = output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Circle(circle)
                if circle.fill == egui::Color32::from_rgb(0, 200, 130) && circle.center.distance(cursor) < 0.1));
                output.textures_delta.clear();
                assert!(painted, "preview must be under the pointer: {target:?}");
                window_frame(&mut app, &context, size, pointer(cursor, true));
                window_frame(&mut app, &context, size, pointer(cursor, false));
                if action == Top11Action::Duplicate {
                    app.controller.submit("").unwrap();
                }
                let result = if action == Top11Action::Duplicate {
                    app.controller
                        .document()
                        .entities
                        .values()
                        .find(|entity| entity.id != id)
                        .unwrap()
                } else {
                    &app.controller.document().entities[&id]
                };
                let actual = result.placement.point([0.; 3]).unwrap();
                assert!(
                    (0..3).all(|i| (actual[i] - target[i]).abs() < 1e-4),
                    "actual={actual:?}, target={target:?}"
                );
                assert_eq!(app.controller.workspace.active, ViewId::ThroughFinger);
            }
        }
    }

    #[test]
    fn top11_rotation_and_mirror_preview_use_hovered_view_plane() {
        use om9_core::{geometry::Geometry, model::Entity, top11::Top11Action};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        for action in [Top11Action::Mirror, Top11Action::Rotate] {
            let entity = Entity::new("source", Geometry::Point([1., 0., 0.]), None);
            let id = entity.id;
            let mut doc = Document::new("inactive view transform");
            doc.entities.insert(id, entity);
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.workspace.active = ViewId::LookingDown;
            app.controller.select_objects(&[id]).unwrap();
            app.controller.top11_invoke(action).unwrap();
            app.controller.submit("Copy=No").unwrap();
            app.controller.accept_world_point([0.; 3]).unwrap();
            if action == Top11Action::Rotate {
                app.controller.accept_world_point([1., 0., 0.]).unwrap();
            }
            app.snap_state.settings.enabled = 0;
            let context = egui::Context::default();
            let size = Vec2::new(1400., 900.);
            for _ in 0..3 {
                window_frame(&mut app, &context, size, vec![]);
            }
            let rect = app.view_rects[&ViewId::ThroughFinger];
            let viewport = Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(title, _, _)| *title == "Through Finger")
                .unwrap()
                .1;
            let screen = |p| {
                let q = camera.project(p, viewport).unwrap().unwrap().screen;
                egui::pos2(q[0] as f32, q[1] as f32)
            };
            let cursor = screen([0., 0., 3.]);
            let expected = if action == Top11Action::Mirror {
                [-1., 0., 0.]
            } else {
                [0., 0., 1.]
            };
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(cursor)],
            );
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            let painted = output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Circle(circle)
                if circle.fill == egui::Color32::from_rgb(0, 200, 130) && circle.center.distance(screen(expected)) < 0.1));
            output.textures_delta.clear();
            assert!(painted, "{action:?} preview");
            window_frame(&mut app, &context, size, pointer(cursor, true));
            window_frame(&mut app, &context, size, pointer(cursor, false));
            let entity = &app.controller.document().entities[&id];
            let Geometry::Point(point) = entity.geometry else {
                panic!("point")
            };
            let actual = entity.placement.point(point).unwrap();
            assert!(
                (0..3).all(|i| (actual[i] - expected[i]).abs() < 1e-4),
                "{action:?} {actual:?}"
            );
        }
    }

    #[test]
    fn top11_parallel_view_ray_cannot_reuse_stale_hover() {
        use om9_core::{geometry::Geometry, model::Entity, top11::Top11Action};
        use om9_scene::{
            camera::{CPlane, CPlanePreset},
            workspace::ViewId,
        };
        let entity = Entity::new("source", Geometry::Point([0.; 3]), None);
        let id = entity.id;
        let mut doc = Document::new("parallel view");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller
            .workspace
            .views
            .get_mut(&ViewId::ThroughFinger)
            .unwrap()
            .cplane = CPlane::preset(CPlanePreset::Top);
        app.controller.top11_invoke(Top11Action::Move).unwrap();
        app.controller.accept_world_point([0.; 3]).unwrap();
        let before = app.controller.document().clone();
        let context = egui::Context::default();
        let size = Vec2::new(1400., 900.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        app.hover = Some([5., 5., 0.]);
        app.hover_view = Some(ViewId::LookingDown);
        let cursor = app.view_rects[&ViewId::ThroughFinger].center();
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::PointerMoved(cursor)],
        );
        assert!(app.hover.is_none());
        window_frame(&mut app, &context, size, pointer(cursor, true));
        window_frame(&mut app, &context, size, pointer(cursor, false));
        assert_eq!(app.controller.document(), &before);
        assert!(app.controller.is_running());
    }

    #[test]
    fn top11_inactive_view_projected_snap_matches_preview_and_commit() {
        use om9_core::{geometry::Geometry, model::Entity, top11::Top11Action};
        use om9_scene::{camera::Viewport, workspace::ViewId};
        for project in [false, true] {
            let source = Entity::new("source", Geometry::Point([-4., 0., -4.]), None);
            let id = source.id;
            let snap = Entity::new("snap", Geometry::Point([3., 2., 5.]), None);
            let snap_id = snap.id;
            let mut doc = Document::new("inactive view project snap");
            doc.entities.insert(id, source);
            doc.entities.insert(snap_id, snap);
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.workspace.active = ViewId::LookingDown;
            app.controller.set_project(project);
            app.controller.select_objects(&[id]).unwrap();
            app.controller.top11_invoke(Top11Action::Move).unwrap();
            app.controller.accept_world_point([0.; 3]).unwrap();
            let context = egui::Context::default();
            let size = Vec2::new(1400., 900.);
            for _ in 0..3 {
                window_frame(&mut app, &context, size, vec![]);
            }
            let screen = |app: &MatrixApp, view: ViewId, p| {
                let rect = app.view_rects[&view];
                let viewport = Viewport {
                    origin: [rect.min.x as f64, rect.min.y as f64],
                    size: [rect.width() as f64, rect.height() as f64],
                };
                let camera = app
                    .controller
                    .view_cameras()
                    .into_iter()
                    .find(|(title, _, _)| *title == view.title())
                    .unwrap()
                    .1;
                let q = camera.project(p, viewport).unwrap().unwrap().screen;
                egui::pos2(q[0] as f32, q[1] as f32)
            };
            let cursor = screen(&app, ViewId::ThroughFinger, [3., 2., 5.]);
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(cursor)],
            );
            assert_eq!(app.snap.as_ref().and_then(|s| s.root), Some(snap_id));
            let expected = [-1., if project { 0. } else { 2. }, 1.];
            let expected_screen = screen(&app, ViewId::SideView, expected);
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            let painted = output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Circle(circle)
                if circle.fill == egui::Color32::from_rgb(0, 200, 130) && circle.center.distance(expected_screen) < 0.1));
            output.textures_delta.clear();
            assert!(painted, "Project={project} preview");
            window_frame(&mut app, &context, size, pointer(cursor, true));
            window_frame(&mut app, &context, size, pointer(cursor, false));
            let entity = &app.controller.document().entities[&id];
            let Geometry::Point(point) = entity.geometry else {
                panic!("point")
            };
            let actual = entity.placement.point(point).unwrap();
            assert!(
                (0..3).all(|i| (actual[i] - expected[i]).abs() < 1e-8),
                "Project={project}: {actual:?}"
            );
        }
    }

    fn pointer(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    #[test]
    fn dragging_a_menu_to_the_right_edge_docks_it_and_shows_the_hidden_dock() {
        let size = Vec2::new(1280., 720.);
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("drop zone")).unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        assert!(app.right_dock_rect.is_none());
        assert!(app.drop_preview.is_none());
        let before = app.view_rects.clone();
        let canvas = before.values().copied().reduce(|a, b| a.union(b)).unwrap();
        let at = egui::pos2(canvas.max.x - 6., canvas.center().y);

        // Drag in progress: the preview offers the right column without moving
        // any viewport.
        app.begin_section_drag(dock_layout::RIGHT_DOCK_FIXTURE);
        window_frame(&mut app, &context, size, pointer(at, true));
        let preview = app.drop_preview.expect("preview while dragging");
        assert_eq!(
            preview.target,
            crate::panel_ops::DropTarget::Place {
                side: crate::panel_ops::ColumnSide::Right,
                index: 0
            }
        );
        assert!(preview.indicator.is_some(), "the drop shows an indicator");
        assert!(app.right_dock_rect.is_none());
        assert_eq!(app.view_rects, before, "the overlay must not reflow");

        // Release inside the strip docks the menu right.
        window_frame(&mut app, &context, size, pointer(at, false));
        assert_eq!(
            app.dock.right_sections(),
            vec![dock_layout::RIGHT_DOCK_FIXTURE]
        );
        assert!(app.drop_preview.is_none());
        assert!(app.section_drag().is_none());
        window_frame(&mut app, &context, size, vec![]);
        assert!(app.right_dock_rect.is_some(), "dock appears the next frame");

        // The last menu leaving hides the dock again.
        assert!(app.dock_menu_section(dock_layout::RIGHT_DOCK_FIXTURE, DockSide::Left));
        window_frame(&mut app, &context, size, vec![]);
        window_frame(&mut app, &context, size, vec![]);
        assert!(app.right_dock_rect.is_none());
        assert!(app.dock.right_sections().is_empty());
    }

    #[test]
    fn a_window_under_1200_points_offers_no_dock_strip_and_floats_instead() {
        let size = Vec2::new(1100., 720.);
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("narrow drop")).unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let right_view = app
            .view_rects
            .values()
            .copied()
            .max_by(|a, b| a.max.x.total_cmp(&b.max.x))
            .unwrap();
        let at = egui::pos2(right_view.max.x - 6., right_view.center().y);
        app.begin_section_drag(dock_layout::RIGHT_DOCK_FIXTURE);
        window_frame(&mut app, &context, size, pointer(at, true));
        let preview = app.drop_preview.expect("preview while dragging");
        assert!(
            matches!(preview.target, crate::panel_ops::DropTarget::Float { .. }),
            "no strip under 1200 points: {:?}",
            preview.target
        );
        window_frame(&mut app, &context, size, pointer(at, false));
        assert!(app.dock.right_sections().is_empty());
        assert_eq!(
            app.dock.floating_sections(),
            vec![dock_layout::RIGHT_DOCK_FIXTURE]
        );
    }

    #[test]
    fn view_selection_requests_cmd_focus_without_erasing_numeric_input() {
        let mut app = MatrixApp::new(Document::new("view focus")).unwrap();
        app.controller
            .dispatch(LINE, InvocationOrigin::Cmd)
            .unwrap();
        app.transcript.set_draft("12.5,3,");
        app.focus_cmd = false;
        app.controller.workspace.maximized = Some(ViewId::LookingDown);
        app.activate_view(ViewId::SideView);
        assert!(app.focus_cmd);
        assert_eq!(app.transcript.draft(), "12.5,3,");
        assert_eq!(app.controller.workspace.active, ViewId::SideView);
        assert_eq!(app.controller.workspace.maximized, Some(ViewId::SideView));
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }

    #[test]
    fn top11_tab_lists_the_eleven_action_icons() {
        assert!(TABS.contains(&"Top 11"));
        let palette = toolbar_palette("Top 11");
        assert_eq!(palette.len(), 11);
        for action in om9_core::top11::Top11Action::ALL {
            assert!(palette.contains(&action.icon()), "{action:?}");
        }
    }

    #[test]
    fn added_toolbar_groups_use_real_authored_icon_bindings() {
        let mut app = MatrixApp::new(Document::new("toolbar groups")).unwrap();
        for tab in [
            "Set View",
            "Display",
            "Visibility",
            "Mesh Tools",
            "Drafting",
            "SubD",
            "Emboss",
            "Sculpt",
        ] {
            let palette = toolbar_palette(tab);
            assert!(!palette.is_empty(), "{tab}");
            for icon in palette {
                assert!(crate::icons::icon_name(icon).is_some(), "{tab}: {icon}");
            }
            app.toolbar_tab = TABS.iter().position(|item| *item == tab).unwrap();
            narrow_frame(&mut app, false);
        }
        assert_eq!(
            toolbar_palette("Emboss"),
            &[
                "Clay Emboss",
                "Emboss Resources",
                "Emboss Texture Creator",
                "Emboss Raster to Vector",
                "Emboss Text on Curve",
                "Emboss Decimator",
            ]
        );
        assert_eq!(
            crate::icons::feature_for("Emboss Resources"),
            Some("OM9-EMBOSS-014")
        );
        assert_eq!(
            crate::icons::feature_for("Emboss Decimator"),
            Some("OM9-EMBOSS-015")
        );
        assert_eq!(
            toolbar_palette("Sculpt"),
            &[
                "Sculpt",
                "Sculpt Brush",
                "Sculpt Crease",
                "Sculpt Flatten",
                "Sculpt Inflate",
                "Sculpt Pinch",
                "Sculpt Smooth",
                "Sculpt Twist",
                "Sculpt Drag",
            ]
        );
        assert!(
            toolbar_palette("Sculpt")
                .iter()
                .all(|name| crate::icons::feature_for(name).is_none())
        );
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }

    #[test]
    fn top11_tab_renders_without_mutating_the_document() {
        let mut app = MatrixApp::new(Document::new("top11 chrome")).unwrap();
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Top 11").unwrap();
        narrow_frame(&mut app, false);
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }

    #[test]
    fn command_toolbar_slots_are_75_percent_of_previous_responsive_size() {
        let mut app = MatrixApp::new(Document::new("toolbar size")).unwrap();
        let context = egui::Context::default();
        for size in [Vec2::new(1280., 720.), Vec2::new(1920., 1080.)] {
            for _ in 0..3 {
                window_frame(&mut app, &context, size, vec![]);
            }
            let previous = (dock_layout::ICON_BASE_EDGE * app.screen_icon_scale)
                .clamp(dock_layout::ICON_MIN_EDGE, 32.)
                .round();
            assert!((app.toolbar_icon_edge - previous * 0.75).abs() < 0.01);
            let new = app.file_toolbar_rects[&crate::file_lifecycle::FileKind::New];
            assert!((new.width() - previous * 0.75).abs() < 0.01);
            assert!(((new.width() - 2.25) - (previous - 3.) * 0.75).abs() < 0.01);
        }
    }

    #[test]
    fn standard_toolbar_new_unavailable_icons_render_without_dispatch() {
        assert_eq!(
            &toolbar_palette("Standard")[..32],
            &[
                "New",
                "Open",
                "Save",
                "FilePrint",
                "Standard Document Properties",
                "Standard Cut",
                "Standard Copy",
                "Standard Paste",
                "Undo",
                "Standard Pan",
                "Standard Rotate View",
                "ViewZoomZoomDynamic",
                "ViewZoomZoomWindow",
                "Fit",
                "ViewZoomZoomSelected",
                "Standard Undo View",
                "Standard Four Views",
                "Standard Right View",
                "Standard CPlane Origin",
                "Standard Object Snaps",
                "Standard Select All",
                "Standard Hide Objects",
                "Standard Lock Objects",
                "Layers",
                "ObjectProperties",
                "Standard Shade",
                "Shaded",
                "Standard Render",
                "Standard Spotlight",
                "RhinoOptions",
                "Standard Linear Dimension",
                "Standard Help",
            ]
        );
        for name in &toolbar_palette("Standard")[..32] {
            assert!(
                crate::icons::icon_name(name).is_some(),
                "missing icon for {name}"
            );
        }
        let mut app = MatrixApp::new(Document::new("standard toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Standard").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for name in [
            "Standard Document Properties",
            "Standard Cut",
            "Standard Copy",
            "Standard Paste",
            "Standard Pan",
            "Standard Rotate View",
            "Standard Undo View",
            "Standard Four Views",
            "Standard Right View",
            "Standard CPlane Origin",
            "Standard Object Snaps",
            "Standard Select All",
            "Standard Hide Objects",
            "Standard Lock Objects",
            "Standard Shade",
            "Standard Render",
            "Standard Spotlight",
            "Standard Linear Dimension",
            "Standard Help",
        ] {
            assert!(crate::icons::icon_name(name).is_some(), "{name}");
            assert!(crate::icons::feature_for(name).is_none(), "{name}");
            let rect: Rect = context.memory(|m| {
                m.data
                    .get_temp(egui::Id::new(("toolbar-test", "Standard", &name)))
                    .unwrap()
            });
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert!(!app.controller.is_running(), "{name}");
            assert!(app.controller.document().entities.is_empty(), "{name}");
        }
    }

    #[test]
    fn visibility_toolbar_keeps_reference_order_and_unavailable_commands_inert() {
        let names = toolbar_palette("Visibility");
        assert_eq!(
            names,
            &[
                "Visibility Hide Objects",
                "Visibility Show Objects",
                "Visibility Show Selected Objects",
                "Visibility Invert Selection and Hide",
                "Visibility Swap Hidden Objects",
                "Visibility Lock Objects",
                "Visibility Unlock Objects",
                "Visibility Unlock Selected Objects",
                "Visibility Invert Selection and Lock",
                "Visibility Swap Locked Objects",
                "Visibility Invert and Hide Control Points",
                "Visibility Hide Control Points",
                "Visibility Add Clipping Plane",
                "Visibility Disable Clipping Plane",
                "Visibility Hide A",
                "Visibility Hide B",
                "Visibility Hide C",
                "Visibility Hide Custom",
            ]
        );
        let mut app = MatrixApp::new(Document::new("visibility toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Visibility").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for name in names {
            assert!(crate::icons::icon_name(name).is_some(), "{name}");
            assert!(crate::icons::feature_for(name).is_none(), "{name}");
            let rect: Rect = context.memory(|m| {
                m.data
                    .get_temp(egui::Id::new(("toolbar-test", "Visibility", name)))
                    .unwrap()
            });
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert!(!app.controller.is_running(), "{name}");
            assert!(app.controller.document().entities.is_empty(), "{name}");
        }
    }

    #[test]
    fn display_toolbar_switches_only_active_view_with_existing_view_owner() {
        use om9_scene::workspace::DisplayMode;
        assert_eq!(
            toolbar_palette("Display"),
            &[
                "Wireframe",
                "Shaded",
                "Standard Shade",
                "Display Rendered Viewport",
                "Dial_Ghosted",
                "Display X-Ray Viewport",
                "Dial_Tech Shade",
                "Display Pen Viewport",
                "Display Artistic Viewport",
                "Display Render Mesh Settings",
                "Display Flat Shade",
                "ShadeSelectedOnlyON",
                "Display Object Shading",
                "ViewCaptureToFile",
                "Visibility Add Clipping Plane",
                "Visibility Disable Clipping Plane",
                "Display Panel",
            ]
        );
        let mut app = MatrixApp::new(Document::new("display toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Display").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let target = app.controller.workspace.active;
        let original = app.controller.workspace.views.clone();
        for (name, mode) in [
            ("Shaded", DisplayMode::Shaded),
            ("Wireframe", DisplayMode::Wireframe),
        ] {
            let rect: Rect = context.memory(|m| {
                m.data
                    .get_temp(egui::Id::new(("toolbar-test", "Display", &name)))
                    .unwrap()
            });
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert_eq!(app.controller.workspace.views[&target].mode, mode, "{name}");
            for (id, view) in &original {
                if *id != target {
                    assert_eq!(app.controller.workspace.views[id], *view);
                }
            }
            assert_eq!(app.controller.coordinator().undo_len(), 0);
        }
    }

    #[test]
    fn display_toolbar_new_unavailable_buttons_are_inert() {
        let mut app = MatrixApp::new(Document::new("display disabled")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Display").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for name in [
            "Display Rendered Viewport",
            "Display X-Ray Viewport",
            "Display Pen Viewport",
            "Display Artistic Viewport",
            "Display Render Mesh Settings",
            "Display Flat Shade",
            "Display Object Shading",
            "Display Panel",
        ] {
            assert!(crate::icons::feature_for(name).is_none(), "{name}");
            let rect: Rect = context.memory(|m| {
                m.data
                    .get_temp(egui::Id::new(("toolbar-test", "Display", &name)))
                    .unwrap()
            });
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert!(!app.controller.is_running(), "{name}");
            assert!(app.controller.document().entities.is_empty(), "{name}");
        }
    }

    #[test]
    fn solid_unimplemented_rib_and_slab_remain_unavailable() {
        for name in ["SolidRib", "SolidSlab"] {
            assert!(crate::menu_catalog::SOLID.contains(&name), "{name}");
            let feature = crate::icons::feature_for(name).expect("palette binding");
            assert!(Controller::availability(feature).is_err(), "{name}");
        }
    }

    #[test]
    fn emboss_toolbar_unsupported_buttons_do_not_dispatch() {
        let mut app = MatrixApp::new(Document::new("emboss toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Standard").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Emboss").unwrap();
        window_frame(&mut app, &context, size, vec![]);
        let rects: Vec<Rect> = toolbar_palette("Emboss")
            .iter()
            .map(|name| {
                context.memory(|m| {
                    m.data
                        .get_temp(egui::Id::new(("toolbar-test", "Emboss", name)))
                        .unwrap()
                })
            })
            .collect();
        assert!(rects[1].left() - rects[0].right() > rects[2].left() - rects[1].right());
        for (name, rect) in toolbar_palette("Emboss").iter().zip(rects) {
            if let Some(feature) = crate::icons::feature_for(name) {
                assert!(Controller::availability(feature).is_err(), "{name}");
            }
            let at = rect.center();
            window_frame(&mut app, &context, size, pointer(at, true));
            window_frame(&mut app, &context, size, pointer(at, false));
            assert!(
                !app.controller.is_running(),
                "{name} dispatched unexpectedly"
            );
            assert!(app.controller.document().entities.is_empty(), "{name}");
            assert_eq!(app.controller.coordinator().undo_len(), 0, "{name}");
        }
    }

    #[test]
    fn sculpt_toolbar_retains_reference_separator() {
        let mut app = MatrixApp::new(Document::new("sculpt toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Sculpt").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rects: Vec<Rect> = toolbar_palette("Sculpt")
            .iter()
            .map(|name| {
                context.memory(|m| {
                    m.data
                        .get_temp(egui::Id::new(("toolbar-test", "Sculpt", name)))
                        .unwrap()
                })
            })
            .collect();
        assert!(rects[1].left() - rects[0].right() > rects[2].left() - rects[1].right());
    }

    #[test]
    fn subd_flyouts_preserve_catalog_and_open_from_toolbar() {
        let names = [
            "SubD Primitives",
            "SubD Creation",
            "SubD Edition",
            "SubD Selection",
        ];
        let all: Vec<_> = names
            .iter()
            .flat_map(|name| subd_flyout_items(name).copied())
            .collect();
        let original: std::collections::BTreeSet<_> =
            crate::menu_catalog::SUBD.iter().copied().collect();
        let grouped: std::collections::BTreeSet<_> = all.iter().copied().collect();
        assert_eq!(grouped, original);
        assert_eq!(
            all.len(),
            grouped.len(),
            "every original icon appears exactly once"
        );

        let mut app = MatrixApp::new(Document::new("subd toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "SubD").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for name in names {
            let rect: Rect = context.memory(|m| {
                m.data
                    .get_temp(egui::Id::new(("toolbar-test", "SubD", &name)))
                    .unwrap()
            });
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert!(egui::Popup::is_any_open(&context), "{name} opens a flyout");
            assert!(!app.controller.is_running());
            egui::Popup::close_all(&context);
            window_frame(&mut app, &context, size, vec![]);
        }
    }

    #[test]
    fn set_view_partial_toolbar_renders_and_unavailable_buttons_do_not_dispatch() {
        let mut app = MatrixApp::new(Document::new("set view toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Set View").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rects: Vec<Rect> = toolbar_palette("Set View")
            .iter()
            .map(|name| {
                context.memory(|m| {
                    m.data
                        .get_temp(egui::Id::new(("toolbar-test", "Set View", name)))
                        .unwrap()
                })
            })
            .collect();
        for (name, rect) in toolbar_palette("Set View").iter().zip(rects) {
            if let Some(feature) = crate::icons::feature_for(name) {
                assert!(Controller::availability(feature).is_err(), "{name}");
            }
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert!(!app.controller.is_running(), "{name}");
            assert!(app.controller.document().entities.is_empty(), "{name}");
        }
    }

    #[test]
    fn top11_ringrail_sidebar_click_builds_and_undoes_a_rail() {
        use om9_core::geometry::{Curve, Geometry};
        let mut app =
            MatrixApp::new(Document::with_template_tolerance("ring rail sidebar")).unwrap();
        let before = app.controller.document().clone();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let ring = context.memory(|memory| {
            memory
                .data
                .get_temp::<Rect>(egui::Id::new(("top11-sidebar", &"RingRail")))
                .expect("ring rail icon")
                .center()
        });
        window_frame(&mut app, &context, size, pointer(ring, true));
        window_frame(&mut app, &context, size, pointer(ring, false));
        assert!(app.controller.builder().is_some());
        app.controller.submit("Diameter=20").unwrap();
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let sidebar = app.sidebar_content_rect.unwrap().center();
        window_frame(
            &mut app,
            &context,
            size,
            vec![
                egui::Event::PointerMoved(sidebar),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: Vec2::new(0., -600.),
                    phase: egui::TouchPhase::Move,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        for _ in 0..4 {
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(sidebar)],
            );
        }
        let apply = context.memory(|memory| {
            memory
                .data
                .get_temp::<Rect>(egui::Id::new("builder-apply-test"))
                .expect("builder Apply")
                .center()
        });
        window_frame(&mut app, &context, size, pointer(apply, true));
        window_frame(&mut app, &context, size, pointer(apply, false));
        let after = app.controller.document().clone();
        assert_eq!(after.entities.len(), 1);
        assert_eq!(after.history.recipes.len(), 1);
        let Geometry::Curve(Curve::Circle { radius, .. }) =
            &after.entities.values().next().unwrap().geometry
        else {
            panic!("ring rail circle")
        };
        assert_eq!(*radius, 10.);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }

    #[test]
    fn top11_sidebar_is_one_ordered_row_above_display() {
        let mut app = MatrixApp::new(Document::new("top11 sidebar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let names = toolbar_palette("Top 11");
        assert_eq!(names.len(), 11);
        let rects: Vec<Rect> = names
            .iter()
            .map(|name| {
                context.memory(|memory| {
                    memory
                        .data
                        .get_temp(egui::Id::new(("top11-sidebar", name)))
                        .expect("sidebar icon")
                })
            })
            .collect();
        let sidebar = app.sidebar_content_rect.unwrap();
        let display = app.section_rect("display").unwrap();
        for rect in &rects {
            assert!(rect.left() >= sidebar.left() && rect.right() <= sidebar.right());
            assert!(rect.bottom() <= display.top());
            assert_eq!(rect.top(), rects[0].top());
        }
        assert!(
            rects
                .windows(2)
                .all(|pair| pair[0].right() <= pair[1].left())
        );
        assert_eq!(
            names[..6],
            [
                "Duplicate",
                "EditPoints",
                "ControlPoints",
                "Mirror",
                "Move",
                "Rotate"
            ]
        );
        assert!(app.controller.document().entities.is_empty());
    }

    #[test]
    fn trim_many_viewport_click_targets_a_curve() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        let mut doc = Document::with_template_tolerance("trim viewport");
        let line = |name, start, end| {
            Entity::new(
                name,
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 20.],
                }),
                None,
            )
        };
        let first = line("first", [-10., 0., 0.], [10., 0., 0.]);
        let second = line("second", [-10., 4., 0.], [10., 4., 0.]);
        let cutter = line("cutter", [0., -5., 0.], [0., 10., 0.]);
        let (first_id, second_id, cutter_id) = (first.id, second.id, cutter.id);
        for entity in [first, second, cutter] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller
            .select_objects(&[first_id, second_id, cutter_id])
            .unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([5., 0., 0.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        let Geometry::Curve(first_curve) = &app.controller.document().entities[&first_id].geometry
        else {
            panic!("curve")
        };
        let Geometry::Curve(second_curve) =
            &app.controller.document().entities[&second_id].geometry
        else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(first_curve, app.controller.coordinator().limits())
                .unwrap()
                - 10.)
                .abs()
                < 1e-7
        );
        assert!(
            (om9_core::curve_ops::arc_length(second_curve, app.controller.coordinator().limits())
                .unwrap()
                - 20.)
                .abs()
                < 1e-7
        );
        let screen = camera
            .project([-5., 0., 0.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        assert!(!app.controller.document().entities.contains_key(&first_id));
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        let Geometry::Curve(curve) = &app.controller.document().entities[&first_id].geometry else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(curve, app.controller.coordinator().limits())
                .unwrap()
                - 10.)
                .abs()
                < 1e-7
        );
        app.controller.undo().unwrap();
        let Geometry::Curve(curve) = &app.controller.document().entities[&first_id].geometry else {
            panic!("curve")
        };
        assert!(
            (om9_core::curve_ops::arc_length(curve, app.controller.coordinator().limits())
                .unwrap()
                - 20.)
                .abs()
                < 1e-7
        );
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn split_isocurve_picks_selected_surface_in_viewport() {
        use om9_core::geometry::Geometry;
        use om9_core::model::Entity;
        let limits = om9_core::budget::Limits::default();
        let surface = om9_core::surface_kernel::plane(
            [0., 0., 3.],
            [1., 0., 0.],
            [0., 1., 0.],
            10.,
            20.,
            &limits,
        )
        .unwrap();
        let target = Entity::new("surface", Geometry::Surface(surface), None);
        let id = target.id;
        let mut doc = Document::with_template_tolerance("isocurve viewport");
        doc.entities.insert(id, target);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(om9_core::top11::Top11Action::Split)
            .unwrap();
        app.controller.submit("Isocurve").unwrap();
        app.controller.submit("V").unwrap();
        app.controller.submit("Shrink=Yes").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([2., 6., 3.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        assert!(!app.controller.document().entities.contains_key(&id));
        assert_eq!(app.controller.document().entities.len(), 2);
        for entity in app.controller.document().entities.values() {
            let Geometry::Surface(surface) = &entity.geometry else {
                panic!("surface")
            };
            let [low, high] = surface.domain[0];
            assert!(high <= 0.21 || low >= 0.19, "{low} {high}");
            assert!((surface.knots[0][0] - low).abs() < 1e-8);
            assert!((*surface.knots[0].last().unwrap() - high).abs() < 1e-8);
        }
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.contains_key(&id));
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn split_isocurve_picks_brep_face_in_viewport() {
        use om9_core::geometry::Geometry;
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        let profile = Profile::new(vec![
            [0., 0., 3.],
            [10., 0., 3.],
            [10., 20., 3.],
            [0., 20., 3.],
        ])
        .unwrap();
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let target = Entity::new("face", Geometry::Brep(face), None);
        let id = target.id;
        let mut doc = Document::with_template_tolerance("brep isocurve viewport");
        doc.entities.insert(id, target);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(om9_core::top11::Top11Action::Split)
            .unwrap();
        app.controller.submit("Isocurve").unwrap();
        app.controller.submit("Both").unwrap();
        app.controller.submit("Shrink=Yes").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([2., 6., 3.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        assert!(!app.controller.document().entities.contains_key(&id));
        assert_eq!(app.controller.document().entities.len(), 4);
        let area: f64 = app
            .controller
            .document()
            .entities
            .values()
            .map(|e| {
                let Geometry::Brep(brep) = &e.geometry else {
                    panic!("BRep")
                };
                let bounds =
                    om9_kernel::measurement::face_underlying_bounds(brep, &limits).unwrap();
                assert!((bounds[1] - bounds[0]) < 10.0001 && (bounds[3] - bounds[2]) < 20.0001);
                creation::measure(brep, &limits).unwrap().area_mm2
            })
            .sum();
        assert!((area - 200.).abs() < 1e-6);
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.contains_key(&id));
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn split_isocurve_picks_curved_face_in_viewport() {
        use om9_core::geometry::Geometry;
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Primitive};
        let limits = om9_core::budget::Limits::default();
        let body = creation::primitive(
            Primitive::Cylinder {
                radius_mm: 5.,
                height_mm: 10.,
            },
            creation::DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let side = (0..creation::measure(&body, &limits).unwrap().faces)
            .map(|i| {
                om9_kernel::editing::extract_face(
                    &body,
                    i,
                    creation::DEFAULT_DEFLECTION_MM,
                    &limits,
                )
                .unwrap()
            })
            .find(|face| {
                (creation::measure(face, &limits).unwrap().area_mm2 - 100. * std::f64::consts::PI)
                    .abs()
                    < 1e-5
            })
            .unwrap();
        let target = Entity::new("cylinder side", Geometry::Brep(side), None);
        let id = target.id;
        let mut doc = Document::with_template_tolerance("curved isocurve viewport");
        doc.entities.insert(id, target);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Split").unwrap();
        app.controller.submit("Isocurve").unwrap();
        app.controller.submit("U").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Through Finger")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Through Finger")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([0., 5., 5.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        assert!(!app.controller.document().entities.contains_key(&id));
        assert_eq!(app.controller.document().entities.len(), 2);
        for entity in app.controller.document().entities.values() {
            let Geometry::Brep(brep) = &entity.geometry else {
                panic!("BRep")
            };
            assert!(
                (creation::measure(brep, &limits).unwrap().area_mm2 - 50. * std::f64::consts::PI)
                    .abs()
                    < 0.5
            );
        }
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.contains_key(&id));
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn split_surface_select_cutter_via_viewport() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        let profile = Profile::new(vec![
            [-10., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let target = Entity::new("face", Geometry::Brep(face), None);
        let cutter = Entity::new(
            "cutter",
            Geometry::Curve(Curve::Line {
                start: [0., -15., 0.],
                end: [0., 15., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let ids = [target.id, cutter.id];
        let mut doc = Document::with_template_tolerance("split surface viewport");
        for entity in [target, cutter] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[ids[0]]).unwrap();
        app.controller.submit("Split").unwrap();
        app.controller.submit("Shrink=Yes").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([0., 13., 0.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        app.controller.submit("").unwrap();
        assert!(!app.controller.document().entities.contains_key(&ids[0]));
        assert_eq!(app.controller.document().entities.len(), 3);
        for entity in app
            .controller
            .document()
            .entities
            .values()
            .filter(|e| e.id != ids[1])
        {
            let Geometry::Brep(brep) = &entity.geometry else {
                panic!("BRep")
            };
            let bounds = om9_kernel::measurement::face_underlying_bounds(brep, &limits).unwrap();
            assert!(
                (bounds[1] - bounds[0]).abs() <= 20.0001
                    && (bounds[3] - bounds[2]).abs() <= 20.0001
            );
        }
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.contains_key(&ids[0]));
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn split_polysurface_select_cutter_via_viewport() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        for along_seam in [false, true] {
            let face = |points| {
                creation::planar(
                    &Profile::new(points).unwrap(),
                    creation::DEFAULT_DEFLECTION_MM,
                    &limits,
                )
                .unwrap()
            };
            let left = face(vec![
                [-10., -10., 0.],
                [0., -10., 0.],
                [0., 10., 0.],
                [-10., 10., 0.],
            ]);
            let right = face(vec![
                [0., -10., 0.],
                [10., -10., 0.],
                [10., 10., 0.],
                [0., 10., 0.],
            ]);
            let joined = om9_kernel::editing::sew(
                &left,
                &right,
                0.001,
                creation::DEFAULT_DEFLECTION_MM,
                &limits,
            )
            .unwrap();
            let target = Entity::new("joined", Geometry::Brep(joined), None);
            let (start, end, pick) = if along_seam {
                ([0., -15., 0.], [0., 15., 0.], [0., 13., 0.])
            } else {
                ([-15., 0., 0.], [15., 0., 0.], [13., 0., 0.])
            };
            let cutter = Entity::new(
                "cutter",
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                }),
                None,
            );
            let ids = [target.id, cutter.id];
            let mut doc = Document::with_template_tolerance("split polysurface viewport");
            for entity in [target, cutter] {
                doc.entities.insert(entity.id, entity);
            }
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.select_objects(&[ids[0]]).unwrap();
            app.controller
                .top11_invoke(om9_core::top11::Top11Action::Split)
                .unwrap();
            app.controller.submit("Shrink=Yes").unwrap();
            let context = egui::Context::default();
            let size = Vec2::new(1280., 720.);
            for _ in 0..3 {
                window_frame(&mut app, &context, size, vec![]);
            }
            let rect = app
                .view_rects
                .iter()
                .find(|(id, _)| id.title() == "Looking Down")
                .unwrap()
                .1;
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(title, _, _)| *title == "Looking Down")
                .unwrap()
                .1;
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let screen = camera.project(pick, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
            app.controller.submit("").unwrap();
            assert!(!app.controller.document().entities.contains_key(&ids[0]));
            let pieces: Vec<_> = app
                .controller
                .document()
                .entities
                .values()
                .filter(|e| e.id != ids[1])
                .collect();
            assert_eq!(pieces.len(), 2);
            for piece in pieces {
                let Geometry::Brep(brep) = &piece.geometry else {
                    panic!("expected BRep")
                };
                let m = creation::measure(brep, &limits).unwrap();
                assert_eq!(m.faces, if along_seam { 1 } else { 2 });
                assert!((m.area_mm2 - 200.).abs() < 1e-6);
                for i in 0..m.faces {
                    let face = om9_kernel::editing::extract_face(
                        brep,
                        i,
                        creation::DEFAULT_DEFLECTION_MM,
                        &limits,
                    )
                    .unwrap();
                    let bounds =
                        om9_kernel::measurement::face_underlying_bounds(&face, &limits).unwrap();
                    assert!(
                        (bounds[1] - bounds[0]) <= 20.0001 && (bounds[3] - bounds[2]) <= 20.0001
                    );
                }
            }
            app.controller.undo().unwrap();
            assert!(app.controller.document().entities.contains_key(&ids[0]));
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_surface_regions_via_viewport_clicks() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let profile = Profile::new(vec![
            [-10., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let face = creation::planar(
            &profile,
            creation::DEFAULT_DEFLECTION_MM,
            &om9_core::budget::Limits::default(),
        )
        .unwrap();
        let target = Entity::new("face", Geometry::Brep(face), None);
        let line = |name, start, end| {
            Entity::new(
                name,
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                }),
                None,
            )
        };
        let a = line("vertical", [0., -12., 0.], [0., 12., 0.]);
        let b = line("horizontal", [-12., 0., 0.], [12., 0., 0.]);
        let ids = [target.id, a.id, b.id];
        let mut doc = Document::with_template_tolerance("trim surface viewport");
        for entity in [target, a, b] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        for (point, area) in [([-5., -5., 0.], 300.), ([5., 5., 0.], 200.)] {
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
            let Geometry::Brep(brep) = &app.controller.document().entities[&ids[0]].geometry else {
                panic!("face")
            };
            let actual = creation::measure(brep, app.controller.coordinator().limits())
                .unwrap()
                .area_mm2;
            assert!((actual - area).abs() < 1e-6, "{actual}");
        }
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        app.controller.undo().unwrap();
        let Geometry::Brep(brep) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("face")
        };
        assert!(
            (creation::measure(brep, app.controller.coordinator().limits())
                .unwrap()
                .area_mm2
                - 400.)
                .abs()
                < 1e-6
        );
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_surface_and_curve_via_viewport_clicks() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let profile = Profile::new(vec![
            [-10., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let face = creation::planar(
            &profile,
            creation::DEFAULT_DEFLECTION_MM,
            &om9_core::budget::Limits::default(),
        )
        .unwrap();
        let target = Entity::new("face", Geometry::Brep(face), None);
        let line = |name, start, end| {
            Entity::new(
                name,
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                }),
                None,
            )
        };
        let a = line("vertical", [0., -12., 0.], [0., 12., 0.]);
        let b = line("horizontal", [-12., 0., 0.], [12., 0., 0.]);
        let ids = [target.id, a.id, b.id];
        let mut doc = Document::with_template_tolerance("trim mixed viewport");
        for entity in [target, a, b] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        for point in [[0., -5., 0.], [-5., -5., 0.]] {
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
        }
        let Geometry::Brep(brep) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("face")
        };
        let actual = creation::measure(brep, app.controller.coordinator().limits())
            .unwrap()
            .area_mm2;
        assert!((actual - 200.).abs() < 1e-6, "{actual}");
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        app.controller.undo().unwrap();
        let Geometry::Brep(brep) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("face")
        };
        assert!(
            (creation::measure(brep, app.controller.coordinator().limits())
                .unwrap()
                .area_mm2
                - 400.)
                .abs()
                < 1e-6
        );
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_two_surfaces_via_viewport_clicks() {
        use om9_core::geometry::Geometry;
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        let horizontal = Profile::new(vec![
            [-10., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let vertical = Profile::new(vec![
            [-10., 0., -10.],
            [10., 0., -10.],
            [10., 0., 10.],
            [-10., 0., 10.],
        ])
        .unwrap();
        let a = Entity::new(
            "horizontal",
            Geometry::Brep(
                creation::planar(&horizontal, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap(),
            ),
            None,
        );
        let b = Entity::new(
            "vertical",
            Geometry::Brep(
                creation::planar(&vertical, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap(),
            ),
            None,
        );
        let ids = [a.id, b.id];
        let mut doc = Document::with_template_tolerance("mutual surfaces viewport");
        for entity in [a, b] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for (title, point, id) in [
            ("Looking Down", [0., -5., 0.], ids[0]),
            ("Through Finger", [0., 0., -5.], ids[1]),
        ] {
            let rect = app
                .view_rects
                .iter()
                .find(|(view, _)| view.title() == title)
                .unwrap()
                .1;
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(view, _, _)| *view == title)
                .unwrap()
                .1;
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
            let Geometry::Brep(face) = &app.controller.document().entities[&id].geometry else {
                panic!("face")
            };
            assert!((creation::measure(face, &limits).unwrap().area_mm2 - 200.).abs() < 1e-6);
        }
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        app.controller.undo().unwrap();
        for id in ids {
            let Geometry::Brep(face) = &app.controller.document().entities[&id].geometry else {
                panic!("face")
            };
            assert!((creation::measure(face, &limits).unwrap().area_mm2 - 400.).abs() < 1e-6);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_multiple_surfaces_and_curve_via_viewport_clicks() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        let horizontal = Profile::new(vec![
            [-10., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let vertical = Profile::new(vec![
            [-10., 0., -10.],
            [10., 0., -10.],
            [10., 0., 10.],
            [-10., 0., 10.],
        ])
        .unwrap();
        let a = Entity::new(
            "horizontal",
            Geometry::Brep(
                creation::planar(&horizontal, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap(),
            ),
            None,
        );
        let b = Entity::new(
            "vertical",
            Geometry::Brep(
                creation::planar(&vertical, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap(),
            ),
            None,
        );
        let curve = Entity::new(
            "curve",
            Geometry::Curve(Curve::Line {
                start: [0., -12., 0.],
                end: [0., 12., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let ids = [a.id, b.id, curve.id];
        let mut doc = Document::with_template_tolerance("mixed mutual viewport");
        for entity in [a, b, curve] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for (title, point) in [
            ("Looking Down", [0., -5., 0.]),
            ("Looking Down", [-5., -5., 0.]),
            ("Through Finger", [5., 0., -5.]),
        ] {
            let rect = app
                .view_rects
                .iter()
                .find(|(view, _)| view.title() == title)
                .unwrap()
                .1;
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(view, _, _)| *view == title)
                .unwrap()
                .1;
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
        }
        for id in &ids[..2] {
            let Geometry::Brep(face) = &app.controller.document().entities[id].geometry else {
                panic!("face")
            };
            assert!((creation::measure(face, &limits).unwrap().area_mm2 - 200.).abs() < 1e-6);
        }
        app.controller.submit("").unwrap();
        for _ in 0..3 {
            app.controller.undo().unwrap();
        }
        for id in &ids[..2] {
            let Geometry::Brep(face) = &app.controller.document().entities[id].geometry else {
                panic!("face")
            };
            assert!((creation::measure(face, &limits).unwrap().area_mm2 - 400.).abs() < 1e-6);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_multiface_brep_and_curve_via_viewport_clicks() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        let left = Profile::new(vec![
            [-10., -10., 0.],
            [0., -10., 0.],
            [0., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let right = Profile::new(vec![
            [0., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [0., 10., 0.],
        ])
        .unwrap();
        let left = creation::planar(&left, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let right = creation::planar(&right, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let joined = om9_kernel::editing::sew(
            &left,
            &right,
            1e-6,
            creation::DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let target = Entity::new("face", Geometry::Brep(joined), None);
        let line = |name, start, end| {
            Entity::new(
                name,
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                }),
                None,
            )
        };
        let a = line("vertical", [-6., -12., 0.], [-6., 12., 0.]);
        let b = line("horizontal", [-12., 0., 0.], [12., 0., 0.]);
        let ids = [target.id, a.id, b.id];
        let mut doc = Document::with_template_tolerance("multiface trim viewport");
        for entity in [target, a, b] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        for point in [[-8., -5., 0.], [-6., 5., 0.], [-8., 5., 0.]] {
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
        }
        let Geometry::Brep(face) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("face")
        };
        let measure = creation::measure(face, &limits).unwrap();
        assert_eq!(measure.faces, 2);
        assert!(
            (measure.area_mm2 - 260.).abs() < 1e-6,
            "{}",
            measure.area_mm2
        );
        app.controller.submit("").unwrap();
        for _ in 0..3 {
            app.controller.undo().unwrap();
        }
        let Geometry::Brep(face) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("face")
        };
        assert!((creation::measure(face, &limits).unwrap().area_mm2 - 400.).abs() < 1e-6);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_two_multiface_breps_and_curve_via_viewport_clicks() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Profile};
        let limits = om9_core::budget::Limits::default();
        let face = |points| {
            creation::planar(
                &Profile::new(points).unwrap(),
                creation::DEFAULT_DEFLECTION_MM,
                &limits,
            )
            .unwrap()
        };
        let join = |a, b| {
            om9_kernel::editing::sew(&a, &b, 1e-6, creation::DEFAULT_DEFLECTION_MM, &limits)
                .unwrap()
        };
        let a = join(
            face(vec![
                [-10., -10., 0.],
                [0., -10., 0.],
                [0., 10., 0.],
                [-10., 10., 0.],
            ]),
            face(vec![
                [0., -10., 0.],
                [10., -10., 0.],
                [10., 10., 0.],
                [0., 10., 0.],
            ]),
        );
        let b = join(
            face(vec![
                [-10., 0., -10.],
                [0., 0., -10.],
                [0., 0., 10.],
                [-10., 0., 10.],
            ]),
            face(vec![
                [0., 0., -10.],
                [10., 0., -10.],
                [10., 0., 10.],
                [0., 0., 10.],
            ]),
        );
        let a = Entity::new("horizontal", Geometry::Brep(a), None);
        let b = Entity::new("vertical", Geometry::Brep(b), None);
        let curve = Entity::new(
            "curve",
            Geometry::Curve(Curve::Line {
                start: [-6., -12., 0.],
                end: [-6., 12., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let ids = [a.id, b.id, curve.id];
        let mut doc = Document::with_template_tolerance("two multiface trim viewport");
        for entity in [a, b, curve] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        for (title, point) in [
            ("Through Finger", [-5., 0., -5.]),
            ("Looking Down", [-8., -5., 0.]),
            ("Looking Down", [-6., 5., 0.]),
            ("Looking Down", [-8., 5., 0.]),
        ] {
            let rect = app
                .view_rects
                .iter()
                .find(|(view, _)| view.title() == title)
                .unwrap()
                .1;
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(view, _, _)| *view == title)
                .unwrap()
                .1;
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
        }
        for (id, area) in [(ids[0], 260.), (ids[1], 300.)] {
            let Geometry::Brep(face) = &app.controller.document().entities[&id].geometry else {
                panic!("face")
            };
            let measured = creation::measure(face, &limits).unwrap();
            assert_eq!(measured.faces, 2);
            assert!(
                (measured.area_mm2 - area).abs() < 1e-6,
                "{}",
                measured.area_mm2
            );
        }
        app.controller.submit("").unwrap();
        for _ in 0..4 {
            app.controller.undo().unwrap();
        }
        for id in &ids[..2] {
            let Geometry::Brep(face) = &app.controller.document().entities[id].geometry else {
                panic!("face")
            };
            assert!((creation::measure(face, &limits).unwrap().area_mm2 - 400.).abs() < 1e-6);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_solid_boundary_via_viewport_click() {
        use om9_core::geometry::Geometry;
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Primitive, Profile};
        let limits = om9_core::budget::Limits::default();
        let solid = creation::primitive(
            Primitive::Box {
                x_mm: 10.,
                y_mm: 10.,
                z_mm: 10.,
            },
            creation::DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let profile = Profile::new(vec![
            [5., -5., -5.],
            [5., 15., -5.],
            [5., 15., 15.],
            [5., -5., 15.],
        ])
        .unwrap();
        let cutter = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let a = Entity::new("solid", Geometry::Brep(solid), None);
        let b = Entity::new("cutter", Geometry::Brep(cutter), None);
        let ids = [a.id, b.id];
        let mut doc = Document::with_template_tolerance("solid trim viewport");
        for entity in [a, b] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([2., 2., 10.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        let Geometry::Brep(face) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("BRep")
        };
        let measured = creation::measure(face, &limits).unwrap();
        assert_eq!(measured.solids, 0);
        assert!(
            (measured.area_mm2 - 550.).abs() < 1e-6,
            "{}",
            measured.area_mm2
        );
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        let Geometry::Brep(solid) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("BRep")
        };
        assert_eq!(creation::measure(solid, &limits).unwrap().solids, 1);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn trim_sphere_boundary_via_viewport_click() {
        use om9_core::geometry::Geometry;
        use om9_core::model::Entity;
        use om9_kernel::creation::{self, Primitive, Profile};
        let limits = om9_core::budget::Limits::default();
        let sphere = creation::primitive(
            Primitive::Sphere { radius_mm: 5. },
            creation::DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let profile = Profile::new(vec![
            [-10., -10., 0.],
            [10., -10., 0.],
            [10., 10., 0.],
            [-10., 10., 0.],
        ])
        .unwrap();
        let cutter = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let a = Entity::new("sphere", Geometry::Brep(sphere), None);
        let b = Entity::new("cutter", Geometry::Brep(cutter), None);
        let ids = [a.id, b.id];
        let mut doc = Document::with_template_tolerance("sphere trim viewport");
        for entity in [a, b] {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Trim").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let screen = camera
            .project([0., 3., 4.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        let Geometry::Brep(result) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("BRep")
        };
        let measure = creation::measure(result, &limits).unwrap();
        assert_eq!(measure.solids, 0);
        assert!(
            (measure.area_mm2 - 50. * std::f64::consts::PI).abs() < 1e-5,
            "{}",
            measure.area_mm2
        );
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        let Geometry::Brep(result) = &app.controller.document().entities[&ids[0]].geometry else {
            panic!("BRep")
        };
        assert_eq!(creation::measure(result, &limits).unwrap().solids, 1);
    }

    #[test]
    fn copy_and_mirror_cmd_widget_accept_typed_coordinates() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        for mirror in [false, true] {
            let entity = Entity::new(
                "source",
                Geometry::Curve(Curve::Line {
                    start: [2., 0., 0.],
                    end: [3., 0., 0.],
                    domain: [0., 1.],
                }),
                None,
            );
            let id = entity.id;
            let mut doc = Document::with_template_tolerance("typed cmd widget");
            doc.entities.insert(id, entity);
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.select_objects(&[id]).unwrap();
            let before = app.controller.document().clone();
            let ctx = egui::Context::default();
            let size = Vec2::new(1280., 720.);
            app.focus_cmd = true;
            for _ in 0..3 {
                window_frame(&mut app, &ctx, size, vec![]);
            }
            let enter = || egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            };
            let words = if mirror {
                vec!["Mirror", "Copy=No", "W1,2,0", "@0,3,0"]
            } else {
                vec!["Copy", "W1,2,0", "@0,3,0"]
            };
            for word in words {
                window_frame(&mut app, &ctx, size, vec![egui::Event::Text(word.into())]);
                window_frame(&mut app, &ctx, size, vec![enter()]);
            }
            if !mirror {
                window_frame(&mut app, &ctx, size, vec![enter()]);
                let copy = app
                    .controller
                    .document()
                    .entities
                    .values()
                    .find(|e| e.id != id)
                    .unwrap();
                let p = copy.placement.point([0.; 3]).unwrap();
                assert!((p[1] - 3.).abs() < 1e-9, "{p:?}");
            } else {
                let Geometry::Curve(Curve::Line { start, end, .. }) =
                    &app.controller.document().entities[&id].geometry
                else {
                    panic!("line")
                };
                assert!(
                    start[0].abs() < 1e-9 && (end[0] + 1.).abs() < 1e-9,
                    "{start:?} {end:?}"
                );
            }
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
        }
    }

    #[test]
    fn split_point_cmd_widget_preserves_mode_after_object_selection() {
        let mut document = Document::with_template_tolerance("split point cmd");
        let entity = om9_core::model::Entity::new(
            "line",
            om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut app = MatrixApp::new(document).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.focus_cmd = true;
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let enter = || egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::Text("Split Point".into())],
        );
        window_frame(&mut app, &context, size, vec![enter()]);
        assert!(app.controller.top11_needs_object());
        let pos = app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Perspective")
            .expect("perspective viewport")
            .1
            .center();
        window_frame(&mut app, &context, size, pointer(pos, true));
        window_frame(&mut app, &context, size, pointer(pos, false));
        assert!(
            app.controller.selected().contains(&id),
            "viewport pick selects the line"
        );
        app.focus_cmd = true;
        window_frame(&mut app, &context, size, vec![]);
        window_frame(&mut app, &context, size, vec![enter()]);
        assert_eq!(
            app.controller.top11_prompt(),
            Some("pick a point on the curve")
        );
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::Text("W4,0,0".into())],
        );        window_frame(&mut app, &context, size, vec![enter()]);
        eprintln!(
            "DBG prompt={:?} with_options={:?} running={} entities={} transcript={:?}",
            app.controller.top11_prompt(),
            app.controller.prompt_with_options(),
            app.controller.is_running(),
            app.controller.document().entities.len(),
            app.transcript.text()
        );
        assert_eq!(app.controller.document().entities.len(), 2);
        let mut lengths: Vec<_> = app
            .controller
            .document()
            .entities
            .values()
            .map(|entity| {
                let om9_core::geometry::Geometry::Curve(curve) = &entity.geometry else {
                    panic!("curve")
                };
                om9_core::curve_ops::arc_length(curve, app.controller.coordinator().limits())
                    .unwrap()
            })
            .collect();
        lengths.sort_by(f64::total_cmp);
        assert!((lengths[0] - 4.).abs() < 1e-8 && (lengths[1] - 6.).abs() < 1e-8);
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.contains_key(&id));
    }

    #[test]
    fn top11_multi_split_cmd_undo_redo_from_clean_document() {
        let mut app =
            MatrixApp::new(Document::with_template_tolerance("split ui history")).unwrap();
        let mut ids = Vec::new();
        for (start, end) in [
            ("-10,0,0", "10,0,0"),
            ("-10,4,0", "10,4,0"),
            ("0,-5,0", "0,10,0"),
        ] {
            let before = app
                .controller
                .document()
                .entities
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            app.controller.submit("Line").unwrap();
            app.controller.submit(start).unwrap();
            app.controller.submit(end).unwrap();
            ids.push(
                *app.controller
                    .document()
                    .entities
                    .keys()
                    .find(|id| !before.contains(id))
                    .unwrap(),
            );
        }
        app.controller.select_objects(&ids[..2]).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let split = context.memory(|memory| {
            memory
                .data
                .get_temp::<Rect>(egui::Id::new(("top11-sidebar", &"Split")))
                .expect("sidebar Split icon")
                .center()
        });
        window_frame(&mut app, &context, size, pointer(split, true));
        window_frame(&mut app, &context, size, pointer(split, false));
        assert!(app.controller.top11_needs_object(), "Split sidebar click");
        app.controller.top11_accept_object(ids[2]).unwrap();
        app.controller.submit("").unwrap();
        assert_eq!(app.controller.document().entities.len(), 5);
        assert_eq!(app.controller.coordinator().undo_len(), 4);

        app.focus_cmd = true;
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let enter = || egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::Text("Undo".into())],
        );
        window_frame(&mut app, &context, size, vec![enter()]);
        assert_eq!(
            app.controller.document().entities.len(),
            3,
            "CMD Undo restores both original targets"
        );
        assert!(
            ids.iter()
                .all(|id| app.controller.document().entities.contains_key(id))
        );
        assert_eq!(app.controller.coordinator().undo_len(), 3);
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::Text("Redo".into())],
        );
        window_frame(&mut app, &context, size, vec![enter()]);
        assert_eq!(
            app.controller.document().entities.len(),
            5,
            "CMD Redo restores all split pieces"
        );
        assert_eq!(app.controller.coordinator().undo_len(), 4);
        assert_eq!(app.controller.coordinator().redo_len(), 0);

        // Use the actual widget rectangles, independent of toolbar order or scaling.
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Standard").unwrap();
        window_frame(&mut app, &context, size, vec![]);
        let undo: Rect = context.memory(|m| {
            m.data
                .get_temp(egui::Id::new(("toolbar-test", "Standard", &"Undo")))
                .unwrap()
        });
        let undo = undo.center();
        window_frame(&mut app, &context, size, pointer(undo, true));
        window_frame(&mut app, &context, size, pointer(undo, false));
        assert_eq!(app.controller.document().entities.len(), 3, "toolbar Undo");
        assert_eq!(app.controller.coordinator().redo_len(), 1);
        let redo: Rect = context.memory(|m| {
            m.data
                .get_temp(egui::Id::new(("toolbar-test", "Standard", &"Redo")))
                .unwrap()
        });
        let redo = redo.center();
        window_frame(&mut app, &context, size, pointer(redo, true));
        window_frame(&mut app, &context, size, pointer(redo, false));
        assert_eq!(app.controller.document().entities.len(), 5, "toolbar Redo");
        assert_eq!(app.controller.coordinator().redo_len(), 0);
    }
    #[test]
    fn control_surface_grips_pointer_shift_delete_and_undo() {
        use om9_core::geometry::{Geometry, NurbsSurface};
        use om9_core::model::Entity;
        let surface = NurbsSurface {
            degree: [2, 2],
            size: [4, 4],
            poles: (0..4)
                .flat_map(|u| (0..4).map(move |v| [u as f64 * 10. - 15., v as f64 * 10. - 15., 0.]))
                .collect(),
            weights: vec![1.; 16],
            knots: [
                vec![0., 0., 0., 0.5, 1., 1., 1.],
                vec![0., 0., 0., 0.5, 1., 1., 1.],
            ],
            domain: [[0., 1.], [0., 1.]],
            periodic: [false, false],
            reversed: false,
        };
        let entity = Entity::new("surface", Geometry::Surface(surface), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("surface pointer delete");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("PointsOn").unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(id, _)| id.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let handles = app.controller.top11_grip_handles().unwrap();
        let before = app.controller.document().clone();
        for index in 4..8 {
            let screen = camera
                .project(handles[index].point, viewport)
                .unwrap()
                .unwrap()
                .screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            assert!(rect.contains(pos));
            let modifiers = if index == 4 {
                Modifiers::NONE
            } else {
                Modifiers {
                    shift: true,
                    ..Modifiers::NONE
                }
            };
            for pressed in [true, false] {
                let events = vec![
                    egui::Event::ModifiersChanged(modifiers),
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers,
                    },
                ];
                let raw = egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                    events,
                    ..Default::default()
                };
                let _ = context.run_ui(raw, |ui| app.show(ui));
            }
            assert_eq!(app.selected_grips.len(), index - 3, "grip {index}");
        }
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
        let Geometry::Surface(edited) = &app.controller.document().entities[&id].geometry else {
            panic!("surface lost")
        };
        assert_eq!(edited.size, [3, 4]);
        let after = app.controller.document().clone();
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }

    #[test]
    fn mixed_object_selection_still_allows_pointer_grip_edit() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_scene::workspace::ViewId;
        for mode in ["PointsOn", "EditPtOn"] {
            let line = Entity::new(
                "line",
                Geometry::Curve(Curve::Line {
                    start: [0., 0., 0.],
                    end: [2., 0., 0.],
                    domain: [0., 1.],
                }),
                None,
            );
            let point = Entity::new("point", Geometry::Point([8., 0., 0.]), None);
            let ids = [line.id, point.id];
            let mut doc = Document::with_template_tolerance("mixed grip viewport");
            doc.entities.insert(ids[0], line);
            doc.entities.insert(ids[1], point);
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.select_objects(&ids).unwrap();
            app.controller.submit(mode).unwrap();
            app.controller.workspace.active = ViewId::LookingDown;
            let context = egui::Context::default();
            let size = Vec2::new(1280., 720.);
            for _ in 0..3 {
                window_frame(&mut app, &context, size, vec![]);
            }
            let rect = *app
                .view_rects
                .iter()
                .find(|(v, _)| **v == ViewId::LookingDown)
                .unwrap()
                .1;
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(title, _, _)| *title == "Looking Down")
                .unwrap()
                .1;
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let screen = |p| {
                let q = camera.project(p, viewport).unwrap().unwrap().screen;
                egui::pos2(q[0] as f32, q[1] as f32)
            };
            let start = screen([2., 0., 0.]);
            let end = screen([4., 1., 0.]);
            let before = app.controller.document().clone();
            assert_eq!(app.controller.top11_grip_handles().unwrap().len(), 2);
            window_frame(&mut app, &context, size, pointer(start, true));
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(end)],
            );
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(end)],
            );
            window_frame(&mut app, &context, size, pointer(end, false));
            let Geometry::Curve(Curve::Line { end, .. }) =
                &app.controller.document().entities[&ids[0]].geometry
            else {
                panic!("line")
            };
            assert!(
                (end[0] - 4.).abs() < 1e-4 && (end[1] - 1.).abs() < 1e-4,
                "{mode} {end:?}"
            );
            assert_eq!(
                app.controller.document().entities[&ids[1]],
                before.entities[&ids[1]]
            );
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
        }
    }

    #[test]
    fn grip_drag_cancels_on_context_change_and_escape() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_scene::workspace::ViewId;
        for scenario in [
            "selection",
            "geometry",
            "camera",
            "view",
            "plane",
            "maximize",
            "resize",
            "mode",
            "focus",
            "escape",
        ] {
            let make = |name, start, end| {
                Entity::new(
                    name,
                    Geometry::Curve(Curve::Line {
                        start,
                        end,
                        domain: [0., 1.],
                    }),
                    None,
                )
            };
            let first = make("first", [0., 0., 0.], [2., 0., 0.]);
            let second = make("second", [6., 0., 0.], [8., 0., 0.]);
            let ids = [first.id, second.id];
            let mut doc = Document::with_template_tolerance("grip lifecycle");
            doc.entities.insert(ids[0], first);
            doc.entities.insert(ids[1], second);
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.select_objects(&[ids[0]]).unwrap();
            app.controller.submit("PointsOn").unwrap();
            app.controller.workspace.active = ViewId::LookingDown;
            let context = egui::Context::default();
            let size = Vec2::new(1280., 720.);
            for _ in 0..3 {
                window_frame(&mut app, &context, size, vec![]);
            }
            let rect = *app
                .view_rects
                .iter()
                .find(|(v, _)| **v == ViewId::LookingDown)
                .unwrap()
                .1;
            let camera = app
                .controller
                .view_cameras()
                .into_iter()
                .find(|(title, _, _)| *title == "Looking Down")
                .unwrap()
                .1;
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            let screen = |p| {
                let v = camera.project(p, viewport).unwrap().unwrap().screen;
                egui::pos2(v[0] as f32, v[1] as f32)
            };
            let start = screen([2., 0., 0.]);
            let end = screen([4., 1., 0.]);
            window_frame(&mut app, &context, size, pointer(start, true));
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(end)],
            );
            window_frame(
                &mut app,
                &context,
                size,
                vec![egui::Event::PointerMoved(end)],
            );
            assert!(app.grip_drag.is_some(), "{scenario}");
            match scenario {
                "selection" => app.controller.select_objects(&[ids[1]]).unwrap(),
                "geometry" => {
                    let h = app.grip_drag.as_ref().unwrap().handle;
                    app.controller.top11_move_grip(h, [3., 0., 0.]).unwrap();
                }
                "camera" => app
                    .controller
                    .workspace
                    .views
                    .get_mut(&ViewId::LookingDown)
                    .unwrap()
                    .camera
                    .zoom(2.)
                    .unwrap(),
                "view" => app.controller.workspace.active = ViewId::ThroughFinger,
                "plane" => app
                    .controller
                    .set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
                    .unwrap(),
                "maximize" => app.controller.workspace.maximized = Some(ViewId::Perspective),
                "resize" => window_frame(&mut app, &context, size + egui::vec2(50., 0.), vec![]),
                "mode" => {
                    app.controller.submit("PointsOn").unwrap();
                }
                "focus" => {
                    let raw = egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                        focused: false,
                        ..Default::default()
                    };
                    let mut output = context.run_ui(raw, |ui| app.show(ui));
                    output.textures_delta.clear();
                }
                "escape" => window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::Key {
                        key: Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Modifiers::NONE,
                    }],
                ),
                _ => unreachable!(),
            }
            let expected = app.controller.document().clone();
            let undo = app.controller.coordinator().undo_len();
            window_frame(&mut app, &context, size, pointer(end, false));
            assert_eq!(
                app.controller.document(),
                &expected,
                "{scenario} committed a stale grip drag"
            );
            assert_eq!(app.controller.coordinator().undo_len(), undo, "{scenario}");
            assert!(app.grip_drag.is_none(), "{scenario}");
        }
    }

    #[test]
    fn grip_drag_parallel_construction_plane_reports_failure_without_losing_selection() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_scene::{
            camera::{CPlane, CPlanePreset},
            workspace::ViewId,
        };
        let entity = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [2., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("parallel grip");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("PointsOn").unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller
            .workspace
            .views
            .get_mut(&ViewId::ThroughFinger)
            .unwrap()
            .cplane = CPlane::preset(CPlanePreset::Top);
        let handle = app
            .controller
            .top11_grip_handles()
            .unwrap()
            .into_iter()
            .find(|h| h.index == 1)
            .unwrap();
        app.selected_grips
            .push((handle, app.controller.document().revision));
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(v, _)| **v == ViewId::ThroughFinger)
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Through Finger")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let p = camera
            .project(handle.point, viewport)
            .unwrap()
            .unwrap()
            .screen;
        let start = egui::pos2(p[0] as f32, p[1] as f32);
        let end = start + egui::vec2(0., -25.);
        let before = app.controller.document().clone();
        window_frame(&mut app, &context, size, pointer(start, true));
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::PointerMoved(end)],
        );
        window_frame(&mut app, &context, size, pointer(end, false));
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        assert_eq!(app.controller.workspace.active, ViewId::LookingDown);
        assert_eq!(app.selected_grips.len(), 1);
        assert!(
            app.status.message().contains("construction plane"),
            "{}",
            app.status.message()
        );
    }

    #[test]
    fn grip_drag_uses_the_clicked_view_plane_when_another_view_is_active() {
        use om9_core::geometry::{Curve, Geometry, NurbsSurface};
        use om9_core::model::Entity;
        use om9_scene::{camera::CPlane, workspace::ViewId};
        let q = std::f64::consts::FRAC_1_SQRT_2;
        for surface in [false, true] {
            for mode in ["PointsOn", "EditPtOn"] {
                for (view_id, start, target, active, tilted) in [
                    (
                        ViewId::ThroughFinger,
                        [2., 0., 0.],
                        [2., 0., 2.],
                        ViewId::LookingDown,
                        false,
                    ),
                    (
                        ViewId::SideView,
                        [0., 2., 0.],
                        [0., 2., 2.],
                        ViewId::LookingDown,
                        false,
                    ),
                    (
                        ViewId::Perspective,
                        [2., 0., 0.],
                        [2., 2., 0.],
                        ViewId::LookingDown,
                        false,
                    ),
                    (
                        ViewId::LookingDown,
                        [2., 0., 2.],
                        [2. + 2. * q, -2., 2. + 2. * q],
                        ViewId::ThroughFinger,
                        true,
                    ),
                ] {
                    let offset = if tilted { [0., -3., 0.] } else { [0., 0., -3.] };
                    let geometry = if surface {
                        Geometry::Surface(NurbsSurface {
                            degree: [1, 1],
                            size: [2, 2],
                            poles: vec![
                                [0.; 3],
                                start,
                                offset,
                                std::array::from_fn(|i| start[i] + offset[i]),
                            ],
                            weights: vec![1.; 4],
                            knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
                            domain: [[0., 1.], [0., 1.]],
                            periodic: [false, false],
                            reversed: false,
                        })
                    } else {
                        Geometry::Curve(Curve::Line {
                            start: [0.; 3],
                            end: start,
                            domain: [0., 1.],
                        })
                    };
                    let entity = Entity::new("view geometry", geometry, None);
                    let id = entity.id;
                    let mut doc = Document::with_template_tolerance("inactive view grip");
                    doc.entities.insert(id, entity);
                    let mut app = MatrixApp::new(doc).unwrap();
                    app.controller.select_objects(&[id]).unwrap();
                    app.controller.submit(mode).unwrap();
                    app.controller.workspace.active = active;
                    if tilted {
                        app.controller
                            .workspace
                            .views
                            .get_mut(&view_id)
                            .unwrap()
                            .cplane = CPlane {
                            origin: [0.; 3],
                            x: [q, 0., q],
                            y: [0., 1., 0.],
                            revision: 1,
                        };
                    }
                    let context = egui::Context::default();
                    let size = Vec2::new(1280., 720.);
                    for _ in 0..3 {
                        window_frame(&mut app, &context, size, vec![]);
                    }
                    let rect = *app
                        .view_rects
                        .iter()
                        .find(|(v, _)| **v == view_id)
                        .unwrap()
                        .1;
                    let camera = app
                        .controller
                        .view_cameras()
                        .into_iter()
                        .find(|(title, _, _)| *title == view_id.title())
                        .unwrap()
                        .1;
                    let viewport = om9_scene::camera::Viewport {
                        origin: [rect.min.x as f64, rect.min.y as f64],
                        size: [rect.width() as f64, rect.height() as f64],
                    };
                    let screen = |p| {
                        let a = camera.project(p, viewport).unwrap().unwrap().screen;
                        egui::pos2(a[0] as f32, a[1] as f32)
                    };
                    let from = screen(start);
                    let to = screen(target);
                    assert!(rect.contains(from) && rect.contains(to));
                    let before = app.controller.document().clone();
                    window_frame(&mut app, &context, size, pointer(from, true));
                    window_frame(
                        &mut app,
                        &context,
                        size,
                        vec![egui::Event::PointerMoved(to)],
                    );
                    window_frame(
                        &mut app,
                        &context,
                        size,
                        vec![egui::Event::PointerMoved(to)],
                    );
                    let preview = app
                        .grip_drag
                        .as_ref()
                        .expect("grip drag in clicked view")
                        .current;
                    assert!(
                        (0..3).all(|i| (preview[i] - target[i]).abs() < 1e-4),
                        "{view_id:?} {preview:?}"
                    );
                    assert_eq!(app.controller.document(), &before);
                    window_frame(&mut app, &context, size, pointer(to, false));
                    let point = match &app.controller.document().entities[&id].geometry {
                        Geometry::Curve(Curve::Line { end, .. }) => *end,
                        Geometry::Surface(s) => s.poles[1],
                        _ => panic!("geometry"),
                    };
                    assert!(
                        (0..3).all(|i| (point[i] - target[i]).abs() < 1e-4),
                        "{view_id:?} {mode} surface={surface} {point:?}"
                    );
                    assert_eq!(app.controller.workspace.active, view_id);
                    let after = app.controller.document().clone();
                    let path = std::env::temp_dir()
                        .join(format!("om9-view-grip-{}.om9", uuid::Uuid::new_v4()));
                    om9_archive::Archive::save(
                        &path,
                        &after,
                        &om9_core::budget::Limits::default(),
                        &om9_core::transaction::Cancellation::default(),
                    )
                    .unwrap();
                    assert_eq!(
                        om9_archive::Archive::open(&path, &om9_core::budget::Limits::default())
                            .unwrap(),
                        after
                    );
                    std::fs::remove_file(path).unwrap();
                    app.controller.undo().unwrap();
                    assert_eq!(app.controller.document(), &before);
                    app.controller.redo().unwrap();
                    assert_eq!(app.controller.document(), &after);
                }
            }
        }
    }

    #[test]
    fn control_grip_pointer_drag_preserves_small_visible_motion() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        use om9_scene::{camera::Projection, workspace::ViewId};
        for mode in ["PointsOn", "EditPtOn"] {
            for delta in [[5e-10, 0.], [-5e-10, 0.], [0., 5e-10], [0., -5e-10]] {
                let entity = Entity::new(
                    "small line",
                    Geometry::Curve(Curve::Line {
                        start: [-2e-9, 0., 0.],
                        end: [2e-9, 0., 0.],
                        domain: [0., 1.],
                    }),
                    None,
                );
                let id = entity.id;
                let mut doc = Document::with_template_tolerance("small grip drag");
                doc.tolerance.as_mut().unwrap().absolute_mm = 1e-13;
                doc.entities.insert(id, entity);
                let mut app = MatrixApp::new(doc).unwrap();
                app.controller.select_objects(&[id]).unwrap();
                app.controller.submit(mode).unwrap();
                app.controller.workspace.active = ViewId::LookingDown;
                app.controller
                    .workspace
                    .views
                    .get_mut(&ViewId::LookingDown)
                    .unwrap()
                    .camera
                    .projection = Projection::Orthographic {
                    vertical_span_mm: 1e-8,
                };
                let context = egui::Context::default();
                let size = Vec2::new(1280., 720.);
                for _ in 0..3 {
                    window_frame(&mut app, &context, size, vec![]);
                }
                let rect = *app
                    .view_rects
                    .iter()
                    .find(|(v, _)| **v == ViewId::LookingDown)
                    .unwrap()
                    .1;
                let camera = app
                    .controller
                    .view_cameras()
                    .into_iter()
                    .find(|(title, _, _)| *title == "Looking Down")
                    .unwrap()
                    .1;
                let viewport = om9_scene::camera::Viewport {
                    origin: [rect.min.x as f64, rect.min.y as f64],
                    size: [rect.width() as f64, rect.height() as f64],
                };
                let target = [2e-9 + delta[0], delta[1], 0.];
                let from = camera
                    .project([2e-9, 0., 0.], viewport)
                    .unwrap()
                    .unwrap()
                    .screen;
                let to = camera.project(target, viewport).unwrap().unwrap().screen;
                let start = egui::pos2(from[0] as f32, from[1] as f32);
                let end = egui::pos2(to[0] as f32, to[1] as f32);
                assert!((end - start).length() > 8.);
                let before = app.controller.document().clone();
                window_frame(&mut app, &context, size, pointer(start, true));
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(end)],
                );
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(end)],
                );
                let preview = app.grip_drag.as_ref().expect("active grip preview").current;
                assert!((0..3).all(|i| (preview[i] - target[i]).abs() < 1e-11));
                assert_eq!(app.controller.document(), &before);
                window_frame(&mut app, &context, size, pointer(end, false));
                let Geometry::Curve(Curve::Line { end: line_end, .. }) =
                    &app.controller.document().entities[&id].geometry
                else {
                    panic!("line")
                };
                assert!(
                    (0..3).all(|i| (line_end[i] - target[i]).abs() < 1e-11),
                    "{mode} {line_end:?}"
                );
                let after = app.controller.document().clone();
                let file = std::env::temp_dir()
                    .join(format!("om9-small-grip-{}.om9", uuid::Uuid::new_v4()));
                om9_archive::Archive::save(
                    &file,
                    &after,
                    &om9_core::budget::Limits::default(),
                    &om9_core::transaction::Cancellation::default(),
                )
                .unwrap();
                assert_eq!(
                    om9_archive::Archive::open(&file, &om9_core::budget::Limits::default())
                        .unwrap(),
                    after
                );
                std::fs::remove_file(file).unwrap();
                app.controller.undo().unwrap();
                assert_eq!(app.controller.document(), &before);
                app.controller.redo().unwrap();
                assert_eq!(app.controller.document(), &after);
                app.controller.undo().unwrap();
                let undo_depth = app.controller.coordinator().undo_len();
                let off_center = start + egui::vec2(1., 0.);
                window_frame(&mut app, &context, size, pointer(off_center, true));
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(end)],
                );
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(off_center)],
                );
                window_frame(&mut app, &context, size, pointer(off_center, false));
                assert_eq!(
                    app.controller.document(),
                    &before,
                    "returning the pointer to its press point must not edit the grip"
                );
                assert_eq!(app.controller.coordinator().undo_len(), undo_depth);
                let off_end = end + egui::vec2(1., 0.);
                window_frame(&mut app, &context, size, pointer(off_center, true));
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(off_end)],
                );
                window_frame(
                    &mut app,
                    &context,
                    size,
                    vec![egui::Event::PointerMoved(off_end)],
                );
                window_frame(&mut app, &context, size, pointer(off_end, false));
                let Geometry::Curve(Curve::Line { end: line_end, .. }) =
                    &app.controller.document().entities[&id].geometry
                else {
                    panic!("line")
                };
                assert!(
                    (0..3).all(|i| (line_end[i] - target[i]).abs() < 1e-11),
                    "off-center {mode} {line_end:?}"
                );
                app.controller.undo().unwrap();
                assert_eq!(app.controller.document(), &before);
            }
        }
    }

    #[test]
    fn control_grip_pointer_drag_does_not_trigger_selection_band() {
        use om9_core::geometry::{Curve, Geometry, NurbsCurve};
        use om9_core::model::Entity;
        let curve = NurbsCurve {
            degree: 2,
            poles: vec![[-10., 0., 0.], [-5., 5., 0.], [5., 5., 0.], [10., 0., 0.]],
            weights: vec![1.; 4],
            knots: vec![0., 0., 0., 0.5, 1., 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed: false,
            seam: None,
        };
        let entity = Entity::new("curve", Geometry::Curve(Curve::Nurbs(curve.clone())), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("grip drag viewport");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("PointsOn").unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let from = camera
            .project(curve.poles[1], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let to = camera
            .project([-3., 7., 0.], viewport)
            .unwrap()
            .unwrap()
            .screen;
        let start = egui::pos2(from[0] as f32, from[1] as f32);
        let end = egui::pos2(to[0] as f32, to[1] as f32);
        window_frame(&mut app, &context, size, pointer(start, true));
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::PointerMoved(end)],
        );
        window_frame(
            &mut app,
            &context,
            size,
            vec![egui::Event::PointerMoved(end)],
        );
        window_frame(&mut app, &context, size, pointer(end, false));
        assert!(app.controller.selected().contains(&id));
        let Geometry::Curve(Curve::Nurbs(result)) =
            &app.controller.document().entities[&id].geometry
        else {
            panic!("curve lost")
        };
        assert!(
            (0..3).all(|i| (result.poles[1][i] - [-3., 7., 0.][i]).abs() < 1e-4),
            "{:?}",
            result.poles[1]
        );
        app.controller.undo().unwrap();
        let Geometry::Curve(Curve::Nurbs(result)) =
            &app.controller.document().entities[&id].geometry
        else {
            panic!("curve lost")
        };
        assert_eq!(result, &curve);
    }

    #[test]
    fn duplicate_invalid_viewport_point_keeps_prompt_for_retry() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        let source = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let id = source.id;
        let mut doc = Document::with_template_tolerance("duplicate retry pointer");
        doc.entities.insert(id, source);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Copy UseLastDistance=Yes").unwrap();
        app.controller.accept_world_point([0.; 3]).unwrap();
        app.controller.accept_world_point([10., 0., 0.]).unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let pick = |world| {
            let p = camera.project(world, viewport).unwrap().unwrap().screen;
            egui::pos2(p[0] as f32, p[1] as f32)
        };
        let invalid = pick([0., 0., 0.]);
        let before = app.controller.document().clone();
        window_frame(&mut app, &context, size, pointer(invalid, true));
        window_frame(&mut app, &context, size, pointer(invalid, false));
        assert_eq!(app.controller.document(), &before);
        assert!(app.controller.is_running());
        let valid = pick([0., 5., 0.]);
        window_frame(&mut app, &context, size, pointer(valid, true));
        window_frame(&mut app, &context, size, pointer(valid, false));
        assert_eq!(app.controller.document().entities.len(), 3);
        assert!(
            app.controller.document().entities.values().any(|e| (e
                .placement
                .point([0.; 3])
                .unwrap()[1]
                - 10.)
                .abs()
                < 1e-3)
        );
        app.controller.submit("").unwrap();
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }

    #[test]
    fn mirror_invalid_viewport_point_keeps_first_point_for_retry() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        let source = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [1., 0., 0.],
                end: [2., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let id = source.id;
        let mut doc = Document::with_template_tolerance("mirror retry pointer");
        doc.entities.insert(id, source);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Mirror").unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let pick = |world| {
            let p = camera.project(world, viewport).unwrap().unwrap().screen;
            egui::pos2(p[0] as f32, p[1] as f32)
        };
        let origin = pick([0.; 3]);
        for _ in 0..2 {
            window_frame(&mut app, &context, size, pointer(origin, true));
            window_frame(&mut app, &context, size, pointer(origin, false));
        }
        assert!(app.controller.is_running());
        assert_eq!(app.controller.document().entities.len(), 1);
        let end = pick([0., 5., 0.]);
        window_frame(&mut app, &context, size, pointer(end, true));
        window_frame(&mut app, &context, size, pointer(end, false));
        assert_eq!(app.controller.document().entities.len(), 2);
        assert!(!app.controller.is_running());
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document().entities.len(), 1);
    }

    #[test]
    fn mirror_block_with_viewport_points_keeps_definition() {
        use om9_core::{
            geometry::{Geometry, Transform},
            model::{Definition, Entity},
        };
        let leaf = Entity::new("point", Geometry::Point([1., 2., 0.]), None);
        let leaf_id = leaf.id;
        let def = Definition::new("block", vec![leaf]);
        let def_id = def.id;
        let mut block = Entity::new("instance", Geometry::Instance(def_id), None);
        block.placement = Transform::translation([4., 0., 0.]);
        let id = block.id;
        let mut doc = Document::with_template_tolerance("mirror block pointer");
        doc.definitions.insert(def_id, def);
        doc.entities.insert(id, block);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(om9_core::top11::Top11Action::Mirror)
            .unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        for p in [[0., -5., 0.], [0., 5., 0.]] {
            let screen = camera.project(p, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
        }
        assert!(!app.controller.is_running());
        let doc = app.controller.document();
        assert_eq!(doc.entities.len(), 2);
        assert_eq!(doc.definitions.len(), 1);
        let copy = *app.controller.selected().iter().next().unwrap();
        let p = doc
            .member_transform(copy, &[leaf_id])
            .unwrap()
            .point([1., 2., 0.])
            .unwrap();
        assert!(
            (p[0] + 5.).abs() < 1e-3 && (p[1] - 2.).abs() < 1e-3,
            "{p:?}"
        );
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document().entities.len(), 1);
    }

    #[test]
    fn join_loop_and_tail_selects_curves_via_viewport() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        let ends = [
            ([-15., -10., 0.], [-5., -10., 0.]),
            ([-5., -10., 0.], [5., -10., 0.]),
            ([-5., -10., 0.], [-5., 0., 0.]),
            ([-5., 0., 0.], [-15., 0., 0.]),
            ([-15., 0., 0.], [-5., -10., 0.]),
        ];
        let mut entities: Vec<_> = (0..5)
            .map(|_| {
                Entity::new(
                    "edge",
                    Geometry::Curve(Curve::Line {
                        start: [0.; 3],
                        end: [1., 0., 0.],
                        domain: [0., 1.],
                    }),
                    None,
                )
            })
            .collect();
        entities.sort_by_key(|e| e.id);
        for (e, (start, end)) in entities.iter_mut().zip(ends) {
            e.geometry = Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0., 1.],
            });
        }
        let mut doc = Document::with_template_tolerance("join pointer");
        for entity in entities {
            doc.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        app.controller.submit("Join").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        for (i, (a, b)) in ends.iter().enumerate() {
            let point = std::array::from_fn(|j| (a[j] + b[j]) * 0.5);
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(screen[0] as f32, screen[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
            assert_eq!(app.controller.selected().len(), i + 1, "selection {i}");
        }
        let before = app.controller.document().clone();
        app.controller.submit("").unwrap();
        let joined = app.controller.document().entities.values().next().unwrap();
        let Geometry::Curve(Curve::Polycurve { segments, .. }) = &joined.geometry else {
            panic!("polycurve")
        };
        assert_eq!(segments.len(), 5);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }

    #[test]
    fn join_toleranced_gap_uses_midpoint_after_viewport_selection() {
        use om9_core::geometry::{Curve, Geometry};
        use om9_core::model::Entity;
        let edge = |start, end| {
            Entity::new(
                "edge",
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                }),
                None,
            )
        };
        let a = edge([-10., 0., 0.], [0., 0., 0.]);
        let b = edge([0.0008, 0., 0.], [10., 0., 0.]);
        let mut doc = Document::with_template_tolerance("join midpoint pointer");
        doc.entities.insert(a.id, a);
        doc.entities.insert(b.id, b);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.workspace.active = om9_scene::workspace::ViewId::LookingDown;
        app.controller.submit("Join").unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        for _ in 0..3 {
            window_frame(&mut app, &context, size, vec![]);
        }
        let rect = *app
            .view_rects
            .iter()
            .find(|(view, _)| view.title() == "Looking Down")
            .unwrap()
            .1;
        let camera = app
            .controller
            .view_cameras()
            .into_iter()
            .find(|(title, _, _)| *title == "Looking Down")
            .unwrap()
            .1;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        for point in [[-5., 0., 0.], [5., 0., 0.]] {
            let p = camera.project(point, viewport).unwrap().unwrap().screen;
            let pos = egui::pos2(p[0] as f32, p[1] as f32);
            window_frame(&mut app, &context, size, pointer(pos, true));
            window_frame(&mut app, &context, size, pointer(pos, false));
        }
        assert_eq!(app.controller.selected().len(), 2);
        let before = app.controller.document().clone();
        app.controller.submit("").unwrap();
        let Geometry::Curve(Curve::Polycurve { segments, .. }) = &app
            .controller
            .document()
            .entities
            .values()
            .next()
            .unwrap()
            .geometry
        else {
            panic!("polycurve")
        };
        let a = om9_core::curve_ops::curve_endpoints(
            &segments[0],
            &om9_core::budget::Limits::default(),
        )
        .unwrap()[1];
        let b = om9_core::curve_ops::curve_endpoints(
            &segments[1],
            &om9_core::budget::Limits::default(),
        )
        .unwrap()[0];
        assert!((a[0] - 0.0004).abs() < 1e-12 && (a[0] - b[0]).abs() < 1e-12);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}
