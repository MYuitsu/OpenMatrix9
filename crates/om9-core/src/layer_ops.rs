// SPDX-License-Identifier: LGPL-2.1-or-later
//! Layer clipboard, exchange and codec (M01 layer owner, wave W5).
//!
//! Ported from the legacy bridge (`rust/src/{layer_clipboard,layer_exchange,layer_codec,
//! layer_retained}.rs`) into safe Rust. The legacy C ABI, the Qt clipboard formats and the
//! host layer list stay behind; the portable part — what a layer copy contains, how ids are
//! remapped on paste, how contents exchange between layers, and the versioned wire form —
//! is re-implemented here on the target's own records.
//!
//! Provenance note: a copied entity keeps its geometry, attributes and placement, but the
//! import namespace/retained source records stay with the original object. A copy therefore
//! never claims to be the imported object (the L2-01-03 rule: a chain is provenance only when
//! it is complete).
use crate::{
    Error, Result,
    budget::Limits,
    identity::{LayerId, ObjectId},
    model::{Document, Entity, Layer},
    transaction::Mutation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The wire schema of `LayerClipboard`. A payload from another schema is refused rather than
/// silently reinterpreted.
pub const CLIPBOARD_SCHEMA: u32 = 1;

/// What one layer copy contains: the layer, its descendant sublayers and the entities that
/// live on them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerClipboard {
    pub schema: u32,
    pub source: LayerId,
    pub layers: Vec<Layer>,
    pub entities: Vec<Entity>,
}

/// The result of planning a paste: layers to create (through the layer plan) plus the entity
/// inserts. Nothing is applied here, so the caller can create the layers first and commit one
/// transaction for the entities.
#[derive(Debug, Clone)]
pub struct PastePlan {
    /// New layers, parents already remapped, in parent-before-child order.
    pub layers: Vec<Layer>,
    /// One `Insert` per copied entity, in clipboard order.
    pub entities: Vec<Mutation>,
}

/// Collect the layer, every descendant sublayer, and every entity on them.
pub fn copy_layer(document: &Document, layer: LayerId, limits: &Limits) -> Result<LayerClipboard> {
    if !document.layers.contains_key(&layer) {
        return Err(Error::Missing("layer"));
    }
    // Descendants: walk children until nothing new is found, so a deep tree is copied whole.
    let mut selected = BTreeSet::new();
    selected.insert(layer);
    loop {
        let mut added = false;
        for (id, candidate) in &document.layers {
            if let Some(parent) = candidate.parent
                && selected.contains(&parent)
                && selected.insert(*id)
            {
                added = true;
            }
        }
        if !added {
            break;
        }
        if selected.len() > limits.max_depth.saturating_mul(limits.max_depth) {
            return Err(Error::Overflow("layer subtree"));
        }
    }
    if selected.len() as u64 > limits.max_objects {
        return Err(Error::Budget("layer clipboard"));
    }
    // Deterministic order: the source layer first, then the rest by name so a clipboard is
    // stable across runs.
    let mut layers = Vec::new();
    layers
        .try_reserve_exact(selected.len())
        .map_err(|_| Error::Budget("layer clipboard"))?;
    let source = document.layers[&layer].clone();
    layers.push(source);
    for (id, candidate) in &document.layers {
        if selected.contains(id) && *id != layer {
            layers.push(candidate.clone());
        }
    }
    let mut entities = Vec::new();
    entities
        .try_reserve_exact(document.entities.len().min(1024))
        .map_err(|_| Error::Budget("layer clipboard"))?;
    for entity in document.entities.values() {
        if let Some(owner) = entity.attributes.layer
            && selected.contains(&owner)
        {
            entities.push(entity.clone());
        }
    }
    if entities.len() as u64 > limits.max_outputs {
        return Err(Error::Budget("layer clipboard"));
    }
    entities.sort_by_key(|entity| entity.id);
    Ok(LayerClipboard {
        schema: CLIPBOARD_SCHEMA,
        source: layer,
        layers,
        entities,
    })
}

