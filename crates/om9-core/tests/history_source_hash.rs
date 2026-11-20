// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    associative::output_hash,
    brep::{Brep, BrepDisplay, BrepTopology},
    budget::{Limits, Usage, memory::MemoryClass},
    geometry::{Curve, Geometry, Mesh, NativeBrep},
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use uuid::Uuid;
use sha2::{Digest, Sha256};

fn mesh() -> Mesh {
    Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![] }
}

fn shape(source: &str, face: Uuid) -> Brep {
    Brep::from_native(source.to_owned(), BrepDisplay { source_sha256: [0; 32], deflection_mm: 0.1,
        points: mesh().vertices, triangles: vec![[0, 1, 2, 0]], segments: vec![], vertices: vec![],
        topology: BrepTopology { faces: vec![face], edges: vec![], vertex_ids: vec![], face_edges: vec![] }.into(),
    }, &Limits::default()).unwrap()
}

fn canonical(brep: Brep) -> Geometry {
    Geometry::Brep(brep.into_canonical_and_representation().0)
}

fn native_source(brep: NativeBrep) -> Geometry {
    Geometry::NativeBrep(brep.into_source_and_representation().0)
}

fn hash(geometry: Geometry) -> [u8; 32] {
    let mut document = Document::new("hash");
    let entity = Entity::new("output", geometry, None);
    let id = entity.id;
    document.entities.insert(id, entity);
    output_hash(&document, &[id]).unwrap()
}

#[test]
fn streamed_hash_matches_the_previous_length_prefixed_json_format() {
    let mut document = Document::new("streamed hashes");
    let mut ids = Vec::new();
    for geometry in [
        Geometry::Curve(Curve::Polycurve { segments: vec![
            Curve::Line { start: [-1.25, 0., 0.], end: [0., 0., 0.], domain: [0., 1.] },
            Curve::Line { start: [0., 0., 0.], end: [2., 3., 4.], domain: [0., 1.] },
        ], reversed: false }),
        Geometry::Mesh(mesh()),
    ] {
        let entity = Entity::new("output", geometry, None);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut legacy = Sha256::new();
    for id in &ids {
        let bytes = serde_json::to_vec(&document.entities[id].geometry).unwrap();
        legacy.update((bytes.len() as u64).to_le_bytes());
        legacy.update(bytes);
    }
    let expected: [u8; 32] = legacy.finalize().into();
    assert_eq!(output_hash(&document, &ids).unwrap(), expected);
}

#[test]
fn preparation_peer_retains_evidence_after_primary_history_clears() {
    let limits = Limits::default();
    let doc = Document::new("peer evidence");
    let id = doc.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let entity = Entity::new("body", canonical(shape("CASCADE Topology A", Uuid::from_u128(1))), None);
    let object = entity.id;
    let transaction = coordinator.prepare(TransactionRequest::new("insert", vec![Mutation::Insert { document: id, entity }])).unwrap();
    coordinator.commit(transaction).unwrap();
    let stamp = coordinator.world().documents[&id].stamp(object).unwrap();
    let owner = std::sync::Arc::new(vec![0u8; 4096]);
    let weak = std::sync::Arc::downgrade(&owner);
    coordinator.retain_insert_evidence(&stamp, owner, 4096, Usage::default()).unwrap();
    let peer = coordinator.capture_transaction_preparation_peer(Usage::default()).unwrap();
    assert!(coordinator.external_pending_bytes(MemoryClass::CpuCache) >= 4096);
    let mut request = TransactionRequest::new("delete", vec![Mutation::Delete { source: stamp }]);
    request.recording = false;
    coordinator.commit(coordinator.prepare(request).unwrap()).unwrap();
    assert_eq!(coordinator.undo_len(), 0);
    assert!(weak.upgrade().is_some());
    drop(peer);
    assert!(weak.upgrade().is_none());
}

#[test]
fn brep_hash_excludes_display_but_includes_source_and_topology() {
    let original = shape("CASCADE Topology A", Uuid::from_u128(1));
    let expected = hash(canonical(original.clone()));
    let mut remeshed = original.clone();
    remeshed.edit_display_for_validation(|display| {
        display.deflection_mm = 0.02;
        display.points[1][0] = 2.;
    });
    assert_eq!(expected, hash(canonical(remeshed)));
    assert_ne!(expected, hash(canonical(shape("CASCADE Topology B", Uuid::from_u128(1)))));
    assert_ne!(expected, hash(canonical(shape("CASCADE Topology A", Uuid::from_u128(2)))));
}

#[test]
fn native_hash_keeps_payload_and_authored_mesh_keeps_vertices() {
    let native = NativeBrep { display: mesh().into(), payload: vec![1, 2, 3].into(), crc: 17 };
    let expected = hash(native_source(native.clone()));
    let mut remeshed = native.clone();
    std::sync::Arc::make_mut(&mut remeshed.display).vertices[1][0] = 2.;
    assert_eq!(expected, hash(native_source(remeshed)));
    let mut changed = native.clone();
    std::sync::Arc::make_mut(&mut changed.payload)[0] = 4;
    assert_ne!(expected, hash(native_source(changed)));
    let mut changed = native;
    changed.crc += 1;
    assert_ne!(expected, hash(native_source(changed)));
    let original = mesh();
    let mut edited = original.clone();
    edited.vertices[1][0] = 2.;
    assert_ne!(hash(Geometry::Mesh(original)), hash(Geometry::Mesh(edited)));
}
