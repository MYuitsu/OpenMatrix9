// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compact Matrix workspace chrome. Buttons keep the shared command dispatcher;
//! decorative icon identity is separate from accepted operation availability.
use super::*;
use egui::{Align2, FontId, RichText};

const GREEN: Color32 = Color32::from_rgb(130, 180, 140);
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
        style.visuals.widgets.inactive.corner_radius = 0.into();
        style.visuals.widgets.hovered.corner_radius = 0.into();
        style.visuals.widgets.active.corner_radius = 0.into();
        style.visuals.selection.bg_fill = GREEN;
        style.visuals.selection.stroke = Stroke::new(1., Color32::BLACK);
        ctx.set_theme(egui::ThemePreference::Light);
        ctx.set_style_of(egui::Theme::Light, style);
    }
    pub(super) fn menu_ui(&mut self, ui: &mut egui::Ui) {
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
                    "File" => {
                        for kind in crate::file_lifecycle::FileKind::ALL {
                            let label=if kind==crate::file_lifecycle::FileKind::New {kind.label().to_owned()} else {format!("{}…",kind.label())};
                            let response=ui.button(label).on_hover_text("Native OM9 document (.om9)");
                            self.file_menu_items.insert(kind,response.rect);
                            if response.clicked() {self.invoke(kind.feature_id(),InvocationOrigin::Menu);ui.close();}
                        }
                        for item in ["Import…","Export…"] {ui.add_enabled(false,egui::Button::new(item)).on_hover_text("3DM file command UI not accepted yet");}
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
        gradient(
            ui.painter(),
            rect,
            Color32::from_rgb(199, 199, 186),
            Color32::from_rgb(151, 151, 141),
        );
        ui.painter().text(
            rect.left_center() + Vec2::new(2., 0.),
            Align2::LEFT_CENTER,
            id.bar_title(),
            FontId::proportional(11.),
            Color32::BLACK,
        );
        ui.painter().text(
            rect.right_center() - Vec2::new(6., 0.),
            Align2::CENTER_CENTER,
            if state.collapsed { "+" } else { "−" },
            FontId::proportional(12.),
            Color32::from_rgb(150, 55, 55),
        );
        self.section_rects.insert(id.key(), rect);
        if response.clicked() {
            let next = self.dock.toggle_section(id);
            self.dock_dirty = true;
            self.status.info(&format!(
                "{} {}",
                id.title(),
                if next.collapsed {
                    "rolled up"
                } else {
                    "expanded"
                }
            ));
        }
        if response.drag_started() {
            self.section_drag = Some(id);
        }
        let collapsed = self.dock.section(id).collapsed;
        if collapsed && id.keeps_controls_while_collapsed() {
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
    /// Apply the in-progress menu drag against the live column rectangles.
    pub(super) fn settle_section_drag(&mut self, pointer: egui::Pos2) {
        let Some(id) = self.section_drag else {
            return;
        };
        let sidebar = self.sidebar_rect;
        let dock = self.right_dock_rect;
        // Release inside the viewports floats the menu over the model.
        if self.view_rects.values().any(|rect| rect.contains(pointer)) {
            self.section_drag = None;
            self.float_section_at(id, pointer);
            return;
        }
        if dock.is_some_and(|rect| rect.contains(pointer)) {
            self.section_drag = None;
            self.dock_menu_section(id, DockSide::Right);
            return;
        }
        let Some(sidebar) = sidebar else {
            return;
        };
        if !sidebar.contains(pointer) {
            return;
        }
        let hovered = self
            .section_rects
            .iter()
            .filter(|(_, rect)| rect.contains(pointer))
            .filter_map(|(key, _)| SectionId::from_key(key))
            .find(|section| self.dock.section(*section).dock == DockSide::Left);
        let Some(hovered) = hovered else {
            return;
        };
        if hovered == id {
            return;
        }
        let current = self.dock.left_sections();
        if !current.contains(&id) {
            self.dock_menu_section(id, DockSide::Left);
            return;
        }
        let from = current
            .iter()
            .position(|section| *section == id)
            .unwrap_or(0) as i32;
        let to = current
            .iter()
            .position(|section| *section == hovered)
            .unwrap_or(0) as i32;
        if to != from {
            let _ = self.move_menu_section(id, to - from);
        }
    }
    /// Float a menu where the pointer released it, clamped by the layout model.
    fn float_section_at(&mut self, id: SectionId, pointer: egui::Pos2) {
        let size = Vec2::new(300., 240.);
        let x = (pointer.x - size.x / 2.).max(0.);
        let y = (pointer.y - 7.).max(0.);
        self.dock.float_section(
            id,
            FloatRect {
                x,
                y,
                width: size.x,
                height: size.y,
            },
        );
        self.dock_dirty = true;
        self.status
            .info(&format!("{} floated over the model", id.title()));
    }
    fn icon(&mut self, ui: &mut egui::Ui, name: &str, enabled: bool, help: &str) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::splat(22.),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let fill = if response.hovered() {
            Color32::from_rgb(122, 152, 184)
        } else {
            Color32::from_gray(95)
        };
        ui.painter().rect_filled(rect, 0, fill);
        bevel(ui.painter(), rect);
        self.icons.paint(ui, rect.shrink(2.), name);
        response.on_hover_text(if enabled {
            help.to_owned()
        } else {
            format!("{help} — operation unavailable")
        })
    }
    fn feature_icon(&mut self, ui: &mut egui::Ui, id: &str, name: &str) {
        let actual = self
            .features
            .iter()
            .find(|f| crate::icons::icon_name(&f.id) == Some(name))
            .map(|f| f.id.clone());
        let actual = if id == LINE || id == POINT {
            Some(id.to_owned())
        } else {
            actual
        };
        if let Some(actual) = actual {
            if self
                .icon(ui, name, Controller::availability(&actual).is_ok(), &actual)
                .clicked()
            {
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
        ui.visuals_mut().override_text_color = Some(Color32::WHITE);
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_gray(105);
        ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::from_gray(105);
        // The column order, visibility and roll-up state come from the typed
        // layout, so the documented Matrix order survives a session.
        let sections = self.dock.left_sections();
        let scroll = std::mem::take(&mut self.sidebar_scroll);
        egui::ScrollArea::vertical()
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
                for (row, categories) in CATEGORIES.chunks(6).enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        for (column, category) in categories.iter().enumerate() {
                            let i = row * 6 + column;
                            let colors = [
                                Color32::YELLOW,
                                Color32::from_rgb(30, 215, 214),
                                Color32::from_rgb(255, 110, 100),
                                Color32::from_rgb(230, 200, 110),
                                Color32::from_rgb(140, 175, 255),
                                Color32::from_rgb(225, 130, 235),
                            ];
                            let text = RichText::new(format!("•{category}"))
                                .color(colors[i % 6])
                                .size(10.);
                            if ui.add(egui::Button::new(text).frame(false)).clicked() {
                                self.main_category = i;
                                self.focus_cmd = true;
                            }
                        }
                    });
                }
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
            Color32::from_rgb(154, 185, 158),
            Color32::from_rgb(215, 225, 217),
        );
        let tab = TABS[self.toolbar_tab];
        egui::ScrollArea::horizontal()
            .id_salt(("toolbar-palette", tab))
            .max_height(24.)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(22.);
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
            ui.horizontal(|ui| {
                let arrow = if state.collapsed { "+" } else { "−" };
                let response = ui
                    .selectable_label(!state.collapsed, format!("{arrow} {}", id.bar_title()))
                    .on_hover_text("Collapse or expand this menu");
                self.section_rects.insert(id.key(), response.rect);
                toggle = response.clicked();
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
                self.dock.toggle_section(*id);
                self.dock_dirty = true;
            }
            if let Some(delta) = move_by {
                self.move_menu_section(*id, delta);
            }
            if dock_left {
                self.dock_menu_section(*id, DockSide::Left);
                continue;
            }
            if float {
                self.float_section_at(
                    *id,
                    ui.max_rect().center() + Vec2::new(-40. * index as f32, 40.),
                );
                continue;
            }
            if self.dock.section(*id).collapsed {
                continue;
            }
            self.sidebar_section_ui(ui, *id);
        }
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
            let window = egui::Window::new(id.title())
                .id(egui::Id::new(("om9-float-menu", id.key())))
                .default_pos([stored.x, stored.y])
                .default_size([stored.width, stored.height])
                .open(&mut open)
                .show(ctx, |ui| {
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
                    self.dock.float_section(
                        id,
                        FloatRect {
                            x: rect.min.x,
                            y: rect.min.y,
                            width: rect.width(),
                            height: rect.height(),
                        },
                    );
                    self.dock_dirty = true;
                }
            }
            if dock_left {
                self.dock_menu_section(id, DockSide::Left);
            } else if dock_right {
                self.dock_menu_section(id, DockSide::Right);
            } else if !open {
                // Closing a floating menu returns it to the documented home
                // column instead of losing it.
                self.dock_menu_section(id, DockSide::Left);
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
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{}: {message}", kind.label()))
                    .color(color)
                    .size(11.),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if kind.is_failure() && ui.small_button("Dismiss").clicked() {
                    self.status.acknowledge();
                }
                ui.label(
                    RichText::new(summary)
                        .color(Color32::from_gray(205))
                        .size(10.),
                );
            });
        });
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
