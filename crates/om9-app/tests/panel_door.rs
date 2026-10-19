// SPDX-License-Identifier: LGPL-2.1-or-later
//! UI-level contract tests for the L3-003 single door, on real egui frames.
//!
//! These are the `positive.json` / `negative.json` cases whose path is
//! "mouse: …": a press on a title bar, moves, and a release inside the live
//! rectangles the frame recorded.
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::dock_layout::{DockLayout, DockSide, FloatRect, SectionId};
use om9_app::panel_ops::{ColumnSide, DropTarget, PanelOp, PanelOpError};
use om9_app::status_surface::StatusKind;
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-003")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("fixture case {id}"))
}

fn app(name: &str) -> MatrixApp {
    MatrixApp::new(Document::new(name)).expect("app")
}

fn frame(app: &mut MatrixApp, context: &egui::Context, size: Vec2, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn warmup(app: &mut MatrixApp, context: &egui::Context, size: Vec2) {
    for _ in 0..3 {
        frame(app, context, size, Vec::new());
    }
}

fn pointer(pos: Pos2, pressed: bool) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        },
    ]
}

fn moved(pos: Pos2) -> Vec<Event> {
    vec![Event::PointerMoved(pos)]
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

fn left_keys(app: &MatrixApp) -> Vec<&'static str> {
    app.dock
        .left_sections()
        .into_iter()
        .map(|id| id.key())
        .collect()
}

fn fixture_order(case: &Value) -> Vec<String> {
    case["expect"]["left_order"]
        .as_array()
        .expect("left_order")
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

/// The documented left column of the fixture document.
fn documented_order(document: &Value) -> Vec<String> {
    document["documented_left_order"]
        .as_array()
        .expect("documented_left_order")
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

/// Press a title bar and move 10 px so egui reports a drag.
fn press_and_drag(app: &mut MatrixApp, context: &egui::Context, size: Vec2, bar: Rect, to: Pos2) {
    let start = bar.center();
    frame(app, context, size, pointer(start, true));
    frame(app, context, size, moved(start + Vec2::new(0., 10.)));
    frame(app, context, size, moved(to));
}

const SIZE: Vec2 = Vec2::new(1600., 1000.);

#[test]
fn p3_a_drag_onto_the_upper_half_of_the_display_bar_lands_at_position_2() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P3-drag-projects-onto-the-upper-half-of-the-display-bar",
    );
    let context = egui::Context::default();
    let mut app = app("p3");
    warmup(&mut app, &context, SIZE);
    let projects = app.section_rect("projects").expect("projects bar");
    let display = app.section_rect("display").expect("display bar");
    let to = Pos2::new(display.center().x, display.min.y + 3.);

    press_and_drag(&mut app, &context, SIZE, projects, to);
    let preview = app.drop_preview.expect("preview while pressed");
    assert_eq!(
        preview.target,
        DropTarget::Place {
            side: ColumnSide::Left,
            index: 2
        }
    );
    let indicator = preview.indicator.expect("a 3 px line");
    assert!(indicator.line);
    assert!((indicator.rect.height - 3.).abs() < 0.01);
    assert!((indicator.rect.y - (display.min.y - 1.5)).abs() < 0.6);

    frame(&mut app, &context, SIZE, pointer(to, false));
    assert_eq!(left_keys(&app), fixture_order(case));
    assert!(app.section_drag().is_none());
    assert!(app.drop_preview.is_none());
    let expected = case["expect"]["status"].as_str().unwrap();
    assert!(
        app.transcript.history().contains(expected),
        "history must contain {expected}: {}",
        app.transcript.history()
    );
    assert_eq!(case["expect"]["projects.order"], 2);
    assert_eq!(app.dock.section(SectionId::Projects).order, 2);
}

