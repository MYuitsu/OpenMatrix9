// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Writer requests bind native IDs/frames without cloning the current model.
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::{Geometry, Transform},
    model::{Definition, Document, Entity, Layer},
    retained::HostRef,
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{ExportCapture, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::io::Read;
fn exchange() -> NativeExchange {
    let path = std::env::current_exe().unwrap();
    let mut file = std::fs::File::open(&path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    NativeExchange::new(
        NativeProgram::admit(&path, hash.finalize().into()).unwrap(),
        ProcessLimits::default(),
    )
    .unwrap()
}
#[test]
fn request_uses_native_references_for_layers_instances_and_definition_members() {
    let mut d = Document::new("writer request");
    let mut parent = Layer::new("parent", [1, 2, 3, 255]);
    let mut child = Layer::new("child", [4, 5, 6, 255]);
    parent.id.0 = uuid::Uuid::from_u128(200);
    child.id.0 = uuid::Uuid::from_u128(100);
    child.parent = Some(parent.id);
    let (pid, cid) = (parent.id, child.id);
    d.layer_order = vec![pid, cid];
    d.layers.insert(pid, parent);
    d.layers.insert(cid, child);
    let point = Entity::new("local", Geometry::Point([2., 3., 4.]), Some(cid));
    let mid = point.id;
    let def = Definition::new("shared", vec![point]);
    let did = def.id;
    d.definitions.insert(did, def);
    let mut root = Entity::new("instance", Geometry::Instance(did), Some(pid));
    root.placement = Transform::translation([5., 6., 7.]);
    let oid = root.id;
    d.entities.insert(oid, root);
    d.current_layer = Some(cid);
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let before = c.snapshot();
    let capture = ExportCapture::all(&c, id).unwrap();
    let original = Cancellation::default();
    let prepared = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &original,
            Usage::default(),
        )
        .unwrap();
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let bytes = wire
        .encode_retained_request(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let p: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(p["schema_version"], 4);
    assert_eq!(p["writer_policy"], "om9-retained-current-v1");
    assert_eq!(p["metadata_policy"], "new_mm");
    assert_eq!(p["bounds_policy"], "cold_leaf_hull_v1");
    assert!(p["sources"].as_array().unwrap().is_empty());
    let member = wire
        .host(HostRef::DefinitionMember {
            definition: did,
            object: mid,
        })
        .unwrap();
    let definition = wire.host(HostRef::Definition(did)).unwrap();
    let root = wire.host(HostRef::Object(oid)).unwrap();
    let child = wire.host(HostRef::Layer(cid)).unwrap();
    let parent = wire.host(HostRef::Layer(pid)).unwrap();
    let rows = p["additions"].as_array().unwrap();
    let row = |id: uuid::Uuid| {
        rows.iter()
            .find(|r| r["output"]["id"] == id.to_string())
            .unwrap()
    };
    assert_eq!(
        row(child.id)["current"]["Layer"]["parent"],
        parent.id.to_string()
    );
    assert_eq!(
        row(root.id)["current"]["Entity"]["geometry"]["Instance"],
        definition.id.to_string()
    );
    assert_eq!(
        row(root.id)["current"]["Entity"]["attributes"]["layer_index"],
        0
    );
    assert_eq!(row(root.id)["current"]["Entity"]["placement"][0][3], 5.);
    assert_eq!(
        row(member.id)["current"]["Entity"]["attributes"]["layer_index"],
        1
    );
    assert_eq!(
        row(member.id)["current"]["Entity"]["definition_member"],
        true
    );
    assert_eq!(
        row(definition.id)["current"]["Definition"]["members"],
        serde_json::json!([member.id])
    );
    assert_eq!(p["document"]["current_layer"]["policy"], "explicit_output");
    assert_eq!(p["document"]["current_layer"]["id"], child.id.to_string());
    assert_eq!(p["document"]["current_layer"]["archive_reference_index"], 1);
    assert_eq!(c.snapshot(), before);
    original.cancel();
    assert!(matches!(
        wire.encode_retained_request(&c, &Cancellation::default(), Usage::default()),
        Err(Error::Canceled)
    ));
}
#[test]
fn request_refuses_external_budget_and_stale_owner_before_serialized_growth() {
    let d = Document::new("empty");
    let id = d.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let prepared = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &Cancellation::default(),
            Usage::default(),
        )
        .unwrap();
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let bytes = wire
        .encode_retained_request(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let p: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        p["default_layer"]["output"]["id"],
        p["document"]["current_layer"]["id"]
    );
    assert_eq!(
        p["default_layer"]["current"]["Layer"]["name"],
        "OM9 Unassigned"
    );
    assert!(matches!(
        wire.encode_retained_request(
            &c,
            &Cancellation::default(),
            Usage {
                bytes: c.limits().max_bytes,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
    c.close_document(id).unwrap();
    assert!(
        wire.encode_retained_request(&c, &Cancellation::default(), Usage::default())
            .is_err()
    );
}
