// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-03 / L3-008 (L4-04 integration): one block definition shared by
//! two occurrences.
//!
//! The definition owns the member topology; an occurrence owns only a placement
//! and a reference to the definition. These tests prove that the scene layer
//! expands the shared definition once per occurrence (never promoting the
//! definition's members to extra scene roots), that picking stays rooted at the
//! occurrence that was hit while still reporting which definition member was
//! hit, that occurrence placements move independently, and that a single
//! `DefinitionInsert` plus a single `undo()` removes the definition, both
//! occurrences and every scene object built from them.
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Transform},
    identity::{DefinitionId, DocumentId, ObjectId},
    model::{Definition, Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickHit, PickIntent, Scene, SceneObject, Subelement},
};

/// Definition member 1: a line from `[0,0,0]` to `[10,0,0]` in definition space.
const CURVE_START: Point3 = [0., 0., 0.];
const CURVE_END: Point3 = [10., 0., 0.];
/// Definition member 2: a point at `[4,12,0]` in definition space.
const POINT_MEMBER: Point3 = [4., 12., 0.];
/// The second occurrence moves; the first stays at the identity placement.
const SECOND_PLACEMENT: Point3 = [0., -15., 0.];

struct Fixture {
    coordinator: TransactionCoordinator,
    document: DocumentId,
    definition: DefinitionId,
    first: ObjectId,
    second: ObjectId,
    curve_member: ObjectId,
    point_member: ObjectId,
}

/// Build the shared block through exactly one `Mutation::DefinitionInsert`, so
/// the definition and both occurrences exist atomically and one Undo can remove
/// them together.
fn fixture() -> Fixture {
    let document = Document::new("shared block instances");
    let curve = Entity::new(
        "block curve",
        Geometry::Curve(Curve::Line {
            start: CURVE_START,
            end: CURVE_END,
            domain: [0., 10.],
        }),
        document.current_layer,
    );
    let curve_member = curve.id;
    let point = Entity::new(
        "block point",
        Geometry::Point(POINT_MEMBER),
        document.current_layer,
    );
    let point_member = point.id;
    let definition = Definition::new("block", vec![curve, point]);
    let definition_id = definition.id;
    let first = Entity::new(
        "first occurrence",
        Geometry::Instance(definition_id),
        document.current_layer,
    );
    let first_id = first.id;
    let second = Entity {
        placement: Transform::translation(SECOND_PLACEMENT),
        ..Entity::new(
            "second occurrence",
            Geometry::Instance(definition_id),
            document.current_layer,
        )
    };
    let second_id = second.id;
    let document_id = document.id;
    let mut coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let prepared = coordinator
        .prepare(TransactionRequest::new(
            "shared block",
            vec![Mutation::DefinitionInsert {
                document: document_id,
                expected_revision: 1,
                definition: Box::new(definition),
                instances: vec![first, second],
            }],
        ))
        .unwrap();
    coordinator.commit(prepared).unwrap();
    Fixture {
        coordinator,
        document: document_id,
        definition: definition_id,
        first: first_id,
        second: second_id,
        curve_member,
        point_member,
    }
}

fn document(f: &Fixture) -> &Document {
    f.coordinator.world().documents[&f.document].as_ref()
}

fn capture(f: &Fixture) -> Scene {
    Scene::capture(
        std::sync::Arc::clone(&f.coordinator.world().documents[&f.document]),
        f.coordinator.epoch(),
        &Limits::default(),
    )
    .unwrap()
}

fn camera() -> Camera {
    Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap()
}

fn viewport() -> Viewport {
    Viewport {
        origin: [0., 0.],
        size: [400., 400.],
    }
}

/// The occurrence's world location of one definition member local point, taken
/// through the documented `Document::member_transform` path.
fn member_world(f: &Fixture, occurrence: ObjectId, member: ObjectId, local: Point3) -> Point3 {
    document(f)
        .member_transform(occurrence, &[member])
        .unwrap()
        .point(local)
        .unwrap()
}

fn scene_object(scene: &Scene, root: ObjectId, member: ObjectId) -> &SceneObject {
    scene
        .objects
        .iter()
        .find(|object| object.root == root && object.path == [member])
        .expect("scene object for occurrence member")
}

