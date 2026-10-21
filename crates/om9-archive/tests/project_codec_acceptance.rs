// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-12 acceptance: the native project codec saves and reopens a complete
//! document (layers, entities, B-Rep payload, instance, group, feature, history
//! placeholder and an embedded jobbag reference) and refuses every damaged bundle
//! before a document is published.
//!
//! The B-Rep here carries a synthetic native source: the portable codec owns the
//! payload identity and the display snapshot, while real OCCT measurement is
//! exercised separately by `om9-app/tests/brep_selection.rs`.

use om9_archive::{Archive, ReplayKey};
use om9_core::{
    Error,
    brep::{Brep, BrepDisplay},
    budget::Limits,
    geometry::{Geometry, Transform, Unit},
    history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
    identity::{DefinitionId, GroupId, HistoryId, LayerId, ObjectId, ResourceId},
    model::{Definition, Document, Entity, Feature, FeatureKind, Group, Layer},
    resource::{Resource, ResourceKind},
    transaction::Cancellation,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use uuid::Uuid;

const MAGIC: &[u8; 8] = b"OM9\0\r\n\x1a\n";
const VERSION: u32 = 4;
const ENTRY: usize = 72;
const HEADER: usize = 16;

struct Fixture {
    document: Document,
    root: LayerId,
    detail: LayerId,
    box_id: ObjectId,
    instance_id: ObjectId,
    member_id: ObjectId,
    definition_id: DefinitionId,
    group_id: GroupId,
    feature_id: om9_core::identity::FeatureId,
    resource_id: ResourceId,
    recipe_id: HistoryId,
}

fn uuid() -> Uuid {
    Uuid::new_v4()
}

/// A 10 x 20 x 30 mm box: twelve outward-wound triangles (volume 6000 mm3) plus
/// the face/edge/vertex identity rows a real B-Rep snapshot carries.
fn box_brep() -> Brep {
    let points = vec![
        [0., 0., 0.],
        [10., 0., 0.],
        [10., 20., 0.],
        [0., 20., 0.],
        [0., 0., 30.],
        [10., 0., 30.],
        [10., 20., 30.],
        [0., 20., 30.],
    ];
    let triangles = vec![
        [0, 2, 1, 0],
        [0, 3, 2, 0],
        [4, 5, 6, 1],
        [4, 6, 7, 1],
        [0, 1, 5, 2],
        [0, 5, 4, 2],
        [2, 3, 7, 3],
        [2, 7, 6, 3],
        [1, 2, 6, 4],
        [1, 6, 5, 4],
        [0, 4, 7, 5],
        [0, 7, 3, 5],
    ];
    let segments = vec![
        [0, 1, 0],
        [1, 2, 1],
        [2, 3, 2],
        [3, 0, 3],
        [4, 5, 4],
        [5, 6, 5],
        [6, 7, 6],
        [7, 4, 7],
        [0, 4, 8],
        [1, 5, 9],
        [2, 6, 10],
        [3, 7, 11],
    ];
    let vertices = (0..8).collect();
    let faces = (0..6).map(|_| uuid()).collect();
    let edges = (0..12).map(|_| uuid()).collect();
    let vertex_ids = (0..8).map(|_| uuid()).collect();
    let face_edges = vec![
        [0, 0],
        [0, 1],
        [0, 2],
        [0, 3],
        [1, 4],
        [1, 5],
        [1, 6],
        [1, 7],
        [2, 0],
        [2, 8],
        [2, 4],
        [2, 9],
        [3, 2],
        [3, 10],
        [3, 6],
        [3, 11],
        [4, 1],
        [4, 9],
        [4, 5],
        [4, 10],
        [5, 3],
        [5, 11],
        [5, 7],
        [5, 8],
    ];
    let display = BrepDisplay {
        source_sha256: [0; 32],
        deflection_mm: 0.01,
        points,
        triangles,
        segments,
        vertices,
        faces,
        edges,
        vertex_ids,
        face_edges,
    };
    Brep::from_native(
        "CASCADE Topology V3 synthetic box 10x20x30".into(),
        display,
        &Limits::default(),
    )
    .expect("portable B-Rep snapshot")
}

/// Volume of the closed display triangle soup by the divergence theorem, so the
/// 6000 mm3 oracle does not depend on the OCCT adapter.
fn display_volume(display: &BrepDisplay) -> f64 {
    let mut six_volume = 0.0;
    for [a, b, c, _] in &display.triangles {
        let point = |index: u32| display.points[index as usize];
        let (a, b, c) = (point(*a), point(*b), point(*c));
        six_volume += a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);
    }
    six_volume / 6.0
}

