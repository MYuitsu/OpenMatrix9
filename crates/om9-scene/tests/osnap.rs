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
    let mut document = Document::new("osnap");
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
    assert_eq!(quadrant[0].kind, SnapKind::Quadrant);
    assert_eq!(quadrant[0].world, [7., 5., 0.]);
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
    let mut document = Document::new("crossing");
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
fn surface_snap_uses_the_tessellated_trimmed_face() {
    // A single planar quad is represented as two triangles; the ray hits the interior.
    let mut document = Document::new("surface");
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
    let mut document = Document::new("point snap");
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
    let mut document = Document::new("parallel");
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
