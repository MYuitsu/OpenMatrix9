// SPDX-License-Identifier: LGPL-2.1-or-later
//! Controller bridge for the Project Manager Job Bags (OM9-L2-03-15).
use om9_core::{
    Error, Result,
    changes::ChangeCause,
    identity::ObjectId,
    model::Entity,
    transaction::{Mutation, TransactionRequest},
};

impl super::Controller {
    /// Clone the selected root objects for a Job Bag capture. An empty set is
    /// an explicit error: it must not create an empty "ghost" slot.
    pub fn jobbag_selected_entities(&self) -> Result<Vec<Entity>> {
        if self.selection.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        let mut entities = Vec::with_capacity(self.selection.len());
        for id in &self.selection {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("selected jobbag object"))?;
            entities.push(entity.clone());
        }
        Ok(entities)
    }

    /// Insert restored Job Bag entities as one transaction, so Undo removes the
    /// whole restore. IDs are minted by `ProjectManager::restore_entities`.
    pub fn insert_jobbag_entities(&mut self, entities: Vec<Entity>) -> Result<Vec<ObjectId>> {
        if entities.is_empty() {
            return Err(Error::Invalid("jobbag restore selection"));
        }
        let mut ids = Vec::with_capacity(entities.len());
        let mut mutations = Vec::with_capacity(entities.len());
        for entity in entities {
            if self.document().entities.contains_key(&entity.id) {
                return Err(Error::Invalid("jobbag restore id collision"));
            }
            ids.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        if self.scene.is_none() {
            return Err(Error::Missing("jobbag restore scene"));
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new("Jobbag Restore", mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(ChangeCause::Edit);
        }
        Ok(ids)
    }
}
