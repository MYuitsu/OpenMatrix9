// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::{
    file_lifecycle::{FileKind, FileOutcome, FilePhase, FileToken, UnsavedDecision},
    session::Controller,
};
use om9_archive::Archive;
use om9_core::{Error, budget::Limits, model::Document};
use std::path::{Path, PathBuf};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap()
            .join("build/standalone-validation")
            .join(format!("file-lifecycle-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for entry in std::fs::read_dir(&self.0).unwrap() {
            let p = entry.unwrap().path();
            assert!(p.parent() == Some(self.0.as_path()) && p.is_file());
            std::fs::remove_file(p).unwrap();
        }
        std::fs::remove_dir(&self.0).unwrap();
    }
}
fn start() -> Controller {
    Controller::new(Document::new("Untitled"), Limits::default()).unwrap()
}
fn point(c: &mut Controller, p: &str) {
    c.submit("Point").unwrap();
    c.submit(p).unwrap();
}
fn token(c: &Controller) -> FileToken {
    c.file_operation().unwrap().token()
}
fn save_as(c: &mut Controller, path: &Path) -> FileOutcome {
    assert_eq!(
        c.begin_file_operation(FileKind::SaveAs).unwrap(),
        FileOutcome::Pending
    );
    c.accept_file_path(token(c), path).unwrap()
}
#[test]
fn close_clean_or_discard_keeps_document_and_history_until_native_shutdown() {
    let mut c = start();
    let before = c.coordinator().snapshot();
    assert_eq!(c.begin_window_close().unwrap(), FileOutcome::CloseReady);
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::CloseReady);
    c.validate_file_operation(token(&c)).unwrap();
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel_file_operation(token(&c)).unwrap();
    point(&mut c, "1,2,3");
    point(&mut c, "4,5,6");
    c.undo().unwrap();
    c.submit("Line").unwrap();
    c.submit("0,0,0").unwrap();
    let before = c.coordinator().snapshot();
    assert_eq!(c.begin_window_close().unwrap(), FileOutcome::Pending);
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::Unsaved);
    assert_eq!(
        c.decide_unsaved(token(&c), UnsavedDecision::Discard)
            .unwrap(),
        FileOutcome::CloseReady
    );
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(c.is_running());
    assert!(c.is_dirty().unwrap());
}
#[test]
fn close_save_first_path_and_known_path_preserve_geometry_undo() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    let before = c.coordinator().snapshot();
    c.begin_window_close().unwrap();
    assert_eq!(
        c.decide_unsaved(token(&c), UnsavedDecision::Save).unwrap(),
        FileOutcome::Pending
    );
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::SavePath);
    let path = f.path("close.om9");
    assert_eq!(
        c.accept_file_path(token(&c), &path).unwrap(),
        FileOutcome::SavedAndCloseReady
    );
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(!c.is_dirty().unwrap());
    assert_eq!(
        Archive::open(&path, &Limits::default()).unwrap(),
        *c.document()
    );
    c.cancel_file_operation(token(&c)).unwrap();
    point(&mut c, "4,5,6");
    let before = c.coordinator().snapshot();
    c.begin_window_close().unwrap();
    assert_eq!(
        c.decide_unsaved(token(&c), UnsavedDecision::Save).unwrap(),
        FileOutcome::SavedAndCloseReady
    );
    assert_eq!(c.coordinator().snapshot(), before);
    c.validate_file_operation(token(&c)).unwrap();
    assert_eq!(
        Archive::open(&path, &Limits::default()).unwrap(),
        *c.document()
    );
}
#[test]
fn close_cancel_save_failure_and_stale_consent_cannot_approve_shutdown() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    c.begin_window_close().unwrap();
    c.cancel_file_operation(token(&c)).unwrap();
    assert!(c.is_dirty().unwrap());
    c.begin_window_close().unwrap();
    c.decide_unsaved(token(&c), UnsavedDecision::Save).unwrap();
    let before = c.coordinator().snapshot();
    assert!(
        c.accept_file_path(token(&c), &f.path("missing/close.om9"))
            .is_err()
    );
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::SavePath);
    assert!(c.is_dirty().unwrap());
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel_file_operation(token(&c)).unwrap();
    c.begin_window_close().unwrap();
    c.decide_unsaved(token(&c), UnsavedDecision::Discard)
        .unwrap();
    let approved = token(&c);
    point(&mut c, "9,8,7");
    assert!(matches!(
        c.validate_file_operation(approved),
        Err(Error::Stale(_))
    ));
    c.cancel_file_operation(approved).unwrap();
    assert!(c.is_dirty().unwrap());
}
#[test]
fn save_clean_marker_tracks_undo_redo_and_different_same_revision_branch() {
    let f = Fixture::new();
    let mut c = start();
    assert!(!c.is_dirty().unwrap());
    point(&mut c, "1,2,3");
    assert!(c.is_dirty().unwrap());
    assert_eq!(save_as(&mut c, &f.path("saved.om9")), FileOutcome::Saved);
    assert!(!c.is_dirty().unwrap());
    let revision = c.document().revision;
    c.undo().unwrap();
    assert!(c.is_dirty().unwrap());
    c.redo().unwrap();
    assert!(!c.is_dirty().unwrap());
    c.undo().unwrap();
    point(&mut c, "9,8,7");
    assert_eq!(c.document().revision, revision);
    assert!(c.is_dirty().unwrap());
}
#[test]
fn first_save_and_subsequent_save_preserve_model_undo_and_redo() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    point(&mut c, "4,5,6");
    c.undo().unwrap();
    let before = c.coordinator().snapshot();
    assert_eq!(
        c.begin_file_operation(FileKind::Save).unwrap(),
        FileOutcome::Pending
    );
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::SavePath);
    assert_eq!(
        c.accept_file_path(token(&c), &f.path("first.om9")).unwrap(),
        FileOutcome::Saved
    );
    assert_eq!(c.coordinator().snapshot(), before);
    c.redo().unwrap();
    let before = c.coordinator().snapshot();
    assert_eq!(
        c.begin_file_operation(FileKind::Save).unwrap(),
        FileOutcome::Saved
    );
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(
        Archive::open(&f.path("first.om9"), &Limits::default()).unwrap(),
        *c.document()
    );
}
#[test]
fn save_as_cancel_failure_and_unsupported_format_preserve_identity_and_target() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    save_as(&mut c, &f.path("old.om9"));
    point(&mut c, "3,2,1");
    let path = c.active_file_path().unwrap().to_path_buf();
    let before = c.coordinator().snapshot();
    c.begin_file_operation(FileKind::SaveAs).unwrap();
    c.cancel_file_operation(token(&c)).unwrap();
    assert_eq!(c.active_file_path(), Some(path.as_path()));
    assert!(c.is_dirty().unwrap());
    c.begin_file_operation(FileKind::SaveAs).unwrap();
    let t = token(&c);
    let target = f.path("not-3dm.3dm");
    std::fs::write(&target, b"old target").unwrap();
    assert!(matches!(
        c.accept_file_path(t, &target),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(std::fs::read(&target).unwrap(), b"old target");
    let t = token(&c);
    assert!(
        c.accept_file_path(t, &f.path("missing-parent/file.om9"))
            .is_err()
    );
    assert_eq!(c.active_file_path(), Some(path.as_path()));
    assert!(c.is_dirty().unwrap());
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel_file_operation(token(&c)).unwrap();
}
#[test]
fn save_as_existing_destination_requires_explicit_overwrite_and_cancel_preserves_bytes() {
    let f = Fixture::new();
    let target = f.path("existing.om9");
    std::fs::write(&target, b"old bytes").unwrap();
    let mut c = start();
    point(&mut c, "1,2,3");
    assert_eq!(save_as(&mut c, &target), FileOutcome::Pending);
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::Overwrite);
    let t = token(&c);
    assert_eq!(
        c.confirm_file_overwrite(t, false).unwrap(),
        FileOutcome::Pending
    );
    assert_eq!(std::fs::read(&target).unwrap(), b"old bytes");
    assert!(c.active_file_path().is_none());
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::SavePath);
    assert_eq!(
        c.accept_file_path(token(&c), &target).unwrap(),
        FileOutcome::Pending
    );
    assert_eq!(
        c.confirm_file_overwrite(token(&c), true).unwrap(),
        FileOutcome::Saved
    );
    assert!(!c.is_dirty().unwrap());
    assert_eq!(
        Archive::open(&target, &Limits::default()).unwrap(),
        *c.document()
    );
}
#[test]
fn dirty_new_cancel_preserves_pending_line_and_discard_replaces_only_after_acceptance() {
    let mut c = start();
    point(&mut c, "1,2,3");
    c.submit("Line").unwrap();
    c.submit("0,0,0").unwrap();
    let before = c.coordinator().snapshot();
    let authority = c.coordinator().authority();
    let views = c.workspace.views.clone();
    c.begin_file_operation(FileKind::New).unwrap();
    assert_eq!(c.file_operation().unwrap().phase(), FilePhase::Unsaved);
    c.cancel_file_operation(token(&c)).unwrap();
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.workspace.views, views);
    assert!(c.is_running());
    assert_eq!(c.preview([2.; 3]).unwrap().unwrap().0, [0.; 3]);
    c.begin_file_operation(FileKind::New).unwrap();
    assert_eq!(
        c.decide_unsaved(token(&c), UnsavedDecision::Discard)
            .unwrap(),
        FileOutcome::Replaced
    );
    assert_ne!(c.coordinator().authority(), authority);
    assert!(c.document().entities.is_empty());
    assert!(!c.is_running());
    assert!(!c.is_dirty().unwrap());
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 0);
}
#[test]
fn new_save_failure_or_path_cancel_does_not_discard_old_document() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    let before = c.coordinator().snapshot();
    c.begin_file_operation(FileKind::New).unwrap();
    let t = token(&c);
    assert_eq!(
        c.decide_unsaved(t, UnsavedDecision::Save).unwrap(),
        FileOutcome::Pending
    );
    let t = token(&c);
    assert!(c.accept_file_path(t, &f.path("missing/file.om9")).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel_file_operation(token(&c)).unwrap();
    assert_eq!(c.coordinator().snapshot(), before);
    c.begin_file_operation(FileKind::New).unwrap();
    let t = token(&c);
    c.decide_unsaved(t, UnsavedDecision::Save).unwrap();
    let t = token(&c);
    assert_eq!(
        c.accept_file_path(t, &f.path("before-new.om9")).unwrap(),
        FileOutcome::SavedAndReplaced
    );
    assert_eq!(
        Archive::open(&f.path("before-new.om9"), &Limits::default()).unwrap(),
        *before.world.documents.values().next().unwrap().as_ref()
    );
    assert!(c.active_file_path().is_none());
}
#[test]
fn open_invalid_or_missing_candidate_keeps_document_scene_command_and_redo() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    point(&mut c, "4,5,6");
    c.undo().unwrap();
    c.submit("Line").unwrap();
    c.submit("0,0,0").unwrap();
    c.begin_file_operation(FileKind::Open).unwrap();
    let t = token(&c);
    c.decide_unsaved(t, UnsavedDecision::Discard).unwrap();
    let t = token(&c);
    let before = c.coordinator().snapshot();
    let scene = c.scene().unwrap().objects.len();
    let target = f.path("corrupt.om9");
    std::fs::write(&target, b"bad bytes").unwrap();
    assert!(c.accept_file_path(t, &target).is_err());
    assert!(
        c.accept_file_path(token(&c), &f.path("missing.om9"))
            .is_err()
    );
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.scene().unwrap().objects.len(), scene);
    assert!(c.is_running());
    c.cancel_file_operation(token(&c)).unwrap();
    c.cancel();
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}
#[test]
fn same_file_open_renews_runtime_authority_and_invalidates_old_modal_tokens() {
    use om9_app::layer_dialog::LayerDialogKind;
    use om9_core::layers::PresetSlot;
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    save_as(&mut c, &f.path("same.om9"));
    let docid = c.document().id;
    let authority = c.coordinator().authority();
    let owner = c.document().owner_session;
    let old = c
        .begin_layer_dialog(LayerDialogKind::Rename, Some(PresetSlot::new(17).unwrap()))
        .unwrap();
    c.cancel_layer_dialog(old).unwrap();
    c.begin_file_operation(FileKind::Open).unwrap();
    assert_eq!(
        c.accept_file_path(token(&c), &f.path("same.om9")).unwrap(),
        FileOutcome::Replaced
    );
    assert_eq!(c.document().id, docid);
    assert_ne!(c.document().owner_session, owner);
    assert_ne!(c.coordinator().authority(), authority);
    assert!(!c.is_dirty().unwrap());
    let new = c
        .begin_layer_dialog(LayerDialogKind::Rename, Some(PresetSlot::new(17).unwrap()))
        .unwrap();
    assert!(matches!(
        c.set_layer_dialog_name(old, "stale"),
        Err(Error::Stale(_))
    ));
    assert_eq!(c.layer_dialog().unwrap().token(), new);
}
#[test]
fn file_tokens_are_foreign_consumed_and_stale_after_undo_roundtrip() {
    let f = Fixture::new();
    let mut a = start();
    let mut b = start();
    a.begin_file_operation(FileKind::SaveAs).unwrap();
    let old = token(&a);
    b.begin_file_operation(FileKind::SaveAs).unwrap();
    assert!(matches!(
        b.accept_file_path(old, &f.path("foreign.om9")),
        Err(Error::Stale(_))
    ));
    a.cancel_file_operation(old).unwrap();
    a.begin_file_operation(FileKind::SaveAs).unwrap();
    let new = token(&a);
    assert!(matches!(a.cancel_file_operation(old), Err(Error::Stale(_))));
    assert_eq!(token(&a), new);
    point(&mut a, "1,2,3");
    a.undo().unwrap();
    assert!(matches!(
        a.accept_file_path(new, &f.path("stale.om9")),
        Err(Error::Stale(_))
    ));
    assert!(!f.path("stale.om9").exists());
    a.cancel_file_operation(new).unwrap();
}
#[test]
fn file_and_layer_dialogs_cannot_overlap_owners() {
    use om9_app::layer_dialog::LayerDialogKind;
    let mut c = start();
    let layer = c.begin_layer_dialog(LayerDialogKind::Show, None).unwrap();
    assert!(matches!(
        c.begin_file_operation(FileKind::Open),
        Err(Error::Busy)
    ));
    c.cancel_layer_dialog(layer).unwrap();
    c.begin_file_operation(FileKind::Open).unwrap();
    assert!(matches!(
        c.begin_layer_dialog(LayerDialogKind::Show, None),
        Err(Error::Busy)
    ));
}
#[test]
fn open_candidate_must_fit_alongside_old_world_scene_and_undo() {
    use om9_core::{
        geometry::{Cloud, Geometry},
        model::Entity,
        transaction::Cancellation,
    };
    let f = Fixture::new();
    let limits = Limits {
        max_bytes: 10_500_000,
        ..Limits::default()
    };
    let mut old = Document::new("held old owner");
    for i in 0..1000 {
        let e = Entity::new("x".repeat(8192), Geometry::Point([i as f64, 0., 0.]), None);
        old.entities.insert(e.id, e);
    }
    let mut candidate = Document::new("candidate alone fits");
    let e = Entity::new(
        "cloud",
        Geometry::Cloud(Cloud {
            points: vec![[1., 2., 3.]; 65537],
            normals: vec![],
            colors: vec![],
            hidden: vec![],
        }),
        None,
    );
    candidate.entities.insert(e.id, e);
    Controller::new(candidate.clone(), limits).expect("candidate alone must fit");
    let path = f.path("candidate.om9");
    Archive::save(
        &path,
        &candidate,
        &Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    let mut c = Controller::new(old, limits).expect("old alone must fit");
    let before = c.coordinator().snapshot();
    let views = c.workspace.views.clone();
    c.begin_file_operation(FileKind::Open).unwrap();
    let t = token(&c);
    assert!(matches!(
        c.accept_file_path(t, &path),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.workspace.views, views);
    assert!(c.active_file_path().is_none());
    assert_eq!(*c.coordinator().limits(), limits);
    c.cancel_file_operation(token(&c)).unwrap();
}
#[test]
fn missing_replay_module_is_diagnostic_and_not_a_clean_restore() {
    use om9_core::{
        geometry::{Geometry, Transform, Unit},
        history::{Recipe, ReplayEnvironment, Tolerance},
        model::Entity,
        transaction::Cancellation,
    };
    let f = Fixture::new();
    let mut d = Document::new("missing module");
    let e = Entity::new("output", Geometry::Point([1., 2., 3.]), None);
    let id = e.id;
    d.entities.insert(id, e);
    let recipe = Recipe::new(
        d.id,
        "OM9-UNAVAILABLE",
        vec![],
        vec![id],
        ReplayEnvironment {
            command_version: 1,
            solver: "missing-solver".into(),
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
    let path = f.path("missing-module.om9");
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    let mut c = start();
    c.begin_file_operation(FileKind::Open).unwrap();
    assert_eq!(
        c.accept_file_path(token(&c), &path).unwrap(),
        FileOutcome::Replaced
    );
    assert!(
        c.file_diagnostics()
            .iter()
            .any(|line| line.contains("Missing replay module missing-solver@1"))
    );
    assert!(c.is_dirty().unwrap());
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.document().entities.len(), 1);
}
#[test]
fn save_keeps_hidden_locked_off_layers_and_explicit_preferences_in_complete_document() {
    use om9_app::layer_command::{LayerChoice, LayerProperty};
    use om9_core::layers::PresetSlot;
    let f = Fixture::new();
    let mut c = start();
    let slot = PresetSlot::new(21).unwrap();
    c.set_current_layer(LayerChoice::Slot(slot)).unwrap();
    point(&mut c, "1,2,3");
    let source = c
        .document()
        .stamp(*c.document().entities.keys().next().unwrap())
        .unwrap();
    c.set_objects_hidden(vec![source], true).unwrap();
    c.set_layer_property(slot, LayerProperty::Locked(true))
        .unwrap();
    c.set_layer_property(slot, LayerProperty::Visible(false))
        .unwrap();
    let before = c.coordinator().snapshot();
    let path = f.path("complete.om9");
    assert_eq!(save_as(&mut c, &path), FileOutcome::Saved);
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(
        Archive::open(&path, &Limits::default()).unwrap(),
        *c.document()
    );
    assert!(
        c.document()
            .entities
            .values()
            .all(|entity| entity.attributes.hidden
                && entity.attributes.visible
                && entity.attributes.selectable)
    );
}
#[test]
fn consumed_save_picker_reply_cannot_become_a_later_open_selection() {
    let f = Fixture::new();
    let mut c = start();
    point(&mut c, "1,2,3");
    c.begin_file_operation(FileKind::Open).unwrap();
    c.decide_unsaved(token(&c), UnsavedDecision::Save).unwrap();
    let save_token = token(&c);
    let path = f.path("saved-before-open.om9");
    assert_eq!(
        c.accept_file_path(save_token, &path).unwrap(),
        FileOutcome::SavedPending
    );
    assert_eq!(c.file_save_receipt().unwrap().token(), save_token);
    let open_token = token(&c);
    let before = c.coordinator().snapshot();
    assert!(
        matches!(c.accept_file_path(save_token, &path), Err(Error::Stale(_))),
        "consumed Save reply must not act as Open"
    );
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(token(&c), open_token);
    assert!(matches!(
        c.cancel_file_operation(save_token),
        Err(Error::Stale(_))
    ));
    c.cancel_file_operation(open_token).unwrap();
}
#[test]
fn old_overwrite_confirmation_cannot_authorize_reselected_destination() {
    let f = Fixture::new();
    let first = f.path("first.om9");
    let second = f.path("second.om9");
    std::fs::write(&first, b"first old").unwrap();
    std::fs::write(&second, b"second old").unwrap();
    let mut c = start();
    point(&mut c, "1,2,3");
    c.begin_file_operation(FileKind::SaveAs).unwrap();
    c.accept_file_path(token(&c), &first).unwrap();
    let first_confirm = token(&c);
    c.confirm_file_overwrite(first_confirm, false).unwrap();
    c.accept_file_path(token(&c), &second).unwrap();
    let second_confirm = token(&c);
    assert!(
        matches!(
            c.confirm_file_overwrite(first_confirm, true),
            Err(Error::Stale(_))
        ),
        "old confirmation must not authorize another file"
    );
    assert_eq!(std::fs::read(&first).unwrap(), b"first old");
    assert_eq!(std::fs::read(&second).unwrap(), b"second old");
    assert_eq!(token(&c), second_confirm);
    c.cancel_file_operation(second_confirm).unwrap();
}
#[test]
fn rejected_picker_reply_is_consumed_even_if_the_path_later_becomes_valid() {
    let f = Fixture::new();
    let mut c = start();
    let before = c.coordinator().snapshot();
    c.begin_file_operation(FileKind::Open).unwrap();
    let old = token(&c);
    let path = f.path("later-valid.om9");
    assert!(c.accept_file_path(old, &path).is_err());
    om9_archive::Archive::save(
        &path,
        &Document::new("later file"),
        &Limits::default(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
    assert!(
        matches!(c.accept_file_path(old, &path), Err(Error::Stale(_))),
        "rejected reply must not be replayed as a new selection"
    );
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel_file_operation(token(&c)).unwrap();
}
#[test]
fn failing_save_to_active_path_does_not_turn_into_a_save_as_picker() {
    struct RestorePermissions(std::path::PathBuf, std::fs::Permissions);
    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            std::fs::set_permissions(&self.0, self.1.clone()).unwrap();
        }
    }
    let f = Fixture::new();
    let path = f.path("active.om9");
    let mut c = start();
    point(&mut c, "1,2,3");
    save_as(&mut c, &path);
    let bytes = std::fs::read(&path).unwrap();
    let original = std::fs::metadata(&path).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    std::fs::set_permissions(&path, readonly).unwrap();
    let _restore = RestorePermissions(path.clone(), original);
    point(&mut c, "4,5,6");
    let before = c.coordinator().snapshot();
    assert!(matches!(
        c.begin_file_operation(FileKind::Save),
        Err(Error::Permission(_))
    ));
    assert!(
        c.file_operation().is_none(),
        "failed known-path Save must not ask for another Save path"
    );
    assert_eq!(c.active_file_path(), Some(path.as_path()));
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(c.is_dirty().unwrap());
}
