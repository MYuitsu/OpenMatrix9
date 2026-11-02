#![cfg(feature = "occt-brep")]

use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn tangentsrf_options_stage_edge_then_direction() {
    let mut doc = Document::new("tangent surface option");
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
    c.dispatch("OM9-SURFACE-040", InvocationOrigin::Menu)
        .unwrap();
    let edge = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Edge")
        .expect("Edge option");
    assert_eq!(
        c.activate_command_option(edge.id).unwrap(),
        CommandOptionEffect::Draft("Edge=".into())
    );
    c.submit("Edge=U1").unwrap();
    assert!(c.is_running());
    let direction = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Direction")
        .expect("Direction option");
    assert_eq!(
        c.activate_command_option(direction.id).unwrap(),
        CommandOptionEffect::Draft("Direction=".into())
    );
    c.submit("Direction=0,0,1").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 2
    );
    assert_eq!(c.coordinator().undo_len(), 1);
}
