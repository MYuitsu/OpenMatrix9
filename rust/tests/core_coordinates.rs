//! Hand-computed RCORE-03 Cartesian fixtures, exercised through real Curve commands.
use openmatrix9_rust::coordinates::{Frame, PointInput, PointKind};
use openmatrix9_rust::curve::{CurveSession, Effect};

const ORIGIN: [f64; 3] = [10., 20., 30.];
const AXES: [[f64; 3]; 3] = [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]];
const FAMILIES: [&str; 5] = ["Line", "Polyline", "InterpCrv", "Rectangle", "Circle"];

fn session(name: &str) -> CurveSession {
    let mut session = CurveSession::default();
    session.start(name).unwrap();
    session.set_frame(ORIGIN, AXES).unwrap();
    session
}

#[test]
fn t01_three_components_are_cplane_coordinates_in_every_existing_point_family() {
    for family in FAMILIES {
        let mut session = session(family);
        assert_eq!(session.input("1,2,3").unwrap(), Effect::Waiting);
        assert_eq!(session.points(), &[[13., 21., 32.]], "{family}");
    }
}

#[test]
fn t02_two_components_and_explicit_world_have_distinct_frames() {
    for family in FAMILIES {
        for (input, expected) in [
            ("1,2", [10., 21., 32.]),
            ("w1,2,3", [1., 2., 3.]),
            ("w0,0,0", [0., 0., 0.]),
        ] {
            let mut session = session(family);
            assert_eq!(session.input(input).unwrap(), Effect::Waiting);
            assert_eq!(session.points(), &[expected], "{family}: {input}");
        }
    }
}

#[test]
fn t03_relative_prefixes_resolve_against_previous_world_point() {
    for family in ["Line", "Polyline", "InterpCrv", "Circle"] {
        for (input, expected) in [
            ("r1,2,3", [16., 22., 34.]),
            ("R1,2,3", [16., 22., 34.]),
            ("@1,2,3", [16., 22., 34.]),
            ("wr1,2,3", [14., 23., 35.]),
        ] {
            let mut session = session(family);
            session.input("1,2,3").unwrap();
            session.input(input).unwrap();
            assert_eq!(
                session.points().last(),
                Some(&expected),
                "{family}: {input}"
            );
        }
    }
}

#[test]
fn t03_relative_without_previous_is_atomic_in_every_point_family() {
    for family in FAMILIES {
        for input in ["r1,2,3", "R1,2,3", "@1,2,3", "wr1,2,3"] {
            let mut session = session(family);
            let prompt = session.prompt();
            assert!(session.input(input).is_err(), "{family}: {input}");
            assert!(session.points().is_empty());
            assert!(session.output().is_empty());
            assert!(session.active());
            assert_eq!(session.prompt(), prompt);
        }
    }
}

#[test]
fn t06_cartesian_input_rejects_internal_whitespace_and_malformed_values_atomically() {
    for input in [
        "4, 5,6",
        "r 1,2,3",
        "4 mm,5,6",
        "1,,3",
        "1,2,3,4",
        "NaN,1,2",
        "w1,2,inf",
        "1,2,3junk",
        "1m2,2,3",
        "rw1,2,3",
        "@w1,2,3",
    ] {
        let mut session = session("Polyline");
        session.input("1,2,3").unwrap();
        let prompt = session.prompt();
        assert!(session.input(input).is_err(), "accepted {input}");
        assert_eq!(session.points(), &[[13., 21., 32.]], "{input}");
        assert_eq!(session.prompt(), prompt, "{input}");
        assert!(session.active());
    }
}

#[test]
fn t06_invalid_frames_are_rejected_without_replacing_the_valid_frame() {
    let mut session = session("Polyline");
    let left_handed = [[0., 1., 0.], [0., 0., 1.], [-1., 0., 0.]];
    assert!(session.set_frame([0.; 3], left_handed).is_err());
    assert!(session.set_frame([0.; 3], [[0.; 3]; 3]).is_err());
    assert!(session.set_frame([f64::NAN, 0., 0.], AXES).is_err());
    session.input("1,2,3").unwrap();
    assert_eq!(session.points(), &[[13., 21., 32.]]);
}

#[test]
fn t07_world_mouse_points_are_never_transformed_by_the_input_cplane() {
    for family in FAMILIES {
        let mut session = session(family);
        assert_eq!(
            session.preview_point([13., 21., 32.], false).unwrap(),
            [13., 21., 32.]
        );
        assert!(session.points().is_empty());
        session.mouse_point([13., 21., 32.], false).unwrap();
        assert_eq!(session.points(), &[[13., 21., 32.]], "{family}");
    }
}

#[test]
fn parsed_points_keep_the_original_syntax_frame_and_relative_base() {
    let frame = Frame::new(ORIGIN, AXES).unwrap();
    for (source, kind, expected) in [
        ("1,2,3", PointKind::AbsoluteCPlane, [13., 21., 32.]),
        ("w1,2,3", PointKind::AbsoluteWorld, [1., 2., 3.]),
        ("r1,2,3", PointKind::RelativeCPlane, [16., 22., 34.]),
        ("wr1,2,3", PointKind::RelativeWorld, [14., 23., 35.]),
    ] {
        let parsed = PointInput::parse(source, |value| {
            value.parse().map_err(|_| "bad number".into())
        })
        .unwrap();
        let resolved = parsed.resolve(frame, Some([13., 21., 32.])).unwrap();
        assert_eq!(resolved.world, expected);
        assert_eq!(resolved.input.original, source);
        assert_eq!(resolved.input.kind, kind);
        assert_eq!(resolved.input.components, [1., 2., 3.]);
        assert_eq!(resolved.frame, frame);
        assert_eq!(resolved.previous, Some([13., 21., 32.]));
    }
}

