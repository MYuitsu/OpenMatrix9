// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
    retained::{
        HostRef, ImportBundle, ImportNamespace, NativeRecordKey, NativeRecordKind, NativeUnits,
        RecordDisposition, ResolvedBy, RetainedSource, SourceRecord, SourceUnits, fingerprint,
    },
    transaction::*,
};

fn units(factor: f64) -> SourceUnits {
    let u = NativeUnits {
        native_enum: 8,
        name: "Inches".into(),
        declared_mm_per_unit: Some(factor),
        millimeters_per_unit: factor,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.001,
        angular_tolerance_radians: 0.001,
        relative_tolerance: 0.,
    };
    SourceUnits {
        model: u.clone(),
        page: u,
        definitions: Default::default(),
    }
}
fn bundle() -> ImportBundle {
    let resource = Resource::from_bytes(
        "source.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored ownership fixture, native parsing is separately tested",
        &Limits::default(),
    )
    .unwrap();
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 50,
        units: units(25.4),
    };
    let entity = Entity::new("imported A", Geometry::Point([25.4, 0., 0.]), None);
    let key = NativeRecordKey {
        kind: NativeRecordKind::Geometry,
        id: uuid::Uuid::new_v4(),
        native_index: Some(0),
    };
    let record = SourceRecord::bound(
        key,
        "ON_Point",
        HostRef::Object(entity.id),
        fingerprint(&entity).unwrap(),
    );
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: None,
        records: vec![record],
    };
    ImportBundle {
        resource,
        source,
        namespace,
        entities: vec![entity],
        layers: vec![],
        definitions: vec![],
        groups: vec![],
        exchange: Default::default(),
    }
}
fn fixture() -> (TransactionCoordinator, DocumentId, DocumentId) {
    let d = Document::new("target");
    let id = d.id;
    let other = Document::new("other");
    let oid = other.id;
    (
        TransactionCoordinator::new(
            World::new(vec![d, other], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        id,
        oid,
    )
}
fn import(c: &mut TransactionCoordinator, d: DocumentId, bundle: ImportBundle) {
    let revision = c.world().documents[&d].revision;
    let p = c
        .prepare(TransactionRequest::new(
            "Import",
            vec![Mutation::Import {
                document: d,
                expected_revision: revision,
                bundle: Box::new(bundle),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
}

#[test]
fn layers_assignment_refuses_retained_root_without_erasing_namespace_or_redo() {
    use om9_core::{
        layers::{LayerAction, LayerPlan},
        model::Layer,
    };
    let (mut c, did, _) = fixture();
    let imported = bundle();
    let eid = imported.entities[0].id;
    import(&mut c, did, imported);
    let target = Layer::new("plain target", [1, 2, 3, 255]);
    let lid = target.id;
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer: target,
            make_current: false,
        },
        None,
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "new target",
            vec![Mutation::Layers(Box::new(plan))],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let source = c.world().documents[&did].stamp(eid).unwrap();
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Assign {
            layer: lid,
            sources: vec![source],
        },
        None,
    )
    .unwrap();
    let before = c.snapshot();
    assert!(matches!(
        c.prepare(TransactionRequest::new(
            "unsafe retained assign",
            vec![Mutation::Layers(Box::new(plan))]
        )),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(c.snapshot(), before);
    c.undo().unwrap();
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&did].import_namespaces,
        before.world.documents[&did].import_namespaces
    );
    assert_eq!(
        c.world().documents[&did].entities[&eid],
        before.world.documents[&did].entities[&eid]
    );
}

#[test]
fn two_imports_of_identical_source_use_separate_host_and_retained_namespaces() {
    let (mut c, d, _) = fixture();
    let first = bundle();
    let mut second = bundle();
    second.namespace.records[0].key = first.namespace.records[0].key;
    let a = first.entities[0].id;
    let b = second.entities[0].id;
    let source_a = first.source.id;
    let source_b = second.source.id;
    let ns_a = first.namespace.id;
    let ns_b = second.namespace.id;
    import(&mut c, d, first);
    import(&mut c, d, second);
    let doc = &c.world().documents[&d];
    assert_ne!(a, b);
    assert_ne!(ns_a, ns_b);
    assert_ne!(source_a, source_b);
    assert_eq!(doc.import_namespaces.len(), 2);
    assert_eq!(doc.retained_sources.len(), 2);
    assert_eq!(doc.entities.len(), 2);
    assert_eq!(c.undo_len(), 2);
    assert_eq!(
        doc.import_namespaces[&ns_a].records[0].key,
        doc.import_namespaces[&ns_b].records[0].key
    );
    c.undo().unwrap();
    assert!(c.world().documents[&d].entities.contains_key(&a));
    assert!(!c.world().documents[&d].entities.contains_key(&b));
    c.redo().unwrap();
    assert!(c.world().documents[&d].entities.contains_key(&b));
}

#[test]
fn edited_deleted_and_new_records_remain_distinct_and_undo_restores_original_binding() {
    let (mut c, d, _) = fixture();
    let bundle = bundle();
    let id = bundle.entities[0].id;
    let ns = bundle.namespace.id;
    import(&mut c, d, bundle);
    let before = c.world().clone();
    let p = c
        .prepare(TransactionRequest::new(
            "Edit A",
            vec![Mutation::Geometry {
                source: c.world().documents[&d].stamp(id).unwrap(),
                geometry: Geometry::Point([50.8, 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(
        c.world().documents[&d].import_namespaces[&ns].records[0].disposition,
        RecordDisposition::Edited
    );
    let p = c
        .prepare(TransactionRequest::new(
            "Delete A, add C",
            vec![
                Mutation::Delete {
                    source: c.world().documents[&d].stamp(id).unwrap(),
                },
                Mutation::Insert {
                    document: d,
                    entity: Entity::new("new C", Geometry::Point([0., 1., 0.]), None),
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let deleted = c.world().clone();
    assert_eq!(
        deleted.documents[&d].import_namespaces[&ns].records[0].disposition,
        RecordDisposition::Deleted
    );
    assert_eq!(
        deleted.documents[&d].import_namespaces[&ns].records[0].host,
        Some(HostRef::Object(id))
    );
    assert!(!deleted.documents[&d].entities.contains_key(&id));
    c.undo().unwrap();
    c.undo().unwrap();
    assert_eq!(c.world(), &before);
    c.redo().unwrap();
    c.redo().unwrap();
    assert_eq!(c.world(), &deleted);
}

#[test]
fn invalid_import_has_no_partial_source_namespace_objects_or_undo() {
    for failure in [
        "hash",
        "version",
        "unitless",
        "scale",
        "tolerance underflow",
        "missing relation",
        "duplicate native",
        "host baseline",
        "missing host",
    ] {
        let (c, d, _) = fixture();
        let mut bundle = bundle();
        match failure {
            "hash" => bundle.source.sha256 = [0; 32],
            "version" => bundle.source.archive_version = 60,
            "unitless" => bundle.source.units.model.declared_mm_per_unit = None,
            "scale" => bundle.source.units.model.millimeters_per_unit = f64::INFINITY,
            "tolerance underflow" => {
                bundle.source.units.model.millimeters_per_unit = 1e-300;
                bundle.source.units.model.declared_mm_per_unit = Some(1e-300);
                bundle.source.units.model.absolute_tolerance = 1e-300;
            }
            "missing relation" => {
                bundle.namespace.records[0]
                    .relations
                    .push(om9_core::retained::NativeRelation {
                        role: "layer".into(),
                        target: NativeRecordKey {
                            kind: NativeRecordKind::Layer,
                            id: uuid::Uuid::new_v4(),
                            native_index: Some(2),
                        },
                    })
            }
            "duplicate native" => bundle
                .namespace
                .records
                .push(bundle.namespace.records[0].clone()),
            "host baseline" => bundle.namespace.records[0].baseline_sha256 = Some([0; 32]),
            "missing host" => bundle.entities.clear(),
            _ => unreachable!(),
        }
        let before = c.snapshot();
        assert!(
            c.prepare(TransactionRequest::new(
                "Invalid",
                vec![Mutation::Import {
                    document: d,
                    expected_revision: 1,
                    bundle: Box::new(bundle)
                }]
            ))
            .is_err(),
            "{failure}"
        );
        assert_eq!(c.snapshot(), before, "{failure}");
    }
}

#[test]
fn retained_only_native_record_has_no_fabricated_entity_and_referenced_resource_cannot_be_deleted()
{
    let (mut c, d, _) = fixture();
    let mut bundle = bundle();
    let resource = bundle.resource.id;
    bundle.namespace.records.push(SourceRecord::retained(
        NativeRecordKey {
            kind: NativeRecordKind::Geometry,
            id: uuid::Uuid::new_v4(),
            native_index: Some(1),
        },
        "ON_Brep",
    ));
    import(&mut c, d, bundle);
    assert_eq!(c.world().documents[&d].entities.len(), 1);
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Delete source bytes",
            vec![Mutation::ResourceDelete {
                document: d,
                expected_revision: c.world().documents[&d].revision,
                resource
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
}

#[test]
fn retained_transfer_is_rejected_before_changing_either_document() {
    let (mut c, d, other) = fixture();
    let bundle = bundle();
    let id = bundle.entities[0].id;
    import(&mut c, d, bundle);
    let before = c.snapshot();
    assert!(matches!(
        c.prepare(TransactionRequest::new(
            "Move retained A",
            vec![Mutation::Move {
                source: c.world().documents[&d].stamp(id).unwrap(),
                destination: other,
                layer: None
            }]
        )),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn source_and_namespace_id_reuse_after_import_undo_is_rejected_but_redo_restores_them() {
    let (mut c, d, _) = fixture();
    let original = bundle();
    let source = original.source.id;
    let namespace = original.namespace.id;
    import(&mut c, d, original);
    c.undo().unwrap();
    for reuse_source in [true, false] {
        let mut new = bundle();
        if reuse_source {
            new.source.id = source;
            new.namespace.source = source;
        } else {
            new.namespace.id = namespace;
        }
        let before = c.snapshot();
        assert!(
            c.prepare(TransactionRequest::new(
                "Reallocated namespace",
                vec![Mutation::Import {
                    document: d,
                    expected_revision: c.world().documents[&d].revision,
                    bundle: Box::new(new)
                }]
            ))
            .is_err()
        );
        assert_eq!(c.snapshot(), before);
    }
    c.redo().unwrap();
    assert!(
        c.world().documents[&d]
            .retained_sources
            .contains_key(&source)
    );
    assert!(
        c.world().documents[&d]
            .import_namespaces
            .contains_key(&namespace)
    );
}

#[test]
fn placement_attributes_and_prepared_replacement_all_update_the_retained_disposition() {
    use om9_core::result::{ConversionReport, PreparedOutput, PreparedResult};
    for edit in ["placement", "attributes", "publish"] {
        let (mut c, d, _) = fixture();
        let bundle = bundle();
        let id = bundle.entities[0].id;
        let ns = bundle.namespace.id;
        import(&mut c, d, bundle);
        let source = c.world().documents[&d].stamp(id).unwrap();
        let mutation = match edit {
            "placement" => Mutation::Placement {
                source,
                placement: Transform::translation([5., 0., 0.]),
            },
            "attributes" => {
                let mut attributes = c.world().documents[&d].entities[&id].attributes.clone();
                attributes.color = Some([11, 22, 33, 255]);
                Mutation::Attributes { source, attributes }
            }
            "publish" => {
                let mut entity = c.world().documents[&d].entities[&id].clone();
                entity.geometry = Geometry::Point([2., 3., 4.]);
                Mutation::Publish(Box::new(PreparedResult {
                    document: d,
                    operation: "replacement".into(),
                    sources: vec![],
                    recipe: None,
                    outputs: vec![PreparedOutput {
                        geometry_snapshot: entity.geometry.clone(),
                        geometry_result: entity.geometry.clone(),
                        source_stamps: vec![],
                        conversion: ConversionReport::identity(entity.geometry.kind()),
                        entity,
                        expected_existing: Some(source),
                    }],
                }))
            }
            _ => unreachable!(),
        };
        let before = c.world().clone();
        let p = c
            .prepare(TransactionRequest::new(edit, vec![mutation]))
            .unwrap();
        assert_eq!(c.world(), &before);
        c.commit(p).unwrap();
        assert_eq!(
            c.world().documents[&d].import_namespaces[&ns].records[0].disposition,
            RecordDisposition::Edited,
            "{edit}"
        );
        c.undo().unwrap();
        assert_eq!(c.world(), &before);
        c.redo().unwrap();
        assert_eq!(
            c.world().documents[&d].import_namespaces[&ns].records[0].disposition,
            RecordDisposition::Edited
        );
    }
}

#[test]
fn direct_cold_metadata_tampering_cannot_claim_unchanged_or_resurrect_deleted_source() {
    let (mut c, d, _) = fixture();
    let bundle = bundle();
    let id = bundle.entities[0].id;
    let ns = bundle.namespace.id;
    let original = bundle.entities[0].clone();
    import(&mut c, d, bundle);
    let mut corrupt = c.world().documents[&d].as_ref().clone();
    corrupt.entities.get_mut(&id).unwrap().geometry = Geometry::Point([999., 0., 0.]);
    assert!(corrupt.validate(&Limits::default()).is_err());
    let p = c
        .prepare(TransactionRequest::new(
            "Delete",
            vec![Mutation::Delete {
                source: c.world().documents[&d].stamp(id).unwrap(),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let mut corrupt = c.world().documents[&d].as_ref().clone();
    corrupt.entities.insert(id, original.clone());
    assert!(corrupt.validate(&Limits::default()).is_err());
    let mut relocated = c.world().documents[&d].as_ref().clone();
    let definition = om9_core::model::Definition::new("resurrected member", vec![original.clone()]);
    relocated.definitions.insert(definition.id, definition);
    assert!(
        relocated.validate(&Limits::default()).is_err(),
        "a tombstone ID cannot reappear as a definition member"
    );
    let tombstone = c.world().documents[&d].as_ref().clone();
    let mut other = Document::new("other owner");
    other.entities.insert(id, original);
    assert!(World::new(vec![tombstone, other], &Limits::default()).is_err());
    assert_eq!(
        c.world().documents[&d].import_namespaces[&ns].records[0].disposition,
        RecordDisposition::Deleted
    );
}

#[test]
fn definition_member_binding_remains_local_to_its_shared_definition() {
    use om9_core::model::{Definition, Layer};
    let (mut c, d, _) = fixture();
    let mut bundle = bundle();
    let layer = Layer::new("imported layer", [10, 20, 30, 255]);
    let layer_id = layer.id;
    let member = Entity::new("definition member", Geometry::Point([1., 2., 3.]), layer_id);
    let member_id = member.id;
    let definition = Definition::new("shared native block", vec![member]);
    let definition_id = definition.id;
    for index in 0..2 {
        let mut entity = Entity::new("instance", Geometry::Instance(definition_id), layer_id);
        entity.placement = Transform::translation([index as f64 * 10., 0., 0.]);
        bundle.namespace.records.push(SourceRecord::bound(
            NativeRecordKey {
                kind: NativeRecordKind::Geometry,
                id: uuid::Uuid::new_v4(),
                native_index: Some(index + 2),
            },
            "ON_InstanceRef",
            HostRef::Object(entity.id),
            fingerprint(&entity).unwrap(),
        ));
        bundle.entities.push(entity);
    }
    bundle.namespace.records.push(SourceRecord::bound(
        NativeRecordKey {
            kind: NativeRecordKind::Layer,
            id: uuid::Uuid::new_v4(),
            native_index: Some(0),
        },
        "ON_Layer",
        HostRef::Layer(layer_id),
        fingerprint(&layer).unwrap(),
    ));
    bundle.namespace.records.push(SourceRecord::bound(
        NativeRecordKey {
            kind: NativeRecordKind::Definition,
            id: uuid::Uuid::new_v4(),
            native_index: Some(0),
        },
        "ON_InstanceDefinition",
        HostRef::Definition(definition_id),
        fingerprint(&definition).unwrap(),
    ));
    bundle.namespace.records.push(SourceRecord::bound(
        NativeRecordKey {
            kind: NativeRecordKind::Geometry,
            id: uuid::Uuid::new_v4(),
            native_index: Some(1),
        },
        "ON_Point",
        HostRef::DefinitionMember {
            definition: definition_id,
            object: member_id,
        },
        fingerprint(&definition.members[0]).unwrap(),
    ));
    bundle.layers.push(layer);
    bundle.definitions.push(definition);
    let ns = bundle.namespace.id;
    import(&mut c, d, bundle);
    let doc = &c.world().documents[&d];
    assert_eq!(doc.definitions.len(), 1);
    assert_eq!(doc.entities.len(), 3);
    assert!(!doc.entities.contains_key(&member_id));
    assert!(
        doc.import_namespaces[&ns]
            .records
            .iter()
            .all(|r| r.disposition == RecordDisposition::Unchanged)
    );
    c.undo().unwrap();
    assert!(c.world().documents[&d].definitions.is_empty());
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&d].definitions[&definition_id].members[0].id,
        member_id
    );
}

#[test]
fn caller_rollback_preserves_retired_namespace_authority_and_prior_redo() {
    let (mut c, d, _) = fixture();
    let original = bundle();
    let source_id = original.source.id;
    let namespace_id = original.namespace.id;
    let token = c.book("Import scope", true).unwrap();
    c.begin(&token).unwrap();
    c.savepoint(&token).unwrap();
    let mut request = TransactionRequest::new(
        "Private import",
        vec![Mutation::Import {
            document: d,
            expected_revision: 1,
            bundle: Box::new(original),
        }],
    );
    request.owner = Some(token.clone());
    let p = c.prepare(request).unwrap();
    c.commit(p).unwrap();
    assert!(c.world().documents[&d].retained_sources.is_empty());
    assert!(
        c.editing_world(&token).unwrap().documents[&d]
            .retained_sources
            .contains_key(&source_id)
    );
    c.rollback_savepoint(&token).unwrap();
    c.abort(&token).unwrap();
    let mut new = bundle();
    new.source.id = source_id;
    new.namespace.source = source_id;
    new.namespace.id = namespace_id;
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Reuse aborted namespace",
            vec![Mutation::Import {
                document: d,
                expected_revision: 1,
                bundle: Box::new(new)
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
}

#[test]
fn retained_validation_admits_record_scratch_with_all_owned_payload_and_document_storage() {
    let mut bundle = bundle();
    bundle.resource = Resource::from_bytes(
        "large original",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        &vec![9; 2 * 1024 * 1024],
        &Limits::default(),
    )
    .unwrap();
    bundle.source.resource = bundle.resource.id;
    bundle.source.sha256 = bundle.resource.sha256();
    bundle.entities.clear();
    bundle.namespace.records = (0..2048)
        .map(|index| {
            SourceRecord::retained(
                NativeRecordKey {
                    kind: NativeRecordKind::Geometry,
                    id: uuid::Uuid::new_v4(),
                    native_index: Some(index),
                },
                "ON_Brep",
            )
        })
        .collect();
    let mut d = Document::new("validation peak");
    d.resources.insert(bundle.resource.id, bundle.resource);
    d.retained_sources.insert(bundle.source.id, bundle.source);
    d.import_namespaces
        .insert(bundle.namespace.id, bundle.namespace);
    let owned = d.validate(&Limits::default()).unwrap().bytes;
    let limits = Limits {
        max_bytes: owned + 8192,
        ..Limits::default()
    };
    assert!(matches!(d.validate(&limits), Err(Error::Budget(_))));
}

#[test]
fn late_cancellation_stale_import_and_failed_acceptance_keep_prior_redo_and_unaccepted_ids() {
    for failure in ["late cancel", "stale", "guard"] {
        let (mut c, d, _) = fixture();
        let original = bundle();
        let ns = original.namespace.id;
        import(&mut c, d, original);
        c.undo().unwrap();
        let incoming = bundle();
        let source = incoming.source.id;
        let mut request = TransactionRequest::new(
            "Rejected new import",
            vec![Mutation::Import {
                document: d,
                expected_revision: if failure == "stale" { 999 } else { 1 },
                bundle: Box::new(incoming.clone()),
            }],
        );
        let before = c.snapshot();
        if failure == "stale" {
            assert!(matches!(c.prepare(request), Err(Error::Stale(_))));
        } else {
            let cancel = request.cancel.clone();
            let p = c.prepare(request).unwrap();
            if failure == "late cancel" {
                cancel.cancel();
                assert!(matches!(c.commit(p), Err(Error::Canceled)));
            } else {
                assert!(
                    c.commit_with_guard(p, |_| Err(Error::Invalid("reject import acceptance")))
                        .is_err()
                );
            }
        }
        assert_eq!(c.snapshot(), before);
        assert!(
            !c.world().documents[&d]
                .retained_sources
                .contains_key(&source)
        );
        request = TransactionRequest::new(
            "Same unaccepted identities",
            vec![Mutation::Import {
                document: d,
                expected_revision: 1,
                bundle: Box::new(incoming),
            }],
        );
        assert!(c.prepare(request).is_ok());
        c.redo().unwrap();
        assert!(c.world().documents[&d].import_namespaces.contains_key(&ns));
    }
}

#[test]
fn unresolved_units_require_an_explicit_owned_resolution_before_import() {
    let (mut c, d, _) = fixture();
    let mut bundle = bundle();
    for unit in [
        &mut bundle.source.units.model,
        &mut bundle.source.units.page,
    ] {
        unit.native_enum = 0;
        unit.name = "unitless authored source".into();
        unit.declared_mm_per_unit = None;
        unit.resolved_by = ResolvedBy::Explicit;
        unit.millimeters_per_unit = 2.54;
    }
    let source = bundle.source.id;
    import(&mut c, d, bundle);
    assert_eq!(
        c.world().documents[&d].retained_sources[&source]
            .units
            .model
            .millimeters_per_unit,
        2.54
    );
    let before = c.world().clone();
    let p = c
        .prepare(TransactionRequest::new(
            "Page units only",
            vec![Mutation::Context {
                document: d,
                expected_revision: c.world().documents[&d].context_revision,
                page_unit: om9_core::geometry::Unit::Inch,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(
        c.world().documents[&d].retained_sources[&source].units,
        before.documents[&d].retained_sources[&source].units
    );
}

#[test]
fn aggregate_object_count_rejects_import_before_candidate_allocation() {
    let limits = Limits {
        max_objects: 1,
        ..Limits::default()
    };
    let doc = Document::new("bounded import");
    let id = doc.id;
    let c = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let mut bundle = bundle();
    let e = Entity::new(
        "second imported entity",
        Geometry::Point([1., 2., 3.]),
        None,
    );
    bundle.namespace.records.push(SourceRecord::bound(
        NativeRecordKey {
            kind: NativeRecordKind::Geometry,
            id: uuid::Uuid::new_v4(),
            native_index: Some(1),
        },
        "ON_Point",
        HostRef::Object(e.id),
        fingerprint(&e).unwrap(),
    ));
    bundle.entities.push(e);
    let before = c.snapshot();
    assert!(matches!(
        c.prepare(TransactionRequest::new(
            "too many objects",
            vec![Mutation::Import {
                document: id,
                expected_revision: 1,
                bundle: Box::new(bundle)
            }]
        )),
        Err(Error::Budget("objects"))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn deleted_member_aliases_cannot_claim_two_retired_owners_in_a_cold_document() {
    let (mut c, d, _) = fixture();
    let bundle = bundle();
    let id = bundle.entities[0].id;
    let ns = bundle.namespace.id;
    import(&mut c, d, bundle);
    let p = c
        .prepare(TransactionRequest::new(
            "Delete",
            vec![Mutation::Delete {
                source: c.world().documents[&d].stamp(id).unwrap(),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let mut corrupt = c.world().documents[&d].as_ref().clone();
    let records = &mut corrupt.import_namespaces.get_mut(&ns).unwrap().records;
    records[0].host = Some(HostRef::DefinitionMember {
        definition: DefinitionId::new(),
        object: id,
    });
    let mut alias = records[0].clone();
    alias.key.id = uuid::Uuid::new_v4();
    alias.key.native_index = Some(1);
    alias.host = Some(HostRef::DefinitionMember {
        definition: DefinitionId::new(),
        object: id,
    });
    records.push(alias);
    assert!(corrupt.validate(&Limits::default()).is_err());
}

#[test]
fn import_requires_every_root_layer_definition_and_local_member_to_be_source_bound() {
    use om9_core::model::{Definition, Layer};
    for missing in [
        None,
        Some("root"),
        Some("layer"),
        Some("definition"),
        Some("member"),
    ] {
        let (mut c, d, _) = fixture();
        let mut incoming = bundle();
        let layer = Layer::new("source layer", [10, 20, 30, 255]);
        let member = Entity::new("source member", Geometry::Point([2., 3., 4.]), layer.id);
        let definition = Definition::new("source definition", vec![member]);
        for (kind, family, host, baseline) in [
            (
                NativeRecordKind::Layer,
                "ON_Layer",
                HostRef::Layer(layer.id),
                fingerprint(&layer).unwrap(),
            ),
            (
                NativeRecordKind::Definition,
                "ON_InstanceDefinition",
                HostRef::Definition(definition.id),
                fingerprint(&definition).unwrap(),
            ),
            (
                NativeRecordKind::Geometry,
                "ON_Point",
                HostRef::DefinitionMember {
                    definition: definition.id,
                    object: definition.members[0].id,
                },
                fingerprint(&definition.members[0]).unwrap(),
            ),
        ] {
            incoming.namespace.records.push(SourceRecord::bound(
                NativeRecordKey {
                    kind,
                    id: uuid::Uuid::new_v4(),
                    native_index: Some(1),
                },
                family,
                host,
                baseline,
            ));
        }
        incoming.layers.push(layer);
        incoming.definitions.push(definition);
        if let Some(missing) = missing {
            let i = match missing {
                "root" => 0,
                "layer" => 1,
                "definition" => 2,
                "member" => 3,
                _ => unreachable!(),
            };
            let original = &incoming.namespace.records[i];
            incoming.namespace.records[i] = SourceRecord::retained(original.key, &original.family);
        }
        let before = c.snapshot();
        let result = c.prepare(TransactionRequest::new(
            "Complete source closure",
            vec![Mutation::Import {
                document: d,
                expected_revision: 1,
                bundle: Box::new(incoming),
            }],
        ));
        if let Some(missing) = missing {
            assert!(
                matches!(result, Err(Error::Missing("imported host source binding"))),
                "{missing}"
            );
            assert_eq!(c.snapshot(), before);
        } else {
            c.commit(result.unwrap()).unwrap();
            let doc = &c.world().documents[&d];
            assert_eq!(doc.entities.len(), 1);
            assert_eq!(doc.layers.len(), 1);
            assert_eq!(doc.definitions.len(), 1);
            assert_eq!(doc.definitions.values().next().unwrap().members.len(), 1);
            c.undo().unwrap();
            assert_eq!(c.world(), &before.world);
        }
    }
}
