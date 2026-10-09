// SPDX-License-Identifier: LGPL-2.1-or-later
//! RCORE-10 worker-count policy. This limits process count, not actual CPU/RAM use.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerBudgetError {
    InvalidOverride,
}

/// OM9 default: max(1, floor(0.60 * logical CPUs)), without floating-point rounding
/// or an intermediate multiplication that can overflow at u32::MAX.
pub fn default_worker_count(logical_cpus: u32) -> u32 {
    (logical_cpus / 5 * 3 + logical_cpus % 5 * 3 / 5).max(1)
}

/// Plan the existing native import path from value-only host observations.
///
/// Unknown/nonpositive Qt CPU counts use one. A configured override is a positive
/// signed-native-int ceiling and cannot increase the default or task count.
/// A result of one selects the existing serial host converter, including an empty
/// archive. The <8-root and >16 MiB fallbacks are retained import heuristics, not
/// a measured RAM budget or a complete aggregate scheduling policy.
pub fn import_worker_count(
    logical_cpus: i32,
    independent_tasks: u64,
    source_bytes: u64,
    requested_workers: Option<i64>,
) -> Result<u32, WorkerBudgetError> {
    let ceiling = match requested_workers {
        Some(value) if value > 0 && value <= i64::from(i32::MAX) => value as u32,
        Some(_) => return Err(WorkerBudgetError::InvalidOverride),
        None => i32::MAX as u32,
    };
    if independent_tasks < 8 || source_bytes > 16 * 1024 * 1024 {
        return Ok(1);
    }
    let workers = default_worker_count(logical_cpus.max(1) as u32).min(ceiling);
    // Convert only after the u64 task count has been capped by the native-int
    // worker count. No allocation or borrowed native pointer is involved.
    Ok(u64::from(workers).min(independent_tasks) as u32)
}

/// Pointer-free, allocation-free C ABI; zero rejects an invalid override.
/// Every successful result fits a positive native signed 32-bit count.
/// An unused requested_workers value is ignored when override_present is false.
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_import_worker_count(
    logical_cpus: i32,
    independent_tasks: u64,
    source_bytes: u64,
    override_present: bool,
    requested_workers: i64,
) -> u32 {
    import_worker_count(
        logical_cpus,
        independent_tasks,
        source_bytes,
        override_present.then_some(requested_workers),
    )
    .unwrap_or(0)
}
