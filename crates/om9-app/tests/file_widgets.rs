// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::{
    file_lifecycle::{FileKind, FilePhase},
    session::InvocationOrigin,
    ui::{FileControl, MatrixApp},
};
use om9_core::model::Document;
fn frame(app: &mut MatrixApp, ctx: &egui::Context, mut events: Vec<Event>) -> egui::FullOutput {
    let modifiers = events
        .iter()
        .find_map(|event| match event {
            Event::PointerButton { modifiers, .. } | Event::Key { modifiers, .. } => {
                Some(*modifiers)
            }
            _ => None,
        })
        .unwrap_or(Modifiers::NONE);
    events.insert(0, Event::ModifiersChanged(modifiers));
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 1000.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}
fn start() -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    let ctx = egui::Context::default();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
fn click(app: &mut MatrixApp, ctx: &egui::Context, pos: Pos2) {
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    for _ in 0..3 {
        frame(app, ctx, vec![]);
    }
}
fn key(key: egui::Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn dirty_line(app: &mut MatrixApp) {
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,2,3").unwrap();
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.transcript.set_draft("12,6,");
}
fn close_frame(app: &mut MatrixApp, ctx: &egui::Context) -> egui::FullOutput {
    let mut raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 1000.))),
        ..Default::default()
    };
    raw.viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .events
        .push(egui::ViewportEvent::Close);
    let mut output = ctx.run_ui(raw, |ui| app.show(ui));
    output.textures_delta.clear();
    output
}
fn canceled_close(output: &egui::FullOutput) -> bool {
    output.viewport_output[&egui::ViewportId::ROOT]
        .commands
        .contains(&egui::ViewportCommand::CancelClose)
}
#[test]
fn window_close_cancel_and_repeated_requests_keep_line_draft_and_same_prompt() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    let token = app.controller.file_operation().unwrap().token();
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    assert_eq!(app.controller.file_operation().unwrap().token(), token);
    let pos = app.file_control_rect(FileControl::Cancel).unwrap().center();
    click(&mut app, &ctx, pos);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "12,6,");
}
#[test]
fn window_close_discard_requests_shutdown_without_replacing_old_document() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let pos = app
        .file_control_rect(FileControl::Discard)
        .unwrap()
        .center();
    click(&mut app, &ctx, pos);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::CloseReady
    );
    let output = close_frame(&mut app, &ctx);
    assert!(!canceled_close(&output));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
    assert!(app.controller.is_dirty().unwrap());
}
#[test]
fn clean_close_accepts_and_other_file_owner_defers_close() {
    let (mut app, ctx) = start();
    assert!(!canceled_close(&close_frame(&mut app, &ctx)));
    app.controller
        .cancel_file_operation(app.controller.file_operation().unwrap().token())
        .unwrap();
    app.controller.begin_file_operation(FileKind::Open).unwrap();
    let token = app.controller.file_operation().unwrap().token();
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    assert_eq!(app.controller.file_operation().unwrap().token(), token);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::OpenPath
    );
}
#[test]
fn window_close_first_save_picker_cancel_preserves_document_and_draft() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let pos = app.file_control_rect(FileControl::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, None).unwrap();
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
}
#[test]
fn file_menu_and_cmd_dispatch_owned_intents_through_same_controller() {
    let (mut app, _) = start();
    dirty_line(&mut app);
    app.controller
        .dispatch(FileKind::New.feature_id(), InvocationOrigin::Menu)
        .unwrap();
    assert_eq!(
        app.controller.file_operation().unwrap().kind(),
        Some(FileKind::New)
    );
    let token = app.controller.file_operation().unwrap().token();
    app.controller.cancel_file_operation(token).unwrap();
    app.controller.submit("New").unwrap();
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
}
#[test]
fn toolbar_new_opens_unsaved_prompt_and_escape_keeps_line_draft_and_redo() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let views = app.controller.workspace.views.clone();
    let pos = app
        .file_toolbar_rect(FileKind::New)
        .expect("New toolbar target")
        .center();
    click(&mut app, &ctx, pos);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    assert!(app.file_control_rect(FileControl::Discard).is_some());
    frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(app.controller.file_operation().is_none());
    assert!(app.controller.is_running());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.workspace.views, views);
    assert_eq!(app.transcript.draft(), "12,6,");
}
struct Files(std::path::PathBuf);
impl Files {
    fn new() -> Self {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let path = root
            .join("build/standalone-validation")
            .join(format!("file-widget-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        for entry in std::fs::read_dir(&self.0).unwrap() {
            let path = entry.unwrap().path();
            assert_eq!(path.parent(), Some(self.0.as_path()));
            assert!(path.is_file());
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(&self.0).unwrap();
    }
}
#[test]
fn close_save_failure_at_active_path_keeps_prompt_draft_and_path() {
    let files = Files::new();
    let path = files.0.join("active.om9");
    let (mut app, ctx) = start();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,2,3").unwrap();
    app.controller.begin_file_operation(FileKind::Save).unwrap();
    let token = app.controller.file_operation().unwrap().token();
    app.accept_file_picker_reply(&ctx, token, Some(path.clone()))
        .unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("9,8,7").unwrap();
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.transcript.set_draft("12,6,");
    let before = app.controller.coordinator().snapshot();
    // Replace our owned file with a same-name directory; output must fail
    // without making this a Save As or adopting any other destination.
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert_eq!(path.parent(), Some(files.0.as_path()));
    struct EmptyDirectoryCleanup<'a>(&'a std::path::Path);
    impl Drop for EmptyDirectoryCleanup<'_> {
        fn drop(&mut self) {
            // Nonrecursive cleanup of the exact owned fixture also runs on
            // assertion unwind, before Files checks its direct file entries.
            let _ = std::fs::remove_dir(self.0);
        }
    }
    let directory_cleanup = EmptyDirectoryCleanup(&path);
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let pos = app.file_control_rect(FileControl::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    assert!(app.pending_file_picker().is_none());
    assert!(app.file_control_rect(FileControl::Cancel).is_some());
    assert!(canceled_close(&close_frame(&mut app, &ctx)));
    assert_eq!(app.controller.active_file_path(), Some(path.as_path()));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert!(app.controller.is_dirty().unwrap());
    assert_eq!(app.transcript.draft(), "12,6,");
    assert!(path.is_dir());
    assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
    // Explicit, nonrecursive teardown of this test's verified path.
    assert_eq!(path.parent(), Some(files.0.as_path()));
    drop(directory_cleanup);
    assert!(!path.exists());
}
#[test]
fn ui_startup_from_file_keeps_active_path_and_missing_module_diagnostics() {
    use om9_core::{
        budget::Limits,
        geometry::{Geometry, Transform, Unit},
        history::{Recipe, ReplayEnvironment, Tolerance},
        model::Entity,
        transaction::Cancellation,
    };
    let files = Files::new();
    let path = files.0.join("startup.om9");
    let mut d = Document::new("startup owned");
    let e = Entity::new("output", Geometry::Point([1., 2., 3.]), None);
    let id = e.id;
    d.entities.insert(id, e);
    let recipe = Recipe::new(
        d.id,
        "OM9-MISSING",
        vec![],
        vec![id],
        ReplayEnvironment {
            command_version: 1,
            solver: "missing".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance {
                absolute_mm: 0.001,
                relative: 1e-6,
                angular_rad: 1e-5,
            },
        },
    );
    d.history.recipes.insert(recipe.id, recipe);
    om9_archive::Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    let app = MatrixApp::from_file(&path).unwrap();
    assert_eq!(app.controller.active_file_path(), Some(path.as_path()));
    assert!(app.controller.is_dirty().unwrap());
    assert!(
        app.transcript
            .history()
            .contains("Missing replay module missing@1")
    );
    assert!(!app.controller.file_diagnostics().is_empty());
}
#[test]
fn real_file_menu_cancel_and_toolbar_picker_reply_preserve_or_replace_exactly_once() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let pos = app.file_menu_rect().unwrap().center();
    click(&mut app, &ctx, pos);
    let pos = app.file_menu_item_rect(FileKind::Open).unwrap().center();
    click(&mut app, &ctx, pos);
    let pos = app.file_control_rect(FileControl::Cancel).unwrap().center();
    click(&mut app, &ctx, pos);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
    let pos = app.file_toolbar_rect(FileKind::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, None).unwrap();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
}
#[test]
fn actual_save_control_routes_pending_new_without_a_second_geometry_undo() {
    let files = Files::new();
    let path = files.0.join("before-new.om9");
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let pos = app.file_toolbar_rect(FileKind::New).unwrap().center();
    click(&mut app, &ctx, pos);
    let pos = app.file_control_rect(FileControl::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, Some(path.clone()))
        .unwrap();
    frame(&mut app, &ctx, vec![]);
    assert!(app.controller.document().entities.is_empty());
    assert!(!app.controller.is_running());
    assert_eq!(app.transcript.draft(), "");
    assert_eq!(
        om9_archive::Archive::open(&path, &om9_core::budget::Limits::default()).unwrap(),
        *before.world.documents.values().next().unwrap().as_ref()
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn owned_overwrite_cancel_and_stale_picker_response_keep_a_newer_owner() {
    let files = Files::new();
    let target = files.0.join("existing.om9");
    std::fs::write(&target, b"old target").unwrap();
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let pos = app.file_toolbar_rect(FileKind::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let old = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, old, Some(target.clone()))
        .unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let current = app.controller.file_operation().unwrap().token();
    assert!(app.accept_file_picker_reply(&ctx, old, None).is_err());
    assert_eq!(app.controller.file_operation().unwrap().token(), current);
    let pos = app.file_control_rect(FileControl::Cancel).unwrap().center();
    click(&mut app, &ctx, pos);
    assert_eq!(std::fs::read(target).unwrap(), b"old target");
    assert!(app.controller.is_dirty().unwrap());
    assert_eq!(app.transcript.draft(), "12,6,");
}

#[test]
fn cmd_enter_opens_prompt_without_saving_and_ime_enter_cannot_affirm_it() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    app.transcript.set_draft("New");
    frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    assert!(app.pending_file_picker().is_none());
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "新".into(),
                active_range_chars: None,
            }),
            key(egui::Key::Enter),
        ],
    );
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    assert_eq!(app.controller.coordinator().snapshot(), before);
}

