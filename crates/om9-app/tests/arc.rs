#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

fn controller() -> Controller {
    Controller::new(Document::new("arc"), Limits::default()).unwrap()
}

fn arc(c: &Controller) -> (f64, [f64; 2]) {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle {
        radius,
        angle_domain,
        ..
    }) = &c.document().entities[&id].geometry
    else {
        panic!("arc circle");
    };
    (*radius, angle_domain.expect("arc angles"))
}

#[test]
fn arc_three_points_match_the_circumcircle() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    let (radius, angles) = arc(&c);
    assert!((radius - 1.0).abs() < 1e-9);
    assert!((angles[1] - angles[0] - std::f64::consts::PI).abs() < 1e-6);
}

#[test]
fn arc_center_mode_uses_start_and_end_angles() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    let (radius, angles) = arc(&c);
    assert!((radius - 1.0).abs() < 1e-9);
    assert!((angles[1] - angles[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn arc_preview_has_thirty_two_segments() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    assert_eq!(c.arc_preview([-1.0, 0.0, 0.0]).len(), 32);
}

#[test]
fn arc_start_direction_uses_the_start_tangent() {
    let mut c = controller();
    c.submit("Arc_StartPoint").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    let (radius, angles) = arc(&c);
    assert!((radius - 0.5).abs() < 1e-9);
    assert!((angles[1] - angles[0] - std::f64::consts::PI).abs() < 1e-6);
}

#[test]
fn arc_center_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 0.0, 5.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { x, y, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("arc");
    };
    let normal = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    assert!(normal[1].abs() > 0.99, "{normal:?}");
}

#[test]
fn arc_center_preview_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    let wire = c.arc_preview([0.0, 0.0, 5.0]);
    assert_eq!(wire.len(), 32);
    for [point, _] in wire {
        assert!(point[1].abs() < 1e-9);
        let radius = (point[0].powi(2) + point[2].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9, "{radius}");
    }
}

#[test]
fn arc_start_point_preview_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Arc").unwrap();
    c.submit("StartPoint").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    let wire = c.arc_preview([0.0, 0.0, 5.0]);
    assert_eq!(wire.len(), 32);
    for [point, _] in wire {
        assert!(point[1].abs() < 1e-9);
    }
}

#[test]
fn arc_mode_tokens_and_mode_lock() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    let tokens = c.arc_option_tokens();
    for expected in ["3Point", "Center", "StartPoint"] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert!(c.submit("3point").is_err());
}

#[test]
fn arc_radius_option_sets_the_radius() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.submit("Radius=5").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    let (radius, angles) = arc(&c);
    assert!((radius - 5.0).abs() < 1e-9);
    assert!((angles[1] - angles[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn arc_mode_change_clears_the_radius_option() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.submit("Radius=5").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    let (radius, _) = arc(&c);
    assert!((radius - 1.0).abs() < 1e-9, "{radius}");
}

#[test]
fn arc_radius_preview_uses_the_preset() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.submit("Radius=5").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let wire = c.arc_preview([0.0, 10.0, 0.0]);
    assert_eq!(wire.len(), 32);
    for [point, _] in wire {
        let radius = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9, "{radius}");
    }
}

#[test]
fn arc_accepts_diameter_option() {
    let mut c = controller();
    c.submit("Arc").unwrap();
    c.submit("Center").unwrap();
    c.submit("Diameter=10").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    let (radius, _) = arc(&c);
    assert!((radius - 5.0).abs() < 1e-9);
}

#[test]
fn arc_center_uses_the_shared_choice_option() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut c = controller();
    c.submit("Arc").unwrap();
    let center = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Center")
        .expect("Center option");
    assert_eq!(
        c.activate_command_option(center.id).unwrap(),
        CommandOptionEffect::Applied
    );
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    let (radius, angles) = arc(&c);
    assert!((radius - 1.0).abs() < 1e-9);
    assert!((angles[1] - angles[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
}