fn pick_world(scene: &Scene, world: Point3) -> Option<PickHit> {
    let camera = camera();
    let viewport = viewport();
    let screen = camera.project(world, viewport).unwrap().unwrap().screen;
    scene
        .pick(screen, &camera, viewport, 6., PickIntent::Select)
        .unwrap()
}

#[test]
fn shared_definition_captures_both_occurrences_without_duplicating_members_as_roots() {
    let f = fixture();
    let scene = capture(&f);
    let doc = document(&f);

    // Stored topology stays shared: one definition with two members, two
    // occurrences that reference it, and no member copied into the entity map.
    assert_eq!(doc.definitions.len(), 1);
    assert_eq!(doc.definitions[&f.definition].members.len(), 2);
    assert_eq!(doc.entities.len(), 2);
    for occurrence in [f.first, f.second] {
        assert!(matches!(
            doc.entities[&occurrence].geometry,
            Geometry::Instance(id) if id == f.definition
        ));
    }
    assert!(!doc.entities.contains_key(&f.curve_member));
    assert!(!doc.entities.contains_key(&f.point_member));

    // Two scene roots, one per occurrence.
    let mut roots: Vec<ObjectId> = scene.objects.iter().map(|object| object.root).collect();
    roots.sort_unstable();
    roots.dedup();
    assert_eq!(roots, vec![f.first.min(f.second), f.first.max(f.second)]);
    assert!(scene.diagnostics.is_empty());

    // Each occurrence materialises both definition members exactly once, and no
    // definition member is ever promoted to a scene root of its own.
    assert_eq!(
        scene.objects.len(),
        4,
        "two drawable definition members per occurrence"
    );
    for object in &scene.objects {
        assert_ne!(object.root, f.curve_member, "member promoted to a root");
        assert_ne!(object.root, f.point_member, "member promoted to a root");
        assert_eq!(object.path.len(), 1);
        assert_eq!(object.leaf, object.path[0]);
        assert!(object.path[0] == f.curve_member || object.path[0] == f.point_member);
    }
    for member in [f.curve_member, f.point_member] {
        let first = scene_object(&scene, f.first, member);
        let second = scene_object(&scene, f.second, member);
        // Shared topology: both occurrences display the same member geometry in
        // definition space; only the occurrence transform differs.
        assert_eq!(first.buffers, second.buffers);
    }
    assert_eq!(
        scene_object(&scene, f.first, f.curve_member).transform,
        Transform::identity()
    );
    assert_eq!(
        scene_object(&scene, f.second, f.curve_member).transform,
        Transform::translation(SECOND_PLACEMENT)
    );
}

#[test]
fn each_occurrence_picks_its_own_root_and_reports_the_definition_member_path() {
    let f = fixture();
    let scene = capture(&f);

    // A world point on the first occurrence's transformed curve member.
    let first_world = member_world(&f, f.first, f.curve_member, [5., 0., 0.]);
    let first_hit = pick_world(&scene, first_world).expect("first occurrence member is pickable");
    assert_eq!(first_hit.root, f.first);
    assert_eq!(first_hit.path, vec![f.curve_member]);
    assert_eq!(
        scene_object(&scene, first_hit.root, first_hit.path[0]).leaf,
        f.curve_member
    );
    let Subelement::CurveParameter(parameter) = first_hit.subset else {
        panic!("expected the curve parameter of the hit member")
    };
    assert!((parameter - 5.).abs() < 1e-9);
    for (axis, expected) in first_world.iter().enumerate().take(2) {
        assert!((first_hit.world_point[axis] - expected).abs() < 1e-9);
    }

    // The same definition member at the second occurrence's own placement.
    let second_world = member_world(&f, f.second, f.curve_member, [6., 0., 0.]);
    let second_hit =
        pick_world(&scene, second_world).expect("second occurrence member is pickable");
    assert_eq!(second_hit.root, f.second);
    assert_eq!(second_hit.path, vec![f.curve_member]);
    assert_ne!(
        first_hit.root, second_hit.root,
        "shared topology, independent occurrences"
    );
    let Subelement::CurveParameter(parameter) = second_hit.subset else {
        panic!("expected the curve parameter of the hit member")
    };
    assert!((parameter - 6.).abs() < 1e-9);
    for (axis, expected) in second_world.iter().enumerate().take(2) {
        assert!((second_hit.world_point[axis] - expected).abs() < 1e-9);
    }

    // The second definition member is reported as its own member path, once per
    // occurrence, still rooted at the occurrence that owns the placement.
    for occurrence in [f.first, f.second] {
        let world = member_world(&f, occurrence, f.point_member, POINT_MEMBER);
        let hit = pick_world(&scene, world).expect("definition point member is pickable");
        assert_eq!(hit.root, occurrence);
        assert_eq!(hit.path, vec![f.point_member]);
        assert_eq!(hit.subset, Subelement::Whole);
        assert_eq!(
            scene_object(&scene, hit.root, hit.path[0]).leaf,
            f.point_member
        );
    }
}

