// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-03-007`: the viewport marker/point path consumes the osnap candidate.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::osnap::SnapKind;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))
}

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn viewport(app: &MatrixApp) -> Viewport {
    let rect = app.view_rects[&ViewId::LookingDown];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let projected = app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .project(point, viewport(app))
        .unwrap()
        .unwrap();
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn key(key: egui::Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) {
    frame(app, context, vec![Event::PointerMoved(at)]);
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
}

#[test]
fn an_end_marker_is_consumed_by_the_point_prompt_and_smarttrack_keeps_two_anchors() {
    let context = egui::Context::default();
    let mut document = Document::new("osnap ui");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;

    app.controller.submit("Line").unwrap();
    let first = screen_of(&app, [0.2, 0., 0.]);
    click(&mut app, &context, first);
    assert_eq!(
        app.snap_candidate().map(|candidate| candidate.kind),
        Some(SnapKind::End),
        "the pointer is close enough to the line end to show END"
    );
    assert_eq!(
        app.snap_candidate().map(|candidate| candidate.tooltip()),
        Some("End point".to_owned())
    );
    assert!(
        app.smarttrack_guides().len() == 2,
        "two CPlane guides per anchor"
    );
    let empty = screen_of(&app, [5., 8., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(empty)]);
    assert!(
        app.snap_candidate().is_none(),
        "no candidate, no stale marker"
    );
    let second = screen_of(&app, [9.8, 0., 0.]);
    click(&mut app, &context, second);
    let Geometry::Curve(Curve::Line { start, end, .. }) = &app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .geometry
    else {
        panic!("line entity");
    };
    assert_eq!(*start, [0., 0., 0.]);
    assert_eq!(*end, [10., 0., 0.]);
    assert_eq!(app.controller.document().entities.len(), 2);
}

#[test]
fn escape_clears_the_smarttrack_guides_and_pending_snap() {
    let context = egui::Context::default();
    let mut document = Document::new("osnap escape");
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
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let first = screen_of(&app, [0.2, 0., 0.]);
    click(&mut app, &context, first);
    assert_eq!(app.smarttrack_guides().len(), 2);
    frame(&mut app, &context, vec![key(egui::Key::Escape)]);
    assert!(app.smarttrack_guides().is_empty());
    assert!(app.snap_candidate().is_none());
    assert!(!app.controller.is_running());
}

#[test]
fn disabling_end_removes_the_marker_from_the_next_frame() {
    let context = egui::Context::default();
    let mut document = Document::new("osnap disabled");
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
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Point").unwrap();
    let at = screen_of(&app, [9.8, 0., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    assert_eq!(app.snap_candidate().map(|c| c.kind), Some(SnapKind::End));
    app.set_snap_enabled(SnapKind::End, false);
    let empty = screen_of(&app, [5., 8., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(empty)]);
    assert!(app.snap_candidate().is_none(), "disabled END has no marker");
    app.set_snap_enabled(SnapKind::End, true);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    assert_eq!(app.snap_candidate().map(|c| c.kind), Some(SnapKind::End));
}
