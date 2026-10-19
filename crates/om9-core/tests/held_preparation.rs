// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::Geometry,
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};

#[test]
fn external_input_is_reserved_before_preparing_a_candidate_and_undo() {
    let limits = Limits {
        max_bytes: 256 * 1024,
        ..Limits::default()
    };
    let document = Document::new("held input");
    let id = document.id;
    let coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let owned = coordinator.retained_usage().unwrap().bytes;
    let input = vec![0u8; (limits.max_bytes - owned) as usize];
    let before = coordinator.snapshot();
    let request = || {
        TransactionRequest::new(
            "Insert",
            vec![Mutation::Insert {
                document: id,
                entity: Entity::new("point", Geometry::Point([1., 2., 3.]), None),
            }],
        )
    };
    assert!(coordinator.prepare(request()).is_ok());
    assert!(matches!(
        coordinator.prepare_with_held_usage(
            request(),
            Usage {
                bytes: input.capacity() as u64,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
    assert_eq!(coordinator.snapshot(), before);
}

#[test]
fn held_output_reservation_uses_the_same_counter_as_new_mutations() {
    let limits = Limits {
        max_outputs: 1,
        ..Limits::default()
    };
    let document = Document::new("held output counter");
    let id = document.id;
    let coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let request = || {
        TransactionRequest::new(
            "Insert",
            vec![Mutation::Insert {
                document: id,
                entity: Entity::new("point", Geometry::Point([1., 2., 3.]), None),
            }],
        )
    };
    assert!(coordinator.prepare(request()).is_ok());
    assert!(matches!(
        coordinator.prepare_with_held_usage(
            request(),
            Usage {
                outputs: 1,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
}
fn fixture(limits: Limits) -> (TransactionCoordinator, om9_core::identity::DocumentId) {
    let d = Document::new("receipt owner");
    let id = d.id;
    (
        TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap(),
        id,
    )
}
fn insertion(id: om9_core::identity::DocumentId) -> TransactionRequest {
    TransactionRequest::new(
        "prepared",
        vec![Mutation::Insert {
            document: id,
            entity: Entity::new("point", Geometry::Point([1.; 3]), None),
        }],
    )
}
#[test]
fn extending_prepared_receipt_reserves_full_candidate_and_failure_keeps_old_reservation() {
    let limits = Limits {
        max_bytes: 128 * 1024,
        ..Limits::default()
    };
    let (mut c, id) = fixture(limits);
    let mut p = c.prepare(insertion(id)).unwrap();
    let before = c.snapshot();
    let actual_receipt = vec![0u8; limits.max_bytes as usize];
    assert!(matches!(
        c.reserve_prepared_held_usage(
            &mut p,
            Usage {
                bytes: actual_receipt.capacity() as u64,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
    drop(actual_receipt);
    c.commit(p).unwrap();
    assert_eq!(c.world().documents[&id].entities.len(), 1);
}
#[test]
fn prepared_receipt_reservation_replaces_all_counters_instead_of_accumulating_old_metadata() {
    let limits = Limits {
        max_outputs: 3,
        ..Limits::default()
    };
    let (mut c, id) = fixture(limits);
    let mut p = c
        .prepare_with_held_usage(
            insertion(id),
            Usage {
                outputs: 1,
                ..Usage::default()
            },
        )
        .unwrap();
    c.reserve_prepared_held_usage(
        &mut p,
        Usage {
            outputs: 2,
            ..Usage::default()
        },
    )
    .unwrap();
    assert!(matches!(
        c.reserve_prepared_held_usage(
            &mut p,
            Usage {
                outputs: 3,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
    c.commit(p).unwrap();
}
#[test]
fn prepared_receipt_reservation_cannot_cross_coordinator_or_caller_lifecycle() {
    let limits = Limits::default();
    let (mut c, id) = fixture(limits);
    let other = TransactionCoordinator::new(c.world().clone(), limits).unwrap();
    let mut p = c.prepare(insertion(id)).unwrap();
    assert_eq!(
        other.reserve_prepared_held_usage(&mut p, Usage::default()),
        Err(Error::WrongOwner)
    );
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    c.abort(&token).unwrap();
    assert!(matches!(
        c.reserve_prepared_held_usage(&mut p, Usage::default()),
        Err(Error::Stale(_))
    ));
}
#[test]
fn prepared_receipt_reservation_checks_bound_cancellation() {
    let (c, id) = fixture(Limits::default());
    let request = insertion(id);
    let cancel = request.cancel.clone();
    let mut p = c.prepare(request).unwrap();
    cancel.cancel();
    assert_eq!(
        c.reserve_prepared_held_usage(&mut p, Usage::default()),
        Err(Error::Canceled)
    );
}
