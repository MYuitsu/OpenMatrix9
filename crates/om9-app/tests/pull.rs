#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_curve_and_plane() -> Controller {
    let mut doc = Document::new("pull");
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
    let curve = om9_core::curve_build::polyline(&[
        [-5.0, 0.0, 5.0],
        [5.0, 0.0, 5.0],
    ])
    .unwrap();
    let curve = Entity::new("line", Geometry::Curve(curve), None);
    let ids = [surface.id, curve.id];
    doc.entities.insert(surface.id, surface);
    doc.entities.insert(curve.id, curve);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c
}

#[test]
fn pull_curve_lands_on_the_surface() {
    let mut c = controller_with_curve_and_plane();
    c.submit("Pull").unwrap();
    c.submit("").unwrap();
    let pulled = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Pull")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &pulled.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[2].abs() < 1e-6));
    let min_x = curve.poles.iter().map(|point| point[0]).fold(f64::MAX, f64::min);
    let max_x = curve.poles.iter().map(|point| point[0]).fold(f64::MIN, f64::max);
    assert!((min_x + 5.0).abs() < 0.1, "{min_x}");
    assert!((max_x - 5.0).abs() < 0.1, "{max_x}");
}


#[test]
fn pull_exposes_tolerance_token() {
    let mut c = controller_with_curve_and_plane();
    c.submit("Pull").unwrap();
    assert!(c
        .pull_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Tolerance="));
}
