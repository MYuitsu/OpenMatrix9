// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows, feature = "occt-brep"))]

use om9_core::{brep::Brep, budget::Limits, transaction::Cancellation};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, path::PathBuf};

#[test]
#[ignore = "requires a configured canonical BRep worker executable"]
fn configured_canonical_worker_restores_box_in_contained_process() {
    use om9_core::{brep::BrepSourceAccess, budget::Usage, model::Document, transaction::{TransactionCoordinator, World}};
    let path = PathBuf::from(std::env::var_os("OM9_CANONICAL_BREP_WORKER").expect("canonical BRep worker path"));
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&path).unwrap()).into();
    let host = om9_app::brep_restore_host::BrepRestoreHost::admit(&path, digest, ProcessLimits::default()).unwrap();
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("release worker")], &limits).unwrap(), limits).unwrap();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let source = om9_kernel::brep::capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_source();
    let held = Usage { bytes: seed.storage_usage(&limits).unwrap().bytes + source.storage_bytes().unwrap(), ..Usage::default() };
    let restored = host.restore(&coordinator, source.source(), source.topology(), source.shape_set(), 0.1, 0.3, held, &Cancellation::default()).unwrap();
    assert_eq!(restored.candidate().source(), source.source());
    assert!(!restored.candidate().display().triangles.is_empty());
}

#[test]
#[ignore = "writes an explicitly requested v7 release-validation fixture"]
fn create_native_v7_release_fixture() {
    use om9_core::{geometry::Geometry, model::{Document, Entity}, transaction::{TransactionCoordinator, World}};
    let path = PathBuf::from(std::env::var_os("OM9_V7_FIXTURE_OUT").expect("fixture output path"));
    assert!(path.is_absolute() && !path.exists());
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let mut document = Document::with_template_tolerance("release native v7 fixture");
    let entity = Entity::new("native Box", Geometry::NativeBrep(native.canonical_source()), None);
    document.entities.insert(entity.id, entity);
    let owner = TransactionCoordinator::new(World::new(vec![Document::new("fixture owner")], &limits).unwrap(), limits).unwrap();
    let bytes = om9_archive::canonical_brep::encode_mixed_document_candidate(&document, &[],
        &owner.external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    use std::io::Write;
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(&bytes).unwrap();
    writer.flush().unwrap();
}

#[test]
#[ignore = "requires a separately built OM9_NATIVE_BREP_WORKER"]
fn contained_native_payload_rebuilds_display_and_rejects_bad_requests() {
    let path = PathBuf::from(std::env::var_os("OM9_NATIVE_BREP_WORKER").expect("native BRep worker path"));
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&path).unwrap()).into();
    let program = NativeProgram::admit(&path, digest).unwrap();
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    struct Temp(PathBuf);
    impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let input = Temp(std::env::temp_dir().join(format!("om9-native-brep-worker-{}.json", uuid::Uuid::new_v4())));
    let process = ProcessLimits::default();
    let run = |bytes: &[u8], fingerprint: String, process: &ProcessLimits, cancel: &Cancellation| {
        std::fs::write(&input.0, bytes).unwrap();
        let args: [OsString; 4] = [input.0.as_os_str().to_owned(), fingerprint.into(),
            (16 * 1024 * 1024u64).to_string().into(), process.native_committed_bytes.to_string().into()];
        program.run(&args, process, cancel)
    };
    let hash = |bytes: &[u8]| Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let mut bounds = Vec::new();
    for deflection in [0.1, 0.02] {
        let request = serde_json::to_vec(&(1u32, native.payload.as_slice(), 0.001f64, deflection, 0.3f64)).unwrap();
        let output = run(&request, hash(&request), &process, &Cancellation::default()).unwrap();
        assert!(output.stats.exited && output.stats.active_processes == 0);
        let candidate: Brep = serde_json::from_slice(&output.stdout).unwrap();
        candidate.validate(&limits).unwrap();
        assert!(!candidate.display().points.is_empty() && !candidate.display().triangles.is_empty());
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for point in &candidate.display().points {
            for axis in 0..3 { min[axis] = min[axis].min(point[axis]); max[axis] = max[axis].max(point[axis]); }
        }
        for axis in 0..3 { assert!(((max[axis] - min[axis]) - [4., 5., 6.][axis]).abs() < 1e-6); }
        bounds.push((min, max));
    }
    assert_eq!(bounds[0], bounds[1]);
    let request = serde_json::to_vec(&(1u32, native.payload.as_slice(), 0.001f64, 0.1f64, 0.3f64)).unwrap();
    assert!(run(&request, "0".repeat(64), &process, &Cancellation::default()).is_err());
    let bad = serde_json::to_vec(&(1u32, &[1u8, 2, 3][..], 0.001f64, 0.1f64, 0.3f64)).unwrap();
    assert!(run(&bad, hash(&bad), &process, &Cancellation::default()).is_err());
    let canceled = Cancellation::default(); canceled.cancel();
    assert!(run(&request, hash(&request), &process, &canceled).is_err());
    let small_output = ProcessLimits { stdout_bytes: 64, ..ProcessLimits::default() };
    assert!(run(&request, hash(&request), &small_output, &Cancellation::default()).is_err());
}

