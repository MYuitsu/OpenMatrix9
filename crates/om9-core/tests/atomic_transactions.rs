// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    brep::{Brep, BrepDisplay},
    budget::Limits,
    geometry::{Geometry, Transform},
    identity::*,
    model::{Document, Entity},
    transaction::*,
};
use uuid::Uuid;

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
fn native_brep_payload_is_shared_across_move_undo_and_redo() {
    use om9_core::geometry::{Mesh, NativeBrep};
    use std::sync::Arc;
    let limits = Limits::default();
    let native = NativeBrep {
        display: Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![] },
        payload: vec![7; 1024 * 1024].into(), crc: 1,
    };
    let source = Arc::clone(&native.payload);
    let mut document = Document::new("native shared payload");
    let entity = Entity::new("native", Geometry::NativeBrep(native), None);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let check = |coordinator: &TransactionCoordinator, x| {
        let entity = &coordinator.world().documents[&document_id].entities[&id];
        let Geometry::NativeBrep(native) = &entity.geometry else { panic!("native B-rep") };
        assert!(Arc::ptr_eq(&native.payload, &source));
        assert_eq!(entity.placement, Transform::translation([x, 0., 0.]));
    };
    check(&coordinator, 0.);
    let prepared = coordinator.prepare(translated(&coordinator, document_id, id, 4.)).unwrap();
    coordinator.commit(prepared).unwrap();
    check(&coordinator, 4.);
    coordinator.undo().unwrap();
    check(&coordinator, 0.);
    coordinator.redo().unwrap();
    check(&coordinator, 4.);
}

#[test]
fn native_brep_undo_inventory_tracks_shared_payload_ownership() {
    use om9_core::geometry::{Mesh, NativeBrep};
    let limits = Limits::default();
    let native = NativeBrep {
        display: Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![] },
        payload: vec![7; 1024 * 1024].into(), crc: 1,
    };
    let mut document = Document::new("native shared undo");
    let entity = Entity::new("native", Geometry::NativeBrep(native), None);
    let id = entity.id;
    let doc = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    for x in 1..=3 {
        let prepared = coordinator.prepare(translated(&coordinator, doc, id, x as f64)).unwrap();
        coordinator.commit(prepared).unwrap();
    }
    assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024);
    let capture = coordinator.capture_tracked_document(doc).unwrap();
    let source = coordinator.world().documents[&doc].stamp(id).unwrap();
    let prepared = coordinator.prepare(TransactionRequest::new("delete", vec![Mutation::Delete { source }])).unwrap();
    coordinator.commit(prepared).unwrap();
    assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024);
    drop(capture);
    let retained = coordinator.undo_memory_bytes().unwrap();
    assert!((1024 * 1024..1536 * 1024).contains(&retained), "history owns one payload: {retained}");
    coordinator.undo().unwrap();
    assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024);
    coordinator.redo().unwrap();
    assert!((1024 * 1024..1536 * 1024).contains(&coordinator.undo_memory_bytes().unwrap()));
}

#[test]
fn native_brep_document_inventory_deduplicates_only_shared_allocations() {
    use om9_core::geometry::{Mesh, NativeBrep};
    let limits = Limits::default();
    let native = NativeBrep {
        display: Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![] },
        payload: vec![7; 1024 * 1024].into(), crc: 1,
    };
    let mut shared = Document::new("payload inventory");
    let first = Entity::new("first", Geometry::NativeBrep(native.clone()), None);
    let second = Entity::new("second", Geometry::NativeBrep(native), None);
    let second_id = second.id;
    shared.entities.insert(first.id, first);
    shared.entities.insert(second.id, second);
    let shared_bytes = shared.validate(&limits).unwrap().bytes;
    let mut independent = shared.clone();
    let Geometry::NativeBrep(brep) = &mut independent.entities.get_mut(&second_id).unwrap().geometry else { panic!("native") };
    brep.payload = (*brep.payload).clone().into();
    let independent_bytes = independent.validate(&limits).unwrap().bytes;
    assert!(independent_bytes - shared_bytes >= 1024 * 1024);
    let bound = Limits { max_bytes: shared_bytes + 256 * 1024, ..limits };
    assert!(shared.validate(&bound).is_ok());
    assert!(independent.validate(&bound).is_err());
    let doc = shared.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![shared], &limits).unwrap(), limits).unwrap();
    let prepared = coordinator.prepare(translated(&coordinator, doc, second_id, 2.)).unwrap();
    coordinator.commit(prepared).unwrap();
    assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024);
    coordinator.undo().unwrap();
    coordinator.redo().unwrap();
}

