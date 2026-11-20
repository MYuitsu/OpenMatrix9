// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows, feature = "test-fixtures"))]

use om9_core::{brep::Brep, budget::{Limits, Usage, memory::MemoryClass}, model::Document,
    transaction::{Cancellation, TransactionCoordinator, World}};
use om9_kernel::brep::{box_brep, capture_source_with_shape_set};
use om9_native_owner::{FailureKind, NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{io::Read, os::windows::fs::OpenOptionsExt};

fn guarded_fixture(path: &std::path::Path, bytes: &[u8]) -> (std::fs::File, String) {
    std::fs::write(path, bytes).unwrap();
    let mut owner = std::fs::OpenOptions::new().read(true).share_mode(1).custom_flags(0x0020_0000).open(path).unwrap();
    let metadata = owner.metadata().unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let mut checked = Vec::new();
    owner.read_to_end(&mut checked).unwrap();
    assert_eq!(checked, bytes);
    let fingerprint = Sha256::digest(&checked).iter().map(|byte| format!("{byte:02x}")).collect();
    (owner, fingerprint)
}

#[test]
fn shapeset_probe_is_contained_and_parent_validates_returned_geometry() {
    let limits = Limits { max_bytes: 1024 * 1024 * 1024, ..Limits::default() };
    let seed = box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let captured = capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_source();
    let dir = std::env::temp_dir().join(format!("om9-isolated-shapeset-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("captured.json");
    let bytes = serde_json::to_vec(&(captured.source(), captured.topology(), captured.shape_set())).unwrap();
    let (input_owner, input_hash) = guarded_fixture(&input, &bytes);
    assert!(std::fs::OpenOptions::new().write(true).open(&input).is_err());
    assert!(std::fs::remove_file(&input).is_err());
    let probe = std::path::PathBuf::from(env!("CARGO_BIN_EXE_om9-shapeset-probe"));
    let digest = Sha256::digest(std::fs::read(&probe).unwrap()).into();
    assert!(matches!(NativeProgram::admit(&probe, [0; 32]), Err(error) if matches!(error.kind, FailureKind::Fingerprint)));
    let program = NativeProgram::admit(&probe, digest).unwrap();
    let process_limits = ProcessLimits { native_committed_bytes: 256 * 1024 * 1024, ..Default::default() };
    let coordinator = TransactionCoordinator::new(World::new(vec![Document::new("isolated probe")], &limits).unwrap(), limits).unwrap();
    let held = Usage { bytes: seed.storage_usage(&limits).unwrap().bytes + captured.storage_bytes().unwrap()
        + bytes.capacity() as u64 + 8 * 1024 * 1024, ..Usage::default() };
    // The process reservation includes its bounded captured output. JSON
    // preflight and decoding receive a separate, incrementally grown lease.
    let allowance = process_limits.reservation_bytes().unwrap();
    let pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
    assert!(coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, allowance, None, held).is_err());
    drop(pressure);
    let reservation = coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, allowance, None, held).unwrap();
    let args = [input.as_os_str().to_owned(), input_hash.into()];
    let output = program.run(&args, &process_limits, &Cancellation::default()).unwrap();
    assert!(output.stats.exited);
    assert!(output.stats.peak_committed_bytes > 0);
    assert!(output.stats.peak_committed_bytes <= process_limits.native_committed_bytes);
    println!("isolated ShapeSet peak committed bytes: {}", output.stats.peak_committed_bytes);
    let mut decode = coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 1, None, held).unwrap();
    let usage = om9_archive::admit_json_usage_with_admission(&output.stdout, &limits, |needed| {
        if needed > decode.bytes() {
            let extra = needed - decode.bytes();
            coordinator.grow_external_memory_with_held(&mut decode, extra, None, held)?;
        }
        decode.recheck_pending()
    }).unwrap();
    println!("ShapeSet response JSON estimated bytes: {}", usage.bytes);
    assert!(om9_archive::admit_json_usage_with_admission(br#"{"x":1,"x":2}"#, &limits, |_| Ok(())).is_err());
    assert!(om9_archive::admit_json_usage_with_admission(&output.stdout, &limits, |_| Err(om9_core::Error::Budget("fixture decode"))).is_err());
    let restored: Brep = serde_json::from_slice(&output.stdout).unwrap();
    let restored_bytes = restored.storage_usage(&limits).unwrap().bytes;
    println!("ShapeSet restored fixture storage bytes: {restored_bytes}");
    assert!(restored_bytes <= usage.bytes);
    restored.validate(&limits).unwrap();
    assert_eq!(restored.source(), captured.source());
    let ids = |items: &[uuid::Uuid]| items.iter().copied().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids(&restored.topology().faces), ids(&captured.topology().faces));
    assert_eq!(ids(&restored.topology().edges), ids(&captured.topology().edges));
    assert_eq!(ids(&restored.topology().vertex_ids), ids(&captured.topology().vertex_ids));
    let adjacency = |topology: &om9_core::brep::BrepTopology| topology.face_edges.iter().map(|[face, edge]| {
        (topology.faces[*face as usize], topology.edges[*edge as usize])
    }).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(adjacency(restored.topology()), adjacency(captured.topology()));

    let invalid = dir.join("bad.json");
    let bad_packet = b"OM9-SHAPESET 1\n6 12 8\nnot-a-shape";
    let bad_bytes = serde_json::to_vec(&(captured.source(), captured.topology(), bad_packet.as_slice())).unwrap();
    let (bad_owner, bad_hash) = guarded_fixture(&invalid, &bad_bytes);
    let failure = program.run(&[invalid.as_os_str().to_owned(), bad_hash.into()], &process_limits, &Cancellation::default()).unwrap_err();
    assert!(matches!(failure.kind, FailureKind::NativeExit(_)));

    let wrong_hash = [input.as_os_str().to_owned(), "0".repeat(64).into()];
    let failure = program.run(&wrong_hash, &process_limits, &Cancellation::default()).unwrap_err();
    assert!(matches!(failure.kind, FailureKind::NativeExit(_)));
    assert!(String::from_utf8_lossy(&failure.stderr).contains("fixture fingerprint mismatch"));

    let output_limited = ProcessLimits { stdout_bytes: 32, ..process_limits };
    let failure = program.run(&args, &output_limited, &Cancellation::default()).unwrap_err();
    assert!(matches!(failure.kind, FailureKind::OutputBudget));
    drop((restored, output, decode, reservation));
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    drop(program);
    drop((input_owner, bad_owner));
    std::fs::remove_dir_all(dir).unwrap();
}
