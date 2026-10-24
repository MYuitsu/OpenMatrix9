#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn curve_through_points_interpolates_selected_points() {
    let mut doc = Document::new("through");
    let mut ids = Vec::new();
    for point in [
        [0.0, 0.0, 0.0],
        [5.0, 5.0, 0.0],
        [10.0, -5.0, 0.0],
        [15.0, 0.0, 0.0],
    ] {
        let entity = Entity::new("p", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("CurveThroughPt").unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "CurveThroughPt")
        .unwrap();
    assert!(matches!(
        curve.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(_))
    ));
}


#[test]
fn through_points_accepts_degree_option() {
    let mut doc = Document::new("through degree");
    let mut ids = Vec::new();
    for index in 0..6 {
        let point = [index as f64 * 5.0, (index % 2) as f64 * 5.0, 0.0];
        let entity = Entity::new("p", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("CurveThroughPt").unwrap();
    c.submit("Degree=5").unwrap();
    c.submit("").unwrap();
    let entity = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "CurveThroughPt")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &entity.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.degree, 5);
}
