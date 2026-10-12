//! Bounded plugin-owned source bytes. Base64/digest validation is portable Rust;
//! native openNURBS must still prove archive inventory before metadata is trusted.
use crate::layer_state::LayerError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
pub const CHUNK_BYTES: usize = 64 * 1024;
pub const MAX_PAYLOAD_BYTES: usize = 512 * 1024 * 1024;
pub const MAX_CHUNKS: usize = MAX_PAYLOAD_BYTES / CHUNK_BYTES;
pub const MAX_ENCODED_CHUNK: usize = 4 * CHUNK_BYTES.div_ceil(3);
pub const MAX_MANIFEST_BYTES: usize = 32 * 1024 * 1024;
/// Native inventory must come from openNURBS reading the verified payload, not
/// another stored marker. Matching JSON alone does not establish that witness.
pub fn verify_manifest(stored: &[u8], native: &[u8], digest: &str) -> Result<(), LayerError> {
    let expected = parse_manifest(native, digest)?;
    let actual = parse_manifest(stored, digest)?;
    if actual != expected {
        return Err(LayerError::InvalidSnapshot);
    }
    Ok(())
}
fn parse_manifest(bytes: &[u8], digest: &str) -> Result<Value, LayerError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let expected: Value = serde_json::from_slice(bytes).map_err(|_| LayerError::InvalidSnapshot)?;
    if !expected.is_object()
        || expected["schema_version"] != 1
        || expected["archive_sha256"].as_str() != Some(digest)
    {
        return Err(LayerError::InvalidSnapshot);
    }
    let scale = expected["scale_mm"]
        .as_f64()
        .ok_or(LayerError::InvalidSnapshot)?;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(LayerError::InvalidSnapshot);
    }
    for key in ["records", "components", "issues"] {
        let rows = expected[key]
            .as_array()
            .ok_or(LayerError::InvalidSnapshot)?;
        if rows.len() > crate::core_3dm_archive::MAX_ARCHIVE_NODES {
            return Err(LayerError::LimitExceeded);
        }
        if key == "issues" && !rows.is_empty() {
            return Err(LayerError::InvalidSnapshot);
        }
    }
    Ok(expected)
}

