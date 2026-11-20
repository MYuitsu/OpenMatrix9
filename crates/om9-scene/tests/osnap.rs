// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L2-02-03`: object snaps and SmartTrack inference.
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Mesh},
    model::{Document, Entity},
    transaction::{TransactionCoordinator, World},
};
use om9_scene::{
    camera::{CPlane, Camera, Viewport},
    osnap::{SmartTrack, SnapKind, SnapSettings, SnapState, candidates},
    scene::Scene,
};

#[test]
fn iterator_selection_matches_slice_selection_for_ties_and_empty_input() {
    use om9_scene::osnap::{SnapCandidate, choose, choose_iter};
    let first = SnapCandidate { kind: SnapKind::End, root: None, world: [1., 0., 0.], screen: [0., 0.],
        distance_px: 0., normal: None, parameter: None, tracking_tangent: None, source: None, second_source: None, surface_source: None, apparent: false };
    let mut second = first.clone();
    second.world = [2., 0., 0.];
    let values = vec![first, second];
    assert_eq!(choose(&values), choose_iter(values.iter()));
    let reversed: Vec<_> = values.iter().cloned().rev().collect();
    assert_eq!(choose(&reversed), choose_iter(reversed.iter()));
    assert_eq!(choose_iter(std::iter::empty()), None);
}

fn camera() -> Camera {
    Camera::orthographic([0., 0., 100.], [0.; 3], [0., 1., 0.], 40.).unwrap()
}

fn viewport() -> Viewport {
    Viewport {
        origin: [0., 0.],
        size: [400., 400.],
    }
}

fn screen(point: [f64; 3]) -> [f64; 2] {
    camera().project(point, viewport()).unwrap().unwrap().screen
}

struct Fixture {
    scene: Scene,
    line: om9_core::identity::ObjectId,
    circle: om9_core::identity::ObjectId,
}

fn fixture() -> Fixture {
    let mut document = Document::with_template_tolerance("osnap");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let circle = Entity::new(
        "circle",
        Geometry::Curve(Curve::Circle {
            center: [5., 5., 0.],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 2.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let circle_id = circle.id;
    document.entities.insert(circle_id, circle);
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene = Scene::capture_in_world(
        &coordinator,
        document_id,
        om9_core::budget::Usage::default(),
    )
    .unwrap();
    Fixture {
        scene,
        line: line_id,
        circle: circle_id,
    }
}

fn settings(radius: f64) -> SnapSettings {
    SnapSettings {
        radius_px: radius,
        ..SnapSettings::default()
    }
}

#[test]
fn source_end_direction_does_not_change_at_the_snap_aperture() {
    use om9_core::geometry::Transform;
    use om9_scene::osnap::{occurrence_source, source_curve_candidates};

    let first = [0., 0., 0.];
    let second = [0.005, 0., 0.];
    let curves = [
        Curve::Line { start: first, end: [-4., 0., 0.], domain: [0., 1.] },
        Curve::Line { start: second, end: [0.005, 4., 0.], domain: [0., 1.] },
    ];
    let fixture = fixture();
    let source = occurrence_source(&fixture.scene, 0).unwrap();
    let settings = SnapSettings { enabled: SnapKind::End.bit(), radius_px: 8., ..Default::default() };
    for (cursor, count) in [(screen(first), 2), (screen([-0.798, 0., 0.]), 1)] {
        let hits = source_curve_candidates(&curves, &[], &source, Transform::identity(), &camera(),
            viewport(), cursor, settings, None, &Limits::default(), CPlane::world_xy(), 0.01,
            || false).unwrap();
        assert_eq!(hits.len(), count);
        assert_eq!(hits[0].world, first);
        assert!(hits.iter().all(|hit| hit.tracking_tangent.is_none()),
            "incident edges with different directions are still a corner at {cursor:?}");
    }
}

#[test]
fn source_end_stays_positional_when_direction_witnesses_are_exhausted() {
    use om9_core::geometry::Transform;
    use om9_scene::osnap::{occurrence_source, source_curve_candidates};

    let selected = Curve::Line { start: [0.; 3], end: [4., 0., 0.], domain: [0., 1.] };
    let outside = Curve::Line { start: [2., 2., 0.], end: [2., 6., 0.], domain: [0., 1.] };
    let curves: Vec<_> = std::iter::once(selected).chain(std::iter::repeat_n(outside, 2048)).collect();
    let fixture = fixture();
    let source = occurrence_source(&fixture.scene, 0).unwrap();
    let settings = SnapSettings { enabled: SnapKind::End.bit(), radius_px: 8., ..Default::default() };
    let hits = source_curve_candidates(&curves, &[], &source, Transform::identity(), &camera(),
        viewport(), screen([0.; 3]), settings, None, &Limits::default(), CPlane::world_xy(), 0.01,
        || false).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].world, [0.; 3]);
    assert_eq!(hits[0].tracking_tangent, None);
}

#[test]
fn nonuniform_arc_midpoint_uses_world_length_and_deferred_source() {
    use om9_core::geometry::Transform;
    use om9_scene::osnap::{candidates_in_plane, candidates_in_plane_deferred, occurrence_source, source_curve_candidates};
    let arc = Curve::Circle { center: [0.; 3], x: [1., 0., 0.], y: [0., 1., 0.], radius: 4.,
        domain: [0., std::f64::consts::FRAC_PI_2], angle_domain: None };
    let mut entity = Entity::new("arc", Geometry::Curve(arc.clone()), None);
    entity.placement = Transform::scale([3., 1., 1.]).unwrap();
    let mut document = Document::with_template_tolerance("ellipse midpoint");
    document.entities.insert(entity.id, entity);
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, id, Default::default()).unwrap();
    let settings = SnapSettings { enabled: SnapKind::Mid.bit(), radius_px: 1000., ..Default::default() };
    let cursor = [200., 200.];
    let plane = CPlane::world_xy();
    let regular = candidates_in_plane(&scene, &camera(), viewport(), cursor, settings, None, plane).unwrap();
    let midpoint = regular.iter().find(|candidate| candidate.kind == SnapKind::Mid).unwrap();
    let angle = midpoint.parameter.unwrap();
    let integral = |end: f64| {
        let n = 10_000;
        let step = end / n as f64;
        (0..n).map(|index| {
            let a = (index as f64 + 0.5) * step;
            (12. * a.sin()).hypot(4. * a.cos()) * step
        }).sum::<f64>()
    };
    let total = integral(std::f64::consts::FRAC_PI_2);
    assert!((integral(angle) - total * 0.5).abs() < 0.002, "midpoint angle {angle}");
    let deferred = candidates_in_plane_deferred(&scene, &camera(), viewport(), cursor, settings, None, plane).unwrap();
    assert!(deferred.iter().all(|candidate| candidate.kind != SnapKind::Mid));
    let source = occurrence_source(&scene, 0).unwrap();
    let worker = source_curve_candidates(&[arc.clone()], &[], &source, scene.objects[0].display_transform().unwrap(), &camera(),
        viewport(), cursor, settings, None, &limits, plane, scene.document.effective_tolerance().unwrap().absolute_mm, || false).unwrap();
    let from_worker = worker.iter().find(|candidate| candidate.kind == SnapKind::Mid).unwrap();
    assert_eq!(from_worker.world, midpoint.world);
    assert_eq!(from_worker.source.as_ref(), midpoint.source.as_ref());

    let perspective = Camera::perspective([12., -10., 20.], [0.; 3], [0., 0., 1.], 1.).unwrap();
    let cursor = perspective.project([5., 2., 0.], viewport()).unwrap().unwrap().screen;
    let near_settings = SnapSettings { enabled: SnapKind::Near.bit(), radius_px: 1000., ..Default::default() };
    let hits = candidates_in_plane(&scene, &perspective, viewport(), cursor, near_settings, None, plane).unwrap();
    let near = hits.iter().find(|candidate| candidate.kind == SnapKind::Near).unwrap();
    let dense_best = (0..=20_000).map(|index| {
        let angle = std::f64::consts::FRAC_PI_2 * index as f64 / 20_000.;
        let projected = perspective.project([12. * angle.cos(), 4. * angle.sin(), 0.], viewport()).unwrap().unwrap();
        (projected.screen[0] - cursor[0]).hypot(projected.screen[1] - cursor[1])
    }).fold(f64::INFINITY, f64::min);
    assert!(near.distance_px <= dense_best + 0.06, "{} versus {dense_best}", near.distance_px);
    let deferred = candidates_in_plane_deferred(&scene, &perspective, viewport(), cursor, near_settings, None, plane).unwrap();
    assert!(deferred.iter().all(|candidate| candidate.kind != SnapKind::Near));
    let worker = source_curve_candidates(&[arc], &[], &source, scene.objects[0].display_transform().unwrap(), &perspective,
        viewport(), cursor, near_settings, None, &limits, plane, scene.document.effective_tolerance().unwrap().absolute_mm, || false).unwrap();
    assert_eq!(worker.iter().find(|candidate| candidate.kind == SnapKind::Near).unwrap().world, near.world);
}

