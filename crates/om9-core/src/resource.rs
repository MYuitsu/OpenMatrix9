// SPDX-License-Identifier: LGPL-2.1-or-later
//! Embedded immutable resource ownership. A serialized descriptor is incomplete
//! until its exact payload has been admitted; data never chooses an executable.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    identity::ResourceId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResourceKind {
    OpaqueBinary,
    /// Declared original version. This alone is no native read/edit/export
    /// capability and is checked against actual native inventory at import.
    RetainedThreeDm {
        archive_version: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: ResourceId,
    pub name: String,
    pub kind: ResourceKind,
    byte_length: u64,
    sha256: [u8; 32],
    #[serde(skip)]
    payload: Option<Arc<[u8]>>,
}
impl Resource {
    pub fn from_bytes(
        name_: &str,
        kind: ResourceKind,
        bytes: &[u8],
        limits: &Limits,
    ) -> Result<Self> {
        name(name_, limits)?;
        let byte_length =
            u64::try_from(bytes.len()).map_err(|_| Error::Overflow("resource bytes"))?;
        let mut usage = Usage::array::<Self>(1)?;
        usage.add(
            Usage {
                bytes: name_.len() as u64,
                ..Usage::default()
            },
            limits,
        )?;
        usage.add(Self::copy_usage(byte_length)?, limits)?;
        Self::validate_kind(&kind)?;
        let mut name = String::new();
        name.try_reserve_exact(name_.len())
            .map_err(|_| Error::Budget("resource name allocation"))?;
        name.push_str(name_);
        Ok(Self {
            id: ResourceId::new(),
            name,
            kind,
            byte_length,
            sha256: Sha256::digest(bytes).into(),
            payload: Some(Arc::from(bytes)),
        })
    }
    fn validate_kind(kind: &ResourceKind) -> Result<()> {
        if matches!(kind, ResourceKind::RetainedThreeDm { archive_version: 0 }) {
            return Err(Error::Invalid("retained archive version"));
        }
        Ok(())
    }
    fn copy_usage(bytes: u64) -> Result<Usage> {
        Ok(Usage {
            bytes: bytes
                .checked_mul(2)
                .and_then(|n| n.checked_add((3 * std::mem::size_of::<usize>()) as u64))
                .ok_or(Error::Overflow("resource copy peak"))?,
            ..Usage::default()
        })
    }
    pub fn byte_length(&self) -> u64 {
        self.byte_length
    }
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn payload(&self) -> Result<&[u8]> {
        self.payload
            .as_deref()
            .ok_or(Error::Missing("embedded resource payload"))
    }
    /// Hydrate a descriptor only after matching its content and size. The
    /// caller additionally admits this copy with all retained document state.
    pub fn attach_payload(&mut self, bytes: &[u8], limits: &Limits) -> Result<()> {
        if self.payload.is_some() {
            return Err(Error::Invalid("resource already attached"));
        }
        self.manifest_usage(limits)?;
        Self::copy_usage(self.byte_length)?.check(limits)?;
        if bytes.len() as u64 != self.byte_length || Sha256::digest(bytes).as_slice() != self.sha256
        {
            return Err(Error::Invalid("resource size/hash mismatch"));
        }
        self.payload = Some(Arc::from(bytes));
        Ok(())
    }
    /// Prospective storage includes the as-yet-unallocated payload. This does
    /// not authorize a descriptor as a live runtime resource.
    pub fn manifest_usage(&self, limits: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("resource identity"));
        }
        name(&self.name, limits)?;
        Self::validate_kind(&self.kind)?;
        let usage = Usage {
            bytes: self
                .byte_length
                .checked_add(self.name.capacity() as u64)
                .and_then(|n| n.checked_add((3 * std::mem::size_of::<usize>()) as u64))
                .ok_or(Error::Overflow("resource storage"))?,
            nodes: 1,
            ..Usage::default()
        };
        usage.check(limits)?;
        Ok(usage)
    }
    pub fn validate(&self, limits: &Limits) -> Result<Usage> {
        let usage = self.manifest_usage(limits)?;
        if self.payload()?.len() as u64 != self.byte_length {
            return Err(Error::Invalid("resource payload length"));
        }
        // Payload and hash/length are private and immutable once attached.
        // Creation/hydration already checked SHA; don't rehash unchanged bytes
        // on every transaction or render admission.
        Ok(usage)
    }
}
