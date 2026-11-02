// SPDX-License-Identifier: LGPL-2.1-or-later
//! Floating windows of the gem tools (`OM9-L2-04-09`).
//!
//! The window is a view of the session state only: every control edits the preview, and the
//! single Apply button commits one transaction through `Controller::gem_apply`. Cancel drops
//! the session, so no phantom document object ever exists.
use eframe::egui;

use super::MatrixApp;
use crate::session::gem::{GemPropertiesEditor, GemSession, material_color};
use om9_core::gem::{GemCut, GemLibrary};
use om9_core::geometry::{Mesh, Transform};
use om9_scene::camera::{Camera, Viewport};

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
            if a != b {
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
        let color = egui::Color32::from_rgb(110, 235, 170);
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

    pub(super) fn gem_window_ui(&mut self, ctx: &egui::Context) {
        if let Some(editor) = self.controller.gem_editor().cloned() {
            self.gem_editor_window(ctx, editor);
            return;
        }
        let Some(session) = self.controller.gem_session().cloned() else {
            return;
        };
        let mut session = session;
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        egui::Window::new(session.tool.title())
            .id(egui::Id::new(("om9-gem-tool", session.tool.command())))
            .open(&mut open)
            .default_width(360.0)
            .resizable(false)
            .show(ctx, |ui| {
                match session.tool {
                    crate::session::gem::GemTool::Loader => loader_ui(ui, &mut session),
                    crate::session::gem::GemTool::Custom => custom_ui(ui, &mut session),
                    crate::session::gem::GemTool::Emerald
                    | crate::session::gem::GemTool::Baguette => emerald_ui(ui, &mut session),
                    crate::session::gem::GemTool::Positioner => positioner_ui(ui, &mut session),
                }
                ui.separator();
                ui.label(session.summary());
                if session.replace.is_some() {
                    ui.label("Apply replaces the selected gem (same GemId).");
                }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        apply = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if !open && !apply {
            cancel = true;
        }
        if cancel {
            self.controller.gem_cancel();
            self.status.info("Gem command canceled");
            return;
        }
        if apply && let Some(slot) = self.controller.gem_session_mut() {
            *slot = session;
            match self.controller.gem_apply() {
                Ok(()) => {
                    self.status.info("Gem committed");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
        }
    }

    fn gem_editor_window(&mut self, ctx: &egui::Context, mut editor: GemPropertiesEditor) {
        let mut open = true;
        let mut apply = false;
        let mut close = false;
        egui::Window::new("Gem Properties")
            .id(egui::Id::new("om9-gem-properties"))
            .open(&mut open)
            .default_width(340.0)
            .resizable(false)
            .show(ctx, |ui| {
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
                    if ui.button("Apply").clicked() {
                        apply = true;
                    }
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            });
        if !open {
            close = true;
        }
        if close {
            self.controller.gem_properties_close();
            return;
        }
        if apply {
            if let Some(slot) = self.controller.gem_editor_mut() {
                *slot = editor;
            }
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

fn loader_ui(ui: &mut egui::Ui, session: &mut GemSession) {
    egui::ComboBox::from_id_salt("om9-gem-cut")
        .selected_text(session.cut.label())
        .show_ui(ui, |ui| {
            for cut in GemCut::ALL {
                if ui
                    .selectable_label(session.cut == cut, cut.label())
                    .clicked()
                    && session.cut != cut
                {
                    session.cut = cut;
                    if !cut.shapes().contains(&session.shape) {
                        session.shape = cut.shapes()[0];
                    }
                    session.size_index = 0;
                    session.size_page = 1;
                }
            }
        });
    ui.horizontal_wrapped(|ui| {
        for shape in session.cut.shapes() {
            let selected = session.shape == *shape;
            if ui.selectable_label(selected, shape.label()).clicked() && !selected {
                session.shape = *shape;
                session.size_index = 0;
                session.size_page = 1;
                session.corner_mm =
                    shape.default_corner_ratio() * session.dimensions[1].min(session.dimensions[0]);
            }
        }
    });
    ui.separator();
    let Some(asset) = GemLibrary::canonical()
        .asset(session.shape, session.cut)
        .cloned()
    else {
        ui.label("This cut has no asset for the selected shape.");
        return;
    };
    let page = session.size_page.max(1);
    let pages = asset.sizes.len().div_ceil(12).max(1);
    ui.horizontal(|ui| {
        ui.label(format!("Size (page {page}/{pages})"));
        if ui.button("◀").clicked() {
            session.size_page = session.size_page.saturating_sub(1).max(1);
        }
        if ui.button("▶").clicked() {
            session.size_page = (session.size_page + 1).min(pages);
        }
    });
    let start = (page - 1) * 12;
    let end = (start + 12).min(asset.sizes.len());
    egui::Grid::new("om9-gem-sizes")
        .num_columns(4)
        .spacing([8.0, 2.0])
        .show(ui, |ui| {
            for (offset, size) in asset.sizes[start..end].iter().enumerate() {
                let index = start + offset;
                if ui
                    .selectable_label(session.size_index == index, size.label())
                    .clicked()
                {
                    session.size_index = index;
                    session.dimensions = [size.length_mm, size.width_mm, size.depth_mm];
                    session.girdle_mm = (size.span_mm() * 0.02).max(0.06);
                }
                if (offset + 1) % 4 == 0 {
                    ui.end_row();
                }
            }
        });
    ui.separator();
    ui.checkbox(&mut session.keep_original, "Keep Original Size");
    dimension_ui(ui, session);
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

fn dimension_ui(ui: &mut egui::Ui, session: &mut GemSession) {
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
    if session.shape.default_corner_ratio() > 0.0 {
        ui.horizontal(|ui| {
            ui.label("Corner");
            ui.add(
                egui::DragValue::new(&mut session.corner_mm)
                    .speed(0.02)
                    .suffix(" mm"),
            );
        });
    }
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