#[test]
fn discard_new_resets_runtime_only_after_actual_button_and_blocks_canvas_input() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let pos = app.file_toolbar_rect(FileKind::New).unwrap().center();
    click(&mut app, &ctx, pos);
    click(&mut app, &ctx, Pos2::new(800., 400.));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
    let pos = app
        .file_control_rect(FileControl::Discard)
        .unwrap()
        .center();
    click(&mut app, &ctx, pos);
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(!app.controller.is_running());
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.transcript.draft(), "");
}

#[test]
fn shortcuts_preserve_numeric_draft_and_save_as_requests_a_new_path_after_save() {
    let files = Files::new();
    let path = files.0.join("shortcut.om9");
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let mut event = key(egui::Key::S);
    if let Event::Key { modifiers, .. } = &mut event {
        *modifiers = Modifiers {
            ctrl: true,
            command: true,
            ..Modifiers::NONE
        };
    }
    frame(&mut app, &ctx, vec![event]);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, Some(path.clone()))
        .unwrap();
    assert_eq!(app.transcript.draft(), "12,6,");
    assert!(app.controller.is_running());
    assert!(!app.controller.is_dirty().unwrap());
    frame(&mut app, &ctx, vec![]);
    let mut event = key(egui::Key::S);
    if let Event::Key { modifiers, .. } = &mut event {
        *modifiers = Modifiers {
            ctrl: true,
            command: true,
            shift: true,
            ..Modifiers::NONE
        };
    }
    frame(&mut app, &ctx, vec![event]);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    assert_eq!(
        app.controller.file_operation().unwrap().kind(),
        Some(FileKind::SaveAs)
    );
    assert!(app.pending_file_picker().is_some());
    frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert_eq!(app.controller.active_file_path(), Some(path.as_path()));
    assert_eq!(app.transcript.draft(), "12,6,");
}

