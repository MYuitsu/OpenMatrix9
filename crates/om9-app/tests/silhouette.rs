#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn silhouette_finds_the_view_contour_of_a_cylinder() {
    let mut doc = Document::new("silhouette");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
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
    c.submit("Silhouette").unwrap();
    let found: Vec<_> = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "Silhouette")
        .collect();
    assert!(!found.is_empty());
    for entity in found {
        let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
            &entity.geometry
        else {
            panic!("polycurve");
        };
        for segment in segments {
            let om9_core::geometry::Curve::Line { start, .. } = segment else {
                panic!("line");
            };
            assert!(start[2].abs() < 0.2, "{start:?}");
        }
    }
}
