// SPDX-License-Identifier: LGPL-2.1-or-later
//! Authored ledger ownership and schema; no native merge acceptance is claimed.
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity, Layer},
    resource::{Resource, ResourceKind},
    retained::{
        EntityBaseline, HostRef, ImportMode, ImportNamespace, NativeRecordKey, NativeRecordKind,
        NativeUnits, RecordDisposition, ResolvedBy, RetainedSource, SourceRecord, SourceUnits,
        fingerprint,
    },
};
use serde_json::json;
use uuid::Uuid;

fn record_json() -> serde_json::Value {
    json!({
        "key": {"kind":"Geometry", "id":Uuid::from_u128(1), "native_index":null},
        "family":"ON_Point", "host":{"Object":Uuid::from_u128(2)},
        "baseline_sha256":([7u8;32]), "disposition":"Unchanged", "relations":[],
        "entity_baseline":{
            "geometry_sha256":([1u8;32]), "placement_sha256":([2u8;32]),
            "attributes_sha256":([3u8;32]), "label_sha256":([4u8;32])
        }
    })
}
#[test]
fn policy_and_independent_baselines_survive_cold_serde_without_reinterpretation() {
    for mode in ["Preserve", "Geometry"] {
        let ns = json!({
            "id":Uuid::from_u128(3), "source":Uuid::from_u128(4),
            "import_mode":mode, "records":[record_json()]
        });
        let decoded: ImportNamespace =
            serde_json::from_value(ns.clone()).expect("durable import policy and field baselines");
        let cold: ImportNamespace =
            serde_json::from_slice(&serde_json::to_vec(&decoded).unwrap()).unwrap();
        assert_eq!(decoded, cold);
        assert_eq!(serde_json::to_value(cold).unwrap(), ns);
    }
}
#[test]
fn legacy_policy_and_field_baselines_stay_unknown_after_cold_serde() {
    let mut record = record_json();
    record.as_object_mut().unwrap().remove("entity_baseline");
    let ns = json!({"id":Uuid::from_u128(3),"source":Uuid::from_u128(4),"records":[record]});
    let decoded: ImportNamespace = serde_json::from_value(ns).unwrap();
    let cold: ImportNamespace =
        serde_json::from_slice(&serde_json::to_vec(&decoded).unwrap()).unwrap();
    let value = serde_json::to_value(cold).unwrap();
    assert!(
        value
            .get("import_mode")
            .is_some_and(serde_json::Value::is_null),
        "legacy mode must remain explicitly unknown"
    );
    assert!(
        value["records"][0]
            .get("entity_baseline")
            .is_some_and(serde_json::Value::is_null),
        "legacy field baseline must remain explicitly unknown"
    );
}
#[test]
fn unknown_policy_and_incomplete_baseline_packets_reject() {
    let ns = json!({"id":Uuid::from_u128(3),"source":Uuid::from_u128(4),"import_mode":"Native","records":[]});
    assert!(serde_json::from_value::<ImportNamespace>(ns).is_err());
    let mut record = record_json();
    record["entity_baseline"]
        .as_object_mut()
        .unwrap()
        .remove("placement_sha256");
    assert!(serde_json::from_value::<SourceRecord>(record).is_err());
}
#[test]
fn retained_only_records_do_not_collapse_preserve_geometry_or_unknown_policy() {
    let record = SourceRecord::retained(key(), "ON_unsupported_owned_record");
    let mut packets = Vec::new();
    for import_mode in [None, Some(ImportMode::Preserve), Some(ImportMode::Geometry)] {
        let ns = ImportNamespace {
            id: ImportNamespaceId(Uuid::from_u128(3)),
            source: SourceId(Uuid::from_u128(4)),
            import_mode,
            records: vec![record.clone()],
        };
        let packet = serde_json::to_vec(&ns).unwrap();
        let cold: ImportNamespace = serde_json::from_slice(&packet).unwrap();
        assert_eq!(cold.import_mode, import_mode);
        assert_eq!(cold.records[0], record);
        assert_eq!(cold.records[0].disposition, RecordDisposition::RetainedOnly);
        assert!(cold.records[0].host.is_none());
        packets.push(packet);
    }
    assert_ne!(packets[0], packets[1]);
    assert_ne!(packets[0], packets[2]);
    assert_ne!(packets[1], packets[2]);
}
#[test]
fn legacy_bound_helper_never_invents_per_field_baselines() {
    let e = Entity::new("legacy", Geometry::Point([1.; 3]), None);
    let record = SourceRecord::bound(
        key(),
        "ON_Point",
        HostRef::Object(e.id),
        fingerprint(&e).unwrap(),
    );
    assert!(record.entity_baseline.is_none());
    let cold: SourceRecord = serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
    assert!(cold.entity_baseline.is_none());
    assert_eq!(cold.baseline_sha256, record.baseline_sha256);
}
fn key() -> NativeRecordKey {
    NativeRecordKey {
        kind: NativeRecordKind::Geometry,
        id: Uuid::new_v4(),
        native_index: Some(0),
    }
}
fn fixture() -> (Document, ObjectId, ImportNamespaceId) {
    let mut d = Document::new("baseline ownership");
    let e = Entity::new("original", Geometry::Point([1., 2., 3.]), None);
    let object = e.id;
    let record = SourceRecord::bound_entity(key(), "ON_Point", HostRef::Object(e.id), &e).unwrap();
    let resource = Resource::from_bytes(
        "source.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored structural bytes; not native merge evidence",
        &Limits::default(),
    )
    .unwrap();
    let unit = NativeUnits {
        native_enum: 2,
        name: "Millimeters".into(),
        declared_mm_per_unit: Some(1.),
        millimeters_per_unit: 1.,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.01,
        angular_tolerance_radians: 0.01,
        relative_tolerance: 0.01,
    };
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 50,
        units: SourceUnits {
            model: unit.clone(),
            page: unit,
            definitions: Default::default(),
        },
    };
    let ns = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: Some(ImportMode::Preserve),
        records: vec![record],
    };
    let namespace = ns.id;
    d.entities.insert(e.id, e);
    d.resources.insert(resource.id, resource);
    d.retained_sources.insert(source.id, source);
    d.import_namespaces.insert(ns.id, ns);
    (d, object, namespace)
}
fn hashes(b: EntityBaseline) -> [[u8; 32]; 4] {
    [
        b.geometry_sha256,
        b.placement_sha256,
        b.attributes_sha256,
        b.label_sha256,
    ]
}
#[test]
fn field_fingerprints_ignore_entity_identity_and_revision_counters() {
    let mut e = Entity::new("original", Geometry::Point([1., 2., 3.]), None);
    let fields = EntityBaseline::capture(&e).unwrap();
    let full = fingerprint(&e).unwrap();
    e.id = ObjectId::new();
    e.generation += 1;
    e.geometry_revision += 3;
    e.attributes_revision += 7;
    assert_eq!(EntityBaseline::capture(&e).unwrap(), fields);
    assert_ne!(fingerprint(&e).unwrap(), full);
}
#[test]
fn geometry_placement_attributes_and_label_edits_have_independent_hashes() {
    let original = Entity::new("original", Geometry::Point([1., 2., 3.]), None);
    let baseline = hashes(EntityBaseline::capture(&original).unwrap());
    for field in 0..4 {
        let mut edited = original.clone();
        match field {
            0 => edited.geometry = Geometry::Point([4., 5., 6.]),
            1 => edited.placement = Transform::translation([7., 8., 9.]),
            2 => edited.attributes.visible = false,
            3 => edited.label = "edited nhãn".into(),
            _ => unreachable!(),
        }
        let current = hashes(EntityBaseline::capture(&edited).unwrap());
        for i in 0..4 {
            assert_eq!(
                baseline[i] != current[i],
                i == field,
                "field {field} also altered hash {i}"
            );
        }
    }
}
#[test]
fn entity_baseline_helper_rejects_wrong_kind_host_and_native_identity_alias() {
    let e = Entity::new("original", Geometry::Point([1.; 3]), None);
    let mut wrong_kind = key();
    wrong_kind.kind = NativeRecordKind::Layer;
    let mut alias = key();
    alias.id = e.id.0;
    for (key, host) in [
        (wrong_kind, HostRef::Object(e.id)),
        (key(), HostRef::Object(ObjectId::new())),
        (key(), HostRef::Layer(LayerId::new())),
        (
            key(),
            HostRef::DefinitionMember {
                definition: DefinitionId(Uuid::nil()),
                object: e.id,
            },
        ),
        (alias, HostRef::Object(e.id)),
    ] {
        assert!(matches!(
            SourceRecord::bound_entity(key, "ON_Point", host, &e),
            Err(Error::Invalid(_))
        ));
    }
}
#[test]
fn field_baseline_is_invalid_on_retained_only_or_layer_records() {
    let (mut d, object, ns) = fixture();
    let original = d.import_namespaces[&ns].records[0].entity_baseline;
    let record = &mut d.import_namespaces.get_mut(&ns).unwrap().records[0];
    record.host = None;
    record.baseline_sha256 = None;
    record.disposition = RecordDisposition::RetainedOnly;
    assert!(matches!(
        d.validate(&Limits::default()),
        Err(Error::Invalid(_))
    ));
    let layer = Layer::new("native layer", [1, 2, 3, 255]);
    let mut layer_key = key();
    layer_key.kind = NativeRecordKind::Layer;
    let mut record = SourceRecord::bound(
        layer_key,
        "ON_Layer",
        HostRef::Layer(layer.id),
        fingerprint(&layer).unwrap(),
    );
    record.entity_baseline = original;
    d.layer_order.push(layer.id);
    d.layers.insert(layer.id, layer);
    d.import_namespaces.get_mut(&ns).unwrap().records[0] = record;
    assert!(d.entities.contains_key(&object));
    assert!(matches!(
        d.validate(&Limits::default()),
        Err(Error::Invalid(_))
    ));
}
#[test]
fn unchanged_record_cannot_claim_forged_independent_baselines() {
    let (mut d, _, ns) = fixture();
    d.import_namespaces.get_mut(&ns).unwrap().records[0]
        .entity_baseline
        .as_mut()
        .unwrap()
        .geometry_sha256[0] ^= 1;
    assert!(matches!(
        d.validate(&Limits::default()),
        Err(Error::Invalid(_))
    ));
}
#[test]
fn field_baselines_remain_original_across_edit_and_delete_dispositions() {
    let (mut d, object, ns) = fixture();
    let baseline = d.import_namespaces[&ns].records[0].entity_baseline;
    d.entities.get_mut(&object).unwrap().geometry = Geometry::Point([5.; 3]);
    d.entities.get_mut(&object).unwrap().geometry_revision += 1;
    d.import_namespaces.get_mut(&ns).unwrap().records[0].disposition = RecordDisposition::Edited;
    d.validate(&Limits::default()).unwrap();
    assert_eq!(
        d.import_namespaces[&ns].records[0].entity_baseline,
        baseline
    );
    d.entities.remove(&object);
    d.import_namespaces.get_mut(&ns).unwrap().records[0].disposition = RecordDisposition::Deleted;
    d.validate(&Limits::default()).unwrap();
    let cold: ImportNamespace =
        serde_json::from_slice(&serde_json::to_vec(&d.import_namespaces[&ns]).unwrap()).unwrap();
    assert_eq!(cold.records[0].entity_baseline, baseline);
    assert_eq!(cold.records[0].disposition, RecordDisposition::Deleted);
}
#[test]
fn spare_record_capacity_charges_inline_field_baselines_and_rejects_tight_limits() {
    let (mut d, _, ns) = fixture();
    let before = d.validate(&Limits::default()).unwrap();
    let records = &mut d.import_namespaces.get_mut(&ns).unwrap().records;
    let old_capacity = records.capacity();
    records.reserve_exact(64);
    let extra = (records.capacity() - old_capacity) * std::mem::size_of::<SourceRecord>();
    let after = d.validate(&Limits::default()).unwrap();
    assert_eq!(after.bytes - before.bytes, extra as u64);
    let l = Limits {
        max_bytes: before.bytes + extra as u64 - 1,
        ..Limits::default()
    };
    assert!(matches!(d.validate(&l), Err(Error::Budget(_))));
}
#[test]
fn root_and_definition_member_helpers_capture_full_entity_and_independent_fields() {
    let e = Entity::new("member", Geometry::Point([1.; 3]), None);
    for host in [
        HostRef::Object(e.id),
        HostRef::DefinitionMember {
            definition: DefinitionId::new(),
            object: e.id,
        },
    ] {
        let r = SourceRecord::bound_entity(key(), "ON_Point", host, &e).unwrap();
        assert_eq!(r.baseline_sha256, Some(fingerprint(&e).unwrap()));
        assert_eq!(
            r.entity_baseline.unwrap().geometry_sha256,
            fingerprint(&e.geometry).unwrap()
        );
        assert_eq!(
            r.entity_baseline.unwrap().placement_sha256,
            fingerprint(&e.placement).unwrap()
        );
        assert_eq!(
            r.entity_baseline.unwrap().attributes_sha256,
            fingerprint(&e.attributes).unwrap()
        );
        assert_eq!(
            r.entity_baseline.unwrap().label_sha256,
            fingerprint(&e.label).unwrap()
        );
    }
}
