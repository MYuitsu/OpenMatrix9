// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::worker_budget::{
    WorkerBudgetError, default_worker_count, import_worker_count, om9_3dm_import_worker_count,
};

#[test]
fn default_uses_sixty_percent_without_a_fixed_four_worker_cap() {
    for (cpus, expected) in [
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 2),
        (5, 3),
        (6, 3),
        (7, 4),
        (8, 4),
        (9, 5),
        (10, 6),
        (12, 7),
        (16, 9),
        (24, 14),
        (32, 19),
        (64, 38),
    ] {
        assert_eq!(default_worker_count(cpus), expected, "CPU count {cpus}");
    }
}

#[test]
fn integer_budget_does_not_overflow_at_counter_limits() {
    assert_eq!(default_worker_count(u32::MAX), 2_576_980_377);
    assert_eq!(default_worker_count(u32::MAX - 1), 2_576_980_376);
    assert_eq!(
        import_worker_count(i32::MAX, u64::MAX, 1024, None),
        Ok(1_288_490_188)
    );
}

#[test]
fn independent_work_and_existing_serial_thresholds_bound_imports() {
    assert_eq!(import_worker_count(64, 8, 16 * 1024 * 1024, None), Ok(8));
    assert_eq!(import_worker_count(64, 100, 1024, None), Ok(38));
    assert_eq!(import_worker_count(64, 7, 1024, None), Ok(1));
    assert_eq!(import_worker_count(64, 0, 1024, None), Ok(1));
    assert_eq!(
        import_worker_count(64, 100, 16 * 1024 * 1024 + 1, None),
        Ok(1)
    );
    assert_eq!(import_worker_count(64, 100, u64::MAX, None), Ok(1));
    assert_eq!(import_worker_count(-1, 100, 1024, None), Ok(1));
    assert_eq!(import_worker_count(0, 100, 1024, None), Ok(1));
}

#[test]
fn explicit_worker_override_is_a_validated_ceiling() {
    assert_eq!(import_worker_count(32, 100, 1024, Some(1)), Ok(1));
    assert_eq!(import_worker_count(32, 100, 1024, Some(6)), Ok(6));
    assert_eq!(import_worker_count(32, 100, 1024, Some(100)), Ok(19));
    assert_eq!(import_worker_count(64, 8, 1024, Some(20)), Ok(8));
    assert_eq!(import_worker_count(64, 7, 1024, Some(20)), Ok(1));
    for invalid in [i64::MIN, -1, 0, i32::MAX as i64 + 1, i64::MAX] {
        assert_eq!(
            import_worker_count(32, 100, 1024, Some(invalid)),
            Err(WorkerBudgetError::InvalidOverride),
        );
        assert_eq!(
            import_worker_count(32, 0, u64::MAX, Some(invalid)),
            Err(WorkerBudgetError::InvalidOverride),
        );
    }
}

#[test]
fn pointer_free_abi_preserves_bounds_and_rejects_invalid_configuration() {
    assert_eq!(om9_3dm_import_worker_count(32, 100, 1024, false, -1), 19);
    assert_eq!(om9_3dm_import_worker_count(32, 100, 1024, true, 6), 6);
    assert_eq!(om9_3dm_import_worker_count(32, 100, 1024, true, 0), 0);
    assert_eq!(om9_3dm_import_worker_count(32, 100, 1024, true, -1), 0);
    assert_eq!(
        om9_3dm_import_worker_count(i32::MAX, u64::MAX, 1024, true, i64::MAX),
        0
    );
    assert_eq!(
        om9_3dm_import_worker_count(i32::MAX, u64::MAX, 1024, false, 0),
        1_288_490_188
    );
}
