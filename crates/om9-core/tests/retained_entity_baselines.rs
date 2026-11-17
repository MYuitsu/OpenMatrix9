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

fn brep_fixture(source: &str) -> om9_core::brep::Brep {
    use om9_core::brep::{Brep, BrepDisplay, BrepTopology};
    Brep::from_native(source.into(), BrepDisplay { source_sha256: [0; 32], deflection_mm: 0.1,
        points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], triangles: vec![[0, 1, 2, 0]],
        segments: vec![], vertices: vec![], topology: BrepTopology { faces: vec![Uuid::from_u128(20)],
            edges: vec![], vertex_ids: vec![], face_edges: vec![] }.into() }, &Limits::default()).unwrap()
}

#[test]
fn source_baseline_ignores_display_only_remesh_and_detects_source_change() {
    let (mut document, object, namespace) = fixture();
    let entity = document.entities.get_mut(&object).unwrap();
    entity.geometry = Geometry::Brep(brep_fixture("CASCADE Topology source A"));
    let record = SourceRecord::bound_entity(key(), "ON_Brep", HostRef::Object(object), entity).unwrap();
    assert_eq!(record.entity_baseline.unwrap().geometry_hash_version, 1);
    document.import_namespaces.get_mut(&namespace).unwrap().records[0] = record;
    document.validate(&Limits::default()).unwrap();
    let Geometry::Brep(shape) = &mut document.entities.get_mut(&object).unwrap().geometry else { panic!() };
    let mut display = shape.display().clone();
    display.deflection_mm = 0.02;
    display.points[1][0] = 2.;
    *shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &Limits::default()).unwrap();
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    assert_eq!(document.import_namespaces[&namespace].records[0].disposition, RecordDisposition::Unchanged);
    document.entities.get_mut(&object).unwrap().geometry = Geometry::Brep(brep_fixture("CASCADE Topology source B"));
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    assert_eq!(document.import_namespaces[&namespace].records[0].disposition, RecordDisposition::Edited);
}

#[test]
fn legacy_brep_baseline_keeps_original_display_sensitive_interpretation() {
    let (mut document, object, namespace) = fixture();
    let entity = document.entities.get_mut(&object).unwrap();
    entity.geometry = Geometry::Brep(brep_fixture("CASCADE Topology source A"));
    let mut record = SourceRecord::bound(key(), "ON_Brep", HostRef::Object(object), fingerprint(entity).unwrap());
    record.entity_baseline = Some(EntityBaseline::capture_for_version(entity, 0).unwrap());
    let encoded = serde_json::to_value(&record).unwrap();
    assert!(encoded["entity_baseline"].get("geometry_hash_version").is_none());
    document.import_namespaces.get_mut(&namespace).unwrap().records[0] = serde_json::from_value(encoded).unwrap();
    document.validate(&Limits::default()).unwrap();
    let Geometry::Brep(shape) = &mut document.entities.get_mut(&object).unwrap().geometry else { panic!() };
    let mut display = shape.display().clone();
    display.points[1][0] = 2.;
    *shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &Limits::default()).unwrap();
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    assert_eq!(document.import_namespaces[&namespace].records[0].disposition, RecordDisposition::Edited);
}

#[test]
fn native_brep_baseline_ignores_separate_display_but_keeps_payload() {
    use om9_core::geometry::{Mesh, NativeBrep};
    let (mut document, object, namespace) = fixture();
    let entity = document.entities.get_mut(&object).unwrap();
    let display = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    entity.geometry = Geometry::NativeBrep(NativeBrep { display: display.into(), payload: vec![1, 2, 3].into(), crc: 12 });
    let record = SourceRecord::bound_entity(key(), "ON_Brep", HostRef::Object(object), entity).unwrap();
    let bytes = serde_json::to_vec(&record).unwrap();
    let record: SourceRecord = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record.entity_baseline.unwrap().geometry_hash_version, 1);
    document.import_namespaces.get_mut(&namespace).unwrap().records[0] = record;
    document.validate(&Limits::default()).unwrap();
    let Geometry::NativeBrep(shape) = &mut document.entities.get_mut(&object).unwrap().geometry else { panic!() };
    std::sync::Arc::make_mut(&mut shape.display).vertices[1][0] = 2.;
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    assert_eq!(document.import_namespaces[&namespace].records[0].disposition, RecordDisposition::Unchanged);
    let Geometry::NativeBrep(shape) = &mut document.entities.get_mut(&object).unwrap().geometry else { panic!() };
    std::sync::Arc::make_mut(&mut shape.payload)[0] = 9;
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    assert_eq!(document.import_namespaces[&namespace].records[0].disposition, RecordDisposition::Edited);
}

#[test]
fn unknown_entity_baseline_hash_version_is_refused() {
    let (mut document, _, namespace) = fixture();
    document.import_namespaces.get_mut(&namespace).unwrap().records[0].entity_baseline.as_mut().unwrap().geometry_hash_version = 2;
    assert!(matches!(document.validate(&Limits::default()), Err(Error::Unsupported("entity baseline hash version"))));
}

