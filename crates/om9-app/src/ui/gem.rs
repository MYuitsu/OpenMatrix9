// SPDX-License-Identifier: LGPL-2.1-or-later
//! Floating windows of the gem tools (`OM9-L2-04-09`).
//!
//! The window is a view of the session state only: every control edits the preview, and the
//! single Apply button commits one transaction through `Controller::gem_apply`. Cancel drops
//! the session, so no phantom document object ever exists.
use eframe::egui;

use super::MatrixApp;
use crate::session::{
    gem::{GemPropertiesEditor, GemSession, GemTool, LOADER_GALLERY_COLUMNS, gem_library, material_color},
    gem_placement::{GemBuilderPick, GemPlacementSession, GemPlacementTool},
};
use om9_core::{
    Error, Result,
    gem::{GemAsset, GemCut, GemShape, GemSize, GemSpec, build_mesh, girdle_outline, table_outline},
    gem_edits::{DIAMOND_DENSITY_G_CM3, gem_weight, material_density},
    gem_placement::RowFit,
    identity::ObjectId,
    model::Document,
};
use om9_core::geometry::{Mesh, Transform};
use om9_scene::camera::{Camera, Viewport};

mod placement_art;
use placement_art::Illustration;

/// Project one gem mesh as deduplicated triangle/quad edges. A preview is a wireframe overlay on
/// the committed scene: it shows the exact body that Apply will insert without touching the
/// document or the scene cache.
pub(crate) fn gem_preview_segments(
    mesh: &Mesh,
    placement: &Transform,
    camera: &Camera,
    viewport: Viewport,
) -> Vec<[egui::Pos2; 2]> {
    let project = |point: [f64; 3]| -> Option<egui::Pos2> {
        let world = placement.point(point).ok()?;
        let projected = camera.project(world, viewport).ok()??;
        Some(egui::pos2(
            projected.screen[0] as f32,
            projected.screen[1] as f32,
        ))
    };
    let mut edges = std::collections::BTreeSet::new();
    for face in &mesh.faces {
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            if a != b && !om9_core::gem::gem_tessellation_edge(mesh, a, b) {
                edges.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
    }
    let mut segments = Vec::with_capacity(edges.len());
    for (a, b) in edges {
        let (Some(a), Some(b)) = (
            mesh.vertices.get(a as usize).copied().and_then(project),
            mesh.vertices.get(b as usize).copied().and_then(project),
        ) else {
            continue;
        };
        segments.push([a, b]);
    }
    segments
}

impl MatrixApp {
    /// Paint the open gem tool's preview body in the active viewport.
    pub(super) fn gem_preview_ui(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
    ) {
        let Some(preview) = self.controller.gem_world_preview() else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        let Some(active) = self.controller.gem_session() else {return};
        let key = egui::Id::new((self.controller.document().id, self.controller.coordinator().epoch(),
            active.shape.tag(), active.cut.tag(), &active.material, active.dimensions.map(f64::to_bits),
            active.layer, active.target, active.replace));
        let color = ui.ctx().data_mut(|data| {
            let cache = egui::Id::new("gem preview layer color");
            if let Some((previous,color)) = data.get_temp::<(egui::Id,egui::Color32)>(cache)
                && previous == key {return color;}
            let [r,g,b,_] = self.controller.gem_preview_color(active);
            let color = egui::Color32::from_rgb(r,g,b);
            data.insert_temp(cache,(key,color));
            color
        });
        if view.mode == om9_scene::workspace::DisplayMode::Wireframe
            && active.shape == GemShape::Round && active.cut == GemCut::Diamond
            && preview.mesh.authored_faces().is_some() {
            let mesh = &preview.mesh.mesh;
            let triangles: Vec<_> = mesh.faces.iter().enumerate().flat_map(|(i, face)|
                face[1..].windows(2).map(move |pair| ([face[0],pair[0],pair[1]], i as u32))
            ).collect();
            let guides = om9_core::gem::round_facet_isocurves(&mesh.vertices,&triangles);
            let draw = |placement: Transform| {
                let project = |p| {
                    let screen = view.camera.project(placement.point(p).ok()?, viewport).ok()??.screen;
                    Some(egui::pos2(screen[0] as f32,screen[1] as f32))
                };
                for &[a,b] in &guides {
                    if let (Some(a),Some(b)) = (project(a),project(b)) { painter.line_segment([a,b],egui::Stroke::new(1.,color)); }
                }
            };
            if active.points.is_empty() { draw(preview.placement); }
            else { for &point in active.points.iter().take(128) {draw(Transform::translation(point));} }
        }
        if let Some(session) = self.controller.gem_session() {
            for point in session.points.iter().take(128) {
                for [a, b] in gem_preview_segments(
                    &preview.mesh.mesh, &Transform::translation(*point), &view.camera, viewport,
                ) {
                    painter.line_segment([a, b], egui::Stroke::new(1.2, color));
                }
            }
            if !session.points.is_empty() {
                return;
            }
        }
        let mut painted = false;
        for [a, b] in gem_preview_segments(
            &preview.mesh.mesh,
            &preview.placement,
            &view.camera,
            viewport,
        ) {
            painter.line_segment([a, b], egui::Stroke::new(1.2, color));
            painted = true;
        }
        // An empty projection (or a sliver seen edge-on) still shows where the gem will land.
        if let Ok(world) = preview.placement.point([0.0, 0.0, 0.0])
            && let Ok(Some(point)) = view.camera.project(world, viewport)
        {
            let center = egui::pos2(point.screen[0] as f32, point.screen[1] as f32);
            if !painted {
                painter.circle_stroke(center, 5.0, egui::Stroke::new(1.2, color));
            }
            painter.circle_filled(center, 1.8, color);
        }
    }

    pub(super) fn gem_viewport_click(
        &mut self, ui: &egui::Ui, view: &om9_scene::workspace::ViewState,
        viewport: Viewport, pos: egui::Pos2,
    ) -> Option<Result<()>> {
        let screen = [pos.x.into(), pos.y.into()];
        if let Some(pick) = self.controller.gem_builder_pick() {
            return Some((|| {
                use om9_scene::scene::{PickIntent, PickKind};
                let scene = self.controller.scene().ok_or(Error::Missing("viewport scene"))?;
                let kind = if pick == GemBuilderPick::Surface { PickKind::Face } else { PickKind::Object };
                let hits = scene.pick_kind_all(
                    screen, &view.camera, viewport, super::pick_tolerance(ui), PickIntent::Select, kind,
                )?;
                let document = self.controller.document();
                let hit = hits.iter().find(|hit| hit.path.is_empty() && document.entities.get(&hit.root)
                    .is_some_and(|entity| pick.accepts(document, entity)))
                    .ok_or(Error::Missing(pick.prompt()))?;
                hit.revalidate_in_scene(self.controller.coordinator(), Some(scene), &view.camera, viewport)?;
                self.controller.gem_placement_pick_target(hit.root)
            })());
        }
        if self.controller.gem_surface_manual() {
            let snap = self.gem_surface_snap_point(view, viewport, pos);
            return Some(view.camera.visible_ray(screen, viewport)
                .and_then(|ray| self.controller.gem_surface_click(ray, snap)));
        }
        if self.controller.gem_loader_accepts_point() {
            return Some(view.camera.ray(screen, viewport).and_then(|ray| {
                let point = ray.intersect_plane(view.cplane)?.ok_or(Error::Missing("point on CPlane"))?;
                self.controller.gem_loader_point(point)
            }));
        }
        None
    }

    pub(super) fn gem_surface_snap_point(&self, view: &om9_scene::workspace::ViewState,
        viewport: Viewport, pos: egui::Pos2) -> Option<[f64; 3]> {
        let session = self.controller.gem_placement_session()?;
        if !session.surface_snap { return None; }
        let scene = self.controller.scene()?;
        let mut settings = self.snap_state.settings;
        settings.one_shot = None;
        let hits = om9_scene::osnap::candidates_in_plane(scene, &view.camera, viewport,
            [pos.x.into(), pos.y.into()], settings, None, view.cplane).ok()?;
        hits.into_iter().find(|candidate| matches!(candidate.kind,
            om9_scene::osnap::SnapKind::End | om9_scene::osnap::SnapKind::Mid |
            om9_scene::osnap::SnapKind::Point | om9_scene::osnap::SnapKind::Center))
            .and_then(|candidate| {
                let world = candidate.world;
                self.validate_snap_anchor(&Some(candidate)).ok().map(|()| world)
            })
    }

    pub(super) fn gem_placement_preview_ui(
        &self, ui: &egui::Ui, view: &om9_scene::workspace::ViewState, viewport: Viewport, rect: egui::Rect,
    ) {
        let Some(session) = self.controller.gem_placement_session() else { return; };
        let painter = ui.painter().with_clip_rect(rect);
        let color = egui::Color32::from_rgb(110, 235, 170);
        let project = |point| view.camera.project(point, viewport).ok().flatten()
            .map(|projected| egui::pos2(projected.screen[0] as f32, projected.screen[1] as f32));
        for mesh in session.curve_prongs.iter().take(128) {
            for [a, b] in gem_preview_segments(mesh, &Transform::identity(), &view.camera, viewport) {
                painter.line_segment([a, b], egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE));
            }
        }
        let spacing_ring = |transform: &Transform, radius: f64| {
            let ring: Vec<_> = (0..=24).filter_map(|side| {
                let (sin, cos) = (std::f64::consts::TAU * side as f64 / 24.0).sin_cos();
                transform.point([radius * cos, radius * sin, 0.0]).ok().and_then(project)
            }).collect();
            if ring.len() == 25 {
                painter.add(egui::Shape::line(ring, egui::Stroke::new(1.0, color.gamma_multiply(0.6))));
            }
        };
        for (transform, size) in session.surface_guides.iter().take(128) {
            spacing_ring(transform, (size + session.surface_spacing) * 0.5);
        }
        let mut cached_mesh = None;
        for (index, station) in session.preview_stations().enumerate() {
            if index < 128 {
                if cached_mesh.as_ref().is_none_or(|(spec, _)| *spec != &station.spec) {
                    let mesh = if session.previews_spheres() {
                        om9_core::settings::sphere([0.; 3], station.spec.length_mm, 24).ok()
                    } else { build_mesh(&station.spec).ok().map(|gem| gem.mesh) };
                    cached_mesh = Some((&station.spec, mesh));
                }
                if let Some((_, Some(mesh))) = &cached_mesh {
                    for [a, b] in gem_preview_segments(mesh, &station.transform, &view.camera, viewport) {
                        painter.line_segment([a, b], egui::Stroke::new(1.0, color));
                    }
                }
            } else if let Some(point) = project(station.point) {
                painter.circle_filled(point, 2.0, color);
            }
            if index < 128 && session.surface_spacing > 0.0 && session.tool == crate::session::gem_placement::GemPlacementTool::OnSurface {
                let radius = (station.spec.length_mm + session.surface_spacing) * 0.5;
                spacing_ring(&station.transform, radius);
            }
        }
    }

    pub(super) fn gem_curve_handles_ui(
        &mut self, ui: &mut egui::Ui, view_id: om9_scene::workspace::ViewId,
        view: &om9_scene::workspace::ViewState, viewport: Viewport, rect: egui::Rect,
    ) {
        use crate::session::gem_placement::GemPlacementTool;
        let Some(session) = self.controller.gem_placement_session() else { return; };
        if session.tool != GemPlacementTool::OnCurve { return; }
        let Some(plan) = session.plans.first() else { return; };
        let Some((_, curve)) = session.curves.first() else { return; };
        use om9_core::curve_ops::{length_between, parameter_at_length, point_at};
        let limits = self.controller.coordinator().limits();
        let project = |point| view.camera.project(point, viewport).ok().flatten()
            .map(|p| egui::pos2(p.screen[0] as f32, p.screen[1] as f32));
        let mut option = None;
        for (index, name, color) in [
            (0, "Start", egui::Color32::from_rgb(184, 230, 255)),
            (1, "End", egui::Color32::from_rgb(194, 237, 199)),
        ] {
            let Some(handle) = parameter_at_length(curve, plan.range[index], limits).ok()
                .and_then(|parameter| point_at(curve, parameter, limits).ok()).and_then(project)
                .filter(|position| rect.contains(*position))
            else { continue; };
            let response = ui.interact(egui::Rect::from_center_size(handle, egui::Vec2::splat(14.0)),
                ui.id().with(("gem-row-limit", view_id as u8, index)), egui::Sense::drag());
            ui.painter().with_clip_rect(rect).circle_filled(handle, 4.5, color);
            ui.painter().with_clip_rect(rect).circle_stroke(handle, 6.0,
                egui::Stroke::new(1.0, egui::Color32::BLACK));
            if response.dragged() && let Some(pointer) = response.interact_pointer_pos() {
                let screen_error = |fraction| point_at(curve, fraction, limits).ok()
                    .and_then(project).map_or(f32::INFINITY, |p| p.distance_sq(pointer));
                let Some(sample) = (0..=64).map(|step| step as f64 / 64.0)
                    .min_by(|&a, &b| screen_error(a).total_cmp(&screen_error(b)))
                else { continue; };
                let mut lo = (sample - 1.0 / 64.0).max(0.0);
                let mut hi = (sample + 1.0 / 64.0).min(1.0);
                for _ in 0..14 {
                    let left = (2.0 * lo + hi) / 3.0;
                    let right = (lo + 2.0 * hi) / 3.0;
                    if screen_error(left) < screen_error(right) { hi = right; } else { lo = left; }
                }
                if let Ok(arc) = length_between(curve, 0.0, (lo + hi) * 0.5, limits) {
                    let percent = (100.0 * arc / plan.total_length_mm).clamp(0.0, 100.0);
                    option = Some(format!("{name}={percent}"));
                }
            }
        }
        if let Some(option) = option {
            let result = self.controller.submit(&option);
            self.feedback(result);
            ui.ctx().request_repaint();
        }
    }

    pub(super) fn gem_placement_panel_ui(&mut self, ui: &mut egui::Ui) {
        let Some(session) = self.controller.gem_placement_session() else { return; };
        if !matches!(session.tool, GemPlacementTool::OnCurve | GemPlacementTool::OnSurface) {
            return;
        }
        let document = self.controller.document();
        let mut action = None;

        ui.scope(|ui| {
                placement_source_picker(ui, document, session, &mut action);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("placement-parameters")
                    .max_height((ui.available_height() - 58.0).max(56.0))
                    .show(ui, |ui| {
                        if session.tool == GemPlacementTool::OnCurve {
                            curve_builder_ui(ui, document, session, &mut action);
                        } else {
                            surface_builder_ui(ui, document, session, &mut action);
                        }
                    });
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    let count = session.preview_count();
                    if ui.add_enabled(count > 0 && session.builder_pick.is_none(), super::theme::primary_button("Apply")).clicked() {
                        action = Some(GemBuilderAction::Option("Apply".to_owned()));
                    }
                    if ui.button("Cancel").clicked() {
                        action = Some(GemBuilderAction::Option("Cancel".to_owned()));
                    }
                    ui.colored_label(super::theme::ACCENT, format!("{count} in preview"));
                });
            });
        if let Some(action) = action {
            let result = match action {
                GemBuilderAction::Option(command) => self.controller.submit(&command),
                GemBuilderAction::Pick(pick) => self.controller.request_gem_builder_pick(pick),
                GemBuilderAction::CancelPick => {
                    self.controller.cancel_gem_builder_pick();
                    Ok(())
                }
            };
            self.refresh_command_prompt();
            self.feedback(result);
        }
    }

    fn refresh_command_prompt(&mut self) {
        self.transcript.set_prompt(&self.live_command_prompt());
    }

    pub(super) fn gem_panel_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(editor) = self.controller.gem_editor().cloned() {
            self.gem_editor_window(ui, editor);
            return;
        }
        let Some(session) = self.controller.gem_session().cloned() else {
            return;
        };
        let mut session = session;

        let mut apply = false;
        let mut cancel = false;
        let mut on_surface = false;
        ui.scope(|ui| {
                if session.tool == GemTool::Loader {
                    ui.spacing_mut().item_spacing.y = 2.0;
                }
                match session.tool {
                    GemTool::Loader => {
                        egui::ScrollArea::vertical()
                            .id_salt("loader-parameters")
                            .max_height((ui.available_height() - 64.0).max(56.0))
                            .show(ui, |ui| loader_ui(ui, &mut session));
                    }
                    crate::session::gem::GemTool::Custom => custom_ui(ui, &mut session),
                    crate::session::gem::GemTool::Emerald
                    | crate::session::gem::GemTool::Baguette => emerald_ui(ui, &mut session),
                    crate::session::gem::GemTool::Positioner => positioner_ui(ui, &mut session),
                }
                ui.separator();
                ui.colored_label(super::theme::ACCENT, session.summary());
                if session.replace.is_some() {
                    ui.label("Replace selected gem");
                }
                ui.horizontal_wrapped(|ui| {
                    let apply_button = ui.add_enabled(session.spec().is_ok(), super::theme::primary_button("Apply"));
                    let apply_button = if session.tool == GemTool::Loader && session.replace.is_none() {
                        apply_button.on_hover_text(if session.points.is_empty() {
                            "Pick points in the viewport, or Apply to place at F4."
                        } else {
                            "Apply to place the previewed gems."
                        })
                    } else {
                        apply_button
                    };
                    if apply_button.clicked() {
                        apply = true;
                    }
                    if !session.points.is_empty() && ui.button("Undo Point").clicked() {
                        session.points.pop();
                    }
                    if session.tool == crate::session::gem::GemTool::Loader
                        && ui.button("On Surface").clicked()
                    {
                        on_surface = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if on_surface {
            if let Some(slot) = self.controller.gem_session_mut() {
                *slot = session;
            }
            let result = self.controller.submit("OnSurface");
            self.feedback(result);
            return;
        }
        if cancel {
            self.controller.gem_cancel();
            self.refresh_command_prompt();
            self.status.info("Gem command canceled");
            return;
        }
        if let Some(slot) = self.controller.gem_session_mut() {
            *slot = session;
        }
        if apply {
            match self.controller.gem_apply() {
                Ok(()) => {
                    self.refresh_command_prompt();
                    self.status.info("Gem committed");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
        }
    }

    fn gem_editor_window(&mut self, ui: &mut egui::Ui, mut editor: GemPropertiesEditor) {

        let mut apply = false;
        let mut close = false;
        ui.scope(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut editor.label);
                });
                ui.horizontal(|ui| {
                    ui.label("Material");
                    ui.text_edit_singleline(&mut editor.material);
                });
                let mut color = egui::Color32::from_rgba_unmultiplied(
                    editor.color[0],
                    editor.color[1],
                    editor.color[2],
                    editor.color[3],
                );
                ui.horizontal(|ui| {
                    ui.label("Color");
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        editor.color = [color.r(), color.g(), color.b(), color.a()];
                    }
                    if ui.button("Material color").clicked()
                        && let Some(default) = material_color(&editor.material)
                    {
                        editor.color = default;
                    }
                });
                ui.separator();
                if editor.shared_definition() {
                    ui.label("This gem is shared by several instances; only name, material, colour and layer can change.");
                }
                ui.add_enabled_ui(editor.editable_geometry(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Length");
                        ui.add(
                            egui::DragValue::new(&mut editor.length_mm)
                                .speed(0.05)
                                .suffix(" mm"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Width");
                        ui.add(
                            egui::DragValue::new(&mut editor.width_mm)
                                .speed(0.05)
                                .suffix(" mm"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Depth");
                        ui.add(
                            egui::DragValue::new(&mut editor.depth_mm)
                                .speed(0.05)
                                .suffix(" mm"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Girdle");
                        ui.add(
                            egui::DragValue::new(&mut editor.girdle_mm)
                                .speed(0.01)
                                .suffix(" mm"),
                        );
                    });
                });
                ui.separator();
                ui.label(editor.summary());
                if let Some(status) = &editor.status {
                    ui.label(status);
                }
                ui.horizontal(|ui| {
                    if ui.add(super::theme::primary_button("Apply")).clicked() {
                        apply = true;
                    }
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            });
        if close {
            self.controller.gem_properties_close();
            return;
        }
        if let Some(slot) = self.controller.gem_editor_mut() {
            *slot = editor;
        }
        if apply {
            match self.controller.gem_properties_apply() {
                Ok(()) => {
                    self.status.info("Gem properties updated");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
        }
    }
}

enum GemBuilderAction {
    Option(String),
    Pick(GemBuilderPick),
    CancelPick,
}

fn object_label(document: &Document, id: Option<ObjectId>) -> &str {
    id.and_then(|id| document.entities.get(&id)).map_or("None", |entity| entity.label.as_str())
}

fn placement_source_picker(
    ui: &mut egui::Ui,
    document: &Document,
    session: &GemPlacementSession,
    action: &mut Option<GemBuilderAction>,
) {
    let (label, art, source, pick, editable) = if session.tool == GemPlacementTool::OnCurve {
        ("Curve", Illustration::Curve, session.curves.first().map(|(id, _)| *id),
            GemBuilderPick::RowCurve, session.edit_row.is_none())
    } else {
        ("Surface", Illustration::Surface, session.surface.as_ref().map(|(id, _)| *id),
            GemBuilderPick::Surface, session.surface_stations.is_empty())
    };
    ui.horizontal(|ui| {
        let button = ui.add_enabled_ui(editable, |ui| {
            placement_art::tile(ui, art, label, 76.0, session.builder_pick == Some(pick))
        }).inner.on_hover_text(pick.prompt());
        if button.clicked() {
            *action = Some(GemBuilderAction::Pick(pick));
        }
        ui.vertical(|ui| {
            if source.is_some() {
                ui.label(object_label(document, source));
            }
            if let Some(active_pick) = session.builder_pick {
                ui.label(active_pick.prompt());
                if ui.small_button("Back").clicked() {
                    *action = Some(GemBuilderAction::CancelPick);
                }
            } else if source.is_none() {
                ui.label(pick.prompt());
            }
        });
    });
}

fn picker_row(
    ui: &mut egui::Ui,
    name: &str,
    value: &str,
    pick: GemBuilderPick,
    editable: bool,
    action: &mut Option<GemBuilderAction>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(name).strong().color(super::theme::TITLE));
        ui.label(value);
        if ui.add_enabled(editable, egui::Button::new("Pick…")).on_hover_text(pick.prompt()).clicked() {
            *action = Some(GemBuilderAction::Pick(pick));
        }
    });
}

fn builder_option(action: &mut Option<GemBuilderAction>, option: &str, value: impl std::fmt::Display) {
    *action = Some(GemBuilderAction::Option(format!("{option}={value}")));
}

fn builder_number(
    ui: &mut egui::Ui,
    label: &str,
    option: &str,
    mut value: f64,
    range: std::ops::RangeInclusive<f64>,
    suffix: &str,
    action: &mut Option<GemBuilderAction>,
) {
    ui.push_id(option, |ui| {
        if ui.add(egui::Slider::new(&mut value, range)
            .text(label).suffix(suffix)
            .clamping(egui::SliderClamping::Never)).changed()
        {
            builder_option(action, option, value);
        }
    });
}

struct GemBuilderField {
    art: Illustration,
    label: &'static str,
    option: &'static str,
    value: f64,
    range: std::ops::RangeInclusive<f64>,
    suffix: &'static str,
}

fn field(
    art: Illustration, label: &'static str, option: &'static str, value: f64,
    range: std::ops::RangeInclusive<f64>, suffix: &'static str,
) -> GemBuilderField {
    GemBuilderField { art, label, option, value, range, suffix }
}

fn builder_field_selector(
    ui: &mut egui::Ui,
    fields: &[GemBuilderField],
    columns: usize,
    action: &mut Option<GemBuilderAction>,
) {
    let id = ui.id().with("gem-active-field");
    let option = ui.data(|data| data.get_temp::<&'static str>(id));
    let mut active = fields.iter().find(|field| option == Some(field.option)).unwrap_or(&fields[0]);
    let spacing = ui.spacing().item_spacing.x;
    let available = ui.available_width();
    let fit = (((available + spacing) / (58.0 + spacing)) as usize).max(1);
    let columns = if fit >= columns { columns } else { fit.min(2) };
    let tile_width = ((available - spacing * (columns - 1) as f32) / columns as f32).floor();
    for row in fields.chunks(columns) {
        ui.horizontal(|ui| {
            for field in row {
                let label = if field.option == "VerticalOffset" { "Offset" } else { field.label };
                let clicked = ui.push_id(field.option, |ui| {
                    placement_art::tile(ui, field.art, label, tile_width, active.option == field.option)
                        .on_hover_text(field.label)
                        .clicked()
                }).inner;
                if clicked {
                    active = field;
                }
            }
        });
    }
    ui.data_mut(|data| data.insert_temp(id, active.option));
    builder_number(ui, active.label, active.option, active.value, active.range.clone(), active.suffix, action);
}

fn builder_flag(ui: &mut egui::Ui, label: &str, option: &str, mut value: bool,
    action: &mut Option<GemBuilderAction>) {
    if ui.checkbox(&mut value, label).changed() {
        builder_option(action, option, value);
    }
}

const MATERIALS: &[(&str, &str)] = &[
    ("diamond", "Diamond"), ("ruby", "Ruby"), ("sapphire", "Sapphire"),
    ("emerald", "Emerald"), ("amethyst", "Amethyst"), ("topaz", "Topaz"), ("clear", "Clear"),
];

fn material_picker(ui: &mut egui::Ui, material: &str) -> Option<&'static str> {
    let current = MATERIALS.iter().find(|(tag, _)| *tag == material).map_or(material, |&(_, label)| label);
    let mut choice = None;
    ui.horizontal(|ui| {
        ui.label("Material");
        egui::ComboBox::from_id_salt("gem-material").selected_text(current).show_ui(ui, |ui| {
            for &(tag, label) in MATERIALS {
                let [r, g, b, _] = material_color(tag).expect("known material");
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(r, g, b), "●");
                    if ui.selectable_label(tag == material, label).clicked() { choice = Some(tag); }
                });
            }
        });
    });
    choice
}

fn curve_builder_ui(
    ui: &mut egui::Ui,
    document: &Document,
    session: &GemPlacementSession,
    action: &mut Option<GemBuilderAction>,
) {
    let row = &session.curve_row;
    egui::CollapsingHeader::new("Adjust").default_open(true).show(ui, |ui| {
        let mut fields = vec![
            field(Illustration::Start, "Start", "Start", row.start, 0.0..=100.0, " %"),
            field(Illustration::End, "End", "End", row.end, 0.0..=100.0, " %"),
            field(Illustration::Gap, "Spacing", "Spacing", row.gap, 0.0..=5.0, " mm"),
            field(Illustration::Offset, "Z Offset", "ZOffset", session.params.z_offset_mm, -5.0..=5.0, " mm"),
            field(Illustration::Roll, "Roll", "Roll", session.params.rotation_deg, -180.0..=180.0, "°"),
            field(Illustration::Size, "Start Size", "StartSize", session.spec.length_mm, 0.3..=15.0, " mm"),
        ];
        if let Some(spec) = &session.end_spec {
            fields.push(field(Illustration::Size, "End Size", "EndSize", spec.length_mm, 0.3..=15.0, " mm"));
        }
        builder_field_selector(ui, &fields, 3, action);
    });
    egui::CollapsingHeader::new("Layout").default_open(true).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (mode, command, label) in [
                (RowFit::Spacing, "Spacing", "Spacing"),
                (RowFit::SizeGems, "SizeGems", "Size Gems"),
                (RowFit::Fixed, "Fixed", "Fixed"),
            ] {
                let active = row.fit == mode && row.count.is_none() && row.center_step.is_none();
                if ui.selectable_label(active, label).clicked() {
                    builder_option(action, "SpacingMethod", command);
                }
            }
        });
        if let Some(count) = row.count {
            ui.horizontal(|ui| {
                ui.label(format!("Count override: {count}"));
                if ui.small_button("Auto count").clicked() {
                    builder_option(action, "Count", "Auto");
                }
            });
        }
        if let Some(step) = row.center_step {
            ui.label(format!("Centre step: {step:.2} mm"));
        }
        ui.horizontal_wrapped(|ui| {
            builder_flag(ui, "Mirror", "Mirror", row.mirror, action);
            builder_flag(ui, "Snap to Corners", "SnapToCorners", row.snap_corners, action);
            builder_flag(ui, "Show Prongs", "ShowProngs", row.show_prongs, action);
        });
    });
    egui::CollapsingHeader::new("Gem and taper").default_open(true).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Round Gem").clicked() { builder_option(action, "ObjectType", "RoundGem"); }
            if ui.button("Select Gem…").clicked() { *action = Some(GemBuilderAction::Pick(GemBuilderPick::Gem)); }
            if ui.selectable_label(row.sphere, "Sphere").clicked() { builder_option(action, "ObjectType", "Sphere"); }
            for label in ["Object", "Browser"] {
                ui.add_enabled(false, egui::Button::new(label))
                    .on_disabled_hover_text("This object type is not available in OM9 yet.");
            }
        });
        ui.label(if row.sphere { "Sphere".to_owned() } else {
            format!("{} · {}", session.spec.shape.label(), session.spec.cut.label())
        });
        builder_flag(ui, "Taper Gems", "TaperGems", session.end_spec.is_some(), action);
        builder_flag(ui, "Taper Spacing", "TaperSpacing", row.taper_gap, action);
        if let Some(material) = material_picker(ui, &session.material) {
            builder_option(action, "Material", material);
        }
    });
    egui::CollapsingHeader::new("Orientation").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Culet");
            if ui.selectable_label(!row.toward_origin && row.guide.is_none(), "Down").clicked() {
                builder_option(action, "CuletDir", "Down");
            }
            if ui.selectable_label(row.toward_origin, "F4").clicked() {
                builder_option(action, "CuletDir", "F4");
            }
            if ui.selectable_label(row.guide.is_some(), "Object…").clicked() {
                *action = Some(GemBuilderAction::Pick(GemBuilderPick::CuletGuide));
            }
        });
        if row.guide.is_some() { ui.label(object_label(document, row.guide)); }
        ui.horizontal_wrapped(|ui| {
            builder_flag(ui, "Flip", "Flip", row.flipped, action);
            builder_flag(ui, "Gem Tops", "GemTops", row.tops, action);
        });
    });
    if !session.builder_styles.is_empty() {
        egui::ComboBox::from_id_salt("gem-curve-styles").selected_text("Gem Styles").show_ui(ui, |ui| {
            for name in &session.builder_styles {
                if ui.selectable_label(false, name).clicked() {
                    builder_option(action, "Style", name);
                }
            }
        });
    }
}

