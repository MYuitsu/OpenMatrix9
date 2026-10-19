// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-04 wave W7: associative dependency maintenance over `HistoryGraph`.
//!
//! Oracles: a changed source makes its recipe stale, dirtiness reaches every consumer in
//! topological order, a missing source is `Unresolved` rather than dirty, detached recipes
//! are skipped, and the recompute plan contains only what can run.
use om9_core::{
    Error,
    geometry::{Curve, Geometry, Transform, Unit},
    history::{EvaluationState, HistoryGraph, HistoryPolicy, Recipe, ReplayEnvironment, Tolerance},
    history_ops::{
        detach, ensure_document, freeze, mark_clean, mark_failed, mark_stale, plan_recompute,
        stale_recipes, state_counts,
    },
    identity::{DocumentId, HistoryId, ObjectId},
    model::{Document, Entity, Layer},
    request::SourceBinding,
};

fn environment() -> ReplayEnvironment {
    ReplayEnvironment {
        command_version: 1,
        solver: "om9".into(),
        solver_version: "1".into(),
        schema_version: 1,
        length_unit: Unit::Millimeter,
        frame: Transform::identity(),
        tolerance: Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap(),
    }
}

fn binding(document: &Document, id: ObjectId, role: &str) -> SourceBinding {
    SourceBinding {
        role: role.into(),
        source: document.stamp(id).unwrap(),
        selection: om9_core::request::Selection::Whole,
    }
}

struct Fixture {
    document: Document,
    graph: HistoryGraph,
    source: ObjectId,
    middle: ObjectId,
    top: ObjectId,
    first: HistoryId,
    second: HistoryId,
}

/// A two-step chain: `source --first--> middle --second--> top`.
fn fixture() -> Fixture {
    let mut document = Document::new("associative");
    let layer = Layer::new("Metals", [1, 2, 3, 255]);
    document.layers.insert(layer.id, layer.clone());
    document.layer_order.push(layer.id);
    document.current_layer = Some(layer.id);
    // Collect the ids from the values before inserting: a BTreeMap order is not insertion
    // order, and the chain depends on which id is which.
    let source_entity = Entity::new("Source", Geometry::Point([0.0; 3]), Some(layer.id));
    let middle_entity = Entity::new("Middle", Geometry::Point([0.0; 3]), Some(layer.id));
    let top_entity = Entity::new("Top", Geometry::Point([0.0; 3]), Some(layer.id));
    let (source, middle, top) = (source_entity.id, middle_entity.id, top_entity.id);
    for entity in [source_entity, middle_entity, top_entity] {
        document.entities.insert(entity.id, entity);
    }
    let mut graph = HistoryGraph::default();
    let first = Recipe::new(
        document.id,
        "Ring",
        vec![binding(&document, source, "profile")],
        vec![middle],
        environment(),
    );
    let second = Recipe::new(
        document.id,
        "Prong",
        vec![binding(&document, middle, "body")],
        vec![top],
        environment(),
    );
    let (first_id, second_id) = (first.id, second.id);
    graph.recipes.insert(first_id, first);
    graph.recipes.insert(second_id, second);
    Fixture {
        document,
        graph,
        source,
        middle,
        top,
        first: first_id,
        second: second_id,
    }
}

fn state_of(graph: &HistoryGraph, id: HistoryId) -> EvaluationState {
    graph.recipes[&id].state.clone()
}

#[test]
fn a_fresh_graph_reports_no_stale_recipe() {
    let fixture = fixture();
    assert!(stale_recipes(&fixture.document, &fixture.graph).is_empty());
    assert!(plan_recompute(&fixture.graph).unwrap().is_empty());
    // The chain is ordered: the producer before the consumer.
    assert_eq!(
        fixture.graph.execution_order().unwrap(),
        vec![fixture.first, fixture.second]
    );
    // The consumer owns the last object in the chain.
    assert_eq!(fixture.graph.owner(fixture.top), Some(fixture.second));
}

