// SPDX-License-Identifier: LGPL-2.1-or-later
//! End-to-end host-adapter test: the Rust CLI imports a 3DM through the admitted native reader,
//! saves an OM9 project, exports it through the admitted native writer and inspects the result.
//!
//! Ignored by default because it needs the built `om9-exchange-native.exe` and a `.3dm` fixture.
//! Run with:
//! `OM9_EXCHANGE_NATIVE=<abs exe> OM9_EXCHANGE_NATIVE_SHA256=<64 hex>
//!  OM9_THREE_DM_FIXTURE=<abs fixture.3dm> cargo test -p om9-app --test three_dm_native_roundtrip -- --ignored`
#![cfg(windows)]

use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
};
use std::{fs, path::PathBuf, process::Command};

fn required_env(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("{name} is required")))
}

fn native() -> Command {
    Command::new(required_env("OM9_EXCHANGE_NATIVE"))
}

#[test]
#[ignore = "requires an admitted om9-exchange-native.exe and a 3DM fixture"]
fn rust_cli_imports_and_exports_a_3dm_roundtrip() {
    let fixture = required_env("OM9_THREE_DM_FIXTURE");
    assert!(fixture.is_file(), "{}", fixture.display());
    let folder = std::env::temp_dir().join(format!("om9-3dm-host-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let imported = folder.join("imported.om9");
    let exported = folder.join("exported.3dm");
    let binary = env!("CARGO_BIN_EXE_om9");

    let imported_output = Command::new(binary)
        .arg("--import-3dm")
        .arg(&fixture)
        .arg(&imported)
        .output()
        .unwrap();
    assert!(
        imported_output.status.success(),
        "import failed: {}",
        String::from_utf8_lossy(&imported_output.stderr)
    );
    assert!(imported.is_file());

    let exported_output = Command::new(binary)
        .arg("--export-3dm")
        .arg(&imported)
        .arg(&exported)
        .output()
        .unwrap();
    assert!(
        exported_output.status.success(),
        "export failed: {}",
        String::from_utf8_lossy(&exported_output.stderr)
    );
    assert!(exported.is_file());

    let inspected = native().arg("--inspect").arg(&exported).output().unwrap();
    assert!(
        inspected.status.success(),
        "native inspect failed: {}",
        String::from_utf8_lossy(&inspected.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert!(
        !report["objects"].as_array().unwrap().is_empty(),
        "the exported archive must contain objects"
    );
    assert_eq!(report["diagnostics"].as_array().map(Vec::len), Some(0));

    fs::remove_file(imported).unwrap();
    fs::remove_file(exported).unwrap();
    fs::remove_dir(folder).unwrap();
}

#[test]
#[ignore = "requires an admitted om9-exchange-native.exe and a 3DM fixture"]
fn controller_host_adapter_imports_and_exports_without_the_cli() {
    let fixture = required_env("OM9_THREE_DM_FIXTURE");
    let folder = std::env::temp_dir().join(format!("om9-3dm-controller-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let exported = folder.join("controller-export.3dm");
    let mut document = Document::with_template_tolerance("controller 3DM");
    let existing = Entity::new("existing point", Geometry::Point([1.0, 2.0, 3.0]), None);
    let existing_id = existing.id;
    document.entities.insert(existing_id, existing);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    let diagnostics = controller.import_3dm(&fixture).unwrap();
    assert!(
        controller.document().entities.len() > 1,
        "import must add document objects: {diagnostics:?}"
    );
    assert!(
        controller.document().entities.contains_key(&existing_id),
        "import must not replace the existing document"
    );
    let source_before_export = controller.coordinator().snapshot();
    controller.export_3dm(&exported).unwrap();
    assert_eq!(controller.coordinator().snapshot(), source_before_export,
        "native export projection must not mutate the source or history");
    assert!(exported.is_file());

    let inspected = native().arg("--inspect").arg(&exported).output().unwrap();
    assert!(inspected.status.success());
    let report: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert!(!report["objects"].as_array().unwrap().is_empty());
    fs::remove_file(exported).unwrap();
    fs::remove_dir(folder).unwrap();
}

#[test]
#[ignore = "requires an admitted om9-exchange-native.exe"]
fn failed_import_keeps_document_and_undo() {
    let folder = std::env::temp_dir().join(format!("om9-bad-3dm-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let bad = folder.join("not-a-3dm.3dm");
    fs::write(&bad, b"this is not a 3dm archive").unwrap();
    let mut document = Document::new("rollback");
    let existing = Entity::new("existing", Geometry::Point([1.0, 2.0, 3.0]), None);
    let existing_id = existing.id;
    document.entities.insert(existing_id, existing);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    let before_undo = controller.coordinator().undo_len();
    let kernel_before = controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::Kernel);
    let error = controller.import_3dm(&bad).unwrap_err();
    assert!(!error.to_string().is_empty());
    assert_eq!(controller.document().entities.len(), 1);
    assert!(controller.document().entities.contains_key(&existing_id));
    assert_eq!(controller.coordinator().undo_len(), before_undo);
    assert_eq!(controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::Kernel), kernel_before);
    fs::remove_file(bad).unwrap();
    fs::remove_dir(folder).unwrap();
}

#[test]
#[ignore = "requires an admitted native 3DM executable"]
fn memory_refused_import_preserves_scene_selection_and_redo_then_retries() {
    use om9_core::budget::memory::MemoryClass;
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let fixture = Cleanup(std::env::temp_dir().join(format!("om9-3dm-pressure-{}.3dm", uuid::Uuid::new_v4())));
    let mut source = Document::with_template_tolerance("3DM pressure source");
    let point = Entity::new("exported point", Geometry::Point([4., 5., 6.]), None);
    source.entities.insert(point.id, point);
    Controller::new(source, Limits::default()).unwrap().export_3dm(&fixture.0).unwrap();
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("3DM pressure");
    let existing = Entity::new("existing", Geometry::Point([1., 2., 3.]), None);
    let id = existing.id;
    document.entities.insert(id, existing);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.numeric_move([1., 0., 0.]).unwrap();
    controller.undo().unwrap();
    let held = controller.scene_cache_usage().unwrap();
    let (mut low, mut high) = (0u64, limits.max_bytes);
    while low < high {
        let mid = low + (high - low + 1) / 2;
        let trial = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, mid, None, held);
        if trial.is_ok() { low = mid; } else { high = mid - 1; }
    }
    let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, low, None, held).unwrap();
    let before = controller.coordinator().snapshot();
    let scene_epoch = controller.scene().unwrap().epoch;
    assert!(controller.import_3dm(&fixture.0).is_err());
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.scene().unwrap().epoch, scene_epoch);
    assert_eq!(controller.selected_ordered(), vec![id]);
    assert_eq!(controller.coordinator().redo_len(), 1);
    drop(reservation);
    controller.import_3dm(&fixture.0).unwrap();
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    assert!(controller.document().entities.len() > 1);
    assert!(controller.selected_ordered().is_empty());
    assert_eq!(controller.coordinator().redo_len(), 0);
    assert_eq!(controller.scene().unwrap().epoch, controller.coordinator().epoch());
}

#[test]
#[ignore = "requires an admitted native 3DM executable"]
fn small_ram_limit_reports_the_native_process_reservation() {
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let fixture = Cleanup(std::env::temp_dir().join(format!("om9-3dm-small-limit-{}.3dm", uuid::Uuid::new_v4())));
    let mut source = Document::with_template_tolerance("small RAM source");
    let point = Entity::new("point", Geometry::Point([4., 5., 6.]), None);
    source.entities.insert(point.id, point);
    Controller::new(source, Limits::default()).unwrap().export_3dm(&fixture.0).unwrap();
    let limits = Limits { max_bytes: 32 * 1024 * 1024, ..Limits::default() };
    let mut target = Controller::new(Document::with_template_tolerance("small RAM target"), limits).unwrap();
    let before = target.coordinator().snapshot();
    let error = target.import_3dm(&fixture.0).unwrap_err();
    assert!(matches!(error, om9_core::Error::Budget("3DM native process reservation")), "{error}");
    assert_eq!(target.coordinator().snapshot(), before);
}

#[test]
#[ignore = "requires an admitted native 3DM executable"]
fn cli_import_obeys_ram_override_and_preserves_destination_on_refusal() {
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let source = Cleanup(std::env::temp_dir().join(format!("om9-cli-ram-source-{}.3dm", uuid::Uuid::new_v4())));
    let target = Cleanup(std::env::temp_dir().join(format!("om9-cli-ram-target-{}.om9", uuid::Uuid::new_v4())));
    let mut document = Document::with_template_tolerance("CLI RAM source");
    let point = Entity::new("point", Geometry::Point([4., 5., 6.]), None);
    document.entities.insert(point.id, point);
    Controller::new(document, Limits::default()).unwrap().export_3dm(&source.0).unwrap();
    fs::write(&target.0, b"existing destination").unwrap();
    let run = |bytes: u64| Command::new(env!("CARGO_BIN_EXE_om9"))
        .arg("--import-3dm").arg(&source.0).arg(&target.0)
        .env("OM9_MAX_BYTES", bytes.to_string()).output().unwrap();
    let refused = run(32 * 1024 * 1024);
    assert!(!refused.status.success());
    let message = format!("{}{}", String::from_utf8_lossy(&refused.stdout), String::from_utf8_lossy(&refused.stderr));
    assert!(message.contains("3DM native process reservation"), "unexpected failure: {message}");
    assert_eq!(fs::read(&target.0).unwrap(), b"existing destination");
    let accepted = run(512 * 1024 * 1024);
    assert!(accepted.status.success(), "{}", String::from_utf8_lossy(&accepted.stderr));
    let imported = om9_archive::Archive::open(&target.0, &Limits::default()).unwrap();
    assert!(imported.entities.values().any(|entity| matches!(entity.geometry, Geometry::Point([4., 5., 6.]))));
}

#[test]
#[ignore = "requires an admitted native 3DM executable"]
fn cli_export_obeys_ram_override_and_preserves_files_on_refusal() {
    use om9_core::{resource::{Resource, ResourceKind}, transaction::Cancellation};
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let source = Cleanup(std::env::temp_dir().join(format!("om9-cli-export-source-{}.fixture", uuid::Uuid::new_v4())));
    let target = Cleanup(std::env::temp_dir().join(format!("om9-cli-export-target-{}.3dm", uuid::Uuid::new_v4())));
    let mut document = Document::with_template_tolerance("original source name");
    let point = Entity::new("point", Geometry::Point([4., 5., 6.]), None);
    document.entities.insert(point.id, point);
    let resource = Resource::from_bytes("fixture", ResourceKind::OpaqueBinary, &vec![0x5a; 2 * 1024 * 1024], &Limits::default()).unwrap();
    document.resources.insert(resource.id, resource);
    om9_archive::Archive::save(&source.0, &document, &Limits::default(), &Cancellation::default()).unwrap();
    let before = fs::read(&source.0).unwrap();
    fs::write(&target.0, b"existing destination").unwrap();
    let run = |bytes: u64| Command::new(env!("CARGO_BIN_EXE_om9"))
        .arg("--export-3dm").arg(&source.0).arg(&target.0)
        .env("OM9_MAX_BYTES", bytes.to_string()).output().unwrap();
    let refused = run(1024 * 1024);
    assert!(!refused.status.success());
    assert_eq!(fs::read(&target.0).unwrap(), b"existing destination");
    assert_eq!(fs::read(&source.0).unwrap(), before);
    let accepted = run(512 * 1024 * 1024);
    assert!(accepted.status.success(), "{}", String::from_utf8_lossy(&accepted.stderr));
    assert_eq!(fs::read(&source.0).unwrap(), before);
    let inspected = native().arg("--inspect").arg(&target.0).output().unwrap();
    assert!(inspected.status.success());
    let report: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(report["objects"].as_array().unwrap().len(), 1);
}

#[test]
#[ignore = "requires an admitted native 3DM executable"]
fn export_native_allowance_competes_with_other_shared_reservations() {
    use om9_core::budget::memory::MemoryClass;
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let target = Cleanup(std::env::temp_dir().join(format!("om9-3dm-export-pressure-{}.3dm", uuid::Uuid::new_v4())));
    fs::write(&target.0, b"existing destination").unwrap();
    let mut document = Document::with_template_tolerance("export pressure");
    let point = Entity::new("point", Geometry::Point([4., 5., 6.]), None);
    document.entities.insert(point.id, point);
    let controller = Controller::new(document, Limits::default()).unwrap();
    let before = controller.coordinator().snapshot();
    let competing = controller.coordinator().reserve_external_memory(MemoryClass::Export, 100 * 1024 * 1024, None).unwrap();
    assert!(controller.export_3dm(&target.0).is_err());
    assert_eq!(fs::read(&target.0).unwrap(), b"existing destination");
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    drop(competing);
    controller.export_3dm(&target.0).unwrap();
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let inspected = native().arg("--inspect").arg(&target.0).output().unwrap();
    assert!(inspected.status.success());
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires admitted native 3DM executable and OCCT runtime"]
fn editable_sphere_export_preserves_source_with_projection_reservation() {
    use om9_core::budget::memory::MemoryClass;
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let target = Cleanup(std::env::temp_dir().join(format!("om9-export-sphere-{}.3dm", uuid::Uuid::new_v4())));
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("sphere export");
    let brep = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let entity = Entity::new("sphere", Geometry::Brep(brep), None);
    document.entities.insert(entity.id, entity);
    let controller = Controller::new(document, limits).unwrap();
    let before = controller.coordinator().snapshot();
    controller.export_3dm(&target.0).unwrap();
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), 0);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let output = native().arg("--inspect").arg(&target.0).output().unwrap();
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["objects"].as_array().unwrap().len(), 1);
    assert_eq!(report["objects"][0]["native_kind"], "ON_Brep");
}

#[test]
#[ignore = "requires an admitted native 3DM executable and a 3DM fixture"]
fn prepared_import_candidate_remains_counted_after_native_guard_handoff() {
    use om9_core::{budget::memory::MemoryClass, transaction::{Cancellation, TransactionCoordinator, World}};
    use om9_exchange::{BrepMode, ImportMode, ImportOptions};
    let limits = Limits::default();
    let document = Document::with_template_tolerance("handoff");
    let id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
    peer.adopt_external_captures(&coordinator).unwrap();
    let before = peer.retained_usage().unwrap().bytes;
    let host = om9_app::three_dm_host::ThreeDmHost::configured().unwrap();
    let options = ImportOptions { mode: ImportMode::Geometry, brep: BrepMode::EditableBestEffort, ..Default::default() };
    let (prepared, _receipt) = host.prepare_import_with_metadata(&coordinator, id, &required_env("OM9_THREE_DM_FIXTURE"), &options, None, &Cancellation::default()).unwrap();
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0, "native guard has been handed off");
    let candidate = std::sync::Arc::downgrade(&prepared.candidate_world().documents[&id]);
    let during = peer.retained_usage().unwrap().bytes;
    assert!(during > before, "peer must account for the staged imported document");
    drop(prepared);
    assert!(candidate.upgrade().is_none());
    assert!(peer.retained_usage().unwrap().bytes < during);
}

