// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1280., 800.),
            )),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
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
#[test]
fn live_command_is_pinned_and_unwrapped_while_history_scrolls_and_copies_as_one_document() {
    let mut app = MatrixApp::new(Document::new("pinned input")).unwrap();
    let ctx = egui::Context::default();
    for i in 0..80 {
        app.transcript
            .append_history(&format!("History {i}: thông tin"));
    }
    frame(&mut app, &ctx, vec![]);
    let long_draft = "coordinate ".repeat(100);
    let output = frame(&mut app, &ctx, vec![Event::Text(long_draft.clone())]);
    let input = app.command_input_rect().unwrap();
    let history = app
        .command_history_rect()
        .expect("independent history scroll bounds");
    assert!(history.bottom() <= input.top());
    let live = format!("Command: {long_draft}");
    let live_shape = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == live => Some(t),
            _ => None,
        })
        .expect("pinned live row rendered even with long history");
    assert_eq!(live_shape.galley.rows.len(), 1, "draft must not wrap");
    assert!(live_shape.galley.size().x > input.width());
    assert_eq!(app.transcript.draft(), long_draft);
    let settled = frame(&mut app, &ctx, vec![]);
    let history_y = settled
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text().starts_with("History 0:") => Some(t.pos.y),
            _ => None,
        })
        .unwrap();
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(history.center()),
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0., 500.),
                phase: egui::TouchPhase::Move,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let scrolled = frame(&mut app, &ctx, vec![]);
    let scrolled_y = scrolled
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text().starts_with("History 0:") => Some(t.pos.y),
            _ => None,
        })
        .unwrap();
    assert!(scrolled_y > history_y, "wheel must actually scroll history");
    assert_eq!(
        app.command_input_rect(),
        Some(input),
        "history scrolling cannot move the prompt"
    );
    let ctrl = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    let output = frame(&mut app, &ctx, vec![key(Key::A, ctrl), Event::Copy]);
    assert!(
        output.platform_output.commands.iter().any(|c| matches!(c,
            egui::OutputCommand::CopyText(text) if text == app.transcript.text()
        )),
        "one selection copies history, prompt, and draft together: {:?}, focus {:?}, cursor {:?}",
        output.platform_output.commands,
        ctx.memory(|m| m.focused()),
        egui::TextEdit::load_state(&ctx, egui::Id::new("matrix-command-transcript"))
            .map(|s| s.cursor.char_range())
    );
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    let prefix = app.transcript.history().to_owned();
    frame(&mut app, &ctx, vec![Event::Cut]);
    assert_eq!(app.transcript.draft(), "");
    assert_eq!(app.transcript.history(), prefix);
}

#[test]
fn pinned_row_pointer_edits_unicode_suffix_and_preserves_prompt() {
    let mut app = MatrixApp::new(Document::new("pointer input")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("αβγ".into())]);
    let row = app.command_input_rect().unwrap();
    let prefix_width = ctx.fonts_mut(|f| {
        f.layout_no_wrap(
            "Command: α".into(),
            egui::FontId::monospace(12.),
            egui::Color32::BLACK,
        )
        .size()
        .x
    });
    let pos = egui::pos2(row.left() + prefix_width, row.center().y);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(&mut app, &ctx, vec![Event::Paste("đ\nộ".into())]);
    assert_eq!(app.transcript.draft(), "αđ ộβγ");
    assert_eq!(app.transcript.text(), "Command: αđ ộβγ");
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn viewport_title_click_preserves_numeric_draft_and_enter_accepts_it() {
    use om9_scene::workspace::ViewId;
    let mut app = MatrixApp::new(Document::new("viewport focus")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("Point".into())]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    frame(&mut app, &ctx, vec![Event::Text("3,2,1".into())]);
    let pos = app.view_rects[&ViewId::LookingDown].min + Vec2::new(20., 8.);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert_eq!(app.controller.workspace.active, ViewId::LookingDown);
    assert_eq!(app.transcript.draft(), "3,2,1");
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    let entity = app.controller.document().entities.values().next().unwrap();
    assert_eq!(
        entity.geometry,
        om9_core::geometry::Geometry::Point([3., 2., 1.])
    );
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn paste_that_retires_unicode_history_recomputes_layout_boundary_without_panicking() {
    let mut app = MatrixApp::new(Document::new("bounded history")).unwrap();
    let ctx = egui::Context::default();
    app.transcript.append_history(&"中".repeat(4096));
    for _ in 0..13 {
        app.transcript.append_history(&"h".repeat(4000));
    }
    app.transcript.append_history(&"x".repeat(700));
    frame(&mut app, &ctx, vec![]);
    assert!(app.transcript.history().starts_with('中'));
    let draft = "é".repeat(1800);
    let output = frame(&mut app, &ctx, vec![Event::Paste(draft.clone())]);
    assert!(!app.transcript.history().starts_with('中'));
    assert_eq!(app.transcript.draft(), draft);
    assert!(app.transcript.text().len() <= om9_app::transcript::Transcript::MAX_BYTES);
    let live = format!("Command: {draft}");
    assert!(output.shapes.iter().any(|s| matches!(&s.shape,
        egui::Shape::Text(t) if t.galley.text() == live
    )));
}

#[test]
fn dragging_from_pinned_suffix_to_history_selects_and_copies_the_same_document() {
    let mut app = MatrixApp::new(Document::new("cross region selection")).unwrap();
    let ctx = egui::Context::default();
    app.transcript.append_history("đầu");
    app.transcript.append_history("cuối");
    frame(&mut app, &ctx, vec![]);
    let output = frame(&mut app, &ctx, vec![Event::Text("draft".into())]);
    let history_origin = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text().starts_with("đầu\ncuối\n") => Some(t.pos),
            _ => None,
        })
        .unwrap();
    let start = app.command_input_rect().unwrap().right_center() - Vec2::new(4., 0.);
    let end = history_origin + Vec2::new(0., 6.);
    frame(&mut app, &ctx, vec![Event::PointerMoved(start)]);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    let output = frame(&mut app, &ctx, vec![Event::Copy]);
    assert!(
        output.platform_output.commands.iter().any(|c| matches!(c,
            egui::OutputCommand::CopyText(text) if text == app.transcript.text()
        )),
        "cross-region drag must copy the whole source: {:?}",
        output.platform_output.commands
    );
}

