// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder panel controls through the controller (`OM9-L3-03-11-002`, `-003`, `-005`, `-006`,
//! `-008`, `-009`): mode tabs, sliders, stale panel events, locked inputs and the
//! Enter/Escape/Reset journal semantics.
//!
//! These tests use the same `submit`/controller path the panel buttons, the command line and
//! the viewport handle drag use, so a passing test is evidence for both invocation routes.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    builder::BuilderKind,
    geometry::{Curve, Geometry},
    identity::{LayerId, ObjectId},
    model::{Document, Entity, Layer},
    request::Value,
    units::LengthMm,
};

fn limits() -> Limits {
    Limits::default()
}

fn line(start: [f64; 3], end: [f64; 3]) -> Geometry {
    Geometry::Curve(Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    })
}

/// A rail (closed circle), a profile line and one layer that both live on.
fn document() -> (Document, ObjectId, ObjectId, LayerId) {
    let mut document = Document::new("builder-controls-ui");
    let layer = Layer::new("Rails", [200, 180, 120, 255]);
    let layer_id = layer.id;
    document.layers.insert(layer_id, layer);
    document.layer_order.push(layer_id);
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
        Some(layer_id),
    );
    let profile = Entity::new(
        "Profile",
        line([9.0, -1.0, 0.0], [9.0, 1.0, 0.0]),
        Some(layer_id),
    );
    let (rail_id, profile_id) = (rail.id, profile.id);
    document.entities.insert(rail_id, rail);
    document.entities.insert(profile_id, profile);
    (document, rail_id, profile_id, layer_id)
}

fn controller_with_rail() -> (Controller, ObjectId) {
    let (document, rail, _profile, _layer) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    (controller, rail)
}

#[test]
fn a_mode_tab_switch_keeps_each_modes_own_parameters() {
    let (mut controller, _rail) = controller_with_rail();
    // Open the profile tab and move its width away from the default.
    controller.submit("RailProfile").unwrap();
    controller.submit("Width=3").unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(3.0)
    );

    // Switch to the ring rail tab: a different schema, its own defaults.
    controller.switch_builder(BuilderKind::RingRail).unwrap();
    assert_eq!(controller.builder().unwrap().kind(), BuilderKind::RingRail);
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("diameter_mm")
            .unwrap()
            .numeric(),
        Some(18.0)
    );
    controller.submit("Diameter=22").unwrap();

    // Back to the profile tab: the 3.0 mm width survived the round trip.
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
    // And the ring rail tab kept its 22 mm diameter.
    controller.switch_builder(BuilderKind::RingRail).unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("diameter_mm")
            .unwrap()
            .numeric(),
        Some(22.0)
    );

    // The tab row lists every mode and marks exactly one active.
    let tabs = controller.builder_tabs();
    assert!(
        tabs.iter()
            .any(|(kind, _)| *kind == BuilderKind::RailProfile)
    );
    assert_eq!(tabs.iter().filter(|(_, active)| *active).count(), 1);
}

#[test]
fn a_tab_with_an_invalid_preview_is_never_silently_applied_or_silently_dropped() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    // Removing the selected shape leaves the active tab without a preview while it is
    // still the active tab: the pending invalid preview the spec names as the negative
    // case. The tab keeps the user's parameters; it is not reset behind their back.
    controller
        .builder_remove_preset("rectangle")
        .expect_err("removing the selected preset drops the preview");
    assert!(controller.builder().is_some());
    assert!(
        controller.builder().unwrap().preview().is_none(),
        "a removed preset drops the preview until the tab is reset"
    );
    let width_before = controller
        .builder()
        .unwrap()
        .field("width_mm")
        .unwrap()
        .numeric();

    // Switching away is allowed, and the invalid tab is saved rather than applied.
    controller.switch_builder(BuilderKind::RingRail).unwrap();
    assert_eq!(controller.builder().unwrap().kind(), BuilderKind::RingRail);
    assert_eq!(controller.document().history.recipes.len(), 0);

    // Returning to it is refused, because its selected preset is gone; the ring rail tab
    // stays active instead of leaving the panel empty.
    assert_eq!(
        controller
            .switch_builder(BuilderKind::RailProfile)
            .unwrap_err(),
        om9_core::Error::Missing("profile preset")
    );
    assert_eq!(controller.builder().unwrap().kind(), BuilderKind::RingRail);
    assert_eq!(
        controller
            .builder_saved()
            .get(&BuilderKind::RailProfile)
            .and_then(|session| session.field("width_mm"))
            .and_then(|field| field.numeric()),
        width_before,
        "the invalid tab keeps the user's parameters while it is parked"
    );
}

