// SPDX-License-Identifier: LGPL-2.1-or-later
//! Import source provenance (OM9-L3-01-03-013).
//!
//! A document object may only be reported as a native import when the retained
//! source proof for it is complete: a namespace record bound to that object, a
//! retained source whose embedded resource hash agrees, and a declared archive
//! version. Anything less is reported as having no provenance, never as a
//! `.3dm`-native object.
use crate::{
    Error, Result,
    identity::*,
    model::Document,
    resource::ResourceKind,
    retained::{HostRef, ImportMode, RecordDisposition},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub object: ObjectId,
    pub source: SourceId,
    pub resource: ResourceId,
    /// Original resource name as stored with the embedded source (source path
    /// or import file name).
    pub source_name: String,
    pub source_sha256: [u8; 32],
    pub archive_version: u32,
    pub namespace: ImportNamespaceId,
    pub import_mode: Option<ImportMode>,
    pub kind: crate::retained::NativeRecordKind,
    pub original_uuid: uuid::Uuid,
    pub original_index: Option<i32>,
    pub original_family: String,
    pub disposition: RecordDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportSummary {
    pub sources: usize,
    pub namespaces: usize,
    pub records: usize,
    pub bound: usize,
    pub retained_only: usize,
    pub deleted: usize,
    /// Records that could not be represented in this document and are kept as
    /// retained native data only. Reported, never dropped silently.
    pub unsupported: usize,
    pub objects_with_provenance: usize,
}
impl ImportSummary {
    pub fn is_empty(&self) -> bool {
        self.records == 0 && self.sources == 0 && self.namespaces == 0
    }
}

impl Document {
    /// Complete source proof for one object, or `None` when any part of the
    /// chain is missing. The chain is never inferred from entity data.
    pub fn provenance(&self, id: ObjectId) -> Option<Provenance> {
        for namespace in self.import_namespaces.values() {
            // An incomplete namespace is skipped, never allowed to mask a
            // later namespace that does carry the full proof.
            let Some(source) = self.retained_sources.get(&namespace.source) else {
                continue;
            };
            let Some(resource) = self.resources.get(&source.resource) else {
                continue;
            };
            if resource.sha256() != source.sha256
                || resource.kind
                    != (ResourceKind::RetainedThreeDm {
                        archive_version: source.archive_version,
                    })
            {
                continue;
            }
            for record in &namespace.records {
                let bound = match record.host {
                    Some(HostRef::Object(object)) => object == id,
                    Some(HostRef::DefinitionMember { object, .. }) => object == id,
                    Some(HostRef::Definition(_) | HostRef::Layer(_)) | None => false,
                };
                if !bound {
                    continue;
                }
                return Some(Provenance {
                    object: id,
                    source: source.id,
                    resource: resource.id,
                    source_name: resource.name.clone(),
                    source_sha256: source.sha256,
                    archive_version: source.archive_version,
                    namespace: namespace.id,
                    import_mode: namespace.import_mode,
                    kind: record.key.kind,
                    original_uuid: record.key.id,
                    original_index: record.key.native_index,
                    original_family: record.family.clone(),
                    disposition: record.disposition,
                });
            }
        }
        None
    }
    pub fn has_source_proof(&self, id: ObjectId) -> bool {
        self.provenance(id).is_some()
    }
    pub fn import_summary(&self) -> ImportSummary {
        let mut summary = ImportSummary {
            sources: self.retained_sources.len(),
            namespaces: self.import_namespaces.len(),
            ..ImportSummary::default()
        };
        // One pass builds the proven-object set; asking `provenance()` per entity
        // would rescan every namespace and record for every object (quadratic in
        // documents that hold many imported parts).
        let mut proven = std::collections::BTreeSet::new();
        for namespace in self.import_namespaces.values() {
            accumulate(&mut summary, namespace);
            let Some(source) = self.retained_sources.get(&namespace.source) else {
                continue;
            };
            let Some(resource) = self.resources.get(&source.resource) else {
                continue;
            };
            if resource.sha256() != source.sha256
                || resource.kind
                    != (ResourceKind::RetainedThreeDm {
                        archive_version: source.archive_version,
                    })
            {
                continue;
            }
            for record in &namespace.records {
                match record.host {
                    Some(HostRef::Object(object))
                    | Some(HostRef::DefinitionMember { object, .. }) => {
                        if self.entities.contains_key(&object) {
                            proven.insert(object);
                        }
                    }
                    Some(HostRef::Definition(_) | HostRef::Layer(_)) | None => {}
                }
            }
        }
        summary.objects_with_provenance = proven.len();
        summary
    }
    /// One namespace's counts. An unknown namespace is `Missing`, never an empty
    /// summary that would look like a successful import of nothing.
    pub fn namespace_summary(&self, namespace: ImportNamespaceId) -> Result<ImportSummary> {
        let namespace = self
            .import_namespaces
            .get(&namespace)
            .ok_or(Error::Missing("import namespace"))?;
        let mut summary = ImportSummary {
            sources: usize::from(self.retained_sources.contains_key(&namespace.source)),
            namespaces: 1,
            ..ImportSummary::default()
        };
        accumulate(&mut summary, namespace);
        Ok(summary)
    }
}

fn accumulate(summary: &mut ImportSummary, namespace: &crate::retained::ImportNamespace) {
    summary.records = summary.records.saturating_add(namespace.records.len());
    for record in &namespace.records {
        match record.disposition {
            RecordDisposition::Deleted => {
                summary.deleted = summary.deleted.saturating_add(1);
            }
            RecordDisposition::RetainedOnly => {
                summary.retained_only = summary.retained_only.saturating_add(1);
                if record.host.is_none() {
                    summary.unsupported = summary.unsupported.saturating_add(1);
                }
            }
            RecordDisposition::Unchanged | RecordDisposition::Edited => {
                summary.bound = summary.bound.saturating_add(1);
            }
        }
    }
}
