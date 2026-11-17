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
    pub rail_library_open: bool,
    pub rail_name: String,
    pub rail_edit_curve: Option<om9_core::geometry::Curve>,
    pub rail_edit_point: usize,
    pub rail_error: Option<String>,
    /// The parameter the slider is tuning (a numeric setting key).
    pub focus: Option<String>,
    /// The builder code last seen, to notice a builder opening or closing.
    pub seen: Option<String>,
    /// The prompt last written by the builder, so it is rewritten only when it changes.
    pub prompt: Option<String>,
    pub seen_metal: bool,
    pub metal_user_id: String,
    pub metal_user_name: String,
    pub metal_user_density: String,
    pub metal_user_provenance: String,
    pub metal_user_uncertainty: String,
    pub metal_editing: Option<String>,
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
    pub(super) fn ring_rail_confirmation_ui(&mut self, ctx: &egui::Context) {
        use crate::{command_options::CommandOptionInterface, session::ring_rail::ReplaceChoice};
        let Some(label) = self.controller.ring_rail_confirmation_label().map(str::to_owned) else { return; };
        let unavailable = self.controller.ring_rail_replace_unavailable();
        let mut choice = None;
        let mut painted = false;
        egui::Modal::new(egui::Id::new("ring-rail-replace"))
            .frame(egui::Frame::popup(&ctx.style_of(egui::Theme::Light)))
            .show(ctx, |ui| {
                painted = !ui.is_sizing_pass();
                *ui.visuals_mut() = egui::Visuals::light();
                ui.set_width(340.);
                ui.heading(self.controller.tr().text("Replace Ring Rail?"));
                ui.label(format!("{}: {label}", self.controller.tr().text("Current Ring Rail")));
                ui.label(self.controller.tr().text("Yes replaces it. No creates an additional rail."));
                if let Some(reason) = unavailable { ui.colored_label(WARN, self.controller.tr().text(reason)); }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.add_enabled(unavailable.is_none(), egui::Button::new(self.controller.tr().text("Yes"))).clicked() { choice = Some(ReplaceChoice::Yes); }
                    if ui.button(self.controller.tr().text("No")).clicked() { choice = Some(ReplaceChoice::No); }
                    if ui.button(self.controller.tr().text("Cancel")).clicked() { choice = Some(ReplaceChoice::Cancel); }
                });
            });
        if !painted { return; }
        let composing = self.cmd_ime_composing || ctx.input(|input| input.events.iter().any(|event| matches!(event, egui::Event::Ime(_))));
        if !composing && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) { choice = Some(ReplaceChoice::Cancel); }
        if choice.is_none() && !composing && self.transcript.draft().is_empty() {
            for option in self.controller.command_options() {
                if option.id.name() == "Yes" && unavailable.is_some() { continue; }
                if let Some(shortcut) = option.shortcut && super::command::consume_command_shortcut(ctx, shortcut) {
                    self.submit_command_option(option.id);
                    ctx.request_repaint();
                    return;
                }
            }
        }
        if let Some(choice) = choice {
            let result = self.controller.decide_ring_rail_replace(choice);
            self.feedback(result);
            ctx.request_repaint();
        }
    }

    /// Load a complete validated user catalog; a corrupt file never replaces the active one.
    pub fn load_user_metal_catalog(&mut self, path: &std::path::Path) -> om9_core::Result<()> {
        let catalog = if path.exists() { crate::session::metal_catalog::load(path)? } else { Default::default() };
        self.controller.restore_user_metal_catalog(&catalog)?;
        self.user_metal_catalog = catalog;
        self.metal_catalog_path = Some(path.to_path_buf());
        Ok(())
    }

    /// Persist the validated candidate before exposing it to calculations.
    pub fn add_user_metal(&mut self, material: om9_core::metal_weight::MetalMaterial) -> om9_core::Result<()> {
        let mut candidate = self.user_metal_catalog.clone();
        candidate.materials.push(material);
        candidate.validate()?;
        if let Some(path) = &self.metal_catalog_path { crate::session::metal_catalog::save(path, &candidate)?; }
        self.controller.apply_user_metal_catalog(&candidate)?;
        self.user_metal_catalog = candidate;
        Ok(())
    }

    pub fn update_user_metal(&mut self, material: om9_core::metal_weight::MetalMaterial) -> om9_core::Result<()> {
        let mut candidate = self.user_metal_catalog.clone();
        let old = candidate.materials.iter_mut().find(|item| item.id.trim().eq_ignore_ascii_case(material.id.trim()))
            .ok_or(om9_core::Error::Missing("user metal material"))?;
        *old = material;
        candidate.validate()?;
        if let Some(path) = &self.metal_catalog_path { crate::session::metal_catalog::save(path, &candidate)?; }
        self.controller.apply_user_metal_catalog(&candidate)?;
        self.user_metal_catalog = candidate;
        Ok(())
    }

    pub fn delete_user_metal(&mut self, id: &str) -> om9_core::Result<()> {
        let mut candidate = self.user_metal_catalog.clone();
        let index = candidate.materials.iter().position(|item| item.id.trim().eq_ignore_ascii_case(id.trim()))
            .ok_or(om9_core::Error::Missing("user metal material"))?;
        if self.controller.metal_density_in_use(id)
            || candidate.bindings.iter().any(|binding| binding.density_id.trim().eq_ignore_ascii_case(id.trim())) {
            return Err(om9_core::Error::Invalid("metal material still bound"));
        }
        candidate.materials.remove(index);
        candidate.validate()?;
        if let Some(path) = &self.metal_catalog_path { crate::session::metal_catalog::save(path, &candidate)?; }
        self.controller.apply_user_metal_catalog(&candidate)?;
        self.user_metal_catalog = candidate;
        Ok(())
    }

    /// Save the candidate before changing the controller's effective binding.
    pub fn set_user_metal_binding(&mut self, render: om9_core::identity::MaterialId, density: Option<&str>) -> om9_core::Result<()> {
        self.controller.validate_metal_material_binding(render, density)?;
        let document = self.controller.document().id;
        let mut candidate = self.user_metal_catalog.clone();
        candidate.bindings.retain(|entry| entry.document != document || entry.render_material != render);
        if let Some(id) = density {
            let density_id = self.controller.metal_registry().get(id)?.id.clone();
            candidate.bindings.push(crate::session::metal_catalog::MetalDensityBinding { document, render_material: render, density_id });
        }
        candidate.validate()?;
        if let Some(path) = &self.metal_catalog_path { crate::session::metal_catalog::save(path, &candidate)?; }
        self.controller.bind_metal_material(render, density)?;
        self.user_metal_catalog = candidate;
        Ok(())
    }

    /// Per frame while the application runs: keep the preview and the prompt current, and
    /// echo the command name when a builder opens from a menu or icon.
    pub(super) fn builder_frame(&mut self) {
        let metal_open = self.controller.metal_panel_open();
        if metal_open && !self.builder_ui.seen_metal {
            self.sidebar_scroll = self.sidebar_content_size.map_or(10_000.0, |size| size.y + 200.0);
        }
        self.builder_ui.seen_metal = metal_open;
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
        if let Some(preview) = self.controller.builder()
            .and_then(|session| session.preview_for_document(self.controller.document()))
            && let om9_core::geometry::Geometry::Brep(brep) = &preview.geometry {
            use om9_core::brep::BrepSourceAccess;
            let stored = self.controller.coordinator().brep_representation(brep).ok();
            let candidate = preview.brep_candidates.iter().find(|candidate| candidate.to_canonical().identity() == brep.identity());
            let display = stored.as_ref().map(|entry| entry.display()).or_else(|| candidate.map(|entry| entry.display()));
            if let Some(display) = display {
                for [a, b, _] in &display.segments {
                    if let (Some(a), Some(b)) = (display.points.get(*a as usize), display.points.get(*b as usize)) {
                        super::paint_line(painter, camera, viewport, *a, *b, PREVIEW, 1.0);
                    }
                }
            }
        }
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
        if self.controller.metal_panel_open() {
            ui.label(RichText::new("Metal Weights").strong().color(PANEL_INK));
            ui.label("Select closed solids, then calculate.");
            let materials: Vec<_> = self.controller.metal_registry().iter()
                .map(|material| (material.id.clone(), material.name.clone())).collect();
            let mut use_object_materials = self.controller.metal_panel_object_materials();
            if ui.checkbox(&mut use_object_materials, "Use Object Materials").changed() {
                self.controller.set_metal_panel_object_materials(use_object_materials);
            }
            if use_object_materials {
                ui.label(if self.metal_catalog_path.is_some() {
                    "Assign a density to each render material (saved for this document)."
                } else { "Assign a density to each render material (this session)." });
                let render_materials: Vec<_> = self.controller.document().exchange.materials.values()
                    .map(|material| (material.id, material.name.clone())).collect();
                if render_materials.is_empty() { ui.label("No render materials in this document."); }
                for (id, name) in render_materials {
                    let mut binding = self.controller.bound_metal_material(id).map(str::to_owned);
                    let selected = binding.as_ref().and_then(|value|
                        materials.iter().find(|(candidate, _)| candidate.eq_ignore_ascii_case(value)))
                        .map(|(_, name)| name.as_str()).unwrap_or("Unassigned");
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{name} [{id}]"));
                        egui::ComboBox::from_id_salt(("metal-render-binding", id))
                            .selected_text(selected).show_ui(ui, |ui| {
                                ui.selectable_value(&mut binding, None, "Unassigned");
                                for (material_id, material_name) in &materials {
                                    ui.selectable_value(&mut binding, Some(material_id.clone()), material_name);
                                }
                            });
                    });
                    if binding.as_deref() != self.controller.bound_metal_material(id) {
                        let result = self.set_user_metal_binding(id, binding.as_deref());
                        self.feedback(result);
                    }
                }
            } else {
            let (mut reference, mut user) = self.controller.metal_panel_filters();
            ui.horizontal_wrapped(|ui| {
                let mut gv = false;
                ui.add_enabled(false, egui::Checkbox::new(&mut gv, "GV"))
                    .on_disabled_hover_text("Gemvision material catalog is not available");
                ui.checkbox(&mut reference, "OM9 reference");
                ui.checkbox(&mut user, "User");
            });
            self.controller.set_metal_panel_filters(reference, user);
            let visible: Vec<_> = materials.iter().filter(|(id, _)| {
                if self.controller.metal_registry().is_reference(id) { reference } else { user }
            }).collect();
            let mut material = self.controller.metal_panel_material().map(str::to_owned);
            if material.as_ref().is_some_and(|id|
                !visible.iter().any(|(candidate, _)| candidate.eq_ignore_ascii_case(id))) {
                material = None;
                let _ = self.controller.set_metal_panel_material(None);
            }
            let selected = material.as_ref()
                .and_then(|id| visible.iter().find(|(candidate, _)| candidate.eq_ignore_ascii_case(id)))
                .map(|(_, name)| name.as_str()).unwrap_or("All visible materials");
            egui::ComboBox::from_id_salt("metal-weight-material")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut material, None, "All visible materials");
                    for (id, name) in visible {
                        ui.selectable_value(&mut material, Some(id.clone()), name);
                    }
                });
            if material.as_deref() != self.controller.metal_panel_material() {
                let result = self.controller.set_metal_panel_material(material.as_deref());
                self.feedback(result);
            }
            let mut add_user = false;
            let mut cancel_edit = false;
            let mut edit_user = None;
            let mut delete_user = None;
            let entries = self.user_metal_catalog.materials.clone();
            egui::CollapsingHeader::new("Add user material").show(ui, |ui| {
                for entry in &entries {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{} ({})", entry.name, entry.id));
                        let edit = ui.small_button("Edit");
                        let delete = ui.small_button("Delete");
                        ui.ctx().memory_mut(|memory| {
                            memory.data.insert_temp(egui::Id::new(("metal-user-edit", &entry.id)), edit.rect);
                            memory.data.insert_temp(egui::Id::new(("metal-user-delete", &entry.id)), delete.rect);
                        });
                        if edit.clicked() { edit_user = Some(entry.clone()); }
                        if delete.clicked() { delete_user = Some(entry.id.clone()); }
                    });
                }
                ui.label("ID"); let id = ui.add_enabled(self.builder_ui.metal_editing.is_none(), egui::TextEdit::singleline(&mut self.builder_ui.metal_user_id));
                ui.label("Name"); let name = ui.text_edit_singleline(&mut self.builder_ui.metal_user_name);
                ui.label("Density (g/cm³)"); let density = ui.text_edit_singleline(&mut self.builder_ui.metal_user_density);
                ui.label("Density source"); let provenance = ui.text_edit_singleline(&mut self.builder_ui.metal_user_provenance);
                ui.label("Uncertainty (%)"); ui.text_edit_singleline(&mut self.builder_ui.metal_user_uncertainty);
                for (key, rect) in [("id", id.rect), ("name", name.rect), ("density", density.rect), ("provenance", provenance.rect)] {
                    ui.ctx().memory_mut(|memory| memory.data.insert_temp(egui::Id::new(("metal-user-field", key)), rect));
                }
                if self.metal_catalog_path.is_none() { ui.label("Available for this session only"); }
                add_user = ui.button(if self.builder_ui.metal_editing.is_some() { "Save changes" } else { "Save material" }).clicked();
                if self.builder_ui.metal_editing.is_some() { cancel_edit = ui.button("Cancel edit").clicked(); }
            });
            if let Some(entry) = edit_user {
                self.builder_ui.metal_user_id = entry.id.clone();
                self.builder_ui.metal_user_name = entry.name;
                self.builder_ui.metal_user_density = entry.density_g_cm3.to_string();
                self.builder_ui.metal_user_provenance = entry.provenance;
                self.builder_ui.metal_user_uncertainty = entry.uncertainty_percent.to_string();
                self.builder_ui.metal_editing = Some(entry.id);
            }
            if let Some(id) = delete_user {
                let result = self.delete_user_metal(&id);
                if result.is_ok() && self.builder_ui.metal_editing.as_deref() == Some(id.as_str()) { cancel_edit = true; }
                self.feedback(result);
            }
            if cancel_edit {
                self.builder_ui.metal_user_id.clear(); self.builder_ui.metal_user_name.clear();
                self.builder_ui.metal_user_density.clear(); self.builder_ui.metal_user_provenance.clear();
                self.builder_ui.metal_user_uncertainty.clear(); self.builder_ui.metal_editing = None;
            }
            if add_user {
                let editing = self.builder_ui.metal_editing.is_some();
                let result = self.builder_ui.metal_user_density.parse::<f64>()
                    .map_err(|_| om9_core::Error::Invalid("metal material density"))
                    .and_then(|density| om9_core::metal_weight::MetalMaterial::new(
                        &self.builder_ui.metal_user_id, &self.builder_ui.metal_user_name, density,
                        om9_core::metal_weight::DensityUnit::GramsPerCubicCentimeter,
                        &self.builder_ui.metal_user_provenance))
                    .and_then(|material| {
                        let text = self.builder_ui.metal_user_uncertainty.trim();
                        let uncertainty = if text.is_empty() { Ok(0.) } else {
                            text.parse::<f64>().map_err(|_| om9_core::Error::Invalid("metal material uncertainty"))
                        }?;
                        material.with_uncertainty_percent(uncertainty)
                    })
                    .and_then(|material| if editing { self.update_user_metal(material) } else { self.add_user_metal(material) });
                if result.is_ok() {
                    self.builder_ui.metal_user_id.clear(); self.builder_ui.metal_user_name.clear();
                    self.builder_ui.metal_user_density.clear(); self.builder_ui.metal_user_provenance.clear();
                    self.builder_ui.metal_user_uncertainty.clear(); self.builder_ui.metal_editing = None;
                }
                self.feedback(result);
            }
            }
            let mut calculate = false;
            let mut close = false;
            #[cfg(feature = "occt-brep")]
            let mut cancel_calculation = false;
            ui.horizontal_wrapped(|ui| {
                #[cfg(feature = "occt-brep")]
                { calculate = ui.add_enabled(!self.controller.metal_worker_pending() && !self.controller.metal_worker_active(), egui::Button::new("Calculate")).clicked(); }
                #[cfg(not(feature = "occt-brep"))]
                { calculate = ui.button("Calculate").clicked(); }
                #[cfg(feature = "occt-brep")]
                if self.controller.metal_worker_pending() { cancel_calculation = ui.button("Cancel calculation").clicked(); }
                close = ui.button("Close").clicked();
            });
            #[cfg(feature = "occt-brep")]
            if cancel_calculation { self.controller.cancel_metal_worker(); }
            #[cfg(feature = "occt-brep")]
            if self.controller.metal_worker_pending() { ui.label("Calculating…"); }
            #[cfg(feature = "occt-brep")]
            if self.controller.metal_worker_active() && !self.controller.metal_worker_pending() { ui.label("Canceling…"); }
            if calculate {
                let result = self.controller.calculate_metal_panel();
                self.feedback(result);
                ui.ctx().request_repaint();
            }
            if close { self.controller.close_metal_panel(); }
            return true;
        }
        let Some(b) = self.controller.addon_builder().cloned() else {
            return false;
        };
        let mut changes: Vec<(String, SettingValue)> = Vec::new();
        let (mut accept, mut cancel, mut reset) = (false, false, false);
        ui.label(RichText::new(&b.label).strong().color(PANEL_INK))
            .on_hover_text(format!("Add-on: {}", b.addon_name));
        self.builder_thumbnail(ui, &b);
        if matches!(b.code.as_str(), "BuilderRingRail" | "TopIconRingRail") { self.ring_rail_shape_ui(ui); }
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
                    // Greyed out while it means nothing (Fillet radius with Create=Curves).
                    let enabled = spec.enabled(|k| {
                        b.value(k)
                            .cloned()
                            .or_else(|| b.specs.iter().find(|s| s.key == k).map(|s| s.default.clone()))
                    });
                    let selected = focus.as_deref() == Some(spec.key.as_str());
                    let label = ui
                        .add_enabled(enabled, egui::Button::selectable(selected, RichText::new(&spec.label).small()))
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
                    if let Some(change) = ui.add_enabled_ui(enabled, |ui| param_widget(ui, &b, spec, &value)).inner {
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
