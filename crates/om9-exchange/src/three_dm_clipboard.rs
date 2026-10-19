// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 clipboard binding (`OM9-FILE-012`), ported from the Phase2 Rust owner
//! `rust/src/layer_clipboard.rs`.
//!
//! Geometry stays borrowed for one call; Rust owns the layer metadata, identity binding and size
//! bounds. The native 3DM archive is opaque here: its header/version and SHA-256 are checked, but
//! the native SDK remains the geometry validator.

use crate::three_dm::{
    LayerSnapshotError, ThreeDmLayerSnapshot, generation, metadata_bytes, valid_identity,
    validate_layer_snapshot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

pub const RHINO5_FORMAT: &str = "Rhino 5.0 3DM Clip global mem";
pub const LAYER_FORMAT: &str = "OM9.LayerSession.v1";
pub const MAX_GEOMETRY_BYTES: usize = 512 * 1024 * 1024;
pub const MAX_METADATA_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    InvalidSnapshot,
    MissingLayer,
    LimitExceeded,
    ClipboardGeometry,
    ClipboardBinding,
    UnsupportedVersion,
}

fn layer_error(error: LayerSnapshotError) -> ClipboardError {
    match error {
        LayerSnapshotError::Limit => ClipboardError::LimitExceeded,
        LayerSnapshotError::MissingLayer => ClipboardError::MissingLayer,
        LayerSnapshotError::Invalid => ClipboardError::InvalidSnapshot,
        LayerSnapshotError::Duplicate
        | LayerSnapshotError::AmbiguousPath
        | LayerSnapshotError::Cycle => ClipboardError::ClipboardBinding,
    }
}

/// Never overwrite a later clipboard owner, including an unknown sequence 0.
pub fn rollback_owned_publication(before: u32, published: u32, current: u32) -> bool {
    published != 0 && published != before && current == published
}

