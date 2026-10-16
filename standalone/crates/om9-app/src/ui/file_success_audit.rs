// SPDX-License-Identifier: LGPL-2.1-or-later
//! Explicit own-window audit: seeded native chooser, actual returned selection,
//! normal owned overwrite widget, fresh New and native Open.
use super::*;
use crate::file_lifecycle::{FileKind, FilePhase};
use om9_archive::Archive;

fn fixture_digest(path: &std::path::Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0; 8192];
    loop {
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        bytes = bytes.checked_add(count as u64)?;
        if bytes > Limits::default().max_bytes {
            return None;
        }
        digest.update(&buffer[..count]);
    }
    Some(format!("{:x}", digest.finalize()))
}

fn matches_restored_fixture(mut saved: Document, restored: &Document) -> bool {
    // Restore deliberately renews these three runtime ownership fields. All
    // other data in this authored, recipe-free fixture must remain identical.
    if saved.owner_session == restored.owner_session
        || saved.epoch.checked_add(1) != Some(restored.epoch)
        || saved.session_generation.checked_add(1) != Some(restored.session_generation)
    {
        return false;
    }
    saved.owner_session = restored.owner_session;
    saved.epoch = restored.epoch;
    saved.session_generation = restored.session_generation;
    saved == *restored
}

impl MatrixApp {
    pub(super) fn file_success_audit_input(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
    ) {
        let audit = self.audit.as_ref().expect("audit");
        if audit.awaiting_screenshot || audit.receipt_written {
            ctx.request_repaint();
            return;
        }
        let step = audit.step;
        let original = audit.directory.join("selected.om9");
        let fixture = audit.directory.join(if step <= 6 {
            "selected.om9"
        } else {
            "alternate.om9"
        });
        let receipts = audit.file_picker_receipts.len();
        let clean = self.controller.is_dirty().is_ok_and(|dirty| !dirty);
        let phase = self
            .controller
            .file_operation()
            .map(|operation| operation.phase());
        let ready = match step {
            6 => receipts == 1 && phase.is_none() && clean && fixture.is_file(),
            13..=15 => {
                receipts == 2
                    && phase == Some(FilePhase::Overwrite)
                    && self.file_control_rect(FileControl::Overwrite).is_some()
            }
            16 => receipts == 2 && phase.is_none() && clean,
            19 => self.controller.document().entities.is_empty() && phase.is_none(),
            22 => receipts == 3 && phase.is_none() && clean,
            _ => true,
        };
        if !ready || !raw.focused {
            let audit = self.audit.as_mut().expect("audit");
            let started = audit
                .file_success_wait_started
                .get_or_insert_with(std::time::Instant::now);
            if started.elapsed() > std::time::Duration::from_secs(5) {
                audit
                    .checks
                    .push(format!("FAILED selection step {step}: readiness/focus"));
                audit.awaiting_screenshot = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            } else if !raw.focused && audit.native_focus_wait_started.is_none() {
                audit.native_focus_wait_started = Some(std::time::Instant::now());
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.audit
            .as_mut()
            .expect("audit")
            .file_success_wait_started = None;
        self.audit
            .as_mut()
            .expect("audit")
            .native_focus_wait_started = None;
        // Each checkpoint compares real archive data, not just dialog closure.
        let count = self.controller.document().entities.len();
        let check = match step {
            6 if count == 1
                && Archive::open(&fixture, &Limits::default())
                    .is_ok_and(|document| document == *self.controller.document()) =>
            {
                if std::fs::copy(&fixture, fixture.with_file_name("alternate.om9")).is_err() {
                    self.audit
                        .as_mut()
                        .expect("audit")
                        .checks
                        .push("FAILED authored overwrite fixture copy".into());
                    self.audit.as_mut().expect("audit").awaiting_screenshot = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                    return;
                }
                self.audit.as_mut().expect("audit").file_success_document =
                    Some(self.controller.document().id);
                let original_sha = fixture_digest(&fixture);
                let seed_sha = fixture_digest(&fixture.with_file_name("alternate.om9"));
                if original_sha.is_none() || original_sha != seed_sha {
                    self.audit
                        .as_mut()
                        .expect("audit")
                        .checks
                        .push("FAILED copied archive fingerprint".into());
                    self.audit.as_mut().expect("audit").awaiting_screenshot = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                    return;
                }
                self.audit
                    .as_mut()
                    .expect("audit")
                    .file_success_original_sha = original_sha;
                self.audit.as_mut().expect("audit").file_success_seed_sha = seed_sha;
                Some("real Windows Save selection writes exact owned document")
            }
            13 if count == 2
                && self.controller.active_file_path() == Some(original.as_path())
                && fixture_digest(&original)
                    == self
                        .audit
                        .as_ref()
                        .expect("audit")
                        .file_success_original_sha
                && fixture_digest(&fixture)
                    == self.audit.as_ref().expect("audit").file_success_seed_sha
                && Archive::open(&fixture, &Limits::default())
                    .is_ok_and(|document| document.entities.len() == 1) =>
            {
                Some("native SaveAs selection waits for owned overwrite without changing file")
            }
            16 if count == 2
                && self.controller.active_file_path() == Some(fixture.as_path())
                && fixture_digest(&original)
                    == self
                        .audit
                        .as_ref()
                        .expect("audit")
                        .file_success_original_sha
                && Archive::open(&original, &Limits::default())
                    .is_ok_and(|document| document.entities.len() == 1)
                && self.controller.coordinator().undo_len() == 2
                && Archive::open(&fixture, &Limits::default())
                    .is_ok_and(|document| document == *self.controller.document()) =>
            {
                Some("native owned Replace widget adopts SaveAs path without geometry Undo")
            }
            19 if count == 0
                && self.controller.coordinator().undo_len() == 0
                && self.controller.active_file_path().is_none() =>
            {
                Some("native New toolbar renews runtime before reopen")
            }
            22 if count == 2
                && fixture_digest(&original)
                    == self
                        .audit
                        .as_ref()
                        .expect("audit")
                        .file_success_original_sha
                && self.controller.coordinator().undo_len() == 0
                && self.controller.active_file_path() == Some(fixture.as_path())
                && Some(self.controller.document().id)
                    == self.audit.as_ref().expect("audit").file_success_document
                && Archive::open(&fixture, &Limits::default()).is_ok_and(|document| {
                    matches_restored_fixture(document, self.controller.document())
                }) =>
            {
                Some("real Windows Open selection restores document path and scene")
            }
            _ => None,
        };
        if matches!(step, 6 | 13 | 16 | 19 | 22) {
            let audit = self.audit.as_mut().expect("audit");
            if let Some(check) = check {
                audit.checks.push(check.into());
            } else {
                audit.checks.push(format!(
                    "FAILED selection step {step}: document/IO invariant"
                ));
                audit.awaiting_screenshot = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                return;
            }
        }
        let rect = match step {
            14 | 15 => self.file_control_rect(FileControl::Overwrite),
            17 | 18 => self.file_toolbar_rect(FileKind::New),
            _ => None,
        };
        if let Some(rect) = rect {
            let pos = rect.center();
            let pressed = matches!(step, 14 | 17);
            raw.events.push(egui::Event::PointerMoved(pos));
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        let text = match step {
            0 | 7 => Some("Point"),
            2 => Some("1,2,3"),
            4 => Some("Save"),
            9 => Some("9,8,7"),
            11 => Some("SaveAs"),
            20 => Some("Open"),
            _ => None,
        };
        if let Some(text) = text {
            raw.events.push(egui::Event::Text(text.into()));
        }
        if matches!(step, 1 | 3 | 5 | 8 | 10 | 12 | 21) {
            raw.events.push(egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            });
        }
        if step == 23 {
            self.audit.as_mut().expect("audit").awaiting_screenshot = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        let audit = self.audit.as_mut().expect("audit");
        audit.viewport_replay.push(serde_json::json!({"step":step,"raw_focused":raw.focused,"phase":format!("{phase:?}"),"objects":count,"picker_receipts":receipts,"checks":audit.checks}));
        let _ = std::fs::write(
            audit.directory.join("file-replay.json"),
            serde_json::to_vec_pretty(&audit.viewport_replay).expect("trace"),
        );
        audit.step += 1;
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restore_comparison_requires_renewal_and_rejects_other_content_changes() {
        let saved = Document::new("persisted");
        assert!(!matches_restored_fixture(saved.clone(), &saved));
        let mut restored = saved.clone();
        restored.renew_session(&Limits::default()).unwrap();
        assert!(matches_restored_fixture(saved.clone(), &restored));
        restored.name = "changed persistent data".into();
        assert!(!matches_restored_fixture(saved.clone(), &restored));
        restored.name = saved.name.clone();
        restored.context_revision += 1;
        assert!(!matches_restored_fixture(saved, &restored));
    }
    #[test]
    fn missing_owned_overwrite_never_injects_an_accept_click() {
        let mut app = MatrixApp::new(Document::new("guard")).unwrap();
        let mut audit = UiAudit::new(PathBuf::from("H:/owned-audit-guard"));
        audit.file_success = true;
        audit.step = 14;
        app.audit = Some(audit);
        let ctx = egui::Context::default();
        let mut raw = egui::RawInput {
            focused: true,
            ..Default::default()
        };
        app.file_success_audit_input(&ctx, &mut raw);
        assert!(raw.events.is_empty());
        assert_eq!(app.audit.as_ref().unwrap().step, 14);
        assert!(app.controller.file_operation().is_none());
    }
}
