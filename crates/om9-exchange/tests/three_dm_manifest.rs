// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ported from `rust/tests/core_3dm_manifest.rs` and `rust/tests/core_3dm_schema2.rs`.

use om9_exchange::three_dm::validate_export_manifest;
use serde_json::{Value, json};

const MESH: &str = r#"{"items":[{"name":"mesh","layer":"Default","visible":true,"locked":false,"color":[128,0,255],"vertices":[[0,0,0],[1,0,0],[0,1,0]],"faces":[[0,1,2,2]]}],"tolerance":0.000001}"#;

#[test]
fn export_manifest_preserves_exact_mesh_and_brep_schema() {
    assert!(validate_export_manifest(MESH).is_ok());
    assert!(
        validate_export_manifest(
            r#"{"items":[{"brep":"C:/staged/shape.brep","color":[0,1,255]}],"tolerance":0.001}"#
        )
        .is_ok()
    );
}

#[test]
fn malformed_mesh_tuples_cannot_be_coerced_to_zero_or_truncated() {
    for (from, to) in [
        ("[0,0,0]", "[0,0]"),
        ("[0,0,0]", "[0,0,0,7]"),
        ("[0,0,0]", "[0,\"1\",0]"),
        ("[0,1,2,2]", "[0,1,2]"),
        ("[0,1,2,2]", "[0,1,3,3]"),
        ("[0,1,2,2]", "[0,1,2.1,2]"),
        ("[0,1,2,2]", "[0,-1,2,2]"),
        ("[128,0,255]", "[128,0]"),
        ("[128,0,255]", "[128,0,256]"),
        ("true", "null"),
    ] {
        assert!(
            validate_export_manifest(&MESH.replace(from, to)).is_err(),
            "{to}"
        );
    }
}

#[test]
fn invalid_schema_or_retained_data_is_rejected_before_native_conversion() {
    for raw in [
        "{}",
        r#"{"items":[],"tolerance":1}"#,
        r#"{"items":[{"brep":"","color":[0,0,0]}],"tolerance":1}"#,
        r#"{"items":[{"brep":"x\u0000y","color":[0,0,0]}],"tolerance":1}"#,
    ] {
        assert!(validate_export_manifest(raw).is_err(), "{raw}");
    }
    for (from, to) in [
        ("0.000001", "0"),
        ("0.000001", "\"0.1\""),
        ("\"name\":\"mesh\"", "\"retained\":true"),
        ("\"name\":\"mesh\"", "\"brep\":\"shape.brep\""),
        (
            "\"name\":\"mesh\"",
            "\"name\":\"mesh\",\"name\":\"duplicate\"",
        ),
    ] {
        assert!(
            validate_export_manifest(&MESH.replace(from, to)).is_err(),
            "{to}"
        );
    }
}

#[test]
fn current_point_cloud_export_fields_survive_p0_preflight() {
    let cloud = r#"{"items":[{"point_cloud_fields":"C:/staged/cloud.json","point_cloud_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","point_cloud_transform":[1,0,0,10,0,1,0,20,0,0,1,30,0,0,0,1]}],"tolerance":0.001}"#;
    assert!(validate_export_manifest(cloud).is_ok());
    for (from, to) in [
        ("C:/staged/cloud.json", ""),
        (
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "short",
        ),
        ("[1,0,0,10,0,1,0,20,0,0,1,30,0,0,0,1]", "[1,0,0]"),
        (
            "\"point_cloud_fields\":",
            "\"brep\":\"shape.brep\",\"point_cloud_fields\":",
        ),
    ] {
        assert!(
            validate_export_manifest(&cloud.replace(from, to)).is_err(),
            "{to}"
        );
    }
}

#[test]
fn legacy_optional_color_keeps_the_native_default_and_checks_explicit_values() {
    assert!(validate_export_manifest(r#"{"items":[{"brep":"shape.brep"}]}"#).is_ok());
    assert!(
        validate_export_manifest(r#"{"items":[{"brep":"shape.brep"}],"tolerance":0.001}"#).is_ok()
    );
    assert!(
        validate_export_manifest(
            r#"{"items":[{"brep":"shape.brep","color":null}],"tolerance":0.001}"#
        )
        .is_err()
    );
}

fn canonical() -> Value {
    json!({"schema_version":2,"tolerance":0.001,
        "layer_session":{"version":1,"snapshot":{
            "document_id":"source","generation":"7","active_layer":"metal",
            "layers":[{"source_id":"metal","parent_id":null,"name":"Metal",
                "path_components":["Metal"],"rgb":[1,2,3],"locked":false,"visible":true,
                "persistent_locked":null,"persistent_visible":null}],
            "objects":[{"id":"ring","layer_id":"metal","locked":false,"visible":true,
                "color_source":"ByLayer","rgb":[1,2,3]}]}},
        "items":[{"layer_object_id":"ring","brep":"ring.brep"}]})
}

#[test]
fn schema2_carries_exact_layer_bindings_without_losing_geometry_preflight() {
    let packet = canonical();
    assert!(validate_export_manifest(&packet.to_string()).is_ok());
    let mut mesh = packet.clone();
    mesh["items"][0] =
        json!({"layer_object_id":"ring","vertices":[[0,0,0],[1,0,0],[0,1,0]],"faces":[[0,1,2,2]]});
    assert!(validate_export_manifest(&mesh.to_string()).is_ok());
    mesh["items"][0]["faces"][0] = json!([0, 1, 99, 99]);
    assert!(validate_export_manifest(&mesh.to_string()).is_err());
}

#[test]
fn schema2_rejects_missing_duplicate_foreign_and_extra_layer_witnesses() {
    let packet = canonical();
    for id in [Value::Null, json!("foreign"), json!(7)] {
        let mut changed = packet.clone();
        changed["items"][0]["layer_object_id"] = id;
        assert!(validate_export_manifest(&changed.to_string()).is_err());
    }
    let mut duplicate = packet.clone();
    duplicate["items"] = json!([packet["items"][0], packet["items"][0]]);
    assert!(validate_export_manifest(&duplicate.to_string()).is_err());
    let mut missing = packet.clone();
    missing["items"] = json!([]);
    assert!(validate_export_manifest(&missing.to_string()).is_err());
    let mut extra = packet.clone();
    extra["layer_session"]["snapshot"]["objects"] = json!([]);
    assert!(validate_export_manifest(&extra.to_string()).is_err());
}

#[test]
fn palette_only_schema2_requires_valid_empty_geometry_and_object_inventory() {
    let mut palette = canonical();
    palette["items"] = json!([]);
    palette["layer_session"]["snapshot"]["objects"] = json!([]);
    assert!(validate_export_manifest(&palette.to_string()).is_ok());
    palette["layer_session"]["version"] = json!(2);
    assert!(validate_export_manifest(&palette.to_string()).is_err());
}
