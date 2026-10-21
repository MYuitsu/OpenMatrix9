// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-007 (Object Property Editor Adapter), driven
//! by `implementation/fixtures/M01/OM9-L3-03-14-007/`.
use om9_app::session::{Controller, FIELDS, InvocationOrigin, LINE, PropertyEdit, PropertyField};
use om9_core::budget::Limits;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-007")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn controller_with_line() -> (Controller, om9_core::identity::ObjectId) {
    let mut controller =
        Controller::new(Document::new("properties"), Limits::default()).expect("controller");
    controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("dispatch");
    controller.accept_world_point([0., 0., 0.]).expect("first");
    controller
        .accept_world_point([10., 0., 0.])
        .expect("second");
    let object = *controller
        .document()
        .entities
        .keys()
        .next()
        .expect("object");
    (controller, object)
}

fn sources(
    controller: &Controller,
    object: om9_core::identity::ObjectId,
) -> Vec<om9_core::identity::SourceStamp> {
    controller
        .collect_object_sources(std::iter::once(object))
        .expect("sources")
}

#[test]
fn e1_to_e4_an_edit_is_one_command_and_undo_restores_it() {
    let document = fixture("positive.json");
    let (mut controller, object) = controller_with_line();

    // E1: colour, one journal entry, undo restores the previous value.
    let e1 = &document["cases"][0];
    let before = controller.document().entities[&object].attributes.color;
    let journal = controller.coordinator().undo_len();
    controller
        .edit_object_property(
            sources(&controller, object),
            PropertyEdit::Color(Some([255, 0, 0, 255])),
        )
        .expect("colour edit");
    assert_eq!(
        controller.document().entities[&object].attributes.color,
        Some([255, 0, 0, 255])
    );
    assert_eq!(
        controller.coordinator().undo_len() as u64,
        journal as u64 + e1["expect"]["journal_delta"].as_u64().unwrap()
    );
    controller.undo().expect("undo");
    assert_eq!(
        controller.document().entities[&object].attributes.color,
        before
    );

    // E2: the hidden flag follows the same rule.
    let e2 = &document["cases"][1];
    let journal = controller.coordinator().undo_len();
    controller
        .edit_object_property(sources(&controller, object), PropertyEdit::Hidden(true))
        .expect("hide edit");
    assert!(controller.document().entities[&object].attributes.hidden);
    assert_eq!(
        controller.coordinator().undo_len() as u64,
        journal as u64 + e2["expect"]["journal_delta"].as_u64().unwrap()
    );
    controller.undo().expect("undo");
    assert!(!controller.document().entities[&object].attributes.hidden);

    // E3: the editable and read-only fields are exactly the documented ones.
    let e3 = &document["cases"][2];
    let editable: Vec<&str> = FIELDS
        .iter()
        .filter(|state| state.editable)
        .map(|state| field_name(state.field))
        .collect();
    let read_only: Vec<&str> = FIELDS
        .iter()
        .filter(|state| !state.editable)
        .map(|state| field_name(state.field))
        .collect();
    let expected: Vec<&str> = e3["expect"]["editable"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(editable, expected);
    let expected_ro: Vec<&str> = e3["expect"]["read_only"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(read_only, expected_ro);

    // E4: a no-op edit writes nothing.
    let e4 = &document["cases"][3];
    let journal = controller.coordinator().undo_len();
    let same = controller.document().entities[&object].attributes.color;
    controller
        .edit_object_property(sources(&controller, object), PropertyEdit::Color(same))
        .expect("no-op edit");
    assert_eq!(
        controller.coordinator().undo_len() as u64,
        journal as u64 + e4["expect"]["journal_delta"].as_u64().unwrap()
    );
}

fn field_name(field: PropertyField) -> &'static str {
    match field {
        PropertyField::Name => "Name",
        PropertyField::Color => "Color",
        PropertyField::Visible => "Visible",
        PropertyField::Hidden => "Hidden",
        PropertyField::Selectable => "Selectable",
        PropertyField::Locked => "Locked",
        PropertyField::Geometry => "Geometry",
    }
}

#[test]
fn f1_to_f4_read_only_fields_are_disabled_and_a_locked_layer_refuses_the_edit() {
    let document = fixture("negative.json");

    // F1: an empty or padded label is refused by the document rule.
    let f1 = &document["cases"][0];
    let (mut controller, object) = controller_with_line();
    let journal = controller.coordinator().undo_len();
    let error = controller
        .edit_object_property(
            sources(&controller, object),
            PropertyEdit::Label("  ".into()),
        )
        .expect_err("a padded label is refused");
    assert_eq!(
        match &error {
            om9_core::Error::Invalid(_) => "Invalid",
            other => panic!("unexpected {other}"),
        },
        f1["expect"]["error"].as_str().unwrap()
    );
    assert_eq!(controller.coordinator().undo_len(), journal);

    // F2: the geometry is still read-only.
    let f2 = &document["cases"][1];
    let geometry = FIELDS
        .iter()
        .find(|state| state.field == PropertyField::Geometry)
        .expect("geometry");
    assert_eq!(
        geometry.editable,
        f2["expect"]["editable"].as_bool().unwrap()
    );
    assert!(
        geometry
            .reason
            .contains(f2["expect"]["reason_contains"].as_str().unwrap()),
        "{}",
        geometry.reason
    );

    // F3/F4: an edit on an object whose layer is locked is refused and changes
    // nothing. The document starts with no layer, so this uses the hidden flag on
    // a layer the test locks first.
    let (mut controller, object) = controller_with_line();
    // Move it onto a layer first, THEN lock that layer: the assignment itself is
    // refused on a locked layer (OM9-L3-03-14-002), the property edit below is the
    // thing this case is about.
    controller
        .assign_layer(
            om9_core::layers::PresetSlot::new(1).expect("slot"),
            sources(&controller, object),
        )
        .expect("assign while unlocked");
    controller
        .set_layer_property(
            om9_core::layers::PresetSlot::new(1).expect("slot"),
            om9_app::layer_command::LayerProperty::Locked(true),
        )
        .expect("lock Metal 01");
    // The object now sits on a locked layer, so the core refuses the attribute edit.
    let f3 = &document["cases"][2];
    let journal = controller.coordinator().undo_len();
    let before = controller.document().entities[&object].attributes.color;
    let error = controller
        .edit_object_property(
            sources(&controller, object),
            PropertyEdit::Color(Some([1, 2, 3, 255])),
        )
        .expect_err("a locked layer refuses the edit");
    assert_eq!(
        match &error {
            om9_core::Error::Permission(_) => "Permission",
            other => panic!("unexpected {other}"),
        },
        f3["expect"]["error"].as_str().unwrap()
    );
    assert_eq!(controller.coordinator().undo_len(), journal);
    assert_eq!(
        controller.document().entities[&object].attributes.color,
        before
    );
    assert_eq!(document["cases"][3]["expect"]["value_unchanged"], true);
}

#[test]
fn e5_renaming_is_one_command_and_undo_restores_the_name() {
    let (mut controller, object) = controller_with_line();
    let journal = controller.coordinator().undo_len();
    let before = controller.document().entities[&object].label.clone();
    controller
        .edit_object_property(
            sources(&controller, object),
            PropertyEdit::Label("Gem rail".into()),
        )
        .expect("rename");
    assert_eq!(controller.document().entities[&object].label, "Gem rail");
    assert_eq!(controller.coordinator().undo_len(), journal + 1);
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities[&object].label, before);
}

#[test]
fn the_app_adapter_edits_the_single_selection() {
    use om9_app::ui::MatrixApp;
    let (controller, object) = controller_with_line();
    let document = controller.document().clone();
    let mut app = MatrixApp::new(document).expect("app");
    // Nothing selected: the adapter refuses instead of guessing.
    assert!(
        app.edit_selected_property(PropertyEdit::Visible(false))
            .is_err()
    );
    app.select_objects(&[object]).expect("select");
    let journal = app.undo_len();
    app.edit_selected_property(PropertyEdit::Visible(false))
        .expect("hide through the adapter");
    assert_eq!(app.undo_len(), journal + 1);
}
