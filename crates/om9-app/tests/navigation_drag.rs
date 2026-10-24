// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-02-001/002/004`: right-drag ownership and orbit/pan semantics.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_scene::camera::{Camera, Projection};
use om9_scene::workspace::ViewId;

struct Harness {
    app: MatrixApp,
    ctx: egui::Context,
    time: f64,
}

impl Harness {
    fn new() -> Self {
        let mut document = Document::new("navigation drag");
        let line = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        let id = line.id;
        document.entities.insert(id, line);
        let mut h = Self {
            app: MatrixApp::new(document).unwrap(),
            ctx: Default::default(),
            time: 0.,
        };
        h.frame(vec![]);
        h.frame(vec![]);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.time += 1. / 30.;
        let mut output = self.ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
                events,
                time: Some(self.time),
                ..Default::default()
            },
            |ui| self.app.show(ui),
        );
        output.textures_delta.clear();
    }
    fn drag(&mut self, view: ViewId, delta: Vec2, shift: bool) {
        self.app.controller.workspace.active = view;
        let start = self.app.view_rects[&view].center();
        let modifiers = Modifiers {
            shift,
            ..Modifiers::NONE
        };
        self.frame(vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: PointerButton::Secondary,
                pressed: true,
                modifiers,
            },
        ]);
        self.frame(vec![Event::PointerMoved(start + delta)]);
        self.frame(vec![Event::PointerMoved(start + delta)]);
        self.frame(vec![Event::PointerButton {
            pos: start + delta,
            button: PointerButton::Secondary,
            pressed: false,
            modifiers,
        }]);
    }
}

fn direction(camera: &Camera) -> [f64; 3] {
    [
        camera.eye[0] - camera.target[0],
        camera.eye[1] - camera.target[1],
        camera.eye[2] - camera.target[2],
    ]
}

fn same(a: [f64; 3], b: [f64; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 1e-9)
}

#[test]
fn perspective_right_drag_orbits_without_moving_target_or_model() {
    let mut h = Harness::new();
    let before = h.app.controller.workspace.views[&ViewId::Perspective]
        .camera
        .clone();
    let entity = h
        .app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .clone();
    h.drag(ViewId::Perspective, Vec2::new(100., 20.), false);
    let after = &h.app.controller.workspace.views[&ViewId::Perspective].camera;
    assert_eq!(after.target, before.target);
    assert!(!same(direction(after), direction(&before)));
    assert_eq!(after.projection, before.projection);
    assert_eq!(
        h.app
            .controller
            .document()
            .entities
            .values()
            .next()
            .unwrap(),
        &entity
    );
}

#[test]
fn orthographic_right_drag_pans_without_changing_zoom_or_model() {
    let mut h = Harness::new();
    let before = h.app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .clone();
    assert!(matches!(before.projection, Projection::Orthographic { .. }));
    let entity = h
        .app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .clone();
    h.drag(ViewId::LookingDown, Vec2::new(80., 30.), false);
    let after = &h.app.controller.workspace.views[&ViewId::LookingDown].camera;
    assert!(!same(after.target, before.target));
    assert!(same(direction(after), direction(&before)));
    let Projection::Orthographic {
        vertical_span_mm: before_span,
    } = before.projection
    else {
        panic!("ortho");
    };
    let Projection::Orthographic {
        vertical_span_mm: after_span,
    } = after.projection
    else {
        panic!("ortho");
    };
    assert_eq!(after_span, before_span);
    assert_eq!(
        h.app
            .controller
            .document()
            .entities
            .values()
            .next()
            .unwrap(),
        &entity
    );
}

#[test]
fn shift_right_drag_pans_perspective_without_rotating_it() {
    let mut h = Harness::new();
    let before = h.app.controller.workspace.views[&ViewId::Perspective]
        .camera
        .clone();
    h.drag(ViewId::Perspective, Vec2::new(-70., 40.), true);
    let after = &h.app.controller.workspace.views[&ViewId::Perspective].camera;
    assert!(!same(after.target, before.target));
    assert!(same(direction(after), direction(&before)));
    assert_eq!(after.projection, before.projection);
}
