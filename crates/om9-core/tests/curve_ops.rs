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

#[test]
fn reseamed_parameter_trim_preserves_affine_fraction_across_original_seam() {
    use om9_core::{curve_ops::trim_curve_by_parameter, geometry::NurbsCurve};
    let limits=Limits::default();
    for reversed in [false,true] {for nested in [false,true] {for rational in [false,true] {
        let leaf=Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:if rational {vec![1.,0.7,1.2,0.9,1.,0.7]}else{vec![1.;6]},knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:Some(1.24)});
        let source=if nested {Curve::Polycurve{segments:vec![Curve::Polycurve{segments:vec![leaf],reversed:false}],reversed:false}}else{leaf};
        let (from,to)=(0.1,0.9);let result=trim_curve_by_parameter(&source,from,to,&limits).unwrap();
        for f in [0.,0.1,0.25,0.5,0.75,0.9,1.] {
            let expected=point_at(&source,from+(to-from)*f,&limits).unwrap();let actual=point_at(&result,f,&limits).unwrap();
            assert!(close(actual,expected,1e-9),"reverse={reversed} nested={nested} rational={rational} f={f}: {actual:?} != {expected:?}");
        }
    }}}
}

#[test]
fn periodic_nurbs_control_and_edit_grips_preserve_repeated_basis() {
    use om9_core::{curve_ops::move_edit_point,geometry::NurbsCurve};
    let limits=Limits::default();
    for reversed in [false,true] {for rational in [false,true] {
        let n=NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:if rational {vec![1.,0.7,1.2,0.9,1.,0.7]}else{vec![1.;6]},knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:None};
        let curve=Curve::Nurbs(n.clone());let grips=curve_grips(&curve,1e-9,&limits).unwrap();assert_eq!(grips.poles.len(),4);assert_eq!(grips.edit_points.len(),4);
        for index in 0..4 {
            let target=[0.25,-0.5,0.2];let moved=move_grip(&curve,index,target,&limits).unwrap();let Curve::Nurbs(moved)=moved else{panic!("NURBS")};
            assert_eq!(moved.poles[index],target);if index<2 {assert_eq!(moved.poles[index+4],target);}moved.certify_repeated_periodic(&limits).unwrap();
            let target=[grips.edit_points[index][0]+0.2,grips.edit_points[index][1]-0.3,0.4];let edited=move_edit_point(&curve,index,target,&limits).unwrap();let Curve::Nurbs(ref edited_n)=edited else{panic!("NURBS")};
            edited_n.certify_repeated_periodic(&limits).unwrap();assert_eq!(edited_n.weights,n.weights);assert_eq!(edited_n.knots,n.knots);assert_eq!(edited_n.reversed,reversed);
            let actual=curve_grips(&edited,1e-9,&limits).unwrap();for i in 0..4 {assert!(close(actual.edit_points[i],if i==index{target}else{grips.edit_points[i]},1e-8),"{i}");}
            let ends=curve_endpoints(&edited,&limits).unwrap();assert!(close(ends[0],ends[1],1e-10));
        }
        assert!(move_grip(&curve,4,[1.,1.,0.],&limits).is_err());
        let mut invalid=n.clone();invalid.poles[4][0]+=0.01;assert!(curve_grips(&Curve::Nurbs(invalid),1e-9,&limits).is_err());
    }}
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
fn polycurve_trim_boundaries_match_evaluation_with_unequal_segments() {
    let limits = Limits::default();
    let forward = Curve::Polycurve {
        segments: vec![
            line([0., 0., 0.], [1., 0., 0.]),
            line([1., 0., 0.], [11., 0., 0.]),
        ],
        reversed: false,
    };
    let historical = trim_curve(&forward, 0.2, 0.4, &limits).unwrap();
    let ends = curve_endpoints(&historical, &limits).unwrap();
    assert!(close(ends[0], [2.2, 0., 0.], 1e-8) && close(ends[1], [4.4, 0., 0.], 1e-8));
    let Curve::Polycurve { segments, .. } = &forward else {
        unreachable!()
    };
    let flagged = Curve::Polycurve {
        segments: segments.clone(),
        reversed: true,
    };
    for curve in [
        forward.clone(),
        reverse_curve(&forward, &limits).unwrap(),
        flagged,
    ] {
        for (from, to) in [(0.2, 0.4), (0.25, 0.75), (0.5, 0.9), (0.1, 1.0)] {
            let trimmed =
                om9_core::curve_ops::trim_curve_by_parameter(&curve, from, to, &limits).unwrap();
            let actual = curve_endpoints(&trimmed, &limits).unwrap();
            assert!(
                close(actual[0], point_at(&curve, from, &limits).unwrap(), 1e-8),
                "{from} {to}: {actual:?}"
            );
            assert!(
                close(actual[1], point_at(&curve, to, &limits).unwrap(), 1e-8),
                "{from} {to}: {actual:?}"
            );
        }
    }
}

