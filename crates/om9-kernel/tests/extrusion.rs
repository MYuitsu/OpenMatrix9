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
fn certified_periodic_rational_cubic_has_continuous_jet_and_extrudes() {
    use om9_core::geometry::NurbsCurve;
    let limits=Limits::default();
    let curve=Curve::Nurbs(NurbsCurve {
        degree:3,poles:vec![[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.],[0.,-4.,0.],[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.]],
        weights:vec![1.,0.7,1.3,0.9,1.,0.7,1.3],knots:(0..=10).map(|v|v as f64).collect(),
        domain:[3.,7.],periodic:true,reversed:false,seam:None,
    });
    let a=curve_ops::fraction_derivatives(&curve,0.,2,&limits).unwrap();
    let b=curve_ops::fraction_derivatives(&curve,1.,2,&limits).unwrap();
    for k in 0..=2 {for axis in 0..3 {assert!((a[k][axis]-b[k][axis]).abs()<1e-9,"D{k} {a:?} {b:?}");}}
    assert!(curve_ops::fraction_derivatives(&curve,0.,3,&limits).is_err());
    let points=curve_ops::sample_curve(&curve,4097,&limits).unwrap();
    let area=0.5*points.windows(2).map(|p|p[0][0]*p[1][1]-p[1][0]*p[0][1]).sum::<f64>().abs();
    for reversed in [false,true] {
        let source=if reversed {curve_ops::reverse_curve(&curve,&limits).unwrap()} else {curve.clone()};
        let body=extrusion::solid(&source,[0.,0.,2.],0.01,&limits).unwrap();
        let m=creation::measure(&body,&limits).unwrap();
        assert!(m.closed && m.solids==1);assert!((m.volume_mm3-2.*area).abs()<1e-4,"{m:?} area {area}");
    }
    let Curve::Nurbs(mut doubled)=curve else {unreachable!()};
    doubled.knots=vec![0.,1.,2.,2.,4.,5.,6.,6.,8.,9.,10.];doubled.domain=[2.,6.];
    let doubled=Curve::Nurbs(doubled);
    let a=curve_ops::fraction_derivatives(&doubled,0.,1,&limits).unwrap();
    let b=curve_ops::fraction_derivatives(&doubled,1.,1,&limits).unwrap();
    for k in 0..=1 {for axis in 0..3 {assert!((a[k][axis]-b[k][axis]).abs()<1e-9,"D{k} {a:?} {b:?}");}}
    assert!(curve_ops::fraction_derivatives(&doubled,0.,2,&limits).is_err());
    assert!(curve_ops::validate_ring_rail(&doubled,1e-7,&limits).is_ok());
    assert!(extrusion::planar_face(&doubled,0.01,&limits).is_ok());
    let points=curve_ops::sample_curve(&doubled,4097,&limits).unwrap();
    let area=0.5*points.windows(2).map(|p|p[0][0]*p[1][1]-p[1][0]*p[0][1]).sum::<f64>().abs();
    for reversed in [false,true] {for height in [-2.,2.] {
        let source=if reversed {curve_ops::reverse_curve(&doubled,&limits).unwrap()} else {doubled.clone()};
        let body=extrusion::solid(&source,[0.,0.,height],0.01,&limits).unwrap();let m=creation::measure(&body,&limits).unwrap();
        assert!(m.closed && m.solids==1);assert!((m.volume_mm3-2.*area).abs()<1e-4,"{m:?} area {area}");
    }}
    for (base,scale) in [(10.,0.01),(10.,1000.),(0.,1e-9),(1e6,1e-4)] {
        let Curve::Nurbs(mut scaled)=doubled.clone() else {unreachable!()};
        for knot in &mut scaled.knots {*knot=base+*knot*scale;}scaled.domain=scaled.domain.map(|u|base+u*scale);
        let curve=Curve::Nurbs(scaled);let body=extrusion::solid(&curve,[0.,0.,2.],0.01,&limits).unwrap();
        let m=creation::measure(&body,&limits).unwrap_or_else(|e|panic!("base {base} scale {scale}: {e:?}"));
        assert!((m.volume_mm3-2.*area).abs()<1e-4,"base {base} scale {scale} {m:?}");
    }
    let Curve::Nurbs(mut moved)=doubled.clone() else {unreachable!()};
    for p in &mut moved.poles {p[0]+=100.;p[1]-=200.;p[2]+=30.;}
    let body=extrusion::solid(&Curve::Nurbs(moved),[0.,0.,-2.],0.01,&limits).unwrap();
    let m=creation::measure(&body,&limits).unwrap();
    assert!((m.volume_mm3-2.*area).abs()<1e-4,"translated {m:?}");
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
