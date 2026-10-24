// SPDX-License-Identifier: LGPL-2.1-or-later
//! R5-A (no OCCT): a retained NativeBrep must get a display proxy from the native reader.
//!
//! ```text
//! OM9_EXCHANGE_NATIVE=... OM9_EXCHANGE_NATIVE_SHA256=... OM9_SOURCE_FIXTURE=...
//! cargo test -p om9-app --test r5a_display -- --ignored --test-threads=1
//! ```
#![cfg(all(windows, not(feature = "occt-brep")))]

use om9_app::three_dm_host::ThreeDmHost;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::Document,
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{BrepMode, ImportMode, ImportOptions};
use std::{path::PathBuf, process::Command};

fn native() -> PathBuf {
    PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").expect("OM9_EXCHANGE_NATIVE"))
}

fn fixture(path: &std::path::Path) {
    let output = Command::new(std::env::var_os("OM9_SOURCE_FIXTURE").expect("fixture"))
        .arg("mm")
        .arg(path)
        .output()
        .expect("fixture builder");
    assert!(output.status.success());
}

#[test]
#[ignore = "requires the admitted native reader and binary fixture builder"]
fn retained_native_brep_has_a_display_proxy_without_occt() {
    let dir = std::env::temp_dir().join(format!("om9-r5a-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let source = dir.join("mm.3dm");
    fixture(&source);
    let host = ThreeDmHost::from_environment().expect("native reader");
    assert_eq!(host.program(), native().as_path());

    let document = Document::with_template_tolerance("R5-A");
    let id = document.id;
    let limits = Limits::default();
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let receipt = host
        .import_with_options(
            &mut coordinator,
            id,
            &source,
            &ImportOptions {
                mode: ImportMode::Geometry,
                brep: BrepMode::RetainOpaque,
                ..ImportOptions::default()
            },
            &Cancellation::default(),
        )
        .expect("retained import");
    assert!(receipt.breps_opaque > 0);
    assert!(receipt.breps_displayable > 0);
    assert_eq!(receipt.breps_display_unavailable, 0);
    let native_breps = coordinator.world().documents[&id]
        .entities
        .values()
        .filter_map(|entity| match &entity.geometry {
            Geometry::NativeBrep(native) => Some(native),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!native_breps.is_empty());
    for native in native_breps {
        assert!(!native.display.vertices.is_empty());
        assert!(!native.display.faces.is_empty());
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "requires the admitted native reader and binary fixture builder"]
fn best_effort_without_occt_retains_opaque_and_does_not_abort() {
    let dir = std::env::temp_dir().join(format!("om9-r5a-be-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let source = dir.join("mm.3dm");
    fixture(&source);
    let host = ThreeDmHost::from_environment().expect("native reader");
    let document = Document::with_template_tolerance("R5-A best effort");
    let id = document.id;
    let limits = Limits::default();
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let receipt = host
        .import_with_options(
            &mut coordinator,
            id,
            &source,
            &ImportOptions {
                mode: ImportMode::Geometry,
                brep: BrepMode::EditableBestEffort,
                ..ImportOptions::default()
            },
            &Cancellation::default(),
        )
        .expect("best-effort import");
    assert!(receipt.breps_opaque > 0);
    assert_eq!(receipt.breps_editable, 0);
    assert!(
        receipt
            .diagnostics
            .iter()
            .any(|line| line.contains("BREP_CONVERSION_UNAVAILABLE"))
    );
    let _ = std::fs::remove_dir_all(dir);
}
