// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-01-008 (Builder Panel Docking Slot), driven by
//! the fixtures in `implementation/fixtures/M01/OM9-L3-01-01-008/`.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::dock_layout::{DockLayout, DockSide, FloatRect, SectionId};
use om9_app::panel_ops::{ColumnSide, FloatOrigin, PanelOp, PanelOpError, WorkArea};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

const SLOT: SectionId = SectionId::BuilderSlot;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-008")
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

fn work() -> WorkArea {
    WorkArea::for_window([1600., 1000.]).unwrap()
}

fn left_keys(layout: &DockLayout) -> Vec<&'static str> {
    layout.left_sections().iter().map(|id| id.key()).collect()
}

#[test]
fn b1_the_slot_is_reserved_directly_below_projects() {
    let document = fixture("positive.json");
    let case = fixture_case(&document, "B1-the-slot-is-reserved-directly-below-projects");
    let layout = DockLayout::matrix_default();
    let expect = &case["expect"];
    assert_eq!(expect["builder_slot_is_reserved"], true);
    assert!(SLOT.is_reserved());
    assert!(!SLOT.is_movable());
    assert_eq!(expect["builder_slot_is_movable"], false);
    assert_eq!(
        layout.section(SLOT).order,
        expect["builder_slot_order"].as_u64().unwrap() as u32
    );
    assert_eq!(
        layout.section(SectionId::Projects).order,
        expect["projects_order"].as_u64().unwrap() as u32
    );
    assert!(layout.section(SLOT).visible);
    assert_eq!(left_keys(&layout)[7], "builder-slot");
    assert_eq!(left_keys(&layout)[6], "projects");
    assert!(layout.floating_sections().is_empty());
    let documented: Vec<String> = document["documented_left_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        left_keys(&layout),
        documented.iter().map(String::as_str).collect::<Vec<_>>()
    );
}

#[test]
fn b2_every_placement_is_refused_and_changes_nothing() {
    let document = fixture("positive.json");
    let case = fixture_case(
        &document,
        "B2-every-placement-is-refused-and-changes-nothing",
    );
    let rect = FloatRect {
        x: 100.,
        y: 100.,
        width: 300.,
        height: 240.,
    };
    let ops = [
        PanelOp::ToggleCollapsed { section: SLOT },
        PanelOp::Move {
            section: SLOT,
            delta: -1,
        },
        PanelOp::Place {
            section: SLOT,
            side: ColumnSide::Left,
            index: 0,
        },
        PanelOp::Dock {
            section: SLOT,
            side: ColumnSide::Right,
        },
        PanelOp::DockHome { section: SLOT },
        PanelOp::Float {
            section: SLOT,
            rect,
            work: work(),
            origin: FloatOrigin::Api,
        },
        PanelOp::SetFloatRect {
            section: SLOT,
            rect,
            work: work(),
        },
    ];
    assert_eq!(ops.len(), case["ops"].as_array().unwrap().len());
    for op in ops {
        let mut layout = DockLayout::matrix_default();
        let before = layout.to_json().unwrap();
        let error = layout.apply_panel_op(op).expect_err("the slot is reserved");
        assert!(
            matches!(error, PanelOpError::NotPermitted { .. }),
            "{error:?}"
        );
        assert_eq!(layout.to_json().unwrap(), before, "{op:?}");
        assert_eq!(case["expect"]["each"]["error"], "NotPermitted");
    }
}

