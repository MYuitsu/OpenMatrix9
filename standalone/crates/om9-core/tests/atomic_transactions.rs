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
    let mut doc = Document::new("atomic");
    let entity = Entity::new("source", Geometry::Point([1., 2., 3.]), doc.current_layer);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let did = doc.id;
    (
        TransactionCoordinator::new(
            World::new(vec![doc], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        did,
        id,
    )
}
fn translated(
    c: &TransactionCoordinator,
    doc: DocumentId,
    id: ObjectId,
    x: f64,
) -> TransactionRequest {
    TransactionRequest::new(
        "Move",
        vec![Mutation::Placement {
            source: c.world().documents[&doc].stamp(id).unwrap(),
            placement: Transform::translation([x, 0., 0.]),
        }],
    )
}
fn seeded_redo() -> (TransactionCoordinator, DocumentId, ObjectId) {
    let (mut c, d, id) = fixture();
    let p = c.prepare(translated(&c, d, id, 10.)).unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    assert_eq!(c.redo_len(), 1);
    (c, d, id)
}

#[test]
fn deleted_uuid_cannot_be_reallocated_to_resurrect_an_old_source_stamp() {
    let (mut c, d, id) = fixture();
    let original = c.world().documents[&d].entities[&id].clone();
    let source = c.world().documents[&d].stamp(id).unwrap();
    let deletion = c
        .prepare(TransactionRequest::new(
            "Delete",
            vec![Mutation::Delete { source }],
        ))
        .unwrap();
    c.commit(deletion).unwrap();
    let other = Entity::new("other", Geometry::Point([0.; 3]), None);
    let other_id = other.id;
    let p = c
        .prepare(TransactionRequest::new(
            "Insert",
            vec![Mutation::Insert {
                document: d,
                entity: other,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let p = c.prepare(translated(&c, d, other_id, 10.)).unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let baseline = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Reallocate UUID",
            vec![Mutation::Insert {
                document: d,
                entity: original
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), baseline);
    assert!(c.world().documents[&d].resolve(&source).is_err());
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&d].entities[&other_id].placement.0[0][3],
        10.
    );
}

#[test]
fn undo_of_new_insert_does_not_make_its_uuid_available_for_another_object() {
    let doc = Document::new("undo identity");
    let d = doc.id;
    let entity = Entity::new("A", Geometry::Point([1., 2., 3.]), None);
    let id = entity.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Insert",
            vec![Mutation::Insert {
                document: d,
                entity: entity.clone(),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let stamp = c.world().documents[&d].stamp(id).unwrap();
    c.undo().unwrap();
    let baseline = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Recreate",
            vec![Mutation::Insert {
                document: d,
                entity
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
    assert!(
        c.world().documents[&d].resolve(&stamp).is_ok(),
        "Redo restores the original persistent object"
    );
}

#[test]
fn close_and_reopen_of_a_document_clone_cannot_restore_an_old_owner_stamp() {
    let (mut c, d, id) = fixture();
    let saved = c.world().documents[&d].as_ref().clone();
    let old = saved.stamp(id).unwrap();
    c.close_document(d).unwrap();
    c.open_document(saved).unwrap();
    assert!(c.world().documents[&d].resolve(&old).is_err());
    assert_eq!(
        c.world().documents[&d].entities[&id].geometry,
        Geometry::Point([1., 2., 3.])
    );
}

#[test]
fn legacy_nil_session_is_format_data_until_explicit_runtime_restore() {
    let mut document = Document::new("legacy");
    document.owner_session = SessionId::legacy_nil();
    let world = World::new(vec![document.clone()], &Limits::default()).unwrap();
    assert!(TransactionCoordinator::new(world, Limits::default()).is_err());
    let (mut c, _, _) = fixture();
    let before = c.snapshot();
    assert!(c.open_document(document).is_err());
    assert_eq!(c.snapshot(), before);
}

#[test]
fn accepted_move_is_one_atomic_undo_and_redo() {
    let (mut c, d, id) = fixture();
    let before = c.world().documents[&d].entities[&id].clone();
    let prepared = c.prepare(translated(&c, d, id, 10.)).unwrap();
    assert_eq!(
        c.world().documents[&d].entities[&id],
        before,
        "prepare must stay private"
    );
    c.commit(prepared).unwrap();
    assert_eq!(c.undo_len(), 1);
    assert_eq!(
        c.world().documents[&d].entities[&id]
            .placement
            .point([1., 2., 3.])
            .unwrap(),
        [11., 2., 3.]
    );
    c.undo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id], before);
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&d].entities[&id]
            .placement
            .point([1., 2., 3.])
            .unwrap(),
        [11., 2., 3.]
    );
}

#[test]
fn invalid_source_keeps_immediate_state_and_executable_prior_redo() {
    let (mut c, d, id) = seeded_redo();
    let baseline = c.snapshot();
    let mut request = translated(&c, d, id, 20.);
    if let Mutation::Placement { source, .. } = &mut request.mutations[0] {
        source.document = DocumentId::new();
    }
    assert!(c.prepare(request).is_err());
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 10.);
}

#[test]
fn cancellation_after_prepare_keeps_every_record_and_redo() {
    let (mut c, d, id) = seeded_redo();
    let baseline = c.snapshot();
    let request = translated(&c, d, id, 20.);
    let cancel = request.cancel.clone();
    let prepared = c.prepare(request).unwrap();
    cancel.cancel();
    assert_eq!(c.commit(prepared), Err(Error::Canceled));
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 10.);
}

#[test]
fn intervening_commit_and_undo_do_not_resurrect_a_prepared_job() {
    let (mut c, d, id) = fixture();
    let old = c.prepare(translated(&c, d, id, 20.)).unwrap();
    let p = c.prepare(translated(&c, d, id, 10.)).unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let baseline = c.snapshot();
    assert!(matches!(c.commit(old), Err(Error::Stale(_))));
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 10.);
}

