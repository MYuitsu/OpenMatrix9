//! Versioned host boundary. Native views are copied; results use caller buffers.
//! GUI/native lifetimes never escape into a Rust snapshot. Registry IDs never wrap.
use crate::{layer_exchange::*, layer_state::*};
use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, OnceLock},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ByteView {
    pub data: *const u8,
    pub len: usize,
}
#[repr(C)]
pub struct LayerView {
    pub id: ByteView,
    pub parent: ByteView,
    pub name: ByteView,
    pub path: ByteView,
    pub rgb: [u8; 3],
    pub locked: u8,
    pub visible: u8,
    pub persistent_locked: i8,
    pub persistent_visible: i8,
    pub reserved: u8,
}
#[repr(C)]
pub struct ObjectView {
    pub id: ByteView,
    pub layer: ByteView,
    pub rgb: [u8; 3],
    pub locked: u8,
    pub visible: u8,
    pub color_source: u8,
    pub reserved: [u8; 2],
}
#[repr(C)]
pub struct LegacyObjectView {
    pub id: ByteView,
    pub path: ByteView,
    pub rgb: [u8; 3],
    pub locked: u8,
    pub visible: u8,
    pub path_present: u8,
    pub reserved: [u8; 2],
}
#[repr(C)]
pub struct SnapshotView {
    pub version: u32,
    pub reserved: u32,
    pub document: ByteView,
    pub generation: u64,
    pub active: ByteView,
    pub layers: *const LayerView,
    pub layer_count: usize,
    pub objects: *const ObjectView,
    pub object_count: usize,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct EffectiveView {
    pub rgb: [u8; 3],
    pub locked: u8,
    pub visible: u8,
    pub selectable: u8,
    pub snap_eligible: u8,
    pub reserved: u8,
}
impl From<EffectiveState> for EffectiveView {
    fn from(s: EffectiveState) -> Self {
        Self {
            rgb: s.rgb,
            locked: u8::from(s.locked),
            visible: u8::from(s.visible),
            selectable: u8::from(s.selectable()),
            snap_eligible: u8::from(s.snap_eligible()),
            reserved: 0,
        }
    }
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SnapshotCounts {
    pub layer_count: usize,
    pub object_count: usize,
    pub generation: u64,
    pub active_layer_index: usize,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct LayerInfo {
    pub rgb: [u8; 3],
    pub locked: u8,
    pub visible: u8,
    pub persistent_locked: i8,
    pub persistent_visible: i8,
    pub reserved: u8,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ObjectInfo {
    pub rgb: [u8; 3],
    pub locked: u8,
    pub visible: u8,
    pub color_source: u8,
    pub reserved: [u8; 2],
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct ClipboardInfo {
    pub version: u32,
    pub evidence: u32,
    pub scope: u32,
    pub geometry_version: u32,
    pub geometry_length: u64,
    pub metadata_length: u64,
}
#[repr(C)]
pub struct GeometryBindingView {
    pub physical: ByteView,
    pub source: ByteView,
}
#[repr(C)]
pub struct CollectionNodeView {
    pub id: ByteView,
    pub label: ByteView,
    pub native_type: ByteView,
    pub children: *const ByteView,
    pub child_count: usize,
    pub role: u32,
    pub reserved: u32,
}

struct SnapshotRecord {
    state: LayerSnapshotV1,
    by_object: HashMap<String, usize>,
    by_layer: HashMap<String, usize>,
    objects: Vec<EffectiveState>,
    layers: Vec<EffectiveState>,
    charge: usize,
}
impl SnapshotRecord {
    fn new(state: LayerSnapshotV1) -> Result<Self, LayerError> {
        let charge = state
            .metadata_bytes()?
            .checked_mul(4)
            .ok_or(LayerError::LimitExceeded)?;
        if charge > MAX_REGISTRY_BYTES {
            return Err(LayerError::LimitExceeded);
        }
        let index = LayerIndex::new(&state)?;
        let objects = state
            .objects
            .iter()
            .map(|o| index.effective(&o.id))
            .collect::<Result<_, _>>()?;
        let layers = state
            .layers
            .iter()
            .map(|l| index.layer_effective(&l.source_id))
            .collect::<Result<_, _>>()?;
        drop(index);
        let by_object = state
            .objects
            .iter()
            .enumerate()
            .map(|(i, o)| (o.id.clone(), i))
            .collect();
        let by_layer = state
            .layers
            .iter()
            .enumerate()
            .map(|(i, l)| (l.source_id.clone(), i))
            .collect();
        Ok(Self {
            state,
            by_object,
            by_layer,
            objects,
            layers,
            charge,
        })
    }
    fn effective(&self, id: &str) -> Result<EffectiveState, LayerError> {
        self.by_object
            .get(id)
            .map(|&i| self.objects[i])
            .ok_or(LayerError::MissingObject)
    }
    fn layer_effective(&self, id: &str) -> Result<EffectiveState, LayerError> {
        self.by_layer
            .get(id)
            .map(|&i| self.layers[i])
            .ok_or(LayerError::MissingLayer)
    }
}
enum Entry {
    Snapshot(Arc<SnapshotRecord>),
    Plan(Arc<LayerApplyPlan>, usize),
    Binding(Arc<Vec<u8>>),
    Collection(Arc<Vec<u8>>),
    SourcePayload(Arc<Vec<u8>>, String),
}
impl Entry {
    fn charge(&self) -> usize {
        match self {
            Self::Snapshot(s) => s.charge,
            Self::Plan(_, bytes) => *bytes,
            Self::Binding(bytes) => bytes.capacity() + std::mem::size_of::<Vec<u8>>(),
            Self::Collection(bytes) => bytes.capacity() + std::mem::size_of::<Vec<u8>>(),
            Self::SourcePayload(bytes, digest) => {
                bytes.capacity()
                    + digest.capacity()
                    + std::mem::size_of::<Vec<u8>>()
                    + std::mem::size_of::<String>()
            }
        }
    }
}
const MAX_REGISTRY_BYTES: usize = 512 * 1024 * 1024;
const MAX_HANDLES: usize = 128;
#[derive(Default)]
struct Registry {
    next: u64,
    bytes: usize,
    entries: HashMap<u64, Entry>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}
fn insert(entry: Entry) -> Result<u64, LayerError> {
    let mut r = registry().lock().map_err(|_| LayerError::InvalidHandle)?;
    let bytes = r
        .bytes
        .checked_add(entry.charge())
        .ok_or(LayerError::LimitExceeded)?;
    if r.entries.len() >= MAX_HANDLES || bytes > MAX_REGISTRY_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let id = r.next.checked_add(1).ok_or(LayerError::LimitExceeded)?;
    r.next = id;
    r.bytes = bytes;
    r.entries.insert(id, entry);
    Ok(id)
}
fn snapshot(handle: u64) -> Result<Arc<SnapshotRecord>, LayerError> {
    match registry()
        .lock()
        .map_err(|_| LayerError::InvalidHandle)?
        .entries
        .get(&handle)
    {
        Some(Entry::Snapshot(s)) => Ok(Arc::clone(s)),
        _ => Err(LayerError::InvalidHandle),
    }
}
fn plan(handle: u64) -> Result<Arc<LayerApplyPlan>, LayerError> {
    match registry()
        .lock()
        .map_err(|_| LayerError::InvalidHandle)?
        .entries
        .get(&handle)
    {
        Some(Entry::Plan(p, _)) => Ok(Arc::clone(p)),
        _ => Err(LayerError::InvalidHandle),
    }
}
#[derive(Clone, Copy)]
enum HandleKind {
    Snapshot,
    Plan,
    Binding,
    Collection,
    SourcePayload,
}
fn free(handle: u64, kind: HandleKind) -> Result<(), LayerError> {
    let mut r = registry().lock().map_err(|_| LayerError::InvalidHandle)?;
    let entry = r.entries.get(&handle).ok_or(LayerError::InvalidHandle)?;
    if !matches!(
        (entry, kind),
        (Entry::Snapshot(_), HandleKind::Snapshot)
            | (Entry::Plan(..), HandleKind::Plan)
            | (Entry::Binding(_), HandleKind::Binding)
            | (Entry::Collection(_), HandleKind::Collection)
            | (Entry::SourcePayload(..), HandleKind::SourcePayload)
    ) {
        return Err(LayerError::InvalidHandle);
    }
    let charge = entry.charge();
    r.entries.remove(&handle);
    r.bytes -= charge;
    Ok(())
}
fn boundary(f: impl FnOnce() -> Result<(), LayerError>) -> u32 {
    catch_unwind(AssertUnwindSafe(f))
        .unwrap_or(Err(LayerError::InvalidSnapshot))
        .err()
        .map_or(0, |e| e as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_collection_check_size(bytes: usize) -> u32 {
    boundary(|| {
        if bytes <= MAX_METADATA_BYTES {
            Ok(())
        } else {
            Err(LayerError::LimitExceeded)
        }
    })
}
/// Native object-role and child-link facts are copied into safe owned values.
/// No geometry, document pointers, callbacks or mutable host state is retained.
/// # Safety
/// Node/child/selection spans are readable until return; out is writable u64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_collection_prepare(
    nodes: *const CollectionNodeView,
    count: usize,
    scope: u32,
    selected: *const ByteView,
    selected_count: usize,
    out: *mut u64,
) -> u32 {
    use crate::layer_collection::{Node, Role, Scope};
    boundary(|| {
        unsafe { write(out, 0)? };
        let scope = match scope {
            1 => Scope::Selected,
            2 => Scope::Session,
            _ => return Err(LayerError::UnsupportedVersion),
        };
        let views = unsafe { array(nodes, count, MAX_OBJECTS)? };
        // Validate and charge every borrowed span before allocating owned nodes.
        let mut charge = 0usize;
        for v in views {
            if v.reserved != 0 {
                return Err(LayerError::InvalidSnapshot);
            }
            let id = unsafe { text(v.id, 1024)? };
            let label = unsafe { text(v.label, 4096)? };
            let kind = unsafe { text(v.native_type, 1024)? };
            charge = charge
                .checked_add(std::mem::size_of::<Node>() + id.len() + label.len() + kind.len())
                .ok_or(LayerError::LimitExceeded)?;
            for &child in unsafe { array(v.children, v.child_count, MAX_OBJECTS)? } {
                charge = charge
                    .checked_add(
                        std::mem::size_of::<String>() + unsafe { text(child, 1024)? }.len(),
                    )
                    .ok_or(LayerError::LimitExceeded)?;
            }
            if charge > MAX_METADATA_BYTES {
                return Err(LayerError::LimitExceeded);
            }
        }
        let mut facts = Vec::new();
        facts
            .try_reserve_exact(count)
            .map_err(|_| LayerError::LimitExceeded)?;
        for v in views {
            let role = match v.role {
                1 => Role::Geometry,
                2 => Role::Group,
                3 => Role::Aggregate,
                4 => Role::Definition,
                5 => Role::Instance,
                6 => Role::Metadata,
                7 => Role::Unsupported,
                _ => return Err(LayerError::InvalidSnapshot),
            };
            facts.push(Node {
                id: unsafe { text(v.id, 1024)? }.into(),
                label: unsafe { text(v.label, 4096)? }.into(),
                native_type: unsafe { text(v.native_type, 1024)? }.into(),
                children: unsafe { selection(v.children, v.child_count)? },
                role,
            });
        }
        let selected = unsafe { selection(selected, selected_count)? };
        let result = crate::layer_collection::collect(&facts, scope, &selected)?;
        let json = serde_json::to_vec(&result).map_err(|_| LayerError::InvalidSnapshot)?;
        if json.len() > MAX_METADATA_BYTES {
            return Err(LayerError::LimitExceeded);
        }
        let handle = insert(Entry::Collection(Arc::new(json)))?;
        unsafe { write(out, handle) }
    })
}
/// Copy cached owned metadata only; collecting and graph validation run once.
/// # Safety
/// needed is writable usize; output has capacity writable bytes until return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_collection_json(
    handle: u64,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let bytes = match registry()
            .lock()
            .map_err(|_| LayerError::InvalidHandle)?
            .entries
            .get(&handle)
        {
            Some(Entry::Collection(value)) => Arc::clone(value),
            _ => return Err(LayerError::InvalidHandle),
        };
        let text = std::str::from_utf8(&bytes).map_err(|_| LayerError::InvalidSnapshot)?;
        unsafe { copy_text(text, output, capacity, needed) }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_collection_free(handle: u64) -> u32 {
    boundary(|| free(handle, HandleKind::Collection))
}
/// Decode native storage chunks into verified owned bytes. No document pointers
/// or borrowed chunk views survive return. A hash proves bytes, not provenance.
/// # Safety
/// Chunks/text spans are readable until return; out is a writable aligned u64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_source_payload_prepare(
    chunks: *const ByteView,
    count: usize,
    digest: ByteView,
    out: *mut u64,
) -> u32 {
    use crate::layer_source_payload::{
        MAX_CHUNKS, MAX_ENCODED_CHUNK, MAX_PAYLOAD_BYTES, decode_chunks,
    };
    boundary(|| {
        unsafe { write(out, 0)? };
        let views = unsafe { array(chunks, count, MAX_CHUNKS)? };
        let hash = unsafe { text(digest, 64)? };
        let mut texts = Vec::new();
        texts
            .try_reserve_exact(count)
            .map_err(|_| LayerError::LimitExceeded)?;
        for &chunk in views {
            texts.push(unsafe { text(chunk, MAX_ENCODED_CHUNK)? });
        }
        let payload = decode_chunks(&texts, hash, MAX_PAYLOAD_BYTES)?;
        let handle = insert(Entry::SourcePayload(Arc::new(payload), hash.into()))?;
        unsafe { write(out, handle) }
    })
}
/// Copy cached verified binary bytes; decoding/hashing is not repeated.
/// # Safety
/// needed is writable aligned usize; output has capacity writable bytes. It must
/// not alias needed. No caller pointers are retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_source_payload_bytes(
    handle: u64,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let payload = match registry()
            .lock()
            .map_err(|_| LayerError::InvalidHandle)?
            .entries
            .get(&handle)
        {
            Some(Entry::SourcePayload(value, _)) => Arc::clone(value),
            _ => return Err(LayerError::InvalidHandle),
        };
        unsafe { write(needed, payload.len())? };
        if capacity < payload.len() {
            return Err(LayerError::BufferTooSmall);
        }
        if payload.is_empty() {
            return Ok(());
        }
        if output.is_null() {
            return Err(LayerError::InvalidSnapshot);
        }
        unsafe { std::ptr::copy_nonoverlapping(payload.as_ptr(), output, payload.len()) };
        Ok(())
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_source_payload_free(handle: u64) -> u32 {
    boundary(|| free(handle, HandleKind::SourcePayload))
}
/// Bounded copying avoids allocating a second full archive in the native bridge.
/// # Safety
/// output is writable for count bytes, written is writable usize and does not
/// alias output. No pointers are retained; callers retain ownership of buffers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_source_payload_range(
    handle: u64,
    offset: usize,
    output: *mut u8,
    count: usize,
    written: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(written, 0)? };
        if count > crate::layer_source_payload::CHUNK_BYTES {
            return Err(LayerError::LimitExceeded);
        }
        let payload = match registry()
            .lock()
            .map_err(|_| LayerError::InvalidHandle)?
            .entries
            .get(&handle)
        {
            Some(Entry::SourcePayload(value, _)) => Arc::clone(value),
            _ => return Err(LayerError::InvalidHandle),
        };
        let end = offset.checked_add(count).ok_or(LayerError::LimitExceeded)?;
        let bytes = payload
            .get(offset..end)
            .ok_or(LayerError::InvalidSnapshot)?;
        if !bytes.is_empty() {
            if output.is_null() {
                return Err(LayerError::InvalidSnapshot);
            }
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, count) };
        }
        unsafe { write(written, count) }
    })
}
/// Require the stored manifest to agree with the native inventory of this
/// verified payload. Native caller must read that same owned source file.
/// # Safety
/// stored/native byte spans remain readable until return; no pointers retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_source_manifest_verify(
    handle: u64,
    stored: ByteView,
    native: ByteView,
) -> u32 {
    boundary(|| {
        let hash = match registry()
            .lock()
            .map_err(|_| LayerError::InvalidHandle)?
            .entries
            .get(&handle)
        {
            Some(Entry::SourcePayload(_, digest)) => digest.clone(),
            _ => return Err(LayerError::InvalidHandle),
        };
        let stored = unsafe {
            array(
                stored.data,
                stored.len,
                crate::layer_source_payload::MAX_MANIFEST_BYTES,
            )?
        };
        let native = unsafe {
            array(
                native.data,
                native.len,
                crate::layer_source_payload::MAX_MANIFEST_BYTES,
            )?
        };
        crate::layer_source_payload::verify_manifest(stored, native, &hash)
    })
}
/// Verify stored context against two independent native inventories of the
/// payload owned by this handle. Never pass stored metadata as the witness.
/// # Safety
/// All three byte spans remain readable until return; no pointers retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_source_manifest_verify_witness(
    handle: u64,
    stored: ByteView,
    native: ByteView,
    witness: ByteView,
) -> u32 {
    boundary(|| {
        let hash = match registry()
            .lock()
            .map_err(|_| LayerError::InvalidHandle)?
            .entries
            .get(&handle)
        {
            Some(Entry::SourcePayload(_, digest)) => digest.clone(),
            _ => return Err(LayerError::InvalidHandle),
        };
        let spans = [stored, native, witness];
        if spans
            .iter()
            .any(|span| span.len > crate::layer_source_payload::MAX_MANIFEST_BYTES)
        {
            return Err(LayerError::LimitExceeded);
        }
        let stored = unsafe {
            array(
                stored.data,
                stored.len,
                crate::layer_source_payload::MAX_MANIFEST_BYTES,
            )?
        };
        let native = unsafe {
            array(
                native.data,
                native.len,
                crate::layer_source_payload::MAX_MANIFEST_BYTES,
            )?
        };
        let witness = unsafe {
            array(
                witness.data,
                witness.len,
                crate::layer_source_payload::MAX_MANIFEST_BYTES,
            )?
        };
        crate::layer_source_payload::verify_manifest_with_witness(stored, native, witness, &hash)
    })
}
/// # Safety
/// Output span and needed pointer must be writable for this call. No pointers retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_native_persistent_encode(
    child: u8,
    locked: u8,
    visible: u8,
    persistent_locked: i8,
    persistent_visible: i8,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let bytes = crate::layer_native::encode_native_persistent(
            boolean(child)?,
            boolean(locked)?,
            boolean(visible)?,
            crate::layer_native::PersistentFields {
                locked: persistent(persistent_locked)?,
                visible: persistent(persistent_visible)?,
            },
        )?;
        let value = std::str::from_utf8(&bytes).map_err(|_| LayerError::InvalidSnapshot)?;
        unsafe { copy_text(value, output, capacity, needed) }
    })
}
/// # Safety
/// Marker span is readable; result pointer is writable. No native pointers retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_native_persistent_decode(
    child: u8,
    locked: u8,
    visible: u8,
    marker: ByteView,
    out: *mut LayerInfo,
) -> u32 {
    boundary(|| {
        unsafe { write(out, LayerInfo::default())? };
        let fields = crate::layer_native::decode_native_persistent(
            boolean(child)?,
            boolean(locked)?,
            boolean(visible)?,
            unsafe { array(marker.data, marker.len, 256)? },
        )?;
        let flag = |v: Option<bool>| v.map_or(-1, i8::from);
        unsafe {
            write(
                out,
                LayerInfo {
                    locked,
                    visible,
                    persistent_locked: flag(fields.locked),
                    persistent_visible: flag(fields.visible),
                    ..LayerInfo::default()
                },
            )
        }
    })
}
fn binding(handle: u64) -> Result<Arc<Vec<u8>>, LayerError> {
    match registry()
        .lock()
        .map_err(|_| LayerError::InvalidHandle)?
        .entries
        .get(&handle)
    {
        Some(Entry::Binding(bytes)) => Ok(Arc::clone(bytes)),
        _ => Err(LayerError::InvalidHandle),
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_clipboard_check_size(kind: u32, length: usize) -> u32 {
    boundary(|| match kind {
        1 => crate::layer_clipboard::check_geometry_size(length),
        2 if length <= MAX_METADATA_BYTES => Ok(()),
        2 => Err(LayerError::LimitExceeded),
        _ => Err(LayerError::UnsupportedVersion),
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_clipboard_check_backup(bytes: usize, formats: usize) -> u32 {
    boundary(|| crate::layer_clipboard::check_clipboard_backup(bytes, formats))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_clipboard_rollback_owned(
    before: u32,
    published: u32,
    current: u32,
) -> u8 {
    u8::from(crate::layer_clipboard::rollback_owned_publication(
        before, published, current,
    ))
}
/// Hashes borrowed geometry once and stores only owned, bounded metadata.
/// # Safety
/// Geometry span is readable until return; out is writable u64. No pointers retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_clipboard_prepare(
    source: u64,
    scope: u32,
    geometry: ByteView,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let geometry = unsafe {
            array(
                geometry.data,
                geometry.len,
                crate::layer_clipboard::MAX_GEOMETRY_BYTES,
            )?
        };
        let scope = match scope {
            1 => crate::layer_clipboard::LayerClipboardScope::Selected,
            2 => crate::layer_clipboard::LayerClipboardScope::Session,
            _ => return Err(LayerError::UnsupportedVersion),
        };
        let source = snapshot(source)?;
        let bytes = crate::layer_clipboard::prepare_clipboard(&source.state, scope, geometry)?;
        let handle = insert(Entry::Binding(Arc::new(bytes)))?;
        unsafe { write(out, handle) }
    })
}
/// Copies cached metadata without hashing geometry again. No partial writes.
/// # Safety
/// needed is writable usize; output has capacity writable bytes until return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_clipboard_bytes(
    handle: u64,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let bytes = binding(handle)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| LayerError::InvalidSnapshot)?;
        unsafe { copy_text(text, output, capacity, needed) }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_clipboard_free(handle: u64) -> u32 {
    boundary(|| free(handle, HandleKind::Binding))
}
/// Build an owned detached layer overlay after exact native object-tag validation.
/// # Safety
/// Binding array and UTF-8 spans are readable until return; out is writable u64.
/// Caller extracted facts from the digest-verified native geometry archive.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_clipboard_bind_objects(
    source: u64,
    rows: *const GeometryBindingView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let rows = unsafe { array(rows, count, MAX_OBJECTS)? };
        let source = snapshot(source)?;
        let mut bindings = Vec::new();
        bindings
            .try_reserve_exact(rows.len())
            .map_err(|_| LayerError::LimitExceeded)?;
        for row in rows {
            bindings.push(crate::layer_clipboard::GeometryObjectBinding {
                physical_id: unsafe { text(row.physical, 1024)? },
                source_id: unsafe { text(row.source, 1024)? },
            });
        }
        let state = crate::layer_clipboard::bind_geometry_objects(&source.state, &bindings)?;
        let record = SnapshotRecord::new(state)?;
        let handle = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, handle) }
    })
}
/// Geometry-free source-wins palette overlay on an existing detached archive.
/// # Safety
/// IDs are readable UTF-8 spans for this call, out is writable u64. No input retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_subset(
    source: u64,
    ids: *const ByteView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let ids = unsafe { selection(ids, count)? };
        let source = snapshot(source)?;
        let state = crate::layer_retained::subset_retained_snapshot(&source.state, &ids)?;
        let handle = insert(Entry::Snapshot(Arc::new(SnapshotRecord::new(state)?)))?;
        unsafe { write(out, handle) }
    })
}
/// Semantic inventory excludes only native-verified metadata helpers. Does not
/// delete host objects, increment generation or change the complete palette.
/// # Safety
/// Native caller verifies helper identity and no model payload. IDs are readable
/// UTF-8 spans for this call; out is writable u64. No pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_filter_metadata(
    source: u64,
    ids: *const ByteView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let ids = unsafe { selection(ids, count)? };
        let source = snapshot(source)?;
        let state = crate::layer_lifecycle::filter_verified_metadata(&source.state, &ids)?;
        let handle = insert(Entry::Snapshot(Arc::new(SnapshotRecord::new(state)?)))?;
        unsafe { write(out, handle) }
    })
}
/// Geometry-free source-wins palette overlay on an existing detached archive.
/// Destination must contain only palette facts. Unlisted native objects and
/// instance members stay under the native preservation contract.
/// # Safety
/// Binding array/UTF-8 spans are readable until return; out is writable u64.
/// Caller must verify physical IDs against the detached native model before apply.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_retained_overlay(
    source: u64,
    destination: u64,
    rows: *const GeometryBindingView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let rows = unsafe { array(rows, count, MAX_OBJECTS)? };
        let source = snapshot(source)?;
        let destination = snapshot(destination)?;
        let mut bindings = Vec::new();
        bindings
            .try_reserve_exact(rows.len())
            .map_err(|_| LayerError::LimitExceeded)?;
        for row in rows {
            bindings.push(crate::layer_clipboard::GeometryObjectBinding {
                physical_id: unsafe { text(row.physical, 1024)? },
                source_id: unsafe { text(row.source, 1024)? },
            });
        }
        let state = crate::layer_retained::plan_retained_overlay(
            &source.state,
            &destination.state,
            &bindings,
        )?;
        let record = SnapshotRecord::new(state)?;
        let handle = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, handle) }
    })
}
/// Extended metadata must match this exact geometry payload. Absent metadata
/// returns explicit native-only evidence; present invalid metadata never downgrades.
/// Geometry parsing and native object binding must still finish before host mutation.
/// # Safety
/// Input spans are readable and distinct output records writable until return.
/// has_metadata is 0 or 1; absent metadata requires an empty span. No input retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_clipboard_receive(
    geometry: ByteView,
    metadata: ByteView,
    has_metadata: u8,
    out: *mut u64,
    info: *mut ClipboardInfo,
) -> u32 {
    boundary(|| {
        unsafe {
            write(out, 0)?;
            write(info, ClipboardInfo::default())?;
        }
        let geometry = unsafe {
            array(
                geometry.data,
                geometry.len,
                crate::layer_clipboard::MAX_GEOMETRY_BYTES,
            )?
        };
        let metadata = unsafe { array(metadata.data, metadata.len, MAX_METADATA_BYTES)? };
        let present = boolean(has_metadata)?;
        if !present && !metadata.is_empty() {
            return Err(LayerError::InvalidSnapshot);
        }
        let received =
            crate::layer_clipboard::receive_clipboard(geometry, present.then_some(metadata))?;
        let evidence = match received.evidence {
            crate::layer_clipboard::PaletteEvidence::NativeOnly => 1,
            crate::layer_clipboard::PaletteEvidence::Extended => 2,
        };
        let scope = match received.scope {
            None => 0,
            Some(crate::layer_clipboard::LayerClipboardScope::Selected) => 1,
            Some(crate::layer_clipboard::LayerClipboardScope::Session) => 2,
        };
        let handle = match received.snapshot {
            Some(state) => insert(Entry::Snapshot(Arc::new(SnapshotRecord::new(state)?)))?,
            None => 0,
        };
        // Both outputs were validated before allocation/registration above.
        unsafe {
            write(out, handle)?;
            write(
                info,
                ClipboardInfo {
                    version: 1,
                    evidence,
                    scope,
                    geometry_version: received.geometry_version,
                    geometry_length: geometry.len() as u64,
                    metadata_length: metadata.len() as u64,
                },
            )
        }
    })
}
fn aligned<T>(p: *const T) -> bool {
    !p.is_null() && (p as usize).is_multiple_of(std::mem::align_of::<T>())
}
// SAFETY: caller guarantees initialized, readable memory for the complete span until return.
unsafe fn array<'a, T>(p: *const T, count: usize, max: usize) -> Result<&'a [T], LayerError> {
    if count > max
        || count
            .checked_mul(std::mem::size_of::<T>())
            .is_none_or(|n| n > isize::MAX as usize)
    {
        return Err(LayerError::LimitExceeded);
    }
    if count == 0 {
        return Ok(&[]);
    }
    if !aligned(p) {
        return Err(LayerError::InvalidSnapshot);
    }
    Ok(unsafe { std::slice::from_raw_parts(p, count) })
}
// SAFETY: output points to writable, aligned T until return; never retain it.
unsafe fn write<T: Copy>(p: *mut T, value: T) -> Result<(), LayerError> {
    if !aligned(p) {
        return Err(LayerError::InvalidSnapshot);
    }
    unsafe { p.write(value) };
    Ok(())
}
unsafe fn text<'a>(v: ByteView, max: usize) -> Result<&'a str, LayerError> {
    std::str::from_utf8(unsafe { array(v.data, v.len, max)? })
        .map_err(|_| LayerError::InvalidSnapshot)
}
fn boolean(v: u8) -> Result<bool, LayerError> {
    match v {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(LayerError::InvalidSnapshot),
    }
}
fn persistent(v: i8) -> Result<Option<bool>, LayerError> {
    match v {
        -1 => Ok(None),
        0 => Ok(Some(false)),
        1 => Ok(Some(true)),
        _ => Err(LayerError::InvalidSnapshot),
    }
}
unsafe fn read_snapshot(p: *const SnapshotView) -> Result<LayerSnapshotV1, LayerError> {
    let view = &unsafe { array(p, 1, 1)? }[0];
    if view.version != 1 {
        return Err(LayerError::UnsupportedVersion);
    }
    if view.reserved != 0 {
        return Err(LayerError::InvalidSnapshot);
    }
    let layers = unsafe { array(view.layers, view.layer_count, MAX_LAYERS)? };
    let objects = unsafe { array(view.objects, view.object_count, MAX_OBJECTS)? };
    let doc = unsafe { text(view.document, 1024)? };
    let active = unsafe { text(view.active, 1024)? };
    // Bound cumulative input before constructing owned allocations.
    let mut charge = std::mem::size_of::<LayerSnapshotV1>() + doc.len() + active.len();
    for l in layers {
        for (v, max) in [
            (l.id, 1024),
            (l.parent, 1024),
            (l.name, 1024),
            (l.path, 4096),
        ] {
            let s = unsafe { text(v, max)? };
            charge = charge
                .checked_add(s.len())
                .ok_or(LayerError::LimitExceeded)?;
        }
        charge = charge
            .checked_add(
                std::mem::size_of::<LayerRow>() + MAX_DEPTH * std::mem::size_of::<String>(),
            )
            .ok_or(LayerError::LimitExceeded)?;
        if charge > MAX_METADATA_BYTES {
            return Err(LayerError::LimitExceeded);
        }
    }
    for o in objects {
        let id = unsafe { text(o.id, 1024)? };
        let layer = unsafe { text(o.layer, 1024)? };
        charge = charge
            .checked_add(std::mem::size_of::<ObjectLayerRow>() + id.len() + layer.len())
            .ok_or(LayerError::LimitExceeded)?;
        if charge > MAX_METADATA_BYTES {
            return Err(LayerError::LimitExceeded);
        }
    }
    let layers = layers
        .iter()
        .map(|l| {
            if l.reserved != 0 {
                return Err(LayerError::InvalidSnapshot);
            }
            let parent = unsafe { text(l.parent, 1024)? };
            let path = unsafe { text(l.path, 4096)? };
            let parts: Vec<String> = path
                .split("::")
                .take(MAX_DEPTH + 1)
                .map(String::from)
                .collect();
            if parts.len() > MAX_DEPTH {
                return Err(LayerError::LimitExceeded);
            }
            Ok(LayerRow {
                source_id: unsafe { text(l.id, 1024)? }.into(),
                parent_id: (!parent.is_empty()).then(|| parent.into()),
                name: unsafe { text(l.name, 1024)? }.into(),
                path_components: parts,
                rgb: l.rgb,
                locked: boolean(l.locked)?,
                visible: boolean(l.visible)?,
                persistent_locked: persistent(l.persistent_locked)?,
                persistent_visible: persistent(l.persistent_visible)?,
            })
        })
        .collect::<Result<_, _>>()?;
    let objects = objects
        .iter()
        .map(|o| {
            if o.reserved != [0; 2] {
                return Err(LayerError::InvalidSnapshot);
            }
            Ok(ObjectLayerRow {
                id: unsafe { text(o.id, 1024)? }.into(),
                layer_id: unsafe { text(o.layer, 1024)? }.into(),
                rgb: o.rgb,
                locked: boolean(o.locked)?,
                visible: boolean(o.visible)?,
                color_source: match o.color_source {
                    1 => ColorSource::ByLayer,
                    2 => ColorSource::ByObject,
                    _ => return Err(LayerError::InvalidSnapshot),
                },
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(LayerSnapshotV1 {
        document_id: doc.into(),
        generation: view.generation,
        active_layer: (!active.is_empty()).then(|| active.into()),
        layers,
        objects,
    })
}
unsafe fn selection(p: *const ByteView, count: usize) -> Result<Vec<String>, LayerError> {
    let views = unsafe { array(p, count, MAX_OBJECTS)? };
    let mut charge = 0_usize;
    for &v in views {
        let value = unsafe { text(v, 1024)? };
        charge = charge
            .checked_add(std::mem::size_of::<String>() + value.len())
            .ok_or(LayerError::LimitExceeded)?;
        if charge > MAX_METADATA_BYTES {
            return Err(LayerError::LimitExceeded);
        }
    }
    views
        .iter()
        .map(|&v| Ok(unsafe { text(v, 1024)? }.into()))
        .collect()
}
unsafe fn copy_text(
    value: &str,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> Result<(), LayerError> {
    unsafe { write(needed, value.len())? };
    if capacity < value.len() {
        return Err(LayerError::BufferTooSmall);
    }
    if value.is_empty() {
        return Ok(());
    }
    if output.is_null() {
        return Err(LayerError::InvalidSnapshot);
    }
    unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), output, value.len()) };
    Ok(())
}

/// # Safety
/// View/spans remain initialized/readable until return; out is writable u64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_create(
    view: *const SnapshotView,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let record = SnapshotRecord::new(unsafe { read_snapshot(view)? })?;
        let handle = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, handle) }
    })
}
/// Explicit native archive compatibility; canonical JSON/create remain strict.
/// # Safety
/// Same copied input-view and writable output contract as snapshot_create.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_create_native(
    view: *const SnapshotView,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let state = crate::layer_legacy::normalize_native_layers(unsafe { read_snapshot(view)? })?;
        let record = SnapshotRecord::new(state)?;
        let handle = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, handle) }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_snapshot_free(handle: u64) -> u32 {
    boundary(|| free(handle, HandleKind::Snapshot))
}
/// # Safety
/// document is readable UTF-8 for this call, out writable u64. State is owned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_default(document: ByteView, out: *mut u64) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let doc = unsafe { text(document, 1024)? };
        let record = SnapshotRecord::new(crate::layer_document::default_document(doc)?)?;
        let handle = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// document and complete fact spans readable until return; out writable u64.
