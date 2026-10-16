// SPDX-License-Identifier: LGPL-2.1-or-later
//! Authored cold ABI reports over toy bytes: no native 3DM parser claim.
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Geometry, Transform, Unit},
    model::{Document, Entity},
    retained::HostRef,
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{
    ExpectedRetainedOutput, ExportCapture, NativeExchange, WireOutputPlan, retained_cold,
};
use om9_native_owner::{NativeProgram, ProcessLimits};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs,
    io::Read,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TEST: Mutex<()> = Mutex::new(());
fn added(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::SeqCst) + bytes;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            added(l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::SeqCst);
        unsafe { System.dealloc(p, l) };
    }
    unsafe fn realloc(&self, p: *mut u8, old: Layout, new: usize) -> *mut u8 {
        let result = unsafe { System.realloc(p, old, new) };
        if !result.is_null() {
            if new >= old.size() {
                added(new - old.size());
            } else {
                LIVE.fetch_sub(old.size() - new, Ordering::SeqCst);
            }
        }
        result
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn exchange() -> NativeExchange {
    let path = std::env::current_exe().unwrap();
    let mut f = fs::File::open(&path).unwrap();
    let mut hash = Sha256::new();
    let mut buf = [0; 8192];
    loop {
        let n = f.read(&mut buf).unwrap();
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
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let parent =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../build/standalone-validation");
        fs::create_dir_all(&parent).unwrap();
        let p = parent.join(format!("cold-review-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("output.3dm"));
        let _ = fs::remove_dir(&self.0);
    }
}
fn case(test: impl FnOnce(&WireOutputPlan<'_>, &fs::File, Value, ExpectedRetainedOutput)) {
    case_with_label_cancel("Current point", |wire, held, report, expected, _| {
        test(wire, held, report, expected)
    });
}
fn case_with_label_cancel(
    label: &str,
    test: impl FnOnce(&WireOutputPlan<'_>, &fs::File, Value, ExpectedRetainedOutput, &Cancellation),
) {
    let original = Cancellation::default();
    let temp = Temp::new();
    let mut d = Document::new("cold capture");
    d.page_unit = Unit::Inch;
    let mut e = Entity::new(label, Geometry::Point([1., 2., 3.]), None);
    e.placement = Transform::translation([5., 0., 0.]);
    let oid = e.id;
    d.entities.insert(e.id, e);
    let did = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, did).unwrap();
    let inputs = exchange()
        .prepare_merge_inputs(&c, &capture, None, &temp.0, &original, Default::default())
        .unwrap();
    assert!(inputs.native_stats().is_empty());
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Default::default())
        .unwrap();
    let request = wire
        .encode_retained_request(&c, &Cancellation::default(), Default::default())
        .unwrap();
    let bytes: Vec<u8> = (0..180).map(|x| x as u8).collect();
    fs::write(temp.0.join("output.3dm"), &bytes).unwrap();
    let held = fs::File::open(temp.0.join("output.3dm")).unwrap();
    // Native example supplies the documented rich preservation field shape;
    // identities, capture geometry and actual toy-byte spans are independently set.
    let mut report: Value = serde_json::from_str(TEMPLATE).unwrap();
    let default = wire.default_layer().unwrap();
    let object = wire.host(HostRef::Object(oid)).unwrap();
    report["actual"]["layers"][0]["id"] = json!(default.id);
    report["actual"]["layers"][0]["name"] = json!(wire.default_layer_name().unwrap());
    report["actual"]["layers"][0]["color"] = json!([128, 128, 128, 255]);
    report["actual"]["objects"][0]["id"] = json!(object.id);
    report["actual"]["objects"][0]["label"] = json!(label);
    report["actual"]["objects"][0]["geometry"] = json!({"Point":[6.,2.,3.]});
    report["actual"]["objects"][0]["attributes"]["layer"] = json!(default.id);
    report["actual"]["current_layer"]["id"] = json!(default.id);
    let top = |ty, offset: usize, n: usize| json!({"kind":"top_level","native_component_type":null,"id":null,"table_type":ty,"record_type":null,"offset":offset,"byte_length":n,"sha256":digest(&bytes[offset..offset+n])});
    let record = |ty, id, table, kind, offset: usize, n: usize| json!({"kind":"record","native_component_type":ty,"id":id,"table_type":table,"record_type":kind,"offset":offset,"byte_length":n,"sha256":digest(&bytes[offset..offset+n])});
    report["raw_spans"] = json!([
        top(0, 0, 32),
        top(0x10000011u32, 32, 64),
        top(0x10000013u32, 96, 64),
        top(0x7fffu32, 160, 20),
        record(5, default.id, 0x10000011u32, 0x20008050u32, 44, 32),
        record(12, object.id, 0x10000013u32, 0x20008070u32, 108, 32),
    ]);
    let expected = ExpectedRetainedOutput {
        request_sha256: Sha256::digest(&request).into(),
        output_sha256: Sha256::digest(&bytes).into(),
        output_bytes: bytes.len() as u64,
    };
    report["request_sha256"] = json!(digest(&request));
    report["output_sha256"] = json!(digest(&bytes));
    report["output_bytes"] = json!(bytes.len());
    test(&wire, &held, report, expected, &original);
}
fn verify(
    wire: &WireOutputPlan<'_>,
    held: &fs::File,
    report: &Value,
    expected: ExpectedRetainedOutput,
) -> om9_core::Result<om9_exchange::ColdRetainedProof> {
    retained_cold::verify(
        wire,
        &serde_json::to_vec(report).unwrap(),
        held,
        expected,
        &Limits::default(),
        &Cancellation::default(),
    )
}
#[test]
fn valid_cold_capture_geometry_and_actual_span_hashes_are_accepted() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case(|wire, held, report, expected| {
        verify(wire, held, &report, expected).unwrap();
    });
}
#[test]
fn false_geometry_identity_indices_layer_or_envelope_never_prove_current_output() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case(|wire, held, report, expected| {
        for variant in 0..12 {
            let mut bad = report.clone();
            match variant {
                0 => bad["actual"]["objects"][0]["geometry"]["Point"][0] = json!(1.),
                1 => bad["actual"]["objects"][0]["id"] = json!(uuid::Uuid::new_v4()),
                2 => bad["actual"]["objects"][0]["wire"]["sdk_destination_index"] = json!(0),
                3 => {
                    bad["actual"]["objects"][0]["attributes"]["layer"] = json!(uuid::Uuid::new_v4())
                }
                4 => bad["actual"]["objects"][0]["geometry"] = Value::Null,
                5 => bad["status"] = json!("written_retained_current"),
                6 => bad["host_ready"] = json!(true),
                7 => bad["checks"]["live_records"] = json!(1),
                8 => bad["actual"]["layers"][0]["color"] = json!([1, 2, 3, 255]),
                9 => bad["actual"]["current_layer"]["id"] = json!(uuid::Uuid::new_v4()),
                10 => bad["output_sha256"] = json!("00".repeat(32)),
                _ => bad["actual"]["objects"][0]["native_class_uuid"] = json!(uuid::Uuid::new_v4()),
            }
            assert!(
                verify(wire, held, &bad, expected).is_err(),
                "variant {variant}"
            );
        }
    });
}
#[test]
fn missing_duplicate_overlapping_or_falsified_raw_spans_reject() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case(|wire, held, report, expected| {
        for variant in 0..7 {
            let mut bad = report.clone();
            match variant {
                0 => {
                    bad["raw_spans"].as_array_mut().unwrap().pop();
                }
                1 => {
                    let duplicate = bad["raw_spans"][4].clone();
                    bad["raw_spans"].as_array_mut().unwrap().push(duplicate);
                }
                2 => bad["raw_spans"][1]["offset"] = json!(31),
                3 => bad["raw_spans"][4]["offset"] = json!(90),
                4 => bad["raw_spans"][4]["sha256"] = json!("00".repeat(32)),
                5 => bad["raw_spans"][5]["byte_length"] = json!(u64::MAX),
                _ => bad["raw_spans"][5]["table_type"] = json!(0x10000011u32),
            }
            assert!(
                verify(wire, held, &bad, expected).is_err(),
                "variant {variant}"
            );
        }
    });
}
#[test]
fn strict_keys_required_nulls_digest_spelling_budget_and_cancel_reject() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case(|wire, held, report, expected| {
        let mut bad = report.clone();
        bad["unrequested"] = json!(0);
        assert!(verify(wire, held, &bad, expected).is_err());
        let mut bad = report.clone();
        bad["raw_spans"][0].as_object_mut().unwrap().remove("id");
        assert!(verify(wire, held, &bad, expected).is_err());
        let mut bytes = serde_json::to_vec(&report).unwrap();
        bytes.pop();
        bytes.extend_from_slice(b",\"schema_version\":4}");
        assert!(
            retained_cold::verify(
                wire,
                &bytes,
                held,
                expected,
                &Limits::default(),
                &Cancellation::default()
            )
            .is_err()
        );
        let bytes = serde_json::to_vec(&report).unwrap();
        let l = Limits {
            max_bytes: bytes.len() as u64,
            ..Limits::default()
        };
        assert!(matches!(
            retained_cold::verify(wire, &bytes, held, expected, &l, &Cancellation::default()),
            Err(Error::Budget(_))
        ));
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(matches!(
            retained_cold::verify(wire, &bytes, held, expected, &Limits::default(), &cancel),
            Err(Error::Canceled)
        ));
    });
}

