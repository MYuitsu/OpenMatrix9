// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent authored logical cases; no native archive correspondence implied.
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Definition, Document, Entity},
    resource::{Resource, ResourceKind},
    retained::{
        HostRef, ImportMode, ImportNamespace, NativeRecordKey, NativeRecordKind, NativeRelation,
        NativeUnits, RecordDisposition, ResolvedBy, RetainedSource, SourceRecord, SourceUnits,
        fingerprint,
    },
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{ExportCapture, RecordAction};
use uuid::Uuid;

fn key(kind: NativeRecordKind, n: u128) -> NativeRecordKey {
    NativeRecordKey {
        kind,
        id: Uuid::from_u128(n),
        native_index: None,
    }
}
fn fixture(
    mode: ImportMode,
    opaque_dependency: bool,
) -> (
    TransactionCoordinator,
    DocumentId,
    DefinitionId,
    ObjectId,
    ObjectId,
) {
    let mut d = Document::new("independent shared definition overlay");
    let a = Entity::new("A", Geometry::Point([1., 2., 3.]), None);
    let b = Entity::new("B", Geometry::Point([4., 5., 6.]), None);
    let a_id = a.id;
    let b_id = b.id;
    let mut def = Definition::new("shared", vec![a, b]);
    let def_id = def.id;
    let def_key = key(NativeRecordKind::Definition, 1);
    let a_key = key(NativeRecordKind::Geometry, 2);
    let b_key = key(NativeRecordKind::Geometry, 3);
    let mut definition = SourceRecord::bound(
        def_key,
        "ON_InstanceDefinition",
        HostRef::Definition(def.id),
        fingerprint(&def).unwrap(),
    );
    definition.relations = [a_key, b_key]
        .into_iter()
        .map(|target| NativeRelation {
            role: "definition_member".into(),
            target,
        })
        .collect();
    let mut records = vec![definition];
    for (key, entity) in [(a_key, &def.members[0]), (b_key, &def.members[1])] {
        records.push(
            SourceRecord::bound_entity(
                key,
                "ON_Point",
                HostRef::DefinitionMember {
                    definition: def.id,
                    object: entity.id,
                },
                entity,
            )
            .unwrap(),
        );
    }
    for n in 0..2 {
        let mut instance = Entity::new("instance", Geometry::Instance(def.id), None);
        instance.placement = Transform::translation([10. * n as f64, 0., 0.]);
        let mut record = SourceRecord::bound_entity(
            key(NativeRecordKind::Geometry, 4 + n),
            "ON_InstanceRef",
            HostRef::Object(instance.id),
            &instance,
        )
        .unwrap();
        record.relations.push(NativeRelation {
            role: "instance_definition".into(),
            target: def_key,
        });
        records.push(record);
        d.entities.insert(instance.id, instance);
    }
    records.push(SourceRecord::retained(
        key(NativeRecordKind::Geometry, 6),
        "ON_Brep",
    ));
    if opaque_dependency {
        let mut history =
            SourceRecord::retained(key(NativeRecordKind::History, 7), "ON_HistoryRecord");
        history.relations.push(NativeRelation {
            role: "history_antecedent".into(),
            target: b_key,
        });
        records.push(history);
    }
    // A coherent current state with one edited and one removed local member.
    def.members[0].geometry = Geometry::Point([7., 8., 9.]);
    def.members[0].geometry_revision += 1;
    def.members.pop();
    def.revision += 1;
    records[0].disposition = RecordDisposition::Edited;
    records[1].disposition = RecordDisposition::Edited;
    records[2].disposition = RecordDisposition::Deleted;
    d.definitions.insert(def.id, def);
    let resource = Resource::from_bytes(
        "authored source",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"owned logical fixture, no native parse claim",
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
    let ns = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: Some(mode),
        records,
    };
    d.resources.insert(resource.id, resource);
    d.retained_sources.insert(source.id, source);
    d.import_namespaces.insert(ns.id, ns);
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    (c, id, def_id, a_id, b_id)
}

