// SPDX-License-Identifier: LGPL-2.1-or-later
//! Package admission tests use synthetic PE/Mach-O headers, never executable
//! plugins. Passing these structural checks makes no native/runtime claim.

use om9_plugin_host::package::{self, LibraryEntry, Manifest};
use om9_sdk::{CommandDescriptor, PluginDescriptor, Result, SDK_VERSION};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const WINDOWS_X64: &str = "x86_64-pc-windows-msvc";
const WINDOWS_ARM64: &str = "aarch64-pc-windows-msvc";
const MAC_ARM64: &str = "aarch64-apple-darwin";

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "om9-package-structural-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // This is the exact unique root created by Scratch::new; no input or
        // computed external path is removed by these tests.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: "org.openmatrix9.structural-fixture".into(),
        name: "Structural package fixture".into(),
        version: "0.1.0".into(),
        sdk_version: SDK_VERSION,
        commands: vec![CommandDescriptor {
            id: "FixtureCommand".into(),
            title: "Fixture command".into(),
        }],
    }
}

/// Only a DOS/COFF signature, machine and DLL flag. Not a loadable library.
fn structural_pe(machine: u16) -> Vec<u8> {
    let pe_offset = 0x80usize;
    let mut bytes = vec![0; pe_offset + 24];
    bytes[..2].copy_from_slice(b"MZ");
    bytes[0x3c..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
    bytes[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
    bytes[pe_offset + 4..pe_offset + 6].copy_from_slice(&machine.to_le_bytes());
    bytes[pe_offset + 22..pe_offset + 24].copy_from_slice(&0x2000u16.to_le_bytes());
    bytes
}

/// Only a 64-bit little-endian Mach-O header, CPU and MH_DYLIB file type.
fn structural_macho(cpu: u32) -> Vec<u8> {
    let mut bytes = vec![0; 32];
    bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
    bytes[4..8].copy_from_slice(&cpu.to_le_bytes());
    bytes[12..16].copy_from_slice(&6u32.to_le_bytes());
    bytes
}

fn manifest(target: &str, path: &str, bytes: &[u8]) -> Manifest {
    Manifest {
        format_version: 1,
        plugin: descriptor(),
        libraries: BTreeMap::from([(
            target.into(),
            LibraryEntry {
                path: path.into(),
                sha256: format!("{:x}", Sha256::digest(bytes)),
            },
        )]),
    }
}

fn write_zip(path: &Path, entries: &[(String, Vec<u8>)]) {
    let mut writer = ZipWriter::new(File::create(path).unwrap());
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in entries {
        writer.start_file(name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap();
}

fn assert_error<T>(result: Result<T>, code: &str) {
    match result {
        Ok(_) => panic!("expected package rejection with {code}"),
        Err(error) => assert_eq!(error.code, code, "unexpected rejection: {error}"),
    }
}

#[test]
fn pack_and_read_preserve_descriptor_payload_and_measured_digest() {
    let scratch = Scratch::new();
    let bytes = structural_pe(0x8664);
    let library = scratch.file("fixture.dll", &bytes);
    let output = scratch.path("fixture.om9p");
    let packed = package::pack(
        descriptor(),
        &BTreeMap::from([(WINDOWS_X64.into(), library)]),
        &output,
    )
    .unwrap();
    let admitted = package::read(&output).unwrap();

    assert_eq!(admitted.manifest.plugin, descriptor());
    assert_eq!(admitted.manifest.format_version, 1);
    let entry = &admitted.manifest.libraries[WINDOWS_X64];
    assert_eq!(entry.sha256, format!("{:x}", Sha256::digest(&bytes)));
    assert_eq!(entry.sha256, packed.libraries[WINDOWS_X64].sha256);
    let mut archive = ZipArchive::new(File::open(&output).unwrap()).unwrap();
    let mut payload = Vec::new();
    archive
        .by_name(&entry.path)
        .unwrap()
        .read_to_end(&mut payload)
        .unwrap();
    assert_eq!(payload, bytes);
    assert_eq!(archive.len(), 2);
}

#[test]
fn one_package_admits_distinct_structural_windows_and_mac_targets() {
    let scratch = Scratch::new();
    let windows = scratch.file("fixture.dll", &structural_pe(0x8664));
    let mac = scratch.file("fixture.dylib", &structural_macho(0x0100_000c));
    let output = scratch.path("multi-target.om9p");
    package::pack(
        descriptor(),
        &BTreeMap::from([(WINDOWS_X64.into(), windows), (MAC_ARM64.into(), mac)]),
        &output,
    )
    .unwrap();
    let admitted = package::read(&output).unwrap();
    assert_eq!(admitted.manifest.libraries.len(), 2);
    assert!(
        admitted.manifest.libraries[WINDOWS_X64]
            .path
            .ends_with(".dll")
    );
    assert!(
        admitted.manifest.libraries[MAC_ARM64]
            .path
            .ends_with(".dylib")
    );
    assert_ne!(
        admitted.manifest.libraries[WINDOWS_X64].path,
        admitted.manifest.libraries[MAC_ARM64].path
    );
    // No LoadedPlugin::open call: synthetic headers prove packaging only.
}

#[test]
fn a_changed_library_is_refused_even_when_its_target_header_is_valid() {
    let scratch = Scratch::new();
    let bytes = structural_pe(0x8664);
    let path = "native/windows/plugin.dll";
    let metadata = manifest(WINDOWS_X64, path, &bytes);
    let mut changed = bytes;
    changed[0x50] = 7; // Outside the structural admission fields.
    let output = scratch.path("changed.om9p");
    write_zip(
        &output,
        &[
            (
                "manifest.json".into(),
                serde_json::to_vec(&metadata).unwrap(),
            ),
            (path.into(), changed),
        ],
    );
    assert_error(package::read(&output), "hash");
}

#[test]
fn traversal_absolute_and_windows_path_entries_are_refused() {
    let scratch = Scratch::new();
    for (index, name) in [
        "../escape.dll",
        "/absolute.dll",
        "native/../../escape.dll",
        "C:/escape.dll",
        "native\\plugin.dll",
    ]
    .iter()
    .enumerate()
    {
        let output = scratch.path(&format!("unsafe-{index}.om9p"));
        write_zip(&output, &[(name.to_string(), vec![0])]);
        assert_error(package::read(&output), "path");
    }
}

#[test]
fn case_colliding_zip_entries_are_refused_before_manifest_use() {
    let scratch = Scratch::new();
    let output = scratch.path("duplicate.om9p");
    write_zip(
        &output,
        &[
            ("native/Plugin.dll".into(), vec![1]),
            ("native/plugin.dll".into(), vec![2]),
        ],
    );
    assert_error(package::read(&output), "path");
}

#[test]
fn missing_declared_library_is_refused() {
    let scratch = Scratch::new();
    let metadata = manifest(WINDOWS_X64, "native/missing.dll", &structural_pe(0x8664));
    let output = scratch.path("missing.om9p");
    write_zip(
        &output,
        &[(
            "manifest.json".into(),
            serde_json::to_vec(&metadata).unwrap(),
        )],
    );
    assert_error(package::read(&output), "manifest");
}

#[test]
fn outer_package_size_is_checked_before_zip_parsing() {
    let scratch = Scratch::new();
    let output = scratch.path("oversized.om9p");
    let file = File::create(&output).unwrap();
    // Extend only this temporary fixture; no large byte allocation is needed.
    file.set_len(package::MAX_PACKAGE_BYTES + 1).unwrap();
    drop(file);
    assert_error(package::read(&output), "budget");
}

#[test]
fn oversized_manifest_and_excessive_entry_count_are_refused() {
    let scratch = Scratch::new();
    let manifest_output = scratch.path("oversized-manifest.om9p");
    write_zip(
        &manifest_output,
        &[("manifest.json".into(), vec![b' '; 64 * 1024 + 1])],
    );
    assert_error(package::read(&manifest_output), "budget");

    let entries_output = scratch.path("too-many-entries.om9p");
    let entries: Vec<_> = (0..=package::MAX_ENTRIES)
        .map(|index| (format!("entry-{index}.bin"), vec![]))
        .collect();
    write_zip(&entries_output, &entries);
    assert_error(package::read(&entries_output), "budget");
}

#[test]
fn declared_oversized_library_is_rejected_before_payload_allocation() {
    let scratch = Scratch::new();
    let output = scratch.path("oversized-library.om9p");
    write_zip(&output, &[("native/plugin.dll".into(), vec![0])]);
    let mut bytes = fs::read(&output).unwrap();
    // Inflate only the central-directory uncompressed-size field of a tiny ZIP.
    // This models an adversarial size claim without writing/allocating 64 MiB.
    let directory = bytes
        .windows(4)
        .position(|window| window == b"PK\x01\x02")
        .unwrap();
    bytes[directory + 24..directory + 28]
        .copy_from_slice(&((package::MAX_LIBRARY_BYTES + 1) as u32).to_le_bytes());
    fs::write(&output, bytes).unwrap();
    assert_error(package::read(&output), "budget");
}

#[test]
fn unsupported_target_is_rejected_in_both_pack_and_read() {
    let scratch = Scratch::new();
    let bytes = structural_macho(0x0100_0007);
    let library = scratch.file("fixture.dylib", &bytes);
    let packed_output = scratch.path("unsupported-pack.om9p");
    assert_error(
        package::pack(
            descriptor(),
            &BTreeMap::from([("x86_64-unknown-linux-gnu".into(), library)]),
            &packed_output,
        ),
        "manifest",
    );
    assert!(!packed_output.exists());

    let path = "native/plugin.dylib";
    let metadata = manifest("x86_64-unknown-linux-gnu", path, &bytes);
    let read_output = scratch.path("unsupported-read.om9p");
    write_zip(
        &read_output,
        &[
            (
                "manifest.json".into(),
                serde_json::to_vec(&metadata).unwrap(),
            ),
            (path.into(), bytes),
        ],
    );
    assert_error(package::read(&read_output), "manifest");
}

#[test]
fn architecture_and_binary_format_must_match_the_manifest_target() {
    let scratch = Scratch::new();
    let cases = [
        (WINDOWS_ARM64, structural_pe(0x8664), "dll"),
        (WINDOWS_X64, structural_pe(0xaa64), "dll"),
        (MAC_ARM64, structural_macho(0x0100_0007), "dylib"),
        (MAC_ARM64, structural_pe(0x8664), "dylib"),
        (WINDOWS_X64, vec![0; 4], "dll"),
    ];
    for (index, (target, bytes, extension)) in cases.iter().enumerate() {
        let library = scratch.file(&format!("wrong-{index}.{extension}"), bytes);
        let output = scratch.path(&format!("wrong-{index}.om9p"));
        assert_error(
            package::pack(
                descriptor(),
                &BTreeMap::from([((*target).into(), library)]),
                &output,
            ),
            "binary",
        );
        assert!(!output.exists());
    }
}

#[test]
fn pack_never_overwrites_an_existing_output() {
    let scratch = Scratch::new();
    let original = b"existing user output";
    let output = scratch.file("keep.om9p", original);
    let library = scratch.file("fixture.dll", &structural_pe(0x8664));
    assert_error(
        package::pack(
            descriptor(),
            &BTreeMap::from([(WINDOWS_X64.into(), library)]),
            &output,
        ),
        "io",
    );
    assert_eq!(fs::read(output).unwrap(), original);
}
