// SPDX-License-Identifier: LGPL-2.1-or-later
//! Allocation-free request policy. Owns values only, never host pointers.
#[repr(C)]
pub struct Gate {
    next: u64,
    current: u64,
    slots: [u64; 16],
}
impl Default for Gate {
    fn default() -> Self {
        Self::with_next(1)
    }
}
impl Gate {
    pub fn with_next(next: u64) -> Self {
        Self {
            next,
            current: 0,
            slots: [0; 16],
        }
    }
    /// Zero means exhausted or at capacity; no state changes on failure.
    pub fn issue(&mut self) -> u64 {
        if self.next == 0 {
            return 0;
        }
        let Some(slot) = self.slots.iter_mut().find(|s| **s == 0) else {
            return 0;
        };
        let ticket = self.next;
        self.next = ticket.checked_add(1).unwrap_or(0);
        *slot = ticket;
        self.current = ticket;
        ticket
    }
    /// Cancellation invalidates acceptance but not the owned pending lease.
    pub fn invalidate(&mut self) {
        self.current = 0;
    }
    /// The latest pending result can be inspected only once, even when stale.
    /// An older/bogus delivery must not consume the newer result's permission.
    pub fn claim(&mut self, ticket: u64, context_valid: bool) -> bool {
        if ticket == 0 || ticket != self.current || !self.slots.contains(&ticket) {
            return false;
        }
        self.current = 0;
        context_valid
    }
    /// Native RAII releases its output owners and retires the matching slot once.
    pub fn retire(&mut self, ticket: u64) -> bool {
        if ticket == 0 {
            return false;
        }
        let Some(slot) = self.slots.iter_mut().find(|s| **s == ticket) else {
            return false;
        };
        *slot = 0;
        if self.current == ticket {
            self.current = 0;
        }
        true
    }
    pub fn pending(&self) -> usize {
        self.slots.iter().filter(|s| **s != 0).count()
    }
}
/// # Safety
/// Non-null gate must be aligned, initialized and exclusively writable for the
/// synchronous call. The pointer is never retained. Use Gate::default first.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_result_issue(gate: *mut Gate) -> u64 {
    unsafe { gate.as_mut() }.map_or(0, Gate::issue)
}
/// # Safety
/// Same exclusive, initialized caller-owned Gate contract as om9_result_issue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_result_invalidate(gate: *mut Gate) {
    if let Some(g) = unsafe { gate.as_mut() } {
        g.invalidate()
    }
}
/// # Safety
/// Same exclusive, initialized caller-owned Gate contract as om9_result_issue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_result_claim(
    gate: *mut Gate,
    ticket: u64,
    context_valid: bool,
) -> bool {
    unsafe { gate.as_mut() }.is_some_and(|g| g.claim(ticket, context_valid))
}
/// # Safety
/// Same exclusive, initialized caller-owned Gate contract as om9_result_issue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_result_retire(gate: *mut Gate, ticket: u64) -> bool {
    unsafe { gate.as_mut() }.is_some_and(|g| g.retire(ticket))
}
