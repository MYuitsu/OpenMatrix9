// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
    surface_kernel,
};

#[test]
fn surface_distance_uses_the_shared_value_entry() {
    let limits = Limits::default();
    let surface = surface_kernel::plane([0.; 3], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits)
        .expect("plane");
    let mut document = Document::new("surface options");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    controller
        .dispatch("OM9-SURFACE-025", InvocationOrigin::Menu)
        .unwrap();
    let option = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Distance")
        .expect("Distance option");
    assert_eq!(
        controller.activate_command_option(option.id).unwrap(),
        CommandOptionEffect::Draft("Distance=".into())
    );
    controller.submit("Distance=2").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
}
