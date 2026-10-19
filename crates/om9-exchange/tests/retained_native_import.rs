// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Actual source files authored by the independently compiled SDK fixture.
use om9_core::{budget::Limits, geometry::Geometry, model::Document, transaction::*};
use om9_exchange::{
    ExportCapture, ImportCapture, ImportMode, ImportOptions, NativeExchange, RecordAction,
};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::var_os("OM9_NATIVE_IMPORT_SCRATCH")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let p = root.join(format!("om9-source-import-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        for f in std::fs::read_dir(&self.0).unwrap() {
            let p = f.unwrap().path();
            if p.is_file() {
                let _ = std::fs::remove_file(p);
            }
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}
fn program(variable: &str) -> NativeProgram {
    let path = PathBuf::from(
        std::env::var_os(variable).expect("explicit authored native binary required"),
    );
    let hash: [u8; 32] = Sha256::digest(std::fs::read(&path).unwrap()).into();
    NativeProgram::admit(&path, hash).unwrap()
}
fn fixture(mode: &str, path: &Path) {
    program("OM9_SOURCE_FIXTURE")
        .run(
            &[OsString::from(mode), path.as_os_str().to_owned()],
            &ProcessLimits::default(),
            &Cancellation::default(),
        )
        .unwrap();
}
fn coordinator() -> (TransactionCoordinator, om9_core::identity::DocumentId) {
    let d = Document::new("native import target");
    let id = d.id;
    (
        TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        id,
    )
}
fn exchange() -> NativeExchange {
    NativeExchange::new(program("OM9_EXCHANGE_NATIVE"), ProcessLimits::default()).unwrap()
}
fn native_id(n: u32) -> uuid::Uuid {
    uuid::Uuid::parse_str(&format!("20000000-0000-4000-8000-{n:012x}")).unwrap()
}
fn bound_object(document: &Document, n: u32) -> Option<om9_core::identity::ObjectId> {
    document
        .import_namespaces
        .values()
        .flat_map(|ns| &ns.records)
        .find_map(|record| {
            (record.key.id == native_id(n))
                .then_some(record.host)
                .flatten()
        })
        .and_then(|host| match host {
            om9_core::retained::HostRef::Object(id) => Some(id),
            _ => None,
        })
}
fn explicit_units() -> ImportOptions {
    let mut options = ImportOptions::default();
    options.units.model_mm_per_unit = Some(25.4);
    options.units.page_mm_per_unit = Some(7.);
    options.units.definition_mm_per_unit = [30, 31, 32].map(|n| (native_id(n), 25.4)).into();
    options
}
fn import(
    c: &mut TransactionCoordinator,
    id: om9_core::identity::DocumentId,
    source: &Path,
    stage: &Path,
    options: &ImportOptions,
) -> om9_exchange::ImportReceipt {
    let ready = exchange()
        .prepare_import(
            c,
            ImportCapture::target(c, id).unwrap(),
            source,
            stage,
            options,
            &Cancellation::default(),
        )
        .unwrap();
    let (prepared, receipt) = ready.prepare(c, &Cancellation::default()).unwrap();
    c.commit(prepared).unwrap();
    let namespace = &c.world().documents[&id].import_namespaces[&receipt.namespace];
    let expected_mode = match options.mode {
        ImportMode::Preserve => om9_core::retained::ImportMode::Preserve,
        ImportMode::Geometry => om9_core::retained::ImportMode::Geometry,
    };
    assert_eq!(namespace.import_mode, Some(expected_mode));
    for record in &namespace.records {
        assert_eq!(
            record.entity_baseline.is_some(),
            matches!(
                record.host,
                Some(
                    om9_core::retained::HostRef::Object(_)
                        | om9_core::retained::HostRef::DefinitionMember { .. }
                )
            )
        );
    }
    let capture = ExportCapture::all(c, id).unwrap();
    let overlay = capture
        .current_overlay(c, &Cancellation::default())
        .unwrap();
    let rows = &overlay
        .namespaces
        .iter()
        .find(|n| n.namespace.id == receipt.namespace)
        .unwrap()
        .records;
    for row in rows {
        assert_eq!(
            row.action,
            match (expected_mode, row.source.host) {
                (om9_core::retained::ImportMode::Preserve, _) => RecordAction::Retain,
                (om9_core::retained::ImportMode::Geometry, Some(_)) => RecordAction::Regenerate,
                (om9_core::retained::ImportMode::Geometry, None) => RecordAction::Omit,
            }
        );
    }
    let primary = (options.mode == ImportMode::Preserve).then_some(receipt.namespace);
    let output = capture
        .current_output_plan(c, primary, &Cancellation::default(), Default::default())
        .unwrap();
    assert_eq!(
        output.source_identities.len(),
        c.world().documents[&id]
            .import_namespaces
            .values()
            .map(|ns| ns.records.len())
            .sum::<usize>()
    );
    for id in c.world().documents[&id].entities.keys() {
        assert!(
            output
                .host_identities
                .contains_key(&om9_core::retained::HostRef::Object(*id))
        );
    }
    receipt
}

#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_units_and_mixed_definition_contexts_preserve_the_same_world_coordinates() {
    use om9_scene::scene::{RenderBuffers, Scene};
    for mode in [
        "clean-mm",
        "clean-inch",
        "clean-custom",
        "clean-mixed",
        "clean-unitless",
        "clean-unset",
    ] {
        let temp = Temp::new();
        let source = temp.0.join(format!("{mode}.3dm"));
        fixture(mode, &source);
        let options = if matches!(mode, "clean-unitless" | "clean-unset") {
            explicit_units()
        } else {
            ImportOptions::default()
        };
        let (mut c, id) = coordinator();
        let receipt = import(&mut c, id, &source, &temp.0, &options);
        assert_eq!(
            (
                receipt.root_objects,
                receipt.definition_members,
                receipt.definitions
            ),
            (12, 3, 3),
            "{mode}"
        );
        assert!(receipt.native.exited && receipt.native.active_processes == 0);
        assert!(
            receipt.native.peak_committed_bytes <= ProcessLimits::default().native_committed_bytes
        );
        let d = &c.world().documents[&id];
        let point = &d.entities[&bound_object(d, 10).unwrap()];
        assert!(
            matches!(point.geometry, Geometry::Point(p) if (p[0]-25.4).abs()<1e-10 && (p[1]-50.8).abs()<1e-10),
            "{mode}"
        );
        let scene = Scene::capture(d.clone(), c.epoch(), &Limits::default()).unwrap();
        for (n, expected) in [(18, [8., 21., 12.]), (19, [1., 2., 3.]), (27, [8., 2., 3.])] {
            let host = bound_object(d, n).unwrap();
            let leaves: Vec<_> = scene.objects.iter().filter(|o| o.root == host).collect();
            assert_eq!(leaves.len(), 1, "{mode}, root {n}");
            let RenderBuffers::Points { points, .. } = leaves[0].buffers.as_ref() else {
                panic!("instance point buffer")
            };
            let world = leaves[0].transform.point(points[0]).unwrap();
            for axis in 0..3 {
                assert!(
                    (world[axis] - expected[axis]).abs() < 1e-10,
                    "{mode}, root {n}: {world:?}"
                );
            }
        }
        let contexts = &d.retained_sources[&receipt.source].units.definitions;
        assert_eq!(contexts.len(), 3);
        if mode == "clean-mixed" {
            assert_eq!(contexts[&native_id(30)].millimeters_per_unit, Some(25.4));
            assert_eq!(contexts[&native_id(31)].millimeters_per_unit, Some(1.));
            assert_eq!(contexts[&native_id(32)].millimeters_per_unit, Some(2.5));
        }
    }
}

