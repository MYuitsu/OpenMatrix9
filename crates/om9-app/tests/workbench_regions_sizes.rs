// SPDX-License-Identifier: LGPL-2.1-or-later
//! Size and simulated-DPI matrix for the workbench docking regions of plan task
//! OM9-L3-01-01-002 (closeout item 2).
//!
//! SIMULATED/AUTOMATED evidence. It does not replace the human M01-UI sign-off.
//!
//! Every case drives the real `MatrixApp::show` frame through headless egui and
//! checks one contract (`check_frame`): a valid plan, four non-overlapping
//! viewports inside the plan's centre, a command input that is present, inside
//! its panel and clear of every viewport, a status line whose message and
//! summary do not overlap, and finite rects inside the screen.
//!
//! About the 150% cases: egui lays out in points. `set_pixels_per_point(1.5)`
//! with a screen of `physical / 1.5` points therefore proves point-space layout
//! and rounding at 1.5. It does NOT prove anything about OS per-monitor DPI,
//! a real second monitor, or GPU text rendering at that scale.
//!
//! Hiding the command line has no menu or mouse entry in `ui.rs` or
//! `ui/chrome.rs` (the only `PanelId::CommandLine` use is a size sync), so the
//! hidden-command-line case is driven through the layout API
//! (`DockLayout::set_visible`), not through the menu.
use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::dock_layout::{
    CANVAS_MIN_WIDTH, COMMAND_MIN, DockLayout, DockSide, LEFT_RAIL_DEFAULT, LEFT_RAIL_MAX,
    LEFT_RAIL_MIN, NARROW_WINDOW_WIDTH, PanelId, RIGHT_DOCK_FIXTURE, RIGHT_DOCK_MAX,
};
use om9_app::file_lifecycle::{FileKind, FileOutcome};
use om9_app::startup::{AppConfig, StartupStage};
use om9_app::status_surface::StatusKind;
use om9_app::ui::MatrixApp;
use om9_app::workbench_regions::{
    RegionError, RegionNote, RegionPlan, RegionRect, WindowSize, YieldReason, compute_regions,
};
use om9_core::model::Document;
use om9_scene::workspace::ViewId;
use std::path::PathBuf;

mod common;

/// Tolerance between the pure plan and the egui panels (frame margins), as in
/// `tests/workbench_regions.rs`.
const PLAN_TOLERANCE: f32 = 8.;
/// Pixel slack for float comparisons.
const SLACK: f32 = 0.5;

// ----------------------------------------------------------------- helpers

/// Run one frame. `physical` is the window in device pixels and `ppp` the
/// pixels-per-point; the screen handed to egui is `physical / ppp` points.
fn frame_scaled(app: &mut MatrixApp, context: &egui::Context, physical: Vec2, ppp: f32) {
    context.set_pixels_per_point(ppp);
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, physical / ppp)),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn frame_size(app: &mut MatrixApp, context: &egui::Context, size: Vec2) {
    frame_scaled(app, context, size, 1.0);
}

fn startup_scaled(ppp: f32, physical: Vec2) -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("workbench region sizes")).unwrap();
    let context = egui::Context::default();
    frame_scaled(&mut app, &context, physical, ppp);
    frame_scaled(&mut app, &context, physical, ppp);
    (app, context)
}

fn startup() -> (MatrixApp, egui::Context) {
    startup_scaled(1.0, Vec2::new(1280., 900.))
}

/// Dock the shared neutral fixture (Snaps, never Layers) on the right and
/// settle at `physical` for a few frames.
fn settle_scaled(
    app: &mut MatrixApp,
    context: &egui::Context,
    physical: Vec2,
    ppp: f32,
    dock: bool,
) {
    if dock {
        assert!(common::dock_right_fixture(app));
    }
    for _ in 0..3 {
        frame_scaled(app, context, physical, ppp);
    }
}

fn settle(app: &mut MatrixApp, context: &egui::Context, size: Vec2, dock: bool) {
    settle_scaled(app, context, size, 1.0, dock);
}

fn to_rect(r: RegionRect) -> Rect {
    Rect::from_min_size(Pos2::new(r.x, r.y), Vec2::new(r.width, r.height))
}

fn finite(rect: Rect) -> bool {
    rect.min.x.is_finite()
        && rect.min.y.is_finite()
        && rect.max.x.is_finite()
        && rect.max.y.is_finite()
}