fn fixture() -> Fixture {
    let mut document = Document::new("L2-01-12 acceptance project");
    let root = Layer::new("Body", [200, 120, 60, 255]);
    let mut detail = Layer::new("Details", [60, 120, 200, 255]);
    detail.parent = Some(root.id);
    detail.locked = true;
    let root_id = root.id;
    let detail_id = detail.id;
    document.layers.insert(root_id, root.clone());
    document.layers.insert(detail_id, detail.clone());
    document.layer_order = vec![root_id, detail_id];
    document.current_layer = Some(detail_id);

    let box_entity = Entity::new("Box 10x20x30", Geometry::Brep(box_brep()), Some(detail.id));
    let box_id = box_entity.id;
    document.entities.insert(box_id, box_entity);

    let member = Entity::new("gem seat", Geometry::Point([0., 0., 0.]), None);
    let member_id = member.id;
    let definition = Definition::new("gem blank", vec![member]);
    let definition_id = definition.id;
    document.definitions.insert(definition_id, definition);
    let mut instance = document
        .instance(
            definition_id,
            Transform::translation([12., 0., 0.]),
            Some(root.id),
        )
        .unwrap();
    instance.attributes.visible = false;
    instance.attributes.hidden = true;
    instance.attributes.locked = true;
    let instance_id = instance.id;
    document.entities.insert(instance_id, instance);

    let group = Group::new("cluster", vec![box_id, instance_id]);
    let group_id = group.id;
    document.groups.insert(group_id, group);

    let feature = Feature::new("jobbag marker", FeatureKind::Semantic, vec![box_id]);
    let feature_id = feature.id;
    document.features.insert(feature_id, feature);

    // Jobbag semantics belong to M01 L2-03-15. The codec only owns the bytes of
    // the reference, so the fixture stores it as an opaque embedded resource.
    let resource = Resource::from_bytes(
        "jobbag.reference",
        ResourceKind::OpaqueBinary,
        b"jobbag/A-102.om9",
        &Limits::default(),
    )
    .unwrap();
    let resource_id = resource.id;
    document.resources.insert(resource_id, resource);

    let recipe = Recipe::new(
        document.id,
        "future-builder-v9",
        vec![],
        vec![box_id],
        ReplayEnvironment {
            command_version: 9,
            solver: "future-solver".into(),
            solver_version: "999.0".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance {
                absolute_mm: 0.001,
                relative: 1e-6,
                angular_rad: 1e-5,
            },
        },
    );
    let recipe_id = recipe.id;
    document.history.recipes.insert(recipe_id, recipe);
    document.validate(&Limits::default()).unwrap();
    Fixture {
        document,
        root: root_id,
        detail: detail_id,
        box_id,
        instance_id,
        member_id,
        definition_id,
        group_id,
        feature_id,
        resource_id,
        recipe_id,
    }
}

fn metadata_of(bytes: &[u8]) -> Vec<u8> {
    let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    assert!(count >= 1);
    let start = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
    let length = u64::from_le_bytes(bytes[48..56].try_into().unwrap()) as usize;
    bytes[start..start + length].to_vec()
}

fn entry(bytes: &[u8], index: usize) -> (u32, u32, [u8; 16], usize, usize, [u8; 32]) {
    let position = HEADER + index * ENTRY;
    let kind = u32::from_le_bytes(bytes[position..position + 4].try_into().unwrap());
    let flags = u32::from_le_bytes(bytes[position + 4..position + 8].try_into().unwrap());
    let id = bytes[position + 8..position + 24].try_into().unwrap();
    let start =
        u64::from_le_bytes(bytes[position + 24..position + 32].try_into().unwrap()) as usize;
    let length =
        u64::from_le_bytes(bytes[position + 32..position + 40].try_into().unwrap()) as usize;
    let hash = bytes[position + 40..position + 72].try_into().unwrap();
    (kind, flags, id, start, length, hash)
}

