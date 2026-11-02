#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn plane_controller() -> Controller {
    let mut doc = Document::new("surface edit");
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
    c
}

#[test]
fn extendsrf_extends_a_plane_edge() {
    let mut c = plane_controller();
    c.submit("ExtendSrf").unwrap();
    c.submit("U1,5").unwrap();
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
fn shrinksrf_shrinks_a_plane() {
    let mut c = plane_controller();
    c.submit("ShrinkSrf").unwrap();
    c.submit("0.1,0.9,0.1,0.9").unwrap();
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
fn extendsrf_options_stage_edge_then_distance() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut c = plane_controller();
    c.submit("ExtendSrf").unwrap();
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
    let distance = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Distance")
        .expect("Distance option");
    assert_eq!(
        c.activate_command_option(distance.id).unwrap(),
        CommandOptionEffect::Draft("Distance=".into())
    );
    c.submit("Distance=5").unwrap();
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
fn shrinksrf_options_stage_four_uv_values() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut c = plane_controller();
    c.submit("ShrinkSrf").unwrap();
    for (name, draft, value) in [
        ("U0", "U0=", "U0=0.1"),
        ("U1", "U1=", "U1=0.9"),
        ("V0", "V0=", "V0=0.1"),
        ("V1", "V1=", "V1=0.9"),
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