#[test]
fn parameter_trim_preserves_arc_and_line_polycurve_endpoints() {
    let limits = Limits::default();
    let curve = Curve::Polycurve {
        segments: vec![
            Curve::Circle {
                center: [0.; 3],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 1.,
                domain: [0., 1.],
                angle_domain: Some([0., std::f64::consts::FRAC_PI_2]),
            },
            line([0., 1., 0.], [0., 11., 0.]),
        ],
        reversed: false,
    };
    for curve in [curve.clone(), reverse_curve(&curve, &limits).unwrap()] {
        for (from, to) in [(0.1, 0.4), (0.25, 0.75)] {
            let piece =
                om9_core::curve_ops::trim_curve_by_parameter(&curve, from, to, &limits).unwrap();
            let ends = curve_endpoints(&piece, &limits).unwrap();
            assert!(close(
                ends[0],
                point_at(&curve, from, &limits).unwrap(),
                1e-8
            ));
            assert!(close(ends[1], point_at(&curve, to, &limits).unwrap(), 1e-8));
        }
    }
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

#[test]
fn nurbs_edit_point_preserves_rational_representation_and_other_grips() {
    use om9_core::geometry::NurbsCurve;
    use om9_core::curve_ops::move_edit_point;
    let limits = Limits::default();
    for reversed in [false, true] {
        let source = Curve::Nurbs(NurbsCurve {
            degree: 3, poles: vec![[0., 0., 0.], [1., 3., 0.], [4., -1., 0.], [6., 2., 0.], [8., 0., 0.]],
            weights: vec![1., 0.7, 1.4, 0.8, 1.],
            knots: vec![2., 2., 2., 2., 4., 7., 7., 7., 7.], domain: [2., 7.], periodic: false, reversed, seam: None,
        });
        let original = curve_grips(&source, 1e-9, &limits).unwrap();
        assert_eq!(move_edit_point(&source, 2, original.edit_points[2], &limits).unwrap(), source);
        let target = [original.edit_points[2][0] + 1., original.edit_points[2][1] - 2., 3.];
        let moved = move_edit_point(&source, 2, target, &limits).unwrap();
        let (Curve::Nurbs(before), Curve::Nurbs(after)) = (&source, &moved) else { unreachable!() };
        assert_eq!((&after.knots, &after.weights, after.domain, after.reversed, after.periodic, after.seam), (&before.knots, &before.weights, before.domain, before.reversed, before.periodic, before.seam));
        let actual = curve_grips(&moved, 1e-9, &limits).unwrap();
        for i in 0..original.edit_points.len() {
            assert!(close(actual.edit_points[i], if i == 2 { target } else { original.edit_points[i] }, 1e-9), "grip {i}");
        }
    }
}

#[test]
fn closed_nurbs_edit_endpoint_keeps_both_seam_points_together() {
    use om9_core::geometry::NurbsCurve;
    use om9_core::curve_ops::move_edit_point;
    let limits = Limits::default();
    let curve = Curve::Nurbs(NurbsCurve { degree: 2, poles: vec![[0., 0., 0.], [3., 0., 0.], [3., 3., 0.], [0., 0., 0.]], weights: vec![1.; 4], knots: vec![0., 0., 0., 0.5, 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None });
    let moved = move_edit_point(&curve, 0, [-1., 0., 1.], &limits).unwrap();
    let points = curve_grips(&moved, 1e-9, &limits).unwrap();
    assert!(points.closed);
    assert!(close(points.edit_points[0], [-1., 0., 1.], 1e-9));
    assert!(close(*points.edit_points.last().unwrap(), [-1., 0., 1.], 1e-9));
}

#[test]
fn nurbs_edit_rejects_periodic_and_insufficient_budget() {
    use om9_core::curve_ops::move_edit_point;
    use om9_core::geometry::NurbsCurve;
    let limits = Limits::default();
    let mut n = NurbsCurve { degree: 2, poles: vec![[0.,0.,0.], [1.,2.,0.], [3.,0.,0.]], weights: vec![1.;3], knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None };
    let small = Limits { max_bytes: 128, ..limits };
    assert!(move_edit_point(&Curve::Nurbs(n.clone()), 1, [1.,3.,0.], &small).is_err());
    n.periodic = true;
    assert!(move_edit_point(&Curve::Nurbs(n), 1, [1.,3.,0.], &limits).is_err());
}

#[test]
fn circle_and_arc_edit_grips_reach_targets_and_keep_other_end() {
    use om9_core::curve_ops::move_edit_point;
    let limits = Limits::default();
    for angles in [None, Some([0.4, 2.3])] {
        let source = Curve::Circle { center: [1., 2., 3.], x: [1.,0.,0.], y: [0.,1.,0.], radius: 4., domain: if angles.is_none() { [0.,std::f64::consts::TAU] } else { [0.,1.] }, angle_domain: angles };
        let grips = curve_grips(&source, 1e-9, &limits).unwrap();
        assert_eq!(move_edit_point(&source, 0, grips.edit_points[0], &limits).unwrap(), source);
        for index in 0..2 {
            let target = [grips.edit_points[index][0] + 2., grips.edit_points[index][1] - 1., grips.edit_points[index][2] + 3.];
            let moved = move_edit_point(&source, index, target, &limits).unwrap();
            let endpoints = curve_endpoints(&moved, &limits).unwrap();
            assert!(close(endpoints[index], target, 1e-8), "{endpoints:?}");
            if angles.is_none() { assert!(close(endpoints[1-index], target, 1e-8)); }
            else { assert!(close(endpoints[1-index], grips.edit_points[1-index], 1e-8)); }
        }
    }
}

#[test]
fn polycurve_edit_joint_moves_adjacent_segments_without_reversing_traversal() {
    use om9_core::curve_ops::move_edit_point;
    let limits = Limits::default();
    for reversed in [false, true] {
        for index in [1, 2] {
            let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), line([5.,0.,0.], [5.,5.,0.])], reversed };
            let original_ends = curve_endpoints(&curve, &limits).unwrap();
            let target = [6., 1., 2.];
            let edited = move_edit_point(&curve, index, target, &limits).unwrap();
            assert_eq!(curve_endpoints(&edited, &limits).unwrap(), original_ends);
            let grips = curve_grips(&edited, 1e-9, &limits).unwrap();
            assert!(close(grips.edit_points[1], target, 1e-9));
            assert!(close(grips.edit_points[2], target, 1e-9));
            assert!(matches!(edited, Curve::Polycurve { reversed: flag, .. } if flag == reversed));
        }
    }
}