/// Rebuild a schema-4 container from edited metadata while preserving the
/// original resource chunks byte for byte.
fn repack(original: &[u8], metadata: &[u8]) -> Vec<u8> {
    let count = u32::from_le_bytes(original[12..16].try_into().unwrap()) as usize;
    let mut resource_chunks = Vec::new();
    for index in 1..count {
        let (_, _, _, start, length, _) = entry(original, index);
        resource_chunks.push(&original[start..start + length]);
    }
    let table = HEADER + count * ENTRY;
    let mut out = vec![0u8; table];
    out[..8].copy_from_slice(MAGIC);
    out[8..12].copy_from_slice(&VERSION.to_le_bytes());
    out[12..16].copy_from_slice(&(count as u32).to_le_bytes());
    out[16..20].copy_from_slice(&1u32.to_le_bytes());
    out[40..48].copy_from_slice(&(table as u64).to_le_bytes());
    out[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
    out[56..88].copy_from_slice(&Sha256::digest(metadata));
    out.extend_from_slice(metadata);
    for (offset, chunk) in resource_chunks.iter().enumerate() {
        let (kind, flags, id, _, _, hash) = entry(original, offset + 1);
        let position = HEADER + (offset + 1) * ENTRY;
        let start = out.len() as u64;
        out[position..position + 4].copy_from_slice(&kind.to_le_bytes());
        out[position + 4..position + 8].copy_from_slice(&flags.to_le_bytes());
        out[position + 8..position + 24].copy_from_slice(&id);
        out[position + 24..position + 32].copy_from_slice(&start.to_le_bytes());
        out[position + 32..position + 40].copy_from_slice(&(chunk.len() as u64).to_le_bytes());
        out[position + 40..position + 72].copy_from_slice(&hash);
        out.extend_from_slice(chunk);
    }
    out
}

fn edited(document: &Document, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let bytes = Archive::encode(document, &Limits::default()).unwrap();
    let mut value: Value = serde_json::from_slice(&metadata_of(&bytes)).unwrap();
    edit(&mut value);
    repack(&bytes, &serde_json::to_vec(&value).unwrap())
}

fn without_resources(fixture: &Fixture) -> Document {
    let mut document = fixture.document.clone();
    document.resources.clear();
    document
}

#[test]
fn container_states_schema_document_id_layers_and_brep_payload() {
    let fixture = fixture();
    let bytes = Archive::encode(&fixture.document, &Limits::default()).unwrap();
    assert_eq!(&bytes[..8], MAGIC);
    assert_eq!(
        u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
        VERSION
    );
    // Metadata plus the one embedded jobbag reference chunk.
    assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 2);
    let (metadata_kind, _, _, _, _, _) = entry(&bytes, 0);
    assert_eq!(metadata_kind, 1);
    let (resource_kind, _, _, _, _, _) = entry(&bytes, 1);
    assert_eq!(resource_kind, 2);

    let manifest: Value = serde_json::from_slice(&metadata_of(&bytes)).unwrap();
    assert_eq!(manifest["id"], json!(fixture.document.id.to_string()));
    assert_eq!(manifest["page_unit"], json!("Millimeter"));
    assert_eq!(manifest["layers"].as_object().unwrap().len(), 2);
    assert_eq!(manifest["layer_order"].as_array().unwrap().len(), 2);
    let box_object = &manifest["entities"][fixture.box_id.to_string()];
    assert_eq!(box_object["geometry"].as_object().unwrap().len(), 1);
    assert!(
        box_object["geometry"]["Brep"]["source"]
            .as_str()
            .unwrap()
            .contains("CASCADE Topology"),
        "the B-Rep native source is the persisted payload"
    );
    assert_eq!(
        manifest["resources"][fixture.resource_id.to_string()]["name"],
        json!("jobbag.reference")
    );

    let decoded = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(decoded, fixture.document);
    // The container is deterministic for the same document.
    assert_eq!(
        bytes,
        Archive::encode(&decoded, &Limits::default()).unwrap()
    );
}