fn surface_builder_ui(
    ui: &mut egui::Ui,
    document: &Document,
    session: &GemPlacementSession,
    action: &mut Option<GemBuilderAction>,
) {
    egui::CollapsingHeader::new("Placement").default_open(true).show(ui, |ui| {
        builder_field_selector(ui, &[
            field(Illustration::Angle, "Angle", "Angle", session.surface_params.rotation_deg, -180.0..=180.0, "°"),
            field(Illustration::Offset, "Vertical Offset", "VerticalOffset", session.surface_params.z_offset_mm, -5.0..=5.0, " mm"),
            field(Illustration::Scale, "Scale", "Scale", session.spec.length_mm, 0.3..=15.0, " mm"),
            field(Illustration::SurfaceSpacing, "Spacing", "Spacing", session.surface_spacing, 0.0..=3.0, " mm"),
        ], 4, action);
        ui.label("A / D · Angle    W / S · Offset    Q / E · Scale");
        ui.horizontal_wrapped(|ui| {
            builder_flag(ui, "Copy", "Copy", session.surface_copy, action);
            builder_flag(ui, "Snap", "Snap", session.surface_snap, action);
            builder_flag(ui, "Flip", "Flip", session.surface_flip, action);
        });
        ui.horizontal_wrapped(|ui| {
            let manual = session.surface.is_some() && session.grid == 0 && session.builder_pick.is_none();
            if ui.add_enabled(manual, egui::Button::new("Start")).clicked() {
                *action = Some(GemBuilderAction::Option("Start".to_owned()));
            }
            if ui.add_enabled(manual, egui::Button::new("Reset")).clicked() {
                *action = Some(GemBuilderAction::Option("Reset".to_owned()));
            }
            if ui.add_enabled(manual && !session.surface_stations.is_empty(), egui::Button::new("Undo Point")).clicked() {
                *action = Some(GemBuilderAction::Option("Undo".to_owned()));
            }
            if session.grid > 0 && ui.button("Manual placement").clicked() {
                builder_option(action, "Grid", 0);
            }
        });
    });
    egui::CollapsingHeader::new("Gem").show(ui, |ui| {
        egui::ComboBox::from_id_salt("gem-surface-cut").selected_text(session.spec.cut.label()).show_ui(ui, |ui| {
            for cut in GemCut::ALL {
                if ui.selectable_label(cut == session.spec.cut, cut.label()).clicked() {
                    builder_option(action, "Cut", cut.tag());
                }
            }
        });
        egui::ComboBox::from_id_salt("gem-surface-shape").selected_text(session.spec.shape.label()).show_ui(ui, |ui| {
            for &shape in session.spec.cut.shapes() {
                if ui.selectable_label(shape == session.spec.shape, shape.label()).clicked() {
                    builder_option(action, "Shape", shape.tag());
                }
            }
        });
        if !session.spec.shape.is_round() {
            builder_number(ui, "Width", "Width", session.spec.width_mm, 0.3..=session.spec.length_mm.max(0.3), " mm", action);
        }
        if let Some(material) = material_picker(ui, &session.material) {
            builder_option(action, "Material", material);
        }
        if ui.add_enabled(session.surface.is_some(), egui::Button::new("Select existing gem…")).clicked() {
            *action = Some(GemBuilderAction::Pick(GemBuilderPick::Gem));
        }
    });
    egui::CollapsingHeader::new("Guides").show(ui, |ui| {
        picker_row(
            ui, "Guide curve",
            object_label(document, session.curves.first().map(|(id, _)| *id)),
            GemBuilderPick::SurfaceGuide, session.surface.is_some(), action,
        );
        if !session.curves.is_empty() {
            builder_flag(ui, "Constrain to curve", "Curve", session.surface_curve, action);
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Reference gems: {}", session.surface_guides.len()));
            if ui.add_enabled(session.surface.is_some(), egui::Button::new("Add gem…")).clicked() {
                *action = Some(GemBuilderAction::Pick(GemBuilderPick::ReferenceGem));
            }
            if !session.surface_guides.is_empty() && ui.small_button("Clear guides").clicked() {
                builder_option(action, "Gems", false);
            }
        });
    });
}

