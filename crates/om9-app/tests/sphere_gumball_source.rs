// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

use om9_app::session::{Controller, StlExportProfile};
use om9_core::{budget::Limits, geometry::{Geometry, Transform}, gumball::{Drag, Handle, Session}, model::Document};
use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind, Subelement}};

#[test]
fn sphere_command_gumball_translation_and_history_keep_source_and_scene_in_step() {
    let limits = Limits::default();
    let mut controller = Controller::new(Document::with_template_tolerance("sphere translation"), limits).unwrap();
    controller.submit("Sphere 5").unwrap();
    let id = *controller.document().entities.keys().next().unwrap();
    controller.select_objects(&[id]).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let target = Cleanup(std::env::temp_dir().join(format!("om9-sphere-move-{}.stl", uuid::Uuid::new_v4())));
    let profile = StlExportProfile { linear_tolerance_mm: 0.05, angular_tolerance_rad: 0.3 };
    controller.export_stl_selected_with_profile(&target.0, profile).unwrap();
    let original_stl = std::fs::read(&target.0).unwrap();
    let original = controller.document().entities[&id].clone();
    let Geometry::Brep(shape) = &original.geometry else { panic!("sphere BRep") };
    let source = shape.source().to_owned();
    let topology = shape.topology().clone();
    let display = shape.display_identity();
    let face = topology.faces[0];
    let camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 30.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let pick = |controller: &Controller, point: [f64; 3]| {
        let screen = camera.project(point, viewport).unwrap().unwrap().screen;
        controller.scene().unwrap().pick_kind(screen, &camera, viewport, 0.1, PickIntent::Select, PickKind::Face).unwrap()
    };
    let before_move = pick(&controller, [0., 0., 5.]).unwrap();
    assert_eq!(before_move.root, id);
    assert_eq!(before_move.subset, Subelement::BrepFace(face));
    let mut gesture = Session::begin(controller.gumball_pivot().unwrap(), controller.gumball_targets(), false).unwrap();
    gesture.drag(Handle::MoveFree, Drag::WorldDelta([7., -2., 3.])).unwrap();
    controller.commit_gumball(&gesture, "Move").unwrap();
    controller.export_stl_selected_with_profile(&target.0, profile).unwrap();
    let moved_stl = std::fs::read(&target.0).unwrap();
    assert_ne!(moved_stl, original_stl);
    assert!(before_move.revalidate_in_scene(controller.coordinator(), controller.scene(), &camera, viewport).is_err());
    assert!(pick(&controller, [0., 0., 5.]).is_none());
    let after_move = pick(&controller, [7., -2., 8.]).unwrap();
    assert_eq!(after_move.root, id);
    assert_eq!(after_move.subset, Subelement::BrepFace(face));
    assert!(after_move.revalidate_in_scene(controller.coordinator(), controller.scene(), &camera, viewport).is_ok());
    let moved_screen = camera.project([7., -2., 8.], viewport).unwrap().unwrap().screen;
    let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, moved_screen, Default::default(), None).unwrap();
    let candidate = candidates.iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface).unwrap();
    let refined = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
    let radius = ((refined.world[0] - 7.).powi(2) + (refined.world[1] + 2.).powi(2)
        + (refined.world[2] - 3.).powi(2)).sqrt();
    assert!((radius - 5.).abs() < 1e-6, "{refined:?}");

    let check = |controller: &Controller, expected: [f64; 3]| {
        let entity = &controller.document().entities[&id];
        let Geometry::Brep(brep) = &entity.geometry else { panic!("sphere BRep") };
        assert_eq!(brep.source(), source);
        assert_eq!(brep.topology(), &topology);
        assert!(display.same(&brep.display_identity()));
        assert_eq!(entity.placement, Transform::translation(expected));
        let bounds = om9_kernel::brep::world_bounds(brep, &entity.placement, &limits).unwrap();
        for axis in 0..3 {
            assert!((bounds[axis] - (expected[axis] - 5.)).abs() < 1e-6, "{bounds:?}");
            assert!((bounds[axis + 3] - (expected[axis] + 5.)).abs() < 1e-6, "{bounds:?}");
        }
        let scene = controller.scene().unwrap();
        let object = scene.objects.iter().find(|object| object.root == id).unwrap();
        assert_eq!(object.display_transform().unwrap(), entity.placement);
    };
    check(&controller, [7., -2., 3.]);
    controller.undo().unwrap();
    assert!(after_move.revalidate_in_scene(controller.coordinator(), controller.scene(), &camera, viewport).is_err());
    assert!(controller.refine_surface_snap(candidate, &camera, viewport).is_err());
    assert_eq!(pick(&controller, [0., 0., 5.]).unwrap().subset, Subelement::BrepFace(face));
    controller.select_objects(&[id]).unwrap();
    controller.export_stl_selected_with_profile(&target.0, profile).unwrap();
    assert_eq!(std::fs::read(&target.0).unwrap(), original_stl);
    check(&controller, [0., 0., 0.]);
    controller.redo().unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.export_stl_selected_with_profile(&target.0, profile).unwrap();
    assert_eq!(std::fs::read(&target.0).unwrap(), moved_stl);
    check(&controller, [7., -2., 3.]);
    controller.undo().unwrap();
    controller.undo().unwrap();
    assert!(!controller.document().entities.contains_key(&id));
    controller.redo().unwrap();
    check(&controller, [0., 0., 0.]);
    controller.redo().unwrap();
    check(&controller, [7., -2., 3.]);
}

