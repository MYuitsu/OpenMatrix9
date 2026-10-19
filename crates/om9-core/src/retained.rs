// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned source provenance and namespace-local native identities. This stores
//! merge inputs; it does not certify native parsing or enable a merge writer.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    identity::*,
    model::{Definition, Document, Entity, Layer},
    resource::{Resource, ResourceKind},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};

/// Hash typed host state directly into the digest, without a serialized buffer.
pub fn fingerprint<T: Serialize>(value: &T) -> Result<[u8; 32]> {
    struct HashWriter(Sha256);
    impl Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter(Sha256::new());
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| Error::Invalid("host fingerprint serialization"))?;
    Ok(writer.0.finalize().into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolvedBy {
    Declared,
    Explicit,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeUnits {
    pub native_enum: u32,
    pub name: String,
    pub declared_mm_per_unit: Option<f64>,
    pub millimeters_per_unit: f64,
    pub resolved_by: ResolvedBy,
    /// Original source-space values; host lengths are canonical millimeters.
    pub absolute_tolerance: f64,
    pub angular_tolerance_radians: f64,
    pub relative_tolerance: f64,
}
impl NativeUnits {
    fn validate(&self, l: &Limits) -> Result<Usage> {
        name(&self.name, l)?;
        if self.name.is_empty()
            || !self.millimeters_per_unit.is_finite()
            || self.millimeters_per_unit <= 0.
            || !self.absolute_tolerance.is_finite()
            || self.absolute_tolerance <= 0.
            || !self.angular_tolerance_radians.is_finite()
            || self.angular_tolerance_radians < 0.
            || !self.relative_tolerance.is_finite()
            || self.relative_tolerance < 0.
            || !(self.absolute_tolerance * self.millimeters_per_unit).is_finite()
            || self.absolute_tolerance * self.millimeters_per_unit <= 0.
        {
            return Err(Error::Invalid("source unit/tolerance resolution"));
        }
        match (self.resolved_by, self.declared_mm_per_unit) {
            (ResolvedBy::Declared, Some(f))
                if f.is_finite()
                    && f > 0.
                    && f == self.millimeters_per_unit
                    && self.native_enum != 0
                    && self.native_enum != 255 => {}
            (ResolvedBy::Explicit, None) => {}
            _ => return Err(Error::Invalid("source units require explicit resolution")),
        }
        Ok(Usage {
            bytes: self.name.capacity() as u64,
            ..Usage::default()
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefinitionUnits {
    pub native_enum: u32,
    pub name: String,
    pub declared_mm_per_unit: Option<f64>,
    /// None keeps unresolved source definitions retained without inventing a scale.
    pub millimeters_per_unit: Option<f64>,
    pub resolved_by: Option<ResolvedBy>,
}
impl DefinitionUnits {
    fn usage(&self, l: &Limits) -> Result<Usage> {
        name(&self.name, l)?;
        if self.name.is_empty() {
            return Err(Error::Invalid("definition unit name"));
        }
        match (
            self.declared_mm_per_unit,
            self.millimeters_per_unit,
            self.resolved_by,
        ) {
            (Some(declared), Some(resolved), Some(ResolvedBy::Declared))
                if declared.is_finite()
                    && declared > 0.
                    && declared == resolved
                    && self.native_enum != 0
                    && self.native_enum != 255 => {}
            (None, Some(resolved), Some(ResolvedBy::Explicit))
                if resolved.is_finite() && resolved > 0. => {}
            (None, None, None) => {}
            _ => return Err(Error::Invalid("definition unit resolution")),
        }
        Ok(Usage {
            bytes: self.name.capacity() as u64,
            ..Usage::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceUnits {
    pub model: NativeUnits,
    pub page: NativeUnits,
    /// Original native UUIDs are local to this retained source. An empty map
    /// supports earlier schema-3 documents that carried model/page context only.
    #[serde(default, deserialize_with = "crate::identity::unique_uuid_map")]
    pub definitions: BTreeMap<uuid::Uuid, DefinitionUnits>,
}
impl SourceUnits {
    fn usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = self.model.validate(l)?;
        u.add(self.page.validate(l)?, l)?;
        u.add(
            Usage::btree::<uuid::Uuid, DefinitionUnits>(self.definitions.len())?,
            l,
        )?;
        for (id, context) in &self.definitions {
            if id.is_nil() {
                return Err(Error::Invalid("definition unit identity"));
            }
            u.add(context.usage(l)?, l)?;
        }
        Ok(u)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedSource {
    pub id: SourceId,
    pub resource: ResourceId,
    pub sha256: [u8; 32],
    pub archive_version: u32,
    pub units: SourceUnits,
}
impl RetainedSource {
    fn validate(&self, resources: &BTreeMap<ResourceId, Resource>, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("source identity"));
        }
        let resource = resources
            .get(&self.resource)
            .ok_or(Error::Missing("retained source resource"))?;
        if resource.sha256() != self.sha256
            || resource.kind
                != (ResourceKind::RetainedThreeDm {
                    archive_version: self.archive_version,
                })
        {
            return Err(Error::Invalid("retained source hash/version"));
        }
        self.units.usage(l)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeRecordKind {
    Geometry,
    Layer,
    Definition,
    Material,
    Light,
    History,
    Group,
    LineType,
    DimStyle,
    HatchPattern,
    TextureMapping,
    Image,
    RenderContent,
    EmbeddedFile,
    Other { native_type: u32 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRecordKey {
    pub kind: NativeRecordKind,
    pub id: uuid::Uuid,
    pub native_index: Option<i32>,
}
impl NativeRecordKey {
    fn validate(self) -> Result<()> {
        if self.native_index.is_some_and(|i| i < 0)
            || (self.id.is_nil() && self.native_index.is_none())
        {
            return Err(Error::Invalid("native record identity/index"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HostRef {
    Object(ObjectId),
    Definition(DefinitionId),
    Layer(LayerId),
    DefinitionMember {
        definition: DefinitionId,
        object: ObjectId,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HostIdentity {
    Object(ObjectId),
    Definition(DefinitionId),
    Layer(LayerId),
}
impl HostRef {
    fn identity(self) -> HostIdentity {
        match self {
            Self::Object(id) | Self::DefinitionMember { object: id, .. } => {
                HostIdentity::Object(id)
            }
            Self::Definition(id) => HostIdentity::Definition(id),
            Self::Layer(id) => HostIdentity::Layer(id),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordDisposition {
    Unchanged,
    Edited,
    Deleted,
    RetainedOnly,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRelation {
    pub role: String,
    pub target: NativeRecordKey,
}
/// The explicit policy used by an import. Missing legacy policy is represented
/// by None on the namespace, never silently interpreted as preservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportMode {
    Preserve,
    Geometry,
}

/// Original typed host fields independently of entity identity and revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityBaseline {
    pub geometry_sha256: [u8; 32],
    pub placement_sha256: [u8; 32],
    pub attributes_sha256: [u8; 32],
    pub label_sha256: [u8; 32],
}
impl EntityBaseline {
    pub fn capture(entity: &Entity) -> Result<Self> {
        Ok(Self {
            geometry_sha256: fingerprint(&entity.geometry)?,
            placement_sha256: fingerprint(&entity.placement)?,
            attributes_sha256: fingerprint(&entity.attributes)?,
            label_sha256: fingerprint(&entity.label)?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    pub key: NativeRecordKey,
    pub family: String,
    pub host: Option<HostRef>,
    pub baseline_sha256: Option<[u8; 32]>,
    #[serde(default)]
    pub entity_baseline: Option<EntityBaseline>,
    pub disposition: RecordDisposition,
    pub relations: Vec<NativeRelation>,
}
impl SourceRecord {
    pub fn bound(key: NativeRecordKey, family: &str, host: HostRef, baseline: [u8; 32]) -> Self {
        Self {
            key,
            family: family.into(),
            host: Some(host),
            baseline_sha256: Some(baseline),
            entity_baseline: None,
            disposition: RecordDisposition::Unchanged,
            relations: vec![],
        }
    }
    pub fn bound_entity(
        key: NativeRecordKey,
        family: &str,
        host: HostRef,
        entity: &Entity,
    ) -> Result<Self> {
        key.validate()?;
        let object = match host {
            HostRef::Object(id) => id,
            HostRef::DefinitionMember { definition, object } if !definition.is_nil() => object,
            _ => return Err(Error::Invalid("entity baseline host association")),
        };
        if key.kind != NativeRecordKind::Geometry
            || entity.id.is_nil()
            || object != entity.id
            || key.id == entity.id.0
            || family.is_empty()
            || family.contains('\0')
        {
            return Err(Error::Invalid("entity baseline native/host association"));
        }
        let mut record = Self::bound(key, family, host, fingerprint(entity)?);
        record.entity_baseline = Some(EntityBaseline::capture(entity)?);
        Ok(record)
    }
    pub fn retained(key: NativeRecordKey, family: &str) -> Self {
        Self {
            key,
            family: family.into(),
            host: None,
            baseline_sha256: None,
            entity_baseline: None,
            disposition: RecordDisposition::RetainedOnly,
            relations: vec![],
        }
    }
    fn owned_usage(&self, l: &Limits) -> Result<Usage> {
        self.key.validate()?;
        name(&self.family, l)?;
        if self.family.is_empty() {
            return Err(Error::Invalid("native family"));
        }
        if self.entity_baseline.is_some()
            && (self.key.kind != NativeRecordKind::Geometry
                || !matches!(
                    self.host,
                    Some(HostRef::Object(_) | HostRef::DefinitionMember { .. })
                )
                || self.disposition == RecordDisposition::RetainedOnly)
        {
            return Err(Error::Invalid("entity baseline binding kind"));
        }
        let valid = matches!(
            (self.key.kind, self.host),
            (
                NativeRecordKind::Geometry,
                Some(HostRef::Object(_) | HostRef::DefinitionMember { .. })
            ) | (NativeRecordKind::Layer, Some(HostRef::Layer(_)))
                | (NativeRecordKind::Definition, Some(HostRef::Definition(_)))
        );
        if let Some(host) = self.host {
            let id = match host {
                HostRef::Object(id) => id.0,
                HostRef::Definition(id) => id.0,
                HostRef::Layer(id) => id.0,
                HostRef::DefinitionMember { definition, object } => {
                    if definition.is_nil() {
                        return Err(Error::Invalid("bound member definition identity"));
                    }
                    object.0
                }
            };
            if id.is_nil() || id == self.key.id {
                return Err(Error::Invalid("native UUID used as host identity"));
            }
        }
        if self.disposition == RecordDisposition::RetainedOnly {
            if self.host.is_some() || self.baseline_sha256.is_some() {
                return Err(Error::Invalid("retained-only host binding"));
            }
        } else if !valid || self.baseline_sha256.is_none() {
            return Err(Error::Invalid("native/host binding kind"));
        }
        let mut u = Usage {
            bytes: self.family.capacity() as u64,
            nodes: 1,
            ..Usage::default()
        };
        u.add(
            Usage::array::<NativeRelation>(self.relations.capacity())?,
            l,
        )?;
        for relation in &self.relations {
            relation.target.validate()?;
            name(&relation.role, l)?;
            if relation.role.is_empty() {
                return Err(Error::Invalid("native relation role"));
            }
            u.add(
                Usage {
                    bytes: relation.role.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        Ok(u)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportNamespace {
    pub id: ImportNamespaceId,
    pub source: SourceId,
    #[serde(default)]
    pub import_mode: Option<ImportMode>,
    pub records: Vec<SourceRecord>,
}
impl ImportNamespace {
    pub(crate) fn owned_usage(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() || self.source.is_nil() {
            return Err(Error::Invalid("import namespace identity"));
        }
        let mut u = Usage::array::<SourceRecord>(self.records.capacity())?;
        for r in &self.records {
            u.add(r.owned_usage(l)?, l)?;
        }
        Ok(u)
    }
}
/// Existing root/definition/layer maps stay borrowed. Local members get one
/// prospectively admitted index, shared by every namespace and tombstone check.
struct HostLookup<'a> {
    entities: &'a BTreeMap<ObjectId, Entity>,
    definitions: &'a BTreeMap<DefinitionId, Definition>,
    layers: &'a BTreeMap<LayerId, Layer>,
    members: BTreeMap<ObjectId, (DefinitionId, &'a Entity)>,
}
impl<'a> HostLookup<'a> {
    fn build(
        entities: &'a BTreeMap<ObjectId, Entity>,
        definitions: &'a BTreeMap<DefinitionId, Definition>,
        layers: &'a BTreeMap<LayerId, Layer>,
        limits: &Limits,
        mut held: Usage,
    ) -> Result<Self> {
        let count = definitions.values().try_fold(0usize, |n, d| {
            n.checked_add(d.members.len())
                .ok_or(Error::Overflow("retained member index"))
        })?;
        held.add(Usage::array::<Self>(1)?, limits)?;
        held.add(
            Usage::btree::<ObjectId, (DefinitionId, &Entity)>(count)?,
            limits,
        )?;
        let mut members = BTreeMap::new();
        for definition in definitions.values() {
            for member in &definition.members {
                if entities.contains_key(&member.id)
                    || members.insert(member.id, (definition.id, member)).is_some()
                {
                    return Err(Error::Invalid("retained member index identity repeated"));
                }
            }
        }
        Ok(Self {
            entities,
            definitions,
            layers,
            members,
        })
    }
    fn entity(&self, host: HostRef) -> Option<&'a Entity> {
        match host {
            HostRef::Object(id) => self.entities.get(&id),
            HostRef::DefinitionMember { definition, object } => self
                .members
                .get(&object)
                .filter(|(owner, _)| *owner == definition)
                .map(|(_, entity)| *entity),
            _ => None,
        }
    }
    fn hash(&self, host: HostRef) -> Result<Option<[u8; 32]>> {
        match host {
            HostRef::Object(_) | HostRef::DefinitionMember { .. } => {
                self.entity(host).map(fingerprint).transpose()
            }
            HostRef::Definition(id) => self.definitions.get(&id).map(fingerprint).transpose(),
            HostRef::Layer(id) => self.layers.get(&id).map(fingerprint).transpose(),
        }
    }
    fn any_live(&self, host: HostRef) -> bool {
        match host {
            HostRef::Object(id) | HostRef::DefinitionMember { object: id, .. } => {
                self.entities.contains_key(&id) || self.members.contains_key(&id)
            }
            HostRef::Layer(id) => self.layers.contains_key(&id),
            HostRef::Definition(id) => self.definitions.contains_key(&id),
        }
    }
}
impl Document {
    pub(crate) fn retained_validation_scratch_usage(&self, l: &Limits) -> Result<Usage> {
        let records = self.import_namespaces.values().try_fold(0usize, |n, ns| {
            n.checked_add(ns.records.len())
                .ok_or(Error::Overflow("retained records"))
        })?;
        let mut scratch = Usage::default();
        for storage in [
            Usage::btree::<NativeRecordKey, ()>(records)?,
            Usage::btree::<(NativeRecordKind, uuid::Uuid), ()>(records)?,
            Usage::btree::<(NativeRecordKind, i32), ()>(records)?,
            Usage::btree::<HostIdentity, ()>(records)?,
            Usage::btree::<SourceId, ()>(self.retained_sources.len())?,
        ] {
            scratch.add(storage, l)?;
        }
        if self
            .import_namespaces
            .values()
            .any(|ns| ns.records.iter().any(|r| r.host.is_some()))
        {
            let members = self.definitions.values().try_fold(0usize, |n, d| {
                n.checked_add(d.members.len())
                    .ok_or(Error::Overflow("retained member index"))
            })?;
            scratch.add(Usage::array::<HostLookup>(1)?, l)?;
            scratch.add(
                Usage::btree::<ObjectId, (DefinitionId, &Entity)>(members)?,
                l,
            )?;
        }
        Ok(scratch)
    }
    pub(crate) fn retained_owned_usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = Usage::btree::<SourceId, RetainedSource>(self.retained_sources.len())?;
        u.add(
            Usage::btree::<ImportNamespaceId, ImportNamespace>(self.import_namespaces.len())?,
            l,
        )?;
        for (id, source) in &self.retained_sources {
            if *id != source.id {
                return Err(Error::Invalid("retained source map key"));
            }
            u.add(source.validate(&self.resources, l)?, l)?;
            u.add(
                Usage {
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
        }
        for (id, ns) in &self.import_namespaces {
            if *id != ns.id || !self.retained_sources.contains_key(&ns.source) {
                return Err(Error::Invalid("import namespace source/key"));
            }
            u.add(ns.owned_usage(l)?, l)?;
            u.add(
                Usage {
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
        }
        Ok(u)
    }
    pub(crate) fn validate_retained(&self, l: &Limits, mut peak: Usage) -> Result<()> {
        // Admission precedes all identity/dependency sets. Baselines stream
        // directly to SHA, so validation has no serialized geometry copy.
        let records = self.import_namespaces.values().try_fold(0usize, |n, ns| {
            n.checked_add(ns.records.len())
                .ok_or(Error::Overflow("retained records"))
        })?;
        for scratch in [
            Usage::btree::<NativeRecordKey, ()>(records)?,
            Usage::btree::<(NativeRecordKind, uuid::Uuid), ()>(records)?,
            Usage::btree::<(NativeRecordKind, i32), ()>(records)?,
            Usage::btree::<HostIdentity, ()>(records)?,
            Usage::btree::<SourceId, ()>(self.retained_sources.len())?,
        ] {
            peak.add(scratch, l)?;
        }
        let lookup = self
            .import_namespaces
            .values()
            .any(|ns| ns.records.iter().any(|r| r.host.is_some()))
            .then(|| HostLookup::build(&self.entities, &self.definitions, &self.layers, l, peak))
            .transpose()?;
        let mut hosts = BTreeSet::new();
        let mut covered_sources = BTreeSet::new();
        for ns in self.import_namespaces.values() {
            covered_sources.insert(ns.source);
            let mut keys = BTreeSet::new();
            let mut ids = BTreeSet::new();
            let mut indices = BTreeSet::new();
            for record in &ns.records {
                let key = record.key;
                if !keys.insert(key)
                    || (!key.id.is_nil() && !ids.insert((key.kind, key.id)))
                    || key
                        .native_index
                        .is_some_and(|i| !indices.insert((key.kind, i)))
                {
                    return Err(Error::Invalid("duplicate native namespace record"));
                }
                if key.kind == NativeRecordKind::Definition
                    && record.host.is_some()
                    && self.retained_sources[&ns.source]
                        .units
                        .definitions
                        .get(&key.id)
                        .is_some_and(|context| context.millimeters_per_unit.is_none())
                {
                    return Err(Error::Invalid("bound definition units unresolved"));
                }
                if let Some(host) = record.host {
                    let lookup = lookup
                        .as_ref()
                        .ok_or(Error::Invalid("retained host lookup"))?;
                    if !hosts.insert(host.identity()) {
                        return Err(Error::Invalid("host bound by multiple native records"));
                    }
                    let current = lookup.hash(host)?;
                    match record.disposition {
                        RecordDisposition::Unchanged
                            if current.is_some() && current == record.baseline_sha256 => {}
                        RecordDisposition::Edited if current.is_some() => {}
                        RecordDisposition::Deleted
                            if current.is_none() && !lookup.any_live(host) => {}
                        _ => return Err(Error::Invalid("retained binding disposition/baseline")),
                    }
                    if record.disposition == RecordDisposition::Unchanged
                        && let Some(baseline) = record.entity_baseline
                    {
                        let entity = lookup
                            .entity(host)
                            .ok_or(Error::Missing("entity baseline host"))?;
                        if baseline != EntityBaseline::capture(entity)? {
                            return Err(Error::Invalid("unchanged entity field baseline"));
                        }
                    }
                }
            }
            for record in &ns.records {
                for relation in &record.relations {
                    if !keys.contains(&relation.target) {
                        return Err(Error::Missing("native namespace dependency"));
                    }
                }
            }
            for definition in self.retained_sources[&ns.source].units.definitions.keys() {
                if !ids.contains(&(NativeRecordKind::Definition, *definition)) {
                    return Err(Error::Missing("definition unit native namespace record"));
                }
            }
        }
        if self.retained_sources.iter().any(|(id, source)| {
            !source.units.definitions.is_empty() && !covered_sources.contains(id)
        }) {
            return Err(Error::Missing("definition unit source namespace"));
        }
        Ok(())
    }
    pub(crate) fn reconcile_retained(&mut self, limits: &Limits, held: Usage) -> Result<()> {
        if !self
            .import_namespaces
            .values()
            .any(|ns| ns.records.iter().any(|r| r.host.is_some()))
        {
            return Ok(());
        }
        let lookup = HostLookup::build(
            &self.entities,
            &self.definitions,
            &self.layers,
            limits,
            held,
        )?;
        for ns in self.import_namespaces.values_mut() {
            for record in &mut ns.records {
                let Some(host) = record.host else { continue };
                let current = lookup.hash(host)?;
                if record.disposition == RecordDisposition::Deleted {
                    if lookup.any_live(host) {
                        return Err(Error::Invalid("deleted native binding resurrected"));
                    }
                } else if current.is_none() {
                    if lookup.any_live(host) {
                        return Err(Error::Invalid("native binding owner changed"));
                    }
                    record.disposition = RecordDisposition::Deleted;
                } else if current != record.baseline_sha256 {
                    record.disposition = RecordDisposition::Edited;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn retained_object(&self, id: ObjectId) -> bool {
        self.import_namespaces
            .values()
            .flat_map(|ns| &ns.records)
            .any(|r| matches!(r.host,Some(HostRef::Object(i)) if i==id))
    }
}

#[cfg(test)]
mod indexed_reconciliation_tests {
    use super::*;
    use crate::geometry::Geometry;

    fn fixture(limits: &Limits) -> (Document, ObjectId, ImportNamespaceId) {
        let mut d = Document::new("indexed preflight");
        let entity = Entity::new("A", Geometry::Point([1.; 3]), None);
        let object = entity.id;
        let record = SourceRecord::bound_entity(
            NativeRecordKey {
                kind: NativeRecordKind::Geometry,
                id: uuid::Uuid::new_v4(),
                native_index: None,
            },
            "ON_Point",
            HostRef::Object(object),
            &entity,
        )
        .unwrap();
        let resource = Resource::from_bytes(
            "owned",
            ResourceKind::RetainedThreeDm {
                archive_version: 50,
            },
            b"authored structural fixture",
            limits,
        )
        .unwrap();
        let unit = NativeUnits {
            native_enum: 2,
            name: "Millimeters".into(),
            declared_mm_per_unit: Some(1.),
            millimeters_per_unit: 1.,
            resolved_by: ResolvedBy::Declared,
            absolute_tolerance: 0.001,
            angular_tolerance_radians: 0.001,
            relative_tolerance: 0.,
        };
        let source = RetainedSource {
            id: SourceId::new(),
            resource: resource.id,
            sha256: resource.sha256(),
            archive_version: 50,
            units: SourceUnits {
                model: unit.clone(),
                page: unit,
                definitions: Default::default(),
            },
        };
        let ns = ImportNamespace {
            id: ImportNamespaceId::new(),
            source: source.id,
            import_mode: Some(ImportMode::Preserve),
            records: vec![record],
        };
        let namespace = ns.id;
        d.entities.insert(object, entity);
        d.resources.insert(resource.id, resource);
        d.retained_sources.insert(source.id, source);
        d.import_namespaces.insert(ns.id, ns);
        (d, object, namespace)
    }
    #[test]
    fn aggregate_held_owners_refuse_reconciliation_scratch_before_changing_disposition() {
        let limits = Limits {
            max_bytes: 64 * 1024,
            ..Limits::default()
        };
        let (mut d, object, namespace) = fixture(&limits);
        let mut held = d.validate(&limits).unwrap();
        let external = vec![0u8; (limits.max_bytes - held.bytes) as usize];
        held.add(
            Usage {
                bytes: external.capacity() as u64,
                ..Usage::default()
            },
            &limits,
        )
        .unwrap();
        let original = d.import_namespaces[&namespace].records[0].baseline_sha256;
        d.entities.get_mut(&object).unwrap().geometry = Geometry::Point([2.; 3]);
        assert!(matches!(
            d.reconcile_retained(&limits, held),
            Err(Error::Budget(_))
        ));
        assert_eq!(
            d.import_namespaces[&namespace].records[0].disposition,
            RecordDisposition::Unchanged
        );
        assert_eq!(
            d.import_namespaces[&namespace].records[0].baseline_sha256,
            original
        );
    }
    fn member_fixture() -> (Document, ObjectId, DefinitionId, ImportNamespaceId) {
        let (mut d, object, ns) = fixture(&Limits::default());
        let entity = d.entities.remove(&object).unwrap();
        let definition = Definition::new(
            "original parent",
            vec![
                entity,
                Entity::new("independent sibling", Geometry::Point([6.; 3]), None),
            ],
        );
        let parent = definition.id;
        d.import_namespaces.get_mut(&ns).unwrap().records[0].host =
            Some(HostRef::DefinitionMember {
                definition: parent,
                object,
            });
        d.definitions.insert(parent, definition);
        d.validate(&Limits::default()).unwrap();
        (d, object, parent, ns)
    }
    #[test]
    fn indexed_reconciliation_keeps_original_member_baselines_and_marks_missing_members_deleted() {
        let (mut d, object, parent, ns) = member_fixture();
        let l = Limits::default();
        let held = d.validate(&l).unwrap();
        let baseline = d.import_namespaces[&ns].records[0].entity_baseline;
        let full = d.import_namespaces[&ns].records[0].baseline_sha256;
        d.reconcile_retained(&l, held).unwrap();
        assert_eq!(
            d.import_namespaces[&ns].records[0].disposition,
            RecordDisposition::Unchanged
        );
        let member = &mut d.definitions.get_mut(&parent).unwrap().members[0];
        assert_eq!(member.id, object);
        member.geometry = Geometry::Point([2.; 3]);
        member.geometry_revision += 1;
        d.reconcile_retained(&l, held).unwrap();
        assert_eq!(
            d.import_namespaces[&ns].records[0].disposition,
            RecordDisposition::Edited
        );
        assert_eq!(
            d.import_namespaces[&ns].records[0].entity_baseline,
            baseline
        );
        assert_eq!(d.import_namespaces[&ns].records[0].baseline_sha256, full);
        d.validate(&l).unwrap();
        d.definitions.get_mut(&parent).unwrap().members.remove(0);
        d.reconcile_retained(&l, held).unwrap();
        assert_eq!(
            d.import_namespaces[&ns].records[0].disposition,
            RecordDisposition::Deleted
        );
        assert_eq!(
            d.import_namespaces[&ns].records[0].entity_baseline,
            baseline
        );
        d.validate(&l).unwrap();
    }
    #[test]
    fn indexed_reconciliation_rejects_live_reparented_members_and_deleted_parent_aliases() {
        let (mut d, object, parent, ns) = member_fixture();
        let member = d.definitions.get_mut(&parent).unwrap().members.remove(0);
        let new_parent = Definition::new("other parent", vec![member]);
        d.definitions.insert(new_parent.id, new_parent);
        let l = Limits::default();
        let held = Usage {
            bytes: 64 * 1024,
            ..Usage::default()
        };
        assert_eq!(
            d.reconcile_retained(&l, held),
            Err(Error::Invalid("native binding owner changed"))
        );
        assert_eq!(
            d.import_namespaces[&ns].records[0].disposition,
            RecordDisposition::Unchanged
        );
        d.import_namespaces.get_mut(&ns).unwrap().records[0].disposition =
            RecordDisposition::Deleted;
        assert_eq!(
            d.reconcile_retained(&l, held),
            Err(Error::Invalid("deleted native binding resurrected"))
        );
        assert!(d.validate(&l).is_err());
        assert!(
            d.definitions
                .values()
                .any(|d| d.members.iter().any(|e| e.id == object))
        );
    }
}

/// Structurally prepared owned import; native correspondence remains the
/// responsibility of the separately gated exchange preparation pipeline.
#[derive(Debug, Clone)]
pub struct ImportBundle {
    pub resource: Resource,
    pub source: RetainedSource,
    pub namespace: ImportNamespace,
    pub entities: Vec<Entity>,
    pub layers: Vec<Layer>,
    pub definitions: Vec<Definition>,
}
impl ImportBundle {
    pub(crate) fn storage_usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = self.resource.validate(l)?;
        u.add(Usage::array::<Self>(1)?, l)?;
        let objects = self
            .entities
            .len()
            .checked_add(self.definitions.iter().try_fold(0usize, |n, d| {
                n.checked_add(d.members.len())
                    .ok_or(Error::Overflow("import members"))
            })?)
            .ok_or(Error::Overflow("import objects"))?;
        let nodes = self
            .layers
            .len()
            .checked_add(self.definitions.len())
            .and_then(|n| n.checked_add(2))
            .ok_or(Error::Overflow("import components"))?;
        u.add(
            Usage {
                objects: objects as u64,
                nodes: nodes as u64,
                ..Usage::default()
            },
            l,
        )?;
        u.add(self.source.units.usage(l)?, l)?;
        u.add(self.namespace.owned_usage(l)?, l)?;
        u.add(Usage::array::<Entity>(self.entities.capacity())?, l)?;
        u.add(Usage::array::<Layer>(self.layers.capacity())?, l)?;
        u.add(Usage::array::<Definition>(self.definitions.capacity())?, l)?;
        for e in self
            .entities
            .iter()
            .chain(self.definitions.iter().flat_map(|d| &d.members))
        {
            u.add(e.geometry.validate(l)?, l)?;
            u.add(
                Usage {
                    bytes: e.label.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        for layer in &self.layers {
            u.add(
                Usage {
                    bytes: layer.name.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        for definition in &self.definitions {
            u.add(Usage::array::<Entity>(definition.members.capacity())?, l)?;
            u.add(
                Usage {
                    bytes: definition.label.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        Ok(u)
    }
    pub(crate) fn admit(&self, document: DocumentId, l: &Limits, peak: &mut Usage) -> Result<()> {
        if self.source.resource != self.resource.id
            || self.namespace.source != self.source.id
            || self.source.id.is_nil()
            || self.source.sha256 != self.resource.sha256()
            || self.resource.kind
                != (ResourceKind::RetainedThreeDm {
                    archive_version: self.source.archive_version,
                })
            || self.namespace.records.iter().any(|r| {
                matches!(
                    r.disposition,
                    RecordDisposition::Edited | RecordDisposition::Deleted
                )
            })
        {
            return Err(Error::Invalid("new import source/disposition"));
        }
        let u = self.storage_usage(l)?;
        // The complete request was admitted by the coordinator before any
        // document validation. Add this prospective private validation clone.
        peak.add(u, l)?;
        for storage in [
            Usage::array::<Document>(1)?,
            Usage::btree::<ObjectId, Entity>(self.entities.len())?,
            Usage::btree::<LayerId, Layer>(self.layers.len())?,
            Usage::array::<LayerId>(self.layers.len())?,
            Usage::btree::<DefinitionId, Definition>(self.definitions.len())?,
            Usage::btree::<ResourceId, Resource>(1)?,
            Usage::btree::<SourceId, RetainedSource>(1)?,
            Usage::btree::<ImportNamespaceId, ImportNamespace>(1)?,
        ] {
            peak.add(storage, l)?;
        }
        let mut candidate = Document::new("import preflight");
        candidate.id = document;
        self.clone().apply(&mut candidate)?;
        // Charge verification scratch together with all already held input.
        let remaining = l.remaining_after(*peak)?;
        candidate.validate(&remaining)?;
        // Validation above proves every binding resolves to one unique current
        // host in this private candidate. Equal cardinality therefore proves
        // complete host closure without scanning the source records per host.
        let hosts = usize::try_from(u.objects)
            .map_err(|_| Error::Overflow("import host closure"))?
            .checked_add(self.layers.len())
            .and_then(|n| n.checked_add(self.definitions.len()))
            .ok_or(Error::Overflow("import host closure"))?;
        if self
            .namespace
            .records
            .iter()
            .filter(|record| record.host.is_some())
            .count()
            != hosts
        {
            return Err(Error::Missing("imported host source binding"));
        }
        Ok(())
    }
    pub(crate) fn apply(self, d: &mut Document) -> Result<()> {
        macro_rules! insert {
            ($map:expr,$id:expr,$value:expr) => {
                if $map.insert($id, $value).is_some() {
                    return Err(Error::Invalid("import host identity collision"));
                }
            };
        }
        d.layer_order
            .try_reserve_exact(self.layers.len())
            .map_err(|_| Error::Budget("import layer order allocation"))?;
        for entity in self.entities {
            insert!(d.entities, entity.id, entity);
        }
        for layer in self.layers {
            d.layer_order.push(layer.id);
            insert!(d.layers, layer.id, layer);
        }
        for definition in self.definitions {
            insert!(d.definitions, definition.id, definition);
        }
        insert!(d.resources, self.resource.id, self.resource);
        insert!(d.retained_sources, self.source.id, self.source);
        insert!(d.import_namespaces, self.namespace.id, self.namespace);
        Ok(())
    }
}
