// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Gem Cutter window and its viewport preview (`OM9-L2-04-16`).
//!
//! The window is the manual's cutter dialog: the builder Mode, the seat/depth controls of the
//! Gem Cutter, the library Preset and 2D scale, the Interactive Boolean and the Preserve answer.
//! Every edit rebuilds the preview; nothing reaches the document until Apply (Enter).
use eframe::egui;
use om9_core::{
    cutters::{
        EdgeKind, EdgeShape,
        boolean::{BooleanMode, CutterKeep},
        gem::CutterMode,
        library::CutterPreset,
    },
    geometry::Transform,
};
use om9_scene::camera::Viewport;

use super::MatrixApp;
use crate::session::gem_cutters::CutterTool;

fn number_row(ui: &mut egui::Ui, label: &str, value: &mut f64, suffix: &str) -> bool {
    let before = *value;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).speed(0.05).suffix(suffix));
    });
    *value != before
}

impl MatrixApp {
    /// The live preview: staged cutter bodies in amber, the Boolean cut result in green.
    pub(super) fn gem_cutters_preview_ui(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
    ) {
        let Some(session) = self.controller.gem_cutter_session() else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        let project = |mesh: &om9_core::geometry::Mesh, colour: egui::Color32| {
            for [a, b] in super::gem::gem_preview_segments(
                mesh,
                &Transform::identity(),
                &view.camera,
                viewport,
            ) {
                painter.line_segment([a, b], egui::Stroke::new(1.0, colour));
            }
        };
        for mesh in session.body_meshes() {
            project(&mesh, egui::Color32::from_rgb(255, 176, 64));
        }
        if let Some(cut) = &session.cut {
            project(&cut.result, egui::Color32::from_rgb(120, 230, 150));
        }
    }

    /// The cutter window. `Apply` commits one transaction; `Cancel` drops the session.
    pub(super) fn gem_cutters_window_ui(&mut self, ctx: &egui::Context) {
        let Some(mut session) = self.controller.gem_cutter_session().cloned() else {
            return;
        };
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let mut dirty = false;
        let title = format!("{} ({} stone)", session.tool.label(), session.stones.len());
        egui::Window::new(title)
            .id(egui::Id::new(("om9-gem-cutters", session.tool.feature())))
            .open(&mut open)
            .default_width(410.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(session.message.clone());
                ui.separator();
                match session.tool {
                    CutterTool::GemCutter => {
                        let mut mode = session.params.mode;
                        ui.horizontal(|ui| {
                            ui.label("Mode");
                            egui::ComboBox::from_id_salt("cutter-mode")
                                .selected_text(mode.label())
                                .show_ui(ui, |ui| {
                                    for candidate in CutterMode::ALL {
                                        ui.selectable_value(
                                            &mut mode,
                                            candidate,
                                            candidate.label(),
                                        );
                                    }
                                });
                        });
                        if mode != session.params.mode {
                            session.params = session.params.with_mode(mode);
                            dirty = true;
                        }
                        dirty |= number_row(
                            ui,
                            "Girdle scale",
                            &mut session.params.girdle_scale_percent,
                            " %",
                        );
                        dirty |= number_row(
                            ui,
                            "Lower seat size",
                            &mut session.params.lower_seat_size_percent,
                            " %",
                        );
                        dirty |= number_row(
                            ui,
                            "Lower depth",
                            &mut session.params.lower_depth_percent,
                            " %",
                        );
                        dirty |=
                            number_row(ui, "Clearance", &mut session.params.clearance_mm, " mm");
                        let mut edge = session.params.edge.kind;
                        ui.horizontal(|ui| {
                            ui.label("Edge profile");
                            egui::ComboBox::from_id_salt("cutter-edge")
                                .selected_text(edge.label())
                                .show_ui(ui, |ui| {
                                    for candidate in EdgeKind::ALL {
                                        ui.selectable_value(
                                            &mut edge,
                                            candidate,
                                            candidate.label(),
                                        );
                                    }
                                });
                        });
                        if edge != session.params.edge.kind {
                            session.params.edge = EdgeShape {
                                kind: edge,
                                ..session.params.edge
                            };
                            dirty = true;
                        }
                    }
                    CutterTool::GemCutterLibrary => {
                        let mut preset = session.library.preset;
                        ui.horizontal(|ui| {
                            ui.label("Preset");
                            egui::ComboBox::from_id_salt("cutter-preset")
                                .selected_text(preset.label())
                                .show_ui(ui, |ui| {
                                    for candidate in CutterPreset::ALL {
                                        ui.selectable_value(
                                            &mut preset,
                                            candidate,
                                            candidate.label(),
                                        );
                                    }
                                });
                        });
                        if preset != session.library.preset {
                            session.library.preset = preset;
                            dirty = true;
                        }
                        dirty |= number_row(
                            ui,
                            "Scale X",
                            &mut session.library.scale_x_percent,
                            " %",
                        );
                        dirty |= number_row(
                            ui,
                            "Scale Y",
                            &mut session.library.scale_y_percent,
                            " %",
                        );
                        dirty |= number_row(
                            ui,
                            "Z scale",
                            &mut session.library.z_scale_percent,
                            " %",
                        );
                        dirty |= number_row(ui, "Z offset", &mut session.library.z_offset_mm, " mm");
                    }
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Boolean");
                    egui::ComboBox::from_id_salt("cutter-boolean")
                        .selected_text(session.boolean.label())
                        .show_ui(ui, |ui| {
                            for candidate in BooleanMode::ALL {
                                ui.selectable_value(
                                    &mut session.boolean,
                                    candidate,
                                    candidate.label(),
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Cutter body");
                    egui::ComboBox::from_id_salt("cutter-keep")
                        .selected_text(session.keep.label())
                        .show_ui(ui, |ui| {
                            for candidate in CutterKeep::ALL {
                                ui.selectable_value(&mut session.keep, candidate, candidate.label());
                            }
                        });
                });
                if session.boolean.cuts() && session.target.is_none() {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 120, 90),
                        "Select a metal body for the Interactive Boolean.",
                    );
                }
                ui.separator();
                ui.horizontal(|ui| {
                    cancel = ui.button("Cancel").clicked();
                    apply = ui.button("Apply").clicked();
                });
            });
        if dirty {
            match session.recompute() {
                Ok(()) => {
                    if let Some(stored) = self.controller.gem_cutter_session_mut() {
                        *stored = session;
                    }
                }
                Err(error) => self.controller.warn(&format!("{error:?}")),
            }
        } else if apply {
            if let Err(error) = self.controller.gem_cutter_commit() {
                self.controller.warn(&format!("{error:?}"));
            }
        } else if cancel || !open {
            self.controller.gem_cutter_cancel();
        }
    }
}
