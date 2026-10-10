#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{
    layer_clipboard::{GeometryObjectBinding, bind_geometry_objects},
    layer_state::*,
};
fn binding<'a>(physical_id: &'a str, source_id: &'a str) -> GeometryObjectBinding<'a> {
    GeometryObjectBinding {
        physical_id,
        source_id,
    }
}
#[test]
fn exact_tagged_binding_uses_physical_identity_and_retains_full_palette_and_own_state() {
    let mut source = snapshot();
    source.generation = u64::MAX;
    source.layers.push(layer("empty", None, &["Empty color"]));
    source.objects[0].locked = true;
    source.objects[1].color_source = ColorSource::ByObject;
    let before = source.clone();
    let result = bind_geometry_objects(
        &source,
        &[
            binding("native-3", "detail-curve"),
            binding("native-1", "ring"),
            binding("native-2", "stone"),
        ],
    )
    .unwrap();
    assert_eq!(source, before);
    assert_eq!(result.layers, source.layers);
    assert_eq!(result.generation, u64::MAX);
    assert_eq!(result.active_layer, source.active_layer);
    let mut expected = source.objects;
    for (row, id) in expected
        .iter_mut()
        .zip(["native-1", "native-2", "native-3"])
    {
        row.id = id.into();
    }
    assert_eq!(result.objects, expected);
}
#[test]
fn missing_duplicate_unknown_or_invalid_binding_rejects_instead_of_mapping_by_order() {
    let source = snapshot();
    let before = source.clone();
    for rows in [
        vec![binding("a", "ring"), binding("b", "stone")],
        vec![
            binding("a", "ring"),
            binding("b", "ring"),
            binding("c", "detail-curve"),
        ],
        vec![
            binding("a", "ring"),
            binding("a", "stone"),
            binding("c", "detail-curve"),
        ],
        vec![
            binding("a", "ring"),
            binding("b", "unknown"),
            binding("c", "detail-curve"),
        ],
        vec![
            binding("a", "ring"),
            binding("b", "stone"),
            binding("c", ""),
        ],
        vec![
            binding("a", "ring"),
            binding("b", "stone"),
            binding("", "detail-curve"),
        ],
    ] {
        assert_eq!(
            bind_geometry_objects(&source, &rows),
            Err(LayerError::ClipboardBinding)
        );
        assert_eq!(source, before);
    }
    let oversized = "x".repeat(1025);
    assert_eq!(
        bind_geometry_objects(
            &source,
            &[
                binding(&oversized, "ring"),
                binding("b", "stone"),
                binding("c", "detail-curve")
            ]
        ),
        Err(LayerError::LimitExceeded)
    );
}
#[test]
fn selected_subset_keeps_empty_layers_but_never_accepts_unserialized_objects() {
    let mut source = snapshot();
    source.objects.truncate(1);
    let result = bind_geometry_objects(&source, &[binding("copied-ring", "ring")]).unwrap();
    assert_eq!(result.layers.len(), 3);
    assert_eq!(result.objects.len(), 1);
    assert_eq!(
        bind_geometry_objects(&source, &[binding("a", "ring"), binding("b", "stone")]),
        Err(LayerError::ClipboardBinding)
    );
    source.objects.clear();
    assert_eq!(bind_geometry_objects(&source, &[]).unwrap(), source);
}
