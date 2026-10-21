// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-002 (Current Layer And Assignment UI),
//! driven by `implementation/fixtures/M01/OM9-L3-03-14-002/`.
use om9_app::layer_command::{LayerChoice, LayerProperty};
use om9_app::session::{Controller, InvocationOrigin, LINE};
use om9_core::budget::Limits;
use om9_core::layers::PresetSlot;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-002")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn controller() -> Controller {
    Controller::new(Document::new("current layer"), Limits::default()).expect("controller")
}

fn slot(number: u64) -> PresetSlot {
    PresetSlot::new(number as u8).expect("bounded slot")
}

fn commit_line(controller: &mut Controller) {
    controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("dispatch");
    controller
        .accept_world_point([0., 0., 0.])
        .expect("first point");
    controller
        .accept_world_point([10., 0., 0.])
        .expect("second point");
}

fn only_object(controller: &Controller) -> om9_core::identity::ObjectId {
    *controller
        .document()
        .entities
        .keys()
        .next()
        .expect("one object")
}

fn sources(
    controller: &Controller,
    object: om9_core::identity::ObjectId,
) -> Vec<om9_core::identity::SourceStamp> {
    controller
        .collect_object_sources(std::iter::once(object))
        .expect("sources")
}

#[test]
fn c1_to_c4_the_panel_and_the_model_agree_on_the_current_layer() {
    let document = fixture("positive.json");

    // C1: a preset becomes a real layer and becomes current in one step.
    let c1 = &document["cases"][0];
    let mut controller = controller();
    let slot_number = c1["slot"].as_u64().unwrap();
    controller
        .set_current_layer(LayerChoice::Slot(slot(slot_number)))
        .expect("make Metals current");
    let id = controller
        .layer_id(slot(slot_number))
        .expect("the preset layer exists");
    assert_eq!(controller.document().current_layer, Some(id));
    assert!(controller.document().layers[&id].preset_slot == Some(slot(slot_number)));
    assert_eq!(c1["expect"]["panel_and_model_agree"], true);

    // C2: assigning a selected object moves it to that layer and keeps the
    // current layer where it was; the assignment is one journal entry.
    let c2 = &document["cases"][1];
    commit_line(&mut controller);
    let object = only_object(&controller);
    let before_journal = controller.coordinator().undo_len();
    let assign_slot = c2["slot"].as_u64().unwrap();
    controller
        .assign_layer(slot(assign_slot), sources(&controller, object))
        .expect("assign to Gems");
    let target = controller.layer_id(slot(assign_slot)).expect("Gems exists");
    assert_eq!(
        controller.document().entities[&object].attributes.layer,
        Some(target)
    );
    assert_eq!(
        controller.document().current_layer,
        Some(id),
        "assignment does not steal the current layer"
    );
    assert_eq!(controller.coordinator().undo_len(), before_journal + 1);
    assert_eq!(c2["expect"]["assign_is_one_transaction"], true);

    // C3: the panel reads the same id the document holds.
    let c3 = &document["cases"][2];
    assert_eq!(
        controller.layer_id(slot(slot_number)),
        controller.document().current_layer
    );
    assert_eq!(c3["expect"]["panel_current_equals_document_current"], true);

    // C4: None clears the current layer.
    let c4 = &document["cases"][3];
    controller
        .set_current_layer(LayerChoice::None)
        .expect("clear");
    assert!(controller.document().current_layer.is_none());
    assert_eq!(c4["expect"]["current_layer_none"], true);
}

#[test]
fn l1_to_l4_a_locked_layer_refuses_assignment_and_still_accepts_being_current() {
    let document = fixture("negative.json");

    // L1/L2: assigning onto a locked layer is refused, explicitly, and nothing
    // moves.
    let l1 = &document["cases"][0];
    let l2 = &document["cases"][1];
    let mut controller = controller();
    commit_line(&mut controller);
    let object = only_object(&controller);
    let layer_before = controller.document().entities[&object].attributes.layer;
    let lock_slot = l1["lock_slot"].as_u64().unwrap();
    controller
        .set_layer_property(slot(lock_slot), LayerProperty::Locked(true))
        .expect("lock Gems");
    // The lock itself is a transaction; what must add nothing is the REFUSAL.
    let journal_before = controller.coordinator().undo_len();
    let error = controller
        .assign_layer(
            slot(l1["assign_slot"].as_u64().unwrap()),
            sources(&controller, object),
        )
        .expect_err("a locked layer must refuse the assignment");
    assert_eq!(
        match &error {
            om9_core::Error::Permission(_) => "Permission",
            other => panic!("unexpected {other}"),
        },
        l1["expect"]["error"].as_str().unwrap()
    );
    let message = format!("{error}");
    for needle in l2["expect"]["message_contains"].as_array().unwrap() {
        assert!(
            message.contains(needle.as_str().unwrap()),
            "{message} lacks {needle}"
        );
    }
    assert_eq!(
        controller.document().entities[&object].attributes.layer,
        layer_before,
        "a refused assignment changes no layer"
    );
    assert_eq!(controller.coordinator().undo_len(), journal_before);

    // L3: assigning with nothing selected is a no-op that succeeds.
    let l3 = &document["cases"][2];
    let journal = controller.coordinator().undo_len();
    controller
        .assign_layer(slot(1), Vec::new())
        .expect("nothing to assign is not an error");
    assert_eq!(controller.coordinator().undo_len(), journal);
    assert_eq!(l3["expect"]["ok"], true);

    // L4: the palette never creates a loose preset; it creates the group.
    let l4 = &document["cases"][3];
    assert!(!l4["expect"]["detail"].as_str().unwrap().is_empty());
    let preset_layers: Vec<u64> = controller
        .document()
        .layers
        .values()
        .filter_map(|layer| layer.preset_slot.map(|slot| slot.number() as u64))
        .collect();
    for number in &preset_layers {
        let first = ((number - 1) / 4) * 4 + 1;
        for member in first..first + 4 {
            assert!(
                preset_layers.contains(&member),
                "preset {number} was created without its group ({member} missing)"
            );
        }
    }
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-03-14-002/api.md");
    assert!(api.exists(), "{}", api.display());
}
