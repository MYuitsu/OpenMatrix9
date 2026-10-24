#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

fn controller() -> Controller {
    Controller::new(Document::new("polyline"), Limits::default()).unwrap()
}

fn segments(c: &Controller) -> usize {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("polycurve");
    };
    segments.len()
}

#[test]
fn polyline_enter_commits_open_segments() {
    let mut c = controller();
    c.submit("Polyline").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 10.0, 0.0]).unwrap();
    c.submit("").unwrap();
    assert_eq!(segments(&c), 2);
}

#[test]
fn polyline_close_adds_the_closing_segment() {
    let mut c = controller();
    c.submit("Polyline").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 10.0, 0.0]).unwrap();
    c.submit("Close").unwrap();
    assert_eq!(segments(&c), 3);
}

#[test]
fn polyline_preview_follows_the_cursor() {
    let mut c = controller();
    c.submit("Polyline").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.polyline_preview([10.0, 0.0, 0.0]).len(), 1);
}
