#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn seamsrf_moves_a_closed_surface_seam() {
    let mut doc = Document::new("seam srf");
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
    c.submit("SeamSrf").unwrap();
    c.submit("U,0.01").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 2
    );
}

#[test]
fn seam_options_stage_axis_then_tolerance() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut doc = Document::new("seam options");
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
    c.submit("SeamSrf").unwrap();
    let axis = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Axis")
        .expect("Axis option");
    assert_eq!(
        c.activate_command_option(axis.id).unwrap(),
        CommandOptionEffect::Draft("Axis=".into())
    );
    c.submit("Axis=U").unwrap();
    assert!(c.is_running());
    let tolerance = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Tolerance")
        .expect("Tolerance option");
    assert_eq!(
        c.activate_command_option(tolerance.id).unwrap(),
        CommandOptionEffect::Draft("Tolerance=".into())
    );
    c.submit("Tolerance=0.01").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 2
    );
}