fn loader_ui(ui: &mut egui::Ui, session: &mut GemSession) {
    if !session.custom_styles.is_empty() {
        let mut choice = None;
        egui::ComboBox::from_id_salt("loader-custom-styles")
            .selected_text("Custom")
            .show_ui(ui, |ui| {
                for style in &session.custom_styles {
                    if ui.selectable_label(false, &style.name).clicked() {
                        choice = Some(style.name.clone());
                    }
                }
            });
        if let Some(name) = choice && let Err(error) = session.load_custom_style(&name) {
            ui.label(error.to_string());
        }
    }
    let cut_index = GemCut::ALL.iter().position(|cut| *cut == session.cut).unwrap_or(0);
    let mut cut = session.cut;
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Cut type").strong().color(super::theme::TITLE));
        if ui.button("<").on_hover_text("Previous cut type").clicked() {
            cut = GemCut::ALL[(cut_index + GemCut::ALL.len() - 1) % GemCut::ALL.len()];
        }
        egui::ComboBox::from_id_salt("om9-gem-cut")
            .selected_text(session.cut.label())
            .show_ui(ui, |ui| {
                for family in GemCut::ALL {
                    ui.selectable_value(&mut cut, family, family.label());
                }
            });
        if ui.button(">").on_hover_text("Next cut type").clicked() {
            cut = GemCut::ALL[(cut_index + 1) % GemCut::ALL.len()];
        }
    });
    if cut != session.cut {
        let shape = if cut.loader_shapes().contains(&session.shape) { session.shape } else { cut.loader_shapes()[0] };
        session.select_loader_asset(cut, shape);
        session.loader_size_picker = false;
    }
    ui.add_enabled(
        session.replace.is_some(),
        egui::Checkbox::new(&mut session.keep_original, "Keep Original Size"),
    );
    let library = gem_library();
    let asset = library.asset(session.shape, session.cut);
    let pages = asset.map_or(1, GemAsset::size_page_count);
    let columns = LOADER_GALLERY_COLUMNS;
    let gap = 4.0;
    let tile_width = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32)
        .floor()
        .clamp(32.0, 58.0);
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut session.loader_size_picker, false, "Gem Shape");
        if asset.is_some() {
            ui.selectable_value(&mut session.loader_size_picker, true, "Size (mm)");
        }
        if session.loader_size_picker {
            ui.strong(session.shape.label());
            if pages > 1 {
                for page in 1..=pages {
                    ui.selectable_value(&mut session.size_page, page, page.to_string());
                }
            }
        }
    });
    if session.loader_size_picker {
        if let Some(asset) = asset {
            loader_size_dial(ui, session, asset);
        }
    } else {
            egui::Grid::new("om9-gem-shapes").num_columns(columns).spacing([gap, gap]).show(ui, |ui| {
            for (index, &shape) in session.cut.loader_shapes().iter().enumerate() {
                if let Some(asset) = library.asset(shape, session.cut)
                    && let Some(size) = asset.sizes.get(asset.sizes.len() / 2)
                    && loader_gem_tile(
                        ui, asset, size, shape.label(), shape == session.shape,
                        tile_width, (26.0 / size.length_mm.max(size.width_mm)) as f32,
                    ).on_hover_text(&asset.name).clicked()
                {
                    session.select_loader_asset(session.cut, shape);
                    session.size_page = 1;
                    session.loader_size_picker = true;
                }
                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
        });
    }
    ui.separator();
    ui.add_enabled_ui(!session.keep_original || session.replace.is_none(), |ui| {
        let columns = if ui.available_width() >= 264.0 { 4 } else { 2 };
        let field_gap = 6.0;
        let field_width = ((ui.available_width() - field_gap * (columns - 1) as f32) / columns as f32)
            .clamp(60.0, 100.0);
        egui::Grid::new("om9-loader-dimensions").num_columns(columns).spacing([field_gap, 4.0]).show(ui, |ui| {
            let round = session.shape.is_round();
            let [length, width, depth] = session.dimensions;
            let axis = if session.loader_corner_to_corner && asset.is_some_and(GemAsset::has_corner_to_corner_sizes) {
                0
            } else {
                1
            };
            let span = session.dimensions[axis];
            let percent = if span > 0.0 { 100.0 * depth / span } else { 0.0 };
            let mut dimensions = [length, width, depth, percent];
            for (index, label) in ["Length", "Width", "Depth", "Depth %"].into_iter().enumerate() {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(label).strong());
                    let value = egui::DragValue::new(&mut dimensions[index]);
                    let value = if index == 3 {
                        value.range(1.0..=1000.0).suffix(" %")
                    } else {
                        value.speed(0.05).suffix(" mm")
                    };
                    let changed = ui.add_sized([field_width, 23.0], value).changed();
                    if changed {
                        if round && index < 2 {
                            dimensions[1 - index] = dimensions[index];
                        }
                        if index == axis || (round && index == 0) || index == 3 {
                            dimensions[2] = dimensions[axis] * dimensions[3] / 100.0;
                        } else if index == 2 && dimensions[axis] > 0.0 {
                            dimensions[3] = 100.0 * dimensions[2] / dimensions[axis];
                        }
                    }
                });
                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
            let [length, width, depth, _] = dimensions;
            session.dimensions = [length, width, depth];
        });
    });
    egui::CollapsingHeader::new("Material and body").show(ui, |ui| {
        if let Some(material) = material_picker(ui, &session.material) {
            session.material = material.to_owned();
        }
        ui.horizontal(|ui| {
            ui.label("Body");
            if session.allows_brep() {
                ui.selectable_value(&mut session.brep, true, "B-rep");
                ui.selectable_value(&mut session.brep, false, "Mesh");
            } else {
                ui.label("Mesh");
            }
        });
        ui.add_enabled_ui(!session.keep_original || session.replace.is_none(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Girdle");
                ui.add(egui::DragValue::new(&mut session.girdle_mm).speed(0.01).suffix(" mm"));
                if session.shape.default_corner_ratio() > 0.0 {
                    ui.label("Corner");
                    ui.add(egui::DragValue::new(&mut session.corner_mm).speed(0.02).suffix(" mm"));
                }
            });
        });
    });
}

