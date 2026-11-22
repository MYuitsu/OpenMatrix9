// SPDX-License-Identifier: LGPL-2.1-or-later
//! Project Manager and Job Bag surface checks for OM9-L2-03-15.
use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};

fn frame(app: &mut MatrixApp, context: &egui::Context) {
    frame_with_events(app, context, vec![]);
}

#[test]
fn project_controls_do_not_overlap_thumbnails_or_named_divider() {
    for scale in [1.0, 1.25, 1.5] {
        let mut app = MatrixApp::new(Document::new("project layout")).unwrap();
        for x in [1, 2] {
            app.controller.submit("Point").unwrap();
            app.controller.submit(&format!("{x},0,0")).unwrap();
            let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
            app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
        }
        let context = egui::Context::default();
        context.set_pixels_per_point(scale);
        for _ in 0..6 { frame(&mut app, &context); }
        let numbered = app.jobbag_slot_rects[&0];
        let capture = app.project_action_rects["capture-first-slot"];
        let divider = app.project_action_rects["toggle-named"];
        let master = app.project_bag_rects["Master"];
        let save = app.project_action_rects["save-master"];
        let viewport = app.project_action_rects["named-strip"];
        assert!(capture.top() >= numbered.bottom(), "numbered controls overlap at {scale}x");
        assert!(master.top() >= divider.bottom(), "named card overlaps divider at {scale}x: {master:?}, {divider:?}");
        assert!(save.top() >= master.bottom(), "named controls overlap at {scale}x");
        assert!(viewport.contains(save.center()), "named controls clipped at {scale}x: {save:?}, {viewport:?}");
    }
}

#[test]
fn project_action_icons_are_bound_to_authored_assets() {
    for name in ["ProjectAdd", "ProjectIn", "ProjectOut", "ProjectSave", "ProjectDelete"] {
        assert_eq!(om9_app::icons::icon_name(name), Some(name), "missing Project action icon: {name}");
    }
}

#[test]
fn project_commands_open_the_expected_surface_from_cmd() {
    use om9_app::dock_layout::SectionId;
    for (command, manager) in [("gvProjectManager", false), ("gvOpenProjectManager", true)] {
        let mut app = MatrixApp::new(Document::new("command routing")).unwrap();
        let before = app.project_manager.to_json().unwrap();
        app.dock.set_section_visible(SectionId::Projects, false);
        app.dock.set_section_collapsed(SectionId::Projects, true);
        let context = egui::Context::default();
        for _ in 0..3 { frame(&mut app, &context); }
        let editor = app.command_input_rect().unwrap();
        click(&mut app, &context, editor.center());
        frame_with_events(&mut app, &context, vec![egui::Event::Text(command.into())]);
        frame_with_events(&mut app, &context, vec![egui::Event::Key {
            key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE,
        }]);
        for _ in 0..4 { frame(&mut app, &context); }
        let section = app.dock.section(SectionId::Projects);
        assert!(section.visible && !section.collapsed, "{command} did not open Projects");
        assert_eq!(app.project_action_rects.contains_key("manager-project"), manager, "{command} opened the wrong surface");
        assert_eq!(app.project_manager.to_json().unwrap(), before);
    }
}

fn frame_with_events(app: &mut MatrixApp, context: &egui::Context, events: Vec<egui::Event>) {
    frame_with_size(app, context, Vec2::new(1280., 900.), events);
}

fn frame_with_size(app: &mut MatrixApp, context: &egui::Context, size: Vec2, events: Vec<egui::Event>) {
    let time = context.input(|input| input.time) + 1. / 60.;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn click(app: &mut MatrixApp, context: &egui::Context, pos: Pos2) {
    frame_with_events(app, context, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
        pos, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE,
    }]);
    frame_with_events(app, context, vec![egui::Event::PointerButton {
        pos, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE,
    }]);
}

#[test]
#[cfg(feature = "test-fixtures")]
fn manager_actions_fit_and_work_in_smaller_windows() {
    for size in [Vec2::new(1280., 720.), Vec2::new(800., 600.), Vec2::new(895., 432.), Vec2::new(800., 400.)] {
        let mut app = MatrixApp::new(Document::new("compact manager")).unwrap();
        let context = egui::Context::default();
        let run = |app: &mut MatrixApp, events| frame_with_size(app, &context, size, events);
        run(&mut app, vec![]);
        let screen = Rect::from_min_size(Pos2::ZERO, size);
        for _ in 0..12 {
            let manager = app.project_action_rects["manager"];
            let sidebar = app.sidebar_content_rect.unwrap();
            if sidebar.contains_rect(manager) { break; }
            run(&mut app, vec![egui::Event::PointerMoved(sidebar.center()),
                egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -120.),
                    modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
            for _ in 0..4 { run(&mut app, vec![]); }
        }
        let manager_rect = app.project_action_rects["manager"];
        assert!(app.sidebar_content_rect.unwrap().contains_rect(manager_rect), "Mngr unreachable at {size:?}: {manager_rect:?}");
        let manager = manager_rect.center();
        for pressed in [true, false] {
            run(&mut app, vec![egui::Event::PointerMoved(manager), egui::Event::PointerButton {
                pos: manager, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
            }]);
        }
        for _ in 0..12 { run(&mut app, vec![]); }
        for key in ["manager-refresh", "manager-import-project", "manager-import-folder", "manager-change-root", "manager-export", "manager-open-folder"] {
            let rect = *app.project_action_rects.get(key).unwrap_or_else(|| panic!("{key} missing at {size:?}"));
            assert!(screen.contains_rect(rect), "{key} outside {size:?}: {rect:?}");
            if matches!(key, "manager-import-project" | "manager-import-folder" | "manager-change-root" | "manager-export") {
                let pos = rect.center();
                for pressed in [true, false] {
                    run(&mut app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
                        pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
                    }]);
                }
                let expected = match key {
                    "manager-import-project" => "import-project",
                    "manager-import-folder" => "import-folder",
                    "manager-change-root" => "change-root",
                    "manager-export" => "export-project",
                    _ => unreachable!(),
                };
                assert_eq!(app.test_take_project_file_request_kind(), Some(expected), "{key} inaccessible at {size:?}");
            }
        }
    }
}

#[test]
#[cfg(feature = "test-fixtures")]
fn manager_export_remains_clickable_at_multiple_scales() {
    for scale in [1.0, 1.25, 1.5] {
        let mut app = MatrixApp::new(Document::new("manager footer")).unwrap();
        let context = egui::Context::default();
        context.set_pixels_per_point(scale);
        frame(&mut app, &context);
        let manager = app.project_action_rects["manager"].center();
        click(&mut app, &context, manager);
        for _ in 0..12 { frame(&mut app, &context); }
        let export = app.project_action_rects["manager-export"];
        assert!(context.content_rect().contains_rect(export), "export outside screen at {scale}x: {export:?}");
        click(&mut app, &context, export.center());
        assert!(app.test_take_project_file_request(), "export did not respond at {scale}x");
    }
}

