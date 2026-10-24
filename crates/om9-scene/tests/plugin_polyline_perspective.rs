// SPDX-License-Identifier: LGPL-2.1-or-later
//! Perspective picking of exact SDK line polylines must return a real point on
//! the segment, including when its first endpoint is outside the near plane.

use om9_core::{
    budget::Limits,
    curve_build::polyline,
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, Scene, Subelement},
};
use std::sync::Arc;

fn assert_segment_pick(points: [[f64; 3]; 3], expected: [f64; 3], near: f64) {
    let mut document = Document::new("Perspective SDK polyline");
    let entity = Entity::new(
        "SDK polyline",
        Geometry::Curve(polyline(&points).unwrap()),
        document.current_layer,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    let scene = Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
    let mut camera = Camera::perspective(
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
        std::f64::consts::FRAC_PI_2,
    )
    .unwrap();
    camera.near_mm = near;
    camera.far_mm = 100.0;
    camera.validate().unwrap();
    let viewport = Viewport {
        origin: [0.0; 2],
        size: [600.0, 600.0],
    };
    if points[0][2] < near {
        assert!(camera.project(points[0], viewport).unwrap().is_none());
    }
    let screen = camera.project(expected, viewport).unwrap().unwrap().screen;
    for intent in [
        PickIntent::Select,
        PickIntent::Reference { snap_locked: false },
    ] {
        let hit = scene
            .pick(screen, &camera, viewport, 0.1, intent)
            .unwrap()
            .unwrap();
        assert_eq!(hit.root, id);
        assert!(matches!(hit.subset, Subelement::Whole));
        for axis in 0..3 {
            assert!(
                (hit.world_point[axis] - expected[axis]).abs() < 1e-10,
                "perspective pick {:?} must equal analytic segment point {expected:?}",
                hit.world_point,
            );
        }
    }
}

#[test]
fn unequal_endpoint_depths_preserve_analytic_segment_point() {
    // Exactly one quarter of the way from A=(-2,0,2) to B=(4,0,10).
    // Linear screen interpolation without depth correction gives a different
    // world point, even though the projected polyline is a straight segment.
    assert_segment_pick(
        [[-2.0, 0.0, 2.0], [4.0, 0.0, 10.0], [4.0, 4.0, 10.0]],
        [-0.5, 0.0, 4.0],
        0.01,
    );
}

#[test]
fn near_plane_clipped_segment_preserves_analytic_world_point() {
    // A is in front of the eye but outside the near plane. The expected point
    // is the original segment's exact midpoint, still inside the visible part.
    assert_segment_pick(
        [[-1.0, 0.0, 0.25], [2.0, 0.0, 4.0], [2.0, 2.0, 4.0]],
        [0.5, 0.0, 2.125],
        1.0,
    );
}
