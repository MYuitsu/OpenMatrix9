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
    let limits = Limits::default();
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 3,
        poles: vec![
            [4., 0., 0.],
            [0., 4., 0.],
            [-4., 0., 0.],
            [0., -4., 0.],
            [4., 0., 0.],
            [0., 4., 0.],
            [-4., 0., 0.],
        ],
        weights: vec![1., 0.7, 1.3, 0.9, 1., 0.7, 1.3],
        knots: (0..=10).map(|v| v as f64).collect(),
        domain: [3., 7.],
        periodic: true,
        reversed: false,
        seam: None,
    });
    let a = curve_ops::fraction_derivatives(&curve, 0., 2, &limits).unwrap();
    let b = curve_ops::fraction_derivatives(&curve, 1., 2, &limits).unwrap();
    for k in 0..=2 {
        for axis in 0..3 {
            assert!((a[k][axis] - b[k][axis]).abs() < 1e-9, "D{k} {a:?} {b:?}");
        }
    }
    assert!(curve_ops::fraction_derivatives(&curve, 0., 3, &limits).is_err());
    let points = curve_ops::sample_curve(&curve, 4097, &limits).unwrap();
    let area = 0.5
        * points
            .windows(2)
            .map(|p| p[0][0] * p[1][1] - p[1][0] * p[0][1])
            .sum::<f64>()
            .abs();
    for reversed in [false, true] {
        let source = if reversed {
            curve_ops::reverse_curve(&curve, &limits).unwrap()
        } else {
            curve.clone()
        };
        let body = extrusion::solid(&source, [0., 0., 2.], 0.01, &limits).unwrap();
        let m = creation::measure(&body, &limits).unwrap();
        assert!(m.closed && m.solids == 1);
        assert!((m.volume_mm3 - 2. * area).abs() < 1e-4, "{m:?} area {area}");
    }
    let Curve::Nurbs(mut doubled) = curve else {
        unreachable!()
    };
    doubled.knots = vec![0., 1., 2., 2., 4., 5., 6., 6., 8., 9., 10.];
    doubled.domain = [2., 6.];
    let doubled = Curve::Nurbs(doubled);
    let a = curve_ops::fraction_derivatives(&doubled, 0., 1, &limits).unwrap();
    let b = curve_ops::fraction_derivatives(&doubled, 1., 1, &limits).unwrap();
    for k in 0..=1 {
        for axis in 0..3 {
            assert!((a[k][axis] - b[k][axis]).abs() < 1e-9, "D{k} {a:?} {b:?}");
        }
    }
    assert!(curve_ops::fraction_derivatives(&doubled, 0., 2, &limits).is_err());
    assert!(curve_ops::validate_ring_rail(&doubled, 1e-7, &limits).is_ok());
    assert!(extrusion::planar_face(&doubled, 0.01, &limits).is_ok());
    let points = curve_ops::sample_curve(&doubled, 4097, &limits).unwrap();
    let area = 0.5
        * points
            .windows(2)
            .map(|p| p[0][0] * p[1][1] - p[1][0] * p[0][1])
            .sum::<f64>()
            .abs();
    for reversed in [false, true] {
        for height in [-2., 2.] {
            let source = if reversed {
                curve_ops::reverse_curve(&doubled, &limits).unwrap()
            } else {
                doubled.clone()
            };
            let body = extrusion::solid(&source, [0., 0., height], 0.01, &limits).unwrap();
            let m = creation::measure(&body, &limits).unwrap();
            assert!(m.closed && m.solids == 1);
            assert!((m.volume_mm3 - 2. * area).abs() < 1e-4, "{m:?} area {area}");
        }
    }
    for (base, scale) in [(10., 0.01), (10., 1000.), (0., 1e-9), (1e6, 1e-4)] {
        let Curve::Nurbs(mut scaled) = doubled.clone() else {
            unreachable!()
        };
        for knot in &mut scaled.knots {
            *knot = base + *knot * scale;
        }
        scaled.domain = scaled.domain.map(|u| base + u * scale);
        let curve = Curve::Nurbs(scaled);
        let body = extrusion::solid(&curve, [0., 0., 2.], 0.01, &limits).unwrap();
        let m = creation::measure(&body, &limits)
            .unwrap_or_else(|e| panic!("base {base} scale {scale}: {e:?}"));
        assert!(
            (m.volume_mm3 - 2. * area).abs() < 1e-4,
            "base {base} scale {scale} {m:?}"
        );
    }
    let Curve::Nurbs(mut moved) = doubled.clone() else {
        unreachable!()
    };
    for p in &mut moved.poles {
        p[0] += 100.;
        p[1] -= 200.;
        p[2] += 30.;
    }
    let body = extrusion::solid(&Curve::Nurbs(moved), [0., 0., -2.], 0.01, &limits).unwrap();
    let m = creation::measure(&body, &limits).unwrap();
    assert!((m.volume_mm3 - 2. * area).abs() < 1e-4, "translated {m:?}");
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

