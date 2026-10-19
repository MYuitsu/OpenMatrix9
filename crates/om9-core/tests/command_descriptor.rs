// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-002 (Typed Command Descriptor), driven by the
//! fixtures in `implementation/fixtures/M01/OM9-L3-01-02-002/`.
use om9_core::capability::{Backend, CapabilityRegistry, Operation, RegistryError};
use om9_core::geometry::GeometryKind;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-02-002")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn kind(name: &str) -> GeometryKind {
    match name {
        "Curve" => GeometryKind::Curve,
        "Point" => GeometryKind::Point,
        "Surface" => GeometryKind::Surface,
        other => panic!("unknown geometry kind {other}"),
    }
}

fn kinds(value: &Value) -> Vec<GeometryKind> {
    value
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| kind(item.as_str().unwrap()))
                .collect()
        })
        .unwrap_or_default()
}

fn operation(value: &Value) -> Operation {
    let precondition = |key: &str| value["preconditions"][key].as_bool().unwrap_or(false);
    Operation {
        id: value["id"].as_str().expect("id").to_owned(),
        feature: value["feature"].as_str().expect("feature").to_owned(),
        aliases: Vec::new(),
        prompt: value["prompt"].as_str().unwrap_or_default().to_owned(),
        input_types: kinds(&value["input_types"]),
        requires_input: value["requires_input"].as_bool().unwrap_or(false),
        output: kinds(&value["output"]),
        options: value["options"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|option| option.as_str().unwrap().to_owned())
                    .collect()
            })
            .unwrap_or_default(),
        preconditions: om9_core::capability::Preconditions {
            needs_document: precondition("needs_document"),
            needs_selection: precondition("needs_selection"),
            needs_active_view: precondition("needs_active_view"),
        },
        backend: Backend::Rust {
            version: "1".into(),
        },
        prepare: value["prepare"].as_bool().unwrap_or(true),
        preview: value["preview"].as_bool().unwrap_or(true),
        commit: value["commit"].as_bool().unwrap_or(true),
        receipts: Vec::new(),
    }
}

#[test]
fn d1_the_descriptor_declares_its_signature() {
    let document = fixture("positive.json");
    let case = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"].as_str().unwrap().starts_with("D1"))
        .unwrap();
    let operation = operation(&document["operation"]);
    assert_eq!(
        operation.input_types,
        kinds(&case["expect"]["input_types"]),
        "the input signature is the declared one"
    );
    assert_eq!(operation.output, kinds(&case["expect"]["output"]));
    assert_eq!(operation.prompt, case["expect"]["prompt"].as_str().unwrap());
    assert_eq!(
        operation.options,
        case["expect"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    assert!(operation.requires_input);
}

#[test]
fn d2_a_declared_option_resolves_ignoring_case() {
    let document = fixture("positive.json");
    let case = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"].as_str().unwrap().starts_with("D2"))
        .unwrap();
    let operation = operation(&document["operation"]);
    let expected: Vec<&str> = case["expect"]["declared"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    for (index, lookup) in case["lookup"].as_array().unwrap().iter().enumerate() {
        let declared = operation
            .option(lookup.as_str().unwrap())
            .unwrap_or_else(|error| panic!("{lookup}: {error}"));
        assert_eq!(declared, expected[index], "{lookup}");
    }
}

#[test]
fn d3_to_d4_the_descriptor_is_serializable_and_deterministic() {
    let document = fixture("positive.json");
    let operation = operation(&document["operation"]);
    let first = serde_json::to_string(&operation).expect("serialize");
    let restored: Operation = serde_json::from_str(&first).expect("deserialize");
    assert_eq!(restored, operation, "round trip");
    assert_eq!(
        document["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"].as_str().unwrap().starts_with("D3"))
            .unwrap()["expect"]["equal_after_round_trip"],
        true
    );
    for _ in 0..10 {
        assert_eq!(serde_json::to_string(&operation).unwrap(), first);
    }
}

#[test]
fn d5_an_option_less_command_is_still_a_descriptor() {
    let document = fixture("positive.json");
    let case = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"].as_str().unwrap().starts_with("D5"))
        .unwrap();
    let mut registry = CapabilityRegistry::default();
    registry
        .try_register(operation(&document["operation"]))
        .expect("the main descriptor");
    registry
        .try_register(operation(&case["second"]))
        .expect("a command with no input, no output and a prompt is a descriptor");
    let fit = registry.resolve("fit").expect("registered");
    assert!(fit.options.is_empty());
    assert_eq!(case["expect"]["options_empty"], true);
    assert!(!fit.requires_input);
}

#[test]
fn e1_to_e4_a_descriptor_that_cannot_be_honoured_is_refused() {
    let document = fixture("negative.json");
    let positive = fixture("positive.json");
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if let Some(value) = case.get("operation").filter(|value| value.is_object()) {
            let mut registry = CapabilityRegistry::default();
            let error = registry
                .try_register(operation(value))
                .expect_err(&format!("{id}: must be refused"));
            let expected = case["expect"]["error"].as_str().unwrap();
            let actual = match &error {
                RegistryError::InvalidOperation { .. } => "InvalidOperation",
                RegistryError::UnknownOption { .. } => "UnknownOption",
                other => panic!("{id}: unexpected {other}"),
            };
            assert_eq!(actual, expected, "{id}");
            if let Some(message) = case["expect"]["message"].as_str() {
                assert_eq!(format!("{error}"), message, "{id}");
            }
            for needle in case["expect"]["message_contains"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .map(|value| value.as_str().unwrap())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
            {
                assert!(format!("{error}").contains(needle), "{id}: {error}");
            }
            continue;
        }
        // An option lookup: the descriptor registers, the option does not resolve.
        let mut registry = CapabilityRegistry::default();
        registry
            .try_register(operation(&positive["operation"]))
            .expect("the main descriptor");
        let line = registry.resolve("line").expect("registered");
        let error = line
            .option(case["lookup"].as_str().unwrap())
            .expect_err(&format!("{id}: must be refused"));
        assert!(matches!(error, RegistryError::UnknownOption { .. }), "{id}");
        assert_eq!(
            format!("{error}"),
            case["expect"]["message"].as_str().unwrap(),
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
        .join("../../implementation/design/M01/OM9-L3-01-02-002/api.md");
    assert!(api.exists(), "{}", api.display());
}
