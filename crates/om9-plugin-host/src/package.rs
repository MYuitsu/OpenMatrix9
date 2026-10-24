// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded .om9p ZIP container. Hashes detect corruption, not publisher identity.
use crate::error;
use om9_sdk::{PluginDescriptor, Result, SDK_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

pub const MAX_PACKAGE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_LIBRARY_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 128;
pub const TARGETS: &[&str] = &[
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryEntry {
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format_version: u32,
    pub plugin: PluginDescriptor,
    pub libraries: BTreeMap<String, LibraryEntry>,
}
pub struct Package {
    pub manifest: Manifest,
    files: BTreeMap<String, Vec<u8>>,
}

pub fn current_target() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok(TARGETS[0]),
        ("windows", "aarch64") => Ok(TARGETS[1]),
        ("macos", "x86_64") => Ok(TARGETS[2]),
        ("macos", "aarch64") => Ok(TARGETS[3]),
        _ => Err(error(
            "platform",
            "native OM9 SDK package host currently targets Windows and macOS",
        )),
    }
}
fn identifier(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 160
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
pub fn validate_descriptor(d: &PluginDescriptor) -> Result<()> {
    if d.sdk_version != SDK_VERSION {
        return Err(error("sdk_version", "unsupported SDK version"));
    }
    if !identifier(&d.id)
        || d.name.trim().is_empty()
        || d.name.len() > 256
        || !identifier(&d.version)
        || d.commands.is_empty()
        || d.commands.len() > 128
    {
        return Err(error("manifest", "invalid plugin descriptor"));
    }
    let mut commands = BTreeSet::new();
    for c in &d.commands {
        if !identifier(&c.id)
            || c.title.trim().is_empty()
            || c.title.len() > 256
            || !commands.insert(c.id.to_ascii_lowercase())
        {
            return Err(error("manifest", "invalid or duplicate command descriptor"));
        }
    }
    Ok(())
}
fn safe_path(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && !name.contains(['\\', ':', '\0'])
        && name.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with(['.', ' '])
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn validate_manifest(m: &Manifest) -> Result<()> {
    validate_descriptor(&m.plugin)?;
    if m.format_version != 1 || m.libraries.is_empty() || m.libraries.len() > TARGETS.len() {
        return Err(error("manifest", "invalid format version or library count"));
    }
    let mut seen = BTreeSet::new();
    for (target, lib) in &m.libraries {
        let suffix = if target.ends_with("windows-msvc") {
            ".dll"
        } else {
            ".dylib"
        };
        if !TARGETS.contains(&target.as_str())
            || !safe_path(&lib.path)
            || !lib.path.ends_with(suffix)
            || lib.path == "manifest.json"
            || !seen.insert(lib.path.to_ascii_lowercase())
            || lib.sha256.len() != 64
            || !lib
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(error("manifest", "invalid target, library path or SHA256"));
        }
    }
    Ok(())
}
pub fn read(path: &Path) -> Result<Package> {
    let file = File::open(path).map_err(|e| error("io", e))?;
    if file.metadata().map_err(|e| error("io", e))?.len() > MAX_PACKAGE_BYTES {
        return Err(error("budget", "package size"));
    }
    let mut archive = ZipArchive::new(file).map_err(|e| error("zip", e))?;
    if archive.len() > MAX_ENTRIES {
        return Err(error("budget", "package entry count"));
    }
    let mut files = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| error("zip", e))?;
        let name = entry.name().map_err(|e| error("zip", e))?.into_owned();
        if !safe_path(&name)
            || !names.insert(name.to_ascii_lowercase())
            || entry.is_dir()
            || entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(error("path", "unsafe, duplicate or non-file ZIP entry"));
        }
        let limit = if name == "manifest.json" {
            64 * 1024
        } else {
            MAX_LIBRARY_BYTES
        };
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| error("budget", "package sizes overflow"))?;
        if entry.size() > limit || total > MAX_PACKAGE_BYTES {
            return Err(error("budget", "unpacked package size"));
        }
        let mut data = Vec::new();
        entry
            .by_ref()
            .take(limit + 1)
            .read_to_end(&mut data)
            .map_err(|e| error("zip", e))?;
        if data.len() as u64 > limit || data.len() as u64 != entry.size() {
            return Err(error("budget", "ZIP entry size mismatch"));
        }
        files.insert(name, data);
    }
    let bytes = files
        .get("manifest.json")
        .ok_or_else(|| error("manifest", "manifest.json missing"))?;
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|e| error("manifest", e))?;
    validate_manifest(&manifest)?;
    for (target, entry) in &manifest.libraries {
        let data = files
            .get(&entry.path)
            .ok_or_else(|| error("manifest", "declared library missing"))?;
        if format!("{:x}", Sha256::digest(data)) != entry.sha256 {
            return Err(error("hash", "library checksum mismatch"));
        }
        validate_binary(target, data)?;
    }
    Ok(Package { manifest, files })
}

