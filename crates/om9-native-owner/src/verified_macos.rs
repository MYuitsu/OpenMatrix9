// SPDX-License-Identifier: LGPL-2.1-or-later
//! Retain a verified regular staging file and publish with an atomic Darwin
//! rename. Locks coordinate cooperating processes. The stage must be owned
//! by the app and is not a security boundary against the same user.
use super::*;
use crate::io;
use sha2::{Digest, Sha256};
use std::{
    ffi::CString,
    fs::{File, Metadata},
    io::{Read, Seek, SeekFrom},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
};

fn invalid(reason: &'static str) -> Failure {
    Failure::new(FailureKind::Invalid(reason))
}

fn cancel_point(cancel: &Cancellation) -> Result<()> {
    if cancel.is_canceled() {
        return Err(Failure::new(FailureKind::Canceled));
    }
    Ok(())
}

fn same_file(a: &Metadata, b: &Metadata) -> bool {
    a.is_file() && b.is_file()
        && (a.dev(), a.ino(), a.len(), a.ctime(), a.ctime_nsec())
            == (b.dev(), b.ino(), b.len(), b.ctime(), b.ctime_nsec())
}

fn fingerprint(file: &File, max_bytes: u64, cancel: &Cancellation) -> Result<([u8; 32], u64)> {
    let base = file.metadata().map_err(io)?;
    if !base.is_file() || base.nlink() != 1 || base.len() == 0 || base.len() > max_bytes {
        return Err(invalid("verified staged file shape or size"));
    }
    let mut input = file;
    input.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 8192];
    let mut total = 0;
    loop {
        cancel_point(cancel)?;
        let read = input.read(&mut buffer).map_err(io)?;
        if read == 0 {
            break;
        }
        total += read as u64;
        if total > base.len() {
            return Err(Failure::new(FailureKind::Fingerprint));
        }
        digest.update(&buffer[..read]);
    }
    if total != base.len() || !same_file(&base, &file.metadata().map_err(io)?) {
        return Err(Failure::new(FailureKind::Fingerprint));
    }
    Ok((digest.finalize().into(), total))
}

fn name(path: &Path) -> Result<CString> {
    if !path.is_absolute() {
        return Err(invalid("verified file path must be absolute"));
    }
    CString::new(path.as_os_str().as_bytes()).map_err(|_| invalid("NUL in verified file path"))
}

pub(super) fn open(path: &Path, max_bytes: u64, cancel: &Cancellation) -> Result<VerifiedFile> {
    name(path)?;
    let file = crate::open_regular_read_locked(path).map_err(io)?;
    let (sha256, bytes) = fingerprint(&file, max_bytes, cancel)?;
    if !same_file(&file.metadata().map_err(io)?, &std::fs::symlink_metadata(path).map_err(io)?) {
        return Err(Failure::new(FailureKind::Fingerprint));
    }
    Ok(VerifiedFile { file, path: path.to_path_buf(), sha256, bytes })
}

fn verify(file: &VerifiedFile, cancel: &Cancellation) -> Result<()> {
    if fingerprint(&file.file, file.bytes, cancel)? != (file.sha256, file.bytes)
        || !same_file(&file.file.metadata().map_err(io)?,
                      &std::fs::symlink_metadata(&file.path).map_err(io)?)
    {
        return Err(Failure::new(FailureKind::Fingerprint));
    }
    cancel_point(cancel)
}

pub(super) fn publish(file: &VerifiedFile, destination: &Path, cancel: &Cancellation,
                      replace: bool) -> Result<()> {
    let from = name(&file.path)?;
    let to = name(destination)?;
    if replace {
        match std::fs::symlink_metadata(destination) {
            Ok(info) if !info.is_file() || info.permissions().readonly() =>
                return Err(invalid("verified destination is not a writable regular file")),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io(error)),
        }
    }
    verify(file, cancel)?;
    let flags = if replace { 0 } else { libc::RENAME_EXCL };
    // SAFETY: both paths are live C strings. RENAME_EXCL delegates the
    // no-replace check to the atomic filesystem operation.
    if unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), flags) } != 0 {
        return Err(io(std::io::Error::last_os_error()));
    }
    Ok(())
}

pub(super) fn rollback(file: &VerifiedFile) -> Result<()> {
    verify(file, &Cancellation::default())?;
    std::fs::remove_file(&file.path).map_err(io)
}
