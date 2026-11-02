// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless UI replay: the real `MatrixApp` widget tree runs on an `egui::Context` without a
//! window. Clicking the line in the perspective viewport must reach the pick path and select it,
//! and the receipt must expose the viewport/panel rectangles and focus after every step.
use om9_app::ui_replay;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use serde_json::json;

#[test]
fn a_click_picks_the_line_in_the_perspective_viewport_without_a_window() {
    let mut document = Document::new("ui replay");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    document.entities.insert(line.id, line);
    let probe = ui_replay::replay(document.clone(), [1280., 800.], 1.0, &[json!({"wait": 1})])
        .expect("headless frames");
    let views = probe["steps"][0]["views"].as_array().expect("view rects");
    assert_eq!(views.len(), 4, "the workbench lays out four viewports");
    let rect = views
        .iter()
        .find(|view| view["view"] == "Perspective")
        .expect("perspective viewport")["rect"]
        .as_array()
        .expect("rect");
    let centre = [
        rect[0].as_f64().unwrap() + rect[2].as_f64().unwrap() / 2.,
        rect[1].as_f64().unwrap() + rect[3].as_f64().unwrap() / 2.,
    ];
    let result = ui_replay::replay(document, [1280., 800.], 1.0, &[json!({"click": centre})])
        .expect("click replay");
    let selection = result["steps"][1]["selection"]
        .as_array()
        .expect("selection receipt");
    assert!(
        selection.iter().any(|item| item["label"] == "line"),
        "clicking the line must select it: {selection:?}"
    );
    assert_eq!(
        result["steps"][1]["active_view"], "Perspective",
        "the pointer makes its viewport active"
    );
    assert!(
        result["steps"][1]["panels"].is_array(),
        "the receipt always carries the panel rectangle list"
    );
}

#[test]
fn delete_removes_the_selected_line() {
    let mut document = Document::new("delete replay");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    document.entities.insert(line.id, line);
    let probe = ui_replay::replay(document.clone(), [1280., 800.], 1.0, &[json!({"wait":1})]).unwrap();
    let rect = probe["steps"][0]["views"].as_array().unwrap().iter().find(|v| v["view"]=="Perspective").unwrap()["rect"].as_array().unwrap();
    let centre = [rect[0].as_f64().unwrap() + rect[2].as_f64().unwrap()/2., rect[1].as_f64().unwrap() + rect[3].as_f64().unwrap()/2.];
    let result = ui_replay::replay(document, [1280., 800.], 1.0, &[json!({"click": centre}), json!({"key":"Delete"})]).unwrap();
    let after = &result["steps"][2];
    assert_eq!(after["objects"], 0, "Delete must remove the selected line: {}", after);
    assert_eq!(after["undo"], 1, "one Delete transaction: {}", after);
}
