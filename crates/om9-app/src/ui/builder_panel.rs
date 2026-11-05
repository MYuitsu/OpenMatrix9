// SPDX-License-Identifier: LGPL-2.1-or-later
//! The add-on builder in the BUILDERS area below the Project Manager, and its viewport preview.
//!
//! The layout follows the Matrix builder idea (thumbnail on top, parameters below, one slider
//! for the parameter being tuned) but every parameter stays visible as a draggable/typable
//! field, so no parameter is hidden behind an icon. The same values are on the command line
//! (`Size=1.5 …`); both edit the one builder state owned by the controller.
use super::MatrixApp;
use super::chrome::PANEL_INK;
use crate::session::{AddonBuilder, display_value};
use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, Vec2};
use om9_addons::settings::{Kind, SettingSpec, SettingValue, Unit};
use om9_scene::camera::{Camera, Viewport};

/// Preview outline colour (cyan, readable on the dark viewport and over white grids).
const PREVIEW: [u8; 4] = [64, 200, 255, 230];
/// Rubber band from the last point to the cursor.
const RUBBER: [u8; 4] = [255, 220, 90, 230];
const WARN: Color32 = Color32::from_rgb(255, 150, 120);

#[derive(Default)]
pub(crate) struct BuilderUi {
    /// The parameter the slider is tuning (a numeric setting key).
    pub focus: Option<String>,
    /// The builder code last seen, to notice a builder opening or closing.
    pub seen: Option<String>,
    /// The prompt last written by the builder, so it is rewritten only when it changes.
    pub prompt: Option<String>,
}

fn numeric_range(spec: &SettingSpec) -> Option<(f64, f64, f64, Unit)> {
    match &spec.kind {
        Kind::Float {
            min,
            max,
            step,
            unit,
        } => {
            let (lo, hi) = (min.unwrap_or(0.0), max.unwrap_or(100.0));
            Some((lo, hi, step.unwrap_or((hi - lo) / 200.0).max(1e-4), *unit))
        }
        Kind::Int { min, max } => Some((
            min.unwrap_or(0) as f64,
            max.unwrap_or(100) as f64,
            1.0,
            Unit::None,
        )),
        _ => None,
    }
}

fn as_f64(v: &SettingValue) -> f64 {
    match v {
        SettingValue::Float(f) => *f,
        SettingValue::Int(i) => *i as f64,
        _ => 0.0,
    }
}

fn decimals(step: f64) -> usize {
    if step >= 1.0 {
        0
    } else if step >= 0.1 {
        1
    } else if step >= 0.01 {
        2
    } else {
        3
    }
}

impl MatrixApp {
    /// Per frame while the application runs: keep the preview and the prompt current, and
    /// echo the command name when a builder opens from a menu or icon.
    pub(super) fn builder_frame(&mut self) {
        self.controller.refresh_addon_session();
        self.controller.refresh_builder_preview();
        let code = self
            .controller
            .addon_builder()
            .map(|b| b.code.clone())
            .or_else(|| self.controller.addon_session().map(|s| s.code.clone()));
        if code != self.builder_ui.seen {
            if code.is_some() {
                self.builder_ui.focus = None;
                // Enter ends the builder wherever the command was started from (icon, menu):
                // the command line takes the keyboard, as after typing the command.
                self.focus_cmd = true;
            }
            self.builder_ui.seen = code.clone();
        }
        if code.is_some() || self.builder_ui.prompt.is_some() {
            let prompt = self.controller.prompt_text();
            if self.builder_ui.prompt.as_deref() != Some(prompt.as_str()) {
                self.transcript.set_prompt(&prompt);
            }
            self.builder_ui.prompt = code.is_some().then_some(prompt);
        }
    }

    /// The command name echoed into the history when a builder opens without typing.
    pub(super) fn echo_builder_start(&mut self, before: Option<String>) {
        let started = self
            .controller
            .addon_session()
            .map(|s| (s.code.clone(), s.typed.clone(), s.label.clone()))
            .or_else(|| {
                self.controller
                    .addon_builder()
                    .map(|b| (b.code.clone(), b.typed.clone(), b.label.clone()))
            });
        let Some((code, typed, label)) = started else {
            return;
        };
        if before.as_deref() == Some(code.as_str()) {
            return;
        }
        let name = typed.unwrap_or(label);
        self.transcript.append_history(&format!("Command: {name}"));
    }

    /// Draw the open builder's preview over one viewport.
    pub(super) fn paint_builder_preview(
        &self,
        painter: &egui::Painter,
        camera: &Camera,
        viewport: Viewport,
    ) {
        if let Some(session) = self.controller.addon_session() {
            for [a, c] in session.lines.iter().chain(&session.hover_lines) {
                super::paint_line(painter, camera, viewport, *a, *c, PREVIEW, 1.2);
            }
            if let (Some(from), Some(to)) = (session.rubber_from, self.hover) {
                super::paint_line(painter, camera, viewport, from, to, RUBBER, 1.0);
            }
        }
        let Some(b) = self.controller.addon_builder() else {
            return;
        };
        for [a, c] in &b.preview.edges {
            super::paint_line(painter, camera, viewport, *a, *c, PREVIEW, 1.0);
        }
    }

