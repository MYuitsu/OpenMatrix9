// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_scene::{
    camera::Viewport,
    workspace::{ViewId, Workspace},
};
#[test]
fn through_finger_reference_projects_world_xz_and_side_reference_projects_world_yz() {
    let w = Workspace::new();
    let viewport = Viewport {
        origin: [0.; 2],
        size: [800., 600.],
    };
    for (id, p) in [
        (ViewId::ThroughFinger, [7., 0., 5.]),
        (ViewId::SideView, [0., 7., 5.]),
    ] {
        let v = &w.views[&id];
        let projected = v.camera.project(p, viewport).unwrap().unwrap();
        let picked = v
            .camera
            .ray(projected.screen, viewport)
            .unwrap()
            .intersect_plane(v.cplane)
            .unwrap()
            .unwrap();
        for i in 0..3 {
            assert!((picked[i] - p[i]).abs() < 1e-10);
        }
    }
}
