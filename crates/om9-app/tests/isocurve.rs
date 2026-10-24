#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_plane() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("isocurve");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    (Controller::new(doc, Limits::default()).unwrap(), id)
}

#[test]
fn extract_isocurve_creates_a_curve_from_a_surface() {
    let (mut c, id) = controller_with_plane();
    c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap();
    c.submit("V=0.5").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ExtractIsocurve")
        .unwrap();
    assert!(matches!(
        curve.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(_))
    ));
}


#[test]
fn isocurve_exposes_u_v_tokens() {
    let mut doc = Document::new("iso token");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap();
    let tokens = c.isocurve_option_tokens();
    for expected in ["U=", "V="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}