#[test]
#[ignore = "requires an admitted native 3DM executable and a 3DM fixture"]
fn prepared_native_import_holds_and_releases_shared_reservation() {
    use om9_core::{budget::memory::MemoryClass, transaction::{Cancellation, TransactionCoordinator, World}};
    use om9_exchange::{ImportCapture, ImportOptions, NativeExchange};
    use om9_native_owner::{NativeProgram, ProcessLimits};
    let native = required_env("OM9_EXCHANGE_NATIVE");
    let hex = std::env::var("OM9_EXCHANGE_NATIVE_SHA256").unwrap();
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap();
    }
    let limits = Limits::default();
    let document = Document::with_template_tolerance("prepared import guard");
    let id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
    peer.adopt_external_captures(&coordinator).unwrap();
    let exchange = NativeExchange::new(NativeProgram::admit(&native, digest).unwrap(), ProcessLimits::default()).unwrap();
    let capture = ImportCapture::target(&coordinator, id).unwrap();
    let prepared = exchange.prepare_import(&coordinator, capture, &required_env("OM9_THREE_DM_FIXTURE"), &std::env::temp_dir(), &ImportOptions::default(), &Cancellation::default()).unwrap();
    let reserved = coordinator.external_reserved_bytes(MemoryClass::Kernel);
    assert!(reserved > 0 && reserved < ProcessLimits::default().reservation_bytes().unwrap(),
        "the finished child reservation should shrink to the retained bundle");
    assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), reserved);
    drop(prepared);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), 0);
}

