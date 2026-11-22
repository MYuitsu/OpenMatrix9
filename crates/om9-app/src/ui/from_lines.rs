// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;

impl MatrixApp {
    pub(super) fn from_lines_view(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        camera: &Camera,
        viewport: Viewport,
        id: ViewId,
        response: &egui::Response,
    ) -> bool {
        let Some(session) = self
            .controller
            .addon_session()
            .filter(|s| s.code == crate::session::from_lines::CODE)
        else {
            self.from_lines_popup = None;
            return false;
        };
        if session.reply.want == om9_addon_abi::session::Want::Objects {
            return false;
        }
        let clicked = response.clicked() && response.hovered() && !self.viewport_menu_owns_frame;
        let Some(layout) = self.controller.from_lines_layout() else {
            return clicked;
        };
        let net = layout.network();
        let Ok(projector) = camera.projector(viewport) else {
            return clicked;
        };
        let projected: Vec<_> = net
            .vertices
            .iter()
            .map(|p| projector.project(*p).ok().flatten())
            .collect();
        let pos = |i: u32| {
            let p = projected[i as usize]?;
            Some((egui::pos2(p.screen[0] as f32, p.screen[1] as f32), p.depth))
        };
        let mut polygons = Vec::new();
        for triangles in layout.triangles() {
            for &[a, b, c] in triangles {
                if let (Some(a), Some(b), Some(c)) = (pos(a), pos(b), pos(c)) {
                    polygons.push((a.1 + b.1 + c.1, [a.0, b.0, c.0]));
                }
            }
        }
        polygons.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut mesh = egui::Mesh::default();
        let fill = Color32::from_rgba_unmultiplied(168, 74, 180, 158);
        for (_, triangle) in polygons {
            let start = mesh.vertices.len() as u32;
            for p in triangle {
                mesh.colored_vertex(p, fill);
            }
            mesh.add_triangle(start, start + 1, start + 2);
        }
        painter.add(egui::Shape::mesh(mesh));
        let cursor = response.hover_pos();
        let mut nearest = None::<(f32, f64, (u32, u32))>;
        for &e in &net.edges {
            let (Some((a, az)), Some((b, bz))) = (pos(e.0), pos(e.1)) else {
                continue;
            };
            let unused = layout.unused_edges.binary_search(&e).is_ok();
            painter.line_segment(
                [a, b],
                Stroke::new(
                    if unused { 2. } else { 1. },
                    if unused { Color32::RED } else { Color32::GOLD },
                ),
            );
            if clicked && let Some(p) = cursor {
                let d = b - a;
                let t = if d.length_sq() > 0. {
                    ((p - a).dot(d) / d.length_sq()).clamp(0., 1.)
                } else {
                    0.
                };
                let screen = p.distance(a + t * d);
                let depth = az + (bz - az) * f64::from(t);
                if screen <= 5.
                    && nearest.is_none_or(|(r, z, _)| {
                        screen < r - 0.25 || (screen - r).abs() <= 0.25 && depth < z
                    })
                {
                    nearest = Some((screen, depth, e));
                }
            }
        }
        if self.controller.from_lines_snap_marks() {
            for (i, &n) in net.endpoint_counts.iter().enumerate() {
                if n > 0
                    && let Some((p, _)) = pos(i as u32)
                {
                    let color = match n {
                        1 => Color32::RED,
                        2 => Color32::GREEN,
                        _ => Color32::LIGHT_BLUE,
                    };
                    painter.circle_filled(p, 3., color);
                    painter.text(
                        p + egui::vec2(4., -3.),
                        egui::Align2::LEFT_BOTTOM,
                        n.to_string(),
                        egui::FontId::proportional(10.),
                        color,
                    );
                }
            }
        }
        for &i in &layout.nonmanifold_vertices {
            if let Some((p, _)) = pos(i) {
                painter.circle_filled(p, 4., Color32::RED);
            }
        }
        if let Some((owner, anchor)) = self.from_lines_popup
            && owner == id
            && !self.controller.from_lines_choices().is_empty()
        {
            let mut chosen = None;
            let mut highlight = None;
            let vi = self.controller.tr().lang() == om9_addons::i18n::Lang::Vi;
            let panel = egui::Area::new(egui::Id::new("From Lines face chooser"))
                .order(egui::Order::Foreground)
                .movable(false)
                .fixed_pos(anchor)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(224.)
                            .show(ui, |ui| {
                                for (index, face) in
                                    self.controller.from_lines_choices().iter().enumerate()
                                {
                                    let n = face.outer.len();
                                    let label = if vi {
                                        format!("Mặt {} · {n} cạnh", index + 1)
                                    } else {
                                        format!("Face {} · {n} edges", index + 1)
                                    };
                                    let button = ui.button(label);
                                    self.session_dialog.rects.insert(
                                        format!("FromLines:Face{}", index + 1),
                                        button.rect,
                                    );
                                    if button.hovered() {
                                        highlight = Some(index);
                                    }
                                    if button.clicked() {
                                        chosen = Some(index);
                                    }
                                }
                            });
                    });
                });
            if let Some(i) = highlight {
                let face = &self.controller.from_lines_choices()[i];
                for ring in std::iter::once(&face.outer).chain(&face.holes) {
                    for (&a, &b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
                        if let (Some(a), Some(b)) = (pos(a), pos(b)) {
                            painter.line_segment([a.0, b.0], Stroke::new(2., Color32::WHITE));
                        }
                    }
                }
            }
            if let Some(i) = chosen {
                let r = self.controller.from_lines_add(i);
                self.feedback(r);
                self.from_lines_popup = None;
                return true;
            }
            if clicked && cursor.is_some_and(|p| !panel.response.rect.contains(p)) {
                let r = self.controller.from_lines_dismiss_choices();
                self.feedback(r);
                self.from_lines_popup = None;
            }
            return clicked;
        }
        if clicked && let Some(p) = cursor {
            if self.controller.from_lines_pending() {
                return true;
            }
            let ray = camera.visible_ray([p.x as f64, p.y as f64], viewport);
            let face = ray
                .ok()
                .and_then(|r| layout.pick_face(r.origin, r.direction));
            let result = if let Some((_, _, edge)) = nearest {
                self.from_lines_popup = Some((id, p));
                self.controller.from_lines_edge(edge)
            } else if let Some(index) = face {
                self.from_lines_popup = None;
                self.controller.from_lines_remove(index)
            } else {
                return true;
            };
            self.controller.workspace.active = id;
            self.feedback(result);
        }
        clicked
    }
}
