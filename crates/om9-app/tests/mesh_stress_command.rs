// SPDX-License-Identifier: LGPL-2.1-or-later
//! `MeshStress` command: it creates a lattice of mesh objects in one transaction so the GPU viewport can
//! be stressed and the objects can be selected like any other geometry.
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{budget::Limits, model::Document};

fn controller() -> Controller {
    Controller::new(Document::new("mesh stress"), Limits::default()).expect("controller")
}

#[test]
fn the_command_creates_the_requested_meshes_in_one_entry() {
    let mut controller = controller();
    let undo_before = controller.coordinator().undo_len();
    controller.submit("MeshStress 40").expect("command");
    assert_eq!(controller.document().entities.len(), 40);
    assert_eq!(
        controller.coordinator().undo_len(),
        undo_before + 1,
        "one command, one journal entry"
    );
    // The objects are real entities: they can be selected.
    let first = *controller
        .document()
        .entities
        .keys()
        .next()
        .expect("one mesh");
    controller.select_root(Some(first), false).expect("select");
    assert!(controller.selected().contains(&first));
    controller.undo().expect("undo");
    assert!(
        controller.document().entities.is_empty(),
        "one Undo removes the whole stress scene"
    );
}

#[test]
fn a_bare_command_uses_the_default_count() {
    let mut controller = controller();
    controller.submit("MeshStress").expect("command");
    assert_eq!(
        controller.document().entities.len(),
        om9_app::session::stress::DEFAULT_STRESS_MESHES
    );
}

#[test]
fn a_bad_count_is_refused_without_touching_the_document() {
    let mut controller = controller();
    let undo_before = controller.coordinator().undo_len();
    assert!(
        controller.submit("MeshStress many").is_err(),
        "a non-number must be refused"
    );
    assert!(controller.document().entities.is_empty());
    assert_eq!(controller.coordinator().undo_len(), undo_before);
    assert!(
        controller.submit("MeshStress 999999").is_err(),
        "the count is bounded"
    );
    assert!(controller.document().entities.is_empty());
}

#[test]
fn the_menu_dispatch_path_also_creates_the_default_scene() {
    let mut controller = controller();
    controller
        .dispatch("MeshStress", InvocationOrigin::Cmd)
        .expect("dispatch");
    assert_eq!(
        controller.document().entities.len(),
        om9_app::session::stress::DEFAULT_STRESS_MESHES,
        "a menu invocation and a typed command create the same scene"
    );
}
