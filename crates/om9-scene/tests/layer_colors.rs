// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    layers::{LayerAction, LayerPlan},
    model::{Definition, Document, Entity, Layer},
    transaction::*,
};
use om9_scene::scene::Scene;
#[test]
fn assigned_root_uses_layer_color_and_later_swatch_changes_without_geometry_edits() {
    let limits = Limits::default();
    let mut d = Document::new("by layer scene");
    let layer = Layer::new("Metal 01", [34, 153, 135, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut e = Entity::new("root", Geometry::Point([1., 2., 3.]), None);
    e.attributes.color = Some([200, 10, 10, 255]);
    let eid = e.id;
    let geometry = e.geometry.clone();
    d.entities.insert(eid, e);
    let did = d.id;
    let stamp = d.stamp(eid).unwrap();
    let mut c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    for action in [
        LayerAction::Assign {
            layer: lid,
            sources: vec![stamp],
        },
        LayerAction::Update {
            layer: {
                let mut l = c.world().documents[&did].layers[&lid].clone();
                l.color = [9, 8, 7, 255];
                l
            },
        },
    ] {
        // Capture each command at its actual current document revision.
        let plan = LayerPlan::capture(&c, did, action, None).unwrap();
        let p = c
            .prepare(TransactionRequest::new(
                "Layers",
                vec![Mutation::Layers(Box::new(plan))],
            ))
            .unwrap();
        c.commit(p).unwrap();
        let d = c.world().documents[&did].clone();
        let scene = Scene::capture(d.clone(), c.epoch(), &limits).unwrap();
        assert_eq!(scene.objects[0].color, d.layers[&lid].color);
        assert_eq!(d.entities[&eid].geometry, geometry);
        assert_eq!(d.current_layer, None);
    }
}

#[test]
fn hidden_root_and_definition_member_mask_scene_display_pick_edit_and_reference() {
    let limits = Limits::default();
    let mut d = Document::new("hidden scene ownership");
    let mut root = Entity::new("hidden root", Geometry::Point([1., 0., 0.]), None);
    root.attributes.hidden = true;
    let root_id = root.id;
    d.entities.insert(root.id, root);
    let mut member = Entity::new("hidden member", Geometry::Point([2., 0., 0.]), None);
    member.attributes.hidden = true;
    let definition = Definition::new("definition", vec![member]);
    let instance = Entity::new("instance", Geometry::Instance(definition.id), None);
    let instance_id = instance.id;
    d.definitions.insert(definition.id, definition);
    d.entities.insert(instance.id, instance);
    let scene = Scene::capture(d.clone().into(), 1, &limits).unwrap();
    assert_eq!(scene.objects.len(), 2);
    for object in &scene.objects {
        assert!(
            !object.visible
                && !object.selectable
                && !object.editable
                && !object.reference_snappable
                && !object.reference_unlocked
        );
    }
    d.entities.get_mut(&root_id).unwrap().attributes.hidden = false;
    let scene = Scene::capture(d.into(), 2, &limits).unwrap();
    assert!(
        scene
            .objects
            .iter()
            .find(|o| o.root == root_id)
            .unwrap()
            .visible
    );
    assert!(
        !scene
            .objects
            .iter()
            .find(|o| o.root == instance_id)
            .unwrap()
            .visible
    );
}
