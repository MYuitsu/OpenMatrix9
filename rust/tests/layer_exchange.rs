#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_exchange::*, layer_state::*};

#[test]
fn selected_transfer_keeps_empty_palette_source_wins_and_existing_object_state() {
    let mut source = snapshot();
    source.document_id = "source".into();
    source.layers[0].locked = true;
    source.layers[0].rgb = [222, 23, 45];
    source
        .layers
        .push(layer("empty", None, &["Empty purple slot"]));
    source.objects[0].locked = false;
    let mut target = snapshot();
    target.document_id = "destination".into();
    target.layers[0].source_id = "target-metal".into();
    target.layers[2].parent_id = Some("target-metal".into());
    target.objects[0].layer_id = "target-metal".into();
    target.active_layer = Some("gem".into());
    let before = target.clone();
    let p = plan_receive(
        &source,
        &target,
        TransferScope::Selected(vec!["ring".into()]),
    )
    .unwrap();
    assert_eq!(p.before, before);
    assert_eq!(target, before);
    assert_eq!(p.layer_mapping["metal"], "target-metal");
    assert_eq!(p.after.layers.len(), 4);
    assert_eq!(p.after.objects.len(), 4);
    let e = effective_state(&p.after, "ring").unwrap();
    assert!(e.locked);
    assert_eq!(e.rgb, [222, 23, 45]);
    let imported = &p.after.objects[3];
    assert!(!imported.locked);
    assert!(effective_state(&p.after, &imported.id).unwrap().locked);
    assert_ne!(imported.id, "ring");
    assert_eq!(p.after.active_layer.as_deref(), Some("gem"));
    assert!(p.check_destination(&before).is_ok());
}
#[test]
fn session_includes_hidden_locked_geometry_and_preserves_by_object_color() {
    let mut source = snapshot();
    source.layers[0].locked = true;
    source.layers[1].visible = false;
    source.objects[0].locked = true;
    source.objects[0].color_source = ColorSource::ByObject;
    source.objects[0].rgb = [111, 112, 113];
    let target = LayerSnapshotV1::empty("destination");
    let p = plan_receive(&source, &target, TransferScope::Session).unwrap();
    assert_eq!(p.after.objects.len(), 3);
    assert_eq!(p.object_mapping.len(), 3);
    let e = effective_state(&p.after, &p.object_mapping["ring"]).unwrap();
    assert!(e.locked);
    assert_eq!(e.rgb, [111, 112, 113]);
    assert!(
        !effective_state(&p.after, &p.object_mapping["stone"])
            .unwrap()
            .visible
    );
    let active = p
        .after
        .layers
        .iter()
        .find(|l| Some(&l.source_id) == p.after.active_layer.as_ref())
        .unwrap();
    assert_eq!(active.name, "OM9 Transfer Work");
}
#[test]
fn palette_only_transfer_is_valid_and_active_layer_fallback_never_unlocks_source() {
    let mut source = snapshot();
    source.objects.clear();
    for l in &mut source.layers {
        l.locked = true;
        l.visible = false;
    }
    source
        .layers
        .push(layer("taken", None, &["OM9 Transfer Work"]));
    source.layers.last_mut().unwrap().locked = true;
    let p = plan_receive(
        &source,
        &LayerSnapshotV1::empty("destination"),
        TransferScope::Selected(vec![]),
    )
    .unwrap();
    assert!(p.after.objects.is_empty());
    assert_eq!(p.after.layers.len(), 5);
    assert!(p.after.layers[..4].iter().all(|l| l.locked));
    let active = p
        .after
        .layers
        .iter()
        .find(|l| Some(&l.source_id) == p.after.active_layer.as_ref())
        .unwrap();
    assert_eq!(active.name, "OM9 Transfer Work 2");
    assert_eq!(active.rgb, [180, 180, 180]);
    assert!(!active.locked);
    assert!(active.visible);
}
#[test]
fn repeat_paste_adds_new_objects_without_uuid_merge_and_preserves_destination_only_layers() {
    let source = snapshot();
    let mut target = LayerSnapshotV1::empty("target");
    target
        .layers
        .push(layer("local", None, &["Destination only"]));
    let first = plan_receive(&source, &target, TransferScope::Session).unwrap();
    let second = plan_receive(&source, &first.after, TransferScope::Session).unwrap();
    assert_eq!(second.after.layers.len(), 4);
    assert_eq!(second.after.objects.len(), 6);
    assert!(second.after.layers.iter().any(|l| l.source_id == "local"));
}
#[test]
fn checks_generation_document_and_same_generation_metadata_before_commit() {
    let source = snapshot();
    let target = LayerSnapshotV1::empty("target");
    let p = plan_receive(&source, &target, TransferScope::Session).unwrap();
    let mut changed = target.clone();
    changed.generation += 1;
    assert_eq!(p.check_destination(&changed), Err(LayerError::Stale));
    let mut changed = target.clone();
    changed.document_id = "other-document".into();
    assert_eq!(p.check_destination(&changed), Err(LayerError::Stale));
    let mut changed = target.clone();
    changed.layers.push(layer("new", None, &["Changed"]));
    assert_eq!(p.check_destination(&changed), Err(LayerError::Stale));
}
#[test]
fn preserves_distinct_parent_paths_and_remaps_colliding_source_uuids() {
    let source = snapshot();
    let mut target = LayerSnapshotV1::empty("target");
    target.layers = vec![
        layer("metal", None, &["Different metal"]),
        layer("detail", Some("metal"), &["Different metal", "Detail"]),
    ];
    let p = plan_receive(&source, &target, TransferScope::Session).unwrap();
    assert_eq!(p.after.layers.len(), 5);
    assert_ne!(p.layer_mapping["metal"], "metal");
    let imported = p
        .after
        .layers
        .iter()
        .find(|l| l.source_id == p.layer_mapping["detail"])
        .unwrap();
    assert_eq!(imported.parent_id.as_ref(), Some(&p.layer_mapping["metal"]));
}
#[test]
fn wrong_and_duplicate_selected_ids_fail_in_preflight() {
    let s = snapshot();
    let t = LayerSnapshotV1::empty("target");
    assert_eq!(
        plan_receive(&s, &t, TransferScope::Selected(vec!["missing".into()])).unwrap_err(),
        LayerError::MissingObject
    );
    assert_eq!(
        plan_receive(
            &s,
            &t,
            TransferScope::Selected(vec!["ring".into(), "ring".into()])
        )
        .unwrap_err(),
        LayerError::DuplicateId
    );
}
#[test]
fn generation_overflow_rejects_instead_of_reusing_stale_generation() {
    let s = snapshot();
    let mut t = LayerSnapshotV1::empty("target");
    t.generation = u64::MAX;
    assert_eq!(
        plan_receive(&s, &t, TransferScope::Session).unwrap_err(),
        LayerError::LimitExceeded
    );
}
