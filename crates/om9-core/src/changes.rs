// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ordered document change batches (OM9-L3-01-03-011).
//!
//! One accepted publication produces at most one batch, with a deterministic
//! order: the current layer, the layer order, layer records, entities, groups,
//! then features. Inside one family, removals are reported before creations and
//! edits, and each list follows typed identity order, so the same pair of
//! revisions always yields the same batch. An idle document produces no batch at
//! all, so a viewport is never asked to redraw for a change that did not happen.
use crate::{Error, Result, identity::*, model::Document};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    CurrentLayerChanged(Option<LayerId>),
    LayerOrderChanged,
    LayerCreated(LayerId),
    LayerModified(LayerId),
    LayerRemoved(LayerId),
    EntityAdded(ObjectId),
    EntityRemoved(ObjectId),
    EntityModified(ObjectId),
    ExtensionChanged(ObjectId),
    GroupCreated(GroupId),
    GroupModified(GroupId),
    GroupRemoved(GroupId),
    FeatureAdded(FeatureId),
    FeatureRemoved(FeatureId),
    FeatureModified(FeatureId),
}

/// Why a batch exists. A caller may treat Undo/Redo exactly like an edit but
/// must not treat a document replacement as an incremental change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeCause {
    Edit,
    Undo,
    Redo,
    Replace,
    Restore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeBatch {
    pub document: DocumentId,
    pub from_revision: u64,
    pub to_revision: u64,
    pub cause: ChangeCause,
    pub changes: Vec<Change>,
}
impl ChangeBatch {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
    /// Diff two revisions of one document. `None` means nothing changed, which
    /// is not the same as an empty batch reported for a real edit.
    pub fn between(
        before: &Document,
        after: &Document,
        cause: ChangeCause,
    ) -> Result<Option<Self>> {
        let changes = diff(before, after)?;
        if changes.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            document: after.id,
            from_revision: before.revision,
            to_revision: after.revision,
            cause,
            changes,
        }))
    }
}

fn entity_changed(before: &crate::model::Entity, after: &crate::model::Entity) -> (bool, bool) {
    if before == after {
        return (false, false);
    }
    // Field-wise comparison, never a clone: a recording controller diffs every
    // entity on every publication, and cloning labels/geometry there would cost
    // O(document bytes) per publication.
    let extensions = before.extensions != after.extensions;
    let modified = before.id != after.id
        || before.label != after.label
        || before.generation != after.generation
        || before.geometry_revision != after.geometry_revision
        || before.attributes_revision != after.attributes_revision
        || before.geometry != after.geometry
        || before.placement != after.placement
        || before.attributes != after.attributes;
    (modified, extensions)
}

/// Deterministic, allocation-bounded diff of two revisions of one document.
pub fn diff(before: &Document, after: &Document) -> Result<Vec<Change>> {
    if before.id != after.id {
        return Err(Error::Invalid("change document"));
    }
    let mut changes = Vec::new();
    if before.current_layer != after.current_layer {
        changes.push(Change::CurrentLayerChanged(after.current_layer));
    }
    if before.layer_order != after.layer_order {
        changes.push(Change::LayerOrderChanged);
    }
    for id in before.layers.keys() {
        if !after.layers.contains_key(id) {
            changes.push(Change::LayerRemoved(*id));
        }
    }
    for (id, layer) in &after.layers {
        match before.layers.get(id) {
            None => changes.push(Change::LayerCreated(*id)),
            Some(previous) if previous != layer => changes.push(Change::LayerModified(*id)),
            Some(_) => {}
        }
    }
    for id in before.entities.keys() {
        if !after.entities.contains_key(id) {
            changes.push(Change::EntityRemoved(*id));
        }
    }
    for (id, entity) in &after.entities {
        match before.entities.get(id) {
            None => changes.push(Change::EntityAdded(*id)),
            Some(previous) => {
                let (modified, extensions) = entity_changed(previous, entity);
                if modified {
                    changes.push(Change::EntityModified(*id));
                }
                if extensions {
                    changes.push(Change::ExtensionChanged(*id));
                }
            }
        }
    }
    for id in before.groups.keys() {
        if !after.groups.contains_key(id) {
            changes.push(Change::GroupRemoved(*id));
        }
    }
    for (id, group) in &after.groups {
        match before.groups.get(id) {
            None => changes.push(Change::GroupCreated(*id)),
            Some(previous) if previous != group => changes.push(Change::GroupModified(*id)),
            Some(_) => {}
        }
    }
    for id in before.features.keys() {
        if !after.features.contains_key(id) {
            changes.push(Change::FeatureRemoved(*id));
        }
    }
    for (id, feature) in &after.features {
        match before.features.get(id) {
            None => changes.push(Change::FeatureAdded(*id)),
            Some(previous) if previous != feature => changes.push(Change::FeatureModified(*id)),
            Some(_) => {}
        }
    }
    Ok(changes)
}
