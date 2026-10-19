// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-08-007 typed world/local coordinate input.
use om9_core::{
    Error,
    coords::{CoordinateInput, LocalFrame, LocalPoint, WorldPoint, parse_local, resolve},
    geometry::Unit,
};

/// A shifted/rotated frame so the conversion cannot be the identity.
struct Shifted;
impl LocalFrame for Shifted {
    fn to_world(&self, point: LocalPoint) -> om9_core::Result<WorldPoint> {
        let [x, y, z] = point.xyz();
        WorldPoint::try_new([10. + x, 20. + y, 30. + z])
    }
    fn to_local(&self, point: WorldPoint) -> om9_core::Result<LocalPoint> {
        let [x, y, z] = point.xyz();
        LocalPoint::try_new([x - 10., y - 20., z - 30.])
    }
}

#[test]
fn l3_007_a_local_point_needs_a_frame() {
    let local = LocalPoint::try_new([1., 2., 3.]).unwrap();
    assert_eq!(
        resolve(CoordinateInput::Local(local), None),
        Err(Error::Missing("coordinate frame"))
    );
    let world = resolve(CoordinateInput::Local(local), Some(&Shifted)).unwrap();
    assert_eq!(world.xyz(), [11., 22., 33.]);
}

#[test]
fn l3_007_a_world_point_resolves_without_a_frame_and_round_trips() {
    let world = WorldPoint::try_new([4., 5., 6.]).unwrap();
    assert_eq!(resolve(CoordinateInput::World(world), None).unwrap(), world);
    let local = Shifted.to_local(world).unwrap();
    assert_eq!(local.xyz(), [-6., -15., -24.]);
    assert_eq!(Shifted.to_world(local).unwrap(), world);
    assert_eq!(
        WorldPoint::try_new([f64::NAN, 0., 0.]),
        Err(Error::Invalid("nonfinite coordinate"))
    );
}

#[test]
fn l3_007_the_parser_keeps_the_session_grammar_and_error_labels() {
    assert_eq!(
        parse_local("1,2", Unit::Millimeter).unwrap().xyz(),
        [1., 2., 0.]
    );
    assert_eq!(
        parse_local(" 1 , 2 , 3 ", Unit::Millimeter).unwrap().xyz(),
        [1., 2., 3.]
    );
    assert_eq!(
        parse_local("1,2", Unit::Inch).unwrap().xyz(),
        [25.4, 50.8, 0.]
    );
    assert_eq!(
        parse_local("1", Unit::Millimeter).err(),
        Some(Error::Invalid("x,y[,z] coordinate"))
    );
    assert_eq!(
        parse_local("1,2,3,4", Unit::Millimeter).err(),
        Some(Error::Invalid("x,y[,z] coordinate"))
    );
    assert_eq!(
        parse_local("1,x", Unit::Millimeter).err(),
        Some(Error::Invalid("coordinate number"))
    );
}

#[test]
fn l3_007_a_parsed_local_point_becomes_world_only_through_the_frame() {
    let local = parse_local("1,2,3", Unit::Millimeter).unwrap();
    let world = resolve(CoordinateInput::Local(local), Some(&Shifted)).unwrap();
    assert_eq!(world.xyz(), [11., 22., 33.]);
    // The frame type is what owns the mapping; the point alone cannot claim world identity.
    assert_ne!(world.xyz(), local.xyz());
}
