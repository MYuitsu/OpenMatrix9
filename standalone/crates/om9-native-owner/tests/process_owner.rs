// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
use om9_core::transaction::Cancellation;
use om9_native_owner::{FailureKind, NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    path::Path,
    thread,
    time::{Duration, Instant},
};
fn program() -> NativeProgram {
    let path = Path::new(env!("CARGO_BIN_EXE_om9-process-fixture"));
    let digest: [u8; 32] = Sha256::digest(std::fs::read(path).unwrap()).into();
    NativeProgram::admit(path, digest).unwrap()
}
fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
#[test]
fn admitted_program_returns_owned_bounded_bytes_and_actual_job_peak_with_no_live_child() {
    let result = program()
        .run(
            &args(&["echo", "xin chào", "quote\"slash\\"]),
            &ProcessLimits::default(),
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(
        std::str::from_utf8(&result.stdout).unwrap(),
        "xin chào|quote\"slash\\"
    );
    assert_eq!(result.stderr, b"fixture diagnostics");
    assert!(result.stats.pid > 0 && result.stats.peak_committed_bytes > 0);
    assert_eq!(result.stats.active_processes, 0);
    assert!(result.stats.exited);
}
#[test]
fn wrong_binary_fingerprint_and_invalid_limits_reject_before_process_creation() {
    assert!(
        NativeProgram::admit(
            Path::new(env!("CARGO_BIN_EXE_om9-process-fixture")),
            [0; 32]
        )
        .is_err()
    );
    let limits = ProcessLimits {
        native_committed_bytes: 0,
        ..ProcessLimits::default()
    };
    let failure = program()
        .run(&args(&["echo"]), &limits, &Cancellation::default())
        .unwrap_err();
    assert!(matches!(failure.kind, FailureKind::Invalid(_)));
    assert!(failure.stats.is_none());
}
#[test]
fn canceled_before_launch_never_allocates_a_native_process() {
    let cancel = Cancellation::default();
    cancel.cancel();
    let failure = program()
        .run(&args(&["hold"]), &ProcessLimits::default(), &cancel)
        .unwrap_err();
    assert_eq!(failure.kind, FailureKind::Canceled);
    assert!(failure.stats.is_none());
}
#[test]
fn running_cancel_and_deadline_kill_reap_and_join_output_readers_within_bound() {
    let cancel = Cancellation::default();
    let trigger = cancel.clone();
    let thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        trigger.cancel();
    });
    let now = Instant::now();
    let failure = program()
        .run(&args(&["hold"]), &ProcessLimits::default(), &cancel)
        .unwrap_err();
    thread.join().unwrap();
    assert_eq!(failure.kind, FailureKind::Canceled);
    let stats = failure.stats.unwrap();
    assert!(stats.exited);
    assert_eq!(stats.active_processes, 0);
    assert!(now.elapsed() < Duration::from_secs(3));
    let limits = ProcessLimits {
        deadline: Duration::from_millis(100),
        ..ProcessLimits::default()
    };
    let failure = program()
        .run(&args(&["hold"]), &limits, &Cancellation::default())
        .unwrap_err();
    assert_eq!(failure.kind, FailureKind::Deadline);
    assert_eq!(failure.stats.unwrap().active_processes, 0);
}
#[test]
fn output_and_stderr_overflow_reject_success_and_stop_the_native_writer() {
    for mode in ["stdout-flood", "stderr-flood"] {
        let limits = ProcessLimits {
            stdout_bytes: 1024,
            stderr_bytes: 1024,
            ..ProcessLimits::default()
        };
        let now = Instant::now();
        let failure = program()
            .run(&args(&[mode]), &limits, &Cancellation::default())
            .unwrap_err();
        assert_eq!(failure.kind, FailureKind::OutputBudget);
        assert_eq!(failure.stats.unwrap().active_processes, 0);
        assert!(now.elapsed() < Duration::from_secs(3));
    }
}
#[test]
fn process_committed_memory_is_limited_before_native_entry_and_failure_is_not_success() {
    let limits = ProcessLimits {
        native_committed_bytes: 32 * 1024 * 1024,
        ..ProcessLimits::default()
    };
    let failure = program()
        .run(&args(&["allocate"]), &limits, &Cancellation::default())
        .unwrap_err();
    assert!(matches!(failure.kind, FailureKind::NativeExit(_)));
    let stats = failure.stats.unwrap();
    assert!(stats.peak_committed_bytes <= limits.native_committed_bytes);
    assert_eq!(stats.active_processes, 0);
}
#[test]
fn descendant_creation_is_denied_by_the_declared_single_process_job() {
    let result = program()
        .run(
            &args(&["spawn-child"]),
            &ProcessLimits::default(),
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(result.stdout, b"child creation rejected");
    assert_eq!(result.stats.active_processes, 0);
}
#[test]
fn native_error_cannot_publish_stdout_as_success_and_parent_retains_stderr_diagnostic() {
    let failure = program()
        .run(
            &args(&["fail"]),
            &ProcessLimits::default(),
            &Cancellation::default(),
        )
        .unwrap_err();
    assert_eq!(failure.kind, FailureKind::NativeExit(42));
    assert_eq!(failure.stderr, b"fixture failure");
    assert_eq!(failure.stats.unwrap().active_processes, 0);
}
