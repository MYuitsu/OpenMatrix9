// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    curve_build,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
    rhino_semantics::{MeshUv, PbrMaterial, mesh_uv_of, set_mesh_uv},
    surface_kernel,
};

fn controller(entities: Vec<Entity>) -> Controller {
    let limits = Limits::default();
    let mut document = Document::new("utilities");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, limits).unwrap()
}

#[test]
fn bounds_center_group_and_ungroup_are_undoable() {
    let line = Entity::new(
        "line",
        Geometry::Curve(curve_build::polyline(&[[10.0, 10.0, 0.0], [12.0, 10.0, 0.0]]).unwrap()),
        None,
    );
    let id = line.id;
    let second = Entity::new("second", Geometry::Point([1.0, 2.0, 3.0]), None);
    let second_id = second.id;
    let mut controller = controller(vec![line, second]);
    controller.select_objects(&[id]).unwrap();
    controller.center_selected().unwrap();
    let placement = controller.document().entities[&id].placement;
    assert!((placement.0[0][3] + 11.0).abs() <= 1.0e-9);
    controller.undo().unwrap();
    assert!((controller.document().entities[&id].placement.0[0][3]).abs() <= 1.0e-9);

    controller.select_objects(&[id, second_id]).unwrap();
    controller.group_selection().unwrap();
    assert_eq!(controller.document().groups.len(), 1);
    controller.ungroup_selection().unwrap();
    assert_eq!(controller.document().groups.len(), 0);
}

#[test]
fn mesh_from_nurbs_and_reduce_basic() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("surface", Geometry::Surface(surface), None);
    let id = entity.id;
    let mut controller = controller(vec![entity]);
    controller.select_objects(&[id]).unwrap();
    assert_eq!(controller.mesh_from_nurbs_selected().unwrap(), 1);
    let mesh_id = controller
        .document()
        .entities
        .values()
        .find(|entity| matches!(entity.geometry, Geometry::Mesh(_)))
        .expect("mesh")
        .id;
    controller.select_objects(&[mesh_id]).unwrap();
    let before = match &controller.document().entities[&mesh_id].geometry {
        Geometry::Mesh(mesh) => mesh.faces.len(),
        _ => unreachable!(),
    };
    controller.reduce_mesh_selected().unwrap();
    let after = match &controller.document().entities[&mesh_id].geometry {
        Geometry::Mesh(mesh) => mesh.faces.len(),
        _ => unreachable!(),
    };
    assert!(after <= before / 2);
}

#[test]
fn naked_edges_select_open_meshes() {
    let mesh = Mesh {
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        faces: vec![vec![0, 1, 2]],
        normals: Vec::new(),
        colors: Vec::new(),

        corner_uvs: Vec::new(),
    };
    let entity = Entity::new("open", Geometry::Mesh(mesh), None);
    let mut controller = controller(vec![entity]);
    assert_eq!(controller.health_entries()[0].naked_edges, 3);
    assert!(controller.health_summary().contains("Health:"));
    assert_eq!(controller.select_naked_edges().unwrap(), 1);
    assert_eq!(controller.selected().len(), 1);
}

#[test]
fn mesh_reduce_preserves_uv_and_material_sidecar() {
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let mut entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let uv = MeshUv {
        uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        corners: vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ],
        material: "Gold".into(),
        pbr: PbrMaterial {
            base_color: [1.0, 0.8, 0.2, 1.0],
            metallic: 1.0,
            roughness: 0.25,
            texture: Some("gold.png".into()),
        },
    };
    set_mesh_uv(&mut entity, &uv, &Limits::default()).unwrap();
    let id = entity.id;
    let mut controller = controller(vec![entity]);
    controller.select_objects(&[id]).unwrap();
    controller.reduce_mesh_selected().unwrap();
    let stored = mesh_uv_of(&controller.document().entities[&id], &Limits::default())
        .unwrap()
        .expect("uv preserved");
    assert_eq!(stored.material, "Gold");
    assert_eq!(stored.pbr.metallic, 1.0);
    assert_eq!(stored.pbr.texture.as_deref(), Some("gold.png"));
    assert_eq!(
        stored.uvs.len(),
        match &controller.document().entities[&id].geometry {
            Geometry::Mesh(mesh) => mesh.vertices.len(),
            _ => 0,
        }
    );
}

#[test]
fn checked_repair_error_keeps_all_selected_objects_and_undo_unchanged() {
    let clean = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let crossing = Mesh {
        vertices: vec![
            [-1., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., -1., -1.],
            [0., -1., 1.],
            [0., 1., 0.],
        ],
        faces: vec![vec![0, 1, 2], vec![3, 4, 5]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let a = Entity::new("clean", Geometry::Mesh(clean), None);
    let b = Entity::new("open crossing", Geometry::Mesh(crossing), None);
    let ids = [a.id, b.id];
    let mut controller = controller(vec![a, b]);
    controller.select_objects(&ids).unwrap();
    let before = controller.document().clone();
    let undo_before = controller.coordinator().undo_len();
    let redo_before = controller.coordinator().redo_len();
    assert!(controller.reduce_mesh_selected().is_err());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), undo_before);
    assert_eq!(controller.coordinator().redo_len(), redo_before);
}
