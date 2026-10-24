#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn offset_curve_on_surface_shifts_across_the_surface() {
    let mut doc = Document::new("offset on surface");
    let surface = om9_core::surface_kernel::plane(
        [-20.0, -20.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        40.0,
        40.0,
        &Limits::default(),
    )
    .unwrap();
    let surface = Entity::new("plane", Geometry::Surface(surface), None);
    let curve = Entity::new(
        "line",
        Geometry::Curve(om9_core::curve_build::polyline(&[
            [-10.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
        ]).unwrap()),
        None,
    );
    let ids = [surface.id, curve.id];
    doc.entities.insert(surface.id, surface);
    doc.entities.insert(curve.id, curve);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("OffsetCrvOnSrf").unwrap();
    assert!(c
        .offset_on_surface_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Distance="));
    c.submit("Distance=2").unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "OffsetCrvOnSrf")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    let average_y = curve.poles.iter().map(|point| point[1]).sum::<f64>() / curve.poles.len() as f64;
    assert!((average_y.abs() - 2.0).abs() < 0.2, "{average_y}");
}
