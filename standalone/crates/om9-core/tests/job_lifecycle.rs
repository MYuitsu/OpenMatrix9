// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, GeometryKind, Transform},
    jobs::*,
    model::{Document, Entity},
    request::*,
    transaction::*,
};
fn fixture() -> (TransactionCoordinator, AdmittedRequest) {
    let mut d = Document::new("jobs");
    let e = Entity::new("point", Geometry::Point([1., 2., 3.]), d.current_layer);
    let id = e.id;
    d.entities.insert(id, e);
    let binding = SourceBinding {
        role: "Point".into(),
        source: d.stamp(id).unwrap(),
        selection: Selection::Whole,
    };
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let s = RequestSchema {
        feature: "fixture".into(),
        roles: vec![Role {
            name: "Point".into(),
            kinds: vec![GeometryKind::Point],
            min: 1,
            max: 1,
            access: Access::Reference,
            allow_reuse: false,
        }],
        options: Default::default(),
    };
    let r = admit(
        c.world(),
        &s,
        vec![binding],
        Default::default(),
        &Limits::default(),
    )
    .unwrap();
    (c, r)
}
#[test]
fn worker_budget_uses_floor_sixty_percent_without_four_worker_ceiling() {
    for (cpus, tasks, want) in [
        (1, 10, 1),
        (4, 10, 2),
        (8, 10, 4),
        (16, 50, 9),
        (64, 50, 38),
        (64, 2, 2),
        (64, 0, 0),
    ] {
        assert_eq!(plan_workers(cpus, tasks, 1, 1000, 0).unwrap().workers, want);
    }
    assert_eq!(plan_workers(64, 50, 100, 450, 100).unwrap().workers, 3);
    assert!(plan_workers(64, 50, u64::MAX, 1000, 100).is_err());
}
#[test]
fn canceled_and_duplicate_late_replies_never_become_ready() {
    let (c, r) = fixture();
    let mut jobs = JobManager::default();
    let input = jobs.start(&c, r).unwrap();
    input.cancel.cancel();
    let before = c.snapshot();
    assert!(jobs.complete(&input.token, &c, 7u32).is_err());
    assert!(jobs.complete(&input.token, &c, 7u32).is_err());
    assert_eq!(c.snapshot(), before);
}
#[test]
fn commit_then_undo_still_invalidates_job_even_when_source_stamp_matches_again() {
    let (mut c, r) = fixture();
    let source = r.inputs[0].binding.source;
    let mut jobs = JobManager::default();
    let input = jobs.start(&c, r).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "move",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([10., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let before = c.snapshot();
    assert!(jobs.complete(&input.token, &c, 7u32).is_err());
    assert_eq!(c.snapshot(), before);
}
#[test]
fn immutable_worker_input_survives_close_and_reply_is_discarded() {
    let (mut c, r) = fixture();
    let d = r.inputs[0].binding.source.document;
    let mut jobs = JobManager::default();
    let input = jobs.start(&c, r).unwrap();
    c.close_document(d).unwrap();
    jobs.close_document(d);
    let copied = std::thread::spawn(move || {
        assert_eq!(
            input.request.inputs[0].selected_points().unwrap(),
            vec![(0, [1., 2., 3.])]
        );
        input
    })
    .join()
    .unwrap();
    assert!(jobs.complete(&copied.token, &c, 7u32).is_err());
    assert!(c.world().documents.is_empty());
}
#[test]
fn active_savepoints_share_aggregate_budget_with_next_private_clone() {
    let mut d = Document::new("budget");
    let e = Entity::new("point", Geometry::Point([0., 0., 0.]), d.current_layer);
    let id = e.id;
    let did = d.id;
    d.entities.insert(id, e);
    let limits = Limits {
        max_points: 2,
        ..Limits::default()
    };
    let mut c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    let source = c.editing_world(&owner).unwrap().documents[&did]
        .stamp(id)
        .unwrap();
    let mut r = TransactionRequest::new(
        "first",
        vec![Mutation::Placement {
            source,
            placement: Transform::translation([1., 0., 0.]),
        }],
    );
    r.owner = Some(owner.clone());
    let p = c.prepare(r).unwrap();
    c.commit(p).unwrap();
    c.savepoint(&owner).unwrap();
    let before = c.snapshot();
    let source = c.editing_world(&owner).unwrap().documents[&did]
        .stamp(id)
        .unwrap();
    let mut r = TransactionRequest::new(
        "second",
        vec![Mutation::Placement {
            source,
            placement: Transform::translation([2., 0., 0.]),
        }],
    );
    r.owner = Some(owner.clone());
    assert!(c.prepare(r).is_err());
    assert_eq!(c.snapshot(), before);
}
