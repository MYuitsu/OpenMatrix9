// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent analytic oracles on the serialized OCCT body, including seams and poles.
use om9_core::{budget::Limits, geometry::Curve};
use om9_kernel::{
    brep::{read_brep, source_volume},
    creation::{self, Primitive},
    intersect::curve_brep_fractions,
};
use std::f64::consts::PI;

#[test]
fn ellipsoid_exact_volume_topology_and_surface() {
    let limits = Limits::default();
    for axes in [
        [1.0, 1.0, 1.0],
        [2.0, 3.0, 4.0],
        [3.0, 4.0, 5.0],
        [0.1, 0.2, 0.3],
        [20.0, 0.5, 3.0],
        [4.0, 3.0, 2.0],
    ] {
        let body = creation::primitive(
            Primitive::Ellipsoid {
                x_mm: axes[0],
                y_mm: axes[1],
                z_mm: axes[2],
            },
            0.01,
            &limits,
        )
        .unwrap();
        let measured = creation::measure(&body, &limits).unwrap();
        let expected = 4.0 * PI / 3.0 * axes.iter().product::<f64>();
        println!(
            "axes={axes:?} volume={:.12} expected={expected:.12} relative_error={:.3e} faces={} naked_edges={}",
            measured.volume_mm3,
            (measured.volume_mm3 / expected - 1.0).abs(),
            measured.faces,
            measured.naked_edges
        );
        assert!((measured.volume_mm3 / expected - 1.0).abs() < 1e-10);
        assert_eq!(measured.solids, 1);
        assert_eq!(measured.shells, 1);
        assert_eq!(measured.naked_edges, 0);
        assert!(measured.closed);
        assert!((source_volume(&body, &limits).unwrap() / expected - 1.0).abs() < 1e-10);
        for (axis, radius) in axes.iter().enumerate() {
            assert!((measured.bounds_mm[axis] + radius).abs() < 1e-6);
            assert!((measured.bounds_mm[axis + 3] - radius).abs() < 1e-6);
        }
        // OCCT reads its stored B-Rep again; no recipe or analytic-volume shortcut is available.
        let reloaded = read_brep(body.source(), 0.05, &limits).unwrap();
        assert!(
            (creation::measure(&reloaded, &limits).unwrap().volume_mm3 / expected - 1.0).abs()
                < 1e-10
        );
        // Intersect diameters in 3 axis directions plus a latitude/longitude grid.
        // The normalized implicit ellipsoid equation predicts fractions 1/4 and 3/4.
        let mut directions = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        for longitude in 0..16 {
            for latitude in [-0.9_f64, -0.4, 0.0, 0.4, 0.9] {
                let angle = 2.0 * PI * longitude as f64 / 16.0;
                let radial = (1.0 - latitude * latitude).sqrt();
                directions.push([radial * angle.cos(), radial * angle.sin(), latitude]);
            }
        }
        for direction in &directions {
            let end = std::array::from_fn(|i| 2.0 * axes[i] * direction[i]);
            let start = end.map(|v| -v);
            let ray = Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            };
            let hits = curve_brep_fractions(&reloaded, &ray, 1e-9, &limits).unwrap();
            assert_eq!(hits.len(), 2, "{axes:?} {direction:?}: {hits:?}");
            for (hit, expected) in hits.iter().zip([0.25, 0.75]) {
                assert!(
                    (hit - expected).abs() < 1e-8,
                    "{axes:?} {direction:?}: {hits:?}"
                );
            }
        }
        println!(
            "axes={axes:?} analytic_diameters={} reload=PASS",
            directions.len()
        );
    }
}
