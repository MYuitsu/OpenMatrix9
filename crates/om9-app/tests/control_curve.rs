#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

#[test]
fn control_curve_uses_the_clicked_poles() {
    let mut c = Controller::new(Document::new("control curve"), Limits::default()).unwrap();
    c.submit("Curve").unwrap();
    for point in [
        [0.0, 0.0, 0.0],
        [5.0, 10.0, 0.0],
        [10.0, -10.0, 0.0],
        [15.0, 0.0, 0.0],
    ] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    assert_eq!(curve.degree, 3);
    assert_eq!(curve.poles.len(), 4);
    assert_eq!(curve.poles[1], [5.0, 10.0, 0.0]);
}


#[test]
fn control_curve_accepts_degree_option() {
    let mut c = Controller::new(Document::new("control degree"), Limits::default()).unwrap();
    c.submit("Curve").unwrap();
    c.submit("Degree=2").unwrap();
    for point in [
        [0.0, 0.0, 0.0],
        [5.0, 5.0, 0.0],
        [10.0, 0.0, 0.0],
        [15.0, 5.0, 0.0],
    ] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    assert_eq!(curve.degree, 2);
}