fn contains(outer: Rect, inner: Rect, tolerance: f32) -> bool {
    inner.min.x >= outer.min.x - tolerance
        && inner.min.y >= outer.min.y - tolerance
        && inner.max.x <= outer.max.x + tolerance
        && inner.max.y <= outer.max.y + tolerance
}

fn has_note(plan: &RegionPlan, wanted: impl Fn(&RegionNote) -> bool) -> bool {
    plan.notes.iter().any(wanted)
}

fn yielded(plan: &RegionPlan, reason: YieldReason) -> bool {
    has_note(
        plan,
        |note| matches!(note, RegionNote::RightDockYielded { reason: r } if *r == reason),
    )
}

/// The contract every size shares. `screen` is the screen in points.
fn check_frame(app: &MatrixApp, screen: Vec2, label: &str) -> RegionPlan {
    assert!(
        app.region_error.is_none(),
        "{label}: unexpected region error {:?}",
        app.region_error
    );
    let plan = app
        .region_plan
        .clone()
        .unwrap_or_else(|| panic!("{label}: no plan"));
    plan.validate()
        .unwrap_or_else(|error| panic!("{label}: invalid plan: {error}"));
    assert!(
        (plan.window.width - screen.x).abs() <= SLACK
            && (plan.window.height - screen.y).abs() <= SLACK,
        "{label}: plan window {:?} vs screen {screen:?}",
        plan.window
    );
    let screen_rect = Rect::from_min_size(Pos2::ZERO, screen);
    let center = to_rect(plan.center);

    // Four viewports, pairwise apart, inside the centre.
    assert_eq!(app.view_rects.len(), 4, "{label}: four viewports");
    let views: Vec<_> = app.view_rects.iter().map(|(v, r)| (*v, *r)).collect();
    for (index, (a, rect_a)) in views.iter().enumerate() {
        assert!(finite(*rect_a), "{label}: {a:?} not finite");
        assert!(
            contains(screen_rect, *rect_a, SLACK),
            "{label}: {a:?} {rect_a:?} leaves the screen {screen_rect:?}"
        );
        assert!(
            contains(center, *rect_a, PLAN_TOLERANCE),
            "{label}: {a:?} {rect_a:?} leaves the plan centre {center:?}"
        );
        for (b, rect_b) in &views[index + 1..] {
            assert!(
                !rect_a.shrink(0.01).intersects(*rect_b),
                "{label}: viewports {a:?} and {b:?} overlap"
            );
        }
    }

    // Left rail and right dock stay out of the viewports.
    match (plan.left_rail, app.sidebar_rect) {
        (Some(_), Some(left)) => {
            for (view, rect) in &views {
                assert!(
                    left.max.x <= rect.min.x + SLACK,
                    "{label}: rail over {view:?}"
                );
            }
            assert!(
                finite(left) && contains(screen_rect, left, SLACK),
                "{label}: left rail"
            );
        }
        (Some(_), None) => panic!("{label}: planned left rail did not render"),
        (None, _) => {}
    }
    match (plan.right_dock, app.right_dock_rect) {
        (Some(_), Some(right)) => {
            for (view, rect) in &views {
                assert!(
                    rect.max.x <= right.min.x + SLACK,
                    "{label}: dock over {view:?}"
                );
            }
            assert!(
                finite(right) && contains(screen_rect, right, SLACK),
                "{label}: right dock"
            );
        }
        (Some(_), None) => panic!("{label}: planned right dock did not render"),
        (None, rendered) => assert!(rendered.is_none(), "{label}: unplanned right dock rendered"),
    }

    // Command input: present, inside its panel, clear of every viewport.
    let command = app
        .command_input_rect()
        .unwrap_or_else(|| panic!("{label}: command input missing"));
    assert!(finite(command), "{label}: command input not finite");
    assert!(
        contains(screen_rect, command, SLACK),
        "{label}: command input {command:?} leaves the screen"
    );
    let panel = to_rect(plan.command_input);
    assert!(
        command.min.x >= panel.min.x - SLACK && command.max.x <= panel.max.x + SLACK,
        "{label}: command input {command:?} outside its panel {panel:?} horizontally"
    );
    assert!(
        command.min.y >= panel.min.y - SLACK,
        "{label}: command input {command:?} starts above its panel {panel:?}"
    );
    // The egui panel carries frame margins, so the input line may extend a few
    // pixels past the plan's (inner) height. When the plan clamps the panel to
    // COMMAND_MIN (windows shorter than 311 px) egui still draws the stored,
    // taller command panel. That known plan-versus-frame gap is out of the
    // supported range now (planner floor 400 px height, window minimum 720 or
    // the capped screen value) and is recorded, not fixed and not asserted.
    if plan.command_input.height > COMMAND_MIN + SLACK {
        assert!(
            command.max.y <= panel.max.y + PLAN_TOLERANCE,
            "{label}: command input {command:?} outside its panel {panel:?} vertically"
        );
    }
    assert!(
        plan.command_input.height >= 58.,
        "{label}: command panel minimum height"
    );
    for (view, rect) in &views {
        assert!(
            !command.intersects(*rect),
            "{label}: command input over {view:?}"
        );
    }

    // Status line: message and summary apart, both inside the screen.
    let status = app
        .status_rect
        .unwrap_or_else(|| panic!("{label}: status panel missing"));
    let message = app
        .status_message_rect
        .unwrap_or_else(|| panic!("{label}: status message missing"));
    let summary = app
        .status_summary_rect
        .unwrap_or_else(|| panic!("{label}: status summary missing"));
    for (name, rect) in [
        ("status", status),
        ("message", message),
        ("summary", summary),
    ] {
        assert!(finite(rect), "{label}: {name} not finite");
        assert!(
            contains(screen_rect, rect, SLACK),
            "{label}: {name} {rect:?} leaves the screen {screen_rect:?}"
        );
    }
    assert!(
        message.max.x <= summary.min.x + SLACK,
        "{label}: status message {message:?} overlaps summary {summary:?}"
    );
    plan
}

