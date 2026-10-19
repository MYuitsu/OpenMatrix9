// SPDX-License-Identifier: LGPL-2.1-or-later
//! Window-level minimum size for plan task OM9-L3-01-01-002 (user decision
//! 2026-10-16: the window must not shrink below 1280x720 logical points).
//!
//! AUTOMATED/headless: the real `MatrixApp::show` frame runs through egui with
//! a hand-made `ViewportInfo::monitor_size` and the test reads the
//! `ViewportCommand::MinInnerSize` the app asks the window for. It proves the
//! command is sent on the first frame and only when the value changes. It does
//! NOT prove that Windows enforces the size on a real window or on a real
//! 1366x768 laptop at 125% or 150% DPI; the real-exe matrix and the human
//! checklist cover that.
use eframe::egui::{self, Pos2, Rect, Vec2, ViewportCommand, ViewportId, ViewportInfo};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;

/// Run one frame with the given monitor size (logical points) and return the
/// `MinInnerSize` commands emitted by that frame.
fn frame(app: &mut MatrixApp, context: &egui::Context, monitor: Option<Vec2>) -> Vec<Vec2> {
    let mut viewports = egui::ViewportIdMap::default();
    viewports.insert(
        ViewportId::ROOT,
        ViewportInfo {
            monitor_size: monitor,
            ..Default::default()
        },
    );
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            viewports,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
        .viewport_output
        .get(&ViewportId::ROOT)
        .map(|viewport| {
            viewport
                .commands
                .iter()
                .filter_map(|command| match command {
                    ViewportCommand::MinInnerSize(size) => Some(*size),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn new_app() -> (MatrixApp, egui::Context) {
    (
        MatrixApp::new(Document::new("window minimum")).unwrap(),
        egui::Context::default(),
    )
}

fn assert_close(actual: Vec2, expected: [f32; 2], label: &str) {
    assert!(
        (actual.x - expected[0]).abs() < 0.1 && (actual.y - expected[1]).abs() < 0.1,
        "{label}: got {actual:?}, expected {expected:?}"
    );
}

#[test]
fn first_frame_on_a_large_screen_asks_for_1280x720() {
    for monitor in [Vec2::new(2560., 1440.), Vec2::new(1920., 1080.)] {
        let (mut app, context) = new_app();
        let commands = frame(&mut app, &context, Some(monitor));
        assert_eq!(commands.len(), 1, "{monitor:?}: one command on frame 0");
        assert_close(commands[0], [1280., 720.], &format!("{monitor:?}"));
    }
}

#[test]
fn first_frame_on_a_1366x768_screen_at_100_percent_asks_for_1280x688() {
    let (mut app, context) = new_app();
    let commands = frame(&mut app, &context, Some(Vec2::new(1366., 768.)));
    assert_eq!(commands.len(), 1);
    assert_close(commands[0], [1280., 688.], "1366x768 @100%");
}

#[test]
fn small_logical_screens_cap_the_minimum_to_the_usable_area() {
    // 1366x768 at 125% DPI is 1092.8x614.4 logical points, at 150% 910.7x512.
    let (mut app, context) = new_app();
    let commands = frame(
        &mut app,
        &context,
        Some(Vec2::new(1366. / 1.25, 768. / 1.25)),
    );
    assert_eq!(commands.len(), 1);
    assert_close(commands[0], [1076.8, 534.4], "1366x768 @125%");

    let (mut app, context) = new_app();
    let commands = frame(&mut app, &context, Some(Vec2::new(1366. / 1.5, 768. / 1.5)));
    assert_eq!(commands.len(), 1);
    assert_close(commands[0], [894.67, 432.], "1366x768 @150%");
}

#[test]
fn unknown_monitor_asks_for_the_plain_supported_minimum() {
    let (mut app, context) = new_app();
    let commands = frame(&mut app, &context, None);
    assert_eq!(commands.len(), 1);
    assert_close(commands[0], [1280., 720.], "no monitor information");
}

#[test]
fn the_command_is_sent_once_and_again_only_when_the_monitor_changes() {
    let (mut app, context) = new_app();
    let large = Some(Vec2::new(2560., 1440.));
    assert_eq!(frame(&mut app, &context, large).len(), 1, "frame 0");
    for pass in 1..4 {
        assert!(
            frame(&mut app, &context, large).is_empty(),
            "pass {pass}: same monitor, no second command"
        );
    }
    // The window moves to a smaller, scaled monitor: exactly one new command.
    let small = Some(Vec2::new(1366. / 1.5, 768. / 1.5));
    let commands = frame(&mut app, &context, small);
    assert_eq!(commands.len(), 1);
    assert_close(commands[0], [894.67, 432.], "moved to the 150% screen");
    assert!(frame(&mut app, &context, small).is_empty());
    // And back: the minimum returns to 1280x720.
    let commands = frame(&mut app, &context, large);
    assert_eq!(commands.len(), 1);
    assert_close(commands[0], [1280., 720.], "moved back to the large screen");
}
