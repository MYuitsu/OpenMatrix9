#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn plane_controller() -> Controller {
    let mut doc = Document::new("surface edit");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        20.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c
}

#[test]
fn extendsrf_extends_a_plane_edge() {
    let mut c = plane_controller();
    c.submit("ExtendSrf").unwrap();
    c.submit("U1,5").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count()
        >= 2);
}

#[test]
fn shrinksrf_shrinks_a_plane() {
    let mut c = plane_controller();
    c.submit("ShrinkSrf").unwrap();
    c.submit("0.1,0.9,0.1,0.9").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count()
        >= 2);
}