#[test]
fn ellipse_perpendicular_finds_even_multiplicity_contact() {
    use om9_core::geometry::Transform;
    use om9_scene::osnap::{candidates_in_plane, candidates_in_plane_deferred, occurrence_source, source_curve_candidates};
    let arc = Curve::Circle { center: [0.; 3], x: [1., 0., 0.], y: [0., 1., 0.], radius: 4.,
        domain: [0., std::f64::consts::FRAC_PI_2], angle_domain: None };
    let mut entity = Entity::new("ellipse", Geometry::Curve(arc.clone()), None);
    entity.placement = Transform::scale([3., 1., 1.]).unwrap();
    let mut document = Document::with_template_tolerance("ellipse perpendicular");
    document.entities.insert(entity.id, entity);
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, id, Default::default()).unwrap();
    let q = std::f64::consts::FRAC_1_SQRT_2;
    // Center of curvature at pi/4: the normal equation touches zero without crossing it.
    let from = [128. / 12. * q.powi(3), -32. * q.powi(3), 0.];
    let expected = [12. * q, 4. * q, 0.];
    let cursor = screen(expected);
    let settings = SnapSettings { enabled: SnapKind::Perpendicular.bit(), radius_px: 20., ..Default::default() };
    let plane = CPlane::world_xy();
    let hits = candidates_in_plane(&scene, &camera(), viewport(), cursor, settings, Some(from), plane).unwrap();
    let near_expected = |hit: &om9_scene::osnap::SnapCandidate| hit.kind == SnapKind::Perpendicular
        && (0..3).map(|i| (hit.world[i] - expected[i]).powi(2)).sum::<f64>().sqrt() < 0.05;
    assert!(hits.iter().any(near_expected), "{hits:?}");
    let deferred = candidates_in_plane_deferred(&scene, &camera(), viewport(), cursor, settings, Some(from), plane).unwrap();
    assert!(deferred.iter().all(|hit| hit.kind != SnapKind::Perpendicular));
    let source = occurrence_source(&scene, 0).unwrap();
    let worker = source_curve_candidates(&[arc], &[], &source, scene.objects[0].display_transform().unwrap(), &camera(),
        viewport(), cursor, settings, Some(from), &limits, plane, scene.document.effective_tolerance().unwrap().absolute_mm, || false).unwrap();
    assert!(worker.iter().any(near_expected), "{worker:?}");
}

#[test]
fn nurbs_source_modes_resolve_to_the_transformed_parabola() {
    use om9_core::{curve_ops::{derivative_at, point_at}, geometry::{NurbsCurve, Transform}};
    use om9_scene::osnap::{candidates_in_plane_deferred, occurrence_source, source_curve_candidates};
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[-1., 0., 0.], [0., 2., 0.], [1., 0., 0.]],
        weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    });
    let transform = Transform::scale([2., 1., 1.]).unwrap();
    let mut entity = Entity::new("parabola", Geometry::Curve(curve.clone()), None);
    entity.placement = transform;
    let mut document = Document::with_template_tolerance("NURBS source modes");
    document.entities.insert(entity.id, entity);
    let document_id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let source = occurrence_source(&scene, 0).unwrap();
    let plane = CPlane::world_xy();
    let tolerance = scene.document.effective_tolerance().unwrap().absolute_mm;
    let expected = [0., 1., 0.];
    for (kind, from) in [
        (SnapKind::Mid, None), (SnapKind::Near, None), (SnapKind::Quadrant, None),
        (SnapKind::Tangent, Some([2., 1., 0.])), (SnapKind::Perpendicular, Some([0., 3., 0.])),
    ] {
        let settings = SnapSettings { enabled: kind.bit(), radius_px: 20., ..Default::default() };
        let cursor = screen(expected);
        let deferred = candidates_in_plane_deferred(&scene, &camera(), viewport(), cursor, settings, from, plane).unwrap();
        assert!(deferred.iter().all(|hit| hit.kind != kind), "{kind:?}: {deferred:?}");
        let hits = source_curve_candidates(std::slice::from_ref(&curve), &[], &source, transform, &camera(), viewport(),
            cursor, settings, from, &limits, plane, tolerance, || false).unwrap();
        let hit = hits.iter().find(|hit| hit.kind == kind && (0..3).map(|i| (hit.world[i] - expected[i]).powi(2)).sum::<f64>().sqrt() < 0.02)
            .unwrap_or_else(|| panic!("{kind:?}: {hits:?}"));
        let source_point = transform.point(point_at(&curve, hit.parameter.unwrap(), &limits).unwrap()).unwrap();
        assert!((0..3).all(|i| (source_point[i] - hit.world[i]).abs() <= tolerance));
        assert_eq!(hit.source.as_ref(), Some(&source));
        if let Some(from) = from {
            let local_derivative = derivative_at(&curve, hit.parameter.unwrap(), &limits).unwrap();
            let origin = transform.point([0.; 3]).unwrap();
            let end = transform.point(local_derivative).unwrap();
            let tangent = std::array::from_fn::<_, 3, _>(|i| end[i] - origin[i]);
            let delta = std::array::from_fn::<_, 3, _>(|i| hit.world[i] - from[i]);
            let length = tangent.iter().map(|value| value * value).sum::<f64>().sqrt();
            let residual = if kind == SnapKind::Perpendicular {
                (0..3).map(|i| delta[i] * tangent[i]).sum::<f64>().abs() / length
            } else {
                (delta[0] * tangent[1] - delta[1] * tangent[0]).abs() / length
            };
            assert!(residual <= tolerance, "{kind:?} residual {residual}");
        }
    }
    assert!(matches!(source_curve_candidates(std::slice::from_ref(&curve), &[], &source, transform, &camera(),
        viewport(), screen(expected), settings(20.), None, &limits, plane, tolerance, || true),
        Err(om9_core::Error::Canceled)));
}