#[test]
fn malformed_rich_metadata_and_unrequested_new_master_properties_reject() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case(|wire, held, report, expected| {
        for variant in 0..5 {
            let mut bad = report.clone();
            match variant {
                0 => {
                    bad["actual"]["objects"][0]["preservation"]["display_mode_overrides"] =
                        json!([{"unrequested": 0}])
                }
                1 => {
                    bad["actual"]["objects"][0]["preservation"]["render_material_references"] =
                        json!([{"unrequested": 0}])
                }
                2 => {
                    bad["actual"]["layers"][0]["preservation"]["native_material_index"] = json!(0.5)
                }
                3 => {
                    bad["actual"]["objects"][0]["preservation"]["url"] =
                        json!("unrequested native property")
                }
                _ => bad["actual"]["layers"][0]["preservation"]["plot_weight_mm"] = json!(42.),
            }
            assert!(
                verify(wire, held, &bad, expected).is_err(),
                "variant {variant}"
            );
        }
    });
}

#[test]
fn nested_native_instance_ids_members_and_independent_leaf_bounds_match_capture() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    use om9_core::model::Definition;
    let temp = Temp::new();
    let mut d = Document::new("nested cold");
    d.page_unit = Unit::Inch;
    let mut point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    point.placement = Transform::translation([2., 0., 0.]);
    let point_id = point.id;
    let leaf = Definition::new("leaf", vec![point]);
    let leaf_id = leaf.id;
    let mut member = Entity::new("member", Geometry::Instance(leaf_id), None);
    member.placement = Transform::translation([5., 0., 0.]);
    let member_id = member.id;
    let parent = Definition::new("parent", vec![member]);
    let parent_id = parent.id;
    let mut root = Entity::new("root", Geometry::Instance(parent_id), None);
    root.placement = Transform([
        [-2e-12, 0., 0., 10e-12],
        [0., 3e-12, 0., 0.],
        [0., 0., 1e-12, 0.],
        [0., 0., 0., 1.],
    ]);
    let root_id = root.id;
    d.entities.insert(root_id, root);
    d.definitions.insert(leaf_id, leaf);
    d.definitions.insert(parent_id, parent);
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let inputs = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &temp.0,
            &Cancellation::default(),
            Default::default(),
        )
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Default::default())
        .unwrap();
    let packet = wire
        .encode_retained_request(&c, &Cancellation::default(), Default::default())
        .unwrap();
    let mut report: Value = serde_json::from_str(TEMPLATE).unwrap();
    let default = wire.default_layer().unwrap();
    report["actual"]["layers"][0]["id"] = json!(default.id);
    report["actual"]["layers"][0]["name"] = json!(wire.default_layer_name().unwrap());
    report["actual"]["layers"][0]["color"] = json!([128, 128, 128, 255]);
    report["actual"]["current_layer"]["id"] = json!(default.id);
    let object_template = report["actual"]["objects"][0].clone();
    let mut objects = std::collections::BTreeMap::new();
    let mut definitions = std::collections::BTreeMap::new();
    let d = capture.document();
    for host in &inputs.plan().overlay.additions {
        match *host {
            om9_exchange::HostValue::Entity { host, entity } => {
                let output = wire.host(host).unwrap();
                let mut o = object_template.clone();
                o["id"] = json!(output.id);
                o["label"] = json!(entity.label);
                o["is_definition_member"] = json!(matches!(host, HostRef::DefinitionMember { .. }));
                o["attributes"]["mode"] =
                    json!(if matches!(host, HostRef::DefinitionMember { .. }) {
                        3
                    } else {
                        0
                    });
                o["attributes"]["layer"] = json!(default.id);
                if let Geometry::Instance(id) = entity.geometry {
                    o["native_kind"] = json!("ON_InstanceRef");
                    o["native_class_uuid"] = json!("f9cfb638-b9d4-4340-87e3-c56e7865d96a");
                    o["geometry"] =
                        json!({"Instance":wire.host(HostRef::Definition(id)).unwrap().id});
                    o["placement"] = json!(entity.placement);
                } else if let Geometry::Point(p) = entity.geometry {
                    o["geometry"] = json!({"Point":entity.placement.point(p).unwrap()});
                }
                objects.insert(output.id, o);
            }
            om9_exchange::HostValue::Definition(def) => {
                let output = wire.host(HostRef::Definition(def.id)).unwrap();
                definitions.insert(output.id,json!({"id":output.id,"native_component_type":11,"native_kind":"ON_InstanceDefinition","native_class_uuid":"26f8bff6-2618-417f-a158-153d64a94989",
                    "wire":{"serialized_index":null,"archive_reference_index":output.archive_reference_index,"sdk_destination_index":output.sdk_destination_index},"label":def.label,
                    "members":def.members.iter().map(|e|wire.host(HostRef::DefinitionMember{definition:def.id,object:e.id}).unwrap().id).collect::<Vec<_>>(),
                    "units":{"native_enum":2,"name":"millimeters","millimeters_per_unit":1.,"meters_per_unit":0.001,"requires_resolution":false},"linked":false,
                    "preservation":{"description":"","url":"","url_tag":"","native_definition_type":1,"linked_file_path":"","has_linked_component_settings":false,"skip_nested_linked_definitions":false,"linked_component_appearance":0,"userdata":[]}}));
            }
            _ => {}
        }
    }
    report["actual"]["objects"] = Value::Array(
        wire.table_order(12)
            .iter()
            .map(|id| objects.remove(id).unwrap())
            .collect(),
    );
    report["actual"]["definitions"] = Value::Array(
        wire.table_order(11)
            .iter()
            .map(|id| definitions.remove(id).unwrap())
            .collect(),
    );
    report["objects"] = json!(3);
    report["definitions"] = json!(2);
    report["checks"]["live_records"] = json!(6);
    report["checks"]["bounds"] = json!(4);
    report["actual"]["bounds"] = json!([
        {"id":wire.host(HostRef::Definition(leaf_id)).unwrap().id,"native_component_type":11,"stored":[[3.,2.,3.],[3.,2.,3.]],"computed":[[3.,2.,3.],[3.,2.,3.]],"matched":true},
        {"id":wire.host(HostRef::Definition(parent_id)).unwrap().id,"native_component_type":11,"stored":[[8.,2.,3.],[8.,2.,3.]],"computed":[[8.,2.,3.],[8.,2.,3.]],"matched":true},
        {"id":wire.host(HostRef::DefinitionMember{definition:parent_id,object:member_id}).unwrap().id,"native_component_type":12,"stored":[[8.,2.,3.],[8.,2.,3.]],"computed":[[8.,2.,3.],[8.,2.,3.]],"matched":true},
        {"id":wire.host(HostRef::Object(root_id)).unwrap().id,"native_component_type":12,"stored":[[-6e-12,6e-12,3e-12],[-6e-12,6e-12,3e-12]],"computed":[[-6e-12,6e-12,3e-12],[-6e-12,6e-12,3e-12]],"matched":true},
    ]);
    let mut spans = vec![];
    let mut position = 0usize;
    let mut add_top = |ty, n: usize| {
        let start = position;
        position += n;
        spans.push(json!({"kind":"top_level","native_component_type":null,"id":null,"table_type":ty,"record_type":null,"offset":start,"byte_length":n,"sha256":""}));
        start
    };
    add_top(0, 32);
    let mut record_specs = vec![];
    for (ty, table, record) in [
        (5, 0x10000011u32, 0x20008050u32),
        (11, 0x10000021u32, 0x20008076u32),
        (12, 0x10000013u32, 0x20008070u32),
    ] {
        let start = add_top(table, 24 + 32 * wire.table_order(ty).len());
        for (i, id) in wire.table_order(ty).iter().enumerate() {
            record_specs.push((ty, *id, table, record, start + 12 + 32 * i));
        }
    }
    add_top(0x7fffu32, 20);
    let bytes: Vec<u8> = (0..position).map(|n| n as u8).collect();
    for span in &mut spans {
        let a = span["offset"].as_u64().unwrap() as usize;
        let n = span["byte_length"].as_u64().unwrap() as usize;
        span["sha256"] = json!(digest(&bytes[a..a + n]));
    }
    for (ty, id, table, record, start) in record_specs {
        spans.push(json!({"kind":"record","native_component_type":ty,"id":id,"table_type":table,"record_type":record,"offset":start,"byte_length":32,"sha256":digest(&bytes[start..start+32])}));
    }
    report["raw_spans"] = Value::Array(spans);
    let expected = ExpectedRetainedOutput {
        request_sha256: Sha256::digest(&packet).into(),
        output_sha256: Sha256::digest(&bytes).into(),
        output_bytes: bytes.len() as u64,
    };
    report["request_sha256"] = json!(digest(&packet));
    report["output_sha256"] = json!(digest(&bytes));
    report["output_bytes"] = json!(bytes.len());
    fs::write(temp.0.join("output.3dm"), &bytes).unwrap();
    let held = fs::File::open(temp.0.join("output.3dm")).unwrap();
    let proof = verify(&wire, &held, &report, expected).unwrap();
    assert_eq!(proof.represented_bounds_checked(), 4);
    assert_eq!(proof.native_only_bounds_checked(), 0);
    for variant in 0..6 {
        let mut bad = report.clone();
        match variant {
            0 => {
                bad["actual"]["bounds"][3]["computed"][0][0] = json!(123.);
                bad["actual"]["bounds"][3]["computed"][1][0] = json!(123.);
                bad["actual"]["bounds"][3]["stored"] =
                    bad["actual"]["bounds"][3]["computed"].clone();
            }
            1 => bad["actual"]["definitions"][0]["members"][0] = json!(point_id.0),
            2 => {
                let object = bad["actual"]["objects"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|o| o["is_definition_member"] == false)
                    .unwrap();
                object["geometry"]["Instance"] = json!(parent_id.0);
            }
            3 => bad["actual"]["definitions"][0]["units"]["millimeters_per_unit"] = json!(25.4),
            4 => {
                bad["actual"]["definitions"][0]["preservation"]["description"] =
                    json!("unrequested")
            }
            _ => {
                bad["actual"]["bounds"][3]["stored"] = json!([[0., 0., 0.], [0., 0., 0.]]);
                bad["actual"]["bounds"][3]["computed"] = json!([[0., 0., 0.], [0., 0., 0.]]);
            }
        }
        assert!(
            verify(&wire, &held, &bad, expected).is_err(),
            "nested variant {variant}"
        );
    }
    assert_eq!(d.definitions.len(), 2);
}