#[test]
fn layer_visibility_and_lock_state_round_trip_with_stable_ids() {
    let fixture = fixture();
    let bytes = Archive::encode(&fixture.document, &Limits::default()).unwrap();
    let decoded = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(decoded.id, fixture.document.id);
    assert_eq!(decoded.layers.len(), 2);
    assert_eq!(decoded.layers[&fixture.root].name, "Body");
    assert!(decoded.layers[&fixture.detail].locked);
    assert_eq!(decoded.layers[&fixture.detail].parent, Some(fixture.root));
    assert_eq!(decoded.current_layer, Some(fixture.detail));
    assert_eq!(
        decoded.entities[&fixture.box_id].attributes.layer,
        Some(fixture.detail)
    );
    // Layer lock and object flags are both preserved and still compose.
    assert!(!decoded.permissions(fixture.box_id, false).unwrap().editable);
    assert!(
        !decoded
            .permissions(fixture.instance_id, false)
            .unwrap()
            .visible
    );
    assert!(decoded.groups[&fixture.group_id].contains(fixture.box_id));

    let missing_layer = edited(&without_resources(&fixture), |value| {
        value["layers"]
            .as_object_mut()
            .unwrap()
            .remove(&fixture.detail.to_string());
    });
    assert!(matches!(
        Archive::decode(&missing_layer, &Limits::default()),
        Err(Error::Invalid("layer order"))
    ));
}

#[test]
fn brep_source_and_display_round_trip_without_mesh_substitution() {
    let fixture = fixture();
    let bytes = Archive::encode(&fixture.document, &Limits::default()).unwrap();
    let decoded = Archive::decode(&bytes, &Limits::default()).unwrap();
    let Geometry::Brep(loaded) = &decoded.entities[&fixture.box_id].geometry else {
        panic!("B-Rep must never be downgraded to a display mesh");
    };
    let Geometry::Brep(original) = &fixture.document.entities[&fixture.box_id].geometry else {
        unreachable!();
    };
    assert_eq!(loaded, original);
    assert_eq!(loaded.source(), original.source());
    assert_eq!(loaded.display.faces, original.display.faces);
    assert!((display_volume(&loaded.display) - 6000.0).abs() < 1e-9);

    let missing_source = edited(&without_resources(&fixture), |value| {
        value["entities"][fixture.box_id.to_string()]["geometry"]["Brep"]
            .as_object_mut()
            .unwrap()
            .remove("source");
    });
    assert!(matches!(
        Archive::decode(&missing_source, &Limits::default()),
        Err(Error::Invalid("typed document metadata/schema"))
    ));

    let tampered_source = edited(&without_resources(&fixture), |value| {
        value["entities"][fixture.box_id.to_string()]["geometry"]["Brep"]["source"] =
            json!("CASCADE Topology tampered");
    });
    assert!(matches!(
        Archive::decode(&tampered_source, &Limits::default()),
        Err(Error::Invalid("BRep source/cache binding"))
    ));
}

#[test]
fn history_and_feature_placeholders_reopen_without_recompute_support() {
    let fixture = fixture();
    let bytes = Archive::encode(&fixture.document, &Limits::default()).unwrap();
    let decoded = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(decoded.features.len(), 1);
    let feature = &decoded.features[&fixture.feature_id];
    assert_eq!(feature.label, "jobbag marker");
    assert_eq!(feature.kind, FeatureKind::Semantic);
    let recipe = &decoded.history.recipes[&fixture.recipe_id];
    assert_eq!(recipe.command, "future-builder-v9");
    assert_eq!(recipe.environment.solver, "future-solver");
    assert_eq!(recipe.environment.command_version, 9);
    assert_eq!(recipe.state, EvaluationState::Clean);

    // Runtime restoration does not invent recompute support: the unknown module
    // is reported and only the recipe state changes.
    let restored = Archive::restore(&bytes, &Limits::default(), &BTreeSet::new()).unwrap();
    assert_eq!(restored.diagnostics.len(), 1);
    assert!(restored.diagnostics[0].contains("future-solver"));
    assert_eq!(
        restored.document.history.recipes[&fixture.recipe_id].state,
        EvaluationState::Unresolved(restored.diagnostics[0].clone())
    );
    assert!(restored.document.entities.contains_key(&fixture.box_id));

    // An unknown persistent feature schema is refused with a typed diagnostic
    // instead of being silently dropped or reinterpreted.
    let unknown_feature = edited(&without_resources(&fixture), |value| {
        value["features"][fixture.feature_id.to_string()]["kind"] = json!("future-semantic");
    });
    assert!(matches!(
        Archive::decode(&unknown_feature, &Limits::default()),
        Err(Error::Unsupported("document variant"))
    ));
}