#[test]
fn rational_nurbs_mid_and_perspective_near_match_sampled_source() {
    use om9_core::{curve_ops::point_at, geometry::{NurbsCurve, Transform}};
    use om9_scene::osnap::{occurrence_source, source_curve_candidates};
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
        weights: vec![1., std::f64::consts::FRAC_1_SQRT_2, 1.], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    });
    let transform = Transform::scale([12., 4., 1.]).unwrap();
    let mut entity = Entity::new("rational arc", Geometry::Curve(curve.clone()), None);
    entity.placement = transform;
    let mut document = Document::with_template_tolerance("rational source modes");
    document.entities.insert(entity.id, entity);
    let document_id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let source = occurrence_source(&scene, 0).unwrap();
    let tolerance = scene.document.effective_tolerance().unwrap().absolute_mm;
    let plane = CPlane::world_xy();
    let camera = Camera::perspective([12., -10., 20.], [0.; 3], [0., 0., 1.], 1.).unwrap();
    let cursor = camera.project([5., 2., 0.], viewport()).unwrap().unwrap().screen;
    let query = |kind: SnapKind| source_curve_candidates(std::slice::from_ref(&curve), &[], &source, transform, &camera,
        viewport(), cursor, SnapSettings { enabled: kind.bit(), radius_px: 1000., ..Default::default() },
        None, &limits, plane, tolerance, || false).unwrap();
    let midpoint = query(SnapKind::Mid).into_iter().find(|hit| hit.kind == SnapKind::Mid).unwrap();
    let near = query(SnapKind::Near).into_iter().find(|hit| hit.kind == SnapKind::Near).unwrap();
    let sample_count = 20_000;
    let mut points = Vec::with_capacity(sample_count + 1);
    let mut near_distance = f64::INFINITY;
    for index in 0..=sample_count {
        let point = transform.point(point_at(&curve, index as f64 / sample_count as f64, &limits).unwrap()).unwrap();
        let projected = camera.project(point, viewport()).unwrap().unwrap().screen;
        near_distance = near_distance.min((projected[0] - cursor[0]).hypot(projected[1] - cursor[1]));
        points.push(point);
    }
    let midpoint_index = (midpoint.parameter.unwrap() * sample_count as f64).round() as usize;
    let length = |slice: &[[f64; 3]]| slice.windows(2).map(|pair|
        (0..3).map(|i| (pair[1][i] - pair[0][i]).powi(2)).sum::<f64>().sqrt()).sum::<f64>();
    assert!((length(&points[..=midpoint_index]) - length(&points[midpoint_index..])).abs() < 0.003);
    assert!(near.distance_px <= near_distance + 0.06, "{} versus {near_distance}", near.distance_px);
    for hit in [&midpoint, &near] {
        let source_point = transform.point(point_at(&curve, hit.parameter.unwrap(), &limits).unwrap()).unwrap();
        assert!((0..3).all(|i| (source_point[i] - hit.world[i]).abs() <= tolerance));
        assert_eq!(hit.source.as_ref(), Some(&source));
    }
}

#[test]
fn transformed_ellipse_intersection_retains_both_source_identities() {
    use om9_core::geometry::Transform;
    use om9_scene::osnap::{occurrence_source, source_curve_intersection_candidates};
    let ellipse = Curve::Circle { center: [0.; 3], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2.,
        domain: [0., std::f64::consts::TAU], angle_domain: None };
    let line = Curve::Line { start: [0., -4., 0.], end: [0., 4., 0.], domain: [0., 8.] };
    let mut first = Entity::new("ellipse", Geometry::Curve(ellipse.clone()), None);
    first.placement = Transform::scale([3., 1., 1.]).unwrap();
    let first_id = first.id;
    let second = Entity::new("line", Geometry::Curve(line.clone()), None);
    let second_id = second.id;
    let mut document = Document::with_template_tolerance("ellipse intersection");
    document.entities.insert(first_id, first);
    document.entities.insert(second_id, second);
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, id, Default::default()).unwrap();
    let first_index = scene.objects.iter().position(|object| object.root == first_id).unwrap();
    let second_index = scene.objects.iter().position(|object| object.root == second_id).unwrap();
    let first_source = occurrence_source(&scene, first_index).unwrap();
    let second_source = occurrence_source(&scene, second_index).unwrap();
    let tolerance = scene.document.effective_tolerance().unwrap().absolute_mm;
    let hits = source_curve_intersection_candidates(&ellipse, scene.objects[first_index].display_transform().unwrap(), &first_source,
        &line, scene.objects[second_index].display_transform().unwrap(), &second_source,
        &camera(), viewport(), screen([0., 2., 0.]), 8., tolerance, &limits, &|| false).unwrap();
    let hit = hits.iter().find(|hit| (hit.world[0].abs() + (hit.world[1] - 2.).abs()) <= tolerance * 2.).unwrap();
    assert_eq!(hit.source.as_ref(), Some(&first_source));
    assert_eq!(hit.second_source.as_ref(), Some(&second_source));
    assert!((hit.parameter.unwrap() - std::f64::consts::FRAC_PI_2).abs() < 0.01);
}

#[test]
fn curved_nurbs_intersection_candidate_binds_both_sources_once() {
    use om9_core::geometry::{NurbsCurve, Transform};
    use om9_scene::osnap::{occurrence_source, source_curve_intersection_candidates};
    let curve = |ys: [f64; 4]| Curve::Nurbs(NurbsCurve {
        degree: 3, poles: vec![[-1., ys[0], 0.], [-1. / 3., ys[1], 0.],
            [1. / 3., ys[2], 0.], [1., ys[3], 0.]],
        weights: vec![1.; 4], knots: vec![0., 0., 0., 0., 1., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    });
    let first = curve([-0.75, -0.25, 1. / 12., 1.25]);
    let second = curve([1.25, 1. / 12., -0.25, -0.75]);
    let transform = Transform::scale([2., 1., 1.]).unwrap();
    let mut document = Document::with_template_tolerance("curved intersection");
    let mut a = Entity::new("a", Geometry::Curve(first.clone()), None);
    let mut b = Entity::new("b", Geometry::Curve(second.clone()), None);
    a.placement = transform; b.placement = transform;
    let (a_id, b_id, document_id) = (a.id, b.id, document.id);
    document.entities.insert(a_id, a); document.entities.insert(b_id, b);
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let a_index = scene.objects.iter().position(|object| object.root == a_id).unwrap();
    let b_index = scene.objects.iter().position(|object| object.root == b_id).unwrap();
    let a_source = occurrence_source(&scene, a_index).unwrap();
    let b_source = occurrence_source(&scene, b_index).unwrap();
    let tolerance = scene.document.effective_tolerance().unwrap().absolute_mm;
    let hits = source_curve_intersection_candidates(&first, transform, &a_source, &second, transform, &b_source,
        &camera(), viewport(), screen([0.; 3]), 8., tolerance, &limits, &|| false).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].source.as_ref(), Some(&a_source));
    assert_eq!(hits[0].second_source.as_ref(), Some(&b_source));
    assert!(hits[0].world.iter().all(|coordinate| coordinate.abs() <= tolerance));
}

#[test]
fn source_vertex_candidates_obey_the_remaining_budget() {
    use om9_core::geometry::Transform;
    use om9_scene::osnap::{occurrence_source, source_point_candidates};
    let fixture = fixture();
    let index = fixture.scene.objects.iter().position(|object| object.root == fixture.line).unwrap();
    let source = occurrence_source(&fixture.scene, index).unwrap();
    let cursor = screen([0., 0., 0.]);
    let query = |points: &[[f64; 3]], max| source_point_candidates(points, SnapKind::End, &source,
        Transform::identity(), &camera(), viewport(), cursor, 2., max, || false);
    let hits = query(&[[0., 0., 0.], [10., 10., 0.]], 1).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].source.as_ref(), Some(&source));
    assert!(matches!(query(&[[0., 0., 0.], [0.01, 0., 0.]], 1),
        Err(om9_core::Error::Budget("source point snap candidates"))));
    assert!(query(&[[10., 10., 0.]], 0).unwrap().is_empty());
    assert!(matches!(source_point_candidates(&[[0., 0., 0.]], SnapKind::End, &source,
        Transform::identity(), &camera(), viewport(), cursor, 2., 1, || true), Err(om9_core::Error::Canceled)));
}

