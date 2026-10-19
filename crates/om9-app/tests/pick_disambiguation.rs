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
fn document_with_overlap() -> (Document, om9_core::identity::ObjectId, om9_core::identity::ObjectId) {
    let mut document = Document::new("ambiguous");
    let near = Entity::new("near", Geometry::Point([0.0, 0.0, 0.0]), None);
    let near_id = near.id;
    // The default view looks along the opposite of this direction, so this point projects onto the same
    // pixel as the origin.
    let direction = [-35.0f64, 35.0, -35.0];
    let length = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
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
        Event::PointerButton { pos: position, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        Event::PointerButton { pos: position, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
    ] {
        let mut output = context.run_ui(
            egui::RawInput { screen_rect: Some(screen_rect()), events: vec![event], ..Default::default() },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
    frames(app, context, 2);
}

fn screen_of_origin(app: &MatrixApp) -> Pos2 {
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
    let projected = camera.project([0.0, 0.0, 0.0], viewport).expect("project").expect("front");
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
    assert_eq!(candidates.len(), 2, "both points sit under the pointer: {candidates:?}");
    assert_eq!(candidates[0], near, "the nearest object is offered first");
    assert_eq!(candidates[1], far);
    app.choose_ambiguous(1).expect("choose the far object");
    assert_eq!(
        app.controller.selection_state().objects.len(),
        1
    );
    assert!(app.controller.selected().contains(&far), "the chosen object is the selected one");
    assert!(app.ambiguous_candidates().is_empty(), "the popup closes after a choice");
}

#[test]
fn a_click_on_one_object_selects_it_without_a_popup() {
    let context = egui::Context::default();
    let mut document = Document::new("single");
    let entity = Entity::new("only", Geometry::Point([0.0, 0.0, 0.0]), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    let position = screen_of_origin(&app);
    click(&mut app, &context, position);
    assert!(app.ambiguous_candidates().is_empty(), "no ambiguity, no popup");
    assert!(app.controller.selected().contains(&id));
}

#[test]
fn escaping_the_choice_keeps_the_previous_selection() {
    let context=egui::Context::default();
    let (document,near,_)=document_with_overlap();
    let mut app=MatrixApp::new(document).unwrap();
    frames(&mut app,&context,4);
    app.controller.select_objects(&[near]).unwrap();
    let pos=screen_of_origin(&app);
    click(&mut app,&context,pos);
    assert_eq!(app.ambiguous_candidates().len(),2);
    let mut output=context.run_ui(egui::RawInput {screen_rect:Some(screen_rect()),events:vec![Event::Key {key:egui::Key::Escape,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}],..Default::default()},|ui|app.show(ui));
    output.textures_delta.clear();
    assert!(app.ambiguous_candidates().is_empty());
    assert!(app.controller.selected().contains(&near));
}

#[test]
fn a_choice_is_refused_if_the_document_changed_while_the_popup_was_open() {
    let context=egui::Context::default();
    let (document,near,_)=document_with_overlap();
    let mut app=MatrixApp::new(document).unwrap(); frames(&mut app,&context,4);
    let pos=screen_of_origin(&app); click(&mut app,&context,pos);
    app.controller.delete_objects(&[near]).unwrap();
    assert!(app.choose_ambiguous(1).is_err(),"even a surviving candidate must be queried again in the new publication");
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
    assert!(app.ambiguous_candidates().is_empty(), "Esc closes the popup");
    assert!(
        app.controller.selected().is_empty(),
        "Esc on the popup must not change the selection"
    );
    let _ = near;
}
