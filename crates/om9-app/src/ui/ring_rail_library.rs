// SPDX-License-Identifier: LGPL-2.1-or-later
use super::MatrixApp;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use om9_core::{budget::Limits, ring_rail_shape::RailShape};

impl MatrixApp {
    pub(super) fn ring_rail_shape_ui(&mut self, ui: &mut egui::Ui) {
        let names = self.controller.ring_rail_shape_names();
        let previous = self.controller.ring_rail_selected_shape().map(|shape| shape.name()).unwrap_or("Round").to_owned();
        let mut selected = previous.clone();
        let editable = !self.controller.join_history_worker_pending()
            && (self.controller.addon_builder().is_some() || self.controller.builder_editable());
        ui.horizontal_wrapped(|ui| {
            ui.label("Shape");
            ui.add_enabled_ui(editable, |ui| {
                egui::ComboBox::from_id_salt("ring-rail-shape").selected_text(&selected).show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, "Round".into(), "Round");
                    for name in &names { ui.selectable_value(&mut selected, name.clone(), name); }
                });
                if ui.button("Library…").clicked() { self.builder_ui.rail_library_open = true; }
            });
        });
        if selected != previous { let result = self.controller.select_ring_rail_shape(&selected); self.feedback(result); }
        if self.controller.ring_rail_selected_shape().is_some() { ui.small("Size sets the inner circumference; shape scales uniformly."); }
    }

    pub(super) fn ring_rail_library_body(&mut self, ui: &mut egui::Ui) {
        let names = self.controller.ring_rail_shape_names();
        let editable = !self.controller.join_history_worker_pending()
            && (self.controller.addon_builder().is_some() || self.controller.builder_editable());
        let mut add = false;
        let mut save = false;
        let mut choose = None;
        let mut edit = None;
        let mut discard = false;
        ui.scope(|ui| {
                ui.label("Draw a closed curve in the Front view around F4, then select it.");
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.builder_ui.rail_name);
                });
                add = ui.add_enabled(editable && !self.builder_ui.rail_name.trim().is_empty(),
                    egui::Button::new("Add / replace from selected curve")).clicked();
                if let Some(shape) = self.controller.ring_rail_selected_shape()
                    && ui.add_enabled(editable, egui::Button::new("Edit current shape")).clicked() { edit = Some(shape.name().to_owned()); }
                ui.separator();
                if names.is_empty() { ui.label("No user shapes saved yet."); }
                for name in &names {
                    ui.horizontal(|ui| {
                        ui.label(name);
                        if ui.add_enabled(editable, egui::Button::new("Use")).clicked() { choose = Some(name.clone()); }
                        if ui.add_enabled(editable, egui::Button::new("Edit")).clicked() { edit = Some(name.clone()); }
                    });
                }
                if let Some(curve) = self.builder_ui.rail_edit_curve.clone() {
                    ui.separator();
                    ui.label("Drag a control point, or enter its X/Z coordinates in mm.");
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 260.), Sense::hover());
                    let grips = om9_core::curve_ops::curve_grips(&curve, 1e-9, &Limits::default())
                        .map(|grips| grips.poles).unwrap_or_default();
                    let points = om9_core::surface_kernel::sample_curve(&curve, 129, &Limits::default()).unwrap_or_default();
                    let mut changed = shape_canvas(ui, rect, &points, &grips, &mut self.builder_ui.rail_edit_point, editable);
                    if let Some(point) = grips.get(self.builder_ui.rail_edit_point).copied() {
                        let mut x = point[0]; let mut z = point[2];
                        ui.add_enabled_ui(editable, |ui| { ui.horizontal(|ui| {
                            ui.label(format!("Point {}", self.builder_ui.rail_edit_point + 1));
                            let a = ui.add(egui::DragValue::new(&mut x).speed(0.1).prefix("X ").suffix(" mm"));
                            let b = ui.add(egui::DragValue::new(&mut z).speed(0.1).prefix("Z ").suffix(" mm"));
                            if a.changed() || b.changed() { changed = Some((self.builder_ui.rail_edit_point, [x, 0., z])); }
                        }); });
                    }
                    if let Some((index, point)) = changed {
                        match om9_core::ring_rail_shape::move_shape_grip(&curve, index, point, &Limits::default()) {
                            Ok(curve) => { self.builder_ui.rail_edit_curve = Some(curve); self.builder_ui.rail_error = None; }
                            Err(error) => self.builder_ui.rail_error = Some(error.to_string()),
                        }
                    }
                    ui.horizontal(|ui| {
                        save = ui.add_enabled(editable, egui::Button::new("Save shape")).clicked();
                        discard = ui.button("Cancel edit").clicked();
                    });
                }
                if let Some(error) = &self.builder_ui.rail_error { ui.colored_label(Color32::from_rgb(240, 140, 100), error); }
            });
        if add {
            let name = self.builder_ui.rail_name.trim().to_owned();
            let result = self.controller.add_selected_ring_rail_shape(&name)
                .and_then(|_| self.controller.select_ring_rail_shape(&name));
            self.builder_ui.rail_error = result.as_ref().err().map(ToString::to_string);
            self.feedback(result);
        }
        if let Some(name) = choose { let result = self.controller.select_ring_rail_shape(&name); self.feedback(result); }
        if let Some(name) = edit && let Some(shape) = self.controller.ring_rail_library_shape(&name)
            .or_else(|| self.controller.ring_rail_selected_shape().filter(|shape| shape.name() == name)) {
            self.builder_ui.rail_name = name;
            self.builder_ui.rail_edit_curve = Some(shape.curve().clone());
            self.builder_ui.rail_edit_point = 0;
            self.builder_ui.rail_error = None;
        }
        if save && let Some(curve) = self.builder_ui.rail_edit_curve.clone() {
            let name = self.builder_ui.rail_name.trim().to_owned();
            let result = RailShape::new(&name, curve, &Limits::default())
                .and_then(|shape| self.controller.store_ring_rail_shape(shape))
                .and_then(|_| self.controller.select_ring_rail_shape(&name));
            self.builder_ui.rail_error = result.as_ref().err().map(ToString::to_string);
            if result.is_ok() { self.builder_ui.rail_edit_curve = None; }
            self.feedback(result);
        }
        if discard { self.builder_ui.rail_edit_curve = None; self.builder_ui.rail_error = None; }
    }
}

