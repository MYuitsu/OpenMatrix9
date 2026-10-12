//! Bounded borrowed C views copied into owned Rust requests; no native objects retained.
use crate::phase3_modeling::{
    InputFacts, Kind, Operation, Reason, Request, Snapshot, validate_inputs,
};
use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, OnceLock},
};
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Facts {
    pub kind: u32,
    pub valid: u8,
    pub closed: u8,
    pub protected: u8,
    pub reserved: u8,
    pub components: u32,
}
#[repr(C)]
pub struct SnapshotView {
    pub identity: *const u8,
    pub identity_len: usize,
    pub signature: *const u8,
    pub signature_len: usize,
}
#[derive(Default)]
struct Registry {
    next: u64,
    requests: HashMap<u64, Request>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}
fn code(result: Result<(), Reason>) -> u32 {
    result.err().map_or(0, |e| e as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase3_result_count(count: usize) -> u32 {
    code(crate::phase3_modeling::validate_result_count(count))
}
// SAFETY: caller provides an aligned, initialized slice readable for this call;
// count is bounded before slice creation. Null/oversized views reject before dereference.
unsafe fn slice<'a, T>(pointer: *const T, count: usize, max: usize) -> Result<&'a [T], Reason> {
    if count == 0 || count > max || pointer.is_null() {
        return Err(Reason::InvalidSnapshot);
    }
    if !(pointer as usize).is_multiple_of(std::mem::align_of::<T>()) {
        return Err(Reason::InvalidSnapshot);
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, count) })
}
unsafe fn string<'a>(pointer: *const u8, length: usize) -> Result<&'a str, Reason> {
    std::str::from_utf8(unsafe { slice(pointer, length, 4096)? })
        .map_err(|_| Reason::InvalidSnapshot)
}
unsafe fn snapshots(pointer: *const SnapshotView, count: usize) -> Result<Vec<Snapshot>, Reason> {
    unsafe { slice(pointer, count, 256)? }
        .iter()
        .map(|s| {
            Ok(Snapshot {
                identity: unsafe { string(s.identity, s.identity_len)? }.into(),
                signature: unsafe { string(s.signature, s.signature_len)? }.into(),
            })
        })
        .collect()
}
/// # Safety
/// `inputs` points to `count` initialized Facts valid until return. No pointer retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase3_capability(
    operation: u32,
    inputs: *const Facts,
    count: usize,
    complete: bool,
) -> u32 {
    catch_unwind(AssertUnwindSafe(|| {
        code((|| {
            let op = Operation::from_u32(operation).ok_or(Reason::Unsupported)?;
            let facts = unsafe { slice(inputs, count, 256)? }
                .iter()
                .map(|v| {
                    if v.valid > 1 || v.closed > 1 || v.protected > 1 || v.reserved != 0 {
                        return Err(Reason::InvalidSnapshot);
                    }
                    Ok(InputFacts {
                        kind: Kind::from_u32(v.kind).ok_or(Reason::Unsupported)?,
                        valid: v.valid == 1,
                        closed: v.closed == 1,
                        protected: v.protected == 1,
                        components: v.components,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            validate_inputs(op, &facts, complete)
        })())
    }))
    .unwrap_or(Reason::InvalidSnapshot as u32)
}
/// # Safety
/// All byte views and `count` SnapshotViews remain initialized/readable until return.
/// UTF-8 views are bounded to 4096 bytes, input count to 256. Data is copied into Rust.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase3_request_begin(
    document: *const u8,
    length: usize,
    inputs: *const SnapshotView,
    count: usize,
) -> u64 {
    catch_unwind(AssertUnwindSafe(|| {
        let request = Request::new(unsafe { string(document, length).ok()? }, unsafe {
            snapshots(inputs, count).ok()?
        })
        .ok()?;
        let mut r = registry().lock().ok()?;
        if r.requests.len() >= 1024 {
            return None;
        }
        let id = r.next.checked_add(1)?;
        r.next = id;
        r.requests.insert(id, request);
        Some(id)
    }))
    .ok()
    .flatten()
    .unwrap_or(0)
}
/// # Safety
/// Same borrowed view requirements as request_begin; no borrowed pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase3_request_validate(
    handle: u64,
    document: *const u8,
    length: usize,
    inputs: *const SnapshotView,
    count: usize,
    available: bool,
) -> u32 {
    catch_unwind(AssertUnwindSafe(|| {
        code((|| {
            let doc = unsafe { string(document, length)? };
            let values = unsafe { snapshots(inputs, count)? };
            let r = registry().lock().map_err(|_| Reason::Unavailable)?;
            r.requests
                .get(&handle)
                .ok_or(Reason::Cancelled)?
                .validate_current(doc, &values, available)
        })())
    }))
    .unwrap_or(Reason::InvalidSnapshot as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase3_request_cancel(handle: u64) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Ok(mut r) = registry().lock() {
            r.requests.remove(&handle);
        }
    }));
}
/// # Safety
/// `output` is writable for `capacity` bytes for this call; no pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase3_message(
    reason: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    if output.is_null() || capacity == 0 || capacity > 4096 {
        return 0;
    }
    let value = match reason {
        1 => Reason::Unsupported,
        2 => Reason::InputCount,
        3 => Reason::InvalidTopology,
        4 => Reason::ClosedSolidRequired,
        5 => Reason::MixedInputs,
        6 => Reason::Protected,
        7 => Reason::Stale,
        8 => Reason::Unavailable,
        9 => Reason::Cancelled,
        11 => Reason::InvalidOptions,
        _ => Reason::InvalidSnapshot,
    }
    .message()
    .as_bytes();
    let length = value.len().min(capacity - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), output, length);
        *output.add(length) = 0;
    }
    length
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase3_surface_options(
    kind: u32,
    style: u32,
    closed: bool,
    mode: u32,
    points: u32,
    tolerance: f64,
    count: usize,
) -> u32 {
    code(crate::phase3_modeling::surface_options(
        kind, style, closed, mode, points, tolerance, count,
    ))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase3_join_route(kind: u32) -> bool {
    crate::phase3_modeling::join_uses_surface_adapter(
        crate::phase3_modeling::Kind::from_u32(kind)
            .unwrap_or(crate::phase3_modeling::Kind::Unknown),
    )
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase3_kernel_tolerance() -> f64 {
    crate::phase3_modeling::KERNEL_TOLERANCE
}
