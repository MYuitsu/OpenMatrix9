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

    pub(super) fn select_control_grip_at(
        &mut self,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        screen: [f64; 2],
        additive: bool,
    ) -> bool {
        if self.controller.top11_grip_mode() != Some(om9_core::top11::GripMode::Control) {
            return false;
        }
        let revision = self.controller.document().revision;
        self.selected_grips = self.controller.selected_grips().into_iter().map(|h| (h, revision)).collect();
        let Some(handle) = self.grip_at(view, viewport, screen) else {
            if !additive {
                self.selected_grips.clear();
                self.controller.select_grips(&[], crate::session::GripPick::Replace);
            }
            return false;
        };
        if !additive {
            self.selected_grips.clear();
        }
        if let Some(index) = self
            .selected_grips
            .iter()
            .position(|(selected, _)| same_grip(*selected, handle))
        {
            self.selected_grips.remove(index);
        } else {
            self.selected_grips.push((handle, revision));
        }
        let handles: Vec<_> = self.selected_grips.iter().map(|(handle, _)| *handle).collect();
        self.controller.select_grips(&handles, crate::session::GripPick::Replace);
        true
    }

    /// Consume Delete while grips are active, so it cannot delete the whole object.
    pub(super) fn delete_selected_control_grip(&mut self) -> bool {
        if self.controller.top11_grip_mode().is_none() {
            self.selected_grips.clear();
            self.controller.select_grips(&[], crate::session::GripPick::Replace);
            return false;
        }
        let revision = self.controller.document().revision;
        self.selected_grips.retain(|(_, r)| *r == revision);
        let handles = self.controller.selected_grips();
        if handles.is_empty() {
            return true;
        }
        let result = self.controller.top11_delete_grips(&handles).map(|_| ());
        if result.is_ok() {
            self.selected_grips.clear();
            self.controller.select_grips(&[], crate::session::GripPick::Replace);
        }
        *self.grip_cache.borrow_mut() = None;
        self.feedback(result);
        true
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
        let picked_colour = Color32::from_rgb(255, 230, 0);
        let gumball = self.controller.grip_preview();
        let drag_offset = self.grip_drag.as_ref().map(|drag| {
            [
                drag.current[0] - drag.handle.point[0],
                drag.current[1] - drag.handle.point[1],
                drag.current[2] - drag.handle.point[2],
            ]
        });
        let project = |point: Point3| -> Option<egui::Pos2> {
            let projected = view.camera.project(point, viewport).ok()??;
            Some(egui::pos2(
                projected.screen[0] as f32,
                projected.screen[1] as f32,
            ))
        };
        // Preview of the edited curves while grips move.
        let preview = if let (Some(drag), Some(offset)) = (self.grip_drag.as_ref(), drag_offset) {
            let moves: Vec<_> = drag
                .group
                .iter()
                .map(|grip| (*grip, add(grip.point, offset)))
                .collect();
            self.controller.grip_edit_preview(&moves)
        } else if gumball.is_some() {
            self.controller.grip_preview_curves()
        } else {
            Vec::new()
        };
        for curve in &preview {
            let points: Vec<egui::Pos2> = curve.iter().filter_map(|point| project(*point)).collect();
            if points.len() >= 2 {
                painter.add(egui::Shape::line(points, Stroke::new(1.0, picked_colour)));
            }
        }
        for handle in handles.iter() {
            let picked = self.controller.grip_is_selected(handle);
            let in_drag = self
                .grip_drag
                .as_ref()
                .is_some_and(|drag| drag.group.iter().any(|grip| same_grip(*grip, *handle)));
            let point = match (in_drag, drag_offset, gumball) {
                (true, Some(offset), _) => add(handle.point, offset),
                (false, _, Some(delta)) if picked => {
                    delta.point(handle.point).unwrap_or(handle.point)
                }
                _ => handle.point,
            };
            let Some(centre) = project(point) else {
                continue;
            };
            let (fill, size) = if picked || in_drag {
                (picked_colour, 7.0)
            } else {
                (colour, 6.0)
            };
            let square = Rect::from_center_size(centre, egui::vec2(size, size));
            painter.rect_filled(square, 0.0, fill);
            painter.rect_stroke(
                square,
                0.0,
                egui::Stroke::new(1.0, Color32::from_rgb(30, 30, 30)),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// Drag grips in the viewport. Pressing a picked grip drags the whole picked group;
    /// pressing another grip picks it (Shift adds it) and drags it. A release commits one
    /// geometry transaction; Escape cancels the drag through the normal cancel path. The drag
    /// follows object snaps, so a grip can be put on an end, a midpoint or another grip.
    pub(super) fn top11_grip_drag_ui(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
    ) {
        if self.controller.top11_grip_mode().is_none() || self.gumball_session.is_some() {
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
            // A drag can start in a viewport that was not previously active.
            // Its camera rays must use that viewport's own construction plane.
            let plane = view.cplane;
            let plane = CPlane {
                origin: handle.point,
                x: plane.x,
                y: plane.y,
                revision: plane.revision,
            };
            let press_world = match view
                .camera
                .ray([press.x as f64, press.y as f64], viewport)
                .and_then(|ray| ray.intersect_plane(plane))
            {
                Ok(Some(point)) => point,
                _ => {
                    let message =
                        "Grip edit: cannot intersect the view ray with the construction plane";
                    self.status.error(message);
                    self.transcript.append_history(message);
                    return;
                }
            };
            if !self.controller.grip_is_selected(&handle) {
                let (shift, _) = ctrl_or_command(ui);
                self.controller.select_grips(&[handle], if shift { crate::session::GripPick::Add } else { crate::session::GripPick::Replace });
            }
            let group = self.controller.selected_grips();
            self.controller.workspace.active = view.id;
            let Ok(source) = self.controller.document().stamp(handle.object) else {
                return;
            };
            self.grip_drag = Some(GripDrag {
                handle,
                group,
                source,
                epoch: self.controller.coordinator().epoch(),
                selection: self.controller.selection_state().revision,
                view: view.id,
                camera: view.camera.clone(),
                workspace_camera: self.controller.workspace.views[&view.id].camera.clone(),
                maximized: self.controller.workspace.maximized,
                viewport,
                view_plane: view.cplane,
                plane,
                press_world,
                current: handle.point,
            });
        }
        let Some(mut drag) = self.grip_drag.take() else {
            return;
        };
        if drag.view != view.id {
            self.grip_drag = Some(drag);
            return;
        }
        if drag.camera != view.camera || drag.viewport != viewport || drag.view_plane != view.cplane
        {
            self.status.info("Grip edit cancelled: viewport changed");
            return;
        }
        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let world = view
                .camera
                .ray([pointer.x as f64, pointer.y as f64], viewport)
                .and_then(|ray| ray.intersect_plane(drag.plane));
            if let Ok(Some(world)) = world {
                drag.current = self.snap.as_ref().map_or_else(
                    || std::array::from_fn(|i| drag.handle.point[i] + (world[i] - drag.press_world[i])),
                    |candidate| candidate.world,
                );
            }
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            let offset = [
                drag.current[0] - drag.handle.point[0],
                drag.current[1] - drag.handle.point[1],
                drag.current[2] - drag.handle.point[2],
            ];
            self.grip_drag = None;
            if drag.current == drag.handle.point {
                return;
            }
            let moves: Vec<_> = drag
                .group
                .iter()
                .map(|grip| (*grip, add(grip.point, offset)))
                .collect();
            *self.grip_cache.borrow_mut() = None;
            match self.controller.top11_move_grips(&moves) {
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

    /// The grips whose screen points fall inside `band` (window pick of grips).
    pub(super) fn grips_in_band(
        &self,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        band: Rect,
    ) -> Vec<GripHandle> {
        let Some(handles) = self.grip_handles_cached() else {
            return Vec::new();
        };
        handles
            .iter()
            .filter(|handle| {
                view.camera
                    .project(handle.point, viewport)
                    .ok()
                    .flatten()
                    .is_some_and(|projected| {
                        band.contains(egui::pos2(
                            projected.screen[0] as f32,
                            projected.screen[1] as f32,
                        ))
                    })
            })
            .copied()
            .collect()
    }

    pub(super) fn grip_at(
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

fn add(point: Point3, offset: [f64; 3]) -> Point3 {
    [point[0] + offset[0], point[1] + offset[1], point[2] + offset[2]]
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
    #[test]
    fn picked_control_grip_delete_is_undoable_and_never_deletes_object() {
        use om9_core::geometry::{Curve, Geometry, NurbsCurve};
        use om9_core::model::{Document, Entity};
        let n = NurbsCurve {
            degree: 2,
            poles: vec![[0., 0., 0.], [1., 2., 0.], [3., 2., 0.], [4., 0., 0.]],
            weights: vec![1.; 4],
            knots: vec![0., 0., 0., 0.5, 1., 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed: false,
            seam: None,
        };
        let entity = Entity::new("curve", Geometry::Curve(Curve::Nurbs(n)), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("grip delete");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(Top11Action::ControlPoints)
            .unwrap();
        let viewport = Viewport {
            origin: [0., 0.],
            size: [800., 600.],
        };
        let view = app
            .controller
            .workspace
            .views
            .get(&om9_scene::workspace::ViewId::LookingDown)
            .unwrap()
            .clone();
        let handle = app.controller.top11_grip_handles().unwrap()[1];
        let screen = view
            .camera
            .project(handle.point, viewport)
            .unwrap()
            .unwrap()
            .screen;
        assert!(app.select_control_grip_at(&view, viewport, screen, false));
        let before = app.controller.document().clone();
        let context = egui::Context::default();
        let key = || egui::RawInput {
            events: vec![egui::Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut raw = key();
        app.delete_selection_from_input(&context, &mut raw);
        assert!(raw.events.is_empty());
        let Geometry::Curve(Curve::Nurbs(result)) =
            &app.controller.document().entities[&id].geometry
        else {
            panic!("object lost")
        };
        assert_eq!(result.poles.len(), 3);
        let after = app.controller.document().clone();
        let mut raw = key();
        app.delete_selection_from_input(&context, &mut raw);
        assert_eq!(app.controller.document(), &after);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }
    #[test]
    fn shift_selects_multiple_grips_for_one_delete_and_undo() {
        use om9_core::geometry::{Curve, Geometry, NurbsCurve};
        use om9_core::model::{Document, Entity};
        let n = NurbsCurve {
            degree: 2,
            poles: vec![
                [0., 0., 0.],
                [1., 3., 0.],
                [3., 2., 0.],
                [5., 3., 0.],
                [6., 0., 0.],
            ],
            weights: vec![1.; 5],
            knots: vec![0., 0., 0., 0.3, 0.7, 1., 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed: false,
            seam: None,
        };
        let entity = Entity::new("curve", Geometry::Curve(Curve::Nurbs(n)), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("multiple grip delete");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(Top11Action::ControlPoints)
            .unwrap();
        let viewport = Viewport {
            origin: [0., 0.],
            size: [800., 600.],
        };
        let view = app
            .controller
            .workspace
            .views
            .get(&om9_scene::workspace::ViewId::LookingDown)
            .unwrap()
            .clone();
        let handles = app.controller.top11_grip_handles().unwrap();
        let screen = |i: usize| {
            view.camera
                .project(handles[i].point, viewport)
                .unwrap()
                .unwrap()
                .screen
        };
        assert!(app.select_control_grip_at(&view, viewport, screen(1), false));
        assert!(app.select_control_grip_at(&view, viewport, screen(3), true));
        assert_eq!(app.selected_grips.len(), 2);
        let before = app.controller.document().clone();
        let mut raw = egui::RawInput {
            events: vec![egui::Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        app.delete_selection_from_input(&egui::Context::default(), &mut raw);
        let Geometry::Curve(Curve::Nurbs(result)) =
            &app.controller.document().entities[&id].geometry
        else {
            panic!("object lost")
        };
        assert_eq!(result.poles.len(), 3);
        let after = app.controller.document().clone();
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }
    #[test]
    fn selected_surface_control_row_deletes_via_key_and_undo() {
        use om9_core::geometry::{Geometry, NurbsSurface};
        use om9_core::model::{Document, Entity};
        let surface = NurbsSurface {
            degree: [2, 2],
            size: [4, 4],
            poles: (0..4)
                .flat_map(|u| (0..4).map(move |v| [u as f64 * 10., v as f64 * 10., 0.]))
                .collect(),
            weights: vec![1.; 16],
            knots: [
                vec![0., 0., 0., 0.5, 1., 1., 1.],
                vec![0., 0., 0., 0.5, 1., 1., 1.],
            ],
            domain: [[0., 1.], [0., 1.]],
            periodic: [false, false],
            reversed: false,
        };
        let entity = Entity::new("surface", Geometry::Surface(surface), None);
        let id = entity.id;
        let mut doc = Document::with_template_tolerance("surface control delete");
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .top11_invoke(Top11Action::ControlPoints)
            .unwrap();
        let viewport = Viewport {
            origin: [0., 0.],
            size: [1200., 900.],
        };
        let view = app
            .controller
            .workspace
            .views
            .get(&om9_scene::workspace::ViewId::LookingDown)
            .unwrap()
            .clone();
        let handles = app.controller.top11_grip_handles().unwrap();
        for index in 4..8 {
            let screen = view
                .camera
                .project(handles[index].point, viewport)
                .unwrap()
                .unwrap()
                .screen;
            assert!(app.select_control_grip_at(&view, viewport, screen, index != 4));
        }
        assert_eq!(app.selected_grips.len(), 4);
        let before = app.controller.document().clone();
        let mut raw = egui::RawInput {
            events: vec![egui::Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        app.delete_selection_from_input(&egui::Context::default(), &mut raw);
        let Geometry::Surface(result) = &app.controller.document().entities[&id].geometry else {
            panic!("surface lost")
        };
        assert_eq!(result.size, [3, 4]);
        let after = app.controller.document().clone();
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }
}
