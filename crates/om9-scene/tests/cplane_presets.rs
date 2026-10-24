// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-02-001/002/003`: CPlane presets and affine frame construction.
use om9_core::Error;
use om9_scene::camera::{CPlane, CPlanePreset};
use om9_scene::workspace::{ViewId, Workspace};

fn close(a: [f64; 3], b: [f64; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 1e-12)
}

#[test]
fn top_front_and_right_presets_have_the_documented_axes_and_handedness() {
    let cases = [
        (CPlanePreset::Top, [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]),
        (
            CPlanePreset::Front,
            [1., 0., 0.],
            [0., 0., 1.],
            [0., -1., 0.],
        ),
        (
            CPlanePreset::Right,
            [0., 1., 0.],
            [0., 0., 1.],
            [1., 0., 0.],
        ),
    ];
    for (preset, x, y, normal) in cases {
        let plane = CPlane::preset(preset);
        assert_eq!(plane.origin, [0.; 3]);
        assert!(close(plane.x, x), "{preset:?} x {:?}", plane.x);
        assert!(close(plane.y, y), "{preset:?} y {:?}", plane.y);
        assert!(
            close(plane.normal(), normal),
            "{preset:?} normal {:?}",
            plane.normal()
        );
        plane.validate().unwrap();
    }
}

#[test]
fn invalid_presets_and_collinear_points_are_typed_errors() {
    assert_eq!(
        CPlane::new([0.; 3], [0.; 3], [0., 1., 0.]),
        Err(Error::Invalid("orthonormal CPlane"))
    );
    assert_eq!(
        CPlane::new([0.; 3], [1., 0., 0.], [2., 0., 0.]),
        Err(Error::Invalid("orthonormal CPlane"))
    );
    assert_eq!(
        CPlane::from_three_points([0.; 3], [1., 0., 0.], [2., 0., 0.]),
        Err(Error::Invalid("CPlane frame points"))
    );
    assert_eq!(
        CPlane::from_three_points([0.; 3], [0.; 3], [0., 1., 0.]),
        Err(Error::Invalid("zero direction"))
    );
}

#[test]
fn three_point_frame_uses_the_first_axis_and_orthonormalizes_the_second() {
    let plane =
        CPlane::from_three_points([10., 20., 0.], [15., 20., 0.], [10., 23., 0.]).expect("frame");
    assert_eq!(plane.origin, [10., 20., 0.]);
    assert!(close(plane.x, [1., 0., 0.]));
    assert!(close(plane.y, [0., 1., 0.]));
    assert!(close(plane.normal(), [0., 0., 1.]));
    assert_eq!(plane.point([2., 3., 0.]).unwrap(), [12., 23., 0.]);
}

#[test]
fn rotation_about_the_normal_keeps_the_frame_right_handed() {
    let plane = CPlane::world_xy()
        .rotated(std::f64::consts::FRAC_PI_2)
        .expect("rotate");
    assert!(close(plane.x, [0., 1., 0.]));
    assert!(close(plane.y, [-1., 0., 0.]));
    assert!(close(plane.normal(), [0., 0., 1.]));
    // Local (5,0,0) on a plane rotated 90 degrees about Z is world (0,5,0).
    assert!(close(plane.point([5., 0., 0.]).unwrap(), [0., 5., 0.]));
    assert_eq!(plane.revision, 2);
    assert_eq!(
        plane.rotated(f64::NAN),
        Err(Error::Invalid("CPlane rotation"))
    );
}

#[test]
fn moving_the_origin_does_not_change_the_axes_or_existing_world_geometry() {
    let plane = CPlane::preset(CPlanePreset::Top)
        .with_origin([10., 20., 0.])
        .expect("origin");
    assert_eq!(plane.origin, [10., 20., 0.]);
    assert_eq!(plane.x, [1., 0., 0.]);
    assert_eq!(plane.y, [0., 1., 0.]);
    assert_eq!(plane.point([0., 0., 0.]).unwrap(), [10., 20., 0.]);
}

#[test]
fn the_four_default_views_use_the_documented_construction_planes() {
    let workspace = Workspace::new();
    assert_eq!(
        workspace.views[&ViewId::LookingDown].cplane,
        CPlane::preset(CPlanePreset::Top)
    );
    assert_eq!(
        workspace.views[&ViewId::Perspective].cplane,
        CPlane::preset(CPlanePreset::Top)
    );
    assert_eq!(
        workspace.views[&ViewId::ThroughFinger].cplane,
        CPlane::preset(CPlanePreset::Front)
    );
    assert_eq!(
        workspace.views[&ViewId::SideView].cplane,
        CPlane::preset(CPlanePreset::Right)
    );
}
