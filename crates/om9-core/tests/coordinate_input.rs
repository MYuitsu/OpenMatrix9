// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-02-004`: world/local/relative coordinate entry.
use om9_core::{
    Error,
    coords::{
        CoordinateInput, LocalFrame, LocalPoint, WorldPoint, parse_input, parse_local, resolve,
        resolve_with_base,
    },
    geometry::Unit,
};

#[test]
fn w_prefix_is_a_world_point_and_bare_values_are_local() {
    assert_eq!(
        parse_input("W10,20,30", Unit::Millimeter).unwrap(),
        CoordinateInput::World(WorldPoint::try_new([10., 20., 30.]).unwrap())
    );
    assert_eq!(
        parse_input("w10,20", Unit::Millimeter).unwrap(),
        CoordinateInput::World(WorldPoint::try_new([10., 20., 0.]).unwrap())
    );
    assert_eq!(
        parse_input("10,20,30", Unit::Millimeter).unwrap(),
        CoordinateInput::Local(LocalPoint::try_new([10., 20., 30.]).unwrap())
    );
    assert_eq!(
        parse_input("1,2", Unit::Inch).unwrap(),
        CoordinateInput::Local(LocalPoint::try_new([25.4, 50.8, 0.]).unwrap())
    );
}

#[test]
fn relative_input_is_a_displacement_and_needs_a_base() {
    assert_eq!(
        parse_input("@5,0,0", Unit::Millimeter).unwrap(),
        CoordinateInput::Relative(LocalPoint::try_new([5., 0., 0.]).unwrap())
    );
    assert_eq!(
        resolve(parse_input("@5,0,0", Unit::Millimeter).unwrap(), None),
        Err(Error::Missing("relative base point"))
    );
    struct Identity;
    impl LocalFrame for Identity {
        fn to_world(&self, point: LocalPoint) -> om9_core::Result<WorldPoint> {
            WorldPoint::try_new(point.xyz())
        }
        fn to_local(&self, point: WorldPoint) -> om9_core::Result<LocalPoint> {
            LocalPoint::try_new(point.xyz())
        }
    }
    let base = WorldPoint::try_new([10., 20., 30.]).unwrap();
    let resolved = resolve_with_base(
        parse_input("@5,-2,0", Unit::Millimeter).unwrap(),
        Some(&Identity),
        Some(base),
    )
    .unwrap();
    assert_eq!(resolved.xyz(), [15., 18., 30.]);
}

#[test]
fn a_shifted_frame_does_not_leak_its_origin_into_a_relative_entry() {
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
    let base = WorldPoint::try_new([100., 200., 300.]).unwrap();
    let resolved = resolve_with_base(
        parse_input("@1,2,3", Unit::Millimeter).unwrap(),
        Some(&Shifted),
        Some(base),
    )
    .unwrap();
    assert_eq!(resolved.xyz(), [101., 202., 303.]);
}

#[test]
fn malformed_nan_overflow_and_ambiguous_separators_are_typed_errors() {
    for text in [
        "W",
        "@",
        "W1",
        "@1,2,3,4",
        "W1 2 3",
        "WNaN,0,0",
        "W1e309,0,0",
        "W1,x,0",
    ] {
        assert!(
            parse_input(text, Unit::Millimeter).is_err(),
            "{text:?} must be refused"
        );
    }
    assert_eq!(
        parse_local("1,2", Unit::Millimeter).unwrap().xyz(),
        [1., 2., 0.]
    );
}
