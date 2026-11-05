// SPDX-License-Identifier: LGPL-2.1-or-later
//! An entity inserted with a semantic slot (e.g. a gem created by an add-on command) is
//! admitted in the same transaction that creates it: the request charges its extensions
//! exactly as the stored document does, and one Undo removes object and slot together.
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity, Extension, GEM_EXTENSION_SLOT},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};

fn tetra() -> Mesh {
    Mesh {
        vertices: vec![[0.; 3], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        faces: vec![vec![0, 2, 1], vec![0, 1, 3], vec![1, 2, 3], vec![2, 0, 3]],
        normals: Vec::new(),
        colors: Vec::new(),
    }
}

fn gem(i: usize) -> Entity {
    let mut e = Entity::new("Gem", Geometry::Mesh(tetra()), None);
    e.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        Extension::new(
            1,
            format!(r#"{{"cut":"round","diameter_mm":1.5,"index":{i}}}"#),
        )
        .unwrap(),
    );
    e
}

#[test]
fn many_inserted_entities_with_extensions_commit_in_one_transaction() {
    let doc = Document::new("gems");
    let d = doc.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let mutations = (0..64)
        .map(|i| Mutation::Insert {
            document: d,
            entity: gem(i),
        })
        .collect();
    let p = c
        .prepare(TransactionRequest::new("Gem On Crv", mutations))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert_eq!(doc.entities.len(), 64);
    assert!(doc.entities.values().all(|e| {
        e.extension(GEM_EXTENSION_SLOT)
            .is_some_and(|x| x.payload.contains("round"))
    }));
    c.undo().unwrap();
    assert!(
        c.world().documents[&d].entities.is_empty(),
        "one Undo removes every gem"
    );
}

#[test]
fn an_invalid_slot_name_on_insert_is_refused_before_commit() {
    let doc = Document::new("gems");
    let d = doc.id;
    let c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let mut e = gem(0);
    let ext = e.extensions.remove(GEM_EXTENSION_SLOT).unwrap();
    e.extensions.insert("Not A Slot!".to_owned(), ext);
    let r = c.prepare(TransactionRequest::new(
        "bad",
        vec![Mutation::Insert {
            document: d,
            entity: e,
        }],
    ));
    assert!(
        matches!(r, Err(Error::Invalid("extension slot"))),
        "{:?}",
        r.err()
    );
}
