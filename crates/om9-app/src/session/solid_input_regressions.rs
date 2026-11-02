// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

use super::{Controller, InvocationOrigin};
use om9_core::geometry::Geometry;
use om9_core::{budget::Limits, model::Document};

fn controller() -> Controller {
    Controller::new(Document::new("Solid input regression"), Limits::default()).unwrap()
}

#[test]
fn pyramid_rejects_fractional_side_count_without_mutation() {
    let mut c = controller();
    assert!(c.submit("Pyramid 4.5 3 5").is_err());
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn pyramid_prompt_retries_invalid_complete_input() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-018", InvocationOrigin::Menu).unwrap();
    assert!(c.submit("4.5 3 5").is_err());
    assert_eq!(c.prompt(), "Center of pyramid base");
    assert!(c.document().entities.is_empty());
    c.submit("4 3 5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn pyramid_rejects_invalid_side_count_before_advancing() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-018", InvocationOrigin::Menu).unwrap();
    assert!(c.submit("NumSides=4.5").is_err());
    assert_eq!(c.prompt(), "Center of pyramid base");
    c.submit("NumSides=4").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    assert_eq!(c.prompt(), "Base corner or radius");
    c.submit("3").unwrap();
    c.submit("5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn numeric_prompt_preserves_previous_inputs_on_parse_and_geometry_errors() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-016", InvocationOrigin::Menu).unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("3").unwrap();
    assert_eq!(c.prompt(), "Height or point");
    assert!(c.submit("5 nope").is_err());
    assert_eq!(c.prompt(), "Height or point");
    assert!(c.submit("0").is_err());
    assert_eq!(c.prompt(), "Height or point");
    assert!(c.document().entities.is_empty());
    c.submit("5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn boolean_operation_failure_keeps_prompt_and_document_unchanged() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap();
    assert!(c.submit("U").is_err());
    assert!(c.prompt().contains("Operation"));
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.cancel();
}

#[test]
fn boolean_difference_uses_operand_pick_order_for_asymmetric_boxes() {
    for route in 0..4 {
        for method in 0..4 {
            for reverse in [false, true] {
                let limits = Limits::default();
                let large = om9_core::identity::ObjectId(uuid::Uuid::from_u128(1));
                let small = om9_core::identity::ObjectId(uuid::Uuid::from_u128(2));
                let mut doc = Document::new("Asymmetric Boolean");
                for (id, side) in [(large, 10.0), (small, 5.0)] {
                    let shape = om9_kernel::creation::primitive(
                        om9_kernel::creation::Primitive::Box {
                            x_mm: side,
                            y_mm: side,
                            z_mm: side,
                        },
                        0.01,
                        &limits,
                    )
                    .unwrap();
                    let mut entity =
                        om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
                    entity.id = id;
                    doc.entities.insert(id, entity);
                }
                let mut c = Controller::new(doc, limits).unwrap();
                c.select_objects(&[small]).unwrap();
                c.submit("Move").unwrap();
                c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
                c.accept_world_point([7.5, 0.0, 0.0]).unwrap();
                c.clear_selection();
                let ids = if reverse {
                    [small, large]
                } else {
                    [large, small]
                };
                match method {
                    0 => c.select_objects(&ids).unwrap(),
                    1 => {
                        c.select_root(Some(ids[0]), false).unwrap();
                        c.select_root(Some(ids[1]), true).unwrap();
                    }
                    2 => {
                        c.add_objects(&ids[..1]).unwrap();
                        c.add_objects(&ids[1..]).unwrap();
                    }
                    _ => {
                        c.toggle_selection(ids[0]).unwrap();
                        c.toggle_selection(ids[1]).unwrap();
                    }
                }
                if route < 2 {
                    c.remove_objects(&[ids[1]]).unwrap();
                }
                match route {
                    0 => c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap(),
                    1 => c.submit("BooleanDifference").unwrap(),
                    2 => {
                        c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap();
                        c.submit("DeleteInput=Yes").unwrap();
                        c.submit("D").unwrap();
                        c.submit("").unwrap();
                    }
                    _ => {
                        c.submit("Boolean2Objects").unwrap();
                        c.submit("DeleteInput=Yes").unwrap();
                        c.submit("D").unwrap();
                        c.submit("").unwrap();
                    }
                }
                if route < 2 {
                    c.submit("DeleteInput=Yes").unwrap();
                    c.submit("").unwrap();
                    c.solid_difference_pick(ids[1], false).unwrap();
                    c.submit("").unwrap();
                }
                let body = c
                    .document()
                    .entities
                    .values()
                    .find_map(|e| match &e.geometry {
                        Geometry::Brep(b) => Some(b),
                        _ => None,
                    })
                    .unwrap();
                let volume = om9_kernel::creation::measure(body, c.coordinator().limits())
                    .unwrap()
                    .volume_mm3;
                let expected = if reverse { 62.5 } else { 937.5 };
                assert!(
                    (volume - expected).abs() < 1e-6,
                    "route={route} method={method} reverse={reverse} volume={volume}"
                );
                c.undo().unwrap();
                assert_eq!(c.document().entities.len(), 2);
                c.redo().unwrap();
                assert_eq!(c.document().entities.len(), 1);
            }
        }
    }
}

#[test]
fn selection_order_follows_remove_reselect_and_replacement() {
    let mut c = controller();
    c.submit("Box 2 2 2").unwrap();
    c.submit("Box 3 3 3").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&[ids[1], ids[0]]).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[1], ids[0]]);
    c.remove_objects(&[ids[1]]).unwrap();
    c.add_objects(&[ids[1]]).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0], ids[1]]);
    c.toggle_selection(ids[0]).unwrap();
    c.toggle_selection(ids[0]).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[1], ids[0]]);
    c.clear_selection();
    c.select_root(Some(ids[0]), false).unwrap();
    c.select_root(Some(ids[1]), true).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0], ids[1]]);
    let revision = c.selection_revision;
    c.select_objects(&[ids[1], ids[0]]).unwrap();
    assert!(c.selection_revision > revision);
    assert_eq!(c.selected_ordered(), vec![ids[1], ids[0]]);
}

