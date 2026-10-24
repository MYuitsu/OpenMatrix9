// SPDX-License-Identifier: LGPL-2.1-or-later
//! Top 11 toolbar adapter (OM9-L2-03-08): one click path, one status line.
use super::*;
use om9_core::top11::{GripHandle, Top11Action};

impl MatrixApp {
    /// Run the toolbar button whose icon was clicked.
    pub(super) fn top11_action(&mut self, icon: &str) {
        let Some(action) = Top11Action::from_icon(icon) else {
            return;
        };
        match self.controller.top11_invoke(action) {
            Ok(result) => {
                let message = result.message();
                self.status.info(&message);
                self.transcript.append_history(&message);
            }
            Err(error) => {
                let message = format!("{}: {error}", action.label());
                self.transcript.append_history(&message);
                self.status.error(&message);
            }
        }
    }

    /// Return the cached grip handles, recomputing only when the document revision,
    /// selection or grip mode changed. The overlay runs every frame.
    fn grip_handles_cached(&self) -> Option<std::cell::Ref<'_, Vec<GripHandle>>> {
        let mode = self.controller.top11_grip_mode()?;
        let revision = self.controller.document().revision;
        let selection: Vec<_> = self.controller.selected().iter().copied().collect();
        let stale = {
            let cache = self.grip_cache.borrow();
            cache.as_ref().is_none_or(|cache| {
                cache.revision != revision || cache.selection != selection || cache.mode != mode
            })
        };
        if stale {
            let handles = self.controller.top11_grip_handles().ok()?;
            *self.grip_cache.borrow_mut() = Some(GripCache {
                revision,
                selection,
                mode,
                handles,
            });
        }
        Some(std::cell::Ref::map(self.grip_cache.borrow(), |cache| {
            &cache.as_ref().expect("grip cache was just filled").handles
        }))
    }

    /// Draw the active edit/control grips over one viewport, including a drag preview.
    pub(super) fn top11_grip_overlay(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: Rect,
    ) {
        let Some(mode) = self.controller.top11_grip_mode() else {
            return;
        };
        let Some(handles) = self.grip_handles_cached() else {
            return;
        };
        let painter = ui.painter().with_clip_rect(rect);
        let colour = match mode {
            om9_core::top11::GripMode::Edit => Color32::from_rgb(235, 150, 40),
            om9_core::top11::GripMode::Control => Color32::from_rgb(70, 200, 235),
        };
        for handle in handles.iter() {
            let point = self
                .grip_drag
                .as_ref()
                .filter(|drag| same_grip(drag.handle, *handle))
                .map_or(handle.point, |drag| drag.current);
            let Ok(Some(projected)) = view.camera.project(point, viewport) else {
                continue;
            };
            let centre = egui::pos2(projected.screen[0] as f32, projected.screen[1] as f32);
            let square = Rect::from_center_size(centre, egui::vec2(6.0, 6.0));
            painter.rect_filled(square, 0.0, colour);
            painter.rect_stroke(
                square,
                0.0,
                egui::Stroke::new(1.0, Color32::from_rgb(30, 30, 30)),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// Start and update a grip drag in the viewport. A release commits one geometry
    /// transaction; Escape cancels the drag through the normal cancel path.
    pub(super) fn top11_grip_drag_ui(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
    ) {
        if self.controller.top11_grip_mode().is_none() {
            return;
        }
        if self.grip_drag.is_none() && response.drag_started_by(egui::PointerButton::Primary) {
            let Some(press) = ui.input(|input| input.pointer.press_origin()) else {
                return;
            };
            let Some(handle) = self.grip_at(view, viewport, [press.x as f64, press.y as f64])
            else {
                return;
            };
            let plane = self.controller.input_plane();
            self.grip_drag = Some(GripDrag {
                handle,
                plane: CPlane {
                    origin: handle.point,
                    x: plane.x,
                    y: plane.y,
                    revision: plane.revision,
                },
                current: handle.point,
            });
        }
        let Some(mut drag) = self.grip_drag.take() else {
            return;
        };
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let world = view
                .camera
                .ray([pointer.x as f64, pointer.y as f64], viewport)
                .and_then(|ray| ray.intersect_plane(drag.plane));
            if let Ok(Some(world)) = world {
                drag.current = world;
            }
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            let handle = drag.handle;
            let world = drag.current;
            self.grip_drag = None;
            let moved = (0..3)
                .map(|axis| (world[axis] - handle.point[axis]).abs())
                .fold(0.0, f64::max);
            if moved <= 1e-9 {
                return;
            }
            *self.grip_cache.borrow_mut() = None;
            match self.controller.top11_move_grip(handle, world) {
                Ok(result) => {
                    let message = result.message();
                    self.status.info(&message);
                    self.transcript.append_history(&message);
                }
                Err(error) => {
                    let message = format!("Grip edit: {error}");
                    self.status.error(&message);
                    self.transcript.append_history(&message);
                }
            }
        } else {
            self.grip_drag = Some(drag);
        }
    }

    fn grip_at(
        &self,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        screen: [f64; 2],
    ) -> Option<GripHandle> {
        let handles = self.grip_handles_cached()?;
        handles
            .iter()
            .filter_map(|handle| {
                let projected = view.camera.project(handle.point, viewport).ok()??;
                let distance =
                    (projected.screen[0] - screen[0]).hypot(projected.screen[1] - screen[1]);
                (distance <= 7.0).then_some((distance, *handle))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, handle)| handle)
    }
}

fn same_grip(left: GripHandle, right: GripHandle) -> bool {
    left.object == right.object && left.index == right.index && left.mode == right.mode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_top11_icon_is_addressable_from_the_toolbar_name() {
        for action in Top11Action::ALL {
            assert_eq!(Top11Action::from_icon(action.icon()), Some(action));
        }
    }

    #[test]
    fn a_control_grip_is_hit_at_its_projected_screen_point() {
        use crate::session::LINE;
        let mut app = MatrixApp::new(om9_core::model::Document::new("grip hit")).unwrap();
        app.controller.submit(LINE).unwrap();
        app.controller.submit("0,0,0").unwrap();
        app.controller.submit("10,0,0").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(Top11Action::ControlPoints)
            .unwrap();
        let viewport = Viewport {
            origin: [0.0, 0.0],
            size: [800.0, 600.0],
        };
        let view = &app
            .controller
            .workspace
            .views
            .get(&om9_scene::workspace::ViewId::LookingDown)
            .unwrap();
        let handle = app.controller.top11_grip_handles().unwrap()[0];
        let screen = view
            .camera
            .project(handle.point, viewport)
            .unwrap()
            .unwrap();
        let hit = app
            .grip_at(view, viewport, [screen.screen[0], screen.screen[1]])
            .unwrap();
        assert_eq!(hit.object, handle.object);
        assert_eq!(hit.index, handle.index);
    }
}
