#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn planarsrf_builds_a_surface_from_a_closed_curve() {
    let mut doc = Document::new("planar srf");
    let curve = om9_core::curve_build::rectangle(
        [-10.0, -10.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        20.0,
        20.0,
    )
    .unwrap();
    let entity = Entity::new("rect", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("PlanarSrf").unwrap();
    assert!(c.document().entities.values().any(|entity| {
        matches!(entity.geometry, Geometry::Surface(_))
    }));
}
