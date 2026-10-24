// SPDX-License-Identifier: LGPL-2.1-or-later
//! Display preferences embedded as an opaque `.om9` resource.
//!
//! The record is presentation state, so it is written without an Undo entry, exactly like the
//! project-manager record. It is versioned by [`crate::display::DisplayPreferences`].
use om9_core::{
    Error, Result,
    resource::{Resource, ResourceKind},
    transaction::{Mutation, TransactionRequest},
};

pub const DISPLAY_STATE_RESOURCE: &str = "om9.display.v1";

impl super::Controller {
    pub fn embedded_display_state(&self) -> Result<Option<Vec<u8>>> {
        let mut found: Option<Vec<u8>> = None;
        for resource in self.document().resources.values() {
            if resource.name != DISPLAY_STATE_RESOURCE {
                continue;
            }
            if !matches!(resource.kind, ResourceKind::OpaqueBinary) {
                return Err(Error::Invalid("display state resource kind"));
            }
            if found.is_some() {
                return Err(Error::Invalid("duplicate display state resource"));
            }
            found = Some(resource.payload()?.to_vec());
        }
        Ok(found)
    }

    /// Replace the display record without adding an Undo entry. Called immediately before Save
    /// so the archive carries the current wireframe/edge/gem/category preferences.
    pub fn store_display_state(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > 1024 * 1024 {
            return Err(Error::Budget("display state bytes"));
        }
        let limits = *self.coordinator.limits();
        let resource = Resource::from_bytes(
            DISPLAY_STATE_RESOURCE,
            ResourceKind::OpaqueBinary,
            bytes,
            &limits,
        )?;
        let mut mutations = Vec::new();
        for old in self.document().resources.values() {
            if old.name == DISPLAY_STATE_RESOURCE {
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
        let mut request = TransactionRequest::new("Display State", mutations);
        request.recording = false;
        let prepared = self.coordinator.prepare(request)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }
}
