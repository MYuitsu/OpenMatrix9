// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-003 (Layer Visible And Locked Controls),
//! driven by `implementation/fixtures/M01/OM9-L3-03-14-003/`.
use om9_app::layer_command::LayerProperty;
use om9_app::session::{Controller, InvocationOrigin, LINE};
use om9_core::budget::Limits;
use om9_core::layers::PresetSlot;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-003")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn slot(number: u64) -> PresetSlot {
    PresetSlot::new(number as u8).expect("bounded slot")
}

/// A controller with one Gems layer holding one object.
fn gems_with_object() -> (Controller, om9_core::identity::ObjectId) {
    let mut controller =
        Controller::new(Document::new("visibility"), Limits::default()).expect("controller");
    controller
        .set_current_layer(om9_app::layer_command::LayerChoice::Slot(slot(5)))
        .expect("make Gems current");
    controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("dispatch");
    controller.accept_world_point([0., 0., 0.]).expect("first");
    controller
        .accept_world_point([10., 0., 0.])
        .expect("second");
    let object = *controller
        .document()
        .entities
        .keys()
        .next()
        .expect("object");
    (controller, object)
}

fn scene_visible(controller: &Controller, object: om9_core::identity::ObjectId) -> bool {
    controller
        .scene()
        .expect("scene")
        .objects
        .iter()
        .find(|entry| entry.root == object)
        .map(|entry| entry.visible)
        .unwrap_or(false)
}

#[test]
fn s1_to_s5_visible_and_locked_change_the_model_the_viewport_and_picking() {
    let document = fixture("positive.json");
    let slot_number = document["layer"]["slot"].as_u64().unwrap();
    let (mut controller, object) = gems_with_object();
    let gems = controller.layer_id(slot(slot_number)).expect("Gems");

    // S1: hiding the layer hides its objects at once.
    let s1 = &document["cases"][0];
    controller
        .set_layer_property(slot(slot_number), LayerProperty::Visible(false))
        .expect("hide Gems");
    let attributes = controller
        .document()
        .display_attributes(object)
        .expect("attributes");
    assert_eq!(
        attributes.visible,
        s1["expect"]["display_visible"].as_bool().unwrap()
    );
    assert_eq!(
        scene_visible(&controller, object),
        s1["expect"]["scene_visible"].as_bool().unwrap()
    );
    // A different, empty layer stays visible: hiding is per layer.
    assert!(
        !controller
            .document()
            .layers
            .get(&gems)
            .is_some_and(|layer| layer.visible),
        "the toggled layer itself is the hidden one"
    );

    // S2: nothing on the hidden layer can be picked or selected.
    let s2 = &document["cases"][1];
    assert_eq!(
        attributes.selectable,
        s2["expect"]["pick_capable"].as_bool().unwrap()
    );
    controller
        .select_layer(slot(slot_number))
        .expect("select Gems");
    assert_eq!(
        controller.selected().len() as u64,
        s2["expect"]["select_layer_selects"].as_u64().unwrap()
    );

    // S3: showing it again brings both back.
    let s3 = &document["cases"][2];
    controller
        .set_layer_property(slot(slot_number), LayerProperty::Visible(true))
        .expect("show Gems");
    let attributes = controller
        .document()
        .display_attributes(object)
        .expect("attributes");
    assert_eq!(
        attributes.visible,
        s3["expect"]["display_visible"].as_bool().unwrap()
    );
    assert_eq!(
        scene_visible(&controller, object),
        s3["expect"]["scene_visible"].as_bool().unwrap()
    );
    assert_eq!(
        attributes.selectable,
        s3["expect"]["pick_capable"].as_bool().unwrap()
    );

    // S4: locking keeps the objects visible but stops editing.
    let s4 = &document["cases"][3];
    controller
        .set_layer_property(slot(slot_number), LayerProperty::Locked(true))
        .expect("lock Gems");
    let attributes = controller
        .document()
        .display_attributes(object)
        .expect("attributes");
    assert_eq!(
        attributes.visible,
        s4["expect"]["display_visible"].as_bool().unwrap()
    );
    assert_eq!(
        attributes.editable,
        s4["expect"]["editable"].as_bool().unwrap()
    );

    // S5: unlocking restores editability.
    let s5 = &document["cases"][4];
    controller
        .set_layer_property(slot(slot_number), LayerProperty::Locked(false))
        .expect("unlock Gems");
    let attributes = controller
        .document()
        .display_attributes(object)
        .expect("attributes");
    assert_eq!(
        attributes.editable,
        s5["expect"]["editable"].as_bool().unwrap()
    );
}

#[test]
fn t1_to_t4_a_locked_edit_is_refused_without_mutation() {
    let document = fixture("negative.json");
    let (mut controller, object) = gems_with_object();
    let journal = controller.coordinator().undo_len();
    let entities = controller.document().entities.len();
    let revision = controller.document().context_revision;
    controller
        .set_layer_property(slot(5), LayerProperty::Locked(true))
        .expect("lock Gems");
    let journal_after_lock = controller.coordinator().undo_len();
    let sources = controller
        .collect_object_sources(std::iter::once(object))
        .expect("sources");

    // T1/T2: the edit is refused, the reason names the lock, nothing moved.
    let t1 = &document["cases"][0];
    let t2 = &document["cases"][1];
    let error = controller
        .assign_layer(slot(1), sources)
        .expect_err("a locked object cannot be moved");
    assert_eq!(
        match &error {
            om9_core::Error::Permission(_) => "Permission",
            other => panic!("unexpected {other}"),
        },
        t1["expect"]["error"].as_str().unwrap()
    );
    let message = format!("{error}");
    for needle in t2["expect"]["message_contains"].as_array().unwrap() {
        assert!(message.contains(needle.as_str().unwrap()), "{message}");
    }
    assert_eq!(controller.document().entities.len(), entities);
    assert_eq!(controller.coordinator().undo_len(), journal_after_lock);

    // T3: hiding is a property change, not a document edit.
    let t3 = &document["cases"][2];
    controller
        .set_layer_property(slot(5), LayerProperty::Visible(false))
        .expect("hide Gems");
    assert!(!controller.document().layers.is_empty());
    assert_eq!(t3["expect"]["journal_unchanged"], true);
    assert!(journal_after_lock >= journal);

    // T4: a locked/hidden layer still describes its objects for inspection.
    let t4 = &document["cases"][3];
    assert!(
        t4["expect"]["display_attributes_available"]
            .as_bool()
            .unwrap()
    );
    let attributes = controller
        .document()
        .display_attributes(object)
        .expect("attributes");
    assert!(
        !attributes.name.is_empty(),
        "a locked layer still names its objects"
    );
    assert!(!attributes.visible);
    let _ = revision;
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-03-14-003/api.md");
    assert!(api.exists(), "{}", api.display());
}
