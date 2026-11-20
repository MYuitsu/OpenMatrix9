// SPDX-License-Identifier: LGPL-2.1-or-later
//! Application-scoped Project Manager state, independent of a drawing.
use crate::project_manager::ProjectManager;
use om9_core::{Error, Result};
use sha2::{Digest, Sha256};
use std::{io::{Read, Write}, path::{Path, PathBuf}};

const MAX_BYTES: u64 = 16 * 1024 * 1024;

fn asset_directory(path: &Path) -> Result<PathBuf> {
    let name = path.file_name().ok_or(Error::Invalid("project catalog filename"))?.to_string_lossy();
    Ok(path.with_file_name(format!("{name}.assets")))
}

fn asset_path(directory: &Path, hash: &[u8; 32]) -> PathBuf {
    use std::fmt::Write;
    let mut name = String::with_capacity(64);
    for byte in hash { let _ = write!(name, "{byte:02x}"); }
    directory.join(name)
}

fn read_asset(path: &Path, expected: &[u8; 32], length: u64, max_asset_bytes: u64) -> Result<Vec<u8>> {
    read_asset_checked(path, expected, length, max_asset_bytes, || Ok(()))
}

fn read_bounded(file: std::fs::File, length: u64, mut check: impl FnMut() -> Result<()>) -> Result<Vec<u8>> {
    let capacity = usize::try_from(length.checked_add(1).ok_or(Error::Overflow("project file bytes"))?)
        .map_err(|_| Error::Overflow("project file bytes"))?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(capacity).map_err(|_| Error::Budget("project file allocation"))?;
    let mut reader = file.take(length + 1);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check()?;
        let count = reader.read(&mut buffer).map_err(|error| Error::Io(error.to_string()))?;
        if count == 0 { break; }
        bytes.extend_from_slice(&buffer[..count]);
    }
    if bytes.len() as u64 != length { return Err(Error::Stale("project file changed while reading")); }
    Ok(bytes)
}

fn read_asset_checked(path: &Path, expected: &[u8; 32], length: u64, max_asset_bytes: u64, mut check: impl FnMut() -> Result<()>) -> Result<Vec<u8>> {
    if length == 0 || length > max_asset_bytes { return Err(Error::Budget("project asset bytes")); }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| Error::Io(error.to_string()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != length {
        return Err(Error::Invalid("project asset file"));
    }
    let file = std::fs::File::open(path).map_err(|error| Error::Io(error.to_string()))?;
    let bytes = read_bounded(file, length, &mut check)?;
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) { check()?; digest.update(chunk); }
    if <[u8; 32]>::from(digest.finalize()) != *expected {
        return Err(Error::Invalid("project asset integrity"));
    }
    Ok(bytes)
}

pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("OM9_PROJECT_CATALOG") {
        return Some(PathBuf::from(path));
    }
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("OpenMatrix9").join("project-catalog.json"))
}

pub(crate) fn root_settings_path() -> Option<PathBuf> {
    if std::env::var_os("OM9_PROJECT_CATALOG").is_some() { return None; }
    default_path().map(|path| path.with_file_name("project-root.json"))
}

pub fn root_changes_supported() -> bool { root_settings_path().is_some() }

pub fn configured_path() -> Result<Option<PathBuf>> {
    let Some(default) = default_path() else { return Ok(None); };
    let Some(settings) = root_settings_path() else { return Ok(Some(default)); };
    let file = match std::fs::File::open(&settings) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Some(default)),
        Err(error) => return Err(Error::Io(error.to_string())),
    };
    let mut bytes = Vec::new();
    file.take(32769).read_to_end(&mut bytes).map_err(|error| Error::Io(error.to_string()))?;
    if bytes.len() > 32768 { return Err(Error::Budget("project root settings")); }
    let root: PathBuf = serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("project root settings"))?;
    if !root.is_absolute() || !root.is_dir() { return Err(Error::Missing("project root folder")); }
    Ok(Some(root.join("project-catalog.json")))
}

