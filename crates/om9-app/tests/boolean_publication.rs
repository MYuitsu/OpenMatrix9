#![cfg(feature = "occt-brep")]

use om9_app::{display::ViewportDetail, session::InvocationOrigin, ui::MatrixApp};
use om9_core::{Error, budget::memory::MemoryClass, geometry::Geometry, model::Document};
use std::time::{Duration, Instant};

fn fixture(detail: ViewportDetail) -> MatrixApp {
    let mut app = MatrixApp::new(Document::with_template_tolerance("prepared Boolean publication")).unwrap();
    app.controller.request_viewport_detail(detail).unwrap();
    for x in [0., 5., 20., 25.] { app.controller.solid_sphere_points([x, 0., 0.], 5.).unwrap(); }
    let ids = app.controller.document().entities.keys().copied().collect::<Vec<_>>();
    app.controller.select_objects(&ids).unwrap();
    app
}

fn start(app: &mut MatrixApp, menu: bool) {
    if menu { app.controller.dispatch("OM9-SOLID-003", InvocationOrigin::Menu).unwrap(); }
    else { app.controller.submit("BooleanUnion").unwrap(); }
    app.controller.submit("DeleteInput=Yes").unwrap();
    app.controller.submit("").unwrap();
    assert!(app.controller.boolean_union_worker_pending());
    assert_eq!(app.controller.boolean_union_request_capture(), Some((true, 4)));
}

fn finish(app: &mut MatrixApp) -> om9_core::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(45);
    let before = app.controller.document().clone();
    let publication = app.controller.scene().unwrap().publication_id();
    loop {
        if let Some(result) = app.controller.poll_boolean_union_worker() { return result; }
        assert_eq!(app.controller.document(), &before, "partial document publication");
        assert_eq!(app.controller.scene().unwrap().publication_id(), publication, "partial scene publication");
        assert!(Instant::now() < deadline, "Boolean worker timeout");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn prepared_union_keeps_source_wires_snap_and_atomic_undo_redo_in_both_profiles() {
    for detail in [ViewportDetail::Light, ViewportDetail::High] {
        let mut app = fixture(detail);
        let before = app.controller.document().clone();
        let history = app.controller.coordinator().undo_len();
        start(&mut app, detail == ViewportDetail::High);
        finish(&mut app).unwrap();
        assert_eq!(app.controller.coordinator().undo_len(), history + 1);
        assert_eq!(app.controller.document().entities.len(), 1);
        let result = app.controller.document().clone();
        let Geometry::Brep(body) = &result.entities.values().next().unwrap().geometry else { panic!("BRep result") };
        let measured = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        assert!((measured.volume_mm3 - 1125. * std::f64::consts::PI / 2.).abs() < 1e-5);
        assert!(measured.closed && measured.solids == 2 && measured.naked_edges == 0);
        assert_eq!(measured.bounds_mm, [-5., -5., -5., 30., 5., 5.]);
        let scene = app.controller.scene().unwrap();
        assert!(scene.brep_occurrence_has_interior_wires(&scene.objects[0]).unwrap());
        let camera = om9_scene::camera::Camera::orthographic([0., 0., 20.], [0.; 3], [0., 1., 0.], 12.).unwrap();
        let viewport = om9_scene::camera::Viewport { origin: [0.; 2], size: [600.; 2] };
        let expected = [-3., 1., 15f64.sqrt()];
        let cursor = camera.project(expected, viewport).unwrap().unwrap().screen;
        let candidates = om9_scene::osnap::candidates(scene, &camera, viewport, cursor, Default::default(), None).unwrap();
        let hit = candidates.iter().find(|c| c.kind == om9_scene::osnap::SnapKind::Surface).expect("source surface candidate");
        let refined = app.controller.refine_surface_snap(hit, &camera, viewport).unwrap();
        assert!(refined.world.into_iter().zip(expected).all(|(a, b)| (a-b).abs() < 1e-6));
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &result);
        let scene = app.controller.scene().unwrap();
        assert!(scene.brep_occurrence_has_interior_wires(&scene.objects[0]).unwrap());
    }
}

#[test]
fn prepared_union_rejects_stale_selection_and_cancel_without_publication() {
    for cancel in [false, true] {
        let mut app = fixture(ViewportDetail::Light);
        let before = app.controller.document().clone();
        let publication = app.controller.scene().unwrap().publication_id();
        let history = app.controller.coordinator().undo_len();
        start(&mut app, false);
        if cancel { app.controller.cancel(); }
        else {
            app.controller.select_objects(&[]).unwrap();
            assert!(matches!(finish(&mut app), Err(Error::Stale(_))));
        }
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.scene().unwrap().publication_id(), publication);
        assert_eq!(app.controller.coordinator().undo_len(), history);
    }
}

