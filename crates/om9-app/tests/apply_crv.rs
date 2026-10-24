#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn apply_crv_maps_a_curve_onto_the_surface() {
    let mut doc = Document::new("apply crv");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        40.0,
        40.0,
        &Limits::default(),
    )
    .unwrap();
    let surface = Entity::new("plane", Geometry::Surface(surface), None);
    let curve = om9_core::curve_build::polyline(&[
        [0.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [20.0, 0.0, 0.0],
    ])
    .unwrap();
    let curve = Entity::new("curve", Geometry::Curve(curve), None);
    let ids = [surface.id, curve.id];
    doc.entities.insert(surface.id, surface);
    doc.entities.insert(curve.id, curve);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("ApplyCrv").unwrap();
    let applied = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ApplyCrv")
        .unwrap();
    assert!(matches!(
        applied.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(_))
    ));
}
