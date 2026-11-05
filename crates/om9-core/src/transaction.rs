// SPDX-License-Identifier: LGPL-2.1-or-later
//! Atomic standalone publication. Callbacks cannot borrow mutation privilege.
pub use crate::scope::ScopeToken;
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Geometry, Transform, Unit},
    identity::*,
    model::{Attributes, Definition, Document, Entity, Extension, Feature, Group},
    resource::Resource,
    retained::{HostRef, ImportBundle},
    scope::Scope,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

/// A proof is valid only while its owning UndoEntry still owns the exact Arc.
/// The recorded limits are at least as restrictive as any cache hit's limits.
#[derive(Debug, Clone, Copy)]
struct RetainedValidationProof {
    address: usize,
    limits: Limits,
}

#[derive(Debug, Default)]
struct RetainedValidationCache(Mutex<[Option<RetainedValidationProof>; 2]>);

impl Clone for RetainedValidationCache {
    fn clone(&self) -> Self {
        let slots = *self.0.lock().unwrap_or_else(|error| error.into_inner());
        Self(Mutex::new(slots))
    }
}

// Validation receipts are transient optimization state, not transaction state.
impl PartialEq for RetainedValidationCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct World {
    pub documents: BTreeMap<DocumentId, Arc<Document>>,
}
impl World {
    pub fn new(documents: Vec<Document>, l: &Limits) -> Result<Self> {
        let mut map = BTreeMap::new();
        for d in documents {
            let id = d.id;
            if map.insert(id, Arc::new(d)).is_some() {
                return Err(Error::Invalid("duplicate document ID"));
            }
        }
        let world = Self { documents: map };
        world.validate(l)?;
        Ok(world)
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        let total = self.storage_usage(l)?;
        for (id, d) in &self.documents {
            if *id != d.id {
                return Err(Error::Invalid("document map key"));
            }
            let storage = d.storage_usage(l)?;
            let mut remaining = validation_limits(l, total, storage)?;
            // Point and node ceilings apply independently to each document;
            // other documents still consume their actual shared byte budget.
            remaining.max_points = l.max_points;
            remaining.max_nodes = l.max_nodes;
            let validated = d.validate(&remaining)?;
            debug_assert_eq!(validated, storage);
        }
        let count = allocated_identities(self).try_fold(0usize, |n, _| {
            n.checked_add(1)
                .ok_or(Error::Overflow("world identity count"))
        })?;
        let mut peak = total;
        peak.add(Usage::btree::<AllocatedIdentity, ()>(count)?, l)?;
        let mut identities = BTreeSet::new();
        for id in allocated_identities(self) {
            if !identities.insert(id) {
                return Err(Error::Invalid(
                    "identity owned by multiple documents or retired host resurrected",
                ));
            }
        }
        Ok(total)
    }
    fn storage_usage(&self, l: &Limits) -> Result<Usage> {
        let mut total = Usage::btree::<DocumentId, Arc<Document>>(self.documents.len())?;
        for (id, d) in &self.documents {
            if *id != d.id {
                return Err(Error::Invalid("document map key"));
            }
            let mut document_usage = d.storage_usage(l)?;
            // ponytail: allocation-free scan is quadratic; switch to an
            // admitted source index if profiling shows large worlds need it.
            for (index, brep) in retained_breps(d).enumerate() {
                let key = brep.source_allocation_key();
                if retained_breps(d).take(index).any(|other| other.source_allocation_key() == key) {
                    continue;
                }
                if self.documents.range(..*id).any(|(_, prior)| {
                    retained_breps(prior).any(|other| other.source_allocation_key() == key)
                }) {
                    document_usage.bytes = document_usage.bytes.checked_sub(brep.source_allocation_bytes()?)
                        .ok_or(Error::Overflow("shared world BRep source accounting"))?;
                }
            }
            for (index, brep) in retained_native_breps(d).enumerate() {
                let key = brep.payload_allocation_key();
                if retained_native_breps(d).take(index).any(|other| other.payload_allocation_key() == key) { continue; }
                if self.documents.range(..*id).any(|(_, prior)| retained_native_breps(prior).any(|other| other.payload_allocation_key() == key)) {
                    document_usage.bytes = document_usage.bytes.checked_sub(brep.payload_allocation_bytes()?)
                        .ok_or(Error::Overflow("shared world native BRep payload accounting"))?;
                }
            }
            for (index, brep) in retained_breps(d).enumerate() {
                let key = brep.display_allocation_key();
                if retained_breps(d).take(index).any(|other| other.display_allocation_key() == key) { continue; }
                if self.documents.range(..*id).any(|(_, prior)| retained_breps(prior).any(|other| other.display_allocation_key() == key)) {
                    document_usage.bytes = document_usage.bytes.checked_sub(brep.display_allocation_bytes()?)
                        .ok_or(Error::Overflow("shared world BRep display accounting"))?;
                }
            }
            add_retained_document_usage(&mut total, document_usage, l)?;
            total.add(Usage::array::<usize>(2)?, l)?;
        }
        Ok(total)
    }
    fn document(&self, id: DocumentId) -> Result<&Document> {
        self.documents
            .get(&id)
            .map(Arc::as_ref)
            .ok_or(Error::Missing("document"))
    }
    fn document_mut(&mut self, id: DocumentId) -> Result<&mut Document> {
        self.documents
            .get_mut(&id)
            .map(Arc::make_mut)
            .ok_or(Error::Missing("document"))
    }
}

/// Conservative storage of one published result, mirroring `PublishedResult::validate`
/// without requiring the post-mutation document. The exact revision check runs on the
/// staged document.
fn published_result_usage(
    id: ObjectId,
    result: &crate::result::PublishedResult,
    l: &Limits,
) -> Result<Usage> {
    let _ = id;
    let mut u = Usage {
        bytes: std::mem::size_of::<crate::result::PublishedResult>() as u64
            + result.operation.capacity() as u64,
        ..Usage::default()
    };
    u.add(result.geometry_snapshot.validate(l)?, l)?;
    u.add(
        Usage::array::<crate::identity::SourceStamp>(result.source_stamps.capacity())?,
        l,
    )?;
    u.add(result.conversion.validate(l)?, l)?;
    Ok(u)
}

/// The validated owner is already in total; all other counters remain reserved.
fn validation_limits(l: &Limits, total: Usage, owner: Usage) -> Result<Limits> {
    let difference = |all: u64, own: u64| {
        all.checked_sub(own)
            .ok_or(Error::Invalid("validation owner reservation"))
    };
    l.remaining_after(Usage {
        bytes: difference(total.bytes, owner.bytes)?,
        objects: difference(total.objects, owner.objects)?,
        // Nodes and points are per-document structural quotas, not additive
        // storage shared by retained versions or independent documents.
        nodes: 0,
        points: 0,
        history: difference(total.history, owner.history)?,
        outputs: difference(total.outputs, owner.outputs)?,
    })
}

/// History versions each have their own per-document point and node ceilings.
/// Bytes remain additive across simultaneously retained versions.
fn add_retained_document_usage(total: &mut Usage, mut document: Usage, limits: &Limits) -> Result<()> {
    let maximum_points = total.points.max(document.points);
    let maximum_nodes = total.nodes.max(document.nodes);
    if maximum_points > limits.max_points { return Err(Error::Budget("points")); }
    if maximum_nodes > limits.max_nodes { return Err(Error::Budget("nodes")); }
    document.points = 0;
    document.nodes = 0;
    total.nodes = 0;
    total.add(document, limits)?;
    total.points = maximum_points;
    total.nodes = maximum_nodes;
    Ok(())
}

fn retained_breps(document: &Document) -> impl Iterator<Item = &crate::brep::Brep> {
    document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .filter_map(|entity| match &entity.geometry { Geometry::Brep(brep) => Some(brep), _ => None })
}

fn retained_native_breps(document: &Document) -> impl Iterator<Item = &crate::geometry::NativeBrep> {
    document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .filter_map(|entity| match &entity.geometry { Geometry::NativeBrep(brep) => Some(brep), _ => None })
}

#[derive(Debug, Clone, Default)]
pub struct Cancellation {
    own: Arc<AtomicBool>,
    linked: [Option<Arc<AtomicBool>>; 3],
}
impl Cancellation {
    /// A bounded union of shared operation owners. Neither caller can replace
    /// a cancellation already bound to a completed worker's result.
    pub fn either(&self, other: &Self) -> Result<Self> {
        let mut union = self.clone();
        for flag in std::iter::once(&other.own).chain(other.linked.iter().flatten()) {
            if Arc::ptr_eq(&union.own, flag)
                || union
                    .linked
                    .iter()
                    .flatten()
                    .any(|held| Arc::ptr_eq(held, flag))
            {
                continue;
            }
            let slot = union
                .linked
                .iter_mut()
                .find(|slot| slot.is_none())
                .ok_or(Error::Budget("cancellation owner limit"))?;
            *slot = Some(flag.clone());
        }
        Ok(union)
    }
    pub fn cancel(&self) {
        self.own.store(true, Ordering::Release);
    }
    pub fn is_canceled(&self) -> bool {
        self.own.load(Ordering::Acquire)
            || self
                .linked
                .iter()
                .flatten()
                .any(|flag| flag.load(Ordering::Acquire))
    }
    fn check(&self) -> Result<()> {
        if self.is_canceled() {
            Err(Error::Canceled)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Mutation {
    Layers(Box<crate::layers::LayerPlan>),
    Publish(Box<crate::result::PreparedResult>),
    Insert {
        document: DocumentId,
        entity: Entity,
    },
    ResourceInsert {
        document: DocumentId,
        expected_revision: u64,
        resource: Resource,
    },
    /// Insert the typed exchange records referenced by a copied object.
    ExchangeInsert {
        document: DocumentId,
        expected_revision: u64,
        records: Box<crate::exchange::ExchangeMetadata>,
    },
    ResourceDelete {
        document: DocumentId,
        expected_revision: u64,
        resource: ResourceId,
    },
    Import {
        document: DocumentId,
        expected_revision: u64,
        bundle: Box<ImportBundle>,
    },
    Delete {
        source: SourceStamp,
    },
    Placement {
        source: SourceStamp,
        placement: Transform,
    },
    Geometry {
        source: SourceStamp,
        geometry: Geometry,
    },
    Attributes {
        source: SourceStamp,
        attributes: Attributes,
    },
    /// Replace the semantic extension slots of one object (for example the
    /// FourRailProfile metadata on a profile curve).
    Extensions {
        source: SourceStamp,
        extensions: std::collections::BTreeMap<String, crate::model::Extension>,
    },
    /// Rename one object (OM9-L3-03-14-007). The label is document state, so it
    /// goes through a transaction like every other attribute and Undo restores it.
    Label {
        source: SourceStamp,
        label: String,
    },
    Move {
        source: SourceStamp,
        destination: DocumentId,
        layer: Option<LayerId>,
    },
    Context {
        document: DocumentId,
        expected_revision: u64,
        page_unit: Unit,
    },
    /// Sets the document tolerance policy without touching `context_revision`, so
    /// existing stamps and Clean recipes stay valid (OM9-L3-01-08-003).
    Tolerance {
        document: DocumentId,
        expected_revision: u64,
        tolerance: crate::history::Tolerance,
    },
    /// Insert one history recipe in the same transaction as its output entity.
    History {
        document: DocumentId,
        expected_revision: u64,
        recipe: Box<crate::history::Recipe>,
        /// Published results for the recipe's outputs, applied after the entity insert.
        /// An insert without a result is valid for a feature whose geometry is not yet
        /// published; a builder that computes the geometry publishes both together.
        published: Vec<(ObjectId, crate::result::PublishedResult)>,
    },
    /// Replace one recorded feature and the published results of its outputs after a
    /// recompute, in one atomic step. The recipes must already exist; a recompute never
    /// invents a feature. Geometry mutations for the same outputs come first so the
    /// published snapshots and the entities agree when the transaction is validated.
    HistoryUpdate {
        document: DocumentId,
        expected_revision: u64,
        recipes: Vec<Box<crate::history::Recipe>>,
        published: Vec<(ObjectId, crate::result::PublishedResult)>,
    },
    /// Adds one occurrence set of a definition plus the definition itself in a
    /// single atomic step: a definition never exists without its occurrences and
    /// one Undo removes both.
    DefinitionInsert {
        document: DocumentId,
        expected_revision: u64,
        definition: Box<Definition>,
        instances: Vec<Entity>,
    },
    FeatureInsert {
        document: DocumentId,
        expected_revision: u64,
        feature: Box<Feature>,
    },
    GroupCreate {
        document: DocumentId,
        expected_revision: u64,
        group: Box<Group>,
    },
    GroupDelete {
        document: DocumentId,
        expected_revision: u64,
        group: GroupId,
    },
    /// Replace an existing group's membership in the same atomic edit as its objects.
    GroupReplace {
        document: DocumentId,
        expected_revision: u64,
        group: Box<Group>,
    },
    /// Sets or clears one named extension slot of one object. `None` removes the
    /// slot, so a caller cannot resurrect a slot it meant to clear.
    Extension {
        source: SourceStamp,
        slot: String,
        extension: Option<Extension>,
    },
}
impl Mutation {
    fn source(&self) -> Option<SourceStamp> {
        match self {
            Self::Delete { source }
            | Self::Placement { source, .. }
            | Self::Geometry { source, .. }
            | Self::Attributes { source, .. }
            | Self::Label { source, .. }
            | Self::Move { source, .. }
            | Self::Extension { source, .. } => Some(*source),
            _ => None,
        }
    }
    /// The field a source mutation writes. Two mutations may touch the same object in one
    /// transaction as long as they write different fields (geometry + payload + label +
    /// attributes is one user edit, so it must be one transaction and one Undo).
    fn write_field(&self) -> &'static str {
        match self {
            Self::Delete { .. } => "delete",
            Self::Placement { .. } => "placement",
            Self::Geometry { .. } => "geometry",
            Self::Attributes { .. } => "attributes",
            Self::Label { .. } => "label",
            Self::Move { .. } => "move",
            Self::Extension { .. } => "extension",
            _ => "other",
        }
    }
    fn participants(&self) -> Vec<DocumentId> {
        match self {
            Self::Layers(plan) => {
                if plan.changed() {
                    vec![plan.document()]
                } else {
                    vec![]
                }
            }
            Self::Publish(result) => vec![result.document],
            Self::Insert { document, .. }
            | Self::ResourceInsert { document, .. }
            | Self::ExchangeInsert { document, .. }
            | Self::ResourceDelete { document, .. }
            | Self::Import { document, .. }
            | Self::Context { document, .. }
            | Self::Tolerance { document, .. }
            | Self::History { document, .. }
            | Self::HistoryUpdate { document, .. }
            | Self::DefinitionInsert { document, .. }
            | Self::FeatureInsert { document, .. }
            | Self::GroupCreate { document, .. }
            | Self::GroupDelete { document, .. }
            | Self::GroupReplace { document, .. } => vec![*document],
            Self::Extensions { source, .. } => vec![source.document],
            Self::Move {
                source,
                destination,
                ..
            } => vec![source.document, *destination],
            _ => vec![self.source().expect("source mutation").document],
        }
    }
}
#[derive(Debug, Clone)]
pub struct TransactionRequest {
    pub label: String,
    pub mutations: Vec<Mutation>,
    pub recording: bool,
    pub owner: Option<ScopeToken>,
    pub cancel: Cancellation,
}
impl TransactionRequest {
    fn storage_usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = Usage::array::<Mutation>(self.mutations.capacity())?;
        u.add(
            Usage::array::<usize>(3 * (1 + self.cancel.linked.iter().flatten().count()))?,
            l,
        )?;
        u.add(
            Usage {
                bytes: self.label.capacity() as u64,
                outputs: self.mutations.len() as u64,
                ..Usage::default()
            },
            l,
        )?;
        for mutation in &self.mutations {
            let mut value = Usage::default();
            match mutation {
                Mutation::Layers(plan) => value = plan.storage_usage(l)?,
                Mutation::Import { bundle, .. } => value = bundle.storage_usage(l)?,
                Mutation::ResourceInsert { resource, .. } => value = resource.validate(l)?,
                Mutation::ExchangeInsert { records, .. } => value = records.validate(l)?,
                Mutation::Insert { entity, .. } => {
                    if !entity.extensions.is_empty() {
                        value.add(
                            Usage::btree::<String, Extension>(entity.extensions.len())?,
                            l,
                        )?;
                    }
                    for (slot, extension) in &entity.extensions {
                        value.add(
                            Usage {
                                bytes: slot.capacity() as u64,
                                ..Usage::default()
                            },
                            l,
                        )?;
                        value.add(extension.validate(l)?, l)?;
                    }
                    value.add(entity.geometry.validate(l)?, l)?;
                    value.add(
                        Usage {
                            bytes: entity.label.capacity() as u64,
                            objects: 1,
                            ..Usage::default()
                        },
                        l,
                    )?;
                    // An inserted entity may carry semantic slots (e.g. `om9.gem`); the
                    // request owns them until commit, exactly as the stored document does.
                    if !entity.extensions.is_empty() {
                        value.add(
                            Usage::btree::<String, Extension>(entity.extensions.len())?,
                            l,
                        )?;
                    }
                    for (slot, extension) in &entity.extensions {
                        value.add(
                            Usage {
                                bytes: slot.capacity() as u64,
                                ..Usage::default()
                            },
                            l,
                        )?;
                        value.add(extension.validate(l)?, l)?;
                    }
                }
                Mutation::Geometry { geometry, .. } => value = geometry.validate(l)?,
                Mutation::DefinitionInsert {
                    definition,
                    instances,
                    ..
                } => {
                    value.add(
                        Usage {
                            bytes: definition.label.capacity() as u64,
                            // The definition label plus one traversal node per
                            // member is what validate_internal charges; nested
                            // definitions are already charged by the base world.
                            nodes: 1 + definition.members.len() as u64,
                            ..Usage::default()
                        },
                        l,
                    )?;
                    for member in definition.members.iter().chain(instances) {
                        value.add(member.geometry.validate(l)?, l)?;
                        value.add(
                            Usage {
                                bytes: member.label.capacity() as u64,
                                objects: 1,
                                ..Usage::default()
                            },
                            l,
                        )?;
                    }
                }
                Mutation::FeatureInsert { feature, .. } => {
                    value.add(
                        Usage {
                            bytes: feature.label.capacity() as u64,
                            nodes: 1,
                            ..Usage::default()
                        },
                        l,
                    )?;
                }
                Mutation::GroupCreate { group, .. } | Mutation::GroupReplace { group, .. } => {
                    value.add(
                        Usage {
                            bytes: group.name.capacity() as u64,
                            nodes: 1,
                            ..Usage::default()
                        },
                        l,
                    )?;
                }
                Mutation::Extension {
                    slot, extension, ..
                } => {
                    value.add(
                        Usage {
                            bytes: slot.capacity() as u64,
                            ..Usage::default()
                        },
                        l,
                    )?;
                    if let Some(extension) = extension {
                        value.add(extension.validate(l)?, l)?;
                    }
                }
                Mutation::HistoryUpdate {
                    recipes, published, ..
                } => {
                    for recipe in recipes {
                        value.add(crate::model::Document::recipe_storage_usage(recipe, l)?, l)?;
                    }
                    for (id, result) in published {
                        value.add(published_result_usage(*id, result, l)?, l)?;
                    }
                }
                Mutation::History {
                    recipe, published, ..
                } => {
                    value.add(crate::model::Document::recipe_storage_usage(recipe, l)?, l)?;
                    for (id, result) in published {
                        value.add(published_result_usage(*id, result, l)?, l)?;
                    }
                }
                Mutation::Publish(result) => {
                    value.add(Usage::array::<crate::result::PreparedResult>(1)?, l)?;
                    value.add(
                        Usage {
                            bytes: result.operation.capacity() as u64,
                            outputs: result.outputs.len() as u64,
                            ..Usage::default()
                        },
                        l,
                    )?;
                    value.add(
                        Usage::array::<crate::request::SourceBinding>(result.sources.capacity())?,
                        l,
                    )?;
                    value.add(
                        Usage::array::<crate::result::PreparedOutput>(result.outputs.capacity())?,
                        l,
                    )?;
                    for input in &result.sources {
                        value.add(input.owned_usage(l)?, l)?;
                    }
                    for output in &result.outputs {
                        value.add(
                            Usage {
                                bytes: output.entity.label.capacity() as u64,
                                objects: u64::from(output.expected_existing.is_none()),
                                ..Usage::default()
                            },
                            l,
                        )?;
                        for geometry in [
                            &output.entity.geometry,
                            &output.geometry_snapshot,
                            &output.geometry_result,
                        ] {
                            value.add(geometry.validate(l)?, l)?;
                        }
                        value.add(output.conversion.validate(l)?, l)?;
                        value.add(
                            Usage::array::<SourceStamp>(output.source_stamps.capacity())?,
                            l,
                        )?;
                    }
                    if let Some(recipe) = &result.recipe {
                        value.add(Document::recipe_storage_usage(recipe, l)?, l)?;
                    }
                }
                _ => {}
            }
            u.add(value, l)?;
        }
        Ok(u)
    }

    pub fn new(label: impl Into<String>, mutations: Vec<Mutation>) -> Self {
        Self {
            label: label.into(),
            mutations,
            recording: true,
            owner: None,
            cancel: Cancellation::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UndoEntry {
    id: TransactionId,
    label: String,
    before: BTreeMap<DocumentId, Arc<Document>>,
    after: BTreeMap<DocumentId, Arc<Document>>,
    validation_cache: RetainedValidationCache,
}
impl UndoEntry {
    fn storage_usage(&self, limits: &Limits) -> Result<Usage> {
        let mut u = Usage {
            bytes: self.label.capacity() as u64,
            ..Usage::default()
        };
        u.add(
            Usage::btree::<DocumentId, Arc<Document>>(self.before.len())?,
            limits,
        )?;
        u.add(
            Usage::btree::<DocumentId, Arc<Document>>(self.after.len())?,
            limits,
        )?;
        Ok(u)
    }
    fn between(id: TransactionId, label: String, before: &World, after: &World) -> Self {
        let keys: BTreeSet<_> = before
            .documents
            .keys()
            .chain(after.documents.keys())
            .copied()
            .collect();
        let changed: Vec<_> = keys
            .into_iter()
            .filter(|id| before.documents.get(id) != after.documents.get(id))
            .collect();
        Self {
            id,
            label,
            before: changed
                .iter()
                .filter_map(|id| before.documents.get(id).map(|d| (*id, d.clone())))
                .collect(),
            after: changed
                .iter()
                .filter_map(|id| after.documents.get(id).map(|d| (*id, d.clone())))
                .collect(),
            validation_cache: RetainedValidationCache::default(),
        }
    }
    fn empty(&self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }
}

#[derive(Debug)]
pub struct PreparedTransaction {
    id: TransactionId,
    authority: SessionId,
    epoch: u64,
    scope_generation: u64,
    owner: Option<ScopeToken>,
    scope_version: Option<u64>,
    staged: World,
    entry: UndoEntry,
    recording: bool,
    cancel: Cancellation,
    held: Usage,
    reservation: Usage,
    identities: IdentityLedger,
}

/// Comparable runtime origin and caller lifecycle, without mutation privilege.
/// It is deliberately absent from every serialized document/archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationContext {
    authority: SessionId,
    epoch: u64,
    scope_generation: u64,
    scope_version: Option<u64>,
}
impl OperationContext {
    pub fn authority(&self) -> SessionId {
        self.authority
    }
}
impl PreparedTransaction {
    /// Read-only private candidate for scene/resource preparation before
    /// acceptance. Holding an Arc snapshot never grants publication privilege.
    pub fn candidate_world(&self) -> &World {
        &self.staged
    }
    /// Caller-owned allocations already admitted alongside this transaction.
    pub fn held_usage(&self) -> Usage {
        self.held
    }
    pub fn belongs_to(&self, token: &ScopeToken) -> bool {
        self.owner.as_ref() == Some(token)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    pub transaction: TransactionId,
    pub epoch: u64,
    pub published: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordinatorSnapshot {
    pub world: World,
    authority: SessionId,
    epoch: u64,
    scope_generation: u64,
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    scope: Option<Scope>,
    identities: IdentityLedger,
}
pub struct TransactionCoordinator {
    world: World,
    limits: Limits,
    physical_memory_sampler: Option<fn() -> Option<(u64, u64)>>,
    authority: SessionId,
    epoch: u64,
    scope_generation: u64,
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    scope: Option<Scope>,
    barrier: bool,
    benchmark_no_undo: bool,
    undo_policy: UndoPolicy,
    undo_ram_limit_mb: Option<u64>,
    undo_memory_stats: UndoMemoryStats,
    undo_capture_generation: u64,
    identities: IdentityLedger,
    current_validation_cache: Mutex<Option<(Weak<Document>, Limits)>>,
    /// Weak references to immutable documents held by external captures. They
    /// do not keep documents alive but allow the coordinator to charge them
    /// when Undo and the current world no longer own the same allocation.
    external_documents: Arc<Mutex<Vec<Weak<Document>>>>,
    external_reservations: Arc<Mutex<ExternalReservations>>,
    capture_generation: Arc<AtomicU64>,
    external_export_active: Arc<std::sync::atomic::AtomicBool>,
    /// Identifies the empty ledger charged once by the shared account rather
    /// than once again for every admission peer that shares it.
    shared_peer_identities: Option<IdentityLedger>,
    memory_peer_initialized: AtomicBool,
}

/// Bytes reserved for a live external owner. Dropping the guard releases them.
pub struct ExternalMemoryReservation {
    registry: Arc<Mutex<ExternalReservations>>,
    class: crate::budget::memory::MemoryClass,
    bytes: u64,
    pending_bytes: u64,
    physical_total: Option<u64>,
    commit_sampled: bool,
    admission_ceiling: Option<u64>,
}
#[derive(Default)]
struct ExternalReservations {
    by_class: [u64; 6],
    pending_by_class: [u64; 6],
    physical_sampler: Option<fn() -> Option<(u64, u64)>>,
    commit_sampler: Option<fn() -> Option<crate::budget::memory::CommitMemory>>,
    surface_snap_active: bool,
    /// A replacement no longer owns this account; stale handles cannot start jobs.
    retired: bool,
    /// Weak lease handles distinguish actual tracked captures from a document
    /// kept alive solely by the world or journal.
    capture_leases: Vec<Weak<CapturedDocumentInner>>,
    capture_generation: Arc<AtomicU64>,
    /// Tightest 60%-of-total-RAM ceiling learned from a valid physical sample.
    managed_ceiling: Option<u64>,
    optional_caches: Vec<OptionalCacheEntry>,
    optional_cache_sequence: u64,
    empty_peer_identities: IdentityLedger,
    last_external_refusal: Option<memory_refusal::StoredExternalRefusal>,
}

mod cache_pressure;
pub use cache_pressure::OptionalMemoryCache;
use cache_pressure::OptionalCacheEntry;
pub mod memory_diagnostics;
pub mod memory_refusal;

#[derive(Debug)]
struct CapturedDocumentInner {
    document: Arc<Document>,
    generation: Arc<AtomicU64>,
}
impl Drop for CapturedDocumentInner {
    fn drop(&mut self) { bump_capture_generation(&self.generation); }
}
fn bump_capture_generation(generation: &AtomicU64) {
    let _ = generation.fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| Some(value.saturating_add(1)));
}

/// A tracked immutable external capture. Clones share one lease; the lease
/// expires only when the last capture handle is dropped.
#[derive(Debug, Clone)]
pub struct CapturedDocument(Arc<CapturedDocumentInner>);
impl std::ops::Deref for CapturedDocument {
    type Target = Document;
    fn deref(&self) -> &Self::Target { &self.0.document }
}
impl CapturedDocument {
    /// Compare allocation identity without exposing a new untracked strong owner.
    pub fn matches_weak(&self, other: &Weak<Document>) -> bool {
        Arc::downgrade(&self.0.document).ptr_eq(other)
    }
}
impl ExternalReservations {
    /// Fixed allocations shared by the account, including Arc bodies as well
    /// as counters. Dynamic Vec capacity is added separately below.
    fn fixed_metadata_bytes() -> Result<u64> {
        let parts = [
            std::mem::size_of::<Mutex<Self>>(),
            std::mem::size_of::<Mutex<Vec<Weak<Document>>>>(),
            std::mem::size_of::<AtomicBool>(),
            std::mem::size_of::<AtomicU64>(),
            std::mem::size_of::<Vec<AllocatedIdentity>>(),
            // Preserve the conservative four-word Arc allowance used by the
            // existing generation/lease inventory; the ledger uses two.
            4 * 4 * std::mem::size_of::<usize>() + 2 * std::mem::size_of::<usize>(),
        ];
        parts.into_iter().try_fold(0u64, |total, bytes| {
            total.checked_add(bytes as u64).ok_or(Error::Overflow("shared account roots"))
        })
    }
    fn commit_available(&self, pending: u64) -> Result<Option<u64>> {
        let Some(sample) = self.commit_sampler else { return Ok(None); };
        let sample = sample().ok_or(Error::Unsupported("system commit availability is unavailable"))?;
        sample.available_for_pending(pending).map(Some)
    }
    /// Called with the account lock held, serializing a fresh sample with
    /// pending-to-allocated transitions. Keep a stricter caller sample (used
    /// for injected pressure), but never use its stale higher headroom.
    fn refresh_physical(&self, physical: Option<(u64, u64)>) -> Result<Option<(u64, u64)>> {
        let Some((total, available)) = physical else { return Ok(None); };
        crate::budget::memory::available_for_pending(total, available, 0)?;
        let Some(sample) = self.physical_sampler else { return Ok(physical); };
        let (fresh_total, fresh_available) = sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
        crate::budget::memory::available_for_pending(fresh_total, fresh_available, 0)?;
        let total = total.min(fresh_total);
        Ok(Some((total, available.min(fresh_available).min(total))))
    }
    fn pending_bytes(&self) -> Result<u64> {
        self.pending_by_class.iter().try_fold(0u64, |sum, bytes| sum.checked_add(*bytes).ok_or(Error::Overflow("pending reservations")))
    }
    fn constrain(&self, mut limits: Limits) -> Limits {
        if let Some(ceiling) = self.managed_ceiling {
            limits.max_bytes = limits.max_bytes.min(ceiling);
        }
        limits
    }

    /// Conservative managed bytes for the lease registry's retained Vec
    /// capacity and Arc control blocks, including expired weak entries until
    /// the registry prunes them. Allocator overhead is not included.
    fn accounted_classes(&self, additional_leases: usize) -> Result<[u64; 6]> {
        let count = self.capture_leases.len().checked_add(additional_leases)
            .ok_or(Error::Overflow("external capture leases"))?;
        let registry = Usage::array::<Weak<CapturedDocumentInner>>(self.capture_leases.capacity())?.bytes;
        // Four machine words conservatively cover the two counters and their
        // alignment in the pinned standard-library Arc allocation layout.
        let per_lease = std::mem::size_of::<CapturedDocumentInner>()
            .checked_add(4 * std::mem::size_of::<usize>())
            .ok_or(Error::Overflow("external capture lease bytes"))?;
        let blocks = count.checked_mul(per_lease).ok_or(Error::Overflow("external capture lease bytes"))?;
        let cache_index = Usage::array::<OptionalCacheEntry>(self.optional_caches.capacity())?.bytes;
        let fixed = Self::fixed_metadata_bytes()?;
        let metadata = registry.checked_add(u64::try_from(blocks).map_err(|_| Error::Overflow("external capture lease bytes"))?)
            .and_then(|bytes| bytes.checked_add(fixed))
            .and_then(|bytes| bytes.checked_add(cache_index))
            .ok_or(Error::Overflow("external capture lease bytes"))?;
        let mut classes = self.by_class;
        let document = crate::budget::memory::MemoryClass::Document.index();
        classes[document] = classes[document].checked_add(metadata).ok_or(Error::Overflow("external capture lease bytes"))?;
        Ok(classes)
    }
}
#[derive(Clone)]
pub struct ExternalMemoryAccount(Arc<Mutex<ExternalReservations>>, Arc<std::sync::atomic::AtomicBool>);
/// Locks shared retained owners through publication of a replacement. Callers
/// must not drop reservations or invoke account methods while the guard lives.
pub struct ReplacementAdmission {
    documents: Arc<Mutex<Vec<Weak<Document>>>>,
    reservations: Arc<Mutex<ExternalReservations>>,
}
pub struct ReplacementAdmissionGuard<'a> {
    owner: &'a ReplacementAdmission,
    documents: std::sync::MutexGuard<'a, Vec<Weak<Document>>>,
    reservations: std::sync::MutexGuard<'a, ExternalReservations>,
}
/// A validated transfer which retains the shared lock until publication.
pub struct CandidateReservationTransfer<'a, 'b> {
    guard: &'a mut ReplacementAdmissionGuard<'b>,
    reservation: &'a mut ExternalMemoryReservation,
    retired_bytes: u64,
}
impl CandidateReservationTransfer<'_, '_> {
    pub fn publish(self, publish: impl FnOnce()) {
        let index = self.reservation.class.index();
        self.guard.reservations.by_class[index] -= self.reservation.bytes;
        self.guard.reservations.pending_by_class[index] -= self.reservation.pending_bytes;
        self.reservation.bytes = 0;
        self.reservation.pending_bytes = 0;
        publish();
    }
    /// Keep the old coordinator metadata and auxiliary estimate charged while
    /// the caller drops the replaced owner after releasing the publication lock.
    pub fn publish_retaining_old(self, publish: impl FnOnce()) -> ExternalMemoryReservation {
        use crate::budget::memory::MemoryClass;
        let index = self.reservation.class.index();
        self.guard.reservations.by_class[index] -= self.reservation.bytes;
        self.guard.reservations.pending_by_class[index] -= self.reservation.pending_bytes;
        self.reservation.bytes = 0;
        self.reservation.pending_bytes = 0;
        let retired_index = MemoryClass::CpuCache.index();
        self.guard.reservations.by_class[retired_index] += self.retired_bytes;
        let mut retired = ExternalMemoryReservation {
            registry: Arc::clone(&self.guard.owner.reservations), class: MemoryClass::CpuCache,
            bytes: self.retired_bytes, pending_bytes: 0, physical_total: None, commit_sampled: false, admission_ceiling: None,
        };
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(publish)) {
            self.guard.reservations.by_class[retired_index] -= retired.bytes;
            retired.bytes = 0;
            drop(retired);
            std::panic::resume_unwind(panic);
        }
        retired
    }
}
impl ReplacementAdmission {
    pub fn lock(&self) -> ReplacementAdmissionGuard<'_> {
        let documents = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = self.reservations.lock().unwrap_or_else(|e| e.into_inner());
        ReplacementAdmissionGuard { owner: self, documents, reservations }
    }
}
impl<'registry> ReplacementAdmissionGuard<'registry> {
    /// `held` includes the replacement and non-coordinator owners, in bytes.
    pub fn check(&self, previous: &TransactionCoordinator, held: Usage) -> Result<()> {
        if !Arc::ptr_eq(&self.owner.documents, &previous.external_documents)
            || !Arc::ptr_eq(&self.owner.reservations, &previous.external_reservations) {
            return Err(Error::Invalid("replacement admission account"));
        }
        let limits = self.reservations.constrain(previous.limits);
        let remaining = limits.remaining_after(held)?;
        let mut usage = held;
        usage.add(previous.validate_retained_owners_with_external(
            &previous.identities, None, false, &remaining, &self.documents,
            &self.reservations.accounted_classes(0)?,
        )?, &limits)?;
        Ok(())
    }