fn loader_size_dial(ui: &mut egui::Ui, session: &mut GemSession, asset: &GemAsset) {
    use super::theme::{ACCENT, EDGE, INK, SELECTED, TITLE, TOOL_FIELD};
    const MAX_DIAMETER: f32 = 224.0;

    let range = asset.size_page_range(session.size_page);
    let start = range.start;
    let corner_to_corner = asset.has_corner_to_corner_sizes() && session.loader_corner_to_corner;
    let sizes = &asset.sizes_for_axis(corner_to_corner)[range];
    let longest = sizes.iter()
        .map(|size| size.length_mm.max(size.width_mm))
        .fold(1.0_f64, f64::max);
    let width = ui.available_width();
    let diameter = width.min(MAX_DIAMETER);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, diameter), egui::Sense::hover());
    let center = rect.center();
    let radius = diameter * 0.365;
    let target_width = (radius * std::f32::consts::TAU / sizes.len() as f32 * 0.76)
        .min(diameter * 0.15);
    let face_radius = diameter * 0.265;
    let painter = ui.painter().with_clip_rect(rect);
    painter.circle_stroke(center, radius, egui::Stroke::new(1.0, EDGE));
    painter.circle(center, face_radius, TOOL_FIELD, egui::Stroke::new(1.5, ACCENT));
    let toggle_axis = asset.has_corner_to_corner_sizes().then(|| {
        let response = ui.interact(
            egui::Rect::from_center_size(center, egui::Vec2::splat(face_radius * 1.4)),
            ui.make_persistent_id("loader-dial-axis"),
            egui::Sense::click(),
        );
        response.widget_info(|| egui::WidgetInfo::selected(
            egui::WidgetType::Button, ui.is_enabled(), corner_to_corner, "Corner-to-corner sizes",
        ));
        response.on_hover_text(if corner_to_corner {
            "Switch to length and width"
        } else {
            "Switch to corner-to-corner sizes"
        })
        .clicked()
    }).unwrap_or(false);

    let mut hovered_size = None;
    for (offset, size) in sizes.iter().enumerate() {
        let index = start + offset;
        let angle = std::f32::consts::TAU * offset as f32 / sizes.len() as f32 - 1.8;
        let direction = egui::vec2(angle.sin(), -angle.cos());
        let point = center + direction * radius;
        let hit = egui::Rect::from_center_size(point, egui::Vec2::splat(target_width));
        let id = ui.make_persistent_id(("loader-dial-preset", &asset.key, index));
        let response = ui.interact(hit, id, egui::Sense::click());
        let selected = session.dimensions == [size.length_mm, size.width_mm, size.depth_mm];
        let label = size.label();
        response.widget_info(|| egui::WidgetInfo::selected(
            egui::WidgetType::Button, ui.is_enabled(), selected, &label,
        ));
        let visuals = ui.style().interact_selectable(&response, selected);
        if selected || response.hovered() || response.has_focus() {
            let fill = if selected { SELECTED } else { visuals.bg_fill };
            painter.rect(hit, visuals.corner_radius, fill, egui::Stroke::new(1.5, ACCENT),
                egui::StrokeKind::Inside);
        } else {
            painter.rect_filled(hit, visuals.corner_radius, ui.visuals().window_fill);
        }
        let span = size.length_mm.max(size.width_mm);
        let extent = target_width * (0.60 + 0.33 * (span / longest) as f32);
        paint_loader_outline(&painter, &loader_preset_outline(ui, asset, size), point, extent / span as f32, selected);
        let caption = format!("{:.2}", size.length_mm);
        let caption = if caption.ends_with('0') { &caption[..caption.len() - 1] } else { &caption };
        painter.text(
            center + direction * (diameter * 0.467), egui::Align2::CENTER_CENTER,
            caption, egui::FontId::proportional(11.0),
            if selected { TITLE } else { INK },
        );
        if response.hovered() {
            hovered_size = Some(size);
        }
        if response.on_hover_text(format!("{} · {label}", asset.name)).clicked() {
            session.select_loader_size(asset, index);
        }
    }

    let hover_spec = hovered_size.filter(|_| !session.keep_original).and_then(|size| asset.spec_for(size).ok());
    if let Some(spec) = hover_spec.or_else(|| session.spec().ok()) {
        let span = spec.length_mm.max(spec.width_mm) as f32;
        let glyph_center = center - egui::vec2(0.0, face_radius * 0.32);
        paint_loader_outline(&painter, &loader_outline(&spec), glyph_center, face_radius * 0.8 / span, true);
        let dimensions = if corner_to_corner {
            format!("{:.2} mm C–C", spec.length_mm)
        } else if spec.shape.is_round() {
            format!("Ø {:.2} mm", spec.length_mm)
        } else {
            format!("{:.2} × {:.2} mm", spec.length_mm, spec.width_mm)
        };
        painter.text(center + egui::vec2(0.0, face_radius * 0.23), egui::Align2::CENTER_CENTER,
            dimensions, egui::FontId::proportional((diameter / 20.0).clamp(9.0, 13.0)), INK);
        let preview_size = GemSize { length_mm: spec.length_mm, width_mm: spec.width_mm, depth_mm: spec.depth_mm() };
        let carat = asset.factory_carat(preview_size, corner_to_corner).or_else(|| {
            let density = material_density(&session.material).unwrap_or(DIAMOND_DENSITY_G_CM3);
            gem_weight(&spec, density).ok().map(|weight| weight.carat)
        });
        if let Some(carat) = carat {
            let digits = if carat < 0.1 { 3 } else { 2 };
            painter.text(center + egui::vec2(0.0, face_radius * 0.56), egui::Align2::CENTER_CENTER,
                format!("≈ {carat:.digits$} ct"), egui::FontId::proportional(12.0), ACCENT);
        }
    }
    if toggle_axis {
        session.toggle_loader_size_axis(asset);
        ui.ctx().request_repaint();
    }
}

