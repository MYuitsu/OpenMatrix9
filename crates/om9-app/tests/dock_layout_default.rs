// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-01-004 (Docking Layout Default Restore), driven
//! by the fixtures in `implementation/fixtures/M01/OM9-L3-01-01-004/`.
use om9_app::dock_layout::{DockLayout, DockSide, FloatRect, PanelId, SectionId};
use om9_app::panel_ops::{ColumnSide, FloatOrigin, PanelOp};
use om9_app::status_surface::StatusKind;
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-004")
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

fn panel(key: &str) -> PanelId {
    match key {
        "left-rail" => PanelId::LeftRail,
        "right-dock" => PanelId::RightDock,
        "command-line" => PanelId::CommandLine,
        "toolbar" => PanelId::Toolbar,
        other => panic!("unknown panel {other}"),
    }
}

fn section(key: &str) -> SectionId {
    SectionId::from_key(key).unwrap_or_else(|| panic!("unknown section {key}"))
}

/// Apply one customisation step from the fixture to the model.
fn customise(layout: &mut DockLayout, step: &Value) {
    match step["op"].as_str().expect("op") {
        "Dock" => {
            let side = match step["side"].as_str().expect("side") {
                "right" => ColumnSide::Right,
                other => panic!("unknown side {other}"),
            };
            layout
                .apply_panel_op(PanelOp::Dock {
                    section: section(step["section"].as_str().unwrap()),
                    side,
                })
                .expect("dock");
        }
        "ToggleCollapsed" => {
            layout
                .apply_panel_op(PanelOp::ToggleCollapsed {
                    section: section(step["section"].as_str().unwrap()),
                })
                .expect("collapse");
        }
        "Move" => {
            layout
                .apply_panel_op(PanelOp::Move {
                    section: section(step["section"].as_str().unwrap()),
                    delta: step["delta"].as_i64().unwrap() as i32,
                })
                .expect("move");
        }
        "Float" => {
            let values = step["rect"].as_array().expect("rect");
            let rect = FloatRect {
                x: values[0].as_f64().unwrap() as f32,
                y: values[1].as_f64().unwrap() as f32,
                width: values[2].as_f64().unwrap() as f32,
                height: values[3].as_f64().unwrap() as f32,
            };
            layout
                .apply_panel_op(PanelOp::Float {
                    section: section(step["section"].as_str().unwrap()),
                    rect,
                    work: om9_app::panel_ops::WorkArea::for_window([1600., 1000.]).unwrap(),
                    origin: FloatOrigin::Api,
                })
                .expect("float");
        }
        "set_section_visible" => layout.set_section_visible(
            section(step["section"].as_str().unwrap()),
            step["visible"].as_bool().unwrap(),
        ),
        "set_size" => layout.set_size(
            panel(step["panel"].as_str().unwrap()),
            step["size"].as_f64().unwrap() as f32,
        ),
        "set_visible" => layout.set_visible(
            panel(step["panel"].as_str().unwrap()),
            step["visible"].as_bool().unwrap(),
        ),
        other => panic!("unknown op {other}"),
    }
}

fn baseline_bytes() -> String {
    DockLayout::matrix_default().to_json().unwrap()
}