/// All bytes are copied into owned Rust values; path_present separates unset/empty.
unsafe fn legacy_facts(
    views: *const LegacyObjectView,
    count: usize,
) -> Result<Vec<crate::layer_lifecycle::LegacyObject>, LayerError> {
    let views = unsafe { array(views, count, MAX_OBJECTS)? };
    let mut facts = Vec::with_capacity(views.len());
    let mut bytes = 0usize;
    for view in views {
        if view.reserved != [0; 2] {
            return Err(LayerError::InvalidSnapshot);
        }
        let locked = boolean(view.locked)?;
        let visible = boolean(view.visible)?;
        let present = boolean(view.path_present)?;
        let id = unsafe { text(view.id, 1024)? };
        let path = unsafe { text(view.path, MAX_DEPTH * 1026)? };
        if !present && !path.is_empty() {
            return Err(LayerError::InvalidSnapshot);
        }
        bytes = bytes
            .checked_add(id.len())
            .and_then(|n| n.checked_add(path.len()))
            .and_then(|n| n.checked_add(128))
            .filter(|n| *n <= MAX_METADATA_BYTES)
            .ok_or(LayerError::LimitExceeded)?;
        facts.push(crate::layer_lifecycle::LegacyObject {
            id: id.into(),
            path: present.then(|| path.into()),
            rgb: view.rgb,
            locked,
            visible,
        });
    }
    Ok(facts)
}
/// # Safety
/// document and complete fact spans readable until return; out writable u64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_legacy(
    document: ByteView,
    views: *const LegacyObjectView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let document = unsafe { text(document, 1024)? };
        let facts = unsafe { legacy_facts(views, count)? };
        let state = crate::layer_lifecycle::migrate_legacy(document, &facts)?;
        let handle = insert(Entry::Snapshot(Arc::new(SnapshotRecord::new(state)?)))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// Complete native fact spans readable for this call; out writable u64 plan.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_reconcile(
    current: u64,
    views: *const LegacyObjectView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let state = snapshot(current)?;
        let facts = unsafe { legacy_facts(views, count)? };
        let plan = crate::layer_lifecycle::plan_reconcile_legacy(&state.state, &facts)?;
        let bytes = plan
            .before
            .metadata_bytes()?
            .checked_add(plan.after.metadata_bytes()?)
            .and_then(|n| n.checked_mul(4))
            .ok_or(LayerError::LimitExceeded)?;
        let handle = insert(Entry::Plan(Arc::new(plan), bytes))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// Complete live identity spans readable for this call; out writable u64 plan.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_observed(
    current: u64,
    ids: *const ByteView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let state = snapshot(current)?;
        let ids = unsafe { selection(ids, count)? };
        let plan = crate::layer_lifecycle::plan_observed(&state.state, &ids)?;
        let bytes = plan
            .before
            .metadata_bytes()?
            .checked_add(plan.after.metadata_bytes()?)
            .and_then(|n| n.checked_mul(4))
            .ok_or(LayerError::LimitExceeded)?;
        let handle = insert(Entry::Plan(Arc::new(plan), bytes))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// output writable for capacity, needed writable usize. No partial JSON writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_panel(
    handle: u64,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let state = snapshot(handle)?;
        let encoded = crate::layer_document::panel_json_cached(&state.state, &state.layers)?;
        let text = std::str::from_utf8(&encoded).map_err(|_| LayerError::InvalidSnapshot)?;
        unsafe { copy_text(text, output, capacity, needed) }
    })
}
/// # Safety
/// text/selected ID spans readable for call; out writable aligned u64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_text(
    current: u64,
    command: ByteView,
    ids: *const ByteView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let state = snapshot(current)?;
        let command = unsafe { text(command, 32768)? };
        let selected = unsafe { selection(ids, count)? };
        let p = crate::layer_document::plan_text(&state.state, command, &selected)?;
        let bytes = p
            .before
            .metadata_bytes()?
            .checked_add(p.after.metadata_bytes()?)
            .and_then(|v| v.checked_mul(4))
            .ok_or(LayerError::LimitExceeded)?;
        let handle = insert(Entry::Plan(Arc::new(p), bytes))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// JSON bytes are readable for this call; out is writable u64. No input pointer retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_from_json(input: ByteView, out: *mut u64) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let bytes = unsafe { array(input.data, input.len, MAX_METADATA_BYTES)? };
        let state = crate::layer_codec::decode_snapshot(bytes)?;
        let record = SnapshotRecord::new(state)?;
        let handle = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// needed is writable usize; output is writable for the returned byte count.
