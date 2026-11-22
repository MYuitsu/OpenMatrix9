// SPDX-License-Identifier: LGPL-2.1-or-later
//! Graphics for the placement Builder's source and parameter buttons.

use eframe::egui::{self, Color32, Painter, Pos2, Stroke, pos2, vec2};

use crate::ui::theme;

#[derive(Clone, Copy)]
pub(super) enum Illustration {
    Curve,
    Surface,
    Start,
    End,
    Gap,
    Offset,
    Roll,
    Size,
    Angle,
    Scale,
    SurfaceSpacing,
}

pub(super) fn tile(
    ui: &mut egui::Ui,
    illustration: Illustration,
    label: &str,
    width: f32,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 56.0), egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(
        egui::WidgetType::Button, ui.is_enabled(), selected, label,
    ));
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, selected);
        let border = if response.has_focus() {
            Stroke::new(1.5, theme::TITLE)
        } else if selected {
            Stroke::new(1.5, theme::ACCENT)
        } else {
            visuals.bg_stroke
        };
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect(rect, visuals.corner_radius, visuals.bg_fill, border, egui::StrokeKind::Inside);
        let center = pos2(rect.center().x, rect.top() + 20.0);
        illustration.paint(&painter, center, (width / 62.0).clamp(0.45, 1.0));
        painter.text(
            pos2(rect.center().x, rect.bottom() - 9.5), egui::Align2::CENTER_CENTER,
            label, egui::TextStyle::Small.resolve(ui.style()), theme::INK,
        );
    }
    response
}

impl Illustration {
    fn paint(self, painter: &Painter, center: Pos2, scale: f32) {
        let p = |x, y| center + vec2(x, y) * scale;
        let guide = Stroke::new(1.15, theme::ACCENT);
        let mark = Stroke::new(1.5, theme::TITLE);
        let gem = |x, y, size| stone(painter, p(x, y), size * scale);
        let line = |a, b, stroke| { painter.line_segment([a, b], stroke); };
        let arrow = |from, to| {
            line(from, to, mark);
            arrowhead(painter, from, to, mark, scale);
        };
        let dimension = |a, b| {
            arrow(a, b);
            arrowhead(painter, b, a, mark, scale);
        };

        match self {
            Self::Curve => {
                painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                    [p(-27., 12.), p(-13., -20.), p(9., 18.), p(27., -12.)],
                    false, Color32::TRANSPARENT, guide,
                ));
                for (x, y) in [(-17., -1.), (0., -3.), (17., -4.)] {
                    gem(x, y, 6.5);
                }
            }
            Self::Surface => {
                let corners = [p(-29., 5.), p(-9., -11.), p(28., -4.), p(7., 13.)];
                painter.add(egui::Shape::convex_polygon(corners.to_vec(), theme::SELECTED, guide));
                for t in [0.25, 0.5, 0.75] {
                    line(corners[0].lerp(corners[1], t), corners[3].lerp(corners[2], t), guide);
                    line(corners[0].lerp(corners[3], t), corners[1].lerp(corners[2], t), guide);
                }
                gem(0., -4., 10.5);
            }
            Self::Start | Self::End => {
                let start = matches!(self, Self::Start);
                let sign = if start { -1.0 } else { 1.0 };
                line(p(-26., 9.), p(26., 9.), guide);
                for x in [-19., 0., 19.] { gem(x, 0., 8.); }
                line(p(sign * 26., -9.), p(sign * 26., 13.), mark);
                arrow(p(sign * 8., -13.), p(sign * 25., -13.));
            }
            Self::Gap | Self::SurfaceSpacing => {
                if matches!(self, Self::SurfaceSpacing) {
                    painter.circle_stroke(p(-18., 4.), 11.5 * scale, guide);
                    painter.circle_stroke(p(18., 4.), 11.5 * scale, guide);
                }
                gem(-18., 4., 11.);
                gem(18., 4., 11.);
                for x in [-6.5, 6.5] { line(p(x, -13.), p(x, -6.), mark); }
                dimension(p(-6.5, -10.), p(6.5, -10.));
            }
            Self::Offset => {
                line(p(-28., 13.), p(27., 13.), guide);
                gem(8., -4., 15.5);
                line(p(-26., -7.), p(-8., -7.), guide);
                dimension(p(-19., -7.), p(-19., 13.));
            }
            Self::Roll | Self::Angle => {
                let roll = matches!(self, Self::Roll);
                let y = if roll { 0. } else { -5. };
                gem(0., 3., 16.);
                let arc: Vec<_> = (0..=20).map(|i| {
                    let angle = 0.3 + i as f32 * 0.255;
                    let x = 22. * angle.cos();
                    p(x, y + 7. * angle.sin() + if roll { -0.35 * x } else { 0. })
                }).collect();
                arrowhead(painter, arc[17], arc[20], mark, scale);
                painter.add(egui::Shape::line(arc, mark));
            }
            Self::Size => {
                gem(0., 3., 17.);
                for x in [-17., 17.] { line(p(x, -16.), p(x, -5.), guide); }
                dimension(p(-17., -13.), p(17., -13.));
            }
            Self::Scale => {
                gem(0., 0., 13.);
                for (x, y) in [(-1., -1.), (1., -1.), (-1., 1.), (1., 1.)] {
                    arrow(p(15. * x, 6. * y), p(24. * x, 14. * y));
                }
            }
        }
    }
}

fn arrowhead(painter: &Painter, from: Pos2, to: Pos2, pen: Stroke, scale: f32) {
    let back = (from - to).normalized();
    let across = vec2(-back.y, back.x);
    painter.add(egui::Shape::line(vec![
        to + (back * 4.5 + across * 2.8) * scale,
        to,
        to + (back * 4.5 - across * 2.8) * scale,
    ], pen));
}

fn stone(painter: &Painter, center: Pos2, size: f32) {
    let p = |x, y| center + vec2(x, y) * size;
    let left = p(-1., -0.14);
    let right = p(1., -0.14);
    let shoulder_left = p(-0.54, -0.62);
    let shoulder_right = p(0.54, -0.62);
    let table_left = p(-0.42, -0.27);
    let table_right = p(0.42, -0.27);
    let culet = p(0., 0.65);
    let edge = Stroke::new(0.8, theme::TOOL_FIELD);
    let light = theme::INK.lerp_to_gamma(theme::SNAP_ON, 0.25);
    let middle = theme::INK.lerp_to_gamma(theme::SNAP_ON, 0.58);
    let dark = theme::SNAP_ON.lerp_to_gamma(theme::BODY, 0.38);
    for (face, color) in [
        ([left, shoulder_left, table_left, left], middle),
        ([shoulder_left, shoulder_right, table_right, table_left], light),
        ([table_right, shoulder_right, right, table_right], middle),
        ([left, table_left, culet, left], dark),
        ([table_left, table_right, culet, table_left], middle),
        ([table_right, right, culet, table_right], dark),
    ] {
        let len = if face[0] == face[3] { 3 } else { 4 };
        painter.add(egui::Shape::convex_polygon(face[..len].to_vec(), color, edge));
    }
    painter.add(egui::Shape::closed_line(
        vec![left, shoulder_left, shoulder_right, right, culet], Stroke::new(0.9, theme::INK),
    ));
}
