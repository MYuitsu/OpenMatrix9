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
                    }
                    "Edit" => {
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
                                    let available = Controller::availability(&f.id).is_ok();
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
        if matches!(name, "SolidBooleanSplit" | "SolidCreateSolid") {
            let enabled = Controller::availability(name).is_ok();
            let label = if name == "SolidBooleanSplit" { "Boolean Split" } else { "Create Solid" };
            if self.icon(ui, name, enabled, label).clicked() {
                self.invoke(name, InvocationOrigin::Menu);
            }
            return;
        }
        let curve = crate::curve_menu::by_icon(name);
        let surface = crate::surface_menu::by_icon(name);
        let catalog_feature = curve
            .map(|binding| binding.feature)
            .or_else(|| surface.map(|binding| binding.feature));
        let catalog_enabled = curve.is_none_or(crate::curve_menu::CurveMenuBinding::enabled)
            && surface.is_none_or(crate::surface_menu::SurfaceMenuBinding::enabled);
        let catalog_reason = curve
            .map(|binding| binding.disabled_reason)
            .or_else(|| surface.map(|binding| binding.disabled_reason));
        let actual = catalog_feature
            .map(str::to_owned)
            .or_else(|| crate::icons::feature_for(name).map(str::to_owned))
            .or_else(|| {
                self.features
                    .iter()
                    .find(|f| crate::icons::icon_name(&f.id) == Some(name))
                    .map(|f| f.id.clone())
            })
            .or_else(|| (id == LINE || id == POINT).then(|| id.to_owned()));
        let feature = actual
            .as_ref()
            .and_then(|id| self.features.iter().find(|f| &f.id == id).cloned());
        if let Some(actual) = actual {
            let enabled = Controller::availability(&actual).is_ok() && catalog_enabled;
            let help = if enabled {
                feature
                    .as_ref()
                    .map_or(if name == "SolidGVDMultiPipe" { "Solid Multi Pipe" } else { name }, |f| f.name.as_str())
                    .to_owned()
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
            } else { feature
                .as_ref()
                .and_then(|f| f.right_click.as_deref())
                .filter(|command| Controller::availability(command).is_ok()) };
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
                        let response = response.on_hover_text(format!("{label} menu"));
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
                            *label,
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
                            let galley = ui.painter().layout_no_wrap(
                                (*category).to_owned(),
                                font.clone(),
                                text_color,
                            );
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::new(card_width, 18.), Sense::click());
                            let response =
                                response.on_hover_text(format!("{category} command category"));
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
        self.builder_slot_rect = Some(ui.max_rect());
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
            if ui
                .add_enabled(editable, egui::Button::new("Apply"))
                .clicked()
            {
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
            let result = self.controller.begin_builder_from_history(object).map(|_| ());
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
                                    let result = self.controller
                                        .capture_view_menu(self.controller.workspace.active)
                                        .and_then(|capture| self.controller.apply_view_menu(&capture, action));
                                    self.feedback(result);
                                }
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
            "ClayooPrimitivesPlane", "ClayooPrimitivesBox", "ClayooPrimitivesTorus",
            "ClayooPrimitivesSphere", "ClayooPrimitivesCylinder", "ClayooPrimitivesCone",
        ],
        "SubD Creation" => &[
            "ClayooCreationAppendFace", "ClayooCreationFromCurve", "ClayooCreationFrom2Curves",
            "ClayooCreationLoft", "ClayooCreationSweep1Rail", "ClayooCreationSweep2Rails",
            "ClayooCreationPipe", "ClayooCreationRevolve", "ClayooCreateIsocurves",
            "ClayooCreationRing", "ClayooCreationSignetRing", "ClayooCreationClayooBezel",
            "ClayooCreationFromSurface", "ClayooCreationFromMesh", "ClayooCreationFromTSplines",
            "ClayooCreationToNurbs", "ClayooCreationClayooLibrary",
        ],
        "SubD Edition" => &[
            "ClayooEditDivide", "ClayooEditSplitSides", "ClayooEditMatch", "ClayooEditExtract",
            "ClayooEditOffset", "ClayooEditInset", "ClayooEditExtrude", "ClayooEditWeld",
            "ClayooEditCollapse", "ClayooEditBridge", "ClayooEditFill", "ClayooEditShell",
            "ClayooEditKnife", "ClayooEditWrap", "ClayooEditSymmetry", "ClayooEditFlipNormals",
            "ClayooEditCrease", "ClayooEditProjectToPlane", "ClayooEditSubdivisionLevel", "ClayooEditItem",
        ],
        "SubD Selection" => &[
            "ClayooSelectionSelectAll", "ClayooSelectionSelectNone", "ClayooSelectionInvertSelection",
            "ClayooSelectionSelectCreased", "ClayooSelectionSelectNaked", "ClayooSelectionSelectCoplanar",
            "ClayooSelectionPaintSelection", "ClayooSelectionGrowSelection", "ClayooSelectionShrink",
            "ClayooSelectUV", "ClayooSelectionSelectRing", "ClayooSelectionSelectLoop",
            "ClayooSelectionSetPlane", "ClayooSelectionResetPlane", "ClayooSelectionSelectionSets",
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
            "Wireframe", "Shaded", "Standard Shade", "Display Rendered Viewport",
            "Dial_Ghosted", "Display X-Ray Viewport", "Dial_Tech Shade",
            "Display Pen Viewport", "Display Artistic Viewport", "Display Render Mesh Settings",
            "Display Flat Shade", "ShadeSelectedOnlyON", "Display Object Shading", "ViewCaptureToFile",
            "Visibility Add Clipping Plane", "Visibility Disable Clipping Plane", "Display Panel",
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
        assert_eq!(&toolbar_palette("Standard")[..32], &[
            "New", "Open", "Save", "FilePrint", "Standard Document Properties",
            "Standard Cut", "Standard Copy", "Standard Paste", "Undo", "Standard Pan",
            "Standard Rotate View", "ViewZoomZoomDynamic", "ViewZoomZoomWindow", "Fit",
            "ViewZoomZoomSelected", "Standard Undo View", "Standard Four Views", "Standard Right View",
            "Standard CPlane Origin", "Standard Object Snaps", "Standard Select All", "Standard Hide Objects",
            "Standard Lock Objects", "Layers", "ObjectProperties", "Standard Shade", "Shaded",
            "Standard Render", "Standard Spotlight", "RhinoOptions", "Standard Linear Dimension", "Standard Help",
        ]);
        for name in &toolbar_palette("Standard")[..32] {
            assert!(crate::icons::icon_name(name).is_some(), "missing icon for {name}");
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
        assert_eq!(names, &[
            "Visibility Hide Objects", "Visibility Show Objects", "Visibility Show Selected Objects",
            "Visibility Invert Selection and Hide", "Visibility Swap Hidden Objects",
            "Visibility Lock Objects", "Visibility Unlock Objects", "Visibility Unlock Selected Objects",
            "Visibility Invert Selection and Lock", "Visibility Swap Locked Objects",
            "Visibility Invert and Hide Control Points", "Visibility Hide Control Points",
            "Visibility Add Clipping Plane", "Visibility Disable Clipping Plane",
            "Visibility Hide A", "Visibility Hide B", "Visibility Hide C", "Visibility Hide Custom",
        ]);
        let mut app = MatrixApp::new(Document::new("visibility toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Visibility").unwrap();
        for _ in 0..3 { window_frame(&mut app, &context, size, vec![]); }
        for name in names {
            assert!(crate::icons::icon_name(name).is_some(), "{name}");
            assert!(crate::icons::feature_for(name).is_none(), "{name}");
            let rect: Rect = context.memory(|m| m.data.get_temp(egui::Id::new(("toolbar-test", "Visibility", name))).unwrap());
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert!(!app.controller.is_running(), "{name}");
            assert!(app.controller.document().entities.is_empty(), "{name}");
        }
    }

    #[test]
    fn display_toolbar_switches_only_active_view_with_existing_view_owner() {
        use om9_scene::workspace::DisplayMode;
        assert_eq!(toolbar_palette("Display"), &[
            "Wireframe", "Shaded", "Standard Shade", "Display Rendered Viewport",
            "Dial_Ghosted", "Display X-Ray Viewport", "Dial_Tech Shade",
            "Display Pen Viewport", "Display Artistic Viewport", "Display Render Mesh Settings",
            "Display Flat Shade", "ShadeSelectedOnlyON", "Display Object Shading", "ViewCaptureToFile",
            "Visibility Add Clipping Plane", "Visibility Disable Clipping Plane", "Display Panel",
        ]);
        let mut app = MatrixApp::new(Document::new("display toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "Display").unwrap();
        for _ in 0..3 { window_frame(&mut app, &context, size, vec![]); }
        let target = app.controller.workspace.active;
        let original = app.controller.workspace.views.clone();
        for (name, mode) in [("Shaded", DisplayMode::Shaded), ("Wireframe", DisplayMode::Wireframe)] {
            let rect: Rect = context.memory(|m| m.data.get_temp(egui::Id::new(("toolbar-test", "Display", &name))).unwrap());
            window_frame(&mut app, &context, size, pointer(rect.center(), true));
            window_frame(&mut app, &context, size, pointer(rect.center(), false));
            assert_eq!(app.controller.workspace.views[&target].mode, mode, "{name}");
            for (id, view) in &original {
                if *id != target { assert_eq!(app.controller.workspace.views[id], *view); }
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
        for _ in 0..3 { window_frame(&mut app, &context, size, vec![]); }
        for name in ["Display Rendered Viewport", "Display X-Ray Viewport", "Display Pen Viewport", "Display Artistic Viewport", "Display Render Mesh Settings", "Display Flat Shade", "Display Object Shading", "Display Panel"] {
            assert!(crate::icons::feature_for(name).is_none(), "{name}");
            let rect: Rect = context.memory(|m| m.data.get_temp(egui::Id::new(("toolbar-test", "Display", &name))).unwrap());
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
        let names = ["SubD Primitives", "SubD Creation", "SubD Edition", "SubD Selection"];
        let all: Vec<_> = names.iter().flat_map(|name| subd_flyout_items(name).copied()).collect();
        let original: std::collections::BTreeSet<_> = crate::menu_catalog::SUBD.iter().copied().collect();
        let grouped: std::collections::BTreeSet<_> = all.iter().copied().collect();
        assert_eq!(grouped, original);
        assert_eq!(all.len(), grouped.len(), "every original icon appears exactly once");

        let mut app = MatrixApp::new(Document::new("subd toolbar")).unwrap();
        let context = egui::Context::default();
        let size = Vec2::new(1280., 720.);
        app.toolbar_tab = TABS.iter().position(|tab| *tab == "SubD").unwrap();
        for _ in 0..3 { window_frame(&mut app, &context, size, vec![]); }
        for name in names {
            let rect: Rect = context.memory(|m| m.data.get_temp(egui::Id::new(("toolbar-test", "SubD", &name))).unwrap());
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
    fn top11_multi_split_cmd_undo_redo_from_clean_document() {
        let mut app = MatrixApp::new(Document::new("split ui history")).unwrap();
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
}
