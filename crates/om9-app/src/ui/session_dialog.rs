// SPDX-License-Identifier: LGPL-2.1-or-later
//! The small options window of an interactive add-on command (Rhino 5 / Matrix 9 "Sweep 2
//! Rails Options" …), drawn from [`crate::session::AddonDialog`].
//!
//! Checkboxes, radio buttons and number fields edit the same values as the command line; a
//! number option named after a choice (`RebuildPoints` after `Rebuild`) sits beside that
//! choice, as in Rhino's Simplify group. OK = Enter = right click, Cancel = Esc.
use super::MatrixApp;
use crate::session::{AddonDialog, DialogKind, DialogRow};
use eframe::egui::{self, RichText};
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
    pub(super) fn sync_session_dialog(&mut self) {
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
    }

    pub(super) fn session_options_ui(&mut self, ui: &mut egui::Ui) {
        let Some(dialog) = self.controller.addon_dialog() else { return; };
        let vi = self.controller.tr().lang() == om9_addons::i18n::Lang::Vi;
        let mut acts = Vec::new();
        let response = ui.scope(|ui| self.session_dialog_body(ui, &dialog, vi, &mut acts));
        self.session_dialog.rects.insert("OptionsDialog".into(), response.response.rect);
        for act in acts {
            let result = match act {
                Act::Set(name, value) => {
                    let result = self.controller.set_addon_dialog_value(&name, &value);
                    if result.is_ok() {
                        self.session_dialog.drafts.remove(&name);
                    }
                    result
                }
                Act::Ok => self.controller.submit_effect("").map(|_| ()),
                Act::Cancel => {
                    self.controller.cancel();
                    self.transcript
                        .append_history(&format!("{}: {}", dialog.title, if vi { "đã huỷ" } else { "cancelled" }));
                    Ok(())
                }
            };
            let failed = result.is_err();
            self.feedback(result);
            self.focus_cmd = true;
            if failed && dialog.code == "tsSplitCurves" {
                break;
            }
        }
    }

    fn session_dialog_body(
        &mut self,
        ui: &mut egui::Ui,
        dialog: &AddonDialog,
        vi: bool,
        acts: &mut Vec<Act>,
    ) {
        if dialog.code == "tsSplitCurves" {
            self.split_curve_controls(ui, dialog, vi, acts);
            return;
        }
        if dialog.code == "ReduceMesh" {
            self.reduce_mesh_controls(ui, dialog, vi, acts);
        } else {
            self.generic_dialog_controls(ui, dialog, vi, acts);
        }
        for line in &dialog.footer {
            ui.label(line);
        }
        ui.add_space(4.0);
        ui.label(RichText::new(&dialog.prompt).small().weak());
        ui.separator();
        ui.horizontal(|ui| {
            let ok = ui
                .add_enabled(dialog.ready, egui::Button::new(RichText::new("OK").strong())
                        .min_size(egui::vec2(72.0, 0.0))
                        .fill(super::theme::SELECTED)
                        .stroke(egui::Stroke::new(1.0, super::theme::EDGE)))
                .on_hover_text(if vi { "Tạo (Enter hoặc chuột phải)" } else { "Create (Enter or right click)" });
            self.session_dialog.rects.insert("Dialog:OK".into(), ok.rect);
            if ok.clicked() {
                acts.push(Act::Ok);
            }
            let cancel = ui
                .add(egui::Button::new(if vi { "Huỷ" } else { "Cancel" })
                        .min_size(egui::vec2(72.0, 0.0))
                        .fill(super::theme::HEADER)
                        .stroke(egui::Stroke::new(1.0, super::theme::EDGE)))
                .on_hover_text("Esc");
            self.session_dialog.rects.insert("Dialog:Cancel".into(), cancel.rect);
            if cancel.clicked() {
                acts.push(Act::Cancel);
            }
            if let Some(row) = dialog.rows.iter().find(|r| r.name == "Preview" && r.kind == DialogKind::Action) {
                let preview = ui.add(egui::Button::new(&row.label).min_size(egui::vec2(72., 0.)));
                self.session_dialog.rects.insert("Dialog:Preview".into(), preview.rect);
                if preview.clicked() { acts.push(Act::Set(row.name.clone(), String::new())); }
            }
            // Rhino's Preview button, beside OK and Cancel: shows or hides the result.
            if let Some(row) = dialog.rows.iter().find(|r| r.name == "Preview" && r.kind == DialogKind::Toggle) {
                let on = row.value.eq_ignore_ascii_case("yes");
                let preview = ui
                    .add(egui::Button::selectable(on, row.label.clone()).min_size(egui::vec2(72.0, 0.0)))
                    .on_hover_text(if vi { "Xem trước kết quả" } else { "Show the result before OK" });
                self.session_dialog.rects.insert("Dialog:Preview".into(), preview.rect);
                if preview.clicked() {
                    acts.push(Act::Set(row.name.clone(), if on { "No" } else { "Yes" }.into()));
                }
            }
        });
    }

    /// One row of the window: a checkbox, radio buttons (or a list when `list` and there are
    /// many choices) with their numbers beside, a labelled number field, or a button.
    #[allow(clippy::too_many_arguments)]
    fn dialog_row(
        &mut self,
        ui: &mut egui::Ui,
        dialog: &AddonDialog,
        row: &DialogRow,
        beside: &BTreeMap<String, (String, String)>,
        vi: bool,
        acts: &mut Vec<Act>,
        framed: bool,
    ) {
        let enabled = dialog.enabled(&row.name);
        match &row.kind {
            DialogKind::Toggle => {
                let mut on = row.value.eq_ignore_ascii_case("yes");
                let check = ui.add_enabled(enabled, egui::Checkbox::new(&mut on, &row.label));
                self.session_dialog.rects.insert(format!("Dialog:{}", row.name), check.rect);
                if check.changed() {
                    acts.push(Act::Set(row.name.clone(), if on { "Yes" } else { "No" }.into()));
                }
            }
            DialogKind::Choice(choices) if choices.len() <= 4 => {
                let mut body = |ui: &mut egui::Ui| {
                    ui.label(RichText::new(&row.label).small().strong());
                    for (value, shown) in choices {
                        ui.horizontal(|ui| {
                            let on = row.value.eq_ignore_ascii_case(value);
                            let radio = ui.add_enabled(enabled, egui::RadioButton::new(on, shown));
                            self.session_dialog.rects.insert(format!("Dialog:{}={}", row.name, value), radio.rect);
                            if radio.clicked() && !on {
                                acts.push(Act::Set(row.name.clone(), value.clone()));
                            }
                            // The number that goes with this choice.
                            if let Some((n, _)) = beside.iter().find(|(_, (owner, v))| *owner == row.name && v == value)
                                && let Some(nr) = dialog.rows.iter().find(|r| &r.name == n)
                            {
                                ui.add_enabled_ui(on && enabled, |ui| {
                                    self.number_field(ui, nr, acts);
                                    ui.label(suffix(&nr.name, value, vi));
                                });
                            }
                        });
                    }
                };
                if framed {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_min_width(220.0);
                        body(ui);
                    });
                } else {
                    body(ui);
                }
            }
            DialogKind::Choice(choices) => {
                ui.horizontal(|ui| {
                    ui.label(&row.label);
                    let current = choices
                        .iter()
                        .find(|(v, _)| v.eq_ignore_ascii_case(&row.value))
                        .map(|(_, s)| s.clone())
                        .unwrap_or_else(|| row.value.clone());
                    ui.add_enabled_ui(enabled, |ui| {
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
                });
            }
            DialogKind::Number => {
                ui.add_enabled_ui(enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&row.label);
                        self.number_with_note(ui, dialog, row, acts);
                    });
                });
            }
            DialogKind::Action => {
                let text = match dialog.notes.get(&row.name) {
                    Some(note) => format!("{} {note}", row.label),
                    None => row.label.clone(),
                };
                let button = ui.add_enabled(enabled, egui::Button::new(text).min_size(egui::vec2(220.0, 0.0)));
                self.session_dialog.rects.insert(format!("Dialog:{}", row.name), button.rect);
                if button.clicked() {
                    acts.push(Act::Set(row.name.clone(), String::new()));
                }
            }
        }
    }

    fn split_curve_controls(
        &mut self,
        ui: &mut egui::Ui,
        dialog: &AddonDialog,
        vi: bool,
        acts: &mut Vec<Act>,
    ) {
        let Some(file) = dialog.rows.iter().find(|r| r.name == "UseFileTolerance") else {
            return;
        };
        let Some(tolerance) = dialog.rows.iter().find(|r| r.name == "Tolerance") else {
            return;
        };
        let mut use_file = file.value.eq_ignore_ascii_case("yes");
        egui::Grid::new("tsSplitCurves fields")
            .spacing([16., 5.])
            .show(ui, |ui| {
                let has_file = !dialog.notes.contains_key("UseFileTolerance");
                let label = ui.add_enabled(
                    has_file,
                    egui::Label::new(if vi {
                        "Dùng dung sai tài liệu"
                    } else {
                        "Use File Tolerance"
                    })
                    .sense(egui::Sense::click()),
                );
                let toggle = ui.allocate_ui_with_layout(
                    egui::vec2(76., 18.), egui::Layout::right_to_left(egui::Align::Center),
                    |ui| ui.add_enabled(has_file, egui::Checkbox::new(&mut use_file, "")).labelled_by(label.id),
                ).inner;
                self.session_dialog
                    .rects
                    .insert("Dialog:UseFileTolerance".into(), toggle.rect);
                if label.clicked() {
                    use_file = !use_file;
                }
                if toggle.changed() || label.clicked() {
                    acts.push(Act::Set(
                        file.name.clone(),
                        if use_file { "Yes" } else { "No" }.into(),
                    ));
                }
                ui.end_row();
                ui.label(if vi { "Dung sai" } else { "Tolerance" });
                ui.allocate_ui_with_layout(egui::vec2(76.,18.),egui::Layout::left_to_right(egui::Align::Center),|ui| {
                    ui.add_enabled_ui(!use_file,|ui|self.number_field(ui,tolerance,acts));
                });
                ui.end_row();
            });
        ui.add_space(6.);
        ui.horizontal(|ui| {
            for (name, label) in [
                ("OK", "OK"),
                ("Display", if vi { "Hiển thị" } else { "Display" }),
                ("Cancel", if vi { "Huỷ" } else { "Cancel" }),
            ] {
                let response = ui.add(egui::Button::new(label).min_size(egui::vec2(58., 0.)));
                self.session_dialog
                    .rects
                    .insert(format!("Dialog:{name}"), response.rect);
                if response.clicked() {
                    // A click on OK/Display must include the current number edit even
                    // when egui transfers focus after laying out the number field.
                    if name != "Cancel"
                        && !use_file
                        && let Some((old, text)) = self.session_dialog.drafts.get("Tolerance")
                        && text.trim() != old
                        && !acts
                            .iter()
                            .any(|a| matches!(a,Act::Set(n,_) if n=="Tolerance"))
                    {
                        acts.push(Act::Set("Tolerance".into(), text.trim().into()));
                    }
                    if name == "Cancel" {
                        acts.clear();
                    }
                    acts.push(match name {
                        "OK" => Act::Ok,
                        "Display" => Act::Set(name.into(), String::new()),
                        _ => Act::Cancel,
                    });
                }
            }
        });
        for line in &dialog.footer {
            ui.label(line);
        }
    }

    /// Rhino: "Point count: (2) [5]", the current value before the field.
    fn number_with_note(&mut self, ui: &mut egui::Ui, dialog: &AddonDialog, row: &DialogRow, acts: &mut Vec<Act>) {
        if let Some(note) = dialog.notes.get(&row.name) {
            ui.label(RichText::new(note).weak());
        }
        self.number_field(ui, row, acts);
    }

    /// A number field: typed, then committed with Enter or by leaving the field.
    fn generic_dialog_controls(&mut self, ui: &mut egui::Ui, dialog: &AddonDialog, vi: bool, acts: &mut Vec<Act>) {
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
        // Framed groups first (Rhino's "Rail curve options", "Cross-section curve options" …),
        // each in the order the command gives.
        let mut drawn: std::collections::BTreeSet<String> = beside.keys().cloned().collect();
        drawn.insert("Preview".into());
        for (title, names) in &dialog.groups {
            let rows: Vec<&DialogRow> = names
                .iter()
                .filter_map(|n| dialog.rows.iter().find(|r| &r.name == n))
                .filter(|r| !drawn.contains(&r.name))
                .collect();
            if rows.is_empty() {
                continue;
            }
            ui.add_space(4.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(240.0);
                if !title.is_empty() {
                    ui.label(RichText::new(title).small().strong());
                }
                for row in rows {
                    drawn.insert(row.name.clone());
                    self.dialog_row(ui, dialog, row, &beside, vi, acts, false);
                }
            });
        }
        // The rest: checkboxes, choices, numbers, then buttons (Rhino's usual order).
        let rest = |kind: fn(&DialogKind) -> bool| -> Vec<&DialogRow> {
            dialog.rows.iter().filter(|r| kind(&r.kind) && !drawn.contains(&r.name)).collect()
        };
        for row in rest(|k| *k == DialogKind::Toggle) {
            self.dialog_row(ui, dialog, row, &beside, vi, acts, false);
        }
        for row in rest(|k| matches!(k, DialogKind::Choice(_))) {
            ui.add_space(4.0);
            self.dialog_row(ui, dialog, row, &beside, vi, acts, true);
        }
        let numbers = rest(|k| *k == DialogKind::Number);
        if !numbers.is_empty() {
            ui.add_space(4.0);
            egui::Grid::new(("om9-dialog-numbers", &dialog.code))
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    for row in numbers {
                        ui.add_enabled(dialog.enabled(&row.name), egui::Label::new(&row.label));
                        ui.add_enabled_ui(dialog.enabled(&row.name), |ui| {
                            ui.horizontal(|ui| self.number_with_note(ui, dialog, row, acts));
                        });
                        ui.end_row();
                    }
                });
        }
        let actions = rest(|k| *k == DialogKind::Action);
        if !actions.is_empty() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for row in actions {
                    self.dialog_row(ui, dialog, row, &beside, vi, acts, false);
                }
            });
        }
    }

    fn reduce_mesh_controls(&mut self, ui: &mut egui::Ui, dialog: &AddonDialog, vi: bool, acts: &mut Vec<Act>) {
        let row = |name: &str| dialog.rows.iter().find(|r| r.name == name);
        let Some(method) = row("Method") else { return };
        for (value, field, label) in [
            ("Count", "Polygons", if vi { "Giảm còn (tam giác)" } else { "Reduce to (polygons)" }),
            ("Percent", "Percent", if vi { "Giảm theo (%)" } else { "Reduce by (%)" }),
        ] {
            ui.horizontal(|ui| {
                let selected = method.value == value;
                if ui.radio(selected, label).clicked() { acts.push(Act::Set("Method".into(), value.into())); }
                if let Some(r) = row(field) {
                    ui.add_enabled_ui(selected, |ui| self.number_field(ui, r, acts));
                }
            });
        }
        if let Some(r) = row("PlanarOnly") {
            let mut on = r.value.eq_ignore_ascii_case("yes");
            if ui.checkbox(&mut on, if vi { "Chỉ giảm vùng phẳng" } else { "Planar only" }).changed() {
                acts.push(Act::Set(r.name.clone(), if on { "Yes" } else { "No" }.into()));
            }
        }
        if let Some(r) = row("Accuracy") {
            let mut accuracy = r.value.parse::<u8>().unwrap_or(8);
            ui.horizontal(|ui| {
                ui.label(if vi { "Nhanh" } else { "Fast" });
                if ui.add(egui::Slider::new(&mut accuracy,1..=10).show_value(false)).changed() {
                    acts.push(Act::Set(r.name.clone(),accuracy.to_string()));
                }
                ui.label(if vi { "Chính xác" } else { "Accurate" });
            });
        }
    }

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
