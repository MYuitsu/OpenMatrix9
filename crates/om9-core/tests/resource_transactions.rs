// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
    transaction::*,
};

fn resource(bytes: &[u8]) -> Resource {
    Resource::from_bytes(
        "source.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        bytes,
        &Limits::default(),
    )
    .unwrap()
}

#[test]
fn a_resource_cannot_be_owned_by_two_documents() {
    let retained = resource(b"independent source bytes");
    let mut a = Document::new("A");
    let mut b = Document::new("B");
    a.resources.insert(retained.id, retained.clone());
    b.resources.insert(retained.id, retained);
    assert!(matches!(
        World::new(vec![a, b], &Limits::default()),
        Err(Error::Invalid(_))
    ));
}

fn fixture() -> (TransactionCoordinator, DocumentId, DocumentId, ObjectId) {
    let mut doc = Document::new("resources");
    let entity = Entity::new("existing", Geometry::Point([1., 2., 3.]), None);
    let object = entity.id;
    doc.entities.insert(object, entity);
    let document = doc.id;
    let mut other = Document::new("other");
    let initial = resource(b"initial other-document source");
    other.resources.insert(initial.id, initial);
    let other_id = other.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![doc, other], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    (coordinator, document, other_id, object)
}
fn insert(document: DocumentId, expected_revision: u64, resource: Resource) -> Mutation {
    Mutation::ResourceInsert {
        document,
        expected_revision,
        resource,
    }
}
fn delete(document: DocumentId, expected_revision: u64, resource: ResourceId) -> Mutation {
    Mutation::ResourceDelete {
        document,
        expected_revision,
        resource,
    }
}
fn seeded_redo() -> (TransactionCoordinator, DocumentId, DocumentId, ObjectId) {
    let (mut c, d, other, object) = fixture();
    let source = c.world().documents[&d].stamp(object).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([9., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    (c, d, other, object)
}
fn redo_existing(c: &mut TransactionCoordinator, document: DocumentId, object: ObjectId) {
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&document].entities[&object].placement.0[0][3],
        9.
    );
}

#[test]
fn resources_and_geometry_publish_privately_as_one_undo_entry() {
    let (mut c, d, _, _) = fixture();
    let before = c.world().clone();
    let revision = c.world().documents[&d].revision;
    let a = resource(b"A original bytes");
    let b = resource(b"B independently retained bytes");
    let point = Entity::new("imported", Geometry::Point([4., 5., 6.]), None);
    let object = point.id;
    let p = c
        .prepare(TransactionRequest::new(
            "Import",
            vec![
                insert(d, revision, a.clone()),
                insert(d, revision, b.clone()),
                Mutation::Insert {
                    document: d,
                    entity: point,
                },
            ],
        ))
        .unwrap();
    assert_eq!(c.world(), &before);
    assert_eq!(
        p.candidate_world().documents[&d].resources[&a.id]
            .payload()
            .unwrap(),
        b"A original bytes"
    );
    c.commit(p).unwrap();
    assert_eq!(c.undo_len(), 1);
    assert_eq!(c.world().documents[&d].revision, revision + 1);
    assert_eq!(
        c.world().documents[&d].entities[&object].geometry,
        Geometry::Point([4., 5., 6.])
    );
    let after = c.world().clone();
    c.undo().unwrap();
    assert_eq!(c.world(), &before);
    c.redo().unwrap();
    assert_eq!(c.world(), &after);
    assert_eq!(
        c.world().documents[&d].resources[&b.id].payload().unwrap(),
        b"B independently retained bytes"
    );
}

#[test]
fn invalid_resource_batches_preserve_the_snapshot_and_executable_redo() {
    for failure in [
        "stale",
        "canceled",
        "wrong resource",
        "missing resource",
        "missing document",
        "descriptor",
        "duplicate",
        "cross-document duplicate",
    ] {
        let (mut c, d, other, object) = seeded_redo();
        let revision = c.world().documents[&d].revision;
        let retained = resource(b"new source");
        let mut mutations = vec![insert(d, revision, retained.clone())];
        match failure {
            "stale" => mutations.push(delete(d, revision + 1, retained.id)),
            "wrong resource" => mutations.push(delete(
                d,
                revision,
                *c.world().documents[&other].resources.keys().next().unwrap(),
            )),
            "missing resource" => mutations.push(delete(d, revision, ResourceId::new())),
            "missing document" => {
                mutations.push(insert(DocumentId::new(), 1, resource(b"missing owner")))
            }
            "descriptor" => {
                let descriptor: Resource =
                    serde_json::from_slice(&serde_json::to_vec(&retained).unwrap()).unwrap();
                mutations = vec![insert(d, revision, descriptor)];
            }
            "duplicate" => mutations.push(insert(d, revision, retained)),
            "cross-document duplicate" => mutations.push(insert(
                other,
                c.world().documents[&other].revision,
                retained,
            )),
            _ => {}
        }
        let request = TransactionRequest::new(failure, mutations);
        if failure == "canceled" {
            request.cancel.cancel();
        }
        let before = c.snapshot();
        assert!(c.prepare(request).is_err(), "{failure}");
        assert_eq!(c.snapshot(), before, "{failure}");
        redo_existing(&mut c, d, object);
    }
}

