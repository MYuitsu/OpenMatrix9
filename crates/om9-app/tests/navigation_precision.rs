// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::{camera_rig::CameraRig, navigation::ZoomResponse};
use om9_scene::camera::{Camera, Projection, Viewport};
use std::time::{Duration, Instant};

#[test]
fn deep_zoom_anchor_stays_within_a_tenth_physical_pixel_at_large_coordinates() {
    for offset in [0., 1e9] {
        for orthographic in [false, true] {
            let mut camera = Camera::perspective(
                [offset, offset - 100., offset + 20.],
                [offset; 3],
                [0., 0., 1.],
                0.6,
            )
            .unwrap();
            if orthographic {
                camera.projection = Projection::Orthographic {
                    vertical_span_mm: 100.,
                };
            }
            let rig = CameraRig::from_camera(&camera).unwrap();
            let right = rig.orientation.rotate([1., 0., 0.]);
            let anchor = std::array::from_fn(|a| rig.pivot[a] + right[a] * 2.);
            let vp = Viewport {
                origin: [10., 20.],
                size: [800., 600.],
            };
            let before = camera.project(anchor, vp).unwrap().unwrap().screen;
            let now = Instant::now();
            let mut zoom = ZoomResponse::new(rig, anchor, now, 1.);
            let initial = rig.ortho_span.unwrap_or(rig.distance);
            zoom.add_points(40. * (0.2 / initial).ln() / 0.9_f64.ln(), now);
            for ms in 0..=180 {
                let (pose, _) = zoom.sample(now + Duration::from_millis(ms));
                let sampled = pose.to_camera(&camera).unwrap();
                let projected = sampled.project(anchor, vp).unwrap().unwrap().screen;
                for axis in 0..2 {
                    assert!(
                        (projected[axis] - before[axis]).abs() * 1.5 <= 0.1,
                        "offset={offset},ortho={orthographic},ms={ms},drift={:?}",
                        [projected[0] - before[0], projected[1] - before[1]]
                    );
                }
            }
        }
    }
}

#[test]
fn pole_crossing_input_cannot_flip_or_invalidate_the_camera() {
    let camera = Camera::perspective([0., -100., 20.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    let mut rig = CameraRig::from_camera(&camera).unwrap();
    let distance = rig.distance;
    for (yaw, pitch) in [(0., 100.), (2., -100.), (-100., 0.7), (100., -0.7)] {
        rig.orbit_turntable(yaw, pitch);
        let pose = rig.to_camera(&camera).unwrap();
        pose.validate().unwrap();
        let forward = rig.orientation.rotate([0., 0., -1.]);
        assert!(forward[2].abs() < 1.);
        assert_eq!(rig.distance, distance);
        assert!(pose.eye.into_iter().chain(pose.up).all(f64::is_finite));
    }
}
#[test]
fn restoring_a_different_pose_with_the_same_revision_cancels_the_old_zoom() {
    use om9_app::navigation::NavigationController;
    use om9_scene::workspace::ViewId;
    let now = Instant::now();
    let camera = Camera::perspective([0., -100., 20.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    let vp = Viewport {
        origin: [0.; 2],
        size: [800., 600.],
    };
    let mut nav = NavigationController::default();
    nav.wheel(
        ViewId::Perspective,
        &camera,
        vp,
        [400., 300.],
        252.,
        now,
        1.,
    );
    let restored = Camera::perspective([100., 0., 20.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    assert_eq!(restored.revision, camera.revision);
    nav.sync(
        ViewId::Perspective,
        &restored,
        now + Duration::from_millis(10),
    );
    assert!(
        nav.tick(
            ViewId::Perspective,
            &restored,
            now + Duration::from_millis(50)
        )
        .is_none()
    );
}
