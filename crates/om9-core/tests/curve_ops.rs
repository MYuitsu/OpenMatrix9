// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    curve_build::{
        arc_by_center, arc_through, circle_on_frame, circle_through, polygon, polyline, rectangle,
    },
    curve_ops::{
        arc_length, close_curve, curve_endpoints, curve_grips, divide_by_count, divide_by_length,
        explode_curve, insert_control_point, is_closed, join_curves, move_grip, point_at,
        reverse_curve, sample_curve, set_seam, split_curve, tangent_at, trim_curve,
    },
    geometry::{Curve, Point3},
};

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= tolerance)
}

fn line(a: Point3, b: Point3) -> Curve {
    let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, length],
    }
}

#[test]
fn line_length_division_split_and_trim_use_the_declared_domain() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
    assert!((arc_length(&curve, &limits).unwrap() - 5.0).abs() < 1e-10);
    let divisions = divide_by_count(&curve, 5, &limits).unwrap();
    for (index, fraction) in divisions.iter().enumerate() {
        assert!((fraction - index as f64 / 5.0).abs() < 1e-8);
    }
    let (left, right) = split_curve(&curve, 0.5, &limits).unwrap();
    assert!((arc_length(&left, &limits).unwrap() - 2.5).abs() < 1e-9);
    assert!((arc_length(&right, &limits).unwrap() - 2.5).abs() < 1e-9);
    assert!(close(
        curve_endpoints(&left, &limits).unwrap()[1],
        [1.5, 2.0, 0.0],
        1e-10
    ));
    let trimmed = trim_curve(&curve, 0.25, 0.75, &limits).unwrap();
    assert!((arc_length(&trimmed, &limits).unwrap() - 2.5).abs() < 1e-9);
}

#[test]
fn circle_length_division_and_reverse_keep_the_angular_direction() {
    let limits = Limits::default();
    let circle = circle_through([10.0, 0.0, 0.0], [0.0, 10.0, 0.0], [-10.0, 0.0, 0.0]).unwrap();
    let tau = std::f64::consts::TAU;
    assert!((arc_length(&circle, &limits).unwrap() - tau * 10.0).abs() < 1e-6);
    let quarters = divide_by_count(&circle, 4, &limits).unwrap();
    let at_quarter = point_at(&circle, quarters[1], &limits).unwrap();
    assert!(close(at_quarter, [0.0, 10.0, 0.0], 1e-6));
    let reversed = reverse_curve(&circle, &limits).unwrap();
    let before = point_at(&circle, 0.25, &limits).unwrap();
    let after = point_at(&reversed, 0.75, &limits).unwrap();
    assert!(close(before, after, 1e-8));
}

#[test]
fn nurbs_split_and_control_point_insertion_preserve_geometry() {
    let limits = Limits::default();
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 3.0, 0.0],
        [4.0, 2.0, 0.0],
        [6.0, 0.0, 0.0],
    ];
    let curve = Curve::Nurbs(
        om9_core::curve_build::interpolate(
            &points,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            &limits,
        )
        .unwrap(),
    );
    let inserted = insert_control_point(&curve, 0.5, &limits).unwrap();
    for step in 0..=8 {
        let fraction = step as f64 / 8.0;
        assert!(close(
            point_at(&curve, fraction, &limits).unwrap(),
            point_at(&inserted, fraction, &limits).unwrap(),
            1e-8
        ));
    }
    let (left, right) = split_curve(&curve, 0.5, &limits).unwrap();
    let total = arc_length(&curve, &limits).unwrap();
    let parts = arc_length(&left, &limits).unwrap() + arc_length(&right, &limits).unwrap();
    assert!((parts - total).abs() < 1e-6);
}

