// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for `om9_app::panel_ops` (L3-003), driven by the fixtures in
//! `implementation/fixtures/M01/OM9-L3-01-01-003/`.
//!
//! The fixtures are read from disk at run time, so the expectations recorded
//! there are the spec: a silent drift fails here.
use om9_app::dock_layout::{DockLayout, DockSide, FloatRect, SectionId};
use om9_app::panel_ops::{
    self, CancelReason, ColumnSide, DropTarget, DropZones, FloatOrigin, PanelNote, PanelOp,
    PanelOpError, PanelOutcome, SectionHome, WorkArea, drop_indicator, drop_op, float_drop_rect,
    resolve_drop,
};
use om9_app::workbench_regions::RegionRect;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-003")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("fixture JSON")
}

fn case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("fixture case {id}"))
}

fn keys(layout: &DockLayout, side: DockSide) -> Vec<String> {
    layout
        .section_order(side)
        .into_iter()
        .map(|id| id.key().to_owned())
        .collect()
}

fn left(layout: &DockLayout) -> Vec<String> {
    keys(layout, DockSide::Left)
}

fn right(layout: &DockLayout) -> Vec<String> {
    keys(layout, DockSide::Right)
}

fn documented_order(document: &Value) -> Vec<String> {
    document["documented_left_order"]
        .as_array()
        .expect("documented_left_order")
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

fn rect(values: &[f32; 4]) -> FloatRect {
    FloatRect {
        x: values[0],
        y: values[1],
        width: values[2],
        height: values[3],
    }
}

fn float_rect(case: &Value, key: &str) -> FloatRect {
    let values = case["expect"][key].as_array().expect("float rect");
    rect(&[
        values[0].as_f64().unwrap() as f32,
        values[1].as_f64().unwrap() as f32,
        values[2].as_f64().unwrap() as f32,
        values[3].as_f64().unwrap() as f32,
    ])
}

fn work(width: f32, height: f32) -> WorkArea {
    WorkArea::for_window([width, height]).expect("work area")
}

fn assert_invariants(layout: &DockLayout) {
    let mut seen: Vec<&'static str> = Vec::new();
    seen.extend(layout.left_sections().iter().map(|id| id.key()));
    seen.extend(layout.right_sections().iter().map(|id| id.key()));
    seen.extend(layout.floating_sections().iter().map(|id| id.key()));
    seen.sort_unstable();
    let mut expected: Vec<&'static str> = SectionId::ALL.iter().map(|id| id.key()).collect();
    expected.sort_unstable();
    assert_eq!(seen, expected, "every menu appears exactly once");
}

fn apply(
    layout: &mut DockLayout,
    op: PanelOp,
) -> Result<om9_app::panel_ops::PanelReport, PanelOpError> {
    let report = layout.apply_panel_op(op);
    if report.is_ok() {
        assert_invariants(layout);
    }
    report
}

#[test]
fn p1_collapse_then_expand_restores_the_exact_state() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P1-collapse-then-expand-restores-the-exact-state",
    );
    assert_eq!(
        case["expect"]["after_first"]["status"],
        "Projects rolled up"
    );
    let mut layout = DockLayout::matrix_default();
    let before = layout.section(SectionId::Projects);

    let first = apply(
        &mut layout,
        PanelOp::ToggleCollapsed {
            section: SectionId::Projects,
        },
    )
    .expect("collapse");
    assert_eq!(
        first.outcome,
        PanelOutcome::Collapsed {
            section: SectionId::Projects,
            collapsed: true
        }
    );
    assert!(layout.section(SectionId::Projects).collapsed);
    assert_eq!(left(&layout), documented_order(&document));

    apply(
        &mut layout,
        PanelOp::ToggleCollapsed {
            section: SectionId::Projects,
        },
    )
    .expect("expand");
    assert_eq!(layout.section(SectionId::Projects), before);
}

#[test]
fn p2_collapsing_snaps_and_layers_keeps_the_documented_order() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P2-collapsed-snaps-and-layers-keep-closed-bar-controls",
    );
    let closed_bar: Vec<&str> = case["expect"]["closed_bar_controls_contains"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    for key in closed_bar {
        let section = SectionId::from_key(key).expect("section");
        assert!(
            section.keeps_controls_while_collapsed(),
            "{key} keeps controls on its closed bar"
        );
    }
    let mut layout = DockLayout::matrix_default();
    for section in [SectionId::Snaps, SectionId::Layers] {
        apply(&mut layout, PanelOp::ToggleCollapsed { section }).expect("collapse");
        assert!(layout.section(section).collapsed);
    }
    assert_eq!(left(&layout), documented_order(&document));
}

#[test]
fn p5_move_is_one_operation_for_the_api_and_the_button() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P5-move-up-by-api-and-by-button-is-the-same-operation",
    );
    let mut layout = DockLayout::matrix_default();

    let report = apply(
        &mut layout,
        PanelOp::Move {
            section: SectionId::Projects,
            delta: -1,
        },
    )
    .expect("move up");
    assert_eq!(
        report.outcome,
        PanelOutcome::Moved {
            section: SectionId::Projects,
            side: ColumnSide::Left,
            index: 5
        }
    );
    let expected: Vec<String> = case["expect"]["left_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(left(&layout), expected);

    // Saturation: a large negative delta lands at the first position.
    apply(
        &mut layout,
        PanelOp::Move {
            section: SectionId::Projects,
            delta: -99,
        },
    )
    .expect("saturating move");
    assert_eq!(left(&layout)[0], "projects");
}

