// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rust-authored current-state actions. This is not a native write receipt.
use crate::ExportCapture;
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::SourceStamp,
    model::{Definition, Document, Entity, Layer},
    retained::{
        EntityBaseline, HostRef, ImportMode, ImportNamespace, NativeRecordKey, RecordDisposition,
        RetainedSource, SourceRecord,
    },
    transaction::{Cancellation, TransactionCoordinator},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordAction {
    Retain,
    Delete,
    Omit,
    Regenerate,
    PatchEntity,
    PatchLayer,
    PatchDefinition,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChangedFields {
    pub geometry: bool,
    pub placement: bool,
    pub attributes: bool,
    pub label: bool,
}
#[derive(Debug, Clone, Copy)]
pub enum HostValue<'a> {
    Entity { host: HostRef, entity: &'a Entity },
    Layer(&'a Layer),
    Definition(&'a Definition),
}
impl HostValue<'_> {
    pub fn host(&self) -> HostRef {
        match self {
            Self::Entity { host, .. } => *host,
            Self::Layer(layer) => HostRef::Layer(layer.id),
            Self::Definition(definition) => HostRef::Definition(definition.id),
        }
    }
}
#[derive(Debug)]
pub struct RecordOverlay<'a> {
    pub source: &'a SourceRecord,
    pub action: RecordAction,
    pub changed: ChangedFields,
    pub current: Option<HostValue<'a>>,
    /// A changed definition dependency forbids raw replay of its cached bounds,
    /// even when this record's own host fields are unchanged or retained-only.
    pub needs_cache_refresh: bool,
}
#[derive(Debug)]
pub struct NamespaceOverlay<'a> {
    pub namespace: &'a ImportNamespace,
    pub source: &'a RetainedSource,
    pub mode: ImportMode,
    pub records: Vec<RecordOverlay<'a>>,
}
#[derive(Debug)]
pub struct CurrentOverlay<'a> {
    pub namespaces: Vec<NamespaceOverlay<'a>>,
    pub additions: Vec<HostValue<'a>>,
    pub owned_usage: Usage,
}
impl ExportCapture {
    /// Borrows immutable captured state; no source geometry or payload is cloned.
    pub fn current_overlay<'a>(
        &'a self,
        coordinator: &TransactionCoordinator,
        cancel: &Cancellation,
    ) -> Result<CurrentOverlay<'a>> {
        self.current_overlay_with_held_usage(coordinator, cancel, Usage::default())
    }
    /// Additional owners (for example a native Job reservation) stay live with
    /// the capture and validation scratch. Admission precedes scratch allocation.
    pub fn current_overlay_with_held_usage<'a>(
        &'a self,
        coordinator: &TransactionCoordinator,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<CurrentOverlay<'a>> {
        canceled(cancel)?;
        self.revalidate(coordinator)?;
        let d = self.document();
        let limits = coordinator.limits();
        external.add(
            Usage::array::<SourceStamp>(self.sources.capacity())?,
            limits,
        )?;
        let mut held = coordinator.retained_usage_with_held_usage(external)?;
        let (records, current_count) = counts(d)?;
        let mut owned = Usage::array::<CurrentOverlay>(1)?;
        owned.add(
            Usage::array::<NamespaceOverlay>(d.import_namespaces.len())?,
            limits,
        )?;
        owned.add(Usage::array::<RecordOverlay>(records)?, limits)?;
        owned.add(Usage::array::<HostValue>(current_count)?, limits)?;
        held.add(owned, limits)?;
        // Admit maps before their first insert; payloads and geometry remain borrowed.
        held.add(Usage::btree::<HostRef, ()>(records)?, limits)?;
        held.add(Usage::btree::<HostRef, HostValue>(current_count)?, limits)?;
        for ns in d.import_namespaces.values() {
            held.add(
                Usage::btree::<NativeRecordKey, usize>(ns.records.len())?,
                limits,
            )?;
        }
        held.add(crate::dependency_cache::reservation(d)?, limits)?;
        let mut bound = BTreeSet::new();
        let mut hosts = BTreeMap::new();
        for entity in d.entities.values() {
            canceled(cancel)?;
            let host = HostRef::Object(entity.id);
            hosts.insert(host, HostValue::Entity { host, entity });
        }
        for layer in d.layers.values() {
            canceled(cancel)?;
            hosts.insert(HostRef::Layer(layer.id), HostValue::Layer(layer));
        }
        for definition in d.definitions.values() {
            canceled(cancel)?;
            hosts.insert(
                HostRef::Definition(definition.id),
                HostValue::Definition(definition),
            );
            for entity in &definition.members {
                canceled(cancel)?;
                let host = HostRef::DefinitionMember {
                    definition: definition.id,
                    object: entity.id,
                };
                hosts.insert(host, HostValue::Entity { host, entity });
            }
        }
        let mut namespaces = Vec::new();
        namespaces
            .try_reserve_exact(d.import_namespaces.len())
            .map_err(|_| Error::Budget("overlay namespaces"))?;
        for namespace in d.import_namespaces.values() {
            canceled(cancel)?;
            let mode = namespace
                .import_mode
                .ok_or(Error::Unsupported("legacy import policy is unknown"))?;
            let source = d
                .retained_sources
                .get(&namespace.source)
                .ok_or(Error::Missing("overlay retained source"))?;
            let mut rows = Vec::new();
            rows.try_reserve_exact(namespace.records.len())
                .map_err(|_| Error::Budget("overlay records"))?;
            let mut slots = BTreeMap::new();
            for record in &namespace.records {
                canceled(cancel)?;
                if let Some(host) = record.host
                    && !bound.insert(host)
                {
                    return Err(Error::Invalid("overlay host bound twice"));
                }
                let current = record.host.and_then(|host| hosts.get(&host).copied());
                let (action, changed) = action(record, current, mode)?;
                if slots.insert(record.key, rows.len()).is_some() {
                    return Err(Error::Invalid("overlay native key repeated"));
                }
                rows.push(RecordOverlay {
                    source: record,
                    action,
                    changed,
                    current,
                    needs_cache_refresh: false,
                });
            }
            for row in &rows {
                canceled(cancel)?;
                if matches!(
                    row.action,
                    RecordAction::Delete | RecordAction::Omit | RecordAction::Regenerate
                ) {
                    continue;
                }
                for relation in &row.source.relations {
                    if replaces_relation(row, &relation.role) {
                        continue;
                    }
                    let target = slots
                        .get(&relation.target)
                        .ok_or(Error::Missing("overlay native relation"))?;
                    if matches!(
                        rows[*target].action,
                        RecordAction::Delete | RecordAction::Omit
                    ) {
                        return Err(Error::Unsupported(
                            "unchanged native dependency was deleted",
                        ));
                    }
                }
            }
            namespaces.push(NamespaceOverlay {
                namespace,
                source,
                mode,
                records: rows,
            });
        }
        let mut additions = Vec::new();
        additions
            .try_reserve_exact(current_count)
            .map_err(|_| Error::Budget("overlay additions"))?;
        for entity in d.entities.values() {
            canceled(cancel)?;
            let host = HostRef::Object(entity.id);
            if !bound.contains(&host) {
                additions.push(HostValue::Entity { host, entity });
            }
        }
        for id in &d.layer_order {
            canceled(cancel)?;
            let layer = &d.layers[id];
            if !bound.contains(&HostRef::Layer(*id)) {
                additions.push(HostValue::Layer(layer));
            }
        }
        for definition in d.definitions.values() {
            canceled(cancel)?;
            if !bound.contains(&HostRef::Definition(definition.id)) {
                additions.push(HostValue::Definition(definition));
            }
            for entity in &definition.members {
                canceled(cancel)?;
                let host = HostRef::DefinitionMember {
                    definition: definition.id,
                    object: entity.id,
                };
                if !bound.contains(&host) {
                    additions.push(HostValue::Entity { host, entity });
                }
            }
        }
        crate::dependency_cache::refresh(d, &mut namespaces, cancel)?;
        self.revalidate(coordinator)?;
        canceled(cancel)?;
        Ok(CurrentOverlay {
            namespaces,
            additions,
            owned_usage: owned,
        })
    }
}