fn loader_gem_tile(
    ui: &mut egui::Ui,
    asset: &GemAsset,
    size: &GemSize,
    label: &str,
    selected: bool,
    tile_width: f32,
    scale: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(tile_width, 42.0), egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(
        egui::WidgetType::Button, ui.is_enabled(), selected, label,
    ));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let visuals = ui.style().interact_selectable(&response, selected);
    let painter = ui.painter().with_clip_rect(rect);
    let (fill, border) = if selected {
        (super::theme::SELECTED, egui::Stroke::new(1.5, super::theme::ACCENT))
    } else {
        (visuals.bg_fill, visuals.bg_stroke)
    };
    painter.rect(rect, visuals.corner_radius, fill, border, egui::StrokeKind::Inside);
    let center = egui::pos2(rect.center().x, rect.top() + 15.0);
    paint_loader_outline(&painter, &loader_preset_outline(ui, asset, size), center, scale, selected);
    let color = super::theme::INK;
    let font = egui::FontId::proportional(11.0);
    let caption = painter.layout_no_wrap(label.to_owned(), font.clone(), color);
    let width = rect.width() - 4.0;
    let caption = if caption.size().x > width {
        painter.layout_no_wrap(label.to_owned(), egui::FontId::proportional(font.size * width / caption.size().x), color)
    } else {
        caption
    };
    painter.galley(
        egui::pos2(rect.center().x - caption.size().x * 0.5, rect.bottom() - caption.size().y - 2.0),
        caption,
        color,
    );
    response
}