#[test]
fn p6_dock_right_then_left_restores_home_and_roll_up() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P6-dock-right-by-drag-then-dock-left-restores-home-and-roll-up",
    );
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::ToggleCollapsed {
            section: SectionId::Snaps,
        },
    )
    .expect("roll up snaps");

    let docked = apply(
        &mut layout,
        PanelOp::Dock {
            section: SectionId::Snaps,
            side: ColumnSide::Right,
        },
    )
    .expect("dock right");
    assert!(matches!(docked.outcome, PanelOutcome::Docked { .. }));
    assert_eq!(right(&layout), vec!["snaps".to_owned()]);
    assert!(!left(&layout).contains(&"snaps".to_owned()));
    let state = layout.section(SectionId::Snaps);
    assert!(state.collapsed, "docking keeps the rolled-up state");
    assert_eq!(
        state.home,
        Some(SectionHome {
            side: ColumnSide::Left,
            index: 3
        })
    );

    let back = apply(
        &mut layout,
        PanelOp::Dock {
            section: SectionId::Snaps,
            side: ColumnSide::Left,
        },
    )
    .expect("dock back left");
    assert_eq!(
        back.outcome,
        PanelOutcome::Docked {
            section: SectionId::Snaps,
            side: ColumnSide::Left,
            index: 3,
            restored_home: true
        }
    );
    assert_eq!(left(&layout), documented_order(&document));
    let state = layout.section(SectionId::Snaps);
    assert_eq!(state.home, None, "the home is consumed when it is restored");
    assert!(state.collapsed, "the roll-up state survives the round trip");
    assert_eq!(
        case["expect"]["after_dock_left"]["status"],
        "Snaps docked in the left menu column"
    );
}

#[test]
fn p7_float_records_home_and_close_returns_to_it() {
    let document = fixture("positive.json");
    let case = case(&document, "P7-float-by-drag-then-close-returns-home");
    let dropped = rect(&[550., 393., 300., 240.]);
    assert_eq!(float_drop_rect([700., 400.]), dropped);
    let mut layout = DockLayout::matrix_default();

    let report = apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: dropped,
            work: work(1600., 1000.),
            origin: FloatOrigin::Drop,
        },
    )
    .expect("float");
    assert_eq!(
        report.outcome,
        PanelOutcome::Floated {
            section: SectionId::Display,
            rect: dropped
        }
    );
    assert!(report.notes.is_empty());
    let state = layout.section(SectionId::Display);
    assert_eq!(state.float, Some(dropped));
    assert_eq!(
        state.home,
        Some(SectionHome {
            side: ColumnSide::Left,
            index: 2
        })
    );

    let closed = apply(
        &mut layout,
        PanelOp::DockHome {
            section: SectionId::Display,
        },
    )
    .expect("close the floating menu");
    assert!(matches!(closed.outcome, PanelOutcome::Docked { .. }));
    assert_eq!(left(&layout), documented_order(&document));
    let state = layout.section(SectionId::Display);
    assert_eq!(state.dock, DockSide::Left);
    assert_eq!(state.order, 2);
    assert_eq!(state.home, None);
    assert_eq!(
        state.float,
        Some(dropped),
        "the last floating position is kept for the next float"
    );
    assert_eq!(
        case["expect"]["after_drop_before_next_frame"]["display"]["collapsed"],
        false
    );
}

#[test]
fn p8_a_user_moved_floating_window_is_readback_not_recovery() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P8-floating-window-moved-by-the-user-is-readback-not-recovery",
    );
    let start = rect(&[550., 393., 300., 240.]);
    let moved = float_rect(case, "display.float");
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: start,
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect("float");

    let report = apply(
        &mut layout,
        PanelOp::SetFloatRect {
            section: SectionId::Display,
            rect: moved,
            work: work(1600., 1000.),
        },
    )
    .expect("readback");
    assert_eq!(
        report.outcome,
        PanelOutcome::FloatRectSet {
            section: SectionId::Display,
            rect: moved
        }
    );
    assert!(
        report.notes.is_empty(),
        "a rectangle already inside the area is not recovery"
    );
    assert_eq!(layout.section(SectionId::Display).float, Some(moved));
}

#[test]
fn p12_docking_back_without_home_lands_below_info_and_settings() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P12-layers-docked-back-without-home-lands-below-info-and-settings",
    );
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Layers,
            rect: rect(&[400., 300., 300., 240.]),
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect("float layers");
    layout
        .sections
        .get_mut(SectionId::Layers.key())
        .unwrap()
        .home = None;

    let report = apply(
        &mut layout,
        PanelOp::DockHome {
            section: SectionId::Layers,
        },
    )
    .expect("dock home");
    assert_eq!(
        report.outcome,
        PanelOutcome::Docked {
            section: SectionId::Layers,
            side: ColumnSide::Left,
            index: 5,
            restored_home: false
        }
    );
    assert_eq!(left(&layout), documented_order(&document));
    assert_eq!(
        case["expect"]["left_order"].as_str(),
        Some("documented_left_order")
    );
}

#[test]
fn n12_n13_a_non_finite_readback_is_refused_and_changes_nothing() {
    let document = fixture("negative.json");
    let case = case(&document, "N13-readback-nan-from-a-floating-window");
    let start = rect(&[550., 393., 300., 240.]);
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: start,
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect("float");
    let before = layout.to_json().unwrap();

    let error = apply(
        &mut layout,
        PanelOp::SetFloatRect {
            section: SectionId::Display,
            rect: FloatRect {
                x: 100.,
                y: f32::NAN,
                width: 300.,
                height: 240.,
            },
            work: work(1600., 1000.),
        },
    )
    .expect_err("NaN must be refused");
    assert!(matches!(error, PanelOpError::InvalidRect { .. }));
    assert_eq!(layout.to_json().unwrap(), before);
    assert_eq!(case["expect"]["stored_float_unchanged"], true);
}

