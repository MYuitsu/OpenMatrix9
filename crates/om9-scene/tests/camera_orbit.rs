//! Camera orbit, pan and zoom: the three mouse operations the viewport exposes.
//!
//! Every expectation is geometric: the orbit keeps the distance and the target, the elevation is
//! clamped short of the poles, panning moves eye and target together, and zoom scales the distance.

use om9_scene::camera::{Camera, Projection};

fn perspective() -> Camera {
    Camera::perspective([0., -20., 10.], [0., 0., 0.], [0., 0., 1.], 0.8).expect("camera")
}

fn distance(camera: &Camera) -> f64 {
    let offset = [
        camera.eye[0] - camera.target[0],
        camera.eye[1] - camera.target[1],
        camera.eye[2] - camera.target[2],
    ];
    (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt()
}

#[test]
fn orbit_keeps_the_target_and_the_distance() {
    let mut camera = perspective();
    let before = distance(&camera);
    camera.orbit(0.4, 0.2).expect("orbit");
    assert_eq!(camera.target, [0., 0., 0.], "the target must not move");
    assert!((distance(&camera) - before).abs() < 1e-9, "the distance changed");
    assert!(camera.revision > 1, "an orbit bumps the revision");
}

#[test]
fn orbit_clamps_the_elevation_below_the_pole() {
    let mut camera = perspective();
    camera.orbit(0., 10.).expect("pitch up");
    let offset = [
        camera.eye[0] - camera.target[0],
        camera.eye[1] - camera.target[1],
        camera.eye[2] - camera.target[2],
    ];
    let radius = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
    let elevation = (offset[2] / radius).clamp(-1., 1.).asin();
    assert!(elevation < std::f64::consts::FRAC_PI_2 - 0.01, "the eye reached the up axis: {elevation}");
    camera.orbit(0., -20.).expect("pitch down");
    let offset = [
        camera.eye[0] - camera.target[0],
        camera.eye[1] - camera.target[1],
        camera.eye[2] - camera.target[2],
    ];
    let radius = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
    let elevation = (offset[2] / radius).clamp(-1., 1.).asin();
    assert!(elevation > -std::f64::consts::FRAC_PI_2 + 0.01, "the eye reached the down axis: {elevation}");
}

#[test]
fn pan_moves_eye_and_target_together() {
    let mut camera = perspective();
    let eye_before = camera.eye;
    let target_before = camera.target;
    camera.pan([2., -1.]).expect("pan");
    let eye_shift = [camera.eye[0] - eye_before[0], camera.eye[1] - eye_before[1], camera.eye[2] - eye_before[2]];
    let target_shift = [
        camera.target[0] - target_before[0],
        camera.target[1] - target_before[1],
        camera.target[2] - target_before[2],
    ];
    for axis in 0..3 {
        assert!((eye_shift[axis] - target_shift[axis]).abs() < 1e-12, "eye and target must shift equally");
    }
    assert!(eye_shift.iter().any(|value| value.abs() > 0.), "the pan must move the camera");
}

#[test]
fn zoom_scales_the_distance_in_perspective() {
    let mut camera = perspective();
    let before = distance(&camera);
    camera.zoom(0.9).expect("zoom in");
    assert!((distance(&camera) - before * 0.9).abs() < 1e-9);
    camera.zoom(2.0).expect("zoom out");
    assert!((distance(&camera) - before * 1.8).abs() < 1e-9);
}

#[test]
fn an_orthographic_view_zooms_its_span_and_keeps_the_eye() {
    let mut camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 100.).expect("ortho");
    let eye = camera.eye;
    match camera.projection {
        Projection::Orthographic { vertical_span_mm } => assert!((vertical_span_mm - 100.).abs() < 1e-9),
        _ => panic!("expected an orthographic projection"),
    }
    camera.zoom(0.5).expect("zoom");
    match camera.projection {
        Projection::Orthographic { vertical_span_mm } => assert!((vertical_span_mm - 50.).abs() < 1e-9),
        _ => panic!("expected an orthographic projection"),
    }
    assert_eq!(camera.eye, eye, "an orthographic zoom must not move the eye");
}
