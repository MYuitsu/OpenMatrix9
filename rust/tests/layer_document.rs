#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_document::*, layer_state::*};

#[test]
fn default_palette_is_32_empty_distinct_slots_in_existing_sidebar_order() {
    let state = default_document("fresh-document").unwrap();
    assert_eq!(state.layers.len(), 32);
    assert!(state.objects.is_empty());
    assert_eq!(state.layers[0].name, "Metal 01");
    assert_eq!(state.layers[0].rgb, [34, 153, 135]);
    assert_eq!(state.layers[15].name, "Creation");
    assert_eq!(state.layers[20].name, "User 25");
    assert_eq!(state.layers[24].name, "User 21");
    assert_eq!(state.layers[31].rgb, [27, 53, 85]);
    assert_eq!(
        state.active_layer.as_deref(),
        Some(state.layers[0].source_id.as_str())
    );
    assert!(
        state
            .layers
            .iter()
            .all(|row| !row.locked && row.visible && row.parent_id.is_none())
    );
    assert!(validate_layers(&state).is_ok());
    assert_eq!(default_document(""), Err(LayerError::InvalidSnapshot));
}

#[test]
fn panel_view_uses_current_palette_and_rust_inherited_eligibility_without_objects() {
    let mut state = snapshot();
    state.layers[0].rgb = [201, 202, 203];
    state.layers[0].locked = true;
    let view: serde_json::Value = serde_json::from_slice(&panel_json(&state).unwrap()).unwrap();
    assert!(view.get("objects").is_none());
    assert_eq!(view["layers"][0]["rgb"], serde_json::json!([201, 202, 203]));
    assert_eq!(view["layers"][2]["path"], "Metal 01::Detail");
    assert_eq!(view["layers"][2]["locked"], false);
    assert_eq!(view["layers"][2]["effective_locked"], true);
    assert_eq!(view["layers"][2]["can_current"], false);
    assert_eq!(view["generation"], "7");
}

#[test]
fn text_commands_resolve_exact_fullpath_and_share_guarded_document_plan() {
    let mut state = snapshot();
    state.layers.push(layer("other", None, &["Other"]));
    state
        .layers
        .push(layer("other-detail", Some("other"), &["Other", "Detail"]));
    let p = plan_text(&state, "Layer Current \"Other::Detail\"", &[]).unwrap();
    assert_eq!(p.after.active_layer.as_deref(), Some("other-detail"));
    let p = plan_text(&state, "Layer Color \"Metal 01\" #C90102", &[]).unwrap();
    assert_eq!(p.after.layers[0].rgb, [201, 1, 2]);
    let p = plan_text(&state, "Layer Lock \"Metal 01::Detail\" Toggle", &[]).unwrap();
    assert!(p.after.layers[2].locked);
    assert!(!p.after.layers[4].locked);
    assert_eq!(
        plan_text(&p.after, "Layer Current \"Metal 01::Detail\"", &[]),
        Err(LayerError::Locked)
    );
    let p = plan_text(&state, "Layer Assign \"Gem 01\"", &["ring".into()]).unwrap();
    assert_eq!(p.after.objects[0].layer_id, "gem");
    assert_eq!(
        plan_text(&state, "Layer Assign \"Gem 01\"", &[]),
        Err(LayerError::EmptySelection)
    );
    for text in [
        "Layer Color Gem 01 #gg0000",
        "Layer Lock Metal 01 yes",
        "Layer Current Detail",
        "Layer Unknown Metal 01",
    ] {
        assert!(plan_text(&state, text, &[]).is_err());
    }
}

#[test]
fn current_layer_requires_visible_unlocked_hierarchy() {
    let mut state = snapshot();
    state.layers[0].locked = true;
    let before = state.clone();
    assert_eq!(
        plan_command(
            &state,
            LayerCommand::SetCurrent {
                layer_id: "detail".into()
            }
        ),
        Err(LayerError::Locked)
    );
    state.layers[0].locked = false;
    state.layers[0].visible = false;
    assert_eq!(
        plan_command(
            &state,
            LayerCommand::SetCurrent {
                layer_id: "detail".into()
            }
        ),
        Err(LayerError::Hidden)
    );
    assert_eq!(before.generation, state.generation);
    let plan = plan_command(
        &state,
        LayerCommand::SetCurrent {
            layer_id: "gem".into(),
        },
    )
    .unwrap();
    assert_eq!(plan.before, state);
    assert_eq!(plan.after.active_layer.as_deref(), Some("gem"));
    assert_eq!(plan.after.generation, 8);
}

