// SPDX-License-Identifier: LGPL-2.1-or-later
//! Workbench docking regions evidence for plan task OM9-L3-01-01-002.
//!
//! Two layers:
//! 1. the independent fixtures `positive.json` / `negative.json` (hand-computed
//!    expectations) are replayed against the pure region model;
//! 2. the real `MatrixApp::show` frame is driven at a normal, a narrow and a
//!    too-small window to prove the same contract through the egui shell.
use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::dock_layout::{DockLayout, DockSide, PanelId, SectionId};
use om9_app::status_surface::StatusKind;
use om9_app::ui::MatrixApp;
use om9_app::workbench_regions::{
    CommandInputHost, RegionError, RegionNote, RegionPlan, RegionRect, ViewportSceneHost,
    WindowSize, compute_regions, compute_regions_for,
};
use om9_core::model::Document;
use om9_scene::workspace::ViewId;
use serde_json::Value;

mod common;
use common::dock_right_fixture;

const POSITIVE: &str =
    include_str!("../../../implementation/fixtures/M01/OM9-L3-01-01-002/positive.json");
const NEGATIVE: &str =
    include_str!("../../../implementation/fixtures/M01/OM9-L3-01-01-002/negative.json");

// ---------------------------------------------------------------- fixtures

struct FixedScene(Vec<ViewId>);

impl ViewportSceneHost for FixedScene {
    fn visible_views(&self) -> Vec<ViewId> {
        self.0.clone()
    }
}

struct FixedCommand(bool);

impl CommandInputHost for FixedCommand {
    fn command_input_ready(&self) -> bool {
        self.0
    }
}

fn number(value: &Value) -> f32 {
    match value {
        Value::Number(n) => n.as_f64().expect("finite number") as f32,
        Value::String(s) => match s.as_str() {
            "NaN" => f32::NAN,
            "inf" => f32::INFINITY,
            "-inf" => f32::NEG_INFINITY,
            other => panic!("unknown numeric token {other}"),
        },
        other => panic!("not a number: {other}"),
    }
}

fn window_of(case: &Value) -> WindowSize {
    let size = case["window"].as_array().expect("window");
    WindowSize::new(number(&size[0]), number(&size[1]))
}

fn views_of(case: &Value) -> Vec<ViewId> {
    match case.get("views") {
        Some(list) => serde_json::from_value(list.clone()).expect("view ids"),
        None => ViewId::ALL.to_vec(),
    }
}

fn layout_of(case: &Value) -> DockLayout {
    let mut layout = DockLayout::matrix_default();
    let Some(spec) = case.get("layout") else {
        return layout;
    };
    for key in spec
        .get("dock_right")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = SectionId::from_key(key.as_str().unwrap()).expect("section key");
        layout.dock_section(id, DockSide::Right).unwrap();
    }
    for key in spec
        .get("hide_panels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = PanelId::from_key(key.as_str().unwrap()).expect("panel key");
        layout.set_visible(id, false);
    }
    layout
}

fn rect_of(value: &Value) -> RegionRect {
    let v = value.as_array().expect("[x, y, w, h]");
    RegionRect::new(number(&v[0]), number(&v[1]), number(&v[2]), number(&v[3]))
}

fn note_names(plan: &RegionPlan) -> Vec<&'static str> {
    plan.notes
        .iter()
        .map(|note| match note {
            RegionNote::RightDockYielded { .. } => "RightDockYielded",
            RegionNote::LeftRailShrunk { .. } => "LeftRailShrunk",
            RegionNote::CommandInputKept => "CommandInputKept",
        })
        .collect()
}

fn error_name(error: &RegionError) -> &'static str {
    match error {
        RegionError::InvalidWindow { .. } => "InvalidWindow",
        RegionError::WindowTooSmall { .. } => "WindowTooSmall",
        RegionError::CenterTooSmall { .. } => "CenterTooSmall",
        RegionError::CommandInputUnavailable => "CommandInputUnavailable",
        RegionError::ViewSetInvalid { .. } => "ViewSetInvalid",
        RegionError::Overlap { .. } => "Overlap",
        RegionError::OutOfBounds { .. } => "OutOfBounds",
        RegionError::NonFinite { .. } => "NonFinite",
        RegionError::CommandInputTooSmall { .. } => "CommandInputTooSmall",
        RegionError::ViewSlotInvalid { .. } => "ViewSlotInvalid",
    }
}

