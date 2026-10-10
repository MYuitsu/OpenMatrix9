use openmatrix9_rust::phase3_ffi::{
    Facts, SnapshotView, om9_phase3_capability, om9_phase3_message, om9_phase3_request_begin,
    om9_phase3_request_cancel, om9_phase3_request_validate,
};
use openmatrix9_rust::phase3_modeling::Reason;
fn view(name: &[u8], digest: &[u8]) -> SnapshotView {
    SnapshotView {
        identity: name.as_ptr(),
        identity_len: name.len(),
        signature: digest.as_ptr(),
        signature_len: digest.len(),
    }
}
#[test]
fn abi_rejects_invalid_flags_null_arrays_and_open_shell() {
    let good = Facts {
        kind: 5,
        valid: 1,
        closed: 1,
        protected: 0,
        reserved: 0,
        components: 6,
    };
    let mut values = [good, good];
    unsafe {
        assert_eq!(om9_phase3_capability(4, values.as_ptr(), 2, true), 0);
        values[1].kind = 4;
        values[1].closed = 0;
        assert_eq!(
            om9_phase3_capability(4, values.as_ptr(), 2, true),
            Reason::ClosedSolidRequired as u32
        );
        assert_ne!(om9_phase3_capability(4, std::ptr::null(), 2, true), 0);
        values[1] = good;
        values[1].valid = 2;
        assert_ne!(om9_phase3_capability(4, values.as_ptr(), 2, true), 0);
        assert_ne!(om9_phase3_capability(999, values.as_ptr(), 2, true), 0);
        assert_ne!(om9_phase3_capability(4, values.as_ptr(), 257, true), 0);
    }
}
#[test]
fn abi_registry_owns_snapshots_and_cancel_releases_handle() {
    let doc = b"Document";
    let names = b"A.Edge1";
    let signature = b"geometry-and-world-placement";
    let views = [view(names, signature)];
    unsafe {
        let handle = om9_phase3_request_begin(doc.as_ptr(), doc.len(), views.as_ptr(), views.len());
        assert_ne!(handle, 0);
        assert_eq!(
            om9_phase3_request_validate(
                handle,
                doc.as_ptr(),
                doc.len(),
                views.as_ptr(),
                views.len(),
                true
            ),
            0
        );
        let changed = [view(names, b"modified")];
        assert_eq!(
            om9_phase3_request_validate(handle, doc.as_ptr(), doc.len(), changed.as_ptr(), 1, true),
            Reason::Stale as u32
        );
        assert_eq!(
            om9_phase3_request_validate(handle, doc.as_ptr(), doc.len(), views.as_ptr(), 1, false),
            Reason::Unavailable as u32
        );
        om9_phase3_request_cancel(handle);
        assert_eq!(
            om9_phase3_request_validate(handle, doc.as_ptr(), doc.len(), views.as_ptr(), 1, true),
            Reason::Cancelled as u32
        );
    }
}
#[test]
fn abi_bounds_utf8_and_invalid_identity_do_not_allocate_a_request() {
    unsafe {
        assert_eq!(
            om9_phase3_request_begin(std::ptr::null(), 1, std::ptr::null(), 1),
            0
        );
        assert_eq!(
            om9_phase3_request_begin(b"D".as_ptr(), 1, std::ptr::null(), 257),
            0
        );
        let bad = [view(&[255], b"digest")];
        assert_eq!(
            om9_phase3_request_begin(b"D".as_ptr(), 1, bad.as_ptr(), 1),
            0
        );
        let huge = [SnapshotView {
            identity: std::ptr::null(),
            identity_len: usize::MAX,
            signature: std::ptr::null(),
            signature_len: 0,
        }];
        assert_eq!(
            om9_phase3_request_begin(b"D".as_ptr(), 1, huge.as_ptr(), 1),
            0
        );
    }
}
#[test]
fn native_reason_text_is_owned_and_bounded() {
    let mut bytes = [0u8; 128];
    assert!(unsafe { om9_phase3_message(4, bytes.as_mut_ptr(), bytes.len()) } > 0);
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .contains("closed solid")
    );
    let mut short = [255u8; 1];
    assert_eq!(unsafe { om9_phase3_message(7, short.as_mut_ptr(), 1) }, 0);
    assert_eq!(short, [0]);
    assert_eq!(unsafe { om9_phase3_message(7, std::ptr::null_mut(), 0) }, 0);
}
