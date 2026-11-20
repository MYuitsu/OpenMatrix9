// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "test-fixtures")]

use om9_core::{brep::BrepTopology, budget::{Limits, Usage}, geometry::{Curve, Transform}};
use om9_kernel::brep::{box_brep, capture_source_topology, capture_source_with_shape_set, sphere_brep, test_restore_captured_shape_set};
use uuid::Uuid;

type FaceProbe = (Uuid, [f64; 3], [f64; 3], [f64; 3], [f64; 3]);
type EdgeProbe = (Uuid, [f64; 3], [f64; 3]);
type VertexProbe = (Uuid, [f64; 3]);
type CircularProbe = (Uuid, Curve);

#[test]
fn registered_source_captures_named_packet_with_budget_and_cancellation() {
    use om9_core::{Error, budget::memory::MemoryClass, geometry::Geometry, model::{Document, Entity},
        transaction::{Mutation, TransactionCoordinator, TransactionRequest, World}};
    use om9_kernel::brep::{RegisteredTopologyProof, SharedTopologyProof};
    let limits = Limits::default();
    let document = Document::new("named source");
    let document_id = document.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let mut creation = coordinator.reserve_external_memory(MemoryClass::CpuCache, 4 * 1024 * 1024, None).unwrap();
    let captured = om9_kernel::creation::primitive_with_topology(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits,
        |live, pending| {
            if live + pending > creation.bytes() { return Err(Error::Budget("fixture creation")); }
            creation.set_allocated_bytes(live)?;
            creation.recheck_pending()
        }).unwrap();
    let (body, proof) = captured.into_parts();
    let proof_bytes = proof.storage_bytes().unwrap();
    let body_bytes = body.storage_usage(&limits).unwrap().bytes;
    creation.set_allocated_bytes(body_bytes + proof_bytes).unwrap();
    creation.shrink_to(body_bytes + proof_bytes).unwrap();
    let proof_reservation = creation.split_off(proof_bytes).unwrap();
    let source = coordinator.register_reserved_brep_candidate(body, &mut creation, Usage::default()).unwrap();
    let entity = Entity::new("box", Geometry::Brep(source), None);
    let id = entity.id;
    let prepared = coordinator.prepare(TransactionRequest::new("insert", vec![Mutation::Insert { document: document_id, entity }])).unwrap();
    coordinator.commit(prepared).unwrap();
    drop(creation);
    let stamp = coordinator.world().documents[&document_id].stamp(id).unwrap();
    let registered = RegisteredTopologyProof::adopt(&coordinator, stamp, proof, proof_reservation, |_, _| Ok(())).unwrap();
    let owner = coordinator.reserve_external_memory(MemoryClass::CpuCache, SharedTopologyProof::allocation_bytes(), None).unwrap();
    let proof = SharedTopologyProof::new(registered, &coordinator, owner).unwrap();
    let source = proof.capture_source(&coordinator).unwrap();
    assert!(matches!(source.capture_named_source(0.1, 0.3, &limits, |_, _| Ok(()), || true), Err(Error::Canceled)));
    assert!(matches!(source.capture_named_source(0.1, 0.3, &limits, |_, _| Err(Error::Budget("fixture refusal")), || false), Err(Error::Budget("fixture refusal"))));
    let mut budget = coordinator.reserve_external_memory(MemoryClass::Kernel, 8 * 1024 * 1024, None).unwrap();
    let named = source.capture_named_source(0.1, 0.3, &limits, |live, pending| {
        if live + pending > budget.bytes() { return Err(Error::Budget("named source fixture")); }
        budget.set_allocated_bytes(live)?;
        budget.recheck_pending()
    }, || false).unwrap();
    assert_eq!(named.source(), source.source());
    assert_eq!(named.topology(), source.topology());
    assert!(budget.bytes() >= named.storage_bytes().unwrap());
    let restored = named.restore(0.05, 0.3, &limits, |live, pending| {
        if live + pending > budget.bytes() { return Err(Error::Budget("named restore fixture")); }
        budget.set_allocated_bytes(live)?;
        budget.recheck_pending()
    }).unwrap();
    assert_eq!(restored.topology(), source.topology());
    proof.verify_unchanged_owner(&coordinator).unwrap();
    let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
        source: stamp, placement: Transform::translation([1., 0., 0.]),
    }])).unwrap();
    coordinator.commit(prepared).unwrap();
    assert!(matches!(proof.verify_unchanged_owner(&coordinator), Err(Error::Stale(_))));
}

