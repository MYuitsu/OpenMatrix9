#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn intersect_two_sets_creates_all_crossing_points() {
    let mut doc = Document::new("intersect two sets");
    let lines = [
        ([-1.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        ([-1.0, 10.0, 0.0], [10.0, 10.0, 0.0]),
        ([0.0, -1.0, 0.0], [0.0, 10.0, 0.0]),
        ([10.0, -1.0, 0.0], [10.0, 10.0, 0.0]),
    ];
    let mut ids = Vec::new();
    for (start, end) in lines {
        let curve = om9_core::curve_build::polyline(&[start, end]).unwrap();
        let entity = Entity::new("line", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("IntersectTwoSets").unwrap();
    let points: Vec<_> = c
        .document()
        .entities
        .values()
        .filter_map(|entity| match entity.geometry {
            Geometry::Point(point) => Some(point),
            _ => None,
        })
        .collect();
    assert_eq!(points.len(), 4, "{points:?}");
}
