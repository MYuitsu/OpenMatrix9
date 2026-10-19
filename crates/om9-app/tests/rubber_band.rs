// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rubber-band selection in the viewport: dragging a rectangle over several objects selects exactly the
//! ones it touches, through the real `MatrixApp::show` input path.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn document_with_points() -> Document {
    let mut document = Document::new("band");
    for (index, x) in [-8.0, 8.0].into_iter().enumerate() {
        let entity = Entity::new(
            if index == 0 { "left" } else { "right" },
            Geometry::Point([x, 0.0, 0.0]),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

fn frames(app: &mut MatrixApp, context: &egui::Context, frames: usize) {
    for _ in 0..frames {
        let mut output = context.run_ui(
            egui::RawInput { screen_rect: Some(screen_rect()), ..Default::default() },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
}

fn events(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput { screen_rect: Some(screen_rect()), events, ..Default::default() },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn viewport_of(app: &MatrixApp, view: ViewId) -> Viewport {
    let rect = app.view_rects[&view];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, view: ViewId, point: [f64; 3]) -> Pos2 {
    let viewport = viewport_of(app, view);
    let camera = &app.controller.workspace.views[&view].camera;
    let projected = camera.project(point, viewport).expect("projection").expect("in front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

/// Presses at `start`, moves to `end` in steps, releases: the pointer path of a rubber band.
fn drag(app: &mut MatrixApp, context: &egui::Context, start: Pos2, end: Pos2) {
    events(app, context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    events(app, context, vec![Event::PointerMoved(start)]);
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        events(app, context, vec![Event::PointerMoved(start + (end - start) * t)]);
    }
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(app, context, 2);
}

#[test]
fn a_band_over_two_points_selects_both() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_points()).expect("app");
    frames(&mut app, &context, 4);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    assert_eq!(ids.len(), 2);
    let left = screen_of(&app, ViewId::Perspective, [-8.0, 0.0, 0.0]);
    let right = screen_of(&app, ViewId::Perspective, [8.0, 0.0, 0.0]);
    eprintln!("left {left:?} right {right:?} rect {:?}", app.view_rects[&ViewId::Perspective]);
    let start = Pos2::new(left.x.min(right.x) - 25.0, left.y.min(right.y) - 25.0);
    let end = Pos2::new(left.x.max(right.x) + 25.0, left.y.max(right.y) + 25.0);
    drag(&mut app, &context, start, end);
    let selected = app.controller.selection_state().objects;
    assert_eq!(selected.len(), 2, "both points are inside the band: {selected:?}");
    for id in &ids {
        assert!(selected.contains(id));
    }
}

#[test]
fn a_band_around_one_point_selects_only_that_one() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_points()).expect("app");
    frames(&mut app, &context, 4);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    let left = screen_of(&app, ViewId::Perspective, [-8.0, 0.0, 0.0]);
    let start = Pos2::new(left.x - 20.0, left.y - 20.0);
    let end = Pos2::new(left.x + 20.0, left.y + 20.0);
    drag(&mut app, &context, start, end);
    let selected = app.controller.selection_state().objects;
    assert_eq!(selected.len(), 1, "only the left point is inside: {selected:?}");
    assert!(!ids.is_empty());
}
