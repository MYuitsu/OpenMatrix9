// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Actual bounded writer through immutable Rust staging, no public publication.
use om9_core::{
    budget::{Limits, Usage},
    geometry::Geometry,
    model::{Document, Entity},
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{ExportCapture, ImportCapture, ImportOptions, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{io::Read, path::PathBuf};
fn exchange() -> NativeExchange {
    let path =
        PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").expect("pinned retained writer"));
    let mut file = std::fs::File::open(&path).unwrap();
    let mut hash = Sha256::new();
    let mut buf = [0; 8192];
    loop {
        let n = file.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    NativeExchange::new(
        NativeProgram::admit(&path, hash.finalize().into()).unwrap(),
        ProcessLimits::default(),
    )
    .unwrap()
}
fn measured_source(operation: &str, stats: &om9_native_owner::ProcessStats) {
    println!(
        "OM9_RETAINED_SOURCE_JOB {}",
        serde_json::json!({
            "operation":operation,"peak_committed_bytes":stats.peak_committed_bytes,
            "active_processes":stats.active_processes,"exited":stats.exited
        })
    );
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let parent = PathBuf::from(
            std::env::var_os("OM9_NATIVE_IMPORT_SCRATCH").expect("explicit H scratch"),
        );
        let path = parent.join(format!("retained-write-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.join("target.3dm"));
        let _ = std::fs::remove_dir(&self.0);
    }
}
#[test]
#[ignore = "requires pinned native retained-current writer"]
fn real_private_writer_hashes_actual_held_file_and_preserves_public_target() {
    let temp = Temp::new();
    let target = temp.0.join("target.3dm");
    std::fs::write(&target, b"old target sentinel").unwrap();
    let mut d = Document::new("private retained write");
    let entity = Entity::new("current point", Geometry::Point([1., 2., 3.]), None);
    d.entities.insert(entity.id, entity);
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let before = c.snapshot();
    let e = exchange();
    let capture = ExportCapture::all(&c, id).unwrap();
    let token = Cancellation::default();
    let inputs = e
        .prepare_merge_inputs(&c, &capture, None, &temp.0, &token, Usage::default())
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &token, Usage::default())
        .unwrap();
    let stage = wire
        .stage_retained_request(&c, &temp.0, &token, Usage::default())
        .unwrap();
    let written = e
        .write_retained_private(&c, &stage, &token, Usage::default())
        .unwrap();
    let r = written.receipt();
    assert_eq!(r.objects, 1);
    assert_eq!(r.layers, 1);
    assert_eq!(r.definitions, 0);
    assert!(r.native.exited && r.native.active_processes == 0);
    let bytes = std::fs::read(written.output_path()).unwrap();
    assert_eq!(<[u8; 32]>::from(Sha256::digest(&bytes)), r.output_sha256);
    assert_eq!(bytes.len() as u64, r.output_bytes);
    assert_eq!(r.request_sha256, stage.request_sha256());
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(written.output_path())
            .is_err()
    );
    assert!(std::fs::remove_file(written.output_path()).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"old target sentinel");
    assert_eq!(c.snapshot(), before);
    println!(
        "OM9_RETAINED_WRITE_JOB {}",
        serde_json::json!({"program_sha256":r.program_sha256,"peak_committed_bytes":r.native.peak_committed_bytes,"active_processes":r.native.active_processes,"exited":r.native.exited})
    );
    let validated = e
        .inspect_retained_private(&c, written, &token, Usage::default())
        .unwrap();
    let cold = validated.proof();
    assert_eq!(cold.live_records(), 2);
    assert_eq!(cold.represented_bounds_checked(), 0);
    assert_eq!(cold.native_only_bounds_checked(), 0);
    assert_eq!(
        cold.output_sha256(),
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
    assert_eq!(cold.request_sha256(), stage.request_sha256());
    let stats = validated.inspect_stats();
    assert!(stats.exited && stats.active_processes == 0);
    assert_eq!(std::fs::read(&target).unwrap(), b"old target sentinel");
    assert_eq!(c.snapshot(), before);
    println!(
        "OM9_RETAINED_COLD_JOB {}",
        serde_json::json!({"program_sha256":validated.write_receipt().program_sha256,"peak_committed_bytes":stats.peak_committed_bytes,"active_processes":stats.active_processes,"exited":stats.exited})
    );
    let output = validated.output_path().to_owned();
    token.cancel();
    assert!(matches!(
        validated.revalidate(&c, &token),
        Err(om9_core::Error::Canceled)
    ));
    assert_eq!(std::fs::read(&target).unwrap(), b"old target sentinel");
    assert_eq!(c.snapshot(), before);
    drop(validated);
    drop(stage);
    assert!(!output.exists());
}

#[test]
#[ignore = "requires pinned native retained-current writer"]
fn real_cold_publication_keeps_old_target_on_loss_permission_cancel_close_and_budget_refusals() {
    let e = exchange();
    for case in [
        "accept",
        "missing_loss_ack",
        "wrong_loss_ack",
        "readonly",
        "cancel",
        "write_cancel",
        "closed",
        "budget",
    ] {
        let temp = Temp::new();
        let target = temp.0.join("target.3dm");
        std::fs::write(&target, b"old target sentinel").unwrap();
        let original_permissions = std::fs::metadata(&target).unwrap().permissions();
        let mut d = Document::new("retained publication");
        let entity = Entity::new("point to publish", Geometry::Point([4., 5., 6.]), None);
        d.entities.insert(entity.id, entity);
        let id = d.id;
        let mut c = TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap();
        let token = Cancellation::default();
        let write_token = if case == "write_cancel" {
            Cancellation::default()
        } else {
            token.clone()
        };
        let capture = ExportCapture::all(&c, id).unwrap();
        let inputs = e
            .prepare_merge_inputs(&c, &capture, None, &temp.0, &token, Usage::default())
            .unwrap();
        let wire = inputs
            .output_wire_plan(&c, &token, Usage::default())
            .unwrap();
        let stage = wire
            .stage_retained_request(&c, &temp.0, &token, Usage::default())
            .unwrap();
        let written = e
            .write_retained_private(&c, &stage, &write_token, Usage::default())
            .unwrap();
        println!(
            "OM9_RETAINED_WRITE_JOB {}",
            serde_json::json!({
                "case":case,"program_sha256":written.receipt().program_sha256,
                "peak_committed_bytes":written.receipt().native.peak_committed_bytes,
                "active_processes":written.receipt().native.active_processes,"exited":written.receipt().native.exited
            })
        );
        let validated = e
            .inspect_retained_private(&c, written, &token, Usage::default())
            .unwrap();
        println!(
            "OM9_RETAINED_COLD_JOB {}",
            serde_json::json!({
                "case":case,"program_sha256":validated.write_receipt().program_sha256,
                "peak_committed_bytes":validated.inspect_stats().peak_committed_bytes,
                "active_processes":validated.inspect_stats().active_processes,"exited":validated.inspect_stats().exited
            })
        );
        let expected_hash = validated.proof().output_sha256();
        let expected_bytes = validated.proof().output_bytes();
        let mut losses = validated.losses().unwrap();
        assert!(losses.om9_state_not_encoded);
        if case == "wrong_loss_ack" {
            losses.om9_resources += 1;
        }
        if case == "readonly" {
            let mut p = std::fs::metadata(&target).unwrap().permissions();
            p.set_readonly(true);
            std::fs::set_permissions(&target, p).unwrap();
        }
        if case == "cancel" {
            token.cancel();
        }
        if case == "write_cancel" {
            write_token.cancel();
        }
        if case == "closed" {
            c.close_document(id).unwrap();
        }
        let before = c.snapshot();
        let external = if case == "budget" {
            Usage {
                bytes: c.limits().max_bytes,
                ..Usage::default()
            }
        } else {
            Usage::default()
        };
        let result = validated.publish(
            &c,
            &target,
            &Cancellation::default(),
            if case == "missing_loss_ack" {
                None
            } else {
                Some(losses)
            },
            external,
        );
        assert_eq!(c.snapshot(), before, "{case}");
        if case == "accept" {
            let receipt = result.unwrap();
            assert_eq!(receipt.path, target.canonicalize().unwrap());
            assert_eq!(receipt.proof.output_sha256(), expected_hash);
            let data = std::fs::read(&target).unwrap();
            assert_eq!(data.len() as u64, expected_bytes);
            assert_eq!(<[u8; 32]>::from(Sha256::digest(&data)), expected_hash);
        } else {
            assert!(result.is_err(), "{case}");
            assert_eq!(
                std::fs::read(&target).unwrap(),
                b"old target sentinel",
                "{case}"
            );
        }
        if case == "readonly" {
            std::fs::set_permissions(&target, original_permissions).unwrap();
        }
        drop(stage);
        assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1, "{case}");
        println!("OM9_RETAINED_PUBLICATION_CASE {case}");
    }
}

#[test]
#[ignore = "requires pinned native writer and SDK authored source fixture"]
fn real_retained_sources_cold_verify_original_units_raw_tables_and_block_structure_without_external_source()
 {
    let e = exchange();
    let fixture =
        PathBuf::from(std::env::var_os("OM9_SOURCE_FIXTURE").expect("pinned source fixture"));
    let fixture_sha = Sha256::digest(std::fs::read(&fixture).unwrap()).into();
    let fixture = NativeProgram::admit(&fixture, fixture_sha).unwrap();
    for mode in [
        "clean-mm",
        "clean-inch",
        "clean-custom",
        "clean-mixed",
        "clean-unitless",
        "clean-unset",
        "mm",
    ] {
        let temp = Temp::new();
        let source = temp.0.join("source.3dm");
        let authored = fixture
            .run(
                &[mode.into(), source.as_os_str().to_owned()],
                &ProcessLimits::default(),
                &Cancellation::default(),
            )
            .unwrap();
        measured_source("author_fixture", &authored.stats);
        let mut d = Document::new("retained source cold publication");
        let id = d.id;
        d.page_unit = om9_core::geometry::Unit::Millimeter;
        let mut c = TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap();
        let mut options = ImportOptions::default();
        if matches!(mode, "clean-unitless" | "clean-unset") {
            options.units.model_mm_per_unit = Some(25.4);
            options.units.page_mm_per_unit = Some(7.);
            for n in [30, 31, 32] {
                options.units.definition_mm_per_unit.insert(
                    uuid::Uuid::parse_str(&format!("20000000-0000-4000-8000-{n:012x}")).unwrap(),
                    25.4,
                );
            }
        }
        let token = Cancellation::default();
        let imported = e
            .prepare_import(
                &c,
                ImportCapture::target(&c, id).unwrap(),
                &source,
                &temp.0,
                &options,
                &token,
            )
            .unwrap();
        measured_source("project_source", &imported.receipt().native);
        let (prepared, receipt) = imported.prepare(&c, &token).unwrap();
        c.commit(prepared).unwrap();
        let namespace = receipt.namespace;
        std::fs::remove_file(&source).unwrap();
        let before = c.snapshot();
        let capture = ExportCapture::all(&c, id).unwrap();
        let inputs = e
            .prepare_merge_inputs(
                &c,
                &capture,
                Some(namespace),
                &temp.0,
                &token,
                Usage::default(),
            )
            .unwrap();
        for stats in inputs.native_stats().values() {
            measured_source("prepare_retained_source", stats);
        }
        let wire = inputs
            .output_wire_plan(&c, &token, Usage::default())
            .unwrap();
        let stage = wire
            .stage_retained_request(&c, &temp.0, &token, Usage::default())
            .unwrap();
        let written = e
            .write_retained_private(&c, &stage, &token, Usage::default())
            .unwrap();
        measured_source("write_retained", &written.receipt().native);
        let validated = e
            .inspect_retained_private(&c, written, &token, Usage::default())
            .unwrap_or_else(|err| panic!("{mode}: {err}"));
        measured_source("inspect_retained_current", validated.inspect_stats());
        assert_eq!(validated.proof().represented_bounds_checked(), 0);
        assert_eq!(validated.proof().native_only_bounds_checked(), 0);
        let expected = validated.proof().output_sha256();
        let target = temp.0.join("target.3dm");
        std::fs::write(&target, b"old target sentinel").unwrap();
        let losses = validated.losses().unwrap();
        assert_eq!(losses.om9_resources, capture.document().resources.len());
        let receipt = validated
            .publish(&c, &target, &token, Some(losses), Usage::default())
            .unwrap();
        assert_eq!(receipt.proof.output_sha256(), expected);
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(std::fs::read(&target).unwrap())),
            expected
        );
        assert_eq!(c.snapshot(), before);
        drop(stage);
        assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
        println!("OM9_RETAINED_CURRENT_SOURCE {mode}");
    }
}

