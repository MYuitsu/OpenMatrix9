// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{Error, transaction::Cancellation};

#[test]
fn cancellation_union_observes_both_original_owners() {
    for original_first in [true, false] {
        let a = Cancellation::default();
        let b = Cancellation::default();
        let linked = a.either(&b).unwrap();
        assert!(!linked.is_canceled());
        if original_first {
            a.cancel();
        } else {
            b.cancel();
        }
        assert!(
            linked.is_canceled(),
            "either operation owner must veto late acceptance"
        );
    }
}
#[test]
fn cancellation_union_is_transitive_and_own_cancel_remains_shared() {
    let a = Cancellation::default();
    let b = Cancellation::default();
    let c = Cancellation::default();
    let linked = a.either(&b).unwrap().either(&c).unwrap();
    c.cancel();
    assert!(linked.is_canceled());
    assert!(!a.is_canceled());
    assert!(!b.is_canceled());
    linked.cancel();
    assert!(a.is_canceled());
}
#[test]
fn cancellation_union_deduplicates_shared_flags_and_bounds_distinct_owners() {
    let a = Cancellation::default();
    let b = Cancellation::default();
    let c = Cancellation::default();
    let d = Cancellation::default();
    let e = Cancellation::default();
    let linked = a
        .either(&b)
        .unwrap()
        .either(&c)
        .unwrap()
        .either(&d)
        .unwrap();
    linked.either(&linked.clone()).unwrap();
    linked.either(&a).unwrap().either(&b).unwrap();
    assert!(matches!(linked.either(&e), Err(Error::Budget(_))));
    assert!(!linked.is_canceled());
    d.cancel();
    assert!(linked.is_canceled());
}