#[test]
fn occurrence_placements_are_independent_and_member_transform_equals_the_placement() {
    let mut f = fixture();
    {
        let doc = document(&f);
        assert_eq!(
            doc.member_transform(f.first, &[]).unwrap(),
            Transform::identity()
        );
        assert_eq!(
            doc.member_transform(f.second, &[]).unwrap(),
            Transform::translation(SECOND_PLACEMENT)
        );
    }
    let members_before = document(&f).definitions[&f.definition].members.clone();

    // A placement-only write on the first occurrence.
    let source = document(&f).stamp(f.first).unwrap();
    let prepared = f
        .coordinator
        .prepare(TransactionRequest::new(
            "move first occurrence",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([0., 5., 0.]),
            }],
        ))
        .unwrap();
    f.coordinator.commit(prepared).unwrap();

    let doc = document(&f);
    assert_eq!(
        doc.member_transform(f.first, &[]).unwrap(),
        Transform::translation([0., 5., 0.])
    );
    assert_eq!(
        doc.member_transform(f.second, &[]).unwrap(),
        Transform::translation(SECOND_PLACEMENT),
        "moving the first occurrence must not move the second"
    );
    assert_eq!(
        doc.entities[&f.second].placement,
        Transform::translation(SECOND_PLACEMENT)
    );
    // The shared definition topology is not touched by an occurrence move.
    assert_eq!(doc.definitions.len(), 1);
    assert_eq!(doc.definitions[&f.definition].members, members_before);

    // The moved member left its old world location; the untouched occurrence is
    // still pickable exactly where it was.
    let scene = capture(&f);
    assert!(
        pick_world(&scene, [5., 0., 0.]).is_none(),
        "the first occurrence must not still be at its old placement"
    );
    let moved = member_world(&f, f.first, f.curve_member, [5., 0., 0.]);
    assert!((moved[1] - 5.).abs() < 1e-12);
    assert_eq!(
        pick_world(&scene, moved)
            .expect("moved occurrence is pickable")
            .root,
        f.first
    );
    let second_world = member_world(&f, f.second, f.curve_member, [6., 0., 0.]);
    assert_eq!(second_world[1], SECOND_PLACEMENT[1]);
    assert_eq!(
        pick_world(&scene, second_world)
            .expect("second occurrence kept its own placement")
            .root,
        f.second
    );
}

#[test]
fn one_definition_insert_and_one_undo_remove_definition_both_occurrences_and_their_objects() {
    let mut f = fixture();
    assert_eq!(
        f.coordinator.undo_len(),
        1,
        "the fixture is exactly one DefinitionInsert"
    );
    let scene = capture(&f);
    assert_eq!(scene.objects.len(), 4);

    f.coordinator.undo().unwrap();
    let doc = document(&f);
    assert!(
        doc.definitions.is_empty(),
        "one Undo removed the definition"
    );
    assert!(doc.entities.is_empty(), "one Undo removed both occurrences");
    assert!(!doc.definitions.contains_key(&f.definition));
    for id in [f.first, f.second, f.curve_member, f.point_member] {
        assert!(!doc.entities.contains_key(&id));
    }

    // The scene captured after the Undo contains none of their objects.
    let scene = capture(&f);
    assert!(scene.objects.is_empty());
    assert!(scene.diagnostics.is_empty());
    for object in &scene.objects {
        for id in [f.first, f.second, f.curve_member, f.point_member] {
            assert_ne!(object.root, id);
            assert!(!object.path.contains(&id));
        }
    }
}
