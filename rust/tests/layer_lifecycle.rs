use openmatrix9_rust::{layer_document::default_document, layer_lifecycle::*, layer_state::*};
#[path = "common/layers.rs"]
mod common;
use common::{object, snapshot};

#[test]
fn legacy_reconciliation_preserves_custom_palette_and_known_own_states() {
    let mut before = default_document("custom").unwrap();
    before.layers.truncate(1);
    before.layers[0].source_id = "om9-legacy-layer-2".into();
    before.layers[0].rgb = [93, 94, 95];
    before.active_layer = Some(before.layers[0].source_id.clone());
    before = plan_observed(&before, &["known".into()]).unwrap().after;
    before.objects[0].locked = true;
    before.objects[0].visible = false;
    before.objects[0].color_source = ColorSource::ByObject;
    before.objects[0].rgb = [7, 8, 9];
    let facts = vec![
        LegacyObject {
            id: "known".into(),
            path: Some("Ignored projected path".into()),
            rgb: [1, 2, 3],
            locked: false,
            visible: true,
        },
        LegacyObject {
            id: "native".into(),
            path: Some("Custom::Detail".into()),
            rgb: [31, 32, 33],
            locked: true,
            visible: false,
        },
        LegacyObject {
            id: "unbound".into(),
            path: None,
            rgb: [81, 82, 83],
            locked: false,
            visible: true,
        },
    ];
    let plan = plan_reconcile_legacy(&before, &facts).unwrap();
    assert_eq!(plan.before, before);
    assert_eq!(plan.after.layers.len(), 3);
    assert_eq!(plan.after.layers[0], before.layers[0]);
    assert_eq!(plan.after.objects[0], before.objects[0]);
    assert_eq!(plan.after.active_layer, before.active_layer);
    assert_eq!(plan.after.generation, before.generation + 1);
    let native = effective_state(&plan.after, "native").unwrap();
    assert!(native.locked && !native.visible);
    assert_eq!(native.rgb, [31, 32, 33]);
    assert_eq!(plan.after.objects[1].color_source, ColorSource::ByObject);
    assert_eq!(plan.after.objects[2].layer_id, before.layers[0].source_id);
    assert_eq!(plan.after.objects[2].rgb, [81, 82, 83]);
    assert_ne!(
        plan.after.layers[1].source_id,
        plan.after.layers[0].source_id
    );
    assert_eq!(
        plan_reconcile_legacy(&plan.after, &facts).unwrap().after,
        plan.after
    );
}

#[test]
fn legacy_reconciliation_rejects_locked_removal_duplicate_facts_and_generation_overflow() {
    let mut before = plan_observed(&default_document("doc").unwrap(), &["a".into()])
        .unwrap()
        .after;
    before.objects[0].locked = true;
    assert_eq!(plan_reconcile_legacy(&before, &[]), Err(LayerError::Locked));
    let fact = LegacyObject {
        id: "new".into(),
        path: None,
        rgb: [1, 2, 3],
        locked: false,
        visible: true,
    };
    assert_eq!(
        plan_reconcile_legacy(&before, &[fact.clone(), fact.clone()]),
        Err(LayerError::DuplicateId)
    );
    before.objects[0].locked = false;
    before.generation = u64::MAX;
    assert_eq!(
        plan_reconcile_legacy(&before, &[fact]),
        Err(LayerError::LimitExceeded)
    );
}

#[test]
fn legacy_reconciliation_matches_full_paths_without_recoloring_existing_layers() {
    let before = migrate_legacy(
        "doc",
        &[LegacyObject {
            id: "a".into(),
            path: Some("Custom::Detail".into()),
            rgb: [1, 2, 3],
            locked: false,
            visible: true,
        }],
    )
    .unwrap();
    let facts = [
        LegacyObject {
            id: "a".into(),
            path: None,
            rgb: [0, 0, 0],
            locked: false,
            visible: true,
        },
        LegacyObject {
            id: "b".into(),
            path: Some("Custom::Detail".into()),
            rgb: [7, 8, 9],
            locked: false,
            visible: true,
        },
    ];
    let plan = plan_reconcile_legacy(&before, &facts).unwrap();
    assert_eq!(plan.after.layers, before.layers);
    assert_eq!(plan.after.objects[0], before.objects[0]);
    assert_eq!(plan.after.objects[1].layer_id, before.objects[0].layer_id);
    assert_eq!(plan.after.objects[1].rgb, [7, 8, 9]);
    let mut ambiguous = facts.clone();
    ambiguous[1].path = Some("custom::Detail".into());
    assert_eq!(
        plan_reconcile_legacy(&before, &ambiguous),
        Err(LayerError::AmbiguousPath)
    );
}