#[test]
fn cancellation_after_prepare_does_not_publish_or_retire_the_resource_id() {
    let (mut c, d, _, object) = seeded_redo();
    let retained = resource(b"cancel before acceptance");
    let revision = c.world().documents[&d].revision;
    let request = TransactionRequest::new("Import", vec![insert(d, revision, retained.clone())]);
    let cancel = request.cancel.clone();
    let p = c.prepare(request).unwrap();
    let before = c.snapshot();
    cancel.cancel();
    assert!(matches!(c.commit(p), Err(Error::Canceled)));
    assert_eq!(c.snapshot(), before);
    assert!(
        c.prepare(TransactionRequest::new(
            "Retry",
            vec![insert(d, revision, retained)]
        ))
        .is_ok()
    );
    redo_existing(&mut c, d, object);
}

#[test]
fn rejected_acceptance_guard_preserves_resource_authority_and_prior_redo() {
    let (mut c, d, _, object) = seeded_redo();
    let retained = resource(b"rejected guard");
    let revision = c.world().documents[&d].revision;
    let p = c
        .prepare(TransactionRequest::new(
            "Import",
            vec![insert(d, revision, retained.clone())],
        ))
        .unwrap();
    let before = c.snapshot();
    assert!(
        c.commit_with_guard(p, |_| Err(Error::Invalid("reject import")))
            .is_err()
    );
    assert_eq!(c.snapshot(), before);
    assert!(
        c.prepare(TransactionRequest::new(
            "Retry",
            vec![insert(d, revision, retained)]
        ))
        .is_ok()
    );
    redo_existing(&mut c, d, object);
}

