#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_circle() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("offset");
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
fn offset_curve_moves_a_circle_outward() {
    let (mut c, id) = controller_with_circle();
    c.select_objects(&[id]).unwrap();
    c.submit("Offset").unwrap();
    c.submit("Distance=2").unwrap();
    c.submit("").unwrap();
    let offset = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Offset")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &offset.geometry else {
        panic!("nurbs");
    };
    let radius = curve
        .poles
        .iter()
        .map(|point| (point[0].powi(2) + point[1].powi(2)).sqrt())
        .sum::<f64>()
        / curve.poles.len() as f64;
    assert!((radius - 12.0).abs() < 0.2, "{radius}");
}


#[test]
fn offset_curve_follows_the_active_cplane() {
    let mut doc = Document::new("offset cplane");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Offset").unwrap();
    c.submit("Distance=2").unwrap();
    c.submit("").unwrap();
    let offset = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Offset")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &offset.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[1].abs() < 1e-6));
    assert!(curve.poles.iter().all(|point| (point[2].abs() - 2.0).abs() < 0.2));
}


#[test]
fn offset_exposes_distance_token() {
    let mut doc = Document::new("offset token");
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
    c.submit("Offset").unwrap();
    assert!(c
        .offset_curve_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Distance="));
}