#[test]
fn shared_definition_patches_member_list_once_and_does_not_publish_removed_member_as_addition() {
    let (c, d, def, a, b) = fixture(ImportMode::Preserve, false);
    let before = c.snapshot();
    let capture = ExportCapture::all(&c, d).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    let rows = &overlay.namespaces[0].records;
    assert_eq!(rows.len(), 6);
    let row = |host| rows.iter().find(|r| r.source.host == Some(host)).unwrap();
    assert_eq!(
        row(HostRef::Definition(def)).action,
        RecordAction::PatchDefinition
    );
    assert_eq!(
        row(HostRef::DefinitionMember {
            definition: def,
            object: a
        })
        .action,
        RecordAction::PatchEntity
    );
    assert!(
        row(HostRef::DefinitionMember {
            definition: def,
            object: a
        })
        .changed
        .geometry
    );
    let deleted = row(HostRef::DefinitionMember {
        definition: def,
        object: b,
    });
    assert_eq!(deleted.action, RecordAction::Delete);
    assert!(deleted.current.is_none());
    assert_eq!(
        rows.iter()
            .filter(|r| r.source.family == "ON_InstanceRef" && r.action == RecordAction::Retain)
            .count(),
        2
    );
    assert!(row(HostRef::Definition(def)).needs_cache_refresh);
    assert!(
        rows.iter()
            .filter(|r| r.source.family == "ON_InstanceRef")
            .all(|r| r.needs_cache_refresh),
        "unchanged instance UUIDs still need current cached bounds"
    );
    assert!(overlay.additions.is_empty());
    assert_eq!(c.snapshot(), before);
}
#[test]
fn rewriting_definition_member_list_does_not_hide_an_opaque_history_dependency_on_deleted_member() {
    let (c, d, _, _, _) = fixture(ImportMode::Preserve, true);
    let before = c.snapshot();
    let capture = ExportCapture::all(&c, d).unwrap();
    assert!(matches!(
        capture.current_overlay(&c, &Cancellation::default()),
        Err(Error::Unsupported(
            "unchanged native dependency was deleted"
        ))
    ));
    assert_eq!(c.snapshot(), before);
}
#[test]
fn geometry_policy_regenerates_shared_current_members_and_omits_opaque_history_without_resurrection()
 {
    let (c, d, def, a, b) = fixture(ImportMode::Geometry, true);
    let capture = ExportCapture::all(&c, d).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    let rows = &overlay.namespaces[0].records;
    let row = |host| rows.iter().find(|r| r.source.host == Some(host)).unwrap();
    assert_eq!(
        row(HostRef::Definition(def)).action,
        RecordAction::Regenerate
    );
    assert_eq!(
        row(HostRef::DefinitionMember {
            definition: def,
            object: a
        })
        .action,
        RecordAction::Regenerate
    );
    assert_eq!(
        row(HostRef::DefinitionMember {
            definition: def,
            object: b
        })
        .action,
        RecordAction::Delete
    );
    assert!(
        rows.iter()
            .filter(|r| r.source.host.is_none())
            .all(|r| r.action == RecordAction::Omit && r.current.is_none())
    );
    assert!(overlay.additions.is_empty());
}
#[test]
fn decoded_native_light_without_a_component_index_is_supported_or_omitted_by_policy() {
    use om9_core::retained::DefinitionUnits;
    use om9_exchange::ScopedNativeKey;
    for mode in [ImportMode::Preserve, ImportMode::Geometry] {
        let (c, id, _, _, _) = fixture(mode, false);
        let mut d = c.world().documents[&id].as_ref().clone();
        let namespace = *d.import_namespaces.keys().next().unwrap();
        let ns = d.import_namespaces.get_mut(&namespace).unwrap();
        let original_definition = ns.records[0].key.id;
        ns.records[0].key.native_index = Some(0);
        let definition_key = ns.records[0].key;
        for record in &mut ns.records {
            for relation in &mut record.relations {
                if relation.target.kind == NativeRecordKind::Definition
                    && relation.target.id == original_definition
                {
                    relation.target = definition_key;
                }
            }
        }
        let light = key(NativeRecordKind::Light, 99);
        ns.records.push(SourceRecord::retained(light, "ON_Light"));
        d.retained_sources
            .get_mut(&ns.source)
            .unwrap()
            .units
            .definitions
            .insert(
                original_definition,
                DefinitionUnits {
                    native_enum: 2,
                    name: "Millimeters".into(),
                    declared_mm_per_unit: Some(1.),
                    millimeters_per_unit: Some(1.),
                    resolved_by: Some(ResolvedBy::Declared),
                },
            );
        let c = TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap();
        let capture = ExportCapture::all(&c, id).unwrap();
        let plan = capture
            .current_output_plan(&c, None, &Cancellation::default(), Default::default())
            .expect(
                "light source rows have decoded native_index=None, not a fabricated table ordinal",
            );
        let output = plan.source_identities[&ScopedNativeKey {
            namespace,
            key: light,
        }];
        if mode == ImportMode::Preserve {
            let output = output.unwrap();
            assert_eq!(output.native_component_type, 9);
            assert_eq!(output.id, light.id);
            assert!(output.native_index.is_none());
        } else {
            assert!(output.is_none());
        }
    }
}
#[test]
fn equal_extreme_model_and_definition_units_cancel_to_identity_without_intermediate_overflow() {
    use om9_core::retained::DefinitionUnits;
    for factor in [1e120, 1e-120] {
        let (c, id, _, _, _) = fixture(ImportMode::Preserve, false);
        let mut d = c.world().documents[&id].as_ref().clone();
        let namespace = *d.import_namespaces.keys().next().unwrap();
        let ns = d.import_namespaces.get_mut(&namespace).unwrap();
        ns.records[0].key.native_index = Some(0);
        let definition_key = ns.records[0].key;
        for record in &mut ns.records {
            for relation in &mut record.relations {
                if relation.target.kind == NativeRecordKind::Definition
                    && relation.target.id == definition_key.id
                {
                    relation.target = definition_key;
                }
            }
        }
        let units = &mut d.retained_sources.get_mut(&ns.source).unwrap().units;
        units.model.native_enum = 11;
        units.model.name = "custom model units".into();
        units.model.declared_mm_per_unit = Some(factor);
        units.model.millimeters_per_unit = factor;
        units.definitions.insert(
            definition_key.id,
            DefinitionUnits {
                native_enum: 11,
                name: "custom definition units".into(),
                declared_mm_per_unit: Some(factor),
                millimeters_per_unit: Some(factor),
                resolved_by: Some(ResolvedBy::Declared),
            },
        );
        let c = TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap();
        let capture = ExportCapture::all(&c, id).unwrap();
        let plan = capture
            .current_output_plan(&c, None, &Cancellation::default(), Default::default())
            .unwrap();
        let entity = capture
            .document()
            .entities
            .values()
            .find(|e| e.placement == Transform::identity())
            .unwrap();
        assert_eq!(
            plan.native_placement(HostRef::Object(entity.id), entity)
                .unwrap(),
            Transform::identity()
        );
    }
}

