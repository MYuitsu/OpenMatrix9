#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_curves(curves: Vec<om9_core::geometry::Curve>) -> Controller {
    let mut doc = Document::new("surface alias");
    let mut ids = Vec::new();
    for curve in curves {
        let entity = Entity::new("curve", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c
}

fn line(start: [f64; 3], end: [f64; 3]) -> om9_core::geometry::Curve {
    om9_core::geometry::Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    }
}

#[test]
fn blendsrf_builds_a_surface_between_two_curves() {
    let mut c = controller_with_curves(vec![
        line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        line([0.0, 10.0, 5.0], [10.0, 10.0, 5.0]),
    ]);
    c.submit("BlendSrf").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .any(|entity| matches!(entity.geometry, Geometry::Surface(_))));
}

#[test]
fn sweep1_builds_a_surface_along_a_rail() {
    let mut c = controller_with_curves(vec![
        line([0.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
        line([0.0, 0.0, 0.0], [0.0, 0.0, 10.0]),
    ]);
    c.submit("Sweep1").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .any(|entity| matches!(entity.geometry, Geometry::Surface(_))));
}

