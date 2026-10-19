// SPDX-License-Identifier: LGPL-2.1-or-later
//! Observe native --ui path startup without injecting any editor/widget input.
use super::*;

impl MatrixApp {
    pub(super) fn startup_file_audit_input(&mut self, ctx: &egui::Context) {
        let audit = self.audit.as_ref().expect("audit");
        if audit.awaiting_screenshot || audit.receipt_written {
            ctx.request_repaint();
            return;
        }
        let path = self.controller.active_file_path();
        let title_matches = path
            .and_then(|path| path.file_name())
            .is_some_and(|name| self.window_title.contains(name.to_string_lossy().as_ref()));
        let views_ready = self.view_rects.len() == 4;
        if path.is_none() || !title_matches || !views_ready {
            if audit.started.elapsed() > std::time::Duration::from_secs(5) {
                let audit = self.audit.as_mut().expect("audit");
                audit
                    .checks
                    .push("FAILED native startup ownership/title/views".into());
                audit.awaiting_screenshot = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let clean = self.controller.is_dirty().is_ok_and(|dirty| !dirty);
        let diagnostics = self.controller.file_diagnostics();
        let diagnostics_visible = diagnostics
            .iter()
            .all(|message| self.transcript.text().contains(message));
        let audit = self.audit.as_mut().expect("audit");
        audit
            .checks
            .push("native --ui retains accepted path and window title".into());
        audit
            .checks
            .push("native --ui restores document before four viewport rendering".into());
        if diagnostics_visible && (clean || !diagnostics.is_empty()) {
            audit.checks.push(
                "native --ui retains explicit restoration diagnostics and clean state".into(),
            );
        } else {
            audit
                .checks
                .push("FAILED native startup diagnostics/dirty state".into());
        }
        audit.awaiting_screenshot = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        ctx.request_repaint();
    }
}
