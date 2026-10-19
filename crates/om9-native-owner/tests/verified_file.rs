// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
use om9_core::transaction::Cancellation;
use om9_native_owner::{FailureKind, VerifiedFile};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

struct Folder(PathBuf);
impl Folder {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("om9-verified-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Folder {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn bounded_fingerprint_keeps_one_handle_and_denies_content_or_name_changes() {
    let folder = Folder::new();
    let content = vec![b'a'; 200_000];
    let staged = folder.file("private.3dm", &content);
    let other = folder.file("replacement.3dm", &vec![b'z'; content.len()]);
    let destination = folder.file("good.3dm", b"good prior file");
    let cancel = Cancellation::default();
    let guard = VerifiedFile::open(&staged, content.len() as u64, &cancel).unwrap();
    assert_eq!(guard.len(), content.len() as u64);
    assert_eq!(
        guard.fingerprint(),
        <[u8; 32]>::from(Sha256::digest(&content))
    );
    assert!(fs::write(&staged, vec![b'z'; content.len()]).is_err());
    assert!(fs::OpenOptions::new().write(true).open(&staged).is_err());
    assert!(fs::remove_file(&staged).is_err());
    assert!(fs::rename(&staged, folder.0.join("stolen.3dm")).is_err());
    assert!(fs::rename(&other, &staged).is_err());
    assert_eq!(fs::read(&staged).unwrap(), content);
    guard.publish(&destination, &cancel).unwrap();
    assert!(!staged.exists());
    assert_eq!(fs::read(&destination).unwrap(), content);
    // Publication consumes and closes the held handle, so normal later changes work.
    fs::write(&destination, b"later ordinary write").unwrap();
}

#[test]
fn invalid_size_nonregular_missing_and_early_cancel_never_admit_a_file() {
    let folder = Folder::new();
    let staged = folder.file("private.3dm", b"file");
    let empty = folder.file("empty.3dm", b"");
    let cancel = Cancellation::default();
    for (path, maximum) in [(&empty, 100), (&staged, 3), (&staged, 0), (&folder.0, 100)] {
        assert!(VerifiedFile::open(path, maximum, &cancel).is_err());
    }
    assert!(VerifiedFile::open(&folder.0.join("missing.3dm"), 100, &cancel).is_err());
    cancel.cancel();
    assert_eq!(
        VerifiedFile::open(&staged, 100, &cancel).unwrap_err().kind,
        FailureKind::Canceled
    );
    assert_eq!(fs::read(&staged).unwrap(), b"file");
}

#[test]
fn guard_drop_releases_write_delete_and_rename_denials_without_changing_bytes() {
    let folder = Folder::new();
    let staged = folder.file("private.3dm", b"file");
    let guard = VerifiedFile::open(&staged, 100, &Cancellation::default()).unwrap();
    assert!(fs::write(&staged, b"same").is_err());
    drop(guard);
    fs::write(&staged, b"same").unwrap();
    let renamed = folder.0.join("released.3dm");
    fs::rename(&staged, &renamed).unwrap();
    fs::remove_file(&renamed).unwrap();
}

#[test]
fn readonly_or_directory_target_and_cancel_keep_the_good_destination() {
    let folder = Folder::new();
    let destination = folder.file("good.3dm", b"prior");
    let original_permissions = fs::metadata(&destination).unwrap().permissions();
    let mut permissions = original_permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&destination, permissions).unwrap();
    let staged = folder.file("private.3dm", b"new");
    let cancel = Cancellation::default();
    let guard = VerifiedFile::open(&staged, 100, &cancel).unwrap();
    assert!(guard.publish(&destination, &cancel).is_err());
    assert_eq!(fs::read(&destination).unwrap(), b"prior");
    assert_eq!(fs::read(&staged).unwrap(), b"new");
    fs::set_permissions(&destination, original_permissions).unwrap();
    let guard = VerifiedFile::open(&staged, 100, &cancel).unwrap();
    cancel.cancel();
    assert_eq!(
        guard.publish(&destination, &cancel).unwrap_err().kind,
        FailureKind::Canceled
    );
    assert_eq!(fs::read(&destination).unwrap(), b"prior");
    assert_eq!(fs::read(&staged).unwrap(), b"new");
    let guard = VerifiedFile::open(&staged, 100, &Cancellation::default()).unwrap();
    assert!(guard.publish(&folder.0, &Cancellation::default()).is_err());
    assert!(folder.0.is_dir());
}