type LoaderOutline = (Vec<egui::Vec2>, Vec<egui::Vec2>);

fn loader_preset_outline(ui: &egui::Ui, asset: &GemAsset, size: &GemSize) -> LoaderOutline {
    let thumbnail_id = egui::Id::new((
        "loader-gem-outline",
        &asset.key,
        [size.length_mm, size.width_mm, size.depth_mm].map(f64::to_bits),
    ));
    ui.data_mut(|data| {
        data.get_temp::<LoaderOutline>(thumbnail_id).unwrap_or_else(|| {
            let outlines = asset.spec_for(size).map_or_else(|_| LoaderOutline::default(), |spec| loader_outline(&spec));
            data.insert_temp(thumbnail_id, outlines.clone());
            outlines
        })
    })
}

fn loader_outline(spec: &GemSpec) -> LoaderOutline {
    let project = |[x, y, _]: [f64; 3]| {
        let (horizontal, vertical) = match spec.shape {
            GemShape::Heart => (x, -y),
            GemShape::Trillion | GemShape::TrillionStraight | GemShape::Triangle => (x, y),
            _ => (y, -x),
        };
        egui::vec2(horizontal as f32, vertical as f32)
    };
    let girdle = girdle_outline(spec).unwrap_or_default().into_iter().map(project).collect();
    let table = if matches!(spec.cut, GemCut::Cabochon | GemCut::Pearl) {
        Vec::new()
    } else {
        table_outline(spec).unwrap_or_default().into_iter().map(project).collect()
    };
    (girdle, table)
}

