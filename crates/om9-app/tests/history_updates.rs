// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix History record/update toggles, broken-link report, per-object clear, edit from
//! history and the history command line (`OM9-L3-03-13-001` … `-006`). The same
//! `Controller` entry points the panel, the sidebar icons and the command line use.
use om9_app::session::Controller;
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Curve, Geometry, Transform, Unit},
    history::{EvaluationState, HistoryPolicy, Recipe, ReplayEnvironment, Tolerance},
    identity::ObjectId,
    model::{Document, Entity, Layer},
    request::{Selection, SourceBinding, Value},
    units::LengthMm,
};

fn controller() -> Controller {
    Controller::new(Document::new("history-ui"), Limits::default()).expect("controller")
}

fn circle(radius: f64) -> Geometry {
    Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn line() -> Geometry {
    Geometry::Curve(Curve::Line {
        start: [9.0, -1.0, 0.0],
        end: [9.0, 1.0, 0.0],
        domain: [0.0, 1.0],
    })
}

/// A document with one rail, one profile and a `profile <- rail` recipe. With
/// `drop_input` the rail is removed after its stamp is captured, producing the stale
/// reference a reopened document shows; `lock` freezes the recipe.
fn broken_document(drop_input: bool, lock: bool, pending: bool) -> (Document, ObjectId) {
    let mut document = Document::new("history-broken");
    let layer = Layer::new("Rails", [200, 180, 120, 255]);
    let layer_id = layer.id;
    document.layers.insert(layer_id, layer);
    document.layer_order.push(layer_id);
    let rail = Entity::new("Rail", circle(9.0), Some(layer_id));
    let profile = Entity::new("Profile", line(), Some(layer_id));
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
            solver: "om9-app-test".into(),
            solver_version: "0".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance::om9_template(),
        },
    );
    recipe.state = if pending {
        EvaluationState::Pending
    } else {
        EvaluationState::Unresolved("source missing".into())
    };
    if lock {
        recipe.policy = HistoryPolicy {
            record: true,
            update: true,
            lock: true,
        };
    }
    document.history.recipes.insert(recipe.id, recipe);
    if drop_input {
        document.entities.remove(&rail_id);
    }
    (document, profile_id)
}

fn ring_recipe(controller: &Controller) -> om9_core::identity::HistoryId {
    controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RingRail")
        .expect("ring recipe")
        .id
}

fn profile_recipe(controller: &Controller) -> om9_core::identity::HistoryId {
    controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RailProfile")
        .expect("profile recipe")
        .id
}

fn radius(controller: &Controller, object: ObjectId) -> f64 {
    match &controller.document().entities[&object].geometry {
        Geometry::Curve(Curve::Circle { radius, .. }) => *radius,
        other => panic!("expected circle, got {other:?}"),
    }
}

fn ring_and_profile(mut controller: Controller) -> Controller {
    controller.submit("RingRail").unwrap();
    controller.submit("Diameter=18").unwrap();
    controller.builder_apply().unwrap();
    let rail = controller.document().history.recipes.values()
        .find(|recipe| recipe.command == "RingRail")
        .unwrap()
        .outputs[0];
    controller.select_objects(&[rail]).unwrap();
    controller.submit("RailProfile").unwrap();
    controller.submit("Width=2").unwrap();
    controller.builder_apply().unwrap();
    controller
}

#[test]
fn record_off_commits_geometry_without_a_recipe_and_toggles_stamp_new_records() {
    let mut controller = controller();
    controller.set_history_record(false).unwrap();
    controller.submit("RingRail").unwrap();
    controller.submit("Diameter=20").unwrap();
    controller.builder_apply().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert!(controller.document().history.recipes.is_empty());
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert!(controller.document().entities.is_empty());

    controller.set_history_record(true).unwrap();
    controller.submit("RingRail").unwrap();
    controller.submit("Diameter=20").unwrap();
    controller.builder_apply().unwrap();
    let recipe = controller.document().history.recipes.values().next().unwrap();
    assert_eq!(recipe.command, "RingRail");
    assert!(recipe.policy.record);

    // A toggle during an open builder transaction is refused and changes nothing.
    controller.submit("RingRail").unwrap();
    assert_eq!(
        controller.set_history_record(false).unwrap_err(),
        Error::Invalid("history record during edit transaction")
    );
    assert!(controller.history_record_enabled());
}

