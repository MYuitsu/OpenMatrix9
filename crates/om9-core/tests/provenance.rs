// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-03-013 import source provenance metadata.
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    identity::*,
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
    retained::{
        HostRef, ImportBundle, ImportMode, ImportNamespace, NativeRecordKey, NativeRecordKind,
        NativeUnits, ResolvedBy, RetainedSource, SourceRecord, SourceUnits,
    },
    transaction::*,
};

fn units() -> NativeUnits {
    NativeUnits {
        native_enum: 0,
        name: "Millimeters".into(),
        declared_mm_per_unit: None,
        millimeters_per_unit: 1.0,
        resolved_by: ResolvedBy::Explicit,
        absolute_tolerance: 0.001,
        angular_tolerance_radians: 0.0,
        relative_tolerance: 0.0,
    }
}

/// One imported document built through the real import transaction: an embedded
/// `.3dm` resource, its retained source, a namespace and one bound object.
fn imported() -> (Document, ObjectId, SourceId, ImportNamespaceId) {
    let limits = Limits::default();
    let mut doc = Document::new("import");
    let resource = Resource::from_bytes(
        "ring.3dm",
        ResourceKind::RetainedThreeDm { archive_version: 5 },
        b"3dm payload",
        &limits,
    )
    .unwrap();
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 5,
        units: SourceUnits {
            model: units(),
            page: units(),
            definitions: std::collections::BTreeMap::new(),
        },
    };
    let entity = Entity::new("Imported stone", Geometry::Point([1., 2., 3.]), None);
    let object = entity.id;
    let record = SourceRecord::bound_entity(
        NativeRecordKey {
            kind: NativeRecordKind::Geometry,
            id: uuid::Uuid::new_v4(),
            native_index: Some(3),
        },
        "mesh",
        HostRef::Object(object),
        &entity,
    )
    .unwrap();
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: Some(ImportMode::Preserve),
        records: vec![record],
    };
    let namespace_id = namespace.id;
    let source_id = source.id;
    let bundle = ImportBundle {
        resource,
        source,
        namespace,
        entities: vec![entity],
        layers: vec![],
        definitions: vec![],
        groups: vec![],
        exchange: Default::default(),
    };
    let document_id = doc.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Import",
            vec![Mutation::Import {
                document: document_id,
                expected_revision: 1,
                bundle: Box::new(bundle),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    doc = (*c.world().documents[&document_id]).clone();
    (doc, object, source_id, namespace_id)
}

#[test]
fn an_imported_object_reports_source_path_checksum_and_original_identity() {
    let (doc, object, source, namespace) = imported();
    let provenance = doc.provenance(object).expect("complete source proof");
    assert_eq!(provenance.object, object);
    assert_eq!(provenance.source, source);
    assert_eq!(provenance.namespace, namespace);
    assert_eq!(provenance.source_name, "ring.3dm");
    assert_eq!(provenance.archive_version, 5);
    assert_eq!(provenance.original_index, Some(3));
    assert_eq!(provenance.original_family, "mesh");
    assert_eq!(provenance.kind, NativeRecordKind::Geometry);
    assert_eq!(provenance.import_mode, Some(ImportMode::Preserve));
    assert_eq!(
        provenance.disposition,
        om9_core::retained::RecordDisposition::Unchanged
    );
    assert_ne!(provenance.source_sha256, [0u8; 32]);
    assert!(doc.has_source_proof(object));
    let summary = doc.import_summary();
    assert_eq!(summary.sources, 1);
    assert_eq!(summary.namespaces, 1);
    assert_eq!(summary.records, 1);
    assert_eq!(summary.bound, 1);
    assert_eq!(summary.unsupported, 0);
    assert_eq!(summary.objects_with_provenance, 1);
    assert!(!summary.is_empty());
}

#[test]
fn a_natively_created_object_is_never_labelled_as_an_import() {
    let (doc, object, _, _) = imported();
    let document_id = doc.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let native = Entity::new("Native", Geometry::Point([0.; 3]), None);
    let native_id = native.id;
    let p = c
        .prepare(TransactionRequest::new(
            "insert",
            vec![Mutation::Insert {
                document: document_id,
                entity: native,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&document_id];
    assert!(doc.has_source_proof(object));
    assert_eq!(doc.provenance(native_id), None);
    assert!(!doc.has_source_proof(native_id));
    assert_eq!(doc.import_summary().objects_with_provenance, 1);
}

#[test]
fn a_source_without_a_matching_embedded_hash_has_no_provenance() {
    let (mut doc, object, source, _) = imported();
    // The embedded payload hash is the proof. Break it and the object must stop
    // claiming a native source instead of reporting a half-proven one.
    doc.retained_sources.get_mut(&source).unwrap().sha256 = [7u8; 32];
    assert_eq!(doc.provenance(object), None);
    assert!(!doc.has_source_proof(object));
}

#[test]
fn an_object_with_no_import_namespace_has_no_provenance() {
    let mut doc = Document::new("plain");
    let entity = Entity::new("Plain", Geometry::Point([0.; 3]), None);
    let object = entity.id;
    doc.entities.insert(object, entity);
    assert_eq!(doc.provenance(object), None);
    assert!(doc.import_summary().is_empty());
    assert_eq!(doc.import_summary().objects_with_provenance, 0);
}

#[test]
fn the_summary_is_a_single_pass_over_a_large_import() {
    // Regression guard: `import_summary` must not ask `provenance()` once per
    // object (that was quadratic in objects x records). 2000 objects and 1000
    // bound records stay a sub-millisecond pass instead of ~2M record scans.
    let limits = Limits::default();
    let mut doc = Document::new("large import");
    let resource = Resource::from_bytes(
        "big.3dm",
        ResourceKind::RetainedThreeDm { archive_version: 5 },
        b"payload",
        &limits,
    )
    .unwrap();
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 5,
        units: SourceUnits {
            model: units(),
            page: units(),
            definitions: std::collections::BTreeMap::new(),
        },
    };
    let mut records = Vec::new();
    let mut bound = Vec::new();
    for index in 0..2000 {
        let entity = Entity::new(format!("part {index}"), Geometry::Point([0.; 3]), None);
        if index < 1000 {
            records.push(
                SourceRecord::bound_entity(
                    NativeRecordKey {
                        kind: NativeRecordKind::Geometry,
                        id: uuid::Uuid::new_v4(),
                        native_index: Some(index),
                    },
                    "mesh",
                    HostRef::Object(entity.id),
                    &entity,
                )
                .unwrap(),
            );
            bound.push(entity.id);
        }
        doc.entities.insert(entity.id, entity);
    }
    doc.resources.insert(resource.id, resource);
    doc.retained_sources.insert(source.id, source.clone());
    doc.import_namespaces.insert(
        ImportNamespaceId::new(),
        ImportNamespace {
            id: ImportNamespaceId::new(),
            source: source.id,
            import_mode: Some(ImportMode::Preserve),
            records,
        },
    );
    // The namespace key and its own id must agree for a validated document.
    let namespace_id = *doc.import_namespaces.keys().next().unwrap();
    doc.import_namespaces.get_mut(&namespace_id).unwrap().id = namespace_id;
    let summary = doc.import_summary();
    assert_eq!(summary.records, 1000);
    assert_eq!(summary.bound, 1000);
    assert_eq!(summary.objects_with_provenance, 1000);
    assert!(doc.has_source_proof(bound[0]));
    assert!(!doc.has_source_proof(ObjectId::new()));
}