/// BufferTooSmall writes no partial JSON. Caller owns all result bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_json(
    handle: u64,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let state = snapshot(handle)?;
        let encoded = crate::layer_codec::encode_snapshot(&state.state)?;
        let text = std::str::from_utf8(&encoded).map_err(|_| LayerError::InvalidSnapshot)?;
        unsafe { copy_text(text, output, capacity, needed) }
    })
}
/// # Safety
/// Identity bytes readable until return; out writable EffectiveView. No pointers retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_effective(
    handle: u64,
    identity: ByteView,
    out: *mut EffectiveView,
) -> u32 {
    boundary(|| {
        let s = snapshot(handle)?;
        let id = unsafe { text(identity, 1024)? };
        unsafe { write(out, s.effective(id)?.into()) }
    })
}
/// # Safety
/// Selection views/bytes readable for call; destination is a bounded UTF-8 view.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_can_mutate(
    handle: u64,
    ids: *const ByteView,
    count: usize,
    operation: u32,
    destination: ByteView,
) -> u32 {
    boundary(|| {
        let s = snapshot(handle)?;
        let op = match operation {
            1 => LayerOperation::Edit,
            2 => LayerOperation::Transform,
            3 => LayerOperation::Delete,
            4 => LayerOperation::AssignLayer(unsafe { text(destination, 1024)? }.into()),
            5 => LayerOperation::Export,
            _ => return Err(LayerError::UnsupportedVersion),
        };
        check_mutation(
            &unsafe { selection(ids, count)? },
            &op,
            |id| s.effective(id),
            |id| s.layer_effective(id),
        )
    })
}
/// # Safety
/// out is writable SnapshotCounts until return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_counts(handle: u64, out: *mut SnapshotCounts) -> u32 {
    boundary(|| {
        let s = snapshot(handle)?;
        unsafe {
            write(
                out,
                SnapshotCounts {
                    layer_count: s.state.layers.len(),
                    object_count: s.state.objects.len(),
                    generation: s.state.generation,
                    active_layer_index: s
                        .state
                        .active_layer
                        .as_ref()
                        .and_then(|id| s.by_layer.get(id))
                        .copied()
                        .unwrap_or(usize::MAX),
                },
            )
        }
    })
}
/// # Safety
/// out is writable LayerInfo until return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_layer(
    handle: u64,
    index: usize,
    out: *mut LayerInfo,
) -> u32 {
    boundary(|| {
        let s = snapshot(handle)?;
        let l = s.state.layers.get(index).ok_or(LayerError::MissingLayer)?;
        unsafe {
            write(
                out,
                LayerInfo {
                    rgb: l.rgb,
                    locked: u8::from(l.locked),
                    visible: u8::from(l.visible),
                    persistent_locked: l.persistent_locked.map_or(-1, i8::from),
                    persistent_visible: l.persistent_visible.map_or(-1, i8::from),
                    reserved: 0,
                },
            )
        }
    })
}
/// # Safety
/// out is writable ObjectInfo until return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_object(
    handle: u64,
    index: usize,
    out: *mut ObjectInfo,
) -> u32 {
    boundary(|| {
        let s = snapshot(handle)?;
        let o = s
            .state
            .objects
            .get(index)
            .ok_or(LayerError::MissingObject)?;
        unsafe {
            write(
                out,
                ObjectInfo {
                    rgb: o.rgb,
                    locked: u8::from(o.locked),
                    visible: u8::from(o.visible),
                    color_source: o.color_source as u8,
                    reserved: [0; 2],
                },
            )
        }
    })
}
/// Copies UTF-8, without NUL: kind0 document/active (fields0/1), kind1 layer
/// id/parent/name/path (fields0..3), kind2 object id/layer (fields0/1).
/// # Safety
/// output has capacity writable bytes, needed is writable usize; no retained pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_snapshot_text(
    handle: u64,
    kind: u32,
    index: usize,
    field: u32,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let s = snapshot(handle)?;
        let joined;
        let value = match kind {
            0 => match field {
                0 => s.state.document_id.as_str(),
                1 => s.state.active_layer.as_deref().unwrap_or(""),
                _ => return Err(LayerError::UnsupportedVersion),
            },
            1 => {
                let l = s.state.layers.get(index).ok_or(LayerError::MissingLayer)?;
                match field {
                    0 => &l.source_id,
                    1 => l.parent_id.as_deref().unwrap_or(""),
                    2 => &l.name,
                    3 => {
                        joined = l.path_components.join("::");
                        &joined
                    }
                    _ => return Err(LayerError::UnsupportedVersion),
                }
            }
            2 => {
                let o = s
                    .state
                    .objects
                    .get(index)
                    .ok_or(LayerError::MissingObject)?;
                match field {
                    0 => &o.id,
                    1 => &o.layer_id,
                    _ => return Err(LayerError::UnsupportedVersion),
                }
            }
            _ => return Err(LayerError::UnsupportedVersion),
        };
        unsafe { copy_text(value, output, capacity, needed) }
    })
}
/// # Safety
/// Selected IDs are readable UTF-8 views for call, out writable u64. Session scope2
/// requires count0; selected scope1 allows count0 for palette-only transfers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_plan_receive(
    source: u64,
    destination: u64,
    scope: u32,
    ids: *const ByteView,
    count: usize,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let src = snapshot(source)?;
        let dst = snapshot(destination)?;
        let scope = match scope {
            1 => TransferScope::Selected(unsafe { selection(ids, count)? }),
            2 if count == 0 => TransferScope::Session,
            2 => return Err(LayerError::InvalidSnapshot),
            _ => return Err(LayerError::UnsupportedVersion),
        };
        let p = plan_receive(&src.state, &dst.state, scope)?;
        let bytes = p
            .before
            .metadata_bytes()?
            .checked_add(p.after.metadata_bytes()?)
            .and_then(|v| v.checked_mul(4))
            .ok_or(LayerError::LimitExceeded)?;
        let handle = insert(Entry::Plan(Arc::new(p), bytes))?;
        unsafe { write(out, handle) }
    })
}
/// # Safety
/// command is readable bounded JSON for this call; out is writable aligned u64.
/// Plan owns before/after metadata; no input pointer or host object is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_document_plan(
    current: u64,
    command: ByteView,
    out: *mut u64,
) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let state = snapshot(current)?;
        let command = crate::layer_document::decode_command(unsafe {
            array(command.data, command.len, MAX_METADATA_BYTES)?
        })?;
        let p = crate::layer_document::plan_command(&state.state, command)?;
        let bytes = p
            .before
            .metadata_bytes()?
            .checked_add(p.after.metadata_bytes()?)
            .and_then(|v| v.checked_mul(4))
            .ok_or(LayerError::LimitExceeded)?;
        let handle = insert(Entry::Plan(Arc::new(p), bytes))?;
        unsafe { write(out, handle) }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_plan_free(handle: u64) -> u32 {
    boundary(|| free(handle, HandleKind::Plan))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_plan_validate(handle: u64, current: u64) -> u32 {
    boundary(|| plan(handle)?.check_destination(&snapshot(current)?.state))
}
/// # Safety
/// out is writable u64. New snapshot has independent lifetime from the plan.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_plan_after_snapshot(handle: u64, out: *mut u64) -> u32 {
    boundary(|| {
        unsafe { write(out, 0)? };
        let record = SnapshotRecord::new(plan(handle)?.after.clone())?;
        let id = insert(Entry::Snapshot(Arc::new(record)))?;
        unsafe { write(out, id) }
    })
}
/// # Safety
/// source is a readable bounded UTF-8 view, output/needed writable as documented.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_plan_map(
    handle: u64,
    kind: u32,
    source: ByteView,
    output: *mut u8,
    capacity: usize,
    needed: *mut usize,
) -> u32 {
    boundary(|| {
        unsafe { write(needed, 0)? };
        let p = plan(handle)?;
        let source = unsafe { text(source, 1024)? };
        let value = match kind {
            1 => p
                .layer_mapping
                .get(source)
                .ok_or(LayerError::MissingLayer)?,
            2 => p
                .object_mapping
                .get(source)
                .ok_or(LayerError::MissingObject)?,
            _ => return Err(LayerError::UnsupportedVersion),
        };
        unsafe { copy_text(value, output, capacity, needed) }
    })
}
