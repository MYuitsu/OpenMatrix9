// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "test-fixtures")]
use om9_app::ui::MatrixApp;
use om9_core::{geometry::Geometry, model::{Document, Entity}};

fn drain(app: &mut MatrixApp) {
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_capture_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_capture(ctx));
        assert!(std::time::Instant::now() < deadline, "capture worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn drain_export(app: &mut MatrixApp) {
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_export_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "export worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn drain_root(app: &mut MatrixApp) {
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_project_root_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "root worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn brep_capture_preserves_document_and_updates_numbered_and_named_bags() {
    let mut app = MatrixApp::new(Document::with_template_tolerance("BRep capture")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,2,3").unwrap();
    let point = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[point]).unwrap();
    app.test_start_jobbag_capture(0).unwrap();
    drain(&mut app);
    assert!(!app.status.kind().is_failure());
    app.test_start_named_capture("Master", None).unwrap();
    drain(&mut app);
    assert!(!app.status.kind().is_failure());
    let master = app.project_manager.named_master_snapshot("Master").unwrap().id;
    app.controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
    let sphere = app.controller.document().entities.values().find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    app.controller.select_objects(&[sphere]).unwrap();
    let before_document = app.controller.document().clone();
    let before_undo = app.controller.coordinator().undo_len();
    if option_env!("OM9_BREP_WORKER_SHA256").is_none() {
        assert!(app.test_start_jobbag_capture(0).is_err());
        assert_eq!(app.controller.document(), &before_document);
        assert_eq!(app.controller.coordinator().undo_len(), before_undo);
        return;
    }
    for named in [false, true] {
        let start = if named { app.test_start_named_capture("Master", Some(master)) } else { app.test_start_jobbag_capture(0) };
        start.unwrap();
        drain(&mut app);
        assert!(!app.status.kind().is_failure(), "{}", app.status.message());
        assert_eq!(app.controller.document(), &before_document);
        assert_eq!(app.controller.coordinator().undo_len(), before_undo);
        app.status.acknowledge();
    }
    let numbered = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap();
    assert!(numbered.archive.as_ref().unwrap().starts_with(b"OM9\0\r\n\x1a\n\x06\0\0\0"));
    let named = app.project_manager.named_master_snapshot("Master").unwrap();
    assert!(named.archive.as_ref().unwrap().starts_with(b"OM9\0\r\n\x1a\n\x06\0\0\0"));
}

