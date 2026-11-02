// SPDX-License-Identifier: LGPL-2.1-or-later
//! SDK polylines use canonical connected line Polycurves. Display must preserve
//! their exact corners and selection identity without a fictitious native domain.
use om9_core::{
    Error,
    budget::Limits,
    curve_build::polyline,
    geometry::{Curve, Geometry, Transform},
    identity::ObjectId,
    model::{Document, Entity},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind, RenderBuffers, Scene, Subelement},
};
use std::sync::Arc;

const RECTANGLE: [[f64; 3]; 5] = [
    [0., 0., 0.],
    [10., 0., 0.],
    [10., 20., 0.],
    [0., 20., 0.],
    [0., 0., 0.],
];

fn document_with_curve(curve: Curve, placement: Transform) -> (Document, ObjectId) {
    let mut document = Document::new("SDK polyline");
    let mut entity = Entity::new(
        "SDK profile",
        Geometry::Curve(curve),
        document.current_layer,
    );
    entity.placement = placement;
    let id = entity.id;
    document.entities.insert(id, entity);
    (document, id)
}

fn view() -> Viewport {
    Viewport {
        origin: [0., 0.],
        size: [400., 400.],
    }
}

fn near(a: [f64; 3], b: [f64; 3]) {
    for axis in 0..3 {
        assert!((a[axis] - b[axis]).abs() < 1e-10, "{a:?} != {b:?}");
    }
}

