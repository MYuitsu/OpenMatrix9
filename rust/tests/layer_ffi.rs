#[path = "common/layers.rs"]
mod common;
use common::*;
use openmatrix9_rust::{layer_ffi::*, layer_state::*};

fn bytes(s: &str) -> ByteView {
    ByteView {
        data: s.as_ptr(),
        len: s.len(),
    }
}
#[test]
fn metadata_filter_abi_owns_result_and_resets_invalid_outputs() {
    let mut state = snapshot();
    state.layers[0].locked = true;
    state.layers[0].visible = false;
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
    let mut helper_id = String::from("ring");
    let ids = [bytes(&helper_id)];
    let mut filtered = 0;
    assert_eq!(
        unsafe { om9_layer_snapshot_filter_metadata(source, ids.as_ptr(), 1, &mut filtered) },
        0
    );
    helper_id.clear();
    let mut before_counts = SnapshotCounts::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_counts(source, &mut before_counts) },
        0
    );
    assert_eq!(before_counts.object_count, 3);
    assert_eq!(om9_layer_snapshot_free(source), 0);
    let mut counts = SnapshotCounts::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_counts(filtered, &mut counts) },
        0
    );
    assert_eq!(
        (
            counts.layer_count,
            counts.object_count,
            counts.generation,
            counts.active_layer_index
        ),
        (3, 2, 7, 0)
    );
    let mut flags = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(filtered, bytes("detail-curve"), &mut flags) },
        0
    );
    assert_eq!((flags.locked, flags.visible), (1, 0));
    for (views, count, expected) in [
        (std::ptr::null(), MAX_OBJECTS + 1, LayerError::LimitExceeded),
        (std::ptr::null(), 1, LayerError::InvalidSnapshot),
    ] {
        let mut result = 77;
        assert_eq!(
            unsafe { om9_layer_snapshot_filter_metadata(filtered, views, count, &mut result) },
            expected as u32
        );
        assert_eq!(result, 0);
    }
    let duplicate = [bytes("stone"), bytes("stone")];
    let mut result = 77;
    assert_eq!(
        unsafe {
            om9_layer_snapshot_filter_metadata(
                filtered,
                duplicate.as_ptr(),
                duplicate.len(),
                &mut result,
            )
        },
        LayerError::DuplicateId as u32
    );
    assert_eq!(result, 0);
    result = 77;
    assert_eq!(
        unsafe { om9_layer_snapshot_filter_metadata(source, std::ptr::null(), 0, &mut result) },
        LayerError::InvalidHandle as u32
    );
    assert_eq!(result, 0);
    assert_eq!(om9_layer_snapshot_free(filtered), 0);
}
#[test]
fn legacy_reconciliation_abi_owns_facts_and_rejects_invalid_spans_without_writes() {
    let mut state = 0;
    assert_eq!(
        unsafe { om9_layer_document_default(bytes("doc"), &mut state) },
        0
    );
    let mut path = String::from("Native::Detail");
    let view = LegacyObjectView {
        id: bytes("new"),
        path: bytes(&path),
        rgb: [13, 14, 15],
        locked: 1,
        visible: 0,
        path_present: 1,
        reserved: [0; 2],
    };
    let mut plan = 77;
    assert_eq!(
        unsafe { om9_layer_document_reconcile(state, &view, 1, &mut plan) },
        0
    );
    path.clear();
    assert_eq!(om9_layer_snapshot_free(state), 0);
    let mut after = 0;
    assert_eq!(
        unsafe { om9_layer_plan_after_snapshot(plan, &mut after) },
        0
    );
    assert_eq!(om9_layer_plan_free(plan), 0);
    let mut effective = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(after, bytes("new"), &mut effective) },
        0
    );
    assert_eq!(
        (effective.rgb, effective.locked, effective.visible),
        ([13, 14, 15], 1, 0)
    );
    plan = 77;
    assert_eq!(
        unsafe {
            om9_layer_document_reconcile(after, std::ptr::null(), MAX_OBJECTS + 1, &mut plan)
        },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(plan, 0);
    plan = 77;
    let invalid = LegacyObjectView {
        id: bytes("new"),
        path: bytes(""),
        locked: 2,
        path_present: 0,
        ..view
    };
    assert_eq!(
        unsafe { om9_layer_document_reconcile(after, &invalid, 1, &mut plan) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(plan, 0);
    assert_eq!(om9_layer_snapshot_free(after), 0);
}
#[test]
fn legacy_and_observation_abi_own_inputs_and_zero_failed_outputs() {
    let mut path = String::from("Custom::Detail");
    let view = LegacyObjectView {
        id: bytes("old"),
        path: bytes(&path),
        rgb: [21, 22, 23],
        locked: 1,
        visible: 1,
        path_present: 1,
        reserved: [0; 2],
    };
    let mut state = 0;
    assert_eq!(
        unsafe { om9_layer_document_legacy(bytes("legacy"), &view, 1, &mut state) },
        0
    );
    path.clear();
    let mut flags = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(state, bytes("old"), &mut flags) },
        0
    );
    assert_eq!(flags.rgb, [21, 22, 23]);
    assert_eq!(flags.locked, 1);
    let ids = [bytes("old"), bytes("new")];
    let mut plan = 0;
    assert_eq!(
        unsafe { om9_layer_document_observed(state, ids.as_ptr(), 2, &mut plan) },
        0
    );
    let mut after = 0;
    assert_eq!(
        unsafe { om9_layer_plan_after_snapshot(plan, &mut after) },
        0
    );
    assert_eq!(om9_layer_plan_free(plan), 0);
    assert_eq!(om9_layer_snapshot_free(state), 0);
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(after, bytes("new"), &mut flags) },
        0
    );
    assert_eq!(flags.rgb, [34, 153, 135]);
    assert_eq!(flags.locked, 0);
    plan = 77;
    assert_eq!(
        unsafe { om9_layer_document_observed(after, ids[1..].as_ptr(), 1, &mut plan) },
        LayerError::Locked as u32
    );
    assert_eq!(plan, 0);
    assert_eq!(
        unsafe { om9_layer_document_observed(after, std::ptr::null(), MAX_OBJECTS + 1, &mut plan) },
        LayerError::LimitExceeded as u32
    );
    let invalid = LegacyObjectView {
        locked: 2,
        path: bytes(""),
        ..view
    };
    state = 77;
    assert_eq!(
        unsafe { om9_layer_document_legacy(bytes("legacy"), &invalid, 1, &mut state) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(state, 0);
    assert_eq!(om9_layer_snapshot_free(after), 0);
}
#[test]
fn panel_default_and_text_abi_return_owned_32_slots_and_guarded_results() {
    let mut state = 0;
    assert_eq!(
        unsafe { om9_layer_document_default(bytes("native-document"), &mut state) },
        0
    );
    let mut counts = SnapshotCounts::default();
    assert_eq!(unsafe { om9_layer_snapshot_counts(state, &mut counts) }, 0);
    assert_eq!(
        (counts.layer_count, counts.object_count, counts.generation),
        (32, 0, 0)
    );
    let mut needed = 0;
    assert_eq!(
        unsafe { om9_layer_document_panel(state, std::ptr::null_mut(), 0, &mut needed) },
        LayerError::BufferTooSmall as u32
    );
    let mut output = vec![0u8; needed];
    assert_eq!(
        unsafe { om9_layer_document_panel(state, output.as_mut_ptr(), output.len(), &mut needed) },
        0
    );
    let panel: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(panel["layers"][0]["path"], "Metal 01");
    assert_eq!(panel["layers"][31]["rgb"], serde_json::json!([27, 53, 85]));
    assert!(panel.get("objects").is_none());
    let mut plan = 0;
    assert_eq!(
        unsafe {
            om9_layer_document_text(
                state,
                bytes("Layer Lock \"Metal 01\" Toggle"),
                std::ptr::null(),
                0,
                &mut plan,
            )
        },
        0
    );
    assert_eq!(om9_layer_plan_validate(plan, state), 0);
    let mut after = 0;
    assert_eq!(
        unsafe { om9_layer_plan_after_snapshot(plan, &mut after) },
        0
    );
    assert_eq!(om9_layer_plan_free(plan), 0);
    let mut info = LayerInfo::default();
    assert_eq!(unsafe { om9_layer_snapshot_layer(after, 0, &mut info) }, 0);
    assert_eq!(info.locked, 1);
    let mut rejected = 99;
    assert_eq!(
        unsafe {
            om9_layer_document_text(
                after,
                bytes("Layer Current \"Metal 01\""),
                std::ptr::null(),
                0,
                &mut rejected,
            )
        },
        LayerError::Locked as u32
    );
    assert_eq!(rejected, 0);
    assert_eq!(om9_layer_snapshot_free(state), 0);
    assert_eq!(om9_layer_snapshot_free(after), 0);
}
#[test]
fn document_plan_abi_copies_command_and_survives_input_snapshot_release() {
    let current = create(&snapshot());
    let mut output = 0;
    let command = String::from(r#"{"operation":"set_locked","layer_id":"metal","locked":true}"#);
    assert_eq!(
        unsafe { om9_layer_document_plan(current, bytes(&command), &mut output) },
        0
    );
    drop(command);
    assert_ne!(output, 0);
    assert_eq!(om9_layer_plan_validate(output, current), 0);
    assert_eq!(om9_layer_snapshot_free(current), 0);
    let mut after = 0;
    assert_eq!(
        unsafe { om9_layer_plan_after_snapshot(output, &mut after) },
        0
    );
    assert_eq!(om9_layer_plan_free(output), 0);
    let mut state = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(after, bytes("detail-curve"), &mut state) },
        0
    );
    assert_eq!(
        (
            state.locked,
            state.visible,
            state.selectable,
            state.snap_eligible
        ),
        (1, 1, 0, 1)
    );
    let mut counts = SnapshotCounts::default();
    assert_eq!(unsafe { om9_layer_snapshot_counts(after, &mut counts) }, 0);
    assert_eq!(counts.generation, 8);
    assert_eq!(counts.active_layer_index, 1);
    assert_eq!(om9_layer_snapshot_free(after), 0);
}