#[test]
fn original_and_fresh_cancellation_remain_bound_to_cold_acceptance() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case_with_label_cancel("Current point", |wire, held, report, expected, original| {
        let response = serde_json::to_vec(&report).unwrap();
        let fresh = Cancellation::default();
        retained_cold::verify(wire, &response, held, expected, &Limits::default(), &fresh).unwrap();
        original.cancel();
        assert!(matches!(
            retained_cold::verify(wire, &response, held, expected, &Limits::default(), &fresh),
            Err(Error::Canceled)
        ));
    });
    case(|wire, held, report, expected| {
        let response = serde_json::to_vec(&report).unwrap();
        let fresh = Cancellation::default();
        fresh.cancel();
        assert!(matches!(
            retained_cold::verify(wire, &response, held, expected, &Limits::default(), &fresh),
            Err(Error::Canceled)
        ));
    });
}
#[test]
fn nullable_relationship_uuid_aliases_are_rejected_before_acceptance() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    case(|wire, held, report, expected| {
        for variant in 0..3 {
            let mut bad = report.clone();
            let v = match variant {
                0 => &mut bad["actual"]["current_layer"]["id"],
                1 => &mut bad["actual"]["objects"][0]["attributes"]["layer"],
                _ => &mut bad["raw_spans"][4]["id"],
            };
            *v = json!(v.as_str().unwrap().replace('-', ""));
            assert!(
                verify(wire, held, &bad, expected).is_err(),
                "UUID alias {variant}"
            );
        }
    });
}
#[test]
fn cold_report_acceptance_and_refusal_obey_measured_remaining_allocation_peak() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    // Parent reserves coordinator/prepared/wire owners before passing remaining
    // Limits; response is explicitly included in this verifier's remaining budget.
    for label in ["Current point".to_string(), "\u{00e9}".repeat(8000)] {
        case_with_label_cancel(&label, |wire, held, report, expected, _| {
            let response = serde_json::to_vec(&report).unwrap();
            drop(report);
            let mut lower = 0;
            let mut upper = 4 * 1024 * 1024;
            while lower < upper {
                let midpoint = lower + (upper - lower) / 2;
                let l = Limits {
                    max_bytes: midpoint,
                    ..Limits::default()
                };
                match retained_cold::verify(
                    wire,
                    &response,
                    held,
                    expected,
                    &l,
                    &Cancellation::default(),
                ) {
                    Ok(_) => upper = midpoint,
                    Err(Error::Budget(_)) => lower = midpoint + 1,
                    Err(_) => panic!("unexpected correspondence refusal"),
                }
            }
            assert!(lower > response.len() as u64);
            for (limit, accepted) in [(lower, true), (lower - 1, false)] {
                let baseline = LIVE.load(Ordering::SeqCst);
                PEAK.store(baseline, Ordering::SeqCst);
                let l = Limits {
                    max_bytes: limit,
                    ..Limits::default()
                };
                let result = retained_cold::verify(
                    wire,
                    &response,
                    held,
                    expected,
                    &l,
                    &Cancellation::default(),
                );
                let new_peak = PEAK.load(Ordering::SeqCst).saturating_sub(baseline) as u64;
                let total = response.capacity() as u64 + new_peak;
                assert!(total <= limit, "remaining peak {total} exceeds {limit}");
                if accepted {
                    result.unwrap();
                } else {
                    assert!(matches!(result, Err(Error::Budget(_))));
                }
            }
        });
    }
}