#[test]
fn sdk_profile_keeps_every_corner_closure_and_world_bounds() {
    let placement = Transform::translation([100., 200., 3.]);
    let (document, id) = document_with_curve(polyline(&RECTANGLE).unwrap(), placement);
    let scene = Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
    assert!(scene.diagnostics.is_empty(), "{:?}", scene.diagnostics);
    assert_eq!(
        scene.objects.len(),
        1,
        "supported SDK profile must not disappear"
    );
    let object = &scene.objects[0];
    assert_eq!(object.root, id);
    let RenderBuffers::Curve { points, parameters } = object.buffers.as_ref() else {
        panic!("SDK polyline must have curve buffers");
    };
    assert_eq!(points.as_slice(), RECTANGLE.as_slice());
    assert!(parameters.is_empty(), "no invented global native parameter");
    assert_eq!(points.windows(2).count(), 4);
    assert_eq!(points.first(), points.last());
    assert!(object.has_pick_geometry());
    let (center, radius) = object.sphere_bounds().unwrap();
    near(center, [105., 210., 3.]);
    assert!((radius - 125f64.sqrt()).abs() < 1e-10);
    for point in points {
        let world = placement.point(*point).unwrap();
        let distance = (0..3)
            .map(|axis| (world[axis] - center[axis]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(distance <= radius);
    }
}

#[test]
fn sdk_example_line_circle_and_profile_all_enter_the_scene() {
    let (mut document, _) =
        document_with_curve(polyline(&RECTANGLE).unwrap(), Transform::identity());
    for curve in [
        Curve::Line {
            start: [0.; 3],
            end: [10., 0., 0.],
            domain: [0., 1.],
        },
        Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 5.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        },
    ] {
        let entity = Entity::new(
            "SDK example",
            Geometry::Curve(curve),
            document.current_layer,
        );
        document.entities.insert(entity.id, entity);
    }
    let scene = Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
    assert_eq!(scene.objects.len(), 3);
    assert!(scene.diagnostics.is_empty());
    assert!(
        scene
            .objects
            .iter()
            .all(|object| object.has_pick_geometry())
    );
}

#[test]
fn placed_polyline_picks_exact_segments_without_adding_diagonals_or_native_bindings() {
    let placement = Transform::translation([100., 200., 3.]);
    let (document, id) = document_with_curve(polyline(&RECTANGLE).unwrap(), placement);
    let scene = Scene::capture(Arc::new(document), 7, &Limits::default()).unwrap();
    let camera =
        Camera::orthographic([105., 210., 103.], [105., 210., 3.], [0., 1., 0.], 40.).unwrap();
    for local in [[3., 0., 0.], [10., 7., 0.], [4., 20., 0.], [0., 8., 0.]] {
        let world = placement.point(local).unwrap();
        let screen = camera.project(world, view()).unwrap().unwrap().screen;
        let hit = scene
            .pick(screen, &camera, view(), 1., PickIntent::Select)
            .unwrap()
            .unwrap();
        assert_eq!(hit.root, id);
        assert!(matches!(hit.subset, Subelement::Whole));
        near(hit.world_point, world);
        let reference = scene
            .pick(
                screen,
                &camera,
                view(),
                1.,
                PickIntent::Reference { snap_locked: false },
            )
            .unwrap()
            .unwrap();
        near(reference.world_point, world);
    }
    let interior = camera
        .project([105., 210., 3.], view())
        .unwrap()
        .unwrap()
        .screen;
    assert!(
        scene
            .pick(interior, &camera, view(), 1., PickIntent::Select)
            .unwrap()
            .is_none(),
        "display must not invent a diagonal across the rectangle"
    );
    let edge = camera
        .project([110., 207., 3.], view())
        .unwrap()
        .unwrap()
        .screen;
    assert!(
        scene
            .pick_kind(
                edge,
                &camera,
                view(),
                1.,
                PickIntent::Select,
                PickKind::Edge
            )
            .unwrap()
            .is_none(),
        "segment subobjects need an explicit native segment binding"
    );
}

#[test]
fn polyline_display_admission_charges_its_exact_vertex_count() {
    let (document, _) = document_with_curve(polyline(&RECTANGLE).unwrap(), Transform::identity());
    let source_points = document.validate(&Limits::default()).unwrap().points;
    let limits = Limits {
        max_points: source_points + RECTANGLE.len() as u64,
        ..Limits::default()
    };
    let scene = Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
    assert_eq!(scene.objects.len(), 1);
    assert_eq!(scene.cache_usage().unwrap().points, RECTANGLE.len() as u64);
    let smaller = Limits {
        max_points: limits.max_points - 1,
        ..limits
    };
    document.validate(&smaller).unwrap();
    assert!(
        matches!(
            Scene::capture(Arc::new(document), 1, &smaller),
            Err(Error::Budget(_))
        ),
        "preflight must reject the display allocation rather than skip the profile"
    );
}

#[test]
fn reversed_polycurve_is_published_with_reversed_display_order() {
    let mut curve = polyline(&[[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,2.0,0.0]]).unwrap();
    if let Curve::Polycurve { reversed, .. } = &mut curve { *reversed = true; }
    let (document, id) = document_with_curve(curve, Transform::identity());
    let scene = Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
    assert!(scene.diagnostics.is_empty()); assert_eq!(scene.objects[0].root, id);
    let RenderBuffers::Curve { points, .. } = scene.objects[0].buffers.as_ref() else { panic!() };
    assert_eq!(points.as_slice(), &[[1.0,2.0,0.0],[1.0,0.0,0.0],[0.0,0.0,0.0]]);
    assert!(scene.objects[0].has_pick_geometry());
}

#[test]
fn disconnected_line_strokes_remain_disconnected() {
    let line = |start, end| Curve::Line { start, end, domain: [0.0,1.0] };
    let curve = Curve::Polycurve { segments: vec![line([0.0,0.0,0.0],[1.0,0.0,0.0]), line([2.0,0.0,0.0],[3.0,0.0,0.0])], reversed: false };
    let (document, _) = document_with_curve(curve, Transform::identity());
    let scene = Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
    let RenderBuffers::Mesh { vertices, edges, .. } = scene.objects[0].buffers.as_ref() else { panic!() };
    assert_eq!(vertices.len(), 4); assert_eq!(edges.as_slice(), &[[0,1],[2,3]]);
}

#[test]
fn unsupported_polycurves_report_precise_diagnostics() {
    let cases = [
        (
            Curve::Polycurve {
                segments: vec![Curve::Circle {
                    center: [0.; 3],
                    x: [1., 0., 0.],
                    y: [0., 1., 0.],
                    radius: 1.,
                    domain: [0., std::f64::consts::TAU],
                    angle_domain: None,
                }],
                reversed: false,
            },
            "direct line",
        ),
    ];
    for (curve, reason) in cases {
        let (document, _) = document_with_curve(curve, Transform::identity());
        let scene = Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
        assert!(scene.objects.is_empty());
        assert_eq!(scene.diagnostics.len(), 1);
        assert!(
            scene.diagnostics[0].contains(reason),
            "{:?}",
            scene.diagnostics
        );
    }
}