#[test]
fn boolean_union_of_three_breps_is_atomic_and_preserves_disconnected_solids() {
    for via_menu in [true, false] {
        let limits = Limits::default();
        let mut doc = Document::new("Three-way union");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0, 30.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        if via_menu {
            c.select_objects(&ids).unwrap();
        } else {
            c.select_objects(&[ids[0], ids[2], ids[1]]).unwrap();
        }
        if via_menu {
            c.dispatch("OM9-SOLID-003", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("BooleanUnion").unwrap();
        }
        c.submit("DeleteInput=Yes").unwrap();
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.coordinator().undo_len(), 1);
        let Geometry::Brep(result) = &c.document().entities.values().next().unwrap().geometry
        else {
            panic!("expected BRep");
        };
        let measures = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
        assert!((measures.volume_mm3 - 2500.0).abs() < 1e-6);
        assert_eq!(measures.solids, 2);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 3);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 1);
        let result_id = *c.document().entities.keys().next().unwrap();
        let path = std::env::temp_dir().join(format!("om9-union-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(
            &path,
            c.document(),
            c.coordinator().limits(),
            &om9_core::transaction::Cancellation::default(),
        )
        .unwrap();
        let restored = om9_archive::Archive::open_restored(
            &path,
            c.coordinator().limits(),
            &Default::default(),
        )
        .unwrap();
        std::fs::remove_file(&path).unwrap();
        let Geometry::Brep(restored_body) = &restored.document.entities[&result_id].geometry else {
            panic!("restored BRep");
        };
        let restored_measures =
            om9_kernel::creation::measure(restored_body, c.coordinator().limits()).unwrap();
        assert!((restored_measures.volume_mm3 - 2500.0).abs() < 1e-6);
        assert_eq!(restored_measures.solids, 2);
    }
}

#[test]
fn boolean_union_invalid_late_operand_preserves_document_and_redo() {
    let limits = Limits::default();
    let mut doc = Document::new("Invalid union operand");
    let mut ids = Vec::new();
    for (index, side) in [2.0, 3.0].into_iter().enumerate() {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: side,
                y_mm: side,
                z_mm: side,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut point =
        om9_core::model::Entity::new("not a solid", Geometry::Point([10.0, 0.0, 0.0]), None);
    point.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(3));
    ids.push(point.id);
    doc.entities.insert(point.id, point);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("BooleanUnion").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn boolean_union_late_open_brep_failure_is_atomic() {
    let limits = Limits::default();
    let mut doc = Document::new("Late union failure");
    let mut ids = Vec::new();
    for index in 0..3 {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let body = if index == 2 {
            om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap()
        } else {
            body
        };
        let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(body), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        entity.placement =
            om9_core::geometry::Transform::translation([index as f64 * 5.0, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("BooleanUnion").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn boolean_union_staged_selection_and_delete_input_are_atomic() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    for delete_input in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Union selection and input retention");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        if delete_input {
            c.select_objects(&[ids[0]]).unwrap();
            c.submit("BooleanUnion DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("OM9-SOLID-003", InvocationOrigin::Menu).unwrap();
            c.solid_boolean_pick(ids[0], false).unwrap();
        }
        assert!(c.solid_boolean_needs_object());
        assert_eq!(c.active_feature(), Some("OM9-SOLID-003"));
        assert_eq!(
            c.command_options()
                .into_iter()
                .find(|o| o.id.name() == "DeleteInput")
                .unwrap()
                .value,
            CommandValue::Flag(delete_input)
        );
        let before = c.document().clone();
        assert!(c.submit("").is_err());
        assert_eq!(c.document(), &before);
        assert!(c.submit("DeleteInput=invalid").is_err());
        c.solid_boolean_pick(ids[1], false).unwrap();
        c.submit("").unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
        for id in &ids {
            assert_eq!(c.document().entities.contains_key(id), !delete_input);
        }
        let output = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &output.geometry else {
            panic!("BRep result")
        };
        let measured = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert!((measured.volume_mm3 - 1500.0).abs() < 1e-6);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
        c.redo().unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
    }
}

#[test]
fn boolean_union_pending_undo_and_cancel_preserve_document() {
    let mut c = controller();
    c.submit("Box 2 2 2").unwrap();
    c.submit("Box 3 3 3").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.clear_selection();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.submit("BooleanUnion").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    assert!(c.submit("BooleanUnion unexpected").is_err());
    assert!(c.solid_boolean_needs_object());
    assert_eq!(c.document(), &before);
    c.cancel();
}

#[test]
fn boolean_intersection_fuses_two_sets_and_honors_delete_input() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    for delete_input in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Intersection sets");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0, 8.0, 12.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids[..2]).unwrap();
        let before = c.document().clone();
        assert!(c.dispatch_solid_args("OM9-SOLID-002", &[]).is_err());
        if delete_input {
            c.submit("BooleanIntersection DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("OM9-SOLID-002", InvocationOrigin::Menu).unwrap();
        }
        assert!(c.solid_boolean_needs_object());
        assert_eq!(c.active_feature(), Some("OM9-SOLID-002"));
        assert!(c.prompt().contains("first set"));
        assert_eq!(c.document(), &before);
        let option = c
            .command_options()
            .into_iter()
            .find(|o| o.id.name() == "DeleteInput")
            .unwrap();
        assert_eq!(option.value, CommandValue::Flag(delete_input));
        assert!(c.submit("DeleteInput=invalid").is_err());
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        assert!(c.prompt().contains("second set"));
        assert!(c.solid_boolean_pick(ids[0], false).is_err());
        for id in &ids[2..] {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 5 }
        );
        for id in &ids {
            assert_eq!(c.document().entities.contains_key(id), !delete_input);
        }
        let entity = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &entity.geometry else {
            panic!("BRep result")
        };
        let measured = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(measured.solids, 1);
        assert!(
            (measured.volume_mm3 - 700.0).abs() < 1e-6,
            "{}",
            measured.volume_mm3
        );
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 4);
        c.redo().unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 5 }
        );
    }
}

