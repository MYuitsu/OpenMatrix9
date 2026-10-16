//! The contours of a document: four structures that have to agree, and are therefore one.
//!
//! `contours`, `contour_ids`, `contour_parent` and `contour_ents` used to sit side by side in the document, with
//! their agreement resting on discipline. Discipline did not hold: removing a contour was written in three
//! places and each cleaned its own subset — `remove_contour` and the removal of a sketch left orphaned records
//! in `ents` and `parent`, and the two places that do clean them do not agree with each other. The orphans
//! accumulated in the document and travelled into the file.
//!
//! Removing a contour without touching everything else is now impossible: the lists are private and there is one
//! way in. Reading is unaffected — a `Deref` to a slice keeps `p.contours[i]`, `.len()` and `.iter()` as they
//! were.

use serde::{Deserialize, Serialize};

use crate::geom::Contour;
use crate::model::Id;

type Map<K, V> = std::collections::HashMap<K, V>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Contours {
    list: Vec<Contour>,
    /// the stable id of each contour, parallel to `list`, always of the same length
    ids: Vec<Id>,
    /// contour id to the entities of its boundary: the provenance, used for matching on the next edit
    ents: Map<Id, Vec<Id>>,
    /// contour id to the id of the enclosing contour: the nesting
    parent: Map<Id, Id>,
    /// THE PLACE OF EACH ID in `ids`, kept by every door that moves an id; empty for a set read some other way, which
    /// is then looked along (`index_of`). Looked along, every place asked for in a rebuild of 17 500 loops was 1.5e8
    /// steps.
    #[serde(skip)]
    at: Map<Id, usize>,
}

impl std::ops::Deref for Contours {
    type Target = [Contour];
    fn deref(&self) -> &[Contour] {
        &self.list
    }
}

/// THE FOUR LISTS A CONTOUR SET IS MADE OF, borrowed for writing to a file.
///
/// It used to be a four-place tuple, and the file writer read it as `parts.0`, `parts.1`, `parts.2` -
/// numbers that say nothing about which list is which, in the one place where getting them the wrong way
/// round would write a broken document.
pub(crate) struct ContourParts<'a> {
    /// The contours themselves, in order.
    pub list: &'a [Contour],
    /// Their stable ids, parallel to `list`.
    pub ids: &'a [Id],
    /// Which entities each contour is made of.
    pub ents: &'a Map<Id, Vec<Id>>,
    /// Which contour each one sits inside, for the nesting.
    pub parent: &'a Map<Id, Id>,
}

impl Contours {
    /// Assemble from the flat lists when reading a file. The lengths are levelled: a contour without an id
    /// means a damaged file, and it is better lost here than spread further through the document.
    pub(crate) fn from_parts(list: Vec<Contour>, ids: Vec<Id>, ents: Map<Id, Vec<Id>>, parent: Map<Id, Id>) -> Self {
        let n = list.len().min(ids.len());
        let ids: Vec<Id> = ids.into_iter().take(n).collect();
        let at = ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        Self { list: list.into_iter().take(n).collect(), ids, ents, parent, at }
    }

    pub(crate) fn parts(&self) -> ContourParts<'_> {
        ContourParts { list: &self.list, ids: &self.ids, ents: &self.ents, parent: &self.parent }
    }

    pub fn ids(&self) -> &[Id] {
        &self.ids
    }

    /// The index for a stable id.
    pub fn index_of(&self, id: Id) -> Option<usize> {
        if self.at.len() == self.ids.len() {
            return self.at.get(&id).copied();
        }
        self.ids.iter().position(|x| *x == id)
    }

    pub fn id_at(&self, index: usize) -> Option<Id> {
        self.ids.get(index).copied()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Contour> {
        self.list.get_mut(index)
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Contour> {
        self.list.iter_mut()
    }

    /// Add a contour with an id already allocated.
    pub fn push(&mut self, id: Id, c: Contour) {
        self.list.push(c);
        if self.at.len() == self.ids.len() {
            self.at.insert(id, self.ids.len());
        }
        self.ids.push(id);
    }

    /// Remove a contour by index, together with its provenance and its nesting.
    ///
    /// Returns the id of the removed contour so the caller can detach its own references.
    pub fn remove_at(&mut self, index: usize) -> Option<Id> {
        if index >= self.list.len() {
            return None;
        }
        let id = self.ids[index];
        self.list.remove(index);
        self.ids.remove(index);
        if self.at.len() == self.ids.len() + 1 {
            self.at.remove(&id);
            for (i, later) in self.ids.iter().enumerate().skip(index) {
                self.at.insert(*later, i);
            }
        }
        self.ents.remove(&id);
        self.parent.remove(&id);
        // the children of the removed contour are no longer nested in anything
        self.parent.retain(|_, p| *p != id);
        Some(id)
    }

    /// REMOVE THE CONTOURS `gone` in one pass, each with its provenance and its nesting, as `remove_at` removes them one
    /// by one: removed one by one, each shifted the whole list and the places of all after it.
    pub fn remove_ids(&mut self, gone: &std::collections::HashSet<Id>) {
        if gone.is_empty() {
            return;
        }
        let keep: Vec<bool> = self.ids.iter().map(|id| !gone.contains(id)).collect();
        let mut k = 0;
        self.list.retain(|_| {
            k += 1;
            keep[k - 1]
        });
        self.ids.retain(|id| !gone.contains(id));
        self.at = self.ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        for id in gone {
            self.ents.remove(id);
            self.parent.remove(id);
        }
        // the children of a removed contour are no longer nested in anything
        self.parent.retain(|_, p| !gone.contains(p));
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.ids.clear();
        self.at.clear();
        self.ents.clear();
        self.parent.clear();
    }

    // --- provenance: which sketch entities form the boundary ---
    pub fn ents_of(&self, id: Id) -> Option<&Vec<Id>> {
        self.ents.get(&id)
    }
    pub fn set_ents(&mut self, id: Id, e: Vec<Id>) {
        self.ents.insert(id, e);
    }
    pub fn clear_ents(&mut self, id: Id) {
        self.ents.remove(&id);
    }

    // --- nesting ---
    pub fn parent_of(&self, id: Id) -> Option<Id> {
        self.parent.get(&id).copied()
    }
    pub fn set_parent(&mut self, id: Id, parent: Id) {
        self.parent.insert(id, parent);
    }
    pub fn clear_parent(&mut self, id: Id) {
        self.parent.remove(&id);
    }
    pub fn children_of(&self, id: Id) -> Vec<Id> {
        self.parent.iter().filter(|(_, p)| **p == id).map(|(c, _)| *c).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Contours;
    use crate::geom::Contour;

    #[test]
    fn the_place_of_an_id_is_where_a_look_along_the_list_finds_it() {
        let mut set = Contours::default();
        let mut seed: u64 = 5;
        let mut next = |n: u64| {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % n
        };
        let mut id = 0;
        for _ in 0..2_000 {
            if set.ids().is_empty() || next(3) > 0 {
                id += 1;
                set.push(id, Contour::closed(Vec::new()));
            } else {
                let at = next(set.ids().len() as u64) as usize;
                set.remove_at(at);
            }
            for probe in [1, id / 2, id, id + 1] {
                assert_eq!(set.index_of(probe), set.ids().iter().position(|x| *x == probe), "the place of {probe}");
            }
        }
        // a set read past the doors that keep the table is looked along
        let read: Contours = ron::from_str(&ron::to_string(&set).expect("written")).expect("read");
        for &probe in set.ids().iter().take(50) {
            assert_eq!(read.index_of(probe), set.index_of(probe), "the place of {probe} in a set read back");
        }
    }
}
