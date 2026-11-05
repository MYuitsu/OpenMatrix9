// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
    curve_build::circle_through,
    curve_ops::point_at,
    curve_surface_ops::{
        boundary_edges, closest_point_on_mesh, closest_point_on_surface, curve_curve_intersections,
        curve_surface_intersections, duplicate_brep_edge, duplicate_isocurve, point_from_two_views,
        pullback_curve, sketch_on_mesh, sketch_on_surface,
    },
    geometry::{Curve, Mesh, Point3},
    surface_kernel::{self, SurfaceIso},
};
use uuid::Uuid;

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= tolerance)
}

#[test]
fn nested_polycurve_closest_and_intersections_preserve_fraction_and_reversal() {
    use om9_core::curve_surface_ops::closest_point_on_curve;
    let limits = Limits::default();
    let nested = Curve::Polycurve {
        segments: vec![
            Curve::Polycurve {
                segments: vec![line([0., 0., 0.], [2., 0., 0.]), line([2., 0., 0.], [4., 0., 0.])],
                reversed: true,
            },
            line([0., 0., 0.], [0., -2., 0.]),
        ],
        reversed: false,
    };
    let cutter = Curve::Polycurve {
        segments: vec![line([3., -1., 0.], [3., 1., 0.])],
        reversed: true,
    };
    let (fraction, point, distance) = closest_point_on_curve(&nested, [3., 0.2, 0.], &limits).unwrap();
    assert!((fraction - 0.125).abs() < 1e-12);
    assert!(close(point, [3., 0., 0.], 1e-12));
    assert!((distance - 0.2).abs() < 1e-12);
    assert!(close(point_at(&nested, fraction, &limits).unwrap(), point, 1e-12));
    assert!(close(om9_core::curve_ops::derivative_at(&nested, fraction, &limits).unwrap(), [-8., 0., 0.], 1e-12));
    for (a, b, expected) in [(&nested, &cutter, [0.125, 0.5]), (&cutter, &nested, [0.5, 0.125])] {
        let hits = curve_curve_intersections(a, b, 1e-9, &limits).unwrap();
        assert_eq!(hits.len(), 1);
        assert!((hits[0].first_fraction - expected[0]).abs() < 1e-12);
        assert!((hits[0].second_fraction - expected[1]).abs() < 1e-12);
        assert!(close(hits[0].point, [3., 0., 0.], 1e-12));
    }
    let reversed = Curve::Polycurve { segments: vec![nested.clone()], reversed: true };
    let hit = curve_curve_intersections(&reversed, &cutter, 1e-9, &limits).unwrap();
    assert_eq!(hit.len(), 1);
    assert!((hit[0].first_fraction - 0.875).abs() < 1e-12);
    assert!(close(om9_core::curve_ops::derivative_at(&reversed, 0.875, &limits).unwrap(), [8., 0., 0.], 1e-12));
    let shallow = Limits { max_depth: 1, ..limits };
    assert!(closest_point_on_curve(&reversed, [3., 0., 0.], &shallow).is_err());
    assert!(curve_curve_intersections(&reversed, &cutter, 1e-9, &shallow).is_err());
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
fn line_intersections_respect_tolerance_at_finite_segment_endpoints() {
    let limits = Limits::default();
    let target = line([0., 0., 0.], [2., 0., 0.]);
    for reverse in [false, true] {
        let ends = if reverse { [[1., 1., 0.], [1., 0.0005, 0.]] } else { [[1., 0.0005, 0.], [1., 1., 0.]] };
        let cutter = line(ends[0], ends[1]);
        for swapped in [false, true] {
            let (a, b) = if swapped { (&cutter, &target) } else { (&target, &cutter) };
            let hits = curve_curve_intersections(a, b, 0.001, &limits).unwrap();
            assert_eq!(hits.len(), 1, "{hits:?}");
            let (target_fraction, cutter_fraction) = if swapped { (hits[0].second_fraction, hits[0].first_fraction) } else { (hits[0].first_fraction, hits[0].second_fraction) };
            assert!((target_fraction - 0.5).abs() < 1e-12);
            assert_eq!(cutter_fraction, if reverse { 1. } else { 0. });
            assert!((hits[0].distance - 0.0005).abs() < 1e-12);
            assert!(curve_curve_intersections(a, b, 0.0001, &limits).unwrap().is_empty());
        }
    }
    let corner = line([2.0004, 0.0003, 0.], [2.0004, 1., 0.]);
    let hit=curve_curve_intersections(&target,&corner,0.001,&limits).unwrap();
    assert_eq!(hit.len(),1);assert_eq!((hit[0].first_fraction,hit[0].second_fraction),(1.,0.));assert!((hit[0].distance-0.0005).abs()<1e-12);
    let skew=line([1.,0.0005,0.0009],[1.,1.,0.0009]);
    assert!(curve_curve_intersections(&target,&skew,0.001,&limits).unwrap().is_empty());
}

fn plane() -> om9_core::geometry::NurbsSurface {
    surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap()
}

#[test]
fn surface_intersection_includes_isolated_curve_endpoints() {
    let limits = Limits::default();
    let surface = plane();
    for (start, end, expected) in [
        ([2., 2., 0.], [2., 2., 5.], 0.),
        ([2., 2., -5.], [2., 2., 0.], 1.),
    ] {
        let hits = curve_surface_intersections(&line(start, end), &surface, 1e-6, &limits).unwrap();
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!((hits[0].curve_fraction - expected).abs() < 1e-12);
    }
    let overlap = line([2., 2., 0.], [8., 2., 0.]);
    assert!(curve_surface_intersections(&overlap, &surface, 1e-6, &limits).is_err());
    let partial_overlap = line([-2., 2., 0.], [12., 2., 0.]);
    assert!(curve_surface_intersections(&partial_overlap, &surface, 1e-6, &limits).is_err());
}

#[test]
fn surface_projection_and_pullback_measure_the_deviation() {
    let limits = Limits::default();
    let surface = plane();
    let projection = closest_point_on_surface(&surface, [2.0, 3.0, 5.0], &limits).unwrap();
    assert!(close(projection.point, [2.0, 3.0, 0.0], 1e-9));
    assert!((projection.distance - 5.0).abs() < 1e-9);
    assert!((projection.parameters[0] - 0.2).abs() < 1e-9);
    assert!((projection.parameters[1] - 0.3).abs() < 1e-9);
    let source = line([1.0, 1.0, 1.0], [9.0, 8.0, 1.0]);
    let pullback = pullback_curve(&source, &surface, 17, 1.0 + 1e-6, &limits).unwrap();
    assert_eq!(pullback.parameters.len(), 17);
    assert!((pullback.max_deviation - 1.0).abs() < 1e-6);
    for point in &pullback.points {
        assert!(point[2].abs() < 1e-9);
    }
    assert!(pullback_curve(&source, &surface, 17, 0.5, &limits).is_err());
}

#[test]
fn tiny_open_multispan_nurbs_retains_its_endpoint_intersection() {
    use om9_core::geometry::NurbsCurve;let limits=Limits::default();
    let curve=Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.5e-11,0.,0.],[5e-11,0.,0.],[7.5e-11,0.,0.],[1e-10,0.,0.]],weights:vec![1.;5],knots:vec![0.,0.,0.,0.5,0.5,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let cutter=line([1e-10,-1e-10,0.],[1e-10,1e-10,0.]);let hits=curve_curve_intersections(&curve,&cutter,1e-13,&limits).unwrap();
    assert_eq!(hits.len(),1,"{hits:?}");assert!((hits[0].first_fraction-1.).abs()<1e-12);
}

