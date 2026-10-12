use openmatrix9_rust::core_3dm_archive::validate_export_manifest;
use serde_json::{Value, json};

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
    mesh["items"][0] = json!({"layer_object_id":"ring","vertices":[[0,0,0],[1,0,0],[0,1,0]],"faces":[[0,1,2,2]]});
    assert!(validate_export_manifest(&mesh.to_string()).is_ok());
    mesh["items"][0]["faces"][0] = json!([0,1,99,99]);
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
