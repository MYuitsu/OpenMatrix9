// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh, Transform},
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, UndoPolicy, World},
};

fn fixture(
    vertices: usize,
) -> (
    TransactionCoordinator,
    om9_core::identity::DocumentId,
    om9_core::identity::ObjectId,
) {
    let mut d = Document::new("Undo memory");
    let mut points = vec![[0., 0., 0.]; vertices.max(3)];
    points[1] = [1., 0., 0.];
    points[2] = [0., 1., 0.];
    let e = Entity::new(
        "mesh",
        Geometry::Mesh(Mesh {
            vertices: points,
            faces: vec![vec![0, 1, 2]],
            normals: vec![],
            colors: vec![],
            corner_uvs: vec![],
        }),
        None,
    );
    let id = e.id;
    d.entities.insert(id, e);
    let document = d.id;
    (
        TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        document,
        id,
    )
}
fn move_to(
    c: &mut TransactionCoordinator,
    d: om9_core::identity::DocumentId,
    id: om9_core::identity::ObjectId,
    x: f64,
) {
    let source = c.world().documents[&d].stamp(id).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([x, 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
}

#[test]
fn retained_admission_counts_a_live_shared_payload_once_across_real_undo_history() {
    use om9_core::resource::{Resource, ResourceKind};
    let limits = Limits::default();
    let bytes = vec![11_u8; 4 * 1024 * 1024];
    let resource = Resource::from_bytes("shared", ResourceKind::OpaqueBinary, &bytes, &limits).unwrap();
    let mut document = Document::new("shared resource history");
    document.resources.insert(resource.id, resource);
    let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let baseline = coordinator.retained_usage().unwrap().bytes;
    for x in [1., 2., 3.] { move_to(&mut coordinator, document_id, id, x); }
    let retained = coordinator.retained_usage().unwrap().bytes;
    assert!(retained - baseline < bytes.len() as u64 / 2, "shared payload counted again: baseline={baseline}, retained={retained}");
    coordinator.undo().unwrap();
    assert_eq!(coordinator.world().documents[&document_id].entities[&id].placement.0[0][3], 2.);
    coordinator.redo().unwrap();
    assert_eq!(coordinator.world().documents[&document_id].entities[&id].placement.0[0][3], 3.);
    let after_redo = coordinator.retained_usage().unwrap().bytes;
    assert!(after_redo.abs_diff(retained) < 64 * 1024,
        "Undo/Redo journal capacity may grow, but the payload must not be charged again");
}

#[test]
fn oldest_steps_are_pruned_by_bytes_and_the_remaining_chain_undoes_and_redoes() {
    let (mut c, d, id) = fixture(14000);
    c.set_undo_policy(UndoPolicy {
        minimum_steps: 1,
        max_memory_mb: 1,
    })
    .unwrap();
    for x in 1..=8 {
        move_to(&mut c, d, id, x as f64);
    }
    let kept = c.undo_len();
    assert!(kept > 0 && kept < 8);
    assert!(c.undo_memory_bytes().unwrap() <= 1_048_576);
    for _ in 0..kept {
        c.undo().unwrap();
    }
    assert_eq!(
        c.world().documents[&d].entities[&id].placement.0[0][3],
        (8 - kept) as f64
    );
    assert!(c.undo().is_err());
    for _ in 0..kept {
        c.redo().unwrap();
    }
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 8.);
}

#[test]
fn minimum_steps_win_even_when_they_exceed_the_memory_budget() {
    let (mut c, d, id) = fixture(24000);
    c.set_undo_policy(UndoPolicy {
        minimum_steps: 4,
        max_memory_mb: 1,
    })
    .unwrap();
    for x in 1..=7 {
        move_to(&mut c, d, id, x as f64);
    }
    assert_eq!(c.undo_len(), 4);
    assert!(c.undo_memory_bytes().unwrap() > 1_048_576);
}

#[test]
fn changing_policy_preserves_current_model_epoch_and_redo_and_failed_edits_preserve_history() {
    let (mut c, d, id) = fixture(14000);
    for x in 1..=7 {
        move_to(&mut c, d, id, x as f64);
    }
    c.undo().unwrap();
    let model = c.world().clone();
    let epoch = c.epoch();
    c.set_undo_policy(UndoPolicy {
        minimum_steps: 1,
        max_memory_mb: 1,
    })
    .unwrap();
    assert_eq!(c.world(), &model);
    assert_eq!(c.epoch(), epoch);
    assert_eq!(c.redo_len(), 1);
    let saved = c.snapshot();
    let source = c.world().documents[&d].stamp(id).unwrap();
    assert!(
        c.prepare(TransactionRequest::new(
            "invalid",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([f64::NAN, 0., 0.])
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), saved);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&d].entities[&id].placement.0[0][3], 7.);
}

