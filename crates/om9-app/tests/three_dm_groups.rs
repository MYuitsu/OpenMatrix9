// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows, feature = "occt-brep"))]

use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity, Group, Layer},
};

#[test]
#[ignore = "requires OM9_EXCHANGE_NATIVE and the OCCT runtime on PATH"]
fn rhino5_group_membership_round_trips() {
    let mut document = Document::new("group round trip");
    let layer = Layer::new("Layer 01", [10, 20, 30, 255]);
    document.current_layer = Some(layer.id);
    document.layer_order.push(layer.id);
    document.layers.insert(layer.id, layer.clone());
    let entity = Entity::new(
        "grouped line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        layer.id,
    );
    document.entities.insert(entity.id, entity.clone());
    let group = Group::new("Group A", vec![entity.id]);
    document.groups.insert(group.id, group.clone());
    let controller = Controller::new(document, Limits::default()).unwrap();

    let root = std::env::temp_dir().join(format!("om9-group-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("groups.3dm");
    controller.export_3dm(&path).unwrap();

    let mut reopened = Controller::new(Document::new("reopen"), Limits::default()).unwrap();
    reopened.import_3dm(&path).unwrap();
    let groups = &reopened.document().groups;
    assert_eq!(groups.len(), 1);
    let reopened_group = groups.values().next().unwrap();
    assert_eq!(reopened_group.name, group.name);
    assert_eq!(reopened_group.members.len(), 1);
    assert_eq!(
        reopened.document().entities[&reopened_group.members[0]].label,
        "grouped line"
    );
    let _ = std::fs::remove_dir_all(root);
}
