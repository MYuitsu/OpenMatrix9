#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn extrude_builds_a_surface_from_a_curve() {
    let mut doc = Document::new("extrude");
    let line = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let id = line.id;
    doc.entities.insert(id, line);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("ExtrudeSurface").unwrap();
    assert!(c
        .extrude_surface_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Height="));
    c.submit("Height=10").unwrap();
    c.submit("").unwrap();
    let surface = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Extrude")
        .unwrap();
    let Geometry::Surface(surface) = &surface.geometry else {
        panic!("surface");
    };
    let z_min = surface.poles.iter().map(|point| point[2]).fold(f64::MAX, f64::min);
    let z_max = surface.poles.iter().map(|point| point[2]).fold(f64::MIN, f64::max);
    assert!((z_min - 0.0).abs() < 1e-6);
    assert!((z_max - 10.0).abs() < 1e-6);
}
