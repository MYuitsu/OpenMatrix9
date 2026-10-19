// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::{layer_dialog::LayerDialogKind, session::Controller};
use om9_core::{
    Error,
    budget::Limits,
    geometry::Geometry,
    layers::PresetSlot,
    model::{Document, Entity, Layer},
};
use std::sync::Arc;
fn slot(n: u8) -> PresetSlot {
    PresetSlot::new(n).unwrap()
}
fn custom_fixture() -> Document {
    let mut d = Document::new("owned dialog fixture");
    for n in 17..=20 {
        let mut layer = Layer::preset(slot(n));
        if n == 17 {
            layer.color = [10, 20, 30, 90];
            d.current_layer = Some(layer.id);
        }
        d.layer_order.push(layer.id);
        d.layers.insert(layer.id, layer);
    }
    let mut root = Entity::new("member", Geometry::Point([1., 2., 3.]), d.current_layer);
    root.attributes.color = None;
    d.entities.insert(root.id, root);
    d
}
#[test]
fn open_cancel_does_not_materialize_lazy_layers_or_consume_redo() {
    let mut c = Controller::new(Document::new("lazy modal"), Limits::default()).unwrap();
    let before = c.coordinator().snapshot();
    let token = c
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot(17)))
        .unwrap();
    assert_eq!(c.layer_dialog().unwrap().name(), Some("User 17"));
    assert_eq!(c.coordinator().snapshot(), before);
    c.set_layer_dialog_name(token, "Đá xanh").unwrap();
    c.cancel_layer_dialog(token).unwrap();
    assert!(c.layer_dialog().is_none());
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(matches!(
        c.confirm_layer_dialog(token),
        Err(Error::Stale(_))
    ));
}
#[test]
fn rename_confirmation_materializes_one_group_without_changing_current_or_objects() {
    let mut c = Controller::new(Document::new("lazy rename"), Limits::default()).unwrap();
    let token = c
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot(21)))
        .unwrap();
    c.set_layer_dialog_name(token, "  Đá quý  ").unwrap();
    c.confirm_layer_dialog(token).unwrap();
    let id = c.layer_id(slot(21)).unwrap();
    assert_eq!(c.document().layers[&id].name, "Đá quý");
    assert_eq!(c.document().layers.len(), 4);
    assert!(c.document().current_layer.is_none());
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert!(c.document().layers.is_empty());
    c.redo().unwrap();
    assert_eq!(c.layer_id(slot(21)), Some(id));
}
#[test]
fn color_confirmation_refreshes_by_layer_scene_and_preserves_alpha_identity_and_geometry() {
    let mut c = Controller::new(custom_fixture(), Limits::default()).unwrap();
    let id = c.layer_id(slot(17)).unwrap();
    let before = c.document().entities.clone();
    let buffers = c.scene().unwrap().objects[0].buffers.clone();
    let token = c
        .begin_layer_dialog(LayerDialogKind::Color, Some(slot(17)))
        .unwrap();
    c.set_layer_dialog_color(token, [21, 43, 65]).unwrap();
    c.confirm_layer_dialog(token).unwrap();
    assert_eq!(c.document().layers[&id].color, [21, 43, 65, 90]);
    assert_eq!(c.document().current_layer, Some(id));
    assert_eq!(c.document().entities, before);
    assert_eq!(c.scene().unwrap().objects[0].color, [21, 43, 65, 90]);
    assert!(Arc::ptr_eq(
        &buffers,
        &c.scene().unwrap().objects[0].buffers
    ));
    c.undo().unwrap();
    assert_eq!(c.document().layers[&id].color, [10, 20, 30, 90]);
    c.redo().unwrap();
    assert_eq!(c.document().layers[&id].color, [21, 43, 65, 90]);
}
#[test]
fn invalid_names_leave_dialog_editable_and_document_unchanged() {
    let mut d = custom_fixture();
    let other = Layer::new("Other", [255; 4]);
    d.layer_order.push(other.id);
    d.layers.insert(other.id, other);
    let mut c = Controller::new(d, Limits::default()).unwrap();
    let token = c
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot(17)))
        .unwrap();
    let before = c.coordinator().snapshot();
    for name in [
        " ".to_owned(),
        "Metal 01".to_owned(),
        "USER 18".to_owned(),
        "other".to_owned(),
        "bad\nname".to_owned(),
        "石".repeat(43),
    ] {
        c.set_layer_dialog_name(token, &name).unwrap();
        assert!(c.confirm_layer_dialog(token).is_err(), "{name:?}");
        assert_eq!(c.coordinator().snapshot(), before);
        assert!(c.layer_dialog().is_some());
    }
    assert!(c.set_layer_dialog_name(token, &"x".repeat(513)).is_err());
    c.set_layer_dialog_name(token, &"x".repeat(128)).unwrap();
    c.confirm_layer_dialog(token).unwrap();
    assert_eq!(
        c.document().layers[&c.layer_id(slot(17)).unwrap()]
            .name
            .len(),
        128
    );
}
#[test]
fn basic_lights_and_conflicting_dialog_requests_refuse_before_mutation() {
    let mut c = Controller::new(Document::new("dialog guards"), Limits::default()).unwrap();
    let before = c.coordinator().snapshot();
    for n in [1, 16, 33] {
        for kind in [LayerDialogKind::Rename, LayerDialogKind::Color] {
            assert!(matches!(
                c.begin_layer_dialog(kind, Some(slot(n))),
                Err(Error::Permission(_))
            ));
        }
    }
    assert_eq!(c.coordinator().snapshot(), before);
    let token = c
        .begin_layer_dialog(LayerDialogKind::Color, Some(slot(17)))
        .unwrap();
    assert!(matches!(
        c.begin_layer_dialog(LayerDialogKind::Show, None),
        Err(Error::Busy)
    ));
    c.cancel_layer_dialog(token).unwrap();
}
#[test]
fn old_and_foreign_tokens_cannot_mutate_a_new_dialog_or_a_reopened_document() {
    let d = custom_fixture();
    let mut a = Controller::new(d.clone(), Limits::default()).unwrap();
    let old = a
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot(17)))
        .unwrap();
    a.cancel_layer_dialog(old).unwrap();
    let newer = a
        .begin_layer_dialog(LayerDialogKind::Color, Some(slot(17)))
        .unwrap();
    assert!(matches!(a.confirm_layer_dialog(old), Err(Error::Stale(_))));
    assert!(a.cancel_layer_dialog(old).is_err());
    assert_eq!(a.layer_dialog().unwrap().token(), newer);
    let mut reopened = d;
    reopened.renew_session(&Limits::default()).unwrap();
    let mut b = Controller::new(reopened, Limits::default()).unwrap();
    let current = b
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot(17)))
        .unwrap();
    let before = b.coordinator().snapshot();
    assert!(b.set_layer_dialog_name(newer, "foreign write").is_err());
    assert!(b.confirm_layer_dialog(newer).is_err());
    assert_eq!(b.coordinator().snapshot(), before);
    assert_eq!(b.layer_dialog().unwrap().token(), current);
}
#[test]
fn undo_roundtrip_does_not_revive_a_dialog_snapshot() {
    use om9_app::layer_command::LayerProperty;
    let mut c = Controller::new(custom_fixture(), Limits::default()).unwrap();
    let before = c.document().clone();
    let token = c
        .begin_layer_dialog(LayerDialogKind::Rename, Some(slot(17)))
        .unwrap();
    c.set_layer_property(slot(17), LayerProperty::Visible(false))
        .unwrap();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    let snapshot = c.coordinator().snapshot();
    assert!(matches!(
        c.confirm_layer_dialog(token),
        Err(Error::Stale(_))
    ));
    assert_eq!(c.coordinator().snapshot(), snapshot);
    c.cancel_layer_dialog(token).unwrap();
}
#[test]
fn selective_show_changes_only_chosen_hidden_flag_and_keeps_canvas_selection_and_masks() {
    let mut d = Document::new("selective Show");
    let mut layer = Layer::preset(slot(1));
    layer.visible = false;
    layer.locked = true;
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut chosen = Entity::new("chosen off locked", Geometry::Point([0.; 3]), Some(lid));
    chosen.attributes.hidden = true;
    chosen.attributes.selectable = false;
    let chosen_id = chosen.id;
    d.entities.insert(chosen_id, chosen);
    let mut other = Entity::new("other invisible", Geometry::Point([1.; 3]), None);
    other.attributes.hidden = true;
    other.attributes.visible = false;
    let other_id = other.id;
    d.entities.insert(other_id, other);
    let visible = Entity::new("canvas selected", Geometry::Point([2.; 3]), None);
    let selected = visible.id;
    d.entities.insert(selected, visible);
    let mut c = Controller::new(d, Limits::default()).unwrap();
    c.select_root(Some(selected), false).unwrap();
    let token = c.begin_layer_dialog(LayerDialogKind::Show, None).unwrap();
    assert_eq!(c.layer_dialog().unwrap().entries().len(), 2);
    assert!(
        c.layer_dialog()
            .unwrap()
            .entries()
            .iter()
            .all(|e| !e.selected())
    );
    let index = c
        .layer_dialog()
        .unwrap()
        .entries()
        .iter()
        .position(|e| e.source().object == chosen_id)
        .unwrap();
    c.set_layer_dialog_choice(token, index, true).unwrap();
    c.confirm_layer_dialog(token).unwrap();
    assert!(!c.document().entities[&chosen_id].attributes.hidden);
    assert!(!c.document().entities[&chosen_id].attributes.selectable);
    assert!(c.document().entities[&other_id].attributes.hidden);
    assert!(
        !c.scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == chosen_id)
            .unwrap()
            .visible
    );
    assert_eq!(
        c.selected().iter().copied().collect::<Vec<_>>(),
        vec![selected]
    );
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert!(c.document().entities[&chosen_id].attributes.hidden);
    c.redo().unwrap();
    assert!(!c.document().entities[&chosen_id].attributes.hidden);
}
#[test]
fn empty_confirmation_and_existing_customization_noop_preserve_redo() {
    let mut c = Controller::new(custom_fixture(), Limits::default()).unwrap();
    c.submit("Point").unwrap();
    c.submit("3,4,5").unwrap();
    c.undo().unwrap();
    let before = c.coordinator().snapshot();
    for (kind, target) in [
        (LayerDialogKind::Rename, Some(slot(17))),
        (LayerDialogKind::Color, Some(slot(17))),
        (LayerDialogKind::Show, None),
    ] {
        let token = c.begin_layer_dialog(kind, target).unwrap();
        c.confirm_layer_dialog(token).unwrap();
        assert_eq!(c.coordinator().snapshot(), before);
        assert!(c.layer_dialog().is_none());
    }
    assert_eq!(c.coordinator().redo_len(), 1);
}
#[test]
fn hidden_snapshot_count_is_admitted_before_allocation() {
    let mut d = Document::new("hidden snapshot budget");
    for _ in 0..2 {
        let mut e = Entity::new("hidden", Geometry::Point([0.; 3]), None);
        e.attributes.hidden = true;
        d.entities.insert(e.id, e);
    }
    let mut c = Controller::new(
        d,
        Limits {
            max_outputs: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let before = c.coordinator().snapshot();
    assert!(matches!(
        c.begin_layer_dialog(LayerDialogKind::Show, None),
        Err(Error::Budget(_))
    ));
    assert!(c.layer_dialog().is_none());
    assert_eq!(c.coordinator().snapshot(), before);
}
#[test]
fn empty_show_confirmation_needs_no_mutation_buffers_in_a_tight_budget() {
    let d = Document::new("tight no-op dialog");
    let probe = Controller::new(d.clone(), Limits::default()).unwrap();
    let base = probe
        .coordinator()
        .retained_usage_with_held_usage(probe.scene_cache_usage().unwrap())
        .unwrap()
        .bytes;
    let mut c = Controller::new(
        d,
        Limits {
            max_bytes: base + 8192,
            ..Limits::default()
        },
    )
    .unwrap();
    let token = c.begin_layer_dialog(LayerDialogKind::Show, None).unwrap();
    let epoch = c.coordinator().epoch();
    c.confirm_layer_dialog(token).unwrap();
    assert!(c.layer_dialog().is_none());
    assert_eq!(c.coordinator().epoch(), epoch);
    assert_eq!(c.coordinator().undo_len(), 0);
}