#[test]
fn reseamed_quadratic_ellipse_intersections_do_not_duplicate_old_seam() {
    let limits=Limits::default();let cutter=line([-5.,0.,0.],[5.,0.,0.]);
    for reversed in [false,true] {
        let mut n=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],4.,2.,&limits).unwrap();n.reversed=reversed;
        let base=Curve::Nurbs(n);let curve=om9_core::curve_ops::set_seam(&base,0.31,1e-9,&limits).unwrap();
        let hits=curve_curve_intersections(&curve,&cutter,1e-8,&limits).unwrap();assert_eq!(hits.len(),2,"{hits:?}");
        assert!(hits.iter().any(|h|(h.point[0]-4.).abs()<1e-7));assert!(hits.iter().any(|h|(h.point[0]+4.).abs()<1e-7));
        let mut bounded=limits;bounded.max_outputs=2;assert_eq!(curve_curve_intersections(&curve,&cutter,1e-8,&bounded).unwrap().len(),2);
    }
}

#[test]
fn periodic_reseamed_quadratic_line_tangent_is_one_hit() {
    use om9_core::{curve_ops,geometry::NurbsCurve};
    let limits=Limits::default();let cutter=line([1.5,-1.,0.],[1.5,2.,0.]);
    for reversed in [false,true] { for shifted in [false,true] {
        let curve=Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.;6],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:None});
        let curve=if shifted {curve_ops::set_seam(&curve,0.31,1e-9,&limits).unwrap()} else {curve};
        let original=if reversed {0.875_f64} else {0.125_f64};let expected=if shifted {(original-0.31).rem_euclid(1.)} else {original};
        for swap in [false,true] {
            let hits=if swap {curve_curve_intersections(&cutter,&curve,1e-9,&limits)} else {curve_curve_intersections(&curve,&cutter,1e-9,&limits)}.unwrap();
            assert_eq!(hits.len(),1,"reversed={reversed} shifted={shifted} swap={swap}: {hits:?}");
            let f=if swap {hits[0].second_fraction} else {hits[0].first_fraction};assert!((f-expected).abs()<1e-9,"{f} {expected}");assert!(close(hits[0].point,[1.5,0.25,0.],1e-8));
        }
    }}
}

