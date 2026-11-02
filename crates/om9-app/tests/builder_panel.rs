// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder panel and command frame for the M04 ring chain (`OM9-L2-03-11`, `OM9-L2-04-02`).
//!
//! These are controller-level tests: the same `submit` path the panel buttons and the command
//! line use, the same one-transaction Apply, and the recompute journal.
use om9_app::session::Controller;
use om9_core::{
    associative::RecomputeOutcome,
    budget::Limits,
    geometry::{Curve, Geometry},
    identity::ObjectId,
    model::{Document, Entity},
    request::Value,
};

fn limits() -> Limits {
    Limits::default()
}

fn curve(start: [f64; 3], end: [f64; 3]) -> Geometry {
    Geometry::Curve(Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    })
}

fn document() -> (Document, ObjectId, ObjectId) {
    let mut document = Document::new("builder-ui");
    let rail = Entity::new(
        "Rail",
        Geometry::Curve(Curve::Circle {
            center: [0.0, 0.0, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 9.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let profile = Entity::new("Profile", curve([9.0, -1.0, 0.0], [9.0, 1.0, 0.0]), None);
    let (rail_id, profile_id) = (rail.id, profile.id);
    document.entities.insert(rail_id, rail);
    document.entities.insert(profile_id, profile);
    (document, rail_id, profile_id)
}

fn controller() -> Controller {
    Controller::new(document().0, limits()).unwrap()
}

#[test]
fn the_ring_rail_command_line_opens_edits_and_applies_one_undoable_feature() {
    let mut controller = controller();
    controller.submit("RingRail").unwrap();
    assert!(controller.builder().is_some());
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("diameter_mm")
            .unwrap()
            .numeric(),
        Some(18.0)
    );
    controller.submit("Diameter=20").unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("diameter_mm")
            .unwrap()
            .numeric(),
        Some(20.0)
    );
    // Enter (empty text) applies.
    controller.submit("").unwrap();
    assert!(controller.builder().is_none());
    assert_eq!(controller.document().history.recipes.len(), 1);
    let recipe = controller
        .document()
        .history
        .recipes
        .values()
        .next()
        .unwrap();
    assert_eq!(recipe.command, "RingRail");
    assert_eq!(recipe.state, om9_core::history::EvaluationState::Clean);
    let output = recipe.outputs[0];
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        controller.document().entities[&output].geometry
    else {
        panic!("ring rail output must be a circle");
    };
    assert_eq!(radius, 10.0);
    assert_eq!(
        controller.document().results[&output].status,
        om9_core::result::ResultStatus::Current
    );
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert!(controller.document().history.recipes.is_empty());
    assert!(!controller.document().entities.contains_key(&output));
    controller.redo().unwrap();
    assert!(controller.document().entities.contains_key(&output));
}

#[test]
fn cancel_and_reset_leave_the_document_untouched() {
    let mut controller = controller();
    controller.submit("Ring Rail").unwrap();
    controller.submit("Diameter=22").unwrap();
    controller.submit("Reset").unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("diameter_mm")
            .unwrap()
            .numeric(),
        Some(18.0)
    );
    controller.submit("Cancel").unwrap();
    assert!(controller.builder().is_none());
    assert!(controller.document().history.recipes.is_empty());
    assert_eq!(controller.coordinator().undo_len(), 0);
    // A profile builder needs exactly one selected curve.
    controller.submit("RailProfile").unwrap_err();
    assert!(controller.builder().is_none());
}

#[test]
fn profile_and_sweep_modes_read_the_selection_and_commit_their_own_command() {
    let (document, rail, _profile) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    controller.submit("Rail Profile").unwrap();
    controller.submit("Width=3").unwrap();
    controller.submit("").unwrap();
    let profile_recipe = controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RailProfile")
        .unwrap();
    let profile_output = profile_recipe.outputs[0];
    assert_eq!(profile_recipe.inputs.len(), 1);
    assert_eq!(profile_recipe.input_ports[0].role, "rail");

    // The sweep reads the profile feature's own result, not the original line.
    controller.select_objects(&[rail, profile_output]).unwrap();
    controller.submit("RailSweep").unwrap();
    controller.submit("").unwrap();
    let sweep = controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RailSweep")
        .unwrap();
    assert_eq!(sweep.inputs.len(), 2);
    assert_eq!(sweep.input_ports[1].role, "profile");
    let output = sweep.outputs[0];
    match &controller.document().entities[&output].geometry {
        #[cfg(feature = "occt-brep")]
        Geometry::Brep(_) => {}
        #[cfg(not(feature = "occt-brep"))]
        Geometry::Surface(_) => {}
        other => panic!("sweep output kind {}", other.kind().tag()),
    }
}

#[test]
fn a_handle_drag_and_a_typed_value_share_the_one_field() {
    let mut controller = controller();
    controller.submit("RingRail").unwrap();
    controller.builder_adjust_field("diameter_mm", 2.0).unwrap();
    assert_eq!(controller.builder_handles()[0].value, 20.0);
    assert_eq!(controller.builder_handles()[0].position, [10.0, 0.0, 0.0]);
    controller.submit("").unwrap();
    let output = controller
        .document()
        .history
        .recipes
        .values()
        .next()
        .unwrap()
        .outputs[0];
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        controller.document().entities[&output].geometry
    else {
        panic!("circle");
    };
    assert_eq!(radius, 10.0);
}

#[test]
fn a_parameter_edit_and_its_recompute_are_two_undoable_steps() {
    let mut controller = controller();
    controller.submit("RingRail").unwrap();
    controller.submit("").unwrap();
    let recipe = controller
        .document()
        .history
        .recipes
        .values()
        .next()
        .unwrap();
    let id = recipe.id;
    let output = recipe.outputs[0];
    controller
        .set_history_parameter(
            id,
            "diameter_mm",
            Value::length(om9_core::units::LengthMm::try_new(22.0).unwrap()),
        )
        .unwrap();
    assert_eq!(
        controller.document().history.recipes[&id].state,
        om9_core::history::EvaluationState::Dirty("parameter edit".into())
    );
    let outcome = controller.recompute_history(id).unwrap();
    assert!(matches!(outcome, RecomputeOutcome::Rebuilt { .. }));
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        controller.document().entities[&output].geometry
    else {
        panic!("circle");
    };
    assert_eq!(radius, 11.0);
    assert_eq!(
        controller.document().history.recipes[&id].state,
        om9_core::history::EvaluationState::Clean
    );
    assert_eq!(
        controller.document().results[&output].geometry_snapshot,
        controller.document().entities[&output].geometry
    );
    // Undo the recompute: geometry goes back to the previous rebuild, the parameter stays.
    controller.undo().unwrap();
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        controller.document().entities[&output].geometry
    else {
        panic!("circle");
    };
    assert_eq!(radius, 9.0);
    assert_eq!(
        controller.document().history.recipes[&id].state,
        om9_core::history::EvaluationState::Dirty("parameter edit".into())
    );
    controller.undo().unwrap();
    assert_eq!(
        controller.document().history.recipes[&id].parameters["diameter_mm"]
            .as_length()
            .unwrap()
            .mm(),
        18.0
    );
}

#[test]
fn tabs_keep_each_mode_parameters_and_switch_safely() {
    use om9_core::builder::BuilderKind;
    let (document, rail, _) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    controller.submit("Rail Profile").unwrap();
    assert_eq!(
        controller.builder().unwrap().kind(),
        BuilderKind::RailProfile
    );
    controller.submit("Width=3").unwrap();
    controller.switch_builder(BuilderKind::RingRail).unwrap();
    assert_eq!(controller.builder().unwrap().kind(), BuilderKind::RingRail);
    controller.submit("Diameter=20").unwrap();
    // Returning to a tab restores its own fields.
    controller.switch_builder(BuilderKind::RailProfile).unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(3.0)
    );
    controller.switch_builder(BuilderKind::RingRail).unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("diameter_mm")
            .unwrap()
            .numeric(),
        Some(20.0)
    );
    let tabs = controller.builder_tabs();
    // The ring family (`OM9-L2-04-03`) added its four modes to the tab strip.
    assert_eq!(tabs.len(), BuilderKind::ALL.len());
    assert_eq!(tabs.len(), 9);
    assert_eq!(tabs.iter().filter(|(_, active)| *active).count(), 1);
    assert!(
        tabs.iter()
            .any(|(kind, active)| *kind == BuilderKind::RingRail && *active)
    );
}