#[test]
fn boolean_intersection_empty_result_preserves_document_and_redo() {
    let limits = Limits::default();
    let mut doc = Document::new("Disjoint intersection");
    let mut ids = Vec::new();
    for (index, x) in [0.0, 30.0].into_iter().enumerate() {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 2 2 2").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("BooleanIntersection DeleteInput=Yes").unwrap();
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert!(c.solid_boolean_needs_object());
    c.cancel();
}

#[test]
fn boolean_intersection_pending_undo_and_cancel_preserve_document() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanIntersection").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[2], false).unwrap();
    c.submit("Undo").unwrap();
    c.submit("Undo").unwrap();
    assert!(c.prompt().contains("first set"));
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn boolean_intersection_open_late_input_is_atomic_and_retryable() {
    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let entity = doc.entities.get_mut(&ids[3]).unwrap();
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep")
    };
    entity.geometry = Geometry::Brep(
        om9_kernel::editing::remove_face(body, 0, 0.01, &Limits::default()).unwrap(),
    );
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("BooleanIntersection").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[2], false).unwrap();
    c.solid_boolean_pick(ids[3], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert!(c.solid_boolean_needs_object());
    c.solid_boolean_pick(ids[3], true).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn boolean_two_objects_still_rejects_three_operands() {
    let mut c = controller();
    for side in [2, 3, 4] {
        c.submit(&format!("Box {side} {side} {side}")).unwrap();
    }
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.submit("Boolean2Objects").unwrap();
    assert!(c.submit("U").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
}

#[test]
fn boolean_split_stages_same_inputs_and_commits_separate_pieces_atomically() {
    for delete in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Boolean Split sets");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        if delete {
            c.submit("BooleanSplit DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("SolidBooleanSplit", InvocationOrigin::Menu)
                .unwrap();
        }
        for id in &ids {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        for id in &ids {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 5 });
        for id in &ids {
            assert_eq!(c.document().entities.contains_key(id), !delete);
        }
        let mut positions = Vec::new();
        for (id, entity) in &c.document().entities {
            if ids.contains(id) {
                continue;
            }
            let Geometry::Brep(body) = &entity.geometry else {
                panic!("BRep piece")
            };
            let m = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(m.closed && m.solids == 1 && (m.volume_mm3 - 500.0).abs() < 1e-6);
            positions.push(m.bounds_mm[0]);
        }
        positions.sort_by(f64::total_cmp);
        assert_eq!(positions.len(), 3);
        for (actual, expected) in positions.into_iter().zip([0.0, 5.0, 10.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 5 });
    }
}

#[test]
fn boolean_split_rejects_stale_or_open_input_without_partial_commit() {
    use om9_core::transaction::{Mutation, TransactionRequest};
    let (mut stale, ids) = difference_group_controller();
    stale.submit("BooleanSplit").unwrap();
    stale.solid_boolean_pick(ids[0], false).unwrap();
    stale.submit("").unwrap();
    let source = stale.document().stamp(ids[0]).unwrap();
    let edit = stale
        .coordinator
        .prepare(TransactionRequest::new(
            "external edit",
            vec![Mutation::Placement {
                source,
                placement: om9_core::geometry::Transform::translation([1.0, 0.0, 0.0]),
            }],
        ))
        .unwrap();
    stale.coordinator.commit(edit).unwrap();
    stale.refresh_scene().unwrap();
    stale.solid_boolean_pick(ids[2], false).unwrap();
    let before = stale.document().clone();
    let undo = stale.coordinator.undo_len();
    assert!(matches!(stale.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(stale.document(), &before);
    assert_eq!(stale.coordinator.undo_len(), undo);
    assert!(stale.is_running());

    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let entity = doc.entities.get_mut(&ids[3]).unwrap();
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep cutter")
    };
    entity.geometry = Geometry::Brep(
        om9_kernel::editing::remove_face(body, 0, 0.01, &Limits::default()).unwrap(),
    );
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("BooleanSplit DeleteInput=Yes").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[3], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
    assert!(c.is_running());
    c.cancel();
}

#[test]
fn boolean_split_pending_undo_and_cancel_do_not_publish() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanSplit").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("Undo").unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
}

#[test]
fn create_solid_forms_regions_from_faces_and_honors_delete_input() {
    for delete in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Create solid faces");
        let mut ids = Vec::new();
        for (body_index, x) in [0.0, 5.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            for face_index in 0..6 {
                let face =
                    om9_kernel::editing::extract_face(&body, face_index, 0.01, &limits).unwrap();
                let mut entity = om9_core::model::Entity::new("face", Geometry::Brep(face), None);
                entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(
                    (body_index * 6 + face_index + 1) as u128,
                ));
                entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
                ids.push(entity.id);
                doc.entities.insert(entity.id, entity);
            }
        }
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        c.select_objects(&ids).unwrap();
        if delete {
            c.submit("CreateSolid DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("SolidCreateSolid", InvocationOrigin::Menu)
                .unwrap();
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator.undo_len(), 0);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 15 });
        let mut starts = Vec::new();
        for (id, entity) in &c.document().entities {
            if ids.contains(id) {
                continue;
            }
            let Geometry::Brep(body) = &entity.geometry else {
                panic!("solid region")
            };
            let m = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(m.closed && m.solids == 1 && (m.volume_mm3 - 500.0).abs() < 1e-6);
            starts.push(m.bounds_mm[0]);
        }
        starts.sort_by(f64::total_cmp);
        for (actual, expected) in starts.into_iter().zip([0.0, 5.0, 10.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 15 });
    }
}

#[test]
fn create_solid_open_set_and_cancel_preserve_document_and_redo() {
    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let body = match &doc.entities[&ids[0]].geometry {
        Geometry::Brep(body) => body,
        _ => panic!("BRep"),
    };
    let face = om9_kernel::editing::extract_face(body, 0, 0.01, &Limits::default()).unwrap();
    doc.entities.get_mut(&ids[0]).unwrap().geometry = Geometry::Brep(face);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("CreateSolid DeleteInput=Yes").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
    assert!(c.is_running());
    c.submit("Undo").unwrap();
    assert!(c.selected().is_empty());
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.redo_len(), 1);
}