#[test]
fn prepared_union_refuses_managed_pressure_without_publication() {
    let mut app = fixture(ViewportDetail::Light);
    app.controller.solid_sphere_points([100., 0., 0.], 1.).unwrap();
    app.controller.undo().unwrap();
    let before = app.controller.document().clone();
    let publication = app.controller.scene().unwrap().publication_id();
    let history = app.controller.coordinator().undo_len();
    let redo = app.controller.coordinator().redo_len();
    assert_eq!(redo, 1);
    start(&mut app, false);
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, Default::default()).unwrap();
    assert!(matches!(finish(&mut app), Err(Error::Budget(_))));
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.scene().unwrap().publication_id(), publication);
    assert_eq!(app.controller.coordinator().undo_len(), history);
    assert_eq!(app.controller.coordinator().redo_len(), redo);
    drop(pressure);
}

#[test]
fn prepared_binary_results_keep_exact_volume_and_undo() {
    for detail in [ViewportDetail::Light, ViewportDetail::High] {
        for (command, factor) in [("BooleanDifference", 1375.), ("BooleanIntersection", 625.)] {
            let mut app = MatrixApp::new(Document::with_template_tolerance("binary publication")).unwrap();
            app.controller.request_viewport_detail(detail).unwrap();
            app.controller.solid_sphere_points([0.;3], 5.).unwrap();
            let left = *app.controller.document().entities.keys().next().unwrap();
            app.controller.solid_sphere_points([5.,0.,0.], 5.).unwrap();
            let right = *app.controller.document().entities.keys().find(|id| **id != left).unwrap();
            let before = app.controller.document().clone();
            app.controller.select_objects(&[left]).unwrap();
            app.controller.submit(command).unwrap();
            app.controller.submit("DeleteInput=Yes").unwrap(); app.controller.submit("").unwrap();
            app.controller.select_objects(&[right]).unwrap(); app.controller.submit("").unwrap();
            finish(&mut app).unwrap();
            assert_eq!(app.controller.document().entities.len(),1);
            let result = app.controller.document().clone();
            let Geometry::Brep(body) = &result.entities.values().next().unwrap().geometry else { panic!() };
            let measured = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
            assert!((measured.volume_mm3 - factor*std::f64::consts::PI/12.).abs() < 1e-5, "{command}: {measured:?}");
            assert!(measured.closed && measured.solids == 1 && measured.naked_edges == 0);
            let scene = app.controller.scene().unwrap();
            assert!(scene.brep_occurrence_has_interior_wires(&scene.objects[0]).unwrap());
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &result);
        }
    }
}

#[test]
fn worker_buffer_receipt_cannot_target_another_source_or_stale_root() {
    use std::sync::Arc;
    let app = fixture(ViewportDetail::Light);
    let coordinator = app.controller.coordinator();
    let document = app.controller.scene().unwrap().document.clone();
    let ids = document.entities.keys().copied().collect::<Vec<_>>();
    let make_scene = || om9_scene::scene::Scene::capture_with_representations(document.clone(),
        coordinator.epoch(), coordinator.limits(), coordinator.representation_store()).unwrap();
    let buffers = |index: usize| {
        let Geometry::Brep(source) = &document.entities[&ids[index]].geometry else { panic!() };
        let candidate = coordinator.brep_representation(source).unwrap().materialize(source, coordinator.limits()).unwrap();
        let (prepared, _) = om9_scene::scene::Scene::prepare_brep_buffers(&candidate, coordinator.limits(), |_, _| Ok(())).unwrap();
        prepared.with_interior_wires(&[], coordinator.limits(), |_, _| Ok(())).unwrap().0
    };
    let scene = make_scene();
    let stamp = document.stamp(ids[0]).unwrap();
    assert!(scene.try_install_prepared_brep_wires(&stamp, buffers(1), Arc::new(())).is_err());
    let mut stale = stamp; stale.geometry_revision += 1;
    assert!(scene.try_install_prepared_brep_wires(&stale, buffers(0), Arc::new(())).is_err());
    let changed = scene.try_install_prepared_brep_wires(&stamp, buffers(0), Arc::new(())).unwrap();
    assert!(Arc::ptr_eq(&scene.document, &changed.document));
    assert!(!scene.brep_occurrence_has_interior_wires(&scene.objects[0]).unwrap());
    assert!(changed.brep_occurrence_has_interior_wires(&changed.objects[0]).unwrap());
}
