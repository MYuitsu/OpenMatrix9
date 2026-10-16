// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity, Layer},
    transaction::*,
};
#[test]
fn new_document_has_no_current_layer_or_eager_groups() {
    let d = Document::new("new");
    assert!(d.current_layer.is_none());
    assert!(d.layers.is_empty());
    assert!(d.layer_order.is_empty());
    d.validate(&Limits::default()).unwrap();
}
#[test]
fn current_none_preserves_default_curve_style_without_creating_a_group() {
    let d = Document::new("curve");
    let e = Entity::new(
        "Line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        d.current_layer,
    );
    assert!(e.attributes.layer.is_none());
    assert_eq!(e.attributes.color, Some([0, 130, 85, 255]));
}
#[test]
fn output_on_off_layer_is_admitted_invisible_and_locked_layer_rejects() {
    let mut d = Document::new("off output");
    let mut layer = Layer::new("Metal 01", [34, 153, 135, 255]);
    layer.visible = false;
    let lid = layer.id;
    d.layers.insert(lid, layer);
    d.layer_order.push(lid);
    d.current_layer = Some(lid);
    let did = d.id;
    let e = Entity::new("output", Geometry::Point([1., 2., 3.]), d.current_layer);
    let id = e.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "output",
            vec![Mutation::Insert {
                document: did,
                entity: e,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(
        !c.world().documents[&did]
            .permissions(id, true)
            .unwrap()
            .visible
    );
    assert_eq!(
        c.world().documents[&did].entities[&id].attributes.layer,
        Some(lid)
    );
    c.undo().unwrap();
    assert!(!c.world().documents[&did].entities.contains_key(&id));
    let mut d = c.world().documents[&did].as_ref().clone();
    d.layers.get_mut(&lid).unwrap().locked = true;
    let e = Entity::new("blocked", Geometry::Point([1., 2., 3.]), d.current_layer);
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "blocked",
            vec![Mutation::Insert {
                document: did,
                entity: e
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
}
