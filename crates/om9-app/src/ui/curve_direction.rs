// SPDX-License-Identifier: LGPL-2.1-or-later
use super::{MatrixApp, pick_tolerance};
use crate::session::curve_direction::ArrowLocation;
use eframe::egui::{self, Color32, Pos2, Stroke};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::PickIntent,
};
impl MatrixApp {
    pub(super) fn curve_direction_view(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        camera: &Camera,
        viewport: Viewport,
        response: &egui::Response,
    ) -> bool {
        let Some(session) = &self.controller.curve_direction else {
            return false;
        };
        let hover = response
            .hover_pos()
            .filter(|p| p.y > response.rect.top() + 20.);
        let hit = hover
            .and_then(|p| {
                self.controller
                    .scene()?
                    .pick(
                        [p.x as f64, p.y as f64],
                        camera,
                        viewport,
                        pick_tolerance(ui),
                        PickIntent::Select,
                    )
                    .ok()
                    .flatten()
            })
            .filter(|h| h.path.is_empty());
        for &id in &session.targets {
            let hover = hit
                .as_ref()
                .filter(|h| h.root == id)
                .map(|h| ArrowLocation::Hover(h.world_point));
            // Rhino 5 draws the same 40-pixel shaft at the ends and midpoint, even on a short line.
            for location in [0., 0.5, 1.]
                .map(ArrowLocation::Fraction)
                .into_iter()
                .chain(hover)
            {
                let Ok((base, tip)) = self.controller.direction_arrow(id, location) else {
                    continue;
                };
                let (Ok(Some(a)), Ok(Some(b))) = (
                    camera.project(base, viewport),
                    camera.project(tip, viewport),
                ) else {
                    continue;
                };
                let start = Pos2::new(a.screen[0] as f32, a.screen[1] as f32);
                let delta = Pos2::new(b.screen[0] as f32, b.screen[1] as f32) - start;
                if matches!(location, ArrowLocation::Fraction(f) if f==session.start_fraction(id)) {
                    painter.rect_filled(
                        egui::Rect::from_center_size(start, egui::vec2(3., 3.)),
                        0.,
                        Color32::WHITE,
                    );
                }
                if delta.length() > 0.001 {
                    painter.arrow(
                        start,
                        delta.normalized() * 40.,
                        Stroke::new(1.6, Color32::WHITE),
                    );
                }
            }
        }
        if response.clicked() && hover.is_some() {
            if let Some(hit) = hit {
                let result = self.controller.direction_pick(hit.root);
                self.feedback(result);
            }
            return true;
        }
        if response.secondary_clicked() && hover.is_some() {
            let result = self.controller.submit("");
            self.feedback(result);
            return true;
        }
        false
    }
}