#[test]
fn cross_document_prepare_failure_never_transfers_first_participant() {
    let (mut c, d, id) = seeded_redo();
    let destination = Document::new("destination");
    let dest = destination.id;
    let layer = destination.current_layer;
    c.open_document(destination).unwrap();
    let baseline = c.snapshot();
    let source = c.world().documents[&d].stamp(id).unwrap();
    let request = TransactionRequest::new(
        "transfer",
        vec![
            Mutation::Move {
                source,
                destination: dest,
                layer,
            },
            Mutation::Insert {
                document: dest,
                entity: Entity::new("bad", Geometry::Point([f64::NAN, 0., 0.]), layer),
            },
        ],
    );
    assert!(c.prepare(request).is_err());
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
    assert!(c.world().documents[&d].entities.contains_key(&id));
    assert!(c.world().documents[&dest].entities.is_empty());
}

#[test]
fn cross_document_move_undo_restores_both_document_owners() {
    let (mut c, d, id) = fixture();
    let destination = Document::new("destination");
    let dest = destination.id;
    let layer = destination.current_layer;
    c.open_document(destination).unwrap();
    let before = c.world().clone();
    let source = c.world().documents[&d].stamp(id).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "transfer",
            vec![Mutation::Move {
                source,
                destination: dest,
                layer,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(!c.world().documents[&d].entities.contains_key(&id));
    assert!(c.world().documents[&dest].entities.contains_key(&id));
    c.undo().unwrap();
    assert_eq!(c.world(), &before);
    c.redo().unwrap();
    assert!(c.world().documents[&dest].entities.contains_key(&id));
}

#[test]
fn preacceptance_callback_cannot_mutate_or_veto_with_partial_state() {
    let (mut c, d, id) = seeded_redo();
    let baseline = c.snapshot();
    let p = c.prepare(translated(&c, d, id, 20.)).unwrap();
    let result = c.commit_with_guard(p, |coordinator| {
        assert_eq!(coordinator.undo(), Err(Error::Busy));
        assert_eq!(coordinator.close_document(d), Err(Error::Busy));
        Err(Error::Invalid("acceptance callback"))
    });
    assert!(result.is_err());
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
}

#[test]
fn panicking_callback_is_rejected_and_barrier_is_released() {
    let (mut c, d, id) = seeded_redo();
    let baseline = c.snapshot();
    let p = c.prepare(translated(&c, d, id, 20.)).unwrap();
    assert!(c.commit_with_guard(p, |_| panic!("observer")).is_err());
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
}

#[test]
fn postacceptance_notification_failure_reports_committed_receipt() {
    let (mut c, d, id) = fixture();
    let p = c.prepare(translated(&c, d, id, 10.)).unwrap();
    let receipt = c
        .commit_with_observer(p, |coordinator| {
            assert_eq!(coordinator.undo(), Err(Error::Busy));
            Err(Error::Invalid("notification"))
        })
        .unwrap();
    assert_eq!(receipt.diagnostics.len(), 1);
    assert_eq!(c.undo_len(), 1);
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 10.);
    c.undo().unwrap();
}

#[test]
fn source_document_close_invalidates_job_without_resurrection() {
    let (mut c, d, id) = fixture();
    let p = c.prepare(translated(&c, d, id, 10.)).unwrap();
    c.close_document(d).unwrap();
    let baseline = c.snapshot();
    assert!(c.commit(p).is_err());
    assert_eq!(c.snapshot(), baseline);
    assert!(!c.world().documents.contains_key(&d));
}

#[test]
fn revision_overflow_rejects_before_public_state_or_redo_changes() {
    let mut doc = Document::new("overflow");
    let mut e = Entity::new("point", Geometry::Point([1., 2., 3.]), doc.current_layer);
    e.geometry_revision = u64::MAX;
    let id = e.id;
    let d = doc.id;
    doc.entities.insert(id, e);
    let c = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let baseline = c.snapshot();
    assert!(matches!(
        c.prepare(translated(&c, d, id, 10.)),
        Err(Error::Overflow(_))
    ));
    assert_eq!(c.snapshot(), baseline);
}

#[test]
fn no_op_commit_never_erases_redo_or_creates_empty_group() {
    let (mut c, _, _) = seeded_redo();
    let baseline = c.snapshot();
    let p = c.prepare(TransactionRequest::new("empty", vec![])).unwrap();
    c.commit(p).unwrap();
    assert_eq!(c.snapshot(), baseline);
    c.redo().unwrap();
}