#[test]
#[ignore = "requires an admitted native 3DM executable and a B-rep fixture"]
fn conversion_reservation_is_visible_to_competing_account_and_released_on_error() {
    use om9_core::{budget::memory::MemoryClass, transaction::{Cancellation, TransactionCoordinator, World}};
    use om9_exchange::{BrepBridge, BrepMode, ImportCapture, ImportMode, ImportOptions, NativeExchange};
    use om9_native_owner::{NativeProgram, ProcessLimits};
    use std::{sync::mpsc, time::Duration};
    struct PausedBridge { ready: mpsc::Sender<()>, proceed: mpsc::Receiver<()> }
    impl BrepBridge for PausedBridge {
        fn import(&self, _: &[u8], _: f64, _: f64, _: &Limits) -> om9_core::Result<om9_core::brep::Brep> {
            self.ready.send(()).unwrap();
            self.proceed.recv_timeout(Duration::from_secs(30)).unwrap();
            Err(om9_core::Error::Canceled)
        }
    }
    let native = required_env("OM9_EXCHANGE_NATIVE");
    let fixture = required_env("OM9_THREE_DM_FIXTURE");
    let hex = std::env::var("OM9_EXCHANGE_NATIVE_SHA256").unwrap();
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap();
    }
    let limits = Limits::default();
    let document = Document::with_template_tolerance("concurrent import");
    let id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
    peer.adopt_external_captures(&coordinator).unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (proceed_tx, proceed_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let exchange = NativeExchange::new(NativeProgram::admit(&native, digest).unwrap(), ProcessLimits::default()).unwrap();
        let capture = ImportCapture::target(&coordinator, id).unwrap();
        let options = ImportOptions { mode: ImportMode::Geometry, brep: BrepMode::EditableStrict, ..Default::default() };
        let bridge = PausedBridge { ready: ready_tx, proceed: proceed_rx };
        let result = exchange.prepare_import_with_brep(&coordinator, capture, &fixture, &std::env::temp_dir(), &options, Some(&bridge), &Cancellation::default());
        if let Err(error) = &result { eprintln!("conversion worker result: {error}"); }
        result.is_err()
    });
    ready_rx.recv_timeout(Duration::from_secs(30)).expect("conversion must reach bridge");
    let reserved = peer.external_reserved_bytes(MemoryClass::Kernel);
    let source_bytes = fs::metadata(required_env("OM9_THREE_DM_FIXTURE")).unwrap().len();
    let minimum = ProcessLimits::default().reservation_bytes().unwrap() + 3 * source_bytes + 16384;
    let used = peer.retained_usage().unwrap().bytes;
    let over_headroom = limits.max_bytes.checked_sub(used).unwrap() + 1;
    let competing = peer.reserve_external_memory(MemoryClass::CpuCache, over_headroom, None);
    proceed_tx.send(()).unwrap();
    assert!(worker.join().unwrap());
    assert!(reserved > minimum, "inventory and conversion estimates must be reserved in addition to the native process and source");
    assert!(matches!(competing, Err(om9_core::Error::Budget(_))), "a competing request cannot spend reserved headroom");
    assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), 0);
}

