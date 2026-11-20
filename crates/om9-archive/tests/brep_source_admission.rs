// SPDX-License-Identifier: LGPL-2.1-or-later
//! A canonical OCCT BRep source is a document payload, not a name. It must be
//! admitted above the 16 KiB name cap while the aggregate byte budget still
//! bounds it.
use om9_archive::Archive;
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
};
use uuid::Uuid;

fn fixture() -> (Document, String) {
    let source = format!(
        "CASCADE Topology V3, (c) Open Cascade\n{}",
        "x".repeat(24 * 1024)
    );
    let display = BrepDisplay {
        source_sha256: [0; 32],
        deflection_mm: 0.01,
        points: vec![[0., 0., 0.], [10., 0., 0.], [0., 10., 0.]],
        triangles: vec![[0, 1, 2, 0]],
        segments: Vec::new(),
        vertices: vec![0, 1, 2],
        topology: om9_core::brep::BrepTopology {
            faces: vec![Uuid::new_v4()],
            edges: Vec::new(),
            vertex_ids: (0..3).map(|_| Uuid::new_v4()).collect(),
            face_edges: Vec::new(),
        }.into(),
    };
    let brep = Brep::from_native(source.clone(), display, &Limits::default()).unwrap();
    let mut document = Document::new("BRep source admission");
    let (canonical, _display) = brep.into_canonical_and_representation();
    let entity = Entity::new("large BRep", Geometry::Brep(canonical), document.current_layer);
    document.entities.insert(entity.id, entity);
    (document, source)
}

#[test]
fn large_brep_source_is_admitted_but_v5_archive_refuses_it() {
    let (document, source) = fixture();
    document.validate(&Limits::default()).unwrap();
    assert!(matches!(Archive::encode(&document, &Limits::default()),
        Err(om9_core::Error::Unsupported("BRep archive migration to v5 is not available"))));
    let retained = document.entities.values().next().unwrap();
    let Geometry::Brep(retained) = &retained.geometry else {
        panic!("expected a Brep");
    };
    assert_eq!(retained.source(), source);
}

#[test]
#[ignore = "BRep source round-trip remains an open codec acceptance gate"]
fn large_brep_source_round_trips_through_the_archive() {
    let (document, source) = fixture();
    let folder = std::env::temp_dir().join(format!("om9-brep-source-{}", Uuid::new_v4()));
    std::fs::create_dir(&folder).unwrap();
    let path = folder.join("document.om9");
    Archive::save(&path, &document, &Limits::default(), &Default::default()).unwrap();
    let restored = Archive::open(&path, &Limits::default()).unwrap();
    let Geometry::Brep(loaded) = &restored.entities.values().next().unwrap().geometry else { panic!("expected a BRep") };
    assert_eq!(loaded.source(), source);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
