// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact section fixtures for the Extrude Lines ToBoundary command.
#![cfg(feature = "3dm-brep")]
use om9_core::{
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Geometry, Transform},
    surface_kernel::plane,
};
use om9_kernel::{editing, measurement};

#[test]
fn side_wall_meets_a_sloping_or_curved_boundary_along_the_exact_section() {
    let limits = Default::default();
    let identity = Transform::identity();
    let side =
        Geometry::Surface(plane([0.; 3], [1., 0., 0.], [0., 0., 1.], 2., 10., &limits).unwrap());
    let base = plane([-1., -1., 0.], [1., 0., 0.], [0., 1., 0.], 4., 2., &limits).unwrap();
    for curved in [false, true] {
        let mut stop = base.clone();
        if curved {
            // Quadratic Bezier section, z = 2 + (x - 1)^2 across [-1, 3].
            stop.degree[0] = 2;
            stop.size[0] = 3;
            stop.knots[0] = vec![0., 0., 0., 1., 1., 1.];
            stop.poles = [-1., 1., 3.]
                .into_iter()
                .zip([6., -2., 6.])
                .flat_map(|(x, z)| [[x, -1., z], [x, 1., z]])
                .collect();
            stop.weights = vec![1.; 6];
        } else {
            for p in &mut stop.poles {
                p[2] = 2. + p[0];
            }
        }
        if curved {
            // The projected endpoints both hit, even when an inner trim removes
            // the middle of the curve. Endpoint hits alone cannot validate a rail.
            let trim = om9_core::rhino_semantics::TrimmedSurface {
                underlying: stop.clone(),
                outer: vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
                inners: vec![vec![[0.4, 0.4], [0.6, 0.4], [0.6, 0.6], [0.4, 0.6]]],
                reversed: false,
            };
            let source =
                measurement::trimmed_surface_to_face_source_admitted(&trim, &limits, |_, _| Ok(()))
                    .unwrap();
            let face = om9_kernel::brep::read_brep(&source, 0.02, &limits).unwrap();
            let holed = om9_kernel::extrusion::boundary::BoundaryProjection::new(
                face.clone().into_canonical_and_representation().0,
                [0., 0., 1.],
                [0.; 3],
                1e-6,
                0.02,
                &limits,
            )
            .unwrap();
            let curve = om9_core::geometry::Curve::Line {
                start: [0.; 3],
                end: [2., 0., 0.],
                domain: [0., 1.],
            };
            assert!(
                holed.rail(&curve, identity, &limits).is_err(),
                "inner trim must interrupt the rail"
            );
            for x in [0., 1., 2.] {
                let probe = om9_core::geometry::Curve::Line {
                    start: [x, 0., 0.],
                    end: [x, 0., 10.],
                    domain: [0., 1.],
                };
                let hits = om9_kernel::intersect::curve_brep_fractions_including_endpoints(
                    &face, &probe, 1e-7, &limits,
                )
                .unwrap();
                if x == 1. {
                    assert!(hits.is_empty(), "ray in trim hole: {hits:?}");
                } else {
                    assert_eq!(hits.len(), 1);
                    assert!((10. * hits[0] - 3.).abs() < 1e-6);
                }
            }
        }
        // Exercise the same projection and capped prism used by the command.
        let start = [0.; 3];
        let axis = [0., 0., 1.];
        let stop_source = measurement::surface_to_face_source(&stop, identity, &limits).unwrap();
        let face = om9_kernel::brep::read_brep(&stop_source, 0.02, &limits)
            .unwrap()
            .into_canonical_and_representation()
            .0;
        let projection = om9_kernel::extrusion::boundary::BoundaryProjection::new(
            face, axis, start, 1e-6, 0.02, &limits,
        )
        .unwrap();
        let source = om9_core::geometry::Curve::Line {
            start,
            end: [2., 0., 0.],
            domain: [0., 1.],
        };
        let projected = projection.rail(&source, identity, &limits).unwrap();
        let elbow = om9_core::geometry::Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 1,
            poles: vec![[0., 0., 0.], [1., 0.5, 0.], [2., 0., 0.]],
            weights: vec![1., 2., 1.],
            knots: vec![0., 0., 0.5, 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed: true,
            seam: None,
        });
        let elbow_rail = projection.rail(&elbow, identity, &limits).unwrap();
        let pieces =
            om9_core::curve_ops::extrude_lines::profile_segments(&elbow_rail, &limits).unwrap();
        assert_eq!(pieces.len(), 2);
        let corner = om9_core::curve_ops::curve_endpoints(&pieces[0], &limits).unwrap()[1];
        assert!((corner[0] - 1.).abs() < 1e-6 && (corner[1] - 0.5).abs() < 1e-6);
        assert!((corner[2] - if curved { 2. } else { 3. }).abs() < 1e-6);

        let at = CurveEvaluator::capture(&projected, &limits)
            .unwrap()
            .evaluate(Parameter::Fraction(0.5), 0, identity)
            .unwrap()[0];
        assert!(
            (at[2]
                - if curved {
                    2. + (at[0] - 1.).powi(2)
                } else {
                    2. + at[0]
                })
            .abs()
                < 1e-6
        );
        let profile = om9_core::curve_build::polyline(&[
            [0., -0.5, 0.],
            [2., -0.5, 0.],
            [2., 0.5, 0.],
            [0., 0.5, 0.],
            [0., -0.5, 0.],
        ])
        .unwrap();
        let solid = projection.solid(&profile, identity, &limits).unwrap();
        let volume = om9_kernel::brep::source_volume(&solid, &limits).unwrap();
        assert!(
            (volume - if curved { 4. + 2. / 3. } else { 6. }).abs() < 1e-6,
            "{volume}"
        );
        let circle =
            om9_core::curve_build::circle_on_frame([1., 0., 0.], [1., 0., 0.], [0., 1., 0.], 0.5)
                .unwrap();
        let ring = projection.rail(&circle, identity, &limits).unwrap();
        assert!(om9_core::curve_ops::is_closed(&ring, 1e-6, &limits).unwrap());
        for [x, y, z] in om9_core::surface_kernel::sample_curve(&ring, 33, &limits).unwrap() {
            assert!(((x - 1.).hypot(y) - 0.5).abs() < 1e-5);
            assert!(
                (z - if curved {
                    2. + (x - 1.).powi(2)
                } else {
                    2. + x
                })
                .abs()
                    < 1e-5
            );
        }
        let boundary = Geometry::Surface(stop);
        let split = measurement::split_surface_by_geometry(
            &side,
            identity,
            &[(&boundary, identity)],
            0.02,
            &limits,
        )
        .unwrap();
        assert_eq!(split.pieces, 2);
        let topology = split.brep.topology();
        let mut adjacent = vec![0; topology.edges.len()];
        for &[_, edge] in &topology.face_edges {
            adjacent[edge as usize] += 1;
        }
        let shared: Vec<_> = adjacent
            .iter()
            .enumerate()
            .filter_map(|(i, &count)| (count == 2).then_some(i))
            .collect();
        assert_eq!(shared.len(), 1);
        let rail = editing::edge_nurbs(&split.brep, shared[0], &limits).unwrap();
        let curve = CurveEvaluator::capture(&rail, &limits).unwrap();
        let mut x_limits = [f64::INFINITY, f64::NEG_INFINITY];
        for i in 0..=16 {
            let [x, y, z] = curve
                .evaluate(Parameter::Fraction(f64::from(i) / 16.), 0, identity)
                .unwrap()[0];
            let height = if curved {
                2. + (x - 1.).powi(2)
            } else {
                2. + x
            };
            assert!(
                y.abs() < 1e-6 && (z - height).abs() < 1e-6,
                "section at {x}: {y}, {z}"
            );
            x_limits = [x_limits[0].min(x), x_limits[1].max(x)];
        }
        assert!(x_limits[0].abs() < 1e-6 && (x_limits[1] - 2.).abs() < 1e-6);
    }
}