#[test]
fn tangent_and_kinked_arc_profiles_match_rhino5_wall_topology() {
    use om9_core::geometry::Transform;
    let l = Limits::default();
    let root = [0., 0., 0.];
    let joint = [1., 1., 0.];
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let first = curve_build::arc_through(root, [h, 1. - h, 0.], joint).unwrap();
    for (second_mid, kink) in [
        ([3. - 2. * h, 1. + 2. * h, 0.], false),
        ([1. + 2. * h, 3. - 2. * h, 0.], true),
    ] {
        let second = curve_build::arc_through(joint, second_mid, [3., 3., 0.]).unwrap();
        let curve = Curve::Polycurve {
            segments: vec![first.clone(), second],
            reversed: false,
        };
        for split in [false, true] {
            let prepared = curve_ops::extrude_lines::prism_profile(
                &curve,
                Transform::identity(),
                split,
                1e-6,
                1e-6,
                &l,
            )
            .unwrap();
            let walls = extrusion::side_wall(&prepared, [0., 0., 3.], 0.05, &l).unwrap();
            let two = split || kink;
            assert_eq!(
                walls.topology().faces.len(),
                if two { 2 } else { 1 },
                "kink={kink} split={split}"
            );
            assert_eq!(walls.topology().edges.len(), if two { 7 } else { 4 });
            if !split && !kink {
                let draft = extrusion::tapered(
                    &prepared,
                    [0., 0., 1.],
                    3.,
                    10f64.to_radians(),
                    extrusion::DraftCorners::Sharp,
                    false,
                    0.01,
                    &l,
                )
                .unwrap();
                assert_eq!(draft.topology().faces.len(), 1);
                assert_eq!(draft.topology().edges.len(), 4);
            }
            let to_point =
                extrusion::to_point(&prepared, [1., 2., 6.], false, None, 0.01, &l).unwrap();
            assert_eq!(to_point.topology().faces.len(), if two { 2 } else { 1 });
            let visible: std::collections::BTreeSet<_> =
                to_point.display().segments.iter().map(|s| s[2]).collect();
            assert_eq!(visible.len(), if two { 5 } else { 3 });
        }
    }
}

