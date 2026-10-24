// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT line/NURBS closest-parameter adapter.
#![cfg(feature = "3dm-brep")]

use om9_core::{
    budget::Limits,
    geometry::{Curve, NurbsCurve},
};
use om9_kernel::intersect::{line_curve_fraction, line_nurbs_fraction};

#[test]
fn line_crosses_a_degree_one_nurbs_at_the_expected_fraction() {
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 1,
        poles: vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
        weights: vec![1.0, 1.0],
        knots: vec![0.0, 0.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    });
    let fraction = line_nurbs_fraction(
        &curve,
        [5.0, -5.0, 0.0],
        [5.0, 5.0, 0.0],
        &Limits::default(),
    )
    .expect("line/nurbs");
    assert!((fraction - 0.5).abs() <= 1.0e-6, "fraction {fraction}");
}

#[test]
fn line_and_circle_curves_also_use_the_kernel() {
    let limits = Limits::default();
    let line = Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let fraction =
        line_curve_fraction(&line, [5.0, -5.0, 0.0], [5.0, 5.0, 0.0], &limits).expect("line/line");
    assert!((fraction - 0.5).abs() <= 1.0e-6, "fraction {fraction}");

    let circle = Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 5.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let fraction = line_curve_fraction(&circle, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0], &limits)
        .expect("line/circle");
    assert!((0.0..=1.0).contains(&fraction), "fraction {fraction}");

    let polycurve = Curve::Polycurve {
        segments: vec![
            Curve::Line {
                start: [0.0, 0.0, 0.0],
                end: [5.0, 0.0, 0.0],
                domain: [0.0, 1.0],
            },
            Curve::Line {
                start: [5.0, 0.0, 0.0],
                end: [10.0, 0.0, 0.0],
                domain: [0.0, 1.0],
            },
        ],
        reversed: false,
    };
    let fraction = line_curve_fraction(&polycurve, [7.0, -5.0, 0.0], [7.0, 5.0, 0.0], &limits)
        .expect("line/polycurve");
    assert!(fraction > 0.5, "fraction {fraction}");
}
