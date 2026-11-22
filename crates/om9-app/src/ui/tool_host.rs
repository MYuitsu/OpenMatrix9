// SPDX-License-Identifier: LGPL-2.1-or-later
//! Presentation state for open command panels. All parameter edits still reach their owner.
use super::*;
use crate::panel_ops::ColumnSide;
use crate::session::{gem_edits::GemEditsTool, gem_placement::GemPlacementTool};
use om9_core::builder::BuilderKind;
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct ToolHost {
    pub(super) active: BTreeSet<ToolPanel>,
    pub(super) reveal: BTreeSet<ToolPanel>,
    pub(super) drawn: BTreeSet<ToolPanel>,
}

impl MatrixApp {
    pub(super) fn sync_tools(&mut self) {
        use ToolPanel::*;
        self.sync_ring_rail_toolbar();
        let c = &self.controller;
        let builder = if self.builder_ui.rail_toolbar.is_some() {
            Some(Builder(BuilderKind::RingRail))
        } else if c.metal_panel_open() {
            Some(Metal)
        } else if let Some(b) = c.addon_builder() {
            Some(
                BuilderKind::ALL
                    .into_iter()
                    .find(|kind| kind.command() == b.code)
                    .map(Builder)
                    .unwrap_or(HostedBuilder),
            )
        } else {
            c.builder().map(|b| Builder(b.kind()))
        };
        if builder == Some(HostedBuilder)
            && let Some(b) = c.addon_builder()
        {
            self.dock
                .select_hosted_panel(HostedBuilder, &format!("{}/{}", b.owner, b.code));
        }
        if let Some(s) = c.addon_session()
            && c.addon_dialog().is_some()
        {
            self.dock
                .select_hosted_panel(SessionOptions, &format!("options/{}/{}", s.owner, s.code));
        }
        let active: BTreeSet<_> = [
            builder,
            c.gem_session()
                .filter(|_| c.gem_editor().is_none())
                .map(|s| Gem(s.tool)),
            c.gem_editor().map(|_| Properties),
            c.gem_placement_session()
                .filter(|s| {
                    matches!(
                        s.tool,
                        GemPlacementTool::OnCurve | GemPlacementTool::OnSurface
                    )
                })
                .map(|s| Placement(s.tool)),
            c.gem_cluster_session().map(|s| Cluster(s.tool)),
            c.gem_pave_session().map(|s| Pave(s.tool)),
            c.gem_cutter_session().map(|s| Cutter(s.tool)),
            c.row_cutter_session().map(|s| Row(s.tool)),
            c.gem_edits_session().and_then(|s| match s.tool {
                GemEditsTool::Map => Some(GemMap),
                GemEditsTool::Reporter => Some(GemReport),
                _ => None,
            }),
            self.history_window.then_some(History),
            self.builder_style_window.then_some(Styles),
            self.builder_profile_window.then_some(Profiles),
            self.health_panel_open.then_some(Health),
            self.addons_ui.open.then_some(Addons),
            self.gumball_settings_open.then_some(Gumball),
            self.project_manager_open.then_some(ProjectManager),
            (self.project_manager_open && self.project_draft.is_some()).then_some(ProjectDetails),
            (self.builder_ui.rail_library_open && c.ring_rail_is_open()).then_some(RailShapes),
            c.addon_dialog()
                .filter(|d| self.session_dialog.hidden.as_deref() != Some(&d.code))
                .map(|_| SessionOptions),
            self.stl_profile_draft.map(|_| StlOptions),
            self.stl_job.as_ref().map(|_| StlProgress),
            self.f6_open.then_some(F6),
            self.command_history_open.then_some(CommandHistory),
        ]
        .into_iter()
        .flatten()
        .collect();
        self.tool_host.reveal = active.difference(&self.tool_host.active).copied().collect();
        for tool in &active {
            self.dock.register_tool(*tool);
        }
        self.tool_host.active = active;
        self.tool_host.drawn.clear();
    }

    pub(super) fn tool_visible(&self, id: SectionId) -> bool {
        match id {
            SectionId::Tool(tool) => self.tool_host.active.contains(&tool),
            _ => true,
        }
    }

    pub(super) fn remember_tool(&mut self, tool: ToolPanel) {
        if self.named_layouts {
            let _ = self.save_dock_layout();
            return;
        }
        let Some(path) = self.dock_path.as_deref() else {
            return;
        };
        let (mut saved, issues) = DockLayout::load(path, &Limits::default());
        if !issues.is_empty() {
            self.status.warning(&format!(
                "Tool placement could not be saved: {}",
                issues.join(", ")
            ));
            return;
        }
        saved.remember_tool(&self.dock, tool);
        if let Err(error) = saved.save(path) {
            self.status
                .warning(&format!("Tool placement could not be saved: {error}"));
        }
    }