#[test]
fn extrusion_to_point_matches_rhino5_off_axis_topology_and_volume() {
    let limits = Limits::default();
    let apex = [1., 2., 6.];
    let circle = Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 2.,
        domain: [0., std::f64::consts::TAU],
        angle_domain: None,
    };
    let line = Curve::Line {
        start: [-2., 0., 0.],
        end: [2., 0., 0.],
        domain: [0., 1.],
    };
    let rectangle =
        curve_build::rectangle([-2., -1., 0.], [1., 0., 0.], [0., 1., 0.], 4., 2.).unwrap();
    for (name, profile, capped, faces, edges, volume) in [
        ("triangle", &line, false, 1, 3, 0.),
        ("open cone", &circle, false, 1, 2, 0.),
        ("closed cone", &circle, true, 2, 2, 8. * PI),
        ("pyramid", &rectangle, true, 5, 8, 16.),
    ] {
        let body = extrusion::to_point(profile, apex, capped, None, 0.01, &limits).unwrap();
        let measure = creation::measure(&body, &limits).unwrap();
        assert_eq!(body.topology().faces.len(), faces, "{name}");
        // OCCT also stores a degenerate edge for the singular trim at a conical apex.
        let visible_edges: std::collections::BTreeSet<_> =
            body.display().segments.iter().map(|s| s[2]).collect();
        assert_eq!(visible_edges.len(), edges, "{name}");
        assert_eq!(measure.solids, usize::from(capped), "{name}");
        if capped {
            assert!(
                (measure.volume_mm3 - volume).abs() < 1e-5,
                "{name}: {}",
                measure.volume_mm3
            );
        }
        assert!(
            body.display()
                .points
                .iter()
                .any(|p| p.iter().zip(apex).all(|(v, w)| (v - w).abs() < 1e-7)),
            "{name} missing apex"
        );
    }
    assert!(extrusion::to_point(&line, [0.; 3], false, None, 0.01, &limits).is_err());
    for (profile, solid, faces) in [
        (&circle, false, 1),
        (&circle, true, 2),
        (&rectangle, true, 5),
    ] {
        let flat = extrusion::to_point(profile, [0.; 3], solid, None, 0.01, &limits).unwrap();
        assert_eq!(flat.topology().faces.len(), faces);
        assert!(creation::measure(&flat, &limits).unwrap().volume_mm3.abs() < 1e-10);
    }
    let [a, b, c, d] = [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]];
    let segment = |start, end| Curve::Line {
        start,
        end,
        domain: [0., 1.],
    };
    let stadium = Curve::Polycurve {
        segments: vec![
            segment(a, b),
            curve_build::arc_through(b, [2., 0., 0.], c).unwrap(),
            segment(c, d),
            curve_build::arc_through(d, [-2., 0., 0.], a).unwrap(),
        ],
        reversed: false,
    };
    for split in [false, true] {
        let prepared = curve_ops::extrude_lines::prism_profile(
            &stadium,
            om9_core::geometry::Transform::identity(),
            split,
            1e-6,
            1e-6,
            &limits,
        )
        .unwrap();
        if !split {
            let draft = extrusion::tapered(
                &prepared,
                [0., 0., 1.],
                3.,
                10f64.to_radians(),
                extrusion::DraftCorners::Sharp,
                true,
                0.01,
                &limits,
            )
            .unwrap();
            assert_eq!(draft.topology().faces.len(), 3);
            assert!(
                (creation::measure(&draft, &limits).unwrap().volume_mm3 - 14.1444474578).abs()
                    < 5e-5
            );
        }
        let body = extrusion::to_point(&prepared, apex, true, None, 0.01, &limits).unwrap();
        assert_eq!(body.topology().faces.len(), if split { 5 } else { 2 });
        let visible: std::collections::BTreeSet<_> =
            body.display().segments.iter().map(|s| s[2]).collect();
        assert_eq!(visible.len(), if split { 8 } else { 2 });
        let v = creation::measure(&body, &limits).unwrap().volume_mm3;
        assert!(
            (v - (8. + 2. * PI)).abs() < 1e-5,
            "split={split}, volume={v}"
        );
    }
    let boundary =
        curve_build::rectangle([-8., -8., 3.], [1., 0., 0.], [0., 1., 0.], 16., 16.).unwrap();
    let face = extrusion::planar_face(&boundary, 0.01, &limits)
        .unwrap()
        .into_canonical_and_representation()
        .0;
    for (name, profile, solid, faces, edges) in [
        ("frustum", &circle, true, 3, 3),
        ("trapezoid", &line, false, 1, 4),
    ] {
        let body = extrusion::to_point(profile, apex, solid, Some(&face), 0.01, &limits).unwrap();
        let visible: std::collections::BTreeSet<_> =
            body.display().segments.iter().map(|s| s[2]).collect();
        assert_eq!(body.topology().faces.len(), faces, "{name}");
        assert_eq!(visible.len(), edges, "{name}");
        if solid {
            assert!((creation::measure(&body, &limits).unwrap().volume_mm3 - 7. * PI).abs() < 1e-5);
        }
        assert!(body.display().points.iter().all(|p| p[2] <= 3. + 1e-7));
    }
    assert!(extrusion::to_point(&line, apex, true, None, 0.01, &limits).is_err());
    let offset = Curve::Line {
        start: [0., 1., 0.],
        end: [2., 1., 0.],
        domain: [0., 1.],
    };
    assert!(extrusion::to_point(&offset, [0.; 3], false, None, 0.01, &limits).is_ok());
}