#[test]
fn update_off_marks_dirty_and_rebuilds_only_after_update_is_reenabled() {
    let mut controller = ring_and_profile(controller());
    let ring = ring_recipe(&controller);
    let rail = controller.document().history.recipes[&ring].outputs[0];
    let profile = profile_recipe(&controller);
    assert_eq!(radius(&controller, rail), 9.0);

    controller.set_history_update(false).unwrap();
    controller
        .apply_history_parameter(ring, "diameter_mm", Value::length(LengthMm::try_new(20.0).unwrap()))
        .unwrap();
    assert_eq!(
        controller.document().history.recipes[&ring].state,
        EvaluationState::Dirty("parameter edit".into())
    );
    assert_eq!(
        controller.document().history.recipes[&profile].state,
        EvaluationState::Clean
    );
    // The geometry waits for the switch, not for a silent rebuild.
    assert_eq!(radius(&controller, rail), 9.0);
    // The report warns about the Dirty recipe without calling it missing.
    let dirty = controller
        .history_entries()
        .into_iter()
        .find(|entry| entry.recipe == ring)
        .expect("ring row");
    assert!(dirty.problem);
    assert!(dirty.detail.contains("stale"), "detail: {}", dirty.detail);
    assert!(controller
        .take_history_notice()
        .unwrap()
        .contains("update off"));

    controller.set_history_update(true).unwrap();
    controller
        .apply_history_parameter(ring, "diameter_mm", Value::length(LengthMm::try_new(22.0).unwrap()))
        .unwrap();
    assert_eq!(
        controller.document().history.recipes[&ring].state,
        EvaluationState::Clean
    );
    assert_eq!(
        controller.document().history.recipes[&profile].state,
        EvaluationState::Clean
    );
    assert_eq!(radius(&controller, rail), 11.0);
    // The command-event surface distinguishes the updating phase from the terminal
    // complete state (`OM9-L3-03-13-006`).
    let trail: Vec<_> = controller
        .events
        .events()
        .iter()
        .filter(|event| event.command == "History")
        .map(|event| (event.kind.label(), event.detail.clone()))
        .collect();
    assert!(trail.iter().any(|(kind, _)| *kind == "started"));
    assert!(trail
        .iter()
        .any(|(kind, detail)| *kind == "finished" && detail.contains("complete")));
}

#[test]
fn reenabling_update_refuses_a_broken_dependency() {
    let (document, _profile) = broken_document(true, false, false);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.set_history_update(false).unwrap();
    assert_eq!(
        controller.set_history_update(true).unwrap_err(),
        Error::Invalid("history dependency broken")
    );
    assert!(!controller.history_update_enabled());
}

#[test]
fn the_report_names_a_missing_input_and_survives_a_reopen() {
    let (document, _profile) = broken_document(true, false, false);
    let controller = Controller::new(document, Limits::default()).unwrap();
    let rows = controller.history_broken_links();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].missing_inputs, vec!["rail".to_string()]);
    assert_eq!(rows[0].command, "RailProfile");
    let entries = controller.history_entries();
    let broken = entries.iter().find(|entry| entry.problem).expect("broken row");
    assert_eq!(broken.command, "RailProfile");
    assert!(
        broken.detail.contains("missing input rail (source"),
        "detail: {}",
        broken.detail
    );

    // Save/reopen keeps the phantom stale reference visible instead of silently dropping it.
    let json = serde_json::to_string(controller.document()).unwrap();
    let reopened: Document = serde_json::from_str(&json).unwrap();
    let reopened = Controller::new(reopened, Limits::default()).unwrap();
    let rows = reopened.history_broken_links();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].missing_inputs, vec!["rail".to_string()]);
    assert_eq!(rows[0].state, EvaluationState::Unresolved("source missing".into()));
}

