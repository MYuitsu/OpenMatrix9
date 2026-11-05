// SPDX-License-Identifier: LGPL-2.1-or-later
//! Job Bag persistence inside `.om9` and live Master restore (OM9-L2-03-15).
use om9_app::{file_lifecycle::FileKind, ui::MatrixApp};
use om9_core::model::Document;
use std::path::PathBuf;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "om9-project-manager-{}.jobbags.om9",
            std::process::id()
        ));
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn add_point(app: &mut MatrixApp, x: f64) {
    app.controller.submit("Point").expect("Point");
    app.controller
        .submit(&format!("{x},0,0"))
        .expect("coordinates");
}

#[test]
fn staging_default_ui_state_preserves_document_and_redo() {
    let mut app = MatrixApp::new(Document::new("no implicit metadata")).unwrap();
    add_point(&mut app, 1.);
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    app.stage_project_state().unwrap();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}

#[test]
fn unchanged_embedded_ui_records_preserve_document_and_redo() {
    let mut app = MatrixApp::new(Document::new("stable embedded metadata")).unwrap();
    add_point(&mut app, 0.);
    let buffers = app.controller.scene().unwrap().objects[0].buffers.clone();
    app.project_manager
        .set_notes("Keep all four metadata resources")
        .unwrap();
    app.display_preferences.wireframe = false;
    app.controller
        .store_f6_state(app.f6_custom.to_json().unwrap().as_bytes())
        .unwrap();
    let pictures = om9_app::pictureframe::PictureFrameStore {
        version: 1,
        frames: vec![],
    };
    app.controller
        .store_pictureframes(pictures.to_json().unwrap().as_bytes())
        .unwrap();
    app.stage_project_state().unwrap();
    let state = app.controller.document_scene_state();
    assert_eq!(state.scene_epoch, Some(state.publication_epoch));
    assert!(std::sync::Arc::ptr_eq(
        &buffers,
        &app.controller.scene().unwrap().objects[0].buffers
    ));
    add_point(&mut app, 1.);
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    app.stage_project_state().unwrap();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}

#[test]
fn metadata_admission_failure_preserves_document_scene_and_redo() {
    let mut app = MatrixApp::new(Document::new("metadata pressure")).unwrap();
    add_point(&mut app, 0.);
    add_point(&mut app, 1.);
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    let state = app.controller.document_scene_state();
    let buffers = app.controller.scene().unwrap().objects[0].buffers.clone();
    let pressure = app
        .controller
        .coordinator()
        .reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,
            None,
            app.controller.scene_cache_usage().unwrap(),
        )
        .unwrap();
    let bytes = serde_json::to_vec(&app.display_preferences).unwrap();
    assert!(matches!(
        app.controller.store_display_state(&bytes),
        Err(om9_core::Error::Budget(_))
    ));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.document_scene_state(), state);
    assert!(std::sync::Arc::ptr_eq(
        &buffers,
        &app.controller.scene().unwrap().objects[0].buffers
    ));
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    drop(pressure);
    app.controller.store_display_state(&bytes).unwrap();
    let state = app.controller.document_scene_state();
    assert_eq!(state.scene_epoch, Some(state.publication_epoch));
    add_point(&mut app, 2.);
}

#[test]
fn jobbag_state_survives_save_and_reopen_inside_the_om9_file() {
    let fixture = Fixture::new();
    let mut app = MatrixApp::new(Document::new("Persisted jobbag")).expect("app");
    add_point(&mut app, 1.);
    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("point");
    app.controller.select_objects(&[id]).expect("select");
    app.project_manager
        .capture(
            0,
            app.controller
                .jobbag_selected_entities()
                .expect("selected entities"),
            app.controller.document(),
        )
        .expect("capture");
    app.project_manager
        .set_notes("Khách hàng: Nguyễn Văn A")
        .expect("notes");
    app.stage_project_state().expect("embed project state");
    let staged = app.controller.coordinator().snapshot();
    app.stage_project_state()
        .expect("unchanged state is a no-op");
    assert_eq!(app.controller.coordinator().snapshot(), staged);

    app.controller
        .begin_file_operation(FileKind::SaveAs)
        .expect("begin save");
    let token = app.controller.file_operation().expect("operation").token();
    let outcome = app
        .controller
        .accept_file_path(token, &fixture.0)
        .expect("save file");
    assert!(matches!(
        outcome,
        om9_app::file_lifecycle::FileOutcome::Saved
    ));

    let reopened = MatrixApp::from_file(&fixture.0).expect("reopen");
    assert_eq!(
        reopened.project_manager.active().notes,
        "Khách hàng: Nguyễn Văn A"
    );
    assert!(
        reopened
            .project_manager
            .slot_available(0, reopened.controller.document())
    );
    assert_eq!(
        reopened
            .project_manager
            .active()
            .slot(0)
            .and_then(|slot| slot.snapshot.as_ref())
            .map(|snapshot| snapshot.entities.len()),
        Some(1)
    );
}

#[test]
fn restore_master_live_replaces_the_document_atomically() {
    let mut app = MatrixApp::new(Document::new("Master restore")).expect("app");
    add_point(&mut app, 1.);
    app.project_manager
        .capture_master(app.controller.document())
        .expect("master");
    let captured = app.controller.document().entities.len();
    add_point(&mut app, 2.);
    assert_eq!(app.controller.document().entities.len(), captured + 1);
    app.restore_master_live().expect("restore master");
    assert_eq!(app.controller.document().entities.len(), captured);
}
