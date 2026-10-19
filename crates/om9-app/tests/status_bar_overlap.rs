// SPDX-License-Identifier: LGPL-2.1-or-later
//! Regression for the status-bar text overlap (closeout item 1 of plan task
//! OM9-L3-01-01-002): the right-aligned dock summary used to run left over the
//! message label at narrow windows. The message label has priority, the summary
//! is elided to the remaining width, and the full strings stay reachable.
use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;

mod common;
use common::dock_right_fixture;

fn frame_size(app: &mut MatrixApp, context: &egui::Context, size: Vec2) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn startup() -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("status bar overlap")).unwrap();
    let context = egui::Context::default();
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    (app, context)
}

/// Dock the shared neutral fixture right, then settle at `size` for a few frames.
fn settle(app: &mut MatrixApp, context: &egui::Context, size: Vec2) {
    assert!(dock_right_fixture(app));
    for _ in 0..3 {
        frame_size(app, context, size);
    }
}

/// The contract every case shares.
fn assert_status_line_clean(app: &MatrixApp, case: &str) {
    let status = app.status_rect.expect("status panel renders");
    let message = app.status_message_rect.expect("message label recorded");
    let summary = app.status_summary_rect.expect("summary label recorded");
    for (name, rect) in [("message", message), ("summary", summary)] {
        assert!(
            rect.min.x >= status.min.x - 0.5 && rect.max.x <= status.max.x + 0.5,
            "{case}: {name} {rect:?} leaves the status panel {status:?}"
        );
    }
    assert!(
        message.max.x <= summary.min.x + 0.5,
        "{case}: message {message:?} overlaps summary {summary:?}"
    );
    if app.status.kind().is_failure() {
        let dismiss = app
            .status_dismiss_rect
            .expect("Dismiss is shown for a failure");
        assert!(
            dismiss.min.x >= status.min.x - 0.5 && dismiss.max.x <= status.max.x + 0.5,
            "{case}: Dismiss {dismiss:?} leaves the status panel {status:?}"
        );
        assert!(
            summary.max.x <= dismiss.min.x + 0.5,
            "{case}: summary {summary:?} overlaps Dismiss {dismiss:?}"
        );
        assert!(
            message.max.x <= dismiss.min.x + 0.5,
            "{case}: message {message:?} overlaps Dismiss {dismiss:?}"
        );
    } else {
        assert!(app.status_dismiss_rect.is_none(), "{case}: stray Dismiss");
    }
}

/// Nothing is lost by eliding: the full strings still exist in the model.
fn assert_full_text_reachable(app: &MatrixApp, case: &str) {
    assert!(
        app.status.line().ends_with(app.status.message()),
        "{case}: status line lost its message"
    );
    let summary = app.dock_summary();
    assert!(
        summary.contains("left ") && summary.contains("right "),
        "{case}: dock summary text changed: {summary}"
    );
}

#[test]
fn narrow_800x600_with_right_dock_does_not_overlap() {
    let (mut app, context) = startup();
    settle(&mut app, &context, Vec2::new(800., 600.));
    assert_status_line_clean(&app, "800x600");
    assert_full_text_reachable(&app, "800x600");
}

#[test]
fn capped_911x480_with_right_dock_does_not_overlap() {
    // 1366x768 at 150% DPI after the window-level cap: the smallest plannable
    // supported window in logical points (the dock yields here).
    let (mut app, context) = startup();
    settle(&mut app, &context, Vec2::new(911., 480.));
    assert_status_line_clean(&app, "911x480");
    assert_full_text_reachable(&app, "911x480");
}

#[test]
fn normal_1600x1000_control_shows_the_full_summary() {
    let (mut app, context) = startup();
    settle(&mut app, &context, Vec2::new(1600., 1000.));
    assert_status_line_clean(&app, "1600x1000");
    assert_full_text_reachable(&app, "1600x1000");
    // Wide enough for the whole summary: the label is not elided, so its width
    // is the natural width of the full text, well above a clipped stub.
    let summary = app.status_summary_rect.unwrap();
    assert!(
        summary.width() > 200.,
        "summary unexpectedly elided at 1600x1000: {summary:?}"
    );
}

#[test]
fn long_error_at_floor_800x400_keeps_dismiss_and_message_apart() {
    let (mut app, context) = startup();
    settle(&mut app, &context, Vec2::new(800., 400.));
    let long = "Export failed: ".to_owned() + &"the destination is not writable ".repeat(12);
    assert!(app.status.error(&long));
    for _ in 0..2 {
        frame_size(&mut app, &context, Vec2::new(800., 400.));
    }
    assert!(app.status.kind().is_failure(), "error stays unacknowledged");
    assert_status_line_clean(&app, "800x400 error");
    assert_full_text_reachable(&app, "800x400 error");
    assert!(
        app.status.message().len() >= long.len().min(512) - 1,
        "full failure text is still stored"
    );
}
