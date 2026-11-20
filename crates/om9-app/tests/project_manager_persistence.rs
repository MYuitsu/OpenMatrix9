// SPDX-License-Identifier: LGPL-2.1-or-later
//! Job Bag persistence inside `.om9` and live Master restore (OM9-L2-03-15).
use om9_app::{file_lifecycle::FileKind, ui::MatrixApp};
use om9_core::model::Document;
use std::path::PathBuf;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "om9-project-manager-{}.jobbags.om9",
            uuid::Uuid::new_v4()
        ));
        Self(path)
    }
}

#[test]
fn project_asset_budget_counts_shared_owners_once_and_refuses_distinct_owners() {
    use om9_app::project_manager::ProjectManager;
    use om9_core::budget::Limits;
    let mut manager = ProjectManager::new("asset ownership");
    let limit = Limits { max_bytes: 10, ..Limits::default() };
    let first = manager.capture_named_version_summary_with_limits("Master", 0, 0, vec![1; 8], &limit).unwrap();
    assert!(manager.capture_named_version_summary_with_limits("Master", 0, 0, vec![1; 8], &limit).is_err());
    assert_eq!(manager.named_master_versions("Master").len(), 1);
    let second = manager.capture_named_version_summary("Master", 0, 0, vec![2; 8]).unwrap();
    let shared = manager.named_master_by_id("Master", first).unwrap().archive.clone();
    manager.active_mut().named_versions.get_mut("Master").unwrap().iter_mut()
        .find(|version| version.id == second).unwrap().archive = shared;
    manager.set_thumbnail_with_limits(manager.active().id, second, vec![3], &limit).unwrap();
    assert!(manager.named_master_by_id("Master", second).unwrap().thumbnail_png.is_some());
}

#[test]
fn all_named_groups_preserve_multiple_document_versions_through_save_and_reopen() {
    let fixture = Fixture::new();
    let mut app = MatrixApp::new(Document::new("all named versions")).unwrap();
    let mut expected = Vec::new();
    for name in ["Master", "Creation", "Parts", "Render", "Output"] {
        for _ in 0..2 {
            let count = app.controller.document().entities.len();
            add_point(&mut app, count as f64);
            let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
            let id = app.project_manager.capture_named_version(name, snapshot, archive).unwrap();
            expected.push((name, id, count + 1));
        }
    }
    app.stage_project_state().unwrap();
    app.controller.begin_file_operation(FileKind::SaveAs).unwrap();
    let token = app.controller.file_operation().unwrap().token();
    app.controller.accept_file_path(token, &fixture.0).unwrap();
    let reopened = MatrixApp::from_file(&fixture.0).unwrap();
    for (name, id, count) in expected {
        let version = reopened.project_manager.named_master_by_id(name, id).unwrap();
        let document = reopened.controller.decode_jobbag_document(version.archive.as_ref().unwrap()).unwrap();
        assert_eq!(document.entities.len(), count, "{name} version {id:?}");
    }
    for name in ["Master", "Creation", "Parts", "Render", "Output"] {
        assert_eq!(reopened.project_manager.named_master_versions(name).len(), 2, "{name}");
    }
}

