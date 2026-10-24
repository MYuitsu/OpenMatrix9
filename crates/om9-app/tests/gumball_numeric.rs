// SPDX-License-Identifier: LGPL-2.1-or-later
//! Numeric gumball edits typed in the command line: `MoveX 5`, `RotateZ 30`, `Scale 2`.
//!
//! A point authored by the `Point` command keeps its coordinates in the geometry and has an identity
//! placement, so every assertion here reads the **world** position (`placement * geometry`), the same thing
//! the viewport draws.
use om9_app::session::Controller;
use om9_core::{budget::Limits, model::Document};

fn controller_with_points() -> (Controller, Vec<om9_core::identity::ObjectId>) {
    let mut controller =
        Controller::new(Document::new("numeric"), Limits::default()).expect("controller");
    for x in ["0,0,0", "10,0,0"] {
        controller.submit("Point").expect("Point");
        controller.submit(x).expect("coordinates");
    }
    let ids = controller.document().entities.keys().copied().collect();
    (controller, ids)
}

/// Where the object actually is: its geometry through its placement.
fn world(controller: &Controller, id: om9_core::identity::ObjectId) -> [f64; 3] {
    let entity = &controller.document().entities[&id];
    let om9_core::geometry::Geometry::Point(point) = entity.geometry else {
        panic!("expected a point");
    };
    entity.placement.point(point).expect("placed point")
}

/// World positions of both points, ordered by X.
fn ends(controller: &Controller, ids: &[om9_core::identity::ObjectId]) -> ([f64; 3], [f64; 3]) {
    let mut points: Vec<[f64; 3]> = ids.iter().map(|id| world(controller, *id)).collect();
    points.sort_by(|left, right| left[0].total_cmp(&right[0]));
    (points[0], points[1])
}

#[test]
fn a_numeric_move_is_one_transaction() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    let before: Vec<[f64; 3]> = ids.iter().map(|id| world(&controller, *id)).collect();
    let undo_before = controller.coordinator().undo_len();
    controller.submit("MoveX 5").expect("command");
    assert_eq!(
        controller.coordinator().undo_len(),
        undo_before + 1,
        "one entry, like a drag"
    );
    for (id, before) in ids.iter().zip(before) {
        let after = world(&controller, *id);
        assert!(
            (after[0] - before[0] - 5.0).abs() < 1e-9,
            "moved along X: {before:?} -> {after:?}"
        );
        assert!((after[1] - before[1]).abs() < 1e-9 && (after[2] - before[2]).abs() < 1e-9);
    }
    controller.undo().expect("undo");
    let (left, right) = ends(&controller, &ids);
    assert!(
        left[0].abs() < 1e-9 && (right[0] - 10.0).abs() < 1e-9,
        "undo restores: {left:?} {right:?}"
    );
}

#[test]
fn a_numeric_rotation_and_scale_turn_about_the_pivot() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    // Pivot is (5,0,0); a quarter turn sends the ends to (5,-5,0) and (5,5,0).
    controller.submit("RotateZ 90").expect("rotate");
    let mut points: Vec<[f64; 3]> = ids.iter().map(|id| world(&controller, *id)).collect();
    points.sort_by(|left, right| left[1].total_cmp(&right[1]));
    assert!(
        (points[0][0] - 5.0).abs() < 1e-9 && (points[0][1] + 5.0).abs() < 1e-9,
        "{points:?}"
    );
    assert!(
        (points[1][0] - 5.0).abs() < 1e-9 && (points[1][1] - 5.0).abs() < 1e-9,
        "{points:?}"
    );
    controller.undo().expect("undo");
    controller.submit("Scale 2").expect("scale");
    let (left, right) = ends(&controller, &ids);
    assert!(
        (left[0] + 5.0).abs() < 1e-9,
        "uniform scale about the pivot: {left:?}"
    );
    assert!((right[0] - 15.0).abs() < 1e-9, "{right:?}");
}

#[test]
fn numeric_commands_without_a_selection_are_refused() {
    let (mut controller, _) = controller_with_points();
    let undo_before = controller.coordinator().undo_len();
    assert!(controller.submit("MoveX 5").is_err(), "nothing selected");
    assert!(controller.submit("Scale 2").is_err());
    assert_eq!(controller.coordinator().undo_len(), undo_before);
}

#[test]
fn a_malformed_numeric_command_changes_nothing() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    let before: Vec<[f64; 3]> = ids.iter().map(|id| world(&controller, *id)).collect();
    let undo_before = controller.coordinator().undo_len();
    assert!(controller.submit("Move 1,2").is_err());
    assert!(
        controller.submit("Scale 0").is_err(),
        "a zero factor is refused"
    );
    assert_eq!(controller.coordinator().undo_len(), undo_before);
    for (id, before) in ids.iter().zip(before) {
        assert_eq!(world(&controller, *id), before);
    }
}

#[test]
fn nan_and_empty_numeric_inputs_commit_nothing_and_keep_the_gumball() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    assert!(
        controller.gumball_pivot().is_some(),
        "the gumball is active before typing"
    );
    let placement = controller.document().entities[&ids[0]].placement;
    let before = world(&controller, ids[0]);
    let undo_before = controller.coordinator().undo_len();
    for input in ["MoveX NaN", "MoveX", "RotateZ inf", "Scale -2"] {
        assert!(
            controller.submit(input).is_err(),
            "{input:?} must be refused"
        );
    }
    assert_eq!(
        controller.coordinator().undo_len(),
        undo_before,
        "nothing was written"
    );
    assert_eq!(
        controller.document().entities[&ids[0]].placement,
        placement,
        "the object is unchanged"
    );
    assert!(
        controller.gumball_pivot().is_some() && !controller.selected().is_empty(),
        "a parse failure keeps the gizmo and the selection active"
    );
    // A valid typed value then commits exactly one entry.
    controller.submit("MoveX 12.5").expect("exact value");
    assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
    let moved = world(&controller, ids[0]);
    assert!((moved[0] - before[0] - 12.5).abs() < 1e-9, "{moved:?}");
}

#[test]
fn a_twelve_point_five_millimetre_handle_input_is_exact_and_one_entry() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    let before: Vec<[f64; 3]> = ids.iter().map(|id| world(&controller, *id)).collect();
    let undo_before = controller.coordinator().undo_len();
    controller.submit("MoveX 12.5").expect("MoveX");
    assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
    for (id, before) in ids.iter().zip(before) {
        let after = world(&controller, *id);
        assert!(
            (after[0] - before[0] - 12.5).abs() < 1e-9
                && (after[1] - before[1]).abs() < 1e-9
                && (after[2] - before[2]).abs() < 1e-9,
            "{before:?} -> {after:?}"
        );
    }
}
