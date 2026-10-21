// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-004 (Select All Objects By Layer), driven by
//! `implementation/fixtures/M01/OM9-L3-03-14-004/`.
use om9_app::layer_command::{LayerChoice, LayerProperty};
use om9_app::session::{Controller, InvocationOrigin, LINE};
use om9_core::budget::Limits;
use om9_core::layers::PresetSlot;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-004")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn slot(number: u64) -> PresetSlot {
    PresetSlot::new(number as u8).expect("bounded slot")
}

/// Two lines on Gems, one line on Metals.
fn two_layers() -> Controller {
    let mut controller =
        Controller::new(Document::new("select by layer"), Limits::default()).expect("controller");
    controller
        .set_current_layer(LayerChoice::Slot(slot(5)))
        .expect("Gems current");
    for end in [[10., 0., 0.], [10., 4., 0.]] {
        controller
            .dispatch(LINE, InvocationOrigin::Cmd)
            .expect("dispatch");
        controller.accept_world_point([0., 0., 0.]).expect("first");
        controller.accept_world_point(end).expect("second");
    }
    controller
        .set_current_layer(LayerChoice::Slot(slot(1)))
        .expect("Metals current");
    controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("dispatch");
    controller.accept_world_point([0., 20., 0.]).expect("first");
    controller
        .accept_world_point([10., 20., 0.])
        .expect("second");
    controller.clear_selection();
    controller
}

fn gems_objects(controller: &Controller) -> Vec<om9_core::identity::ObjectId> {
    let gems = controller.layer_id(slot(5)).expect("Gems");
    controller
        .document()
        .entities
        .values()
        .filter(|entity| entity.attributes.layer == Some(gems))
        .map(|entity| entity.id)
        .collect()
}

#[test]
fn a1_to_a4_selecting_the_layer_selects_exactly_its_editable_objects() {
    let document = fixture("positive.json");
    let mut controller = two_layers();

    // A1: both objects of the current layer are selected, and the selection is
    // replaced, not appended.
    let a1 = &document["cases"][0];
    controller.select_layer(slot(5)).expect("select Gems");
    assert_eq!(
        controller.selected().len() as u64,
        a1["expect"]["selected"].as_u64().unwrap()
    );
    controller.select_layer(slot(5)).expect("select Gems again");
    assert_eq!(
        controller.selected().len() as u64,
        a1["expect"]["selected"].as_u64().unwrap(),
        "a second gesture replaces the selection"
    );
    assert_eq!(a1["expect"]["selection_replaces_previous"], true);

    // A2: the Metals object is not part of it.
    let a2 = &document["cases"][1];
    let metals = controller.layer_id(slot(1)).expect("Metals");
    let metals_object = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.attributes.layer == Some(metals))
        .map(|entity| entity.id)
        .expect("a Metals object");
    assert_eq!(
        controller.selected().contains(&metals_object),
        a2["expect"]["other_layer_object_selected"]
            .as_bool()
            .unwrap()
    );

    // A3: hiding one Gems object takes it out of the gesture.
    let a3 = &document["cases"][2];
    let hidden = *gems_objects(&controller).first().expect("a Gems object");
    let sources = controller
        .collect_object_sources(std::iter::once(hidden))
        .expect("sources");
    controller
        .set_objects_hidden(sources, true)
        .expect("hide one object");
    controller.select_layer(slot(5)).expect("select Gems");
    assert_eq!(
        controller.selected().len() as u64,
        a3["expect"]["selected_after_hiding_one"].as_u64().unwrap()
    );
    assert!(!controller.selected().contains(&hidden));

    // A4: the group gesture follows the same selectable policy.
    let a4 = &document["cases"][3];
    assert!(
        a4["expect"]["select_group_uses_the_same_policy"]
            .as_bool()
            .unwrap()
    );
    let selectable = controller
        .selectable_layer_member_count(slot(5))
        .expect("count");
    assert_eq!(selectable, 1, "only the visible object is selectable");
}

#[test]
fn b1_to_b5_hidden_and_locked_objects_respect_the_selectable_policy() {
    let document = fixture("negative.json");
    let b1 = &document["cases"][0];
    let b2 = &document["cases"][1];
    let b3 = &document["cases"][2];
    let b4 = &document["cases"][3];
    let b5 = &document["cases"][4];

    // B1: a locked layer selects nothing, and the previous selection is cleared
    // rather than silently kept.
    let mut controller = two_layers();
    controller.select_layer(slot(5)).expect("select Gems");
    assert!(!controller.selected().is_empty());
    controller
        .set_layer_property(slot(5), LayerProperty::Locked(true))
        .expect("lock Gems");
    let journal = controller.coordinator().undo_len();
    controller
        .select_layer(slot(5))
        .expect("select a locked layer");
    assert_eq!(
        controller.selected().len() as u64,
        b1["expect"]["selected"].as_u64().unwrap()
    );
    assert!(!b3["expect"]["detail"].is_null());
    assert!(controller.selected().is_empty(), "not a stale selection");

    // B4: selecting never writes to the document.
    assert_eq!(controller.coordinator().undo_len(), journal);
    assert!(b4["expect"]["journal_unchanged"].as_bool().unwrap());

    // B2: a hidden layer selects nothing either.
    controller
        .set_layer_property(slot(5), LayerProperty::Locked(false))
        .expect("unlock");
    controller
        .set_layer_property(slot(5), LayerProperty::Visible(false))
        .expect("hide");
    controller
        .select_layer(slot(5))
        .expect("select a hidden layer");
    assert_eq!(
        controller.selected().len() as u64,
        b2["expect"]["selected"].as_u64().unwrap()
    );

    // B5: a slot with no layer selects nothing and is not an error.
    let fresh = Controller::new(Document::new("empty"), Limits::default()).expect("controller");
    let mut fresh = fresh;
    fresh
        .select_layer(slot(9))
        .expect("no layer is not an error");
    assert_eq!(
        fresh.selected().len() as u64,
        b5["expect"]["selected"].as_u64().unwrap()
    );
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-03-14-004/api.md");
    assert!(api.exists(), "{}", api.display());
}