#[test]
fn trimmed_periodic_quadratic_tangent_preserves_active_parameter() {
    use om9_core::geometry::NurbsCurve;let limits=Limits::default();let cutter=line([1.5,-1.,0.],[1.5,2.,0.]);
    for reversed in [false,true] {
        let mut record=NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.;6],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.3,3.7],periodic:true,reversed,seam:None};
        let curve=Curve::Nurbs(record.clone());
        let hits=curve_curve_intersections(&curve,&cutter,1e-9,&limits).unwrap();assert_eq!(hits.len(),1,"{hits:?}");let f=0.2/3.4;let expected=if reversed{1.-f}else{f};assert!((hits[0].first_fraction-expected).abs()<1e-9);assert!(close(hits[0].point,[1.5,0.25,0.],1e-8));
        record.domain=[1.,3.];assert!(curve_curve_intersections(&Curve::Nurbs(record.clone()),&cutter,1e-9,&limits).unwrap().is_empty());
        record.domain=[0.5,3.];let ends=curve_curve_intersections(&Curve::Nurbs(record),&cutter,1e-9,&limits).unwrap();assert_eq!(ends.len(),1);assert!((ends[0].first_fraction-if reversed{1.}else{0.}).abs()<1e-9);
    }
}

#[test]
fn multispan_quadratic_nurbs_line_tangent_is_one_hit() {
    use om9_core::geometry::NurbsCurve;
    let limits = Limits::default();
    let line = line([0., 0., 0.], [2., 0., 0.]);
    for reversed in [false, true] {
        let curve = Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[0., 1., 0.], [0.5, -1., 0.], [1., 1., 0.], [1.5, 2., 0.], [2., 3., 0.]], weights: vec![1.; 5], knots: vec![0., 0., 0., 0.5, 0.5, 1., 1., 1.], domain: [0., 1.], periodic: false, reversed, seam: None });
        for swap in [false, true] {
            let hits = if swap { curve_curve_intersections(&line, &curve, 1e-10, &limits) } else { curve_curve_intersections(&curve, &line, 1e-10, &limits) }.unwrap();
            assert_eq!(hits.len(), 1, "reversed={reversed} swap={swap}: {hits:?}");
            let f = if swap { hits[0].second_fraction } else { hits[0].first_fraction };
            assert!((f - if reversed {0.75} else {0.25}).abs() < 1e-10); assert!(close(hits[0].point, [0.5, 0., 0.], 1e-10));
        }
    }
}

#[test]
fn multispan_quadratic_keeps_distinct_visits_and_merges_shared_knot() {
    use om9_core::geometry::NurbsCurve;
    let limits = Limits::default(); let cutter = line([0., 0., 0.], [2., 0., 0.]);
    let make = |poles| Curve::Nurbs(NurbsCurve { degree: 2, poles, weights: vec![1.; 5], knots: vec![0., 0., 0., 0.5, 0.5, 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None });
    let shared = make(vec![[0.,1.,0.], [0.5,0.5,0.], [1.,0.,0.], [1.5,0.5,0.], [2.,1.,0.]]);
    let hits = curve_curve_intersections(&shared, &cutter, 1e-10, &limits).unwrap(); assert_eq!(hits.len(), 1, "{hits:?}"); assert!((hits[0].first_fraction-0.5).abs()<1e-12);
    let twice = make(vec![[0.,1.,0.], [0.5,-1.,0.], [1.,1.,0.], [0.5,-1.,0.], [0.,1.,0.]]);
    let hits = curve_curve_intersections(&twice, &cutter, 1e-10, &limits).unwrap(); assert_eq!(hits.len(), 2, "{hits:?}");
    assert!((hits[0].first_fraction-0.25).abs()<1e-12 && (hits[1].first_fraction-0.75).abs()<1e-12);
    let mut limited=limits;limited.max_outputs=1;assert!(curve_curve_intersections(&twice,&cutter,1e-10,&limited).is_err());
    let rational=Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[0.,0.25,0.],[0.5,-0.125,0.],[1.,0.25,0.],[1.5,0.5,0.],[2.,1.,0.]],weights:vec![1.,2.,1.,2.,1.],knots:vec![0.,0.,0.,0.5,0.5,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let hits=curve_curve_intersections(&rational,&cutter,1e-10,&limits).unwrap();assert_eq!(hits.len(),1,"{hits:?}");assert!((hits[0].first_fraction-0.25).abs()<1e-12);
}

#[test]
fn quadratic_nurbs_finds_close_crossings_and_tangency() {
    use om9_core::geometry::NurbsCurve;
    let limits = Limits::default(); let center = 0.501_f64;
    let curve = |center: f64, half_width: f64| {
        let c = center * center - half_width * half_width;
        Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[0., c, 0.], [0.5, c - center, 0.], [1., 1. - 2. * center + c, 0.]], weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None })
    };
    let cutter = line([0., 0., 0.], [1., 0., 0.]);
    for swap in [false, true] {
        let source = curve(center, 0.0004);
        let hits = if swap { curve_curve_intersections(&cutter, &source, 1e-10, &limits) } else { curve_curve_intersections(&source, &cutter, 1e-10, &limits) }.unwrap();
        assert_eq!(hits.len(), 2, "crossings: {hits:?}");
        let mut xs: Vec<_> = hits.iter().map(|h| h.point[0]).collect(); xs.sort_by(f64::total_cmp);
        assert!((xs[0] - (center - 0.0004)).abs() < 1e-7 && (xs[1] - (center + 0.0004)).abs() < 1e-7, "{xs:?}");
        let source = curve(0.5, 0.);
        let hits = if swap { curve_curve_intersections(&cutter, &source, 1e-10, &limits) } else { curve_curve_intersections(&source, &cutter, 1e-10, &limits) }.unwrap();
        assert_eq!(hits.len(), 1, "tangent: {hits:?}"); assert!((hits[0].point[0] - 0.5).abs() < 1e-5);
        let uncertain = curve(center, 0.);
        assert!(curve_curve_intersections(&uncertain, &cutter, 1e-10, &limits).is_err());
    }
    let rational = Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[0., 0.25, 0.], [0.5, -0.125, 0.], [1., 0.25, 0.]], weights: vec![1., 2., 1.], knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: true, seam: None });
    let hits = curve_curve_intersections(&rational, &cutter, 1e-10, &limits).unwrap();
    assert_eq!(hits.len(), 1); assert!((hits[0].first_fraction - 0.5).abs() < 1e-12);
    let mut low = limits; low.max_outputs = 1;
    assert!(curve_curve_intersections(&curve(center, 0.0004), &cutter, 1e-10, &low).is_err());
    let skew = line([0., 0., 0.01], [1., 0., 0.01]);
    assert!(curve_curve_intersections(&curve(0.5, 0.), &skew, 1e-6, &limits).unwrap().is_empty());
    let collinear = Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[2., 0., 0.], [3., 0., 0.], [4., 0., 0.]], weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None });
    assert!(curve_curve_intersections(&collinear, &cutter, 1e-10, &limits).unwrap().is_empty());
}

