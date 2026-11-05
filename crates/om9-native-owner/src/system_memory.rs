// SPDX-License-Identifier: LGPL-2.1-or-later
/// A point-in-time Windows physical-memory sample, excluding swap and VRAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalMemory {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

pub use om9_core::budget::memory::CommitMemory;

fn commit_from_pages(limit: usize, used: usize, page_size: usize) -> Option<CommitMemory> {
    if limit == 0 || used > limit || page_size == 0 { return None; }
    let page_size = u64::try_from(page_size).ok()?;
    Some(CommitMemory {
        limit_bytes: u64::try_from(limit).ok()?.checked_mul(page_size)?,
        used_bytes: u64::try_from(used).ok()?.checked_mul(page_size)?,
    })
}

/// System commit in bytes; the Windows fields are page counts, not bytes.
/// https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-performance_information
#[cfg(windows)]
pub fn commit_memory() -> Option<CommitMemory> {
    use windows_sys::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};
    let size = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
    let mut info = PERFORMANCE_INFORMATION { cb: size, ..Default::default() };
    // SAFETY: initialized writable structure and matching size; Windows retains
    // no pointer. Policy and byte arithmetic stay in safe Rust.
    if unsafe { K32GetPerformanceInfo(&mut info, size) } == 0 { return None; }
    commit_from_pages(info.CommitLimit, info.CommitTotal, info.PageSize)
}

#[cfg(not(windows))]
pub fn commit_memory() -> Option<CommitMemory> { None }

/// Process-wide diagnostics; peak values include all earlier work in this process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessMemory {
    pub working_set_bytes: u64,
    pub peak_working_set_bytes: u64,
    pub private_bytes: u64,
    pub peak_pagefile_bytes: u64,
}

#[cfg(windows)]
pub fn process_memory() -> Option<ProcessMemory> {
    use windows_sys::Win32::System::{ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX}, Threading::GetCurrentProcess};
    let mut info = PROCESS_MEMORY_COUNTERS_EX { cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32, ..Default::default() };
    // SAFETY: the pseudo-handle is valid for this process and Windows writes only
    // into the correctly sized EX structure for the duration of the call.
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), (&mut info as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(), info.cb) };
    (ok != 0).then_some(ProcessMemory {
        working_set_bytes: info.WorkingSetSize as u64,
        peak_working_set_bytes: info.PeakWorkingSetSize as u64,
        private_bytes: info.PrivateUsage as u64,
        peak_pagefile_bytes: info.PeakPagefileUsage as u64,
    })
}

#[cfg(not(windows))]
pub fn process_memory() -> Option<ProcessMemory> { None }

#[cfg(windows)]
pub fn physical_memory() -> Option<PhysicalMemory> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: Windows receives an initialized, correctly sized writable structure;
    // it retains no pointer. Platform access is isolated in the native owner.
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    (ok != 0 && status.ullTotalPhys > 0 && status.ullAvailPhys <= status.ullTotalPhys)
        .then_some(PhysicalMemory { total_bytes: status.ullTotalPhys, available_bytes: status.ullAvailPhys })
}

#[cfg(not(windows))]
pub fn physical_memory() -> Option<PhysicalMemory> {
    None
}

/// Physical RAM capacity, not available RAM, swap or GPU memory.
pub fn physical_memory_bytes() -> Option<u64> {
    physical_memory().map(|sample| sample.total_bytes)
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn system_commit_reports_valid_byte_counts() {
        let sample = super::commit_memory().expect("system commit sample");
        assert!(sample.limit_bytes > 0 && sample.used_bytes <= sample.limit_bytes);
    }

    #[test]
    fn commit_page_conversion_checks_units_ranges_and_overflow() {
        assert_eq!(super::commit_from_pages(100, 40, 4096), Some(super::CommitMemory {
            limit_bytes: 409600, used_bytes: 163840,
        }));
        assert!(super::commit_from_pages(0, 0, 4096).is_none());
        assert!(super::commit_from_pages(100, 101, 4096).is_none());
        assert!(super::commit_from_pages(100, 40, 0).is_none());
        assert!(super::commit_from_pages(usize::MAX, 0, usize::MAX).is_none());
    }
    #[test]
    fn windows_reports_physical_capacity() {
        assert!(super::physical_memory_bytes().is_some_and(|bytes| bytes >= 1024 * 1024));
    }

    #[test]
    fn windows_reports_valid_available_ram() {
        let sample = super::physical_memory().expect("physical memory sample");
        assert!(sample.total_bytes >= 1024 * 1024);
        assert!(sample.available_bytes <= sample.total_bytes);
    }

    #[test]
    fn process_memory_reports_current_and_peak_values() {
        let memory = super::process_memory().expect("process memory sample");
        assert!(memory.working_set_bytes > 0 && memory.private_bytes > 0);
        assert!(memory.peak_working_set_bytes >= memory.working_set_bytes);
        assert!(memory.peak_pagefile_bytes >= memory.private_bytes);
    }
}
