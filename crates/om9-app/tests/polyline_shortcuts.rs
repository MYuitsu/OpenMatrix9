// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Rect, Vec2};
use om9_app::session::InvocationOrigin;
use om9_app::ui::MatrixApp;
use om9_core::model::Document;

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1280., 800.),
            )),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
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

#[test]
fn underlined_polyline_shortcut_toggles_without_writing_to_draft() {
    let mut app = MatrixApp::new(Document::new("shortcuts")).unwrap();
    let ctx = egui::Context::default();
    app.controller
        .dispatch("OM9-CURVE-001", InvocationOrigin::Menu)
        .unwrap();
    frame(&mut app, &ctx, Vec::new());
    frame(&mut app, &ctx, vec![key(Key::P), Event::Text("P".into())]);
    assert!(
        app.controller.prompt().contains("Close=Yes"),
        "prompt: {}",
        app.controller.prompt()
    );
    assert_eq!(app.transcript.draft(), "");
    frame(&mut app, &ctx, vec![key(Key::H), Event::Text("H".into())]);
    assert!(
        app.controller
            .polyline_options()
            .is_some_and(|(_, helpers, _)| helpers)
    );
    assert_eq!(app.transcript.draft(), "");
}
