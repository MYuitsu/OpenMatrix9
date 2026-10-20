// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compact Matrix workspace chrome. Buttons keep the shared command dispatcher;
//! decorative icon identity is separate from accepted operation availability.
use super::*;
use egui::{Align2, FontId, RichText};

const GREEN: Color32 = Color32::from_rgb(130, 180, 140);
const PANEL_BG: Color32 = Color32::from_rgb(10, 14, 19);
const PANEL_ALT: Color32 = Color32::from_rgb(16, 23, 31);
const PANEL_EDGE: Color32 = Color32::from_rgb(42, 57, 73);
const PANEL_INK: Color32 = Color32::from_rgb(230, 237, 243);
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
const TABS: [&str; 20] = [
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
];

impl MatrixApp {
    pub(super) fn configure_style(&mut self, ctx: &egui::Context) {
        if self.style_ready {
            return;
        }
        self.style_ready = true;
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
        style.visuals.text_options.font_hinting = false;
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
            PANEL_INK,
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
                accent,
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
                for name in ["GridSnap", "Ortho", "Planar"] {
                    self.icon(ui, name, false, name);
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
    fn icon_slot(
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
            PANEL_EDGE
        } else {
            PANEL_ALT
        };
        ui.painter().rect_filled(rect, 4, fill);
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            4,
            Stroke::new(
                1.,
                if enabled {
                    PANEL_EDGE
                } else {
                    Color32::from_rgb(38, 52, 68)
                },
            ),
            egui::StrokeKind::Inside,
        );
        self.icons.paint(ui, rect.shrink(3.), name);
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
        let feature = self
            .features
            .iter()
            .find(|f| crate::icons::icon_name(&f.id) == Some(name))
            .cloned();
        let actual = feature
            .as_ref()
            .map(|f| f.id.clone())
            .or_else(|| (id == LINE || id == POINT).then(|| id.to_owned()));
        if let Some(actual) = actual {
            let enabled = Controller::availability(&actual).is_ok();
            let right = feature
                .as_ref()
                .and_then(|f| f.right_click.as_deref())
                .filter(|command| Controller::availability(command).is_ok());
            if self.icon_slot(ui, name, enabled, &actual, right).clicked() {
                self.invoke(&actual, InvocationOrigin::Menu);
            }
        } else {
            self.icon(ui, name, false, name);
        }
    }
    fn icon_row(&mut self, ui: &mut egui::Ui, names: &[&str]) {
        ui.horizontal_wrapped(|ui| {
            for name in names {
                // Display, snaps and settings are separate command domains.
                // Reusing an authored glyph never makes its geometry action
                // available here. These adapters have no accepted operation yet.
                self.icon(ui, name, false, name);
            }
        });
    }
    pub(super) fn sidebar_ui(&mut self, ui: &mut egui::Ui) {
        ui.visuals_mut().override_text_color = Some(PANEL_INK);
        ui.visuals_mut().widgets.inactive.bg_fill = PANEL_ALT;
        ui.visuals_mut().widgets.inactive.weak_bg_fill = PANEL_ALT;
        ui.visuals_mut().widgets.hovered.bg_fill = PANEL_EDGE;
        ui.visuals_mut().widgets.hovered.weak_bg_fill = PANEL_EDGE;
        ui.visuals_mut().widgets.active.bg_fill = ACCENT;
        ui.visuals_mut().widgets.active.weak_bg_fill = ACCENT;
        // The column order, visibility and roll-up state come from the typed
        // layout, so the documented Matrix order survives a session.
        let sections = self.dock.left_sections();
        let scroll = std::mem::take(&mut self.sidebar_scroll);
        self.rail_icon_rects.clear();
        let scrolled = egui::ScrollArea::vertical()
            .id_salt("matrix-sidebar-sections")
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
        self.left_content_bottom = scrolled.inner_rect.bottom();
        self.sidebar_scroll = ui.ctx().memory(|memory| {
            memory
                .data
                .get_temp::<f32>(egui::Id::new("matrix-sidebar-sections-offset"))
                .unwrap_or(0.)
        });
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
                ui.horizontal_wrapped(|ui| {
                    for label in ["File", "View", "Utilities", "Measure", "Custom"] {
                        ui.add_enabled(false, egui::Button::new(label).frame(false))
                            .on_hover_text("Menu customization pending");
                    }
                });
                self.main_category_rects.clear();
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3., 3.);
                    for (i, category) in CATEGORIES.iter().enumerate() {
                        let accent = CATEGORY_COLORS[i];
                        let selected = self.main_category == i;
                        let text = RichText::new(*category)
                            .color(category_text(accent))
                            .size(11.);
                        let button = egui::Button::new(text)
                            .fill(if selected {
                                SELECTED_CARD
                            } else {
                                Color32::from_rgb(24, 35, 48)
                            })
                            .corner_radius(4)
                            .stroke(Stroke::new(1., if selected { accent } else { PANEL_EDGE }))
                            .min_size(Vec2::new(0., 20.));
                        let response = ui
                            .add(button)
                            .on_hover_text(format!("{category} command category"));
                        self.main_category_rects.insert(i, response.rect);
                        if response.clicked() {
                            self.main_category = i;
                            self.focus_cmd = true;
                        }
                    }
                });
                let category = CATEGORIES[self.main_category];
                let palette = category_palette(category);
                ui.horizontal_wrapped(|ui| {
                    for name in palette {
                        self.feature_icon(ui, "", name);
                    }
                });
                if palette.is_empty() {
                    ui.label(format!("{category} tools unavailable"));
                }
            }
            SectionId::Display => {
                self.icon_row(
                    ui,
                    &[
                        "Grid",
                        "Wireframe",
                        "Shaded",
                        "Sphere",
                        "Gem",
                        "Render",
                        "Layers",
                        "Fit",
                        "Rotate",
                        "Properties",
                        "Settings",
                        "Project",
                    ],
                );
                ui.columns(2, |columns| {
                    columns[0].add_enabled(false, egui::Button::new("Coarse Mesh ▾"));
                    columns[1].add_enabled(false, egui::Button::new("Wireframe ▾"));
                });
            }
            SectionId::Snaps => {
                self.icon_row(
                    ui,
                    &[
                        "SnapEnd",
                        "SnapMid",
                        "SnapPoint",
                        "SnapCenter",
                        "GridSnap",
                        "Ortho",
                        "Planar",
                        "Settings",
                    ],
                );
                ui.horizontal_wrapped(|ui| {
                    for text in ["Grid", "Ortho", "Planar", "0.1", "0.25", "0.5", "1.0"] {
                        ui.add_enabled(false, egui::Button::new(text));
                    }
                });
            }
            SectionId::InfoSettings => {
                self.icon_row(
                    ui,
                    &[
                        "Settings",
                        "Properties",
                        "Project",
                        "Layers",
                        "Save",
                        "Open",
                        "Builder",
                        "Gem",
                        "Fit",
                        "Move",
                        "Rotate",
                        "Scale",
                    ],
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Units: {:?}", self.controller.document().page_unit));
                    ui.separator();
                    ui.label(format!(
                        "Objects: {}",
                        self.controller.document().entities.len()
                    ));
                });
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
        ui.horizontal_wrapped(|ui| {
            let (list, _) = ui.allocate_exact_size(Vec2::new(62., 76.), Sense::hover());
            ui.painter().rect_filled(list, 0, Color32::from_gray(74));
            ui.painter().text(
                list.min + Vec2::new(2., 3.),
                Align2::LEFT_TOP,
                "Default",
                FontId::proportional(10.),
                Color32::WHITE,
            );
            for (index, name) in ["Project", "Gem", "Emerald"].iter().enumerate() {
                ui.allocate_ui_with_layout(
                    Vec2::new(62., 79.),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::new(62., 60.), Sense::hover());
                        gradient(
                            ui.painter(),
                            rect,
                            Color32::from_rgb(20, 47, 160),
                            Color32::from_rgb(5, 12, 45),
                        );
                        self.icons.paint(ui, rect.shrink(12.), name);
                        ui.painter().text(
                            rect.min + Vec2::new(2., 2.),
                            Align2::LEFT_TOP,
                            (index + 1).to_string(),
                            FontId::proportional(10.),
                            Color32::WHITE,
                        );
                        response.on_hover_text("Project preview unavailable");
                        ui.add_enabled(false, egui::Button::new("↑  ▣  ×"));
                    },
                );
            }
        });
        ui.horizontal_wrapped(|ui| {
            for name in ["Master", "Creation", "Parts", "Render", "Output"] {
                let (rect, response) = ui.allocate_exact_size(Vec2::new(55., 49.), Sense::hover());
                ui.painter().rect_filled(rect, 0, Color32::from_gray(79));
                bevel(ui.painter(), rect);
                ui.painter().text(
                    rect.min + Vec2::new(2., 3.),
                    Align2::LEFT_TOP,
                    name,
                    FontId::proportional(10.),
                    Color32::WHITE,
                );
                response.on_hover_text("Project operation unavailable");
            }
        });
    }
    /// BUILDERS: the reserved area in the space below the Project Manager. M04
    /// owns the real builder; M01 keeps the slot visible and non-interactive.
    fn builder_slot_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let response = ui
                .add_enabled(false, egui::Button::new("Builder (reserved)"))
                .on_hover_text(
                    "Builders open in the space below the Project Manager; M04 implements them",
                );
            self.builder_slot_rect = Some(response.rect);
        });
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                ui.label("Gem");
                let (rect, response) = ui.allocate_exact_size(Vec2::new(104., 76.), Sense::hover());
                ui.painter().rect_filled(rect, 0, Color32::from_gray(53));
                self.icons.paint(ui, rect.shrink(16.), "Emerald");
                response.on_hover_text("No active gem; builder operation unavailable");
            });
            ui.add_space(36.);
            ui.vertical(|ui| {
                for name in ["New Emerald", "Size ▾", "Show Diag.", "Reset"] {
                    ui.add_enabled(false, egui::Button::new(name))
                        .on_hover_text("Emerald builder operation unavailable");
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            for name in ["Emerald", "Scale", "Move", "Rotate", "Gem"] {
                self.icon(ui, name, false, "Builder parameter");
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled(
                false,
                egui::Slider::new(&mut 0.5f32, 0.0..=1.0).show_value(false),
            );
            ui.label("LENGTH");
            ui.add_enabled(false, egui::Button::new("0.5 mm"));
        });
    }
    pub(super) fn toolbars_ui(&mut self, ui: &mut egui::Ui) {
        self.toolbar_icon_edge = (dock_layout::ICON_BASE_EDGE * self.screen_icon_scale)
            .clamp(dock_layout::ICON_MIN_EDGE, 32.)
            .round();
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
                            _ => {
                                self.icon(ui, name, false, name);
                            }
                        }
                    }
                });
            });
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
        ui.set_min_width(ui.available_width());
        self.right_dock_icon_rects.clear();
        let sections = self.dock.right_sections();
        if sections.is_empty() {
            ui.label(
                RichText::new("Drag a menu title bar here to dock it")
                    .size(10.)
                    .color(Color32::from_gray(190)),
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
                            format!("{arrow} {}", id.bar_title()),
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
            let mut builder = egui::Window::new(id.title())
                .id(egui::Id::new(("om9-float-menu", id.key())))
                .default_pos([stored.x, stored.y])
                .default_size([stored.width, stored.height])
                .open(&mut open);
            if let Some(work) = work {
                builder = builder.constrain_to(work);
            }
            let window = builder.show(ctx, |ui| {
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
    match category {
        "Curve" => &[
            "Polyline",
            "Line",
            "Interpolate",
            "Circle",
            "Rectangle",
            "Ellipse",
            "Point",
            "Offset",
            "ProjectCurve",
            "Trim",
            "Join",
            "Split",
        ],
        "Surface" => &["Sweep", "Loft", "Revolve"],
        "Solid" => &["Box", "Sphere", "Union", "Difference", "Intersection"],
        "Transform" => &["Move", "Rotate", "Scale", "Mirror", "Duplicate"],
        "Builder" => &["Builder", "RingRail"],
        "Tools" => &["RingRail", "Fit", "Properties"],
        "Gems" => &["Gem", "Emerald"],
        "Setting" => &["Settings", "Properties", "Layers"],
        "Cutters" => &["Cutters"],
        "Render" => &["Render"],
        // There is no accepted operation or selected authored palette for these
        // groups yet. Show their availability rather than unrelated solids.
        _ => &[],
    }
}
fn toolbar_palette(tab: &str) -> &'static [&'static str] {
    match tab {
        "Standard" => &[
            "New",
            "Open",
            "Save",
            "Undo",
            "Redo",
            "Line",
            "Point",
            "Circle",
            "Rectangle",
        ],
        "Set View" => &["Fit"],
        "Display" => &["Wireframe", "Shaded", "Grid", "Properties"],
        "Visibility" => &["Layers", "LayerShow", "LayerHide", "LayerLock"],
        "Transform" => category_palette("Transform"),
        "Curve Tools" => category_palette("Curve"),
        "Surface Tools" => category_palette("Surface"),
        "Solid Tools" => category_palette("Solid"),
        "Render Tools" => category_palette("Render"),
        "Tools" => category_palette("Tools"),
        "Settings" => category_palette("Setting"),
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
fn category_text(accent: Color32) -> Color32 {
    let luminance =
        (0.2126 * accent.r() as f32 + 0.7152 * accent.g() as f32 + 0.0722 * accent.b() as f32)
            / 255.;
    let mix = if luminance < 0.42 { 0.42 } else { 0.18 };
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
}
