// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_plugin_host::bundle;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
};
use zip::{CompressionMethod, ZipArchive};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("om9-sdk-bundle-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        assert!(
            self.0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("om9-sdk-bundle-test-")
        );
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn checkout() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn entries(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
    let mut entries = BTreeMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        assert_eq!(file.compression(), CompressionMethod::Stored);
        let name = file.name().unwrap().into_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        assert!(entries.insert(name, bytes).is_none());
    }
    entries
}

#[test]
fn checkout_bundle_is_an_independent_allowlisted_sdk_and_example() {
    let temp = Temp::new();
    let output = temp.0.join("om9-sdk.zip");
    let original_sdk_manifest = fs::read(checkout().join("crates/om9-sdk/Cargo.toml")).unwrap();
    let original_example_manifest =
        fs::read(checkout().join("sdk/examples/hello-plugin/Cargo.toml")).unwrap();
    bundle::create(checkout(), &output).unwrap();
    assert_eq!(
        fs::read(checkout().join("crates/om9-sdk/Cargo.toml")).unwrap(),
        original_sdk_manifest
    );
    assert_eq!(
        fs::read(checkout().join("sdk/examples/hello-plugin/Cargo.toml")).unwrap(),
        original_example_manifest
    );
    let files = entries(&output);
    for required in [
        "BUNDLE-README.md",
        "LICENSE",
        "crates/om9-sdk/Cargo.toml",
        "crates/om9-sdk/src/lib.rs",
        "crates/om9-sdk/src/abi.rs",
        "crates/om9-sdk/include/om9_sdk.h",
        "crates/om9-sdk/README.md",
        "sdk/examples/hello-plugin/Cargo.toml",
        "sdk/examples/hello-plugin/src/lib.rs",
        "sdk/examples/hello-plugin/descriptor.json",
        "sdk/examples/hello-plugin/arguments.json",
    ] {
        assert!(files.contains_key(required), "missing {required}");
    }
    assert!(!files.contains_key("Cargo.toml"));
    for name in files.keys() {
        assert!(
            name == "BUNDLE-README.md"
                || name == "LICENSE"
                || name.starts_with("crates/om9-sdk/")
                || name.starts_with("sdk/")
                || name.starts_with("skills/openmatrix9-plugin-port/"),
            "unexpected file {name}"
        );
        assert!(
            !name.contains("om9-core")
                && !name.contains("om9-app")
                && !name.starts_with("native/")
                && !name.starts_with("ref/")
        );
        assert!(
            ![".dll", ".dylib", ".exe", ".rhp"]
                .iter()
                .any(|suffix| name.ends_with(suffix))
        );
    }

    let sdk = std::str::from_utf8(&files["crates/om9-sdk/Cargo.toml"]).unwrap();
    assert!(!sdk.contains(".workspace"));
    assert!(!sdk.contains("om9-core") && !sdk.contains("om9-app") && !sdk.contains("path ="));
    for expected in [
        "version = \"0.1.0\"",
        "edition = \"2024\"",
        "rust-version = \"1.88\"",
        "=1.0.228",
        "=1.0.150",
        "float_roundtrip",
    ] {
        assert!(sdk.contains(expected), "missing standalone pin {expected}");
    }
    let example = std::str::from_utf8(&files["sdk/examples/hello-plugin/Cargo.toml"]).unwrap();
    assert!(!example.contains(".workspace"));
    assert!(example.ends_with("[workspace]\n"));
    assert!(example.contains("path = \"../../../crates/om9-sdk\""));
    let relative =
        Path::new("sdk/examples/hello-plugin").join("../../../crates/om9-sdk/Cargo.toml");
    let mut resolved = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir => {
                assert!(resolved.pop());
            }
            _ => panic!("unexpected absolute/path prefix in example dependency"),
        }
    }
    assert_eq!(resolved, Path::new("crates/om9-sdk/Cargo.toml"));
    assert!(files.contains_key(resolved.to_str().unwrap().replace('\\', "/").as_str()));
    assert_eq!(
        files["crates/om9-sdk/src/lib.rs"],
        fs::read(checkout().join("crates/om9-sdk/src/lib.rs")).unwrap()
    );
}

#[test]
fn bundle_refuses_overwrite_and_keeps_existing_user_bytes() {
    let temp = Temp::new();
    let output = temp.0.join("existing.zip");
    fs::write(&output, b"existing user file").unwrap();
    assert_eq!(
        bundle::create(checkout(), &output).unwrap_err().code,
        "sdk_bundle_output"
    );
    assert_eq!(fs::read(output).unwrap(), b"existing user file");
}

#[test]
fn manifest_drift_is_refused_before_creating_output() {
    let temp = Temp::new();
    let sdk_directory = temp.0.join("crates/om9-sdk");
    let example_directory = temp.0.join("sdk/examples/hello-plugin");
    fs::create_dir_all(&sdk_directory).unwrap();
    fs::create_dir_all(&example_directory).unwrap();
    for name in [
        "Cargo.toml",
        "crates/om9-sdk/Cargo.toml",
        "sdk/examples/hello-plugin/Cargo.toml",
    ] {
        fs::copy(checkout().join(name), temp.0.join(name)).unwrap();
    }
    let root_manifest = fs::read_to_string(temp.0.join("Cargo.toml")).unwrap();
    fs::write(
        temp.0.join("Cargo.toml"),
        root_manifest.replace("=1.0.228", "=1.0.229"),
    )
    .unwrap();
    let output = temp.0.join("drift.zip");
    assert_eq!(
        bundle::create(&temp.0, &output).unwrap_err().code,
        "sdk_bundle_manifest_drift"
    );
    assert!(!output.exists());
}
