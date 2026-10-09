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

const MAX_MANIFEST_BYTES: usize = 32 * 1024 * 1024;
const MAX_MANIFEST_NODES: usize = 1_000_000;

/// Validate the owned geometry staging schema before Qt or kernel conversion can
/// coerce missing/wrongly typed fields to zeros. This is geometry-only export;
/// unrecognized retained metadata is never silently accepted as exportable.
pub fn validate_export_manifest(raw: &str) -> Result<(), &'static str> {
    let Json::Object(root) = parse_document(raw, MAX_MANIFEST_BYTES, MAX_MANIFEST_NODES)? else {
        return Err("export manifest object required");
    };
    if root.keys().any(|key| !matches!(key.as_str(), "items" | "tolerance")) {
        return Err("unsupported export manifest field");
    }
    if !matches!(root.get("tolerance"), Some(Json::Number(n)) if *n > 0.) {
        return Err("positive export tolerance required");
    }
    let Some(Json::Array(items)) = root.get("items") else {return Err("export items required");};
    if items.is_empty() {return Err("no geometry to export");}
    for item in items {
        let Json::Object(item) = item else {return Err("export item object required");};
        if item.keys().any(|key| !matches!(key.as_str(), "name" | "layer" | "visible" | "locked" | "color" | "brep" | "vertices" | "faces")) {
            return Err("unsupported geometry-only item field");
        }
        for key in ["name", "layer"] {
            if let Some(value) = item.get(key) {
                if !matches!(value, Json::String(s) if s.len() <= 32768 && !s.contains('\0')) {return Err("invalid export name or layer");}
            }
        }
        for key in ["visible", "locked"] {
            if item.get(key).is_some_and(|value| !matches!(value, Json::Boolean)) {return Err("export visibility and lock must be boolean");}
        }
        let Some(Json::Array(color)) = item.get("color") else {return Err("export color required");};
        if color.len()!=3 || color.iter().any(|value| !matches!(value, Json::Number(n) if n.fract()==0. && (0. ..=255.).contains(n))) {
            return Err("export color must contain three byte values");
        }
        if let Some(brep) = item.get("brep") {
            if item.contains_key("vertices") || item.contains_key("faces") || !matches!(brep, Json::String(path) if !path.is_empty() && path.len() <= 32768 && !path.contains('\0')) {
                return Err("exclusive staged BRep path required");
            }
            continue;
        }
        let (Some(Json::Array(vertices)), Some(Json::Array(faces))) = (item.get("vertices"),item.get("faces")) else {return Err("mesh vertices and faces required");};
        if vertices.len()<3 || faces.is_empty() {return Err("empty mesh geometry");}
        for vertex in vertices {
            if !matches!(vertex, Json::Array(values) if values.len()==3 && values.iter().all(|value| matches!(value, Json::Number(_)))) {
                return Err("mesh vertex must contain three finite numbers");
            }
        }
        for face in faces {
            if !matches!(face, Json::Array(values) if values.len()==4 && values.iter().all(|value| matches!(value, Json::Number(n) if *n >= 0. && n.fract()==0. && *n < vertices.len() as f64))) {
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
    if raw.is_null() || length == 0 || length > MAX_MANIFEST_BYTES {return false;}
    let Ok(raw) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(raw,length) }) else {return false;};
    validate_export_manifest(raw).is_ok()
}
