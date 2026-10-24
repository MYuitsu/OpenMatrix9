// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix9 row gestures route to the same owned controller as Layer CMD.
use super::*;
use crate::layer_command::{LayerChoice, LayerProperty, group};
use crate::layer_dialog::LayerDialogKind;
use egui::{Align2, FontId, PointerButton};
use om9_core::{identity::SourceStamp, layers::PresetSlot};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerControl {
    Name,
    Arrow,
    Swatch,
    Lock,
    Visibility,
    Group,
}
impl MatrixApp {
    fn selected_sources(&self) -> Result<Vec<SourceStamp>> {
        self.layer_sources(self.controller.selected().iter().copied())
    }
    fn layer_sources(
        &self,
        ids: impl Iterator<Item = om9_core::identity::ObjectId> + Clone,
    ) -> Result<Vec<SourceStamp>> {
        self.controller.collect_object_sources(ids)
    }
    fn select_layer_members(&mut self, slot: PresetSlot) -> Result<()> {
        self.controller.select_layer(slot)
    }
    pub(super) fn layers_ui(&mut self, ui: &mut egui::Ui) {
        self.layer_rects.clear();
        self.layer_action_rects.clear();
        ui.horizontal(|ui| {
            let width = (ui.available_width() - 2.) / 2.;
            self.owned_layer_row(ui, 33, width);
            let hide = ui.small_button("Hide");
            self.layer_action_rects.insert("Hide", hide.rect);
            let show = ui.small_button("Show");
            self.layer_action_rects.insert("Show", show.rect);
            if hide.clicked() || hide.secondary_clicked() {
                let sources = if hide.secondary_clicked() {
                    self.controller
                        .scene()
                        .ok_or(Error::Missing("Hide scene"))
                        .and_then(|scene| {
                            self.layer_sources(
                                scene
                                    .objects
                                    .chunk_by(|a, b| a.root == b.root)
                                    .filter(|group| {
                                        group.iter().any(|o| o.visible && o.has_pick_geometry())
                                            && !self.controller.selected().contains(&group[0].root)
                                    })
                                    .map(|group| group[0].root),
                            )
                        })
                } else {
                    self.selected_sources()
                };
                let result =
                    sources.and_then(|sources| self.controller.set_objects_hidden(sources, true));
                if result.is_ok() && hide.clicked() {
                    self.controller.clear_selection();
                }
                self.feedback(result);
            }
            if show.clicked() {
                let sources = self.layer_sources(
                    self.controller
                        .document()
                        .entities
                        .values()
                        .filter(|e| e.attributes.hidden)
                        .map(|e| e.id),
                );
                let result =
                    sources.and_then(|sources| self.controller.set_objects_hidden(sources, false));
                self.feedback(result);
            }
            if show.secondary_clicked() {
                let result = self
                    .controller
                    .begin_layer_dialog(LayerDialogKind::Show, None)
                    .map(|_| ());
                self.feedback(result);
            }
            show.on_hover_text(
                "Left: Show OM9 Hidden objects. Right: choose hidden objects to show.",
            );
        });
        ui.columns(2, |columns| {
            self.layer_column(&mut columns[0], 1);
            self.layer_column(&mut columns[1], 9);
        });
        let advanced = ui.add_sized([ui.available_width(), 14.], egui::Button::new(""));
        self.layer_action_rects.insert("Advanced", advanced.rect);
        let advanced_clicked = advanced.clicked();
        if let Some(audit) = &mut self.audit
            && audit.layer_dialogs
            && audit.step <= 10
        {
            let position = ui.input(|input| input.pointer.interact_pos());
            let top_layer = format!("{:?}", position.and_then(|p| ui.ctx().layer_id_at(p)));
            audit.layer_clicks.push(ui.input(|input|serde_json::json!({"control":"Advanced","step_after_injection":audit.step,"time":input.time,
                "sizing_pass":ui.is_sizing_pass(),"enabled":ui.is_enabled(),"rect":[advanced.rect.min.x,advanced.rect.min.y,advanced.rect.max.x,advanced.rect.max.y],
                "hover":advanced.hovered(),"clicked":advanced_clicked,"down":input.pointer.primary_down(),"pressed":input.pointer.primary_pressed(),"released":input.pointer.primary_released(),
                "position":position.map(|p|[p.x,p.y]),"top_layer":top_layer})));
        }
        let c = advanced.rect.center();
        let sign = if self.advanced_layers { -1. } else { 1. };
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                c + Vec2::new(-4., -2. * sign),
                c + Vec2::new(4., -2. * sign),
                c + Vec2::new(0., 3. * sign),
            ],
            Color32::GRAY,
            Stroke::NONE,
        ));
        if advanced_clicked {
            self.advanced_layers = !self.advanced_layers;
            self.focus_cmd = true;
        }
        advanced.on_hover_text("Advanced layers17–32; collapse keeps document state");
        if self.advanced_layers {
            ui.columns(2, |columns| {
                self.layer_column(&mut columns[0], 17);
                self.layer_column(&mut columns[1], 25);
            });
        }
    }
    fn layer_column(&mut self, ui: &mut egui::Ui, first: u8) {
        let width = (ui.available_width() - 6.).max(90.);
        let right = ui.cursor().left() + width + 6.;
        for block in 0..2 {
            let first = first + block * 4;
            let top = ui.cursor().top();
            for number in first..first + 4 {
                self.owned_layer_row(ui, number, width);
            }
            let bottom = ui.cursor().top() - ui.spacing().item_spacing.y;
            let rect = Rect::from_min_max(egui::pos2(right - 5., top), egui::pos2(right, bottom));
            let response = ui.interact(rect, ui.id().with(("layer-group", first)), Sense::click());
            self.layer_rects.insert((first, LayerControl::Group), rect);
            let slot = PresetSlot::new(first).expect("bounded UI group");
            let visible = group(slot).all(|number| {
                self.controller
                    .layer_id(PresetSlot::new(number).expect("slot"))
                    .is_none_or(|id| self.controller.document().layers[&id].visible)
            });
            ui.painter().rect_filled(
                rect,
                0,
                if visible {
                    Color32::from_rgb(105, 147, 227)
                } else {
                    Color32::GRAY
                },
            );
            if response.clicked() {
                let result = self.controller.toggle_layer_group(slot);
                self.feedback(result);
            }
            response.on_hover_text("Toggle these four layers; mixed state becomes visible");
            ui.add_space(3.);
        }
    }
    fn owned_layer_row(&mut self, ui: &mut egui::Ui, number: u8, width: f32) {
        let slot = PresetSlot::new(number).expect("bounded UI slot");
        let layer = self
            .controller
            .layer_id(slot)
            .map(|id| self.controller.document().layers[&id].clone());
        let name = layer
            .as_ref()
            .map_or_else(|| slot.name().to_owned(), |l| l.name.clone());
        let color = layer.as_ref().map_or(slot.color(), |l| l.color);
        let locked = layer.as_ref().is_some_and(|l| l.locked);
        let visible = layer.as_ref().is_none_or(|l| l.visible);
        let current = layer
            .as_ref()
            .is_some_and(|l| Some(l.id) == self.controller.document().current_layer);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 14.), Sense::hover());
        ui.painter().rect_filled(rect, 0, Color32::from_gray(76));
        // The compact name region has room for one label. Keep the category in
        // the tooltip rather than overprinting the layer's own name.
        let x = rect.right() - 64.;
        let controls = [
            (
                LayerControl::Name,
                Rect::from_min_max(rect.min, egui::pos2(x, rect.bottom())),
            ),
            (
                LayerControl::Arrow,
                Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(10., 14.)),
            ),
            (
                LayerControl::Swatch,
                Rect::from_min_size(egui::pos2(x + 10., rect.top() + 1.), Vec2::new(20., 12.)),
            ),
            (
                LayerControl::Lock,
                Rect::from_min_size(egui::pos2(x + 31., rect.top()), Vec2::new(13., 14.)),
            ),
            (
                LayerControl::Visibility,
                Rect::from_min_size(egui::pos2(x + 44., rect.top() + 1.), Vec2::new(20., 12.)),
            ),
        ];
        for (control, region) in controls {
            self.layer_rects.insert((number, control), region);
            if control == LayerControl::Name && current {
                ui.painter()
                    .rect_filled(region, 0, Color32::from_rgb(90, 130, 210));
            }
            let label = match control {
                LayerControl::Name => name.as_str(),
                LayerControl::Arrow | LayerControl::Lock | LayerControl::Visibility => "",
                _ => "",
            };
            if control == LayerControl::Swatch {
                ui.painter().rect_filled(
                    region,
                    0,
                    Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]),
                );
            }
            if control == LayerControl::Visibility {
                ui.painter()
                    .rect_filled(region, 0, Color32::from_rgb(105, 147, 227));
            }
            let center = region.center();
            match control {
                LayerControl::Arrow => {
                    ui.painter().add(egui::Shape::convex_polygon(
                        vec![
                            center + Vec2::new(-2.5, -4.),
                            center + Vec2::new(3., 0.),
                            center + Vec2::new(-2.5, 4.),
                        ],
                        Color32::WHITE,
                        Stroke::NONE,
                    ));
                }
                LayerControl::Lock => {
                    let color = if locked {
                        Color32::WHITE
                    } else {
                        Color32::GRAY
                    };
                    ui.painter().rect_filled(
                        Rect::from_center_size(center + Vec2::new(0., 2.), Vec2::new(6., 6.)),
                        1,
                        color,
                    );
                    ui.painter().add(egui::Shape::line(
                        vec![
                            center + Vec2::new(-2., 0.),
                            center + Vec2::new(-2., -3.),
                            center + Vec2::new(2., -3.),
                            center + Vec2::new(2., if locked { 0. } else { -1. }),
                        ],
                        Stroke::new(1., color),
                    ));
                }
                LayerControl::Visibility => {
                    let delta = if visible {
                        Vec2::new(0., 4.)
                    } else {
                        Vec2::new(4., 0.)
                    };
                    ui.painter().line_segment(
                        [center - delta, center + delta],
                        Stroke::new(1., Color32::BLACK),
                    );
                }
                _ => {}
            }
            let (pos, anchor) = if control == LayerControl::Name {
                (
                    region.left_center() + Vec2::new(1., 0.),
                    Align2::LEFT_CENTER,
                )
            } else {
                (region.center(), Align2::CENTER_CENTER)
            };
            ui.painter().with_clip_rect(region).text(
                pos,
                anchor,
                label,
                FontId::proportional(10.),
                if current && control == LayerControl::Name {
                    Color32::from_rgb(5, 24, 14)
                } else {
                    chrome::PANEL_INK
                },
            );
            let response = ui.interact(
                region,
                ui.id().with(("layer", number, control)),
                Sense::click(),
            );
            if let Some(audit) = &mut self.audit
                && audit.layers
                && (response.clicked() || response.secondary_clicked())
            {
                audit.layer_clicks.push(serde_json::json!({"step":audit.step,"slot":number,"control":format!("{control:?}"),"primary":response.clicked(),"secondary":response.secondary_clicked()}));
            }
            if let Some(audit) = &mut self.audit
                && audit.layers
                && number == 1
                && control == LayerControl::Swatch
                && (9..=10).contains(&audit.step)
            {
                audit.layer_clicks.push(ui.input(|i|serde_json::json!({"step":audit.step,"hover":response.hovered(),"pressed":i.pointer.secondary_pressed(),"released":i.pointer.secondary_released(),"clicked":i.pointer.secondary_clicked(),"position":i.pointer.interact_pos().map(|p|[p.x,p.y]),"region":[region.min.x,region.min.y,region.max.x,region.max.y]})));
            }
            let action = if response.clicked() {
                Some(match control {
                    LayerControl::Name | LayerControl::Swatch => {
                        self.controller.set_current_layer(LayerChoice::Slot(slot))
                    }
                    LayerControl::Arrow => self
                        .selected_sources()
                        .and_then(|sources| self.controller.assign_layer(slot, sources)),
                    LayerControl::Lock => self
                        .controller
                        .set_layer_property(slot, LayerProperty::Locked(!locked)),
                    LayerControl::Visibility => self
                        .controller
                        .set_layer_property(slot, LayerProperty::Visible(!visible)),
                    _ => Ok(()),
                })
            } else if response.clicked_by(PointerButton::Secondary)
                && (control == LayerControl::Swatch || control == LayerControl::Name)
            {
                if ui.input(|i| i.modifiers.shift) {
                    let kind = if control == LayerControl::Name {
                        LayerDialogKind::Rename
                    } else {
                        LayerDialogKind::Color
                    };
                    Some(
                        self.controller
                            .begin_layer_dialog(kind, Some(slot))
                            .map(|_| ()),
                    )
                } else if control == LayerControl::Swatch {
                    Some(self.select_layer_members(slot))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(result) = action {
                self.feedback(result);
            }
            let help = match control {
                LayerControl::Name => "Left: current layer. Advanced Shift+right: rename",
                LayerControl::Arrow => "Assign selection; keep current layer",
                LayerControl::Swatch => {
                    "Left: current. Right: select visible selectable members. Advanced Shift+right: color"
                }
                LayerControl::Lock => "Lock prevents pick/edit/output; keeps display",
                LayerControl::Visibility => {
                    "Toggle layer visibility; preserve object Hidden/preferences"
                }
                _ => "",
            };
            response.on_hover_text(format!("{name} — {}\n{help}", slot.category_label()));
        }
    }
}
