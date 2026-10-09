use openmatrix9_rust::result_publication::Gate;

#[test]
fn current_result_claims_once_and_retires_once() {
    let mut gate = Gate::default();
    let a = gate.issue();
    assert_ne!(a, 0);
    assert!(gate.claim(a, true));
    assert!(!gate.claim(a, true));
    assert_eq!(gate.pending(), 1);
    assert!(gate.retire(a));
    assert!(!gate.retire(a));
    assert_eq!(gate.pending(), 0);
}
#[test]
fn canceled_and_invalid_context_results_never_reclaim() {
    let mut gate = Gate::default();
    let a = gate.issue();
    gate.invalidate();
    assert!(!gate.claim(a, true));
    assert!(gate.retire(a));
    let b = gate.issue();
    assert!(b > a);
    assert!(!gate.claim(b, false));
    assert!(!gate.claim(b, true));
    assert!(gate.retire(b));
}
#[test]
fn old_delivery_preserves_new_current_request() {
    let mut gate = Gate::default();
    let a = gate.issue();
    let b = gate.issue();
    assert!(!gate.claim(a, true));
    assert!(gate.retire(a));
    assert!(gate.claim(b, true));
    assert!(gate.retire(b));
}
#[test]
fn fixed_capacity_rejects_before_more_owned_results() {
    let mut gate = Gate::default();
    let tickets: Vec<_> = (0..16).map(|_| gate.issue()).collect();
    assert!(tickets.iter().all(|t| *t != 0));
    assert_eq!(gate.issue(), 0);
    assert_eq!(gate.pending(), 16);
    assert!(gate.claim(tickets[15], true));
    assert!(gate.retire(tickets[0]));
    let next = gate.issue();
    assert!(next > tickets[15]);
    for t in tickets.into_iter().skip(1) {
        assert!(gate.retire(t));
    }
    assert!(gate.retire(next));
    assert_eq!(gate.pending(), 0);
}
#[test]
fn identifier_exhaustion_never_wraps_or_reuses() {
    let mut gate = Gate::with_next(u64::MAX);
    let a = gate.issue();
    assert_eq!(a, u64::MAX);
    assert_eq!(gate.issue(), 0);
    assert!(gate.claim(a, true));
    assert!(gate.retire(a));
    assert_eq!(gate.issue(), 0);
}
#[test]
fn bogus_identifiers_do_not_consume_or_release_current() {
    let mut gate = Gate::default();
    let a = gate.issue();
    for wrong in [0, a + 1, u64::MAX] {
        assert!(!gate.claim(wrong, true));
        assert!(!gate.retire(wrong));
    }
    assert_eq!(gate.pending(), 1);
    assert!(gate.claim(a, true));
    assert!(gate.retire(a));
}
#[test]
fn abi_layout_and_null_fail_closed() {
    use openmatrix9_rust::result_publication::*;
    assert_eq!(std::mem::size_of::<Gate>(), 144);
    assert_eq!(std::mem::align_of::<Gate>(), 8);
    unsafe {
        assert_eq!(om9_result_issue(std::ptr::null_mut()), 0);
        assert!(!om9_result_claim(std::ptr::null_mut(), 1, true));
        assert!(!om9_result_retire(std::ptr::null_mut(), 1));
        om9_result_invalidate(std::ptr::null_mut());
    }
    let mut gate = Gate::default();
    unsafe {
        let a = om9_result_issue(&mut gate);
        assert!(om9_result_claim(&mut gate, a, true));
        assert!(om9_result_retire(&mut gate, a));
    }
}
