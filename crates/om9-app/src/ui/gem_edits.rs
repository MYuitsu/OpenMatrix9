// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem Map overlay and Gem Reporter key window (`OM9-L2-04-13`, L3-007/L3-008).
//!
//! The map rows are typed data in the open session (`om9_core::gem_edits::GemMapEntry`); this
//! module only projects them. The viewport paints one numbered marker per stone with its girdle
//! size, and the key window lists the same numbers so the drawing can be read off the screen.
//! The reporter key window prints the count/weight rows with the material density and the
//! trade-formula cross-check, and never presents the computed mass as a certificate.

use eframe::egui;

use super::MatrixApp;
use crate::session::gem_edits::GemEditsTool;
use om9_core::gem_edits::GemMapEntry;
use om9_scene::camera::{Camera, Viewport};

/// One projected map label: the stone anchor and the screen position of its caption.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GemMapLabel {
    /// The projected stone centre.
    pub anchor: egui::Pos2,
    /// The top-left of the caption.
    pub label: egui::Pos2,
    /// `#n  L × W`.
    pub text: String,
    /// The key line shown in the window: `#n Shape Cut · L × W mm @ (x, y, z)`.
    pub key: String,
}

/// Project the map rows into numbered labels inside the viewport rectangle.
///
/// A stone behind the camera or outside the rectangle is skipped instead of being pinned to the
/// edge, so a caption never points at an empty part of the screen. The caption is offset up and
/// right of the stone and clamped into the rectangle.
pub(crate) fn gem_map_labels(
    rows: &[GemMapEntry],
    camera: &Camera,
    viewport: Viewport,
    rect: egui::Rect,
) -> Vec<GemMapLabel> {
    let mut labels = Vec::new();
    for row in rows {
        let Ok(Some(projected)) = camera.project(row.point, viewport) else {
            continue;
        };
        let anchor = egui::pos2(projected.screen[0] as f32, projected.screen[1] as f32);
        if !rect.contains(anchor) {
            continue;
        }
        let text = format!(
            "#{}  {:.2} × {:.2}",
            row.number, row.length_mm, row.width_mm
        );
        let label = egui::pos2(
            (anchor.x + 18.0).min(rect.right() - 8.0),
            (anchor.y - 22.0).max(rect.top() + 8.0),
        );
        let key = format!(
            "#{} {} {} · {:.2} × {:.2} mm @ ({:.2}, {:.2}, {:.2})",
            row.number,
            row.shape.label(),
            row.cut.label(),
            row.length_mm,
            row.width_mm,
            row.point[0],
            row.point[1],
            row.point[2]
        );
        labels.push(GemMapLabel {
            anchor,
            label,
            text,
            key,
        });
    }
    labels
}

impl MatrixApp {
    /// Paint the open Gem Map: one numbered marker and girdle size per stone.
    pub(super) fn gem_map_overlay_ui(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
    ) {
        let rows = self.controller.gem_map_rows();
        if rows.is_empty() {
            return;
        }
        let painter = ui.painter().with_clip_rect(rect);
        let accent = egui::Color32::from_rgb(255, 196, 92);
        for label in gem_map_labels(&rows, &view.camera, viewport, rect) {
            painter.line_segment(
                [label.anchor, label.label],
                egui::Stroke::new(1.0, accent.gamma_multiply(0.85)),
            );
            painter.circle_filled(label.anchor, 3.4, accent);
            painter.circle_stroke(
                label.anchor,
                6.2,
                egui::Stroke::new(1.0, accent.gamma_multiply(0.7)),
            );
            painter.text(
                label.label,
                egui::Align2::LEFT_TOP,
                label.text,
                egui::FontId::monospace(11.0),
                egui::Color32::from_rgb(255, 226, 170),
            );
        }
    }

    /// The key window of an open Gem Map or Gem Reporter.
    pub(super) fn gem_edits_window_ui(&mut self, ctx: &egui::Context) {
        let tool = self.controller.gem_edits_session().map(|session| session.tool);
        match tool {
            Some(GemEditsTool::Map) => self.gem_map_window(ctx),
            Some(GemEditsTool::Reporter) => self.gem_report_window(ctx),
            _ => {}
        }
    }