fn canonical_id(text: &str) -> bool {
    text.len() == 36
        && text.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
        && text != "00000000-0000-0000-0000-000000000000"
}
fn legacy_style(row: &Value) -> bool {
    row["class_name"] == "ON_DimStyle"
        && row["component_type"] == "AnnotationStyle"
        && row["role"] == "top-level"
        && row["capability"] == "retained"
        && row["dependencies"].as_array().is_some_and(Vec::is_empty)
}
fn style_key(row: &Value) -> Result<Vec<u8>, LayerError> {
    let mut value = row.clone();
    value
        .as_object_mut()
        .ok_or(LayerError::InvalidSnapshot)?
        .remove("source_uuid");
    serde_json::to_vec(&value).map_err(|_| LayerError::InvalidSnapshot)
}
fn uuid_occurrences(root: &Value, ids: &HashSet<String>) -> HashMap<String, usize> {
    fn text(text: &str, ids: &HashSet<String>, counts: &mut HashMap<String, usize>) {
        // Include embedded identities in user text/keys, not only exact JSON
        // values. Four fixed hyphens reject almost all windows without allocation.
        for bytes in text.as_bytes().windows(36) {
            if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
                continue;
            }
            if let Ok(candidate) = std::str::from_utf8(bytes) {
                let id = candidate.to_ascii_lowercase();
                if ids.contains(&id) {
                    *counts.entry(id).or_default() += 1;
                }
            }
        }
    }
    fn visit(root: &Value, ids: &HashSet<String>, counts: &mut HashMap<String, usize>) {
        match root {
            Value::String(value) => text(value, ids, counts),
            Value::Array(values) => {
                for value in values {
                    visit(value, ids, counts);
                }
            }
            Value::Object(values) => {
                for (key, value) in values {
                    text(key, ids, counts);
                    visit(value, ids, counts);
                }
            }
            _ => {}
        }
    }
    let mut counts = HashMap::new();
    visit(root, ids, &mut counts);
    counts
}
/// Legacy V5 readers may generate missing/nil DimStyle UUIDs on each read
/// (openNURBS opennurbs_internal_V5_dimstyle.cpp). Only two independent native
/// inventories of the SAME verified bytes can witness that instability.
/// Every other fact stays exact. Stable, referenced and ambiguous IDs reject.
/// Native callers must never supply stored metadata as the second witness.
pub fn verify_manifest_with_witness(
    stored: &[u8],
    native: &[u8],
    witness: &[u8],
    digest: &str,
) -> Result<(), LayerError> {
    let actual = parse_manifest(stored, digest)?;
    let mut first = parse_manifest(native, digest)?;
    let mut second = parse_manifest(witness, digest)?;
    if actual == first && first == second {
        return Ok(());
    }
    let version = first["source_version"]
        .as_u64()
        .ok_or(LayerError::InvalidSnapshot)?;
    if !(1..=50).contains(&version)
        || second["source_version"] != first["source_version"]
        || actual["source_version"] != first["source_version"]
    {
        return Err(LayerError::InvalidSnapshot);
    }
    let a = actual["components"]
        .as_array()
        .ok_or(LayerError::InvalidSnapshot)?;
    let b = first["components"]
        .as_array()
        .ok_or(LayerError::InvalidSnapshot)?;
    let c = second["components"]
        .as_array()
        .ok_or(LayerError::InvalidSnapshot)?;
    if a.len() != b.len() || a.len() != c.len() {
        return Err(LayerError::InvalidSnapshot);
    }
    let mut keys = HashMap::new();
    for row in b.iter().filter(|row| legacy_style(row)) {
        *keys.entry(style_key(row)?).or_insert(0usize) += 1;
    }
    let mut changes = Vec::new();
    let mut ids = HashSet::new();
    for (i, ((a, b), c)) in a.iter().zip(b).zip(c).enumerate() {
        if a == b && b == c {
            continue;
        }
        if !legacy_style(a) || !legacy_style(b) || !legacy_style(c) {
            return Err(LayerError::InvalidSnapshot);
        }
        let key = style_key(b)?;
        if style_key(a)? != key || style_key(c)? != key || keys.get(&key) != Some(&1) {
            return Err(LayerError::InvalidSnapshot);
        }
        let ai = a["source_uuid"]
            .as_str()
            .ok_or(LayerError::InvalidSnapshot)?;
        let bi = b["source_uuid"]
            .as_str()
            .ok_or(LayerError::InvalidSnapshot)?;
        let ci = c["source_uuid"]
            .as_str()
            .ok_or(LayerError::InvalidSnapshot)?;
        if !canonical_id(ai) || !canonical_id(bi) || !canonical_id(ci) || bi == ci {
            return Err(LayerError::InvalidSnapshot);
        }
        ids.extend([ai.to_owned(), bi.to_owned(), ci.to_owned()]);
        changes.push((i, ai.to_owned(), bi.to_owned(), ci.to_owned()));
    }
    let counts_a = uuid_occurrences(&actual, &ids);
    let counts_b = uuid_occurrences(&first, &ids);
    let counts_c = uuid_occurrences(&second, &ids);
    for (i, a, b, c) in changes {
        if counts_a.get(&a) != Some(&1)
            || counts_b.get(&b) != Some(&1)
            || counts_c.get(&c) != Some(&1)
        {
            return Err(LayerError::InvalidSnapshot);
        }
        first["components"][i]["source_uuid"] = Value::String(a.clone());
        second["components"][i]["source_uuid"] = Value::String(a);
    }
    if actual != first || actual != second {
        return Err(LayerError::InvalidSnapshot);
    }
    Ok(())
}
fn value(byte: u8) -> Result<u8, LayerError> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(LayerError::InvalidSnapshot),
    }
}
fn length(text: &str) -> Result<usize, LayerError> {
    if text.len() > MAX_ENCODED_CHUNK {
        return Err(LayerError::LimitExceeded);
    }
    if !text.len().is_multiple_of(4) {
        return Err(LayerError::InvalidSnapshot);
    }
    let bytes = text.as_bytes();
    let padding = if bytes.ends_with(b"==") {
        2
    } else if bytes.ends_with(b"=") {
        1
    } else {
        0
    };
    let size = (bytes.len() / 4)
        .checked_mul(3)
        .and_then(|n| n.checked_sub(padding))
        .ok_or(LayerError::InvalidSnapshot)?;
    if size > CHUNK_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    Ok(size)
}
/// Validate all lengths and total budget before allocation, then decode and hash
/// once. No permissive whitespace/alphabet or noncanonical pad bits are accepted.
pub fn decode_chunks(chunks: &[&str], digest: &str, limit: usize) -> Result<Vec<u8>, LayerError> {
    if chunks.len() > MAX_CHUNKS || limit > MAX_PAYLOAD_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    if chunks.is_empty()
        || digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(LayerError::InvalidSnapshot);
    }
    let mut total = 0usize;
    for (index, chunk) in chunks.iter().enumerate() {
        let size = length(chunk)?;
        if (index + 1 < chunks.len() && size != CHUNK_BYTES) || (chunks.len() > 1 && size == 0) {
            return Err(LayerError::InvalidSnapshot);
        }
        total = total.checked_add(size).ok_or(LayerError::LimitExceeded)?;
        if total > limit {
            return Err(LayerError::LimitExceeded);
        }
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(total)
        .map_err(|_| LayerError::LimitExceeded)?;
    let mut hash = Sha256::new();
    for text in chunks {
        let start = output.len();
        let bytes = text.as_bytes();
        for (index, group) in bytes.as_chunks::<4>().0.iter().enumerate() {
            let a = value(group[0])?;
            let b = value(group[1])?;
            let last = index + 1 == bytes.len() / 4;
            if group[2] == b'=' {
                if !last || group[3] != b'=' || b & 15 != 0 {
                    return Err(LayerError::InvalidSnapshot);
                }
                output.push((a << 2) | (b >> 4));
            } else {
                let c = value(group[2])?;
                output.push((a << 2) | (b >> 4));
                output.push((b << 4) | (c >> 2));
                if group[3] == b'=' {
                    if !last || c & 3 != 0 {
                        return Err(LayerError::InvalidSnapshot);
                    }
                } else {
                    output.push((c << 6) | value(group[3])?);
                }
            }
        }
        hash.update(&output[start..]);
    }
    if output.len() != total || format!("{:x}", hash.finalize()) != digest {
        return Err(LayerError::InvalidSnapshot);
    }
    Ok(output)
}
