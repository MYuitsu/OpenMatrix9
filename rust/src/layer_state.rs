//! Owned layer metadata. Display meshes and geometry are deliberately absent.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type LayerId = String;
pub type ObjectId = String;
pub const MAX_LAYERS: usize = 65_536;
pub const MAX_OBJECTS: usize = 2_000_000;
pub const MAX_DEPTH: usize = 128;
pub const MAX_METADATA_BYTES: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum LayerError {
    InvalidSnapshot = 1,
    MissingLayer = 2,
    MissingObject = 3,
    DuplicateId = 4,
    AmbiguousPath = 5,
    InvalidPath = 6,
    Cycle = 7,
    LimitExceeded = 8,
    Locked = 9,
    Hidden = 10,
    EmptySelection = 11,
    Stale = 12,
    InvalidHandle = 13,
    UnsupportedVersion = 14,
    BufferTooSmall = 15,
    ClipboardGeometry = 16,
    ClipboardBinding = 17,
}
impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidSnapshot => "Invalid layer snapshot",
            Self::MissingLayer => "Layer reference does not exist",
            Self::MissingObject => "Object reference does not exist",
            Self::DuplicateId => "Duplicate identity or selection",
            Self::AmbiguousPath => "Ambiguous layer full path",
            Self::InvalidPath => "Invalid layer path or parent path",
            Self::Cycle => "Layer parent cycle",
            Self::LimitExceeded => "Layer metadata limit exceeded",
            Self::Locked => "Object or layer is locked",
            Self::Hidden => "Object or layer is hidden",
            Self::EmptySelection => "No objects selected",
            Self::Stale => "Document state changed before commit",
            Self::InvalidHandle => "Expired layer state handle",
            Self::UnsupportedVersion => "Unsupported layer schema or operation",
            Self::BufferTooSmall => "Layer result buffer is too small",
            Self::ClipboardGeometry => "Clipboard does not contain a bounded Rhino5 native archive",
            Self::ClipboardBinding => {
                "Layer metadata does not match the clipboard geometry payload"
            }
        })
    }
}
impl std::error::Error for LayerError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ColorSource {
    ByLayer = 1,
    ByObject = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerRow {
    pub source_id: LayerId,
    pub parent_id: Option<LayerId>,
    pub name: String,
    pub path_components: Vec<String>,
    pub rgb: [u8; 3],
    /// Local desired state, not the flattened state imposed by a parent.
    pub locked: bool,
    pub visible: bool,
    /// Original native persistent fields, including unset versus explicitly set.
    pub persistent_locked: Option<bool>,
    pub persistent_visible: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectLayerRow {
    pub id: ObjectId,
    pub layer_id: LayerId,
    pub locked: bool,
    pub visible: bool,
    pub color_source: ColorSource,
    pub rgb: [u8; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerSnapshotV1 {
    pub document_id: String,
    #[serde(with = "crate::layer_codec::generation")]
    pub generation: u64,
    #[serde(deserialize_with = "crate::layer_codec::deserialize_layers")]
    pub layers: Vec<LayerRow>,
    #[serde(deserialize_with = "crate::layer_codec::deserialize_objects")]
    pub objects: Vec<ObjectLayerRow>,
    pub active_layer: Option<LayerId>,
}
impl LayerSnapshotV1 {
    pub fn empty(document_id: &str) -> Self {
        Self {
            document_id: document_id.into(),
            generation: 0,
            layers: vec![],
            objects: vec![],
            active_layer: None,
        }
    }
    /// Conservative bounded payload accounting, excluding geometry by design.
    pub fn metadata_bytes(&self) -> Result<usize, LayerError> {
        let mut total = self.document_id.len() + std::mem::size_of::<Self>();
        let mut add = |n: usize| -> Result<(), LayerError> {
            total = total.checked_add(n).ok_or(LayerError::LimitExceeded)?;
            if total > MAX_METADATA_BYTES {
                return Err(LayerError::LimitExceeded);
            }
            Ok(())
        };
        if self.layers.len() > MAX_LAYERS || self.objects.len() > MAX_OBJECTS {
            return Err(LayerError::LimitExceeded);
        }
        for l in &self.layers {
            add(std::mem::size_of::<LayerRow>())?;
            add(l.source_id.len())?;
            add(l.name.len())?;
            if let Some(p) = &l.parent_id {
                add(p.len())?;
            }
            for c in &l.path_components {
                add(std::mem::size_of::<String>())?;
                add(c.len())?;
            }
        }
        for o in &self.objects {
            add(std::mem::size_of::<ObjectLayerRow>())?;
            add(o.id.len())?;
            add(o.layer_id.len())?;
        }
        if let Some(active) = &self.active_layer {
            add(active.len())?;
        }
        Ok(total)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveState {
    pub locked: bool,
    pub visible: bool,
    pub rgb: [u8; 3],
}
impl EffectiveState {
    pub fn selectable(self) -> bool {
        self.visible && !self.locked
    }
    /// Eligibility only; the existing CAD-kind/budget policy still gates snap.
    pub fn snap_eligible(self) -> bool {
        self.visible
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerOperation {
    Edit,
    Transform,
    Delete,
    AssignLayer(LayerId),
    Export,
}

/// Per-component simple uppercase; expansions remain distinct (ß is not SS).
/// No Unicode normalization: distinct native names must not be merged silently.
/// Native adapters must verify their ordinal mapping with the installed host.
pub fn path_key(components: &[String]) -> Vec<String> {
    components
        .iter()
        .map(|s| {
            s.chars()
                .map(|c| {
                    let mut upper = c.to_uppercase();
                    let first = upper.next().unwrap_or(c);
                    if upper.next().is_none() { first } else { c }
                })
                .collect()
        })
        .collect()
}
pub(crate) fn valid_identity(s: &str) -> bool {
    !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control)
}
pub(crate) fn valid_component(s: &str) -> bool {
    valid_identity(s) && !s.contains("::")
}

/// Borrowed index only lives as long as its owned snapshot. No geometry access.
pub struct LayerIndex<'a> {
    snapshot: &'a LayerSnapshotV1,
    by_layer: HashMap<&'a str, usize>,
    by_object: HashMap<&'a str, usize>,
    layer_effective: Vec<EffectiveState>,
    members: Vec<Vec<usize>>,
}
impl<'a> LayerIndex<'a> {
    pub fn new(snapshot: &'a LayerSnapshotV1) -> Result<Self, LayerError> {
        snapshot.metadata_bytes()?;
        if !valid_identity(&snapshot.document_id) {
            return Err(LayerError::InvalidSnapshot);
        }
        let mut by_layer = HashMap::with_capacity(snapshot.layers.len());
        let mut by_object = HashMap::with_capacity(snapshot.objects.len());
        let mut paths = HashSet::with_capacity(snapshot.layers.len());
        for (i, l) in snapshot.layers.iter().enumerate() {
            if !valid_identity(&l.source_id) {
                return Err(LayerError::InvalidSnapshot);
            }
            if by_layer.insert(l.source_id.as_str(), i).is_some() {
                return Err(LayerError::DuplicateId);
            }
            if l.path_components.is_empty()
                || l.path_components.len() > MAX_DEPTH
                || l.path_components.iter().any(|c| !valid_component(c))
                || l.path_components.last() != Some(&l.name)
            {
                return Err(LayerError::InvalidPath);
            }
            if l.path_components.iter().map(String::len).sum::<usize>() > 4096 {
                return Err(LayerError::LimitExceeded);
            }
            if !paths.insert(path_key(&l.path_components)) {
                return Err(LayerError::AmbiguousPath);
            }
        }
        let parents: Vec<Option<usize>> = snapshot
            .layers
            .iter()
            .map(|l| {
                l.parent_id
                    .as_ref()
                    .map(|p| {
                        by_layer
                            .get(p.as_str())
                            .copied()
                            .ok_or(LayerError::MissingLayer)
                    })
                    .transpose()
            })
            .collect::<Result<_, _>>()?;
        // Resolve parents iteratively before checking path text so cycles are explicit.
        let mut effective: Vec<Option<EffectiveState>> = vec![None; snapshot.layers.len()];
        for start in 0..snapshot.layers.len() {
            let mut chain = vec![];
            let mut current = Some(start);
            while let Some(i) = current {
                if effective[i].is_some() {
                    break;
                }
                if chain.contains(&i) {
                    return Err(LayerError::Cycle);
                }
                if chain.len() >= MAX_DEPTH {
                    return Err(LayerError::LimitExceeded);
                }
                chain.push(i);
                current = parents[i];
            }
            while let Some(i) = chain.pop() {
                let l = &snapshot.layers[i];
                let parent = parents[i].and_then(|p| effective[p]);
                effective[i] = Some(EffectiveState {
                    locked: l.locked || parent.is_some_and(|p| p.locked),
                    visible: l.visible && parent.is_none_or(|p| p.visible),
                    rgb: l.rgb,
                });
            }
        }
        for (i, l) in snapshot.layers.iter().enumerate() {
            match parents[i] {
                None if l.path_components.len() != 1 => return Err(LayerError::InvalidPath),
                Some(p)
                    if l.path_components[..l.path_components.len() - 1]
                        != snapshot.layers[p].path_components =>
                {
                    return Err(LayerError::InvalidPath);
                }
                _ => {}
            }
        }
        if snapshot
            .active_layer
            .as_ref()
            .is_some_and(|id| !by_layer.contains_key(id.as_str()))
        {
            return Err(LayerError::MissingLayer);
        }
        let mut members = vec![Vec::new(); snapshot.layers.len()];
        for (i, o) in snapshot.objects.iter().enumerate() {
            if !valid_identity(&o.id) {
                return Err(LayerError::InvalidSnapshot);
            }
            if by_object.insert(o.id.as_str(), i).is_some() {
                return Err(LayerError::DuplicateId);
            }
            let l = *by_layer
                .get(o.layer_id.as_str())
                .ok_or(LayerError::MissingLayer)?;
            members[l].push(i);
        }
        Ok(Self {
            snapshot,
            by_layer,
            by_object,
            layer_effective: effective
                .into_iter()
                .map(|e| e.expect("resolved layer"))
                .collect(),
            members,
        })
    }
    pub fn layer_effective(&self, id: &str) -> Result<EffectiveState, LayerError> {
        self.by_layer
            .get(id)
            .map(|&i| self.layer_effective[i])
            .ok_or(LayerError::MissingLayer)
    }
    pub fn members(&self, id: &str) -> Result<&[usize], LayerError> {
        self.by_layer
            .get(id)
            .map(|&i| self.members[i].as_slice())
            .ok_or(LayerError::MissingLayer)
    }
    pub fn object_index(&self, id: &str) -> Result<usize, LayerError> {
        self.by_object
            .get(id)
            .copied()
            .ok_or(LayerError::MissingObject)
    }
    pub fn effective(&self, id: &str) -> Result<EffectiveState, LayerError> {
        let o = &self.snapshot.objects[self.object_index(id)?];
        let l = self.layer_effective(&o.layer_id)?;
        Ok(EffectiveState {
            locked: o.locked || l.locked,
            visible: o.visible && l.visible,
            rgb: if o.color_source == ColorSource::ByLayer {
                l.rgb
            } else {
                o.rgb
            },
        })
    }
}
pub fn validate_layers(snapshot: &LayerSnapshotV1) -> Result<(), LayerError> {
    LayerIndex::new(snapshot).map(|_| ())
}
pub fn effective_state(snapshot: &LayerSnapshotV1, id: &str) -> Result<EffectiveState, LayerError> {
    LayerIndex::new(snapshot)?.effective(id)
}
pub fn can_mutate(
    snapshot: &LayerSnapshotV1,
    ids: &[ObjectId],
    operation: &LayerOperation,
) -> Result<(), LayerError> {
    let index = LayerIndex::new(snapshot)?;
    check_mutation(
        ids,
        operation,
        |id| index.effective(id),
        |id| index.layer_effective(id),
    )
}
/// Shared policy for both borrowed indexes and the owned ABI cache.
pub fn check_mutation(
    ids: &[ObjectId],
    operation: &LayerOperation,
    object_state: impl Fn(&str) -> Result<EffectiveState, LayerError>,
    layer_state: impl Fn(&str) -> Result<EffectiveState, LayerError>,
) -> Result<(), LayerError> {
    if ids.is_empty() {
        return Err(LayerError::EmptySelection);
    }
    let mut seen = HashSet::with_capacity(ids.len());
    // Validate the complete selection first, independent of its order.
    let mut states = Vec::with_capacity(ids.len());
    for id in ids {
        states.push(object_state(id)?);
        if !seen.insert(id) {
            return Err(LayerError::DuplicateId);
        }
    }
    if *operation == LayerOperation::Export {
        return Ok(());
    }
    for e in states {
        if e.locked {
            return Err(LayerError::Locked);
        }
        if !e.visible {
            return Err(LayerError::Hidden);
        }
    }
    if let LayerOperation::AssignLayer(id) = operation {
        let e = layer_state(id)?;
        if e.locked {
            return Err(LayerError::Locked);
        }
        if !e.visible {
            return Err(LayerError::Hidden);
        }
    }
    Ok(())
}
