// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity},
    transaction::*,
};
fn fixture() -> (TransactionCoordinator, DocumentId, ObjectId) {
    let mut d = Document::new("scope");
    let e = Entity::new("point", Geometry::Point([1., 2., 3.]), d.current_layer);
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    (
        TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        did,
        id,
    )
}
fn change(
    c: &TransactionCoordinator,
    token: &ScopeToken,
    d: DocumentId,
    id: ObjectId,
    x: f64,
) -> TransactionRequest {
    let source = c.editing_world(token).unwrap().documents[&d]
        .stamp(id)
        .unwrap();
    let mut r = TransactionRequest::new(
        "Move",
        vec![Mutation::Placement {
            source,
            placement: Transform::translation([x, 0., 0.]),
        }],
    );
    r.owner = Some(token.clone());
    r
}

#[test]
fn rolled_back_caller_identity_is_retired_and_global_prior_redo_remains_executable() {
    let (mut c, d, id) = fixture();
    let source = c.world().documents[&d].stamp(id).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([10., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    c.savepoint(&token).unwrap();
    let entity = Entity::new("private", Geometry::Point([3., 4., 5.]), None);
    let private_id = entity.id;
    let mut request = TransactionRequest::new(
        "private insert",
        vec![Mutation::Insert {
            document: d,
            entity: entity.clone(),
        }],
    );
    request.owner = Some(token.clone());
    let p = c.prepare(request).unwrap();
    assert!(!c.commit(p).unwrap().published);
    let private_stamp = c.editing_world(&token).unwrap().documents[&d]
        .stamp(private_id)
        .unwrap();
    c.rollback_savepoint(&token).unwrap();
    let baseline = c.snapshot();
    let mut request = TransactionRequest::new(
        "recreate private identity",
        vec![Mutation::Insert {
            document: d,
            entity,
        }],
    );
    request.owner = Some(token.clone());
    assert!(c.prepare(request).is_err());
    assert_eq!(c.snapshot(), baseline);
    assert!(
        c.editing_world(&token).unwrap().documents[&d]
            .resolve(&private_stamp)
            .is_err()
    );
    c.abort(&token).unwrap();
    c.redo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 10.);
    assert!(!c.world().documents[&d].entities.contains_key(&private_id));
}
#[test]
fn booked_scope_failed_prepare_keeps_booking_and_prior_redo() {
    let (mut c, d, id) = fixture();
    let token = c.book("caller", true).unwrap();
    let baseline = c.snapshot();
    let mut r = change(&c, &token, d, id, 10.);
    if let Mutation::Placement { source, .. } = &mut r.mutations[0] {
        source.generation += 1;
    }
    assert!(c.prepare(r).is_err());
    assert_eq!(c.snapshot(), baseline);
    c.begin(&token).unwrap();
    c.abort(&token).unwrap();
}
#[test]
fn active_scope_merge_retains_first_write_baseline_and_one_undo() {
    let (mut c, d, id) = fixture();
    let baseline = c.world().clone();
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    for x in [10., 20.] {
        let p = c.prepare(change(&c, &token, d, id, x)).unwrap();
        c.commit(p).unwrap();
    }
    assert_eq!(c.world(), &baseline);
    assert_eq!(c.undo_len(), 0);
    assert_eq!(
        c.editing_world(&token).unwrap().documents[&d].entities[&id]
            .placement
            .0[0][3],
        20.
    );
    c.finish(&token).unwrap();
    assert_eq!(c.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.world(), &baseline);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 20.);
}
#[test]
fn nested_savepoint_abort_restores_parent_pending_state() {
    let (mut c, d, id) = fixture();
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    let p = c.prepare(change(&c, &token, d, id, 10.)).unwrap();
    c.commit(p).unwrap();
    c.savepoint(&token).unwrap();
    let p = c.prepare(change(&c, &token, d, id, 20.)).unwrap();
    c.commit(p).unwrap();
    c.rollback_savepoint(&token).unwrap();
    assert_eq!(
        c.editing_world(&token).unwrap().documents[&d].entities[&id]
            .placement
            .0[0][3],
        10.
    );
    c.finish(&token).unwrap();
    c.undo().unwrap();
    assert_eq!(
        c.world().documents[&d].entities[&id].placement,
        Transform::identity()
    );
}
#[test]
fn cancellation_keeps_active_nested_scope_and_old_redo_immediately() {
    let (mut c, d, id) = fixture();
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    c.savepoint(&token).unwrap();
    let r = change(&c, &token, d, id, 10.);
    let cancel = r.cancel.clone();
    let p = c.prepare(r).unwrap();
    let baseline = c.snapshot();
    cancel.cancel();
    assert_eq!(c.commit(p), Err(Error::Canceled));
    assert_eq!(c.snapshot(), baseline);
    c.rollback_savepoint(&token).unwrap();
    c.abort(&token).unwrap();
}
#[test]
fn wrong_transaction_identity_cannot_borrow_caller_scope() {
    let (mut c, d, id) = fixture();
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    let mut wrong = token.clone();
    wrong.id = TransactionId::new();
    let baseline = c.snapshot();
    assert_eq!(c.abort(&wrong), Err(Error::WrongOwner));
    assert!(
        c.prepare(change(&c, &token, d, id, 10.))
            .unwrap()
            .belongs_to(&token)
    );
    assert_eq!(c.snapshot(), baseline);
}
#[test]
fn nonrecording_empty_scope_keeps_prior_stacks_and_success_creates_no_undo() {
    let (mut c, d, id) = fixture();
    let token = c.book("unrecorded", false).unwrap();
    c.begin(&token).unwrap();
    let p = c.prepare(change(&c, &token, d, id, 10.)).unwrap();
    c.commit(p).unwrap();
    c.finish(&token).unwrap();
    assert_eq!(c.undo_len(), 0);
    assert_eq!(c.redo_len(), 0);
    let baseline = c.snapshot();
    let token = c.book("empty", false).unwrap();
    c.begin(&token).unwrap();
    c.finish(&token).unwrap();
    assert_eq!(c.world(), &baseline.world);
    assert_eq!(c.undo_len(), 0);
}
#[test]
fn scope_owned_by_another_thread_cannot_be_finished() {
    let (mut c, _, _) = fixture();
    let token = c.book("thread", true).unwrap();
    c.begin(&token).unwrap();
    let (mut c, token, result) = std::thread::spawn(move || {
        let result = c.finish(&token);
        (c, token, result)
    })
    .join()
    .unwrap();
    assert_eq!(result, Err(Error::WrongOwner));
    c.abort(&token).unwrap();
}
