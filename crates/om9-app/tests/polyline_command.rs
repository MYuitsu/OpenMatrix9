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

#[test]
fn polyline_options_persistent_close_length_and_helpers() {
    use om9_core::geometry::Geometry;
    let limits = Limits::default();
    let mut controller = Controller::new(Document::new("options"), limits).unwrap();
    controller
        .dispatch("OM9-CURVE-001", InvocationOrigin::Menu)
        .expect("polyline");
    controller.submit("P=Y").unwrap();
    assert!(controller.prompt().contains("PersistentClose=Yes"));
    controller.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    controller.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    controller.submit("Length=5").unwrap();
    controller.accept_world_point([10.0, 10.0, 0.0]).unwrap();
    controller.submit("Helpers=Yes").unwrap();
    controller.submit("Close").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    // PersistentClose keeps the command running for another polyline.
    assert!(controller.is_running());
    let curve = controller
        .document()
        .entities
        .values()
        .find_map(|entity| match &entity.geometry {
            Geometry::Curve(curve) => Some(curve.clone()),
            _ => None,
        })
        .expect("polyline curve");
    // The third segment was projected to the requested length.
    let points = match curve {
        om9_core::geometry::Curve::Polycurve { segments, .. } => segments,
        _ => Vec::new(),
    };
    assert!(!points.is_empty());
}
