// SPDX-License-Identifier: LGPL-2.1-or-later
//! Grips (control/edit points) are picked like objects: a click or a window picks grips, the
//! gumball sits on the picked grips, and dragging moves every picked grip in one Undo entry.
use om9_app::session::{Controller, GripPick};
use om9_core::{
    budget::Limits,
    gumball::{Drag, Handle, Session},
    identity::ObjectId,
    model::Document,
    top11::Top11Action,
};

fn polyline_with_grips() -> (Controller, ObjectId) {
    let mut c = Controller::new(Document::new("grips"), Limits::default()).expect("controller");
    c.submit("OM9-CURVE-001").expect("Polyline");
    for point in ["0,0,0", "10,0,0", "10,10,0", "0,10,0"] {
        c.submit(point).expect("vertex");
    }
    c.submit("").expect("finish");
    let id = *c.document().entities.keys().next().expect("polyline");
    c.select_objects(&[id]).expect("select");
    c.top11_invoke(Top11Action::ControlPoints)
        .expect("control points on");
    (c, id)
}

#[test]
fn picked_grips_carry_the_gumball_and_move_together() {
    let (mut c, _) = polyline_with_grips();
    let handles = c.top11_grip_handles().expect("grips");
    assert_eq!(handles.len(), 4);
    assert!(!c.grips_selected());

    // Window-pick the two right-hand grips.
    c.select_grips(&[handles[1], handles[2]], GripPick::Replace);
    assert!(c.grips_selected());
    assert_eq!(c.selected_grips().len(), 2);
    let frame = c.gumball_pivot().expect("gumball on grips");
    assert_eq!(frame.origin, [10.0, 5.0, 0.0], "pivot at the picked grips");

    // A gumball move gesture moves only the picked grips, as one Undo entry.
    let undo_before = c.coordinator().undo_len();
    let mut session = Session::begin(frame, c.gumball_targets(), false).expect("session");
    session
        .drag(Handle::MoveFree, Drag::WorldDelta([5.0, 0.0, 0.0]))
        .expect("drag");
    c.preview_gumball(session.accumulated()).expect("preview");
    assert_eq!(c.grip_preview_curves().len(), 1, "edited curve previews");
    c.commit_gumball(&session, "Move").expect("commit");
    c.clear_gumball_preview();
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    let moved: Vec<_> = c
        .top11_grip_handles()
        .expect("grips")
        .into_iter()
        .map(|handle| handle.point)
        .collect();
    assert_eq!(
        moved,
        vec![
            [0.0, 0.0, 0.0],
            [15.0, 0.0, 0.0],
            [15.0, 10.0, 0.0],
            [0.0, 10.0, 0.0]
        ]
    );
    // The grips stay picked after the edit, so the next gesture continues from them.
    assert_eq!(c.selected_grips().len(), 2);
}

#[test]
fn grip_picks_add_remove_and_reset() {
    let (mut c, _) = polyline_with_grips();
    let handles = c.top11_grip_handles().expect("grips");
    c.select_grips(&[handles[0]], GripPick::Replace);
    c.select_grips(&[handles[3]], GripPick::Add);
    assert_eq!(c.selected_grips().len(), 2);
    c.select_grips(&[handles[0]], GripPick::Remove);
    let left = c.selected_grips();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].index, handles[3].index);
    // Turning grips off forgets the picks; the object gumball comes back.
    c.top11_invoke(Top11Action::ControlPoints)
        .expect("control points off");
    assert!(!c.grips_selected());
    assert!(c.selected_grips().is_empty());
}

#[test]
fn several_grips_move_in_one_transaction() {
    let (mut c, _) = polyline_with_grips();
    let handles = c.top11_grip_handles().expect("grips");
    let undo_before = c.coordinator().undo_len();
    c.top11_move_grips(&[
        (handles[0], [0.0, -2.0, 0.0]),
        (handles[1], [10.0, -2.0, 0.0]),
    ])
    .expect("move grips");
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    let points: Vec<_> = c
        .top11_grip_handles()
        .expect("grips")
        .into_iter()
        .map(|handle| handle.point)
        .collect();
    assert_eq!(points[0], [0.0, -2.0, 0.0]);
    assert_eq!(points[1], [10.0, -2.0, 0.0]);
    assert_eq!(points[2], [10.0, 10.0, 0.0]);
}