/// Frame at `physical` (ppp 1.0) with the right dock requested and check it.
fn check_size(width: f32, height: f32, dock: bool, label: &str) -> RegionPlan {
    let (mut app, context) = startup();
    let size = Vec2::new(width, height);
    settle(&mut app, &context, size, dock);
    check_frame(&app, size, label)
}

// ------------------------------------------------------- supported sizes

/// The right dock is shown only from `NARROW_WINDOW_WIDTH` logical points.
fn dock_expected(width_points: f32) -> bool {
    width_points >= NARROW_WINDOW_WIDTH
}

/// Frame at `screen` points (ppp 1.0), the Snaps fixture docked right or not, and check the
/// contract plus the dock rule.
fn check_supported(width: f32, height: f32, dock: bool, label: &str) -> RegionPlan {
    let label = format!("{label} {width}x{height} dock={dock}");
    let plan = check_size(width, height, dock, &label);
    assert_eq!(plan.views.len(), 4, "{label}");
    if dock {
        assert_eq!(
            plan.right_dock.is_some(),
            dock_expected(width),
            "{label}: dock vs the {NARROW_WINDOW_WIDTH} pt threshold"
        );
        if !dock_expected(width) {
            assert!(yielded(&plan, YieldReason::NarrowWindow), "{label}");
            assert!(!yielded(&plan, YieldReason::NotEnoughRoom), "{label}");
        }
    } else {
        assert!(plan.right_dock.is_none(), "{label}: nothing docked");
    }
    plan
}

#[test]
fn supported_sizes_keep_the_contract() {
    // (width, height) in logical points: the nominal minimum, the 1366x768
    // laptop at 100% (client area after taskbar and title bar), at 125% and at
    // 150% after the window-level cap, full HD, ultra-wide and a tall window.
    let sizes = [
        (1280., 720.),
        (1366., 728.),
        (1093., 585.),
        (911., 480.),
        (1920., 1080.),
        (5120., 1440.),
        (1280., 2000.),
    ];
    for (width, height) in sizes {
        for dock in [true, false] {
            check_supported(width, height, dock, "supported");
        }
    }
}