#[test]
fn named_snapshot_restores_hidden_and_locked_objects_and_layers_after_reopen() {
    use om9_core::{associative::FeaturePort, budget::Limits, exchange::RenderMaterial,
        geometry::{Geometry, GeometryKind, Transform, Unit}, history::{Recipe, ReplayEnvironment, Tolerance},
        identity::MaterialId, model::{Entity, Layer}, request::{Selection, SourceBinding}, resource::{Resource, ResourceKind}};
    let fixture = Fixture::new();
    let mut document = Document::new("hidden and locked snapshot");
    let mut layer = Layer::new("hidden locked layer", [70, 80, 90, 255]);
    layer.visible = false;
    layer.locked = true;
    let material = RenderMaterial { id: MaterialId::new(), name: "authored gold".into(), diffuse: [220, 170, 40, 255] };
    layer.material = Some(material.id);
    document.exchange.materials.insert(material.id, material.clone());
    let layer_id = layer.id;
    document.layer_order.push(layer_id);
    document.layers.insert(layer_id, layer.clone());
    let mut hidden = Entity::new("hidden", Geometry::Point([1., 2., 3.]), Some(layer_id));
    hidden.attributes.hidden = true;
    hidden.attributes.visible = false;
    let mut locked = Entity::new("locked", Geometry::Point([4., 5., 6.]), Some(layer_id));
    locked.attributes.locked = true;
    locked.attributes.material = Some(material.id);
    let hidden_id = hidden.id;
    let locked_id = locked.id;
    document.entities.insert(hidden_id, hidden.clone());
    document.entities.insert(locked_id, locked.clone());
    let source = SourceBinding { role: "source".into(), source: document.stamp(hidden_id).unwrap(), selection: Selection::Whole };
    let recipe = Recipe::new(document.id, "fixture point", vec![source], vec![locked_id], ReplayEnvironment {
        command_version: 1, solver: "om9".into(), solver_version: "1".into(), schema_version: 1,
        length_unit: Unit::Millimeter, frame: Transform::identity(),
        tolerance: Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap(),
    }).with_ports(vec![FeaturePort::geometry("source", &[GeometryKind::Point])], vec![FeaturePort::geometry("result", &[GeometryKind::Point])]);
    document.history.recipes.insert(recipe.id, recipe.clone());
    let resource = Resource::from_bytes("design asset", ResourceKind::OpaqueBinary, b"snapshot resource", &Limits::default()).unwrap();
    let resource_id = resource.id;
    document.resources.insert(resource_id, resource);

    let mut app = MatrixApp::new(document).unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    add_point(&mut app, 9.);
    app.stage_project_state().unwrap();
    app.controller.begin_file_operation(FileKind::SaveAs).unwrap();
    let token = app.controller.file_operation().unwrap().token();
    app.controller.accept_file_path(token, &fixture.0).unwrap();

    let mut reopened = MatrixApp::from_file(&fixture.0).unwrap();
    assert_eq!(reopened.controller.document().entities.len(), 3);
    reopened.restore_master_live().unwrap();
    let restored = reopened.controller.document();
    assert_eq!(restored.entities.len(), 2);
    assert_eq!(restored.entities[&hidden_id], hidden);
    assert_eq!(restored.entities[&locked_id], locked);
    assert_eq!(restored.layers[&layer_id], layer);
    assert_eq!(restored.exchange.materials[&material.id], material);
    assert_eq!(restored.history.recipes[&recipe.id], recipe);
    assert_eq!(restored.history.owner(locked_id), Some(recipe.id));
    assert_eq!(restored.resources[&resource_id].payload().unwrap(), b"snapshot resource");
}

#[test]
fn manually_entered_ring_rail_radius_round_trips_and_rejects_invalid_values() {
    let mut manager = om9_app::project_manager::ProjectManager::new("radius");
    let id = manager.active().id;
    let mut metadata = manager.active().metadata.clone();
    metadata.ring_rail_radius_mm = Some(8.5);
    manager.update_project(id, "radius", "", "", metadata.clone()).unwrap();
    let restored = om9_app::project_manager::ProjectManager::from_json(&manager.to_json().unwrap(), "radius").unwrap();
    assert_eq!(restored.active().metadata.ring_rail_radius_mm, Some(8.5));
    metadata.ring_rail_radius_mm = Some(-1.);
    assert!(manager.update_project(id, "radius", "", "", metadata).is_err());
    assert_eq!(manager.active().metadata.ring_rail_radius_mm, Some(8.5));
}

