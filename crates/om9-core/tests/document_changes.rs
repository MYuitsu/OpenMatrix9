// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-03-011 ordered document change notification.
use om9_core::{
    Error,
    budget::Limits,
    changes::{Change, ChangeBatch, ChangeCause, diff},
    geometry::Geometry,
    identity::*,
    model::{Document, Entity, Extension, Feature, FeatureKind, GEM_EXTENSION_SLOT, Group, Layer},
    transaction::*,
};

fn layer_document() -> (Document, LayerId) {
    let mut doc = Document::new("changes");
    let layer = Layer::new("Metals", [34, 153, 135, 255]);
    let id = layer.id;
    doc.layers.insert(id, layer);
    doc.layer_order.push(id);
    doc.current_layer = Some(id);
    (doc, id)
}

#[test]
fn adding_an_entity_and_a_group_is_one_ordered_batch_each_with_a_new_revision() {
    let (doc, layer) = layer_document();
    let d = doc.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let before = c.world().documents[&d].clone();
    let entity = Entity::new("Stone", Geometry::Point([1., 2., 3.]), Some(layer));
    let entity_id = entity.id;
    let p = c
        .prepare(TransactionRequest::new(
            "insert",
            vec![Mutation::Insert {
                document: d,
                entity: entity.clone(),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let after_insert = c.world().documents[&d].clone();
    let batch = ChangeBatch::between(&before, &after_insert, ChangeCause::Edit)
        .unwrap()
        .expect("a real edit produces a batch");
    assert_eq!(batch.document, d);
    assert_eq!(batch.from_revision, before.revision);
    assert_eq!(batch.to_revision, after_insert.revision);
    assert!(batch.to_revision > batch.from_revision);
    assert_eq!(batch.changes, vec![Change::EntityAdded(entity_id)]);
    assert_eq!(
        ChangeBatch::between(&after_insert, &after_insert, ChangeCause::Edit),
        Ok(None),
        "an idle document is never asked to redraw"
    );
    let group = Group::new("Pair", vec![entity_id]);
    let group_id = group.id;
    let p = c
        .prepare(TransactionRequest::new(
            "group",
            vec![Mutation::GroupCreate {
                document: d,
                expected_revision: after_insert.revision,
                group: Box::new(group),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let after_group = c.world().documents[&d].clone();
    assert_eq!(
        diff(&after_insert, &after_group).unwrap(),
        vec![Change::GroupCreated(group_id)]
    );
}

#[test]
fn an_idle_document_produces_no_batch() {
    let (doc, _) = layer_document();
    assert_eq!(
        ChangeBatch::between(&doc, &doc, ChangeCause::Edit),
        Ok(None)
    );
    let mut after = doc.clone();
    after.revision += 1;
    assert_eq!(
        ChangeBatch::between(&doc, &after, ChangeCause::Edit).unwrap(),
        None,
        "a revision bump without a content change is not a change batch"
    );
}

#[test]
fn deleting_an_entity_reports_removal_and_group_pruning() {
    let (mut doc, layer) = layer_document();
    let a = Entity::new("A", Geometry::Point([0.; 3]), Some(layer));
    let b = Entity::new("B", Geometry::Point([1.; 3]), Some(layer));
    let (a_id, b_id) = (a.id, b.id);
    doc.entities.insert(a_id, a);
    doc.entities.insert(b_id, b.clone());
    let pair = Group::new("Pair", vec![a_id, b_id]);
    let single = Group::new("Only", vec![a_id]);
    let (pair_id, single_id) = (pair.id, single.id);
    doc.groups.insert(pair_id, pair);
    doc.groups.insert(single_id, single);
    let before = doc.clone();
    let mut after = doc;
    after.entities.remove(&a_id);
    // What the Delete mutation applies at the owner: the surviving group keeps
    // its identity and loses the member, an emptied group disappears.
    let pair = after.groups.get_mut(&pair_id).unwrap();
    pair.members.retain(|member| *member != a_id);
    pair.revision += 1;
    after.groups.remove(&single_id);
    after.revision += 1;
    let changes = diff(&before, &after).unwrap();
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0], Change::EntityRemoved(a_id));
    assert!(
        changes[1..].contains(&Change::GroupRemoved(single_id)),
        "an emptied group is removed: {changes:?}"
    );
    assert!(changes[1..].contains(&Change::GroupModified(pair_id)));
}

#[test]
fn an_extension_edit_is_reported_as_an_extension_change() {
    let (mut doc, layer) = layer_document();
    let entity = Entity::new("Stone", Geometry::Point([0.; 3]), Some(layer));
    let id = entity.id;
    doc.entities.insert(id, entity);
    let before = doc.clone();
    doc.entities.get_mut(&id).unwrap().extensions.insert(
        GEM_EXTENSION_SLOT.into(),
        Extension::new(1, r#"{"shape":"round"}"#).unwrap(),
    );
    doc.entities.get_mut(&id).unwrap().attributes_revision += 1;
    doc.revision += 1;
    assert_eq!(
        diff(&before, &doc).unwrap(),
        vec![Change::EntityModified(id), Change::ExtensionChanged(id)]
    );
}

#[test]
fn feature_and_group_lifecycle_changes_are_ordered_and_deterministic() {
    let (mut doc, layer) = layer_document();
    let entity = Entity::new("A", Geometry::Point([0.; 3]), Some(layer));
    let id = entity.id;
    doc.entities.insert(id, entity);
    let before = doc.clone();
    let group = Group::new("G", vec![id]);
    let group_id = group.id;
    let feature = Feature::new("Note", FeatureKind::Annotation, vec![id]);
    let feature_id = feature.id;
    doc.groups.insert(group_id, group);
    doc.features.insert(feature_id, feature);
    let after = doc.clone();
    let first = diff(&before, &after).unwrap();
    assert_eq!(
        first,
        vec![
            Change::GroupCreated(group_id),
            Change::FeatureAdded(feature_id)
        ]
    );
    assert_eq!(diff(&before, &after).unwrap(), first);
    assert_eq!(
        diff(&after, &before).unwrap(),
        vec![
            Change::GroupRemoved(group_id),
            Change::FeatureRemoved(feature_id)
        ]
    );
}

#[test]
fn a_diff_between_two_documents_is_refused() {
    let (doc, _) = layer_document();
    let other = Document::new("other");
    assert_eq!(diff(&doc, &other), Err(Error::Invalid("change document")));
}
