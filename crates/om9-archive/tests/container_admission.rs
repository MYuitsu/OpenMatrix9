// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::Archive;
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Cloud, Curve, Geometry, Mesh, NurbsCurve},
    model::{Document, Entity},
};
use sha2::{Digest, Sha256};

fn fixture() -> Document {
    let mut d = Document::new("mixed document");
    for (label, g) in [
        ("point", Geometry::Point([1., 2., 3.])),
        (
            "rational",
            Geometry::Curve(Curve::Nurbs(NurbsCurve {
                degree: 2,
                poles: vec![[10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
                weights: vec![1., std::f64::consts::FRAC_1_SQRT_2, 1.],
                knots: vec![0., 0., 0., 1., 1., 1.],
                domain: [0., 1.],
                periodic: false,
                reversed: false,
                seam: None,
            })),
        ),
        (
            "mesh",
            Geometry::Mesh(Mesh {
                vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                faces: vec![vec![0, 1, 2]],
                normals: vec![],
                colors: vec![],
            }),
        ),
        (
            "cloud",
            Geometry::Cloud(Cloud {
                points: vec![[3., 4., 5.], [6., 7., 8.]],
                normals: vec![],
                colors: vec![[255, 0, 0, 255], [0, 255, 0, 255]],
                hidden: vec![false, true],
            }),
        ),
    ] {
        let e = Entity::new(label, g, d.current_layer);
        d.entities.insert(e.id, e);
    }
    d
}
fn pack_json(json: &[u8]) -> Vec<u8> {
    // Schema 4: 16-byte header, one 72-byte metadata entry, then payload.
    let mut out = vec![0u8; 88];
    out[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    out[8..12].copy_from_slice(&4u32.to_le_bytes());
    out[12..16].copy_from_slice(&1u32.to_le_bytes());
    out[16..20].copy_from_slice(&1u32.to_le_bytes());
    out[40..48].copy_from_slice(&88u64.to_le_bytes());
    out[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    out[56..88].copy_from_slice(&Sha256::digest(json));
    out.extend_from_slice(json);
    out
}
#[test]
fn typed_document_roundtrip_preserves_master_identity_basis_attributes_and_channels() {
    let d = fixture();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    let decoded = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(decoded, d);
    assert_eq!(decoded.entities.len(), 4);
}
#[test]
fn checksum_truncation_version_and_trailing_payload_corruption_reject() {
    let bytes = Archive::encode(&fixture(), &Limits::default()).unwrap();
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(Archive::decode(&corrupt, &Limits::default()).is_err());
    assert!(Archive::decode(&bytes[..bytes.len() - 1], &Limits::default()).is_err());
    let mut version = bytes.clone();
    version[8..12].copy_from_slice(&99u32.to_le_bytes());
    assert!(matches!(
        Archive::decode(&version, &Limits::default()),
        Err(Error::Unsupported(_))
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Archive::decode(&trailing, &Limits::default()).is_err());
}
#[test]
fn offset_and_length_overflow_are_rejected_before_payload_allocation() {
    let bytes = Archive::encode(&fixture(), &Limits::default()).unwrap();
    for (offset, length) in [(u64::MAX, 1), (88, u64::MAX), (16, 1)] {
        let mut bad = bytes.clone();
        bad[40..48].copy_from_slice(&offset.to_le_bytes());
        bad[48..56].copy_from_slice(&length.to_le_bytes());
        assert!(Archive::decode(&bad, &Limits::default()).is_err());
    }
    let mut count = bytes;
    count[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Archive::decode(&count, &Limits::default()).is_err());
}
#[test]
fn overlapping_chunks_are_rejected_even_with_valid_payload_checksums() {
    let json = serde_json::to_vec(&fixture()).unwrap();
    let mut out = vec![0u8; 160];
    out[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    out[8..12].copy_from_slice(&4u32.to_le_bytes());
    out[12..16].copy_from_slice(&2u32.to_le_bytes());
    for start in [16, 88] {
        out[start..start + 4].copy_from_slice(&1u32.to_le_bytes());
        out[start + 24..start + 32].copy_from_slice(&160u64.to_le_bytes());
        out[start + 32..start + 40].copy_from_slice(&(json.len() as u64).to_le_bytes());
        out[start + 40..start + 72].copy_from_slice(&Sha256::digest(&json));
    }
    out[96] = 1;
    out.extend_from_slice(&json);
    assert_eq!(
        Archive::decode(&out, &Limits::default()),
        Err(Error::Invalid("chunk overlap"))
    );
}
#[test]
fn duplicate_json_identity_is_rejected_without_silent_last_record_wins() {
    let d = fixture();
    let (id, e) = d.entities.iter().next().unwrap();
    let mut value = serde_json::to_value(&d).unwrap();
    value["entities"] = serde_json::json!({});
    let text = serde_json::to_string(&value).unwrap();
    let entity = serde_json::to_string(e).unwrap();
    let duplicate = format!("\"entities\":{{\"{id}\":{entity},\"{id}\":{entity}}}");
    let text = text.replace("\"entities\":{}", &duplicate);
    assert_eq!(
        Archive::decode(&pack_json(text.as_bytes()), &Limits::default()),
        Err(Error::Invalid("duplicate JSON key"))
    );
}

#[test]
fn uuid_spelling_aliases_cannot_overwrite_a_different_master_during_typed_decode() {
    let mut d = Document::new("UUID aliases");
    let mut entity = Entity::new("first", Geometry::Point([1., 2., 3.]), None);
    entity.id = om9_core::identity::ObjectId(
        uuid::Uuid::parse_str("10000000-0000-4000-8000-abcdef123456").unwrap(),
    );
    d.entities.insert(entity.id, entity.clone());
    let mut value = serde_json::to_value(d).unwrap();
    value["entities"] = serde_json::json!({});
    let text = serde_json::to_string(&value).unwrap();
    let first = serde_json::to_string(&entity).unwrap();
    entity.geometry = Geometry::Point([9., 8., 7.]);
    let second = serde_json::to_string(&entity).unwrap();
    for alias in [
        entity.id.to_string().to_uppercase(),
        entity.id.0.simple().to_string(),
        format!("{{{}}}", entity.id),
    ] {
        let map = format!(
            "\"entities\":{{\"{}\":{first},\"{alias}\":{second}}}",
            entity.id
        );
        let input = text.replace("\"entities\":{}", &map);
        assert_eq!(
            Archive::decode(&pack_json(input.as_bytes()), &Limits::default()),
            Err(Error::Invalid("noncanonical UUID map key"))
        );
    }
}
#[test]
fn unknown_fields_and_missing_required_metadata_do_not_migrate_implicitly() {
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["unknown_field"] = serde_json::json!(true);
    assert!(
        Archive::decode(
            &pack_json(&serde_json::to_vec(&value).unwrap()),
            &Limits::default()
        )
        .is_err()
    );
    value.as_object_mut().unwrap().remove("unknown_field");
    value.as_object_mut().unwrap().remove("page_unit");
    assert!(
        Archive::decode(
            &pack_json(&serde_json::to_vec(&value).unwrap()),
            &Limits::default()
        )
        .is_err()
    );
}
#[test]
fn aggregate_json_nodes_and_strings_are_preflighted_before_typed_decode() {
    let bytes = Archive::encode(&fixture(), &Limits::default()).unwrap();
    let limits = Limits {
        max_nodes: 3,
        ..Limits::default()
    };
    assert!(matches!(
        Archive::decode(&bytes, &limits),
        Err(Error::Budget(_))
    ));
    let deep = "[".repeat(40) + &"]".repeat(40);
    assert!(matches!(
        Archive::decode(&pack_json(deep.as_bytes()), &Limits::default()),
        Err(Error::Budget(_))
    ));
}
