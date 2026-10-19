// SPDX-License-Identifier: LGPL-2.1-or-later
//! Authored ownership fixtures; these tests do not claim native serialization.
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
    retained::{
        HostRef, ImportBundle, ImportMode, ImportNamespace, NativeRecordKey, NativeRecordKind,
        NativeRelation, NativeUnits, ResolvedBy, RetainedSource, SourceRecord, SourceUnits,
    },
    transaction::*,
};
use om9_exchange::{ExportCapture, RecordAction};
use uuid::Uuid;

fn key(kind: NativeRecordKind) -> NativeRecordKey {
    NativeRecordKey {
        kind,
        id: Uuid::new_v4(),
        native_index: None,
    }
}
fn bundle(mode: Option<ImportMode>) -> ImportBundle {
    let resource = Resource::from_bytes(
        "owned logical overlay fixture",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored fixture; actual native preservation is separately tested",
        &Limits::default(),
    )
    .unwrap();
    let unit = NativeUnits {
        native_enum: 2,
        name: "Millimeters".into(),
        declared_mm_per_unit: Some(1.),
        millimeters_per_unit: 1.,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.001,
        angular_tolerance_radians: 0.001,
        relative_tolerance: 0.,
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
    let a = Entity::new("A", Geometry::Point([1., 2., 3.]), None);
    let b = Entity::new("B", Geometry::Point([4., 5., 6.]), None);
    let records = vec![
        SourceRecord::bound_entity(
            key(NativeRecordKind::Geometry),
            "ON_Point",
            HostRef::Object(a.id),
            &a,
        )
        .unwrap(),
        SourceRecord::bound_entity(
            key(NativeRecordKind::Geometry),
            "ON_Point",
            HostRef::Object(b.id),
            &b,
        )
        .unwrap(),
        SourceRecord::retained(key(NativeRecordKind::Geometry), "ON_Brep"),
    ];
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: mode,
        records,
    };
    ImportBundle {
        resource,
        source,
        namespace,
        entities: vec![a, b],
        layers: vec![],
        definitions: vec![],
    }
}
fn fixture(
    bundle: ImportBundle,
) -> (
    TransactionCoordinator,
    DocumentId,
    ImportNamespaceId,
    ObjectId,
    ObjectId,
) {
    let d = Document::new("overlay target");
    let id = d.id;
    let ns = bundle.namespace.id;
    let a = bundle.entities[0].id;
    let b = bundle.entities[1].id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    commit(
        &mut c,
        vec![Mutation::Import {
            document: id,
            expected_revision: 1,
            bundle: Box::new(bundle),
        }],
    );
    (c, id, ns, a, b)
}
fn commit(c: &mut TransactionCoordinator, mutations: Vec<Mutation>) {
    let prepared = c
        .prepare(TransactionRequest::new(
            "authored current change",
            mutations,
        ))
        .unwrap();
    c.commit(prepared).unwrap();
}

#[test]
fn edited_a_deleted_b_and_new_c_are_current_actions_with_retained_brep_and_undo() {
    let (mut c, d, _, a, b) = fixture(bundle(Some(ImportMode::Preserve)));
    let new = Entity::new("C", Geometry::Point([9., 8., 7.]), None);
    let new_id = new.id;
    let baseline = c.world().clone();
    let doc = &c.world().documents[&d];
    let a_stamp = doc.stamp(a).unwrap();
    let b_stamp = doc.stamp(b).unwrap();
    commit(
        &mut c,
        vec![
            Mutation::Geometry {
                source: a_stamp,
                geometry: Geometry::Point([10., 2., 3.]),
            },
            Mutation::Delete { source: b_stamp },
            Mutation::Insert {
                document: d,
                entity: new,
            },
        ],
    );
    let capture = ExportCapture::all(&c, d).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    let records = &overlay.namespaces[0].records;
    assert_eq!(records[0].action, RecordAction::PatchEntity);
    assert!(records[0].changed.geometry);
    assert!(
        !records[0].changed.placement
            && !records[0].changed.attributes
            && !records[0].changed.label
    );
    assert_eq!(records[1].action, RecordAction::Delete);
    assert!(records[1].current.is_none());
    assert_eq!(records[2].action, RecordAction::Retain);
    assert_eq!(overlay.additions.len(), 1);
    assert_eq!(overlay.additions[0].host(), HostRef::Object(new_id));
    c.undo().unwrap();
    assert_eq!(c.world(), &baseline);
    assert!(matches!(
        capture.current_overlay(&c, &Cancellation::default()),
        Err(Error::Stale(_))
    ));
    let restored = ExportCapture::all(&c, d).unwrap();
    assert!(
        restored
            .current_overlay(&c, &Cancellation::default())
            .unwrap()
            .namespaces[0]
            .records
            .iter()
            .all(|r| r.action == RecordAction::Retain)
    );
}

