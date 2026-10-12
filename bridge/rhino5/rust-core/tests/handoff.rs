use om9_rhino_layer::{prepare, receive};
use openmatrix9_rust::{layer_codec::encode_snapshot, layer_state::*};
use serde_json::Value;

fn source() -> LayerSnapshotV1 {
    let mut s = LayerSnapshotV1::empty("Matrix");
    for i in 0..32 {
        s.layers.push(LayerRow {
            source_id: format!("layer-{i}"),
            parent_id: None,
            name: format!("Slot {i}"),
            path_components: vec![format!("Slot {i}")],
            rgb: [i, 125, 251],
            locked: i == 1,
            visible: i != 1,
            persistent_locked: None,
            persistent_visible: None,
        });
    }
    s.layers.push(LayerRow {
        source_id: "child".into(),
        parent_id: Some("layer-1".into()),
        name: "Own state".into(),
        path_components: vec!["Slot 1".into(), "Own state".into()],
        rgb: [31, 71, 131],
        locked: false,
        visible: true,
        persistent_locked: Some(false),
        persistent_visible: Some(true),
    });
    s.objects.push(ObjectLayerRow {
        id: "ring".into(),
        layer_id: "child".into(),
        locked: true,
        visible: false,
        color_source: ColorSource::ByObject,
        rgb: [211, 19, 81],
    });
    s.active_layer = Some("layer-0".into());
    s
}
fn geometry() -> Vec<u8> {
    let mut g = b"3D Geometry File Format        5\n".to_vec();
    g.extend_from_slice(b"native-fixture");
    g
}
fn bindings() -> Vec<u8> {
    br#"[{"physical_id":"archive-ring","source_id":"ring"}]"#.to_vec()
}
fn destination() -> LayerSnapshotV1 {
    let mut d = LayerSnapshotV1::empty("OM9");
    d.generation = 9007199254740993;
    let mut l = source().layers[0].clone();
    l.source_id = "existing".into();
    l.rgb = [1, 2, 3];
    d.layers.push(l);
    d.active_layer = Some("existing".into());
    d
}
#[test]
fn selected_carries_all_empty_palette_and_source_colors() {
    let g = geometry();
    let metadata = prepare(&encode_snapshot(&source()).unwrap(), 1, &g).unwrap();
    let p: Value = serde_json::from_slice(
        &receive(
            &metadata,
            &g,
            &encode_snapshot(&destination()).unwrap(),
            &bindings(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(p["after"]["layers"].as_array().unwrap().len(), 33);
    assert_eq!(
        p["after"]["layers"][0]["rgb"],
        serde_json::json!([0, 125, 251])
    );
    assert_eq!(p["layer_mapping"]["layer-0"], "existing");
    assert_eq!(p["after"]["generation"], "9007199254740994");
    assert_eq!(
        p["before"]["layers"][0]["rgb"],
        serde_json::json!([1, 2, 3])
    );
}
#[test]
fn session_preserves_child_desired_and_object_own_states() {
    let g = geometry();
    let metadata = prepare(&encode_snapshot(&source()).unwrap(), 2, &g).unwrap();
    let p: Value = serde_json::from_slice(
        &receive(
            &metadata,
            &g,
            &encode_snapshot(&destination()).unwrap(),
            &bindings(),
        )
        .unwrap(),
    )
    .unwrap();
    let child = p["after"]["layers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["source_id"] == "child")
        .unwrap();
    assert_eq!(child["locked"], false);
    assert_eq!(child["persistent_visible"], true);
    assert_eq!(p["after"]["objects"][0]["locked"], true);
    assert_eq!(p["after"]["objects"][0]["visible"], false);
    assert_eq!(p["after"]["objects"][0]["color_source"], "ByObject");
    assert!(p["object_mapping"].get("archive-ring").is_some());
}
#[test]
fn mismatched_geometry_is_rejected_before_native_commit() {
    let mut g = geometry();
    let metadata = prepare(&encode_snapshot(&source()).unwrap(), 2, &g).unwrap();
    g.push(0);
    assert_eq!(
        receive(
            &metadata,
            &g,
            &encode_snapshot(&destination()).unwrap(),
            &bindings()
        ),
        Err(LayerError::ClipboardBinding)
    );
}
#[test]
fn missing_or_duplicate_archive_provenance_rejects() {
    let g = geometry();
    let metadata = prepare(&encode_snapshot(&source()).unwrap(), 2, &g).unwrap();
    assert_eq!(
        receive(
            &metadata,
            &g,
            &encode_snapshot(&destination()).unwrap(),
            b"[]"
        ),
        Err(LayerError::ClipboardBinding)
    );
}
#[test]
fn unknown_scope_or_native_only_does_not_claim_full_handoff() {
    assert_eq!(
        prepare(&encode_snapshot(&source()).unwrap(), 3, &geometry()),
        Err(LayerError::UnsupportedVersion)
    );
    assert_eq!(
        receive(
            b"",
            &geometry(),
            &encode_snapshot(&destination()).unwrap(),
            &bindings()
        ),
        Err(LayerError::ClipboardBinding)
    );
}
