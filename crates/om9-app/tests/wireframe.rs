#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};
use om9_kernel::creation;

#[test]
fn dup_edge_extracts_the_brep_display_edges() {
    let brep = creation::primitive(
        creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .unwrap();
    let mut doc = Document::new("wireframe");
    let entity = Entity::new("box", Geometry::Brep(brep), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("DupEdge").unwrap();
    let wire = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "DupEdge")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) = &wire.geometry
    else {
        panic!("polycurve");
    };
    assert!(!segments.is_empty());
}
