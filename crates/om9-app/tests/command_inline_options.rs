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
fn command_suggestions_drop_below_the_command_row() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("suggestions")).expect("app");
    frames(&mut app, &context, 3);
    app.transcript.set_draft("box");
    app.focus_command_editor();
    frames(&mut app, &context, 2);
    let input = app.command_input_rect().expect("command row");
    let list = app.command_suggestions_rect().expect("suggestion list");
    assert!(
        list.top() >= input.bottom(),
        "the suggestion list must drop below the command row: row={input:?} list={list:?}"
    );
    assert!(
        (list.left() - input.left()).abs() < 1.0,
        "the list stays aligned with the row: row={input:?} list={list:?}"
    );
}

#[test]
fn a_command_started_outside_the_command_line_still_names_its_options_inline() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("inline start")).expect("app");
    frames(&mut app, &context, 4);
    // Toolbar/top11/MCP starts do not pass through `feedback`, so the command
    // bar itself must adopt the controller prompt each frame.
    app.controller.submit("Box").expect("start box");
    frames(&mut app, &context, 2);
    let prompt = app.controller.prompt_with_options();
    assert!(prompt.contains("Diagonal"), "{prompt}");
    assert!(
        app.transcript.text().contains("Diagonal"),
        "live row kept a stale prompt: {:?}",
        app.transcript.text()
    );
    let row = app.command_input_rect().expect("command row");
    for name in ["Diagonal", "3Point", "Vertical", "Center"] {
        let token = app
            .command_option_token_rect(name)
            .unwrap_or_else(|| panic!("{name} is clickable inside the prompt"));
        assert!(
            token.top() >= row.top() - 1.0 && token.bottom() <= row.bottom() + 1.0,
            "{name} must stay on the command row, not float above it: row={row:?} token={token:?}"
        );
    }
    // The clicked inline word drives the owner, and the next prompt stays inline.
    let center = app
        .command_option_token_rect("Center")
        .expect("Center token")
        .center();
    for event in [
        egui::Event::PointerMoved(center),
        egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        egui::Event::PointerButton {
            pos: center,
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
    frames(&mut app, &context, 1);
    assert!(
        app.controller.prompt().contains("Center of base"),
        "clicked Center did not reach the owner: {:?}",
        app.controller.prompt()
    );
}

#[test]
fn duplicate_options_stay_inline_when_started_from_the_controller() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("inline copy")).expect("app");
    frames(&mut app, &context, 3);
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.controller.submit("10,0,0").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("Copy").expect("start copy");
    frames(&mut app, &context, 2);
    let row = app.command_input_rect().expect("command row");
    for name in ["InPlace", "Vertical"] {
        let token = app
            .command_option_token_rect(name)
            .unwrap_or_else(|| panic!("{name} is clickable inside the prompt"));
        assert!(
            token.top() >= row.top() - 1.0 && token.bottom() <= row.bottom() + 1.0,
            "{name} must stay on the command row, not float above it: row={row:?} token={token:?}"
        );
    }
    // Matrix reveals FromLastPoint/UseLast* only after the first copy is placed.
    app.controller
        .accept_world_point([0., 0., 0.])
        .expect("base point");
    frames(&mut app, &context, 1);
    assert!(
        app.command_option_token_rect("FromLastPoint").is_none(),
        "FromLastPoint appeared before the first copy"
    );
    app.controller
        .accept_world_point([5., 0., 0.])
        .expect("first copy");
    frames(&mut app, &context, 2);
    for name in ["FromLastPoint", "UseLastDistance", "UseLastDirection"] {
        let token = app
            .command_option_token_rect(name)
            .unwrap_or_else(|| panic!("{name} is clickable after the first copy"));
        assert!(
            token.top() >= row.top() - 1.0 && token.bottom() <= row.bottom() + 1.0,
            "{name} must stay on the command row: row={row:?} token={token:?}"
        );
    }
}

#[test]
fn toolbar_top11_start_keeps_its_options_inline() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("inline top11")).expect("app");
    frames(&mut app, &context, 3);
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.controller.submit("10,0,0").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    // The toolbar path never calls `feedback`, so this is the case that used to
    // leave a floating option row over the transcript.
    app.controller
        .top11_invoke(om9_core::top11::Top11Action::Duplicate)
        .expect("duplicate");
    frames(&mut app, &context, 2);
    assert!(
        app.transcript.text().contains("InPlace"),
        "stale live row: {:?}",
        app.transcript.text()
    );
    let row = app.command_input_rect().expect("command row");
    for name in ["InPlace", "Vertical"] {
        let token = app
            .command_option_token_rect(name)
            .unwrap_or_else(|| panic!("{name} is clickable inside the prompt"));
        assert!(
            token.top() >= row.top() - 1.0 && token.bottom() <= row.bottom() + 1.0,
            "{name} must stay on the command row: row={row:?} token={token:?}"
        );
    }
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

/// Every catalogued command that starts headlessly must name all of its options in the
/// prompt. The command bar has no second presentation left, so this is the executable
/// form of "inline like Polyline, in every command".
#[test]
fn every_catalogued_command_names_its_options_inline() {
    let catalog =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/features.json"))
            .expect("features catalog");
    let rows: Vec<serde_json::Value> = serde_json::from_str(&catalog).expect("catalog json");
    let mut started = 0;
    let mut checked = 0;
    let mut panicked = Vec::new();
    for row in &rows {
        let Some(command) = row["command"].as_str() else {
            continue;
        };
        if command.trim().is_empty() {
            continue;
        }
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut controller =
                om9_app::session::Controller::new(Document::new("catalog"), Default::default())
                    .ok()?;
            controller.submit(command).ok()?;
            let options = controller.command_options();
            if options.is_empty() {
                controller.cancel();
                return Some(None);
            }
            let prompt = controller.prompt_with_options();
            let labels: Vec<String> = options.iter().map(|o| o.label.clone()).collect();
            controller.cancel();
            Some(Some((prompt, labels)))
        }));
        let Ok(Some(case)) = attempt else {
            if attempt.is_err() {
                panicked.push(command.to_owned());
            }
            continue;
        };
        started += 1;
        let Some((prompt, labels)) = case else {
            continue;
        };
        checked += 1;
        assert!(
            !prompt.contains('\n'),
            "{command}: inline signature must stay on the prompt line: {prompt:?}"
        );
        assert!(
            prompt.trim_end().ends_with("):"),
            "{command}: inline signature must close the prompt: {prompt:?}"
        );
        let mut search_from = 0;
        for label in &labels {
            let at = prompt[search_from..]
                .find(label.as_str())
                .unwrap_or_else(|| panic!("{command}: {label:?} missing from {prompt:?}"));
            search_from += at + label.len();
        }
    }
    assert!(
        started >= 40,
        "only {started} catalogued commands started; the sweep needs updating"
    );
    assert!(
        checked >= 25,
        "only {checked} catalogued commands published options; the sweep needs updating"
    );
    assert!(
        panicked.len() <= 2,
        "starting catalogued commands panicked: {panicked:?}"
    );
    eprintln!("inline sweep: {started} commands started, {checked} published inline options");
}
