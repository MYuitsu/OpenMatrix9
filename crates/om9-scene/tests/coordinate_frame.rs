// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-08-007: a CPlane is the local frame for typed coordinate input.
use om9_core::{
    Error,
    coords::{CoordinateInput, LocalFrame, LocalPoint, WorldPoint, parse_local, resolve},
    geometry::Unit,
};
use om9_scene::camera::CPlane;

#[test]
fn a_shifted_cplane_maps_local_coordinates_to_world() {
    let plane = CPlane::new([10., 20., 30.], [0., 1., 0.], [0., 0., 1.]).unwrap();
    let local = LocalPoint::try_new([1., 2., 3.]).unwrap();
    let world = resolve(CoordinateInput::Local(local), Some(&plane)).unwrap();
    assert_eq!(world.xyz(), [13., 21., 32.]);
}

#[test]
fn to_local_is_the_inverse_of_to_world_for_a_shifted_and_rotated_plane() {
    for (origin, x, y) in [
        ([0., 0., 0.], [1., 0., 0.], [0., 1., 0.]),
        ([10., 20., 30.], [0., 1., 0.], [0., 0., 1.]),
        ([-5., 2.5, 0.], [1., 0., 0.], [0., 0., 1.]),
    ] {
        let plane = CPlane::new(origin, x, y).unwrap();
        for xyz in [[1., 2., 3.], [-4., 0.5, 7.], [0., 0., 0.]] {
            let local = LocalPoint::try_new(xyz).unwrap();
            let world = plane.to_world(local).unwrap();
            let back = plane.to_local(world).unwrap();
            for axis in 0..3 {
                assert!(
                    (back.xyz()[axis] - xyz[axis]).abs() < 1e-12,
                    "plane {origin:?} point {xyz:?} axis {axis}: {:?}",
                    back.xyz()
                );
            }
        }
    }
}

#[test]
fn the_world_xy_plane_keeps_millimetre_coordinates_unchanged() {
    let plane = CPlane::world_xy();
    let local = parse_local("1.5,-2,0.25", Unit::Millimeter).unwrap();
    let world = resolve(CoordinateInput::Local(local), Some(&plane)).unwrap();
    assert_eq!(world.xyz(), [1.5, -2., 0.25]);
}

#[test]
fn a_local_point_without_a_frame_is_refused_by_the_typed_door() {
    let local = parse_local("1,2", Unit::Millimeter).unwrap();
    assert_eq!(
        resolve(CoordinateInput::Local(local), None),
        Err(Error::Missing("coordinate frame"))
    );
    let world = WorldPoint::try_new([1., 2., 0.]).unwrap();
    assert_eq!(resolve(CoordinateInput::World(world), None).unwrap(), world);
}