pub fn save_root(root: &Path) -> Result<()> {
    let settings = root_settings_path().ok_or(Error::Unsupported("project catalog path is overridden by environment"))?;
    if !root.is_absolute() || !root.is_dir() { return Err(Error::Invalid("project root folder")); }
    let bytes = serde_json::to_vec(root).map_err(|_| Error::Invalid("project root path"))?;
    if bytes.len() > 32768 { return Err(Error::Budget("project root settings")); }
    if let Some(parent) = settings.parent() { std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?; }
    #[cfg(windows)]
    let _writer_lock = writer_lock(&settings)?;
    publish_bytes(&settings, &bytes)
}

#[cfg(test)]
pub fn load(path: &Path, name: &str) -> Result<Option<(ProjectManager, [u8; 32])>> {
    load_with_admission(path, name, |_| Ok(()))
}

/// Admit the manifest and progressively loaded assets before allocating them.
/// The callback receives a conservative combined peak, including the input.
pub fn load_with_admission(path: &Path, name: &str, admit: impl FnMut(u64) -> Result<()>) -> Result<Option<(ProjectManager, [u8; 32])>> {
    load_with_limits_and_admission(path, name, &om9_core::budget::Limits::default(), admit)
}

pub fn load_with_limits_and_admission(path: &Path, name: &str, limits: &om9_core::budget::Limits, mut admit: impl FnMut(u64) -> Result<()>) -> Result<Option<(ProjectManager, [u8; 32])>> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::Io(error.to_string())),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(Error::Invalid("project catalog file"));
    }
    if metadata.len() > MAX_BYTES { return Err(Error::Budget("project catalog bytes")); }
    let file = std::fs::File::open(path).map_err(|error| Error::Io(error.to_string()))?;
    let length = file.metadata().map_err(|error| Error::Io(error.to_string()))?.len();
    if length > MAX_BYTES { return Err(Error::Budget("project catalog bytes")); }
    let manifest_peak = length.checked_mul(17).and_then(|n| n.checked_add(1024 * 1024))
        .ok_or(Error::Overflow("project catalog admission"))?;
    admit(manifest_peak)?;
    let bytes = read_bounded(file, length, || admit(manifest_peak))?;
    let hash = Sha256::digest(&bytes).into();
    let manager = ProjectManager::from_json(&bytes, name)?;
    let needed = manager.required_assets()?;
    let mut total = 0u64;
    let mut assets = crate::project_manager::ProjectAssets::new();
    let directory = asset_directory(path)?;
    for (digest, length) in needed {
        total = total.checked_add(length).ok_or(Error::Overflow("project asset bytes"))?;
        if total > limits.max_bytes { return Err(Error::Budget("project asset bytes")); }
        let peak = total.checked_add(length).and_then(|n| n.checked_add(manifest_peak))
            .ok_or(Error::Overflow("project catalog asset admission"))?;
        admit(peak)?;
        assets.insert(digest, read_asset_checked(&asset_path(&directory, &digest), &digest, length, limits.max_bytes, || admit(peak))?.into());
    }
    let retained = total.checked_add(manifest_peak).ok_or(Error::Overflow("project catalog asset admission"))?;
    let manager = manager.hydrate_assets_with_check(&assets, &mut || admit(retained))?;
    Ok(Some((manager, hash)))
}

fn manifest_fingerprint(path: &Path) -> Result<Option<[u8; 32]>> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::Io(error.to_string())),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_BYTES {
        return Err(Error::Invalid("project catalog file"));
    }
    let mut file = std::fs::File::open(path).map_err(|error| Error::Io(error.to_string()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 16384];
    let mut total = 0u64;
    loop {
        let read = file.read(&mut buffer).map_err(|error| Error::Io(error.to_string()))?;
        if read == 0 { break; }
        total = total.checked_add(read as u64).ok_or(Error::Overflow("project catalog bytes"))?;
        if total > MAX_BYTES { return Err(Error::Budget("project catalog bytes")); }
        digest.update(&buffer[..read]);
    }
    Ok(Some(digest.finalize().into()))
}

/// Retain the source catalog through a root change so another cooperating OM9
/// writer cannot make the prepared copy stale before it is published.
pub(crate) struct SourceCatalogGuard {
    #[cfg(windows)]
    _writer_lock: std::fs::File,
    _file: Option<std::fs::File>,
    path: PathBuf,
}

impl SourceCatalogGuard {
    pub(crate) fn verify_absent_source(&self) -> Result<()> {
        if self._file.is_some() { return Ok(()); }
        match std::fs::symlink_metadata(&self.path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(_) => Err(Error::Stale("project catalog appeared during root change")),
            Err(error) => Err(Error::Io(error.to_string())),
        }
    }
}

