// Audit harness only. Compiles the current production module unchanged.
#[allow(dead_code)]
#[path = "../../../../../crates/om9-app/src/project_manager.rs"]
mod project_manager;

use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
};
use project_manager::ProjectManager;
use serde_json::json;

fn main() {
    let mut document = Document::new("audit");
    let mut entity = Entity::new("placed point", Geometry::Point([2., 3., 4.]), None);
    entity.placement = Transform::translation([10., 20., 30.]);
    let id = entity.id;
    let source_world = entity.placement.point([2., 3., 4.]).unwrap();
    document.entities.insert(id, entity);
    let mut manager = ProjectManager::new("Default");
    manager.capture_selection(0, &[id], &document).unwrap();
    let restored = manager.restore_entities(0).unwrap();
    let restored_world = restored[0].placement.point([2., 3., 4.]).unwrap();
    document.entities.remove(&id);
    let available_after_delete = manager.slot_available(0, &document);
    let saved_entities_after_delete = manager.restore_entities(0).unwrap().len();

    let mut renamed = ProjectManager::new("Default");
    renamed.rename_project(renamed.active().id, "Renamed").unwrap();
    let renamed_is_persistent = renamed.has_persistent_state();
    renamed.set_notes("note to remove").unwrap();
    let clear_notes_allowed = renamed.set_notes("").is_ok();

    let resource = Resource::from_bytes(
        "authored opaque fixture", ResourceKind::OpaqueBinary, b"keep these bytes", &Limits::default(),
    ).unwrap();
    let resource_id = resource.id;
    let mut master_document = Document::new("master with resource");
    let entity = Entity::new("point", Geometry::Point([0.; 3]), None);
    master_document.entities.insert(entity.id, entity);
    master_document.resources.insert(resource_id, resource);
    let mut master = ProjectManager::new("Master resource");
    master.capture_master(&master_document).unwrap();
    let before = master.master_document().unwrap().resources[&resource_id].payload().unwrap().len();
    let bytes = master.to_json().unwrap();
    let decoded = ProjectManager::from_json(&bytes, "Master resource").unwrap();
    let after = decoded.master_document().unwrap().resources[&resource_id].payload();

    let mut malformed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let row = malformed["rows"][0].clone();
    malformed["rows"].as_array_mut().unwrap().push(row);
    let duplicate_ids_accepted = ProjectManager::from_json(
        &serde_json::to_vec(&malformed).unwrap(), "duplicate project IDs",
    ).is_ok();

    println!("{}", serde_json::to_string_pretty(&json!({
        "scope": "production ProjectManager model compiled by path; no UI or host replacement",
        "placement": {"source_world":source_world,"restored_world":restored_world,"preserved":source_world==restored_world},
        "snapshot_after_source_delete": {"ui_available":available_after_delete,"saved_entity_count":saved_entities_after_delete},
        "rename_only_persisted_by_staging_predicate": renamed_is_persistent,
        "clear_notes_allowed":clear_notes_allowed,
        "master_resource_json_roundtrip": {"before_payload_bytes":before,"after_payload_available":after.is_ok(),"after_error":after.err().map(|e|e.to_string())},
        "invalid_duplicate_project_ids_accepted":duplicate_ids_accepted
    })).unwrap());
}
