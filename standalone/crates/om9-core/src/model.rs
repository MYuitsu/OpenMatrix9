// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Geometry, Transform, Unit},
    identity::*,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
    pub layer: Option<LayerId>,
    pub visible: bool,
    #[serde(default, skip_serializing_if = "not_hidden")]
    pub hidden: bool,
    pub selectable: bool,
    pub locked: bool,
    pub color: Option<[u8; 4]>,
}
fn not_hidden(value: &bool) -> bool {
    !*value
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub id: ObjectId,
    pub label: String,
    pub generation: u64,
    pub geometry_revision: u64,
    pub attributes_revision: u64,
    pub geometry: Geometry,
    pub placement: Transform,
    pub attributes: Attributes,
}
impl Entity {
    pub fn new(
        label: impl Into<String>,
        geometry: Geometry,
        layer: impl Into<Option<LayerId>>,
    ) -> Self {
        let layer = layer.into();
        let color = if layer.is_none() && matches!(geometry, Geometry::Curve(_)) {
            Some([0, 130, 85, 255])
        } else {
            None
        };
        Self {
            id: ObjectId::new(),
            label: label.into(),
            generation: 1,
            geometry_revision: 1,
            attributes_revision: 1,
            geometry,
            placement: Transform::identity(),
            attributes: Attributes {
                layer,
                visible: true,
                hidden: false,
                selectable: true,
                locked: false,
                color,
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: DefinitionId,
    pub label: String,
    pub revision: u64,
    pub members: Vec<Entity>,
}
impl Definition {
    pub fn new(label: impl Into<String>, members: Vec<Entity>) -> Self {
        Self {
            id: DefinitionId::new(),
            label: label.into(),
            revision: 1,
            members,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub parent: Option<LayerId>,
    pub visible: bool,
    pub selectable: bool,
    pub locked: bool,
    pub color: [u8; 4],
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_slot: Option<crate::layers::PresetSlot>,
}
impl Layer {
    pub fn new(name: impl Into<String>, color: [u8; 4]) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            parent: None,
            visible: true,
            selectable: true,
            locked: false,
            color,
            revision: 1,
            preset_slot: None,
        }
    }
    pub fn preset(slot: crate::layers::PresetSlot) -> Self {
        let mut layer = Self::new(slot.name(), slot.color());
        layer.preset_slot = Some(slot);
        layer
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    pub visible: bool,
    pub selectable: bool,
    pub editable: bool,
    pub reference_snappable: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: DocumentId,
    pub name: String,
    pub epoch: u64,
    pub session_generation: u64,
    #[serde(default = "SessionId::legacy_nil")]
    pub owner_session: SessionId,
    pub revision: u64,
    pub context_revision: u64,
    pub page_unit: Unit,
    pub current_layer: Option<LayerId>,
    pub layer_order: Vec<LayerId>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub layers: BTreeMap<LayerId, Layer>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub entities: BTreeMap<ObjectId, Entity>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub definitions: BTreeMap<DefinitionId, Definition>,
    pub history: crate::history::HistoryGraph,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub results: BTreeMap<ObjectId, crate::result::PublishedResult>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub resources: BTreeMap<ResourceId, crate::resource::Resource>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub retained_sources: BTreeMap<SourceId, crate::retained::RetainedSource>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub import_namespaces: BTreeMap<ImportNamespaceId, crate::retained::ImportNamespace>,
}
struct DefinitionPath<'a> {
    id: DefinitionId,
    parent: Option<&'a DefinitionPath<'a>>,
}
impl Document {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: DocumentId::new(),
            name: name.into(),
            epoch: 1,
            session_generation: 1,
            owner_session: SessionId::new(),
            revision: 1,
            context_revision: 1,
            page_unit: Unit::Millimeter,
            current_layer: None,
            layer_order: vec![],
            layers: BTreeMap::new(),
            entities: BTreeMap::new(),
            definitions: BTreeMap::new(),
            history: crate::history::HistoryGraph::default(),
            results: BTreeMap::new(),
            resources: BTreeMap::new(),
            retained_sources: BTreeMap::new(),
            import_namespaces: BTreeMap::new(),
        }
    }
    pub fn stamp(&self, id: ObjectId) -> Result<SourceStamp> {
        let e = self.entities.get(&id).ok_or(Error::Missing("object"))?;
        Ok(SourceStamp {
            document: self.id,
            document_epoch: self.epoch,
            session_generation: self.session_generation,
            owner_session: self.owner_session,
            context_revision: self.context_revision,
            object: e.id,
            generation: e.generation,
            geometry_revision: e.geometry_revision,
            attributes_revision: e.attributes_revision,
        })
    }
    pub fn resolve(&self, stamp: &SourceStamp) -> Result<&Entity> {
        if stamp.document != self.id || &self.stamp(stamp.object)? != stamp {
            return Err(Error::Stale("source stamp"));
        }
        self.entities
            .get(&stamp.object)
            .ok_or(Error::Missing("object"))
    }
    /// Fresh runtime ownership even when two opens start with identical stored
    /// counters. Rebind only complete stamps proved current before renewal;
    /// stale/unresolved references never become valid merely by changing owner.
    pub fn renew_session(&mut self, limits: &Limits) -> Result<()> {
        let next_session = self
            .session_generation
            .checked_add(1)
            .ok_or(Error::Overflow("session generation"))?;
        let next_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(Error::Overflow("document epoch"))?;
        let mut usage = self.validate(limits)?;
        usage.add(Usage::array::<SourceStamp>(self.entities.len())?, limits)?;
        let mut current = Vec::new();
        current
            .try_reserve_exact(self.entities.len())
            .map_err(|_| Error::Budget("session stamp capture"))?;
        for id in self.entities.keys() {
            current.push(self.stamp(*id)?);
        }
        current.sort_unstable();
        let owner = SessionId::new();
        let rebind = |stamp: &mut SourceStamp| {
            if current.binary_search(stamp).is_ok() {
                stamp.session_generation = next_session;
                stamp.document_epoch = next_epoch;
                stamp.owner_session = owner;
            }
        };
        for recipe in self.history.recipes.values_mut() {
            for binding in &mut recipe.inputs {
                rebind(&mut binding.source);
            }
        }
        for result in self.results.values_mut() {
            for stamp in &mut result.source_stamps {
                rebind(stamp);
            }
        }
        self.session_generation = next_session;
        self.epoch = next_epoch;
        self.owner_session = owner;
        Ok(())
    }
    pub(crate) fn layer_permissions(
        &self,
        id: Option<LayerId>,
        snap_locked: bool,
    ) -> Result<Permissions> {
        let mut p = Permissions {
            visible: true,
            selectable: true,
            editable: true,
            reference_snappable: true,
        };
        let mut current = id;
        let mut visited = BTreeSet::new();
        while let Some(id) = current {
            if !visited.insert(id) {
                return Err(Error::Invalid("layer cycle"));
            }
            let l = self.layers.get(&id).ok_or(Error::Missing("layer"))?;
            p.visible &= l.visible;
            p.selectable &= l.selectable && !l.locked;
            p.editable &= !l.locked;
            p.reference_snappable &= !l.locked || snap_locked;
            current = l.parent;
        }
        p.selectable &= p.visible;
        // A layer Off can still receive new output. Entity editing/picking
        // separately masks visibility; lock alone denies layer writes.
        p.reference_snappable &= p.visible;
        Ok(p)
    }
    pub fn permissions(&self, id: ObjectId, snap_locked: bool) -> Result<Permissions> {
        let e = self.entities.get(&id).ok_or(Error::Missing("object"))?;
        self.permissions_for_entity(e, snap_locked)
    }
    pub fn permissions_for_entity(&self, e: &Entity, snap_locked: bool) -> Result<Permissions> {
        let mut p = self.layer_permissions(e.attributes.layer, snap_locked)?;
        p.visible &= e.attributes.visible && !e.attributes.hidden;
        p.selectable &= p.visible && e.attributes.selectable && !e.attributes.locked;
        p.editable &= p.visible && !e.attributes.locked;
        p.reference_snappable &= p.visible && (!e.attributes.locked || snap_locked);
        Ok(p)
    }
    fn validate_entity(
        &self,
        e: &Entity,
        l: &Limits,
        total: &mut Usage,
        ids: &mut BTreeSet<ObjectId>,
    ) -> Result<()> {
        if e.id.is_nil()
            || !ids.insert(e.id)
            || e.generation == 0
            || e.geometry_revision == 0
            || e.attributes_revision == 0
        {
            return Err(Error::Invalid("entity identity/revision"));
        }
        name(&e.label, l)?;
        e.placement.validate()?;
        if e.attributes
            .layer
            .is_some_and(|id| !self.layers.contains_key(&id))
        {
            return Err(Error::Missing("entity layer"));
        }
        total.add(
            Usage {
                bytes: e.label.capacity() as u64,
                objects: 1,
                ..Usage::default()
            },
            l,
        )?;
        total.add(e.geometry.validate(l)?, l)?;
        if let Geometry::Instance(id) = e.geometry
            && !self.definitions.contains_key(&id)
        {
            return Err(Error::Missing("instance definition"));
        }
        Ok(())
    }
    fn visit_definition(
        &self,
        id: DefinitionId,
        l: &Limits,
        depth: usize,
        active: &mut BTreeSet<DefinitionId>,
        expanded: &mut Usage,
    ) -> Result<()> {
        if depth > l.max_depth {
            return Err(Error::Budget("instance depth"));
        }
        if !active.insert(id) {
            return Err(Error::Invalid("definition cycle"));
        }
        let def = self
            .definitions
            .get(&id)
            .ok_or(Error::Missing("definition"))?;
        for e in &def.members {
            expanded.add(
                Usage {
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            if let Geometry::Instance(child) = e.geometry {
                self.visit_definition(child, l, depth + 1, active, expanded)?;
            }
        }
        active.remove(&id);
        Ok(())
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        self.validate_internal(l, true)
    }
    /// Archive preflight only: validate typed metadata and charge prospective
    /// embedded payload storage before hydration. Runtime owners use validate.
    pub fn validate_manifest(&self, l: &Limits) -> Result<Usage> {
        self.validate_internal(l, false)
    }
    /// Nonallocating inventory of immutable owned storage. Cross-document
    /// callers reserve every peer before any identity/graph validation scratch.
    pub(crate) fn storage_usage(&self, l: &Limits) -> Result<Usage> {
        self.storage_usage_internal(l, true)
    }
    fn storage_usage_internal(&self, l: &Limits, payloads: bool) -> Result<Usage> {
        let mut total = Usage::array::<Self>(1)?;
        total.add(
            Usage {
                bytes: self.name.capacity() as u64,
                ..Usage::default()
            },
            l,
        )?;
        for storage in [
            Usage::array::<LayerId>(self.layer_order.capacity())?,
            Usage::btree::<ObjectId, Entity>(self.entities.len())?,
            Usage::btree::<LayerId, Layer>(self.layers.len())?,
            Usage::btree::<DefinitionId, Definition>(self.definitions.len())?,
            Usage::btree::<ObjectId, crate::result::PublishedResult>(self.results.len())?,
            Usage::btree::<ResourceId, crate::resource::Resource>(self.resources.len())?,
        ] {
            total.add(storage, l)?;
        }
        for resource in self.resources.values() {
            total.add(
                if payloads {
                    resource.validate(l)?
                } else {
                    resource.manifest_usage(l)?
                },
                l,
            )?;
        }
        for layer in self.layers.values() {
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Layer>() as u64 + layer.name.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
        }
        for definition in self.definitions.values() {
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Definition>() as u64
                        + definition.label.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<Entity>(definition.members.capacity())?, l)?;
        }
        for entity in self
            .entities
            .values()
            .chain(self.definitions.values().flat_map(|d| &d.members))
        {
            total.add(
                Usage {
                    bytes: entity.label.capacity() as u64,
                    objects: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(entity.geometry.validate(l)?, l)?;
        }
        let mut expanded = Usage::default();
        for id in self.definitions.keys() {
            self.definition_storage_nodes(*id, l, 0, None, &mut expanded)?;
        }
        total.add(expanded, l)?;
        total.add(
            Usage::btree::<HistoryId, crate::history::Recipe>(self.history.recipes.len())?,
            l,
        )?;
        for recipe in self.history.recipes.values() {
            total.add(Self::recipe_storage_usage(recipe, l)?, l)?;
        }
        total.add(self.retained_owned_usage(l)?, l)?;
        for (id, result) in &self.results {
            total.add(result.validate(*id, self, l)?, l)?;
        }
        Ok(total)
    }
    pub(crate) fn recipe_storage_usage(
        recipe: &crate::history::Recipe,
        l: &Limits,
    ) -> Result<Usage> {
        let mut total = Usage::default();
        total.add(
            Usage {
                history: 1,
                ..Usage::array::<crate::history::Recipe>(1)?
            },
            l,
        )?;
        for text in [
            &recipe.command,
            &recipe.environment.solver,
            &recipe.environment.solver_version,
        ] {
            total.add(
                Usage {
                    bytes: text.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        total.add(
            Usage::array::<crate::request::SourceBinding>(recipe.inputs.capacity())?,
            l,
        )?;
        total.add(Usage::array::<ObjectId>(recipe.outputs.capacity())?, l)?;
        for input in &recipe.inputs {
            total.add(input.owned_usage(l)?, l)?;
        }
        total.add(crate::request::options_usage(&recipe.parameters, l)?, l)?;
        if let crate::history::EvaluationState::Dirty(text)
        | crate::history::EvaluationState::Unresolved(text)
        | crate::history::EvaluationState::Failed(text) = &recipe.state
        {
            total.add(
                Usage {
                    bytes: text.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }

        Ok(total)
    }
    fn definition_storage_nodes(
        &self,
        id: DefinitionId,
        l: &Limits,
        depth: usize,
        parent: Option<&DefinitionPath<'_>>,
        expanded: &mut Usage,
    ) -> Result<()> {
        if depth > l.max_depth {
            return Err(Error::Budget("instance depth"));
        }
        let mut ancestor = parent;
        while let Some(path) = ancestor {
            if path.id == id {
                return Err(Error::Invalid("definition cycle"));
            }
            ancestor = path.parent;
        }
        let path = DefinitionPath { id, parent };
        let definition = self
            .definitions
            .get(&id)
            .ok_or(Error::Missing("definition"))?;
        for entity in &definition.members {
            expanded.add(
                Usage {
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            if let Geometry::Instance(child) = entity.geometry {
                self.definition_storage_nodes(child, l, depth + 1, Some(&path), expanded)?;
            }
        }
        Ok(())
    }
    fn validation_scratch_usage(&self, l: &Limits) -> Result<Usage> {
        let objects = self
            .definitions
            .values()
            .try_fold(self.entities.len(), |n, d| {
                n.checked_add(d.members.len())
                    .ok_or(Error::Overflow("validation entities"))
            })?;
        let outputs = self
            .history
            .recipes
            .values()
            .map(|r| r.outputs.len())
            .max()
            .unwrap_or(0);
        let mut scratch = Usage::default();
        for storage in [
            Usage::btree::<LayerId, ()>(self.layer_order.len())?,
            Usage::btree::<ObjectId, ()>(objects)?,
            Usage::btree::<LayerId, ()>(self.layers.len())?,
            Usage::btree::<DefinitionId, ()>(
                self.definitions.len().min(l.max_depth.saturating_add(1)),
            )?,
            Usage::btree::<ObjectId, ()>(outputs)?,
            self.retained_validation_scratch_usage(l)?,
            self.history.execution_scratch_usage()?,
        ] {
            scratch.add(storage, l)?;
        }
        Ok(scratch)
    }
    fn validate_internal(&self, l: &Limits, require_resource_payloads: bool) -> Result<Usage> {
        // No validation set or graph traversal is allocated until owned storage
        // and its entire conservative scratch profile fit together.
        let storage = self.storage_usage_internal(l, require_resource_payloads)?;
        let mut admitted = storage;
        admitted.add(self.validation_scratch_usage(l)?, l)?;
        if self.id.is_nil()
            || [
                self.epoch,
                self.session_generation,
                self.revision,
                self.context_revision,
            ]
            .contains(&0)
        {
            return Err(Error::Invalid("document identity/revision"));
        }
        name(&self.name, l)?;
        let mut total = Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
            ..Usage::default()
        };
        total.add(Usage::array::<LayerId>(self.layer_order.capacity())?, l)?;
        total.add(Usage::btree::<ObjectId, Entity>(self.entities.len())?, l)?;
        total.add(Usage::btree::<LayerId, Layer>(self.layers.len())?, l)?;
        total.add(
            Usage::btree::<DefinitionId, Definition>(self.definitions.len())?,
            l,
        )?;
        total.add(
            Usage::btree::<ObjectId, crate::result::PublishedResult>(self.results.len())?,
            l,
        )?;
        total.add(
            Usage::btree::<ResourceId, crate::resource::Resource>(self.resources.len())?,
            l,
        )?;
        for (id, resource) in &self.resources {
            if *id != resource.id {
                return Err(Error::Invalid("resource map key"));
            }
            total.add(
                if require_resource_payloads {
                    resource.validate(l)?
                } else {
                    resource.manifest_usage(l)?
                },
                l,
            )?;
        }
        let mut order = BTreeSet::new();
        for id in &self.layer_order {
            if !order.insert(id) || !self.layers.contains_key(id) {
                return Err(Error::Invalid("layer order"));
            }
        }
        if order.len() != self.layers.len()
            || self.current_layer.is_some_and(|id| !order.contains(&id))
        {
            return Err(Error::Invalid("current layer/order"));
        }
        let mut preset_slots = [false; 33];
        for (id, layer) in &self.layers {
            if let Some(slot) = layer.preset_slot {
                if !crate::layers::valid_label(slot, &layer.name) {
                    return Err(Error::Invalid("OM9 preset label"));
                }
                if !slot.is_custom() && layer.color != slot.color() {
                    return Err(Error::Invalid("OM9 basic preset palette"));
                }
                // Generic imported layers may share display labels. An OM9
                // Advanced custom label must be unambiguous even after restore
                // or a batch that changes a generic layer's name.
                if slot.is_custom()
                    && layer.name != slot.name()
                    && self.layers.values().any(|other| {
                        other.id != layer.id && crate::layers::same_name(&layer.name, &other.name)
                    })
                {
                    return Err(Error::Invalid("duplicate OM9 custom layer label"));
                }
                let seen = &mut preset_slots[(slot.number() - 1) as usize];
                if *seen {
                    return Err(Error::Invalid("duplicate OM9 preset slot"));
                }
                *seen = true;
            }
            if id.is_nil() || *id != layer.id || layer.revision == 0 {
                return Err(Error::Invalid("layer ID/revision"));
            }
            name(&layer.name, l)?;
            self.layer_permissions(Some(*id), false)?;
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Layer>() as u64 + layer.name.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
        }
        let mut ids = BTreeSet::new();
        for (id, e) in &self.entities {
            if *id != e.id {
                return Err(Error::Invalid("entity key"));
            }
            self.validate_entity(e, l, &mut total, &mut ids)?;
        }
        for (id, def) in &self.definitions {
            if id.is_nil() || *id != def.id || def.revision == 0 {
                return Err(Error::Invalid("definition identity"));
            }
            name(&def.label, l)?;
            total.add(
                Usage {
                    bytes: std::mem::size_of::<Definition>() as u64 + def.label.capacity() as u64,
                    nodes: 1,
                    ..Usage::default()
                },
                l,
            )?;
            total.add(Usage::array::<Entity>(def.members.capacity())?, l)?;
            for e in &def.members {
                self.validate_entity(e, l, &mut total, &mut ids)?;
            }
        }
        let mut expanded = Usage::default();
        // Validate orphan definitions too, since they are persistent masters.
        for id in self.definitions.keys() {
            self.visit_definition(*id, l, 0, &mut BTreeSet::new(), &mut expanded)?;
        }
        total.add(expanded, l)?;
        total.add(self.history.owned_usage(self, l)?, l)?;
        total.add(self.retained_owned_usage(l)?, l)?;
        for (id, result) in &self.results {
            total.add(result.validate(*id, self, l)?, l)?;
        }
        total.check(l)?;
        let mut verification_held = total;
        verification_held.add(Usage::btree::<ObjectId, ()>(ids.len())?, l)?;
        verification_held.add(Usage::btree::<LayerId, ()>(order.len())?, l)?;
        self.validate_retained(l, verification_held)?;
        self.history.execution_order_bounded(l, verification_held)?;
        debug_assert_eq!(total, storage);
        Ok(total)
    }
    pub fn member_transform(&self, root: ObjectId, path: &[ObjectId]) -> Result<Transform> {
        let mut entity = self
            .entities
            .get(&root)
            .ok_or(Error::Missing("instance root"))?;
        let mut matrix = entity.placement;
        let mut visited = BTreeSet::new();
        for id in path {
            if !visited.insert(*id) {
                return Err(Error::Invalid("duplicate occurrence path"));
            }
            let Geometry::Instance(def) = entity.geometry else {
                return Err(Error::Invalid("non-instance path"));
            };
            let definition = self
                .definitions
                .get(&def)
                .ok_or(Error::Missing("definition"))?;
            entity = definition
                .members
                .iter()
                .find(|e| e.id == *id)
                .ok_or(Error::Missing("instance member"))?;
            matrix = matrix.compose(entity.placement)?;
        }
        Ok(matrix)
    }
    pub(crate) fn invalidate_dependents(&mut self, changed: ObjectId, missing: bool) -> Result<()> {
        let affected = self.history.invalidate(changed, missing)?;
        for id in affected {
            if let Some(result) = self.results.get_mut(&id) {
                result.status = crate::result::ResultStatus::Stale;
            }
        }
        Ok(())
    }
    pub(crate) fn invalidate_results(&mut self, changed: ObjectId, missing: bool) -> Result<()> {
        self.invalidate_dependents(changed, missing)?;
        if let Some(result) = self.results.get_mut(&changed) {
            result.status = crate::result::ResultStatus::Stale;
        }
        if let Some(owner) = self.history.owner(changed) {
            let recipe = self
                .history
                .recipes
                .get_mut(&owner)
                .ok_or(Error::Missing("recipe owner"))?;
            recipe.revision = recipe
                .revision
                .checked_add(1)
                .ok_or(Error::Overflow("recipe revision"))?;
            recipe.state = if missing {
                crate::history::EvaluationState::Unresolved(
                    "output was deleted or transferred".into(),
                )
            } else {
                crate::history::EvaluationState::Dirty("output geometry edited".into())
            };
        }
        Ok(())
    }
}
