// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-03-010 selection state document contract.
use om9_core::{
    Error,
    budget::Limits,
    geometry::Geometry,
    identity::*,
    model::{Document, Entity, Layer},
    selection::{SelectionState, SubshapePart, SubshapeRef},
};

fn document() -> (Document, ObjectId, ObjectId) {
    let mut doc = Document::new("selection");
    let layer = Layer::new("Metals", [1, 2, 3, 255]);
    doc.layers.insert(layer.id, layer.clone());
    doc.layer_order.push(layer.id);
    doc.current_layer = Some(layer.id);
    let a = Entity::new("A", Geometry::Point([0.; 3]), Some(layer.id));
    let b = Entity::new("B", Geometry::Point([1.; 3]), Some(layer.id));
    let (a_id, b_id) = (a.id, b.id);
    doc.entities.insert(a_id, a);
    doc.entities.insert(b_id, b);
    (doc, a_id, b_id)
}

#[test]
fn selection_is_typed_identity_and_survives_a_stable_document() {
    let (doc, a, b) = document();
    let mut selection = SelectionState::new(&doc);
    assert!(selection.is_empty());
    selection.select(&doc, a).unwrap();
    selection.select(&doc, b).unwrap();
    assert!(selection.contains(a) && selection.contains(b));
    assert_eq!(selection.ordered(), vec![a.min(b), a.max(b)]);
    selection.validate(&doc).unwrap();
    selection.deselect(a);
    assert!(!selection.contains(a) && selection.contains(b));
    assert_eq!(selection.revision, 4);
}

#[test]
fn selecting_an_unknown_object_is_refused() {
    let (doc, a, _b) = document();
    let mut selection = SelectionState::new(&doc);
    assert_eq!(
        selection.select(&doc, ObjectId::new()),
        Err(Error::Missing("selection object"))
    );
    selection.select(&doc, a).unwrap();
    assert_eq!(selection.objects.len(), 1);
}

#[test]
fn a_selection_from_another_document_generation_is_stale() {
    let (mut doc, a, _b) = document();
    let mut selection = SelectionState::new(&doc);
    selection.select(&doc, a).unwrap();
    // A new epoch is what another publication produces, and it is reported
    // separately from a different document so a caller can tell them apart.
    doc.epoch += 1;
    assert_eq!(
        selection.validate(&doc),
        Err(Error::Stale("selection epoch"))
    );
    assert_eq!(
        selection.select(&doc, a),
        Err(Error::Stale("selection epoch"))
    );
    let other = Document::new("other");
    let mut selection = SelectionState::new(&doc);
    selection.select(&doc, a).unwrap();
    assert_eq!(
        selection.validate(&other),
        Err(Error::Stale("selection document"))
    );
}

#[test]
fn deleting_an_object_invalidates_its_selection_entry() {
    let (mut doc, a, b) = document();
    let mut selection = SelectionState::new(&doc);
    selection.select(&doc, a).unwrap();
    selection.select(&doc, b).unwrap();
    selection
        .select_subshape(
            &doc,
            SubshapeRef {
                object: a,
                part: SubshapePart::Edge,
                index: 2,
            },
        )
        .unwrap();
    doc.entities.remove(&a);
    assert!(selection.prune(&doc), "the view must be told to refresh");
    assert!(!selection.contains(a));
    assert!(selection.contains(b));
    assert!(selection.subshapes.is_empty());
    selection.validate(&doc).unwrap();
    assert!(!selection.prune(&doc), "a second prune changes nothing");
}

#[test]
fn a_stale_list_index_is_never_resolved_as_a_neighbour() {
    let (doc, a, b) = document();
    let mut selection = SelectionState::new(&doc);
    selection.select(&doc, a).unwrap();
    selection.select(&doc, b).unwrap();
    let published = selection.ordered();
    // The index is only valid for the exact published list.
    assert_eq!(
        selection.resolve_index(&published, 0).unwrap(),
        published[0]
    );
    let mut reordered = published.clone();
    reordered.reverse();
    assert_eq!(
        selection.resolve_index(&reordered, 0),
        Err(Error::Stale("selection index"))
    );
    let shorter = &published[..1];
    assert_eq!(
        selection.resolve_index(shorter, 0),
        Err(Error::Stale("selection index"))
    );
    assert_eq!(
        selection.resolve_index(&published, 9),
        Err(Error::Stale("selection index"))
    );
}

#[test]
fn a_subshape_reference_must_name_a_live_object() {
    let (doc, a, _b) = document();
    let mut selection = SelectionState::new(&doc);
    assert_eq!(
        selection.select_subshape(
            &doc,
            SubshapeRef {
                object: ObjectId::new(),
                part: SubshapePart::Face,
                index: 0,
            }
        ),
        Err(Error::Missing("selection object"))
    );
    selection
        .select_subshape(
            &doc,
            SubshapeRef {
                object: a,
                part: SubshapePart::Face,
                index: 0,
            },
        )
        .unwrap();
    assert_eq!(selection.subshapes.len(), 1);
    selection.deselect(a);
    assert!(
        selection.subshapes.is_empty(),
        "dropping the object drops its parts"
    );
}

#[test]
fn the_selection_only_describes_a_document_that_validates() {
    let (doc, a, _b) = document();
    let mut selection = SelectionState::new(&doc);
    selection.select(&doc, a).unwrap();
    doc.validate(&Limits::default()).unwrap();
    selection.validate(&doc).unwrap();
}