#[test]
fn a_stale_panel_event_cannot_overwrite_a_newer_handle_drag() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    let painted_at = controller.builder_revision().unwrap();

    // The viewport drag lands first and moves the shared field.
    controller.builder_adjust_field("width_mm", 1.0).unwrap();
    let dragged = controller
        .builder()
        .unwrap()
        .field("width_mm")
        .unwrap()
        .numeric()
        .unwrap();
    assert_eq!(dragged, 3.0);

    // A panel event painted before the drag is refused instead of reverting it.
    let stale = controller.builder_set_field_at(
        "width_mm",
        Value::length(LengthMm::try_new(2.0).unwrap()),
        painted_at,
    );
    assert!(stale.is_err());
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(3.0)
    );

    // The current revision is accepted.
    let current = controller.builder_revision().unwrap();
    controller
        .builder_set_field_at(
            "width_mm",
            Value::length(LengthMm::try_new(2.5).unwrap()),
            current,
        )
        .unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(2.5)
    );
}

#[test]
fn a_slider_edit_from_the_panel_quantises_and_previews_the_same_value() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    let ratio = (2.5 - 0.1) / (20.0 - 0.1);
    controller.builder_set_slider("width_mm", ratio).unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(2.5)
    );
    assert!(
        controller
            .builder_summary()
            .unwrap_or_default()
            .contains("2.50 x 1.50"),
        "the preview reads the quantised field"
    );
    // The panel view carries the step and the slider position the widget paints.
    let view = controller
        .builder_field_views()
        .into_iter()
        .find(|field| field.name == "width_mm")
        .unwrap();
    assert_eq!(view.step, Some(0.1));
    assert_eq!(view.value, 2.5);
    assert!(view.ratio.is_some());
}

#[test]
fn a_locked_input_makes_the_builder_read_only_and_hides_its_handles() {
    let (document, rail, _profile, _layer) = document();
    let mut controller = Controller::new(document, limits()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    // Lock the object after it was selected, the way the property editor does: the
    // selection survives, so the builder can open on a now read-only input.
    let stamp = controller.document().stamp(rail).unwrap();
    controller
        .edit_object_property(vec![stamp], om9_app::session::PropertyEdit::Locked(true))
        .unwrap();
    controller.submit("RailProfile").unwrap();
    assert!(!controller.builder_editable());
    assert!(controller.builder_handles().is_empty());
    assert!(controller.builder_adjust_field("width_mm", 0.5).is_err());
    assert!(controller.builder_set_slider("width_mm", 0.5).is_err());
    assert!(
        controller
            .builder_set_field("width_mm", Value::length(LengthMm::try_new(2.5).unwrap()))
            .is_err()
    );
    // The lock transaction is the only journal entry; no builder edit was recorded.
    let recipes_before = controller.document().history.recipes.len();
    assert_eq!(recipes_before, 0);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn double_enter_applies_once_and_reset_creates_no_journal_entry() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    controller.submit("Width=3").unwrap();
    // Reset is a panel action, not a document edit.
    controller.submit("Reset").unwrap();
    assert_eq!(controller.coordinator().undo_len(), 0);
    assert!(controller.document().history.recipes.is_empty());

    // First Enter applies the feature as exactly one undoable step.
    controller.submit("").unwrap();
    assert!(controller.builder().is_none());
    assert_eq!(controller.coordinator().undo_len(), 1);
    assert_eq!(controller.document().history.recipes.len(), 1);

    // A second Enter has no builder to apply: no second history entry.
    assert!(controller.builder_text("").is_none());
    assert_eq!(controller.coordinator().undo_len(), 1);
    assert_eq!(controller.document().history.recipes.len(), 1);
}

#[test]
fn escape_after_preview_edits_leaves_the_document_bytewise_unchanged() {
    let (mut controller, _rail) = controller_with_rail();
    let before = controller.document().clone();
    controller.submit("RailProfile").unwrap();
    controller.submit("Width=3").unwrap();
    controller.builder_adjust_field("width_mm", 1.0).unwrap();
    controller.submit("Cancel").unwrap();
    assert!(controller.builder().is_none());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn the_f6_prefill_and_the_command_line_reach_the_same_builder_service() {
    // `begin_builder` is what both the F6 prefill and the command line call; the command
    // name therefore opens the same mode with the same typed defaults.
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RingRail").unwrap();
    let from_command = controller
        .builder()
        .unwrap()
        .field("diameter_mm")
        .unwrap()
        .numeric();
    controller.submit("Cancel").unwrap();

    // The L3-009 fixture calls the command-line route `RingBuilder`; it opens the same mode
    // with the same typed defaults as the tab/service route.
    controller.submit("RingBuilder").unwrap();
    let from_builder_name = controller
        .builder()
        .unwrap()
        .field("diameter_mm")
        .unwrap()
        .numeric();
    assert_eq!(from_builder_name, from_command);
    assert_eq!(controller.builder().unwrap().kind(), BuilderKind::RingRail);
    controller.submit("Cancel").unwrap();

    controller.begin_builder(BuilderKind::RingRail).unwrap();
    let from_service = controller
        .builder()
        .unwrap()
        .field("diameter_mm")
        .unwrap()
        .numeric();
    assert_eq!(from_command, from_service);
    controller.submit("Cancel").unwrap();

    // An unknown command is refused, and a mode whose input is missing never opens.
    assert!(controller.submit("NotABuilderCommand").is_err());
    assert!(controller.builder().is_none());
    controller.clear_selection();
    assert!(controller.submit("RailProfile").is_err());
    assert!(controller.builder().is_none());
}
