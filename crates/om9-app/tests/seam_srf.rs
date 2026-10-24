#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn seamsrf_moves_a_closed_surface_seam() {
    let mut doc = Document::new("seam srf");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("SeamSrf").unwrap();
    c.submit("U,0.01").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count()
        >= 2);
}
