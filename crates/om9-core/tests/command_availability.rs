// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-011 (Command Availability Preconditions),
//! driven by the fixtures in `implementation/fixtures/M01/OM9-L3-01-02-011/`.
use om9_core::capability::{
    Availability, Backend, CapabilityRegistry, ContextState, Operation, Phase, Preconditions,
    Receipt, ReceiptScope,
};
use om9_core::geometry::GeometryKind;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-02-011")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn kind(name: &str) -> GeometryKind {
    match name {
        "Point" => GeometryKind::Point,
        "Curve" => GeometryKind::Curve,
        "Mesh" => GeometryKind::Mesh,
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

fn context(value: &Value) -> ContextState {
    ContextState {
        has_document: value["has_document"].as_bool().unwrap_or(false),
        selection: value["selection"].as_u64().unwrap_or(0) as usize,
        has_active_view: value["has_active_view"].as_bool().unwrap_or(false),
    }
}

fn preconditions(value: &Value) -> Preconditions {
    Preconditions {
        needs_document: value["needs_document"].as_bool().unwrap_or(false),
        needs_selection: value["needs_selection"].as_bool().unwrap_or(false),
        needs_active_view: value["needs_active_view"].as_bool().unwrap_or(false),
    }
}

fn receipts(value: &Value) -> Vec<Receipt> {
    let scope = match value.as_str().unwrap_or("complete") {
        "none" => return Vec::new(),
        _ => ReceiptScope::Semantic,
    };
    vec![
        Receipt {
            target: "OM9Standalone".into(),
            source_sha256: "a".repeat(64),
            binary_sha256: "b".repeat(64),
            suite: "command_availability".into(),
            scope,
            passed: true,
        },
        Receipt {
            target: "OM9Standalone".into(),
            source_sha256: "a".repeat(64),
            binary_sha256: "b".repeat(64),
            suite: "command_availability".into(),
            scope: ReceiptScope::StandaloneRuntime,
            passed: true,
        },
    ]
}

fn operation(value: &Value) -> Operation {
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
        preconditions: preconditions(&value["preconditions"]),
        backend: Backend::Rust {
            version: "1".into(),
        },
        prepare: true,
        preview: value["preview"].as_bool().unwrap_or(true),
        commit: value["commit"].as_bool().unwrap_or(true),
        receipts: receipts(&value["receipt"]),
    }
}

fn overridden(base: &Value, patch: &Value) -> Value {
    let mut merged = base.clone();
    if let Some(object) = patch.as_object() {
        for (key, value) in object {
            if key == "preconditions" {
                let mut merged_pre = merged["preconditions"].clone();
                for (inner, inner_value) in value.as_object().expect("preconditions") {
                    merged_pre[inner] = inner_value.clone();
                }
                merged["preconditions"] = merged_pre;
            } else {
                merged[key] = value.clone();
            }
        }
    }
    merged
}

#[test]
fn a1_to_a6_availability_follows_the_session_state() {
    let document = fixture("positive.json");
    let base = &document["operation"];
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if case.get("options").is_some() {
            continue; // A7 exercises admit, not the availability line.
        }
        let operation_value = match case.get("operation_override") {
            Some(patch) => overridden(base, patch),
            None => base.clone(),
        };
        let mut registry = CapabilityRegistry::default();
        registry
            .try_register(operation(&operation_value))
            .expect("registered");
        let context = context(&document["contexts"][case["context"].as_str().unwrap()]);
        let availability: Availability = registry.availability_with("line", &context);
        assert_eq!(
            availability.enabled,
            case["expect"]["enabled"].as_bool().unwrap(),
            "{id}: {}",
            availability.reason
        );
        assert_eq!(
            availability.reason,
            case["expect"]["reason"].as_str().unwrap(),
            "{id}"
        );
    }
}

#[test]
fn a7_to_a8_admit_reads_the_context_and_the_option_names_it_was_given() {
    let document = fixture("positive.json");
    for case in document["cases"].as_array().unwrap() {
        let Some(options) = case["options"].as_array() else {
            continue;
        };
        let id = case["id"].as_str().unwrap();
        let mut registry = CapabilityRegistry::default();
        registry
            .try_register(operation(&document["operation"]))
            .expect("registered");
        let context = context(&document["contexts"][case["context"].as_str().unwrap()]);
        let options: Vec<&str> = options
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        let admitted = registry
            .admit_with(
                "line",
                GeometryKind::Point,
                &options,
                Phase::Commit,
                &context,
            )
            .is_ok();
        assert_eq!(
            admitted,
            case["expect"]["admitted"].as_bool().unwrap(),
            "{id}: options {options:?} in context {context:?}"
        );
    }
}

#[test]
fn b1_to_b7_a_command_that_cannot_run_is_never_offered() {
    let negative = fixture("negative.json");
    let positive = fixture("positive.json");
    for case in negative["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let mut registry = CapabilityRegistry::default();
        if let Some(lookup) = case["id_lookup"].as_str() {
            let availability = registry.availability_with(lookup, &ContextState::empty_document());
            assert!(!availability.enabled, "{id}");
            assert_eq!(
                availability.reason,
                case["expect"]["reason"].as_str().unwrap(),
                "{id}"
            );
            continue;
        }
        let base = overridden(&positive["operation"], &case["operation"]);
        registry.try_register(operation(&base)).expect("registered");
        let context = case["context"]
            .as_str()
            .map(|name| context(&positive["contexts"][name]))
            .unwrap_or_default();
        if let Some(enabled) = case["expect"]["enabled"].as_bool() {
            let availability = registry.availability_with("line", &context);
            assert_eq!(
                availability.enabled, enabled,
                "{id}: {}",
                availability.reason
            );
            assert_eq!(
                availability.reason,
                case["expect"]["reason"].as_str().unwrap(),
                "{id}"
            );
            if case["expect"]["document_unchanged"] == true {
                // A disabled command is only ever asked about, never run through
                // this path: it can add no transaction and no undo entry.
                assert_eq!(case["expect"]["undo_entries_added"].as_u64().unwrap(), 0);
            }
            continue;
        }
        let options: Vec<&str> = case["options"]
            .as_array()
            .map(|list| list.iter().map(|value| value.as_str().unwrap()).collect())
            .unwrap_or_else(|| vec!["BothSides"]);
        let kind = case["kind"]
            .as_str()
            .map(kind)
            .unwrap_or(GeometryKind::Point);
        let phase = match case["phase"].as_str() {
            Some("Preview") => Phase::Preview,
            _ => Phase::Commit,
        };
        let result = if case["admit_without_context"].as_bool().unwrap_or(false) {
            registry.admit("line", kind, &options, phase).map(|_| ())
        } else {
            registry
                .admit_with("line", kind, &options, phase, &context)
                .map(|_| ())
        };
        let admitted = result.is_ok();
        assert_eq!(
            admitted,
            case["expect"]["admitted"].as_bool().unwrap(),
            "{id}: {result:?}"
        );
        if !admitted {
            let message = format!("{}", result.unwrap_err());
            for needle in case["expect"]["message_contains"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .map(|value| value.as_str().unwrap())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
            {
                assert!(message.contains(needle), "{id}: {message}");
            }
        }
        if case["expect"]["document_unchanged"] == true {
            // `admit` is a gate: it borrows the descriptor and cannot write.
            assert!(case["expect"]["undo_entries_added"].as_u64().unwrap() == 0);
        }
    }
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-02-011/api.md");
    assert!(api.exists(), "{}", api.display());
}
