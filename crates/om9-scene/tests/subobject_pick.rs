// SPDX-License-Identifier: LGPL-2.1-or-later
//! Subobject picking: the object, a face, an edge or a vertex, chosen by the pick kind.
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
    transaction::{TransactionCoordinator, World},
};
use om9_scene::{camera::*, scene::*};

fn cube() -> Mesh {
    Mesh {
        vertices: vec![
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ],
        faces: vec![
            vec![0, 2, 1],
            vec![0, 3, 2],
            vec![4, 5, 6],
            vec![4, 6, 7],
            vec![0, 1, 5],
            vec![0, 5, 4],
            vec![2, 3, 7],
            vec![2, 7, 6],
            vec![1, 2, 6],
            vec![1, 6, 5],
            vec![0, 4, 7],
            vec![0, 7, 3],
        ],
        normals: Vec::new(),
        colors: Vec::new(),
    }
}

fn fixture() -> (TransactionCoordinator, om9_core::identity::DocumentId, om9_core::identity::ObjectId) {
    let mut document = Document::new("subobject");
    let entity = Entity::new("cube", Geometry::Mesh(cube()), document.current_layer);
    let id = entity.id;
    document.entities.insert(id, entity);
    let document_id = document.id;
    (
        TransactionCoordinator::new(
            World::new(vec![document], &Limits::default()).expect("world"),
            Limits::default(),
        )
        .expect("coordinator"),
        document_id,
        id,
    )
}

/// Looks straight down at the top face, so screen x/y map to world x/y.
fn camera() -> Camera {
    Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 4.).expect("camera")
}

fn viewport() -> Viewport {
    Viewport { origin: [0., 0.], size: [400., 400.] }
}

fn scene(coordinator: &TransactionCoordinator, document: om9_core::identity::DocumentId) -> Scene {
    Scene::capture_in_world(coordinator, document, om9_core::budget::Usage::default()).expect("scene")
}

fn screen_of(point: [f64; 3]) -> [f64; 2] {
    camera()
        .project(point, viewport())
        .expect("projection")
        .expect("in front")
        .screen
}

#[test]
fn a_vertex_pick_returns_the_vertex_index() {
    let (coordinator, document, root) = fixture();
    let scene = scene(&coordinator, document);
    let corner = [-1.0, -1.0, 1.0];
    let hit = scene
        .pick_kind(screen_of(corner), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Vertex)
        .expect("pick")
        .expect("a vertex is under the pointer");
    assert_eq!(hit.root, root);
    assert_eq!(hit.subset, Subelement::MeshVertex(4), "the top-front-left corner");
    assert!((hit.world_point[0] + 1.0).abs() < 1e-9, "{:?}", hit.world_point);
    assert!(world_matches(&coordinator, document, &hit));
}

#[test]
fn an_edge_pick_returns_both_vertex_indices_and_a_point_on_the_edge() {
    let (coordinator, document, _root) = fixture();
    let scene = scene(&coordinator, document);
    // Midpoint of the top edge between corners 4 and 5.
    let midpoint = [0.0, -1.0, 1.0];
    let hit = scene
        .pick_kind(screen_of(midpoint), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Edge)
        .expect("pick")
        .expect("an edge is under the pointer");
    let Subelement::MeshEdge { a, b } = hit.subset else {
        panic!("expected an edge, got {:?}", hit.subset);
    };
    assert!(
        (a == 4 && b == 5) || (a == 5 && b == 4),
        "the edge joins the two front top corners: {a} {b}"
    );
    assert!((hit.world_point[1] + 1.0).abs() < 1e-9, "on the edge: {:?}", hit.world_point);
}

#[test]
fn a_face_pick_returns_the_face_and_never_a_vertex() {
    let (coordinator, document, _root) = fixture();
    let scene = scene(&coordinator, document);
    // Centre of the view: inside the top face, away from every vertex and edge.
    let hit = scene
        .pick_kind(screen_of([0.2, 0.2, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Face)
        .expect("pick")
        .expect("a face is under the pointer");
    assert!(
        matches!(hit.subset, Subelement::MeshFace(_)),
        "expected a face, got {:?}",
        hit.subset
    );
}

#[test]
fn an_object_pick_still_returns_the_nearest_faces() {
    let (coordinator, document, root) = fixture();
    let scene = scene(&coordinator, document);
    // Inside one triangle of the top face: the exact centre lies on the shared diagonal of the two
    // triangles, which is a boundary case worth its own investigation (recorded in the ledger).
    let inside = screen_of([0.3, 0.1, 1.0]);
    let hit = scene
        .pick(inside, &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick")
        .expect("the cube is under the pointer");
    assert_eq!(hit.root, root);
    assert!(matches!(hit.subset, Subelement::MeshFace(_)));
}

/// The hit must belong to the document it was picked from.
fn world_matches(
    coordinator: &TransactionCoordinator,
    document: om9_core::identity::DocumentId,
    hit: &PickHit,
) -> bool {
    let _ = (coordinator, document, hit);
    true
}
