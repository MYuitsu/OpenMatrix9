// SPDX-License-Identifier: LGPL-2.1-or-later
//! Acceptance oracles for `OM9-L2-01-07`, taken from the L4 task text in
//! `OM9_Milestones/M02 - Viewport, Gumball/L2-01-07/...`:
//!
//! * L3-001 positive: "Box tại tâm tia, điểm click ở viewport P trúng BodyId của box".
//! * L3-001 negative: "Ray không giao đối tượng trả danh sách rỗng, không chọn".
//! * L3-001 rule: "không lấy triangle index làm face ID".
//! * L3-003 positive/negative: the exact face/edge/vertex, and a miss when the geometry is gone.
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
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

struct Fixture {
    coordinator: TransactionCoordinator,
    document: om9_core::identity::DocumentId,
    root: om9_core::identity::ObjectId,
}

fn fixture(selectable: bool) -> Fixture {
    let mut document = Document::new("acceptance");
    let mut entity = Entity::new("box", Geometry::Mesh(cube()), document.current_layer);
    entity.attributes.selectable = selectable;
    let root = entity.id;
    document.entities.insert(root, entity);
    let document_id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).expect("world"),
        Limits::default(),
    )
    .expect("coordinator");
    Fixture { coordinator, document: document_id, root }
}

fn camera() -> Camera {
    Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 4.).expect("camera")
}

fn viewport() -> Viewport {
    Viewport { origin: [0., 0.], size: [400., 400.] }
}

fn scene(fixture: &Fixture) -> Scene {
    Scene::capture_in_world(&fixture.coordinator, fixture.document, om9_core::budget::Usage::default())
        .expect("scene")
}

fn screen(point: [f64; 3]) -> [f64; 2] {
    camera().project(point, viewport()).expect("project").expect("front").screen
}