#[test]
#[cfg(feature = "test-fixtures")]
fn queued_export_rejects_replaced_catalog_before_picker() {
    let mut app = MatrixApp::new(Document::new("replacement")).unwrap();
    let old = app.project_manager.active().id;
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    for _ in 0..8 { frame(&mut app, &context); }
    let export = app.project_action_rects["manager-export"].center();
    click(&mut app, &context, export);
    app.reload_project_state().unwrap();
    assert_eq!(app.project_manager.active().id, old);
    assert!(matches!(app.test_take_current_project_file_request(), Err(om9_core::Error::Stale(_))));
    frame(&mut app, &context);
    let export = app.project_action_rects["manager-export"].center();
    click(&mut app, &context, export);
    assert_eq!(app.test_take_current_project_file_request().unwrap(), true);
}

#[test]
#[cfg(feature = "test-fixtures")]
fn legacy_bag_exports_are_disabled_until_native_archives_exist() {
    let (document, ids) = document_with_two_points();
    let mut app = MatrixApp::new(document).unwrap();
    let entity = app.controller.document().entities[&ids[0]].clone();
    app.project_manager.capture(0, vec![entity.clone()], app.controller.document()).unwrap();
    app.project_manager.capture_master(app.controller.document()).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let slot = app.project_action_rects["save-first-slot"].center();
    click(&mut app, &context, slot);
    assert!(!app.test_take_project_file_request());
    let master = app.project_action_rects["save-master"].center();
    click(&mut app, &context, master);
    assert!(!app.test_take_project_file_request());

    let archive = app.controller.copy_jobbag_objects_om9(&[ids[0]]).unwrap();
    app.project_manager.capture_archive(0, vec![entity.clone()], app.controller.document(), archive.clone()).unwrap();
    let (snapshot, full_archive) = app.controller.capture_jobbag_document().unwrap();
    app.project_manager.capture_named_version("Master", snapshot, full_archive).unwrap();
    frame(&mut app, &context);
    let slot = app.project_action_rects["save-first-slot"].center();
    click(&mut app, &context, slot);
    assert!(app.test_take_project_file_request());
    let master = app.project_action_rects["save-master"].center();
    click(&mut app, &context, master);
    assert!(app.test_take_project_file_request());

    app.project_manager.capture(0, vec![entity.clone()], app.controller.document()).unwrap();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    let export = app.project_action_rects["manager-export"].center();
    click(&mut app, &context, export);
    assert!(!app.test_take_project_file_request());
    for _ in 0..4 { frame(&mut app, &context); }
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    let files = app.project_action_rects["manager-files"].center();
    click(&mut app, &context, files);
    frame(&mut app, &context);
    let files_export = app.project_action_rects["manager-files-export"].center();
    click(&mut app, &context, files_export);
    assert!(!app.test_take_project_file_request(), "Files export must be disabled for legacy bags");
    let legacy_master = app.project_manager.active().master.as_ref().unwrap().id;
    app.project_manager.clear_named_version("Master", legacy_master).unwrap();
    app.project_manager.capture_archive(0, vec![entity], app.controller.document(), archive).unwrap();
    assert!(!app.project_manager.active().has_legacy_bags());
    frame(&mut app, &context);
    let files_export = app.project_action_rects["manager-files-export"].center();
    click(&mut app, &context, files_export);
    assert_eq!(app.test_take_project_file_request_kind(), Some("export-project"));
}

#[test]
fn project_creation_dialog_does_not_apply_to_replaced_catalog() {
    let mut app = MatrixApp::new(Document::new("replacement")).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let add = app.project_action_rects["new-project"].center();
    click(&mut app, &context, add);
    frame(&mut app, &context);
    let name = app.project_action_rects["new-project-name"].center();
    click(&mut app, &context, name);
    frame_with_events(&mut app, &context, vec![egui::Event::Text("new project".into())]);
    app.reload_project_state().unwrap();
    let before = app.project_manager.to_json().unwrap();
    frame(&mut app, &context);
    let create = app.project_action_rects["new-project-create"].center();
    click(&mut app, &context, create);
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    assert!(app.project_action_rects.contains_key("new-project-name"), "name must remain available");
}

