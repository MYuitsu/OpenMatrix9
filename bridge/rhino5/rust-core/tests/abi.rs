use om9_rhino_layer::*;
use openmatrix9_rust::{layer_codec::encode_snapshot, layer_state::LayerSnapshotV1};
fn bytes(v: &[u8]) -> Bytes {
    Bytes {
        data: v.as_ptr(),
        len: v.len(),
    }
}
fn state() -> Vec<u8> {
    encode_snapshot(&LayerSnapshotV1::empty("Matrix ABI fixture")).unwrap()
}
#[test]
fn result_query_short_buffer_free_and_stale_handle() {
    let s = state();
    let mut h = 0;
    assert_eq!(unsafe { om9_rhino_layer_validate(bytes(&s), &mut h) }, 0);
    let mut needed = 0;
    assert_eq!(
        unsafe { om9_rhino_layer_result(h, std::ptr::null_mut(), 0, &mut needed) },
        15
    );
    assert!(needed > 4);
    let mut canary = [0x7f; 4];
    assert_eq!(
        unsafe { om9_rhino_layer_result(h, canary.as_mut_ptr(), canary.len(), &mut needed) },
        15
    );
    assert_eq!(canary, [0x7f; 4]);
    let mut out = vec![0; needed];
    let cap = out.len();
    assert_eq!(
        unsafe { om9_rhino_layer_result(h, out.as_mut_ptr(), cap, &mut needed) },
        0
    );
    assert_eq!(out, s);
    assert_eq!(om9_rhino_layer_free(h), 0);
    assert_eq!(om9_rhino_layer_free(h), 13);
    assert_eq!(
        unsafe { om9_rhino_layer_result(h, std::ptr::null_mut(), 0, &mut needed) },
        13
    );
    assert_eq!(needed, 0);
}
#[test]
fn invalid_spans_reset_output_before_error_and_do_not_allocate() {
    let mut h = 987;
    assert_eq!(
        unsafe {
            om9_rhino_layer_validate(
                Bytes {
                    data: std::ptr::null(),
                    len: 4,
                },
                &mut h,
            )
        },
        1
    );
    assert_eq!(h, 0);
    h = 987;
    assert_eq!(
        unsafe {
            om9_rhino_layer_validate(
                Bytes {
                    data: std::ptr::null(),
                    len: usize::MAX,
                },
                &mut h,
            )
        },
        8
    );
    assert_eq!(h, 0);
    let s = state();
    assert_eq!(
        unsafe { om9_rhino_layer_validate(bytes(&s), std::ptr::null_mut()) },
        1
    );
}
#[test]
fn stale_check_catches_unreported_changes_with_same_generation() {
    let a = state();
    let b = encode_snapshot(&LayerSnapshotV1::empty("different native document")).unwrap();
    assert_eq!(unsafe { om9_rhino_layer_check(bytes(&a), bytes(&a)) }, 0);
    assert_eq!(unsafe { om9_rhino_layer_check(bytes(&a), bytes(&b)) }, 12);
}
#[test]
fn persistent_presence_uses_shared_rust_witness_and_native_edits_win() {
    let witness = r#"{"version":1,"child":true,"locked":false,"visible":true,"persistent_locked":null,"persistent_visible":null}"#;
    for changed in [false, true] {
        let input = serde_json::to_vec(
            &serde_json::json!({"child":true,"locked":changed,"visible":true,"witness":witness}),
        )
        .unwrap();
        let mut h = 0;
        assert_eq!(
            unsafe { om9_rhino_layer_persistent(bytes(&input), 0, &mut h) },
            0
        );
        let mut needed = 0;
        assert_eq!(
            unsafe { om9_rhino_layer_result(h, std::ptr::null_mut(), 0, &mut needed) },
            15
        );
        let mut out = vec![0; needed];
        assert_eq!(
            unsafe { om9_rhino_layer_result(h, out.as_mut_ptr(), out.len(), &mut needed) },
            0
        );
        let row: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            row["persistent_locked"],
            if changed {
                serde_json::json!(true)
            } else {
                serde_json::Value::Null
            }
        );
        assert!(row["persistent_visible"].is_null());
        assert_eq!(om9_rhino_layer_free(h), 0);
    }
}