#[test]
fn folded_face_uses_the_first_complete_crossing() {
    use om9_core::geometry::{Curve, NurbsSurface};
    use om9_kernel::extrusion::boundary::BoundaryProjection;
    let limits = Default::default();
    let identity = Transform::identity();
    let stop = NurbsSurface {
        degree: [1, 2],
        size: [2, 3],
        poles: [-1., 3.]
            .into_iter()
            .flat_map(|x| [[x, 3., 0.], [x, -5., 4.], [x, 3., 8.]])
            .collect(),
        weights: vec![1.; 6],
        knots: [vec![0., 0., 1., 1.], vec![0., 0., 0., 1., 1., 1.]],
        domain: [[0., 1.], [0., 1.]],
        periodic: [false, false],
        reversed: false,
    };
    let src = measurement::surface_to_face_source(&stop, identity, &limits).unwrap();
    let face = om9_kernel::brep::read_brep(&src, 0.02, &limits)
        .unwrap()
        .into_canonical_and_representation()
        .0;
    for (z, expected) in [(0., 2.), (8., 6.)] {
        let a = [0., 0., z];
        let projection =
            BoundaryProjection::new(face.clone(), [0., 0., 1.], a, 1e-6, 0.02, &limits).unwrap();
        let line = Curve::Line {
            start: a,
            end: [2., 0., z],
            domain: [0., 1.],
        };
        let rail = projection.rail(&line, identity, &limits).unwrap();
        let ends = om9_core::curve_ops::curve_endpoints(&rail, &limits).unwrap();
        assert!(
            ends.iter().all(|p| (p[2] - expected).abs() < 1e-6),
            "{ends:?}"
        );
    }
}
