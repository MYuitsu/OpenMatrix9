// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, InvocationOrigin, LINE, POINT};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Unit},
    model::Document,
};
use om9_scene::camera::CPlane;
fn controller() -> Controller {
    Controller::new(Document::new("test"), Limits::default()).unwrap()
}
#[test]
fn menu_cmd_and_f6_dispatch_the_same_line_phases_and_atomic_commit() {
    for origin in [
        InvocationOrigin::Menu,
        InvocationOrigin::Cmd,
        InvocationOrigin::F6,
    ] {
        let mut c = controller();
        c.dispatch(LINE, origin).unwrap();
        assert_eq!(c.prompt(), "Start of line (BothSides=No Length=): ");
        c.accept_world_point([1., 2., 3.]).unwrap();
        assert!(c.document().entities.is_empty());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.accept_world_point([4., 6., 3.]).unwrap();
        assert!(!c.is_running());
        assert_eq!(c.coordinator().undo_len(), 1);
        let e = c.document().entities.values().next().unwrap();
        assert_eq!(
            e.geometry,
            Geometry::Curve(Curve::Line {
                start: [1., 2., 3.],
                end: [4., 6., 3.],
                domain: [0., 5.]
            })
        );
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 1);
    }
}
#[test]
fn cancel_and_invalid_second_point_preserve_world_and_executable_redo() {
    let mut c = controller();
    c.dispatch(POINT, InvocationOrigin::Menu).unwrap();
    c.accept_world_point([9., 8., 7.]).unwrap();
    c.undo().unwrap();
    let before = c.coordinator().snapshot();
    c.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    c.accept_world_point([1., 2., 3.]).unwrap();
    assert!(c.accept_world_point([f64::NAN, 0., 0.]).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel();
    assert_eq!(c.coordinator().snapshot(), before);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}
#[test]
fn numeric_cmd_points_use_page_units_and_captured_cplane_exactly_once() {
    let mut d = Document::new("inch");
    d.page_unit = Unit::Inch;
    let mut c = Controller::new(d, Limits::default()).unwrap();
    c.set_cplane(CPlane::new([10., 20., 30.], [0., 1., 0.], [0., 0., 1.]).unwrap())
        .unwrap();
    c.submit("Line").unwrap();
    c.submit("1,2,0").unwrap();
    c.submit("2,2,0").unwrap();
    let Geometry::Curve(Curve::Line { start, end, .. }) =
        c.document().entities.values().next().unwrap().geometry
    else {
        panic!("line master")
    };
    assert_eq!(start, [10., 45.4, 80.8]);
    assert_eq!(end, [10., 70.8, 80.8]);
}
#[test]
fn both_sides_uses_start_as_midpoint_and_empty_enter_never_repeats_active_session() {
    let mut c = controller();
    c.submit("Line").unwrap();
    c.submit("BothSides=Yes").unwrap();
    c.submit("1,2,3").unwrap();
    assert!(c.submit("").is_err());
    assert!(c.is_running());
    assert!(c.document().entities.is_empty());
    c.submit("4,6,3").unwrap();
    let Geometry::Curve(Curve::Line { start, end, domain }) =
        c.document().entities.values().next().unwrap().geometry
    else {
        panic!("line master")
    };
    assert_eq!(start, [-2., -2., 3.]);
    assert_eq!(end, [4., 6., 3.]);
    assert_eq!(domain, [0., 10.]);
    c.submit("").unwrap();
    assert!(c.is_running());
}
#[test]
fn unavailable_command_and_unknown_options_cannot_publish_or_replace_active_session() {
    let mut c = controller();
    c.submit("Line").unwrap();
    c.submit("0,0,0").unwrap();
    let before = c.coordinator().snapshot();
    assert!(c.dispatch("OM9-SOLID-012", InvocationOrigin::F6).is_err());
    assert!(c.submit("Normal=Yes").is_err());
    assert!(c.submit("Invented=Yes").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.prompt(), "End of line (BothSides=No Length=): ");
}
#[test]
fn scene_is_derived_from_published_document_and_recent_icons_hold_nine_completed_commands() {
    let mut c = controller();
    for i in 0..11 {
        c.dispatch(POINT, InvocationOrigin::Menu).unwrap();
        c.accept_world_point([i as f64, 0., 0.]).unwrap();
    }
    assert_eq!(c.recent().len(), 9);
    assert_eq!(c.scene().unwrap().objects.len(), 11);
    assert_eq!(c.scene().unwrap().epoch, c.coordinator().epoch());
    c.dispatch(LINE, InvocationOrigin::Menu).unwrap();
    c.cancel();
    assert_eq!(c.recent().len(), 9);
}
