// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Channel / Micro Prong / Bright Cut window and its viewport preview (`OM9-L2-04-17`).
//!
//! One window serves the four row commands: the channel width/depth/end controls, the scallop
//! controls, the bright Height/Length/2D Scale, the Interactive Boolean of every selected metal
//! body and the Preserve answer. Every edit rebuilds the preview; nothing reaches the document
//! until Apply (Enter).
use eframe::egui;
use om9_core::{
    cutters::{
        boolean::{BooleanMode, CutterKeep},
        brightcut::BrightMode,
        channel::{ChannelEnd, ChannelMiddle},
    },
    geometry::Transform,
};
use om9_scene::camera::Viewport;

use super::MatrixApp;
use crate::session::channel_cutters::RowTool;

fn number_row(ui: &mut egui::Ui, label: &str, value: &mut f64, suffix: &str) -> bool {
    let before = *value;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).speed(0.05).suffix(suffix));
    });
    *value != before
}

fn toggle_row(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool {
    let before = *value;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.checkbox(value, "");
    });
    *value != before
}

impl MatrixApp {
    /// The live preview: staged cutter bodies in amber, the Boolean cut result in green.
    pub(super) fn row_cutters_preview_ui(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
    ) {
        let Some(session) = self.controller.row_cutter_session() else {
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
        for mesh in &session.cutters {
            project(mesh, egui::Color32::from_rgb(255, 176, 64));
        }
        for cut in &session.cuts {
            project(&cut.result, egui::Color32::from_rgb(120, 230, 150));
        }
    }

    /// The row cutter window. `Apply` commits one transaction; `Cancel` drops the session.
    pub(super) fn row_cutters_window_ui(&mut self, ctx: &egui::Context) {
        let Some(mut session) = self.controller.row_cutter_session().cloned() else {
            return;
        };
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let mut dirty = false;
        let title = format!(
            "{} ({} stone{}, {} body)",
            session.tool.label(),
            session.stones.len(),
            if session.stones.len() == 1 { "" } else { "s" },
            session.targets.len()
        );
        egui::Window::new(title)
            .id(egui::Id::new(("om9-row-cutters", session.tool.feature())))
            .open(&mut open)
            .default_width(430.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(session.message.clone());
                ui.separator();
                match session.tool {
                    RowTool::ChannelBuilder => {
                        dirty |= number_row(ui, "Width", &mut session.channel.width_mm, " mm");
                        dirty |= number_row(ui, "Depth", &mut session.channel.depth_mm, " mm");
                        dirty |= number_row(ui, "Height", &mut session.channel.height_mm, " mm");
                        dirty |= number_row(ui, "Wall", &mut session.channel.wall_mm, " mm");
                        dirty |= number_row(ui, "Spacing", &mut session.channel.spacing_mm, " mm");
                        let mut start = session.channel.start;
                        let mut end = session.channel.end;
                        ui.horizontal(|ui| {
                            ui.label("Start");
                            egui::ComboBox::from_id_salt("channel-start")
                                .selected_text(start.label())
                                .show_ui(ui, |ui| {
                                    for candidate in ChannelEnd::ALL {
                                        ui.selectable_value(&mut start, candidate, candidate.label());
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("End");
                            egui::ComboBox::from_id_salt("channel-end")
                                .selected_text(end.label())
                                .show_ui(ui, |ui| {
                                    for candidate in ChannelEnd::ALL {
                                        ui.selectable_value(&mut end, candidate, candidate.label());
                                    }
                                });
                        });
                        if start != session.channel.start || end != session.channel.end {
                            session.channel.start = start;
                            session.channel.end = end;
                            dirty = true;
                        }
                        dirty |= number_row(
                            ui,
                            "Start depth",
                            &mut session.channel.start_depth_mm,
                            " mm",
                        );
                        dirty |= number_row(
                            ui,
                            "End depth",
                            &mut session.channel.end_depth_mm,
                            " mm",
                        );
                        dirty |= toggle_row(ui, "Mirror", &mut session.channel.mirror);
                        dirty |= toggle_row(ui, "Flip A/B", &mut session.channel.flip);
                        let mut middle = session.channel.middle;
                        ui.horizontal(|ui| {
                            ui.label("Middle profile");
                            egui::ComboBox::from_id_salt("channel-middle")
                                .selected_text(middle.label())
                                .show_ui(ui, |ui| {
                                    for candidate in ChannelMiddle::ALL {
                                        ui.selectable_value(
                                            &mut middle,
                                            candidate,
                                            candidate.label(),
                                        );
                                    }
                                });
                        });
                        if middle != session.channel.middle {
                            session.channel.middle = middle;
                            dirty = true;
                        }
                    }
                    RowTool::MicroProng => {
                        dirty |=
                            number_row(ui, "Radius", &mut session.microprong.radius_mm, " mm");
                        dirty |= number_row(ui, "Depth", &mut session.microprong.depth_mm, " mm");
                        dirty |= number_row(ui, "Height", &mut session.microprong.height_mm, " mm");
                        let mut count = session.microprong.count as f64;
                        let before = count;
                        ui.horizontal(|ui| {
                            ui.label("Count (0 = one per gap)");
                            ui.add(egui::DragValue::new(&mut count).speed(1.0).range(0.0..=256.0));
                        });
                        if count != before {
                            session.microprong.count = count.round() as usize;
                            dirty = true;
                        }
                        dirty |=
                            number_row(ui, "Spacing", &mut session.microprong.spacing_mm, " mm");
                        dirty |=
                            number_row(ui, "Angle", &mut session.microprong.angle_deg, " °");
                    }
                    RowTool::BrightCutChannel | RowTool::BrightCutter => {
                        dirty |= number_row(ui, "Height", &mut session.bright.height_mm, " mm");
                        dirty |= number_row(ui, "Length", &mut session.bright.length_mm, " mm");
                        dirty |= number_row(
                            ui,
                            "2D scale",
                            &mut session.bright.scale_2d_percent,
                            " %",
                        );
                        dirty |= number_row(ui, "Angle", &mut session.bright.angle_deg, " °");
                        dirty |= number_row(ui, "Depth", &mut session.bright.depth_mm, " mm");
                        if session.tool == RowTool::BrightCutChannel {
                            dirty |= toggle_row(ui, "Mirror both sides", &mut session.bright.mirror);
                        } else {
                            let mut count = session.bright.count as f64;
                            let before = count;
                            ui.horizontal(|ui| {
                                ui.label("Facets");
                                ui.add(
                                    egui::DragValue::new(&mut count).speed(1.0).range(3.0..=256.0),
                                );
                            });
                            if count != before {
                                session.bright.count = count.round() as usize;
                                dirty = true;
                            }
                        }
                        ui.label(format!(
                            "{}: {:.2} mm drop over {:.2} mm",
                            BrightMode::Single.label(),
                            session.bright.slope_mm(),
                            session.bright.length_mm
                        ));
                    }
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Boolean");
                    egui::ComboBox::from_id_salt("row-cutter-boolean")
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
                    egui::ComboBox::from_id_salt("row-cutter-keep")
                        .selected_text(session.keep.label())
                        .show_ui(ui, |ui| {
                            for candidate in CutterKeep::ALL {
                                ui.selectable_value(&mut session.keep, candidate, candidate.label());
                            }
                        });
                });
                if session.boolean.cuts() && session.targets.is_empty() {
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
                    if let Some(stored) = self.controller.row_cutter_session_mut() {
                        *stored = session;
                    }
                }
                Err(error) => self.controller.warn(&format!("{error:?}")),
            }
        } else if apply {
            if let Err(error) = self.controller.row_cutter_commit() {
                self.controller.warn(&format!("{error:?}"));
            }
        } else if cancel || !open {
            self.controller.row_cutter_cancel();
        }
    }
}
