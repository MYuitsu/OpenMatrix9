// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg_attr(windows, windows_subsystem = "windows")]
//! Explicit developer receipt tool, not the application's capability registry.
use om9_core::transaction::Cancellation;
use om9_native_owner::{NativeProgram, ProcessLimits, ProcessStats};
use std::path::Path;
fn stats(s: &ProcessStats) -> serde_json::Value {
    serde_json::json!({"pid":s.pid,"peak_committed_bytes":s.peak_committed_bytes,"active_processes":s.active_processes,"exited":s.exited,"elapsed_ms":s.elapsed.as_millis()})
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 5 || args[0] != "--program" || args[2] != "--sha256" || args[4] != "--" {
        eprintln!("usage: om9-native-runner --program absolute.exe --sha256 hex -- arguments");
        std::process::exit(2);
    }
    let Some(digest) = args[3]
        .to_str()
        .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
    else {
        eprintln!("invalid native fingerprint");
        std::process::exit(2);
    };
    let mut hash = [0; 32];
    for (i, b) in hash.iter_mut().enumerate() {
        *b = u8::from_str_radix(&digest[i * 2..i * 2 + 2], 16).expect("validated hex");
    }
    let program = match NativeProgram::admit(Path::new(&args[1]), hash) {
        Ok(p) => p,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let limits = ProcessLimits::default();
    match program.run(&args[5..], &limits, &Cancellation::default()) {
        Ok(output) => println!(
            "{}",
            serde_json::json!({"status":"native_owner_result","limits":{"native_committed_bytes":limits.native_committed_bytes,"stdout_bytes":limits.stdout_bytes,"stderr_bytes":limits.stderr_bytes,"reservation_bytes":limits.reservation_bytes().expect("limits")},"stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr),"stats":stats(&output.stats)})
        ),
        Err(error) => {
            println!(
                "{}",
                serde_json::json!({"status":"native_owner_rejected","reason":format!("{:?}",error.kind),"stderr":String::from_utf8_lossy(&error.stderr),"stats":error.stats.as_ref().map(stats)})
            );
            std::process::exit(1);
        }
    }
}