    /// The builder panel. Returns false when no builder is open (the reserved slot shows).
    pub(super) fn builder_panel_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(b) = self.controller.addon_builder().cloned() else {
            return false;
        };
        let mut changes: Vec<(String, SettingValue)> = Vec::new();
        let (mut accept, mut cancel, mut reset) = (false, false, false);
        ui.label(RichText::new(&b.label).strong().color(PANEL_INK))
            .on_hover_text(format!("Add-on: {}", b.addon_name));
        self.builder_thumbnail(ui, &b);
        // Accept / Cancel stay at the top so they are visible however short the column is.
        ui.horizontal_wrapped(|ui| {
            let can = b.preview.count > 0;
            accept = ui
                .add_enabled(can, egui::Button::new(RichText::new("✔ Đặt").strong()))
                .on_hover_text("Tạo đá như bản xem trước (Enter)")
                .on_disabled_hover_text("Chọn đường cong để có bản xem trước")
                .clicked();
            cancel = ui
                .button("✖ Huỷ")
                .on_hover_text("Đóng builder (Esc)")
                .clicked();
            reset = ui
                .button("↺")
                .on_hover_text("Về thông số gốc của add-on")
                .clicked();
        });
        match &b.preview.error {
            Some(e) => {
                ui.colored_label(WARN, RichText::new(e).small());
            }
            None => {
                let summary = b.preview.messages.join("\n");
                ui.label(RichText::new(summary).small());
            }
        }
        ui.separator();
        let focus = self.builder_ui.focus.clone().or_else(|| {
            b.specs
                .iter()
                .find(|s| numeric_range(s).is_some())
                .map(|s| s.key.clone())
        });
        egui::Grid::new(("builder-params", &b.code))
            .num_columns(2)
            .spacing([4., 2.])
            .show(ui, |ui| {
                for spec in &b.specs {
                    let value = b.value(&spec.key).unwrap_or(&spec.default).clone();
                    let selected = focus.as_deref() == Some(spec.key.as_str());
                    let label = ui
                        .selectable_label(selected, RichText::new(&spec.label).small())
                        .on_hover_text(format!(
                            "{}{}  ·  CMD: {}=…",
                            spec.label,
                            spec.description
                                .as_deref()
                                .map(|d| format!(" — {d}"))
                                .unwrap_or_default(),
                            spec.option
                        ));
                    if label.clicked() && numeric_range(spec).is_some() {
                        self.builder_ui.focus = Some(spec.key.clone());
                    }
                    if let Some(change) = param_widget(ui, &b, spec, &value) {
                        if numeric_range(spec).is_some() {
                            self.builder_ui.focus = Some(spec.key.clone());
                        }
                        changes.push((spec.key.clone(), change));
                    }
                    ui.end_row();
                }
            });
        // One slider for the parameter being tuned, full width (Matrix-style quick tuning).
        if let Some(spec) = focus
            .as_deref()
            .and_then(|k| b.specs.iter().find(|s| s.key == k))
            && let Some((lo, hi, step, _)) = numeric_range(spec)
        {
            let mut v = as_f64(b.value(&spec.key).unwrap_or(&spec.default));
            ui.horizontal(|ui| {
                ui.label(RichText::new(&spec.option).small());
                ui.spacing_mut().slider_width = (ui.available_width() - 8.).max(60.);
                let r = ui.add(
                    egui::Slider::new(&mut v, lo..=hi)
                        .step_by(step)
                        .show_value(false),
                );
                if r.changed() {
                    changes.push((spec.key.clone(), number_value(spec, v)));
                }
            });
        }
        for (key, value) in changes {
            if let Err(e) = self.controller.set_builder_value(&key, value) {
                self.status.error(&e.to_string());
            }
        }
        if reset {
            self.controller.reset_builder_defaults();
        }
        if cancel {
            // Closes the builder and, for an interactive command, its session.
            self.controller.cancel();
            self.transcript
                .append_history(&format!("{}: đã huỷ", b.label));
            self.feedback(Ok(()));
        } else if accept {
            let result = self.controller.accept_addon_builder();
            self.feedback(result);
        }
        true
    }

    /// The thumbnail: the command's menu picture and a live sketch of the current layout.
    fn builder_thumbnail(&mut self, ui: &mut egui::Ui, b: &AddonBuilder) {
        let width = ui.available_width().clamp(120., 220.);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 52.), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2, Color32::from_gray(48));
        let icon = Rect::from_min_size(rect.min + Vec2::new(3., 3.), Vec2::splat(46.));
        if let Some(name) = crate::icons::icon_name(&b.code) {
            self.icons.paint(ui, icon, name);
        }
        let sketch = Rect::from_min_max(
            egui::pos2(icon.max.x + 6., rect.min.y + 6.),
            rect.max - Vec2::new(6., 6.),
        );
        paint_layout_sketch(&painter, sketch, b);
        response.on_hover_text(
            "Xem trước hiện trong khung nhìn. Chỉnh thông số ở đây hoặc trên thanh Command.",
        );
    }
}

