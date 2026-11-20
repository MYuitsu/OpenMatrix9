// SPDX-License-Identifier: LGPL-2.1-or-later
//! Test-only isolated ShapeSet probe. Not an archive reader or product worker.
use om9_core::{brep::BrepTopology, budget::{Limits, Usage}};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("missing fixture path")?;
    let expected = args.next().ok_or("missing fixture fingerprint")?.into_string().map_err(|_| "invalid fingerprint")?;
    if args.next().is_some() || expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err("invalid fixture fingerprint".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 8 * 1024 * 1024 { return Err("fixture input limit".into()); }
    let actual = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    if actual != expected { return Err("fixture fingerprint mismatch".into()); }
    let (source, topology, packet): (String, BrepTopology, Vec<u8>) = serde_json::from_slice(&bytes)?;
    drop(bytes);
    let limits = Limits::default();
    let held = source.capacity() as u64 + packet.capacity() as u64
        + Usage::array::<uuid::Uuid>(topology.faces.capacity() + topology.edges.capacity() + topology.vertex_ids.capacity())?.bytes
        + Usage::array::<[u32; 2]>(topology.face_edges.capacity())?.bytes
        + std::mem::size_of::<BrepTopology>() as u64;
    // The Job Object bounds the whole process, including JSON and OCCT's
    // internal parser allocations; this is not trusted-file admission.
    let restored = om9_kernel::brep::test_restore_captured_shape_set(&packet, &source, &topology,
        held, 0.1, 0.3, &limits, |_, _| Ok(()))?;
    serde_json::to_writer(std::io::stdout().lock(), &restored)?;
    std::io::stdout().lock().flush()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
