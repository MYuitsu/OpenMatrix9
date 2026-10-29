// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-03-20-005 at the UI seam: a Measure read-out appears in the Command Line
//! and never touches the document.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn enter() -> Event {
    Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) {
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
}

fn typed(app: &mut MatrixApp, context: &egui::Context, text: &str) {
    frame(app, context, vec![Event::Text(text.into())]);
    frame(app, context, vec![enter()]);
}

fn three_four_five() -> Geometry {
    Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [3.0, 4.0, 0.0],
        domain: [0.0, 5.0],
    })
}

fn app_with_line() -> (MatrixApp, egui::Context, om9_core::identity::ObjectId) {
    let line = Entity::new("line", three_four_five(), None);
    let id = line.id;
    let mut document = Document::new("measure feedback");
    document.entities.insert(id, line);
    let mut app = MatrixApp::new(document).expect("app");
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    (app, context, id)
}

#[test]
fn the_length_read_out_reaches_the_command_line_without_mutating_the_document() {
    let (mut app, context, id) = app_with_line();
    let undo = app.undo_len();
    assert_eq!(app.status_line(), "Ready: Ready");

    app.select_objects(&[id]).expect("select the line");
    typed(&mut app, &context, "Length");

    assert_eq!(
        app.status_line(),
        "Info: Length = 5 mm",
        "the status line carries the measured value, not a bare Ready"
    );
    assert!(
        app.transcript
            .history()
            .lines()
            .any(|line| line == "Length = 5 mm"),
        "the read-out is echoed in the Command Line history: {}",
        app.transcript.history()
    );
    assert_eq!(app.undo_len(), undo, "measuring is not an Undo step");
    assert_eq!(app.controller.document().entities.len(), 1);
    assert!(!app.controller.is_running());
}

#[test]
fn the_measure_palette_icon_dispatches_the_length_read_out() {
    let (mut app, context, id) = app_with_line();
    // Nothing has run yet, so Icon History draws exactly Undo and Redo before the
    // main menu palette; the Measure palette then starts at index 2.
    assert!(app.controller.recent().is_empty());
    let measure = app.top_menu_rect(3).expect("Measure top menu").center();
    click(&mut app, &context, measure);
    assert_eq!(app.top_menu_selected(), Some(3));
    frame(&mut app, &context, vec![]);
    assert!(
        app.rail_icon_rects.len() >= 2 + om9_app::menu_catalog::MEASURE.len(),
        "the Measure palette must draw its icons: {}",
        app.rail_icon_rects.len()
    );

    app.select_objects(&[id]).expect("select the line");
    // Measure palette order: AnalyzeAngle, AnalyzeDistance, AnalyzeLength, …
    let length = app.rail_icon_rects[2 + 2].center();
    click(&mut app, &context, length);

    assert_eq!(app.status_line(), "Info: Length = 5 mm");
    assert_eq!(app.undo_len(), 0);
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn the_distance_prompts_and_final_label_are_the_command_line_state() {
    let (mut app, context, _id) = app_with_line();
    typed(&mut app, &context, "Distance");
    assert_eq!(
        app.status_line(),
        "Info: First distance point:  — type a value and press Enter"
    );
    typed(&mut app, &context, "0,0,0");
    assert_eq!(
        app.status_line(),
        "Info: Second distance point:  — type a value and press Enter"
    );
    typed(&mut app, &context, "3,4,0");
    assert_eq!(app.status_line(), "Info: Distance = 5 mm");
    assert_eq!(
        app.transcript
            .history()
            .lines()
            .filter(|line| *line == "Distance = 5 mm")
            .count(),
        1,
        "the label is shown once: {}",
        app.transcript.history()
    );
    assert_eq!(app.undo_len(), 0);
    assert_eq!(app.controller.document().entities.len(), 1);
}
