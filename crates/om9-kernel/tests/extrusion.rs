// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits, curve_build, curve_ops, geometry::Curve};
use om9_kernel::{creation, extrusion};
use std::f64::consts::PI;

#[test]
fn exact_extrusion_retains_rational_basis_and_reversal() {
    let limits = Limits::default();
    let ellipse = Curve::Nurbs(
        curve_build::ellipse(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            2.0,
            3.0,
            &limits,
        )
        .unwrap(),
    );
    let rectangle =
        curve_build::rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0, 3.0).unwrap();
    let mut periodic = match &ellipse {
        Curve::Nurbs(n) => n.clone(),
        _ => unreachable!(),
    };
    periodic.periodic = true;
    periodic.seam = Some(0.125);
    for (name, curve, area) in [
        ("ellipse", ellipse.clone(), 6.0 * PI),
        (
            "ellipse_reversed",
            curve_ops::reverse_curve(&ellipse, &limits).unwrap(),
            6.0 * PI,
        ),
        (
            "rectangle_reversed",
            curve_ops::reverse_curve(&rectangle, &limits).unwrap(),
            6.0,
        ),
    ] {
        for height in [-5.0, 5.0] {
            let body = extrusion::solid(&curve, [0.0, 0.0, height], 0.01, &limits).unwrap();
            let measured = creation::measure(&body, &limits).unwrap();
            let ray = Curve::Line {
                start: [-4.0, 0.0, height / 2.0],
                end: [4.0, 0.0, height / 2.0],
                domain: [0.0, 1.0],
            };
            if name.starts_with("ellipse") {
                let hits = om9_kernel::intersect::curve_brep_fractions(&body, &ray, 1e-9, &limits)
                    .unwrap();
                assert_eq!(hits.len(), 2);
                assert!(
                    (hits[0] - 0.25).abs() < 1e-8 && (hits[1] - 0.75).abs() < 1e-8,
                    "{name}: {hits:?}"
                );
                println!("{name} exact side intersections: {hits:?}");
            }
            assert_eq!(measured.solids, 1);
            assert!(measured.closed);
            assert_eq!(measured.naked_edges, 0);
            assert!(
                (measured.volume_mm3 / (area * height.abs()) - 1.0).abs() < 1e-10,
                "{name}: {}",
                measured.volume_mm3
            );
            println!(
                "exact_extrusion {name} height={height} volume={:.12}",
                measured.volume_mm3
            );
        }
    }
    // The existing core evaluator explicitly refuses uncertified periodic continuity.
    assert!(extrusion::solid(&Curve::Nurbs(periodic), [0.0, 0.0, 5.0], 0.01, &limits).is_err());
}

#[test]
fn exact_extrusion_refuses_open_nonplanar_and_disconnected_boundaries() {
    let limits = Limits::default();
    let open = Curve::Line {
        start: [0.0; 3],
        end: [2.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let nonplanar = curve_build::polyline(&[
        [0.0; 3],
        [2.0, 0.0, 0.0],
        [2.0, 2.0, 1.0],
        [0.0, 2.0, 0.0],
        [0.0; 3],
    ])
    .unwrap();
    let disconnected = Curve::Polycurve {
        segments: vec![
            open.clone(),
            Curve::Line {
                start: [5.0, 0.0, 0.0],
                end: [0.0; 3],
                domain: [0.0, 1.0],
            },
        ],
        reversed: false,
    };
    for curve in [open, nonplanar, disconnected] {
        assert!(extrusion::solid(&curve, [0.0, 0.0, 5.0], 0.01, &limits).is_err());
    }
    let circle =
        curve_build::circle_on_frame([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0).unwrap();
    for direction in [[0.0; 3], [f64::NAN, 0.0, 1.0], [1.0, 0.0, 0.0]] {
        assert!(extrusion::solid(&circle, direction, 0.01, &limits).is_err());
    }
}
