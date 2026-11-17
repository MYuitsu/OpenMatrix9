// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    associative::output_hash,
    brep::{Brep, BrepDisplay, BrepTopology},
    budget::Limits,
    geometry::{Geometry, Mesh, NativeBrep},
    model::{Document, Entity},
};
use uuid::Uuid;

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

fn hash(geometry: Geometry) -> [u8; 32] {
    let mut document = Document::new("hash");
    let entity = Entity::new("output", geometry, None);
    let id = entity.id;
    document.entities.insert(id, entity);
    output_hash(&document, &[id]).unwrap()
}

#[test]
fn brep_hash_excludes_display_but_includes_source_and_topology() {
    let original = shape("CASCADE Topology A", Uuid::from_u128(1));
    let expected = hash(Geometry::Brep(original.clone()));
    let mut remeshed = original.clone();
    remeshed.edit_display_for_validation(|display| {
        display.deflection_mm = 0.02;
        display.points[1][0] = 2.;
    });
    assert_eq!(expected, hash(Geometry::Brep(remeshed)));
    assert_ne!(expected, hash(Geometry::Brep(shape("CASCADE Topology B", Uuid::from_u128(1)))));
    assert_ne!(expected, hash(Geometry::Brep(shape("CASCADE Topology A", Uuid::from_u128(2)))));
}

#[test]
fn native_hash_keeps_payload_and_authored_mesh_keeps_vertices() {
    let native = NativeBrep { display: mesh().into(), payload: vec![1, 2, 3].into(), crc: 17 };
    let expected = hash(Geometry::NativeBrep(native.clone()));
    let mut remeshed = native.clone();
    std::sync::Arc::make_mut(&mut remeshed.display).vertices[1][0] = 2.;
    assert_eq!(expected, hash(Geometry::NativeBrep(remeshed)));
    let mut changed = native.clone();
    std::sync::Arc::make_mut(&mut changed.payload)[0] = 4;
    assert_ne!(expected, hash(Geometry::NativeBrep(changed)));
    let mut changed = native;
    changed.crc += 1;
    assert_ne!(expected, hash(Geometry::NativeBrep(changed)));
    let original = mesh();
    let mut edited = original.clone();
    edited.vertices[1][0] = 2.;
    assert_ne!(hash(Geometry::Mesh(original)), hash(Geometry::Mesh(edited)));
}
