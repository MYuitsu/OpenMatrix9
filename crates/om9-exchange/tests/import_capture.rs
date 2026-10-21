// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
    retained::{
        HostRef, ImportBundle, ImportNamespace, NativeRecordKey, NativeRecordKind, NativeUnits,
        ResolvedBy, RetainedSource, SourceRecord, SourceUnits, fingerprint,
    },
    transaction::*,
};
use om9_exchange::ImportCapture;
use std::sync::Arc;

fn fixture() -> (TransactionCoordinator, DocumentId, ObjectId) {
    let mut document = Document::new("import target");
    let entity = Entity::new("existing point", Geometry::Point([1., 2., 3.]), None);
    let object = entity.id;
    document.entities.insert(object, entity);
    let id = document.id;
    (
        TransactionCoordinator::new(
            World::new(vec![document], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        id,
        object,
    )
}
fn bundle() -> ImportBundle {
    let resource = Resource::from_bytes(
        "authored.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored ownership fixture; no native correspondence claimed",
        &Limits::default(),
    )
    .unwrap();
    let unit = NativeUnits {
        native_enum: 2,
        name: "millimeters".into(),
        declared_mm_per_unit: Some(1.),
        millimeters_per_unit: 1.,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.001,
        angular_tolerance_radians: 0.001,
        relative_tolerance: 0.001,
    };
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 50,
        units: SourceUnits {
            model: unit.clone(),
            page: unit,
            definitions: Default::default(),
        },
    };
    let entity = Entity::new("imported point", Geometry::Point([4., 5., 6.]), None);
    let record = SourceRecord::bound(
        NativeRecordKey {
            kind: NativeRecordKind::Geometry,
            id: uuid::Uuid::new_v4(),
            native_index: None,
        },
        "ON_Point",
        HostRef::Object(entity.id),
        fingerprint(&entity).unwrap(),
    );
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: None,
        records: vec![record],
    };
    ImportBundle {
        resource,
        source,
        namespace,
        entities: vec![entity],
        layers: vec![],
        definitions: vec![],
        groups: vec![],
        exchange: Default::default(),
    }
}
fn move_point(
    c: &TransactionCoordinator,
    id: DocumentId,
    object: ObjectId,
    owner: Option<&ScopeToken>,
) -> PreparedTransaction {
    let world = owner.map_or(c.world(), |token| c.editing_world(token).unwrap());
    let mut request = TransactionRequest::new(
        "Move",
        vec![Mutation::Placement {
            source: world.documents[&id].stamp(object).unwrap(),
            placement: Transform::translation([7., 0., 0.]),
        }],
    );
    request.owner = owner.cloned();
    c.prepare(request).unwrap()
}