fn paint_loader_outline(
    painter: &egui::Painter,
    (girdle, table): &LoaderOutline,
    center: egui::Pos2,
    scale: f32,
    selected: bool,
) {
    let to_screen = |point: egui::Vec2| center + point * scale;
    let color = super::theme::INK;
    let line = egui::Stroke::new(1.4, if selected { super::theme::ACCENT } else { color });
    if !table.is_empty() {
        let facet = egui::Stroke::new(0.75, color.gamma_multiply(0.65));
        for (&a, &b) in girdle.iter().zip(table) {
            painter.line_segment([to_screen(a), to_screen(b)], facet);
        }
        painter.add(egui::Shape::closed_line(table.iter().copied().map(to_screen).collect(), line));
    }
    if !girdle.is_empty() {
        painter.add(egui::Shape::closed_line(girdle.iter().copied().map(to_screen).collect(), line));
    }
}

fn custom_ui(ui: &mut egui::Ui, session: &mut GemSession) {
    ui.label(format!(
        "Profile: {} points, closed planar curve",
        session.profile.as_ref().map_or(0, Vec::len)
    ));
    ui.horizontal(|ui| {
        ui.label("Facets");
        ui.add(egui::DragValue::new(&mut session.custom.facet_count).range(3..=128));
    });
    ui.horizontal(|ui| {
        ui.label("Crown angle");
        ui.add(
            egui::DragValue::new(&mut session.custom.crown_angle_deg)
                .speed(0.1)
                .suffix("°"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Crown height");
        ui.add(
            egui::DragValue::new(&mut session.custom.crown_height_mm)
                .speed(0.02)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Girdle");
        ui.add(
            egui::DragValue::new(&mut session.girdle_mm)
                .speed(0.01)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Pavilion depth");
        ui.add(
            egui::DragValue::new(&mut session.custom.pavilion_depth_mm)
                .speed(0.02)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Pavilion rows");
        ui.add(egui::DragValue::new(&mut session.custom.pavilion_rows).range(0..=6));
    });
    ui.horizontal(|ui| {
        ui.label("Bulge");
        ui.add(
            egui::DragValue::new(&mut session.custom.bulge)
                .speed(0.01)
                .range(0.0..=1.0),
        );
    });
    session.custom.girdle_mm = session.girdle_mm;
    session.dimensions[2] =
        session.custom.crown_height_mm + session.girdle_mm + session.custom.pavilion_depth_mm;
}

fn emerald_ui(ui: &mut egui::Ui, session: &mut GemSession) {
    ui.horizontal(|ui| {
        ui.label("Length");
        ui.add(
            egui::DragValue::new(&mut session.dimensions[0])
                .speed(0.05)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Width");
        ui.add(
            egui::DragValue::new(&mut session.dimensions[1])
                .speed(0.05)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Corner");
        ui.add(
            egui::DragValue::new(&mut session.corner_mm)
                .speed(0.02)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Depth");
        ui.add(
            egui::DragValue::new(&mut session.dimensions[2])
                .speed(0.05)
                .suffix(" mm"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Girdle");
        ui.add(
            egui::DragValue::new(&mut session.girdle_mm)
                .speed(0.01)
                .suffix(" mm"),
        );
    });
}

fn positioner_ui(ui: &mut egui::Ui, session: &mut GemSession) {
    ui.label("Move (mm)");
    ui.horizontal(|ui| {
        for (axis, label) in ["X", "Y", "Z"].iter().enumerate() {
            ui.label(*label);
            ui.add(egui::DragValue::new(&mut session.moved[axis]).speed(0.05));
        }
    });
    ui.label("Rotate (degrees)");
    ui.horizontal(|ui| {
        for (axis, label) in ["X", "Y", "Z"].iter().enumerate() {
            ui.label(*label);
            ui.add(egui::DragValue::new(&mut session.rotation_deg[axis]).speed(0.5));
        }
    });
    ui.horizontal(|ui| {
        ui.label("Scale");
        ui.add(
            egui::DragValue::new(&mut session.scale)
                .speed(0.01)
                .range(0.01..=100.0),
        );
    });
    ui.checkbox(&mut session.duplicate, "Duplicate as instance");
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use om9_core::gem::{GemSpec, build_mesh};

    fn camera() -> Camera {
        Camera::orthographic([0.0, 0.0, 100.0], [0.0; 3], [0.0, 1.0, 0.0], 20.0).expect("camera")
    }

    fn viewport() -> Viewport {
        Viewport {
            origin: [0.0, 0.0],
            size: [800.0, 600.0],
        }
    }

    #[test]
    fn the_preview_wireframe_covers_the_gem_and_stays_on_screen() {
        let gem = build_mesh(&GemSpec::round(6.0, 4.0).expect("round")).expect("mesh");
        let segments =
            gem_preview_segments(&gem.mesh, &Transform::identity(), &camera(), viewport());
        assert!(!segments.is_empty(), "a closed gem projects visible edges");
        assert!(
            segments.len() >= gem.mesh.faces.len(),
            "one edge per face at minimum"
        );
        for [a, b] in &segments {
            for point in [a, b] {
                assert!(point.x.is_finite() && point.y.is_finite());
                assert!(
                    point.x > -20.0 && point.x < 820.0 && point.y > -20.0 && point.y < 620.0,
                    "projected point stays inside the viewport: {point:?}"
                );
            }
        }
    }

    #[test]
    fn the_preview_follows_its_placement() {
        let gem = build_mesh(&GemSpec::round(6.0, 4.0).expect("round")).expect("mesh");
        let at_origin =
            gem_preview_segments(&gem.mesh, &Transform::identity(), &camera(), viewport());
        let moved = Transform::translation([2.0, 0.0, 0.0]);
        let shifted = gem_preview_segments(&gem.mesh, &moved, &camera(), viewport());
        assert_eq!(shifted.len(), at_origin.len());
        let (a, b) = (at_origin[0][0].x, shifted[0][0].x);
        assert!(
            (b - a - 60.0).abs() < 1.0,
            "2 mm at 30 px/mm shifts the preview by 60 px, got {}",
            b - a
        );
    }
}
