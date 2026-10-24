// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_kernel::creation;

fn controller() -> Controller {
    Controller::new(Document::new("Solid review"), Limits::default()).unwrap()
}
fn volume(c: &Controller) -> f64 {
    c.document()
        .entities
        .values()
        .filter_map(|e| match &e.geometry {
            Geometry::Brep(b) => Some(
                creation::measure(b, c.coordinator().limits())
                    .unwrap()
                    .volume_mm3,
            ),
            _ => None,
        })
        .sum()
}
// Optional authored fixtures for human review; ordinary tests write no files.
fn review_fixture(c: &Controller, name: &str) {
    let Some(directory) = std::env::var_os("OM9_SOLID_REVIEW_DIRECTORY") else {
        return;
    };
    let path = std::path::PathBuf::from(directory).join(format!("{name}.om9"));
    assert!(!path.exists(), "refuse to overwrite {}", path.display());
    om9_archive::Archive::save(
        &path,
        c.document(),
        c.coordinator().limits(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
}
fn circles(count: usize) -> Controller {
    let mut doc = Document::new("Exact extrusion");
    for n in 0..count {
        let curve = om9_core::curve_build::circle_on_frame(
            [n as f64 * 20.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            3.0,
        )
        .unwrap();
        let mut entity = Entity::new("profile", Geometry::Curve(curve), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(n as u128 + 1));
        doc.entities.insert(entity.id, entity);
    }
    let ids = doc.entities.keys().copied().collect::<Vec<_>>();
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c
}

#[test]
fn text_menu_has_content_height_retry_and_cancel() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-008", InvocationOrigin::Menu).unwrap();
    assert_eq!(c.prompt(), "Text");
    c.submit("H 2026").unwrap();
    assert_eq!(c.prompt(), "Height");
    assert!(c.submit("NaN").is_err());
    assert_eq!(c.prompt(), "Height");
    c.cancel();
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.dispatch("OM9-SOLID-008", InvocationOrigin::Menu).unwrap();
    c.submit("H").unwrap();
    c.submit("5").unwrap();
    review_fixture(&c, "Text-after");
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), 1);
    assert!(volume(&c) > 0.0);
    c.undo().unwrap();
    assert!(c.document().entities.is_empty());
    c.redo().unwrap();
    assert!(volume(&c) > 0.0);
}

#[test]
fn boolean_two_objects_menu_has_operation_prompt() {
    for (op, expected) in [("U", 1500.0), ("D", 500.0), ("I", 500.0)] {
        let mut c = controller();
        c.submit("Box 10 10 10").unwrap();
        c.submit("Box 10 10 10").unwrap();
        let ids = c.document().entities.keys().copied().collect::<Vec<_>>();
        c.select_objects(&[ids[1]]).unwrap();
        c.submit("Move").unwrap();
        c.accept_world_point([0.0; 3]).unwrap();
        c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.coordinator().undo_len();
        review_fixture(&c, &format!("Boolean2-{op}-before"));
        c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap();
        assert!(c.prompt().contains("Union"));
        assert!(c.submit("X").is_err());
        assert_eq!(c.document().entities.len(), 2);
        c.submit(op).unwrap();
        review_fixture(&c, &format!("Boolean2-{op}-after"));
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.coordinator().undo_len(), before + 1);
        assert!((volume(&c) - expected).abs() < 1e-7);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
    }
}

#[test]
fn immediate_boolean_menu_actions_have_measured_results() {
    for (feature, expected) in [
        ("OM9-SOLID-001", 500.0),
        ("OM9-SOLID-002", 500.0),
        ("OM9-SOLID-003", 1500.0),
    ] {
        let mut c = controller();
        c.submit("Box 10 10 10").unwrap();
        c.submit("Box 10 10 10").unwrap();
        let ids = c.document().entities.keys().copied().collect::<Vec<_>>();
        c.select_objects(&[ids[1]]).unwrap();
        c.submit("Move").unwrap();
        c.accept_world_point([0.0; 3]).unwrap();
        c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
        c.select_objects(&ids).unwrap();
        let undo = c.coordinator().undo_len();
        review_fixture(&c, &format!("{feature}-before"));
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        review_fixture(&c, &format!("{feature}-after"));
        assert!((volume(&c) - expected).abs() < 1e-7);
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.coordinator().undo_len(), undo + 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
        println!("menu_measured {feature} volume={expected}");
    }
}

