#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{
    layer_clipboard::GeometryObjectBinding, layer_retained::plan_retained_overlay, layer_state::*,
};
#[test]
fn retained_subset_keeps_full_palette_and_original_own_ids_and_context() {
    let mut source = snapshot();
    source.generation = u64::MAX;
    let before = source.clone();
    let subset =
        openmatrix9_rust::layer_retained::subset_retained_snapshot(&source, &["stone".into()])
            .unwrap();
    assert_eq!(subset.layers, source.layers);
    assert_eq!(subset.active_layer, source.active_layer);
    assert_eq!(subset.document_id, source.document_id);
    assert_eq!(subset.generation, u64::MAX);
    assert_eq!(subset.objects, vec![source.objects[1].clone()]);
    assert_eq!(source, before);
    let empty = openmatrix9_rust::layer_retained::subset_retained_snapshot(&source, &[]).unwrap();
    assert!(empty.objects.is_empty());
    assert_eq!(empty.layers, source.layers);
}
#[test]
fn retained_subset_rejects_unknown_duplicate_and_invalid_source() {
    use openmatrix9_rust::layer_retained::subset_retained_snapshot;
    let source = snapshot();
    assert_eq!(
        subset_retained_snapshot(&source, &["missing".into()]),
        Err(LayerError::MissingObject)
    );
    assert!(subset_retained_snapshot(&source, &["stone".into(), "stone".into()]).is_err());
    let mut invalid = source;
    invalid.layers[0].parent_id = Some("missing".into());
    assert!(subset_retained_snapshot(&invalid, &[]).is_err());
}
fn bindings() -> Vec<GeometryObjectBinding<'static>> {
    vec![
        GeometryObjectBinding {
            physical_id: "native-ring",
            source_id: "ring",
        },
        GeometryObjectBinding {
            physical_id: "native-stone",
            source_id: "stone",
        },
        GeometryObjectBinding {
            physical_id: "native-detail",
            source_id: "detail-curve",
        },
    ]
}
#[test]
fn retained_overlay_merges_full_palette_and_own_state_without_appending_objects() {
    let mut source = snapshot();
    source.generation = u64::MAX;
    source.layers[0].locked = true;
    source.layers[0].rgb = [201, 202, 203];
    source.layers.push(layer("empty", None, &["Empty custom"]));
    source.objects[1].color_source = ColorSource::ByObject;
    source.objects[1].locked = true;
    source.objects[1].visible = false;
    let mut destination = snapshot();
    destination.document_id = "archive".into();
    destination.objects.clear();
    destination.layers[0].source_id = "native-metal".into();
    destination.layers[2].parent_id = Some("native-metal".into());
    destination.active_layer = Some("gem".into());
    destination
        .layers
        .push(layer("native-only", None, &["Untouched native layer"]));
    let before = destination.clone();
    let source_before = source.clone();
    let result = plan_retained_overlay(&source, &destination, &bindings()).unwrap();
    assert_eq!(destination, before);
    assert_eq!(source, source_before);
    assert_eq!(result.generation, u64::MAX);
    assert_eq!(result.document_id, source.document_id);
    assert_eq!(result.layers.len(), 5);
    assert_eq!(result.objects.len(), 3);
    assert_eq!(result.objects[0].id, "native-ring");
    assert_eq!(result.objects[0].layer_id, "native-metal");
    assert!(!result.objects[0].locked);
    assert!(effective_state(&result, "native-ring").unwrap().locked);
    assert_eq!(result.objects[1].color_source, ColorSource::ByObject);
    assert!(result.objects[1].locked);
    assert!(!result.objects[1].visible);
    assert_eq!(result.active_layer.as_deref(), Some("gem"));
}
#[test]
fn malformed_or_unmatched_retained_binding_rejects_without_partial_plan() {
    let source = snapshot();
    let mut destination = snapshot();
    destination.objects.clear();
    let before = destination.clone();
    let mut bad = bindings();
    bad[1].source_id = "ring";
    assert_eq!(
        plan_retained_overlay(&source, &destination, &bad),
        Err(LayerError::ClipboardBinding)
    );
    assert_eq!(destination, before);
    bad = bindings();
    bad[1].physical_id = "native-ring";
    assert_eq!(
        plan_retained_overlay(&source, &destination, &bad),
        Err(LayerError::ClipboardBinding)
    );
    assert_eq!(
        plan_retained_overlay(&source, &destination, &bindings()[..2]),
        Err(LayerError::ClipboardBinding)
    );
}
#[test]
fn palette_only_retained_overlay_accepts_no_geometry_and_chooses_work_layer() {
    let mut source = snapshot();
    source.objects.clear();
    source.layers.iter_mut().for_each(|row| row.locked = true);
    let result = plan_retained_overlay(&source, &LayerSnapshotV1::empty("archive"), &[]).unwrap();
    assert_eq!(result.layers.len(), 4);
    assert!(result.objects.is_empty());
    assert!(
        result
            .layers
            .iter()
            .filter(|row| !row.locked)
            .all(|row| row.name.starts_with("OM9 Transfer Work"))
    );
    assert_eq!(source.layers.len(), 3);
}
#[test]
fn retained_overlay_requires_a_palette_destination_and_resolves_uuid_collisions() {
    let mut source = snapshot();
    source.objects.truncate(1);
    assert_eq!(
        plan_retained_overlay(&source, &snapshot(), &bindings()[..1]),
        Err(LayerError::InvalidSnapshot)
    );
    let mut destination = LayerSnapshotV1::empty("archive");
    destination
        .layers
        .push(layer("metal", None, &["Different native path"]));
    let result = plan_retained_overlay(&source, &destination, &bindings()[..1]).unwrap();
    assert_ne!(result.objects[0].layer_id, "metal");
    assert!(result.layers.iter().any(|row|row.source_id=="metal"&&row.path_components==["Different native path"]));
}