#[test]
fn project_timer_survives_serialization_and_uses_elapsed_wall_time() {
    let mut manager = om9_app::project_manager::ProjectManager::new("timer");
    let id = manager.active().id;
    manager.timer_start(id, 100).unwrap();
    manager.timer_stop(id, 135).unwrap();
    manager.timer_start(id, 200).unwrap();
    let mut reopened = om9_app::project_manager::ProjectManager::from_json(&manager.to_json().unwrap(), "unused").unwrap();
    assert_eq!(reopened.active().elapsed_at(210), 45);
    reopened.timer_stop(id, 220).unwrap();
    assert_eq!(reopened.active().elapsed_at(500), 55);
    reopened.timer_start(id, 300).unwrap();
    reopened.timer_reset(id, 305).unwrap();
    assert_eq!(reopened.active().elapsed_at(315), 10);
    reopened.timer_stop(id, 320).unwrap();
    assert_eq!(reopened.active().elapsed_at(500), 15);
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn add_point(app: &mut MatrixApp, x: f64) {
    app.controller.submit("Point").expect("Point");
    app.controller
        .submit(&format!("{x},0,0"))
        .expect("coordinates");
}

#[test]
fn staging_default_ui_state_preserves_document_and_redo() {
    let mut app = MatrixApp::new(Document::new("no implicit metadata")).unwrap();
    add_point(&mut app, 1.);
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    app.stage_project_state().unwrap();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}

#[test]
fn unchanged_embedded_ui_records_preserve_document_and_redo() {
    let mut app = MatrixApp::new(Document::new("stable embedded metadata")).unwrap();
    add_point(&mut app, 0.);
    let buffers = app.controller.scene().unwrap().objects[0].buffers.clone();
    app.project_manager
        .set_notes("Keep all four metadata resources")
        .unwrap();
    app.display_preferences.wireframe = false;
    app.controller
        .store_f6_state(app.f6_custom.to_json().unwrap().as_bytes())
        .unwrap();
    let pictures = om9_app::pictureframe::PictureFrameStore {
        version: 1,
        frames: vec![],
    };
    app.controller
        .store_pictureframes(pictures.to_json().unwrap().as_bytes())
        .unwrap();
    app.stage_project_state().unwrap();
    let state = app.controller.document_scene_state();
    assert_eq!(state.scene_epoch, Some(state.publication_epoch));
    assert!(buffers.ptr_eq(&app.controller.scene().unwrap().objects[0].buffers));
    add_point(&mut app, 1.);
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    app.stage_project_state().unwrap();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}

#[test]
fn metadata_admission_failure_preserves_document_scene_and_redo() {
    let mut app = MatrixApp::new(Document::new("metadata pressure")).unwrap();
    add_point(&mut app, 0.);
    add_point(&mut app, 1.);
    app.controller.undo().unwrap();
    let before = app.controller.coordinator().snapshot();
    let state = app.controller.document_scene_state();
    let buffers = app.controller.scene().unwrap().objects[0].buffers.clone();
    let pressure = app
        .controller
        .coordinator()
        .reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,
            None,
            app.controller.scene_cache_usage().unwrap(),
        )
        .unwrap();
    let bytes = serde_json::to_vec(&app.display_preferences).unwrap();
    assert!(matches!(
        app.controller.store_display_state(&bytes),
        Err(om9_core::Error::Budget(_))
    ));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.document_scene_state(), state);
    assert!(buffers.ptr_eq(&app.controller.scene().unwrap().objects[0].buffers));
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    drop(pressure);
    app.controller.store_display_state(&bytes).unwrap();
    let state = app.controller.document_scene_state();
    assert_eq!(state.scene_epoch, Some(state.publication_epoch));
    add_point(&mut app, 2.);
}