#[test]
#[ignore = "requires pinned native writer and SDK authored source fixture"]
fn real_current_edits_deletions_and_additions_cold_verify_native_frames_and_dirty_bounds() {
    use om9_core::{
        model::{Definition, Layer},
        retained::{HostRef, RecordDisposition},
    };
    let e = exchange();
    let fixture_path =
        PathBuf::from(std::env::var_os("OM9_SOURCE_FIXTURE").expect("pinned source fixture"));
    let fixture = NativeProgram::admit(
        &fixture_path,
        Sha256::digest(std::fs::read(&fixture_path).unwrap()).into(),
    )
    .unwrap();
    for case in [
        "edit-member-mm",
        "edit-member-mixed",
        "delete-root",
        "new-definition-layer",
        "patch-root-attributes",
    ] {
        let temp = Temp::new();
        let source = temp.0.join("source.3dm");
        let mode = if case == "edit-member-mixed" {
            "clean-mixed"
        } else {
            "clean-mm"
        };
        let output = fixture
            .run(
                &[mode.into(), source.as_os_str().to_owned()],
                &ProcessLimits::default(),
                &Cancellation::default(),
            )
            .unwrap();
        measured_source("author_fixture", &output.stats);
        let initial = Document::new("current retained edits");
        let id = initial.id;
        let mut original = TransactionCoordinator::new(
            World::new(vec![initial], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap();
        let token = Cancellation::default();
        let imported = e
            .prepare_import(
                &original,
                ImportCapture::target(&original, id).unwrap(),
                &source,
                &temp.0,
                &ImportOptions::default(),
                &token,
            )
            .unwrap();
        measured_source("project_source", &imported.receipt().native);
        let (prepared, receipt) = imported.prepare(&original, &token).unwrap();
        original.commit(prepared).unwrap();
        std::fs::remove_file(&source).unwrap();
        let ns = receipt.namespace;
        // Fixture bootstrap of current member/layer edits. Actual original
        // bytes and import authority still come from the pinned native reader;
        // transaction acceptance is exercised by the separate core suites.
        let mut d = original.world().documents[&id].as_ref().clone();
        let number = if case.starts_with("edit-member") {
            40
        } else if case == "delete-root" {
            13
        } else {
            10
        };
        let native_id =
            uuid::Uuid::parse_str(&format!("20000000-0000-4000-8000-{number:012x}")).unwrap();
        let row = d
            .import_namespaces
            .get_mut(&ns)
            .unwrap()
            .records
            .iter_mut()
            .find(|r| r.key.id == native_id)
            .unwrap();
        let host = row.host.unwrap();
        if case.starts_with("edit-member") {
            row.disposition = RecordDisposition::Edited;
            let HostRef::DefinitionMember { definition, object } = host else {
                panic!("member binding")
            };
            let def = d.definitions.get_mut(&definition).unwrap();
            let member = def.members.iter_mut().find(|m| m.id == object).unwrap();
            member.geometry = Geometry::Point([101., 202., 303.]);
            member.geometry_revision += 1;
            def.revision += 1;
            d.import_namespaces
                .get_mut(&ns)
                .unwrap()
                .records
                .iter_mut()
                .find(|r| r.host == Some(HostRef::Definition(definition)))
                .unwrap()
                .disposition = RecordDisposition::Edited;
        } else if case == "delete-root" {
            row.disposition = RecordDisposition::Deleted;
            let HostRef::Object(object) = host else {
                panic!("root binding")
            };
            d.entities.remove(&object).unwrap();
        } else if case == "patch-root-attributes" {
            row.disposition = RecordDisposition::Edited;
            let HostRef::Object(object) = host else {
                panic!("root binding")
            };
            let entity = d.entities.get_mut(&object).unwrap();
            entity.label = "edited current label".into();
            entity.attributes.visible = true;
            entity.attributes.locked = true;
            entity.attributes.color = Some([7, 8, 9, 255]);
            entity.attributes_revision += 1;
        } else {
            let layer = Layer::new("new writer layer", [8, 16, 32, 255]);
            let lid = layer.id;
            d.layer_order.push(lid);
            d.layers.insert(lid, layer);
            let def = Definition::new(
                "new writer definition",
                vec![Entity::new(
                    "new definition member",
                    Geometry::Point([7., 8., 9.]),
                    Some(lid),
                )],
            );
            let did = def.id;
            d.definitions.insert(did, def);
            let root = Entity::new("new writer instance", Geometry::Instance(did), Some(lid));
            d.entities.insert(root.id, root);
        }
        let c = TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap();
        let before = c.snapshot();
        let capture = ExportCapture::all(&c, id).unwrap();
        let inputs = e
            .prepare_merge_inputs(&c, &capture, Some(ns), &temp.0, &token, Usage::default())
            .unwrap();
        for stats in inputs.native_stats().values() {
            measured_source("prepare_retained_source", stats);
        }
        let wire = inputs
            .output_wire_plan(&c, &token, Usage::default())
            .unwrap();
        let stage = wire
            .stage_retained_request(&c, &temp.0, &token, Usage::default())
            .unwrap();
        let written = e
            .write_retained_private(&c, &stage, &token, Usage::default())
            .unwrap_or_else(|err| panic!("{case}: write: {err}"));
        measured_source("write_retained", &written.receipt().native);
        let validated = e
            .inspect_retained_private(&c, written, &token, Usage::default())
            .unwrap_or_else(|err| panic!("{case}: inspect: {err}"));
        measured_source("inspect_retained_current", validated.inspect_stats());
        if case.starts_with("edit-member") {
            // Changed leaf, two ancestor definitions and two root instances.
            assert_eq!(validated.proof().represented_bounds_checked(), 5, "{case}");
        } else if case == "new-definition-layer" {
            assert_eq!(validated.proof().represented_bounds_checked(), 2, "{case}");
        } else {
            assert_eq!(validated.proof().represented_bounds_checked(), 0, "{case}");
        }
        assert_eq!(validated.proof().native_only_bounds_checked(), 0, "{case}");
        let target = temp.0.join("target.3dm");
        std::fs::write(&target, b"old target sentinel").unwrap();
        let losses = validated.losses().unwrap();
        validated
            .publish(&c, &target, &token, Some(losses), Usage::default())
            .unwrap();
        assert_eq!(c.snapshot(), before);
        drop(stage);
        assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
        println!("OM9_RETAINED_CURRENT_EDIT {case}");
    }
}
