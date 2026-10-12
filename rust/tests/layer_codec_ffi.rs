#[path = "common/layers.rs"]
mod common;
use openmatrix9_rust::{layer_codec::*, layer_ffi::*, layer_state::*};
#[test]
fn json_abi_owns_input_and_copies_results_without_partial_buffer_writes() {
    let expected = common::snapshot();
    let mut source = encode_snapshot(&expected).unwrap();
    let mut handle = 999;
    let view = ByteView {
        data: source.as_ptr(),
        len: source.len(),
    };
    assert_eq!(
        unsafe { om9_layer_snapshot_from_json(view, &mut handle) },
        0
    );
    source.fill(0);
    let mut needed = 0;
    assert_eq!(
        unsafe { om9_layer_snapshot_json(handle, std::ptr::null_mut(), 0, &mut needed) },
        LayerError::BufferTooSmall as u32
    );
    let mut output = vec![0x77; needed + 1];
    assert_eq!(
        unsafe { om9_layer_snapshot_json(handle, output.as_mut_ptr(), needed - 1, &mut needed) },
        LayerError::BufferTooSmall as u32
    );
    assert!(output.iter().all(|b| *b == 0x77));
    assert_eq!(
        unsafe { om9_layer_snapshot_json(handle, output.as_mut_ptr(), needed, &mut needed) },
        0
    );
    assert_eq!(output[needed], 0x77);
    assert_eq!(decode_snapshot(&output[..needed]).unwrap(), expected);
    assert_eq!(om9_layer_snapshot_free(handle), 0);
    assert_eq!(
        unsafe { om9_layer_snapshot_json(handle, output.as_mut_ptr(), output.len(), &mut needed) },
        LayerError::InvalidHandle as u32
    );
}
#[test]
fn json_abi_resets_output_on_error_and_checks_oversized_views_before_reading() {
    let mut handle = 999;
    let bad = ByteView {
        data: std::ptr::null(),
        len: MAX_METADATA_BYTES + 1,
    };
    assert_eq!(
        unsafe { om9_layer_snapshot_from_json(bad, &mut handle) },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(handle, 0);
    let input = b"{\"version\":1}";
    handle = 999;
    assert_eq!(
        unsafe {
            om9_layer_snapshot_from_json(
                ByteView {
                    data: input.as_ptr(),
                    len: input.len(),
                },
                &mut handle,
            )
        },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(handle, 0);
}
