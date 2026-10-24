#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::{Geometry};
use om9_core::model::{Document, Entity};

fn controller_with_curve() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("rebuild");
    let curve = om9_core::curve_build::circle_on_frame(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
    )
    .unwrap();
    let entity = Entity::new("source", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    (Controller::new(doc, Limits::default()).unwrap(), id)
}

#[test]
fn rebuild_curve_uses_the_requested_point_count() {
    let (mut c, id) = controller_with_curve();
    c.select_objects(&[id]).unwrap();
    c.submit("Rebuild").unwrap();
    c.submit("Points=8").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let rebuilt = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Rebuild")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &rebuilt.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 8);
    assert_eq!(curve.degree, 3);
}

#[test]
fn rebuild_curve_accepts_a_degree_option() {
    let (mut c, id) = controller_with_curve();
    c.select_objects(&[id]).unwrap();
    c.submit("Rebuild").unwrap();
    c.submit("Degree=5").unwrap();
    c.submit("Points=10").unwrap();
    c.submit("").unwrap();
    let rebuilt = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Rebuild")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &rebuilt.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.degree, 5);
}


#[test]
fn rebuild_exposes_degree_and_points_tokens() {
    let (mut c, id) = controller_with_curve();
    c.select_objects(&[id]).unwrap();
    c.submit("Rebuild").unwrap();
    let tokens = c.rebuild_option_tokens();
    for expected in ["Degree=", "Points="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}
