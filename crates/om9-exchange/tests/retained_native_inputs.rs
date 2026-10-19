// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Real original-wire preparations through Rust's bounded operation owner.
use om9_core::{
    Error, budget::Limits, identity::*, model::Document, retained::NativeRecordKind, transaction::*,
};
use om9_exchange::{
    ExportCapture, ImportCapture, ImportOptions, NativeExchange, SourcePreparation,
};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use uuid::Uuid;
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let parent = PathBuf::from(
            std::env::var_os("OM9_NATIVE_IMPORT_SCRATCH").expect("explicit H-drive scratch"),
        );
        let path = parent.join(format!("om9-retained-inputs-{}", Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        for item in std::fs::read_dir(&self.0).unwrap() {
            let p = item.unwrap().path();
            if p.is_file() {
                let _ = std::fs::remove_file(p);
            }
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}
fn program(var: &str) -> NativeProgram {
    let path = PathBuf::from(std::env::var_os(var).expect("explicit authored native binary"));
    let hash = Sha256::digest(std::fs::read(&path).unwrap()).into();
    NativeProgram::admit(&path, hash).unwrap()
}
fn exchange() -> NativeExchange {
    NativeExchange::new(program("OM9_EXCHANGE_NATIVE"), ProcessLimits::default()).unwrap()
}
fn measured(operation: &str, stats: &om9_native_owner::ProcessStats) {
    println!(
        "OM9_RETAINED_INPUT_JOB {}",
        serde_json::json!({"operation":operation,"stats":{
            "pid":stats.pid,"peak_committed_bytes":stats.peak_committed_bytes,"active_processes":stats.active_processes,
            "exited":stats.exited,"elapsed_ms":stats.elapsed.as_millis()
        }})
    );
}
fn run(var: &str, args: &[std::ffi::OsString]) {
    let out = program(var)
        .run(args, &ProcessLimits::default(), &Cancellation::default())
        .unwrap();
    measured("author_fixture", &out.stats);
}
fn author(mode: &str, path: &Path) {
    run(
        "OM9_SOURCE_FIXTURE",
        &[mode.into(), path.as_os_str().to_owned()],
    );
}
fn import(
    c: &mut TransactionCoordinator,
    id: DocumentId,
    path: &Path,
    temp: &Path,
    options: &ImportOptions,
) -> ImportNamespaceId {
    let p = exchange()
        .prepare_import(
            c,
            ImportCapture::target(c, id).unwrap(),
            path,
            temp,
            options,
            &Cancellation::default(),
        )
        .unwrap();
    measured("project_source", &p.receipt().native);
    let (p, receipt) = p.prepare(c, &Cancellation::default()).unwrap();
    c.commit(p).unwrap();
    receipt.namespace
}
fn coordinator() -> (TransactionCoordinator, DocumentId) {
    let d = Document::new("retained preparation target");
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
fn explicit() -> ImportOptions {
    let mut options = ImportOptions::default();
    options.units.model_mm_per_unit = Some(25.4);
    options.units.page_mm_per_unit = Some(7.);
    for n in [30, 31, 32] {
        options.units.definition_mm_per_unit.insert(
            Uuid::parse_str(&format!("20000000-0000-4000-8000-{n:012x}")).unwrap(),
            25.4,
        );
    }
    options
}
fn prepare(
    c: &TransactionCoordinator,
    id: DocumentId,
    ns: ImportNamespaceId,
    temp: &Path,
) -> SourcePreparation {
    let capture = ExportCapture::all(c, id).unwrap();
    let before = c.snapshot();
    let input = exchange()
        .prepare_merge_inputs(
            c,
            &capture,
            Some(ns),
            temp,
            &Cancellation::default(),
            Default::default(),
        )
        .unwrap();
    assert_eq!(c.snapshot(), before);
    for stats in input.native_stats().values() {
        assert!(stats.exited && stats.active_processes == 0);
        measured("prepare_retained_source", stats);
    }
    input.revalidate(c, &Cancellation::default()).unwrap();
    let wire = input
        .output_wire_plan(c, &Cancellation::default(), Default::default())
        .unwrap();
    for original in input.source(ns).unwrap().records() {
        let key = original.key().unwrap();
        let mapped = wire
            .source(om9_exchange::ScopedNativeKey { namespace: ns, key })
            .unwrap()
            .unwrap();
        assert_eq!(mapped.id, original.wire.id);
        assert_eq!(mapped.serialized_index, original.wire.serialized_index);
        assert_eq!(
            mapped.archive_reference_index,
            original.wire.archive_reference_index
        );
        assert_eq!(
            mapped.sdk_destination_index,
            original.wire.sdk_destination_index
        );
    }
    wire.revalidate(c, &Cancellation::default()).unwrap();
    verify_staged_request(c, &wire, input.source(ns).unwrap(), ns, temp);
    drop(wire);
    let source = input.into_source(ns).unwrap();
    assert_eq!(
        source.source_sha256(),
        capture.document().retained_sources[&capture.document().import_namespaces[&ns].source]
            .sha256
    );
    assert_eq!(
        source.records().count(),
        capture.document().import_namespaces[&ns].records.len()
    );
    assert!(!std::fs::read_dir(temp).unwrap().any(|f| {
        f.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".om9-merge-source-")
    }));
    source
}
fn verify_staged_request(
    c: &TransactionCoordinator,
    wire: &om9_exchange::WireOutputPlan<'_>,
    prepared: &SourcePreparation,
    ns: ImportNamespaceId,
    temp: &Path,
) {
    let stage = wire
        .stage_retained_request(c, temp, &Cancellation::default(), Default::default())
        .unwrap();
    let bytes = std::fs::read(stage.request_path()).unwrap();
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&bytes)),
        stage.request_sha256()
    );
    let p: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(p["metadata_policy"], "primary_archive");
    assert_eq!(p["document"]["current_layer"]["policy"], "preserve_primary");
    assert_eq!(p["sources"].as_array().unwrap().len(), 1);
    let source = &p["sources"][0];
    let d = c.world().documents.values().next().unwrap();
    let original = &d.retained_sources[&d.import_namespaces[&ns].source];
    assert_eq!(
        source["model_mm_per_unit"],
        original.units.model.millimeters_per_unit
    );
    assert_eq!(
        source["page_mm_per_unit"],
        original.units.page.millimeters_per_unit
    );
    let rows = source["records"].as_array().unwrap();
    assert_eq!(rows.len(), d.import_namespaces[&ns].records.len());
    for record in &d.import_namespaces[&ns].records {
        let row = rows
            .iter()
            .find(|row| {
                row["source"]["id"] == record.key.id.to_string()
                    && row["source"]["native_index"] == serde_json::json!(record.key.native_index)
            })
            .unwrap();
        assert_eq!(row["action"], "retain");
        assert_eq!(row["output"]["id"], record.key.id.to_string());
        assert_eq!(row["expected_wire"]["id"], record.key.id.to_string());
        let expected = &prepared.record(record.key).unwrap().wire;
        assert_eq!(
            row["expected_wire"]["serialized_index"],
            serde_json::json!(expected.serialized_index)
        );
        assert_eq!(
            row["expected_wire"]["archive_reference_index"],
            serde_json::json!(expected.archive_reference_index)
        );
        assert_eq!(
            row["expected_wire"]["sdk_destination_index"],
            serde_json::json!(expected.sdk_destination_index)
        );
        assert_eq!(row["expected_wire"]["offset"], expected.offset);
        assert_eq!(row["expected_wire"]["byte_length"], expected.byte_length);
        let hash: String = expected
            .sha256
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(row["expected_wire"]["sha256"], hash);
        assert_eq!(row["current"].is_null(), record.host.is_none());
        if record.key.kind == NativeRecordKind::Definition {
            assert_eq!(
                row["definition_mm_per_unit"],
                original.units.definitions[&record.key.id]
                    .millimeters_per_unit
                    .unwrap()
            );
        }
    }
    let staged_source = stage
        .request_path()
        .parent()
        .unwrap()
        .join(format!("{}.3dm", ns.0));
    let mut hash = Sha256::new();
    use std::io::Read;
    let mut file = std::fs::File::open(&staged_source).unwrap();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    assert_eq!(<[u8; 32]>::from(hash.finalize()), original.sha256);
    drop(file);
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&staged_source)
            .is_err()
    );
    assert!(std::fs::remove_file(&staged_source).is_err());
    stage.revalidate(c, &Cancellation::default()).unwrap();
    let request = stage.request_path().to_owned();
    drop(stage);
    assert!(!request.exists() && !staged_source.exists());
}
#[test]
#[ignore = "requires pinned authored native source and retained fixtures"]
fn real_model_page_and_definition_units_match_original_wire_contexts() {
    for mode in [
        "clean-mm",
        "clean-inch",
        "clean-custom",
        "clean-mixed",
        "clean-unitless",
        "clean-unset",
    ] {
        let temp = Temp::new();
        let path = temp.0.join("source.3dm");
        author(mode, &path);
        let options = if matches!(mode, "clean-unitless" | "clean-unset") {
            explicit()
        } else {
            ImportOptions::default()
        };
        let (mut c, id) = coordinator();
        let ns = import(&mut c, id, &path, &temp.0, &options);
        std::fs::remove_file(&path).unwrap();
        prepare(&c, id, ns, &temp.0);
        println!("OM9_RETAINED_INPUT_SOURCE {mode}");
    }
}
#[test]
#[ignore = "requires pinned authored native source and retained fixtures"]
fn real_sparse_indices_lights_and_copycount_zero_attachments_keep_original_owners() {
    for mode in ["sparse", "light", "copycount-zero"] {
        let temp = Temp::new();
        let base = temp.0.join("base.3dm");
        let path = temp.0.join("source.3dm");
        author("mm", &base);
        run(
            "OM9_RETAINED_SOURCE_FIXTURE",
            &[
                mode.into(),
                base.into_os_string(),
                path.as_os_str().to_owned(),
            ],
        );
        let (mut c, id) = coordinator();
        let ns = import(&mut c, id, &path, &temp.0, &ImportOptions::default());
        std::fs::remove_file(&path).unwrap();
        let p = prepare(&c, id, ns, &temp.0);
        if mode == "sparse" {
            let layer = p
                .records()
                .find(|r| {
                    r.key().unwrap().kind == NativeRecordKind::Layer
                        && r.wire.serialized_index == Some(7)
                })
                .unwrap();
            assert_eq!(layer.wire.archive_reference_index, Some(1));
            assert_eq!(layer.wire.sdk_destination_index, Some(1));
            let group = p
                .records()
                .find(|r| r.key().unwrap().kind == NativeRecordKind::Group)
                .unwrap();
            assert_eq!(group.wire.serialized_index, Some(5));
            assert_eq!(group.wire.archive_reference_index, Some(5));
            assert_eq!(group.key().unwrap().native_index, Some(0));
        } else if mode == "light" {
            let light = p
                .records()
                .find(|r| r.key().unwrap().kind == NativeRecordKind::Light)
                .unwrap();
            assert_eq!(light.key().unwrap().native_index, None);
            assert_eq!(light.wire.serialized_index, Some(7));
            assert_eq!(light.wire.archive_reference_index, Some(0));
        } else {
            let style = p
                .records()
                .find(|r| r.key().unwrap().kind == NativeRecordKind::DimStyle)
                .unwrap();
            assert_eq!(style.native_kind, "ON_V5x_DimStyle");
            assert!(
                style.preservation_flags.master_userdata_present
                    && style.preservation_flags.unknown_userdata_present
            );
        }
        println!("OM9_RETAINED_INPUT_SOURCE {mode}");
    }
}
#[test]
#[ignore = "requires pinned authored native source and retained fixtures"]
fn cancel_budget_stale_and_wrong_coordinator_reject_before_child_or_staging() {
    let temp = Temp::new();
    let path = temp.0.join("source.3dm");
    author("clean-mm", &path);
    let (mut c, id) = coordinator();
    let ns = import(&mut c, id, &path, &temp.0, &ImportOptions::default());
    let capture = ExportCapture::all(&c, id).unwrap();
    let before = c.snapshot();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        exchange().prepare_merge_inputs(
            &c,
            &capture,
            Some(ns),
            &temp.0,
            &cancel,
            Default::default()
        ),
        Err(Error::Canceled)
    ));
    let held = om9_core::budget::Usage {
        bytes: c.limits().max_bytes,
        ..Default::default()
    };
    assert!(matches!(
        exchange().prepare_merge_inputs(
            &c,
            &capture,
            Some(ns),
            &temp.0,
            &Cancellation::default(),
            held
        ),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
    let (other, _) = coordinator();
    assert!(matches!(
        exchange().prepare_merge_inputs(
            &other,
            &capture,
            Some(ns),
            &temp.0,
            &Cancellation::default(),
            Default::default()
        ),
        Err(Error::WrongOwner)
    ));
    c.undo().unwrap();
    assert!(matches!(
        exchange().prepare_merge_inputs(
            &c,
            &capture,
            Some(ns),
            &temp.0,
            &Cancellation::default(),
            Default::default()
        ),
        Err(Error::Stale(_))
    ));
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
    println!("OM9_RETAINED_INPUT_SOURCE ownership");
}

#[test]
#[ignore = "requires pinned authored native source and retained fixtures"]
fn real_sparse_source_new_layer_definition_members_and_unassigned_output_have_separate_wire_slots()
{
    use om9_core::{
        geometry::Geometry,
        model::{Definition, Entity, Layer},
        retained::HostRef,
    };
    let temp = Temp::new();
    let base = temp.0.join("base.3dm");
    let path = temp.0.join("source.3dm");
    // The clean fixture deliberately removes the child layer. Use the full
    // fixture so the sparse writer actually emits its child at wire index 7.
    author("mm", &base);
    run(
        "OM9_RETAINED_SOURCE_FIXTURE",
        &[
            "sparse".into(),
            base.into_os_string(),
            path.as_os_str().to_owned(),
        ],
    );
    let (mut original, id) = coordinator();
    let ns = import(&mut original, id, &path, &temp.0, &ImportOptions::default());
    // Bootstrap a current-state fixture. Transaction publication is separately
    // tested; this exercises original native spans plus unbound current hosts.
    let mut d = original.world().documents[&id].as_ref().clone();
    let layer = Layer::new("OM9 Unassigned", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let member = Entity::new("new local member", Geometry::Point([7., 8., 9.]), Some(lid));
    let member_id = member.id;
    let def = Definition::new("new shared definition", vec![member]);
    let did = def.id;
    d.definitions.insert(did, def);
    let root = Entity::new("new unassigned C", Geometry::Point([4., 5., 6.]), None);
    let root_id = root.id;
    d.entities.insert(root_id, root);
    let instance = Entity::new("new shared instance", Geometry::Instance(did), Some(lid));
    let instance_id = instance.id;
    d.entities.insert(instance_id, instance);
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let before = c.snapshot();
    let inputs = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            Some(ns),
            &temp.0,
            &Cancellation::default(),
            Default::default(),
        )
        .unwrap();
    for stats in inputs.native_stats().values() {
        measured("prepare_retained_source", stats);
    }
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Default::default())
        .unwrap();
    let layer = wire.host(HostRef::Layer(lid)).unwrap();
    assert_eq!(
        inputs.plan().host_identities[&HostRef::Layer(lid)].native_index,
        Some(2)
    );
    assert_eq!(layer.serialized_index, Some(8));
    assert_eq!(layer.archive_reference_index, Some(2));
    assert_eq!(layer.sdk_destination_index, Some(2));
    let default = wire.default_layer().unwrap();
    assert_eq!(default.serialized_index, Some(9));
    assert_eq!(default.archive_reference_index, Some(3));
    assert_eq!(default.sdk_destination_index, Some(3));
    assert_ne!(wire.default_layer_name(), Some("OM9 Unassigned"));
    assert!(
        wire.default_layer_name()
            .unwrap()
            .starts_with("OM9 Unassigned ")
    );
    let definition = wire.host(HostRef::Definition(did)).unwrap();
    assert_eq!(definition.serialized_index, None);
    // The full source includes an unbound orphan definition. Retained rows
    // still occupy the native table even when they have no current host.
    assert_eq!(
        inputs
            .source(ns)
            .unwrap()
            .records()
            .filter(|row| row.key().unwrap().kind == NativeRecordKind::Definition)
            .count(),
        4
    );
    assert_eq!(definition.archive_reference_index, Some(4));
    assert_eq!(definition.sdk_destination_index, Some(4));
    for host in [
        HostRef::Object(root_id),
        HostRef::Object(instance_id),
        HostRef::DefinitionMember {
            definition: did,
            object: member_id,
        },
    ] {
        let geom = wire.host(host).unwrap();
        assert_eq!(geom.serialized_index, None);
        assert_eq!(geom.archive_reference_index, None);
        assert_eq!(geom.sdk_destination_index, None);
        assert_eq!(
            wire.table_order(12)
                .iter()
                .filter(|id| **id == geom.id)
                .count(),
            1
        );
    }
    assert_eq!(c.snapshot(), before);
    assert_eq!(wire.table_order(5).len(), 4);
    assert_eq!(wire.table_order(11).len(), 5);
    verify_staged_request(&c, &wire, inputs.source(ns).unwrap(), ns, &temp.0);
    println!("OM9_RETAINED_INPUT_SOURCE wire-additions");
}