/// Materialized prior formats must fit this budget before replacing the owner.
pub fn check_clipboard_backup(bytes: usize, formats: usize) -> Result<(), ClipboardError> {
    if bytes > MAX_GEOMETRY_BYTES || formats > 256 {
        return Err(ClipboardError::LimitExceeded);
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
    source: &ThreeDmLayerSnapshot,
    bindings: &[GeometryObjectBinding<'_>],
) -> Result<ThreeDmLayerSnapshot, ClipboardError> {
    const MAX_OBJECTS: usize = 2_000_000;
    if bindings.len() > MAX_OBJECTS {
        return Err(ClipboardError::LimitExceeded);
    }
    validate_layer_snapshot(source).map_err(layer_error)?;
    if bindings.len() != source.objects.len() {
        return Err(ClipboardError::ClipboardBinding);
    }
    let mut charge = metadata_bytes(source).map_err(layer_error)?;
    for binding in bindings {
        for value in [binding.physical_id, binding.source_id] {
            if value.len() > 1024 {
                return Err(ClipboardError::LimitExceeded);
            }
            if !valid_identity(value) {
                return Err(ClipboardError::ClipboardBinding);
            }
            charge = charge
                .checked_add(value.len())
                .ok_or(ClipboardError::LimitExceeded)?;
        }
        charge = charge
            .checked_add(std::mem::size_of::<GeometryObjectBinding<'_>>())
            .ok_or(ClipboardError::LimitExceeded)?;
        if charge > MAX_METADATA_BYTES {
            return Err(ClipboardError::LimitExceeded);
        }
    }
    let mut known = HashSet::new();
    let mut physical = HashSet::new();
    let mut by_source = HashMap::new();
    for table in [&mut known, &mut physical] {
        table
            .try_reserve(bindings.len())
            .map_err(|_| ClipboardError::LimitExceeded)?;
    }
    by_source
        .try_reserve(bindings.len())
        .map_err(|_| ClipboardError::LimitExceeded)?;
    known.extend(source.objects.iter().map(|row| row.id.as_str()));
    for binding in bindings {
        if !known.contains(binding.source_id)
            || !physical.insert(binding.physical_id)
            || by_source
                .insert(binding.source_id, binding.physical_id)
                .is_some()
        {
            return Err(ClipboardError::ClipboardBinding);
        }
    }
    let mut result = source.clone();
    for row in &mut result.objects {
        row.id = by_source
            .get(row.id.as_str())
            .ok_or(ClipboardError::ClipboardBinding)?
            .to_string();
    }
    validate_layer_snapshot(&result).map_err(layer_error)?;
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
    pub snapshot: Option<ThreeDmLayerSnapshot>,
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
    #[serde(with = "generation")]
    geometry_length: u64,
    geometry_sha256: [u8; 32],
    geometry_version: u32,
    scope: LayerClipboardScope,
    snapshot: ThreeDmLayerSnapshot,
}

pub fn check_geometry_size(size: usize) -> Result<(), ClipboardError> {
    if size > MAX_GEOMETRY_BYTES {
        return Err(ClipboardError::LimitExceeded);
    }
    if size < 33 {
        return Err(ClipboardError::ClipboardGeometry);
    }
    Ok(())
}

fn geometry_version(geometry: &[u8]) -> Result<u32, ClipboardError> {
    check_geometry_size(geometry.len())?;
    if &geometry[..24] != b"3D Geometry File Format " {
        return Err(ClipboardError::ClipboardGeometry);
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
        _ => Err(ClipboardError::ClipboardGeometry),
    }
}

pub fn prepare_clipboard(
    snapshot: &ThreeDmLayerSnapshot,
    scope: LayerClipboardScope,
    geometry: &[u8],
) -> Result<Vec<u8>, ClipboardError> {
    let version = geometry_version(geometry)?;
    validate_layer_snapshot(snapshot).map_err(layer_error)?;
    #[derive(Serialize)]
    struct Borrowed<'a> {
        version: u32,
        layer_schema: u32,
        geometry_format: &'a str,
        #[serde(with = "generation")]
        geometry_length: u64,
        geometry_sha256: [u8; 32],
        geometry_version: u32,
        scope: LayerClipboardScope,
        snapshot: &'a ThreeDmLayerSnapshot,
    }
    let digest: [u8; 32] = Sha256::digest(geometry).into();
    let bytes = serde_json::to_vec(&Borrowed {
        version: 1,
        layer_schema: 1,
        geometry_format: RHINO5_FORMAT,
        geometry_length: geometry.len() as u64,
        geometry_sha256: digest,
        geometry_version: version,
        scope,
        snapshot,
    })
    .map_err(|_| ClipboardError::InvalidSnapshot)?;
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(ClipboardError::LimitExceeded);
    }
    Ok(bytes)
}

pub fn receive_clipboard(
    geometry: &[u8],
    metadata: Option<&[u8]>,
) -> Result<LayerClipboardReceived, ClipboardError> {
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
        return Err(ClipboardError::LimitExceeded);
    }
    let envelope: Envelope =
        serde_json::from_slice(metadata).map_err(|_| ClipboardError::InvalidSnapshot)?;
    if envelope.version != 1 || envelope.layer_schema != 1 {
        return Err(ClipboardError::UnsupportedVersion);
    }
    if envelope.geometry_format != RHINO5_FORMAT
        || envelope.geometry_length != geometry.len() as u64
        || envelope.geometry_version != version
    {
        return Err(ClipboardError::ClipboardBinding);
    }
    let digest: [u8; 32] = Sha256::digest(geometry).into();
    if envelope.geometry_sha256 != digest {
        return Err(ClipboardError::ClipboardBinding);
    }
    validate_layer_snapshot(&envelope.snapshot).map_err(layer_error)?;
    Ok(LayerClipboardReceived {
        snapshot: Some(envelope.snapshot),
        scope: Some(envelope.scope),
        evidence: PaletteEvidence::Extended,
        geometry_version: version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::three_dm::{ColorSource, ThreeDmLayerRow, ThreeDmObjectRow};

    fn layer(id: &str, parent: Option<&str>, names: &[&str]) -> ThreeDmLayerRow {
        ThreeDmLayerRow {
            source_id: id.into(),
            parent_id: parent.map(String::from),
            name: names.last().unwrap().to_string(),
            path_components: names.iter().map(|value| value.to_string()).collect(),
            rgb: [34, 153, 135],
            locked: false,
            visible: true,
            persistent_locked: None,
            persistent_visible: None,
        }
    }

    fn object(id: &str, layer: &str) -> ThreeDmObjectRow {
        ThreeDmObjectRow {
            id: id.into(),
            layer_id: layer.into(),
            locked: false,
            visible: true,
            color_source: ColorSource::ByLayer,
            rgb: [9, 8, 7],
        }
    }

    fn snapshot() -> ThreeDmLayerSnapshot {
        ThreeDmLayerSnapshot {
            document_id: "test-document".into(),
            generation: 7,
            active_layer: Some("metal".into()),
            layers: vec![
                layer("metal", None, &["Metal 01"]),
                layer("gem", None, &["Gem 01"]),
                layer("detail", Some("metal"), &["Metal 01", "Detail"]),
            ],
            objects: vec![
                object("ring", "metal"),
                object("stone", "gem"),
                object("detail-curve", "detail"),
            ],
        }
    }

    fn geometry() -> Vec<u8> {
        let mut value = b"3D Geometry File Format       50".to_vec();
        value.extend_from_slice(b"stub-body-for-header-policy-test");
        value
    }

    #[test]
    fn bound_metadata_is_owned_keeps_palette_and_own_flags() {
        let mut source = snapshot();
        source.generation = u64::MAX;
        source.layers.push(layer("empty", None, &["Empty"]));
        source.layers[0].locked = true;
        source.objects[1].locked = true;
        source.objects[1].color_source = ColorSource::ByObject;
        source.objects[1].rgb = [3, 5, 7];
        let body = geometry();
        let mut metadata =
            prepare_clipboard(&source, LayerClipboardScope::Selected, &body).unwrap();
        let received = receive_clipboard(&body, Some(&metadata)).unwrap();
        metadata.fill(0);
        assert_eq!(received.snapshot.as_ref(), Some(&source));
        assert_eq!(received.scope, Some(LayerClipboardScope::Selected));
        assert_eq!(received.evidence, PaletteEvidence::Extended);
        assert_eq!(received.geometry_version, 50);
    }

    #[test]
    fn native_only_never_claims_complete_empty_palette() {
        let received = receive_clipboard(&geometry(), None).unwrap();
        assert_eq!(received.evidence, PaletteEvidence::NativeOnly);
        assert!(received.snapshot.is_none() && received.scope.is_none());
    }

    #[test]
    fn digest_length_format_and_version_mismatches_reject() {
        let body = geometry();
        let metadata = prepare_clipboard(&snapshot(), LayerClipboardScope::Session, &body).unwrap();
        let mut other = body.clone();
        other[35] ^= 1;
        assert_eq!(
            receive_clipboard(&other, Some(&metadata)),
            Err(ClipboardError::ClipboardBinding)
        );
        for (key, value) in [
            ("geometry_length", serde_json::json!("1")),
            ("geometry_version", serde_json::json!(5)),
            ("geometry_format", serde_json::json!("text/plain")),
            ("geometry_sha256", serde_json::to_value([0u8; 32]).unwrap()),
        ] {
            let mut changed: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
            changed[key] = value;
            assert_eq!(
                receive_clipboard(&body, Some(&serde_json::to_vec(&changed).unwrap())),
                Err(ClipboardError::ClipboardBinding)
            );
        }
    }

    #[test]
    fn unknown_versions_fields_truncated_and_missing_metadata_are_fatal() {
        let body = geometry();
        let metadata =
            prepare_clipboard(&snapshot(), LayerClipboardScope::Selected, &body).unwrap();
        for key in ["version", "layer_schema"] {
            let mut changed: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
            changed[key] = serde_json::json!(2);
            assert_eq!(
                receive_clipboard(&body, Some(&serde_json::to_vec(&changed).unwrap())),
                Err(ClipboardError::UnsupportedVersion)
            );
        }
        assert!(receive_clipboard(&body, Some(&metadata[..metadata.len() - 1])).is_err());
        assert_eq!(
            receive_clipboard(&body, Some(b"")),
            Err(ClipboardError::InvalidSnapshot)
        );
        let mut extra: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
        extra["render_mesh"] = serde_json::json!([]);
        assert_eq!(
            receive_clipboard(&body, Some(&serde_json::to_vec(&extra).unwrap())),
            Err(ClipboardError::InvalidSnapshot)
        );
    }

    #[test]
    fn payload_header_and_size_policy_is_not_sdk_geometry_acceptance() {
        for body in [
            b"".as_slice(),
            b"H:/a.3dm",
            b"3D Geometry File Format       80invalid",
            b"3D Geometry File Format       50",
        ] {
            assert_eq!(
                receive_clipboard(body, None),
                Err(ClipboardError::ClipboardGeometry)
            );
        }
        assert_eq!(
            check_geometry_size(MAX_GEOMETRY_BYTES + 1),
            Err(ClipboardError::LimitExceeded)
        );
        assert_eq!(
            check_geometry_size(32),
            Err(ClipboardError::ClipboardGeometry)
        );
        let mut v5 = geometry();
        v5[24..32].copy_from_slice(b"       5");
        assert_eq!(receive_clipboard(&v5, None).unwrap().geometry_version, 5);
    }

    #[test]
    fn palette_only_session_and_invalid_source_are_explicit() {
        let mut source = ThreeDmLayerSnapshot {
            document_id: "palette".into(),
            generation: 0,
            layers: vec![layer("empty", None, &["Empty"])],
            objects: vec![],
            active_layer: None,
        };
        let bytes = prepare_clipboard(&source, LayerClipboardScope::Session, &geometry()).unwrap();
        assert!(
            receive_clipboard(&geometry(), Some(&bytes))
                .unwrap()
                .snapshot
                .unwrap()
                .objects
                .is_empty()
        );
        source.layers[0].parent_id = Some("missing".into());
        assert_eq!(
            prepare_clipboard(&source, LayerClipboardScope::Selected, &geometry()),
            Err(ClipboardError::MissingLayer)
        );
    }

    #[test]
    fn member_copy_budget_and_rollback_ownership_are_explicit() {
        assert!(rollback_owned_publication(1, 2, 2));
        assert!(!rollback_owned_publication(1, 2, 3));
        assert!(!rollback_owned_publication(1, 0, 0));
        assert_eq!(check_clipboard_backup(0, 256), Ok(()));
        assert_eq!(
            check_clipboard_backup(MAX_GEOMETRY_BYTES + 1, 1),
            Err(ClipboardError::LimitExceeded)
        );
        assert_eq!(
            check_clipboard_backup(0, 257),
            Err(ClipboardError::LimitExceeded)
        );
    }

    #[test]
    fn geometry_binding_requires_one_unique_tag_per_source_object() {
        let source = snapshot();
        let bindings = [
            GeometryObjectBinding {
                physical_id: "p-ring",
                source_id: "ring",
            },
            GeometryObjectBinding {
                physical_id: "p-stone",
                source_id: "stone",
            },
            GeometryObjectBinding {
                physical_id: "p-curve",
                source_id: "detail-curve",
            },
        ];
        let bound = bind_geometry_objects(&source, &bindings).unwrap();
        assert_eq!(bound.objects[0].id, "p-ring");
        assert_eq!(bound.objects[1].id, "p-stone");
        assert_eq!(bound.objects[2].id, "p-curve");
        assert_eq!(
            bind_geometry_objects(
                &source,
                &[
                    GeometryObjectBinding {
                        physical_id: "p-ring",
                        source_id: "ring",
                    },
                    GeometryObjectBinding {
                        physical_id: "p-stone",
                        source_id: "ring",
                    },
                    GeometryObjectBinding {
                        physical_id: "p-curve",
                        source_id: "detail-curve",
                    },
                ]
            ),
            Err(ClipboardError::ClipboardBinding)
        );
        assert_eq!(
            bind_geometry_objects(&source, &bindings[..2]),
            Err(ClipboardError::ClipboardBinding)
        );
    }
}
