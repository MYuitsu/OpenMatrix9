// SPDX-License-Identifier: LGPL-2.1-or-later
//! Developer fallback rejects unavailable BRep operations without publishing geometry.
#![cfg(not(feature = "occt-brep"))]
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{budget::Limits, model::Document};

#[test]
fn sphere_and_box_fallback_reject_without_document_changes() {
    let mut controller = Controller::new(Document::new("no OCCT"), Limits::default()).unwrap();
    let before = controller.document().clone();
    assert!(controller.accept_sphere_point([1., 2., 3.]).is_err());
    let plane = controller.input_plane();
    assert!(controller.solid_box_dimensions([0., 0., 0.], 2., 3., 4., plane).is_err());
    assert!(Controller::availability("OM9-SOLID-014").is_err());
    assert!(Controller::availability("OM9-SOLID-012").is_err());
    assert!(controller.submit("Sphere").is_err());
    assert!(controller.submit("Box").is_err());
    assert!(controller.dispatch("OM9-SOLID-014", InvocationOrigin::Menu).is_err());
    assert!(controller.dispatch("OM9-SOLID-012", InvocationOrigin::Menu).is_err());
    assert!(!controller.is_running());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}
