#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn section_intersects_a_cylinder_with_the_cplane() {
    let mut doc = Document::new("section");
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
    c.set_cplane_origin_rotation([0.0, 0.0, 5.0], 0.0)
        .unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Section").unwrap();
    let sections: Vec<_> = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "Section")
        .collect();
    assert!(!sections.is_empty());
    let section = sections[0];
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &section.geometry
    else {
        panic!("polycurve");
    };
    assert!(!segments.is_empty());
    for segment in segments {
        let om9_core::geometry::Curve::Line { start, .. } = segment else {
            panic!("line");
        };
        assert!((start[2] - 5.0).abs() < 0.1);
        assert!(((start[0].powi(2) + start[1].powi(2)).sqrt() - 10.0).abs() < 0.1);
    }
}
