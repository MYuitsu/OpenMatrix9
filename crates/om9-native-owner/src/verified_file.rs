// SPDX-License-Identifier: LGPL-2.1-or-later
//! Final-publication file ownership. The handle that supplies the admitted
//! fingerprint also performs the atomic rename. This is ordinary file/resource
//! ownership, not a security sandbox or a general filesystem integrity claim.
use crate::{Failure, FailureKind, Result};
use om9_core::transaction::Cancellation;
use std::path::Path;

/// Owns one regular staged file with write/delete sharing denied until the
/// accepted bytes have been renamed or the guard is dropped. Acquire this only
/// after native cold inspection; native CRT readers use independent handles.
#[derive(Debug)]
pub struct VerifiedFile {
    #[cfg(windows)]
    file: std::fs::File,
    #[cfg(windows)]
    metadata: platform::Snapshot,
    sha256: [u8; 32],
    bytes: u64,
}
impl VerifiedFile {
    /// Open, bound and stream-hash the same held file. Ordinary writers and
    /// namespace replacement cannot enter after successful admission.
    pub fn open(path: &Path, max_bytes: u64, cancel: &Cancellation) -> Result<Self> {
        if cancel.is_canceled() {
            return Err(Failure::new(FailureKind::Canceled));
        }
        #[cfg(windows)]
        {
            platform::open(path, max_bytes, cancel)
        }
        #[cfg(not(windows))]
        {
            let _ = (path, max_bytes);
            Err(Failure::new(FailureKind::Invalid(
                "verified file publication requires Windows",
            )))
        }
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn len(&self) -> u64 {
        self.bytes
    }
    pub fn is_empty(&self) -> bool {
        self.bytes == 0
    }
    /// Atomically rename the admitted handle, replacing an existing regular,
    /// writable destination. Failure leaves both existing files intact. Once
    /// the OS accepts the rename, closing the owner cannot report a rollback.
    pub fn publish(self, destination: &Path, cancel: &Cancellation) -> Result<()> {
        if cancel.is_canceled() {
            return Err(Failure::new(FailureKind::Canceled));
        }
        #[cfg(windows)]
        {
            platform::publish(self, destination, cancel)
        }
        #[cfg(not(windows))]
        {
            let _ = destination;
            Err(Failure::new(FailureKind::Invalid(
                "verified file publication requires Windows",
            )))
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use crate::io;
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::Read,
        mem::{align_of, offset_of, size_of},
        os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
        ptr::{copy_nonoverlapping, null_mut},
    };

    const REPLACE_IF_EXISTS: u32 = 1;
    const _: () = assert!(align_of::<FILE_RENAME_INFO>() <= align_of::<usize>());
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, HANDLE},
        Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY,
            FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAG_SEQUENTIAL_SCAN,
            FILE_RENAME_INFO, FILE_RENAME_INFO_0, FILE_SHARE_READ, FILE_TYPE_DISK,
            FileRenameInfoEx, GetFileInformationByHandle, GetFileType, SetFileInformationByHandle,
        },
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) struct Snapshot {
        attributes: u32,
        volume: u32,
        file_index: u64,
        links: u32,
        bytes: u64,
        created: [u32; 2],
        written: [u32; 2],
    }
    fn canceled(cancel: &Cancellation) -> Result<()> {
        if cancel.is_canceled() {
            Err(Failure::new(FailureKind::Canceled))
        } else {
            Ok(())
        }
    }
    fn invalid(message: &'static str) -> Failure {
        Failure::new(FailureKind::Invalid(message))
    }
    fn handle(file: &File) -> HANDLE {
        file.as_raw_handle()
    }
    fn snapshot(file: &File) -> Result<Snapshot> {
        let raw = handle(file);
        // SAFETY: the uniquely owned File keeps this live handle open for both
        // calls. The initialized typed metadata output has the exact ABI size.
        if unsafe { GetFileType(raw) } != FILE_TYPE_DISK {
            return Err(invalid("verified staged file must be a regular disk file"));
        }
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: all-zero BY_HANDLE_FILE_INFORMATION is a valid writable
        // output structure and remains live throughout the call.
        if unsafe { GetFileInformationByHandle(raw, &mut info) } == 0 {
            return Err(io(std::io::Error::last_os_error()));
        }
        if info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
            return Err(invalid("verified staged file directory/reparse point"));
        }
        Ok(Snapshot {
            attributes: info.dwFileAttributes,
            volume: info.dwVolumeSerialNumber,
            file_index: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
            links: info.nNumberOfLinks,
            bytes: (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow),
            created: [
                info.ftCreationTime.dwHighDateTime,
                info.ftCreationTime.dwLowDateTime,
            ],
            written: [
                info.ftLastWriteTime.dwHighDateTime,
                info.ftLastWriteTime.dwLowDateTime,
            ],
        })
    }
    fn absolute_name(path: &Path) -> Result<Vec<u16>> {
        if !path.is_absolute() {
            return Err(invalid("verified file path must be absolute"));
        }
        let units = path.as_os_str().encode_wide().count();
        if units == 0 || units > 32766 {
            return Err(invalid("verified file UTF-16 path length"));
        }
        let mut wide = Vec::new();
        wide.try_reserve_exact(units + 1)
            .map_err(|_| invalid("verified file path allocation"))?;
        wide.extend(path.as_os_str().encode_wide());
        if wide.contains(&0) {
            return Err(invalid("verified file NUL in path"));
        }
        wide.push(0);
        Ok(wide)
    }
    pub(super) fn open(path: &Path, max_bytes: u64, cancel: &Cancellation) -> Result<VerifiedFile> {
        canceled(cancel)?;
        if max_bytes == 0 {
            return Err(invalid("verified file byte budget"));
        }
        let _ = absolute_name(path)?;
        // OpenOptions transfers the one Win32 handle directly into File. Read
        // + DELETE rights permit the later handle-based rename; READ sharing
        // denies existing or future ordinary write/delete/name-replacement owners.
        // OPEN_REPARSE_POINT inspects the named entry rather than following a
        // symlink. A successful open is not a claim about hostile mapped views.
        let mut file = OpenOptions::new()
            .read(true)
            .access_mode(GENERIC_READ | DELETE)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_SEQUENTIAL_SCAN)
            .open(path)
            .map_err(io)?;
        let before = snapshot(&file)?;
        if before.bytes == 0 || before.bytes > max_bytes {
            return Err(invalid("verified file empty/oversize"));
        }
        let mut digest = Sha256::new();
        let mut buffer = [0; 8192];
        let mut bytes = 0u64;
        loop {
            canceled(cancel)?;
            let count = file.read(&mut buffer).map_err(io)?;
            if count == 0 {
                break;
            }
            bytes = bytes
                .checked_add(count as u64)
                .ok_or_else(|| invalid("verified file byte overflow"))?;
            if bytes > max_bytes || bytes > before.bytes {
                return Err(invalid("verified file grew while hashing"));
            }
            digest.update(&buffer[..count]);
        }
        canceled(cancel)?;
        if bytes != before.bytes || snapshot(&file)? != before {
            return Err(Failure::new(FailureKind::Fingerprint));
        }
        Ok(VerifiedFile {
            file,
            metadata: before,
            sha256: digest.finalize().into(),
            bytes,
        })
    }
    fn check_destination(destination: &Path) -> Result<()> {
        use std::os::windows::fs::MetadataExt;
        // Inspect the named target without following its final reparse point.
        // The rename itself remains one kernel operation and does not request
        // IGNORE_READONLY_ATTRIBUTE or POSIX deletion semantics.
        match std::fs::symlink_metadata(destination) {
            Ok(metadata) => {
                if !metadata.is_file()
                    || metadata.file_attributes()
                        & (FILE_ATTRIBUTE_DIRECTORY
                            | FILE_ATTRIBUTE_REPARSE_POINT
                            | FILE_ATTRIBUTE_READONLY)
                        != 0
                {
                    return Err(invalid(
                        "publication target readonly/nonregular/reparse point",
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io(error)),
        }
        Ok(())
    }
    pub(super) fn publish(
        owner: VerifiedFile,
        destination: &Path,
        cancel: &Cancellation,
    ) -> Result<()> {
        canceled(cancel)?;
        let name = absolute_name(destination)?;
        if destination.file_name().is_none() {
            return Err(invalid("publication destination filename"));
        }
        check_destination(destination)?;
        if snapshot(&owner.file)? != owner.metadata {
            return Err(Failure::new(FailureKind::Fingerprint));
        }
        // FILE_RENAME_INFO has a variable UTF-16 tail. Its first byte must have
        // the struct's alignment, and the payload must include all name bytes.
        // Word-backed storage provides that alignment on both 32- and 64-bit
        // Windows. Include a terminal NUL, but exclude it from FileNameLength.
        let name_bytes = name
            .len()
            .checked_sub(1)
            .and_then(|n| n.checked_mul(size_of::<u16>()))
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| invalid("publication UTF-16 filename length overflow"))?;
        let tail_bytes = name
            .len()
            .checked_mul(size_of::<u16>())
            .ok_or_else(|| invalid("publication UTF-16 buffer overflow"))?;
        let buffer_bytes = offset_of!(FILE_RENAME_INFO, FileName)
            .checked_add(tail_bytes)
            .map(|n| n.max(size_of::<FILE_RENAME_INFO>()))
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| invalid("publication rename buffer overflow"))?;
        let words = (buffer_bytes as usize).div_ceil(size_of::<usize>());
        let mut storage = Vec::<usize>::new();
        storage
            .try_reserve_exact(words)
            .map_err(|_| invalid("publication rename buffer allocation"))?;
        storage.resize(words, 0);
        let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
        // SAFETY: the word allocation is aligned for FILE_RENAME_INFO and
        // contains at least buffer_bytes initialized bytes. Its tail has room
        // for the complete owned UTF-16 name (including the extra NUL), and the
        // two nonoverlapping allocations remain live until the system call.
        unsafe {
            (*info).Anonymous = FILE_RENAME_INFO_0 {
                Flags: REPLACE_IF_EXISTS,
            };
            (*info).RootDirectory = null_mut();
            (*info).FileNameLength = name_bytes;
            let tail = storage
                .as_mut_ptr()
                .cast::<u8>()
                .add(offset_of!(FILE_RENAME_INFO, FileName))
                .cast::<u16>();
            copy_nonoverlapping(name.as_ptr(), tail, name.len());
        }
        // This is the acceptance boundary. No subsequent cancellation or
        // cleanup error is reported as if an accepted publication rolled back.
        canceled(cancel)?;
        // SAFETY: the held file handle is the same handle that supplied the
        // fingerprint; DELETE access was acquired at open. The exact aligned
        // owned buffer and its checked byte length remain live for the call.
        if unsafe {
            SetFileInformationByHandle(
                handle(&owner.file),
                FileRenameInfoEx,
                info.cast(),
                buffer_bytes,
            )
        } == 0
        {
            return Err(io(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}