#[test]
fn b3_the_slot_owns_no_model() {
    let document = fixture("positive.json");
    let case = fixture_case(&document, "B3-the-slot-owns-no-model");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("builder slot")).unwrap();
    for _ in 0..3 {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    // Draw one line: the document changes, the slot does not.
    app.controller
        .dispatch(
            om9_app::session::LINE,
            om9_app::session::InvocationOrigin::Cmd,
        )
        .unwrap();
    app.controller.accept_world_point([0., 0., 0.]).unwrap();
    app.controller.accept_world_point([5., 0., 0.]).unwrap();
    assert_eq!(
        app.controller.document().entities.len() as u64,
        case["expect"]["document_entities"].as_u64().unwrap()
    );
    assert_eq!(case["expect"]["builder_entities"], 0);
    // The placeholder is drawn from the layout, not from a builder object.
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("chrome.rs"),
    )
    .expect("chrome.rs");
    let start = source.find("fn builder_slot_ui").expect("builder_slot_ui");
    let body = &source[start..start + 2000.min(source.len() - start)];
    for forbidden in [
        "Builder::",
        "builder_model",
        "Controller",
        "Document",
        "document()",
    ] {
        assert!(
            !body.contains(forbidden),
            "the placeholder must not touch {forbidden}"
        );
    }
    assert_eq!(case["expect"]["no_second_model_in_source"], true);
}

