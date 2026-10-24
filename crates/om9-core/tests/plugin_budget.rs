// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::{Limits, Usage},
    geometry::Geometry,
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};

#[test]
fn prepared_external_allocation_checks_authority_epoch_and_reservation() {
    let limits = Limits::default();
    let document = Document::new("SDK reservation");
    let id = document.id;
    let world = World::new(vec![document], &limits).unwrap();
    let mut owner = TransactionCoordinator::new(world.clone(), limits).unwrap();
    let foreign = TransactionCoordinator::new(world, limits).unwrap();
    let request = || {
        TransactionRequest::new(
            "SDK point",
            vec![Mutation::Insert {
                document: id,
                entity: Entity::new("point", Geometry::Point([1., 2., 3.]), None),
            }],
        )
    };
    let prepared = owner.prepare(request()).unwrap();
    let remaining = owner
        .prepared_remaining_limits(&prepared, Usage::default())
        .unwrap();
    assert!(remaining.max_bytes < limits.max_bytes);
    assert!(
        foreign
            .prepared_remaining_limits(&prepared, Usage::default())
            .is_err()
    );
    assert!(
        owner
            .prepared_remaining_limits(
                &prepared,
                Usage {
                    bytes: limits.max_bytes,
                    ..Usage::default()
                }
            )
            .is_err()
    );
    let newer = owner.prepare(request()).unwrap();
    owner.commit(newer).unwrap();
    assert!(
        owner
            .prepared_remaining_limits(&prepared, Usage::default())
            .is_err()
    );
}
