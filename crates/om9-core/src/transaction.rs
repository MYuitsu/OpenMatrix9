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
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

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
            let remaining = validation_limits(l, total, storage)?;
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
            total.add(d.storage_usage(l)?, l)?;
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
        nodes: difference(total.nodes, owner.nodes)?,
        points: difference(total.points, owner.points)?,
        history: difference(total.history, owner.history)?,
        outputs: difference(total.outputs, owner.outputs)?,
    })
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
            | Self::ResourceDelete { document, .. }
            | Self::Import { document, .. }
            | Self::Context { document, .. }
            | Self::Tolerance { document, .. }
            | Self::History { document, .. }
            | Self::HistoryUpdate { document, .. }
            | Self::DefinitionInsert { document, .. }
            | Self::FeatureInsert { document, .. }
            | Self::GroupCreate { document, .. }
            | Self::GroupDelete { document, .. } => vec![*document],
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
                Mutation::GroupCreate { group, .. } => {
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
    authority: SessionId,
    epoch: u64,
    scope_generation: u64,
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    scope: Option<Scope>,
    barrier: bool,
    identities: IdentityLedger,
}
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
        usage.add(world.validate(&limits.remaining_after(held)?)?, &limits)?;
        let initial = IdentityLedger::default();
        usage.add(initial.usage()?, &limits)?;
        let identities = initial.extend(&world, &limits, &mut usage)?;
        Ok(Self {
            world,
            limits,
            authority: SessionId::new(),
            epoch: 1,
            scope_generation: 0,
            undo: vec![],
            redo: vec![],
            scope: None,
            barrier: false,
            identities,
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
    /// alongside this usage before launching work.
    pub fn retained_usage(&self) -> Result<Usage> {
        self.retained_usage_with_limits(&self.limits)
    }
    /// External owners stay live during validation scratch allocation.
    pub fn retained_usage_with_held_usage(&self, held: Usage) -> Result<Usage> {
        let limits = self.limits.remaining_after(held)?;
        let mut usage = held;
        usage.add(self.retained_usage_with_limits(&limits)?, &self.limits)?;
        Ok(usage)
    }
    fn retained_usage_with_limits(&self, limits: &Limits) -> Result<Usage> {
        self.validate_retained_owners(&self.identities, None, false, limits)
    }
    pub fn undo_len(&self) -> usize {
        self.undo.len()
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
            recording,
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
        let mut held = Usage {
            bytes: s.label.capacity() as u64,
            ..Usage::default()
        };
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
        let mut usage = self.retained_usage_with_held_usage(held)?;
        if s.recording {
            let next = self
                .undo
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(2))
                .ok_or(Error::Overflow("Undo allocation"))?
                .max(4);
            usage.add(Usage::array::<UndoEntry>(next)?, &self.limits)?;
        }
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
        self.barrier = true;
        let published = !entry.empty();
        if published {
            self.publish(staged, entry, recording, next);
        }
        self.scope = None;
        self.barrier = false;
        Ok(CommitReceipt {
            transaction: token.id,
            epoch: self.epoch,
            published,
            diagnostics: vec![],
        })
    }
    pub fn open_document(&mut self, mut document: Document) -> Result<()> {
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
        let input = document.storage_usage(&self.limits)?;
        let mut usage = self.retained_usage_with_limits(&self.limits.remaining_after(input)?)?;
        document.renew_session(&self.limits.remaining_after(usage)?)?;
        usage.add(input, &self.limits)?;
        usage.add(Usage::array::<usize>(2)?, &self.limits)?;
        let documents = self
            .world
            .documents
            .len()
            .checked_add(1)
            .ok_or(Error::Overflow("open document map"))?;
        usage.add(
            Usage::btree::<DocumentId, Arc<Document>>(documents)?,
            &self.limits,
        )?;
        let next = bump(self.epoch, "publication epoch")?;
        let mut staged = self.world.clone();
        staged.documents.insert(document.id, Arc::new(document));
        let staged_storage = staged.storage_usage(&self.limits)?;
        staged.validate(&validation_limits(&self.limits, usage, staged_storage)?)?;
        let identities = self.identities.extend(&staged, &self.limits, &mut usage)?;
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
        let mut usage = identities.usage()?;
        if !Arc::ptr_eq(&identities.0, &self.identities.0) {
            usage.add(self.identities.usage()?, limits)?;
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
            usage.add(document.storage_usage(limits)?, limits)?;
            usage.add(Usage::array::<usize>(2)?, limits)?;
        }
        Ok(usage)
    }
    fn validate_retained_owners(
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
        let metadata = self.retained_metadata_usage(identities, entry, recording, limits)?;
        let conservative = (|| -> Result<()> {
            let mut upper = metadata;
            upper.add(index, limits)?;
            // A duplicate-inclusive bound is nonallocating and dominates the
            // exact live owner inventory. Only then may the dedup index grow.
            for document in self.retained_documents(entry) {
                upper.add(document.storage_usage(limits)?, limits)?;
                upper.add(Usage::array::<usize>(2)?, limits)?;
            }
            Ok(())
        })();
        match conservative {
            Ok(()) => {
                let mut shared = BTreeSet::new();
                let mut usage = metadata;
                for document in self.retained_documents(entry) {
                    if shared.insert(Arc::as_ptr(document) as usize) {
                        usage.add(document.storage_usage(limits)?, limits)?;
                        usage.add(Usage::array::<usize>(2)?, limits)?;
                    }
                }
                let indexed_limits = limits.remaining_after(index)?;
                let attempt = (|| -> Result<()> {
                    for document in self.retained_documents(entry) {
                        if shared.remove(&(Arc::as_ptr(document) as usize)) {
                            let storage = document.storage_usage(limits)?;
                            let remaining = validation_limits(&indexed_limits, usage, storage)?;
                            let validated = document.validate(&remaining)?;
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
            let validated = document.validate(&remaining)?;
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
        self.limits
            .remaining_after(held)?
            .remaining_after(prepared.reservation)
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
        let limits = self.limits.remaining_after(held)?;
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
        self.available()?;
        request.cancel.check()?;
        let limits = self.limits.remaining_after(held)?;
        name(&request.label, &limits)?;
        let (base, scope_version) = self.request_world(request.owner.as_ref())?;
        let mut participants = BTreeSet::new();
        let mut writes = BTreeSet::new();
        let mut resource_writes = BTreeSet::new();
        let mut inserted_resources = BTreeSet::new();
        let mut resource_insert_counts = BTreeMap::<DocumentId, usize>::new();
        let mut import_counts = BTreeMap::<DocumentId, (usize, usize, usize)>::new();
        let mut layer_writes = BTreeSet::new();
        let mut created_preset_slots = BTreeSet::new();
        let mut created_layers = BTreeMap::new();
        let mut created_objects = BTreeSet::new();
        let mut current_writes = BTreeSet::new();
        let mut proposed_identities = BTreeSet::new();
        let request_storage = request.storage_usage(&limits)?;
        let mut peak =
            self.retained_usage_with_limits(&limits.remaining_after(request_storage)?)?;
        peak.add(request_storage, &limits)?;
        // Admit temporary ownership maps and the eventual private Undo entry
        // before their first insert or any candidate document clone. Import
        // bundles can propose many identities in a single mutation.
        let mut identities_bound = request.mutations.len();
        let mut writes_bound = request.mutations.len();
        for mutation in &request.mutations {
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
            .map_or(request.recording, |s| s.recording);
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
                    let d = base.document(*document)?;
                    if d.revision != *expected_revision {
                        return Err(Error::Stale("group document revision"));
                    }
                    d.groups.get(group).ok_or(Error::Missing("group"))?;
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
            peak.add(storage, &limits)?;
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
            .map_or(request.recording, |s| s.recording);
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
        let staged_storage = staged.storage_usage(&limits)?;
        staged.validate(&validation_limits(&limits, peak, staged_storage)?)?;
        let identities = self.identities.extend(&staged, &limits, &mut peak)?;
        request.cancel.check()?;
        let id = TransactionId::new();
        let entry = UndoEntry::between(id, request.label, base, &staged);
        self.check_retention(&entry, recording, &identities, &limits)?;
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
    fn publish(&mut self, staged: World, entry: UndoEntry, recording: bool, epoch: u64) {
        self.world = staged;
        self.epoch = epoch;
        self.redo.clear();
        if recording {
            self.undo.push(entry);
        } else {
            self.undo.clear();
        }
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
        if p.owner.is_none() && p.recording {
            let limits = self.limits.remaining_after(p.held)?;
            self.check_retention(&p.entry, p.recording, &p.identities, &limits)?;
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
        self.barrier = true;
        let published = p.owner.is_none();
        self.identities = p.identities;
        if published {
            self.publish(p.staged, p.entry, p.recording, next);
        } else {
            let s = self.scope.as_mut().expect("verified scope");
            s.staged = p.staged;
            s.version = next;
        }
        self.barrier = false;
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
        self.barrier = true;
        let e = self.undo.pop().expect("checked Undo");
        self.world = staged;
        self.epoch = next;
        self.redo.push(e);
        self.barrier = false;
        Ok(())
    }
    pub fn redo(&mut self) -> Result<()> {
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
        self.barrier = true;
        let e = self.redo.pop().expect("checked Redo");
        self.world = staged;
        self.epoch = next;
        self.undo.push(e);
        self.barrier = false;
        Ok(())
    }
}
