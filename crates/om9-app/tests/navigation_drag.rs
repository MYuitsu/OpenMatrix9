// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-02-001/002/004`: right-drag ownership and orbit/pan semantics.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::session::InvocationOrigin;
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
        Self::from_document(document)
    }
    fn from_document(document: Document) -> Self {
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

#[test]
fn right_click_finishes_polyline_but_right_drag_does_not() {
    let mut h = Harness::new();
    h.app
        .controller
        .dispatch("OM9-CURVE-001", InvocationOrigin::Menu)
        .unwrap();
    h.app.controller.submit_effect("0,0,0").unwrap();
    h.app.controller.submit_effect("10,0,0").unwrap();
    assert!(h.app.controller.is_running());
    h.drag(ViewId::Perspective, Vec2::new(80., 20.), false);
    assert!(h.app.controller.is_running());
    let pos = h.app.view_rects[&ViewId::Perspective].center();
    h.frame(vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Secondary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    ]);
    h.frame(vec![Event::PointerButton {
        pos,
        button: PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }]);
    assert!(!h.app.controller.is_running());
    assert_eq!(h.app.controller.document().entities.len(), 2);
}

#[test]
fn typed_curve_right_click_rejects_insufficient_points_then_commits() {
    let mut h = Harness::new();
    h.app.controller.submit_effect("Curve").unwrap();
    assert_eq!(h.app.controller.running_feature(), Some("OM9-CURVE-042"));
    let pos = h.app.view_rects[&ViewId::Perspective].center();
    let click = |h: &mut Harness| {
        h.frame(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Secondary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        h.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Secondary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]);
    };
    click(&mut h);
    assert!(h.app.controller.is_running());
    assert_eq!(h.app.controller.document().entities.len(), 1);
    for point in [[0., 0., 0.], [5., 5., 0.], [10., 0., 0.], [15., 5., 0.]] {
        h.app.controller.accept_control_curve_point(point).unwrap();
    }
    click(&mut h);
    assert!(!h.app.controller.is_running());
    assert_eq!(h.app.controller.document().entities.len(), 2);
}

#[cfg(feature = "occt-brep")]
#[test]
fn curve_menu_pending_commands_route_right_click_like_empty_enter() {
    use om9_app::{curve_menu, session::Controller};
    use om9_core::budget::Limits;
    let mut doc = Document::new("curve finish equivalence");
    let mut ids = Vec::new();
    for y in [0., 5., 10.] {
        let curve = om9_core::curve_build::polyline(&[[0., y, 0.], [10., y, 0.]]).unwrap();
        let entity = Entity::new("curve", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    for point in [[0., 0., 1.], [5., 5., 1.], [10., 0., 1.], [15., 5., 1.]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let surface = om9_core::surface_kernel::plane(
        [-20., -20., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        40.,
        40.,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("surface", Geometry::Surface(surface), None);
    ids.push(entity.id);
    doc.entities.insert(entity.id, entity);
    let selections: &[&[usize]] = &[
        &[],
        &[0],
        &[0, 1],
        &[0, 1, 2],
        &[3, 4],
        &[3, 4, 5, 6],
        &[7],
        &[0, 7],
        &[0, 1, 7],
    ];
    let mut checked = Vec::new();
    for binding in curve_menu::BINDINGS {
        let selected = selections.iter().find_map(|selection| {
            let selected: Vec<_> = selection.iter().map(|&i| ids[i]).collect();
            let mut probe = Controller::new(doc.clone(), Limits::default()).unwrap();
            probe.select_objects(&selected).unwrap();
            (probe
                .dispatch(binding.feature, InvocationOrigin::Menu)
                .is_ok()
                && probe.is_running())
            .then_some(selected)
        });
        let Some(selected) = selected else { continue };
        let mut enter = Harness::from_document(doc.clone());
        let mut click = Harness::from_document(doc.clone());
        for h in [&mut enter, &mut click] {
            h.app.controller.select_objects(&selected).unwrap();
            h.app
                .controller
                .dispatch(binding.feature, InvocationOrigin::Menu)
                .unwrap();
        }
        let _ = enter.app.controller.submit_effect("");
        let pos = click.app.view_rects[&ViewId::Perspective].center();
        click.frame(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Secondary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        click.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Secondary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]);
        assert_eq!(
            click.app.controller.is_running(),
            enter.app.controller.is_running(),
            "{}",
            binding.label
        );
        assert_eq!(
            click.app.controller.prompt(),
            enter.app.controller.prompt(),
            "{}",
            binding.label
        );
        let geometries = |h: &Harness| {
            let mut values: Vec<_> = h
                .app
                .controller
                .document()
                .entities
                .values()
                .map(|entity| format!("{}:{:?}", entity.label, entity.geometry))
                .collect();
            values.sort();
            values
        };
        assert_eq!(geometries(&click), geometries(&enter), "{}", binding.label);
        checked.push(binding.label);
    }
    eprintln!("checked {}: {checked:?}", checked.len());
    assert!(checked.len() >= 40);
}
