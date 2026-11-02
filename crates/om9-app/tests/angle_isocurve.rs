#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn angle_isocurve_extracts_at_the_requested_angle() {
    let mut doc = Document::new("angle isocurve");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("AngleIsocurve").unwrap();
    assert!(
        c.angle_isocurve_option_tokens()
            .iter()
            .any(|(_, command)| *command == "Direction=")
    );
    c.submit("Angle=180").unwrap();
    c.submit("Direction=U").unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "AngleIsocurve")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.len() > 1);
}

#[test]
fn angle_isocurve_options_use_the_shared_snapshot() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut doc = Document::new("angle isocurve option");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("AngleIsocurve").unwrap();
    for (name, draft, value) in [
        ("Angle", "Angle=", "Angle=180"),
        ("Direction", "Direction=", "Direction=U"),
    ] {
        let option = c
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == name)
            .unwrap_or_else(|| panic!("{name} option"));
        assert_eq!(
            c.activate_command_option(option.id).unwrap(),
            CommandOptionEffect::Draft(draft.into())
        );
        c.submit(value).unwrap();
    }
    c.submit("").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .any(|entity| entity.label == "AngleIsocurve")
    );
}