#[test]
fn changed_leaf_invalidates_nested_retained_ancestors_and_unchanged_irefs_only() {
    let (c, id, _, _, _) = fixture(ImportMode::Preserve, false);
    let mut d = c.world().documents[&id].as_ref().clone();
    let ns = d.import_namespaces.values_mut().next().unwrap();
    let leaf = ns.records[0].key;
    let parent = key(NativeRecordKind::Definition, 10);
    let parent_member = key(NativeRecordKind::Geometry, 11);
    let ancestor = key(NativeRecordKind::Definition, 12);
    let ancestor_member = key(NativeRecordKind::Geometry, 13);
    let ancestor_root = key(NativeRecordKind::Geometry, 14);
    let unrelated = key(NativeRecordKind::Definition, 20);
    let unrelated_member = key(NativeRecordKind::Geometry, 21);
    let unrelated_root = key(NativeRecordKind::Geometry, 22);
    for (record_key, family, role, target) in [
        (
            parent,
            "ON_InstanceDefinition",
            "definition_member",
            parent_member,
        ),
        (parent_member, "ON_InstanceRef", "instance_definition", leaf),
        (
            ancestor,
            "ON_InstanceDefinition",
            "definition_member",
            ancestor_member,
        ),
        (
            ancestor_member,
            "ON_InstanceRef",
            "instance_definition",
            parent,
        ),
        (
            ancestor_root,
            "ON_InstanceRef",
            "instance_definition",
            ancestor,
        ),
        (
            unrelated,
            "ON_InstanceDefinition",
            "definition_member",
            unrelated_member,
        ),
        (
            unrelated_root,
            "ON_InstanceRef",
            "instance_definition",
            unrelated,
        ),
    ] {
        let mut row = SourceRecord::retained(record_key, family);
        row.relations.push(NativeRelation {
            role: role.into(),
            target,
        });
        ns.records.push(row);
    }
    ns.records
        .push(SourceRecord::retained(unrelated_member, "ON_Brep"));
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    let rows = &overlay.namespaces[0].records;
    let row = |key| rows.iter().find(|row| row.source.key == key).unwrap();
    for key in [
        parent,
        parent_member,
        ancestor,
        ancestor_member,
        ancestor_root,
    ] {
        assert_eq!(row(key).action, RecordAction::Retain);
        assert!(row(key).needs_cache_refresh, "{key:?}");
    }
    for instance in rows
        .iter()
        .filter(|row| row.source.host.is_some() && row.source.family == "ON_InstanceRef")
    {
        assert_eq!(instance.action, RecordAction::Retain);
        assert_eq!(instance.changed, Default::default());
        assert!(instance.needs_cache_refresh);
    }
    for key in [unrelated, unrelated_member, unrelated_root] {
        assert_eq!(row(key).action, RecordAction::Retain);
        assert!(!row(key).needs_cache_refresh, "unrelated {key:?}");
    }
}

