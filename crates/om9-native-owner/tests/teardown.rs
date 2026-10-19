// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
use om9_core::transaction::Cancellation;
use om9_native_owner::{FailureKind, NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

#[test]
fn concurrent_short_success_and_failure_jobs_reap_before_returning_bytes_or_error() {
    let path = Path::new(env!("CARGO_BIN_EXE_om9-process-fixture"));
    let hash: [u8; 32] = Sha256::digest(std::fs::read(path).unwrap()).into();
    let program = Arc::new(NativeProgram::admit(path, hash).unwrap());
    let barrier = Arc::new(Barrier::new(8));
    thread::scope(|scope| {
        for worker in 0..8 {
            let program = program.clone();
            let barrier = barrier.clone();
            scope.spawn(move || {
                let limits = ProcessLimits::default();
                barrier.wait();
                for iteration in 0..16 {
                    let fail = (worker + iteration) % 2 != 0;
                    let args: Vec<OsString> = if fail {
                        vec!["fail".into()]
                    } else {
                        vec!["echo".into(), "bounded concurrent result".into()]
                    };
                    let result = program.run(&args, &limits, &Cancellation::default());
                    let stats = if fail {
                        let failure = result.unwrap_err();
                        assert_eq!(failure.kind, FailureKind::NativeExit(42));
                        assert_eq!(failure.stderr, b"fixture failure");
                        failure.stats.expect("native error job accounting")
                    } else {
                        let output = result.unwrap();
                        assert_eq!(output.stdout, b"bounded concurrent result");
                        assert_eq!(output.stderr, b"fixture diagnostics");
                        output.stats
                    };
                    assert!(stats.pid != 0 && stats.exited);
                    assert_eq!(stats.active_processes, 0);
                    assert!(
                        stats.peak_committed_bytes > 0
                            && stats.peak_committed_bytes <= limits.native_committed_bytes
                    );
                }
            });
        }
    });
}