#[test]
fn polycurve_edit_closing_joint_keeps_curve_closed() {
    use om9_core::curve_ops::move_edit_point;
    let limits = Limits::default();
    for reversed in [false, true] {
        for index in [0, 5] {
            let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), line([5.,0.,0.], [5.,5.,0.]), line([5.,5.,0.], [0.,0.,0.])], reversed };
            let target = [-1.,-1.,2.];
            let edited = move_edit_point(&curve, index, target, &limits).unwrap();
            let ends = curve_endpoints(&edited, &limits).unwrap();
            assert!(close(ends[0], target, 1e-9) && close(ends[1], target, 1e-9));
            assert!(is_closed(&edited, 1e-9, &limits).unwrap());
        }
    }
}

#[test]
fn polycurve_edit_joint_between_line_and_arc_remains_connected() {
    use om9_core::curve_ops::move_edit_point;
    let limits = Limits::default();
    let arc = Curve::Circle { center: [5.,2.,0.], x: [1.,0.,0.], y: [0.,1.,0.], radius: 2., domain: [0.,1.], angle_domain: Some([-std::f64::consts::FRAC_PI_2, 0.]) };
    let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), arc], reversed: false };
    let target = [5., -1., 1.];
    let moved = move_edit_point(&curve, 1, target, &limits).unwrap();
    let Curve::Polycurve { segments, .. } = moved else { panic!("lost polycurve") };
    assert!(close(curve_endpoints(&segments[0], &limits).unwrap()[1], target, 1e-9));
    assert!(close(curve_endpoints(&segments[1], &limits).unwrap()[0], target, 1e-9));
    assert!(close(curve_endpoints(&segments[1], &limits).unwrap()[1], [7.,2.,0.], 1e-9));
}

