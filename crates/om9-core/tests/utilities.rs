// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh, Point3},
    mesh_repair::{
        feature_edges, reduce_mesh_advanced, repair_mesh, repair_self_intersections,
        self_intersections,
    },
    surface_kernel,
    utilities::{bounds, mesh_from_surface, reduce_mesh},
};

#[test]
fn bounds_and_surface_mesh_are_available() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        20.0,
        &limits,
    )
    .unwrap();
    let bounds = bounds(&Geometry::Surface(surface.clone())).expect("surface bounds");
    assert!((bounds.size()[0] - 10.0).abs() <= 1.0e-6);
    assert!((bounds.size()[1] - 20.0).abs() <= 1.0e-6);
    let mesh = mesh_from_surface(&surface, 4, 4, &limits).expect("surface mesh");
    assert_eq!(mesh.vertices.len(), 25);
    assert_eq!(mesh.faces.len(), 32);
}

#[test]
fn basic_mesh_reduction_drops_faces_and_merges_vertices() {
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 2, 3], vec![1, 4, 5], vec![1, 5, 2]],
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let reduced = reduce_mesh(&mesh, 2).expect("reduce");
    assert!(reduced.faces.len() <= 2);
    assert!(reduced.vertices.len() <= mesh.vertices.len());
    let _: Point3 = reduced.vertices[0];
}

#[test]
fn advanced_repair_removes_degenerate_duplicate_and_isolated_data() {
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [9.0, 9.0, 9.0],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 1], vec![0, 2, 3]],
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let (repaired, report) = repair_mesh(&mesh, 1.0e-9).expect("repair");
    assert_eq!(repaired.faces.len(), 2);
    assert!(report.duplicate_faces >= 1);
    assert!(report.degenerate_faces >= 1);
    assert!(report.isolated_vertices >= 1);
}

#[test]
fn advanced_reduce_uses_qem_edge_collapse() {
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
    let mesh = mesh_from_surface(&surface, 6, 6, &limits).unwrap();
    let (reduced, _) = reduce_mesh_advanced(&mesh, 20, true).expect("advanced reduce");
    assert!(reduced.faces.len() <= 20);
}

#[test]
fn self_intersection_detection_and_repair_refuse_open_crossing_faces() {
    let mesh = Mesh {
        vertices: vec![
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, -1.0],
            [0.0, -1.0, 1.0],
            [0.0, 1.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![3, 4, 5]],
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    assert!(!self_intersections(&mesh, 16).is_empty());
    let before = mesh.clone();
    assert!(repair_self_intersections(&mesh, om9_core::mesh_csg::CsgOptions::default()).is_err());
    assert_eq!(mesh, before);
}

#[test]
fn dihedral_feature_edges_are_detected() {
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 1, 3]],
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let features = feature_edges(&mesh, 30.0);
    assert!(features.contains(&(0, 1)));
}