#[test]
fn join_explode_close_and_seam_cover_a_closed_polycurve() {
    let limits = Limits::default();
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let joined = join_curves(&[first, second], 1e-9, &limits).unwrap();
    assert!((arc_length(&joined, &limits).unwrap() - 20.0).abs() < 1e-9);
    assert_eq!(explode_curve(&joined, &limits).unwrap().len(), 2);
    let closed = close_curve(&joined, 1e-9, &limits).unwrap();
    assert!(is_closed(&closed, 1e-9, &limits).unwrap());
    let closed_length = arc_length(&closed, &limits).unwrap();
    assert!(
        (closed_length - 20.0 - (200.0f64).sqrt()).abs() < 1e-9,
        "closed length {closed_length}"
    );
    let square = rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0).unwrap();
    let seamed = set_seam(&square, 0.25, 1e-9, &limits).unwrap();
    let old_quarter = point_at(&square, 0.25, &limits).unwrap();
    let new_start = curve_endpoints(&seamed, &limits).unwrap()[0];
    assert!(close(old_quarter, new_start, 1e-9));
}

#[test]
fn grips_expose_poles_edit_points_and_allow_a_move() {
    let limits = Limits::default();
    let curve = Curve::Nurbs(
        om9_core::curve_build::interpolate(
            &[[0.0; 3], [1.0, 2.0, 0.0], [3.0, 1.0, 0.0], [4.0, 0.0, 0.0]],
            3,
            om9_core::curve_build::KnotMode::Uniform,
            &limits,
        )
        .unwrap(),
    );
    let grips = curve_grips(&curve, 1e-9, &limits).unwrap();
    assert_eq!(grips.poles.len(), 4);
    assert_eq!(grips.edit_points.len(), 4);
    assert!(!grips.closed);
    let moved = move_grip(&curve, 1, [1.0, 5.0, 0.0], &limits).unwrap();
    let moved_grips = curve_grips(&moved, 1e-9, &limits).unwrap();
    assert!(close(moved_grips.poles[1], [1.0, 5.0, 0.0], 0.0));
}

#[test]
fn divide_by_length_and_endpoint_tangent_are_measurable() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let fractions = divide_by_length(&curve, 3.0, &limits).unwrap();
    assert_eq!(fractions.len(), 5);
    assert!((fractions[1] - 0.3).abs() < 1e-8);
    assert!((fractions[3] - 0.9).abs() < 1e-8);
    assert!((fractions[4] - 1.0).abs() < 1e-12);
    assert!(close(
        tangent_at(&curve, 0.5, &limits).unwrap(),
        [1.0, 0.0, 0.0],
        1e-12
    ));
    let samples = sample_curve(&curve, 11, &limits).unwrap();
    assert_eq!(samples.len(), 11);
    assert!(close(samples[5], [5.0, 0.0, 0.0], 1e-12));
}

#[test]
fn arc_through_three_points_has_the_expected_length() {
    let limits = Limits::default();
    let quarter = arc_through([10.0, 0.0, 0.0], [0.0, 10.0, 0.0], [-10.0, 0.0, 0.0]).unwrap();
    let expected = std::f64::consts::PI * 10.0;
    assert!((arc_length(&quarter, &limits).unwrap() - expected).abs() < 1e-6);
}

#[test]
fn polyline_joining_refuses_a_gap() {
    let limits = Limits::default();
    let a = line([0.0; 3], [1.0, 0.0, 0.0]);
    let b = line([2.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
    assert!(join_curves(&[a, b], 1e-9, &limits).is_err());
    let poly = polyline(&[[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]).unwrap();
    assert!((arc_length(&poly, &limits).unwrap() - 2.0).abs() < 1e-9);
}

#[test]
fn framed_circle_arc_and_polygon_creation_are_measurable() {
    let limits = Limits::default();
    let circle = circle_on_frame([1.0, 2.0, 3.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 4.0).unwrap();
    assert!((arc_length(&circle, &limits).unwrap() - std::f64::consts::TAU * 4.0).abs() < 1e-6);
    let quarter = point_at(&circle, 0.25, &limits).unwrap();
    assert!(close(quarter, [1.0, 6.0, 3.0], 1e-9));
    let arc = arc_by_center(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        2.0,
        0.0,
        std::f64::consts::FRAC_PI_2,
    )
    .unwrap();
    assert!((arc_length(&arc, &limits).unwrap() - std::f64::consts::PI).abs() < 1e-6);
    let square = polygon(&[
        [0.0; 3],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ])
    .unwrap();
    assert!(is_closed(&square, 1e-9, &limits).unwrap());
    assert!((arc_length(&square, &limits).unwrap() - 40.0).abs() < 1e-9);
    assert!(polygon(&[[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]).is_err());
}
