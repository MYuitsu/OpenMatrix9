use openmatrix9_rust::core_3dm_archive::*;
#[test]
fn native_geometry_transform_policy_requires_known_non_instance_payload() {
    for capability in [1, 2, 3] {
        assert!(om9_3dm_overlay_allowed(capability, 4, true, false));
        assert!(!om9_3dm_overlay_allowed(capability, 4, true, true));
        assert!(!om9_3dm_overlay_allowed(capability, 4, false, false));
    }
    assert!(!om9_3dm_overlay_allowed(4, 4, true, false));
    assert!(!om9_3dm_overlay_allowed(5, 4, true, false));
}
#[test]
fn overlay_policy_rejects_unsupported_edits_and_unknown_references() {
    assert!(om9_3dm_overlay_allowed(1, 1, true, false));
    assert!(!om9_3dm_overlay_allowed(3, 1, true, false));
    assert!(om9_3dm_overlay_allowed(3, 2, true, true));
    assert!(!om9_3dm_overlay_allowed(3, 2, true, false));
    assert!(om9_3dm_overlay_allowed(1, 3, true, false));
    assert!(!om9_3dm_overlay_allowed(4, 0, true, false));
    assert!(!om9_3dm_overlay_allowed(1, 0, false, false));
    assert!(!om9_3dm_overlay_allowed(5, 0, true, false));
    assert!(!om9_3dm_overlay_allowed(1, 5, true, false));
}
#[test]
fn closure_ffi_checks_capacity_and_preserves_output_on_failure() {
    let offsets = [0, 2, 2, 2];
    let edges = [1, 2];
    let selected = [0];
    let mut output = [99usize; 3];
    unsafe {
        assert_eq!(
            om9_3dm_dependency_closure(
                3,
                offsets.as_ptr(),
                edges.as_ptr(),
                2,
                selected.as_ptr(),
                1,
                std::ptr::null_mut(),
                0
            ),
            3
        );
        assert_eq!(
            om9_3dm_dependency_closure(
                3,
                offsets.as_ptr(),
                edges.as_ptr(),
                2,
                selected.as_ptr(),
                1,
                output.as_mut_ptr(),
                2
            ),
            -4
        );
        assert_eq!(output, [99; 3]);
        assert_eq!(
            om9_3dm_dependency_closure(
                3,
                offsets.as_ptr(),
                edges.as_ptr(),
                2,
                selected.as_ptr(),
                1,
                output.as_mut_ptr(),
                3
            ),
            3
        );
        assert_eq!(output, [0, 1, 2]);
        assert_eq!(
            om9_3dm_dependency_closure(
                3,
                std::ptr::null(),
                edges.as_ptr(),
                2,
                selected.as_ptr(),
                1,
                output.as_mut_ptr(),
                3
            ),
            -1
        );
        assert_eq!(
            om9_3dm_dependency_closure(
                1_000_001,
                std::ptr::null(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0
            ),
            -3
        );
        let cycle_offsets = [0, 1, 2];
        let cycle_edges = [1, 0];
        assert_eq!(
            om9_3dm_dependency_closure(
                2,
                cycle_offsets.as_ptr(),
                cycle_edges.as_ptr(),
                2,
                selected.as_ptr(),
                1,
                output.as_mut_ptr(),
                3
            ),
            -2
        );
    }
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
    assert!(om9_3dm_archive_mode_valid(0));
    assert!(om9_3dm_archive_mode_valid(1));
    assert!(!om9_3dm_archive_mode_valid(2));
    for value in 1..=4 {
        assert!(om9_3dm_archive_capability_valid(value));
    }
    assert!(!om9_3dm_archive_capability_valid(0));
    assert!(!om9_3dm_archive_capability_valid(5));
    assert!(om9_3dm_archive_legacy_export_allowed(0));
    assert!(!om9_3dm_archive_legacy_export_allowed(1));
    assert!(!om9_3dm_archive_legacy_export_allowed(u32::MAX));
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
    use openmatrix9_rust::core_3dm_archive::om9_3dm_copy_budget;
    let limit = 512 * 1024 * 1024;
    assert_eq!(om9_3dm_copy_budget(0, 1, limit), limit as isize);
    assert_eq!(om9_3dm_copy_budget(1024 * 1024, 512, 0), limit as isize);
    assert_eq!(om9_3dm_copy_budget(1024 * 1024, 513, 0), -1);
    assert_eq!(om9_3dm_copy_budget(1, 1, limit), -1);
    assert_eq!(om9_3dm_copy_budget(0, 0, limit + 1), -1);
    assert_eq!(om9_3dm_copy_budget(usize::MAX, 2, 0), -1);
    assert_eq!(om9_3dm_copy_budget(1, 1, usize::MAX), -1);
}
