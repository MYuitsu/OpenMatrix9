#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

#[test]
fn spiral_grows_from_start_to_end_radius() {
    let mut c = Controller::new(Document::new("spiral"), Limits::default()).unwrap();
    c.submit("Spiral").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("EndRadius=20").unwrap();
    c.submit("Turns=2").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    let radius = |point: &[f64; 3]| (point[0].powi(2) + point[1].powi(2)).sqrt();
    let first = radius(&curve.poles[0]);
    let last = radius(curve.poles.last().unwrap());
    assert!((first - 10.0).abs() < 0.5, "{first}");
    assert!((last - 20.0).abs() < 0.5, "{last}");
}


#[test]
fn spiral_preview_grows_with_the_radius() {
    let mut c = Controller::new(Document::new("spiral preview"), Limits::default()).unwrap();
    c.submit("Spiral").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("EndRadius=20").unwrap();
    let wire = c.spiral_preview([10.0, 0.0, 0.0]);
    assert!(wire.len() >= 8);
    let max_radius = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| (point[0].powi(2) + point[1].powi(2)).sqrt())
        .fold(f64::MIN, f64::max);
    assert!(max_radius > 19.0, "{max_radius}");
}


#[test]
fn spiral_exposes_options() {
    let mut c = Controller::new(Document::new("spiral options"), Limits::default()).unwrap();
    c.submit("Spiral").unwrap();
    let tokens = c.spiral_option_tokens();
    for expected in ["EndRadius=", "Turns=", "Height="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}


#[test]
fn spiral_accepts_a_custom_axis() {
    let mut c = Controller::new(Document::new("spiral axis"), Limits::default()).unwrap();
    c.submit("Spiral").unwrap();
    c.submit("Axis=1,0,0").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    c.submit("EndRadius=20").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[0].abs() < 0.2));
    let max_radius = curve
        .poles
        .iter()
        .map(|point| (point[1].powi(2) + point[2].powi(2)).sqrt())
        .fold(f64::MIN, f64::max);
    assert!(max_radius > 19.0, "{max_radius}");
}


#[test]
fn spiral_accepts_start_radius_option() {
    let mut c = Controller::new(Document::new("spiral start"), Limits::default()).unwrap();
    c.submit("Spiral").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("StartRadius=10").unwrap();
    c.submit("EndRadius=20").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    let min_radius = curve
        .poles
        .iter()
        .map(|point| (point[0].powi(2) + point[1].powi(2)).sqrt())
        .fold(f64::MAX, f64::min);
    assert!((min_radius - 10.0).abs() < 0.3, "{min_radius}");
}
