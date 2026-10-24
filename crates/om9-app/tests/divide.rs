#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn divide_places_intermediate_points() {
    let mut doc = Document::new("divide");
    let curve = om9_core::curve_build::circle_on_frame(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
    )
    .unwrap();
    let entity = Entity::new("circle", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Divide").unwrap();
    c.submit("Count=4").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert_eq!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Point(_)))
            .count(),
        3
    );
}


#[test]
fn divide_uses_equal_arc_length_stations() {
    let mut doc = Document::new("divide line");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Divide").unwrap();
    c.submit("Count=4").unwrap();
    c.submit("").unwrap();
    let mut xs: Vec<f64> = c
        .document()
        .entities
        .values()
        .filter_map(|entity| match entity.geometry {
            Geometry::Point(point) => Some(point[0]),
            _ => None,
        })
        .collect();
    xs.sort_by(f64::total_cmp);
    assert_eq!(xs.len(), 3);
    for (index, expected) in [2.5, 5.0, 7.5].into_iter().enumerate() {
        assert!((xs[index] - expected).abs() < 1e-6, "{xs:?}");
    }
}


#[test]
fn divide_exposes_count_token() {
    let mut doc = Document::new("divide token");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Divide").unwrap();
    assert!(c.divide_option_tokens().iter().any(|(_, command)| *command == "Count="));
}
