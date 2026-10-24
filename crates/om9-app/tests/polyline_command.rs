// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{budget::Limits, model::Document};

#[test]
fn empty_enter_finishes_polyline_and_delete_removes_it() {
    let limits = Limits::default();
    let mut controller = Controller::new(Document::new("polyline"), limits).unwrap();
    controller
        .dispatch("OM9-CURVE-001", InvocationOrigin::Menu)
        .expect("polyline");
    controller.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    controller.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    controller.accept_world_point([10.0, 10.0, 0.0]).unwrap();
    controller.submit("").expect("Enter finishes polyline");
    assert_eq!(controller.document().entities.len(), 1);
    let id = *controller.document().entities.keys().next().unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.delete_objects(&[id]).expect("delete");
    assert_eq!(controller.document().entities.len(), 0);
    assert_eq!(controller.coordinator().undo_len(), 2);
}
