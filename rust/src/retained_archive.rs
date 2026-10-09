// SPDX-License-Identifier: LGPL-2.1-or-later
//! Portable retained-archive policy. Native adapters own file hashing and document resolution.
use crate::builder_history::{Json, parse_document};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::{CStr, c_char};

const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_NODES: usize = 1_000_000;

/// Stable boundary categories; no output value is produced on error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ManifestError {
    Json = -1,
    Integrity = -2,
    Identity = -3,
    Scale = -4,
    Dependencies = -5,
}

fn string(value: Option<&Json>) -> Option<&str> {
    match value {
        Some(Json::String(s)) => Some(s),
        _ => None,
    }
}

fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Canonical UUID syntax with optional braces and case-insensitive hexadecimal.
/// Normalized bytes, rather than spelling, determine identity and duplication.
fn uuid(value: &str) -> Option<[u8; 16]> {
    let value = if value.starts_with('{') {
        value.strip_prefix('{')?.strip_suffix('}')?
    } else {
        value
    };
    if value.len() != 36 {
        return None;
    }
    let mut bytes = [0; 16];
    let mut nibble = 0;
    for (index, byte) in value.bytes().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return None;
            }
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        bytes[nibble / 2] = bytes[nibble / 2] * 16 + digit;
        nibble += 1;
    }
    if bytes.iter().all(|b| *b == 0) {
        None
    } else {
        Some(bytes)
    }
}

/// Pure record identity policy, usable before native document resolution.
pub fn valid_record_identity(source_uuid: &str, namespace: &str) -> bool {
    uuid(source_uuid).is_some()
        && canonical_uuid(namespace).as_deref() == Some(namespace)
}

/// Canonical lowercase UUID spelling used by native archive decoders.
pub fn canonical_uuid(value: &str) -> Option<String> {
    let bytes = uuid(value)?;
    let mut result = String::with_capacity(36);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (index, byte) in bytes.into_iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            result.push('-');
        }
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    Some(result)
}

/// Normalize a borrowed UUID into a caller-owned 37-byte C string.
///
/// # Safety
/// Input must be readable and NUL-terminated throughout this call; the caller
/// must reject embedded NULs. Output must be writable for `capacity` bytes and
/// not alias input. No pointers are retained; output is unchanged on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_retained_uuid_normalize(
    value: *const c_char,
    output: *mut c_char,
    capacity: usize,
) -> bool {
    if value.is_null() || output.is_null() || capacity < 37 {
        return false;
    }
    let Ok(value) = (unsafe { CStr::from_ptr(value) }).to_str() else {
        return false;
    };
    let Some(normalized) = canonical_uuid(value) else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(normalized.as_ptr(), output.cast(), 36);
        output.add(36).write(0);
    }
    true
}

/// Check record identity before native archive/document lookup.
///
/// # Safety
/// Both pointers must address readable NUL-terminated C strings for this call.
/// The caller must reject embedded NULs before creating those strings, because
/// this ABI cannot distinguish them from terminators. No pointers are retained;
/// native pointer provenance remains the caller's obligation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_retained_record_identity_valid(
    source_uuid: *const c_char,
    namespace: *const c_char,
) -> bool {
    if source_uuid.is_null() || namespace.is_null() {
        return false;
    }
    let (Ok(source_uuid), Ok(namespace)) = (
        (unsafe { CStr::from_ptr(source_uuid) }).to_str(),
        (unsafe { CStr::from_ptr(namespace) }).to_str(),
    ) else {
        return false;
    };
    valid_record_identity(source_uuid, namespace)
}

fn validate_dependencies(manifest: &BTreeMap<String, Json>) -> Result<(), ManifestError> {
    if let Some(issues) = manifest.get("issues") {
        if !matches!(issues, Json::Array(rows) if rows.is_empty()) {
            return Err(ManifestError::Dependencies);
        }
    }
    let mut graph = BTreeMap::new();
    for table in ["records", "components"] {
        let rows = match manifest.get(table) {
            Some(Json::Array(rows)) => rows,
            None if table == "components" => continue, // schema-1 legacy inventory
            _ => return Err(ManifestError::Identity),
        };
        for row in rows {
            let Json::Object(row) = row else { return Err(ManifestError::Identity); };
            let id = string(row.get("source_uuid")).and_then(uuid).ok_or(ManifestError::Identity)?;
            let mut dependencies = BTreeSet::new();
            if let Some(value) = row.get("dependencies") {
                let Json::Array(values) = value else { return Err(ManifestError::Dependencies); };
                for value in values {
                    let dependency = string(Some(value)).and_then(uuid).ok_or(ManifestError::Dependencies)?;
                    // Repeated semantic dependencies are harmless (e.g. same material).
                    dependencies.insert(dependency);
                }
            }
            if graph.insert(id, dependencies).is_some() { return Err(ManifestError::Identity); }
        }
    }
    let mut counts = BTreeMap::new();
    let mut dependents: BTreeMap<[u8; 16], Vec<[u8; 16]>> = BTreeMap::new();
    for (id, dependencies) in &graph {
        counts.insert(*id, dependencies.len());
        for dependency in dependencies {
            if !graph.contains_key(dependency) { return Err(ManifestError::Dependencies); }
            dependents.entry(*dependency).or_default().push(*id);
        }
    }
    let mut ready: VecDeque<_> = counts.iter().filter(|(_, count)| **count == 0).map(|(id, _)| *id).collect();
    let mut resolved = 0;
    while let Some(id) = ready.pop_front() {
        resolved += 1;
        if let Some(children) = dependents.get(&id) {
            for child in children {
                let count = counts.get_mut(child).expect("known manifest row");
                *count -= 1;
                if *count == 0 { ready.push_back(*child); }
            }
        }
    }
    if resolved != graph.len() { return Err(ManifestError::Dependencies); }
    Ok(())
}