#[test]
fn damaged_container_is_refused_before_any_document_is_published() {
    let fixture = fixture();
    let bytes = Archive::encode(&fixture.document, &Limits::default()).unwrap();
    let limits = Limits::default();

    let mut unsupported = bytes.clone();
    unsupported[8..12].copy_from_slice(&(VERSION + 1).to_le_bytes());
    assert!(matches!(
        Archive::decode(&unsupported, &limits),
        Err(Error::Unsupported(_))
    ));

    assert!(matches!(
        Archive::decode(&bytes[..bytes.len() - 1], &limits),
        Err(Error::Invalid(_))
    ));

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(
        Archive::decode(&trailing, &limits),
        Err(Error::Invalid(_))
    ));

    let mut flags = bytes.clone();
    flags[20..24].copy_from_slice(&1u32.to_le_bytes());
    assert!(matches!(
        Archive::decode(&flags, &limits),
        Err(Error::Unsupported(_))
    ));

    // The in-memory document that a caller already had is untouched by every
    // refused read: it still validates and reopens to itself.
    assert!(fixture.document.validate(&limits).is_ok());
    let again = Archive::encode(&fixture.document, &limits).unwrap();
    assert_eq!(Archive::decode(&again, &limits).unwrap(), fixture.document);
}

#[test]
fn roundtrip_fixture_keeps_two_layers_box_instance_group_and_jobbag_reference() {
    let fixture = fixture();
    let folder = std::env::temp_dir().join(format!("om9-l2-01-12-{}", uuid()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("project.om9");
    Archive::save(
        &path,
        &fixture.document,
        &Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    let reopened = Archive::open(&path, &Limits::default()).unwrap();
    assert_eq!(reopened, fixture.document);
    assert_eq!(reopened.id, fixture.document.id);
    assert_eq!(reopened.layers.len(), 2);
    assert_eq!(reopened.layer_order, fixture.document.layer_order);
    assert_eq!(
        reopened.entities.keys().copied().collect::<BTreeSet<_>>(),
        fixture.document.entities.keys().copied().collect()
    );
    let Geometry::Instance(definition) = reopened.entities[&fixture.instance_id].geometry else {
        panic!("instance link lost");
    };
    assert_eq!(definition, fixture.definition_id);
    assert_eq!(
        reopened.definitions[&fixture.definition_id].members[0].id,
        fixture.member_id
    );
    assert_eq!(
        reopened.groups[&fixture.group_id].members,
        vec![fixture.box_id, fixture.instance_id]
    );
    let resource = &reopened.resources[&fixture.resource_id];
    assert_eq!(resource.payload().unwrap(), b"jobbag/A-102.om9");
    let Geometry::Brep(box_brep) = &reopened.entities[&fixture.box_id].geometry else {
        panic!("box lost");
    };
    assert!((display_volume(&box_brep.display) - 6000.0).abs() < 1e-9);

    // A damaged saved bundle is refused and the original document stays usable.
    let mut damaged = std::fs::read(&path).unwrap();
    let last = damaged.len() - 1;
    damaged[last] ^= 0xff;
    assert!(Archive::decode(&damaged, &Limits::default()).is_err());
    assert_eq!(
        Archive::decode(
            &Archive::encode(&reopened, &Limits::default()).unwrap(),
            &Limits::default()
        )
        .unwrap(),
        reopened
    );

    // Duplicate identities / broken hierarchy fail the golden reopen instead of
    // silently keeping the last record.
    let mut duplicate = without_resources(&fixture);
    duplicate.resources.clear();
    let duplicate_bytes = Archive::encode(&duplicate, &Limits::default()).unwrap();
    let mut manifest: Value = serde_json::from_slice(&metadata_of(&duplicate_bytes)).unwrap();
    let box_value = manifest["entities"][fixture.box_id.to_string()].clone();
    manifest["definitions"][fixture.definition_id.to_string()]["members"]
        .as_array_mut()
        .unwrap()
        .push(box_value);
    let duplicate = repack(&duplicate_bytes, &serde_json::to_vec(&manifest).unwrap());
    assert!(matches!(
        Archive::decode(&duplicate, &Limits::default()),
        Err(Error::Invalid("entity identity/revision"))
    ));

    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(folder).unwrap();
}

#[test]
fn replay_key_matches_are_not_required_for_plain_document_reopen() {
    // Opening a project does not need a replay registry; only runtime restoration
    // reports unresolved modules. This pins the boundary the app relies on.
    let fixture = fixture();
    let bytes = Archive::encode(&fixture.document, &Limits::default()).unwrap();
    let document = Archive::decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(document, fixture.document);
    let keys: BTreeSet<ReplayKey> = BTreeSet::new();
    assert!(Archive::restore(&bytes, &Limits::default(), &keys).is_ok());
}
