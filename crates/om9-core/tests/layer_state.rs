// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    layers::{LayerAction, LayerPlan, PresetSlot},
    model::{Document, Entity, Layer},
    transaction::*,
};
#[test]
fn preset_slot_is_durable_identity_not_label_and_duplicate_slots_refuse() {
    let slot = PresetSlot::new(17).unwrap();
    assert_eq!(slot.name(), "User 17");
    assert!(slot.is_custom());
    assert_eq!(PresetSlot::new(21).unwrap().name(), "User 25");
    assert_eq!(PresetSlot::new(33).unwrap().name(), "Lights");
    assert_eq!(PresetSlot::new(33).unwrap().color(), [255; 4]);
    assert!(PresetSlot::new(0).is_err());
    assert!(PresetSlot::new(34).is_err());
    let mut d = Document::new("preset identity");
    let mut layer = Layer::preset(slot);
    let lid = layer.id;
    layer.name = "Custom label".into();
    d.layers.insert(lid, layer.clone());
    d.layer_order.push(lid);
    d.validate(&Limits::default()).unwrap();
    assert_eq!(d.layers[&lid].preset_slot, Some(slot));
    let other = Layer::preset(slot);
    let oid = other.id;
    d.layers.insert(oid, other);
    d.layer_order.push(oid);
    assert!(d.validate(&Limits::default()).is_err());
}
#[test]
fn hiding_and_showing_do_not_restore_explicit_invisible_or_unselectable_state() {
    let limits = Limits::default();
    let mut d = Document::new("independent Hidden");
    let layer = Layer::preset(PresetSlot::new(1).unwrap());
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut e = Entity::new(
        "explicit invisible",
        Geometry::Point([1., 2., 3.]),
        Some(lid),
    );
    e.attributes.visible = false;
    e.attributes.selectable = false;
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    let mut c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    for hidden in [true, false] {
        let source = c.world().documents[&did].stamp(id).unwrap();
        let p = LayerPlan::capture(
            &c,
            did,
            LayerAction::Hidden {
                sources: vec![source],
                hidden,
            },
            None,
        )
        .unwrap();
        let p = c
            .prepare(TransactionRequest::new(
                "Hide/Show",
                vec![Mutation::Layers(Box::new(p))],
            ))
            .unwrap();
        c.commit(p).unwrap();
        let d = &c.world().documents[&did];
        assert_eq!(d.entities[&id].attributes.hidden, hidden);
        assert!(!d.entities[&id].attributes.visible);
        assert!(!d.entities[&id].attributes.selectable);
        assert!(!d.permissions(id, true).unwrap().visible);
        assert!(!d.permissions(id, true).unwrap().reference_snappable);
    }
    c.undo().unwrap();
    assert!(c.world().documents[&did].entities[&id].attributes.hidden);
    c.redo().unwrap();
    assert!(!c.world().documents[&did].entities[&id].attributes.hidden);
}
#[test]
fn hidden_masks_all_interaction_without_mutating_explicit_selectability() {
    let mut d = Document::new("hidden permission");
    let mut e = Entity::new("visible root", Geometry::Point([0.; 3]), None);
    let id = e.id;
    e.attributes.hidden = true;
    d.entities.insert(id, e);
    let p = d.permissions(id, true).unwrap();
    assert!(!p.visible && !p.selectable && !p.editable && !p.reference_snappable);
    d.entities.get_mut(&id).unwrap().attributes.hidden = false;
    assert!(d.permissions(id, false).unwrap().selectable);
}

#[test]
fn preset_updates_keep_slot_identity_and_reject_basic_customization_and_reserved_aliases() {
    let limits = Limits::default();
    let mut d = Document::new("preset update contract");
    let basic = Layer::preset(PresetSlot::new(1).unwrap());
    let bid = basic.id;
    let advanced = Layer::preset(PresetSlot::new(17).unwrap());
    let aid = advanced.id;
    d.layer_order.extend([bid, aid]);
    d.layers.insert(bid, basic);
    d.layers.insert(aid, advanced);
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let before = c.snapshot();
    for (id, label, color, slot) in [
        (bid, "Rename", None, None),
        (bid, "Metal 01", Some([1, 2, 3, 255]), None),
        (aid, "Gem 01", None, None),
        (aid, "Custom", None, Some(PresetSlot::new(18).unwrap())),
    ] {
        let mut layer = c.world().documents[&did].layers[&id].clone();
        layer.name = label.into();
        if let Some(color) = color {
            layer.color = color;
        }
        if let Some(slot) = slot {
            layer.preset_slot = Some(slot);
        }
        let plan = LayerPlan::capture(&c, did, LayerAction::Update { layer }, None).unwrap();
        assert!(
            c.prepare(TransactionRequest::new(
                "invalid preset edit",
                vec![Mutation::Layers(Box::new(plan))]
            ))
            .is_err()
        );
        assert_eq!(c.snapshot(), before);
    }
}

#[test]
fn two_proposed_advanced_renames_cannot_collide_in_one_batch() {
    let limits = Limits::default();
    let mut d = Document::new("batch rename");
    let a = Layer::preset(PresetSlot::new(17).unwrap());
    let b = Layer::preset(PresetSlot::new(18).unwrap());
    let ids = [a.id, b.id];
    d.layer_order.extend(ids);
    d.layers.insert(a.id, a);
    d.layers.insert(b.id, b);
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let before = c.snapshot();
    let mut mutations = vec![];
    for (id, label) in ids.into_iter().zip(["Đá riêng", "đÁ RIÊNG"]) {
        let mut layer = c.world().documents[&did].layers[&id].clone();
        layer.name = label.into();
        let plan = LayerPlan::capture(&c, did, LayerAction::Update { layer }, None).unwrap();
        mutations.push(Mutation::Layers(Box::new(plan)));
    }
    assert!(
        c.prepare(TransactionRequest::new("colliding labels", mutations))
            .is_err()
    );
    assert_eq!(c.snapshot(), before);
}

#[test]
fn basic_preset_palette_cannot_be_customized_on_creation_or_restore() {
    let limits = Limits::default();
    let d = Document::new("fixed basic palette");
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let before = c.snapshot();
    let mut layer = Layer::preset(PresetSlot::new(1).unwrap());
    layer.color = [1, 2, 3, 255];
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer: layer.clone(),
            make_current: false,
        },
        None,
    )
    .unwrap();
    assert!(
        c.prepare(TransactionRequest::new(
            "invalid basic palette",
            vec![Mutation::Layers(Box::new(plan))]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
    let mut restored = Document::new("invalid restored palette");
    restored.layer_order.push(layer.id);
    restored.layers.insert(layer.id, layer);
    assert!(restored.validate(&limits).is_err());
}

#[test]
fn restored_custom_labels_cannot_collide_but_generic_import_names_can() {
    let limits = Limits::default();
    let mut d = Document::new("label policy");
    for _ in 0..2 {
        let layer = Layer::new("Imported duplicate", [123; 4]);
        d.layer_order.push(layer.id);
        d.layers.insert(layer.id, layer);
    }
    d.validate(&limits).unwrap();
    let mut advanced = Layer::preset(PresetSlot::new(17).unwrap());
    advanced.name = "IMPORTED DUPLICATE".into();
    d.layer_order.push(advanced.id);
    d.layers.insert(advanced.id, advanced);
    assert!(d.validate(&limits).is_err());
}