#[test]
fn named_open_rejects_reused_snapshot_id_after_catalog_replacement() {
    use om9_app::file_lifecycle::FilePhase;
    let mut app = MatrixApp::new(Document::new("pending named open")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,0,0").unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let old = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("2,0,0").unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    assert_eq!(app.controller.file_operation().unwrap().phase(), FilePhase::Unsaved);
    let before = app.controller.coordinator().snapshot();
    app.reload_project_state().unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let replacement = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    assert_eq!(old, replacement, "fixture must reproduce snapshot ID reuse");
    frame(&mut app, &context);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert!(app.status.kind().is_failure());
}

#[test]
fn stale_project_delete_confirmation_cannot_remove_reused_id() {
    let mut app = MatrixApp::new(Document::new("replacement")).unwrap();
    let old = app.project_manager.active().id;
    app.project_manager.create_project("second", 0).unwrap();
    app.project_manager.select(old).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let row = app.project_row_rects[&old].center();
    for pressed in [true, false] {
        frame_with_events(&mut app, &context, vec![egui::Event::PointerMoved(row), egui::Event::PointerButton {
            pos: row, button: egui::PointerButton::Secondary, pressed, modifiers: egui::Modifiers::NONE,
        }]);
    }
    frame(&mut app, &context);
    let delete = app.project_action_rects["project-row-delete"].center();
    click(&mut app, &context, delete);
    frame(&mut app, &context);
    assert!(app.project_action_rects.contains_key("project-confirm"));
    app.reload_project_state().unwrap();
    assert_eq!(app.project_manager.active().id, old);
    app.project_manager.create_project("other replacement", 0).unwrap();
    let before = app.project_manager.to_json().unwrap();
    frame(&mut app, &context);
    let confirm = app.project_action_rects["project-confirm"].center();
    click(&mut app, &context, confirm);
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    assert!(app.status.kind().is_failure());
}

#[test]
fn stale_directory_removal_cannot_delete_reused_catalog_id() {
    use om9_app::project_manager::DirectoryKind;
    let mut app = MatrixApp::new(Document::new("replacement")).unwrap();
    let old = app.project_manager.create_directory_entry(DirectoryKind::Customer, "old customer").unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    for _ in 0..4 { frame(&mut app, &context); }
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    let directory = app.project_action_rects["manager-directory"].center();
    click(&mut app, &context, directory);
    for _ in 0..4 { frame(&mut app, &context); }
    let remove = app.project_action_rects["directory-customer-remove"].center();
    click(&mut app, &context, remove);
    frame(&mut app, &context);
    assert!(app.project_action_rects.contains_key("project-confirm"));
    app.reload_project_state().unwrap();
    let replacement = app.project_manager.create_directory_entry(DirectoryKind::Customer, "new customer").unwrap();
    assert_eq!(old, replacement);
    frame(&mut app, &context);
    let confirm = app.project_action_rects["project-confirm"].center();
    click(&mut app, &context, confirm);
    assert_eq!(app.project_manager.directory_entries(DirectoryKind::Customer)[0].name, "new customer");
    assert!(app.status.kind().is_failure());
}

#[test]
fn stale_directory_editor_cannot_rename_reused_catalog_id() {
    use om9_app::project_manager::DirectoryKind;
    let mut app = MatrixApp::new(Document::new("replacement")).unwrap();
    let old = app.project_manager.create_directory_entry(DirectoryKind::Customer, "old customer").unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    for _ in 0..4 { frame(&mut app, &context); }
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    let directory = app.project_action_rects["manager-directory"].center();
    click(&mut app, &context, directory);
    for _ in 0..4 { frame(&mut app, &context); }
    let edit = app.project_action_rects["directory-customer-edit"].center();
    click(&mut app, &context, edit);
    frame(&mut app, &context);
    let name = app.project_action_rects["directory-editor-name"].center();
    click(&mut app, &context, name);
    frame_with_events(&mut app, &context, vec![egui::Event::Text("X".into())]);
    app.reload_project_state().unwrap();
    let replacement = app.project_manager.create_directory_entry(DirectoryKind::Customer, "new customer").unwrap();
    assert_eq!(old, replacement, "fixture must reproduce directory ID reuse");
    frame(&mut app, &context);
    let save = app.project_action_rects["directory-editor-save"].center();
    click(&mut app, &context, save);
    assert_eq!(app.project_manager.directory_entries(DirectoryKind::Customer)[0].name, "new customer");
    assert!(app.project_action_rects.contains_key("directory-editor-name"));
}

#[test]
fn stale_info_draft_cannot_overwrite_replaced_catalog_row() {
    let mut app = MatrixApp::new(Document::new("new document project")).unwrap();
    let id = app.project_manager.active().id;
    app.project_manager.rename_project(id, "original project").unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    for _ in 0..4 { frame(&mut app, &context); }
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    let name = app.project_action_rects["manager-info-name"].center();
    click(&mut app, &context, name);
    frame_with_events(&mut app, &context, vec![egui::Event::Text("X".into())]);
    app.reload_project_state().unwrap();
    assert_eq!(app.project_manager.active().id, id, "fixture must reproduce ID reuse");
    assert_eq!(app.project_manager.active().name, "new document project");
    for _ in 0..12 {
        frame(&mut app, &context);
        let viewport = app.project_action_rects["manager-info-viewport"];
        let save = app.project_action_rects["manager-info-save"];
        if viewport.contains_rect(save) { break; }
        frame_with_events(&mut app, &context, vec![egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -120.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { frame(&mut app, &context); }
    }
    let save = app.project_action_rects["manager-info-save"];
    assert!(app.project_action_rects["manager-info-viewport"].contains_rect(save));
    click(&mut app, &context, save.center());
    assert_eq!(app.project_manager.active().name, "new document project");
    assert!(app.project_action_rects.contains_key("manager-info-name"), "draft must remain available");
}

#[test]
fn manager_info_save_is_reachable_in_compact_window() {
    let mut app = MatrixApp::new(Document::new("compact info")).unwrap();
    let context = egui::Context::default();
    let size = Vec2::new(800., 400.);
    let run = |app: &mut MatrixApp, events| frame_with_size(app, &context, size, events);
    let click_at = |app: &mut MatrixApp, pos: Pos2| {
        for pressed in [true, false] {
            run(app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
                pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
            }]);
        }
    };
    run(&mut app, vec![]);
    for _ in 0..12 {
        let sidebar = app.sidebar_content_rect.unwrap();
        if sidebar.contains_rect(app.project_action_rects["manager"]) { break; }
        run(&mut app, vec![egui::Event::PointerMoved(sidebar.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -120.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { run(&mut app, vec![]); }
    }
    let manager = app.project_action_rects["manager"].center();
    click_at(&mut app, manager);
    for _ in 0..8 { run(&mut app, vec![]); }
    let tile = app.project_action_rects["manager-project"].center();
    click_at(&mut app, tile);
    for _ in 0..8 { run(&mut app, vec![]); }
    let screen = Rect::from_min_size(Pos2::ZERO, size);
    let name = app.project_action_rects["manager-info-name"];
    assert!(screen.contains_rect(name), "Info name outside compact screen: {name:?}");
    click_at(&mut app, name.center());
    run(&mut app, vec![egui::Event::Text("X".into())]);
    for _ in 0..16 {
        let viewport = app.project_action_rects["manager-info-viewport"];
        let save = app.project_action_rects["manager-info-save"];
        if viewport.contains_rect(save) && screen.contains_rect(save) { break; }
        run(&mut app, vec![egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -100.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { run(&mut app, vec![]); }
    }
    let viewport = app.project_action_rects["manager-info-viewport"];
    let save = app.project_action_rects["manager-info-save"];
    let cancel = app.project_action_rects["manager-info-cancel"];
    assert!(viewport.contains_rect(save) && screen.contains_rect(save), "Info Save unreachable: {save:?}, viewport {viewport:?}");
    assert!(viewport.contains_rect(cancel) && screen.contains_rect(cancel), "Info Cancel unreachable: {cancel:?}, viewport {viewport:?}");
    click_at(&mut app, save.center());
    assert!(app.project_manager.active().name.contains('X'));
}

#[test]
fn manager_info_draft_survives_directory_rename_and_removal() {
    use om9_app::project_manager::DirectoryKind;
    let mut app = MatrixApp::new(Document::new("Draft")).unwrap();
    let project = app.project_manager.active().id;
    let customer = app.project_manager.create_directory_entry(DirectoryKind::Customer, "Alice").unwrap();
    let designer = app.project_manager.create_directory_entry(DirectoryKind::Designer, "Bob").unwrap();
    let mut metadata = app.project_manager.active().metadata.clone();
    metadata.customer_id = Some(customer);
    metadata.designer_id = Some(designer);
    metadata.designer = "Bob".into();
    app.project_manager.update_project(project, "Draft", "Alice", "", metadata).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    let other = app.project_manager.create_project("Other", 0).unwrap();
    app.project_manager.select(project).unwrap();
    frame(&mut app, &context);
    let other_row = app.project_row_rects[&other].center();
    click(&mut app, &context, other_row);
    assert_eq!(app.project_manager.active().id, project, "sidebar changed project during an open draft");
    let count = app.project_manager.rows().len();
    let add = app.project_action_rects["new-project"].center();
    click(&mut app, &context, add);
    frame(&mut app, &context);
    assert!(!app.project_action_rects.contains_key("new-project-name"), "creation opened during an Info draft");
    assert_eq!(app.project_manager.rows().len(), count);
    assert_eq!(app.project_manager.active().id, project);
    let name = app.project_action_rects["manager-info-name"].center();
    click(&mut app, &context, name);
    frame_with_events(&mut app, &context, vec![egui::Event::Text("X".into())]);
    app.project_manager.rename_directory_entry(DirectoryKind::Customer, customer, "Alicia").unwrap();
    app.project_manager.remove_directory_entry(DirectoryKind::Designer, designer).unwrap();
    for _ in 0..12 {
        frame(&mut app, &context);
        let viewport = app.project_action_rects["manager-info-viewport"];
        let save = app.project_action_rects["manager-info-save"];
        if viewport.contains_rect(save) { break; }
        frame_with_events(&mut app, &context, vec![egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -120.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { frame(&mut app, &context); }
    }
    let viewport = app.project_action_rects["manager-info-viewport"];
    let save = app.project_action_rects["manager-info-save"];
    assert!(viewport.contains_rect(save), "Save must be reachable: {save:?} / {viewport:?}");
    click(&mut app, &context, save.center());
    let row = app.project_manager.active();
    assert!(row.name.contains('X'), "draft edit was lost: {}", row.name);
    assert_eq!((row.customer.as_str(), row.metadata.customer_id), ("Alicia", Some(customer)));
    assert_eq!((row.metadata.designer.as_str(), row.metadata.designer_id), ("Bob", None));
}

#[test]
fn manager_overview_opens_and_named_bags_toggle() {
    let mut app = MatrixApp::new(Document::new("overview")).unwrap();
    let project = app.project_manager.active().id;
    app.project_manager.timer_start(project, 100).unwrap();
    app.project_manager.timer_stop(project, 135).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    assert!(app.project_bag_rects.contains_key("Master"));
    let toggle = app.project_action_rects["toggle-named"].center();
    click(&mut app, &context, toggle);
    assert!(app.project_bag_rects.is_empty());
    frame(&mut app, &context);
    let toggle = app.project_action_rects["toggle-named"].center();
    click(&mut app, &context, toggle);
    assert!(app.project_bag_rects.contains_key("Master"));
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    assert!(app.project_action_rects.contains_key("manager-project"));
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    assert!(app.project_action_rects.contains_key("manager-info-save"));
    let output = context.run_ui(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
        time: Some(context.input(|input| input.time) + 1. / 60.), ..Default::default()
    }, |ui| app.show(ui));
    fn contains_text(shape: &egui::Shape, expected: &str) -> bool {
        match shape {
            egui::Shape::Text(text) => text.galley.job.text == expected,
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| contains_text(shape, expected)),
            _ => false,
        }
    }
    assert!(output.shapes.iter().any(|shape| contains_text(&shape.shape, "Timer 00:00:35 (stopped)")));
}

#[test]
fn manager_overview_uses_named_snapshot_thumbnail_when_no_numbered_bag_exists() {
    use std::io::Cursor;
    let mut app = MatrixApp::new(Document::new("named preview")).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    let id = app.project_manager.capture_named_version_summary("Master", 0, 0, vec![1]).unwrap();
    let image = image::RgbaImage::from_pixel(96, 96, image::Rgba([200, 50, 20, 255]));
    let mut png = Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).unwrap();
    app.project_manager.set_thumbnail(app.project_manager.active().id, id, png.into_inner()).unwrap();
    let mut output = context.run_ui(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
        time: Some(context.input(|input| input.time) + 1. / 60.), ..Default::default()
    }, |ui| app.show(ui));
    output.textures_delta.clear();
    let tile = app.project_action_rects["manager-project"];
    fn has_image(shape: &egui::Shape, tile: Rect) -> bool {
        match shape {
            egui::Shape::Mesh(mesh) => mesh.texture_id != egui::TextureId::Managed(0) && tile.contains(mesh.calc_bounds().center()),
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| has_image(shape, tile)),
            _ => false,
        }
    }
    assert!(output.shapes.iter().any(|shape| has_image(&shape.shape, tile)), "the project tile must paint the named snapshot preview");
}

#[cfg(feature = "test-fixtures")]
#[test]
fn manager_schedules_thumbnail_for_a_non_active_project() {
    let mut app = MatrixApp::new(Document::new("thumbnail queue")).unwrap();
    let first = app.project_manager.active().id;
    // The intentionally invalid archive makes the worker exit before GPU use;
    // this test checks scheduling and cleanup, not the rendered image.
    let snapshot = app.project_manager.capture_named_version_summary("Master", 0, 0, vec![1]).unwrap();
    let second = app.project_manager.create_project("second", 1).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    assert!(!app.test_thumbnail_attempted(first, snapshot));
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    assert_eq!(app.project_manager.active().id, second);
    assert!(app.test_thumbnail_attempted(first, snapshot));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_thumbnail_pending() && std::time::Instant::now() < deadline {
        app.test_service_thumbnails(&context, false);
        std::thread::yield_now();
    }
    assert!(!app.test_thumbnail_pending());
}

#[cfg(feature = "test-fixtures")]
#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn manager_renders_thumbnail_for_a_non_active_project() {
    use om9_core::budget::Limits;
    let mut captured = Document::new("thumbnail source");
    for point in [[0., 0., 0.], [4., 3., 2.]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        captured.entities.insert(entity.id, entity);
    }
    let archive = om9_archive::Archive::encode(&captured, &Limits::default()).unwrap();
    let mut app = MatrixApp::new(Document::new("thumbnail queue")).unwrap();
    let first = app.project_manager.active().id;
    let snapshot = app.project_manager.capture_named_version_summary("Master", 2, captured.layers.len(), archive).unwrap();
    let second = app.project_manager.create_project("second", 1).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    loop {
        app.test_service_thumbnails(&context, true);
        let row = app.project_manager.rows().iter().find(|row| row.id == first).unwrap();
        let thumbnail = row.named_versions.get("Master").unwrap().iter().find(|item| item.id == snapshot).unwrap().thumbnail_png.as_ref();
        if let Some(png) = thumbnail {
            let image = image::load_from_memory(png).unwrap().to_rgba8();
            assert_eq!((image.width(), image.height()), (96, 96));
            let first_pixel = image.get_pixel(0, 0);
            assert!(image.pixels().any(|pixel| pixel != first_pixel));
            break;
        }
        assert_ne!(app.test_thumbnail_succeeded(), Some(false), "thumbnail worker failed");
        assert!(std::time::Instant::now() < deadline, "thumbnail worker timed out");
        std::thread::yield_now();
    }
    assert_eq!(app.project_manager.active().id, second);
}

#[test]
fn named_filmstrip_scrolls_without_moving_numbered_bags_or_rows() {
    for scale in [1.0, 1.25, 1.5] {
        let mut app = MatrixApp::new(Document::new("named scroll")).unwrap();
        let context = egui::Context::default();
        context.set_pixels_per_point(scale);
        for _ in 0..6 { frame(&mut app, &context); }
        let named = app.project_bag_rects["Master"];
        let numbered = app.jobbag_slot_rects[&0];
        let row = app.project_row_rects[&app.project_manager.active().id];
        let viewport = app.project_action_rects["named-strip"];
        frame_with_events(&mut app, &context, vec![egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(-160., 0.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..8 { frame(&mut app, &context); }
        assert!(app.project_bag_rects["Master"].left() < named.left() - 10., "named strip did not scroll at {scale}x");
        assert_eq!(app.jobbag_slot_rects[&0], numbered, "numbered strip moved at {scale}x");
        assert_eq!(app.project_row_rects[&app.project_manager.active().id], row, "row list moved at {scale}x");
        let toggle = app.project_action_rects["toggle-named"].center();
        click(&mut app, &context, toggle);
        assert!(app.project_bag_rects.is_empty(), "named strip did not collapse at {scale}x");
        click(&mut app, &context, toggle);
        assert!(app.project_bag_rects.contains_key("Master"), "named strip did not expand at {scale}x");
    }
}

#[test]
fn numbered_filmstrip_scrolls_independently_at_multiple_scales() {
    for scale in [1.0, 1.25, 1.5] {
        let mut app = MatrixApp::new(Document::new("scroll test")).unwrap();
        let context = egui::Context::default();
        context.set_pixels_per_point(scale);
        for _ in 0..3 { frame(&mut app, &context); }
        let first = app.jobbag_slot_rects[&0];
        let named = app.project_bag_rects["Master"];
        let row = app.project_row_rects[&app.project_manager.active().id];
        frame_with_events(&mut app, &context, vec![
            egui::Event::PointerMoved(first.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(-180., 0.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move },
        ]);
        for _ in 0..6 { frame(&mut app, &context); }
        assert!(app.jobbag_slot_rects[&0].left() < first.left() - 10., "numbered strip did not scroll at {scale}x");
        assert_eq!(app.project_bag_rects["Master"].left(), named.left(), "named strip scrolled at {scale}x");
        assert_eq!(app.project_row_rects[&app.project_manager.active().id], row, "project list moved at {scale}x");
    }
}

#[test]
fn numbered_filmstrip_scrollbar_can_be_dragged() {
    for scale in [1.0, 1.25, 1.5] {
        let mut app = MatrixApp::new(Document::new("drag scroll test")).unwrap();
        let context = egui::Context::default();
        context.set_pixels_per_point(scale);
        for _ in 0..12 { frame(&mut app, &context); }
        let before = app.jobbag_slot_rects[&0];
        let viewport = app.project_action_rects["numbered-strip"];
        let bar = app.project_action_rects["numbered-scrollbar"];
        let start = Pos2::new(bar.left() + 12., bar.center().y);
        let end = Pos2::new(bar.right() - 12., start.y);
        frame_with_events(&mut app, &context, vec![egui::Event::PointerMoved(start), egui::Event::PointerButton {
            pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE,
        }]);
        frame_with_events(&mut app, &context, vec![egui::Event::PointerMoved(end)]);
        frame_with_events(&mut app, &context, vec![egui::Event::PointerButton {
            pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE,
        }]);
        frame(&mut app, &context);
        assert!(app.jobbag_slot_rects[&0].left() < before.left() - 10., "scrollbar drag did not move the strip at {scale}x; viewport={viewport:?}, bar={bar:?}");
    }
}

#[test]
fn manager_search_applies_on_submit_and_reset_restores_results() {
    let mut app = MatrixApp::new(Document::new("overview")).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    let field = app.project_action_rects["manager-search"].center();
    click(&mut app, &context, field);
    frame_with_events(&mut app, &context, vec![egui::Event::Text("no such project".into())]);
    assert!(app.project_action_rects.contains_key("manager-project"));
    let submit = app.project_action_rects["manager-search-submit"].center();
    click(&mut app, &context, submit);
    frame(&mut app, &context);
    assert!(!app.project_action_rects.contains_key("manager-project"));
    let reset = app.project_action_rects["manager-search-reset"].center();
    click(&mut app, &context, reset);
    frame(&mut app, &context);
    assert!(app.project_action_rects.contains_key("manager-project"));
}

#[test]
fn manager_numeric_filter_works_after_scrolling_in_compact_window() {
    let size = Vec2::new(895., 432.);
    let mut app = MatrixApp::new(Document::new("compact first")).unwrap();
    let first = app.project_manager.active().id;
    let mut metadata = app.project_manager.active().metadata.clone();
    metadata.gem_weight_carats = Some(1.5);
    app.project_manager.update_project(first, "compact first", "", "", metadata.clone()).unwrap();
    let second = app.project_manager.create_project("compact second", 0).unwrap();
    metadata.gem_weight_carats = Some(3.0);
    app.project_manager.update_project(second, "compact second", "", "", metadata).unwrap();
    app.project_manager.select(second).unwrap();
    let context = egui::Context::default();
    let run = |app: &mut MatrixApp, events| frame_with_size(app, &context, size, events);
    run(&mut app, vec![]);
    for _ in 0..12 {
        let sidebar = app.sidebar_content_rect.unwrap();
        if sidebar.contains_rect(app.project_action_rects["manager"]) { break; }
        run(&mut app, vec![egui::Event::PointerMoved(sidebar.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -120.),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { run(&mut app, vec![]); }
    }
    let click_at = |app: &mut MatrixApp, pos: Pos2| {
        for pressed in [true, false] {
            run(app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
                pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
            }]);
        }
    };
    let manager = app.project_action_rects["manager"].center();
    click_at(&mut app, manager);
    for _ in 0..6 { run(&mut app, vec![]); }
    for _ in 0..12 {
        let viewport = app.project_action_rects["manager-filters-viewport"];
        let header = app.project_action_rects["manager-gem-filter"];
        if viewport.contains_rect(header) { break; }
        let direction = if header.center().y > viewport.center().y { -60. } else { 60. };
        run(&mut app, vec![egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., direction),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { run(&mut app, vec![]); }
    }
    let header = app.project_action_rects["manager-gem-filter"];
    assert!(app.project_action_rects["manager-filters-viewport"].contains_rect(header));
    click_at(&mut app, header.center());
    for _ in 0..8 { run(&mut app, vec![]); }
    for _ in 0..12 {
        let viewport = app.project_action_rects["manager-filters-viewport"];
        let Some(max) = app.project_action_rects.get("manager-gem-max").copied() else { run(&mut app, vec![]); continue; };
        if viewport.contains_rect(max) { break; }
        let direction = if max.center().y > viewport.center().y { -40. } else { 40. };
        run(&mut app, vec![egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., direction),
                modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move }]);
        for _ in 0..4 { run(&mut app, vec![]); }
    }
    let viewport = app.project_action_rects["manager-filters-viewport"];
    let min = app.project_action_rects["manager-gem-min"];
    let max = app.project_action_rects["manager-gem-max"];
    assert!(viewport.contains_rect(min) && viewport.contains_rect(max), "range fields clipped: {min:?} {max:?} / {viewport:?}");
    click_at(&mut app, min.center());
    run(&mut app, vec![egui::Event::Text("1.0".into())]);
    click_at(&mut app, max.center());
    run(&mut app, vec![egui::Event::Text("2.0".into())]);
    let search = app.project_action_rects["manager-search-submit"].center();
    click_at(&mut app, search);
    run(&mut app, vec![]);
    let tile = app.project_action_rects["manager-project"].center();
    click_at(&mut app, tile);
    assert_eq!(app.project_manager.active().id, first);
}

#[test]
fn manager_numeric_filter_accepts_both_bounds_and_selects_matching_project() {
    let mut app = MatrixApp::new(Document::new("first project")).unwrap();
    let first = app.project_manager.active().id;
    let mut metadata = app.project_manager.active().metadata.clone();
    metadata.gem_weight_carats = Some(1.5);
    app.project_manager.update_project(first, "first project", "", "", metadata).unwrap();
    let second = app.project_manager.create_project("second project", 0).unwrap();
    let mut metadata = app.project_manager.active().metadata.clone();
    metadata.gem_weight_carats = Some(3.0);
    app.project_manager.update_project(second, "second project", "", "", metadata).unwrap();
    app.project_manager.select(second).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    for _ in 0..4 { frame(&mut app, &context); }
    let header = app.project_action_rects["manager-gem-filter"];
    click(&mut app, &context, header.center());
    frame(&mut app, &context);
    let min = app.project_action_rects["manager-gem-min"];
    let max = app.project_action_rects["manager-gem-max"];
    let panel = app.project_action_rects["manager-filter-panel"];
    assert!(panel.contains_rect(min) && panel.contains_rect(max), "range fields extend beyond filter column: {min:?} / {max:?} / {panel:?}");
    click(&mut app, &context, min.center());
    frame_with_events(&mut app, &context, vec![egui::Event::Text("1.0".into())]);
    click(&mut app, &context, max.center());
    frame_with_events(&mut app, &context, vec![egui::Event::Text("2.0".into())]);
    let submit = app.project_action_rects["manager-search-submit"].center();
    click(&mut app, &context, submit);
    frame(&mut app, &context);
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    assert_eq!(app.project_manager.active().id, first);
}

#[test]
fn manager_jobs_filter_finds_hidden_projects() {
    let mut app = MatrixApp::new(Document::new("visible project")).unwrap();
    let visible = app.project_manager.active().id;
    let hidden = app.project_manager.create_project("hidden project", 0).unwrap();
    app.project_manager.set_visible(hidden, false).unwrap();
    app.project_manager.select(visible).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    let jobs = app.project_action_rects["manager-jobs-filter"].center();
    click(&mut app, &context, jobs);
    frame(&mut app, &context);
    let hidden_filter = app.project_action_rects["manager-hidden-filter"].center();
    click(&mut app, &context, hidden_filter);
    let submit = app.project_action_rects["manager-search-submit"].center();
    click(&mut app, &context, submit);
    frame(&mut app, &context);
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    assert_eq!(app.project_manager.active().id, hidden);
}

#[test]
fn manager_files_single_click_previews_without_inserting_geometry() {
    let (document, ids) = document_with_two_points();
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[ids[0]]).unwrap();
    let archive = app.controller.copy_selected_jobbag_om9().unwrap();
    let entities = app.controller.jobbag_selected_entities().unwrap();
    app.project_manager.capture_archive(0, entities, app.controller.document(), archive).unwrap();
    let before = app.controller.coordinator().snapshot();
    let undo = app.controller.coordinator().undo_len();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let manager = app.project_action_rects["manager"].center();
    click(&mut app, &context, manager);
    frame(&mut app, &context);
    let tile = app.project_action_rects["manager-project"].center();
    click(&mut app, &context, tile);
    frame(&mut app, &context);
    let files = app.project_action_rects["manager-files"].center();
    click(&mut app, &context, files);
    frame(&mut app, &context);
    assert!(app.project_action_rects.contains_key("manager-file-preview"));
    for _ in 0..40 { frame(&mut app, &context); }
    let thumbnail = app.project_action_rects["manager-file-thumbnail"].center();
    click(&mut app, &context, thumbnail);
    frame(&mut app, &context);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    let thumbnail = app.project_action_rects["manager-file-thumbnail"].center();
    click(&mut app, &context, thumbnail);
    assert_eq!(app.controller.document().entities.len(), 3);
    assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
}

#[test]
fn every_named_group_opens_its_latest_version_after_discard() {
    use om9_app::{file_lifecycle::FilePhase, ui::FileControl};
    for name in ["Master", "Creation", "Parts", "Render", "Output"] {
        let mut app = MatrixApp::new(Document::new(name)).unwrap();
        for x in [1, 2] {
            app.controller.submit("Point").unwrap();
            app.controller.submit(&format!("{x},0,0")).unwrap();
            let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
            app.project_manager.capture_named_version(name, snapshot, archive).unwrap();
        }
        app.controller.submit("Point").unwrap();
        app.controller.submit("3,0,0").unwrap();
        let context = egui::Context::default();
        for _ in 0..4 { frame(&mut app, &context); }
        let card = app.project_bag_rects[name];
        let viewport = app.project_action_rects["numbered-strip"];
        if card.right() > viewport.right() {
            frame_with_events(&mut app, &context, vec![
                egui::Event::PointerMoved(Pos2::new(viewport.center().x, card.center().y)),
                egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: Vec2::new(-500., 0.),
                    modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move },
            ]);
            for _ in 0..12 { frame(&mut app, &context); }
        }
        let card = app.project_bag_rects[name];
        let open = card.right_top() + Vec2::new(-6., 8.);
        click(&mut app, &context, open);
        frame(&mut app, &context);
        assert_eq!(app.controller.file_operation().map(|op| op.phase()), Some(FilePhase::Unsaved), "{name}");
        let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
        click(&mut app, &context, discard);
        frame(&mut app, &context);
        assert!(app.controller.file_operation().is_none(), "{name}");
        assert_eq!(app.controller.document().entities.len(), 2, "{name} should open the second version");
    }
}

#[test]
fn named_version_selection_resets_when_catalog_reuses_ids() {
    use om9_app::ui::FileControl;
    let mut app = MatrixApp::new(Document::new("version reset")).unwrap();
    let mut original_ids = Vec::new();
    for x in [1, 2] {
        app.controller.submit("Point").unwrap();
        app.controller.submit(&format!("{x},0,0")).unwrap();
        let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
        original_ids.push(app.project_manager.capture_named_version("Master", snapshot, archive).unwrap());
    }
    let context = egui::Context::default();
    fn text_position(shape: &egui::Shape, expected: &str) -> Option<Pos2> {
        match shape {
            egui::Shape::Text(text) if text.galley.job.text == expected => Some(text.pos + text.galley.size() / 2.),
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| text_position(shape, expected)),
            _ => None,
        }
    }
    let render = |app: &mut MatrixApp| {
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            time: Some(context.input(|input| input.time) + 1. / 60.), ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
        output
    };
    let output = render(&mut app);
    let dropdown = output.shapes.iter().find_map(|shape| text_position(&shape.shape, "2 / 2")).unwrap();
    click(&mut app, &context, dropdown);
    let output = render(&mut app);
    let first = output.shapes.iter().find_map(|shape| text_position(&shape.shape, "Version 1")).unwrap();
    click(&mut app, &context, first);
    app.reload_project_state().unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let first_new = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("3,0,0").unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let second_new = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    assert_eq!([first_new, second_new], original_ids.as_slice());
    app.controller.submit("Point").unwrap();
    app.controller.submit("4,0,0").unwrap();
    for _ in 0..4 { frame(&mut app, &context); }
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
    click(&mut app, &context, discard);
    frame(&mut app, &context);
    assert_eq!(app.controller.document().entities.len(), 3, "new catalog should default to its latest version");
}

#[test]
fn named_card_can_open_an_older_version() {
    use om9_app::ui::FileControl;
    let mut app = MatrixApp::new(Document::new("version selection")).unwrap();
    for x in [1, 2] {
        app.controller.submit("Point").unwrap();
        app.controller.submit(&format!("{x},0,0")).unwrap();
        let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
        app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    }
    let context = egui::Context::default();
    fn text_position(shape: &egui::Shape, expected: &str) -> Option<Pos2> {
        match shape {
            egui::Shape::Text(text) if text.galley.job.text == expected => Some(text.pos + text.galley.size() / 2.),
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| text_position(shape, expected)),
            _ => None,
        }
    }
    let render = |app: &mut MatrixApp| {
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            time: Some(context.input(|input| input.time) + 1. / 60.), ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
        output
    };
    let output = render(&mut app);
    let master = app.project_bag_rects["Master"];
    let creation = app.project_bag_rects["Creation"];
    assert!(creation.left() - master.right() <= 6., "version selector widened the Master card: {master:?}, {creation:?}");
    let dropdown = output.shapes.iter().find_map(|shape| text_position(&shape.shape, "2 / 2")).unwrap();
    click(&mut app, &context, dropdown);
    let output = render(&mut app);
    let first = output.shapes.iter().find_map(|shape| text_position(&shape.shape, "Version 1")).unwrap();
    click(&mut app, &context, first);
    frame(&mut app, &context);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
    click(&mut app, &context, discard);
    frame(&mut app, &context);
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn named_open_prompts_and_cancel_or_discard_preserves_the_expected_document() {
    use om9_app::{file_lifecycle::FilePhase, ui::FileControl};
    let mut app = MatrixApp::new(Document::new("named open")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,0,0").unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("2,0,0").unwrap();
    let before = app.controller.coordinator().snapshot();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    assert_eq!(app.controller.file_operation().unwrap().phase(), FilePhase::Unsaved);
    let cancel = app.file_control_rect(FileControl::Cancel).unwrap().center();
    click(&mut app, &context, cancel);
    frame(&mut app, &context);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
    click(&mut app, &context, discard);
    frame(&mut app, &context);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn named_open_save_writes_current_document_before_replacement() {
    use om9_app::{file_lifecycle::FilePhase, ui::FileControl};
    let path = std::env::temp_dir().join(format!("om9-named-open-{}.om9", uuid::Uuid::new_v4()));
    let mut app = MatrixApp::new(Document::new("named save")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,0,0").unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("2,0,0").unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    let save = app.file_control_rect(FileControl::Save).unwrap().center();
    click(&mut app, &context, save);
    frame(&mut app, &context);
    let operation = app.controller.file_operation().unwrap();
    assert_eq!(operation.phase(), FilePhase::SavePath);
    let token = operation.token();
    app.controller.accept_file_path(token, &path).unwrap();
    frame(&mut app, &context);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(MatrixApp::from_file(&path).unwrap().controller.document().entities.len(), 2);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn corrupt_named_snapshot_does_not_replace_dirty_document() {
    use om9_app::ui::FileControl;
    let mut app = MatrixApp::new(Document::new("corrupt named snapshot")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,0,0").unwrap();
    app.project_manager.capture_named_version_summary("Master", 1, 0, b"invalid archive".to_vec()).unwrap();
    let before = app.controller.coordinator().snapshot();
    let undo = app.controller.coordinator().undo_len();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
    click(&mut app, &context, discard);
    frame(&mut app, &context);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
}

#[test]
fn removing_named_snapshot_while_prompt_is_open_cannot_replace_document() {
    use om9_app::ui::FileControl;
    let mut app = MatrixApp::new(Document::new("stale named snapshot")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,0,0").unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let id = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("2,0,0").unwrap();
    let before = app.controller.coordinator().snapshot();
    let context = egui::Context::default();
    frame(&mut app, &context);
    let open = app.project_action_rects["restore-master"].center();
    click(&mut app, &context, open);
    frame(&mut app, &context);
    app.project_manager.clear_named_version("Master", id).unwrap();
    let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
    click(&mut app, &context, discard);
    frame(&mut app, &context);
    assert!(app.controller.file_operation().is_none());
    assert_eq!(app.controller.coordinator().snapshot(), before);
}

#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn captured_jobbag_produces_a_visible_gpu_thumbnail() {
    let (document, ids) = document_with_two_points();
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&ids).unwrap();
    let archive = app.controller.copy_selected_jobbag_om9().unwrap();
    let selected = app.controller.jobbag_selected_entities().unwrap();
    app.project_manager.capture_archive(0, selected, app.controller.document(), archive).unwrap();
    let context = egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let png = loop {
        frame(&mut app, &context);
        if let Some(png) = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.clone() { break png; }
        assert!(std::time::Instant::now() < deadline, "GPU thumbnail was not published within 30 seconds");
        std::thread::sleep(std::time::Duration::from_millis(30));
    };
    let image = image::load_from_memory(&png).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (96, 96));
    let first = image.get_pixel(0, 0);
    assert!(image.pixels().any(|pixel| pixel != first), "rendered thumbnail has no visible variation");
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
fn projects_and_initial_jobbag_slots_are_visible() {
    let mut app = MatrixApp::new(Document::new("jobbag-ui")).expect("app");
    let context = egui::Context::default();
    frame(&mut app, &context);
    assert!(!app.project_row_rects.is_empty());
    assert_eq!(app.jobbag_slot_rects.len(), 10);
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
fn detached_archive_keeps_placement_after_source_delete_and_other_document_insert() {
    let mut document = Document::new("source");
    let mut point = Entity::new("placed", Geometry::Point([1., 2., 3.]), None);
    point.placement = om9_core::geometry::Transform::translation([12., 23., 34.]);
    let placement = point.placement.clone();
    let id = point.id;
    document.entities.insert(id, point);
    let mut source = MatrixApp::new(document).unwrap();
    source.controller.select_objects(&[id]).unwrap();
    let archive = source.controller.copy_selected_jobbag_om9().unwrap();
    let selected = source.controller.jobbag_selected_entities().unwrap();
    source.project_manager.capture_archive(0, selected, source.controller.document(), archive).unwrap();
    source.controller.delete_objects(&[id]).unwrap();
    assert!(source.project_manager.slot_available(0, source.controller.document()));
    let archive = source.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.clone().unwrap();
    let mut destination = MatrixApp::new(Document::new("destination")).unwrap();
    let inserted = destination.controller.paste_om9(&archive).unwrap();
    assert_eq!(destination.controller.document().entities[&inserted[0]].placement, placement);
    destination.controller.undo().unwrap();
    assert!(destination.controller.document().entities.is_empty());
    destination.controller.redo().unwrap();
    assert_eq!(destination.controller.document().entities[&inserted[0]].placement, placement);
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

#[cfg(feature = "test-fixtures")]
#[test]
fn jobbag_rejects_archive_with_extra_root_before_insert_or_import() {
    use om9_app::project_manager::{BagFilter, ProjectManager};
    use om9_core::{budget::Limits, geometry::Curve};
    let mut document = Document::new("root mismatch");
    let selected = Entity::new("selected", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [1., 0., 0.], domain: [0., 1.],
    }), None);
    let extra = Entity::new("extra", Geometry::Point([4., 0., 0.]), None);
    document.entities.insert(selected.id, selected.clone());
    document.entities.insert(extra.id, extra.clone());
    let limits = Limits::default();
    let archive = om9_archive::Archive::encode(&document, &limits).unwrap();
    let mut app = MatrixApp::new(document.clone()).unwrap();
    app.project_manager.capture_archive(0, vec![selected.clone()], &document, archive).unwrap();
    let before = app.controller.coordinator().snapshot();
    let scene = app.controller.scene().unwrap().publication_id();
    assert!(matches!(app.test_insert_filtered_jobbag(0, BagFilter::All), Err(om9_core::Error::Invalid("jobbag archive root set mismatch"))));
    assert!(matches!(app.test_insert_filtered_jobbag(0, BagFilter::Curves), Err(om9_core::Error::Invalid("jobbag archive root set mismatch"))));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.scene().unwrap().publication_id(), scene);
    let exported = app.project_manager.export_project(app.project_manager.active().id).unwrap();
    let mut target = ProjectManager::new("target");
    let before = target.to_json().unwrap();
    assert!(matches!(target.import_project(&exported, &limits), Err(om9_core::Error::Invalid("jobbag archive root set mismatch"))));
    assert_eq!(target.to_json().unwrap(), before);

    let mut projection = document.clone();
    projection.entities.remove(&extra.id);
    let archive = om9_archive::Archive::encode(&projection, &limits).unwrap();
    app.project_manager.capture_archive(0, vec![selected], &document, archive).unwrap();
    app.test_insert_filtered_jobbag(0, BagFilter::All).unwrap();
    assert_eq!(app.controller.document().entities.len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
}
#[cfg(feature = "test-fixtures")]
#[test]
fn filtered_nested_mesh_instance_survives_capture_and_insert() {
    use om9_app::project_manager::BagFilter;
    use om9_core::{geometry::{Geometry, Mesh}, model::{Definition, Document, Entity}};
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let leaf = Definition::new("mesh", vec![Entity::new("face", Geometry::Mesh(mesh.clone()), None)]);
    let outer = Definition::new("nested", vec![Entity::new("child", Geometry::Instance(leaf.id), None)]);
    let root = Entity::new("root", Geometry::Instance(outer.id), None);
    let root_id = root.id;
    let mut document = Document::new("filtered instance");
    document.definitions.insert(leaf.id, leaf);
    document.definitions.insert(outer.id, outer);
    document.entities.insert(root_id, root);
    let mixed = Definition::new("mixed", vec![
        Entity::new("mesh", Geometry::Mesh(mesh), None),
        Entity::new("point", Geometry::Point([0., 0., 0.]), None),
    ]);
    let mixed_root = Entity::new("mixed root", Geometry::Instance(mixed.id), None);
    document.definitions.insert(mixed.id, mixed);
    assert!(BagFilter::Meshes.matches_in_document(&mixed_root, &document, &om9_core::budget::Limits::default()).is_err());
    assert!(BagFilter::Gems.matches_in_document(&mixed_root, &document, &om9_core::budget::Limits::default()).is_err());
    let mut app = om9_app::ui::MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[root_id]).unwrap();
    app.test_capture_filtered_jobbag(0, BagFilter::Meshes).unwrap();
    let context = egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_capture_pending() && std::time::Instant::now() < deadline {
        app.test_service_capture(&context);
        std::thread::yield_now();
    }
    assert!(!app.test_capture_pending());
    let snapshot = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap();
    assert_eq!(snapshot.filtered_ids(BagFilter::Meshes).unwrap(), vec![root_id]);
    assert!(snapshot.filtered_ids(BagFilter::Curves).unwrap().is_empty());
    let mut legacy: serde_json::Value = serde_json::from_slice(&app.project_manager.to_json().unwrap()).unwrap();
    legacy["rows"][0]["slots"][0]["snapshot"]["root_classes"][0] = serde_json::json!("Curve");
    app.project_manager = om9_app::project_manager::ProjectManager::from_json(&serde_json::to_vec(&legacy).unwrap(), "invalid").unwrap();
    assert!(app.test_insert_filtered_jobbag(0, BagFilter::Curves).is_err());
    assert_eq!(app.controller.document().entities.len(), 1);
    legacy["rows"][0]["slots"][0]["snapshot"]["root_classes"][0] = serde_json::json!("UnknownInstance");
    app.project_manager = om9_app::project_manager::ProjectManager::from_json(&serde_json::to_vec(&legacy).unwrap(), "legacy").unwrap();
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().filtered_ids(BagFilter::Meshes).is_err());
    app.stage_project_state().unwrap();
    let mut app = om9_app::ui::MatrixApp::new(app.controller.document().clone()).unwrap();
    app.test_insert_filtered_jobbag(0, BagFilter::Meshes).unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
    assert!(app.controller.document().entities.values().all(|entity| matches!(entity.geometry, Geometry::Instance(_))));
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn filtered_instance_classification_shares_a_budget_across_roots() {
    use om9_app::project_manager::{BagFilter, ProjectManager};
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Definition, Document, Entity}, Error};
    let limits = Limits::default();
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let leaf = Definition::new("mesh", vec![Entity::new("face", Geometry::Mesh(mesh), None)]);
    let outer = Definition::new("nested", vec![Entity::new("child", Geometry::Instance(leaf.id), None)]);
    let first = Entity::new("first", Geometry::Instance(outer.id), None);
    let second = Entity::new("second", Geometry::Instance(outer.id), None);
    let mut document = Document::new("classification budget");
    document.definitions.insert(leaf.id, leaf);
    document.definitions.insert(outer.id, outer);
    document.entities.insert(first.id, first.clone());
    document.entities.insert(second.id, second.clone());
    let archive = om9_archive::Archive::encode(&document, &limits).unwrap();
    let mut manager = ProjectManager::new("classification budget");
    manager.capture_archive_with_limits(0, vec![first, second], &document, archive, &limits).unwrap();
    let snapshot = manager.active().slot(0).unwrap().snapshot.as_ref().unwrap();
    assert_eq!(snapshot.filtered_ids_in_document(BagFilter::Meshes, &document, &limits).unwrap().len(), 2);
    let tight = Limits { max_nodes: 5, ..limits };
    assert!(matches!(snapshot.filtered_ids_in_document(BagFilter::Meshes, &document, &tight), Err(Error::Budget("jobbag classification"))));
}
