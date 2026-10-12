use openmatrix9_rust::layer_ffi::*;
use openmatrix9_rust::{layer_document::default_document, layer_lifecycle::*, layer_state::*};

fn fact(id: &str, rgb: [u8; 3], locked: bool, visible: bool) -> LegacyObject {
    LegacyObject {
        id: id.into(),
        path: None,
        rgb,
        locked,
        visible,
    }
}

fn mixed_state() -> LayerSnapshotV1 {
    let mut state = default_document("mixed-panel").unwrap();
    state.active_layer = Some("om9-preset-7".into());
    state = plan_observed(&state, &["canonical".into()]).unwrap().after;
    let mut legacy = fact("legacy", [1, 2, 3], false, true);
    legacy.path = Some("Gem 01".into());
    plan_reconcile_legacy(
        &state,
        &[fact("canonical", [99, 99, 99], true, false), legacy],
    )
    .unwrap()
    .after
}

#[test]
fn verified_legacy_refresh_updates_only_own_flags_and_color_with_exact_before_guard() {
    let before = mixed_state();
    let plan = plan_refresh_verified_legacy(
        &before,
        &["legacy".into()],
        &[fact("legacy", [12, 100, 180], true, false)],
    )
    .unwrap();
    assert_eq!(plan.before, before);
    assert_eq!(plan.after.layers, before.layers);
    assert_eq!(plan.after.document_id, before.document_id);
    assert_eq!(plan.after.active_layer, before.active_layer);
    assert_eq!(plan.after.generation, before.generation + 1);
    assert_eq!(plan.after.objects[0], before.objects[0]);
    assert_eq!(plan.after.objects[1].layer_id, before.objects[1].layer_id);
    assert!(plan.layer_mapping.is_empty() && plan.object_mapping.is_empty());
    let effective = effective_state(&plan.after, "legacy").unwrap();
    assert!(effective.locked && !effective.visible);
    assert_eq!(effective.rgb, [12, 100, 180]);
    assert_eq!(plan.after.objects[1].color_source, ColorSource::ByObject);
    let released = plan_refresh_verified_legacy(
        &plan.after,
        &["legacy".into()],
        &[fact("legacy", [3, 4, 5], false, true)],
    )
    .unwrap();
    let effective = effective_state(&released.after, "legacy").unwrap();
    assert!(!effective.locked && effective.visible);
    assert_eq!(effective.rgb, [3, 4, 5]);
    assert_eq!(released.after.objects[0], before.objects[0]);
    let unchanged = plan_refresh_verified_legacy(
        &released.after,
        &["legacy".into()],
        &[fact("legacy", [3, 4, 5], false, true)],
    )
    .unwrap();
    assert_eq!(unchanged.after, released.after);
    let mut stale = before.clone();
    stale.generation += 1;
    assert_eq!(plan.check_destination(&stale), Err(LayerError::Stale));
}

#[test]
fn verified_refresh_rejects_unknown_or_mismatched_witness_and_duplicate_ids() {
    let state = mixed_state();
    let legacy = fact("legacy", [12, 100, 180], true, false);
    assert_eq!(
        plan_refresh_verified_legacy(
            &state,
            &["missing".into()],
            &[fact("missing", [1, 2, 3], false, true)]
        ),
        Err(LayerError::MissingObject)
    );
    assert_eq!(
        plan_refresh_verified_legacy(
            &state,
            &["legacy".into()],
            &[fact("canonical", [1, 2, 3], false, true)]
        ),
        Err(LayerError::InvalidSnapshot)
    );
    assert_eq!(
        plan_refresh_verified_legacy(&state, &[], &[legacy.clone()]),
        Err(LayerError::InvalidSnapshot)
    );
    assert_eq!(
        plan_refresh_verified_legacy(&state, &["legacy".into()], &[]),
        Err(LayerError::InvalidSnapshot)
    );
    assert_eq!(
        plan_refresh_verified_legacy(
            &state,
            &["legacy".into(), "legacy".into()],
            &[legacy.clone(), legacy.clone()]
        ),
        Err(LayerError::DuplicateId)
    );
    assert_eq!(
        plan_refresh_verified_legacy(&state, &["legacy".into()], &[legacy.clone(), legacy]),
        Err(LayerError::DuplicateId)
    );
    assert_eq!(state, mixed_state());
}

