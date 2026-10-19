// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::DocumentSnapshot;
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Curve, Geometry},
    identity::{DocumentId, FeatureId, GroupId, LayerId, ObjectId},
    model::{Document, Entity, Feature, FeatureKind, Group, Layer},
};

/// The stored parameter is a value that cannot be printed exactly as a short
/// decimal, so a lossy float path would change its bits.
fn domain_end() -> f64 {
    12f64.hypot(6.)
}
fn fixture() -> (Document, ObjectId, LayerId, GroupId, FeatureId) {
    let mut document = Document::new("snapshot source");
    let layer = Layer::new("Layer 01", [10, 20, 30, 255]);
    document.layers.insert(layer.id, layer.clone());
    document.layer_order.push(layer.id);
    document.current_layer = Some(layer.id);
    let entity = Entity::new(
        "segment",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [3., 4., 0.],
            domain: [0., domain_end()],
        }),
        Some(layer.id),
    );
    document.entities.insert(entity.id, entity.clone());
    let group = Group::new("pair", vec![entity.id]);
    document.groups.insert(group.id, group.clone());
    let feature = Feature::new("semantic", FeatureKind::Semantic, vec![entity.id]);
    document.features.insert(feature.id, feature.clone());
    (document, entity.id, layer.id, group.id, feature.id)
}
fn line_domain_end(document: &Document, id: ObjectId) -> f64 {
    match &document.entities[&id].geometry {
        Geometry::Curve(Curve::Line { domain, .. }) => domain[1],
        other => panic!("expected a line entity, found {other:?}"),
    }
}
fn envelope(snapshot: &DocumentSnapshot) -> serde_json::Value {
    serde_json::from_str(&snapshot.to_json().unwrap()).unwrap()
}
#[test]
fn snapshot_restore_reproduces_ids_and_geometry_bits() {
    let limits = Limits::default();
    let (source, entity, layer, group, feature) = fixture();
    let snapshot = DocumentSnapshot::capture(&source, &limits).unwrap();
    assert_eq!(snapshot.document_id(), source.id);
    assert_eq!(snapshot.epoch(), source.epoch);
    assert_eq!(snapshot.sha256().len(), 64);
    let restored = snapshot.restore(&limits).unwrap();
    assert_eq!(restored.id, source.id);
    assert_eq!(restored.entities.keys().collect::<Vec<_>>(), vec![&entity]);
    assert_eq!(restored.layers.keys().collect::<Vec<_>>(), vec![&layer]);
    assert_eq!(restored.groups.keys().collect::<Vec<_>>(), vec![&group]);
    assert_eq!(restored.features.keys().collect::<Vec<_>>(), vec![&feature]);
    assert_eq!(restored.groups[&group].members, vec![entity]);
    assert_eq!(restored.features[&feature].outputs, vec![entity]);
    assert_eq!(restored, source);
    assert_eq!(
        line_domain_end(&restored, entity).to_bits(),
        line_domain_end(&source, entity).to_bits()
    );
    assert_eq!(line_domain_end(&restored, entity), domain_end());
    assert_eq!(
        line_domain_end(&restored, entity).to_bits(),
        domain_end().to_bits()
    );
    // The JSON envelope is a stable, repeatable round trip.
    let text = snapshot.to_json().unwrap();
    let parsed = DocumentSnapshot::from_json(&text, &limits).unwrap();
    assert_eq!(parsed, snapshot);
    assert_eq!(parsed.to_json().unwrap(), text);
    assert_eq!(parsed.document_id(), source.id);
    assert_eq!(parsed.epoch(), source.epoch);
    assert_eq!(parsed.sha256(), snapshot.sha256());
    assert_eq!(parsed.restore(&limits).unwrap(), source);
}
#[test]
fn restoring_another_documents_snapshot_is_refused_atomically() {
    let limits = Limits::default();
    let (source, ..) = fixture();
    let snapshot = DocumentSnapshot::capture(&source, &limits).unwrap();
    let mut value = envelope(&snapshot);
    let foreign = DocumentId::new();
    value["document_id"] = serde_json::Value::String(foreign.to_string());
    let swapped = DocumentSnapshot::from_json(&serde_json::to_string(&value).unwrap(), &limits)
        .expect("a well-formed envelope still loads");
    assert_eq!(swapped.document_id(), foreign);
    assert_ne!(swapped.document_id(), source.id);
    assert_eq!(
        swapped.restore(&limits),
        Err(Error::Invalid("snapshot document"))
    );
    // Nothing was published from the refused restore.
    assert_eq!(snapshot.restore(&limits).unwrap(), source);
}
#[test]
fn a_tampered_snapshot_is_refused() {
    let limits = Limits::default();
    let (source, ..) = fixture();
    let snapshot = DocumentSnapshot::capture(&source, &limits).unwrap();
    let text = snapshot.to_json().unwrap();
    let payload = envelope(&snapshot)["payload"]
        .as_str()
        .expect("payload text")
        .to_string();
    assert!(payload.starts_with("4f4d39000d0a1a0a"), "container magic");
    // One edited byte of the encoded payload.
    let mut edited = payload.clone().into_bytes();
    edited[0] = if edited[0] == b'0' { b'1' } else { b'0' };
    for tampered in [
        String::from_utf8(edited).expect("hex text"),
        payload.replacen("4f", "zz", 1),
        payload[..payload.len() - 1].to_string(),
        format!("{payload}00"),
    ] {
        let mut value = envelope(&snapshot);
        value["payload"] = serde_json::Value::String(tampered);
        assert_eq!(
            DocumentSnapshot::from_json(&serde_json::to_string(&value).unwrap(), &limits),
            Err(Error::Invalid("snapshot checksum"))
        );
    }
    // A rewritten digest no longer describes the stored bytes.
    let mut value = envelope(&snapshot);
    value["sha256"] = serde_json::Value::String("0".repeat(64));
    assert_eq!(
        DocumentSnapshot::from_json(&serde_json::to_string(&value).unwrap(), &limits),
        Err(Error::Invalid("snapshot checksum"))
    );
    // The untouched snapshot is still the only restorable record.
    assert_eq!(
        DocumentSnapshot::from_json(&text, &limits)
            .unwrap()
            .restore(&limits)
            .unwrap(),
        source
    );
}
#[test]
fn an_unknown_snapshot_version_is_refused_without_migration() {
    let limits = Limits::default();
    let (source, ..) = fixture();
    let snapshot = DocumentSnapshot::capture(&source, &limits).unwrap();
    let mut value = envelope(&snapshot);
    value["version"] = serde_json::Value::from(2u32);
    let future = DocumentSnapshot::from_json(&serde_json::to_string(&value).unwrap(), &limits)
        .expect("an unknown version is recorded, not guessed at");
    assert_eq!(future.epoch(), source.epoch);
    assert_eq!(
        future.restore(&limits),
        Err(Error::Unsupported(
            "OM9 schema version; no implicit migration"
        ))
    );
    value["format"] = serde_json::Value::String("om9.other.snapshot".into());
    assert_eq!(
        DocumentSnapshot::from_json(&serde_json::to_string(&value).unwrap(), &limits),
        Err(Error::Unsupported(
            "OM9 schema version; no implicit migration"
        ))
    );
    // A payload that is not the snapshot envelope at all is malformed shape.
    assert_eq!(
        DocumentSnapshot::from_json("{\"version\":1}", &limits),
        Err(Error::Invalid("snapshot format"))
    );
}
