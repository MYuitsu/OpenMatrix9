// SPDX-License-Identifier: LGPL-2.1-or-later
//! Closed Rust-authored identities and physical frames for current-state merge.
//! A map is an intent; native remap/preservation admission remains mandatory.
use crate::{
    CurrentOverlay, ExportCapture, RecordAction,
    current_overlay::{canceled, counts},
};
use om9_core::{
    Error, Result,
    budget::Usage,
    geometry::{Geometry, Transform},
    identity::{DefinitionId, ImportNamespaceId},
    model::{Document, Entity},
    retained::{HostRef, ImportMode, NativeRecordKey, NativeRecordKind},
    transaction::{Cancellation, TransactionCoordinator},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NativeIdentity {
    pub native_component_type: u32,
    pub id: Uuid,
    pub native_index: Option<i32>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedNativeKey {
    pub namespace: ImportNamespaceId,
    pub key: NativeRecordKey,
}
#[derive(Debug)]
pub struct OutputPlan<'a> {
    pub overlay: CurrentOverlay<'a>,
    pub primary_namespace: Option<ImportNamespaceId>,
    pub source_identities: BTreeMap<ScopedNativeKey, Option<NativeIdentity>>,
    pub host_identities: BTreeMap<HostRef, NativeIdentity>,
    pub model_mm_per_unit: f64,
    pub definition_mm_per_unit: BTreeMap<DefinitionId, f64>,
    pub owned_usage: Usage,
}
impl OutputPlan<'_> {
    /// Geometry is borrowed in canonical millimeters. The native adapter applies
    /// this already-decided frame once; it does not choose a unit policy.
    pub fn native_placement(&self, host: HostRef, entity: &Entity) -> Result<Transform> {
        let parent = match host {
            HostRef::Object(id) if id == entity.id => self.model_mm_per_unit,
            HostRef::DefinitionMember { definition, object } if object == entity.id => *self
                .definition_mm_per_unit
                .get(&definition)
                .ok_or(Error::Missing("output parent definition units"))?,
            _ => return Err(Error::Invalid("native placement host association")),
        };
        let child = if let Geometry::Instance(definition) = entity.geometry {
            *self
                .definition_mm_per_unit
                .get(&definition)
                .ok_or(Error::Missing("output child definition units"))?
        } else {
            1.
        };
        if !parent.is_finite() || parent <= 0. || !child.is_finite() || child <= 0. {
            return Err(Error::Invalid("native placement unit factors"));
        }
        // Parent and child scales may cancel even when either intermediate
        // uniform matrix determinant overflows/underflows. Validate the final
        // frame, rather than rejecting a gratuitous intermediate matrix.
        let ratio = child / parent;
        let mut matrix = entity.placement.0;
        for row in &mut matrix[..3] {
            for value in &mut row[..3] {
                *value *= ratio;
            }
            row[3] /= parent;
        }
        let result = Transform(matrix);
        result.validate()?;
        Ok(result)
    }
}
impl ExportCapture {
    pub fn current_output_plan<'a>(
        &'a self,
        coordinator: &TransactionCoordinator,
        primary_namespace: Option<ImportNamespaceId>,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<OutputPlan<'a>> {
        canceled(cancel)?;
        self.revalidate(coordinator)?;
        let d = self.document();
        let primary_namespace = primary(d, primary_namespace)?;
        let limits = coordinator.limits();
        let (records, hosts) = counts(d)?;
        let identities = records
            .checked_add(hosts)
            .ok_or(Error::Overflow("output identity count"))?;
        let mut owned = Usage::array::<OutputPlan>(1)?;
        owned.add(
            Usage::btree::<ScopedNativeKey, Option<NativeIdentity>>(records)?,
            limits,
        )?;
        owned.add(Usage::btree::<HostRef, NativeIdentity>(hosts)?, limits)?;
        owned.add(
            Usage::btree::<DefinitionId, f64>(d.definitions.len())?,
            limits,
        )?;
        let mut reservation = owned;
        let live_and_fresh = identities
            .checked_mul(2)
            .ok_or(Error::Overflow("output UUID reservation"))?;
        reservation.add(Usage::btree::<Uuid, ()>(live_and_fresh)?, limits)?;
        reservation.add(Usage::btree::<Uuid, ()>(records)?, limits)?;
        reservation.add(Usage::btree::<u32, i32>(17)?, limits)?;
        // Reserve maps with overlay/retained-validation scratch before allocating
        // either of them. No temporary source-geometry or payload copies occur.
        external.add(reservation, limits)?;
        let overlay = self.current_overlay_with_held_usage(coordinator, cancel, external)?;
        let mut used = BTreeSet::new();
        let mut primary_ids = BTreeSet::new();
        let mut next = BTreeMap::<u32, i32>::new();
        for ns in &overlay.namespaces {
            for row in &ns.records {
                canceled(cancel)?;
                let key = row.source.key;
                let ty = component_type(key.kind)?;
                // Source object-table lights deliberately carry no decoded
                // component index. This does not assert their wire light index.
                if ty != 9 && indexed(ty) != key.native_index.is_some() {
                    return Err(Error::Invalid("source component index semantics"));
                }
                if !key.id.is_nil() {
                    used.insert(key.id);
                }
                if Some(ns.namespace.id) == primary_namespace {
                    if !key.id.is_nil() && !primary_ids.insert(key.id) {
                        return Err(Error::Invalid("duplicate primary native UUID"));
                    }
                    if let Some(index) = key.native_index {
                        let after = index
                            .checked_add(1)
                            .ok_or(Error::Overflow("output native index"))?;
                        let value = next.entry(ty).or_insert(0);
                        *value = (*value).max(after);
                    }
                }
            }
        }
        for entity in d
            .entities
            .values()
            .chain(d.definitions.values().flat_map(|def| &def.members))
        {
            used.insert(entity.id.0);
        }
        for id in d.layers.keys() {
            used.insert(id.0);
        }
        for id in d.definitions.keys() {
            used.insert(id.0);
        }
        let mut source_identities = BTreeMap::new();
        let mut host_identities = BTreeMap::new();
        let mut definition_mm_per_unit = BTreeMap::new();
        for id in d.definitions.keys() {
            definition_mm_per_unit.insert(*id, 1.);
        }
        let mut model_mm_per_unit = 1.;
        for ns in &overlay.namespaces {
            canceled(cancel)?;
            let is_primary = Some(ns.namespace.id) == primary_namespace;
            if is_primary {
                model_mm_per_unit = ns.source.units.model.millimeters_per_unit;
            }
            for row in &ns.records {
                canceled(cancel)?;
                let source = row.source.key;
                let ty = component_type(source.kind)?;
                let output = if matches!(row.action, RecordAction::Delete | RecordAction::Omit) {
                    None
                } else if is_primary {
                    Some(NativeIdentity {
                        native_component_type: ty,
                        id: source.id,
                        native_index: source.native_index,
                    })
                } else {
                    Some(fresh(ty, &mut next, &mut used)?)
                };
                source_identities.insert(
                    ScopedNativeKey {
                        namespace: ns.namespace.id,
                        key: source,
                    },
                    output,
                );
                if let Some(current) = row.current {
                    let host = current.host();
                    let output = output.ok_or(Error::Invalid("live output binding omitted"))?;
                    if host_identities.insert(host, output).is_some() {
                        return Err(Error::Invalid("output host mapped twice"));
                    }
                    if is_primary && let HostRef::Definition(id) = host {
                        let factor = ns
                            .source
                            .units
                            .definitions
                            .get(&source.id)
                            .and_then(|unit| unit.millimeters_per_unit)
                            .ok_or(Error::Unsupported("output definition unit context missing"))?;
                        definition_mm_per_unit.insert(id, factor);
                    }
                }
            }
        }
        for current in &overlay.additions {
            canceled(cancel)?;
            let host = current.host();
            let ty = match host {
                HostRef::Layer(_) => 5,
                HostRef::Definition(_) => 11,
                _ => 12,
            };
            if host_identities
                .insert(host, fresh(ty, &mut next, &mut used)?)
                .is_some()
            {
                return Err(Error::Invalid("new output host already mapped"));
            }
        }
        owned.add(overlay.owned_usage, limits)?;
        self.revalidate(coordinator)?;
        canceled(cancel)?;
        Ok(OutputPlan {
            overlay,
            primary_namespace,
            source_identities,
            host_identities,
            model_mm_per_unit,
            definition_mm_per_unit,
            owned_usage: owned,
        })
    }
}
fn primary(d: &Document, selected: Option<ImportNamespaceId>) -> Result<Option<ImportNamespaceId>> {
    for ns in d.import_namespaces.values() {
        if ns.import_mode.is_none() {
            return Err(Error::Unsupported("legacy import policy is unknown"));
        }
    }
    if let Some(id) = selected {
        let ns = d
            .import_namespaces
            .get(&id)
            .ok_or(Error::Missing("primary import namespace"))?;
        if ns.import_mode != Some(ImportMode::Preserve) {
            return Err(Error::Invalid("primary namespace requires Preserve mode"));
        }
        return Ok(Some(id));
    }
    let mut primary = None;
    for ns in d
        .import_namespaces
        .values()
        .filter(|ns| ns.import_mode == Some(ImportMode::Preserve))
    {
        if primary.replace(ns.id).is_some() {
            return Err(Error::Unsupported(
                "multiple Preserve namespaces require a primary selection",
            ));
        }
    }
    Ok(primary)
}
fn fresh(
    ty: u32,
    next: &mut BTreeMap<u32, i32>,
    used: &mut BTreeSet<Uuid>,
) -> Result<NativeIdentity> {
    let native_index = if indexed(ty) {
        let index = *next.entry(ty).or_insert(0);
        next.insert(
            ty,
            index
                .checked_add(1)
                .ok_or(Error::Overflow("output native index"))?,
        );
        Some(index)
    } else {
        None
    };
    for _ in 0..16 {
        let id = Uuid::new_v4();
        if used.insert(id) {
            return Ok(NativeIdentity {
                native_component_type: ty,
                id,
                native_index,
            });
        }
    }
    Err(Error::Invalid("output UUID allocation collision"))
}
fn indexed(ty: u32) -> bool {
    ty != 12 && ty != 13
}
pub(crate) fn component_type(kind: NativeRecordKind) -> Result<u32> {
    Ok(match kind {
        NativeRecordKind::Image => 1,
        NativeRecordKind::TextureMapping => 2,
        NativeRecordKind::Material => 3,
        NativeRecordKind::LineType => 4,
        NativeRecordKind::Layer => 5,
        NativeRecordKind::Group => 6,
        NativeRecordKind::Other { native_type: 7 } => 7,
        NativeRecordKind::DimStyle => 8,
        NativeRecordKind::Light => 9,
        NativeRecordKind::HatchPattern => 10,
        NativeRecordKind::Definition => 11,
        NativeRecordKind::Geometry => 12,
        NativeRecordKind::History => 13,
        NativeRecordKind::RenderContent => 14,
        NativeRecordKind::EmbeddedFile => 15,
        NativeRecordKind::Other { native_type: 17 } => 17,
        _ => return Err(Error::Unsupported("native component type mapping")),
    })
}
