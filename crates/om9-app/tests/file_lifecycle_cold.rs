// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
use om9_app::{
    file_lifecycle::{FileKind, FileOutcome, UnsavedDecision},
    layer_command::{LayerChoice, LayerProperty},
    session::Controller,
};
use om9_core::{budget::Limits, layers::PresetSlot, model::Document, transaction::Cancellation};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
struct Files(PathBuf);
impl Drop for Files {
    fn drop(&mut self) {
        for entry in fs::read_dir(&self.0).unwrap() {
            let path = entry.unwrap().path();
            assert_eq!(path.parent(), Some(self.0.as_path()));
            assert!(path.is_file());
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir(&self.0).unwrap();
    }
}
fn inspect(path: &Path, case: &str) -> serde_json::Value {
    let binary = Path::new(env!("CARGO_BIN_EXE_om9"));
    let mut file = fs::File::open(binary).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let program = NativeProgram::admit(binary, hash.finalize().into()).unwrap();
    let output = program
        .run(
            &["--inspect".into(), path.as_os_str().to_owned()],
            &ProcessLimits {
                stdout_bytes: 64 * 1024,
                deadline: Duration::from_secs(10),
                ..ProcessLimits::default()
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert!(output.stats.exited);
    assert_eq!(output.stats.active_processes, 0);
    assert!(output.stderr.is_empty());
    println!(
        "OM9_FILE_RECEIPT {}",
        serde_json::json!({"case":case,"pid":output.stats.pid,"exited":output.stats.exited,
        "active_processes":output.stats.active_processes,"peak_committed_bytes":output.stats.peak_committed_bytes,
        "binary_sha256":program.fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>(),
        "archive_sha256":format!("{:x}",Sha256::digest(fs::read(path).unwrap())),"full_file_acceptance":false})
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn controller_saved_documents_survive_new_open_and_independent_cold_inspection() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let folder = root
        .join("build/standalone-validation")
        .join(format!("cold-file-lifecycle-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&folder).unwrap();
    let files = Files(folder);
    let first = files.0.join("first.om9");
    let second = files.0.join("stage-two.om9");
    let mut c = Controller::new(Document::new("cold full file state"), Limits::default()).unwrap();
    let slot = PresetSlot::new(21).unwrap();
    c.set_current_layer(LayerChoice::Slot(slot)).unwrap();
    c.submit("Point").unwrap();
    c.submit("2,3,4").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let source = c.document().stamp(id).unwrap();
    c.set_objects_hidden(vec![source], true).unwrap();
    c.set_layer_property(slot, LayerProperty::Locked(true))
        .unwrap();
    c.set_layer_property(slot, LayerProperty::Visible(false))
        .unwrap();
    let docid = c.document().id;
    let layers = c.document().layers.clone();
    let attrs = c.document().entities[&id].attributes.clone();
    c.begin_file_operation(FileKind::Save).unwrap();
    let token = c.file_operation().unwrap().token();
    assert_eq!(
        c.accept_file_path(token, &first).unwrap(),
        FileOutcome::Saved
    );
    let old_bytes = fs::read(&first).unwrap();
    assert!(!c.is_dirty().unwrap());
    let report = inspect(&first, "cold_file_first_save");
    assert_eq!(report["layers"], serde_json::to_value(&layers).unwrap());
    assert_eq!(
        report["objects"][0]["attributes"],
        serde_json::to_value(&attrs).unwrap()
    );
    assert_eq!(
        report["objects"][0]["geometry"],
        serde_json::json!({"Point":[2.,3.,4.]})
    );
    assert_eq!(
        c.begin_file_operation(FileKind::New).unwrap(),
        FileOutcome::Replaced
    );
    c.submit("Point").unwrap();
    c.submit("9,8,7").unwrap();
    c.begin_file_operation(FileKind::Open).unwrap();
    let token = c.file_operation().unwrap().token();
    c.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    let token = c.file_operation().unwrap().token();
    assert_eq!(
        c.accept_file_path(token, &first).unwrap(),
        FileOutcome::Replaced
    );
    assert_eq!(c.document().id, docid);
    assert_eq!(c.document().layers, layers);
    assert_eq!(c.document().entities[&id].attributes, attrs);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.begin_file_operation(FileKind::SaveAs).unwrap();
    let token = c.file_operation().unwrap().token();
    assert_eq!(
        c.accept_file_path(token, &second).unwrap(),
        FileOutcome::Saved
    );
    assert_eq!(fs::read(&first).unwrap(), old_bytes);
    assert_eq!(c.active_file_path(), Some(second.as_path()));
    let second_report = inspect(&second, "cold_file_save_as_after_open");
    assert_eq!(second_report["layers"], report["layers"]);
    assert_eq!(second_report["objects"], report["objects"]);
}
