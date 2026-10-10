//! Matrix boundary. Policy is shared with the FreeCAD OM9 Rust core.
use openmatrix9_rust::{
    layer_clipboard::{self, GeometryObjectBinding, LayerClipboardScope},
    layer_codec::{decode_snapshot, encode_snapshot},
    layer_exchange::{TransferScope, plan_receive},
    layer_state::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, OnceLock},
};

pub fn prepare(input: &[u8], scope: u32, geometry: &[u8]) -> Result<Vec<u8>, LayerError> {
    let scope = match scope {
        1 => LayerClipboardScope::Selected,
        2 => LayerClipboardScope::Session,
        _ => return Err(LayerError::UnsupportedVersion),
    };
    layer_clipboard::prepare_clipboard(&decode_snapshot(input)?, scope, geometry)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    physical_id: String,
    source_id: String,
}
#[derive(Serialize)]
struct Plan<'a> {
    version: u32,
    before: &'a LayerSnapshotV1,
    after: &'a LayerSnapshotV1,
    layer_mapping: &'a BTreeMap<LayerId, LayerId>,
    object_mapping: &'a BTreeMap<ObjectId, ObjectId>,
}
pub fn receive(
    metadata: &[u8],
    geometry: &[u8],
    destination: &[u8],
    bindings: &[u8],
) -> Result<Vec<u8>, LayerError> {
    if metadata.is_empty() {
        return Err(LayerError::ClipboardBinding);
    }
    if bindings.len() > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let received = layer_clipboard::receive_clipboard(geometry, Some(metadata))?;
    let source = received.snapshot.ok_or(LayerError::ClipboardBinding)?;
    let bindings: Vec<Binding> =
        serde_json::from_slice(bindings).map_err(|_| LayerError::InvalidSnapshot)?;
    let views: Vec<_> = bindings
        .iter()
        .map(|b| GeometryObjectBinding {
            physical_id: &b.physical_id,
            source_id: &b.source_id,
        })
        .collect();
    let bound = layer_clipboard::bind_geometry_objects(&source, &views)?;
    let destination = decode_snapshot(destination)?;
    let plan = plan_receive(&bound, &destination, TransferScope::Session)?;
    let charge = plan
        .before
        .metadata_bytes()?
        .checked_add(plan.after.metadata_bytes()?)
        .and_then(|n| n.checked_mul(4))
        .ok_or(LayerError::LimitExceeded)?;
    if charge > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let bytes = serde_json::to_vec(&Plan {
        version: 1,
        before: &plan.before,
        after: &plan.after,
        layer_mapping: &plan.layer_mapping,
        object_mapping: &plan.object_mapping,
    })
    .map_err(|_| LayerError::InvalidSnapshot)?;
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    Ok(bytes)
}
#[derive(Default)]
struct Results {
    next: u64,
    bytes: usize,
    rows: BTreeMap<u64, Vec<u8>>,
}
fn results() -> &'static Mutex<Results> {
    static R: OnceLock<Mutex<Results>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(Results::default()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistentFact {
    child: bool,
    locked: bool,
    visible: bool,
    witness: String,
}
#[derive(Serialize)]
struct PersistentReply {
    persistent_locked: Option<bool>,
    persistent_visible: Option<bool>,
}
/// # Safety
/// Input row/facts readable for this call; writable out. Rust interprets native presence witnesses.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_persistent(
    input: Bytes,
    encode: u32,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { output(out, 0)? };
        let bytes = unsafe { span(input, 8192)? };
        let result = match encode {
            1 => {
                let row: LayerRow =
                    serde_json::from_slice(bytes).map_err(|_| LayerError::InvalidSnapshot)?;
                openmatrix9_rust::layer_native::encode_native_persistent(
                    row.parent_id.is_some(),
                    row.locked,
                    row.visible,
                    openmatrix9_rust::layer_native::PersistentFields {
                        locked: row.persistent_locked,
                        visible: row.persistent_visible,
                    },
                )?
            }
            0 => {
                let facts: PersistentFact =
                    serde_json::from_slice(bytes).map_err(|_| LayerError::InvalidSnapshot)?;
                let fields = openmatrix9_rust::layer_native::decode_native_persistent(
                    facts.child,
                    facts.locked,
                    facts.visible,
                    facts.witness.as_bytes(),
                )?;
                serde_json::to_vec(&PersistentReply {
                    persistent_locked: fields.locked,
                    persistent_visible: fields.visible,
                })
                .map_err(|_| LayerError::InvalidSnapshot)?
            }
            _ => return Err(LayerError::UnsupportedVersion),
        };
        unsafe { output(out, insert(result)?) }
    })
}
fn boundary(f: impl FnOnce() -> Result<(), LayerError>) -> u32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => e as u32,
        Err(_) => LayerError::InvalidSnapshot as u32,
    }
}
fn insert(value: Vec<u8>) -> Result<u64, LayerError> {
    let mut r = results().lock().map_err(|_| LayerError::InvalidHandle)?;
    let bytes = r
        .bytes
        .checked_add(value.len())
        .ok_or(LayerError::LimitExceeded)?;
    if r.rows.len() >= 32 || bytes > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let id = r.next.checked_add(1).ok_or(LayerError::LimitExceeded)?;
    r.next = id;
    r.bytes = bytes;
    r.rows.insert(id, value);
    Ok(id)
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bytes {
    pub data: *const u8,
    pub len: usize,
}
unsafe fn span<'a>(v: Bytes, max: usize) -> Result<&'a [u8], LayerError> {
    if v.len > max || v.len > isize::MAX as usize {
        return Err(LayerError::LimitExceeded);
    }
    if v.len == 0 {
        return Ok(&[]);
    }
    if v.data.is_null() {
        return Err(LayerError::InvalidSnapshot);
    }
    Ok(unsafe { std::slice::from_raw_parts(v.data, v.len) })
}
unsafe fn output<T>(p: *mut T, value: T) -> Result<(), LayerError> {
    if p.is_null() || !(p as usize).is_multiple_of(std::mem::align_of::<T>()) {
        return Err(LayerError::InvalidSnapshot);
    }
    unsafe { p.write(value) };
    Ok(())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_rhino_layer_abi() -> u32 {
    1
}
/// # Safety
/// Readable spans and writable/aligned out. Validates pair before any native archive read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_capture(
    metadata: Bytes,
    geometry: Bytes,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { output(out, 0)? };
        let g = unsafe { span(geometry, layer_clipboard::MAX_GEOMETRY_BYTES)? };
        let m = unsafe { span(metadata, MAX_METADATA_BYTES)? };
        let state = layer_clipboard::receive_clipboard(g, Some(m))?
            .snapshot
            .ok_or(LayerError::ClipboardBinding)?;
        unsafe { output(out, insert(encode_snapshot(&state)?)?) }
    })
}
/// # Safety
/// Input spans readable until return, out writable/aligned u64. No pointer retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_prepare(
    snapshot: Bytes,
    scope: u32,
    geometry: Bytes,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { output(out, 0)? };
        let bytes = prepare(
            unsafe { span(snapshot, MAX_METADATA_BYTES)? },
            scope,
            unsafe { span(geometry, layer_clipboard::MAX_GEOMETRY_BYTES)? },
        )?;
        unsafe { output(out, insert(bytes)?) }
    })
}
/// # Safety
/// Same span/output contract as prepare. Complete provenance is mandatory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_receive(
    metadata: Bytes,
    geometry: Bytes,
    destination: Bytes,
    bindings: Bytes,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { output(out, 0)? };
        let bytes = receive(
            unsafe { span(metadata, MAX_METADATA_BYTES)? },
            unsafe { span(geometry, layer_clipboard::MAX_GEOMETRY_BYTES)? },
            unsafe { span(destination, MAX_METADATA_BYTES)? },
            unsafe { span(bindings, MAX_METADATA_BYTES)? },
        )?;
        unsafe { output(out, insert(bytes)?) }
    })
}
/// # Safety
/// needed writable usize; out writable for capacity. Null/0 queries exact size.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_result(
    handle: u64,
    out: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { output(needed, 0)? };
        let r = results().lock().map_err(|_| LayerError::InvalidHandle)?;
        let value = r.rows.get(&handle).ok_or(LayerError::InvalidHandle)?;
        unsafe { output(needed, value.len())? };
        if capacity < value.len() {
            return Err(LayerError::BufferTooSmall);
        }
        if out.is_null() {
            return Err(LayerError::InvalidSnapshot);
        }
        unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len()) };
        Ok(())
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_rhino_layer_free(handle: u64) -> u32 {
    boundary(|| {
        let mut r = results().lock().map_err(|_| LayerError::InvalidHandle)?;
        let value = r.rows.remove(&handle).ok_or(LayerError::InvalidHandle)?;
        r.bytes -= value.len();
        Ok(())
    })
}
/// # Safety
/// Both spans readable until return. Compares full state, not just generation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_check(before: Bytes, current: Bytes) -> u32 {
    boundary(|| {
        let a = decode_snapshot(unsafe { span(before, MAX_METADATA_BYTES)? })?;
        let b = decode_snapshot(unsafe { span(current, MAX_METADATA_BYTES)? })?;
        if a == b {
            Ok(())
        } else {
            Err(LayerError::Stale)
        }
    })
}
/// # Safety
/// Readable snapshot/writable out contract; preflight before archive serialization.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_rhino_layer_validate(snapshot: Bytes, out: *mut u64) -> u32 {
    boundary(|| {
        unsafe { output(out, 0)? };
        let s = decode_snapshot(unsafe { span(snapshot, MAX_METADATA_BYTES)? })?;
        unsafe { output(out, insert(encode_snapshot(&s)?)?) }
    })
}
