#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

fn controller() -> Controller {
    Controller::new(Document::new("interp"), Limits::default()).unwrap()
}

fn degree_of(c: &Controller) -> usize {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    curve.degree
}

#[test]
fn interp_curve_fits_four_points() {
    let mut c = controller();
    c.submit("InterpCrv").unwrap();
    assert!(c
        .interp_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Degree="));
    for point in [
        [0.0, 0.0, 0.0],
        [5.0, 5.0, 0.0],
        [10.0, -5.0, 0.0],
        [15.0, 0.0, 0.0],
    ] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    assert_eq!(degree_of(&c), 3);
}

#[test]
fn interp_curve_accepts_degree_option() {
    let mut c = controller();
    c.submit("InterpCrv").unwrap();
    c.submit("Degree=5").unwrap();
    for index in 0..7 {
        c.accept_world_point([index as f64 * 5.0, (index % 2) as f64 * 5.0, 0.0])
            .unwrap();
    }
    c.submit("").unwrap();
    assert_eq!(degree_of(&c), 5);
}

#[test]
fn interp_curve_preview_follows_the_cursor() {
    let mut c = controller();
    c.submit("InterpCrv").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.interp_preview([10.0, 0.0, 0.0]).len(), 2);
}


#[test]
fn interp_preview_uses_the_interpolated_curve() {
    let mut c = controller();
    c.submit("InterpCrv").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 5.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let wire = c.interp_preview([15.0, 5.0, 0.0]);
    assert_eq!(wire.len(), 31);
    assert!((wire[0][0][0]).abs() < 1e-9);
}
