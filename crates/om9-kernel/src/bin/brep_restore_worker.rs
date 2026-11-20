// SPDX-License-Identifier: LGPL-2.1-or-later
//! Restore one BRep in an externally contained worker process.
//!
//! This is not an archive reader or a publication authority. The parent must
//! admit this executable from trusted configuration, retain a verified input
//! owner, enforce process/output limits, and validate the returned candidate.
use om9_core::{brep::BrepTopology, budget::{Limits, Usage}};
use sha2::{Digest, Sha256};
use std::{io::{Read, Write}, path::PathBuf};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or("missing input path")?);
    let expected = args.next().ok_or("missing input fingerprint")?.into_string().map_err(|_| "invalid fingerprint")?;
    let input_limit: u64 = args.next().ok_or("missing input limit")?.into_string().map_err(|_| "invalid input limit")?.parse()?;
    let working_limit: u64 = args.next().ok_or("missing working limit")?.into_string().map_err(|_| "invalid working limit")?.parse()?;
    if args.next().is_some() || !path.is_absolute() || expected.len() != 64
        || !expected.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || input_limit == 0 || input_limit > 1024 * 1024 * 1024 || working_limit < input_limit
        || working_limit > 4 * 1024 * 1024 * 1024 {
        return Err("invalid worker request".into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // The parent's verified read handle also owns DELETE rights for its
        // lifecycle. Permit that existing handle while requesting read only;
        // its own sharing mode still excludes new writers and deleters.
        options.share_mode(1 | 4).custom_flags(0x0020_0000);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > input_limit {
        return Err("invalid worker input file".into());
    }
    let mut bytes = Vec::new();
    file.take(input_limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > input_limit { return Err("worker input size changed".into()); }
    let actual = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    if actual != expected { return Err("worker input fingerprint mismatch".into()); }
    let (version, source, topology, packet, deflection, angular): (u32, String, BrepTopology, Vec<u8>, f64, f64) =
        serde_json::from_slice(&bytes)?;
    if version != 1 { return Err("unsupported worker request version".into()); }
    drop(bytes);
    let limits = Limits { max_bytes: working_limit, ..Limits::default() };
    let count = topology.faces.capacity().checked_add(topology.edges.capacity())
        .and_then(|value| value.checked_add(topology.vertex_ids.capacity())).ok_or("topology size overflow")?;
    let held = (source.capacity() as u64).checked_add(packet.capacity() as u64)
        .and_then(|value| value.checked_add(Usage::array::<uuid::Uuid>(count).ok()?.bytes))
        .and_then(|value| value.checked_add(Usage::array::<[u32; 2]>(topology.face_edges.capacity()).ok()?.bytes))
        .and_then(|value| value.checked_add(std::mem::size_of::<BrepTopology>() as u64))
        .ok_or("worker input accounting overflow")?;
    let result = om9_kernel::brep::restore_isolated_worker_input(&packet, &source, &topology, held,
        deflection, angular, &limits, |_, _| Ok(()))?;
    serde_json::to_writer(std::io::stdout().lock(), &result)?;
    std::io::stdout().lock().flush()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