#[test]
fn existing_writer_prevents_fingerprint_admission_and_missing_target_publishes() {
    let folder = Folder::new();
    let staged = folder.file("tệp riêng (quoted 'name').3dm", b"UTF-8 names and spaces");
    let writer = fs::OpenOptions::new().write(true).open(&staged).unwrap();
    assert!(VerifiedFile::open(&staged, 100, &Cancellation::default()).is_err());
    drop(writer);
    let guard = VerifiedFile::open(&staged, 100, &Cancellation::default()).unwrap();
    let destination = folder.0.join("kết quả 'accepted'.3dm");
    guard
        .publish(&destination, &Cancellation::default())
        .unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"UTF-8 names and spaces");
    assert!(!staged.exists());
}

#[test]
fn invalid_relative_or_quoted_windows_paths_are_rejected_without_touching_target() {
    let folder = Folder::new();
    let staged = folder.file("private.3dm", b"new");
    let destination = folder.file("good.3dm", b"prior");
    let guard = VerifiedFile::open(&staged, 100, &Cancellation::default()).unwrap();
    assert!(
        guard
            .publish(&PathBuf::from("relative.3dm"), &Cancellation::default())
            .is_err()
    );
    let guard = VerifiedFile::open(&staged, 100, &Cancellation::default()).unwrap();
    assert!(
        guard
            .publish(
                &folder.0.join("invalid\"quoted.3dm"),
                &Cancellation::default()
            )
            .is_err()
    );
    assert_eq!(fs::read(&destination).unwrap(), b"prior");
    assert_eq!(fs::read(&staged).unwrap(), b"new");
}

#[test]
fn reparse_point_staged_file_and_target_are_rejected_when_symlinks_are_available() {
    let folder = Folder::new();
    let target = folder.file("link-target.3dm", b"prior");
    let link = folder.0.join("link.3dm");
    match std::os::windows::fs::symlink_file(&target, &link) {
        Ok(()) => eprintln!("verified-file reparse-point fixture created and checked"),
        Err(error) if error.raw_os_error() == Some(1314) => {
            eprintln!("verified-file reparse fixture unavailable: symlink privilege");
            return;
        }
        Err(error) => panic!("symlink fixture: {error}"),
    }
    assert!(VerifiedFile::open(&link, 100, &Cancellation::default()).is_err());
    let staged = folder.file("private.3dm", b"new");
    let guard = VerifiedFile::open(&staged, 100, &Cancellation::default()).unwrap();
    assert!(guard.publish(&link, &Cancellation::default()).is_err());
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(&target).unwrap(), b"prior");
}

#[test]
fn cancellation_during_streaming_hash_releases_the_held_file() {
    let folder = Folder::new();
    let staged = folder.file("large-private.3dm", b"initial");
    fs::OpenOptions::new()
        .write(true)
        .open(&staged)
        .unwrap()
        .set_len(64 * 1024 * 1024)
        .unwrap();
    let cancel = Cancellation::default();
    let trigger = cancel.clone();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1));
        trigger.cancel();
    });
    let result = VerifiedFile::open(&staged, 64 * 1024 * 1024, &cancel);
    thread.join().unwrap();
    assert_eq!(result.unwrap_err().kind, FailureKind::Canceled);
    fs::write(&staged, b"released after cancel").unwrap();
}
