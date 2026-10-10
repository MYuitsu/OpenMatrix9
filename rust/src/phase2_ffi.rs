//! C ABI: caller retains valid, aligned input memory for the entire call. Bounds
//! are checked before slices; arbitrary address validity is a caller obligation.
//! Stored sessions own copies. Handles never contain native pointers or repeat.
use crate::phase2_curve::{Basis, join_options, validate_wire};
use crate::phase2_session::{CurveSession, RebuildInput, RebuildOptions, RebuildSession, Witness};
pub use crate::phase2_snap_ffi::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
pub(crate) fn allocate_handle() -> Result<u64, String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_update(
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
        |n| n.checked_add(1),
    )
    .map_err(|_| "Rust handle space exhausted".into())
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9BasisInput {
    pub degree: f64,
    pub periodic: u32,
    pub poles: *const f64,
    pub poles_len: usize,
    pub weights: *const f64,
    pub weights_len: usize,
    pub knots: *const f64,
    pub knots_len: usize,
    pub multiplicities: *const f64,
    pub multiplicities_len: usize,
    pub first: f64,
    pub last: f64,
}
#[repr(C)]
#[derive(Default)]
pub struct Om9BasisOutput {
    pub degree: f64,
    pub periodic: u32,
    pub poles: *mut f64,
    pub poles_len: usize,
    pub weights: *mut f64,
    pub weights_len: usize,
    pub knots: *mut f64,
    pub knots_len: usize,
    pub multiplicities: *mut f64,
    pub multiplicities_len: usize,
    pub first: f64,
    pub last: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9WitnessInput {
    pub document: u64,
    pub object: u64,
    pub generation: u64,
    pub signature: *const u8,
    pub signature_len: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9WireInput {
    pub vertex_degrees: *const u64,
    pub vertex_degrees_len: usize,
    pub ordered_edges: *const u64,
    pub ordered_edges_len: usize,
    pub expected_edges: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9RebuildOptions {
    pub degree: usize,
    pub point_count: usize,
    pub delete_input: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9RebuildInput {
    pub witness: Om9WitnessInput,
    pub has_dependents: u32,
}
#[derive(Default)]
struct Registry {
    curves: HashMap<u64, CurveSession>,
    rebuilds: HashMap<u64, RebuildSession>,
}
fn registry() -> std::sync::MutexGuard<'static, Registry> {
    static STORE: OnceLock<Mutex<Registry>> = OnceLock::new();
    STORE
        .get_or_init(|| Mutex::new(Registry::default()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}
impl Registry {
    fn handle(&mut self) -> Result<u64, String> {
        if self.curves.len() + self.rebuilds.len() >= 1024 {
            return Err("Rust editor session budget exhausted".into());
        }
        allocate_handle()
    }
}
thread_local! {static ERRORS:RefCell<HashMap<u64,String>>=RefCell::new(HashMap::new());}
pub(crate) fn report<T>(handle: u64, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(value) => {
            ERRORS.with(|e| {
                e.borrow_mut().remove(&handle);
            });
            Some(value)
        }
        Err(error) => {
            ERRORS.with(|e| {
                e.borrow_mut().insert(handle, error);
            });
            None
        }
    }
}
pub(crate) fn aligned<T>(p: *const T) -> bool {
    !p.is_null() && (p as usize).is_multiple_of(std::mem::align_of::<T>())
}
pub(crate) unsafe fn record<'a, T>(p: *const T) -> Result<&'a T, String> {
    if !aligned(p) {
        return Err("Null or misaligned typed input".into());
    }
    Ok(unsafe { &*p })
}
pub(crate) unsafe fn buffer<'a, T>(p: *const T, n: usize, cap: usize) -> Result<&'a [T], String> {
    if n > cap
        || n.checked_mul(std::mem::size_of::<T>())
            .is_none_or(|bytes| bytes > isize::MAX as usize)
    {
        return Err("Input buffer exceeds allocation budget".into());
    }
    if n == 0 {
        return Ok(&[]);
    }
    if !aligned(p) {
        return Err("Null or misaligned input buffer".into());
    }
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
fn integer(value: f64, lo: usize, hi: usize) -> Result<usize, String> {
    if !value.is_finite() || value.fract() != 0. || value < lo as f64 || value > hi as f64 {
        return Err("Invalid integer field".into());
    }
    Ok(value as usize)
}
unsafe fn basis(p: *const Om9BasisInput) -> Result<Basis, String> {
    let b = unsafe { record(p) }?;
    let degree = integer(b.degree, 1, 25)?;
    if b.periodic > 1
        || b.poles_len > 4096 * 3
        || !b.poles_len.is_multiple_of(3)
        || b.weights_len != b.poles_len / 3
        || b.knots_len != b.multiplicities_len
    {
        return Err("Invalid basis dimensions".into());
    }
    let poles = unsafe { buffer(b.poles, b.poles_len, 4096 * 3) }?
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| [p[0], p[1], p[2]])
        .collect();
    let weights = unsafe { buffer(b.weights, b.weights_len, 4096) }?.to_vec();
    let knots = unsafe { buffer(b.knots, b.knots_len, 4096) }?.to_vec();
    let multiplicities = unsafe { buffer(b.multiplicities, b.multiplicities_len, 4096) }?
        .iter()
        .map(|n| integer(*n, 1, degree + 1))
        .collect::<Result<Vec<_>, _>>()?;
    let result = Basis {
        degree,
        periodic: b.periodic == 1,
        poles,
        weights,
        knots,
        multiplicities,
        first: b.first,
        last: b.last,
    };
    result.validate()?;
    Ok(result)
}
unsafe fn witness(p: *const Om9WitnessInput) -> Result<Witness, String> {
    let w = unsafe { record(p) }?;
    let bytes = unsafe { buffer(w.signature, w.signature_len, 512) }?;
    let signature = std::str::from_utf8(bytes)
        .map_err(|_| "Witness signature is not UTF-8")?
        .to_owned();
    let result = Witness {
        document: w.document,
        object: w.object,
        generation: w.generation,
        signature,
    };
    result.validate()?;
    Ok(result)
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_basis_validate(p: *const Om9BasisInput) -> bool {
    report(0, unsafe { basis(p) }).is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_wire_validate(p: *const Om9WireInput) -> bool {
    report(
        0,
        (|| {
            let w = unsafe { record(p) }?;
            let degrees = unsafe { buffer(w.vertex_degrees, w.vertex_degrees_len, 1_000_000) }?
                .iter()
                .map(|x| usize::try_from(*x).map_err(|_| "Vertex degree overflow".to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let edges = unsafe { buffer(w.ordered_edges, w.ordered_edges_len, 1_000_000) }?;
            validate_wire(&degrees, edges, w.expected_edges)
        })(),
    )
    .is_some()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_join_options(inputs: usize, edges: usize) -> bool {
    report(0, join_options(inputs, edges)).is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_join_selection(p: *const u64, n: usize, edges: usize) -> bool {
    report(
        0,
        (|| {
            join_options(n, edges)?;
            let tokens = unsafe { buffer(p, n, 16) }?;
            if tokens.contains(&0)
                || tokens
                    .iter()
                    .copied()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != n
            {
                return Err("Join requires distinct native curves".into());
            }
            Ok(())
        })(),
    )
    .is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_session_create(
    w: *const Om9WitnessInput,
    b: *const Om9BasisInput,
) -> u64 {
    report(
        0,
        (|| {
            let s = CurveSession::new(unsafe { witness(w) }?, unsafe { basis(b) }?)?;
            let mut r = registry();
            let h = r.handle()?;
            r.curves.insert(h, s);
            Ok(h)
        })(),
    )
    .unwrap_or(0)
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_session_replace(h: u64, b: *const Om9BasisInput) -> bool {
    report(
        h,
        (|| {
            let b = unsafe { basis(b) }?;
            registry()
                .curves
                .get_mut(&h)
                .ok_or("Unknown curve session")?
                .replace_draft(b)
        })(),
    )
    .is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_session_check(h: u64, w: *const Om9WitnessInput) -> bool {
    report(
        h,
        (|| {
            let w = unsafe { witness(w) }?;
            registry()
                .curves
                .get(&h)
                .ok_or("Unknown curve session")?
                .commit_request(&w)
                .map(|_| ())
        })(),
    )
    .is_some()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_session_drop(h: u64) {
    registry().curves.remove(&h);
    ERRORS.with(|e| {
        e.borrow_mut().remove(&h);
    });
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_session_copy(h: u64, p: *mut Om9BasisOutput) -> bool {
    report(
        h,
        (|| {
            if !aligned(p) {
                return Err("Null or misaligned basis output".into());
            }
            let store = registry();
            let b = store
                .curves
                .get(&h)
                .ok_or("Unknown curve session")?
                .draft()?;
            let o = unsafe { &mut *p };
            let enough = o.poles_len >= b.poles.len() * 3
                && o.weights_len >= b.weights.len()
                && o.knots_len >= b.knots.len()
                && o.multiplicities_len >= b.multiplicities.len()
                && aligned(o.poles)
                && aligned(o.weights)
                && aligned(o.knots)
                && aligned(o.multiplicities);
            o.degree = b.degree as f64;
            o.periodic = u32::from(b.periodic);
            o.first = b.first;
            o.last = b.last;
            o.poles_len = b.poles.len() * 3;
            o.weights_len = b.weights.len();
            o.knots_len = b.knots.len();
            o.multiplicities_len = b.multiplicities.len();
            if !enough {
                return Err("Insufficient caller-owned basis output capacity".into());
            }
            // SAFETY: Caller supplies disjoint valid writable buffers of declared capacity.
            unsafe {
                for (i, point) in b.poles.iter().enumerate() {
                    for (axis, value) in point.iter().enumerate() {
                        o.poles.add(i * 3 + axis).write(*value);
                    }
                }
                std::ptr::copy_nonoverlapping(b.weights.as_ptr(), o.weights, b.weights.len());
                std::ptr::copy_nonoverlapping(b.knots.as_ptr(), o.knots, b.knots.len());
                for (i, value) in b.multiplicities.iter().enumerate() {
                    o.multiplicities.add(i).write(*value as f64);
                }
            }
            Ok(())
        })(),
    )
    .is_some()
}
unsafe fn options(p: *const Om9RebuildOptions) -> Result<RebuildOptions, String> {
    let o = unsafe { record(p) }?;
    if o.delete_input > 1 {
        return Err("Invalid DeleteInput flag".into());
    }
    let result = RebuildOptions {
        degree: o.degree,
        point_count: o.point_count,
        delete_input: o.delete_input == 1,
    };
    result.validate()?;
    Ok(result)
}
unsafe fn inputs(p: *const Om9RebuildInput, n: usize) -> Result<Vec<RebuildInput>, String> {
    unsafe { buffer(p, n, 16) }?
        .iter()
        .map(|x| {
            if x.has_dependents > 1 {
                return Err("Invalid dependency flag".into());
            }
            Ok(RebuildInput {
                witness: unsafe { witness(&x.witness) }?,
                has_dependents: x.has_dependents == 1,
            })
        })
        .collect()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_rebuild_create(
    p: *const Om9RebuildInput,
    n: usize,
    o: *const Om9RebuildOptions,
) -> u64 {
    report(
        0,
        (|| {
            let s = RebuildSession::new(unsafe { inputs(p, n) }?, unsafe { options(o) }?)?;
            let mut r = registry();
            let h = r.handle()?;
            r.rebuilds.insert(h, s);
            Ok(h)
        })(),
    )
    .unwrap_or(0)
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_rebuild_replace(h: u64, o: *const Om9RebuildOptions) -> bool {
    report(
        h,
        (|| {
            let o = unsafe { options(o) }?;
            registry()
                .rebuilds
                .get_mut(&h)
                .ok_or("Unknown Rebuild session")?
                .replace_options(o)
        })(),
    )
    .is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_rebuild_check(
    h: u64,
    p: *const Om9RebuildInput,
    n: usize,
) -> bool {
    report(
        h,
        (|| {
            let current = unsafe { inputs(p, n) }?;
            registry()
                .rebuilds
                .get(&h)
                .ok_or("Unknown Rebuild session")?
                .commit_request(&current)
                .map(|_| ())
        })(),
    )
    .is_some()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_rebuild_drop(h: u64) {
    registry().rebuilds.remove(&h);
    ERRORS.with(|e| {
        e.borrow_mut().remove(&h);
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_rebuild_input_count(n: usize) -> bool {
    report(0, crate::phase2_session::validate_rebuild_count(n)).is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_rebuild_get(h: u64, p: *mut Om9RebuildOptions) -> bool {
    report(
        h,
        (|| {
            if !aligned(p) {
                return Err("Null or misaligned Rebuild options output".into());
            }
            let r = registry();
            let o = r
                .rebuilds
                .get(&h)
                .ok_or("Unknown Rebuild session")?
                .options()?;
            unsafe {
                p.write(Om9RebuildOptions {
                    degree: o.degree,
                    point_count: o.point_count,
                    delete_input: u32::from(o.delete_input),
                });
            }
            Ok(())
        })(),
    )
    .is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_error(h: u64, p: *mut u8, capacity: usize) -> usize {
    ERRORS.with(|errors| {
        let e = errors.borrow();
        let text = e.get(&h).map(String::as_bytes).unwrap_or(&[]);
        let needed = text.len() + 1;
        if capacity >= needed && aligned(p) && capacity <= isize::MAX as usize {
            unsafe {
                std::ptr::copy_nonoverlapping(text.as_ptr(), p, text.len());
                p.add(text.len()).write(0);
            }
        }
        needed
    })
}
