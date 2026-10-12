use openmatrix9_rust::{layer_source_payload::*, layer_state::LayerError};
use sha2::{Digest, Sha256};
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[test]
fn payload_decodes_binary_and_canonical_padding_with_exact_digest() {
    for (encoded, plain) in [
        ("", b"".as_slice()),
        ("Zg==", b"f"),
        ("Zm8=", b"fo"),
        ("Zm9v", b"foo"),
        ("AP+A", &[0, 255, 128]),
    ] {
        assert_eq!(
            decode_chunks(&[encoded], &digest(plain), MAX_PAYLOAD_BYTES).unwrap(),
            plain
        );
    }
}
#[test]
fn payload_checks_digest_and_rejects_noncanonical_or_malformed_encoding() {
    assert_eq!(
        decode_chunks(&["Zg=="], &digest(b"g"), MAX_PAYLOAD_BYTES),
        Err(LayerError::InvalidSnapshot)
    );
    for bad in [
        "Zh==", "Zm9=", "Zg", "Zg===", "=m9v", "Zm=v", "Zg==AAAA", "Zg==\n", "é===", "____",
    ] {
        assert!(
            decode_chunks(&[bad], &digest(b"f"), MAX_PAYLOAD_BYTES).is_err(),
            "{bad}"
        );
    }
    assert!(decode_chunks(&["Zg=="], &digest(b"f").to_uppercase(), MAX_PAYLOAD_BYTES).is_err());
}
#[test]
fn payload_respects_native_chunk_boundaries_and_total_budget() {
    let full = "AAAA".repeat(CHUNK_BYTES / 3) + "AA==";
    let mut expected = vec![0; CHUNK_BYTES];
    expected.push(b'f');
    assert_eq!(
        decode_chunks(&[&full, "Zg=="], &digest(&expected), MAX_PAYLOAD_BYTES).unwrap(),
        expected
    );
    assert_eq!(
        decode_chunks(&[&full, "Zg=="], &digest(&expected), CHUNK_BYTES),
        Err(LayerError::LimitExceeded)
    );
    for parts in [vec![], vec!["Zg==", "Zg=="], vec![&full, ""], vec!["", ""]] {
        assert!(decode_chunks(&parts, &digest(b"f"), MAX_PAYLOAD_BYTES).is_err());
    }
    assert_eq!(
        decode_chunks(&vec![""; MAX_CHUNKS + 1], &digest(b""), MAX_PAYLOAD_BYTES),
        Err(LayerError::LimitExceeded)
    );
    assert_eq!(
        decode_chunks(&["Zg=="], &digest(b"f"), MAX_PAYLOAD_BYTES + 1),
        Err(LayerError::LimitExceeded)
    );
}
#[test]
fn manifest_is_trusted_only_when_it_matches_the_actual_native_source_inventory() {
    let hash = digest(b"f");
    let native = serde_json::json!({"schema_version":1,"archive_sha256":hash,"scale_mm":1.0,"records":[],"components":[],"issues":[]});
    let bytes = serde_json::to_vec(&native).unwrap();
    assert!(verify_manifest(&bytes, &bytes, &hash).is_ok());
    let mut forged = native.clone();
    forged["records"] = serde_json::json!([{"source_uuid":"invented"}]);
    assert!(verify_manifest(&serde_json::to_vec(&forged).unwrap(), &bytes, &hash).is_err());
    assert!(verify_manifest(&bytes, &bytes, &digest(b"g")).is_err());
    for (field, value) in [
        ("schema_version", serde_json::json!(2)),
        ("scale_mm", serde_json::json!(0)),
        ("issues", serde_json::json!(["missing dependency"])),
    ] {
        let mut invalid = native.clone();
        invalid[field] = value;
        let invalid = serde_json::to_vec(&invalid).unwrap();
        assert!(verify_manifest(&invalid, &invalid, &hash).is_err());
    }
    assert!(verify_manifest(b"null", b"null", &hash).is_err());
}

fn legacy_manifest(id: &str) -> serde_json::Value {
    serde_json::json!({"schema_version":1,"source_version":50,"archive_sha256":digest(b"f"),"scale_mm":1.0,
        "records":[],"issues":[],"components":[{"class_name":"ON_DimStyle","component_type":"AnnotationStyle",
        "role":"top-level","capability":"retained","source_uuid":id,"name":"Millimeter Small","dependencies":[]}]})
}
fn witness_result(
    stored: &serde_json::Value,
    first: &serde_json::Value,
    second: &serde_json::Value,
) -> Result<(), LayerError> {
    verify_manifest_with_witness(
        &serde_json::to_vec(stored).unwrap(),
        &serde_json::to_vec(first).unwrap(),
        &serde_json::to_vec(second).unwrap(),
        &digest(b"f"),
    )
}
#[test]
fn legacy_unused_dimstyle_identity_requires_an_independent_unstable_native_witness() {
    let stored = legacy_manifest("11111111-1111-4111-8111-111111111111");
    let first = legacy_manifest("22222222-2222-4222-8222-222222222222");
    let second = legacy_manifest("33333333-3333-4333-8333-333333333333");
    assert!(witness_result(&stored, &first, &second).is_ok());
    assert!(
        witness_result(&stored, &first, &first).is_err(),
        "stable native ID cannot be forged"
    );
    assert!(witness_result(&stored, &first, &stored).is_ok());
    let mut altered = stored.clone();
    altered["components"][0]["name"] = serde_json::json!("different");
    assert!(witness_result(&altered, &first, &second).is_err());
    let mut modern = stored.clone();
    modern["source_version"] = serde_json::json!(80);
    let mut modern_first = first.clone();
    modern_first["source_version"] = serde_json::json!(80);
    let mut modern_second = second.clone();
    modern_second["source_version"] = serde_json::json!(80);
    assert!(witness_result(&modern, &modern_first, &modern_second).is_err());
}
#[test]
fn native_witness_does_not_relax_geometry_layer_block_or_referenced_style_identity() {
    let stored = legacy_manifest("11111111-1111-4111-8111-111111111111");
    let first = legacy_manifest("22222222-2222-4222-8222-222222222222");
    let second = legacy_manifest("33333333-3333-4333-8333-333333333333");
    for kind in ["ON_NurbsCurve", "ON_Layer", "ON_InstanceDefinition"] {
        let mut a = stored.clone();
        let mut b = first.clone();
        let mut c = second.clone();
        for row in [&mut a, &mut b, &mut c] {
            row["components"][0]["class_name"] = serde_json::json!(kind);
        }
        assert!(witness_result(&a, &b, &c).is_err(), "{kind}");
    }
    let mut referenced = first.clone();
    referenced["records"] = serde_json::json!([{"source_uuid":"44444444-4444-4444-8444-444444444444","text":"prefix 22222222-2222-4222-8222-222222222222 suffix"}]);
    assert!(witness_result(&stored, &referenced, &second).is_err());
    let mut self_reference = first.clone();
    self_reference["components"][0]["user_strings"] =
        serde_json::json!(["22222222-2222-4222-8222-222222222222"]);
    assert!(witness_result(&stored, &self_reference, &second).is_err());
    let mut duplicate = stored.clone();
    duplicate["components"]
        .as_array_mut()
        .unwrap()
        .push(stored["components"][0].clone());
    assert!(witness_result(&duplicate, &first, &second).is_err());
    let mut records = stored.clone();
    records["records"] =
        serde_json::json!([{"source_uuid":"44444444-4444-4444-8444-444444444444"}]);
    assert!(witness_result(&records, &first, &second).is_err());
}
