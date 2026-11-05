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
fn reseamed_nurbs_brep_intersection_fractions_follow_curve_origin() {
    let limits=Limits::default();
    let body=creation::primitive(Primitive::Box{x_mm:10.,y_mm:10.,z_mm:10.},0.05,&limits).unwrap();
    for reversed in [false,true] {
        let mut n=om9_core::curve_build::ellipse([5.,5.,5.],[1.,0.,0.],[0.,1.,0.],8.,3.,&limits).unwrap();n.reversed=reversed;
        let base=Curve::Nurbs(n);let curve=om9_core::curve_ops::set_seam(&base,0.31,1e-9,&limits).unwrap();
        let hits=curve_brep_fractions(&body,&curve,1e-8,&limits).unwrap();assert_eq!(hits.len(),4,"{hits:?}");
        for f in hits {let p=om9_core::curve_ops::point_at(&curve,f,&limits).unwrap();
            assert!((p[0].abs().min((p[0]-10.).abs()))<1e-6,"{f} {p:?}");}
    }
}

#[test]
fn repeated_periodic_nurbs_intersects_brep_in_active_parameterization() {
    use om9_core::{curve_ops,geometry::NurbsCurve};
    let limits=Limits::default();
    let body=creation::primitive(Primitive::Box{x_mm:10.,y_mm:20.,z_mm:10.},0.05,&limits).unwrap();
    let mut n=NurbsCurve{degree:2,poles:vec![[-8.,5.,5.],[18.,5.,5.],[18.,15.,5.],[-8.,15.,5.],[-8.,5.,5.],[18.,5.,5.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed:false,seam:None};
    for reversed in [false,true] {for seam in [false,true] {
        n.reversed=reversed;let base=Curve::Nurbs(n.clone());
        let curve=if seam {curve_ops::set_seam(&base,0.31,1e-9,&limits).unwrap()} else {base};
        let hits=curve_brep_fractions(&body,&curve,1e-8,&limits).unwrap();assert_eq!(hits.len(),4,"{reversed} {seam} {hits:?}");
        for f in hits {let p=curve_ops::point_at(&curve,f,&limits).unwrap();
            assert!(p[0].abs().min((p[0]-10.).abs())<1e-6,"{f} {p:?}");}
    }}
    n.domain=[0.2,3.8];n.reversed=false;
    let trimmed=Curve::Nurbs(n.clone());let hits=curve_brep_fractions(&body,&trimmed,1e-8,&limits).unwrap();
    assert!(!hits.is_empty());for f in hits {let p=curve_ops::point_at(&trimmed,f,&limits).unwrap();assert!(p[0].abs().min((p[0]-10.).abs())<1e-6);}
    n.poles[5][0]+=0.1;assert!(curve_brep_fractions(&body,&Curve::Nurbs(n),1e-8,&limits).is_err());
}

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