#[test]
fn polycurve_control_joint_keeps_segments_and_orientation() {
    let limits = Limits::default();
    for reversed in [false, true] {
        for index in [1, 2] {
            let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), line([5.,0.,0.], [5.,5.,0.])], reversed };
            let old_ends = curve_endpoints(&curve, &limits).unwrap();
            let target = [6.,1.,2.];
            let edited = move_grip(&curve, index, target, &limits).unwrap();
            assert_eq!(curve_endpoints(&edited, &limits).unwrap(), old_ends);
            let grips = curve_grips(&edited, 1e-9, &limits).unwrap();
            assert!(close(grips.poles[1], target, 1e-9) && close(grips.poles[2], target, 1e-9));
            assert!(matches!(edited, Curve::Polycurve { reversed: flag, .. } if flag == reversed));
        }
    }
}

#[test]
fn polycurve_control_closed_joint_and_line_arc_remain_connected() {
    let limits = Limits::default();
    for reversed in [false, true] {
        let closed = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), line([5.,0.,0.], [5.,5.,0.]), line([5.,5.,0.], [0.,0.,0.])], reversed };
        let target = [-1.,-1.,2.];
        let edited = move_grip(&closed, 0, target, &limits).unwrap();
        let ends = curve_endpoints(&edited, &limits).unwrap();
        assert!(close(ends[0], target, 1e-9) && close(ends[1], target, 1e-9));
    }
    let arc = Curve::Circle { center: [5.,2.,0.], x: [1.,0.,0.], y: [0.,1.,0.], radius: 2., domain: [0.,1.], angle_domain: Some([-std::f64::consts::FRAC_PI_2, 0.]) };
    let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), arc], reversed: false };
    // Moving the line joint converts the arc as needed to move its endpoint.
    let edited = move_grip(&curve, 1, [5.,-1.,1.], &limits).unwrap();
    let Curve::Polycurve { segments, .. } = edited else { panic!("lost polycurve") };
    assert!(close(curve_endpoints(&segments[0], &limits).unwrap()[1], curve_endpoints(&segments[1], &limits).unwrap()[0], 1e-9));
    // Moving the arc center changes its start; the adjacent line follows.
    let edited = move_grip(&curve, 2, [6.,2.,1.], &limits).unwrap();
    let Curve::Polycurve { segments, .. } = edited else { panic!("lost polycurve") };
    assert!(close(curve_endpoints(&segments[0], &limits).unwrap()[1], curve_endpoints(&segments[1], &limits).unwrap()[0], 1e-9));
}

#[test]
fn polycurve_control_joint_moves_rational_nurbs_endpoint() {
    use om9_core::geometry::NurbsCurve;
    let limits = Limits::default();
    let n = NurbsCurve { degree: 2, poles: vec![[5.,0.,0.], [6.,2.,0.], [8.,0.,0.]], weights: vec![1.,0.7,1.], knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None };
    let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), Curve::Nurbs(n.clone())], reversed: false };
    let target = [5.,1.,2.];
    let edited = move_grip(&curve, 1, target, &limits).unwrap();
    let Curve::Polycurve { segments, .. } = edited else { panic!("lost polycurve") };
    let Curve::Nurbs(result) = &segments[1] else { panic!("lost nurbs") };
    assert_eq!(result.poles[0], target);
    assert_eq!(&result.poles[1..], &n.poles[1..]);
    assert_eq!(result.weights, n.weights);
    assert!(close(curve_endpoints(&segments[0], &limits).unwrap()[1], curve_endpoints(&segments[1], &limits).unwrap()[0], 1e-9));
}

#[test]
fn delete_control_point_rejects_periodic_curve() {
    use om9_core::geometry::NurbsCurve;
    use om9_core::curve_ops::delete_control_point;
    let n = NurbsCurve { degree: 2, poles: vec![[0.,0.,0.],[1.,2.,0.],[3.,2.,0.],[4.,0.,0.]], weights: vec![1.;4], knots: vec![0.,0.,0.,0.5,1.,1.,1.], domain: [0.,1.], periodic: true, reversed: false, seam: None };
    assert!(delete_control_point(&Curve::Nurbs(n), 1, &Limits::default()).is_err());
}
