// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder styles through the controller (`OM9-L3-03-12-001` … `-006`): the browser list, the
//! preview-only apply, user saves, the mode filter, attribute matching and persistence.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    identity::ObjectId,
    model::{Document, Entity, Extension, Layer},
};

fn circle_curve() -> Geometry {
    Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 9.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn document() -> (Document, ObjectId, ObjectId) {
    let mut document = Document::new("builder-styles-ui");
    let layer = Layer::new("Rings", [200, 180, 120, 255]);
    let layer_id = layer.id;
    document.layers.insert(layer_id, layer);
    document.layer_order.push(layer_id);
    let mut rail = Entity::new("Rail", circle_curve(), Some(layer_id));
    rail.extensions.insert(
        "om9.rail".to_string(),
        Extension::new(
            1,
            r#"{"width_mm": 3.0, "height_mm": 2.0, "material": "synthetic"}"#.to_string(),
        )
        .unwrap(),
    );
    let rail_id = rail.id;
    document.entities.insert(rail_id, rail);
    let other = Entity::new(
        "Other",
        Geometry::Curve(Curve::Line {
            start: [9.0, -1.0, 0.0],
            end: [9.0, 1.0, 0.0],
            domain: [0.0, 1.0],
        }),
        Some(layer_id),
    );
    let other_id = other.id;
    document.entities.insert(other_id, other);
    (document, rail_id, other_id)
}

fn controller_with_rail() -> (Controller, ObjectId) {
    let (document, rail, _other) = document();
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[rail]).unwrap();
    (controller, rail)
}

#[test]
fn the_browser_lists_the_styles_of_the_open_mode() {
    let (mut controller, _rail) = controller_with_rail();
    assert!(controller.builder_styles().is_empty());
    controller.submit("RailProfile").unwrap();
    let rows = controller.builder_styles();
    assert_eq!(rows.len(), 3);
    let gem = rows
        .iter()
        .find(|row| row.name == "Brilliant Gem 1.9")
        .unwrap();
    assert!(!gem.enabled, "a gem style is disabled for a ring builder");
    assert!(gem.builtin);
    // Both ring styles carry an authored thumbnail; none of them falls back by accident.
    assert!(
        rows.iter()
            .filter(|row| row.enabled)
            .all(|row| !row.fallback)
    );
}

#[test]
fn applying_a_style_changes_the_preview_only() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    let before = controller.document().clone();
    let entities = before.entities.len();

    controller.builder_apply_style("Classic 2.2").unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(2.2)
    );
    assert_eq!(controller.builder_applied_style(), Some("Classic 2.2"));
    assert!(
        !controller
            .builder()
            .unwrap()
            .preview()
            .unwrap()
            .summary
            .is_empty()
    );

    // Nothing is committed: no entity, no journal entry, the document is byte-identical.
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.document().entities.len(), entities);
    assert_eq!(controller.coordinator().undo_len(), 0);

    // An unknown name is an explicit error, not a silent no-op.
    assert_eq!(
        controller.builder_apply_style("Nope").err().unwrap(),
        om9_core::Error::Missing("builder style")
    );
}

#[test]
fn a_style_for_another_mode_is_listed_disabled_and_refused() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    assert_eq!(
        controller
            .builder_apply_style("Brilliant Gem 1.9")
            .err()
            .unwrap(),
        om9_core::Error::Invalid("style mode")
    );
    // The refusal left the session alone.
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(2.0)
    );
    assert_eq!(controller.builder_applied_style(), None);
}

#[test]
fn saving_a_style_adds_a_user_row_without_rewriting_the_defaults() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    controller.submit("Width=2.4").unwrap();

    let summary = controller.builder_save_style("Classic 2.5").unwrap();
    assert!(summary.contains("Classic 2.5"));
    let rows = controller.builder_styles();
    let saved = rows.iter().find(|row| row.name == "Classic 2.5").unwrap();
    assert!(!saved.builtin);
    assert!(saved.enabled);
    // The built-in list is untouched: three defaults, one user row.
    assert_eq!(rows.iter().filter(|row| row.builtin).count(), 3);
    assert_eq!(rows.iter().filter(|row| !row.builtin).count(), 1);

    // A duplicate is refused, and so is a built-in name.
    assert_eq!(
        controller.builder_save_style("classic 2.5").err().unwrap(),
        om9_core::Error::Invalid("style exists")
    );
    assert_eq!(
        controller.builder_save_style("Classic 2.2").err().unwrap(),
        om9_core::Error::Invalid("style exists")
    );
    // A name that could be a path is refused before it can become a file name.
    assert_eq!(
        controller.builder_save_style("../escape").err().unwrap(),
        om9_core::Error::Invalid("style name")
    );
    // Saving without an open builder is an explicit error.
    controller.submit("Cancel").unwrap();
    assert_eq!(
        controller.builder_save_style("Late").err().unwrap(),
        om9_core::Error::Missing("builder session")
    );
}

