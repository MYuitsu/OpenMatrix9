#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{
    layer_clipboard::MAX_GEOMETRY_BYTES, layer_codec::encode_snapshot, layer_ffi::*,
    layer_state::MAX_METADATA_BYTES,
};
fn view(bytes: &[u8]) -> ByteView {
    ByteView {
        data: bytes.as_ptr(),
        len: bytes.len(),
    }
}
fn geometry() -> Vec<u8> {
    let mut value = b"3D Geometry File Format       50".to_vec();
    value.extend_from_slice(b"header-only-policy-fixture");
    value
}
#[test]
fn prepared_binding_owns_bytes_and_is_a_distinct_handle_kind() {
    unsafe {
        let json = encode_snapshot(&snapshot()).unwrap();
        let mut source = 0;
        assert_eq!(om9_layer_snapshot_from_json(view(&json), &mut source), 0);
        let mut body = geometry();
        let mut binding = 0;
        assert_eq!(
            om9_layer_clipboard_prepare(source, 1, view(&body), &mut binding),
            0
        );
        body.fill(0);
        assert_eq!(om9_layer_snapshot_free(source), 0);
        assert_eq!(om9_layer_snapshot_free(binding), 13);
        assert_eq!(om9_layer_plan_free(binding), 13);
        let mut needed = 99;
        assert_eq!(
            om9_layer_clipboard_bytes(binding, std::ptr::null_mut(), 0, &mut needed),
            15
        );
        let mut output = vec![0x5a; needed + 1];
        let mut small = 0;
        assert_eq!(
            om9_layer_clipboard_bytes(binding, output.as_mut_ptr(), needed - 1, &mut small),
            15
        );
        assert_eq!(small, needed);
        assert!(output.iter().all(|&v| v == 0x5a));
        assert_eq!(
            om9_layer_clipboard_bytes(binding, output.as_mut_ptr(), needed, &mut small),
            0
        );
        assert_eq!(output[needed], 0x5a);
        let mut received = 77;
        let mut info = ClipboardInfo::default();
        assert_eq!(
            om9_layer_clipboard_receive(
                view(&geometry()),
                view(&output[..needed]),
                1,
                &mut received,
                &mut info
            ),
            0
        );
        assert_eq!(info.evidence, 2);
        assert_eq!(info.scope, 1);
        assert_eq!(info.geometry_version, 50);
        assert_eq!(om9_layer_snapshot_free(received), 0);
        assert_eq!(om9_layer_clipboard_free(binding), 0);
        needed = 99;
        assert_eq!(
            om9_layer_clipboard_bytes(binding, std::ptr::null_mut(), 0, &mut needed),
            13
        );
        assert_eq!(needed, 0);
    }
}
#[test]
fn native_only_and_invalid_binding_never_leak_a_snapshot() {
    unsafe {
        let body = geometry();
        let mut handle = 999;
        let mut info = ClipboardInfo::default();
        assert_eq!(
            om9_layer_clipboard_receive(view(&body), view(b""), 0, &mut handle, &mut info),
            0
        );
        assert_eq!(handle, 0);
        assert_eq!(info.evidence, 1);
        assert_eq!(info.scope, 0);
        handle = 999;
        assert_eq!(
            om9_layer_clipboard_receive(view(&body), view(b""), 1, &mut handle, &mut info),
            1
        );
        assert_eq!(handle, 0);
        assert_eq!(info.evidence, 0);
        let json = encode_snapshot(&snapshot()).unwrap();
        let mut source = 0;
        assert_eq!(om9_layer_snapshot_from_json(view(&json), &mut source), 0);
        let mut binding = 0;
        assert_eq!(
            om9_layer_clipboard_prepare(source, 2, view(&body), &mut binding),
            0
        );
        let mut needed = 0;
        assert_eq!(
            om9_layer_clipboard_bytes(binding, std::ptr::null_mut(), 0, &mut needed),
            15
        );
        let mut metadata = vec![0; needed];
        assert_eq!(
            om9_layer_clipboard_bytes(binding, metadata.as_mut_ptr(), needed, &mut needed),
            0
        );
        let mut wrong = body.clone();
        wrong[35] ^= 1;
        handle = 999;
        assert_eq!(
            om9_layer_clipboard_receive(view(&wrong), view(&metadata), 1, &mut handle, &mut info),
            17
        );
        assert_eq!(handle, 0);
        assert_eq!(info.evidence, 0);
        assert_eq!(om9_layer_clipboard_free(source), 13);
        assert_eq!(om9_layer_snapshot_free(source), 0);
        assert_eq!(om9_layer_clipboard_free(binding), 0);
    }
}
#[test]
fn oversize_views_reject_before_dereference_and_reset_outputs() {
    unsafe {
        let oversized = ByteView {
            data: std::ptr::null(),
            len: MAX_GEOMETRY_BYTES + 1,
        };
        let mut handle = 999;
        let mut info = ClipboardInfo::default();
        assert_eq!(om9_layer_clipboard_prepare(0, 1, oversized, &mut handle), 8);
        assert_eq!(handle, 0);
        assert_eq!(
            om9_layer_clipboard_receive(oversized, view(b""), 0, &mut handle, &mut info),
            8
        );
        assert_eq!(handle, 0);
        let metadata = ByteView {
            data: std::ptr::null(),
            len: MAX_METADATA_BYTES + 1,
        };
        assert_eq!(
            om9_layer_clipboard_receive(view(&geometry()), metadata, 1, &mut handle, &mut info),
            8
        );
        assert_eq!(handle, 0);
        assert_eq!(om9_layer_clipboard_check_size(1, MAX_GEOMETRY_BYTES + 1), 8);
        assert_eq!(om9_layer_clipboard_check_size(2, MAX_METADATA_BYTES + 1), 8);
        assert_eq!(om9_layer_clipboard_check_size(1, 32), 16);
    }
}

