// SPDX-License-Identifier: LGPL-2.1-or-later
//! Authored structural ownership fixtures; no native correspondence is claimed.
use om9_core::{
    Error,
    budget::Limits,
    identity::*,
    model::Document,
    resource::{Resource, ResourceKind},
    retained::{
        ImportNamespace, NativeRecordKey, NativeRecordKind, RetainedSource, SourceRecord,
        SourceUnits,
    },
};
use serde_json::{Value, json};
use uuid::Uuid;

fn units(definition: Uuid, context: Value) -> SourceUnits {
    let native = json!({
        "native_enum": 2, "name": "Millimeters", "declared_mm_per_unit": 1.,
        "millimeters_per_unit": 1., "resolved_by": "Declared",
        "absolute_tolerance": 0.01, "angular_tolerance_radians": 0.01,
        "relative_tolerance": 0.01
    });
    serde_json::from_value(json!({
        "model": native, "page": native,
        "definitions": {definition.to_string(): context}
    }))
    .expect("per-native-definition resolution must have durable typed ownership")
}
fn explicit() -> Value {
    json!({"native_enum": 0, "name": "None", "declared_mm_per_unit": null,
        "millimeters_per_unit": 25.4, "resolved_by": "Explicit"})
}
fn document(definition: Uuid, context: Value) -> Document {
    let mut d = Document::new("unit contexts");
    let resource = Resource::from_bytes(
        "source.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored retained bytes; not a native archive",
        &Limits::default(),
    )
    .unwrap();
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 50,
        units: units(definition, context),
    };
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: None,
        records: vec![SourceRecord::retained(
            NativeRecordKey {
                kind: NativeRecordKind::Definition,
                id: definition,
                native_index: Some(0),
            },
            "ON_InstanceDefinition",
        )],
    };
    d.resources.insert(resource.id, resource);
    d.retained_sources.insert(source.id, source);
    d.import_namespaces.insert(namespace.id, namespace);
    d
}
#[test]
fn explicit_definition_units_survive_owned_serialization_and_are_distinct_from_model_units() {
    let id = Uuid::new_v4();
    let d = document(id, explicit());
    d.validate(&Limits::default()).unwrap();
    let encoded = serde_json::to_value(d.retained_sources.values().next().unwrap()).unwrap();
    assert_eq!(encoded["units"]["model"]["millimeters_per_unit"], 1.);
    assert_eq!(
        encoded["units"]["definitions"][id.to_string()]["millimeters_per_unit"],
        25.4
    );
    let decoded: RetainedSource = serde_json::from_value(encoded).unwrap();
    assert_eq!(&decoded, d.retained_sources.values().next().unwrap());
}
#[test]
fn unresolved_definition_units_are_retained_without_fabricating_a_resolution() {
    let id = Uuid::new_v4();
    let mut context = explicit();
    context["millimeters_per_unit"] = Value::Null;
    context["resolved_by"] = Value::Null;
    let d = document(id, context);
    d.validate(&Limits::default()).unwrap();
    let value = serde_json::to_value(&d.retained_sources).unwrap();
    let context = &value[d.retained_sources.keys().next().unwrap().to_string()]["units"]["definitions"]
        [id.to_string()];
    assert!(context["millimeters_per_unit"].is_null());
    assert!(context["resolved_by"].is_null());
}
#[test]
fn incomplete_conflicting_or_nonpositive_definition_resolutions_reject() {
    for patch in [
        json!({"millimeters_per_unit": 0.}),
        json!({"millimeters_per_unit": -1.}),
        json!({"resolved_by": null}),
        json!({"millimeters_per_unit": null}),
        json!({"declared_mm_per_unit": 25.4}),
        json!({"resolved_by": "Declared"}),
        json!({"name": ""}),
    ] {
        let mut value = explicit();
        for (k, v) in patch.as_object().unwrap() {
            value[k] = v.clone();
        }
        let d = document(Uuid::new_v4(), value);
        assert!(matches!(
            d.validate(&Limits::default()),
            Err(Error::Invalid(_))
        ));
    }
}
#[test]
fn definition_resolution_requires_a_native_definition_in_its_source_namespace() {
    let id = Uuid::new_v4();
    let mut d = document(id, explicit());
    d.import_namespaces.values_mut().next().unwrap().records[0]
        .key
        .kind = NativeRecordKind::Layer;
    assert!(matches!(
        d.validate(&Limits::default()),
        Err(Error::Missing(_))
    ));
}
#[test]
fn definition_context_owned_name_capacity_is_admitted() {
    let id = Uuid::new_v4();
    let mut d = document(id, explicit());
    let baseline = d.validate(&Limits::default()).unwrap();
    let context = d
        .retained_sources
        .values_mut()
        .next()
        .unwrap()
        .units
        .definitions
        .get_mut(&id)
        .unwrap();
    context.name.reserve_exact(2 * 1024 * 1024);
    let limits = Limits {
        max_bytes: baseline.bytes + 1024 * 1024,
        ..Limits::default()
    };
    assert!(matches!(d.validate(&limits), Err(Error::Budget(_))));
}
#[test]
fn definition_unit_map_rejects_uuid_aliases_and_duplicate_keys_before_the_value() {
    let id = Uuid::from_u128(0xabcdef1234567890abcdef1234567890);
    let d = document(id, explicit());
    let source = d.retained_sources.values().next().unwrap();
    let canonical = serde_json::to_string(&source.units).unwrap();
    let duplicate = canonical.replace(
        &format!("\"{id}\":"),
        &format!("\"{id}\":{},\"{id}\":", explicit()),
    );
    assert!(
        serde_json::from_str::<SourceUnits>(&duplicate)
            .unwrap_err()
            .to_string()
            .contains("duplicate typed UUID")
    );
    for alias in [id.to_string().to_uppercase(), id.simple().to_string()] {
        let malformed = canonical.replace(&id.to_string(), &alias);
        assert!(serde_json::from_str::<SourceUnits>(&malformed).is_err());
    }
}
#[test]
fn earlier_source_units_without_definition_context_remain_compatible() {
    let d = document(Uuid::new_v4(), explicit());
    let mut value =
        serde_json::to_value(&d.retained_sources.values().next().unwrap().units).unwrap();
    value.as_object_mut().unwrap().remove("definitions");
    let decoded: SourceUnits = serde_json::from_value(value).unwrap();
    assert!(decoded.definitions.is_empty());
}
#[test]
fn nonfinite_declared_resolution_and_unresolved_bound_definitions_reject() {
    use om9_core::{
        geometry::Geometry,
        model::{Definition, Entity},
        retained::{HostRef, SourceRecord, fingerprint},
    };
    let id = Uuid::new_v4();
    let mut d = document(id, explicit());
    d.retained_sources
        .values_mut()
        .next()
        .unwrap()
        .units
        .definitions
        .get_mut(&id)
        .unwrap()
        .millimeters_per_unit = Some(f64::INFINITY);
    assert!(matches!(
        d.validate(&Limits::default()),
        Err(Error::Invalid(_))
    ));
    let context = d
        .retained_sources
        .values_mut()
        .next()
        .unwrap()
        .units
        .definitions
        .get_mut(&id)
        .unwrap();
    context.millimeters_per_unit = None;
    context.resolved_by = None;
    let def = Definition::new(
        "native local definition",
        vec![Entity::new("member", Geometry::Point([0.; 3]), None)],
    );
    let record = &mut d.import_namespaces.values_mut().next().unwrap().records[0];
    *record = SourceRecord::bound(
        record.key,
        "ON_InstanceDefinition",
        HostRef::Definition(def.id),
        fingerprint(&def).unwrap(),
    );
    d.definitions.insert(def.id, def);
    assert_eq!(
        d.validate(&Limits::default()),
        Err(Error::Invalid("bound definition units unresolved"))
    );
}
