#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn revolve_builds_a_surface_around_the_cplane_normal() {
    let mut doc = Document::new("revolve");
    let profile = om9_core::geometry::Curve::Line {
        start: [10.0, 0.0, 0.0],
        end: [10.0, 0.0, 10.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("profile", Geometry::Curve(profile), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Revolve").unwrap();
    let surface = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Revolve")
        .unwrap();
    let Geometry::Surface(surface) = &surface.geometry else {
        panic!("surface");
    };
    for point in &surface.poles {
        let radius = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radius - 10.0).abs() < 0.2, "{radius}");
    }
}