#[test]
fn invalid_controls_outputs_and_wrong_handles_reject_without_consuming_state() {
    unsafe {
        let body = geometry();
        let json = encode_snapshot(&snapshot()).unwrap();
        let mut source = 0;
        assert_eq!(om9_layer_snapshot_from_json(view(&json), &mut source), 0);
        let mut out = 999;
        assert_eq!(
            om9_layer_clipboard_prepare(source, 3, view(&body), &mut out),
            14
        );
        assert_eq!(out, 0);
        assert_eq!(
            om9_layer_clipboard_prepare(source, 1, view(&body), std::ptr::null_mut()),
            1
        );
        let mut info = ClipboardInfo::default();
        for (present, bytes) in [(2, b"".as_slice()), (0, b"unexpected".as_slice())] {
            out = 999;
            assert_eq!(
                om9_layer_clipboard_receive(view(&body), view(bytes), present, &mut out, &mut info),
                1
            );
            assert_eq!((out, info.evidence), (0, 0));
        }
        out = 999;
        assert_eq!(
            om9_layer_clipboard_receive(view(&body), view(b""), 0, &mut out, std::ptr::null_mut()),
            1
        );
        assert_eq!(out, 0);
        let mut needed = 999;
        assert_eq!(
            om9_layer_clipboard_bytes(source, std::ptr::null_mut(), 0, &mut needed),
            13
        );
        assert_eq!(needed, 0);
        assert_eq!(om9_layer_clipboard_check_size(3, 0), 14);
        assert_eq!(om9_layer_snapshot_free(source), 0);
    }
}

#[test]
fn native_geometry_binding_ffi_rejects_missing_or_duplicate_tags_before_overlay() {
    unsafe {
        let json = encode_snapshot(&snapshot()).unwrap();
        let mut source = 0;
        assert_eq!(om9_layer_snapshot_from_json(view(&json), &mut source), 0);
        let mut rows = [
            GeometryBindingView {
                physical: view(b"p1"),
                source: view(b"ring"),
            },
            GeometryBindingView {
                physical: view(b"p2"),
                source: view(b"stone"),
            },
            GeometryBindingView {
                physical: view(b"p3"),
                source: view(b"detail-curve"),
            },
        ];
        let mut after = 0;
        assert_eq!(
            om9_layer_clipboard_bind_objects(source, rows.as_ptr(), rows.len(), &mut after),
            0
        );
        let mut info = ObjectInfo::default();
        assert_eq!(om9_layer_snapshot_object(after, 0, &mut info), 0);
        let mut needed = 0;
        let mut id = [0; 2];
        assert_eq!(
            om9_layer_snapshot_text(after, 2, 0, 0, id.as_mut_ptr(), 2, &mut needed),
            0
        );
        assert_eq!(&id, b"p1");
        assert_eq!(om9_layer_snapshot_free(after), 0);
        rows[1].source = view(b"ring");
        after = 999;
        assert_eq!(
            om9_layer_clipboard_bind_objects(source, rows.as_ptr(), rows.len(), &mut after),
            17
        );
        assert_eq!(after, 0);
        assert_eq!(
            om9_layer_clipboard_bind_objects(source, std::ptr::null(), 2_000_001, &mut after),
            8
        );
        assert_eq!(om9_layer_snapshot_free(source), 0);
    }
}
