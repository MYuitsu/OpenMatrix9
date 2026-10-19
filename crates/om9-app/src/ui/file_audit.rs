// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in events in our native window. Real OS picker cancellation is measured
//! separately; no scripted path is advertised as OS selection.
use super::*;
use crate::file_lifecycle::{FileKind, FilePhase};
impl MatrixApp {
    pub(super) fn file_audit_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let audit = self.audit.as_ref().expect("audit");
        if audit.awaiting_screenshot || audit.receipt_written {
            ctx.request_repaint();
            return;
        }
        if audit.file_close && audit.step >= 52 {
            self.file_close_audit_input(ctx, raw);
            return;
        }
        let step = audit.step;
        self.audit.as_mut().expect("audit").step += 1;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let rect = match step {
            11 | 12 => self.file_toolbar_rect(FileKind::New),
            16 | 17 => self.file_toolbar_rect(FileKind::Save),
            20 | 21 | 37 | 38 => self.file_menu_rect(),
            23 | 24 | 40 | 41 => self.file_menu_item_rect(FileKind::Open),
            26 | 27 => self.file_control_rect(FileControl::Cancel),
            31 | 32 => self.file_f6_rect(FileKind::New),
            34 | 35 => self.file_control_rect(FileControl::Discard),
            _ => None,
        };
        if let Some(rect) = rect {
            let pressed = matches!(step, 11 | 16 | 20 | 23 | 26 | 31 | 34 | 37 | 40);
            let pos = rect.center();
            raw.events.push(egui::Event::PointerMoved(pos));
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
            self.audit.as_mut().expect("audit").viewport_replay.push(serde_json::json!({"step":step,"pointer_down":pressed,"rect":[rect.min.x,rect.min.y,rect.max.x,rect.max.y]}));
        }
        let text = match step {
            0 | 6 => Some("Line"),
            2 | 8 => Some("0,0,0"),
            4 => Some("12,6,0"),
            10 => Some("12,6,"),
            44 => Some("Point"),
            46 => Some("1,2,3"),
            48 => Some("SaveAs"),
            _ => None,
        };
        if let Some(text) = text {
            raw.events.push(egui::Event::Text(text.into()));
        }
        match step {
            1 | 3 | 5 | 7 | 9 | 45 | 47 | 49 => raw.events.push(key(Key::Enter)),
            14 => raw.events.push(key(Key::Escape)),
            29 => raw.events.push(key(Key::F6)),
            52 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.audit.as_mut().expect("audit").awaiting_screenshot = true;
            }
            _ => {}
        }
        let intact = self.controller.document().entities.len() == 1
            && self.controller.coordinator().undo_len() == 1
            && self.controller.is_running()
            && self.transcript.draft() == "12,6,";
        let check = match step {
            13 if intact
                && self
                    .controller
                    .file_operation()
                    .is_some_and(|op| op.phase() == FilePhase::Unsaved) =>
            {
                Some("native New toolbar opens owned prompt preserves Line")
            }
            15 if intact && self.controller.file_operation().is_none() => {
                Some("native prompt Escape retains old document and draft")
            }
            19 if intact
                && self.controller.file_operation().is_none()
                && self.controller.active_file_path().is_none() =>
            {
                Some("real Windows Save chooser Cancel preserves pending Line")
            }
            28 if intact && self.controller.file_operation().is_none() => {
                Some("native File menu Open Cancel preserves pending Line")
            }
            33 if intact
                && self
                    .controller
                    .file_operation()
                    .is_some_and(|op| op.kind() == Some(FileKind::New)) =>
            {
                Some("native F6 New uses same owned prompt")
            }
            36 if self.controller.document().entities.is_empty()
                && self.controller.coordinator().undo_len() == 0
                && !self.controller.is_running()
                && self.transcript.draft().is_empty() =>
            {
                Some("native Discard replaces session only after button")
            }
            43 if self.controller.document().entities.is_empty()
                && self.controller.file_operation().is_none()
                && self.controller.active_file_path().is_none() =>
            {
                Some("real Windows Open chooser Cancel keeps fresh document")
            }
            51 if self.controller.document().entities.len() == 1
                && self.controller.coordinator().undo_len() == 1
                && self.controller.file_operation().is_none()
                && self.controller.active_file_path().is_none() =>
            {
                Some("native CMD SaveAs uses real Windows chooser Cancel")
            }
            _ => None,
        };
        if let Some(check) = check {
            self.audit
                .as_mut()
                .expect("audit")
                .checks
                .push(check.into());
        }
        let audit = self.audit.as_ref().expect("audit");
        let _=std::fs::write(audit.directory.join("file-replay.json"),serde_json::to_vec_pretty(&serde_json::json!({"step":step,"checks":audit.checks,"picker":audit.file_picker_receipts,"draft":self.transcript.draft(),"phase":self.controller.file_operation().map(|op|format!("{:?}",op.phase()))})).expect("trace"));
        ctx.request_repaint();
    }
    pub(super) fn service_file_close_probe(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(audit) = self
            .audit
            .as_mut()
            .filter(|audit| audit.file_close && audit.file_close_post_pending)
        else {
            return;
        };
        audit.file_close_post_pending = false;
        #[cfg(windows)]
        let result = {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            frame
                .window_handle()
                .map_err(|error| error.to_string())
                .and_then(|handle| {
                    if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                        om9_native_owner::dialog_probe::post_owned_root_close(
                            handle.hwnd.get() as usize
                        )
                        .map_err(|error| error.to_string())
                    } else {
                        Err("Win32 root required".into())
                    }
                })
        };
        #[cfg(not(windows))]
        let result: std::result::Result<(), String> = {
            let _ = frame;
            Err("Win32 root required".into())
        };
        if let Err(error) = result {
            audit
                .checks
                .push(format!("FAILED native close request: {error}"));
            audit.awaiting_screenshot = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint();
    }
    fn file_close_audit_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let step = self.audit.as_ref().expect("audit").step;
        if raw.viewport().close_requested() {
            self.audit.as_mut().expect("audit").file_close_events += 1;
        }
        let audit = self.audit.as_ref().expect("audit");
        let expected_events = if step <= 56 { 1 } else { 2 };
        let prompt_ready = self
            .controller
            .file_operation()
            .is_some_and(|op| op.phase() == FilePhase::Unsaved)
            && audit.file_close_events >= expected_events
            && self.file_control_rect(FileControl::Cancel).is_some();
        let chooser_returned =
            self.controller.file_operation().is_none() && audit.file_picker_receipts.len() == 4;
        if (matches!(step, 53 | 58) && !prompt_ready) || (step == 61 && !chooser_returned) {
            if audit.started.elapsed().as_secs() > 20 {
                self.audit
                    .as_mut()
                    .expect("audit")
                    .checks
                    .push("FAILED native WM_CLOSE prompt/chooser readiness".into());
                self.audit.as_mut().expect("audit").awaiting_screenshot = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.audit.as_mut().expect("audit").step += 1;
        let rect = match step {
            54 | 55 => self.file_control_rect(FileControl::Cancel),
            59 | 60 => self.file_control_rect(FileControl::Save),
            _ => None,
        };
        if let Some(rect) = rect {
            let pressed = matches!(step, 54 | 59);
            let pos = rect.center();
            raw.events.push(egui::Event::PointerMoved(pos));
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
            self.audit.as_mut().expect("audit").viewport_replay.push(serde_json::json!({"step":step,"pointer_down":pressed,"rect":[rect.min.x,rect.min.y,rect.max.x,rect.max.y]}));
        }
        let intact = self.controller.document().entities.len() == 1
            && self.controller.coordinator().undo_len() == 1
            && self.controller.is_dirty().is_ok_and(|dirty| dirty)
            && self.controller.active_file_path().is_none();
        let check = match step {
            53 if prompt_ready && intact => Some("real WM_CLOSE opens owned dirty prompt"),
            56 if intact && self.controller.file_operation().is_none() => {
                Some("native close Cancel keeps unsaved document and window")
            }
            61 if intact
                && self.controller.file_operation().is_none()
                && self
                    .audit
                    .as_ref()
                    .expect("audit")
                    .file_picker_receipts
                    .len()
                    == 4 =>
            {
                Some("native close Save then real Windows chooser Cancel keeps window")
            }
            _ => None,
        };
        if let Some(check) = check {
            self.audit
                .as_mut()
                .expect("audit")
                .checks
                .push(check.into());
        }
        match step {
            52 | 57 => self.audit.as_mut().expect("audit").file_close_post_pending = true,
            62 => {
                self.audit.as_mut().expect("audit").awaiting_screenshot = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            _ => {}
        }
        let audit = self.audit.as_ref().expect("audit");
        let _=std::fs::write(audit.directory.join("file-replay.json"),serde_json::to_vec_pretty(&serde_json::json!({"step":step,"checks":audit.checks,"native_close_events":audit.file_close_events,"picker":audit.file_picker_receipts,"phase":self.controller.file_operation().map(|op|format!("{:?}",op.phase()))})).expect("trace"));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
