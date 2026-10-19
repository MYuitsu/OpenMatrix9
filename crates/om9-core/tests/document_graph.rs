// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-03 document scene entity contract.
//!
//! Every test here names the L3 oracle it proves:
//! - L3-001 stable EntityId/FeatureId allocation and lifetime
//! - L3-002 typed scene entity records and unsupported kinds
//! - L3-003 geometry handle plus display/metadata attributes
//! - L3-004 layer domain records
//! - L3-005 entity to layer membership
//! - L3-006 layer selection/visibility/lock rules
//! - L3-007 group membership model
//! - L3-008 block definition and instance graph
//! - L3-009 jewellery semantic extension record
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Cloud, Curve, Geometry, GeometryKind, Mesh, NurbsSurface, Transform},
    identity::*,
    model::{
        Document, Entity, Extension, Feature, FeatureKind, GEM_EXTENSION_SLOT, Group, Layer,
        extension_slot_known,
    },
    transaction::*,
};

fn plane_surface() -> NurbsSurface {
    NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![[0., 0., 0.], [0., 1., 0.], [1., 0., 0.], [1., 1., 0.]],
        weights: vec![1., 1., 1., 1.],
        knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
        domain: [[0., 1.], [0., 1.]],
        periodic: [false, false],
        reversed: false,
    }
}

fn triangle_mesh() -> Mesh {
    Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
    }
}

fn point_cloud() -> Cloud {
    Cloud {
        points: vec![[0., 0., 0.], [1., 1., 1.]],
        normals: vec![],
        colors: vec![],
        hidden: vec![],
    }
}

fn line_curve() -> Curve {
    Curve::Line {
        start: [0., 0., 0.],
        end: [10., 0., 0.],
        domain: [0., 1.],
    }
}

fn layer_document() -> (Document, LayerId) {
    let mut doc = Document::new("layers");
    let layer = Layer::new("Metals", [34, 153, 135, 255]);
    let id = layer.id;
    doc.layers.insert(id, layer);
    doc.layer_order.push(id);
    doc.current_layer = Some(id);
    (doc, id)
}

