// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]
use om9_app::{display::ViewportDetail, ui::MatrixApp};
use om9_core::{Error, geometry::Geometry, model::Document, transaction::Cancellation};
use om9_scene::scene::RenderBuffers;
use std::time::{Duration, Instant};

fn fixture(detail: ViewportDetail) -> MatrixApp {
    let mut app = MatrixApp::new(Document::with_template_tolerance("speculative Boolean")).unwrap();
    app.controller.request_viewport_detail(detail).unwrap();
    for x in [0., 5., 20., 25.] {
        app.controller.solid_sphere_points([x, 0., 0.], 5.).unwrap();
    }
    let ids = app
        .controller
        .document()
        .entities
        .keys()
        .copied()
        .collect::<Vec<_>>();
    app.controller.select_objects(&ids).unwrap();
    app
}
fn begin(app: &mut MatrixApp) {
    app.controller.submit("BooleanUnion").unwrap();
    app.controller.submit("DeleteInput=Yes").unwrap();
    app.controller.submit("").unwrap();
}
fn wait_preview(app: &mut MatrixApp) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        app.controller.service_boolean_union_worker();
        assert!(
            app.controller.boolean_mesh_preview_failure().is_none(),
            "{:?}",
            app.controller.boolean_mesh_preview_failure()
        );
        if app.controller.boolean_mesh_preview().is_some() {
            return;
        }
        assert!(Instant::now() < deadline, "preview timed out");
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn approximate_preview_is_closed_and_disposable_in_both_profiles() {
    for detail in [ViewportDetail::Light, ViewportDetail::High] {
        let mut app = fixture(detail);
        let before = app.controller.document().clone();
        let saved = serde_json::to_vec(&before).unwrap();
        let undo = app.controller.coordinator().undo_len();
        begin(&mut app);
        wait_preview(&mut app);
        assert_eq!(app.controller.document(), &before);
        assert_eq!(
            serde_json::to_vec(app.controller.document()).unwrap(),
            saved
        );
        assert_eq!(app.controller.coordinator().undo_len(), undo);
        let (buffers, stamps, _) = app.controller.boolean_mesh_preview().unwrap();
        assert_eq!(stamps.len(), 4);
        let RenderBuffers::Mesh {
            vertices,
            triangles,
            ..
        } = buffers.as_ref()
        else {
            panic!()
        };
        let mut incidence = std::collections::BTreeMap::new();
        let mut volume = 0.;
        for (f, _) in triangles {
            let [a, b, c] = f.map(|i| vertices[i as usize]);
            volume += (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.;
            for [a, b] in [[f[0], f[1]], [f[1], f[2]], [f[2], f[0]]] {
                *incidence.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        assert!(incidence.values().all(|n| *n == 2));
        let expected = 1125. * std::f64::consts::PI / 2.;
        assert!(
            (volume - expected).abs() / expected < 0.15,
            "{detail:?}: {volume}"
        );
        let retained = buffers.clone();
        app.controller.cancel();
        assert!(app.controller.boolean_mesh_preview().is_none());
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), undo);
        assert!(matches!(retained.as_ref(), RenderBuffers::Mesh { .. }));
    }
}
#[test]
fn stale_preview_is_hidden_and_exact_commit_remains_atomic() {
    for stale in [false, true] {
        let mut app = fixture(ViewportDetail::Light);
        let before = app.controller.document().clone();
        let undo = app.controller.coordinator().undo_len();
        begin(&mut app);
        wait_preview(&mut app);
        if stale {
            app.controller.select_objects(&[]).unwrap();
            assert!(app.controller.boolean_mesh_preview().is_none());
        }
        let deadline = Instant::now() + Duration::from_secs(30);
        let result = loop {
            if let Some(result) = app.controller.poll_boolean_union_worker() {
                break result;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        };
        assert!(app.controller.boolean_mesh_preview().is_none());
        if stale {
            assert!(matches!(result, Err(Error::Stale(_))));
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().undo_len(), undo);
        } else {
            result.unwrap();
            let after = app.controller.document().clone();
            assert_eq!(after.entities.len(), 1);
            assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
            let Geometry::Brep(source) = &after.entities.values().next().unwrap().geometry else {
                panic!()
            };
            let measure =
                om9_kernel::creation::measure(source, app.controller.coordinator().limits())
                    .unwrap();
            assert!((measure.volume_mm3 - 1125. * std::f64::consts::PI / 2.).abs() < 1e-5);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        }
    }
}
#[test]
fn source_only_fold_succeeds_with_no_viewport_point_budget() {
    let app = fixture(ViewportDetail::High);
    let coordinator = app.controller.coordinator();
    let mut inputs = Vec::new();
    for entity in app.controller.document().entities.values() {
        let stamp = app.controller.document().stamp(entity.id).unwrap();
        inputs.push((
            coordinator.capture_brep_source(&stamp).unwrap(),
            entity.placement,
        ));
    }
    let mut limits = *coordinator.limits();
    limits.max_points = 32;
    let cancel = Cancellation::default();
    let source = om9_kernel::editing::boolean_union_captured_source_cancellable(
        &inputs,
        &limits,
        &cancel,
        |_, _| Ok(()),
    )
    .unwrap();
    assert!(!source.is_empty());
    assert!(
        om9_kernel::brep::capture_source_topology(&source, 0.1, 0.3, &limits, |_, _| Ok(()))
            .is_err()
    );
    let final_body = om9_kernel::brep::capture_source_topology(
        &source,
        0.1,
        0.3,
        coordinator.limits(),
        |_, _| Ok(()),
    )
    .unwrap()
    .into_parts()
    .0;
    let measured = om9_kernel::creation::measure(&final_body, coordinator.limits()).unwrap();
    assert_eq!(measured.solids, 2);
    assert_eq!(measured.naked_edges, 0);
    assert!((measured.volume_mm3 - 1125. * std::f64::consts::PI / 2.).abs() < 1e-5);
    cancel.cancel();
    assert!(matches!(
        om9_kernel::editing::boolean_union_captured_source_cancellable(
            &inputs,
            &limits,
            &cancel,
            |_, _| Ok(())
        ),
        Err(Error::Canceled)
    ));
    let refusal = om9_kernel::editing::boolean_union_captured_source_cancellable(
        &inputs,
        &limits,
        &Cancellation::default(),
        |_, _| Err(Error::Budget("test refusal")),
    );
    assert!(matches!(refusal, Err(Error::Budget(_))));
}
