#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_codec::encode_snapshot, layer_ffi::*, layer_state::*};
#[test]
fn retained_subset_ffi_owns_partition_and_rejects_invalid_inputs() {
    unsafe {
        let source = handle(&snapshot());
        let selection = [view(b"stone")];
        let mut result = 99;
        assert_eq!(
            om9_layer_snapshot_subset(source, selection.as_ptr(), 1, &mut result),
            0
        );
        assert_eq!(om9_layer_snapshot_free(source), 0);
        let mut counts = SnapshotCounts::default();
        assert_eq!(om9_layer_snapshot_counts(result, &mut counts), 0);
        assert_eq!(counts.object_count, 1);
        assert_eq!(counts.layer_count, 3);
        let mut effective = EffectiveView::default();
        assert_eq!(
            om9_layer_snapshot_effective(result, view(b"stone"), &mut effective),
            0
        );
        let mut invalid = 99;
        assert_eq!(
            om9_layer_snapshot_subset(result, std::ptr::null(), usize::MAX, &mut invalid),
            8
        );
        assert_eq!(invalid, 0);
        invalid = 99;
        let missing = [view(b"missing")];
        assert_eq!(
            om9_layer_snapshot_subset(result, missing.as_ptr(), 1, &mut invalid),
            3
        );
        assert_eq!(invalid, 0);
        invalid = 99;
        assert_eq!(
            om9_layer_snapshot_subset(0, std::ptr::null(), 0, &mut invalid),
            13
        );
        assert_eq!(invalid, 0);
        assert_eq!(om9_layer_snapshot_free(result), 0);
    }
}
fn view(value: &[u8]) -> ByteView {
    ByteView {
        data: value.as_ptr(),
        len: value.len(),
    }
}
unsafe fn handle(snapshot: &LayerSnapshotV1) -> u64 {
    let bytes = encode_snapshot(snapshot).unwrap();
    let mut result = 0;
    assert_eq!(
        unsafe { om9_layer_snapshot_from_json(view(&bytes), &mut result) },
        0
    );
    result
}
#[test]
fn retained_ffi_returns_owned_rebound_snapshot_with_full_palette() {
    unsafe {
        let mut source = snapshot();
        source.generation = u64::MAX;
        source.objects.truncate(1);
        source.layers[0].locked = true;
        source.layers.push(layer("empty", None, &["Empty color"]));
        let mut destination = source.clone();
        destination.objects.clear();
        destination.document_id = "archive".into();
        destination.generation = 0;
        let source = handle(&source);
        let destination = handle(&destination);
        let row = GeometryBindingView {
            physical: view(b"physical-ring"),
            source: view(b"ring"),
        };
        let mut result = 99;
        assert_eq!(
            om9_layer_retained_overlay(source, destination, &row, 1, &mut result),
            0
        );
        assert_eq!(om9_layer_snapshot_free(source), 0);
        assert_eq!(om9_layer_snapshot_free(destination), 0);
        let mut counts = SnapshotCounts::default();
        assert_eq!(om9_layer_snapshot_counts(result, &mut counts), 0);
        assert_eq!(counts.layer_count, 4);
        assert_eq!(counts.object_count, 1);
        assert_eq!(counts.generation, u64::MAX);
        let mut state = EffectiveView::default();
        assert_eq!(
            om9_layer_snapshot_effective(result, view(b"physical-ring"), &mut state),
            0
        );
        assert_eq!(state.locked, 1);
        assert_eq!(om9_layer_snapshot_free(result), 0);
    }
}
#[test]
fn retained_ffi_clears_outputs_and_rejects_invalid_handles_counts_and_bindings() {
    unsafe {
        let source = handle(&snapshot());
        let destination = handle(&LayerSnapshotV1::empty("archive"));
        let mut result = 99;
        assert_eq!(
            om9_layer_retained_overlay(
                source,
                destination,
                std::ptr::null(),
                usize::MAX,
                &mut result
            ),
            8
        );
        assert_eq!(result, 0);
        result = 99;
        assert_eq!(
            om9_layer_retained_overlay(source, destination, std::ptr::null(), 0, &mut result),
            17
        );
        assert_eq!(result, 0);
        result = 99;
        assert_eq!(
            om9_layer_retained_overlay(0, destination, std::ptr::null(), 0, &mut result),
            13
        );
        assert_eq!(result, 0);
        assert_eq!(om9_layer_snapshot_free(source), 0);
        assert_eq!(om9_layer_snapshot_free(destination), 0);
    }
}
