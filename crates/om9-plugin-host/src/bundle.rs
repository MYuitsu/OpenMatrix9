// SPDX-License-Identifier: LGPL-2.1-or-later
//! Allowlisted source SDK distribution, independent of the OM9 application tree.

use crate::error;
use om9_sdk::Result;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use zip::{ZipWriter, write::SimpleFileOptions};

const MAX_SOURCE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_BUNDLE_BYTES: usize = 16 * 1024 * 1024;

const REQUIRED_FILES: &[&str] = &[
    "LICENSE",
    "crates/om9-sdk/src/lib.rs",
    "crates/om9-sdk/src/abi.rs",
    "crates/om9-sdk/include/om9_sdk.h",
    "crates/om9-sdk/README.md",
    "sdk/examples/hello-plugin/src/lib.rs",
    "sdk/examples/hello-plugin/descriptor.json",
    "sdk/examples/hello-plugin/arguments.json",
];
const OPTIONAL_FILES: &[&str] = &[
    "sdk/README.md",
    "sdk/API-INVENTORY.md",
    "skills/openmatrix9-plugin-port/SKILL.md",
    "skills/openmatrix9-plugin-port/references/rhino-api-map.md",
];

const SDK_MANIFEST: &str = r#"[package]
name = "om9-sdk"
version = "0.1.0"
edition = "2024"
license = "LGPL-2.1-or-later"
rust-version = "1.88"
description = "Public plugin contract and versioned native ABI for standalone OpenMatrix9"
readme = "README.md"

[dependencies]
serde = { version = "=1.0.228", features = ["derive", "rc"] }
serde_json = { version = "=1.0.150", features = ["float_roundtrip"] }

[workspace]
"#;

const BUNDLE_README: &str = r#"# Standalone OM9 plugin development SDK

Extract this archive into a new directory. No OM9 core, application, kernel,
native SDK, Rhino runtime or private reference sources are required to compile
the supplied plugin. Install Rust 1.88 or newer and the native toolchain for
the machine on which you build. Cargo downloads the pinned public Rust
dependencies from its configured registry; an offline build requires those
dependencies to be cached beforehand.

Choose a build directory outside this extracted source tree. For example, in
PowerShell use `$env:CARGO_TARGET_DIR = 'L:\om9-plugin-target'`; in a macOS
shell use `export CARGO_TARGET_DIR="$HOME/.cache/om9-plugin-target"`.
From the extraction directory run:

```text
cargo build --release --manifest-path sdk/examples/hello-plugin/Cargo.toml -j 2
```

The example's relative dependency resolves to `crates/om9-sdk` in this archive.
Both manifests are independent of the OM9 repository workspace. The source
SDK exposes document/selection/layer queries and a deliberately limited set of
geometry edits; see `sdk/README.md`, `sdk/API-INVENTORY.md` and the crate README
for capabilities and remaining work. This archive does not claim full Rhino SDK
coverage or automatic execution of existing .rhp binaries.

Windows produces a .dll and macOS produces a .dylib. Build separately for each
promised OS/CPU target, then package those libraries into one .om9p. This archive
contains source, not a compiled package tool or an OM9 runtime. Build the
`om9-plugin` tool separately from the full OM9 repository, or obtain a matching
compiled tool separately; use its `pack` command with the example descriptor.
Native plugin code is trusted code with the host process's privileges.

The root LICENSE and original notices apply to these distributed sources.
"#;

/// Create a standalone source SDK ZIP without modifying any source file.
/// Existing output files are refused. Only explicitly allowlisted source and
/// documentation files are copied; symlinks and oversized inputs are rejected.
/// Supported manifest pins are checked before writing. On manifest drift this
/// command fails explicitly instead of silently distributing stale dependencies.
pub fn create(root: &Path, output: &Path) -> Result<()> {
    let root_manifest = text(root, "Cargo.toml")?;
    let sdk_manifest = text(root, "crates/om9-sdk/Cargo.toml")?;
    let example_manifest = text(root, "sdk/examples/hello-plugin/Cargo.toml")?;
    check_manifests(&root_manifest, &sdk_manifest, &example_manifest)?;

    let mut files = BTreeMap::<String, Vec<u8>>::new();
    files.insert("BUNDLE-README.md".into(), BUNDLE_README.as_bytes().to_vec());
    files.insert(
        "crates/om9-sdk/Cargo.toml".into(),
        SDK_MANIFEST.as_bytes().to_vec(),
    );
    files.insert(
        "sdk/examples/hello-plugin/Cargo.toml".into(),
        format!("{}\n[workspace]\n", example_manifest.trim_end()).into_bytes(),
    );
    for name in REQUIRED_FILES {
        files.insert((*name).into(), source(root, name)?);
    }
    for name in OPTIONAL_FILES {
        match fs::symlink_metadata(root.join(name)) {
            Ok(_) => {
                files.insert((*name).into(), source(root, name)?);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(error("sdk_bundle_io", e)),
        }
    }
    if files.values().map(Vec::len).sum::<usize>() > MAX_BUNDLE_BYTES {
        return Err(error("sdk_bundle_budget", "SDK bundle exceeds 16 MiB"));
    }

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| error("sdk_bundle_output", e))?;
    let result = (|| -> Result<()> {
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, bytes) in files {
            writer
                .start_file(name, options)
                .map_err(|e| error("sdk_bundle_zip", e))?;
            writer
                .write_all(&bytes)
                .map_err(|e| error("sdk_bundle_io", e))?;
        }
        writer
            .finish()
            .map_err(|e| error("sdk_bundle_zip", e))?
            .sync_all()
            .map_err(|e| error("sdk_bundle_io", e))?;
        Ok(())
    })();
    if result.is_err() {
        // create_new established ownership of this exact output. No source or
        // preexisting user file is removed on a failed attempt.
        let _ = fs::remove_file(output);
    }
    result
}

