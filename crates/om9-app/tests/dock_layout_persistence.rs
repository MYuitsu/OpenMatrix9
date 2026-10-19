// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-01-005 (Docking Layout Persistence), driven by
//! the fixtures in `implementation/fixtures/M01/OM9-L3-01-01-005/`.
use om9_app::dock_layout::{DockLayout, DockSide, FloatRect, PanelId, SectionId};
use om9_app::panel_ops::{ColumnSide, FloatOrigin, PanelOp, WorkArea};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-005")
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
        "command-search" => PanelId::CommandSearch,
        "toolbar" => PanelId::Toolbar,
        other => panic!("unknown panel {other}"),
    }
}

fn section(key: &str) -> SectionId {
    SectionId::from_key(key).unwrap_or_else(|| panic!("unknown section {key}"))
}

fn work() -> WorkArea {
    WorkArea::for_window([1600., 1000.]).unwrap()
}

fn arrange(app: &mut MatrixApp, steps: &[Value]) {
    for step in steps {
        match step["op"].as_str().expect("op") {
            "Dock" => {
                let side = match step["side"].as_str().unwrap() {
                    "right" => ColumnSide::Right,
                    other => panic!("unknown side {other}"),
                };
                app.dock
                    .apply_panel_op(PanelOp::Dock {
                        section: section(step["section"].as_str().unwrap()),
                        side,
                    })
                    .expect("dock");
            }
            "ToggleCollapsed" => {
                app.dock
                    .apply_panel_op(PanelOp::ToggleCollapsed {
                        section: section(step["section"].as_str().unwrap()),
                    })
                    .expect("collapse");
            }
            "Move" => {
                app.dock
                    .apply_panel_op(PanelOp::Move {
                        section: section(step["section"].as_str().unwrap()),
                        delta: step["delta"].as_i64().unwrap() as i32,
                    })
                    .expect("move");
            }
            "Float" => {
                let values = step["rect"].as_array().unwrap();
                app.dock
                    .apply_panel_op(PanelOp::Float {
                        section: section(step["section"].as_str().unwrap()),
                        rect: FloatRect {
                            x: values[0].as_f64().unwrap() as f32,
                            y: values[1].as_f64().unwrap() as f32,
                            width: values[2].as_f64().unwrap() as f32,
                            height: values[3].as_f64().unwrap() as f32,
                        },
                        work: work(),
                        origin: FloatOrigin::Api,
                    })
                    .expect("float");
            }
            "set_section_visible" => app.dock.set_section_visible(
                section(step["section"].as_str().unwrap()),
                step["visible"].as_bool().unwrap(),
            ),
            "set_section_height" => app.dock.set_section_height(
                section(step["section"].as_str().unwrap()),
                step["height"].as_f64().unwrap() as f32,
            ),
            "set_size" => app.dock.set_size(
                panel(step["panel"].as_str().unwrap()),
                step["size"].as_f64().unwrap() as f32,
            ),
            "set_visible" => app.dock.set_visible(
                panel(step["panel"].as_str().unwrap()),
                step["visible"].as_bool().unwrap(),
            ),
            other => panic!("unknown op {other}"),
        }
    }
}

/// A store path that does not exist yet, unique to the test.
fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("om9-l3-005-{name}.json"));
    let _ = std::fs::remove_file(&path);
    path
}

fn baseline_bytes() -> String {
    DockLayout::matrix_default().to_json().unwrap()
}