#[test]
fn user_styles_roundtrip_through_the_configured_file() {
    let directory = std::env::temp_dir().join(format!("om9-app-styles-{}", std::process::id()));
    let path = directory.join("styles.json");
    let _ = std::fs::remove_dir_all(&directory);

    let (first_document, first_rail, _other) = document();
    let mut controller = Controller::new(first_document, Limits::default()).unwrap();
    controller.select_objects(&[first_rail]).unwrap();
    controller.builder_set_style_path(Some(path.clone()));
    controller.submit("RailProfile").unwrap();
    controller.submit("Width=2.4").unwrap();
    controller.builder_save_style("My Ring 2.4").unwrap();
    assert!(path.is_file(), "the save wrote the configured user file");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("om9.builder_style.v1"));
    assert!(text.contains("My Ring 2.4"));
    assert!(!text.contains("Classic 2.2"));
    controller.submit("Cancel").unwrap();

    // A fresh controller that points at the same file gets the user style back, and the
    // built-ins are still the shipped three.
    let (second_document, second_rail, _other) = document();
    let mut fresh = Controller::new(second_document, Limits::default()).unwrap();
    fresh.select_objects(&[second_rail]).unwrap();
    fresh.builder_set_style_path(Some(path.clone()));
    assert_eq!(fresh.builder_load_styles().unwrap(), 1);
    fresh.submit("RailProfile").unwrap();
    let rows = fresh.builder_styles();
    assert!(
        rows.iter()
            .any(|row| row.name == "My Ring 2.4" && !row.builtin)
    );
    assert_eq!(rows.iter().filter(|row| row.builtin).count(), 3);

    // A corrupt file reports itself instead of silently emptying the user list.
    std::fs::write(&path, "{ not json").unwrap();
    assert_eq!(
        fresh.builder_load_styles().err().unwrap(),
        om9_core::Error::Invalid("style schema")
    );
    assert!(fresh.builder_style_error().is_some());
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn matching_attributes_copies_from_the_selection_and_keeps_the_ids() {
    let (mut controller, rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    let before = controller.document().entities[&rail].clone();

    let summary = controller.builder_match_attributes().unwrap();
    assert!(summary.contains("width_mm"));
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(3.0)
    );
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("height_mm")
            .unwrap()
            .numeric(),
        Some(2.0)
    );
    // `material` has no builder field: it is reported, not silently dropped.
    assert!(summary.contains("material") || summary.contains("no field for material"));
    // The source entity keeps its identity and attributes; nothing was committed.
    assert_eq!(controller.document().entities[&rail], before);
    assert_eq!(controller.coordinator().undo_len(), 0);

    // Without a selection the copy has no source.
    controller.clear_selection();
    assert_eq!(
        controller.builder_match_attributes().err().unwrap(),
        om9_core::Error::Missing("style source")
    );
}

#[test]
fn the_command_line_routes_style_apply_and_save_to_the_same_service() {
    let (mut controller, _rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    controller.submit("Style=Classic 2.2").unwrap();
    assert_eq!(
        controller
            .builder()
            .unwrap()
            .field("width_mm")
            .unwrap()
            .numeric(),
        Some(2.2)
    );
    controller.submit("StyleSave=Command 2.6").unwrap();
    assert!(
        controller
            .builder_styles()
            .iter()
            .any(|row| row.name == "Command 2.6" && !row.builtin)
    );
    // The same names the panel shows are the names the command line accepts.
    assert_eq!(
        controller.submit("Style=Missing").err().unwrap(),
        om9_core::Error::Missing("builder style")
    );
}

#[test]
fn cancel_and_apply_clear_the_applied_style_marker() {
    let (mut controller, rail) = controller_with_rail();
    controller.submit("RailProfile").unwrap();
    controller.builder_apply_style("Classic 2.2").unwrap();
    assert!(controller.builder_applied_style().is_some());
    controller.submit("Cancel").unwrap();
    assert_eq!(controller.builder_applied_style(), None);

    // Applying the feature commits and clears the marker too.
    controller.select_objects(&[rail]).unwrap();
    controller.submit("RailProfile").unwrap();
    controller.builder_apply_style("Classic 2.2").unwrap();
    controller.submit("").unwrap();
    assert!(controller.builder().is_none());
    assert_eq!(controller.builder_applied_style(), None);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn a_locked_input_keeps_the_browser_read_only() {
    let (mut controller, rail) = controller_with_rail();
    // Lock after selection, the way the property editor does; the selection survives so the
    // builder can open on a now read-only input.
    let stamp = controller.document().stamp(rail).unwrap();
    controller
        .edit_object_property(vec![stamp], om9_app::session::PropertyEdit::Locked(true))
        .unwrap();
    controller.submit("RailProfile").unwrap();
    assert!(!controller.builder_editable());
    // The list still shows (a style browser is not an edit), but loading one is refused.
    assert_eq!(controller.builder_styles().len(), 3);
    assert_eq!(
        controller.builder_apply_style("Classic 2.2").err().unwrap(),
        om9_core::Error::Invalid("builder input locked")
    );
}