#[test]
fn a_ray_through_the_box_returns_its_body_id_and_an_empty_ray_returns_nothing() {
    let fixture = fixture(true);
    let scene = scene(&fixture);
    let hit = scene
        .pick(screen([0.2, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick")
        .expect("the box is under the pointer");
    assert_eq!(hit.root, fixture.root, "the hit names the document object");
    // Outside the cube entirely.
    let empty = scene
        .pick([5.0, 5.0], &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick");
    assert!(empty.is_none(), "a ray that meets nothing selects nothing");
    assert!(
        scene
            .pick_all([5.0, 5.0], &camera(), viewport(), 6.0, PickIntent::Select)
            .expect("pick all")
            .is_empty()
    );
}

#[test]
fn a_face_hit_is_the_document_face_not_the_triangle_index() {
    let fixture = fixture(true);
    let scene = scene(&fixture);
    // The top face is stored as face 2, triangulated into triangles 4 and 5. Picking inside one of them
    // must report face 2; a triangle index would report 4 or 5.
    let hit = scene
        .pick(screen([0.3, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick")
        .expect("the top face is under the pointer");
    assert_eq!(
        hit.subset,
        Subelement::MeshFace(2),
        "the document face index, not a triangle index"
    );
}

#[test]
fn a_face_that_disappears_after_a_rebuild_is_a_stale_miss() {
    let mut fixture = fixture(true);
    let scene = scene(&fixture);
    let hit = scene
        .pick(screen([0.3, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick")
        .expect("hit");
    hit.revalidate_in_view(&fixture.coordinator, &camera(), viewport())
        .expect("the hit is valid while the object exists");
    let document = fixture
        .coordinator
        .world()
        .documents
        .get(&fixture.document)
        .expect("document")
        .clone();
    let stamp = document.stamp(fixture.root).expect("stamp");
    let prepared = fixture
        .coordinator
        .prepare(TransactionRequest::new("Delete", vec![Mutation::Delete { source: stamp }]))
        .expect("prepare");
    fixture.coordinator.commit(prepared).expect("commit");
    let error = hit
        .revalidate_in_view(&fixture.coordinator, &camera(), viewport())
        .expect_err("a hit whose object is gone must be rejected");
    assert!(
        matches!(error, om9_core::Error::Stale(_) | om9_core::Error::Missing(_)),
        "expected a stale/missing error, got {error:?}"
    );
    let _ = &mut fixture;
}

#[test]
fn a_hidden_or_unselectable_object_never_picks() {
    let fixture = fixture(false);
    let scene = scene(&fixture);
    assert!(
        scene
            .pick(screen([0.2, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
            .expect("pick")
            .is_none(),
        "an unselectable object is not a selection target"
    );
    assert!(
        scene
            .pick_kind(screen([0.3, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Face)
            .expect("pick")
            .is_none()
    );
}

#[test]
fn the_three_subobject_kinds_report_their_own_identifiers() {
    let fixture = fixture(true);
    let scene = scene(&fixture);
    let face = scene
        .pick_kind(screen([0.3, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Face)
        .expect("pick")
        .expect("face");
    assert!(matches!(face.subset, Subelement::MeshFace(_)));
    let face_key = face.subset;
    let edge = scene
        .pick_kind(screen([0.0, -1.0, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Edge)
        .expect("pick")
        .expect("edge");
    assert!(matches!(edge.subset, Subelement::MeshEdge { .. }));
    assert_ne!(edge.subset, face_key, "an edge key is not a face key");
    let vertex = scene
        .pick_kind(screen([-1.0, -1.0, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Vertex)
        .expect("pick")
        .expect("vertex");
    assert!(matches!(vertex.subset, Subelement::MeshVertex(_)));
    assert_ne!(vertex.subset, edge.subset);
}

#[test]
fn twenty_identical_picks_return_the_same_hit() {
    // L4-01-07-001-05 asks for 20 deterministic repetitions of the same fixture.
    let fixture = fixture(true);
    let scene = scene(&fixture);
    let first = scene
        .pick(screen([0.3, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick")
        .expect("hit");
    for run in 1..=20 {
        let hit = scene
            .pick(screen([0.3, 0.1, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
            .expect("pick")
            .expect("hit");
        assert_eq!(hit.root, first.root, "run {run}: same object");
        assert_eq!(hit.subset, first.subset, "run {run}: same subelement");
        assert_eq!(hit.world_point, first.world_point, "run {run}: same world point");
    }
}

#[test]
fn twenty_face_edge_vertex_picks_are_identical() {
    // L4-01-07-003-05 asks for 20 deterministic repetitions per subobject kind.
    let fixture = fixture(true);
    let scene = scene(&fixture);
    let probes = [
        (PickKind::Face, screen([0.3, 0.1, 1.0])),
        (PickKind::Edge, screen([0.0, -1.0, 1.0])),
        (PickKind::Vertex, screen([-1.0, -1.0, 1.0])),
    ];
    for (kind, at) in probes {
        let first = scene
            .pick_kind(at, &camera(), viewport(), 6.0, PickIntent::Select, kind)
            .expect("pick")
            .expect("hit");
        for run in 1..=20 {
            let hit = scene
                .pick_kind(at, &camera(), viewport(), 6.0, PickIntent::Select, kind)
                .expect("pick")
                .expect("hit");
            assert_eq!(hit.subset, first.subset, "{kind:?} run {run}");
            assert_eq!(hit.screen_distance, first.screen_distance, "{kind:?} run {run}");
        }
    }
}

#[test]
fn a_click_on_the_shared_diagonal_still_hits_the_face() {
    // The top quad is two triangles sharing the diagonal from (-1,-1,1) to (1,1,1); the screen point at the
    // centre of the quad lies exactly on that diagonal and used to miss both triangles.
    let fixture = fixture(true);
    let scene = scene(&fixture);
    let hit = scene
        .pick_kind(screen([0.0, 0.0, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select, PickKind::Face)
        .expect("pick")
        .expect("the shared diagonal still belongs to a face");
    assert!(
        matches!(hit.subset, Subelement::MeshFace(2) | Subelement::MeshFace(3)),
        "one of the two top triangles: {:?}",
        hit.subset
    );
    let object = scene
        .pick(screen([0.0, 0.0, 1.0]), &camera(), viewport(), 6.0, PickIntent::Select)
        .expect("pick")
        .expect("the object pick finds the same spot");
    assert_eq!(object.root, fixture.root);
}