fn source(root: &Path, name: &str) -> Result<Vec<u8>> {
    let path = root.join(name);
    if !fs::symlink_metadata(&path)
        .map_err(|e| error("sdk_bundle_io", e))?
        .file_type()
        .is_file()
    {
        return Err(error(
            "sdk_bundle_source",
            format!("expected a regular source file: {name}"),
        ));
    }
    let canonical_root = root.canonicalize().map_err(|e| error("sdk_bundle_io", e))?;
    if !path
        .canonicalize()
        .map_err(|e| error("sdk_bundle_io", e))?
        .starts_with(canonical_root)
    {
        return Err(error(
            "sdk_bundle_source",
            "allowlisted source escapes repository root",
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| error("sdk_bundle_io", e))?
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error("sdk_bundle_io", e))?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(error(
            "sdk_bundle_budget",
            format!("source exceeds 2 MiB: {name}"),
        ));
    }
    Ok(bytes)
}

fn text(root: &Path, name: &str) -> Result<String> {
    String::from_utf8(source(root, name)?).map_err(|e| error("sdk_bundle_manifest", e))
}

// This is intentionally a narrow recognizer for our checked-in manifests, not
// a general TOML parser. Unsupported/multiline declarations fail closed. The
// bundle's standalone manifests must be updated deliberately with SDK changes.
fn section(source: &str, wanted: &str) -> Result<BTreeMap<String, String>> {
    let mut active = false;
    let mut found = false;
    let mut entries = BTreeMap::new();
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            active = line == format!("[{wanted}]");
            if active && found {
                return Err(drift());
            }
            found |= active;
            continue;
        }
        if active {
            let (key, value) = line.split_once('=').ok_or_else(drift)?;
            let key = key.trim().to_owned();
            let value: String = value.chars().filter(|c| !c.is_ascii_whitespace()).collect();
            if entries.insert(key, value).is_some() {
                return Err(drift());
            }
        }
    }
    if !found {
        return Err(drift());
    }
    Ok(entries)
}

fn expected(entries: &BTreeMap<String, String>, key: &str, value: &str) -> Result<()> {
    let compact: String = value.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if entries.get(key) != Some(&compact) {
        return Err(drift());
    }
    Ok(())
}

fn drift() -> om9_sdk::SdkError {
    error(
        "sdk_bundle_manifest_drift",
        "unsupported SDK/workspace/example manifest change; update the standalone SDK bundle pins and allowlist",
    )
}

fn check_manifests(root: &str, sdk: &str, example: &str) -> Result<()> {
    let workspace = section(root, "workspace.package")?;
    for (key, value) in [
        ("version", "\"0.1.0\""),
        ("edition", "\"2024\""),
        ("license", "\"LGPL-2.1-or-later\""),
        ("rust-version", "\"1.88\""),
    ] {
        expected(&workspace, key, value)?;
    }
    let dependencies = section(root, "workspace.dependencies")?;
    expected(
        &dependencies,
        "serde",
        r#"{ version = "=1.0.228", features = ["derive", "rc"] }"#,
    )?;
    expected(
        &dependencies,
        "serde_json",
        r#"{ version = "=1.0.150", features = ["float_roundtrip"] }"#,
    )?;
    let package = section(sdk, "package")?;
    if package.len() != 7 {
        return Err(drift());
    }
    expected(&package, "name", "\"om9-sdk\"")?;
    for key in [
        "version.workspace",
        "edition.workspace",
        "license.workspace",
        "rust-version.workspace",
    ] {
        expected(&package, key, "true")?;
    }
    expected(
        &package,
        "description",
        r#""Public plugin contract and versioned native ABI for standalone OpenMatrix9""#,
    )?;
    expected(&package, "readme", "\"README.md\"")?;
    let dependencies = section(sdk, "dependencies")?;
    if dependencies.len() != 2 {
        return Err(drift());
    }
    expected(&dependencies, "serde.workspace", "true")?;
    expected(&dependencies, "serde_json.workspace", "true")?;

    let package = section(example, "package")?;
    if package.len() != 5 {
        return Err(drift());
    }
    for (key, value) in [
        ("name", "\"om9-hello-plugin\""),
        ("version", "\"0.1.0\""),
        ("edition", "\"2024\""),
        ("license", "\"LGPL-2.1-or-later\""),
        ("rust-version", "\"1.88\""),
    ] {
        expected(&package, key, value)?;
    }
    let library = section(example, "lib")?;
    if library.len() != 1 {
        return Err(drift());
    }
    expected(&library, "crate-type", r#"["cdylib"]"#)?;
    let dependencies = section(example, "dependencies")?;
    if dependencies.len() != 2 {
        return Err(drift());
    }
    expected(
        &dependencies,
        "om9-sdk",
        r#"{ path = "../../../crates/om9-sdk" }"#,
    )?;
    expected(&dependencies, "serde_json", "\"=1.0.150\"")?;
    // Additional source sections could add build scripts, native dependencies,
    // target-specific code or workspace inheritance. Require the exact shape.
    for (source, allowed) in [
        (sdk, &["[package]", "[dependencies]"][..]),
        (example, &["[package]", "[lib]", "[dependencies]"][..]),
    ] {
        if source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with('['))
            .any(|line| !allowed.contains(&line))
        {
            return Err(drift());
        }
    }
    Ok(())
}
