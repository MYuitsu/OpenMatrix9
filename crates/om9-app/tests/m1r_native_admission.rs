// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]
use om9_app::{display::ViewportDetail, session::Controller};
use om9_core::{budget::Limits, geometry::Geometry, model::Document};

fn run(point_limit: u64, expected: usize) {
    let limits = Limits { max_bytes: 40 * 1024 * 1024 * 1024, max_objects: 400_000,
        max_points: point_limit, ..Limits::default() };
    let mut controller = Controller::new(Document::new("M1-R native admission"), limits).unwrap();
    controller.request_viewport_detail(ViewportDetail::High).unwrap();
    while controller.viewport_detail_pending() {
        if let Some(result) = controller.poll_viewport_detail() { result.unwrap(); }
        std::thread::yield_now();
    }
    assert_eq!(controller.viewport_detail(), ViewportDetail::High);
    'spheres: for index in 0..768usize {
        let pair = index / 2;
        let x = (pair % 20) as f64 * 20. - 190. + (index % 2) as f64 * 5.;
        let y = (pair / 20) as f64 * 16. - 152.;
        for text in ["Sphere".to_owned(), format!("{x},{y},0"), "5".to_owned()] {
            if let Err(error) = controller.submit(&text) {
                if point_limit == 2_000_000 && index == 648 && text == "5" {
                    assert!(matches!(error, om9_core::Error::Budget("points")), "{error:?}");
                    assert_eq!(controller.document().entities.len(), 648);
                    assert_eq!(controller.coordinator().undo_len(), 648);
                    assert_eq!(controller.coordinator().redo_len(), 0);
                    controller.submit("Cancel").unwrap();
                    println!("expected point refusal at 649; 648 objects/Undo preserved; scene_points={}",
                        controller.scene().unwrap().cache_usage().unwrap().points);
                    break 'spheres;
                }
                panic!("sphere={} input={text:?} error={error:?} objects={} undo={} prompt={:?} scene={:?} refusal={:?}",
                    index + 1, controller.document().entities.len(), controller.coordinator().undo_len(),
                    controller.prompt(), controller.scene_cache_usage(), controller.coordinator().last_external_memory_refusal());
            }
        }
        assert_eq!(controller.document().entities.len(), index + 1);
        assert_eq!(controller.coordinator().undo_len(), index + 1);
        if (index + 1) % 64 == 0 { println!("sphere={} scene={:?}", index + 1, controller.scene_cache_usage()); }
    }
    assert_eq!(controller.document().entities.len(), expected);
    println!("point_allowance={point_limit} objects={expected} undo={} scene_points={}",
        controller.coordinator().undo_len(), controller.scene().unwrap().cache_usage().unwrap().points);
    for entity in controller.document().entities.values().take(1).chain(controller.document().entities.values().rev().take(1)) {
        let Geometry::Brep(body) = &entity.geometry else { panic!("Sphere source changed type"); };
        let measured = om9_kernel::creation::measure(body, &limits).unwrap();
        assert!(measured.closed && measured.solids == 1 && measured.naked_edges == 0);
        assert!((measured.volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6);
    }
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), expected - 1);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), expected);
    assert_eq!(controller.coordinator().undo_len(), expected);
}

#[test]
#[ignore = "manual native default point-ceiling/rollback regression; no UI FPS claim"]
fn high_detail_default_point_ceiling_preserves_source_and_648_undo_steps() { run(2_000_000, 648); }

#[test]
#[ignore = "manual native stress fixture with explicit 4M point allowance; full ordinary Undo"]
fn high_detail_768_spheres_keep_exact_source_and_complete_history() { run(4_000_000, 768); }
