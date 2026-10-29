// SPDX-License-Identifier: LGPL-2.1-or-later
//! `Smash` (OM9-SURFACE-141): flatten a surface that `Unroll Surface` can only
//! approximate, report the measured stretch, and commit one transaction.
use om9_app::command_events::EventKind;
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::{Geometry, NurbsSurface};
use om9_core::identity::ObjectId;
use om9_core::model::{Document, Entity};
use om9_core::surface_kernel;

fn saddle() -> NurbsSurface {
    let mut poles = Vec::new();
    for i in 0..3 {
        let x = i as f64 - 1.0;
        for j in 0..3 {
            let y = j as f64 - 1.0;
            poles.push([2.0 * x, 2.0 * y, 1.5 * x * y]);
        }
    }
    NurbsSurface {
        degree: [2, 2],
        size: [3, 3],
        poles,
        weights: vec![1.0; 9],
        knots: [
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

fn controller_with(surface: NurbsSurface) -> (Controller, ObjectId) {
    let mut document = Document::new("smash");
    let entity = Entity::new("surface", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    (controller, id)
}

fn stretch_report(controller: &Controller) -> f64 {
    let event = controller
        .events
        .events()
        .iter()
        .find(|event| event.command == "OM9-SURFACE-141" && event.kind == EventKind::Progress)
        .expect("smash reports its measured stretch");
    event
        .detail
        .trim_start_matches("stretch ")
        .parse()
        .expect("numeric stretch")
}

#[test]
fn smash_flattens_a_non_developable_surface_and_reports_the_stretch() {
    let (mut controller, source) = controller_with(saddle());
    controller
        .dispatch("OM9-SURFACE-141", InvocationOrigin::Menu)
        .expect("smash");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let flattened = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("flattened surface");
    let Geometry::Surface(flat) = &flattened.geometry else {
        panic!("smash inserts a surface");
    };
    assert!(
        flat.poles.iter().all(|pole| pole[2].abs() < 1e-9),
        "the smashed surface lies on the plane"
    );
    assert!(
        stretch_report(&controller) > 0.05,
        "a doubly curved surface reports its distortion"
    );
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn smash_of_a_developable_cylinder_reports_no_stretch() {
    let cylinder = surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        6.0,
        &Limits::default(),
    )
    .unwrap();
    let (mut controller, _) = controller_with(cylinder);
    controller
        .dispatch("OM9-SURFACE-141", InvocationOrigin::Menu)
        .expect("smash");
    assert!(stretch_report(&controller) < 1e-9);
}

#[test]
fn the_typed_smash_name_dispatches_the_same_command() {
    let (mut controller, _) = controller_with(saddle());
    controller.submit("Smash").expect("typed smash");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn smash_without_a_surface_selection_is_refused_without_a_transaction() {
    let mut controller =
        Controller::new(Document::new("smash refusal"), Limits::default()).unwrap();
    assert!(
        controller
            .dispatch("OM9-SURFACE-141", InvocationOrigin::Menu)
            .is_err()
    );
    assert_eq!(controller.coordinator().undo_len(), 0);
    assert!(controller.document().entities.is_empty());
    assert!(controller.events.events().is_empty());
}
