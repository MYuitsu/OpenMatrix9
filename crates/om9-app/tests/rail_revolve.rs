#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn rail_revolve_uses_the_axis_input() {
    let mut doc = Document::new("rail revolve");
    let mut ids = Vec::new();
    for (start, end) in [
        ([10.0, 0.0, 0.0], [10.0, 0.0, 10.0]),
        ([10.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
    ] {
        let entity = Entity::new(
            "curve",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        );
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("RailRevolve").unwrap();
    c.submit("0,0,0,0,0,1").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .any(|entity| matches!(entity.geometry, Geometry::Surface(_)))
    );
}

#[test]
fn rail_revolve_axis_option_uses_the_shared_entry() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut doc = Document::new("rail revolve option");
    let mut ids = Vec::new();
    for (start, end) in [
        ([10.0, 0.0, 0.0], [10.0, 0.0, 10.0]),
        ([10.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
    ] {
        let entity = Entity::new(
            "curve",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        );
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("RailRevolve").unwrap();
    let option = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Axis")
        .expect("Axis option");
    assert_eq!(
        c.activate_command_option(option.id).unwrap(),
        CommandOptionEffect::Draft("Axis=".into())
    );
    c.submit("Axis=0,0,0,0,0,1").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .any(|entity| matches!(entity.geometry, Geometry::Surface(_)))
    );
}