    /// Atomically convert a candidate reservation into accounted replacement
    /// storage and publish it under the same lock. On refusal, the reservation
    /// is left intact and the publication callback is not called.
    pub fn prepare_candidate_reservation<'guard>(
        &'guard mut self,
        previous: &TransactionCoordinator,
        reservation: &'guard mut ExternalMemoryReservation,
        candidate: Usage,
        auxiliary: Usage,
    ) -> Result<CandidateReservationTransfer<'guard, 'registry>> {
        if !Arc::ptr_eq(&self.owner.documents, &previous.external_documents)
            || !Arc::ptr_eq(&self.owner.reservations, &previous.external_reservations)
            || !Arc::ptr_eq(&self.owner.reservations, &reservation.registry) {
            return Err(Error::Invalid("replacement reservation account"));
        }
        if reservation.bytes < candidate.bytes {
            return Err(Error::Budget("replacement candidate reservation"));
        }
        let limits = self.reservations.constrain(previous.limits);
        let mut held = auxiliary;
        held.add(candidate, &limits)?;
        let remaining = limits.remaining_after(held)?;
        let mut classes = self.reservations.accounted_classes(0)?;
        let index = reservation.class.index();
        classes[index] = classes[index].checked_sub(reservation.bytes)
            .ok_or(Error::Invalid("replacement reservation accounting"))?;
        previous.validate_retained_owners_with_external(
            &previous.identities, None, false, &remaining, &self.documents, &classes,
        )?;
        self.reservations.by_class[index].checked_sub(reservation.bytes)
            .ok_or(Error::Invalid("replacement reservation accounting"))?;
        let retired_bytes = previous.retained_metadata_usage(&previous.identities, None, false, &limits)?.bytes
            .checked_add(auxiliary.bytes).ok_or(Error::Overflow("retired controller storage"))?;
        let retired_index = crate::budget::memory::MemoryClass::CpuCache.index();
        let current_retired = self.reservations.by_class[retired_index];
        let after_candidate = if index == retired_index { current_retired.checked_sub(reservation.bytes)
            .ok_or(Error::Invalid("replacement reservation accounting"))? } else { current_retired };
        after_candidate.checked_add(retired_bytes).ok_or(Error::Overflow("retired controller reservation"))?;
        Ok(CandidateReservationTransfer { guard: self, reservation, retired_bytes })
    }
}
/// One active export worker for a shared account; held until the worker exits.
pub struct ExternalExportPermit(Arc<std::sync::atomic::AtomicBool>);
/// One Surface snap worker per shared coordinator account.
pub struct ExternalSurfaceSnapPermit(Arc<Mutex<ExternalReservations>>);
impl ExternalMemoryAccount {
    pub fn try_acquire_surface_snap(&self) -> Result<ExternalSurfaceSnapPermit> {
        let mut registry = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if registry.retired { return Err(Error::Invalid("retired memory account")); }
        if registry.surface_snap_active { return Err(Error::Invalid("surface snap already active")); }
        registry.surface_snap_active = true;
        Ok(ExternalSurfaceSnapPermit(Arc::clone(&self.0)))
    }
    pub fn try_acquire_export(&self) -> Result<ExternalExportPermit> {
        use std::sync::atomic::Ordering;
        let registry = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if registry.retired { return Err(Error::Invalid("retired memory account")); }
        self.1.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::Invalid("export already active"))?;
        Ok(ExternalExportPermit(Arc::clone(&self.1)))
    }
}
impl Drop for ExternalSurfaceSnapPermit {
    fn drop(&mut self) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).surface_snap_active = false;
    }
}
impl Drop for ExternalExportPermit {
    fn drop(&mut self) { self.0.store(false, std::sync::atomic::Ordering::Release); }
}
impl ExternalMemoryReservation {
    pub fn class(&self) -> crate::budget::memory::MemoryClass { self.class }
    pub fn bytes(&self) -> u64 { self.bytes }
    pub fn pending_bytes(&self) -> u64 { self.pending_bytes }
    /// Reconcile a reusable owner's existing allocation after allocation/free.
    /// The managed allowance stays unchanged; future reuse must recheck pending
    /// headroom before allocating again.
    pub fn set_allocated_bytes(&mut self, allocated: u64) -> Result<()> {
        let pending = self.bytes.checked_sub(allocated).ok_or(Error::Invalid("allocated reservation bytes"))?;
        let mut registry = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        let index = self.class.index();
        let others = registry.pending_by_class[index].checked_sub(self.pending_bytes)
            .ok_or(Error::Invalid("pending reservation accounting"))?;
        let next = others.checked_add(pending).ok_or(Error::Overflow("pending reservations"))?;
        registry.pending_by_class[index] = next;
        self.pending_bytes = pending;
        Ok(())
    }
    /// Recheck configured platform probes while holding the account lock. Used
    /// by an owner that allocates lazily from an already admitted allowance.
    pub fn recheck_pending(&self) -> Result<()> {
        let registry = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        if registry.managed_ceiling < self.admission_ceiling { return Err(Error::Budget("physical RAM policy changed")); }
        let pending = registry.pending_bytes()?;
        if let Some(sample) = registry.physical_sampler {
            let (total, available) = sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
            if self.physical_total.is_some_and(|previous| total < previous) {
                return Err(Error::Budget("physical RAM policy changed"));
            }
            if pending > crate::budget::memory::available_for_pending(total, available, 0)? {
                return Err(Error::Budget("available physical RAM"));
            }
        }
        if registry.commit_available(0)?.is_some_and(|available| pending > available) {
            return Err(Error::Budget("available system commit"));
        }
        Ok(())
    }
    /// Mark an already-created allocation as live without releasing its managed
    /// charge. Do not call for planned capacity or future native allocations.
    /// Allocation does not certify current residency; OS/commit probes and
    /// margins remain necessary when admitting subsequent work.
    pub fn mark_allocated(&mut self, bytes: u64) -> Result<()> {
        let pending = self.pending_bytes.checked_sub(bytes).ok_or(Error::Invalid("allocated reservation bytes"))?;
        let mut reservations = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        reservations.pending_by_class[self.class.index()] -= bytes;
        self.pending_bytes = pending;
        Ok(())
    }
    /// A point-in-time sample was used for this reservation's admission.
    /// This does not guarantee that current free memory is unchanged.
    pub fn physical_sampled(&self) -> bool { self.physical_total.is_some() }
    pub fn commit_sampled(&self) -> bool { self.commit_sampled }
    pub fn belongs_to(&self, account: &ExternalMemoryAccount) -> bool { Arc::ptr_eq(&self.registry, &account.0) }
    /// Divide an existing reservation without changing the account total.
    /// Both returned guards keep their portions charged until dropped.
    pub fn split_off(&mut self, bytes: u64) -> Result<Self> {
        if bytes == 0 || bytes > self.bytes {
            return Err(Error::Invalid("reservation split"));
        }
        self.bytes -= bytes;
        // Transfer pending capacity first; allocated and pending portions are
        // fungible byte allowances, and the shared phase totals do not change.
        let pending_bytes = bytes.min(self.pending_bytes);
        self.pending_bytes -= pending_bytes;
        Ok(Self { registry: Arc::clone(&self.registry), class: self.class, bytes,
            pending_bytes,
            physical_total: self.physical_total, commit_sampled: self.commit_sampled, admission_ceiling: self.admission_ceiling })
    }
    /// Move ownership of existing materialized bytes between guards without
    /// releasing or re-admitting them. No allocation, account total or pending
    /// request changes; the receiver's existing pending/probe metadata is unchanged.
    /// The caller must keep data covered by one guard throughout publication.
    pub fn transfer_allocated_from(&mut self, source: &mut Self, bytes: u64) -> Result<()> {
        if self.class != source.class || !Arc::ptr_eq(&self.registry, &source.registry) {
            return Err(Error::Invalid("allocated transfer account or class"));
        }
        let allocated = source.bytes.checked_sub(source.pending_bytes)
            .ok_or(Error::Invalid("allocated transfer accounting"))?;
        if bytes > allocated { return Err(Error::Invalid("allocated transfer bytes")); }
        let total = self.bytes.checked_add(bytes).ok_or(Error::Overflow("allocated transfer"))?;
        source.bytes -= bytes;
        self.bytes = total;
        Ok(())
    }
    /// Atomically release a no-longer-needed portion without opening a gap for
    /// the still-live allocation. Increasing a reservation requires admission.
    pub fn shrink_to(&mut self, bytes: u64) -> Result<()> {
        let released = self.bytes.checked_sub(bytes).ok_or(Error::Invalid("reservation cannot grow by shrinking"))?;
        let mut reservations = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        let index = self.class.index();
        let remaining = reservations.by_class[index].checked_sub(released)
            .ok_or(Error::Invalid("reservation accounting"))?;
        reservations.by_class[index] = remaining;
        let pending_released = released.min(self.pending_bytes);
        reservations.pending_by_class[index] -= pending_released;
        self.pending_bytes -= pending_released;
        self.bytes = bytes;
        Ok(())
    }
    /// Recheck a fresh physical sample while accounting for all outstanding
    /// reservations. Tightened capacity or policy requires re-admission.
    pub fn recheck_physical(&self, total: u64, available: u64) -> Result<()> {
        if total == 0 || available > total { return Err(Error::Invalid("physical RAM sample")); }
        let original_total = self.physical_total.ok_or(Error::Invalid("physical RAM sample required"))?;
        let reservations = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        let (total, available) = reservations.refresh_physical(Some((total, available)))?
            .ok_or(Error::Invalid("physical RAM sample required"))?;
        if total < original_total || reservations.managed_ceiling < self.admission_ceiling {
            return Err(Error::Budget("physical RAM policy changed"));
        }
        let pending = reservations.pending_bytes()?;
        if pending > crate::budget::memory::available_for_pending(total, available, 0)? {
            return Err(Error::Budget("available physical RAM"));
        }
        if reservations.commit_available(0)?.is_some_and(|available| pending > available) {
            return Err(Error::Budget("available system commit"));
        }
        Ok(())
    }
}
impl Drop for ExternalMemoryReservation {
    fn drop(&mut self) {
        if self.bytes == 0 { return; }
        let mut reservations = self.registry.lock().unwrap_or_else(|e| e.into_inner());
        reservations.by_class[self.class.index()] -= self.bytes;
        reservations.pending_by_class[self.class.index()] -= self.pending_bytes;
    }
}
mod undo_memory;
pub use undo_memory::{UndoMemoryStats, UndoPolicy};
/// Session identity authority is independent of Undo snapshots. Insert may not
/// reallocate a UUID that an accepted public/caller state already used. Undo
/// and Redo restore original objects/resources, rather than allocating their
/// IDs again. Typed identities have independent authority even if raw UUIDs
/// happen to match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AllocatedIdentity {
    Object(ObjectId),
    Resource(ResourceId),
    Source(SourceId),
    Namespace(ImportNamespaceId),
    Layer(LayerId),
    Definition(DefinitionId),
    Feature(FeatureId),
    Group(GroupId),
}
fn allocated_identities(world: &World) -> impl Iterator<Item = AllocatedIdentity> + '_ {
    world.documents.values().flat_map(|d| {
        d.entities
            .values()
            .chain(d.definitions.values().flat_map(|def| &def.members))
            .map(|e| AllocatedIdentity::Object(e.id))
            .chain(d.resources.keys().copied().map(AllocatedIdentity::Resource))
            .chain(
                d.retained_sources
                    .keys()
                    .copied()
                    .map(AllocatedIdentity::Source),
            )
            .chain(
                d.import_namespaces
                    .keys()
                    .copied()
                    .map(AllocatedIdentity::Namespace),
            )
            .chain(d.layers.keys().copied().map(AllocatedIdentity::Layer))
            .chain(
                d.definitions
                    .keys()
                    .copied()
                    .map(AllocatedIdentity::Definition),
            )
            .chain(d.features.keys().copied().map(AllocatedIdentity::Feature))
            .chain(d.groups.keys().copied().map(AllocatedIdentity::Group))
            .chain(d.import_namespaces.values().flat_map(|ns| {
                ns.records.iter().filter_map(|r| {
                    if r.disposition == crate::retained::RecordDisposition::Deleted {
                        r.host.map(|host| match host {
                            HostRef::Object(id) | HostRef::DefinitionMember { object: id, .. } => {
                                AllocatedIdentity::Object(id)
                            }
                            HostRef::Layer(id) => AllocatedIdentity::Layer(id),
                            HostRef::Definition(id) => AllocatedIdentity::Definition(id),
                        })
                    } else {
                        None
                    }
                })
            }))
    })
}
#[derive(Debug, Clone, PartialEq, Default)]
struct IdentityLedger(Arc<Vec<AllocatedIdentity>>);
impl IdentityLedger {
    fn contains(&self, id: ObjectId) -> bool {
        self.contains_identity(AllocatedIdentity::Object(id))
    }
    fn contains_resource(&self, id: ResourceId) -> bool {
        self.contains_identity(AllocatedIdentity::Resource(id))
    }
    fn contains_feature(&self, id: FeatureId) -> bool {
        self.contains_identity(AllocatedIdentity::Feature(id))
    }
    fn contains_group(&self, id: GroupId) -> bool {
        self.contains_identity(AllocatedIdentity::Group(id))
    }
    fn contains_identity(&self, id: AllocatedIdentity) -> bool {
        self.0.binary_search(&id).is_ok()
    }
    fn usage_for(count: usize) -> Result<Usage> {
        let mut u = Usage::array::<AllocatedIdentity>(count)?;
        u.bytes = u
            .bytes
            .checked_add(
                (std::mem::size_of::<Vec<AllocatedIdentity>>() + 2 * std::mem::size_of::<usize>())
                    as u64,
            )
            .ok_or(Error::Overflow("identity ledger bytes"))?;
        u.nodes = count as u64;
        Ok(u)
    }
    fn usage(&self) -> Result<Usage> {
        Self::usage_for(self.0.len())
    }
    fn extend(&self, world: &World, limits: &Limits, peak: &mut Usage) -> Result<Self> {
        let live = || allocated_identities(world);
        let added = live()
            .filter(|id| !self.contains_identity(*id))
            .try_fold(0usize, |n, _| {
                n.checked_add(1).ok_or(Error::Overflow("identity count"))
            })?;
        if added == 0 {
            return Ok(self.clone());
        }
        let total = self
            .0
            .len()
            .checked_add(added)
            .ok_or(Error::Overflow("identity count"))?;
        // Charge old authority and this private candidate together before any
        // allocation. Sorted Vec enables a fallible exact reservation and cheap
        // immutable sharing; no semantic/native-handle alias deduplication.
        peak.add(Self::usage_for(total)?, limits)?;
        let mut ids = Vec::new();
        ids.try_reserve_exact(total)
            .map_err(|_| Error::Budget("identity ledger allocation"))?;
        ids.extend_from_slice(&self.0);
        ids.extend(live().filter(|id| !self.contains_identity(*id)));
        ids.sort_unstable();
        ids.dedup();
        Ok(Self(Arc::new(ids)))
    }
}
fn bump(n: u64, field: &'static str) -> Result<u64> {
    n.checked_add(1).ok_or(Error::Overflow(field))
}
impl TransactionCoordinator {
    /// Reserve external owners during private startup validation/identity capture.
    /// The accepted coordinator retains the original configured limits.
    pub fn new_with_held(world: World, limits: Limits, held: Usage) -> Result<Self> {
        Self::initialize(world, limits, held)
    }
    pub fn new(world: World, limits: Limits) -> Result<Self> {
        Self::initialize(world, limits, Usage::default())
    }
    fn initialize(world: World, limits: Limits, held: Usage) -> Result<Self> {
        if world.documents.values().any(|d| d.owner_session.is_nil()) {
            return Err(Error::Invalid("runtime owner session requires restore"));
        }
        let mut usage = held;
        usage.add(Usage::array::<Self>(1)?, &limits)?;
        usage.add(Usage { bytes: ExternalReservations::fixed_metadata_bytes()?, ..Usage::default() }, &limits)?;
        usage.add(IdentityLedger::usage_for(0)?, &limits)?;
        usage.add(world.validate(&limits.remaining_after(usage)?)?, &limits)?;
        let initial = IdentityLedger::default();
        let identities = initial.extend(&world, &limits, &mut usage)?;
        let external_reservations = Arc::new(Mutex::new(ExternalReservations::default()));
        let capture_generation = Arc::clone(&external_reservations.lock().unwrap_or_else(|e| e.into_inner()).capture_generation);
        Ok(Self {
            world,
            limits,
            physical_memory_sampler: None,
            authority: SessionId::new(),
            epoch: 1,
            scope_generation: 0,
            undo: vec![],
            redo: vec![],
            scope: None,
            barrier: false,
            benchmark_no_undo: false,
            undo_policy: UndoPolicy::default(),
            undo_ram_limit_mb: None,
            undo_memory_stats: UndoMemoryStats::default(),
            undo_capture_generation: 0,
            identities,
            current_validation_cache: Mutex::new(None),
            external_documents: Arc::new(Mutex::new(Vec::new())),
            external_reservations,
            capture_generation,
            external_export_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            shared_peer_identities: None,
            memory_peer_initialized: AtomicBool::new(false),
        })
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Ephemeral authority of this coordinator instance. Shared document Arcs
    /// and equal publication counters never transfer acceptance privilege.
    pub fn authority(&self) -> SessionId {
        self.authority
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    /// Supply the host's current physical (total, available) RAM when known.
    /// The portable coordinator can also be used without a platform sampler.
    /// The sampler must be short and must not reenter the coordinator/account:
    /// admission also calls it while holding the shared reservation lock.
    pub fn set_physical_memory_sampler(&mut self, sampler: fn() -> Option<(u64, u64)>) {
        self.physical_memory_sampler = Some(sampler);
        self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).physical_sampler = Some(sampler);
    }
    /// Optional platform system-commit probe. Like the physical probe, it is
    /// called under the shared account lock and must not reenter the account.
    pub fn set_commit_memory_sampler(&mut self, sampler: fn() -> Option<crate::budget::memory::CommitMemory>) {
        self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).commit_sampler = Some(sampler);
    }
    /// Snapshot the strictest currently known managed-byte ceiling while
    /// retaining the caller's per-document structural limits.
    fn admission_limits(&self) -> Limits {
        self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).constrain(self.limits)
    }
    pub fn external_memory_account(&self) -> ExternalMemoryAccount {
        ExternalMemoryAccount(Arc::clone(&self.external_reservations), Arc::clone(&self.external_export_active))
    }
    pub fn replacement_admission(&self) -> ReplacementAdmission {
        ReplacementAdmission { documents: Arc::clone(&self.external_documents), reservations: Arc::clone(&self.external_reservations) }
    }
    pub fn external_reserved_bytes(&self, class: crate::budget::memory::MemoryClass) -> u64 {
        self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).by_class[class.index()]
    }
    pub fn external_pending_bytes(&self, class: crate::budget::memory::MemoryClass) -> u64 {
        self.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).pending_by_class[class.index()]
    }
    /// Set a non-widening physical-RAM ceiling for the shared managed account.
    /// This accounts for known retained owners and caller-held bytes; it does
    /// not sample currently available RAM or account for unregistered owners.
    pub fn constrain_managed_ram(&self, total_physical: u64, held: Usage) -> Result<()> {
        if total_physical == 0 { return Err(Error::Invalid("physical RAM capacity")); }
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let mut reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let requested = (u128::from(total_physical) * 3 / 5) as u64;
        let ceiling = reservations.managed_ceiling.map_or(requested, |old| old.min(requested));
        let limits = Limits { max_bytes: self.limits.max_bytes.min(ceiling), ..self.limits };
        let remaining = limits.remaining_after(held)?;
        self.validate_retained_owners_with_external(&self.identities, None, false, &remaining, &external, &reservations.accounted_classes(0)?)?;
        reservations.managed_ceiling = Some(ceiling);
        Ok(())
    }
    /// Register an immutable current document retained outside the coordinator.
    /// Identical Arc allocations are inventoried once while the coordinator lives.
    pub fn capture_retained_document(&self, id: DocumentId) -> Result<Arc<Document>> {
        self.available()?;
        let document = self.world.documents.get(&id).ok_or(Error::Missing("capture document"))?.clone();
        self.register_external_document(&document)?;
        Ok(document)
    }
    /// True only for the exact immutable allocation already held by the world
    /// or history; used when constructing a scene for a staged history state.
    pub fn owns_retained_document(&self, document: &Arc<Document>) -> bool {
        self.retained_documents(None).any(|owned| Arc::ptr_eq(owned, document))
    }
    /// Register weak references to every currently retained document version.
    /// Existing world/history owners keep them alive; no history is copied.
    pub fn register_retained_documents(&self) -> Result<()> {
        self.available()?;
        for document in self.unique_retained_documents(None) {
            self.register_external_document(document)?;
        }
        Ok(())
    }
    /// Register an already-owned candidate for peer accounting. Validate its
    /// contents before treating it as an immutable external document.
    pub fn register_external_candidate(&self, document: &Arc<Document>) -> Result<()> {
        self.available()?;
        document.validate(&self.admission_limits())?;
        self.register_external_document(document)
    }
    fn register_external_document(&self, document: &Arc<Document>) -> Result<()> {
        let mut external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        external.retain(|item| item.strong_count() != 0);
        let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        self.register_external_document_locked(document, &mut external, &reservations)
    }
    fn register_external_document_locked(&self, document: &Arc<Document>, external: &mut Vec<Weak<Document>>, reservations: &ExternalReservations) -> Result<()> {
        let weak = Arc::downgrade(document);
        let inserted = !external.iter().any(|item| item.ptr_eq(&weak));
        let effective = reservations.constrain(self.limits);
        if inserted && external.len() == external.capacity() {
            let requested = external.len().checked_add(1).ok_or(Error::Overflow("external document owners"))?
                .max(external.capacity().checked_mul(2).ok_or(Error::Overflow("external document owners"))?);
            let mut classes = reservations.accounted_classes(0)?;
            let index = crate::budget::memory::MemoryClass::Document.index();
            // Before allocation, include both the existing and requested vectors.
            classes[index] = classes[index].checked_add(Usage::array::<Weak<Document>>(requested)?.bytes)
                .ok_or(Error::Overflow("external document owners"))?;
            self.validate_retained_owners_with_external(&self.identities, None, false, &effective, &external, &classes)?;
            let mut next = Vec::new();
            next.try_reserve_exact(requested).map_err(|_| Error::Budget("external document owners"))?;
            next.extend(external.iter().cloned());
            next.push(weak);
            let mut classes = reservations.accounted_classes(0)?;
            classes[index] = classes[index].checked_add(Usage::array::<Weak<Document>>(external.capacity())?.bytes)
                .ok_or(Error::Overflow("external document owners"))?;
            self.validate_retained_owners_with_external(&self.identities, None, false, &effective, &next, &classes)?;
            *external = next;
        } else {
            if inserted { external.push(weak.clone()); }
            let validation = reservations.accounted_classes(0).and_then(|classes| {
                self.validate_retained_owners_with_external(&self.identities, None, false, &effective, &external, &classes)
            });
            if let Err(error) = validation {
                if inserted { external.retain(|item| !item.ptr_eq(&weak)); }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Capture an immutable document with a separately observable lease
    /// lifetime. The existing untracked Arc-returning API remains available.
    pub fn capture_tracked_document(&self, id: DocumentId) -> Result<CapturedDocument> {
        self.capture_tracked_document_with_guard(id, || {})
    }

    fn capture_tracked_document_with_guard(&self, id: DocumentId, guard: impl FnOnce()) -> Result<CapturedDocument> {
        self.available()?;
        let document = self.world.documents.get(&id).ok_or(Error::Missing("capture document"))?.clone();
        let weak = Arc::downgrade(&document);
        // Use the same lock order as publication and reservation admission.
        // Neither registry becomes observable independently of the other.
        let mut external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        external.retain(|item| item.strong_count() != 0);
        let inserted = !external.iter().any(|item| item.ptr_eq(&weak));
        let mut reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        reservations.capture_leases.retain(|lease| lease.strong_count() != 0);
        guard();
        let effective = reservations.constrain(self.limits);
        let grow_documents = inserted && external.len() == external.capacity();
        let grow_leases = reservations.capture_leases.len() == reservations.capture_leases.capacity();
        let requested = |len: usize, capacity: usize| -> Result<usize> {
            Ok(len.checked_add(1).ok_or(Error::Overflow("external capture capacity"))?
                .max(capacity.checked_mul(2).ok_or(Error::Overflow("external capture capacity"))?))
        };
        let document_slots = if grow_documents { requested(external.len(), external.capacity())? } else { 0 };
        let lease_slots = if grow_leases { requested(reservations.capture_leases.len(), reservations.capture_leases.capacity())? } else { 0 };
        let validate = |document_slots: usize, lease_slots: usize| -> Result<()> {
            let mut classes = reservations.accounted_classes(1)?;
            let document_class = crate::budget::memory::MemoryClass::Document.index();
            let lease_bytes = Usage::array::<Weak<CapturedDocumentInner>>(lease_slots)?.bytes;
            classes[document_class] = classes[document_class]
                .checked_add(Usage::array::<Weak<Document>>(document_slots)?.bytes)
                .and_then(|bytes| bytes.checked_add(lease_bytes))
                .ok_or(Error::Overflow("external capture capacity"))?;
            self.validate_retained_owners_with_external(&self.identities, None, false, &effective, &external, &classes)?;
            Ok(())
        };
        // Existing and replacement allocations coexist until both checks pass.
        validate(document_slots, lease_slots)?;
        let mut document_replacement = None;
        if grow_documents {
            let mut next = Vec::new();
            next.try_reserve_exact(document_slots).map_err(|_| Error::Budget("external document owners"))?;
            next.extend(external.iter().cloned());
            validate(next.capacity(), lease_slots)?;
            document_replacement = Some(next);
        }
        let mut replacement = None;
        if grow_leases {
            let mut next = Vec::new();
            next.try_reserve_exact(lease_slots).map_err(|_| Error::Budget("external capture leases"))?;
            next.extend(reservations.capture_leases.iter().cloned());
            validate(document_replacement.as_ref().map_or(0, Vec::capacity), next.capacity())?;
            replacement = Some(next);
        }
        let capture = Arc::new(CapturedDocumentInner { document, generation: Arc::clone(&reservations.capture_generation) });
        if inserted {
            if let Some(mut next) = document_replacement {
                next.push(weak);
                *external = next;
            } else {
                external.push(weak);
            }
        }
        if let Some(mut next) = replacement {
            next.push(Arc::downgrade(&capture));
            reservations.capture_leases = next;
        } else {
            reservations.capture_leases.push(Arc::downgrade(&capture));
        }
        bump_capture_generation(&reservations.capture_generation);
        Ok(CapturedDocument(capture))
    }

    /// Reserve external scratch before starting an operation. RAM availability
    /// is an optional point-in-time (total, available) OS sample; outstanding
    /// pending allocations are subtracted from that sample. Allocated owners
    /// remain in managed usage, not in the pending physical request.
    pub fn reserve_external_memory(
        &self,
        class: crate::budget::memory::MemoryClass,
        bytes: u64,
        physical: Option<(u64, u64)>,
    ) -> Result<ExternalMemoryReservation> {
        self.reserve_external_memory_with_held(class, bytes, physical, Usage::default())
    }

    /// Reserve an external allocation while other caller-owned allocations
    /// remain live. `held` must not include coordinator owners or reservations.
    pub fn reserve_external_memory_with_held(
        &self,
        class: crate::budget::memory::MemoryClass,
        bytes: u64,
        physical: Option<(u64, u64)>,
        held: Usage,
    ) -> Result<ExternalMemoryReservation> {
        self.reserve_external_memory_with_allocated(class, bytes, 0, physical, held)
    }

    /// Register already-existing allocations plus an allowance for new bytes.
    /// Only `bytes - allocated` is a new physical request; the entire `bytes`
    /// counts against the managed ceiling until this guard is dropped.
    pub fn reserve_external_memory_with_allocated(
        &self,
        class: crate::budget::memory::MemoryClass,
        bytes: u64,
        allocated: u64,
        physical: Option<(u64, u64)>,
        held: Usage,
    ) -> Result<ExternalMemoryReservation> {
        let mut refusal = None;
        let result = self.with_optional_cache_reclamation(bytes.saturating_sub(allocated).max(1), || {
            self.try_reserve_external_memory_with_allocated(class, bytes, allocated, physical, held, &mut refusal)
        });
        if let Err(error) = &result {
            if let Some(refusal) = refusal.filter(|refusal| refusal.matches(error)) { self.store_external_refusal(refusal); }
        }
        result
    }
    fn try_reserve_external_memory_with_allocated(
        &self, class: crate::budget::memory::MemoryClass, bytes: u64, allocated: u64,
        physical: Option<(u64, u64)>, held: Usage, refusal: &mut Option<memory_refusal::StoredExternalRefusal>,
    ) -> Result<ExternalMemoryReservation> {
        self.available()?;
        use crate::budget::memory::MemoryClass;
        if bytes == 0 || matches!(class, MemoryClass::Document | MemoryClass::Undo) {
            return Err(Error::Invalid("external memory reservation"));
        }
        let pending_bytes = bytes.checked_sub(allocated).ok_or(Error::Invalid("allocated reservation bytes"))?;
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let mut reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let mut attempt = self.refusal_attempt(class, bytes, allocated, physical.is_some(), held, &reservations);
        let result = (|| {
        let physical = reservations.refresh_physical(physical)?;
        let existing = reservations.pending_bytes()?;
        if let Some((total, available)) = physical {
            if total == 0 || available > total { return Err(Error::Invalid("physical RAM sample")); }
            let managed_ceiling = (u128::from(total) * 3 / 5) as u64;
            reservations.managed_ceiling = Some(reservations.managed_ceiling.map_or(managed_ceiling, |old| old.min(managed_ceiling)));
            let available = crate::budget::memory::available_for_pending(total, available, existing)?;
            attempt.physical_new_pending_headroom_bytes = Some(available);
            if pending_bytes > available {
                return Err(Error::Budget("available physical RAM"));
            }
        }
        // A valid physical sample still tightens the managed ceiling when an
        // independent commit check refuses this request.
        attempt.commit_was_checked = true;
        let commit_available = reservations.commit_available(existing)?;
        attempt.commit_new_pending_headroom_bytes = commit_available;
        if commit_available.is_some_and(|available| pending_bytes > available) {
            return Err(Error::Budget("available system commit"));
        }
        let admission_limits = reservations.constrain(self.limits);
        let remaining = admission_limits.remaining_after(held)?;
        let index = class.index();
        let next = reservations.by_class[index].checked_add(bytes).ok_or(Error::Overflow("external reservation"))?;
        let next_pending = reservations.pending_by_class[index].checked_add(pending_bytes).ok_or(Error::Overflow("pending reservation"))?;
        let mut proposed = reservations.accounted_classes(0)?;
        proposed[index] = proposed[index].checked_add(bytes).ok_or(Error::Overflow("external reservation"))?;
        if let Err(error) = self.validate_retained_owners_with_external(&self.identities, None, false, &remaining, &external, &proposed) {
            return Err(match error {
                Error::Budget("bytes") if admission_limits.max_bytes < self.limits.max_bytes => Error::Budget("managed RAM"),
                other => other,
            });
        }
        reservations.by_class[index] = next;
        reservations.pending_by_class[index] = next_pending;
        Ok(ExternalMemoryReservation { registry: Arc::clone(&self.external_reservations), class, bytes,
            pending_bytes,
            physical_total: physical.map(|(total, _)| total),
            commit_sampled: commit_available.is_some(),
            admission_ceiling: physical.and(reservations.managed_ceiling) })
        })();
        *refusal = result.as_ref().err().and_then(|error| attempt.finish(error, &reservations, self.limits));
        result
    }

    /// Reserve the currently available managed headroom for an operation whose
    /// incremental allocations are bounded by the returned reservation. The
    /// inventory and reservation are read/updated under the same locks.
    pub fn reserve_remaining_external_memory_with_held(
        &self,
        class: crate::budget::memory::MemoryClass,
        physical: Option<(u64, u64)>,
        held: Usage,
    ) -> Result<ExternalMemoryReservation> {
        self.available()?;
        use crate::budget::memory::MemoryClass;
        if matches!(class, MemoryClass::Document | MemoryClass::Undo) {
            return Err(Error::Invalid("external memory reservation"));
        }
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let mut reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let physical = reservations.refresh_physical(physical)?;
        let classes = reservations.accounted_classes(0)?;
        let existing = reservations.pending_bytes()?;
        let physical_available = if let Some((total, available)) = physical {
            if total == 0 || available > total { return Err(Error::Invalid("physical RAM sample")); }
            let ceiling = (u128::from(total) * 3 / 5) as u64;
            reservations.managed_ceiling = Some(reservations.managed_ceiling.map_or(ceiling, |old| old.min(ceiling)));
            Some(crate::budget::memory::available_for_pending(total, available, existing)?)
        } else { None };
        let commit_available = reservations.commit_available(existing)?;
        let remaining = reservations.constrain(self.limits).remaining_after(held)?;
        let used = self.validate_retained_owners_with_external(&self.identities, None, false, &remaining, &external, &classes)?;
        let bytes = remaining.max_bytes.saturating_sub(used.bytes)
            .min(physical_available.unwrap_or(u64::MAX)).min(commit_available.unwrap_or(u64::MAX));
        if bytes == 0 { return Err(Error::Budget("external memory headroom")); }
        let index = class.index();
        reservations.by_class[index] = reservations.by_class[index].checked_add(bytes).ok_or(Error::Overflow("external reservation"))?;
        reservations.pending_by_class[index] = reservations.pending_by_class[index].checked_add(bytes).ok_or(Error::Overflow("pending reservation"))?;
        Ok(ExternalMemoryReservation { registry: Arc::clone(&self.external_reservations), class, bytes,
            pending_bytes: bytes,
            physical_total: physical.map(|(total, _)| total), commit_sampled: commit_available.is_some(),
            admission_ceiling: physical.and(reservations.managed_ceiling) })
    }

    /// Grow a reservation without ever releasing its existing bytes. The
    /// additional guard is admitted under the same lock as other reservations,
    /// then transferred into the original guard without changing the total.
    pub fn grow_external_memory_with_held(
        &self,
        reservation: &mut ExternalMemoryReservation,
        additional: u64,
        physical: Option<(u64, u64)>,
        held: Usage,
    ) -> Result<()> {
        if !Arc::ptr_eq(&self.external_reservations, &reservation.registry) {
            return Err(Error::Invalid("external reservation account"));
        }
        let total = reservation.bytes.checked_add(additional).ok_or(Error::Overflow("external reservation"))?;
        let pending_total = reservation.pending_bytes.checked_add(additional).ok_or(Error::Overflow("pending reservation"))?;
        let mut extra = self.reserve_external_memory_with_held(reservation.class, additional, physical, held)?;
        reservation.bytes = total;
        reservation.pending_bytes = pending_total;
        // A fresh sample covers all pending bytes, including the original
        // reservation. An unsampled increment invalidates that claim.
        reservation.physical_total = extra.physical_total;
        reservation.commit_sampled = extra.commit_sampled;
        reservation.admission_ceiling = extra.admission_ceiling;
        extra.bytes = 0;
        extra.pending_bytes = 0;
        drop(extra);
        Ok(())
    }

    /// Carry an external-capture inventory onto a replacement coordinator.
    /// The caller must also account for both coordinators during construction.
    pub fn adopt_external_captures(&mut self, previous: &Self) -> Result<()> {
        self.available()?;
        previous.available()?;
        if Arc::ptr_eq(&self.external_reservations, &previous.external_reservations) {
            return Err(Error::Invalid("replacement already shares memory account"));
        }
        if !self.external_documents.lock().unwrap_or_else(|e| e.into_inner()).is_empty() {
            return Err(Error::Invalid("replacement external capture inventory"));
        }
        let replacement_ceiling = {
            let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
            if reservations.by_class.iter().any(|bytes| *bytes != 0) || reservations.surface_snap_active
                || reservations.capture_leases.iter().any(|lease| lease.strong_count() != 0) {
                return Err(Error::Invalid("replacement external reservations"));
            }
            reservations.managed_ceiling
        };
        let previous_ceiling = previous.external_reservations.lock().unwrap_or_else(|e| e.into_inner()).managed_ceiling;
        if replacement_ceiling.is_some_and(|ceiling| previous_ceiling.is_none_or(|old| old > ceiling)) {
            return Err(Error::Invalid("replacement RAM ceiling would widen"));
        }
        if self.external_export_active.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Error::Invalid("replacement export already active"));
        }
        // Publish the source coordinator's existing world and history into
        // its own shared inventory before another coordinator joins it.
        previous.register_retained_documents()?;
        // Stale handles can acquire permits without borrowing this coordinator.
        // Hold the old account lock across the final recheck and switch.
        let retired_account = Arc::clone(&self.external_reservations);
        let mut retired = retired_account.lock().unwrap_or_else(|e| e.into_inner());
        if retired.by_class.iter().any(|bytes| *bytes != 0) || retired.surface_snap_active
            || retired.capture_leases.iter().any(|lease| lease.strong_count() != 0)
            || self.external_export_active.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Error::Invalid("replacement external reservations"));
        }
        let shared_documents = Arc::clone(&previous.external_documents);
        let shared_reservations = Arc::clone(&previous.external_reservations);
        {
            let mut external = shared_documents.lock().unwrap_or_else(|e| e.into_inner());
            external.retain(|item| item.strong_count() != 0);
            let original_len = external.len();
            let reservations = shared_reservations.lock().unwrap_or_else(|e| e.into_inner());
            let registration = (|| -> Result<()> {
                for document in self.unique_retained_documents(None) {
                    previous.register_external_document_locked(document, &mut external, &reservations)?;
                }
                let limits = reservations.constrain(self.limits);
                self.validate_retained_owners_with_external(&self.identities, None, false, &limits,
                    &external, &reservations.accounted_classes(0)?)?;
                Ok(())
            })();
            if let Err(error) = registration {
                external.truncate(original_len);
                return Err(error);
            }
            retired.retired = true;
            self.external_documents = Arc::clone(&shared_documents);
            self.external_reservations = Arc::clone(&shared_reservations);
            self.external_export_active = Arc::clone(&previous.external_export_active);
            self.capture_generation = Arc::clone(&previous.capture_generation);
            // Even equal numeric generations belong to different accounts.
            self.undo_capture_generation = u64::MAX;
        }
        Ok(())
    }
    /// Capture the actual public or active caller view. Existing ownership and
    /// thread checks apply before any operation can borrow the scoped document.
    pub fn capture_context(
        &self,
        owner: Option<&ScopeToken>,
    ) -> Result<(OperationContext, &World)> {
        let (world, scope_version) = self.request_world(owner)?;
        Ok((
            OperationContext {
                authority: self.authority,
                epoch: self.epoch,
                scope_generation: self.scope_generation,
                scope_version,
            },
            world,
        ))
    }
    /// Immutable document allocations retained by the world, Undo/Redo and
    /// caller scopes. Operation owners must reserve their additional buffers
    /// alongside this usage before launching work. `points` is the largest
    /// retained document's count, not the sum across history versions; bytes
    /// still account for the distinct retained allocations.
    pub fn retained_usage(&self) -> Result<Usage> {
        self.retained_usage_with_limits(&self.limits)
    }
    /// External owners stay live during validation scratch allocation.
    pub fn retained_usage_with_held_usage(&self, held: Usage) -> Result<Usage> {
        self.retained_usage_and_limits_with_held(held).map(|(usage, _)| usage)
    }
    /// Return retained usage and the effective limits from the same locked
    /// account snapshot, for planning allocations against the same ceiling.
    pub fn retained_usage_and_limits_with_held(&self, held: Usage) -> Result<(Usage, Limits)> {
        // Take one policy snapshot for both held bytes and retained owners.
        // A peer can tighten the shared ceiling between separate lock
        // acquisitions; constraining only after subtracting `held` would then
        // fail to charge those bytes against the new ceiling.
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let admission = reservations.constrain(self.limits);
        let limits = admission.remaining_after(held)?;
        let mut usage = held;
        usage.add(self.validate_retained_owners_with_external(&self.identities, None, false, &limits, &external, &reservations.accounted_classes(0)?)?, &admission)?;
        Ok((usage, admission))
    }
    fn retained_usage_with_limits(&self, limits: &Limits) -> Result<Usage> {
        self.with_optional_cache_reclamation(1, || self.validate_retained_owners(&self.identities, None, false, limits))
    }
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    /// Benchmark only: opt out before the first edit; the policy cannot be changed later.
    pub fn disable_undo_for_benchmark(&mut self) -> Result<()> {
        if self.epoch != 1 || self.scope.is_some() || !self.undo.is_empty() || !self.redo.is_empty() {
            return Err(Error::Invalid("benchmark Undo policy must be set before editing"));
        }
        self.benchmark_no_undo = true;
        Ok(())
    }
    pub fn benchmark_no_undo(&self) -> bool {
        self.benchmark_no_undo
    }
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }
    pub fn snapshot(&self) -> CoordinatorSnapshot {
        CoordinatorSnapshot {
            world: self.world.clone(),
            authority: self.authority,
            epoch: self.epoch,
            scope_generation: self.scope_generation,
            undo: self.undo.clone(),
            redo: self.redo.clone(),
            scope: self.scope.clone(),
            identities: self.identities.clone(),
        }
    }
    fn available(&self) -> Result<()> {
        if self.barrier {
            Err(Error::Busy)
        } else {
            Ok(())
        }
    }
    fn owner(&self, token: &ScopeToken) -> Result<&Scope> {
        let scope = self.scope.as_ref().ok_or(Error::WrongOwner)?;
        if scope.token != *token || token.thread != std::thread::current().id() {
            return Err(Error::WrongOwner);
        }
        Ok(scope)
    }
    pub fn editing_world(&self, token: &ScopeToken) -> Result<&World> {
        Ok(&self.owner(token)?.staged)
    }
    pub fn book(&mut self, label: impl Into<String>, recording: bool) -> Result<ScopeToken> {
        self.available()?;
        if self.scope.is_some() {
            return Err(Error::WrongOwner);
        }
        let label = label.into();
        name(&label, &self.limits)?;
        let mut held = Usage {
            bytes: label.capacity() as u64,
            ..Usage::default()
        };
        held.add(
            Usage::btree::<DocumentId, Arc<Document>>(self.world.documents.len())?,
            &self.limits,
        )?;
        held.add(
            Usage::btree::<DocumentId, Arc<Document>>(self.world.documents.len())?,
            &self.limits,
        )?;
        self.retained_usage_with_held_usage(held)?;
        let generation = bump(self.scope_generation, "scope generation")?;
        let token = ScopeToken {
            id: TransactionId::new(),
            generation,
            thread: std::thread::current().id(),
        };
        self.scope = Some(Scope {
            token: token.clone(),
            label,
            recording: recording && !self.benchmark_no_undo,
            active: false,
            baseline: self.world.clone(),
            staged: self.world.clone(),
            version: 1,
            savepoints: vec![],
        });
        self.scope_generation = generation;
        Ok(token)
    }
    pub fn begin(&mut self, token: &ScopeToken) -> Result<()> {
        self.available()?;
        if self.owner(token)?.active {
            return Err(Error::Invalid("scope already active"));
        }
        self.scope.as_mut().expect("validated scope").active = true;
        Ok(())
    }
    pub fn savepoint(&mut self, token: &ScopeToken) -> Result<()> {
        self.available()?;
        let s = self.owner(token)?;
        if !s.active {
            return Err(Error::Invalid("inactive scope"));
        }
        if s.savepoints.len() >= self.limits.max_depth {
            return Err(Error::Budget("savepoint depth"));
        }
        let version = bump(s.version, "scope version")?;
        let next = s
            .savepoints
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(2))
            .ok_or(Error::Overflow("savepoint allocation"))?
            .max(4);
        let mut held = Usage::array::<World>(next)?;
        held.add(
            Usage::btree::<DocumentId, Arc<Document>>(s.staged.documents.len())?,
            &self.limits,
        )?;
        self.retained_usage_with_held_usage(held)?;
        let saved = s.staged.clone();
        let s = self.scope.as_mut().expect("validated scope");
        s.savepoints
            .try_reserve(1)
            .map_err(|_| Error::Budget("savepoint allocation"))?;
        s.savepoints.push(saved);
        s.version = version;
        Ok(())
    }
    pub fn rollback_savepoint(&mut self, token: &ScopeToken) -> Result<()> {
        self.available()?;
        let s = self.owner(token)?;
        if s.savepoints.is_empty() {
            return Err(Error::Invalid("missing savepoint"));
        }
        let version = bump(s.version, "scope version")?;
        let s = self.scope.as_mut().expect("validated scope");
        s.staged = s.savepoints.pop().expect("validated savepoint");
        s.version = version;
        Ok(())
    }
    pub fn release_savepoint(&mut self, token: &ScopeToken) -> Result<()> {
        self.available()?;
        let s = self.owner(token)?;
        if s.savepoints.is_empty() {
            return Err(Error::Invalid("missing savepoint"));
        }
        let version = bump(s.version, "scope version")?;
        let s = self.scope.as_mut().expect("validated scope");
        s.savepoints.pop();
        s.version = version;
        Ok(())
    }
    pub fn abort(&mut self, token: &ScopeToken) -> Result<()> {
        self.available()?;
        self.owner(token)?;
        self.scope = None;
        Ok(())
    }
    pub fn finish(&mut self, token: &ScopeToken) -> Result<CommitReceipt> {
        self.finish_with_guard(token, Usage::default(), || Ok(()))
    }
    fn finish_with_guard<F>(&mut self, token: &ScopeToken, additional_held: Usage, guard: F) -> Result<CommitReceipt>
    where F: FnOnce() -> Result<()> {
        self.available()?;
        let s = self.owner(token)?;
        if !s.active || !s.savepoints.is_empty() {
            return Err(Error::Invalid("unfinished scope/savepoint"));
        }
        if s.baseline != self.world {
            return Err(Error::Stale("scope participants"));
        }
        let count = s
            .baseline
            .documents
            .len()
            .checked_add(s.staged.documents.len())
            .ok_or(Error::Overflow("scope Undo maps"))?;
        let mut held = additional_held;
        held.add(Usage {
            bytes: s.label.capacity() as u64,
            ..Usage::default()
        }, &self.limits)?;
        for storage in [
            Usage::btree::<DocumentId, Arc<Document>>(s.staged.documents.len())?,
            Usage::btree::<DocumentId, ()>(count)?,
            Usage::array::<DocumentId>(
                count
                    .checked_mul(2)
                    .ok_or(Error::Overflow("scope Undo keys"))?
                    .max(4),
            )?,
            Usage::btree::<DocumentId, Arc<Document>>(count)?,
            Usage::btree::<DocumentId, Arc<Document>>(count)?,
            Usage::array::<UndoEntry>(1)?,
        ] {
            held.add(storage, &self.limits)?;
        }
        if s.recording {
            let next = self
                .undo
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(2))
                .ok_or(Error::Overflow("Undo allocation"))?
                .max(4);
            held.add(Usage::array::<UndoEntry>(next)?, &self.limits)?;
        }
        let usage = self.retained_usage_with_held_usage(held)?;
        let entry = UndoEntry::between(token.id, s.label.clone(), &s.baseline, &s.staged);
        let staged = s.staged.clone();
        let recording = s.recording;
        let staged_storage = staged.storage_usage(&self.limits)?;
        staged.validate(&validation_limits(&self.limits, usage, staged_storage)?)?;
        self.check_retention(&entry, recording, &self.identities, &self.limits)?;
        let next = if entry.empty() {
            self.epoch
        } else {
            bump(self.epoch, "publication epoch")?
        };
        if recording && !entry.empty() {
            self.undo
                .try_reserve(1)
                .map_err(|_| Error::Budget("Undo allocation"))?;
        }
        guard()?;
        let published = !entry.empty();
        // Recheck all retained owners and the finish scratch under the shared
        // account locks and keep the reservation lock through publication.
        let external_registry = Arc::clone(&self.external_documents);
        let reservation_registry = Arc::clone(&self.external_reservations);
        let external = external_registry.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = reservation_registry.lock().unwrap_or_else(|e| e.into_inner());
        if published {
            let limits = reservations.constrain(self.limits).remaining_after(held)?;
            self.validate_retained_owners_with_external(&self.identities, Some(&entry), recording,
                &limits, &external, &reservations.accounted_classes(0)?)?;
        }
        drop(external);
        self.barrier = true;
        // The finished scope no longer owns its baseline or savepoints.
        // Remove it before Undo inventory so retained history is not mistaken
        // for data still required by an active scope.
        self.scope = None;
        if published {
            self.publish(staged, entry, recording, next, &reservations);
        }
        self.barrier = false;
        drop(reservations);
        Ok(CommitReceipt {
            transaction: token.id,
            epoch: self.epoch,
            published,
            diagnostics: vec![],
        })
    }
    pub fn open_document(&mut self, document: Document) -> Result<()> {
        self.open_document_with_guard(document, || Ok(()))
    }
    fn open_document_with_guard<F>(&mut self, mut document: Document, guard: F) -> Result<()>
    where F: FnOnce() -> Result<()> {
        self.available()?;
        if self.scope.is_some() {
            return Err(Error::WrongOwner);
        }
        if self.world.documents.contains_key(&document.id) {
            return Err(Error::Invalid("duplicate document"));
        }
        if document.owner_session.is_nil() {
            return Err(Error::Invalid("runtime owner session requires restore"));
        }
        let limits = self.admission_limits();
        let incoming_id = document.id;
        let input = document.storage_usage(&limits)?;
        let baseline = self.retained_usage_with_limits(&limits.remaining_after(input)?)?;
        let mut usage = baseline;
        document.renew_session(&limits.remaining_after(usage)?)?;
        usage.add(input, &limits)?;
        usage.add(Usage::array::<usize>(2)?, &limits)?;
        let documents = self
            .world
            .documents
            .len()
            .checked_add(1)
            .ok_or(Error::Overflow("open document map"))?;
        usage.add(
            Usage::btree::<DocumentId, Arc<Document>>(documents)?,
            &limits,
        )?;
        let next = bump(self.epoch, "publication epoch")?;
        let mut staged = self.world.clone();
        staged.documents.insert(document.id, Arc::new(document));
        let staged_storage = staged.storage_usage(&limits)?;
        staged.validate(&validation_limits(&limits, usage, staged_storage)?)?;
        let identities = self.identities.extend(&staged, &limits, &mut usage)?;
        // Keep the separately admitted candidate/preparation peak, then add it
        // to a fresh retained-owner inventory while holding the shared account.
        let difference = |after: u64, before: u64| after.checked_sub(before).ok_or(Error::Overflow("open document peak"));
        let additional = Usage {
            bytes: difference(usage.bytes, baseline.bytes)?,
            objects: difference(usage.objects, baseline.objects)?,
            nodes: difference(usage.nodes, baseline.nodes)?,
            points: difference(usage.points, baseline.points)?,
            history: difference(usage.history, baseline.history)?,
            outputs: difference(usage.outputs, baseline.outputs)?,
        };
        guard()?;
        self.register_external_document(staged.documents.get(&incoming_id).ok_or(Error::Missing("staged open document"))?)?;
        // This matches the capture-before-reservation lock order used by
        // transaction publication. A peer cannot reserve between this final
        // admission and making the new document visible.
        let external_registry = Arc::clone(&self.external_documents);
        let reservation_registry = Arc::clone(&self.external_reservations);
        let external = external_registry.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = reservation_registry.lock().unwrap_or_else(|e| e.into_inner());
        let final_limits = reservations.constrain(self.limits);
        let mut final_peak = self.validate_retained_owners_with_external_except(
            &self.identities, None, false, &final_limits, &external, &reservations.accounted_classes(0)?,
            staged.documents.get(&incoming_id),
        )?;
        final_peak.add(additional, &final_limits)?;
        self.world = staged;
        self.identities = identities;
        self.epoch = next;
        Ok(())
    }
    pub fn close_document(&mut self, id: DocumentId) -> Result<()> {
        self.available()?;
        if !self.world.documents.contains_key(&id) {
            return Err(Error::Missing("document"));
        }
        let next = bump(self.epoch, "publication epoch")?;
        // Explicit close cancels caller staging; it cannot resurrect closed owners.
        self.scope = None;
        self.world.documents.remove(&id);
        self.undo
            .retain(|e| !e.before.contains_key(&id) && !e.after.contains_key(&id));
        self.redo
            .retain(|e| !e.before.contains_key(&id) && !e.after.contains_key(&id));
        self.epoch = next;
        Ok(())
    }
    fn request_world(&self, owner: Option<&ScopeToken>) -> Result<(&World, Option<u64>)> {
        match (self.scope.as_ref(), owner) {
            (None, None) => Ok((&self.world, None)),
            (Some(_), Some(token)) => {
                let s = self.owner(token)?;
                if !s.active {
                    return Err(Error::Invalid("inactive scope"));
                }
                Ok((&s.staged, Some(s.version)))
            }
            _ => Err(Error::WrongOwner),
        }
    }
    fn check_retention(
        &self,
        entry: &UndoEntry,
        recording: bool,
        identities: &IdentityLedger,
        limits: &Limits,
    ) -> Result<()> {
        if entry.empty() {
            return Ok(());
        }
        self.validate_retained_owners(identities, Some(entry), recording, limits)?;
        Ok(())
    }
    fn saved_worlds(&self) -> impl Iterator<Item = &World> {
        std::iter::once(&self.world).chain(self.scope.iter().flat_map(|scope| {
            std::iter::once(&scope.baseline)
                .chain(std::iter::once(&scope.staged))
                .chain(&scope.savepoints)
        }))
    }
    fn retained_documents<'a>(
        &'a self,
        entry: Option<&'a UndoEntry>,
    ) -> impl Iterator<Item = &'a Arc<Document>> {
        self.saved_worlds()
            .flat_map(|world| world.documents.values())
            .chain(
                self.undo
                    .iter()
                    .chain(&self.redo)
                    .chain(entry)
                    .flat_map(|e| e.before.values().chain(e.after.values())),
            )
    }
    fn unique_retained_documents<'a>(
        &'a self,
        entry: Option<&'a UndoEntry>,
    ) -> impl Iterator<Item = &'a Arc<Document>> {
        self.retained_documents(entry)
            .enumerate()
            .filter(move |(index, document)| {
                !self
                    .retained_documents(entry)
                    .take(*index)
                    .any(|prior| Arc::ptr_eq(prior, document))
            })
            .map(|(_, document)| document)
    }
    fn retained_metadata_usage(
        &self,
        identities: &IdentityLedger,
        entry: Option<&UndoEntry>,
        recording: bool,
        limits: &Limits,
    ) -> Result<Usage> {
        // This pass allocates nothing. All distinct immutable document owners,
        // metadata and old Redo remain live while any document validates.
        let shared = |ledger: &IdentityLedger| self.shared_peer_identities.as_ref()
            .is_some_and(|owned| Arc::ptr_eq(&owned.0, &ledger.0));
        let mut usage = if shared(identities) { Usage::default() } else { identities.usage()? };
        if !Arc::ptr_eq(&identities.0, &self.identities.0) {
            if !shared(&self.identities) { usage.add(self.identities.usage()?, limits)?; }
        }
        usage.add(Usage::array::<Self>(1)?, limits)?;
        usage.add(Usage::array::<UndoEntry>(self.undo.capacity())?, limits)?;
        usage.add(Usage::array::<UndoEntry>(self.redo.capacity())?, limits)?;
        if entry.is_some() && recording {
            let next = self
                .undo
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(2))
                .ok_or(Error::Overflow("Undo allocation"))?
                .max(4);
            usage.add(Usage::array::<UndoEntry>(next)?, limits)?;
        }
        if let Some(scope) = &self.scope {
            usage.add(
                Usage {
                    bytes: scope.label.capacity() as u64,
                    ..Usage::default()
                },
                limits,
            )?;
            usage.add(Usage::array::<World>(scope.savepoints.capacity())?, limits)?;
        }
        for world in self.saved_worlds() {
            usage.add(
                Usage::btree::<DocumentId, Arc<Document>>(world.documents.len())?,
                limits,
            )?;
        }
        for retained in self.undo.iter().chain(&self.redo) {
            usage.add(retained.storage_usage(limits)?, limits)?;
            usage.add(
                Usage {
                    history: 1,
                    ..Usage::default()
                },
                limits,
            )?;
        }
        if let Some(entry) = entry {
            usage.add(entry.storage_usage(limits)?, limits)?;
            // The independently prepared candidate World map stays alive with
            // its private entry until acceptance, even when recording is off.
            usage.add(
                Usage::btree::<DocumentId, Arc<Document>>(self.world.documents.len())?,
                limits,
            )?;
            if recording {
                usage.add(
                    Usage {
                        history: 1,
                        ..Usage::default()
                    },
                    limits,
                )?;
            }
        }
        Ok(usage)
    }
    fn retained_storage_usage(
        &self,
        identities: &IdentityLedger,
        entry: Option<&UndoEntry>,
        recording: bool,
        limits: &Limits,
    ) -> Result<Usage> {
        let mut usage = self.retained_metadata_usage(identities, entry, recording, limits)?;
        for document in self.unique_retained_documents(entry) {
            add_retained_document_usage(&mut usage, self.retained_document_storage(document, entry, limits)?, limits)?;
            usage.add(Usage::array::<usize>(2)?, limits)?;
        }
        Ok(usage)
    }

    /// Charge immutable resource payloads shared across document versions once.
    /// If a single document aliases the same payload through multiple resource IDs,
    /// retain the largest per-document multiplicity so its full validation remains
    /// conservatively admitted. This allocation-free pass also works near the limit.
    fn retained_document_storage(
        &self,
        document: &Arc<Document>,
        entry: Option<&UndoEntry>,
        limits: &Limits,
    ) -> Result<Usage> {
        let mut usage = document.storage_usage(limits)?;
        for (index, resource) in document.resources.values().enumerate() {
            let payload = resource.payload()?;
            let same = |other: &crate::resource::Resource| {
                other.payload().is_ok_and(|bytes| std::ptr::eq(bytes, payload))
            };
            let rank = document.resources.values().take(index + 1).filter(|item| same(item)).count();
            let max_prior = self.retained_documents(entry)
                .take_while(|prior| !Arc::ptr_eq(prior, document))
                .map(|prior| prior.resources.values().filter(|item| same(item)).count())
                .max().unwrap_or(0);
            if rank <= max_prior {
                usage.bytes = usage.bytes.checked_sub(payload.len() as u64)
                    .ok_or(Error::Overflow("shared resource accounting"))?;
            }
        }
        // ponytail: allocation-free fallback scans earlier owners quadratically;
        // use the admitted index on the normal path for large histories.
        for (index, brep) in retained_breps(document).enumerate() {
            let key = brep.source_allocation_key();
            if retained_breps(document).take(index).any(|other| other.source_allocation_key() == key) { continue; }
            let previously_owned = self.retained_documents(entry)
                .take_while(|prior| !Arc::ptr_eq(prior, document))
                .any(|prior| retained_breps(prior).any(|other| other.source_allocation_key() == key));
            if previously_owned {
                usage.bytes = usage.bytes.checked_sub(brep.source_allocation_bytes()?)
                    .ok_or(Error::Overflow("shared BRep source accounting"))?;
            }
        }
        for (index, brep) in retained_native_breps(document).enumerate() {
            let key = brep.payload_allocation_key();
            if retained_native_breps(document).take(index).any(|other| other.payload_allocation_key() == key) { continue; }
            if self.retained_documents(entry).take_while(|prior| !Arc::ptr_eq(prior, document))
                .any(|prior| retained_native_breps(prior).any(|other| other.payload_allocation_key() == key)) {
                usage.bytes = usage.bytes.checked_sub(brep.payload_allocation_bytes()?)
                    .ok_or(Error::Overflow("shared native BRep payload accounting"))?;
            }
        }
        for (index, brep) in retained_breps(document).enumerate() {
            let key = brep.display_allocation_key();
            if retained_breps(document).take(index).any(|other| other.display_allocation_key() == key) { continue; }
            if self.retained_documents(entry).take_while(|prior| !Arc::ptr_eq(prior, document))
                .any(|prior| retained_breps(prior).any(|other| other.display_allocation_key() == key)) {
                usage.bytes = usage.bytes.checked_sub(brep.display_allocation_bytes()?)
                    .ok_or(Error::Overflow("shared BRep display accounting"))?;
            }
        }
        Ok(usage)
    }

    fn indexed_retained_document_storage(
        document: &Arc<Document>,
        limits: &Limits,
        seen: &mut BTreeMap<(usize, usize), usize>,
        brep_seen: &mut BTreeSet<usize>,
        display_seen: &mut BTreeSet<usize>,
        native_seen: &mut BTreeSet<usize>,
    ) -> Result<Usage> {
        let mut usage = document.storage_usage(limits)?;
        for (index, resource) in document.resources.values().enumerate() {
            let payload = resource.payload()?;
            let key = (payload.as_ptr() as usize, payload.len());
            let rank = document.resources.values().take(index + 1)
                .filter(|item| item.payload().is_ok_and(|bytes| std::ptr::eq(bytes, payload))).count();
            let maximum = seen.entry(key).or_default();
            if rank <= *maximum {
                usage.bytes = usage.bytes.checked_sub(payload.len() as u64)
                    .ok_or(Error::Overflow("shared resource accounting"))?;
            } else {
                *maximum = rank;
            }
        }
        for (index, brep) in retained_breps(document).enumerate() {
            let key = brep.source_allocation_key();
            if retained_breps(document).take(index).any(|other| other.source_allocation_key() == key) { continue; }
            if !brep_seen.insert(key) {
                usage.bytes = usage.bytes.checked_sub(brep.source_allocation_bytes()?)
                    .ok_or(Error::Overflow("shared BRep source accounting"))?;
            }
        }
        for (index, brep) in retained_native_breps(document).enumerate() {
            let key = brep.payload_allocation_key();
            if retained_native_breps(document).take(index).any(|other| other.payload_allocation_key() == key) { continue; }
            if !native_seen.insert(key) {
                usage.bytes = usage.bytes.checked_sub(brep.payload_allocation_bytes()?)
                    .ok_or(Error::Overflow("shared native BRep payload accounting"))?;
            }
        }
        for (index, brep) in retained_breps(document).enumerate() {
            let key = brep.display_allocation_key();
            if retained_breps(document).take(index).any(|other| other.display_allocation_key() == key) { continue; }
            if !display_seen.insert(key) {
                usage.bytes = usage.bytes.checked_sub(brep.display_allocation_bytes()?)
                    .ok_or(Error::Overflow("shared BRep display accounting"))?;
            }
        }
        Ok(usage)
    }

    /// Reuse a successful validation only while the exact immutable Arc is
    /// still owned by an Undo/Redo entry and the current limits are no tighter.
    fn validate_retained_document(
        &self,
        document: &Arc<Document>,
        entry: Option<&UndoEntry>,
        storage: Usage,
        limits: &Limits,
    ) -> Result<Usage> {
        let owner = self.undo.iter().chain(&self.redo).chain(entry).find(|owner| {
            owner
                .before
                .values()
                .chain(owner.after.values())
                .any(|d| Arc::ptr_eq(d, document))
        });
        storage.check(limits)?;
        let covers = |old: &Limits| {
            limits.max_bytes >= old.max_bytes
                && limits.max_objects >= old.max_objects
                && limits.max_nodes >= old.max_nodes
                && limits.max_points >= old.max_points
                && limits.max_history >= old.max_history
                && limits.max_outputs >= old.max_outputs
                && limits.max_depth >= old.max_depth
                && limits.max_degree >= old.max_degree
                && limits.max_name_bytes >= old.max_name_bytes
        };
        let Some(owner) = owner else {
            let mut cache = self
                .current_validation_cache
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if cache.as_ref().is_some_and(|(weak, proved)| {
                weak.ptr_eq(&Arc::downgrade(document)) && covers(proved)
            }) {
                return Ok(storage);
            }
            let halfway = |max: u64, used: u64| used + (max - used) / 2;
            let tighter = Limits {
                max_bytes: halfway(limits.max_bytes, storage.bytes),
                max_objects: halfway(limits.max_objects, storage.objects),
                max_nodes: halfway(limits.max_nodes, storage.nodes),
                max_points: halfway(limits.max_points, storage.points),
                max_history: halfway(limits.max_history, storage.history),
                max_outputs: halfway(limits.max_outputs, storage.outputs),
                ..*limits
            };
            let (validated, proved) = match document.validate(&tighter) {
                Ok(usage) => (usage, tighter),
                Err(_) => (document.validate(limits)?, *limits),
            };
            *cache = Some((Arc::downgrade(document), proved));
            return Ok(validated);
        };
        let address = Arc::as_ptr(document) as usize;
        let mut slots = *owner.validation_cache.0.lock().unwrap_or_else(|error| error.into_inner());
        if slots
            .iter()
            .flatten()
            .any(|p| p.address == address && covers(&p.limits))
        {
            return Ok(storage);
        }
        // Leave half of the available headroom for later history growth. If
        // tighter validation refuses, retry with the exact caller limits.
        let halfway = |max: u64, used: u64| used + (max - used) / 2;
        let tighter = Limits {
            max_bytes: halfway(limits.max_bytes, storage.bytes),
            max_objects: halfway(limits.max_objects, storage.objects),
            max_nodes: halfway(limits.max_nodes, storage.nodes),
            max_points: halfway(limits.max_points, storage.points),
            max_history: halfway(limits.max_history, storage.history),
            max_outputs: halfway(limits.max_outputs, storage.outputs),
            ..*limits
        };
        let (validated, proved_limits) = match document.validate(&tighter) {
            Ok(usage) => (usage, tighter),
            Err(_) => (document.validate(limits)?, *limits),
        };
        debug_assert_eq!(validated, storage);
        let slot = slots
            .iter()
            .position(|p| p.is_some_and(|p| p.address == address))
            .or_else(|| slots.iter().position(Option::is_none))
            .unwrap_or(0);
        slots[slot] = Some(RetainedValidationProof {
            address,
            limits: proved_limits,
        });
        *owner.validation_cache.0.lock().unwrap_or_else(|error| error.into_inner()) = slots;
        Ok(validated)
    }
    fn validate_retained_owners(
        &self,
        identities: &IdentityLedger,
        entry: Option<&UndoEntry>,
        recording: bool,
        limits: &Limits,
    ) -> Result<Usage> {
        let external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let effective = reservations.constrain(*limits);
        self.validate_retained_owners_with_external(identities, entry, recording, &effective, &external, &reservations.accounted_classes(0)?)
    }

    fn validate_retained_owners_with_external(
        &self,
        identities: &IdentityLedger,
        entry: Option<&UndoEntry>,
        recording: bool,
        limits: &Limits,
        external: &Vec<Weak<Document>>,
        reservations: &[u64; 6],
    ) -> Result<Usage> {
        self.validate_retained_owners_with_external_except(identities, entry, recording, limits, external, reservations, None)
    }
    /// The excluded candidate is already charged in the caller's separately
    /// admitted preparation peak. All other external owners remain charged.
    fn validate_retained_owners_with_external_except(
        &self,
        identities: &IdentityLedger,
        entry: Option<&UndoEntry>,
        recording: bool,
        limits: &Limits,
        external: &Vec<Weak<Document>>,
        reservations: &[u64; 6],
        excluded: Option<&Arc<Document>>,
    ) -> Result<Usage> {
        let held = self.external_storage_usage(entry, limits, external, reservations, excluded)?;
        {
            let mut proof = self.current_validation_cache.lock().unwrap_or_else(|e| e.into_inner());
            if proof.as_ref().is_some_and(|(weak, _)| weak.strong_count() == 0) {
                *proof = None;
            }
        }
        let reduced = limits.remaining_after(held)?;
        let mut usage = self.validate_internal_retained_owners(identities, entry, recording, &reduced)?;
        usage.add(held, limits)?;
        Ok(usage)
    }
    /// Storage projection only: no validation, proof updates or reclamation.
    fn external_storage_usage(
        &self, entry: Option<&UndoEntry>, limits: &Limits,
        external: &Vec<Weak<Document>>, reservations: &[u64; 6],
        excluded: Option<&Arc<Document>>,
    ) -> Result<Usage> {
        let mut held = Usage::array::<Weak<Document>>(external.capacity())?;
        for bytes in reservations {
            held.add(Usage { bytes: *bytes, ..Usage::default() }, limits)?;
        }
        let weak_block = Usage { bytes: (std::mem::size_of::<Document>() + 2 * std::mem::size_of::<usize>()) as u64,
            ..Usage::default() };
        // A Weak keeps the entire Arc allocation (but not Document's dropped
        // heap payload) until its last handle goes away. Upgrade once per
        // entry so a concurrent last strong drop cannot skip both branches.
        for (external_index, weak) in external.iter().enumerate() {
            if external.iter().take(external_index).any(|prior| prior.ptr_eq(weak)) { continue; }
            let Some(document) = weak.upgrade() else {
                held.add(weak_block, limits)?;
                continue;
            };
            if excluded.is_some_and(|candidate| Arc::ptr_eq(candidate, &document)) { continue; }
            if !self.retained_documents(entry).any(|owned| Arc::ptr_eq(owned, &document)) {
                // This is an already-validated immutable document version. Its
                // bytes remain live, while structural point limits are per document.
                let mut storage = document.storage_usage(limits)?;
                for (resource_index, resource) in document.resources.values().enumerate() {
                    let payload = resource.payload()?;
                    let same = |other: &Resource| other.payload().is_ok_and(|bytes| std::ptr::eq(bytes, payload));
                    let rank = document.resources.values().take(resource_index + 1).filter(|item| same(item)).count();
                    let internal_max = self.retained_documents(entry)
                        .map(|prior| prior.resources.values().filter(|item| same(item)).count()).max().unwrap_or(0);
                    let external_max = external.iter().take(external_index).filter_map(Weak::upgrade)
                        .map(|prior| prior.resources.values().filter(|item| same(item)).count()).max().unwrap_or(0);
                    if rank <= internal_max.max(external_max) {
                        storage.bytes = storage.bytes.checked_sub(payload.len() as u64)
                            .ok_or(Error::Overflow("shared external resource accounting"))?;
                    }
                }
                for (brep_index, brep) in retained_breps(&document).enumerate() {
                    let key = brep.source_allocation_key();
                    if retained_breps(&document).take(brep_index).any(|other| other.source_allocation_key() == key) { continue; }
                    let internal_has = self.retained_documents(entry)
                        .any(|prior| retained_breps(prior).any(|other| other.source_allocation_key() == key));
                    let external_has = external.iter().take(external_index).filter_map(Weak::upgrade)
                        .any(|prior| retained_breps(&prior).any(|other| other.source_allocation_key() == key));
                    if internal_has || external_has {
                        storage.bytes = storage.bytes.checked_sub(brep.source_allocation_bytes()?)
                            .ok_or(Error::Overflow("shared external BRep source accounting"))?;
                    }
                }
                for (index, brep) in retained_native_breps(&document).enumerate() {
                    let key = brep.payload_allocation_key();
                    if retained_native_breps(&document).take(index).any(|other| other.payload_allocation_key() == key) { continue; }
                    let internal_has = self.retained_documents(entry).any(|prior| retained_native_breps(prior).any(|other| other.payload_allocation_key() == key));
                    let external_has = external.iter().take(external_index).filter_map(Weak::upgrade)
                        .any(|prior| retained_native_breps(&prior).any(|other| other.payload_allocation_key() == key));
                    if internal_has || external_has {
                        storage.bytes = storage.bytes.checked_sub(brep.payload_allocation_bytes()?)
                            .ok_or(Error::Overflow("shared external native BRep payload accounting"))?;
                    }
                }
                for (index, brep) in retained_breps(&document).enumerate() {
                    let key = brep.display_allocation_key();
                    if retained_breps(&document).take(index).any(|other| other.display_allocation_key() == key) { continue; }
                    let internal_has = self.retained_documents(entry).any(|prior| retained_breps(prior).any(|other| other.display_allocation_key() == key));
                    let external_has = external.iter().take(external_index).filter_map(Weak::upgrade)
                        .any(|prior| retained_breps(&prior).any(|other| other.display_allocation_key() == key));
                    if internal_has || external_has {
                        storage.bytes = storage.bytes.checked_sub(brep.display_allocation_bytes()?)
                            .ok_or(Error::Overflow("shared external BRep display accounting"))?;
                    }
                }
                held.add(Usage { bytes: storage.bytes, ..Usage::default() }, limits)?;
                held.add(Usage::array::<usize>(2)?, limits)?;
            }
        }
        Ok(held)
    }

    fn validate_internal_retained_owners(
        &self,
        identities: &IdentityLedger,
        entry: Option<&UndoEntry>,
        recording: bool,
        limits: &Limits,
    ) -> Result<Usage> {
        let count = self.retained_documents(entry).try_fold(0usize, |n, _| {
            n.checked_add(1)
                .ok_or(Error::Overflow("retained Arc owners"))
        })?;
        let index = Usage::btree::<usize, ()>(count)?;
        let resource_slots = self.retained_documents(entry).try_fold(0usize, |n, document| {
            n.checked_add(document.resources.len()).ok_or(Error::Overflow("retained resource owners"))
        })?;
        let resource_index = if resource_slots == 0 { Usage::default() }
            else { Usage::btree::<(usize, usize), usize>(resource_slots)? };
        let brep_slots = self.retained_documents(entry).try_fold(0usize, |n, document| {
            n.checked_add(retained_breps(document).count()).ok_or(Error::Overflow("retained BRep source owners"))
        })?;
        let brep_index = if brep_slots == 0 { Usage::default() }
            else { Usage::btree::<usize, ()>(brep_slots)? };
        let display_index = if brep_slots == 0 { Usage::default() }
            else { Usage::btree::<usize, ()>(brep_slots)? };
        let native_slots = self.retained_documents(entry).try_fold(0usize, |n, document| {
            n.checked_add(retained_native_breps(document).count()).ok_or(Error::Overflow("retained native BRep payload owners"))
        })?;
        let native_index = if native_slots == 0 { Usage::default() }
            else { Usage::btree::<usize, ()>(native_slots)? };
        let metadata = self.retained_metadata_usage(identities, entry, recording, limits)?;
        let conservative = (|| -> Result<()> {
            let mut upper = metadata;
            upper.add(index, limits)?;
            upper.add(resource_index, limits)?;
            upper.add(brep_index, limits)?;
            upper.add(display_index, limits)?;
            upper.add(native_index, limits)?;
            // A duplicate-inclusive bound is nonallocating and dominates the
            // exact live owner inventory. Only then may the dedup index grow.
            for document in self.retained_documents(entry) {
                add_retained_document_usage(&mut upper, document.storage_usage(limits)?, limits)?;
                upper.add(Usage::array::<usize>(2)?, limits)?;
            }
            Ok(())
        })();
        match conservative {
            Ok(()) => {
                let mut shared = BTreeSet::new();
                let mut shared_payloads = BTreeMap::new();
                let mut shared_breps = BTreeSet::new();
                let mut shared_displays = BTreeSet::new();
                let mut shared_native = BTreeSet::new();
                let mut usage = metadata;
                for document in self.retained_documents(entry) {
                    if shared.insert(Arc::as_ptr(document) as usize) {
                        add_retained_document_usage(&mut usage, Self::indexed_retained_document_storage(document, limits, &mut shared_payloads, &mut shared_breps, &mut shared_displays, &mut shared_native)?, limits)?;
                        usage.add(Usage::array::<usize>(2)?, limits)?;
                    }
                }
                let indexed_limits = limits.remaining_after(index)?.remaining_after(resource_index)?.remaining_after(brep_index)?.remaining_after(display_index)?.remaining_after(native_index)?;
                let attempt = (|| -> Result<()> {
                    for document in self.retained_documents(entry) {
                        if shared.remove(&(Arc::as_ptr(document) as usize)) {
                            let storage = document.storage_usage(limits)?;
                            let remaining = validation_limits(&indexed_limits, usage, storage)?;
                            let validated = self.validate_retained_document(
                                document, entry, storage, &remaining,
                            )?;
                            debug_assert_eq!(validated, storage);
                        }
                    }
                    Ok(())
                })();
                match attempt {
                    Ok(()) => return Ok(usage),
                    Err(Error::Budget(_)) => { /* retry without the temporary index */ }
                    Err(error) => return Err(error),
                }
            }
            Err(Error::Budget(_)) => { /* exact allocation-free admission below */ }
            Err(error) => return Err(error),
        }
        // Under tight limits even a conservative pointer index may not fit.
        // Exact borrowed pointer comparisons preserve sharing without allocation
        // or order-dependent subtraction of only previously visited owners.
        let usage = self.retained_storage_usage(identities, entry, recording, limits)?;
        for document in self.unique_retained_documents(entry) {
            let storage = document.storage_usage(limits)?;
            let remaining = validation_limits(limits, usage, storage)?;
            let validated =
                self.validate_retained_document(document, entry, storage, &remaining)?;
            debug_assert_eq!(validated, storage);
        }
        Ok(usage)
    }
    pub fn prepare(&self, request: TransactionRequest) -> Result<PreparedTransaction> {
        self.prepare_with_held_usage(request, Usage::default())
    }
    /// Admit external scene/resource allocation while a private candidate and
    /// its preparation reservation are still retained. This is conservative:
    /// scene owners may count the shared candidate document a second time.
    pub fn prepared_remaining_limits(
        &self,
        prepared: &PreparedTransaction,
        held: Usage,
    ) -> Result<Limits> {
        self.available()?;
        self.verify(prepared)?;
        // A prepared document's structural point quota is independent of a
        // candidate scene's quota. Keep the reservation's bytes and other
        // counters charged while planning that separately bounded scene.
        let mut reservation = prepared.reservation;
        reservation.points = 0;
        self.admission_limits()
            .remaining_after(held)?
            .remaining_after(reservation)
    }
    /// Register the exact already-validated candidate allocation so peers on
    /// this memory account count it throughout dependent-scene construction.
    /// The weak registration expires when the prepared transaction is dropped.
    pub fn register_prepared_document(
        &self,
        prepared: &PreparedTransaction,
        id: DocumentId,
    ) -> Result<Arc<Document>> {
        self.available()?;
        self.verify(prepared)?;
        let document = prepared.staged.documents.get(&id)
            .ok_or(Error::Missing("prepared document"))?.clone();
        self.register_external_document(&document)?;
        self.verify(prepared)?;
        Ok(document)
    }
    /// Replace an operation's external metadata reservation without changing its
    /// acceptance authority. The original conservative preparation peak covers
    /// the immutable candidate, saved states, private Undo and preparation maps.
    pub fn reserve_prepared_held_usage(
        &self,
        prepared: &mut PreparedTransaction,
        held: Usage,
    ) -> Result<()> {
        self.available()?;
        self.verify(prepared)?;
        let limits = self.admission_limits().remaining_after(held)?;
        // Admission precedes any retention-validation scratch allocation. The
        // candidate is immutable and was structurally validated during prepare.
        prepared.reservation.check(&limits)?;
        self.check_retention(
            &prepared.entry,
            prepared.recording,
            &prepared.identities,
            &limits,
        )?;
        self.verify(prepared)?;
        prepared.held = held;
        Ok(())
    }
    /// Reserve external receipt/input owners alongside all prospective state.
    /// Mutation-owned input is admitted separately; do not count it twice.
    pub fn prepare_with_held_usage(
        &self,
        request: TransactionRequest,
        held: Usage,
    ) -> Result<PreparedTransaction> {
        self.prepare_with_held_usage_and_guard(request, held, || Ok(()))
    }
    fn prepare_with_held_usage_and_guard(
        &self,
        request: TransactionRequest,
        held: Usage,
        before_clone: impl FnOnce() -> Result<()>,
    ) -> Result<PreparedTransaction> {
        self.prepare_with_held_usage_and_guards(request, held, || Ok(()), before_clone)
    }
    fn prepare_with_held_usage_and_guards(
        &self,
        request: TransactionRequest,
        held: Usage,
        before_planning: impl FnOnce() -> Result<()>,
        before_clone: impl FnOnce() -> Result<()>,
    ) -> Result<PreparedTransaction> {
        self.available()?;
        request.cancel.check()?;
        let limits = self.admission_limits().remaining_after(held)?;
        name(&request.label, &limits)?;
        let (base, scope_version) = self.request_world(request.owner.as_ref())?;
        let request_storage = request.storage_usage(&limits)?;
        let mut peak =
            self.retained_usage_with_limits(&limits.remaining_after(request_storage)?)?;
        peak.add(request_storage, &limits)?;
        // Admit temporary ownership maps and the eventual private Undo entry
        // before their first insert or any candidate document clone. Import
        // bundles can propose many identities in a single mutation.
        let mut identities_bound = request.mutations.len();
        let mut writes_bound = request.mutations.len();
        let mut created_objects_bound = 0usize;
        for mutation in &request.mutations {
            let created = match mutation {
                Mutation::Insert { .. } => 1,
                Mutation::DefinitionInsert { definition, instances, .. } => definition.members.len()
                    .checked_add(instances.len()).ok_or(Error::Overflow("created object scratch"))?,
                Mutation::Publish(result) => result.outputs.len(),
                Mutation::Import { bundle, .. } => bundle.definitions.iter().try_fold(bundle.entities.len(), |n, d| {
                    n.checked_add(d.members.len()).ok_or(Error::Overflow("created object scratch"))
                })?,
                _ => 0,
            };
            created_objects_bound = created_objects_bound.checked_add(created)
                .ok_or(Error::Overflow("created object scratch"))?;
            if let Mutation::Layers(plan) = mutation {
                writes_bound = writes_bound
                    .checked_add(plan.assigned_sources().len())
                    .ok_or(Error::Overflow("layer assignment scratch"))?;
            }
            if let Mutation::Import { bundle, .. } = mutation {
                let members = bundle.definitions.iter().try_fold(0usize, |n, d| {
                    n.checked_add(d.members.len())
                        .ok_or(Error::Overflow("import scratch"))
                })?;
                let added = bundle
                    .entities
                    .len()
                    .checked_add(members)
                    .and_then(|n| n.checked_add(bundle.layers.len()))
                    .and_then(|n| n.checked_add(bundle.definitions.len()))
                    .and_then(|n| n.checked_add(bundle.groups.len()))
                    .and_then(|n| n.checked_add(bundle.exchange.line_patterns.len()))
                    .and_then(|n| n.checked_add(3))
                    .ok_or(Error::Overflow("import scratch"))?;
                identities_bound = identities_bound
                    .checked_add(added)
                    .ok_or(Error::Overflow("import scratch"))?;
                writes_bound = writes_bound
                    .checked_add(bundle.entities.len())
                    .ok_or(Error::Overflow("import writes scratch"))?;
            }
        }
        let count = request.mutations.len();
        let documents = base.documents.len();
        let changed = count
            .checked_mul(2)
            .ok_or(Error::Overflow("transaction participants"))?
            .min(documents);
        let keys = documents
            .checked_mul(2)
            .ok_or(Error::Overflow("Undo keys"))?;
        let recording = self
            .scope
            .as_ref()
            .map_or(request.recording && !self.benchmark_no_undo, |s| s.recording);
        let potential_changes = request.mutations.iter().any(|mutation| match mutation {
            Mutation::Layers(plan) => plan.changed(),
            _ => true,
        });
        let undo_slots = if recording && potential_changes {
            self.undo
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(2))
                .ok_or(Error::Overflow("Undo allocation"))?
                .max(4)
        } else {
            0
        };
        let map_slots = changed.checked_mul(4).ok_or(Error::Overflow("Undo maps"))?;
        for storage in [
            Usage::btree::<DocumentId, ()>(changed)?,
            Usage::btree::<DocumentId, ()>(changed)?,
            Usage::btree::<(DocumentId, ObjectId, &'static str), ()>(writes_bound)?,
            Usage::btree::<(DocumentId, LayerId), ()>(count)?,
            Usage::btree::<(DocumentId, crate::layers::PresetSlot), ()>(count)?,
            Usage::btree::<(DocumentId, LayerId), &crate::model::Layer>(count)?,
            Usage::btree::<DocumentId, ()>(changed)?,
            Usage::btree::<(DocumentId, ResourceId), ()>(count)?,
            Usage::btree::<ResourceId, ()>(count)?,
            Usage::btree::<DocumentId, usize>(changed)?,
            Usage::btree::<DocumentId, (usize, usize, usize)>(changed)?,
            Usage::btree::<AllocatedIdentity, ()>(identities_bound)?,
            Usage::btree::<DocumentId, Arc<Document>>(documents)?,
            Usage::btree::<DocumentId, ()>(documents)?,
            Usage::array::<DocumentId>(keys.max(4))?,
            Usage::array::<(DocumentId, Arc<Document>)>(map_slots)?,
            Usage::btree::<DocumentId, Arc<Document>>(changed)?,
            Usage::btree::<DocumentId, Arc<Document>>(changed)?,
            Usage::array::<PreparedTransaction>(1)?,
            Usage::array::<UndoEntry>(undo_slots)?,
        ] {
            peak.add(storage, &limits)?;
        }
        // Reserve the known temporary BTree indexes before their first insert.
        // Other validation scratch and candidate growth require separate admission.
        let object_slots = identities_bound.checked_add(created_objects_bound)
            .ok_or(Error::Overflow("transaction planning indexes"))?;
        let mut planning = Usage::default();
        let planning_limits = self.admission_limits();
        for storage in [
            Usage::btree::<DocumentId, ()>(changed)?,
            Usage::btree::<(DocumentId, ObjectId, &'static str), ()>(writes_bound)?,
            Usage::btree::<(DocumentId, ResourceId), ()>(count)?,
            Usage::btree::<ResourceId, ()>(count)?,
            Usage::btree::<DocumentId, ()>(changed)?,
            Usage::btree::<DocumentId, usize>(changed)?,
            Usage::btree::<DocumentId, (usize, usize, usize)>(changed)?,
            Usage::btree::<(DocumentId, LayerId), ()>(count)?,
            Usage::btree::<(DocumentId, crate::layers::PresetSlot), ()>(count)?,
            Usage::btree::<(DocumentId, LayerId), &crate::model::Layer>(count)?,
            Usage::btree::<(DocumentId, ObjectId), ()>(created_objects_bound)?,
            Usage::btree::<DocumentId, ()>(changed)?,
            Usage::btree::<AllocatedIdentity, ()>(object_slots)?,
            Usage::btree::<(DocumentId, GroupId), ()>(count)?,
        ] { planning.add(storage, &planning_limits)?; }
        let planning_reservation = if count == 0 { None } else {
            let mut live = held;
            live.add(request_storage, &self.admission_limits())?;
            let physical = self.physical_memory_sampler.and_then(|sample| sample());
            Some(self.reserve_external_memory_with_held(
                crate::budget::memory::MemoryClass::CpuCache, planning.bytes, physical, live,
            )?)
        };
        before_planning()?;
        // These owners are declared after their guard, so error and cancellation
        // paths drop the actual indexes before releasing the reservation.
        let mut participants = BTreeSet::new();
        let mut writes = BTreeSet::new();
        let mut resource_writes = BTreeSet::new();
        let mut inserted_resources = BTreeSet::new();
        let mut exchange_writes = BTreeSet::new();
        let mut resource_insert_counts = BTreeMap::<DocumentId, usize>::new();
        let mut import_counts = BTreeMap::<DocumentId, (usize, usize, usize)>::new();
        let mut layer_writes = BTreeSet::new();
        let mut created_preset_slots = BTreeSet::new();
        let mut created_layers = BTreeMap::new();
        let mut created_objects = BTreeSet::new();
        let mut current_writes = BTreeSet::new();
        let mut proposed_identities = BTreeSet::new();
        let mut group_writes = BTreeSet::new();
        // Borrow only explicit Create plans, never arbitrary metadata Updates.
        // Every proposed creator still passes its normal admission below; this
        // lookup lets a lazy Assign reference its new target atomically in any
        // request order without using an earlier write's old permission.
        for mutation in &request.mutations {
            request.cancel.check()?;
            if let Mutation::Layers(plan) = mutation
                && let Some(layer) = plan.created_layer()
                && created_layers
                    .insert((plan.document(), layer.id), layer)
                    .is_some()
            {
                return Err(Error::Invalid("duplicate proposed layer creation"));
            }
            // Group and feature membership may reference an object created by
            // the same request in any order, so the proposed objects are
            // collected before any admission decision.
            match mutation {
                Mutation::Insert { document, entity } => {
                    created_objects.insert((*document, entity.id));
                }
                Mutation::DefinitionInsert {
                    document,
                    definition,
                    instances,
                    ..
                } => {
                    for member in definition.members.iter().chain(instances) {
                        created_objects.insert((*document, member.id));
                    }
                }
                Mutation::Publish(result) => {
                    for output in &result.outputs {
                        if output.expected_existing.is_none() {
                            created_objects.insert((result.document, output.entity.id));
                        }
                    }
                }
                Mutation::Import {
                    document, bundle, ..
                } => {
                    for entity in bundle
                        .entities
                        .iter()
                        .chain(bundle.definitions.iter().flat_map(|d| &d.members))
                    {
                        created_objects.insert((*document, entity.id));
                    }
                }
                _ => {}
            }
        }
        // Freeze and admit every source and target before any document clone.
        for mutation in &request.mutations {
            request.cancel.check()?;
            for id in mutation.participants() {
                base.document(id)?;
                participants.insert(id);
            }
            if let Some(source) = mutation.source() {
                let d = base.document(source.document)?;
                d.resolve(&source)?;
                if !d.permissions(source.object, false)?.editable {
                    return Err(Error::Permission("source edit"));
                }
                if !writes.insert((source.document, source.object, mutation.write_field())) {
                    return Err(Error::Invalid("conflicting source writes"));
                }
                if matches!(
                    mutation,
                    Mutation::Geometry { .. } | Mutation::Placement { .. } | Mutation::Move { .. }
                ) && d.history.locked(source.object)
                {
                    return Err(Error::Permission("History child geometry lock"));
                }
            }
            match mutation {
                Mutation::Layers(plan) => {
                    let (context, _) = self.capture_context(request.owner.as_ref())?;
                    plan.admit(
                        context,
                        base.document(plan.document())?,
                        plan.assignment_target()
                            .and_then(|id| created_layers.get(&(plan.document(), id)).copied()),
                        &created_layers,
                        &limits,
                        &mut peak,
                        &request.cancel,
                    )?;
                    if let Some(id) = plan.created_id()
                        && (self
                            .identities
                            .contains_identity(AllocatedIdentity::Layer(id))
                            || !proposed_identities.insert(AllocatedIdentity::Layer(id)))
                    {
                        return Err(Error::Invalid("allocated layer UUID"));
                    }
                    if plan.created_id().is_some()
                        && let Some(slot) =
                            plan.proposed_layer().and_then(|layer| layer.preset_slot)
                        && !created_preset_slots.insert((plan.document(), slot))
                    {
                        return Err(Error::Invalid("duplicate proposed OM9 preset slot"));
                    }
                    if let Some(id) = plan.updated_id().or(plan.created_id())
                        && !layer_writes.insert((plan.document(), id))
                    {
                        return Err(Error::Invalid("conflicting layer writes"));
                    }
                    if plan.changes_current() && !current_writes.insert(plan.document()) {
                        return Err(Error::Invalid("conflicting current layer writes"));
                    }
                    for source in plan.assigned_sources() {
                        if !writes.insert((source.document, source.object, "layer-assign")) {
                            return Err(Error::Invalid("conflicting assigned object writes"));
                        }
                    }
                }
                Mutation::Publish(result) => {
                    for output in &result.outputs {
                        if output.expected_existing.is_none()
                            && self.identities.contains(output.entity.id)
                        {
                            return Err(Error::Invalid("retired output UUID"));
                        }
                        if output.expected_existing.is_none() {
                            proposed_identities.insert(AllocatedIdentity::Object(output.entity.id));
                        }
                    }
                    let d = base.document(result.document)?;
                    let mut verification = peak;
                    for scratch in [
                        Usage::array::<SourceStamp>(result.sources.len())?,
                        Usage::btree::<(&String, SourceStamp, crate::request::Selection), ()>(
                            result.sources.len(),
                        )?,
                        Usage::btree::<ObjectId, ()>(result.outputs.len())?,
                        Usage::btree::<LayerId, ()>(d.layers.len())?,
                        Usage::array::<ObjectId>(result.outputs.len())?,
                    ] {
                        verification.add(scratch, &limits)?;
                    }
                    for input in &result.sources {
                        verification.add(input.owned_usage(&limits)?, &limits)?;
                    }
                    result.validate(d, &limits)?;
                    if result.recipe.is_some() {
                        peak.add(
                            Usage::btree::<HistoryId, crate::history::Recipe>(
                                d.history
                                    .recipes
                                    .len()
                                    .checked_add(1)
                                    .ok_or(Error::Overflow("published recipes"))?,
                            )?,
                            &limits,
                        )?;
                    }
                    peak.add(
                        Usage::btree::<ObjectId, crate::result::PublishedResult>(
                            d.results
                                .len()
                                .checked_add(result.outputs.len())
                                .ok_or(Error::Overflow("published results"))?,
                        )?,
                        &limits,
                    )?;
                    peak.add(
                        Usage::btree::<ObjectId, Entity>(
                            d.entities
                                .len()
                                .checked_add(result.outputs.len())
                                .ok_or(Error::Overflow("published entities"))?,
                        )?,
                        &limits,
                    )?;
                    for output in &result.outputs {
                        if !writes.insert((result.document, output.entity.id, "publish")) {
                            return Err(Error::Invalid("conflicting prepared output writes"));
                        }
                    }
                }
                Mutation::Insert { document, entity } => {
                    if self.identities.contains(entity.id) {
                        return Err(Error::Invalid("allocated object UUID"));
                    }
                    proposed_identities.insert(AllocatedIdentity::Object(entity.id));
                    entity.geometry.validate(&limits)?;
                    name(&entity.label, &limits)?;
                    entity.placement.validate()?;
                    for (slot, extension) in &entity.extensions {
                        if !crate::model::slot_name_valid(slot) {
                            return Err(Error::Invalid("extension slot"));
                        }
                        extension.validate(&limits)?;
                    }
                    // A layer created earlier in this request has no captured state; its
                    // proposed record and parent chain decide whether it can receive output.
                    let d = base.document(*document)?;
                    if !d
                        .layer_permissions_with(entity.attributes.layer, false, |id| {
                            created_layers
                                .get(&(*document, id))
                                .copied()
                                .or_else(|| d.layers.get(&id))
                        })?
                        .editable
                    {
                        return Err(Error::Permission("output layer"));
                    }
                    peak.add(
                        Usage::btree::<ObjectId, Entity>(
                            base.document(*document)?
                                .entities
                                .len()
                                .checked_add(1)
                                .ok_or(Error::Overflow("inserted entities"))?,
                        )?,
                        &limits,
                    )?;
                }
                Mutation::ResourceInsert {
                    document,
                    expected_revision,
                    resource,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("resource document revision"));
                    }
                    if self.identities.contains_resource(resource.id) {
                        return Err(Error::Invalid("allocated resource UUID"));
                    }
                    if !resource_writes.insert((*document, resource.id))
                        || !inserted_resources.insert(resource.id)
                    {
                        return Err(Error::Invalid("conflicting resource writes"));
                    }
                    proposed_identities.insert(AllocatedIdentity::Resource(resource.id));
                    let count = resource_insert_counts.entry(*document).or_default();
                    *count = count
                        .checked_add(1)
                        .ok_or(Error::Overflow("resource count"))?;
                }
                Mutation::ExchangeInsert {
                    document,
                    expected_revision,
                    records,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("exchange document revision"));
                    }
                    if !exchange_writes.insert(*document) {
                        return Err(Error::Invalid("conflicting exchange writes"));
                    }
                    if !records.text_styles.is_empty()
                        || !records.dim_styles.is_empty()
                        || !records.hatch_patterns.is_empty()
                        || !records.lights.is_empty()
                    {
                        return Err(Error::Unsupported("exchange record type"));
                    }
                    for (id, record) in &records.line_patterns {
                        if *id != record.id || d.exchange.line_patterns.contains_key(id) {
                            return Err(Error::Invalid("duplicate line pattern"));
                        }
                    }
                    for (id, record) in &records.materials {
                        if *id != record.id || d.exchange.materials.contains_key(id) {
                            return Err(Error::Invalid("duplicate material"));
                        }
                    }
                    let patterns = d.exchange.line_patterns.len()
                        .checked_add(records.line_patterns.len())
                        .ok_or(Error::Overflow("line pattern count"))?;
                    let materials = d.exchange.materials.len()
                        .checked_add(records.materials.len())
                        .ok_or(Error::Overflow("material count"))?;
                    peak.add(Usage::btree::<LinePatternId, crate::exchange::LinePattern>(patterns)?, &limits)?;
                    peak.add(Usage::btree::<MaterialId, crate::exchange::RenderMaterial>(materials)?, &limits)?;
                }
                Mutation::ResourceDelete {
                    document,
                    expected_revision,
                    resource,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("resource document revision"));
                    }
                    d.resources
                        .get(resource)
                        .ok_or(Error::Missing("resource"))?;
                    if d.retained_sources
                        .values()
                        .any(|source| source.resource == *resource)
                    {
                        return Err(Error::Permission(
                            "resource owns retained source/tombstones",
                        ));
                    }
                    if !resource_writes.insert((*document, *resource)) {
                        return Err(Error::Invalid("conflicting resource writes"));
                    }
                }
                Mutation::Import {
                    document,
                    expected_revision,
                    bundle,
                } => {
                    if base.document(*document)?.revision != *expected_revision {
                        return Err(Error::Stale("import document revision"));
                    }
                    bundle.admit(*document, &limits, &mut peak)?;
                    let ids = std::iter::once(AllocatedIdentity::Resource(bundle.resource.id))
                        .chain(std::iter::once(AllocatedIdentity::Source(bundle.source.id)))
                        .chain(std::iter::once(AllocatedIdentity::Namespace(
                            bundle.namespace.id,
                        )))
                        .chain(
                            bundle
                                .entities
                                .iter()
                                .chain(bundle.definitions.iter().flat_map(|d| &d.members))
                                .map(|e| AllocatedIdentity::Object(e.id)),
                        )
                        .chain(bundle.layers.iter().map(|l| AllocatedIdentity::Layer(l.id)))
                        .chain(
                            bundle
                                .definitions
                                .iter()
                                .map(|d| AllocatedIdentity::Definition(d.id)),
                        );
                    for id in ids {
                        if self.identities.contains_identity(id) || !proposed_identities.insert(id)
                        {
                            return Err(Error::Invalid("allocated import identity"));
                        }
                    }
                    if !resource_writes.insert((*document, bundle.resource.id)) {
                        return Err(Error::Invalid("conflicting import resource writes"));
                    }
                    for entity in &bundle.entities {
                        if !writes.insert((*document, entity.id, "import")) {
                            return Err(Error::Invalid("conflicting imported object writes"));
                        }
                    }
                    let count = resource_insert_counts.entry(*document).or_default();
                    *count = count
                        .checked_add(1)
                        .ok_or(Error::Overflow("import resource count"))?;
                    let count = import_counts.entry(*document).or_default();
                    count.0 = count
                        .0
                        .checked_add(bundle.entities.len())
                        .ok_or(Error::Overflow("import entities"))?;
                    count.1 = count
                        .1
                        .checked_add(bundle.layers.len())
                        .ok_or(Error::Overflow("import layers"))?;
                    count.2 = count
                        .2
                        .checked_add(bundle.definitions.len())
                        .ok_or(Error::Overflow("import definitions"))?;
                }
                Mutation::Placement { placement, .. } => placement.validate()?,
                Mutation::Geometry { geometry, .. } => {
                    geometry.validate(&limits)?;
                }
                Mutation::Attributes { source, attributes } => {
                    if !base
                        .document(source.document)?
                        .layer_permissions(attributes.layer, false)?
                        .editable
                    {
                        return Err(Error::Permission("assigned layer"));
                    }
                }
                Mutation::Extensions { source, extensions } => {
                    if base
                        .document(source.document)?
                        .resolve(source)?
                        .geometry_revision
                        == 0
                    {
                        return Err(Error::Invalid("extension source"));
                    }
                    for (slot, extension) in extensions {
                        if !crate::model::slot_name_valid(slot)
                            || !crate::model::extension_slot_known(slot)
                        {
                            return Err(Error::Invalid("extension slot"));
                        }
                        peak.add(extension.validate(&limits)?, &limits)?;
                        peak.add(
                            Usage {
                                bytes: slot.capacity() as u64,
                                ..Usage::default()
                            },
                            &limits,
                        )?;
                    }
                }
                Mutation::Label { label, .. } => {
                    if label.is_empty()
                        || label.len() > 128
                        || label.trim() != label
                        || label.chars().any(char::is_control)
                    {
                        return Err(Error::Invalid("object label"));
                    }
                }
                Mutation::Move {
                    source,
                    destination,
                    layer,
                } => {
                    if source.document == *destination {
                        return Err(Error::Invalid("same-document ownership transfer"));
                    }
                    if base
                        .document(source.document)?
                        .retained_object(source.object)
                    {
                        return Err(Error::Unsupported(
                            "cross-document retained provenance transfer",
                        ));
                    }
                    if matches!(
                        base.document(source.document)?.resolve(source)?.geometry,
                        Geometry::Instance(_)
                    ) {
                        return Err(Error::Unsupported(
                            "cross-document instance dependency transfer",
                        ));
                    }
                    if !base
                        .document(*destination)?
                        .layer_permissions(*layer, false)?
                        .editable
                    {
                        return Err(Error::Permission("destination layer"));
                    }
                }
                Mutation::Context {
                    document,
                    expected_revision,
                    ..
                } => {
                    if base.document(*document)?.context_revision != *expected_revision {
                        return Err(Error::Stale("unit context"));
                    }
                }
                Mutation::Tolerance {
                    document,
                    expected_revision,
                    tolerance,
                } => {
                    if base.document(*document)?.revision != *expected_revision {
                        return Err(Error::Stale("document tolerance revision"));
                    }
                    tolerance.validate()?;
                    peak.add(
                        Usage {
                            bytes: std::mem::size_of::<crate::history::Tolerance>() as u64,
                            ..Usage::default()
                        },
                        &limits,
                    )?;
                }
                Mutation::History {
                    document,
                    expected_revision,
                    recipe,
                    published,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("history document revision"));
                    }
                    if d.history.recipes.contains_key(&recipe.id) {
                        return Err(Error::Invalid("duplicate history recipe"));
                    }
                    for (id, _) in published {
                        if !recipe.outputs.contains(id) {
                            return Err(Error::Invalid("history result output"));
                        }
                    }
                    // A builder records a Clean recipe whose output entity arrives in the
                    // same request; only ids inserted by this request count as pending.
                    let pending: BTreeSet<ObjectId> = created_objects
                        .iter()
                        .filter(|(created, _)| created == document)
                        .map(|(_, id)| *id)
                        .collect();
                    peak.add(recipe.validate_with_pending(d, &limits, &pending)?, &limits)?;
                }
                Mutation::HistoryUpdate {
                    document,
                    expected_revision,
                    recipes,
                    published,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("history document revision"));
                    }
                    for recipe in recipes {
                        if recipe.document != *document
                            || !d.history.recipes.contains_key(&recipe.id)
                        {
                            return Err(Error::Missing("history recipe"));
                        }
                        peak.add(recipe.validate(d, &limits)?, &limits)?;
                    }
                    for (id, _) in published {
                        if !recipes.iter().any(|recipe| recipe.outputs.contains(id)) {
                            return Err(Error::Invalid("history result output"));
                        }
                        // The new result is checked against the staged document after the
                        // geometry mutations run; validating it here would compare it with
                        // the pre-rebuild revision.
                        peak.add(
                            Usage::array::<(ObjectId, crate::result::PublishedResult)>(1)?,
                            &limits,
                        )?;
                    }
                }
                Mutation::DefinitionInsert {
                    document,
                    expected_revision,
                    definition,
                    instances,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("definition document revision"));
                    }
                    if definition.revision == 0 {
                        return Err(Error::Invalid("definition revision"));
                    }
                    name(&definition.label, &limits)?;
                    if definition.members.is_empty() {
                        return Err(Error::Invalid("empty definition"));
                    }
                    if self
                        .identities
                        .contains_identity(AllocatedIdentity::Definition(definition.id))
                        || !proposed_identities.insert(AllocatedIdentity::Definition(definition.id))
                    {
                        return Err(Error::Invalid("allocated definition UUID"));
                    }
                    for member in definition.members.iter().chain(instances) {
                        if self.identities.contains(member.id)
                            || !proposed_identities.insert(AllocatedIdentity::Object(member.id))
                        {
                            return Err(Error::Invalid("allocated object UUID"));
                        }
                        name(&member.label, &limits)?;
                        member.placement.validate()?;
                        member.geometry.validate(&limits)?;
                    }
                    for instance in instances {
                        if !matches!(instance.geometry, Geometry::Instance(id) if id == definition.id)
                        {
                            return Err(Error::Invalid("instance definition"));
                        }
                        if !d
                            .layer_permissions(instance.attributes.layer, false)?
                            .editable
                        {
                            return Err(Error::Permission("output layer"));
                        }
                    }
                    peak.add(
                        Usage::btree::<DefinitionId, Definition>(
                            d.definitions
                                .len()
                                .checked_add(1)
                                .ok_or(Error::Overflow("inserted definitions"))?,
                        )?,
                        &limits,
                    )?;
                    let added = instances
                        .len()
                        .checked_add(1)
                        .ok_or(Error::Overflow("inserted instances"))?;
                    peak.add(
                        Usage::btree::<ObjectId, Entity>(
                            d.entities
                                .len()
                                .checked_add(added)
                                .ok_or(Error::Overflow("inserted entities"))?,
                        )?,
                        &limits,
                    )?;
                }
                Mutation::FeatureInsert {
                    document,
                    expected_revision,
                    feature,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("feature document revision"));
                    }
                    if feature.revision == 0 {
                        return Err(Error::Invalid("feature revision"));
                    }
                    name(&feature.label, &limits)?;
                    if self.identities.contains_feature(feature.id)
                        || !proposed_identities.insert(AllocatedIdentity::Feature(feature.id))
                    {
                        return Err(Error::Invalid("allocated feature UUID"));
                    }
                    for output in &feature.outputs {
                        if output.is_nil()
                            || (!d.entities.contains_key(output)
                                && !created_objects.contains(&(*document, *output)))
                        {
                            return Err(Error::Missing("feature output"));
                        }
                    }
                    peak.add(
                        Usage::btree::<FeatureId, Feature>(
                            d.features
                                .len()
                                .checked_add(1)
                                .ok_or(Error::Overflow("inserted features"))?,
                        )?,
                        &limits,
                    )?;
                }
                Mutation::GroupCreate {
                    document,
                    expected_revision,
                    group,
                } => {
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("group document revision"));
                    }
                    if group.id.is_nil() || group.revision == 0 {
                        return Err(Error::Invalid("group ID/revision"));
                    }
                    name(&group.name, &limits)?;
                    if group.members.is_empty() {
                        return Err(Error::Invalid("empty group"));
                    }
                    if self.identities.contains_group(group.id)
                        || !proposed_identities.insert(AllocatedIdentity::Group(group.id))
                    {
                        return Err(Error::Invalid("allocated group UUID"));
                    }
                    for (index, member) in group.members.iter().enumerate() {
                        if member.is_nil()
                            || (!d.entities.contains_key(member)
                                && !created_objects.contains(&(*document, *member)))
                        {
                            return Err(Error::Missing("group member"));
                        }
                        if group.members[..index].contains(member) {
                            return Err(Error::Invalid("duplicate group member"));
                        }
                    }
                    peak.add(
                        Usage::btree::<GroupId, Group>(
                            d.groups
                                .len()
                                .checked_add(1)
                                .ok_or(Error::Overflow("inserted groups"))?,
                        )?,
                        &limits,
                    )?;
                }
                Mutation::GroupDelete {
                    document,
                    expected_revision,
                    group,
                } => {
                    if !group_writes.insert((*document,*group)) {return Err(Error::Invalid("conflicting group writes"));}
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("group document revision"));
                    }
                    d.groups.get(group).ok_or(Error::Missing("group"))?;
                }
                Mutation::GroupReplace { document, expected_revision, group } => {
                    if !group_writes.insert((*document,group.id)) {return Err(Error::Invalid("conflicting group writes"));}
                    let d=base.document(*document)?;
                    if d.revision != *expected_revision {return Err(Error::Stale("group document revision"));}
                    let old=d.groups.get(&group.id).ok_or(Error::Missing("group"))?;
                    if group.revision != old.revision.checked_add(1).ok_or(Error::Overflow("group revision"))? {
                        return Err(Error::Stale("group revision"));
                    }
                    name(&group.name,&limits)?;
                    if group.members.is_empty(){return Err(Error::Invalid("empty group"));}
                    for (i,member) in group.members.iter().enumerate() {
                        if member.is_nil() || (!d.entities.contains_key(member) && !created_objects.contains(&(*document,*member))) {
                            return Err(Error::Missing("group member"));
                        }
                        if group.members[..i].contains(member){return Err(Error::Invalid("duplicate group member"));}
                    }
                    peak.add(Usage::array::<ObjectId>(group.members.capacity())?,&limits)?;
                }
                Mutation::Extension {
                    slot, extension, ..
                } => {
                    if !crate::model::slot_name_valid(slot) {
                        return Err(Error::Invalid("extension slot"));
                    }
                    if let Some(extension) = extension {
                        extension.validate(&limits)?;
                    }
                    peak.add(
                        Usage {
                            bytes: slot.capacity() as u64,
                            ..Usage::default()
                        },
                        &limits,
                    )?;
                }
                Mutation::Delete { .. } => {}
            }
        }
        // Existing labels were checked during admission. Compare proposed
        // labels as well, before cloning: two renames must never validate only
        // against the same old world. Slot uniqueness bounds custom candidates
        // to sixteen per document; comparisons borrow strings without a DOM.
        for mutation in &request.mutations {
            request.cancel.check()?;
            if let Mutation::Layers(plan) = mutation
                && plan.has_custom_label()
            {
                let layer = plan
                    .proposed_layer()
                    .ok_or(Error::Invalid("custom layer plan"))?;
                for other in &request.mutations {
                    request.cancel.check()?;
                    if let Mutation::Layers(other) = other
                        && other.document() == plan.document()
                        && let Some(candidate) = other.proposed_layer()
                        && candidate.id != layer.id
                        && crate::layers::same_name(&layer.name, &candidate.name)
                    {
                        return Err(Error::Invalid("duplicate proposed layer label"));
                    }
                }
            }
        }
        // A permission read is based on the captured world, never on an
        // earlier metadata write in this same batch. Reject the overlap before
        // candidate clones; otherwise Lock+Assign could use the old permission.
        // A layer created by this batch has no captured permission to be stale, so
        // it is excluded from that overlap; its record (and parent chain) was already
        // checked when the output/assignment was admitted.
        let readable_writes: BTreeSet<(DocumentId, LayerId)> = layer_writes
            .iter()
            .filter(|write| !created_layers.contains_key(*write))
            .copied()
            .collect();
        for mutation in &request.mutations {
            request.cancel.check()?;
            if let Some(source) = mutation.source() {
                let d = base.document(source.document)?;
                crate::layers::check_layer_reads(
                    d,
                    d.resolve(&source)?.attributes.layer,
                    &layer_writes,
                )?;
            }
            match mutation {
                Mutation::Layers(plan) => plan.check_permission_reads(
                    base.document(plan.document())?,
                    plan.assignment_target()
                        .and_then(|id| created_layers.get(&(plan.document(), id)).copied()),
                    &layer_writes,
                    &request.cancel,
                )?,
                Mutation::Insert { document, entity } => {
                    let d = base.document(*document)?;
                    crate::layers::check_layer_reads_with(
                        d,
                        entity.attributes.layer,
                        &readable_writes,
                        |id| {
                            created_layers
                                .get(&(*document, id))
                                .copied()
                                .or_else(|| d.layers.get(&id))
                        },
                    )?
                }
                Mutation::Attributes { source, attributes } => crate::layers::check_layer_reads(
                    base.document(source.document)?,
                    attributes.layer,
                    &layer_writes,
                )?,
                Mutation::DefinitionInsert {
                    document,
                    instances,
                    ..
                } => {
                    for instance in instances {
                        crate::layers::check_layer_reads(
                            base.document(*document)?,
                            instance.attributes.layer,
                            &layer_writes,
                        )?;
                    }
                }
                Mutation::Move {
                    destination, layer, ..
                } => crate::layers::check_layer_reads(
                    base.document(*destination)?,
                    *layer,
                    &layer_writes,
                )?,
                Mutation::Publish(result)
                    if layer_writes.iter().any(|(id, _)| *id == result.document) =>
                {
                    return Err(Error::Invalid(
                        "layer metadata changed during result publication batch",
                    ));
                }
                _ => {}
            }
        }
        for id in &participants {
            let document = base.document(*id)?;
            let storage = document.storage_usage(&limits)?;
            document.validate(&validation_limits(&limits, peak, storage)?)?;
            add_retained_document_usage(&mut peak, storage, &limits)?;
        }
        // New resource map storage and the prospective Undo entry coexist with
        // the admitted request and saved worlds. Reject these bounds before
        // Arc::make_mut allocates any candidate document.
        for (document, added) in resource_insert_counts {
            let count = base
                .document(document)?
                .resources
                .len()
                .checked_add(added)
                .ok_or(Error::Overflow("resource count"))?;
            peak.add(Usage::btree::<ResourceId, Resource>(count)?, &limits)?;
        }
        for (document, (entities, layers, definitions)) in import_counts {
            let d = base.document(document)?;
            peak.add(
                Usage::btree::<ObjectId, Entity>(
                    d.entities
                        .len()
                        .checked_add(entities)
                        .ok_or(Error::Overflow("import entities"))?,
                )?,
                &limits,
            )?;
            peak.add(
                Usage::btree::<LayerId, crate::model::Layer>(
                    d.layers
                        .len()
                        .checked_add(layers)
                        .ok_or(Error::Overflow("import layers"))?,
                )?,
                &limits,
            )?;
            peak.add(
                Usage::array::<LayerId>(
                    d.layer_order
                        .len()
                        .checked_add(layers)
                        .ok_or(Error::Overflow("import layer order"))?,
                )?,
                &limits,
            )?;
            peak.add(
                Usage::btree::<DefinitionId, crate::model::Definition>(
                    d.definitions
                        .len()
                        .checked_add(definitions)
                        .ok_or(Error::Overflow("import definitions"))?,
                )?,
                &limits,
            )?;
        }
        let recording = self
            .scope
            .as_ref()
            .map_or(request.recording && !self.benchmark_no_undo, |s| s.recording);
        if recording && potential_changes {
            peak.add(
                Usage {
                    history: 1,
                    ..Usage::default()
                },
                &limits,
            )?;
        }
        if !proposed_identities.is_empty() {
            let count = self
                .identities
                .0
                .len()
                .checked_add(proposed_identities.len())
                .ok_or(Error::Overflow("identity count"))?;
            let mut identity_peak = peak;
            identity_peak.add(IdentityLedger::usage_for(count)?, &limits)?;
        }
        drop(writes);
        drop(resource_writes);
        drop(inserted_resources);
        drop(exchange_writes);
        drop(layer_writes);
        drop(created_preset_slots);
        drop(created_layers);
        drop(created_objects);
        drop(current_writes);
        drop(proposed_identities);
        drop(group_writes);
        // Reserve each participant's nonshared clone before Arc::make_mut.
        // This covers the known document-copy estimate, not all planning,
        // request, scene, allocator or native temporaries.
        let clone_bytes = participants.iter().try_fold(0u64, |total, id| {
            let bytes = base.document(*id)?.clone_owned_bytes_estimate(&limits)?;
            total.checked_add(bytes).and_then(|n| n.checked_add(2 * std::mem::size_of::<usize>() as u64))
                .ok_or(Error::Overflow("transaction document clone reservation"))
        })?;
        let mut clone_reservation = if clone_bytes == 0 { None } else {
            let mut live = held;
            live.add(request_storage, &self.admission_limits())?;
            let physical = self.physical_memory_sampler.and_then(|sample| sample());
            Some(self.reserve_external_memory_with_held(
                crate::budget::memory::MemoryClass::CpuCache, clone_bytes, physical, live,
            )?)
        };
        before_clone()?;
        let mut staged = base.clone();
        for mutation in request.mutations {
            request.cancel.check()?;
            match mutation {
                Mutation::Layers(plan) => {
                    if plan.changed() {
                        let id = plan.document();
                        plan.apply(staged.document_mut(id)?, &request.cancel)?;
                    }
                }
                Mutation::Publish(result) => {
                    let d = staged.document_mut(result.document)?;
                    result.apply(d)?;
                }
                Mutation::Insert { document, entity } => {
                    let d = staged.document_mut(document)?;
                    if d.entities.contains_key(&entity.id) {
                        return Err(Error::Invalid("duplicate object ID"));
                    }
                    d.entities.insert(entity.id, entity);
                }
                Mutation::ResourceInsert {
                    document, resource, ..
                } => {
                    let d = staged.document_mut(document)?;
                    if d.resources.insert(resource.id, resource).is_some() {
                        return Err(Error::Invalid("duplicate resource ID"));
                    }
                }
                Mutation::ExchangeInsert {
                    document, records, ..
                } => {
                    let d = staged.document_mut(document)?;
                    for (id, record) in records.line_patterns {
                        if d.exchange.line_patterns.insert(id, record).is_some() {
                            return Err(Error::Invalid("duplicate line pattern"));
                        }
                    }
                    for (id, record) in records.materials {
                        if d.exchange.materials.insert(id, record).is_some() {
                            return Err(Error::Invalid("duplicate material"));
                        }
                    }
                }
                Mutation::ResourceDelete {
                    document, resource, ..
                } => {
                    staged
                        .document_mut(document)?
                        .resources
                        .remove(&resource)
                        .ok_or(Error::Missing("resource"))?;
                }
                Mutation::Import {
                    document, bundle, ..
                } => bundle.apply(staged.document_mut(document)?)?,
                Mutation::Delete { source } => {
                    let d = staged.document_mut(source.document)?;
                    d.invalidate_results(source.object, true)?;
                    d.results.remove(&source.object);
                    d.entities
                        .remove(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    d.prune_membership(source.object)?;
                }
                Mutation::Placement { source, placement } => {
                    staged
                        .document_mut(source.document)?
                        .invalidate_results(source.object, false)?;
                    let e = staged
                        .document_mut(source.document)?
                        .entities
                        .get_mut(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.geometry_revision = bump(e.geometry_revision, "geometry revision")?;
                    e.placement = placement;
                }
                Mutation::Geometry { source, geometry } => {
                    staged
                        .document_mut(source.document)?
                        .invalidate_results(source.object, false)?;
                    let e = staged
                        .document_mut(source.document)?
                        .entities
                        .get_mut(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.geometry_revision = bump(e.geometry_revision, "geometry revision")?;
                    e.geometry = geometry;
                }
                Mutation::Attributes { source, attributes } => {
                    staged
                        .document_mut(source.document)?
                        .invalidate_dependents(source.object, false)?;
                    let e = staged
                        .document_mut(source.document)?
                        .entities
                        .get_mut(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.attributes_revision = bump(e.attributes_revision, "attributes revision")?;
                    e.attributes = attributes;
                }
                Mutation::Extensions { source, extensions } => {
                    staged
                        .document_mut(source.document)?
                        .invalidate_dependents(source.object, false)?;
                    let e = staged
                        .document_mut(source.document)?
                        .entities
                        .get_mut(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.attributes_revision = bump(e.attributes_revision, "attributes revision")?;
                    e.extensions = extensions;
                }
                Mutation::Label { source, label } => {
                    staged
                        .document_mut(source.document)?
                        .invalidate_dependents(source.object, false)?;
                    let e = staged
                        .document_mut(source.document)?
                        .entities
                        .get_mut(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.attributes_revision = bump(e.attributes_revision, "attributes revision")?;
                    e.label = label;
                }
                Mutation::Move {
                    source,
                    destination,
                    layer,
                } => {
                    let mut e = staged
                        .document_mut(source.document)?
                        .entities
                        .remove(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.generation = bump(e.generation, "ownership generation")?;
                    e.attributes_revision = bump(e.attributes_revision, "attributes revision")?;
                    e.attributes.layer = layer;
                    staged
                        .document_mut(source.document)?
                        .prune_membership(source.object)?;
                    if staged
                        .document_mut(destination)?
                        .entities
                        .insert(e.id, e)
                        .is_some()
                    {
                        return Err(Error::Invalid("destination object collision"));
                    }
                }
                Mutation::Context {
                    document,
                    page_unit,
                    ..
                } => {
                    let d = staged.document_mut(document)?;
                    d.context_revision = bump(d.context_revision, "context revision")?;
                    d.page_unit = page_unit;
                }
                Mutation::Tolerance {
                    document,
                    tolerance,
                    ..
                } => {
                    staged.document_mut(document)?.tolerance = Some(tolerance);
                }
                Mutation::History {
                    document,
                    recipe,
                    published,
                    ..
                } => {
                    let d = staged.document_mut(document)?;
                    if d.history.recipes.insert(recipe.id, *recipe).is_some() {
                        return Err(Error::Invalid("duplicate history recipe"));
                    }
                    for (id, result) in published {
                        d.results.insert(id, result);
                    }
                }
                Mutation::HistoryUpdate {
                    document,
                    recipes,
                    published,
                    ..
                } => {
                    let d = staged.document_mut(document)?;
                    for recipe in recipes {
                        if d.history.recipes.insert(recipe.id, *recipe).is_none() {
                            return Err(Error::Missing("history recipe"));
                        }
                    }
                    for (id, result) in published {
                        d.results.insert(id, result);
                    }
                }
                Mutation::DefinitionInsert {
                    document,
                    definition,
                    instances,
                    ..
                } => {
                    let d = staged.document_mut(document)?;
                    if d.definitions.contains_key(&definition.id) {
                        return Err(Error::Invalid("duplicate definition ID"));
                    }
                    for instance in instances {
                        if d.entities.contains_key(&instance.id) {
                            return Err(Error::Invalid("duplicate object ID"));
                        }
                        d.entities.insert(instance.id, instance);
                    }
                    d.definitions.insert(definition.id, *definition);
                }
                Mutation::FeatureInsert {
                    document, feature, ..
                } => {
                    let d = staged.document_mut(document)?;
                    if d.features.insert(feature.id, *feature).is_some() {
                        return Err(Error::Invalid("duplicate feature ID"));
                    }
                }
                Mutation::GroupCreate {
                    document, group, ..
                } => {
                    let d = staged.document_mut(document)?;
                    if d.groups.insert(group.id, *group).is_some() {
                        return Err(Error::Invalid("duplicate group ID"));
                    }
                }
                Mutation::GroupDelete {
                    document, group, ..
                } => {
                    staged
                        .document_mut(document)?
                        .groups
                        .remove(&group)
                        .ok_or(Error::Missing("group"))?;
                }
                Mutation::GroupReplace { document, group, .. } => {
                    staged.document_mut(document)?.groups.insert(group.id,*group);
                }
                Mutation::Extension {
                    source,
                    slot,
                    extension,
                } => {
                    staged
                        .document_mut(source.document)?
                        .invalidate_dependents(source.object, false)?;
                    let e = staged
                        .document_mut(source.document)?
                        .entities
                        .get_mut(&source.object)
                        .ok_or(Error::Missing("source"))?;
                    e.attributes_revision = bump(e.attributes_revision, "attributes revision")?;
                    match extension {
                        Some(extension) => {
                            e.extensions.insert(slot, extension);
                        }
                        None => {
                            e.extensions.remove(&slot);
                        }
                    }
                }
            }
        }
        for id in participants {
            let d = staged.document_mut(id)?;
            d.reconcile_retained(&limits, peak)?;
            d.revision = bump(d.revision, "document revision")?;
        }
        drop(planning_reservation);
        let staged_storage = staged.storage_usage(&limits)?;
        staged.validate(&validation_limits(&limits, peak, staged_storage)?)?;
        let identities = self.identities.extend(&staged, &limits, &mut peak)?;
        request.cancel.check()?;
        let id = TransactionId::new();
        let entry = UndoEntry::between(id, request.label, base, &staged);
        // Convert the clone reservation to registered candidates under both
        // account locks, so peers never see an uncharged handoff or both charges.
        let mut external = self.external_documents.lock().unwrap_or_else(|e| e.into_inner());
        external.retain(|item| item.strong_count() != 0);
        let mut reservations = self.external_reservations.lock().unwrap_or_else(|e| e.into_inner());
        let reserved = clone_reservation.as_ref().map_or(0, |guard| guard.bytes);
        let pending = clone_reservation.as_ref().map_or(0, |guard| guard.pending_bytes);
        let class = crate::budget::memory::MemoryClass::CpuCache.index();
        let remaining_reserved = reservations.by_class[class].checked_sub(reserved)
            .ok_or(Error::Invalid("transaction clone reservation accounting"))?;
        let remaining_pending = reservations.pending_by_class[class].checked_sub(pending)
            .ok_or(Error::Invalid("transaction clone pending accounting"))?;
        reservations.by_class[class] = remaining_reserved;
        reservations.pending_by_class[class] = remaining_pending;
        let admission = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
            for (id, candidate) in &staged.documents {
                if base.documents.get(id).is_none_or(|previous| !Arc::ptr_eq(previous, candidate)) {
                    self.register_external_document_locked(candidate, &mut external, &reservations)?;
                }
            }
            let effective = reservations.constrain(self.limits).remaining_after(held)?;
            let retained_entry = (!entry.empty()).then_some(&entry);
            self.validate_retained_owners_with_external(&identities, retained_entry, recording && retained_entry.is_some(),
                &effective, &external, &reservations.accounted_classes(0)?)?;
            Ok(())
        }));
        if !matches!(admission, Ok(Ok(()))) {
            reservations.by_class[class] += reserved;
            reservations.pending_by_class[class] += pending;
        } else if let Some(guard) = &mut clone_reservation {
            guard.bytes = 0;
            guard.pending_bytes = 0;
        }
        drop(reservations);
        drop(external);
        match admission {
            Ok(result) => result?,
            Err(panic) => std::panic::resume_unwind(panic),
        }
        Ok(PreparedTransaction {
            id,
            authority: self.authority,
            epoch: self.epoch,
            scope_generation: self.scope_generation,
            owner: request.owner,
            scope_version,
            staged,
            entry,
            recording,
            cancel: request.cancel,
            held,
            reservation: peak,
            identities,
        })
    }
    fn verify(&self, p: &PreparedTransaction) -> Result<()> {
        if self.authority != p.authority {
            return Err(Error::WrongOwner);
        }
        p.cancel.check()?;
        if self.scope_generation != p.scope_generation {
            return Err(Error::Stale("caller generation"));
        }
        if self.epoch != p.epoch {
            return Err(Error::Stale("publication epoch"));
        }
        let (_, version) = self.request_world(p.owner.as_ref())?;
        if version != p.scope_version {
            return Err(Error::Stale("caller scope version"));
        }
        Ok(())
    }
    fn publish(&mut self, staged: World, entry: UndoEntry, recording: bool, epoch: u64, reservations: &ExternalReservations) {
        self.world = staged;
        self.epoch = epoch;
        self.redo.clear();
        if recording {
            self.undo.push(entry);
            self.note_undo_memory_growth();
        } else {
            self.undo.clear();
            let _ = self.refresh_undo_memory_inventory_with_reservations(reservations);
        }
        self.trim_undo_memory_with_reservations(reservations);
    }
    pub fn commit(&mut self, p: PreparedTransaction) -> Result<CommitReceipt> {
        self.commit_with_guard(p, |_| Ok(()))
    }
    pub fn commit_with_guard<F>(
        &mut self,
        p: PreparedTransaction,
        guard: F,
    ) -> Result<CommitReceipt>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        self.available()?;
        self.verify(&p)?;
        if p.entry.empty() {
            return Ok(CommitReceipt {
                transaction: p.id,
                epoch: self.epoch,
                published: false,
                diagnostics: vec![],
            });
        }
        let next = match p.scope_version {
            Some(v) => bump(v, "scope version")?,
            None => bump(self.epoch, "publication epoch")?,
        };
        let limits = self.admission_limits().remaining_after(p.held)?;
        p.reservation.check(&limits)?;
        self.check_retention(&p.entry, p.recording, &p.identities, &limits)?;
        if p.owner.is_none() && p.recording {
            self.undo
                .try_reserve(1)
                .map_err(|_| Error::Budget("Undo allocation"))?;
        }
        self.barrier = true;
        let verdict = catch_unwind(AssertUnwindSafe(|| guard(self)))
            .unwrap_or(Err(Error::Invalid("panicking acceptance callback")));
        self.barrier = false;
        verdict?;
        self.verify(&p)?;
        // Register changed staged documents for shared-account peers, including
        // private scopes whose allocations are live before finish/abort.
        // A failed commit leaves only weak entries which expire with staging.
        let (base, _) = self.request_world(p.owner.as_ref())?;
        for (id, candidate) in &p.staged.documents {
            if base.documents.get(id).is_none_or(|previous| !Arc::ptr_eq(previous, candidate)) {
                self.register_external_document(candidate)?;
            }
        }
        // Other coordinators sharing the external account can tighten RAM
        // policy or reserve memory during the guard. Check again under the
        // shared locks, and hold the reservation lock through publication.
        // Capture-before-reservation is the existing global lock order.
        let external_registry = Arc::clone(&self.external_documents);
        let reservation_registry = Arc::clone(&self.external_reservations);
        let external = external_registry.lock().unwrap_or_else(|e| e.into_inner());
        let reservations = reservation_registry.lock().unwrap_or_else(|e| e.into_inner());
        let limits = reservations.constrain(self.limits).remaining_after(p.held)?;
        p.reservation.check(&limits)?;
        self.validate_retained_owners_with_external(&p.identities, Some(&p.entry), p.recording, &limits, &external, &reservations.accounted_classes(0)?)?;
        self.verify(&p)?;
        drop(external);
        self.barrier = true;
        let published = p.owner.is_none();
        self.identities = p.identities;
        if published {
            self.publish(p.staged, p.entry, p.recording, next, &reservations);
        } else {
            let s = self.scope.as_mut().expect("verified scope");
            s.staged = p.staged;
            s.version = next;
        }
        self.barrier = false;
        drop(reservations);
        Ok(CommitReceipt {
            transaction: p.id,
            epoch: self.epoch,
            published,
            diagnostics: vec![],
        })
    }
    pub fn commit_with_observer<F>(
        &mut self,
        p: PreparedTransaction,
        observer: F,
    ) -> Result<CommitReceipt>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let mut receipt = self.commit(p)?;
        if !receipt.published {
            return Ok(receipt);
        }
        self.barrier = true;
        let result = catch_unwind(AssertUnwindSafe(|| observer(self)))
            .unwrap_or(Err(Error::Invalid("panicking notification callback")));
        self.barrier = false;
        if let Err(e) = result {
            receipt.diagnostics.push(e.to_string());
        }
        Ok(receipt)
    }
    fn restored(&self, e: &UndoEntry, redo: bool) -> Result<World> {
        let (expected, restored) = if redo {
            (&e.before, &e.after)
        } else {
            (&e.after, &e.before)
        };
        let keys = || {
            e.before
                .keys()
                .chain(e.after.keys().filter(|id| !e.before.contains_key(id)))
        };
        for id in keys() {
            if self.world.documents.get(id) != expected.get(id) {
                return Err(Error::Stale("Undo participant"));
            }
        }
        let slots = self
            .world
            .documents
            .len()
            .checked_add(
                restored
                    .keys()
                    .filter(|id| !self.world.documents.contains_key(id))
                    .count(),
            )
            .ok_or(Error::Overflow("Undo world allocation"))?;
        let receiving = if redo {
            self.undo.len()
        } else {
            self.redo.len()
        };
        let next = receiving
            .checked_add(1)
            .and_then(|n| n.checked_mul(2))
            .ok_or(Error::Overflow("Undo/Redo allocation"))?
            .max(4);
        let mut held = Usage::btree::<DocumentId, Arc<Document>>(slots)?;
        held.add(Usage::array::<UndoEntry>(next)?, &self.limits)?;
        let usage = self.retained_usage_with_held_usage(held)?;
        let mut staged = self.world.clone();
        for id in keys() {
            match restored.get(id) {
                Some(d) => {
                    staged.documents.insert(*id, d.clone());
                }
                None => {
                    staged.documents.remove(id);
                }
            }
        }
        let staged_storage = staged.storage_usage(&self.limits)?;
        staged.validate(&validation_limits(&self.limits, usage, staged_storage)?)?;
        Ok(staged)
    }
    pub fn undo(&mut self) -> Result<()> {
        self.undo_with(|_, _, _, _| Ok(()))
    }
    /// Prepare dependent state before publishing an Undo. A failed callback
    /// leaves the world and both history stacks unchanged.
    pub fn undo_with<T>(&mut self, prepare: impl FnOnce(&Self, &World, u64, Usage) -> Result<T>) -> Result<T> {
        self.available()?;
        if self.scope.is_some() {
            return Err(Error::WrongOwner);
        }
        let e = self.undo.last().ok_or(Error::Missing("Undo entry"))?;
        let staged = self.restored(e, false)?;
        let next = bump(self.epoch, "publication epoch")?;
        self.redo
            .try_reserve(1)
            .map_err(|_| Error::Budget("Redo allocation"))?;
        let staged_map = Usage::btree::<DocumentId, Arc<Document>>(staged.documents.len())?;
        let dependent = prepare(self, &staged, next, staged_map)?;
        self.barrier = true;
        let e = self.undo.pop().expect("checked Undo");
        self.world = staged;
        self.epoch = next;
        self.redo.push(e);
        let _ = self.refresh_undo_memory_inventory();
        self.barrier = false;
        Ok(dependent)
    }
    pub fn redo(&mut self) -> Result<()> {
        self.redo_with(|_, _, _, _| Ok(()))
    }
    /// Prepare dependent state before publishing a Redo.
    pub fn redo_with<T>(&mut self, prepare: impl FnOnce(&Self, &World, u64, Usage) -> Result<T>) -> Result<T> {
        self.available()?;
        if self.scope.is_some() {
            return Err(Error::WrongOwner);
        }
        let e = self.redo.last().ok_or(Error::Missing("Redo entry"))?;
        let staged = self.restored(e, true)?;
        let next = bump(self.epoch, "publication epoch")?;
        self.undo
            .try_reserve(1)
            .map_err(|_| Error::Budget("Undo allocation"))?;
        let staged_map = Usage::btree::<DocumentId, Arc<Document>>(staged.documents.len())?;
        let dependent = prepare(self, &staged, next, staged_map)?;
        self.barrier = true;
        let e = self.redo.pop().expect("checked Redo");
        self.world = staged;
        self.epoch = next;
        self.undo.push(e);
        let _ = self.refresh_undo_memory_inventory();
        self.barrier = false;
        Ok(dependent)
    }
}

