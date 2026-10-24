// SPDX-License-Identifier: LGPL-2.1-or-later
//! F6 custom menu persisted as an opaque `.om9` resource; the command IDs remain the record.
use om9_core::{
    Error, Result,
    resource::{Resource, ResourceKind},
    transaction::{Mutation, TransactionRequest},
};

pub const F6_STATE_RESOURCE: &str = "om9.f6.v1";

impl super::Controller {
    pub fn embedded_f6_state(&self) -> Result<Option<Vec<u8>>> {
        let mut found: Option<Vec<u8>> = None;
        for resource in self.document().resources.values() {
            if resource.name != F6_STATE_RESOURCE {
                continue;
            }
            if !matches!(resource.kind, ResourceKind::OpaqueBinary) {
                return Err(Error::Invalid("F6 state resource kind"));
            }
            if found.is_some() {
                return Err(Error::Invalid("duplicate F6 state resource"));
            }
            found = Some(resource.payload()?.to_vec());
        }
        Ok(found)
    }

    pub fn store_f6_state(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > 1024 * 1024 {
            return Err(Error::Budget("F6 state bytes"));
        }
        let limits = *self.coordinator.limits();
        let resource = Resource::from_bytes(
            F6_STATE_RESOURCE,
            ResourceKind::OpaqueBinary,
            bytes,
            &limits,
        )?;
        let mut mutations = Vec::new();
        for old in self.document().resources.values() {
            if old.name == F6_STATE_RESOURCE {
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
        let mut request = TransactionRequest::new("F6 State", mutations);
        request.recording = false;
        let prepared = self.coordinator.prepare(request)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }
}
