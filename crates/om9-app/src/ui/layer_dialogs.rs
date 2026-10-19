// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::layer_dialog::LayerDialogKind;
use om9_core::identity::ObjectId;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerDialogControl {
    Name,
    Red,
    Green,
    Blue,
    Ok,
    Cancel,
    Hidden(ObjectId),
}
impl MatrixApp {
    fn clear_layer_dialog_editors(
        &self,
        ctx: &egui::Context,
        token: crate::layer_dialog::LayerDialogToken,
    ) {
        ctx.data_mut(|data| {
            data.remove::<egui::text_edit::TextEditState>(egui::Id::new((
                "owned-layer-name",
                token.authority(),
                token.serial(),
            )));
            for index in 0..3 {
                data.remove::<egui::text_edit::TextEditState>(egui::Id::new((
                    "owned-layer-rgb",
                    token.authority(),
                    token.serial(),
                    index,
                )));
            }
        });
    }
    pub(super) fn layer_dialog_ui(&mut self, ctx: &egui::Context) {
        self.layer_dialog_rects.clear();
        let current = self.controller.layer_dialog().map(|dialog| dialog.token());
        if current != self.layer_dialog_input_token {
            if let Some(old) = self.layer_dialog_input_token {
                self.clear_layer_dialog_editors(ctx, old);
            }
            self.layer_dialog_input_token = current;
            self.layer_dialog_error.clear();
            self.layer_dialog_ime_composing = false;
            if let Some(color) = self
                .controller
                .layer_dialog()
                .and_then(|dialog| dialog.color())
            {
                for (index, input) in self.layer_color_inputs.iter_mut().enumerate() {
                    input.clear();
                    input.push_str(&color[index].to_string());
                }
            }
        }
        let Some(dialog) = self.controller.layer_dialog() else {
            self.layer_dialog_focused = None;
            self.layer_dialog_ime_composing = false;
            return;
        };
        let token = dialog.token();
        let kind = dialog.kind();
        let initial = self.layer_dialog_focused != Some(token);
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                        self.layer_dialog_ime_composing = !text.is_empty()
                    }
                    egui::Event::Ime(egui::ImeEvent::Commit(_))
                    | egui::Event::WindowFocused(false) => self.layer_dialog_ime_composing = false,
                    _ => {}
                }
            }
        });
        let enter = !self.layer_dialog_ime_composing
            && ctx.input(|i| {
                i.key_pressed(Key::Enter)
                    && !i.events.iter().any(|e| matches!(e, egui::Event::Ime(_)))
            });
        self.viewport_menu_owns_frame = true;
        self.focus_cmd = false;
        let title = match kind {
            LayerDialogKind::Rename => "Rename Advanced layer",
            LayerDialogKind::Color => "Advanced layer color",
            LayerDialogKind::Show => "Show hidden objects",
        };
        let mut confirm = false;
        let mut cancel = false;
        let mut painted = false;
        let mut color_valid = true;
        // The token owns mutations, while one reusable area owns UI layout state.
        egui::Modal::new(egui::Id::new("owned-layer-dialog"))
            .frame(egui::Frame::popup(&ctx.style_of(egui::Theme::Light)))
            .show(ctx, |ui| {
                painted = !ui.is_sizing_pass();
                *ui.visuals_mut() = egui::Visuals::light();
                ui.set_width(320.);
                ui.heading(title);
                ui.separator();
                match kind {
                    LayerDialogKind::Rename => {
                        let mut name = String::with_capacity(512);
                        name.push_str(
                            self.controller
                                .layer_dialog()
                                .expect("dialog")
                                .name()
                                .expect("Rename"),
                        );
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut name)
                                .char_limit(128)
                                .id(egui::Id::new((
                                    "owned-layer-name",
                                    token.authority(),
                                    token.serial(),
                                )))
                                .desired_width(300.),
                        );
                        self.layer_dialog_rects
                            .insert(LayerDialogControl::Name, response.rect);
                        if initial && painted {
                            response.request_focus();
                        }
                        if response.changed() {
                            let result = self.controller.set_layer_dialog_name(token, &name);
                            if let Err(error) = result {
                                self.layer_dialog_error = error.to_string();
                            }
                        }
                        ui.label("Up to 128 UTF-8 bytes; label stays unique.");
                    }
                    LayerDialogKind::Color => {
                        let color = self
                            .controller
                            .layer_dialog()
                            .expect("dialog")
                            .color()
                            .expect("Color");
                        let mut rgb = [color[0], color[1], color[2]];
                        let mut changed = false;
                        for (index, (label, control)) in [
                            ("Red", LayerDialogControl::Red),
                            ("Green", LayerDialogControl::Green),
                            ("Blue", LayerDialogControl::Blue),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            ui.horizontal(|ui| {
                                ui.label(label);
                                let id = egui::Id::new((
                                    "owned-layer-rgb",
                                    token.authority(),
                                    token.serial(),
                                    index,
                                ));
                                if ctx.memory(|memory| memory.has_focus(id)) {
                                    let delta = ui.input_mut(|input| {
                                        input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp)
                                            as i64
                                            - input.count_and_consume_key(
                                                Modifiers::NONE,
                                                Key::ArrowDown,
                                            ) as i64
                                    });
                                    if delta != 0 {
                                        rgb[index] =
                                            (i64::from(rgb[index]) + delta).clamp(0, 255) as u8;
                                        self.layer_color_inputs[index].clear();
                                        self.layer_color_inputs[index]
                                            .push_str(&rgb[index].to_string());
                                        changed = true;
                                    }
                                }
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.layer_color_inputs[index])
                                        .id(id)
                                        .char_limit(3)
                                        .desired_width(48.),
                                );
                                self.layer_dialog_rects.insert(control, response.rect);
                                if initial && painted && index == 0 {
                                    response.request_focus();
                                }
                                changed |= response.changed();
                                if let Ok(value) = self.layer_color_inputs[index].parse::<u8>() {
                                    rgb[index] = value;
                                } else {
                                    color_valid = false;
                                }
                            });
                        }
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(300., 24.), Sense::hover());
                        ui.painter().rect_filled(
                            rect,
                            0,
                            Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                        );
                        if !color_valid {
                            ui.colored_label(
                                Color32::DARK_RED,
                                "RGB channels must be integers from 0 to 255.",
                            );
                        }
                        if changed && color_valid {
                            let result = self.controller.set_layer_dialog_color(token, rgb);
                            if let Err(error) = result {
                                self.layer_dialog_error = error.to_string();
                            }
                        }
                    }
                    LayerDialogKind::Show => {
                        let count = self
                            .controller
                            .layer_dialog()
                            .expect("dialog")
                            .entries()
                            .len();
                        ui.label("Choose hidden objects to show:");
                        if count == 0 {
                            ui.label("No objects hidden by OM9 Hide.");
                        }
                        let height = (ctx.content_rect().height() - 180.).clamp(48., 400.);
                        let mut scroll = egui::ScrollArea::vertical()
                            .max_height(height)
                            .id_salt("hidden-layer-choices");
                        if initial {
                            scroll = scroll.vertical_scroll_offset(0.);
                        }
                        scroll.show_rows(ui, 18., count, |ui, range| {
                            for index in range {
                                let entry =
                                    &self.controller.layer_dialog().expect("dialog").entries()
                                        [index];
                                let id = entry.source().object;
                                let mut selected = entry.selected();
                                let response = ui
                                    .push_id((id, token.authority(), token.serial()), |ui| {
                                        let before = selected;
                                        let inner = ui
                                            .horizontal(|ui| {
                                                let checkbox = ui.add(
                                                    egui::Checkbox::without_text(&mut selected),
                                                );
                                                let label = ui.add(
                                                    egui::Label::new(entry.label())
                                                        .truncate()
                                                        .selectable(false)
                                                        .sense(Sense::click())
                                                        .show_tooltip_when_elided(false),
                                                );
                                                checkbox | label
                                            })
                                            .inner;
                                        let row = ui.interact(
                                            inner.rect,
                                            ui.id().with("choice-row"),
                                            Sense::click(),
                                        );
                                        let clicked = inner.clicked() || row.clicked();
                                        let mut response = inner | row;
                                        if clicked {
                                            selected = !before;
                                            response.mark_changed();
                                        }
                                        response
                                    })
                                    .inner;
                                self.layer_dialog_rects
                                    .insert(LayerDialogControl::Hidden(id), response.rect);
                                let changed = response.changed();
                                response.on_hover_text(id.to_string());
                                if changed {
                                    let result = self
                                        .controller
                                        .set_layer_dialog_choice(token, index, selected);
                                    if let Err(error) = result {
                                        self.layer_dialog_error = error.to_string();
                                    }
                                }
                            }
                        });
                    }
                }
                if !self.layer_dialog_error.is_empty() {
                    ui.colored_label(Color32::DARK_RED, &self.layer_dialog_error);
                }
                ui.separator();
                ui.horizontal(|ui| {
                    let ok = ui
                        .push_id(("confirm", token.authority(), token.serial()), |ui| {
                            ui.add_enabled(
                                color_valid && !self.layer_dialog_ime_composing,
                                egui::Button::new("OK"),
                            )
                        })
                        .inner;
                    self.layer_dialog_rects
                        .insert(LayerDialogControl::Ok, ok.rect);
                    if initial && painted && kind == LayerDialogKind::Show {
                        ok.request_focus();
                    }
                    confirm = ok.clicked();
                    let close = ui
                        .push_id(("cancel", token.authority(), token.serial()), |ui| {
                            ui.button("Cancel")
                        })
                        .inner;
                    self.layer_dialog_rects
                        .insert(LayerDialogControl::Cancel, close.rect);
                    cancel = close.clicked();
                    if let Some(audit)=self.audit.as_mut().filter(|audit|audit.layer_dialogs)
                        && audit.layer_clicks.len()<128 && ui.input(|input|input.events.iter().any(|event|matches!(event,egui::Event::PointerButton{..}))) {
                            audit.layer_clicks.push(serde_json::json!({"step":audit.step,"token":token.serial(),
                                "ok_id":format!("{:?}",ok.id),"cancel_id":format!("{:?}",close.id),
                                "ok_hover":ok.hovered(),"cancel_hover":close.hovered(),"ok_clicked":confirm,"cancel_clicked":cancel,
                                "pointer":ui.input(|input|input.pointer.latest_pos().map(|pos|[pos.x,pos.y])),
                                "primary_down":ui.input(|input|input.pointer.primary_down())}));
                            let row=audit.layer_clicks.last_mut().expect("click");
                            row["cancel_contains"]=serde_json::json!(close.contains_pointer());
                            row["cancel_down"]=serde_json::json!(close.is_pointer_button_down_on());
                            row["button_layer"]=serde_json::json!(format!("{:?}",close.layer_id));
                            row["top_layer"]=serde_json::json!(ui.input(|input|input.pointer.latest_pos()).map(|pos|format!("{:?}",ctx.layer_id_at(pos))));
                            row["pressed"]=serde_json::json!(ui.input(|input|input.pointer.primary_pressed()));
                            row["released"]=serde_json::json!(ui.input(|input|input.pointer.primary_released()));
                    }
                });
            });
        if painted {
            self.layer_dialog_focused = Some(token);
        }
        if cancel {
            let result = self.controller.cancel_layer_dialog(token);
            self.feedback(result);
            self.layer_dialog_ime_composing = false;
        } else if painted && color_valid && (confirm || enter) {
            let result = self.controller.confirm_layer_dialog(token);
            if let Err(error) = &result {
                self.layer_dialog_error = error.to_string();
            }
            self.feedback(result);
            if self.controller.layer_dialog().is_some() {
                self.focus_cmd = false;
            }
        }
        if self.controller.layer_dialog().is_none() {
            self.clear_layer_dialog_editors(ctx, token);
            self.layer_dialog_input_token = None;
        }
    }
}
