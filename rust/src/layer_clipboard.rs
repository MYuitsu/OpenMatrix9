//! Geometry stays borrowed for one call. Rust owns metadata, identity and bounds.
use crate::{layer_codec::encode_metadata, layer_state::*};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub const RHINO5_FORMAT: &str = "Rhino 5.0 3DM Clip global mem";
pub const LAYER_FORMAT: &str = "OM9.LayerSession.v1";
pub const MAX_GEOMETRY_BYTES: usize = 512 * 1024 * 1024;
/// Never overwrite a later clipboard owner, including an unknown sequence0.
pub fn rollback_owned_publication(before: u32, published: u32, current: u32) -> bool {
    published != 0 && published != before && current == published
}
/// Materialized prior formats must fit this budget before replacing the owner.
pub fn check_clipboard_backup(bytes: usize, formats: usize) -> Result<(), LayerError> {
    if bytes > MAX_GEOMETRY_BYTES || formats > 256 {
        return Err(LayerError::LimitExceeded);
    }
    Ok(())
}
/// Facts extracted from the exact native archive whose digest was verified.
/// Native IDs remain distinct from transfer tags; native-only imports ignore tags.
pub struct GeometryObjectBinding<'a> {
    pub physical_id: &'a str,
    pub source_id: &'a str,
}
/// Reject missing/duplicate/unknown tags before constructing a detached overlay.
/// This receives identities only, never geometry, render meshes or point arrays.
pub fn bind_geometry_objects(
    source: &LayerSnapshotV1,
    bindings: &[GeometryObjectBinding<'_>],
) -> Result<LayerSnapshotV1, LayerError> {
    use std::collections::{HashMap, HashSet};
    if bindings.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    validate_layers(source)?;
    if bindings.len() != source.objects.len() {
        return Err(LayerError::ClipboardBinding);
    }
    let mut charge = source.metadata_bytes()?;
    for binding in bindings {
        for value in [binding.physical_id, binding.source_id] {
            if value.len() > 1024 {
                return Err(LayerError::LimitExceeded);
            }
            if !valid_identity(value) {
                return Err(LayerError::ClipboardBinding);
            }
            charge = charge
                .checked_add(value.len())
                .ok_or(LayerError::LimitExceeded)?;
        }
        charge = charge
            .checked_add(std::mem::size_of::<GeometryObjectBinding<'_>>())
            .ok_or(LayerError::LimitExceeded)?;
        if charge > MAX_METADATA_BYTES {
            return Err(LayerError::LimitExceeded);
        }
    }
    let mut known = HashSet::new();
    let mut physical = HashSet::new();
    let mut by_source = HashMap::new();
    for table in [&mut known, &mut physical] {
        table
            .try_reserve(bindings.len())
            .map_err(|_| LayerError::LimitExceeded)?;
    }
    by_source
        .try_reserve(bindings.len())
        .map_err(|_| LayerError::LimitExceeded)?;
    known.extend(source.objects.iter().map(|row| row.id.as_str()));
    for binding in bindings {
        if !known.contains(binding.source_id)
            || !physical.insert(binding.physical_id)
            || by_source
                .insert(binding.source_id, binding.physical_id)
                .is_some()
        {
            return Err(LayerError::ClipboardBinding);
        }
    }
    let mut result = source.clone();
    for row in &mut result.objects {
        row.id = by_source
            .get(row.id.as_str())
            .ok_or(LayerError::ClipboardBinding)?
            .to_string();
    }
    validate_layers(&result)?;
    Ok(result)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerClipboardScope {
    Selected,
    Session,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteEvidence {
    NativeOnly,
    Extended,
}
#[derive(Debug, PartialEq, Eq)]
pub struct LayerClipboardReceived {
    pub snapshot: Option<LayerSnapshotV1>,
    pub scope: Option<LayerClipboardScope>,
    pub evidence: PaletteEvidence,
    pub geometry_version: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    layer_schema: u32,
    geometry_format: String,
    #[serde(with = "crate::layer_codec::generation")]
    geometry_length: u64,
    geometry_sha256: [u8; 32],
    geometry_version: u32,
    scope: LayerClipboardScope,
    snapshot: LayerSnapshotV1,
}
pub fn check_geometry_size(size: usize) -> Result<(), LayerError> {
    if size > MAX_GEOMETRY_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    if size < 33 {
        return Err(LayerError::ClipboardGeometry);
    }
    Ok(())
}
fn geometry_version(geometry: &[u8]) -> Result<u32, LayerError> {
    check_geometry_size(geometry.len())?;
    if &geometry[..24] != b"3D Geometry File Format " {
        return Err(LayerError::ClipboardGeometry);
    }
    let mut version = &geometry[24..32];
    while version.first() == Some(&b' ') {
        version = &version[1..];
    }
    while version.last() == Some(&b' ') {
        version = &version[..version.len() - 1];
    }
    match version {
        b"5" => Ok(5),
        b"50" => Ok(50),
        _ => Err(LayerError::ClipboardGeometry),
    }
}
pub fn prepare_clipboard(
    snapshot: &LayerSnapshotV1,
    scope: LayerClipboardScope,
    geometry: &[u8],
) -> Result<Vec<u8>, LayerError> {
    let version = geometry_version(geometry)?;
    validate_layers(snapshot)?;
    #[derive(Serialize)]
    struct Borrowed<'a> {
        version: u32,
        layer_schema: u32,
        geometry_format: &'a str,
        #[serde(with = "crate::layer_codec::generation")]
        geometry_length: u64,
        geometry_sha256: [u8; 32],
        geometry_version: u32,
        scope: LayerClipboardScope,
        snapshot: &'a LayerSnapshotV1,
    }
    let digest: [u8; 32] = Sha256::digest(geometry).into();
    encode_metadata(&Borrowed {
        version: 1,
        layer_schema: 1,
        geometry_format: RHINO5_FORMAT,
        geometry_length: geometry.len() as u64,
        geometry_sha256: digest,
        geometry_version: version,
        scope,
        snapshot,
    })
}
pub fn receive_clipboard(
    geometry: &[u8],
    metadata: Option<&[u8]>,
) -> Result<LayerClipboardReceived, LayerError> {
    let version = geometry_version(geometry)?;
    let Some(metadata) = metadata else {
        return Ok(LayerClipboardReceived {
            snapshot: None,
            scope: None,
            evidence: PaletteEvidence::NativeOnly,
            geometry_version: version,
        });
    };
    if metadata.len() > MAX_METADATA_BYTES {
        return Err(LayerError::LimitExceeded);
    }
    let envelope: Envelope =
        serde_json::from_slice(metadata).map_err(|_| LayerError::InvalidSnapshot)?;
    if envelope.version != 1 || envelope.layer_schema != 1 {
        return Err(LayerError::UnsupportedVersion);
    }
    if envelope.geometry_format != RHINO5_FORMAT
        || envelope.geometry_length != geometry.len() as u64
        || envelope.geometry_version != version
    {
        return Err(LayerError::ClipboardBinding);
    }
    let digest: [u8; 32] = Sha256::digest(geometry).into();
    if envelope.geometry_sha256 != digest {
        return Err(LayerError::ClipboardBinding);
    }
    validate_layers(&envelope.snapshot)?;
    Ok(LayerClipboardReceived {
        snapshot: Some(envelope.snapshot),
        scope: Some(envelope.scope),
        evidence: PaletteEvidence::Extended,
        geometry_version: version,
    })
}
