// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::layer_command::{LayerChoice, LayerProperty, group};
use om9_core::{
    identity::{LayerId, ObjectId, SourceStamp},
    layers::{LayerAction, LayerPlan, PresetSlot},
    model::Layer,
};
impl Controller {
    pub fn layer_id(&self, slot: PresetSlot) -> Option<LayerId> {
        self.document()
            .layers
            .values()
            .find(|l| l.preset_slot == Some(slot))
            .map(|l| l.id)
    }
    pub(super) fn missing_group(&self, slot: PresetSlot) -> Result<Vec<Layer>> {
        let mut missing = Vec::with_capacity(4);
        for number in group(slot) {
            let slot = PresetSlot::new(number)?;
            if self.layer_id(slot).is_none() {
                missing.push(Layer::preset(slot));
            }
        }
        Ok(missing)
    }
    pub(super) fn apply_layer_actions(&mut self, actions: Vec<LayerAction>) -> Result<()> {
        if self.is_running() {
            self.validate_session()?;
        }
        let mutations = actions
            .into_iter()
            .map(|action| {
                LayerPlan::capture(&self.coordinator, self.document, action, None)
                    .map(|p| Mutation::Layers(Box::new(p)))
            })
            .collect::<Result<Vec<_>>>()?;
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("layer scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("layer scene"));
        }
        let old_usage = self.scene_cache_usage()?;
        let mut prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new("Layers", mutations), old_usage)?;
        let same = Arc::ptr_eq(
            &prepared.candidate_world().documents[&self.document],
            &self.coordinator.world().documents[&self.document],
        );
        let scene = if same {
            None
        } else {
            let mut held = old_usage;
            held.add(old_scene.update_usage(None)?, self.coordinator.limits())?;
            self.coordinator
                .reserve_prepared_held_usage(&mut prepared, held)?;
            Some(
                old_scene.update_owned(
                    prepared.candidate_world().documents[&self.document].clone(),
                    self.coordinator
                        .epoch()
                        .checked_add(1)
                        .ok_or(Error::Overflow("layer scene epoch"))?,
                    None,
                    self.coordinator.limits(),
                )?,
            )
        };
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.scene = scene;
            let epoch = self.coordinator.epoch();
            let revision = self.document().context_revision;
            if let Some(s) = &mut self.session {
                // A layer change inside the command's own transaction keeps the
                // snapshot valid: the document, the view and the revision are the
                // ones the command started against (OM9-L3-01-02-003).
                s.context.epoch = epoch;
                s.context.revision = revision;
            }
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }
    pub fn set_current_layer(&mut self, choice: LayerChoice) -> Result<()> {
        let mut actions = vec![];
        let id = match choice {
            LayerChoice::None => None,
            LayerChoice::Existing(id) => Some(id),
            LayerChoice::Slot(slot) => {
                let missing = self.missing_group(slot)?;
                let id = self
                    .layer_id(slot)
                    .or_else(|| {
                        missing
                            .iter()
                            .find(|l| l.preset_slot == Some(slot))
                            .map(|l| l.id)
                    })
                    .ok_or(Error::Missing("preset current"))?;
                let created = missing.iter().any(|l| l.id == id);
                actions.extend(missing.into_iter().map(|layer| {
                    let make_current = layer.id == id;
                    LayerAction::Create {
                        layer,
                        make_current,
                    }
                }));
                if created {
                    return self.apply_layer_actions(actions);
                }
                Some(id)
            }
        };
        actions.push(LayerAction::SetCurrent { layer: id });
        self.apply_layer_actions(actions)
    }
    pub fn set_layer_property(&mut self, slot: PresetSlot, property: LayerProperty) -> Result<()> {
        let missing = self.missing_group(slot)?;
        let mut actions = vec![];
        let apply = |layer: &mut Layer| match property {
            LayerProperty::Visible(value) => layer.visible = value,
            LayerProperty::Locked(value) => layer.locked = value,
        };
        if let Some(id) = self.layer_id(slot) {
            let mut layer = self.document().layers[&id].clone();
            apply(&mut layer);
            actions.push(LayerAction::Update { layer });
        }
        actions.extend(missing.into_iter().map(|mut layer| {
            if layer.preset_slot == Some(slot) {
                apply(&mut layer);
            }
            LayerAction::Create {
                layer,
                make_current: false,
            }
        }));
        self.apply_layer_actions(actions)
    }
    pub fn toggle_layer_group(&mut self, slot: PresetSlot) -> Result<()> {
        let visible = !group(slot).all(|n| {
            self.layer_id(PresetSlot::new(n).expect("bounded group"))
                .is_none_or(|id| self.document().layers[&id].visible)
        });
        let mut actions = vec![];
        for n in group(slot) {
            let slot = PresetSlot::new(n)?;
            if let Some(id) = self.layer_id(slot) {
                let mut layer = self.document().layers[&id].clone();
                layer.visible = visible;
                actions.push(LayerAction::Update { layer });
            } else {
                let mut layer = Layer::preset(slot);
                layer.visible = visible;
                actions.push(LayerAction::Create {
                    layer,
                    make_current: false,
                });
            }
        }
        self.apply_layer_actions(actions)
    }
    pub fn assign_layer(&mut self, slot: PresetSlot, sources: Vec<SourceStamp>) -> Result<()> {
        if sources.is_empty() {
            return Ok(());
        }
        let missing = self.missing_group(slot)?;
        let id = self
            .layer_id(slot)
            .or_else(|| {
                missing
                    .iter()
                    .find(|l| l.preset_slot == Some(slot))
                    .map(|l| l.id)
            })
            .ok_or(Error::Missing("preset assignment"))?;
        let mut actions: Vec<_> = missing
            .into_iter()
            .map(|layer| LayerAction::Create {
                layer,
                make_current: false,
            })
            .collect();
        // The SOURCE side of the move is protected too: an object living on a
        // locked layer must not be moved off it, or the lock would only guard the
        // target (OM9-L3-03-14-003, oracle am: no mutation, informative status).
        for source in &sources {
            let entity = self
                .document()
                .entities
                .get(&source.object)
                .ok_or(Error::Missing("assigned source"))?;
            let locked_layer = entity
                .attributes
                .layer
                .and_then(|id| self.document().layers.get(&id))
                .is_some_and(|layer| layer.locked);
            if entity.attributes.locked || locked_layer {
                return Err(Error::Permission("object is on a locked layer"));
            }
        }
        // A locked layer is locked against exactly this: moving geometry onto it.
        // Refuse before anything is applied, so the refusal costs no transaction
        // and the user reads why (OM9-L3-03-14-002, oracle am).
        if self
            .document()
            .layers
            .get(&id)
            .is_some_and(|layer| layer.locked)
        {
            return Err(Error::Permission("layer is locked"));
        }
        actions.push(LayerAction::Assign { layer: id, sources });
        self.apply_layer_actions(actions)
    }
    pub fn set_objects_hidden(&mut self, sources: Vec<SourceStamp>, hidden: bool) -> Result<()> {
        self.apply_layer_actions(vec![LayerAction::Hidden { sources, hidden }])
    }
    pub fn scene_cache_usage(&self) -> Result<om9_core::budget::Usage> {
        let mut usage = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("scene"))?
            .cache_usage()?;
        usage.add(self.auxiliary_usage()?, self.coordinator.limits())?;
        Ok(usage)
    }
    pub(super) fn auxiliary_usage(&self) -> Result<om9_core::budget::Usage> {
        let mut usage = om9_core::budget::Usage::btree::<ObjectId, ()>(self.selection.len())?;
        usage.add(
            om9_core::budget::Usage::array::<ObjectId>(self.selection_order.capacity())?,
            self.coordinator.limits(),
        )?;
        if let Some(pending) = &self.solid_pending {
            usage.add(pending.usage()?, self.coordinator.limits())?;
        }
        usage.add(
            om9_core::budget::Usage::array::<super::SelectedSubobject>(
                self.subselection.capacity(),
            )?,
            self.coordinator.limits(),
        )?;
        for picked in &self.subselection {
            usage.add(
                om9_core::budget::Usage::array::<ObjectId>(picked.path.capacity())?,
                self.coordinator.limits(),
            )?;
        }
        usage.add(self.workspace.storage_usage()?, self.coordinator.limits())?;
        usage.add(
            om9_core::budget::Usage::array::<String>(self.recent.capacity())?,
            self.coordinator.limits(),
        )?;
        for command in &self.recent {
            usage.add(
                om9_core::budget::Usage {
                    bytes: command.capacity() as u64,
                    ..Default::default()
                },
                self.coordinator.limits(),
            )?;
        }
        if let Some(command) = &self.repeat {
            usage.add(
                om9_core::budget::Usage {
                    bytes: command.capacity() as u64,
                    ..Default::default()
                },
                self.coordinator.limits(),
            )?;
        }
        usage.add(
            self.file.usage(self.coordinator.limits())?,
            self.coordinator.limits(),
        )?;
        if let Some(dialog) = &self.layer_dialog {
            usage.add(
                dialog.usage(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
        }
        usage.add(self.contract_usage()?, self.coordinator.limits())?;
        Ok(usage)
    }
    pub fn selected(&self) -> &std::collections::BTreeSet<ObjectId> {
        &self.selection
    }
    /// Selection in pick order, then any remaining ids in stable set order.
    pub fn selected_ordered(&self) -> Vec<ObjectId> {
        let mut ordered: Vec<ObjectId> = self
            .selection_order
            .iter()
            .filter(|id| self.selection.contains(id))
            .copied()
            .collect();
        for id in &self.selection {
            if !ordered.contains(id) {
                ordered.push(*id);
            }
        }
        ordered
    }
    pub fn clear_selection(&mut self) {
        self.clear_subselection();
        self.clear_gumball_preview();
        if !self.selection.is_empty() {
            self.selection_revision = self.selection_revision.saturating_add(1);
        }
        self.selection.clear();
        self.selection_order.clear();
    }
    /// Adds objects to the selection without dropping the ones already there (Shift + window).
    pub fn add_objects(&mut self, ids: &[ObjectId]) -> Result<()> {
        self.extend_selection(ids, true)
    }

    /// Removes objects from the selection (Ctrl + window in Rhino).
    pub fn remove_objects(&mut self, ids: &[ObjectId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            self.selection.remove(id);
        }
        self.selection_order.retain(|id| self.selection.contains(id));
        self.selection_revision = self.selection_revision.saturating_add(1);
        Ok(())
    }

    /// Shared body for "add to selection": every id must be a selectable root, then all are inserted.
    fn extend_selection(&mut self, ids: &[ObjectId], add: bool) -> Result<()> {
        if ids.is_empty() && !add {
            return Ok(());
        }
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("selection scene"))?;
        if scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("selection scene"));
        }
        for id in ids {
            if !scene
                .objects
                .iter()
                .any(|object| object.root == *id && object.selectable && object.has_pick_geometry())
            {
                return Err(Error::Permission("selected root"));
            }
        }
        let mut held = self.scene_cache_usage()?;
        let added = ids
            .iter()
            .filter(|id| !self.selection.contains(*id))
            .count();
        held.add(
            om9_core::budget::Usage::btree::<ObjectId, ()>(self.selection.len() + added)?,
            self.coordinator.limits(),
        )?;
        held.add(om9_core::budget::Usage::array::<ObjectId>(added)?, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        self.selection_order.try_reserve_exact(added).map_err(|_| Error::Budget("selection order"))?;
        for id in ids {
            if self.selection.insert(*id) {
                self.selection_order.push(*id);
            }
        }
        self.selection_revision = self.selection_revision.saturating_add(1);
        Ok(())
    }

    /// Adds or removes one object, which is what Ctrl+click does in Matrix9.
    pub fn toggle_selection(&mut self, id: ObjectId) -> Result<()> {
        if self.selection.contains(&id) {
            self.selection.remove(&id);
            self.selection_order.retain(|selected| *selected != id);
            self.selection_revision = self.selection_revision.saturating_add(1);
            return Ok(());
        }
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("selection scene"))?;
        if scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("selection scene"));
        }
        if !scene
            .objects
            .iter()
            .any(|object| object.root == id && object.selectable && object.has_pick_geometry())
        {
            return Err(Error::Permission("selected root"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::btree::<ObjectId, ()>(self.selection.len() + 1)?,
            self.coordinator.limits(),
        )?;
        held.add(om9_core::budget::Usage::array::<ObjectId>(1)?, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        self.selection_order.try_reserve_exact(1).map_err(|_| Error::Budget("selection order"))?;
        self.selection.insert(id);
        self.selection_order.push(id);
        self.selection_revision = self.selection_revision.saturating_add(1);
        Ok(())
    }

    /// Selects exactly `ids`, replacing the current selection in one step, as a rubber band does.
    ///
    /// Every id must be a selectable root of the current scene; one bad id refuses the whole band, so a
    /// stale pick can never leave a half-updated selection behind.
    pub fn select_objects(&mut self, ids: &[ObjectId]) -> Result<()> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("selection scene"))?;
        if scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("selection scene"));
        }
        if ids.len() as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("root selection count"));
        }
        let mut next = std::collections::BTreeSet::new();
        for id in ids {
            if id.is_nil()
                || !scene.objects.iter().any(|object| {
                    object.root == *id && object.selectable && object.has_pick_geometry()
                })
            {
                return Err(Error::Permission("selected root"));
            }
            next.insert(*id);
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::btree::<ObjectId, ()>(next.len())?,
            self.coordinator.limits(),
        )?;
        held.add(om9_core::budget::Usage::array::<ObjectId>(ids.len())?, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut order = Vec::new();
        order.try_reserve_exact(ids.len()).map_err(|_| Error::Budget("selection order"))?;
        for id in ids {
            if !order.contains(id) {
                order.push(*id);
            }
        }
        if next != self.selection || order != self.selection_order {
            self.selection_revision = self.selection_revision.saturating_add(1);
        }
        self.selection_order = order;
        self.selection = next;
        Ok(())
    }

    pub fn select_root(&mut self, root: Option<ObjectId>, extend: bool) -> Result<()> {
        if let Some(id) = root
            && self.scene.as_ref().is_none_or(|scene| {
                scene.epoch != self.coordinator.epoch()
                    || !scene
                        .objects
                        .iter()
                        .any(|o| o.root == id && o.selectable && o.has_pick_geometry())
            })
        {
            return Err(Error::Permission("selected root"));
        }
        let count = if extend {
            self.selection
                .len()
                .checked_add(usize::from(
                    root.is_some_and(|id| !self.selection.contains(&id)),
                ))
                .ok_or(Error::Overflow("root selection count"))?
        } else {
            usize::from(root.is_some())
        };
        if count as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("root selection count"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::btree::<ObjectId, ()>(count)?,
            self.coordinator.limits(),
        )?;
        held.add(om9_core::budget::Usage::array::<ObjectId>(usize::from(root.is_some()))?, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        self.selection_order.try_reserve_exact(usize::from(root.is_some())).map_err(|_| Error::Budget("selection order"))?;
        if extend {
            // Shift means ADD (Rhino p33: "Hold the Shift key while clicking the objects"), not toggle:
            // clicking an already selected object again must leave it selected. Ctrl is the toggle gesture
            // and lives in `toggle_selection`.
            if let Some(id) = root {
                if self.selection.insert(id) {
                    self.selection_order.push(id);
                }
            }
        } else {
            let mut selection = std::collections::BTreeSet::new();
            if let Some(id) = root {
                selection.insert(id);
            }
            self.selection = selection;
            self.selection_order.clear();
            self.selection_order.extend(root);
        }
        self.selection_revision = self.selection_revision.saturating_add(1);
        Ok(())
    }
    pub fn select_layer(&mut self, slot: PresetSlot) -> Result<()> {
        let count = self.selectable_layer_member_count(slot)?;
        let limits = self.coordinator.limits();
        if count as u64 > limits.max_outputs {
            return Err(Error::Budget("layer selection count"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(om9_core::budget::Usage::array::<ObjectId>(count)?, limits)?;
        held.add(
            om9_core::budget::Usage::btree::<ObjectId, ()>(count)?,
            limits,
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let ids = self.selectable_layer_members(slot)?;
        self.selection = ids.iter().copied().collect();
        self.selection_order = ids;
        self.selection_revision = self.selection_revision.saturating_add(1);
        Ok(())
    }
    pub fn selectable_layer_member_count(&self, slot: PresetSlot) -> Result<usize> {
        let Some(id) = self.layer_id(slot) else {
            return Ok(0);
        };
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("layer selection scene"))?;
        if scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("layer selection scene"));
        }
        Ok(scene
            .objects
            .chunk_by(|a, b| a.root == b.root)
            .filter(|group| {
                group.iter().any(|o| o.selectable && o.has_pick_geometry())
                    && self
                        .document()
                        .entities
                        .get(&group[0].root)
                        .is_some_and(|e| e.attributes.layer == Some(id))
            })
            .count())
    }
    pub fn selectable_layer_members(&self, slot: PresetSlot) -> Result<Vec<ObjectId>> {
        let count = self.selectable_layer_member_count(slot)?;
        if count as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("layer selection count"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<ObjectId>(count)?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("layer selection allocation"))?;
        if let Some(id) = self.layer_id(slot) {
            let scene = self.scene.as_ref().ok_or(Error::Missing("layer scene"))?;
            for group in scene.objects.chunk_by(|a, b| a.root == b.root) {
                if group.iter().any(|o| o.selectable && o.has_pick_geometry())
                    && self
                        .document()
                        .entities
                        .get(&group[0].root)
                        .is_some_and(|e| e.attributes.layer == Some(id))
                {
                    result.push(group[0].root);
                }
            }
        }
        Ok(result)
    }
    pub fn collect_object_sources(
        &self,
        ids: impl Iterator<Item = ObjectId> + Clone,
    ) -> Result<Vec<SourceStamp>> {
        let count = ids.clone().count();
        if count as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("layer source selection"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<SourceStamp>(count)?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("layer source allocation"))?;
        for id in ids {
            if sources.len() >= count {
                return Err(Error::Invalid("layer source iterator changed"));
            }
            sources.push(self.document().stamp(id)?);
        }
        if sources.len() != count {
            return Err(Error::Invalid("layer source iterator changed"));
        }
        Ok(sources)
    }
}
