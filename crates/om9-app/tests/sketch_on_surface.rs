#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn sketch_on_surface_projects_points_into_a_polyline() {
    let mut doc = Document::new("sketch on surface");
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
    let mut ids = vec![surface.id];
    doc.entities.insert(surface.id, surface);
    for point in [[-5.0, 0.0, 5.0], [0.0, 5.0, 5.0], [5.0, 0.0, 5.0]] {
        let entity = Entity::new("p", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("SketchOnSrf").unwrap();
    let sketch = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "SketchOnSrf")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &sketch.geometry
    else {
        panic!("polycurve");
    };
    assert_eq!(segments.len(), 2);
}
