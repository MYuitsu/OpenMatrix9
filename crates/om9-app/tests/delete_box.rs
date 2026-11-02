// SPDX-License-Identifier: LGPL-2.1-or-later
//! A selected Box must delete in one transaction, both through the controller and after a
//! viewport click plus the Delete key.
use om9_app::session::Controller;
use om9_app::ui_replay;
use om9_core::budget::Limits;
use om9_core::model::Document;
use serde_json::json;

fn box_document() -> Document {
    let mut controller = Controller::new(Document::new("box delete"), Limits::default()).unwrap();
    controller.submit("Box").unwrap();
    controller.submit("0,0,0").unwrap();
    controller.submit("10,20,0").unwrap();
    controller.submit("30").unwrap();
    assert_eq!(controller.document().entities.len(), 1, "box committed");
    controller.document().clone()
}

#[test]
fn a_selected_box_deletes_through_the_controller() {
    let mut controller = Controller::new(box_document(), Limits::default()).unwrap();
    let id = *controller.document().entities.keys().next().unwrap();
    controller.select_objects(&[id]).expect("box selection");
    assert_eq!(controller.delete_objects(&[id]).unwrap(), 1);
    assert!(controller.document().entities.is_empty());
}

#[test]
fn a_box_selected_by_click_then_delete_leaves_an_empty_document() {
    let document = box_document();
    let probe = ui_replay::replay(document.clone(), [1280., 800.], 1.0, &[json!({"wait":1})]).unwrap();
    let rect = probe["steps"][0]["views"].as_array().unwrap().iter().find(|v| v["view"]=="Perspective").unwrap()["rect"].as_array().unwrap();
    let centre = [rect[0].as_f64().unwrap() + rect[2].as_f64().unwrap()/2., rect[1].as_f64().unwrap() + rect[3].as_f64().unwrap()/2.];
    let result = ui_replay::replay(document, [1280., 800.], 1.0, &[json!({"click": centre}), json!({"key":"Delete"})]).unwrap();
    let step = &result["steps"][2];
    assert_eq!(step["objects"], 0, "Delete must remove the selected box: {step}");
    assert_eq!(step["undo"], 1, "one Delete transaction: {step}");
}