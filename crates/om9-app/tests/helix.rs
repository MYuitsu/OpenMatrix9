#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

#[test]
fn helix_uses_height_and_turns() {
    let mut c = Controller::new(Document::new("helix"), Limits::default()).unwrap();
    c.submit("Helix").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Height=20").unwrap();
    c.submit("Turns=2").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    let min_z = curve.poles.iter().map(|point| point[2]).fold(f64::MAX, f64::min);
    let max_z = curve.poles.iter().map(|point| point[2]).fold(f64::MIN, f64::max);
    assert!(min_z.abs() < 1e-6);
    assert!((max_z - 20.0).abs() < 1e-6);
}


#[test]
fn helix_follows_the_active_cplane_axis() {
    let mut c = Controller::new(Document::new("helix front"), Limits::default()).unwrap();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Helix").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Height=20").unwrap();
    c.submit("Turns=1").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    let y_max = curve.poles.iter().map(|point| point[1].abs()).fold(f64::MIN, f64::max);
    assert!((y_max - 20.0).abs() < 1e-6);
}


#[test]
fn helix_preview_is_a_wire() {
    let mut c = Controller::new(Document::new("helix preview"), Limits::default()).unwrap();
    c.submit("Helix").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.helix_preview([10.0, 0.0, 0.0]);
    assert!(wire.len() >= 8);
    for [point, _] in wire {
        let radius = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radius - 10.0).abs() < 1e-9);
    }
}


#[test]
fn helix_exposes_options_and_undo() {
    let mut c = Controller::new(Document::new("helix options"), Limits::default()).unwrap();
    c.submit("Helix").unwrap();
    let tokens = c.helix_option_tokens();
    for expected in ["Height=", "Turns="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Undo").unwrap();
    assert!(c.prompt().starts_with("Axis start"));
}


#[test]
fn helix_accepts_a_custom_axis() {
    let mut c = Controller::new(Document::new("helix axis"), Limits::default()).unwrap();
    c.submit("Helix").unwrap();
    c.submit("Axis=1,0,0").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    c.submit("Height=20").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    let x_max = curve.poles.iter().map(|point| point[0]).fold(f64::MIN, f64::max);
    assert!((x_max - 20.0).abs() < 1e-6, "{x_max}");
    assert!(curve
        .poles
        .iter()
        .all(|point| ((point[1].powi(2) + point[2].powi(2)).sqrt() - 10.0).abs() < 0.2));
}


#[test]
fn helix_accepts_radius_option() {
    let mut c = Controller::new(Document::new("helix radius"), Limits::default()).unwrap();
    c.submit("Helix").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Radius=10").unwrap();
    c.submit("Height=20").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    assert!(curve
        .poles
        .iter()
        .all(|point| ((point[0].powi(2) + point[1].powi(2)).sqrt() - 10.0).abs() < 0.3));
}