#[test]
fn geometry_mode_regenerates_current_bindings_and_does_not_resurrect_retained_only_data() {
    let (mut c, d, _, _, b) = fixture(bundle(Some(ImportMode::Geometry)));
    let stamp = c.world().documents[&d].stamp(b).unwrap();
    commit(&mut c, vec![Mutation::Delete { source: stamp }]);
    let capture = ExportCapture::all(&c, d).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    assert_eq!(
        overlay.namespaces[0].records[0].action,
        RecordAction::Regenerate
    );
    assert_eq!(
        overlay.namespaces[0].records[1].action,
        RecordAction::Delete
    );
    assert_eq!(overlay.namespaces[0].records[2].action, RecordAction::Omit);
    assert!(overlay.namespaces[0].records[2].current.is_none());
}

#[test]
fn missing_legacy_policy_and_missing_edited_field_baseline_reject_without_mutation() {
    for policy_missing in [true, false] {
        let mut input = bundle(if policy_missing {
            None
        } else {
            Some(ImportMode::Preserve)
        });
        if !policy_missing {
            input.namespace.records[0].entity_baseline = None;
        }
        let (mut c, d, _, a, _) = fixture(input);
        if !policy_missing {
            let stamp = c.world().documents[&d].stamp(a).unwrap();
            commit(
                &mut c,
                vec![Mutation::Placement {
                    source: stamp,
                    placement: Transform::translation([5., 0., 0.]),
                }],
            );
        }
        let before = c.snapshot();
        let capture = ExportCapture::all(&c, d).unwrap();
        assert!(matches!(
            capture.current_overlay(&c, &Cancellation::default()),
            Err(Error::Unsupported(_))
        ));
        assert_eq!(c.snapshot(), before);
    }
}

#[test]
fn reverting_host_fields_retains_native_geometry_despite_advanced_revisions() {
    let (mut c, d, _, a, _) = fixture(bundle(Some(ImportMode::Preserve)));
    for point in [[10., 2., 3.], [1., 2., 3.]] {
        let source = c.world().documents[&d].stamp(a).unwrap();
        commit(
            &mut c,
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Point(point),
            }],
        );
    }
    assert!(c.world().documents[&d].entities[&a].geometry_revision > 1);
    let capture = ExportCapture::all(&c, d).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    assert_eq!(
        overlay.namespaces[0].records[0].action,
        RecordAction::Retain
    );
    assert_eq!(overlay.namespaces[0].records[0].changed, Default::default());
}

