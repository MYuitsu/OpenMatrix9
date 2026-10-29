// SPDX-License-Identifier: LGPL-2.1-or-later
//! Broken-link report and single-object detach over the document history graph
//! (`OM9-L3-03-13-003`, `-004`): the rows name the missing role, a detached recipe leaves
//! the other feature alone, and a frozen recipe refuses detach.
use om9_core::{
    Error,
    geometry::{Curve, Geometry, Transform, Unit},
    history::{EvaluationState, HistoryPolicy, Recipe, ReplayEnvironment, Tolerance},
    history_ops,
    identity::{HistoryId, ObjectId},
    model::{Document, Entity, Layer},
    request::{Selection, SourceBinding},
};

fn rail_curve() -> Geometry {
    Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 9.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn profile_curve() -> Geometry {
    Geometry::Curve(Curve::Line {
        start: [9.0, -1.0, 0.0],
        end: [9.0, 1.0, 0.0],
        domain: [0.0, 1.0],
    })
}

/// A document with `rail` and `profile`, plus one recipe `profile <- rail`.
fn fixture() -> (Document, ObjectId, ObjectId, HistoryId) {
    let mut document = Document::new("history-links");
    let layer = Layer::new("Rails", [200, 180, 120, 255]);
    let layer_id = layer.id;
    document.layers.insert(layer_id, layer);
    document.layer_order.push(layer_id);
    let rail = Entity::new("Rail", rail_curve(), Some(layer_id));
    let profile = Entity::new("Profile", profile_curve(), Some(layer_id));
    let (rail_id, profile_id) = (rail.id, profile.id);
    document.entities.insert(rail_id, rail);
    document.entities.insert(profile_id, profile);
    let mut recipe = Recipe::new(
        document.id,
        "RailProfile",
        vec![SourceBinding {
            role: "rail".into(),
            source: document.stamp(rail_id).unwrap(),
            selection: Selection::Whole,
        }],
        vec![profile_id],
        ReplayEnvironment {
            command_version: 1,
            solver: "om9-core-test".into(),
            solver_version: "0".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance::om9_template(),
        },
    );
    recipe.state = EvaluationState::Unresolved("source missing".into());
    let recipe_id = recipe.id;
    document.history.recipes.insert(recipe_id, recipe);
    (document, rail_id, profile_id, recipe_id)
}

#[test]
fn a_missing_input_is_reported_with_its_role_and_a_clean_recipe_is_not() {
    let (mut document, rail, _profile, recipe) = fixture();
    // The rail exists, so only the Unresolved state flags the row and no input is missing.
    let rows = history_ops::broken_links(&document, &document.history);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].recipe, recipe);
    assert!(rows[0].missing_inputs.is_empty());
    assert!(document.history.recipes[&recipe].state == EvaluationState::Unresolved("source missing".into()));

    // Deleting the source keeps the row and still names the missing role.
    document.entities.remove(&rail);
    let rows = history_ops::broken_links(&document, &document.history);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].missing_inputs, vec!["rail".to_string()]);

    // A clean recipe with every binding intact never appears. The restored entity must
    // carry the id the stamp names, not only the map key.
    document.history.recipes.get_mut(&recipe).unwrap().state = EvaluationState::Clean;
    let mut restored = Entity::new("Rail", rail_curve(), None);
    restored.id = rail;
    document.entities.insert(rail, restored);
    let rows = history_ops::broken_links(&document, &document.history);
    assert!(rows.is_empty(), "clean recipe is not a broken link");
}

#[test]
fn a_missing_output_is_reported_by_id() {
    let (mut document, _rail, profile, recipe) = fixture();
    document.entities.remove(&profile);
    let rows = history_ops::broken_links(&document, &document.history);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].missing_outputs, vec![profile]);
    assert_eq!(rows[0].recipe, recipe);
}

#[test]
fn detach_clears_one_recipe_and_leaves_the_other_association_alone() {
    let (mut document, _rail, profile, recipe) = fixture();
    let other = Recipe::new(
        document.id,
        "RailSweep",
        vec![],
        vec![ObjectId::new()],
        document.history.recipes[&recipe].environment.clone(),
    );
    let other_id = other.id;
    document.history.recipes.insert(other_id, other);

    let detached = history_ops::detach_object(&mut document.history, profile).unwrap();
    assert_eq!(detached, recipe);
    assert_eq!(
        document.history.recipes[&recipe].state,
        EvaluationState::Detached
    );
    assert_eq!(
        document.history.recipes[&other_id].state,
        EvaluationState::Clean
    );
    // The detached output is a plain object, not an owned history result.
    assert!(document.entities.contains_key(&profile));
    assert_eq!(document.history.owner(profile), None);
}

#[test]
fn a_frozen_recipe_refuses_detach_and_a_missing_owner_is_typed() {
    let (mut document, _rail, profile, recipe) = fixture();
    document.history.recipes.get_mut(&recipe).unwrap().policy = HistoryPolicy {
        record: true,
        update: true,
        lock: true,
    };
    assert_eq!(
        history_ops::detach_object(&mut document.history, profile).unwrap_err(),
        Error::Invalid("history recipe locked")
    );
    assert_eq!(
        document.history.recipes[&recipe].state,
        EvaluationState::Unresolved("source missing".into())
    );
    assert_eq!(
        history_ops::detach_object(&mut document.history, ObjectId::new()).unwrap_err(),
        Error::Missing("history recipe")
    );
}

#[test]
fn a_changed_source_is_reported_as_changed_not_missing() {
    let (mut document, rail, _profile, recipe) = fixture();
    document.history.recipes.get_mut(&recipe).unwrap().state = EvaluationState::Clean;
    let entity = document.entities.get_mut(&rail).unwrap();
    entity.geometry = Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 12.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    });
    entity.geometry_revision += 1;

    let rows = history_ops::broken_links(&document, &document.history);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].changed_inputs, vec!["rail".to_string()]);
    assert!(rows[0].missing_inputs.is_empty());
    assert!(rows[0].missing_outputs.is_empty());
    assert_eq!(rows[0].state, EvaluationState::Clean);
}

#[test]
fn a_pending_recipe_is_visible_to_the_lifecycle_guard() {
    let (mut document, _rail, _profile, recipe) = fixture();
    assert!(!history_ops::has_pending(&document.history));
    document.history.recipes.get_mut(&recipe).unwrap().state = EvaluationState::Pending;
    assert!(history_ops::has_pending(&document.history));
}
