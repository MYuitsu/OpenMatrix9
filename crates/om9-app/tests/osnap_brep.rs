// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-03-004`: surface snap on a trimmed BRep display face.
#![cfg(all(windows, feature = "occt-brep"))]

use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::camera::{Camera, Viewport};
use om9_scene::osnap::{SnapKind, SnapSettings, candidates};

#[test]
fn a_ray_through_a_brep_face_returns_a_surface_snap_on_the_trimmed_face() {
    let mut document = Document::new("brep osnap");
    let entity = Entity::new(
        "box",
        Geometry::Brep(
            om9_kernel::brep::box_brep([10., 10., 10.], 0.02, &Limits::default()).unwrap(),
        ),
        None,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    let controller = Controller::new(document, Limits::default()).unwrap();
    let camera = Camera::orthographic([0., 0., 100.], [0.; 3], [0., 1., 0.], 40.).unwrap();
    let viewport = Viewport {
        origin: [0., 0.],
        size: [400., 400.],
    };
    let cursor = camera
        .project([0., 0., 10.], viewport)
        .unwrap()
        .unwrap()
        .screen;
    let settings = SnapSettings {
        radius_px: 8.,
        ..SnapSettings::default()
    };
    let result = candidates(
        controller.scene().unwrap(),
        &camera,
        viewport,
        cursor,
        settings,
        None,
    )
    .unwrap();
    let surface = result
        .iter()
        .find(|candidate| candidate.kind == SnapKind::Surface)
        .expect("surface snap");
    assert_eq!(surface.root, Some(id));
    assert!((surface.world[2] - 10.).abs() < 0.05);
    let normal = surface.normal.expect("face normal");
    assert!(normal[2].abs() > 0.99);

    let outside = candidates(
        controller.scene().unwrap(),
        &camera,
        viewport,
        camera
            .project([50., 0., 10.], viewport)
            .unwrap()
            .unwrap()
            .screen,
        settings,
        None,
    )
    .unwrap();
    assert!(
        !outside
            .iter()
            .any(|candidate| candidate.kind == SnapKind::Surface),
        "a ray outside the trimmed face must not snap"
    );
}