fn check_rects(case_id: &str, plan: &RegionPlan, expect: &Value) {
    let pairs = [
        ("menu_bar", Some(plan.menu_bar)),
        ("center", Some(plan.center)),
        ("command_input", Some(plan.command_input)),
        ("toolbar", Some(plan.toolbar)),
        ("status_bar", Some(plan.status_bar)),
        ("left_rail", plan.left_rail),
        ("right_dock", plan.right_dock),
    ];
    for (key, actual) in pairs {
        let Some(expected) = expect.get(key) else {
            continue;
        };
        if expected.is_null() {
            assert_eq!(actual, None, "{case_id}: {key} must be absent");
        } else {
            assert_eq!(actual, Some(rect_of(expected)), "{case_id}: {key}");
        }
    }
    if let Some(views) = expect.get("views").and_then(Value::as_object) {
        assert_eq!(plan.views.len(), views.len(), "{case_id}: view count");
        for (name, expected) in views {
            let view: ViewId = serde_json::from_value(Value::String(name.clone())).unwrap();
            let slot = plan
                .views
                .iter()
                .find(|slot| slot.view == view)
                .unwrap_or_else(|| panic!("{case_id}: missing view {name}"));
            assert_eq!(slot.rect, rect_of(expected), "{case_id}: view {name}");
        }
    }
    if let Some(notes) = expect.get("notes").and_then(Value::as_array) {
        let expected: Vec<&str> = notes.iter().map(|n| n.as_str().unwrap()).collect();
        assert_eq!(note_names(plan), expected, "{case_id}: notes");
    }
}

#[test]
fn positive_fixture_cases_match_the_hand_computed_regions() {
    let fixture: Value = serde_json::from_str(POSITIVE).unwrap();
    assert_eq!(fixture["schema"], "om9.m01.fixture.v1");
    assert_eq!(fixture["l3"], "OM9-L3-01-01-002");
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() >= 4);
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let layout = layout_of(case);
        let before = layout.to_json().unwrap();
        let plan = compute_regions(&layout, window_of(case), &views_of(case))
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        plan.validate()
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        check_rects(id, &plan, &case["expect"]);
        assert_eq!(layout.to_json().unwrap(), before, "{id}: layout unchanged");
    }
}

#[test]
fn negative_fixture_cases_are_explicit_and_leave_the_layout_unchanged() {
    let fixture: Value = serde_json::from_str(NEGATIVE).unwrap();
    assert_eq!(fixture["schema"], "om9.m01.fixture.v1");
    let mut replayed = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if case.get("sweep").is_some() {
            continue;
        }
        let id = case["id"].as_str().unwrap();
        let expect = &case["expect"];
        let layout = layout_of(case);
        let before = layout.to_json().unwrap();
        let ready = case
            .get("host")
            .and_then(|h| h.get("command_input_ready"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let views = views_of(case);
        let result = if case.get("host").is_some() {
            compute_regions_for(
                &layout,
                window_of(case),
                &FixedScene(views),
                &FixedCommand(ready),
            )
        } else {
            compute_regions(&layout, window_of(case), &views)
        };
        match expect["result"].as_str().unwrap() {
            "ok" => {
                let plan = result.unwrap_or_else(|error| panic!("{id}: {error}"));
                plan.validate().unwrap();
                check_rects(id, &plan, expect);
                if expect["command_input_overlaps_nothing"] == Value::Bool(true) {
                    let command = plan.command_input;
                    for other in [
                        Some(plan.menu_bar),
                        plan.left_rail,
                        Some(plan.center),
                        plan.right_dock,
                        Some(plan.toolbar),
                        Some(plan.status_bar),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        assert!(!command.overlaps(&other), "{id}");
                    }
                }
            }
            "error" => {
                let error = result.expect_err(id);
                assert_eq!(
                    error_name(&error),
                    expect["error"].as_str().unwrap(),
                    "{id}"
                );
                if let RegionError::WindowTooSmall {
                    min_width,
                    min_height,
                    ..
                } = error
                {
                    let min = expect["min_window"].as_array().unwrap();
                    assert_eq!((min_width, min_height), (number(&min[0]), number(&min[1])));
                }
                if let RegionError::ViewSetInvalid { count } = error {
                    assert_eq!(count as u64, expect["count"].as_u64().unwrap(), "{id}");
                }
            }
            other => panic!("{id}: unknown result {other}"),
        }
        if expect["layout_unchanged"] == Value::Bool(true) {
            assert_eq!(layout.to_json().unwrap(), before, "{id}: layout unchanged");
        }
        replayed += 1;
    }
    assert!(replayed >= 15, "replayed {replayed} negative cases");
}

#[test]
fn negative_fixture_sweep_never_overlaps_or_hides_the_command_input() {
    let fixture: Value = serde_json::from_str(NEGATIVE).unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case.get("sweep").is_some())
        .expect("sweep case");
    let axis = |name: &str| {
        let a = &case["sweep"][name];
        (
            a["from"].as_u64().unwrap() as usize,
            a["to"].as_u64().unwrap() as usize,
            a["step"].as_u64().unwrap() as usize,
        )
    };
    let (w_from, w_to, w_step) = axis("width");
    let (h_from, h_to, h_step) = axis("height");
    let mut ok_plans = 0;
    let mut errors = 0;
    for name in case["layouts"].as_array().unwrap() {
        let layout = match name.as_str().unwrap() {
            "default" => DockLayout::matrix_default(),
            "right-dock-main-menu" => {
                let mut layout = DockLayout::matrix_default();
                layout
                    .dock_section(SectionId::MainMenu, DockSide::Right)
                    .unwrap();
                layout
            }
            "command-line-hidden" => {
                let mut layout = DockLayout::matrix_default();
                layout.set_visible(PanelId::CommandLine, false);
                layout
            }
            other => panic!("unknown sweep layout {other}"),
        };
        for width in (w_from..=w_to).step_by(w_step) {
            for height in (h_from..=h_to).step_by(h_step) {
                let size = WindowSize::new(width as f32, height as f32);
                match compute_regions(&layout, size, &ViewId::ALL) {
                    Ok(plan) => {
                        ok_plans += 1;
                        let command = plan.command_input;
                        assert!(command.height >= 58., "{width}x{height}");
                        for other in [
                            plan.left_rail,
                            Some(plan.center),
                            plan.right_dock,
                            Some(plan.toolbar),
                            Some(plan.status_bar),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            assert!(!command.overlaps(&other), "{width}x{height}");
                        }
                    }
                    Err(error) => {
                        errors += 1;
                        assert!(
                            matches!(error, RegionError::WindowTooSmall { .. }),
                            "{width}x{height}: {error}"
                        );
                    }
                }
            }
        }
    }
    assert!(ok_plans > 0 && errors > 0);
}

