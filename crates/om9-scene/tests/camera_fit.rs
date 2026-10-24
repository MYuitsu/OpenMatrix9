// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-02-005/006`: camera fit and center primitives.
use om9_scene::camera::{Camera, Projection, Viewport};

fn viewport() -> Viewport {
    Viewport {
        origin: [0., 0.],
        size: [800., 600.],
    }
}

#[test]
fn orthographic_fit_contains_a_10_by_20_by_30_box_with_margin() {
    let mut camera = Camera::orthographic([0., 0., 100.], [0.; 3], [0., 1., 0.], 40.).unwrap();
    let center = [5., 10., 15.];
    let radius = (10f64 * 10. + 20. * 20. + 30. * 30.).sqrt() * 0.5;
    camera.fit_bounds(center, radius, 4. / 3., 1.1).unwrap();
    assert_eq!(camera.target, center);
    let Projection::Orthographic { vertical_span_mm } = camera.projection else {
        panic!("orthographic");
    };
    assert!(vertical_span_mm.is_finite() && vertical_span_mm > 0.);
    for x in [0., 10.] {
        for y in [0., 20.] {
            for z in [0., 30.] {
                let projected = camera.project([x, y, z], viewport()).unwrap().unwrap();
                assert!(
                    (0.0..=800.).contains(&projected.screen[0])
                        && (0.0..=600.).contains(&projected.screen[1]),
                    "corner outside fit: {projected:?}"
                );
            }
        }
    }
}

#[test]
fn perspective_fit_contains_the_same_box() {
    let mut camera =
        Camera::perspective([35., -35., 35.], [0.; 3], [0., 0., 1.], 35f64.to_radians()).unwrap();
    let center = [5., 10., 15.];
    let radius = (10f64 * 10. + 20. * 20. + 30. * 30.).sqrt() * 0.5;
    camera.fit_bounds(center, radius, 4. / 3., 1.1).unwrap();
    for x in [0., 10.] {
        for y in [0., 20.] {
            for z in [0., 30.] {
                let projected = camera.project([x, y, z], viewport()).unwrap().unwrap();
                assert!(
                    (0.0..=800.).contains(&projected.screen[0])
                        && (0.0..=600.).contains(&projected.screen[1]),
                    "corner outside fit: {projected:?}"
                );
            }
        }
    }
}

#[test]
fn center_preserves_rotation_zoom_and_projection() {
    let mut camera = Camera::perspective(
        [35., -35., 35.],
        [1., 2., 3.],
        [0., 0., 1.],
        35f64.to_radians(),
    )
    .unwrap();
    let before = camera.clone();
    camera.center_at([0.; 3]).unwrap();
    assert_eq!(camera.target, [0.; 3]);
    assert_eq!(camera.projection, before.projection);
    let direction = |c: &Camera| {
        [
            c.eye[0] - c.target[0],
            c.eye[1] - c.target[1],
            c.eye[2] - c.target[2],
        ]
    };
    assert_eq!(direction(&camera), direction(&before));
}

#[test]
fn invalid_fit_inputs_and_zoom_factors_are_refused() {
    let mut camera = Camera::orthographic([0., 0., 100.], [0.; 3], [0., 1., 0.], 40.).unwrap();
    assert!(camera.fit_bounds([0.; 3], 0., 1., 1.1).is_err());
    assert!(camera.fit_bounds([0.; 3], 1., 0., 1.1).is_err());
    assert!(camera.fit_bounds([0.; 3], 1., 1., 0.9).is_err());
    for factor in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(camera.zoom(factor).is_err(), "factor {factor}");
    }
}
