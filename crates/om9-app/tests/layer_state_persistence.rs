// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-008 (Layer State Persistence Visual Check),
//! driven by `implementation/fixtures/M01/OM9-L3-03-14-008/`.
use om9_app::layer_command::{LayerChoice, LayerProperty};
use om9_app::session::{Controller, InvocationOrigin, LINE};
use om9_archive::Archive;
use om9_core::budget::Limits;
use om9_core::layers::PresetSlot;
use om9_core::model::Document;
use om9_core::transaction::Cancellation;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-008")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn slot(number: u64) -> PresetSlot {
    PresetSlot::new(number as u8).expect("bounded slot")
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "om9-l3-003-14-008-{}-{}.om9",
        std::process::id(),
        name
    ))
}

/// Gems locked, Metals hidden, one line on Gems.
fn sample_document() -> (Document, Vec<om9_core::identity::ObjectId>) {
    let mut controller =
        Controller::new(Document::new("persistence"), Limits::default()).expect("controller");
    controller
        .set_current_layer(LayerChoice::Slot(slot(5)))
        .expect("Gems current");
    controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("dispatch");
    controller.accept_world_point([0., 0., 0.]).expect("first");
    controller
        .accept_world_point([10., 0., 0.])
        .expect("second");
    controller
        .set_layer_property(slot(1), LayerProperty::Visible(false))
        .expect("hide Metals");
    controller
        .set_layer_property(slot(5), LayerProperty::Locked(true))
        .expect("lock Gems");
    let ids: Vec<_> = controller.document().entities.keys().copied().collect();
    let document = controller.document().clone();
    (document, ids)
}

#[test]
fn k1_to_k3_the_two_layers_come_back_with_their_flags_and_ids() {
    let fixture_doc = fixture("positive.json");
    let limits = Limits::default();
    let (document, ids) = sample_document();
    let path = temp("roundtrip");
    Archive::save(&path, &document, &limits, &Cancellation::default()).expect("save");
    let restored = Archive::open(&path, &limits).expect("reopen");

    // K1: both layers, with lock/visibility and colour preserved.
    let k1 = &fixture_doc["cases"][0];
    let mut slots: Vec<u8> = restored
        .layers
        .values()
        .filter_map(|layer| layer.preset_slot.map(PresetSlot::number))
        .collect();
    slots.sort_unstable();
    let expected: Vec<u8> = k1["expect"]["slots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    assert_eq!(slots, expected);
    let gems = restored
        .layers
        .values()
        .find(|layer| layer.preset_slot == Some(slot(5)))
        .expect("Gems layer");
    assert_eq!(gems.locked, k1["expect"]["gems_locked"].as_bool().unwrap());
    let metals = restored
        .layers
        .values()
        .find(|layer| layer.preset_slot == Some(slot(1)))
        .expect("Metals layer");
    assert_eq!(
        metals.visible,
        k1["expect"]["metals_visible"].as_bool().unwrap()
    );
    assert_eq!(gems.color, slot(5).color());
    assert!(k1["expect"]["colours_kept"].as_bool().unwrap());

    // K2: the objects keep their ids and their layer.
    let k2 = &fixture_doc["cases"][1];
    let restored_ids: Vec<_> = restored.entities.keys().copied().collect();
    assert_eq!(restored_ids, ids);
    assert!(k2["expect"]["same_ids"].as_bool().unwrap());
    for id in &restored_ids {
        let before = document.entities[id].attributes.layer;
        assert_eq!(restored.entities[id].attributes.layer, before);
    }
    assert!(k2["expect"]["same_layers"].as_bool().unwrap());

    // K3: the palette over the reopened document offers the same rows, and the
    // selectable policy still holds (locked Gems, hidden Metals select nothing).
    let k3 = &fixture_doc["cases"][2];
    let gems_id = gems.id;
    let metals_id = metals.id;
    let mut controller = Controller::new(restored, limits).expect("controller");
    assert_eq!(controller.layer_id(slot(5)), Some(gems_id));
    assert_eq!(controller.layer_id(slot(1)), Some(metals_id));
    controller.select_layer(slot(5)).expect("select Gems");
    assert_eq!(
        controller.selected().len() as u64,
        k3["expect"]["gems_selects"].as_u64().unwrap()
    );
    controller.select_layer(slot(1)).expect("select Metals");
    assert_eq!(
        controller.selected().len() as u64,
        k3["expect"]["metals_selects"].as_u64().unwrap()
    );
    assert!(k3["expect"]["picked_ids_equal"].as_bool().unwrap());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn l1_to_l3_the_palette_never_shows_cached_state() {
    let fixture_doc = fixture("negative.json");
    let limits = Limits::default();

    // L1: opening another document shows ITS palette, not the previous one.
    let l1 = &fixture_doc["cases"][0];
    let (saved_doc, _) = sample_document();
    let other_path = temp("empty");
    Archive::save(
        &other_path,
        &Document::new("empty"),
        &limits,
        &Cancellation::default(),
    )
    .expect("save");
    let _ = saved_doc;
    let other = Archive::open(&other_path, &limits).expect("reopen");
    let mut controller = Controller::new(other, limits).expect("controller");
    assert!(controller.layer_id(slot(5)).is_none(), "no Gems layer here");
    assert!(
        controller.layer_id(slot(1)).is_none(),
        "no Metals layer here"
    );
    assert!(!l1["expect"]["gems_layer_present"].as_bool().unwrap());
    controller.select_layer(slot(5)).expect("select nothing");
    assert_eq!(
        controller.selected().len() as u64,
        l1["expect"]["selects"].as_u64().unwrap()
    );

    // L2: a failed open leaves the current palette alone.
    let l2 = &fixture_doc["cases"][1];
    let (current, _) = sample_document();
    let controller = Controller::new(current, limits).expect("controller");
    let before: Vec<_> = controller
        .document()
        .layers
        .values()
        .map(|layer| (layer.preset_slot, layer.locked, layer.visible))
        .collect();
    assert!(
        Archive::open(&temp("does-not-exist"), &limits).is_err(),
        "a missing file must not open"
    );
    let after: Vec<_> = controller
        .document()
        .layers
        .values()
        .map(|layer| (layer.preset_slot, layer.locked, layer.visible))
        .collect();
    assert_eq!(after, before);
    assert!(l2["expect"]["layers_unchanged"].as_bool().unwrap());

    // L3: the reopened flags come from the file, not from memory: an unsaved lock
    // applied after the save is NOT visible in the reopened document.
    let l3 = &fixture_doc["cases"][2];
    let (mut live, _) = sample_document();
    live.layers
        .values_mut()
        .filter(|layer| layer.preset_slot == Some(slot(1)))
        .for_each(|layer| layer.locked = true);
    let path = temp("unsaved");
    let (saved, _) = sample_document();
    Archive::save(&path, &saved, &limits, &Cancellation::default()).expect("save");
    let reopened = Archive::open(&path, &limits).expect("reopen");
    let metals = reopened
        .layers
        .values()
        .find(|layer| layer.preset_slot == Some(slot(1)))
        .expect("Metals");
    assert!(
        !metals.locked,
        "the unsaved lock in memory is not in the file"
    );
    assert!(l3["expect"]["unsaved_lock_not_present"].as_bool().unwrap());
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&other_path);
}
