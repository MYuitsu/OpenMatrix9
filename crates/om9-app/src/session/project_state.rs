// SPDX-License-Identifier: LGPL-2.1-or-later
//! Project-manager state embedded as an opaque `.om9` resource (OM9-L2-03-15).
use om9_core::{
    Error, Result,
    budget::Usage,
    resource::{Resource, ResourceKind},
    transaction::{Mutation, TransactionRequest},
};

pub const PROJECT_STATE_RESOURCE: &str = "om9.project-manager.v1";

impl super::Controller {
    /// Read the project manager JSON embedded in the open document. A missing
    /// resource is `Ok(None)`; a malformed/foreign resource is an error.
    pub fn embedded_project_state(&self) -> Result<Option<Vec<u8>>> {
        let mut found: Option<Vec<u8>> = None;
        for resource in self.document().resources.values() {
            if resource.name != PROJECT_STATE_RESOURCE {
                continue;
            }
            if !matches!(resource.kind, ResourceKind::OpaqueBinary) {
                return Err(Error::Invalid("project state resource kind"));
            }
            if found.is_some() {
                return Err(Error::Invalid("duplicate project state resource"));
            }
            found = Some(resource.payload()?.to_vec());
        }
        Ok(found)
    }

    /// Replace the embedded project-manager resource without adding an Undo
    /// entry. This runs immediately before Save/Save As, so the archive contains
    /// the current rows/slots/notes.
    pub fn store_project_state(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::Budget("project state bytes"));
        }
        let mut existing = self
            .document()
            .resources
            .values()
            .filter(|r| r.name == PROJECT_STATE_RESOURCE);
        if let Some(old) = existing.next()
            && existing.next().is_none()
            && matches!(old.kind, ResourceKind::OpaqueBinary)
            && old.payload()? == bytes
        {
            return Ok(());
        }
        let limits = *self.coordinator.limits();
        let resource = Resource::from_bytes(
            PROJECT_STATE_RESOURCE,
            ResourceKind::OpaqueBinary,
            bytes,
            &limits,
        )?;
        let mut mutations = Vec::new();
        for old in self.document().resources.values() {
            if old.name == PROJECT_STATE_RESOURCE {
                mutations.push(Mutation::ResourceDelete {
                    document: self.document,
                    expected_revision: self.document().revision,
                    resource: old.id,
                });
            }
        }
        mutations.push(Mutation::ResourceInsert {
            document: self.document,
            expected_revision: self.document().revision,
            resource,
        });
        let mut request = TransactionRequest::new("Project State", mutations);
        request.recording = false;
        self.commit_ui_metadata(request)
    }

    /// Publish resource-only edits with the document's matching scene metadata.
    /// Geometry buffers stay shared; a failed admission keeps both old versions.
    pub(super) fn commit_ui_metadata(&mut self, request: TransactionRequest) -> Result<()> {
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("UI metadata scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("UI metadata scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let increment = old_scene.update_held_usage(None)?;
        held.add(increment, self.coordinator.limits())?;
        let prepared = self.coordinator.prepare_with_held_usage(request, held)?;
        let receipt = self.commit_incremental_scene(prepared, held, increment, None, false)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }

    /// Atomically replace the live document from a captured Master snapshot.
    /// Validation/construction of the candidate happens before the assignment;
    /// a failure leaves the current document untouched.
    pub fn restore_master_document(&mut self, document: om9_core::model::Document) -> Result<()> {
        let limits = *self.coordinator.limits();
        let held = self.replacement_hold(Usage::default())?;
        let auxiliary = self.scene_cache_usage()?;
        let initial = document.validation_peak_estimate(&limits, true)?.bytes.max(1);
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, initial, physical, auxiliary,
        )?;
        let (candidate, reservation) = self.build_reserved_document_replacement(document, held, auxiliary, reservation)?;
        self.publish_reserved_replacement(reservation, candidate, om9_core::changes::ChangeCause::Replace, Usage::default())
    }
}
