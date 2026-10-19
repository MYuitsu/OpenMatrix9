// SPDX-License-Identifier: LGPL-2.1-or-later
use super::MatrixApp;
use crate::viewport_menu::{ENTRIES, MenuAction};
use eframe::egui::{self, Response};
use om9_scene::workspace::ViewId;
impl MatrixApp {
    pub(super) fn viewport_title_popup(
        &mut self,
        ui: &mut egui::Ui,
        view: ViewId,
        response: &Response,
    ) {
        let popup = egui::Popup::menu(response).width(236.).frame(
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgb(248, 248, 248))
                .stroke(egui::Stroke::new(1., egui::Color32::from_gray(160))),
        );
        let popup_id = popup.get_id();
        let opening = response.clicked();
        if response.clicked() {
            match self.controller.capture_view_menu(view) {
                Ok(capture) => self.view_menu_capture = Some((popup_id, capture)),
                Err(error) => {
                    self.feedback(Err(error));
                    egui::Popup::close_id(ui.ctx(), popup_id);
                    return;
                }
            }
        }
        let capture = self
            .view_menu_capture
            .as_ref()
            .filter(|(id, _)| *id == popup_id)
            .map(|(_, capture)| capture.clone());
        popup.show(|ui| {
            *ui.visuals_mut() = egui::Visuals::light();
            ui.set_min_width(236.);
            ui.set_max_width(236.);
            self.viewport_menu_owns_frame = true;
            self.focus_cmd = false;
            let maximum = (ui.ctx().content_rect().height() - 32.).max(48.);
            // Popup Area's default height is only an initial sizing hint. Let
            // this long reference menu use the work area, then scroll/clamp.
            ui.set_max_height(maximum);
            egui::ScrollArea::vertical()
                .id_salt(("viewport-title-menu", view.title()))
                .auto_shrink([false, true])
                .max_height(maximum)
                .show(ui, |ui| {
                    let mut focus_ids = Vec::new();
                    for entry in ENTRIES {
                        let Some(action) = entry.action else {
                            ui.separator();
                            continue;
                        };
                        let availability = action.availability();
                        let enabled = availability.is_ok() && capture.is_some();
                        let mode = self.controller.workspace.views.get(&view).map(|v| v.mode);
                        let response = if matches!(action, MenuAction::Preset(_)) {
                            ui.add_enabled(
                                enabled,
                                egui::RadioButton::new(
                                    mode.is_some_and(|m| action.is_checked(m)),
                                    entry.label,
                                ),
                            )
                        } else {
                            let button = egui::Button::new(entry.label);
                            let button = if entry.submenu {
                                button.right_text("›")
                            } else {
                                button
                            };
                            ui.add_enabled(enabled, button)
                        };
                        let response = if let Err(error) = availability {
                            response.on_disabled_hover_text(error.to_string())
                        } else {
                            response
                        };
                        if enabled {
                            if opening && focus_ids.is_empty() {
                                response.request_focus();
                            }
                            ui.ctx().memory_mut(|m| {
                                m.set_focus_lock_filter(
                                    response.id,
                                    egui::EventFilter {
                                        tab: true,
                                        horizontal_arrows: true,
                                        vertical_arrows: true,
                                        ..Default::default()
                                    },
                                )
                            });
                            focus_ids.push(response.id);
                        }
                        if response.clicked()
                            && let Some(capture) = &capture
                        {
                            let result = self.controller.apply_view_menu(capture, action);
                            self.feedback(result);
                            ui.close();
                        }
                    }
                    let direction = ui.input_mut(|i| {
                        if i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab)
                            || i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                        {
                            -1
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                            || i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                        {
                            1
                        } else {
                            0
                        }
                    });
                    if !focus_ids.is_empty() && direction != 0 {
                        let current = ui
                            .ctx()
                            .memory(|m| m.focused())
                            .and_then(|id| focus_ids.iter().position(|item| *item == id))
                            .unwrap_or(0);
                        let next = (current as isize + direction)
                            .rem_euclid(focus_ids.len() as isize)
                            as usize;
                        ui.ctx().memory_mut(|m| m.request_focus(focus_ids[next]));
                    }
                });
        });
    }
}
