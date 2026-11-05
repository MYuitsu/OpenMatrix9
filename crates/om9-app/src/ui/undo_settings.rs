// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_core::transaction::UndoPolicy;

impl MatrixApp {
    pub(super) fn open_undo_settings_command(&mut self, text: &str) -> bool {
        if !["Options", "RhinoOptions", "UndoSettings"]
            .iter()
            .any(|s| text.trim().eq_ignore_ascii_case(s))
        {
            return false;
        }
        self.undo_settings_draft = Some(self.controller.coordinator().undo_policy());
        self.focus_cmd = false;
        true
    }

    pub fn load_undo_settings(&mut self, path: &std::path::Path) -> Result<()> {
        self.undo_settings_path = Some(path.to_path_buf());
        if !path.exists() {
            return Ok(());
        }
        if std::fs::metadata(path)
            .map_err(|e| Error::Io(e.to_string()))?
            .len()
            > 4096
        {
            return Err(Error::Budget("Undo preferences file"));
        }
        let bytes = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
        let policy: UndoPolicy =
            serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("Undo preferences"))?;
        self.controller.set_undo_policy(policy)
    }

    fn save_undo_settings(&self, policy: UndoPolicy) -> Result<()> {
        let Some(path) = &self.undo_settings_path else {
            return Ok(());
        };
        let bytes = serde_json::to_vec(&policy).map_err(|_| Error::Invalid("Undo preferences"))?;
        let parent = path
            .parent()
            .ok_or(Error::Invalid("Undo preferences path"))?;
        std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
        let staging = parent.join(format!(".undo-{}.tmp", uuid::Uuid::new_v4()));
        std::fs::write(&staging, bytes).map_err(|e| Error::Io(e.to_string()))?;
        if let Err(error) = std::fs::rename(&staging, path) {
            let _ = std::fs::remove_file(&staging);
            return Err(Error::Io(error.to_string()));
        }
        Ok(())
    }

    pub(super) fn undo_settings_ui(&mut self, ctx: &egui::Context) {
        let Some(mut draft) = self.undo_settings_draft else {
            return;
        };
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let max_mb = self.controller.coordinator().undo_ram_limit_mb();
        egui::Window::new("Options — General")
            .id(egui::Id::new("undo-settings"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Undo");
                egui::Grid::new("undo-policy-fields").show(ui, |ui| {
                    ui.label("Min number of undos:");
                    let r =
                        ui.add(egui::DragValue::new(&mut draft.minimum_steps).range(1..=1_000_000));
                    ctx.data_mut(|m| m.insert_temp(egui::Id::new("undo-minimum-rect"), r.rect));
                    ui.end_row();
                    ui.label("Max memory used (MB):");
                    let r = ui.add(
                        egui::DragValue::new(&mut draft.max_memory_mb)
                            .range(1..=max_mb.unwrap_or(1024)),
                    );
                    ctx.data_mut(|m| m.insert_temp(egui::Id::new("undo-memory-rect"), r.rect));
                    ui.end_row();
                });
                ui.label("Oldest steps are removed over the memory budget; the minimum is kept.");
                if let Some(max) = max_mb {
                    ui.label(format!(
                        "Maximum configurable: {max} MB (50% of physical RAM)."
                    ));
                } else {
                    ui.colored_label(
                        Color32::YELLOW,
                        "Physical RAM capacity unavailable; the 50% limit cannot be verified.",
                    );
                }
                let c = self.controller.coordinator();
                {
                    let stats = c.undo_memory_stats();
                    let bytes = stats.estimated_bytes;
                    ui.label(format!(
                        "Undo: {}   Redo: {}   Estimated history memory: {:.1} MB",
                        c.undo_len(),
                        c.redo_len(),
                        bytes as f64 / 1_048_576.
                    ));
                    if stats.capture_change_pending {
                        ui.label("History memory estimate awaits capture-owner reconciliation.");
                    }
                }
                ui.label(
                    "Full memory inventory every 100 MB of growth, or earlier near the budget.",
                );
                ui.horizontal(|ui| {
                    let r = ui.add_enabled(!self.controller.is_running(), egui::Button::new("OK"));
                    apply = r.clicked();
                    ctx.data_mut(|m| m.insert_temp(egui::Id::new("undo-apply-rect"), r.rect));
                    cancel = ui.button("Cancel").clicked();
                });
            });
        if apply {
            let result = self
                .controller
                .set_undo_policy(draft)
                .and_then(|_| self.save_undo_settings(draft));
            if result.is_ok() {
                open = false;
            }
            self.feedback(result);
        }
        self.undo_settings_draft = (open && !cancel).then_some(draft);
        if self.undo_settings_draft.is_none() {
            self.focus_cmd = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn saved_budget_above_half_ram_is_rejected_without_changing_active_policy() {
        let mut app = MatrixApp::new(Document::new("RAM cap")).unwrap();
        let max = app
            .controller
            .coordinator()
            .undo_ram_limit_mb()
            .expect("Windows RAM query");
        let current = app.controller.coordinator().undo_policy();
        let path = std::env::temp_dir().join(format!("om9-undo-cap-{}.json", uuid::Uuid::new_v4()));
        let excessive = UndoPolicy {
            minimum_steps: 1,
            max_memory_mb: max + 1,
        };
        std::fs::write(&path, serde_json::to_vec(&excessive).unwrap()).unwrap();
        assert!(app.load_undo_settings(&path).is_err());
        assert_eq!(app.controller.coordinator().undo_policy(), current);
        std::fs::remove_file(path).unwrap();
    }
    fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(900., 600.))),
                focused: true,
                events,
                ..Default::default()
            },
            |ui| {
                app.undo_settings_ui(ui.ctx());
            },
        );
        out.textures_delta.clear();
    }
    #[test]
    fn options_apply_by_click_persists_without_editing_document_and_rejects_corruption() {
        let path = std::env::temp_dir().join(format!("om9-undo-{}.json", uuid::Uuid::new_v4()));
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("Undo settings")).unwrap();
        app.load_undo_settings(&path).unwrap();
        let before = app.controller.coordinator().snapshot();
        app.transcript.set_draft("Options");
        app.submit_draft();
        assert_eq!(app.undo_settings_draft, Some(UndoPolicy::default()));
        let policy = UndoPolicy {
            minimum_steps: 7,
            max_memory_mb: 64,
        };
        app.undo_settings_draft = Some(policy);
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        let pos = ctx
            .data(|m| m.get_temp::<Rect>(egui::Id::new("undo-apply-rect")))
            .unwrap()
            .center();
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(app.undo_settings_draft.is_none());
        assert_eq!(app.controller.coordinator().undo_policy(), policy);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        let mut reopened = MatrixApp::new(Document::new("reopened")).unwrap();
        reopened.load_undo_settings(&path).unwrap();
        assert_eq!(reopened.controller.coordinator().undo_policy(), policy);
        std::fs::write(&path, b"{bad json").unwrap();
        assert!(reopened.load_undo_settings(&path).is_err());
        assert_eq!(reopened.controller.coordinator().undo_policy(), policy);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn options_sidebar_icon_opens_the_same_policy_and_close_does_not_apply() {
        let ctx = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("Options icon")).unwrap();
        app.icon_column = IconColumn::Left;
        let mut pos = egui::Pos2::ZERO;
        for step in 0..4 {
            app.rail_icon_rects.clear();
            let events = if step == 1 || step == 2 {
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: step == 1,
                        modifiers: Modifiers::NONE,
                    },
                ]
            } else {
                vec![]
            };
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(400., 200.))),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| app.fixed_sidebar_palette(ui, SectionId::InfoSettings),
            );
            out.textures_delta.clear();
            if step == 0 {
                pos = app.rail_icon_rects[0].center();
            }
        }
        assert_eq!(app.undo_settings_draft, Some(UndoPolicy::default()));
        app.undo_settings_draft = None;
        assert_eq!(
            app.controller.coordinator().undo_policy(),
            UndoPolicy::default()
        );
    }
    #[test]
    fn undo_preferences_survive_new_and_master_restore() {
        let mut c = Controller::new(Document::new("before"), Limits::default()).unwrap();
        let policy = UndoPolicy {
            minimum_steps: 5,
            max_memory_mb: 32,
        };
        c.set_undo_policy(policy).unwrap();
        c.restore_master_document(Document::new("restored"))
            .unwrap();
        assert_eq!(c.coordinator().undo_policy(), policy);
        c.begin_file_operation(crate::file_lifecycle::FileKind::New)
            .unwrap();
        assert_eq!(c.coordinator().undo_policy(), policy);
    }
}