/// Plan a paste. Every layer gets a fresh id and every entity a fresh id, so pasting twice
/// produces two independent copies; when `target` is given the copied root layer is attached
/// under it, otherwise the pasted root stays a top-level layer.
pub fn paste_layer(
    document: &Document,
    clipboard: &LayerClipboard,
    target: Option<LayerId>,
    _limits: &Limits,
) -> Result<PastePlan> {
    if clipboard.schema != CLIPBOARD_SCHEMA {
        return Err(Error::Unsupported("layer clipboard schema"));
    }
    if let Some(target) = target
        && !document.layers.contains_key(&target)
    {
        return Err(Error::Missing("layer"));
    }
    if clipboard.layers.is_empty() {
        return Err(Error::Invalid("layer clipboard payload"));
    }
    // Remap the layer ids first, keeping the clipboard order (source first, then children).
    let mut remap: BTreeMap<LayerId, LayerId> = BTreeMap::new();
    for layer in &clipboard.layers {
        remap.insert(layer.id, LayerId::new());
    }
    let mut layers = Vec::new();
    layers
        .try_reserve_exact(clipboard.layers.len())
        .map_err(|_| Error::Budget("layer paste"))?;
    for layer in &clipboard.layers {
        let mut copy = layer.clone();
        copy.id = remap[&layer.id];
        copy.revision = 1;
        let parent = if layer.id == clipboard.source {
            target
        } else {
            layer.parent.map(|parent| {
                remap
                    .get(&parent)
                    .copied()
                    .unwrap_or_else(|| remap[&clipboard.source])
            })
        };
        copy.parent = parent;
        // A pasted layer must not collide with an existing name; the layer plan enforces the
        // final spelling, so the kernel only guarantees unique ids inside the payload.
        layers.push(copy);
    }
    let mut mutations = Vec::new();
    mutations
        .try_reserve_exact(clipboard.entities.len())
        .map_err(|_| Error::Budget("layer paste"))?;
    for entity in &clipboard.entities {
        let mut copy = entity.clone();
        copy.id = ObjectId::new();
        copy.attributes.layer = entity
            .attributes
            .layer
            .and_then(|owner| remap.get(&owner).copied());
        mutations.push(Mutation::Insert {
            document: document.id,
            entity: copy,
        });
    }
    Ok(PastePlan {
        layers,
        entities: mutations,
    })
}

/// Move every entity from `from` to `to` (layer exchange). The layer records themselves stay
/// untouched, so an exchange is a membership change and nothing else.
pub fn exchange_contents(
    document: &Document,
    from: LayerId,
    to: LayerId,
    limits: &Limits,
) -> Result<Vec<Mutation>> {
    if from == to {
        return Err(Error::Invalid("layer exchange"));
    }
    for layer in [from, to] {
        if !document.layers.contains_key(&layer) {
            return Err(Error::Missing("layer"));
        }
    }
    let mut mutations = Vec::new();
    for entity in document.entities.values() {
        if entity.attributes.layer == Some(from) {
            mutations.push(Mutation::Move {
                source: document.stamp(entity.id)?,
                destination: document.id,
                layer: Some(to),
            });
        }
    }
    if mutations.len() as u64 > limits.max_outputs {
        return Err(Error::Budget("layer exchange"));
    }
    Ok(mutations)
}

/// Encode a clipboard to its wire bytes with the document's byte budget.
pub fn encode(clipboard: &LayerClipboard, limits: &Limits) -> Result<Vec<u8>> {
    if clipboard.schema != CLIPBOARD_SCHEMA {
        return Err(Error::Unsupported("layer clipboard schema"));
    }
    let bytes =
        serde_json::to_vec(clipboard).map_err(|_| Error::Invalid("layer clipboard wire"))?;
    if bytes.len() as u64 > limits.max_bytes {
        return Err(Error::Budget("layer clipboard bytes"));
    }
    Ok(bytes)
}

/// Decode clipboard bytes. The size is checked before parsing so a hostile payload cannot
/// make the parser allocate first.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<LayerClipboard> {
    if bytes.len() as u64 > limits.max_bytes {
        return Err(Error::Budget("layer clipboard bytes"));
    }
    let clipboard: LayerClipboard =
        serde_json::from_slice(bytes).map_err(|_| Error::Invalid("layer clipboard wire"))?;
    if clipboard.schema != CLIPBOARD_SCHEMA {
        return Err(Error::Unsupported("layer clipboard schema"));
    }
    Ok(clipboard)
}
