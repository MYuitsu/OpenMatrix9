// SPDX-License-Identifier: LGPL-2.1-or-later
//! Super Select behind the Tools menu (OM9-L3-03-14-006).
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::geometry::{Curve, Geometry, Mesh};
use om9_core::model::{Document, Entity, Extension, GEM_EXTENSION_SLOT};

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

fn gem_mesh() -> Mesh {
    Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: Vec::new(),
        colors: Vec::new(),
    }
}

#[test]
fn the_tools_menu_selects_the_semantic_gem() {
    let mut document = Document::new("super select");
    let mut gem = Entity::new("Gem", Geometry::Mesh(gem_mesh()), None);
    gem.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        Extension::new(1, "{\"ref\":\"gem-blue-round\"}").expect("payload"),
    );
    let gem_id = gem.id;
    document.entities.insert(gem_id, gem);
    let line = Entity::new(
        "Line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 4., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let line_id = line.id;
    document.entities.insert(line_id, line);

    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap_or_else(|error| panic!("app: {error:?}"));
    let tools = until(&mut app, &context, "Tools");
    click(&mut app, &context, tools.center());
    let item = until(&mut app, &context, "super-gems");
    click(&mut app, &context, item.center());
    for _ in 0..3 {
        frame(&mut app, &context, Vec::new());
    }
    println!(
        "selected: {:?}",
        app.selected_objects().iter().copied().collect::<Vec<_>>()
    );
    println!("status  : {:?}", app.status.message());
    assert_eq!(
        app.selected_objects().iter().copied().collect::<Vec<_>>(),
        vec![gem_id]
    );
}
