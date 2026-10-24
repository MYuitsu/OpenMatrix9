#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn continue_curve_starts_at_the_last_control_point() {
    let mut doc = Document::new("continue curve");
    let curve = om9_core::curve_build::interpolate(
        &[
            [0.0, 0.0, 0.0],
            [5.0, 5.0, 0.0],
            [10.0, 0.0, 0.0],
            [15.0, 5.0, 0.0],
        ],
        3,
        om9_core::curve_build::KnotMode::ChordLength,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("source", Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("ContinueCurve").unwrap();
    c.accept_world_point([20.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([25.0, 5.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ContinueCurve")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.first().unwrap()[0], 15.0);
    assert_eq!(curve.poles.last().unwrap()[0], 25.0);
}


#[test]
fn continue_curve_accepts_degree_option() {
    let mut doc = Document::new("continue degree");
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
    c.submit("ContinueCurve").unwrap();
    c.submit("Degree=2").unwrap();
    for point in [[15.0, 5.0, 0.0], [20.0, 0.0, 0.0], [25.0, 5.0, 0.0]] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    let entity = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ContinueCurve")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &entity.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.degree, 2);
}