#[test]
fn parent_lock_and_visibility_changes_preserve_child_and_object_own_state() {
    let mut state = snapshot();
    state.layers[2].persistent_locked = Some(false);
    state.layers[2].persistent_visible = Some(true);
    state.objects[0].locked = true;
    let child_before = state.layers[2].clone();
    let objects_before = state.objects.clone();
    let locked = plan_command(
        &state,
        LayerCommand::SetLocked {
            layer_id: "metal".into(),
            locked: true,
        },
    )
    .unwrap()
    .after;
    assert_eq!(locked.layers[2], child_before);
    assert_eq!(locked.objects, objects_before);
    assert!(effective_state(&locked, "detail-curve").unwrap().locked);
    assert_eq!(locked.active_layer.as_deref(), Some("gem"));
    let unlocked = plan_command(
        &locked,
        LayerCommand::SetLocked {
            layer_id: "metal".into(),
            locked: false,
        },
    )
    .unwrap()
    .after;
    assert!(!effective_state(&unlocked, "detail-curve").unwrap().locked);
    assert!(effective_state(&unlocked, "ring").unwrap().locked);
    let hidden = plan_command(
        &unlocked,
        LayerCommand::SetVisible {
            layer_id: "metal".into(),
            visible: false,
        },
    )
    .unwrap()
    .after;
    assert!(!effective_state(&hidden, "detail-curve").unwrap().visible);
    let shown = plan_command(
        &hidden,
        LayerCommand::SetVisible {
            layer_id: "metal".into(),
            visible: true,
        },
    )
    .unwrap()
    .after;
    assert!(effective_state(&shown, "detail-curve").unwrap().visible);
    assert_eq!(shown.layers[2], child_before);
    assert_eq!(shown.objects, objects_before);
}

#[test]
fn explicit_child_toggle_records_desired_persistent_state_under_parent() {
    let mut state = snapshot();
    state.layers[0].locked = true;
    state.layers[0].visible = false;
    let plan = plan_command(
        &state,
        LayerCommand::SetLocked {
            layer_id: "detail".into(),
            locked: true,
        },
    )
    .unwrap();
    assert_eq!(plan.after.layers[2].persistent_locked, Some(true));
    let plan = plan_command(
        &plan.after,
        LayerCommand::SetVisible {
            layer_id: "detail".into(),
            visible: false,
        },
    )
    .unwrap();
    assert_eq!(plan.after.layers[2].persistent_visible, Some(false));
    assert!(!plan.after.layers[2].visible);
    assert!(plan.after.layers[2].locked);
}

#[test]
fn locking_last_usable_layer_creates_unique_fallback_without_unlocking_palette() {
    let mut state = snapshot();
    state.layers[1].locked = true;
    state
        .layers
        .push(layer("taken", None, &["OM9 Transfer Work"]));
    state.layers[3].visible = false;
    let plan = plan_command(
        &state,
        LayerCommand::SetLocked {
            layer_id: "metal".into(),
            locked: true,
        },
    )
    .unwrap();
    assert!(plan.after.layers[0].locked);
    assert!(plan.after.layers[1].locked);
    assert!(!plan.after.layers[3].visible);
    let fallback = plan.after.layers.last().unwrap();
    assert_eq!(fallback.name, "OM9 Transfer Work 2");
    assert_eq!(
        plan.after.active_layer.as_deref(),
        Some(fallback.source_id.as_str())
    );
    assert!(
        plan.after
            .objects
            .iter()
            .zip(&state.objects)
            .all(|(a, b)| a == b)
    );
}