pub(crate) fn guard_source_catalog(path: &Path, expected: Option<[u8; 32]>) -> Result<SourceCatalogGuard> {
    #[cfg(windows)]
    let writer = {
        let parent = path.parent().ok_or(Error::Invalid("project catalog parent"))?;
        if !parent.is_dir() {
            if expected.is_some() { return Err(Error::Stale("project catalog changed on disk")); }
            std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
        }
        writer_lock(path)?
    };
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(Error::Io(error.to_string())),
    };
    let file = match metadata {
        Some(metadata) => Some(open_project_for_import(path, metadata.len())?),
        None => None,
    };
    if manifest_fingerprint(path)? != expected { return Err(Error::Stale("project catalog changed on disk")); }
    Ok(SourceCatalogGuard {
        #[cfg(windows)]
        _writer_lock: writer,
        _file: file,
        path: path.to_path_buf(),
    })
}

#[cfg(feature = "test-fixtures")]
pub fn test_guard_source_catalog(path: &Path, expected: Option<[u8; 32]>) -> Result<impl Sized> {
    guard_source_catalog(path, expected)
}

pub(crate) fn root_settings_fingerprint(path: &Path) -> Result<Option<[u8; 32]>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.len() > 32768 => return Err(Error::Budget("project root settings")),
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(Error::Io(error.to_string())),
        _ => {}
    }
    manifest_fingerprint(path)
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); }
}

/// A stable lock file remains in the workspace; the OS releases the exclusive
/// handle on process exit, including a crash. It serializes cooperating OM9
/// writers across the fingerprint check and manifest publication.
#[cfg(windows)]
fn writer_lock(path: &Path) -> Result<std::fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    let name = path.file_name().ok_or(Error::Invalid("project catalog filename"))?.to_string_lossy();
    let lock = path.with_file_name(format!("{name}.lock"));
    if let Ok(metadata) = std::fs::symlink_metadata(&lock) {
        if !metadata.is_file() || metadata.file_type().is_symlink() { return Err(Error::Invalid("project catalog lock")); }
    }
    std::fs::OpenOptions::new().read(true).write(true).create(true).share_mode(0).open(lock)
        .map_err(|error| if matches!(error.raw_os_error(), Some(32 | 33)) { Error::Busy } else { Error::Io(error.to_string()) })
}

pub fn save(path: &Path, manager: &ProjectManager, expected: Option<[u8; 32]>) -> Result<[u8; 32]> {
    save_with_limits(path, manager, expected, &om9_core::budget::Limits::default())
}

pub fn save_with_limits(path: &Path, manager: &ProjectManager, expected: Option<[u8; 32]>, limits: &om9_core::budget::Limits) -> Result<[u8; 32]> {
    let (bytes, assets) = manager.externalized()?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
    #[cfg(windows)]
    let _writer_lock = writer_lock(path)?;
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.permissions().readonly() {
            return Err(Error::Permission("project catalog destination"));
        }
    }
    let actual = manifest_fingerprint(path)?;
    if actual != expected { return Err(Error::Stale("project catalog changed on disk")); }
    let directory = asset_directory(path)?;
    if !assets.is_empty() { std::fs::create_dir_all(&directory).map_err(|error| Error::Io(error.to_string()))?; }
    let mut total = 0u64;
    for (hash, payload) in &assets {
        total = total.checked_add(payload.len() as u64).ok_or(Error::Overflow("project asset bytes"))?;
        if total > limits.max_bytes { return Err(Error::Budget("project asset bytes")); }
        let target = asset_path(&directory, hash);
        match std::fs::symlink_metadata(&target) {
            Ok(_) => { read_asset(&target, hash, payload.len() as u64, limits.max_bytes)?; }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => publish_bytes(&target, payload)?,
            Err(error) => return Err(Error::Io(error.to_string())),
        }
    }
    publish_bytes(path, &bytes)?;
    Ok(Sha256::digest(&bytes).into())
}

