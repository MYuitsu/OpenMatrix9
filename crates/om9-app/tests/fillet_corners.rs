#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::{Curve, Geometry};
use om9_core::model::{Document, Entity};

#[test]
fn fillet_corners_creates_an_arc_at_each_corner() {
    let points = [
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, 0.0],
    ];
    let mut segments = Vec::new();
    for pair in points.windows(2) {
        segments.push(Curve::Line {
            start: pair[0],
            end: pair[1],
            domain: [0.0, 1.0],
        });
    }
    let polyline = Curve::Polycurve {
        segments,
        reversed: false,
    };
    let mut doc = Document::new("fillet corners");
    let entity = Entity::new("polyline", Geometry::Curve(polyline), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("FilletCorners").unwrap();
    c.submit("Radius=2").unwrap();
    c.submit("").unwrap();
    let arcs = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "FilletCorners")
        .count();
    assert_eq!(arcs, 4);
}


#[test]
fn fillet_corners_exposes_radius_token() {
    let mut doc = Document::new("fillet token");
    let curve = om9_core::curve_build::rectangle(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
    )
    .unwrap();
    let entity = Entity::new("rectangle", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("FilletCorners").unwrap();
    assert!(c
        .fillet_corners_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Radius="));
}