#[test]
fn coordinate_conversion_applies_length_units_once_before_the_frame_transform() {
    let frame = Frame::new(ORIGIN, AXES).unwrap();
    let point = PointInput::parse("1,2cm,3mm", |text| {
        openmatrix9_rust::units::parse_length(text, 25.4).map_err(|error| error.to_string())
    })
    .unwrap()
    .resolve(frame, None)
    .unwrap();
    assert_eq!(point.world, [13., 45.4, 50.]);
    assert_eq!(point.input.components, [25.4, 20., 3.]);
}

#[test]
fn coordinates_remain_in_domain_after_transforming_or_adding_relative_base() {
    let mut session = session("Polyline");
    session.input("1,2,3").unwrap();
    for input in ["999999999,2,3", "r1,2,999999999", "wr999999999,2,3"] {
        assert!(session.input(input).is_err(), "{input}");
        assert_eq!(session.points(), &[[13., 21., 32.]]);
        assert!(session.active());
    }
}

#[test]
fn rectangle_and_circle_keep_the_existing_first_point_latch_until_point_undo() {
    for family in ["Rectangle", "Circle"] {
        let mut session = session(family);
        session.input("1,2,3").unwrap();
        session.set_frame([0.; 3], Frame::default().axes()).unwrap();
        if family == "Rectangle" {
            // The second corner is projected onto the plane through the first;
            // its transformed CPlane x/y remain in the latched Y/Z directions.
            session.input("4,6,3").unwrap();
            assert_eq!(
                session.output(),
                &[
                    [13., 21., 32.],
                    [13., 24., 32.],
                    [13., 24., 36.],
                    [13., 21., 36.],
                    [13., 21., 32.],
                ]
            );
        } else {
            session.input("4,6,3").unwrap();
            let circle = session.circle().unwrap();
            assert_eq!(circle.center, [13., 21., 32.]);
            assert_eq!(circle.normal, [1., 0., 0.]);
            assert_eq!(circle.radius, 5.);
        }

        let mut session = self::session(family);
        session.input("1,2,3").unwrap();
        session.input("Undo").unwrap();
        session.set_frame([0.; 3], Frame::default().axes()).unwrap();
        session.input("1,2,3").unwrap();
        assert_eq!(session.points(), &[[1., 2., 3.]]);
    }
}

#[test]
fn cancel_and_restart_do_not_reuse_previous_coordinates_or_construction_frame() {
    for family in FAMILIES {
        let mut session = session(family);
        session.input("1,2,3").unwrap();
        session.cancel();
        session.start(family).unwrap();
        assert!(session.input("@1,2,3").is_err());
        session.input("1,2,3").unwrap();
        assert_eq!(session.points(), &[[1., 2., 3.]], "{family}");
    }
}

#[test]
fn t01_t02_box_and_sphere_use_the_same_explicit_frame_grammar() {
    use openmatrix9_rust::solid::{Effect, Kind, Session};
    for kind in [Kind::Box, Kind::Sphere] {
        for (input, expected) in [
            ("1,2,3", [13., 21., 32.]),
            ("1,2", [10., 21., 32.]),
            ("w1,2,3", [1., 2., 3.]),
            ("w0,0,0", [0., 0., 0.]),
        ] {
            let mut session = Session::new(kind);
            session.set_frame(ORIGIN, AXES).unwrap();
            session.input(input).unwrap();
            if kind == Kind::Box {
                session.input("4").unwrap();
                session.input("5").unwrap();
            }
            assert_eq!(session.input("6").unwrap(), Effect::Commit);
            assert_eq!(
                &session.output().unwrap()[..3],
                &expected,
                "{kind:?}: {input}"
            );
        }
    }
}

#[test]
fn t03_box_and_sphere_relative_input_uses_the_previous_world_point() {
    use openmatrix9_rust::solid::{Effect, Kind, Session};
    for (input, expected_box, expected_sphere) in [
        ("r1,2,3", [1., 2., 4.], [14.5, 21.5, 33.]),
        ("R1,2,3", [1., 2., 4.], [14.5, 21.5, 33.]),
        ("@1,2,3", [1., 2., 4.], [14.5, 21.5, 33.]),
        ("wr1,2,3", [2., 3., 4.], [13.5, 22., 33.5]),
    ] {
        for kind in [Kind::Box, Kind::Sphere] {
            let mut session = Session::new(kind);
            session.set_frame(ORIGIN, AXES).unwrap();
            if kind == Kind::Sphere {
                session.input("2Point").unwrap();
            }
            assert!(session.input(input).is_err());
            assert_eq!(session.phase(), 1);
            session.input("1,2,3").unwrap();
            session.input(input).unwrap();
            if kind == Kind::Box {
                assert_eq!(session.input("4").unwrap(), Effect::Commit);
                assert_eq!(&session.output().unwrap()[12..], &expected_box);
            } else {
                assert_eq!(&session.output().unwrap()[..3], &expected_sphere);
                assert!((session.output().unwrap()[12] - 14_f64.sqrt() / 2.).abs() < 1e-12);
            }
        }
    }
}
