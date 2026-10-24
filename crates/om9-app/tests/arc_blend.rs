#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn arc_blend_connects_two_curve_ends() {
    let mut doc = Document::new("arc blend");
    let first = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let second = om9_core::curve_build::polyline(&[[10.0, 10.0, 0.0], [20.0, 10.0, 0.0]]).unwrap();
    for (name, curve) in [("a", first), ("b", second)] {
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("ArcBlend").unwrap();
    c.submit("").unwrap();
    let blend = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ArcBlend")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { radius, .. }) = &blend.geometry else {
        panic!("circle");
    };
    assert!((radius - 5.0).abs() < 0.1, "{radius}");
}


#[test]
fn arc_blend_follows_the_active_cplane() {
    let mut doc = Document::new("arc blend cplane");
    let first = Entity::new(
        "first",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let second = Entity::new(
        "second",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [20.0, 0.0, 10.0],
            end: [30.0, 0.0, 10.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let ids = [first.id, second.id];
    doc.entities.insert(first.id, first);
    doc.entities.insert(second.id, second);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("ArcBlend").unwrap();
    c.submit("").unwrap();
    let blend = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ArcBlend")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { x, y, .. }) = &blend.geometry else {
        panic!("arc");
    };
    let normal = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    assert!(normal[1].abs() > 0.99, "{normal:?}");
}