#[cfg(windows)]
#[test]
fn absent_source_directory_is_locked_until_root_change_finishes() {
    let dir = std::env::temp_dir().join(format!("om9-absent-source-{}", uuid::Uuid::new_v4()));
    let catalog = dir.join("missing").join("project-catalog.json");
    let guard = om9_app::project_catalog::test_guard_source_catalog(&catalog, None).unwrap();
    assert!(catalog.parent().unwrap().is_dir());
    assert!(!catalog.exists());
    let manager = om9_app::project_manager::ProjectManager::new("other session");
    assert!(matches!(om9_app::project_catalog::save(&catalog, &manager, None), Err(om9_core::Error::Busy)));
    drop(guard);
    om9_app::project_catalog::save(&catalog, &manager, None).unwrap();
    assert!(catalog.is_file());
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn root_change_refuses_source_created_after_preparation() {
    let dir = std::env::temp_dir().join(format!("om9-root-appearing-source-{}", uuid::Uuid::new_v4()));
    let old_root = dir.join("old");
    let new_root = dir.join("new");
    std::fs::create_dir_all(&new_root).unwrap();
    let source = old_root.join("project-catalog.json");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let mut app = MatrixApp::new(Document::new("not yet persisted")).unwrap();
    app.test_set_unpersisted_catalog_path(source.clone());
    app.test_start_project_root_change(&new_root, &settings).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let staged = std::fs::read_dir(&new_root).unwrap().any(|entry| {
            entry.unwrap().file_name().to_string_lossy().starts_with(".om9-project-catalog-")
        });
        if staged { break; }
        assert!(std::time::Instant::now() < deadline, "root preparation did not stage catalog");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    std::fs::write(&source, b"external source").unwrap();
    drain_root(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&source).unwrap(), b"external source");
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert!(!new_root.join("project-catalog.json").exists());
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn root_change_refuses_a_catalog_modified_by_another_writer() {
    let dir = std::env::temp_dir().join(format!("om9-root-stale-source-{}", uuid::Uuid::new_v4()));
    let old_root = dir.join("old");
    let new_root = dir.join("new");
    std::fs::create_dir_all(&old_root).unwrap();
    std::fs::create_dir_all(&new_root).unwrap();
    let old_catalog = old_root.join("project-catalog.json");
    let initial = om9_app::project_manager::ProjectManager::new("initial");
    let fingerprint = om9_app::project_catalog::save(&old_catalog, &initial, None).unwrap();
    let mut app = MatrixApp::new(Document::new("root source")).unwrap();
    app.test_start_project_catalog_at(old_catalog.clone()).unwrap();
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_project_catalog_load_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let mut external = initial;
    let id = external.active().id;
    external.rename_project(id, "other session").unwrap();
    om9_app::project_catalog::save(&old_catalog, &external, Some(fingerprint)).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    app.test_start_project_root_change(&new_root, &settings).unwrap();
    drain_root(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.active().name, "initial");
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert!(!new_root.join("project-catalog.json").exists());
    let (remaining, _) = om9_app::project_catalog::load_with_admission(&old_catalog, "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(remaining.active().name, "other session");
    drop(app);
    let mut reloaded = MatrixApp::new(Document::new("reloaded source")).unwrap();
    reloaded.test_start_project_catalog_at(old_catalog.clone()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while reloaded.test_project_catalog_load_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| reloaded.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    reloaded.test_start_project_root_change(&new_root, &settings).unwrap();
    #[cfg(windows)]
    {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match std::fs::OpenOptions::new().write(true).open(&old_catalog) {
                Err(error) if error.raw_os_error() == Some(32) => break,
                Err(error) => panic!("unexpected source open failure: {error}"),
                Ok(handle) => drop(handle),
            }
            assert!(std::time::Instant::now() < deadline, "source catalog was not guarded");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(matches!(om9_app::project_catalog::save(&old_catalog, &external, Some(fingerprint)), Err(om9_core::Error::Busy)));
    }
    drain_root(&mut reloaded);
    assert!(!reloaded.status.kind().is_failure(), "{}", reloaded.status.message());
    #[cfg(windows)]
    drop(std::fs::OpenOptions::new().write(true).open(&old_catalog).unwrap());
    let (moved, _) = om9_app::project_catalog::load_with_admission(&new_root.join("project-catalog.json"), "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(moved.active().name, "other session");
    drop(reloaded);
    std::fs::remove_dir_all(dir).unwrap();
}

fn close_request() -> eframe::egui::RawInput {
    let mut input = eframe::egui::RawInput::default();
    input.viewports.get_mut(&eframe::egui::ViewportId::ROOT).unwrap().events.push(eframe::egui::ViewportEvent::Close);
    input
}

fn close_after_workers_drain(app: &mut MatrixApp, context: &eframe::egui::Context) {
    let result = context.run_logic(&close_request(), |ctx| assert!(!app.test_service_window_close(ctx)));
    assert!(result.viewport_commands[&eframe::egui::ViewportId::ROOT].contains(&eframe::egui::ViewportCommand::CancelClose));
    assert!(app.test_project_close_waiting());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut requested_close = false;
    while app.test_project_close_waiting() {
        let output = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        requested_close |= output.viewport_commands.get(&eframe::egui::ViewportId::ROOT)
            .is_some_and(|commands| commands.contains(&eframe::egui::ViewportCommand::Close));
        assert!(std::time::Instant::now() < deadline, "close did not wait for worker completion");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(requested_close);
}

#[test]
fn close_cancels_import_and_releases_source_without_publishing() {
    let dir = std::env::temp_dir().join(format!("om9-import-close-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("source.om9project");
    let source = om9_app::project_manager::ProjectManager::new("imported");
    std::fs::write(&path, om9_app::project_catalog::encode_project(&source, source.active().id).unwrap()).unwrap();
    let mut app = MatrixApp::new(Document::new("close import")).unwrap();
    let before = app.project_manager.to_json().unwrap();
    app.test_start_project_import(vec![path.clone()]).unwrap();
    close_after_workers_drain(&mut app, &eframe::egui::Context::default());
    assert!(!app.test_project_import_pending());
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    drop(std::fs::OpenOptions::new().write(true).open(&path).unwrap());
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn canceled_prepared_root_does_not_publish_new_assets() {
    let mut document = Document::new("staged root assets");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let controller = om9_app::session::Controller::new(document, om9_core::budget::Limits::default()).unwrap();
    let archive = controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let mut manager = om9_app::project_manager::ProjectManager::new("staged");
    manager.capture_archive(0, vec![point], controller.document(), archive).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-root-staged-{}", uuid::Uuid::new_v4()));
    let root = dir.join("projects");
    std::fs::create_dir_all(&root).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let cancel = om9_core::transaction::Cancellation::default();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&std::path::absolute(&root).unwrap(), &settings, &manager,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    let assets = root.join("project-catalog.json.assets");
    let staged: Vec<_> = std::fs::read_dir(&assets).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(staged.len(), 1);
    assert!(staged[0].to_string_lossy().starts_with(".om9-project-catalog-"));
    cancel.cancel();
    drop(prepared);
    assert_eq!(std::fs::read_dir(&assets).unwrap().count(), 0);
    assert!(!root.join("project-catalog.json").exists());
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert!(std::fs::read_dir(&root).unwrap().all(|entry| !entry.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
    assert!(std::fs::read_dir(&dir).unwrap().all(|entry| !entry.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
    let bytes = manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_ref().unwrap();
    use sha2::{Digest, Sha256};
    let name = Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let shared_asset = assets.join(name);
    std::fs::write(&shared_asset, bytes).unwrap();
    let cancel = om9_core::transaction::Cancellation::default();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&std::path::absolute(&root).unwrap(), &settings, &manager,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    #[cfg(windows)]
    assert!(std::fs::write(&shared_asset, b"changed").is_err());
    drop(prepared);
    assert_eq!(std::fs::read(&shared_asset).unwrap().as_slice(), bytes.as_ref());
    std::fs::write(&shared_asset, b"released").unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(windows)]
#[ignore = "child process used by root retry test"]
fn root_retry_interrupted_child() {
    let root = std::path::PathBuf::from(std::env::var_os("OM9_TEST_ROOT_RETRY_ROOT").unwrap());
    let settings = std::path::PathBuf::from(std::env::var_os("OM9_TEST_ROOT_RETRY_SETTINGS").unwrap());
    let source = std::path::PathBuf::from(std::env::var_os("OM9_TEST_ROOT_RETRY_SOURCE").unwrap());
    let (manager, _) = om9_app::project_catalog::load_with_admission(&source, "unused", |_| Ok(())).unwrap().unwrap();
    let cancel = om9_core::transaction::Cancellation::default();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&root, &settings, &manager,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    let _ = prepared.test_publish_with_hook(&cancel, || std::process::exit(91));
    panic!("child did not exit at publication boundary");
}

#[test]
#[cfg(windows)]
fn interrupted_root_publication_can_retry_from_another_process() {
    use std::os::windows::process::CommandExt;
    let dir = std::env::temp_dir().join(format!("om9-root-interrupted-{}", uuid::Uuid::new_v4()));
    let old_root = dir.join("old");
    let new_root = dir.join("new");
    std::fs::create_dir_all(&old_root).unwrap();
    std::fs::create_dir_all(&new_root).unwrap();
    let new_root = std::path::absolute(new_root).unwrap();
    let source = old_root.join("project-catalog.json");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let mut document = Document::new("interrupted root source");
    let point = Entity::new("point", Geometry::Point([2., 3., 4.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let controller = om9_app::session::Controller::new(document, om9_core::budget::Limits::default()).unwrap();
    let archive = controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let mut manager = om9_app::project_manager::ProjectManager::new("interrupted project");
    manager.capture_archive(0, vec![point], controller.document(), archive.clone()).unwrap();
    om9_app::project_catalog::save(&source, &manager, None).unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "root_retry_interrupted_child"])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .env("OM9_TEST_ROOT_RETRY_ROOT", &new_root)
        .env("OM9_TEST_ROOT_RETRY_SETTINGS", &settings)
        .env("OM9_TEST_ROOT_RETRY_SOURCE", &source)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().unwrap();
    assert_eq!(child.code(), Some(91));
    let target = new_root.join("project-catalog.json");
    let interrupted_catalog = std::fs::read(&target).unwrap();
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    let (reloaded, _) = om9_app::project_catalog::load_with_admission(&source, "unused", |_| Ok(())).unwrap().unwrap();
    let cancel = om9_core::transaction::Cancellation::default();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&new_root, &settings, &reloaded,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    prepared.publish(&cancel).unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), interrupted_catalog);
    let configured: std::path::PathBuf = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(configured, new_root);
    let (moved, _) = om9_app::project_catalog::load_with_admission(&target, "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(moved.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_deref(), Some(archive.as_slice()));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(windows)]
fn root_change_worker_reuses_identical_existing_catalog() {
    let dir = std::env::temp_dir().join(format!("om9-root-worker-retry-{}", uuid::Uuid::new_v4()));
    let old = dir.join("old");
    let new = dir.join("new");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    let manager = om9_app::project_manager::ProjectManager::new("retry source");
    let old_catalog = old.join("project-catalog.json");
    let new_catalog = new.join("project-catalog.json");
    om9_app::project_catalog::save(&old_catalog, &manager, None).unwrap();
    om9_app::project_catalog::save(&new_catalog, &manager, None).unwrap();
    let original = std::fs::read(&new_catalog).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let mut app = MatrixApp::new(Document::new("root retry")).unwrap();
    app.test_start_project_catalog_at(old_catalog).unwrap();
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_project_catalog_load_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.test_start_project_root_change(&new, &settings).unwrap();
    drain_root(&mut app);
    assert!(!app.status.kind().is_failure(), "{}", app.status.message());
    assert_eq!(std::fs::read(&new_catalog).unwrap(), original);
    let configured: std::path::PathBuf = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(configured, std::path::absolute(&new).unwrap());
    assert_eq!(app.project_manager.active().name, "retry source");
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(windows)]
fn matching_existing_root_is_reused_without_overwrite_and_missing_assets_are_refused() {
    let mut document = Document::new("existing root");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let controller = om9_app::session::Controller::new(document, om9_core::budget::Limits::default()).unwrap();
    let archive = controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let mut manager = om9_app::project_manager::ProjectManager::new("same project");
    manager.capture_archive(0, vec![point], controller.document(), archive).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-existing-root-{}", uuid::Uuid::new_v4()));
    let root = dir.join("projects");
    std::fs::create_dir_all(&root).unwrap();
    let root = std::path::absolute(root).unwrap();
    let target = root.join("project-catalog.json");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    om9_app::project_catalog::save(&target, &manager, None).unwrap();
    let original = std::fs::read(&target).unwrap();
    let assets = root.join("project-catalog.json.assets");
    let asset = std::fs::read_dir(&assets).unwrap().next().unwrap().unwrap().path();
    let saved_asset = std::fs::read(&asset).unwrap();
    std::fs::remove_file(&asset).unwrap();
    let cancel = om9_core::transaction::Cancellation::default();
    assert!(om9_app::project_catalog::test_prepare_catalog_root(&root, &settings, &manager, &om9_core::budget::Limits::default(), &cancel).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), original);
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    std::fs::write(&asset, saved_asset).unwrap();
    let mut different = manager.clone();
    different.rename_project(different.active().id, "different project").unwrap();
    assert!(om9_app::project_catalog::test_prepare_catalog_root(&root, &settings, &different, &om9_core::budget::Limits::default(), &cancel).is_err());
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&root, &settings, &manager, &om9_core::budget::Limits::default(), &cancel).unwrap();
    assert!(std::fs::OpenOptions::new().write(true).open(&target).is_err());
    let result = prepared.test_publish_with_hook(&cancel, || std::fs::write(&settings, b"external change").unwrap());
    assert!(result.is_err());
    assert_eq!(std::fs::read(&target).unwrap(), original);
    assert_eq!(std::fs::read(&settings).unwrap(), b"external change");
    std::fs::write(&settings, b"old settings").unwrap();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&root, &settings, &manager, &om9_core::budget::Limits::default(), &cancel).unwrap();
    prepared.publish(&cancel).unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), original);
    let configured: std::path::PathBuf = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(configured, root);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(windows)]
fn root_change_preserves_settings_changed_after_catalog_publication() {
    let dir = std::env::temp_dir().join(format!("om9-root-settings-race-{}", uuid::Uuid::new_v4()));
    let root = dir.join("projects");
    std::fs::create_dir_all(&root).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let manager = om9_app::project_manager::ProjectManager::new("source");
    let cancel = om9_core::transaction::Cancellation::default();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&std::path::absolute(&root).unwrap(), &settings, &manager,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    let result = prepared.test_publish_with_hook(&cancel, || {
        std::fs::write(&settings, b"external change").unwrap();
    });
    assert!(result.is_err());
    assert_eq!(std::fs::read(&settings).unwrap(), b"external change");
    assert!(!root.join("project-catalog.json").exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(windows)]
fn failed_root_setting_publication_rolls_back_owned_catalog_and_can_retry() {
    let mut document = Document::new("rollback source");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let controller = om9_app::session::Controller::new(document, om9_core::budget::Limits::default()).unwrap();
    let archive = controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let mut manager = om9_app::project_manager::ProjectManager::new("rollback root");
    manager.capture_archive(0, vec![point], controller.document(), archive.clone()).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-root-rollback-{}", uuid::Uuid::new_v4()));
    let root = dir.join("projects");
    std::fs::create_dir_all(&root).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let unrelated = root.join("unrelated.txt");
    std::fs::write(&unrelated, b"keep").unwrap();
    let original_permissions = std::fs::metadata(&settings).unwrap().permissions();
    let cancel = om9_core::transaction::Cancellation::default();
    let prepared = om9_app::project_catalog::test_prepare_catalog_root(&std::path::absolute(&root).unwrap(), &settings, &manager,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    let result = prepared.test_publish_with_hook(&cancel, || {
        let mut permissions = std::fs::metadata(&settings).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&settings, permissions).unwrap();
    });
    assert!(result.is_err());
    std::fs::set_permissions(&settings, original_permissions).unwrap();
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert_eq!(std::fs::read(&unrelated).unwrap(), b"keep");
    assert!(!root.join("project-catalog.json").exists());
    let retry = om9_app::project_catalog::test_prepare_catalog_root(&std::path::absolute(&root).unwrap(), &settings, &manager,
        &om9_core::budget::Limits::default(), &cancel).unwrap();
    retry.publish(&cancel).unwrap();
    assert!(root.join("project-catalog.json").is_file());
    let configured: std::path::PathBuf = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(configured, std::path::absolute(&root).unwrap());
    assert_eq!(std::fs::read(&unrelated).unwrap(), b"keep");
    let (loaded, _) = om9_app::project_catalog::load_with_admission(&root.join("project-catalog.json"), "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(loaded.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_deref(), Some(archive.as_slice()));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn root_change_publishes_only_current_uncanceled_catalog() {
    let mut document = Document::new("root change");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive.clone()).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-root-change-{}", uuid::Uuid::new_v4()));
    let root = dir.join("projects");
    std::fs::create_dir_all(&root).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    let target = root.join("project-catalog.json");
    app.test_start_project_root_change(&root, &settings).unwrap();
    assert!(app.test_project_root_pending());
    app.test_cancel_project_root();
    drain_root(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert!(!target.exists());
    app.status.acknowledge();
    app.test_start_project_root_change(&root, &settings).unwrap();
    std::fs::write(&settings, b"changed externally").unwrap();
    drain_root(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&settings).unwrap(), b"changed externally");
    assert!(!target.exists());
    std::fs::write(&settings, b"old settings").unwrap();
    app.status.acknowledge();
    app.test_start_project_root_change(&root, &settings).unwrap();
    let id = app.project_manager.active().id;
    app.project_manager.rename_project(id, "changed during root move").unwrap();
    drain_root(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert!(!target.exists());
    app.status.acknowledge();
    app.test_start_project_root_change(&root, &settings).unwrap();
    std::fs::write(&target, b"existing user catalog").unwrap();
    drain_root(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert_eq!(std::fs::read(&target).unwrap(), b"existing user catalog");
    std::fs::remove_file(&target).unwrap();
    app.status.acknowledge();
    app.test_start_project_root_change(&root, &settings).unwrap();
    drain_root(&mut app);
    assert!(!app.status.kind().is_failure());
    let configured: std::path::PathBuf = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(configured, std::path::absolute(&root).unwrap());
    let (loaded, _) = om9_app::project_catalog::load_with_admission(&target, "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(loaded.active().name, "changed during root move");
    assert_eq!(loaded.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_deref(), Some(archive.as_slice()));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn close_cancels_and_drains_root_change() {
    let mut app = MatrixApp::new(Document::new("close root")).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-root-close-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let settings = dir.join("settings.json");
    std::fs::write(&settings, b"old settings").unwrap();
    app.test_start_project_root_change(&dir, &settings).unwrap();
    close_after_workers_drain(&mut app, &eframe::egui::Context::default());
    assert!(!app.test_project_root_pending());
    assert_eq!(std::fs::read(&settings).unwrap(), b"old settings");
    assert!(!dir.join("project-catalog.json").exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn project_export_reserves_memory_before_replacing_destination() {
    let mut document = Document::new("export admission");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive).unwrap();
    let path = std::env::temp_dir().join(format!("om9-export-admission-{}.om9project", uuid::Uuid::new_v4()));
    std::fs::write(&path, b"original file").unwrap();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap(),
    ).unwrap();
    assert!(app.test_export_project(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"original file");
    assert!(app.project_manager.slot_available(0, app.controller.document()));
    drop(pressure);
    app.test_export_project(&path).unwrap();
    assert!(app.test_project_export_pending());
    assert_eq!(std::fs::read(&path).unwrap(), b"original file");
    drain_export(&mut app);
    assert!(!app.status.kind().is_failure());
    let file = std::fs::File::open(&path).unwrap();
    let (manifest, assets) = om9_app::project_catalog::read_project_file(file, &om9_core::transaction::Cancellation::default()).unwrap();
    assert!(!manifest.is_empty());
    assert_eq!(assets.len(), 1);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn canceled_or_stale_project_export_keeps_destination() {
    let mut app = MatrixApp::new(Document::new("export state")).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-export-stale-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("export.om9project");
    std::fs::write(&path, b"original file").unwrap();
    app.test_export_project(&path).unwrap();
    app.test_cancel_project_export();
    drain_export(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), b"original file");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    app.status.acknowledge();
    app.test_export_project(&path).unwrap();
    let id = app.project_manager.active().id;
    app.project_manager.rename_project(id, "changed while exporting").unwrap();
    drain_export(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), b"original file");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn jobbag_exports_validate_and_publish_only_current_archives() {
    let mut document = Document::new("jobbag export");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    app.project_manager.capture_archive(0, vec![point.clone()], app.controller.document(), archive.clone()).unwrap();
    let (document, full_archive) = app.controller.capture_jobbag_document().unwrap();
    app.project_manager.capture_named_version("Master", document, full_archive.clone()).unwrap();
    let dir = std::env::temp_dir().join(format!("om9-bag-export-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("bag.om9");
    std::fs::write(&path, b"original").unwrap();
    app.test_export_jobbag(0, &path).unwrap();
    assert!(app.test_project_export_pending());
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    drain_export(&mut app);
    assert!(!app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), archive);
    app.test_export_named_jobbag("Master", &path).unwrap();
    drain_export(&mut app);
    assert!(!app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), full_archive);
    app.test_export_jobbag(0, &path).unwrap();
    app.test_cancel_project_export();
    drain_export(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), full_archive);
    app.status.acknowledge();
    app.test_export_jobbag(0, &path).unwrap();
    app.project_manager.clear_slot(0).unwrap();
    drain_export(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), full_archive);
    app.status.acknowledge();
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), b"corrupt".to_vec()).unwrap();
    app.test_export_jobbag(0, &path).unwrap();
    drain_export(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(std::fs::read(&path).unwrap(), full_archive);
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn nested_instance_jobbag_thumbnail_is_rendered_from_detached_archive() {
    use om9_core::{geometry::{Mesh, Transform}, model::Definition};
    let mesh = Entity::new("triangle", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [4., 0., 0.], [0., 3., 0.]], faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let inner = Definition::new("inner", vec![mesh]);
    let mut nested = Entity::new("nested", Geometry::Instance(inner.id), None);
    nested.placement = Transform::translation([2., 3., 4.]);
    let outer = Definition::new("outer", vec![nested]);
    let mut root = Entity::new("root", Geometry::Instance(outer.id), None);
    root.placement = Transform::translation([10., 20., 30.]);
    let id = root.id;
    let mut document = Document::new("instance thumbnail source");
    document.definitions.insert(inner.id, inner);
    document.definitions.insert(outer.id, outer);
    document.entities.insert(id, root.clone());
    let source = MatrixApp::new(document).unwrap();
    let archive = source.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let mut manager = om9_app::project_manager::ProjectManager::new("instance thumbnail");
    manager.capture_archive(0, vec![root], source.controller.document(), archive).unwrap();
    let manager = om9_app::project_manager::ProjectManager::from_json(&manager.to_json().unwrap(), "unused").unwrap();
    drop(source);
    let mut target = MatrixApp::new(Document::new("empty destination")).unwrap();
    target.project_manager = manager;
    let context = eframe::egui::Context::default();
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| target.test_service_thumbnails(ctx, true));
    assert!(target.test_thumbnail_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while target.test_thumbnail_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| target.test_service_thumbnails(ctx, false));
        assert!(std::time::Instant::now() < deadline, "thumbnail worker timed out");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(target.test_thumbnail_succeeded(), Some(true));
    let png = target.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.as_ref().unwrap();
    let image = image::load_from_memory(png).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (96, 96));
    let background = *image.get_pixel(0, 0);
    assert!(image.pixels().any(|pixel| *pixel != background), "thumbnail has no visible geometry");
    assert!(target.controller.document().entities.is_empty());
}

#[test]
fn close_cancels_and_drains_project_export() {
    let mut app = MatrixApp::new(Document::new("close export")).unwrap();
    let path = std::env::temp_dir().join(format!("om9-export-close-{}.om9project", uuid::Uuid::new_v4()));
    std::fs::write(&path, b"original file").unwrap();
    app.test_export_project(&path).unwrap();
    close_after_workers_drain(&mut app, &eframe::egui::Context::default());
    assert!(!app.test_project_export_pending());
    assert_eq!(std::fs::read(&path).unwrap(), b"original file");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn close_waits_for_real_selection_capture_to_cancel() {
    let mut document = Document::new("close capture");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.test_start_jobbag_capture(0).unwrap();
    close_after_workers_drain(&mut app, &eframe::egui::Context::default());
    assert!(!app.test_capture_pending());
    assert!(app.project_manager.active().slot(0).unwrap().is_empty());
}

#[test]
fn close_after_capture_still_prompts_for_dirty_document() {
    let mut app = MatrixApp::new(Document::new("dirty close capture")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,2,3").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.test_start_jobbag_capture(0).unwrap();
    let context = eframe::egui::Context::default();
    close_after_workers_drain(&mut app, &context);
    let output = context.run_logic(&close_request(), |ctx| { app.test_service_window_close(ctx); });
    assert!(output.viewport_commands[&eframe::egui::ViewportId::ROOT].contains(&eframe::egui::ViewportCommand::CancelClose));
    assert_eq!(app.controller.file_operation().unwrap().phase(), om9_app::file_lifecycle::FilePhase::Unsaved);
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn captured_archive_keeps_its_reservation_until_last_shared_owner_is_dropped() {
    let mut document = Document::new("capture asset lifetime");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.test_start_jobbag_capture(0).unwrap();
    drain(&mut app);
    assert_eq!(app.test_project_asset_lease_count(), 1);
    let shared = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.clone().unwrap();
    app.project_manager.clear_slot(0).unwrap();
    let context = eframe::egui::Context::default();
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
    assert_eq!(app.test_project_asset_lease_count(), 1);
    drop(shared);
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
    assert_eq!(app.test_project_asset_lease_count(), 0);
}

#[test]
fn selection_capture_rejects_cancel_changed_selection_and_changed_slot() {
    let mut document = Document::new("capture worker");
    let first = Entity::new("first", Geometry::Point([1., 2., 3.]), None);
    let second = Entity::new("second", Geometry::Point([4., 5., 6.]), None);
    let (a, b) = (first.id, second.id);
    document.entities.insert(a, first);
    document.entities.insert(b, second);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[a]).unwrap();
    app.test_start_jobbag_capture(0).unwrap();
    app.controller.select_objects(&[b]).unwrap();
    drain(&mut app);
    assert!(app.status.kind().is_failure());
    assert!(app.project_manager.active().slot(0).unwrap().is_empty());
    app.status.acknowledge();
    app.test_start_jobbag_capture(0).unwrap();
    app.test_cancel_capture();
    drain(&mut app);
    assert!(app.status.kind().is_failure());
    assert!(app.project_manager.active().slot(0).unwrap().is_empty());
    app.status.acknowledge();
    app.test_start_jobbag_capture(0).unwrap();
    drain(&mut app);
    let initial = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().id;
    app.test_start_jobbag_capture(0).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[a]).unwrap();
    let replacement = app.project_manager.capture_archive(0, vec![app.controller.document().entities[&a].clone()], app.controller.document(), archive).unwrap();
    assert_ne!(replacement, initial);
    drain(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().id, replacement);
}

#[test]
fn document_capture_rejects_cancel_and_stale_overwrite() {
    let mut app = MatrixApp::new(Document::new("document capture worker")).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,0,0").unwrap();
    app.test_start_named_capture("Master", None).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("2,0,0").unwrap();
    drain(&mut app);
    assert!(app.status.kind().is_failure());
    assert!(app.project_manager.named_master_snapshot("Master").is_none());
    app.status.acknowledge();
    app.test_start_named_capture("Master", None).unwrap();
    app.test_cancel_capture();
    drain(&mut app);
    assert!(app.status.kind().is_failure());
    assert!(app.project_manager.named_master_snapshot("Master").is_none());
    app.status.acknowledge();
    app.test_start_named_capture("Master", None).unwrap();
    drain(&mut app);
    let snapshot = app.project_manager.named_master_snapshot("Master").unwrap().id;
    app.test_start_named_capture("Master", Some(snapshot)).unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("3,0,0").unwrap();
    drain(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.named_master_snapshot("Master").unwrap().id, snapshot);
}

#[test]
fn failed_thumbnail_retries_after_backoff() {
    let mut document = Document::new("thumbnail retry");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    document.entities.insert(point.id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    // Simulate an archive that a worker cannot decode after it has started.
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), vec![1, 2, 3]).unwrap();
    let context = eframe::egui::Context::default();
    let service = |app: &mut MatrixApp| {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, true));
    };
    service(&mut app);
    assert!(app.test_thumbnail_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_thumbnail_pending() {
        service(&mut app);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(false));
    service(&mut app);
    assert!(!app.test_thumbnail_pending(), "retry must respect the backoff");
    while !app.test_thumbnail_pending() {
        service(&mut app);
        assert!(std::time::Instant::now() < deadline, "failed thumbnail was never retried");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn completed_thumbnail_is_not_attached_to_a_replaced_bag() {
    let mut document = Document::new("thumbnail stale");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let original = app.project_manager.capture_archive(0, vec![point.clone()], app.controller.document(), archive.clone()).unwrap();
    let context = eframe::egui::Context::default();
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, true));
    assert!(app.test_thumbnail_pending());
    app.project_manager.clear_slot(0).unwrap();
    let replacement = app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive).unwrap();
    assert_ne!(original, replacement);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_thumbnail_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, false));
        assert!(std::time::Instant::now() < deadline, "thumbnail worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    let current = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap();
    assert_eq!(current.id, replacement);
    assert!(current.thumbnail.png.is_none());
}

#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn thumbnail_rejects_a_new_archive_owner_with_the_same_snapshot_id() {
    let mut document = Document::new("thumbnail archive replacement");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let snapshot = app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive.clone()).unwrap();
    let context = eframe::egui::Context::default();
    let service = |app: &mut MatrixApp, start| {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, start));
    };
    service(&mut app, true);
    assert!(app.test_thumbnail_pending());
    let bag = app.project_manager.active_mut().slots[0].snapshot.as_mut().unwrap();
    assert_eq!(bag.id, snapshot);
    bag.archive = Some(std::sync::Arc::from(archive));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_thumbnail_pending() {
        service(&mut app, false);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.is_none());
    service(&mut app, true);
    assert!(app.test_thumbnail_pending());
    while app.test_thumbnail_pending() {
        service(&mut app, false);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.is_some());
}

#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn close_waits_for_real_thumbnail_worker_to_cancel() {
    let mut document = Document::new("close thumbnail");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive).unwrap();
    let context = eframe::egui::Context::default();
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, true));
    assert!(app.test_thumbnail_pending());
    close_after_workers_drain(&mut app, &context);
    assert!(!app.test_thumbnail_pending());
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.is_none());
}