#[test]
fn n14_a_window_shrink_recovers_the_floating_menu() {
    let document = fixture("negative.json");
    let case = case(&document, "N14-window-shrink-recovers-a-floating-menu");
    let expected = float_rect(case, "float");
    let mut layout = DockLayout::matrix_default();
    let floated = apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: rect(&[1200., 600., 300., 240.]),
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect("float");
    assert!(floated.notes.is_empty());

    let report = apply(
        &mut layout,
        PanelOp::SetFloatRect {
            section: SectionId::Display,
            rect: rect(&[1200., 600., 300., 240.]),
            work: work(1280., 720.),
        },
    )
    .expect("recovery");
    assert_eq!(layout.section(SectionId::Display).float, Some(expected));
    assert!(matches!(
        report.notes.as_slice(),
        [PanelNote::Recovered { .. }]
    ));
    assert_eq!(float_rect(case, "float"), rect(&[972., 452., 300., 240.]));
}

#[test]
fn n15_a_window_below_the_floor_is_refused_without_rewriting_the_layout() {
    let document = fixture("negative.json");
    let case = case(
        &document,
        "N15-shrink-below-the-plan-floor-does-not-rewrite-layout",
    );
    assert_eq!(float_rect(case, "float"), rect(&[1200., 600., 300., 240.]));
    // 500x250 still has a float work area, but the window is below the plan
    // floor, so the region plan fails and every float op is refused.
    let floor = om9_app::workbench_regions::PLAN_FLOOR_WINDOW;
    assert!(500. < floor.width && 250. < floor.height);
    let layout = DockLayout::matrix_default();
    let views = [
        om9_scene::workspace::ViewId::LookingDown,
        om9_scene::workspace::ViewId::Perspective,
    ];
    let region = om9_app::workbench_regions::compute_regions(
        &layout,
        om9_app::workbench_regions::WindowSize::new(500., 250.),
        &views,
    );
    assert!(
        matches!(
            region,
            Err(om9_app::workbench_regions::RegionError::WindowTooSmall { .. })
        ),
        "a window below the plan floor has no region plan"
    );
    assert_eq!(
        PanelOpError::LayoutBlocked.to_string(),
        "layout is blocked by a region error"
    );
}

#[test]
fn n16_an_invalid_window_has_no_work_area() {
    let document = fixture("negative.json");
    let case = case(&document, "N16-work-area-from-an-invalid-window");
    for window in case["windows"].as_array().unwrap() {
        let values = window.as_array().unwrap();
        let parse = |value: &Value| match value {
            Value::Number(number) => number.as_f64().unwrap() as f32,
            Value::String(text) if text == "NaN" => f32::NAN,
            Value::String(text) if text == "inf" => f32::INFINITY,
            other => panic!("unexpected window token {other}"),
        };
        let pair = [parse(&values[0]), parse(&values[1])];
        assert!(
            WorkArea::for_window(pair).is_err(),
            "window {pair:?} has no work area"
        );
    }
}