fn coordinator(doc: Document) -> (TransactionCoordinator, DocumentId) {
    let id = doc.id;
    (
        TransactionCoordinator::new(
            World::new(vec![doc], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        id,
    )
}

/* ------------------------------- L3-001 ------------------------------- */

#[test]
fn l3_001_two_entities_and_one_feature_receive_unique_stable_ids() {
    let (doc, _) = layer_document();
    let (mut c, d) = coordinator(doc);
    let a = Entity::new("A", Geometry::Point([0., 0., 0.]), None);
    let b = Entity::new("B", Geometry::Point([1., 0., 0.]), None);
    let (a_id, b_id) = (a.id, b.id);
    let feature = Feature::new("Note", FeatureKind::Annotation, vec![a_id]);
    let feature_id = feature.id;
    assert_ne!(a_id, b_id);
    assert!(!feature_id.is_nil());
    let p = c
        .prepare(TransactionRequest::new(
            "seed",
            vec![
                Mutation::Insert {
                    document: d,
                    entity: a,
                },
                Mutation::Insert {
                    document: d,
                    entity: b,
                },
                Mutation::FeatureInsert {
                    document: d,
                    expected_revision: 1,
                    feature: Box::new(feature),
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert_eq!(doc.feature(feature_id).unwrap().outputs, vec![a_id]);
    assert_eq!(doc.entities.len(), 2);
    // Undo/Redo restores the same identities, it never reallocates them.
    c.undo().unwrap();
    assert!(c.world().documents[&d].features.is_empty());
    assert!(c.world().documents[&d].entities.is_empty());
    c.redo().unwrap();
    assert!(c.world().documents[&d].features.contains_key(&feature_id));
    assert!(c.world().documents[&d].entities.contains_key(&a_id));
}

#[test]
fn l3_001_retired_feature_id_is_never_reallocated() {
    let (doc, _) = layer_document();
    let (mut c, d) = coordinator(doc);
    let feature = Feature::new("Retired", FeatureKind::Semantic, vec![]);
    let feature_id = feature.id;
    let p = c
        .prepare(TransactionRequest::new(
            "feature",
            vec![Mutation::FeatureInsert {
                document: d,
                expected_revision: 1,
                feature: Box::new(feature.clone()),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let baseline = c.snapshot();
    let refused = c.prepare(TransactionRequest::new(
        "reallocate feature",
        vec![Mutation::FeatureInsert {
            document: d,
            expected_revision: 1,
            feature: Box::new(feature),
        }],
    ));
    assert_eq!(
        refused.err(),
        Some(Error::Invalid("allocated feature UUID"))
    );
    assert_eq!(c.snapshot(), baseline);
    assert!(!c.world().documents[&d].features.contains_key(&feature_id));
}

#[test]
fn l3_001_a_feature_output_must_be_a_live_object() {
    let (doc, _) = layer_document();
    let (c, d) = coordinator(doc);
    let ghost = Feature::new("Ghost", FeatureKind::Dimension, vec![ObjectId::new()]);
    let refused = c.prepare(TransactionRequest::new(
        "feature",
        vec![Mutation::FeatureInsert {
            document: d,
            expected_revision: 1,
            feature: Box::new(ghost),
        }],
    ));
    assert_eq!(refused.err(), Some(Error::Missing("feature output")));
    assert!(c.world().documents[&d].features.is_empty());
}

/* ------------------------------- L3-002 ------------------------------- */

#[test]
fn l3_002_every_typed_entity_record_keeps_its_kind_when_round_tripped() {
    let mut doc = Document::new("kinds");
    let definition = om9_core::model::Definition::new(
        "block",
        vec![Entity::new("member", Geometry::Point([0.; 3]), None)],
    );
    let definition_id = definition.id;
    doc.definitions.insert(definition_id, definition);
    for (kind, geometry) in [
        (GeometryKind::Point, Geometry::Point([1., 2., 3.])),
        (GeometryKind::Curve, Geometry::Curve(line_curve())),
        (GeometryKind::Surface, Geometry::Surface(plane_surface())),
        (GeometryKind::Mesh, Geometry::Mesh(triangle_mesh())),
        (GeometryKind::Cloud, Geometry::Cloud(point_cloud())),
        (GeometryKind::Instance, Geometry::Instance(definition_id)),
    ] {
        let entity = Entity::new(kind.tag(), geometry, None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        assert_eq!(doc.geometry_kind(id).unwrap(), kind);
    }
    let text = serde_json::to_string(&doc).unwrap();
    let reopened: Document = serde_json::from_str(&text).unwrap();
    for (id, entity) in &doc.entities {
        assert_eq!(
            reopened.entities[id].geometry.kind(),
            entity.geometry.kind()
        );
        assert_eq!(reopened.entities[id], *entity);
    }
    assert_eq!(reopened, doc);
}

#[test]
fn l3_002_unsupported_entity_kinds_are_reported_never_downgraded() {
    assert_eq!(
        GeometryKind::from_tag("Brep"),
        Ok(GeometryKind::Brep),
        "a B-Rep envelope must not be decoded as a mesh"
    );
    assert_eq!(
        GeometryKind::from_tag("T-Spline"),
        Err(Error::Unsupported("geometry kind"))
    );
    assert_eq!(GeometryKind::from_tag("Mesh"), Ok(GeometryKind::Mesh));
}

/* ------------------------------- L3-003 ------------------------------- */

#[test]
fn l3_003_geometry_handle_and_display_attributes_are_queryable_separately() {
    let (mut doc, layer) = layer_document();
    let mut entity = Entity::new("Named", Geometry::Point([1., 2., 3.]), Some(layer));
    entity.attributes.color = Some([9, 8, 7, 255]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let handle = doc.geometry_handle(id).unwrap();
    assert_eq!(doc.resolve_handle(&handle).unwrap().id, id);
    let display = doc.display_attributes(id).unwrap();
    assert_eq!(display.name, "Named");
    assert_eq!(display.color, Some([9, 8, 7, 255]));
    assert!(display.visible && display.selectable && display.editable);
    // A display query never mutates the record it reports.
    assert_eq!(doc.entities[&id].geometry_revision, 1);
}

#[test]
fn l3_003_a_stale_or_dangling_handle_is_never_a_live_object() {
    let (mut doc, _) = layer_document();
    let entity = Entity::new("Handle", Geometry::Point([0.; 3]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let handle = doc.geometry_handle(id).unwrap();
    doc.entities.get_mut(&id).unwrap().geometry_revision = 2;
    assert_eq!(
        doc.resolve_handle(&handle).err(),
        Some(Error::Stale("geometry handle"))
    );
    doc.entities.remove(&id);
    assert_eq!(
        doc.resolve_handle(&handle).err(),
        Some(Error::Stale("geometry handle"))
    );
}

/* ------------------------------- L3-004 ------------------------------- */

#[test]
fn l3_004_layer_record_keeps_its_identity_when_renamed() {
    let (mut doc, id) = layer_document();
    let color = doc.layers[&id].color;
    let mut layer = doc.layers[&id].clone();
    layer.name = "Metals 02".into();
    layer.visible = false;
    layer.locked = true;
    doc.layers.insert(id, layer);
    let stored = &doc.layers[&id];
    assert_eq!(stored.id, id);
    assert_eq!(stored.name, "Metals 02");
    assert_eq!(stored.color, color);
    assert!(!stored.visible && stored.locked);
    doc.validate(&Limits::default()).unwrap();
    assert_eq!(doc.layer_order, vec![id]);
}

#[test]
fn l3_004_layer_with_an_unknown_parent_is_refused() {
    let (mut doc, id) = layer_document();
    doc.layers.get_mut(&id).unwrap().parent = Some(LayerId::new());
    assert_eq!(
        doc.validate(&Limits::default()).err(),
        Some(Error::Missing("layer"))
    );
}

#[test]
fn l3_004_layer_order_must_list_every_layer_exactly_once() {
    let (mut doc, id) = layer_document();
    doc.layer_order.push(id);
    assert_eq!(
        doc.validate(&Limits::default()).err(),
        Some(Error::Invalid("layer order"))
    );
}

/* ------------------------------- L3-005 ------------------------------- */

#[test]
fn l3_005_reassigning_an_object_to_another_layer_keeps_its_entity_id() {
    let mut doc = Document::new("membership");
    let metals = Layer::new("Metals", [1, 2, 3, 255]);
    let gems = Layer::new("Gems", [4, 5, 6, 255]);
    for layer in [&metals, &gems] {
        doc.layers.insert(layer.id, layer.clone());
        doc.layer_order.push(layer.id);
    }
    doc.current_layer = Some(metals.id);
    let entity = Entity::new("Stone", Geometry::Point([0.; 3]), Some(metals.id));
    let id = entity.id;
    doc.entities.insert(id, entity);
    let (mut c, d) = coordinator(doc);
    let mut attributes = c.world().documents[&d].entities[&id].attributes.clone();
    attributes.layer = Some(gems.id);
    let p = c
        .prepare(TransactionRequest::new(
            "layer assignment",
            vec![Mutation::Attributes {
                source: c.world().documents[&d].stamp(id).unwrap(),
                attributes,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let stored = &c.world().documents[&d].entities[&id];
    assert_eq!(stored.id, id);
    assert_eq!(stored.attributes.layer, Some(gems.id));
    assert_eq!(stored.attributes_revision, 2);
}

#[test]
fn l3_005_a_deleted_layer_never_leaves_dangling_membership() {
    let (mut doc, id) = layer_document();
    let entity = Entity::new("Orphan", Geometry::Point([0.; 3]), Some(id));
    let object = entity.id;
    doc.entities.insert(object, entity);
    let saved = doc.layers[&id].clone();
    doc.layers.remove(&id);
    doc.layer_order.retain(|entry| *entry != id);
    doc.current_layer = None;
    assert_eq!(
        doc.validate(&Limits::default()).err(),
        Some(Error::Missing("entity layer")),
        "a layer deletion may not leave a dangling membership"
    );
    // Restoring the layer record restores the membership on the same entity ID.
    doc.layers.insert(id, saved);
    doc.layer_order.push(id);
    doc.validate(&Limits::default()).unwrap();
    assert_eq!(doc.entities[&object].attributes.layer, Some(id));
}

/* ------------------------------- L3-006 ------------------------------- */

#[test]
fn l3_006_a_hidden_layer_hides_and_a_locked_layer_denies_editing() {
    let (mut doc, id) = layer_document();
    let entity = Entity::new("Masked", Geometry::Point([0.; 3]), Some(id));
    let object = entity.id;
    doc.entities.insert(object, entity);
    let visible = doc.permissions(object, false).unwrap();
    assert!(visible.visible && visible.selectable && visible.editable);
    doc.layers.get_mut(&id).unwrap().visible = false;
    let hidden = doc.permissions(object, false).unwrap();
    assert!(!hidden.visible && !hidden.selectable && !hidden.editable);
    doc.layers.get_mut(&id).unwrap().visible = true;
    doc.layers.get_mut(&id).unwrap().locked = true;
    let locked = doc.permissions(object, false).unwrap();
    assert!(locked.visible, "a locked layer may still be drawn");
    assert!(!locked.selectable && !locked.editable);
}

#[test]
fn l3_006_a_locked_layer_refuses_the_write_without_mutating_the_document() {
    let (mut doc, id) = layer_document();
    let entity = Entity::new("Locked", Geometry::Point([0.; 3]), Some(id));
    let object = entity.id;
    doc.entities.insert(object, entity);
    doc.layers.get_mut(&id).unwrap().locked = true;
    let (c, d) = coordinator(doc);
    let before = c.snapshot();
    let mut attributes = c.world().documents[&d].entities[&object].attributes.clone();
    attributes.visible = false;
    let refused = c.prepare(TransactionRequest::new(
        "hide locked",
        vec![Mutation::Attributes {
            source: c.world().documents[&d].stamp(object).unwrap(),
            attributes,
        }],
    ));
    assert_eq!(refused.err(), Some(Error::Permission("source edit")));
    assert_eq!(c.snapshot(), before);
    assert!(c.world().documents[&d].entities[&object].attributes.visible);
}

/* ------------------------------- L3-007 ------------------------------- */

fn grouped_coordinator() -> (TransactionCoordinator, DocumentId, ObjectId, ObjectId) {
    let (mut doc, _) = layer_document();
    let a = Entity::new("A", Geometry::Point([0.; 3]), None);
    let b = Entity::new("B", Geometry::Point([1.; 3]), None);
    let (a_id, b_id) = (a.id, b.id);
    doc.entities.insert(a_id, a);
    doc.entities.insert(b_id, b);
    let (c, d) = coordinator(doc);
    (c, d, a_id, b_id)
}

#[test]
fn l3_007_grouping_and_ungrouping_never_changes_geometry_or_object_ids() {
    let (mut c, d, a, b) = grouped_coordinator();
    let before: Vec<Entity> = c.world().documents[&d].entities.values().cloned().collect();
    let group = Group::new("Pair", vec![a, b]);
    let group_id = group.id;
    let p = c
        .prepare(TransactionRequest::new(
            "group",
            vec![Mutation::GroupCreate {
                document: d,
                expected_revision: 1,
                group: Box::new(group),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(c.world().documents[&d].groups_of(a), vec![group_id]);
    assert_eq!(
        c.world().documents[&d]
            .group(group_id)
            .unwrap()
            .members
            .len(),
        2
    );
    let p = c
        .prepare(TransactionRequest::new(
            "ungroup",
            vec![Mutation::GroupDelete {
                document: d,
                expected_revision: 2,
                group: group_id,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let after: Vec<Entity> = c.world().documents[&d].entities.values().cloned().collect();
    assert_eq!(after, before, "grouping is an overlay, never a merge");
    assert!(c.world().documents[&d].groups.is_empty());
    assert!(c.world().documents[&d].groups_of(a).is_empty());
}

#[test]
fn l3_007_deleting_the_last_member_removes_the_empty_group_and_keeps_the_rest() {
    let (mut c, d, a, b) = grouped_coordinator();
    let pair = Group::new("Pair", vec![a, b]);
    let pair_id = pair.id;
    let single = Group::new("Only", vec![a]);
    let single_id = single.id;
    let p = c
        .prepare(TransactionRequest::new(
            "groups",
            vec![
                Mutation::GroupCreate {
                    document: d,
                    expected_revision: 1,
                    group: Box::new(pair),
                },
                Mutation::GroupCreate {
                    document: d,
                    expected_revision: 1,
                    group: Box::new(single),
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let source = c.world().documents[&d].stamp(a).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "delete A",
            vec![Mutation::Delete { source }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert!(
        !doc.groups.contains_key(&single_id),
        "empty group is removed"
    );
    assert_eq!(doc.group(pair_id).unwrap().members, vec![b]);
    assert!(doc.entities.contains_key(&b));
    c.undo().unwrap();
    let doc = &c.world().documents[&d];
    assert_eq!(doc.group(single_id).unwrap().members, vec![a]);
    assert_eq!(doc.group(pair_id).unwrap().members, vec![a, b]);
}

#[test]
fn l3_007_a_group_can_reference_an_object_created_by_the_same_request() {
    let (mut c, d, a, _b) = grouped_coordinator();
    let fresh = Entity::new("Fresh", Geometry::Point([2.; 3]), None);
    let fresh_id = fresh.id;
    let group = Group::new("New pair", vec![a, fresh_id]);
    let group_id = group.id;
    let p = c
        .prepare(TransactionRequest::new(
            "group with insert",
            vec![
                Mutation::GroupCreate {
                    document: d,
                    expected_revision: 1,
                    group: Box::new(group),
                },
                Mutation::Insert {
                    document: d,
                    entity: fresh,
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(
        c.world().documents[&d].group(group_id).unwrap().members,
        vec![a, fresh_id]
    );
}

#[test]
fn l3_007_unknown_or_duplicate_members_are_refused_atomically() {
    let (c, d, a, _b) = grouped_coordinator();
    let before = c.snapshot();
    let unknown = c.prepare(TransactionRequest::new(
        "dangling group",
        vec![Mutation::GroupCreate {
            document: d,
            expected_revision: 1,
            group: Box::new(Group::new("Ghost", vec![ObjectId::new()])),
        }],
    ));
    assert_eq!(unknown.err(), Some(Error::Missing("group member")));
    let duplicate = c.prepare(TransactionRequest::new(
        "duplicate member",
        vec![Mutation::GroupCreate {
            document: d,
            expected_revision: 1,
            group: Box::new(Group::new("Twice", vec![a, a])),
        }],
    ));
    assert_eq!(
        duplicate.err(),
        Some(Error::Invalid("duplicate group member"))
    );
    let empty = c.prepare(TransactionRequest::new(
        "empty group",
        vec![Mutation::GroupCreate {
            document: d,
            expected_revision: 1,
            group: Box::new(Group::new("Empty", vec![])),
        }],
    ));
    assert_eq!(empty.err(), Some(Error::Invalid("empty group")));
    assert_eq!(c.snapshot(), before);
    assert!(c.world().documents[&d].groups.is_empty());
}

#[test]
fn l3_007_an_undone_group_id_is_never_reallocated() {
    let (mut c, d, a, b) = grouped_coordinator();
    let group = Group::new("Pair", vec![a, b]);
    let p = c
        .prepare(TransactionRequest::new(
            "group",
            vec![Mutation::GroupCreate {
                document: d,
                expected_revision: 1,
                group: Box::new(group.clone()),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let refused = c.prepare(TransactionRequest::new(
        "reallocate group",
        vec![Mutation::GroupCreate {
            document: d,
            expected_revision: 1,
            group: Box::new(group),
        }],
    ));
    assert_eq!(refused.err(), Some(Error::Invalid("allocated group UUID")));
    assert!(c.world().documents[&d].groups.is_empty());
}

/* ------------------------------- L3-008 ------------------------------- */

#[test]
fn l3_008_two_instances_share_one_definition_without_cloning_topology() {
    let (mut c, d, _a, _b) = grouped_coordinator();
    let definition = om9_core::model::Definition::new(
        "Ring",
        vec![
            Entity::new("shank", Geometry::Curve(line_curve()), None),
            Entity::new("stone", Geometry::Point([0., 0., 1.]), None),
        ],
    );
    let definition_id = definition.id;
    let first = {
        let doc = &c.world().documents[&d];
        doc.instance(definition_id, Transform::identity(), None)
    };
    // The definition has to exist before an occurrence can be built from it, so
    // the first request inserts the definition with one occurrence.
    let first = match first {
        Ok(_) => unreachable!("definition does not exist yet"),
        Err(e) => {
            assert_eq!(e, Error::Missing("instance definition"));
            Entity::new("Ring", Geometry::Instance(definition_id), None)
        }
    };
    let (first_id, second_id) = (first.id, ObjectId::new());
    let p = c
        .prepare(TransactionRequest::new(
            "block",
            vec![Mutation::DefinitionInsert {
                document: d,
                expected_revision: 1,
                definition: Box::new(definition),
                instances: vec![first],
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert_eq!(doc.definitions[&definition_id].members.len(), 2);
    assert_eq!(doc.entities.len(), 3, "two objects plus one occurrence");
    assert!(matches!(
        doc.entities[&first_id].geometry,
        Geometry::Instance(id) if id == definition_id
    ));
    // A second occurrence reuses the stored topology and only adds a placement.
    let second = Entity::new("Ring copy", Geometry::Instance(definition_id), None);
    let second = Entity {
        id: second_id,
        placement: Transform::translation([25., 0., 0.]),
        ..second
    };
    let p = c
        .prepare(TransactionRequest::new(
            "second occurrence",
            vec![Mutation::Insert {
                document: d,
                entity: second,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert_eq!(doc.definitions[&definition_id].members.len(), 2);
    assert_eq!(
        doc.member_transform(second_id, &[]).unwrap(),
        Transform::translation([25., 0., 0.])
    );
    assert_eq!(
        doc.member_transform(first_id, &[]).unwrap(),
        Transform::identity()
    );
    // One Undo removes the second occurrence and never touches the definition.
    c.undo().unwrap();
    let doc = &c.world().documents[&d];
    assert!(doc.definitions.contains_key(&definition_id));
    assert!(!doc.entities.contains_key(&second_id));
}

#[test]
fn l3_008_one_undo_removes_the_definition_and_all_of_its_occurrences() {
    let (mut c, d, _a, _b) = grouped_coordinator();
    let definition = om9_core::model::Definition::new(
        "Wide",
        vec![Entity::new("body", Geometry::Point([0.; 3]), None)],
    );
    let definition_id = definition.id;
    let instances = vec![
        Entity::new("occ 1", Geometry::Instance(definition_id), None),
        Entity::new("occ 2", Geometry::Instance(definition_id), None),
    ];
    let (one, two) = (instances[0].id, instances[1].id);
    let entries = c.world().documents[&d].entities.len();
    let p = c
        .prepare(TransactionRequest::new(
            "block",
            vec![Mutation::DefinitionInsert {
                document: d,
                expected_revision: 1,
                definition: Box::new(definition),
                instances,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(c.world().documents[&d].entities.len(), entries + 2);
    c.undo().unwrap();
    let doc = &c.world().documents[&d];
    assert!(!doc.definitions.contains_key(&definition_id));
    assert!(!doc.entities.contains_key(&one) && !doc.entities.contains_key(&two));
    assert_eq!(doc.entities.len(), entries);
}

#[test]
fn l3_008_a_self_referencing_definition_is_refused_without_a_partial_write() {
    let (c, d, _a, _b) = grouped_coordinator();
    let mut definition = om9_core::model::Definition::new(
        "Cycle",
        vec![Entity::new("member", Geometry::Point([0.; 3]), None)],
    );
    let definition_id = definition.id;
    let member = Entity::new("self", Geometry::Instance(definition_id), None);
    let member_id = member.id;
    definition.members.push(member);
    let before = c.snapshot();
    let refused = c.prepare(TransactionRequest::new(
        "cycle",
        vec![Mutation::DefinitionInsert {
            document: d,
            expected_revision: 1,
            definition: Box::new(definition),
            instances: vec![],
        }],
    ));
    assert!(refused.is_err(), "a definition may not contain itself");
    assert_eq!(c.snapshot(), before);
    let doc = &c.world().documents[&d];
    assert!(doc.definitions.is_empty() && !doc.entities.contains_key(&member_id));
}

#[test]
fn l3_008_nested_cycles_are_refused_by_document_validation() {
    let (mut doc, _) = layer_document();
    let outer = om9_core::model::Definition::new(
        "outer",
        vec![Entity::new("inner occ", Geometry::Point([0.; 3]), None)],
    );
    let outer_id = outer.id;
    let inner_id = DefinitionId::new();
    let mut outer = outer;
    outer
        .members
        .push(Entity::new("inner", Geometry::Instance(inner_id), None));
    let inner = om9_core::model::Definition::new(
        "inner",
        vec![Entity::new("outer occ", Geometry::Instance(outer_id), None)],
    );
    let mut inner = inner;
    inner.id = inner_id;
    for member in &mut inner.members {
        member.geometry = Geometry::Instance(outer_id);
    }
    doc.definitions.insert(outer_id, outer);
    doc.definitions.insert(inner_id, inner);
    assert_eq!(
        doc.validate(&Limits::default()).err(),
        Some(Error::Invalid("definition cycle"))
    );
}

/* ------------------------------- L3-009 ------------------------------- */

#[test]
fn l3_009_extension_slots_keep_object_identity_and_bump_attributes_only() {
    let (mut c, d, a, _b) = grouped_coordinator();
    let before = c.world().documents[&d].entities[&a].clone();
    let gem = Extension::new(1, r#"{"shape":"round","carat":0.5}"#).unwrap();
    let future = Extension::new(3, r#"{"vendor":"future"}"#).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "gem",
            vec![Mutation::Extension {
                source: c.world().documents[&d].stamp(a).unwrap(),
                slot: GEM_EXTENSION_SLOT.into(),
                extension: Some(gem),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    // A second slot is a second accepted write, with the stamp that write needs.
    let p = c
        .prepare(TransactionRequest::new(
            "future slot",
            vec![Mutation::Extension {
                source: c.world().documents[&d].stamp(a).unwrap(),
                slot: "vendor.future".into(),
                extension: Some(future),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let after = &c.world().documents[&d].entities[&a];
    assert_eq!(after.id, before.id);
    assert_eq!(after.generation, before.generation);
    assert_eq!(after.geometry_revision, before.geometry_revision);
    assert_eq!(after.attributes_revision, before.attributes_revision + 2);
    assert!(extension_slot_known(GEM_EXTENSION_SLOT));
    let gem = c.world().documents[&d]
        .extension(a, GEM_EXTENSION_SLOT)
        .unwrap()
        .unwrap();
    assert_eq!(gem.version, 1);
    assert!(gem.payload.contains("carat"));
    assert_eq!(
        c.world().documents[&d].unknown_extension_slots(),
        ["vendor.future".to_string()].into_iter().collect()
    );
}

#[test]
fn l3_009_an_unknown_slot_survives_a_json_round_trip() {
    let (mut doc, _) = layer_document();
    let mut entity = Entity::new("Stoned", Geometry::Point([0.; 3]), None);
    entity.extensions.insert(
        "vendor.future".into(),
        Extension::new(2, r#"{"vendor":"future","nested":{"a":[1,2]}}"#).unwrap(),
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let text = serde_json::to_string(&doc).unwrap();
    let reopened: Document = serde_json::from_str(&text).unwrap();
    let slot = reopened.entities[&id].extension("vendor.future").unwrap();
    assert_eq!(slot.version, 2);
    assert_eq!(slot.payload, r#"{"vendor":"future","nested":{"a":[1,2]}}"#);
    assert_eq!(
        reopened.unknown_extension_slots(),
        ["vendor.future".to_string()].into_iter().collect()
    );
}

#[test]
fn l3_009_a_malformed_extension_payload_is_refused() {
    assert_eq!(
        Extension::new(1, "[1,2]"),
        Err(Error::Invalid("extension payload")),
        "a slot payload is a named member set, not a bare array"
    );
    assert_eq!(
        Extension::new(1, "{}"),
        Err(Error::Invalid("empty extension payload"))
    );
    assert_eq!(
        Extension::new(1, "{"),
        Err(Error::Invalid("extension payload"))
    );
    assert_eq!(
        Extension::new(0, r#"{"a":1}"#),
        Err(Error::Invalid("extension version"))
    );
}

#[test]
fn l3_009_a_bare_slot_name_is_not_a_slot() {
    let (c, d, a, _b) = grouped_coordinator();
    let before = c.snapshot();
    let refused = c.prepare(TransactionRequest::new(
        "bad slot",
        vec![Mutation::Extension {
            source: c.world().documents[&d].stamp(a).unwrap(),
            slot: "Gem".into(),
            extension: Some(Extension::new(1, r#"{"shape":"round"}"#).unwrap()),
        }],
    ));
    assert_eq!(refused.err(), Some(Error::Invalid("extension slot")));
    assert_eq!(c.snapshot(), before);
    for valid in ["om9.gem", "vendor.futurething", "a.b"] {
        assert!(om9_core::model::slot_name_valid(valid), "{valid}");
    }
    for invalid in [
        "Gem",
        "",
        ".",
        "Gem.round",
        "om9.",
        "om9..round",
        "om9.gem.",
    ] {
        assert!(!om9_core::model::slot_name_valid(invalid), "{invalid}");
    }
}

#[test]
fn l3_009_a_locked_source_refuses_the_extension_write() {
    let (mut doc, layer) = layer_document();
    let entity = Entity::new("Locked stone", Geometry::Point([0.; 3]), Some(layer));
    let object = entity.id;
    doc.entities.insert(object, entity);
    doc.layers.get_mut(&layer).unwrap().locked = true;
    let (c, d) = coordinator(doc);
    let before = c.snapshot();
    let refused = c.prepare(TransactionRequest::new(
        "gem",
        vec![Mutation::Extension {
            source: c.world().documents[&d].stamp(object).unwrap(),
            slot: GEM_EXTENSION_SLOT.into(),
            extension: Some(Extension::new(1, r#"{"shape":"round"}"#).unwrap()),
        }],
    ));
    assert_eq!(refused.err(), Some(Error::Permission("source edit")));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn l3_009_clearing_a_slot_removes_it_and_reports_it_as_unknown_when_replaced() {
    let (mut c, d, a, _b) = grouped_coordinator();
    let p = c
        .prepare(TransactionRequest::new(
            "gem",
            vec![Mutation::Extension {
                source: c.world().documents[&d].stamp(a).unwrap(),
                slot: GEM_EXTENSION_SLOT.into(),
                extension: Some(Extension::new(1, r#"{"shape":"round"}"#).unwrap()),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(
        c.world().documents[&d]
            .extension(a, GEM_EXTENSION_SLOT)
            .unwrap()
            .is_some()
    );
    let p = c
        .prepare(TransactionRequest::new(
            "clear gem",
            vec![Mutation::Extension {
                source: c.world().documents[&d].stamp(a).unwrap(),
                slot: GEM_EXTENSION_SLOT.into(),
                extension: None,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(
        c.world().documents[&d]
            .extension(a, GEM_EXTENSION_SLOT)
            .unwrap(),
        None
    );
    c.undo().unwrap();
    assert!(
        c.world().documents[&d]
            .extension(a, GEM_EXTENSION_SLOT)
            .unwrap()
            .is_some(),
        "Undo restores the cleared slot on the same object"
    );
}

/* --------------------- accounting regression (L3-008) --------------------- */

#[test]
fn l3_008_a_definition_insert_is_admitted_on_a_layerless_document() {
    // Regression: the reservation charged one node per definition label but no
    // traversal node per member, so a document whose base usage carries no nodes
    // refused a valid block insert with an internal reservation error.
    let mut doc = Document::new("layerless");
    doc.validate(&Limits::default()).unwrap();
    let entity = Entity::new("A", Geometry::Point([0.; 3]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let (mut c, d) = coordinator(doc);
    let definition = om9_core::model::Definition::new(
        "Block",
        vec![
            Entity::new("m1", Geometry::Point([1.; 3]), None),
            Entity::new("m2", Geometry::Point([2.; 3]), None),
        ],
    );
    let definition_id = definition.id;
    let first = Entity::new("occ 1", Geometry::Instance(definition_id), None);
    let p = c
        .prepare(TransactionRequest::new(
            "block",
            vec![Mutation::DefinitionInsert {
                document: d,
                expected_revision: 1,
                definition: Box::new(definition),
                instances: vec![first],
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let document = &c.world().documents[&d];
    assert!(document.definitions.contains_key(&definition_id));
    assert_eq!(document.definitions[&definition_id].members.len(), 2);
    assert_eq!(
        document.entities.len(),
        2,
        "the two definition members are not document objects; only the occurrence is"
    );
}
