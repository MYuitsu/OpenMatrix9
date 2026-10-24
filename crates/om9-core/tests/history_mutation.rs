// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    history::{EvaluationState, HistoryPolicy, Recipe, ReplayEnvironment, Tolerance},
    identity::*,
    model::{Document, Entity},
    request::{Selection, SourceBinding},
    transaction::*,
};

fn recipe_for(doc: &Document, command: &str, input: ObjectId, output: ObjectId) -> Recipe {
    let mut recipe = Recipe::new(
        doc.id,
        command,
        vec![SourceBinding {
            role: "input".into(),
            source: doc.stamp(input).unwrap(),
            selection: Selection::Whole,
        }],
        vec![output],
        ReplayEnvironment {
            command_version: 1,
            solver: "om9-test".into(),
            solver_version: "0.1.0".into(),
            schema_version: 1,
            length_unit: om9_core::geometry::Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance::om9_template(),
        },
    );
    recipe.state = EvaluationState::Pending;
    recipe.policy = HistoryPolicy {
        record: true,
        update: true,
        lock: false,
    };
    recipe
}

#[test]
fn history_recipe_inserts_and_undoes_with_its_output_in_one_transaction() {
    let limits = Limits::default();
    let mut doc = Document::new("history mutation");
    let source = Entity::new("source", Geometry::Point([0.0, 0.0, 0.0]), None);
    let source_id = source.id;
    doc.entities.insert(source_id, source);
    let output = Entity::new("output", Geometry::Point([1.0, 2.0, 3.0]), None);
    let output_id = output.id;
    let recipe = recipe_for(&doc, "TestHistory", source_id, output_id);
    let recipe_id = recipe.id;
    let mut c =
        TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let document = c.world().documents.keys().next().copied().unwrap();
    let expected = c.world().documents[&document].revision;
    let request = TransactionRequest::new(
        "HistoryTest",
        vec![
            Mutation::Insert {
                document,
                entity: output,
            },
            Mutation::History {
                document,
                expected_revision: expected,
                recipe: Box::new(recipe),
            },
        ],
    );
    let prepared = c.prepare(request).unwrap();
    c.commit(prepared).unwrap();
    assert!(
        c.world().documents[&document]
            .entities
            .contains_key(&output_id)
    );
    assert!(
        c.world().documents[&document]
            .history
            .recipes
            .contains_key(&recipe_id)
    );
    c.undo().unwrap();
    assert!(
        !c.world().documents[&document]
            .entities
            .contains_key(&output_id)
    );
    assert!(
        !c.world().documents[&document]
            .history
            .recipes
            .contains_key(&recipe_id)
    );
    c.redo().unwrap();
    assert!(
        c.world().documents[&document]
            .entities
            .contains_key(&output_id)
    );
    assert!(
        c.world().documents[&document]
            .history
            .recipes
            .contains_key(&recipe_id)
    );
}
