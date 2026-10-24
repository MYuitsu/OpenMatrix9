// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused tests for `OM9-L2-02-06` interactive transform sessions.
//!
//! The oracles are analytic: a move by a reference vector, a 90-degree rotation about Z, a
//! 1D/2D/3D scale factor, and a SetPt/align projection. The session itself is pure, so the
//! preview and cancel semantics are checked without a window.
use om9_core::{
    Error,
    geometry::Transform,
    gumball::{Axis, Plane},
    transform_session::{
        AlignMode, AlignRequest, MoveSession, RotateSession, ScaleAxes, ScaleSession,
        align_point_to_axis, align_point_to_plane, align_rotation, align_transform,
        set_point_transform,
    },
};

fn point(x: f64, y: f64, z: f64) -> [f64; 3] {
    [x, y, z]
}

fn close(a: [f64; 3], b: [f64; 3], tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

fn applied(matrix: &Transform, point: [f64; 3]) -> [f64; 3] {
    matrix.point(point).unwrap()
}

#[test]
fn m03_02_06_move_preview_copy_and_cancel() {
    let mut session = MoveSession::begin(point(1.0, 2.0, 3.0), false).unwrap();
    // Before the reference the preview is identity.
    assert_eq!(session.preview().unwrap(), Transform::identity());
    session.set_reference(point(4.0, 6.0, 8.0)).unwrap();
    let preview = session.preview().unwrap();
    assert!(close(
        applied(&preview, point(1.0, 2.0, 3.0)),
        point(4.0, 6.0, 8.0),
        1e-12
    ));
    assert!(close(
        applied(&preview, point(-1.0, 0.0, 1.0)),
        point(2.0, 4.0, 6.0),
        1e-12
    ));
    assert!(!session.copy());
    session.set_copy(true);
    assert!(session.copy());
    session.cancel();
    assert_eq!(session.preview().unwrap(), Transform::identity());
    assert_eq!(session.reference(), None);
    assert!(MoveSession::begin([f64::NAN, 0.0, 0.0], false).is_err());
    assert!(session.set_reference([f64::INFINITY, 0.0, 0.0]).is_err());
}

#[test]
fn m03_02_06_rotate_uses_the_reference_and_copy_option() {
    let mut session = RotateSession::begin([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], true).unwrap();
    assert_eq!(session.angle().unwrap(), 0.0);
    session.set_reference(point(1.0, 0.0, 0.0)).unwrap();
    session.set_current(point(0.0, 1.0, 0.0)).unwrap();
    assert!((session.angle().unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    let preview = session.preview().unwrap();
    assert!(close(
        applied(&preview, point(1.0, 0.0, 0.0)),
        point(0.0, 1.0, 0.0),
        1e-12
    ));
    assert!(close(
        applied(&preview, point(0.0, 2.0, 3.0)),
        point(-2.0, 0.0, 3.0),
        1e-12
    ));
    assert!(session.copy());
    session.cancel();
    assert_eq!(session.preview().unwrap(), Transform::identity());
    // A point on the axis has no reference direction and is refused.
    session.set_reference(point(0.0, 0.0, 2.0)).unwrap();
    session.set_current(point(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(
        session.angle(),
        Err(Error::Invalid("rotate reference vector"))
    );
}

#[test]
fn m03_02_06_scale_1d_2d_and_3d_apply_the_right_axes() {
    let cases = [
        (
            ScaleAxes::OneD(Axis::X),
            point(2.0, 0.0, 0.0),
            point(4.0, 0.0, 0.0),
            [1.0, 2.0, 3.0],
            [2.0, 2.0, 3.0],
        ),
        (
            ScaleAxes::TwoD(Plane::XY),
            point(2.0, 2.0, 0.0),
            point(4.0, 4.0, 0.0),
            [1.0, 2.0, 3.0],
            [2.0, 4.0, 3.0],
        ),
        (
            ScaleAxes::ThreeD,
            point(1.0, 1.0, 1.0),
            point(2.0, 2.0, 2.0),
            [1.0, 2.0, 3.0],
            [2.0, 4.0, 6.0],
        ),
        (
            ScaleAxes::TwoD(Plane::YZ),
            point(0.0, 2.0, 2.0),
            point(0.0, 4.0, 4.0),
            [1.0, 2.0, 3.0],
            [1.0, 4.0, 6.0],
        ),
        (
            ScaleAxes::TwoD(Plane::ZX),
            point(2.0, 0.0, 2.0),
            point(4.0, 0.0, 4.0),
            [1.0, 2.0, 3.0],
            [2.0, 2.0, 6.0],
        ),
        (
            ScaleAxes::OneD(Axis::Y),
            point(0.0, 2.0, 0.0),
            point(0.0, 4.0, 0.0),
            [1.0, 2.0, 3.0],
            [1.0, 4.0, 3.0],
        ),
        (
            ScaleAxes::OneD(Axis::Z),
            point(0.0, 0.0, 2.0),
            point(0.0, 0.0, 4.0),
            [1.0, 2.0, 3.0],
            [1.0, 2.0, 6.0],
        ),
    ];
    for (axes, reference, current, source, expected) in cases {
        let mut session = ScaleSession::begin([0.0; 3], axes, false).unwrap();
        assert_eq!(session.factor().unwrap(), 1.0);
        session.set_reference(reference).unwrap();
        session.set_current(current).unwrap();
        assert!((session.factor().unwrap() - 2.0).abs() < 1e-12, "{axes:?}");
        let preview = session.preview().unwrap();
        assert!(
            close(applied(&preview, source), expected, 1e-12),
            "{axes:?}: {expected:?}"
        );
    }
    // A zero reference distance has no finite factor.
    let mut session = ScaleSession::begin([0.0; 3], ScaleAxes::OneD(Axis::X), false).unwrap();
    session.set_reference([0.0; 3]).unwrap();
    session.set_current(point(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(
        session.factor(),
        Err(Error::Invalid("scale reference distance"))
    );
}

#[test]
fn m03_02_06_setpoint_and_align_project_to_world_or_plane() {
    let set = set_point_transform(
        point(1.0, 2.0, 3.0),
        point(4.0, 6.0, 8.0),
        [true, true, false],
    )
    .unwrap();
    assert!(close(
        applied(&set, point(1.0, 2.0, 3.0)),
        point(4.0, 6.0, 3.0),
        1e-12
    ));

    let plane =
        align_point_to_plane(point(1.0, 1.0, 5.0), [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]).unwrap();
    assert!(close(
        applied(&plane, point(1.0, 1.0, 5.0)),
        point(1.0, 1.0, 1.0),
        1e-12
    ));

    let axis = align_point_to_axis(point(2.0, 3.0, 4.0), [0.0; 3], [1.0, 0.0, 0.0]).unwrap();
    assert!(close(
        applied(&axis, point(2.0, 3.0, 4.0)),
        point(2.0, 0.0, 0.0),
        1e-12
    ));

    assert!(align_point_to_plane(point(0.0, 0.0, 0.0), [0.0; 3], [0.0; 3]).is_err());
    assert!(set_point_transform([f64::NAN, 0.0, 0.0], [0.0; 3], [true; 3]).is_err());
}

#[test]
fn m03_02_06_preview_is_one_pure_transform_and_survives_copy() {
    // The same preview can be applied to every selected object and composed for a second
    // gesture: this is what the UI preview reads before the single commit.
    let mut first = MoveSession::begin([0.0; 3], true).unwrap();
    first.set_reference(point(5.0, 0.0, 0.0)).unwrap();
    let mut second = RotateSession::begin([5.0, 0.0, 0.0], [0.0, 0.0, 1.0], true).unwrap();
    second.set_reference(point(6.0, 0.0, 0.0)).unwrap();
    second.set_current(point(5.0, 1.0, 0.0)).unwrap();
    // Apply the move first, then the rotation around its destination.
    let combined = second
        .preview()
        .unwrap()
        .compose(first.preview().unwrap())
        .unwrap();
    // (0,0,0) -> (5,0,0) inside the second rotation about (5,0,0) -> unchanged at (5,0,0).
    assert!(close(
        applied(&combined, [0.0; 3]),
        point(5.0, 0.0, 0.0),
        1e-12
    ));
    assert!(first.copy() && second.copy());
}

#[test]
fn m03_02_06_integration_stress_many_sessions_stay_exact() {
    for index in 0..200 {
        let offset = index as f64 * 0.5;
        let mut move_session = MoveSession::begin([offset, 0.0, 0.0], index % 2 == 0).unwrap();
        move_session
            .set_reference([offset + 1.0, 1.0, 1.0])
            .unwrap();
        let preview = move_session.preview().unwrap();
        assert!(close(
            applied(&preview, [offset, 0.0, 0.0]),
            [offset + 1.0, 1.0, 1.0],
            1e-12
        ));
        let mut scale_session = ScaleSession::begin([0.0; 3], ScaleAxes::ThreeD, false).unwrap();
        scale_session.set_reference(point(1.0, 1.0, 1.0)).unwrap();
        scale_session.set_current(point(1.5, 1.5, 1.5)).unwrap();
        assert!((scale_session.factor().unwrap() - 1.5).abs() < 1e-12);
    }
}

#[test]
fn m03_02_06_receipt() {
    let mut move_session = MoveSession::begin([1.0, 2.0, 3.0], true).unwrap();
    move_session.set_reference([4.0, 6.0, 8.0]).unwrap();
    let mut rotate = RotateSession::begin([0.0; 3], [0.0, 0.0, 1.0], false).unwrap();
    rotate.set_reference([1.0, 0.0, 0.0]).unwrap();
    rotate.set_current([0.0, 1.0, 0.0]).unwrap();
    let mut scale = ScaleSession::begin([0.0; 3], ScaleAxes::ThreeD, false).unwrap();
    scale.set_reference([1.0, 1.0, 1.0]).unwrap();
    scale.set_current([2.0, 2.0, 2.0]).unwrap();
    let setpoint =
        set_point_transform([1.0, 2.0, 3.0], [4.0, 6.0, 8.0], [true, true, false]).unwrap();
    let receipt = serde_json::json!({
        "status": "measured",
        "move_preview": move_session.preview().unwrap().0,
        "rotate_angle_rad": rotate.angle().unwrap(),
        "scale_factor": scale.factor().unwrap(),
        "setpoint_maps": applied(&setpoint, [1.0, 2.0, 3.0]),
        "copy_option": move_session.copy(),
    });
    println!("{receipt}");
}

#[test]
fn m03_02_06_align_rotates_and_scales_not_only_translates() {
    let rotation = align_rotation(point(1.0, 0.0, 0.0), point(0.0, 1.0, 0.0), [0.0; 3]).unwrap();
    assert!(close(
        applied(&rotation, point(1.0, 0.0, 0.0)),
        point(0.0, 1.0, 0.0),
        1e-12
    ));

    let rotate = AlignRequest::rotate(
        point(1.0, 2.0, 3.0),
        point(4.0, 5.0, 6.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
    );
    let transform = align_transform(&rotate).unwrap();
    assert!(close(
        applied(&transform, point(1.0, 2.0, 3.0)),
        point(4.0, 5.0, 6.0),
        1e-12
    ));
    assert!(close(
        applied(&transform, point(2.0, 2.0, 3.0)),
        point(4.0, 6.0, 6.0),
        1e-12
    ));

    let scale = AlignRequest::scale(point(0.0, 0.0, 0.0), point(10.0, 0.0, 0.0), 2.0, 4.0);
    let transform = align_transform(&scale).unwrap();
    assert!(close(
        applied(&transform, point(1.0, 0.0, 0.0)),
        point(12.0, 0.0, 0.0),
        1e-12
    ));
    assert!(close(
        applied(&transform, point(0.0, 1.0, 0.0)),
        point(10.0, 2.0, 0.0),
        1e-12
    ));

    let both = AlignRequest::rotate_scale(
        point(0.0, 0.0, 0.0),
        point(10.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        2.0,
        4.0,
    );
    let transform = align_transform(&both).unwrap();
    assert!(close(
        applied(&transform, point(1.0, 0.0, 0.0)),
        point(10.0, 2.0, 0.0),
        1e-12
    ));
    assert!(close(
        applied(&transform, point(0.0, 1.0, 0.0)),
        point(8.0, 0.0, 0.0),
        1e-12
    ));
    assert_eq!(both.mode, AlignMode::RotateScale);

    // Missing axes/extents and degenerate extents are refused.
    let missing_axis = AlignRequest {
        mode: AlignMode::Rotate,
        source_axis: None,
        ..AlignRequest::translate([0.0; 3], [1.0, 0.0, 0.0])
    };
    assert_eq!(
        align_transform(&missing_axis),
        Err(Error::Missing("align source axis"))
    );
    let missing_extent = AlignRequest {
        mode: AlignMode::Scale,
        source_extent: Some(2.0),
        target_extent: None,
        ..AlignRequest::translate([0.0; 3], [1.0, 0.0, 0.0])
    };
    assert_eq!(
        align_transform(&missing_extent),
        Err(Error::Missing("align target extent"))
    );
    let zero_extent = AlignRequest::scale([0.0; 3], [1.0, 0.0, 0.0], 0.0, 2.0);
    assert_eq!(
        align_transform(&zero_extent),
        Err(Error::Invalid("align extent"))
    );
    assert!(align_rotation(point(0.0, 0.0, 1.0), point(0.0, 0.0, -1.0), [0.0; 3]).is_ok());
}
