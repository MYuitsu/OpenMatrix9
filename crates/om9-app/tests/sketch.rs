#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn sketch_connects_selected_points() {
    let mut doc = Document::new("sketch");
    let mut ids = Vec::new();
    for point in [[0.0, 0.0, 0.0], [5.0, 5.0, 0.0], [10.0, 0.0, 0.0]] {
        let entity = Entity::new("p", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Sketch").unwrap();
    let sketch = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Sketch")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &sketch.geometry
    else {
        panic!("polycurve");
    };
    assert_eq!(segments.len(), 2);
}