#[test]
fn external_reservation_blocks_oversized_edit_without_losing_redo() {
    use om9_core::budget::memory::MemoryClass;
    let (mut coordinator, document, _) = seeded_redo();
    let before = coordinator.snapshot();
    let remaining = coordinator.limits().max_bytes - coordinator.retained_usage().unwrap().bytes;
    let reservation = coordinator.reserve_external_memory(MemoryClass::Export, remaining - 64 * 1024, None).unwrap();
    let mesh = om9_core::geometry::Mesh {
        vertices: (0..10_000).map(|i| [f64::from(i), 0., 0.]).collect(),
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let entity = Entity::new("large pending mesh", Geometry::Mesh(mesh), None);
    assert!(matches!(coordinator.prepare(TransactionRequest::new("insert", vec![Mutation::Insert { document, entity }])), Err(Error::Budget(_))));
    assert_eq!(coordinator.snapshot(), before);
    drop(reservation);
    coordinator.redo().unwrap();
}

#[test]
fn external_capture_keeps_old_document_charged_after_real_transaction() {
    let (mut coordinator, document, id) = fixture();
    coordinator.disable_undo_for_benchmark().unwrap();
    let captured = coordinator.capture_retained_document(document).unwrap();
    let p = coordinator.prepare(translated(&coordinator, document, id, 5.)).unwrap();
    coordinator.commit(p).unwrap();
    assert_eq!(coordinator.undo_len(), 0);
    let retained = coordinator.retained_usage().unwrap().bytes;
    drop(captured);
    let released = coordinator.retained_usage().unwrap().bytes;
    assert!(retained > released, "captured={retained}, released={released}");
}

#[test]
fn undo_versions_do_not_spend_current_document_point_limit() {
    let limits = Limits { max_points: 30, ..Limits::default() };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("history points");
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), document.current_layer);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let before_bytes = coordinator.retained_usage().unwrap().bytes;
    for step in 1..=16 {
        let request = translated(&coordinator, document_id, id, f64::from(step));
        let prepared = coordinator.prepare(request).unwrap_or_else(|error| panic!("prepare step {step}: {error}"));
        coordinator.commit(prepared).unwrap();
    }
    assert_eq!(coordinator.undo_len(), 16);
    assert!(coordinator.retained_usage().unwrap().bytes > before_bytes, "history bytes remain charged");
    for _ in 0..16 { coordinator.undo().unwrap(); }
    for _ in 0..16 { coordinator.redo().unwrap(); }
}

#[test]
fn undo_versions_do_not_sum_document_mesh_node_limits() {
    let limits = Limits { max_nodes: 100, ..Limits::default() };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]; 70], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("history nodes");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let bytes_before = coordinator.retained_usage().unwrap().bytes;
    for step in 1..=3 {
        let prepared = coordinator.prepare(translated(&coordinator, document_id, id, f64::from(step))).unwrap();
        coordinator.commit(prepared).unwrap();
    }
    assert_eq!(coordinator.undo_len(), 3);
    assert!(coordinator.retained_usage().unwrap().bytes > bytes_before);
    for _ in 0..3 { coordinator.undo().unwrap(); }
    for _ in 0..3 { coordinator.redo().unwrap(); }
}

#[test]
fn adding_faces_beyond_one_document_node_limit_preserves_redo() {
    let limits = Limits { max_nodes: 100, ..Limits::default() };
    let mesh = |faces| om9_core::geometry::Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]; faces], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("node ceiling");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh(70)), None);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let prepared = coordinator.prepare(translated(&coordinator, document_id, id, 1.)).unwrap();
    coordinator.commit(prepared).unwrap();
    coordinator.undo().unwrap();
    let before = coordinator.snapshot();
    let other = Entity::new("over limit", Geometry::Mesh(mesh(31)), None);
    let result = coordinator.prepare(TransactionRequest::new("too many faces", vec![Mutation::Insert { document: document_id, entity: other }]));
    assert!(matches!(result, Err(Error::Budget("nodes"))));
    assert_eq!(coordinator.snapshot(), before);
    assert_eq!(coordinator.redo_len(), 1);
    coordinator.redo().unwrap();
}