#[test]
fn resource_insert_rejects_aggregate_retention_before_a_candidate_is_returned() {
    let limits = Limits {
        max_bytes: 256 * 1024,
        ..Limits::default()
    };
    let mut doc = Document::new("bounded resources");
    let existing = resource(&vec![1; 20 * 1024]);
    doc.resources.insert(existing.id, existing);
    let entity = Entity::new("existing", Geometry::Point([1., 2., 3.]), None);
    let object = entity.id;
    doc.entities.insert(object, entity);
    let d = doc.id;
    let mut c =
        TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let source = c.world().documents[&d].stamp(object).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([9., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let revision = c.world().documents[&d].revision;
    let incoming = resource(&vec![2; 200 * 1024]);
    assert!(
        incoming.validate(&limits).is_ok(),
        "only aggregate retention exceeds the budget"
    );
    let before = c.snapshot();
    assert!(matches!(
        c.prepare(TransactionRequest::new(
            "Oversized import",
            vec![insert(d, revision, incoming)]
        )),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
    redo_existing(&mut c, d, object);
    let revision = c.world().documents[&d].revision;
    let p = c.prepare(TransactionRequest::new(
        "Small import",
        vec![insert(d, revision, resource(b"small"))],
    ));
    assert!(p.is_ok(), "{p:?}");
    c.commit(p.unwrap()).unwrap();
}

#[test]
fn resource_id_stays_retired_after_undo_and_after_delete() {
    for undo in [true, false] {
        let (mut c, d, _, _) = fixture();
        let retained = resource(b"one resource identity");
        let revision = c.world().documents[&d].revision;
        let p = c
            .prepare(TransactionRequest::new(
                "Insert",
                vec![insert(d, revision, retained.clone())],
            ))
            .unwrap();
        c.commit(p).unwrap();
        if undo {
            c.undo().unwrap();
        } else {
            let revision = c.world().documents[&d].revision;
            let p = c
                .prepare(TransactionRequest::new(
                    "Delete",
                    vec![delete(d, revision, retained.id)],
                ))
                .unwrap();
            c.commit(p).unwrap();
        }
        assert!(!c.world().documents[&d].resources.contains_key(&retained.id));
        let before = c.snapshot();
        let revision = c.world().documents[&d].revision;
        assert!(
            c.prepare(TransactionRequest::new(
                "Reuse",
                vec![insert(d, revision, retained.clone())]
            ))
            .is_err()
        );
        assert_eq!(c.snapshot(), before);
        if undo {
            c.redo().unwrap();
        } else {
            c.undo().unwrap();
        }
        assert_eq!(
            c.world().documents[&d].resources[&retained.id]
                .payload()
                .unwrap(),
            b"one resource identity"
        );
    }
}

#[test]
fn caller_resource_identity_stays_retired_after_savepoint_rollback_and_abort() {
    let (mut c, d, _, object) = seeded_redo();
    let public = c.world().clone();
    let token = c.book("Import scope", true).unwrap();
    c.begin(&token).unwrap();
    c.savepoint(&token).unwrap();
    let retained = resource(b"private accepted resource");
    let revision = c.editing_world(&token).unwrap().documents[&d].revision;
    let mut request = TransactionRequest::new(
        "Private import",
        vec![insert(d, revision, retained.clone())],
    );
    request.owner = Some(token.clone());
    let p = c.prepare(request).unwrap();
    assert!(!c.commit(p).unwrap().published);
    assert_eq!(c.world(), &public);
    assert_eq!(
        c.editing_world(&token).unwrap().documents[&d].resources[&retained.id]
            .payload()
            .unwrap(),
        b"private accepted resource"
    );
    c.rollback_savepoint(&token).unwrap();
    let revision = c.editing_world(&token).unwrap().documents[&d].revision;
    let mut request = TransactionRequest::new(
        "Reuse private ID",
        vec![insert(d, revision, retained.clone())],
    );
    request.owner = Some(token.clone());
    let before = c.snapshot();
    assert!(c.prepare(request).is_err());
    assert_eq!(c.snapshot(), before);
    c.abort(&token).unwrap();
    let revision = c.world().documents[&d].revision;
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Reuse aborted ID",
            vec![insert(d, revision, retained)]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
    redo_existing(&mut c, d, object);
}

#[test]
fn resource_revisions_use_the_initial_batch_snapshot() {
    let (mut c, d, _, object) = fixture();
    let old = resource(b"old source");
    let revision = c.world().documents[&d].revision;
    let p = c
        .prepare(TransactionRequest::new(
            "Insert old",
            vec![insert(d, revision, old.clone())],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let replacement = resource(b"independent new identity");
    let revision = c.world().documents[&d].revision;
    let source = c.world().documents[&d].stamp(object).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Change current state",
            vec![
                delete(d, revision, old.id),
                insert(d, revision, replacement.clone()),
                Mutation::Placement {
                    source,
                    placement: Transform::translation([7., 0., 0.]),
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert_eq!(c.world().documents[&d].revision, revision + 1);
    assert!(!c.world().documents[&d].resources.contains_key(&old.id));
    assert_eq!(
        c.world().documents[&d].resources[&replacement.id]
            .payload()
            .unwrap(),
        b"independent new identity"
    );
    assert_eq!(
        c.world().documents[&d].entities[&object].placement.0[0][3],
        7.
    );
}

#[test]
fn resource_and_object_identity_authority_are_tagged_independently() {
    let (mut c, d, _, _) = fixture();
    let entity = Entity::new("object", Geometry::Point([0.; 3]), None);
    let mut retained = resource(b"same raw UUID different typed identity");
    retained.id = ResourceId(entity.id.0);
    let object = entity.id;
    let revision = c.world().documents[&d].revision;
    let p = c
        .prepare(TransactionRequest::new(
            "Typed IDs",
            vec![
                insert(d, revision, retained.clone()),
                Mutation::Insert {
                    document: d,
                    entity,
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(c.world().documents[&d].entities.contains_key(&object));
    assert!(c.world().documents[&d].resources.contains_key(&retained.id));
}

#[test]
fn initial_resource_id_cannot_be_replaced_or_reallocated_after_deletion() {
    let (mut c, d, other, _) = fixture();
    let original = c.world().documents[&other]
        .resources
        .values()
        .next()
        .unwrap()
        .clone();
    let revision = c.world().documents[&d].revision;
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Reallocate initial identity",
            vec![insert(d, revision, original.clone())]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
    let p = c
        .prepare(TransactionRequest::new(
            "Delete initial resource",
            vec![delete(
                other,
                c.world().documents[&other].revision,
                original.id,
            )],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let revision = c.world().documents[&d].revision;
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "Reallocate deleted initial identity",
            vec![insert(d, revision, original)]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
}

#[test]
fn caller_rejected_resource_batch_keeps_pending_state_and_prior_redo() {
    let (mut c, d, _, object) = seeded_redo();
    let token = c.book("Import", true).unwrap();
    c.begin(&token).unwrap();
    let revision = c.editing_world(&token).unwrap().documents[&d].revision;
    let retained = resource(b"would be private");
    let descriptor: Resource =
        serde_json::from_slice(&serde_json::to_vec(&resource(b"unhydrated")).unwrap()).unwrap();
    let mut request = TransactionRequest::new(
        "Reject entire resource batch",
        vec![
            insert(d, revision, retained.clone()),
            insert(d, revision, descriptor),
        ],
    );
    request.owner = Some(token.clone());
    let before = c.snapshot();
    assert!(c.prepare(request).is_err());
    assert_eq!(c.snapshot(), before);
    let mut valid =
        TransactionRequest::new("ID was not accepted", vec![insert(d, revision, retained)]);
    valid.owner = Some(token.clone());
    assert!(c.prepare(valid).is_ok());
    c.abort(&token).unwrap();
    redo_existing(&mut c, d, object);
}
