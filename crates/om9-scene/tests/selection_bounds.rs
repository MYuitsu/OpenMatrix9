use om9_scene::{
    camera::{Camera, Projection, Viewport},
    scene::{RenderBuffers, SceneBuffers},
};
use std::sync::Arc;

#[test]
fn bounds_are_shared_immutable_and_replaced_with_the_payload() {
    let mut buffers = SceneBuffers::transient(
        RenderBuffers::Points {
            points: vec![[-1., -2., -3.], [4., 5., 6.]],
            native_indices: vec![0, 1],
        },
        Arc::new(()),
    );
    // Registered payloads are deliberately immutable; no stale bounds can be
    // installed through the escape hatch used by unregistered test fixtures.
    let other = buffers.clone();
    assert!(buffers.ptr_eq(&other));
    assert_eq!(
        buffers.local_bounds(),
        Some(([-1., -2., -3.], [4., 5., 6.]))
    );
    assert!(buffers.make_mut_unregistered().is_err());
    let replacement = SceneBuffers::transient(
        RenderBuffers::Points {
            points: vec![[20., 0., 0.]],
            native_indices: vec![0],
        },
        Arc::new(()),
    );
    assert_eq!(
        replacement.local_bounds(),
        Some(([20., 0., 0.], [20., 0., 0.]))
    );
    assert_ne!(replacement.local_bounds(), other.local_bounds());
}
#[test]
fn projected_box_encloses_display_points_and_clips_the_near_plane() {
    let camera = Camera {
        eye: [0., 0., 0.],
        target: [0., 0., 1.],
        up: [0., 1., 0.],
        projection: Projection::Perspective {
            vertical_fov_rad: 1.,
        },
        near_mm: 0.1,
        far_mm: 100.,
        revision: 1,
    };
    let viewport = Viewport {
        origin: [0., 0.],
        size: [800., 600.],
    };
    let projector = camera.projector(viewport).unwrap();
    for depth in [2., 0.05, -5.] {
        let corners = std::array::from_fn(|i| {
            [
                if i & 1 == 0 { -1. } else { 1. },
                if i & 2 == 0 { -1. } else { 1. },
                depth + if i & 4 == 0 { 0. } else { 1. },
            ]
        });
        let projected = projector.project_box(corners).unwrap();
        if depth < 0. {
            assert!(projected.is_none());
            continue;
        }
        let [min, max] = projected.unwrap();
        for ix in 0..=10 {
            for iy in 0..=10 {
                for iz in 0..=10 {
                    let p = [
                        -1. + ix as f64 / 5.,
                        -1. + iy as f64 / 5.,
                        depth + iz as f64 / 10.,
                    ];
                    if let Some(q) = projector.project(p).unwrap() {
                        for axis in 0..2 {
                            assert!(
                                q.screen[axis] >= min[axis] - 1e-6
                                    && q.screen[axis] <= max[axis] + 1e-6
                            );
                        }
                    }
                }
            }
        }
    }
}
