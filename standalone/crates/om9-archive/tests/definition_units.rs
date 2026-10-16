// SPDX-License-Identifier: LGPL-2.1-or-later
//! Actual cold container ownership; authored bytes do not claim native parsing.
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    identity::*,
    model::Document,
    resource::{Resource, ResourceKind},
    retained::{
        DefinitionUnits, ImportNamespace, NativeRecordKey, NativeRecordKind, NativeUnits,
        ResolvedBy, RetainedSource, SourceRecord, SourceUnits,
    },
    transaction::Cancellation,
};
use std::{collections::BTreeMap, path::PathBuf, process::Command};
use uuid::Uuid;

const BYTES: &[u8] =
    b"authored opaque unit ownership fixture; independent native tests own correspondence";
fn definitions() -> BTreeMap<Uuid, DefinitionUnits> {
    [
        (
            Uuid::from_u128(1),
            DefinitionUnits {
                native_enum: 0,
                name: "None".into(),
                declared_mm_per_unit: None,
                millimeters_per_unit: Some(25.4),
                resolved_by: Some(ResolvedBy::Explicit),
            },
        ),
        (
            Uuid::from_u128(2),
            DefinitionUnits {
                native_enum: 8,
                name: "Inches".into(),
                declared_mm_per_unit: Some(25.4),
                millimeters_per_unit: Some(25.4),
                resolved_by: Some(ResolvedBy::Declared),
            },
        ),
        (
            Uuid::from_u128(3),
            DefinitionUnits {
                native_enum: 255,
                name: "Unset".into(),
                declared_mm_per_unit: None,
                millimeters_per_unit: None,
                resolved_by: None,
            },
        ),
    ]
    .into()
}
fn document() -> Document {
    let mut d = Document::new("durable local definition units");
    let r = Resource::from_bytes(
        "external.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        BYTES,
        &Limits::default(),
    )
    .unwrap();
    let model = NativeUnits {
        native_enum: 2,
        name: "Millimeters".into(),
        declared_mm_per_unit: Some(1.),
        millimeters_per_unit: 1.,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.01,
        angular_tolerance_radians: 0.01,
        relative_tolerance: 0.01,
    };
    let source = RetainedSource {
        id: SourceId::new(),
        resource: r.id,
        sha256: r.sha256(),
        archive_version: 50,
        units: SourceUnits {
            model: model.clone(),
            page: model,
            definitions: definitions(),
        },
    };
    let records = source
        .units
        .definitions
        .keys()
        .enumerate()
        .map(|(i, id)| {
            SourceRecord::retained(
                NativeRecordKey {
                    kind: NativeRecordKind::Definition,
                    id: *id,
                    native_index: Some(i as i32),
                },
                "ON_InstanceDefinition",
            )
        })
        .collect();
    let ns = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: None,
        records,
    };
    d.resources.insert(r.id, r);
    d.retained_sources.insert(source.id, source);
    d.import_namespaces.insert(ns.id, ns);
    d
}
#[test]
fn cold_archive_reopens_explicit_declared_and_unresolved_definition_units_without_external_source()
{
    const ENV: &str = "OM9_DEFINITION_UNITS_COLD_FIXTURE";
    if let Some(path) = std::env::var_os(ENV) {
        let restored = Archive::open_restored(
            &PathBuf::from(path),
            &Limits::default(),
            &Default::default(),
        )
        .unwrap();
        assert!(restored.diagnostics.is_empty());
        let d = restored.document;
        assert_eq!(
            d.retained_sources
                .values()
                .next()
                .unwrap()
                .units
                .definitions,
            definitions()
        );
        assert_eq!(
            d.resources.values().next().unwrap().payload().unwrap(),
            BYTES
        );
        assert_eq!(
            d.import_namespaces.values().next().unwrap().records.len(),
            3
        );
        return;
    }
    let dir = std::env::temp_dir().join(format!("om9-definition-cold-{}", Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let external = dir.join("external.3dm");
    let file = dir.join("owned.om9");
    std::fs::write(&external, BYTES).unwrap();
    let d = document();
    Archive::save(&file, &d, &Limits::default(), &Cancellation::default()).unwrap();
    std::fs::remove_file(&external).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cold_archive_reopens_explicit_declared_and_unresolved_definition_units_without_external_source", "--nocapture"])
        .env(ENV, &file).output().unwrap();
    std::fs::remove_file(&file).unwrap();
    std::fs::remove_dir(&dir).unwrap();
    assert!(
        output.status.success(),
        "cold reader failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
#[test]
fn definition_units_roundtrip_preserves_native_and_source_identity_in_schema4() {
    let d = document();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    let reopened = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(reopened, d);
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 4);
}
