// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Definition, Document, Entity, Layer},
};
use om9_scene::scene::Scene;
use std::sync::Arc;
#[test]
fn metadata_refresh_reuses_buffers_and_updates_instance_masks_color_and_stamps() {
    let limits = Limits::default();
    let mut d = Document::new("metadata buffers");
    let layer = Layer::new("generic", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layers.insert(lid, layer);
    d.layer_order.push(lid);
    let mut member = Entity::new("member", Geometry::Point([1., 2., 3.]), Some(lid));
    member.attributes.color = None;
    member.placement = Transform::translation([2., 0., 0.]);
    let definition = Definition::new("definition", vec![member]);
    let mut root = Entity::new("root", Geometry::Instance(definition.id), None);
    root.placement = Transform::translation([3., 0., 0.]);
    d.definitions.insert(definition.id, definition);
    let eid = root.id;
    d.entities.insert(eid, root);
    let old = Scene::capture(Arc::new(d.clone()), 1, &limits).unwrap();
    d.layers.get_mut(&lid).unwrap().locked = true;
    d.layers.get_mut(&lid).unwrap().color = [8, 9, 10, 255];
    d.entities.get_mut(&eid).unwrap().attributes_revision += 1;
    let new = old.update_owned(Arc::new(d), 2, None, &limits).unwrap();
    assert!(Arc::ptr_eq(
        &old.objects[0].buffers,
        &new.objects[0].buffers
    ));
    assert_eq!(new.objects[0].transform, old.objects[0].transform);
    assert_eq!(new.objects[0].color, [8, 9, 10, 255]);
    assert!(new.objects[0].visible);
    assert!(!new.objects[0].selectable);
    assert_eq!(new.objects[0].source.attributes_revision, 2);
}
#[test]
fn cache_refresh_refuses_changed_geometry_owner_and_unreserved_metadata_clone() {
    let limits = Limits::default();
    let mut d = Document::new("cache proof");
    let root = Entity::new("root", Geometry::Point([0.; 3]), None);
    let id = root.id;
    d.entities.insert(id, root);
    let scene = Scene::capture(Arc::new(d.clone()), 1, &limits).unwrap();
    let tiny = Limits {
        max_bytes: scene.update_usage(None).unwrap().bytes - 1,
        ..limits
    };
    assert!(
        scene
            .update_owned(Arc::new(d.clone()), 2, None, &tiny)
            .is_err()
    );
    d.entities.get_mut(&id).unwrap().geometry = Geometry::Point([1.; 3]);
    assert!(scene.update_owned(Arc::new(d), 2, None, &limits).is_err());
    let mut foreign = scene.document.as_ref().clone();
    foreign.renew_session(&limits).unwrap();
    assert!(
        scene
            .update_owned(Arc::new(foreign), 2, None, &limits)
            .is_err()
    );
}
