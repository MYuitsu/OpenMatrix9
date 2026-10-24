// SPDX-License-Identifier: LGPL-2.1-or-later
//! The opt-in interaction log must record a real camera gesture from the widget path.
//!
//! A right-drag over the Perspective viewport is delivered as egui pointer events, exactly like the
//! native window path; the log then has to contain an `orbit` record with the frame time, the camera
//! and the scene counters. No GPU is needed: the camera input runs before the viewport surface.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::workspace::ViewId;

#[test]
fn a_right_drag_writes_an_orbit_record() {
    let directory =
        std::env::temp_dir().join(format!("om9-interaction-log-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("log directory");
    let path = directory.join("interaction.jsonl");
    // The test binary owns this variable; it is set before the application reads it.
    unsafe { std::env::set_var("OM9_INTERACTION_LOG", &path) };

    let mut app = MatrixApp::new(Document::new("orbit log")).expect("app");
    let context = egui::Context::default();
    let frame = |app: &mut MatrixApp, events: Vec<Event>| {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
            events,
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| app.show(ui));
        output.textures_delta.clear();
        output
    };

    frame(&mut app, vec![]);
    let rect = app.view_rects[&ViewId::Perspective];
    let start = rect.center();
    frame(
        &mut app,
        vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: PointerButton::Secondary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let moved = start + Vec2::new(48., 12.);
    frame(&mut app, vec![Event::PointerMoved(moved)]);
    frame(
        &mut app,
        vec![Event::PointerButton {
            pos: moved,
            button: PointerButton::Secondary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    // A middle drag pans the same view; it must produce its own record.
    frame(
        &mut app,
        vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: PointerButton::Middle,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![Event::PointerMoved(start + Vec2::new(-30., 18.))],
    );
    frame(
        &mut app,
        vec![Event::PointerButton {
            pos: start + Vec2::new(-30., 18.),
            button: PointerButton::Middle,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );

    app.flush_interaction_log();
    let lines = std::fs::read_to_string(&path).expect("interaction log");
    let records: Vec<serde_json::Value> = lines
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect();
    let orbit = records
        .iter()
        .find(|value| value["event"] == "camera" && value["action"] == "orbit")
        .expect("an orbit record after the right drag");
    assert_eq!(orbit["view"], "Perspective");
    assert!(orbit["frame_ms"].as_f64().is_some_and(|ms| ms > 0.0));
    assert!(orbit["revision"].as_u64().is_some());
    assert!(orbit["eye"].is_array());
    assert!(
        records
            .iter()
            .any(|value| value["event"] == "camera" && value["action"] == "pan"),
        "a middle drag must record a pan: {records:?}"
    );

    unsafe { std::env::remove_var("OM9_INTERACTION_LOG") };
    let _ = std::fs::remove_dir_all(&directory);
}