#[test]
fn native_brep_surface_candidates_follow_display_quad_triangulation() {
    use om9_core::geometry::NativeBrep;
    let mut document = Document::with_template_tolerance("native quad");
    let (source, representation) = NativeBrep {
        display: std::sync::Arc::new(Mesh {
            vertices: vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 4.]],
            faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }),
        payload: vec![1].into(), crc: 0,
    }.into_source_and_representation();
    let entity = Entity::new("quad", Geometry::NativeBrep(source.clone()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let limits = Limits::default();
    let document_id = document.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let mut reservation = coordinator.reserve_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, 1024 * 1024, None, Default::default()).unwrap();
    let displays = om9_core::legacy_v4::LegacyDisplays { occt: vec![], native: vec![(source, representation)] };
    coordinator.adopt_legacy_displays(displays, &mut reservation, Default::default()).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    for (x, y, z) in [(-4., 3., 2.8), (-1., 3., 1.6), (3., -1., 0.)] {
        let hits = candidates(&scene, &camera(), viewport(), screen([x, y, 0.]), settings(8.), None).unwrap();
        let hit = hits.iter().find(|hit| hit.kind == SnapKind::Surface && hit.root == Some(id))
            .unwrap_or_else(|| panic!("no candidate at {x}, {y}"));
        assert!((hit.world[2] - z).abs() < 1e-9, "display depth mismatch at {x}, {y}: {:?}", hit.world);
    }
}