#[test]
#[ignore = "requires an admitted native 3DM executable and a 3DM fixture"]
fn imported_3dm_document_json_preserves_canonical_state() {
    let fixture = required_env("OM9_THREE_DM_FIXTURE");
    let mut controller = Controller::new(Document::with_template_tolerance("roundtrip"), Limits::default()).unwrap();
    controller.import_3dm(&fixture).unwrap();
    let source = controller.document();
    let data = serde_json::to_vec(source).unwrap();
    let decoded: Document = serde_json::from_slice(&data).unwrap();
    for (id, entity) in &source.entities {
        let restored = &decoded.entities[id];
        assert_eq!(om9_core::retained::fingerprint(entity).unwrap(), om9_core::retained::fingerprint(restored).unwrap(),
            "entity {id}: geometry_equal={} placement_equal={} attributes_equal={} label_equal={}",
            entity.geometry == restored.geometry, entity.placement == restored.placement,
            entity.attributes == restored.attributes, entity.label == restored.label);
    }
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let path = Cleanup(std::env::temp_dir().join(format!("om9-brep-fingerprint-{}.om9", uuid::Uuid::new_v4())));
    om9_archive::Archive::save(&path.0, source, &Limits::default(), &om9_core::transaction::Cancellation::default()).unwrap();
    let reopened = om9_archive::Archive::open(&path.0, &Limits::default()).unwrap();
    assert_eq!(reopened, *source);
    om9_archive::Archive::open_restored(&path.0, &Limits::default(), &Default::default()).unwrap();
}
