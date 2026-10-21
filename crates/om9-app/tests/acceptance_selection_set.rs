// SPDX-License-Identifier: LGPL-2.1-or-later
//! Acceptance oracles for the selection set, from `OM9-L3-01-07-002`:
//!
//! * positive: "Click ID A rồi Ctrl click ID B trả set {A,B}, toggle A trả {B}".
//! * negative: "ID đã xóa hoặc layer khóa bị loại và không lưu stale ID".
//! * rule: "add/remove/toggle không thay document history, deterministic ordering".
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    identity::ObjectId,
    model::{Document, Entity},
};

fn points(count: usize) -> Document {
    let mut document = Document::new("selection set");
    for index in 0..count {
        let entity = Entity::new(
            format!("point {index}"),
            Geometry::Point([index as f64 * 3.0, 0.0, 0.0]),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

fn ids(controller: &Controller) -> Vec<ObjectId> {
    let mut ids: Vec<ObjectId> = controller.document().entities.keys().copied().collect();
    ids.sort();
    ids
}

#[test]
fn click_a_then_ctrl_click_b_gives_both_and_toggling_a_leaves_b() {
    let mut controller = Controller::new(points(2), Limits::default()).expect("controller");
    let ids = ids(&controller);
    controller
        .select_root(Some(ids[0]), false)
        .expect("click A");
    assert_eq!(controller.selection_state().objects.len(), 1);
    controller.toggle_selection(ids[1]).expect("ctrl click B");
    assert_eq!(
        controller.selection_state().objects.clone(),
        ids.iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        "the set is exactly A and B"
    );
    controller.toggle_selection(ids[0]).expect("toggle A off");
    assert_eq!(controller.selection_state().objects.len(), 1);
    assert!(controller.selected().contains(&ids[1]));
    // Deterministic ordering: the set reads in identifier order, not insertion order.
    let listed: Vec<ObjectId> = controller
        .selection_state()
        .objects
        .iter()
        .copied()
        .collect();
    let mut sorted = listed.clone();
    sorted.sort();
    assert_eq!(listed, sorted);
}

#[test]
fn selecting_never_writes_document_history() {
    let mut controller = Controller::new(points(2), Limits::default()).expect("controller");
    let ids = ids(&controller);
    let undo_before = controller.coordinator().undo_len();
    let before = controller.document().revision;
    controller.select_root(Some(ids[0]), false).expect("select");
    controller.toggle_selection(ids[1]).expect("add");
    controller.select_objects(&ids).expect("replace");
    controller.clear_selection();
    assert_eq!(
        controller.coordinator().undo_len(),
        undo_before,
        "no journal entry"
    );
    assert_eq!(
        controller.document().revision,
        before,
        "no document revision change"
    );
}

#[test]
fn a_locked_or_hidden_object_is_refused_and_not_stored() {
    let mut document = Document::new("locked");
    let mut locked = Entity::new("locked", Geometry::Point([0.0; 3]), None);
    locked.attributes.locked = true;
    let locked_id = locked.id;
    document.entities.insert(locked_id, locked);
    let mut hidden = Entity::new("hidden", Geometry::Point([1.0, 0.0, 0.0]), None);
    hidden.attributes.hidden = true;
    let hidden_id = hidden.id;
    document.entities.insert(hidden_id, hidden);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    let undo_before = controller.coordinator().undo_len();
    assert!(
        controller.select_root(Some(locked_id), false).is_err(),
        "locked is refused"
    );
    assert!(
        controller.select_objects(&[locked_id]).is_err(),
        "a locked batch is refused whole"
    );
    assert!(
        controller.toggle_selection(hidden_id).is_err(),
        "hidden is refused"
    );
    assert!(controller.selected().is_empty(), "no stale id is stored");
    assert_eq!(controller.coordinator().undo_len(), undo_before);
}

#[test]
fn an_identifier_that_no_longer_exists_is_refused() {
    let mut controller = Controller::new(points(1), Limits::default()).expect("controller");
    let unknown = ObjectId::new();
    let undo_before = controller.coordinator().undo_len();
    assert!(controller.select_root(Some(unknown), false).is_err());
    assert!(controller.select_objects(&[unknown]).is_err());
    assert!(controller.toggle_selection(unknown).is_err());
    assert!(controller.selected().is_empty());
    assert_eq!(controller.coordinator().undo_len(), undo_before);
}

#[test]
fn a_batch_with_one_bad_id_is_refused_whole() {
    let mut controller = Controller::new(points(2), Limits::default()).expect("controller");
    let ids = ids(&controller);
    // Select one object first, then ask for a batch that mixes it with an unknown id.
    controller
        .select_root(Some(ids[0]), false)
        .expect("select one");
    let before = controller.selection_state().objects.clone();
    assert!(
        controller
            .select_objects(&[ids[1], ObjectId::new()])
            .is_err()
    );
    assert_eq!(
        controller.selection_state().objects,
        before,
        "a refused batch leaves the old selection untouched"
    );
}

#[test]
fn twenty_selection_round_trips_are_identical() {
    // L4-01-07-002-05 style repetition: the same sequence of add/toggle must leave the same set every time.
    let mut controller = Controller::new(points(3), Limits::default()).expect("controller");
    let ids = ids(&controller);
    for run in 1..=20 {
        controller.clear_selection();
        controller.select_root(Some(ids[0]), false).expect("A");
        controller.toggle_selection(ids[1]).expect("B");
        controller.toggle_selection(ids[2]).expect("C");
        controller.toggle_selection(ids[1]).expect("toggle B off");
        let listed: Vec<ObjectId> = controller
            .selection_state()
            .objects
            .iter()
            .copied()
            .collect();
        assert_eq!(listed, vec![ids[0], ids[2]], "run {run}");
    }
}
