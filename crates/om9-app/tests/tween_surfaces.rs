#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn tweensurfaces_builds_an_intermediate_surface() {
    let mut doc = Document::new("tween surfaces");
    let mut ids = Vec::new();
    for z in [0.0, 10.0] {
        let surface = om9_core::surface_kernel::plane(
            [0.0, 0.0, z],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            20.0,
            20.0,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("TweenSurfaces").unwrap();
    c.submit("0.5").unwrap();
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
fn tween_fraction_uses_the_shared_entry() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut doc = Document::new("tween surfaces option");
    let mut ids = Vec::new();
    for z in [0.0, 10.0] {
        let surface = om9_core::surface_kernel::plane(
            [0.0, 0.0, z],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            20.0,
            20.0,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("TweenSurfaces").unwrap();
    let option = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Fraction")
        .expect("Fraction option");
    assert_eq!(
        c.activate_command_option(option.id).unwrap(),
        CommandOptionEffect::Draft("Fraction=".into())
    );
    c.submit("Fraction=0.5").unwrap();
    assert!(
        c.document()
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
            .count()
            >= 3
    );
}