#[test]
fn s1_a_session_keeps_what_the_user_arranged() {
    let document = fixture("positive.json");
    let case = case(&document, "S1-a-session-keeps-what-the-user-arranged");
    let path = temp_path("s1");
    let mut saved = MatrixApp::new(Document::new("persist s1")).unwrap();
    saved.set_dock_layout_path(Some(path.clone()));
    arrange(&mut saved, case["arrangement"].as_array().unwrap());
    saved.set_menus_as_default(); // the documented writer
    let written = saved.dock.to_json().unwrap();

    // A fresh session opening the same file.
    let mut reopened = MatrixApp::new(Document::new("persist s1 again")).unwrap();
    reopened.set_dock_layout_path(Some(path.clone()));
    let diagnostics = reopened.load_dock_layout();
    assert_eq!(case["expect"]["diagnostics"].as_array().unwrap().len(), 0);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let expect = &case["expect"];
    assert_eq!(
        expect["reopened_equals_saved_bytes"], true,
        "the fixture states the round trip is exact"
    );
    assert_eq!(reopened.dock.to_json().unwrap(), written);

    let left: Vec<String> = reopened
        .dock
        .left_sections()
        .iter()
        .map(|id| id.key().to_owned())
        .collect();
    let expected_left: Vec<String> = expect["left_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(left, expected_left);
    let right: Vec<String> = reopened
        .dock
        .right_sections()
        .iter()
        .map(|id| id.key().to_owned())
        .collect();
    let expected_right: Vec<String> = expect["right_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(right, expected_right);
    assert_eq!(reopened.dock.floating_sections(), vec![SectionId::Display]);
    let display = reopened.dock.section(SectionId::Display);
    let rect = expect["display_float"].as_array().unwrap();
    assert_eq!(
        display.float,
        Some(FloatRect {
            x: rect[0].as_f64().unwrap() as f32,
            y: rect[1].as_f64().unwrap() as f32,
            width: rect[2].as_f64().unwrap() as f32,
            height: rect[3].as_f64().unwrap() as f32,
        })
    );
    assert!(reopened.dock.section(SectionId::Projects).collapsed);
    assert!(!reopened.dock.section(SectionId::InfoSettings).visible);
    assert_eq!(
        reopened.dock.section(SectionId::Layers).height,
        expect["layers_height"].as_f64().unwrap() as f32
    );
    assert_eq!(
        reopened.dock.state(PanelId::LeftRail).size,
        expect["left_rail_size"].as_f64().unwrap() as f32
    );
    assert!(reopened.dock.state(PanelId::RightDock).visible);
    assert_eq!(
        expect["right_dock_visible"], true,
        "a session that docked right reopens with the column"
    );
    let home = reopened.dock.section(SectionId::Snaps).home.expect("home");
    assert_eq!(home.side, ColumnSide::Left);
    assert_eq!(
        home.index,
        expect["snaps_home"]["index"].as_u64().unwrap() as u32
    );
    assert!(!reopened.menus_have_unsaved_changes());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn s2_a_written_file_is_a_fixed_point() {
    let document = fixture("positive.json");
    let case = case(&document, "S2-a-written-file-is-a-fixed-point");
    let path = temp_path("s2");
    let mut app = MatrixApp::new(Document::new("persist s2")).unwrap();
    app.set_dock_layout_path(Some(path.clone()));
    arrange(&mut app, case["arrangement"].as_array().unwrap());
    app.set_menus_as_default();
    let first = std::fs::read_to_string(&path).expect("file");

    let mut reopened = MatrixApp::new(Document::new("persist s2 again")).unwrap();
    reopened.set_dock_layout_path(Some(path.clone()));
    reopened.load_dock_layout();
    assert_eq!(
        reopened.dock.to_json().unwrap(),
        app.dock.to_json().unwrap()
    );
    reopened.set_menus_as_default();
    let second = std::fs::read_to_string(&path).expect("file");
    assert_eq!(case["expect"]["second_save_equals_first"], true);
    assert_eq!(first, second, "reopening and saving again must not drift");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn s3_a_kept_file_reopens_the_same_named_ids() {
    let document = fixture("positive.json");
    let case = case(&document, "S3-a-kept-file-reopens-the-same-named-ids");
    let path = temp_path("s3");
    let mut app = MatrixApp::new(Document::new("persist s3")).unwrap();
    app.set_dock_layout_path(Some(path.clone()));
    arrange(&mut app, case["arrangement"].as_array().unwrap());
    app.set_menus_as_default();
    let json: Value = serde_json::from_str(&app.dock.to_json().unwrap()).unwrap();

    let mut panels: Vec<String> = json["panels"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    panels.sort();
    let expected_panels: Vec<String> = case["expect"]["panel_keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(panels, expected_panels);
    let mut sections: Vec<String> = json["sections"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    sections.sort();
    let expected_sections: Vec<String> = case["expect"]["section_keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(sections, expected_sections);
    assert_eq!(case["expect"]["diagnostics"].as_array().unwrap().len(), 0);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn s4_a_session_without_a_kept_file_is_the_baseline() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "S4-a-session-without-a-kept-file-is-the-baseline",
    );
    assert!(case["store"].is_null());
    let mut app = MatrixApp::new(Document::new("persist s4")).unwrap();
    assert!(app.dock_layout_path().is_none());
    let diagnostics = app.load_dock_layout();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(app.dock.to_json().unwrap(), baseline_bytes());
    assert!(!app.menus_have_unsaved_changes());
    assert_eq!(case["expect"]["menus_have_unsaved_changes"], false);
}

#[test]
fn m1_to_m13_a_file_that_cannot_be_honoured_is_repaired_with_a_warning() {
    let document = fixture("negative.json");
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let bytes: Vec<u8> = match case.get("bytes_padding") {
            Some(padding) => {
                let padding = padding.as_u64().unwrap() as usize;
                let mut bytes = b"{\"schema_version\": 2, \"padding\": \"".to_vec();
                bytes.extend(std::iter::repeat_n(b'x', padding));
                bytes.extend_from_slice(b"\"}");
                bytes
            }
            None => case["bytes"].as_str().unwrap().as_bytes().to_vec(),
        };
        let outcome = DockLayout::from_bytes(&bytes);
        if let Some(kind) = case["expect"]["error"].as_str() {
            let error = outcome.expect_err(&format!("{id}: must be refused"));
            let text = format!("{error:?}");
            assert!(text.contains(kind), "{id}: {text} does not contain {kind}");
            assert_eq!(case["expect"]["panics"], false, "{id}");
            continue;
        }
        let (layout, diagnostics) = outcome.unwrap_or_else(|error| panic!("{id}: {error:?}"));
        let expect = &case["expect"];
        if let Some(needle) = expect["diagnostic_contains"].as_str() {
            assert!(
                diagnostics.iter().any(|line| line.contains(needle)),
                "{id}: {diagnostics:?} lacks {needle:?}"
            );
        }
        if let Some(size) = expect["left_rail_size"].as_f64() {
            assert_eq!(layout.state(PanelId::LeftRail).size, size as f32, "{id}");
        }
        if let Some(size) = expect["kept_size"].as_f64() {
            let key = expect["kept_panel"].as_str().unwrap();
            assert_eq!(layout.state(panel(key)).size, size as f32, "{id}");
        }
        if let Some(dock) = expect["display_dock"].as_str() {
            let actual = serde_json::to_value(layout.section(SectionId::Display).dock)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned();
            assert_eq!(actual, dock, "{id}");
        }
        if expect["display_float_is_finite"] == true {
            let rect = layout
                .section(SectionId::Display)
                .float
                .unwrap_or_else(|| panic!("{id}: a floating menu keeps a rectangle"));
            assert!(
                [rect.x, rect.y, rect.width, rect.height]
                    .into_iter()
                    .all(f32::is_finite),
                "{id}"
            );
        }
        if let Some(values) = expect["display_float"].as_array() {
            let rect = layout.section(SectionId::Display).float.expect("rect");
            assert_eq!(
                [rect.x, rect.y, rect.width, rect.height],
                [
                    values[0].as_f64().unwrap() as f32,
                    values[1].as_f64().unwrap() as f32,
                    values[2].as_f64().unwrap() as f32,
                    values[3].as_f64().unwrap() as f32,
                ],
                "{id}"
            );
        }
        if expect["snaps_float_is_some"] == true {
            assert!(layout.section(SectionId::Snaps).float.is_some(), "{id}");
        }
        if expect["orders_are_dense"] == true {
            for side in [DockSide::Left, DockSide::Right, DockSide::Floating] {
                for (index, id) in layout.section_order(side).into_iter().enumerate() {
                    assert_eq!(layout.section(id).order, index as u32, "{} order", id.key());
                }
            }
        }
        if expect["left_order_is_documented"] == true {
            let left: Vec<&'static str> =
                layout.left_sections().iter().map(|id| id.key()).collect();
            assert_eq!(
                left,
                SectionId::ALL.iter().map(|id| id.key()).collect::<Vec<_>>()
            );
        }
        if expect["equals_matrix_default_bytes"] == true {
            assert_eq!(layout.to_json().unwrap(), baseline_bytes(), "{id}");
        }
        if expect["diagnostics"].as_array().is_some() {
            assert!(
                diagnostics.is_empty(),
                "{id}: a document that needs no repair must not warn: {diagnostics:?}"
            );
        }
        assert_eq!(expect["panics"], false, "{id}");
    }
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-01-005/api.md");
    assert!(api.exists(), "{}", api.display());
}