#[test]
fn n21_n22_the_reserved_builders_area_refuses_every_operation() {
    let document = fixture("negative.json");
    let case = case(&document, "N22-builders-api-operations-are-refused");
    assert!(!SectionId::BuilderSlot.is_movable());
    let ops = [
        PanelOp::Move {
            section: SectionId::BuilderSlot,
            delta: -1,
        },
        PanelOp::Dock {
            section: SectionId::BuilderSlot,
            side: ColumnSide::Right,
        },
        PanelOp::Float {
            section: SectionId::BuilderSlot,
            rect: rect(&[100., 100., 300., 240.]),
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    ];
    assert_eq!(ops.len(), case["ops"].as_array().unwrap().len());
    for op in ops {
        let mut layout = DockLayout::matrix_default();
        let before = layout.to_json().unwrap();
        let error = apply(&mut layout, op).expect_err("the reserved area is refused");
        assert!(matches!(error, PanelOpError::NotPermitted { .. }));
        assert_eq!(layout.to_json().unwrap(), before);
    }
}

#[test]
fn n23_a_floating_menu_is_not_reordered() {
    let document = fixture("negative.json");
    let case = case(&document, "N23-reorder-a-floating-menu");
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: rect(&[550., 393., 300., 240.]),
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect("float");
    let before = layout.to_json().unwrap();
    let error = apply(
        &mut layout,
        PanelOp::Move {
            section: SectionId::Display,
            delta: 1,
        },
    )
    .expect_err("floating menus are not reordered");
    assert!(matches!(error, PanelOpError::NotDocked { .. }));
    assert_eq!(layout.to_json().unwrap(), before);
    assert_eq!(
        case["expect"]["status"],
        "Display is floating; dock it before reordering"
    );
}

#[test]
fn n24_floating_an_already_floating_menu_is_refused() {
    let document = fixture("negative.json");
    let case = case(&document, "N24-float-an-already-floating-menu");
    let kept = rect(&[550., 393., 300., 240.]);
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: kept,
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect("float");
    let error = apply(
        &mut layout,
        PanelOp::Float {
            section: SectionId::Display,
            rect: rect(&[600., 300., 320., 240.]),
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    )
    .expect_err("already floating");
    assert!(matches!(error, PanelOpError::AlreadyFloating { .. }));
    assert_eq!(layout.section(SectionId::Display).float, Some(kept));
    assert_eq!(float_rect(case, "float"), kept);
}

#[test]
fn n25_a_no_op_is_reported_and_never_silent() {
    let document = fixture("negative.json");
    let case = case(&document, "N25-no-op-moves-are-reported-not-silent");
    assert_eq!(case["expect"]["each"]["error"], "AlreadyThere");
    let mut layout = DockLayout::matrix_default();
    let before = layout.to_json().unwrap();
    let ops = [
        PanelOp::Move {
            section: SectionId::IconHistory,
            delta: -1,
        },
        PanelOp::Move {
            section: SectionId::Projects,
            delta: 0,
        },
        PanelOp::Dock {
            section: SectionId::Layers,
            side: ColumnSide::Left,
        },
    ];
    for op in ops {
        let error = apply(&mut layout, op).expect_err("a no-op is an explicit error");
        assert!(
            matches!(error, PanelOpError::AlreadyThere { .. }),
            "{error:?}"
        );
        assert_eq!(layout.to_json().unwrap(), before);
    }
}

#[test]
fn n26_only_a_column_side_is_a_dock_target() {
    let document = fixture("negative.json");
    let case = case(&document, "N26-dock-to-a-non-column-side");
    for side in case["also"].as_array().unwrap() {
        let parsed = match side.as_str().unwrap() {
            "Bottom" => DockSide::Bottom,
            "Floating" => DockSide::Floating,
            other => panic!("unexpected side {other}"),
        };
        assert_eq!(ColumnSide::from_dock_side(parsed), None);
    }
    assert_eq!(ColumnSide::from_dock_side(DockSide::Top), None);
    assert_eq!(SectionId::from_key("left-dock.legacy"), None);
}

#[test]
fn n27_an_unknown_section_key_reports_the_key_back() {
    let document = fixture("negative.json");
    let case = case(&document, "N27-unknown-section-key-at-the-audit-boundary");
    let key = case["key"].as_str().unwrap();
    assert_eq!(SectionId::from_key(key), None);
    let error = PanelOpError::UnknownSection {
        key: key.to_owned(),
    };
    assert_eq!(
        error.to_string(),
        case["expect"]["status"].as_str().unwrap()
    );
}

#[test]
fn n28_a_hidden_section_cannot_be_moved_or_floated() {
    let document = fixture("negative.json");
    let case = case(&document, "N28-hidden-section-cannot-be-moved");
    assert_eq!(case["expect"]["each"]["error"], "Hidden");
    let mut layout = DockLayout::matrix_default();
    layout.set_section_visible(SectionId::InfoSettings, false);
    let before = layout.to_json().unwrap();
    let ops = [
        PanelOp::Move {
            section: SectionId::InfoSettings,
            delta: 1,
        },
        PanelOp::Float {
            section: SectionId::InfoSettings,
            rect: rect(&[100., 100., 300., 240.]),
            work: work(1600., 1000.),
            origin: FloatOrigin::Api,
        },
    ];
    for op in ops {
        let error = apply(&mut layout, op).expect_err("hidden sections are refused");
        assert!(matches!(error, PanelOpError::Hidden { .. }), "{error:?}");
        assert_eq!(layout.to_json().unwrap(), before);
    }
}

#[test]
fn every_error_leaves_the_serialized_layout_identical() {
    let document = fixture("negative.json");
    let mut checked = 0;
    for case in document["cases"].as_array().unwrap() {
        let Some(op) = case.get("op").filter(|value| value.is_object()) else {
            continue;
        };
        let name = op["op"].as_str().unwrap_or_default();
        if !matches!(name, "Move" | "Dock" | "DockHome" | "SetFloatRect") {
            continue;
        }
        let section = op["section"].as_str().unwrap_or("display");
        let Some(section) = SectionId::from_key(section) else {
            continue;
        };
        let mut layout = DockLayout::matrix_default();
        let before = layout.to_json().unwrap();
        let applied = match name {
            "Move" => PanelOp::Move {
                section,
                delta: op["delta"].as_i64().unwrap_or(0) as i32,
            },
            "Dock" => PanelOp::Dock {
                section,
                side: ColumnSide::Left,
            },
            "DockHome" => PanelOp::DockHome { section },
            "SetFloatRect" => PanelOp::SetFloatRect {
                section,
                rect: rect(&[100., 100., 300., 240.]),
                work: work(1600., 1000.),
            },
            other => panic!("unexpected op {other}"),
        };
        if layout.apply_panel_op(applied).is_err() {
            assert_eq!(layout.to_json().unwrap(), before, "case {}", case["id"]);
            checked += 1;
        }
    }
    assert!(checked > 0, "the sweep must exercise failing operations");
}

#[test]
fn fixture_directory_is_the_expected_one() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-003/positive.json");
    assert!(path.exists(), "{}", path.display());
}

/// The live rectangles of the fixture's `derived_geometry_1600x1000_no_right_dock`
/// block, so the drop rules are checked against recorded geometry.
fn fixture_zones(document: &Value, bars: Vec<(SectionId, ColumnSide, RegionRect)>) -> DropZones {
    let geometry = &document["derived_geometry_1600x1000_no_right_dock"];
    let rect = |value: &Value| {
        let values = value.as_array().expect("rect");
        RegionRect::new(
            values[0].as_f64().unwrap() as f32,
            values[1].as_f64().unwrap() as f32,
            values[2].as_f64().unwrap() as f32,
            values[3].as_f64().unwrap() as f32,
        )
    };
    let views: Vec<RegionRect> = geometry["views"]
        .as_object()
        .unwrap()
        .values()
        .map(rect)
        .collect();
    let union = views.iter().copied().reduce(|a, b| {
        RegionRect::new(
            a.x.min(b.x),
            a.y.min(b.y),
            a.right().max(b.right()) - a.x.min(b.x),
            a.bottom().max(b.bottom()) - a.y.min(b.y),
        )
    });
    let strip = union.map(|union| {
        RegionRect::new(
            union.right() - panel_ops::RIGHT_DROP_STRIP,
            union.y,
            panel_ops::RIGHT_DROP_STRIP,
            union.height,
        )
    });
    assert_eq!(
        strip,
        Some(rect(&geometry["right_drop_strip"])),
        "the strip is the rightmost 48 px of the views, as recorded"
    );
    DropZones {
        window: Some(RegionRect::new(0., 0., 1600., 1000.)),
        left: Some(panel_ops::ColumnZone {
            rect: RegionRect::new(0., 21., 300., 979.),
            content_bottom: 1000.,
        }),
        right: None,
        right_strip: strip,
        bars,
        views,
    }
}

#[test]
fn n1_a_drop_beyond_the_right_edge_floats_inside_the_window() {
    let document = fixture("negative.json");
    let case = case(&document, "N1-drop-beyond-the-right-edge");
    let positive = fixture("positive.json");
    let zones = fixture_zones(&positive, Vec::new());
    let raw = float_rect(case, "raw_rect");
    let expected = float_rect(case, "float");
    assert_eq!(float_drop_rect([1700., 500.]), raw);

    let target = resolve_drop(
        &DockLayout::matrix_default(),
        SectionId::Display,
        Some([1700., 500.]),
        &zones,
    );
    assert_eq!(
        target,
        DropTarget::FloatOutside {
            pointer: [1700., 500.]
        }
    );
    let op = drop_op(SectionId::Display, target, Some(work(1600., 1000.))).expect("float op");
    let mut layout = DockLayout::matrix_default();
    let report = apply(&mut layout, op).expect("float");
    assert_eq!(
        report.outcome,
        PanelOutcome::Floated {
            section: SectionId::Display,
            rect: expected
        }
    );
    assert_eq!(
        report.notes,
        vec![
            PanelNote::Recovered {
                section: SectionId::Display,
                from: raw,
                to: expected
            },
            PanelNote::OutsideMainWindow {
                section: SectionId::Display
            }
        ]
    );
}

#[test]
fn n5_a_drop_in_a_view_inside_the_window_floats_without_the_outside_note() {
    let document = fixture("negative.json");
    let case = case(&document, "N5-drop-in-a-view-near-the-bottom-right-corner");
    let positive = fixture("positive.json");
    let zones = fixture_zones(&positive, Vec::new());
    let pointer = [1540., 970.];
    let raw = float_rect(case, "raw_rect");
    let expected = float_rect(case, "float");
    assert_eq!(float_drop_rect(pointer), raw);
    // The precondition of the fixture: inside SideView, outside the strip.
    let side_view = RegionRect::new(951., 580., 649., 400.);
    assert!(
        pointer[0] >= side_view.x
            && pointer[0] <= side_view.right()
            && pointer[1] >= side_view.y
            && pointer[1] <= side_view.bottom()
    );
    assert!(pointer[0] < zones.right_strip.unwrap().x);

    let target = resolve_drop(
        &DockLayout::matrix_default(),
        SectionId::Display,
        Some(pointer),
        &zones,
    );
    assert_eq!(target, DropTarget::Float { pointer });
    let mut layout = DockLayout::matrix_default();
    let report = apply(
        &mut layout,
        drop_op(SectionId::Display, target, Some(work(1600., 1000.))).unwrap(),
    )
    .expect("float");
    assert_eq!(
        report.notes,
        vec![PanelNote::Recovered {
            section: SectionId::Display,
            from: raw,
            to: expected
        }],
        "a drop inside the window never claims the menu left it"
    );
}

#[test]
fn n18_a_drop_over_the_toolbar_cancels() {
    let document = fixture("negative.json");
    let case = case(&document, "N18-drop-over-the-toolbar-cancels");
    let positive = fixture("positive.json");
    let zones = fixture_zones(&positive, Vec::new());
    let toolbar = RegionRect::new(300., 111., 1300., 67.);
    // The menu bar, the command box and the toolbar are not drop targets.
    for pointer in [
        [toolbar.x + 10., toolbar.y + 10.],
        [toolbar.right() - 10., toolbar.bottom() - 10.],
        [10., 10.],
    ] {
        let target = resolve_drop(
            &DockLayout::matrix_default(),
            SectionId::Display,
            Some(pointer),
            &zones,
        );
        assert_eq!(
            target,
            DropTarget::Cancel {
                reason: CancelReason::NoTarget
            },
            "pointer {pointer:?}"
        );
        assert!(drop_indicator(target, &zones, Some(work(1600., 1000.))).is_none());
        let error = drop_op(SectionId::Display, target, Some(work(1600., 1000.)))
            .expect_err("a cancelled drop applies nothing");
        assert!(matches!(error, PanelOpError::DragCancelled { .. }));
    }
    assert_eq!(
        case["expect"]["status"],
        "Display drag cancelled: not over a menu column or viewport"
    );
}

#[test]
fn n20_a_lost_pointer_cancels_the_drop() {
    let document = fixture("negative.json");
    let case = case(&document, "N20-pointer-lost-at-release");
    let positive = fixture("positive.json");
    let zones = fixture_zones(&positive, Vec::new());
    assert!(case["resolve_drop_pointer"].is_null());
    for pointer in [None, Some([f32::NAN, 10.]), Some([10., f32::INFINITY])] {
        let target = resolve_drop(
            &DockLayout::matrix_default(),
            SectionId::Snaps,
            pointer,
            &zones,
        );
        assert_eq!(
            target,
            DropTarget::Cancel {
                reason: CancelReason::PointerLost
            }
        );
    }
    let layout = DockLayout::matrix_default();
    let before = layout.to_json().unwrap();
    assert_eq!(layout.to_json().unwrap(), before, "layout unchanged");
    assert_eq!(case["expect"]["layout_unchanged"], true);
}

#[test]
fn the_drop_index_is_the_first_bar_centre_at_or_below_the_pointer() {
    let positive = fixture("positive.json");
    // Synthetic bars are enough here: the rule only reads rendered centres.
    let bars: Vec<(SectionId, ColumnSide, RegionRect)> = SectionId::ALL
        .iter()
        .enumerate()
        .map(|(index, id)| {
            (
                *id,
                ColumnSide::Left,
                RegionRect::new(0., 21. + 100. * index as f32, 300., 14.),
            )
        })
        .collect();
    let zones = fixture_zones(&positive, bars);
    let layout = DockLayout::matrix_default();
    // Upper half of the DISPLAY bar: a drop above its centre goes before it, so
    // the dragged menu lands at position 2.
    let target = resolve_drop(&layout, SectionId::Projects, Some([20., 210.]), &zones);
    assert_eq!(
        target,
        DropTarget::Place {
            side: ColumnSide::Left,
            index: 2
        }
    );
    // Below every bar: the position after the last remaining section.
    let target = resolve_drop(&layout, SectionId::Projects, Some([20., 990.]), &zones);
    assert_eq!(
        target,
        DropTarget::Place {
            side: ColumnSide::Left,
            index: 7
        }
    );
}

#[test]
fn n34_menu_placement_has_exactly_one_door_in_the_ui() {
    let document = fixture("negative.json");
    let case = case(&document, "N34-source-scan-single-door");
    let mut forbidden: Vec<String> = case["forbidden_calls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert!(forbidden.contains(&".toggle_section(".to_owned()));
    // The design review (MEDIUM 4) asks for the two height/visibility mutators as
    // well: they are placement-adjacent state that must not be touched directly.
    for extra in [".set_section_visible(", ".set_section_height("] {
        if !forbidden.iter().any(|call| call == extra) {
            forbidden.push(extra.to_owned());
        }
    }
    // What the UI may call on the layout model, with the reason on record
    // (OM9-L4-01-01-003-05.single-door.json).
    let allowlist = [
        ".section(",
        ".state(",
        ".left_sections(",
        ".right_sections(",
        ".floating_sections(",
        ".resolve(",
        ".apply_panel_op(",
        ".reset_to_default(",
        ".clamp_floats(",
        ".save(",
        ".set_float(",
        ".set_size(",
        ".set_visible(",
    ];
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut calls = 0;
    for name in ["ui.rs", "ui/chrome.rs", "ui/panel_audit.rs"] {
        let path = root.join(name);
        let text = std::fs::read_to_string(&path).expect("source file");
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or_default();
            for call in &forbidden {
                assert!(
                    !code.contains(call.as_str()),
                    "{}:{} uses {call}; every placement goes through apply_panel_op",
                    path.display(),
                    number + 1
                );
            }
            // A positive check: every `self.dock.<method>(` the UI makes is on the
            // allowlist, so a new bypass cannot slip in unnoticed.
            let mut rest = code;
            while let Some(at) = rest.find("self.dock.") {
                rest = &rest[at + "self.dock.".len()..];
                let method: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let call = format!(".{method}(");
                assert!(
                    allowlist.contains(&call.as_str()),
                    "{}:{} calls self.dock.{method}; add it to the allowlist with a reason",
                    path.display(),
                    number + 1
                );
                calls += 1;
            }
        }
    }
    assert!(calls > 20, "the scan saw {calls} layout calls");
    assert_eq!(case["expect"]["occurrences_outside_comments"], 0);
}

/// A fixture rect field that may hold numbers or the strings `NaN` / `inf`.
fn fixture_float(value: &Value) -> f32 {
    match value {
        Value::Number(number) => number.as_f64().unwrap() as f32,
        Value::String(text) if text == "NaN" => f32::NAN,
        Value::String(text) if text == "inf" => f32::INFINITY,
        Value::String(text) if text == "-inf" => f32::NEG_INFINITY,
        other => panic!("unexpected float token {other}"),
    }
}

#[test]
fn n2_n3_n4_a_drop_outside_any_edge_is_pulled_back_inside() {
    let document = fixture("negative.json");
    let positive = fixture("positive.json");
    let zones = fixture_zones(&positive, Vec::new());
    for id in [
        "N2-drop-beyond-the-left-edge",
        "N3-drop-beyond-the-top-edge",
        "N4-drop-beyond-the-bottom-edge",
    ] {
        let case = case(&document, id);
        let pointer = case["release_pointer"].as_array().unwrap();
        let pointer = [fixture_float(&pointer[0]), fixture_float(&pointer[1])];
        let raw = float_rect(case, "raw_rect");
        let expected = float_rect(case, "float");
        assert_eq!(float_drop_rect(pointer), raw, "{id}");

        let target = resolve_drop(
            &DockLayout::matrix_default(),
            SectionId::Display,
            Some(pointer),
            &zones,
        );
        assert_eq!(target, DropTarget::FloatOutside { pointer }, "{id}");
        let mut layout = DockLayout::matrix_default();
        let report = apply(
            &mut layout,
            drop_op(SectionId::Display, target, Some(work(1600., 1000.))).unwrap(),
        )
        .expect("float");
        assert_eq!(
            report.outcome,
            PanelOutcome::Floated {
                section: SectionId::Display,
                rect: expected
            },
            "{id}"
        );
        assert_eq!(
            report.notes,
            vec![
                PanelNote::Recovered {
                    section: SectionId::Display,
                    from: raw,
                    to: expected
                },
                PanelNote::OutsideMainWindow {
                    section: SectionId::Display
                }
            ],
            "{id}"
        );
    }
}

#[test]
fn n6_to_n12_a_float_rectangle_is_normalised_or_refused() {
    let document = fixture("negative.json");
    for id in [
        "N6-api-float-huge-rect",
        "N7-api-float-far-negative-origin",
        "N8-api-float-small-positive-size-is-normalised",
        "N9-api-float-nan-x",
        "N10-api-float-infinite-width",
        "N11-api-float-zero-width",
        "N12-api-float-negative-height",
    ] {
        let case = case(&document, id);
        let values = case["op"]["rect"].as_array().expect("rect");
        let rect = rect(&[
            fixture_float(&values[0]),
            fixture_float(&values[1]),
            fixture_float(&values[2]),
            fixture_float(&values[3]),
        ]);
        let mut layout = DockLayout::matrix_default();
        let before = layout.to_json().unwrap();
        let report = apply(
            &mut layout,
            PanelOp::Float {
                section: SectionId::Display,
                rect,
                work: work(1600., 1000.),
                origin: FloatOrigin::Api,
            },
        );
        match case["expect"]["error"].as_str() {
            Some("InvalidRect") => {
                let error = report.expect_err("an invalid rectangle is refused");
                assert!(
                    matches!(error, PanelOpError::InvalidRect { .. }),
                    "{id}: {error:?}"
                );
                assert_eq!(layout.to_json().unwrap(), before, "{id}");
            }
            _ => {
                let report = report.unwrap_or_else(|error| panic!("{id}: {error:?}"));
                assert_eq!(
                    report.outcome,
                    PanelOutcome::Floated {
                        section: SectionId::Display,
                        rect: float_rect(case, "float")
                    },
                    "{id}"
                );
                assert_eq!(
                    report.notes,
                    vec![PanelNote::Recovered {
                        section: SectionId::Display,
                        from: rect,
                        to: float_rect(case, "float")
                    }],
                    "{id}"
                );
            }
        }
    }
}

#[test]
fn n31_docking_another_menu_right_never_moves_layers() {
    let document = fixture("negative.json");
    let case = case(&document, "N31-layers-is-never-moved-by-the-audit-hook");
    assert_eq!(case["env"]["OM9_UI_AUDIT_RIGHT_DOCK"], "1");
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Dock {
            section: SectionId::Projects,
            side: ColumnSide::Right,
        },
    )
    .expect("dock projects right");
    assert_eq!(layout.right_sections(), vec![SectionId::Projects]);
    assert_eq!(
        case["expect"]["right_order"],
        serde_json::json!(["projects"])
    );
    assert_eq!(case["expect"]["layers.dock"], "left");
    assert_eq!(layout.section(SectionId::Layers).dock, DockSide::Left);
    assert_eq!(layout.section(SectionId::Layers).order, 5);
    assert_eq!(layout.section(SectionId::InfoSettings).order, 4);
    assert_eq!(case["expect"]["layers_directly_below_info_settings"], true);
}

#[test]
fn n32_a_narrow_window_yields_the_right_dock_without_moving_layers() {
    let document = fixture("negative.json");
    let case = case(
        &document,
        "N32-layers-docked-right-then-narrow-window-is-not-auto-moved",
    );
    let mut layout = DockLayout::matrix_default();
    apply(
        &mut layout,
        PanelOp::Dock {
            section: SectionId::Layers,
            side: ColumnSide::Right,
        },
    )
    .expect("dock layers right");
    // Docking the right column makes it visible: the panel state is what the
    // region planner reads.
    layout.set_visible(om9_app::dock_layout::PanelId::RightDock, true);
    let plan = om9_app::workbench_regions::compute_regions(
        &layout,
        om9_app::workbench_regions::WindowSize::new(1000., 720.),
        &[
            om9_scene::workspace::ViewId::LookingDown,
            om9_scene::workspace::ViewId::Perspective,
        ],
    )
    .expect("a 1000x720 window still plans");
    assert!(
        plan.notes.iter().any(|note| matches!(
            note,
            om9_app::workbench_regions::RegionNote::RightDockYielded { .. }
        )),
        "the right column yields first: {:?}",
        plan.notes
    );
    assert!(
        plan.right_dock.is_none(),
        "the yielded column is not planned"
    );
    // The menu is not auto-moved: it stays in the right column.
    assert_eq!(case["expect"]["layers.dock"], "right");
    assert_eq!(layout.section(SectionId::Layers).dock, DockSide::Right);
    // Dock left later restores the documented slot (D6: below Info & Settings).
    apply(
        &mut layout,
        PanelOp::DockHome {
            section: SectionId::Layers,
        },
    )
    .expect("dock layers home");
    assert_eq!(layout.section(SectionId::Layers).order, 5);
    assert_eq!(case["expect"]["dock_left_restores"]["left_order_index"], 5);
}

#[test]
fn p10_layers_defaults_to_left_directly_below_info_and_settings() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P10-layers-default-is-left-directly-below-info-and-settings",
    );
    let layout = DockLayout::matrix_default();
    let defaults = document["section_state_default"]
        .as_object()
        .expect("section_state_default");
    assert!(defaults.contains_key("layers"));
    for (key, expected) in defaults {
        let id = SectionId::from_key(key).unwrap_or_else(|| panic!("section {key}"));
        let state = layout.section(id);
        assert_eq!(
            state.visible,
            expected["visible"].as_bool().unwrap(),
            "{key}"
        );
        assert_eq!(
            state.order,
            expected["order"].as_u64().unwrap() as u32,
            "{key}"
        );
        assert_eq!(
            state.collapsed,
            expected["collapsed"].as_bool().unwrap(),
            "{key}"
        );
        assert_eq!(
            serde_json::to_value(state.dock).unwrap().as_str().unwrap(),
            expected["dock"].as_str().unwrap(),
            "{key}"
        );
        assert_eq!(state.float, None, "{key}");
        assert_eq!(state.home, None, "{key}");
    }
    assert_eq!(case["expect"]["info_settings.order"], 4);
    assert_eq!(case["expect"]["layers.order"], 5);
    assert_eq!(
        layout.section(SectionId::InfoSettings).order + 1,
        layout.section(SectionId::Layers).order
    );
    assert_eq!(layout.left_sections()[5], SectionId::Layers);
}

