// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ported from `rust/tests/core_3dm_archive.rs` (archive/member-copy/closure policy).

use om9_exchange::three_dm::{
    ArchiveIdentity, ClosureError, copy_budget, dependency_closure, overlay_allowed,
};

#[test]
fn native_geometry_transform_policy_requires_known_non_instance_payload() {
    for capability in [1, 2, 3] {
        assert!(overlay_allowed(capability, 4, true, false));
        assert!(!overlay_allowed(capability, 4, true, true));
        assert!(!overlay_allowed(capability, 4, false, false));
    }
    assert!(!overlay_allowed(4, 4, true, false));
    assert!(!overlay_allowed(5, 4, true, false));
}

#[test]
fn overlay_policy_rejects_unsupported_edits_and_unknown_references() {
    assert!(overlay_allowed(1, 1, true, false));
    assert!(!overlay_allowed(3, 1, true, false));
    assert!(overlay_allowed(3, 2, true, true));
    assert!(!overlay_allowed(3, 2, true, false));
    assert!(overlay_allowed(1, 3, true, false));
    assert!(!overlay_allowed(4, 0, true, false));
    assert!(!overlay_allowed(1, 0, false, false));
    assert!(!overlay_allowed(5, 0, true, false));
    assert!(!overlay_allowed(1, 5, true, false));
}

#[test]
fn closure_selects_only_reachable_unique_records() {
    assert_eq!(
        dependency_closure(5, &[0, 2, 3, 3, 4, 4], &[1, 2, 2, 4], &[0, 0]).unwrap(),
        vec![0, 1, 2]
    );
    assert_eq!(
        dependency_closure(5, &[0, 2, 3, 3, 4, 4], &[1, 2, 2, 4], &[3]).unwrap(),
        vec![3, 4]
    );
    assert!(dependency_closure(0, &[0], &[], &[]).unwrap().is_empty());
}

#[test]
fn closure_rejects_malformed_cycles_and_limits() {
    assert_eq!(
        dependency_closure(2, &[0, 1, 2], &[1, 0], &[0]),
        Err(ClosureError::Cycle)
    );
    assert_eq!(
        dependency_closure(1, &[0, 1], &[1], &[0]),
        Err(ClosureError::Malformed)
    );
    assert_eq!(
        dependency_closure(1, &[1, 1], &[], &[0]),
        Err(ClosureError::Malformed)
    );
    assert_eq!(
        dependency_closure(2, &[0, 2, 1], &[0], &[0]),
        Err(ClosureError::Malformed)
    );
    assert_eq!(
        dependency_closure(1_000_001, &[], &[], &[]),
        Err(ClosureError::Limit)
    );
    assert_eq!(
        dependency_closure(1, &[0, 0], &[], &[1]),
        Err(ClosureError::Malformed)
    );
    // Unselected cycles cannot add unrelated source objects or block a safe selection.
    assert_eq!(
        dependency_closure(3, &[0, 0, 1, 2], &[2, 1], &[0]).unwrap(),
        vec![0]
    );
}

#[test]
fn closure_handles_long_component_chains_without_recursion() {
    let count = 100_000;
    let offsets: Vec<_> = (0..count).chain(std::iter::once(count - 1)).collect();
    let edges: Vec<_> = (1..count).collect();
    assert_eq!(
        dependency_closure(count, &offsets, &edges, &[0])
            .unwrap()
            .len(),
        count
    );
}

#[test]
fn modes_and_capabilities_reject_unknown_values() {
    use om9_exchange::three_dm::ArchiveMode;
    assert!(ArchiveMode::from_code(0).is_some());
    assert!(ArchiveMode::from_code(1).is_some());
    assert!(ArchiveMode::from_code(2).is_none());
    for value in 1..=4 {
        assert!(om9_exchange::three_dm::ArchiveCapability::from_code(value).is_some());
    }
    assert!(om9_exchange::three_dm::ArchiveCapability::from_code(0).is_none());
    assert!(om9_exchange::three_dm::ArchiveCapability::from_code(5).is_none());
}

#[test]
fn identities_are_scoped_and_canonical() {
    let source = "12345678-1234-1234-1234-123456789abc";
    let a = ArchiveIdentity::new(source, source).unwrap();
    let b = ArchiveIdentity::new("12345678-1234-1234-1234-123456789abd", source).unwrap();
    assert_ne!(a, b);
    for invalid in [
        "",
        "00000000-0000-0000-0000-000000000000",
        "12345678-1234-1234-1234-123456789ABC",
        "12345678123412341234123456789abc",
        "é",
    ] {
        assert!(ArchiveIdentity::new(invalid, source).is_err());
        assert!(ArchiveIdentity::new(source, invalid).is_err());
    }
}

#[test]
fn member_copy_budget_bounds_total_and_arithmetic() {
    let limit = 512 * 1024 * 1024;
    assert_eq!(copy_budget(0, 1, limit), Some(limit));
    assert_eq!(copy_budget(1024 * 1024, 512, 0), Some(limit));
    assert_eq!(copy_budget(1024 * 1024, 513, 0), None);
    assert_eq!(copy_budget(1, 1, limit), None);
    assert_eq!(copy_budget(0, 0, limit + 1), None);
    assert_eq!(copy_budget(usize::MAX, 2, 0), None);
    assert_eq!(copy_budget(1, 1, usize::MAX), None);
}