/// Basic file-format and CPU validation, before the platform loader executes code.
fn validate_binary(target: &str, data: &[u8]) -> Result<()> {
    let valid = if target.ends_with("windows-msvc") {
        let offset = data
            .get(0x3c..0x40)
            .map(|a| u32::from_le_bytes(a.try_into().unwrap()) as usize);
        data.starts_with(b"MZ")
            && offset
                .and_then(|o| {
                    o.checked_add(24)
                        .filter(|end| *end <= data.len())
                        .map(|_| o)
                })
                .is_some_and(|o| {
                    let machine = u16::from_le_bytes([data[o + 4], data[o + 5]]);
                    let flags = u16::from_le_bytes([data[o + 22], data[o + 23]]);
                    &data[o..o + 4] == b"PE\0\0"
                        && flags & 0x2000 != 0
                        && machine
                            == if target.starts_with("aarch64") {
                                0xaa64
                            } else {
                                0x8664
                            }
                })
    } else {
        data.get(..4) == Some(&[0xcf, 0xfa, 0xed, 0xfe])
            && data.get(4..8).is_some_and(|a| {
                u32::from_le_bytes(a.try_into().unwrap())
                    == if target.starts_with("aarch64") {
                        0x0100000c
                    } else {
                        0x01000007
                    }
            })
            && data
                .get(12..16)
                .is_some_and(|a| u32::from_le_bytes(a.try_into().unwrap()) == 6)
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            "binary",
            format!("library does not match target {target}"),
        ))
    }
}

/// Build one universal package from libraries already compiled for each target.
/// A new file is created; an existing package is never overwritten implicitly.
pub fn pack(
    descriptor: PluginDescriptor,
    libraries: &BTreeMap<String, PathBuf>,
    output: &Path,
) -> Result<Manifest> {
    validate_descriptor(&descriptor)?;
    let mut entries = BTreeMap::new();
    let mut files = BTreeMap::new();
    for (target, source) in libraries {
        let mut data = Vec::new();
        File::open(source)
            .map_err(|e| error("io", e))?
            .take(MAX_LIBRARY_BYTES + 1)
            .read_to_end(&mut data)
            .map_err(|e| error("io", e))?;
        if data.len() as u64 > MAX_LIBRARY_BYTES {
            return Err(error("budget", "library size"));
        }
        validate_binary(target, &data)?;
        let suffix = if target.ends_with("windows-msvc") {
            "dll"
        } else {
            "dylib"
        };
        let path = format!("native/{target}/plugin.{suffix}");
        entries.insert(
            target.clone(),
            LibraryEntry {
                path: path.clone(),
                sha256: format!("{:x}", Sha256::digest(&data)),
            },
        );
        files.insert(path, data);
    }
    let manifest = Manifest {
        format_version: 1,
        plugin: descriptor,
        libraries: entries,
    };
    validate_manifest(&manifest)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| error("json", e))?;
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| error("io", e))?;
    let attempt = (|| -> Result<()> {
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer
            .start_file("manifest.json", options)
            .map_err(|e| error("zip", e))?;
        writer
            .write_all(&manifest_bytes)
            .map_err(|e| error("io", e))?;
        for (path, data) in files {
            writer
                .start_file(path, options)
                .map_err(|e| error("zip", e))?;
            writer.write_all(&data).map_err(|e| error("io", e))?;
        }
        let file = writer.finish().map_err(|e| error("zip", e))?;
        file.sync_all().map_err(|e| error("io", e))?;
        if file.metadata().map_err(|e| error("io", e))?.len() > MAX_PACKAGE_BYTES {
            return Err(error("budget", "completed package exceeds size limit"));
        }
        Ok(())
    })();
    if attempt.is_err() {
        let _ = fs::remove_file(output);
    }
    attempt?;
    Ok(manifest)
}

pub(crate) struct ExtractedLibrary {
    pub path: PathBuf,
    directory: PathBuf,
}
impl Package {
    pub(crate) fn extract_current(&self) -> Result<ExtractedLibrary> {
        let target = current_target()?;
        let entry = self
            .manifest
            .libraries
            .get(target)
            .ok_or_else(|| error("platform", format!("package has no library for {target}")))?;
        let directory = std::env::temp_dir().join(format!("om9-plugin-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory).map_err(|e| error("io", e))?;
        let path = directory.join(if cfg!(windows) {
            "plugin.dll"
        } else {
            "plugin.dylib"
        });
        let extracted = ExtractedLibrary { path, directory };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&extracted.path)
            .map_err(|e| error("io", e))?;
        file.write_all(&self.files[&entry.path])
            .map_err(|e| error("io", e))?;
        file.sync_all().map_err(|e| error("io", e))?;
        Ok(extracted)
    }
}
impl Drop for ExtractedLibrary {
    fn drop(&mut self) {
        // Only the unique directory created by extract_current is removed.
        let _ = fs::remove_dir_all(&self.directory);
    }
}