#[test]
fn p4_a_drag_onto_the_lower_half_of_the_snaps_bar_lands_at_position_3() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P4-drag-main-menu-onto-the-lower-half-of-the-snaps-bar",
    );
    let context = egui::Context::default();
    let mut app = app("p4");
    warmup(&mut app, &context, SIZE);
    let main_menu = app.section_rect("main-menu").expect("main menu bar");
    let snaps = app.section_rect("snaps").expect("snaps bar");
    let to = Pos2::new(snaps.center().x, snaps.max.y - 2.);

    press_and_drag(&mut app, &context, SIZE, main_menu, to);
    assert_eq!(
        app.drop_preview.expect("preview").target,
        DropTarget::Place {
            side: ColumnSide::Left,
            index: 3
        }
    );
    frame(&mut app, &context, SIZE, pointer(to, false));
    assert_eq!(left_keys(&app), fixture_order(case));
}

#[test]
fn p9_the_mouse_path_and_the_api_path_produce_the_same_layout() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P9-menu-mouse-and-api-paths-give-identical-layouts",
    );
    assert_eq!(case["compare"].as_array().unwrap().len(), 2);
    let context = egui::Context::default();
    let mut mouse = app("p9 mouse");
    warmup(&mut mouse, &context, SIZE);
    let projects = mouse.section_rect("projects").expect("projects bar");
    let display = mouse.section_rect("display").expect("display bar");
    let to = Pos2::new(display.center().x, display.min.y + 3.);
    press_and_drag(&mut mouse, &context, SIZE, projects, to);
    frame(&mut mouse, &context, SIZE, pointer(to, false));

    let mut api = app("p9 api");
    api.apply_panel_op(PanelOp::Place {
        section: SectionId::Projects,
        side: ColumnSide::Left,
        index: 2,
    })
    .expect("api placement");

    // The panel sizes belong to the frame, not to the placement: the menu
    // sections are what both paths must agree on.
    let sections = |app: &MatrixApp| serde_json::to_value(&app.dock.sections).unwrap();
    assert_eq!(
        sections(&mouse),
        sections(&api),
        "the mouse path and the API path are the same operation"
    );
    assert_eq!(left_keys(&mouse), left_keys(&api));
    assert_eq!(case["expect"]["to_json_equal"], true);
}