    fn close_tool(&mut self, tool: ToolPanel) {
        use ToolPanel::*;
        match tool {
            Gem(_) => {
                self.controller.gem_cancel();
            }
            Placement(_) => {
                self.controller.gem_placement_cancel();
            }
            Cluster(_) => {
                self.controller.gem_cluster_cancel();
            }
            Pave(_) => {
                self.controller.gem_pave_cancel();
            }
            Cutter(_) => {
                self.controller.gem_cutter_cancel();
            }
            Row(_) => {
                self.controller.row_cutter_cancel();
            }
            Properties => {
                self.controller.gem_properties_close();
            }
            GemMap | GemReport => {
                self.controller.gem_edits_cancel();
            }
            Metal => {
                self.controller.close_metal_panel();
            }
            Builder(BuilderKind::RingRail) if self.builder_ui.rail_toolbar.is_some() => {
                self.builder_ui.rail_toolbar = None;
                if self.controller.ring_rail_is_open() {
                    self.controller.cancel();
                }
            }
            Builder(_) | HostedBuilder => self.controller.cancel(),
            Health => self.health_panel_open = false,
            History => self.history_window = false,
            Styles => self.builder_style_window = false,
            Profiles => self.builder_profile_window = false,
            Addons => self.addons_ui.open = false,
            Gumball => self.gumball_settings_open = false,
            StlOptions => self.cancel_stl_export_options(),
            StlProgress => {
                if let Some(job) = &self.stl_job {
                    job.cancel();
                }
            }
            RailShapes => {
                self.builder_ui.rail_library_open = false;
                self.builder_ui.rail_edit_curve = None;
                self.builder_ui.rail_error = None;
            }
            ProjectDetails => self.project_draft = None,
            SessionOptions => {
                if self.controller.addon_session().is_some_and(|s|s.code=="tsSplitCurves") {self.controller.cancel();}
                else {self.session_dialog.hidden = self.controller.addon_dialog().map(|d| d.code);}
            }
            F6 => {
                self.f6_open = false;
                self.f6_editor = None;
                if self.f6_pinned {
                    self.f6_pinned = false;
                    self.dock.f6_pinned = false;
                    self.dock_dirty = true;
                    self.remember_tool(F6);
                }
            }
            CommandHistory => self.command_history_open = false,
            ProjectManager => {
                self.project_manager_open = false;
                self.project_draft = None;
            }
        }
        self.transcript.set_prompt(&self.live_command_prompt());
        self.focus_cmd = true;
    }

    fn tool_body(&mut self, ui: &mut egui::Ui, tool: ToolPanel) {
        use ToolPanel::*;
        match tool {
            Gem(_) | Properties => self.gem_panel_ui(ui),
            Placement(_) => self.gem_placement_panel_ui(ui),
            Cluster(_) => self.gem_cluster_panel_ui(ui),
            Pave(_) => self.gem_pave_panel_ui(ui),
            Cutter(_) => self.gem_cutters_panel_ui(ui),
            Row(_) => self.row_cutters_panel_ui(ui),
            GemMap | GemReport => self.gem_edits_panel_ui(ui),
            Builder(_) | HostedBuilder | Metal => self.builder_slot_ui(ui),
            Health => self.health_panel_ui(ui),
            History => self.history_panel_ui(ui),
            Styles => self.builder_style_panel_ui(ui),
            Profiles => self.builder_profile_panel_ui(ui),
            Addons => self.addons_panel_ui(ui),
            StlOptions => self.stl_profile_ui(ui),
            StlProgress => self.stl_progress_ui(ui),
            ProjectDetails => self.project_details_ui(ui),
            RailShapes => self.ring_rail_library_body(ui),
            SessionOptions => self.session_options_ui(ui),
            F6 => self.f6_body(ui),
            CommandHistory => self.command_history_body(ui),
            ProjectManager => self.project_metadata_ui(ui),
            Gumball => self.gumball_settings_ui(ui),
        }
    }

