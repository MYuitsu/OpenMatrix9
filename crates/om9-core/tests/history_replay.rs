// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    history::*,
    identity::*,
    model::{Document, Entity},
    request::{Selection, SourceBinding},
    result::*,
    transaction::*,
};
fn fixture() -> (TransactionCoordinator, DocumentId, ObjectId) {
    let mut d = Document::new("history");
    let e = Entity::new("A", Geometry::Point([1., 2., 3.]), d.current_layer);
    let a = e.id;
    d.entities.insert(a, e);
    let id = d.id;
    (
        TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        id,
        a,
    )
}
fn result(
    c: &TransactionCoordinator,
    d: DocumentId,
    source: ObjectId,
    label: &str,
    p: [f64; 3],
) -> PreparedResult {
    let document = &c.world().documents[&d];
    let binding = SourceBinding {
        role: "Point".into(),
        source: document.stamp(source).unwrap(),
        selection: Selection::Whole,
    };
    let output = Entity::new(label, Geometry::Point(p), document.current_layer);
    let id = output.id;
    let environment = ReplayEnvironment {
        command_version: 1,
        solver: "rust-test-fixture".into(),
        solver_version: "1".into(),
        schema_version: 1,
        length_unit: om9_core::geometry::Unit::Millimeter,
        frame: Transform::identity(),
        tolerance: Tolerance {
            absolute_mm: 0.001,
            relative: 0.000001,
            angular_rad: 0.00001,
        },
    };
    let recipe = Recipe::new(
        d,
        "fixture-point",
        vec![binding.clone()],
        vec![id],
        environment,
    );
    PreparedResult {
        document: d,
        operation: "fixture-point".into(),
        sources: vec![binding.clone()],
        outputs: vec![PreparedOutput {
            entity: output.clone(),
            expected_existing: None,
            geometry_snapshot: output.geometry.clone(),
            geometry_result: output.geometry.clone(),
            source_stamps: vec![binding.source],
            conversion: ConversionReport::identity(output.geometry.kind()),
        }],
        recipe: Some(recipe),
    }
}
fn publish(c: &mut TransactionCoordinator, p: PreparedResult) -> ObjectId {
    let id = p.outputs[0].entity.id;
    let prepared = c
        .prepare(TransactionRequest::new(
            "solver",
            vec![Mutation::Publish(Box::new(p))],
        ))
        .unwrap();
    c.commit(prepared).unwrap();
    id
}

