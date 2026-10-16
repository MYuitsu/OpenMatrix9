// SPDX-License-Identifier: LGPL-2.1-or-later
//! Trusted build/test receipts govern availability. Archive metadata never
//! registers or enables runtime backends. Receipts are produced by validation.
use crate::{Error, Result, geometry::GeometryKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backend {
    Rust {
        version: String,
    },
    Native {
        library: String,
        version: String,
        abi: String,
        toolkits: Vec<String>,
        closure_verified: bool,
        concurrent: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptScope {
    SourceAudit,
    Semantic,
    NativeOperation,
    StandaloneRuntime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub target: String,
    pub source_sha256: String,
    pub binary_sha256: String,
    pub suite: String,
    pub scope: ReceiptScope,
    pub passed: bool,
}
impl Receipt {
    fn usable(&self, scope: ReceiptScope) -> bool {
        let hash = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
        self.scope == scope
            && self.passed
            && self.target == "OM9Standalone"
            && !self.suite.is_empty()
            && hash(&self.source_sha256)
            && hash(&self.binary_sha256)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: String,
    pub feature: String,
    pub input_types: Vec<GeometryKind>,
    pub options: Vec<String>,
    pub backend: Backend,
    pub prepare: bool,
    pub preview: bool,
    pub commit: bool,
    pub receipts: Vec<Receipt>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Prepare,
    Preview,
    Commit,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Availability {
    pub enabled: bool,
    pub reason: String,
}
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    operations: BTreeMap<String, Operation>,
}
impl CapabilityRegistry {
    pub fn register(&mut self, operation: Operation) -> Result<()> {
        if operation.id.is_empty()
            || operation.feature.is_empty()
            || operation.input_types.is_empty()
            || self.operations.contains_key(&operation.id)
        {
            return Err(Error::Invalid("operation registration"));
        }
        let mut names = BTreeSet::new();
        if operation
            .options
            .iter()
            .any(|s| s.is_empty() || !names.insert(s))
        {
            return Err(Error::Invalid("operation options"));
        }
        self.operations.insert(operation.id.clone(), operation);
        Ok(())
    }
    pub fn availability(&self, id: &str) -> Availability {
        let Some(operation) = self.operations.get(id) else {
            return Availability {
                enabled: false,
                reason: "Operation has no standalone implementation".into(),
            };
        };
        let has = |scope| operation.receipts.iter().any(|r| r.usable(scope));
        let failure = match &operation.backend {
            Backend::Rust { version } => {
                if version.is_empty()
                    || !has(ReceiptScope::Semantic) && !has(ReceiptScope::StandaloneRuntime)
                {
                    Some("Rust semantic receipt is missing")
                } else {
                    None
                }
            }
            Backend::Native {
                library,
                version,
                abi,
                toolkits,
                closure_verified,
                ..
            } => {
                if library.is_empty()
                    || version.is_empty()
                    || abi.is_empty()
                    || toolkits.is_empty()
                    || !closure_verified
                {
                    Some("Native ABI/dependency closure has not been verified")
                } else if !has(ReceiptScope::NativeOperation) {
                    Some("Native operation receipt is missing")
                } else {
                    None
                }
            }
        };
        let failure = failure.or_else(|| {
            if operation.commit && !has(ReceiptScope::StandaloneRuntime) {
                Some("Standalone commit/runtime receipt is missing")
            } else {
                None
            }
        });
        match failure {
            Some(reason) => Availability {
                enabled: false,
                reason: reason.into(),
            },
            None => Availability {
                enabled: true,
                reason: String::new(),
            },
        }
    }
    pub fn admit(
        &self,
        id: &str,
        kind: GeometryKind,
        options: &[&str],
        phase: Phase,
    ) -> Result<&Operation> {
        if !self.availability(id).enabled {
            return Err(Error::Unsupported("operation evidence gate"));
        }
        let operation = self.operations.get(id).ok_or(Error::Missing("operation"))?;
        if !operation.input_types.contains(&kind)
            || options
                .iter()
                .any(|s| !operation.options.iter().any(|option| option == s))
        {
            return Err(Error::Unsupported("operation representation/options"));
        }
        if !match phase {
            Phase::Prepare => operation.prepare,
            Phase::Preview => operation.preview,
            Phase::Commit => operation.commit,
        } {
            return Err(Error::Unsupported("operation phase"));
        }
        Ok(operation)
    }
}
