// SPDX-License-Identifier: LGPL-2.1-or-later
//! The small options window of an interactive add-on command (Rhino 5 / Matrix 9 "Sweep 2
//! Rails Options" …), drawn from [`crate::session::AddonDialog`].
//!
//! Checkboxes, radio buttons and number fields edit the same values as the command line; a
//! number option named after a choice (`RebuildPoints` after `Rebuild`) sits beside that
//! choice, as in Rhino's Simplify group. OK = Enter = right click, Cancel = Esc.
use super::MatrixApp;
use crate::session::{AddonDialog, DialogKind, DialogRow};
use eframe::egui::{self, Align2, RichText};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct SessionDialogUi {
    /// The command whose window the user closed with ✕ (shown again on the next command).
    pub hidden: Option<String>,
    /// The command last shown, to reset the fields when another one starts.
    pub seen: Option<String>,
    /// Number fields being typed: name → (value they started from, text).
    pub drafts: BTreeMap<String, (String, String)>,
    /// Where each control was drawn last frame (`Dialog:OK`, `Dialog:MaintainHeight` …), for
    /// input replay.
    pub rects: BTreeMap<String, egui::Rect>,
}

/// The words after a number that sits beside its choice (Rhino: "Rebuild with [10] control
/// points", "Refit within [0.01] mm").
fn suffix(name: &str, choice: &str, vi: bool) -> String {
    let rest = &name[choice.len().min(name.len())..];
    match (rest.to_ascii_lowercase().as_str(), vi) {
        ("points", true) => "điểm điều khiển".into(),
        ("points", false) => "control points".into(),
        ("tolerance", _) => "mm".into(),
        _ => crate::session::humanize(rest),
    }
}

/// What the user did in the window this frame.
enum Act {
    Set(String, String),
    Ok,
    Cancel,
}

