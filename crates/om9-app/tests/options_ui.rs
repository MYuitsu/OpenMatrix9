// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, Rect, Shape, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            focused: true,
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn click(app: &mut MatrixApp, ctx: &egui::Context, pos: Pos2) {
    frame(app, ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
}

fn text_position(app: &mut MatrixApp, ctx: &egui::Context, label: &str) -> Pos2 {
    fn find(shape: &Shape, label: &str) -> Option<Pos2> {
        match shape {
            Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.size() / 2.)
            }
            Shape::Vec(shapes) => shapes.iter().find_map(|shape| find(shape, label)),
            _ => None,
        }
    }
    for _ in 0..4 {
        let result = frame(app, ctx, vec![]);
        if let Some(pos) = result.shapes.iter().find_map(|shape| {
            find(&shape.shape, label).filter(|pos| shape.clip_rect.contains(*pos))
        }) {
            return pos;
        }
    }
    let output = frame(app, ctx, vec![]);
    let visible: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Text(text) if shape.clip_rect.contains(text.pos + text.galley.size() / 2.) => {
                Some(text.galley.job.text.as_str())
            }
            _ => None,
        })
        .collect();
    panic!("Options text not visible: {label}. Visible text: {visible:?}");
}

fn control_position(app: &mut MatrixApp, ctx: &egui::Context, id: &str) -> Pos2 {
    for _ in 0..3 {
        frame(app, ctx, vec![]);
    }
    ctx.data(|data| data.get_temp::<Rect>(egui::Id::new(id)))
        .unwrap_or_else(|| panic!("missing Options control: {id}"))
        .center()
}

fn open_options(app: &mut MatrixApp, ctx: &egui::Context) {
    frame(app, ctx, vec![]);
    let command = app.command_input_rect().unwrap().center();
    click(app, ctx, command);
    frame(app, ctx, vec![Event::Text("Options".into())]);
    press_key(app, ctx, Key::Enter);
    text_position(app, ctx, "Single-click maximize");
}

fn press_key(app: &mut MatrixApp, ctx: &egui::Context, key: Key) {
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
    }
}

#[test]
fn options_tree_grid_draft_cancel_and_apply() {
    let mut app = MatrixApp::new(Document::with_template_tolerance("Options test")).unwrap();
    let ctx = egui::Context::default();
    let original = app.controller.grid_snap();
    let document = app.controller.document().clone();
    for (button, expected) in [
        (None, original),
        (Some("options-cancel-rect"), original),
        (Some("undo-apply-rect"), !original),
    ] {
        open_options(&mut app, &ctx);
        let grid = control_position(&mut app, &ctx, "options-tree-grid-rect");
        click(&mut app, &ctx, grid);
        let snap = control_position(&mut app, &ctx, "options-grid-snap-rect");
        click(&mut app, &ctx, snap);
        assert_eq!(
            app.controller.grid_snap(),
            original,
            "draft applied before OK"
        );
        if let Some(button) = button {
            let close = control_position(&mut app, &ctx, button);
            click(&mut app, &ctx, close);
        } else {
            press_key(&mut app, &ctx, Key::Escape);
        }
        assert_eq!(app.controller.grid_snap(), expected);
        assert_eq!(app.controller.document(), &document);
    }
}

#[test]
fn options_single_click_maximize_changes_viewport_clicks() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(check_single_click_maximize)
        .unwrap()
        .join()
        .unwrap();
}

fn check_single_click_maximize() {
    let directory = std::env::temp_dir().join(format!("om9-options-test-{}", std::process::id()));
    let config = om9_app::startup::AppConfig {
        layout_path: Some(directory.join("layout.json")),
        ..Default::default()
    };
    let mut app = MatrixApp::start(&config).unwrap();
    let ctx = egui::Context::default();
    open_options(&mut app, &ctx);
    let toggle = control_position(&mut app, &ctx, "options-maximize-rect");
    click(&mut app, &ctx, toggle);
    let ok = control_position(&mut app, &ctx, "undo-apply-rect");
    click(&mut app, &ctx, ok);
    let preferences_file = directory.join("options.json");
    let preferences = std::fs::read_to_string(&preferences_file)
        .unwrap_or_else(|error| panic!("Options did not write changed preferences: {error}"));
    assert!(
        preferences.contains("\"single_click_maximize\":true"),
        "{preferences}"
    );
    let title = text_position(&mut app, &ctx, "Perspective");
    assert!(app.controller.workspace.maximized.is_none());
    click(&mut app, &ctx, title);
    assert_eq!(
        app.controller.workspace.maximized.map(|view| view.title()),
        Some("Perspective")
    );
    std::fs::remove_file(preferences_file).unwrap();
}