#[test]
#[ignore = "requires a compatible GPU adapter and memory for a 65 MiB embedded texture"]
fn gpu_thumbnail_accepts_large_archive_within_configured_budget() {
    use om9_core::{budget::Limits, geometry::Mesh, resource::{Resource, ResourceKind}, rhino_semantics::{MeshUv, PbrMaterial, set_mesh_uv}};
    let limits = Limits { max_bytes: 2 * 1024 * 1024 * 1024, ..Limits::default() };
    let data = vec![0x5a; 65 * 1024 * 1024];
    let resource = Resource::from_bytes("large texture", ResourceKind::OpaqueBinary, &data, &limits).unwrap();
    drop(data);
    let mut mesh = Entity::new("textured mesh", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let uv = MeshUv { uvs: vec![[0., 0.], [1., 0.], [0., 1.]], corners: vec![], material: "fixture".into(),
        pbr: PbrMaterial { texture: Some(om9_core::rhino_semantics::embedded_texture_uri(resource.id).unwrap()), ..Default::default() } };
    set_mesh_uv(&mut mesh, &uv, &limits).unwrap();
    let id = mesh.id;
    let mut document = Document::new("large thumbnail");
    document.resources.insert(resource.id, resource);
    document.entities.insert(id, mesh.clone());
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let archive = controller.copy_jobbag_objects_om9(&[id]).unwrap();
    assert!(archive.len() > 64 * 1024 * 1024);
    let mut app = MatrixApp::test_from_controller(controller).unwrap();
    app.project_manager.capture_archive(0, vec![mesh], app.controller.document(), archive).unwrap();
    let context = eframe::egui::Context::default();
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, true));
    assert!(app.test_thumbnail_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_thumbnail_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, false));
        assert!(std::time::Instant::now() < deadline, "large thumbnail worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    let png = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.as_ref().unwrap();
    let image = image::load_from_memory(png).unwrap();
    assert_eq!((image.width(), image.height()), (96, 96));
}

#[test]
#[ignore = "requires a compatible GPU adapter for offscreen thumbnail rendering"]
fn thumbnail_admission_refusal_leaves_bag_usable_and_retries_after_pressure() {
    let mut document = Document::new("thumbnail pressure");
    let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive).unwrap();
    let before = app.controller.coordinator().snapshot();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap(),
    ).unwrap();
    let context = eframe::egui::Context::default();
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, true));
    assert!(!app.test_thumbnail_pending());
    assert!(app.project_manager.slot_available(0, app.controller.document()));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    drop(pressure);
    let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, true));
    assert!(app.test_thumbnail_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.test_thumbnail_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_thumbnails(ctx, false));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.is_some());
    assert_eq!(app.controller.coordinator().snapshot(), before);
}