#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_unresolved_definition_closure_stays_retained_while_independent_roots_import() {
    let temp = Temp::new();
    let source = temp.0.join("unresolved.3dm");
    fixture("clean-unresolved", &source);
    for resolve in [false, true] {
        let (mut c, id) = coordinator();
        let mut options = ImportOptions::default();
        if resolve {
            options
                .units
                .definition_mm_per_unit
                .insert(native_id(33), 25.4);
        }
        let receipt = import(&mut c, id, &source, &temp.0, &options);
        let d = &c.world().documents[&id];
        assert_eq!(receipt.root_objects, if resolve { 13 } else { 12 });
        assert_eq!(receipt.definitions, if resolve { 4 } else { 3 });
        assert!(bound_object(d, 27).is_some());
        assert_eq!(bound_object(d, 28).is_some(), resolve);
        let context = &d.retained_sources[&receipt.source].units.definitions[&native_id(33)];
        assert_eq!(context.native_enum, 0);
        assert_eq!(context.millimeters_per_unit, resolve.then_some(25.4));
        assert_eq!(
            context.resolved_by,
            resolve.then_some(om9_core::retained::ResolvedBy::Explicit)
        );
        if !resolve {
            let records = &d.import_namespaces[&receipt.namespace].records;
            for n in [28, 33, 44] {
                assert!(
                    records
                        .iter()
                        .any(|r| r.key.id == native_id(n) && r.host.is_none())
                );
            }
        }
    }
}

