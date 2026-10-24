#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn curve_boolean_joins_connected_boundary_curves() {
    let mut doc = Document::new("curve boolean");
    let first = Entity::new(
        "a",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [5.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let second = Entity::new(
        "b",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [5.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let ids = [first.id, second.id];
    doc.entities.insert(first.id, first);
    doc.entities.insert(second.id, second);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("CurveBoolean").unwrap();
    let joined = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "CurveBoolean")
        .unwrap();
    let Geometry::Curve(curve) = &joined.geometry else {
        panic!("curve");
    };
    let length = om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap();
    assert!((length - 10.0).abs() < 1e-6);
}


#[test]
fn curve_boolean_unions_two_overlapping_circles() {
    let mut doc = Document::new("curve boolean circles");
    let mut ids = Vec::new();
    for center in [[0.0, 0.0, 0.0], [8.0, 0.0, 0.0]] {
        let curve = om9_core::curve_build::circle_on_frame(
            center,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            5.0,
        )
        .unwrap();
        let entity = Entity::new("circle", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("CurveBoolean").unwrap();
    let curves: Vec<_> = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "CurveBoolean")
        .collect();
    assert!(!curves.is_empty());
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    for entity in curves {
        let Geometry::Curve(curve) = &entity.geometry else {
            panic!("curve");
        };
        for point in om9_core::surface_kernel::sample_curve(curve, 64, &Limits::default()).unwrap() {
            min_x = min_x.min(point[0]);
            max_x = max_x.max(point[0]);
        }
    }
    assert!((min_x + 5.0).abs() < 0.5, "{min_x}");
    assert!((max_x - 13.0).abs() < 0.5, "{max_x}");
}


#[test]
fn curve_boolean_keeps_two_disjoint_circles() {
    let mut doc = Document::new("curve boolean disjoint");
    let mut ids = Vec::new();
    for center in [[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]] {
        let curve = om9_core::curve_build::circle_on_frame(
            center,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            5.0,
        )
        .unwrap();
        let entity = Entity::new("circle", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("CurveBoolean").unwrap();
    assert_eq!(
        c.document()
            .entities
            .values()
            .filter(|entity| entity.label == "CurveBoolean")
            .count(),
        2
    );
}

#[test]
fn curve_boolean_nested_circle_keeps_the_outer_boundary() {
    let mut doc = Document::new("curve boolean nested");
    let mut ids = Vec::new();
    for radius in [5.0, 2.0] {
        let curve = om9_core::curve_build::circle_on_frame(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            radius,
        )
        .unwrap();
        let entity = Entity::new("circle", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("CurveBoolean").unwrap();
    assert_eq!(
        c.document()
            .entities
            .values()
            .filter(|entity| entity.label == "CurveBoolean")
            .count(),
        1
    );
}