#[test]
fn there_is_no_256_step_cap_and_invalid_preferences_are_atomic() {
    let (mut c, d, id) = fixture(3);
    for x in 1..=260 {
        move_to(&mut c, d, id, x as f64);
    }
    assert_eq!(c.undo_len(), 260);
    let saved = c.snapshot();
    for p in [
        UndoPolicy {
            minimum_steps: 0,
            max_memory_mb: 1,
        },
        UndoPolicy {
            minimum_steps: 1,
            max_memory_mb: 0,
        },
        UndoPolicy {
            minimum_steps: 1,
            max_memory_mb: u64::MAX,
        },
    ] {
        assert!(c.set_undo_policy(p).is_err());
        assert_eq!(c.snapshot(), saved);
    }
}

#[test]
fn shared_embedded_payload_is_not_charged_per_snapshot_or_when_live() {
    use om9_core::resource::{Resource, ResourceKind};
    let (mut c, d, id) = fixture(3);
    let resource = Resource::from_bytes(
        "shared",
        ResourceKind::OpaqueBinary,
        &vec![7; 2 * 1_048_576],
        &Limits::default(),
    )
    .unwrap();
    let rid = resource.id;
    let p = c
        .prepare(TransactionRequest::new(
            "resource",
            vec![Mutation::ResourceInsert {
                document: d,
                expected_revision: c.world().documents[&d].revision,
                resource,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    for x in 1..=10 {
        move_to(&mut c, d, id, x as f64);
    }
    assert!(
        c.undo_memory_bytes().unwrap() < 1_048_576,
        "payload needed by live document must not be charged per Undo step"
    );
    let p = c
        .prepare(TransactionRequest::new(
            "delete resource",
            vec![Mutation::ResourceDelete {
                document: d,
                expected_revision: c.world().documents[&d].revision,
                resource: rid,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let bytes = c.undo_memory_bytes().unwrap();
    assert!(
        (2 * 1_048_576..3 * 1_048_576).contains(&bytes),
        "shared history-only payload counted more than once: {bytes}"
    );
}

#[test]
fn configurable_budget_is_limited_to_half_physical_ram() {
    let (mut c, _, _) = fixture(3);
    c.set_undo_ram_capacity(16 * 1024 * 1024 * 1024).unwrap();
    assert_eq!(c.undo_policy().max_memory_mb, 1024);
    assert_eq!(c.undo_ram_limit_mb(), Some(8192));
    let old = c.undo_policy();
    assert!(
        c.set_undo_policy(UndoPolicy {
            minimum_steps: 1,
            max_memory_mb: 8193
        })
        .is_err()
    );
    assert_eq!(c.undo_policy(), old);
    c.set_undo_policy(UndoPolicy {
        minimum_steps: 1,
        max_memory_mb: 8192,
    })
    .unwrap();
    c.set_undo_ram_capacity(1024 * 1024 * 1024).unwrap();
    assert_eq!(c.undo_policy().max_memory_mb, 512);
    assert_eq!(c.undo_ram_limit_mb(), Some(512));
}

#[test]
fn complete_inventory_runs_at_100mb_growth_without_scanning_every_commit() {
    use om9_core::resource::{Resource, ResourceKind};
    let (mut c, d, id) = fixture(3);
    let resource = Resource::from_bytes(
        "shared-growth",
        ResourceKind::OpaqueBinary,
        &vec![1; 8 * 1_048_576],
        &Limits::default(),
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "resource",
            vec![Mutation::ResourceInsert {
                document: d,
                expected_revision: c.world().documents[&d].revision,
                resource,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.set_undo_policy(UndoPolicy::default()).unwrap();
    let inventories = c.undo_memory_stats().full_inventories;
    for x in 1..=12 {
        move_to(&mut c, d, id, x as f64);
        let s = c.undo_memory_stats();
        assert_eq!(
            s.full_inventories, inventories,
            "unexpected whole-journal inventory below 100 MB"
        );
        assert!(
            s.estimated_bytes >= c.undo_memory_bytes().unwrap(),
            "interim accounting must be conservative"
        );
    }
    let model = c.world().clone();
    move_to(&mut c, d, id, 13.);
    assert_eq!(c.undo_memory_stats().full_inventories, inventories + 1);
    assert_eq!(c.undo_memory_stats().growth_since_inventory, 0);
    assert_eq!(
        c.undo_memory_stats().estimated_bytes,
        c.undo_memory_bytes().unwrap()
    );
    assert_eq!(
        c.world().documents[&d].entities[&id].geometry,
        model.documents[&d].entities[&id].geometry
    );
}
