#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn crv_seam_moves_the_circle_start_angle() {
    let mut doc = Document::new("crv seam");
    let curve = om9_core::curve_build::circle_on_frame(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
    )
    .unwrap();
    let entity = Entity::new("circle", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("CrvSeam").unwrap();
    c.submit("Seam=0.5").unwrap();
    c.submit("").unwrap();
    let seam = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "CrvSeam")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { angle_domain, .. }) = &seam.geometry
    else {
        panic!("circle");
    };
    let angles = angle_domain.expect("angles");
    assert!((angles[0] - std::f64::consts::PI).abs() < 1e-9);
}


#[test]
fn crv_seam_exposes_seam_token() {
    let mut doc = Document::new("seam token");
    let curve = om9_core::curve_build::circle_on_frame(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
    )
    .unwrap();
    let entity = Entity::new("circle", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("CrvSeam").unwrap();
    assert!(c
        .crv_seam_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Seam="));
}
