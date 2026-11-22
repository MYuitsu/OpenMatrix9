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
    #[test]
    fn named_jobbag_open_uses_unsaved_lifecycle() {
        let mut app = MatrixApp::new(Document::new("snapshot lifecycle")).unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("1,0,0").unwrap();
        let (document, archive) = app.controller.capture_jobbag_document().unwrap();
        let snapshot = app.project_manager.capture_named_version("Master", document, archive).unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("2,0,0").unwrap();
        let ctx = egui::Context::default();
        let before = app.controller.coordinator().snapshot();
        app.begin_named_jobbag_open("Master", snapshot).unwrap();
        let operation = app.controller.file_operation().unwrap();
        assert_eq!(operation.phase(), FilePhase::Unsaved);
        let token = operation.token();
        app.controller.cancel_file_operation(token).unwrap();
        ctx.run_logic(&egui::RawInput::default(), |ctx| app.file_ui(ctx));
        assert_eq!(app.controller.document().entities.len(), 2);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert!(app.pending_jobbag_open.is_none());

        app.begin_named_jobbag_open("Master", snapshot).unwrap();
        let token = app.controller.file_operation().unwrap().token();
        app.controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
        ctx.run_logic(&egui::RawInput::default(), |ctx| app.file_ui(ctx));
        assert_eq!(app.controller.document().entities.len(), 1);
        assert!(app.controller.file_operation().is_none());
    }
    fn raw_close() -> egui::RawInput {
        let mut raw = egui::RawInput::default();
        raw.viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
        raw
    }

    #[cfg(feature = "occt-brep")]
    fn stl_job_fixture() -> (MatrixApp, om9_core::identity::ObjectId) {
        let limits = om9_core::budget::Limits::default();
        let sphere = om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("STL background fixture");
        let object = om9_core::model::Entity::new("sphere", om9_core::geometry::Geometry::Brep(sphere), None);
        let id = object.id;
        document.entities.insert(id, object);
        let mut app = MatrixApp::new(document).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        (app, id)
    }

    #[cfg(feature = "occt-brep")]
    const STL_PROFILE: crate::session::StlExportProfile = crate::session::StlExportProfile {
        linear_tolerance_mm: 0.01,
        angular_tolerance_rad: 0.3,
    };

    #[cfg(feature = "occt-brep")]
    struct StlOutput(PathBuf);

    #[cfg(feature = "occt-brep")]
    impl StlOutput {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!("om9-ui-stl-{}.stl", uuid::Uuid::new_v4())))
        }
    }

    #[cfg(feature = "occt-brep")]
    impl Drop for StlOutput {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[cfg(feature = "occt-brep")]
    fn drain_stl_job(app: &mut MatrixApp, ctx: &egui::Context) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.stl_job.is_some() {
            assert!(std::time::Instant::now() < deadline, "STL worker timed out");
            app.service_stl_export(ctx);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_background_menu_preserves_bytes_during_edit_undo_and_redo() {
        let (mut app, source) = stl_job_fixture();
        let baseline = StlOutput::new();
        let target = StlOutput::new();
        let ctx = egui::Context::default();
        let receipt = app.controller.export_stl_selected_with_profile(&baseline.0, STL_PROFILE).unwrap();
        assert!(receipt.watertight());
        app.invoke("OM9-FILE-001", crate::session::InvocationOrigin::Menu);
        let capture = app.stl_capture.take().unwrap();
        app.cancel_stl_export_options();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        app.begin_stl_export_options();
        assert!(app.stl_capture.is_none());
        assert_eq!(app.status.message(), "STL export is already in progress");
        assert!(!target.0.exists(), "worker waits for admission while UI remains usable");

        app.controller.delete_objects(&[source]).unwrap();
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.contains_key(&source));
        app.controller.redo().unwrap();
        assert!(!app.controller.document().entities.contains_key(&source));
        let before_completion = app.controller.coordinator().snapshot();
        drain_stl_job(&mut app, &ctx);
        assert!(std::fs::read(&target.0).unwrap() == std::fs::read(&baseline.0).unwrap(), "the same captured sphere and profile must produce identical STL bytes");
        assert!(app.transcript.history().contains("Exported STL:"));
        assert_eq!(app.controller.coordinator().snapshot(), before_completion);
        assert_eq!(app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::Export), 0);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_background_admission_sees_pressure_added_after_capture() {
        use om9_core::budget::memory::MemoryClass;
        let (mut app, _) = stl_job_fixture();
        let target = StlOutput::new();
        std::fs::write(&target.0, b"previous STL").unwrap();
        let ctx = egui::Context::default();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap(),
        ).unwrap();
        let before = app.controller.coordinator().snapshot();
        drain_stl_job(&mut app, &ctx);
        assert!(app.status.kind().is_failure());
        assert!(app.status.message().contains("budget"), "{}", app.status_line());
        assert_eq!(std::fs::read(&target.0).unwrap(), b"previous STL");
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        drop(pressure);
        app.status.acknowledge();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        drain_stl_job(&mut app, &ctx);
        assert_ne!(std::fs::read(&target.0).unwrap(), b"previous STL");
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_background_close_discards_old_completion_and_releases_busy_slot() {
        let (mut app, source) = stl_job_fixture();
        app.controller.submit("Point").unwrap();
        app.controller.submit("2,3,4").unwrap();
        app.controller.select_objects(&[source]).unwrap();
        let target = StlOutput::new();
        std::fs::write(&target.0, b"previous STL").unwrap();
        let ctx = egui::Context::default();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        let _ = ctx.run_logic(&raw_close(), |ctx| { app.service_window_close(ctx); });
        assert!(app.stl_job.as_ref().unwrap().canceling());
        let history = app.transcript.history().to_owned();
        drain_stl_job(&mut app, &ctx);
        assert_eq!(app.transcript.history(), history);
        assert_eq!(std::fs::read(&target.0).unwrap(), b"previous STL");
        let token = app.controller.file_operation().unwrap().token();
        app.controller.cancel_file_operation(token).unwrap();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        drain_stl_job(&mut app, &ctx);
        assert_ne!(std::fs::read(&target.0).unwrap(), b"previous STL");
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_background_clean_close_waits_for_worker_cleanup() {
        let (mut app, _) = stl_job_fixture();
        assert!(!app.controller.is_dirty().unwrap());
        let target = StlOutput::new();
        std::fs::write(&target.0, b"previous STL").unwrap();
        let ctx = egui::Context::default();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        let _ = ctx.run_logic(&raw_close(), |ctx| {
            assert!(!app.service_window_close(ctx));
        });
        assert_eq!(app.controller.file_operation().unwrap().phase(), FilePhase::CloseReady);
        assert!(app.stl_job.as_ref().unwrap().canceling());
        drain_stl_job(&mut app, &ctx);
        assert_eq!(std::fs::read(&target.0).unwrap(), b"previous STL");
        let _ = ctx.run_logic(&egui::RawInput::default(), |ctx| {
            assert!(app.service_window_close(ctx));
        });
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_background_same_uuid_open_suppresses_old_completion() {
        let (mut app, _) = stl_job_fixture();
        let project = StlOutput(StlOutput::new().0.with_extension("om9"));
        let target = StlOutput::new();
        std::fs::write(&target.0, b"previous STL").unwrap();
        app.controller.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = app.controller.file_operation().unwrap().token();
        assert_eq!(app.controller.accept_file_path(token, &project.0).unwrap(), FileOutcome::Saved);
        let document_id = app.controller.document().id;
        let owner = app.controller.document().owner_session;
        let ctx = egui::Context::default();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        app.controller.begin_file_operation(FileKind::Open).unwrap();
        let token = app.controller.file_operation().unwrap().token();
        assert_eq!(app.controller.accept_file_path(token, &project.0).unwrap(), FileOutcome::Replaced);
        assert_eq!(app.controller.document().id, document_id);
        assert_ne!(app.controller.document().owner_session, owner);
        let before = app.controller.coordinator().snapshot();
        let history = app.transcript.history().to_owned();
        drain_stl_job(&mut app, &ctx);
        assert_eq!(app.transcript.history(), history);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(std::fs::read(&target.0).unwrap(), b"previous STL");
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn stl_background_cancel_button_preserves_destination() {
        let (mut app, _) = stl_job_fixture();
        let target = StlOutput::new();
        std::fs::write(&target.0, b"previous STL").unwrap();
        let ctx = egui::Context::default();
        let capture = app.controller.capture_stl_selected().unwrap();
        app.launch_stl_export(&ctx, capture, target.0.clone(), STL_PROFILE).unwrap();
        let frame = |app: &mut MatrixApp, events| {
            let mut output = ctx.run_ui(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900., 600.))),
                focused: true, events, ..Default::default()
            }, |ui| app.stl_progress_ui(ui));
            output.textures_delta.clear();
        };
        for _ in 0..3 { frame(&mut app, vec![]); }
        let pos = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("stl-worker-cancel-rect"))).unwrap().center();
        for pressed in [true, false] {
            frame(&mut app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
                pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
            }]);
        }
        assert!(app.stl_job.as_ref().unwrap().canceling());
        drain_stl_job(&mut app, &ctx);
        assert_eq!(app.status.message(), "STL export cancelled");
        assert_eq!(std::fs::read(&target.0).unwrap(), b"previous STL");
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn close_request_discards_pending_stl_capture() {
        let mut app = MatrixApp::new(Document::with_template_tolerance("close stl")).unwrap();
        app.controller.submit("Box 10 10 10").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.begin_stl_export_options();
        assert!(app.stl_capture.is_some());
        let detached = app.controller.capture_stl_selected().unwrap();
        let ctx = egui::Context::default();
        let _ = ctx.run_logic(&raw_close(), |ctx| { app.service_window_close(ctx); });
        assert!(app.stl_capture.is_none());
        assert!(app.stl_profile_draft.is_none());
        let path = std::env::temp_dir().join(format!("om9-close-stl-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"previous destination").unwrap();
        let profile = crate::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 };
        assert!(matches!(detached.export_with_profile(&path, profile), Err(om9_core::Error::Canceled)));
        assert_eq!(std::fs::read(&path).unwrap(), b"previous destination");
        let token = app.controller.file_operation().unwrap().token();
        app.controller.cancel_file_operation(token).unwrap();
        let fresh = app.controller.capture_stl_selected().unwrap();
        fresh.export_with_profile(&path, profile).unwrap();
        std::fs::remove_file(path).unwrap();
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
    pub(super) fn begin_named_jobbag_open(&mut self, name: &'static str, snapshot: crate::project_manager::SnapshotId) -> Result<()> {
        if self.controller.file_operation().is_some() || self.pending_jobbag_open.is_some() {
            return Err(Error::Busy);
        }
        let row = self.project_manager.active();
        let project = row.id;
        self.project_manager.named_master_by_id(name, snapshot).ok_or(Error::Missing("named jobbag version"))?;
        self.controller.begin_file_operation(FileKind::Open)?;
        self.pending_jobbag_open = Some((project, snapshot, name, self.project_manager_generation));
        Ok(())
    }
    /// Runs in both normal UI and eframe's minimized/occluded logic pass.
    /// Keep the approved operation alive until native shutdown, so a later
    /// document mutation cannot reuse consent captured for an older owner.
    pub(super) fn service_window_close(&mut self, ctx: &egui::Context) -> bool {
        let requested = ctx.input(|input| input.viewport().close_requested());
        if requested {
            if self.project_capture_worker.is_some() || self.project_brep_insert_pending() || self.project_brep_open_pending() || self.project_document_worker.is_some() || self.project_import_worker.is_some() || self.project_export_worker.is_some() || self.project_root_worker.is_some() || self.project_catalog_load_worker.is_some() || self.project_thumbnail_worker.is_some() {
                if let Some(worker) = &self.project_capture_worker { worker.cancel(); }
                #[cfg(feature = "occt-brep")]
                if let Some(worker) = &self.project_brep_insert_worker { worker.cancel(); }
                #[cfg(feature = "occt-brep")]
                if let Some(worker) = &self.project_brep_open_worker { worker.cancel(); }
                if let Some(worker) = &self.project_document_worker { worker.cancel(); }
                if let Some(worker) = &self.project_import_worker { worker.cancel(); }
                if let Some(worker) = &self.project_export_worker { worker.cancel(); }
                if let Some(worker) = &self.project_root_worker { worker.cancel(); }
                if let Some(worker) = &self.project_catalog_load_worker { worker.cancel(); }
                if let Some(worker) = &self.project_thumbnail_worker { worker.cancel(); }
                self.project_capture_close_wait = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.request_repaint();
                return false;
            }
            let revision = self.project_manager.change_revision();
            let dirty_catalog = self.project_catalog_path.is_some()
                && (self.project_catalog_ready && self.project_catalog_revision != Some(revision)
                    || !self.project_catalog_ready && revision != 0);
            if dirty_catalog && self.project_catalog_discard_revision != Some(revision) {
                if self.project_catalog_worker.is_some() {
                    self.project_catalog_close_wait = true;
                } else {
                    self.apply_project_action(crate::project_manager::ProjectAction::OpenManager);
                    self.project_confirmation = Some((super::projects::ProjectConfirmation::CloseUnsavedCatalog(revision), self.project_manager_generation));
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.request_repaint();
                return false;
            }
            self.cancel_stl_export();
            self.controller.invalidate_stl_session_for_close();
        }
        let approved = self.controller.file_operation().is_some_and(|operation| {
            operation.phase() == FilePhase::CloseReady
                && self
                    .controller
                    .validate_file_operation(operation.token())
                    .is_ok()
        });
        if approved {
            if let Some(job) = &self.stl_job {
                // Let the canceled worker remove staging output before the
                // process exits. Completion wakes the normal close logic.
                if !job.canceling() {
                    job.cancel();
                }
                if requested {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                return false;
            }
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
            if clean_close && self.stl_job.is_none() {
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
        #[cfg(feature = "occt-brep")]
        let sphere_audit = self.sphere_boolean_audit.as_ref().is_some_and(|audit| audit.can_discard());
        #[cfg(not(feature = "occt-brep"))]
        let sphere_audit = false;
        if self.audit.is_none() && !sphere_audit {
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
    /// The current File palette's icon target in the left MAIN MENU.
    pub fn file_sidebar_rect(&self, kind: FileKind) -> Option<Rect> {
        (self.top_menu_selected() == Some(0)).then(|| {
            self.current_ui_context.as_ref()?.memory(|memory|
                memory.data.get_temp::<Rect>(egui::Id::new(("file-sidebar", kind.feature_id()))))
        }).flatten()
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
    fn restore_keyboard_after_file_action(&mut self, ctx: &egui::Context) {
        if self.controller.file_operation().is_none()
            && (self.controller.is_running() || self.controller.addon_session().is_some())
        {
            self.focus_command_editor();
        } else {
            self.release_command_editor_focus(ctx);
        }
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
                    self.transcript
                        .set_prompt(&self.controller.prompt_with_options());
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
                self.transcript
                    .set_prompt(&self.controller.prompt_with_options());
                self.restore_keyboard_after_file_action(ctx);
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
                self.restore_keyboard_after_file_action(ctx);
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
            self.pending_jobbag_open = None;
            self.file_error.clear();
            self.restore_keyboard_after_file_action(ctx);
            ctx.request_repaint();
            return Ok(());
        };
        // Save invocation stages UI metadata before capturing this token.
        // A picker response must not mutate the captured document/context.
        let result = if self.controller.file_operation().is_some_and(|op| op.phase() == FilePhase::OpenPath) {
            self.controller.request_file_open_worker(token, &path)
        } else {
            self.controller.accept_file_path(token, &path)
        };
        self.file_feedback(ctx, result)
    }
    pub(super) fn file_ui(&mut self, ctx: &egui::Context) {
        self.file_controls.clear();
        let Some(operation) = self.controller.file_operation() else {
            self.pending_jobbag_open = None;
            self.file_prompt_token = None;
            self.file_picker_request = None;
            return;
        };
        let token = operation.token();
        let phase = operation.phase();
        if self.pending_jobbag_open.is_some_and(|(_, _, _, generation)| generation != self.project_manager_generation) {
            self.pending_jobbag_open = None;
            self.file_picker_request = None;
            let result = self.controller.cancel_file_operation(token).and(Err(Error::Stale("project catalog changed")));
            self.feedback(result);
            return;
        }
        if self.project_brep_open_pending() { return; }
        if phase == FilePhase::OpenPath && let Some((project, snapshot, name, _)) = self.pending_jobbag_open.take() {
            let current = self.project_manager.active().id == project
                && self.project_manager.named_master_by_id(name, snapshot).is_some();
            #[cfg(feature = "occt-brep")]
            if current && let Some(archive) = self.project_manager.named_master_by_id(name, snapshot).and_then(|item| item.archive.clone())
                && archive.len() >= 12 && &archive[..8] == b"OM9\0\r\n\x1a\n"
                && matches!(u32::from_le_bytes(archive[8..12].try_into().expect("header checked")), 6 | 7) {
                if let Err(error) = self.start_named_brep_open(token, project, name, snapshot, archive) {
                    let _ = self.controller.cancel_file_operation(token);
                    self.feedback(Err(error));
                }
                return;
            }
            let result = self.controller.cancel_file_operation(token).and_then(|_| {
                if !current { return Err(Error::Stale("named jobbag changed")); }
                self.restore_named_jobbag_version(name, snapshot)?;
                Ok(FileOutcome::Replaced)
            });
            let _ = self.file_feedback(ctx, result);
            return;
        }
        if phase == FilePhase::CloseReady {
            self.service_window_close(ctx);
            return;
        }
        if matches!(phase, FilePhase::OpenPreparing | FilePhase::SavePreparing) { return; }
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
        super::theme::modal(ctx, egui::Id::new("owned-file-operation"))
            .show(ctx, |ui| {
                painted = !ui.is_sizing_pass();

                ui.set_width(360.);
                ui.heading(match phase {
                    FilePhase::Unsaved => "Save changes?",
                    FilePhase::Overwrite => "Replace existing file?",
                    FilePhase::OpenPath => "Open document",
                    FilePhase::SavePath => "Save document",
                    FilePhase::CloseReady | FilePhase::OpenPreparing | FilePhase::SavePreparing => unreachable!("handled before modal"),
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
                    FilePhase::CloseReady | FilePhase::OpenPreparing | FilePhase::SavePreparing => unreachable!("handled before modal"),
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
            let closing = self.controller.file_operation().is_some_and(|operation| operation.intent() == crate::file_lifecycle::FileIntent::WindowClose);
            let result = self.controller.cancel_file_operation(token);
            if closing { self.project_catalog_discard_revision = None; }
            self.pending_jobbag_open = None;
            self.file_picker_request = None;
            self.feedback(result);
            self.restore_keyboard_after_file_action(ctx);
        } else if let Some(decision) = decision {
            let result = if decision == UnsavedDecision::Save && self.pending_jobbag_open.is_some() && !self.preserve_embedded_project_state() {
                self.controller.stage_project_for_unsaved_save(token, &self.project_manager)
                    .and_then(|next| self.controller.decide_unsaved(next, decision))
            } else {
                self.controller.decide_unsaved(token, decision)
            };
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

    pub(super) fn begin_stl_export_options(&mut self) {
        if self.stl_job.is_some() {
            self.status.info("STL export is already in progress");
            return;
        }
        if self.stl_capture.is_some() {
            self.status.error("STL export is already in progress");
            return;
        }
        let result = self.controller.document().effective_tolerance()
            .and_then(|tolerance| self.controller.capture_stl_selected().map(|capture| (tolerance, capture)));
        match result {
            Ok((tolerance, capture)) => {
                self.stl_capture = Some(capture);
                self.stl_profile_draft = Some(tolerance.absolute_mm);
                self.stl_angle_draft = 0.3;
                self.stl_selected_profile = None;
                self.stl_request = false;
                self.focus_cmd = false;
                self.status.info("Choose STL export quality");
            }
            Err(error) => self.feedback(Err(error)),
        }
    }

    pub(super) fn cancel_stl_export_options(&mut self) {
        self.stl_profile_draft = None;
        self.stl_selected_profile = None;
        self.stl_request = false;
        self.stl_capture = None;
    }

    pub(super) fn cancel_stl_export(&mut self) {
        self.cancel_stl_export_options();
        if let Some(job) = &self.stl_job {
            job.cancel();
        }
    }

    pub(super) fn stl_progress_ui(&mut self, ui: &mut egui::Ui) {
        let Some(job) = &self.stl_job else { return };
        ui.scope(|ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    let canceling = job.canceling();
                    ui.label(if canceling { "Cancelling STL…" } else { "Exporting STL…" });
                    let cancel = ui.add_enabled(!canceling, egui::Button::new("Cancel"));
                    #[cfg(test)]
                    ui.ctx().data_mut(|data| data.insert_temp(egui::Id::new("stl-worker-cancel-rect"), cancel.rect));
                    if cancel.clicked() {
                        job.cancel();
                    }
                });
            });
    }

    pub(super) fn service_stl_export(&mut self, ctx: &egui::Context) {
        let Some(job) = &mut self.stl_job else { return };
        let current = job.belongs_to(&self.controller);
        let Some(result) = job.poll(&self.controller) else { return };
        // Keep the receipt's lease alive through transcript formatting.
        let _finished = self.stl_job.take();
        if current {
            match result {
                Ok(receipt) => {
                    self.transcript.append_history(&receipt.message());
                    self.status.complete_success();
                }
                Err(Error::Canceled) => {
                    self.status.info("STL export cancelled");
                }
                Err(error) => {
                    let message = crate::transcript::redact_paths(&error.to_string());
                    self.transcript.append_history(&message);
                    self.status.error(&message);
                }
            }
        }
        ctx.request_repaint();
    }

    pub(super) fn stl_profile_ui(&mut self, ui: &mut egui::Ui) {
        let Some(mut tolerance) = self.stl_profile_draft else { return };
        let ctx = ui.ctx().clone();
        let mut accept = false;
        let mut cancel = false;
        let mut angle = self.stl_angle_draft;
        ui.scope(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Requested linear tolerance (mm):");
                    ui.add(egui::DragValue::new(&mut tolerance).speed(0.0001).max_decimals(12));
                });
                ui.horizontal(|ui| {
                    ui.label("Angular deflection (radians):");
                    ui.add(egui::DragValue::new(&mut angle).speed(0.01).max_decimals(8));
                });
                ui.label("The final geometric deviation is not yet certified.");
                ui.horizontal(|ui| {
                    let profile = crate::session::StlExportProfile { linear_tolerance_mm: tolerance, angular_tolerance_rad: angle };
                    let button = ui.add_enabled(profile.validate().is_ok(), egui::Button::new("Choose file"));
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("stl-profile-accept-rect"), button.rect));
                    accept = button.clicked();
                    let button = ui.button("Cancel");
                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("stl-profile-cancel-rect"), button.rect));
                    cancel = button.clicked();
                });
            });
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            cancel = true;
        }
        if accept {
            self.stl_selected_profile = Some(crate::session::StlExportProfile { linear_tolerance_mm: tolerance, angular_tolerance_rad: angle });
            self.stl_request = true;
            self.stl_profile_draft = None;
            self.status.info("Choose an STL destination");
        } else if cancel {
            self.cancel_stl_export_options();
            self.focus_cmd = true;
            self.status.info("STL export cancelled");
        } else {
            self.stl_profile_draft = Some(tolerance);
            self.stl_angle_draft = angle;
        }
    }

    /// File ▸ Export Selected: pick the `.stl` destination, then export the
    /// selected meshes/solids through the Controller. Export never changes the document.
    pub(super) fn service_stl_picker(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if !self.stl_request {
            return;
        }
        self.stl_request = false;
        let Some(profile) = self.stl_selected_profile.take() else {
            self.stl_capture = None;
            self.status.error("Missing STL export quality");
            return;
        };
        let Some(capture) = self.stl_capture.take() else {
            self.status.error("Missing captured STL source");
            return;
        };
        let current = self
            .controller
            .active_file_path()
            .map(std::path::Path::to_path_buf);
        match crate::native_picker::pick_stl(frame, current.as_deref()) {
            Ok(Some(path)) => {
                if let Err(error) = self.launch_stl_export(ctx, capture, path, profile) {
                    let message = crate::transcript::redact_paths(&format!("{error}"));
                    self.transcript.append_history(&message);
                    self.status.error(&message);
                }
            }
            Ok(None) => {}
            Err(error) => {
                let message = crate::transcript::redact_paths(&format!("{error}"));
                self.transcript.append_history(&message);
                self.status.error(&message);
            }
        }
        self.focus_cmd = true;
        ctx.request_repaint();
    }

    fn launch_stl_export(
        &mut self,
        ctx: &egui::Context,
        capture: crate::session::StlExportCapture,
        path: PathBuf,
        profile: crate::session::StlExportProfile,
    ) -> Result<()> {
        if self.stl_job.is_some() {
            return Err(Error::Busy);
        }
        let wake = ctx.clone();
        self.stl_job = Some(self.controller.start_interactive_stl(capture, path, profile, move || wake.request_repaint())?);
        self.status.info("Exporting STL…");
        ctx.request_repaint();
        Ok(())
    }

    /// Load application-scoped Gumball preferences. A missing file is defaults;
    /// a corrupt file is reported without blocking startup.
    pub fn load_gumball_settings(&mut self, path: &std::path::Path) -> Result<()> {
        use crate::session::gumball;
        self.gumball_settings_path = Some(path.to_path_buf());
        self.gumball_settings_saved = self.controller.gumball_settings();
        if !path.exists() {
            return Ok(());
        }
        match gumball::load_settings(path) {
            Ok(settings) => {
                self.controller.apply_gumball_settings(settings)?;
                self.gumball_settings_saved = settings;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Write changed preferences once; a failed write stays visible but never spams
    /// the status line on every frame.
    pub(super) fn persist_gumball_settings(&mut self) {
        let Some(path) = self.gumball_settings_path.clone() else {
            return;
        };
        let current = self.controller.gumball_settings();
        if current == self.gumball_settings_saved {
            return;
        }
        match crate::session::gumball::save_settings(&path, &current) {
            Ok(()) => {
                self.gumball_settings_saved = current;
                self.gumball_settings_error_reported = false;
            }
            Err(error) => {
                if !self.gumball_settings_error_reported {
                    self.gumball_settings_error_reported = true;
                    self.status.error(&format!("Gumball settings: {error}"));
                }
            }
        }
    }

    pub fn gumball_settings_path(&self) -> Option<&std::path::Path> {
        self.gumball_settings_path.as_deref()
    }

    /// Gumball settings menu: axes, Smooth/Snappy, drag strength, pivot reset and
    /// the documented defaults. Opened from the Info & Settings alignment icon.
    pub(super) fn gumball_settings_ui(&mut self, ui: &mut egui::Ui) {
        use crate::session::GumballAlignment;
        if !self.gumball_settings_open {
            return;
        }

        let mut alignment = self.controller.gumball_alignment();
        let mut snappy = self.controller.gumball_snappy();
        let mut strength = self.controller.gumball_drag_strength() * 100.0;
        let mut reset_pivot = false;
        let mut restore = false;
        ui.scope(|ui| {
                ui.label("Alignment");
                ui.horizontal(|ui| {
                    for (label, mode) in [
                        ("World", GumballAlignment::World),
                        ("Object", GumballAlignment::Object),
                        ("CPlane", GumballAlignment::CPlane),
                    ] {
                        if ui.selectable_label(alignment == mode, label).clicked() {
                            alignment = mode;
                        }
                    }
                });
                ui.separator();
                ui.label("Drag");
                ui.horizontal(|ui| {
                    if ui.selectable_label(!snappy, "Smooth").clicked() {
                        snappy = false;
                    }
                    if ui.selectable_label(snappy, "Snappy").clicked() {
                        snappy = true;
                    }
                });
                ui.add(
                    egui::Slider::new(&mut strength, 1.0..=200.0)
                        .suffix("%")
                        .text("Drag strength"),
                );
                ui.separator();
                reset_pivot = ui.button("Reset pivot").clicked();
                restore = ui.button("Restore defaults").clicked();
            });
        if restore {
            self.controller.restore_gumball_defaults();
        } else {
            if self.controller.gumball_alignment() != alignment {
                self.controller.set_gumball_alignment(alignment);
            }
            if self.controller.gumball_snappy() != snappy {
                self.controller.set_gumball_snappy(snappy);
            }
            let _ = self.controller.set_gumball_drag_strength(strength / 100.0);
            if reset_pivot {
                self.controller.relocate_gumball(None);
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