#[test]
fn untouched_native_dependency_cannot_point_to_a_deleted_record() {
    let mut input = bundle(Some(ImportMode::Preserve));
    let target = input.namespace.records[1].key;
    input.namespace.records[2].relations.push(NativeRelation {
        role: "history_source".into(),
        target,
    });
    let (mut c, d, _, _, b) = fixture(input);
    let source = c.world().documents[&d].stamp(b).unwrap();
    commit(&mut c, vec![Mutation::Delete { source }]);
    let before = c.snapshot();
    let capture = ExportCapture::all(&c, d).unwrap();
    assert!(matches!(
        capture.current_overlay(&c, &Cancellation::default()),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn original_cancel_and_equal_counter_wrong_coordinator_are_refused() {
    let (c, d, _, _, _) = fixture(bundle(Some(ImportMode::Preserve)));
    let capture = ExportCapture::all(&c, d).unwrap();
    let token = Cancellation::default();
    token.cancel();
    assert!(matches!(
        capture.current_overlay(&c, &token),
        Err(Error::Canceled)
    ));
    let wrong = TransactionCoordinator::new(c.world().clone(), Limits::default()).unwrap();
    assert!(matches!(
        capture.current_overlay(&wrong, &Cancellation::default()),
        Err(Error::WrongOwner)
    ));
}

#[test]
fn placement_attributes_and_label_change_only_their_explicit_native_patch_fields() {
    use om9_core::result::{ConversionReport, PreparedOutput, PreparedResult};
    for field in ["placement", "attributes", "label"] {
        let (mut c, d, _, a, _) = fixture(bundle(Some(ImportMode::Preserve)));
        let source = c.world().documents[&d].stamp(a).unwrap();
        let mutation = match field {
            "placement" => Mutation::Placement {
                source,
                placement: Transform::translation([5., 0., 0.]),
            },
            "attributes" => {
                let mut attributes = c.world().documents[&d].entities[&a].attributes.clone();
                attributes.color = Some([11, 22, 33, 200]);
                Mutation::Attributes { source, attributes }
            }
            "label" => {
                let mut entity = c.world().documents[&d].entities[&a].clone();
                entity.label = "new native name".into();
                Mutation::Publish(Box::new(PreparedResult {
                    document: d,
                    operation: "rename current result".into(),
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
        commit(&mut c, vec![mutation]);
        let capture = ExportCapture::all(&c, d).unwrap();
        let overlay = capture
            .current_overlay(&c, &Cancellation::default())
            .unwrap();
        let row = &overlay.namespaces[0].records[0];
        assert_eq!(row.action, RecordAction::PatchEntity, "{field}");
        assert!(!row.changed.geometry, "{field}");
        assert_eq!(row.changed.placement, field == "placement");
        assert_eq!(row.changed.attributes, field == "attributes");
        assert_eq!(row.changed.label, field == "label");
    }
}

#[test]
fn overlay_scratch_is_admitted_with_current_world_before_allocation() {
    let (c, d, _, _, _) = fixture(bundle(Some(ImportMode::Preserve)));
    let capture = ExportCapture::all(&c, d).unwrap();
    let external = om9_core::budget::Usage {
        bytes: c.limits().max_bytes
            - c.retained_usage().unwrap().bytes
            - om9_core::budget::Usage::array::<SourceStamp>(c.world().documents[&d].entities.len())
                .unwrap()
                .bytes
            - 64,
        ..Default::default()
    };
    let before = c.snapshot();
    assert!(matches!(
        capture.current_overlay_with_held_usage(&c, &Cancellation::default(), external),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn repeated_native_keys_remain_distinct_between_preserve_and_geometry_namespaces() {
    let first = bundle(Some(ImportMode::Preserve));
    let keys: Vec<_> = first.namespace.records.iter().map(|r| r.key).collect();
    let (mut c, d, first_ns, _, _) = fixture(first);
    let mut second = bundle(Some(ImportMode::Geometry));
    for (record, key) in second.namespace.records.iter_mut().zip(keys) {
        record.key = key;
    }
    let second_ns = second.namespace.id;
    let revision = c.world().documents[&d].revision;
    commit(
        &mut c,
        vec![Mutation::Import {
            document: d,
            expected_revision: revision,
            bundle: Box::new(second),
        }],
    );
    let capture = ExportCapture::all(&c, d).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    assert_eq!(overlay.namespaces.len(), 2);
    let preserved = overlay
        .namespaces
        .iter()
        .find(|n| n.namespace.id == first_ns)
        .unwrap();
    let geometry = overlay
        .namespaces
        .iter()
        .find(|n| n.namespace.id == second_ns)
        .unwrap();
    assert_eq!(
        preserved.records[0].source.key,
        geometry.records[0].source.key
    );
    assert_ne!(
        preserved.records[0].current.unwrap().host(),
        geometry.records[0].current.unwrap().host()
    );
    assert!(
        preserved
            .records
            .iter()
            .all(|r| r.action == RecordAction::Retain)
    );
    assert_eq!(geometry.records[0].action, RecordAction::Regenerate);
    assert_eq!(geometry.records[2].action, RecordAction::Omit);
    assert!(overlay.additions.is_empty());
}

#[test]
fn output_map_keeps_primary_native_ids_deletes_b_and_allocates_distinct_new_c() {
    let input = bundle(Some(ImportMode::Preserve));
    let native_a = input.namespace.records[0].key;
    let native_b = input.namespace.records[1].key;
    let (mut c, d, ns, a, b) = fixture(input);
    let new = Entity::new("C", Geometry::Point([9., 8., 7.]), None);
    let new_id = new.id;
    let doc = &c.world().documents[&d];
    let a_stamp = doc.stamp(a).unwrap();
    let b_stamp = doc.stamp(b).unwrap();
    commit(
        &mut c,
        vec![
            Mutation::Geometry {
                source: a_stamp,
                geometry: Geometry::Point([10., 2., 3.]),
            },
            Mutation::Delete { source: b_stamp },
            Mutation::Insert {
                document: d,
                entity: new,
            },
        ],
    );
    let capture = ExportCapture::all(&c, d).unwrap();
    let plan = capture
        .current_output_plan(&c, None, &Cancellation::default(), Default::default())
        .unwrap();
    assert_eq!(plan.primary_namespace, Some(ns));
    assert_eq!(plan.host_identities[&HostRef::Object(a)].id, native_a.id);
    assert_eq!(
        plan.source_identities[&om9_exchange::ScopedNativeKey {
            namespace: ns,
            key: native_b
        }],
        None
    );
    assert!(!plan.host_identities.contains_key(&HostRef::Object(b)));
    let out_c = plan.host_identities[&HostRef::Object(new_id)];
    assert_ne!(out_c.id, new_id.0);
    assert_ne!(out_c.id, native_a.id);
    assert_ne!(out_c.id, native_b.id);
    assert_eq!(out_c.native_component_type, 12);
    assert_eq!(out_c.native_index, None);
    assert_eq!(plan.source_identities.len(), 3);
}

#[test]
fn multiple_preserve_imports_require_primary_and_remap_repeated_source_uuids() {
    let first = bundle(Some(ImportMode::Preserve));
    let keys: Vec<_> = first.namespace.records.iter().map(|r| r.key).collect();
    let native_a = keys[0].id;
    let (mut c, d, primary, a, _) = fixture(first);
    let mut second = bundle(Some(ImportMode::Preserve));
    for (record, key) in second.namespace.records.iter_mut().zip(keys) {
        record.key = key;
    }
    let second_a = second.entities[0].id;
    let revision = c.world().documents[&d].revision;
    commit(
        &mut c,
        vec![Mutation::Import {
            document: d,
            expected_revision: revision,
            bundle: Box::new(second),
        }],
    );
    let capture = ExportCapture::all(&c, d).unwrap();
    assert!(matches!(
        capture.current_output_plan(&c, None, &Cancellation::default(), Default::default()),
        Err(Error::Unsupported(_))
    ));
    let plan = capture
        .current_output_plan(
            &c,
            Some(primary),
            &Cancellation::default(),
            Default::default(),
        )
        .unwrap();
    assert_eq!(plan.host_identities[&HostRef::Object(a)].id, native_a);
    assert_ne!(
        plan.host_identities[&HostRef::Object(second_a)].id,
        native_a
    );
    let ids: std::collections::BTreeSet<_> = plan
        .source_identities
        .values()
        .flatten()
        .map(|id| id.id)
        .collect();
    assert_eq!(ids.len(), 6);
    assert_eq!(plan.source_identities.len(), 6);
}

#[test]
fn geometry_output_gets_fresh_native_ids_and_has_no_retained_only_output_identity() {
    let input = bundle(Some(ImportMode::Geometry));
    let omitted = input.namespace.records[2].key;
    let native_a = input.namespace.records[0].key.id;
    let (c, d, ns, a, _) = fixture(input);
    let capture = ExportCapture::all(&c, d).unwrap();
    assert!(matches!(
        capture.current_output_plan(&c, Some(ns), &Cancellation::default(), Default::default()),
        Err(Error::Invalid(_))
    ));
    let plan = capture
        .current_output_plan(&c, None, &Cancellation::default(), Default::default())
        .unwrap();
    assert_eq!(plan.primary_namespace, None);
    assert_ne!(plan.host_identities[&HostRef::Object(a)].id, native_a);
    assert_eq!(
        plan.source_identities[&om9_exchange::ScopedNativeKey {
            namespace: ns,
            key: omitted
        }],
        None
    );
    assert_eq!(plan.model_mm_per_unit, 1.);
}

#[test]
fn native_frames_preserve_physical_nested_points_reflection_and_nonuniform_scale_once() {
    use om9_core::{
        model::Definition,
        retained::{DefinitionUnits, fingerprint},
    };
    let mut input = bundle(Some(ImportMode::Preserve));
    input.source.units.model.native_enum = 8;
    input.source.units.model.name = "Inches".into();
    input.source.units.model.declared_mm_per_unit = Some(25.4);
    input.source.units.model.millimeters_per_unit = 25.4;
    let member = Entity::new("local inch point", Geometry::Point([25.4, 0., 0.]), None);
    let def = Definition::new("shared inch definition", vec![member]);
    let member = &def.members[0];
    let member_host = HostRef::DefinitionMember {
        definition: def.id,
        object: member.id,
    };
    let member_key = key(NativeRecordKind::Geometry);
    let member_record =
        SourceRecord::bound_entity(member_key, "ON_Point", member_host, member).unwrap();
    let def_key = NativeRecordKey {
        kind: NativeRecordKind::Definition,
        id: Uuid::new_v4(),
        native_index: Some(0),
    };
    let mut def_record = SourceRecord::bound(
        def_key,
        "ON_InstanceDefinition",
        HostRef::Definition(def.id),
        fingerprint(&def).unwrap(),
    );
    def_record.relations.push(NativeRelation {
        role: "definition_member".into(),
        target: member_key,
    });
    input.source.units.definitions.insert(
        def_key.id,
        DefinitionUnits {
            native_enum: 8,
            name: "Inches".into(),
            declared_mm_per_unit: Some(25.4),
            millimeters_per_unit: Some(25.4),
            resolved_by: Some(ResolvedBy::Declared),
        },
    );
    input.entities[0].geometry = Geometry::Instance(def.id);
    input.entities[0].placement = Transform::translation([10., 8., 0.])
        .compose(Transform::scale([-2., 3., 1.]).unwrap())
        .unwrap();
    input.namespace.records[0] = SourceRecord::bound_entity(
        input.namespace.records[0].key,
        "ON_InstanceRef",
        HostRef::Object(input.entities[0].id),
        &input.entities[0],
    )
    .unwrap();
    input.namespace.records[0].relations.push(NativeRelation {
        role: "instance_definition".into(),
        target: def_key,
    });
    input.namespace.records.extend([member_record, def_record]);
    input.definitions.push(def);
    let (c, d, _, a, _) = fixture(input);
    let capture = ExportCapture::all(&c, d).unwrap();
    let plan = capture
        .current_output_plan(&c, None, &Cancellation::default(), Default::default())
        .unwrap();
    let root = &capture.document().entities[&a];
    let Geometry::Instance(def) = root.geometry else {
        panic!("fixture root instance")
    };
    let member = &capture.document().definitions[&def].members[0];
    let Geometry::Point(local_mm) = member.geometry else {
        panic!("fixture member point")
    };
    let native_local = plan
        .native_placement(member_host, member)
        .unwrap()
        .point(local_mm)
        .unwrap();
    for (actual, expected) in native_local.into_iter().zip([1., 0., 0.]) {
        assert!((actual - expected).abs() < 1e-12, "{native_local:?}");
    }
    let native_world = plan
        .native_placement(HostRef::Object(a), root)
        .unwrap()
        .point(native_local)
        .unwrap();
    let mm_world = native_world.map(|x| x * plan.model_mm_per_unit);
    for (actual, expected) in mm_world.into_iter().zip([-40.8, 8., 0.]) {
        assert!((actual - expected).abs() < 1e-12, "{mm_world:?}");
    }
    assert_eq!(
        plan.host_identities[&HostRef::Definition(def)].id,
        def_key.id
    );
    assert_eq!(plan.host_identities[&member_host].id, member_key.id);
}

#[test]
fn object_ordinal_is_not_accepted_as_a_native_geometry_index() {
    let mut input = bundle(Some(ImportMode::Preserve));
    input.namespace.records[0].key.native_index = Some(0);
    let (c, d, _, _, _) = fixture(input);
    let capture = ExportCapture::all(&c, d).unwrap();
    assert!(matches!(
        capture.current_output_plan(&c, None, &Cancellation::default(), Default::default()),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn source_object_table_light_has_no_decoded_component_index_in_both_import_modes() {
    for mode in [ImportMode::Preserve, ImportMode::Geometry] {
        let mut input = bundle(Some(mode));
        let light = key(NativeRecordKind::Light);
        input
            .namespace
            .records
            .push(SourceRecord::retained(light, "ON_Light"));
        let (c, d, ns, _, _) = fixture(input);
        let capture = ExportCapture::all(&c, d).unwrap();
        let plan = capture
            .current_output_plan(&c, None, &Cancellation::default(), Default::default())
            .unwrap();
        let output = plan.source_identities[&om9_exchange::ScopedNativeKey {
            namespace: ns,
            key: light,
        }];
        if mode == ImportMode::Preserve {
            let output = output.unwrap();
            assert_eq!(output.id, light.id);
            assert_eq!(output.native_index, None);
        } else {
            assert_eq!(output, None);
        }
    }
}