#[test]
fn v4_sphere_with_unattested_face_id_keeps_display_when_remesh_is_requested() {
    let limits = Limits::default();
    let mut creator = Controller::new(Document::with_template_tolerance("v4 sphere"), limits).unwrap();
    creator.submit("Sphere 5").unwrap();
    let id = *creator.document().entities.keys().next().unwrap();
    let mut document = creator.document().clone();
    let entity = document.entities.get_mut(&id).unwrap();
    let Geometry::Brep(original) = &entity.geometry else { panic!("sphere BRep") };
    let original_face = original.topology().faces[0];
    let mut display = original.display().clone();
    let unattested_face = uuid::Uuid::new_v4();
    display.topology.faces[0] = unattested_face;
    let unverified = om9_core::brep::Brep::from_native(original.source().to_owned(), display, &limits).unwrap();
    entity.geometry = Geometry::Brep(unverified);
    let bytes = om9_archive::Archive::encode(&document, &limits).unwrap();
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 4);
    let decoded = om9_archive::Archive::decode(&bytes, &limits).unwrap();
    let mut controller = Controller::new(decoded, limits).unwrap();
    let before = controller.document().clone();
    let Geometry::Brep(brep) = &before.entities[&id].geometry else { panic!("sphere BRep") };
    assert_ne!(unattested_face, original_face);
    assert_eq!(brep.topology().faces[0], unattested_face);
    let display = brep.display_identity();
    let publication = controller.scene().unwrap().publication_id();
    let undo = controller.coordinator().undo_len();
    let redo = controller.coordinator().redo_len();
    assert!(matches!(controller.request_topology_remesh_for(id, 0.02, 0.3,
        16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024),
        Err(om9_core::Error::Missing("topology proof"))));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.scene().unwrap().publication_id(), publication);
    assert_eq!(controller.coordinator().undo_len(), undo);
    assert_eq!(controller.coordinator().redo_len(), redo);
    let Geometry::Brep(current) = &controller.document().entities[&id].geometry else { panic!("sphere BRep") };
    assert!(display.same(&current.display_identity()), "v4 document must retain the same display allocation");
    let camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 20.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let cursor = camera.project([0., 0., 5.], viewport).unwrap().unwrap().screen;
    let hit = controller.scene().unwrap().pick_kind(cursor, &camera, viewport, 0.1, PickIntent::Select, PickKind::Face).unwrap().unwrap();
    assert_eq!(hit.root, id);
    assert_eq!(hit.subset, Subelement::BrepFace(unattested_face));
}