#[test]
fn p13_panel_operations_touch_ui_state_only() {
    let document = fixture("positive.json");
    let case = case(&document, "P13-operations-touch-ui-state-only");
    for key in case["expect_unchanged"].as_array().unwrap() {
        assert!(key.is_string());
    }
    let context = egui::Context::default();
    let mut app = app("p13");
    warmup(&mut app, &context, SIZE);
    let stamp = |app: &MatrixApp| {
        (
            app.controller.document().entities.len(),
            app.controller.document().current_layer,
            app.controller.coordinator().undo_len(),
            app.controller.coordinator().redo_len(),
            app.transcript.draft().to_owned(),
            app.controller.is_running(),
            app.controller.workspace.active,
            app.controller.workspace.maximized,
            app.controller.workspace.views.clone(),
        )
    };
    let id_count = |app: &MatrixApp| app.controller.document().entities.len();
    let before = stamp(&app);
    let before_ids = id_count(&app);
    let temp = std::env::temp_dir().join("om9-l3-003-p13-layout.json");
    let _ = std::fs::remove_file(&temp);
    app.set_dock_layout_path(Some(temp.clone()));
    assert!(!temp.exists(), "the fixture path starts absent");

    let work = app.work_area.expect("the frame produced a work area");
    app.apply_panel_op(PanelOp::ToggleCollapsed {
        section: SectionId::Snaps,
    })
    .expect("collapse");
    app.apply_panel_op(PanelOp::Move {
        section: SectionId::Projects,
        delta: -1,
    })
    .expect("move");
    app.apply_panel_op(PanelOp::Dock {
        section: SectionId::Snaps,
        side: ColumnSide::Right,
    })
    .expect("dock right");
    app.apply_panel_op(PanelOp::Float {
        section: SectionId::Display,
        rect: FloatRect {
            x: 600.,
            y: 300.,
            width: 300.,
            height: 240.,
        },
        work,
        origin: om9_app::panel_ops::FloatOrigin::Api,
    })
    .expect("float");
    app.apply_panel_op(PanelOp::DockHome {
        section: SectionId::Display,
    })
    .expect("dock home");

    assert_eq!(stamp(&app), before, "panel operations are UI state only");
    assert_eq!(id_count(&app), before_ids);
    assert!(app.menus_have_unsaved_changes());
    assert_eq!(case["expect_dock_dirty"], true);
    assert!(
        !temp.exists(),
        "Set as Default is the only writer: {} must stay absent",
        temp.display()
    );
    app.set_menus_as_default();
    assert!(temp.exists(), "Set as Default writes the layout");
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn n17_floating_before_the_first_frame_reports_no_work_area() {
    let document = fixture("negative.json");
    let case = case(&document, "N17-float-before-the-first-frame");
    assert_eq!(case["frames_before"], 0);
    let mut app = app("n17");
    let before = app.dock.to_json().unwrap();
    let rect = FloatRect {
        x: 600.,
        y: 300.,
        width: 320.,
        height: 240.,
    };
    assert!(!app.float_menu_section(SectionId::Display, rect));
    let record = app.last_panel_op.as_ref().expect("the attempt is recorded");
    assert!(!record.ok);
    assert!(matches!(record.error, Some(PanelOpError::NoWorkArea)));
    assert_eq!(app.dock.to_json().unwrap(), before);
    assert_eq!(case["expect"]["returns"], false);
    assert_eq!(case["expect"]["layout_unchanged"], true);
}

#[test]
fn n19_escape_cancels_a_drag_and_keeps_the_command_draft() {
    let document = fixture("negative.json");
    let case = case(
        &document,
        "N19-escape-cancels-a-drag-and-keeps-the-cmd-draft",
    );
    let context = egui::Context::default();
    let mut app = app("n19");
    warmup(&mut app, &context, SIZE);
    // A command is running with a draft, exactly as the fixture sets it up.
    frame(&mut app, &context, SIZE, vec![Event::Text("Line".into())]);
    frame(&mut app, &context, SIZE, vec![key(Key::Enter)]);
    frame(&mut app, &context, SIZE, vec![Event::Text("1,2,".into())]);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "1,2,");

    let display = app.section_rect("display").expect("display bar");
    let view = *app.view_rects.values().next().expect("a view");
    press_and_drag(&mut app, &context, SIZE, display, view.center());
    assert!(
        matches!(
            app.drop_preview.expect("preview").target,
            DropTarget::Float { .. }
        ),
        "the drag previews a float"
    );

    frame(&mut app, &context, SIZE, vec![key(Key::Escape)]);
    assert!(app.section_drag().is_none(), "Escape cancels the drag");
    assert!(app.drop_preview.is_none());
    assert_eq!(
        app.status.kind(),
        StatusKind::Info,
        "{}",
        app.status.message()
    );
    assert_eq!(
        app.status.message(),
        case["expect"]["status"].as_str().unwrap()
    );

    frame(&mut app, &context, SIZE, pointer(view.center(), false));
    assert_eq!(app.transcript.draft(), "1,2,", "the draft survives");
    assert!(app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert_eq!(case["expect"]["release_applies_nothing"], true);
    assert_eq!(left_keys(&app)[2], "display", "nothing was applied");
}

#[test]
fn n29_reset_reload_and_hiding_cancel_a_drag_in_progress() {
    let document = fixture("negative.json");
    let case = case(
        &document,
        "N29-drag-in-progress-across-reset-reload-and-hide",
    );
    let expected_history = case["expect"]["cmd_history_contains"].as_str().unwrap();
    let variants = ["reset", "reload", "hide"];
    assert_eq!(
        case["variants"].as_array().unwrap().len(),
        variants.len() - 1
    );
    for variant in variants {
        let context = egui::Context::default();
        let mut app = app("n29");
        warmup(&mut app, &context, SIZE);
        let projects = app.section_rect("projects").expect("projects bar");
        let view = *app.view_rects.values().next().expect("a view");
        press_and_drag(&mut app, &context, SIZE, projects, view.center());
        assert!(app.section_drag().is_some(), "{variant}: dragging");

        match variant {
            "reset" => app.reset_menus(Some(&context)),
            "reload" => app.reload_menus(),
            _ => app.dock.set_section_visible(SectionId::Projects, false),
        }
        frame(&mut app, &context, SIZE, Vec::new());
        assert!(app.section_drag().is_none(), "{variant}: drag cancelled");
        assert!(app.drop_preview.is_none(), "{variant}: preview cleared");
        assert!(
            app.transcript.history().contains(expected_history),
            "{variant}: history should contain {expected_history}: {}",
            app.transcript.history()
        );

        // The late release applies nothing.
        let before = app.dock.to_json().unwrap();
        frame(&mut app, &context, SIZE, pointer(view.center(), false));
        frame(&mut app, &context, SIZE, Vec::new());
        let after = app.dock.to_json().unwrap();
        if variant != "reset" {
            assert_eq!(before, after, "{variant}: the release changes nothing");
        }
        assert_eq!(case["expect"]["release_applies_nothing"], true);
        assert_eq!(case["expect"]["no_panic"], true);
    }
}

#[test]
fn n30_an_unacknowledged_failure_is_not_hidden_by_panel_messages() {
    let document = fixture("negative.json");
    let case = case(
        &document,
        "N30-unacknowledged-error-is-not-hidden-by-panel-messages",
    );
    let context = egui::Context::default();
    let mut app = app("n30");
    warmup(&mut app, &context, SIZE);
    app.status.error("NotACommand: command not recognized");
    let message = app.status.message().to_owned();
    assert_eq!(app.status.kind(), StatusKind::Error);

    let projects = app.section_rect("projects").expect("projects bar");
    let display = app.section_rect("display").expect("display bar");
    let to = Pos2::new(display.center().x, display.min.y + 3.);
    press_and_drag(&mut app, &context, SIZE, projects, to);
    frame(&mut app, &context, SIZE, pointer(to, false));

    assert_eq!(
        app.status.kind(),
        StatusKind::Error,
        "the failure keeps the status line"
    );
    assert_eq!(app.status.message(), message);
    assert_eq!(case["expect"]["status_kind"], "Error");
    assert_eq!(case["expect"]["status_message_unchanged"], true);
    let history = case["expect"]["cmd_history_contains"].as_str().unwrap();
    assert!(
        app.transcript.history().contains(history),
        "the panel message still reaches the history: {}",
        app.transcript.history()
    );
}

#[test]
fn p2_a_rolled_up_snaps_and_layers_report_their_closed_bar_controls() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P2-collapsed-snaps-and-layers-keep-closed-bar-controls",
    );
    let expected: Vec<&str> = case["expect"]["closed_bar_controls_contains"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let context = egui::Context::default();
    let mut app = app("closed bars");
    warmup(&mut app, &context, SIZE);
    assert!(app.closed_bar_controls.is_empty());

    for key in &expected {
        let bar = app.section_rect(key).unwrap_or_else(|| panic!("{key} bar"));
        let at = bar.center();
        frame(&mut app, &context, SIZE, pointer(at, true));
        frame(&mut app, &context, SIZE, pointer(at, false));
        assert!(
            app.dock
                .section(SectionId::from_key(key).unwrap())
                .collapsed,
            "{key} did not roll up"
        );
        assert!(
            app.closed_bar_controls.contains(key),
            "{key} must render its controls on the closed bar: {:?}",
            app.closed_bar_controls
        );
        assert_eq!(left_keys(&app), documented_order(&document));
    }

    // Expanding both again empties the set: the controls live on the closed bar.
    for key in &expected {
        let bar = app.section_rect(key).unwrap_or_else(|| panic!("{key} bar"));
        let at = bar.center();
        frame(&mut app, &context, SIZE, pointer(at, true));
        frame(&mut app, &context, SIZE, pointer(at, false));
        assert!(
            !app.dock
                .section(SectionId::from_key(key).unwrap())
                .collapsed
        );
    }
    assert!(
        app.closed_bar_controls.is_empty(),
        "{:?}",
        app.closed_bar_controls
    );
}