#[cfg(test)]
mod retained_validation_tests {
    use super::*;

    #[test]
    fn allocated_transfer_preserves_shared_totals_pending_and_receiver_probe() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(Vec::new(), &limits).unwrap(), limits).unwrap();
        let mut source = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache, 100, 80, None, Usage::default()).unwrap();
        let mut target = coordinator.reserve_external_memory(MemoryClass::CpuCache, 50, Some((1024*1024*1024, 1024*1024*1024))).unwrap();
        let total = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let pending = coordinator.external_pending_bytes(MemoryClass::CpuCache);
        coordinator.external_reservations.lock().unwrap().managed_ceiling = Some(1);
        target.transfer_allocated_from(&mut source, 80).unwrap();
        assert_eq!((source.bytes(), source.pending_bytes()), (20, 20));
        assert_eq!((target.bytes(), target.pending_bytes()), (130, 50));
        assert!(target.physical_sampled());
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), total);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), pending);
        drop(source);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 130);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 50);
        drop(target);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn allocated_transfer_rejects_pending_foreign_and_other_class_without_changes() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(Vec::new(), &limits).unwrap(), limits).unwrap();
        let foreign = TransactionCoordinator::new(World::new(Vec::new(), &limits).unwrap(), limits).unwrap();
        let mut source = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache, 100, 40, None, Usage::default()).unwrap();
        let mut target = coordinator.reserve_external_memory(MemoryClass::CpuCache, 30, None).unwrap();
        assert!(target.transfer_allocated_from(&mut source, 41).is_err());
        let mut wrong_class = coordinator.reserve_external_memory(MemoryClass::Kernel, 20, None).unwrap();
        assert!(wrong_class.transfer_allocated_from(&mut source, 1).is_err());
        let mut wrong_account = foreign.reserve_external_memory(MemoryClass::CpuCache, 20, None).unwrap();
        assert!(wrong_account.transfer_allocated_from(&mut source, 1).is_err());
        assert_eq!((source.bytes(), source.pending_bytes()), (100, 60));
        assert_eq!((target.bytes(), target.pending_bytes()), (30, 30));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 130);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 90);
        target.transfer_allocated_from(&mut source, 0).unwrap();
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 130);
    }

    #[test]
    fn shrinking_external_reservation_preserves_accounting_and_rejects_growth() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("shrink")], &limits).unwrap(), limits).unwrap();
        let mut reservation = coordinator.reserve_external_memory(MemoryClass::Kernel, 4096, None).unwrap();
        reservation.shrink_to(1024).unwrap();
        assert_eq!(reservation.bytes(), 1024);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 1024);
        assert!(reservation.shrink_to(2048).is_err());
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 1024);
        drop(reservation);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn splitting_external_reservation_keeps_total_until_each_owner_drops() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        let mut original = coordinator.reserve_external_memory(MemoryClass::CpuCache, 4096, None).unwrap();
        assert!(original.split_off(0).is_err());
        assert!(original.split_off(4097).is_err());
        let part = original.split_off(1024).unwrap();
        assert_eq!((original.bytes(), part.bytes()), (3072, 1024));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 4096);
        drop(original);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 1024);
        drop(part);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn history_dependent_preparation_failure_preserves_world_and_both_stacks() {
        let limits = Limits::default();
        let mut document = Document::new("history scene admission");
        let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let id = point.id;
        document.entities.insert(id, point);
        let source = document.stamp(id).unwrap();
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let before = coordinator.snapshot();
        let result: Result<()> = coordinator.undo_with(|_, _, _, _| Err(Error::Budget("scene construction")));
        assert!(matches!(result, Err(Error::Budget("scene construction"))));
        assert_eq!(coordinator.snapshot(), before);
        coordinator.undo().unwrap();
        let before = coordinator.snapshot();
        let result: Result<()> = coordinator.redo_with(|_, _, _, _| Err(Error::Budget("scene construction")));
        assert!(matches!(result, Err(Error::Budget("scene construction"))));
        assert_eq!(coordinator.snapshot(), before);
        coordinator.redo().unwrap();
    }

    fn source_fixture() -> crate::brep::Brep {
        use crate::brep::{Brep, BrepDisplay};
        Brep::from_native(format!("CASCADE Topology {}", "x".repeat(1024 * 1024)), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1, points: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            triangles: vec![[0,1,2,0]], segments: vec![], vertices: vec![], faces: vec![uuid::Uuid::new_v4()],
            edges: vec![], vertex_ids: vec![], face_edges: vec![],
        }, &Limits::default()).unwrap()
    }

    #[test]
    fn shared_brep_display_is_counted_once_across_history_and_captures() {
        use crate::brep::{Brep, BrepDisplay};
        let limits = Limits::default();
        let mut points = vec![[0., 0., 0.]; 50_000];
        points[1] = [1., 0., 0.];
        points[2] = [0., 1., 0.];
        let shape = Brep::from_native("CASCADE Topology fixture".into(), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1, points,
            triangles: vec![[0, 1, 2, 0]], segments: vec![], vertices: vec![],
            faces: vec![uuid::Uuid::new_v4()], edges: vec![], vertex_ids: vec![], face_edges: vec![],
        }, &limits).unwrap();
        let display = shape.display.clone();
        let mut document = Document::new("shared display");
        let entity = Entity::new("shape", Geometry::Brep(shape.clone()), None);
        let entity_id = entity.id;
        document.entities.insert(entity_id, entity);
        let one = document.storage_usage(&limits).unwrap().bytes;
        let alias = Entity::new("alias", Geometry::Brep(shape.clone()), None);
        let alias_id = alias.id;
        document.entities.insert(alias_id, alias);
        let aliased = document.storage_usage(&limits).unwrap();
        assert!(aliased.bytes - one < 512 * 1024, "shared display duplicated within document");
        assert_eq!(aliased, document.validate(&limits).unwrap());
        document.entities.remove(&alias_id);
        let independent = Brep::from_native(shape.source().to_owned(), (*shape.display).clone(), &limits).unwrap();
        let other = Entity::new("independent", Geometry::Brep(independent), None);
        let other_id = other.id;
        document.entities.insert(other_id, other);
        assert!(document.storage_usage(&limits).unwrap().bytes - one >= 1_000_000,
            "equal independent displays must be charged separately");
        document.entities.remove(&other_id);
        let document_id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let before = coordinator.retained_usage().unwrap().bytes;
        let capture = coordinator.capture_retained_document(document_id).unwrap();
        let stamp = coordinator.world.documents[&document_id].stamp(entity_id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source: stamp, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let Geometry::Brep(current) = &coordinator.world.documents[&document_id].entities[&entity_id].geometry else { panic!("B-rep") };
        assert!(Arc::ptr_eq(&display, &current.display));
        let after = coordinator.retained_usage().unwrap().bytes;
        assert!(after - before < 512 * 1024, "shared display charged twice: {before} -> {after}");
        let fallback = coordinator.retained_storage_usage(&coordinator.identities, None, false, &limits).unwrap();
        let indexed = coordinator.validate_internal_retained_owners(&coordinator.identities, None, false, &limits).unwrap();
        assert_eq!(fallback, indexed);
        drop(capture);
        coordinator.undo().unwrap();
        coordinator.redo().unwrap();
        assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024,
            "display still owned by the live document must not be charged to Undo");
    }

    #[test]
    fn tracked_capture_lifetime_is_distinct_from_document_lifetime_and_follows_replacement() {
        let limits = Limits::default();
        let old_document = Document::new("tracked old");
        let old_id = old_document.id;
        let old = TransactionCoordinator::new(World::new(vec![old_document], &limits).unwrap(), limits).unwrap();
        let capture = old.capture_tracked_document(old_id).unwrap();
        let clone = capture.clone();
        let source = Arc::downgrade(&old.world.documents[&old_id]);
        assert!(capture.matches_weak(&source));
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("replacement")], &limits).unwrap(), limits).unwrap();
        replacement.adopt_external_captures(&old).unwrap();
        assert!(Arc::ptr_eq(&replacement.external_reservations, &old.external_reservations));
        assert!(Arc::ptr_eq(&replacement.capture_generation, &old.capture_generation));
        assert_eq!(replacement.external_reservations.lock().unwrap().capture_leases.iter().filter(|lease| lease.strong_count() != 0).count(), 1);
        drop(capture);
        assert_eq!(replacement.external_reservations.lock().unwrap().capture_leases.iter().filter(|lease| lease.strong_count() != 0).count(), 1);
        drop(clone);
        assert!(source.upgrade().is_some(), "the original coordinator still owns the document");
        assert_eq!(replacement.external_reservations.lock().unwrap().capture_leases.iter().filter(|lease| lease.strong_count() != 0).count(), 0,
            "document ownership must not masquerade as a live capture lease");
    }

    #[test]
    fn undo_stats_reads_capture_generation_without_waiting_for_account_lock() {
        use std::{sync::mpsc, time::Duration};
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("stats signal")], &limits).unwrap(), limits).unwrap();
        let registry = Arc::clone(&coordinator.external_reservations);
        let held = registry.lock().unwrap();
        std::thread::scope(|scope| {
            let (started_tx, started_rx) = mpsc::channel();
            let (result_tx, result_rx) = mpsc::channel();
            let coordinator_ref = &coordinator;
            let worker = scope.spawn(move || {
                started_tx.send(()).unwrap();
                result_tx.send(coordinator_ref.undo_memory_stats()).unwrap();
            });
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let observed = result_rx.recv_timeout(Duration::from_secs(2));
            drop(held);
            worker.join().unwrap();
            assert!(observed.is_ok(), "reading cached Undo stats must not wait for the account mutex");
        });
    }

    #[test]
    fn replacement_marks_undo_inventory_stale_even_when_account_generations_match() {
        let limits = Limits::default();
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        replacement.set_undo_policy(UndoPolicy::default()).unwrap();
        assert!(!replacement.undo_memory_stats().capture_change_pending);
        replacement.adopt_external_captures(&old).unwrap();
        assert!(replacement.undo_memory_stats().capture_change_pending);
    }

    #[test]
    fn tracked_capture_registration_holds_both_account_locks_until_lease_is_registered() {
        let limits = Limits::default();
        let document = Document::new("atomic tracked capture");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let documents = Arc::clone(&coordinator.external_documents);
        let reservations = Arc::clone(&coordinator.external_reservations);
        let capture = coordinator.capture_tracked_document_with_guard(id, || {
            assert!(matches!(documents.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
            assert!(matches!(reservations.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
        }).unwrap();
        assert_eq!(capture.id, id);
        assert_eq!(documents.lock().unwrap().len(), 1);
        assert_eq!(reservations.lock().unwrap().capture_leases.iter().filter(|lease| lease.strong_count() != 0).count(), 1);
    }

    #[test]
    fn shared_account_fixed_roots_are_charged_once_across_admission_peers() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        let before = coordinator.external_reservations.lock().unwrap().accounted_classes(0).unwrap();
        let minimum = std::mem::size_of::<Mutex<ExternalReservations>>()
            + std::mem::size_of::<Mutex<Vec<Weak<Document>>>>()
            + std::mem::size_of::<AtomicBool>() + std::mem::size_of::<AtomicU64>()
            + std::mem::size_of::<Vec<AllocatedIdentity>>() + 10 * std::mem::size_of::<usize>();
        assert!(before[MemoryClass::Document.index()] >= minimum as u64,
            "account, document index, export gate, generation and empty peer ledger allocations all remain live");
        let peer = coordinator.memory_admission_peer().unwrap();
        let account = peer.external_memory_account();
        assert_eq!(peer.external_reservations.lock().unwrap().accounted_classes(0).unwrap(), before);
        drop(account);
        drop(peer);
        assert_eq!(coordinator.external_reservations.lock().unwrap().accounted_classes(0).unwrap(), before);
    }

    #[test]
    fn startup_refuses_when_world_fits_but_coordinator_and_account_roots_do_not() {
        let base = Limits::default();
        let world = World::new(vec![], &base).unwrap();
        let max_bytes = world.validate(&base).unwrap().bytes + IdentityLedger::usage_for(0).unwrap().bytes
            + Usage::btree::<AllocatedIdentity, ()>(0).unwrap().bytes;
        let limits = Limits { max_bytes, ..base };
        assert!(matches!(TransactionCoordinator::new(world, limits), Err(Error::Budget(_))),
            "startup must admit fixed owners before creating their allocations");
    }

    #[test]
    fn expired_document_weak_owners_retain_one_fixed_arc_block_until_pruned() {
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        let document = Arc::new(Document::new("expired weak block"));
        let weak = Arc::downgrade(&document);
        {
            let mut external = coordinator.external_documents.lock().unwrap();
            external.push(weak.clone());
            external.push(weak.clone());
        }
        *coordinator.current_validation_cache.lock().unwrap() = Some((weak, limits));
        drop(document);
        let block = (std::mem::size_of::<Document>() + 2 * std::mem::size_of::<usize>()) as u64;
        let before = coordinator.retained_usage().unwrap().bytes;
        assert!(coordinator.current_validation_cache.lock().unwrap().is_none(), "expired validation proof is retired during inventory");
        coordinator.external_documents.lock().unwrap().clear();
        let after = coordinator.retained_usage().unwrap().bytes;
        assert_eq!(before - after, block, "payload is dropped, but the Arc allocation containing Document is still live");
    }

    #[test]
    fn tracked_capture_registry_capacity_and_control_blocks_are_in_retained_usage() {
        let limits = Limits::default();
        let document = Document::new("lease accounting");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let before = coordinator.retained_usage().unwrap().bytes;
        let capture = coordinator.capture_tracked_document(id).unwrap();
        let after = coordinator.retained_usage().unwrap().bytes;
        let external_capacity = coordinator.external_documents.lock().unwrap().capacity();
        let (lease_capacity, lease_len) = {
            let registry = coordinator.external_reservations.lock().unwrap();
            (registry.capture_leases.capacity(), registry.capture_leases.len())
        };
        assert_eq!(lease_len, 1);
        let minimum = external_capacity * std::mem::size_of::<Weak<Document>>()
            + lease_capacity * std::mem::size_of::<Weak<CapturedDocumentInner>>()
            + std::mem::size_of::<CapturedDocumentInner>() + 2 * std::mem::size_of::<usize>();
        assert!(after - before >= minimum as u64, "lease allocation and both vector capacities must be charged");
        let copy = capture.clone();
        assert_eq!(coordinator.retained_usage().unwrap().bytes, after, "cloned handles share one allocation");
        drop(capture);
        drop(copy);
        assert_eq!(coordinator.retained_usage().unwrap().bytes, after, "expired weak entry still retains its control block until pruned");
        let next = coordinator.capture_tracked_document(id).unwrap();
        assert_eq!(coordinator.external_reservations.lock().unwrap().capture_leases.len(), 1);
        assert_eq!(coordinator.retained_usage().unwrap().bytes, after);
        drop(next);
    }

    #[test]
    fn tracked_capture_refusal_preserves_both_empty_registries() {
        let limits = Limits::default();
        let document = Document::new("tracked simultaneous registry growth");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let baseline = coordinator.retained_usage().unwrap().bytes;
        coordinator.external_reservations.lock().unwrap().managed_ceiling = Some(baseline);
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.capture_tracked_document(id), Err(Error::Budget(_))));
        let documents = coordinator.external_documents.lock().unwrap();
        let reservations = coordinator.external_reservations.lock().unwrap();
        assert_eq!((documents.len(), documents.capacity()), (0, 0));
        assert_eq!((reservations.capture_leases.len(), reservations.capture_leases.capacity()), (0, 0));
        drop(reservations);
        drop(documents);
        assert_eq!(coordinator.snapshot(), before);
        coordinator.external_reservations.lock().unwrap().managed_ceiling = None;
        let capture = coordinator.capture_tracked_document(id).unwrap();
        assert_eq!(capture.id, id);
        assert_eq!(coordinator.external_documents.lock().unwrap().len(), 1);
        assert_eq!(coordinator.external_reservations.lock().unwrap().capture_leases.len(), 1);
    }

    #[test]
    fn untracked_capture_refused_before_registry_growth_preserves_capacity() {
        let limits = Limits::default();
        let document = Document::new("legacy registry growth refusal");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let baseline = coordinator.retained_usage().unwrap().bytes;
        coordinator.external_reservations.lock().unwrap().managed_ceiling = Some(baseline);
        let before = coordinator.snapshot();
        assert_eq!(coordinator.external_documents.lock().unwrap().capacity(), 0);
        assert!(matches!(coordinator.capture_retained_document(id), Err(Error::Budget(_))));
        let registry = coordinator.external_documents.lock().unwrap();
        assert!(registry.is_empty());
        assert_eq!(registry.capacity(), 0, "refusal must not retain a newly allocated registry");
        drop(registry);
        assert_eq!(coordinator.snapshot(), before);
        coordinator.external_reservations.lock().unwrap().managed_ceiling = None;
        assert_eq!(coordinator.capture_retained_document(id).unwrap().id, id);
    }

    #[test]
    fn failed_capture_growth_preserves_existing_lease_registry_capacity() {
        let limits = Limits::default();
        let document = Document::new("lease growth refusal");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let first = coordinator.capture_tracked_document(id).unwrap();
        let baseline = coordinator.retained_usage().unwrap().bytes;
        let old_capacity = {
            let mut registry = coordinator.external_reservations.lock().unwrap();
            assert_eq!(registry.capture_leases.len(), registry.capture_leases.capacity());
            registry.managed_ceiling = Some(baseline);
            registry.capture_leases.capacity()
        };
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.capture_tracked_document(id), Err(Error::Budget(_))));
        let registry = coordinator.external_reservations.lock().unwrap();
        assert_eq!(registry.capture_leases.len(), 1);
        assert_eq!(registry.capture_leases.capacity(), old_capacity);
        assert_eq!(registry.capture_leases[0].strong_count(), 1);
        drop(registry);
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(first.id, id);
    }

    #[test]
    fn live_capture_metadata_is_not_a_pending_physical_allocation() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let document = Document::new("lease physical recheck");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_tracked_document(id).unwrap();
        let total = 1024 * MIB;
        let reservation = coordinator.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))).unwrap();
        let metadata = coordinator.external_reservations.lock().unwrap().accounted_classes(0).unwrap()[MemoryClass::Document.index()];
        assert!(metadata > 0);
        // The registry metadata is live and still contributes to managed usage;
        // OS Available already reflects its physical pressure at sample time.
        let exact = 256 * MIB + MIB;
        assert!(matches!(reservation.recheck_physical(total, exact - 1), Err(Error::Budget("available physical RAM"))));
        assert!(reservation.recheck_physical(total, exact).is_ok());
        drop(capture);
    }

    #[test]
    fn peer_reservation_waits_for_atomic_tracked_capture_and_counts_its_document() {
        use crate::{budget::memory::MemoryClass, resource::{Resource, ResourceKind}};
        use std::{sync::mpsc, time::Duration};
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let mut document = Document::new("capture under contention");
        let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &vec![17; 2 * MIB as usize], &limits).unwrap();
        document.resources.insert(resource.id, resource);
        let id = document.id;
        let old = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("replacement")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&old).unwrap();
        let before = peer.snapshot();
        std::thread::scope(|scope| {
            let (inside_tx, inside_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let old_ref = &old;
            let capture = scope.spawn(move || old_ref.capture_tracked_document_with_guard(id, || {
                inside_tx.send(()).unwrap();
                release_rx.recv().unwrap();
            }));
            inside_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let (started_tx, started_rx) = mpsc::channel();
            let (done_tx, done_rx) = mpsc::channel();
            let peer_ref = &peer;
            let reservation = scope.spawn(move || {
                started_tx.send(()).unwrap();
                let result = peer_ref.reserve_external_memory(MemoryClass::Export, 15 * MIB, None);
                done_tx.send(result).unwrap();
            });
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let blocked = matches!(done_rx.recv_timeout(Duration::from_millis(100)), Err(mpsc::RecvTimeoutError::Timeout));
            release_tx.send(()).unwrap();
            let capture = capture.join().unwrap().unwrap();
            let result = done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            reservation.join().unwrap();
            assert!(blocked, "peer must wait while capture registration owns the locks");
            assert!(matches!(result, Err(Error::Budget(_))), "peer must account for the captured 2 MiB document");
            assert_eq!(peer.snapshot(), before);
            assert_eq!(peer.external_reserved_bytes(MemoryClass::Export), 0);
            drop(capture);
        });
        drop(old);
        assert!(peer.reserve_external_memory(MemoryClass::Export, 15 * MIB, None).is_ok());
    }

    #[test]
    fn rejected_tracked_capture_removes_its_new_document_entry() {
        use crate::{budget::memory::MemoryClass, resource::{Resource, ResourceKind}};
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let mut document = Document::new("rejected tracked capture");
        let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &vec![7; 2 * MIB as usize], &limits).unwrap();
        document.resources.insert(resource.id, resource);
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        assert!(coordinator.reserve_external_memory(MemoryClass::Export, 1, Some((3 * MIB, 0))).is_err());
        let before = coordinator.snapshot();
        assert!(coordinator.capture_tracked_document(id).is_err());
        assert_eq!(coordinator.snapshot(), before);
        assert!(coordinator.external_documents.lock().unwrap().is_empty());
        assert!(coordinator.external_reservations.lock().unwrap().capture_leases.is_empty());
    }

    #[test]
    fn replacement_with_its_own_active_tracked_capture_is_rejected() {
        let limits = Limits::default();
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let new_document = Document::new("new");
        let new_id = new_document.id;
        let mut replacement = TransactionCoordinator::new(World::new(vec![new_document], &limits).unwrap(), limits).unwrap();
        let capture = replacement.capture_tracked_document(new_id).unwrap();
        assert!(replacement.adopt_external_captures(&old).is_err());
        assert_eq!(capture.id, new_id);
        assert_eq!(replacement.world.documents.len(), 1);
    }

    #[test]
    fn undo_inventory_counts_shared_brep_source_only_when_history_owns_it() {
        let limits = Limits::default();
        let mut document = Document::new("undo shared B-rep source");
        let entity = Entity::new("shape", Geometry::Brep(source_fixture()), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        for x in 1..=3 {
            let source = coordinator.world.documents[&doc].stamp(id).unwrap();
            let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
                source, placement: Transform::translation([x as f64, 0., 0.]),
            }])).unwrap();
            coordinator.commit(prepared).unwrap();
        }
        let live = coordinator.undo_memory_bytes().unwrap();
        assert!(live < 512 * 1024, "live shared source must not be charged to Undo: {live}");
        coordinator.set_undo_policy(UndoPolicy { minimum_steps: 1, max_memory_mb: 1 }).unwrap();
        assert_eq!(coordinator.undo_len(), 3, "shared live source must not evict history");
        coordinator.set_undo_policy(UndoPolicy { minimum_steps: 1, max_memory_mb: 4 }).unwrap();
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("delete", vec![Mutation::Delete { source }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let retained = coordinator.undo_memory_bytes().unwrap();
        assert!((1024 * 1024..1536 * 1024).contains(&retained), "history-only shared source must be counted once: {retained}");
        coordinator.undo().unwrap();
        assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024);
        coordinator.redo().unwrap();
        assert!((1024 * 1024..1536 * 1024).contains(&coordinator.undo_memory_bytes().unwrap()));
    }

    #[test]
    fn tracked_capture_excludes_shared_source_until_last_handle_drops() {
        let limits = Limits::default();
        let mut document = Document::new("captured B-rep Undo");
        let entity = Entity::new("shape", Geometry::Brep(source_fixture()), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_tracked_document(doc).unwrap();
        for x in 1..=3 {
            let source = coordinator.world.documents[&doc].stamp(id).unwrap();
            let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
                source, placement: Transform::translation([x as f64, 0., 0.]),
            }])).unwrap();
            coordinator.commit(prepared).unwrap();
        }
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("delete", vec![Mutation::Delete { source }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let redo_point = Entity::new("redo point", Geometry::Point([1., 2., 3.]), None);
        let prepared = coordinator.prepare(TransactionRequest::new("redo point", vec![Mutation::Insert { document: doc, entity: redo_point }])).unwrap();
        coordinator.commit(prepared).unwrap();
        coordinator.undo().unwrap();
        assert_eq!(coordinator.redo_len(), 1);
        let captured_usage = coordinator.undo_memory_bytes().unwrap();
        assert!(captured_usage < 512 * 1024, "active capture owns shared source: {captured_usage}");
        coordinator.set_undo_policy(UndoPolicy { minimum_steps: 1, max_memory_mb: 1 }).unwrap();
        assert_eq!(coordinator.undo_len(), 4);
        assert!(!coordinator.undo_memory_stats().capture_change_pending);
        let duplicate = capture.clone();
        drop(capture);
        assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024, "clone still owns capture");
        assert!(!coordinator.undo_memory_stats().capture_change_pending, "a remaining handle keeps the same owner");
        drop(duplicate);
        assert!(coordinator.undo_memory_stats().capture_change_pending, "the cached estimate must show the owner change");
        assert!(coordinator.undo_memory_bytes().unwrap() > 1024 * 1024, "history owns the source after last capture drops");
        let before_failure = coordinator.snapshot();
        let invalid = Entity::new("invalid", Geometry::Point([f64::NAN, 0., 0.]), None);
        assert!(coordinator.prepare(TransactionRequest::new("invalid", vec![Mutation::Insert { document: doc, entity: invalid }])).is_err());
        assert_eq!(coordinator.snapshot(), before_failure, "a failed operation cannot prune Undo or change the document");
        assert_eq!(coordinator.undo_len(), 4);
        assert_eq!(coordinator.redo_len(), 1, "failed work must preserve the existing Redo branch");
        assert!(coordinator.undo_memory_stats().capture_change_pending);
        let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let point_id = point.id;
        let prepared = coordinator.prepare(TransactionRequest::new("point", vec![Mutation::Insert { document: doc, entity: point }])).unwrap();
        coordinator.commit(prepared).unwrap();
        assert!(coordinator.undo_len() < 5, "next successful publication must reconcile the changed owner and trim over-budget history");
        assert!(!coordinator.undo_memory_stats().capture_change_pending);
        assert!(coordinator.world.documents[&doc].entities.contains_key(&point_id));
    }

    #[test]
    fn finished_scope_reconciles_undo_after_its_old_world_is_released() {
        let limits = Limits::default();
        let mut document = Document::new("scope Undo accounting");
        let entity = Entity::new("shape", Geometry::Brep(source_fixture()), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        coordinator.set_undo_policy(UndoPolicy { minimum_steps: 1, max_memory_mb: 1 }).unwrap();
        let token = coordinator.book("scoped delete", true).unwrap();
        coordinator.begin(&token).unwrap();
        let source = coordinator.editing_world(&token).unwrap().documents[&doc].stamp(id).unwrap();
        let mut request = TransactionRequest::new("delete", vec![Mutation::Delete { source }]);
        request.owner = Some(token.clone());
        coordinator.commit(coordinator.prepare(request).unwrap()).unwrap();
        coordinator.finish(&token).unwrap();
        let actual = coordinator.undo_memory_bytes().unwrap();
        let stats = coordinator.undo_memory_stats();
        assert!(actual > 1024 * 1024, "the deleted source is now retained only by Undo");
        assert!(stats.estimated_bytes >= actual, "scope release must not leave the estimate undercounted: {} < {actual}", stats.estimated_bytes);
        coordinator.undo().unwrap();
        assert!(coordinator.world.documents[&doc].entities.contains_key(&id));
    }

    #[test]
    fn tracked_resource_capture_is_excluded_until_drop_but_legacy_arc_is_conservative() {
        use crate::resource::{Resource, ResourceKind};
        for tracked in [true, false] {
            let limits = Limits::default();
            let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &vec![9; 2 * 1024 * 1024], &limits).unwrap();
            let rid = resource.id;
            let mut document = Document::new("captured resource Undo");
            document.resources.insert(rid, resource);
            let id = document.id;
            let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
            let capture = tracked.then(|| coordinator.capture_tracked_document(id).unwrap());
            let legacy = (!tracked).then(|| coordinator.capture_retained_document(id).unwrap());
            let revision = coordinator.world.documents[&id].revision;
            let request = TransactionRequest::new("delete resource", vec![Mutation::ResourceDelete { document: id, expected_revision: revision, resource: rid }]);
            coordinator.commit(coordinator.prepare(request).unwrap()).unwrap();
            let during = coordinator.undo_memory_bytes().unwrap();
            if tracked {
                assert!(during < 512 * 1024, "active tracked capture owns resource: {during}");
            } else {
                assert!(during >= 2 * 1024 * 1024, "untracked Arc must not be assumed active from its weak document entry");
            }
            drop(capture);
            drop(legacy);
            assert!(coordinator.undo_memory_bytes().unwrap() >= 2 * 1024 * 1024);
            if tracked { assert!(coordinator.undo_memory_stats().capture_change_pending); }
        }
    }

    #[test]
    fn capture_dropped_during_undo_inventory_invalidates_the_result_without_locking_again() {
        let limits = Limits::default();
        let document = Document::new("capture inventory race");
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_tracked_document(id).unwrap();
        let registry = Arc::clone(&coordinator.external_reservations);
        let reservations = registry.lock().unwrap();
        let result = coordinator.undo_memory_bytes_with_guard(&reservations, || {
            std::thread::spawn(move || drop(capture)).join().unwrap();
        });
        assert!(matches!(result, Err(Error::Busy)));
        drop(reservations);
        assert!(coordinator.undo_memory_bytes().is_ok());
    }

    #[test]
    fn undo_inventory_distinguishes_aliases_from_equal_independent_brep_sources() {
        let limits = Limits::default();
        let first = source_fixture();
        let second = source_fixture();
        assert_eq!(first.source(), second.source());
        assert_ne!(first.source_allocation_key(), second.source_allocation_key());
        let mut document = Document::new("undo B-rep source identities");
        let mut ids = Vec::new();
        for brep in [first.clone(), first, second] {
            let entity = Entity::new("shape", Geometry::Brep(brep), None);
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let doc = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let source = coordinator.world.documents[&doc].stamp(ids[0]).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        assert!(coordinator.undo_memory_bytes().unwrap() < 512 * 1024);
        let deletes = ids.iter().map(|id| Mutation::Delete { source: coordinator.world.documents[&doc].stamp(*id).unwrap() }).collect();
        let prepared = coordinator.prepare(TransactionRequest::new("delete", deletes)).unwrap();
        coordinator.commit(prepared).unwrap();
        let retained = coordinator.undo_memory_bytes().unwrap();
        assert!((2 * 1024 * 1024..2560 * 1024).contains(&retained), "two separate allocations should each count once: {retained}");
    }

    #[test]
    fn world_counts_shared_native_payload_once_across_documents() {
        let limits = Limits::default();
        let native = crate::geometry::NativeBrep {
            display: crate::geometry::Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] },
            payload: vec![7; 1024 * 1024].into(), crc: 1,
        };
        let mut first = Document::new("first");
        let first_entity = Entity::new("shape", Geometry::NativeBrep(native.clone()), None);
        first.entities.insert(first_entity.id, first_entity);
        let mut second = Document::new("second");
        let second_entity = Entity::new("shape", Geometry::NativeBrep(native), None);
        second.entities.insert(second_entity.id, second_entity);
        let world = World::new(vec![first, second], &limits).unwrap();
        let used = world.validate(&limits).unwrap();
        let tight = Limits { max_bytes: used.bytes + 128 * 1024, ..limits };
        assert!(tight.max_bytes < 2 * 1024 * 1024);
        assert_eq!(world.validate(&tight).unwrap(), used);
    }

    #[test]
    fn native_payload_external_versions_distinguish_shared_and_independent_storage() {
        let limits = Limits::default();
        let native = crate::geometry::NativeBrep {
            display: crate::geometry::Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] },
            payload: vec![7; 1024 * 1024].into(), crc: 1,
        };
        let mut document = Document::new("external native payload");
        let entity = Entity::new("shape", Geometry::NativeBrep(native), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let doc = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let first = coordinator.capture_retained_document(doc).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&doc).unwrap()).name = "second".into();
        let second = coordinator.capture_retained_document(doc).unwrap();
        let current = Arc::make_mut(coordinator.world.documents.get_mut(&doc).unwrap());
        let Geometry::NativeBrep(brep) = &mut current.entities.get_mut(&id).unwrap().geometry else { panic!("native B-rep") };
        brep.payload = (*brep.payload).clone().into();
        let two_payloads = coordinator.retained_usage().unwrap().bytes;
        drop(second);
        let one_capture = coordinator.retained_usage().unwrap().bytes;
        assert!(two_payloads - one_capture < 512 * 1024, "shared captures must not duplicate payload");
        drop(first);
        let current_only = coordinator.retained_usage().unwrap().bytes;
        assert!(one_capture - current_only >= 1024 * 1024, "independent allocations must both be counted");
    }

    #[test]
    fn shared_native_payload_is_counted_once_across_capture_and_undo() {
        let limits = Limits::default();
        let native = crate::geometry::NativeBrep {
            display: crate::geometry::Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] },
            payload: vec![7; 1024 * 1024].into(), crc: 1,
        };
        let mut document = Document::new("shared native payload");
        let entity = Entity::new("shape", Geometry::NativeBrep(native), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let doc = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let original = coordinator.retained_usage().unwrap().bytes;
        let capture = coordinator.capture_retained_document(doc).unwrap();
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement { source, placement: Transform::translation([1., 0., 0.]) }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let retained = coordinator.retained_usage().unwrap().bytes;
        assert!(retained - original < 512 * 1024, "shared payload double charged: before={original}, after={retained}");
        let fallback = coordinator.retained_storage_usage(&coordinator.identities, None, false, &limits).unwrap();
        let indexed = coordinator.validate_internal_retained_owners(&coordinator.identities, None, false, &limits).unwrap();
        assert_eq!(fallback, indexed);
        let tight = Limits { max_bytes: fallback.bytes + 64 * 1024, ..limits };
        assert_eq!(coordinator.validate_internal_retained_owners(&coordinator.identities, None, false, &tight).unwrap(), fallback);
        drop(capture);
        coordinator.undo().unwrap();
        coordinator.redo().unwrap();
    }

    #[test]
    fn shared_brep_source_is_counted_once_across_capture_and_undo() {
        let limits = Limits::default();
        let mut document = Document::new("shared B-rep source");
        let entity = Entity::new("shape", Geometry::Brep(source_fixture()), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let document_id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let original = coordinator.retained_usage().unwrap().bytes;
        let capture = coordinator.capture_retained_document(document_id).unwrap();
        let source = coordinator.world.documents[&document_id].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement { source, placement: Transform::translation([1., 0., 0.]) }])).unwrap();
        coordinator.commit(prepared).unwrap();
        let retained = coordinator.retained_usage().unwrap().bytes;
        assert!(retained - original < 512 * 1024, "shared B-rep source double charged: before={original}, after={retained}");
        let fallback = coordinator.retained_storage_usage(&coordinator.identities, None, false, &limits).unwrap();
        let indexed = coordinator.validate_internal_retained_owners(&coordinator.identities, None, false, &limits).unwrap();
        assert_eq!(fallback, indexed, "indexed and allocation-free inventories must agree");
        let tight = Limits { max_bytes: fallback.bytes + 64 * 1024, ..limits };
        assert!(tight.max_bytes < original + 1024 * 1024, "fixture must exclude duplicate-inclusive source storage");
        assert_eq!(coordinator.validate_internal_retained_owners(&coordinator.identities, None, false, &tight).unwrap(), fallback);
        drop(capture);
        coordinator.undo().unwrap();
        coordinator.redo().unwrap();
    }

    #[test]
    fn detached_external_brep_versions_share_the_source_allocation() {
        let limits = Limits::default();
        let mut document = Document::new("external B-rep captures");
        let entity = Entity::new("shape", Geometry::Brep(source_fixture()), None);
        let entity_id = entity.id;
        document.entities.insert(entity_id, entity);
        let id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let first = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).name = "second".into();
        let second = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).entities.remove(&entity_id);
        let both = coordinator.retained_usage().unwrap().bytes;
        drop(second);
        let one = coordinator.retained_usage().unwrap().bytes;
        assert!(both - one < 512 * 1024, "external versions double charged B-rep source");
        drop(first);
        assert!(one - coordinator.retained_usage().unwrap().bytes >= 1024 * 1024);
    }

    #[test]
    fn shared_source_aliases_are_charged_once_across_documents() {
        let limits = Limits::default();
        let mut document = Document::new("shared B-rep aliases");
        let source = source_fixture();
        let first = Entity::new("first", Geometry::Brep(source.clone()), None);
        let second = Entity::new("second", Geometry::Brep(source), None);
        let first_id = first.id;
        let second_id = second.id;
        document.entities.insert(first.id, first);
        document.entities.insert(second.id, second);
        let id = document.id;
        let storage = document.storage_usage(&limits).unwrap();
        let tight = Limits { max_bytes: storage.bytes + 128 * 1024, ..limits };
        assert!(tight.max_bytes < 2 * 1024 * 1024);
        assert_eq!(document.validate(&tight).unwrap(), storage, "document validation must admit the shared allocation once");
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).entities.remove(&second_id);
        let both = coordinator.retained_usage().unwrap().bytes;
        let fallback = coordinator.retained_storage_usage(&coordinator.identities, None, false, &limits).unwrap();
        let indexed = coordinator.validate_internal_retained_owners(&coordinator.identities, None, false, &limits).unwrap();
        assert_eq!(fallback, indexed);
        drop(capture);
        let one = coordinator.retained_usage().unwrap().bytes;
        assert!(both - one < 512 * 1024, "aliases must not duplicate the source charge");
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).entities.remove(&first_id);
        assert!(one - coordinator.retained_usage().unwrap().bytes >= 1024 * 1024, "last owner must retain the source charge");
    }

    #[test]
    fn equal_but_independent_brep_sources_are_charged_separately() {
        let limits = Limits::default();
        let mut document = Document::new("independent B-rep source");
        let first = source_fixture();
        let entity = Entity::new("shape", Geometry::Brep(first), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let document_id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_retained_document(document_id).unwrap();
        let current = Arc::make_mut(coordinator.world.documents.get_mut(&document_id).unwrap());
        let Geometry::Brep(old) = &current.entities[&id].geometry else { panic!("B-rep") };
        let independent = crate::brep::Brep::from_native(old.source().to_string(), (*old.display).clone(), &limits).unwrap();
        current.entities.get_mut(&id).unwrap().geometry = Geometry::Brep(independent);
        let with_both = coordinator.retained_usage().unwrap().bytes;
        drop(capture);
        let one = coordinator.retained_usage().unwrap().bytes;
        assert!(with_both - one >= 1024 * 1024, "independent source allocations must both be charged");
    }

    #[test]
    fn shared_brep_source_is_counted_across_entities_and_definitions() {
        let limits = Limits::default();
        let source = source_fixture();
        let mut document = Document::new("shared definition source");
        let entity = Entity::new("root", Geometry::Brep(source.clone()), None);
        document.entities.insert(entity.id, entity);
        let member = Entity::new("member", Geometry::Brep(source), None);
        let definition = crate::model::Definition::new("definition", vec![member]);
        document.definitions.insert(definition.id, definition);
        let storage = document.storage_usage(&limits).unwrap();
        let tight = Limits { max_bytes: storage.bytes + 128 * 1024, ..limits };
        assert!(tight.max_bytes < 2 * 1024 * 1024);
        assert_eq!(document.validate(&tight).unwrap(), storage);
    }

    #[test]
    fn world_shares_brep_source_across_distinct_documents() {
        let limits = Limits::default();
        let source = source_fixture();
        let mut first = Document::new("first document");
        let entity = Entity::new("first", Geometry::Brep(source.clone()), None);
        first.entities.insert(entity.id, entity);
        let mut second = Document::new("second document");
        for name in ["second", "third"] {
            let entity = Entity::new(name, Geometry::Brep(source.clone()), None);
            second.entities.insert(entity.id, entity);
        }
        let tight = Limits { max_bytes: 1536 * 1024, ..limits };
        let world = World::new(vec![first, second], &tight).unwrap();
        let usage = world.validate(&tight).unwrap();
        assert!(usage.bytes >= 1024 * 1024 && usage.bytes < tight.max_bytes);
    }

    #[test]
    fn world_does_not_merge_equal_independent_brep_sources() {
        let limits = Limits::default();
        let mut first = Document::new("first document");
        let entity = Entity::new("first", Geometry::Brep(source_fixture()), None);
        first.entities.insert(entity.id, entity);
        let mut second = Document::new("second document");
        let entity = Entity::new("second", Geometry::Brep(source_fixture()), None);
        second.entities.insert(entity.id, entity);
        let tight = Limits { max_bytes: 1536 * 1024, ..limits };
        assert!(matches!(World::new(vec![first, second], &tight), Err(Error::Budget("bytes"))));
    }

    #[test]
    fn world_still_validates_each_brep_with_a_shared_source() {
        let limits = Limits::default();
        let source = source_fixture();
        let mut first = Document::new("first document");
        let entity = Entity::new("valid", Geometry::Brep(source.clone()), None);
        first.entities.insert(entity.id, entity);
        let mut invalid = source;
        Arc::make_mut(&mut invalid.display).points[0][0] = f64::NAN;
        let mut second = Document::new("second document");
        let entity = Entity::new("invalid", Geometry::Brep(invalid), None);
        second.entities.insert(entity.id, entity);
        let tight = Limits { max_bytes: 1536 * 1024, ..limits };
        assert!(matches!(World::new(vec![first, second], &tight), Err(Error::Invalid(_))));
    }

    #[test]
    fn external_capture_and_current_document_share_resource_payload_charge() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![23_u8; 4 * 1024 * 1024];
        let resource = Resource::from_bytes("shared", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut document = Document::new("captured");
        document.resources.insert(resource.id, resource);
        let id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).name = "current version".into();
        let retained = coordinator.retained_usage().unwrap().bytes;
        drop(capture);
        let current = coordinator.retained_usage().unwrap().bytes;
        assert!(retained - current < payload.len() as u64 / 2,
            "capture double-charged shared payload: retained={retained}, current={current}");
    }

    #[test]
    fn distinct_external_captures_share_a_payload_after_current_drops_it() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![29_u8; 4 * 1024 * 1024];
        let resource = Resource::from_bytes("shared", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut document = Document::new("first");
        document.resources.insert(resource.id, resource);
        let id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let first = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).name = "second".into();
        let second = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).resources.clear();
        let both = coordinator.retained_usage().unwrap().bytes;
        drop(second);
        let one = coordinator.retained_usage().unwrap().bytes;
        assert!(both - one < payload.len() as u64 / 2,
            "external versions double-charged shared payload: both={both}, one={one}");
        drop(first);
        assert!(one - coordinator.retained_usage().unwrap().bytes >= payload.len() as u64);
    }

    #[test]
    fn external_capture_and_current_equal_but_independent_resources_are_both_charged() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![41_u8; 1024 * 1024];
        let original = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let id = original.id;
        let mut document = Document::new("source");
        document.resources.insert(id, original);
        let document_id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_retained_document(document_id).unwrap();
        let mut replacement = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        replacement.id = id;
        Arc::make_mut(coordinator.world.documents.get_mut(&document_id).unwrap()).resources.insert(id, replacement);
        let with_capture = coordinator.retained_usage().unwrap().bytes;
        drop(capture);
        let without_capture = coordinator.retained_usage().unwrap().bytes;
        assert!(with_capture - without_capture >= payload.len() as u64,
            "distinct allocations with equal contents must both be accounted");
    }

    #[test]
    fn shared_resource_capture_remains_deduplicated_after_coordinator_replacement() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![43_u8; 1024 * 1024];
        let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut old_document = Document::new("old");
        old_document.resources.insert(resource.id, resource.clone());
        let old_id = old_document.id;
        let old = TransactionCoordinator::new(World::new(vec![old_document], &limits).unwrap(), limits).unwrap();
        let capture = old.capture_retained_document(old_id).unwrap();
        let mut new_document = Document::new("new");
        new_document.resources.insert(resource.id, resource);
        let mut new = TransactionCoordinator::new(World::new(vec![new_document], &limits).unwrap(), limits).unwrap();
        new.adopt_external_captures(&old).unwrap();
        drop(old);
        let with_capture = new.retained_usage().unwrap().bytes;
        drop(capture);
        let without_capture = new.retained_usage().unwrap().bytes;
        assert!(with_capture - without_capture < payload.len() as u64 / 2,
            "resource shared across a replacement is still only one allocation");
    }

    #[test]
    fn external_alias_multiplicity_is_conservative_across_current_and_capture() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![47_u8; 1024 * 1024];
        let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut alias = resource.clone();
        alias.id = ResourceId::new();
        let alias_id = alias.id;
        let mut document = Document::new("two aliases");
        document.resources.insert(resource.id, resource);
        document.resources.insert(alias.id, alias);
        let id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).resources.remove(&alias_id);
        let with_capture = coordinator.retained_usage().unwrap().bytes;
        drop(capture);
        let without_capture = coordinator.retained_usage().unwrap().bytes;
        assert!(with_capture - without_capture >= payload.len() as u64,
            "preserve the largest per-document alias charge until document validation is deduplicated");
    }

    #[test]
    fn retained_document_versions_charge_shared_resource_payload_once() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![17_u8; 4 * 1024 * 1024];
        let resource = Resource::from_bytes("shared payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut document = Document::new("resource source");
        document.resources.insert(resource.id, resource);
        let id = document.id;
        let before = World::new(vec![document], &limits).unwrap();
        let mut after = before.clone();
        Arc::make_mut(after.documents.get_mut(&id).unwrap()).name = "different document version".into();
        let entry = UndoEntry::between(TransactionId::new(), "rename".into(), &before, &after);
        let mut coordinator = TransactionCoordinator::new(before, limits).unwrap();
        let original = coordinator.retained_usage().unwrap().bytes;
        coordinator.world = after;
        coordinator.undo.push(entry);
        let with_history = coordinator.retained_usage().unwrap().bytes;
        assert!(with_history - original < payload.len() as u64 / 2,
            "shared payload should be charged once: before={original}, history={with_history}");
        coordinator.limits.max_bytes = with_history + 1024 * 1024;
        assert_eq!(coordinator.retained_usage().unwrap().bytes, with_history,
            "admission under pressure must not fall back to counting the shared payload twice");
    }

    #[test]
    fn retained_resource_accounting_distinguishes_shared_and_independent_payloads() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![17_u8; 1024 * 1024];
        let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut document = Document::new("resource source");
        let resource_id = resource.id;
        document.resources.insert(resource_id, resource);
        let id = document.id;
        let before = World::new(vec![document], &limits).unwrap();
        let mut after = before.clone();
        let replacement = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut replacement = replacement;
        replacement.id = resource_id;
        let changed = Arc::make_mut(after.documents.get_mut(&id).unwrap());
        changed.resources.insert(resource_id, replacement);
        changed.name = "different owner".into();
        let entry = UndoEntry::between(TransactionId::new(), "replace".into(), &before, &after);
        let mut coordinator = TransactionCoordinator::new(before, limits).unwrap();
        let original = coordinator.retained_usage().unwrap().bytes;
        coordinator.world = after;
        coordinator.undo.push(entry);
        assert!(coordinator.retained_usage().unwrap().bytes >= original + payload.len() as u64,
            "equal bytes in distinct allocations must both be charged");
    }

    #[test]
    fn repeated_resource_alias_within_one_document_remains_conservatively_admitted() {
        use crate::resource::{Resource, ResourceKind};
        let limits = Limits::default();
        let payload = vec![17_u8; 1024 * 1024];
        let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut document = Document::new("resource source");
        document.resources.insert(resource.id, resource.clone());
        let id = document.id;
        let before = World::new(vec![document], &limits).unwrap();
        let mut after = before.clone();
        let mut alias = resource;
        alias.id = crate::identity::ResourceId::new();
        Arc::make_mut(after.documents.get_mut(&id).unwrap()).resources.insert(alias.id, alias);
        let entry = UndoEntry::between(TransactionId::new(), "alias".into(), &before, &after);
        let mut coordinator = TransactionCoordinator::new(before, limits).unwrap();
        coordinator.world = after;
        coordinator.undo.push(entry);
        assert!(coordinator.retained_usage().unwrap().bytes >= 2 * payload.len() as u64,
            "retain per-document multiplicity until its own validation is deduplicated");
    }

    #[test]
    fn export_permit_is_shared_across_replacement_and_released_on_drop() {
        let limits = Limits::default();
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let account = old.external_memory_account();
        let permit = account.try_acquire_export().unwrap();
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        replacement.adopt_external_captures(&old).unwrap();
        assert!(replacement.external_memory_account().try_acquire_export().is_err());
        drop(old);
        drop(permit);
        assert!(replacement.external_memory_account().try_acquire_export().is_ok());
    }

    #[test]
    fn joining_account_retires_stale_job_handles_only_after_success() {
        let limits = Limits::default();
        let source = TransactionCoordinator::new(World::new(vec![Document::new("source")], &limits).unwrap(), limits).unwrap();
        let mut joining = TransactionCoordinator::new(World::new(vec![Document::new("joining")], &limits).unwrap(), limits).unwrap();
        let stale = joining.external_memory_account();
        let active = stale.try_acquire_export().unwrap();
        assert!(joining.adopt_external_captures(&source).is_err());
        drop(active);
        drop(stale.try_acquire_surface_snap().unwrap());
        joining.adopt_external_captures(&source).unwrap();
        assert!(matches!(stale.try_acquire_export(), Err(Error::Invalid("retired memory account"))));
        assert!(matches!(stale.try_acquire_surface_snap(), Err(Error::Invalid("retired memory account"))));
        let current = joining.external_memory_account();
        drop(current.try_acquire_export().unwrap());
        drop(current.try_acquire_surface_snap().unwrap());
    }

    #[test]
    fn remaining_reservation_obeys_physical_headroom_and_releases() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 8 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("reserve")], &limits).unwrap(), limits).unwrap();
        let reservation = coordinator.reserve_remaining_external_memory_with_held(
            MemoryClass::CpuCache, Some((1024 * MIB, 258 * MIB)), Usage::default(),
        ).unwrap();
        assert!(reservation.bytes() > MIB && reservation.bytes() <= 2 * MIB);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), reservation.bytes());
        drop(reservation);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert!(matches!(coordinator.reserve_remaining_external_memory_with_held(
            MemoryClass::CpuCache, Some((1024 * MIB, 256 * MIB)), Usage::default(),
        ), Err(Error::Budget("external memory headroom"))));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn account_adoption_and_stale_export_acquisition_cannot_both_succeed() {
        for _ in 0..32 {
            let limits = Limits::default();
            let source = TransactionCoordinator::new(World::new(vec![Document::new("source")], &limits).unwrap(), limits).unwrap();
            let mut joining = TransactionCoordinator::new(World::new(vec![Document::new("joining")], &limits).unwrap(), limits).unwrap();
            let stale = joining.external_memory_account();
            let start = Arc::new(std::sync::Barrier::new(2));
            let (adopt_tx, adopt_rx) = std::sync::mpsc::channel();
            let (permit_tx, permit_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let other_start = Arc::clone(&start);
            let adopter = std::thread::spawn(move || {
                other_start.wait();
                adopt_tx.send(joining.adopt_external_captures(&source).is_ok()).unwrap();
            });
            let worker = std::thread::spawn(move || {
                start.wait();
                let permit = stale.try_acquire_export().ok();
                permit_tx.send(permit.is_some()).unwrap();
                release_rx.recv().unwrap();
                drop(permit);
            });
            let adopted = adopt_rx.recv().unwrap();
            let acquired = permit_rx.recv().unwrap();
            release_tx.send(()).unwrap();
            adopter.join().unwrap();
            worker.join().unwrap();
            assert_ne!(adopted, acquired, "only one account may own the active job");
        }
    }

    #[test]
    fn simultaneous_export_permits_admit_only_one_worker() {
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("parallel")], &limits).unwrap(), limits).unwrap();
        let account = coordinator.external_memory_account();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = (0..2).map(|_| {
            let account = account.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let permit = account.try_acquire_export().ok();
                barrier.wait();
                permit.is_some()
            })
        }).collect();
        assert_eq!(threads.into_iter().map(|thread| thread.join().unwrap()).filter(|success| *success).count(), 1);
        assert!(account.try_acquire_export().is_ok());
    }

    #[test]
    fn surface_snap_permit_serializes_shared_account_and_follows_replacement() {
        let limits = Limits::default();
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let account = old.external_memory_account();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = (0..2).map(|_| {
            let account = account.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let permit = account.try_acquire_surface_snap().ok();
                barrier.wait();
                permit.is_some()
            })
        }).collect();
        assert_eq!(threads.into_iter().map(|thread| thread.join().unwrap()).filter(|success| *success).count(), 1);
        let permit = account.try_acquire_surface_snap().unwrap();
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        replacement.adopt_external_captures(&old).unwrap();
        assert!(replacement.external_memory_account().try_acquire_surface_snap().is_err());
        drop(permit);
        assert!(replacement.external_memory_account().try_acquire_surface_snap().is_ok());
    }

    #[test]
    fn external_reservations_are_counted_and_follow_replacement() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let initial = old.retained_usage().unwrap().bytes;
        let reservation = old.reserve_external_memory(MemoryClass::Export, 10_000, None).unwrap();
        assert_eq!(old.retained_usage().unwrap().bytes, initial + 10_000);
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        let baseline = replacement.retained_usage().unwrap().bytes;
        replacement.adopt_external_captures(&old).unwrap();
        drop(old);
        let with_reservation = replacement.retained_usage().unwrap().bytes;
        drop(reservation);
        let after_release = replacement.retained_usage().unwrap().bytes;
        assert_eq!(with_reservation - after_release, 10_000);
        assert!(after_release >= baseline && after_release - baseline < 1024, "only small registry capacity should remain");
    }

    #[test]
    fn adopting_account_registers_joining_world_for_existing_peers() {
        let limits = Limits::default();
        let source = TransactionCoordinator::new(World::new(vec![Document::new("source")], &limits).unwrap(), limits).unwrap();
        let before = source.retained_usage().unwrap().bytes;
        let mut joining_document = Document::new("joining");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        joining_document.entities.insert(entity.id, entity);
        let mut joining = TransactionCoordinator::new(World::new(vec![joining_document], &limits).unwrap(), limits).unwrap();
        joining.adopt_external_captures(&source).unwrap();
        let together = source.retained_usage().unwrap().bytes;
        assert!(together - before >= 1024 * 1024, "existing source must count joining world");
        drop(joining);
        let after = source.retained_usage().unwrap().bytes;
        assert!(together - after >= 1024 * 1024, "joining world charge ends when its owner drops");
    }

    #[test]
    fn failed_account_adoption_rolls_back_partial_joining_world_registration() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 3 * MIB, ..Limits::default() };
        let cloud_document = |name| {
            let mut document = Document::new(name);
            let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
                points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
            }), None);
            document.entities.insert(entity.id, entity);
            document
        };
        let source = TransactionCoordinator::new(World::new(vec![cloud_document("source")], &limits).unwrap(), limits).unwrap();
        let mut joining = TransactionCoordinator::new(World::new(vec![cloud_document("one"), cloud_document("two")], &limits).unwrap(), limits).unwrap();
        let source_before = source.retained_usage().unwrap().bytes;
        let joining_before = joining.snapshot();
        let joining_account = joining.external_memory_account();
        assert!(matches!(joining.adopt_external_captures(&source), Err(Error::Budget(_))));
        assert_eq!(joining.snapshot(), joining_before);
        drop(joining_account.try_acquire_export().unwrap());
        drop(joining_account.try_acquire_surface_snap().unwrap());
        let source_after = source.retained_usage().unwrap().bytes;
        assert!(source_after - source_before < 4096, "failed adoption must not leave joining documents charged");
    }

    #[test]
    fn adopting_account_registers_initial_source_world_for_peer() {
        let limits = Limits::default();
        let mut source_document = Document::new("initial world");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        source_document.entities.insert(entity.id, entity);
        let source = TransactionCoordinator::new(World::new(vec![source_document], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        let before = peer.retained_usage().unwrap().bytes;
        peer.adopt_external_captures(&source).unwrap();
        let shared = peer.retained_usage().unwrap().bytes;
        assert!(shared - before >= 1024 * 1024, "peer must account for source's initial document");
        drop(source);
        let released = peer.retained_usage().unwrap().bytes;
        assert!(shared - released >= 1024 * 1024, "source document charge must end with its owner");
    }

    #[test]
    fn peer_accounts_document_opened_and_releases_it_after_close() {
        let limits = Limits::default();
        let mut current = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = peer.retained_usage().unwrap().bytes;
        let mut document = Document::new("incoming");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        document.entities.insert(entity.id, entity);
        let id = document.id;
        current.open_document(document).unwrap();
        let opened = peer.retained_usage().unwrap().bytes;
        assert!(opened - before >= 1024 * 1024, "peer must count the opened document");
        current.close_document(id).unwrap();
        let closed = peer.retained_usage().unwrap().bytes;
        assert!(opened - closed >= 1024 * 1024, "peer must release the closed document");
    }

    #[test]
    fn document_open_does_not_double_charge_registered_candidate() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 2 * MIB, ..Limits::default() };
        let mut current = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut document = Document::new("incoming");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        document.entities.insert(entity.id, entity);
        let id = document.id;
        let bytes = document.validate(&limits).unwrap().bytes;
        assert!(bytes < limits.max_bytes && bytes * 2 > limits.max_bytes);
        current.open_document(document).unwrap();
        assert!(current.world.documents.contains_key(&id));
    }

    #[test]
    fn scope_finish_rechecks_competing_reservation_and_keeps_redo_on_failure() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 1024 * MIB, ..Limits::default() };
        let mut document = Document::new("scope finish race");
        let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let prior = current.prepare(TransactionRequest::new("prior", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        current.commit(prior).unwrap();
        current.undo().unwrap();
        let token = current.book("scope", true).unwrap();
        current.begin(&token).unwrap();
        let source = current.editing_world(&token).unwrap().documents[&doc].stamp(id).unwrap();
        let mut request = TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([2., 0., 0.]),
        }]);
        request.owner = Some(token.clone());
        current.commit(current.prepare(request).unwrap()).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = current.snapshot();
        let mut reservation = None;
        let result = current.finish_with_guard(&token, Usage { bytes: 750 * MIB, ..Usage::default() }, || {
            reservation = Some(peer.reserve_external_memory(MemoryClass::Export, 300 * MIB, None)?);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Budget(_))), "competing reservation must refuse finish: {result:?}");
        assert_eq!(current.snapshot(), before);
        assert_eq!(current.redo_len(), 1);
        assert!(current.editing_world(&token).is_ok());
        drop(reservation);
        assert!(current.finish(&token).unwrap().published);
    }

    #[test]
    fn peer_accounts_scoped_document_until_abort_or_finish() {
        let limits = Limits::default();
        let mut document = Document::new("scoped document");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        current.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let token = current.book("scope", true).unwrap();
        current.begin(&token).unwrap();
        let source = current.editing_world(&token).unwrap().documents[&doc].stamp(id).unwrap();
        let mut request = TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }]);
        request.owner = Some(token.clone());
        current.commit(current.prepare(request).unwrap()).unwrap();
        let with_staged = peer.retained_usage().unwrap().bytes;
        current.abort(&token).unwrap();
        let after_abort = peer.retained_usage().unwrap().bytes;
        assert!(with_staged - after_abort >= 1024 * 1024, "peer must charge private staged document until abort");
        let token = current.book("scope again", true).unwrap();
        current.begin(&token).unwrap();
        let source = current.editing_world(&token).unwrap().documents[&doc].stamp(id).unwrap();
        let mut request = TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([2., 0., 0.]),
        }]);
        request.owner = Some(token.clone());
        current.commit(current.prepare(request).unwrap()).unwrap();
        current.finish(&token).unwrap();
        let after_finish = peer.retained_usage().unwrap().bytes;
        assert!(after_finish - after_abort >= 1024 * 1024, "finished version must remain charged");
        current.undo().unwrap();
        current.redo().unwrap();
        assert!(peer.retained_usage().unwrap().bytes >= after_finish);
    }

    #[test]
    fn peer_accounts_direct_edit_versions_through_undo_and_redo() {
        let limits = Limits::default();
        let mut document = Document::new("edited document");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        current.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = peer.retained_usage().unwrap().bytes;
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let prepared = current.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        current.commit(prepared).unwrap();
        let after = peer.retained_usage().unwrap().bytes;
        assert!(after - before >= 1024 * 1024, "peer must count new document version");
        current.undo().unwrap();
        assert!(peer.retained_usage().unwrap().bytes >= after);
        current.redo().unwrap();
        assert!(peer.retained_usage().unwrap().bytes >= after);
        drop(current);
        assert!(after - peer.retained_usage().unwrap().bytes >= 2 * 1024 * 1024);
    }

    #[test]
    fn prepared_document_and_scene_reservation_are_visible_to_shared_peer_until_drop() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut document = Document::new("prepared edit");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        current.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = peer.retained_usage().unwrap().bytes;
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let prepared = current.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        let candidate = current.register_prepared_document(&prepared, doc).unwrap();
        let reservation = current.reserve_external_memory(MemoryClass::CpuCache, 4096, None).unwrap();
        let during = peer.retained_usage().unwrap().bytes;
        assert!(during.saturating_sub(before) >= 1024 * 1024 + 4096);
        drop(candidate);
        drop(prepared);
        drop(reservation);
        let after = peer.retained_usage().unwrap().bytes;
        assert!(during.saturating_sub(after) >= 1024 * 1024 + 4096);
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let mut prepared = current.prepare(TransactionRequest::new("move again", vec![Mutation::Placement {
            source, placement: Transform::translation([2., 0., 0.]),
        }])).unwrap();
        let candidate = current.register_prepared_document(&prepared, doc).unwrap();
        let reservation = current.reserve_external_memory(MemoryClass::CpuCache, 4096, None).unwrap();
        current.reserve_prepared_held_usage(&mut prepared, Usage::default()).unwrap();
        current.commit(prepared).unwrap();
        drop(candidate);
        drop(reservation);
        assert!(peer.retained_usage().unwrap().bytes.saturating_sub(after) >= 1024 * 1024,
            "committed candidate stays charged after the temporary scene reservation is released");
    }

    #[test]
    fn prepared_document_is_visible_to_peer_before_scene_registration_and_released_on_drop() {
        let limits = Limits::default();
        let mut document = Document::new("prepared ownership");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        current.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = peer.retained_usage().unwrap().bytes;
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let prepared = current.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        let during = peer.retained_usage().unwrap().bytes;
        assert!(during.saturating_sub(before) >= 1_000_000, "prepared candidate must be accounted: {before} -> {during}");
        drop(prepared);
        let after = peer.retained_usage().unwrap().bytes;
        assert!(during.saturating_sub(after) >= 1_000_000, "candidate charge must expire on drop");
    }

    #[test]
    fn prepared_document_blocks_competing_reservation_until_dropped() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits { max_bytes: 3 * 1024 * 1024, ..Limits::default() };
        let mut document = Document::new("prepared budget");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        current.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = current.snapshot();
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let prepared = current.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, 1024 * 1024, None), Err(Error::Budget(_))));
        drop(prepared);
        assert_eq!(current.snapshot(), before);
        assert!(peer.reserve_external_memory(MemoryClass::Export, 1024 * 1024, None).is_ok());
    }

    #[test]
    fn document_clone_reservation_blocks_peer_before_allocation_and_transfers_on_success() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits { max_bytes: 3 * 1024 * 1024, ..Limits::default() };
        let mut document = Document::new("clone reservation");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        current.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let before = current.snapshot();
        let source = current.world.documents[&doc].stamp(id).unwrap();
        let request = || TransactionRequest::new("move", vec![Mutation::Placement {
            source: source.clone(), placement: Transform::translation([1., 0., 0.]),
        }]);
        let canceled = current.prepare_with_held_usage_and_guard(request(), Usage::default(), || {
            assert!(current.external_reserved_bytes(MemoryClass::CpuCache) >= 1_000_000);
            assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, 1024 * 1024, None), Err(Error::Budget(_))));
            Err(Error::Canceled)
        });
        assert!(matches!(canceled, Err(Error::Canceled)));
        assert_eq!(current.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(current.external_pending_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(current.snapshot(), before);
        let prepared = current.prepare_with_held_usage_and_guard(request(), Usage::default(), || {
            assert!(current.external_reserved_bytes(MemoryClass::CpuCache) >= 1_000_000);
            assert!(peer.reserve_external_memory(MemoryClass::Export, 1024 * 1024, None).is_err());
            Ok(())
        }).unwrap();
        assert_eq!(current.external_reserved_bytes(MemoryClass::CpuCache), 0, "clone charge transferred to candidate");
        assert_eq!(current.external_pending_bytes(MemoryClass::CpuCache), 0, "allocated candidate must not leave stale pending bytes");
        assert!(peer.reserve_external_memory(MemoryClass::Export, 1024 * 1024, None).is_err());
        drop(prepared);
        assert!(peer.reserve_external_memory(MemoryClass::Export, 1024 * 1024, None).is_ok());
        assert_eq!(current.snapshot(), before);
    }

    #[test]
    fn clone_reservation_checks_sampled_free_ram_before_copy_and_preserves_redo() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let mut document = Document::new("physical clone headroom");
        let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        coordinator.undo().unwrap();
        assert_eq!(coordinator.redo_len(), 1);
        coordinator.set_physical_memory_sampler(|| Some((1024 * MIB, 256 * MIB)));
        let before = coordinator.snapshot();
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let result = coordinator.prepare_with_held_usage_and_guard(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([2., 0., 0.]),
        }]), Usage::default(), || panic!("must refuse before cloning"));
        assert!(matches!(result, Err(Error::Budget("available physical RAM"))));
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.redo_len(), 1);
        assert_eq!(coordinator.external_reserved_bytes(crate::budget::memory::MemoryClass::CpuCache), 0);
        coordinator.set_physical_memory_sampler(|| None);
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("retry", vec![Mutation::Placement {
            source, placement: Transform::translation([2., 0., 0.]),
        }])).unwrap();
        drop(prepared);
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn transaction_planning_indexes_are_reserved_before_build_and_released_on_cancel() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut document = Document::new("planning reservation");
        let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        coordinator.register_retained_documents().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("planning peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&coordinator).unwrap();
        let peer_before = peer.retained_usage().unwrap().bytes;
        let before = coordinator.snapshot();
        let source = coordinator.world.documents[&doc].stamp(id).unwrap();
        let result = coordinator.prepare_with_held_usage_and_guards(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }]), Usage::default(), || {
            let reserved = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
            assert!(reserved > 0);
            assert!(peer.retained_usage().unwrap().bytes >= peer_before + reserved);
            Err(Error::Canceled)
        }, || panic!("canceled before planning"));
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(peer.retained_usage().unwrap().bytes, peer_before);
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn registered_candidate_remains_visible_to_peer_after_publication() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut current = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&current).unwrap();
        let mut new_document = Document::new("candidate");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        new_document.entities.insert(entity.id, entity);
        let id = new_document.id;
        let mut candidate = TransactionCoordinator::new(World::new(vec![new_document], &limits).unwrap(), limits).unwrap();
        let candidate_bytes = candidate.retained_usage().unwrap().bytes;
        let mut reservation = current.reserve_external_memory(MemoryClass::CpuCache, candidate_bytes + 4096, None).unwrap();
        let candidate_document = candidate.world.documents[&id].clone();
        current.register_external_candidate(&candidate_document).unwrap();
        current.register_retained_documents().unwrap();
        candidate.adopt_external_captures(&current).unwrap();
        let admission = current.replacement_admission();
        let mut old = None;
        {
            let mut guard = admission.lock();
            guard.prepare_candidate_reservation(&current, &mut reservation,
                Usage { bytes: candidate_bytes, ..Usage::default() }, Usage::default()).unwrap()
                .publish(|| old = Some(std::mem::replace(&mut current, candidate)));
        }
        drop(old);
        drop(candidate_document);
        let peer_with_candidate = peer.retained_usage().unwrap().bytes;
        drop(current);
        let peer_after_close = peer.retained_usage().unwrap().bytes;
        assert!(peer_with_candidate - peer_after_close >= 1024 * 1024,
            "peer must account for candidate document until its owner drops");
    }

    #[test]
    fn replacement_keeps_old_auxiliary_reserved_until_retired_guard_drops() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let current = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut candidate = current.reserve_external_memory(MemoryClass::CpuCache, 3 * MIB, None).unwrap();
        let admission = current.replacement_admission();
        let retired;
        {
            let mut lock = admission.lock();
            let transfer = lock.prepare_candidate_reservation(&current, &mut candidate,
                Usage { bytes: 2 * MIB, ..Usage::default() }, Usage { bytes: 4 * MIB, ..Usage::default() }).unwrap();
            retired = transfer.publish_retaining_old(|| {});
        }
        assert!(retired.bytes() >= 4 * MIB);
        assert_eq!(current.external_reserved_bytes(MemoryClass::CpuCache), retired.bytes());
        assert!(matches!(current.reserve_external_memory(MemoryClass::Export, 13 * MIB, None), Err(Error::Budget(_))));
        drop(retired);
        assert_eq!(current.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert!(current.reserve_external_memory(MemoryClass::Export, 13 * MIB, None).is_ok());
    }

    #[test]
    fn retired_reservation_does_not_deadlock_or_leak_on_publication_panic() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let current = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut candidate = current.reserve_external_memory(MemoryClass::CpuCache, 1_000_000, None).unwrap();
        let admission = current.replacement_admission();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut lock = admission.lock();
            lock.prepare_candidate_reservation(&current, &mut candidate,
                Usage { bytes: 500_000, ..Usage::default() }, Usage { bytes: 100_000, ..Usage::default() }).unwrap()
                .publish_retaining_old(|| panic!("test publication panic"));
        }));
        assert!(outcome.is_err());
        assert_eq!(current.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn replacement_registers_old_undo_versions_until_old_coordinator_drops() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut document = Document::new("old history");
        let entity = Entity::new("cloud", Geometry::Cloud(crate::geometry::Cloud {
            points: vec![[0., 0., 0.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        let id = entity.id;
        let doc = document.id;
        document.entities.insert(id, entity);
        let mut current = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        for x in 1..=3 {
            let source = current.world.documents[&doc].stamp(id).unwrap();
            let prepared = current.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
                source, placement: Transform::translation([x as f64, 0., 0.]),
            }])).unwrap();
            current.commit(prepared).unwrap();
        }
        assert_eq!(current.undo_len(), 3);
        current.undo().unwrap();
        assert_eq!((current.undo_len(), current.redo_len()), (2, 1));
        current.register_retained_documents().unwrap();
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        let candidate_bytes = replacement.retained_usage().unwrap().bytes;
        let mut reservation = current.reserve_external_memory(MemoryClass::CpuCache, candidate_bytes + 4096, None).unwrap();
        replacement.adopt_external_captures(&current).unwrap();
        let admission = current.replacement_admission();
        let mut old = None;
        {
            let mut lock = admission.lock();
            lock.prepare_candidate_reservation(&current, &mut reservation,
                Usage { bytes: candidate_bytes, ..Usage::default() }, Usage::default()).unwrap()
                .publish(|| old = Some(std::mem::replace(&mut current, replacement)));
        }
        let old_live = current.retained_usage().unwrap().bytes;
        drop(old);
        let old_dropped = current.retained_usage().unwrap().bytes;
        assert!(old_live - old_dropped >= 4 * 1024 * 1024, "all old cloud document versions must remain charged while old history lives: {old_live}, {old_dropped}");
    }

    #[test]
    fn replaced_document_stays_accounted_until_old_owner_is_dropped() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut old_document = Document::new("old retained source");
        let entity = Entity::new("source", Geometry::Brep(source_fixture()), None);
        old_document.entities.insert(entity.id, entity);
        let old_id = old_document.id;
        let mut current = TransactionCoordinator::new(World::new(vec![old_document], &limits).unwrap(), limits).unwrap();
        let old_capture = current.capture_retained_document(old_id).unwrap();
        let new_document = Document::new("new");
        let mut replacement = TransactionCoordinator::new(World::new(vec![new_document], &limits).unwrap(), limits).unwrap();
        let candidate_bytes = replacement.retained_usage().unwrap().bytes;
        let mut reservation = current.reserve_external_memory(MemoryClass::CpuCache, candidate_bytes + 4096, None).unwrap();
        replacement.adopt_external_captures(&current).unwrap();
        let admission = current.replacement_admission();
        let mut old = None;
        {
            let mut lock = admission.lock();
            lock.prepare_candidate_reservation(&current, &mut reservation,
                Usage { bytes: candidate_bytes, ..Usage::default() }, Usage::default()).unwrap()
                .publish(|| old = Some(std::mem::replace(&mut current, replacement)));
        }
        let with_old = current.retained_usage().unwrap().bytes;
        drop(old);
        assert_eq!(current.retained_usage().unwrap().bytes, with_old, "capture keeps old document accounted after old coordinator drops");
        drop(old_capture);
        let without_old = current.retained_usage().unwrap().bytes;
        assert!(with_old - without_old >= 1024 * 1024, "old payload must be accounted until release");
    }

    #[test]
    fn candidate_transfer_allows_actual_coordinator_swap_under_lock() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut current = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let new_document = Document::new("new");
        let new_id = new_document.id;
        let mut replacement = TransactionCoordinator::new(World::new(vec![new_document], &limits).unwrap(), limits).unwrap();
        let candidate_bytes = replacement.retained_usage().unwrap().bytes;
        let mut reservation = current.reserve_external_memory(MemoryClass::CpuCache, candidate_bytes + 1024, None).unwrap();
        replacement.adopt_external_captures(&current).unwrap();
        let admission = current.replacement_admission();
        let mut old = None;
        {
            let mut guard = admission.lock();
            let transfer = guard.prepare_candidate_reservation(&current, &mut reservation,
                Usage { bytes: candidate_bytes, ..Usage::default() }, Usage::default()).unwrap();
            transfer.publish(|| old = Some(std::mem::replace(&mut current, replacement)));
        }
        drop(old);
        assert!(current.world.documents.contains_key(&new_id));
        assert_eq!(current.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn replacement_candidate_reservation_transfer_is_atomic_and_preserves_on_failure() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut candidate = coordinator.reserve_external_memory(MemoryClass::CpuCache, 4 * MIB, None).unwrap();
        let peer = coordinator.reserve_external_memory(MemoryClass::Export, 11 * MIB, None).unwrap();
        let admission = coordinator.replacement_admission();
        let candidate_usage = Usage { bytes: 3 * MIB, ..Usage::default() };
        {
            let mut guard = admission.lock();
            let auxiliary = Usage { bytes: 3 * MIB, ..Usage::default() };
            assert!(matches!(guard.prepare_candidate_reservation(&coordinator, &mut candidate, candidate_usage, auxiliary), Err(Error::Budget(_))));
            assert_eq!(candidate.bytes(), 4 * MIB);
        }
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 4 * MIB);
        drop(peer);
        {
            let mut guard = admission.lock();
            assert!(matches!(guard.prepare_candidate_reservation(&coordinator, &mut candidate,
                Usage { bytes: 5 * MIB, ..Usage::default() }, Usage::default()), Err(Error::Budget("replacement candidate reservation"))));
            assert_eq!(candidate.bytes(), 4 * MIB);
            let mut published = false;
            guard.prepare_candidate_reservation(&coordinator, &mut candidate, candidate_usage, Usage::default()).unwrap().publish(|| published = true);
            assert!(published);
            assert_eq!(candidate.bytes(), 0);
        }
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        drop(candidate);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn external_reservation_growth_is_atomic_and_preserves_existing_bytes_on_refusal() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("growth")], &limits).unwrap(), limits).unwrap();
        let foreign = TransactionCoordinator::new(World::new(vec![Document::new("foreign")], &limits).unwrap(), limits).unwrap();
        let original = coordinator.snapshot();
        let mut guard = coordinator.reserve_external_memory(MemoryClass::CpuCache, 4 * MIB, None).unwrap();
        let competing = coordinator.reserve_external_memory(MemoryClass::Export, 6 * MIB, None).unwrap();
        assert!(matches!(coordinator.grow_external_memory_with_held(&mut guard, 8 * MIB, None, Usage::default()), Err(Error::Budget(_))));
        assert_eq!(guard.bytes(), 4 * MIB);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 4 * MIB);
        assert!(matches!(foreign.grow_external_memory_with_held(&mut guard, MIB, None, Usage::default()), Err(Error::Invalid("external reservation account"))));
        drop(competing);
        coordinator.grow_external_memory_with_held(&mut guard, 8 * MIB, None, Usage::default()).unwrap();
        assert_eq!(guard.bytes(), 12 * MIB);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 12 * MIB);
        assert_eq!(coordinator.snapshot(), original);
        drop(guard);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn sampled_external_reservation_with_held_usage_checks_headroom_and_preserves_state() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 1024 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("hover RAM")], &limits).unwrap(), limits).unwrap();
        let before = coordinator.snapshot();
        let held = Usage { bytes: MIB, ..Usage::default() };
        assert!(matches!(coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 2 * MIB, Some((1024 * MIB, 257 * MIB)), held), Err(Error::Budget("available physical RAM"))));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(coordinator.snapshot(), before);
        let accepted = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 2 * MIB, Some((1024 * MIB, 1024 * MIB)), held).unwrap();
        assert!(accepted.physical_sampled());
        drop(accepted);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn external_reservation_counts_caller_held_memory_and_other_reservations() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("scene owner")], &limits).unwrap(), limits).unwrap();
        let held = Usage { bytes: 8 * MIB, ..Usage::default() };
        let first = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 4 * MIB, None, held).unwrap();
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.reserve_external_memory_with_held(MemoryClass::Export, 5 * MIB, None, held), Err(Error::Budget(_))));
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 4 * MIB);
        drop(first);
        assert!(coordinator.reserve_external_memory_with_held(MemoryClass::Export, 5 * MIB, None, held).is_ok());
    }

    #[test]
    fn initialized_managed_ceiling_accounts_held_bytes_and_cannot_widen() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 2 * 1024 * MIB, ..Limits::default() };
        let old = TransactionCoordinator::new(World::new(vec![Document::new("initialized RAM")], &limits).unwrap(), limits).unwrap();
        let before = old.snapshot();
        assert!(matches!(old.constrain_managed_ram(1024 * MIB, Usage { bytes: 620 * MIB, ..Usage::default() }), Err(Error::Budget(_))));
        assert_eq!(old.external_reservations.lock().unwrap().managed_ceiling, None);
        assert_eq!(old.snapshot(), before);
        old.constrain_managed_ram(1024 * MIB, Usage { bytes: 10 * MIB, ..Usage::default() }).unwrap();
        old.constrain_managed_ram(2 * 1024 * MIB, Usage::default()).unwrap();
        assert_eq!(old.external_reservations.lock().unwrap().managed_ceiling, Some(1024 * MIB * 3 / 5));
        assert!(matches!(old.reserve_external_memory(MemoryClass::Export, 620 * MIB, None), Err(Error::Budget("managed RAM"))));
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("replacement")], &limits).unwrap(), limits).unwrap();
        replacement.adopt_external_captures(&old).unwrap();
        assert!(matches!(replacement.reserve_external_memory(MemoryClass::Export, 620 * MIB, None), Err(Error::Budget("managed RAM"))));
    }

    #[test]
    fn external_reservation_with_physical_sample_respects_managed_sixty_percent() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("physical limit")], &limits).unwrap(), limits).unwrap();
        let before = coordinator.retained_usage().unwrap().bytes;
        // Both requests fit the sampled free-RAM headroom separately; their
        // combined held usage must still fit 60% of total physical RAM.
        let first = coordinator.reserve_external_memory(MemoryClass::Export, 400 * MIB, Some((total, total))).unwrap();
        let before_refusal = coordinator.retained_usage().unwrap();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 220 * MIB, Some((total, total))), Err(Error::Budget("managed RAM"))));
        assert_eq!(coordinator.retained_usage().unwrap(), before_refusal);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
        drop(first);
        assert_eq!(coordinator.retained_usage().unwrap().bytes, before);
    }

    #[test]
    fn sampled_managed_ceiling_cannot_be_bypassed_by_later_unsampled_reservation() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("session RAM ceiling")], &limits).unwrap(), limits).unwrap();
        let first = coordinator.reserve_external_memory(MemoryClass::Export, 400 * MIB, Some((total, total))).unwrap();
        let before = coordinator.retained_usage().unwrap();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 220 * MIB, None), Err(Error::Budget("managed RAM"))));
        assert_eq!(coordinator.retained_usage().unwrap(), before);
        drop(first);
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 650 * MIB, None), Err(Error::Budget("managed RAM"))));
    }

    #[test]
    fn failed_free_ram_check_still_retains_valid_capacity_and_preserves_owners() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("RAM pressure")], &limits).unwrap(), limits).unwrap();
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Export, MIB, Some((total, 0))), Err(Error::Budget("available physical RAM"))));
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 650 * MIB, None), Err(Error::Budget("managed RAM"))));
    }

    #[test]
    fn reservation_rechecks_free_ram_other_reservations_and_tightened_policy() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 4096 * MIB;
        let limits = Limits { max_bytes: 2048 * MIB, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("RAM recheck")], &limits).unwrap(), limits).unwrap();
        let first = coordinator.reserve_external_memory(MemoryClass::Export, 100 * MIB, Some((total, total))).unwrap();
        let second = coordinator.reserve_external_memory(MemoryClass::Kernel, 200 * MIB, None).unwrap();
        let before = coordinator.snapshot();
        assert!(matches!(first.recheck_physical(total, 700 * MIB), Err(Error::Budget("available physical RAM"))));
        assert!(first.recheck_physical(total, 800 * MIB).is_ok());
        assert!(matches!(first.recheck_physical(3072 * MIB, 3072 * MIB), Err(Error::Budget("physical RAM policy changed"))));
        assert!(matches!(first.recheck_physical(total, total + 1), Err(Error::Invalid("physical RAM sample"))));
        drop(second);
        assert!(first.recheck_physical(total, 700 * MIB).is_ok());
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 1, Some((3072 * MIB, 0))), Err(Error::Budget("available physical RAM"))));
        assert!(matches!(first.recheck_physical(total, total), Err(Error::Budget("physical RAM policy changed"))));
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 100 * MIB);
    }

    #[test]
    fn allocated_reservations_preserve_managed_charge_and_release_only_pending_headroom() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("phase accounting")], &limits).unwrap(), limits).unwrap();
        let before = coordinator.snapshot();
        let mut owner = coordinator.reserve_external_memory(MemoryClass::CpuCache, 100 * MIB, Some((total, total))).unwrap();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Export, 20 * MIB, Some((total, 300 * MIB))), Err(Error::Budget("available physical RAM"))));
        owner.mark_allocated(80 * MIB).unwrap();
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 100 * MIB);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 20 * MIB);
        assert!(owner.mark_allocated(21 * MIB).is_err());
        assert_eq!(owner.pending_bytes(), 20 * MIB);
        assert!(owner.recheck_physical(total, 300 * MIB).is_ok());
        let job = coordinator.reserve_external_memory(MemoryClass::Export, 20 * MIB, Some((total, 300 * MIB))).unwrap();
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 20 * MIB);
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 500 * MIB, Some((total, total))), Err(Error::Budget("managed RAM"))));
        drop(job);
        let mut part = owner.split_off(30 * MIB).unwrap();
        assert_eq!(owner.pending_bytes(), 0);
        assert_eq!(part.pending_bytes(), 20 * MIB);
        coordinator.grow_external_memory_with_held(&mut part, 5 * MIB, Some((total, total)), Usage::default()).unwrap();
        assert_eq!(part.pending_bytes(), 25 * MIB);
        part.mark_allocated(10 * MIB).unwrap();
        part.shrink_to(12 * MIB).unwrap();
        assert_eq!(part.pending_bytes(), 0);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 82 * MIB);
        part.set_allocated_bytes(4 * MIB).unwrap();
        assert_eq!(part.pending_bytes(), 8 * MIB);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 8 * MIB);
        assert!(part.set_allocated_bytes(13 * MIB).is_err());
        assert_eq!(part.pending_bytes(), 8 * MIB);
        part.set_allocated_bytes(12 * MIB).unwrap();
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
        drop((owner, part));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn registered_allocations_and_phase_totals_follow_the_shared_account() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let previous = TransactionCoordinator::new(World::new(vec![Document::new("old account")], &limits).unwrap(), limits).unwrap();
        // A known 80 MiB allocation already exists. Only the extra 4 MiB is
        // pending; it must fit the 44 MiB free headroom in this injected sample.
        let mut owner = previous.reserve_external_memory_with_allocated(
            MemoryClass::CpuCache, 84 * MIB, 80 * MIB, Some((total, 300 * MIB)), Usage::default(),
        ).unwrap();
        assert_eq!(owner.pending_bytes(), 4 * MIB);
        let mut next = TransactionCoordinator::new(World::new(vec![Document::new("new account")], &limits).unwrap(), limits).unwrap();
        next.adopt_external_captures(&previous).unwrap();
        assert_eq!(next.external_pending_bytes(MemoryClass::CpuCache), 4 * MIB);
        assert_eq!(next.external_reserved_bytes(MemoryClass::CpuCache), 84 * MIB);
        let remaining = next.reserve_remaining_external_memory_with_held(MemoryClass::Export,
            Some((total, 300 * MIB)), Usage::default()).unwrap();
        assert_eq!(remaining.bytes(), 40 * MIB);
        assert!(matches!(owner.recheck_physical(total, 300 * MIB - 1), Err(Error::Budget("available physical RAM"))));
        assert!(owner.recheck_physical(total, 300 * MIB).is_ok());
        drop(remaining);
        owner.mark_allocated(4 * MIB).unwrap();
        assert_eq!(next.external_pending_bytes(MemoryClass::CpuCache), 0);
        drop(owner);
        assert_eq!(next.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert!(next.reserve_external_memory_with_allocated(MemoryClass::CpuCache, 1, 2, None, Usage::default()).is_err());
    }

    #[test]
    fn physical_sample_is_refreshed_after_materialization_for_all_shared_peers() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let mut coordinator = TransactionCoordinator::new(World::new(vec![Document::new("fresh sample")], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&coordinator).unwrap();
        let mut allocated = coordinator.reserve_external_memory(MemoryClass::CpuCache, 100 * MIB, Some((total, total))).unwrap();
        allocated.mark_allocated(100 * MIB).unwrap();
        coordinator.set_physical_memory_sampler(|| Some((1024 * MIB, 300 * MIB)));
        // The caller's earlier sample predates materialization. The peer must
        // refresh it under the account lock rather than spending that old RAM.
        assert!(matches!(peer.reserve_external_memory(MemoryClass::Kernel, 50 * MIB, Some((total, total))), Err(Error::Budget("available physical RAM"))));
        let job = peer.reserve_external_memory(MemoryClass::Kernel, 40 * MIB, Some((total, total))).unwrap();
        assert!(job.recheck_physical(total, total).is_ok());
        coordinator.set_physical_memory_sampler(|| Some((1024 * MIB, 290 * MIB)));
        assert!(matches!(job.recheck_physical(total, total), Err(Error::Budget("available physical RAM"))));
        coordinator.set_physical_memory_sampler(|| None);
        assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))), Err(Error::Unsupported(_))));
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::Kernel), 40 * MIB);
        drop((allocated, job));
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn commit_pressure_is_independent_of_available_physical_ram_and_follows_peers() {
        use crate::budget::memory::{CommitMemory, MemoryClass};
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let mut coordinator = TransactionCoordinator::new(World::new(vec![Document::new("commit pressure")], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("commit peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&coordinator).unwrap();
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 600 * MIB }));
        let before = coordinator.snapshot();
        let mut owner = coordinator.reserve_external_memory(MemoryClass::CpuCache, 100 * MIB, Some((total, total))).unwrap();
        assert!(owner.commit_sampled());
        assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, 80 * MIB, Some((total, total))), Err(Error::Budget("available system commit"))));
        owner.mark_allocated(100 * MIB).unwrap();
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 700 * MIB }));
        let mut job = peer.reserve_external_memory(MemoryClass::Export, 60 * MIB, Some((total, total))).unwrap();
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 100 * MIB);
        assert!(matches!(peer.grow_external_memory_with_held(&mut job, 10 * MIB, Some((total, total)), Usage::default()), Err(Error::Budget("available system commit"))));
        assert_eq!(job.bytes(), 60 * MIB);
        assert_eq!(job.pending_bytes(), 60 * MIB);
        let split = job.split_off(20 * MIB).unwrap();
        assert!(split.commit_sampled());
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 720 * MIB }));
        assert!(matches!(job.recheck_physical(total, total), Err(Error::Budget("available system commit"))));
        drop((owner, job, split));
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::Export), 0);
        coordinator.set_commit_memory_sampler(|| None);
        assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))), Err(Error::Unsupported("system commit availability is unavailable"))));
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 0, used_bytes: 0 }));
        assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))), Err(Error::Invalid("system commit sample"))));
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 700 * MIB }));
        let remaining = peer.reserve_remaining_external_memory_with_held(MemoryClass::Export, Some((total, total)), Usage::default()).unwrap();
        assert_eq!(remaining.bytes(), 68 * MIB);
        assert!(remaining.commit_sampled());
        drop(remaining);
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn simultaneous_requests_cannot_spend_the_same_system_commit_headroom() {
        use crate::budget::memory::{CommitMemory, MemoryClass};
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let mut coordinator = TransactionCoordinator::new(World::new(vec![Document::new("commit race")], &limits).unwrap(), limits).unwrap();
        coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 700 * MIB }));
        let coordinator = Arc::new(coordinator);
        let start = Arc::new(std::sync::Barrier::new(3));
        let mut threads = Vec::new();
        for _ in 0..2 {
            let coordinator = coordinator.clone();
            let start = start.clone();
            threads.push(std::thread::spawn(move || {
                start.wait();
                coordinator.reserve_external_memory(MemoryClass::Kernel, 50 * MIB, Some((1024 * MIB, 1024 * MIB))).ok()
            }));
        }
        start.wait();
        let guards: Vec<_> = threads.into_iter().map(|thread| thread.join().unwrap()).collect();
        assert_eq!(guards.iter().filter(|guard| guard.is_some()).count(), 1);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::Kernel), 50 * MIB);
        drop(guards);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn failed_commit_admission_still_keeps_the_valid_physical_ceiling() {
        use crate::budget::memory::{CommitMemory, MemoryClass};
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        for remaining in [false, true] {
            let limits = Limits { max_bytes: total, ..Limits::default() };
            let mut coordinator = TransactionCoordinator::new(World::new(vec![Document::new("commit ceiling")], &limits).unwrap(), limits).unwrap();
            coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * MIB, used_bytes: 768 * MIB }));
            let result = if remaining {
                coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, Some((total, total)), Usage::default())
            } else {
                coordinator.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total)))
            };
            assert!(matches!(result, Err(Error::Budget(_))));
            coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 4096 * MIB, used_bytes: 0 }));
            assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Export, 700 * MIB, None), Err(Error::Budget("managed RAM"))));
            assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
        }
    }

    #[test]
    fn sampled_managed_ceiling_follows_external_account_across_replacement() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let guard = old.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))).unwrap();
        drop(guard);
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        replacement.adopt_external_captures(&old).unwrap();
        drop(old);
        assert!(matches!(replacement.reserve_external_memory(MemoryClass::Kernel, 650 * MIB, None), Err(Error::Budget("managed RAM"))));
    }

    #[test]
    fn replacement_cannot_discard_its_tighter_sampled_ceiling() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 1024 * MIB, ..Limits::default() };
        let old = TransactionCoordinator::new(World::new(vec![Document::new("old")], &limits).unwrap(), limits).unwrap();
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        let sample_total = 512 * MIB;
        drop(replacement.reserve_external_memory(MemoryClass::Export, MIB, Some((sample_total, sample_total))).unwrap());
        assert!(replacement.adopt_external_captures(&old).is_err());
        assert!(matches!(replacement.reserve_external_memory(MemoryClass::Kernel, 350 * MIB, None), Err(Error::Budget("managed RAM"))));
        assert!(old.retained_usage().is_ok());
    }

    #[test]
    fn sampled_ceiling_is_applied_to_retained_owner_validation() {
        use crate::{budget::memory::MemoryClass, resource::{Resource, ResourceKind}};
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let payload = vec![31_u8; 2 * 1024 * 1024];
        let resource = Resource::from_bytes("retained", ResourceKind::OpaqueBinary, &payload, &limits).unwrap();
        let mut document = Document::new("retained RAM");
        document.resources.insert(resource.id, resource);
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Export, 1, Some((3 * MIB, 0))), Err(Error::Budget("available physical RAM"))));
        assert!(matches!(coordinator.retained_usage(), Err(Error::Budget(_))));
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn sampled_ceiling_includes_held_transaction_and_scene_bytes() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("held RAM")], &limits).unwrap(), limits).unwrap();
        drop(coordinator.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))).unwrap());
        let before = coordinator.snapshot();
        let held = Usage { bytes: 620 * MIB, ..Usage::default() };
        assert!(coordinator.retained_usage_with_held_usage(held).is_err());
        assert!(coordinator.prepare_with_held_usage(TransactionRequest::new("pending", vec![]), held).is_err());
        assert_eq!(coordinator.snapshot(), before);
    }

    #[test]
    fn held_bytes_and_external_reservations_share_sampled_ceiling() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("combined admission")], &limits).unwrap(), limits).unwrap();
        let reservation = coordinator.reserve_external_memory(MemoryClass::Export, 200 * MIB, Some((total, total))).unwrap();
        let before = coordinator.snapshot();
        let held = Usage { bytes: 420 * MIB, ..Usage::default() };
        assert!(matches!(coordinator.retained_usage_with_held_usage(held), Err(Error::Budget(_))));
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 200 * MIB);
        drop(reservation);
        assert!(coordinator.retained_usage_with_held_usage(held).is_ok());
    }

    #[test]
    fn ceiling_tightened_after_prepare_refuses_commit_without_losing_history() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let mut document = Document::new("prepared RAM");
        let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let object = point.id;
        document.entities.insert(object, point);
        let document_id = document.id;
        let source = document.stamp(object).unwrap();
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let prior = TransactionRequest::new("prior", vec![Mutation::Placement { source, placement: Transform::translation([2., 0., 0.]) }]);
        let prior = coordinator.prepare(prior).unwrap();
        coordinator.commit(prior).unwrap();
        coordinator.undo().unwrap();
        assert_eq!(coordinator.redo_len(), 1);
        let source = coordinator.world().documents[&document_id].stamp(object).unwrap();
        let request = TransactionRequest::new("move", vec![Mutation::Placement { source, placement: Transform::translation([1., 0., 0.]) }]);
        let held = Usage { bytes: 620 * MIB, ..Usage::default() };
        let prepared = coordinator.prepare_with_held_usage(request, held).unwrap();
        drop(coordinator.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total))).unwrap());
        let before = coordinator.snapshot();
        assert!(matches!(coordinator.commit(prepared), Err(Error::Budget(_))));
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.redo_len(), 1);
    }

    #[test]
    fn ceiling_tightened_during_commit_guard_cannot_publish_or_discard_redo() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        for recording in [true, false] {
            let limits = Limits { max_bytes: total, ..Limits::default() };
            let mut document = Document::new("commit race");
            let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
            let object = point.id;
            let document_id = document.id;
            document.entities.insert(object, point);
            let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
            let source = coordinator.world().documents[&document_id].stamp(object).unwrap();
            let prior = coordinator.prepare(TransactionRequest::new("prior", vec![Mutation::Placement { source, placement: Transform::translation([2., 0., 0.]) }])).unwrap();
            coordinator.commit(prior).unwrap();
            coordinator.undo().unwrap();
            assert_eq!(coordinator.redo_len(), 1);
            let source = coordinator.world().documents[&document_id].stamp(object).unwrap();
            let mut request = TransactionRequest::new("move", vec![Mutation::Placement { source, placement: Transform::translation([1., 0., 0.]) }]);
            request.recording = recording;
            let held = Usage { bytes: 620 * MIB, ..Usage::default() };
            let prepared = coordinator.prepare_with_held_usage(request, held).unwrap();
            let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
            peer.adopt_external_captures(&coordinator).unwrap();
            let before = coordinator.snapshot();
            let result = coordinator.commit_with_guard(prepared, |_| {
                drop(peer.reserve_external_memory(MemoryClass::Export, MIB, Some((total, total)))?);
                Ok(())
            });
            assert!(matches!(result, Err(Error::Budget(_))), "recording={recording}: {result:?}");
            assert_eq!(coordinator.snapshot(), before);
            assert_eq!(coordinator.redo_len(), 1);
        }
    }

    #[test]
    fn reservation_added_during_commit_guard_cannot_oversubscribe_prepared_peak() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 1024 * MIB, ..Limits::default() };
        for recording in [true, false] {
            let mut document = Document::new("commit reservation race");
            let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
            let object = point.id;
            document.entities.insert(object, point);
            let source = document.stamp(object).unwrap();
            let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
            let mut request = TransactionRequest::new("move", vec![Mutation::Placement { source, placement: Transform::translation([1., 0., 0.]) }]);
            request.recording = recording;
            let prepared = coordinator.prepare_with_held_usage(request, Usage { bytes: 750 * MIB, ..Usage::default() }).unwrap();
            let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
            peer.adopt_external_captures(&coordinator).unwrap();
            let before = coordinator.snapshot();
            let mut reservation = None;
            let result = coordinator.commit_with_guard(prepared, |_| {
                reservation = Some(peer.reserve_external_memory(MemoryClass::Export, 300 * MIB, None)?);
                Ok(())
            });
            assert!(matches!(result, Err(Error::Budget(_))), "recording={recording}: {result:?}");
            assert_eq!(coordinator.snapshot(), before);
            drop(reservation);
            assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
        }
    }

    #[test]
    fn open_document_rechecks_competing_reservation_before_publication() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 16 * MIB, ..Limits::default() };
        let mut document = Document::new("existing");
        let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let id = point.id;
        let doc_id = document.id;
        document.entities.insert(id, point);
        let source = document.stamp(id).unwrap();
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([1., 0., 0.]),
        }])).unwrap();
        coordinator.commit(prepared).unwrap();
        coordinator.undo().unwrap();
        assert_eq!(coordinator.redo_len(), 1);
        let mut incoming = Document::new("incoming");
        let cloud = crate::geometry::Cloud { points: vec![[0., 0., 0.]; 150_000], normals: vec![], colors: vec![], hidden: vec![] };
        let entity = Entity::new("cloud", Geometry::Cloud(cloud), None);
        incoming.entities.insert(entity.id, entity);
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&coordinator).unwrap();
        let before = coordinator.snapshot();
        let mut held = None;
        let result = coordinator.open_document_with_guard(incoming, || {
            held = Some(peer.reserve_external_memory(MemoryClass::Export, 14 * MIB, None)?);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Budget(_))), "open must not publish over the shared limit: {result:?}");
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.redo_len(), 1);
        assert!(coordinator.world().documents.contains_key(&doc_id));
        drop(held);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
        let mut incoming = Document::new("retry");
        let cloud = crate::geometry::Cloud { points: vec![[0., 0., 0.]; 150_000], normals: vec![], colors: vec![], hidden: vec![] };
        let entity = Entity::new("cloud", Geometry::Cloud(cloud), None);
        incoming.entities.insert(entity.id, entity);
        let incoming_id = incoming.id;
        coordinator.open_document(incoming).unwrap();
        assert!(coordinator.world().documents.contains_key(&incoming_id));
        assert_eq!(coordinator.redo_len(), 1, "successful open keeps the existing history policy");
    }

    #[test]
    fn simultaneous_sampled_reservations_cannot_exceed_managed_ram_ceiling() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let total = 1024 * MIB;
        let limits = Limits { max_bytes: total, ..Limits::default() };
        let coordinator = Arc::new(TransactionCoordinator::new(World::new(vec![Document::new("sampled reservation race")], &limits).unwrap(), limits).unwrap());
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let workers: Vec<_> = (0..2).map(|_| {
            let coordinator = Arc::clone(&coordinator);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let guard = coordinator.reserve_external_memory(MemoryClass::Export, 320 * MIB, Some((total, total))).ok();
                barrier.wait();
                guard.is_some()
            })
        }).collect();
        assert_eq!(workers.into_iter().map(|worker| worker.join().unwrap()).filter(|success| *success).count(), 1);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
    }

    #[test]
    fn simultaneous_external_reservations_cannot_oversubscribe() {
        use crate::budget::memory::MemoryClass;
        let limits = Limits { max_bytes: 1024 * 1024, ..Limits::default() };
        let coordinator = Arc::new(TransactionCoordinator::new(World::new(vec![Document::new("budget")], &limits).unwrap(), limits).unwrap());
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let mut threads = Vec::new();
        for _ in 0..2 {
            let coordinator = Arc::clone(&coordinator);
            let barrier = Arc::clone(&barrier);
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                coordinator.reserve_external_memory(MemoryClass::Export, 700_000, None).ok()
            }));
        }
        barrier.wait();
        let guards: Vec<_> = threads.into_iter().map(|thread| thread.join().unwrap()).collect();
        assert_eq!(guards.iter().filter(|guard| guard.is_some()).count(), 1);
        let before_drop = coordinator.retained_usage().unwrap().bytes;
        drop(guards);
        assert_eq!(before_drop - coordinator.retained_usage().unwrap().bytes, 700_000);
    }

    #[test]
    fn external_reservations_preserve_free_headroom_and_leave_world_unchanged_on_failure() {
        use crate::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("headroom")], &limits).unwrap(), limits).unwrap();
        let before = coordinator.snapshot();
        let sample = Some((8 * 1024 * MIB, 1000 * MIB));
        let first = coordinator.reserve_external_memory(MemoryClass::Export, 120 * MIB, sample).unwrap();
        assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Kernel, 120 * MIB, sample), Err(Error::Budget("available physical RAM"))));
        assert_eq!(coordinator.snapshot(), before);
        drop(first);
        assert!(coordinator.reserve_external_memory(MemoryClass::Kernel, 120 * MIB, sample).is_ok());
    }

    #[test]
    fn external_capture_is_charged_after_coordinator_replacement() {
        let limits = Limits::default();
        let old_document = Document::new("old captured");
        let id = old_document.id;
        let old = TransactionCoordinator::new(World::new(vec![old_document], &limits).unwrap(), limits).unwrap();
        let capture = old.capture_retained_document(id).unwrap();
        let new_document = Document::new("new document");
        let mut replacement = TransactionCoordinator::new(World::new(vec![new_document], &limits).unwrap(), limits).unwrap();
        let before = replacement.retained_usage().unwrap().bytes;
        replacement.adopt_external_captures(&old).unwrap();
        drop(old);
        let held = replacement.retained_usage().unwrap().bytes;
        assert!(held > before);
        drop(capture);
        assert!(replacement.retained_usage().unwrap().bytes < held);
    }

    #[test]
    fn rejected_capture_inventory_adoption_leaves_both_coordinators_usable() {
        let limits = Limits::default();
        let old_document = Document::new("old captured");
        let id = old_document.id;
        let old = TransactionCoordinator::new(World::new(vec![old_document], &limits).unwrap(), limits).unwrap();
        let capture = old.capture_retained_document(id).unwrap();
        let mut replacement = TransactionCoordinator::new(World::new(vec![Document::new("new")], &limits).unwrap(), limits).unwrap();
        let before = replacement.retained_usage().unwrap().bytes;
        replacement.limits.max_bytes = before + 1;
        assert!(matches!(replacement.adopt_external_captures(&old), Err(Error::Budget(_))));
        assert!(replacement.external_documents.lock().unwrap().is_empty());
        replacement.limits = limits;
        assert_eq!(replacement.retained_usage().unwrap().bytes, before);
        assert!(old.retained_usage().is_ok());
        assert_eq!(capture.id, id);
    }

    #[test]
    fn external_captures_share_current_owner_and_charge_detached_document_once() {
        let limits = Limits::default();
        let document = Document::new("externally captured source");
        let id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let first = coordinator.capture_retained_document(id).unwrap();
        let current_usage = coordinator.retained_usage().unwrap();
        let second = coordinator.capture_retained_document(id).unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(coordinator.retained_usage().unwrap(), current_usage);
        let old_bytes = first.storage_usage(&limits).unwrap().bytes;
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).name = "new owner".into();
        let with_capture = coordinator.retained_usage().unwrap();
        drop(first);
        assert_eq!(coordinator.retained_usage().unwrap(), with_capture);
        drop(second);
        let after_drop = coordinator.retained_usage().unwrap();
        assert_eq!(with_capture.bytes - after_drop.bytes, old_bytes - std::mem::size_of::<Document>() as u64,
            "dropped payload is released; the external Weak still retains Document's fixed Arc body/header");
        coordinator.limits.max_bytes = with_capture.bytes - 1;
        assert!(coordinator.retained_usage().is_ok());
    }

    #[test]
    fn detached_external_capture_is_included_in_byte_admission() {
        let limits = Limits::default();
        let document = Document::new("capture retained for budget");
        let id = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = coordinator.capture_retained_document(id).unwrap();
        Arc::make_mut(coordinator.world.documents.get_mut(&id).unwrap()).name = "edited".into();
        let held = coordinator.retained_usage().unwrap().bytes;
        coordinator.limits.max_bytes = held - 1;
        assert!(matches!(coordinator.retained_usage(), Err(Error::Budget(_))));
        drop(capture);
        assert!(coordinator.retained_usage().is_ok());
    }

    #[test]
    fn current_owner_proof_rechecks_tight_limits_and_copy_on_write() {
        let limits = Limits::default();
        let doc = Document::new("current owner");
        let id = doc.id;
        let world = World::new(vec![doc], &limits).unwrap();
        let coordinator = TransactionCoordinator::new(world, limits).unwrap();
        let original = coordinator.world.documents.get(&id).unwrap();
        let storage = original.storage_usage(&limits).unwrap();
        assert_eq!(coordinator.validate_retained_document(original, None, storage, &limits).unwrap(), storage);
        assert!(coordinator.current_validation_cache.lock().unwrap().as_ref().unwrap().0.ptr_eq(&Arc::downgrade(original)));
        assert_eq!(coordinator.validate_retained_document(original, None, storage, &limits).unwrap(), storage);
        let tight = Limits { max_name_bytes: 0, ..limits };
        assert!(coordinator.validate_retained_document(original, None, storage, &tight).is_err());
        let mut changed = original.clone();
        Arc::make_mut(&mut changed).name = "bad\0name".into();
        let changed_storage = changed.storage_usage(&limits).unwrap();
        assert!(!Arc::ptr_eq(original, &changed));
        assert!(coordinator.validate_retained_document(&changed, None, changed_storage, &limits).is_err());
    }

    #[test]
    fn cached_proof_requires_same_immutable_owner_and_sufficient_limits() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TransactionCoordinator>();
        let limits = Limits::default();
        let doc = Document::new("receipt");
        let id = doc.id;
        let before = World::new(vec![doc], &limits).unwrap();
        let mut after = before.clone();
        Arc::make_mut(after.documents.get_mut(&id).unwrap()).name = "changed".into();
        let entry = UndoEntry::between(TransactionId::new(), "edit".into(), &before, &after);
        let coordinator = TransactionCoordinator::new(before, limits).unwrap();
        let original = entry.before.get(&id).unwrap();
        let storage = original.storage_usage(&limits).unwrap();
        assert_eq!(coordinator.validate_retained_document(original, Some(&entry), storage, &limits).unwrap(), storage);
        assert!(entry.validation_cache.0.lock().unwrap().iter().flatten().any(|p| p.address == Arc::as_ptr(original) as usize));
        assert_eq!(coordinator.validate_retained_document(original, Some(&entry), storage, &limits).unwrap(), storage);

        // A still-viable but tighter aggregate budget must produce a new proof.
        let prior = entry.validation_cache.0.lock().unwrap()[0].unwrap().limits;
        let tighter = Limits { max_bytes: prior.max_bytes - 1, ..limits };
        assert_eq!(coordinator.validate_retained_document(original, Some(&entry), storage, &tighter).unwrap(), storage);
        assert!(entry.validation_cache.0.lock().unwrap()[0].unwrap().limits.max_bytes <= tighter.max_bytes);

        // The proof cannot bypass a newly tighter per-name limit.
        let tight = Limits { max_name_bytes: 0, ..limits };
        assert!(coordinator.validate_retained_document(original, Some(&entry), storage, &tight).is_err());

        // Arc copy-on-write produces a different owner; a corrupted clone is
        // validated rather than inheriting the original owner's receipt.
        let mut changed = original.clone();
        Arc::make_mut(&mut changed).name = "bad\0name".into();
        let changed_storage = changed.storage_usage(&limits).unwrap();
        assert!(!Arc::ptr_eq(original, &changed));
        assert!(coordinator.validate_retained_document(&changed, Some(&entry), changed_storage, &limits).is_err());
    }
}
