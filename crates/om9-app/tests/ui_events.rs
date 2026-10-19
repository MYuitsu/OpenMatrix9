// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1280., 800.),
        )),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| app.show(ui));
    // This test exercises the widget reducer without a renderer. Native tests
    // separately apply these deltas through wgpu and inspect the screenshot.
    output.textures_delta.clear();
    output
}
fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn text(app: &mut MatrixApp, ctx: &egui::Context, value: &str) {
    frame(app, ctx, vec![Event::Text(value.into())]);
    frame(app, ctx, vec![key(Key::Enter, Modifiers::NONE)]);
}
#[test]
fn rendered_cmd_events_build_line_undo_redo_and_cancel_without_direct_model_writes() {
    let mut app = MatrixApp::new(Document::new("egui")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.view_rects.len(), 4);
    text(&mut app, &ctx, "Line");
    text(&mut app, &ctx, "0,0,0");
    assert!(app.controller.document().entities.is_empty());
    text(&mut app, &ctx, "12,6,0");
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    text(&mut app, &ctx, "Undo");
    assert!(app.controller.document().entities.is_empty());
    text(&mut app, &ctx, "Redo");
    assert_eq!(app.controller.document().entities.len(), 1);
    text(&mut app, &ctx, "Line");
    text(&mut app, &ctx, "2,2,0");
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!app.controller.is_running());
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert_eq!(app.controller.document().entities.len(), 1);
}
#[test]
fn actual_editor_select_all_cut_paste_cannot_mutate_transcript_prefix_or_execute_paste() {
    let mut app = MatrixApp::new(Document::new("text")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    text(&mut app, &ctx, "Point");
    text(&mut app, &ctx, "1,2,3");
    let history = app.transcript.history().to_owned();
    frame(
        &mut app,
        &ctx,
        vec![
            key(
                Key::A,
                Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                },
            ),
            Event::Cut,
        ],
    );
    assert!(app.transcript.text().starts_with(&history));
    assert_eq!(app.controller.document().entities.len(), 1);
    frame(
        &mut app,
        &ctx,
        vec![Event::Paste("Line\n0,0,0\n9,9,0".into())],
    );
    assert_eq!(app.controller.document().entities.len(), 1);
    assert!(app.transcript.text().starts_with(&history));
    assert!(!app.controller.is_running());
}

#[test]
fn f6_remains_open_between_frames_and_unpinned_escape_restores_command_focus() {
    let mut app = MatrixApp::new(Document::new("F6")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(Key::F6, Modifiers::NONE)]);
    assert!(app.f6_open);
    frame(&mut app, &ctx, vec![]);
    assert!(app.f6_open);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!app.f6_open);
    text(&mut app, &ctx, "Point");
    text(&mut app, &ctx, "3,4,5");
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn printable_viewport_input_routes_to_cmd_when_no_other_editor_owns_focus() {
    let mut app = MatrixApp::new(Document::new("focus")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let cmd = egui::Id::new("matrix-command-transcript");
    ctx.memory_mut(|m| m.surrender_focus(cmd));
    text(&mut app, &ctx, "Point");
    assert!(app.controller.is_running());
}

#[test]
fn reference_workspace_places_sidebar_left_and_through_finger_below_looking_down() {
    use om9_scene::workspace::ViewId;
    let mut app = MatrixApp::new(Document::new("reference layout")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let looking = app.view_rects[&ViewId::LookingDown];
    let perspective = app.view_rects[&ViewId::Perspective];
    let finger = app.view_rects[&ViewId::ThroughFinger];
    let side = app.view_rects[&ViewId::SideView];
    assert!(looking.left() >= 280.);
    assert!(looking.top() >= 150., "{looking:?}");
    assert_eq!(looking.left(), finger.left());
    assert_eq!(perspective.left(), side.left());
    assert!(finger.top() > looking.top());
    assert!(side.top() > perspective.top());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn construction_grid_colors_only_positive_axes_like_the_reference() {
    use om9_scene::workspace::ViewId;
    let mut app = MatrixApp::new(Document::new("grid reference")).unwrap();
    let ctx = egui::Context::default();
    let output = frame(&mut app, &ctx, vec![]);
    let view = app.view_rects[&ViewId::LookingDown];
    let red = egui::Color32::from_rgb(210, 35, 35);
    let axis = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::LineSegment { points, stroke }
                if stroke.color == red && points.iter().all(|p| view.contains(*p)) =>
            {
                Some(points)
            }
            _ => None,
        })
        .unwrap();
    assert!(
        (axis[0].x - view.center().x).abs() < 1.,
        "colored axis starts at origin"
    );
    assert!(axis[1].x > view.center().x);
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
