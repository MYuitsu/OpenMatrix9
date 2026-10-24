#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn loft_command_builds_a_surface_between_two_lines() {
    let mut doc = Document::new("loft");
    let mut ids = Vec::new();
    for (start, end) in [
        ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        ([0.0, 10.0, 5.0], [10.0, 10.0, 5.0]),
    ] {
        let entity = Entity::new(
            "line",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        );
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Loft").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .any(|entity| matches!(entity.geometry, Geometry::Surface(_))));
}