#[test]
fn failed_picker_path_shows_retry_without_loop_and_cancel_preserves_redo() {
    let files = Files::new();
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    app.controller.cancel();
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    let pos = app.file_toolbar_rect(FileKind::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.pending_file_picker().unwrap();
    assert!(
        app.accept_file_picker_reply(&ctx, token, Some(files.0.join("unsupported.3dm")))
            .is_err()
    );
    for _ in 0..10 {
        frame(&mut app, &ctx, vec![]);
        assert!(app.pending_file_picker().is_none());
    }
    let pos = app.file_control_rect(FileControl::Browse).unwrap().center();
    click(&mut app, &ctx, pos);
    assert!(app.pending_file_picker().is_some());
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, None).unwrap();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn overwrite_replace_button_accepts_archive_once_and_replayed_reply_cannot_replace() {
    let files = Files::new();
    let target = files.0.join("overwrite.om9");
    std::fs::write(&target, b"old target").unwrap();
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let pos = app.file_toolbar_rect(FileKind::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, Some(target.clone()))
        .unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let pos = app
        .file_control_rect(FileControl::Overwrite)
        .unwrap()
        .center();
    click(&mut app, &ctx, pos);
    assert!(app.controller.file_operation().is_none());
    assert!(!app.controller.is_dirty().unwrap());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    let bytes = std::fs::read(&target).unwrap();
    assert!(
        app.accept_file_picker_reply(&ctx, token, Some(target.clone()))
            .is_err()
    );
    assert_eq!(std::fs::read(&target).unwrap(), bytes);
}

#[test]
fn old_discard_or_cancel_pointer_release_cannot_answer_a_new_prompt_owner() {
    for control in [FileControl::Discard, FileControl::Cancel] {
        let (mut app, ctx) = start();
        dirty_line(&mut app);
        let before = app.controller.coordinator().snapshot();
        let pos = app.file_toolbar_rect(FileKind::New).unwrap().center();
        click(&mut app, &ctx, pos);
        let old = app.controller.file_operation().unwrap().token();
        let pos = app.file_control_rect(control).unwrap().center();
        frame(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        app.controller.cancel_file_operation(old).unwrap();
        app.controller.begin_file_operation(FileKind::New).unwrap();
        let newer = app.controller.file_operation().unwrap().token();
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert_eq!(app.controller.file_operation().unwrap().token(), newer);
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.transcript.draft(), "12,6,");
    }
}

#[test]
fn ime_commit_enter_and_escape_wait_for_a_fresh_key_after_composition() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let pos = app.file_toolbar_rect(FileKind::New).unwrap().center();
    click(&mut app, &ctx, pos);
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "新".into(),
                active_range_chars: None,
            }),
            key(egui::Key::Escape),
        ],
    );
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(app.controller.file_operation().is_some());
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Commit("新".into())),
            key(egui::Key::Enter),
        ],
    );
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.transcript.draft(), "12,6,");
}