#[test]
fn created_objects_inherit_current_layer_and_color_without_geometry_input() {
    let mut before = default_document("doc").unwrap();
    before.active_layer = Some("om9-preset-5".into());
    let plan = plan_observed(&before, &["new-line".into(), "new-solid".into()]).unwrap();
    assert_eq!(before.objects.len(), 0);
    assert_eq!(plan.before, before);
    assert_eq!(plan.after.generation, 1);
    assert_eq!(plan.after.layers, before.layers);
    for row in &plan.after.objects {
        assert_eq!(row.layer_id, "om9-preset-5");
        assert_eq!(row.color_source, ColorSource::ByLayer);
        assert!(!row.locked && row.visible);
        assert_eq!(row.rgb, before.layers[4].rgb);
    }
    let unchanged = plan_observed(&plan.after, &["new-line".into(), "new-solid".into()]).unwrap();
    assert_eq!(unchanged.after, plan.after);
}
#[test]
fn observations_preserve_existing_own_flags_and_delete_is_all_or_nothing() {
    let before = plan_observed(&default_document("doc").unwrap(), &["a".into(), "b".into()])
        .unwrap()
        .after;
    let mut before = before;
    before.objects[0].locked = true;
    before.objects[0].color_source = ColorSource::ByObject;
    before.objects[0].rgb = [77, 88, 99];
    let plan = plan_observed(&before, &["a".into(), "b".into(), "c".into()]).unwrap();
    assert_eq!(plan.after.objects[0], before.objects[0]);
    assert_eq!(
        plan_observed(&before, &["c".into()]),
        Err(LayerError::Locked)
    );
    assert_eq!(
        plan_observed(&before, &["a".into()]).unwrap().after.objects,
        before.objects[..1]
    );
    assert_eq!(
        plan_observed(&before, &["a".into(), "a".into()]),
        Err(LayerError::DuplicateId)
    );
    assert_eq!(
        plan_observed(&before, &["".into()]),
        Err(LayerError::InvalidSnapshot)
    );
}
#[test]
fn create_rejects_ineligible_current_layer_and_generation_overflow() {
    let mut before = default_document("doc").unwrap();
    before.layers[0].locked = true;
    assert_eq!(
        plan_observed(&before, &["new".into()]),
        Err(LayerError::Locked)
    );
    before.layers[0].locked = false;
    before.layers[0].visible = false;
    assert_eq!(
        plan_observed(&before, &["new".into()]),
        Err(LayerError::Hidden)
    );
    before.layers[0].visible = true;
    before.generation = u64::MAX;
    assert_eq!(
        plan_observed(&before, &["new".into()]),
        Err(LayerError::LimitExceeded)
    );
    assert_eq!(plan_observed(&before, &[]).unwrap().after, before);
}
#[test]
fn legacy_observation_is_conservative_and_preserves_distinct_fullpaths() {
    let facts = vec![
        LegacyObject {
            id: "a".into(),
            path: Some("Custom::Detail".into()),
            rgb: [11, 22, 33],
            locked: true,
            visible: true,
        },
        LegacyObject {
            id: "b".into(),
            path: Some("Other::Detail".into()),
            rgb: [11, 22, 33],
            locked: false,
            visible: false,
        },
        LegacyObject {
            id: "c".into(),
            path: Some("Metal 01".into()),
            rgb: [44, 55, 66],
            locked: false,
            visible: true,
        },
    ];
    let state = migrate_legacy("legacy", &facts).unwrap();
    assert_eq!(state.layers.len(), 36);
    assert_eq!(state.objects.len(), 3);
    assert_ne!(state.objects[0].layer_id, state.objects[1].layer_id);
    assert_eq!(state.objects[2].layer_id, "om9-preset-1");
    assert!(state.objects[0].locked);
    assert!(!state.objects[1].visible);
    assert!(
        state
            .objects
            .iter()
            .all(|o| o.color_source == ColorSource::ByObject)
    );
    assert_eq!(effective_state(&state, "a").unwrap().rgb, [11, 22, 33]);
    assert!(state.layers.iter().all(|l| !l.locked && l.visible));
    assert_eq!(
        migrate_legacy("legacy", &[facts[0].clone(), facts[0].clone()]),
        Err(LayerError::DuplicateId)
    );
}
#[test]
fn legacy_invalid_path_or_ambiguous_case_cannot_silently_repair_identity() {
    let fact = |id: &str, path: &str| LegacyObject {
        id: id.into(),
        path: Some(path.into()),
        rgb: [1, 2, 3],
        locked: false,
        visible: true,
    };
    assert_eq!(
        migrate_legacy("doc", &[fact("a", "Custom::::Detail")]),
        Err(LayerError::InvalidPath)
    );
    assert_eq!(
        migrate_legacy("doc", &[fact("a", "Custom"), fact("b", "custom")]),
        Err(LayerError::AmbiguousPath)
    );
    assert_eq!(
        migrate_legacy("doc", &[fact("a", "")]),
        Err(LayerError::InvalidPath)
    );
}
#[test]
fn verified_metadata_filter_preserves_palette_context_and_model_flags_even_when_locked() {
    let mut state = snapshot();
    state.layers[0].locked = true;
    state.layers[0].visible = false;
    state.layers[2].persistent_locked = Some(true);
    let mut helper = object("verified-notes", "detail");
    helper.locked = true;
    helper.visible = false;
    state.objects.push(helper);
    let original = state.clone();
    let filtered = filter_verified_metadata(
        &state,
        &["verified-notes".into(), "not-yet-canonical".into()],
    )
    .unwrap();
    let mut expected = state.clone();
    expected.objects.pop();
    assert_eq!(filtered, expected);
    assert_eq!(state, original);
    assert_eq!(filter_verified_metadata(&filtered, &[]).unwrap(), filtered);
}

#[test]
fn verified_metadata_filter_rejects_ambiguous_ids_and_invalid_input_without_repair() {
    let state = snapshot();
    for ids in [
        vec!["".into()],
        vec!["bad\0id".into()],
        vec!["a".repeat(1025)],
    ] {
        assert_eq!(
            filter_verified_metadata(&state, &ids),
            Err(LayerError::InvalidSnapshot)
        );
    }
    assert_eq!(
        filter_verified_metadata(&state, &["ring".into(), "ring".into()]),
        Err(LayerError::DuplicateId)
    );
    assert_eq!(
        filter_verified_metadata(&state, &vec!["ring".into(); MAX_OBJECTS + 1]),
        Err(LayerError::LimitExceeded)
    );
    let mut invalid = state.clone();
    invalid.objects[0].layer_id = "missing".into();
    assert_eq!(
        filter_verified_metadata(&invalid, &["ring".into()]),
        Err(LayerError::MissingLayer)
    );
}
