// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in replay of real Layers widgets and CMD in the owned native window.
use super::*;
use om9_core::layers::PresetSlot;
impl MatrixApp {
    pub(super) fn layer_audit_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let step = self.audit.as_ref().expect("audit").step;
        self.audit.as_mut().expect("audit").step += 1;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let primary = egui::PointerButton::Primary;
        let secondary = egui::PointerButton::Secondary;
        let control = match step {
            4 | 5 => Some((1, LayerControl::Name, primary)),
            8 | 9 => Some((1, LayerControl::Swatch, secondary)),
            10 | 11 => Some((5, LayerControl::Arrow, primary)),
            16..=19 => Some((5, LayerControl::Group, primary)),
            20..=23 => Some((5, LayerControl::Lock, primary)),
            26 | 27 => Some((21, LayerControl::Name, primary)),
            _ => None,
        };
        let action = match step {
            12 | 13 => Some("Hide"),
            14 | 15 => Some("Show"),
            24 | 25 | 28 | 29 | 36 | 37 => Some("Advanced"),
            _ => None,
        };
        let pos = control
            .and_then(|(slot, control, _)| self.layer_control_rect(slot, control))
            .or_else(|| action.and_then(|action| self.layer_action_rect(action)))
            .map(|rect| rect.center());
        if let Some(pos) = pos {
            raw.events.push(egui::Event::PointerMoved(pos));
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: control.map_or(primary, |(_, _, b)| b),
                pressed: step.is_multiple_of(2),
                modifiers: Modifiers::NONE,
            });
        }
        match step {
            0 => raw.events.push(egui::Event::Text("Line".into())),
            2 => raw.events.push(egui::Event::Text("0,0,0".into())),
            6 => raw.events.push(egui::Event::Text("12,6,0".into())),
            30 => raw.events.push(egui::Event::Text("Undo".into())),
            33 => raw.events.push(egui::Event::Text("Redo".into())),
            1 | 3 | 7 | 31 | 34 => raw.events.push(key(Key::Enter)),
            38 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.audit.as_mut().expect("audit").awaiting_screenshot = true;
            }
            _ => {}
        }
        let d = self.controller.document();
        let current_slot = d
            .current_layer
            .and_then(|id| d.layers.get(&id))
            .and_then(|layer| layer.preset_slot)
            .map(|slot| slot.number());
        let entity = d.entities.values().next();
        let assigned_slot = entity
            .and_then(|e| e.attributes.layer)
            .and_then(|id| d.layers.get(&id))
            .and_then(|layer| layer.preset_slot)
            .map(|slot| slot.number());
        let group_visible = (5..=8).all(|n| {
            self.controller
                .layer_id(PresetSlot::new(n).expect("slot"))
                .is_some_and(|id| d.layers[&id].visible)
        });
        let check = match step {
            6 if self.controller.is_running()
                && d.entities.is_empty()
                && current_slot == Some(1)
                && d.layers.len() == 4 =>
            {
                Some("native layer current preserves pending Line")
            }
            8 if d.entities.len() == 1
                && assigned_slot == Some(1)
                && self.controller.coordinator().undo_len() == 2 =>
            {
                Some("native Line binds current at acceptance")
            }
            10 if self.controller.selected().len() == 1
                && entity.is_some_and(|e| self.controller.selected().contains(&e.id)) =>
            {
                Some("native swatch selects eligible membership")
            }
            12 if assigned_slot == Some(5)
                && current_slot == Some(1)
                && d.layers.len() == 8
                && self.controller.coordinator().undo_len() == 3 =>
            {
                Some("native lazy arrow assigns without current change")
            }
            14 if entity.is_some_and(|e| {
                e.attributes.hidden && e.attributes.visible && e.attributes.selectable
            }) && self.controller.selected().is_empty() =>
            {
                Some("native Hide owns Hidden only")
            }
            16 if entity.is_some_and(|e| {
                !e.attributes.hidden && e.attributes.visible && e.attributes.selectable
            }) =>
            {
                Some("native Show restores independent Hidden")
            }
            18 if !group_visible
                && entity.is_some_and(|e| {
                    e.attributes.visible && e.attributes.selectable && !e.attributes.hidden
                }) =>
            {
                Some("native group Off preserves explicit preferences")
            }
            22 if group_visible
                && self.controller.scene().is_some_and(|scene| {
                    scene.objects.iter().any(|o| o.visible && !o.selectable)
                }) =>
            {
                Some("native Lock masks pick keeps display")
            }
            24 if self
                .controller
                .scene()
                .is_some_and(|scene| scene.objects.iter().any(|o| o.visible && o.selectable)) =>
            {
                Some("native Unlock restores explicit selectability")
            }
            28 if current_slot == Some(21)
                && d.current_layer
                    .is_some_and(|id| d.layers[&id].name == "User 25") =>
            {
                Some("native Advanced stable label current")
            }
            30 if !self.advanced_layers && current_slot == Some(21) => {
                Some("native collapse preserves state")
            }
            32 if current_slot == Some(1)
                && d.layers.len() == 8
                && self.controller.coordinator().redo_len() == 1 =>
            {
                Some("native CMD Undo restores layer state")
            }
            35 if d.current_layer == self.audit.as_ref().expect("audit").layer_current
                && current_slot == Some(21)
                && d.layers.len() == 12
                && self.controller.coordinator().undo_len() == 10 =>
            {
                Some("native CMD Redo restores stable preset IDs")
            }
            _ => None,
        };
        if step == 28 {
            self.audit.as_mut().expect("audit").layer_current = d.current_layer;
        }
        if let Some(check) = check {
            self.audit
                .as_mut()
                .expect("audit")
                .checks
                .push(check.into());
        }
        ctx.request_repaint();
    }
}
