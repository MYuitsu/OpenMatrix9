// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use std::sync::Arc;

#[test]
fn prepared_transaction_cannot_publish_into_an_equal_coordinator_sharing_the_world() {
    let mut document = Document::new("coordinator authority");
    let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let object = entity.id;
    document.entities.insert(object, entity);
    let id = document.id;
    let world = World::new(vec![document], &Limits::default()).unwrap();
    let first = TransactionCoordinator::new(world.clone(), Limits::default()).unwrap();
    let mut second = TransactionCoordinator::new(world, Limits::default()).unwrap();
    assert_eq!(first.epoch(), second.epoch());
    assert!(Arc::ptr_eq(
        &first.world().documents[&id],
        &second.world().documents[&id]
    ));
    assert_ne!(first.snapshot(), second.snapshot());
    let prepared = first
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source: first.world().documents[&id].stamp(object).unwrap(),
                placement: Transform::translation([7., 0., 0.]),
            }],
        ))
        .unwrap();
    let before = second.snapshot();
    assert_eq!(second.commit(prepared), Err(Error::WrongOwner));
    assert_eq!(second.snapshot(), before);
}

#[test]
fn aborting_a_caller_cannot_revive_a_prepared_global_transaction() {
    let document = Document::new("caller lifecycle");
    let id = document.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let prepared = c
        .prepare(TransactionRequest::new(
            "Insert",
            vec![Mutation::Insert {
                document: id,
                entity: Entity::new("point", Geometry::Point([1., 2., 3.]), None),
            }],
        ))
        .unwrap();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    c.savepoint(&owner).unwrap();
    c.rollback_savepoint(&owner).unwrap();
    c.abort(&owner).unwrap();
    let before = c.snapshot();
    assert!(c.commit(prepared).is_err());
    assert_eq!(c.snapshot(), before);
}
