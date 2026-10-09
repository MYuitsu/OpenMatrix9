//! Public-session routing: these expected points/plans are analytic fixtures,
//! independent of PointInput and the numeric classifier implementation.
use openmatrix9_rust::curve::{CurveSession, Effect as CurveEffect};
use openmatrix9_rust::solid::{Effect as SolidEffect, Kind, Session as SolidSession};

fn near(actual: [f64; 3], expected: [f64; 3]) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, e)| (a - e).abs() < 1e-8),
        "{actual:?} != {expected:?}"
    );
}

fn curve(name: &str) -> CurveSession {
    let mut session = CurveSession::default();
    session.start(name).unwrap();
    session
}

fn unsupported_angle(error: String) {
    let lower = error.to_ascii_lowercase();
    assert!(
        lower.contains("angle") && lower.contains("not supported"),
        "{error}"
    );
}

#[test]
fn line_polar_and_world_relative_polar_keep_frame_and_input_scale_distinct() {
    let mut s = curve("Line");
    s.set_input_scale(10.).unwrap();
    s.set_frame([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    assert_eq!(s.input("2<90").unwrap(), CurveEffect::Waiting);
    assert_eq!(s.input("rw5<0").unwrap(), CurveEffect::Commit);
    near(s.output()[0], [10., 20., 50.]);
    near(s.output()[1], [60., 20., 50.]);
}

#[test]
fn polyline_and_interp_rejections_and_undo_preserve_the_accepted_relative_base() {
    for name in ["Polyline", "InterpCrv"] {
        let mut s = curve(name);
        if name == "InterpCrv" {
            s.input("Degree=3").unwrap();
        }
        s.input("5<0").unwrap();
        if name == "Polyline" {
            s.input("Mode=Line").unwrap();
        }
        for malformed in ["rw5<", "5<<90", "5<N91E", "5<90,", "rw1,2,3"] {
            assert!(s.input(malformed).is_err(), "{name}: {malformed}");
            assert_eq!(s.points().len(), 1);
            near(s.points()[0], [5., 0., 0.]);
            assert!(s.output().is_empty() && s.active());
        }
        unsupported_angle(s.input("<45").unwrap_err());
        s.input("rw5<90").unwrap();
        near(s.points()[1], [5., 5., 0.]);
        s.input("Undo").unwrap();
        assert_eq!(s.points().len(), 1);
        s.input("r5<180").unwrap();
        near(s.points()[1], [0., 0., 0.]);
        s.input("@w5<N0E").unwrap();
        near(s.points()[2], [0., 5., 0.]);
        s.input("5<0<90").unwrap();
        assert_eq!(s.input("").unwrap(), CurveEffect::Commit);
        assert_eq!(s.output().len(), 4);
        near(s.output()[3], [0., 0., 5.]);
    }
}

#[test]
fn circle_polar_points_are_not_misrouted_to_scalar_size() {
    let mut s = curve("Circle");
    s.input("History=Yes").unwrap();
    s.input("5<90").unwrap();
    for malformed in ["5<", "5<90<", "rw5<NaN"] {
        assert!(s.input(malformed).is_err());
        assert_eq!(s.points().len(), 1);
    }
    unsupported_angle(s.input("<45").unwrap_err());
    assert_eq!(s.input("rw5<0").unwrap(), CurveEffect::Commit);
    let plan = s.circle().unwrap();
    near(plan.center, [0., 5., 0.]);
    assert!((plan.radius - 5.).abs() < 1e-8);
    assert!(s.circle_history());
}

#[test]
fn circle_radius_option_precedes_numeric_routing_and_keeps_point_phase() {
    let mut s = curve("Circle");
    s.input("3Point").unwrap();
    s.input("5<180").unwrap();
    s.input("5<0").unwrap();
    s.input("Radius").unwrap();
    unsupported_angle(s.input("<45").unwrap_err());
    assert_eq!(s.points().len(), 2);
    assert_eq!(s.input("5").unwrap(), CurveEffect::Waiting);
    assert_eq!(s.input("5<90").unwrap(), CurveEffect::Commit);
    near(s.circle().unwrap().center, [0., 0., 0.]);
    assert!((s.circle().unwrap().radius - 5.).abs() < 1e-8);
}

#[test]
fn rectangle_polar_corner_and_explicit_scalar_options_remain_distinct() {
    let mut s = curve("Rectangle");
    s.input("5<90").unwrap();
    s.input("Length=10").unwrap();
    s.input("Width").unwrap();
    unsupported_angle(s.input("<45").unwrap_err());
    assert_eq!(s.points().len(), 1);
    assert_eq!(s.input("3").unwrap(), CurveEffect::Commit);
    for (actual, expected) in s.output().iter().zip([
        [0., 5., 0.],
        [10., 5., 0.],
        [10., 8., 0.],
        [0., 8., 0.],
        [0., 5., 0.],
    ]) {
        near(*actual, expected);
    }
    assert_eq!(s.output().len(), 5);
}

#[test]
fn rectangle_polar_opposite_corner_uses_latched_construction_frame() {
    let mut s = curve("Rectangle");
    s.set_frame([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    s.input("0<0").unwrap();
    s.set_frame([0.; 3], [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
        .unwrap();
    assert_eq!(s.input("5<45").unwrap(), CurveEffect::Commit);
    near(s.output()[0], [10., 20., 30.]);
    near(
        s.output()[2],
        [10., 23.535533905932738, 33.535_533_905_932_74],
    );
}

#[test]
fn ellipse_polar_axis_and_scalar_axis_keep_recoverable_phase() {
    let mut s = curve("Ellipse");
    s.input("5<90").unwrap();
    s.input("rw5<0").unwrap();
    for malformed in ["3<", "3<N91E"] {
        assert!(s.input(malformed).is_err());
        assert_eq!(s.points().len(), 2);
    }
    unsupported_angle(s.input("<45").unwrap_err());
    s.input("Undo").unwrap();
    assert_eq!(s.points().len(), 1);
    s.input("5").unwrap();
    assert_eq!(s.input("3").unwrap(), CurveEffect::Commit);
    let plan = s.ellipse().unwrap();
    near(plan.center, [0., 5., 0.]);
    assert!((plan.major - 5.).abs() < 1e-8 && (plan.minor - 3.).abs() < 1e-8);
}

#[test]
fn polyline_pending_length_rejects_angle_constraint_without_losing_the_option() {
    let mut s = curve("Polyline");
    s.input("0<0").unwrap();
    s.input("Length").unwrap();
    unsupported_angle(s.input("<45").unwrap_err());
    assert_eq!(s.points().len(), 1);
    s.input("2").unwrap();
    s.input("rw5<90").unwrap();
    assert_eq!(s.input("").unwrap(), CurveEffect::Commit);
    near(s.output()[1], [0., 2., 0.]);
}

#[test]
fn box_world_relative_polar_edge_and_undo_preserve_numeric_dimension_phases() {
    let mut s = SolidSession::new(Kind::Box);
    s.input("3Point").unwrap();
    s.input("5<0").unwrap();
    s.input("rw5<90").unwrap();
    assert_eq!(s.phase(), 3);
    s.input("Undo").unwrap();
    assert_eq!(s.phase(), 2);
    for malformed in ["rw5<", "5<N91E", "rw1,2,3"] {
        assert!(s.input(malformed).is_err());
        assert_eq!(s.phase(), 2);
        assert!(s.output().is_none());
    }
    unsupported_angle(s.input("<45").unwrap_err());
    assert_eq!(s.phase(), 2);
    s.input("rw5<90").unwrap();
    s.input("Width=3").unwrap();
    assert_eq!(s.input("Height=2").unwrap(), SolidEffect::Commit);
    let output = s.output().unwrap();
    near(output[..3].try_into().unwrap(), [5., 0., 0.]);
    near(output[3..6].try_into().unwrap(), [0., 1., 0.]);
    near(output[6..9].try_into().unwrap(), [-1., 0., 0.]);
    near(output[12..].try_into().unwrap(), [5., 3., 2.]);
}

#[test]
fn sphere_spherical_center_and_polar_radius_keep_last_accepted_world_point() {
    let mut s = SolidSession::new(Kind::Sphere);
    s.input("5<0<90").unwrap();
    for malformed in ["rw5<", "5<N91E", "5<90<", "1,2<bad"] {
        assert!(s.input(malformed).is_err());
        assert_eq!(s.phase(), 5);
        assert!(s.output().is_none());
    }
    unsupported_angle(s.input("<45").unwrap_err());
    assert_eq!(s.input("rw5<90").unwrap(), SolidEffect::Commit);
    let output = s.output().unwrap();
    near(output[..3].try_into().unwrap(), [0., 0., 5.]);
    assert!((output[12] - 5.).abs() < 1e-8);
}

#[test]
fn solid_extended_points_keep_the_existing_length_callback_policy() {
    for kind in [Kind::Box, Kind::Sphere] {
        let mut s = SolidSession::new(kind);
        for unsupported in ["1ft<0", "1/2<0", "1m<0"] {
            assert!(s.input(unsupported).is_err(), "{unsupported}");
            assert_eq!(s.phase(), 1);
        }
        s.input("1cm<0").unwrap();
        if kind == Kind::Sphere {
            assert_eq!(s.input("Diameter=2cm").unwrap(), SolidEffect::Commit);
            assert!((s.output().unwrap()[12] - 10.).abs() < 1e-8);
        } else {
            s.input("Length=2cm").unwrap();
            s.input("Width=3mm").unwrap();
            assert_eq!(s.input("Height=4mm").unwrap(), SolidEffect::Commit);
            near(s.output().unwrap()[12..].try_into().unwrap(), [20., 3., 4.]);
        }
        near(s.output().unwrap()[..3].try_into().unwrap(), [10., 0., 0.]);
    }
}

#[test]
fn sphere_modes_require_an_actual_point_before_relative_input() {
    for mode in ["FitPoints", "Tangent"] {
        let mut s = SolidSession::new(Kind::Sphere);
        s.input(mode).unwrap();
        if mode == "Tangent" {
            s.input("Point").unwrap();
        }
        let phase = s.phase();
        for token in ["wr1,0,0", "rw1<0", "r1<90"] {
            assert!(
                s.input(token).is_err(),
                "{mode} accepted {token} without base"
            );
            assert_eq!(s.phase(), phase);
        }
        s.input("w1,0,0").unwrap();
        s.input("Undo").unwrap();
        assert!(s.input("rw1<0").is_err(), "Undo retained a removed base");
    }
}

#[test]
fn sphere_four_point_relative_base_is_the_third_accepted_point() {
    let mut s = SolidSession::new(Kind::Sphere);
    s.input("4Point").unwrap();
    for token in ["w1,0,0", "w0,1,0", "w-1,0,0"] {
        s.input(token).unwrap();
    }
    assert_eq!(s.phase(), 10);
    // Third point + (1,0,1) = north pole (0,0,1) of the unit sphere.
    assert_eq!(s.input("wr1,0,1").unwrap(), SolidEffect::Commit);
    let output = s.output().unwrap();
    near(output[..3].try_into().unwrap(), [0.; 3]);
    assert!((output[12] - 1.).abs() < 1e-8);
}

#[test]
fn sphere_fit_batch_and_reference_points_restore_the_relative_base_with_undo() {
    let mut fit = SolidSession::new(Kind::Sphere);
    fit.input("FitPoints").unwrap();
    fit.append_fit_points(&[[1., 0., 0.], [0., 1., 0.]])
        .unwrap();
    fit.input("wr0,0,1").unwrap();
    fit.input("Undo").unwrap();
    fit.input("Undo").unwrap();
    assert!(fit.input("rw1<0").is_err());

    let mut tangent = SolidSession::new(Kind::Sphere);
    tangent.input("Tangent").unwrap();
    tangent
        .accept_reference([2., 3., 0.], [1., 0., 0.])
        .unwrap();
    tangent.input("Point").unwrap();
    tangent.input("rw1<90").unwrap();
    tangent.input("Undo").unwrap();
    tangent.input("Undo").unwrap();
    tangent.input("Undo").unwrap();
    tangent.input("Point").unwrap();
    assert!(tangent.input("rw1<0").is_err());
}