#[test]
fn cap_planar_stages_multiple_shells_and_commits_one_transaction() {
    for menu in [true, false] {
        let limits = Limits::default();
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let shell = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
        let mut doc = Document::new("Cap multiple");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 20.0].into_iter().enumerate() {
            let mut entity =
                om9_core::model::Entity::new("open box", Geometry::Brep(shell.clone()), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        if menu {
            c.dispatch("OM9-SOLID-005", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("Cap").unwrap();
        }
        for id in &ids {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator.undo_len(), 0);
        c.submit("").unwrap();
        for (index, id) in ids.iter().enumerate() {
            let entity = &c.document().entities[id];
            assert_eq!(entity.placement, before.entities[id].placement);
            let Geometry::Brep(body) = &entity.geometry else {
                panic!("capped BRep")
            };
            let m = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(
                m.closed && m.solids == 1 && (m.volume_mm3 - 1000.0).abs() < 1e-6,
                "{index}"
            );
        }
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
    }
}

#[test]
fn cap_planar_late_invalid_input_is_atomic_and_retryable() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let shell = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
    let mut doc = Document::new("Cap failure");
    let mut ids = Vec::new();
    for (index, shape) in [shell, body].into_iter().enumerate() {
        let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(shape), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    for id in &ids {
        c.solid_boolean_pick(*id, false).unwrap();
    }
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
    assert!(c.is_running());
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
}

#[test]
fn cap_planar_multiple_loops_through_command_and_undo() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let once = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
    let shell = (0..once.display.faces.len())
        .filter_map(|i| om9_kernel::editing::remove_face(&once, i, 0.01, &limits).ok())
        .find(|shape| {
            om9_kernel::creation::measure(shape, &limits)
                .unwrap()
                .naked_edges
                == 8
        })
        .expect("opposite faces removed");
    let mut doc = Document::new("Cap two loops");
    let entity = om9_core::model::Entity::new("open box", Geometry::Brep(shell), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(
        measure.closed && measure.naked_edges == 0 && (measure.volume_mm3 - 1000.0).abs() < 1e-6
    );
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn cap_planar_annular_hole_through_command_and_undo() {
    let limits = Limits::default();
    let cylinder = |radius_mm| {
        om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Cylinder {
                radius_mm,
                height_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap()
    };
    let ring = om9_kernel::editing::boolean(
        &cylinder(5.0),
        &cylinder(2.0),
        om9_kernel::editing::BooleanOp::Difference,
        0.01,
        &limits,
    )
    .unwrap()
    .body()
    .unwrap()
    .clone();
    let top = (0..ring.display.faces.len())
        .find(|&i| {
            let vertices: Vec<_> = ring
                .display
                .triangles
                .iter()
                .filter(|triangle| triangle[3] as usize == i)
                .flat_map(|triangle| {
                    triangle[..3]
                        .iter()
                        .map(|index| ring.display.points[*index as usize][2])
                })
                .collect();
            !vertices.is_empty() && vertices.iter().all(|z| (*z - 10.0).abs() < 1e-9)
        })
        .expect("top annular face");
    let shell = om9_kernel::editing::remove_face(&ring, top, 0.01, &limits).unwrap();
    let mut doc = Document::new("Cap annulus");
    let entity = om9_core::model::Entity::new("open tube", Geometry::Brep(shell), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(
        measure.closed
            && measure.naked_edges == 0
            && (measure.volume_mm3 - 210.0 * std::f64::consts::PI).abs() < 1e-5
    );
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn cap_planar_partial_shell_through_command_and_undo() {
    use om9_kernel::{creation, editing};
    let limits = Limits::default();
    let box_at = |x: f64, height: f64| {
        let profile = creation::Profile::new(vec![
            [x, 0.0, 0.0],
            [x + 10.0, 0.0, 0.0],
            [x + 10.0, 10.0, 0.0],
            [x, 10.0, 0.0],
        ])
        .unwrap();
        creation::extrude(&profile, [0.0, 0.0, height], true, 0.01, &limits).unwrap()
    };
    let mut shell = editing::boolean(
        &box_at(0.0, 10.0),
        &box_at(10.0, 5.0),
        editing::BooleanOp::Union,
        0.01,
        &limits,
    )
    .unwrap()
    .body()
    .unwrap()
    .clone();
    let vertices = |brep: &om9_core::brep::Brep, face: usize| -> Vec<[f64; 3]> {
        brep.display
            .triangles
            .iter()
            .filter(|t| t[3] as usize == face)
            .flat_map(|t| t[..3].iter().map(|i| brep.display.points[*i as usize]))
            .collect()
    };
    for z in [10.0, 5.0] {
        while let Some(face) = (0..shell.display.faces.len()).find(|&i| {
            let points = vertices(&shell, i);
            !points.is_empty() && points.iter().all(|p| (p[2] - z).abs() < 1e-9)
        }) {
            shell = editing::remove_face(&shell, face, 0.01, &limits).unwrap();
        }
    }
    let step = (0..shell.display.faces.len())
        .find(|&i| {
            let points = vertices(&shell, i);
            !points.is_empty()
                && points.iter().all(|p| (p[0] - 10.0).abs() < 1e-9)
                && points.iter().any(|p| (p[2] - 5.0).abs() < 1e-9)
                && points.iter().any(|p| (p[2] - 10.0).abs() < 1e-9)
        })
        .unwrap();
    shell = editing::remove_face(&shell, step, 0.01, &limits).unwrap();
    while let Some(face) = (0..shell.display.faces.len()).find(|&i| {
        let points = vertices(&shell, i);
        !points.is_empty() && points.iter().all(|p| p[2].abs() < 1e-9)
    }) {
        shell = editing::remove_face(&shell, face, 0.01, &limits).unwrap();
    }
    let before_measure = creation::measure(&shell, &limits).unwrap();
    let mut doc = Document::new("Partial cap");
    let entity = om9_core::model::Entity::new("open stepped shell", Geometry::Brep(shell), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let after = creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(
        !after.closed && after.naked_edges > 0 && after.naked_edges < before_measure.naked_edges
    );
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn extract_surface_separates_face_in_one_undoable_transaction() {
    let mut c = Controller::new(Document::new("Extract"), Limits::default()).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo_before = c.coordinator.undo_len();
    c.submit("ExtractSrf 0").unwrap();
    let Geometry::Brep(remainder) = &c.document().entities[&id].geometry else {
        panic!("shell")
    };
    let measure = om9_kernel::creation::measure(remainder, c.coordinator().limits()).unwrap();
    assert!(!measure.closed);
    assert_eq!(measure.faces, 5);
    assert_eq!(c.document().entities.len(), 2);
    assert_eq!(c.coordinator.undo_len(), undo_before + 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn extract_surface_copy_preserves_source_and_placement() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("Extract copy");
    let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
    entity.placement = om9_core::geometry::Transform::translation([20.0, 3.0, 2.0]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("ExtractSrf 0 Copy=Yes").unwrap();
    assert_eq!(c.document().entities[&id], before.entities[&id]);
    let extracted = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    assert_eq!(extracted.placement, before.entities[&id].placement);
    let Geometry::Brep(face) = &extracted.geometry else {
        panic!("face")
    };
    assert_eq!(
        om9_kernel::creation::measure(face, c.coordinator().limits())
            .unwrap()
            .faces,
        1
    );
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn extract_surface_failure_leaves_document_and_redo_intact() {
    let mut c = Controller::new(Document::new("Extract failure"), Limits::default()).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.submit("Box 2 2 2").unwrap();
    c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo_before = c.coordinator.undo_len();
    let redo_before = c.coordinator.redo_len();
    assert!(c.submit("ExtractSrf 999").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), undo_before);
    assert_eq!(c.coordinator.redo_len(), redo_before);
}

#[test]
fn extract_surface_interactive_options_choose_output_layer() {
    use om9_core::model::Layer;
    for input_layer in [false, true] {
        let limits = Limits::default();
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut doc = Document::new("Extract layers");
        let source_layer = Layer::new("Source", [255, 0, 0, 255]);
        let current_layer = Layer::new("Current", [0, 255, 0, 255]);
        let source_id = source_layer.id;
        let current_id = current_layer.id;
        doc.layers.insert(source_id, source_layer);
        doc.layers.insert(current_id, current_layer);
        doc.layer_order.extend([source_id, current_id]);
        doc.current_layer = Some(current_id);
        let entity = om9_core::model::Entity::new("box", Geometry::Brep(body), Some(source_id));
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        c.submit("ExtractSrf").unwrap();
        c.submit("Copy=Yes").unwrap();
        if input_layer {
            c.submit("OutputLayer=Input").unwrap();
        }
        c.submit("0").unwrap();
        let extracted = c.document().entities.values().find(|e| e.id != id).unwrap();
        assert_eq!(
            extracted.attributes.layer,
            Some(if input_layer { source_id } else { current_id })
        );
        assert_eq!(c.document().entities[&id], before.entities[&id]);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn extract_surface_multiple_faces_and_sources_commit_atomically() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    for copy in [false, true] {
        let mut doc = Document::new("multi extract");
        let mut ids = Vec::new();
        for x in [0.0, 20.0] {
            let mut entity =
                om9_core::model::Entity::new("box", Geometry::Brep(shape.clone()), None);
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        c.submit("ExtractSrf").unwrap();
        if copy {
            c.submit("Copy=Yes").unwrap();
        }
        for (id, index) in [(ids[0], 0), (ids[0], 2), (ids[1], 1)] {
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
                panic!()
            };
            let face = brep.display.faces[index];
            let object = c
                .scene()
                .unwrap()
                .objects
                .iter()
                .find(|o| o.root == id && o.path.is_empty())
                .unwrap();
            let reference = super::SelectedSubobject {
                root: id,
                path: vec![],
                subset: om9_scene::scene::Subelement::BrepFace(face),
                source: object.source,
                world_epoch: c.scene().unwrap().epoch,
            };
            c.solid_extract_pick_face(reference).unwrap();
        }
        assert_eq!(c.subselection().len(), 3);
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), 5);
        assert_eq!(c.coordinator.undo_len(), 1);
        for (id, expected) in [
            (ids[0], if copy { 6 } else { 4 }),
            (ids[1], if copy { 6 } else { 5 }),
        ] {
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
                panic!()
            };
            assert_eq!(
                om9_kernel::creation::measure(brep, c.coordinator().limits())
                    .unwrap()
                    .faces,
                expected
            );
        }
        assert_eq!(
            c.document()
                .entities
                .values()
                .filter(|e| !ids.contains(&e.id))
                .count(),
            3
        );
        let mut positions: Vec<_> = c
            .document()
            .entities
            .values()
            .filter(|e| !ids.contains(&e.id))
            .map(|e| e.placement.0[0][3])
            .collect();
        positions.sort_by(f64::total_cmp);
        assert_eq!(positions, vec![0.0, 0.0, 20.0]);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 5);
    }
}

#[test]
fn extract_surface_all_faces_removes_source_once_and_undo_restores_it() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("extract all faces");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("ExtractSrf").unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
        panic!()
    };
    let faces = brep.display.faces.clone();
    for face in faces {
        let object = c
            .scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == id && o.path.is_empty())
            .unwrap();
        let reference = super::SelectedSubobject {
            root: id,
            path: vec![],
            subset: om9_scene::scene::Subelement::BrepFace(face),
            source: object.source,
            world_epoch: c.scene().unwrap().epoch,
        };
        c.solid_extract_pick_face(reference).unwrap();
    }
    c.submit("").unwrap();
    assert!(!c.document().entities.contains_key(&id));
    assert_eq!(c.document().entities.len(), 6);
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn shell_all_faces_failure_can_be_revised_or_canceled() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let keys = shape.display.faces.clone();
    let mut doc = Document::new("Shell invalid selection");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Shell").unwrap();
    c.submit("Thickness=1").unwrap();
    for key in keys {
        let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key), source: object.source, world_epoch: c.scene().unwrap().epoch };
        c.solid_extract_pick_face(reference).unwrap();
    }
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("Undo").unwrap();
    assert!(c.solid_face_needs_pick());
    c.submit("Undo").unwrap();
    assert_eq!(c.subselection().len(), 5);
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert!(c.subselection().is_empty());
}

#[test]
fn shell_stages_two_faces_and_commits_atomically() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let keys = [body.display.faces[0], body.display.faces[1]];
    let mut doc = Document::new("multi-face shell");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Shell").unwrap();
    assert!(c.submit("").is_err());
    assert!(c.submit("Thickness=0").is_err());
    for key in keys {
        let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key), source: object.source, world_epoch: c.scene().unwrap().epoch };
        c.solid_extract_pick_face(reference).unwrap();
    }
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("").unwrap();
    assert!(!c.solid_face_needs_pick());
    assert!(c.submit("0").is_err());
    assert_eq!(c.document(), &before);
    c.submit("1").unwrap();
    let Geometry::Brep(shell) = &c.document().entities[&id].geometry else { panic!() };
    let measure = om9_kernel::creation::measure(shell, c.coordinator().limits()).unwrap();
    assert!(measure.closed && (measure.volume_mm3 - 360.0).abs() < 1e-5, "{}", measure.volume_mm3);
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
}

