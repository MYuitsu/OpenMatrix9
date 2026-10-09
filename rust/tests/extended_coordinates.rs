use openmatrix9_rust::coordinates::{
    Frame, NumericSyntax, PointInput, PointKind, PointNotation, SurveyorQuadrant, classify_numeric,
};

fn frame() -> Frame {
    Frame::new([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]]).unwrap()
}
fn point(text: &str, previous: Option<[f64; 3]>) -> Result<[f64; 3], String> {
    PointInput::parse(text, |token| {
        openmatrix9_rust::units::parse_length(token, 1.).map_err(|e| e.to_string())
    })?
    .resolve(frame(), previous)
    .map(|p| p.world)
}
fn close(actual: [f64; 3], expected: [f64; 3]) {
    for (a, e) in actual.into_iter().zip(expected) {
        assert!((a - e).abs() < 1e-11, "{actual:?} != {expected:?}");
    }
}

#[test]
fn syntax_routes_noncartesian_points_before_scalar_and_keeps_angle_constraints_distinct() {
    for text in ["1,2", "rw4<45", "5<30,2", "5<30<45", "3,4<45", "5<N30E"] {
        assert_eq!(classify_numeric(text), NumericSyntax::Point, "{text}");
    }
    assert_eq!(classify_numeric("  <90 "), NumericSyntax::AngleConstraint);
    assert_eq!(classify_numeric("<"), NumericSyntax::AngleConstraint);
    assert_eq!(classify_numeric("2cm"), NumericSyntax::Scalar);
}

#[test]
fn spherical_xy_rejects_exact_odd_quarter_turns_after_many_revolutions() {
    for angle in ["99990", "-99990", "999999990", "99990d0'0\"", "99990-0/1"] {
        let text = format!("w0.00001,0.00001<{angle}");
        assert!(point(&text, None).is_err(), "accepted singular {text}");
    }
    assert!(point("w0.00001,0.00001<99989.99", None).is_ok());
}

#[test]
fn polar_and_spherical_points_follow_the_rotated_frame_once() {
    for (text, expected) in [
        ("5<0", [10., 25., 30.]),
        ("5<90", [10., 20., 35.]),
        ("5<90,2", [12., 20., 35.]),
        ("5<0<90", [15., 20., 30.]),
        ("10<30<30", [15., 27.5, 34.33012701892219]),
        ("3,4<45", [15., 23., 34.]),
        ("3,4<-45", [5., 23., 34.]),
        ("-3,4<0", [10., 17., 34.]),
    ] {
        close(point(text, None).unwrap(), expected);
    }
}

#[test]
fn surveyor_quadrants_and_dms_have_independent_world_oracles() {
    for (text, expected) in [
        ("10<N30E", [10., 25., 38.660_254_037_844_39]),
        ("10<N30W", [10., 15., 38.660_254_037_844_39]),
        ("10<S30E", [10., 25., 21.339745962155614]),
        ("10<S30W", [10., 15., 21.339745962155614]),
        ("10<N30d0'0\"E", [10., 25., 38.660_254_037_844_39]),
    ] {
        close(point(text, None).unwrap(), expected);
    }
}

#[test]
fn documented_prefixes_use_previous_without_adding_cplane_origin() {
    let previous = Some([13., 21., 32.]);
    for (text, expected) in [
        ("w5<90,2", [0., 5., 2.]),
        ("w3,4<45", [3., 4., 5.]),
        ("r5<90,2", [15., 21., 37.]),
        ("R5<90,2", [15., 21., 37.]),
        ("@5<90,2", [15., 21., 37.]),
        ("wr5<90,2", [13., 26., 34.]),
        ("rw5<90", [13., 26., 32.]),
        ("wr5<0<90", [13., 21., 37.]),
        ("r3,4<45", [18., 24., 36.]),
        ("wr10<S30W", [8., 12.339745962155614, 32.]),
        ("@w10<N30W", [8., 29.660254037844386, 32.]),
    ] {
        close(point(text, previous).unwrap(), expected);
    }
    for text in [
        "r5<90",
        "R3,4<45",
        "@5<0<90",
        "wr10<N30E",
        "rw5<90",
        "@w10<N30W",
    ] {
        assert!(
            point(text, None).is_err(),
            "missing previous accepted: {text}"
        );
    }
}

#[test]
fn invalid_extended_syntax_and_unsourced_prefixes_reject() {
    for text in [
        "5<",
        "<90",
        "5<<90",
        "5<90<",
        "5<90<2<3",
        "5<90,2,3",
        "3,4,5<45",
        "3,<45",
        "5<NaN",
        "5<90mm",
        "5<30 junk",
        "5< 30",
        "5<N91E",
        "5<N-1E",
        "5<E30N",
        "5<N30",
        "5<N30EE",
        "5<N30E,2",
        "3,4<90",
        "3,4<-90",
        "-1<30",
        "-1<30<20",
        "rw1,2,3",
        "@w1,2,3",
        "rw5<30<20",
        "@w5<30",
        "Rw5<30",
        "w@5<30",
        "5<30garbage",
    ] {
        assert!(
            point(text, Some([13., 21., 32.])).is_err(),
            "accepted {text}"
        );
    }
}

#[test]
fn fractions_feet_and_documented_surveys_keep_lengths_separate_from_angles() {
    close(point("w1'6\"<90,1/2in", None).unwrap(), [0., 457.2, 12.7]);
    close(
        point("10<90/2", None).unwrap(),
        [10., 27.071067811865476, 37.071067811865476],
    );
    let text = "@w11<N30d22'54.43\"W";
    let input = PointInput::parse(text, |token| {
        openmatrix9_rust::units::parse_length(token, 1.).map_err(|e| e.to_string())
    })
    .unwrap();
    assert_eq!(input.original, text);
    assert_eq!(input.kind, PointKind::RelativeWorld);
    match input.notation {
        PointNotation::Surveyor {
            radius_mm,
            bearing_rad,
            quadrant,
        } => {
            assert_eq!(radius_mm, 11.);
            assert!((bearing_rad - 0.5302622002755727).abs() < 1e-15);
            assert_eq!(quadrant, SurveyorQuadrant::NorthWest);
        }
        _ => panic!("surveyor input lost its original notation"),
    }
    close(
        input.resolve(frame(), Some([13., 21., 32.])).unwrap().world,
        [7.436644927158961, 30.48941938863985, 32.],
    );
}

#[test]
fn radius_and_result_domain_policy_is_explicit_and_zero_remains_operation_owned() {
    close(point("0<45", None).unwrap(), [10., 20., 30.]);
    close(point("w1e9<0", None).unwrap(), [1e9, 0., 0.]);
    for text in [
        "1e9<0",
        "w1000000001<45",
        "w1,1<89.999999999",
        "w-1<N30E",
        "w-1<30<45",
        "5<1e309",
        "NaN<45",
        "5<0,NaN",
    ] {
        assert!(point(text, None).is_err(), "accepted {text}");
    }
    assert!(point("r5<90", Some([f64::INFINITY, 0., 0.])).is_err());
}
