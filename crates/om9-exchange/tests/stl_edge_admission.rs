// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_exchange::stl::{try_edge_topology_with_admission, TriangleSoup};

#[test]
fn edge_growth_refuses_when_only_arrays_and_face_table_fit() {
    let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
    let mut face_table = std::collections::HashSet::<[u32; 3]>::new();
    face_table.try_reserve(1).unwrap();
    let arrays = std::mem::size_of::<usize>() + std::mem::size_of::<u8>();
    let face_bytes = face_table.capacity() * std::mem::size_of::<[u32; 3]>();
    let limit = (arrays + face_bytes) as u64;
    let mut largest_accepted = 0;
    let result = try_edge_topology_with_admission(&soup, || Ok::<(), &'static str>(()), || "allocation", |bytes| {
        if bytes > limit { Err("budget") } else { largest_accepted = largest_accepted.max(bytes); Ok(()) }
    });
    assert_eq!(result, Err("budget"));
    assert_eq!(largest_accepted, limit, "the face table fit before the edge table was requested");
}

#[test]
fn admitted_tetrahedron_keeps_closed_index_topology() {
    let soup = TriangleSoup::new_f64(
        vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,1.]],
        vec![[0,2,1], [0,1,3], [1,2,3], [2,0,3]],
    );
    let mut checks = 0usize;
    let topology = try_edge_topology_with_admission(&soup, || Ok::<(), &'static str>(()), || "allocation", |_| {
        checks += 1;
        Ok(())
    }).unwrap();
    assert!(checks > 0);
    assert_eq!((topology.boundary, topology.non_manifold, topology.inconsistent_winding), (0, 0, 0));
    assert_eq!((topology.disconnected_vertices, topology.edge_connected_components, topology.duplicate_faces), (0, 1, 0));
}

#[test]
fn fan_links_are_refused_when_edge_stage_fits_but_fan_growth_does_not() {
    let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]; 100]);
    let mut peak = 0u64;
    let result = try_edge_topology_with_admission(&soup, || Ok::<(), &'static str>(()), || "allocation", |bytes| {
        peak = peak.max(bytes);
        if bytes > 1500 { Err("budget") } else { Ok(()) }
    });
    assert_eq!(result, Err("budget"));
    assert!(peak > 1500);
    let topology = try_edge_topology_with_admission(&soup, || Ok::<(), &'static str>(()), || "allocation", |_| Ok(())).unwrap();
    assert_eq!(topology.duplicate_faces, 99);
    assert_eq!(topology.non_manifold, 3);
    assert_eq!(topology.edge_connected_components, 1);
}

#[test]
fn fan_adjacency_is_refused_when_links_fit_but_adjacency_grows() {
    let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]; 100]);
    let limit = 4000;
    let result = try_edge_topology_with_admission(&soup, || Ok::<(), &'static str>(()), || "allocation", |bytes| {
        if bytes > limit { Err("budget") } else { Ok(()) }
    });
    assert_eq!(result, Err("budget"));
}

#[test]
fn stl_writer_refuses_unadmitted_quantized_position_table() {
    use om9_exchange::stl::{write_binary_stl_to_with_report_and_admission, StlError, StlOptions, StlStreamError};
    let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
    let mut output = Vec::new();
    let mut requested = 0;
    let result = write_binary_stl_to_with_report_and_admission(&mut output, &soup, StlOptions { strict: false, ..Default::default() }, Some(0.001), || Ok(()), |bytes| {
        requested = bytes;
        Err(StlError::Allocation)
    });
    assert!(matches!(result, Err(StlStreamError::Geometry(StlError::Allocation))));
    assert!(requested > 0, "the encoder must ask before allocating its first table");
    output.clear();
    let report = write_binary_stl_to_with_report_and_admission(&mut output, &soup, StlOptions { strict: false, ..Default::default() }, Some(0.001), || Ok(()), |_| Ok(())).unwrap();
    assert!(!report.coincident_distinct_indices);
    assert_eq!(output.len(), 134);
}
