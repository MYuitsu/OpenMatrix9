// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::Archive;
use om9_core::{Error, Result, budget::Limits, model::Document, transaction::Cancellation};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn io(error: std::io::Error) -> Error {
    Error::Io(error.to_string())
}
fn cancel(token: &Cancellation) -> Result<()> {
    if token.is_canceled() {
        Err(Error::Canceled)
    } else {
        Ok(())
    }
}
struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
pub fn read_bounded(path: &Path, limits: &Limits) -> Result<Vec<u8>> {
    let mut file = File::open(path).map_err(io)?;
    let size = file.metadata().map_err(io)?.len();
    if size > limits.max_bytes {
        return Err(Error::Budget("archive file bytes"));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(
            usize::try_from(size).map_err(|_| Error::Overflow("file address space"))?,
        )
        .map_err(|_| Error::Budget("file read allocation"))?;
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(io)?;
        if count == 0 {
            break;
        }
        let next = bytes
            .len()
            .checked_add(count)
            .ok_or(Error::Overflow("file read bytes"))?;
        if next as u64 > limits.max_bytes {
            return Err(Error::Budget("growing archive file"));
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("file read allocation"))?;
        bytes.extend_from_slice(&buffer[..count]);
    }
    Ok(bytes)
}
pub fn save(path: &Path, document: &Document, limits: &Limits, token: &Cancellation) -> Result<()> {
    cancel(token)?;
    let bytes = Archive::encode(document, limits)?;
    cancel(token)?;
    let held = document.validate(limits)?;
    let mut retained = held;
    retained.add(
        om9_core::budget::Usage {
            bytes: bytes.capacity() as u64,
            ..Default::default()
        },
        limits,
    )?;
    let validation = limits.remaining_after(retained)?;
    // Prove the private decoder/model copy fits alongside the live document and
    // encoded buffer before creating any staging file.
    if Archive::decode(&bytes, &validation)? != *document {
        return Err(Error::Invalid("archive encoding verification"));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let filename = path
        .file_name()
        .ok_or(Error::Invalid("save destination filename"))?;
    // Reject readonly targets consistently, including Windows rename variants
    // that could otherwise override readonly through parent permissions.
    if let Ok(metadata) = fs::symlink_metadata(path)
        && (!metadata.is_file() || metadata.permissions().readonly())
    {
        return Err(Error::Permission("save destination"));
    }
    let temporary = parent.join(format!(
        ".{}.{}.om9-tmp",
        filename.to_string_lossy(),
        uuid::Uuid::new_v4()
    ));
    let staging = Staging(temporary);
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staging.0)
        .map_err(io)?;
    file.write_all(&bytes).map_err(io)?;
    file.flush().map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    cancel(token)?;
    let reread = read_bounded(&staging.0, &validation)?;
    if reread != bytes || Archive::decode(&reread, &validation)? != *document {
        return Err(Error::Invalid("staged archive verification"));
    }
    cancel(token)?;
    #[cfg(windows)]
    {
        use om9_native_owner::VerifiedFile;
        use sha2::{Digest, Sha256};
        let temporary = std::path::absolute(&staging.0).map_err(io)?;
        let destination = std::path::absolute(path).map_err(io)?;
        let held =
            VerifiedFile::open(&temporary, bytes.len() as u64, token).map_err(
                |error| match error.kind {
                    om9_native_owner::FailureKind::Canceled => Error::Canceled,
                    _ => Error::Io(format!("archive publication ownership: {error:?}")),
                },
            )?;
        let expected: [u8; 32] = Sha256::digest(&bytes).into();
        if held.len() != bytes.len() as u64 || held.fingerprint() != expected {
            return Err(Error::Invalid("staged archive publication fingerprint"));
        }
        cancel(token)?;
        held.publish(&destination, token)
            .map_err(|error| match error.kind {
                om9_native_owner::FailureKind::Canceled => Error::Canceled,
                _ => Error::Io(format!("archive held-file publication: {error:?}")),
            })?;
    }
    #[cfg(not(windows))]
    fs::rename(&staging.0, path).map_err(io)?;
    // Rename accepted the new file. Drop removes only the now-absent staging
    // name and never treats postacceptance cleanup as a rollback.
    Ok(())
}
