//! Snap ABI owns copied numeric rows/cache/query state only. All native lifetime
//! and projection witnesses stay with the caller. Output buffers are caller owned.
use crate::phase2_ffi::{aligned, allocate_handle, buffer, record, report};
use crate::phase2_snap::*;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9ViewKey {
    pub document: u64,
    pub view: u64,
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub camera: [f64; 16],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9SnapRow {
    pub object: u64,
    pub bounds: [f64; 4],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9SnapLimits {
    pub objects: usize,
    pub per_object: usize,
    pub total: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9SnapPoint {
    pub world: [f64; 3],
    pub screen: [f64; 3],
}
#[repr(C)]
#[derive(Default)]
pub struct Om9SnapResult {
    pub complete: u32,
    pub picked: u32,
    pub point: [f64; 3],
    pub nearby_objects: usize,
    pub visited_objects: usize,
    pub visited_topology: usize,
    pub generated: usize,
    pub max_per_object: usize,
}
struct Query {
    index: u64,
    active: Option<SnapQuery>,
    result: Option<SnapResult>,
}
#[derive(Default)]
struct Store {
    indexes: HashMap<u64, SnapIndex>,
    queries: HashMap<u64, Query>,
}
fn store() -> std::sync::MutexGuard<'static, Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE
        .get_or_init(|| Mutex::new(Store::default()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}
fn query(s: &mut Store, h: u64) -> Result<&mut SnapQuery, String> {
    s.queries
        .get_mut(&h)
        .and_then(|q| q.active.as_mut())
        .ok_or_else(|| "Unknown or finished Rust snap query".into())
}
unsafe fn key(p: *const Om9ViewKey) -> Result<ViewKey, String> {
    let k = unsafe { record(p) }?;
    Ok(ViewKey {
        document: k.document,
        view: k.view,
        generation: k.generation,
        width: k.width,
        height: k.height,
        camera: k.camera,
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_snap_create() -> u64 {
    report(
        0,
        (|| {
            let mut s = store();
            if s.indexes.len() >= 16 {
                return Err("Rust snap index budget exhausted".into());
            }
            let h = allocate_handle()?;
            s.indexes.insert(h, SnapIndex::default());
            Ok(h)
        })(),
    )
    .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_snap_drop(h: u64) {
    let mut s = store();
    s.indexes.remove(&h);
    s.queries.retain(|_, q| q.index != h);
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_snap_begin(h: u64, p: *const Om9ViewKey) -> bool {
    report(
        h,
        (|| {
            let key = unsafe { key(p) }?;
            store()
                .indexes
                .get_mut(&h)
                .ok_or("Unknown snap index")?
                .begin_build(key)
        })(),
    )
    .is_some()
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_snap_add(h: u64, p: *const Om9SnapRow) -> bool {
    report(
        h,
        (|| {
            let r = unsafe { record(p) }?;
            store()
                .indexes
                .get_mut(&h)
                .ok_or("Unknown snap index")?
                .add_row(Row {
                    object: r.object,
                    bounds: r.bounds,
                })
        })(),
    )
    .is_some()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_snap_finish(h: u64) -> bool {
    report(
        h,
        store()
            .indexes
            .get_mut(&h)
            .ok_or_else(|| "Unknown snap index".into())
            .and_then(SnapIndex::finish_build),
    )
    .is_some()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_snap_abort(h: u64) {
    if let Some(i) = store().indexes.get_mut(&h) {
        i.abort_build();
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_snap_invalidate(h: u64) {
    if let Some(i) = store().indexes.get_mut(&h) {
        i.invalidate();
    }
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_snap_matches(h: u64, p: *const Om9ViewKey) -> bool {
    report(
        h,
        (|| {
            let key = unsafe { key(p) }?;
            Ok(store()
                .indexes
                .get(&h)
                .ok_or("Unknown snap index")?
                .matches(&key))
        })(),
    )
    .unwrap_or(false)
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_query_create(
    h: u64,
    xy: *const f64,
    radius: f64,
    modes: u32,
    p: *const Om9SnapLimits,
) -> u64 {
    report(
        0,
        (|| {
            let xy = unsafe { buffer(xy, 2, 2) }?;
            let l = unsafe { record(p) }?;
            let limits = Limits {
                objects: l.objects,
                per_object: l.per_object,
                total: l.total,
            };
            let mut s = store();
            if s.queries.len() >= 128 {
                return Err("Rust snap query budget exhausted".into());
            }
            let q = s
                .indexes
                .get_mut(&h)
                .ok_or("Unknown snap index")?
                .start_query([xy[0], xy[1]], radius, modes, limits)?;
            let token = allocate_handle()?;
            s.queries.insert(
                token,
                Query {
                    index: h,
                    active: Some(q),
                    result: None,
                },
            );
            Ok(token)
        })(),
    )
    .unwrap_or(0)
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_query_objects(h: u64, p: *mut u64, capacity: usize) -> usize {
    report(
        h,
        (|| {
            let mut s = store();
            let q = query(&mut s, h)?;
            let objects = q.objects();
            let n = objects.len();
            if capacity >= n && n > 0 {
                if !aligned(p) || capacity > 64 {
                    return Err("Invalid object-token output buffer".into());
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(objects.as_ptr(), p, n);
                }
            }
            Ok(n)
        })(),
    )
    .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_query_budget(h: u64, object: u64, mode: u32) -> usize {
    report(
        h,
        (|| {
            let mut s = store();
            let q = query(&mut s, h)?;
            if ![2, 4, 8].contains(&mode) {
                return Err("Invalid snap mode".into());
            }
            Ok(q.remaining_budget(object))
        })(),
    )
    .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_query_cached(h: u64, object: u64, mode: u32) -> bool {
    report(
        h,
        (|| {
            let mut s = store();
            query(&mut s, h)?.cached(object, mode)
        })(),
    )
    .unwrap_or(false)
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_query_consume(
    h: u64,
    object: u64,
    mode: u32,
    p: *const Om9SnapPoint,
    n: usize,
    complete: u32,
    visited: usize,
) -> bool {
    report(
        h,
        (|| {
            let mut s = store();
            let q = query(&mut s, h)?;
            let budget = q.remaining_budget(object);
            if complete > 1 || n > budget {
                q.abort();
                return Err("Native extraction exceeds Rust candidate budget".into());
            }
            let raw = match unsafe { buffer(p, n, budget) } {
                Ok(x) => x,
                Err(e) => {
                    q.abort();
                    return Err(e);
                }
            };
            let points: Vec<_> = raw.iter().map(|p| (p.world, p.screen)).collect();
            q.consume(object, mode, &points, complete == 1, visited)
        })(),
    )
    .is_some()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_query_abort(h: u64) {
    if let Ok(q) = query(&mut store(), h) {
        q.abort();
    }
}
/// # Safety
/// Caller retains valid aligned input buffers for the call and disjoint writable
/// output buffers of the declared capacity. No native pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_phase2_query_finish(h: u64, p: *mut Om9SnapResult) -> bool {
    report(
        h,
        (|| {
            if !aligned(p) {
                return Err("Null or misaligned snap output".into());
            }
            let mut s = store();
            let entry = s.queries.get_mut(&h).ok_or("Unknown snap query")?;
            let index = entry.index;
            let active = entry.active.take().ok_or("Snap query already finished")?;
            let result = s
                .indexes
                .get_mut(&index)
                .ok_or("Snap index dropped")?
                .publish_query(active)?;
            let output = Om9SnapResult {
                complete: u32::from(result.complete),
                picked: u32::from(result.picked),
                point: result.point,
                nearby_objects: result.nearby_objects,
                visited_objects: result.visited_objects,
                visited_topology: result.visited_topology,
                generated: result.generated,
                max_per_object: result.max_per_object,
            };
            unsafe {
                p.write(output);
            }
            if let Some(entry) = s.queries.get_mut(&h) {
                entry.result = Some(result);
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
pub unsafe extern "C" fn om9_phase2_query_points(h: u64, p: *mut f64, capacity: usize) -> usize {
    report(
        h,
        (|| {
            let s = store();
            let result = s
                .queries
                .get(&h)
                .and_then(|q| q.result.as_ref())
                .ok_or("No finished snap result")?;
            let n = result.points.len() * 3;
            if capacity >= n && n > 0 {
                if !aligned(p) || capacity > 8192 * 3 {
                    return Err("Invalid candidate output capacity".into());
                }
                for (i, point) in result.points.iter().enumerate() {
                    for (axis, value) in point.iter().enumerate() {
                        unsafe {
                            p.add(i * 3 + axis).write(*value);
                        }
                    }
                }
            }
            Ok(n)
        })(),
    )
    .unwrap_or(0)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_phase2_query_drop(h: u64) {
    store().queries.remove(&h);
}