#[test]
fn a_failed_tab_switch_restores_the_previous_tab() {
    use om9_core::builder::BuilderKind;
    let (document, rail, _) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    controller.submit("Rail Profile").unwrap();
    controller.builder_set_preset("rounded_rectangle").unwrap();
    // Removing the selected preset drops the preview; the tab itself stays active.
    assert_eq!(
        controller
            .builder_remove_preset("rounded_rectangle")
            .unwrap_err(),
        om9_core::Error::Missing("profile preset")
    );
    assert!(controller.builder().unwrap().preview().is_none());
    controller.switch_builder(BuilderKind::RingRail).unwrap();
    // The saved profile tab can no longer refresh (its selected preset is gone), so the
    // switch is refused and the ring rail tab stays active.
    assert_eq!(
        controller
            .switch_builder(BuilderKind::RailProfile)
            .unwrap_err(),
        om9_core::Error::Missing("profile preset")
    );
    assert_eq!(controller.builder().unwrap().kind(), BuilderKind::RingRail);
}

#[test]
fn outside_rail_and_resizer_apply_independent_outputs() {
    let (document, rail, _) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    controller.submit("Outside Rail").unwrap();
    controller.submit("Offset=2").unwrap();
    controller.submit("").unwrap();
    let outside = controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "OutsideRail")
        .unwrap();
    let output = outside.outputs[0];
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        controller.document().entities[&output].geometry
    else {
        panic!("outside rail must be a circle");
    };
    assert_eq!(radius, 11.0);
    let Geometry::Curve(Curve::Circle { radius: source, .. }) =
        controller.document().entities[&rail].geometry
    else {
        panic!("source rail");
    };
    assert_eq!(source, 9.0);
    controller.undo().unwrap();
    assert!(!controller.document().entities.contains_key(&output));

    controller.select_objects(&[rail]).unwrap();
    controller.submit("Ring Resizer").unwrap();
    controller.submit("Target=19").unwrap();
    controller.submit("").unwrap();
    let resizer = controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RingResizer")
        .unwrap();
    let resized = resizer.outputs[0];
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        controller.document().entities[&resized].geometry
    else {
        panic!("resizer output must be a circle");
    };
    assert_eq!(radius, 9.5);
    // The resizer is non-historical here: the original ring object is untouched.
    let Geometry::Curve(Curve::Circle { radius: source, .. }) =
        controller.document().entities[&rail].geometry
    else {
        panic!("source ring");
    };
    assert_eq!(source, 9.0);
    controller.undo().unwrap();
    assert!(!controller.document().entities.contains_key(&resized));
    assert!(controller.document().entities.contains_key(&rail));
}

