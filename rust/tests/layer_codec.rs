#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_codec::*, layer_state::*};

#[test]
fn metadata_roundtrip_is_owned_preserves_empty_palette_and_local_flags() {
    let mut state = snapshot();
    state.generation = u64::MAX;
    state.layers[0].locked = true;
    state.layers[2].persistent_locked = Some(false);
    state.layers[2].persistent_visible = Some(true);
    state.layers.push(layer("empty", None, &["Empty same RGB"]));
    state.objects[1].color_source = ColorSource::ByObject;
    state.objects[1].rgb = [2, 4, 6];
    let mut bytes = encode_snapshot(&state).unwrap();
    let decoded = decode_snapshot(&bytes).unwrap();
    bytes.fill(0);
    assert_eq!(decoded, state);
    assert!(effective_state(&decoded, "detail-curve").unwrap().locked);
    assert_eq!(decoded.layers[2].persistent_locked, Some(false));
}
#[test]
fn metadata_rejects_unknown_version_duplicate_fields_truncation_and_geometry() {
    let bytes = encode_snapshot(&snapshot()).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(
        decode_snapshot(text.replace("\"version\":1", "\"version\":2").as_bytes()).unwrap_err(),
        LayerError::UnsupportedVersion
    );
    assert!(decode_snapshot(&text.as_bytes()[..text.len() - 1]).is_err());
    assert!(
        decode_snapshot(
            text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        decode_snapshot(
            text.replacen("\"version\":1", "\"version\":1,\"vertices\":[[1,2,3]]", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert_eq!(decode_snapshot(&[0xff]), Err(LayerError::InvalidSnapshot));
}
#[test]
fn invalid_native_facts_cannot_be_encoded_or_received_through_json() {
    let mut state = snapshot();
    state.layers[2].parent_id = Some("absent".into());
    assert_eq!(encode_snapshot(&state), Err(LayerError::MissingLayer));
    let bytes = encode_snapshot(&snapshot()).unwrap();
    let text = String::from_utf8(bytes)
        .unwrap()
        .replace("\"parent_id\":\"metal\"", "\"parent_id\":\"absent\"");
    assert_eq!(
        decode_snapshot(text.as_bytes()),
        Err(LayerError::MissingLayer)
    );
}

#[test]
fn generation_has_a_lossless_canonical_decimal_wire_value() {
    let mut state = snapshot();
    state.generation = u64::MAX;
    let text = String::from_utf8(encode_snapshot(&state).unwrap()).unwrap();
    assert!(text.contains("\"generation\":\"18446744073709551615\""));
    for invalid in ["01", "-1", "18446744073709551616", "1e3", "+1", ""] {
        let changed = text.replace("\"18446744073709551615\"", &format!("\"{invalid}\""));
        assert!(decode_snapshot(changed.as_bytes()).is_err());
    }
}