    fn gem_map_window(&mut self, ctx: &egui::Context) {
        let rows = self.controller.gem_map_rows();
        if rows.is_empty() {
            return;
        }
        let mut open = true;
        let mut close = false;
        egui::Window::new(format!("Gem Map · {} stone(s)", rows.len()))
            .id(egui::Id::new("om9-gem-map"))
            .collapsible(false)
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Numbers match the viewport markers. Sizes are girdle L × W in mm.");
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for row in &rows {
                        ui.monospace(om9_core::gem_edits::map_line(row));
                    }
                });
                if ui.button("Close (Enter)").clicked() {
                    close = true;
                }
            });
        if close || !open {
            self.controller.gem_edits_cancel();
        }
    }

    fn gem_report_window(&mut self, ctx: &egui::Context) {
        let Some(report) = self.controller.gem_report_data() else {
            return;
        };
        let mut open = true;
        let mut close = false;
        egui::Window::new(format!(
            "Gem Reporter · {} stone(s) · {:.3} ct",
            report.total_count, report.total_carat
        ))
        .id(egui::Id::new("om9-gem-report"))
        .collapsible(false)
        .resizable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                for row in &report.rows {
                    ui.monospace(om9_core::gem_edits::report_line(row));
                }
            });
            ui.separator();
            ui.label(format!(
                "Total {:.3} ct · {:.3} g · tolerance ±{:.0}% vs trade formula",
                report.total_carat, report.total_grams, report.tolerance_percent
            ));
            if report.assumed_density > 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(230, 180, 90),
                    format!(
                        "{} stone(s) use the diamond basis: their material density is unknown.",
                        report.assumed_density
                    ),
                );
            }
            if report.outside_tolerance > 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(230, 140, 110),
                    format!(
                        "{} row(s) outside ±{:.0}% of the trade formula — inspect the model.",
                        report.outside_tolerance, report.tolerance_percent
                    ),
                );
            }
            ui.colored_label(
                egui::Color32::from_rgb(160, 190, 210),
                "Computed from the authored solid, not a gemological certificate.",
            );
            if ui.button("Close (Enter)").clicked() {
                close = true;
            }
        });
        if close || !open {
            self.controller.gem_edits_cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{
        gem::GemId,
        identity::ObjectId,
    };

    fn camera() -> Camera {
        Camera::orthographic([0.0, 0.0, 100.0], [0.0; 3], [0.0, 1.0, 0.0], 20.0).expect("camera")
    }

    fn viewport() -> Viewport {
        Viewport {
            origin: [0.0, 0.0],
            size: [800.0, 600.0],
        }
    }

    fn rect() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))
    }

    fn entry(number: usize, x: f64) -> GemMapEntry {
        GemMapEntry {
            gem: GemId(ObjectId::new()),
            number,
            shape: om9_core::gem::GemShape::Round,
            cut: om9_core::gem::GemCut::Diamond,
            length_mm: 6.0,
            width_mm: 6.0,
            point: [x, 0.0, 0.0],
        }
    }

    #[test]
    fn every_visible_stone_gets_a_numbered_caption_inside_the_viewport() {
        let rows = vec![entry(1, -4.0), entry(2, 0.0), entry(3, 4.0)];
        let labels = gem_map_labels(&rows, &camera(), viewport(), rect());
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0].text, "#1  6.00 × 6.00");
        assert_eq!(labels[2].text, "#3  6.00 × 6.00");
        assert!(labels[0].key.starts_with("#1 Round Diamond Cut · 6.00 × 6.00 mm @ (-4.00,"));
        for label in &labels {
            assert!(rect().contains(label.anchor), "anchor on screen");
            assert!(rect().contains(label.label), "caption stays on screen");
        }
        assert!(labels[0].anchor.x < labels[1].anchor.x);
        assert!(labels[1].anchor.x < labels[2].anchor.x);
    }

    #[test]
    fn a_stone_outside_the_view_is_skipped_instead_of_pinned_to_the_edge() {
        let rows = vec![entry(1, 0.0), entry(2, 400.0)];
        let labels = gem_map_labels(&rows, &camera(), viewport(), rect());
        assert_eq!(labels.len(), 1, "the far stone has no marker");
        assert_eq!(labels[0].text, "#1  6.00 × 6.00");
        let empty: Vec<GemMapEntry> = Vec::new();
        assert!(gem_map_labels(&empty, &camera(), viewport(), rect()).is_empty());
    }

    #[test]
    fn a_round_stone_with_a_custom_profile_still_maps_its_girdle() {
        let mut tall = entry(1, 0.0);
        tall.length_mm = 6.0;
        tall.width_mm = 6.0;
        let labels = gem_map_labels(&[tall], &camera(), viewport(), rect());
        assert_eq!(labels.len(), 1);
        assert!(labels[0].text.contains("6.00"));
    }
}