#[test]
fn changing_iref_placement_refreshes_its_own_cached_bounds_with_clean_child_definition() {
    let (c, id, def_id, _, b_id) = fixture(ImportMode::Preserve, false);
    let mut d = c.world().documents[&id].as_ref().clone();
    // Restore the independently authored original leaf, leaving only a root
    // placement edit. Its child definition and geometry fields remain clean.
    let definition = d.definitions.get_mut(&def_id).unwrap();
    definition.members[0].geometry = Geometry::Point([1., 2., 3.]);
    definition.members[0].geometry_revision = 1;
    let mut b = Entity::new("B", Geometry::Point([4., 5., 6.]), None);
    b.id = b_id;
    definition.members.push(b);
    definition.revision = 1;
    let ns = d.import_namespaces.values_mut().next().unwrap();
    for row in &mut ns.records[..3] {
        row.disposition = RecordDisposition::Unchanged;
    }
    let root = d
        .entities
        .values_mut()
        .find(|e| e.placement == Transform::identity())
        .unwrap();
    let root_id = root.id;
    root.placement = Transform::translation([100., 0., 0.]);
    root.geometry_revision += 1;
    ns.records
        .iter_mut()
        .find(|row| row.host == Some(HostRef::Object(root_id)))
        .unwrap()
        .disposition = RecordDisposition::Edited;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    let rows = &overlay.namespaces[0].records;
    let definition = rows
        .iter()
        .find(|row| row.source.host == Some(HostRef::Definition(def_id)))
        .unwrap();
    assert_eq!(definition.action, RecordAction::Retain);
    assert!(!definition.needs_cache_refresh);
    let root = rows
        .iter()
        .find(|row| row.source.host == Some(HostRef::Object(root_id)))
        .unwrap();
    assert_eq!(root.action, RecordAction::PatchEntity);
    assert!(root.changed.placement);
    assert!(!root.changed.geometry);
    assert!(
        root.needs_cache_refresh,
        "the instance's own transformed bbox became stale"
    );
}

