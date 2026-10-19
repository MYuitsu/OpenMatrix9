// SPDX-License-Identifier: LGPL-2.1-or-later
//! Document selection contract (OM9-L3-01-03-010).
//!
//! A selection is typed document identity, never a UI index: every entry is an
//! `ObjectId` (plus an optional typed subshape reference), and an entry whose
//! document, epoch or session generation no longer matches is `Stale`. A stale
//! index into an older list is never resolved as if it named a live object.
use crate::{Error, Result, identity::*, model::Document};
use std::collections::BTreeSet;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SubshapePart {
    Face,
    Edge,
    Vertex,
    Segment,
}

/// Stable reference to a part of one object. `index` is the part's position in
/// the owner's own ordered part list, never a screen or list position.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SubshapeRef {
    pub object: ObjectId,
    pub part: SubshapePart,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionState {
    pub document: DocumentId,
    pub epoch: u64,
    pub session_generation: u64,
    pub objects: BTreeSet<ObjectId>,
    pub subshapes: BTreeSet<SubshapeRef>,
    pub revision: u64,
}
impl SelectionState {
    pub fn new(document: &Document) -> Self {
        Self {
            document: document.id,
            epoch: document.epoch,
            session_generation: document.session_generation,
            objects: BTreeSet::new(),
            subshapes: BTreeSet::new(),
            revision: 1,
        }
    }
    /// Selection belongs to one live document generation. A different document
    /// or owner session makes the whole selection stale; a changed document
    /// epoch (another publication) is reported separately so a caller can tell a
    /// reload from a committed edit.
    pub fn validate(&self, document: &Document) -> Result<()> {
        if self.document != document.id || self.session_generation != document.session_generation {
            return Err(Error::Stale("selection document"));
        }
        if self.epoch != document.epoch {
            return Err(Error::Stale("selection epoch"));
        }
        if self.revision == 0 {
            return Err(Error::Invalid("selection revision"));
        }
        Ok(())
    }
    /// The canonical order a caller may hand to a list widget. Entries are
    /// always ordered by typed identity, so the same selection yields the same
    /// order after a reload or a sort.
    pub fn ordered(&self) -> Vec<ObjectId> {
        self.objects.iter().copied().collect()
    }
    /// Resolve a previously published list position. The position is only valid
    /// for the exact ordered list it was published from; a shortened, reordered
    /// or reloaded list is reported as stale instead of selecting a neighbour.
    pub fn resolve_index(&self, published: &[ObjectId], index: usize) -> Result<ObjectId> {
        if published != self.ordered().as_slice() {
            return Err(Error::Stale("selection index"));
        }
        published
            .get(index)
            .copied()
            .ok_or(Error::Stale("selection index"))
    }
    pub fn contains(&self, id: ObjectId) -> bool {
        self.objects.contains(&id)
    }
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.subshapes.is_empty()
    }
    pub fn select(&mut self, document: &Document, id: ObjectId) -> Result<()> {
        self.validate(document)?;
        if !document.entities.contains_key(&id) {
            return Err(Error::Missing("selection object"));
        }
        if self.objects.insert(id) {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or(Error::Overflow("selection revision"))?;
        }
        Ok(())
    }
    pub fn deselect(&mut self, id: ObjectId) {
        let removed = self.objects.remove(&id);
        let before = self.subshapes.len();
        self.subshapes.retain(|subshape| subshape.object != id);
        if removed || self.subshapes.len() != before {
            self.revision = self.revision.saturating_add(1);
        }
    }
    pub fn select_subshape(&mut self, document: &Document, reference: SubshapeRef) -> Result<()> {
        self.validate(document)?;
        if !document.entities.contains_key(&reference.object) {
            return Err(Error::Missing("selection object"));
        }
        if self.subshapes.insert(reference) {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or(Error::Overflow("selection revision"))?;
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        if !self.is_empty() {
            self.objects.clear();
            self.subshapes.clear();
            self.revision = self.revision.saturating_add(1);
        }
    }
    /// Drop every entry whose object no longer exists. Returns true when the
    /// selection changed, which is the caller's signal to refresh its view.
    pub fn prune(&mut self, document: &Document) -> bool {
        let before = (self.objects.len(), self.subshapes.len());
        self.objects.retain(|id| document.entities.contains_key(id));
        self.subshapes
            .retain(|subshape| document.entities.contains_key(&subshape.object));
        let after = (self.objects.len(), self.subshapes.len());
        if before != after {
            self.revision = self.revision.saturating_add(1);
            return true;
        }
        false
    }
}
