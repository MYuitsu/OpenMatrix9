use openmatrix9_rust::phase2_snap::*;
fn key() -> ViewKey {
    ViewKey {
        document: 1,
        view: 2,
        generation: 3,
        width: 1000,
        height: 800,
        camera: [0.; 16],
    }
}
fn limits() -> Limits {
    Limits {
        objects: 64,
        per_object: 2048,
        total: 8192,
    }
}
fn index(n: usize) -> SnapIndex {
    let mut i = SnapIndex::default();
    i.begin_build(key()).unwrap();
    for object in 1..=n {
        i.add_row(Row {
            object: object as u64,
            bounds: [0., 0., 10., 10.],
        })
        .unwrap();
    }
    i.finish_build().unwrap();
    i
}
fn point(x: f64) -> (Point, ScreenPoint) {
    ([x, 0., 0.], [x, 0., 0.5])
}
#[test]
fn missing_requested_mode_cannot_certify_a_partial_query() {
    let mut i = index(1);
    let mut q = i.start_query([0., 0.], 8., 6, limits()).unwrap();
    q.consume(1, 2, &[point(0.)], true, 1).unwrap();
    let r = i.publish_query(q).unwrap();
    assert!(!r.complete && !r.picked);
    let mut q = i.start_query([0., 0.], 8., 2, limits()).unwrap();
    assert!(!q.cached(1, 2).unwrap());
}
#[test]
fn failed_row_poisoning_prevents_accidental_finish() {
    let mut i = index(1);
    i.begin_build(key()).unwrap();
    i.add_row(Row {
        object: 1,
        bounds: [0., 0., 10., 10.],
    })
    .unwrap();
    assert!(
        i.add_row(Row {
            object: 0,
            bounds: [0., 0., 1., 1.]
        })
        .is_err()
    );
    assert!(i.finish_build().is_err());
    assert!(!i.matches(&key()));
}
#[test]
fn tiles_filter_ten_thousand_remote_objects() {
    let mut i = index(1);
    i.begin_build(key()).unwrap();
    i.add_row(Row {
        object: 1,
        bounds: [0., 0., 1., 1.],
    })
    .unwrap();
    for object in 2..=10001 {
        i.add_row(Row {
            object,
            bounds: [500., 500., 600., 600.],
        })
        .unwrap();
    }
    i.finish_build().unwrap();
    assert_eq!(i.near_objects([0., 0.], 8., limits()).unwrap(), vec![1]);
}
#[test]
fn aborted_rebuild_never_publishes_partial_clean_index() {
    let mut i = index(1);
    i.begin_build(key()).unwrap();
    i.add_row(Row {
        object: 2,
        bounds: [0., 0., 10., 10.],
    })
    .unwrap();
    i.abort_build();
    assert!(!i.matches(&key()));
    assert!(i.start_query([0., 0.], 8., 2, limits()).is_err());
    i.begin_build(key()).unwrap();
    i.add_row(Row {
        object: 1,
        bounds: [0., 0., 10., 10.],
    })
    .unwrap();
    i.finish_build().unwrap();
    assert!(i.matches(&key()));
}
#[test]
fn object_and_candidate_caps_precede_candidate_acceptance() {
    let mut too_many = index(65);
    assert!(too_many.start_query([0., 0.], 8., 2, limits()).is_err());
    let mut i = index(5);
    let mut q = i.start_query([0., 0.], 8., 2, limits()).unwrap();
    for id in 1..=4 {
        assert_eq!(q.remaining_budget(id), 2048);
        q.consume(id, 2, &vec![point(0.); 2048], true, 2048)
            .unwrap();
    }
    assert_eq!(q.remaining_budget(5), 0);
    assert!(q.consume(5, 2, &[point(0.)], true, 1).is_err());
    assert!(!q.finish().picked);
}
#[test]
fn nearest_ties_depth_radius_and_native_failure_are_checked() {
    let mut i = index(1);
    let mut q = i.start_query([0., 0.], 8., 2, limits()).unwrap();
    q.consume(
        1,
        2,
        &[point(4.), ([99., 0., 0.], [0., 0., 2.]), point(-4.)],
        true,
        3,
    )
    .unwrap();
    let r = i.publish_query(q).unwrap();
    assert!(r.complete && r.picked);
    assert_eq!(r.point, [-4., 0., 0.]);
    let mut q = i.start_query([0., 0.], 8., 6, limits()).unwrap();
    assert!(q.cached(1, 2).unwrap());
    q.consume(1, 4, &[], false, 99).unwrap();
    let r = q.finish();
    assert!(!r.complete && !r.picked);
}
#[test]
fn cache_key_contains_mode_residual_budget_and_generation() {
    let mut i = index(1);
    let mut q = i
        .start_query(
            [0., 0.],
            8.,
            6,
            Limits {
                objects: 64,
                per_object: 2,
                total: 2,
            },
        )
        .unwrap();
    q.consume(1, 2, &[point(1.)], true, 1).unwrap();
    assert_eq!(q.remaining_budget(1), 1);
    q.consume(1, 4, &[point(2.)], true, 1).unwrap();
    i.publish_query(q).unwrap();
    let mut q = i
        .start_query(
            [0., 0.],
            8.,
            6,
            Limits {
                objects: 64,
                per_object: 2,
                total: 2,
            },
        )
        .unwrap();
    assert!(q.cached(1, 2).unwrap());
    assert!(q.cached(1, 4).unwrap());
    assert_eq!(q.generated, 0);
    assert!(q.consume(1, 4, &[point(0.)], true, 1).is_err());
    let mut changed = key();
    changed.camera[0] = 1.;
    assert!(!i.matches(&changed));
    i.invalidate();
    assert!(!i.matches(&key()));
}
#[test]
fn stale_query_and_invalid_numeric_inputs_never_publish_cache() {
    let mut i = index(1);
    let q = i.start_query([0., 0.], 8., 2, limits()).unwrap();
    i.invalidate();
    assert!(i.publish_query(q).is_err());
    let mut i = index(1);
    assert!(i.start_query([f64::NAN, 0.], 8., 2, limits()).is_err());
    assert!(i.start_query([0., 0.], 8., 16, limits()).is_err());
    let mut q = i.start_query([0., 0.], 8., 2, limits()).unwrap();
    assert!(
        q.consume(1, 2, &[([f64::NAN, 0., 0.], [0., 0., 0.])], true, 1)
            .is_err()
    );
    assert!(!q.finish().picked);
}

