// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_scene::{camera::*, grid::*, workspace::*};
#[test]
fn visible_ray_starts_on_near_plane_for_off_axis_views() {
    let viewport = Viewport { origin: [0., 0.], size: [640., 480.] };
    for perspective in [false, true] {
        let mut camera = if perspective {
            Camera::perspective([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 0.8).unwrap()
        } else { Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap() };
        camera.near_mm = 17.;
        let ray = camera.visible_ray([500., 140.], viewport).unwrap();
        assert!((ray.origin[2] - 83.).abs() < 1e-10, "{ray:?}");
        let original = camera.ray([500., 140.], viewport).unwrap();
        assert_eq!(ray.direction, original.direction);
    }
}
#[test]
fn orthographic_projection_and_ray_share_world_frame_after_pan() {
    let mut camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    let viewport = Viewport {
        origin: [0., 0.],
        size: [640., 480.],
    };
    let p = camera.project([10., 0., 0.], viewport).unwrap().unwrap();
    assert_eq!(p.screen, [440., 240.]);
    let ray = camera.ray(p.screen, viewport).unwrap();
    assert_eq!(
        ray.intersect_plane(CPlane::world_xy()).unwrap().unwrap(),
        [10., 0., 0.]
    );
    camera.pan([10., 0.]).unwrap();
    assert_eq!(
        camera
            .project([10., 0., 0.], viewport)
            .unwrap()
            .unwrap()
            .screen,
        [320., 240.]
    );
    assert_eq!(
        camera
            .ray([320., 240.], viewport)
            .unwrap()
            .intersect_plane(CPlane::world_xy())
            .unwrap()
            .unwrap(),
        [10., 0., 0.]
    );
}
#[test]
fn finite_grid_is_world_geometry_for_display_only_with_exact_subdivision() {
    let grid = Grid::new(CPlane::world_xy()).unwrap();
    assert_eq!(grid.lines.len(), 82);
    assert!(
        grid.lines
            .iter()
            .all(|l| l.start.iter().chain(&l.end).all(|v| v.abs() <= 20.))
    );
    assert_eq!(grid.lines.iter().filter(|l| l.major).count(), 18);
    assert_eq!(grid.lines.iter().filter(|l| l.axis).count(), 2);
    assert_eq!(grid.lines[0].start, [-20., -20., 0.]);
    assert_eq!(grid.lines[0].end, [-20., 20., 0.]);
    let camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    let viewport = Viewport {
        origin: [0., 0.],
        size: [400., 400.],
    };
    assert_eq!(
        camera
            .project(grid.lines[0].start, viewport)
            .unwrap()
            .unwrap()
            .screen,
        [0., 400.]
    );
}

#[test]
fn side_view_grid_uses_blue_horizontal_y_and_yellow_vertical_z() {
    let workspace = Workspace::new();
    let view = &workspace.views[&ViewId::SideView];
    let grid = Grid::new(view.cplane).unwrap();
    let horizontal = grid.lines.iter().find(|line| line.axis && line.start[1] != line.end[1]).unwrap();
    let vertical = grid.lines.iter().find(|line| line.axis && line.start[2] != line.end[2]).unwrap();
    assert_eq!(horizontal.color, [0, 200, 220, 255], "world Y must use the blue grid axis");
    assert_eq!(vertical.color, [235, 180, 50, 255], "world Z must use the yellow axis");
    let viewport = Viewport { origin: [0., 0.], size: [400., 400.] };
    let a = view.camera.project(horizontal.start, viewport).unwrap().unwrap().screen;
    let b = view.camera.project(horizontal.end, viewport).unwrap().unwrap().screen;
    assert!((a[1] - b[1]).abs() < 1e-9 && (a[0] - b[0]).abs() > 100.);
    let a = view.camera.project(vertical.start, viewport).unwrap().unwrap().screen;
    let b = view.camera.project(vertical.end, viewport).unwrap().unwrap().screen;
    assert!((a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() > 100.);
    for id in [ViewId::LookingDown, ViewId::Perspective, ViewId::ThroughFinger] {
        let grid = Grid::new(workspace.views[&id].cplane).unwrap();
        let axes: Vec<_> = grid.lines.iter().filter(|line| line.axis).map(|line| line.color).collect();
        assert_eq!(axes, [[0, 200, 220, 255], [210, 35, 35, 255]], "other views keep their existing grid colours");
    }
}
#[test]
fn cplane_coordinate_resolution_is_independent_from_camera_navigation() {
    let plane = CPlane::new([10., 20., 30.], [0., 1., 0.], [-1., 0., 0.]).unwrap();
    assert_eq!(plane.point([2., 3., 0.]).unwrap(), [7., 22., 30.]);
    let mut view = ViewState::new(
        ViewId::LookingDown,
        Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap(),
        plane,
    );
    view.camera.pan([50., 20.]).unwrap();
    assert_eq!(view.cplane.point([2., 3., 0.]).unwrap(), [7., 22., 30.]);
}
#[test]
fn maximize_restore_preserves_each_view_and_original_titles() {
    let mut workspace = Workspace::new();
    let before = workspace.views.clone();
    workspace.toggle_maximize(ViewId::ThroughFinger).unwrap();
    assert_eq!(workspace.visible_views(), vec![ViewId::ThroughFinger]);
    workspace.toggle_maximize(ViewId::ThroughFinger).unwrap();
    assert_eq!(workspace.views, before);
    assert_eq!(workspace.visible_views().len(), 4);
    assert_eq!(ViewId::LookingDown.title(), "Looking Down");
    assert_eq!(ViewId::SideView.title(), "Side View");
    assert_eq!(ViewId::ThroughFinger.title(), "Through Finger");
}
#[test]
fn invalid_camera_frame_clipping_and_viewport_ranges_reject() {
    assert!(Camera::orthographic([0., 0., 0.], [0., 0., 0.], [0., 1., 0.], 40.).is_err());
    assert!(Camera::orthographic([0., 0., 1.], [0., 0., 0.], [0., 0., 1.], 40.).is_err());
    let c = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    assert!(
        c.ray(
            [0., 0.],
            Viewport {
                origin: [0., 0.],
                size: [0., 480.]
            }
        )
        .is_err()
    );
    assert!(
        c.project(
            [0., 0., 200.],
            Viewport {
                origin: [0., 0.],
                size: [640., 480.]
            }
        )
        .unwrap()
        .is_none()
    );
}
