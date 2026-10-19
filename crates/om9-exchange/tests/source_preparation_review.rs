// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent authored ABI cases; these bytes are not a native 3DM fixture.
use om9_core::{budget::Limits, identity::*, retained::*, transaction::Cancellation};
use om9_exchange::SourcePreparation;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use uuid::Uuid;
static LOCK: Mutex<()> = Mutex::new(());
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
struct Counting;
fn allocated(n: usize) {
    let live = LIVE.fetch_add(n, Ordering::SeqCst) + n;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            allocated(l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::SeqCst);
        unsafe { System.dealloc(p, l) };
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, n) };
        if !q.is_null() {
            LIVE.fetch_sub(l.size(), Ordering::SeqCst);
            allocated(n);
        }
        q
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn fixture() -> (ImportNamespace, RetainedSource, Vec<u8>, Value) {
    let bytes: Vec<u8> = (0..128).collect();
    let key = NativeRecordKey {
        kind: NativeRecordKind::Layer,
        id: Uuid::new_v4(),
        native_index: Some(1),
    };
    let units = NativeUnits {
        native_enum: 2,
        name: "millimeters".into(),
        declared_mm_per_unit: Some(1.),
        millimeters_per_unit: 1.,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.001,
        angular_tolerance_radians: 0.01,
        relative_tolerance: 0.01,
    };
    let source = RetainedSource {
        id: SourceId::new(),
        resource: ResourceId::new(),
        sha256: Sha256::digest(&bytes).into(),
        archive_version: 50,
        units: SourceUnits {
            model: units.clone(),
            page: units,
            definitions: Default::default(),
        },
    };
    let ns = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: Some(ImportMode::Preserve),
        records: vec![SourceRecord::retained(key, "ON_Layer")],
    };
    let unit = json!({"native_enum":2,"name":"millimeters","millimeters_per_unit":1.,"meters_per_unit":0.001,"requires_resolution":false,
        "absolute_tolerance":0.001,"angle_tolerance_radians":0.01,"relative_tolerance":0.01,"distance_display_mode":0,"distance_display_precision":3});
    let dto = json!({"schema_version":4,"status":"retained_source_prepared","decoder_policy":"om9-retained-source-v1","admission_required":true,"host_ready":false,
        "archive":{"version":50,"opennurbs_version":202609088,"file_bytes":128,"endmark_bytes":128,"source_sha256":digest(&bytes)},
        "original_units":{"model":unit,"page":unit},
        "records":[{"source":{"native_component_type":5,"id":key.id,"native_index":1},
            "wire":{"id":key.id,"serialized_index":7,"archive_reference_index":1,"sdk_destination_index":1,"table_type":0x10000011u32,"record_type":0x20008050u32,
                "offset":44,"byte_length":24,"sha256":digest(&bytes[44..68])},"native_kind":"ON_Layer","native_class_uuid":"95809813-e985-11d3-bfe5-0010830122f0",
            "name":"Sparse original layer","userdata":[],"attribute_userdata":[],"definition_units":null,
            "preservation_flags":{"master_userdata_present":false,"attribute_userdata_present":false,"unknown_userdata_present":false,"unrepresented_record_chunks_present":false,
                "opaque_identity_remap_requires_admission":false,"opaque_length_remap_requires_admission":false,"selected_serializer_context_required":false}}],
        "top_level_spans":[{"table_type":0,"offset":0,"byte_length":32,"sha256":digest(&bytes[..32]),"anonymous_user_table":false},
            {"table_type":0x10000011u32,"offset":32,"byte_length":80,"sha256":digest(&bytes[32..112]),"anonymous_user_table":false},
            {"table_type":0x7fffu32,"offset":112,"byte_length":16,"sha256":digest(&bytes[112..]),"anonymous_user_table":false}]});
    (ns, source, bytes, dto)
}
fn admit(
    ns: &ImportNamespace,
    source: &RetainedSource,
    bytes: &[u8],
    dto: &Value,
) -> om9_core::Result<SourcePreparation> {
    SourcePreparation::decode(
        &serde_json::to_vec(dto).unwrap(),
        ns,
        source,
        bytes,
        &Limits::default(),
        &Cancellation::default(),
    )
}
#[test]
fn strict_userdata_discriminants_payload_oracle_and_copycount_zero() {
    let _serial = LOCK.lock().unwrap();
    let (ns, source, bytes, original) = fixture();
    let known = json!({
        "application_id":Uuid::from_u128(101), "class_id":Uuid::from_u128(102),
        "item_id":Uuid::from_u128(103), "copy_count":0, "native_sizeof_bytes":64,
        "unknown":false, "opaque_references":true
    });
    let mut unknown = known.clone();
    unknown["unknown"] = json!(true);
    unknown["archive_version"] = json!(50);
    unknown["opennurbs_version"] = json!(202609088);
    unknown["byte_length"] = json!(6);
    unknown["payload_sha1"] = json!("00".repeat(20));
    for userdata in [known.clone(), unknown.clone()] {
        let mut dto = original.clone();
        dto["records"][0]["userdata"] = json!([userdata]);
        dto["records"][0]["preservation_flags"]["master_userdata_present"] = json!(true);
        dto["records"][0]["preservation_flags"]["unknown_userdata_present"] =
            dto["records"][0]["userdata"][0]["unknown"].clone();
        dto["records"][0]["preservation_flags"]["opaque_identity_remap_requires_admission"] =
            json!(true);
        dto["records"][0]["preservation_flags"]["opaque_length_remap_requires_admission"] =
            json!(true);
        admit(&ns, &source, &bytes, &dto)
            .expect("original copycount=0 attachment remains admitted");
        for failure in [
            "discriminant",
            "opaque",
            "size",
            "nil item",
            "oracle",
            "extra field",
        ] {
            let mut bad = dto.clone();
            let u = &mut bad["records"][0]["userdata"][0];
            match failure {
                "discriminant" => u["unknown"] = json!(!u["unknown"].as_bool().unwrap()),
                "opaque" => u["opaque_references"] = json!(false),
                "size" => u["native_sizeof_bytes"] = json!(0),
                "nil item" => u["item_id"] = json!(Uuid::nil()),
                "oracle" => {
                    if u["unknown"] == true {
                        u.as_object_mut().unwrap().remove("payload_sha1");
                    } else {
                        u["payload_sha1"] = json!("00".repeat(20));
                    }
                }
                _ => u["unrequested_payload_flag"] = json!(true),
            }
            assert!(admit(&ns, &source, &bytes, &bad).is_err(), "{failure}");
        }
    }
}
#[test]
fn unresolved_original_units_require_an_explicit_owned_resolution_and_complete_null_fields() {
    let _serial = LOCK.lock().unwrap();
    let (ns, mut source, bytes, mut dto) = fixture();
    let original = &mut dto["original_units"]["model"];
    original["native_enum"] = json!(0);
    original["name"] = json!("None");
    original["requires_resolution"] = json!(true);
    original["millimeters_per_unit"] = Value::Null;
    original["meters_per_unit"] = Value::Null;
    source.units.model.native_enum = 0;
    source.units.model.name = "None".into();
    source.units.model.declared_mm_per_unit = None;
    source.units.model.millimeters_per_unit = 25.4;
    source.units.model.resolved_by = ResolvedBy::Explicit;
    admit(&ns, &source, &bytes, &dto).unwrap();
    source.units.model.resolved_by = ResolvedBy::Declared;
    assert!(admit(&ns, &source, &bytes, &dto).is_err());
    source.units.model.resolved_by = ResolvedBy::Explicit;
    for field in ["millimeters_per_unit", "meters_per_unit"] {
        let mut bad = dto.clone();
        bad["original_units"]["model"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(admit(&ns, &source, &bytes, &bad).is_err(), "{field}");
    }
    let mut bad = dto.clone();
    bad["records"][0]["wire"]["id"] = json!(Uuid::new_v4());
    assert!(admit(&ns, &source, &bytes, &bad).is_err());
}
#[test]
fn escaped_near_limit_name_has_admitted_parser_and_typed_allocation_peak() {
    let _serial = LOCK.lock().unwrap();
    let (ns, source, bytes, mut dto) = fixture();
    dto["records"][0]["name"] = json!(format!("{}\"", "x".repeat(16 * 1024 - 1)));
    let packet = serde_json::to_vec(&dto).unwrap();
    drop(dto);
    // Find the actual generic admission boundary, independent of its estimates.
    let (mut low, mut high) = (0u64, 1024 * 1024u64);
    while low < high {
        let middle = low + (high - low) / 2;
        let l = Limits {
            max_bytes: middle,
            ..Limits::default()
        };
        if om9_archive::admit_json(&packet, &l).is_ok() {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    let limits = Limits {
        max_bytes: low,
        ..Limits::default()
    };
    let baseline = LIVE.load(Ordering::SeqCst);
    PEAK.store(baseline, Ordering::SeqCst);
    let result = SourcePreparation::decode(
        &packet,
        &ns,
        &source,
        &bytes,
        &limits,
        &Cancellation::default(),
    );
    let allocation = PEAK.load(Ordering::SeqCst) - baseline + packet.len();
    assert!(
        allocation <= limits.max_bytes as usize,
        "source JSON parser/typed peak {allocation} exceeds boundary {} (admitted={})",
        limits.max_bytes,
        result.is_ok()
    );
    result.expect("the valid authored ABI packet fits its generic admission boundary");
}

#[test]
fn tiny_map_and_escaped_large_key_refuse_before_owned_key_and_set_growth() {
    let _serial = LOCK.lock().unwrap();
    let mut object = serde_json::Map::new();
    object.insert(format!("{}\"", "x".repeat(16 * 1024 - 1)), json!(0));
    let large = serde_json::to_vec(&Value::Object(object)).unwrap();
    for (packet, limit) in [
        (b"{\"a\":0}".to_vec(), 240u64),
        (large.clone(), 3 * large.len() as u64 + 512),
    ] {
        let limits = Limits {
            max_bytes: limit,
            ..Limits::default()
        };
        let baseline = LIVE.load(Ordering::SeqCst);
        PEAK.store(baseline, Ordering::SeqCst);
        let result = om9_archive::admit_json(&packet, &limits);
        let allocation = PEAK.load(Ordering::SeqCst) - baseline + packet.len();
        assert!(
            allocation <= limit as usize,
            "key/set refusal peak {allocation}, limit {limit}, admitted={}",
            result.is_ok()
        );
        assert!(matches!(result, Err(om9_core::Error::Budget("bytes"))));
    }
}