#[test]
fn shape_set_preflight_rejects_bad_envelopes_before_native_admission() {
    let limits = Limits::default();
    let seed = box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let source = capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_source();
    assert_eq!((source.topology().faces.len(), source.topology().edges.len(), source.topology().vertex_ids.len()), (6, 12, 8));
    for packet in [
        b"WRONG-SHAPESET 1\n6 12 8\nbody".as_slice(),
        b"OM9-SHAPESET 1\n999999999999999999999999 12 8\nbody".as_slice(),
        b"OM9-SHAPESET 1\n1 2 3\nbody".as_slice(),
        b"OM9-SHAPESET 1\n6 12 8".as_slice(),
        b"OM9-SHAPESET 1\n6 12 8\n".as_slice(),
    ] {
        let mut calls = 0;
        let result = test_restore_captured_shape_set(packet, source.source(), source.topology(), source.storage_bytes().unwrap(),
            0.1, 0.3, &limits, |_, _| { calls += 1; Ok(()) });
        assert!(result.is_err());
        assert_eq!(calls, 0, "bad envelope reached native admission");
    }
    let mut low = limits;
    low.max_bytes = source.shape_set().len() as u64 - 1;
    let mut calls = 0;
    assert!(test_restore_captured_shape_set(source.shape_set(), source.source(), source.topology(), source.storage_bytes().unwrap(),
        0.1, 0.3, &low, |_, _| { calls += 1; Ok(()) }).is_err());
    assert_eq!(calls, 0);
}

#[test]
#[ignore = "child process used by cross-process ShapeSet test"]
fn cross_process_shape_set_child() {
    let path = std::path::PathBuf::from(std::env::var_os("OM9_TEST_SHAPESET_PACKET").unwrap());
    let (source, topology, packet, faces, edges, vertices, circular):
        (String, BrepTopology, Vec<u8>, Vec<FaceProbe>, Vec<EdgeProbe>, Vec<VertexProbe>, Vec<CircularProbe>) =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let limits = Limits::default();
    let held = source.capacity() as u64 + packet.capacity() as u64
        + Usage::array::<Uuid>(topology.faces.capacity() + topology.edges.capacity() + topology.vertex_ids.capacity()).unwrap().bytes
        + Usage::array::<[u32; 2]>(topology.face_edges.capacity()).unwrap().bytes
        + Usage::array::<FaceProbe>(faces.capacity()).unwrap().bytes
        + Usage::array::<EdgeProbe>(edges.capacity()).unwrap().bytes
        + Usage::array::<VertexProbe>(vertices.capacity()).unwrap().bytes
        + Usage::array::<CircularProbe>(circular.capacity()).unwrap().bytes
        + (std::mem::size_of::<BrepTopology>() + std::mem::size_of::<String>()
            + std::mem::size_of::<Vec<u8>>() + std::mem::size_of::<Vec<FaceProbe>>()
            + std::mem::size_of::<Vec<EdgeProbe>>() + std::mem::size_of::<Vec<VertexProbe>>()
            + std::mem::size_of::<Vec<CircularProbe>>()) as u64;
    let ids = |items: &[Uuid]| items.iter().copied().collect::<std::collections::BTreeSet<_>>();
    let adjacency = |topology: &BrepTopology| topology.face_edges.iter().map(|[face, edge]| {
        (topology.faces[*face as usize], topology.edges[*edge as usize])
    }).collect::<std::collections::BTreeSet<_>>();
    for deflection in [0.1, 0.05] {
        let restored = test_restore_captured_shape_set(&packet, &source, &topology, held, deflection, 0.3, &limits, |_, _| Ok(())).unwrap();
        assert_eq!(restored.source(), source);
        assert_eq!(ids(&restored.topology().faces), ids(&topology.faces));
        assert_eq!(ids(&restored.topology().edges), ids(&topology.edges));
        assert_eq!(ids(&restored.topology().vertex_ids), ids(&topology.vertex_ids));
        assert_eq!(adjacency(restored.topology()), adjacency(&topology));
        for (face, origin, direction, reference, expected) in &faces {
            let hit = om9_kernel::measurement::brep_face_ray_hit(&restored, *face, Transform::identity(), *origin, *direction, *reference, 1e-6, &limits).unwrap();
            assert!((0..3).map(|i| (hit[i] - expected[i]).powi(2)).sum::<f64>().sqrt() < 1e-7, "face {face}");
        }
        for (edge, point, expected) in &edges {
            let index = restored.topology().edges.iter().position(|id| id == edge).unwrap();
            let (_, projected) = om9_kernel::editing::fillet_project_edge(&restored, index, *point, &limits).unwrap();
            assert!((0..3).map(|i| (projected[i] - expected[i]).powi(2)).sum::<f64>().sqrt() < 1e-7, "edge {edge}");
        }
        let source_vertices = om9_kernel::brep::source_vertices(&restored, &limits).unwrap();
        for (vertex, expected) in &vertices {
            let index = restored.topology().vertex_ids.iter().position(|id| id == vertex).unwrap();
            assert!((0..3).map(|i| (source_vertices[index][i] - expected[i]).powi(2)).sum::<f64>().sqrt() < 1e-7, "vertex {vertex}");
        }
        for (edge, expected) in &circular {
            let index = restored.topology().edges.iter().position(|id| id == edge).unwrap();
            assert_eq!(&om9_kernel::editing::circular_edge(&restored, index, &limits).unwrap(), expected, "circular edge {edge}");
        }
    }
    let mut damaged = packet.clone();
    damaged[0] ^= 1;
    assert!(test_restore_captured_shape_set(&damaged, &source, &topology, held + damaged.capacity() as u64,
        0.1, 0.3, &limits, |_, _| Ok(())).is_err());
    let other = box_brep([2., 3., 7.], 0.1, &limits).unwrap();
    assert!(test_restore_captured_shape_set(&packet, other.source(), &topology,
        held + other.storage_usage(&limits).unwrap().bytes, 0.1, 0.3, &limits, |_, _| Ok(())).is_err());
}