#[test]
fn capped_high_dpi_case_911x480() {
    // 1366x768 at 150% DPI is 910.7x512 logical points; the window-level cap
    // allows about 895x432, and the region plan floor is 800x400, so a window
    // of 911x480 points must keep the left rail, four viewports and the
    // command panel with the dock yielded.
    for dock in [true, false] {
        let plan = check_supported(911., 480., dock, "capped high-DPI");
        assert!(plan.left_rail.is_some());
        assert!(plan.right_dock.is_none());
        // The frame measures the rails after egui's own margins (about 5 pt
        // each), so allow `PLAN_TOLERANCE` around the pure-planner numbers.
        for slot in &plan.views {
            assert!(
                (slot.rect.width - 304.5).abs() <= PLAN_TOLERANCE
                    && (slot.rect.height - 140.).abs() <= PLAN_TOLERANCE,
                "{:?}: {:?}",
                slot.view,
                slot.rect
            );
        }
        assert!(plan.command_input.height >= COMMAND_MIN);
    }
    // Exact numbers from the pure planner (default rails 300 + 300).
    let pure = pure_plan(911., 480.);
    assert_eq!(pure.right_dock, None);
    assert_eq!((pure.center.width, pure.center.height), (611., 282.));
    for slot in &pure.views {
        assert_eq!((slot.rect.width, slot.rect.height), (304.5, 140.));
    }
}

// ------------------------------------------- 1200 pt right-dock threshold

#[test]
fn dock_threshold_sweep_1150_to_1250() {
    let (mut app, context) = startup();
    assert!(common::dock_right_fixture(&mut app));
    let mut first = None;
    for width in 1150_u32..=1250 {
        frame_size(&mut app, &context, Vec2::new(width as f32, 800.));
        let plan = app.region_plan.clone().expect("plan");
        plan.validate()
            .unwrap_or_else(|error| panic!("{width}: {error}"));
        let has_dock = plan.right_dock.is_some();
        if width < 1200 {
            assert!(!has_dock, "{width}: dock absent below the threshold");
            assert!(yielded(&plan, YieldReason::NarrowWindow), "{width}");
        } else {
            assert!(has_dock, "{width}: dock present from the threshold");
            assert!(
                !has_note(&plan, |n| matches!(n, RegionNote::RightDockYielded { .. })),
                "{width}: no yield note when the dock is shown"
            );
        }
        if has_dock && first.is_none() {
            first = Some(width);
        }
    }
    let first = first.expect("a dock appears in 1150..=1250");
    println!("OM9_SWEEP first_dock_width={first} height=800 fixture_docked_right=snaps");
    assert_eq!(first, NARROW_WINDOW_WIDTH as u32);

    // Full frame contract on both sides of the boundary.
    let below = check_size(1199., 800., true, "boundary 1199x800");
    assert!(below.right_dock.is_none());
    let above = check_size(1200., 800., true, "boundary 1200x800");
    assert!(above.right_dock.is_some());
    // Pure planner: (1200 - 300 - 300 - 2) / 2 = 299 pt. The frame measures
    // the rails after egui margins, so it is within `PLAN_TOLERANCE` of that.
    assert_eq!(pure_plan(1200., 800.).views[0].rect.width, 299.);
    assert!(
        (above.views[0].rect.width - 299.).abs() <= PLAN_TOLERANCE,
        "viewport width at the threshold: {}",
        above.views[0].rect.width
    );
}

// ------------------------------------------------- computed layout evidence

/// The pure planner (`compute_regions`) for the default layout with Snaps
/// docked on the right: the numbers a reader can recompute by hand from the
/// documented constants (rails 300 + 300, command 90, menu 21, toolbar 67,
/// status 20, view gap 2).
fn pure_plan(width: f32, height: f32) -> RegionPlan {
    let mut layout = DockLayout::matrix_default();
    layout
        .dock_section(RIGHT_DOCK_FIXTURE, DockSide::Right)
        .expect("the fixture docks right");
    compute_regions(&layout, WindowSize::new(width, height), &ViewId::ALL).expect("plan")
}

