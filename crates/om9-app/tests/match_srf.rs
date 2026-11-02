#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn matchsrf_matches_two_surface_edges() {
    let mut doc = Document::new("match srf");
    let mut ids = Vec::new();
    for origin in [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]] {
        let surface = om9_core::surface_kernel::plane(
            origin,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            10.0,
            10.0,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("MatchSrf").unwrap();
    c.submit("U1,U0").unwrap();
    c.submit("").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 3
    );
}

#[test]
fn matchsrf_options_stage_source_then_target() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut doc = Document::new("match srf option");
    let mut ids = Vec::new();
    for origin in [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]] {
        let surface = om9_core::surface_kernel::plane(
            origin,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            10.0,
            10.0,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("MatchSrf").unwrap();
    let source = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Source")
        .expect("Source option");
    assert_eq!(
        c.activate_command_option(source.id).unwrap(),
        CommandOptionEffect::Draft("Source=".into())
    );
    c.submit("Source=U1").unwrap();
    assert!(c.is_running());
    let target = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Target")
        .expect("Target option");
    assert_eq!(
        c.activate_command_option(target.id).unwrap(),
        CommandOptionEffect::Draft("Target=".into())
    );
    c.submit("Target=U0").unwrap();
    c.submit("").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 3
    );
}