fn shape_canvas(ui: &mut egui::Ui, rect: Rect, points: &[[f64; 3]], grips: &[[f64; 3]],
    selected: &mut usize, editable: bool) -> Option<(usize, [f64; 3])> {
    let painter = ui.painter();
    painter.rect_filled(rect, 3., Color32::from_rgb(14, 20, 25));
    let mut low = [0f64; 2]; let mut high = [0f64; 2];
    for point in points.iter().chain(grips) { for (axis, value) in [point[0], point[2]].into_iter().enumerate() {
        low[axis] = low[axis].min(value); high[axis] = high[axis].max(value);
    }}
    let span = (high[0] - low[0]).max(high[1] - low[1]).max(1.);
    if !span.is_finite() { return None; }
    let middle = [(low[0] + high[0]) * 0.5, (low[1] + high[1]) * 0.5];
    let edge = rect.width().min(rect.height()) - 30.;
    let project = |point: [f64; 3]| Pos2::new(rect.center().x + ((point[0] - middle[0]) / span) as f32 * edge,
        rect.center().y - ((point[2] - middle[1]) / span) as f32 * edge);
    let origin = project([0.; 3]);
    painter.line_segment([origin - Vec2::new(5.,0.), origin + Vec2::new(5.,0.)], Stroke::new(1., Color32::GOLD));
    painter.line_segment([origin - Vec2::new(0.,5.), origin + Vec2::new(0.,5.)], Stroke::new(1., Color32::GOLD));
    for pair in points.windows(2) { painter.line_segment([project(pair[0]), project(pair[1])], Stroke::new(1.5, Color32::from_rgb(64,200,255))); }
    let mut changed = None;
    for (index, point) in grips.iter().enumerate() {
        let position = project(*point);
        painter.circle_filled(position, 3.5, if index == *selected { Color32::GOLD } else { Color32::WHITE });
        let response = ui.interact(Rect::from_center_size(position, Vec2::splat(12.)), egui::Id::new(("ring-shape-point", index)),
            if editable { Sense::click_and_drag() } else { Sense::hover() });
        if response.clicked() || response.dragged() { *selected = index; }
        if response.dragged() && let Some(position) = response.interact_pointer_pos() {
            changed = Some((index, [middle[0] + ((position.x - rect.center().x) / edge) as f64 * span, 0.,
                middle[1] - ((position.y - rect.center().y) / edge) as f64 * span]));
        }
    }
    changed
}
