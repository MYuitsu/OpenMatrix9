#![cfg(feature = "occt-brep")]

use om9_app::{display::ViewportDetail, ui::MatrixApp};
use om9_core::{brep::BrepRepresentationStore, geometry::Geometry, model::Document};
use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

#[test]
fn coupled_control_retains_document_display_and_exact_union_in_both_profiles() {
    for detail in [ViewportDetail::Light, ViewportDetail::High] {
        let mut outcomes = Vec::new();
        for coupled in [false, true] {
            let mut app = MatrixApp::new(Document::with_template_tolerance("coupled control")).unwrap();
            app.controller.request_viewport_detail(detail).unwrap();
            let store = app.controller.coordinator().representation_store();
            if coupled { store.lock().unwrap().enable_coupled_diagnostic().unwrap(); }
            let mut ids = Vec::new();
            for x in [0., 5., 20., 25.] {
                app.controller.solid_sphere_points([x,0.,0.],5.).unwrap();
                ids.push(app.controller.document().entities.keys().copied().find(|id|!ids.contains(id)).unwrap());
            }
            assert!(store.lock().unwrap().enable_coupled_diagnostic().is_err(), "cannot change architecture after creating geometry");
            for entity in app.controller.document().entities.values() {
                let Geometry::Brep(source) = &entity.geometry else { panic!("BRep") };
                assert_eq!(source.has_coupled_display_diagnostic(), coupled);
                assert_eq!(serde_json::to_value(source).unwrap().get("display").is_some(), coupled);
                let count = source.coupled_diagnostic_validation_count();
                entity.geometry.validate(app.controller.coordinator().limits()).unwrap();
                assert_eq!(source.coupled_diagnostic_validation_count() > count, coupled);
            }
            // The coupled scene reads its document-owned mesh even without an
            // independent representation cache; the separated arm requires it.
            let direct = om9_scene::scene::Scene::capture_with_representations(
                Arc::new(app.controller.document().clone()), 1, app.controller.coordinator().limits(),
                Arc::new(Mutex::new(BrepRepresentationStore::default())));
            assert_eq!(direct.is_ok(), coupled);
            let before = app.controller.document().clone();
            app.controller.select_objects(&ids).unwrap();
            app.controller.submit("BooleanUnion").unwrap();
            app.controller.submit("DeleteInput=Yes").unwrap();
            app.controller.submit("").unwrap();
            let deadline = Instant::now()+Duration::from_secs(45);
            loop {
                if let Some(result)=app.controller.poll_boolean_union_worker() { result.unwrap(); break; }
                assert_eq!(app.controller.document(), &before, "partial publication");
                assert!(Instant::now()<deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
            let result = app.controller.document().clone();
            let entity = result.entities.values().next().unwrap();
            let Geometry::Brep(source) = &entity.geometry else { panic!("union BRep") };
            assert_eq!(source.has_coupled_display_diagnostic(),coupled);
            let measured=om9_kernel::creation::measure(source,app.controller.coordinator().limits()).unwrap();
            assert!((measured.volume_mm3-1125.*std::f64::consts::PI/2.).abs()<1e-5);
            assert!(measured.closed && measured.solids==2 && measured.naked_edges==0);
            let scene=app.controller.scene().unwrap();
            let triangles=scene.root_brep_pick_display(&result.stamp(entity.id).unwrap()).unwrap().triangles.len();
            assert!(scene.brep_occurrence_has_interior_wires(&scene.objects[0]).unwrap());
            let camera=om9_scene::camera::Camera::orthographic([0.,0.,20.],[0.;3],[0.,1.,0.],12.).unwrap();
            let viewport=om9_scene::camera::Viewport {origin:[0.;2],size:[600.;2]};
            let expected=[-3.,1.,15f64.sqrt()];
            let cursor=camera.project(expected,viewport).unwrap().unwrap().screen;
            let candidates=om9_scene::osnap::candidates(scene,&camera,viewport,cursor,Default::default(),None).unwrap();
            let hit=candidates.iter().find(|hit|hit.kind==om9_scene::osnap::SnapKind::Surface).expect("source surface candidate");
            let snap=app.controller.refine_surface_snap(hit,&camera,viewport).unwrap();
            assert!(snap.world.into_iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-6));
            outcomes.push((triangles,measured.bounds_mm,source.source().to_owned()));
            app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
            app.controller.redo().unwrap(); assert_eq!(app.controller.document(),&result);
            let Geometry::Brep(restored)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!("restored")};
            assert_eq!(restored.has_coupled_display_diagnostic(),coupled);
        }
        assert_eq!(outcomes[0],outcomes[1],"architecture must preserve source bytes, bounds and display density");
    }
}
