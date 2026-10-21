// SPDX-License-Identifier: LGPL-2.1-or-later
//! Enter/Space repeat rules for OM9-L3-03-10-006.
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
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

fn press(app: &mut MatrixApp, context: &egui::Context, key: Key) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    frames(app, context, 2);
}

fn app_with_a_repeatable_command() -> (MatrixApp, egui::Context) {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("shortcuts")).expect("app");
    frames(&mut app, &context, 4);
    app.controller.submit("Point").expect("Point");
    app.controller.submit("0,0,0").expect("point");
    assert!(!app.controller.is_running());
    (app, context)
}

#[test]
fn space_repeats_the_allowlisted_command_when_the_editor_is_idle() {
    let (mut app, context) = app_with_a_repeatable_command();
    // The command line is not editing, so Space is a repeat key, not text.
    context.memory_mut(|m| m.surrender_focus(egui::Id::new("matrix-command-transcript")));
    frames(&mut app, &context, 2);
    press(&mut app, &context, Key::Space);
    assert!(
        app.controller.is_running(),
        "Space restarted the repeatable Point command"
    );
    assert_eq!(app.controller.phase(), CmdPhase::AwaitPoint);
    assert!(app.controller.prompt().starts_with("Location of point"));
}

#[test]
fn space_while_editing_the_command_line_does_not_repeat() {
    let (mut app, context) = app_with_a_repeatable_command();
    context.memory_mut(|m| m.request_focus(egui::Id::new("matrix-command-transcript")));
    frames(&mut app, &context, 2);
    press(&mut app, &context, Key::Space);
    assert!(
        !app.controller.is_running(),
        "editing Space types a space, it never repeats"
    );
    assert_eq!(app.controller.phase(), CmdPhase::Idle);
}

#[test]
fn escape_aborts_the_running_command_and_keeps_the_document() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("abort")).expect("app");
    frames(&mut app, &context, 4);
    app.controller.submit("Line").expect("Line");
    frames(&mut app, &context, 2);
    press(&mut app, &context, Key::Escape);
    assert!(!app.controller.is_running(), "Escape aborts the prompt");
    assert_eq!(app.controller.phase(), CmdPhase::Idle);
    assert_eq!(app.controller.prompt(), "Command: ");
    assert!(
        app.controller.document().entities.is_empty(),
        "aborting writes no geometry"
    );
    assert_eq!(
        app.controller.coordinator().undo_len(),
        0,
        "and no history entry"
    );
}