fn definition_fixture(legacy: bool) -> (Document, ImportNamespaceId, DefinitionId) {
    use om9_core::model::Definition;
    let (mut document, _, namespace) = fixture();
    let member = Entity::new("brep member", Geometry::Brep(brep_fixture("CASCADE Topology source A")), None);
    let definition = Definition::new("block", vec![member]);
    let definition_id = definition.id;
    let mut definition_key = key();
    definition_key.kind = NativeRecordKind::Definition;
    let def_record = if legacy {
        SourceRecord::bound(definition_key, "ON_InstanceDefinition", HostRef::Definition(definition_id), fingerprint(&definition).unwrap())
    } else {
        SourceRecord::bound_definition(definition_key, "ON_InstanceDefinition", &definition).unwrap()
    };
    let mut member_key = key();
    member_key.native_index = Some(1);
    let member_record = SourceRecord::bound_entity(member_key, "ON_Brep", HostRef::DefinitionMember {
        definition: definition_id, object: definition.members[0].id,
    }, &definition.members[0]).unwrap();
    document.import_namespaces.get_mut(&namespace).unwrap().records.extend([def_record, member_record]);
    document.definitions.insert(definition_id, definition);
    document.validate(&Limits::default()).unwrap();
    (document, namespace, definition_id)
}

#[test]
fn definition_source_baseline_survives_member_remesh_and_detects_source_edits() {
    let (document, namespace, definition) = definition_fixture(false);
    assert_eq!(document.import_namespaces[&namespace].records[1].definition_hash_version, 1);
    let mut document = document;
    let cold: ImportNamespace = serde_json::from_slice(&serde_json::to_vec(&document.import_namespaces[&namespace]).unwrap()).unwrap();
    document.import_namespaces.insert(namespace, cold);
    let member = &mut document.definitions.get_mut(&definition).unwrap().members[0];
    let Geometry::Brep(shape) = &mut member.geometry else { panic!() };
    let mut display = shape.display().clone();
    display.deflection_mm = 0.02;
    display.points[1][0] = 2.;
    *shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &Limits::default()).unwrap();
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    let records = &document.import_namespaces[&namespace].records;
    assert_eq!((records[1].disposition, records[2].disposition), (RecordDisposition::Unchanged, RecordDisposition::Unchanged));
    document.definitions.get_mut(&definition).unwrap().members[0].geometry = Geometry::Brep(brep_fixture("CASCADE Topology source B"));
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    let records = &document.import_namespaces[&namespace].records;
    assert_eq!((records[1].disposition, records[2].disposition), (RecordDisposition::Edited, RecordDisposition::Edited));
}

#[test]
fn legacy_definition_keeps_display_sensitive_baseline() {
    let (mut document, namespace, definition) = definition_fixture(true);
    let record = &document.import_namespaces[&namespace].records[1];
    assert_eq!(record.definition_hash_version, 0);
    assert!(serde_json::to_value(record).unwrap().get("definition_hash_version").is_none());
    let Geometry::Brep(shape) = &mut document.definitions.get_mut(&definition).unwrap().members[0].geometry else { panic!() };
    let mut display = shape.display().clone();
    display.points[1][0] = 2.;
    *shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &Limits::default()).unwrap();
    document.reconcile_derived_export_bindings(&Limits::default()).unwrap();
    let records = &document.import_namespaces[&namespace].records;
    assert_eq!((records[1].disposition, records[2].disposition), (RecordDisposition::Edited, RecordDisposition::Unchanged));
}

#[test]
fn definition_source_baseline_preserves_member_order_placement_and_metadata() {
    use om9_core::model::Definition;
    let member = Entity::new("brep", Geometry::Brep(brep_fixture("CASCADE Topology source A")), None);
    let mut definition = Definition::new("block", vec![member, Entity::new("point", Geometry::Point([1., 2., 3.]), None)]);
    let mut native_key = key(); native_key.kind = NativeRecordKind::Definition;
    let hash = |definition: &Definition| SourceRecord::bound_definition(native_key, "ON_InstanceDefinition", definition).unwrap().baseline_sha256;
    let original = hash(&definition);
    definition.members.swap(0, 1);
    assert_ne!(original, hash(&definition));
    definition.members.swap(0, 1);
    definition.members[0].placement = Transform::translation([1., 0., 0.]);
    assert_ne!(original, hash(&definition));
    definition.members[0].placement = Transform::identity();
    definition.label.push_str(" changed");
    assert_ne!(original, hash(&definition));
    definition.label = "block".into();
    let authored = om9_core::geometry::Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    definition.members[1].geometry = Geometry::Mesh(authored);
    let mesh_baseline = hash(&definition);
    let Geometry::Mesh(mesh) = &mut definition.members[1].geometry else { panic!() };
    mesh.vertices[1][0] = 2.;
    assert_ne!(mesh_baseline, hash(&definition));
}

#[test]
fn unknown_definition_baseline_version_is_refused() {
    let (mut document, namespace, _) = definition_fixture(false);
    document.import_namespaces.get_mut(&namespace).unwrap().records[1].definition_hash_version = 2;
    assert!(matches!(document.validate(&Limits::default()), Err(Error::Unsupported("definition baseline hash version"))));
    let (mut document, _, namespace) = fixture();
    document.import_namespaces.get_mut(&namespace).unwrap().records[0].definition_hash_version = 1;
    assert!(matches!(document.validate(&Limits::default()), Err(Error::Invalid("definition baseline binding kind"))));
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
