// SPDX-License-Identifier: LGPL-2.1-or-later
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

/// Fully verified staging file. Dropping it leaves the destination untouched.
/// The caller must revalidate its operation context before calling `publish`.
pub struct PreparedArchive {
    #[cfg(any(windows, target_os = "macos"))]
    held: om9_native_owner::VerifiedFile,
    destination: PathBuf,
    staging: Staging,
}
impl PreparedArchive {
    pub fn publish(self, token: &Cancellation) -> Result<()> {
        cancel(token)?;
        #[cfg(any(windows, target_os = "macos"))]
        self.held.publish(&self.destination, token).map_err(|error| match error.kind {
            om9_native_owner::FailureKind::Canceled => Error::Canceled,
            _ => Error::Io(format!("archive held-file publication: {error:?}")),
        })?;
        #[cfg(not(any(windows, target_os = "macos")))]
        fs::rename(&self.staging.0, &self.destination).map_err(io)?;
        Ok(())
    }
}
pub fn read_bounded(path: &Path, limits: &Limits) -> Result<Vec<u8>> {
    read_bounded_reserved(path, limits, |_| Ok(())).map(|(bytes, ())| bytes)
}

/// Reserve the bytes from the same opened file before allocating the input.
/// Reject growth beyond that reservation even if the caller's limit is higher.
pub fn read_bounded_reserved<T>(path: &Path, limits: &Limits, reserve: impl FnOnce(u64) -> Result<T>) -> Result<(Vec<u8>, T)> {
    let mut file = File::open(path).map_err(io)?;
    let size = file.metadata().map_err(io)?.len();
    if size > limits.max_bytes {
        return Err(Error::Budget("archive file bytes"));
    }
    let reservation = reserve(size)?;
    let bytes = read_sized(&mut file, size)?;
    Ok((bytes, reservation))
}

fn read_sized(file: &mut impl Read, size: u64) -> Result<Vec<u8>> {
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
        if next as u64 > size {
            return Err(Error::Budget("growing archive file"));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    if bytes.len() as u64 != size {
        return Err(Error::Stale("archive file size changed during read"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_source_size_changes_while_reading() {
        assert_eq!(read_sized(&mut &b"abc"[..], 3).unwrap(), b"abc");
        assert!(matches!(read_sized(&mut &b"ab"[..], 3), Err(Error::Stale(_))));
        assert!(matches!(read_sized(&mut &b"abcd"[..], 3), Err(Error::Budget(_))));
    }
}
pub fn save(path: &Path, document: &Document, limits: &Limits, token: &Cancellation) -> Result<()> {
    save_with_admission(path, document, limits, token, &mut |_, _| Ok(()))
}
pub(crate) fn save_with_admission(path: &Path, document: &Document, limits: &Limits,
    token: &Cancellation, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<()> {
    cancel(token)?;
    let bytes = crate::container::encode_with_admission(document, limits, admit)?;
    let encoded = bytes.capacity() as u64;
    cancel(token)?;
    admit(encoded, document.validation_scratch_usage(limits)?.bytes)?;
    let held = document.validate(limits)?;
    admit(encoded, 0)?;
    let mut retained = held;
    retained.add(
        om9_core::budget::Usage {
            bytes: bytes.capacity() as u64,
            ..Default::default()
        },
        limits,
    )?;
    // The verification copy coexists in memory with the live document, but
    // structural limits describe one document, not the sum of two versions.
    let validation = limits.remaining_after(om9_core::budget::Usage {
        bytes: retained.bytes,
        ..Default::default()
    })?;
    // Prove the private decoder/model copy fits alongside the live document and
    // encoded buffer before creating any staging file.
    let decoded = crate::container::decode_with_admission(&bytes, &validation, &mut |estimated| {
        admit(encoded, estimated.checked_sub(bytes.len() as u64)
            .ok_or(Error::Invalid("archive decoder input accounting"))?)
    })?;
    if decoded != *document {
        return Err(Error::Invalid("archive encoding verification"));
    }
    drop(decoded);
    admit(encoded, 0)?;
    publish_encoded(path, &bytes, &validation, token, admit, &mut |reread, admit| {
        let live = encoded.checked_add(reread.capacity() as u64).ok_or(Error::Overflow("archive readback capacity"))?;
        let decoded = crate::container::decode_with_admission(reread, &validation, &mut |estimated| {
            admit(live, estimated.checked_sub(reread.len() as u64)
                .ok_or(Error::Invalid("archive decoder input accounting"))?)
        })?;
        if decoded != *document { return Err(Error::Invalid("staged archive verification")); }
        Ok(())
    })
}

/// Publish already validated bytes only after an independently read staging
/// copy is identical and passes the caller's format-specific verification.
/// The caller retains and accounts for `bytes` until this function returns.
pub(crate) fn publish_encoded(path: &Path, bytes: &Vec<u8>, validation: &Limits,
    token: &Cancellation, admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    verify: &mut dyn FnMut(&Vec<u8>, &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<()>) -> Result<()> {
    prepare_encoded(path, bytes, validation, token, admit, verify)?.publish(token)
}

pub(crate) fn prepare_encoded(path: &Path, bytes: &Vec<u8>, validation: &Limits,
    token: &Cancellation, admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    verify: &mut dyn FnMut(&Vec<u8>, &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<()>) -> Result<PreparedArchive> {
    let encoded = bytes.capacity() as u64;
    cancel(token)?;
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
    file.write_all(bytes).map_err(io)?;
    file.flush().map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    cancel(token)?;
    let (reread, ()) = read_bounded_reserved(&staging.0, &validation, |size| admit(encoded, size))?;
    let live = encoded.checked_add(reread.capacity() as u64).ok_or(Error::Overflow("archive readback capacity"))?;
    admit(live, 0)?;
    if reread != *bytes {
        return Err(Error::Invalid("staged archive verification"));
    }
    verify(&reread, admit)?;
    drop(reread);
    admit(encoded, 0)?;
    cancel(token)?;
    #[cfg(any(windows, target_os = "macos"))]
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
        let expected: [u8; 32] = Sha256::digest(bytes).into();
        if held.len() != bytes.len() as u64 || held.fingerprint() != expected {
            return Err(Error::Invalid("staged archive publication fingerprint"));
        }
        cancel(token)?;
        return Ok(PreparedArchive { held, destination, staging });
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    Ok(PreparedArchive { destination: path.to_path_buf(), staging })
}