#[test]
fn fillet_closed_edge_has_one_handle_and_can_commit() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Cylinder { radius_mm: 5.0, height_mm: 10.0 },
        0.01,
        &limits,
    ).unwrap();
    let index = (0..shape.display.edges.len()).find(|&i|
        om9_kernel::editing::oriented_edge_endpoints_or_closed(&shape, i, &limits).unwrap().is_none()
    ).unwrap();
    let edge = shape.display.edges[index];
    let mut doc = Document::new("closed fillet edge");
    let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    c.submit("NextRadius=1").unwrap();
    let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
    let reference = super::SelectedSubobject {
        root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepEdge(edge),
        source: object.source, world_epoch: c.scene().unwrap().epoch,
    };
    c.solid_fillet_pick_edge(reference).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.solid_fillet_handles().len(), 1);
    assert!(c.solid_fillet_show_radius());
    c.submit("ShowRadius=No").unwrap();
    assert!(!c.solid_fillet_show_radius());
    assert_eq!(c.solid_fillet_handles().len(), 1);
    assert!(c.submit("ShowRadius=invalid").is_err());
    assert!(!c.solid_fillet_show_radius());
    c.submit("ShowRadius=Yes").unwrap();
    assert!(c.solid_fillet_show_radius());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_variable_radius_on_selected_tangent_edges() {
    let limits = Limits::default();
    let profile = om9_kernel::creation::Profile::new(vec![
        [0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0], [0.0, 10.0, 0.0],
    ]).unwrap();
    let shape = om9_kernel::creation::extrude(&profile, [0.0, 0.0, 10.0], true, 0.01, &limits).unwrap();
    let selected: Vec<_> = (0..shape.display.edges.len()).filter_map(|index| {
        let (a, b) = om9_kernel::editing::oriented_edge_endpoints(&shape, index, &limits).ok()?;
        if a[1].abs() < 1e-6 && b[1].abs() < 1e-6 && (a[2]-10.0).abs() < 1e-6
            && (b[2]-10.0).abs() < 1e-6 && ((a[0]-b[0]).abs()-5.0).abs() < 1e-6 {
            Some((index, a, b))
        } else { None }
    }).collect();
    assert_eq!(selected.len(), 2);
    let mut doc = Document::new("tangent fillet");
    let entity = om9_core::model::Entity::new("split prism", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    c.submit("NextRadius=1").unwrap();
    for (index, _, _) in &selected {
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
        let edge = brep.display.edges[*index];
        let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepEdge(edge), source: object.source, world_epoch: c.scene().unwrap().epoch };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    c.submit("").unwrap();
    let outer = if (selected[0].1[0] - 5.0).abs() > 1e-6 { 0 } else { 1 };
    let shared = 1 - outer;
    let other_shared = 2 + if (selected[1].1[0] - 5.0).abs() < 1e-6 { 0 } else { 1 };
    assert!(c.solid_fillet_select_hit(&[(outer, 0.0), (shared, 0.1)]).is_err());
    assert_eq!(c.solid_fillet_current_radius(), None);
    c.solid_fillet_select_hit(&[(shared, 0.0), (other_shared, 0.1)]).unwrap();
    assert_eq!(c.solid_fillet_current_radius(), Some(1.0));
    c.solid_fillet_select_handle(outer).unwrap();
    c.submit("CurrentRadius=1.5").unwrap();
    c.solid_fillet_select_handle(shared).unwrap();
    c.submit("CurrentRadius=1.8").unwrap();
    let handles = c.solid_fillet_handles();
    let shared_radii: Vec<_> = handles.iter().filter(|item| (item.1[0]-5.0).abs() < 1e-6).map(|item| item.2).collect();
    assert_eq!(shared_radii, vec![1.8, 1.8]);
    let before_failure = c.solid_fillet_preview().unwrap().0.clone();
    assert!(c.submit("CurrentRadius=1000").is_err());
    assert_eq!(c.solid_fillet_current_radius(), Some(1.8));
    let shared_radii: Vec<_> = c.solid_fillet_handles().iter().filter(|item| (item.1[0]-5.0).abs() < 1e-6).map(|item| item.2).collect();
    assert_eq!(shared_radii, vec![1.8, 1.8]);
    assert_eq!(c.solid_fillet_preview().unwrap().0, &before_failure);
    assert_eq!(c.document(), &before);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_edge_stages_multiple_edges_preview_and_commit() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("staged fillet");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    assert!(c.submit("NextRadius=0").is_err());
    assert!(c.submit("").is_err());
    c.submit("NextRadius=1").unwrap();
    for index in [0, 2] {
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
            panic!()
        };
        let edge = brep.display.edges[index];
        let object = c
            .scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == id && o.path.is_empty())
            .unwrap();
        let reference = super::SelectedSubobject {
            root: id,
            path: vec![],
            subset: om9_scene::scene::Subelement::BrepEdge(edge),
            source: object.source,
            world_epoch: c.scene().unwrap().epoch,
        };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    assert_eq!(c.document(), &before);
    c.submit("NextRadius=2").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document(), &before);
    assert!(c.solid_fillet_preview().is_some());
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("Undo").unwrap();
    assert!(c.solid_fillet_preview().is_none());
    assert_eq!(c.document(), &before);
    c.submit("").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!()
    };
    let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(measure.closed && measure.volume_mm3 < 1000.0 && measure.volume_mm3 > 900.0);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_edge_captures_next_radius_for_each_selected_edge() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let vertical: Vec<_> = (0..shape.display.edges.len())
        .filter(|&index| {
            om9_kernel::editing::edge_endpoints(&shape, index).is_ok_and(|(a, b)| {
                (a[0] - b[0]).abs() < 1e-6
                    && (a[1] - b[1]).abs() < 1e-6
                    && (a[2] - b[2]).abs() > 9.0
            })
        })
        .collect();
    assert!(vertical.len() >= 2);
    let mut doc = Document::new("different fillet radii");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    for (index, radius) in [(vertical[0], 1.0), (vertical[1], 2.0)] {
        c.submit(&format!("NextRadius={radius}")).unwrap();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
            panic!()
        };
        let edge = brep.display.edges[index];
        let object = c
            .scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == id && o.path.is_empty())
            .unwrap();
        let reference = super::SelectedSubobject {
            root: id,
            path: vec![],
            subset: om9_scene::scene::Subelement::BrepEdge(edge),
            source: object.source,
            world_epoch: c.scene().unwrap().epoch,
        };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    c.submit("").unwrap();
    let (preview, _) = c.solid_fillet_preview().unwrap();
    let m = om9_kernel::creation::measure(preview, c.coordinator().limits()).unwrap();
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 50.0;
    assert!((m.volume_mm3 - expected).abs() < 1e-5, "{}", m.volume_mm3);
    assert_eq!(c.document(), &before);
    c.submit("SetAll=1.5").unwrap();
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
    let volume = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap().volume_mm3;
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 45.0;
    assert!((volume - expected).abs() < 1e-5);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_invalid_preview_keeps_document_and_redo() {
    let mut c = Controller::new(Document::new("invalid fillet"), Limits::default()).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.submit("Box 2 2 2").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    let redo_before = c.coordinator.redo_len();
    c.submit("FilletEdge").unwrap();
    c.submit("NextRadius=1000").unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
        panic!()
    };
    let edge = brep.display.edges[0];
    let object = c
        .scene()
        .unwrap()
        .objects
        .iter()
        .find(|o| o.root == id && o.path.is_empty())
        .unwrap();
    let reference = super::SelectedSubobject {
        root: id,
        path: vec![],
        subset: om9_scene::scene::Subelement::BrepEdge(edge),
        source: object.source,
        world_epoch: c.scene().unwrap().epoch,
    };
    c.solid_fillet_pick_edge(reference).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.redo_len(), redo_before);
    assert!(c.is_running());
    c.submit("Undo").unwrap();
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.redo_len(), redo_before);
}

