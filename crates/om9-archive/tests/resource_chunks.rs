// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::Archive;
use om9_core::{
    Error,
    budget::Limits,
    model::Document,
    resource::{Resource, ResourceKind},
};
use sha2::{Digest, Sha256};

fn document() -> Document {
    let mut d = Document::new("embedded resource owner");
    for (name, data) in [
        ("all byte values", (0..=255).collect::<Vec<u8>>()),
        ("empty blob", vec![]),
    ] {
        let r = Resource::from_bytes(name, ResourceKind::OpaqueBinary, &data, &Limits::default())
            .unwrap();
        d.resources.insert(r.id, r);
    }
    d
}
fn pack(version: u32, metadata: &[u8], blobs: &[([u8; 16], &[u8])]) -> Vec<u8> {
    let count = 1 + blobs.len();
    let mut out = vec![0; 16 + 72 * count];
    out[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    out[8..12].copy_from_slice(&version.to_le_bytes());
    out[12..16].copy_from_slice(&(count as u32).to_le_bytes());
    for (i, (kind, id, bytes)) in std::iter::once((1u32, [0u8; 16], metadata))
        .chain(blobs.iter().map(|(id, data)| (2u32, *id, *data)))
        .enumerate()
    {
        let entry = 16 + 72 * i;
        let start = out.len() as u64;
        out[entry..entry + 4].copy_from_slice(&kind.to_le_bytes());
        out[entry + 8..entry + 24].copy_from_slice(&id);
        out[entry + 24..entry + 32].copy_from_slice(&start.to_le_bytes());
        out[entry + 32..entry + 40].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
        out[entry + 40..entry + 72].copy_from_slice(&Sha256::digest(bytes));
        out.extend_from_slice(bytes);
    }
    out
}

#[test]
fn schema4_resources_are_binary_chunks_and_hydrated_exactly_without_external_paths() {
    let d = document();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 4);
    assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 3);
    let decoded = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(decoded, d);
    let a = decoded
        .resources
        .values()
        .find(|r| r.byte_length() == 256)
        .unwrap();
    assert_eq!(a.payload().unwrap(), &(0..=255).collect::<Vec<u8>>());
    assert!(
        decoded
            .resources
            .values()
            .any(|r| r.payload().unwrap().is_empty())
    );
    let metadata = serde_json::to_string(&d).unwrap();
    assert!(!metadata.contains("payload"));
}

#[test]
fn missing_or_unreferenced_resource_chunks_reject_even_with_valid_container_checksums() {
    let d = document();
    let metadata = serde_json::to_vec(&d).unwrap();
    assert!(matches!(
        Archive::decode(&pack(4, &metadata, &[]), &Limits::default()),
        Err(Error::Missing(_))
    ));
    let no_resources = Document::new("no resources");
    let metadata = serde_json::to_vec(&no_resources).unwrap();
    let id = *uuid::Uuid::new_v4().as_bytes();
    assert_eq!(
        Archive::decode(&pack(4, &metadata, &[(id, b"orphan")]), &Limits::default()),
        Err(Error::Invalid("unreferenced resource chunk"))
    );
}

#[test]
fn mismatched_resource_identity_length_hash_and_duplicates_cannot_be_hydrated() {
    let mut d = Document::new("references");
    let resource = Resource::from_bytes(
        "one",
        ResourceKind::OpaqueBinary,
        b"original",
        &Limits::default(),
    )
    .unwrap();
    let id = *resource.id.0.as_bytes();
    d.resources.insert(resource.id, resource);
    let metadata = serde_json::to_vec(&d).unwrap();
    for wrong in [b"other___".as_slice(), b"short".as_slice()] {
        assert!(Archive::decode(&pack(4, &metadata, &[(id, wrong)]), &Limits::default()).is_err());
    }
    let duplicate = pack(4, &metadata, &[(id, b"original"), (id, b"original")]);
    assert_eq!(
        Archive::decode(&duplicate, &Limits::default()),
        Err(Error::Invalid("duplicate chunk ID"))
    );
    let mut corrupt = Archive::encode(&d, &Limits::default()).unwrap();
    *corrupt.last_mut().unwrap() ^= 1;
    assert_eq!(
        Archive::decode(&corrupt, &Limits::default()),
        Err(Error::Invalid("chunk checksum"))
    );
}

