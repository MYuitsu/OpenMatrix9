#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_line() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("crv ends");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    (Controller::new(doc, Limits::default()).unwrap(), id)
}

#[test]
fn crv_start_and_end_create_points() {
    let (mut c, id) = controller_with_line();
    c.select_objects(&[id]).unwrap();
    c.submit("CrvStart").unwrap();
    c.submit("CrvEnd").unwrap();
    let points: Vec<_> = c
        .document()
        .entities
        .values()
        .filter_map(|entity| match entity.geometry {
            Geometry::Point(point) => Some((entity.label.as_str(), point)),
            _ => None,
        })
        .collect();
    assert_eq!(points.len(), 2);
    let start = points.iter().find(|(label, _)| *label == "CrvStart").unwrap().1;
    let end = points.iter().find(|(label, _)| *label == "CrvEnd").unwrap().1;
    assert!(start[0].abs() < 1e-9);
    assert!((end[0] - 10.0).abs() < 1e-9);
}
