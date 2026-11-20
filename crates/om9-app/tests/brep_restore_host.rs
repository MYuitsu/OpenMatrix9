// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows, feature = "occt-brep", feature = "test-fixtures"))]

use om9_app::brep_restore_host::BrepRestoreHost;
use om9_core::{Error, budget::{Limits, Usage, memory::MemoryClass}, model::Document,
    transaction::{Cancellation, Mutation, TransactionCoordinator, TransactionRequest, World}};
use om9_kernel::brep::{box_brep, capture_source_with_shape_set};
use om9_native_owner::ProcessLimits;
use sha2::{Digest, Sha256};

#[test]
fn project_proof_rejects_replaced_or_missing_resource() {
    om9_app::session::Controller::test_project_proof_rejects_replaced_or_missing_resource();
}

#[test]
#[ignore = "requires configured BRep worker and offscreen GPU adapter"]
fn brep_numbered_and_named_thumbnails_render_from_verified_sources() {
    if option_env!("OM9_BREP_WORKER_SHA256").is_none() { panic!("pinned BRep worker required"); }
    use om9_app::{project_manager::BagFilter, ui::MatrixApp};
    fn frame(app: &mut MatrixApp, context: &eframe::egui::Context, events: Vec<eframe::egui::Event>) {
        use eframe::egui::{Pos2, RawInput, Rect, Vec2};
        let time = context.input(|input| input.time) + 1. / 60.;
        let mut output = context.run_ui(RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            time: Some(time), events, ..Default::default() }, |ui| app.show(ui));
        output.textures_delta.clear();
    }
    fn click(app: &mut MatrixApp, context: &eframe::egui::Context, pos: eframe::egui::Pos2) {
        use eframe::egui::{Event, Modifiers, PointerButton};
        frame(app, context, vec![Event::PointerMoved(pos), Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
        frame(app, context, vec![Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    }
    let mut app = MatrixApp::new(Document::new("BRep thumbnails")).unwrap();
    app.controller.submit("Box").unwrap(); app.controller.submit("0,0,0").unwrap();
    app.controller.submit("4,5,0").unwrap(); app.controller.submit("6").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let context = eframe::egui::Context::default();
    app.test_capture_filtered_jobbag(0, BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some(), "{}", app.status_line());
    app.test_start_named_capture("Parts", None).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.named_master_snapshot("Parts").is_some(), "{}", app.status_line());
    app.controller.delete_objects(&[id]).unwrap();
    assert!(app.controller.document().entities.is_empty());
    for _ in 0..2 {
        app.test_service_thumbnails(&context, true);
        assert!(app.test_thumbnail_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
        while app.test_thumbnail_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_thumbnails(&context, false);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    }
    let numbered = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.as_ref().unwrap();
    let named = app.project_manager.named_master_snapshot("Parts").unwrap().thumbnail_png.as_ref().unwrap();
    for png in [numbered, named] {
        let image = image::load_from_memory(png).unwrap().to_rgba8();
        assert_eq!((image.width(), image.height()), (96, 96));
        let first = image.get_pixel(0, 0);
        assert!(image.pixels().any(|pixel| pixel != first), "thumbnail must contain rendered geometry");
    }
    let temp_root = std::env::temp_dir().canonicalize().unwrap();
    let catalog_dir = temp_root.join(format!("om9-brep-preview-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&catalog_dir).unwrap();
    let catalog_dir = catalog_dir.canonicalize().unwrap();
    assert_eq!(catalog_dir.parent(), Some(temp_root.as_path()));
    let catalog = catalog_dir.join("catalog.json");
    om9_app::project_catalog::save(&catalog, &app.project_manager, None).unwrap();
    drop(app);
    let mut app = MatrixApp::new(Document::new("reloaded BRep preview")).unwrap();
    app.test_start_project_catalog_at(catalog).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    while app.test_project_catalog_load_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(!app.status.kind().is_failure(), "{}", app.status_line());
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.is_some());
    assert!(app.project_manager.named_master_snapshot("Parts").unwrap().thumbnail_png.is_some());
    let before_preview = app.controller.coordinator().snapshot();
    let undo_before = app.controller.coordinator().undo_len();
    frame(&mut app, &context, vec![]);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context, vec![]);
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context, vec![]);
    let files = app.project_action_rects["manager-files"].center();
    click(&mut app, &context, files);
    frame(&mut app, &context, vec![]);
    for _ in 0..40 { frame(&mut app, &context, vec![]); }
    let thumbnail = app.project_action_rects["manager-file-thumbnail"].center();
    click(&mut app, &context, thumbnail);
    frame(&mut app, &context, vec![]);
    assert!(app.test_manager_preview_is_numbered(0), "numbered thumbnail was not selected");
    assert_eq!(app.controller.coordinator().snapshot(), before_preview);
    assert_eq!(app.controller.coordinator().undo_len(), undo_before);
    let thumbnail = app.project_action_rects["manager-file-thumbnail"].center();
    click(&mut app, &context, thumbnail);
    assert!(app.test_capture_pending(), "double-click did not start BRep insert: {}", app.status_line());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.controller.document().entities.len(), 1, "{}", app.status_line());
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    let bag = app.project_manager.active_mut().slots[0].snapshot.as_mut().unwrap();
    let mut corrupt = bag.archive.as_ref().unwrap().to_vec();
    *corrupt.last_mut().unwrap() ^= 1;
    bag.archive = Some(corrupt.into());
    bag.thumbnail.png = None;
    let before = app.controller.coordinator().snapshot();
    let scene = app.controller.scene().unwrap().publication_id();
    app.test_service_thumbnails(&context, true);
    assert!(app.test_thumbnail_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    while app.test_thumbnail_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_thumbnails(&context, false);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(false));
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.scene().unwrap().publication_id(), scene);
    drop(app);
    std::fs::remove_dir_all(catalog_dir).unwrap();
}