#[test]
fn verified_refresh_cannot_remap_paths_or_bypass_identity_and_metadata_budgets() {
    let state = mixed_state();
    let mut legacy = fact("legacy", [12, 100, 180], true, false);
    legacy.path = Some("Another::Layer".into());
    assert_eq!(
        plan_refresh_verified_legacy(&state, &["legacy".into()], &[legacy]),
        Err(LayerError::InvalidPath)
    );
    for id in ["".to_owned(), "forged\0id".to_owned(), "a".repeat(1025)] {
        assert_eq!(
            plan_refresh_verified_legacy(
                &state,
                &[id.clone()],
                &[fact(&id, [1, 2, 3], false, true)]
            ),
            Err(LayerError::InvalidSnapshot)
        );
    }
    assert_eq!(
        plan_refresh_verified_legacy(&state, &vec!["legacy".into(); MAX_OBJECTS + 1], &[]),
        Err(LayerError::LimitExceeded)
    );
    let mut maximum = state.clone();
    maximum.generation = u64::MAX;
    assert_eq!(
        plan_refresh_verified_legacy(
            &maximum,
            &["legacy".into()],
            &[fact("legacy", [1, 2, 3], false, true)]
        )
        .unwrap()
        .after,
        maximum
    );
    assert_eq!(
        plan_refresh_verified_legacy(
            &maximum,
            &["legacy".into()],
            &[fact("legacy", [4, 5, 6], false, true)]
        ),
        Err(LayerError::LimitExceeded)
    );
}

fn bytes(text: &str) -> ByteView {
    ByteView {
        data: text.as_ptr(),
        len: text.len(),
    }
}

#[test]
fn verified_refresh_abi_copies_facts_and_rejects_bad_spans_before_publication() {
    let state = mixed_state();
    let encoded = openmatrix9_rust::layer_codec::encode_snapshot(&state).unwrap();
    let mut source = 0;
    assert_eq!(
        unsafe {
            om9_layer_snapshot_from_json(
                ByteView {
                    data: encoded.as_ptr(),
                    len: encoded.len(),
                },
                &mut source,
            )
        },
        0
    );
    let mut id = String::from("legacy");
    let witness = [bytes(&id)];
    let view = LegacyObjectView {
        id: bytes(&id),
        path: bytes(""),
        rgb: [12, 100, 180],
        locked: 1,
        visible: 0,
        path_present: 0,
        reserved: [0; 2],
    };
    let mut plan = 77;
    assert_eq!(
        unsafe {
            om9_layer_document_refresh_verified_legacy(
                source,
                witness.as_ptr(),
                1,
                &view,
                1,
                &mut plan,
            )
        },
        0
    );
    id.clear();
    let mut after = 0;
    assert_eq!(
        unsafe { om9_layer_plan_after_snapshot(plan, &mut after) },
        0
    );
    assert_eq!(om9_layer_plan_free(plan), 0);
    let mut effective = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(after, bytes("legacy"), &mut effective) },
        0
    );
    assert_eq!(
        (effective.rgb, effective.locked, effective.visible),
        ([12, 100, 180], 1, 0)
    );
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(after, bytes("canonical"), &mut effective) },
        0
    );
    let canonical = effective_state(&state, "canonical").unwrap();
    assert_eq!(
        (effective.rgb, effective.locked, effective.visible),
        (
            canonical.rgb,
            u8::from(canonical.locked),
            u8::from(canonical.visible)
        )
    );
    for (id_count, fact_count, expected) in [
        (MAX_OBJECTS + 1, 0, LayerError::LimitExceeded),
        (0, MAX_OBJECTS + 1, LayerError::LimitExceeded),
        (1, 0, LayerError::InvalidSnapshot),
        (0, 1, LayerError::InvalidSnapshot),
    ] {
        plan = 77;
        assert_eq!(
            unsafe {
                om9_layer_document_refresh_verified_legacy(
                    source,
                    std::ptr::null(),
                    id_count,
                    std::ptr::null(),
                    fact_count,
                    &mut plan,
                )
            },
            expected as u32
        );
        assert_eq!(plan, 0);
    }
    let witness = [bytes("legacy")];
    for view in [
        LegacyObjectView {
            id: bytes("legacy"),
            locked: 2,
            ..view
        },
        LegacyObjectView {
            id: bytes("legacy"),
            reserved: [1, 0],
            ..view
        },
    ] {
        plan = 77;
        assert_eq!(
            unsafe {
                om9_layer_document_refresh_verified_legacy(
                    source,
                    witness.as_ptr(),
                    1,
                    &view,
                    1,
                    &mut plan,
                )
            },
            LayerError::InvalidSnapshot as u32
        );
        assert_eq!(plan, 0);
    }
    assert_eq!(om9_layer_snapshot_free(source), 0);
    assert_eq!(om9_layer_snapshot_free(after), 0);
}
