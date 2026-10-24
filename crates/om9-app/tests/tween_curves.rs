#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn tween_curves_builds_the_midpoint_curve() {
    let mut doc = Document::new("tween");
    let first = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let second = om9_core::curve_build::polyline(&[[0.0, 10.0, 0.0], [10.0, 10.0, 0.0]]).unwrap();
    for (name, curve) in [("a", first), ("b", second)] {
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("TweenCurves").unwrap();
    c.submit("").unwrap();
    let tween = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "TweenCurves")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &tween.geometry else {
        panic!("nurbs");
    };
    let average = curve.poles.iter().map(|point| point[1]).sum::<f64>() / curve.poles.len() as f64;
    assert!((average - 5.0).abs() < 0.5, "{average}");
}


#[test]
fn tween_curves_count_creates_multiple_curves() {
    let mut doc = Document::new("tween count");
    let mut ids = Vec::new();
    for y in [0.0, 10.0] {
        let curve = om9_core::curve_build::polyline(&[[0.0, y, 0.0], [10.0, y, 0.0]]).unwrap();
        let entity = Entity::new("curve", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("TweenCurves").unwrap();
    c.submit("Count=3").unwrap();
    c.submit("").unwrap();
    let count = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "TweenCurves")
        .count();
    assert_eq!(count, 3);
}
