// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "gpu-sphere")]
use om9_app::session::{Controller, SelectedSubobject};
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    gumball::{Drag, Handle, Session},
    model::{Definition, Document, Entity},
    transaction::Cancellation,
};
use om9_kernel::brep::{box_brep, read_brep, source_volume, sphere_brep};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind, Subelement},
};
use uuid::Uuid;

fn controller(geometry: Geometry) -> (Controller, om9_core::identity::ObjectId) {
    let mut d = Document::new("native BRep picking");
    let e = Entity::new("native", geometry, None);
    let id = e.id;
    d.entities.insert(id, e);
    (Controller::new(d, Limits::default()).unwrap(), id)
}
fn top() -> (Camera, Viewport) {
    (
        Camera::orthographic([1., 1., 100.], [1., 1., 0.], [0., 1., 0.], 10.).unwrap(),
        Viewport {
            origin: [0.; 2],
            size: [800., 600.],
        },
    )
}
fn pick(c: &Controller, world: [f64; 3], kind: PickKind) -> Option<om9_scene::scene::PickHit> {
    let (camera, viewport) = top();
    let screen = camera.project(world, viewport).unwrap().unwrap().screen;
    c.scene()
        .unwrap()
        .pick_kind(screen, &camera, viewport, 0.1, PickIntent::Select, kind)
        .unwrap()
}
#[test]
fn actual_box_faces_edges_and_vertices_use_topology_ids_not_mesh_offsets() {
    let l = Limits::default();
    let shape = box_brep([2.; 3], 0.01, &l).unwrap();
    assert_eq!(
        (
            shape.display.faces.len(),
            shape.display.edges.len(),
            shape.display.vertices.len()
        ),
        (6, 12, 8)
    );
    assert!((source_volume(&shape, &l).unwrap() - 8.).abs() < 1e-10);
    let (mut c, id) = controller(Geometry::Brep(shape.clone()));
    for (point, kind) in [
        ([0.4, 0.7, 2.], PickKind::Face),
        ([0., 0.7, 2.], PickKind::Edge),
        ([0., 0., 2.], PickKind::Vertex),
    ] {
        let hit = pick(&c, point, kind).unwrap();
        assert_eq!(hit.root, id);
        assert!(match hit.subset {
            Subelement::BrepFace(key) => shape.display.faces.contains(&key),
            Subelement::BrepEdge(key) => shape.display.edges.contains(&key),
            Subelement::BrepVertex(key) => shape.display.vertex_ids.contains(&key),
            _ => false,
        });
        c.select_subobject_ref(SelectedSubobject::from_hit(&hit))
            .unwrap();
    }
    assert!(
        pick(&c, [0.7, 0.7, 2.], PickKind::Edge).is_none(),
        "face triangulation diagonal must not become a BRep edge"
    );
    assert!(pick(&c, [0.7, 0.7, 2.], PickKind::Vertex).is_none());
    assert!(c.select_subobject(id, Subelement::MeshFace(0)).is_err());
    assert!(
        c.select_subobject(id, Subelement::BrepFace(Uuid::new_v4()))
            .is_err()
    );
}
#[test]
fn curved_surface_tessellation_resolves_one_real_face_and_excludes_fake_vertices() {
    let l = Limits::default();
    let shape = sphere_brep(2., 0.03, &l).unwrap();
    assert_eq!(shape.display.faces.len(), 1);
    assert!(shape.display.triangles.len() > 20);
    assert!(
        (source_volume(&shape, &l).unwrap() - 4. / 3. * std::f64::consts::PI * 8.).abs() < 1e-8
    );
    let key = shape.display.faces[0];
    let (c, _) = controller(Geometry::Brep(shape));
    for point in [[0.3, 0.4, 1.], [0.7, -0.4, 1.], [-0.4, 0.8, 1.]] {
        assert_eq!(
            pick(&c, point, PickKind::Face).unwrap().subset,
            Subelement::BrepFace(key)
        );
        assert!(pick(&c, point, PickKind::Vertex).is_none());
    }
}
#[test]
fn source_and_topology_survive_cold_archive_and_transform_undo_without_reviving_old_picks() {
    let l = Limits::default();
    let shape = box_brep([10., 20., 30.], 0.01, &l).unwrap();
    let source = shape.source().to_owned();
    let (mut c, id) = controller(Geometry::Brep(shape.clone()));
    let hit = pick(&c, [1., 1., 30.], PickKind::Face).unwrap();
    let old = SelectedSubobject::from_hit(&hit);
    c.select_subobject_ref(old.clone()).unwrap();
    c.select_objects(&[id]).unwrap();
    let mut g = Session::begin(c.gumball_pivot().unwrap(), c.gumball_targets(), false).unwrap();
    g.drag(Handle::MoveFree, Drag::WorldDelta([5., 2., 1.]))
        .unwrap();
    c.commit_gumball(&g, "Move").unwrap();
    assert!(c.subselection().is_empty());
    assert!(c.select_subobject_ref(old.clone()).is_err());
    c.undo().unwrap();
    assert!(c.select_subobject_ref(old).is_err());
    c.redo().unwrap();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build/standalone-validation")
        .join(format!("m02-brep-cold-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("source.om9");
    om9_archive::Archive::save(&path, c.document(), &l, &Cancellation::default()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_om9"))
        .arg("--inspect")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["objects"][0]["id"], id.to_string());
    let restored = Controller::from_file(&path, l).unwrap();
    assert_eq!(restored.document().entities[&id].placement, Transform::translation([5.,2.,1.]));
    let Geometry::Brep(loaded) = &restored.document().entities[&id].geometry else {
        panic!("BRep downgraded")
    };
    assert_eq!(loaded, &shape);
    assert_eq!(loaded.source(), source);
    assert!((source_volume(loaded, &l).unwrap() - 6000.).abs() < 1e-8);
    let rebuilt = read_brep(loaded.source(), 0.02, &l).unwrap();
    assert!((source_volume(&rebuilt, &l).unwrap() - 6000.).abs() < 1e-8);
    assert_ne!(
        rebuilt.display.faces, loaded.display.faces,
        "retessellation needs a new publication/identity, never guessed native offsets"
    );
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
#[test]
fn source_binding_corrupt_cache_invalid_native_source_and_budgets_are_rejected() {
    let l = Limits::default();
    let shape = box_brep([2.; 3], 0.01, &l).unwrap();
    let mut value = serde_json::to_value(&shape).unwrap();
    value["source"] = serde_json::json!("CASCADE Topology invalid");
    assert!(
        serde_json::from_value::<om9_core::brep::Brep>(value)
            .unwrap()
            .validate(&l)
            .is_err()
    );
    let mut changed = shape.clone();
    changed.display.points[0][0] += 1.;
    assert!(changed.validate(&l).is_err());
    let mut changed = shape.clone();
    changed.display.faces[0] = Uuid::nil();
    assert!(changed.validate(&l).is_err());
    let tight = Limits { max_points: 2, ..l };
    assert!(box_brep([2.; 3], 0.01, &tight).is_err());
    assert!(read_brep("not a shape", 0.01, &l).is_err());
    assert!(read_brep("", 0.01, &l).is_err());
    assert!(box_brep([f64::NAN, 2., 2.], 0.01, &l).is_err());
    assert!(sphere_brep(2., 0., &l).is_err());
}
#[test]
fn mirrored_nested_occurrences_keep_shared_native_face_keys_under_distinct_paths() {
    let l = Limits::default();
    let shape = box_brep([2.; 3], 0.01, &l).unwrap();
    let leaf = Entity::new("native member", Geometry::Brep(shape), None);
    let leaf_id = leaf.id;
    let definition = Definition::new("shared native", vec![leaf]);
    let definition_id = definition.id;
    let mut a = Entity::new("left", Geometry::Instance(definition_id), None);
    a.placement = Transform::translation([-4., 0., 0.]);
    let mut b = Entity::new("right mirror", Geometry::Instance(definition_id), None);
    b.placement = Transform([
        [-1., 0., 0., 6.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let mut d = Document::new("native paths");
    d.definitions.insert(definition_id, definition);
    d.entities.insert(a.id, a);
    d.entities.insert(b.id, b);
    let mut c = Controller::new(d, l).unwrap();
    let left = pick(&c, [-3.6, 0.7, 2.], PickKind::Face).unwrap();
    let right = pick(&c, [5.6, 0.7, 2.], PickKind::Face).unwrap();
    assert_eq!(left.path, vec![leaf_id]);
    assert_eq!(left.subset, right.subset);
    assert_ne!(left.root, right.root);
    c.select_subobject_ref(SelectedSubobject::from_hit(&left))
        .unwrap();
    c.select_subobject_ref(SelectedSubobject::from_hit(&right))
        .unwrap();
    assert_eq!(c.subselection().len(), 2);
}
