// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    curve_build::{KnotMode, circle_through, interpolate},
    curve_modify::{
        Continuity, blend_curves, chamfer_lines, fillet_lines, match_curve, offset_curve,
        rebuild_curve,
    },
    curve_ops::{arc_length, curve_endpoints, point_at, tangent_at},
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
fn line_and_circle_offsets_stay_exact() {
    let limits = Limits::default();
    let horizontal = line([0.0; 3], [10.0, 0.0, 0.0]);
    let offset = offset_curve(&horizontal, 2.0, [0.0, 0.0, 1.0], 16, &limits).unwrap();
    let ends = curve_endpoints(&offset, &limits).unwrap();
    assert!(close(ends[0], [0.0, -2.0, 0.0], 1e-12));
    assert!(close(ends[1], [10.0, -2.0, 0.0], 1e-12));
    let circle = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let grown = offset_curve(&circle, 2.0, [0.0, 0.0, 1.0], 16, &limits).unwrap();
    let expected = std::f64::consts::TAU * 7.0;
    assert!((arc_length(&grown, &limits).unwrap() - expected).abs() < 1e-6);
    let end = point_at(&grown, 0.0, &limits).unwrap();
    assert!((end[0].hypot(end[1]) - 7.0).abs() < 1e-9);
}

#[test]
fn nurbs_offset_tracks_the_requested_distance() {
    let limits = Limits::default();
    let circle = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let points = om9_core::curve_ops::sample_curve(&circle, 24, &limits).unwrap();
    let nurbs = Curve::Nurbs(interpolate(&points, 3, KnotMode::ChordLength, &limits).unwrap());
    let offset = offset_curve(&nurbs, 1.0, [0.0, 0.0, 1.0], 48, &limits).unwrap();
    for step in 0..=12 {
        let point = point_at(&offset, step as f64 / 12.0, &limits).unwrap();
        let radius = point[0].hypot(point[1]);
        assert!((radius - 6.0).abs() < 0.02, "offset radius {radius}");
    }
}

#[test]
fn line_fillet_produces_a_tangent_quarter_arc_and_trimmed_legs() {
    let limits = Limits::default();
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([0.0; 3], [0.0, 10.0, 0.0]);
    let fillet = fillet_lines(&first, &second, 2.0, [0.0, 0.0, 1.0], 1e-7, &limits).unwrap();
    assert!(close(fillet.tangent_points[0], [2.0, 0.0, 0.0], 1e-12));
    assert!(close(fillet.tangent_points[1], [0.0, 2.0, 0.0], 1e-12));
    let arc = curve_endpoints(&fillet.arc, &limits).unwrap();
    assert!(close(arc[0], [2.0, 0.0, 0.0], 1e-9));
    assert!(close(arc[1], [0.0, 2.0, 0.0], 1e-9));
    let quarter = std::f64::consts::FRAC_PI_2 * 2.0;
    assert!((arc_length(&fillet.arc, &limits).unwrap() - quarter).abs() < 1e-6);
    assert!((arc_length(&fillet.first_trimmed, &limits).unwrap() - 8.0).abs() < 1e-9);
    assert!((arc_length(&fillet.second_trimmed, &limits).unwrap() - 8.0).abs() < 1e-9);
}

#[test]
fn line_chamfer_connects_the_requested_distances() {
    let limits = Limits::default();
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([0.0; 3], [0.0, 10.0, 0.0]);
    let chamfer = chamfer_lines(&first, &second, 3.0, 4.0, 1e-7, &limits).unwrap();
    let ends = curve_endpoints(&chamfer.line, &limits).unwrap();
    assert!(close(ends[0], [3.0, 0.0, 0.0], 1e-12));
    assert!(close(ends[1], [0.0, 4.0, 0.0], 1e-12));
    assert!((arc_length(&chamfer.line, &limits).unwrap() - 5.0).abs() < 1e-9);
}

#[test]
fn g1_blend_matches_the_end_tangents() {
    let limits = Limits::default();
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([20.0, 10.0, 0.0], [20.0, 20.0, 0.0]);
    let blend = blend_curves(&first, &second, Continuity::G1, &limits).unwrap();
    let ends = curve_endpoints(&blend, &limits).unwrap();
    assert!(close(ends[0], [10.0, 0.0, 0.0], 1e-9));
    assert!(close(ends[1], [20.0, 10.0, 0.0], 1e-9));
    let start_tangent = tangent_at(&blend, 0.0, &limits).unwrap();
    assert!(close(start_tangent, [1.0, 0.0, 0.0], 1e-9));
    let end_tangent = tangent_at(&blend, 1.0, &limits).unwrap();
    assert!(close(end_tangent, [0.0, 1.0, 0.0], 1e-9));
}

#[test]
fn g2_blend_has_a_continuous_second_derivative_at_the_ends() {
    let limits = Limits::default();
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([20.0, 10.0, 0.0], [20.0, 20.0, 0.0]);
    let blend = blend_curves(&first, &second, Continuity::G2, &limits).unwrap();
    let start = om9_core::curve_ops::fraction_derivatives(&blend, 0.0, 2, &limits).unwrap();
    let end = om9_core::curve_ops::fraction_derivatives(&blend, 1.0, 2, &limits).unwrap();
    assert!(close(start[2], [0.0; 3], 1e-9));
    assert!(close(end[2], [0.0; 3], 1e-9));
}

#[test]
fn match_moves_the_source_end_onto_the_target_with_g1() {
    let limits = Limits::default();
    let source = line([0.0; 3], [10.0, 0.0, 0.0]);
    let target = line([20.0, 10.0, 0.0], [20.0, 20.0, 0.0]);
    let matched = match_curve(&source, true, &target, false, Continuity::G1, &limits).unwrap();
    let ends = curve_endpoints(&matched, &limits).unwrap();
    assert!(close(ends[1], [20.0, 10.0, 0.0], 1e-9));
    let tangent = tangent_at(&matched, 1.0, &limits).unwrap();
    assert!(close(tangent, [0.0, 1.0, 0.0], 1e-9));
    assert!(close(ends[0], [0.0; 3], 1e-9));
}

#[test]
fn rebuild_keeps_the_circle_shape_within_the_sample_tolerance() {
    let limits = Limits::default();
    let circle = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let rebuilt = rebuild_curve(&circle, 48, 3, None, KnotMode::ChordLength, &limits).unwrap();
    for step in 0..=24 {
        let point = point_at(&rebuilt, step as f64 / 24.0, &limits).unwrap();
        let radius = point[0].hypot(point[1]);
        assert!((radius - 5.0).abs() < 0.02, "rebuilt radius {radius}");
    }
}