pub struct PreparedCatalogRoot {
    #[cfg(windows)]
    _writer_lock: std::fs::File,
    #[cfg(windows)]
    _settings_lock: std::fs::File,
    root: PathBuf,
    target: PathBuf,
    settings_path: PathBuf,
    settings_before: Option<[u8; 32]>,
    fingerprint: [u8; 32],
    // On Windows these handles deny writes and replacement until publication.
    _existing_assets: Vec<std::fs::File>,
    assets: Vec<PreparedProjectFile>,
    catalog: RootCatalog,
    settings: PreparedProjectFile,
}

enum RootCatalog {
    New(PreparedProjectFile),
    #[cfg(windows)]
    Existing(std::fs::File),
}

impl PreparedCatalogRoot {
    pub fn publish(self, cancel: &om9_core::transaction::Cancellation) -> Result<(PathBuf, [u8; 32])> {
        self.publish_with_hook(cancel, || {})
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_publish_with_hook(self, cancel: &om9_core::transaction::Cancellation, hook: impl FnOnce()) -> Result<(PathBuf, [u8; 32])> {
        self.publish_with_hook(cancel, hook)
    }

    fn publish_with_hook(self, cancel: &om9_core::transaction::Cancellation, after_catalog: impl FnOnce()) -> Result<(PathBuf, [u8; 32])> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if matches!(&self.catalog, RootCatalog::New(_)) {
            match std::fs::symlink_metadata(&self.target) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                Ok(_) => return Err(Error::Stale("project catalog destination appeared")),
                Err(error) => return Err(Error::Io(error.to_string())),
            }
        }
        if root_settings_fingerprint(&self.settings_path)? != self.settings_before {
            return Err(Error::Stale("project root settings changed"));
        }
        for asset in self.assets { asset.publish_new(cancel)?; }
        #[cfg(windows)]
        let (created, _existing_catalog) = match self.catalog {
            RootCatalog::New(catalog) => (Some(catalog.publish_new_retained(cancel)?), None),
            RootCatalog::Existing(file) => (None, Some(file)),
        };
        #[cfg(not(windows))]
        let RootCatalog::New(catalog) = self.catalog;
        #[cfg(not(windows))]
        catalog.publish_new(cancel)?;
        after_catalog();
        let setting_result = match root_settings_fingerprint(&self.settings_path) {
            Ok(actual) if actual == self.settings_before => {
                if self.settings_before.is_some() { self.settings.publish(cancel) }
                else { self.settings.publish_new(cancel) }
            }
            Ok(_) => Err(Error::Stale("project root settings changed during publication")),
            Err(error) => Err(error),
        };
        if let Err(error) = setting_result {
            #[cfg(windows)]
            {
                return match created {
                    Some(created) => match created.rollback() {
                        Ok(()) => Err(Error::Io(format!("root setting was not changed; new catalog was rolled back: {error}"))),
                        Err(rollback) => Err(Error::Io(format!("catalog remains at {}; root setting was not changed: {error}; rollback failed: {rollback:?}", self.target.display()))),
                    },
                    None => Err(Error::Io(format!("existing catalog was kept; root setting was not changed: {error}"))),
                };
            }
            #[cfg(not(windows))]
            return Err(Error::Io(format!("catalog created at {}; root setting was not changed: {error}", self.target.display())));
        }
        #[cfg(windows)]
        drop(created);
        Ok((self.root.join("project-catalog.json"), self.fingerprint))
    }
}