// ------------------------------------------------------------- egui frames

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
    let mut app = MatrixApp::new(Document::new("workbench regions")).unwrap();
    let context = egui::Context::default();
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    (app, context)
}

fn assert_no_overlap(app: &MatrixApp) {
    let rects: Vec<(ViewId, Rect)> = app.view_rects.iter().map(|(v, r)| (*v, *r)).collect();
    for (index, (a, rect_a)) in rects.iter().enumerate() {
        for (b, rect_b) in &rects[index + 1..] {
            assert!(
                !rect_a.shrink(0.01).intersects(*rect_b),
                "viewports {a:?} and {b:?} overlap"
            );
        }
    }
}

#[test]
fn normal_window_shows_left_rail_four_viewports_and_the_right_dock() {
    let (mut app, context) = startup();
    assert!(dock_right_fixture(&mut app));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));

    let left = app.sidebar_rect.expect("left rail renders");
    let right = app.right_dock_rect.expect("right dock renders");
    assert_eq!(app.view_rects.len(), 4, "four viewports");
    assert_no_overlap(&app);
    let command = app.command_input_rect().expect("command input renders");
    for rect in app.view_rects.values() {
        assert!(left.max.x <= rect.min.x + 0.5, "left rail overlaps a view");
        assert!(
            rect.max.x <= right.min.x + 0.5,
            "right dock overlaps a view"
        );
        assert!(!command.intersects(*rect), "command input overlaps a view");
    }
    assert!(app.region_error.is_none());
    let plan = app.region_plan.as_ref().expect("a validated plan");
    plan.validate().unwrap();
    assert!(plan.left_rail.is_some() && plan.right_dock.is_some());
    assert_eq!(plan.views.len(), 4);
    assert!(
        !app.status.message().starts_with("Layout:"),
        "no region notice for a normal window: {}",
        app.status.message()
    );
}