#[test]
fn tapered_profiles_match_signed_rhino5_extents_corners_and_volume() {
    use extrusion::DraftCorners::{Round, Sharp, Smooth};
    let limits = Limits::default();
    let line = Curve::Line {
        start: [-2., 0., 0.],
        end: [2., 0., 0.],
        domain: [0., 1.],
    };
    let circle = curve_build::circle_on_frame([0.; 3], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let rect = curve_build::rectangle([-2., -1., 0.], [1., 0., 0.], [0., 1., 0.], 4., 2.).unwrap();
    let oblique = extrusion::tapered(
        &line,
        [1., 0., 1.],
        3.,
        10f64.to_radians(),
        Sharp,
        true,
        0.01,
        &limits,
    )
    .unwrap();
    let expected = [
        2. + 3. / 2f64.sqrt(),
        3. * 10f64.to_radians().tan(),
        3. / 2f64.sqrt(),
    ];
    assert!(
        oblique
            .display()
            .points
            .iter()
            .any(|p| p.iter().zip(expected).all(|(v, w)| (v - w).abs() < 1e-7))
    );
    for solid in [false, true] {
        for draft in [-10_f64, 10.] {
            for corners in [Sharp, Round, Smooth] {
                let body = extrusion::tapered(
                    &rect,
                    [1., 0., 1.],
                    3.,
                    draft.to_radians(),
                    corners,
                    solid,
                    0.01,
                    &limits,
                )
                .unwrap();
                assert_eq!(body.topology().faces.len(), 4 + usize::from(solid));
                assert_eq!(body.topology().edges.len(), 16);
                assert!(!creation::measure(&body, &limits).unwrap().closed);
                let source = [2., -1., 0.];
                let expected = [
                    source[0] + 3. / 2_f64.sqrt(),
                    source[1] + 3. * draft.to_radians().tan(),
                    3. / 2_f64.sqrt(),
                ];
                assert!(
                    body.display()
                        .points
                        .iter()
                        .any(|p| p.iter().zip(expected).all(|(v, w)| (v - w).abs() < 1e-7))
                );
            }
        }
    }
    if let Curve::Circle {
        center,
        x,
        y,
        radius,
        ..
    } = &circle
    {
        let segments = (0..4)
            .map(|i| Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: *radius,
                domain: [i as f64, (i + 1) as f64],
                angle_domain: Some([
                    i as f64 * std::f64::consts::FRAC_PI_2,
                    (i + 1) as f64 * std::f64::consts::FRAC_PI_2,
                ]),
            })
            .collect();
        let quarters = Curve::Polycurve {
            segments,
            reversed: false,
        };
        let body = extrusion::tapered(
            &quarters,
            [1., 0., 1.],
            3.,
            10_f64.to_radians(),
            Sharp,
            true,
            0.01,
            &limits,
        )
        .unwrap();
        let expected = [3.747274332264327, 0., 2.4953663548549576];
        assert_eq!(body.topology().faces.len(), 5);
        assert_eq!(body.topology().edges.len(), 12);
        assert!(
            body.display()
                .points
                .iter()
                .any(|p| p.iter().zip(expected).all(|(v, w)| (v - w).abs() < 1e-7))
        );
    }
    let triangle =
        curve_build::polyline(&[[0., 0., 0.], [6., 0., 0.], [1., 3., 0.], [0.; 3]]).unwrap();
    let reversed = curve_ops::reverse_curve(&rect, &limits).unwrap();
    for h in [3., -3.] {
        for (profile, degrees, corners, faces, edges, volume) in [
            (&line, 10., Sharp, 1, 4, 0.),
            (&line, -10., Sharp, 1, 4, 0.),
            (&circle, 10., Sharp, 3, 3, 28.6071394323),
            (&circle, -10., Sharp, 3, 3, 48.5492513785),
            (&rect, 10., Sharp, 6, 12, 15.5976263903),
            (&rect, 10., Round, 6, 12, 15.5976263903),
            (&rect, -10., Sharp, 6, 12, 34.6409403068),
            (&rect, -10., Round, 10, 20, 34.4007400329),
            (&triangle, -10., Smooth, 8, 15, 39.8996576003),
            (&reversed, -10., Round, 6, 12, 15.5976263903),
            (&rect, -10., Smooth, 10, 20, 34.4722240889),
            (&rect, 10., Smooth, 6, 12, 15.5976263903),
        ] {
            let angle = f64::to_radians(degrees);
            let result = extrusion::tapered(
                profile,
                [0., 0., 1.],
                h,
                angle,
                corners,
                true,
                0.01,
                &limits,
            )
            .unwrap_or_else(|e| {
                panic!("h={h} draft={degrees} corners={corners:?} faces={faces}: {e}")
            });
            assert_eq!(result.topology().faces.len(), faces);
            let visible: std::collections::BTreeSet<_> =
                result.display().segments.iter().map(|s| s[2]).collect();
            assert_eq!(visible.len(), edges);
            let m = creation::measure(&result, &limits).unwrap();
            if volume != 0. {
                assert!(m.closed && m.solids == 1);
                assert!(
                    (m.volume_mm3 - volume).abs() < 5e-5,
                    "draft={degrees} {corners:?} {} vs {volume}",
                    m.volume_mm3
                );
            } else {
                assert!(result.display().points.iter().any(
                    |p| (p[2] - h).abs() < 1e-6 && (p[1] - h.abs() * angle.tan()).abs() < 1e-6
                ));
            }
        }
    }
}

