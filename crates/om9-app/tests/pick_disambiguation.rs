// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ambiguous picking: a click that lands on two objects asks which one, instead of guessing.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

/// Two points on the same view ray: the farther one hides exactly behind the nearer one.
fn document_with_overlap() -> (
    Document,
    om9_core::identity::ObjectId,
    om9_core::identity::ObjectId,
) {
    let mut document = Document::with_template_tolerance("ambiguous");
    let near = Entity::new("near", Geometry::Point([0.0, 0.0, 0.0]), None);
    let near_id = near.id;
    // The default view looks along the opposite of this direction, so this point projects onto the same
    // pixel as the origin.
    let direction = [-35.0f64, 35.0, -35.0];
    let length =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    let far = Entity::new(
        "far",
        Geometry::Point([
            direction[0] / length * 6.0,
            direction[1] / length * 6.0,
            direction[2] / length * 6.0,
        ]),
        None,
    );
    let far_id = far.id;
    document.entities.insert(near_id, near);
    document.entities.insert(far_id, far);
    (document, near_id, far_id)
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "create an isolated desktop overlap fixture"]
fn write_overlapping_brep_desktop_fixture() {
    use sha2::{Digest, Sha256};
    use std::io::Write;

    let path = std::env::var_os("OM9_OVERLAP_DESKTOP_FIXTURE").expect("new fixture path");
    let path = std::path::Path::new(&path);
    assert!(!path.exists());
    let limits = om9_core::budget::Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Sphere { radius_mm: 3. },
        0.01,
        &limits,
    )
    .unwrap();
    let legacy_geometry = serde_json::json!({"Brep": &body});
    let (source, _) = body.into_canonical_and_representation();
    let mut document = Document::with_template_tolerance("overlap desktop fixture");
    for label in ["sphere", "Boolean Union"] {
        let entity = Entity::new(label, Geometry::Brep(source.clone()), None);
        document.entities.insert(entity.id, entity);
    }

    // This fixture uses the legacy archive's paired source and display format.
    let mut metadata = serde_json::to_value(document).unwrap();
    for entity in metadata["entities"].as_object_mut().unwrap().values_mut() {
        entity["geometry"] = legacy_geometry.clone();
    }
    let metadata = serde_json::to_vec(&metadata).unwrap();
    let template = include_bytes!("../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/review-fixtures/Boolean2-D-after.om9");
    let start = u64::from_le_bytes(template[40..48].try_into().unwrap()) as usize;
    let mut fixture = template[..start].to_vec();
    fixture[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
    fixture[56..88].copy_from_slice(&Sha256::digest(&metadata));
    fixture.extend_from_slice(&metadata);
    let mut file = std::fs::File::create_new(path).unwrap();
    file.write_all(&fixture).unwrap();
    drop(file);
    let mut app = MatrixApp::from_file(path).unwrap();
    let before = app.controller.document().clone();
    assert_eq!(before.entities.len(), 2);
    assert!(before.entities.values().all(|entity| matches!(entity.geometry, Geometry::Brep(_))));
    let context = egui::Context::default();
    frames(&mut app, &context, 4);
    let position = screen_of_origin(&app);
    click(&mut app, &context, position);
    let candidates = app.ambiguous_candidates();
    assert_eq!(candidates.len(), 2);
    let expected = candidates[0];
    click(&mut app, &context, position + Vec2::new(35., 35.));
    assert!(app.ambiguous_candidates().is_empty());
    assert_eq!(app.controller.selected().len(), 1);
    assert!(app.controller.selected().contains(&expected));
    assert_end_shortcut(&mut app, &context);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

fn frames(app: &mut MatrixApp, context: &egui::Context, count: usize) {
    for _ in 0..count {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                events: vec![Event::ModifiersChanged(Modifiers::NONE)],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
}

fn click(app: &mut MatrixApp, context: &egui::Context, position: Pos2) {
    for event in [
        Event::PointerMoved(position),
        Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ] {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                events: vec![event],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    frames(app, context, 2);
}

fn assert_end_shortcut(app: &mut MatrixApp, context: &egui::Context) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events: vec![
                Event::Key {
                    key: egui::Key::E,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
                Event::Text("e".into()),
            ],
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    assert_eq!(app.snap_override(), Some(om9_scene::osnap::SnapKind::End));
    assert!(app.transcript.draft().is_empty());
    assert!(!context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    assert!(!app.controller.is_running());
}

fn screen_of_origin(app: &MatrixApp) -> Pos2 {
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
    let projected = camera
        .project([0.0, 0.0, 0.0], viewport)
        .expect("project")
        .expect("front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

#[test]
fn a_click_on_two_objects_offers_a_choice_and_the_choice_selects() {
    let context = egui::Context::default();
    let (document, near, far) = document_with_overlap();
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    let position = screen_of_origin(&app);
    click(&mut app, &context, position);
    let candidates = app.ambiguous_candidates().to_vec();
    assert_eq!(
        candidates.len(),
        2,
        "both points sit under the pointer: {candidates:?}"
    );
    assert_eq!(candidates[0], near, "the nearest object is offered first");
    assert_eq!(candidates[1], far);
    app.choose_ambiguous(1).expect("choose the far object");
    assert_eq!(app.controller.selection_state().objects.len(), 1);
    assert!(
        app.controller.selected().contains(&far),
        "the chosen object is the selected one"
    );
    assert!(
        app.ambiguous_candidates().is_empty(),
        "the popup closes after a choice"
    );
}

#[test]
fn clicking_popup_row_selects_without_reopening_popup() {
    let context = egui::Context::default();
    let (document, near, _) = document_with_overlap();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 4);
    let position = screen_of_origin(&app);
    click(&mut app, &context, position);
    assert_eq!(app.ambiguous_candidates().len(), 2);
    // The popup begins at the original click; its first button is below its title.
    click(&mut app, &context, position + Vec2::new(35., 35.));
    assert!(app.ambiguous_candidates().is_empty(), "choosing a row must not reopen the popup");
    assert!(app.controller.selected().contains(&near), "the clicked row must select its object");
    assert_end_shortcut(&mut app, &context);
}

#[test]
fn a_click_on_one_object_selects_it_without_a_popup() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("single");
    let entity = Entity::new("only", Geometry::Point([0.0, 0.0, 0.0]), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    let position = screen_of_origin(&app);
    click(&mut app, &context, position);
    assert!(
        app.ambiguous_candidates().is_empty(),
        "no ambiguity, no popup"
    );
    assert!(app.controller.selected().contains(&id));
}

#[test]
fn escaping_the_choice_keeps_the_previous_selection() {
    let context = egui::Context::default();
    let (document, near, _) = document_with_overlap();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 4);
    app.controller.select_objects(&[near]).unwrap();
    let pos = screen_of_origin(&app);
    click(&mut app, &context, pos);
    assert_eq!(app.ambiguous_candidates().len(), 2);
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events: vec![Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    assert!(app.ambiguous_candidates().is_empty());
    assert!(app.controller.selected().contains(&near));
    assert_end_shortcut(&mut app, &context);
}

#[test]
fn a_choice_is_refused_if_the_document_changed_while_the_popup_was_open() {
    let context = egui::Context::default();
    let (document, near, _) = document_with_overlap();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 4);
    let pos = screen_of_origin(&app);
    click(&mut app, &context, pos);
    app.controller.delete_objects(&[near]).unwrap();
    assert!(
        app.choose_ambiguous(1).is_err(),
        "even a surviving candidate must be queried again in the new publication"
    );
    assert!(app.controller.selected().is_empty());
}

#[test]
fn the_popup_rows_name_the_type_and_escape_closes_without_selecting() {
    let context = egui::Context::default();
    let (document, near, _far) = document_with_overlap();
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    let position = screen_of_origin(&app);
    click(&mut app, &context, position);
    let labels = app.ambiguous_labels();
    assert_eq!(labels.len(), 2, "two candidates: {labels:?}");
    assert!(
        labels.iter().all(|label| label.contains("[Point]")),
        "each row names the geometry type: {labels:?}"
    );
    // Esc closes the chooser and leaves the selection as it was: nothing selected yet.
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events: vec![Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    frames(&mut app, &context, 2);
    assert!(
        app.ambiguous_candidates().is_empty(),
        "Esc closes the popup"
    );
    assert!(
        app.controller.selected().is_empty(),
        "Esc on the popup must not change the selection"
    );
    let _ = near;
}
