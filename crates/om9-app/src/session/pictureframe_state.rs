// SPDX-License-Identifier: LGPL-2.1-or-later
//! Picture-frame reference state embedded as an opaque `.om9` resource.
use om9_core::{
    Error, Result,
    resource::{Resource, ResourceKind},
    transaction::{Mutation, TransactionRequest},
};

/// The JSON manifest is versioned by [`crate::pictureframe::PictureFrameStore`].
pub const PICTUREFRAME_STATE_RESOURCE: &str = "om9.pictureframe.v1";

impl super::Controller {
    pub fn embedded_pictureframes(&self) -> Result<Option<Vec<u8>>> {
        let mut found: Option<Vec<u8>> = None;
        for resource in self.document().resources.values() {
            if resource.name != PICTUREFRAME_STATE_RESOURCE {
                continue;
            }
            if !matches!(resource.kind, ResourceKind::OpaqueBinary) {
                return Err(Error::Invalid("picture frame resource kind"));
            }
            if found.is_some() {
                return Err(Error::Invalid("duplicate picture frame resource"));
            }
            found = Some(resource.payload()?.to_vec());
        }
        Ok(found)
    }

    /// Replace the reference-image manifest without an Undo entry; called before Save.
    pub fn store_pictureframes(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > 256 * 1024 * 1024 {
            return Err(Error::Budget("picture frame state bytes"));
        }
        let limits = *self.coordinator.limits();
        let resource = Resource::from_bytes(
            PICTUREFRAME_STATE_RESOURCE,
            ResourceKind::OpaqueBinary,
            bytes,
            &limits,
        )?;
        let mut mutations = Vec::new();
        for old in self.document().resources.values() {
            if old.name == PICTUREFRAME_STATE_RESOURCE {
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
        let mut request = TransactionRequest::new("Picture Frames", mutations);
        request.recording = false;
        let prepared = self.coordinator.prepare(request)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }
}