#[test]
fn tapered_to_trimmed_boundary_keeps_only_the_base_side() {
    use extrusion::{
        DraftCorners::{Round, Sharp, Smooth},
        boundary::BoundaryProjection,
    };
    let l = Limits::default();
    let circle = curve_build::circle_on_frame([0.; 3], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let line = Curve::Line {
        start: [-2., 0., 0.],
        end: [2., 0., 0.],
        domain: [0., 1.],
    };
    let rect = curve_build::rectangle([-2., -1., 0.], [1., 0., 0.], [0., 1., 0.], 4., 2.).unwrap();
    for h in [3., -3.] {
        let target =
            curve_build::rectangle([-8., -8., h], [1., 0., 0.], [0., 1., 0.], 16., 16.).unwrap();
        let face = extrusion::planar_face(&target, 0.01, &l)
            .unwrap()
            .into_canonical_and_representation()
            .0;
        for (profile, angle, corners, solid, faces, volume) in [
            (&line, 10., Sharp, false, 1, 0.),
            (&circle, 10., Sharp, true, 3, 28.60713943),
            (&rect, -10., Round, false, 8, 0.),
            (&rect, -10., Smooth, true, 10, 34.47222409),
        ] {
            let start = curve_ops::curve_endpoints(profile, &l).unwrap()[0];
            let p =
                BoundaryProjection::new(face.clone(), [0., 0., 1.], start, 1e-6, 0.01, &l).unwrap();
            let body = p
                .tapered(
                    profile,
                    [0., 0., 1.],
                    f64::to_radians(angle),
                    corners,
                    solid,
                    &l,
                )
                .unwrap_or_else(|e| panic!("h={h} {corners:?} faces={faces}: {e}"));
            assert_eq!(body.topology().faces.len(), faces);
            let m = creation::measure(&body, &l).unwrap();
            if solid {
                assert!(m.closed);
                assert!((m.volume_mm3 - volume).abs() < 5e-5);
            }
            let z = body
                .display()
                .points
                .iter()
                .map(|p| p[2] * h.signum())
                .fold(0., f64::max);
            assert!((z - h.abs()).abs() < 1e-6);
        }
    }

    let slope = f64::atan(0.25);
    for (x, width) in [(-4., 8.), (-3., 3.)] {
        let target = curve_build::rectangle(
            [x, -4., 3. + 0.25 * x],
            [slope.cos(), 0., slope.sin()],
            [0., 1., 0.],
            width / slope.cos(),
            8.,
        )
        .unwrap();
        let face = extrusion::planar_face(&target, 0.01, &l)
            .unwrap()
            .into_canonical_and_representation()
            .0;
        let projection =
            BoundaryProjection::new(face, [0., 0., 2.], [-2., 0., 0.], 1e-6, 0.01, &l).unwrap();
        let body = projection.tapered(&line, [0., 0., 2.], 10_f64.to_radians(), Sharp, false, &l);
        if width == 3. {
            assert!(
                body.is_err(),
                "a partial target cannot terminate the whole profile"
            );
        } else {
            let body = body.unwrap();
            assert_eq!(body.topology().faces.len(), 1);
            for (x, z) in [(-2., 2.5), (2., 3.5)] {
                assert!(
                    body.display()
                        .points
                        .iter()
                        .any(|p| (p[0] - x).abs() < 1e-6 && (p[2] - z).abs() < 1e-6)
                );
            }
        }
    }
}