#[test]
fn geometry_result_recipe_and_metadata_publish_in_one_undo_group() {
    let (mut c, d, a) = fixture();
    let initial = c.world().clone();
    let p = result(&c, d, a, "B", [2., 4., 6.]);
    let recipe = p.recipe.as_ref().unwrap().id;
    let b = publish(&mut c, p);
    let document = &c.world().documents[&d];
    assert_eq!(
        document.history.recipes[&recipe].state,
        EvaluationState::Clean
    );
    assert_eq!(document.results[&b].status, ResultStatus::Current);
    assert_eq!(c.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.world(), &initial);
    c.redo().unwrap();
    assert!(c.world().documents[&d].results.contains_key(&b));
}
#[test]
fn mismatch_between_shape_snapshot_result_and_source_stamp_rejects_all_outputs() {
    let (c, d, a) = fixture();
    let before = c.snapshot();
    for case in 0..3 {
        let mut p = result(&c, d, a, "bad", [2., 4., 6.]);
        match case {
            0 => p.outputs[0].geometry_snapshot = Geometry::Point([9., 9., 9.]),
            1 => p.outputs[0].geometry_result = Geometry::Point([8., 8., 8.]),
            _ => p.outputs[0].source_stamps[0].geometry_revision += 1,
        };
        assert!(
            c.prepare(TransactionRequest::new(
                "mismatch",
                vec![Mutation::Publish(Box::new(p))]
            ))
            .is_err()
        );
        assert_eq!(c.snapshot(), before);
    }
}
#[test]
fn source_change_marks_two_generations_dirty_and_old_results_stale() {
    let (mut c, d, a) = fixture();
    let p = result(&c, d, a, "B", [2., 4., 6.]);
    let first = p.recipe.as_ref().unwrap().id;
    let b = publish(&mut c, p);
    let p = result(&c, d, b, "C", [4., 8., 12.]);
    let second = p.recipe.as_ref().unwrap().id;
    let child = publish(&mut c, p);
    let source = c.world().documents[&d].stamp(a).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "edit parent",
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Point([10., 20., 30.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert!(matches!(
        doc.history.recipes[&first].state,
        EvaluationState::Dirty(_)
    ));
    assert!(matches!(
        doc.history.recipes[&second].state,
        EvaluationState::Dirty(_)
    ));
    assert_eq!(doc.results[&b].status, ResultStatus::Stale);
    assert_eq!(doc.results[&child].status, ResultStatus::Stale);
    assert_eq!(doc.entities[&b].geometry, Geometry::Point([2., 4., 6.]));
    c.undo().unwrap();
    assert_eq!(
        c.world().documents[&d].history.recipes[&first].state,
        EvaluationState::Clean
    );
    c.redo().unwrap();
    assert!(matches!(
        c.world().documents[&d].history.recipes[&second].state,
        EvaluationState::Dirty(_)
    ));
}
#[test]
fn missing_parent_is_unresolved_without_rebinding_same_label() {
    let (mut c, d, a) = fixture();
    let p = result(&c, d, a, "B", [2., 4., 6.]);
    let recipe = p.recipe.as_ref().unwrap().id;
    publish(&mut c, p);
    let source = c.world().documents[&d].stamp(a).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "delete",
            vec![Mutation::Delete { source }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(matches!(
        c.world().documents[&d].history.recipes[&recipe].state,
        EvaluationState::Unresolved(_)
    ));
    let e = Entity::new(
        "A",
        Geometry::Point([1., 2., 3.]),
        c.world().documents[&d].current_layer,
    );
    let p = c
        .prepare(TransactionRequest::new(
            "same label",
            vec![Mutation::Insert {
                document: d,
                entity: e,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(matches!(
        c.world().documents[&d].history.recipes[&recipe].state,
        EvaluationState::Unresolved(_)
    ));
}
#[test]
fn replay_preserves_child_identity_and_current_source_revision() {
    let (mut c, d, a) = fixture();
    let first = result(&c, d, a, "B", [2., 4., 6.]);
    let recipe = first.recipe.as_ref().unwrap().id;
    let b = publish(&mut c, first);
    let source = c.world().documents[&d].stamp(a).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "edit",
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Point([10., 20., 30.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let mut replay = result(&c, d, a, "B", [20., 40., 60.]);
    replay.recipe.as_mut().unwrap().id = recipe;
    replay.recipe.as_mut().unwrap().outputs = vec![b];
    replay.outputs[0].entity.id = b;
    replay.outputs[0].entity.generation = c.world().documents[&d].entities[&b].generation;
    replay.outputs[0].expected_existing = Some(c.world().documents[&d].stamp(b).unwrap());
    publish(&mut c, replay);
    let doc = &c.world().documents[&d];
    assert_eq!(doc.entities[&b].geometry, Geometry::Point([20., 40., 60.]));
    assert_eq!(doc.history.recipes[&recipe].state, EvaluationState::Clean);
    assert_eq!(doc.results[&b].source_stamps[0].geometry_revision, 2);
    assert_eq!(doc.history.recipes[&recipe].outputs, vec![b]);
}
#[test]
fn cycle_is_rejected_before_graph_or_shape_mutation() {
    let (mut c, d, a) = fixture();
    let first = result(&c, d, a, "B", [2., 4., 6.]);
    let b = publish(&mut c, first);
    let before = c.snapshot();
    let mut p = result(&c, d, b, "A", [3., 6., 9.]);
    p.outputs[0].entity.id = a;
    p.outputs[0].expected_existing = Some(c.world().documents[&d].stamp(a).unwrap());
    p.recipe.as_mut().unwrap().outputs = vec![a];
    assert!(
        c.prepare(TransactionRequest::new(
            "cycle",
            vec![Mutation::Publish(Box::new(p))]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
}
#[test]
fn history_child_geometry_lock_does_not_block_reference_input_or_appearance() {
    let (mut c, d, a) = fixture();
    let mut p = result(&c, d, a, "B", [2., 4., 6.]);
    p.recipe.as_mut().unwrap().policy.lock = true;
    let b = publish(&mut c, p);
    let source = c.world().documents[&d].stamp(b).unwrap();
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "edit child",
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Point([5., 5., 5.])
            }]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
    let mut attributes = c.world().documents[&d].entities[&b].attributes.clone();
    attributes.color = Some([255, 0, 0, 255]);
    let p = c
        .prepare(TransactionRequest::new(
            "color",
            vec![Mutation::Attributes { source, attributes }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(c.world().documents[&d].stamp(b).is_ok());
}
