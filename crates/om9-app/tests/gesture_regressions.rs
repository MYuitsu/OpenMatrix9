// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent geometry/history oracles replayed through the real viewport widgets.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Curve, Geometry, Transform},
    gumball::{Axis, Handle},
    model::{Document, Entity},
};
use om9_scene::{camera::Viewport, workspace::ViewId};

fn app() -> (MatrixApp, egui::Context, om9_core::identity::ObjectId) {
    let mut d = Document::new("gesture regression");
    let e = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [-5., -3., -2.],
            end: [5., 3., 2.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = e.id;
    d.entities.insert(id, e);
    let mut app = MatrixApp::new(d).unwrap();
    let ctx = egui::Context::default();
    for _ in 0..4 {
        events(&mut app, &ctx, vec![]);
    }
    app.controller.select_objects(&[id]).unwrap();
    (app, ctx, id)
}
fn events(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}
fn viewport(app: &MatrixApp, id: ViewId) -> Viewport {
    let r = app.view_rects[&id];
    Viewport {
        origin: [r.min.x as f64, r.min.y as f64],
        size: [r.width() as f64, r.height() as f64],
    }
}
fn screen(app: &MatrixApp, id: ViewId, p: [f64; 3]) -> Pos2 {
    let s = app.controller.workspace.views[&id]
        .camera
        .project(p, viewport(app, id))
        .unwrap()
        .unwrap()
        .screen;
    Pos2::new(s[0] as f32, s[1] as f32)
}
fn button(app: &mut MatrixApp, ctx: &egui::Context, p: Pos2, pressed: bool) {
    events(
        app,
        ctx,
        vec![
            Event::PointerMoved(p),
            Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ],
    );
}
fn move_start(app: &MatrixApp, view: ViewId, axis: usize) -> [f64; 3] {
    // Orthographic views have a 40 mm height. Perspective is sized at camera target distance.
    let c = &app.controller.workspace.views[&view].camera;
    let span = match c.projection {
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => vertical_span_mm,
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => {
            2. * (vertical_fov_rad / 2.).tan()
                * c.eye
                    .iter()
                    .zip(c.target)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
        }
    };
    let mut p = [0.; 3];
    p[axis] = span / viewport(app, view).size[1] * 90. * 0.55;
    p
}
fn near(a: Transform, b: Transform) {
    for i in 0..4 {
        for j in 0..4 {
            assert!((a.0[i][j] - b.0[i][j]).abs() < 2e-5, "{a:?} != {b:?}");
        }
    }
}
#[test]
fn a_fast_first_sample_and_the_release_position_are_kept_in_every_view() {
    for view in [
        ViewId::LookingDown,
        ViewId::Perspective,
        ViewId::ThroughFinger,
        ViewId::SideView,
    ] {
        let (mut app, ctx, id) = app();
        let axis = if view == ViewId::SideView { 1 } else { 0 };
        app.controller.workspace.active = view;
        let p = move_start(&app, view, axis);
        let mut middle = p;
        middle[axis] += 3.;
        let mut end = p;
        end[axis] += 7.;
        let (a, b, c) = (
            screen(&app, view, p),
            screen(&app, view, middle),
            screen(&app, view, end),
        );
        button(&mut app, &ctx, a, true);
        events(&mut app, &ctx, vec![Event::PointerMoved(b)]);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        assert_eq!(
            app.controller.document().entities[&id].placement,
            Transform::identity()
        );
        let object = &app.controller.scene().unwrap().objects[0];
        assert!(
            (object.display_transform().unwrap().0[axis][3] - 3.).abs() < 2e-5,
            "{view:?}: first preview"
        );
        button(&mut app, &ctx, c, false);
        let mut delta = [0.; 3];
        delta[axis] = 7.;
        near(
            app.controller.document().entities[&id].placement,
            Transform::translation(delta),
        );
        assert_eq!(app.controller.coordinator().undo_len(), 1, "{view:?}");
        app.controller.undo().unwrap();
        near(
            app.controller.document().entities[&id].placement,
            Transform::identity(),
        );
        app.controller.redo().unwrap();
        near(
            app.controller.document().entities[&id].placement,
            Transform::translation(delta),
        );
    }
}
#[test]
fn scaling_is_independent_of_the_number_of_pointer_samples() {
    for samples in [1, 4, 12] {
        let (mut app, ctx, id) = app();
        let view = ViewId::LookingDown;
        let start = app
            .gumball_scale_boxes(view)
            .into_iter()
            .find(|(h, _)| *h == Handle::Scale(Axis::X))
            .unwrap()
            .1;
        let centre = screen(&app, view, [0.; 3]);
        let end = centre + (start - centre) * 2.;
        button(&mut app, &ctx, start, true);
        for i in 1..=samples {
            events(
                &mut app,
                &ctx,
                vec![Event::PointerMoved(
                    start + (end - start) * (i as f32 / samples as f32),
                )],
            );
        }
        button(&mut app, &ctx, end, false);
        let mut expected = Transform::identity();
        expected.0[0][0] = 2.;
        near(app.controller.document().entities[&id].placement, expected);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
    }
}
#[test]
fn losing_selection_cancels_the_preview_and_preserves_old_redo() {
    let (mut app, ctx, id) = app();
    app.controller.numeric_move([1., 0., 0.]).unwrap();
    app.controller.undo().unwrap();
    let view = ViewId::LookingDown;
    let p = move_start(&app, view, 0);
    let a = screen(&app, view, p);
    let b = screen(&app, view, [p[0] + 3., 0., 0.]);
    button(&mut app, &ctx, a, true);
    events(&mut app, &ctx, vec![Event::PointerMoved(b)]);
    app.controller.clear_selection();
    events(&mut app, &ctx, vec![]);
    button(&mut app, &ctx, b, false);
    near(
        app.controller.document().entities[&id].placement,
        Transform::identity(),
    );
    near(
        app.controller.scene().unwrap().objects[0]
            .display_transform()
            .unwrap(),
        Transform::identity(),
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    app.controller.redo().unwrap();
    near(
        app.controller.document().entities[&id].placement,
        Transform::translation([1., 0., 0.]),
    );
}
#[test]
fn pressing_elsewhere_then_crossing_a_handle_does_not_start_a_transform() {
    let (mut app, ctx, id) = app();
    let view = ViewId::LookingDown;
    let r = app.view_rects[&view];
    let a = r.left_top() + Vec2::new(12., 32.);
    let b = screen(&app, view, move_start(&app, view, 0));
    button(&mut app, &ctx, a, true);
    events(&mut app, &ctx, vec![Event::PointerMoved(b)]);
    button(&mut app, &ctx, b, false);
    near(
        app.controller.document().entities[&id].placement,
        Transform::identity(),
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn tapping_alt_during_a_drag_copies_and_tapping_twice_restores_move() {
    for taps in [1, 2] {
        let (mut app, ctx, id) = app();
        let view = ViewId::LookingDown;
        let p = move_start(&app, view, 0);
        let start = screen(&app, view, p);
        let end = screen(&app, view, [p[0] + 3., 0., 0.]);
        button(&mut app, &ctx, start, true);
        events(&mut app, &ctx, vec![Event::PointerMoved(end)]);
        for _ in 0..taps {
            events(
                &mut app,
                &ctx,
                vec![Event::ModifiersChanged(Modifiers::ALT)],
            );
            events(
                &mut app,
                &ctx,
                vec![Event::ModifiersChanged(Modifiers::NONE)],
            );
        }
        button(&mut app, &ctx, end, false);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        if taps == 1 {
            assert_eq!(app.controller.document().entities.len(), 2);
            near(
                app.controller.document().entities[&id].placement,
                Transform::identity(),
            );
            let copied = app
                .controller
                .document()
                .entities
                .values()
                .find(|e| e.id != id)
                .unwrap();
            near(copied.placement, Transform::translation([3., 0., 0.]));
        } else {
            assert_eq!(app.controller.document().entities.len(), 1);
            near(
                app.controller.document().entities[&id].placement,
                Transform::translation([3., 0., 0.]),
            );
        }
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document().entities.len(), 1);
        near(
            app.controller.document().entities[&id].placement,
            Transform::identity(),
        );
    }
}

#[test]
fn losing_window_focus_discards_the_gesture_without_publication() {
    let (mut app, ctx, id) = app();
    let view = ViewId::LookingDown;
    let p = move_start(&app, view, 0);
    let start = screen(&app, view, p);
    let end = screen(&app, view, [p[0] + 3., 0., 0.]);
    button(&mut app, &ctx, start, true);
    events(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    let mut output = ctx.run_ui(
        egui::RawInput {
            focused: false,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
            events: vec![Event::WindowFocused(false)],
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    button(&mut app, &ctx, end, false);
    near(
        app.controller.document().entities[&id].placement,
        Transform::identity(),
    );
    near(
        app.controller.scene().unwrap().objects[0]
            .display_transform()
            .unwrap(),
        Transform::identity(),
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