#[test]
fn aggregate_container_plus_document_and_payload_copy_is_admitted_before_hydration() {
    let mut d = Document::new("resource budget");
    let r = Resource::from_bytes(
        "large",
        ResourceKind::OpaqueBinary,
        &[7; 64 * 1024],
        &Limits::default(),
    )
    .unwrap();
    d.resources.insert(r.id, r);
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    // Individually each side fits; their coexistence must reject.
    let limits = Limits {
        max_bytes: 100 * 1024,
        ..Limits::default()
    };
    assert!(bytes.len() < limits.max_bytes as usize);
    assert!(d.validate(&limits).is_ok());
    assert!(matches!(
        Archive::decode(&bytes, &limits),
        Err(Error::Budget(_))
    ));
}

#[test]
fn legacy_schema1_requires_explicit_migration_with_empty_resources_and_unchanged_ids() {
    let d = Document::new("schema1");
    let bytes = pack(1, &serde_json::to_vec(&d).unwrap(), &[]);
    assert!(matches!(
        Archive::decode(&bytes, &Limits::default()),
        Err(Error::Unsupported(_))
    ));
    let migrated = Archive::migrate_from_v1(&bytes, &Limits::default()).unwrap();
    assert_eq!((migrated.from_version, migrated.to_version), (1, 4));
    assert_eq!(
        Archive::decode(&migrated.bytes, &Limits::default()).unwrap(),
        d
    );
    let mut future = bytes;
    future[8..12].copy_from_slice(&99u32.to_le_bytes());
    assert!(matches!(
        Archive::migrate_from_v1(&future, &Limits::default()),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn schema1_migration_cannot_synthesize_payload_or_accept_hidden_resource_chunks() {
    let d = document();
    let bytes = pack(1, &serde_json::to_vec(&d).unwrap(), &[]);
    assert!(Archive::migrate_from_v1(&bytes, &Limits::default()).is_err());
    let d = Document::new("legacy");
    let id = *uuid::Uuid::new_v4().as_bytes();
    let bytes = pack(1, &serde_json::to_vec(&d).unwrap(), &[(id, b"data")]);
    assert!(Archive::migrate_from_v1(&bytes, &Limits::default()).is_err());
}

#[test]
fn schema2_migration_preserves_binary_resources_without_inventing_source_provenance() {
    let d = document();
    let mut bytes = Archive::encode(&d, &Limits::default()).unwrap();
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    assert!(matches!(
        Archive::decode(&bytes, &Limits::default()),
        Err(Error::Unsupported(_))
    ));
    let migrated = Archive::migrate_from_v2(&bytes, &Limits::default()).unwrap();
    assert_eq!((migrated.from_version, migrated.to_version), (2, 4));
    let restored = Archive::decode(&migrated.bytes, &Limits::default()).unwrap();
    assert_eq!(restored, d);
    assert!(restored.retained_sources.is_empty() && restored.import_namespaces.is_empty());
    for (id, r) in &d.resources {
        assert_eq!(
            restored.resources[id].payload().unwrap(),
            r.payload().unwrap()
        );
    }
}

#[test]
fn ordinary_recipe_option_names_are_not_misinterpreted_as_uuid_table_paths() {
    use om9_core::{
        geometry::{Geometry, Transform, Unit},
        history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
        model::Entity,
        request::Value,
    };
    let mut d = Document::new("valid option names");
    let output = Entity::new("output", Geometry::Point([1., 2., 3.]), None);
    let output_id = output.id;
    d.entities.insert(output.id, output);
    let mut recipe = Recipe::new(
        d.id,
        "opaque-recorded-fixture",
        vec![],
        vec![output_id],
        ReplayEnvironment {
            command_version: 1,
            solver: "fixture".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance {
                absolute_mm: 0.001,
                relative: 0.,
                angular_rad: 0.001,
            },
        },
    );
    recipe.state = EvaluationState::Detached;
    for key in [
        "resources",
        "entities",
        "layers",
        "definitions",
        "recipes",
        "results",
    ] {
        recipe
            .parameters
            .insert(key.into(), Value::Text("opaque".into()));
    }
    d.history.recipes.insert(recipe.id, recipe);
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    assert_eq!(Archive::decode(&bytes, &Limits::default()).unwrap(), d);
    om9_archive::admit_json(
        br#"{"resources":{"Text":"opaque"},"layers":{"Number":1}}"#,
        &Limits::default(),
    )
    .unwrap();
}