#[test]
fn nurbs_intersection_at_a_corner_does_not_require_a_tangent() {
    use om9_core::geometry::NurbsCurve;
    let curve = Curve::Nurbs(NurbsCurve { degree: 1, poles: vec![[-1., 1., 0.], [0., 0., 0.], [1., 1., 0.]], weights: vec![1.; 3], knots: vec![0., 0., 0.5, 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None });
    let cutter = line([-2., 0., 0.], [2., 0., 0.]);
    let hits = curve_curve_intersections(&curve, &cutter, 1e-8, &Limits::default()).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}"); assert!(close(hits[0].point, [0., 0., 0.], 1e-8));
}

#[test]
fn nurbs_intersections_find_crossings_inside_narrow_knot_spans() {
    use om9_core::geometry::NurbsCurve;
    let limits = Limits::default();
    let cutter = line([0., -50., 0.], [502., -50., 0.]);
    for reversed in [false, true] {
        let curve = Curve::Nurbs(NurbsCurve {
            degree: 1, poles: vec![[0., 0., 0.], [1., 0., 0.], [500., 0., 0.], [0., -100., 0.], [502., 0., 0.], [503., 0., 0.]],
            weights: vec![1.; 6], knots: vec![0., 0., 0.4, 0.9, 0.90001, 0.90002, 1., 1.],
            domain: [0., 1.], periodic: false, reversed, seam: None,
        });
        for swap in [false, true] {
            let hits = if swap { curve_curve_intersections(&cutter, &curve, 1e-6, &limits) } else { curve_curve_intersections(&curve, &cutter, 1e-6, &limits) }.unwrap();
            assert_eq!(hits.len(), 2, "reversed={reversed} swap={swap}: {hits:?}");
            let mut x: Vec<_> = hits.iter().map(|h| h.point[0]).collect(); x.sort_by(f64::total_cmp);
            assert!((x[0] - 250.).abs() < 1e-5 && (x[1] - 251.).abs() < 1e-5, "{x:?}");
            for hit in hits { let f = if swap { hit.second_fraction } else { hit.first_fraction }; assert!(close(point_at(&curve, f, &limits).unwrap(), hit.point, 1e-5)); }
        }
        let mut low = limits; low.max_outputs = 1;
        assert!(curve_curve_intersections(&curve, &cutter, 1e-6, &low).is_err());
        let mut low = limits; low.max_bytes = 1000;
        assert!(curve_curve_intersections(&curve, &cutter, 1e-6, &low).is_err());
        if !reversed { assert!(om9_core::top11::trim_by_fractions(&curve, &[0.900005, 0.900015], [0., -100., 0.], &limits).is_ok()); }
    }
}

#[test]
fn parallel_line_segments_distinguish_disjoint_touching_and_overlap() {
    let limits = Limits::default(); let a = line([0., 0., 0.], [1., 0., 0.]);
    assert!(curve_curve_intersections(&a, &line([2., 0., 0.], [3., 0., 0.]), 1e-6, &limits).unwrap().is_empty());
    let touch = curve_curve_intersections(&a, &line([2., 0., 0.], [1., 0., 0.]), 1e-6, &limits).unwrap();
    assert_eq!(touch.len(), 1); assert!(close(touch[0].point, [1., 0., 0.], 1e-12));
    assert!(curve_curve_intersections(&a, &line([0.5, 0., 0.], [2., 0., 0.]), 1e-6, &limits).is_err());
    assert!(curve_curve_intersections(&a, &line([2., 0.01, 0.], [3., 0.01, 0.]), 1e-6, &limits).unwrap().is_empty());
}