#[test]
fn jobbag_state_survives_save_and_reopen_inside_the_om9_file() {
    let fixture = Fixture::new();
    let mut app = MatrixApp::new(Document::new("Persisted jobbag")).expect("app");
    add_point(&mut app, 1.);
    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("point");
    app.controller.select_objects(&[id]).expect("select");
    let archive = app.controller.copy_selected_jobbag_om9().expect("detached archive");
    app.project_manager
        .capture_archive(
            0,
            app.controller
                .jobbag_selected_entities()
                .expect("selected entities"),
            app.controller.document(),
            archive,
        )
        .expect("capture");
    app.project_manager
        .set_notes("Khách hàng: Nguyễn Văn A")
        .expect("notes");
    app.stage_project_state().expect("embed project state");
    assert!(app.controller.document().resources.values().any(|resource| resource.name.starts_with("om9.project-asset.v1.")));
    let staged = app.controller.coordinator().snapshot();
    app.stage_project_state()
        .expect("unchanged state is a no-op");
    assert_eq!(app.controller.coordinator().snapshot(), staged);

    app.controller
        .begin_file_operation(FileKind::SaveAs)
        .expect("begin save");
    let token = app.controller.file_operation().expect("operation").token();
    let outcome = app
        .controller
        .accept_file_path(token, &fixture.0)
        .expect("save file");
    assert!(matches!(
        outcome,
        om9_app::file_lifecycle::FileOutcome::Saved
    ));

    let mut reopened = MatrixApp::from_file(&fixture.0).expect("reopen");
    assert_eq!(
        reopened.project_manager.active().notes,
        "Khách hàng: Nguyễn Văn A"
    );
    assert!(
        reopened
            .project_manager
            .slot_available(0, reopened.controller.document())
    );
    assert_eq!(
        reopened
            .project_manager
            .active()
            .slot(0)
            .and_then(|slot| slot.snapshot.as_ref())
            .map(|snapshot| snapshot.captured_ids.len()),
        Some(1)
    );
    let archive = reopened.project_manager.active().slot(0).unwrap()
        .snapshot.as_ref().unwrap().archive.clone().unwrap();
    let ids = reopened.controller.paste_om9(&archive).expect("insert saved archive");
    assert_eq!(ids.len(), 1);
    assert_eq!(reopened.controller.document().entities.len(), 2);
    reopened.controller.undo().expect("undo insertion");
    assert_eq!(reopened.controller.document().entities.len(), 1);
}

#[test]
fn restore_master_live_replaces_the_document_atomically() {
    let mut app = MatrixApp::new(Document::new("Master restore")).expect("app");
    add_point(&mut app, 1.);
    app.project_manager
        .capture_master(app.controller.document())
        .expect("master");
    let captured = app.controller.document().entities.len();
    add_point(&mut app, 2.);
    assert_eq!(app.controller.document().entities.len(), captured + 1);
    app.restore_master_live().expect("restore master");
    assert_eq!(app.controller.document().entities.len(), captured);
}

#[test]
fn named_versions_hydrate_resources_after_manager_serialization() {
    use om9_core::{budget::Limits, resource::{Resource, ResourceKind}};
    let mut document = Document::new("resource snapshot");
    let resource = Resource::from_bytes("design asset", ResourceKind::OpaqueBinary, b"asset payload", &Limits::default()).unwrap();
    let resource_id = resource.id;
    document.resources.insert(resource_id, resource);
    let mut app = MatrixApp::new(document).unwrap();
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let first = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    add_point(&mut app, 2.);
    let (snapshot, archive) = app.controller.capture_jobbag_document().unwrap();
    let second = app.project_manager.capture_named_version("Master", snapshot, archive).unwrap();
    assert_ne!(first, second);
    let bytes = app.project_manager.to_json().unwrap();
    app.project_manager = om9_app::project_manager::ProjectManager::from_json(&bytes, "resource snapshot").unwrap();
    app.restore_master_live().unwrap();
    assert_eq!(app.controller.document().resources[&resource_id].payload().unwrap(), b"asset payload");
    assert_eq!(app.controller.document().entities.len(), 1);
    app.project_manager.clear_named_master("Master");
    app.restore_master_live().unwrap();
    assert!(app.controller.document().entities.is_empty());
}
