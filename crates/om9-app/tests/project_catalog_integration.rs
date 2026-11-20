// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "test-fixtures")]
//! Catalog and native project archive checks using only authored temporary data.
use om9_app::{project_catalog, project_manager::ProjectManager, session::Controller};
use om9_core::{budget::Limits, geometry::Geometry, model::{Document, Entity}, transaction::Cancellation};

#[test]
fn project_export_checks_cancellation_during_hashing_and_asset_copy() {
    use std::cell::Cell;
    let limits = Limits::default();
    let mut manager = ProjectManager::new("export cancellation");
    let id = manager.active().id;
    let snapshot = manager.capture_named_version_summary("Master", 0, 0, vec![0x53; 4 * 1024 * 1024]).unwrap();
    let checks = Cell::new(0usize);
    let bytes = project_catalog::encode_project_with_check(&manager, id, &limits, || {
        checks.set(checks.get() + 1);
        Ok(())
    }).unwrap();
    let total_checks = checks.get();
    assert!(total_checks > 10);
    for stop_at in [5, total_checks - 2] {
        checks.set(0);
        let result = project_catalog::encode_project_with_check(&manager, id, &limits, || {
            checks.set(checks.get() + 1);
            if checks.get() == stop_at { Err(om9_core::Error::Canceled) } else { Ok(()) }
        });
        assert_eq!(result, Err(om9_core::Error::Canceled));
        assert_eq!(checks.get(), stop_at);
    }
    assert_eq!(project_catalog::encode_project_with_limits(&manager, id, &limits).unwrap(), bytes);
    assert_eq!(manager.named_master_by_id("Master", snapshot).unwrap().archive.as_ref().unwrap().len(), 4 * 1024 * 1024);
}

#[test]
fn catalog_load_can_cancel_during_asset_hydration_without_changing_files() {
    let limits = Limits::default();
    let dir = std::env::temp_dir().join(format!("om9-hydration-cancel-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("catalog.json");
    let mut manager = ProjectManager::new("hydration");
    let snapshot = manager.capture_named_version_summary("Master", 0, 0, vec![0x52; 4 * 1024 * 1024]).unwrap();
    let fingerprint = project_catalog::save(&path, &manager, None).unwrap();
    let mut admissions = Vec::new();
    let (loaded, actual) = project_catalog::load_with_limits_and_admission(&path, "test", &limits, |peak| {
        admissions.push(peak);
        Ok(())
    }).unwrap().unwrap();
    assert_eq!(actual, fingerprint);
    assert_eq!(loaded.named_master_by_id("Master", snapshot).unwrap().archive.as_ref().unwrap().len(), 4 * 1024 * 1024);
    let (highest, _) = admissions.iter().enumerate().max_by_key(|(_, peak)| **peak).unwrap();
    let hydration_start = admissions.iter().enumerate().skip(highest + 1)
        .find(|(_, peak)| **peak < admissions[highest]).map(|(index, _)| index).unwrap();
    let stop_at = hydration_start + 2;
    assert!(stop_at < admissions.len());
    let mut calls = 0;
    let result = project_catalog::load_with_limits_and_admission(&path, "test", &limits, |_| {
        let current = calls;
        calls += 1;
        if current == stop_at { Err(om9_core::Error::Canceled) } else { Ok(()) }
    });
    assert!(matches!(result, Err(om9_core::Error::Canceled)));
    assert_eq!(project_catalog::load_with_limits_and_admission(&path, "test", &limits, |_| Ok(())).unwrap().unwrap().1, fingerprint);
    std::fs::remove_dir_all(dir).unwrap();
}

fn drain_import(app: &mut om9_app::ui::MatrixApp) {
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_project_import_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_import(ctx));
        assert!(std::time::Instant::now() < deadline, "project import worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn drain_catalog_load(app: &mut om9_app::ui::MatrixApp) {
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_project_catalog_load_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "catalog load did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn drain_catalog_save(app: &mut om9_app::ui::MatrixApp) {
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.test_project_catalog_save_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "catalog save did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn ui_frame(app: &mut om9_app::ui::MatrixApp, context: &eframe::egui::Context, events: Vec<eframe::egui::Event>) {
    let mut output = context.run_ui(eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1280., 900.))),
        time: Some(context.input(|input| input.time) + 1. / 60.), events, ..Default::default()
    }, |ui| app.show(ui));
    output.textures_delta.clear();
}

fn ui_click(app: &mut om9_app::ui::MatrixApp, context: &eframe::egui::Context, pos: eframe::egui::Pos2) {
    for pressed in [true, false] {
        ui_frame(app, context, vec![eframe::egui::Event::PointerMoved(pos), eframe::egui::Event::PointerButton {
            pos, button: eframe::egui::PointerButton::Primary, pressed, modifiers: eframe::egui::Modifiers::NONE,
        }]);
    }
    ui_frame(app, context, vec![]);
}

fn named_card_button(app: &mut om9_app::ui::MatrixApp, context: &eframe::egui::Context, name: &'static str, capture: bool) -> eframe::egui::Pos2 {
    for _ in 0..12 {
        ui_frame(app, context, vec![]);
        let viewport = app.project_action_rects["named-strip"];
        let rect = app.project_bag_rects[name];
        let point = rect.right_top() + eframe::egui::Vec2::new(-6., if capture { 25. } else { 8. });
        if viewport.contains(point) { return point; }
        let delta = if point.x > viewport.right() { -80. } else { 80. };
        ui_frame(app, context, vec![eframe::egui::Event::PointerMoved(viewport.center()),
            eframe::egui::Event::MouseWheel { unit: eframe::egui::MouseWheelUnit::Point,
                delta: eframe::egui::Vec2::new(delta, 0.), modifiers: eframe::egui::Modifiers::NONE,
                phase: eframe::egui::TouchPhase::Move }]);
        for _ in 0..5 { ui_frame(app, context, vec![]); }
    }
    panic!("{name} button is unreachable");
}

