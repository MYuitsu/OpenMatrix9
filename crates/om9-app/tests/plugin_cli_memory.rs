// SPDX-License-Identifier: LGPL-2.1-or-later
//! Batch plugin Open/Save uses the same admitted path as the document UI.
use std::{fs, path::PathBuf, process::Command};
use om9_archive::Archive;
use om9_core::{budget::Limits, model::Document, resource::{Resource, ResourceKind}, transaction::Cancellation};

struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn plugin_cli_refuses_low_budget_without_changing_files_and_succeeds_after_retry() {
    let package = std::env::var_os("OM9_TEST_PLUGIN_PACKAGE").expect("built plugin package");
    let folder = TempDir(std::env::temp_dir().join(format!("om9-plugin-cli-{}", uuid::Uuid::new_v4())));
    fs::create_dir(&folder.0).unwrap();
    let input = folder.0.join("input.data");
    let output = folder.0.join("output.data");
    let arguments = folder.0.join("args.json");
    fs::write(&arguments, br#"{"radius_mm":5}"#).unwrap();
    let exe = env!("CARGO_BIN_EXE_om9");
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("plugin memory fixture");
    let resource = Resource::from_bytes("payload", ResourceKind::OpaqueBinary, &vec![0x5a; 2 * 1024 * 1024], &limits).unwrap();
    document.resources.insert(resource.id, resource);
    Archive::save(&input, &document, &limits, &Cancellation::default()).unwrap();
    let original = fs::read(&input).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    let run = |limit: &str| Command::new(exe).arg("--plugin-run-on").arg(&package)
        .arg("CreateProfile").arg(&arguments).arg(&input).arg(&output)
        .env("OM9_MAX_BYTES", limit).output().unwrap();
    assert!(!run("1024").status.success());
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    let result = run("536870912");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(fs::read(&input).unwrap(), original);
    let restored = Archive::open_restored(&output, &Limits::default(), &Default::default()).unwrap();
    assert!(restored.document.entities.values().any(|entity| entity.label == "SDK profile"));
}
