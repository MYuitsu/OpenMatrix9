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
    let mut document = Document::new("controller 3DM");
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
    controller.export_3dm(&exported).unwrap();
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
    let error = controller.import_3dm(&bad).unwrap_err();
    assert!(!error.to_string().is_empty());
    assert_eq!(controller.document().entities.len(), 1);
    assert!(controller.document().entities.contains_key(&existing_id));
    assert_eq!(controller.coordinator().undo_len(), before_undo);
    fs::remove_file(bad).unwrap();
    fs::remove_dir(folder).unwrap();
}