pub(crate) fn canceled(token: &Cancellation) -> Result<()> {
    if token.is_canceled() {
        Err(Error::Canceled)
    } else {
        Ok(())
    }
}
pub(crate) fn counts(d: &Document) -> Result<(usize, usize)> {
    let records = d.import_namespaces.values().try_fold(0usize, |n, ns| {
        n.checked_add(ns.records.len())
            .ok_or(Error::Overflow("overlay source record count"))
    })?;
    let mut current = d
        .entities
        .len()
        .checked_add(d.layers.len())
        .and_then(|n| n.checked_add(d.definitions.len()))
        .ok_or(Error::Overflow("overlay host count"))?;
    for def in d.definitions.values() {
        current = current
            .checked_add(def.members.len())
            .ok_or(Error::Overflow("overlay member count"))?;
    }
    Ok((records, current))
}
fn action(
    record: &SourceRecord,
    current: Option<HostValue<'_>>,
    mode: ImportMode,
) -> Result<(RecordAction, ChangedFields)> {
    let unchanged = ChangedFields::default();
    if record.disposition == RecordDisposition::Deleted {
        if current.is_some() {
            return Err(Error::Invalid("deleted native binding is live"));
        }
        return Ok((RecordAction::Delete, unchanged));
    }
    if record.disposition == RecordDisposition::RetainedOnly {
        if current.is_some() {
            return Err(Error::Invalid("retained-only binding is live"));
        }
        return Ok((
            if mode == ImportMode::Preserve {
                RecordAction::Retain
            } else {
                RecordAction::Omit
            },
            unchanged,
        ));
    }
    let current = current.ok_or(Error::Missing("overlay current native binding"))?;
    if mode == ImportMode::Geometry {
        return Ok((RecordAction::Regenerate, unchanged));
    }
    if record.disposition == RecordDisposition::Unchanged {
        return Ok((RecordAction::Retain, unchanged));
    }
    match current {
        HostValue::Entity { entity, .. } => {
            let original = record.entity_baseline.ok_or(Error::Unsupported(
                "edited native entity lacks field baselines",
            ))?;
            let current = EntityBaseline::capture(entity)?;
            let changed = ChangedFields {
                geometry: current.geometry_sha256 != original.geometry_sha256,
                placement: current.placement_sha256 != original.placement_sha256,
                attributes: current.attributes_sha256 != original.attributes_sha256,
                label: current.label_sha256 != original.label_sha256,
            };
            Ok((
                if changed == unchanged {
                    RecordAction::Retain
                } else {
                    RecordAction::PatchEntity
                },
                changed,
            ))
        }
        HostValue::Layer(_) => Ok((RecordAction::PatchLayer, unchanged)),
        HostValue::Definition(_) => Ok((RecordAction::PatchDefinition, unchanged)),
    }
}
fn replaces_relation(row: &RecordOverlay<'_>, role: &str) -> bool {
    match row.action {
        RecordAction::PatchEntity => {
            (row.changed.geometry && role == "instance_definition")
                || (row.changed.attributes && role == "layer")
        }
        RecordAction::PatchLayer => role == "parent_layer",
        RecordAction::PatchDefinition => role == "definition_member",
        _ => false,
    }
}
