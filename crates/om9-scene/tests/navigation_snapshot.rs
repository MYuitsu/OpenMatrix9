// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::{Limits, Usage},
    geometry::{Geometry, Mesh, Transform},
    model::{Document, Entity},
    transaction::{TransactionCoordinator, World},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind, Scene},
};
fn capture(transform: Transform) -> Scene {
    let mut document = Document::new("navigation snapshot");
    let mut entity = Entity::new(
        "triangle",
        Geometry::Mesh(Mesh {
            vertices: vec![[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]],
            faces: vec![vec![0, 1, 2]],
            normals: vec![],
            colors: vec![],
        }),
        document.current_layer,
    );
    entity.placement = transform;
    document.entities.insert(entity.id, entity);
    let id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    Scene::capture_in_world(&coordinator, id, Usage::default()).unwrap()
}
#[test]
fn conservative_pick_bounds_enclose_scaled_sheared_vertices_at_large_coordinates() {
    let transform = Transform([
        [4., 2., 0., 1e9],
        [0., 0.5, 0., -1e9],
        [0., 0., 3., 1e9],
        [0., 0., 0., 1.],
    ]);
    let scene = capture(transform);
    let object = &scene.objects[0];
    let (centre, radius) = object.sphere_bounds().unwrap();
    for p in [[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]] {
        let p = transform.point(p).unwrap();
        let distance = (0..3)
            .map(|a| (p[a] - centre[a]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(distance <= radius);
    }
}
#[test]
fn worker_snapshot_and_canonical_scene_pick_the_same_face_and_corners() {
    let scene = capture(Transform::identity());
    let snapshot = scene.render_snapshot();
    let camera = Camera::orthographic([0., 0., 10.], [0.; 3], [0., 1., 0.], 4.).unwrap();
    let vp = Viewport {
        origin: [0.; 2],
        size: [400., 400.],
    };
    for (p, kind) in [
        ([0., 0., 0.], PickKind::Face),
        ([-1., -1., 0.], PickKind::Vertex),
        ([1., -1., 0.], PickKind::Vertex),
    ] {
        let screen = camera.project(p, vp).unwrap().unwrap().screen;
        let canonical = scene
            .pick_kind(screen, &camera, vp, 6., PickIntent::Select, kind)
            .unwrap()
            .expect("canonical hit");
        let worker = snapshot
            .pick_kind(screen, &camera, vp, 6., PickIntent::Select, kind)
            .unwrap()
            .expect("snapshot hit");
        assert_eq!(canonical.root, worker.root);
        assert_eq!(canonical.subset, worker.subset);
        assert_eq!(canonical.world_point, worker.world_point);
    }
}