// Documented schema shape copied into source; toy bytes above are not native archives.
const TEMPLATE: &str = r#"{"actual":{"bounds":[],"current_layer":{"archive_reference_index":0,"id":"20000000-0000-4000-8000-000000000001","sdk_destination_index":0},"definitions":[],"layers":[{"color":[20,30,40,255],"id":"20000000-0000-4000-8000-000000000001","locked":false,"name":"Default","native_class_uuid":"95809813-e985-11d3-bfe5-0010830122f0","native_component_type":5,"native_kind":"ON_Layer","parent":null,"preservation":{"display_material_id":null,"expanded":true,"iges_level":-1,"native_line_pattern_index":-1,"native_material_index":-1,"per_viewport_settings":{"crc":0,"present":false},"persistent_locking":false,"persistent_visibility":true,"plot_color":[255,255,255,0],"plot_weight_mm":0.0,"render_material_references":[],"userdata":[]},"visible":true,"wire":{"archive_reference_index":0,"sdk_destination_index":0,"serialized_index":0}}],"model_units":{"absolute_tolerance":0.001,"angle_tolerance_radians":0.017453292519943295,"distance_display_mode":0,"distance_display_precision":3,"meters_per_unit":0.001,"millimeters_per_unit":1.0,"name":"millimeters","native_enum":2,"relative_tolerance":0.01,"requires_resolution":false},"objects":[{"attributes":{"color":null,"color_source":0,"display_order":0,"groups":[],"layer":"20000000-0000-4000-8000-000000000001","line_pattern_source":0,"locked":false,"material_source":0,"mode":0,"native_layer_index":0,"native_line_pattern_index":-1,"native_material_index":-1,"plot_weight_mm":0.0,"space":1,"userdata":[],"viewport_id":null,"visible":true},"geometry":{"Point":[25.4,50.8,0.0]},"id":"20000000-0000-4000-8000-00000000000a","is_definition_member":false,"label":"Current point","native_class_uuid":"c3101a1d-f157-11d3-bfe7-0010830122f0","native_component_type":12,"native_kind":"ON_Point","placement":[[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]],"preservation":{"casts_shadows":true,"clipping_plane_label_style":0,"color_source":0,"custom_linetype_present":false,"custom_section_style_present":false,"display_mode_overrides":[],"display_order":0,"groups":[],"hatch_background_fill":[255,255,255,0],"hatch_boundary_visible":false,"line_pattern_source":0,"linetype_scale":1.0,"material_source":0,"native_color":[255,255,255,0],"native_line_pattern_index":-1,"native_material_index":-1,"object_decoration":0,"object_frame":null,"plot_color":[0,0,0,255],"plot_color_source":0,"plot_weight_mm":0.0,"plot_weight_source":0,"receives_shadows":true,"render_material_references":[],"section_attributes_source":0,"space":1,"texture_mapping_channels":[],"url":"","userdata":[],"viewport_id":null,"wire_density":1},"wire":{"archive_reference_index":null,"sdk_destination_index":null,"serialized_index":null}}],"page_units":{"absolute_tolerance":0.001,"angle_tolerance_radians":0.017453292519943295,"distance_display_mode":0,"distance_display_precision":3,"meters_per_unit":0.0254,"millimeters_per_unit":25.4,"name":"inches","native_enum":8,"relative_tolerance":0.01,"requires_resolution":false}},"admission_required":true,"archive_version":50,"bounds_policy":"cold_leaf_hull_v1","checks":{"bounds":0,"live_records":2,"opaque_chunks":0,"raw_records":0,"raw_tables":0},"cold_validation_required":true,"definitions":0,"host_ready":false,"layers":1,"objects":1,"output_bytes":2709,"output_sha256":"25501039f460bd8b2ab191a9f0df5fef0f83f9e3c2d76b2e574afce4aa6f7022","primary_source_sha256":null,"raw_spans":[],"request_sha256":"d1401a7de8f3cdd73fc28537cff6def1e68d466bbd2e5ca90d561a50649f4f98","schema_version":4,"status":"validated_retained_current","writer_policy":"om9-retained-current-v1"}"#;
