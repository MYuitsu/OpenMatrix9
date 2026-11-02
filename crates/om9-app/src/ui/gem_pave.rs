// SPDX-License-Identifier: LGPL-2.1-or-later
//! Floating windows of the pavé/azure/prong/sphere/springs builders (`OM9-L2-04-12`).
//!
//! The window edits the preview session only. Apply commits one transaction through
//! `Controller::gem_pave_apply`; Cancel drops the session.

use eframe::egui;

use super::MatrixApp;
use crate::session::gem_pave::{PaveKind, PaveTool};
use om9_core::gem_pave::{PaveCorrection, PavePattern, PavePreview, PaveStart, RemovalPattern};
use om9_scene::camera::Viewport;

impl MatrixApp {
    /// The live preview: stone wireframes plus azure/prong polylines.
    pub(super) fn gem_pave_preview_ui(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
    ) {
        let painter = ui.painter().with_clip_rect(rect);
        let color = egui::Color32::from_rgb(255, 196, 92);
        for (mesh, placement) in self.controller.gem_pave_world_preview() {
            for [a, b] in
                super::gem::gem_preview_segments(&mesh.mesh, &placement, &view.camera, viewport)
            {
                painter.line_segment([a, b], egui::Stroke::new(1.0, color));
            }
        }
        for points in self.controller.gem_pave_world_curves() {
            for pair in points.windows(2) {
                if let (Ok(Some(a)), Ok(Some(b))) = (
                    view.camera.project(pair[0], viewport),
                    view.camera.project(pair[1], viewport),
                ) {
                    painter.line_segment(
                        [
                            egui::pos2(a.screen[0] as f32, a.screen[1] as f32),
                            egui::pos2(b.screen[0] as f32, b.screen[1] as f32),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 205, 255)),
                    );
                }
            }
        }
        if let Some(session) = self.controller.gem_pave_session()
            && session.preview == PavePreview::Framework
        {
            for [a, b] in self.controller.gem_pave_world_framework() {
                if let (Ok(Some(a)), Ok(Some(b))) = (
                    view.camera.project(a, viewport),
                    view.camera.project(b, viewport),
                ) {
                    painter.line_segment(
                        [
                            egui::pos2(a.screen[0] as f32, a.screen[1] as f32),
                            egui::pos2(b.screen[0] as f32, b.screen[1] as f32),
                        ],
                        egui::Stroke::new(0.7, egui::Color32::from_rgb(90, 130, 220)),
                    );
                }
            }
        }
    }

    pub(super) fn gem_pave_window_ui(&mut self, ctx: &egui::Context) {
        let Some(session) = self.controller.gem_pave_session().cloned() else {
            return;
        };
        let mut session = session;
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let mut refresh = false;
        let mut dirty = false;
        let style_names = self.controller.gem_pave_style_names();
        let style_error = self.controller.gem_pave_style_error().map(str::to_owned);
        let mut style_name = session.style_name.clone();
        let mut apply_style = false;
        let mut save_style = false;
        egui::Window::new(session.tool.title())
            .id(egui::Id::new(("om9-gem-pave", session.tool.command())))
            .open(&mut open)
            .default_width(400.0)
            .resizable(false)
            .show(ctx, |ui| {
                match &mut session.kind {
                    PaveKind::Stone(params) => {
                        dirty |= stone_ui(ui, params, session.tool, &mut session.quantity);
                        ui.horizontal(|ui| {
                            let label = if style_name.is_empty() {
                                "Choose style".to_owned()
                            } else {
                                style_name.clone()
                            };
                            egui::ComboBox::from_id_salt("pave-style")
                                .selected_text(label)
                                .show_ui(ui, |ui| {
                                    for name in &style_names {
                                        ui.selectable_value(&mut style_name, name.clone(), name);
                                    }
                                });
                            apply_style = ui.button("Apply style").clicked();
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut style_name)
                                    .hint_text("Style name")
                                    .desired_width(160.0),
                            );
                            save_style = ui.button("Save style").clicked();
                        });
                        if let Some(error) = &style_error {
                            ui.colored_label(egui::Color32::from_rgb(255, 140, 120), error);
                        }
                    }
                    PaveKind::Azure(params) => {
                        dirty |= ui
                            .checkbox(&mut params.hexagonal, "Hexagonal azure")
                            .changed();
                        dirty |= slider(ui, &mut params.angle_deg, -180.0..=180.0, "Angle", "°");
                        dirty |= slider(
                            ui,
                            &mut params.azure_depth_mm,
                            0.1..=10.0,
                            "Azure depth",
                            " mm",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.azure_size_mm,
                            0.0..=5.0,
                            "Azure size",
                            " mm",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.through_hole_mm,
                            0.1..=10.0,
                            "Through hole",
                            " mm",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.seat_angle_deg,
                            0.0..=60.0,
                            "Seat angle",
                            "°",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.min_spacing_mm,
                            0.0..=2.0,
                            "Minimum spacing",
                            " mm",
                        );
                        dirty |= slider(ui, &mut params.scale, 0.2..=3.0, "Scale", "×");
                    }
                    PaveKind::Prong(params) => {
                        dirty |= slider(ui, &mut params.fillet, 0.0..=1.0, "Fillet", "");
                        dirty |= slider(ui, &mut params.height_mm, 0.1..=5.0, "Height", " mm");
                        dirty |= slider(ui, &mut params.drop_mm, -2.0..=2.0, "Drop", " mm");
                        dirty |= slider(ui, &mut params.taper, 0.2..=2.0, "Taper", "×");
                        dirty |= slider(ui, &mut params.scale, 0.2..=3.0, "Scale", "×");
                        dirty |= slider(
                            ui,
                            &mut params.shared_size_mm,
                            0.1..=2.0,
                            "Shared size",
                            " mm",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.unshared_size_mm,
                            0.1..=2.0,
                            "Unshared size",
                            " mm",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.center_min_mm,
                            0.1..=2.0,
                            "Center min",
                            " mm",
                        );
                        dirty |= slider(
                            ui,
                            &mut params.center_max_mm,
                            0.1..=4.0,
                            "Center max",
                            " mm",
                        );
                        dirty |= slider(ui, &mut params.touch_fraction, 0.0..=1.0, "Touch", "");
                    }
                    PaveKind::Sphere(params) => {
                        dirty |= slider(
                            ui,
                            &mut params.radius_mm,
                            1.0..=100.0,
                            "Sphere radius",
                            " mm",
                        );
                        let mut count = params.count.unwrap_or(0);
                        if ui
                            .add(egui::DragValue::new(&mut count).speed(1.0).range(0..=1024))
                            .changed()
                        {
                            params.count = (count > 0).then_some(count);
                            dirty = true;
                        }
                        dirty |= ui
                            .checkbox(&mut params.count.is_some(), "Use explicit count")
                            .changed();
                        if params.count.is_some()
                            && ui
                                .add(
                                    egui::DragValue::new(params.count.get_or_insert(60))
                                        .speed(1.0)
                                        .range(1..=1024),
                                )
                                .changed()
                        {
                            dirty = true;
                        }
                        if params.count.is_none() {
                            let mut x = params.x as i32;
                            if ui
                                .add(egui::DragValue::new(&mut x).range(0..=128).prefix("X "))
                                .changed()
                            {
                                params.x = x.max(0) as u32;
                                dirty = true;
                            }
                            let mut t = params.t as i32;
                            if ui
                                .add(egui::DragValue::new(&mut t).range(0..=128).prefix("T "))
                                .changed()
                            {
                                params.t = t.max(0) as u32;
                                dirty = true;
                            }
                        }
                        let mut no_removal = params.removal == RemovalPattern::None;
                        if ui.checkbox(&mut no_removal, "No removal").changed() {
                            params.removal = if no_removal {
                                RemovalPattern::None
                            } else {
                                RemovalPattern::PatternA
                            };
                            dirty = true;
                        }
                        dirty |= slider(
                            ui,
                            &mut params.min_size_mm,
                            0.2..=3.0,
                            "Minimum size",
                            " mm",
                        );
                        let mut amount = params.relax_amount as i32;
                        if ui
                            .add(egui::Slider::new(&mut amount, 0..=64).text("Relax amount"))
                            .changed()
                        {
                            params.relax_amount = amount.max(0) as u32;
                            dirty = true;
                        }
                        dirty |= slider(
                            ui,
                            &mut params.relax_strength,
                            0.0..=1.0,
                            "Relax strength",
                            "",
                        );
                    }
                    PaveKind::Spring(params) => {
                        dirty |= slider(ui, &mut params.spacing_mm, 0.2..=10.0, "Spacing", " mm");
                        dirty |= slider(
                            ui,
                            &mut params.min_spacing_mm,
                            0.0..=2.0,
                            "Minimum spacing",
                            " mm",
                        );
                        let mut iterations = params.iterations as i32;
                        if ui
                            .add(egui::Slider::new(&mut iterations, 0..=64).text("Iterations"))
                            .changed()
                        {
                            params.iterations = iterations as u32;
                            dirty = true;
                        }
                        dirty |= slider(ui, &mut params.strength, 0.0..=1.0, "Strength", "");
                        dirty |= slider(ui, &mut params.resize, 0.5..=2.0, "Resize", "×");
                    }
                }
                if matches!(session.kind, PaveKind::Stone(_)) {
                    let mut preview = session.preview;
                    egui::ComboBox::from_label("Preview")
                        .selected_text(match preview {
                            PavePreview::Stones => "Stones",
                            PavePreview::Framework => "Framework",
                        })
                        .show_ui(ui, |ui| {
                            dirty |= ui
                                .selectable_value(&mut preview, PavePreview::Stones, "Stones")
                                .clicked();
                            dirty |= ui
                                .selectable_value(&mut preview, PavePreview::Framework, "Framework")
                                .clicked();
                        });
                    session.preview = preview;
                }
                ui.separator();
                ui.label(session.summary());
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").clicked();
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    if session.edit.is_some() && ui.button("Refresh").clicked() {
                        refresh = true;
                    }
                    if ui.button("Reset").clicked() {
                        dirty = true;
                        session = reset(&session);
                    }
                });
            });
        if !open || cancel {
            self.controller.gem_pave_cancel();
            self.status.info("Pavé builder canceled");
            return;
        }
        session.style_name = style_name;
        if apply_style {
            match self.controller.gem_pave_apply_style(&session.style_name) {
                Ok(()) => {
                    self.status.info("Pavé style applied");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
            return;
        }
        if save_style {
            match self.controller.gem_pave_save_style(&session.style_name) {
                Ok(()) => {
                    self.status.info("Pavé style saved");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
            return;
        }
        if refresh {
            match self.controller.gem_pave_refresh() {
                Ok(()) => {
                    self.status.info("Pavé stations refreshed");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
            return;
        }
        if dirty && let Err(error) = self.controller.gem_pave_set(session.clone()) {
            self.status.error(&error.to_string());
            return;
        }
        if apply {
            if let Err(error) = self.controller.gem_pave_set(session) {
                self.status.error(&error.to_string());
                return;
            }
            match self.controller.gem_pave_apply() {
                Ok(()) => {
                    self.status.info("Pavé layout committed");
                    self.controller.gem_pave_cancel();
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
        }
    }
}

fn slider(
    ui: &mut egui::Ui,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
    label: &str,
    suffix: &str,
) -> bool {
    ui.add(egui::Slider::new(value, range).text(label).suffix(suffix))
        .changed()
}

fn stone_ui(
    ui: &mut egui::Ui,
    params: &mut om9_core::gem_pave::PaveParams,
    tool: PaveTool,
    quantity: &mut Option<usize>,
) -> bool {
    let mut dirty = false;
    let mut pattern = params.pattern;
    egui::ComboBox::from_label("Pattern")
        .selected_text(pattern.label())
        .show_ui(ui, |ui| {
            for candidate in [
                PavePattern::Hex,
                PavePattern::Square,
                PavePattern::Tri,
                PavePattern::HexPent,
                PavePattern::AllDirections,
                PavePattern::HexReverse,
            ] {
                if ui
                    .selectable_value(&mut pattern, candidate, candidate.label())
                    .clicked()
                {
                    dirty = true;
                }
            }
        });
    params.pattern = pattern;
    let mut start = params.start;
    egui::ComboBox::from_label("Start")
        .selected_text(match start {
            PaveStart::BaseDirection => "Base Direction",
            PaveStart::Curve => "Curve",
            PaveStart::Edges => "Edges",
            PaveStart::SeedGems => "Seed Gems",
        })
        .show_ui(ui, |ui| {
            for candidate in [
                PaveStart::BaseDirection,
                PaveStart::Curve,
                PaveStart::Edges,
                PaveStart::SeedGems,
            ] {
                let label = match candidate {
                    PaveStart::BaseDirection => "Base Direction",
                    PaveStart::Curve => "Curve",
                    PaveStart::Edges => "Edges",
                    PaveStart::SeedGems => "Seed Gems",
                };
                if ui.selectable_value(&mut start, candidate, label).clicked() {
                    dirty = true;
                }
            }
        });
    params.start = start;
    dirty |= slider(ui, &mut params.spacing_mm, 0.2..=10.0, "Spacing", " mm");
    dirty |= slider(
        ui,
        &mut params.min_spacing_mm,
        0.0..=2.0,
        "Minimum spacing",
        " mm",
    );
    dirty |= slider(ui, &mut params.base_size_mm, 0.3..=10.0, "Base size", " mm");
    dirty |= slider(ui, &mut params.min_size_mm, 0.2..=5.0, "Min size", " mm");
    dirty |= slider(ui, &mut params.max_size_mm, 0.3..=15.0, "Max size", " mm");
    dirty |= slider(ui, &mut params.angle_deg, -180.0..=180.0, "Angle", "°");
    dirty |= ui.checkbox(&mut params.flip, "Flip").changed();
    dirty |= slider(ui, &mut params.margin_mm, 0.0..=5.0, "Margin", " mm");
    dirty |= slider(ui, &mut params.border_mm, 0.0..=5.0, "Border", " mm");
    dirty |= slider(ui, &mut params.taper, 0.0..=1.0, "Taper", "");
    dirty |= slider(ui, &mut params.equalize, 0.0..=1.0, "Equalize", "");
    dirty |= slider(ui, &mut params.perturb, 0.0..=1.0, "Perturb", "");
    dirty |= slider(
        ui,
        &mut params.relax_strength,
        0.0..=1.0,
        "Relax strength",
        "",
    );
    let mut iterations = params.relax_iterations as i32;
    if ui
        .add(egui::Slider::new(&mut iterations, 0..=32).text("Relax iterations"))
        .changed()
    {
        params.relax_iterations = iterations as u32;
        dirty = true;
    }
    let mut pre = params.pre;
    egui::ComboBox::from_label("Pre-sizing")
        .selected_text(correction_label(pre))
        .show_ui(ui, |ui| {
            for candidate in [
                PaveCorrection::Normal,
                PaveCorrection::Basic,
                PaveCorrection::Grow,
                PaveCorrection::Shrink,
            ] {
                if ui
                    .selectable_value(&mut pre, candidate, correction_label(candidate))
                    .clicked()
                {
                    dirty = true;
                }
            }
        });
    params.pre = pre;
    let mut post = params.post;
    egui::ComboBox::from_label("Post-sizing")
        .selected_text(correction_label(post))
        .show_ui(ui, |ui| {
            for candidate in [
                PaveCorrection::Normal,
                PaveCorrection::Basic,
                PaveCorrection::Grow,
                PaveCorrection::Shrink,
            ] {
                if ui
                    .selectable_value(&mut post, candidate, correction_label(candidate))
                    .clicked()
                {
                    dirty = true;
                }
            }
        });
    params.post = post;
    if tool == PaveTool::Manual {
        let mut count = quantity.unwrap_or(0);
        if ui
            .add(
                egui::DragValue::new(&mut count)
                    .speed(1.0)
                    .range(0..=1024)
                    .prefix("Exact count "),
            )
            .changed()
        {
            *quantity = (count > 0).then_some(count);
            dirty = true;
        }
    }
    dirty
}

fn correction_label(value: PaveCorrection) -> &'static str {
    match value {
        PaveCorrection::Normal => "Normal",
        PaveCorrection::Basic => "Basic",
        PaveCorrection::Grow => "Grow",
        PaveCorrection::Shrink => "Shrink",
    }
}

fn reset(
    session: &crate::session::gem_pave::GemPaveSession,
) -> crate::session::gem_pave::GemPaveSession {
    crate::session::gem_pave::reset_session(session).unwrap_or_else(|_| session.clone())
}