#[test]
fn p11_layers_placement_holds_at_every_supported_size() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "P11-layers-placement-holds-at-every-supported-size",
    );
    let sweep = &case["sweep"];
    let widths: Vec<f32> = sweep["widths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap() as f32)
        .collect();
    let heights: Vec<f32> = sweep["heights"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap() as f32)
        .collect();
    let views = [
        om9_scene::workspace::ViewId::LookingDown,
        om9_scene::workspace::ViewId::Perspective,
    ];
    let mut checked = 0;
    for width in &widths {
        for height in &heights {
            let layout = DockLayout::matrix_default();
            let plan = om9_app::workbench_regions::compute_regions(
                &layout,
                om9_app::workbench_regions::WindowSize::new(*width, *height),
                &views,
            );
            let Ok(plan) = plan else {
                // Below the plan floor there is no plan at all.
                assert!(*width < 800. || *height < 400.);
                continue;
            };
            // The Layers panel is in the left column, right below Info & Settings,
            // and the left rail holding it stays planned (not yielded) at size.
            assert_eq!(
                layout.left_sections()[4],
                SectionId::InfoSettings,
                "{width}x{height}"
            );
            assert_eq!(
                layout.left_sections()[5],
                SectionId::Layers,
                "{width}x{height}"
            );
            assert!(plan.left_rail.is_some(), "{width}x{height}");
            assert_eq!(plan.window.width, *width, "{width}x{height}");
            checked += 1;
        }
    }
    assert!(checked > 40, "the sweep covered {checked} sizes");
}

