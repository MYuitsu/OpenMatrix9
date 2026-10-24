#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_circle() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("project");
    let curve = om9_core::curve_build::circle_on_frame(
        [0.0, 0.0, 5.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
    )
    .unwrap();
    let entity = Entity::new("circle", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    (Controller::new(doc, Limits::default()).unwrap(), id)
}

#[test]
fn project_curve_drops_to_the_construction_plane() {
    let (mut c, id) = controller_with_circle();
    c.select_objects(&[id]).unwrap();
    c.submit("Project").unwrap();
    c.submit("").unwrap();
    let projected = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Project")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &projected.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[2].abs() < 1e-9));
    let radius = curve
        .poles
        .iter()
        .map(|point| (point[0].powi(2) + point[1].powi(2)).sqrt())
        .sum::<f64>()
        / curve.poles.len() as f64;
    assert!((radius - 10.0).abs() < 0.1, "{radius}");
}


#[test]
fn project_uses_the_active_cplane() {
    let (mut c, id) = controller_with_circle();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Project").unwrap();
    c.submit("").unwrap();
    let projected = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Project")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &projected.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[1].abs() < 1e-6));
}


#[test]
fn project_direction_vector_controls_the_ray() {
    let (mut c, id) = controller_with_circle();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Project").unwrap();
    c.submit("Direction=0,1,0").unwrap();
    c.submit("").unwrap();
    let projected = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Project")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &projected.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[1].abs() < 1e-6));
    assert!(curve.poles.iter().any(|point| (point[2] - 5.0).abs() < 0.1));
}


#[test]
fn project_exposes_direction_token() {
    let mut doc = Document::new("project token");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 5.0], [10.0, 0.0, 5.0]]).unwrap();
    let entity = Entity::new("curve", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Project").unwrap();
    assert!(c
        .project_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Direction="));
}