#[test]
fn a_changed_source_dirties_its_recipe_and_every_consumer() {
    let mut fixture = fixture();
    // Change the source geometry revision: the recorded stamp no longer matches.
    let source = fixture.document.entities.get_mut(&fixture.source).unwrap();
    source.geometry_revision += 1;
    let stale = stale_recipes(&fixture.document, &fixture.graph);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].id, fixture.first);
    assert_eq!(stale[0].reason, "source changed");
    let changed = mark_stale(&fixture.document, &mut fixture.graph).unwrap();
    assert_eq!(changed, vec![fixture.first, fixture.second]);
    assert_eq!(
        state_of(&fixture.graph, fixture.first),
        EvaluationState::Dirty("source changed".into())
    );
    assert_eq!(
        state_of(&fixture.graph, fixture.second),
        EvaluationState::Dirty("downstream of dirty recipe".into())
    );
    // Both are planned, producer first.
    assert_eq!(
        plan_recompute(&fixture.graph).unwrap(),
        vec![fixture.first, fixture.second]
    );
    // A second pass changes nothing.
    assert!(
        mark_stale(&fixture.document, &mut fixture.graph)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_missing_source_is_unresolved_not_dirty() {
    let mut fixture = fixture();
    fixture.document.entities.remove(&fixture.source);
    let stale = stale_recipes(&fixture.document, &fixture.graph);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].reason, "source missing");
    let changed = mark_stale(&fixture.document, &mut fixture.graph).unwrap();
    assert_eq!(changed, vec![fixture.first, fixture.second]);
    assert_eq!(
        state_of(&fixture.graph, fixture.first),
        EvaluationState::Unresolved("source missing".into())
    );
    // The consumer is downstream dirty, and an unresolved recipe is not planned.
    assert_eq!(
        state_of(&fixture.graph, fixture.second),
        EvaluationState::Dirty("downstream of dirty recipe".into())
    );
    assert_eq!(
        plan_recompute(&fixture.graph).unwrap(),
        vec![fixture.second]
    );
}

#[test]
fn a_detached_recipe_is_never_stale_and_blocks_nothing() {
    let mut fixture = fixture();
    detach(&mut fixture.graph, fixture.first).unwrap();
    assert_eq!(
        state_of(&fixture.graph, fixture.first),
        EvaluationState::Detached
    );
    assert!(!fixture.graph.recipes[&fixture.first].policy.record);
    assert!(!fixture.graph.recipes[&fixture.first].policy.update);
    // Detaching twice is idempotent, and the output is no longer owned by the graph.
    detach(&mut fixture.graph, fixture.first).unwrap();
    assert_eq!(fixture.graph.owner(fixture.middle), None);
    // A changed source does not make the detached recipe stale.
    let source = fixture.document.entities.get_mut(&fixture.source).unwrap();
    source.geometry_revision += 1;
    let stale = stale_recipes(&fixture.document, &fixture.graph);
    assert!(stale.is_empty(), "{stale:?}");
    assert!(
        mark_stale(&fixture.document, &mut fixture.graph)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn mark_clean_advances_the_revision_and_clears_the_state() {
    let mut fixture = fixture();
    mark_clean(&mut fixture.graph, fixture.first).unwrap();
    assert_eq!(
        state_of(&fixture.graph, fixture.first),
        EvaluationState::Clean
    );
    assert_eq!(fixture.graph.recipes[&fixture.first].revision, 2);
    // A failed recompute keeps its reason and is not planned.
    mark_failed(&mut fixture.graph, fixture.second, "solution diverged").unwrap();
    assert_eq!(
        state_of(&fixture.graph, fixture.second),
        EvaluationState::Failed("solution diverged".into())
    );
    assert!(plan_recompute(&fixture.graph).unwrap().is_empty());
    assert_eq!(
        mark_failed(&mut fixture.graph, fixture.second, ""),
        Err(Error::Invalid("history failure reason"))
    );
    assert_eq!(
        mark_clean(&mut fixture.graph, HistoryId::new()),
        Err(Error::Missing("history recipe"))
    );
}

#[test]
fn freezing_a_recipe_locks_its_outputs() {
    let mut fixture = fixture();
    freeze(&mut fixture.graph, fixture.first, true).unwrap();
    assert!(fixture.graph.recipes[&fixture.first].policy.lock);
    assert!(fixture.graph.locked(fixture.middle));
    freeze(&mut fixture.graph, fixture.first, false).unwrap();
    assert!(!fixture.graph.locked(fixture.middle));
    // A detached recipe cannot be frozen.
    detach(&mut fixture.graph, fixture.second).unwrap();
    assert_eq!(
        freeze(&mut fixture.graph, fixture.second, true),
        Err(Error::Invalid("detached recipe freeze"))
    );
}

#[test]
fn the_state_counts_match_the_recorded_states() {
    let mut fixture = fixture();
    assert_eq!(state_counts(&fixture.graph)["clean"], 2);
    let source = fixture.document.entities.get_mut(&fixture.source).unwrap();
    source.geometry_revision += 1;
    mark_stale(&fixture.document, &mut fixture.graph).unwrap();
    let counts = state_counts(&fixture.graph);
    assert_eq!(counts["dirty"], 2);
    assert!(!counts.contains_key("clean"));
    // A graph from another document is a programming error, not a user state.
    assert_eq!(
        ensure_document(&fixture.graph, DocumentId::new()),
        Err(Error::Invalid("recipe document mismatch"))
    );
    ensure_document(&fixture.graph, fixture.document.id).unwrap();
    // The curve helper is used by the fixtures through the geometry layer.
    let _ = Curve::Line {
        start: [0.0; 3],
        end: [1.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let _ = HistoryPolicy {
        record: true,
        update: true,
        lock: false,
    };
}
