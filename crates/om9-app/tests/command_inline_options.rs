// SPDX-License-Identifier: LGPL-2.1-or-later
//! Every command spells its clickable options inline in the prompt, like Polyline.
//!
//! The command bar has two option presentations: labels found inside the prompt are
//! underlined in place, everything else used to fall back to a floating chip row over
//! the transcript. `Controller::prompt_with_options` gives every session the inline
//! signature, so this test pins both the text and the clickable token rect.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::command_options::CommandOptionInterface;
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

/// Commands that start without a selection and publish option tokens.
const STARTERS: [&str; 14] = [
    "InterpCrv",
    "Arc",
    "Circle",
    "Ellipse",
    "Rectangle",
    "Polygon",
    "Spiral",
    "Helix",
    "Divide",
    "ThroughPoints",
    "Curve2Views",
    "Rebuild",
    "FitCrv",
    "CurveSoftEdit",
];

#[test]
fn prompt_with_options_names_every_option_inline_for_each_starter() {
    let mut inline = 0;
    for command in STARTERS {
        let mut controller =
            om9_app::session::Controller::new(Document::new("inline"), Default::default())
                .expect("controller");
        if controller.submit(command).is_err() {
            continue;
        }
        let options = controller.command_options();
        assert!(
            !options.is_empty(),
            "{command} started without publishing options"
        );
        for option in &options {
            assert!(
                option.label.ends_with('=') || !option.label.is_empty(),
                "{command}: empty option label"
            );
        }
        let prompt = controller.prompt_with_options();
        assert!(
            prompt.ends_with("): "),
            "{command}: inline signature missing from {prompt:?}"
        );
        assert!(
            !prompt.contains('\n'),
            "{command}: inline signature must stay on the prompt line"
        );
        let mut search_from = 0;
        for option in &options {
            let at = prompt[search_from..]
                .find(option.label.as_str())
                .unwrap_or_else(|| panic!("{command}: {:?} missing from {prompt:?}", option.label));
            search_from += at + option.label.len();
        }
        assert_ne!(
            prompt,
            controller.prompt(),
            "{command}: the plain prompt should not already carry the options"
        );
        inline += 1;
    }
    assert!(
        inline >= 8,
        "only {inline} starter(s) exercised the inline path; the list needs updating"
    );
}

#[test]
fn polyline_keeps_its_own_signature_without_duplication() {
    let mut controller =
        om9_app::session::Controller::new(Document::new("inline"), Default::default())
            .expect("controller");
    controller.submit("Polyline").expect("start polyline");
    assert_eq!(
        controller.prompt_with_options(),
        controller.prompt(),
        "polyline already names its options; nothing may be appended"
    );
}

#[test]
fn idle_prompt_never_gains_an_option_signature() {
    let controller = om9_app::session::Controller::new(Document::new("inline"), Default::default())
        .expect("controller");
    assert_eq!(controller.prompt_with_options(), "Command: ");
}

#[test]
fn the_command_bar_underlines_interp_options_inside_the_prompt() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("inline tokens")).expect("app");
    frames(&mut app, &context, 4);
    app.controller.submit("InterpCrv").expect("start interp");
    frames(&mut app, &context, 2);
    let prompt = app.controller.prompt_with_options();
    assert!(prompt.contains("Degree="), "{prompt}");
    assert!(prompt.contains("Cancel"), "{prompt}");
    // The inline renderer registers the clickable span, so no chip row is needed.
    let degree = app
        .command_option_token_rect("Degree")
        .expect("Degree is clickable inside the prompt");
    assert!(
        degree.width() > 1.0 && degree.height() > 1.0,
        "inline token rect is degenerate: {degree:?}"
    );
    let cancel = app
        .command_option_token_rect("Cancel")
        .expect("Cancel is clickable inside the prompt");
    assert_ne!(degree, cancel, "each option owns its own span");
}