#[test]
fn polycurve_intersections_visit_short_segments_and_preserve_parameters() {
    let limits = Limits::default();
    let segments: Vec<_> = (0..600).map(|i| {
        let height = |n| if n == 301 { 3.0 } else { 0.0 };
        line([i as f64, height(i), 0.], [(i + 1) as f64, height(i + 1), 0.])
    }).collect();
    let cutter = line([300., 1., 0.], [302., 1., 0.]);
    for reversed in [false, true] {
        let curve = Curve::Polycurve { segments: segments.clone(), reversed };
        for swap in [false, true] {
            let hits = if swap { curve_curve_intersections(&cutter, &curve, 1e-8, &limits) } else { curve_curve_intersections(&curve, &cutter, 1e-8, &limits) }.unwrap();
            assert_eq!(hits.len(), 2, "{hits:?}");
            for hit in hits {
                let fraction = if swap { hit.second_fraction } else { hit.first_fraction };
                assert!(close(point_at(&curve, fraction, &limits).unwrap(), hit.point, 1e-8));
                assert!((hit.point[1] - 1.).abs() < 1e-8);
            }
        }
    }
}

#[test]
fn polycurve_intersections_deduplicate_joints_but_keep_distinct_visits() {
    let limits = Limits::default();
    let poly = Curve::Polycurve { segments: vec![line([-1., 0., 0.], [0., 0., 0.]), line([0., 0., 0.], [1., 0., 0.]), line([1., 0., 0.], [0., 0., 0.]), line([0., 0., 0.], [-1., 0., 0.])], reversed: false };
    let cutter = Curve::Polycurve { segments: vec![line([0., -1., 0.], [0., 0., 0.]), line([0., 0., 0.], [0., 1., 0.])], reversed: true };
    let hits = curve_curve_intersections(&poly, &cutter, 1e-9, &limits).unwrap();
    assert_eq!(hits.len(), 2, "{hits:?}");
    assert!((hits[0].first_fraction - 0.25).abs() < 1e-12);
    assert!((hits[1].first_fraction - 0.75).abs() < 1e-12);
    assert!(hits.iter().all(|h| (h.second_fraction - 0.5).abs() < 1e-12));
    let mut low = limits; low.max_outputs = 1;
    assert!(curve_curve_intersections(&poly, &cutter, 1e-9, &low).is_err());
}

#[test]
fn line_circle_intersections_handle_close_crossings_tangency_and_3d() {
    let limits=Limits::default();let circle=Curve::Circle{center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[0.,1.],angle_domain:Some([0.,std::f64::consts::TAU])};
    let y=0.9999995;let chord=line([-2.,y,0.],[2.,y,0.]);let x=(1.0_f64-y*y).sqrt();
    let hits=curve_curve_intersections(&chord,&circle,1e-8,&limits).unwrap();assert_eq!(hits.len(),2,"close crossings: {hits:?}");
    let mut xs:Vec<_>=hits.iter().map(|h|h.point[0]).collect();xs.sort_by(f64::total_cmp);assert!((xs[0]+x).abs()<1e-8);assert!((xs[1]-x).abs()<1e-8);
    let swapped=curve_curve_intersections(&circle,&chord,1e-8,&limits).unwrap();assert_eq!(swapped.len(),2);
    let tangent=line([-2.,1.,0.],[2.,1.,0.]);assert_eq!(curve_curve_intersections(&tangent,&circle,1e-8,&limits).unwrap().len(),1);
    let miss=line([-2.,1.0001,0.],[2.,1.0001,0.]);assert!(curve_curve_intersections(&miss,&circle,1e-8,&limits).unwrap().is_empty());
    let through=line([1.,0.,-2.],[1.,0.,2.]);let hits=curve_curve_intersections(&through,&circle,1e-8,&limits).unwrap();assert_eq!(hits.len(),1);assert!(hits.iter().all(|h|h.distance<=1e-8&&(h.point[0]-1.).abs()<1e-8));
    let skew=line([-2.,0.,0.01],[2.,0.,0.01]);assert!(curve_curve_intersections(&skew,&circle,0.001,&limits).unwrap().is_empty());
    let oblique=line([0.,0.,-1.],[2.,0.,1.]);let hits=curve_curve_intersections(&oblique,&circle,1e-8,&limits).unwrap();assert_eq!(hits.len(),1);assert!((hits[0].first_fraction-0.5).abs()<1e-10);
    let near=line([-2.,1.0000005,0.],[2.,1.0000005,0.]);let hits=curve_curve_intersections(&near,&circle,1e-6,&limits).unwrap();assert_eq!(hits.len(),1);assert!((hits[0].distance-0.0000005).abs()<1e-10);
    let quarter=Curve::Circle{center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[7.,9.],angle_domain:Some([0.,std::f64::consts::FRAC_PI_2])};
    let across=line([-2.,0.,0.],[2.,0.,0.]);let hits=curve_curve_intersections(&across,&quarter,1e-8,&limits).unwrap();assert_eq!(hits.len(),1);assert!((hits[0].point[0]-1.).abs()<1e-10);
    for r in [1e-200,1e200] {
        let scaled_circle=Curve::Circle{center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:r,domain:[0.,std::f64::consts::TAU],angle_domain:None};
        let secant=Curve::Line{start:[-2.*r,0.,0.],end:[2.*r,0.,0.],domain:[0.,1.]};let hits=curve_curve_intersections(&secant,&scaled_circle,r*1e-10,&limits).unwrap();assert_eq!(hits.len(),2,"scale {r}: {hits:?}");
        let mut normalized:Vec<_>=hits.iter().map(|h|h.point[0]/r).collect();normalized.sort_by(f64::total_cmp);assert!((normalized[0]+1.).abs()<1e-10&&(normalized[1]-1.).abs()<1e-10);
    }
}

