// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits, geometry::{Geometry, Hatch, Mesh}};

#[test]
fn hatch_accounts_actual_display_mesh_capacity() {
    let limits = Limits::default();
    let mut vertices = Vec::with_capacity(4096);
    vertices.extend([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
    let mesh = Geometry::Mesh(Mesh {
        vertices, faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    });
    let mesh_usage = mesh.validate(&limits).unwrap();
    let Geometry::Mesh(display) = mesh else { unreachable!() };
    let hatch = Geometry::Hatch(Hatch { display, loops: vec![], pattern: None, color: [255; 4] });
    let usage = hatch.validate(&limits).unwrap();
    assert_eq!(usage.bytes, mesh_usage.bytes + std::mem::size_of::<Geometry>() as u64);
    let tight = Limits { max_bytes: usage.bytes - 1, ..limits };
    assert!(hatch.validate(&tight).is_err());
}

#[test]
fn hatch_still_rejects_invalid_display_faces() {
    let display = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let hatch = Geometry::Hatch(Hatch { display, loops: vec![], pattern: None, color: [255; 4] });
    assert!(hatch.validate(&Limits::default()).is_err());
}

#[test]
fn mesh_counts_uv_capacity_and_rejects_invalid_uvs() {
    let limits = Limits::default();
    let mut mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let base = Geometry::Mesh(mesh.clone()).validate(&limits).unwrap();
    mesh.corner_uvs = Vec::with_capacity(1024);
    mesh.corner_uvs.extend([[0., 0.], [1., 0.], [0., 1.]]);
    let expected = mesh.corner_uvs.capacity() * std::mem::size_of::<[f64; 2]>();
    let direct = mesh.validate(&limits).unwrap();
    let geometry = Geometry::Mesh(mesh);
    let with_uv = geometry.validate(&limits).unwrap();
    assert_eq!(with_uv.bytes - base.bytes, expected as u64);
    assert_eq!(with_uv.bytes, direct.bytes + std::mem::size_of::<Geometry>() as u64);
    let tight = Limits { max_bytes: with_uv.bytes - 1, ..limits };
    assert!(geometry.validate(&tight).is_err());
    let Geometry::Mesh(mut invalid) = geometry else { unreachable!() };
    invalid.corner_uvs.pop();
    assert!(Geometry::Mesh(invalid.clone()).validate(&limits).is_err());
    invalid.corner_uvs.push([f64::NAN, 0.]);
    assert!(Geometry::Mesh(invalid).validate(&limits).is_err());
}

#[test]
fn empty_uv_buffer_still_counts_reserved_capacity() {
    let limits = Limits::default();
    let mut mesh = Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let before = mesh.validate(&limits).unwrap();
    mesh.corner_uvs.reserve_exact(256);
    let after = mesh.validate(&limits).unwrap();
    assert_eq!(after.bytes - before.bytes, (mesh.corner_uvs.capacity() * std::mem::size_of::<[f64; 2]>()) as u64);
    assert!(mesh.validate(&Limits { max_bytes: after.bytes - 1, ..limits }).is_err());
}

#[test]
fn hatch_counts_loop_table_and_pattern_capacity() {
    let limits = Limits::default();
    let make = |reserved: bool| {
        let mut loops = if reserved { Vec::with_capacity(512) } else { Vec::new() };
        let mut pattern = if reserved { String::with_capacity(8192) } else { String::new() };
        pattern.push('x');
        loops.push(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        Geometry::Hatch(Hatch {
            display: Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] },
            loops, pattern: Some(pattern), color: [255; 4],
        })
    };
    let base = make(false);
    let reserved = make(true);
    let Geometry::Hatch(a) = &base else { unreachable!() };
    let Geometry::Hatch(b) = &reserved else { unreachable!() };
    let expected = (b.loops.capacity() - a.loops.capacity()) * std::mem::size_of::<Vec<[f64; 3]>>()
        + b.pattern.as_ref().unwrap().capacity() - a.pattern.as_ref().unwrap().capacity();
    let before = base.validate(&limits).unwrap();
    let after = reserved.validate(&limits).unwrap();
    assert_eq!(after.bytes - before.bytes, expected as u64);
    assert!(reserved.validate(&Limits { max_bytes: after.bytes - 1, ..limits }).is_err());
}

#[test]
fn document_inventory_includes_hatch_reserved_buffers() {
    use om9_core::model::{Document, Entity};
    let limits = Limits::default();
    let mut document = Document::new("hatch accounting");
    let entity = Entity::new("hatch", Geometry::Hatch(Hatch {
        display: Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] },
        loops: vec![], pattern: Some(String::new()), color: [255; 4],
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let before = document.validate(&limits).unwrap();
    let Geometry::Hatch(hatch) = &mut document.entities.get_mut(&id).unwrap().geometry else { unreachable!() };
    hatch.display.corner_uvs.reserve_exact(64);
    hatch.loops.reserve_exact(128);
    hatch.pattern.as_mut().unwrap().reserve_exact(256);
    let expected = hatch.display.corner_uvs.capacity() * std::mem::size_of::<[f64; 2]>()
        + hatch.loops.capacity() * std::mem::size_of::<Vec<[f64; 3]>>()
        + hatch.pattern.as_ref().unwrap().capacity();
    let after = document.validate(&limits).unwrap();
    assert_eq!(after.bytes - before.bytes, expected as u64);
}
