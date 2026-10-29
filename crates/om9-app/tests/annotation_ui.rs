// SPDX-License-Identifier: LGPL-2.1-or-later
//! Real egui input/layout/output frames; this does not claim GPU desktop presentation.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Shape, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{annotation::Annotation, model::Document};
fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    out.textures_delta.clear();
    out
}
fn click(app: &mut MatrixApp, ctx: &egui::Context, pos: Pos2) {
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
}
fn typed(app: &mut MatrixApp, ctx: &egui::Context, text: &str) {
    frame(app, ctx, vec![Event::Text(text.into())]);
    frame(
        app,
        ctx,
        vec![Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
}
fn app() -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("annotation UI")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    (app, ctx)
}
fn texts(shape: &Shape, out: &mut Vec<(String, Pos2)>) {
    match shape {
        Shape::Text(t) => out.push((t.galley.job.text.clone(), t.pos + t.galley.size() / 2.)),
        Shape::Vec(v) => {
            for s in v {
                texts(s, out)
            }
        }
        _ => {}
    }
}
fn output_texts(output: &egui::FullOutput) -> Vec<(String, Pos2)> {
    let mut out = vec![];
    for c in &output.shapes {
        texts(&c.shape, &mut out);
    }
    out
}
#[test]
fn the_measure_palette_horizontal_icon_creates_a_visible_dimension_label() {
    let (mut app, ctx) = app();
    let at = app.top_menu_rect(3).unwrap().center();
    click(&mut app, &ctx, at);
    frame(&mut app, &ctx, vec![]);
    let at = app.rail_icon_rects[2 + 4].center();
    click(&mut app, &ctx, at);
    assert_eq!(app.controller.active_feature(), Some("OM9-MEASURE-004"));
    for value in ["0,0,0", "10,0,0", "4,7,0"] {
        typed(&mut app, &ctx, value);
    }
    assert_eq!(app.undo_len(), 1);
    let entity = app.controller.document().entities.values().next().unwrap();
    let record = Annotation::read(entity).unwrap();
    assert_eq!(
        record
            .layout(&om9_core::budget::Limits::default())
            .unwrap()
            .value,
        Some(10.)
    );
    let output = frame(&mut app, &ctx, vec![]);
    assert!(
        output_texts(&output)
            .iter()
            .any(|(text, _)| text == "10.00 mm")
    );
}
#[test]
fn object_summary_is_reached_by_the_analyze_menu() {
    let (mut app, ctx) = app();
    let at = app.menu_rect("Analyze").unwrap().center();
    click(&mut app, &ctx, at);
    let output = frame(&mut app, &ctx, vec![]);
    let at = output_texts(&output)
        .iter()
        .find(|(text, _)| text == "Object Summary")
        .unwrap()
        .1;
    click(&mut app, &ctx, at);
    assert!(app.transcript.history().contains("Objects: 0 total"));
    assert_eq!(app.undo_len(), 0);
}
#[test]
fn hovering_distance_draws_a_value_before_commit_and_keeps_the_document_unchanged() {
    let (mut app, ctx) = app();
    typed(&mut app, &ctx, "Distance");
    typed(&mut app, &ctx, "0,0,0");
    let before = app.controller.document().clone();
    let rect = app.view_rects[&app.controller.workspace.active];
    let at = rect.center() + Vec2::new(20., 30.);
    frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
    let output = frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
    assert!(
        output_texts(&output)
            .iter()
            .any(|(text, _)| text.starts_with("Distance = "))
    );
    assert!(app.controller.is_running());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.undo_len(), 0);
}
