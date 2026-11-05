// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared reservations for managed runtime allocations. Native allocator overhead
//! and memory owned by other processes remain outside this accounting.
use crate::{Error, Result};
use std::sync::{Arc, Mutex};

const MIB: u64 = 1024 * 1024;

/// Point-in-time system commit, independent of physical RAM and process RSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitMemory {
    pub limit_bytes: u64,
    pub used_bytes: u64,
}
impl CommitMemory {
    pub(crate) fn available_for_pending(self, pending: u64) -> Result<u64> {
        if self.limit_bytes == 0 || self.used_bytes > self.limit_bytes {
            return Err(Error::Invalid("system commit sample"));
        }
        // Reuse the project's minimum 256 MiB safety margin. This is a guard
        // against spending all observed commit, not an allocator/native bound
        // or a percentage of a potentially very large pagefile.
        Ok((self.limit_bytes - self.used_bytes).saturating_sub(256 * MIB).saturating_sub(pending))
    }
}

/// Free physical headroom for allocations not yet materialized. Live managed
/// allocations stay in the managed ceiling, but are not subtracted a second
/// time from a fresh OS Available sample. This is not an OS reservation or a
/// guarantee that committed allocations remain resident.
pub(crate) fn available_for_pending(total: u64, available: u64, pending: u64) -> Result<u64> {
    if total == 0 || available > total { return Err(Error::Invalid("physical RAM sample")); }
    Ok(available.saturating_sub((total / 10).max(256 * MIB)).saturating_sub(pending))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryClass {
    Document,
    Undo,
    CpuCache,
    Upload,
    Kernel,
    Export,
}

#[derive(Debug, Clone)]
pub struct MemoryBudget {
    state: Arc<Mutex<State>>,
    total_physical: u64,
}

#[derive(Debug)]
struct State {
    ceiling: u64,
    reserved: u64,
    pending: u64,
    by_class: [u64; 6],
}

#[derive(Debug)]
pub struct Reservation {
    state: Arc<Mutex<State>>,
    class: MemoryClass,
    bytes: u64,
    pending_bytes: u64,
}

impl MemoryClass {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Document => 0,
            Self::Undo => 1,
            Self::CpuCache => 2,
            Self::Upload => 3,
            Self::Kernel => 4,
            Self::Export => 5,
        }
    }
}

impl MemoryBudget {
    pub fn new(total_physical: u64) -> Result<Self> {
        if total_physical == 0 {
            return Err(Error::Invalid("physical RAM capacity"));
        }
        let ceiling = (u128::from(total_physical) * 3 / 5) as u64;
        Ok(Self {
            state: Arc::new(Mutex::new(State {
                ceiling,
                reserved: 0,
                pending: 0,
                by_class: [0; 6],
            })),
            total_physical,
        })
    }

    /// `available_physical` is sampled before this request, not a reservation by
    /// the OS. None leaves physical admission unverified, not already allocated.
    /// Requests remain pending until explicitly marked allocated.
    /// Concurrent pending requests cannot spend the same sampled headroom.
    pub fn reserve(
        &self,
        class: MemoryClass,
        bytes: u64,
        available_physical: Option<u64>,
    ) -> Result<Reservation> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(available) = available_physical {
            if bytes > available_for_pending(self.total_physical, available, state.pending)? {
                return Err(Error::Budget("available physical RAM"));
            }
        }
        let next = state
            .reserved
            .checked_add(bytes)
            .ok_or(Error::Overflow("managed memory"))?;
        if next > state.ceiling {
            return Err(Error::Budget("managed memory"));
        }
        let index = class.index();
        let class_bytes = state.by_class[index]
            .checked_add(bytes)
            .ok_or(Error::Overflow("memory class"))?;
        state.reserved = next;
        state.by_class[index] = class_bytes;
        state.pending += bytes;
        Ok(Reservation {
            state: Arc::clone(&self.state),
            class,
            bytes,
            pending_bytes: bytes,
        })
    }

    pub fn totals(&self) -> (u64, u64, [u64; 6]) {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        (state.reserved, state.ceiling, state.by_class)
    }
}

impl Reservation {
    pub fn class(&self) -> MemoryClass { self.class }
    pub fn bytes(&self) -> u64 { self.bytes }
    pub fn pending_bytes(&self) -> u64 { self.pending_bytes }
    /// Call only after these allocations exist. This changes physical pending
    /// accounting, never the managed total or ownership lifetime.
    pub fn mark_allocated(&mut self, bytes: u64) -> Result<()> {
        let pending = self.pending_bytes.checked_sub(bytes).ok_or(Error::Invalid("allocated reservation bytes"))?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.pending -= bytes;
        self.pending_bytes = pending;
        Ok(())
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.reserved -= self.bytes;
        state.by_class[self.class.index()] -= self.bytes;
        state.pending -= self.pending_bytes;
    }
}
