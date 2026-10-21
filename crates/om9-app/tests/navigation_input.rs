// SPDX-License-Identifier: LGPL-2.1-or-later
//! Real egui event path for the new wheel transition and cursor pivot.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::camera::Camera;
use om9_scene::workspace::ViewId;

fn radius(camera: &Camera) -> f64 {
    ((camera.eye[0] - camera.target[0]).powi(2)
        + (camera.eye[1] - camera.target[1]).powi(2)
        + (camera.eye[2] - camera.target[2]).powi(2))
    .sqrt()
}

#[test]
fn a_wheel_burst_animates_and_zooms_toward_the_cursor() {
    let mut app = MatrixApp::new(Document::new("wheel")).expect("app");
    let context = egui::Context::default();
    fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
            events,
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| app.show(ui));
        output.textures_delta.clear();
    }

    frame(&mut app, &context, vec![]);
    let rect = app.view_rects[&ViewId::Perspective];
    let cursor = rect.center() + Vec2::new(120.0, 80.0);
    let initial = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    let initial_radius = radius(&initial);
    let initial_target = initial.target;

    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    for _ in 0..8 {
        frame(&mut app, &context, vec![Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(0.0, 120.0),
            modifiers: Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        }]);
        frame(&mut app, &context, vec![]);
    }
    // Let the transition finish.
    for _ in 0..8 {
        frame(&mut app, &context, vec![]);
    }

    let after = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    assert!(after.revision > initial.revision);
    assert!(
        radius(&after) < initial_radius * 0.999,
        "wheel must zoom in: {} -> {}",
        initial_radius,
        radius(&after)
    );
    assert!(
        after.target != initial_target,
        "zoom-to-cursor must move the pivot"
    );
}
