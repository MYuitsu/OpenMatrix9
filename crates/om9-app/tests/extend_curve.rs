#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn extend_curve_adds_a_tangent_line() {
    let mut doc = Document::new("extend");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Extend").unwrap();
    c.submit("Length=5").unwrap();
    c.submit("").unwrap();
    let extension = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Extend")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) =
        &extension.geometry
    else {
        panic!("line");
    };
    assert!((start[0] - 10.0).abs() < 1e-9);
    assert!((end[0] - 15.0).abs() < 1e-9);
}


#[test]
fn extend_can_use_the_start_end() {
    let mut doc = Document::new("extend start");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Extend").unwrap();
    c.submit("Start").unwrap();
    c.submit("Length=5").unwrap();
    c.submit("").unwrap();
    let extended = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Extend")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) = &extended.geometry
    else {
        panic!("line");
    };
    assert!((start[0]).abs() < 1e-9);
    assert!((end[0] + 5.0).abs() < 1e-9);
}
