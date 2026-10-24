#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_corner() -> Controller {
    let mut doc = Document::new("corner");
    let first = Entity::new(
        "x",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let second = Entity::new(
        "y",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [0.0, 10.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let ids = [first.id, second.id];
    doc.entities.insert(first.id, first);
    doc.entities.insert(second.id, second);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c
}

#[test]
fn fillet_creates_a_tangent_arc() {
    let mut c = controller_with_corner();
    c.submit("CurveFillet").unwrap();
    c.submit("Radius=2").unwrap();
    c.submit("").unwrap();
    let fillet = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Fillet")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { radius, .. }) = &fillet.geometry else {
        panic!("arc");
    };
    assert!((radius - 2.0).abs() < 1e-6);
}

#[test]
fn chamfer_creates_a_corner_line() {
    let mut c = controller_with_corner();
    c.submit("CurveChamfer").unwrap();
    c.submit("Distance=2").unwrap();
    c.submit("").unwrap();
    let chamfer = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Chamfer")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) = &chamfer.geometry
    else {
        panic!("line");
    };
    assert!((start[0] - 2.0).abs() < 1e-6 || (end[0] - 2.0).abs() < 1e-6);
}


#[test]
fn corner_ops_expose_the_matching_option() {
    let mut c = controller_with_corner();
    c.submit("CurveFillet").unwrap();
    assert!(c
        .corner_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Radius="));
    let mut c = controller_with_corner();
    c.submit("CurveChamfer").unwrap();
    assert!(c
        .corner_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Distance="));
}
