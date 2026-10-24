#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn create_uv_curves_writes_the_uv_rectangle() {
    let mut doc = Document::new("uv curves");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("CreateUVCrv").unwrap();
    let uv = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "CreateUVCrv")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) = &uv.geometry
    else {
        panic!("polycurve");
    };
    assert_eq!(segments.len(), 4);
}
