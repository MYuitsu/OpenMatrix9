// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-03 wave W4: gumball drag and occurrence-array kernel ported from the legacy bridge.
//!
//! The oracles are geometric: a constrained handle may only move along what it constrains, a
//! rotation about a point keeps that point fixed, a scale along an axis keeps the other axes
//! untouched, and an array produces the expected occurrence positions.
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Point3, Transform},
    gumball::{
        Axis, Drag, Frame, Handle, MAX_ARRAY_OCCURRENCES, Plane, Session, polar_array,
        rectangular_array, rotation_about, scaling_along, transform,
    },
    identity::ObjectId,
    model::{Document, Entity, Layer},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

fn apply(transform: Transform, point: Point3) -> Point3 {
    transform.point(point).unwrap()
}

/* ------------------------- handle model ------------------------- */

#[test]
fn handle_codes_round_trip_like_the_legacy_wire() {
    let handles = [
        Handle::MoveAxis(Axis::X),
        Handle::MoveAxis(Axis::Y),
        Handle::MoveAxis(Axis::Z),
        Handle::MovePlane(Plane::XY),
        Handle::MovePlane(Plane::YZ),
        Handle::MovePlane(Plane::ZX),
        Handle::MoveFree,
        Handle::Rotate(Axis::X),
        Handle::Rotate(Axis::Y),
        Handle::Rotate(Axis::Z),
        Handle::Scale(Axis::X),
        Handle::Scale(Axis::Y),
        Handle::Scale(Axis::Z),
    ];
    for (index, handle) in handles.iter().enumerate() {
        assert_eq!(handle.code() as usize, index, "{handle:?}");
        assert_eq!(Handle::from_code(handle.code()), Some(*handle));
    }
    assert_eq!(Handle::from_code(13), None);
    assert_eq!(Plane::XY.normal_axis(), Axis::Z);
    assert_eq!(Plane::YZ.normal_axis(), Axis::X);
    assert_eq!(Plane::ZX.normal_axis(), Axis::Y);
}

/* ------------------------- constrained drag ------------------------- */

#[test]
fn an_axis_handle_keeps_only_the_component_along_its_axis() {
    let frame = Frame::world();
    let drag = [3.0, 5.0, 7.0];
    let moved = transform(&frame, Handle::MoveAxis(Axis::X), Drag::WorldDelta(drag)).unwrap();
    assert!(close(apply(moved, [0.0; 3]), [3.0, 0.0, 0.0], 1e-12));
    let moved = transform(&frame, Handle::MoveAxis(Axis::Z), Drag::WorldDelta(drag)).unwrap();
    assert!(close(apply(moved, [0.0; 3]), [0.0, 0.0, 7.0], 1e-12));
    // A free handle keeps the whole delta.
    let moved = transform(&frame, Handle::MoveFree, Drag::WorldDelta(drag)).unwrap();
    assert!(close(apply(moved, [0.0; 3]), drag, 1e-12));
}

#[test]
fn a_plane_handle_drops_the_normal_component() {
    let frame = Frame::world();
    let drag = [3.0, 5.0, 7.0];
    let moved = transform(&frame, Handle::MovePlane(Plane::XY), Drag::WorldDelta(drag)).unwrap();
    assert!(close(apply(moved, [0.0; 3]), [3.0, 5.0, 0.0], 1e-12));
    let moved = transform(&frame, Handle::MovePlane(Plane::YZ), Drag::WorldDelta(drag)).unwrap();
    assert!(close(apply(moved, [0.0; 3]), [0.0, 5.0, 7.0], 1e-12));
    let moved = transform(&frame, Handle::MovePlane(Plane::ZX), Drag::WorldDelta(drag)).unwrap();
    assert!(close(apply(moved, [0.0; 3]), [3.0, 0.0, 7.0], 1e-12));
}

#[test]
fn a_rotated_frame_axis_drag_follows_its_axis() {
    // A frame rotated 90 degrees about Z: its x axis is world +Y.
    let frame = Frame::new([0.0; 3], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]).unwrap();
    let moved = transform(
        &frame,
        Handle::MoveAxis(Axis::X),
        Drag::WorldDelta([3.0, 5.0, 0.0]),
    )
    .unwrap();
    assert!(close(apply(moved, [0.0; 3]), [0.0, 5.0, 0.0], 1e-12));
}

