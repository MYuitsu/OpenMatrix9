// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::{Limits, Usage}, geometry::{Geometry, Transform},
    model::{Document, Entity}, transaction::{Mutation, TransactionCoordinator, TransactionRequest, World}};

fn fixture() -> (TransactionCoordinator, om9_core::identity::DocumentId, om9_core::identity::ObjectId) {
    let limits = Limits::default();
    let mut document = Document::new("storage receipts");
    let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let doc = document.id;
    (TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap(), doc, id)
}

#[test]
fn immutable_owner_bound_defers_capture_censuses_without_changing_full_undo() {
    let (mut coordinator, doc, id) = fixture();
    for x in 1..=48 {
        let capture = coordinator.capture_tracked_document(doc).unwrap();
        let source = coordinator.world().documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("Move", vec![Mutation::Placement {
            source, placement: Transform::translation([x as f64, 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        drop(capture);
    }
    let stats = coordinator.undo_memory_stats();
    assert!(stats.deferred_attribution_refreshes >= 32, "{stats:?}");
    assert!(stats.full_inventories < 4, "{stats:?}");
    assert!(stats.capture_change_pending);
    assert!(coordinator.undo_memory_bytes().unwrap() <= coordinator.undo_policy().max_memory_mb * 1024 * 1024);
    for _ in 0..48 { coordinator.undo().unwrap(); }
    assert_eq!(coordinator.world().documents[&doc].entities[&id].placement, Transform::identity());
    for _ in 0..48 { coordinator.redo().unwrap(); }
    assert_eq!(coordinator.undo_len(), 48);
    assert_eq!(coordinator.world().documents[&doc].entities[&id].placement, Transform::translation([48., 0., 0.]));
}

#[test]
fn peer_startup_preserves_private_held_admission_and_parent_history() {
    let (coordinator, _, _) = fixture();
    let before = coordinator.snapshot();
    let usage = coordinator.retained_usage().unwrap();
    assert!(coordinator.memory_admission_peer_with_held(Usage {
        bytes: coordinator.limits().max_bytes, ..Usage::default()
    }).is_err());
    assert_eq!(coordinator.snapshot(), before);
    assert_eq!(coordinator.retained_usage().unwrap(), usage);
    let peer = coordinator.memory_admission_peer_with_held(Usage { bytes: 4096, ..Usage::default() }).unwrap();
    assert_eq!(coordinator.snapshot(), before);
    let live = coordinator.retained_usage().unwrap();
    drop(peer);
    let released = coordinator.retained_usage().unwrap();
    assert!(released.bytes < live.bytes);
    // Initializing the shared weak-owner registry can retain index capacity.
    assert_eq!(Usage { bytes: 0, ..released }, Usage { bytes: 0, ..usage });
    assert_eq!(coordinator.retained_usage().unwrap(), released);
}

#[test]
fn warm_receipts_keep_exact_totals_across_undo_redo_and_branch() {
    let (mut coordinator, doc, id) = fixture();
    for x in 1..=8 {
        let source = coordinator.world().documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("Move", vec![Mutation::Placement {
            source, placement: Transform::translation([x as f64, 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let first = coordinator.retained_usage().unwrap();
        for _ in 0..4 { assert_eq!(coordinator.retained_usage().unwrap(), first); }
    }
    let total = coordinator.retained_usage().unwrap();
    coordinator.undo().unwrap();
    coordinator.redo().unwrap();
    assert_eq!(coordinator.world().documents[&doc].entities[&id].placement, Transform::translation([8., 0., 0.]));
    let restored = coordinator.retained_usage().unwrap();
    // Journal capacities may grow on transfer; payload/object counters do not.
    assert_eq!(restored.objects, total.objects);
    assert_eq!(restored.points, total.points);
    coordinator.undo().unwrap();
    let source = coordinator.world().documents[&doc].stamp(id).unwrap();
    let branch = coordinator.prepare(TransactionRequest::new("Branch", vec![Mutation::Placement {
        source, placement: Transform::translation([99., 0., 0.]),
    }])).unwrap();
    coordinator.commit(branch).unwrap();
    assert!(coordinator.redo().is_err());
    assert_eq!(coordinator.world().documents[&doc].entities[&id].placement, Transform::translation([99., 0., 0.]));
    let first = coordinator.retained_usage().unwrap();
    assert_eq!(coordinator.retained_usage().unwrap(), first);
}

#[test]
fn warm_storage_does_not_bypass_changed_held_budget_or_rollback() {
    let (mut coordinator, doc, id) = fixture();
    let baseline = coordinator.retained_usage().unwrap();
    assert!(coordinator.retained_usage_with_held_usage(Usage { bytes: coordinator.limits().max_bytes, ..Usage::default() }).is_err());
    assert_eq!(coordinator.retained_usage().unwrap(), baseline);
    let source = coordinator.world().documents[&doc].stamp(id).unwrap();
    let before = coordinator.world().clone();
    let result = coordinator.prepare_with_held_usage(TransactionRequest::new("Refused", vec![Mutation::Placement {
        source, placement: Transform::translation([1., 0., 0.]),
    }]), Usage { bytes: coordinator.limits().max_bytes, ..Usage::default() });
    assert!(result.is_err());
    assert_eq!(coordinator.world(), &before);
    assert_eq!(coordinator.retained_usage().unwrap(), baseline);
    assert!(coordinator.undo().is_err());
}

#[test]
fn concurrent_readers_and_tighter_budget_do_not_reuse_a_stale_admission() {
    let (coordinator, _, _) = fixture();
    let warm = coordinator.retained_usage().unwrap();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let c = &coordinator;
            scope.spawn(move || {
                for _ in 0..8 {
                    assert_eq!(c.retained_usage().unwrap(), warm);
                    assert!(c.retained_usage_with_held_usage(Usage {
                        bytes: c.limits().max_bytes, ..Usage::default()
                    }).is_err());
                }
            });
        }
    });
    assert_eq!(coordinator.retained_usage().unwrap(), warm);
}

#[test]
fn an_untracked_external_arc_expiring_invalidates_its_cached_payload_charge() {
    use om9_core::resource::{Resource, ResourceKind};
    use std::sync::Arc;
    let (coordinator, _, _) = fixture();
    let mut candidate = Document::new("external payload");
    let payload = vec![19; 1024 * 1024];
    let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, coordinator.limits()).unwrap();
    candidate.resources.insert(resource.id, resource);
    let candidate = Arc::new(candidate);
    coordinator.register_external_candidate(&candidate).unwrap();
    let live = coordinator.retained_usage().unwrap();
    assert_eq!(coordinator.retained_usage().unwrap(), live);
    drop(candidate); // This API has no CapturedDocument lease generation bump.
    let expired = coordinator.retained_usage().unwrap();
    assert!(live.bytes - expired.bytes >= payload.len() as u64);
    assert_eq!(coordinator.retained_usage().unwrap(), expired);
}

#[test]
fn external_document_counter_maxima_are_rechecked_under_a_tighter_held_budget() {
    use std::sync::Arc;
    let (coordinator, _, _) = fixture();
    let mut candidate = Document::new("external counters");
    for n in 0..3 {
        let entity = Entity::new("external point", Geometry::Point([n as f64, 0., 0.]), None);
        candidate.entities.insert(entity.id, entity);
    }
    let candidate = Arc::new(candidate);
    coordinator.register_external_candidate(&candidate).unwrap();
    let warm = coordinator.retained_usage().unwrap();
    assert_eq!(coordinator.retained_usage().unwrap(), warm);
    // Internal current document fits two objects/points. The external document
    // needs three, even though external objects are not added to the aggregate.
    assert!(coordinator.retained_usage_with_held_usage(Usage {
        objects: coordinator.limits().max_objects - 2, ..Usage::default()
    }).is_err());
    assert!(coordinator.retained_usage_with_held_usage(Usage {
        points: coordinator.limits().max_points - 2, ..Usage::default()
    }).is_err());
    assert_eq!(coordinator.retained_usage().unwrap(), warm);
}

#[test]
fn admission_peers_observe_reclaim_and_release_the_primary_projection_charge() {
    let (mut parent, doc, id) = fixture();
    let peer = parent.memory_admission_peer().unwrap();
    let initial = peer.memory_diagnostics(Ok(Usage::default()), None).projection_cache_accounted_bytes;
    for n in 1..=12 {
        let source = parent.world().documents[&doc].stamp(id).unwrap();
        let prepared = parent.prepare(TransactionRequest::new("Move", vec![Mutation::Placement {
            source, placement: Transform::translation([n as f64, 0., 0.]),
        }])).unwrap();
        parent.commit(prepared).unwrap();
    }
    parent.retained_usage().unwrap();
    let grown = peer.memory_diagnostics(Ok(Usage::default()), None).projection_cache_accounted_bytes;
    assert!(grown > initial, "peer missed newly allocated primary cache storage");
    let world = parent.world().clone();
    let history = parent.undo_len();
    assert!(peer.reclaim_optional_memory(u64::MAX).unwrap() >= grown);
    assert_eq!(peer.memory_diagnostics(Ok(Usage::default()), None).projection_cache_accounted_bytes, 0);
    assert_eq!(parent.world(), &world);
    assert_eq!(parent.undo_len(), history);
    parent.retained_usage().unwrap();
    assert!(peer.memory_diagnostics(Ok(Usage::default()), None).projection_cache_accounted_bytes > 0);
    drop(parent);
    assert_eq!(peer.memory_diagnostics(Ok(Usage::default()), None).projection_cache_accounted_bytes, 0);
}

#[test]
fn account_adoption_retires_private_cache_owners_before_joining_the_shared_account() {
    let (previous, _, _) = fixture();
    previous.retained_usage().unwrap();
    let limits = Limits::default();
    let document = Document::new("replacement");
    let id = document.id;
    let mut replacement = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    replacement.retained_usage().unwrap();
    replacement.adopt_external_captures(&previous).unwrap();
    assert!(replacement.world().documents.contains_key(&id));
    let usage = replacement.retained_usage().unwrap();
    assert_eq!(replacement.retained_usage().unwrap(), usage);
    assert_eq!(previous.world().documents.len(), 1);
    assert_eq!(previous.undo_len(), 0);
}

#[test]
fn shared_and_distinct_brep_allocations_match_debug_storage_oracle() {
    use om9_core::brep::{Brep, BrepDisplay, BrepTopology};
    for distinct in [false, true] {
        let limits = Limits::default();
        let shape = |index| Brep::from_native(format!("CASCADE Topology {index}"), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]], segments: vec![], vertices: vec![],
            topology: BrepTopology { faces: vec![uuid::Uuid::from_u128(1)], edges: vec![], vertex_ids: vec![], face_edges: vec![] }.into(),
        }, &limits).unwrap().into_canonical_and_representation().0;
        let shared = shape(0);
        let mut document = Document::new("BRep receipts");
        let mut first = None;
        for n in 0..16 {
            let entity = Entity::new("BRep", Geometry::Brep(if distinct { shape(n) } else { shared.clone() }), None);
            first.get_or_insert(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let doc = document.id;
        let id = first.unwrap();
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        for n in 1..=16 {
            let source = coordinator.world().documents[&doc].stamp(id).unwrap();
            let prepared = coordinator.prepare(TransactionRequest::new("Move", vec![Mutation::Placement {
                source, placement: Transform::translation([n as f64, 0., 0.]),
            }])).unwrap();
            coordinator.commit(prepared).unwrap();
            let usage = coordinator.retained_usage().unwrap();
            // In debug, every cache hit is also checked against the original
            // allocation-free or indexed document storage implementation.
            assert_eq!(coordinator.retained_usage().unwrap(), usage);
        }
        for _ in 0..16 { coordinator.undo().unwrap(); coordinator.retained_usage().unwrap(); }
        for _ in 0..16 { coordinator.redo().unwrap(); coordinator.retained_usage().unwrap(); }
        assert_eq!(coordinator.world().documents[&doc].entities[&id].placement, Transform::translation([16., 0., 0.]));
    }
}