#[test]
fn the_plan_matches_the_regions_egui_actually_lays_out() {
    let (mut app, context) = startup();
    assert!(dock_right_fixture(&mut app));
    for _ in 0..3 {
        frame_size(&mut app, &context, Vec2::new(1600., 1000.));
    }
    let plan = app.region_plan.clone().expect("plan");
    // The model is exact; egui panels carry frame margins, so allow a few pixels.
    let near = |a: f32, b: f32, what: &str| {
        assert!((a - b).abs() <= 8., "{what}: plan {a} vs frame {b}");
    };
    let left = app.sidebar_rect.expect("left rail");
    let left_plan = plan.left_rail.expect("left rail in the plan");
    near(left_plan.width, left.width(), "left rail width");
    let right = app.right_dock_rect.expect("right dock");
    let right_plan = plan.right_dock.expect("right dock in the plan");
    near(right_plan.width, right.width(), "right dock width");
    for slot in &plan.views {
        let frame = app.view_rects[&slot.view];
        near(slot.rect.x, frame.min.x, "view x");
        near(slot.rect.y, frame.min.y, "view y");
        near(slot.rect.width, frame.width(), "view width");
        near(slot.rect.height, frame.height(), "view height");
    }
}

#[test]
fn narrow_window_yields_the_right_dock_and_keeps_the_command_input() {
    let (mut app, context) = startup();
    assert!(dock_right_fixture(&mut app));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    assert!(app.right_dock_rect.is_some());

    frame_size(&mut app, &context, Vec2::new(800., 600.));
    frame_size(&mut app, &context, Vec2::new(800., 600.));
    assert!(app.right_dock_rect.is_none(), "dock yielded");
    let plan = app.region_plan.as_ref().expect("plan");
    assert_eq!(plan.right_dock, None);
    assert!(
        plan.notes
            .iter()
            .any(|note| matches!(note, RegionNote::RightDockYielded { .. })),
        "the yield is recorded, not silent"
    );
    assert!(
        app.transcript.text().contains("Layout: right dock hidden"),
        "the yield is visible in the CMD history: {}",
        app.transcript.text()
    );
    let command = app.command_input_rect().expect("command input kept");
    assert!(command.width() > 0. && command.height() > 0.);
    let panel = plan.command_input;
    assert!(
        panel.height >= 58.,
        "command panel keeps its minimum height"
    );
    assert!(command.min.x >= panel.x - 0.5 && command.max.x <= panel.right() + 0.5);
    for rect in app.view_rects.values() {
        assert!(
            !command.intersects(*rect),
            "command input covered by a view"
        );
    }
    assert_eq!(app.view_rects.len(), 4);
    assert_no_overlap(&app);

    // Widening brings the dock back and clears the notice that this module set.
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    assert!(app.right_dock_rect.is_some());
    assert!(app.region_error.is_none());
    assert!(
        app.region_plan
            .as_ref()
            .is_some_and(|plan| plan.notes.is_empty()),
        "the notice is gone with the cause"
    );
}

#[test]
fn too_small_window_reports_visibly_and_changes_nothing_else() {
    let (mut app, context) = startup();
    assert!(dock_right_fixture(&mut app));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    let good_plan = app.region_plan.clone().expect("good plan");
    let dock_before = app.dock.to_json().unwrap();
    let workspace_before = app.controller.workspace.clone();
    let entities_before = app.controller.document().entities.len();

    frame_size(&mut app, &context, Vec2::new(300., 150.));
    frame_size(&mut app, &context, Vec2::new(300., 150.));
    assert!(matches!(
        app.region_error,
        Some(RegionError::WindowTooSmall { .. })
    ));
    assert_eq!(app.status.kind(), StatusKind::Error, "visible, not silent");
    assert!(app.status.message().contains("below the minimum"));
    assert_eq!(
        app.region_plan.as_ref(),
        Some(&good_plan),
        "last good plan kept"
    );
    assert_eq!(app.dock.to_json().unwrap(), dock_before, "dock unchanged");
    assert_eq!(
        app.controller.workspace, workspace_before,
        "scene state unchanged"
    );
    assert_eq!(app.controller.document().entities.len(), entities_before);

    // Recovery: a usable window clears the error this module raised.
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    assert!(app.region_error.is_none());
    assert_eq!(app.status.kind(), StatusKind::Ready);
    assert_eq!(app.dock.to_json().unwrap(), dock_before);
}

#[test]
fn an_unrelated_failure_is_not_cleared_by_the_region_check() {
    let (mut app, context) = startup();
    app.status.error("Unknown command: Foo");
    frame_size(&mut app, &context, Vec2::new(1280., 900.));
    frame_size(&mut app, &context, Vec2::new(1600., 1000.));
    assert_eq!(app.status.kind(), StatusKind::Error);
    assert_eq!(app.status.message(), "Unknown command: Foo");
}
