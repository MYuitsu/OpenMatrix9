// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-12-009 acceptance: the public STL export policy states millimetre
//! scale and winding in the file itself, and refuses geometry a slicer would
//! silently misread.
//!
//! The real document-to-mesh STL command is M07 (`OM9-L3-01-12-008` is deferred
//! to M07); this test only pins the codec policy the command will call.

use om9_exchange::stl::{StlError, StlOptions, TriangleSoup, open_edges, write_binary_stl};

#[test]
fn edge_checks_reject_even_non_manifold_incidence_and_same_direction() {
    use om9_exchange::stl::edge_topology;
    let mut mesh = cube();
    mesh.triangles.extend([mesh.triangles[0], mesh.triangles[0]]);
    let report = edge_topology(&mesh);
    assert_eq!(report.boundary, 0);
    assert_eq!(report.non_manifold, 3);
    assert!(write_binary_stl(&mesh, StlOptions::default()).is_err());
    let mut mesh = cube();
    mesh.triangles[0].swap(1, 2);
    let report = edge_topology(&mesh);
    assert_eq!(report.boundary, 0);
    assert_eq!(report.inconsistent_winding, 3);
    assert!(write_binary_stl(&mesh, StlOptions::default()).is_err());
}

#[test]
fn edge_closed_tetrahedra_touching_at_one_vertex_have_disconnected_fans() {
    use om9_exchange::stl::edge_topology;
    let soup = TriangleSoup::new(
        vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,1.], [-1.,0.,0.], [0.,-1.,0.], [0.,0.,-1.]],
        vec![[0,2,1], [0,1,3], [1,2,3], [2,0,3], [0,5,4], [0,4,6], [4,5,6], [5,0,6]],
    );
    let report = edge_topology(&soup);
    assert_eq!((report.boundary, report.non_manifold, report.inconsistent_winding), (0, 0, 0));
    assert_eq!(report.disconnected_vertices, 1);
    assert_eq!(report.edge_connected_components, 2);
    assert!(write_binary_stl(&soup, StlOptions::default()).is_err());
}

#[test]
fn triangle_components_use_shared_edges_even_for_non_manifold_incidence() {
    let soup = TriangleSoup::new(
        vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,-1.,0.], [0.,0.,1.], [2.,0.,0.], [2.,1.,0.]],
        vec![[0,1,2], [1,0,3], [0,1,4], [2,5,6]],
    );
    let report = om9_exchange::stl::edge_topology(&soup);
    assert_eq!(report.edge_connected_components, 2);
    assert_eq!(report.non_manifold, 1);
    assert_eq!(om9_exchange::stl::edge_topology(&TriangleSoup::new(vec![], vec![])).edge_connected_components, 0);
    assert_eq!(om9_exchange::stl::edge_topology(&cube()).edge_connected_components, 1);
}

#[test]
fn writer_reports_distinct_indices_at_same_encoded_position() {
    use om9_exchange::stl::write_binary_stl_to_with_report;
    let options = StlOptions { strict: false, ..StlOptions::default() };
    let mut encoded = Vec::new();
    let report = write_binary_stl_to_with_report(&mut encoded, &cube(), options, None, || Ok(())).unwrap();
    assert!(!report.coincident_distinct_indices, "repeated use of the same indices is unambiguous");
    let soup = TriangleSoup::new(
        vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,-0.], [1.,0.,0.], [0.,1.,0.]],
        vec![[0,1,2], [3,4,5]],
    );
    encoded.clear();
    let report = write_binary_stl_to_with_report(&mut encoded, &soup, options, None, || Ok(())).unwrap();
    assert!(report.coincident_distinct_indices);
    assert_eq!(encoded.len(), 84 + 2 * 50);
}

#[test]
fn topology_cancellation_is_checked_during_both_passes() {
    let soup = TriangleSoup::new(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
    // For one triangle: initial, edge construction, edge traversal, then fan construction.
    for stop_at in [1, 2, 3, 4, 5] {
        let mut calls = 0;
        let result = om9_exchange::stl::try_edge_topology_with_check(&soup, || {
            calls += 1;
            if calls == stop_at { Err("canceled") } else { Ok(()) }
        }, || "allocation");
        assert_eq!(result, Err("canceled"));
        assert_eq!(calls, stop_at);
    }
}

#[test]
fn coincident_opposite_triangles_are_duplicate_faces_not_a_closed_solid() {
    let soup = TriangleSoup::new(
        vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        vec![[0, 1, 2], [0, 2, 1]],
    );
    let report = om9_exchange::stl::edge_topology(&soup);
    assert_eq!((report.boundary, report.non_manifold, report.inconsistent_winding, report.disconnected_vertices), (0, 0, 0, 0));
    assert_eq!(report.duplicate_faces, 1);
    assert!(matches!(write_binary_stl(&soup, StlOptions::default()), Err(StlError::InvalidTopology { duplicate_faces: 1, .. })));
}

#[test]
fn f64_input_checks_quantization_distance_and_triangle_collapse() {
    use om9_exchange::stl::write_binary_stl_with_tolerance;
    let open = StlOptions { strict: false, ..StlOptions::default() };
    let accurate = TriangleSoup::new_f64(vec![[0.0000001, 0., 0.], [1., 0., 0.], [0., 1., 0.]], vec![[0, 1, 2]]);
    assert!(write_binary_stl_with_tolerance(&accurate, open, Some(0.0001)).is_ok());
    let far = TriangleSoup::new_f64(vec![[100000.003, 0., 0.], [100001.003, 0., 0.], [100000.003, 1., 0.]], vec![[0, 1, 2]]);
    assert!(matches!(write_binary_stl_with_tolerance(&far, open, Some(0.0001)), Err(StlError::Quantization { .. })));
    let collapsed = TriangleSoup::new_f64(vec![[100000., 0., 0.], [100000.001, 0., 0.], [100000., 1., 0.]], vec![[0, 1, 2]]);
    assert!(matches!(write_binary_stl_with_tolerance(&collapsed, open, None), Err(StlError::QuantizedTriangle { .. })));
}

/// A closed 10 mm cube, wound counter-clockwise seen from outside.
fn cube() -> TriangleSoup {
    let positions = vec![
        [-5., -5., -5.],
        [5., -5., -5.],
        [5., 5., -5.],
        [-5., 5., -5.],
        [-5., -5., 5.],
        [5., -5., 5.],
        [5., 5., 5.],
        [-5., 5., 5.],
    ];
    let triangles = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [2, 3, 7],
        [2, 7, 6],
        [1, 2, 6],
        [1, 6, 5],
        [0, 4, 7],
        [0, 7, 3],
    ];
    TriangleSoup::new(positions, triangles)
}

