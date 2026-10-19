// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-001 (Command IDs And Alias Registry), driven
//! by the fixtures in `implementation/fixtures/M01/OM9-L3-01-02-001/`.
use om9_core::capability::{Backend, CapabilityRegistry, Operation, RegistryError};
use om9_core::geometry::GeometryKind;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-02-001")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn geometry_kind(name: &str) -> GeometryKind {
    match name {
        "Curve" => GeometryKind::Curve,
        "Point" => GeometryKind::Point,
        other => panic!("unknown geometry kind {other}"),
    }
}

fn operation(value: &Value) -> Operation {
    Operation {
        id: value["id"].as_str().expect("id").to_owned(),
        feature: value["feature"].as_str().expect("feature").to_owned(),
        aliases: value["aliases"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|alias| alias.as_str().unwrap().to_owned())
                    .collect()
            })
            .unwrap_or_default(),
        input_types: value["inputs"]
            .as_array()
            .expect("inputs")
            .iter()
            .map(|kind| geometry_kind(kind.as_str().unwrap()))
            .collect(),
        // The descriptor's declared signature; this item (L3-01-02-001) only
        // registers and resolves, so the extra fields are inert here.
        prompt: value["feature"].as_str().unwrap_or_default().to_owned(),
        requires_input: false,
        output: Vec::new(),
        preconditions: Default::default(),
        options: value["options"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|option| option.as_str().unwrap().to_owned())
                    .collect()
            })
            .unwrap_or_default(),
        backend: Backend::Rust {
            version: "1".into(),
        },
        prepare: true,
        preview: true,
        commit: true,
        receipts: Vec::new(),
    }
}

fn registry_from(document: &Value) -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::default();
    for value in document["operations"].as_array().expect("operations") {
        registry.try_register(operation(value)).expect("register");
    }
    registry
}

#[test]
fn r1_to_r6_a_name_resolves_to_one_descriptor() {
    let document = fixture("positive.json");
    let registry = registry_from(&document);
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if let Some(name) = case["name"].as_str() {
            let resolved = registry
                .resolve(name)
                .unwrap_or_else(|error| panic!("{id}: {error}"));
            if let Some(expected) = case["expect"]["id"].as_str() {
                assert_eq!(resolved.id, expected, "{id}");
            }
            if let Some(repeat) = case["repeat"].as_u64() {
                let expected = case["expect"]["ids"].as_array().expect("ids")[0]
                    .as_str()
                    .expect("id");
                for _ in 0..repeat {
                    assert_eq!(registry.resolve(name).unwrap().id, expected, "{id}");
                }
            }
        }
        if let Some(names) = case["names"].as_array() {
            let expected: Vec<&str> = case["expect"]["ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap())
                .collect();
            for (index, name) in names.iter().enumerate() {
                let resolved = registry.resolve(name.as_str().unwrap()).unwrap();
                assert_eq!(resolved.id, expected[index], "{id}");
            }
            assert_eq!(case["expect"]["aliases_kept"], true);
            assert!(!registry.resolve("Ln").unwrap().aliases.is_empty(), "{id}");
        }
    }
}

#[test]
fn r7_an_operation_round_trips_through_json_with_its_aliases() {
    let document = fixture("positive.json");
    let registry = registry_from(&document);
    let case = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "R7-a-registered-operation-round-trips-through-json")
        .expect("case");
    let id = case["id_to_round_trip"].as_str().unwrap();
    let operation = registry.resolve(id).expect("registered");
    let json = serde_json::to_string(operation).expect("serialize");
    let restored: Operation = serde_json::from_str(&json).expect("deserialize");
    let expected: Vec<String> = case["expect"]["aliases_after_round_trip"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(restored.aliases, expected);
    assert_eq!(
        restored.feature,
        case["expect"]["feature_after_round_trip"].as_str().unwrap()
    );
    assert_eq!(restored.id, operation.id);
}

#[test]
fn n1_to_n7_a_name_the_registry_cannot_give_one_answer_for_is_refused() {
    let document = fixture("negative.json");
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let mut registry = CapabilityRegistry::default();
        let use_total = case["use_total_api"].as_bool().unwrap_or(false);
        let mut outcome: Option<String> = None;
        for (index, value) in case["register"].as_array().unwrap().iter().enumerate() {
            let operation = operation(value);
            if use_total {
                if let Err(error) = registry.register(operation) {
                    let text = format!("{error}");
                    assert!(text.contains("invalid"), "{id}: {text}");
                    outcome = Some("Invalid".into());
                    break;
                }
            } else if let Err(error) = registry.try_register(operation) {
                outcome = Some(match error {
                    RegistryError::UnknownName { .. } => "UnknownName".into(),
                    RegistryError::DuplicateName { .. } => "DuplicateName".into(),
                    RegistryError::EmptyName { .. } => "EmptyName".into(),
                    RegistryError::DuplicateId { .. } => "DuplicateId".into(),
                    RegistryError::InvalidOperation { .. } => "InvalidOperation".into(),
                    RegistryError::UnknownOption { .. } => "UnknownOption".into(),
                });
                if let Some(message) = case["expect"]["message"].as_str() {
                    assert_eq!(format!("{error}"), message, "{id}");
                }
                for needle in case["expect"]["message_contains"]
                    .as_array()
                    .map(|list| list.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>())
                    .unwrap_or_default()
                {
                    assert!(
                        format!("{error}").contains(needle),
                        "{id}: {error} lacks {needle:?}"
                    );
                }
                if case["expect"]["first_registration_survives"] == true {
                    assert!(
                        registry
                            .resolve(case["register"][0]["id"].as_str().unwrap())
                            .is_ok(),
                        "{id}: the first registration must survive"
                    );
                }
                break;
            }
            let _ = index;
        }
        if let Some(name) = case["resolve"].as_str()
            && let Err(error) = registry.resolve(name)
        {
            outcome = Some(match error {
                RegistryError::UnknownName { .. } => "UnknownName".into(),
                other => panic!("{id}: unexpected {other}"),
            });
            assert_eq!(
                format!("{error}"),
                case["expect"]["message"].as_str().unwrap(),
                "{id}"
            );
        }
        assert_eq!(
            outcome.as_deref(),
            Some(case["expect"]["error"].as_str().unwrap()),
            "{id}"
        );
    }
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-02-001/api.md");
    assert!(api.exists(), "{}", api.display());
}