#[test]
#[ignore = "requires a separately built OM9_NATIVE_BREP_WORKER"]
fn native_restore_host_keeps_payload_identity_and_refuses_stale_results() {
    use om9_app::brep_restore_host::NativeBrepRestoreHost;
    use om9_core::{budget::Usage, geometry::NativeBrepSource, model::Document, transaction::{TransactionCoordinator, World}};
    let path = PathBuf::from(std::env::var_os("OM9_NATIVE_BREP_WORKER").expect("native BRep worker path"));
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&path).unwrap()).into();
    let host = NativeBrepRestoreHost::admit(&path, digest, ProcessLimits::default()).unwrap();
    assert!(NativeBrepRestoreHost::admit(&path, [0; 32], ProcessLimits::default()).is_err());
    if option_env!("OM9_NATIVE_BREP_WORKER_SHA256").is_some() {
        NativeBrepRestoreHost::admit_packaged(ProcessLimits::default()).unwrap();
    }
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let source = native.canonical_source();
    let mut document = Document::new("native host");
    let document_id = document.id;
    let entity = om9_core::model::Entity::new("native Box", om9_core::geometry::Geometry::NativeBrep(source.clone()), None);
    let object_id = entity.id;
    document.entities.insert(object_id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let held = Usage { bytes: source.storage_bytes().unwrap() + seed.storage_usage(&limits).unwrap().bytes, ..Usage::default() };
    let before = coordinator.snapshot();
    let restored = host.restore_display(&coordinator, &source, 0.001, 0.1, 0.3, held, &Cancellation::default()).unwrap();
    assert_eq!(restored.source().payload(), source.payload());
    assert_eq!(restored.source().crc(), source.crc());
    assert!(restored.display().matches(&source));
    assert!(!restored.display().display().vertices.is_empty());
    assert!(!restored.display().display().faces.is_empty());
    assert!(restored.reserved_bytes() >= restored.display().display_bytes().unwrap());
    assert_eq!(coordinator.snapshot(), before);
    let separate = NativeBrepSource { payload: std::sync::Arc::new(source.payload().to_vec()), crc: source.crc() };
    assert!(!restored.display().matches(&separate));
    let other = TransactionCoordinator::new(World::new(vec![Document::new("other")], &limits).unwrap(), limits).unwrap();
    assert!(restored.into_parts(&other).is_err());
    let corrupt = NativeBrepSource { payload: std::sync::Arc::new(vec![1, 2, 3]), crc: 0 };
    assert!(host.restore_display(&coordinator, &corrupt, 0.001, 0.1, 0.3, held, &Cancellation::default()).is_err());
    let limited = NativeBrepRestoreHost::admit(&path, digest, ProcessLimits { stdout_bytes: 64, ..ProcessLimits::default() }).unwrap();
    assert!(limited.restore_display(&coordinator, &source, 0.001, 0.1, 0.3, held, &Cancellation::default()).is_err());
    let canceled = Cancellation::default(); canceled.cancel();
    assert!(host.restore_display(&coordinator, &source, 0.001, 0.1, 0.3, held, &canceled).is_err());
    assert_eq!(coordinator.snapshot(), before);
    let restored = host.restore_display(&coordinator, &source, 0.001, 0.1, 0.3, held, &Cancellation::default()).unwrap();
    let (source, display, mut owner) = restored.into_parts(&coordinator).unwrap();
    let replacement_source = source.clone();
    let displays = om9_core::legacy_v4::LegacyDisplays { occt: Vec::new(), native: vec![(source, display)] };
    coordinator.adopt_legacy_displays(displays, &mut owner, held).unwrap();
    let scene = om9_scene::scene::Scene::capture_in_world(&coordinator, document_id, held).unwrap();
    assert!(scene.objects.iter().any(|object| object.root == object_id && object.has_pick_geometry()));
    assert_eq!(coordinator.snapshot(), before);
    let document = coordinator.world().documents[&document_id].as_ref().clone();
    let mut replacement = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    assert!(replacement.share_brep_representations_for_replacement(&coordinator, held, || Ok(())).is_err());
    replacement.adopt_external_captures(&coordinator).unwrap();
    assert!(matches!(replacement.share_brep_representations_for_replacement(&coordinator, held, || Err(om9_core::Error::Canceled)), Err(om9_core::Error::Canceled)));
    assert!(replacement.representation_store().lock().unwrap().get_native(&replacement_source).is_err());
    replacement.share_brep_representations_for_replacement(&coordinator, held, || Ok(())).unwrap();
    let store = replacement.representation_store();
    assert!(store.lock().unwrap().get_native(&replacement_source).unwrap().matches(&replacement_source));
    let scene = om9_scene::scene::Scene::capture_in_world(&replacement, document_id, held).unwrap();
    assert!(scene.objects.iter().any(|object| object.root == object_id && object.has_pick_geometry()));
    let mut foreign_document = coordinator.world().documents[&document_id].as_ref().clone();
    let other_source = NativeBrepSource { payload: std::sync::Arc::new(replacement_source.payload().to_vec()), crc: replacement_source.crc() };
    foreign_document.entities.get_mut(&object_id).unwrap().geometry = om9_core::geometry::Geometry::NativeBrep(other_source.clone());
    let mut foreign = TransactionCoordinator::new(World::new(vec![foreign_document], &limits).unwrap(), limits).unwrap();
    foreign.adopt_external_captures(&coordinator).unwrap();
    assert!(foreign.share_brep_representations_for_replacement(&coordinator, held, || Ok(())).is_err());
    assert!(foreign.representation_store().lock().unwrap().get_native(&other_source).is_err());
}