#[test]
fn a_handle_refuses_the_wrong_kind_of_drag_input() {
    let frame = Frame::world();
    assert_eq!(
        transform(&frame, Handle::MoveAxis(Axis::X), Drag::Angle(1.0)),
        Err(Error::Invalid("gumball drag input"))
    );
    assert_eq!(
        transform(
            &frame,
            Handle::Rotate(Axis::Y),
            Drag::WorldDelta([1.0, 0.0, 0.0])
        ),
        Err(Error::Invalid("gumball drag input"))
    );
    assert_eq!(
        transform(
            &frame,
            Handle::Scale(Axis::Y),
            Drag::WorldDelta([1.0, 0.0, 0.0])
        ),
        Err(Error::Invalid("gumball drag input"))
    );
    assert_eq!(
        transform(
            &frame,
            Handle::MoveFree,
            Drag::WorldDelta([f64::NAN, 0.0, 0.0])
        ),
        Err(Error::Invalid("nonfinite coordinate"))
    );
}

/* ------------------------- rotation / scale ------------------------- */

#[test]
fn rotation_about_a_point_keeps_that_point_fixed() {
    let origin = [2.0, 0.0, 0.0];
    let rotate = rotation_about(origin, [0.0, 0.0, 1.0], std::f64::consts::FRAC_PI_2).unwrap();
    assert!(close(apply(rotate, origin), origin, 1e-12));
    // (3,0,0) turns a quarter turn about (2,0,0) to (2,1,0).
    assert!(close(
        apply(rotate, [3.0, 0.0, 0.0]),
        [2.0, 1.0, 0.0],
        1e-12
    ));
    // A full turn is the identity to numerical precision.
    let full = rotation_about(origin, [0.0, 0.0, 1.0], std::f64::consts::TAU).unwrap();
    assert!(close(apply(full, [3.0, 1.0, 5.0]), [3.0, 1.0, 5.0], 1e-12));
}

