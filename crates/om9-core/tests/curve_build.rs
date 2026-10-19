// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-03 wave W2: curve construction kernel ported from the legacy bridge.
//!
//! Oracles are geometric, not textual: an interpolation must pass through its input
//! points, a circle through three points must have the circumscribed centre/radius, a
//! rectangle must close, and a degenerate input must be refused instead of producing a
//! curve the document would reject later.
use om9_core::{
    Error,
    budget::Limits,
    curve_build::{
        KnotMode, arc_through, basis, circle_through, elevate_degree, ellipse, fit_points,
        insert_knot, interpolate, parameters, polyline, rebuild, rectangle,
    },
    evaluation::{Parameter, evaluate_curve},
    geometry::{Curve, Geometry, Transform},
};

fn limits() -> Limits {
    Limits::default()
}

fn evaluate(curve: &Curve, t: f64) -> [f64; 3] {
    evaluate_curve(
        curve,
        Parameter::Native(t),
        0,
        Transform::identity(),
        &limits(),
    )
    .expect("admitted curve")[0]
}

fn close(a: [f64; 3], b: [f64; 3], tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

/* ------------------------- interpolation ------------------------- */

#[test]
fn interpolation_passes_through_every_input_point() {
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 2.0, 0.0],
        [3.0, 3.0, 1.0],
        [5.0, 1.0, 2.0],
        [6.0, -1.0, 0.5],
    ];
    for mode in [KnotMode::ChordLength, KnotMode::Uniform] {
        let curve = interpolate(&points, 3, mode, &limits()).unwrap();
        let parameter_values = parameters(&points, mode, false).unwrap();
        let nurbs = Curve::Nurbs(curve.clone());
        for (point, parameter) in points.iter().zip(&parameter_values) {
            let evaluated = evaluate(&nurbs, *parameter);
            assert!(
                close(evaluated, *point, 1e-8),
                "{mode:?} at {parameter}: {evaluated:?} != {point:?}"
            );
        }
        // The document admits the constructed record.
        Geometry::Curve(nurbs).validate(&limits()).unwrap();
    }
}

#[test]
fn interpolation_degree_and_point_count_are_refused_when_invalid() {
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 1.0, 0.0],
    ];
    // Even and out-of-range degrees follow the legacy rule (odd 1..=11 only).
    for degree in [0usize, 2, 4, 12] {
        assert_eq!(
            interpolate(&points, degree, KnotMode::ChordLength, &limits()),
            Err(Error::Invalid("interpolation degree")),
            "{degree}"
        );
    }
    // More degree than points is refused.
    assert_eq!(
        interpolate(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            3,
            KnotMode::Uniform,
            &limits()
        ),
        Err(Error::Invalid("interpolation point count"))
    );
    // A non-finite input never reaches the solver.
    assert_eq!(
        interpolate(
            &[[0.0, 0.0, 0.0], [f64::NAN, 0.0, 0.0], [1.0, 1.0, 0.0]],
            1,
            KnotMode::Uniform,
            &limits()
        ),
        Err(Error::Invalid("nonfinite coordinate"))
    );
}

#[test]
fn parameters_are_normalized_and_strictly_increasing() {
    let points = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
    // A repeated point cannot define a chord: chord length falls back to even steps.
    let chord = parameters(&points, KnotMode::ChordLength, false).unwrap();
    assert_eq!(chord.first(), Some(&0.0));
    assert_eq!(chord.last(), Some(&1.0));
    assert!(chord.windows(2).all(|pair| pair[1] > pair[0]), "{chord:?}");
    let uniform = parameters(&points, KnotMode::Uniform, false).unwrap();
    assert_eq!(uniform, vec![0.0, 0.5, 1.0]);
    assert_eq!(
        parameters(&[[0.0, 0.0, 0.0]], KnotMode::Uniform, false),
        Err(Error::Invalid("interpolation point count"))
    );
}