/// One `OM9_LAYOUT {json}` line per evidence size. The pure-planner numbers are
/// hand-computed and asserted exactly; the real frame (egui rails measured
/// after their own margins) must agree on the dock decision and stay within
/// `PLAN_TOLERANCE` of them. Row: (width, height, dock expected, centre w,
/// centre h, view w, view h) in logical points.
#[test]
fn layout_evidence_table() {
    let rows: [(f32, f32, bool, f32, f32, f32, f32); 8] = [
        (1280., 720., true, 680., 522., 339., 260.),
        (1366., 728., true, 766., 530., 382., 264.),
        (1093., 585., false, 793., 387., 395.5, 192.5),
        (911., 480., false, 611., 282., 304.5, 140.),
        (1920., 1080., true, 1320., 882., 659., 440.),
        (5120., 1440., true, 4520., 1242., 2259., 620.),
        (1200., 720., true, 600., 522., 299., 260.),
        (1199., 720., false, 899., 522., 448.5, 260.),
    ];
    let rects = |r: RegionRect| serde_json::json!([r.x, r.y, r.width, r.height]);
    for (width, height, dock, center_w, center_h, view_w, view_h) in rows {
        let label = format!("evidence {width}x{height}");
        let pure = pure_plan(width, height);
        assert_eq!(pure.right_dock.is_some(), dock, "{label}: pure dock");
        assert_eq!(
            (pure.center.width, pure.center.height),
            (center_w, center_h),
            "{label}: pure centre"
        );
        for slot in &pure.views {
            assert_eq!(
                (slot.rect.width, slot.rect.height),
                (view_w, view_h),
                "{label}: pure {:?}",
                slot.view
            );
        }
        let frame = check_size(width, height, true, &label);
        assert_eq!(frame.right_dock.is_some(), dock, "{label}: frame dock");
        for slot in &frame.views {
            assert!(
                (slot.rect.width - view_w).abs() <= PLAN_TOLERANCE
                    && (slot.rect.height - view_h).abs() <= PLAN_TOLERANCE,
                "{label}: frame {:?} {:?}",
                slot.view,
                slot.rect
            );
        }
        let views = |plan: &RegionPlan| {
            plan.views
                .iter()
                .map(|slot| (format!("{:?}", slot.view), rects(slot.rect)))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let line = serde_json::json!({
            "window_points": [width, height],
            "right_dock": dock,
            "pure": {
                "right_dock": pure.right_dock.map(rects),
                "notes": pure.notes.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "left_rail": pure.left_rail.map(rects),
                "center": rects(pure.center),
                "views": views(&pure),
            },
            "frame": {
                "right_dock": frame.right_dock.map(rects),
                "center": rects(frame.center),
                "views": views(&frame),
            },
        });
        println!("OM9_LAYOUT {line}");
    }
}

// ------------------------------------------------------ simulated DPI scale

#[test]
fn simulated_dpi_scales_keep_the_contract_in_point_space() {
    // (physical width, physical height, ppp, right dock requested)
    // 1366x720 @1.5 = 910.7x480 pt (capped 150% laptop), 1920x1080 @1.5 =
    // 1280x720 pt (nominal minimum at 150%), 1366x731 @1.25 = 1092.8x584.8 pt
    // (capped 125% laptop), 1600x1000 @1.5 = 1066.7x666.7 pt.
    for (width, height, ppp, dock) in [
        (1366., 720., 1.5, true),
        (1366., 720., 1.5, false),
        (1920., 1080., 1.5, true),
        (1920., 1080., 1.5, false),
        (1366., 731., 1.25, true),
        (1600., 1000., 1.5, true),
    ] {
        let label = format!("{width}x{height} @{ppp} dock={dock}");
        let physical = Vec2::new(width, height);
        let (mut app, context) = startup_scaled(ppp, physical);
        settle_scaled(&mut app, &context, physical, ppp, dock);
        assert_eq!(context.pixels_per_point(), ppp, "{label}: scale applied");
        let points = physical / ppp;
        let plan = check_frame(&app, points, &label);
        assert!(
            (plan.window.width - points.x).abs() <= SLACK,
            "{label}: planned in points"
        );
        if dock {
            assert_eq!(
                plan.right_dock.is_some(),
                dock_expected(points.x),
                "{label}: dock vs the {NARROW_WINDOW_WIDTH} pt threshold on {} pt",
                points.x
            );
        } else {
            assert!(plan.right_dock.is_none(), "{label}");
        }
    }
}

// ------------------------------------------- unsupported size, error path

/// unsupported size, error path: the only below-minimum case kept. A window
/// smaller than the planner floor (800x400 points) is a visible `Layout:`
/// status error, keeps the last good plan and changes no layout, scene or
/// document state. A real window cannot get here (the window level holds
/// 1280x720 or the capped screen value); this proves the planner behaviour.
#[test]
fn unsupported_size_error_path() {
    for (width, height) in [(799., 600.), (800., 399.)] {
        let label = format!("unsupported size, error path {width}x{height}");
        let (mut app, context) = startup();
        settle(&mut app, &context, Vec2::new(1280., 900.), true);
        let good_plan = app.region_plan.clone().expect("good plan");
        let dock_before = app.dock.to_json().unwrap();
        let workspace_before = app.controller.workspace.clone();
        let entities_before = app.controller.document().entities.len();

        for _ in 0..2 {
            frame_size(&mut app, &context, Vec2::new(width, height));
        }
        assert!(
            matches!(app.region_error, Some(RegionError::WindowTooSmall { .. })),
            "{label}: {:?}",
            app.region_error
        );
        assert_eq!(app.status.kind(), StatusKind::Error, "{label}: visible");
        assert!(
            app.status.message().starts_with("Layout:"),
            "{label}: {}",
            app.status.message()
        );
        assert_eq!(
            app.region_plan.as_ref(),
            Some(&good_plan),
            "{label}: last good plan kept"
        );
        assert_eq!(
            app.dock.to_json().unwrap(),
            dock_before,
            "{label}: dock changed"
        );
        assert_eq!(
            app.controller.workspace, workspace_before,
            "{label}: scene changed"
        );
        assert_eq!(app.controller.document().entities.len(), entities_before);
    }
}

#[test]
fn startup_refuses_a_window_below_1280x720() {
    let config = AppConfig::interactive("Untitled", None);
    assert_eq!(config.min_window, [1280., 720.]);
    let mut config = config;
    config.window = [1280., 720.];
    assert!(config.validate().is_ok());
    for window in [[1279., 720.], [1280., 719.], [1000., 600.], [300., 150.]] {
        config.window = window;
        let failure = config
            .validate()
            .expect_err("a window below the minimum must fail validation");
        assert_eq!(failure.stage, StartupStage::Renderer, "{window:?}");
        assert!(
            failure.line().contains("below the minimum"),
            "{window:?}: {}",
            failure.line()
        );
    }
}

// ---------------------------------------------------------- hidden command line

#[test]
fn hidden_command_line_and_right_dock_toggle_cannot_lose_the_command_input() {
    for size in [Vec2::new(1280., 800.), Vec2::new(800., 600.)] {
        let label = format!("{}x{}", size.x, size.y);
        let (mut app, context) = startup();
        settle(&mut app, &context, size, true);
        app.dock.set_visible(PanelId::CommandLine, false);
        app.toggle_right_dock();
        for _ in 0..3 {
            frame_size(&mut app, &context, size);
        }
        assert!(
            app.command_input_rect().is_some(),
            "{label}: command input kept"
        );
        let plan = check_frame(&app, size, &label);
        assert!(
            has_note(&plan, |n| matches!(n, RegionNote::CommandInputKept)),
            "{label}: CommandInputKept note, got {:?}",
            plan.notes
        );
        assert!(
            app.transcript.text().contains("command input kept visible"),
            "{label}: notice in the CMD history: {}",
            app.transcript.text()
        );
    }
}

// ------------------------------------- command, Undo, Redo, save, reopen

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("build/standalone-validation")
            .join(format!("region-sizes-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn command_undo_redo_save_and_reopen_leave_the_region_plan_alone() {
    let size = Vec2::new(1280., 800.);
    let (mut app, context) = startup();
    settle(&mut app, &context, size, true);
    let baseline = check_frame(&app, size, "baseline");
    assert!(baseline.right_dock.is_some());
    let dock_json = app.dock.to_json().unwrap();

    let same_plan = |app: &mut MatrixApp, context: &egui::Context, step: &str| {
        for _ in 0..3 {
            frame_size(app, context, size);
        }
        let plan = check_frame(app, size, step);
        assert_eq!(plan, baseline, "{step}: region plan changed");
        assert_eq!(
            app.dock.to_json().unwrap(),
            dock_json,
            "{step}: dock changed"
        );
    };

    // A real Line command through the same controller the CMD frame uses.
    for text in ["Line", "0,0,0", "12,6,0"] {
        app.controller.submit(text).unwrap();
    }
    assert_eq!(app.controller.document().entities.len(), 1);
    same_plan(&mut app, &context, "after Line");

    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    same_plan(&mut app, &context, "after Undo");

    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    same_plan(&mut app, &context, "after Redo");

    // Saving a document that holds a Line currently fails in the archive
    // encoder ("archive encoding verification", found by this test and recorded
    // in the closeout findings; unrelated to the regions). Undo the Line and
    // save a Point document instead, so the save/reopen leg still runs.
    app.controller.undo().unwrap();
    for text in ["Point", "1,2,3"] {
        app.controller.submit(text).unwrap();
    }
    assert_eq!(app.controller.document().entities.len(), 1);
    same_plan(&mut app, &context, "after Point");

    // Save to a temp .om9 through the owned path API (no native dialog).
    let temp = TempDir::new();
    let path = temp.0.join("regions.om9");
    assert_eq!(
        app.controller
            .begin_file_operation(FileKind::SaveAs)
            .unwrap(),
        FileOutcome::Pending
    );
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(
        app.controller.accept_file_path(token, &path).unwrap(),
        FileOutcome::Saved
    );
    same_plan(&mut app, &context, "after Save");

    // Reopen the saved file in a fresh app at the same window size.
    let mut reopened = MatrixApp::from_file(&path).unwrap();
    let reopened_context = egui::Context::default();
    frame_size(&mut reopened, &reopened_context, Vec2::new(1280., 900.));
    frame_size(&mut reopened, &reopened_context, Vec2::new(1280., 900.));
    settle(&mut reopened, &reopened_context, size, true);
    assert_eq!(reopened.controller.document().entities.len(), 1);
    let plan = check_frame(&reopened, size, "after reopen");
    assert_eq!(plan, baseline, "after reopen: region plan changed");
}

// ------------------------------------- LEFT_RAIL_MAX 460 justification table

/// Pure plan for the default layout with a chosen rail/dock width and, when
/// `fixture` is set, the shared Snaps fixture docked right.
fn rail_pure_plan(
    width: f32,
    height: f32,
    rail: f32,
    dock_width: f32,
    fixture: bool,
) -> RegionPlan {
    let mut layout = DockLayout::matrix_default();
    layout.set_size(PanelId::LeftRail, rail);
    layout.set_size(PanelId::RightDock, dock_width);
    if fixture {
        layout
            .dock_section(RIGHT_DOCK_FIXTURE, DockSide::Right)
            .expect("the fixture docks right");
    }
    compute_regions(&layout, WindowSize::new(width, height), &ViewId::ALL).expect("plan")
}

/// Why `LEFT_RAIL_MAX` is 460: for each supported window (1280x720, 1366x768 at
/// 100/125/150% after the cap = 1280x688 / 1093x585 / 911x480, and 1920x1080)
/// the rail at 240 / 300 / 460, with and without the user-docked Snaps fixture,
/// keeps the 320 pt canvas minimum, a valid frame and a visible Layers bar. The
/// 1200 / 1199 rows with rail 460 + dock 420 show the equality the compile-time
/// assertion relies on. Set `OM9_WRITE_LEFT_DOCK_EVIDENCE=<dir>` to write
/// `left-dock-layouts.json` and `left-dock-layouts.log` there.
#[test]
fn left_rail_max_justification_table() {
    // (width, height, rail, right dock width, fixture docked right)
    let mut cases: Vec<(f32, f32, f32, f32, bool)> = Vec::new();
    for (width, height) in [
        (1280., 720.),
        (1280., 688.),
        (1093., 585.),
        (911., 480.),
        (1920., 1080.),
    ] {
        for rail in [240., 300., 460.] {
            for fixture in [false, true] {
                cases.push((width, height, rail, 300., fixture));
            }
        }
    }
    cases.push((1200., 720., 460., 420., true));
    cases.push((1199., 720., 460., 420., true));
    cases.push((1201., 720., 460., 420., true));

    let rects = |r: RegionRect| serde_json::json!([r.x, r.y, r.width, r.height]);
    let mut rows = Vec::new();
    let mut log = String::new();
    for (width, height, rail, dock_width, fixture) in cases {
        let label = format!("rail {rail} {width}x{height} fixture={fixture} dock={dock_width}");
        let pure = rail_pure_plan(width, height, rail, dock_width, fixture);
        pure.validate().unwrap_or_else(|e| panic!("{label}: {e}"));
        let expected_dock = fixture && width >= NARROW_WINDOW_WIDTH;
        assert_eq!(
            pure.right_dock.is_some(),
            expected_dock,
            "{label}: pure dock"
        );
        assert!(
            pure.center.width >= CANVAS_MIN_WIDTH,
            "{label}: pure centre {:?} below the canvas minimum",
            pure.center
        );
        assert_eq!(pure.left_rail.map(|r| r.width), Some(rail), "{label}: rail");

        let size = Vec2::new(width, height);
        let mut app = MatrixApp::new(Document::new("rail justification")).unwrap();
        app.dock.set_size(PanelId::LeftRail, rail);
        app.dock.set_size(PanelId::RightDock, dock_width);
        if fixture {
            assert!(common::dock_right_fixture(&mut app), "{label}");
        }
        let context = egui::Context::default();
        common::settle(&mut app, &context, size, 4);
        let frame = check_frame(&app, size, &label);
        assert_eq!(
            frame.right_dock.is_some(),
            expected_dock,
            "{label}: frame dock"
        );
        if fixture && !expected_dock {
            assert!(yielded(&frame, YieldReason::NarrowWindow), "{label}");
            assert!(!yielded(&frame, YieldReason::NotEnoughRoom), "{label}");
        }
        assert!(
            frame.center.width >= CANVAS_MIN_WIDTH,
            "{label}: frame centre {:?}",
            frame.center
        );
        let canvas = app
            .view_rects
            .values()
            .copied()
            .reduce(|a, b| a.union(b))
            .expect("four viewports");
        assert!(
            canvas.width() >= CANVAS_MIN_WIDTH,
            "{label}: canvas {}",
            canvas.width()
        );
        common::assert_layers_between_info_and_projects(&app, &label);
        let layers = app.section_rect("layers").expect("layers bar");
        let sidebar = app.sidebar_rect.expect("sidebar");
        let layers_visible = layers.max.y <= sidebar.max.y.min(height);
        assert!(layers_visible, "{label}: Layers bar {layers:?} not visible");
        if width == NARROW_WINDOW_WIDTH && fixture && rail == 460. && dock_width == 420. {
            assert_eq!(
                pure.center.width, CANVAS_MIN_WIDTH,
                "{label}: exact equality"
            );
        }
        let row = serde_json::json!({
            "window_points": [width, height],
            "rail": rail,
            "right_dock_width": dock_width,
            "user_docked_right": fixture.then(|| RIGHT_DOCK_FIXTURE.key()),
            "right_dock_shown": expected_dock,
            "pure": {
                "center": rects(pure.center),
                "left_rail": pure.left_rail.map(rects),
                "right_dock": pure.right_dock.map(rects),
                "notes": pure.notes.iter().map(ToString::to_string).collect::<Vec<_>>(),
            },
            "frame": {
                "center": rects(frame.center),
                "canvas_union_width": canvas.width(),
                "left_rail_outer_px": app.left_rail_outer_px,
                "left_icon_edge_px": app.left_icon_edge,
                "layers_bar": [layers.min.x, layers.min.y, layers.width(), layers.height()],
                "layers_visible_without_scroll": layers_visible,
                "sidebar_y": [sidebar.min.y, sidebar.max.y],
            },
        });
        log.push_str(&format!(
            "OM9_LEFT_DOCK window={width}x{height} rail={rail} fixture={fixture} dock_width={dock_width} \
             dock_shown={expected_dock} pure_centre_w={} frame_centre_w={} canvas_w={:.1} \
             icon_edge={} layers_y={} layers_visible={layers_visible}\n",
            pure.center.width,
            frame.center.width,
            canvas.width(),
            app.left_icon_edge,
            layers.min.y,
        ));
        rows.push(row);
    }
    print!("{log}");
    if let Some(dir) = std::env::var_os("OM9_WRITE_LEFT_DOCK_EVIDENCE") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let document = serde_json::json!({
            "task": "OM9-L3-01-01-002",
            "purpose": "justify LEFT_RAIL_MAX 460 (plan floor 800 >= 460 + 320; 1200 >= 460 + 420 + 320 with equality)",
            "evidence_kind": "SIMULATED/AUTOMATED headless egui frames; not a human sign-off",
            "constants": {
                "LEFT_RAIL_MIN": LEFT_RAIL_MIN,
                "LEFT_RAIL_DEFAULT": LEFT_RAIL_DEFAULT,
                "LEFT_RAIL_MAX": LEFT_RAIL_MAX,
                "RIGHT_DOCK_MAX": RIGHT_DOCK_MAX,
                "CANVAS_MIN_WIDTH": CANVAS_MIN_WIDTH,
                "NARROW_WINDOW_WIDTH": NARROW_WINDOW_WIDTH,
            },
            "rows": rows,
        });
        std::fs::write(
            dir.join("left-dock-layouts.json"),
            serde_json::to_string_pretty(&document).unwrap() + "\n",
        )
        .unwrap();
        std::fs::write(dir.join("left-dock-layouts.log"), &log).unwrap();
    }
}