#[test]
fn scaling_along_an_axis_keeps_the_other_axes_and_the_origin() {
    let origin = [1.0, 0.0, 0.0];
    let scale = scaling_along(origin, [1.0, 0.0, 0.0], 2.0).unwrap();
    assert!(close(apply(scale, origin), origin, 1e-12));
    // (3,0,0) is 2 away from the origin along x: it moves to 1 + 2*2 = 5.
    assert!(close(apply(scale, [3.0, 0.0, 0.0]), [5.0, 0.0, 0.0], 1e-12));
    // The y and z components are untouched.
    assert!(close(apply(scale, [3.0, 4.0, 5.0]), [5.0, 4.0, 5.0], 1e-12));
    // A rotate handle with an angle produces a rotation about the frame origin.
    let frame = Frame::new([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).unwrap();
    let turned = transform(
        &frame,
        Handle::Rotate(Axis::Z),
        Drag::Angle(std::f64::consts::PI),
    )
    .unwrap();
    assert!(close(
        apply(turned, [3.0, 0.0, 0.0]),
        [-1.0, 0.0, 0.0],
        1e-12
    ));
    let scaled = transform(&frame, Handle::Scale(Axis::X), Drag::Factor(3.0)).unwrap();
    assert!(close(
        apply(scaled, [2.0, 1.0, 0.0]),
        [4.0, 1.0, 0.0],
        1e-12
    ));
}

#[test]
fn rotation_and_scale_refuse_bad_parameters() {
    assert_eq!(
        rotation_about([0.0; 3], [0.0, 0.0, 0.0], 1.0),
        Err(Error::Invalid("gumball rotation axis"))
    );
    assert_eq!(
        rotation_about([0.0; 3], [0.0, 0.0, 1.0], f64::NAN),
        Err(Error::Invalid("gumball rotation angle"))
    );
    assert_eq!(
        scaling_along([0.0; 3], [1.0, 0.0, 0.0], 0.0),
        Err(Error::Invalid("gumball scale factor"))
    );
    assert_eq!(
        scaling_along([0.0; 3], [1.0, 0.0, 0.0], -2.0),
        Err(Error::Invalid("gumball scale factor"))
    );
    let bad = Frame {
        origin: [0.0; 3],
        x: [1.0, 0.0, 0.0],
        y: [1.0, 0.0, 0.0],
        z: [0.0, 0.0, 1.0],
    };
    assert_eq!(bad.validate(), Err(Error::Invalid("gumball frame")));
}

/* ------------------------- occurrence arrays ------------------------- */

#[test]
fn a_rectangular_array_places_every_occurrence_on_the_frame_grid() {
    let frame = Frame::world();
    let occurrences = rectangular_array(&frame, [3, 2], [10.0, 4.0], false).unwrap();
    assert_eq!(occurrences.len(), 5, "the source is not an occurrence");
    let origins: Vec<Point3> = occurrences
        .iter()
        .map(|transform| apply(*transform, [0.0; 3]))
        .collect();
    for expected in [
        [10.0, 0.0, 0.0],
        [20.0, 0.0, 0.0],
        [0.0, 4.0, 0.0],
        [10.0, 4.0, 0.0],
        [20.0, 4.0, 0.0],
    ] {
        assert!(
            origins.iter().any(|origin| close(*origin, expected, 1e-12)),
            "missing {expected:?} in {origins:?}"
        );
    }
    // Including the source keeps the identity first.
    let all = rectangular_array(&frame, [2, 2], [10.0, 4.0], true).unwrap();
    assert_eq!(all.len(), 4);
    assert!(close(apply(all[0], [0.0; 3]), [0.0; 3], 1e-12));
}

#[test]
fn a_polar_array_spreads_occurrences_around_the_frame_axis() {
    let frame = Frame::world();
    let occurrences = polar_array(&frame, 4, std::f64::consts::TAU, false).unwrap();
    assert_eq!(occurrences.len(), 3);
    for (index, transform) in occurrences.iter().enumerate() {
        let quarter = (index + 1) as f64 * std::f64::consts::FRAC_PI_2;
        let expected = [10.0 * quarter.cos(), 10.0 * quarter.sin(), 0.0];
        assert!(close(apply(*transform, [10.0, 0.0, 0.0]), expected, 1e-9));
    }
    // An open spread ends exactly on the requested angle.
    let half = polar_array(&frame, 3, std::f64::consts::PI, true).unwrap();
    assert_eq!(half.len(), 3);
    assert!(close(
        apply(half[0], [10.0, 0.0, 0.0]),
        [10.0, 0.0, 0.0],
        1e-12
    ));
    assert!(close(
        apply(half[2], [10.0, 0.0, 0.0]),
        [-10.0, 0.0, 0.0],
        1e-9
    ));
}

#[test]
fn arrays_refuse_impossible_counts_and_spacing() {
    let frame = Frame::world();
    assert_eq!(
        rectangular_array(&frame, [0, 2], [1.0, 1.0], false),
        Err(Error::Invalid("gumball array count"))
    );
    assert_eq!(
        rectangular_array(&frame, [1, 1], [1.0, 1.0], false),
        Err(Error::Invalid("gumball array count"))
    );
    assert_eq!(
        rectangular_array(&frame, [MAX_ARRAY_OCCURRENCES, 2], [1.0, 1.0], false),
        Err(Error::Invalid("gumball array count"))
    );
    assert_eq!(
        rectangular_array(&frame, [2, 2], [f64::NAN, 1.0], false),
        Err(Error::Invalid("gumball array spacing"))
    );
    assert_eq!(
        polar_array(&frame, 1, std::f64::consts::TAU, false),
        Err(Error::Invalid("gumball array count"))
    );
    assert_eq!(
        polar_array(&frame, 4, 0.0, false),
        Err(Error::Invalid("gumball array angle"))
    );
}

/* ------------------------- drag session ------------------------- */

fn session_document() -> (Document, Vec<(ObjectId, Point3)>) {
    let mut document = Document::new("gumball session");
    let layer = Layer::new("Metals", [1, 2, 3, 255]);
    document.layers.insert(layer.id, layer.clone());
    document.layer_order.push(layer.id);
    document.current_layer = Some(layer.id);
    let mut targets = Vec::new();
    for (name, point) in [("A", [1.0, 0.0, 0.0]), ("B", [2.0, 0.0, 0.0])] {
        let entity = Entity::new(name, Geometry::Point(point), Some(layer.id));
        targets.push((entity.id, point));
        document.entities.insert(entity.id, entity);
    }
    (document, targets)
}

fn coordinator(document: Document) -> TransactionCoordinator {
    TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap()
}

#[test]
fn a_session_composes_successive_drags_in_world_order() {
    let (document, targets) = session_document();
    let ids: Vec<ObjectId> = targets.iter().map(|(id, _)| *id).collect();
    let mut session = Session::begin(Frame::world(), ids, false).unwrap();
    session
        .drag(Handle::MoveAxis(Axis::X), Drag::WorldDelta([5.0, 0.0, 0.0]))
        .unwrap();
    session
        .drag(Handle::MoveAxis(Axis::Y), Drag::WorldDelta([0.0, 3.0, 0.0]))
        .unwrap();
    assert_eq!(session.drag_count(), 2);
    let moved = apply(session.accumulated(), [0.0; 3]);
    assert!(close(moved, [5.0, 3.0, 0.0], 1e-12), "{moved:?}");
    // Reset drops the accumulated drag but keeps the targets.
    session.reset();
    assert_eq!(session.drag_count(), 0);
    assert!(close(
        apply(session.accumulated(), [1.0, 1.0, 1.0]),
        [1.0, 1.0, 1.0],
        1e-12
    ));
    assert_eq!(session.targets().len(), 2);
    let _ = document;
}

#[test]
fn a_session_refuses_an_empty_duplicate_or_nil_target_set() {
    let (_, targets) = session_document();
    let first = targets[0].0;
    assert_eq!(
        Session::begin(Frame::world(), vec![], false),
        Err(Error::Invalid("gumball targets"))
    );
    assert_eq!(
        Session::begin(Frame::world(), vec![first, first], false),
        Err(Error::Invalid("gumball targets"))
    );
    assert!(
        Session::begin(
            Frame::world(),
            vec![ObjectId::new(), ObjectId::new()],
            false
        )
        .is_ok()
    );
    let nil: ObjectId = serde_json::from_str("\"00000000-0000-0000-0000-000000000000\"").unwrap();
    assert_eq!(
        Session::begin(Frame::world(), vec![nil], false),
        Err(Error::Invalid("gumball targets"))
    );
}

#[test]
fn a_move_session_rewrites_placement_only_and_is_one_undo_step() {
    let (document, targets) = session_document();
    let ids: Vec<ObjectId> = targets.iter().map(|(id, _)| *id).collect();
    let mut session = Session::begin(Frame::world(), ids.clone(), false).unwrap();
    session
        .drag(
            Handle::MoveAxis(Axis::X),
            Drag::WorldDelta([10.0, 0.0, 0.0]),
        )
        .unwrap();
    let mutations = session.move_mutations(&document).unwrap();
    assert_eq!(mutations.len(), 2);
    assert!(matches!(mutations[0], Mutation::Placement { .. }));
    let mut coo = coordinator(document.clone());
    let document_id = document.id;
    let before_undo = coo.undo_len();
    let before_geometry: Vec<Geometry> = ids
        .iter()
        .map(|id| {
            coo.world().documents[&document_id].entities[id]
                .geometry
                .clone()
        })
        .collect();
    let prepared = coo
        .prepare(TransactionRequest::new("gumball move", mutations))
        .unwrap();
    coo.commit(prepared).unwrap();
    assert_eq!(
        coo.undo_len(),
        before_undo + 1,
        "one gesture is one Undo entry"
    );
    for (index, id) in ids.iter().enumerate() {
        let entity = &coo.world().documents[&document_id].entities[id];
        // Geometry is untouched: only the placement moved.
        assert_eq!(entity.geometry, before_geometry[index]);
        let world_point = entity.placement.point(targets[index].1).unwrap();
        let expected = [targets[index].1[0] + 10.0, 0.0, 0.0];
        assert!(close(world_point, expected, 1e-12), "{world_point:?}");
    }
    coo.undo().unwrap();
    let entity = &coo.world().documents[&document_id].entities[&ids[0]];
    assert!(close(
        entity.placement.point(targets[0].1).unwrap(),
        targets[0].1,
        1e-12
    ));
}

#[test]
fn a_copy_session_inserts_clones_with_fresh_identities() {
    let (document, targets) = session_document();
    let ids: Vec<ObjectId> = targets.iter().map(|(id, _)| *id).collect();
    let mut session = Session::begin(Frame::world(), ids.clone(), true).unwrap();
    // A copy session refuses the move plan and vice versa.
    assert!(matches!(
        session.move_mutations(&document),
        Err(Error::Invalid("gumball session mode"))
    ));
    session
        .drag(Handle::MoveAxis(Axis::Z), Drag::WorldDelta([0.0, 0.0, 4.0]))
        .unwrap();
    let mutations = session.copy_mutations(&document).unwrap();
    assert_eq!(mutations.len(), 2);
    let mut coo = coordinator(document.clone());
    let document_id = document.id;
    let before = coo.world().documents[&document_id].entities.len();
    let before_undo = coo.undo_len();
    let prepared = coo
        .prepare(TransactionRequest::new("gumball copy", mutations))
        .unwrap();
    coo.commit(prepared).unwrap();
    let after = coo.world().documents[&document_id].entities.len();
    assert_eq!(after, before + 2);
    assert_eq!(coo.undo_len(), before_undo + 1);
    // The clones are new objects with the source geometry and a lifted placement.
    let clones: Vec<&Entity> = coo.world().documents[&document_id]
        .entities
        .values()
        .filter(|entity| !ids.contains(&entity.id))
        .collect();
    assert_eq!(clones.len(), 2);
    for clone in clones {
        assert!(
            matches!(clone.geometry, Geometry::Point(_)),
            "a copy keeps the source geometry"
        );
        assert!((clone.placement.point([0.0; 3]).unwrap()[2] - 4.0).abs() < 1e-12);
    }
    // A non-copy session cannot produce a copy plan.
    let mut moving = Session::begin(Frame::world(), ids, false).unwrap();
    moving
        .drag(Handle::MoveFree, Drag::WorldDelta([1.0, 0.0, 0.0]))
        .unwrap();
    assert!(matches!(
        moving.copy_mutations(&document),
        Err(Error::Invalid("gumball session mode"))
    ));
}
