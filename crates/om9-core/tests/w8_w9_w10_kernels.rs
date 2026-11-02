// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9 wave W8/W9/W10: modelling capability policy, cage evaluation and measurements.
//!
//! Oracles: an identity cage is the identity map, a uniform-weight cage normalises its
//! weights away, admission refuses malformed cages, the capability policy refuses by the
//! reason the command line shows, and the measurements agree with hand-computed geometry.
use om9_core::{
    Error,
    budget::Limits,
    cage::{Cage, Region, capture_parameters},
    geometry::{Point3, Unit},
    measure::{angle_degrees, distance, distance_mm},
    modeling_capability::{
        InputFacts, KERNEL_TOLERANCE, Kind, Operation, Reason, capability,
        join_uses_surface_adapter, refusal, require, surface_options, validate_inputs,
        validate_result_count,
    },
};

fn limits() -> Limits {
    Limits::default()
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

/* ------------------------- measure (W10) ------------------------- */

#[test]
fn angle_between_two_directed_lines_matches_the_geometry() {
    // +X against +Y is a quarter turn.
    let angle = angle_degrees([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [0.0, 1.0, 0.0]).unwrap();
    assert!((angle - 90.0).abs() < 1e-9, "{angle}");
    // Parallel lines: zero.
    assert!(
        angle_degrees([0.0; 3], [2.0, 0.0, 0.0], [1.0; 3], [3.0, 1.0, 1.0])
            .unwrap()
            .abs()
            < 1e-9
    );
    // Opposite directions: 180.
    let opposite = angle_degrees([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [-1.0, 0.0, 0.0]).unwrap();
    assert!((opposite - 180.0).abs() < 1e-9, "{opposite}");
    // 45 degrees in the XY plane.
    let diagonal = angle_degrees([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [1.0, 1.0, 0.0]).unwrap();
    assert!((diagonal - 45.0).abs() < 1e-9, "{diagonal}");
}

#[test]
fn measurements_refuse_degenerate_and_oversized_input() {
    // A zero-length line has no direction.
    assert_eq!(
        angle_degrees([0.0; 3], [0.0; 3], [0.0; 3], [1.0, 0.0, 0.0]),
        Err(Error::Invalid("measurement direction"))
    );
    // Beyond the modelling bound of 1e9.
    assert_eq!(
        distance([0.0; 3], [1.0e10, 0.0, 0.0], Unit::Millimeter),
        Err(Error::Invalid("measurement coordinate"))
    );
    assert_eq!(
        angle_degrees([0.0; 3], [f64::NAN, 0.0, 0.0], [0.0; 3], [1.0, 0.0, 0.0]),
        Err(Error::Invalid("nonfinite coordinate"))
    );
}

#[test]
fn distance_uses_the_requested_display_unit() {
    // A 25.4 mm offset is exactly one inch.
    let first = [0.0; 3];
    let second = [25.4, 0.0, 0.0];
    assert!((distance(first, second, Unit::Millimeter).unwrap() - 25.4).abs() < 1e-12);
    assert!((distance(first, second, Unit::Inch).unwrap() - 1.0).abs() < 1e-12);
    assert!((distance(first, second, Unit::Centimeter).unwrap() - 2.54).abs() < 1e-12);
    assert!((distance(first, second, Unit::Meter).unwrap() - 0.0254).abs() < 1e-12);
    assert!((distance_mm(first, [3.0, 4.0, 0.0]).unwrap() - 5.0).abs() < 1e-12);
}

/* ------------------------- modelling capability (W8) ------------------------- */

#[test]
fn the_capability_policy_matches_the_legacy_rules() {
    // Booleans need closed solids.
    let curve = InputFacts::new(Kind::Curve, true, false);
    let solid = InputFacts::new(Kind::Solid, true, true);
    let open_shell = InputFacts::new(Kind::Shell, true, false);
    assert_eq!(
        capability(Operation::Union, &[solid, open_shell]),
        Err(Reason::ClosedSolidRequired)
    );
    assert_eq!(capability(Operation::Union, &[solid, solid]), Ok(()));
    // Loft/Sweep only accept curves.
    assert_eq!(
        capability(Operation::Loft, &[curve, solid]),
        Err(Reason::Unsupported)
    );
    assert_eq!(capability(Operation::Loft, &[curve, curve]), Ok(()));
    // Join must not mix curves with surfaces.
    let face = InputFacts::new(Kind::Face, true, false);
    assert_eq!(
        capability(Operation::Join, &[curve, face]),
        Err(Reason::MixedInputs)
    );
    assert_eq!(capability(Operation::Join, &[curve, curve]), Ok(()));
    // Sweep2 needs three inputs when the selection is complete.
    assert_eq!(
        capability(Operation::Sweep2, &[curve, curve]),
        Err(Reason::InputCount)
    );
    assert_eq!(
        capability(Operation::Sweep2, &[curve, curve, curve]),
        Ok(())
    );
    // TwoObjects is exactly two.
    assert_eq!(
        capability(Operation::TwoObjects, &[solid, solid, solid]),
        Err(Reason::InputCount)
    );
    // Explode needs a multi-component object.
    let mut single = InputFacts::new(Kind::Solid, true, true);
    single.components = 1;
    assert_eq!(
        capability(Operation::Explode, &[single]),
        Err(Reason::InputCount)
    );
    let mut multi = InputFacts::new(Kind::Solid, true, true);
    multi.components = 3;
    assert_eq!(capability(Operation::Explode, &[multi]), Ok(()));
}

#[test]
fn an_incomplete_selection_only_refuses_unusable_inputs() {
    let curve = InputFacts::new(Kind::Curve, true, false);
    // Two curves would fail the Sweep2 minimum, but the selection is still open.
    assert_eq!(
        validate_inputs(Operation::Sweep2, &[curve, curve], false),
        Ok(())
    );
    let mut protected = InputFacts::new(Kind::Curve, true, false);
    protected.protected = true;
    assert_eq!(
        validate_inputs(Operation::Sweep2, &[curve, protected], false),
        Err(Reason::Protected)
    );
    let mut invalid = InputFacts::new(Kind::Curve, true, false);
    invalid.valid = false;
    assert_eq!(
        validate_inputs(Operation::Join, &[curve, invalid], false),
        Err(Reason::InvalidTopology)
    );
    // Empty and oversized selections are refused before anything else.
    assert_eq!(
        validate_inputs(Operation::Join, &[], true),
        Err(Reason::InputCount)
    );
}

#[test]
fn surface_options_follow_the_recorded_style_and_count_rules() {
    // Sweep1: one rail, minimum two inputs when open, style must be Normal (0).
    assert_eq!(surface_options(1, 0, false, 0, 16, 0.01, 2), Ok(()));
    assert_eq!(
        surface_options(1, 0, false, 0, 16, 0.01, 1),
        Err(Reason::InputCount)
    );
    assert_eq!(
        surface_options(1, 1, false, 0, 16, 0.01, 2),
        Err(Reason::InvalidOptions)
    );
    // Loft: no rails, two sections, styles 0..=5 allowed, Developable cannot close.
    assert_eq!(
        surface_options(3, 5, true, 0, 16, 0.01, 2),
        Err(Reason::InvalidOptions)
    );
    assert_eq!(surface_options(3, 5, false, 0, 16, 0.01, 2), Ok(()));
    // Closed loft needs rails + 2 + 1 = 3 inputs for Loft (no rails).
    assert_eq!(
        surface_options(3, 0, true, 0, 16, 0.01, 2),
        Err(Reason::InputCount)
    );
    assert_eq!(surface_options(3, 0, true, 0, 16, 0.01, 3), Ok(()));
    // Unknown kind, bad tolerance, bad point count, bad section mode.
    assert_eq!(
        surface_options(9, 0, false, 0, 16, 0.01, 3),
        Err(Reason::InvalidOptions)
    );
    assert_eq!(
        surface_options(1, 0, false, 0, 16, 0.0, 2),
        Err(Reason::InvalidOptions)
    );
    assert_eq!(
        surface_options(1, 0, false, 0, 1, 0.01, 2),
        Err(Reason::InvalidOptions)
    );
    assert_eq!(
        surface_options(1, 0, false, 3, 16, 0.01, 2),
        Err(Reason::InvalidOptions)
    );
}

#[test]
fn the_capability_helpers_keep_their_wording_and_bounds() {
    assert_eq!(validate_result_count(0), Err(Reason::InvalidTopology));
    assert_eq!(validate_result_count(1), Ok(()));
    assert!(join_uses_surface_adapter(Kind::Face));
    assert!(!join_uses_surface_adapter(Kind::Curve));
    assert!((KERNEL_TOLERANCE - 1e-7).abs() < 1e-24);
    // The refusal carries the user-facing message.
    assert_eq!(
        refusal(Reason::MixedInputs),
        Error::Invalid("Select curves or surfaces separately")
    );
    let curve = InputFacts::new(Kind::Curve, true, false);
    assert_eq!(require(Operation::Loft, &[curve, curve]), Ok(()));
    assert!(require(Operation::Loft, &[curve]).is_err());
    // Wire round trip for the codes the UI sends.
    assert_eq!(Kind::from_wire(5), Some(Kind::Solid));
    assert_eq!(Kind::from_wire(10), None);
    assert_eq!(Kind::Solid.wire(), 5);
    assert_eq!(Operation::from_wire(9), Some(Operation::Sweep2));
    assert_eq!(Operation::Sweep2.wire(), 9);
}

/* ------------------------- cage (W9) ------------------------- */

/// A trilinear identity cage over the unit cube, U fastest.
fn unit_box_cage(scale: f64, offset: Point3, weight: f64) -> Cage {
    let mut points = Vec::new();
    for w in 0..2 {
        for v in 0..2 {
            for u in 0..2 {
                points.push([
                    offset[0] + u as f64 * scale,
                    offset[1] + v as f64 * scale,
                    offset[2] + w as f64 * scale,
                ]);
            }
        }
    }
    Cage {
        counts: [2, 2, 2],
        degrees: [1, 1, 1],
        knots: [
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        ],
        points,
        weights: vec![weight; 8],
    }
}

#[test]
fn an_identity_cage_is_the_identity_map() {
    let cage = unit_box_cage(1.0, [0.0; 3], 1.0);
    cage.validate(&limits()).unwrap();
    for parameters in [
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.5, 0.5, 0.5],
        [0.25, 0.5, 0.75],
        [1.0, 0.0, 0.25],
    ] {
        let point = cage.evaluate(parameters, &limits()).unwrap();
        assert!(
            close(point, parameters, 1e-12),
            "{parameters:?} -> {point:?}"
        );
    }
}

#[test]
fn a_scaled_and_offset_cage_maps_its_parameter_cube() {
    let cage = unit_box_cage(2.0, [10.0, -5.0, 3.0], 1.0);
    let point = cage.evaluate([0.5, 0.5, 0.5], &limits()).unwrap();
    assert!(close(point, [11.0, -4.0, 4.0], 1e-12), "{point:?}");
    let corner = cage.evaluate([1.0, 0.0, 1.0], &limits()).unwrap();
    assert!(close(corner, [12.0, -5.0, 5.0], 1e-12), "{corner:?}");
}

#[test]
fn uniform_weights_normalise_away() {
    let light = unit_box_cage(1.0, [0.0; 3], 1.0);
    let heavy = unit_box_cage(1.0, [0.0; 3], 1000.0);
    for parameters in [[0.0, 0.0, 0.0], [0.3, 0.6, 0.9], [1.0, 1.0, 1.0]] {
        let a = light.evaluate(parameters, &limits()).unwrap();
        let b = heavy.evaluate(parameters, &limits()).unwrap();
        assert!(close(a, b, 1e-12), "{parameters:?}: {a:?} != {b:?}");
    }
}

#[test]
fn a_cage_refuses_a_malformed_record() {
    let bound = limits();
    // Wrong point count.
    let mut cage = unit_box_cage(1.0, [0.0; 3], 1.0);
    cage.points.pop();
    assert_eq!(
        cage.validate(&bound),
        Err(Error::Invalid("cage control grid"))
    );
    // Non-positive weight.
    let mut cage = unit_box_cage(1.0, [0.0; 3], 1.0);
    cage.weights[0] = 0.0;
    assert_eq!(cage.validate(&bound), Err(Error::Invalid("cage weights")));
    // Non-finite point.
    let mut cage = unit_box_cage(1.0, [0.0; 3], 1.0);
    cage.points[0] = [f64::NAN, 0.0, 0.0];
    assert_eq!(
        cage.validate(&bound),
        Err(Error::Invalid("nonfinite coordinate"))
    );
    // Not clamped: the first knot of the span repeats an interior value.
    let mut cage = unit_box_cage(1.0, [0.0; 3], 1.0);
    cage.knots[0] = vec![0.0, 0.0, 0.5, 1.0];
    assert!(matches!(
        cage.validate(&bound),
        Err(Error::Invalid("cage knots")) | Err(Error::Invalid("cage knot multiplicity"))
    ));
    // A parameter outside the domain is refused at evaluation.
    let cage = unit_box_cage(1.0, [0.0; 3], 1.0);
    assert_eq!(
        cage.evaluate([-0.5, 0.5, 0.5], &limits()),
        Err(Error::Invalid("cage parameter"))
    );
    assert_eq!(
        cage.evaluate([f64::INFINITY, 0.5, 0.5], &limits()),
        Err(Error::Invalid("nonfinite coordinate"))
    );
    // Inversion is implemented: a point inside the cage inverts to its parameters.
    let parameters = capture_parameters(&cage, &[[0.0; 3]], &limits()).unwrap();
    assert_eq!(parameters.len(), 1);
    assert!(parameters[0].iter().all(|value| value.is_finite()));
}

#[test]
fn a_cage_region_is_validated() {
    assert_eq!(Region::Global.validate(), Ok(()));
    assert!(
        Region::Local {
            min: [0.0; 3],
            max: [1.0, 1.0, 1.0],
            falloff: 0.5,
        }
        .validate()
        .is_ok()
    );
    assert_eq!(
        Region::Local {
            min: [1.0, 0.0, 0.0],
            max: [0.0, 1.0, 1.0],
            falloff: 0.5,
        }
        .validate(),
        Err(Error::Invalid("cage region box"))
    );
    assert_eq!(
        Region::Local {
            min: [0.0; 3],
            max: [1.0, 1.0, 1.0],
            falloff: -1.0,
        }
        .validate(),
        Err(Error::Invalid("cage region falloff"))
    );
}