/// Prepare a new root without replacing its manifest or the active root
/// setting. New content-addressed assets remain temporary until publication;
/// existing files are never removed.
pub(crate) fn prepare_catalog_root(root: &Path, settings: &Path, manager: &ProjectManager,
    limits: &om9_core::budget::Limits, cancel: &om9_core::transaction::Cancellation,
    settings_before: Option<[u8; 32]>) -> Result<PreparedCatalogRoot> {
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if !root.is_absolute() || !root.is_dir() { return Err(Error::Invalid("project root folder")); }
    let target = root.join("project-catalog.json");
    #[cfg(windows)]
    let catalog_lock = writer_lock(&target)?;
    let (manifest, assets) = manager.externalized()?;
    let fingerprint: [u8; 32] = Sha256::digest(&manifest).into();
    let existing: Option<std::fs::File> = match std::fs::symlink_metadata(&target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(Error::Io(error.to_string())),
        Ok(metadata) => {
            #[cfg(windows)]
            {
                let file = open_project_for_import(&target, metadata.len())?;
                if manifest_fingerprint(&target)? != Some(fingerprint) { return Err(Error::Stale("project catalog destination differs")); }
                Some(file)
            }
            #[cfg(not(windows))]
            { let _ = metadata; return Err(Error::Stale("project catalog destination exists")); }
        }
    };
    let directory = asset_directory(&target)?;
    if !assets.is_empty() {
        match std::fs::symlink_metadata(&directory) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {},
            Ok(_) => return Err(Error::Invalid("project asset directory")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if existing.is_some() { return Err(Error::Missing("existing project catalog assets")); }
                std::fs::create_dir(&directory).map_err(|error| Error::Io(error.to_string()))?
            }
            Err(error) => return Err(Error::Io(error.to_string())),
        }
    }
    let mut total = 0u64;
    let mut prepared_assets = Vec::new();
    let mut existing_assets = Vec::new();
    prepared_assets.try_reserve(assets.len()).map_err(|_| Error::Budget("project asset staging"))?;
    existing_assets.try_reserve(assets.len()).map_err(|_| Error::Budget("project asset owners"))?;
    for (hash, payload) in &assets {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        total = total.checked_add(payload.len() as u64).ok_or(Error::Overflow("project asset bytes"))?;
        if total > limits.max_bytes { return Err(Error::Budget("project asset bytes")); }
        let path = asset_path(&directory, hash);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != payload.len() as u64 {
                    return Err(Error::Invalid("project asset file"));
                }
                let mut file = open_project_for_import(&path, payload.len() as u64)?;
                let mut digest = Sha256::new();
                let mut buffer = [0u8; 64 * 1024];
                let mut length = 0u64;
                loop {
                    if cancel.is_canceled() { return Err(Error::Canceled); }
                    let count = file.read(&mut buffer).map_err(|error| Error::Io(error.to_string()))?;
                    if count == 0 { break; }
                    length = length.checked_add(count as u64).ok_or(Error::Overflow("project asset bytes"))?;
                    if length > payload.len() as u64 { return Err(Error::Stale("project asset changed")); }
                    digest.update(&buffer[..count]);
                }
                if length != payload.len() as u64 || <[u8; 32]>::from(digest.finalize()) != *hash {
                    return Err(Error::Invalid("project asset integrity"));
                }
                existing_assets.push(file);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if existing.is_some() { return Err(Error::Missing("existing project catalog asset")); }
                prepared_assets.push(prepare_bytes(&path, payload, cancel)?);
            }
            Err(error) => return Err(Error::Io(error.to_string())),
        }
    }
    let catalog = match existing {
        #[cfg(windows)]
        Some(file) => RootCatalog::Existing(file),
        None => RootCatalog::New(prepare_bytes(&target, &manifest, cancel)?),
        #[cfg(not(windows))]
        Some(_) => unreachable!(),
    };
    let settings_bytes = serde_json::to_vec(root).map_err(|_| Error::Invalid("project root path"))?;
    if settings_bytes.len() > 32768 { return Err(Error::Budget("project root settings")); }
    if let Some(parent) = settings.parent() { std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?; }
    #[cfg(windows)]
    let settings_lock = writer_lock(settings)?;
    if root_settings_fingerprint(settings)? != settings_before { return Err(Error::Stale("project root settings changed")); }
    let settings_path = settings.to_path_buf();
    let settings = prepare_bytes(settings, &settings_bytes, cancel)?;
    Ok(PreparedCatalogRoot {
        #[cfg(windows)] _writer_lock: catalog_lock,
        #[cfg(windows)] _settings_lock: settings_lock,
        root: root.to_path_buf(), target, settings_path, settings_before,
        fingerprint, _existing_assets: existing_assets,
        assets: prepared_assets, catalog, settings,
    })
}

#[cfg(feature = "test-fixtures")]
pub fn test_prepare_catalog_root(root: &Path, settings: &Path, manager: &ProjectManager,
    limits: &om9_core::budget::Limits, cancel: &om9_core::transaction::Cancellation) -> Result<PreparedCatalogRoot> {
    prepare_catalog_root(root, settings, manager, limits, cancel, root_settings_fingerprint(settings)?)
}

/// Publish already encoded data at an explicitly selected destination.
pub fn publish_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let cancel = om9_core::transaction::Cancellation::default();
    prepare_bytes(path, bytes, &cancel)?.publish(&cancel)
}

