// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gumball interaction at the controller boundary: a gesture on one or many selected objects becomes
//! exactly one transaction, and Undo puts every object back.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    gumball::{Axis, Drag, Frame, Handle, Session},
    identity::ObjectId,
    model::Document,
};

fn controller() -> Controller {
    Controller::new(Document::new("gumball"), Limits::default()).expect("controller")
}

/// Creates `count` point objects along X and returns their ids in creation order.
fn points(c: &mut Controller, count: usize) -> Vec<ObjectId> {
    let mut ids = Vec::with_capacity(count);
    for index in 0..count {
        let before: Vec<ObjectId> = c.document().entities.keys().copied().collect();
        c.submit("Point").expect("Point command");
        c.submit(&format!("{},0,0", index * 10))
            .expect("point coordinates");
        let after: Vec<ObjectId> = c.document().entities.keys().copied().collect();
        let created = after
            .into_iter()
            .find(|id| !before.contains(id))
            .expect("the Point command published one object");
        ids.push(created);
    }
    ids
}

/// World centre of one object, taken from the same scene geometry the gumball uses: a Point or Line
/// carries its coordinates in the buffers, so its placement alone says nothing about where it is.
fn centre_of(c: &mut Controller, id: ObjectId) -> [f64; 3] {
    c.select_root(Some(id), false).expect("select one");
    let centre = c.gumball_pivot().expect("pivot for one object").origin;
    c.clear_selection();
    centre
}

fn select(c: &mut Controller, ids: &[ObjectId]) {
    c.select_root(Some(ids[0]), false).expect("first selection");
    for id in &ids[1..] {
        c.select_root(Some(*id), true).expect("extend selection");
    }
}

#[test]
fn moving_two_selected_objects_is_one_journal_entry() {
    let mut c = controller();
    let ids = points(&mut c, 2);
    select(&mut c, &ids);
    assert_eq!(c.gumball_targets().len(), 2);
    let frame = c
        .gumball_pivot()
        .expect("a pivot for a non-empty selection");
    assert_eq!(
        frame.origin,
        [5.0, 0.0, 0.0],
        "the pivot is the selection centre"
    );

    let undo_before = c.coordinator().undo_len();
    let mut session = Session::begin(frame, c.gumball_targets(), false).expect("session");
    session
        .drag(Handle::MoveFree, Drag::WorldDelta([3.0, 4.0, 0.0]))
        .expect("drag");
    c.commit_gumball(&session, "Move").expect("commit");
    assert_eq!(
        c.coordinator().undo_len(),
        undo_before + 1,
        "one gesture, one entry"
    );
    for (index, id) in ids.iter().enumerate() {
        let moved = centre_of(&mut c, *id);
        let expected = [index as f64 * 10.0 + 3.0, 4.0, 0.0];
        assert!(
            (moved[0] - expected[0]).abs() < 1e-9 && (moved[1] - expected[1]).abs() < 1e-9,
            "object {index} moved to {moved:?}, expected {expected:?}"
        );
    }

    c.undo().expect("undo");
    for (index, id) in ids.iter().enumerate() {
        let restored = centre_of(&mut c, *id);
        assert!(
            (restored[0] - index as f64 * 10.0).abs() < 1e-9 && restored[1].abs() < 1e-9,
            "undo restores object {index}: {restored:?}"
        );
    }
    c.redo().expect("redo");
    let again = centre_of(&mut c, ids[0]);
    assert!(
        (again[0] - 3.0).abs() < 1e-9 && (again[1] - 4.0).abs() < 1e-9,
        "redo reapplies: {again:?}"
    );
}

#[test]
fn rotating_turns_the_selection_about_its_centre() {
    let mut c = controller();
    let ids = points(&mut c, 2);
    select(&mut c, &ids);
    let frame = c.gumball_pivot().expect("pivot");
    let mut session = Session::begin(frame, c.gumball_targets(), false).expect("session");
    session
        .drag(
            Handle::Rotate(Axis::Z),
            Drag::Angle(std::f64::consts::FRAC_PI_2),
        )
        .expect("drag");
    c.commit_gumball(&session, "Rotate").expect("commit");
    // The point at (0,0,0) is five to the left of the pivot; a quarter turn puts it five below.
    let first = centre_of(&mut c, ids[0]);
    assert!((first[0] - 5.0).abs() < 1e-9, "x after rotation: {first:?}");
    assert!((first[1] + 5.0).abs() < 1e-9, "y after rotation: {first:?}");
    let second = centre_of(&mut c, ids[1]);
    assert!(
        (second[0] - 5.0).abs() < 1e-9 && (second[1] - 5.0).abs() < 1e-9,
        "{second:?}"
    );
}

#[test]
fn scaling_doubles_the_distance_from_the_pivot() {
    let mut c = controller();
    let ids = points(&mut c, 2);
    select(&mut c, &ids);
    let frame = c.gumball_pivot().expect("pivot");
    let mut session = Session::begin(frame, c.gumball_targets(), false).expect("session");
    session
        .drag(Handle::Scale(Axis::X), Drag::Factor(2.0))
        .expect("drag");
    c.commit_gumball(&session, "Scale").expect("commit");
    let first = centre_of(&mut c, ids[0]);
    let second = centre_of(&mut c, ids[1]);
    assert!(
        (first[0] + 5.0).abs() < 1e-9,
        "the left object moved to -5: {first:?}"
    );
    assert!(
        (second[0] - 15.0).abs() < 1e-9,
        "the right object moved to 15: {second:?}"
    );
}

#[test]
fn a_gesture_without_motion_writes_nothing() {
    let mut c = controller();
    let ids = points(&mut c, 1);
    select(&mut c, &ids);
    let frame = c.gumball_pivot().expect("pivot");
    let session = Session::begin(frame, c.gumball_targets(), false).expect("session");
    let undo_before = c.coordinator().undo_len();
    c.commit_gumball(&session, "Move").expect("commit");
    assert_eq!(
        c.coordinator().undo_len(),
        undo_before,
        "a click is not an edit"
    );
}

#[test]
fn the_pivot_is_none_without_a_selection() {
    let mut c = controller();
    let _ = points(&mut c, 1);
    assert!(c.gumball_targets().is_empty());
    assert_eq!(c.gumball_pivot(), None::<Frame>);
    c.select_root(
        Some(*c.document().entities.keys().next().expect("one object")),
        false,
    )
    .expect("selection");
    assert_eq!(c.gumball_targets().len(), 1);
    let pivot = c.gumball_pivot().expect("pivot");
    assert_eq!(pivot.origin, [0.0, 0.0, 0.0]);
}
