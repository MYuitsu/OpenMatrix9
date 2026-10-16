// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
use om9_core::{
    Error,
    budget::{Limits, Usage},
    model::Document,
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{ExportCapture, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{io::Read, path::PathBuf};
fn exchange() -> NativeExchange {
    let path = std::env::current_exe().unwrap();
    let mut file = std::fs::File::open(&path).unwrap();
    let mut hash = Sha256::new();
    let mut buf = [0; 8192];
    loop {
        let n = file.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    NativeExchange::new(
        NativeProgram::admit(&path, hash.finalize().into()).unwrap(),
        ProcessLimits::default(),
    )
    .unwrap()
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let parent =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../build/standalone-validation");
        std::fs::create_dir_all(&parent).unwrap();
        let path = parent.join(format!("retained-stage-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.0);
    }
}
#[test]
fn staged_request_holds_packet_against_write_delete_and_substituted_cancellation() {
    let temp = Temp::new();
    let d = Document::new("stage");
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let original = Cancellation::default();
    let inputs = exchange()
        .prepare_merge_inputs(&c, &capture, None, &temp.0, &original, Usage::default())
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let stage_cancel = Cancellation::default();
    let stage = wire
        .stage_retained_request(&c, &temp.0, &stage_cancel, Usage::default())
        .unwrap();
    let request = stage.request_path().to_owned();
    let p: serde_json::Value = serde_json::from_slice(&std::fs::read(&request).unwrap()).unwrap();
    assert_eq!(p["writer_policy"], "om9-retained-current-v1");
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&request)
            .is_err()
    );
    assert!(std::fs::remove_file(&request).is_err());
    stage.revalidate(&c, &Cancellation::default()).unwrap();
    let fresh = Cancellation::default();
    let joined = stage.native_cancellation(&fresh).unwrap();
    assert!(!joined.is_canceled());
    stage_cancel.cancel();
    assert!(joined.is_canceled());
    assert!(matches!(
        stage.revalidate(&c, &Cancellation::default()),
        Err(Error::Canceled)
    ));
    drop(stage);
    assert!(!request.exists());
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}
#[test]
fn refused_stage_budget_and_cancel_create_no_private_directory() {
    let temp = Temp::new();
    let d = Document::new("stage");
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let inputs = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &temp.0,
            &Cancellation::default(),
            Usage::default(),
        )
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    assert!(matches!(
        wire.stage_retained_request(
            &c,
            &temp.0,
            &Cancellation::default(),
            Usage {
                bytes: c.limits().max_bytes,
                ..Usage::default()
            }
        ),
        Err(Error::Budget(_))
    ));
    let canceled = Cancellation::default();
    canceled.cancel();
    assert!(matches!(
        wire.stage_retained_request(&c, &temp.0, &canceled, Usage::default()),
        Err(Error::Canceled)
    ));
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}
