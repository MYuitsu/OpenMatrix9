// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::layer_dialog::{Draft, HiddenChoice, LayerDialog, LayerDialogKind, LayerDialogToken};
use om9_core::{
    budget::Usage,
    layers::{LayerAction, LayerPlan, PresetSlot},
};
impl Controller {
    pub fn layer_dialog(&self) -> Option<&LayerDialog> {
        self.layer_dialog.as_ref()
    }
    pub fn begin_layer_dialog(
        &mut self,
        kind: LayerDialogKind,
        slot: Option<PresetSlot>,
    ) -> Result<LayerDialogToken> {
        if self.layer_dialog.is_some() || self.file.operation.is_some() {
            return Err(Error::Busy);
        }
        if self.is_running() {
            self.validate_session()?;
        }
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.epoch != self.coordinator.epoch())
        {
            return Err(Error::Stale("layer dialog scene"));
        }
        match kind {
            LayerDialogKind::Rename | LayerDialogKind::Color => {
                if !slot
                    .ok_or(Error::Missing("Advanced dialog slot"))?
                    .is_custom()
                {
                    return Err(Error::Permission("only Advanced layer customization"));
                }
            }
            LayerDialogKind::Show if slot.is_some() => {
                return Err(Error::Invalid("Show has no layer target"));
            }
            _ => {}
        }
        let serial = self
            .layer_dialog_serial
            .checked_add(1)
            .ok_or(Error::Overflow("layer dialog serial"))?;
        let (context, _) = self.coordinator.capture_context(None)?;
        let d = self.document();
        let token = LayerDialogToken {
            context,
            owner: (
                d.id,
                d.owner_session,
                d.epoch,
                d.session_generation,
                d.revision,
                d.context_revision,
            ),
            serial,
        };
        let limits = self.coordinator.limits();
        let mut storage = Usage::array::<LayerDialog>(1)?;
        storage.add(
            Usage {
                bytes: 1024,
                ..Usage::default()
            },
            limits,
        )?;
        let count = if kind == LayerDialogKind::Show {
            let count = d.entities.values().filter(|e| e.attributes.hidden).count();
            if count as u64 > limits.max_outputs {
                return Err(Error::Budget("hidden dialog count"));
            }
            storage.add(Usage::array::<HiddenChoice>(count)?, limits)?;
            for e in d.entities.values().filter(|e| e.attributes.hidden) {
                storage.add(
                    Usage {
                        bytes: e.label.len() as u64,
                        ..Usage::default()
                    },
                    limits,
                )?;
            }
            count
        } else {
            if kind == LayerDialogKind::Rename {
                storage.add(
                    Usage {
                        bytes: 512,
                        ..Usage::default()
                    },
                    limits,
                )?;
            }
            0
        };
        let mut held = self.scene_cache_usage()?;
        held.add(storage, limits)?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let draft = match kind {
            LayerDialogKind::Rename => {
                let slot = slot.expect("validated slot");
                let target = self.layer_id(slot);
                let label = target.map_or(slot.name(), |id| d.layers[&id].name.as_str());
                let mut name = String::new();
                name.try_reserve_exact(512)
                    .map_err(|_| Error::Budget("rename draft allocation"))?;
                name.push_str(label);
                Draft::Rename { slot, target, name }
            }
            LayerDialogKind::Color => {
                let slot = slot.expect("validated slot");
                let target = self.layer_id(slot);
                let color = target.map_or(slot.color(), |id| d.layers[&id].color);
                Draft::Color {
                    slot,
                    target,
                    color,
                }
            }
            LayerDialogKind::Show => {
                let mut entries = Vec::new();
                entries
                    .try_reserve_exact(count)
                    .map_err(|_| Error::Budget("hidden dialog allocation"))?;
                for e in d.entities.values().filter(|e| e.attributes.hidden) {
                    let mut label = String::new();
                    label
                        .try_reserve_exact(e.label.len())
                        .map_err(|_| Error::Budget("hidden label allocation"))?;
                    label.push_str(&e.label);
                    entries.push(HiddenChoice {
                        source: d.stamp(e.id)?,
                        label,
                        selected: false,
                    });
                }
                Draft::Show { entries }
            }
        };
        self.layer_dialog = Some(LayerDialog { token, draft });
        self.layer_dialog_serial = serial;
        Ok(token)
    }
    pub fn validate_layer_dialog(&self, token: LayerDialogToken) -> Result<()> {
        if self.layer_dialog.as_ref().is_none_or(|d| d.token != token) {
            return Err(Error::Stale("layer dialog handle"));
        }
        let (context, _) = self.coordinator.capture_context(None)?;
        let d = self.document();
        if context != token.context
            || token.owner
                != (
                    d.id,
                    d.owner_session,
                    d.epoch,
                    d.session_generation,
                    d.revision,
                    d.context_revision,
                )
        {
            return Err(Error::Stale("layer dialog document/session"));
        }
        Ok(())
    }
    pub fn set_layer_dialog_name(&mut self, token: LayerDialogToken, input: &str) -> Result<()> {
        self.validate_layer_dialog(token)?;
        if input.len() > 512 {
            return Err(Error::Budget("rename draft input"));
        }
        match &mut self.layer_dialog.as_mut().expect("validated dialog").draft {
            Draft::Rename { name, .. } => {
                name.clear();
                name.push_str(input);
                Ok(())
            }
            _ => Err(Error::Invalid("dialog is not Rename")),
        }
    }
    pub fn set_layer_dialog_color(&mut self, token: LayerDialogToken, rgb: [u8; 3]) -> Result<()> {
        self.validate_layer_dialog(token)?;
        match &mut self.layer_dialog.as_mut().expect("validated dialog").draft {
            Draft::Color { color, .. } => {
                color[..3].copy_from_slice(&rgb);
                Ok(())
            }
            _ => Err(Error::Invalid("dialog is not Color")),
        }
    }
    pub fn set_layer_dialog_choice(
        &mut self,
        token: LayerDialogToken,
        index: usize,
        selected: bool,
    ) -> Result<()> {
        self.validate_layer_dialog(token)?;
        match &mut self.layer_dialog.as_mut().expect("validated dialog").draft {
            Draft::Show { entries } => {
                entries
                    .get_mut(index)
                    .ok_or(Error::Invalid("hidden dialog row"))?
                    .selected = selected;
                Ok(())
            }
            _ => Err(Error::Invalid("dialog is not Show")),
        }
    }
    pub fn confirm_layer_dialog(&mut self, token: LayerDialogToken) -> Result<()> {
        self.validate_layer_dialog(token)?;
        if self.is_running() {
            self.validate_session()?;
        }
        let dialog = self.layer_dialog.as_ref().expect("validated dialog");
        let no_op = match &dialog.draft {
            Draft::Show { entries } => entries.iter().all(|e| !e.selected),
            Draft::Rename { slot, target, name } => {
                slot.validate_label(name.trim())?;
                target.is_some_and(|id| {
                    self.document()
                        .layers
                        .get(&id)
                        .is_some_and(|layer| layer.name == name.trim())
                }) && self.custom_group_complete(*slot)
            }
            Draft::Color {
                slot,
                target,
                color,
            } => {
                target.is_some_and(|id| {
                    self.document()
                        .layers
                        .get(&id)
                        .is_some_and(|layer| layer.color == *color)
                }) && self.custom_group_complete(*slot)
            }
        };
        if no_op {
            self.layer_dialog = None;
            return Ok(());
        }
        let limits = self.coordinator.limits();
        let mut scratch = Usage::array::<(LayerAction, LayerPlan, Mutation)>(5)?;
        scratch.add(Usage::array::<om9_core::model::Layer>(4)?, limits)?;
        scratch.add(
            Usage {
                bytes: 1024,
                ..Usage::default()
            },
            limits,
        )?;
        if let Draft::Show { entries } = &dialog.draft {
            scratch.add(
                Usage::array::<om9_core::identity::SourceStamp>(
                    entries.iter().filter(|e| e.selected).count(),
                )?,
                limits,
            )?;
        }
        let mut held = self.scene_cache_usage()?;
        held.add(scratch, limits)?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let actions = match &dialog.draft {
            Draft::Rename { slot, target, name } => {
                slot.validate_label(name.trim())?;
                self.customization_actions(*slot, *target, Some(name.trim()), None)?
            }
            Draft::Color {
                slot,
                target,
                color,
            } => self.customization_actions(*slot, *target, None, Some(*color))?,
            Draft::Show { entries } => {
                let count = entries.iter().filter(|e| e.selected).count();
                let mut sources = Vec::new();
                sources
                    .try_reserve_exact(count)
                    .map_err(|_| Error::Budget("Show source allocation"))?;
                for entry in entries.iter().filter(|e| e.selected) {
                    let e = self.document().resolve(&entry.source)?;
                    if !e.attributes.hidden {
                        return Err(Error::Stale("hidden dialog object"));
                    }
                    sources.push(entry.source);
                }
                vec![LayerAction::Hidden {
                    sources,
                    hidden: false,
                }]
            }
        };
        self.apply_layer_actions(actions)?;
        self.layer_dialog = None;
        Ok(())
    }
    fn custom_group_complete(&self, slot: PresetSlot) -> bool {
        crate::layer_command::group(slot).all(|n| {
            self.layer_id(PresetSlot::new(n).expect("bounded group"))
                .is_some()
        })
    }
    fn customization_actions(
        &self,
        slot: PresetSlot,
        target: Option<om9_core::identity::LayerId>,
        name: Option<&str>,
        color: Option<[u8; 4]>,
    ) -> Result<Vec<LayerAction>> {
        if self.layer_id(slot) != target {
            return Err(Error::Stale("custom layer target"));
        }
        let apply = |layer: &mut om9_core::model::Layer| {
            if let Some(name) = name {
                layer.name = name.to_owned();
            }
            if let Some(color) = color {
                layer.color = color;
            }
        };
        let mut actions = Vec::with_capacity(5);
        if let Some(id) = target {
            let mut layer = self.document().layers[&id].clone();
            apply(&mut layer);
            actions.push(LayerAction::Update { layer });
        }
        for mut layer in self.missing_group(slot)? {
            if layer.preset_slot == Some(slot) {
                apply(&mut layer);
            }
            actions.push(LayerAction::Create {
                layer,
                make_current: false,
            });
        }
        Ok(actions)
    }
    pub fn cancel_layer_dialog(&mut self, token: LayerDialogToken) -> Result<()> {
        if self.layer_dialog.as_ref().is_none_or(|d| d.token != token) {
            return Err(Error::Stale("layer dialog handle"));
        }
        self.layer_dialog = None;
        Ok(())
    }
}