#[test]
fn the_basis_is_a_partition_of_unity_inside_the_domain() {
    // Clamped cubic with one interior knot: five poles, nine knots.
    let knots = [0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0];
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let mut sum = 0.0;
        for index in 0..5 {
            sum += basis(3, &knots, index, t).unwrap();
        }
        assert!((sum - 1.0).abs() < 1e-12, "t={t}: {sum}");
    }
    assert_eq!(basis(3, &knots, 8, 0.5), Err(Error::Invalid("basis index")));
}

/* ------------------------- circle ------------------------- */

#[test]
fn a_circle_through_three_points_has_the_circumscribed_centre_and_radius() {
    // Points on the circle of radius 5 centred at (0,0,0).
    let curve = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let Curve::Circle {
        center,
        radius,
        domain,
        x,
        y,
        ..
    } = &curve
    else {
        panic!("expected an analytic circle, found {curve:?}");
    };
    assert!(close(*center, [0.0, 0.0, 0.0], 1e-9));
    assert!((radius - 5.0).abs() < 1e-9);
    assert_eq!(*domain, [0.0, std::f64::consts::TAU]);
    // An orthonormal frame: unit axes, perpendicular, right handed with the normal.
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    assert!((dot(*x, *x) - 1.0).abs() < 1e-12);
    assert!((dot(*y, *y) - 1.0).abs() < 1e-12);
    assert!(dot(*x, *y).abs() < 1e-12);
    // The parameter origin sits on the first input point.
    let start = evaluate(&curve, domain[0]);
    assert!(close(start, [5.0, 0.0, 0.0], 1e-9), "{start:?}");
    Geometry::Curve(curve).validate(&limits()).unwrap();
}