pub struct PreparedProjectFile {
    // Close the verified Windows handle before Staging tries to remove a
    // canceled or rejected temporary file.
    #[cfg(windows)]
    held: om9_native_owner::VerifiedFile,
    destination: PathBuf,
    staging: Staging,
}

impl PreparedProjectFile {
    pub fn publish(self, cancel: &om9_core::transaction::Cancellation) -> Result<()> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        #[cfg(windows)]
        self.held.publish(&self.destination, cancel).map_err(|error| Error::Io(format!("catalog publication: {error:?}")))?;
        #[cfg(not(windows))]
        std::fs::rename(&self.staging.0, &self.destination).map_err(|error| Error::Io(error.to_string()))?;
        Ok(())
    }

    pub fn publish_new(self, cancel: &om9_core::transaction::Cancellation) -> Result<()> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        #[cfg(windows)]
        self.held.publish_new(&self.destination, cancel).map_err(|error| Error::Io(format!("catalog publication: {error:?}")))?;
        #[cfg(not(windows))]
        std::fs::hard_link(&self.staging.0, &self.destination).map_err(|error| Error::Io(error.to_string()))?;
        Ok(())
    }

    #[cfg(windows)]
    fn publish_new_retained(self, cancel: &om9_core::transaction::Cancellation) -> Result<om9_native_owner::PublishedNewFile> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        self.held.publish_new_retained(&self.destination, cancel)
            .map_err(|error| Error::Io(format!("catalog publication: {error:?}")))
    }
}

pub fn prepare_bytes(path: &Path, bytes: &[u8], cancel: &om9_core::transaction::Cancellation) -> Result<PreparedProjectFile> {
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() { return Err(Error::Missing("project export directory")); }
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.permissions().readonly() {
            return Err(Error::Permission("project export destination"));
        }
    }
    let staging = Staging(parent.join(format!(".om9-project-catalog-{}.tmp", uuid::Uuid::new_v4())));
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(&staging.0)
        .map_err(|error| Error::Io(error.to_string()))?;
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        file.write_all(chunk).map_err(|error| Error::Io(error.to_string()))?;
        digest.update(chunk);
    }
    file.sync_all().map_err(|error| Error::Io(error.to_string()))?;
    drop(file);
    #[cfg(windows)]
    {
        let source = std::path::absolute(&staging.0).map_err(|error| Error::Io(error.to_string()))?;
        let destination = std::path::absolute(path).map_err(|error| Error::Io(error.to_string()))?;
        let held = om9_native_owner::VerifiedFile::open(&source, bytes.len() as u64, cancel)
            .map_err(|error| Error::Io(format!("catalog staging: {error:?}")))?;
        let expected: [u8; 32] = digest.finalize().into();
        if held.len() != bytes.len() as u64 || held.fingerprint() != expected {
            return Err(Error::Invalid("project catalog staging fingerprint"));
        }
        return Ok(PreparedProjectFile { staging, destination, held });
    }
    #[cfg(not(windows))]
    { let _ = digest; Ok(PreparedProjectFile { staging, destination: path.to_path_buf() }) }
}

const PROJECT_MAGIC_V1: &[u8; 8] = b"OM9PROJ1";
const PROJECT_MAGIC_V2: &[u8; 8] = b"OM9PROJ2";
fn max_project_file_bytes(limits: &om9_core::budget::Limits) -> u64 {
    limits.max_bytes.saturating_add(MAX_BYTES + 4 * 1024 * 1024)
}

pub fn encode_project(manager: &ProjectManager, id: crate::project_manager::ProjectId) -> Result<Vec<u8>> {
    encode_project_with_limits(manager, id, &om9_core::budget::Limits::default())
}

pub fn encode_project_with_limits(manager: &ProjectManager, id: crate::project_manager::ProjectId, limits: &om9_core::budget::Limits) -> Result<Vec<u8>> {
    encode_project_with_check(manager, id, limits, || Ok(()))
}

