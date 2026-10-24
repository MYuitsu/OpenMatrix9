// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L2-03-02-005/006`: zoom extents/selected and view reset commands.
use om9_app::session::{Controller, VIEW_CENTER, VIEW_HOME, ZOOM_EXTENTS, ZOOM_SELECTED};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_scene::camera::{Projection, Viewport};
use om9_scene::workspace::{ViewId, Workspace};

fn line(start: [f64; 3], end: [f64; 3]) -> Entity {
    Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start,
            end,
            domain: [0., 1.],
        }),
        None,
    )
}

fn document_with_lines() -> (
    Document,
    om9_core::identity::ObjectId,
    om9_core::identity::ObjectId,
) {
    let mut document = Document::new("navigation");
    let near = line([0., 0., 0.], [10., 20., 30.]);
    let near_id = near.id;
    document.entities.insert(near_id, near);
    let far = line([100., 0., 0.], [110., 20., 30.]);
    let far_id = far.id;
    document.entities.insert(far_id, far);
    (document, near_id, far_id)
}

fn viewport() -> Viewport {
    Viewport {
        origin: [0., 0.],
        size: [800., 600.],
    }
}

#[test]
fn zoom_extents_fits_the_whole_scene_and_keeps_the_view_direction() {
    let (document, _, _) = document_with_lines();
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    let before = controller.workspace.views[&ViewId::Perspective]
        .camera
        .clone();
    controller.zoom_extents(4. / 3.).unwrap();
    let camera = &controller.workspace.views[&ViewId::Perspective].camera;
    assert!((camera.target[0] - 55.).abs() < 1e-9);
    assert!((camera.target[1] - 10.).abs() < 1e-9);
    assert!((camera.target[2] - 15.).abs() < 1e-9);
    let direction = |c: &om9_scene::camera::Camera| {
        [
            c.eye[0] - c.target[0],
            c.eye[1] - c.target[1],
            c.eye[2] - c.target[2],
        ]
    };
    let old = direction(&before);
    let new = direction(camera);
    let cross = [
        old[1] * new[2] - old[2] * new[1],
        old[2] * new[0] - old[0] * new[2],
        old[0] * new[1] - old[1] * new[0],
    ];
    assert!(cross.iter().all(|v| v.abs() < 1e-9), "direction preserved");
    let endpoints = [[0., 0., 0.], [110., 20., 30.]];
    for point in endpoints {
        let projected = camera.project(point, viewport()).unwrap().unwrap();
        assert!(
            (0.0..=800.).contains(&projected.screen[0])
                && (0.0..=600.).contains(&projected.screen[1]),
            "endpoint {point:?} outside fit: {:?}",
            projected.screen
        );
    }
}

#[test]
fn zoom_selected_ignores_far_unselected_geometry() {
    let (document, near, _) = document_with_lines();
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[near]).unwrap();
    controller.zoom_selected(4. / 3.).unwrap();
    let target = controller.workspace.views[&ViewId::Perspective]
        .camera
        .target;
    assert!((target[0] - 5.).abs() < 1e-9, "{target:?}");
    assert!((target[1] - 10.).abs() < 1e-9, "{target:?}");
    assert!((target[2] - 15.).abs() < 1e-9, "{target:?}");
}

#[test]
fn empty_scene_is_a_typed_fit_failure_and_a_point_keeps_a_finite_span() {
    let mut empty = Controller::new(Document::new("empty"), Limits::default()).unwrap();
    assert_eq!(
        empty.zoom_extents(1.0),
        Err(om9_core::Error::Missing("fit bounds"))
    );

    let mut document = Document::new("point");
    let point = Entity::new("point", Geometry::Point([2., 3., 4.]), None);
    let id = point.id;
    document.entities.insert(id, point);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.zoom_extents(1.0).unwrap();
    let camera = &controller.workspace.views[&ViewId::Perspective].camera;
    assert!(
        camera
            .eye
            .iter()
            .chain(&camera.target)
            .all(|v| v.is_finite())
    );
    let Projection::Perspective { .. } = camera.projection else {
        panic!("perspective preset");
    };
}

#[test]
fn home_restores_the_camera_preset_without_touching_cplane_layers_or_document() {
    let mut controller = Controller::new(Document::new("home"), Limits::default()).unwrap();
    let active = ViewId::ThroughFinger;
    controller.workspace.active = active;
    let cplane = om9_scene::camera::CPlane::world_xy();
    controller.set_cplane(cplane).unwrap();
    let layer_state = controller.document().layers.clone();
    controller
        .workspace
        .views
        .get_mut(&active)
        .unwrap()
        .camera
        .orbit(0.4, 0.2)
        .unwrap();
    controller.view_home().unwrap();
    assert_eq!(
        controller.workspace.views[&active].camera,
        Workspace::default_camera(active)
    );
    assert_eq!(controller.input_plane().origin, cplane.origin);
    assert_eq!(controller.document().layers, layer_state);
    assert_eq!(controller.document().entities.len(), 0);
}

#[test]
fn center_moves_the_target_to_the_world_origin_and_preserves_rotation() {
    let mut controller = Controller::new(Document::new("center"), Limits::default()).unwrap();
    let view = controller
        .workspace
        .views
        .get_mut(&ViewId::Perspective)
        .unwrap();
    view.camera.pan([15., -7.]).unwrap();
    let before = view.camera.clone();
    controller.view_center().unwrap();
    let after = &controller.workspace.views[&ViewId::Perspective].camera;
    assert_eq!(after.target, [0.; 3]);
    let direction = |c: &om9_scene::camera::Camera| {
        [
            c.eye[0] - c.target[0],
            c.eye[1] - c.target[1],
            c.eye[2] - c.target[2],
        ]
    };
    assert_eq!(direction(after), direction(&before));
}

#[test]
fn view_commands_are_available_and_dispatch_through_the_session() {
    for (alias, canonical) in [
        ("ZoomExtents", ZOOM_EXTENTS),
        ("ZE", ZOOM_EXTENTS),
        ("ZoomSelected", ZOOM_SELECTED),
        ("Home", VIEW_HOME),
        ("Center", VIEW_CENTER),
    ] {
        assert_eq!(Controller::resolve_alias(alias), Some(canonical));
    }
    let (document, _, _) = document_with_lines();
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.submit("ZoomExtents").unwrap();
    assert!(!controller.is_running());
    controller.submit("Home").unwrap();
}