#[test]
fn document_plan_abi_rejects_invalid_inputs_without_output_or_state_mutation() {
    let current = create(&snapshot());
    for command in [
        r#"{"operation":"assign","layer_id":"gem","object_ids":["ring","missing"]}"#,
        r#"{"operation":"set_locked","layer_id":"metal","locked":2}"#,
    ] {
        let mut output = 99;
        assert_ne!(
            unsafe { om9_layer_document_plan(current, bytes(command), &mut output) },
            0
        );
        assert_eq!(output, 0);
    }
    let mut output = 99;
    let huge = ByteView {
        data: std::ptr::null(),
        len: MAX_METADATA_BYTES + 1,
    };
    assert_eq!(
        unsafe { om9_layer_document_plan(current, huge, &mut output) },
        LayerError::LimitExceeded as u32
    );
    assert_eq!(output, 0);
    assert_eq!(
        unsafe { om9_layer_document_plan(current, bytes("{}"), std::ptr::null_mut()) },
        LayerError::InvalidSnapshot as u32
    );
    let mut state = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(current, bytes("ring"), &mut state) },
        0
    );
    assert_eq!((state.locked, state.visible), (0, 1));
    assert_eq!(om9_layer_snapshot_free(current), 0);
    assert_eq!(
        unsafe { om9_layer_document_plan(current, bytes("{}"), &mut output) },
        LayerError::InvalidHandle as u32
    );
    assert_eq!(output, 0);
}
fn create(s: &LayerSnapshotV1) -> u64 {
    let paths: Vec<String> = s
        .layers
        .iter()
        .map(|l| l.path_components.join("::"))
        .collect();
    let layers: Vec<LayerView> = s
        .layers
        .iter()
        .zip(&paths)
        .map(|(l, p)| LayerView {
            id: bytes(&l.source_id),
            parent: bytes(l.parent_id.as_deref().unwrap_or("")),
            name: bytes(&l.name),
            path: bytes(p),
            rgb: l.rgb,
            locked: u8::from(l.locked),
            visible: u8::from(l.visible),
            persistent_locked: l.persistent_locked.map_or(-1, i8::from),
            persistent_visible: l.persistent_visible.map_or(-1, i8::from),
            reserved: 0,
        })
        .collect();
    let objects: Vec<ObjectView> = s
        .objects
        .iter()
        .map(|o| ObjectView {
            id: bytes(&o.id),
            layer: bytes(&o.layer_id),
            rgb: o.rgb,
            locked: u8::from(o.locked),
            visible: u8::from(o.visible),
            color_source: o.color_source as u8,
            reserved: [0; 2],
        })
        .collect();
    let view = SnapshotView {
        version: 1,
        reserved: 0,
        document: bytes(&s.document_id),
        generation: s.generation,
        active: bytes(s.active_layer.as_deref().unwrap_or("")),
        layers: layers.as_ptr(),
        layer_count: layers.len(),
        objects: objects.as_ptr(),
        object_count: objects.len(),
    };
    let mut handle = 0;
    assert_eq!(unsafe { om9_layer_snapshot_create(&view, &mut handle) }, 0);
    assert_ne!(handle, 0);
    handle
}
#[test]
fn snapshot_abi_copies_caller_data_and_expired_handle_is_rejected() {
    let mut s = snapshot();
    s.layers[0].locked = true;
    let h = create(&s);
    drop(s);
    let mut e = EffectiveView::default();
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(h, bytes("ring"), &mut e) },
        0
    );
    assert_eq!(
        (e.locked, e.visible, e.selectable, e.snap_eligible),
        (1, 1, 0, 1)
    );
    assert_eq!(om9_layer_snapshot_free(h), 0);
    assert_eq!(
        unsafe { om9_layer_snapshot_effective(h, bytes("ring"), &mut e) },
        LayerError::InvalidHandle as u32
    );
    assert_eq!(om9_layer_snapshot_free(h), LayerError::InvalidHandle as u32);
}
#[test]
fn null_oversized_version_and_misaligned_views_reject_before_dereference() {
    let mut output = 999;
    unsafe {
        assert_eq!(
            om9_layer_snapshot_create(std::ptr::null(), &mut output),
            LayerError::InvalidSnapshot as u32
        );
        assert_eq!(output, 0);
        let view = SnapshotView {
            version: 1,
            reserved: 0,
            document: bytes("test"),
            generation: 0,
            active: bytes(""),
            layers: std::ptr::null(),
            layer_count: MAX_LAYERS + 1,
            objects: std::ptr::null(),
            object_count: 0,
        };
        assert_eq!(
            om9_layer_snapshot_create(&view, &mut output),
            LayerError::LimitExceeded as u32
        );
        let mut view = view;
        view.version = 99;
        assert_eq!(
            om9_layer_snapshot_create(&view, &mut output),
            LayerError::UnsupportedVersion as u32
        );
        assert_eq!(
            om9_layer_snapshot_create(&view, std::ptr::null_mut()),
            LayerError::InvalidSnapshot as u32
        );
        let storage = [0_u64; 16];
        let misaligned = storage.as_ptr().cast::<u8>().add(1).cast::<SnapshotView>();
        assert_eq!(
            om9_layer_snapshot_create(misaligned, &mut output),
            LayerError::InvalidSnapshot as u32
        );
    }
}
#[test]
fn invalid_native_boolean_and_utf8_reject_without_creating_handle() {
    let mut row = LayerView {
        id: bytes("metal"),
        parent: bytes(""),
        name: bytes("Metal"),
        path: bytes("Metal"),
        rgb: [0; 3],
        locked: 2,
        visible: 1,
        persistent_locked: -1,
        persistent_visible: -1,
        reserved: 0,
    };
    let mut view = SnapshotView {
        version: 1,
        reserved: 0,
        document: bytes("test"),
        generation: 0,
        active: bytes(""),
        layers: &row,
        layer_count: 1,
        objects: std::ptr::null(),
        object_count: 0,
    };
    let mut h = 4;
    assert_eq!(
        unsafe { om9_layer_snapshot_create(&view, &mut h) },
        LayerError::InvalidSnapshot as u32
    );
    assert_eq!(h, 0);
    row.locked = 0;
    row.persistent_visible = 2;
    view.layers = &row;
    assert_eq!(
        unsafe { om9_layer_snapshot_create(&view, &mut h) },
        LayerError::InvalidSnapshot as u32
    );
    row.persistent_visible = -1;
    let invalid = [255_u8];
    row.name = ByteView {
        data: invalid.as_ptr(),
        len: 1,
    };
    view.layers = &row;
    assert_eq!(
        unsafe { om9_layer_snapshot_create(&view, &mut h) },
        LayerError::InvalidSnapshot as u32
    );
}
#[test]
fn plan_after_snapshot_and_mapping_are_owned_and_stale_destination_rejects() {
    let source = create(&snapshot());
    let target_state = LayerSnapshotV1::empty("target");
    let target = create(&target_state);
    let mut plan = 0;
    assert_eq!(
        unsafe { om9_layer_plan_receive(source, target, 2, std::ptr::null(), 0, &mut plan) },
        0
    );
    assert_eq!(om9_layer_plan_validate(plan, target), 0);
    let mut other = target_state.clone();
    other.generation = 1;
    let stale = create(&other);
    assert_eq!(
        om9_layer_plan_validate(plan, stale),
        LayerError::Stale as u32
    );
    let mut after = 0;
    assert_eq!(
        unsafe { om9_layer_plan_after_snapshot(plan, &mut after) },
        0
    );
    assert_eq!(om9_layer_plan_free(plan), 0);
    let mut counts = SnapshotCounts::default();
    assert_eq!(unsafe { om9_layer_snapshot_counts(after, &mut counts) }, 0);
    assert_eq!(
        (counts.layer_count, counts.object_count, counts.generation),
        (3, 3, 1)
    );
    for h in [source, target, stale, after] {
        assert_eq!(om9_layer_snapshot_free(h), 0);
    }
}
#[test]
fn result_copy_reports_required_size_and_never_returns_internal_pointers() {
    let h = create(&snapshot());
    let mut needed = 0;
    assert_eq!(
        unsafe { om9_layer_snapshot_text(h, 1, 0, 3, std::ptr::null_mut(), 0, &mut needed) },
        LayerError::BufferTooSmall as u32
    );
    assert_eq!(needed, "Metal 01".len());
    let mut result = vec![0_u8; needed];
    assert_eq!(
        unsafe {
            om9_layer_snapshot_text(h, 1, 0, 3, result.as_mut_ptr(), result.len(), &mut needed)
        },
        0
    );
    assert_eq!(result, b"Metal 01");
    assert_eq!(
        unsafe {
            om9_layer_snapshot_text(
                h,
                1,
                9999,
                3,
                result.as_mut_ptr(),
                result.len(),
                &mut needed,
            )
        },
        LayerError::MissingLayer as u32
    );
    assert_eq!(om9_layer_snapshot_free(h), 0);
}
#[test]
fn mutation_batch_abi_uses_effective_lock_and_validates_all_ids() {
    let mut s = snapshot();
    s.layers[0].locked = true;
    let h = create(&s);
    let ids = [bytes("ring"), bytes("stone")];
    assert_eq!(
        unsafe { om9_layer_snapshot_can_mutate(h, ids.as_ptr(), 2, 1, bytes("")) },
        LayerError::Locked as u32
    );
    assert_eq!(
        unsafe { om9_layer_snapshot_can_mutate(h, ids.as_ptr(), 2, 5, bytes("")) },
        0
    );
    assert_eq!(
        unsafe { om9_layer_snapshot_can_mutate(h, ids.as_ptr(), 2, 999, bytes("")) },
        LayerError::UnsupportedVersion as u32
    );
    assert_eq!(om9_layer_snapshot_free(h), 0);
}
