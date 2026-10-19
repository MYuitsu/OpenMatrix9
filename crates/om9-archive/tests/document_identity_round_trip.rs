// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::{Archive, DocumentSnapshot};
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Curve, Geometry},
    identity::{FeatureId, GroupId, LayerId, ObjectId},
    model::{
        Document, Entity, Extension, Feature, FeatureKind, GEM_EXTENSION_SLOT, Group, Layer,
        extension_slot_known,
    },
    transaction::Cancellation,
};
use sha2::{Digest, Sha256};

const GEM_PAYLOAD: &str = r#"{"carat":0.75,"stones":3,"vendor":{"cut":"round"}}"#;

struct Fixture {
    document: Document,
    first: ObjectId,
    second: ObjectId,
    layer: LayerId,
    group: GroupId,
    feature: FeatureId,
}
fn domain_end() -> f64 {
    12f64.hypot(6.)
}
fn identity_fixture() -> Fixture {
    let mut document = Document::new("identity document");
    let layer = Layer::new("Layer 01", [10, 20, 30, 255]);
    document.layers.insert(layer.id, layer.clone());
    document.layer_order.push(layer.id);
    document.current_layer = Some(layer.id);
    let mut first = Entity::new(
        "segment",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [3., 4., 0.],
            domain: [0., domain_end()],
        }),
        Some(layer.id),
    );
    let extension = Extension::new(1, GEM_PAYLOAD).expect("gem payload");
    first
        .extensions
        .insert(GEM_EXTENSION_SLOT.to_string(), extension);
    let second = Entity::new("point", Geometry::Point([1., 2., 3.]), Some(layer.id));
    document.entities.insert(first.id, first.clone());
    document.entities.insert(second.id, second.clone());
    let group = Group::new("pair", vec![first.id, second.id]);
    document.groups.insert(group.id, group.clone());
    let feature = Feature::new("semantic", FeatureKind::Semantic, vec![first.id]);
    document.features.insert(feature.id, feature.clone());
    Fixture {
        document,
        first: first.id,
        second: second.id,
        layer: layer.id,
        group: group.id,
        feature: feature.id,
    }
}
fn line_domain_end(document: &Document, id: ObjectId) -> f64 {
    match &document.entities[&id].geometry {
        Geometry::Curve(Curve::Line { domain, .. }) => domain[1],
        other => panic!("expected a line entity, found {other:?}"),
    }
}
/// Schema-4 metadata chunk in the crate's declared container layout.
fn pack_json(version: u32, metadata: &serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(metadata).unwrap();
    let mut out = vec![0u8; 88];
    out[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    out[8..12].copy_from_slice(&version.to_le_bytes());
    out[12..16].copy_from_slice(&1u32.to_le_bytes());
    out[16..20].copy_from_slice(&1u32.to_le_bytes());
    out[40..48].copy_from_slice(&88u64.to_le_bytes());
    out[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    out[56..88].copy_from_slice(&Sha256::digest(&json));
    out.extend_from_slice(&json);
    out
}
fn geometry(tag: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        tag.to_string(),
        serde_json::json!({ "surface": "unbuildable" }),
    );
    serde_json::Value::Object(map)
}
fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn entity_feature_and_group_ids_survive_save_and_reopen() {
    let limits = Limits::default();
    let fixture = identity_fixture();
    let folder =
        std::env::temp_dir().join(format!("om9-document-identity-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("identity.om9");
    Archive::save(&file, &fixture.document, &limits, &Cancellation::default()).unwrap();
    let reopened = Archive::open(&file, &limits).unwrap();
    assert_eq!(reopened.id, fixture.document.id);
    assert_eq!(
        reopened.entities.keys().collect::<Vec<_>>(),
        fixture.document.entities.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        reopened.layers.keys().collect::<Vec<_>>(),
        vec![&fixture.layer]
    );
    assert_eq!(
        reopened.groups.keys().collect::<Vec<_>>(),
        vec![&fixture.group]
    );
    assert_eq!(
        reopened.features.keys().collect::<Vec<_>>(),
        vec![&fixture.feature]
    );
    assert_eq!(
        reopened.groups[&fixture.group].members,
        vec![fixture.first, fixture.second]
    );
    assert_eq!(
        reopened.features[&fixture.feature].outputs,
        vec![fixture.first]
    );
    let extension = reopened.entities[&fixture.first]
        .extension(GEM_EXTENSION_SLOT)
        .expect("om9.gem survives the reopen");
    assert_eq!(extension.version, 1);
    assert_eq!(extension.payload, GEM_PAYLOAD);
    assert!(extension_slot_known(GEM_EXTENSION_SLOT));
    assert_eq!(reopened, fixture.document);
    assert_eq!(line_domain_end(&reopened, fixture.first), domain_end());
    assert_eq!(
        line_domain_end(&reopened, fixture.first).to_bits(),
        domain_end().to_bits()
    );
    std::fs::remove_file(&file).unwrap();
    std::fs::remove_dir(&folder).unwrap();
}
#[test]
fn unknown_geometry_variant_is_unsupported_not_downgraded() {
    let limits = Limits::default();
    let fixture = identity_fixture();
    let value = serde_json::to_value(&fixture.document).unwrap();
    for tag in ["Brep", "BRep", "B-rep"] {
        let mut variant = value.clone();
        variant["entities"][fixture.first.to_string()]["geometry"] = geometry(tag);
        assert_eq!(
            Archive::decode(&pack_json(4, &variant), &limits),
            Err(Error::Invalid("typed document metadata/schema"))
        );
    }
    // A declared geometry kind this build never had an envelope for is an
    // unsupported document variant, not a malformed document.
    let mut variant = value.clone();
    variant["entities"][fixture.first.to_string()]["geometry"] = geometry("T-Spline");
    assert_eq!(
        Archive::decode(&pack_json(4, &variant), &limits),
        Err(Error::Unsupported("document variant"))
    );
    // A wrong shape is still structurally invalid.
    let mut variant = value.clone();
    variant["entities"][fixture.first.to_string()]["geometry"] = serde_json::json!({ "Curve": 42 });
    assert_eq!(
        Archive::decode(&pack_json(4, &variant), &limits),
        Err(Error::Invalid("typed document metadata/schema"))
    );
    let bytes = Archive::encode(&fixture.document, &limits).unwrap();
    assert_eq!(Archive::decode(&bytes, &limits).unwrap(), fixture.document);
}
#[test]
fn a_document_without_the_new_keys_is_byte_identical() {
    let limits = Limits::default();
    let mut document = Document::new("plain document");
    let layer = Layer::new("Layer 01", [1, 2, 3, 255]);
    document.layers.insert(layer.id, layer.clone());
    document.layer_order.push(layer.id);
    document.current_layer = Some(layer.id);
    let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), Some(layer.id));
    document.entities.insert(entity.id, entity.clone());
    let bytes = Archive::encode(&document, &limits).unwrap();
    // A legacy document with none of the additive records keeps the exact
    // metadata bytes the unchanged serde path produced.
    let golden = serde_json::to_vec(&document).unwrap();
    let text = String::from_utf8(golden.clone()).unwrap();
    for key in ["\"groups\"", "\"features\"", "\"extensions\""] {
        assert!(!text.contains(key), "additive key {key} was written");
    }
    assert_eq!(bytes.len(), 88 + golden.len());
    assert_eq!(&bytes[88..], golden.as_slice());
    assert_eq!(bytes, Archive::encode(&document, &limits).unwrap());
    assert_eq!(Archive::decode(&bytes, &limits).unwrap(), document);
    assert_eq!(
        unhex(
            serde_json::from_str::<serde_json::Value>(
                &DocumentSnapshot::capture(&document, &limits)
                    .unwrap()
                    .to_json()
                    .unwrap()
            )
            .unwrap()["payload"]
                .as_str()
                .unwrap()
        ),
        bytes
    );
}
#[test]
fn additive_keys_are_admitted_in_schema4_and_refused_by_legacy_schema3() {
    let limits = Limits::default();
    let fixture = identity_fixture();
    let value = serde_json::to_value(&fixture.document).unwrap();
    assert!(value.get("groups").is_some());
    assert!(value.get("features").is_some());
    let entity = fixture.first.to_string();
    assert!(value["entities"][&entity].get("extensions").is_some());
    // Schema 4 admits all three additive paths unchanged.
    assert_eq!(
        Archive::decode(&pack_json(4, &value), &limits).unwrap(),
        fixture.document
    );
    let refusal = Error::Invalid("legacy schema has no group/feature/extension fields");
    let mut groups_only = value.clone();
    groups_only.as_object_mut().unwrap().remove("features");
    groups_only["entities"][&entity]
        .as_object_mut()
        .unwrap()
        .remove("extensions");
    assert!(matches!(
        Archive::migrate_from_v3(&pack_json(3, &groups_only), &limits),
        Err(Error::Invalid(
            "legacy schema has no group/feature/extension fields"
        ))
    ));
    let mut features_only = value.clone();
    features_only.as_object_mut().unwrap().remove("groups");
    features_only["entities"][&entity]
        .as_object_mut()
        .unwrap()
        .remove("extensions");
    assert!(matches!(
        Archive::migrate_from_v3(&pack_json(3, &features_only), &limits),
        Err(ref error) if *error == refusal
    ));
    let mut extensions_only = value.clone();
    extensions_only.as_object_mut().unwrap().remove("groups");
    extensions_only.as_object_mut().unwrap().remove("features");
    assert!(matches!(
        Archive::migrate_from_v3(&pack_json(3, &extensions_only), &limits),
        Err(ref error) if *error == refusal
    ));
    // A schema-3 document without them still migrates explicitly.
    let plain = Document::new("plain");
    assert!(
        Archive::migrate_from_v3(
            &pack_json(3, &serde_json::to_value(plain).unwrap()),
            &limits
        )
        .is_ok()
    );
}

/// OM9-L3-01-08-003 persistence: the document tolerance travels with the document,
/// a legacy payload that carries it is refused, and a document without it stays
/// byte-identical.
#[test]
fn a_document_tolerance_is_persisted_admitted_and_refused_in_a_legacy_payload() {
    let limits = Limits::default();
    let tolerance = om9_core::history::Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap();
    let mut document = Document::new("tolerance persistence");
    document.tolerance = Some(tolerance.clone());
    let folder = std::env::temp_dir().join(format!("om9-tolerance-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("tolerance.om9");
    Archive::save(&file, &document, &limits, &Cancellation::default()).unwrap();
    let reopened = Archive::open(&file, &limits).unwrap();
    assert_eq!(reopened.tolerance(), Some(&tolerance));

    // Schema 4 admits the additive key.
    let value = serde_json::to_value(&document).unwrap();
    assert!(value.get("tolerance").is_some());
    assert_eq!(
        Archive::decode(&pack_json(4, &value), &limits).unwrap(),
        document
    );

    // A schema-3 payload may not carry it: it needs an explicit migration.
    assert!(matches!(
        Archive::migrate_from_v3(&pack_json(3, &value), &limits),
        Err(Error::Invalid(
            "legacy schema has no document tolerance field"
        ))
    ));

    // A document without a policy writes no key and decodes byte-identically.
    let plain = Document::new("no tolerance");
    let plain_value = serde_json::to_value(&plain).unwrap();
    assert!(plain_value.get("tolerance").is_none());
    assert_eq!(
        Archive::decode(&pack_json(4, &plain_value), &limits).unwrap(),
        plain
    );
}
