// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::{
    layer_command::{LayerChoice, LayerProperty, parse},
    session::{Controller, InvocationOrigin, LINE},
};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    layers::PresetSlot,
    model::{Document, Entity},
};
fn slot(n: u8) -> PresetSlot {
    PresetSlot::new(n).unwrap()
}
#[test]
fn layer_cmd_creates_only_its_group_and_does_not_assign_existing_objects() {
    let mut d = Document::new("lazy CMD");
    let e = Entity::new("root", Geometry::Point([0.; 3]), None);
    let eid = e.id;
    d.entities.insert(eid, e);
    let mut c = Controller::new(d, Limits::default()).unwrap();
    c.submit("Layer=1").unwrap();
    assert_eq!(c.document().layers.len(), 4);
    assert_eq!(c.document().entities[&eid].attributes.layer, None);
    let current = c.document().current_layer.unwrap();
    assert_eq!(c.document().layers[&current].preset_slot, Some(slot(1)));
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert!(c.document().layers.is_empty());
    c.redo().unwrap();
    assert_eq!(c.document().current_layer, Some(current));
    c.submit("Layer=Lights").unwrap();
    assert_eq!(c.document().layers.len(), 5);
    c.submit("Layer=None").unwrap();
    assert_eq!(c.document().current_layer, None);
}
#[test]
fn pending_line_preserves_start_both_sides_and_binds_current_at_acceptance() {
    let mut c = Controller::new(Document::new("layer during Line"), Limits::default()).unwrap();
    c.dispatch(LINE, InvocationOrigin::Menu).unwrap();
    c.submit("BothSides=Yes").unwrap();
    c.submit("1,2,3").unwrap();
    c.submit("Layer=Creation").unwrap();
    let lid = c.document().current_layer.unwrap();
    assert!(c.is_running());
    assert_eq!(
        c.preview([2., 2., 3.]).unwrap(),
        Some(([0., 2., 3.], [2., 2., 3.]))
    );
    c.submit("2,2,3").unwrap();
    let e = c.document().entities.values().next().unwrap();
    assert_eq!(e.attributes.layer, Some(lid));
    assert!(
        matches!(&e.geometry,Geometry::Curve(Curve::Line{start,end,..}) if *start==[0.,2.,3.] && *end==[2.,2.,3.])
    );
    assert_eq!(c.recent().len(), 1);
    assert_eq!(e.attributes.color, None);
    assert_eq!(
        c.scene().unwrap().objects[0].color,
        c.document().layers[&lid].color
    );
}

#[test]
fn quoted_numeric_none_and_zero_labels_resolve_owned_identity() {
    for name in ["18", "None", "0"] {
        let mut d = Document::new("quoted custom labels");
        let mut layer = om9_core::model::Layer::preset(slot(17));
        layer.name = name.into();
        let lid = layer.id;
        d.layer_order.push(lid);
        d.layers.insert(lid, layer);
        let mut c = Controller::new(d, Limits::default()).unwrap();
        c.submit(&format!("Layer=\"{name}\"")).unwrap();
        assert_eq!(c.document().current_layer, Some(lid));
    }
}

#[test]
fn nested_scene_metadata_and_new_point_refuse_aggregate_budget_without_mutation() {
    use om9_core::model::Definition;
    let mut d = Document::new("scene admission");
    let leaf = Definition::new(
        "leaf",
        (0..8)
            .map(|i| Entity::new("point", Geometry::Point([i as f64, 0., 0.]), None))
            .collect(),
    );
    let mut id = leaf.id;
    d.definitions.insert(id, leaf);
    for _ in 0..3 {
        let definition = Definition::new(
            "nested",
            (0..8)
                .map(|_| Entity::new("child", Geometry::Instance(id), None))
                .collect(),
        );
        id = definition.id;
        d.definitions.insert(id, definition);
    }
    let root = Entity::new("root", Geometry::Instance(id), None);
    d.entities.insert(root.id, root);
    let too_small = Limits {
        max_bytes: 1_900_000,
        ..Limits::default()
    };
    assert!(matches!(
        Controller::new(d.clone(), too_small),
        Err(om9_core::Error::Budget(_))
    ));
    let limits = Limits {
        max_bytes: 3_000_000,
        ..Limits::default()
    };
    let mut c = Controller::new(d, limits).unwrap();
    assert_eq!(c.scene().unwrap().objects.len(), 4096);
    let before = c.coordinator().snapshot();
    assert!(matches!(
        c.submit("Layer=1"),
        Err(om9_core::Error::Budget(_))
    ));
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.scene().unwrap().objects.len(), 4096);
    c.submit("Point").unwrap();
    assert!(matches!(c.submit("1,2,3"), Err(om9_core::Error::Budget(_))));
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(c.is_running());
}

