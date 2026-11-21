// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compact Top11 controls in the reserved slot below Projects.
use super::MatrixApp;
use eframe::egui::{self, Color32, RichText};
use om9_core::{
    identity::DocumentId,
    ring_size::{rail_diameter_mm, rail_size_for_diameter, rail_size_steps},
    top11::Top11Action,
};

const REGIONS: [(&str, &str); 9] = [
    ("us", "USA"),
    ("britain", "Britain"),
    ("japan", "Japan"),
    ("swiss", "Swiss"),
    ("european", "European"),
    ("france", "France"),
    ("german", "German"),
    ("vnm", "VNM"),
    ("diameter", "Custom Ring Size"),
];
const CUSTOM: [(&str, &str); 4] = [
    ("diameter", "Diameter (mm)"),
    ("iso", "Circumference (mm)"),
    ("diameter_inches", "Diameter (inch)"),
    ("circumference_inches", "Circumference (inch)"),
];

pub(super) struct RailToolbar {
    document: DocumentId,
    system: String,
    size: f64,
    draft: Option<String>,
    reveal: bool,
    placed: bool,
    shapes: bool,
}

fn label(system: &str) -> &str {
    REGIONS
        .iter()
        .find(|(id, _)| *id == system)
        .map(|(_, name)| *name)
        .unwrap_or(if system == "vnm_circumference" {
            "VNM"
        } else {
            "Custom"
        })
}

fn size_label(system: &str, size: f64) -> String {
    if system != "britain" {
        return size.to_string();
    }
    let n = size.floor() as u8;
    let letter = if n <= 26 {
        char::from(b'A' + n.saturating_sub(1)).to_string()
    } else {
        format!("Z+{}", n - 26)
    };
    if size.fract() == 0. {
        letter
    } else {
        format!("{letter} ½")
    }
}

fn arrow(ui: &mut egui::Ui, title: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new("▶").color(Color32::WHITE))
            .min_size(egui::vec2(21., 20.))
            .fill(Color32::from_rgb(47, 122, 30)),
    )
    .on_hover_text(title)
}

#[cfg(test)]
fn mark(ui: &egui::Ui, name: &str, response: &egui::Response) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(egui::Id::new(name), response.rect));
}

impl MatrixApp {
    pub(super) fn open_ring_rail_toolbar(&mut self) {
        let document = self.controller.document().id;
        let previous = self
            .builder_ui
            .rail_toolbar
            .take()
            .filter(|bar| bar.document == document);
        let (system, size) =
            previous.map_or_else(|| ("us".to_owned(), 7.), |bar| (bar.system, bar.size));
        let result = self.controller.set_ring_rail_size(&system, size);
        self.feedback(result);
        self.builder_ui.rail_toolbar = Some(RailToolbar {
            document,
            system,
            size,
            draft: None,
            reveal: true,
            placed: false,
            shapes: false,
        });
    }