#[test]
fn clear_history_detaches_only_the_selected_object_and_refuses_a_lock() {
    let mut controller = ring_and_profile(controller());
    let profile = profile_recipe(&controller);
    let output = controller.document().history.recipes[&profile].outputs[0];
    let ring = ring_recipe(&controller);
    controller.clear_object_history(output).unwrap();
    assert_eq!(
        controller.document().history.recipes[&profile].state,
        EvaluationState::Detached
    );
    assert_eq!(
        controller.document().history.recipes[&ring].state,
        EvaluationState::Clean
    );
    assert!(controller.document().entities.contains_key(&output));
    assert_eq!(controller.history_entries().len(), 1);
    assert!(controller.history_entries()[0].recipe == ring);

    let (document, locked_profile) = broken_document(true, true, false);
    let mut locked = Controller::new(document, Limits::default()).unwrap();
    assert_eq!(
        locked.clear_object_history(locked_profile).unwrap_err(),
        Error::Invalid("history recipe locked")
    );
}

#[test]
fn edit_from_history_reopens_the_builder_at_the_recorded_parameters() {
    let mut controller = controller();
    controller.submit("RingRail").unwrap();
    controller.submit("Diameter=22").unwrap();
    controller.builder_apply().unwrap();
    let output = controller.document().history.recipes.values().next().unwrap().outputs[0];

    controller.begin_builder_from_history(output).unwrap();
    let builder = controller.builder().unwrap();
    assert_eq!(builder.kind(), om9_core::builder::BuilderKind::RingRail);
    assert_eq!(
        builder.field("diameter_mm").and_then(|field| field.numeric()),
        Some(22.0)
    );
    controller.submit("Diameter=24").unwrap();
    controller.builder_apply().unwrap();
    // Apply updates the feature that owns the reopened recipe; it does not insert a copy.
    assert_eq!(controller.document().history.recipes.len(), 1);
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(radius(&controller, output), 12.0);

    // Editing an existing recorded feature still updates its recipe when recording of new
    // features is off; the policy stamped on that recipe is the current one.
    controller.set_history_record(false).unwrap();
    controller.begin_builder_from_history(output).unwrap();
    controller.submit("Diameter=26").unwrap();
    controller.builder_apply().unwrap();
    assert_eq!(controller.document().history.recipes.len(), 1);
    assert_eq!(radius(&controller, output), 13.0);
    let recipe = &controller.document().history.recipes.values().next().unwrap();
    assert!(!recipe.policy.record);
    assert_eq!(
        recipe.parameters.get("diameter_mm"),
        Some(&Value::length(LengthMm::try_new(26.0).unwrap()))
    );

    // A missing object/recipe is an explicit refusal, not a silent no-op.
    assert_eq!(
        controller
            .begin_builder_from_history(ObjectId::new())
            .unwrap_err(),
        Error::Missing("history recipe")
    );
}

#[test]
fn the_command_line_drives_the_switches_report_and_clear() {
    let mut controller = controller();
    controller.submit("HistoryRecord").unwrap();
    assert!(!controller.history_record_enabled());
    assert!(controller.take_history_notice().unwrap().contains("off"));
    controller.submit("HistoryRecord").unwrap();
    assert!(controller.history_record_enabled());

    controller.submit("HistoryUpdate").unwrap();
    assert!(!controller.history_update_enabled());
    controller.submit("HistoryUpdate").unwrap();
    assert!(controller.history_update_enabled());

    controller.submit("HistoryReport").unwrap();
    assert!(controller.take_history_window_request());

    // Clear without a selection is a visible error, and no document change happens.
    let revision = controller.document().revision;
    assert_eq!(
        controller.submit("ClearHistory").unwrap_err(),
        Error::Missing("history object")
    );
    assert_eq!(controller.document().revision, revision);
}

#[test]
fn closing_the_window_is_refused_while_a_recipe_is_pending() {
    let (document, _profile) = broken_document(false, false, true);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    assert!(controller.history_has_pending());
    assert_eq!(controller.begin_window_close().unwrap_err(), Error::Busy);
}
