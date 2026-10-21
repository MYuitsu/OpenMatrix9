// SPDX-License-Identifier: LGPL-2.1-or-later
//! The property editor in the Edit menu (OM9-L3-03-14-007): one toggle is one
//! transaction, applied through MatrixApp::edit_selected_property.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::geometry::{Curve, Geometry};
use om9_core::model::{Document, Entity};

const SIZE: Vec2 = Vec2::new(1600., 1000.);

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SIZE)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn until(app: &mut MatrixApp, context: &egui::Context, key: &str) -> Rect {
    for _ in 0..30 {
        frame(app, context, Vec::new());
        if let Some(rect) = app.menu_rect(key) {
            return rect;
        }
    }
    panic!("menu rect {key} never appeared");
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) {
    frame(app, context, vec![Event::PointerMoved(at)]);
    for pressed in [true, false] {
        frame(
            app,
            context,
            vec![Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
}

#[test]
fn locking_from_the_edit_menu_is_one_undo_step() {
    let mut document = Document::new("property menu");
    let line = Entity::new(
        "Line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 4., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = line.id;
    document.entities.insert(id, line);
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).expect("app");
    app.select_objects(&[id]).expect("select");
    let before = app.undo_len();
    let edit = until(&mut app, &context, "Edit");
    click(&mut app, &context, edit.center());
    let locked = until(&mut app, &context, "prop-locked");
    click(&mut app, &context, locked.center());
    for _ in 0..3 {
        frame(&mut app, &context, Vec::new());
    }
    assert_eq!(app.undo_len(), before + 1, "one toggle is one transaction");
}