#[test]
fn folder_import_rejects_invalid_archive_entries_without_publishing_rows() {
    let dir = std::env::temp_dir().join(format!("om9-folder-import-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let source = ProjectManager::new("folder source");
    let archive = project_catalog::encode_project(&source, source.active().id).unwrap();
    std::fs::write(dir.join("valid.om9project"), archive).unwrap();
    std::fs::create_dir(dir.join("invalid.om9project")).unwrap();
    std::fs::create_dir(dir.join("ordinary folder")).unwrap();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("destination")).unwrap();
    let before = app.project_manager.to_json().unwrap();
    app.test_start_project_import(vec![dir.join("missing.om9project")]).unwrap();
    drain_import(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    app.status.acknowledge();
    app.test_import_project_folder(&dir).unwrap();
    drain_import(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    std::fs::remove_dir(dir.join("invalid.om9project")).unwrap();
    app.status.acknowledge();
    app.test_import_project_folder(&dir).unwrap();
    app.test_cancel_project_import();
    drain_import(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    app.status.acknowledge();
    app.test_import_project_folder(&dir).unwrap();
    let id = app.project_manager.active().id;
    app.project_manager.rename_project(id, "changed during scan").unwrap();
    let changed = app.project_manager.to_json().unwrap();
    drain_import(&mut app);
    assert!(app.status.kind().is_failure());
    assert_eq!(app.project_manager.to_json().unwrap(), changed);
    app.status.acknowledge();
    app.test_import_project_folder(&dir).unwrap();
    drain_import(&mut app);
    assert!(!app.status.kind().is_failure());
    assert_eq!(app.project_manager.rows().len(), 2);
    assert!(app.project_manager.rows().iter().any(|row| row.name == "folder source"));
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn import_admission_failure_after_preflight_keeps_catalog_and_allows_retry() {
    let dir = std::env::temp_dir().join(format!("om9-import-pressure-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("source.om9project");
    let source = ProjectManager::new("pressure source");
    std::fs::write(&path, project_catalog::encode_project(&source, source.active().id).unwrap()).unwrap();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("destination")).unwrap();
    let before = app.project_manager.to_json().unwrap();
    app.test_start_project_import(vec![path.clone()]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !app.test_project_import_result_ready() {
        assert!(std::time::Instant::now() < deadline, "preflight did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap(),
    ).unwrap();
    drain_import(&mut app);
    assert!(app.status.kind().is_failure());
    let message = app.status.message().to_lowercase();
    assert!(message.contains("memory") || message.contains("budget"), "{message}");
    assert_eq!(app.project_manager.to_json().unwrap(), before);
    drop(std::fs::OpenOptions::new().write(true).open(&path).unwrap());
    drop(pressure);
    app.status.acknowledge();
    app.test_start_project_import(vec![path.clone()]).unwrap();
    drain_import(&mut app);
    assert!(!app.status.kind().is_failure(), "{}", app.status.message());
    assert!(app.project_manager.rows().iter().any(|row| row.name == "pressure source"));
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn project_import_holds_preflight_file_until_worker_reads_it() {
    let dir = std::env::temp_dir().join(format!("om9-held-project-import-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("source.om9project");
    let source = ProjectManager::new("held source");
    std::fs::write(&path, project_catalog::encode_project(&source, source.active().id).unwrap()).unwrap();
    let mut app = om9_app::ui::MatrixApp::new(Document::new("destination")).unwrap();
    app.test_start_project_import(vec![path.clone()]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        match std::fs::OpenOptions::new().write(true).open(&path) {
            Err(error) if error.raw_os_error() == Some(32) => break,
            Err(error) => panic!("unexpected source open failure: {error}"),
            Ok(handle) => drop(handle),
        }
        assert!(std::time::Instant::now() < deadline, "import did not hold source file");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(std::fs::remove_file(&path).is_err());
    let context = eframe::egui::Context::default();
    while app.test_project_import_preparing() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_import(ctx));
        assert!(std::time::Instant::now() < deadline, "import preflight did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    while !app.test_project_import_result_ready() {
        assert!(std::time::Instant::now() < deadline, "import decode did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.project_manager.rows().len(), 1, "result was applied before the UI accepted it");
    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err(), "source unlocked before publication");
    drain_import(&mut app);
    assert!(!app.status.kind().is_failure(), "{}", app.status.message());
    assert!(app.project_manager.rows().iter().any(|row| row.name == "held source"));
    drop(std::fs::OpenOptions::new().write(true).open(&path).unwrap());
    for stale in [false, true] {
        app.status.acknowledge();
        app.test_start_project_import(vec![path.clone()]).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.test_project_import_preparing() {
            let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_import(ctx));
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        while !app.test_project_import_result_ready() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if stale {
            let id = app.project_manager.active().id;
            app.project_manager.rename_project(id, "changed before publish").unwrap();
        } else {
            app.test_cancel_project_import();
        }
        let before = app.project_manager.to_json().unwrap();
        drain_import(&mut app);
        assert!(app.status.kind().is_failure());
        assert_eq!(app.project_manager.to_json().unwrap(), before);
        drop(std::fs::OpenOptions::new().write(true).open(&path).unwrap());
    }
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn panel_jobbag_preserves_nested_instance_and_embedded_texture_after_catalog_reload() {
    use om9_app::ui::MatrixApp;
    use om9_app::project_manager::BagFilter;
    use om9_core::{geometry::{Mesh, Transform}, model::Definition, resource::{Resource, ResourceKind},
        rhino_semantics::{MeshUv, PbrMaterial, embedded_texture_id, embedded_texture_uri, mesh_uv_of, set_mesh_uv}};
    for filtered in [false, true] {
    let dir = std::env::temp_dir().join(format!("om9-instance-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let catalog = dir.join("catalog.json");
    let texture = Resource::from_bytes("texture", ResourceKind::OpaqueBinary, b"authored texture", &Limits::default()).unwrap();
    let old_resource = texture.id;
    let mut mesh = Entity::new("textured mesh", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let mesh_geometry = mesh.geometry.clone();
    set_mesh_uv(&mut mesh, &MeshUv { uvs: vec![[0., 0.], [1., 0.], [0., 1.]], corners: vec![], material: "fixture".into(),
        pbr: PbrMaterial { texture: Some(embedded_texture_uri(old_resource).unwrap()), ..Default::default() } }, &Limits::default()).unwrap();
    let inner = Definition::new("inner", vec![mesh]);
    let mut nested = Entity::new("nested", Geometry::Instance(inner.id), None);
    nested.placement = Transform::translation([2., 3., 4.]);
    let nested_placement = nested.placement;
    let outer = Definition::new("outer", vec![nested]);
    let mut root = Entity::new("root", Geometry::Instance(outer.id), None);
    root.placement = Transform::translation([10., 20., 30.]);
    let root_placement = root.placement;
    let root_id = root.id;
    let mut document = Document::new("textured instance source");
    document.resources.insert(texture.id, texture);
    document.definitions.insert(inner.id, inner);
    document.definitions.insert(outer.id, outer);
    document.entities.insert(root_id, root);
    let context = eframe::egui::Context::default();
    let mut source = MatrixApp::new(document).unwrap();
    source.test_start_project_catalog_at(catalog.clone()).unwrap();
    drain_catalog_load(&mut source);
    drain_catalog_save(&mut source);
    source.controller.select_objects(&[root_id]).unwrap();
    ui_frame(&mut source, &context, vec![]);
    let capture = source.project_action_rects["capture-first-slot"].center();
    if filtered { source.test_capture_filtered_jobbag(0, BagFilter::Meshes).unwrap(); }
    else { ui_click(&mut source, &context, capture); }
    assert!(source.test_capture_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while source.test_capture_pending() || source.test_project_catalog_save_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| source.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(source.project_manager.active().slot(0).unwrap().snapshot.is_some());
    source.controller.delete_objects(&[root_id]).unwrap();
    drop(source);
    let mut target = MatrixApp::new(Document::new("instance destination")).unwrap();
    target.test_start_project_catalog_at(catalog).unwrap();
    drain_catalog_load(&mut target);
    let context = eframe::egui::Context::default();
    for _ in 0..4 { ui_frame(&mut target, &context, vec![]); }
    let bag = target.jobbag_slot_rects[&0].center();
    if filtered { target.test_insert_filtered_jobbag(0, BagFilter::Meshes).unwrap(); }
    else { ui_click(&mut target, &context, bag); }
    let root = target.controller.document().entities.values().next().unwrap();
    assert_ne!(root.id, root_id);
    assert_eq!(root.placement, root_placement);
    let Geometry::Instance(outer_id) = root.geometry else { panic!("missing outer instance") };
    let nested = &target.controller.document().definitions[&outer_id].members[0];
    assert_eq!(nested.placement, nested_placement);
    let Geometry::Instance(inner_id) = nested.geometry else { panic!("missing nested instance") };
    let mesh = &target.controller.document().definitions[&inner_id].members[0];
    assert_eq!(mesh.geometry, mesh_geometry);
    let uv = mesh_uv_of(mesh, &Limits::default()).unwrap().unwrap();
    let new_resource = embedded_texture_id(uv.pbr.texture.as_deref().unwrap()).unwrap();
    assert_ne!(new_resource, old_resource);
    assert_eq!(target.controller.document().resources[&new_resource].payload().unwrap(), b"authored texture");
    assert_eq!(target.controller.coordinator().undo_len(), 1);
    target.controller.undo().unwrap();
    assert!(target.controller.document().entities.is_empty());
    assert!(target.controller.document().definitions.is_empty());
    assert!(target.controller.document().resources.is_empty());
    target.controller.redo().unwrap();
    assert_eq!(target.controller.document().resources[&new_resource].payload().unwrap(), b"authored texture");
    drain_catalog_save(&mut target);
    drop(target);
    std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn panel_curve_filter_captures_only_curves_and_inserts_them() {
    use om9_app::ui::MatrixApp;
    use om9_core::geometry::{Curve, Mesh};
    let mut document = Document::new("mixed filter source");
    let point = Entity::new("excluded point", Geometry::Point([0., 0., 0.]), None);
    let curve = Entity::new("included curve", Geometry::Curve(Curve::Line {
        start: [1., 2., 3.], end: [4., 5., 6.], domain: [0., 1.],
    }), None);
    let (point_id, curve_id) = (point.id, curve.id);
    let expected = curve.geometry.clone();
    let surface = Entity::new("surface", Geometry::Surface(om9_core::surface_kernel::plane(
        [0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 4., 4., &Limits::default()).unwrap()), None);
    let mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let gem = om9_core::gem::gem_entity("gem", &om9_core::gem::GemSpec::round(4., 2.48).unwrap(), None).unwrap();
    let filtered = [
        ("load-filter-curves", expected.clone()),
        ("load-filter-surfaces", surface.geometry.clone()),
        ("load-filter-meshes", mesh.geometry.clone()),
        ("load-filter-gems", gem.geometry.clone()),
    ];
    let extra_ids = [surface.id, mesh.id, gem.id];
    document.entities.insert(point_id, point);
    document.entities.insert(curve_id, curve);
    for entity in [surface, mesh, gem] { document.entities.insert(entity.id, entity); }
    let mut source = MatrixApp::new(document).unwrap();
    source.controller.select_objects(&[point_id, curve_id, extra_ids[0], extra_ids[1], extra_ids[2]]).unwrap();
    let context = eframe::egui::Context::default();
    ui_frame(&mut source, &context, vec![]);
    let capture = source.project_action_rects["capture-first-slot"].center();
    for pressed in [true, false] {
        ui_frame(&mut source, &context, vec![eframe::egui::Event::PointerMoved(capture), eframe::egui::Event::PointerButton {
            pos: capture, button: eframe::egui::PointerButton::Secondary, pressed, modifiers: eframe::egui::Modifiers::NONE,
        }]);
    }
    ui_frame(&mut source, &context, vec![]);
    let curves = source.project_action_rects["capture-filter-curves"].center();
    ui_click(&mut source, &context, curves);
    assert!(source.test_capture_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while source.test_capture_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| source.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let snapshot = source.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap();
    assert_eq!(snapshot.captured_ids, vec![curve_id]);
    ui_frame(&mut source, &context, vec![]);
    let capture_all = source.project_action_rects["capture-second-slot"].center();
    ui_click(&mut source, &context, capture_all);
    assert!(source.test_capture_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while source.test_capture_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| source.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(source.project_manager.active().slot(1).unwrap().snapshot.as_ref().unwrap().captured_ids.len(), 5);
    for (slot, button_key, item_key, expected_id) in [
        (2, "capture-third-slot", "capture-filter-surfaces", extra_ids[0]),
        (3, "capture-fourth-slot", "capture-filter-meshes", extra_ids[1]),
        (4, "capture-fifth-slot", "capture-filter-gems", extra_ids[2]),
    ] {
        for _ in 0..12 {
            ui_frame(&mut source, &context, vec![]);
            let viewport = source.project_action_rects["numbered-strip"];
            let button = source.project_action_rects[button_key];
            if viewport.contains(button.center()) { break; }
            let direction = if button.center().x > viewport.right() { -100. } else { 100. };
            ui_frame(&mut source, &context, vec![eframe::egui::Event::PointerMoved(viewport.center()),
                eframe::egui::Event::MouseWheel { unit: eframe::egui::MouseWheelUnit::Point,
                    delta: eframe::egui::Vec2::new(direction, 0.), modifiers: eframe::egui::Modifiers::NONE,
                    phase: eframe::egui::TouchPhase::Move }]);
            for _ in 0..5 { ui_frame(&mut source, &context, vec![]); }
        }
        let button = source.project_action_rects[button_key].center();
        assert!(source.project_action_rects["numbered-strip"].contains(button));
        for pressed in [true, false] {
            ui_frame(&mut source, &context, vec![eframe::egui::Event::PointerMoved(button), eframe::egui::Event::PointerButton {
                pos: button, button: eframe::egui::PointerButton::Secondary, pressed, modifiers: eframe::egui::Modifiers::NONE,
            }]);
        }
        ui_frame(&mut source, &context, vec![]);
        let item = source.project_action_rects[item_key].center();
        ui_click(&mut source, &context, item);
        assert!(source.test_capture_pending(), "{item_key}");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while source.test_capture_pending() {
            let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| source.test_service_project_background(ctx));
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(source.project_manager.active().slot(slot).unwrap().snapshot.as_ref().unwrap().captured_ids, vec![expected_id]);
    }
    let manager = ProjectManager::from_json(&source.project_manager.to_json().unwrap(), "unused").unwrap();
    drop(source);
    let mut target = MatrixApp::new(Document::new("filter destination")).unwrap();
    target.project_manager = manager;
    let context = eframe::egui::Context::default();
    for _ in 0..4 { ui_frame(&mut target, &context, vec![]); }
    let bag = target.jobbag_slot_rects[&0].center();
    ui_click(&mut target, &context, bag);
    assert_eq!(target.controller.document().entities.len(), 1);
    assert_eq!(target.controller.document().entities.values().next().unwrap().geometry, expected);
    target.controller.undo().unwrap();
    assert!(target.controller.document().entities.is_empty());
    for (key, geometry) in filtered {
        ui_frame(&mut target, &context, vec![]);
        let mixed_bag = target.jobbag_slot_rects[&1].center();
        for pressed in [true, false] {
            ui_frame(&mut target, &context, vec![eframe::egui::Event::PointerMoved(mixed_bag), eframe::egui::Event::PointerButton {
                pos: mixed_bag, button: eframe::egui::PointerButton::Secondary, pressed, modifiers: eframe::egui::Modifiers::NONE,
            }]);
        }
        ui_frame(&mut target, &context, vec![]);
        let item = target.project_action_rects[key].center();
        ui_click(&mut target, &context, item);
        assert_eq!(target.controller.document().entities.len(), 1, "{key}");
        assert_eq!(target.controller.document().entities.values().next().unwrap().geometry, geometry, "{key}");
        target.controller.undo().unwrap();
        assert!(target.controller.document().entities.is_empty());
    }
}

#[test]
fn panel_capture_catalog_reopen_and_panel_insert_keep_placement_and_undo() {
    use om9_app::ui::MatrixApp;
    use om9_core::{associative::FeaturePort, exchange::RenderMaterial,
        geometry::{GeometryKind, Transform, Unit}, history::{Recipe, ReplayEnvironment, Tolerance},
        identity::MaterialId, model::Layer, request::{Selection, SourceBinding}, resource::{Resource, ResourceKind}};
    let dir = std::env::temp_dir().join(format!("om9-panel-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let catalog = dir.join("catalog.json");
    let mut document = Document::new("panel source");
    let mut point = Entity::new("placed point", Geometry::Point([1., 2., 3.]), None);
    point.placement = Transform::translation([12., 23., 34.]);
    let placement = point.placement;
    let id = point.id;
    document.entities.insert(id, point);
    let mut layer = Layer::new("hidden locked layer", [70, 80, 90, 255]);
    layer.visible = false;
    layer.locked = true;
    let material = RenderMaterial { id: MaterialId::new(), name: "authored gold".into(), diffuse: [220, 170, 40, 255] };
    layer.material = Some(material.id);
    document.exchange.materials.insert(material.id, material.clone());
    let layer_id = layer.id;
    document.layer_order.push(layer_id);
    document.layers.insert(layer_id, layer.clone());
    let mut hidden = Entity::new("hidden", Geometry::Point([4., 5., 6.]), Some(layer_id));
    hidden.attributes.hidden = true;
    hidden.attributes.visible = false;
    let mut locked = Entity::new("locked", Geometry::Point([7., 8., 9.]), Some(layer_id));
    locked.attributes.locked = true;
    locked.attributes.material = Some(material.id);
    let hidden_id = hidden.id;
    let locked_id = locked.id;
    document.entities.insert(hidden_id, hidden.clone());
    document.entities.insert(locked_id, locked.clone());
    let binding = SourceBinding { role: "source".into(), source: document.stamp(hidden_id).unwrap(), selection: Selection::Whole };
    let recipe = Recipe::new(document.id, "fixture point", vec![binding], vec![locked_id], ReplayEnvironment {
        command_version: 1, solver: "om9".into(), solver_version: "1".into(), schema_version: 1,
        length_unit: Unit::Millimeter, frame: Transform::identity(),
        tolerance: Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap(),
    }).with_ports(vec![FeaturePort::geometry("source", &[GeometryKind::Point])], vec![FeaturePort::geometry("result", &[GeometryKind::Point])]);
    document.history.recipes.insert(recipe.id, recipe.clone());
    let resource = Resource::from_bytes("design asset", ResourceKind::OpaqueBinary, b"snapshot resource", &Limits::default()).unwrap();
    let resource_id = resource.id;
    document.resources.insert(resource_id, resource);
    let context = eframe::egui::Context::default();
    let mut source = MatrixApp::new(document).unwrap();
    source.test_start_project_catalog_at(catalog.clone()).unwrap();
    drain_catalog_load(&mut source);
    drain_catalog_save(&mut source);
    source.controller.select_objects(&[id]).unwrap();
    ui_frame(&mut source, &context, vec![]);
    let capture = source.project_action_rects["capture-first-slot"].center();
    ui_click(&mut source, &context, capture);
    assert!(source.test_capture_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while source.test_capture_pending() || source.test_project_catalog_save_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| source.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "capture/catalog did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(source.project_manager.active().slot(0).unwrap().snapshot.is_some());
    for name in ["Master", "Creation", "Parts", "Render", "Output"] {
        let capture = named_card_button(&mut source, &context, name, true);
        ui_click(&mut source, &context, capture);
        assert!(source.test_capture_pending(), "{name} capture did not start");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while source.test_capture_pending() || source.test_project_catalog_save_pending() {
            let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| source.test_service_project_background(ctx));
            assert!(std::time::Instant::now() < deadline, "{name} capture/catalog did not finish");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(source.project_manager.named_master_snapshot(name).is_some());
    }
    source.controller.delete_objects(&[id]).unwrap();
    assert_eq!(source.controller.document().entities.len(), 2);
    drop(source);

    let mut target = MatrixApp::new(Document::new("panel destination")).unwrap();
    target.test_start_project_catalog_at(catalog.clone()).unwrap();
    drain_catalog_load(&mut target);
    assert!(target.project_manager.active().slot(0).unwrap().snapshot.is_some());
    let context = eframe::egui::Context::default();
    for _ in 0..4 { ui_frame(&mut target, &context, vec![]); }
    let bag_rect = target.jobbag_slot_rects[&0];
    assert!(target.sidebar_content_rect.unwrap().contains_rect(bag_rect), "bag outside sidebar: {bag_rect:?}, {:?}", target.sidebar_content_rect);
    assert!(target.project_action_rects["capture-first-slot"].top() >= bag_rect.bottom(), "numbered controls overlap thumbnail");
    assert!(target.project_action_rects["save-master"].top() >= target.project_bag_rects["Master"].bottom(), "named controls overlap thumbnail");
    for (button, file, expected_count) in [("save-first-slot", "selection.om9", 1), ("save-master", "master.om9", 3)] {
        let path = dir.join(file);
        let save = target.project_action_rects[button].center();
        ui_click(&mut target, &context, save);
        target.test_complete_project_file_request(&path).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while target.test_project_export_pending() {
            let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| target.test_service_project_background(ctx));
            assert!(std::time::Instant::now() < deadline, "{button} export did not finish");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!target.status.kind().is_failure(), "{button}: {}", target.status.message());
        let reopened = MatrixApp::from_file(&path).unwrap();
        let exported = reopened.controller.document();
        assert_eq!(exported.entities.len(), expected_count, "{button}");
        assert_eq!(exported.entities[&id].placement, placement, "{button}");
        if expected_count == 3 {
            assert_eq!(exported.entities[&hidden_id], hidden);
            assert_eq!(exported.entities[&locked_id], locked);
            assert_eq!(exported.layers[&layer_id], layer);
            assert_eq!(exported.exchange.materials[&material.id], material);
            let reopened_recipe = &exported.history.recipes[&recipe.id];
            assert_eq!(reopened_recipe.command, recipe.command);
            assert_eq!(reopened_recipe.inputs[0].source.object, hidden_id);
            assert_eq!(reopened_recipe.outputs, vec![locked_id]);
            assert!(matches!(reopened_recipe.state, om9_core::history::EvaluationState::Unresolved(_)));
            assert_eq!(exported.resources[&resource_id].payload().unwrap(), b"snapshot resource");
        }
    }
    let bag = bag_rect.center();
    ui_frame(&mut target, &context, vec![eframe::egui::Event::PointerMoved(bag)]);
    ui_click(&mut target, &context, bag);
    assert!(!target.controller.document().entities.is_empty(), "panel insertion failed: {}", target.status.message());
    let inserted = target.controller.document().entities.values().next().unwrap();
    assert_ne!(inserted.id, id);
    assert_eq!(inserted.placement, placement);
    assert_eq!(target.controller.coordinator().undo_len(), 1);
    target.controller.undo().unwrap();
    assert!(target.controller.document().entities.is_empty());
    target.controller.redo().unwrap();
    assert_eq!(target.controller.document().entities.values().next().unwrap().placement, placement);
    for name in ["Master", "Creation", "Parts", "Render", "Output"] {
        target.controller.submit("Point").unwrap();
        target.controller.submit("8,0,0").unwrap();
        let open = named_card_button(&mut target, &context, name, false);
        ui_click(&mut target, &context, open);
        ui_frame(&mut target, &context, vec![]);
        assert_eq!(target.controller.file_operation().unwrap().phase(), om9_app::file_lifecycle::FilePhase::Unsaved);
        let discard = target.file_control_rect(om9_app::ui::FileControl::Discard).unwrap().center();
        ui_click(&mut target, &context, discard);
        ui_frame(&mut target, &context, vec![]);
        let restored = target.controller.document();
        assert_eq!(restored.entities.len(), 3, "{name}");
        assert_eq!(restored.entities[&id].placement, placement, "{name}");
        assert_eq!(restored.entities[&hidden_id], hidden, "{name}");
        assert_eq!(restored.entities[&locked_id], locked, "{name}");
        assert_eq!(restored.layers[&layer_id], layer, "{name}");
        assert_eq!(restored.exchange.materials[&material.id], material, "{name}");
        assert_eq!(restored.history.recipes[&recipe.id], recipe, "{name}");
        assert_eq!(restored.history.owner(locked_id), Some(recipe.id), "{name}");
        assert_eq!(restored.resources[&resource_id].payload().unwrap(), b"snapshot resource", "{name}");
    }
    drain_catalog_save(&mut target);
    drop(target);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pending_import_and_catalog_load_do_not_replace_open_info_draft() {
    use om9_app::ui::MatrixApp;
    let dir = std::env::temp_dir().join(format!("om9-draft-race-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let disk = ProjectManager::new("on disk");
    let project = dir.join("import.om9project");
    let bytes = project_catalog::encode_project(&disk, disk.active().id).unwrap();
    project_catalog::publish_bytes(&project, &bytes).unwrap();
    let catalog = dir.join("catalog.json");
    project_catalog::save(&catalog, &disk, None).unwrap();

    for load_catalog in [false, true] {
        let mut app = MatrixApp::new(Document::new("current project")).unwrap();
        if load_catalog { app.test_start_project_catalog_at(catalog.clone()).unwrap(); }
        else { app.test_start_project_import(vec![project.clone()]).unwrap(); }
        let before = app.project_manager.to_json().unwrap();
        let context = eframe::egui::Context::default();
        ui_frame(&mut app, &context, vec![]);
        let manager = app.project_action_rects["manager"].center();
        ui_click(&mut app, &context, manager);
        let tile = app.project_action_rects["manager-project"].center();
        ui_click(&mut app, &context, tile);
        assert!(app.project_action_rects.contains_key("manager-info-name"));
        let name = app.project_action_rects["manager-info-name"].center();
        ui_click(&mut app, &context, name);
        ui_frame(&mut app, &context, vec![eframe::egui::Event::Text("X".into())]);
        assert!(matches!(app.test_start_project_import(vec![project.clone()]), Err(om9_core::Error::Busy)));
        if load_catalog { drain_catalog_load(&mut app); } else { drain_import(&mut app); }
        assert_eq!(app.project_manager.to_json().unwrap(), before);
        ui_frame(&mut app, &context, vec![]);
        assert!(app.project_action_rects.contains_key("manager-info-name"), "draft was closed during worker completion");
        for _ in 0..12 {
            let viewport = app.project_action_rects["manager-info-viewport"];
            let save = app.project_action_rects["manager-info-save"];
            if viewport.contains_rect(save) { break; }
            ui_frame(&mut app, &context, vec![eframe::egui::Event::PointerMoved(viewport.center()),
                eframe::egui::Event::MouseWheel { unit: eframe::egui::MouseWheelUnit::Point,
                    delta: eframe::egui::Vec2::new(0., -120.), modifiers: eframe::egui::Modifiers::NONE,
                    phase: eframe::egui::TouchPhase::Move }]);
            for _ in 0..4 { ui_frame(&mut app, &context, vec![]); }
        }
        let save = app.project_action_rects["manager-info-save"];
        assert!(app.project_action_rects["manager-info-viewport"].contains_rect(save));
        ui_click(&mut app, &context, save.center());
        assert!(app.project_manager.active().name.contains('X'), "draft edit was lost");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
#[ignore = "requires memory and disk for a 257 MiB resource"]
fn large_catalog_and_project_archive_follow_configured_budget() {
    use om9_core::resource::{Resource, ResourceKind};
    let limits = Limits { max_bytes: 4 * 1024 * 1024 * 1024, ..Limits::default() };
    let low = Limits::default();
    let dir = std::env::temp_dir().join(format!("om9-large-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let catalog = dir.join("catalog.json");
    let project = dir.join("large.om9project");
    let data = vec![0x5a; 257 * 1024 * 1024];
    let resource = Resource::from_bytes("large authored resource", ResourceKind::OpaqueBinary, &data, &limits).unwrap();
    drop(data);
    let resource_id = resource.id;
    let resource_hash = resource.sha256();
    let mut document = Document::new("large snapshot");
    document.resources.insert(resource_id, resource);
    let entity = Entity::new("point", Geometry::Point([4., 5., 6.]), None);
    document.entities.insert(entity.id, entity);
    let controller = Controller::new(document, limits).unwrap();
    let mut capture_app = om9_app::ui::MatrixApp::test_from_controller(controller).unwrap();
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    capture_app.test_start_named_capture("Master", None).unwrap();
    while capture_app.test_capture_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| capture_app.test_service_capture(ctx));
        assert!(std::time::Instant::now() < deadline, "large document capture did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!capture_app.status.kind().is_failure());
    assert!(capture_app.project_manager.named_master_snapshot("Master").unwrap().archive.as_ref().unwrap().len() > 256 * 1024 * 1024);
    assert!(project_catalog::encode_project_with_limits(&capture_app.project_manager, capture_app.project_manager.active().id, &low).is_err());
    capture_app.test_start_project_catalog_at(catalog.clone()).unwrap();
    while capture_app.test_project_catalog_load_pending() || capture_app.test_project_catalog_save_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| capture_app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "large catalog save did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!capture_app.status.kind().is_failure());
    assert!(catalog.is_file());
    drop(capture_app);
    let controller = Controller::new(Document::new("large catalog destination"), limits).unwrap();
    let mut reload_app = om9_app::ui::MatrixApp::test_from_controller(controller).unwrap();
    reload_app.test_start_project_catalog_at(catalog.clone()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    while reload_app.test_project_catalog_load_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| reload_app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "large catalog load did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!reload_app.status.kind().is_failure());
    assert!(reload_app.project_manager.named_master_snapshot("Master").unwrap().archive.as_ref().unwrap().len() > 256 * 1024 * 1024);
    let bag_export = dir.join("large-master.om9");
    reload_app.test_export_named_jobbag("Master", &bag_export).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    while reload_app.test_project_export_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| reload_app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "large jobbag export did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!reload_app.status.kind().is_failure());
    assert_eq!(std::fs::read(&bag_export).unwrap().as_slice(), reload_app.project_manager.named_master_snapshot("Master").unwrap().archive.as_deref().unwrap());
    std::fs::remove_file(bag_export).unwrap();
    let new_root = dir.join("new-root");
    std::fs::create_dir(&new_root).unwrap();
    let settings = dir.join("new-root-settings.json");
    reload_app.test_start_project_root_change(&new_root, &settings).unwrap();
    assert!(!settings.exists());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    while reload_app.test_project_root_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| reload_app.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "large root change did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!reload_app.status.kind().is_failure());
    let configured: std::path::PathBuf = serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(configured, std::path::absolute(&new_root).unwrap());
    let (moved, _) = project_catalog::load_with_limits_and_admission(&new_root.join("project-catalog.json"), "unused", &limits, |_| Ok(())).unwrap().unwrap();
    assert_eq!(moved.named_master_snapshot("Master").unwrap().archive.as_deref(), reload_app.project_manager.named_master_snapshot("Master").unwrap().archive.as_deref());
    drop(moved);
    drop(reload_app);
    assert!(project_catalog::load_with_limits_and_admission(&catalog, "unused", &low, |_| Ok(())).is_err());
    let (loaded, loaded_hash) = project_catalog::load_with_limits_and_admission(&catalog, "unused", &limits, |_| Ok(())).unwrap().unwrap();
    assert_ne!(loaded_hash, [0; 32]);
    let bytes = project_catalog::encode_project_with_limits(&loaded, loaded.active().id, &limits).unwrap();
    project_catalog::publish_bytes(&project, &bytes).unwrap();
    drop(bytes);
    drop(loaded);
    let file = std::fs::File::open(&project).unwrap();
    assert!(project_catalog::read_project_file_with_limits(file, &Cancellation::default(), &low).is_err());
    let file = std::fs::File::open(&project).unwrap();
    let (manifest, assets) = project_catalog::read_project_file_with_limits(file, &Cancellation::default(), &limits).unwrap();
    let mut imported = ProjectManager::new("existing");
    project_catalog::import_project_for_test(&mut imported, &manifest, &assets, &limits, &Cancellation::default()).unwrap();
    drop(assets);
    let restored = om9_archive::Archive::decode(imported.named_master_snapshot("Master").unwrap().archive.as_deref().unwrap(), &limits).unwrap();
    assert_eq!(restored.resources[&resource_id].sha256(), resource_hash);
    assert_eq!(restored.resources[&resource_id].payload().unwrap().len(), 257 * 1024 * 1024);
    drop(restored);
    drop(imported);
    let controller = Controller::new(Document::new("large import destination"), limits).unwrap();
    let mut app = om9_app::ui::MatrixApp::test_from_controller(controller).unwrap();
    app.test_start_project_import(vec![project.clone()]).unwrap();
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    while app.test_project_import_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_import(ctx));
        assert!(std::time::Instant::now() < deadline, "large project import did not finish");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!app.status.kind().is_failure());
    assert!(app.project_manager.named_master_snapshot("Master").unwrap().archive.as_ref().unwrap().len() > 256 * 1024 * 1024);
    drop(app);
    let temp_root = std::fs::canonicalize(std::env::temp_dir()).unwrap();
    let owned_dir = std::fs::canonicalize(dir).unwrap();
    assert_eq!(owned_dir.parent(), Some(temp_root.as_path()));
    std::fs::remove_dir_all(owned_dir).unwrap();
}

#[test]
fn workspace_catalog_and_project_archive_preserve_assets_and_refuse_corruption() {
    let temp_root = std::fs::canonicalize(std::env::temp_dir()).unwrap();
    let dir = temp_root.join(format!("om9-catalog-integration-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let catalog = dir.join("catalog.json");
    let project_path = dir.join("project.om9project");
    let mut document = Document::new("catalog source");
    let entity = Entity::new("authored point", Geometry::Point([4., 5., 6.]), None);
    let id = entity.id;
    document.entities.insert(id, entity.clone());
    let controller = Controller::new(document, Limits::default()).unwrap();
    let archive = controller.copy_jobbag_objects_om9(&[id]).unwrap();
    let mut manager = ProjectManager::new("Nhẫn mẫu");
    manager.set_notes("Đơn hàng thử").unwrap();
    manager.capture_archive(0, vec![entity], controller.document(), archive.clone()).unwrap();
    let (snapshot, full_archive) = controller.capture_jobbag_document().unwrap();
    manager.capture_named_version("Master", snapshot, full_archive).unwrap();
    let saved_hash = project_catalog::save(&catalog, &manager, None).unwrap();
    let (loaded, loaded_hash) = project_catalog::load_with_admission(&catalog, "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(saved_hash, loaded_hash);
    assert_eq!(loaded.active().name, "Nhẫn mẫu");
    assert_eq!(loaded.active().notes, "Đơn hàng thử");
    assert_eq!(loaded.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_deref(), Some(archive.as_slice()));
    assert!(loaded.named_master_snapshot("Master").unwrap().archive.is_some());
    let child_dir = dir.join("child");
    std::fs::create_dir(&child_dir).unwrap();
    let child_catalog = child_dir.join("catalog.json");
    std::fs::copy(&catalog, &child_catalog).unwrap();
    let child_assets = child_dir.join("catalog.json.assets");
    std::fs::create_dir(&child_assets).unwrap();
    for entry in std::fs::read_dir(dir.join("catalog.json.assets")).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), child_assets.join(entry.file_name())).unwrap();
    }
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "catalog_child_reads_workspace", "--ignored", "--nocapture"])
        .env("OM9_CATALOG_INTEGRATION_PATH", &child_catalog)
        .env("OM9_PROJECT_CATALOG", &child_catalog).output().unwrap();
    assert!(child.status.success(), "child catalog load failed: {}", String::from_utf8_lossy(&child.stderr));
    let mut legacy = serde_json::to_value(&manager).unwrap();
    legacy.as_object_mut().unwrap().remove("schema_version");
    legacy["rows"][0]["slots"].as_array_mut().unwrap().truncate(3);
    legacy["rows"][0]["metadata"].as_object_mut().unwrap().remove("ring_rail_radius_mm");
    let mut migrated = ProjectManager::from_json(&serde_json::to_vec(&legacy).unwrap(), "unused").unwrap();
    assert_eq!(migrated.active().slots.len(), 10);
    assert_eq!(migrated.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_deref(), Some(archive.as_slice()));
    migrated.clear_slot(0).unwrap();
    let compacted = migrated.capture_archive(9, vec![controller.document().entities[&id].clone()], controller.document(), archive.clone()).unwrap();
    assert_eq!(migrated.active().slots.len(), 11);
    migrated.compact_slots();
    assert_eq!(migrated.active().slots.len(), 10);
    assert_eq!(migrated.active().slot(0).unwrap().snapshot.as_ref().unwrap().id, compacted);
    assert!(ProjectManager::from_json(br#"{"schema_version":2,"schema_version":2}"#, "unused").is_err());
    let catalog_before = std::fs::read(&catalog).unwrap();
    assert!(project_catalog::save(&catalog, &loaded, None).is_err());
    assert_eq!(std::fs::read(&catalog).unwrap(), catalog_before);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let locked = std::fs::OpenOptions::new().read(true).write(true).share_mode(0).open(dir.join("catalog.json.lock")).unwrap();
        let result = project_catalog::save(&catalog, &loaded, Some(saved_hash));
        assert!(matches!(result, Err(om9_core::Error::Busy)), "unexpected lock result: {result:?}");
        assert_eq!(std::fs::read(&catalog).unwrap(), catalog_before);
        drop(locked);
    }

    let bytes = project_catalog::encode_project(&loaded, loaded.active().id).unwrap();
    project_catalog::publish_bytes(&project_path, &bytes).unwrap();
    let file = project_catalog::open_project_for_import(&project_path, bytes.len() as u64).unwrap();
    let (manifest, assets) = project_catalog::read_project_file(file, &Cancellation::default()).unwrap();
    assert!(!assets.is_empty());
    let canceled = Cancellation::default(); canceled.cancel();
    let mut bundle_destination = ProjectManager::new("bundle existing");
    let original = bundle_destination.to_json().unwrap();
    assert!(matches!(project_catalog::import_project_for_test(&mut bundle_destination, &manifest, &assets, &Limits::default(), &canceled), Err(om9_core::Error::Canceled)));
    assert_eq!(bundle_destination.to_json().unwrap(), original);
    project_catalog::import_project_for_test(&mut bundle_destination, &manifest, &assets, &Limits::default(), &Cancellation::default()).unwrap();
    assert_eq!(bundle_destination.rows().len(), 2);
    assert_eq!(bundle_destination.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.as_deref(), Some(archive.as_slice()));
    assert!(bundle_destination.named_master_snapshot("Master").unwrap().archive.is_some());
    let mut bad_nested = ProjectManager::new("bad nested archive");
    bad_nested.capture_archive(0, vec![controller.document().entities[&id].clone()], controller.document(), b"not an OM9 archive".to_vec()).unwrap();
    let bad_bundle = project_catalog::encode_project(&bad_nested, bad_nested.active().id).unwrap();
    let bad_path = dir.join("bad-nested.om9project");
    project_catalog::publish_bytes(&bad_path, &bad_bundle).unwrap();
    let file = project_catalog::open_project_for_import(&bad_path, bad_bundle.len() as u64).unwrap();
    let (bad_manifest, bad_assets) = project_catalog::read_project_file(file, &Cancellation::default()).unwrap();
    let before_bad = bundle_destination.to_json().unwrap();
    assert!(project_catalog::import_project_for_test(&mut bundle_destination, &bad_manifest, &bad_assets, &Limits::default(), &Cancellation::default()).is_err());
    assert_eq!(bundle_destination.to_json().unwrap(), before_bad);
    let mut batch = om9_app::ui::MatrixApp::new(Document::new("batch existing")).unwrap();
    let before_batch = batch.project_manager.to_json().unwrap();
    batch.test_start_project_import(vec![project_path.clone(), bad_path.clone()]).unwrap();
    drain_import(&mut batch);
    assert!(batch.status.kind().is_failure());
    assert_eq!(batch.project_manager.to_json().unwrap(), before_batch);
    let mut canceled_batch = om9_app::ui::MatrixApp::new(Document::new("cancel existing")).unwrap();
    let before_cancel = canceled_batch.project_manager.to_json().unwrap();
    canceled_batch.test_start_project_import(vec![project_path.clone()]).unwrap();
    canceled_batch.test_cancel_project_import();
    drain_import(&mut canceled_batch);
    assert!(canceled_batch.status.kind().is_failure());
    assert_eq!(canceled_batch.project_manager.to_json().unwrap(), before_cancel);
    let mut stale_batch = om9_app::ui::MatrixApp::new(Document::new("stale existing")).unwrap();
    stale_batch.test_start_project_import(vec![project_path.clone()]).unwrap();
    let active = stale_batch.project_manager.active().id;
    stale_batch.project_manager.rename_project(active, "edited during import").unwrap();
    let after_edit = stale_batch.project_manager.to_json().unwrap();
    drain_import(&mut stale_batch);
    assert!(stale_batch.status.kind().is_failure());
    assert_eq!(stale_batch.project_manager.to_json().unwrap(), after_edit);
    let mut successful_batch = om9_app::ui::MatrixApp::new(Document::new("success existing")).unwrap();
    successful_batch.test_start_project_import(vec![project_path.clone(), project_path.clone()]).unwrap();
    drain_import(&mut successful_batch);
    assert!(!successful_batch.status.kind().is_failure());
    assert_eq!(successful_batch.project_manager.rows().len(), 3);
    assert_ne!(successful_batch.project_manager.rows()[1].id, successful_batch.project_manager.rows()[2].id);
    let file = project_catalog::open_project_for_import(&project_path, bytes.len() as u64).unwrap();
    assert!(matches!(project_catalog::read_project_file(file, &canceled), Err(om9_core::Error::Canceled)));
    let mut corrupted = bytes.clone();
    *corrupted.last_mut().unwrap() ^= 1;
    std::fs::write(&project_path, &corrupted).unwrap();
    let file = project_catalog::open_project_for_import(&project_path, corrupted.len() as u64).unwrap();
    assert!(project_catalog::read_project_file(file, &Cancellation::default()).is_err());

    let inline = loaded.export_project(loaded.active().id).unwrap();
    let mut destination = ProjectManager::new("existing");
    let existing_snapshot = destination.capture_archive(0, vec![controller.document().entities[&id].clone()], controller.document(), archive.clone()).unwrap();
    let before = destination.to_json().unwrap();
    assert!(destination.import_project(b"corrupt", &Limits::default()).is_err());
    assert_eq!(destination.to_json().unwrap(), before);
    let imported = destination.import_project(&inline, &Limits::default()).unwrap();
    assert_ne!(imported, destination.rows()[0].id);
    assert_eq!(destination.rows().len(), 2);
    assert_eq!(destination.active().name, "Nhẫn mẫu");
    assert_ne!(destination.active().slot(0).unwrap().snapshot.as_ref().unwrap().id, existing_snapshot);
    let resolved = std::fs::canonicalize(&dir).unwrap();
    assert_eq!(resolved.parent(), Some(temp_root.as_path()));
    std::fs::remove_dir_all(resolved).unwrap();
}

#[test]
#[ignore = "child process invoked by workspace catalog integration test"]
fn catalog_child_reads_workspace() {
    std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(catalog_child_reads_workspace_impl).unwrap().join().unwrap();
}

fn catalog_child_reads_workspace_impl() {
    let path = std::path::PathBuf::from(std::env::var_os("OM9_CATALOG_INTEGRATION_PATH").unwrap());
    let (manager, _) = project_catalog::load_with_admission(&path, "unused", |_| Ok(())).unwrap().unwrap();
    assert_eq!(manager.active().name, "Nhẫn mẫu");
    assert!(manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.is_some());
    assert!(manager.named_master_snapshot("Master").unwrap().archive.is_some());
    let mut stale = om9_app::ui::MatrixApp::new(Document::new("initial")).unwrap();
    stale.test_start_project_catalog();
    assert!(stale.test_project_catalog_load_pending());
    let id = stale.project_manager.active().id;
    stale.project_manager.rename_project(id, "edited before load completes").unwrap();
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while stale.test_project_catalog_load_pending() {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| stale.test_service_project_background(ctx));
        assert!(std::time::Instant::now() < deadline, "stale catalog load did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(stale.project_manager.active().name, "edited before load completes");
    assert_eq!(stale.status.kind(), om9_app::status_surface::StatusKind::Warning);
    let mut app = om9_app::ui::MatrixApp::new(Document::new("fresh process")).unwrap();
    app.test_start_project_catalog();
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.project_manager.active().name != "Nhẫn mẫu" {
        let _ = context.run_logic(&eframe::egui::RawInput::default(), |ctx| app.test_service_project_background(ctx));
        let mut output = context.run_ui(eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::Vec2::new(1280., 900.))),
            ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
        assert!(std::time::Instant::now() < deadline, "application did not hydrate workspace catalog");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.is_some());
    assert!(app.project_manager.named_master_snapshot("Master").unwrap().archive.is_some());

    let mut legacy_document = Document::new("embedded document");
    let point = Entity::new("legacy point", Geometry::Point([7., 8., 9.]), None);
    legacy_document.entities.insert(point.id, point.clone());
    let mut legacy = ProjectManager::new("Embedded legacy");
    let old_snapshot = legacy.capture(0, vec![point], &legacy_document).unwrap();
    legacy.capture_master(&legacy_document).unwrap();
    let mut source = Controller::new(legacy_document, Limits::default()).unwrap();
    source.store_project_manager(&legacy).unwrap();
    let original_document = source.document().clone();
    let embedded_before = original_document.resources.values().find(|r| r.name == "om9.project-manager.v1").unwrap().payload().unwrap().to_vec();

    let mut partially_bad = legacy.clone();
    partially_bad.create_project("bad second row", original_document.revision).unwrap();
    let root = original_document.entities.values().next().unwrap().clone();
    partially_bad.capture_archive(0, vec![root], &original_document, b"bad nested archive".to_vec()).unwrap();
    let mut corrupt_source = Controller::new(original_document.clone(), Limits::default()).unwrap();
    corrupt_source.store_project_manager(&partially_bad).unwrap();
    let mut bad_import = om9_app::ui::MatrixApp::new(corrupt_source.document().clone()).unwrap();
    bad_import.test_start_project_catalog();
    drain_catalog_load(&mut bad_import);
    let before_bad_import = bad_import.project_manager.to_json().unwrap();
    bad_import.test_start_embedded_project_import().unwrap();
    drain_import(&mut bad_import);
    assert!(bad_import.status.kind().is_failure());
    assert_eq!(bad_import.project_manager.to_json().unwrap(), before_bad_import);

    let mut stale_embedded = om9_app::ui::MatrixApp::new(original_document.clone()).unwrap();
    stale_embedded.test_start_project_catalog();
    drain_catalog_load(&mut stale_embedded);
    let before_cancel = stale_embedded.project_manager.to_json().unwrap();
    stale_embedded.test_start_embedded_project_import().unwrap();
    stale_embedded.test_cancel_project_import();
    drain_import(&mut stale_embedded);
    assert!(stale_embedded.status.kind().is_failure());
    assert_eq!(stale_embedded.project_manager.to_json().unwrap(), before_cancel);
    stale_embedded.status.acknowledge();
    stale_embedded.test_start_embedded_project_import().unwrap();
    stale_embedded.controller.submit("Point").unwrap();
    stale_embedded.controller.submit("1,0,0").unwrap();
    let after_edit = stale_embedded.project_manager.to_json().unwrap();
    drain_import(&mut stale_embedded);
    assert!(stale_embedded.status.kind().is_failure());
    assert_eq!(stale_embedded.project_manager.to_json().unwrap(), after_edit);

    let mut embedded_app = om9_app::ui::MatrixApp::new(original_document).unwrap();
    embedded_app.test_start_project_catalog();
    drain_catalog_load(&mut embedded_app);
    assert_eq!(embedded_app.project_manager.active().name, "Nhẫn mẫu");
    embedded_app.stage_project_state().unwrap();
    let resource = embedded_app.controller.document().resources.values().find(|r| r.name == "om9.project-manager.v1").unwrap();
    assert_eq!(resource.payload().unwrap(), embedded_before);
    let import_context = eframe::egui::Context::default();
    ui_frame(&mut embedded_app, &import_context, vec![]);
    let manager_button = embedded_app.project_action_rects["manager"].center();
    ui_click(&mut embedded_app, &import_context, manager_button);
    let import_button = embedded_app.project_action_rects["import-document-projects"].center();
    ui_click(&mut embedded_app, &import_context, import_button);
    assert!(embedded_app.test_project_import_pending(), "import button {import_button:?}, status {}", embedded_app.status.line());
    drain_import(&mut embedded_app);
    assert!(!embedded_app.status.kind().is_failure());
    assert_eq!(embedded_app.project_manager.rows().len(), 2);
    let imported = embedded_app.project_manager.active();
    assert_eq!(imported.name, "Embedded legacy");
    let bag = imported.slot(0).unwrap().snapshot.as_ref().unwrap();
    assert_ne!(bag.id, old_snapshot);
    assert!(bag.archive.is_none());
    assert_eq!(bag.entities.len(), 1);
    assert!(embedded_app.project_manager.named_master_snapshot("Master").unwrap().document.is_some());
    embedded_app.stage_project_state().unwrap();
    let resource = embedded_app.controller.document().resources.values().find(|r| r.name == "om9.project-manager.v1").unwrap();
    assert_eq!(resource.payload().unwrap(), embedded_before);
    let save_path = path.parent().unwrap().join("legacy-document.om9");
    embedded_app.controller.begin_file_operation(om9_app::file_lifecycle::FileKind::SaveAs).unwrap();
    let token = embedded_app.controller.file_operation().unwrap().token();
    embedded_app.controller.accept_file_path(token, &save_path).unwrap();
    let reopened = om9_app::ui::MatrixApp::from_file(&save_path).unwrap();
    assert_eq!(reopened.project_manager.active().name, "Embedded legacy");
    assert!(reopened.project_manager.active().slot(0).unwrap().snapshot.is_some());
    embedded_app.controller.submit("Point").unwrap();
    embedded_app.controller.submit("2,0,0").unwrap();
    let context = eframe::egui::Context::default();
    ui_frame(&mut embedded_app, &context, vec![]);
    let open = embedded_app.project_action_rects["restore-master"].center();
    ui_click(&mut embedded_app, &context, open);
    ui_frame(&mut embedded_app, &context, vec![]);
    let save = embedded_app.file_control_rect(om9_app::ui::FileControl::Save).unwrap().center();
    ui_click(&mut embedded_app, &context, save);
    ui_frame(&mut embedded_app, &context, vec![]);
    assert_eq!(embedded_app.controller.document().entities.len(), 1);
    let saved = om9_app::ui::MatrixApp::from_file(&save_path).unwrap();
    let resource = saved.controller.document().resources.values().find(|r| r.name == "om9.project-manager.v1").unwrap();
    assert_eq!(resource.payload().unwrap(), embedded_before);
    drain_catalog_save(&mut embedded_app);
    let mut opened = om9_app::ui::MatrixApp::new(Document::new("workspace already loaded")).unwrap();
    opened.test_start_project_catalog();
    drain_catalog_load(&mut opened);
    opened.controller.begin_file_operation(om9_app::file_lifecycle::FileKind::Open).unwrap();
    let token = opened.controller.file_operation().unwrap().token();
    opened.controller.accept_file_path(token, &save_path).unwrap();
    opened.reload_project_state().unwrap();
    assert!(opened.project_manager.rows().iter().any(|row| row.name == "Nhẫn mẫu"));
    assert!(opened.project_manager.rows().iter().any(|row| row.name == "Embedded legacy"));
    opened.stage_project_state().unwrap();
    let resource = opened.controller.document().resources.values().find(|r| r.name == "om9.project-manager.v1").unwrap();
    assert_eq!(resource.payload().unwrap(), embedded_before);
}