#[test]
fn equal_coordinator_with_shared_target_arc_cannot_borrow_import_capture() {
    let (first, id, _) = fixture();
    let capture = ImportCapture::target(&first, id).unwrap();
    let second = TransactionCoordinator::new(first.world().clone(), Limits::default()).unwrap();
    assert!(Arc::ptr_eq(
        &first.world().documents[&id],
        &second.world().documents[&id]
    ));
    assert_eq!(first.epoch(), second.epoch());
    assert!(capture.revalidate(&first).is_ok());
    assert!(matches!(
        capture.revalidate(&second),
        Err(Error::WrongOwner)
    ));
    assert!(
        capture
            .prepare(&second, bundle(), &Cancellation::default())
            .is_err()
    );
}
#[test]
fn undo_that_restores_the_exact_original_arc_cannot_revive_a_capture() {
    let (mut c, id, object) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let original = c.world().documents[&id].clone();
    let p = move_point(&c, id, object, None);
    c.commit(p).unwrap();
    c.undo().unwrap();
    assert!(Arc::ptr_eq(&original, &c.world().documents[&id]));
    assert!(capture.revalidate(&c).is_err());
}
#[test]
fn plain_capture_rejects_existing_scope_and_completed_caller_rollback() {
    let (mut c, id, _) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let epoch = c.epoch();
    let owner = c.book("caller", true).unwrap();
    assert!(ImportCapture::target(&c, id).is_err());
    assert!(ImportCapture::for_scope(&c, id, &owner).is_err());
    c.begin(&owner).unwrap();
    c.savepoint(&owner).unwrap();
    c.rollback_savepoint(&owner).unwrap();
    c.abort(&owner).unwrap();
    assert_eq!(c.epoch(), epoch);
    assert!(capture.revalidate(&c).is_err());
}
#[test]
fn scoped_capture_observes_savepoint_push_and_rollback_even_without_geometry_changes() {
    let (mut c, id, _) = fixture();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    let before = ImportCapture::for_scope(&c, id, &owner).unwrap();
    c.savepoint(&owner).unwrap();
    assert!(before.revalidate(&c).is_err());
    let inside = ImportCapture::for_scope(&c, id, &owner).unwrap();
    c.rollback_savepoint(&owner).unwrap();
    assert!(inside.revalidate(&c).is_err());
    let after = ImportCapture::for_scope(&c, id, &owner).unwrap();
    c.abort(&owner).unwrap();
    assert!(after.revalidate(&c).is_err());
}
#[test]
fn scoped_import_respects_caller_recording_and_publishes_only_at_finish() {
    for recording in [true, false] {
        let (mut c, id, _) = fixture();
        let before = c.world().clone();
        let owner = c.book("caller", recording).unwrap();
        c.begin(&owner).unwrap();
        let capture = ImportCapture::for_scope(&c, id, &owner).unwrap();
        let bundle = bundle();
        let imported = bundle.entities[0].id;
        let source = bundle.source.id;
        let p = capture
            .prepare(&c, bundle, &Cancellation::default())
            .unwrap();
        assert!(p.belongs_to(&owner));
        assert!(!c.commit(p).unwrap().published);
        assert_eq!(c.world(), &before);
        assert_eq!(c.undo_len(), 0);
        c.finish(&owner).unwrap();
        assert!(c.world().documents[&id].entities.contains_key(&imported));
        assert!(
            c.world().documents[&id]
                .retained_sources
                .contains_key(&source)
        );
        assert_eq!(c.undo_len(), usize::from(recording));
        if recording {
            c.undo().unwrap();
            assert_eq!(c.world(), &before);
            c.redo().unwrap();
            assert!(c.world().documents[&id].entities.contains_key(&imported));
        }
    }
}
#[test]
fn original_caller_thread_is_required_for_scoped_capture() {
    let (mut c, id, _) = fixture();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    let capture = ImportCapture::for_scope(&c, id, &owner).unwrap();
    assert!(matches!(
        std::thread::spawn(move || capture.revalidate(&c))
            .join()
            .unwrap(),
        Err(Error::WrongOwner)
    ));
}
#[test]
fn close_reopen_of_the_same_persistent_document_does_not_revive_capture() {
    let (mut c, id, _) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let document = (*c.world().documents[&id]).clone();
    c.close_document(id).unwrap();
    c.open_document(document).unwrap();
    assert!(capture.revalidate(&c).is_err());
    assert_eq!(capture.document().id, id);
}
#[test]
fn late_cancel_and_public_change_reject_prepared_import_without_partial_state() {
    let (mut c, id, object) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let cancel = Cancellation::default();
    let p = capture.prepare(&c, bundle(), &cancel).unwrap();
    cancel.cancel();
    let before = c.snapshot();
    assert_eq!(c.commit(p), Err(Error::Canceled));
    assert_eq!(c.snapshot(), before);
    let capture = ImportCapture::target(&c, id).unwrap();
    let p = capture
        .prepare(&c, bundle(), &Cancellation::default())
        .unwrap();
    let change = move_point(&c, id, object, None);
    c.commit(change).unwrap();
    let before = c.snapshot();
    assert!(c.commit(p).is_err());
    assert_eq!(c.snapshot(), before);
}
#[test]
fn import_prepare_reserves_external_receipt_memory_without_touching_target() {
    let (c, id, _) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let before = c.snapshot();
    assert!(matches!(
        capture.prepare_with_held_usage(
            &c,
            bundle(),
            &Cancellation::default(),
            Usage {
                bytes: c.limits().max_bytes,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn acceptance_guard_can_revalidate_the_frozen_target_without_mutation_privilege() {
    let (mut c, id, _) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let prepared = capture
        .prepare(&c, bundle(), &Cancellation::default())
        .unwrap();
    let receipt = c
        .commit_with_guard(prepared, |coordinator| {
            assert_eq!(coordinator.close_document(id), Err(Error::Busy));
            capture.revalidate(coordinator)
        })
        .unwrap();
    assert!(receipt.published);
}

#[test]
fn scoped_capture_reads_private_target_and_release_invalidates_its_lifecycle() {
    let (mut c, id, object) = fixture();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    let change = move_point(&c, id, object, Some(&owner));
    c.commit(change).unwrap();
    let capture = ImportCapture::for_scope(&c, id, &owner).unwrap();
    assert_eq!(capture.document().entities[&object].placement.0[0][3], 7.);
    assert_eq!(
        c.world().documents[&id].entities[&object].placement.0[0][3],
        0.
    );
    c.savepoint(&owner).unwrap();
    let capture = ImportCapture::for_scope(&c, id, &owner).unwrap();
    c.release_savepoint(&owner).unwrap();
    assert!(capture.revalidate(&c).is_err());
}

#[test]
fn caller_abort_after_preparation_rejects_a_late_global_import() {
    let (mut c, id, _) = fixture();
    let capture = ImportCapture::target(&c, id).unwrap();
    let prepared = capture
        .prepare(&c, bundle(), &Cancellation::default())
        .unwrap();
    let owner = c.book("intervening caller", false).unwrap();
    c.begin(&owner).unwrap();
    c.abort(&owner).unwrap();
    let before = c.snapshot();
    assert!(c.commit(prepared).is_err());
    assert_eq!(c.snapshot(), before);
}