#[test]
fn fillet_open_joined_shell_staged_and_numeric_paths() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let top = (0..body.display.faces.len())
        .find(|&index| {
            let points: Vec<_> = body
                .display
                .triangles
                .iter()
                .filter(|t| t[3] as usize == index)
                .flat_map(|t| t[..3].iter().map(|i| body.display.points[*i as usize][2]))
                .collect();
            !points.is_empty() && points.iter().all(|z| (*z - 10.0).abs() < 1e-9)
        })
        .unwrap();
    let shell = om9_kernel::editing::remove_face(&body, top, 0.01, &limits).unwrap();
    let edge = (0..shell.display.edges.len())
        .find(|&i| {
            om9_kernel::editing::edge_endpoints(&shell, i).is_ok_and(|(a, b)| {
                (a[0] - b[0]).abs() < 1e-6
                    && (a[1] - b[1]).abs() < 1e-6
                    && (a[2] - b[2]).abs() > 9.0
            })
        })
        .unwrap();
    for staged in [false, true] {
        let mut doc = Document::new("Open shell fillet");
        let entity = om9_core::model::Entity::new("shell", Geometry::Brep(shell.clone()), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        if staged {
            c.submit("FilletEdge").unwrap();
            c.submit("NextRadius=1").unwrap();
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
                panic!()
            };
            let object = c
                .scene()
                .unwrap()
                .objects
                .iter()
                .find(|o| o.root == id && o.path.is_empty())
                .unwrap();
            let reference = super::SelectedSubobject {
                root: id,
                path: vec![],
                subset: om9_scene::scene::Subelement::BrepEdge(brep.display.edges[edge]),
                source: object.source,
                world_epoch: c.scene().unwrap().epoch,
            };
            c.solid_fillet_pick_edge(reference).unwrap();
            c.submit("").unwrap();
            assert_eq!(c.document(), &before);
            c.submit("").unwrap();
        } else {
            c.select_objects(&[id]).unwrap();
            c.submit(&format!("FilletEdge {edge} 1")).unwrap();
        }
        let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
            panic!()
        };
        let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
        let expected =
            500.0 - 20.0 + 5.0 * std::f64::consts::PI - (1.0 - std::f64::consts::PI / 4.0);
        assert!(!measure.closed && (measure.area_mm2 - expected).abs() < 1e-5);
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn boolean_two_objects_selection_undo_and_retry_do_not_commit() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("Boolean2Objects").unwrap();
    assert!(c.solid_boolean_needs_object());
    assert!(c.submit("U").is_err());
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    assert!(c.solid_boolean_pick(ids[2], false).is_err());
    assert_eq!(c.selected_ordered(), vec![ids[0], ids[1]]);
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.solid_boolean_pick(ids[2], false).unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.cancel();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn boolean_two_objects_preview_cycles_all_results_before_commit() {
    let limits = Limits::default();
    let mut doc = Document::new("Boolean2 preview");
    let mut ids = Vec::new();
    for (width, x) in [(10.0, 0.0), (15.0, 5.0)] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: width,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
        entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("Boolean2Objects").unwrap();
    c.submit("").unwrap();
    for (name, expected) in [
        ("Union", 2000.0),
        ("Difference", 500.0),
        ("ReverseDifference", 1000.0),
        ("Intersection", 500.0),
        ("InversionIntersection", 1500.0),
    ] {
        let (body, mode) = c.solid_boolean_two_preview().unwrap();
        assert_eq!(mode, name);
        let volume = om9_kernel::creation::measure(body, c.coordinator().limits())
            .unwrap()
            .volume_mm3;
        assert!((volume - expected).abs() < 1e-6, "{name}: {volume}");
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("Next").unwrap();
    }
    assert_eq!(c.solid_boolean_two_preview().unwrap().1, "Union");
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    assert!(ids.iter().all(|id| c.document().entities.contains_key(id)));
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn boolean_two_objects_stale_preview_rejects_cycle_and_commit() {
    use om9_core::transaction::{Mutation, TransactionRequest};
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    c.submit("Boolean2Objects").unwrap();
    c.submit("").unwrap();
    let source = c.document().stamp(ids[0]).unwrap();
    let prepared = c
        .coordinator
        .prepare(TransactionRequest::new(
            "external edit",
            vec![Mutation::Placement {
                source,
                placement: om9_core::geometry::Transform::translation([1.0, 0.0, 0.0]),
            }],
        ))
        .unwrap();
    c.coordinator.commit(prepared).unwrap();
    c.refresh_scene().unwrap();
    let before = c.document().clone();
    let undo = c.coordinator.undo_len();
    assert!(matches!(c.submit("Next"), Err(om9_core::Error::Stale(_))));
    assert!(matches!(c.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), undo);
    c.cancel();
    assert!(c.solid_boolean_two_preview().is_none());
}

#[test]
fn boolean_two_objects_preview_undo_and_cancel_do_not_publish() {
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    let before = c.document().clone();
    c.submit("Boolean2Objects U").unwrap();
    assert!(c.solid_boolean_two_preview().is_some());
    assert_eq!(c.solid_boolean_two_hidden_sources(), Some([ids[0], ids[1]]));
    assert_eq!(c.document(), &before);
    c.submit("Undo").unwrap();
    assert!(c.solid_boolean_two_preview().is_none());
    assert!(c.solid_boolean_two_hidden_sources().is_none());
    assert!(c.solid_boolean_needs_object());
    c.submit("U").unwrap();
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert!(c.solid_boolean_two_preview().is_none());
    assert!(c.solid_boolean_two_hidden_sources().is_none());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
}

#[test]
fn boolean_two_objects_cycles_through_empty_preview_without_committing_it() {
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    let before = c.document().clone();
    c.submit("Boolean2Objects").unwrap();
    c.submit("").unwrap();
    for _ in 0..3 {
        c.submit("Next").unwrap();
    }
    assert_eq!(c.solid_boolean_two_preview_mode(), Some("Intersection"));
    assert!(c.solid_boolean_two_preview().is_none());
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("Next").unwrap();
    assert_eq!(
        c.solid_boolean_two_preview_mode(),
        Some("InversionIntersection")
    );
    assert!(c.solid_boolean_two_preview().is_some());
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
}

#[test]
fn boolean_two_objects_reverse_and_xor_have_measured_results_and_atomic_empty() {
    for (mode, second_side, offset, expected) in [
        ("R", 15.0, 5.0, Some(1000.0)),
        ("X", 15.0, 5.0, Some(1500.0)),
        ("InversionIntersection", 4.0, 3.0, Some(600.0)),
        ("Xor", 10.0, 30.0, Some(2000.0)),
        ("X", 10.0, 0.0, None),
    ] {
        let limits = Limits::default();
        let mut doc = Document::new("Boolean Two Objects additional results");
        let mut ids = Vec::new();
        for (index, (side, x)) in [(10.0, 0.0), (second_side, offset)].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: side,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        c.submit("Boolean2Objects").unwrap();
        c.submit("DeleteInput=Yes").unwrap();
        let result = c.submit(mode);
        if let Some(expected) = expected {
            result.unwrap();
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), 0);
            c.submit("").unwrap();
            let Geometry::Brep(body) = &c.document().entities.values().next().unwrap().geometry
            else {
                panic!("BRep result")
            };
            let measure = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(
                (measure.volume_mm3 - expected).abs() < 1e-6,
                "{mode}: {}",
                measure.volume_mm3
            );
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
            c.redo().unwrap();
        } else {
            result.unwrap();
            assert!(c.solid_boolean_two_preview().is_none());
            assert_eq!(
                c.solid_boolean_two_preview_mode(),
                Some("InversionIntersection")
            );
            assert!(c.submit("").is_err());
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), 0);
            assert!(c.is_running());
        }
    }
}