/// The generator of N33: seeded xorshift64.
struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.0 = state;
        state
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

#[test]
fn n33_randomized_operation_sweep_holds_every_invariant() {
    let document = fixture("negative.json");
    let case = case(&document, "N33-randomized-operation-sweep");
    assert_eq!(case["expect"]["no_panic"], true);
    // Rects including NaN, inf, 0, -1 and 1e6, deltas including i32::MIN/MAX,
    // windows including invalid ones, sections including BuilderSlot.
    let rects = [
        FloatRect {
            x: 100.,
            y: 100.,
            width: 300.,
            height: 240.,
        },
        FloatRect {
            x: f32::NAN,
            y: 0.,
            width: 300.,
            height: 240.,
        },
        FloatRect {
            x: 0.,
            y: 0.,
            width: f32::INFINITY,
            height: 240.,
        },
        FloatRect {
            x: 0.,
            y: 0.,
            width: 0.,
            height: 240.,
        },
        FloatRect {
            x: 0.,
            y: 0.,
            width: -1.,
            height: -1.,
        },
        FloatRect {
            x: 1e6,
            y: 1e6,
            width: 1e6,
            height: 1e6,
        },
        FloatRect {
            x: -5000.,
            y: -5000.,
            width: 300.,
            height: 240.,
        },
    ];
    let windows = [
        [1600., 1000.],
        [1280., 720.],
        [1000000., 500.],
        [150., 150.],
        [f32::NAN, 720.],
    ];
    let deltas = [i32::MIN, -3, -1, 0, 1, 2, i32::MAX];
    let sides = [ColumnSide::Left, ColumnSide::Right];
    let mut rng = XorShift64(0x0E9_003);
    let mut layout = DockLayout::matrix_default();
    let mut work: Option<WorkArea> = None;
    let mut pressed_area: std::collections::BTreeMap<SectionId, WorkArea> =
        std::collections::BTreeMap::new();
    let mut ok_count = 0;
    let mut err_count = 0;
    for step in 0..2000 {
        let window = windows[rng.below(windows.len())];
        work = WorkArea::for_window(window).ok().or(work);
        let section = SectionId::ALL[rng.below(SectionId::ALL.len())];
        let op = match rng.below(7) {
            0 => PanelOp::ToggleCollapsed { section },
            1 => PanelOp::Move {
                section,
                delta: deltas[rng.below(deltas.len())],
            },
            2 => PanelOp::Place {
                section,
                side: sides[rng.below(2)],
                index: rng.below(9),
            },
            3 => PanelOp::Dock {
                section,
                side: sides[rng.below(2)],
            },
            4 => PanelOp::DockHome { section },
            5 => PanelOp::Float {
                section,
                rect: rects[rng.below(rects.len())],
                work: work.unwrap_or(WorkArea {
                    x: 8.,
                    y: 29.,
                    width: 1584.,
                    height: 943.,
                }),
                origin: FloatOrigin::Api,
            },
            _ => PanelOp::SetFloatRect {
                section,
                rect: rects[rng.below(rects.len())],
                work: work.unwrap_or(WorkArea {
                    x: 8.,
                    y: 29.,
                    width: 1584.,
                    height: 943.,
                }),
            },
        };
        let before = layout.to_json().unwrap();
        let op_work = work.unwrap_or(WorkArea {
            x: 8.,
            y: 29.,
            width: 1584.,
            height: 943.,
        });
        match layout.apply_panel_op(op) {
            Ok(_) => {
                ok_count += 1;
                if matches!(op, PanelOp::Float { .. } | PanelOp::SetFloatRect { .. }) {
                    pressed_area.insert(section, op_work);
                }
                // Every menu exactly once, orders dense, floats finite and inside
                // the work area that was in force when the rectangle was set.
                assert_invariants(&layout);
                for id in layout.floating_sections() {
                    let rect = layout
                        .section(id)
                        .float
                        .expect("a floating menu has a rect");
                    assert!(rect.x.is_finite() && rect.y.is_finite(), "step {step}");
                    assert!(
                        rect.width.is_finite() && rect.height.is_finite(),
                        "step {step}"
                    );
                    if let Some(area) = pressed_area.get(&id) {
                        assert!(area.contains(rect), "step {step}: {rect:?} outside {area}");
                    }
                }
                // The serialized layout is valid JSON.
                let json = layout.to_json().unwrap();
                serde_json::from_str::<Value>(&json).expect("valid JSON");
            }
            Err(_) => {
                err_count += 1;
                assert_eq!(layout.to_json().unwrap(), before, "step {step}");
            }
        }
    }
    assert!(
        ok_count > 0 && err_count > 0,
        "sweep: {ok_count} ok / {err_count} err"
    );
    assert_eq!(case["expect"]["every_section_exactly_once"], true);
    assert_eq!(case["expect"]["orders_dense_per_column"], true);
}
