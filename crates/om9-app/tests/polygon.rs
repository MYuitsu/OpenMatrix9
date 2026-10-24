#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

fn controller() -> Controller {
    Controller::new(Document::new("polygon"), Limits::default()).unwrap()
}

fn bounds(c: &Controller) -> ([f64; 3], [f64; 3], usize) {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("polycurve");
    };
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for segment in segments {
        let om9_core::geometry::Curve::Line { start, end, .. } = segment else {
            panic!("line");
        };
        for point in [start, end] {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
    }
    (min, max, segments.len())
}

#[test]
fn polygon_defaults_to_a_square() {
    let mut c = controller();
    c.submit("Polygon").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let (min, max, segments) = bounds(&c);
    assert_eq!(segments, 4);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((min[0] + 10.0).abs() < 1e-9);
}

#[test]
fn polygon_accepts_a_side_count() {
    let mut c = controller();
    c.submit("Polygon").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Sides=6").unwrap();
    c.submit("").unwrap();
    let (_, _, segments) = bounds(&c);
    assert_eq!(segments, 6);
}


#[test]
fn polygon_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Polygon").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let (min, max, _) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[2] - 10.0).abs() < 1e-9);
    assert!(min[1].abs() < 1e-9 && max[1].abs() < 1e-9);
}


#[test]
fn polygon_exposes_options_and_undo() {
    let mut c = controller();
    c.submit("Polygon").unwrap();
    let tokens = c.polygon_option_tokens();
    for expected in ["Sides=", "Circumscribed", "Inscribed"] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Undo").unwrap();
    assert!(c.prompt().starts_with("Center of polygon"));
}


#[test]
fn polygon_preview_uses_the_cursor_radius() {
    let mut c = controller();
    c.submit("Polygon").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.polygon_preview([10.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 4);
    for [point, _] in wire {
        let radius = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radius - 10.0).abs() < 1e-9);
    }
}


#[test]
fn polygon_circumscribed_uses_the_apothem() {
    let mut c = controller();
    c.submit("Polygon").unwrap();
    c.submit("Circumscribed").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let (_, max, _) = bounds(&c);
    let expected = 10.0 / (std::f64::consts::PI / 4.0).cos();
    assert!((max[0] - expected).abs() < 1e-6, "{max:?}");
}


#[test]
fn polygon_starts_at_the_radius_point() {
    let mut c = controller();
    c.submit("Polygon").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("polycurve");
    };
    let om9_core::geometry::Curve::Line { start, .. } = &segments[0] else {
        panic!("line");
    };
    assert!(start[0].abs() < 1e-9);
    assert!((start[1] - 10.0).abs() < 1e-9);
}