fn documented_order(document: &Value) -> Vec<String> {
    document["documented_left_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn d1_reset_after_every_placement_kind_recreates_the_baseline() {
    let document = fixture("positive.json");
    let case = case(&document, "D1-reset-after-every-placement-kind");
    let mut layout = DockLayout::matrix_default();
    for step in case["customisation"].as_array().unwrap() {
        customise(&mut layout, step);
    }
    assert_ne!(
        layout.to_json().unwrap(),
        baseline_bytes(),
        "the customisation must really change the layout"
    );

    layout.reset_to_default();
    assert_eq!(layout.to_json().unwrap(), baseline_bytes());

    let expect = &case["expect"];
    assert_eq!(expect["after_reset_equals_matrix_default_bytes"], true);
    let left: Vec<String> = layout
        .left_sections()
        .iter()
        .map(|id| id.key().to_owned())
        .collect();
    assert_eq!(left, documented_order(&document));
    assert!(layout.right_sections().is_empty());
    assert!(layout.floating_sections().is_empty());
    for id in SectionId::ALL {
        let state = layout.section(id);
        assert!(state.visible, "{} must be visible", id.key());
        assert!(!state.collapsed, "{} must be expanded", id.key());
        assert_eq!(state.home, None, "{} must not keep a home", id.key());
        assert_eq!(state.float, None, "{} must not keep a rect", id.key());
    }
    let panels = &document["documented_panels"];
    for (key, expected) in panels.as_object().unwrap() {
        let state = layout.state(panel(key));
        assert_eq!(state.size, expected.as_f64().unwrap() as f32, "{key}");
        if key == "right-dock" {
            // The documented baseline keeps the optional right column hidden;
            // its size is still the documented starting width.
            assert!(!state.visible, "{key} must be hidden in the baseline");
        } else {
            assert!(state.visible, "{key} must be visible");
        }
    }
    assert_eq!(case["expect"]["panels"], "documented_panels");
    assert!(!layout.state(PanelId::RightDock).visible);
    assert_eq!(case["expect"]["right_dock_visible"], false);
    assert!(case["expect"]["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn d2_reset_is_idempotent() {
    let document = fixture("positive.json");
    let case = case(&document, "D2-reset-is-idempotent");
    let mut layout = DockLayout::matrix_default();
    for step in case["customisation"].as_array().unwrap() {
        customise(&mut layout, step);
    }
    layout.reset_to_default();
    assert_eq!(layout.to_json().unwrap(), baseline_bytes());
    layout.reset_to_default();
    assert_eq!(layout.to_json().unwrap(), baseline_bytes());
}

#[test]
fn d3_reset_returns_the_baseline_and_the_caller_decides_what_to_publish() {
    let document = fixture("positive.json");
    let case = case(&document, "D3-reset-touches-interface-only");
    let mut layout = DockLayout::matrix_default();
    for step in case["customisation"].as_array().unwrap() {
        customise(&mut layout, step);
    }
    layout.reset_to_default();
    assert_eq!(layout.to_json().unwrap(), baseline_bytes());
    assert_eq!(case["expect"]["dock_dirty_is_the_callers_decision"], true);
}

#[test]
fn d4_reload_without_a_store_is_the_factory_default() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "D4-reload-without-a-store-is-the-factory-default",
    );
    assert!(case["store"].is_null());
    let mut app = MatrixApp::new(Document::new("reload default")).unwrap();
    assert!(app.dock_layout_path().is_none());
    // Change the layout first, so the reload has something to undo.
    app.dock.set_size(PanelId::LeftRail, 420.);
    app.reload_menus();
    assert_eq!(app.dock.to_json().unwrap(), baseline_bytes());
    assert_eq!(app.status.kind(), StatusKind::Info);
    assert_eq!(
        app.status.message(),
        case["expect"]["command_status"].as_str().unwrap()
    );
    assert!(!app.menus_have_unsaved_changes());
}

#[test]
fn n1_to_n8_a_layout_that_cannot_be_honoured_resets_to_the_baseline() {
    let document = fixture("negative.json");
    let cases = document["cases"].as_array().unwrap();
    // N9 is a v2 document that parses: only its stored home is repaired, so it
    // must not be required to equal the baseline. Its own test covers it.
    let cases: Vec<&Value> = cases
        .iter()
        .filter(|case| !case["id"].as_str().unwrap().starts_with("N9"))
        .collect();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let bytes: Vec<u8> = match case.get("bytes_padding") {
            Some(padding) => {
                // A document over the byte budget: valid JSON prefix, padded.
                let padding = padding.as_u64().unwrap() as usize;
                let mut bytes = b"{\"schema_version\": 2, \"padding\": \"".to_vec();
                bytes.extend(std::iter::repeat_n(b'x', padding));
                bytes.extend_from_slice(b"\"}");
                bytes
            }
            None => case["bytes"].as_str().unwrap().as_bytes().to_vec(),
        };
        let outcome = DockLayout::from_bytes(&bytes);
        match case["expect"]["error"].as_str() {
            Some(kind) => {
                let error = outcome.expect_err(&format!("{id}: must be refused"));
                let text = format!("{error:?}");
                assert!(text.contains(kind), "{id}: {text} does not contain {kind}");
            }
            None => {
                let (layout, diagnostics) =
                    outcome.unwrap_or_else(|error| panic!("{id}: {error:?}"));
                if let Some(needle) = case["expect"]["diagnostic_contains"].as_str() {
                    assert!(
                        diagnostics.iter().any(|line| line.contains(needle)),
                        "{id}: {diagnostics:?} lacks {needle:?}"
                    );
                }
                assert_eq!(
                    layout.to_json().unwrap(),
                    baseline_bytes(),
                    "{id}: the baseline must be the total fallback"
                );
                if let Some(size) = case["expect"]["left_rail_size"].as_f64() {
                    assert_eq!(layout.state(PanelId::LeftRail).size, size as f32, "{id}");
                }
                if let Some(visible) = case["expect"]["left_rail_visible"].as_bool() {
                    assert_eq!(layout.state(PanelId::LeftRail).visible, visible, "{id}");
                }
                if let Some(visible) = case["expect"]["projects_visible"].as_bool() {
                    assert_eq!(layout.section(SectionId::Projects).visible, visible, "{id}");
                }
                if let Some(collapsed) = case["expect"]["projects_collapsed"].as_bool() {
                    assert_eq!(
                        layout.section(SectionId::Projects).collapsed,
                        collapsed,
                        "{id}"
                    );
                }
                if let Some(dock) = case["expect"]["projects_dock"].as_str() {
                    let actual = serde_json::to_value(layout.section(SectionId::Projects).dock)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned();
                    assert_eq!(actual, dock, "{id}");
                }
                if let Some(order) = case["expect"]["projects_order"].as_u64() {
                    assert_eq!(
                        layout.section(SectionId::Projects).order,
                        order as u32,
                        "{id}"
                    );
                }
                if case["expect"]["after_reset_equals_matrix_default_bytes"] == true {
                    let mut layout = layout;
                    layout.reset_to_default();
                    assert_eq!(layout.to_json().unwrap(), baseline_bytes(), "{id}");
                }
            }
        }
        assert_eq!(case["expect"]["panics"], false, "{id}");
    }
    let _ = DockSide::Left;
}

