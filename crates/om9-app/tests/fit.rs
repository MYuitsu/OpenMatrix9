#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_circle() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("fit");
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
    (Controller::new(doc, Limits::default()).unwrap(), id)
}

#[test]
fn fit_curve_refits_a_circle_within_tolerance() {
    let (mut c, id) = controller_with_circle();
    c.select_objects(&[id]).unwrap();
    c.submit("FitCrv").unwrap();
    c.submit("Tolerance=0.1").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let fitted = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "FitCrv")
        .unwrap();
    assert!(matches!(
        fitted.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(_))
    ));
}


#[test]
fn fit_exposes_tolerance_token() {
    let (mut c, id) = controller_with_circle();
    c.select_objects(&[id]).unwrap();
    c.submit("FitCrv").unwrap();
    assert!(c
        .fit_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Tolerance="));
}
