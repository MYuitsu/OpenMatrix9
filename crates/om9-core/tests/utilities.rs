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
fn reduce_mesh_preserves_closed_components_and_open_boundaries() {
    use om9_core::mesh_repair::reduce_mesh_feature_aware;
    let tetra = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        faces: vec![vec![0, 2, 1], vec![0, 1, 3], vec![1, 2, 3], vec![2, 0, 3]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    for angle in [30., 180.] {
        let (reduced, report) = reduce_mesh_feature_aware(&tetra, 1, true, angle).unwrap();
        assert_eq!(report.faces_after, 4);
        assert_eq!(reduced.faces.len(), 4);
        assert!((om9_core::cutters::volume_mm3(&reduced).unwrap() - 1. / 6.).abs() < 1e-10);
    }
    let octahedron = Mesh {
        vertices: vec![
            [1., 0., 0.],
            [0., 1., 0.],
            [-1., 0., 0.],
            [0., -1., 0.],
            [0., 0., 1.],
            [0., 0., -1.],
        ],
        faces: (0..4)
            .flat_map(|i| [vec![i, (i + 1) % 4, 4], vec![(i + 1) % 4, i, 5]])
            .collect(),
        ..tetra.clone()
    };
    let (coarse, _) = reduce_mesh_feature_aware(&octahedron, 6, true, 180.).unwrap();
    assert_eq!(coarse.faces.len(), 6);
    assert!(om9_core::cutters::volume_mm3(&coarse).unwrap() > 0.);
    let limits = Limits::default();
    let patch =
        surface_kernel::plane([0.; 3], [1., 0., 0.], [0., 1., 0.], 4., 4., &limits).unwrap();
    let grid = mesh_from_surface(&patch, 3, 3, &limits).unwrap();
    let (reduced, _) = reduce_mesh_feature_aware(&grid, 6, true, 30.).unwrap();
    assert!(reduced.faces.len() <= 6);
    let mut area = 0.0;
    for face in &reduced.faces {
        let [a, b, c] = [face[0], face[1], face[2]].map(|v| reduced.vertices[v as usize]);
        let triangle = ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) / 2.;
        assert!(triangle > 0.);
        area += triangle;
    }
    assert!((area - 16.).abs() < 1e-9, "area {area}");
}

#[test]
fn reduce_document_mesh_preserves_quad_uv_seams_and_vertex_channels() {
    use om9_core::mesh_repair::reduction::{Options, reduce, triangle_count};
    use om9_core::rhino_semantics::MeshUv;
    let mut mesh = Mesh {
        vertices: (0..=4)
            .flat_map(|x| (0..=4).map(move |y| [x as f64, y as f64, 0.]))
            .collect(),
        faces: Vec::new(),
        normals: vec![[0., 0., 1.]; 25],
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    mesh.colors = mesh
        .vertices
        .iter()
        .map(|p| [(40. * p[0]) as u8, (40. * p[1]) as u8, 80, 255])
        .collect();
    for x in 0..4 {
        for y in 0..4 {
            let i = x * 5 + y;
            let quad = vec![i, i + 5, i + 6, i + 1];
            for &id in &quad {
                let [px, py, _] = mesh.vertices[id as usize];
                mesh.corner_uvs
                    .push([px + if x >= 2 { 10. } else { 0. }, py]);
            }
            mesh.faces.push(quad);
        }
    }
    let uv = MeshUv {
        uvs: mesh.vertices.iter().map(|p| [p[0], p[1]]).collect(),
        corners: mesh.corner_uvs.iter().map(|p| [p[0] + 50., p[1]]).collect(),
        material: "test texture".into(),
        pbr: Default::default(),
    };
    for accuracy in [1, 5, 10] {
        let options = Options {
            triangles: 17,
            feature_angle: 0.,
            accuracy,
        };
        let (out, mapped) = reduce(
            &mesh,
            Some(&uv),
            options,
            &Limits::default(),
            &Default::default(),
        )
        .unwrap();
        let mapped = mapped.unwrap();
        assert!(triangle_count(&out) < 32 && out.faces.iter().any(|f| f.len() == 4));
        assert_eq!(mapped.material, uv.material);
        let mut offset = 0;
        for face in &out.faces {
            let right = face.iter().any(|i| out.vertices[*i as usize][0] > 2.);
            for &id in face {
                let i = id as usize;
                let [x, y, _] = out.vertices[i];
                let u = x + if right { 10. } else { 0. };
                assert_eq!(out.corner_uvs[offset], [u, y]);
                assert_eq!(mapped.corners[offset], [u + 50., y]);
                assert_eq!(mapped.uvs[i], [x, y]);
                assert_eq!(
                    out.colors[i],
                    [(40. * x).round() as u8, (40. * y).round() as u8, 80, 255]
                );
                assert_eq!(out.normals[i], [0., 0., 1.]);
                offset += 1;
            }
        }
    }
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