/// Prepare a row archive while allowing cancellation between bounded chunks.
pub fn encode_project_with_check(manager: &ProjectManager, id: crate::project_manager::ProjectId,
    limits: &om9_core::budget::Limits, check: impl Fn() -> Result<()>) -> Result<Vec<u8>> {
    let (payload, assets) = manager.export_project_bundle_with_check(id, &check)?;
    check()?;
    if assets.len() > 100_000 { return Err(Error::Budget("project archive chunks")); }
    let total_assets = assets.values().try_fold(0u64, |total, asset| total.checked_add(asset.len() as u64).ok_or(Error::Overflow("project asset bytes")))?;
    if total_assets > limits.max_bytes { return Err(Error::Budget("project asset bytes")); }
    let size = assets.values().try_fold(56usize.checked_add(payload.len()).ok_or(Error::Overflow("project archive size"))?, |total, asset| {
        total.checked_add(40).and_then(|n| n.checked_add(asset.len())).ok_or(Error::Overflow("project archive size"))
    })?;
    if size as u64 > max_project_file_bytes(limits) { return Err(Error::Budget("project archive bytes")); }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(size)
        .map_err(|_| Error::Budget("project archive bytes"))?;
    bytes.extend_from_slice(PROJECT_MAGIC_V2);
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(assets.len() as u64).to_le_bytes());
    let mut digest = Sha256::new();
    for chunk in payload.chunks(1024 * 1024) { check()?; digest.update(chunk); }
    bytes.extend_from_slice(&digest.finalize());
    for chunk in payload.chunks(1024 * 1024) { check()?; bytes.extend_from_slice(chunk); }
    for (hash, asset) in assets {
        check()?;
        bytes.extend_from_slice(&hash);
        bytes.extend_from_slice(&(asset.len() as u64).to_le_bytes());
        for chunk in asset.chunks(1024 * 1024) { check()?; bytes.extend_from_slice(chunk); }
    }
    check()?;
    Ok(bytes)
}

#[cfg(test)]
fn read_project(path: &Path) -> Result<(Vec<u8>, crate::project_manager::ProjectAssets)> {
    let file = std::fs::File::open(path).map_err(|error| Error::Io(error.to_string()))?;
    read_project_file(file, &om9_core::transaction::Cancellation::default())
}

/// Open the selected file for reading while denying concurrent Windows
/// write/delete sharing; unlike a publication handle this needs no DELETE right.
pub fn open_project_for_import(path: &Path, expected_len: u64) -> Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.share_mode(FILE_SHARE_READ).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path).map_err(|error| Error::Io(error.to_string()))?;
    let metadata = file.metadata().map_err(|error| Error::Io(error.to_string()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != expected_len {
        return Err(Error::Stale("project import file changed"));
    }
    Ok(file)
}

pub fn read_project_file(file: std::fs::File, cancel: &om9_core::transaction::Cancellation) -> Result<(Vec<u8>, crate::project_manager::ProjectAssets)> {
    read_project_file_with_limits(file, cancel, &om9_core::budget::Limits::default())
}

pub fn read_project_file_with_limits(file: std::fs::File, cancel: &om9_core::transaction::Cancellation, limits: &om9_core::budget::Limits) -> Result<(Vec<u8>, crate::project_manager::ProjectAssets)> {
    let length = file.metadata().map_err(|error| Error::Io(error.to_string()))?.len();
    if length < 48 || length > max_project_file_bytes(limits) {
        return Err(Error::Invalid("project archive header"));
    }
    let mut bytes = read_bounded(file, length, || if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) })?;
    let hash_checked = |input: &[u8]| -> Result<[u8; 32]> {
        let mut digest = Sha256::new();
        for chunk in input.chunks(64 * 1024) {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            digest.update(chunk);
        }
        Ok(digest.finalize().into())
    };
    let length = u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| Error::Invalid("project archive length"))?);
    if &bytes[..8] == PROJECT_MAGIC_V1 {
        if length > MAX_BYTES || length != (bytes.len() - 48) as u64 || hash_checked(&bytes[48..])?.as_slice() != &bytes[16..48] {
            return Err(Error::Invalid("project archive integrity"));
        }
        return Ok((bytes.split_off(48), Default::default()));
    }
    if &bytes[..8] != PROJECT_MAGIC_V2 || bytes.len() < 56 || length > MAX_BYTES {
        return Err(Error::Invalid("project archive header"));
    }
    let count = u64::from_le_bytes(bytes[16..24].try_into().map_err(|_| Error::Invalid("project archive count"))?);
    if count > 100_000 { return Err(Error::Budget("project archive chunks")); }
    let end = 56usize.checked_add(usize::try_from(length).map_err(|_| Error::Overflow("project archive length"))?)
        .filter(|end| *end <= bytes.len()).ok_or(Error::Invalid("project archive length"))?;
    if hash_checked(&bytes[56..end])?.as_slice() != &bytes[24..56] {
        return Err(Error::Invalid("project archive integrity"));
    }
    let manifest = bytes[56..end].to_vec();
    let mut assets = crate::project_manager::ProjectAssets::new();
    let mut cursor = end;
    let mut total = 0u64;
    for _ in 0..count {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let header = cursor.checked_add(40).filter(|end| *end <= bytes.len()).ok_or(Error::Invalid("project asset header"))?;
        let hash: [u8; 32] = bytes[cursor..cursor + 32].try_into().map_err(|_| Error::Invalid("project asset hash"))?;
        let size = u64::from_le_bytes(bytes[cursor + 32..header].try_into().map_err(|_| Error::Invalid("project asset size"))?);
        total = total.checked_add(size).ok_or(Error::Overflow("project asset bytes"))?;
        if size == 0 || total > limits.max_bytes { return Err(Error::Budget("project asset bytes")); }
        let end = header.checked_add(usize::try_from(size).map_err(|_| Error::Overflow("project asset size"))?)
            .filter(|end| *end <= bytes.len()).ok_or(Error::Invalid("project asset size"))?;
        if hash_checked(&bytes[header..end])? != hash || assets.insert(hash, std::sync::Arc::from(&bytes[header..end])).is_some() {
            return Err(Error::Invalid("project asset integrity"));
        }
        cursor = end;
    }
    if cursor != bytes.len() { return Err(Error::Invalid("project archive trailing bytes")); }
    Ok((manifest, assets))
}

