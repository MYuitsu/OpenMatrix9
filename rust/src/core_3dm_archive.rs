//! Source archive identity and preservation policy. No host or parser dependencies.
use crate::builder_history::{Json, parse_document};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveMode {
    GeometryOnly = 0,
    Preserve = 1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveCapability {
    Editable = 1,
    DisplayRetained = 2,
    Retained = 3,
    Incompatible = 4,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchiveIdentity {
    pub import_namespace: String,
    pub source_uuid: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityError;
fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(b)
            }
        })
        && bytes.iter().any(|b| *b != b'0' && *b != b'-')
}
impl ArchiveIdentity {
    pub fn new(import_namespace: &str, source_uuid: &str) -> Result<Self, IdentityError> {
        if !canonical_uuid(import_namespace) || !canonical_uuid(source_uuid) {
            return Err(IdentityError);
        }
        Ok(Self {
            import_namespace: import_namespace.into(),
            source_uuid: source_uuid.into(),
        })
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_mode_valid(mode: u32) -> bool {
    mode <= 1
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_capability_valid(capability: u32) -> bool {
    (1..=4).contains(&capability)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_archive_legacy_export_allowed(mode: u32) -> bool {
    mode == ArchiveMode::GeometryOnly as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureError {
    Malformed,
    Cycle,
    Limit,
}
pub const MAX_ARCHIVE_NODES: usize = 1_000_000;
pub const MAX_ARCHIVE_EDGES: usize = 16_000_000;
pub const MAX_COPY_BYTES: usize = 512 * 1024 * 1024;
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_copy_budget(bytes: usize, copies: usize, total: usize) -> isize {
    match bytes
        .checked_mul(copies)
        .and_then(|value| total.checked_add(value))
    {
        Some(value) if value <= MAX_COPY_BYTES => value as isize,
        _ => -1,
    }
}
// actions: unchanged=0, replace geometry=1, transform instance=2, duplicate=3,
// native non-instance geometry transform=4.
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_overlay_allowed(
    capability: u32,
    action: u32,
    known_references: bool,
    is_instance: bool,
) -> bool {
    if !known_references || !om9_3dm_archive_capability_valid(capability) || capability == 4 {
        return false;
    }
    match action {
        0 | 3 => true,
        1 => capability == 1,
        2 => is_instance,
        4 => !is_instance,
        _ => false,
    }
}
pub fn dependency_closure(
    node_count: usize,
    offsets: &[usize],
    edges: &[usize],
    selected: &[usize],
) -> Result<Vec<usize>, ClosureError> {
    if node_count > MAX_ARCHIVE_NODES
        || edges.len() > MAX_ARCHIVE_EDGES
        || selected.len() > MAX_ARCHIVE_NODES
    {
        return Err(ClosureError::Limit);
    }
    if offsets.len() != node_count + 1
        || offsets.first() != Some(&0)
        || offsets.last() != Some(&edges.len())
        || offsets.windows(2).any(|p| p[0] > p[1])
        || edges.iter().any(|i| *i >= node_count)
        || selected.iter().any(|i| *i >= node_count)
    {
        return Err(ClosureError::Malformed);
    }
    let mut state = vec![0u8; node_count];
    let mut stack = Vec::new();
    for &root in selected {
        if state[root] == 2 {
            continue;
        }
        state[root] = 1;
        stack.push((root, offsets[root]));
        while let Some((node, next)) = stack.last_mut() {
            if *next == offsets[*node + 1] {
                state[*node] = 2;
                stack.pop();
                continue;
            }
            let child = edges[*next];
            *next += 1;
            match state[child] {
                1 => return Err(ClosureError::Cycle),
                0 => {
                    state[child] = 1;
                    stack.push((child, offsets[child]));
                }
                _ => {}
            }
        }
    }
    Ok(state
        .iter()
        .enumerate()
        .filter_map(|(i, s)| (*s == 2).then_some(i))
        .collect())
}

/// # Safety
/// Nonempty pointer ranges must be aligned and readable for the declared lengths.
/// Output must be writable for capacity entries and must not overlap input ranges.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_3dm_dependency_closure(
    node_count: usize,
    offsets: *const usize,
    edges: *const usize,
    edge_count: usize,
    selected: *const usize,
    selected_count: usize,
    output: *mut usize,
    capacity: usize,
) -> isize {
    if node_count > MAX_ARCHIVE_NODES
        || edge_count > MAX_ARCHIVE_EDGES
        || selected_count > MAX_ARCHIVE_NODES
    {
        return -3;
    }
    if offsets.is_null()
        || (edge_count > 0 && edges.is_null())
        || (selected_count > 0 && selected.is_null())
        || (capacity > 0 && output.is_null())
    {
        return -1;
    }
    let offsets = unsafe { std::slice::from_raw_parts(offsets, node_count + 1) };
    let edges = if edge_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(edges, edge_count) }
    };
    let selected = if selected_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(selected, selected_count) }
    };
    match dependency_closure(node_count, offsets, edges, selected) {
        Err(ClosureError::Malformed) => -1,
        Err(ClosureError::Cycle) => -2,
        Err(ClosureError::Limit) => -3,
        Ok(result) => {
            if capacity == 0 {
                return result.len() as isize;
            }
            if capacity < result.len() {
                return -4;
            }
            if !result.is_empty() {
                unsafe { std::ptr::copy_nonoverlapping(result.as_ptr(), output, result.len()) };
            }
            result.len() as isize
        }
    }
}