#[test]
fn ime_confirmation_enter_cannot_execute_provisional_command_or_coordinate() {
    let mut app = MatrixApp::new(Document::new("IME input")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(
        &mut app,
        &ctx,
        vec![Event::Ime(egui::ImeEvent::Preedit {
            text: "Point".into(),
            active_range_chars: Some(0..5),
        })],
    );
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(
        !app.controller.is_running(),
        "IME confirmation must not dispatch provisional Point"
    );
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Commit("Point".into())),
            key(Key::Enter, Modifiers::NONE),
        ],
    );
    assert!(
        !app.controller.is_running(),
        "same-frame IME commit/Enter must not dispatch"
    );
    assert_eq!(app.transcript.draft(), "Point");
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.controller.is_running());
    frame(
        &mut app,
        &ctx,
        vec![Event::Ime(egui::ImeEvent::Preedit {
            text: "1,2,3".into(),
            active_range_chars: None,
        })],
    );
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.controller.document().entities.is_empty());
    frame(
        &mut app,
        &ctx,
        vec![Event::Ime(egui::ImeEvent::Commit("1,2,3".into()))],
    );
    assert!(app.controller.document().entities.is_empty());
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    frame(&mut app, &ctx, vec![Event::Text("Point".into())]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "9,8,7".into(),
                active_range_chars: None,
            }),
            key(Key::Escape, Modifiers::NONE),
        ],
    );
    assert!(
        app.controller.is_running(),
        "IME dismissal must not cancel the active command"
    );
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn dragging_scrolled_history_into_pinned_row_keeps_shared_selection() {
    let mut app = MatrixApp::new(Document::new("history drag")).unwrap();
    let ctx = egui::Context::default();
    for i in 0..80 {
        app.transcript.append_history(&format!("H{i:02}"));
    }
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("draft".into())]);
    frame(&mut app, &ctx, vec![]);
    let history = app.command_history_rect().unwrap();
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(history.center()),
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0., 500.),
                phase: egui::TouchPhase::Move,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    for _ in 0..30 {
        frame(&mut app, &ctx, vec![]);
    }
    let output = frame(&mut app, &ctx, vec![]);
    let shape = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text().starts_with("H00\n") => Some(t),
            _ => None,
        })
        .unwrap();
    let visible = shape
        .galley
        .rows
        .iter()
        .find(|r| {
            let pos = shape.pos + r.rect().center().to_vec2();
            history.shrink2(Vec2::new(0., 5.)).contains(pos)
        })
        .unwrap();
    let label: String = visible.glyphs.iter().map(|g| g.chr).collect();
    let offset = app.transcript.text().find(&label).unwrap();
    let expected = app.transcript.text()[offset..].to_owned();
    assert!(
        label.starts_with('H') && label != "H79",
        "history is scrolled away from live row"
    );
    let start = egui::pos2(shape.pos.x, shape.pos.y + visible.rect().center().y);
    let end = app.command_input_rect().unwrap().right_center() - Vec2::new(4., 0.);
    frame(&mut app, &ctx, vec![Event::PointerMoved(start)]);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    let output = frame(&mut app, &ctx, vec![Event::Copy]);
    assert!(
        output.platform_output.commands.iter().any(|c| matches!(c,
            egui::OutputCommand::CopyText(text) if text == &expected
        )),
        "history drag must select into footer: {:?}",
        output.platform_output.commands
    );
}
