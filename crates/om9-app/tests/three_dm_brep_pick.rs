// SPDX-License-Identifier: LGPL-2.1-or-later
//! Face/edge picking smoke for a document produced by the editable 3DM import.
#![cfg(all(windows, feature = "occt-brep"))]

use om9_app::session::{Controller, SelectedSubobject};
use om9_core::{
    budget::Limits,
    geometry::Geometry,
};use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind},
};

#[test]
#[ignore = "requires OM9_THREE_DM_OM9 from an editable Rhino 5 import"]
fn imported_brep_faces_and_edges_are_pickable() {
    let path = std::env::var_os("OM9_THREE_DM_OM9").expect("OM9_THREE_DM_OM9");
    let mut controller =
        Controller::from_file(std::path::Path::new(&path), Limits::default()).unwrap();
    let (entity, placement, display) = controller
        .document()
        .entities
        .values()
        .find_map(|entity| {
            let Geometry::Brep(brep) = &entity.geometry else {
                return None;
            };
            (!brep.display.points.is_empty() && !brep.display.triangles.is_empty()).then(|| {
                (
                    entity.clone(),
                    entity.placement,
                    brep.display.clone(),
                )
            })
        })
        .expect("editable Brep");
    let center = display.points.iter().fold([0.; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    let n = display.points.len() as f64;
    let center = [center[0] / n, center[1] / n, center[2] / n];
    let world_center = placement.point(center).unwrap();
    let camera = Camera::orthographic(
        [world_center[0], world_center[1], world_center[2] + 1000.],
        world_center,
        [0., 1., 0.],
        500.,
    )
    .unwrap();
    let viewport = Viewport {
        origin: [0.; 2],
        size: [1024., 768.],
    };
    let face_triangle = display.triangles[0];
    let face_center = [0, 1, 2].map(|i| display.points[face_triangle[i] as usize]);
    let face_center = [
        (face_center[0][0] + face_center[1][0] + face_center[2][0]) / 3.,
        (face_center[0][1] + face_center[1][1] + face_center[2][1]) / 3.,
        (face_center[0][2] + face_center[1][2] + face_center[2][2]) / 3.,
    ];
    let face_world = placement.point(face_center).unwrap();
    let face_screen = camera
        .project(face_world, viewport)
        .unwrap()
        .expect("face in front")
        .screen;
    let face_hit = controller
        .scene()
        .unwrap()
        .pick_kind(
            face_screen,
            &camera,
            viewport,
            0.2,
            PickIntent::Select,
            PickKind::Face,
        )
        .unwrap()
        .expect("face hit");
    assert_eq!(face_hit.root, entity.id);
    controller
        .select_subobject_ref(SelectedSubobject::from_hit(&face_hit))
        .unwrap();

    let segment = display.segments[0];
    let a = placement
        .point(display.points[segment[0] as usize])
        .unwrap();
    let b = placement
        .point(display.points[segment[1] as usize])
        .unwrap();
    let edge_world = [(a[0] + b[0]) / 2., (a[1] + b[1]) / 2., (a[2] + b[2]) / 2.];
    let edge_screen = camera
        .project(edge_world, viewport)
        .unwrap()
        .expect("edge in front")
        .screen;
    let edge_hit = controller
        .scene()
        .unwrap()
        .pick_kind(
            edge_screen,
            &camera,
            viewport,
            0.2,
            PickIntent::Select,
            PickKind::Edge,
        )
        .unwrap()
        .expect("edge hit");
    assert_eq!(edge_hit.root, entity.id);
    controller
        .select_subobject_ref(SelectedSubobject::from_hit(&edge_hit))
        .unwrap();
    assert!(!controller.subselection().is_empty());
}

