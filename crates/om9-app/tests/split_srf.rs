#![cfg(feature = "occt-brep")]

use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn splitsrf_options_stage_axis_then_parameter() {
    let mut doc = Document::new("split surface option");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        20.0,
        20.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.dispatch("OM9-SURFACE-034", InvocationOrigin::Menu)
        .unwrap();
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
    let parameter = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Parameter")
        .expect("Parameter option");
    assert_eq!(
        c.activate_command_option(parameter.id).unwrap(),
        CommandOptionEffect::Draft("Parameter=".into())
    );
    c.submit("Parameter=0.5").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 3
    );
    assert_eq!(c.coordinator().undo_len(), 1);
}
