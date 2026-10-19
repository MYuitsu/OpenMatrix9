// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opaque native BRep round trip: import display/payload, export, re-import.
//!
//! Ignored by default because it needs the admitted native executable and a real
//! 3DM solid fixture (`OM9_BREP_FIXTURE`).
#![cfg(windows)]

use om9_archive::Archive;
use om9_core::{budget::Limits, geometry::Geometry};
use std::{path::PathBuf, process::Command};

fn required_env(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("{name} is required")))
}

fn native() -> Command {
    Command::new(required_env("OM9_EXCHANGE_NATIVE"))
}

#[test]
#[ignore = "requires an admitted om9-exchange-native.exe and a BRep 3DM fixture"]
fn brep_roundtrips_as_brep_without_becoming_a_mesh() {
    let fixture = required_env("OM9_BREP_FIXTURE");
    let folder = std::env::temp_dir().join(format!("om9-brep-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&folder).unwrap();
    let imported = folder.join("imported.om9");
    let exported = folder.join("exported.3dm");
    let reopened = folder.join("reopened.om9");
    let binary = env!("CARGO_BIN_EXE_om9");

    let run = |args: &[&std::ffi::OsStr]| {
        let output = Command::new(binary).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&[
        std::ffi::OsStr::new("--import-3dm"),
        fixture.as_os_str(),
        imported.as_os_str(),
    ]);
    let document = Archive::open(&imported, &Limits::default()).unwrap();
    let first_crc = document
        .entities
        .values()
        .find_map(|entity| match &entity.geometry {
            Geometry::NativeBrep(brep) => Some(brep.crc),
            _ => None,
        })
        .expect("the solid must import as an opaque BRep");

    run(&[
        std::ffi::OsStr::new("--export-3dm"),
        imported.as_os_str(),
        exported.as_os_str(),
    ]);
    let inspected = native().arg("--inspect").arg(&exported).output().unwrap();
    assert!(inspected.status.success());
    let report: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(report["objects"].as_array().map(Vec::len), Some(1));
    assert_eq!(report["objects"][0]["native_kind"], "ON_Brep");

    run(&[
        std::ffi::OsStr::new("--import-3dm"),
        exported.as_os_str(),
        reopened.as_os_str(),
    ]);
    let document = Archive::open(&reopened, &Limits::default()).unwrap();
    let second_crc = document
        .entities
        .values()
        .find_map(|entity| match &entity.geometry {
            Geometry::NativeBrep(brep) => Some(brep.crc),
            _ => None,
        })
        .expect("the reimported solid must remain an opaque BRep");
    assert_eq!(first_crc, second_crc, "BRep topology CRC must round-trip");

    for path in [imported, exported, reopened] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(folder).unwrap();
}
