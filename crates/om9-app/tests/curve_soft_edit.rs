#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn curve_soft_edit_moves_a_single_edit_point() {
    let mut doc = Document::new("curve soft edit");
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
    let entity = Entity::new("curve", Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("CurveSoftEdit").unwrap();
    assert!(c
        .curve_soft_edit_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Point="));
    c.submit("Index=1").unwrap();
    c.submit("Point=5,10,0").unwrap();
    c.submit("").unwrap();
    let edited = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "CurveSoftEdit")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &edited.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().any(|point| point[1] > 6.0), "{:?}", curve.poles);
}