impl MatrixApp {
    pub(super) fn session_dialog_ui(&mut self, ctx: &egui::Context) {
        self.session_dialog.rects.clear();
        let Some(dialog) = self.controller.addon_dialog() else {
            if self.controller.addon_session().is_none() {
                self.session_dialog.hidden = None;
                self.session_dialog.seen = None;
            }
            self.session_dialog.drafts.clear();
            return;
        };
        if self.session_dialog.seen.as_deref() != Some(dialog.code.as_str()) {
            self.session_dialog.seen = Some(dialog.code.clone());
            self.session_dialog.hidden = None;
            self.session_dialog.drafts.clear();
        }
        if self.session_dialog.hidden.as_deref() == Some(dialog.code.as_str()) {
            return;
        }
        let vi = self.controller.tr().lang() == om9_addons::i18n::Lang::Vi;
        let mut open = true;
        let mut acts: Vec<Act> = Vec::new();
        let area = ctx.content_rect();
        // Opaque and light like the application's other windows (Rhino's dialogs are light
        // too): the viewport must not show through the fields.
        let style = ctx.style_of(egui::Theme::Light);
        let frame = egui::Frame::window(&style)
            .fill(egui::Color32::from_rgb(244, 244, 240))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 125, 50)));
        let window = egui::Window::new(RichText::new(&dialog.title).strong())
            .frame(frame)
            .id(egui::Id::new(("om9-addon-dialog", &dialog.code)))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .pivot(Align2::RIGHT_TOP)
            .default_pos(egui::pos2(area.right() - 300.0, area.top() + 130.0))
            .min_width(240.0);
        let response = window.show(ctx, |ui| {
            self.session_dialog_body(ui, &dialog, vi, &mut acts);
        });
        if let Some(r) = response {
            self.session_dialog.rects
                .insert("OptionsDialog".into(), r.response.rect);
        }
        if !open {
            self.session_dialog.hidden = Some(dialog.code.clone());
        }
        for act in acts {
            let result = match act {
                Act::Set(name, value) => {
                    self.session_dialog.drafts.remove(&name);
                    self.controller.set_addon_dialog_value(&name, &value)
                }
                Act::Ok => self.controller.submit_effect("").map(|_| ()),
                Act::Cancel => {
                    self.controller.cancel();
                    self.transcript
                        .append_history(&format!("{}: {}", dialog.title, if vi { "đã huỷ" } else { "cancelled" }));
                    Ok(())
                }
            };
            self.feedback(result);
            self.focus_cmd = true;
        }
    }

    fn session_dialog_body(
        &mut self,
        ui: &mut egui::Ui,
        dialog: &AddonDialog,
        vi: bool,
        acts: &mut Vec<Act>,
    ) {
        // Number options shown beside the choice they belong to (`RebuildPoints` → `Rebuild`).
        let mut beside: BTreeMap<String, (String, String)> = BTreeMap::new();
        for row in &dialog.rows {
            if let DialogKind::Choice(choices) = &row.kind {
                for (value, _) in choices {
                    if let Some(n) = dialog.rows.iter().find(|r| {
                        r.kind == DialogKind::Number
                            && r.name.len() > value.len()
                            && r.name[..value.len()].eq_ignore_ascii_case(value)
                    }) {
                        beside.insert(n.name.clone(), (row.name.clone(), value.clone()));
                    }
                }
            }
        }
        let number_row = |name: &str| dialog.rows.iter().find(|r| r.name == name);
        // Checkboxes first, as in Rhino's dialogs.
        for row in dialog.rows.iter().filter(|r| r.kind == DialogKind::Toggle) {
            let mut on = row.value.eq_ignore_ascii_case("yes");
            let check = ui.checkbox(&mut on, &row.label);
            self.session_dialog.rects.insert(format!("Dialog:{}", row.name), check.rect);
            if check.changed() {
                acts.push(Act::Set(row.name.clone(), if on { "Yes" } else { "No" }.into()));
            }
        }
        for row in &dialog.rows {
            let DialogKind::Choice(choices) = &row.kind else { continue };
            ui.add_space(4.0);
            if choices.len() <= 4 {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_min_width(220.0);
                    ui.label(RichText::new(&row.label).small().strong());
                    for (value, shown) in choices {
                        ui.horizontal(|ui| {
                            let on = row.value.eq_ignore_ascii_case(value);
                            let radio = ui.radio(on, shown);
                            self.session_dialog.rects.insert(format!("Dialog:{}={}", row.name, value), radio.rect);
                            if radio.clicked() && !on {
                                acts.push(Act::Set(row.name.clone(), value.clone()));
                            }
                            // The number that goes with this choice.
                            if let Some((n, _)) = beside
                                .iter()
                                .find(|(_, (owner, v))| *owner == row.name && v == value)
                                && let Some(nr) = number_row(n)
                            {
                                ui.add_enabled_ui(on, |ui| {
                                    self.number_field(ui, nr, acts);
                                    ui.label(suffix(&nr.name, value, vi));
                                });
                            }
                        });
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label(&row.label);
                    let current = choices
                        .iter()
                        .find(|(v, _)| v.eq_ignore_ascii_case(&row.value))
                        .map(|(_, s)| s.clone())
                        .unwrap_or_else(|| row.value.clone());
                    egui::ComboBox::from_id_salt(("om9-dialog-choice", &row.name))
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for (value, shown) in choices {
                                let on = row.value.eq_ignore_ascii_case(value);
                                if ui.selectable_label(on, shown).clicked() && !on {
                                    acts.push(Act::Set(row.name.clone(), value.clone()));
                                }
                            }
                        });
                });
            }
        }
        let numbers: Vec<&DialogRow> = dialog
            .rows
            .iter()
            .filter(|r| r.kind == DialogKind::Number && !beside.contains_key(&r.name))
            .collect();
        if !numbers.is_empty() {
            ui.add_space(4.0);
            egui::Grid::new(("om9-dialog-numbers", &dialog.code))
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    for row in numbers {
                        ui.label(&row.label);
                        self.number_field(ui, row, acts);
                        ui.end_row();
                    }
                });
        }
        let actions: Vec<&DialogRow> = dialog.rows.iter().filter(|r| r.kind == DialogKind::Action).collect();
        if !actions.is_empty() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for row in actions {
                    if ui.button(&row.label).clicked() {
                        acts.push(Act::Set(row.name.clone(), String::new()));
                    }
                }
            });
        }
        ui.add_space(4.0);
        ui.label(RichText::new(&dialog.prompt).small().weak());
        ui.separator();
        ui.horizontal(|ui| {
            let ok = ui
                .add_enabled(dialog.ready, egui::Button::new(RichText::new("OK").strong())
                        .min_size(egui::vec2(72.0, 0.0))
                        .fill(egui::Color32::from_rgb(222, 228, 236))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 140, 170))))
                .on_hover_text(if vi { "Tạo (Enter hoặc chuột phải)" } else { "Create (Enter or right click)" });
            self.session_dialog.rects.insert("Dialog:OK".into(), ok.rect);
            if ok.clicked() {
                acts.push(Act::Ok);
            }
            let cancel = ui
                .add(egui::Button::new(if vi { "Huỷ" } else { "Cancel" })
                        .min_size(egui::vec2(72.0, 0.0))
                        .fill(egui::Color32::from_rgb(232, 232, 228))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 155))))
                .on_hover_text("Esc");
            self.session_dialog.rects.insert("Dialog:Cancel".into(), cancel.rect);
            if cancel.clicked() {
                acts.push(Act::Cancel);
            }
        });
    }

    /// A number field: typed, then committed with Enter or by leaving the field.
    fn number_field(&mut self, ui: &mut egui::Ui, row: &DialogRow, acts: &mut Vec<Act>) {
        let entry = self
            .session_dialog
            .drafts
            .entry(row.name.clone())
            .or_insert_with(|| (row.value.clone(), row.value.clone()));
        // The value changed elsewhere (command line): show it.
        if entry.0 != row.value {
            *entry = (row.value.clone(), row.value.clone());
        }
        let edit = ui.add(egui::TextEdit::singleline(&mut entry.1).desired_width(64.0));
        self.session_dialog.rects.insert(format!("Dialog:{}", row.name), edit.rect);
        let done = edit.lost_focus() || (edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
        if done && entry.1.trim() != row.value && !entry.1.trim().is_empty() {
            acts.push(Act::Set(row.name.clone(), entry.1.trim().to_owned()));
        }
    }
}