    pub(super) fn tool_panel_ui(&mut self, ui: &mut egui::Ui, tool: ToolPanel, side: DockSide) {
        if !self.tool_host.active.contains(&tool) || !self.tool_host.drawn.insert(tool) {
            return;
        }
        let readable = matches!(
            tool,
            ToolPanel::Gem(_)
                | ToolPanel::Placement(_)
                | ToolPanel::Cluster(_)
                | ToolPanel::Pave(_)
                | ToolPanel::Cutter(_)
                | ToolPanel::Row(_)
                | ToolPanel::Properties
                | ToolPanel::GemMap
                | ToolPanel::GemReport
                | ToolPanel::F6
        );
        let id = SectionId::Tool(tool);
        let state = self.dock.section(id);
        let title = if tool == ToolPanel::HostedBuilder {
            self.controller
                .addon_builder()
                .map_or(tool.title(), |b| b.label.as_str())
        } else {
            tool.title()
        }
        .to_owned();
        let title = if tool == ToolPanel::SessionOptions {
            self.controller
                .addon_dialog()
                .map(|d| d.title)
                .unwrap_or(title)
        } else {
            title
        };
        let mut destination = None;
        let mut close = false;
        let header = egui::Frame::new()
            .fill(theme::HEADER)
            .stroke(if readable {
                Stroke::new(1., theme::EDGE)
            } else {
                Stroke::NONE
            })
            .inner_margin(2)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let collapse = ui.small_button(if state.collapsed { "+" } else { "−" });
                    if collapse.clicked() {
                        self.toggle_menu_section(id);
                    }
                    let reserve = if tool == ToolPanel::F6 { 104. } else if readable { 62. } else { 52. };
                    let width = (ui.available_width() - reserve).max(24.);
                    let mut label = egui::RichText::new(&title).color(theme::TITLE).strong();
                    let height = if readable {
                        label = label.size(theme::TOOL_TITLE_SIZE);
                        26.
                    } else {
                        20.
                    };
                    let bar = ui.add_sized(
                        [width, height],
                        egui::Button::new(label)
                            .truncate()
                            .sense(Sense::click_and_drag()),
                    ).on_hover_text(&title);
                    self.section_rects.insert(id.key(), bar.rect);
                    if let Some(column) = ColumnSide::from_dock_side(side) {
                        self.panel_bars.push((id, column, bar.rect));
                    }
                    if bar.drag_started() {
                        self.begin_section_drag(id);
                    }
                    if bar.clicked() {
                        self.toggle_menu_section(id);
                    }
                    if tool == ToolPanel::F6 {
                        if ui.toggle_value(&mut self.f6_pinned, "Pin").on_hover_text("Keep F6 open after choosing an action").changed() {
                            self.dock.f6_pinned = self.f6_pinned;
                            self.dock_dirty = true;
                            self.remember_tool(ToolPanel::F6);
                        }
                    }
                    ui.menu_button("≡", |ui| {
                        for (label, target) in [
                            ("Dock left", DockSide::Left),
                            ("Dock right", DockSide::Right),
                            ("Float", DockSide::Floating),
                        ] {
                            if ui.button(label).clicked() {
                                destination = Some(target);
                                ui.close();
                            }
                        }
                    })
                    .response
                    .on_hover_text("Panel position");
                    close = ui.small_button("×").on_hover_text("Close panel").clicked();
                });
            });
        if side == DockSide::Right && self.tool_host.reveal.contains(&tool) {
            ui.scroll_to_rect(header.response.rect, Some(egui::Align::Min));
        }
        if close {
            self.close_tool(tool);
            return;
        }
        if let Some(target) = destination {
            if target == DockSide::Floating {
                let rect = state.float.unwrap_or_else(|| {
                    let mut rect = crate::panel_ops::float_drop_rect([
                        ui.ctx().content_rect().center().x,
                        header.response.rect.top(),
                    ]);
                    if let Some([width, height]) = tool.preferred_float_size() {
                        rect.width = width;
                        rect.height = height;
                    }
                    rect
                });
                self.float_menu_section(id, rect);
            } else {
                self.dock_menu_section(id, target);
            }
        }
        if !self.dock.section(id).collapsed {
            egui::Frame::new()
                .fill(theme::BODY)
                .inner_margin(if readable { 7 } else { 3 })
                .show(ui, |ui| {
                    if readable {
                        theme::readable_tool_style(ui);
                    }
                    ui.scope_builder(
                        egui::UiBuilder::new().id(egui::Id::new(("om9-tool-body", tool.key()))),
                        |ui| {
                            let height = if side == DockSide::Floating {
                                ui.available_height() - 12.
                            } else {
                                state.height
                            };
                            let width = ui.available_width();
                            if tool == ToolPanel::F6 && self.f6_editor.is_none() {
                                ui.set_max_size(Vec2::new(width, height.max(48.)));
                                self.f6_body(ui);
                                return;
                            }
                            egui::ScrollArea::both()
                                .id_salt("fields")
                                .max_height(height.max(48.))
                                .min_scrolled_height(height.max(48.))
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.set_max_size(Vec2::new(width, height.max(48.)));
                                    self.tool_body(ui, tool);
                                });
                        },
                    );
                });
            if side != DockSide::Floating {
                let (_, grip) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 5.), Sense::drag());
                let grip = grip.on_hover_cursor(egui::CursorIcon::ResizeVertical);
                ui.painter().hline(
                    grip.rect.x_range(),
                    grip.rect.center().y,
                    Stroke::new(1., theme::EDGE),
                );
                if grip.dragged() {
                    let _ = self.apply_panel_op(crate::panel_ops::PanelOp::SetHeight {
                        section: id,
                        height: state.height + grip.drag_delta().y,
                    });
                }
                if grip.drag_stopped() {
                    self.remember_tool(tool);
                }
            }
        }
    }

    pub(super) fn floating_tools(&mut self, ctx: &egui::Context) {
        let floating = self
            .dock
            .floating_sections()
            .into_iter()
            .filter(|id| !matches!(id, SectionId::Tool(_)))
            .chain(self.tool_host.active.iter().copied().map(SectionId::Tool))
            .collect::<Vec<_>>();
        for id in floating {
            let state = self.dock.section(id);
            if !state.visible && !matches!(id, SectionId::Tool(_)) {
                continue;
            }
            let yielded = matches!(id, SectionId::Tool(_))
                && (!state.visible
                    || (state.dock == DockSide::Left && self.regions.left_rail == 0.)
                    || (state.dock == DockSide::Right && !self.regions.right_dock_visible));
            if !(yielded || state.dock == DockSide::Floating) {
                continue;
            }
            let [width, height] = match id {
                SectionId::Tool(tool) => tool.preferred_float_size().unwrap_or([350., 405.]),
                _ => [350., 405.],
            };
            let stored = state.float.unwrap_or(FloatRect {
                x: 320.,
                y: 100.,
                width,
                height,
            });
            let rect = self.work_area.map_or(stored, |work| work.clamp(stored));
            let bounds = ctx.content_rect();
            egui::Area::new(egui::Id::new(("om9-floating-tool", id.key(), yielded)))
                .default_size(Vec2::new(rect.width, rect.height))
                .order(egui::Order::Middle)
                .movable(false)
                .constrain_to(bounds)
                .fixed_pos(egui::pos2(rect.x, rect.y))
                .show(ctx, |ui| {
                    let frame = egui::Frame::window(ui.style());
                    let inside = Vec2::new(rect.width, rect.height) - frame.total_margin().sum();
                    frame.show(ui, |ui| {
                        ui.set_width(inside.x.max(1.));
                        if !state.collapsed {
                            ui.set_height(inside.y.max(48.));
                        }
                        if let SectionId::Tool(tool) = id {
                            self.tool_panel_ui(ui, tool, DockSide::Floating);
                        } else if self.section(ui, id) {
                            egui::ScrollArea::both().show(ui, |ui| self.sidebar_section_ui(ui, id));
                        }
                        if !yielded && !state.collapsed {
                            let corner = ui.max_rect().right_bottom();
                            let grip = ui
                                .interact(
                                    Rect::from_min_size(
                                        corner - Vec2::splat(13.),
                                        Vec2::splat(13.),
                                    ),
                                    ui.id().with("resize"),
                                    Sense::drag(),
                                )
                                .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
                            ui.painter().line_segment(
                                [corner - egui::vec2(9., 1.), corner - egui::vec2(1., 9.)],
                                Stroke::new(1.5, theme::MUTED),
                            );
                            if grip.dragged() {
                                let delta = grip.drag_delta();
                                self.readback_float(
                                    id,
                                    FloatRect {
                                        width: (rect.width + delta.x)
                                            .max(crate::panel_ops::FLOAT_MIN_WIDTH),
                                        height: (rect.height + delta.y)
                                            .max(crate::panel_ops::FLOAT_MIN_HEIGHT),
                                        ..rect
                                    },
                                );
                            }
                            if grip.drag_stopped() {
                                if let SectionId::Tool(tool) = id {
                                    self.remember_tool(tool);
                                } else if self.named_layouts {
                                    let _ = self.save_dock_layout();
                                }
                            }
                        }
                    });
                });
        }
    }
}
