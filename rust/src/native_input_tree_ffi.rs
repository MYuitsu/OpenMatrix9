//! One synchronous exclusive borrow of caller-owned initialized Gate storage.
//! The caller supplies one live aligned writable allocation, with no aliased or
//! concurrent access. Address guards cannot establish liveness. No native pointer
//! or owner is retained. Failed state is discarded, including native unwinding;
//! it grants no further work. There are no allocations or intentional panics;
//! release retains panic=abort. Native exceptions stay on the C++ side.
use crate::native_input_tree::Gate;
use crate::native_input_tree::TreeError;
unsafe fn apply(gate: *mut Gate, action: impl FnOnce(&mut Gate) -> Result<(), TreeError>) -> u32 {
    let address = gate as usize;
    if gate.is_null()
        || !address.is_multiple_of(std::mem::align_of::<Gate>())
        || address.checked_add(std::mem::size_of::<Gate>()).is_none()
    {
        return 1;
    }
    // SAFETY: address guards ran; the caller provides the live exclusive borrow.
    match action(unsafe { &mut *gate }) {
        Ok(()) => 0,
        Err(error) => error as u32,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_input_tree_init() -> Gate {
    Gate::new()
}
/// # Safety
/// Caller storage must satisfy the exclusive live-allocation contract above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_input_tree_enter(gate: *mut Gate, identity: u64) -> u32 {
    // SAFETY: forwarded one-call exclusive borrow obligation.
    unsafe { apply(gate, |g| g.enter(identity)) }
}
/// # Safety
/// Caller storage must satisfy the exclusive live-allocation contract above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_input_tree_leave(gate: *mut Gate, identity: u64) -> u32 {
    // SAFETY: forwarded one-call exclusive borrow obligation.
    unsafe { apply(gate, |g| g.leave(identity)) }
}
/// # Safety
/// Caller storage must satisfy the exclusive live-allocation contract above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_input_tree_children(gate: *mut Gate, count: usize) -> u32 {
    // SAFETY: forwarded one-call exclusive borrow obligation.
    unsafe { apply(gate, |g| g.children(count)) }
}
/// # Safety
/// Caller storage must satisfy the exclusive live-allocation contract above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_input_tree_account(gate: *mut Gate, bytes: usize) -> u32 {
    // SAFETY: forwarded one-call exclusive borrow obligation.
    unsafe { apply(gate, |g| g.account(bytes)) }
}
/// # Safety
/// Caller storage must satisfy the exclusive live-allocation contract above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_input_tree_finish(gate: *mut Gate) -> u32 {
    // SAFETY: forwarded one-call exclusive borrow obligation.
    unsafe { apply(gate, Gate::finished) }
}