#[test]
fn a_circle_needs_three_distinct_non_collinear_points() {
    assert_eq!(
        circle_through([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
        Err(Error::Invalid("degenerate circle"))
    );
    assert_eq!(
        circle_through([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
        Err(Error::Invalid("degenerate circle"))
    );
    assert_eq!(
        circle_through([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [f64::INFINITY, 0.0, 0.0]),
        Err(Error::Invalid("nonfinite coordinate"))
    );
}

/* ------------------------- polyline / rectangle ------------------------- */

#[test]
fn a_polyline_is_one_line_per_pair_and_refuses_repeats() {
    let curve = polyline(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]).unwrap();
    let Curve::Polycurve { segments, .. } = &curve else {
        panic!("expected a polycurve, found {curve:?}");
    };
    assert_eq!(segments.len(), 2);
    assert_eq!(
        polyline(&[[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]),
        Err(Error::Invalid("degenerate line"))
    );
    assert_eq!(
        polyline(&[[0.0, 0.0, 0.0]]),
        Err(Error::Invalid("interpolation point count"))
    );
}

#[test]
fn a_rectangle_is_closed_and_keeps_its_frame() {
    let curve = rectangle(
        [10.0, 20.0, 30.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        4.0,
        2.0,
    )
    .unwrap();
    let Curve::Polycurve { segments, .. } = &curve else {
        panic!("expected a polycurve, found {curve:?}");
    };
    assert_eq!(segments.len(), 4);
    let mut corners = Vec::new();
    for segment in segments {
        let Curve::Line { start, end, .. } = segment else {
            panic!("rectangle segments must be lines");
        };
        corners.push((*start, *end));
    }
    assert!(close(corners[0].0, [10.0, 20.0, 30.0], 1e-12));
    assert!(close(corners[0].1, [10.0, 24.0, 30.0], 1e-12));
    assert!(close(corners[1].1, [10.0, 24.0, 32.0], 1e-12));
    assert!(close(corners[2].1, [10.0, 20.0, 32.0], 1e-12));
    assert!(
        close(corners[3].1, corners[0].0, 1e-12),
        "the last segment closes the loop"
    );
    for pair in corners.windows(2) {
        assert_eq!(pair[0].1, pair[1].0, "segments are contiguous");
    }
    Geometry::Curve(curve).validate(&limits()).unwrap();
}

#[test]
fn a_rectangle_refuses_a_non_orthonormal_frame_or_bad_size() {
    assert_eq!(
        rectangle([0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], 2.0, 2.0),
        Err(Error::Invalid("rectangle frame"))
    );
    assert_eq!(
        rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0, 2.0),
        Err(Error::Invalid("rectangle size"))
    );
    assert_eq!(
        rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], f64::NAN, 2.0),
        Err(Error::Invalid("rectangle size"))
    );
}

/* ------------------------- arc ------------------------- */

#[test]
fn an_arc_through_three_points_spans_the_directed_angles() {
    // A quarter turn through the first quadrant of the unit circle.
    let diagonal = std::f64::consts::FRAC_1_SQRT_2;
    let curve = arc_through([1.0, 0.0, 0.0], [diagonal, diagonal, 0.0], [0.0, 1.0, 0.0]).unwrap();
    let Curve::Circle {
        center,
        radius,
        domain,
        angle_domain,
        ..
    } = &curve
    else {
        panic!("expected a circle, found {curve:?}");
    };
    assert!(close(*center, [0.0, 0.0, 0.0], 1e-9));
    assert!((radius - 1.0).abs() < 1e-9);
    assert_eq!(*domain, [0.0, 1.0]);
    let angles = angle_domain.expect("an arc carries an angle map");
    let sweep = angles[1] - angles[0];
    assert!(
        (sweep - std::f64::consts::FRAC_PI_2).abs() < 1e-9,
        "expected a quarter turn, got {angles:?}"
    );
    // The parameter map is increasing, as the evaluator requires.
    assert!(sweep > 0.0);
    // Endpoints and the middle input point are on the parameter image.
    assert!(close(evaluate(&curve, 0.0), [1.0, 0.0, 0.0], 1e-9));
    assert!(close(evaluate(&curve, 1.0), [0.0, 1.0, 0.0], 1e-9));
    let middle = evaluate(&curve, 0.5);
    assert!(close(middle, [diagonal, diagonal, 0.0], 1e-9), "{middle:?}");
    Geometry::Curve(curve).validate(&limits()).unwrap();
}

#[test]
fn a_clockwise_arc_is_mirrored_so_the_parameter_map_stays_increasing() {
    // Same three points listed the other way round: the arc must still be valid.
    let diagonal = std::f64::consts::FRAC_1_SQRT_2;
    let curve = arc_through([0.0, 1.0, 0.0], [diagonal, diagonal, 0.0], [1.0, 0.0, 0.0]).unwrap();
    let Curve::Circle { angle_domain, .. } = &curve else {
        panic!("expected a circle");
    };
    let angles = angle_domain.expect("arc");
    assert!(angles[1] > angles[0], "{angles:?}");
    assert!((angles[1] - angles[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert!(close(evaluate(&curve, 0.0), [0.0, 1.0, 0.0], 1e-9));
    assert!(close(evaluate(&curve, 1.0), [1.0, 0.0, 0.0], 1e-9));
}

#[test]
fn a_degenerate_arc_is_refused() {
    assert_eq!(
        arc_through([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
        Err(Error::Invalid("degenerate circle"))
    );
    // The endpoints coincide: that is a closed circle, not an arc, and it is refused
    // (the degenerate centre is caught before the sweep is measured).
    assert!(arc_through([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]).is_err());
}

/* ------------------------- ellipse ------------------------- */

#[test]
fn an_ellipse_is_an_exact_rational_nurbs_satisfying_its_equation() {
    let curve = ellipse(
        [10.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    assert_eq!(curve.degree, 2);
    assert_eq!(curve.poles.len(), 9);
    assert!(!curve.periodic, "closed but not certified periodic");
    let shape = Curve::Nurbs(curve);
    for step in 0..16 {
        let t = step as f64 / 16.0;
        let point = evaluate(&shape, t);
        // The frame is world +Y (semi-axis 4) and world +Z (semi-axis 2), origin (10,0,0).
        let axis_x = point[1];
        let axis_y = point[2];
        let equation = (axis_x / 4.0).powi(2) + (axis_y / 2.0).powi(2);
        assert!(
            (equation - 1.0).abs() < 1e-9,
            "t={t}: point {point:?} gives {equation}"
        );
    }
    // The start point sits on the +x semi-axis in the frame.
    assert!(close(evaluate(&shape, 0.0), [10.0, 4.0, 0.0], 1e-9));
    assert!(close(evaluate(&shape, 0.25), [10.0, 0.0, 2.0], 1e-9));
    Geometry::Curve(shape).validate(&limits()).unwrap();
}

#[test]
fn an_ellipse_refuses_a_bad_frame_or_radius() {
    assert_eq!(
        ellipse(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            2.0,
            1.0,
            &limits()
        ),
        Err(Error::Invalid("ellipse frame"))
    );
    assert_eq!(
        ellipse(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            0.0,
            1.0,
            &limits()
        ),
        Err(Error::Invalid("ellipse radius"))
    );
}

/* ------------------------- rebuild ------------------------- */

#[test]
fn a_rebuilt_curve_stays_close_to_its_source() {
    let source = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, -5.0, 0.0]).unwrap();
    let rebuilt = rebuild(&source, 3, 24, KnotMode::ChordLength, &limits()).unwrap();
    let shape = Curve::Nurbs(rebuilt);
    // Compare the rebuilt curve with the analytic circle on the same parameters.
    for step in 0..=16 {
        let fraction = step as f64 / 16.0;
        let a = evaluate_curve(
            &source,
            Parameter::Fraction(fraction),
            0,
            Transform::identity(),
            &limits(),
        )
        .unwrap()[0];
        let b = evaluate_curve(
            &shape,
            Parameter::Fraction(fraction),
            0,
            Transform::identity(),
            &limits(),
        )
        .unwrap()[0];
        let deviation =
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        assert!(
            deviation < 0.05,
            "fraction {fraction}: deviation {deviation}"
        );
    }
}

#[test]
fn a_rebuild_refuses_too_few_points() {
    let source = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, -5.0, 0.0]).unwrap();
    for points in [0usize, 1, 3] {
        assert_eq!(
            rebuild(&source, 3, points, KnotMode::Uniform, &limits()),
            Err(Error::Invalid("rebuild point count")),
            "{points}"
        );
    }
}

/* ------------------------- conversion ------------------------- */

fn sample(shape: &Curve, fractions: &[f64]) -> Vec<[f64; 3]> {
    fractions
        .iter()
        .map(|fraction| {
            evaluate_curve(
                shape,
                Parameter::Fraction(*fraction),
                0,
                Transform::identity(),
                &limits(),
            )
            .unwrap()[0]
        })
        .collect()
}

fn assert_same_geometry(before: &Curve, after: &Curve, label: &str) {
    let fractions: Vec<f64> = (0..=20).map(|step| step as f64 / 20.0).collect();
    let a = sample(before, &fractions);
    let b = sample(after, &fractions);
    for (index, fraction) in fractions.iter().enumerate() {
        assert!(
            close(a[index], b[index], 1e-9),
            "{label}: fraction {fraction} moved {:?} -> {:?}",
            a[index],
            b[index]
        );
    }
}

#[test]
fn inserting_a_knot_does_not_move_the_curve() {
    let source = interpolate(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [3.0, 2.0, 0.0],
            [4.0, 0.0, 0.0],
            [5.0, 1.0, 0.0],
        ],
        3,
        KnotMode::ChordLength,
        &limits(),
    )
    .unwrap();
    let before = Curve::Nurbs(source.clone());
    for times in [1usize, 2, 3] {
        let inserted = insert_knot(&source, 0.4, times, &limits()).unwrap();
        assert_eq!(inserted.poles.len(), source.poles.len() + times);
        assert_eq!(inserted.knots.len(), source.knots.len() + times);
        assert_same_geometry(&before, &Curve::Nurbs(inserted), "insert_knot");
    }
    // A parameter outside the domain is refused; inserting zero times is the identity.
    assert_eq!(
        insert_knot(&source, 1.5, 1, &limits()),
        Err(Error::Invalid("knot insertion parameter"))
    );
    assert_eq!(insert_knot(&source, 0.4, 0, &limits()).unwrap(), source);
}

#[test]
fn elevating_the_degree_does_not_move_the_curve() {
    let source = interpolate(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [3.0, 2.0, 0.0],
            [4.0, 0.0, 0.0],
            [5.0, 1.0, 0.0],
        ],
        3,
        KnotMode::ChordLength,
        &limits(),
    )
    .unwrap();
    let before = Curve::Nurbs(source.clone());
    let elevated = elevate_degree(&source, &limits()).unwrap();
    assert_eq!(elevated.degree, 4);
    assert!(elevated.poles.len() > source.poles.len());
    assert_same_geometry(&before, &Curve::Nurbs(elevated.clone()), "elevate_degree");
    Geometry::Curve(Curve::Nurbs(elevated))
        .validate(&limits())
        .unwrap();
}

#[test]
fn a_fit_with_as_many_control_points_as_samples_passes_through_them() {
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 2.0, 0.0],
        [3.0, 2.0, 0.0],
        [4.0, 0.0, 0.0],
        [5.0, 1.0, 0.0],
    ];
    let curve = fit_points(&points, 3, points.len(), KnotMode::ChordLength, &limits()).unwrap();
    let parameters = parameters(&points, KnotMode::ChordLength, false).unwrap();
    let shape = Curve::Nurbs(curve);
    for (point, parameter) in points.iter().zip(&parameters) {
        assert!(close(evaluate(&shape, *parameter), *point, 1e-8));
    }
}

#[test]
fn a_fit_with_fewer_control_points_keeps_the_ends_and_stays_close() {
    // A dense sample of an open half-circle arc: the ends are interpolated and the
    // interior only approximates the source. (A closed circle is not a fit target: its
    // first and last samples coincide, which would force a degenerate closure.)
    let circle = arc_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let mut samples = Vec::new();
    for step in 0..=40 {
        let fraction = step as f64 / 40.0;
        samples.push(
            evaluate_curve(
                &circle,
                Parameter::Fraction(fraction),
                0,
                Transform::identity(),
                &limits(),
            )
            .unwrap()[0],
        );
    }
    let curve = fit_points(&samples, 3, 12, KnotMode::ChordLength, &limits()).unwrap();
    let shape = Curve::Nurbs(curve);
    assert!(
        close(evaluate(&shape, 0.0), samples[0], 1e-9),
        "start is exact"
    );
    assert!(
        close(evaluate(&shape, 1.0), *samples.last().unwrap(), 1e-9),
        "end is exact"
    );
    // A cubic fit of a half turn with 12 control points stays within 0.05 mm.
    let fractions: Vec<f64> = (0..=20).map(|step| step as f64 / 20.0).collect();
    let source = sample(&circle, &fractions);
    let fitted = sample(&shape, &fractions);
    for index in 0..fractions.len() {
        let deviation = ((source[index][0] - fitted[index][0]).powi(2)
            + (source[index][1] - fitted[index][1]).powi(2)
            + (source[index][2] - fitted[index][2]).powi(2))
        .sqrt();
        assert!(
            deviation < 0.05,
            "fraction {}: {deviation}",
            fractions[index]
        );
    }
}

#[test]
fn a_fit_refuses_an_impossible_control_point_count() {
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
    ];
    assert_eq!(
        fit_points(&points, 3, 3, KnotMode::Uniform, &limits()),
        Err(Error::Invalid("fit control point count"))
    );
    assert_eq!(
        fit_points(&points, 3, 9, KnotMode::Uniform, &limits()),
        Err(Error::Invalid("fit control point count"))
    );
    assert_eq!(
        fit_points(&points, 0, 4, KnotMode::Uniform, &limits()),
        Err(Error::Invalid("fit degree"))
    );
}