#[test]
fn the_profile_library_changes_the_preview_and_the_applied_shape() {
    let (document, rail, _) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    controller.submit("Rail Profile").unwrap();
    assert_eq!(controller.builder_presets().len(), 3);
    controller.builder_set_preset("circle").unwrap();
    assert!(matches!(
        controller.builder().unwrap().preview().unwrap().geometry,
        Geometry::Curve(Curve::Circle { .. })
    ));
    // Choosing a preset does not modify the document.
    assert_eq!(controller.document().entities.len(), 2);
    controller.submit("").unwrap();
    let recipe = controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RailProfile")
        .unwrap();
    assert_eq!(recipe.parameters["preset"], Value::Choice("circle".into()));
    let output = recipe.outputs[0];
    assert!(matches!(
        controller.document().entities[&output].geometry,
        Geometry::Curve(Curve::Circle { .. })
    ));
}

#[test]
fn the_preset_can_be_chosen_from_the_command_line() {
    let (document, rail, _) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    controller.submit("Rail Profile").unwrap();
    controller.submit("preset=rounded_rectangle").unwrap();
    let preview = controller.builder().unwrap().preview().unwrap();
    let Geometry::Curve(Curve::Polycurve { segments, .. }) = &preview.geometry else {
        panic!("rounded rectangle is a polycurve");
    };
    assert!(segments.len() > 4);
    controller.submit("").unwrap();
    let recipe = controller
        .document()
        .history
        .recipes
        .values()
        .find(|recipe| recipe.command == "RailProfile")
        .unwrap();
    assert_eq!(
        recipe.parameters["preset"],
        Value::Choice("rounded_rectangle".into())
    );
}