#[test]
fn separate_documents_do_not_consume_each_others_mesh_node_and_point_limits() {
    let limits = Limits { max_nodes: 100, max_points: 10, ..Limits::default() };
    let create = |name| {
        let mut document = Document::new(name);
        let mesh = om9_core::geometry::Mesh {
            vertices: (0..7).map(|i| [f64::from(i), 0., 0.]).collect(),
            faces: vec![vec![0,1,2]; 70], normals: vec![], colors: vec![], corner_uvs: vec![],
        };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        let document_id = document.id;
        document.entities.insert(id, entity);
        (document, document_id, id)
    };
    let (first, document_id, id) = create("first");
    let (second, _, _) = create("second");
    let world = World::new(vec![first.clone(), second.clone()], &limits).expect("each document is within its own structural limits");
    let total_bytes = world.validate(&limits).unwrap().bytes;
    let tight = Limits { max_bytes: total_bytes - 1, ..limits };
    assert!(matches!(World::new(vec![first, second], &tight), Err(Error::Budget("bytes"))),
        "both documents must still consume the shared byte budget");
    let mut coordinator = TransactionCoordinator::new(world, limits).unwrap();
    let before_bytes = coordinator.retained_usage().unwrap().bytes;
    let prepared = coordinator.prepare(translated(&coordinator, document_id, id, 1.)).unwrap();
    coordinator.commit(prepared).unwrap();
    assert!(coordinator.retained_usage().unwrap().bytes > before_bytes);
    coordinator.undo().unwrap();
    coordinator.redo().unwrap();
}

#[test]
fn single_document_still_cannot_exceed_point_limit() {
    let limits = Limits { max_points: 30, ..Limits::default() };
    let mesh = om9_core::geometry::Mesh {
        vertices: (0..31).map(|i| [f64::from(i), 0., 0.]).collect(),
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("over point limit");
    let entity = Entity::new("oversized", Geometry::Mesh(mesh), document.current_layer);
    document.entities.insert(entity.id, entity);
    assert!(matches!(World::new(vec![document], &limits), Err(Error::Budget("points"))));
}

#[test]
fn moving_a_document_near_point_limit_does_not_double_count_candidate() {
    let limits = Limits { max_points: 30, ..Limits::default() };
    let mesh = om9_core::geometry::Mesh {
        vertices: (0..20).map(|i| [f64::from(i), f64::from(i % 3), 0.]).collect(),
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("near point limit");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), document.current_layer);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let prepared = coordinator.prepare(translated(&coordinator, document_id, id, 1.)).unwrap();
    coordinator.commit(prepared).unwrap();
    coordinator.undo().unwrap();
    coordinator.redo().unwrap();
}

#[test]
fn editing_cannot_grow_one_document_past_point_limit() {
    let limits = Limits { max_points: 30, ..Limits::default() };
    let mesh = om9_core::geometry::Mesh {
        vertices: (0..29).map(|i| [f64::from(i), f64::from(i % 3), 0.]).collect(),
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("point growth");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), document.current_layer);
    let document_id = document.id;
    document.entities.insert(entity.id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    for step in 0..2 {
        let entity = Entity::new("point", Geometry::Point([f64::from(step), 0., 0.]), None);
        let before = coordinator.snapshot();
        let request = TransactionRequest::new("insert", vec![Mutation::Insert { document: document_id, entity }]);
        let result = coordinator.prepare(request);
        if step == 0 { coordinator.commit(result.unwrap()).unwrap(); }
        else {
            assert!(matches!(result, Err(Error::Budget("points"))));
            assert_eq!(coordinator.snapshot(), before);
        }
    }
}

#[test]
fn malformed_brep_is_rejected_without_losing_existing_redo() {
    let limits = Limits::default();
    let mut brep = Brep::from_native(
        "CASCADE Topology fixture".into(),
        BrepDisplay {
            source_sha256: [0; 32],
            deflection_mm: 0.01,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]],
            segments: vec![],
            vertices: vec![],
            faces: vec![Uuid::from_u128(1)],
            edges: vec![],
            vertex_ids: vec![],
            face_edges: vec![],
        },
        &limits,
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut brep.display).points[0][0] = 2.;
    let (mut c, d, _) = seeded_redo();
    let before = c.snapshot();
    let invalid = Entity::new("tampered", Geometry::Brep(brep), None);
    assert!(c.prepare(TransactionRequest::new(
        "invalid",
        vec![Mutation::Insert {
            document: d,
            entity: invalid,
        }],
    )).is_err());
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
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
