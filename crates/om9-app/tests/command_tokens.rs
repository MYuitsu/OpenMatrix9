// SPDX-License-Identifier: LGPL-2.1-or-later
//! Option tokens and shortcut keys in the command line (OM9-L3-03-10-003 and -006).
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::session::CmdPhase;
use om9_app::ui::MatrixApp;
use om9_core::model::Document;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn frames(app: &mut MatrixApp, context: &egui::Context, count: usize) {
    for _ in 0..count {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                events: vec![Event::ModifiersChanged(Modifiers::NONE)],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) {
    for event in [
        Event::PointerMoved(at),
        Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ] {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                events: vec![event],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    frames(app, context, 2);
}

#[test]
fn clicking_the_both_sides_token_toggles_through_the_typed_option_path() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("tokens")).expect("app");
    frames(&mut app, &context, 4);
    app.controller
        .submit("Line")
        .expect("start the line command");
    frames(&mut app, &context, 2);
    let token = app
        .command_option_token_rect("BothSides")
        .expect("the declared option is a clickable token");
    click(&mut app, &context, token.center());
    assert_eq!(
        app.controller.both_sides(),
        Some(true),
        "the click set the option"
    );
    assert!(
        app.controller.prompt().contains("BothSides=Yes"),
        "{}",
        app.controller.prompt()
    );
    assert_eq!(app.controller.phase(), CmdPhase::AwaitPoint);
    // Clicking again toggles back and cannot commit the command twice.
    let token = app
        .command_option_token_rect("BothSides")
        .expect("token still there");
    click(&mut app, &context, token.center());
    assert_eq!(app.controller.both_sides(), Some(false));
    assert!(
        app.controller.document().entities.is_empty(),
        "a token click never commits geometry"
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