#[test]
fn circle_intersections_respect_each_arc_interval() {
    let limits=Limits::default();let pi=std::f64::consts::PI;
    let arc=|center,angles|Curve::Circle{center,x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[7.,12.],angle_domain:Some(angles)};
    let first=arc([0.,0.,0.],[0.,pi]);
    let lower=arc([1.,0.,0.],[pi,2.*pi]);
    let upper=arc([1.,0.,0.],[0.,pi]);
    assert!(curve_curve_intersections(&first,&lower,1e-8,&limits).unwrap().is_empty());
    let hits=curve_curve_intersections(&first,&upper,1e-8,&limits).unwrap();assert_eq!(hits.len(),1,"{hits:?}");
    assert!((hits[0].point[0]-0.5).abs()<1e-10);assert!((hits[0].point[1]-3_f64.sqrt()/2.).abs()<1e-10);
    assert!((hits[0].first_fraction-1./3.).abs()<1e-10);assert!((hits[0].second_fraction-2./3.).abs()<1e-10);
    let twice=arc([0.,0.,0.],[0.,4.*pi]);let hits=curve_curve_intersections(&twice,&upper,1e-8,&limits).unwrap();
    assert_eq!(hits.len(),2,"each traversal has its own parameter: {hits:?}");
    let mut fractions:Vec<_>=hits.iter().map(|h|h.first_fraction).collect();fractions.sort_by(f64::total_cmp);
    assert!((fractions[0]-1./12.).abs()<1e-10);assert!((fractions[1]-7./12.).abs()<1e-10);
    let limited=Limits{max_outputs:1,..Limits::default()};assert!(curve_curve_intersections(&twice,&upper,1e-8,&limited).is_err());
    let wrapped=arc([0.,0.,0.],[5.,8.]);let hits=curve_curve_intersections(&wrapped,&upper,1e-8,&limits).unwrap();
    assert_eq!(hits.len(),1);assert!((hits[0].first_fraction-(2.*pi+pi/3.-5.)/3.).abs()<1e-10);
    let near_end=arc([0.,0.,0.],[0.,pi/3.-0.0001]);
    let near=curve_curve_intersections(&near_end,&upper,0.001,&limits).unwrap();assert_eq!(near.len(),1);assert_eq!(near[0].first_fraction,1.);assert!(near[0].distance<0.001);
    assert!(curve_curve_intersections(&near_end,&upper,0.00001,&limits).unwrap().is_empty());
    let offset=arc([1.,0.,0.0005],[0.,pi]);let hits=curve_curve_intersections(&first,&offset,0.001,&limits).unwrap();
    assert_eq!(hits.len(),1);assert!((hits[0].distance-0.0005).abs()<1e-12);
    let tiny=|center|Curve::Circle{center,x:[1.,0.,0.],y:[0.,1.,0.],radius:1e-200,domain:[0.,2.*pi],angle_domain:None};
    let hits=curve_curve_intersections(&tiny([0.,0.,0.]),&tiny([1e-200,0.,0.]),1e-210,&limits).unwrap();
    assert_eq!(hits.len(),2,"tiny circles need two distinct crossings: {hits:?}");
}