#[test]
fn thirty_closed_file_prompts_reuse_area_state_and_keep_cmd_editor() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let before = app.controller.coordinator().snapshot();
    let mut baseline = None;
    for _ in 0..30 {
        let pos = app.file_toolbar_rect(FileKind::New).unwrap().center();
        click(&mut app, &ctx, pos);
        let pos = app.file_control_rect(FileControl::Cancel).unwrap().center();
        click(&mut app, &ctx, pos);
        let state = (
            ctx.data_mut(|data| data.count::<egui::scroll_area::State>()),
            ctx.memory(|memory| {
                format!("{:?}", memory.areas())
                    .matches("AreaState {")
                    .count()
            }),
        );
        if let Some(baseline) = baseline {
            assert_eq!(state, baseline);
        } else {
            baseline = Some(state);
        }
        assert!(
            egui::text_edit::TextEditState::load(&ctx, egui::Id::new("matrix-command-transcript"))
                .is_some()
        );
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.transcript.draft(), "12,6,");
    }
}

#[test]
fn held_opening_enter_cannot_accept_save_on_the_new_prompt() {
    let files = Files::new();
    let target = files.0.join("held-enter.om9");
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    let pos = app.file_toolbar_rect(FileKind::Save).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&ctx, token, Some(target.clone()))
        .unwrap();
    app.controller.cancel();
    app.controller.submit("Point").unwrap();
    app.controller.submit("4,5,6").unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let bytes = std::fs::read(&target).unwrap();
    let before = app.controller.coordinator().snapshot();
    app.transcript.set_draft("New");
    frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let token = app.controller.file_operation().unwrap().token();
    let mut event = key(egui::Key::Enter);
    if let Event::Key { repeat, .. } = &mut event {
        *repeat = true;
    }
    frame(&mut app, &ctx, vec![event]);
    assert_eq!(app.controller.file_operation().unwrap().token(), token);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(std::fs::read(target).unwrap(), bytes);
}

#[test]
fn cmd_composition_transfers_to_file_prompt_until_commit() {
    let (mut app, ctx) = start();
    dirty_line(&mut app);
    frame(
        &mut app,
        &ctx,
        vec![Event::Ime(egui::ImeEvent::Preedit {
            text: "新".into(),
            active_range_chars: None,
        })],
    );
    let pos = app.file_toolbar_rect(FileKind::New).unwrap().center();
    click(&mut app, &ctx, pos);
    let token = app.controller.file_operation().unwrap().token();
    frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    assert_eq!(app.controller.file_operation().unwrap().token(), token);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Commit("新".into())),
            key(egui::Key::Enter),
        ],
    );
    assert_eq!(app.controller.file_operation().unwrap().token(), token);
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(app.controller.file_operation().is_none());
}
