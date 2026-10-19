// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::{Cloud, Geometry},
    layers::PresetSlot,
    model::{Definition, Document, Entity, Layer},
};
use sha2::{Digest, Sha256};
fn container(version: u32, metadata: serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(&metadata).unwrap();
    let mut bytes = vec![0; 88];
    bytes[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    bytes[8..12].copy_from_slice(&version.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes[40..48].copy_from_slice(&88u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    bytes[56..88].copy_from_slice(&Sha256::digest(&json));
    bytes.extend(json);
    bytes
}
#[test]
fn schema4_roundtrip_keeps_preset_hidden_explicit_permissions_and_uuid() {
    let limits = Limits::default();
    let mut d = Document::new("schema4 layers");
    let mut l = Layer::preset(PresetSlot::new(17).unwrap());
    l.name = "Đá riêng".into();
    l.visible = false;
    l.locked = true;
    let lid = l.id;
    d.layers.insert(lid, l);
    d.layer_order.push(lid);
    d.current_layer = Some(lid);
    let mut e = Entity::new("hidden", Geometry::Point([1., 2., 3.]), Some(lid));
    e.attributes.hidden = true;
    e.attributes.selectable = false;
    d.entities.insert(e.id, e);
    let bytes = Archive::encode(&d, &limits).unwrap();
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 4);
    assert_eq!(Archive::decode(&bytes, &limits).unwrap(), d);
}
#[test]
fn schema3_requires_explicit_migration_and_invents_no_slots_from_names() {
    let limits = Limits::default();
    let mut d = Document::new("old ordinary label");
    let l = Layer::new("Metal 01", [1, 2, 3, 255]);
    let lid = l.id;
    d.layers.insert(lid, l);
    d.layer_order.push(lid);
    let e = Entity::new("ordinary", Geometry::Point([1., 2., 3.]), Some(lid));
    d.entities.insert(e.id, e);
    let bytes = container(3, serde_json::to_value(&d).unwrap());
    assert!(Archive::decode(&bytes, &limits).is_err());
    let migrated = Archive::migrate_from_v3(&bytes, &limits).unwrap();
    assert_eq!(migrated.from_version, 3);
    assert_eq!(migrated.to_version, 4);
    let restored = Archive::decode(&migrated.bytes, &limits).unwrap();
    assert_eq!(restored, d);
    assert_eq!(restored.layers[&lid].preset_slot, None);
}
#[test]
fn legacy_migration_rejects_new_fields_even_when_false_or_null() {
    let limits = Limits::default();
    let mut d = Document::new("strict legacy");
    let l = Layer::new("old layer", [1, 2, 3, 255]);
    let lid = l.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, l);
    let e = Entity::new("old root", Geometry::Point([0.; 3]), Some(lid));
    let eid = e.id;
    d.entities.insert(eid, e);
    let mut json = serde_json::to_value(&d).unwrap();
    json["entities"][eid.to_string()]["attributes"]["hidden"] = false.into();
    assert!(Archive::migrate_from_v3(&container(3, json), &limits).is_err());
    let mut json = serde_json::to_value(&d).unwrap();
    json["layers"][lid.to_string()]["preset_slot"] = serde_json::Value::Null;
    assert!(Archive::migrate_from_v3(&container(3, json), &limits).is_err());
}

#[test]
fn legacy_definition_attributes_refuse_hidden_but_cloud_hidden_is_preserved() {
    let limits = Limits::default();
    let mut d = Document::new("legacy nested attributes");
    let member = Entity::new(
        "old member",
        Geometry::Cloud(Cloud {
            points: vec![[0.; 3]],
            normals: vec![],
            colors: vec![],
            hidden: vec![true],
        }),
        None,
    );
    let definition = Definition::new("owned definition", vec![member]);
    let id = definition.id;
    d.definitions.insert(id, definition);
    let json = serde_json::to_value(&d).unwrap();
    let migrated = Archive::migrate_from_v3(&container(3, json.clone()), &limits).unwrap();
    assert_eq!(Archive::decode(&migrated.bytes, &limits).unwrap(), d);
    let mut invalid = json;
    invalid["definitions"][id.to_string()]["members"][0]["attributes"]["hidden"] = false.into();
    assert!(Archive::migrate_from_v3(&container(3, invalid), &limits).is_err());
}

#[test]
fn schema4_corrupt_slots_labels_and_basic_palette_are_rejected_on_decode() {
    let limits = Limits::default();
    let mut d = Document::new("strict preset schema");
    let layer = Layer::preset(PresetSlot::new(1).unwrap());
    let id = layer.id;
    d.layer_order.push(id);
    d.layers.insert(id, layer);
    for slot in [0, 34, 255] {
        let mut invalid = serde_json::to_value(&d).unwrap();
        invalid["layers"][id.to_string()]["preset_slot"] = slot.into();
        assert!(Archive::decode(&container(4, invalid), &limits).is_err());
    }
    for field in ["name", "color"] {
        let mut invalid = serde_json::to_value(&d).unwrap();
        invalid["layers"][id.to_string()][field] = if field == "name" {
            serde_json::json!("Other")
        } else {
            serde_json::json!([1, 2, 3, 255])
        };
        assert!(Archive::decode(&container(4, invalid), &limits).is_err());
    }
}

#[test]
fn schema3_migration_cancel_overflow_and_budget_preserve_original_file() {
    use om9_core::transaction::Cancellation;
    use std::collections::BTreeSet;
    let folder = std::env::temp_dir().join(format!("om9-layer-schema-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("legacy.om9");
    let limits = Limits::default();
    let normal = container(
        3,
        serde_json::to_value(Document::new("old document")).unwrap(),
    );
    std::fs::write(&file, &normal).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(
        Archive::upgrade_v3_restored_in_place(&file, &limits, &cancel, &BTreeSet::new()).is_err()
    );
    assert_eq!(std::fs::read(&file).unwrap(), normal);
    let small = Limits {
        max_bytes: 10,
        ..limits
    };
    assert!(
        Archive::upgrade_v3_restored_in_place(
            &file,
            &small,
            &Cancellation::default(),
            &BTreeSet::new()
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&file).unwrap(), normal);
    let mut d = Document::new("overflow restoration");
    d.session_generation = u64::MAX;
    let overflow = container(3, serde_json::to_value(d).unwrap());
    std::fs::write(&file, &overflow).unwrap();
    assert!(
        Archive::upgrade_v3_restored_in_place(
            &file,
            &limits,
            &Cancellation::default(),
            &BTreeSet::new()
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&file).unwrap(), overflow);
    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
