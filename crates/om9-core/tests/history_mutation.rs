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
                published: vec![],
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

use om9_core::Error;

fn published_for(
    source_stamps: Vec<SourceStamp>,
    recipe: HistoryId,
    geometry: Geometry,
    revision: u64,
) -> om9_core::result::PublishedResult {
    om9_core::result::PublishedResult {
        operation: "TestHistory".into(),
        geometry_revision: revision,
        geometry_snapshot: geometry.clone(),
        source_stamps,
        conversion: om9_core::result::ConversionReport::identity(geometry.kind()),
        recipe: Some(recipe),
        status: om9_core::result::ResultStatus::Current,
    }
}

#[test]
fn a_recompute_is_one_transaction_with_geometry_recipe_and_result_and_undoes_together() {
    let limits = Limits::default();
    let mut doc = Document::new("recompute mutation");
    let source = Entity::new("source", Geometry::Point([0.0, 0.0, 0.0]), None);
    let source_id = source.id;
    doc.entities.insert(source_id, source);
    let output = Entity::new("output", Geometry::Point([1.0, 2.0, 3.0]), None);
    let output_id = output.id;
    doc.entities.insert(output_id, output.clone());
    let mut recipe = recipe_for(&doc, "TestHistory", source_id, output_id);
    recipe.state = EvaluationState::Clean;
    let recipe_id = recipe.id;
    doc.results.insert(
        output_id,
        published_for(
            vec![doc.stamp(source_id).unwrap()],
            recipe_id,
            output.geometry.clone(),
            1,
        ),
    );
    doc.history.recipes.insert(recipe_id, recipe.clone());
    let mut c =
        TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let document = c.world().documents.keys().next().copied().unwrap();
    let base = &c.world().documents[&document];
    let expected = base.revision;
    let next_geometry = Geometry::Point([4.0, 5.0, 6.0]);
    let next_revision = base.entities[&output_id].geometry_revision + 1;
    let mut updated = recipe;
    updated.revision += 1;
    let result = published_for(
        vec![base.stamp(source_id).unwrap()],
        recipe_id,
        next_geometry.clone(),
        next_revision,
    );
    let request = TransactionRequest::new(
        "RecomputeHistory",
        vec![
            Mutation::Geometry {
                source: base.stamp(output_id).unwrap(),
                geometry: next_geometry.clone(),
            },
            Mutation::HistoryUpdate {
                document,
                expected_revision: expected,
                recipes: vec![Box::new(updated)],
                published: vec![(output_id, result)],
            },
        ],
    );
    let prepared = c.prepare(request).unwrap();
    c.commit(prepared).unwrap();
    let after = &c.world().documents[&document];
    assert_eq!(after.entities[&output_id].geometry, next_geometry);
    assert_eq!(after.entities[&output_id].geometry_revision, next_revision);
    assert_eq!(after.history.recipes[&recipe_id].revision, 2);
    assert_eq!(after.results[&output_id].geometry_revision, next_revision);
    assert_eq!(after.results[&output_id].geometry_snapshot, next_geometry);
    c.undo().unwrap();
    let undone = &c.world().documents[&document];
    assert_eq!(undone.entities[&output_id].geometry, output.geometry);
    assert_eq!(undone.history.recipes[&recipe_id].revision, 1);
    assert_eq!(undone.results[&output_id].geometry_revision, 1);
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&document].entities[&output_id].geometry,
        next_geometry
    );
}

#[test]
fn a_history_update_is_refused_for_a_missing_recipe_or_a_foreign_result() {
    let limits = Limits::default();
    let mut doc = Document::new("recompute refusals");
    let source = Entity::new("source", Geometry::Point([0.0, 0.0, 0.0]), None);
    let source_id = source.id;
    doc.entities.insert(source_id, source);
    let output = Entity::new("output", Geometry::Point([1.0, 2.0, 3.0]), None);
    let output_id = output.id;
    doc.entities.insert(output_id, output.clone());
    let mut recipe = recipe_for(&doc, "TestHistory", source_id, output_id);
    recipe.state = EvaluationState::Clean;
    let recipe_id = recipe.id;
    doc.history.recipes.insert(recipe_id, recipe.clone());
    let c = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let document = c.world().documents.keys().next().copied().unwrap();
    let base = &c.world().documents[&document];
    let expected = base.revision;
    // A recipe id that does not exist cannot be "updated".
    let mut missing = recipe.clone();
    missing.id = HistoryId::new();
    assert_eq!(
        c.prepare(TransactionRequest::new(
            "Missing",
            vec![Mutation::HistoryUpdate {
                document,
                expected_revision: expected,
                recipes: vec![Box::new(missing)],
                published: vec![],
            }],
        ))
        .unwrap_err(),
        Error::Missing("history recipe")
    );
    // A published result must belong to an output of one of the updated recipes.
    let foreign = Entity::new("foreign", Geometry::Point([7.0, 8.0, 9.0]), None);
    let foreign_id = foreign.id;
    let result = published_for(
        vec![base.stamp(source_id).unwrap()],
        recipe_id,
        output.geometry.clone(),
        1,
    );
    assert_eq!(
        c.prepare(TransactionRequest::new(
            "Foreign",
            vec![Mutation::HistoryUpdate {
                document,
                expected_revision: expected,
                recipes: vec![Box::new(recipe.clone())],
                published: vec![(foreign_id, result)],
            }],
        ))
        .unwrap_err(),
        Error::Invalid("history result output")
    );
    // A stale document revision is refused before any mutation runs.
    assert_eq!(
        c.prepare(TransactionRequest::new(
            "Stale",
            vec![Mutation::HistoryUpdate {
                document,
                expected_revision: expected + 1,
                recipes: vec![Box::new(recipe)],
                published: vec![],
            }],
        ))
        .unwrap_err(),
        Error::Stale("history document revision")
    );
}
