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
    /// Commit already restored BRep roots only after their candidate scene is
    /// ready. The caller owns the input storage represented by `held`; each
    /// candidate retains its own memory receipt. This does not decode a bag.
    #[cfg(feature = "occt-brep")]
    pub(crate) fn insert_restored_jobbag_breps(&mut self,
        inputs: Vec<(crate::brep_restore_host::ReservedBrepCandidate, Entity)>,
        mut held: om9_core::budget::Usage) -> Result<()> {
        if self.is_running() { return Err(Error::Busy); }
        if self.scene.is_none() { return Err(Error::Missing("jobbag restore scene")); }
        held.add(self.scene_cache_usage()?, self.coordinator.limits())?;
        let prepared = crate::brep_restore_host::prepare_brep_insertions(
            &self.coordinator, self.document, "Jobbag Restore", inputs, held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published { self.sync_published(ChangeCause::Edit); }
        Ok(())
    }

    #[cfg(all(feature = "occt-brep", feature = "test-fixtures"))]
    pub fn test_insert_restored_jobbag_breps(&mut self,
        inputs: Vec<(crate::brep_restore_host::ReservedBrepCandidate, Entity)>,
        held: om9_core::budget::Usage) -> Result<()> {
        self.insert_restored_jobbag_breps(inputs, held)
    }
    /// Reproject selected roots of a detached bag through the same validated
    /// clipboard closure/remapping path, then publish one paste transaction.
    pub fn insert_filtered_jobbag(&mut self, archive: &[u8], roots: &[ObjectId], snapshot: &crate::project_manager::Snapshot) -> Result<Vec<ObjectId>> {
        self.insert_filtered_jobbag_accounted(archive, roots, snapshot, false)
    }

    pub(crate) fn insert_filtered_jobbag_accounted(&mut self, archive: &[u8], roots: &[ObjectId], snapshot: &crate::project_manager::Snapshot, input_accounted: bool) -> Result<Vec<ObjectId>> {
        if roots.is_empty() { return Err(Error::Invalid("jobbag filter matches no objects")); }
        self.insert_filtered_jobbag_inner(archive, snapshot, input_accounted, Some(roots), None)
    }

    pub(crate) fn insert_jobbag_matching_filter(&mut self, archive: &[u8], snapshot: &crate::project_manager::Snapshot,
        filter: crate::project_manager::BagFilter, input_accounted: bool) -> Result<Vec<ObjectId>> {
        self.insert_filtered_jobbag_inner(archive, snapshot, input_accounted, None, Some(filter))
    }

    fn insert_filtered_jobbag_inner(&mut self, archive: &[u8], snapshot: &crate::project_manager::Snapshot,
        input_accounted: bool, requested: Option<&[ObjectId]>, filter: Option<crate::project_manager::BagFilter>) -> Result<Vec<ObjectId>> {
        let held = self.scene_cache_usage()?;
        let limits = *self.coordinator.limits();
        let (_decoded_reservation, document) = super::project_state::decode_project_archive_reserved(
            &self.coordinator, archive, held, None, input_accounted,
        )?;
        let _root_reservation = if filter.is_some() {
            let bytes = om9_core::budget::Usage::array::<ObjectId>(snapshot.captured_ids.len())?.bytes.max(1);
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            Some(self.coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, held)?)
        } else { None };
        let resolved;
        let roots = if let Some(filter) = filter {
            resolved = snapshot.filtered_ids_in_document(filter, &document, &limits)?;
            resolved.as_slice()
        } else {
            snapshot.validate_archive_roots(&document, &limits)?;
            requested.ok_or(Error::Invalid("jobbag filter roots"))?
        };
        if roots.is_empty() { return Err(Error::Invalid("jobbag filter matches no objects")); }
        // The temporary controller, filtered archive and paste coexist with the
        // decoded source. Admit their estimated working set before building them.
        let bytes = document.validate(&limits)?.bytes.checked_mul(4)
            .and_then(|size| size.checked_add(16 * 1024 * 1024))
            .ok_or(Error::Overflow("filtered jobbag admission"))?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let _working_reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, held,
        )?;
        let captured: std::collections::BTreeSet<_> = snapshot.captured_ids.iter().copied().collect();
        let mut unique = std::collections::BTreeSet::new();
        if roots.iter().any(|id| !captured.contains(id) || !unique.insert(*id)) {
            return Err(Error::Invalid("jobbag filter roots"));
        }
        let source = super::Controller::new(document, limits)?;
        let filtered = source.copy_jobbag_objects_om9(roots)?;
        self.paste_om9(&filtered)
    }
    /// Clone the selected root objects for a Job Bag capture. An empty set is
    /// an explicit error: it must not create an empty "ghost" slot.
    pub fn jobbag_selected_entities(&self) -> Result<Vec<Entity>> {
        if self.selection.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        let limits = self.coordinator.limits();
        let mut usage = om9_core::budget::Usage { objects: self.selection.len() as u64, ..Default::default() };
        usage.add(om9_core::budget::Usage::array::<Entity>(self.selection.len())?, limits)?;
        let mut entities = Vec::new();
        entities.try_reserve_exact(self.selection.len()).map_err(|_| Error::Budget("jobbag selection"))?;
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
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.sync_published(ChangeCause::Edit);
        }
        Ok(ids)
    }
}
