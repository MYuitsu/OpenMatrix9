// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::file_lifecycle::{FileKind, FileOutcome, FilePhase, FileToken, UnsavedDecision};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileControl {
    Save,
    Discard,
    Cancel,
    Overwrite,
    Browse,
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    fn raw_close() -> egui::RawInput {
        let mut raw = egui::RawInput::default();
        raw.viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
        raw
    }
    #[test]
    fn minimized_logic_vetoes_dirty_close_and_retains_one_prompt_owner() {
        let mut app = MatrixApp::new(Document::new("Minimized")).unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("1,2,3").unwrap();
        let ctx = egui::Context::default();
        for _ in 0..2 {
            let output = ctx.run_logic(&raw_close(), |ctx| {
                app.service_window_close(ctx);
            });
            assert!(
                output.viewport_commands[&egui::ViewportId::ROOT]
                    .contains(&egui::ViewportCommand::CancelClose)
            );
        }
        let token = app.controller.file_operation().unwrap().token();
        let output = ctx.run_logic(&raw_close(), |ctx| {
            app.service_window_close(ctx);
        });
        assert_eq!(app.controller.file_operation().unwrap().token(), token);
        assert!(
            output.viewport_commands[&egui::ViewportId::ROOT]
                .contains(&egui::ViewportCommand::CancelClose)
        );
        app.controller
            .decide_unsaved(token, UnsavedDecision::Discard)
            .unwrap();
        let output = ctx.run_logic(&raw_close(), |ctx| {
            app.service_window_close(ctx);
        });
        assert!(
            output
                .viewport_commands
                .get(&egui::ViewportId::ROOT)
                .is_none_or(|commands| !commands.contains(&egui::ViewportCommand::CancelClose))
        );
    }
    #[test]
    fn minimized_logic_vetoes_stale_close_approval() {
        let mut app = MatrixApp::new(Document::new("Stale close")).unwrap();
        app.controller.begin_window_close().unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("1,2,3").unwrap();
        let ctx = egui::Context::default();
        let output = ctx.run_logic(&raw_close(), |ctx| {
            app.service_window_close(ctx);
        });
        assert!(
            output.viewport_commands[&egui::ViewportId::ROOT]
                .contains(&egui::ViewportCommand::CancelClose)
        );
        assert!(app.controller.is_dirty().unwrap());
    }
    #[test]
    fn real_adapter_error_keeps_file_owner_and_draft_and_requires_explicit_retry() {
        let mut app = MatrixApp::new(Document::new("Native error")).unwrap();
        let ctx = egui::Context::default();
        app.controller.submit("Point").unwrap();
        app.controller.submit("1,2,3").unwrap();
        app.controller.submit("Line").unwrap();
        app.controller.submit("0,0,0").unwrap();
        app.transcript.set_draft("12,6,");
        let before = app.controller.coordinator().snapshot();
        app.controller.begin_file_operation(FileKind::Save).unwrap();
        let frame = |app: &mut MatrixApp| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1280., 1000.),
                    )),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            output.textures_delta.clear();
        };
        for _ in 0..3 {
            frame(&mut app);
        }
        let token = app.file_picker_request.take().unwrap();
        let error = om9_native_owner::file_dialog::choose(
            0,
            om9_native_owner::file_dialog::Mode::Save,
            None,
        )
        .unwrap_err();
        assert!(
            app.file_feedback(
                &ctx,
                Err(Error::Io(format!("Windows file dialog: {error}")))
            )
            .is_err()
        );
        for _ in 0..10 {
            frame(&mut app);
            assert!(app.file_picker_request.is_none());
        }
        assert!(app.file_error.contains("owner HWND"));
        assert!(app.file_error.contains("0x80070057"));
        assert_eq!(app.controller.file_operation().unwrap().token(), token);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert!(app.controller.is_running());
        assert_eq!(app.transcript.draft(), "12,6,");
        assert!(app.file_control_rect(FileControl::Browse).is_some());
        assert!(app.file_control_rect(FileControl::Cancel).is_some());
    }
}
impl MatrixApp {
    /// Runs in both normal UI and eframe's minimized/occluded logic pass.
    /// Keep the approved operation alive until native shutdown, so a later
    /// document mutation cannot reuse consent captured for an older owner.
    pub(super) fn service_window_close(&mut self, ctx: &egui::Context) -> bool {
        let requested = ctx.input(|input| input.viewport().close_requested());
        let approved = self.controller.file_operation().is_some_and(|operation| {
            operation.phase() == FilePhase::CloseReady
                && self
                    .controller
                    .validate_file_operation(operation.token())
                    .is_ok()
        });
        if approved {
            if !requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return true;
        }
        if !requested {
            return false;
        }
        if self.controller.file_operation().is_none() && self.controller.layer_dialog().is_none() {
            let result = self.controller.begin_window_close();
            let clean_close = matches!(result, Ok(FileOutcome::CloseReady));
            let _ = self.file_feedback(ctx, result);
            if clean_close {
                return true;
            }
        }
        // Even a stale approval or another modal must veto this native request.
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.request_repaint();
        false
    }
    // Only explicit audit runs own throwaway documents. Teardown still uses
    // the same captured Close intent and explicit Discard decision.
    pub(super) fn audit_window_close(&mut self, ctx: &egui::Context) {
        if self.audit.is_none() {
            return;
        }
        if let Some(token) = self
            .controller
            .layer_dialog()
            .map(|operation| operation.token())
        {
            let _ = self.controller.cancel_layer_dialog(token);
        }
        if let Some(token) = self
            .controller
            .file_operation()
            .map(|operation| operation.token())
        {
            let _ = self.controller.cancel_file_operation(token);
        }
        if let Ok(FileOutcome::Pending) = self.controller.begin_window_close() {
            let token = self
                .controller
                .file_operation()
                .expect("close operation")
                .token();
            let _ = self
                .controller
                .decide_unsaved(token, UnsavedDecision::Discard);
        }
        self.service_window_close(ctx);
    }
    pub fn file_control_rect(&self, control: FileControl) -> Option<Rect> {
        self.file_controls.get(&control).copied()
    }
    pub fn file_toolbar_rect(&self, kind: FileKind) -> Option<Rect> {
        self.file_toolbar_rects.get(&kind).copied()
    }
    pub fn file_f6_rect(&self, kind: FileKind) -> Option<Rect> {
        self.file_f6_rects.get(&kind).copied()
    }
    pub fn file_menu_rect(&self) -> Option<Rect> {
        self.file_menu_rect
    }
    pub fn file_menu_item_rect(&self, kind: FileKind) -> Option<Rect> {
        self.file_menu_items.get(&kind).copied()
    }
    pub fn pending_file_picker(&self) -> Option<FileToken> {
        self.file_picker_request
    }
    pub(super) fn file_feedback(
        &mut self,
        ctx: &egui::Context,
        result: Result<FileOutcome>,
    ) -> Result<()> {
        self.viewport_menu_owns_frame = true;
        self.viewport_menu_pointer_guard = true;
        match result {
            Ok(outcome) => {
                self.file_error.clear();
                if matches!(
                    outcome,
                    FileOutcome::Saved
                        | FileOutcome::SavedPending
                        | FileOutcome::SavedAndReplaced
                        | FileOutcome::SavedAndCloseReady
                ) {
                    self.transcript.append_history("Document saved.");
                }
                if matches!(
                    outcome,
                    FileOutcome::Replaced | FileOutcome::SavedAndReplaced
                ) {
                    self.hover = None;
                    self.transcript.set_draft("");
                    self.transcript.set_prompt(self.controller.prompt());
                    self.view_menu_capture = None;
                    self.file_picker_request = None;
                    self.file_picker_attempted = None;
                    self.file_prompt_token = None;
                    self.f6_open = false;
                    egui::Popup::close_all(ctx);
                    ctx.data_mut(|data| {
                        data.remove::<egui::text_edit::TextEditState>(egui::Id::new(
                            "matrix-command-transcript",
                        ))
                    });
                    for diagnostic in self.controller.file_diagnostics() {
                        self.transcript.append_history(diagnostic);
                    }
                    if let Err(error) = self.reload_project_state() {
                        self.transcript
                            .append_history(&format!("project manager state: {error}"));
                    }
                }
                self.transcript.set_prompt(self.controller.prompt());
                self.focus_cmd = self.controller.file_operation().is_none();
                ctx.request_repaint();
                Ok(())
            }
            Err(error) => {
                self.file_error.clear();
                use std::fmt::Write;
                struct Bounded<'a>(&'a mut String);
                impl std::fmt::Write for Bounded<'_> {
                    fn write_str(&mut self, text: &str) -> std::fmt::Result {
                        for ch in text.chars() {
                            if self.0.len() + ch.len_utf8() > 512 {
                                break;
                            }
                            self.0.push(ch);
                        }
                        Ok(())
                    }
                }
                let _ = write!(Bounded(&mut self.file_error), "{error}");
                if self.controller.file_save_receipt().is_some() {
                    self.transcript
                        .append_history("Document saved; requested replacement was not accepted.");
                }
                self.feedback(Err(error.clone()));
                self.focus_cmd = self.controller.file_operation().is_none();
                Err(error)
            }
        }
    }
    /// Owned path response endpoint. Tests exercise this boundary separately
    /// from actual OS selection; it never supplies a raw native pointer.
    pub fn accept_file_picker_reply(
        &mut self,
        ctx: &egui::Context,
        token: FileToken,
        path: Option<PathBuf>,
    ) -> Result<()> {
        self.controller.validate_file_operation(token)?;
        self.file_picker_request = None;
        let Some(path) = path else {
            self.controller.cancel_file_operation(token)?;
            self.file_error.clear();
            self.focus_cmd = true;
            ctx.request_repaint();
            return Ok(());
        };
        let result = self.controller.accept_file_path(token, &path);
        self.file_feedback(ctx, result)
    }
    pub(super) fn file_ui(&mut self, ctx: &egui::Context) {
        self.file_controls.clear();
        let Some(operation) = self.controller.file_operation() else {
            self.file_prompt_token = None;
            self.file_picker_request = None;
            return;
        };
        let token = operation.token();
        let phase = operation.phase();
        if phase == FilePhase::CloseReady {
            self.service_window_close(ctx);
            return;
        }
        let initial = self.file_prompt_token != Some(token);
        let mut painted = false;
        let affirmable = !initial
            && !self.file_ime_composing
            && !ctx.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Ime(_)))
            })
            && !ctx.input(|input| {
                input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::Key {
                            key: Key::Enter | Key::Space,
                            pressed: true,
                            repeat: true,
                            ..
                        }
                    )
                })
            });
        let mut decision = None;
        let mut cancel = false;
        let mut overwrite = None;
        let mut browse = false;
        egui::Modal::new(egui::Id::new("owned-file-operation"))
            .frame(egui::Frame::popup(&ctx.style_of(egui::Theme::Light)))
            .show(ctx, |ui| {
                painted = !ui.is_sizing_pass();
                *ui.visuals_mut() = egui::Visuals::light();
                ui.set_width(360.);
                ui.heading(match phase {
                    FilePhase::Unsaved => "Save changes?",
                    FilePhase::Overwrite => "Replace existing file?",
                    FilePhase::OpenPath => "Open document",
                    FilePhase::SavePath => "Save document",
                    FilePhase::CloseReady => unreachable!("handled before modal"),
                });
                ui.separator();
                match phase {
                    FilePhase::Unsaved => {
                        ui.label("Save the current document before continuing?");
                    }
                    FilePhase::Overwrite => {
                        ui.label("The selected file already exists.");
                        if let Some(path) = self
                            .controller
                            .file_operation()
                            .expect("operation")
                            .destination()
                        {
                            ui.add(egui::Label::new(path.to_string_lossy()).truncate());
                        }
                    }
                    FilePhase::OpenPath | FilePhase::SavePath => {
                        ui.label("Native OM9 document (*.om9)");
                        ui.weak("For Rhino archives use File > Import 3DM… / Export 3DM….");
                    }
                    FilePhase::CloseReady => unreachable!("handled before modal"),
                }
                if !self.file_error.is_empty() {
                    ui.colored_label(Color32::DARK_RED, &self.file_error);
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.push_id((token.authority(), token.serial()), |ui| {
                        if phase == FilePhase::Unsaved {
                            let response = ui.button("Save");
                            self.file_controls.insert(FileControl::Save, response.rect);
                            if initial && painted {
                                response.request_focus();
                            }
                            if response.clicked() && affirmable {
                                decision = Some(UnsavedDecision::Save);
                            }
                            let response = ui.button("Discard");
                            self.file_controls
                                .insert(FileControl::Discard, response.rect);
                            if response.clicked() && affirmable {
                                decision = Some(UnsavedDecision::Discard);
                            }
                        } else if phase == FilePhase::Overwrite {
                            let response = ui.button("Replace");
                            self.file_controls
                                .insert(FileControl::Overwrite, response.rect);
                            if response.clicked() && affirmable {
                                overwrite = Some(true);
                            }
                        } else {
                            let response = ui.button("Browse…");
                            self.file_controls
                                .insert(FileControl::Browse, response.rect);
                            browse = response.clicked() && affirmable;
                        }
                        let response = ui.button("Cancel");
                        self.file_controls
                            .insert(FileControl::Cancel, response.rect);
                        cancel = response.clicked() && affirmable;
                    });
                });
            });
        if painted {
            self.file_prompt_token = Some(token);
        }
        self.focus_cmd = false;
        self.viewport_menu_owns_frame = true;
        if cancel {
            let result = self.controller.cancel_file_operation(token);
            self.file_picker_request = None;
            self.feedback(result);
        } else if let Some(decision) = decision {
            let result = self.controller.decide_unsaved(token, decision);
            let _ = self.file_feedback(ctx, result);
        } else if let Some(yes) = overwrite {
            let result = self.controller.confirm_file_overwrite(token, yes);
            let _ = self.file_feedback(ctx, result);
        } else if painted
            && matches!(phase, FilePhase::OpenPath | FilePhase::SavePath)
            && (browse || (self.file_error.is_empty() && self.file_picker_attempted != Some(token)))
        {
            self.file_error.clear();
            self.file_picker_request = Some(token);
            self.file_picker_attempted = Some(token);
            ctx.request_repaint();
        }
    }
    /// Open the native 3DM picker and run the host adapter. Import commits one transaction;
    /// export only reads the current document. The dialog runs synchronously on the UI thread.
    pub(super) fn service_three_dm_picker(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(export) = self.three_dm_request.take() else {
            return;
        };
        let current = if export {
            self.controller
                .active_file_path()
                .map(std::path::Path::to_path_buf)
        } else {
            None
        };
        match crate::native_picker::pick_3dm(frame, export, current.as_deref()) {
            Ok(Some(path)) => {
                let result = if export {
                    self.controller.export_3dm(&path).map(|diagnostics| {
                        (
                            format!(
                                "Exported Rhino 3DM: {}",
                                crate::transcript::redact_paths(&path.display().to_string())
                            ),
                            diagnostics,
                        )
                    })
                } else {
                    self.controller.import_3dm(&path).map(|diagnostics| {
                        (
                            format!(
                                "Imported Rhino 3DM: {}",
                                crate::transcript::redact_paths(&path.display().to_string())
                            ),
                            diagnostics,
                        )
                    })
                };
                match result {
                    Ok((message, diagnostics)) => {
                        self.transcript.append_history(&message);
                        for diagnostic in &diagnostics {
                            self.transcript.append_history(diagnostic);
                        }
                        if let Some(last) = diagnostics.last() {
                            self.status.warning(last);
                        } else {
                            self.status.complete_success();
                        }
                    }
                    Err(error) => {
                        let message = crate::transcript::redact_paths(&format!("{error}"));
                        self.transcript.append_history(&message);
                        self.status.error(&message);
                    }
                }
                self.focus_cmd = true;
                ctx.request_repaint();
            }
            Ok(None) => {}
            Err(error) => {
                let message = crate::transcript::redact_paths(&format!("{error}"));
                self.transcript.append_history(&message);
                self.status.error(&message);
                self.focus_cmd = true;
                ctx.request_repaint();
            }
        }
    }

    pub(super) fn update_file_title(&mut self, ctx: &egui::Context) {
        let name = self
            .controller
            .active_file_path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| {
                std::borrow::Cow::Borrowed(self.controller.document().name.as_str())
            });
        let mut title = String::with_capacity(512);
        title.push_str("OpenMatrix9 — ");
        for ch in name.chars() {
            if title.len() + ch.len_utf8() > 480 {
                break;
            }
            title.push(ch);
        }
        if self.controller.is_dirty().unwrap_or(true) {
            title.push_str(" *");
        }
        if title != self.window_title {
            self.window_title = title;
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title.clone()));
        }
    }
    pub(super) fn service_native_file_picker(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
    ) {
        let Some(token) = self.file_picker_request.take() else {
            return;
        };
        if self.controller.validate_file_operation(token).is_err() {
            return;
        }
        let phase = self.controller.file_operation().expect("operation").phase();
        #[cfg(windows)]
        if let Some(directory) = self
            .audit
            .as_ref()
            .filter(|audit| audit.file_success)
            .map(|audit| audit.directory.clone())
        {
            let filename = if self
                .audit
                .as_ref()
                .expect("audit")
                .file_picker_receipts
                .is_empty()
            {
                "selected.om9"
            } else {
                "alternate.om9"
            };
            let fixture = directory.join(filename);
            match crate::native_picker::pick_fixture_for_audit(frame, phase, &fixture) {
                Ok((path, receipt)) => {
                    self.audit
                        .as_mut()
                        .expect("audit")
                        .file_picker_receipts
                        .push(receipt);
                    let _ = self.accept_file_picker_reply(ctx, token, path);
                }
                Err(error) => {
                    let _ = self.file_feedback(ctx, Err(error));
                }
            }
            return;
        }
        #[cfg(windows)]
        let probe = if self.audit.as_ref().is_some_and(|audit| audit.files) {
            match crate::native_picker::arm_cancel_probe(frame, phase) {
                Ok(probe) => Some(probe),
                Err(error) => {
                    let _ = self.file_feedback(ctx, Err(error));
                    return;
                }
            }
        } else {
            None
        };
        let result = crate::native_picker::pick(frame, phase, self.controller.active_file_path());
        #[cfg(windows)]
        if let Some(probe) = probe {
            let receipt = match probe.finish() {
                Ok(receipt) => {
                    serde_json::json!({"pid":receipt.pid,"ui_thread":receipt.thread,"observed":receipt.observed,"close_posted":receipt.close_posted,"destroyed":receipt.destroyed,"elapsed_ms":receipt.elapsed_ms,"returned_cancel":matches!(result,Ok(None)),"phase":format!("{phase:?}"),"boundary":"same process/thread, direct owner, #32770 and exact title; WM_CLOSE; joined probe"})
                }
                Err(error) => serde_json::json!({"error":error.to_string()}),
            };
            self.audit
                .as_mut()
                .expect("audit")
                .file_picker_receipts
                .push(receipt);
        }
        match result {
            Ok(path) => {
                let _ = self.accept_file_picker_reply(ctx, token, path);
            }
            Err(error) => {
                let _ = self.file_feedback(ctx, Err(error));
            }
        }
    }
}
