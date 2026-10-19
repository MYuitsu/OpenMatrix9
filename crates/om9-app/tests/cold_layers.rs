// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    layers::{LayerAction, LayerPlan, PresetSlot},
    model::{Document, Entity, Layer},
    transaction::*,
};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Duration};
#[test]
fn independent_owned_process_restores_layer_current_and_explicit_object_preferences() {
    let limits = Limits::default();
    let mut d = Document::new("cold owned layers");
    let mut e = Entity::new(
        "explicit invisible unselectable",
        Geometry::Point([1., 2., 3.]),
        None,
    );
    e.attributes.visible = false;
    e.attributes.selectable = false;
    let eid = e.id;
    d.entities.insert(eid, e);
    let did = d.id;
    let mut c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let layer = Layer::new("User custom metadata", [4, 5, 6, 255]);
    let lid = layer.id;
    let apply = |c: &mut TransactionCoordinator, action| {
        let plan = LayerPlan::capture(c, did, action, None).unwrap();
        let p = c
            .prepare(TransactionRequest::new(
                "Layers",
                vec![Mutation::Layers(Box::new(plan))],
            ))
            .unwrap();
        c.commit(p).unwrap();
    };
    apply(
        &mut c,
        LayerAction::Create {
            layer,
            make_current: true,
        },
    );
    let source = c.world().documents[&did].stamp(eid).unwrap();
    apply(
        &mut c,
        LayerAction::Assign {
            layer: lid,
            sources: vec![source],
        },
    );
    let mut layer = c.world().documents[&did].layers[&lid].clone();
    layer.name = "Saved user label".into();
    layer.visible = false;
    layer.locked = true;
    apply(&mut c, LayerAction::Update { layer });
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let folder = root
        .join("build/standalone-validation")
        .join(format!("cold-layers-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&folder).unwrap();
    let path = folder.join("layers.om9");
    let stored = &c.world().documents[&did];
    Archive::save(&path, stored, &limits, &Cancellation::default()).unwrap();
    let binary = Path::new(env!("CARGO_BIN_EXE_om9"));
    let mut file = fs::File::open(binary).unwrap();
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let program = NativeProgram::admit(binary, hasher.finalize().into()).unwrap();
    let output = program
        .run(
            &["--inspect".into(), path.as_os_str().to_owned()],
            &ProcessLimits {
                stdout_bytes: 64 * 1024,
                deadline: Duration::from_secs(10),
                ..ProcessLimits::default()
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert!(output.stats.exited);
    assert_eq!(output.stats.active_processes, 0);
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["current_layer"], lid.to_string());
    assert_eq!(report["layer_order"], serde_json::json!([lid.to_string()]));
    assert_eq!(
        report["layers"],
        serde_json::to_value(&stored.layers).unwrap()
    );
    assert_eq!(report["objects"][0]["id"], eid.to_string());
    assert_eq!(
        report["objects"][0]["attributes"],
        serde_json::to_value(&stored.entities[&eid].attributes).unwrap()
    );
    assert_eq!(
        report["objects"][0]["geometry"],
        serde_json::json!({"Point":[1.,2.,3.]})
    );
    println!(
        "OM9_LAYER_RECEIPT {}",
        serde_json::json!({"case":"cold_layer_metadata","pid":output.stats.pid,"exited":output.stats.exited,"active_processes":output.stats.active_processes,"peak_committed_bytes":output.stats.peak_committed_bytes,"binary":env!("CARGO_BIN_EXE_om9"),"binary_sha256":program.fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>(),"archive_sha256":format!("{:x}",Sha256::digest(fs::read(&path).unwrap())),"layer_id":lid,"current":report["current_layer"],"layers":report["layers"],"attributes":report["objects"][0]["attributes"],"full_layers_acceptance":false})
    );
    fs::remove_file(path).unwrap();
    fs::remove_dir(folder).unwrap();
}

fn owned_run(arguments: &[std::ffi::OsString], case: &str) -> serde_json::Value {
    let binary = Path::new(env!("CARGO_BIN_EXE_om9"));
    let mut file = fs::File::open(binary).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let program = NativeProgram::admit(binary, hash.finalize().into()).unwrap();
    let output = program
        .run(
            arguments,
            &ProcessLimits {
                stdout_bytes: 64 * 1024,
                deadline: Duration::from_secs(10),
                ..ProcessLimits::default()
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert!(output.stats.exited);
    assert_eq!(output.stats.active_processes, 0);
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    println!(
        "OM9_LAYER_RECEIPT {}",
        serde_json::json!({"case":case,"pid":output.stats.pid,"exited":output.stats.exited,"active_processes":output.stats.active_processes,"peak_committed_bytes":output.stats.peak_committed_bytes,"binary":env!("CARGO_BIN_EXE_om9"),"binary_sha256":program.fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>(),"archive_schema":4,"full_layers_acceptance":false})
    );
    report
}
fn folder() -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let folder = root
        .join("build/standalone-validation")
        .join(format!("cold-layer-state-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&folder).unwrap();
    folder
}
#[test]
fn independent_process_restores_unicode_preset_and_independent_hidden_flags() {
    let limits = Limits::default();
    let mut d = Document::new("cold preset and Hidden");
    let mut layer = Layer::preset(PresetSlot::new(17).unwrap());
    layer.name = "Đá riêng".into();
    layer.color = [4, 5, 6, 255];
    layer.locked = true;
    layer.visible = false;
    let lid = layer.id;
    d.layer_order.push(lid);
    d.current_layer = Some(lid);
    d.layers.insert(lid, layer);
    let mut object = Entity::new(
        "hidden explicit invisible",
        Geometry::Point([1., 2., 3.]),
        Some(lid),
    );
    object.attributes.visible = false;
    object.attributes.selectable = false;
    object.attributes.hidden = true;
    let eid = object.id;
    d.entities.insert(eid, object);
    let folder = folder();
    let file = folder.join("owned.om9");
    Archive::save(&file, &d, &limits, &Cancellation::default()).unwrap();
    let report = owned_run(
        &["--inspect".into(), file.as_os_str().to_owned()],
        "cold_preset_hidden_metadata",
    );
    assert_eq!(report["layers"], serde_json::to_value(&d.layers).unwrap());
    assert_eq!(report["current_layer"], lid.to_string());
    assert_eq!(report["layer_order"], serde_json::json!([lid.to_string()]));
    assert_eq!(
        report["objects"][0]["attributes"],
        serde_json::to_value(&d.entities[&eid].attributes).unwrap()
    );
    assert_eq!(
        report["objects"][0]["geometry"],
        serde_json::json!({"Point":[1.,2.,3.]})
    );
    fs::remove_file(file).unwrap();
    fs::remove_dir(folder).unwrap();
}
#[test]
fn actual_cli_explicitly_migrates_v3_then_fresh_process_reopens_v4_without_inferred_slot() {
    let mut d = Document::new("cold legacy slot name");
    let layer = Layer::new("Metal 01", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let json = serde_json::to_vec(&d).unwrap();
    let mut bytes = vec![0; 88];
    bytes[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    bytes[8..12].copy_from_slice(&3u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes[40..48].copy_from_slice(&88u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    bytes[56..88].copy_from_slice(&Sha256::digest(&json));
    bytes.extend(json);
    let folder = folder();
    let file = folder.join("legacy.om9");
    fs::write(&file, bytes).unwrap();
    let migrated = owned_run(
        &["--migrate-v3".into(), file.as_os_str().to_owned()],
        "cold_schema3_cli_migration",
    );
    let saved = fs::read(&file).unwrap();
    assert_eq!(u32::from_le_bytes(saved[8..12].try_into().unwrap()), 4);
    let report = owned_run(
        &["--inspect".into(), file.as_os_str().to_owned()],
        "cold_schema4_cli_reopen",
    );
    assert_eq!(migrated["layers"], report["layers"]);
    assert_eq!(report["layers"], serde_json::to_value(&d.layers).unwrap());
    assert!(
        report["layers"][lid.to_string()]
            .get("preset_slot")
            .is_none()
    );
    assert!(report["current_layer"].is_null());
    fs::remove_file(file).unwrap();
    fs::remove_dir(folder).unwrap();
}
#[test]
fn fresh_owned_process_restores_actual_dialog_commits_and_rejects_the_old_handle() {
    use om9_app::{
        layer_command::LayerProperty, layer_dialog::LayerDialogKind, session::Controller,
    };
    let limits = Limits::default();
    let mut d = Document::new("cold dialog publications");
    let mut layer = Layer::preset(PresetSlot::new(17).unwrap());
    layer.color = [10, 20, 30, 90];
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    d.current_layer = Some(lid);
    let mut e = Entity::new(
        "hidden independent preferences",
        Geometry::Point([1., 2., 3.]),
        Some(lid),
    );
    e.attributes.hidden = true;
    e.attributes.visible = false;
    e.attributes.selectable = false;
    e.attributes.color = None;
    let eid = e.id;
    d.entities.insert(eid, e);
    let mut controller = Controller::new(d, limits).unwrap();
    let slot = PresetSlot::new(17).unwrap();
    let rename = controller
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot))
        .unwrap();
    controller
        .set_layer_dialog_name(rename, "Đá tùy biến")
        .unwrap();
    controller.confirm_layer_dialog(rename).unwrap();
    let color = controller
        .begin_layer_dialog(LayerDialogKind::Color, Some(slot))
        .unwrap();
    controller.set_layer_dialog_color(color, [7, 8, 9]).unwrap();
    controller.confirm_layer_dialog(color).unwrap();
    let show = controller
        .begin_layer_dialog(LayerDialogKind::Show, None)
        .unwrap();
    controller.set_layer_dialog_choice(show, 0, true).unwrap();
    controller.confirm_layer_dialog(show).unwrap();
    controller.undo().unwrap();
    controller.redo().unwrap();
    controller
        .set_layer_property(slot, LayerProperty::Locked(true))
        .unwrap();
    controller
        .set_layer_property(slot, LayerProperty::Visible(false))
        .unwrap();
    let folder = folder();
    let file = folder.join("dialog.om9");
    Archive::save(
        &file,
        controller.document(),
        &limits,
        &Cancellation::default(),
    )
    .unwrap();
    let report = owned_run(
        &["--inspect".into(), file.as_os_str().to_owned()],
        "cold_actual_layer_dialog_commits",
    );
    assert_eq!(
        report["layers"],
        serde_json::to_value(&controller.document().layers).unwrap()
    );
    assert_eq!(report["current_layer"], lid.to_string());
    assert_eq!(
        report["objects"][0]["attributes"],
        serde_json::to_value(&controller.document().entities[&eid].attributes).unwrap()
    );
    assert_eq!(
        report["objects"][0]["geometry"],
        serde_json::json!({"Point":[1.,2.,3.]})
    );
    let restored = Archive::open(&file, &limits).unwrap();
    let mut reopened = Controller::new(restored, limits).unwrap();
    let fresh = reopened
        .begin_layer_dialog(LayerDialogKind::Color, Some(slot))
        .unwrap();
    let before = reopened.coordinator().snapshot();
    assert!(reopened.confirm_layer_dialog(color).is_err());
    assert_eq!(reopened.coordinator().snapshot(), before);
    assert_eq!(reopened.layer_dialog().unwrap().token(), fresh);
    assert_eq!(reopened.document().layers[&lid].color, [7, 8, 9, 90]);
    assert!(!reopened.document().entities[&eid].attributes.hidden);
    assert!(!reopened.scene().unwrap().objects[0].visible);
    fs::remove_file(file).unwrap();
    fs::remove_dir(folder).unwrap();
}
