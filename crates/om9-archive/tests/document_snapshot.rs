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
fn snapshot_decode_admission_can_refuse_without_changing_the_source() {
    let limits = Limits::default();
    let (document, ..) = fixture();
    let snapshot = DocumentSnapshot::capture(&document, &limits).unwrap();
    assert!(snapshot.storage_usage().unwrap().bytes >= snapshot.encoded_len());
    let digest = snapshot.sha256().to_owned();
    let mut calls = 0;
    assert!(matches!(snapshot.restore_with_admission(&limits, &mut |bytes| {
        calls += 1;
        assert!(bytes >= snapshot.encoded_len());
        Err(Error::Budget("injected snapshot admission"))
    }), Err(Error::Budget("injected snapshot admission"))));
    assert_eq!(calls, 1);
    assert_eq!(snapshot.sha256(), digest);
    assert_eq!(snapshot.restore(&limits).unwrap(), document);
}
#[test]
fn snapshot_roundtrips_native_brep_chunked_payload() {
    use om9_core::geometry::{Mesh, NativeBrep};
    let limits = Limits::default();
    let mut document = Document::new("native payload snapshot");
    let native = NativeBrep {
        display: Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] },
        payload: vec![37; 70_000].into(), crc: 123,
    };
    let entity = Entity::new("retained", Geometry::NativeBrep(native), None);
    document.entities.insert(entity.id, entity);
    let snapshot = DocumentSnapshot::capture(&document, &limits).unwrap();
    let encoded = snapshot.to_json().unwrap();
    let restored = DocumentSnapshot::from_json(&encoded, &limits).unwrap().restore(&limits).unwrap();
    assert_eq!(restored, document);
}

#[test]
fn reserved_open_rejects_growth_after_open_and_releases_guard() {
    use std::{io::Write, sync::{Arc, atomic::{AtomicBool, Ordering}}};
    use om9_archive::Archive;
    use om9_core::transaction::Cancellation;
    struct Guard(Arc<AtomicBool>);
    impl Drop for Guard { fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); } }
    let limits = Limits::default();
    let path = std::env::temp_dir().join(format!("om9-open-growth-{}.om9", uuid::Uuid::new_v4()));
    Archive::save(&path, &Document::new("input"), &limits, &Cancellation::default()).unwrap();
    let initial_size = std::fs::metadata(&path).unwrap().len();
    let released = Arc::new(AtomicBool::new(false));
    let result = Archive::open_restored_reserved(&path, &limits, &Default::default(), |size| {
        assert_eq!(size, initial_size);
        std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(&[42]).unwrap();
        Ok(Guard(Arc::clone(&released)))
    });
    assert!(matches!(result, Err(Error::Budget("growing archive file"))));
    assert!(released.load(Ordering::SeqCst));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn reserved_open_refusal_does_not_read_archive() {
    use om9_archive::Archive;
    let limits = Limits::default();
    let path = std::env::temp_dir().join(format!("om9-open-refusal-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, b"not an archive").unwrap();
    let result = Archive::open_restored_reserved(&path, &limits, &Default::default(), |size| {
        assert_eq!(size, 14);
        Err::<(), _>(Error::Budget("test reservation"))
    });
    assert!(matches!(result, Err(Error::Budget("test reservation"))));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn staged_open_releases_input_guard_if_restore_reservation_fails() {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use om9_archive::Archive;
    struct Guard(Arc<AtomicBool>);
    impl Drop for Guard { fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); } }
    let path = std::env::temp_dir().join(format!("om9-open-stage-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, b"bad archive").unwrap();
    let released = Arc::new(AtomicBool::new(false));
    let result = Archive::open_restored_reserved_staged(&path, &Limits::default(), &Default::default(),
        |_| Ok(Guard(Arc::clone(&released))),
        |_, bytes| { assert_eq!(bytes, 11); Err(Error::Budget("test growth")) });
    assert!(matches!(result, Err(Error::Budget("test growth"))));
    assert!(released.load(Ordering::SeqCst));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn preflight_open_refuses_during_streaming_and_releases_guard() {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use om9_archive::Archive;
    struct Guard(Arc<AtomicBool>);
    impl Drop for Guard { fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); } }
    let path = std::env::temp_dir().join(format!("om9-open-preflight-{}.om9", uuid::Uuid::new_v4()));
    let limits = Limits::default();
    Archive::save(&path, &Document::new("preflight"), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let released = Arc::new(AtomicBool::new(false));
    let mut calls = 0;
    let result = Archive::open_restored_reserved_preflight(&path, &limits, &Default::default(),
        |_| Ok(Guard(Arc::clone(&released))), |_, _| Ok(()),
        |_, estimated| { assert!(estimated > 0); calls += 1; if calls == 5 { Err(Error::Budget("test typed preflight")) } else { Ok(()) } });
    assert!(matches!(result, Err(Error::Budget("test typed preflight"))));
    assert_eq!(calls, 5);
    assert!(released.load(Ordering::SeqCst));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn preflight_reserves_chunk_table_before_allocating_it() {
    use om9_archive::Archive;
    let path = std::env::temp_dir().join(format!("om9-open-chunks-{}.om9", uuid::Uuid::new_v4()));
    let limits = Limits::default();
    Archive::save(&path, &Document::new("chunk preflight"), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let size = std::fs::metadata(&path).unwrap().len();
    let mut calls = 0;
    let result = Archive::open_restored_reserved_preflight(&path, &limits, &Default::default(),
        |_| Ok(()), |_, _| Ok(()), |_, estimated| {
            calls += 1;
            assert!(estimated > size);
            Err(Error::Budget("test chunk table"))
        });
    assert!(matches!(result, Err(Error::Budget("test chunk table"))));
    assert_eq!(calls, 1);
    std::fs::remove_file(path).unwrap();
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