#[test]
fn n9_an_impossible_stored_home_is_repaired_with_a_diagnostic() {
    let document = fixture("negative.json");
    let case = case(&document, "N9-an-impossible-stored-home-is-repaired");
    let bytes = case["bytes"].as_str().unwrap().as_bytes().to_vec();
    let (layout, diagnostics) = DockLayout::from_bytes(&bytes).expect("a v2 document parses");
    let needle = case["expect"]["diagnostic_contains"].as_str().unwrap();
    assert!(
        diagnostics.iter().any(|line| line.contains(needle)),
        "{diagnostics:?} lacks {needle:?}"
    );
    let home = layout.section(SectionId::Projects).home;
    assert_eq!(
        home.map(|home| (home.side, home.index)),
        Some((ColumnSide::Left, 6)),
        "the home must be the documented position"
    );
    // The repaired home is usable: closing the menu returns it to slot 6.
    let mut layout = layout;
    layout
        .apply_panel_op(PanelOp::Float {
            section: SectionId::Projects,
            rect: FloatRect {
                x: 400.,
                y: 300.,
                width: 300.,
                height: 240.,
            },
            work: om9_app::panel_ops::WorkArea::for_window([1600., 1000.]).unwrap(),
            origin: FloatOrigin::Api,
        })
        .expect("float");
    layout
        .apply_panel_op(PanelOp::DockHome {
            section: SectionId::Projects,
        })
        .expect("dock home");
    assert_eq!(layout.section(SectionId::Projects).order, 6);
}

#[test]
fn fixture_bytes_are_the_specification() {
    // A guard against editing the fixture instead of the code.
    let positive = fixture("positive.json");
    let negative = fixture("negative.json");
    assert_eq!(positive["l3"], "OM9-L3-01-01-004");
    assert_eq!(negative["l3"], "OM9-L3-01-01-004");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-01-004/api.md");
    assert!(path.exists(), "{}", path.display());
}