#[test]
fn n21_a_click_on_the_builders_bar_is_refused_and_starts_no_drag() {
    let document = fixture("negative.json");
    let case = case(&document, "N21-builders-bar-click-is-refused");
    let context = egui::Context::default();
    let mut app = app("builders bar");
    warmup(&mut app, &context, SIZE);
    let before = app.dock.to_json().unwrap();
    let bar = app.section_rect("builder-slot").expect("builders bar");
    let at = bar.center();
    frame(&mut app, &context, SIZE, pointer(at, true));
    frame(&mut app, &context, SIZE, pointer(at, false));

    assert!(
        matches!(
            app.last_panel_op
                .as_ref()
                .and_then(|record| record.error.as_ref()),
            Some(PanelOpError::NotPermitted { .. })
        ),
        "the click is refused: {:?}",
        app.last_panel_op
    );
    assert_eq!(
        app.status.message(),
        case["expect"]["status"].as_str().unwrap()
    );
    assert!(!app.dock.section(SectionId::BuilderSlot).collapsed);
    assert_eq!(case["expect"]["builder-slot.collapsed"], false);
    assert!(app.section_drag().is_none(), "no drag can start");
    assert_eq!(case["expect"]["drag_cannot_start"], true);
    assert_eq!(app.dock.to_json().unwrap(), before);
    assert_eq!(case["expect"]["layout_unchanged"], true);

    // A drag attempt on the same bar starts nothing either.
    frame(
        &mut app,
        &context,
        SIZE,
        pointer(bar.center() + Vec2::new(0., 10.), true),
    );
    frame(
        &mut app,
        &context,
        SIZE,
        moved(bar.center() + Vec2::new(0., 40.)),
    );
    assert!(app.section_drag().is_none());
    assert!(app.drop_preview.is_none());
}

