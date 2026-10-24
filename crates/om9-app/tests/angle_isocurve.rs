#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn angle_isocurve_extracts_at_the_requested_angle() {
    let mut doc = Document::new("angle isocurve");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("AngleIsocurve").unwrap();
    assert!(c
        .angle_isocurve_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Direction="));
    c.submit("Angle=180").unwrap();
    c.submit("Direction=U").unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "AngleIsocurve")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.len() > 1);
}