#[test]
fn captured_shape_set_rebinds_topology_in_fresh_process() {
    let limits = Limits::default();
    let dir = std::env::temp_dir().join(format!("om9-shapeset-process-{}", Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let cylinder = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.1, &limits).unwrap();
    for (index, seed) in [box_brep([4., 5., 6.], 0.1, &limits).unwrap(),
        sphere_brep(3., 0.1, &limits).unwrap(), cylinder].into_iter().enumerate() {
        let (original, proof) = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_parts();
        let mut faces: Vec<FaceProbe> = Vec::new();
        for (face_index, face) in original.topology().faces.iter().enumerate() {
            let triangle = original.display().triangles.iter().find(|triangle| triangle[3] as usize == face_index).unwrap();
            let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|i| original.display().points[i as usize]);
            let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
            let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
            let normal = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            assert!(length > 0.);
            let direction = normal.map(|value| -value / length);
            let reference = std::array::from_fn(|i| (a[i] + b[i] + c[i]) / 3.);
            let origin = std::array::from_fn(|i| reference[i] - direction[i]);
            let expected = om9_kernel::measurement::brep_face_ray_hit(&original, *face, Transform::identity(), origin, direction, reference, 1e-6, &limits).unwrap();
            if index == 0 {
                let bounds = [4., 5., 6.];
                assert!((0..3).all(|i| expected[i] >= -1e-7 && expected[i] <= bounds[i] + 1e-7));
                assert!((0..3).any(|i| expected[i].abs() < 1e-7 || (expected[i] - bounds[i]).abs() < 1e-7));
            } else if index == 1 {
                assert!((expected[0].hypot(expected[1]).hypot(expected[2]) - 3.).abs() < 1e-7);
            } else {
                assert!(expected[2] >= -1e-7 && expected[2] <= 5. + 1e-7);
                assert!(expected[0].hypot(expected[1]) <= 2. + 1e-7);
                assert!((expected[0].hypot(expected[1]) - 2.).abs() < 1e-7 || expected[2].abs() < 1e-7 || (expected[2] - 5.).abs() < 1e-7);
            }
            faces.push((*face, origin, direction, reference, expected));
        }
        let mut edges: Vec<EdgeProbe> = Vec::new();
        if index == 0 {
            for (edge_index, edge) in original.topology().edges.iter().enumerate() {
                let segment = original.display().segments.iter().find(|segment| segment[2] as usize == edge_index).unwrap();
                let [a, b] = [segment[0], segment[1]].map(|i| original.display().points[i as usize]);
                let point = std::array::from_fn(|i| (a[i] + b[i]) / 2.);
                let (_, expected) = om9_kernel::editing::fillet_project_edge(&original, edge_index, point, &limits).unwrap();
                let bounds = [4., 5., 6.];
                assert!((0..3).all(|i| expected[i] >= -1e-7 && expected[i] <= bounds[i] + 1e-7));
                assert!((0..3).filter(|&i| expected[i].abs() < 1e-7 || (expected[i] - bounds[i]).abs() < 1e-7).count() >= 2);
                edges.push((*edge, point, expected));
            }
        }
        let points = om9_kernel::brep::source_vertices(&original, &limits).unwrap();
        let vertices: Vec<VertexProbe> = original.topology().vertex_ids.iter().copied().zip(points).collect();
        assert!(!vertices.is_empty());
        for (_, point) in &vertices {
            if index == 0 {
                let bounds = [4., 5., 6.];
                assert!((0..3).all(|i| point[i].abs() < 1e-7 || (point[i] - bounds[i]).abs() < 1e-7));
            } else if index == 1 {
                assert!((point[0].hypot(point[1]).hypot(point[2]) - 3.).abs() < 1e-7);
            } else {
                assert!((point[0].hypot(point[1]) - 2.).abs() < 1e-7);
                assert!(point[2].abs() < 1e-7 || (point[2] - 5.).abs() < 1e-7);
            }
        }
        let mut circular: Vec<CircularProbe> = Vec::new();
        if index == 2 {
            for (edge_index, edge) in original.topology().edges.iter().enumerate() {
                if let Ok(curve) = om9_kernel::editing::circular_edge(&original, edge_index, &limits) {
                    let Curve::Circle { center, radius, .. } = &curve else { panic!("expected circular edge") };
                    assert!((*radius - 2.).abs() < 1e-7 && center[0].abs() < 1e-7 && center[1].abs() < 1e-7);
                    assert!(center[2].abs() < 1e-7 || (center[2] - 5.).abs() < 1e-7);
                    circular.push((*edge, curve));
                }
            }
            assert_eq!(circular.len(), 2, "cylinder must provide two circular edges");
        }
        let named = capture_source_with_shape_set(original.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap()
            .into_rebound_for_brep(&original, &proof, &limits, |_, _| Ok(())).unwrap();
        let ids = |items: &[Uuid]| items.iter().copied().collect::<std::collections::BTreeSet<_>>();
        let original_topology = original.topology();
        let rebound = named.snapshot().brep().topology();
        assert_eq!(ids(&rebound.faces), ids(&original_topology.faces));
        assert_eq!(ids(&rebound.edges), ids(&original_topology.edges));
        assert_eq!(ids(&rebound.vertex_ids), ids(&original_topology.vertex_ids));
        let adjacency = |topology: &BrepTopology| topology.face_edges.iter().map(|[face, edge]| {
            (topology.faces[*face as usize], topology.edges[*edge as usize])
        }).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(adjacency(rebound), adjacency(original_topology));
        let source = named.into_source();
        let path = dir.join(format!("{index}.json"));
        let data = serde_json::to_vec(&(source.source(), source.topology(), source.shape_set(), faces, edges, vertices, circular)).unwrap();
        std::fs::write(&path, data).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child.args(["--ignored", "--exact", "cross_process_shape_set_child"])
            .env("OM9_TEST_SHAPESET_PACKET", &path);
        #[cfg(windows)]
        { use std::os::windows::process::CommandExt; child.creation_flags(0x0800_0000); }
        let status = child.status().unwrap();
        assert!(status.success(), "fresh process failed for fixture {index}: {status}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}
