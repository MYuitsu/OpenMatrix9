// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-02-005`: F4 is an origin shortcut only while a point is requested.
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{geometry::Geometry, model::Document};

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn points(app: &MatrixApp) -> Vec<[f64; 3]> {
    app.controller
        .document()
        .entities
        .values()
        .filter_map(|entity| match &entity.geometry {
            Geometry::Point(point) => Some(*point),
            _ => None,
        })
        .collect()
}

#[test]
fn f4_inserts_world_origin_in_a_point_prompt_and_only_toggles_the_datum_when_idle() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("f4 origin")).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    assert!(app.world_datum_visible());
    app.controller.submit("Point").unwrap();

    frame(&mut app, &context, vec![key(Key::F4)]);
    assert_eq!(points(&app), vec![[0.; 3]]);
    assert!(
        app.world_datum_visible(),
        "the F4 datum toggle must not run while a point prompt owns the key"
    );

    frame(&mut app, &context, vec![key(Key::F4)]);
    assert_eq!(points(&app), vec![[0.; 3]], "idle F4 creates no object");
    assert!(!app.world_datum_visible(), "idle F4 keeps the datum toggle");
}
