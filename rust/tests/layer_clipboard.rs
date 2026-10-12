#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_clipboard::*, layer_state::*};

fn geometry() -> Vec<u8> {
    let mut value = b"3D Geometry File Format       50".to_vec();
    value.extend_from_slice(b"stub-body-for-header-policy-test");
    value
}
#[test]
fn bound_metadata_is_owned_keeps_palette_and_own_flags() {
    let mut source = snapshot();
    source.generation = u64::MAX;
    source.layers.push(layer("empty", None, &["Empty"]));
    source.layers[0].locked = true;
    source.objects[1].locked = true;
    source.objects[1].color_source = ColorSource::ByObject;
    source.objects[1].rgb = [3, 5, 7];
    let body = geometry();
    let mut metadata = prepare_clipboard(&source, LayerClipboardScope::Selected, &body).unwrap();
    let received = receive_clipboard(&body, Some(&metadata)).unwrap();
    metadata.fill(0);
    assert_eq!(received.snapshot.as_ref(), Some(&source));
    assert_eq!(received.scope, Some(LayerClipboardScope::Selected));
    assert_eq!(received.evidence, PaletteEvidence::Extended);
    assert_eq!(received.geometry_version, 50);
}
#[test]
fn native_only_never_claims_complete_empty_palette() {
    let received = receive_clipboard(&geometry(), None).unwrap();
    assert_eq!(received.evidence, PaletteEvidence::NativeOnly);
    assert!(received.snapshot.is_none() && received.scope.is_none());
}
#[test]
fn digest_length_format_and_version_mismatches_reject() {
    let body = geometry();
    let metadata = prepare_clipboard(&snapshot(), LayerClipboardScope::Session, &body).unwrap();
    let mut other = body.clone();
    other[35] ^= 1;
    assert_eq!(
        receive_clipboard(&other, Some(&metadata)),
        Err(LayerError::ClipboardBinding)
    );
    for (key, value) in [
        ("geometry_length", serde_json::json!("1")),
        ("geometry_version", serde_json::json!(5)),
        ("geometry_format", serde_json::json!("text/plain")),
        ("geometry_sha256", serde_json::to_value([0u8; 32]).unwrap()),
    ] {
        let mut changed: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
        changed[key] = value;
        assert_eq!(
            receive_clipboard(&body, Some(&serde_json::to_vec(&changed).unwrap())),
            Err(LayerError::ClipboardBinding)
        );
    }
}
#[test]
fn unknown_versions_fields_truncated_and_missing_metadata_are_fatal() {
    let body = geometry();
    let metadata = prepare_clipboard(&snapshot(), LayerClipboardScope::Selected, &body).unwrap();
    for key in ["version", "layer_schema"] {
        let mut changed: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
        changed[key] = serde_json::json!(2);
        assert_eq!(
            receive_clipboard(&body, Some(&serde_json::to_vec(&changed).unwrap())),
            Err(LayerError::UnsupportedVersion)
        );
    }
    assert!(receive_clipboard(&body, Some(&metadata[..metadata.len() - 1])).is_err());
    assert_eq!(
        receive_clipboard(&body, Some(b"")),
        Err(LayerError::InvalidSnapshot)
    );
    let mut extra: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
    extra["render_mesh"] = serde_json::json!([]);
    assert_eq!(
        receive_clipboard(&body, Some(&serde_json::to_vec(&extra).unwrap())),
        Err(LayerError::InvalidSnapshot)
    );
}
#[test]
fn payload_header_and_size_policy_is_not_sdk_geometry_acceptance() {
    for body in [
        b"".as_slice(),
        b"H:/a.3dm",
        b"3D Geometry File Format       80invalid",
        b"3D Geometry File Format       50",
    ] {
        assert_eq!(
            receive_clipboard(body, None),
            Err(LayerError::ClipboardGeometry)
        );
    }
    assert_eq!(
        check_geometry_size(MAX_GEOMETRY_BYTES + 1),
        Err(LayerError::LimitExceeded)
    );
    assert_eq!(check_geometry_size(32), Err(LayerError::ClipboardGeometry));
    let mut v5 = geometry();
    v5[24..32].copy_from_slice(b"       5");
    assert_eq!(receive_clipboard(&v5, None).unwrap().geometry_version, 5);
}
#[test]
fn palette_only_session_and_invalid_source_are_explicit() {
    let mut source = LayerSnapshotV1::empty("palette");
    source.layers.push(layer("empty", None, &["Empty"]));
    let bytes = prepare_clipboard(&source, LayerClipboardScope::Session, &geometry()).unwrap();
    assert!(
        receive_clipboard(&geometry(), Some(&bytes))
            .unwrap()
            .snapshot
            .unwrap()
            .objects
            .is_empty()
    );
    source.layers[0].parent_id = Some("missing".into());
    assert_eq!(
        prepare_clipboard(&source, LayerClipboardScope::Selected, &geometry()),
        Err(LayerError::MissingLayer)
    );
}
