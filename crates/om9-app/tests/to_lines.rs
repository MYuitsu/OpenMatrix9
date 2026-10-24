#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn convert_curve_to_lines_uses_the_requested_segments() {
    let mut doc = Document::new("to lines");
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
    c.submit("Convert").unwrap();
    c.submit("Count=8").unwrap();
    c.submit("").unwrap();
    let converted = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Convert")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &converted.geometry
    else {
        panic!("polycurve");
    };
    assert_eq!(segments.len(), 8);
}


#[test]
fn to_lines_exposes_count_token() {
    let mut doc = Document::new("to lines token");
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
    c.submit("Convert").unwrap();
    assert!(c
        .to_lines_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Count="));
}
