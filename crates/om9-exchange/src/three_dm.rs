// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 3DM import/export/copy policy (`OM9-FILE-012`).
//!
//! Ported from the Phase2 Rust owner `rust/src/core_3dm_archive.rs` and the layer snapshot
//! validation in `rust/src/layer_state.rs`. The native openNURBS bridge and the FreeCAD host
//! adapters are not copied here; this module owns the portable admission rules the writer runs
//! before native conversion:
//!
//! - archive mode/capability and identity policy;
//! - member-copy budget and retained-overlay permissions;
//! - iterative dependency closure for retained source graphs;
//! - strict geometry-only export manifest validation (schema 1 and the schema-2 canonical layer
//!   session), so a malformed tuple is refused before Qt/native conversion can coerce it to zero.

use om9_core::budget::Limits;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/* ------------------------------- archive policy ------------------------------- */

/// Import policy: editable geometry only, or byte-preserving retained data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveMode {
    GeometryOnly = 0,
    Preserve = 1,
}

impl ArchiveMode {
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::GeometryOnly),
            1 => Some(Self::Preserve),
            _ => None,
        }
    }
}

/// What an imported native record can support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveCapability {
    Editable = 1,
    DisplayRetained = 2,
    Retained = 3,
    Incompatible = 4,
}

impl ArchiveCapability {
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            1 => Some(Self::Editable),
            2 => Some(Self::DisplayRetained),
            3 => Some(Self::Retained),
            4 => Some(Self::Incompatible),
            _ => None,
        }
    }
}

/// Source identity: which import namespace a record came from and its native UUID.
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
        && bytes.iter().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                *byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)
            }
        })
        && bytes.iter().any(|byte| *byte != b'0' && *byte != b'-')
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

/// The largest member-copy operation the host accepts.
pub const MAX_COPY_BYTES: usize = 512 * 1024 * 1024;

/// Admit copying `bytes` per copy, `copies` times, with `total` already retained.
///
/// `None` means the request exceeds the copy budget or overflows `usize`.
pub fn copy_budget(bytes: usize, copies: usize, total: usize) -> Option<usize> {
    bytes
        .checked_mul(copies)
        .and_then(|value| total.checked_add(value))
        .filter(|value| *value <= MAX_COPY_BYTES)
}

/// The edit an overlay may apply to a retained record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum OverlayAction {
    Unchanged = 0,
    ReplaceGeometry = 1,
    TransformInstance = 2,
    Duplicate = 3,
    TransformNativeGeometry = 4,
}

/// Whether the retained source can accept `action`. Unknown references, an incompatible record
/// and unknown codes are refused; only editable records may replace geometry.
pub fn overlay_allowed(
    capability: u32,
    action: u32,
    known_references: bool,
    is_instance: bool,
) -> bool {
    let Some(capability) = ArchiveCapability::from_code(capability) else {
        return false;
    };
    if !known_references || capability == ArchiveCapability::Incompatible {
        return false;
    }
    match action {
        0 | 3 => true,
        1 => capability == ArchiveCapability::Editable,
        2 => is_instance,
        4 => !is_instance,
        _ => false,
    }
}

/* ----------------------------- dependency closure ----------------------------- */

pub const MAX_ARCHIVE_NODES: usize = 1_000_000;
pub const MAX_ARCHIVE_EDGES: usize = 16_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureError {
    Malformed,
    Cycle,
    Limit,
}

/// The selected nodes plus every transitive dependency, in node-index order.
///
/// `offsets[i]..offsets[i + 1]` indexes the outgoing edges of node `i`. The traversal is iterative,
/// so a 100,000-node chain cannot overflow the stack.
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
        || offsets.windows(2).any(|pair| pair[0] > pair[1])
        || edges.iter().any(|index| *index >= node_count)
        || selected.iter().any(|index| *index >= node_count)
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
        .filter_map(|(index, state)| (*state == 2).then_some(index))
        .collect())
}

/* ------------------------------- layer session ------------------------------- */