    pub(super) fn ring_rail_toolbar_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(mut bar) = self.builder_ui.rail_toolbar.take() else {
            return false;
        };
        let active = self.controller.ring_rail_is_open();
        if bar.document != self.controller.document().id
            || (!active
                && (!bar.placed
                    || self.controller.is_running()
                    || self.controller.builder().is_some()
                    || self.controller.addon_builder().is_some()
                    || self.controller.metal_panel_open()))
        {
            return false;
        }
        if active && let Some((id, size)) = self.controller.ring_rail_size() {
            bar.system = if id == "explicit" {
                "diameter".into()
            } else {
                id.to_ascii_lowercase()
            };
            bar.size = size;
        }
        let mut close = false;
        let mut apply = false;
        let mut update = None;
        let enabled = !self.controller.ring_rail_confirmation_pending()
            && !self.controller.join_history_worker_pending()
            && (!active
                || self.controller.addon_builder().is_some()
                || self.controller.builder_editable());
        let row = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.;
            let width = ui.available_width();
            let choice_width = ((width - 126.) / 2.).clamp(40., 72.);
            let icon = ui
                .add(egui::Button::new("").min_size(egui::vec2(24., 22.)))
                .on_hover_text("Ring Rail shapes");
            let painter = ui.painter();
            painter.circle_stroke(
                icon.rect.center(),
                8.,
                egui::Stroke::new(2., Color32::from_rgb(62, 140, 250)),
            );
            painter.circle_stroke(
                icon.rect.center(),
                6.,
                egui::Stroke::new(1., Color32::LIGHT_GRAY),
            );
            if icon.clicked() {
                bar.shapes = !bar.shapes;
            }
            ui.label(RichText::new("RING\nRAIL").size(11.).strong());
            ui.add_enabled_ui(enabled, |ui| {
                let region = ui
                    .add_sized(
                        [choice_width, 20.],
                        egui::Button::new(if bar.system == "diameter" {
                            "Custom"
                        } else {
                            label(&bar.system)
                        }),
                    )
                    .on_hover_text("Select Ring Rail Region Type");
                if region.clicked() {
                    bar.draft = if bar.draft.is_some() {
                        None
                    } else {
                        Some(bar.system.clone())
                    };
                    bar.reveal = true;
                }
                if let Some((lo, hi, step)) = rail_size_steps(&bar.system) {
                    egui::ComboBox::from_id_salt("rail-size")
                        .width(choice_width)
                        .selected_text(size_label(&bar.system, bar.size))
                        .show_ui(ui, |ui| {
                            for q in (lo..=hi).step_by(step) {
                                let size = q as f64 / 4.;
                                if ui
                                    .selectable_label(
                                        size == bar.size,
                                        size_label(&bar.system, size),
                                    )
                                    .clicked()
                                {
                                    update = Some((bar.system.clone(), size));
                                }
                            }
                        });
                } else {
                    let mut size = bar.size;
                    let field = ui
                        .add_sized(
                            [choice_width, 20.],
                            egui::DragValue::new(&mut size).speed(0.05).max_decimals(4),
                        )
                        .on_hover_text(om9_core::ring_size::rail_system_label(&bar.system));
                    if field.changed() {
                        update = Some((bar.system.clone(), size));
                    }
                }
                let button = ui
                    .add_enabled_ui(bar.draft.is_none(), |ui| arrow(ui, "Create Ring Rail"))
                    .inner;
                #[cfg(test)]
                mark(ui, "builder-apply-test", &button);
                apply = button.clicked();
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                close = ui
                    .add_sized([18., 18.], egui::Button::new("×"))
                    .on_hover_text("Close Ring Rail")
                    .clicked();
            });
        });
        if close {
            if active {
                self.controller.cancel();
            }
            return true;
        }
        let mut controls = row.response.rect;
        if let Some(mut draft) = bar.draft.take() {
            let original = draft.clone();
            let mut dismiss = false;
            let panel = egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Select Ring Rail Region Type")
                            .strong()
                            .size(12.),
                    );
                    dismiss = ui.small_button("×").clicked();
                });
                for choices in REGIONS.chunks(4) {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.;
                        for &(id, title) in choices {
                            let selected = draft == id
                                || (id == "diameter"
                                    && CUSTOM.iter().any(|(key, _)| *key == draft))
                                || (id == "vnm" && draft == "vnm_circumference");
                            let choice = ui.radio(selected, title);
                            if choice.clicked() {
                                draft = id.into();
                            }
                        }
                    });
                }
                if draft == "vnm" || draft == "vnm_circumference" {
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(&mut draft, "vnm".into(), "Diameter (mm)");
                        ui.selectable_value(
                            &mut draft,
                            "vnm_circumference".into(),
                            "Circumference (mm)",
                        );
                    });
                } else if CUSTOM.iter().any(|(id, _)| *id == draft) {
                    ui.horizontal_wrapped(|ui| {
                        for (id, title) in CUSTOM {
                            ui.selectable_value(&mut draft, id.into(), title);
                        }
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Set Region");
                    let set = arrow(ui, "Set Region");
                    if set.clicked() {
                        let converted = rail_diameter_mm(&bar.system, bar.size)
                            .and_then(|diameter| rail_size_for_diameter(&draft, diameter));
                        // A custom diameter may lie outside the US table.
                        match converted {
                            Ok(size) => update = Some((draft.clone(), size)),
                            Err(_) if draft == "us" => update = Some((draft.clone(), 7.)),
                            Err(error) => self.feedback(Err(error)),
                        }
                        dismiss = update.is_some();
                    }
                });
            });
            controls = controls.union(panel.response.rect);
            bar.reveal |= original != draft;
            if !dismiss {
                bar.draft = Some(draft);
            }
        }
        if bar.shapes {
            if !active {
                let result = self
                    .controller
                    .top11_invoke(Top11Action::RingRail)
                    .and_then(|_| self.controller.set_ring_rail_size(&bar.system, bar.size));
                if result.is_err() {
                    bar.shapes = false;
                }
                self.feedback(result);
            }
            if bar.shapes {
                self.ring_rail_shape_ui(ui);
            }
        }
        if bar.reveal {
            ui.scroll_to_rect(controls, Some(egui::Align::BOTTOM));
            bar.reveal = false;
        }
        if update.is_some() || apply {
            let (system, size) = update.unwrap_or_else(|| (bar.system.clone(), bar.size));
            let result = (|| {
                if !self.controller.ring_rail_is_open() {
                    self.controller.top11_invoke(Top11Action::RingRail)?;
                }
                self.controller.set_ring_rail_size(&system, size)?;
                bar.system = system;
                bar.size = size;
                if apply {
                    self.controller.submit("Apply")?;
                    bar.placed = true;
                } else {
                    bar.placed = false;
                }
                Ok(())
            })();
            self.feedback(result);
        }
        self.builder_ui.rail_toolbar = Some(bar);
        true
    }
}