#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_preservation_and_explicit_geometry_mode_report_all_bound_field_losses() {
    let temp = Temp::new();
    let source = temp.0.join("complex.3dm");
    fixture("mm", &source);
    for mode in [ImportMode::Preserve, ImportMode::Geometry] {
        let (mut c, id) = coordinator();
        let options = ImportOptions {
            mode,
            ..ImportOptions::default()
        };
        let receipt = import(&mut c, id, &source, &temp.0, &options);
        let d = &c.world().documents[&id];
        assert!(
            d.resources
                .values()
                .all(|r| r.payload().unwrap() == std::fs::read(&source).unwrap())
        );
        if mode == ImportMode::Preserve {
            assert_eq!(
                (receipt.root_objects, receipt.definitions, receipt.layers),
                (0, 0, 1)
            );
        } else {
            assert!(receipt.root_objects > 0 && receipt.definitions > 0 && receipt.layers == 2);
            assert_eq!(
                d.entities[&bound_object(d, 10).unwrap()].attributes.color,
                Some([11, 22, 33, 200])
            );
            for n in [18, 19, 20, 21, 22, 24, 25, 26] {
                assert!(
                    bound_object(d, n).is_none(),
                    "unsupported native closure {n}"
                );
            }
            assert!(
                bound_object(d, 27).is_some(),
                "independent supported definition remains available"
            );
            for (n, family) in [(2, "layer"), (10, "object"), (43, "definition member")] {
                assert!(
                    receipt
                        .diagnostics
                        .iter()
                        .any(|s| s.starts_with(&native_id(n).to_string())
                            && s.contains(family)
                            && s.contains("omits native fields")),
                    "missing {family} loss {n}"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_import_original_cancel_caller_scope_and_budget_failure_never_publish_partial_state() {
    let temp = Temp::new();
    let source = temp.0.join("target.3dm");
    fixture("clean-mm", &source);
    for late in [false, true] {
        let (mut c, id) = coordinator();
        let original = Cancellation::default();
        let ready = exchange()
            .prepare_import(
                &c,
                ImportCapture::target(&c, id).unwrap(),
                &source,
                &temp.0,
                &ImportOptions::default(),
                &original,
            )
            .unwrap();
        if late {
            let (p, _) = ready.prepare(&c, &Cancellation::default()).unwrap();
            original.cancel();
            let before = c.snapshot();
            assert_eq!(c.commit(p), Err(om9_core::Error::Canceled));
            assert_eq!(c.snapshot(), before);
        } else {
            original.cancel();
            let before = c.snapshot();
            assert!(matches!(
                ready.prepare(&c, &Cancellation::default()),
                Err(om9_core::Error::Canceled)
            ));
            assert_eq!(c.snapshot(), before);
        }
    }
    let (mut c, id) = coordinator();
    let before = c.world().clone();
    let owner = c.book("real scoped import", true).unwrap();
    c.begin(&owner).unwrap();
    let ready = exchange()
        .prepare_import(
            &c,
            ImportCapture::for_scope(&c, id, &owner).unwrap(),
            &source,
            &temp.0,
            &ImportOptions::default(),
            &Cancellation::default(),
        )
        .unwrap();
    let (p, _) = ready.prepare(&c, &Cancellation::default()).unwrap();
    assert!(!c.commit(p).unwrap().published);
    assert_eq!(c.world(), &before);
    c.finish(&owner).unwrap();
    assert_eq!(c.world().documents[&id].entities.len(), 12);
    assert_eq!(c.undo_len(), 1);
    for limits in [
        Limits {
            max_bytes: 128 * 1024 * 1024,
            ..Limits::default()
        },
        Limits {
            max_points: 1,
            ..Limits::default()
        },
    ] {
        let d = Document::new("bounded import");
        let id = d.id;
        let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
        let before = c.snapshot();
        assert!(matches!(
            exchange().prepare_import(
                &c,
                ImportCapture::target(&c, id).unwrap(),
                &source,
                &temp.0,
                &ImportOptions::default(),
                &Cancellation::default()
            ),
            Err(om9_core::Error::Budget(_))
        ));
        assert_eq!(c.snapshot(), before);
        assert!(
            std::fs::read_dir(&temp.0)
                .unwrap()
                .all(|entry| entry.unwrap().path().is_file())
        );
    }
}

#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_unsupported_color_selectors_are_retained_or_reported_with_explicit_loss() {
    for fixture_mode in ["clean-color-parent", "clean-color-material"] {
        let temp = Temp::new();
        let source = temp.0.join("color.3dm");
        fixture(fixture_mode, &source);
        for mode in [ImportMode::Preserve, ImportMode::Geometry] {
            let (mut c, id) = coordinator();
            let options = ImportOptions {
                mode,
                ..ImportOptions::default()
            };
            let receipt = import(&mut c, id, &source, &temp.0, &options);
            let d = &c.world().documents[&id];
            assert_eq!(
                receipt.root_objects,
                if mode == ImportMode::Preserve { 11 } else { 12 }
            );
            let point = bound_object(d, 10);
            if mode == ImportMode::Preserve {
                assert!(point.is_none());
            } else {
                assert_eq!(d.entities[&point.unwrap()].attributes.color, None);
                assert!(
                    receipt
                        .diagnostics
                        .iter()
                        .any(|message| message.starts_with(&native_id(10).to_string())
                            && message.contains("omits native fields")
                            && message.contains("color_source")),
                    "{fixture_mode}: {:?}",
                    receipt.diagnostics
                );
            }
        }
    }
}

#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_import_cold_om9_reopen_preserves_explicit_definition_context_without_source_file() {
    const ENV: &str = "OM9_REAL_IMPORT_COLD";
    if let Some(path) = std::env::var_os(ENV) {
        let path = PathBuf::from(path);
        assert!(!path.with_extension("3dm").exists());
        let restored =
            om9_archive::Archive::open_restored(&path, &Limits::default(), &Default::default())
                .unwrap();
        let d = restored.document;
        assert_eq!(
            (
                d.entities.len(),
                d.definitions.len(),
                d.import_namespaces.len()
            ),
            (12, 3, 1)
        );
        let source = d.retained_sources.values().next().unwrap();
        assert_eq!(
            source.units.model.resolved_by,
            om9_core::retained::ResolvedBy::Explicit
        );
        assert_eq!(source.units.model.millimeters_per_unit, 25.4);
        assert_eq!(source.units.page.millimeters_per_unit, 7.);
        for context in source.units.definitions.values() {
            assert_eq!(context.native_enum, 0);
            assert_eq!(context.millimeters_per_unit, Some(25.4));
            assert_eq!(
                context.resolved_by,
                Some(om9_core::retained::ResolvedBy::Explicit)
            );
        }
        let resource = &d.resources[&source.resource];
        assert_eq!(
            source.sha256,
            Sha256::digest(resource.payload().unwrap()).as_slice()
        );
        assert!(bound_object(&d, 10).is_some());
        return;
    }
    let temp = Temp::new();
    let source = temp.0.join("owned.3dm");
    let archive = temp.0.join("owned.om9");
    fixture("clean-unitless", &source);
    let (mut c, id) = coordinator();
    import(&mut c, id, &source, &temp.0, &explicit_units());
    om9_archive::Archive::save(
        &archive,
        &c.world().documents[&id],
        &Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    std::fs::remove_file(source).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "real_import_cold_om9_reopen_preserves_explicit_definition_context_without_source_file",
            "--nocapture",
        ])
        .env(ENV, &archive)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "actual cold import failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_source_import_keeps_native_geometry_shared_members_original_bytes_and_fresh_namespaces() {
    let temp = Temp::new();
    let source = temp.0.join("source.3dm");
    fixture("clean-mm", &source);
    let original = std::fs::read(&source).unwrap();
    let expected: [u8; 32] = Sha256::digest(&original).into();
    let (mut c, id) = coordinator();
    let native = exchange();
    for index in 0..2 {
        let before = c.world().clone();
        let capture = ImportCapture::target(&c, id).unwrap();
        let ready = native
            .prepare_import(
                &c,
                capture,
                &source,
                &temp.0,
                &ImportOptions::default(),
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(c.world(), &before, "native parse/conversion cannot publish");
        assert_eq!(ready.receipt().source_sha256, expected);
        let (prepared, receipt) = ready.prepare(&c, &Cancellation::default()).unwrap();
        c.commit(prepared).unwrap();
        assert_eq!(receipt.root_objects, 12);
        assert_eq!(receipt.definition_members, 3);
        assert_eq!(receipt.definitions, 3);
        let d = &c.world().documents[&id];
        assert_eq!(d.entities.len(), 12 * (index + 1));
        assert_eq!(d.definitions.len(), 3 * (index + 1));
        assert_eq!(d.import_namespaces.len(), index + 1);
        assert_eq!(d.retained_sources.len(), index + 1);
        assert!(d.entities.values().any(|e|matches!(e.geometry,Geometry::Point(p) if (p[0]-25.4).abs()<1e-10 && (p[1]-50.8).abs()<1e-10)));
        assert!(
            d.resources
                .values()
                .all(|r| r.payload().unwrap() == original)
        );
        assert_eq!(
            d.current_layer, None,
            "import preserves target current-layer policy"
        );
    }
    c.undo().unwrap();
    assert_eq!(c.world().documents[&id].entities.len(), 12);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&id].entities.len(), 24);
}
#[test]
#[ignore = "requires actual authored openNURBS source fixture and native projection binary"]
fn real_source_import_rejects_unknown_units_and_stale_canceled_targets_without_partial_state() {
    let temp = Temp::new();
    let source = temp.0.join("unitless.3dm");
    fixture("clean-unitless", &source);
    let (mut c, id) = coordinator();
    let before = c.snapshot();
    let native = exchange();
    assert!(
        native
            .prepare_import(
                &c,
                ImportCapture::target(&c, id).unwrap(),
                &source,
                &temp.0,
                &ImportOptions::default(),
                &Cancellation::default()
            )
            .is_err()
    );
    assert_eq!(c.snapshot(), before);
    let source = temp.0.join("known.3dm");
    fixture("clean-mm", &source);
    let ready = native
        .prepare_import(
            &c,
            ImportCapture::target(&c, id).unwrap(),
            &source,
            &temp.0,
            &ImportOptions::default(),
            &Cancellation::default(),
        )
        .unwrap();
    c.close_document(id).unwrap();
    let closed = c.snapshot();
    assert!(ready.prepare(&c, &Cancellation::default()).is_err());
    assert_eq!(c.snapshot(), closed);
    let (c, id) = coordinator();
    let before = c.snapshot();
    let canceled = Cancellation::default();
    canceled.cancel();
    assert!(
        native
            .prepare_import(
                &c,
                ImportCapture::target(&c, id).unwrap(),
                &source,
                &temp.0,
                &ImportOptions::default(),
                &canceled
            )
            .is_err()
    );
    assert_eq!(c.snapshot(), before);
}
