// SPDX-License-Identifier: LGPL-2.1-or-later
//! Project Manager and Job Bag surface checks for OM9-L2-03-15.
use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};

fn frame(app: &mut MatrixApp, context: &egui::Context) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn document_with_two_points() -> (Document, Vec<om9_core::identity::ObjectId>) {
    let mut document = Document::new("jobbag-ui");
    let mut ids = Vec::new();
    for index in 0..2 {
        let entity = Entity::new(
            format!("point {index}"),
            Geometry::Point([index as f64, 0., 0.]),
            None,
        );
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    (document, ids)
}

#[test]
fn projects_and_three_jobbag_slots_are_visible() {
    let mut app = MatrixApp::new(Document::new("jobbag-ui")).expect("app");
    let context = egui::Context::default();
    frame(&mut app, &context);
    assert!(!app.project_row_rects.is_empty());
    assert_eq!(app.jobbag_slot_rects.len(), 3);
    assert!(app.project_action_rects.contains_key("new-project"));
    assert!(app.project_action_rects.contains_key("capture-master"));
}

#[test]
fn capture_selection_then_restore_inserts_fresh_ids() {
    let (document, ids) = document_with_two_points();
    let mut app = MatrixApp::new(document).expect("app");
    app.controller.select_objects(&ids).expect("select");
    let context = egui::Context::default();
    frame(&mut app, &context);

    let selected = app
        .controller
        .jobbag_selected_entities()
        .expect("selected entities");
    app.project_manager
        .capture(0, selected, app.controller.document())
        .expect("capture");
    assert!(
        app.project_manager
            .slot_available(0, app.controller.document())
    );
    let original_count = app.controller.document().entities.len();
    let restored = app
        .project_manager
        .restore_entities(0)
        .expect("restore entities");
    let restored_ids = app
        .controller
        .insert_jobbag_entities(restored)
        .expect("insert transaction");
    assert_eq!(restored_ids.len(), 2);
    assert_eq!(app.controller.document().entities.len(), original_count + 2);
    for id in restored_ids {
        assert!(!ids.contains(&id));
    }
    frame(&mut app, &context);
    assert!(
        app.project_manager
            .slot_available(0, app.controller.document())
    );
}

#[test]
fn new_project_row_and_notes_keep_selection() {
    let mut app = MatrixApp::new(Document::new("jobbag-ui")).expect("app");
    let revision = app.controller.document().revision;
    let created = app
        .project_manager
        .create_project("Khách A", revision)
        .expect("create project");
    app.project_manager
        .set_notes("Nhẫn vàng 18K")
        .expect("notes");
    let context = egui::Context::default();
    frame(&mut app, &context);
    assert_eq!(app.project_manager.active().id, created);
    assert_eq!(app.project_manager.active().notes, "Nhẫn vàng 18K");
    assert!(app.project_row_rects.contains_key(&created));
}