pub const MAX_LAYERS: usize = 65_536;
pub const MAX_OBJECTS: usize = 2_000_000;
pub const MAX_DEPTH: usize = 128;
pub const MAX_METADATA_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerSnapshotError {
    Invalid,
    MissingLayer,
    Duplicate,
    AmbiguousPath,
    Cycle,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ColorSource {
    ByLayer,
    ByObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeDmLayerRow {
    pub source_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub path_components: Vec<String>,
    pub rgb: [u8; 3],
    /// Local desired state, not the flattened state imposed by a parent.
    pub locked: bool,
    pub visible: bool,
    pub persistent_locked: Option<bool>,
    pub persistent_visible: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeDmObjectRow {
    pub id: String,
    pub layer_id: String,
    pub locked: bool,
    pub visible: bool,
    pub color_source: ColorSource,
    pub rgb: [u8; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeDmLayerSnapshot {
    pub document_id: String,
    #[serde(with = "generation")]
    pub generation: u64,
    pub layers: Vec<ThreeDmLayerRow>,
    pub objects: Vec<ThreeDmObjectRow>,
    pub active_layer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreeDmLayerSession {
    pub version: u32,
    pub snapshot: ThreeDmLayerSnapshot,
}

/// The legacy wire format carried generations as canonical decimal text because a JSON
/// number can round-trip through `f64` and lose a `u64` bit.
pub(crate) mod generation {
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.is_empty()
            || text.len() > 20
            || (text.len() > 1 && text.starts_with('0'))
            || !text.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(de::Error::custom(
                "generation must be canonical uint64 decimal text",
            ));
        }
        text.parse().map_err(de::Error::custom)
    }
}

pub(crate) fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= 1024 && !value.chars().any(char::is_control)
}

fn valid_component(value: &str) -> bool {
    valid_identity(value) && !value.contains("::")
}

/// Per-component simple uppercase; expansions remain distinct (`ß` is not `SS`).
fn path_key(components: &[String]) -> Vec<String> {
    components
        .iter()
        .map(|component| {
            component
                .chars()
                .map(|character| {
                    let mut upper = character.to_uppercase();
                    let first = upper.next().unwrap_or(character);
                    if upper.next().is_none() {
                        first
                    } else {
                        character
                    }
                })
                .collect()
        })
        .collect()
}

pub(crate) fn metadata_bytes(snapshot: &ThreeDmLayerSnapshot) -> Result<usize, LayerSnapshotError> {
    let mut total = snapshot.document_id.len() + std::mem::size_of::<ThreeDmLayerSnapshot>();
    let mut add = |bytes: usize| -> Result<(), LayerSnapshotError> {
        total = total.checked_add(bytes).ok_or(LayerSnapshotError::Limit)?;
        if total > MAX_METADATA_BYTES {
            return Err(LayerSnapshotError::Limit);
        }
        Ok(())
    };
    if snapshot.layers.len() > MAX_LAYERS || snapshot.objects.len() > MAX_OBJECTS {
        return Err(LayerSnapshotError::Limit);
    }
    for layer in &snapshot.layers {
        add(std::mem::size_of::<ThreeDmLayerRow>())?;
        add(layer.source_id.len())?;
        add(layer.name.len())?;
        if let Some(parent) = &layer.parent_id {
            add(parent.len())?;
        }
        for component in &layer.path_components {
            add(std::mem::size_of::<String>())?;
            add(component.len())?;
        }
    }
    for object in &snapshot.objects {
        add(std::mem::size_of::<ThreeDmObjectRow>())?;
        add(object.id.len())?;
        add(object.layer_id.len())?;
    }
    if let Some(active) = &snapshot.active_layer {
        add(active.len())?;
    }
    Ok(total)
}

/// Validate the canonical layer session before any geometry is bound.
pub fn validate_layer_snapshot(snapshot: &ThreeDmLayerSnapshot) -> Result<(), LayerSnapshotError> {
    metadata_bytes(snapshot)?;
    if !valid_identity(&snapshot.document_id) {
        return Err(LayerSnapshotError::Invalid);
    }
    let mut by_layer = HashMap::with_capacity(snapshot.layers.len());
    let mut paths = HashSet::with_capacity(snapshot.layers.len());
    for (index, layer) in snapshot.layers.iter().enumerate() {
        if !valid_identity(&layer.source_id) {
            return Err(LayerSnapshotError::Invalid);
        }
        if by_layer.insert(layer.source_id.as_str(), index).is_some() {
            return Err(LayerSnapshotError::Duplicate);
        }
        if layer.path_components.is_empty()
            || layer.path_components.len() > MAX_DEPTH
            || layer
                .path_components
                .iter()
                .any(|component| !valid_component(component))
            || layer.path_components.last() != Some(&layer.name)
        {
            return Err(LayerSnapshotError::Invalid);
        }
        if layer.path_components.iter().map(String::len).sum::<usize>() > 4096 {
            return Err(LayerSnapshotError::Limit);
        }
        if !paths.insert(path_key(&layer.path_components)) {
            return Err(LayerSnapshotError::AmbiguousPath);
        }
    }
    let parents: Vec<Option<usize>> = snapshot
        .layers
        .iter()
        .map(|layer| {
            layer
                .parent_id
                .as_ref()
                .map(|parent| {
                    by_layer
                        .get(parent.as_str())
                        .copied()
                        .ok_or(LayerSnapshotError::MissingLayer)
                })
                .transpose()
        })
        .collect::<Result<_, _>>()?;
    // Resolve parents iteratively before checking path text so cycles are explicit.
    let mut state = vec![0u8; snapshot.layers.len()];
    for start in 0..snapshot.layers.len() {
        if state[start] == 2 {
            continue;
        }
        let mut chain = Vec::new();
        let mut current = Some(start);
        while let Some(index) = current {
            match state[index] {
                2 => break,
                1 => return Err(LayerSnapshotError::Cycle),
                _ => {}
            }
            if chain.len() >= MAX_DEPTH {
                return Err(LayerSnapshotError::Limit);
            }
            state[index] = 1;
            chain.push(index);
            current = parents[index];
        }
        for index in chain.into_iter().rev() {
            state[index] = 2;
        }
    }
    for (index, layer) in snapshot.layers.iter().enumerate() {
        match parents[index] {
            None if layer.path_components.len() != 1 => {
                return Err(LayerSnapshotError::Invalid);
            }
            Some(parent)
                if layer.path_components[..layer.path_components.len() - 1]
                    != snapshot.layers[parent].path_components =>
            {
                return Err(LayerSnapshotError::Invalid);
            }
            _ => {}
        }
    }
    if snapshot
        .active_layer
        .as_ref()
        .is_some_and(|active| !by_layer.contains_key(active.as_str()))
    {
        return Err(LayerSnapshotError::MissingLayer);
    }
    let mut objects = HashSet::with_capacity(snapshot.objects.len());
    for object in &snapshot.objects {
        if !valid_identity(&object.id) {
            return Err(LayerSnapshotError::Invalid);
        }
        if !objects.insert(object.id.as_str()) {
            return Err(LayerSnapshotError::Duplicate);
        }
        if !by_layer.contains_key(object.layer_id.as_str()) {
            return Err(LayerSnapshotError::MissingLayer);
        }
    }
    Ok(())
}

/* ------------------------------- export manifest ------------------------------- */

pub const MAX_MANIFEST_BYTES: usize = 32 * 1024 * 1024;
const MAX_MANIFEST_NODES: u64 = 1_000_000;
const ITEM_KEYS: [&str; 11] = [
    "name",
    "layer",
    "visible",
    "locked",
    "color",
    "brep",
    "vertices",
    "faces",
    "point_cloud_fields",
    "point_cloud_sha256",
    "point_cloud_transform",
];
const POINT_CLOUD_KEYS: [&str; 3] = [
    "point_cloud_fields",
    "point_cloud_sha256",
    "point_cloud_transform",
];

fn number(value: Option<&Value>) -> Option<f64> {
    value?.as_f64().filter(|value| value.is_finite())
}

fn whole_number(value: Option<&Value>) -> Option<u64> {
    let value = value?;
    if let Some(integer) = value.as_u64() {
        return Some(integer);
    }
    let number = value.as_f64()?;
    if number.is_finite() && number >= 0.0 && number.fract() == 0.0 && number <= u64::MAX as f64 {
        Some(number as u64)
    } else {
        None
    }
}

/// Validate the geometry-only export manifest before native conversion.
///
/// Schema 1 is the legacy mesh/B-Rep/PointCloud manifest. Schema 2 adds a canonical layer session
/// and binds every geometry item to a layer object id. Unknown fields, duplicate JSON keys,
/// malformed tuples, retained data in a geometry-only packet and inventory mismatches are refused.
pub fn validate_export_manifest(raw: &str) -> Result<(), &'static str> {
    let limits = Limits {
        max_bytes: MAX_MANIFEST_BYTES as u64,
        max_nodes: MAX_MANIFEST_NODES,
        ..Limits::default()
    };
    om9_archive::admit_json(raw.as_bytes(), &limits).map_err(|_| "invalid export JSON")?;
    let value: Value = serde_json::from_str(raw).map_err(|_| "invalid export JSON")?;
    let root = value.as_object().ok_or("export manifest object required")?;
    let canonical = match root.get("schema_version") {
        None => false,
        Some(Value::Number(number)) if number.as_f64() == Some(1.0) => false,
        Some(Value::Number(number)) if number.as_f64() == Some(2.0) => true,
        _ => return Err("unsupported export schema version"),
    };
    if root.keys().any(|key| {
        !matches!(key.as_str(), "items" | "tolerance" | "schema_version")
            && !(canonical && key == "layer_session")
    }) {
        return Err("unsupported export manifest field");
    }
    if root
        .get("tolerance")
        .is_some_and(|tolerance| number(Some(tolerance)).is_none_or(|value| value <= 0.0))
    {
        return Err("positive export tolerance required");
    }
    let items = root
        .get("items")
        .and_then(Value::as_array)
        .ok_or("export items required")?;
    let layers = if canonical {
        let session = root
            .get("layer_session")
            .ok_or("canonical layer session required")?;
        let session: ThreeDmLayerSession = serde_json::from_value(session.clone())
            .map_err(|_| "invalid canonical layer session")?;
        if session.version != 1 {
            return Err("invalid canonical layer session");
        }
        validate_layer_snapshot(&session.snapshot)
            .map_err(|_| "invalid canonical layer session")?;
        Some(session.snapshot)
    } else {
        None
    };
    if items.is_empty()
        && layers
            .as_ref()
            .is_none_or(|state| state.layers.is_empty() || !state.objects.is_empty())
    {
        return Err("no geometry to export");
    }
    let mut bindings = HashSet::new();
    for item in items {
        let item = item.as_object().ok_or("export item object required")?;
        validate_item(item, canonical, &mut bindings)?;
    }
    if let Some(state) = layers
        && (state.objects.len() != bindings.len()
            || state
                .objects
                .iter()
                .any(|object| !bindings.contains(object.id.as_str())))
    {
        return Err("canonical geometry and layer inventory disagree");
    }
    Ok(())
}

fn validate_item(
    item: &Map<String, Value>,
    canonical: bool,
    bindings: &mut HashSet<String>,
) -> Result<(), &'static str> {
    if item
        .keys()
        .any(|key| !(canonical && key == "layer_object_id") && !ITEM_KEYS.contains(&key.as_str()))
    {
        return Err("unsupported geometry-only item field");
    }
    if canonical {
        let Some(Value::String(id)) = item.get("layer_object_id") else {
            return Err("canonical geometry object binding required");
        };
        if id.is_empty() || id.len() > 1024 || id.contains('\0') || !bindings.insert(id.clone()) {
            return Err("invalid or duplicate canonical geometry binding");
        }
    }
    for key in ["name", "layer"] {
        if let Some(value) = item.get(key)
            && !matches!(value, Value::String(text) if text.len() <= 32768 && !text.contains('\0'))
        {
            return Err("invalid export name or layer");
        }
    }
    for key in ["visible", "locked"] {
        if item
            .get(key)
            .is_some_and(|value| !matches!(value, Value::Bool(_)))
        {
            return Err("export visibility and lock must be boolean");
        }
    }
    if let Some(color) = item.get("color")
        && !matches!(color, Value::Array(values) if values.len() == 3
            && values.iter().all(|value| whole_number(Some(value)).is_some_and(|byte| byte <= 255)))
    {
        return Err("export color must contain three byte values");
    }
    if POINT_CLOUD_KEYS.iter().any(|key| item.contains_key(*key)) {
        if ["brep", "vertices", "faces"]
            .iter()
            .any(|key| item.contains_key(*key))
        {
            return Err("exclusive staged PointCloud fields required");
        }
        if !matches!(item.get("point_cloud_fields"), Some(Value::String(path))
            if !path.is_empty() && path.len() <= 32768 && !path.contains('\0'))
        {
            return Err("bounded PointCloud fields path required");
        }
        if !matches!(item.get("point_cloud_sha256"), Some(Value::String(hash))
            if hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return Err("PointCloud SHA256 required");
        }
        if item.get("point_cloud_transform").is_some_and(|value| {
            !matches!(value, Value::Array(matrix) if matrix.len() == 16
                && matrix.iter().all(|value| number(Some(value)).is_some()))
        }) {
            return Err("PointCloud transform requires sixteen finite numbers");
        }
        return Ok(());
    }
    if let Some(brep) = item.get("brep") {
        if item.contains_key("vertices")
            || item.contains_key("faces")
            || !matches!(brep, Value::String(path)
                if !path.is_empty() && path.len() <= 32768 && !path.contains('\0'))
        {
            return Err("exclusive staged BRep path required");
        }
        return Ok(());
    }
    let (Some(Value::Array(vertices)), Some(Value::Array(faces))) =
        (item.get("vertices"), item.get("faces"))
    else {
        return Err("mesh vertices and faces required");
    };
    if vertices.len() < 3 || faces.is_empty() {
        return Err("empty mesh geometry");
    }
    for vertex in vertices {
        if !matches!(vertex, Value::Array(values) if values.len() == 3
            && values.iter().all(|value| number(Some(value)).is_some()))
        {
            return Err("mesh vertex must contain three finite numbers");
        }
    }
    for face in faces {
        if !matches!(face, Value::Array(values) if values.len() == 4
            && values.iter().all(|value| whole_number(Some(value))
                .is_some_and(|index| index < vertices.len() as u64)))
        {
            return Err("mesh face must contain four in-range integer indices");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MESH: &str = r#"{"items":[{"name":"mesh","layer":"Default","visible":true,"locked":false,"color":[128,0,255],"vertices":[[0,0,0],[1,0,0],[0,1,0]],"faces":[[0,1,2,2]]}],"tolerance":0.000001}"#;

    #[test]
    fn export_manifest_preserves_exact_mesh_and_brep_schema() {
        assert!(validate_export_manifest(MESH).is_ok());
        assert!(
            validate_export_manifest(
                r#"{"items":[{"brep":"C:/staged/shape.brep","color":[0,1,255]}],"tolerance":0.001}"#
            )
            .is_ok()
        );
    }

    #[test]
    fn malformed_mesh_tuples_cannot_be_coerced_to_zero_or_truncated() {
        for (from, to) in [
            ("[0,0,0]", "[0,0]"),
            ("[0,0,0]", "[0,0,0,7]"),
            ("[0,0,0]", "[0,\"1\",0]"),
            ("[0,1,2,2]", "[0,1,2]"),
            ("[0,1,2,2]", "[0,1,3,3]"),
            ("[0,1,2,2]", "[0,1,2.1,2]"),
            ("[0,1,2,2]", "[0,-1,2,2]"),
            ("[128,0,255]", "[128,0]"),
            ("[128,0,255]", "[128,0,256]"),
            ("true", "null"),
        ] {
            assert!(
                validate_export_manifest(&MESH.replace(from, to)).is_err(),
                "{to}"
            );
        }
    }

    #[test]
    fn invalid_schema_or_retained_data_is_rejected_before_native_conversion() {
        for raw in [
            "{}",
            r#"{"items":[],"tolerance":1}"#,
            r#"{"items":[{"brep":"","color":[0,0,0]}],"tolerance":1}"#,
            r#"{"items":[{"brep":"x\u0000y","color":[0,0,0]}],"tolerance":1}"#,
        ] {
            assert!(validate_export_manifest(raw).is_err(), "{raw}");
        }
        for (from, to) in [
            ("0.000001", "0"),
            ("0.000001", "\"0.1\""),
            ("\"name\":\"mesh\"", "\"retained\":true"),
            ("\"name\":\"mesh\"", "\"brep\":\"shape.brep\""),
            (
                "\"name\":\"mesh\"",
                "\"name\":\"mesh\",\"name\":\"duplicate\"",
            ),
        ] {
            assert!(
                validate_export_manifest(&MESH.replace(from, to)).is_err(),
                "{to}"
            );
        }
    }

    #[test]
    fn current_point_cloud_export_fields_survive_p0_preflight() {
        let cloud = r#"{"items":[{"point_cloud_fields":"C:/staged/cloud.json","point_cloud_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","point_cloud_transform":[1,0,0,10,0,1,0,20,0,0,1,30,0,0,0,1]}],"tolerance":0.001}"#;
        assert!(validate_export_manifest(cloud).is_ok());
        for (from, to) in [
            ("C:/staged/cloud.json", ""),
            (
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "short",
            ),
            ("[1,0,0,10,0,1,0,20,0,0,1,30,0,0,0,1]", "[1,0,0]"),
            (
                "\"point_cloud_fields\":",
                "\"brep\":\"shape.brep\",\"point_cloud_fields\":",
            ),
        ] {
            assert!(
                validate_export_manifest(&cloud.replace(from, to)).is_err(),
                "{to}"
            );
        }
    }

    #[test]
    fn legacy_optional_color_keeps_the_native_default_and_checks_explicit_values() {
        assert!(validate_export_manifest(r#"{"items":[{"brep":"shape.brep"}]}"#).is_ok());
        assert!(
            validate_export_manifest(r#"{"items":[{"brep":"shape.brep"}],"tolerance":0.001}"#)
                .is_ok()
        );
        assert!(
            validate_export_manifest(
                r#"{"items":[{"brep":"shape.brep","color":null}],"tolerance":0.001}"#
            )
            .is_err()
        );
    }
}
