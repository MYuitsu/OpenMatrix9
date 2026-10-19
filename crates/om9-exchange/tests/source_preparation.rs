// SPDX-License-Identifier: LGPL-2.1-or-later
//! Authored ABI admission cases. These bytes do not claim native 3DM parsing.
use om9_core::{budget::Limits, identity::*, retained::*, transaction::Cancellation};
use om9_exchange::SourcePreparation;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

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
fn sparse_wire_index_remains_distinct_from_reference_and_decoded_indices() {
    let (ns, source, bytes, dto) = fixture();
    let prepared = admit(&ns, &source, &bytes, &dto).unwrap();
    let wire = &prepared.record(ns.records[0].key).unwrap().wire;
    assert_eq!(wire.serialized_index, Some(7));
    assert_eq!(wire.archive_reference_index, Some(1));
    assert_eq!(wire.sdk_destination_index, Some(1));
    assert!(prepared.owned_usage().bytes > 0);
}
#[test]
fn changed_source_digest_record_hash_span_coverage_or_catalog_rejects() {
    let (ns, source, bytes, dto) = fixture();
    for variant in 0..8 {
        let mut altered = dto.clone();
        match variant {
            0 => altered["archive"]["source_sha256"] = json!("00".repeat(32)),
            1 => altered["records"][0]["wire"]["sha256"] = json!("00".repeat(32)),
            2 => altered["top_level_spans"][1]["offset"] = json!(33),
            3 => altered["records"][0]["source"]["native_index"] = json!(7),
            4 => altered["records"][0]["wire"]["table_type"] = json!(0x10000018u32),
            5 => altered["records"][0]["wire"]["offset"] = json!(120),
            6 => altered["records"].as_array_mut().unwrap().clear(),
            _ => {
                let duplicate = altered["records"][0].clone();
                altered["records"].as_array_mut().unwrap().push(duplicate);
            }
        }
        assert!(
            admit(&ns, &source, &bytes, &altered).is_err(),
            "variant {variant}"
        );
    }
}
#[test]
fn original_unit_declaration_and_definition_context_cannot_be_forged() {
    let (ns, mut source, bytes, dto) = fixture();
    source.units.model.native_enum = 8;
    assert!(admit(&ns, &source, &bytes, &dto).is_err());
    source.units.model.native_enum = 2;
    source.units.page.name = "other original unit".into();
    assert!(admit(&ns, &source, &bytes, &dto).is_err());
}
#[test]
fn strict_schema_duplicate_keys_budget_and_cancellation_reject() {
    let (ns, source, bytes, mut dto) = fixture();
    dto["host_ready"] = json!(true);
    assert!(admit(&ns, &source, &bytes, &dto).is_err());
    dto["host_ready"] = json!(false);
    dto["unrequested_field"] = json!(0);
    assert!(admit(&ns, &source, &bytes, &dto).is_err());
    dto.as_object_mut().unwrap().remove("unrequested_field");
    let packet = serde_json::to_vec(&dto).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        SourcePreparation::decode(&packet, &ns, &source, &bytes, &Limits::default(), &cancel),
        Err(om9_core::Error::Canceled)
    ));
    let limits = Limits {
        max_bytes: packet.len() as u64,
        ..Limits::default()
    };
    assert!(matches!(
        SourcePreparation::decode(
            &packet,
            &ns,
            &source,
            &bytes,
            &limits,
            &Cancellation::default()
        ),
        Err(om9_core::Error::Budget(_))
    ));
    let mut duplicate = packet.clone();
    duplicate.pop();
    duplicate.extend_from_slice(b",\"schema_version\":4}");
    assert!(
        SourcePreparation::decode(
            &duplicate,
            &ns,
            &source,
            &bytes,
            &Limits::default(),
            &Cancellation::default()
        )
        .is_err()
    );
    for field in [
        "serialized_index",
        "archive_reference_index",
        "sdk_destination_index",
    ] {
        let mut missing = dto.clone();
        missing["records"][0]["wire"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            admit(&ns, &source, &bytes, &missing).is_err(),
            "missing {field}"
        );
    }
    dto["records"][0]
        .as_object_mut()
        .unwrap()
        .remove("definition_units");
    assert!(admit(&ns, &source, &bytes, &dto).is_err());
}

#[cfg(windows)]
#[test]
fn prepared_inputs_keep_original_cancel_owner_after_success_without_native_children() {
    use om9_core::{
        model::Document,
        transaction::{TransactionCoordinator, World},
    };
    use om9_exchange::{ExportCapture, NativeExchange};
    use om9_native_owner::{NativeProgram, ProcessLimits};
    let d = Document::new("zero-source operation owner");
    let id = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    // A trusted test executable admits the program descriptor, but an empty
    // source catalog starts no native child and needs no SDK fixture.
    let path = std::env::current_exe().unwrap();
    let program =
        NativeProgram::admit(&path, Sha256::digest(std::fs::read(&path).unwrap()).into()).unwrap();
    let exchange = NativeExchange::new(program, ProcessLimits::default()).unwrap();
    let original = Cancellation::default();
    let prepared = exchange
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &original,
            Default::default(),
        )
        .unwrap();
    assert!(prepared.native_stats().is_empty());
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Default::default())
        .unwrap();
    let default = wire.default_layer().unwrap();
    assert_eq!(default.native_component_type, 5);
    assert_eq!(default.serialized_index, Some(0));
    assert_eq!(default.archive_reference_index, Some(0));
    assert_eq!(default.sdk_destination_index, Some(0));
    assert_eq!(wire.table_order(5), &[default.id]);
    assert!(wire.table_order(12).is_empty());
    assert!(matches!(
        prepared.output_wire_plan(
            &c,
            &Cancellation::default(),
            om9_core::budget::Usage {
                bytes: c.limits().max_bytes,
                ..Default::default()
            }
        ),
        Err(om9_core::Error::Budget(_))
    ));
    drop(wire);
    prepared.revalidate(&c, &Cancellation::default()).unwrap();
    original.cancel();
    assert!(matches!(
        prepared.revalidate(&c, &Cancellation::default()),
        Err(om9_core::Error::Canceled)
    ));
}