#[test]
fn circle_extrusion_preserves_exact_geometry() {
    let mut c = circles(1);
    review_fixture(&c, "ExtrudeCrv-before");
    c.dispatch("OM9-SOLID-009", InvocationOrigin::Menu).unwrap();
    c.submit("5").unwrap();
    review_fixture(&c, "ExtrudeCrv-after");
    let expected = std::f64::consts::PI * 9.0 * 5.0;
    assert!(
        (volume(&c) / expected - 1.0).abs() < 1e-10,
        "{} vs {expected}",
        volume(&c)
    );
}

#[test]
fn extrude_all_is_one_atomic_transaction() {
    let mut c = circles(2);
    review_fixture(&c, "ExtrudeAll-before");
    c.dispatch("OM9-SOLID-010", InvocationOrigin::Menu).unwrap();
    c.submit("5").unwrap();
    review_fixture(&c, "ExtrudeAll-after");
    assert_eq!(c.coordinator().undo_len(), 1);
    assert_eq!(c.document().entities.len(), 4);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
    // Sorting IDs makes a valid first profile and invalid second profile deterministic.
    let mut doc = Document::new("No partial extrusion");
    let valid =
        om9_core::curve_build::rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0, 3.0)
            .unwrap();
    for (index, curve) in [
        valid,
        Curve::Line {
            start: [0.0; 3],
            end: [0.0, 0.0, 2.0],
            domain: [0.0, 1.0],
        },
    ]
    .into_iter()
    .enumerate()
    {
        let mut entity = Entity::new("profile", Geometry::Curve(curve), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        doc.entities.insert(entity.id, entity);
    }
    let ids = doc.entities.keys().copied().collect::<Vec<_>>();
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 1 1 1").unwrap();
    c.undo().unwrap();
    assert_eq!(c.coordinator().redo_len(), 1);
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.submit("ExtrudeAll 5").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn invalid_topology_indices_do_not_edit_edge_or_face_zero() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    for command in [
        "ExtractSrf -1",
        "ExtractSrf 0.5",
        "FilletEdge -1 1",
        "FilletEdge 0.5 1",
        "Shell -1 1",
        "Shell 0.5 1",
    ] {
        assert!(c.submit(command).is_err(), "{command}");
        assert_eq!(c.document(), &before);
    }
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn remaining_primitive_menu_prompts_produce_analytic_volumes() {
    let pi = std::f64::consts::PI;
    for (feature, input, expected) in [
        ("OM9-SOLID-012", "2 3 4", 24.0),
        ("OM9-SOLID-013", "2 3 4", 32.0 * pi),
        ("OM9-SOLID-014", "3", 36.0 * pi),
        ("OM9-SOLID-015", "5 1", 10.0 * pi * pi),
        ("OM9-SOLID-016", "3 5", 45.0 * pi),
        ("OM9-SOLID-017", "3 2 5", 25.0 * pi),
        ("OM9-SOLID-018", "4 3 5", 30.0),
        ("OM9-SOLID-019", "3 5", 15.0 * pi),
        ("OM9-SOLID-020", "3 1 5", 65.0 * pi / 3.0),
    ] {
        let mut c = controller();
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        c.cancel();
        assert_eq!(c.coordinator().undo_len(), 0);
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        c.submit(input).unwrap();
        review_fixture(&c, &format!("{feature}-after"));
        assert_eq!(c.coordinator().undo_len(), 1);
        assert!(
            (volume(&c) / expected - 1.0).abs() < 1e-9,
            "{feature}: {} expected {expected}",
            volume(&c)
        );
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
        println!("menu_measured {feature} volume={expected:.12}");
    }
}

#[test]
fn pipe_extract_fillet_shell_and_cap_have_measured_menu_results() {
    let limits = Limits::default();
    let mut doc = Document::new("pipe");
    let entity = Entity::new(
        "spine",
        Geometry::Curve(Curve::Line {
            start: [0.0; 3],
            end: [0.0, 0.0, 10.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    review_fixture(&c, "Pipe-before");
    c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap();
    c.submit("1").unwrap();
    review_fixture(&c, "Pipe-after");
    assert!((volume(&c) - 10.0 * std::f64::consts::PI).abs() < 1e-7);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    for (feature, input, expected) in [
        ("OM9-SOLID-006", "0", 1000.0),
        ("OM9-SOLID-007", "0 1", 997.8539816339745),
        ("OM9-SOLID-029", "0 1", 424.0),
    ] {
        let mut c = controller();
        c.submit("Box 10 10 10").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        c.select_objects(&[id]).unwrap();
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        c.submit(input).unwrap();
        review_fixture(&c, &format!("{feature}-after"));
        assert_eq!(c.coordinator().undo_len(), 2);
        assert!(
            (volume(&c) - expected).abs() < 1e-6,
            "{feature}: {}",
            volume(&c)
        );
        if feature == "OM9-SOLID-006" {
            let extracted = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(b) = &extracted.geometry else {
                panic!()
            };
            let m = creation::measure(b, &limits).unwrap();
            assert_eq!(m.faces, 1);
            assert!((m.area_mm2 - 100.0).abs() < 1e-7);
        }
        c.undo().unwrap();
        assert!((volume(&c) - 1000.0).abs() < 1e-7);
    }
    let boxed = creation::primitive(
        creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let open = om9_kernel::editing::remove_face(&boxed, 0, 0.01, &limits).unwrap();
    let mut doc = Document::new("cap");
    let entity = Entity::new("open box", Geometry::Brep(open), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    review_fixture(&c, "Cap-before");
    c.dispatch("OM9-SOLID-005", InvocationOrigin::Menu).unwrap();
    review_fixture(&c, "Cap-after");
    assert!((volume(&c) - 1000.0).abs() < 1e-7);
    assert_eq!(c.coordinator().undo_len(), 1);
    println!("menu_measured Pipe Extract Fillet Shell Cap=PASS");
}

#[test]
#[ignore = "requires files saved by the native UI; set OM9_SOLID_NATIVE_REVIEW_DIRECTORY"]
fn native_ui_saved_solid_workflows() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("OM9_SOLID_NATIVE_REVIEW_DIRECTORY").expect("native review directory"),
    );
    let limits = Limits::default();
    for (name, entities, solids, expected) in [
        ("text-live", 1, 1, None),
        ("boolean2-live", 1, 1, Some(1500.0)),
        ("extrude-all-live", 4, 2, Some(90.0 * std::f64::consts::PI)),
    ] {
        let restored = om9_archive::Archive::open_restored(
            &directory.join(format!("{name}.om9")),
            &limits,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(restored.document.entities.len(), entities);
        let mut count = 0;
        let mut total = 0.0;
        for entity in restored.document.entities.values() {
            if let Geometry::Brep(brep) = &entity.geometry {
                let measured = creation::measure(brep, &limits).unwrap();
                assert_eq!(measured.solids, 1);
                assert!(measured.closed);
                assert_eq!(measured.naked_edges, 0);
                assert!(measured.volume_mm3 > 0.0);
                count += 1;
                total += measured.volume_mm3;
            }
        }
        assert_eq!(count, solids);
        if let Some(expected) = expected {
            assert!((total / expected - 1.0).abs() < 1e-10);
        }
        if name == "text-live" {
            let fixture = om9_archive::Archive::open_restored(
                &directory.join("review-fixtures/Text-after.om9"),
                &limits,
                &Default::default(),
            )
            .unwrap();
            let c = Controller::new(fixture.document, limits).unwrap();
            assert!((total / volume(&c) - 1.0).abs() < 1e-10);
        }
        println!("native_ui_saved {name} entities={entities} solids={count} volume={total:.12}");
    }
}