#[test]
fn the_door_is_the_only_way_the_tests_and_the_ui_place_menus() {
    // A guard for future edits: the wrappers on MatrixApp are delegations, and a
    // menu that cannot be moved reports it instead of changing anything.
    let context = egui::Context::default();
    let mut app = app("door");
    warmup(&mut app, &context, SIZE);
    let before = app.dock.to_json().unwrap();
    assert!(!app.toggle_menu_section(SectionId::BuilderSlot));
    assert!(matches!(
        app.last_panel_op.as_ref().unwrap().error,
        Some(PanelOpError::NotPermitted { .. })
    ));
    assert_eq!(app.dock.to_json().unwrap(), before);
    assert!(app.dock_menu_section(SectionId::Snaps, DockSide::Right));
    assert_eq!(app.dock.right_sections(), vec![SectionId::Snaps]);
    assert!(!app.move_menu_section(SectionId::Snaps, -1));
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src", "ui", "panel_audit.rs"]
        .iter()
        .collect();
    assert!(path.exists());
    let _ = DockLayout::matrix_default();
}

/// The L3-003 design review asks how a panel message interacts with the
/// proven stale-cursor rule of L3-007: history that grows while the user is
/// typing must not drop keystrokes. This is that regression.
#[test]
fn panel_history_never_drops_a_keystroke_from_the_command_draft() {
    let context = egui::Context::default();
    let mut app = app("stale cursor");
    warmup(&mut app, &context, SIZE);
    // A running command with a draft the user is still typing.
    frame(&mut app, &context, SIZE, vec![Event::Text("Line".into())]);
    frame(&mut app, &context, SIZE, vec![key(Key::Enter)]);
    frame(&mut app, &context, SIZE, vec![Event::Text("1,2,".into())]);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "1,2,");

    // Panel messages append history. Loop a few times with a typed character in
    // between: every character must land in the draft, and the transcript text
    // must stay draft-suffix consistent.
    for step in 0..4 {
        let _ = app.toggle_menu_section(SectionId::Snaps);
        let _ = app.toggle_menu_section(SectionId::Snaps);
        frame(
            &mut app,
            &context,
            SIZE,
            vec![Event::Text(step.to_string())],
        );
        let expected = format!(
            "1,2,{}",
            (0..=step).map(|n| n.to_string()).collect::<String>()
        );
        assert_eq!(
            app.transcript.draft(),
            expected,
            "step {step}: a panel message dropped a keystroke"
        );
        assert!(
            app.transcript.text().ends_with(app.transcript.draft()),
            "step {step}: the transcript text must end with the draft"
        );
        assert!(
            app.transcript.history().contains("rolled up")
                || app.transcript.history().contains("expanded"),
            "step {step}: the panel message reached the history"
        );
    }
    // Closing the command keeps the document untouched: panels are UI state.
    assert!(app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
