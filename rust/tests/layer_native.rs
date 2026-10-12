use openmatrix9_rust::layer_native::{
    PersistentFields, decode_native_persistent, encode_native_persistent,
};

#[test]
fn preserves_exact_unset_and_explicit_fields_when_native_facts_match() {
    for child in [false, true] {
        for fields in [
            PersistentFields {
                locked: None,
                visible: None,
            },
            PersistentFields {
                locked: Some(true),
                visible: Some(false),
            },
        ] {
            let marker = encode_native_persistent(child, true, false, fields).unwrap();
            assert_eq!(
                decode_native_persistent(child, true, false, &marker).unwrap(),
                fields
            );
        }
    }
}

#[test]
fn native_edits_take_precedence_over_old_presence_witness() {
    let marker = encode_native_persistent(
        true,
        false,
        true,
        PersistentFields {
            locked: None,
            visible: None,
        },
    )
    .unwrap();
    assert_eq!(
        decode_native_persistent(true, true, false, &marker).unwrap(),
        PersistentFields {
            locked: Some(true),
            visible: Some(false)
        }
    );
    assert_eq!(
        decode_native_persistent(false, false, true, &marker).unwrap(),
        PersistentFields {
            locked: None,
            visible: None
        }
    );
}

#[test]
fn native_only_uses_observed_desired_fields_without_inventing_unset_evidence() {
    assert_eq!(
        decode_native_persistent(true, false, true, b"").unwrap(),
        PersistentFields {
            locked: Some(false),
            visible: Some(true)
        }
    );
    assert_eq!(
        decode_native_persistent(false, true, false, b"").unwrap(),
        PersistentFields {
            locked: None,
            visible: None
        }
    );
}

#[test]
fn malformed_oversized_or_unknown_marker_is_rejected() {
    for marker in [b"garbage".as_slice(), br#"{"version":2,"child":false,"locked":false,"visible":true,"persistent_locked":null,"persistent_visible":null}"#.as_slice(), &[b'x'; 257]] {
        assert!(decode_native_persistent(false, false, true, marker).is_err());
    }
}

#[test]
fn contradictory_child_desired_fields_fail_before_native_write() {
    assert!(
        encode_native_persistent(
            true,
            false,
            true,
            PersistentFields {
                locked: Some(true),
                visible: None
            }
        )
        .is_err()
    );
    assert!(
        encode_native_persistent(
            true,
            false,
            true,
            PersistentFields {
                locked: None,
                visible: Some(false)
            }
        )
        .is_err()
    );
}

#[test]
fn ffi_copies_into_caller_buffers_and_roundtrips_fields() {
    use openmatrix9_rust::layer_ffi::{
        ByteView, LayerInfo, om9_layer_native_persistent_decode, om9_layer_native_persistent_encode,
    };
    let mut needed = 0;
    // All spans and outputs below belong to this synchronous call.
    unsafe {
        assert_eq!(
            om9_layer_native_persistent_encode(
                1,
                1,
                0,
                -1,
                -1,
                std::ptr::null_mut(),
                0,
                &mut needed
            ),
            15
        );
        assert!(needed > 0 && needed <= 256);
        let mut bytes = vec![0; needed];
        assert_eq!(
            om9_layer_native_persistent_encode(
                1,
                1,
                0,
                -1,
                -1,
                bytes.as_mut_ptr(),
                bytes.len(),
                &mut needed
            ),
            0
        );
        let mut result = LayerInfo::default();
        assert_eq!(
            om9_layer_native_persistent_decode(
                1,
                1,
                0,
                ByteView {
                    data: bytes.as_ptr(),
                    len: bytes.len()
                },
                &mut result
            ),
            0
        );
        assert_eq!(
            (
                result.locked,
                result.visible,
                result.persistent_locked,
                result.persistent_visible
            ),
            (1, 0, -1, -1)
        );
    }
}

#[test]
fn ffi_rejects_bad_flags_spans_and_clears_error_outputs() {
    use openmatrix9_rust::layer_ffi::{
        ByteView, LayerInfo, om9_layer_native_persistent_decode, om9_layer_native_persistent_encode,
    };
    let mut needed = 99;
    unsafe {
        assert_ne!(
            om9_layer_native_persistent_encode(
                2,
                1,
                0,
                -1,
                -1,
                std::ptr::null_mut(),
                0,
                &mut needed
            ),
            0
        );
        assert_eq!(needed, 0);
        let mut result = LayerInfo {
            locked: 1,
            ..LayerInfo::default()
        };
        assert_ne!(
            om9_layer_native_persistent_decode(
                1,
                1,
                0,
                ByteView {
                    data: std::ptr::null(),
                    len: 1
                },
                &mut result
            ),
            0
        );
        assert_eq!(result.locked, 0);
        let bytes = [b'x'; 257];
        assert_ne!(
            om9_layer_native_persistent_decode(
                1,
                1,
                0,
                ByteView {
                    data: bytes.as_ptr(),
                    len: bytes.len()
                },
                &mut result
            ),
            0
        );
    }
}
