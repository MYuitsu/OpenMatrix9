#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_views() -> Controller {
    let mut doc = Document::new("crv2view");
    let top = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let front = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 10.0]]).unwrap();
    for (name, curve) in [("top", top), ("front", front)] {
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    Controller::new(doc, Limits::default()).unwrap()
}

#[test]
fn crv2view_combines_two_projection_curves() {
    let mut c = controller_with_views();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Crv2View").unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Crv2View")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    let max_z = curve.poles.iter().map(|point| point[2]).fold(f64::MIN, f64::max);
    assert!((max_z - 10.0).abs() < 0.5, "{max_z}");
}