fn difference_group_controller() -> (Controller, Vec<om9_core::identity::ObjectId>) {
    let limits = Limits::default();
    let mut doc = Document::new("Boolean Difference groups");
    let mut ids = Vec::new();
    for (index, (side, placement)) in [
        (10.0, [0.0, 0.0, 0.0]),
        (10.0, [20.0, 0.0, 0.0]),
        (2.0, [1.0, 1.0, 1.0]),
        (2.0, [21.0, 1.0, 1.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let brep = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: side,
                y_mm: side,
                z_mm: side,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(brep), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        entity.placement = om9_core::geometry::Transform::translation(placement);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    (Controller::new(doc, limits).unwrap(), ids)
}

#[test]
fn difference_fuses_overlapping_groups_and_honors_delete_input() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    for delete_input in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Overlapping Boolean sets");
        let mut ids = Vec::new();
        for (index, (side, placement)) in [
            (10.0, [0.0, 0.0, 0.0]),
            (10.0, [5.0, 0.0, 0.0]),
            (4.0, [6.0, 1.0, 1.0]),
            (4.0, [8.0, 1.0, 1.0]),
        ]
        .into_iter()
        .enumerate()
        {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: side,
                    y_mm: side,
                    z_mm: side,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation(placement);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids[..2]).unwrap();
        if delete_input {
            c.submit("BooleanDifference").unwrap();
        } else {
            c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap();
        }
        assert!(c.is_running());
        assert_eq!(c.selected_ordered(), ids[..2]);
        let option = c
            .command_options()
            .into_iter()
            .find(|o| o.id.name() == "DeleteInput")
            .unwrap();
        assert_eq!(option.value, CommandValue::Flag(false));
        assert_eq!(option.shortcut, Some('D'));
        assert!(c.submit("DeleteInput=invalid").is_err());
        assert_eq!(
            c.command_options()
                .into_iter()
                .find(|o| o.id.name() == "DeleteInput")
                .unwrap()
                .value,
            CommandValue::Flag(false)
        );
        if delete_input {
            let id = c.resolve_command_option("d").unwrap();
            c.activate_command_option(id).unwrap();
            assert_eq!(
                c.command_options()
                    .into_iter()
                    .find(|o| o.id.name() == "DeleteInput")
                    .unwrap()
                    .value,
                CommandValue::Flag(true)
            );
        }
        c.submit("").unwrap();
        for id in &ids[2..] {
            c.solid_difference_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
        for id in &ids[..2] {
            assert!(!c.document().entities.contains_key(id));
        }
        for id in &ids[2..] {
            assert_eq!(c.document().entities.contains_key(id), !delete_input);
        }
        let output = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &output.geometry else {
            panic!("BRep result")
        };
        let measures = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(measures.solids, 1);
        assert!(
            (measures.volume_mm3 - 1404.0).abs() < 1e-6,
            "{}",
            measures.volume_mm3
        );
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 4);
        c.redo().unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
    }
}

#[test]
fn difference_selects_target_and_cutter_groups_through_menu_and_cmd() {
    for menu in [true, false] {
        let (mut c, ids) = difference_group_controller();
        if menu {
            c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("BooleanDifference").unwrap();
        }
        assert!(c.solid_difference_needs_object());
        assert!(c.prompt().contains("subtract from"));
        assert!(c.submit("").is_err());
        for id in &ids[..2] {
            c.solid_difference_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert!(c.prompt().contains("Select objects to subtract ("));
        assert!(c.solid_difference_pick(ids[0], false).is_err());
        assert!(c.submit("").is_err());
        for id in &ids[2..] {
            c.solid_difference_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert!(!c.is_running());
        assert_eq!(c.document().entities.len(), 3);
        assert!(c.document().entities.contains_key(&ids[2]));
        assert!(c.document().entities.contains_key(&ids[3]));
        let entity = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &entity.geometry else {
            panic!("BRep output")
        };
        let measures = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert!((measures.volume_mm3 - 1984.0).abs() < 1e-6);
        assert_eq!(measures.solids, 2);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 4);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 3);
    }
}

#[test]
fn difference_cancel_after_target_selection_leaves_document_unchanged() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanDifference").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.cancel();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn difference_uses_one_preselected_object_as_target() {
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("BooleanDifference").unwrap();
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
}