#[test]
fn switching_iref_child_refreshes_its_own_bounds_while_both_definitions_stay_clean() {
    let (c, id, original_definition_id, _, b_id) = fixture(ImportMode::Preserve, false);
    let mut d = c.world().documents[&id].as_ref().clone();
    let original = d.definitions.get_mut(&original_definition_id).unwrap();
    original.members[0].geometry = Geometry::Point([1., 2., 3.]);
    original.members[0].geometry_revision = 1;
    let mut b = Entity::new("B", Geometry::Point([4., 5., 6.]), None);
    b.id = b_id;
    original.members.push(b);
    original.revision = 1;

    let member = Entity::new(
        "other original member",
        Geometry::Point([1000., 0., 0.]),
        None,
    );
    let replacement = Definition::new("other original child", vec![member]);
    let replacement_id = replacement.id;
    let replacement_key = key(NativeRecordKind::Definition, 30);
    let member_key = key(NativeRecordKind::Geometry, 31);
    let member_record = SourceRecord::bound_entity(
        member_key,
        "ON_Point",
        HostRef::DefinitionMember {
            definition: replacement_id,
            object: replacement.members[0].id,
        },
        &replacement.members[0],
    )
    .unwrap();
    let mut definition_record = SourceRecord::bound(
        replacement_key,
        "ON_InstanceDefinition",
        HostRef::Definition(replacement_id),
        fingerprint(&replacement).unwrap(),
    );
    definition_record.relations.push(NativeRelation {
        role: "definition_member".into(),
        target: member_key,
    });
    d.definitions.insert(replacement_id, replacement);
    let ns = d.import_namespaces.values_mut().next().unwrap();
    for row in &mut ns.records[..3] {
        row.disposition = RecordDisposition::Unchanged;
    }
    ns.records.extend([definition_record, member_record]);
    let root = d
        .entities
        .values_mut()
        .find(|entity| entity.placement == Transform::identity())
        .unwrap();
    let root_id = root.id;
    root.geometry = Geometry::Instance(replacement_id);
    root.geometry_revision += 1;
    // Its original native relation continues to identify the old child. The
    // current overlay must follow the edited host geometry instead.
    ns.records
        .iter_mut()
        .find(|row| row.host == Some(HostRef::Object(root_id)))
        .unwrap()
        .disposition = RecordDisposition::Edited;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let overlay = capture
        .current_overlay(&c, &Cancellation::default())
        .unwrap();
    let rows = &overlay.namespaces[0].records;
    for definition_id in [original_definition_id, replacement_id] {
        let row = rows
            .iter()
            .find(|row| row.source.host == Some(HostRef::Definition(definition_id)))
            .unwrap();
        assert_eq!(row.action, RecordAction::Retain);
        assert!(!row.needs_cache_refresh);
    }
    let root = rows
        .iter()
        .find(|row| row.source.host == Some(HostRef::Object(root_id)))
        .unwrap();
    assert_eq!(root.action, RecordAction::PatchEntity);
    assert!(root.changed.geometry);
    assert!(!root.changed.placement);
    assert!(root.needs_cache_refresh);
    let unchanged_root = rows
        .iter()
        .find(|row| {
            row.source.family == "ON_InstanceRef"
                && row.source.host != Some(HostRef::Object(root_id))
        })
        .unwrap();
    assert_eq!(unchanged_root.action, RecordAction::Retain);
    assert!(!unchanged_root.needs_cache_refresh);
}