#[test]
fn named_brep_snapshots_restore_all_groups_with_unsaved_lifecycle() {
    if option_env!("OM9_BREP_WORKER_SHA256").is_none() { return; }
    use om9_app::{file_lifecycle::{FilePhase, UnsavedDecision}, ui::MatrixApp};
    struct Temp(std::path::PathBuf);
    impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let mut app = MatrixApp::new(Document::new("named BRep source")).unwrap();
    app.controller.submit("Box").unwrap(); app.controller.submit("0,0,0").unwrap();
    app.controller.submit("4,5,0").unwrap(); app.controller.submit("6").unwrap();
    let object_id = *app.controller.document().entities.keys().next().unwrap();
    let original = app.controller.document().entities[&object_id].geometry.clone();
    let context = eframe::egui::Context::default();
    let names = ["Master", "Creation", "Parts", "Render", "Output"];
    for name in names {
        app.test_start_named_capture(name, None).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(app.project_manager.named_master_snapshot(name).is_some(), "{}", app.status_line());
    }
    let named_export = Temp(std::env::temp_dir().join(format!("om9-named-brep-export-{}.om9", uuid::Uuid::new_v4())));
    app.test_export_named_jobbag("Parts", &named_export.0).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(named_export.0.exists(), "{}", app.status_line());
    let mut exported = om9_app::session::Controller::new(Document::new("named export"), Default::default()).unwrap();
    exported.begin_file_operation(om9_app::file_lifecycle::FileKind::Open).unwrap();
    let token = exported.file_operation().unwrap().token();
    assert_eq!(exported.accept_file_path(token, &named_export.0).unwrap(), om9_app::file_lifecycle::FileOutcome::Replaced);
    assert_eq!(exported.document().entities.values().next().unwrap().geometry, original);
    drop(exported);
    let id = app.project_manager.named_master_snapshot("Master").unwrap().id;
    app.test_begin_named_open("Master", id).unwrap();
    if app.controller.file_operation().unwrap().phase() == FilePhase::Unsaved {
        let token = app.controller.file_operation().unwrap().token();
        app.controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    }
    app.test_service_named_open(&context);
    assert!(app.test_capture_pending());
    let before = app.controller.coordinator().snapshot();
    app.project_manager.set_notes("changed during open").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_named_open(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.project_manager.active().notes, "changed during open");
    app.test_begin_named_open("Master", id).unwrap();
    if app.controller.file_operation().unwrap().phase() == FilePhase::Unsaved {
        let token = app.controller.file_operation().unwrap().token();
        app.controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    }
    app.test_service_named_open(&context);
    assert!(app.test_capture_pending());
    let before_cancel = app.controller.coordinator().snapshot();
    let scene_before_cancel = app.controller.scene().unwrap().publication_id();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !app.test_named_open_result_ready() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.test_cancel_capture();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_named_open(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.coordinator().snapshot(), before_cancel);
    assert_eq!(app.controller.scene().unwrap().publication_id(), scene_before_cancel);
    app.controller.clear_selection();
    app.test_begin_named_open("Master", id).unwrap();
    if app.controller.file_operation().unwrap().phase() == FilePhase::Unsaved {
        let token = app.controller.file_operation().unwrap().token();
        app.controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    }
    app.test_service_named_open(&context);
    assert!(app.test_capture_pending());
    let before_selection = app.controller.coordinator().snapshot();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !app.test_named_open_result_ready() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.controller.select_objects(&[object_id]).unwrap();
    let selection_revision = app.controller.selection_state().revision;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_named_open(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.coordinator().snapshot(), before_selection);
    assert_eq!(app.controller.selection_state().revision, selection_revision);
    for name in names {
        app.controller.submit("Point").unwrap(); app.controller.submit("8,9,10").unwrap();
        let id = app.project_manager.named_master_snapshot(name).unwrap().id;
        app.test_begin_named_open(name, id).unwrap();
        if app.controller.file_operation().unwrap().phase() == FilePhase::Unsaved {
            let token = app.controller.file_operation().unwrap().token();
            app.controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
        }
        app.test_service_named_open(&context);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline, "{name}");
            app.test_service_named_open(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.document().entities.len(), 1, "{name}: {}", app.status_line());
        let restored = &app.controller.document().entities.values().next().unwrap().geometry;
        assert_eq!(restored, &original, "{name}");
        assert_eq!(app.project_manager.active().notes, "changed during open");
    }
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    let package = Temp(std::env::temp_dir().join(format!("om9-named-brep-import-{}.om9project", uuid::Uuid::new_v4())));
    let bytes = om9_app::project_catalog::encode_project(&app.project_manager, app.project_manager.active().id).unwrap();
    std::fs::write(&package.0, bytes).unwrap();
    let mut imported = MatrixApp::new(Document::new("named import destination")).unwrap();
    imported.test_start_project_import(vec![package.0.clone()]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while imported.test_project_import_pending() {
        assert!(std::time::Instant::now() < deadline);
        imported.test_service_project_import(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!imported.status.kind().is_failure(), "{}", imported.status_line());
    for name in names { assert!(imported.project_manager.named_master_snapshot(name).is_some(), "{name}"); }
    let imported_id = imported.project_manager.named_master_snapshot("Parts").unwrap().id;
    imported.test_begin_named_open("Parts", imported_id).unwrap();
    imported.test_service_named_open(&context);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while imported.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        imported.test_service_named_open(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(imported.controller.document().entities.len(), 1, "{}", imported.status_line());
    assert_eq!(imported.controller.document().entities.values().next().unwrap().geometry, original);
    drop(imported);
    let path = Temp(std::env::temp_dir().join(format!("om9-named-brep-save-{}.om9", uuid::Uuid::new_v4())));
    app.controller.submit("Point").unwrap(); app.controller.submit("8,9,10").unwrap();
    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &path.0).unwrap(), FileOutcome::Pending);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let saved = loop {
        if let Some(result) = app.controller.poll_file_save_worker() { break result.unwrap(); }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(saved, FileOutcome::Saved);
    let mut reopened = om9_app::session::Controller::new(Document::new("reopen named"), Default::default()).unwrap();
    reopened.begin_file_operation(FileKind::Open).unwrap();
    let token = reopened.file_operation().unwrap().token();
    assert_eq!(reopened.accept_file_path(token, &path.0).unwrap(), FileOutcome::Replaced);
    assert_eq!(reopened.document().entities.len(), 2);
    assert!(reopened.document().entities.values().any(|entity| entity.geometry == original));
}

#[test]
fn project_jobbag_captures_and_restores_a_box_after_source_deletion() {
    if option_env!("OM9_BREP_WORKER_SHA256").is_none() { return; }
    use om9_app::{project_manager::BagFilter, ui::MatrixApp};
    let mut app = MatrixApp::new(Document::new("source project")).unwrap();
    app.controller.submit("Box").unwrap(); app.controller.submit("0,0,0").unwrap();
    app.controller.submit("4,5,0").unwrap(); app.controller.submit("6").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.numeric_move([3., 4., 5.]).unwrap();
    let context = eframe::egui::Context::default();
    app.test_capture_filtered_jobbag(0, BagFilter::All).unwrap();
    app.test_cancel_capture();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_none());
    app.test_capture_filtered_jobbag(0, BagFilter::All).unwrap();
    app.controller.numeric_move([1., 0., 0.]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_none());
    let original = app.controller.document().entities[&id].clone();
    app.test_capture_filtered_jobbag(0, BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let snapshot = app.project_manager.active().slot(0).unwrap().snapshot.as_ref()
        .unwrap_or_else(|| panic!("{}", app.status_line()));
    let archive = snapshot.archive.as_ref().unwrap();
    assert!(archive.starts_with(b"OM9\0\r\n\x1a\n\x06\0\0\0"));
    let mut corrupt_archive = archive.to_vec();
    *corrupt_archive.last_mut().unwrap() ^= 1;
    let temp_root = std::env::temp_dir().canonicalize().unwrap();
    let catalog_dir = temp_root.join(format!("om9-brep-bag-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&catalog_dir).unwrap();
    let catalog_dir = catalog_dir.canonicalize().unwrap();
    assert!(catalog_dir.starts_with(&temp_root) && catalog_dir != temp_root);
    let catalog = catalog_dir.join("catalog.json");
    let bag_export = catalog_dir.join("numbered.om9");
    std::fs::write(&bag_export, b"existing destination").unwrap();
    let mut corrupt_manager = om9_app::project_manager::ProjectManager::new("corrupt export");
    corrupt_manager.capture_archive(0, vec![original.clone()], app.controller.document(), corrupt_archive).unwrap();
    let mut corrupt_app = MatrixApp::new(Document::new("corrupt export target")).unwrap();
    corrupt_app.project_manager = corrupt_manager;
    corrupt_app.test_export_jobbag(0, &bag_export).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while corrupt_app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline);
        corrupt_app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(corrupt_app.status.kind().is_failure());
    assert_eq!(std::fs::read(&bag_export).unwrap(), b"existing destination");
    drop(corrupt_app);
    app.test_export_jobbag(0, &bag_export).unwrap();
    app.test_cancel_project_export();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(std::fs::read(&bag_export).unwrap(), b"existing destination");
    app.status.acknowledge();
    app.test_export_jobbag(0, &bag_export).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let mut exported = om9_app::session::Controller::new(Document::new("numbered export"), Default::default()).unwrap();
    exported.begin_file_operation(om9_app::file_lifecycle::FileKind::Open).unwrap();
    let token = exported.file_operation().unwrap().token();
    assert_eq!(exported.accept_file_path(token, &bag_export).unwrap(), om9_app::file_lifecycle::FileOutcome::Replaced);
    assert_eq!(exported.document().entities.len(), 1, "{}", app.status_line());
    assert_eq!(exported.document().entities.values().next().unwrap().geometry, original.geometry);
    drop(exported);
    om9_app::project_catalog::save(&catalog, &app.project_manager, None).unwrap();
    let package = catalog_dir.join("export.om9project");
    let bytes = om9_app::project_catalog::encode_project(&app.project_manager, app.project_manager.active().id).unwrap();
    std::fs::write(&package, bytes).unwrap();
    app.status.acknowledge();
    app.controller.store_project_manager(&app.project_manager).unwrap();
    app.test_start_project_catalog_at(catalog.clone()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_catalog_load_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!app.status.kind().is_failure(), "{}", app.status_line());
    let previous_rows = app.project_manager.rows().len();
    app.test_start_embedded_project_import().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_import_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_import(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!app.status.kind().is_failure(), "{}", app.status_line());
    assert_eq!(app.project_manager.rows().len(), previous_rows + 1);
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some());
    app.controller.delete_objects(&[id]).unwrap();
    assert!(app.controller.document().entities.is_empty());
    drop(app);
    let mut app = MatrixApp::new(Document::new("different document")).unwrap();
    app.test_start_project_catalog_at(catalog.clone()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_catalog_load_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!app.status.kind().is_failure(), "{}", app.status_line());
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some());
    app.test_insert_filtered_jobbag(0, BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.document().entities.len(), 1, "{}", app.status_line());
    let restored = app.controller.document().entities.values().next().unwrap();
    assert_ne!(restored.id, id);
    assert_eq!(restored.placement, original.placement);
    let (om9_core::geometry::Geometry::Brep(old), om9_core::geometry::Geometry::Brep(new)) = (&original.geometry, &restored.geometry) else { panic!("BRep source") };
    assert_eq!(old.source(), new.source());
    assert_eq!(old.topology(), new.topology());
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    drop(app);
    let mut imported = MatrixApp::new(Document::new("import destination")).unwrap();
    imported.test_start_project_import(vec![package.clone()]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while imported.test_project_import_pending() {
        assert!(std::time::Instant::now() < deadline);
        imported.test_service_project_import(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!imported.status.kind().is_failure(), "{}", imported.status_line());
    assert!(imported.project_manager.active().slot(0).unwrap().snapshot.is_some());
    imported.test_insert_filtered_jobbag(0, BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while imported.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        imported.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(imported.controller.document().entities.len(), 1, "{}", imported.status_line());
    let mut populated = Document::new("populated destination");
    for index in 0..1024 {
        let entity = om9_core::model::Entity::new("existing point",
            om9_core::geometry::Geometry::Point([index as f64, 0., 0.]), None);
        populated.entities.insert(entity.id, entity);
    }
    let mut populated_app = MatrixApp::new(populated).unwrap();
    populated_app.project_manager = imported.project_manager.clone();
    let existing: Vec<_> = populated_app.controller.document().entities.keys().copied().collect();
    populated_app.test_insert_filtered_jobbag(0, BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while populated_app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline);
        populated_app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(populated_app.controller.document().entities.len(), 1025, "{}", populated_app.status_line());
    assert!(existing.iter().all(|id| populated_app.controller.document().entities.contains_key(id)));
    populated_app.controller.undo().unwrap();
    assert_eq!(populated_app.controller.document().entities.len(), 1024);
    populated_app.controller.redo().unwrap();
    assert_eq!(populated_app.controller.document().entities.len(), 1025);
    drop(populated_app);
    drop(imported);
    let invalid = catalog_dir.join("invalid.om9project");
    let mut bytes = std::fs::read(&package).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    std::fs::write(&invalid, bytes).unwrap();
    let mut rejected = MatrixApp::new(Document::new("batch destination")).unwrap();
    let before = rejected.project_manager.to_json().unwrap();
    rejected.test_start_project_import(vec![package, invalid]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while rejected.test_project_import_pending() {
        assert!(std::time::Instant::now() < deadline);
        rejected.test_service_project_import(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(rejected.status.kind().is_failure());
    assert_eq!(rejected.project_manager.to_json().unwrap(), before);
    drop(rejected);
    std::fs::remove_dir_all(catalog_dir).unwrap();
}

#[test]
fn controller_discovers_box_source_and_prepares_save_on_a_worker() {
    use om9_app::session::Controller;
    let limits = Limits { max_bytes: 1024 * 1024 * 1024, ..Limits::default() };
    let mut controller = Controller::new(Document::new("live box"), limits).unwrap();
    controller.submit("Box").unwrap();
    controller.submit("0,0,0").unwrap();
    controller.submit("4,5,0").unwrap();
    controller.submit("6").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    let before = controller.coordinator().snapshot();
    let capture = controller.capture_brep_save_sources(Usage::default()).unwrap();
    let worker = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("om9-brep-restore-worker.exe");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&worker).unwrap()).into();
    struct Temp(std::path::PathBuf);
    impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let path = Temp(std::env::temp_dir().join(format!("om9-controller-brep-save-{}.om9", uuid::Uuid::new_v4())));
    std::fs::write(&path.0, b"existing file").unwrap();
    let worker_path = path.0.clone();
    let job = std::thread::spawn(move || {
        let host = BrepRestoreHost::admit(&worker, digest,
            ProcessLimits { native_committed_bytes: 256 * 1024 * 1024, ..Default::default() }).unwrap();
        capture.prepare(&host, &worker_path, &Cancellation::default())
    });
    let prepared = job.join().unwrap().unwrap();
    assert_eq!(std::fs::read(&path.0).unwrap(), b"existing file");
    assert_eq!(controller.coordinator().snapshot(), before);
    prepared.publish(controller.coordinator(), &Cancellation::default()).unwrap();
    assert!(std::fs::read(&path.0).unwrap().starts_with(b"OM9\0\r\n\x1a\n\x06\0\0\0"));
    assert_eq!(controller.coordinator().snapshot(), before);

    // Save As follows the ordinary controller lifecycle and publishes only
    // after its asynchronous preparation is accepted.
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    assert_eq!(controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = controller.file_operation().unwrap().token();
    assert_eq!(controller.accept_file_path(token, &path.0).unwrap(), FileOutcome::Pending);
    let token = controller.file_operation().unwrap().token();
    assert_eq!(controller.confirm_file_overwrite(token, true).unwrap(), FileOutcome::Pending);
    assert!(controller.file_save_worker_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let saved = loop {
        if let Some(result) = controller.poll_file_save_worker() { break result.unwrap(); }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(saved, FileOutcome::Saved);
    assert!(!controller.is_dirty().unwrap());
    assert_eq!(controller.document().id, before.world.documents.values().next().unwrap().id);
    let accepted_bytes = std::fs::read(&path.0).unwrap();
    controller.begin_file_operation(FileKind::SaveAs).unwrap();
    let token = controller.file_operation().unwrap().token();
    controller.accept_file_path(token, &path.0).unwrap();
    let token = controller.file_operation().unwrap().token();
    controller.confirm_file_overwrite(token, true).unwrap();
    controller.undo().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let stale = loop {
        if let Some(result) = controller.poll_file_save_worker() { break result; }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert!(matches!(stale, Err(Error::Stale(_))));
    assert_eq!(std::fs::read(&path.0).unwrap(), accepted_bytes);
    controller.redo().unwrap();
    controller.begin_file_operation(FileKind::SaveAs).unwrap();
    let token = controller.file_operation().unwrap().token();
    controller.accept_file_path(token, &path.0).unwrap();
    let token = controller.file_operation().unwrap().token();
    controller.confirm_file_overwrite(token, true).unwrap();
    let token = controller.file_operation().unwrap().token();
    assert!(controller.file_save_worker_pending());
    controller.cancel_file_operation(token).unwrap();
    assert!(!controller.file_save_worker_pending());
    assert_eq!(std::fs::read(&path.0).unwrap(), accepted_bytes);
    assert!(!controller.is_dirty().unwrap());
    if option_env!("OM9_BREP_WORKER_SHA256").is_some() {
        let mut reopened = Controller::new(Document::new("before reopen"), limits).unwrap();
        reopened.begin_file_operation(FileKind::Open).unwrap();
        let token = reopened.file_operation().unwrap().token();
        assert_eq!(reopened.accept_file_path(token, &path.0).unwrap(), FileOutcome::Replaced);
        assert_eq!(reopened.document().id, controller.document().id);
        assert_eq!(reopened.document().entities.len(), 1);
        let original = reopened.document().entities.values().find_map(|entity| match &entity.geometry {
            om9_core::geometry::Geometry::Brep(source) => Some((source.source().to_owned(), source.topology().clone())),
            _ => None,
        }).unwrap();
        reopened.submit("Point").unwrap(); reopened.submit("8,9,10").unwrap();
        assert_eq!(reopened.begin_file_operation(FileKind::Save).unwrap(), FileOutcome::Pending);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let cached_save = loop {
            if let Some(result) = reopened.poll_file_save_worker() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(cached_save, FileOutcome::Saved);
        assert!(!reopened.is_dirty().unwrap());
        reopened.submit("Box").unwrap(); reopened.submit("20,0,0").unwrap();
        reopened.submit("22,3,0").unwrap(); reopened.submit("4").unwrap();
        assert!(reopened.is_dirty().unwrap());
        assert_eq!(reopened.begin_file_operation(FileKind::Save).unwrap(), FileOutcome::Pending);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let saved = loop {
            if let Some(result) = reopened.poll_file_save_worker() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(saved, FileOutcome::Saved);
        assert!(!reopened.is_dirty().unwrap());
        let mut again = Controller::new(Document::new("second reopen"), limits).unwrap();
        again.begin_file_operation(FileKind::Open).unwrap();
        let token = again.file_operation().unwrap().token();
        assert_eq!(again.accept_file_path(token, &path.0).unwrap(), FileOutcome::Replaced);
        assert_eq!(again.document().entities.len(), 3);
        let restored = again.document().entities.values().find_map(|entity| match &entity.geometry {
            om9_core::geometry::Geometry::Brep(source) if source.source() == original.0 => Some(source), _ => None,
        }).unwrap();
        assert_eq!(restored.source(), original.0);
        assert_eq!(restored.topology(), &original.1);
        let old_id = again.document().entities.values().find_map(|entity| match &entity.geometry {
            om9_core::geometry::Geometry::Brep(source) if source.source() == original.0 => Some(entity.id), _ => None,
        }).unwrap();
        again.delete_objects(&[old_id]).unwrap();
        assert_eq!(again.begin_file_operation(FileKind::Save).unwrap(), FileOutcome::Pending);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let pruned_save = loop {
            if let Some(result) = again.poll_file_save_worker() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(pruned_save, FileOutcome::Saved);
        let mut final_open = Controller::new(Document::new("after deletion"), limits).unwrap();
        final_open.begin_file_operation(FileKind::Open).unwrap();
        let token = final_open.file_operation().unwrap().token();
        assert_eq!(final_open.accept_file_path(token, &path.0).unwrap(), FileOutcome::Replaced);
        assert_eq!(final_open.document().entities.len(), 2);
        assert!(!final_open.document().entities.contains_key(&old_id));
    }
}

#[test]
fn attested_source_capture_saves_a_shared_document_atomically() {
    use om9_core::{geometry::Geometry, model::Entity};
    use om9_kernel::brep::{RegisteredTopologyProof, SharedTopologyProof, capture_source_topology};
    let worker = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("om9-brep-restore-worker.exe");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&worker).unwrap()).into();
    let host = BrepRestoreHost::admit(&worker, digest,
        ProcessLimits { native_committed_bytes: 256 * 1024 * 1024, ..Default::default() }).unwrap();
    let limits = Limits { max_bytes: 1024 * 1024 * 1024, ..Limits::default() };
    let document = Document::new("attested source");
    let document_id = document.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let physical = Some((8 * 1024 * 1024 * 1024, 6 * 1024 * 1024 * 1024));
    let seed = box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let snapshot = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
    let (body, proof) = snapshot.into_parts();
    let proof_bytes = proof.storage_bytes().unwrap();
    let mut proof_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, proof_bytes, physical, Usage::default()).unwrap();
    proof_owner.set_allocated_bytes(proof_bytes).unwrap();
    let body_bytes = body.storage_usage(&limits).unwrap().bytes;
    let mut body_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, body_bytes, physical, Usage::default()).unwrap();
    body_owner.set_allocated_bytes(body_bytes).unwrap();
    let canonical = coordinator.register_reserved_brep_candidate(body, &mut body_owner, Usage::default()).unwrap();
    let entity = Entity::new("box", Geometry::Brep(canonical), None);
    let object = entity.id;
    let prepared = coordinator.prepare_with_reserved_brep_insert(TransactionRequest::new("insert", vec![Mutation::Insert { document: document_id, entity }]), Usage::default(), &mut body_owner).unwrap();
    coordinator.commit(prepared).unwrap();
    drop(body_owner);
    let stamp = coordinator.world().documents[&document_id].stamp(object).unwrap();
    let mut scratch = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 8 * 1024 * 1024, physical, Usage::default()).unwrap();
    let registered = RegisteredTopologyProof::adopt(&coordinator, stamp, proof, proof_owner, |live, pending| {
        if live.checked_add(pending).ok_or(Error::Overflow("proof scratch"))? > scratch.bytes() { return Err(Error::Budget("proof scratch")); }
        scratch.set_allocated_bytes(live)?; scratch.recheck_pending()
    }).unwrap();
    drop(scratch);
    let owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, SharedTopologyProof::allocation_bytes(), physical, Usage::default()).unwrap();
    let proof = SharedTopologyProof::new(registered, &coordinator, owner).unwrap();
    let source = proof.capture_source(&coordinator).unwrap();
    let mut saved = coordinator.world().documents[&document_id].as_ref().clone();
    let geometry = saved.entities[&object].geometry.clone();
    let definition = om9_core::model::Definition::new("shared source", vec![Entity::new("member", geometry.clone(), None)]);
    let definition_id = definition.id;
    saved.definitions.insert(definition_id, definition);
    let instance = saved.instance(definition_id, om9_core::geometry::Transform::translation([3., 4., 5.]), None).unwrap();
    let instance_id = instance.id;
    saved.entities.insert(instance_id, instance);
    saved.results.insert(object, om9_core::result::PublishedResult {
        operation: "fixture".into(), geometry_revision: saved.entities[&object].geometry_revision,
        geometry_snapshot: geometry, source_stamps: vec![],
        conversion: om9_core::result::ConversionReport::identity(om9_core::geometry::GeometryKind::Brep),
        recipe: None, status: om9_core::result::ResultStatus::Current,
    });
    saved.validate(&limits).unwrap();
    struct Temp(std::path::PathBuf);
    impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let path = Temp(std::env::temp_dir().join(format!("om9-proof-capture-save-{}.om9", uuid::Uuid::new_v4())));
    std::fs::write(&path.0, b"original").unwrap();
    let held = Usage { bytes: saved.validate(&limits).unwrap().bytes + seed.storage_usage(&limits).unwrap().bytes + 1024 * 1024, ..Usage::default() };
    let canceled = Cancellation::default(); canceled.cancel();
    assert!(matches!(host.capture_and_save_document_candidate(&coordinator, &path.0, &saved, std::slice::from_ref(&source), held, &canceled), Err(Error::Canceled)));
    assert_eq!(std::fs::read(&path.0).unwrap(), b"original");
    let prepared = host.capture_and_prepare_document_candidate(&coordinator, &path.0, &saved, std::slice::from_ref(&source), held, &Cancellation::default()).unwrap();
    assert_eq!(std::fs::read(&path.0).unwrap(), b"original", "preparation must not publish");
    assert!(matches!(prepared.publish(&coordinator, &canceled), Err(Error::Canceled)));
    assert_eq!(std::fs::read(&path.0).unwrap(), b"original");
    let prepared = host.capture_and_prepare_document_candidate(&coordinator, &path.0, &saved, std::slice::from_ref(&source), held, &Cancellation::default()).unwrap();
    prepared.publish(&coordinator, &Cancellation::default()).unwrap();
    let bytes = std::fs::read(&path.0).unwrap();
    let restored = host.restore_document(&coordinator, &bytes, false, 0.05, 0.3, held, &Cancellation::default()).unwrap();
    assert_eq!(restored.document(), &saved);
    assert_eq!(restored.candidates().len(), 1, "shared root, definition and result use one packet");
    assert_eq!(restored.document().entities[&instance_id].placement, om9_core::geometry::Transform::translation([3., 4., 5.]));
    drop(restored);
    let captured_context = coordinator.capture_context(None).unwrap().0;
    let worker_source = proof.capture_source(&coordinator).unwrap();
    let peer = coordinator.capture_transaction_preparation_peer(held).unwrap();
    let worker_document_bytes = saved.validate(&limits).unwrap().bytes.max(1);
    let mut worker_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, worker_document_bytes, physical, held).unwrap();
    let worker_document = saved.clone();
    worker_owner.set_allocated_bytes(worker_document_bytes).unwrap();
    let worker_path = path.0.clone();
    let worker_executable = worker.clone();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let worker_thread = std::thread::spawn(move || {
        let _owner = worker_owner;
        let worker_host = BrepRestoreHost::admit(&worker_executable, digest,
            ProcessLimits { native_committed_bytes: 256 * 1024 * 1024, ..Default::default() }).unwrap();
        let result = worker_host.capture_and_prepare_document_candidate(peer.coordinator(), &worker_path,
            &worker_document, std::slice::from_ref(&worker_source), held, &Cancellation::default());
        sender.send(result).unwrap();
    });
    let point = Entity::new("intervening edit", Geometry::Point([1., 2., 3.]), None);
    let edit = coordinator.prepare(TransactionRequest::new("edit", vec![Mutation::Insert { document: document_id, entity: point }])).unwrap();
    coordinator.commit(edit).unwrap();
    let pending = receiver.recv_timeout(std::time::Duration::from_secs(30)).unwrap().unwrap();
    worker_thread.join().unwrap();
    assert_ne!(coordinator.capture_context(None).unwrap().0, captured_context);
    assert!(matches!(pending.publish(&coordinator, &Cancellation::default()), Err(Error::Stale("BRep save context"))));
    assert!(matches!(host.capture_and_save_document_candidate(&coordinator, &path.0, &saved, std::slice::from_ref(&source), held, &Cancellation::default()), Err(Error::Stale(_))));
    assert_eq!(std::fs::read(&path.0).unwrap(), bytes);
}

#[test]
fn isolated_host_restores_attested_box_and_refuses_invalid_requests() {
    let executable = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap()
        .join("om9-brep-restore-worker.exe");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&executable).unwrap()).into();
    let process = ProcessLimits { native_committed_bytes: 256 * 1024 * 1024, ..Default::default() };
    assert!(BrepRestoreHost::admit(&executable, [0; 32], process.clone()).is_err());
    let application = executable.with_file_name("om9.exe");
    let fingerprint: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    assert!(matches!(BrepRestoreHost::test_admit_packaged_at(&application, None, process.clone()), Err(Error::Unsupported("packaged BRep worker fingerprint"))));
    assert!(matches!(BrepRestoreHost::test_admit_packaged_at(&application, Some(&fingerprint.to_uppercase()), process.clone()), Err(Error::Invalid("packaged BRep worker fingerprint"))));
    assert!(BrepRestoreHost::test_admit_packaged_at(&application, Some(&"0".repeat(64)), process.clone()).is_err());
    drop(BrepRestoreHost::test_admit_packaged_at(&application, Some(&fingerprint), process.clone()).unwrap());
    let host = BrepRestoreHost::admit(&executable, digest, process).unwrap();
    let limits = Limits { max_bytes: 1024 * 1024 * 1024, ..Limits::default() };
    let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("restore owner")], &limits).unwrap(), limits).unwrap();
    let seed = box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let source = capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_source();
    let held = Usage { bytes: seed.storage_usage(&limits).unwrap().bytes + source.storage_bytes().unwrap(), ..Usage::default() };
    let canceled = Cancellation::default();
    canceled.cancel();
    assert!(matches!(host.restore(&coordinator, source.source(), source.topology(), source.shape_set(), 0.05, 0.3, held, &canceled), Err(Error::Canceled)));
    let before = coordinator.snapshot();
    let baseline_cpu = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
    let pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
    assert!(matches!(host.restore(&coordinator, source.source(), source.topology(), source.shape_set(), 0.05, 0.3, held, &Cancellation::default()), Err(Error::Budget(_))));
    drop(pressure);
    let result = host.restore(&coordinator, source.source(), source.topology(), source.shape_set(), 0.05, 0.3, held, &Cancellation::default()).unwrap();
    assert_eq!(result.candidate().source(), source.source());
    let ids = |items: &[uuid::Uuid]| items.iter().copied().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids(&result.candidate().topology().faces), ids(&source.topology().faces));
    assert_eq!(ids(&result.candidate().topology().edges), ids(&source.topology().edges));
    assert_eq!(ids(&result.candidate().topology().vertex_ids), ids(&source.topology().vertex_ids));
    assert!(result.reserved_bytes() >= result.candidate().storage_usage(&limits).unwrap().bytes);
    assert_eq!(coordinator.snapshot(), before);
    drop(result);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), baseline_cpu);
    let invalid = b"OM9-SHAPESET 1\n6 12 8\ninvalid";
    assert!(host.restore(&coordinator, source.source(), source.topology(), invalid, 0.05, 0.3, held, &Cancellation::default()).is_err());
    let other = box_brep([2., 3., 7.], 0.1, &limits).unwrap();
    let other_held = Usage { bytes: held.bytes + other.storage_usage(&limits).unwrap().bytes, ..Usage::default() };
    assert!(host.restore(&coordinator, other.source(), source.topology(), source.shape_set(), 0.05, 0.3, other_held, &Cancellation::default()).is_err());
    drop(other);
    let tight_output = ProcessLimits { native_committed_bytes: 256 * 1024 * 1024, stdout_bytes: 32, ..Default::default() };
    let limited_host = BrepRestoreHost::admit(&executable, digest, tight_output).unwrap();
    assert!(limited_host.restore(&coordinator, source.source(), source.topology(), source.shape_set(), 0.05, 0.3, held, &Cancellation::default()).is_err());
    assert_eq!(coordinator.snapshot(), before);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), baseline_cpu);

    let document = Document::new("transaction target");
    let document_id = document.id;
    let mut target = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let stale = host.restore(&target, source.source(), source.topology(), source.shape_set(), 0.05, 0.3, held, &Cancellation::default()).unwrap();
    let point = om9_core::model::Entity::new("intervening edit", om9_core::geometry::Geometry::Point([1., 2., 3.]), None);
    let edit = target.prepare(TransactionRequest::new("edit", vec![Mutation::Insert { document: document_id, entity: point }])).unwrap();
    target.commit(edit).unwrap();
    let cpu_without_candidate = target.external_reserved_bytes(MemoryClass::CpuCache) - stale.reserved_bytes();
    assert!(matches!(stale.register(&target, held), Err(Error::Stale("BRep worker context"))));
    assert_eq!(target.external_reserved_bytes(MemoryClass::Kernel), 0);
    assert_eq!(target.external_reserved_bytes(MemoryClass::CpuCache), cpu_without_candidate);
    // Round-trip the source and packet through a bounded portable envelope.
    let encoded = om9_archive::canonical_brep::encode_packet_with_admission(
        &source.canonical_record(), source.shape_set(), &limits, |_, _| Ok(())).unwrap();
    assert!(matches!(om9_archive::canonical_brep::encode_packet_with_admission(
        &source.canonical_record(), source.shape_set(), &limits, |_, _| Err(Error::Canceled)), Err(Error::Canceled)));
    let held = Usage { bytes: held.bytes + encoded.capacity() as u64, ..held };
    let physical = Some((8 * 1024 * 1024 * 1024, 6 * 1024 * 1024 * 1024));
    let mut damaged = encoded.clone();
    *damaged.last_mut().unwrap() ^= 1;
    let damaged_held = Usage { bytes: held.bytes + damaged.capacity() as u64, ..held };
    let rejected = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, damaged_held).unwrap();
    assert!(matches!(om9_archive::canonical_brep::decode_packet_reserved(&damaged, &limits, &target.external_memory_account(), rejected, || false), Err(Error::Invalid("BRep packet checksum"))));
    damaged[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    let rejected = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, damaged_held).unwrap();
    assert!(om9_archive::canonical_brep::decode_packet_reserved(&damaged, &limits, &target.external_memory_account(), rejected, || false).is_err());
    drop(damaged);
    let decode_reservation = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, held).unwrap();
    let record = om9_archive::canonical_brep::decode_packet_reserved(&encoded, &limits, &target.external_memory_account(), decode_reservation, || false).unwrap();
    assert!(matches!(host.restore_packet(&coordinator, &record, 0.05, 0.3, held, &Cancellation::default()), Err(Error::Invalid("BRep record account"))));
    let candidate = host.restore_packet(&target, &record, 0.05, 0.3, held, &Cancellation::default()).unwrap();
    let (canonical, mut reservation) = candidate.register(&target, held).unwrap();
    let entity = om9_core::model::Entity::new("restored box", om9_core::geometry::Geometry::Brep(canonical), None);
    let id = entity.id;
    let prepared = target.prepare_with_reserved_brep_insert(TransactionRequest::new("restore", vec![Mutation::Insert { document: document_id, entity }]), held, &mut reservation).unwrap();
    target.commit(prepared).unwrap();
    drop(reservation);
    let om9_core::geometry::Geometry::Brep(restored) = &target.world().documents[&document_id].entities[&id].geometry else { panic!("expected BRep") };
    assert_eq!(restored.source(), source.source());
    assert!(target.brep_representation(restored).is_ok());
    target.undo().unwrap();
    assert!(!target.world().documents[&document_id].entities.contains_key(&id));
    target.redo().unwrap();
    assert!(target.world().documents[&document_id].entities.contains_key(&id));

    // Exercise the separate structural document format; ordinary v5 stays guarded.
    let doc = &target.world().documents[&document_id];
    assert!(om9_archive::Archive::encode(doc, &limits).is_err());
    assert!(om9_archive::canonical_brep::encode_document_candidate(doc, &[], &target.external_memory_account(), &limits, |_, _| Ok(())).is_err());
    let v6 = om9_archive::canonical_brep::encode_document_candidate(doc, std::slice::from_ref(&record), &target.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    struct TempArchive(std::path::PathBuf);
    impl Drop for TempArchive { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let path = TempArchive(std::env::temp_dir().join(format!("om9-brep-candidate-{}.om9", uuid::Uuid::new_v4())));
    std::fs::write(&path.0, &v6).unwrap();
    drop(v6);
    let v6 = std::fs::read(&path.0).unwrap();
    drop(path);
    assert!(om9_archive::Archive::decode(&v6, &limits).is_err());
    let mut corrupt = v6.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(om9_archive::canonical_brep::decode_document_candidate_with_admission(&corrupt, &limits, |_| Ok(())).is_err());
    drop(corrupt);
    let decoded = om9_archive::canonical_brep::decode_document_candidate_with_admission(&v6, &limits, |_| Ok(())).unwrap();
    assert_eq!(decoded.packets().len(), 1);
    let om9_core::geometry::Geometry::Brep(decoded_brep) = &decoded.document().entities[&id].geometry else { panic!("decoded BRep") };
    assert_eq!(decoded_brep.source(), source.source());
    let decode_held = Usage { bytes: held.bytes + v6.capacity() as u64 + decoded.document().validate(&limits).unwrap().bytes + 1024 * 1024, ..held };
    let receipt = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, decode_held).unwrap();
    let decoded_packet = om9_archive::canonical_brep::decode_packet_reserved(decoded.packets()[0].1, &limits, &target.external_memory_account(), receipt, || false).unwrap();
    assert_eq!(&decoded_packet.record().record().binding_fingerprint()[..16], &decoded.packets()[0].0);
    assert_eq!(decoded_packet.record().record().topology(), decoded_brep.topology());
    assert!(matches!(decoded.validate_packets(std::slice::from_ref(&record), &target.external_memory_account(), &limits, |_, _| Ok(())), Err(Error::Invalid("BRep document packet owner"))));
    decoded.validate_packets(std::slice::from_ref(&decoded_packet), &target.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let rebuilt = host.restore_packet(&target, &decoded_packet, 0.05, 0.3, decode_held, &Cancellation::default()).unwrap();
    assert_eq!(rebuilt.candidate().source(), source.source());
    drop(rebuilt);
    drop(decoded_packet);
    drop(decoded);
    drop(v6);

    let second_seed = box_brep([2., 3., 7.], 0.1, &limits).unwrap();
    let second_source = capture_source_with_shape_set(second_seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_source();
    let held = Usage { bytes: held.bytes + 1024 * 1024 + second_seed.storage_usage(&limits).unwrap().bytes
        + second_source.storage_bytes().unwrap(), ..held };
    let make_candidate = || host.restore_packet(&target, &record, 0.05, 0.3, held, &Cancellation::default()).unwrap();
    let make_second = || host.restore(&target, second_source.source(), second_source.topology(), second_source.shape_set(), 0.05, 0.3, held, &Cancellation::default()).unwrap();
    let entity = |name| om9_core::model::Entity::new(name, om9_core::geometry::Geometry::Point([0., 0., 0.]), None);
    let baseline = target.external_reserved_bytes(MemoryClass::CpuCache);
    let invalid = om9_core::model::Entity::new("invalid", om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Line { start: [0., 0., 0.], end: [1., 0., 0.], domain: [0., 1.] }), None);
    let failed = vec![(make_candidate(), entity("first")), (make_second(), invalid)];
    assert!(matches!(om9_app::brep_restore_host::prepare_brep_insertions(&target, document_id, "bad batch", failed, held), Err(Error::Invalid("BRep insertion placeholder"))));
    assert_eq!(target.external_reserved_bytes(MemoryClass::CpuCache), baseline);
    let first = entity("first"); let first_id = first.id;
    let second = entity("second"); let second_id = second.id;
    let inputs = vec![(make_candidate(), first), (make_second(), second)];
    let prepared = om9_app::brep_restore_host::prepare_brep_insertions(&target, document_id, "restore batch", inputs, held).unwrap();
    target.commit(prepared).unwrap();
    assert!(target.world().documents[&document_id].entities.contains_key(&first_id));
    assert!(target.world().documents[&document_id].entities.contains_key(&second_id));
    let om9_core::geometry::Geometry::Brep(first_brep) = &target.world().documents[&document_id].entities[&first_id].geometry else { panic!("first BRep") };
    let om9_core::geometry::Geometry::Brep(second_brep) = &target.world().documents[&document_id].entities[&second_id].geometry else { panic!("second BRep") };
    assert_eq!(first_brep.source(), source.source());
    assert_eq!(second_brep.source(), second_source.source());
    target.undo().unwrap();
    assert!(!target.world().documents[&document_id].entities.contains_key(&first_id));
    assert!(!target.world().documents[&document_id].entities.contains_key(&second_id));
    target.redo().unwrap();
    assert!(target.world().documents[&document_id].entities.contains_key(&first_id));
    assert!(target.world().documents[&document_id].entities.contains_key(&second_id));

    let encoded_second = om9_archive::canonical_brep::encode_packet_with_admission(
        &second_source.canonical_record(), second_source.shape_set(), &limits, |_, _| Ok(())).unwrap();
    let two_held = Usage { bytes: held.bytes + encoded_second.capacity() as u64 + 1024 * 1024, ..held };
    let second_guard = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, two_held).unwrap();
    let second_record = om9_archive::canonical_brep::decode_packet_reserved(&encoded_second, &limits, &target.external_memory_account(), second_guard, || false).unwrap();
    let records = [record, second_record];
    let two_doc = &target.world().documents[&document_id];
    let mut enriched = two_doc.as_ref().clone();
    let first_geometry = enriched.entities[&first_id].geometry.clone();
    let definition = om9_core::model::Definition::new("BRep definition", vec![om9_core::model::Entity::new("member", first_geometry.clone(), None)]);
    let definition_id = definition.id;
    enriched.definitions.insert(definition_id, definition);
    let mut instance = om9_core::model::Entity::new("instance", om9_core::geometry::Geometry::Instance(definition_id), None);
    instance.placement = om9_core::geometry::Transform::translation([3., 4., 5.]);
    let instance_id = instance.id;
    enriched.entities.insert(instance_id, instance);
    let archive_resource = om9_core::resource::Resource::from_bytes("archive resource", om9_core::resource::ResourceKind::OpaqueBinary, b"embedded resource", &limits).unwrap();
    let archive_resource_id = archive_resource.id;
    enriched.resources.insert(archive_resource_id, archive_resource);
    let geometry_revision = enriched.entities[&first_id].geometry_revision;
    enriched.results.insert(first_id, om9_core::result::PublishedResult {
        operation: "fixture".into(), geometry_revision, geometry_snapshot: first_geometry, source_stamps: vec![],
        conversion: om9_core::result::ConversionReport::identity(om9_core::geometry::GeometryKind::Brep),
        recipe: None, status: om9_core::result::ResultStatus::Current,
    });
    enriched.validate(&limits).unwrap();
    assert!(om9_archive::canonical_brep::encode_document_candidate(&enriched, &records[..1], &target.external_memory_account(), &limits, |_, _| Ok(())).is_err());
    let two_archive = om9_archive::canonical_brep::encode_document_candidate(&enriched, &records, &target.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let save_path = TempArchive(std::env::temp_dir().join(format!("om9-brep-verified-save-{}.om9", uuid::Uuid::new_v4())));
    std::fs::write(&save_path.0, b"existing destination").unwrap();
    let save_held = Usage { bytes: two_held.bytes + two_archive.capacity() as u64 + enriched.validate(&limits).unwrap().bytes + 1024 * 1024, ..two_held };
    let canceled = Cancellation::default(); canceled.cancel();
    assert!(matches!(host.save_document_candidate(&target, &save_path.0, &enriched, &records, save_held, &canceled), Err(Error::Canceled)));
    assert_eq!(std::fs::read(&save_path.0).unwrap(), b"existing destination");
    assert!(host.save_document_candidate(&target, &save_path.0, &enriched, &records[..1], save_held, &Cancellation::default()).is_err());
    assert_eq!(std::fs::read(&save_path.0).unwrap(), b"existing destination");
    // A structurally valid envelope with the wrong native box must not be
    // published even when both shapes have the same topology counts.
    let bad_envelope = om9_archive::canonical_brep::encode_packet_with_admission(
        &source.canonical_record(), second_source.shape_set(), &limits, |_, _| Ok(())).unwrap();
    let bad_held = Usage { bytes: save_held.bytes + bad_envelope.capacity() as u64, ..save_held };
    let first_guard = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, bad_held).unwrap();
    let second_guard = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, bad_held).unwrap();
    let bad_first = om9_archive::canonical_brep::decode_packet_reserved(&bad_envelope, &limits, &target.external_memory_account(), first_guard, || false).unwrap();
    let good_second = om9_archive::canonical_brep::decode_packet_reserved(&encoded_second, &limits, &target.external_memory_account(), second_guard, || false).unwrap();
    assert!(host.save_document_candidate(&target, &save_path.0, &enriched, &[bad_first, good_second], bad_held, &Cancellation::default()).is_err());
    assert_eq!(std::fs::read(&save_path.0).unwrap(), b"existing destination");
    drop(bad_envelope);
    host.save_document_candidate(&target, &save_path.0, &enriched, &records, save_held, &Cancellation::default()).unwrap();
    assert_eq!(std::fs::read(&save_path.0).unwrap(), two_archive);
    drop(save_path);

    // Restore the detached BRep document through the normal dependency-remap
    // paste path, including a translated block sharing one source.
    let mut bag_document = enriched.clone();
    bag_document.results.clear();
    let bag_bytes = om9_archive::canonical_brep::encode_document_candidate(&bag_document, &records,
        &target.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let mut manager = om9_app::project_manager::ProjectManager::new("bag fixture");
    manager.capture_archive_with_limits(0, bag_document.entities.values().cloned().collect(),
        &bag_document, bag_bytes.clone(), &limits).unwrap();
    let snapshot = manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().clone();
    let mut recipient = om9_app::session::Controller::new(Document::new("bag recipient"), limits).unwrap();
    let bag_held = Usage { bytes: save_held.bytes + bag_bytes.capacity() as u64 + bag_bytes.len() as u64 + bag_document.validate(&limits).unwrap().bytes,
        ..save_held };
    let mut bad_snapshot = snapshot.clone();
    bad_snapshot.captured_ids.pop();
    let unchanged = recipient.coordinator().snapshot();
    let scene = recipient.scene().unwrap().publication_id();
    let detached = host.restore_document(recipient.coordinator(), &bag_bytes, false, 0.05, 0.3, bag_held, &Cancellation::default()).unwrap();
    let bound = detached.bind(recipient.coordinator(), bag_held, &Cancellation::default()).unwrap();
    assert!(matches!(recipient.test_insert_bound_jobbag_document(bound, &bad_snapshot), Err(Error::Invalid("jobbag archive root set mismatch"))));
    assert_eq!(recipient.coordinator().snapshot(), unchanged);
    assert_eq!(recipient.scene().unwrap().publication_id(), scene);
    let detached = host.restore_document(recipient.coordinator(), &bag_bytes, false, 0.05, 0.3, bag_held, &Cancellation::default()).unwrap();
    let bound = detached.bind(recipient.coordinator(), bag_held, &Cancellation::default()).unwrap();
    let inserted = recipient.test_insert_bound_jobbag_document(bound, &snapshot).unwrap();
    assert_eq!(inserted.len(), bag_document.entities.len());
    assert!(recipient.document().entities.values().any(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Instance(_))
        && entity.placement == om9_core::geometry::Transform::translation([3., 4., 5.])));
    assert!(recipient.document().definitions.values().flat_map(|definition| &definition.members)
        .any(|member| matches!(member.geometry, om9_core::geometry::Geometry::Brep(_))));
    assert!(recipient.scene().unwrap().objects.iter().any(|object| object.has_pick_geometry()));
    recipient.undo().unwrap();
    assert!(inserted.iter().all(|id| !recipient.document().entities.contains_key(id)));
    recipient.redo().unwrap();
    assert!(inserted.iter().all(|id| recipient.document().entities.contains_key(id)));
    if option_env!("OM9_BREP_WORKER_SHA256").is_some() {
        let mut app = om9_app::ui::MatrixApp::new(Document::new("UI bag recipient")).unwrap();
        app.project_manager = manager.clone();
        app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
        assert!(app.test_capture_pending());
        let context = eframe::egui::Context::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.document().entities.len(), bag_document.entities.len(), "{}", app.status_line());
        assert!(app.controller.document().entities.values().any(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Instance(_))
            && entity.placement == om9_core::geometry::Transform::translation([3., 4., 5.])));
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.is_empty());
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document().entities.len(), bag_document.entities.len());
        app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
        app.controller.undo().unwrap();
        let stale_before = app.controller.coordinator().snapshot();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.coordinator().snapshot(), stale_before);
        assert!(app.controller.document().entities.is_empty());
        let filter = om9_app::project_manager::BagFilter::Polysurfaces;
        let expected = bag_document.entities.values().filter(|entity|
            filter.matches_in_document(entity, &bag_document, &limits).unwrap()).count();
        assert!(expected > 0 && expected < bag_document.entities.len());
        app.test_insert_filtered_jobbag(0, filter).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.document().entities.len(), expected, "{}", app.status_line());
        assert!(app.controller.document().entities.values().any(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Instance(_))
            && entity.placement == om9_core::geometry::Transform::translation([3., 4., 5.])));
        app.controller.undo().unwrap();
        assert!(app.controller.document().entities.is_empty());
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document().entities.len(), expected);
        let before_prepared = app.controller.coordinator().snapshot();
        app.test_insert_filtered_jobbag(0, filter).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !app.test_brep_insert_result_ready() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        app.project_manager.set_notes("changed after bag preparation").unwrap();
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.coordinator().snapshot(), before_prepared);
        assert_eq!(app.project_manager.active().notes, "changed after bag preparation");
        app.test_insert_filtered_jobbag(0, filter).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !app.test_brep_insert_result_ready() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        app.test_cancel_capture();
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.coordinator().snapshot(), before_prepared);
        let existing: Vec<_> = app.controller.document().entities.keys().copied().collect();
        app.test_insert_filtered_jobbag(0, filter).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.document().entities.len(), expected * 2, "{}", app.status_line());
        assert!(existing.iter().all(|id| app.controller.document().entities.contains_key(id)));
        assert!(existing.iter().all(|id| app.controller.scene().unwrap().objects.iter().any(|object| object.root == *id)));
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document().entities.len(), expected);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document().entities.len(), expected * 2);
        app.controller.undo().unwrap();
        let before_empty = app.controller.coordinator().snapshot();
        app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::Curves).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.coordinator().snapshot(), before_empty);
        app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
        app.test_cancel_capture();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.coordinator().snapshot(), before_empty);
    }
    if option_env!("OM9_BREP_WORKER_SHA256").is_some() {
        let mut mixed = bag_document.clone();
        mixed.definitions.get_mut(&definition_id).unwrap().members.push(
            om9_core::model::Entity::new("different kind", om9_core::geometry::Geometry::Point([0., 0., 0.]), None));
        let bytes = om9_archive::canonical_brep::encode_document_candidate(&mixed, &records,
            &target.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
        let mut manager = om9_app::project_manager::ProjectManager::new("mixed block");
        manager.capture_archive_with_limits(0, mixed.entities.values().cloned().collect(), &mixed, bytes, &limits).unwrap();
        let mut app = om9_app::ui::MatrixApp::new(Document::new("mixed target")).unwrap();
        app.project_manager = manager;
        app.controller.submit("Point").unwrap(); app.controller.submit("1,2,3").unwrap(); app.controller.undo().unwrap();
        let before = app.controller.coordinator().snapshot();
        let scene = app.controller.scene().unwrap().publication_id();
        app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::Polysurfaces).unwrap();
        let context = eframe::egui::Context::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline);
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.controller.scene().unwrap().publication_id(), scene);
        assert!(app.status.kind().is_failure());
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document().entities.len(), 1);
    }
    drop((recipient, bag_document, bag_bytes));
    drop(enriched);
    let two_decoded = om9_archive::canonical_brep::decode_document_candidate_with_admission(&two_archive, &limits, |_| Ok(())).unwrap();
    assert_eq!(two_decoded.packets().len(), 2);
    let two_decoded_held = Usage { bytes: two_held.bytes + two_archive.capacity() as u64 + two_decoded.document().validate(&limits).unwrap().bytes + 1024 * 1024, ..two_held };
    let mut decoded_records = Vec::new();
    for (_, bytes) in two_decoded.packets() {
        let guard = target.reserve_external_memory_with_held(MemoryClass::CpuCache, 16 * 1024 * 1024, physical, two_decoded_held).unwrap();
        decoded_records.push(om9_archive::canonical_brep::decode_packet_reserved(bytes, &limits, &target.external_memory_account(), guard, || false).unwrap());
    }
    two_decoded.validate_packets(&decoded_records, &target.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    drop(decoded_records);
    drop(two_decoded);
    let baseline = target.external_reserved_bytes(MemoryClass::CpuCache);
    let restored_document = host.restore_document(&target, &two_archive, false, 0.05, 0.3, two_held, &Cancellation::default()).unwrap();
    assert_eq!(restored_document.candidates().len(), 2);
    assert_eq!(restored_document.document().entities.len(), two_doc.entities.len() + 1);
    let candidate_sources: std::collections::BTreeSet<_> = restored_document.candidates().iter().map(|(_, candidate)| candidate.candidate().source()).collect();
    assert!(candidate_sources.contains(source.source()));
    assert!(candidate_sources.contains(second_source.source()));
    let bound = restored_document.bind(&target, two_held, &Cancellation::default()).unwrap();
    let bound_doc = bound.document();
    assert_eq!(bound_doc.resources[&archive_resource_id].payload().unwrap(), b"embedded resource");
    let om9_core::geometry::Geometry::Brep(root) = &bound_doc.entities[&first_id].geometry else { panic!("bound root") };
    let om9_core::geometry::Geometry::Brep(member) = &bound_doc.definitions[&definition_id].members[0].geometry else { panic!("bound definition") };
    let om9_core::geometry::Geometry::Brep(result) = &bound_doc.results[&first_id].geometry_snapshot else { panic!("bound result") };
    assert_eq!(root.identity(), member.identity());
    assert_eq!(root.identity(), result.identity());
    assert!(target.brep_representation(root).is_ok());
    let om9_core::geometry::Geometry::Brep(other) = &bound_doc.entities[&second_id].geometry else { panic!("bound second root") };
    assert_ne!(root.identity(), other.identity());
    assert!(target.brep_representation(other).is_ok());
    let mut replacement = TransactionCoordinator::new(World::new(vec![bound_doc.clone()], &limits).unwrap(), limits).unwrap();
    assert!(replacement.share_brep_representations_for_replacement(&target, two_held, || Ok(())).is_err());
    replacement.adopt_external_captures(&target).unwrap();
    assert!(matches!(replacement.share_brep_representations_for_replacement(&target, two_held, || Err(Error::Canceled)), Err(Error::Canceled)));
    replacement.share_brep_representations_for_replacement(&target, two_held, || Ok(())).unwrap();
    assert!(replacement.brep_representation(root).is_ok());
    assert!(replacement.brep_representation(other).is_ok());
    let original_epoch = target.epoch();
    let scene = bound.prepare_scene(&target, two_held, &Cancellation::default()).unwrap();
    assert!(scene.scene().objects.iter().any(|object| object.root == first_id && object.has_pick_geometry()));
    assert!(scene.scene().objects.iter().any(|object| object.root == second_id && object.has_pick_geometry()));
    assert!(scene.scene().objects.iter().any(|object| object.root == instance_id && object.has_pick_geometry()));
    assert_eq!(target.epoch(), original_epoch);
    assert!(!target.world().documents[&document_id].entities.contains_key(&instance_id));
    drop(scene);
    let replacement_document = replacement.world().documents[&document_id].clone();
    let scratch_bytes = om9_scene::scene::Scene::candidate_preflight_scratch(&replacement_document, &limits).unwrap().bytes.max(1);
    let scratch = replacement.reserve_external_memory_with_held(MemoryClass::CpuCache, scratch_bytes, physical, two_held).unwrap();
    let mut scene_owner = replacement.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical, two_held).unwrap();
    let replacement_scene = om9_scene::scene::Scene::capture_with_peak_and_representations(
        replacement_document, replacement.epoch(), &limits, replacement.representation_store(), |peak| {
            if peak > scene_owner.bytes() {
                let extra = peak - scene_owner.bytes();
                replacement.grow_external_memory_with_held(&mut scene_owner, extra, physical, two_held)?;
            }
            Ok(())
        }).unwrap();
    assert!(replacement_scene.objects.iter().any(|object| object.root == instance_id && object.has_pick_geometry()));
    drop(replacement_scene);
    drop((scratch, scene_owner, replacement));
    target.prune_orphaned_brep_representations();
    assert_eq!(target.external_reserved_bytes(MemoryClass::CpuCache), baseline);

    // Exercise the Controller replacement path, including the old owner's drop.
    let mut live = om9_app::session::Controller::new(Document::new("old scene"), limits).unwrap();
    live.submit("Point").unwrap(); live.submit("9,8,7").unwrap(); live.undo().unwrap();
    let before = live.coordinator().snapshot();
    let before_scene = live.scene().unwrap().publication_id();
    let before_redo = live.coordinator().redo_len();
    let external = target.retained_usage_with_held_usage(two_held).unwrap();
    let mut worker_held = external;
    worker_held.add(live.scene_cache_usage().unwrap(), &limits).unwrap();
    let detached = host.restore_document(live.coordinator(), &two_archive, false, 0.05, 0.3, worker_held, &Cancellation::default()).unwrap();
    let bound = detached.bind(live.coordinator(), worker_held, &Cancellation::default()).unwrap();
    let canceled = Cancellation::default(); canceled.cancel();
    assert!(matches!(live.test_replace_bound_brep_document(bound, external, &canceled), Err(Error::Canceled)));
    assert_eq!(live.coordinator().snapshot(), before);
    assert_eq!(live.scene().unwrap().publication_id(), before_scene);
    assert_eq!(live.coordinator().redo_len(), before_redo);
    let detached = host.restore_document(live.coordinator(), &two_archive, false, 0.05, 0.3, worker_held, &Cancellation::default()).unwrap();
    let bound = detached.bind(live.coordinator(), worker_held, &Cancellation::default()).unwrap();
    live.redo().unwrap();
    let after_edit = live.coordinator().snapshot();
    let after_edit_scene = live.scene().unwrap().publication_id();
    assert!(matches!(live.test_replace_bound_brep_document(bound, external, &Cancellation::default()), Err(Error::Stale("BRep replacement context"))));
    assert_eq!(live.coordinator().snapshot(), after_edit);
    assert_eq!(live.scene().unwrap().publication_id(), after_edit_scene);
    drop(after_edit);
    let mut worker_held = external;
    worker_held.add(live.scene_cache_usage().unwrap(), &limits).unwrap();
    let detached = host.restore_document(live.coordinator(), &two_archive, false, 0.05, 0.3, worker_held, &Cancellation::default()).unwrap();
    let bound = detached.bind(live.coordinator(), worker_held, &Cancellation::default()).unwrap();
    live.test_replace_bound_brep_document(bound, external, &Cancellation::default()).unwrap();
    assert_eq!(live.document().id, document_id);
    assert_eq!(live.document().resources[&archive_resource_id].payload().unwrap(), b"embedded resource");
    assert_eq!(live.document().entities[&instance_id].placement, om9_core::geometry::Transform::translation([3., 4., 5.]));
    assert!(live.scene().unwrap().objects.iter().any(|object| object.root == first_id && object.has_pick_geometry()));
    assert!(live.scene().unwrap().objects.iter().any(|object| object.root == second_id && object.has_pick_geometry()));
    assert!(live.scene().unwrap().objects.iter().any(|object| object.root == instance_id && object.has_pick_geometry()));
    let om9_core::geometry::Geometry::Brep(source_after_replace) = &live.document().entities[&first_id].geometry else { panic!("replacement BRep") };
    assert!(live.coordinator().brep_representation(source_after_replace).is_ok());
    drop(before);
    drop(live);

    // Exercise the normal file lifecycle with the pinned, packaged worker.
    if option_env!("OM9_BREP_WORKER_SHA256").is_some() {
        use om9_app::file_lifecycle::{FileKind, FileOutcome};
        let open_path = TempArchive(std::env::temp_dir().join(format!("om9-brep-open-{}.om9", uuid::Uuid::new_v4())));
        std::fs::write(&open_path.0, &two_archive).unwrap();
        let mut opened = om9_app::session::Controller::new(Document::new("before open"), limits).unwrap();
        opened.begin_file_operation(FileKind::Open).unwrap();
        let token = opened.file_operation().unwrap().token();
        assert_eq!(opened.accept_file_path(token, &open_path.0).unwrap(), FileOutcome::Replaced);
        assert_eq!(opened.document().id, document_id);
        assert_eq!(opened.document().resources[&archive_resource_id].payload().unwrap(), b"embedded resource");
        assert_eq!(opened.document().entities[&instance_id].placement, om9_core::geometry::Transform::translation([3., 4., 5.]));
        assert!(opened.scene().unwrap().objects.iter().any(|object| object.root == instance_id && object.has_pick_geometry()));
        let om9_core::geometry::Geometry::Brep(source) = &opened.document().entities[&first_id].geometry else { panic!("opened BRep") };
        assert!(opened.coordinator().brep_representation(source).is_ok());

        let mut invalid = two_archive.clone();
        invalid[168] ^= 1;
        std::fs::write(&open_path.0, invalid).unwrap();
        let mut unchanged = om9_app::session::Controller::new(Document::new("keep on failure"), limits).unwrap();
        let before = unchanged.coordinator().snapshot();
        let scene = unchanged.scene().unwrap().publication_id();
        unchanged.begin_file_operation(FileKind::Open).unwrap();
        let token = unchanged.file_operation().unwrap().token();
        assert!(unchanged.accept_file_path(token, &open_path.0).is_err());
        assert_eq!(unchanged.coordinator().snapshot(), before);
        assert_eq!(unchanged.scene().unwrap().publication_id(), scene);
    }

    let mut corrupt_document = two_archive.clone();
    corrupt_document[168] ^= 1; // first packet chunk identity, after metadata and one resource
    let corrupt_held = Usage { bytes: two_held.bytes + two_archive.capacity() as u64, ..two_held };
    assert!(host.restore_document(&target, &corrupt_document, false, 0.05, 0.3, corrupt_held, &Cancellation::default()).is_err());
    drop(corrupt_document);
    assert_eq!(target.external_reserved_bytes(MemoryClass::CpuCache), baseline);
    drop(two_archive);
    let [record, second_record] = records;
    drop(second_record);
    let held = two_held;

    // A Job Bag may include resource mutations alongside restored geometry.
    let candidate = host.restore_packet(&target, &record, 0.05, 0.3, held, &Cancellation::default()).unwrap();
    let (canonical, guard) = candidate.register(&target, held).unwrap();
    let resource = om9_core::resource::Resource::from_bytes("texture", om9_core::resource::ResourceKind::OpaqueBinary, b"resource payload", &limits).unwrap();
    let resource_id = resource.id;
    let resource_entity = om9_core::model::Entity::new("resource batch", om9_core::geometry::Geometry::Brep(canonical), None);
    let entity_id = resource_entity.id;
    let revision = target.world().documents[&document_id].revision;
    let request = TransactionRequest::new("restore dependencies", vec![
        Mutation::ResourceInsert { document: document_id, expected_revision: revision, resource },
        Mutation::Insert { document: document_id, entity: resource_entity },
    ]);
    let mut guards = [guard];
    let prepared = target.prepare_with_reserved_brep_mutations(request, held, &mut guards).unwrap();
    target.commit(prepared).unwrap();
    assert_eq!(target.world().documents[&document_id].resources[&resource_id].payload().unwrap(), b"resource payload");
    assert!(target.world().documents[&document_id].entities.contains_key(&entity_id));
    target.undo().unwrap();
    assert!(!target.world().documents[&document_id].resources.contains_key(&resource_id));
    assert!(!target.world().documents[&document_id].entities.contains_key(&entity_id));
    target.redo().unwrap();
    assert_eq!(target.world().documents[&document_id].resources[&resource_id].payload().unwrap(), b"resource payload");
    assert!(target.world().documents[&document_id].entities.contains_key(&entity_id));
    target.undo().unwrap();
    let before_rejection = target.snapshot();
    let candidate = host.restore_packet(&target, &record, 0.05, 0.3, held, &Cancellation::default()).unwrap();
    let (canonical, guard) = candidate.register(&target, held).unwrap();
    let stale_resource = om9_core::resource::Resource::from_bytes("stale", om9_core::resource::ResourceKind::OpaqueBinary, b"stale", &limits).unwrap();
    let stale_entity = om9_core::model::Entity::new("stale", om9_core::geometry::Geometry::Brep(canonical), None);
    let revision = target.world().documents[&document_id].revision;
    let mut guards = [guard];
    let request = TransactionRequest::new("stale dependencies", vec![
        Mutation::ResourceInsert { document: document_id, expected_revision: revision + 1, resource: stale_resource },
        Mutation::Insert { document: document_id, entity: stale_entity },
    ]);
    assert!(target.prepare_with_reserved_brep_mutations(request, held, &mut guards).is_err());
    drop(guards);
    assert_eq!(target.snapshot(), before_rejection);
    target.redo().unwrap();
    assert!(target.world().documents[&document_id].resources.contains_key(&resource_id));

    drop(record);
    drop(target);
    let mut controller = om9_app::session::Controller::new(Document::new("scene target"), limits).unwrap();
    let scene_held = controller.scene_cache_usage().unwrap();
    let worker_held = Usage { bytes: held.bytes + scene_held.bytes, ..held };
    let first_candidate = host.restore(controller.coordinator(), source.source(), source.topology(), source.shape_set(), 0.05, 0.3, worker_held, &Cancellation::default()).unwrap();
    let second_candidate = host.restore(controller.coordinator(), second_source.source(), second_source.topology(), second_source.shape_set(), 0.05, 0.3, worker_held, &Cancellation::default()).unwrap();
    let first = entity("scene first"); let first_id = first.id;
    let second = entity("scene second"); let second_id = second.id;
    controller.test_insert_restored_jobbag_breps(vec![(first_candidate, first), (second_candidate, second)], held).unwrap();
    assert!(controller.scene().unwrap().objects.iter().any(|object| object.root == first_id && object.has_pick_geometry()));
    assert!(controller.scene().unwrap().objects.iter().any(|object| object.root == second_id && object.has_pick_geometry()));
    controller.undo().unwrap();
    assert!(!controller.document().entities.contains_key(&first_id) && !controller.document().entities.contains_key(&second_id));
    controller.redo().unwrap();
    assert!(controller.scene().unwrap().objects.iter().any(|object| object.root == first_id && object.has_pick_geometry()));
    assert!(controller.scene().unwrap().objects.iter().any(|object| object.root == second_id && object.has_pick_geometry()));
    let before_scene = controller.scene().unwrap().publication_id();
    let before_undo = controller.coordinator().undo_len();
    let scene_held = controller.scene_cache_usage().unwrap();
    let worker_held = Usage { bytes: held.bytes + scene_held.bytes, ..held };
    let candidate = host.restore(controller.coordinator(), source.source(), source.topology(), source.shape_set(), 0.05, 0.3, worker_held, &Cancellation::default()).unwrap();
    let invalid = om9_core::model::Entity::new("invalid", om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Line { start: [0., 0., 0.], end: [1., 0., 0.], domain: [0., 1.] }), None);
    assert!(controller.test_insert_restored_jobbag_breps(vec![(candidate, invalid)], held).is_err());
    assert_eq!(controller.scene().unwrap().publication_id(), before_scene);
    assert_eq!(controller.coordinator().undo_len(), before_undo);
}
