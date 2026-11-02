// SPDX-License-Identifier: LGPL-2.1-or-later
//! `RefitSrf` (OM9-SURFACE-142, Surface `Refit to Tolerance`): the command waits for a
//! tolerance, refits the selected surface inside it and commits one transaction.
use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::{Geometry, NurbsSurface};
use om9_core::identity::ObjectId;
use om9_core::model::{Document, Entity};
use om9_core::surface_kernel;

fn cylinder() -> NurbsSurface {
    surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        6.0,
        12.0,
        &Limits::default(),
    )
    .unwrap()
}

fn controller_with(surface: NurbsSurface) -> (Controller, ObjectId) {
    let mut document = Document::new("refit srf");
    let entity = Entity::new("surface", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    (controller, id)
}

fn max_deviation(first: &NurbsSurface, second: &NurbsSurface) -> f64 {
    let limits = Limits::default();
    let mut worst = 0.0_f64;
    for i in 0..=12 {
        for j in 0..=12 {
            let u = i as f64 / 12.0;
            let v = j as f64 / 12.0;
            let a = surface_kernel::evaluate(first, u, v, &limits).unwrap();
            let b = surface_kernel::evaluate(second, u, v, &limits).unwrap();
            worst = worst.max(
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
            );
        }
    }
    worst
}

#[test]
fn refit_srf_waits_for_a_tolerance_and_commits_one_surface() {
    let (mut controller, source) = controller_with(cylinder());
    controller
        .dispatch("OM9-SURFACE-142", InvocationOrigin::Menu)
        .expect("refit srf");
    assert!(controller.prompt().contains("Tolerance"));
    assert!(controller.is_running());

    let options = controller.command_options();
    let tolerance = options
        .iter()
        .find(|option| option.label.starts_with("Tolerance="))
        .expect("the run publishes the tolerance option");
    assert_eq!(
        controller.activate_command_option(tolerance.id).unwrap(),
        CommandOptionEffect::Draft("Tolerance=".to_owned())
    );

    controller.submit("Tolerance=0.05").unwrap();
    assert!(controller.is_running(), "setting a value does not commit");
    assert_eq!(controller.coordinator().undo_len(), 0);
    controller.submit("").unwrap();

    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let refit = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("refit surface");
    let Geometry::Surface(refit) = &refit.geometry else {
        panic!("refit inserts a surface");
    };
    let source_surface = match &controller.document().entities[&source].geometry {
        Geometry::Surface(surface) => surface.clone(),
        other => panic!("source is a surface, got {other:?}"),
    };
    assert!(
        max_deviation(&source_surface, refit) <= 0.1,
        "the independent sampling stays inside twice the tolerance"
    );
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
}

#[test]
fn refit_srf_refuses_a_non_positive_tolerance_and_keeps_the_prompt() {
    let (mut controller, _) = controller_with(cylinder());
    controller
        .dispatch("OM9-SURFACE-142", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.submit("Tolerance=0").is_err());
    assert!(controller.submit("Tolerance=abc").is_err());
    assert!(
        controller.submit("0.02").is_err(),
        "bare numbers are not accepted"
    );
    assert!(
        controller.is_running(),
        "invalid input keeps the pending stage"
    );
    assert_eq!(controller.coordinator().undo_len(), 0);
    assert_eq!(controller.document().entities.len(), 1);
    controller.submit("Cancel").unwrap();
    assert!(!controller.is_running());
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn refit_srf_without_a_surface_selection_is_refused_without_a_transaction() {
    let mut controller =
        Controller::new(Document::new("refit refusal"), Limits::default()).unwrap();
    assert!(
        controller
            .dispatch("OM9-SURFACE-142", InvocationOrigin::Menu)
            .is_err()
    );
    assert_eq!(controller.coordinator().undo_len(), 0);
    assert!(controller.document().entities.is_empty());
}

#[test]
fn the_typed_refitsrf_name_dispatches_the_same_command() {
    let (mut controller, _) = controller_with(cylinder());
    controller.submit("RefitSrf").expect("typed refit srf");
    assert!(controller.prompt().contains("Tolerance"));
    controller
        .submit("")
        .expect("Enter refits with the default tolerance");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
}