#[test]
fn end_mid_center_quadrant_are_ranked_inside_the_screen_radius() {
    let fixture = fixture();
    let end = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([10., 0., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert_eq!(end[0].kind, SnapKind::End);
    assert_eq!(end[0].world, [10., 0., 0.]);
    assert_eq!(end[0].root, Some(fixture.line));

    let mid = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([5., 0., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert_eq!(mid[0].kind, SnapKind::Mid);
    assert_eq!(mid[0].world, [5., 0., 0.]);

    let center = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([5., 5., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert_eq!(center[0].kind, SnapKind::Center);
    assert_eq!(center[0].root, Some(fixture.circle));

    let quadrant = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([7., 5., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert!(quadrant.iter().any(|hit| hit.kind == SnapKind::Quadrant && hit.world == [7., 5., 0.]));
}

#[test]
fn candidates_outside_the_radius_and_zero_radius_return_nothing() {
    let fixture = fixture();
    assert!(
        candidates(
            &fixture.scene,
            &camera(),
            viewport(),
            [390., 10.],
            settings(8.),
            None,
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        candidates(
            &fixture.scene,
            &camera(),
            viewport(),
            [390., 10.],
            settings(0.),
            None,
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn near_clamps_to_the_segment_and_carries_a_valid_parameter() {
    let fixture = fixture();
    let near = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([3., 0.2, 0.]),
        settings(8.),
        Some([-5., 10., 0.]),
    )
    .unwrap();
    let candidate = near.iter().find(|c| c.kind == SnapKind::Near).unwrap();
    assert_eq!(candidate.world, [3., 0., 0.]);
    assert!((candidate.parameter.unwrap() - 0.3).abs() < 1e-12);

    let clamped = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([20., 0., 0.]),
        settings(120.),
        Some([20., 10., 0.]),
    )
    .unwrap();
    let candidate = clamped.iter().find(|c| c.kind == SnapKind::Near).unwrap();
    assert_eq!(candidate.world, [10., 0., 0.]);
    assert_eq!(candidate.parameter, Some(1.0));
}

#[test]
fn two_crossing_segments_produce_an_intersection_but_parallel_segments_do_not() {
    let mut document = Document::with_template_tolerance("crossing");
    for (start, end) in [([0., 0., 0.], [10., 0., 0.]), ([5., -5., 0.], [5., 5., 0.])] {
        let entity = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0., 1.],
            }),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene = Scene::capture_in_world(
        &coordinator,
        document_id,
        om9_core::budget::Usage::default(),
    )
    .unwrap();
    let result = candidates(
        &scene,
        &camera(),
        viewport(),
        screen([5., 0., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert!(
        result
            .iter()
            .any(|c| c.kind == SnapKind::Intersection && c.world == [5., 0., 0.]),
        "{result:?}"
    );
}

#[test]
fn off_axis_perspective_near_recovers_the_two_source_parameters() {
    let mut document = Document::with_template_tolerance("perspective source parameters");
    let sources = [
        ([5., 0., 0.], [13., 0., 4.], [9., 0., 2.]),
        ([6., -4., 8.], [6., 4., 8.], [6., 0., 8.]),
    ];
    let roots = sources.map(|(start, end, _)| {
        let entity = Entity::new("reference", Geometry::Curve(Curve::Line {
            start, end, domain: [0., 1.],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        id
    });
    let document_id = document.id;
    let limits = Limits::default();
    let world = World::new(vec![document], &limits).unwrap();
    let coordinator = TransactionCoordinator::new(world, limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let mut camera = Camera::perspective([0., 0., 20.], [0.; 3], [0., 1., 0.], 1.2).unwrap();
    camera.near_mm = 2.;
    let project = |point| camera.project(point, viewport()).unwrap().unwrap().screen;
    let cursor = project(sources[0].2);
    assert_eq!(cursor, project(sources[1].2));
    let start = project(sources[0].0);
    let end = project(sources[0].1);
    assert!(((cursor[0] - start[0]) / (end[0] - start[0]) - 4. / 9.).abs() < 1e-12);
    assert!(candidates(&scene, &camera, viewport(), cursor, only(SnapKind::Intersection), None)
        .unwrap().is_empty());

    for (near, visible_count) in [(2., 2), (17., 1)] {
        camera.near_mm = near;
        let hits = candidates(&scene, &camera, viewport(), cursor, only(SnapKind::Near), None).unwrap();
        assert_eq!(hits.len(), visible_count, "{hits:?}");
        for (root, (_, _, expected)) in roots.into_iter().zip(sources).take(visible_count) {
            let hit = hits.iter().find(|hit| hit.root == Some(root)).unwrap();
            assert!(hit.world.into_iter().zip(expected).all(|(actual, expected)| (actual - expected).abs() < 1e-12));
            assert!((hit.parameter.unwrap() - 0.5).abs() < 1e-12);
        }
        let settings = SnapSettings { apparent_intersections: true, ..only(SnapKind::Intersection) };
        let hits = candidates(&scene, &camera, viewport(), cursor, settings, None).unwrap();
        if visible_count == 2 {
            let [hit] = hits.as_slice() else { panic!("{hits:?}") };
            assert!(hit.apparent);
            assert_eq!(hit.world, sources[1].2);
            assert_eq!(hit.source.as_ref().unwrap().root, roots[1]);
            assert_eq!(hit.second_source.as_ref().unwrap().root, roots[0]);
        } else {
            assert!(hits.is_empty(), "{hits:?}");
        }
    }
}

#[test]
fn apparent_segment_pairs_respect_document_insertion_order() {
    use om9_core::transaction::{Mutation, TransactionRequest};

    let limits = Limits::default();
    let lines = [
        [[-8., 0., 2.], [8., 0., 2.]],
        [[0., -8., 8.], [0., 8., 8.]],
        [[-8., -8., 2.], [8., 8., 2.]],
    ];
    for (order, expected_z) in [([0, 1, 2], 8.), ([1, 0, 2], 2.)] {
        let document = Document::with_template_tolerance("apparent segment order");
        let id = document.id;
        let world = World::new(vec![document], &limits).unwrap();
        let mut coordinator = TransactionCoordinator::new(world, limits).unwrap();
        let mutations = order.map(|index| {
            let [start, end] = lines[index];
            Mutation::Insert {
                document: id,
                entity: Entity::new("line", Geometry::Curve(Curve::Line {
                    start, end, domain: [0., 1.],
                }), None),
            }
        });
        let prepared = coordinator.prepare(TransactionRequest::new("three lines", Vec::from(mutations))).unwrap();
        coordinator.commit(prepared).unwrap();
        let scene = Scene::capture_in_world(&coordinator, id, Default::default()).unwrap();
        let settings = SnapSettings { apparent_intersections: true, ..only(SnapKind::Intersection) };
        let hits = candidates(&scene, &camera(), viewport(), screen([0., 0., 2.]), settings, None).unwrap();
        let [hit] = hits.as_slice() else { panic!("order {order:?}: {hits:?}") };
        assert!((hit.world[2] - expected_z).abs() < 1e-12, "{hit:?}");
        assert_eq!(hit.apparent, expected_z == 8.);
    }
}

#[test]
fn scene_intersection_parameter_follows_the_original_curve() {
    use om9_core::{curve_ops::point_at, geometry::NurbsCurve};

    let limits = Limits::default();
    let line = |start, end| Curve::Line {
        start,
        end,
        domain: [0., 1.],
    };
    let original = Curve::Polycurve {
        segments: vec![
            line([-8., 10., 8.], [0., 10., 8.]),
            Curve::Polycurve {
                segments: vec![
                    line([0., 10., 8.], [0., 0., 8.]),
                    line([0., 0., 8.], [8., 0., 8.]),
                ],
                reversed: false,
            },
        ],
        reversed: true,
    };
    let rational = Curve::Nurbs(NurbsCurve {
        degree: 1,
        poles: vec![[0., -6., 8.], [0., 0., 8.], [8., 0., 8.]],
        weights: vec![1., 2., 1.],
        knots: vec![0., 0., 0.5, 1., 1.],
        domain: [0.125, 0.875],
        periodic: false,
        reversed: true,
        seam: None,
    });
    let verify = |camera: &Camera, curve: &Curve, other: Curve, expected: [f64; 3], fraction: f64, apparent: bool| {
        let mut document = Document::with_template_tolerance("scene source parameter");
        let mut source = Entity::new("source", Geometry::Curve(curve.clone()), None);
        let mut crossing = Entity::new("crossing", Geometry::Curve(other), None);
        // The synchronous true path uses scene order for the first source.
        if crossing.id < source.id {
            std::mem::swap(&mut crossing.id, &mut source.id);
        }
        let roots = [source.id, crossing.id];
        document.entities.insert(source.id, source);
        document.entities.insert(crossing.id, crossing);
        let id = document.id;
        let world = World::new(vec![document], &limits).unwrap();
        let coordinator = TransactionCoordinator::new(world, limits).unwrap();
        let scene = Scene::capture_in_world(&coordinator, id, Default::default()).unwrap();
        let settings = SnapSettings {
            apparent_intersections: apparent,
            ..only(SnapKind::Intersection)
        };
        let cursor = camera.project(expected, viewport()).unwrap().unwrap().screen;
        let hits = candidates(&scene, camera, viewport(), cursor, settings, None).unwrap();
        let [hit] = hits.as_slice() else {
            panic!("expected one hit, found {}", hits.len())
        };
        assert_eq!(hit.apparent, apparent);
        assert_eq!(hit.source.as_ref().unwrap().root, roots[0]);
        assert_eq!(hit.second_source.as_ref().unwrap().root, roots[1]);
        let returned = hit.parameter.unwrap();
        assert!(
            (returned - fraction).abs() < 1e-10,
            "{returned} != {fraction}"
        );
        let source_world = point_at(curve, returned, &limits).unwrap();
        assert!(
            hit.world
                .into_iter()
                .zip(expected)
                .all(|(a, b)| (a - b).abs() < 1e-10)
        );
        assert!(
            source_world
                .into_iter()
                .zip(hit.world)
                .all(|(a, b)| (a - b).abs() < 1e-10)
        );
    };
    let orthographic = camera();
    for (curve, x, fraction) in [(&original, 2., 0.1875), (&rational, 24. / 13., 0.25)] {
        for apparent in [false, true] {
            let z = if apparent { 2. } else { 8. };
            let other = line([x, -4., z], [x, 4., z]);
            verify(&orthographic, curve, other, [x, 0., 8.], fraction, apparent);
        }
    }
    let perspective = Camera::perspective([0., 0., 100.], [0.; 3], [0., 1., 0.], 1.2).unwrap();
    let x = 24. / 13.;
    let far_x = x * 98. / 92.;
    verify(
        &perspective,
        &rational,
        line([far_x, -4., 2.], [far_x, 4., 2.]),
        [x, 0., 8.],
        0.25,
        true,
    );

    let circle = |center, radius, angle_domain| Curve::Circle {
        center,
        radius,
        angle_domain: Some(angle_domain),
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        domain: [0., 1.],
    };
    let full = [0., std::f64::consts::TAU];
    verify(
        &orthographic,
        &original,
        circle([2., 4., 8.], 4., full),
        [2., 0., 8.],
        0.1875,
        false,
    );
    let quarter = std::f64::consts::FRAC_PI_2;
    let arc = Curve::Polycurve {
        segments: vec![
            line([7., 0., 8.], [5., 0., 8.]),
            circle([0., 0., 8.], 5., [0., quarter]),
        ],
        reversed: true,
    };
    let fraction = (1. - 4_f64.atan2(3.) / quarter) / 2.;
    verify(
        &orthographic,
        &arc,
        circle([6., 0., 8.], 5., full),
        [3., 4., 8.],
        fraction,
        false,
    );
}

#[test]
fn apparent_curves_preserve_rational_parameters_and_arc_membership() {
    use om9_core::geometry::{NurbsCurve, Transform};
    use om9_scene::osnap::{apparent_curve_intersections, occurrence_source};
    use std::f64::consts::PI;

    let nurbs = |poles| Curve::Nurbs(NurbsCurve {
        degree: 2, poles, weights: vec![1., 0.5, 1.],
        knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.],
        periodic: false, reversed: false, seam: None,
    });
    let arc = Curve::Circle {
        center: [6., 4., 2.], x: [1., 0., 0.], y: [0., 1., 0.],
        radius: 4., domain: [0., 1.], angle_domain: Some([PI, 2. * PI]),
    };
    let far = nurbs(vec![[5., 2., 0.], [9., -4., 2.], [13., 2., 4.]]);
    let close = nurbs(vec![[5., -4., 7.], [8., 0., 8.], [5., 4., 9.]]);
    let line = Curve::Line { start: [5., 0., 0.], end: [13., 0., 4.], domain: [0., 1.] };
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("perspective apparent curves");
    let arc_placement = Transform::translation([3., 0., 0.]);
    let roots = [
        (&arc, arc_placement), (&far, Transform::identity()),
        (&close, Transform::identity()), (&line, Transform::identity()),
    ]
        .map(|(curve, placement)| {
            let mut entity = Entity::new("reference", Geometry::Curve(curve.clone()), None);
            entity.placement = placement;
            let id = entity.id;
            document.entities.insert(id, entity);
            id
        });
    let document_id = document.id;
    let world = World::new(vec![document], &limits).unwrap();
    let coordinator = TransactionCoordinator::new(world, limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let source = |id| {
        let index = scene.objects.iter().position(|object| object.root == id).unwrap();
        occurrence_source(&scene, index).unwrap()
    };
    let sources = roots.map(source);
    let mut camera = Camera::perspective([0., 0., 20.], [0.; 3], [0., 1., 0.], 1.2).unwrap();
    camera.near_mm = 2.;
    let cursor = camera.project([6., 0., 8.], viewport()).unwrap().unwrap().screen;
    assert_eq!(cursor, camera.project([9., 0., 2.], viewport()).unwrap().unwrap().screen);

    let query = |curve, placement, reference, camera: &Camera, offset| {
        apparent_curve_intersections(
            (curve, placement, reference),
            (&close, Transform::identity(), &sources[2]),
            camera, viewport(), [cursor[0] + offset, cursor[1]], 8., 1e-5, &limits, &|| false,
        ).unwrap()
    };
    for (index, curve, placement, expected_parameter) in [
        (0, &arc, arc_placement, 1.5 * PI), (1, &far, Transform::identity(), 0.5),
        (3, &line, Transform::identity(), 0.5),
    ] {
        for (offset, root, other, expected_point, fraction) in [
            (0., roots[2], roots[index], [6., 0., 8.], 0.5),
            (2., roots[index], roots[2], [9., 0., 2.], expected_parameter),
        ] {
            let hits = query(curve, placement, &sources[index], &camera, offset);
            let hit = om9_scene::osnap::choose_ref(hits.iter()).unwrap();
            assert!(hit.apparent);
            assert_eq!(hit.source.as_ref().unwrap().root, root, "{hit:?}");
            assert_eq!(hit.second_source.as_ref().unwrap().root, other);
            assert!(hit.world.into_iter().zip(expected_point)
                .all(|(actual, expected)| (actual - expected).abs() < 1e-6), "{hit:?}");
            assert!((hit.parameter.unwrap() - fraction).abs() < 1e-6, "{hit:?}");
        }
    }
    camera.far_mm = 19.;
    assert!(camera.project([5., 2., 0.], viewport()).unwrap().is_none());
    let hits = query(&far, Transform::identity(), &sources[1], &camera, 2.);
    let hit = om9_scene::osnap::choose_ref(hits.iter()).unwrap();
    assert_eq!(hit.source.as_ref().unwrap().root, roots[1]);
    assert!((hit.parameter.unwrap() - 0.5).abs() < 1e-6);
    let hits = query(&line, Transform::identity(), &sources[3], &camera, 2.);
    let hit = om9_scene::osnap::choose_ref(hits.iter()).unwrap();
    assert_eq!(hit.source.as_ref().unwrap().root, roots[3]);
    assert!((hit.parameter.unwrap() - 0.5).abs() < 1e-6);

    camera.near_mm = 15.;
    assert!(query(&arc, arc_placement, &sources[0], &camera, 0.).is_empty());
    assert!(query(&far, Transform::identity(), &sources[1], &camera, 0.).is_empty());
}

#[test]
fn apparent_intersection_survives_a_nurbs_control_point_behind_the_eye() {
    use om9_core::{
        curve_ops::point_at,
        geometry::{NurbsCurve, Transform},
    };
    use om9_scene::osnap::{apparent_curve_intersections, occurrence_source};

    // The first pick is exactly at a split boundary. The reversed pick is
    // inside a piece and tests the map back to the original traversal.
    for (reversed, fraction) in [(false, 0.5), (true, 0.375)] {
        let limits = Limits::default();
        let curve = Curve::Nurbs(NurbsCurve {
            degree: 2,
            poles: vec![[5., -4., 7.], [8., 0., 8.], [5., 4., 22.]],
            weights: vec![1., 0.5, 1.],
            knots: vec![0., 0., 0., 1., 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed,
            seam: None,
        });
        let expected = point_at(&curve, fraction, &limits).unwrap();
        let far_scale = 18. / (20. - expected[2]);
        let [far_x, far_y, _] = expected.map(|axis| axis * far_scale);
        let curves = [
            curve,
            Curve::Line {
                start: [far_x - 3., far_y, 2.],
                end: [far_x + 3., far_y, 2.],
                domain: [0., 1.],
            },
        ];
        let mut document = Document::with_template_tolerance("partly visible NURBS");
        let roots = curves.each_ref().map(|curve| {
            let entity = Entity::new("reference", Geometry::Curve(curve.clone()), None);
            let id = entity.id;
            document.entities.insert(id, entity);
            id
        });
        let id = document.id;
        let world = World::new(vec![document], &limits).unwrap();
        let coordinator = TransactionCoordinator::new(world, limits).unwrap();
        let scene = Scene::capture_in_world(&coordinator, id, Default::default()).unwrap();
        let sources = roots.map(|root| {
            let index = scene
                .objects
                .iter()
                .position(|object| object.root == root)
                .unwrap();
            occurrence_source(&scene, index).unwrap()
        });
        let mut camera = Camera::perspective([0., 0., 20.], [0.; 3], [0., 1., 0.], 1.2).unwrap();
        camera.near_mm = 2.;
        let cursor = camera
            .project(expected, viewport())
            .unwrap()
            .unwrap()
            .screen;
        let query = |curve: &Curve, camera: &Camera| {
            apparent_curve_intersections(
                (curve, Transform::identity(), &sources[0]),
                (&curves[1], Transform::identity(), &sources[1]),
                camera,
                viewport(),
                cursor,
                8.,
                1e-5,
                &limits,
                &|| false,
            )
            .unwrap()
        };
        let hit = query(&curves[0], &camera);
        let [hit] = hit.as_slice() else {
            let points = hit
                .iter()
                .map(|hit| (hit.world, hit.parameter))
                .collect::<Vec<_>>();
            panic!("expected one visible crossing: {points:?}");
        };
        assert!(hit.apparent);
        assert_eq!(hit.source.as_ref().unwrap().root, roots[0]);
        assert_eq!(hit.second_source.as_ref().unwrap().root, roots[1]);
        assert!(
            hit.world
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| (actual - expected).abs() < 1e-6),
            "{hit:?}"
        );
        assert!((hit.parameter.unwrap() - fraction).abs() < 1e-6, "{hit:?}");

        camera.near_mm = 8.;
        assert!(query(&curves[0], &camera).is_empty());
    }
}

#[test]
fn collinear_placed_line_touch_is_an_isolated_intersection() {
    use om9_core::geometry::Transform;

    let mut document = Document::with_template_tolerance("collinear endpoint contact");
    let first = Entity::new("first", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [5., 0., 0.], domain: [0., 1.],
    }), None);
    let mut second = Entity::new("second", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [4., 0., 0.], domain: [0., 1.],
    }), None);
    second.placement = Transform::translation([5., 0., 0.]);
    let roots = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let document_id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let cursor = screen([5., 0., 0.]);
    let mut hits = candidates(&scene, &camera(), viewport(), cursor, only(SnapKind::Intersection), None).unwrap();
    assert_eq!(hits.len(), 1);
    let hit = hits.pop().unwrap();
    assert_eq!(hit.world, [5., 0., 0.]);
    let first_source = hit.source.unwrap();
    let second_source = hit.second_source.unwrap();
    assert_ne!(first_source.root, second_source.root);
    assert!(roots.contains(&first_source.root) && roots.contains(&second_source.root));
}

#[test]
fn shared_leaf_occurrences_in_one_block_produce_an_intersection() {
    use om9_core::{geometry::Transform, model::Definition};

    let mut document = Document::with_template_tolerance("block member intersection");
    let member = Entity::new("source line", Geometry::Curve(Curve::Line {
        start: [-4., 0., 0.], end: [4., 0., 0.], domain: [0., 1.],
    }), None);
    let leaf_id = member.id;
    let line_definition = Definition::new("shared line", vec![member]);
    let horizontal = Entity::new("horizontal", Geometry::Instance(line_definition.id), None);
    let mut vertical = Entity::new("vertical", Geometry::Instance(line_definition.id), None);
    vertical.placement = Transform([
        [0., -1., 0., 0.],
        [1., 0., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let paths = [vec![horizontal.id, leaf_id], vec![vertical.id, leaf_id]];
    let block = Definition::new("cross", vec![horizontal, vertical]);
    let mut instance = Entity::new("instance", Geometry::Instance(block.id), None);
    let intersection = [6., 2., 0.];
    instance.placement = Transform::translation(intersection)
        .compose(Transform::scale([2., 3., 1.]).unwrap()).unwrap();
    let root_id = instance.id;
    for definition in [line_definition, block] {
        document.definitions.insert(definition.id, definition);
    }
    document.entities.insert(root_id, instance);
    let document_id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();

    let hits = candidates(&scene, &camera(), viewport(), screen(intersection), only(SnapKind::Intersection), None).unwrap();
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];
    assert_eq!(hit.world, intersection);
    let a = hit.source.as_ref().unwrap();
    let b = hit.second_source.as_ref().unwrap();
    for source in [a, b] {
        source.validate(&scene).unwrap();
        assert_eq!(source.root, root_id);
        assert_eq!(source.leaf, leaf_id);
        assert!(paths.contains(&source.path));
    }
    assert_ne!(a.path, b.path);
}

#[test]
fn tangent_and_perpendicular_are_only_offered_with_a_real_solution() {
    let fixture = fixture();
    // External point (15,5): the upper tangent point is (5.4, 6.96...).
    let tangent_cursor = screen([5.4, 5. + 1.9595917942265426, 0.]);
    let result = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        tangent_cursor,
        settings(10.),
        Some([15., 5., 0.]),
    )
    .unwrap();
    assert!(result.iter().any(|c| c.kind == SnapKind::Tangent));
    let perpendicular = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([7., 5., 0.]),
        settings(10.),
        Some([15., 5., 0.]),
    )
    .unwrap();
    assert!(
        perpendicular
            .iter()
            .any(|c| c.kind == SnapKind::Perpendicular)
    );

    // A point at the centre has no tangent and no distinct perpendicular solution.
    let inside = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([5., 5., 0.]),
        settings(40.),
        Some([5., 5., 0.]),
    )
    .unwrap();
    assert!(!inside.iter().any(|c| c.kind == SnapKind::Tangent));
}

#[test]
fn one_shot_override_wins_one_pick_and_then_resets() {
    let fixture = fixture();
    let mut state = SnapState::default();
    state.settings.radius_px = 60.;
    let cursor = screen([5., 0., 0.]);
    let persistent = state
        .best(&fixture.scene, &camera(), viewport(), cursor, None)
        .unwrap()
        .unwrap();
    assert_eq!(persistent.kind, SnapKind::End);

    state.settings.one_shot = Some(SnapKind::Mid);
    let override_pick = state
        .best(&fixture.scene, &camera(), viewport(), cursor, None)
        .unwrap()
        .unwrap();
    assert_eq!(override_pick.kind, SnapKind::Mid);
    state.accept();
    let next = state
        .best(&fixture.scene, &camera(), viewport(), cursor, None)
        .unwrap()
        .unwrap();
    assert_eq!(next.kind, SnapKind::End);

    state.settings.one_shot = Some(SnapKind::Mid);
    state.cancel();
    assert_eq!(
        state
            .best(&fixture.scene, &camera(), viewport(), cursor, None)
            .unwrap()
            .unwrap()
            .kind,
        SnapKind::End,
        "cancel consumes the one-shot override"
    );
}

#[test]
fn disabled_kinds_are_not_offered_and_marker_text_is_exact() {
    let fixture = fixture();
    let mut state = SnapState::default();
    state.settings.radius_px = 60.;
    state.settings.set(SnapKind::End, false);
    let best = state
        .best(
            &fixture.scene,
            &camera(),
            viewport(),
            screen([5., 0., 0.]),
            None,
        )
        .unwrap()
        .unwrap();
    assert_eq!(best.kind, SnapKind::Mid);
    assert_eq!(best.marker_label(), "Mid");
    assert_eq!(best.tooltip(), "Midpoint");
}

#[test]
fn rational_curve_end_direction_is_evaluated_in_world_space() {
    use om9_core::geometry::{NurbsCurve, Transform};

    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("rational End direction");
    let mut entity = Entity::new("rational curve", Geometry::Curve(Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[0., 0., 0.], [1., 2., 0.], [4., 2., 0.]],
        weights: vec![1., 2., 1.], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    })), None);
    entity.placement = Transform([
        [0., -1., 0., 6.], [3., 0., 0., 8.], [0., 0., 1., 12.], [0., 0., 0., 1.],
    ]);
    let id = entity.id;
    document.entities.insert(id, entity);
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, document_id, Default::default()).unwrap();
    let settings = SnapSettings { one_shot: Some(SnapKind::End), ..settings(4.) };
    let at = [6., 8., 12.];
    let hits = candidates(&scene, &camera(), viewport(), screen(at), settings, None).unwrap();
    let hit = hits.iter().find(|hit| hit.root == Some(id)).expect("rational source End");
    assert_eq!(hit.world, at);
    let tangent = hit.tracking_tangent.expect("source derivative");
    // The first local pole difference is (1,2,0). Placement maps it to (-2,3,0).
    let scale = 13_f64.sqrt();
    assert!(tangent.iter().zip([-2. / scale, 3. / scale, 0.]).all(|(a, b)| (a - b).abs() < 1e-9));
}

#[test]
fn smarttrack_guides_are_transient_and_infer_an_ephemeral_point() {
    let fixture = fixture();
    let first = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([10., 0., 0.]),
        settings(8.),
        None,
    )
    .unwrap()
    .remove(0);
    let second = candidates(
        &fixture.scene,
        &camera(),
        viewport(),
        screen([0., 0., 0.]),
        settings(8.),
        None,
    )
    .unwrap()
    .remove(0);
    let mut track = SmartTrack::new();
    track.push(&first, CPlane::world_xy()).unwrap();
    track.push(&second, CPlane::world_xy()).unwrap();
    assert_eq!(track.anchors().len(), 2);
    assert_eq!(track.guides().len(), 4);
    let inferred = track
        .infer([10., 0., 0.], 1e-6)
        .expect("guide intersection");
    assert_eq!(inferred.kind, SnapKind::SmartTrack);
    assert_eq!(inferred.world, [10., 0., 0.]);
    track.clear();
    assert!(track.guides().is_empty());
    assert!(track.anchors().is_empty());
}

#[test]
fn smarttrack_two_anchors_follow_a_rotated_cplane() {
    let q = std::f64::consts::FRAC_1_SQRT_2;
    let plane = CPlane::new([2., -1., 0.], [q, q, 0.], [-q, q, 0.]).unwrap();
    let mut track = SmartTrack::new();
    for world in [[0., 0., 0.], [4., 0., 0.]] {
        let anchor = om9_scene::osnap::SnapCandidate { kind: SnapKind::Point, root: None, world,
            screen: [0.; 2], distance_px: 0., normal: None, parameter: None, tracking_tangent: None,
            source: None, second_source: None, surface_source: None, apparent: false };
        track.acquire(&anchor, plane, 1e-6).unwrap();
    }
    let hit = track.infer_with_source_tolerance([2.04, 2.06, 0.], 0.1, 1e-6).unwrap();
    assert_eq!(hit.kind, SnapKind::SmartTrack);
    assert!((hit.world[0] - 2.).abs() < 1e-7 && (hit.world[1] - 2.).abs() < 1e-7);
    assert!(hit.source.is_none() && hit.root.is_none());

    let projection = track.infer_with_source_tolerance([3000.04, 2999.96, 0.], 0.1, 1e-6).unwrap();
    assert!(projection.world.iter().zip([3000., 3000., 0.])
        .all(|(actual, expected)| (actual - expected).abs() < 1e-7));
}

#[test]
fn surface_snap_uses_the_tessellated_trimmed_face() {
    // A single planar quad is represented as two triangles; the ray hits the interior.
    let mut document = Document::with_template_tolerance("surface");
    let entity = Entity::new(
        "quad",
        Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
            normals: Vec::new(),
            colors: Vec::new(),

            corner_uvs: Vec::new(),
        }),
        None,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene = Scene::capture_in_world(
        &coordinator,
        document_id,
        om9_core::budget::Usage::default(),
    )
    .unwrap();
    let result = candidates(
        &scene,
        &camera(),
        viewport(),
        screen([3., 4., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert!(
        result
            .iter()
            .all(|candidate| candidate.kind != SnapKind::Surface),
        "plain meshes are not a BRep surface snap"
    );
}

#[test]
fn a_point_entity_is_a_point_snap() {
    let mut document = Document::with_template_tolerance("point snap");
    let entity = Entity::new("point", Geometry::Point([2., 3., 0.]), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene = Scene::capture_in_world(
        &coordinator,
        document_id,
        om9_core::budget::Usage::default(),
    )
    .unwrap();
    let result = candidates(
        &scene,
        &camera(),
        viewport(),
        screen([2., 3., 0.]),
        settings(8.),
        None,
    )
    .unwrap();
    assert_eq!(result[0].kind, SnapKind::Point);
    assert_eq!(result[0].world, [2., 3., 0.]);
}

#[test]
fn parallel_segments_never_produce_a_fake_intersection() {
    let mut document = Document::with_template_tolerance("parallel");
    for (start, end) in [([0., 0., 0.], [10., 0., 0.]), ([0., 1., 0.], [10., 1., 0.])] {
        let entity = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0., 1.],
            }),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene = Scene::capture_in_world(
        &coordinator,
        document_id,
        om9_core::budget::Usage::default(),
    )
    .unwrap();
    let result = candidates(
        &scene,
        &camera(),
        viewport(),
        screen([5., 0.5, 0.]),
        settings(20.),
        None,
    )
    .unwrap();
    assert!(!result.iter().any(|c| c.kind == SnapKind::Intersection));
}

fn nurbs_scene(curves: Vec<om9_core::geometry::NurbsCurve>) -> (Scene, Vec<om9_core::identity::ObjectId>) {
    let mut document = Document::with_template_tolerance("nurbs osnap");
    let mut ids = Vec::new();
    for curve in curves {
        let entity = Entity::new("nurbs", Geometry::Curve(Curve::Nurbs(curve)), None);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene =
        Scene::capture_in_world(&coordinator, document_id, om9_core::budget::Usage::default()).unwrap();
    (scene, ids)
}

fn nurbs(degree: usize, poles: Vec<[f64; 3]>) -> om9_core::geometry::NurbsCurve {
    let n = poles.len();
    let inner = n - degree - 1;
    let mut knots = vec![0.; degree + 1];
    knots.extend((1..=inner).map(|i| i as f64));
    knots.extend(vec![(inner + 1) as f64; degree + 1]);
    om9_core::geometry::NurbsCurve {
        degree,
        weights: vec![1.; n],
        poles,
        domain: [0., (inner + 1) as f64],
        knots,
        periodic: false,
        reversed: false,
        seam: None,
    }
}

fn only(kind: SnapKind) -> SnapSettings {
    SnapSettings { enabled: kind.bit(), ..Default::default() }
}

/// Curves made by the Curve commands are NURBS: their ends, polyline vertices, middles, Near
/// and Perpendicular snap like a line's do.
#[test]
fn nurbs_curves_snap_at_ends_vertices_middles_and_along() {
    let polyline = nurbs(1, vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.]]);
    let arc = nurbs(2, vec![[-15., 0., 0.], [-15., 10., 0.], [-5., 10., 0.]]);
    let (scene, ids) = nurbs_scene(vec![polyline, arc]);
    let at = |kind, point: [f64; 3], offset: [f64; 2]| {
        let s = screen(point);
        candidates(&scene, &camera(), viewport(), [s[0] + offset[0], s[1] + offset[1]], only(kind), None).unwrap()
    };
    // End at a polyline vertex (Rhino: every vertex) and at the curve's ends.
    for p in [[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [-15., 0., 0.], [-5., 10., 0.]] {
        let found = at(SnapKind::End, p, [2., -2.]);
        assert!(found.iter().any(|c| c.world == p), "End {p:?}: {found:?}");
    }
    // Mid of each polyline segment.
    for p in [[5., 0., 0.], [10., 5., 0.]] {
        let found = at(SnapKind::Mid, p, [1., 1.]);
        assert!(found.iter().any(|c| (0..3).all(|i| (c.world[i] - p[i]).abs() < 1e-9)), "Mid {p:?}");
    }
    // Mid of the quadratic by length: on the curve, half way along.
    let mid = at(SnapKind::Mid, [-12.5, 7.5, 0.], [0., 0.]);
    let m = mid.iter().find(|c| c.root == Some(ids[1])).expect("arc middle").world;
    assert!((m[0] + 12.5).abs() < 0.3 && (m[1] - 7.5).abs() < 0.3, "{m:?}");
    // Near: on the polyline under the pointer.
    let near = at(SnapKind::Near, [3., 0., 0.], [0., 2.]);
    let n = near.iter().find(|c| c.root == Some(ids[0])).expect("near").world;
    assert!((n[0] - 3.).abs() < 0.2 && n[1].abs() < 1e-9, "{n:?}");
    // Perpendicular from a point to the second segment.
    let s = screen([10., 4., 0.]);
    let perp = candidates(&scene, &camera(), viewport(), s, only(SnapKind::Perpendicular), Some([20., 4., 0.])).unwrap();
    assert!(perp.iter().any(|c| (c.world[0] - 10.).abs() < 1e-9 && (c.world[1] - 4.).abs() < 1e-9), "{perp:?}");
    // Far from every curve: nothing (and the broad phase skips the curves).
    assert!(at(SnapKind::End, [30., 30., 0.], [0., 0.]).is_empty());
}

/// Two NURBS curves crossing give an Intersection snap.
#[test]
fn crossing_nurbs_curves_snap_at_their_intersection() {
    let a = nurbs(1, vec![[-5., 0., 0.], [5., 0., 0.]]);
    let b = nurbs(1, vec![[0., -5., 0.], [0., 5., 0.]]);
    let (scene, _) = nurbs_scene(vec![a, b]);
    let s = screen([0., 0., 0.]);
    let found = candidates(&scene, &camera(), viewport(), [s[0] + 2., s[1]], only(SnapKind::Intersection), None).unwrap();
    assert!(found.iter().any(|c| c.world[0].abs() < 1e-9 && c.world[1].abs() < 1e-9), "{found:?}");
}
