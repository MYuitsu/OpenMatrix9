// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::f6::{
    Action, Context, CustomEntry, CustomMenu,
    catalog::{CATALOG, Category, Group, Mode},
};
use crate::session::{gem, gem_cluster, gem_edits, gem_pave, gem_placement, ring_tools};

pub(super) struct Editor {
    selected: Option<usize>,
    scroll_to_selected: bool,
    import_path: String,
    import_status: String,
    menu: CustomMenu,
    group: String,
    mode: Mode,
}

#[derive(Clone, Copy)]
pub(super) struct CachedGroup {
    mode: Mode,
    document: om9_core::identity::DocumentId,
    epoch: u64,
    selection: u64,
    group: Option<&'static Group>,
}

pub(super) struct SelectionReport {
    key: (om9_core::identity::DocumentId, u64, u64, u64),
    counts: Vec<String>,
    metals: std::collections::BTreeSet<om9_core::identity::MaterialId>,
    gems: Vec<(usize, String, [u8; 4])>,
}

pub(super) struct PaintStroke {
    view: ViewId,
    camera: om9_scene::camera::Camera,
    viewport: Viewport,
    epoch: u64,
    last: Option<egui::Pos2>,
}

impl MatrixApp {
    pub(super) fn paint_cage_pointer(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
        kind: om9_scene::scene::PickKind,
        current: bool,
    ) {
        if !ui.input(|i| i.focused && (i.pointer.primary_down() || i.pointer.primary_released())) {
            self.cage_paint_stroke = None;
            return;
        }
        if self
            .cage_paint_stroke
            .as_ref()
            .is_some_and(|s| s.view != view.id)
        {
            return;
        }
        let epoch = self.controller.coordinator().epoch();
        if !current
            || self.viewport_menu_owns_frame
            || !ui.input(|i| i.focused)
            || self.cage_paint_stroke.as_ref().is_some_and(|s| {
                s.epoch != epoch || s.camera != view.camera || s.viewport != viewport
            })
        {
            self.cage_paint_stroke = None;
            return;
        }
        let owns_press = response.is_pointer_button_down_on()
            || response.clicked()
            || response.drag_stopped_by(egui::PointerButton::Primary);
        let tolerance = pick_tolerance(ui);
        let result = ui.input(|input| -> om9_core::Result<()> {
            for event in &input.events {
                let (pos, release) = match event {
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        ..
                    } if owns_press && rect.contains(*pos) => {
                        self.cage_paint_stroke = Some(PaintStroke {
                            view: view.id,
                            camera: view.camera.clone(),
                            viewport,
                            epoch,
                            last: None,
                        });
                        (*pos, false)
                    }
                    egui::Event::PointerMoved(pos) => (*pos, false),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        ..
                    } => (*pos, true),
                    egui::Event::PointerGone => {
                        self.cage_paint_stroke = None;
                        continue;
                    }
                    _ => continue,
                };
                let Some(stroke) = self.cage_paint_stroke.as_mut() else {
                    continue;
                };
                if rect.contains(pos) {
                    let start = stroke.last.replace(pos);
                    if start != Some(pos) {
                        let start = start.unwrap_or(pos);
                        let steps = (start.distance(pos) / 2.).ceil().max(1.) as usize;
                        let mut last = None;
                        for step in 1..=steps {
                            let sample = start.lerp(pos, step as f32 / steps as f32);
                            let hit = self
                                .controller
                                .scene()
                                .ok_or(Error::Missing("paint selection scene"))?
                                .pick_kind(
                                    [f64::from(sample.x), f64::from(sample.y)],
                                    &view.camera,
                                    viewport,
                                    tolerance,
                                    PickIntent::Select,
                                    kind,
                                )?;
                            let Some(hit) = hit else {
                                continue;
                            };
                            if !hit.path.is_empty()
                                || !self
                                    .controller
                                    .document()
                                    .entities
                                    .get(&hit.root)
                                    .is_some_and(om9_core::model::is_tspline_box_cage)
                            {
                                continue;
                            }
                            let reference = crate::session::SelectedSubobject::from_hit(&hit);
                            if last.as_ref() == Some(&reference) {
                                continue;
                            }
                            self.controller.select_subobject_ref(reference.clone())?;
                            last = Some(reference);
                        }
                    }
                } else {
                    stroke.last = None;
                }
                if release {
                    self.cage_paint_stroke = None;
                }
            }
            Ok(())
        });
        if let Err(error) = result {
            self.cage_paint_stroke = None;
            self.feedback(Err(error));
        }
    }

    fn workspace_section(name: &str) -> Option<SectionId> {
        match name
            .trim()
            .trim_start_matches('_')
            .to_ascii_lowercase()
            .as_str()
        {
            "gvdisplay" => Some(SectionId::Display),
            "gvsnaps" => Some(SectionId::Snaps),
            "gviconhistory" => Some(SectionId::IconHistory),
            "gvmain" => Some(SectionId::MainMenu),
            "gvlayers" => Some(SectionId::Layers),
            "gvinfoandsettings" => Some(SectionId::InfoSettings),
            _ => None,
        }
    }

    pub(super) fn open_workspace_section(&mut self, name: &str) -> bool {
        let Some(id) = Self::workspace_section(name) else {
            return false;
        };
        self.dock.set_section_visible(id, true);
        self.dock.set_section_collapsed(id, false);
        self.dock_dirty = true;
        true
    }
    pub(super) fn open_context_actions(&mut self, ctx: &egui::Context) {
        self.f6_open = true;
        self.dock.register_tool(ToolPanel::F6);
        let id = SectionId::Tool(ToolPanel::F6);
        let state = self.dock.section(id);
        if !self.f6_pinned
            && state.dock == DockSide::Floating
            && let (Some(anchor), Some(mut rect)) = (ctx.pointer_hover_pos(), state.float)
        {
            // Matrix places the mode strip above the invocation point so the
            // pointer starts near the actions when the popup opens.
            rect.x = anchor.x - 10.;
            rect.y = anchor.y - 100.;
            self.dock
                .float_section(id, self.work_area.map_or(rect, |area| area.clamp(rect)));
        }
    }

    fn f6_group(&self) -> Option<&'static Group> {
        if let Some(kind) = self.controller.tspline_component_kind().or_else(||self.controller.from_lines_faces_selected().then_some(om9_scene::scene::PickKind::Face))
            && let Some(group) = CATALOG.tspline_component_group(self.f6_mode, kind)
        {
            return Some(group);
        }
        if self.controller.grips_selected()
            && let Some(grip) = self.controller.selected_grips().first()
        {
            let (class, _) = self.f6_grip_kind(grip);
            if let Some(group) = CATALOG.grip_group(self.f6_mode, class) {
                return Some(group);
            }
        }
        let document = self.controller.document();
        let epoch = self.controller.coordinator().epoch();
        let selection = self.controller.selection_revision();
        if let Some(cached) = self.f6_group_cache.get()
            && cached.mode == self.f6_mode
            && cached.document == document.id
            && cached.epoch == epoch
            && cached.selection == selection
        {
            return cached.group;
        }
        let group = CATALOG.group(self.f6_mode, document, &self.controller.selected_ordered());
        self.f6_group_cache.set(Some(CachedGroup {
            mode: self.f6_mode,
            document: document.id,
            epoch,
            selection,
            group,
        }));
        group
    }

    /// Resolve the complete action before enabling it. Some input sequences have their
    /// own implemented CMD parser. An unknown suffix must never run a bare command.
    pub fn resolve_f6_action(&self, script: &str, label: &str) -> Action {
        let text = script.trim();
        let text = text.strip_prefix("noecho ").unwrap_or(text);
        let mut action = self.resolve_context_command(text.trim_start_matches('!').trim(), label);
        action.cancel_first = text.starts_with('!');
        action
    }

    fn resolve_context_command(&self, script: &str, label: &str) -> Action {
        let token = script.trim_start_matches('_');
        if token.eq_ignore_ascii_case("delete") && label.eq_ignore_ascii_case("Delete Face(s)") {
            return if self.controller.tspline_faces_selected() {
                Action::enabled("DeleteTSplineFaces", label)
            } else {
                Action::disabled("DeleteTSplineFaces", label, "Select T-Spline faces")
            };
        }
        if Self::workspace_section(token).is_some() {
            return Action::enabled(token, label);
        }
        if token.eq_ignore_ascii_case("gvSignet") && label.eq_ignore_ascii_case("Edit") {
            let selected = self.controller.selected();
            let doc = self.controller.document();
            let editable = selected.len() == 1
                && selected.first().is_some_and(|object| {
                    let recipe = doc
                        .history
                        .owner(*object)
                        .and_then(|id| doc.history.recipes.get(&id));
                    recipe.is_some_and(|r| {
                        r.command == om9_core::ring_builder::SIGNET_RING_COMMAND
                            && !r.policy.lock
                            && r.state != om9_core::history::EvaluationState::Detached
                    }) && doc.entities.get(object).is_some_and(|e| {
                        doc.permissions_for_entity(e, false)
                            .is_ok_and(|p| p.editable)
                    })
                });
            if !editable {
                return Action::disabled(
                    "EditBuilderByHistory",
                    label,
                    "Select a Signet with construction history",
                );
            }
            return Action::enabled("EditBuilderByHistory", label);
        }
        if label.starts_with("Edit ")
            && matches!(
                token.to_ascii_lowercase().as_str(),
                "gvbeadoncrv" | "gvmetalpiece" | "gvprongadder" | "gvbezel"
            )
        {
            return Action::disabled(
                script,
                label,
                "Editing this existing builder requires its saved construction parameters",
            );
        }
        if let Some(preset) = crate::session::context_presets::ContextPreset::parse(token) {
            if let Some(reason) = preset.disabled_reason(&self.controller) {
                return Action::disabled(token, label, reason);
            }
            return Action::enabled(token, label);
        }
        let name = match token.to_ascii_lowercase().as_str() {
            "arc s" | "arcdir" => "ArcDirection",
            "interpcurve" => "InterpCrv",
            "gvringrail" => "OM9-TOOLS-001",
            "gvgemoncrv" if label.eq_ignore_ascii_case("Edit Gem on Curve") => {
                gem_placement::EDIT_CURVE_ROW
            }
            _ => token,
        };
        let id = self
            .controller
            .resolve_typed_command(name)
            .or_else(|| gem::command_name(name))
            .or_else(|| gem_placement::command_name(name))
            .or_else(|| gem_cluster::command_name(name))
            .or_else(|| gem_edits::command_name(name))
            .or_else(|| gem_pave::PaveTool::command_name(name))
            .or_else(|| ring_tools::command_name(name))
            .or_else(|| {
                self.features
                    .iter()
                    .find(|f| {
                        f.command
                            .as_deref()
                            .is_some_and(|cmd| cmd.eq_ignore_ascii_case(name))
                    })
                    .map(|f| f.id.as_str())
            })
            .unwrap_or(name);
        if crate::project_manager::ProjectAction::from_command(name).is_some() {
            return Action::enabled(name, label);
        }
        if let Some(top) = om9_core::top11::Top11Action::from_feature(id) {
            let item = self
                .controller
                .top11_availability()
                .into_iter()
                .find(|item| item.action == top)
                .unwrap();
            return if item.enabled {
                Action::enabled(id, label)
            } else {
                Action::disabled(id, label, item.reason)
            };
        }
        if let Some(code) = self
            .controller
            .addon_code(name)
            .filter(|c| self.controller.is_available(c))
        {
            return Action::enabled(&code, label);
        }
        if self.controller.is_available(id) {
            return Action::enabled(id, label);
        }
        Action::disabled(
            script,
            label,
            &format!("No active OM9 handler for {script}"),
        )
    }

    pub(super) fn f6_actions(&self) -> Context {
        let group = self.f6_group();
        let scope = self.f6_scope(group);
        let doc = self.controller.document();
        let selected = self.controller.selected();
        let mut context = Context {
            kind: crate::f6::classify(doc, selected),
            title: self.f6_mode.label().into(),
            actions: vec![],
        };
        let replaced = self
            .f6_custom
            .replaced_groups
            .iter()
            .any(|(m, g)| *m == self.f6_mode && *g == scope);
        if !replaced && let Some(group) = group {
            for (&index, &category) in group.rows.iter().zip(&group.categories) {
                let (label, script) = &CATALOG.actions[index];
                let mut action = if script.is_empty() {
                    Action::disabled("", label, "")
                } else {
                    self.resolve_f6_action(script, label)
                };
                action.category = category;
                context.actions.push(action);
            }
        }
        if !replaced && self.f6_mode == Mode::General {
            // Keep the project-specific entry points already used by existing OM9 drawings.
            let extra = crate::f6::context(doc, selected, self.f6_providers);
            for a in extra.actions {
                if a.availability.enabled() && !context.actions.iter().any(|b| b.id == a.id) {
                    context.actions.push(a);
                }
            }
        }
        if !replaced && self.f6_mode == Mode::User {
            for id in &self.f6_custom.command_ids {
                context
                    .actions
                    .push(self.resolve_f6_action(id, &self.command_label(id)));
            }
        }
        for row in &self.f6_custom.entries {
            if row.global || (row.mode == self.f6_mode && row.group == scope) {
                let mut action = if row.command.trim().is_empty() {
                    Action::disabled("", &row.label, "")
                } else {
                    self.resolve_f6_action(&row.command, &row.label)
                };
                action.category = row.category;
                context.actions.push(action);
            }
        }
        context
    }

    fn f6_scope(&self, group: Option<&Group>) -> String {
        if let Some(group) = group {
            return group.scope_key();
        }
        format!(
            "{:?}/{}",
            crate::f6::classify(self.controller.document(), self.controller.selected()),
            self.controller.selected().len()
        )
    }

    fn run_f6(&mut self, action: &Action) {
        if action.cancel_first && self.controller.is_running() {
            self.controller.cancel();
        }
        self.invoke(&action.id, InvocationOrigin::F6);
    }

    fn begin_f6_editor(&mut self) {
        let group = self.f6_group();
        let scope = self.f6_scope(group);
        let mode = self.f6_mode;
        let key = (mode, scope.clone());
        let mut menu = self.f6_custom.clone();
        if !menu.replaced_groups.contains(&key) {
            let mut rows = Vec::new();
            if let Some(group) = group {
                rows.extend(
                    group
                        .rows
                        .iter()
                        .zip(&group.categories)
                        .map(|(&index, &category)| {
                            let (label, command) = CATALOG.actions[index].clone();
                            (label, command, category)
                        }),
                );
            }
            if mode == Mode::General {
                let existing: std::collections::BTreeSet<_> = rows
                    .iter()
                    .filter(|(_, script, _)| !script.is_empty())
                    .map(|(label, script, _)| self.resolve_f6_action(script, label).id)
                    .collect();
                let extra = crate::f6::context(
                    self.controller.document(),
                    self.controller.selected(),
                    self.f6_providers,
                );
                rows.extend(
                    extra
                        .actions
                        .into_iter()
                        .filter(|a| a.availability.enabled() && !existing.contains(&a.id))
                        .map(|a| (a.label, a.id, a.category)),
                );
            }
            if mode == Mode::User {
                rows.extend(
                    menu.command_ids
                        .iter()
                        .map(|id| (self.command_label(id), id.clone(), Category::Menu)),
                );
            }
            menu.entries.splice(
                0..0,
                rows.into_iter()
                    .map(|(label, command, category)| CustomEntry {
                        category,
                        mode,
                        group: scope.clone(),
                        global: false,
                        label,
                        command,
                    }),
            );
            menu.replaced_groups.push(key);
        }
        self.f6_editor = Some(Editor {
            selected: None,
            scroll_to_selected: false,
            import_path: String::new(),
            import_status: String::new(),
            mode,
            group: scope,
            menu,
        });
    }

    pub(super) fn f6_body(&mut self, ui: &mut egui::Ui) {
        if self.viewport_menu_owns_frame {
            ui.disable();
        }
        if self.f6_editor.is_some() {
            self.f6_edit_body(ui);
            return;
        }
        ui.strong("Modes");
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.;
            let cell_width = ui.available_width() / Mode::ALL.len() as f32;
            let side = cell_width.clamp(1., 20.);
            for (mode, icon) in Mode::ALL.into_iter().zip([
                "F6ModeGeneral",
                "CurveLineSingleLine",
                "BuilderGemLoader",
                "F6ModeSurface",
                "F6ModeTSplines",
                "F6ModeUser",
                "F6ModeReport",
                "F6ModeMaterials",
            ]) {
                let selected = mode == self.f6_mode;
                let (cell, _) = ui.allocate_exact_size(egui::vec2(cell_width, 28.), egui::Sense::hover());
                let button = egui::Rect::from_center_size(cell.center(), egui::Vec2::splat(side));
                let response = ui.place(button, egui::Button::selectable(selected, ""));
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::SelectableLabel,
                        true,
                        selected,
                        mode.label(),
                    )
                });
                self.icons.paint(ui, response.rect.shrink(2.), icon);
                if response.on_hover_text(mode.tooltip()).clicked() {
                    self.f6_mode = mode;
                }
            }
        });
        ui.separator();
        ui.strong(match self.f6_mode {
            Mode::Report => "Object Report",
            Mode::Materials => "Materials",
            _ => "Actions",
        });
        let height = (ui.available_height() - 32.).max(40.);
        egui::ScrollArea::vertical()
            .id_salt(("f6-actions", self.f6_mode.label()))
            .min_scrolled_height(height)
            .max_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.f6_mode == Mode::General {
                    ui.horizontal_wrapped(|ui| {
                        for kind in crate::file_lifecycle::FileKind::ALL {
                            let response = ui.small_button(kind.label());
                            self.file_f6_rects.insert(kind, response.rect);
                            if response.clicked() {
                                self.invoke(kind.feature_id(), InvocationOrigin::F6);
                            }
                        }
                    });
                }
                if self.f6_mode == Mode::Report {
                    self.f6_report(ui);
                }
                if self.f6_mode == Mode::Materials {
                    self.f6_materials(ui);
                }
                if matches!(self.f6_mode, Mode::Report | Mode::Materials) {
                    return;
                }
                ui.spacing_mut().item_spacing.y = 0.;
                for action in self.f6_actions().actions {
                    if action.id.is_empty() {
                        if action.label.is_empty() {
                            ui.separator();
                        } else {
                            ui.strong(&action.label);
                        }
                        continue;
                    }
                    let response = ui
                        .add_enabled_ui(action.availability.enabled(), |ui| {
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 20.),
                                egui::Sense::click(),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    ui.is_enabled(),
                                    &action.label,
                                )
                            });
                            let style = ui.style().interact(&response);
                            if response.hovered() || response.has_focus() {
                                ui.painter().rect_filled(rect, 2., style.weak_bg_fill);
                            }
                            let center = rect.left_center();
                            if let Some([r, g, b]) = action.category.marker() {
                                ui.painter().circle_filled(
                                    center + egui::vec2(7., 0.),
                                    3.5,
                                    egui::Color32::from_rgb(r, g, b),
                                );
                            }
                            ui.painter().text(
                                center + egui::vec2(17., 0.),
                                egui::Align2::LEFT_CENTER,
                                &action.label,
                                egui::TextStyle::Button.resolve(ui.style()),
                                style.fg_stroke.color,
                            );
                            response
                        })
                        .inner;
                    if let Some(reason) = action.availability.reason() {
                        response.on_disabled_hover_text(reason);
                    } else if response.clicked() {
                        self.run_f6(&action);
                        break;
                    }
                }
            });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if !matches!(self.f6_mode, Mode::Report | Mode::Materials)
                && ui
                    .small_button("⚙")
                    .on_hover_text("Customize F6 menu")
                    .clicked()
            {
                self.begin_f6_editor();
            }
            if ui
                .small_button("⟳")
                .on_hover_text("Refresh F6 menu")
                .clicked()
            {
                self.f6_group_cache.set(None);
                self.f6_report_cache = None;
                ui.ctx().request_repaint();
            }
            let summary = self.f6_object_summary().join(", ");
            let text = if summary.is_empty() {
                "No selection"
            } else {
                &summary
            };
            ui.add(egui::Label::new(egui::RichText::new(text).weak()).truncate())
                .on_hover_text(text);
        });
    }

    fn selection_report(&mut self) -> &SelectionReport {
        let doc = self.controller.document();
        let key = (
            doc.id,
            self.controller.coordinator().epoch(),
            doc.revision,
            self.controller.selection_revision(),
        );
        if self.f6_report_cache.as_ref().is_none_or(|r| r.key != key) {
            let mut counts = BTreeMap::new();
            let mut metals = std::collections::BTreeSet::new();
            let mut stones = BTreeMap::new();
            for id in self.controller.selected() {
                let Some(e) = doc.entities.get(id) else {
                    continue;
                };
                let kind = crate::f6::catalog::object_kind(doc, e);
                let (order, label) = match kind {
                    "Curve" => (1, "Curves"),
                    "Gem" => (2, "Gems"),
                    "PlanarClosedCurve" => (4, "Closed Planar Curves"),
                    "FourRailProfile" => (5, "4 Rail Profiles"),
                    "Surface" => (6, "Surfaces"),
                    "Polysurface" => (7, "Polysurfaces"),
                    "Mesh" => (8, "Meshes"),
                    "TSpline" => (13, "T-Spline Meshes"),
                    _ if matches!(e.geometry, om9_core::geometry::Geometry::Point(_)) => {
                        (18, "Points")
                    }
                    _ => (30, "Other Objects"),
                };
                *counts.entry((order, label)).or_insert(0usize) += 1;
                if kind == "Gem" {
                    if let Some(gem) = om9_core::gem::gem_record_in(doc, *id) {
                        let carat = gem.carat.map(f64::to_bits);
                        *stones
                            .entry((
                                gem.spec.shape.label(),
                                gem.material,
                                gem.color,
                                gem.spec.length_mm.to_bits(),
                                gem.spec.width_mm.to_bits(),
                                carat,
                            ))
                            .or_insert(0usize) += 1;
                    }
                } else if matches!(kind, "Surface" | "Polysurface" | "Mesh" | "TSpline")
                    && !e.label.eq_ignore_ascii_case("groundplane")
                    && let Ok(Some(material)) =
                        crate::session::metal::effective_render_material(doc, e, None, false)
                    && doc
                        .exchange
                        .materials
                        .get(&material)
                        .is_some_and(|m| !m.name.eq_ignore_ascii_case("defaultgroundplane"))
                {
                    metals.insert(material);
                }
            }
            let mut gems: Vec<_> = stones
                .into_iter()
                .map(|((shape, material, color, l, w, carat), n)| {
                    let mut line = format!(
                        "{shape} · {material} · {:.2} × {:.2} mm",
                        f64::from_bits(l),
                        f64::from_bits(w)
                    );
                    if let Some(ct) = carat {
                        line.push_str(&format!(" · {:.3} ct", f64::from_bits(ct)));
                    }
                    (n, line, color)
                })
                .collect();
            gems.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
            self.f6_report_cache = Some(SelectionReport {
                key,
                counts: counts
                    .into_iter()
                    .map(|((_, label), n)| format!("{n} {label}"))
                    .collect(),
                metals,
                gems,
            });
        }
        self.f6_report_cache.as_ref().unwrap()
    }

    fn f6_grip_kind(&self, grip: &om9_core::top11::GripHandle) -> (&'static str, &'static str) {
        if self.controller.has_cage_grips(grip.object) { return ("CRhNurbsCvGrip", "Points"); }
        if self
            .controller
            .document()
            .entities
            .get(&grip.object)
            .is_some_and(|e| matches!(e.geometry, om9_core::geometry::Geometry::Surface(_)))
        {
            return ("CRhSurfaceControlPointGrip", "Surface Points");
        }
        match grip.mode {
            // ShowF6Menu distinguishes the edit-point class but sets IsCurvePoint on its report row.
            om9_core::top11::GripMode::Edit => ("CRhCurveEditPointGrip", "Curve Points"),
            om9_core::top11::GripMode::Control => ("CRhCurveControlPointGrip", "Curve Points"),
        }
    }

    fn f6_object_summary(&mut self) -> Vec<String> {
        if self.controller.from_lines_faces_selected() && self.controller.tspline_component_kind().is_none() {
            let doc=self.controller.document();
            let key=(doc.id,self.controller.coordinator().epoch(),self.controller.sub_revision());
            let count=match self.from_lines_face_count {
                Some((d,e,s,n)) if (d,e,s)==key=>n,
                _=>{
                    let mut maps=BTreeMap::new();
                    let mut logical=std::collections::BTreeSet::new();
                    for pick in self.controller.subselection() {
                        if let om9_scene::scene::Subelement::MeshFace(index)=pick.subset {
                            let ids=maps.entry(pick.root).or_insert_with(||crate::session::from_lines::CageRecord::read(&doc.entities[&pick.root]).ok().map(|r|r.display_face_ids));
                            let id=ids.as_ref().and_then(|list|list.get(index as usize)).copied().unwrap_or(index as usize);
                            logical.insert((pick.root,id));
                        }
                    }
                    let n=logical.len();self.from_lines_face_count=Some((key.0,key.1,key.2,n));n
                }
            };
            return vec![format!("{count} T-Spline Faces")];
        }
        if let Some(kind) = self.controller.tspline_component_kind() {
            use om9_scene::scene::PickKind;
            let label = match kind {
                PickKind::Vertex => "T-Spline Vertices",
                PickKind::Edge => "T-Spline Edges",
                PickKind::Face => "T-Spline Faces",
                PickKind::Object => "T-Splines",
            };
            return vec![format!("{} {label}", self.controller.subselection().len())];
        }
        if self.controller.grips_selected() {
            let mut groups = BTreeMap::new();
            for grip in self.controller.selected_grips() {
                let (_, name) = self.f6_grip_kind(&grip);
                *groups.entry(name).or_insert(0usize) += 1;
            }
            return groups
                .into_iter()
                .map(|(name, n)| format!("{n} {name}"))
                .collect();
        }
        self.selection_report().counts.clone()
    }

    fn f6_report(&mut self, ui: &mut egui::Ui) {
        for line in self.f6_object_summary() {
            ui.label(line);
        }
        ui.separator();
        ui.strong("Metal Info");
        let materials = self.selection_report().metals.clone();
        let mut names = std::collections::BTreeSet::new();
        for id in materials {
            let name = self
                .controller
                .bound_metal_material(id)
                .and_then(|quality| self.controller.metal_registry().get(quality).ok())
                .map(|m| &m.name)
                .or_else(|| {
                    self.controller
                        .document()
                        .exchange
                        .materials
                        .get(&id)
                        .map(|m| &m.name)
                });
            if let Some(name) = name {
                names.insert(name);
            }
        }
        for name in names {
            ui.label(name);
        }
        ui.separator();
        ui.strong("Gem Info");
        let report = self.selection_report();
        for (n, line, rgba) in &report.gems {
            ui.horizontal_wrapped(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(10., 10.), egui::Sense::hover());
                ui.painter().circle_filled(
                    rect.center(),
                    4.,
                    egui::Color32::from_rgb(rgba[0], rgba[1], rgba[2]),
                );
                ui.label(format!("{n} × {line}"));
            });
        }
        let gems = report.gems.len();
        if gems > 0 && ui.button("Gem weight report…").clicked() {
            self.invoke(gem_edits::REPORTER, InvocationOrigin::F6);
        }
    }

    fn f6_materials(&mut self, ui: &mut egui::Ui) {
        let doc = self.controller.document();
        if doc.exchange.materials.is_empty() {
            ui.weak("No render materials in this document");
            return;
        }
        for (id, mat) in &doc.exchange.materials {
            ui.selectable_value(&mut self.f6_material, Some(*id), &mat.name);
        }
        let material = self
            .f6_material
            .filter(|id| doc.exchange.materials.contains_key(id));
        let editable = !self.controller.selected().is_empty()
            && self.controller.selected().iter().all(|id| {
                doc.entities.get(id).is_some_and(|e| {
                    doc.permissions_for_entity(e, false)
                        .is_ok_and(|p| p.editable)
                })
            });
        if ui
            .add_enabled(
                editable && material.is_some(),
                theme::primary_button("Apply material"),
            )
            .clicked()
            && let Some(id) = material
        {
            let stamps = self
                .controller
                .selected()
                .iter()
                .map(|object| doc.stamp(*object))
                .collect::<Result<Vec<_>>>();
            let result = stamps.and_then(|sources| {
                self.controller
                    .edit_object_property(sources, crate::session::PropertyEdit::Material(id))
            });
            self.feedback(result);
        }
        if let Some(id) = material {
            let choices = self
                .controller
                .metal_registry()
                .iter()
                .map(|m| (m.id.clone(), m.name.clone()))
                .collect::<Vec<_>>();
            self.metal_binding_ui(ui, id, "Metal quality", &choices);
        }
    }

    fn f6_edit_body(&mut self, ui: &mut egui::Ui) {
        let editor = self.f6_editor.as_mut().expect("editor open");
        ui.strong(format!("Customize · {}", editor.mode.label()));
        ui.collapsing("Import legacy menu…", |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut editor.import_path).hint_text("Path to .menu file"),
            );
            if ui
                .add_enabled(
                    !editor.import_path.trim().is_empty(),
                    egui::Button::new("Import into draft"),
                )
                .clicked()
            {
                let path = std::path::Path::new(editor.import_path.trim());
                let result = std::fs::read_to_string(path)
                    .map_err(|e| Error::Io(e.to_string()))
                    .and_then(|text| editor.menu.import_legacy(editor.mode, &text));
                editor.import_status = match result {
                    Ok((n, skipped)) => format!("Imported {n} groups · {skipped} unmatched"),
                    Err(error) => error.to_string(),
                };
            }
            if !editor.import_status.is_empty() {
                ui.label(&editor.import_status);
            }
        });
        let visible: Vec<_> = editor
            .menu
            .entries
            .iter()
            .enumerate()
            .filter(|(_, r)| r.global || (r.mode == editor.mode && r.group == editor.group))
            .map(|(i, _)| i)
            .collect();
        // Measure against the panel once. A scroll area's content expands to fit
        // each row, which would otherwise push later checkboxes off the right edge.
        let field_width = ((ui.available_width() - 87.) / 2.).max(28.);
        let height = (ui.available_height() - 62.).max(42.);
        egui::ScrollArea::vertical()
            .id_salt("f6-editor-rows")
            .min_scrolled_height(height)
            .max_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(3., 2.);
                for &index in &visible {
                    let row = &mut editor.menu.entries[index];
                    let row_rect = ui
                        .push_id(index, |ui| {
                            ui.horizontal(|ui| {
                                let dot = |category: Category| {
                                    let [r, g, b] = category.marker().unwrap_or([120, 120, 120]);
                                    egui::RichText::new("●").color(egui::Color32::from_rgb(r, g, b))
                                };
                                let choice = egui::ComboBox::from_id_salt("category")
                                    .width(34.)
                                    .selected_text(dot(row.category))
                                    .show_ui(ui, |ui| {
                                        for category in Category::ALL {
                                            if ui
                                                .selectable_value(
                                                    &mut row.category,
                                                    category,
                                                    category.label(),
                                                )
                                                .clicked()
                                            {
                                                editor.selected = Some(index);
                                            }
                                        }
                                    })
                                    .response
                                    .on_hover_text(row.category.label());
                                let name = ui.add_sized(
                                    [field_width, 20.],
                                    egui::TextEdit::singleline(&mut row.label)
                                        .desired_width(field_width)
                                        .hint_text("Name"),
                                );
                                let command = ui.add_sized(
                                    [field_width, 20.],
                                    egui::TextEdit::singleline(&mut row.command)
                                        .desired_width(field_width)
                                        .hint_text("Command"),
                                );
                                let global = ui
                                    .checkbox(&mut row.global, "")
                                    .on_hover_text("Global · all modes and selections");
                                if choice.clicked()
                                    || global.clicked()
                                    || name.has_focus()
                                    || command.has_focus()
                                {
                                    editor.selected = Some(index);
                                }
                            });
                        })
                        .response;
                    if editor.selected == Some(index) {
                        ui.painter().vline(
                            row_rect.rect.left(),
                            row_rect.rect.y_range(),
                            egui::Stroke::new(1., ui.visuals().selection.stroke.color),
                        );
                        if editor.scroll_to_selected {
                            row_rect.scroll_to_me(Some(egui::Align::Center));
                            editor.scroll_to_selected = false;
                        }
                    }
                }
            });
        let position = editor
            .selected
            .and_then(|index| visible.iter().position(|i| *i == index));
        ui.horizontal(|ui| {
            if ui.small_button("+").on_hover_text("Add command").clicked() {
                editor.selected = Some(editor.menu.entries.len());
                editor.scroll_to_selected = true;
                editor.menu.entries.push(CustomEntry {
                    category: Category::Menu,
                    mode: editor.mode,
                    group: editor.group.clone(),
                    global: false,
                    label: String::new(),
                    command: String::new(),
                });
            }
            if ui
                .add_enabled(position.is_some(), egui::Button::new("−").small())
                .on_hover_text("Remove selected row")
                .clicked()
                && let Some(pos) = position
            {
                editor.menu.entries.remove(visible[pos]);
                editor.selected = None;
            }
            for (glyph, next, hint) in [
                ("↑", position.and_then(|p| p.checked_sub(1)), "Move up"),
                (
                    "↓",
                    position.and_then(|p| (p + 1 < visible.len()).then_some(p + 1)),
                    "Move down",
                ),
            ] {
                if ui
                    .add_enabled(next.is_some(), egui::Button::new(glyph).small())
                    .on_hover_text(hint)
                    .clicked()
                    && let (Some(pos), Some(next)) = (position, next)
                {
                    editor.menu.entries.swap(visible[pos], visible[next]);
                    editor.selected = Some(visible[next]);
                    editor.scroll_to_selected = true;
                }
            }
        });
        let valid = editor
            .menu
            .entries
            .iter()
            .all(|r| r.command.trim().is_empty() || !r.label.trim().is_empty());
        let mut save = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            save = ui
                .add_enabled(valid, theme::primary_button("Save"))
                .clicked();
            cancel = ui.small_button("Cancel").clicked();
        });
        if save {
            self.f6_custom = self.f6_editor.take().unwrap().menu;
        } else if cancel {
            self.f6_editor = None;
        }
    }
}