#[test]
fn difference_inline_option_cannot_bypass_selection_stages() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    let before = c.document().clone();
    assert!(c.dispatch_solid_args("OM9-SOLID-001", &[]).is_err());
    c.submit("BooleanDifference DeleteInput=Yes").unwrap();
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.document(), &before);
    assert_eq!(c.selected_ordered(), ids[..2]);
    assert_eq!(
        c.command_options()
            .into_iter()
            .find(|o| o.id.name() == "DeleteInput")
            .unwrap()
            .value,
        CommandValue::Flag(true)
    );
    c.cancel();
    assert!(c.submit("BooleanDifference unexpected").is_err());
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn difference_empty_result_refuses_without_deleting_inputs() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanDifference").unwrap();
    c.submit("DeleteInput=Yes").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.submit("").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert!(c.solid_difference_needs_object());
}

#[test]
fn difference_pending_undo_revises_selection_and_restores_target_stage() {
    use crate::command_options::CommandOptionInterface;
    let (mut c, ids) = difference_group_controller();
    c.submit("BooleanDifference").unwrap();
    assert_eq!(c.active_feature(), Some("OM9-SOLID-001"));
    c.solid_difference_pick(ids[0], false).unwrap();
    c.solid_difference_pick(ids[1], false).unwrap();
    let undo = c.resolve_command_option("u").expect("Undo shortcut");
    c.activate_command_option(undo).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.submit("Undo").unwrap();
    assert!(c.selected().is_empty());
    c.submit("Undo").unwrap();
    assert!(c.prompt().contains("subtract from"));
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    let cancel = c.resolve_command_option("Cancel").expect("Cancel option");
    c.activate_command_option(cancel).unwrap();
    assert!(!c.is_running());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn difference_rejects_stale_target_before_publication() {
    use om9_core::transaction::{Mutation, TransactionRequest};
    let (mut c, ids) = difference_group_controller();
    c.submit("BooleanDifference").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    let source = c.document().stamp(ids[0]).unwrap();
    let prepared = c
        .coordinator
        .prepare(TransactionRequest::new(
            "external edit",
            vec![Mutation::Placement {
                source,
                placement: om9_core::geometry::Transform::translation([1.0, 0.0, 0.0]),
            }],
        ))
        .unwrap();
    c.coordinator.commit(prepared).unwrap();
    c.refresh_scene().unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    assert!(matches!(c.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    assert!(c.solid_difference_needs_object());
    c.cancel();
}

#[test]
fn difference_late_open_cutter_failure_is_atomic_and_retryable() {
    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let entity = doc.entities.get_mut(&ids[3]).unwrap();
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep cutter")
    };
    let open = om9_kernel::editing::remove_face(body, 0, 0.01, &Limits::default()).unwrap();
    entity.geometry = Geometry::Brep(open);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    assert_eq!(c.coordinator().redo_len(), 1);
    let before = c.document().clone();
    c.submit("BooleanDifference").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    c.solid_difference_pick(ids[1], false).unwrap();
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.solid_difference_pick(ids[3], false).unwrap();
    assert!(c.submit("").is_err());
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
    c.solid_difference_pick(ids[3], true).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}