#[test]
fn assign_is_whole_selection_guarded_and_keeps_by_object_override() {
    let mut state = snapshot();
    state.objects[0].color_source = ColorSource::ByObject;
    state.objects[0].rgb = [111, 112, 113];
    state.objects[1].locked = true;
    let before = state.clone();
    assert_eq!(
        plan_command(
            &state,
            LayerCommand::Assign {
                layer_id: "detail".into(),
                object_ids: vec!["ring".into(), "stone".into()]
            }
        ),
        Err(LayerError::Locked)
    );
    assert_eq!(state, before);
    let plan = plan_command(
        &state,
        LayerCommand::Assign {
            layer_id: "detail".into(),
            object_ids: vec!["ring".into()],
        },
    )
    .unwrap();
    assert_eq!(plan.after.objects[0].layer_id, "detail");
    assert_eq!(
        effective_state(&plan.after, "ring").unwrap().rgb,
        [111, 112, 113]
    );
    assert_eq!(plan.after.objects[1..], state.objects[1..]);
    state.layers[2].visible = false;
    assert_eq!(
        plan_command(
            &state,
            LayerCommand::Assign {
                layer_id: "detail".into(),
                object_ids: vec!["ring".into()]
            }
        ),
        Err(LayerError::Hidden)
    );
}

#[test]
fn layer_color_updates_by_layer_projection_only_and_rejects_stale_plan() {
    let mut state = snapshot();
    state.objects[2].layer_id = "metal".into();
    state.objects[2].color_source = ColorSource::ByObject;
    let plan = plan_command(
        &state,
        LayerCommand::SetColor {
            layer_id: "metal".into(),
            rgb: [201, 202, 203],
        },
    )
    .unwrap();
    assert_eq!(plan.after.objects, state.objects);
    assert_eq!(
        effective_state(&plan.after, "ring").unwrap().rgb,
        [201, 202, 203]
    );
    assert_eq!(
        effective_state(&plan.after, "detail-curve").unwrap().rgb,
        [9, 8, 7]
    );
    assert!(plan.check_destination(&state).is_ok());
    assert_eq!(plan.check_destination(&plan.after), Err(LayerError::Stale));
    state.layers[0].name = "changed without generation".into();
    assert_eq!(plan.check_destination(&state), Err(LayerError::Stale));
}

#[test]
fn no_op_is_generation_stable_and_real_change_overflow_rejects() {
    let mut state = snapshot();
    state.generation = u64::MAX;
    assert_eq!(
        plan_command(
            &state,
            LayerCommand::SetCurrent {
                layer_id: "metal".into()
            }
        )
        .unwrap()
        .after,
        state
    );
    assert_eq!(
        plan_command(
            &state,
            LayerCommand::SetColor {
                layer_id: "metal".into(),
                rgb: [0, 0, 0]
            }
        ),
        Err(LayerError::LimitExceeded)
    );
    assert_eq!(state.generation, u64::MAX);
}

#[test]
fn command_json_is_strict_typed_and_does_not_coerce_or_ignore_fields() {
    assert_eq!(
        decode_command(br#"{"operation":"set_color","layer_id":"metal","rgb":[1,2,3]}"#).unwrap(),
        LayerCommand::SetColor {
            layer_id: "metal".into(),
            rgb: [1, 2, 3]
        }
    );
    for bad in [
        br#"{"operation":"set_locked","layer_id":"metal","locked":1}"#.as_slice(),
        br#"{"operation":"set_locked","layer_id":"metal","locked":true,"extra":1}"#,
        br#"{"operation":"set_color","layer_id":"metal","rgb":[256,2,3]}"#,
        br#"{"operation":"set_current","layer_id":"metal","layer_id":"gem"}"#,
        br#"{"operation":"assign","layer_id":"metal","object_ids":null}"#,
        br#"{"operation":"unknown","layer_id":"metal"}"#,
    ] {
        assert_eq!(decode_command(bad), Err(LayerError::InvalidSnapshot));
    }
}

#[test]
fn assignment_unknown_duplicate_empty_and_overlong_input_are_rejected() {
    let state = snapshot();
    for (ids, error) in [
        (vec![], LayerError::EmptySelection),
        (
            vec!["ring".into(), "missing".into()],
            LayerError::MissingObject,
        ),
        (vec!["ring".into(), "ring".into()], LayerError::DuplicateId),
    ] {
        assert_eq!(
            plan_command(
                &state,
                LayerCommand::Assign {
                    layer_id: "gem".into(),
                    object_ids: ids
                }
            ),
            Err(error)
        );
    }
    let command = format!(
        r#"{{"operation":"assign","layer_id":"gem","object_ids":["{}"]}}"#,
        "x".repeat(1025)
    );
    assert_eq!(
        decode_command(command.as_bytes()),
        Err(LayerError::InvalidSnapshot)
    );
}
