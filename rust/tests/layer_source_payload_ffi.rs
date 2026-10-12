use openmatrix9_rust::{layer_ffi::*, layer_source_payload::*, layer_state::LayerError};
use sha2::{Digest, Sha256};
fn view(s: &str) -> ByteView {
    ByteView {
        data: s.as_ptr(),
        len: s.len(),
    }
}
#[test]
fn payload_abi_owns_binary_without_retaining_borrowed_native_chunks() {
    let mut encoded = String::from("AP+A");
    let hash = format!("{:x}", Sha256::digest([0, 255, 128]));
    let chunks = [view(&encoded)];
    let mut handle = 0;
    assert_eq!(
        unsafe { om9_layer_source_payload_prepare(chunks.as_ptr(), 1, view(&hash), &mut handle) },
        0
    );
    encoded.clear();
    let mut needed = 999;
    assert_eq!(
        unsafe { om9_layer_source_payload_bytes(handle, std::ptr::null_mut(), 0, &mut needed) },
        LayerError::BufferTooSmall as u32
    );
    assert_eq!(needed, 3);
    let mut output = vec![0; needed];
    assert_eq!(
        unsafe {
            om9_layer_source_payload_bytes(handle, output.as_mut_ptr(), output.len(), &mut needed)
        },
        0
    );
    assert_eq!(output, [0, 255, 128]);
    let mut written = 999;
    let mut one = [0];
    assert_eq!(
        unsafe { om9_layer_source_payload_range(handle, 1, one.as_mut_ptr(), 1, &mut written) },
        0
    );
    assert_eq!(one, [255]);
    assert_eq!(written, 1);
    assert_eq!(
        unsafe { om9_layer_source_payload_range(handle, 3, one.as_mut_ptr(), 1, &mut written) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(written, 0);
    assert_eq!(
        unsafe {
            om9_layer_source_payload_range(
                handle,
                0,
                std::ptr::null_mut(),
                CHUNK_BYTES + 1,
                &mut written,
            )
        },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(written, 0);
    let manifest=serde_json::json!({"schema_version":1,"archive_sha256":hash,"scale_mm":1.0,"records":[],"components":[],"issues":[]}).to_string();
    assert_eq!(
        unsafe { om9_layer_source_manifest_verify(handle, view(&manifest), view(&manifest)) },
        0
    );
    assert_eq!(
        unsafe { om9_layer_source_manifest_verify(handle, view("{}"), view(&manifest)) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(
        om9_layer_collection_free(handle),
        LayerError::InvalidHandle as u32
    );
    assert_eq!(om9_layer_source_payload_free(handle), 0);
    assert_eq!(
        om9_layer_source_payload_free(handle),
        LayerError::InvalidHandle as u32
    );
    needed = 999;
    assert_eq!(
        unsafe { om9_layer_source_payload_bytes(handle, std::ptr::null_mut(), 0, &mut needed) },
        LayerError::InvalidHandle as u32
    );
    assert_eq!(needed, 0);
}
#[test]
fn payload_abi_resets_outputs_and_rejects_oversized_views_before_reading() {
    let hash = format!("{:x}", Sha256::digest(b"f"));
    let mut out = 777;
    assert_eq!(
        unsafe {
            om9_layer_source_payload_prepare(
                std::ptr::null(),
                MAX_CHUNKS + 1,
                view(&hash),
                &mut out,
            )
        },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(out, 0);
    let oversized = ByteView {
        data: std::ptr::null(),
        len: MAX_ENCODED_CHUNK + 1,
    };
    out = 777;
    assert_eq!(
        unsafe { om9_layer_source_payload_prepare(&oversized, 1, view(&hash), &mut out) },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(out, 0);
    let null = ByteView {
        data: std::ptr::null(),
        len: 4,
    };
    out = 777;
    assert_eq!(
        unsafe { om9_layer_source_payload_prepare(&null, 1, view(&hash), &mut out) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(out, 0);
    let malformed = view("Zh==");
    out = 777;
    assert_eq!(
        unsafe { om9_layer_source_payload_prepare(&malformed, 1, view(&hash), &mut out) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(out, 0);
}

#[test]
fn source_manifest_witness_abi_checks_owned_digest_handles_and_span_budget() {
    let hash = format!("{:x}", Sha256::digest(b"f"));
    let mut handle = 0;
    let chunk = view("Zg==");
    assert_eq!(
        unsafe { om9_layer_source_payload_prepare(&chunk, 1, view(&hash), &mut handle) },
        0
    );
    let manifest = |id: &str| {
        serde_json::json!({"schema_version":1,"source_version":50,"archive_sha256":hash,"scale_mm":1.0,
        "records":[],"issues":[],"components":[{"class_name":"ON_DimStyle","component_type":"AnnotationStyle",
        "role":"top-level","capability":"retained","source_uuid":id,"name":"Old style","dependencies":[]}]}).to_string()
    };
    let stored = manifest("11111111-1111-4111-8111-111111111111");
    let first = manifest("22222222-2222-4222-8222-222222222222");
    let second = manifest("33333333-3333-4333-8333-333333333333");
    assert_eq!(
        unsafe {
            om9_layer_source_manifest_verify_witness(
                handle,
                view(&stored),
                view(&first),
                view(&second),
            )
        },
        0
    );
    assert_eq!(
        unsafe {
            om9_layer_source_manifest_verify_witness(
                handle,
                view(&stored),
                view(&first),
                view(&first),
            )
        },
        LayerError::InvalidSnapshot as u32
    );
    let bad = ByteView {
        data: std::ptr::null(),
        len: MAX_MANIFEST_BYTES + 1,
    };
    assert_eq!(
        unsafe {
            om9_layer_source_manifest_verify_witness(handle, view(&stored), view(&first), bad)
        },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(om9_layer_source_payload_free(handle), 0);
    assert_eq!(
        unsafe {
            om9_layer_source_manifest_verify_witness(
                handle,
                view(&stored),
                view(&first),
                view(&second),
            )
        },
        LayerError::InvalidHandle as u32
    );
}
