#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_legacy::normalize_native_layers, layer_state::*};

#[test]
fn flat_native_names_gain_parents_without_changing_object_identity_or_own_flags() {
    let mut state = LayerSnapshotV1::empty("legacy");
    let mut flat = layer("native-id", None, &["Working", "Geometry"]);
    flat.name = "Working::Geometry".into();
    flat.locked = true;
    flat.visible = false;
    flat.rgb = [17, 18, 19];
    state.layers.push(flat);
    state.active_layer = Some("native-id".into());
    state.objects.push(object("curve", "native-id"));
    let result = normalize_native_layers(state).unwrap();
    assert_eq!(result.layers.len(), 2);
    let leaf = result
        .layers
        .iter()
        .find(|l| l.source_id == "native-id")
        .unwrap();
    assert_eq!(leaf.name, "Geometry");
    assert_eq!(leaf.rgb, [17, 18, 19]);
    assert!(leaf.locked && !leaf.visible && leaf.parent_id.is_some());
    assert_eq!(result.active_layer.as_deref(), Some("native-id"));
    assert!(!result.objects[0].locked);
    assert!(effective_state(&result, "curve").unwrap().locked);
    validate_layers(&result).unwrap();
}
#[test]
fn genuine_prefix_and_children_keep_their_native_ids_and_states() {
    let mut state = LayerSnapshotV1::empty("legacy");
    let mut prefix = layer("prefix", None, &["Working"]);
    prefix.rgb = [91, 92, 93];
    let mut flat = layer("native-id", None, &["Working", "Geometry"]);
    flat.name = "Working::Geometry".into();
    let child = layer(
        "child",
        Some("native-id"),
        &["Working", "Geometry", "Child"],
    );
    state.layers = vec![child, flat, prefix.clone()];
    let result = normalize_native_layers(state).unwrap();
    assert_eq!(result.layers.len(), 3);
    assert_eq!(
        result
            .layers
            .iter()
            .find(|l| l.source_id == "prefix")
            .unwrap(),
        &prefix
    );
    assert_eq!(
        result
            .layers
            .iter()
            .find(|l| l.source_id == "native-id")
            .unwrap()
            .parent_id
            .as_deref(),
        Some("prefix")
    );
    assert_eq!(
        result
            .layers
            .iter()
            .find(|l| l.source_id == "child")
            .unwrap()
            .parent_id
            .as_deref(),
        Some("native-id")
    );
}
#[test]
fn flat_aliases_and_malformed_native_ancestry_are_rejected() {
    let mut state = LayerSnapshotV1::empty("legacy");
    let root = layer("root", None, &["A"]);
    let child = layer("child", Some("root"), &["A", "B"]);
    let mut flat = layer("flat", None, &["A", "B"]);
    flat.name = "A::B".into();
    state.layers = vec![root, child, flat];
    assert_eq!(
        normalize_native_layers(state.clone()),
        Err(LayerError::AmbiguousPath)
    );
    state.layers.pop();
    state.layers[1].parent_id = Some("absent".into());
    assert_eq!(
        normalize_native_layers(state),
        Err(LayerError::MissingLayer)
    );
}
#[test]
fn explicit_normalization_never_repairs_unrelated_invalid_paths_or_cycles() {
    let mut state = LayerSnapshotV1::empty("legacy");
    let mut flat = layer("flat", None, &["Wrong", "B"]);
    flat.name = "A::B".into();
    state.layers.push(flat);
    assert_eq!(normalize_native_layers(state), Err(LayerError::InvalidPath));
    let mut cycle = LayerSnapshotV1::empty("legacy");
    cycle.layers = vec![layer("a", Some("b"), &["A"]), layer("b", Some("a"), &["B"])];
    assert_eq!(normalize_native_layers(cycle), Err(LayerError::Cycle));
}
#[test]
fn valid_canonical_snapshot_is_exactly_unchanged() {
    let state = snapshot();
    assert_eq!(normalize_native_layers(state.clone()).unwrap(), state);
}