#[cfg(feature = "test-fixtures")]
pub fn import_project_for_test(
    manager: &mut ProjectManager,
    manifest: &[u8],
    assets: &std::collections::BTreeMap<[u8; 32], std::sync::Arc<[u8]>>,
    limits: &om9_core::budget::Limits,
    cancel: &om9_core::transaction::Cancellation,
) -> Result<crate::project_manager::ProjectId> {
    manager.import_project_bundle_with_cancel(manifest, assets, limits, cancel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_refuse_stale_writer() {
        let path = std::env::temp_dir().join(format!("om9-catalog-{}.json", uuid::Uuid::new_v4()));
        let mut manager = ProjectManager::new("Initial");
        let first = save(&path, &manager, None).unwrap();
        manager.rename_project(manager.active().id, "Renamed").unwrap();
        let second = save(&path, &manager, Some(first)).unwrap();
        assert_ne!(first, second);
        assert!(matches!(save(&path, &manager, Some(first)), Err(Error::Stale(_))));
        assert!(matches!(load_with_admission(&path, "unused", |_| Err(Error::Budget("catalog test refusal"))), Err(Error::Budget(_))));
        let mut checks = 0;
        assert!(matches!(load_with_admission(&path, "unused", |_| {
            checks += 1;
            if checks > 1 { Err(Error::Canceled) } else { Ok(()) }
        }), Err(Error::Canceled)));
        let (loaded, hash) = load(&path, "unused").unwrap().unwrap();
        assert_eq!(loaded.active().name, "Renamed");
        assert_eq!(hash, second);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn project_archive_checks_manifest_and_remaps_imported_row() {
        let path = std::env::temp_dir().join(format!("om9-project-{}.om9project", uuid::Uuid::new_v4()));
        let manager = ProjectManager::new("Ring");
        let mut bytes = encode_project(&manager, manager.active().id).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let cancel = om9_core::transaction::Cancellation::default();
        cancel.cancel();
        assert!(matches!(read_project_file(std::fs::File::open(&path).unwrap(), &cancel), Err(Error::Canceled)));
        let (manifest, assets) = read_project(&path).unwrap();
        let mut destination = ProjectManager::new("Existing");
        let imported = destination.import_project_bundle(&manifest, &assets, &om9_core::budget::Limits::default()).unwrap();
        assert_ne!(imported, destination.rows()[0].id);
        assert_eq!(destination.active().name, "Ring");
        *bytes.last_mut().unwrap() ^= 1;
        std::fs::write(&path, &bytes).unwrap();
        assert!(read_project(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
