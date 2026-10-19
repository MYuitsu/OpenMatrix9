// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix four-view defaults and per-view jewelry grid/axis rules (M01 L2-03-01).
use om9_scene::{
    camera::{CPlane, Projection},
    grid::Grid,
    workspace::{ViewId, Workspace},
};

#[test]
fn matrix_four_view_camera_presets_are_distinct_and_valid() {
    let workspace = Workspace::new();
    assert_eq!(workspace.views.len(), 4);
    assert_eq!(workspace.visible_views(), ViewId::ALL.to_vec());
    assert_eq!(workspace.active, ViewId::Perspective);

    let looking = &workspace.views[&ViewId::LookingDown];
    assert!(matches!(
        looking.camera.projection,
        Projection::Orthographic { .. }
    ));
    assert_eq!(looking.camera.eye, [0., 0., 100.]);
    assert_eq!(looking.camera.target, [0.; 3]);
    assert_eq!(looking.cplane, CPlane::world_xy());

    let perspective = &workspace.views[&ViewId::Perspective];
    assert!(matches!(
        perspective.camera.projection,
        Projection::Perspective { .. }
    ));
    assert_eq!(perspective.camera.eye, [35., -35., 35.]);
    assert_eq!(perspective.camera.target, [0.; 3]);

    let side = &workspace.views[&ViewId::SideView];
    assert!(matches!(
        side.camera.projection,
        Projection::Orthographic { .. }
    ));
    assert_eq!(side.camera.eye, [100., 0., 0.]);
    assert_eq!(side.cplane.x, [0., 1., 0.]);
    assert_eq!(side.cplane.y, [0., 0., 1.]);

    let finger = &workspace.views[&ViewId::ThroughFinger];
    assert!(matches!(
        finger.camera.projection,
        Projection::Orthographic { .. }
    ));
    assert_eq!(finger.camera.eye, [0., -100., 0.]);
    assert_eq!(finger.cplane.x, [1., 0., 0.]);
    assert_eq!(finger.cplane.y, [0., 0., 1.]);
}

#[test]
fn jewelry_grid_axes_follow_each_view_cplane() {
    let workspace = Workspace::new();
    for id in ViewId::ALL {
        let grid = Grid::new(workspace.views[&id].cplane).expect("grid");
        assert_eq!(grid.lines.len(), 82);
        let red = grid
            .lines
            .iter()
            .find(|line| line.axis && line.color == [210, 35, 35, 255])
            .expect("red X axis");
        let cyan = grid
            .lines
            .iter()
            .find(|line| line.axis && line.color == [0, 200, 220, 255])
            .expect("cyan Y axis");
        if matches!(id, ViewId::LookingDown | ViewId::Perspective) {
            assert_eq!(red.start, [-20., 0., 0.]);
            assert_eq!(red.end, [20., 0., 0.]);
            assert_eq!(cyan.start, [0., -20., 0.]);
            assert_eq!(cyan.end, [0., 20., 0.]);
        } else {
            let length = |line: &om9_scene::grid::GridLine| {
                line.end
                    .iter()
                    .zip(line.start)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            };
            assert!((length(red) - 40.).abs() < 1e-9);
            assert!((length(cyan) - 40.).abs() < 1e-9);
        }
        assert!(
            grid.lines
                .iter()
                .all(|line| line.start.iter().all(|v| v.is_finite()))
        );
    }
}

#[test]
fn maximize_is_per_view_and_restores_all_four_views() {
    let mut workspace = Workspace::new();
    workspace.toggle_maximize(ViewId::LookingDown).unwrap();
    assert_eq!(workspace.maximized, Some(ViewId::LookingDown));
    assert_eq!(workspace.active, ViewId::LookingDown);
    assert_eq!(workspace.visible_views(), vec![ViewId::LookingDown]);
    workspace.toggle_maximize(ViewId::LookingDown).unwrap();
    assert_eq!(workspace.maximized, None);
    assert_eq!(workspace.visible_views(), ViewId::ALL.to_vec());
    workspace.toggle_maximize(ViewId::SideView).unwrap();
    workspace.active = ViewId::ThroughFinger;
    assert_eq!(workspace.maximized, Some(ViewId::SideView));
    assert_eq!(workspace.visible_views(), vec![ViewId::SideView]);
}