/// A row's editor. Returns the new value when the user changed it.
fn param_widget(
    ui: &mut egui::Ui,
    b: &AddonBuilder,
    spec: &SettingSpec,
    value: &SettingValue,
) -> Option<SettingValue> {
    match (&spec.kind, value) {
        (Kind::Bool, SettingValue::Bool(v)) => {
            let mut on = *v;
            ui.checkbox(&mut on, "")
                .changed()
                .then_some(SettingValue::Bool(on))
        }
        (Kind::Enum { choices }, SettingValue::Str(current)) => {
            let mut chosen = current.clone();
            egui::ComboBox::from_id_salt(("builder-enum", &b.code, &spec.key))
                .selected_text(display_value(spec, value))
                .width(84.)
                .show_ui(ui, |ui| {
                    for (i, c) in choices.iter().enumerate() {
                        let text = spec.choice_labels.get(i).unwrap_or(c);
                        ui.selectable_value(&mut chosen, c.clone(), text);
                    }
                });
            (&chosen != current).then_some(SettingValue::Str(chosen))
        }
        _ => {
            let (lo, hi, step, unit) = numeric_range(spec)?;
            let mut v = as_f64(value);
            let r = ui.add(
                egui::DragValue::new(&mut v)
                    .range(lo..=hi)
                    .speed(step)
                    .fixed_decimals(decimals(step))
                    .suffix(unit.suffix()),
            );
            r.changed().then(|| number_value(spec, v))
        }
    }
}

fn number_value(spec: &SettingSpec, v: f64) -> SettingValue {
    match spec.kind {
        Kind::Int { .. } => SettingValue::Int(v.round() as i64),
        _ => SettingValue::Float((v * 1e6).round() / 1e6),
    }
}

/// Three stones on a rail, drawn to the current diameter/spacing ratio, with the values.
fn paint_layout_sketch(painter: &egui::Painter, rect: Rect, b: &AddonBuilder) {
    let num = |key: &str| b.value(key).map(as_f64);
    let (Some(d), gap) = (num("diameter"), num("spacing").unwrap_or(0.0)) else {
        // Not a gem layout: show the option summary instead.
        let text = b
            .tokens()
            .into_iter()
            .take(3)
            .map(|(_, t)| t)
            .collect::<Vec<_>>()
            .join("\n");
        painter.text(
            rect.left_top(),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::proportional(10.),
            PANEL_INK,
        );
        return;
    };
    let rail_y = rect.center().y + 4.;
    painter.line_segment(
        [
            egui::pos2(rect.left(), rail_y),
            egui::pos2(rect.right(), rail_y),
        ],
        Stroke::new(1.0, Color32::from_rgb(120, 200, 140)),
    );
    // Scale so three stones and two gaps fill the width.
    let span = 3.0 * d + 2.0 * gap;
    let scale = (rect.width() as f64 / span.max(1e-6)).min(rect.height() as f64 * 0.55 / d);
    let r = (d * scale / 2.0) as f32;
    let step = ((d + gap) * scale) as f32;
    let x0 = rect.center().x - step;
    for i in 0..3 {
        let c = egui::pos2(x0 + step * i as f32, rail_y);
        painter.circle(
            c,
            r,
            Color32::from_rgb(200, 220, 240),
            Stroke::new(1.0, Color32::from_rgb(40, 60, 80)),
        );
        painter.circle_stroke(
            c,
            r * 0.55,
            Stroke::new(0.8, Color32::from_rgb(90, 110, 130)),
        );
    }
    let font = egui::FontId::proportional(10.);
    painter.text(
        egui::pos2(rect.left(), rect.top()),
        egui::Align2::LEFT_TOP,
        format!("Ø {}", trim(d)),
        font.clone(),
        PANEL_INK,
    );
    painter.text(
        egui::pos2(rect.right(), rect.top()),
        egui::Align2::RIGHT_TOP,
        format!("khe {}", trim(gap)),
        font.clone(),
        PANEL_INK,
    );
    painter.text(
        egui::pos2(rect.left(), rect.bottom()),
        egui::Align2::LEFT_BOTTOM,
        format!("{} viên", b.preview.count),
        font,
        PANEL_INK,
    );
}

fn trim(v: f64) -> String {
    let t = format!("{v:.2}");
    t.trim_end_matches('0').trim_end_matches('.').to_owned()
}
