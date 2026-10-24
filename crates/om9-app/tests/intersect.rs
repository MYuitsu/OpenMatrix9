#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_lines() -> Controller {
    let mut doc = Document::new("intersect");
    let first = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 10.0, 0.0]]).unwrap();
    let second = om9_core::curve_build::polyline(&[[0.0, 10.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    for (name, curve) in [("a", first), ("b", second)] {
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    Controller::new(doc, Limits::default()).unwrap()
}

#[test]
fn intersect_creates_a_point_at_the_crossing() {
    let mut c = controller_with_lines();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Intersect").unwrap();
    c.submit("").unwrap();
    let point = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Intersection")
        .unwrap();
    let Geometry::Point(point) = point.geometry else {
        panic!("point");
    };
    assert!((point[0] - 5.0).abs() < 1e-6);
    assert!((point[1] - 5.0).abs() < 1e-6);
}


#[test]
fn intersect_handles_two_circles_with_the_core_solver() {
    let mut doc = Document::new("intersect circles");
    for (name, center) in [("a", [0.0, 0.0, 0.0]), ("b", [8.0, 0.0, 0.0])] {
        let curve = om9_core::curve_build::circle_on_frame(
            center,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            5.0,
        )
        .unwrap();
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Intersect").unwrap();
    c.submit("").unwrap();
    let points: Vec<_> = c
        .document()
        .entities
        .values()
        .filter_map(|entity| match entity.geometry {
            Geometry::Point(point) => Some(point),
            _ => None,
        })
        .collect();
    assert_eq!(points.len(), 2);
    assert!(points.iter().all(|point| (point[0] - 4.0).abs() < 1e-4));
}


#[test]
fn intersect_exposes_tolerance_token() {
    let mut doc = Document::new("intersect token");
    let mut ids = Vec::new();
    for points in [
        [[0.0, 0.0, 0.0], [10.0, 10.0, 0.0]],
        [[0.0, 10.0, 0.0], [10.0, 0.0, 0.0]],
    ] {
        let curve = om9_core::curve_build::polyline(&points).unwrap();
        let entity = Entity::new("line", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Intersect").unwrap();
    assert!(c
        .intersect_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Tolerance="));
}
