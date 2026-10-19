// SPDX-License-Identifier: LGPL-2.1-or-later
//! Explicit opt-in replay through the application's real input/widgets.
use super::*;
impl MatrixApp {
    pub(super) fn viewport_menu_audit_input(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
    ) {
        let audit = self.audit.as_mut().expect("opt-in audit");
        let step = audit.step;
        if matches!(step, 8 | 11 | 19) && audit.viewport_idle_step != Some(step) {
            // A tall popup overlaps its anchor after clamping. Egui retains
            // its last painted Area for hit testing until one closed frame
            // has rendered. Let that frame finish before the next gesture.
            audit.viewport_idle_step = Some(step);
            ctx.request_repaint();
            return;
        }
        audit.step += 1;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let arrow = self
            .viewport_title_arrows
            .get(&ViewId::LookingDown)
            .map(|r| r.center());
        if step <= 21 {
            audit.viewport_replay.push(serde_json::json!({
                "step":step, "popup":egui::Popup::is_any_open(ctx),
                "maximized":format!("{:?}",self.controller.workspace.maximized),
                "arrow":arrow.map(|p|[p.x,p.y]),
                "focused":format!("{:?}",ctx.memory(|m|m.focused()))
            }));
        }
        let outside = self
            .view_rects
            .get(&ViewId::Perspective)
            .map(|r| r.center());
        let pointer = |raw: &mut egui::RawInput, pos: Option<egui::Pos2>, pressed| {
            if let Some(pos) = pos {
                raw.events.push(egui::Event::PointerMoved(pos));
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                });
            }
        };
        let preserved = self.controller.document().entities.is_empty()
            && self.controller.is_running()
            && self.controller.coordinator().undo_len() == 0
            && self.transcript.draft() == "1,2,";
        match step {
            0 => raw.events.push(egui::Event::Text("Point".into())),
            1 => raw.events.push(key(Key::Enter)),
            2 => raw.events.push(egui::Event::Text("1,2,".into())),
            3 | 8 | 11 | 15 | 19 => pointer(raw, arrow, true),
            4 | 9 | 12 | 16 | 20 => pointer(raw, arrow, false),
            5 => {
                if preserved && egui::Popup::is_any_open(ctx) {
                    audit
                        .checks
                        .push("native title popup preserves numeric draft".into());
                    audit.viewport_views = Some(self.controller.workspace.views.clone());
                }
                raw.events.push(key(Key::Tab));
            }
            6 => raw.events.push(key(Key::ArrowUp)),
            7 | 10 => raw.events.push(key(Key::Enter)),
            13 => raw.events.push(key(Key::Escape)),
            14 => {
                if preserved && !egui::Popup::is_any_open(ctx) {
                    audit
                        .checks
                        .push("native Escape preserves pending command".into());
                }
            }
            17 => pointer(raw, outside, true),
            18 => pointer(raw, outside, false),
            21 => {
                if preserved && egui::Popup::is_any_open(ctx) {
                    audit
                        .checks
                        .push("native popup reopened after dismissal".into());
                }
                // Native frames can replay faster than the popup's animation.
                // Capture after it settles, rather than an invisible sizing/
                // opening frame that cannot prove the rendered labels.
                audit.viewport_popup_ready_at = Some(ctx.input(|i| i.time) + 0.3);
            }
            _ => {}
        }
        if step > 21
            && !audit.awaiting_screenshot
            && audit
                .viewport_popup_ready_at
                .is_some_and(|ready| ctx.input(|i| i.time) >= ready)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            audit.awaiting_screenshot = true;
        }
        if step == 8
            && preserved
            && self.controller.workspace.maximized == Some(ViewId::LookingDown)
        {
            audit
                .checks
                .push("native keyboard Maximize preserves model and draft".into());
        }
        if step == 11
            && preserved
            && self.controller.workspace.maximized.is_none()
            && audit.viewport_views.as_ref() == Some(&self.controller.workspace.views)
        {
            audit
                .checks
                .push("native keyboard Restore preserves all view states".into());
        }
        if step == 19 && preserved && !egui::Popup::is_any_open(ctx) {
            audit
                .checks
                .push("native outside click only dismisses menu".into());
        }
        ctx.request_repaint();
    }
}