/// Verify an archive inventory before native staging or document mutation.
///
/// # Safety
/// `raw` must remain readable for `length` bytes during this call. Oversized
/// lengths reject before borrowing. No memory or native object is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_retained_dependencies_valid(raw: *const u8, length: usize) -> bool {
    if raw.is_null() || length == 0 || length > MAX_BYTES { return false; }
    let Ok(raw) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(raw, length) }) else { return false; };
    let Ok(Json::Object(manifest)) = parse_document(raw, MAX_BYTES, MAX_NODES) else { return false; };
    validate_dependencies(&manifest).is_ok()
}

/// Validate the manifest against a digest computed from the native archive bytes.
/// This does not read a file, authenticate its origin, or resolve native document objects.
pub fn validate_manifest(
    raw: &str,
    expected_digest: &str,
    source_uuid: &str,
    source_class: &str,
    capability: &str,
    namespace: &str,
    schema: i64,
) -> Result<f64, ManifestError> {
    let Json::Object(manifest) =
        parse_document(raw, MAX_BYTES, MAX_NODES).map_err(|_| ManifestError::Json)?
    else {
        return Err(ManifestError::Json);
    };
    if schema != 1
        || !matches!(manifest.get("schema_version"), Some(Json::Number(1.)))
        || !digest_valid(expected_digest)
        || string(manifest.get("archive_sha256")) != Some(expected_digest)
    {
        return Err(ManifestError::Integrity);
    }
    validate_dependencies(&manifest)?;
    if !valid_record_identity(source_uuid, namespace)
        || [source_class, capability].iter().any(|s| s.contains('\0'))
    {
        return Err(ManifestError::Identity);
    }
    let selected = uuid(source_uuid).ok_or(ManifestError::Identity)?;
    let Some(Json::Array(records)) = manifest.get("records") else {
        return Err(ManifestError::Identity);
    };
    let mut identities = BTreeSet::new();
    let mut found = false;
    for record in records {
        let Json::Object(row) = record else {
            return Err(ManifestError::Identity);
        };
        let id = string(row.get("source_uuid"))
            .and_then(uuid)
            .ok_or(ManifestError::Identity)?;
        if !identities.insert(id) {
            return Err(ManifestError::Identity);
        }
        if id == selected {
            if string(row.get("class_name")) != Some(source_class)
                || string(row.get("capability")) != Some(capability)
            {
                return Err(ManifestError::Identity);
            }
            found = true;
        }
    }
    if !found {
        return Err(ManifestError::Identity);
    }
    match manifest.get("scale_mm") {
        Some(Json::Number(value)) if value.is_finite() && *value > 0. => Ok(*value),
        _ => Err(ManifestError::Scale),
    }
}

/// Validate borrowed native UTF-8 fields; return 0 or a stable ManifestError code.
///
/// # Safety
/// Each input must address a readable, NUL-terminated C string for the duration of
/// this call. Native callers must reject embedded NULs before constructing these
/// strings: this ABI cannot distinguish an embedded NUL from the final terminator.
/// `out_scale` must be nonnull, aligned, and writable for one f64, and must not alias
/// any input bytes. No pointer is retained. Caller provenance and native dependency
/// safety remain native obligations; Rust cannot certify them. Output is unchanged
/// on every reported error. The release crate uses panic=abort.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_retained_manifest_validate(
    raw: *const c_char,
    expected_digest: *const c_char,
    source_uuid: *const c_char,
    source_class: *const c_char,
    capability: *const c_char,
    namespace: *const c_char,
    schema: i64,
    out_scale: *mut f64,
) -> i32 {
    if out_scale.is_null() {
        return ManifestError::Json as i32;
    }
    let inputs = [
        raw,
        expected_digest,
        source_uuid,
        source_class,
        capability,
        namespace,
    ];
    let mut strings = [""; 6];
    for (i, pointer) in inputs.into_iter().enumerate() {
        if pointer.is_null() {
            return ManifestError::Json as i32;
        }
        let Ok(value) = (unsafe { CStr::from_ptr(pointer) }).to_str() else {
            return ManifestError::Json as i32;
        };
        strings[i] = value;
    }
    match validate_manifest(
        strings[0], strings[1], strings[2], strings[3], strings[4], strings[5], schema,
    ) {
        Ok(value) => {
            unsafe {
                out_scale.write(value);
            }
            0
        }
        Err(error) => error as i32,
    }
}