type Triangle = ([f32; 3], [[f32; 3]; 3]);

fn parsed(bytes: &[u8]) -> (String, Vec<Triangle>) {
    let header = String::from_utf8_lossy(&bytes[..80])
        .trim_end_matches('\0')
        .to_string();
    let count = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
    let mut triangles = Vec::with_capacity(count);
    for index in 0..count {
        let start = 84 + index * 50;
        let float = |offset: usize| {
            f32::from_le_bytes(
                bytes[start + offset..start + offset + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        triangles.push((
            [float(0), float(4), float(8)],
            [
                [float(12), float(16), float(20)],
                [float(24), float(28), float(32)],
                [float(36), float(40), float(44)],
            ],
        ));
    }
    (header, triangles)
}

#[test]
fn the_file_header_states_the_scale_so_units_are_never_implicit() {
    let millimetres = write_binary_stl(&cube(), StlOptions::default()).unwrap();
    let (header, triangles) = parsed(&millimetres);
    assert!(header.contains("scale 1 mm-per-unit"), "{header}");
    assert_eq!(triangles.len(), 12);
    // The cube spans 10 mm; every coordinate must be the authored millimetre value.
    assert!(triangles.iter().all(|(_, vertices)| {
        vertices
            .iter()
            .all(|vertex| vertex.iter().all(|value| value.abs() <= 5.0 + 1e-6))
    }));

    let inches = write_binary_stl(
        &cube(),
        StlOptions {
            scale: 1.0 / 25.4,
            ..StlOptions::default()
        },
    )
    .unwrap();
    let (header, triangles) = parsed(&inches);
    assert!(header.contains("mm-per-unit"), "{header}");
    let smallest = triangles
        .iter()
        .flat_map(|(_, vertices)| vertices.iter())
        .flat_map(|vertex| vertex.iter())
        .fold(0.0f32, |value, coordinate| value.max(coordinate.abs()));
    assert!((smallest - (5.0f64 / 25.4) as f32).abs() < 1e-6);
}

#[test]
fn a_closed_surface_is_watertight_and_its_normals_point_outwards() {
    let soup = cube();
    assert_eq!(open_edges(&soup), 0);
    let bytes = write_binary_stl(&soup, StlOptions::default()).unwrap();
    let (_, triangles) = parsed(&bytes);
    for (normal, _) in &triangles {
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        assert!(
            (length - 1.0).abs() < 1e-5,
            "normals must be unit: {length}"
        );
    }
    // The +Z face must point up; a reversed normal would tell a slicer the solid
    // is inside out.
    let top = triangles
        .iter()
        .find(|(_, vertices)| vertices.iter().all(|vertex| vertex[2] > 4.9))
        .expect("top face");
    assert!(top.0[2] > 0.99, "{:?}", top.0);

    let flipped = write_binary_stl(
        &soup,
        StlOptions {
            flip_winding: true,
            ..StlOptions::default()
        },
    )
    .unwrap();
    let (_, flipped_triangles) = parsed(&flipped);
    let a = triangles[0].0;
    let b = flipped_triangles[0].0;
    assert!((a[0] + b[0]).abs() < 1e-5);
    assert!((a[1] + b[1]).abs() < 1e-5);
    assert!((a[2] + b[2]).abs() < 1e-5);
}

#[test]
fn geometry_a_slicer_would_misread_is_refused_before_a_file_exists() {
    let mut open = cube();
    open.triangles.truncate(10);
    assert_eq!(open_edges(&open), 4);
    assert_eq!(
        write_binary_stl(&open, StlOptions::default()),
        Err(StlError::OpenSurface { open_edges: 4 })
    );
    // Explicit lenient export still writes the caller's file.
    assert_eq!(
        write_binary_stl(
            &open,
            StlOptions {
                strict: false,
                ..StlOptions::default()
            }
        )
        .unwrap()
        .len(),
        84 + 10 * 50
    );
    assert_eq!(
        write_binary_stl(
            &cube(),
            StlOptions {
                scale: f64::NAN,
                ..StlOptions::default()
            }
        ),
        Err(StlError::BadScale)
    );
    let nonfinite = TriangleSoup::new(
        vec![[f32::INFINITY, 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        vec![[0, 1, 2]],
    );
    assert_eq!(
        write_binary_stl(
            &nonfinite,
            StlOptions {
                strict: false,
                ..StlOptions::default()
            }
        ),
        Err(StlError::NonFinite { vertex: 0 })
    );
}
