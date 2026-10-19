// SPDX-License-Identifier: LGPL-2.1-or-later
//! Screen-space rotation feedback. Internal subdivision edges must never become visible strokes.
use eframe::egui::{self, Color32, Pos2, Stroke};

pub(super) fn paint_sector(painter: &egui::Painter, sector: &[Pos2]) {
    if sector.len() < 3 || sector.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return;
    }
    let centre = sector[0];
    let mut twice_area = 0f64;
    let mut perimeter = 0f64;
    for pair in sector.windows(2) {
        let a = pair[0] - centre;
        let b = pair[1] - centre;
        twice_area += (f64::from(a.x) * f64::from(b.y) - f64::from(a.y) * f64::from(b.x)).abs();
        perimeter += f64::from(pair[0].distance(pair[1]));
    }
    perimeter += f64::from(sector[sector.len() - 1].distance(centre));
    // Area/perimeter measures the projected width. Edge-on sectors have no useful area and
    // antialiasing their repeatedly retraced boundaries creates a false bright yellow band.
    if perimeter == 0. || twice_area / perimeter < 1. {
        return;
    }
    let mut mesh = egui::Mesh::default();
    let fill = Color32::from_rgba_unmultiplied(255, 220, 0, 24);
    for point in sector {
        mesh.colored_vertex(*point, fill);
    }
    for index in 1..sector.len() - 1 {
        mesh.add_triangle(0, index as u32, index as u32 + 1);
    }
    // A single triangle mesh has shared interior edges, without a separate AA fringe per fan.
    painter.add(egui::Shape::mesh(mesh));
    let mut outline = sector.to_vec();
    outline.push(centre);
    painter.add(egui::Shape::line(
        outline,
        Stroke::new(1.3, Color32::from_rgb(255, 220, 0)),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sector(flatten: f32) -> Vec<Pos2> {
        let mut points = vec![egui::pos2(100., 100.)];
        for i in 0..=48 {
            let a = -std::f32::consts::FRAC_PI_4 + i as f32 * std::f32::consts::FRAC_PI_2 / 48.;
            points.push(egui::pos2(
                100. + 54. * a.cos(),
                100. + flatten * 54. * a.sin(),
            ));
        }
        points
    }
    fn shapes(points: Vec<Pos2>) -> (egui::Context, egui::FullOutput) {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(200., 200.),
                )),
                ..Default::default()
            },
            |ui| paint_sector(ui.painter(), &points),
        );
        output.textures_delta.clear();
        (ctx, output)
    }
    fn alpha_at(meshes: &[egui::ClippedPrimitive], point: Pos2) -> f32 {
        let cross = |a: egui::Vec2, b: egui::Vec2| a.x * b.y - a.y * b.x;
        let mut alpha = 0.;
        for mesh in meshes {
            if !mesh.clip_rect.contains(point) {
                continue;
            }
            let egui::epaint::Primitive::Mesh(mesh) = &mesh.primitive else {
                continue;
            };
            for tri in mesh.indices.as_chunks::<3>().0 {
                let [a, b, c] = [
                    mesh.vertices[tri[0] as usize],
                    mesh.vertices[tri[1] as usize],
                    mesh.vertices[tri[2] as usize],
                ];
                let area = cross(b.pos - a.pos, c.pos - a.pos);
                if area.abs() < 1e-9 {
                    continue;
                }
                let u = cross(b.pos - point, c.pos - point) / area;
                let v = cross(c.pos - point, a.pos - point) / area;
                let w = 1. - u - v;
                if u <= 0. || v <= 0. || w <= 0. {
                    continue;
                }
                let source =
                    (u * a.color.a() as f32 + v * b.color.a() as f32 + w * c.color.a() as f32)
                        / 255.;
                alpha = source + (1. - source) * alpha;
            }
        }
        alpha * 255.
    }
    #[test]
    fn edge_on_rotation_feedback_does_not_leave_a_bright_line() {
        for flatten in [0., 0.0001, 0.005] {
            let (_, output) = shapes(sector(flatten));
            let shapes = output.shapes;
            assert!(
                shapes.iter().all(|s| matches!(s.shape, egui::Shape::Noop)),
                "collapsed sector must not paint a yellow band: flatten={flatten}, shapes={}",
                shapes.len()
            );
        }
    }
    #[test]
    fn visible_sector_keeps_its_fill_without_internal_antialias_spokes() {
        let (ctx, output) = shapes(sector(1.));
        let meshes = ctx.tessellate(output.shapes, 1.);
        for point in [
            egui::pos2(110.2, 101.3),
            egui::pos2(112.3, 97.7),
            egui::pos2(125.7, 104.2),
        ] {
            let alpha = alpha_at(&meshes, point);
            assert!(
                (alpha - 24.).abs() < 0.1,
                "sector interior should have its requested translucent alpha, not overlapping antialias fringes: point={point:?}, alpha={alpha}"
            );
        }
    }
}
