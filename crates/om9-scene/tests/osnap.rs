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
        distance_px: 0., normal: None, parameter: None, source: None, second_source: None, surface_source: None };
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
    let worker = source_curve_candidates(&[arc.clone()], &source, scene.objects[0].display_transform().unwrap(), &camera(),
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
    let worker = source_curve_candidates(&[arc], &source, scene.objects[0].display_transform().unwrap(), &perspective,
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
    let worker = source_curve_candidates(&[arc], &source, scene.objects[0].display_transform().unwrap(), &camera(),
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
        let hits = source_curve_candidates(std::slice::from_ref(&curve), &source, transform, &camera(), viewport(),
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
    assert!(matches!(source_curve_candidates(std::slice::from_ref(&curve), &source, transform, &camera(),
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
    let query = |kind: SnapKind| source_curve_candidates(std::slice::from_ref(&curve), &source, transform, &camera,
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
            screen: [0.; 2], distance_px: 0., normal: None, parameter: None,
            source: None, second_source: None, surface_source: None };
        track.acquire(&anchor, plane, 1e-6).unwrap();
    }
    let hit = track.infer_with_source_tolerance([2., 2., 0.], 1e-6, 1e-6).unwrap();
    assert_eq!(hit.kind, SnapKind::SmartTrack);
    assert!((hit.world[0] - 2.).abs() < 1e-7 && (hit.world[1] - 2.).abs() < 1e-7);
    assert!(hit.source.is_none() && hit.root.is_none());
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