#[test]
fn line_line_and_circle_intersections_are_analytic() {
    let limits = Limits::default();
    let first = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let second = line([5.0, 0.0, 0.0], [5.0, 10.0, 0.0]);
    let hits = curve_curve_intersections(&first, &second, 1e-8, &limits).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(close(hits[0].point, [5.0, 5.0, 0.0], 1e-9));
    assert!((hits[0].first_fraction - 0.5).abs() < 1e-9);
    let circle = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let chord = line([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let hits = curve_curve_intersections(&chord, &circle, 1e-6, &limits).unwrap();
    assert_eq!(hits.len(), 2);
    let mut xs: Vec<f64> = hits.iter().map(|hit| hit.point[0]).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((xs[0] + 5.0).abs() < 1e-6);
    assert!((xs[1] - 5.0).abs() < 1e-6);
}

#[test]
fn separated_line_segments_do_not_report_intersections() {
    let limits = Limits::default();
    let a = line([0., 0., 0.], [10., 0., 0.]);
    let disjoint = line([20., -1., 0.], [20., 1., 0.]);
    let parallel = line([0., 1., 0.], [10., 1., 0.]);
    for cutter in [disjoint, parallel] {
        assert!(
            curve_curve_intersections(&a, &cutter, 0.001, &limits)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn coplanar_circle_circle_intersections_match_the_closed_form() {
    let limits = Limits::default();
    let first = circle_through([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]).unwrap();
    let second = circle_through([2.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
    let hits = curve_curve_intersections(&first, &second, 1e-7, &limits).unwrap();
    assert_eq!(hits.len(), 2);
    for hit in &hits {
        assert!((hit.point[0] - 0.5).abs() < 1e-7);
        assert!((hit.point[1].abs() - 3.0f64.sqrt() / 2.0).abs() < 1e-7);
    }
}

#[test]
fn curve_surface_intersection_finds_a_piercing_line() {
    let limits = Limits::default();
    let surface = plane();
    let pierce = line([2.0, 2.0, -3.0], [2.0, 2.0, 3.0]);
    let hits = curve_surface_intersections(&pierce, &surface, 1e-4, &limits).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(close(hits[0].point, [2.0, 2.0, 0.0], 1e-4));
}

#[test]
fn curve_surface_refines_between_samples_and_rejects_a_near_miss() {
    let limits = Limits::default();
    let surface = plane();
    let crossing = line([2., 2., -3.], [2., 2., 7.]);
    let hits = curve_surface_intersections(&crossing, &surface, 1e-7, &limits).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!((hits[0].curve_fraction - 0.3).abs() < 1e-7, "{hits:?}");
    assert!(close(hits[0].point, [2., 2., 0.], 1e-7));
    let missed = line([11., 2., -3.], [11., 2., 7.]);
    assert!(
        curve_surface_intersections(&missed, &surface, 1e-7, &limits)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn placed_curve_projection_keeps_original_circle_parameter() {
    let limits = Limits::default();
    let circle = Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 1.,
        domain: [0., std::f64::consts::TAU],
        angle_domain: None,
    };
    let placement = om9_core::geometry::Transform::scale([10., 1., 1.]).unwrap();
    let fraction = 0.125;
    let world = placement
        .point(point_at(&circle, fraction, &limits).unwrap())
        .unwrap();
    let normal = [0.1 / 1.01f64.sqrt(), 1. / 1.01f64.sqrt(), 0.];
    let pick = [
        world[0] + 0.0008 * normal[0],
        world[1] + 0.0008 * normal[1],
        0.,
    ];
    let (actual, nearest, distance) = om9_core::curve_surface_ops::closest_point_on_placed_curve(
        &circle, placement, pick, &limits,
    )
    .unwrap();
    assert!((actual - fraction).abs() < 1e-7, "{actual}");
    assert!(close(nearest, world, 1e-6), "{nearest:?}");
    assert!((distance - 0.0008).abs() < 1e-8, "{distance}");
}

#[test]
fn placed_polycurve_projection_checks_segments_missed_by_global_samples() {
    let limits = Limits::default();
    let mut points: Vec<Point3> = (0..=128).map(|i| [i as f64, 0., 0.]).collect();
    points[65] = [0., -100., 0.];
    let segments: Vec<_> = points.windows(2).map(|p| line(p[0], p[1])).collect();
    let placement = om9_core::geometry::Transform::scale([2., 0.5, 1.]).unwrap();
    let pick = placement.point(points[65]).unwrap();
    for reversed in [false, true] {
        let curve = Curve::Polycurve {
            segments: segments.clone(),
            reversed,
        };
        let (fraction, point, measured) =
            om9_core::curve_surface_ops::closest_point_on_placed_curve(
                &curve, placement, pick, &limits,
            )
            .unwrap();
        let expected = if reversed { 63. / 128. } else { 65. / 128. };
        assert!(
            (fraction - expected).abs() < 1e-9,
            "{fraction} vs {expected}"
        );
        assert!(
            close(point, pick, 1e-9) && measured < 1e-9,
            "{point:?} {measured}"
        );
        assert!(close(
            placement
                .point(point_at(&curve, fraction, &limits).unwrap())
                .unwrap(),
            pick,
            1e-9
        ));
    }
}

#[test]
fn nurbs_projection_checks_a_narrow_knot_span() {
    let limits = Limits::default();
    let placement = om9_core::geometry::Transform::scale([2., 0.5, 1.]).unwrap();
    let knot = 0.90001;
    for reversed in [false, true] {
        let curve = Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 1,
            poles: vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [500., 0., 0.],
                [0., -100., 0.],
                [502., 0., 0.],
                [503., 0., 0.],
            ],
            weights: vec![1.; 6],
            knots: vec![0., 0., 0.4, 0.9, knot, 0.90002, 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed,
            seam: None,
        });
        for native in [knot, 0.900005] {
            let expected = if reversed { 1. - native } else { native };
            let pick = placement
                .point(point_at(&curve, expected, &limits).unwrap())
                .unwrap();
            let (fraction, nearest, distance) =
                om9_core::curve_surface_ops::closest_point_on_placed_curve(
                    &curve, placement, pick, &limits,
                )
                .unwrap();
            assert!((fraction - expected).abs() < 1e-10, "{fraction} {expected}");
            assert!(
                close(nearest, pick, 1e-7) && distance < 1e-7,
                "{nearest:?} {distance}"
            );
        }
    }
}

#[test]
fn nurbs_projection_checks_candidate_memory_budget() {
    let limits = Limits {
        max_points: 20,
        ..Limits::default()
    };
    let curve = Curve::Nurbs(om9_core::geometry::NurbsCurve {
        degree: 1,
        poles: vec![[0., 0., 0.], [1., 0., 0.]],
        weights: vec![1.; 2],
        knots: vec![0., 0., 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    assert!(matches!(
        om9_core::curve_surface_ops::closest_point_on_curve(&curve, [0.5, 0., 0.], &limits),
        Err(om9_core::Error::Budget("projection samples"))
    ));
}

#[test]
fn mesh_projection_and_sketch_land_on_the_triangle() {
    let limits = Limits::default();
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [10.0, 10.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![1, 3, 2]],
        normals: Vec::new(),
        colors: Vec::new(),

        corner_uvs: Vec::new(),
    };
    let (point, distance) = closest_point_on_mesh(&mesh, [2.0, 3.0, 4.0], &limits).unwrap();
    assert!(close(point, [2.0, 3.0, 0.0], 1e-9));
    assert!((distance - 4.0).abs() < 1e-9);
    let curve = sketch_on_mesh(
        &mesh,
        &[[1.0, 1.0, 2.0], [4.0, 1.0, 2.0], [4.0, 4.0, 2.0]],
        2.0 + 1e-6,
        &limits,
    )
    .unwrap();
    for step in 0..=8 {
        let on_curve = point_at(&curve, step as f64 / 8.0, &limits).unwrap();
        assert!(on_curve[2].abs() < 1e-9);
    }
}

#[test]
fn two_view_reconstruction_accepts_meeting_rays_and_refuses_skew() {
    let point = point_from_two_views(
        [0.0, 0.0, 5.0],
        [0.0, 0.0, -1.0],
        [5.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        1e-9,
    )
    .unwrap();
    assert!(close(point, [0.0, 0.0, 0.0], 1e-9));
    assert!(
        point_from_two_views(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, -1.0],
            [5.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            1e-6,
        )
        .is_err()
    );
    assert!(
        point_from_two_views([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [1.0, 0.0, 0.0], 1e-6).is_err()
    );
}

#[test]
fn surface_sketch_and_isocurve_use_the_surface_evaluator() {
    let limits = Limits::default();
    let surface = plane();
    let curve =
        sketch_on_surface(&surface, &[[0.1, 0.2], [0.5, 0.5], [0.9, 0.8]], &limits).unwrap();
    let start = point_at(&curve, 0.0, &limits).unwrap();
    assert!(close(start, [1.0, 2.0, 0.0], 1e-9));
    let iso = duplicate_isocurve(&surface, SurfaceIso::U, 0.5, &limits).unwrap();
    let middle = point_at(&iso, 0.5, &limits).unwrap();
    assert!((middle[0] - 5.0).abs() < 1e-9);
    assert!(middle[2].abs() < 1e-9);
}

fn square_brep() -> Brep {
    let face = Uuid::from_u128(1);
    let edges: Vec<Uuid> = (2..6).map(Uuid::from_u128).collect();
    let vertices: Vec<Uuid> = (6..10).map(Uuid::from_u128).collect();
    let display = BrepDisplay {
        source_sha256: [0; 32],
        deflection_mm: 0.01,
        points: vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        ],
        triangles: vec![[0, 1, 2, 0], [0, 2, 3, 0]],
        segments: vec![[0, 1, 0], [1, 2, 1], [2, 3, 2], [3, 0, 3]],
        vertices: vec![0, 1, 2, 3],
        faces: vec![face],
        edges,
        vertex_ids: vertices,
        face_edges: vec![[0, 0], [0, 1], [0, 2], [0, 3]],
    };
    Brep::from_native(
        "CASCADE Topology V3, (c) Open Cascade".to_string(),
        display,
        &Limits::default(),
    )
    .unwrap()
}

#[test]
fn brep_boundary_and_edge_duplication_follow_the_display_snapshot() {
    let limits = Limits::default();
    let brep = square_brep();
    let boundary = boundary_edges(&brep, &limits).unwrap();
    assert_eq!(boundary.len(), 4);
    // Segment 3 is the vertical edge (0,10) -> (0,0); its display chain is ordered from
    // the degree-one endpoint at the origin.
    let edge = brep.display.edges[3];
    let curve = duplicate_brep_edge(&brep, edge, &limits).unwrap();
    let [start, end] = om9_core::curve_ops::curve_endpoints(&curve, &limits).unwrap();
    let length = om9_core::curve_ops::arc_length(&curve, &limits).unwrap();
    assert!((length - 10.0).abs() < 1e-9);
    assert!(close(start, [0.0, 0.0, 0.0], 1e-9));
    assert!(close(end, [0.0, 10.0, 0.0], 1e-9));
}