#[test]
fn selection_filters_hidden_definition_members_and_counts_before_source_allocation() {
    use om9_core::model::{Definition, Layer};
    let mut d = Document::new("selection scene leaves");
    let layer = Layer::preset(slot(5));
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut member = Entity::new("hidden member", Geometry::Point([0.; 3]), None);
    member.attributes.hidden = true;
    let definition = Definition::new("definition", vec![member]);
    let root = Entity::new("instance", Geometry::Instance(definition.id), Some(lid));
    d.definitions.insert(definition.id, definition);
    d.entities.insert(root.id, root);
    let cloud = Entity::new(
        "all point rows hidden",
        Geometry::Cloud(om9_core::geometry::Cloud {
            points: vec![[0.; 3]],
            normals: vec![],
            colors: vec![],
            hidden: vec![true],
        }),
        Some(lid),
    );
    d.entities.insert(cloud.id, cloud);
    let a = Entity::new("a", Geometry::Point([1.; 3]), Some(lid));
    let b = Entity::new("b", Geometry::Point([2.; 3]), Some(lid));
    let ids = [a.id, b.id];
    d.entities.insert(a.id, a);
    d.entities.insert(b.id, b);
    let mut c = Controller::new(
        d,
        Limits {
            max_outputs: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(c.selectable_layer_member_count(slot(5)).unwrap(), 2);
    let before = c.coordinator().snapshot();
    assert!(c.select_layer(slot(5)).is_err());
    assert!(c.selected().is_empty());
    assert!(c.collect_object_sources(ids.into_iter()).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
}
#[test]
fn off_current_accepts_invisible_output_locked_current_refuses_without_losing_draft() {
    let mut c = Controller::new(Document::new("output permissions"), Limits::default()).unwrap();
    c.submit("Layer=17").unwrap();
    c.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    c.submit("0,0").unwrap();
    c.set_layer_property(slot(17), LayerProperty::Visible(false))
        .unwrap();
    c.set_layer_property(slot(17), LayerProperty::Locked(true))
        .unwrap();
    assert!(c.submit("1,0").is_err());
    assert!(c.document().entities.is_empty());
    assert!(c.is_running());
    c.set_layer_property(slot(17), LayerProperty::Locked(false))
        .unwrap();
    c.submit("1,0").unwrap();
    assert!(!c.scene().unwrap().objects[0].visible);
}
#[test]
fn lazy_assign_keeps_current_and_is_one_undo_without_partial_creation_on_failure() {
    let mut d = Document::new("atomic arrow");
    let e = Entity::new("root", Geometry::Point([1., 2., 3.]), None);
    let id = e.id;
    d.entities.insert(id, e);
    let mut c = Controller::new(d, Limits::default()).unwrap();
    let source = c.document().stamp(id).unwrap();
    c.assign_layer(slot(5), vec![source]).unwrap();
    assert_eq!(c.document().layers.len(), 4);
    assert_eq!(c.document().current_layer, None);
    assert_eq!(c.coordinator().undo_len(), 1);
    assert_eq!(
        c.document().entities[&id].attributes.layer,
        c.layer_id(slot(5))
    );
    let stale = c.document().stamp(id).unwrap();
    c.undo().unwrap();
    assert!(c.document().layers.is_empty());
    let before = c.coordinator().snapshot();
    assert!(c.assign_layer(slot(9), vec![stale]).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    c.redo().unwrap();
}
#[test]
fn group_mixed_toggle_keeps_hidden_and_explicit_selectable_preferences() {
    let mut d = Document::new("group mask");
    let mut e = Entity::new("explicit", Geometry::Point([0.; 3]), None);
    e.attributes.hidden = true;
    e.attributes.selectable = false;
    let id = e.id;
    d.entities.insert(id, e);
    let mut c = Controller::new(d, Limits::default()).unwrap();
    c.toggle_layer_group(slot(1)).unwrap();
    assert!(c.document().layers.values().all(|l| !l.visible));
    c.set_layer_property(slot(2), LayerProperty::Visible(true))
        .unwrap();
    c.toggle_layer_group(slot(1)).unwrap();
    assert!(c.document().layers.values().all(|l| l.visible));
    assert!(c.document().entities[&id].attributes.hidden);
    assert!(!c.document().entities[&id].attributes.selectable);
}
#[test]
fn parser_uses_slot_identity_custom_label_none_and_closed_bounded_input() {
    let mut d = Document::new("CMD choice");
    let mut layer = om9_core::model::Layer::preset(slot(21));
    layer.name = "Đá riêng".into();
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    assert_eq!(
        parse("Layer=User 25", &d).unwrap(),
        Some(LayerChoice::Slot(slot(21)))
    );
    assert_eq!(
        parse("Layer=\"đÁ RIÊNG\"", &d).unwrap(),
        Some(LayerChoice::Existing(lid))
    );
    assert_eq!(parse("Layer=0", &d).unwrap(), Some(LayerChoice::None));
    assert_eq!(
        parse("Layer=Lights", &d).unwrap(),
        Some(LayerChoice::Slot(slot(33)))
    );
    assert_eq!(parse("Layers", &d).unwrap(), None);
    for text in [
        "Layer=34",
        "Layer=-1",
        "Layer=Unknown",
        "Layer=\"Lights",
        "Layer=",
        "Layer=Metal\u{0000}01",
    ] {
        assert!(parse(text, &d).is_err(), "{text:?}");
    }
    assert!(parse(&format!("Layer={}", "a".repeat(300)), &d).is_err());
}
