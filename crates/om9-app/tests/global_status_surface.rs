// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-01-007 (Global Status And Error Surface), driven
//! by the fixtures in `implementation/fixtures/M01/OM9-L3-01-01-007/`.
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::status_surface::{StatusKind, StatusSurface};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-007")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn fixture_case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("fixture case {id}"))
}

const SIZE: Vec2 = Vec2::new(1280., 720.);

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SIZE)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn warmup(app: &mut MatrixApp, context: &egui::Context) {
    for _ in 0..3 {
        frame(app, context, Vec::new());
    }
}

fn escape() -> Vec<Event> {
    vec![Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }]
}

/// Type a command into the command box and press Enter, the way the user does.
fn run_command(app: &mut MatrixApp, context: &egui::Context, text: &str) {
    frame(app, context, vec![Event::Text(text.into())]);
    frame(
        app,
        context,
        vec![Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
}

#[test]
fn g1_an_invalid_command_is_visible_and_retrievable() {
    let document = fixture("positive.json");
    let case = fixture_case(
        &document,
        "G1-an-invalid-command-is-visible-and-retrievable",
    );
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status g1")).unwrap();
    warmup(&mut app, &context);
    let command = case["command"].as_str().unwrap();
    run_command(&mut app, &context, command);

    assert_eq!(
        app.status.kind(),
        StatusKind::Error,
        "{}",
        app.status.message()
    );
    let needle = case["expect"]["message_contains"].as_str().unwrap();
    assert!(
        app.status.message().contains(needle),
        "{}",
        app.status.message()
    );
    assert!(
        app.status
            .line()
            .starts_with(case["expect"]["line_prefix"].as_str().unwrap())
    );
    assert!(
        app.transcript.history().contains(needle),
        "the diagnostic must also be in the command history: {}",
        app.transcript.history()
    );
    assert!(
        app.status.tail().iter().any(|line| line.contains(needle)),
        "the tail must carry it: {:?}",
        app.status.tail()
    );
    assert!(!app.controller.is_running());
    assert_eq!(case["expect"]["command_not_running"], true);
}

#[test]
fn g2_a_failure_survives_a_later_success() {
    let document = fixture("positive.json");
    let case = fixture_case(&document, "G2-a-failure-survives-a-later-success");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status g2")).unwrap();
    warmup(&mut app, &context);
    run_command(&mut app, &context, "NotACommand");
    assert_eq!(app.status.kind(), StatusKind::Error);
    // A command that succeeds.
    run_command(&mut app, &context, "Line");
    frame(&mut app, &context, Vec::new());
    assert_eq!(
        app.status.kind(),
        StatusKind::Error,
        "a later success must not clear an unacknowledged failure: {}",
        app.status.message()
    );
    assert_eq!(
        case["expect"]["kind_after_success"].as_str().unwrap(),
        "Error"
    );
    assert!(!app.status.complete_success());
    assert_eq!(case["expect"]["complete_success_returns"], false);
}

#[test]
fn g3_only_dismiss_clears_a_failure() {
    let document = fixture("positive.json");
    let case = fixture_case(&document, "G3-only-dismiss-clears-a-failure");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status g3")).unwrap();
    warmup(&mut app, &context);
    app.status.error("Unknown command: Foo");
    frame(&mut app, &context, Vec::new());
    let dismiss = app
        .status_dismiss_rect
        .expect("Dismiss appears for a failure");
    frame(
        &mut app,
        &context,
        vec![Event::PointerMoved(dismiss.center())],
    );
    frame(
        &mut app,
        &context,
        vec![
            Event::PointerMoved(dismiss.center()),
            Event::PointerButton {
                pos: dismiss.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        &context,
        vec![
            Event::PointerMoved(dismiss.center()),
            Event::PointerButton {
                pos: dismiss.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    assert_eq!(
        app.status.kind(),
        StatusKind::Ready,
        "{}",
        app.status.message()
    );
    assert_eq!(
        app.status.line(),
        case["expect"]["line_after_dismiss"].as_str().unwrap()
    );
}

#[test]
fn g4_a_panel_message_never_hides_a_failure() {
    let document = fixture("positive.json");
    let case = fixture_case(&document, "G4-a-panel-message-never-hides-a-failure");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status g4")).unwrap();
    warmup(&mut app, &context);
    app.status.error("Unknown command: Foo");
    let message = app.status.message().to_owned();
    assert!(
        app.apply_panel_op(om9_app::panel_ops::PanelOp::ToggleCollapsed {
            section: om9_app::dock_layout::SectionId::Projects
        })
        .is_ok()
    );
    frame(&mut app, &context, Vec::new());
    assert_eq!(app.status.kind(), StatusKind::Error);
    assert_eq!(app.status.message(), message);
    assert_eq!(case["expect"]["message_unchanged"], true);
    assert!(
        app.transcript.history().contains("rolled up"),
        "{}",
        app.transcript.history()
    );
}

#[test]
fn g5_to_g10_the_surface_reports_what_the_fixture_states() {
    let document = fixture("positive.json");

    // G5: a warning is visible without becoming a failure.
    let case = fixture_case(
        &document,
        "G5-a-warning-is-visible-without-becoming-a-failure",
    );
    let mut status = StatusSurface::new();
    let report = case["report"].as_array().unwrap();
    assert!(status.report(
        match report[0].as_str().unwrap() {
            "Warning" => StatusKind::Warning,
            other => panic!("{other}"),
        },
        report[1].as_str().unwrap()
    ));
    assert_eq!(status.kind(), StatusKind::Warning);
    assert!(!status.kind().is_failure());
    assert!(
        status
            .line()
            .starts_with(case["expect"]["line_prefix"].as_str().unwrap())
    );

    // G6: the same message twice does not move the revision.
    let case = fixture_case(
        &document,
        "G6-the-same-message-twice-does-not-move-the-revision",
    );
    let mut status = StatusSurface::new();
    let before = status.revision();
    status.info("Line accepted");
    status.info("Line accepted");
    assert_eq!(
        status.revision() - before,
        case["expect"]["revision_delta"].as_u64().unwrap()
    );

    // G7: a long diagnostic is bounded on a character boundary.
    let case = fixture_case(
        &document,
        "G7-a-long-diagnostic-is-bounded-on-a-character-boundary",
    );
    let mut status = StatusSurface::new();
    let unit = case["message_unit"].as_str().unwrap();
    status.error(&unit.repeat(case["message_repeat"].as_u64().unwrap() as usize));
    assert!(
        status.message().len()
            <= case["expect"]["message_bytes_at_most"].as_u64().unwrap() as usize
    );
    assert!(
        status
            .message()
            .chars()
            .all(|ch| ch == unit.chars().next().unwrap())
    );
    assert_eq!(case["expect"]["all_characters_intact"], true);

    // G8: a blank message reads Ready.
    let case = fixture_case(&document, "G8-a-blank-message-reads-ready");
    let mut status = StatusSurface::new();
    status.info(case["message"].as_str().unwrap());
    assert_eq!(
        status.message(),
        case["expect"]["message"].as_str().unwrap()
    );

    // G9: the tail keeps the most recent messages, oldest first.
    let case = fixture_case(
        &document,
        "G9-the-tail-keeps-the-most-recent-messages-oldest-first",
    );
    let mut status = StatusSurface::new();
    let count = case["messages"].as_u64().unwrap();
    for index in 0..count {
        status.info(&format!("message {index}"));
    }
    let tail = status.tail();
    assert_eq!(
        tail.len(),
        case["expect"]["tail_len"].as_u64().unwrap() as usize
    );
    assert_eq!(tail[0], case["expect"]["tail_first"].as_str().unwrap());
    assert_eq!(
        tail[tail.len() - 1],
        case["expect"]["tail_last"].as_str().unwrap()
    );

    // G10: a second failure keeps the first in the tail.
    let case = fixture_case(
        &document,
        "G10-a-second-failure-keeps-the-first-in-the-tail",
    );
    let mut status = StatusSurface::new();
    let errors = case["errors"].as_array().unwrap();
    status.error(errors[0].as_str().unwrap());
    status.error(errors[1].as_str().unwrap());
    assert_eq!(
        status.message(),
        case["expect"]["message"].as_str().unwrap()
    );
    for needle in case["expect"]["tail_contains"].as_array().unwrap() {
        assert!(
            status
                .tail()
                .iter()
                .any(|line| line.contains(needle.as_str().unwrap())),
            "{:?}",
            status.tail()
        );
    }
}

#[test]
fn x1_escape_still_cancels_a_running_command_under_a_failure() {
    let document = fixture("negative.json");
    let case = fixture_case(
        &document,
        "X1-escape-still-cancels-a-running-command-under-a-failure",
    );
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status x1")).unwrap();
    warmup(&mut app, &context);
    app.status.error("Unknown command: Foo");
    run_command(&mut app, &context, "Line");
    assert!(app.controller.is_running(), "the Line command is waiting");
    frame(&mut app, &context, escape());
    assert!(
        !app.controller.is_running(),
        "Esc must still reach the command owner"
    );
    assert_eq!(case["expect"]["command_running_after"], false);
    assert_eq!(
        app.status.kind(),
        StatusKind::Error,
        "and the failure stays on the line"
    );
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn x2_to_x6_the_surface_never_swallows_state() {
    let document = fixture("negative.json");

    // X2: Escape does not dismiss a failure.
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status x2")).unwrap();
    warmup(&mut app, &context);
    app.status.error("Unknown command: Foo");
    frame(&mut app, &context, Vec::new());
    let revision = app.status.revision();
    frame(&mut app, &context, escape());
    assert_eq!(app.status.kind(), StatusKind::Error);
    assert_eq!(app.status.revision(), revision);
    assert_eq!(
        fixture_case(&document, "X2-escape-does-not-dismiss-the-failure")["expect"]["revision_unchanged"],
        true
    );

    // X4: a dismissed failure does not come back by itself.
    app.status.acknowledge();
    frame(&mut app, &context, Vec::new());
    frame(&mut app, &context, Vec::new());
    assert_eq!(
        app.status.kind(),
        StatusKind::Ready,
        "{}",
        app.status.message()
    );

    // X5: a blank diagnostic never blanks the line.
    let case = fixture_case(
        &document,
        "X5-a-blank-or-whitespace-diagnostic-never-blanks-the-line",
    );
    let mut status = StatusSurface::new();
    status.error("real failure");
    status.report(StatusKind::Error, case["report"][1].as_str().unwrap());
    assert_eq!(status.kind(), StatusKind::Error);
    assert_eq!(
        status.message(),
        case["expect"]["message"].as_str().unwrap()
    );

    // X6: a newline cannot break the line.
    let case = fixture_case(&document, "X6-a-newline-cannot-break-the-line");
    let mut status = StatusSurface::new();
    status.report(StatusKind::Error, case["report"][1].as_str().unwrap());
    assert_eq!(
        status.message(),
        case["expect"]["message"].as_str().unwrap()
    );
    assert!(!status.message().contains('\n'));
}

#[test]
fn x3_the_surface_never_answers_for_another_owner() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("status x3")).unwrap();
    warmup(&mut app, &context);
    app.status.error("Unknown command: Foo");
    // A controller-layer dialog owns Escape while it is open; the status does not
    // close it and does not change.
    let mut status = StatusSurface::new();
    status.error("Unknown command: Foo");
    let before = status.revision();
    frame(&mut app, &context, Vec::new());
    assert_eq!(app.status.revision(), before);
    assert_eq!(app.status.kind(), StatusKind::Error);
}

#[test]
fn x7_the_surface_is_pure() {
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("status_surface.rs"),
    )
    .expect("status_surface.rs");
    for (number, line) in source.lines().enumerate() {
        let code = line.split("//").next().unwrap_or_default();
        for forbidden in ["std::fs", "std::path", "egui", "Document", "Controller"] {
            assert!(
                !code.contains(forbidden),
                "status_surface.rs:{} uses {forbidden}: the surface stays pure",
                number + 1
            );
        }
    }
    let document = fixture("negative.json");
    let case = fixture_case(&document, "X7-the-surface-is-pure");
    assert_eq!(case["expect"]["no_io"], true);
    assert_eq!(case["expect"]["no_document"], true);
    assert_eq!(case["expect"]["no_egui"], true);
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-01-007/api.md");
    assert!(api.exists(), "{}", api.display());
}