const MAX_MANIFEST_BYTES: usize = 32 * 1024 * 1024;
const MAX_MANIFEST_NODES: usize = 1_000_000;

/// Validate the owned geometry staging schema before Qt or kernel conversion can
/// coerce missing/wrongly typed fields to zeros. This is geometry-only export;
/// unrecognized retained metadata is never silently accepted as exportable.
pub fn validate_export_manifest(raw: &str) -> Result<(), &'static str> {
    let Json::Object(root) = parse_document(raw, MAX_MANIFEST_BYTES, MAX_MANIFEST_NODES)? else {
        return Err("export manifest object required");
    };
    if root
        .keys()
        .any(|key| !matches!(key.as_str(), "items" | "tolerance"))
    {
        return Err("unsupported export manifest field");
    }
    if root
        .get("tolerance")
        .is_some_and(|value| !matches!(value, Json::Number(n) if *n > 0.))
    {
        return Err("positive export tolerance required");
    }
    let Some(Json::Array(items)) = root.get("items") else {
        return Err("export items required");
    };
    if items.is_empty() {
        return Err("no geometry to export");
    }
    for item in items {
        let Json::Object(item) = item else {
            return Err("export item object required");
        };
        if item.keys().any(|key| {
            !matches!(
                key.as_str(),
                "name"
                    | "layer"
                    | "visible"
                    | "locked"
                    | "color"
                    | "brep"
                    | "vertices"
                    | "faces"
                    | "point_cloud_fields"
                    | "point_cloud_sha256"
                    | "point_cloud_transform"
            )
        }) {
            return Err("unsupported geometry-only item field");
        }
        for key in ["name", "layer"] {
            if let Some(value) = item.get(key) {
                if !matches!(value, Json::String(s) if s.len() <= 32768 && !s.contains('\0')) {
                    return Err("invalid export name or layer");
                }
            }
        }
        for key in ["visible", "locked"] {
            if item
                .get(key)
                .is_some_and(|value| !matches!(value, Json::Boolean))
            {
                return Err("export visibility and lock must be boolean");
            }
        }
        // Missing color/tolerance retain the existing native export defaults.
        // Explicit fields must still be fully typed before Qt conversion.
        if let Some(color) = item.get("color") {
            if !matches!(color, Json::Array(values) if values.len()==3 && values.iter().all(|value| matches!(value, Json::Number(n) if n.fract()==0. && (0. ..=255.).contains(n))))
            {
                return Err("export color must contain three byte values");
            }
        }
        if [
            "point_cloud_fields",
            "point_cloud_sha256",
            "point_cloud_transform",
        ]
        .iter()
        .any(|key| item.contains_key(*key))
        {
            if ["brep", "vertices", "faces"]
                .iter()
                .any(|key| item.contains_key(*key))
            {
                return Err("exclusive staged PointCloud fields required");
            }
            if !matches!(item.get("point_cloud_fields"), Some(Json::String(path)) if !path.is_empty() && path.len() <= 32768 && !path.contains('\0'))
            {
                return Err("bounded PointCloud fields path required");
            }
            if !matches!(item.get("point_cloud_sha256"), Some(Json::String(hash)) if hash.len()==64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
            {
                return Err("PointCloud SHA256 required");
            }
            if item.get("point_cloud_transform").is_some_and(|value| !matches!(value, Json::Array(matrix) if matrix.len()==16 && matrix.iter().all(|value| matches!(value, Json::Number(_))))) {
                return Err("PointCloud transform requires sixteen finite numbers");
            }
            continue;
        }
        if let Some(brep) = item.get("brep") {
            if item.contains_key("vertices")
                || item.contains_key("faces")
                || !matches!(brep, Json::String(path) if !path.is_empty() && path.len() <= 32768 && !path.contains('\0'))
            {
                return Err("exclusive staged BRep path required");
            }
            continue;
        }
        let (Some(Json::Array(vertices)), Some(Json::Array(faces))) =
            (item.get("vertices"), item.get("faces"))
        else {
            return Err("mesh vertices and faces required");
        };
        if vertices.len() < 3 || faces.is_empty() {
            return Err("empty mesh geometry");
        }
        for vertex in vertices {
            if !matches!(vertex, Json::Array(values) if values.len()==3 && values.iter().all(|value| matches!(value, Json::Number(_))))
            {
                return Err("mesh vertex must contain three finite numbers");
            }
        }
        for face in faces {
            if !matches!(face, Json::Array(values) if values.len()==4 && values.iter().all(|value| matches!(value, Json::Number(n) if *n >= 0. && n.fract()==0. && *n < vertices.len() as f64)))
            {
                return Err("mesh face must contain four in-range integer indices");
            }
        }
    }
    Ok(())
}

/// Validate bounded UTF-8 bytes without retaining native memory.
///
/// # Safety
/// `raw` must be readable for `length` bytes for the duration of this call.
/// Oversized lengths are rejected before dereferencing the pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_3dm_export_manifest_valid(raw: *const u8, length: usize) -> bool {
    if raw.is_null() || length == 0 || length > MAX_MANIFEST_BYTES {
        return false;
    }
    let Ok(raw) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(raw, length) }) else {
        return false;
    };
    validate_export_manifest(raw).is_ok()
}
