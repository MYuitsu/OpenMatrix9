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