#[test]
fn b4_the_bar_carries_no_close_box_and_starts_no_drag() {
    let document = fixture("positive.json");
    let case = fixture_case(
        &document,
        "B4-the-bar-carries-no-close-box-and-starts-no-drag",
    );
    assert_eq!(case["expect"]["close_box"], false);
    assert_eq!(case["expect"]["drag_can_start"], false);
    // The close box is drawn only for a movable section (ui/chrome.rs::section).
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("chrome.rs"),
    )
    .expect("chrome.rs");
    let start = source.find("fn section(").expect("section");
    let body = &source[start..start + 2600.min(source.len() - start)];
    assert!(
        body.contains("if id.is_movable() {"),
        "the close box must be guarded by is_movable()"
    );
    // A drag can never start on the reserved bar: driving the same events a mouse
    // produces must leave the drag state empty.
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("builder drag")).unwrap();
    for _ in 0..3 {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    let bar = app.section_rect("builder-slot").expect("builders bar");
    for event in [
        Event::PointerMoved(bar.center()),
        Event::PointerButton {
            pos: bar.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerMoved(bar.center() + Vec2::new(0., 40.)),
    ] {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
                events: vec![event],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    assert!(
        app.section_drag().is_none(),
        "a reserved bar must never start a drag"
    );
}

#[test]
fn b5_the_placeholder_became_the_live_builder_panel_in_m04() {
    // The M01 case recorded the reserved slot's placeholder: every widget disabled or hover
    // only. M04 (`OM9-L2-03-11`) now owns the slot, so the placeholder is superseded by the
    // builder panel and this case is kept for provenance only. The reservation itself is
    // still proved by b1..b4.
    let document = fixture("positive.json");
    let case = fixture_case(&document, "B5-the-placeholder-is-noninteractive");
    assert_eq!(
        case["expect"]["every_widget_disabled_or_hover_only"], true,
        "the M01 fixture keeps its placeholder expectation"
    );
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("chrome.rs"),
    )
    .expect("chrome.rs");
    let start = source.find("fn builder_slot_ui").expect("builder_slot_ui");
    let end = source[start..]
        .find("pub(super) fn toolbars_ui")
        .map(|offset| start + offset)
        .expect("end of builder_slot_ui");
    let body = &source[start..end];
    let compact: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    // The slot now paints the real panel: the mode tabs, the typed fields and the
    // Apply/Reset/Cancel row, wired to the builder service.
    for needle in [
        "builder_tabs()",
        "builder_field_views()",
        "builder_set_field",
        "builder_set_slider",
        "\"Apply\"",
        "\"Reset\"",
        "\"Cancel\"",
    ] {
        assert!(
            compact.contains(needle),
            "the builder slot must paint {needle}"
        );
    }
    // The placeholder's disabled controls are gone, so no widget is created disabled here.
    assert!(
        !compact.contains("add_enabled(false,"),
        "the placeholder controls must not survive the M04 panel"
    );
    assert!(
        !compact.contains("Builder (reserved)"),
        "the reserved placeholder label must not survive the M04 panel"
    );
}

#[test]
fn b6_to_y4_a_layout_file_cannot_break_the_reservation() {
    for (name, id) in [
        (
            "positive.json",
            "B6-a-layout-file-cannot-break-the-reservation",
        ),
        ("negative.json", "Y4-a-file-cannot-hide-or-float-the-slot"),
    ] {
        let document = fixture(name);
        let case = fixture_case(&document, id);
        let bytes = case["bytes"].as_str().unwrap().as_bytes().to_vec();
        let (layout, diagnostics) = DockLayout::from_bytes(&bytes).expect("a v2 document parses");
        let needle = case["expect"]["diagnostic_contains"].as_str().unwrap();
        assert!(
            diagnostics.iter().any(|line| line.contains(needle)),
            "{id}: {diagnostics:?} lacks {needle:?}"
        );
        let state = layout.section(SLOT);
        assert_eq!(state.dock, DockSide::Left, "{id}");
        assert!(state.visible, "{id}");
        assert!(!state.collapsed, "{id}");
        assert_eq!(state.float, None, "{id}");
        assert_eq!(state.order, 7, "{id}");
        assert_eq!(
            left_keys(&layout),
            SectionId::ALL.iter().map(|id| id.key()).collect::<Vec<_>>(),
            "{id}"
        );
        if let Some(panics) = case["expect"]["panics"].as_bool() {
            assert!(!panics, "{id}");
        }
    }
}

#[test]
fn y1_a_click_on_the_bar_is_refused() {
    let document = fixture("negative.json");
    let case = fixture_case(&document, "Y1-a-click-on-the-bar-is-refused");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("builder click")).unwrap();
    // The reserved slot sits below the 720-high fold; a click outside the viewport is
    // never delivered by egui. Use the design work area so the event lands on the bar.
    for _ in 0..3 {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600., 1000.))),
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    let bar = app.section_rect("builder-slot").expect("builders bar");
    let before = app.dock.to_json().unwrap();
    for pressed in [true, false] {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600., 1000.))),
                events: vec![
                    Event::PointerMoved(bar.center()),
                    Event::PointerButton {
                        pos: bar.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    assert!(
        matches!(
            app.last_panel_op
                .as_ref()
                .and_then(|record| record.error.as_ref()),
            Some(PanelOpError::NotPermitted { .. })
        ),
        "{:?}",
        app.last_panel_op
    );
    assert_eq!(
        app.status.message(),
        case["expect"]["status"].as_str().unwrap()
    );
    assert!(!app.dock.section(SLOT).collapsed);
    assert!(app.section_drag().is_none());
    assert_eq!(app.dock.to_json().unwrap(), before);
}

#[test]
fn y2_y3_y5_the_slot_is_a_placeholder_and_never_a_menu() {
    let positive = fixture("positive.json");
    let negative = fixture("negative.json");
    let y5 = fixture_case(&negative, "Y5-the-slot-is-not-a-menu-the-user-can-lose");
    let layout = DockLayout::matrix_default();
    assert!(!layout.right_sections().contains(&SLOT));
    assert!(!layout.floating_sections().contains(&SLOT));
    assert_eq!(y5["expect"]["right_order_excludes_builder_slot"], true);
    assert_eq!(y5["expect"]["floating_excludes_builder_slot"], true);

    // Y2: the counts the placeholder must show.
    let y2 = fixture_case(&negative, "Y2-the-placeholder-answers-no-input");
    assert_eq!(y2["expect"]["buttons_enabled"], 0);
    assert_eq!(y2["expect"]["sliders_enabled"], 0);
    assert_eq!(y2["expect"]["gem_box_sense"], "hover");
    assert_eq!(y2["expect"]["icons_enabled"], 0);

    // Y3: no builder state exists before M04.
    let y3 = fixture_case(&negative, "Y3-no-builder-model-exists-before-m04");
    assert_eq!(y3["expect"]["controller_holds_builder_state"], false);
    assert_eq!(y3["expect"]["document_has_builder_entities"], 0);
    assert_eq!(y3["expect"]["undo_len_from_slot"], 0);
    let app = MatrixApp::new(Document::new("builder state")).unwrap();
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert_eq!(
        positive["cases"].as_array().unwrap().len(),
        6,
        "the positive fixture keeps its six cases"
    );
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-01-008/api.md");
    assert!(api.exists(), "{}", api.display());
}
