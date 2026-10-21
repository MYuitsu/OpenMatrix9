// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-001 (Matrix Layer Category Palette), driven
//! by `implementation/fixtures/M01/OM9-L3-03-14-001/`.
use om9_app::ui::MatrixApp;
use om9_core::layers::{LayerCategory, PaletteGroup, PresetSlot};
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-001")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| entry.as_str().expect("string").to_owned())
        .collect()
}

#[test]
fn p1_and_p2_the_palette_has_the_seven_categories_and_the_four_groups() {
    let document = fixture("positive.json");
    let p1 = &document["cases"][0];
    let labels: Vec<String> = LayerCategory::ALL
        .iter()
        .map(|category| category.label().to_owned())
        .collect();
    assert_eq!(labels, strings(&p1["expect"]["labels"]));
    // The fixture's own category list must be the model's.
    assert_eq!(labels, strings(&document["categories"]));

    let p2 = &document["cases"][1];
    let groups: Vec<String> = PaletteGroup::ALL
        .iter()
        .map(|group| group.label().to_owned())
        .collect();
    assert_eq!(groups, strings(&p2["expect"]["labels"]));
    let builders: Vec<String> = PaletteGroup::Builders
        .categories()
        .iter()
        .map(|category| category.label().to_owned())
        .collect();
    assert_eq!(builders, strings(&p2["expect"]["builders"]));
}

#[test]
fn p3_every_slot_maps_to_its_documented_category() {
    let document = fixture("positive.json");
    let p3 = &document["cases"][2];
    for (number, expected) in p3["slots"].as_object().expect("slots") {
        let number: u8 = number.parse().expect("slot number");
        let slot = PresetSlot::new(number).expect("bounded slot");
        assert_eq!(
            slot.category_label(),
            expected.as_str().unwrap(),
            "slot {number}"
        );
    }
    for number in p3["expect"]["unmapped_numbers"].as_array().unwrap() {
        let number = number.as_u64().unwrap() as u8;
        assert!(
            PresetSlot::new(number).is_err(),
            "{number} must not be a slot"
        );
    }
}

#[test]
fn p4_a_category_lists_the_slots_it_shows() {
    let document = fixture("positive.json");
    let p4 = &document["cases"][3];
    for category in LayerCategory::ALL {
        let expected: Vec<u64> = p4["expect"][category.label()]
            .as_array()
            .expect("expected slots")
            .iter()
            .map(|value| value.as_u64().unwrap())
            .collect();
        let actual: Vec<u64> = category.slots().iter().map(|n| *n as u64).collect();
        assert_eq!(actual, expected, "{}", category.label());
        for number in category.slots() {
            let slot = PresetSlot::new(*number).expect("slot");
            assert_eq!(slot.category(), Some(category));
            assert_eq!(slot.palette_group(), Some(category.group()));
        }
    }
}

#[test]
fn p5_the_palette_model_reaches_the_application() {
    let document = fixture("positive.json");
    let p5 = &document["cases"][4];
    let app = MatrixApp::new(Document::new("palette")).expect("app");
    let labels: Vec<String> = app
        .palette_categories()
        .iter()
        .map(|label| (*label).to_owned())
        .collect();
    assert_eq!(labels, strings(&p5["expect"]["app_categories"]));
    assert_eq!(app.palette_slot_label(1), Some("Metals"));
    assert_eq!(app.palette_slot_label(13), Some("Heads"));
    assert_eq!(app.palette_slot_label(17), Some("User"));
    assert_eq!(app.palette_slot_label(33), Some("Lights"));
    assert_eq!(app.palette_slot_label(34), None);
}

#[test]
fn n1_to_n6_a_display_label_is_not_an_identity() {
    let document = fixture("negative.json");

    // N1: two slots may show the same text and stay different layers.
    let n1 = &document["cases"][0];
    let first = PresetSlot::new(n1["slots"][0].as_u64().unwrap() as u8).expect("slot");
    let second = PresetSlot::new(n1["slots"][1].as_u64().unwrap() as u8).expect("slot");
    assert_ne!(first, second);
    assert_eq!(n1["expect"]["distinct_identity"], true);
    // A custom slot can carry arbitrary text without becoming the preset.
    assert!(second.validate_label("Metal 01").is_err());

    // N2/N3: a custom slot may not take a reserved label, but any other is fine.
    let n2 = &document["cases"][1];
    let custom = PresetSlot::new(n2["slot"].as_u64().unwrap() as u8).expect("slot");
    let error = custom
        .validate_label(n2["label"].as_str().unwrap())
        .expect_err("a reserved label is refused");
    assert_eq!(
        match &error {
            om9_core::Error::Invalid(_) => "Invalid",
            other => panic!("unexpected {other}"),
        },
        n2["expect"]["error"].as_str().unwrap()
    );
    let n3 = &document["cases"][2];
    assert!(custom.validate_label(n3["label"].as_str().unwrap()).is_ok());

    // N4: the identity is the number, so it survives a serialisation round trip.
    let n4 = &document["cases"][3];
    let json = serde_json::to_string(&custom).expect("serialize");
    let restored: PresetSlot = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored, custom);
    assert_eq!(n4["expect"]["serde_round_trip"], true);

    // N5: a number outside 1..=33 is not a slot.
    let n5 = &document["cases"][4];
    for number in n5["numbers"].as_array().unwrap() {
        let number = number.as_u64().unwrap() as u8;
        assert!(PresetSlot::new(number).is_err(), "{number}");
    }

    // N6: Lights is labelled but is not one of the seven categories.
    let n6 = &document["cases"][5];
    let lights = PresetSlot::new(n6["slot"].as_u64().unwrap() as u8).expect("slot");
    assert_eq!(
        lights.category_label(),
        n6["expect"]["label"].as_str().unwrap()
    );
    assert!(lights.category().is_none());
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-03-14-001/api.md");
    assert!(api.exists(), "{}", api.display());
}