#[test]
fn candidate_cap_clears_object_lookup_and_rebuild_has_no_stale_hits() {
    let mut i = SnapIndex::default();
    let mut wide = key();
    wide.width = 10000;
    i.begin_build(wide.clone()).unwrap();
    for object in 1..=65 {
        let x = object as f64 * 100.;
        i.add_row(Row {
            object,
            bounds: [x, 0., x + 1., 1.],
        })
        .unwrap();
    }
    i.finish_build().unwrap();
    for object in 1..=65 {
        let x = object as f64 * 100.;
        let mut q = i.start_query([x, 0.], 8., 2, limits()).unwrap();
        assert_eq!(q.objects(), &[object]);
        q.consume(object, 2, &vec![point(x); 2048], true, 2048)
            .unwrap();
        assert!(i.publish_query(q).unwrap().complete);
    }
    let mut old = i.start_query([100., 0.], 8., 2, limits()).unwrap();
    assert!(!old.cached(1, 2).unwrap());
    let mut newest = i.start_query([6500., 0.], 8., 2, limits()).unwrap();
    assert!(newest.cached(65, 2).unwrap());
    assert_eq!(newest.generated, 0);
    i.invalidate();
    assert!(i.publish_query(newest).is_err());
    i.begin_build(wide).unwrap();
    i.add_row(Row {
        object: 65,
        bounds: [6500., 0., 6501., 1.],
    })
    .unwrap();
    i.finish_build().unwrap();
    let mut rebuilt = i.start_query([6500., 0.], 8., 2, limits()).unwrap();
    assert!(!rebuilt.cached(65, 2).unwrap());
}
