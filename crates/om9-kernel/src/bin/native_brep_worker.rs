// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rebuild a derived BRep display from one opaque 3DM payload in a contained process.
//! The caller must pin the executable, retain the verified input, enforce process
//! limits, and validate the returned candidate before binding it to its source.
use om9_core::budget::Limits;
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
    let (version, payload, tolerance, deflection, angular): (u32, Vec<u8>, f64, f64, f64) = serde_json::from_slice(&bytes)?;
    if version != 1 { return Err("unsupported worker request version".into()); }
    drop(bytes);
    let limits = Limits { max_bytes: working_limit, ..Limits::default() };
    let held = payload.capacity() as u64;
    if held > limits.max_bytes { return Err("worker payload budget".into()); }
    let result = om9_kernel::brep::from_3dm_with_angle_admitted(&payload, tolerance, deflection, angular, &limits, |live, pending| {
        held.checked_add(live).and_then(|n| n.checked_add(pending))
            .filter(|n| *n <= limits.max_bytes).map(|_| ()).ok_or(om9_core::Error::Budget("native BRep worker"))
    })?;
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