#[test]
#[ignore = "requires packaged BRep worker fingerprints and workers"]
fn valid_v7_container_with_invalid_native_payload_cannot_replace_export() {
    use om9_core::{geometry::{Geometry, NativeBrepSource}, model::{Document, Entity}};
    let limits = Limits::default();
    let mut source_document = Document::with_template_tolerance("invalid native source");
    let entity = Entity::new("bad native", Geometry::NativeBrep(NativeBrepSource {
        payload: std::sync::Arc::new(vec![1, 2, 3, 4]), crc: 0,
    }), None);
    source_document.entities.insert(entity.id, entity.clone());
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("unchanged target")).unwrap();
    let archive = om9_archive::canonical_brep::encode_mixed_document_candidate(&source_document, &[],
        &app.controller.coordinator().external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let decoded = om9_archive::canonical_brep::decode_mixed_document_candidate_with_admission(&archive, &limits, |_| Ok(())).unwrap();
    assert_eq!(decoded.document(), &source_document);
    let mut manager = om9_app::project_manager::ProjectManager::new("invalid native project");
    manager.capture_archive(0, vec![entity], &source_document, archive).unwrap();
    app.project_manager = manager;
    let path = std::env::temp_dir().join(format!("om9-invalid-native-export-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, b"existing destination").unwrap();
    let before = app.controller.coordinator().snapshot();
    let scene = app.controller.scene().unwrap().publication_id();
    let context = eframe::egui::Context::default();
    app.test_export_jobbag(0, &path).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline, "invalid native export timeout");
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.status.kind().is_failure());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.scene().unwrap().publication_id(), scene);
    assert_eq!(std::fs::read(&path).unwrap(), b"existing destination");
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires packaged workers and an offscreen GPU adapter"]
fn native_v7_jobbag_thumbnail_renders_without_source_document() {
    use om9_core::{geometry::Geometry, model::{Document, Entity}};
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let mut document = Document::with_template_tolerance("native preview source");
    let entity = Entity::new("native Box", Geometry::NativeBrep(native.canonical_source()), None);
    document.entities.insert(entity.id, entity.clone());
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("empty preview target")).unwrap();
    let archive = om9_archive::canonical_brep::encode_mixed_document_candidate(&document, &[],
        &app.controller.coordinator().external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    app.project_manager.capture_archive(0, vec![entity], &document, archive).unwrap();
    drop(document);
    drop(native);
    drop(seed);
    let before = app.controller.coordinator().snapshot();
    let context = eframe::egui::Context::default();
    app.test_service_thumbnails(&context, true);
    assert!(app.test_thumbnail_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_thumbnail_pending() {
        assert!(std::time::Instant::now() < deadline, "native v7 thumbnail timeout");
        app.test_service_thumbnails(&context, false);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(app.test_thumbnail_succeeded(), Some(true));
    let png = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().thumbnail.png.as_ref().unwrap();
    let image = image::load_from_memory(png).unwrap().to_rgba8();
    assert_eq!((image.width(), image.height()), (96, 96));
    assert!(image.pixels().any(|pixel| pixel != image.get_pixel(0, 0)));
    assert_eq!(app.controller.coordinator().snapshot(), before);
}

#[test]
#[ignore = "requires packaged BRep worker fingerprints and workers"]
fn native_v7_named_groups_capture_and_open() {
    use om9_core::{geometry::Geometry, model::{Document, Entity}};
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let payload = native.payload.clone();
    let mut document = Document::with_template_tolerance("all native groups");
    let entity = Entity::new("native Box", Geometry::NativeBrep(native.canonical_source()), None);
    document.entities.insert(entity.id, entity);
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("named target")).unwrap();
    let archive = om9_archive::canonical_brep::encode_mixed_document_candidate(&document, &[],
        &app.controller.coordinator().external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let path = std::env::temp_dir().join(format!("om9-native-named-source-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, archive).unwrap();
    app.controller = om9_app::session::Controller::open_archive_for_batch(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    let context = eframe::egui::Context::default();
    let names = ["Master", "Creation", "Parts", "Render", "Output"];
    for name in names {
        app.test_start_named_capture(name, None).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline, "{name} capture timeout");
            app.test_service_capture(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(app.project_manager.named_master_snapshot(name).is_some(), "{name}: {}", app.status_line());
    }
    for name in names {
        let id = app.project_manager.named_master_snapshot(name).unwrap().id;
        app.test_begin_named_open(name, id).unwrap();
        if app.controller.file_operation().unwrap().phase() == om9_app::file_lifecycle::FilePhase::Unsaved {
            let token = app.controller.file_operation().unwrap().token();
            app.controller.decide_unsaved(token, om9_app::file_lifecycle::UnsavedDecision::Discard).unwrap();
        }
        app.test_service_named_open(&context);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while app.test_capture_pending() {
            assert!(std::time::Instant::now() < deadline, "{name} Open timeout");
            app.test_service_named_open(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let restored = app.controller.document().entities.values().next().unwrap_or_else(|| panic!("{name}: {}", app.status_line()));
        let Geometry::NativeBrep(source) = &restored.geometry else { panic!("{name}: native source lost") };
        assert_eq!(source.payload(), payload.as_slice());
        assert!(app.controller.scene().unwrap().objects.iter().any(|object| object.root == restored.id && object.has_pick_geometry()));
    }
}

#[test]
#[ignore = "requires packaged BRep worker fingerprints and workers"]
fn native_v7_jobbag_sidebar_click_capture_and_insert() {
    use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
    use om9_core::{geometry::Geometry, model::{Document, Entity}};
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let payload = native.payload.clone();
    let mut document = Document::with_template_tolerance("native sidebar source");
    let entity = Entity::new("native Box", Geometry::NativeBrep(native.canonical_source()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("sidebar target")).unwrap();
    let archive = om9_archive::canonical_brep::encode_mixed_document_candidate(&document, &[],
        &app.controller.coordinator().external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let path = std::env::temp_dir().join(format!("om9-native-sidebar-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, archive).unwrap();
    app.controller = om9_app::session::Controller::open_archive_for_batch(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    let catalog_dir = std::env::temp_dir().join(format!("om9-native-sidebar-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&catalog_dir).unwrap();
    let catalog = catalog_dir.join("catalog.json");
    app.test_start_project_catalog_at(catalog.clone()).unwrap();
    let context = egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_project_catalog_load_pending() || app.test_project_catalog_save_pending() {
        assert!(std::time::Instant::now() < deadline, "initial catalog timeout");
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.controller.select_objects(&[id]).unwrap();
    let frame = |app: &mut om9_app::ui::MatrixApp, events| {
        let time = context.input(|input| input.time) + 1. / 60.;
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            time: Some(time), events, ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
    };
    let click = |app: &mut om9_app::ui::MatrixApp, pos: Pos2| {
        frame(app, vec![Event::PointerMoved(pos), Event::PointerButton {
            pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE,
        }]);
        frame(app, vec![Event::PointerButton {
            pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE,
        }]);
    };
    for _ in 0..4 { frame(&mut app, vec![]); }
    let capture = app.project_action_rects["capture-first-slot"].center();
    click(&mut app, capture);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_capture_pending() || app.test_project_catalog_save_pending() {
        assert!(std::time::Instant::now() < deadline, "sidebar capture timeout");
        app.test_service_project_background(&context);
        frame(&mut app, vec![]);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some(), "{}", app.status_line());
    app.controller.delete_objects(&[id]).unwrap();
    drop(app);
    use std::os::windows::process::CommandExt;
    let expected_hash = Sha256::digest(payload.as_slice());
    let expected_hash = expected_hash.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "native_v7_catalog_child_inserts_sidebar_bag", "--ignored", "--nocapture"])
        .env("OM9_NATIVE_SIDEBAR_CATALOG", &catalog)
        .env("OM9_NATIVE_SIDEBAR_PAYLOAD_SHA256", expected_hash)
        .env("OM9_PROJECT_CATALOG", catalog_dir.join("child-default.json"))
        .env("OM9_UI_LAYOUT", catalog_dir.join("child-layout.json"))
        .creation_flags(0x08000000)
        .output().unwrap();
    assert!(child.status.success(), "child failed: {} {}", String::from_utf8_lossy(&child.stdout), String::from_utf8_lossy(&child.stderr));
    assert!(String::from_utf8_lossy(&child.stdout).contains("1 passed"), "child fixture did not execute");
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("restarted target")).unwrap();
    app.test_start_project_catalog_at(catalog).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_project_catalog_load_pending() || app.test_project_catalog_save_pending() {
        assert!(std::time::Instant::now() < deadline, "reloaded catalog timeout");
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some(), "{}", app.status_line());
    for _ in 0..3 { frame(&mut app, vec![]); }
    let tile = app.jobbag_slot_rects[&0].center();
    click(&mut app, tile);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "sidebar insert timeout");
        app.test_service_capture(&context);
        frame(&mut app, vec![]);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let restored = app.controller.document().entities.values().next().expect(&app.status_line());
    assert_ne!(restored.id, id);
    let Geometry::NativeBrep(source) = &restored.geometry else { panic!("native source lost") };
    assert_eq!(source.payload(), payload.as_slice());
    assert!(app.controller.scene().unwrap().objects.iter().any(|object| object.root == restored.id && object.has_pick_geometry()));
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_project_catalog_save_pending() {
        assert!(std::time::Instant::now() < deadline, "final catalog timeout");
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    drop(app);
    let temp_root = std::env::temp_dir().canonicalize().unwrap();
    let resolved = catalog_dir.canonicalize().unwrap();
    assert_eq!(resolved.parent(), Some(temp_root.as_path()));
    std::fs::remove_dir_all(resolved).unwrap();
}

#[test]
#[ignore = "child fixture for the native sidebar catalog restart test"]
fn native_v7_catalog_child_inserts_sidebar_bag() {
    use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
    use om9_core::{geometry::Geometry, model::Document};
    let catalog = PathBuf::from(std::env::var_os("OM9_NATIVE_SIDEBAR_CATALOG").expect("catalog path"));
    let expected = std::env::var("OM9_NATIVE_SIDEBAR_PAYLOAD_SHA256").unwrap();
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("child target")).unwrap();
    app.test_start_project_catalog_at(catalog).unwrap();
    let context = egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_project_catalog_load_pending() || app.test_project_catalog_save_pending() {
        assert!(std::time::Instant::now() < deadline, "child catalog timeout");
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some(), "{}", app.status_line());
    let frame = |app: &mut om9_app::ui::MatrixApp, events| {
        let time = context.input(|input| input.time) + 1. / 60.;
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            time: Some(time), events, ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
    };
    for _ in 0..4 { frame(&mut app, vec![]); }
    let pos = app.jobbag_slot_rects[&0].center();
    frame(&mut app, vec![Event::PointerMoved(pos), Event::PointerButton {
        pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE,
    }]);
    frame(&mut app, vec![Event::PointerButton {
        pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE,
    }]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "child insert timeout");
        app.test_service_capture(&context);
        frame(&mut app, vec![]);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.document().entities.len(), 1, "{}", app.status_line());
    let restored = app.controller.document().entities.values().next().unwrap();
    let Geometry::NativeBrep(source) = &restored.geometry else { panic!("child native source lost") };
    let actual = Sha256::digest(source.payload()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert_eq!(actual, expected);
    assert!(app.controller.scene().unwrap().objects.iter().any(|object| object.root == restored.id && object.has_pick_geometry()));
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
#[ignore = "requires packaged BRep worker fingerprints and workers"]
fn native_sphere_v7_jobbag_restores_after_source_removal() {
    use om9_core::{geometry::Geometry, model::{Document, Entity}};
    let limits = Limits::default();
    let seed = om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let payload = native.payload.clone();
    let mut document = Document::with_template_tolerance("native Sphere");
    let entity = Entity::new("native Sphere", Geometry::NativeBrep(native.canonical_source()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("sphere target")).unwrap();
    let archive = om9_archive::canonical_brep::encode_mixed_document_candidate(&document, &[],
        &app.controller.coordinator().external_memory_account(), &limits, |_, _| Ok(())).unwrap();
    let path = std::env::temp_dir().join(format!("om9-native-sphere-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, archive).unwrap();
    app.controller = om9_app::session::Controller::open_archive_for_batch(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let context = eframe::egui::Context::default();
    app.test_capture_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native Sphere capture timeout");
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.active().slot(0).unwrap().snapshot.is_some(), "{}", app.status_line());
    app.controller.delete_objects(&[id]).unwrap();
    app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native Sphere insert timeout");
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let restored = app.controller.document().entities.values().next().expect(&app.status_line());
    let Geometry::NativeBrep(source) = &restored.geometry else { panic!("native Sphere source lost") };
    assert_eq!(source.payload(), payload.as_slice());
    assert!(app.controller.scene().unwrap().objects.iter().any(|object| object.root == restored.id && object.has_pick_geometry()));
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
#[ignore = "requires a packaged native BRep worker fingerprint and worker"]
fn legacy_open_rebuilds_missing_native_display_in_contained_worker() {
    use om9_core::{geometry::{Geometry, Mesh}, model::{Document, Entity}};
    assert!(option_env!("OM9_NATIVE_BREP_WORKER_SHA256").is_some());
    let limits = Limits::default();
    let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let mut native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let payload = native.payload.clone();
    native.display = std::sync::Arc::new(Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] });
    let mut document = Document::with_template_tolerance("native legacy Open");
    let entity = Entity::new("native Box", Geometry::NativeBrep(native.canonical_source()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut metadata = serde_json::to_value(&document).unwrap();
    metadata["entities"][id.0.to_string()]["geometry"]["NativeBrep"] = serde_json::to_value(native).unwrap();
    let metadata = serde_json::to_vec(&metadata).unwrap();
    let mut archive = vec![0u8; 88];
    archive[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    archive[8..12].copy_from_slice(&4u32.to_le_bytes());
    archive[12..16].copy_from_slice(&1u32.to_le_bytes());
    archive[16..20].copy_from_slice(&1u32.to_le_bytes());
    archive[40..48].copy_from_slice(&88u64.to_le_bytes());
    archive[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
    archive[56..88].copy_from_slice(&Sha256::digest(&metadata));
    archive.extend_from_slice(&metadata);
    let path = std::env::temp_dir().join(format!("om9-native-open-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, archive).unwrap();
    let opened = om9_app::session::Controller::open_archive_for_batch(&path, limits);
    std::fs::remove_file(path).unwrap();
    let controller = opened.unwrap();
    let document = controller.document();
    let Geometry::NativeBrep(source) = &document.entities[&id].geometry else { panic!("native source lost") };
    assert_eq!(source.payload(), payload.as_slice());
    assert!(controller.scene().unwrap().objects.iter().any(|object| object.root == id && object.has_pick_geometry()));

    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("target")).unwrap();
    app.controller = controller;
    app.controller.select_objects(&[id]).unwrap();
    let context = eframe::egui::Context::default();
    app.test_capture_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native capture timeout");
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let snapshot = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().expect(&app.status_line());
    let archive = snapshot.archive.as_ref().unwrap();
    assert_eq!(u32::from_le_bytes(archive[8..12].try_into().unwrap()), 7);
    let mut corrupt_archive = archive.to_vec();
    *corrupt_archive.last_mut().unwrap() ^= 1;
    let export_path = std::env::temp_dir().join(format!("om9-native-corrupt-export-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&export_path, b"existing destination").unwrap();
    let mut corrupt_manager = om9_app::project_manager::ProjectManager::new("corrupt native export");
    corrupt_manager.capture_archive(0, vec![app.controller.document().entities[&id].clone()], app.controller.document(), corrupt_archive).unwrap();
    let mut corrupt_app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("corrupt target")).unwrap();
    corrupt_app.project_manager = corrupt_manager;
    let corrupt_before = corrupt_app.controller.coordinator().snapshot();
    corrupt_app.test_export_jobbag(0, &export_path).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while corrupt_app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline, "corrupt native export timeout");
        corrupt_app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(corrupt_app.status.kind().is_failure());
    assert_eq!(corrupt_app.controller.coordinator().snapshot(), corrupt_before);
    assert_eq!(std::fs::read(&export_path).unwrap(), b"existing destination");
    drop(corrupt_app);
    app.test_export_jobbag(0, &export_path).unwrap();
    app.test_cancel_project_export();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_project_export_pending() {
        assert!(std::time::Instant::now() < deadline, "canceled native export timeout");
        app.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(std::fs::read(&export_path).unwrap(), b"existing destination");
    std::fs::remove_file(export_path).unwrap();
    app.status.acknowledge();
    let before = app.controller.coordinator().snapshot();
    let scene = app.controller.scene().unwrap().publication_id();
    let baseline_cpu = app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache);
    app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !app.test_brep_insert_result_ready() {
        assert!(std::time::Instant::now() < deadline, "native cancel preparation timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.test_cancel_capture();
    app.test_service_capture(&context);
    assert!(!app.test_capture_pending());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.scene().unwrap().publication_id(), scene);
    assert_eq!(app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache), baseline_cpu);
    app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !app.test_brep_insert_result_ready() {
        assert!(std::time::Instant::now() < deadline, "native stale preparation timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.project_manager.set_notes("changed while inserting").unwrap();
    app.test_service_capture(&context);
    assert!(!app.test_capture_pending());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.scene().unwrap().publication_id(), scene);
    assert_eq!(app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache), baseline_cpu);
    app.test_start_named_capture("Parts", None).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native named capture timeout");
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.project_manager.named_master_snapshot("Parts").is_some(), "{}", app.status_line());
    let catalog_dir = std::env::temp_dir().join(format!("om9-native-v7-catalog-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&catalog_dir).unwrap();
    let catalog = catalog_dir.join("catalog.json");
    om9_app::project_catalog::save(&catalog, &app.project_manager, None).unwrap();
    app.controller.delete_objects(&[id]).unwrap();
    app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native restore timeout");
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.document().entities.len(), 1, "{}", app.status_line());
    let inserted = app.controller.document().entities.values().next().unwrap();
    assert_ne!(inserted.id, id);
    let Geometry::NativeBrep(source) = &inserted.geometry else { panic!("native source lost after Job Bag") };
    assert_eq!(source.payload(), payload.as_slice());
    assert!(app.controller.scene().unwrap().objects.iter().any(|object| object.root == inserted.id && object.has_pick_geometry()));
    let cache_count = app.controller.test_brep_archive_cache_count();
    app.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "repeated native insert timeout");
        app.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(app.controller.document().entities.len(), 2, "{}", app.status_line());
    assert_eq!(app.controller.test_brep_archive_cache_count(), cache_count);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    let save_path = std::env::temp_dir().join(format!("om9-native-v7-save-{}.om9", uuid::Uuid::new_v4()));
    app.controller.begin_file_operation(om9_app::file_lifecycle::FileKind::SaveAs).unwrap();
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &save_path).unwrap(), om9_app::file_lifecycle::FileOutcome::Pending);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if let Some(result) = app.controller.poll_file_save_worker() {
            assert_eq!(result.unwrap(), om9_app::file_lifecycle::FileOutcome::Saved);
            break;
        }
        assert!(std::time::Instant::now() < deadline, "native Save timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(u32::from_le_bytes(std::fs::read(&save_path).unwrap()[8..12].try_into().unwrap()), 7);
    let reopened = om9_app::session::Controller::open_archive_for_batch(&save_path, limits).unwrap();
    std::fs::remove_file(save_path).unwrap();
    let restored = reopened.document().entities.values().next().unwrap();
    let Geometry::NativeBrep(source) = &restored.geometry else { panic!("native source lost after Open") };
    assert_eq!(source.payload(), payload.as_slice());
    assert!(reopened.scene().unwrap().objects.iter().any(|object| object.root == restored.id && object.has_pick_geometry()));
    let mut fresh = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("fresh catalog")).unwrap();
    fresh.test_start_project_catalog_at(catalog).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_project_catalog_load_pending() {
        assert!(std::time::Instant::now() < deadline, "native catalog reload timeout");
        fresh.test_service_project_background(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!fresh.status.kind().is_failure(), "{}", fresh.status_line());
    fresh.test_insert_filtered_jobbag(0, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native catalog insert timeout");
        fresh.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let restored = fresh.controller.document().entities.values().next().expect(&fresh.status_line());
    let Geometry::NativeBrep(source) = &restored.geometry else { panic!("native source lost after catalog reload") };
    assert_eq!(source.payload(), payload.as_slice());
    let named = fresh.project_manager.named_master_snapshot("Parts").unwrap().id;
    fresh.test_begin_named_open("Parts", named).unwrap();
    if fresh.controller.file_operation().unwrap().phase() == om9_app::file_lifecycle::FilePhase::Unsaved {
        let token = fresh.controller.file_operation().unwrap().token();
        fresh.controller.decide_unsaved(token, om9_app::file_lifecycle::UnsavedDecision::Discard).unwrap();
    }
    fresh.test_service_named_open(&context);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "native named open timeout");
        fresh.test_service_named_open(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let restored = fresh.controller.document().entities.values().next().expect(&fresh.status_line());
    let Geometry::NativeBrep(source) = &restored.geometry else { panic!("native source lost after named Open") };
    assert_eq!(source.payload(), payload.as_slice());
    assert!(fresh.controller.scene().unwrap().objects.iter().any(|object| object.root == restored.id && object.has_pick_geometry()));
    let native_id = restored.id;
    fresh.controller.submit("Box").unwrap();
    fresh.controller.submit("10,0,0").unwrap();
    fresh.controller.submit("14,5,0").unwrap();
    fresh.controller.submit("6").unwrap();
    let canonical_id = *fresh.controller.document().entities.iter().find(|(_, entity)| matches!(entity.geometry, Geometry::Brep(_))).unwrap().0;
    fresh.controller.select_objects(&[native_id, canonical_id]).unwrap();
    fresh.test_capture_filtered_jobbag(1, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "mixed capture timeout");
        fresh.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(fresh.project_manager.active().slot(1).unwrap().snapshot.is_some(), "{}", fresh.status_line());
    fresh.controller.delete_objects(&[native_id, canonical_id]).unwrap();
    fresh.test_insert_filtered_jobbag(1, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "mixed insert timeout");
        fresh.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(fresh.controller.document().entities.len(), 2, "{}", fresh.status_line());
    assert!(fresh.controller.document().entities.values().any(|entity| matches!(&entity.geometry, Geometry::NativeBrep(source) if source.payload() == payload.as_slice())));
    assert!(fresh.controller.document().entities.values().any(|entity| matches!(entity.geometry, Geometry::Brep(_))));
    assert_eq!(fresh.controller.scene().unwrap().objects.iter().filter(|object| object.has_pick_geometry()).count(), 2);
    fresh.controller.undo().unwrap();
    assert!(fresh.controller.document().entities.is_empty());
    fresh.controller.redo().unwrap();
    assert_eq!(fresh.controller.document().entities.len(), 2);
    let before_second: std::collections::BTreeSet<_> = fresh.controller.document().entities.keys().copied().collect();
    fresh.controller.submit("Box").unwrap();
    fresh.controller.submit("20,0,0").unwrap();
    fresh.controller.submit("23,2,0").unwrap();
    fresh.controller.submit("2").unwrap();
    let second_id = *fresh.controller.document().entities.keys().find(|id| !before_second.contains(id)).unwrap();
    fresh.controller.select_objects(&[second_id]).unwrap();
    fresh.test_capture_filtered_jobbag(2, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "second canonical capture timeout");
        fresh.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(fresh.project_manager.active().slot(2).unwrap().snapshot.is_some(), "{}", fresh.status_line());
    fresh.controller.delete_objects(&[second_id]).unwrap();
    fresh.test_insert_filtered_jobbag(2, om9_app::project_manager::BagFilter::All).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while fresh.test_capture_pending() {
        assert!(std::time::Instant::now() < deadline, "second canonical insert timeout");
        fresh.test_service_capture(&context);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(fresh.controller.document().entities.len(), 3, "{}", fresh.status_line());
    let mixed_path = std::env::temp_dir().join(format!("om9-mixed-v7-{}.om9", uuid::Uuid::new_v4()));
    fresh.controller.begin_file_operation(om9_app::file_lifecycle::FileKind::SaveAs).unwrap();
    let token = fresh.controller.file_operation().unwrap().token();
    fresh.controller.accept_file_path(token, &mixed_path).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if let Some(result) = fresh.controller.poll_file_save_worker() { result.unwrap(); break; }
        assert!(std::time::Instant::now() < deadline, "mixed Save timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let mut mixed_reopened = om9_app::session::Controller::open_archive_for_batch(&mixed_path, limits).unwrap();
    assert_eq!(mixed_reopened.document().entities.len(), 3);
    let resave_path = std::env::temp_dir().join(format!("om9-mixed-v7-resave-{}.om9", uuid::Uuid::new_v4()));
    mixed_reopened.begin_file_operation(om9_app::file_lifecycle::FileKind::SaveAs).unwrap();
    let token = mixed_reopened.file_operation().unwrap().token();
    mixed_reopened.accept_file_path(token, &resave_path).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if let Some(result) = mixed_reopened.poll_file_save_worker() { result.unwrap(); break; }
        assert!(std::time::Instant::now() < deadline, "mixed resave timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let resaved = om9_app::session::Controller::open_archive_for_batch(&resave_path, limits).unwrap();
    assert_eq!(resaved.document().entities.len(), 3);
    assert!(resaved.document().entities.values().any(|entity| matches!(&entity.geometry, Geometry::NativeBrep(source) if source.payload() == payload.as_slice())));
    assert_eq!(resaved.document().entities.values().filter(|entity| matches!(entity.geometry, Geometry::Brep(_))).count(), 2);
    std::fs::remove_file(mixed_path).unwrap();
    std::fs::remove_file(resave_path).unwrap();
    drop(fresh);
    let temp_root = std::fs::canonicalize(std::env::temp_dir()).unwrap();
    let actual_dir = std::fs::canonicalize(&catalog_dir).unwrap();
    assert_eq!(actual_dir.parent(), Some(temp_root.as_path()));
    std::fs::remove_dir_all(actual_dir).unwrap();
}
