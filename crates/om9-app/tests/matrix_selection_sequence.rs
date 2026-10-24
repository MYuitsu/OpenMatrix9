// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-03-001`: Matrix plain/Shift/Ctrl click mapping and locked refusal.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))
}

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn viewport(app: &MatrixApp) -> Viewport {
    let rect = app.view_rects[&ViewId::LookingDown];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let projected = app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .project(point, viewport(app))
        .unwrap()
        .unwrap();
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2, modifiers: Modifiers) {
    frame(app, context, vec![Event::PointerMoved(at)]);
    frame(
        app,
        context,
        vec![
            Event::ModifiersChanged(modifiers),
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed: true,
                modifiers,
            },
        ],
    );
    frame(
        app,
        context,
        vec![Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed: false,
            modifiers,
        }],
    );
}

#[test]
fn plain_click_replaces_shift_adds_and_ctrl_toggles() {
    let context = egui::Context::default();
    let mut document = Document::new("selection mapping");
    let a = Entity::new("a", Geometry::Point([0., 0., 0.]), None);
    let a_id = a.id;
    document.entities.insert(a_id, a);
    let b = Entity::new("b", Geometry::Point([10., 0., 0.]), None);
    let b_id = b.id;
    document.entities.insert(b_id, b);
    let mut locked = Entity::new("locked", Geometry::Point([20., 0., 0.]), None);
    locked.attributes.selectable = false;
    locked.attributes.locked = true;
    let locked_id = locked.id;
    document.entities.insert(locked_id, locked);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;

    let a_pos = screen_of(&app, [0., 0., 0.]);
    let b_pos = screen_of(&app, [10., 0., 0.]);
    let locked_pos = screen_of(&app, [20., 0., 0.]);

    click(&mut app, &context, a_pos, Modifiers::NONE);
    assert_eq!(app.controller.selected(), &[a_id].into_iter().collect());
    click(&mut app, &context, b_pos, Modifiers::NONE);
    assert_eq!(app.controller.selected(), &[b_id].into_iter().collect());

    click(
        &mut app,
        &context,
        a_pos,
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(
        app.controller.selected(),
        &[a_id, b_id].into_iter().collect()
    );
    click(
        &mut app,
        &context,
        b_pos,
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(app.controller.selected(), &[a_id].into_iter().collect());
    click(
        &mut app,
        &context,
        b_pos,
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(
        app.controller.selected(),
        &[a_id, b_id].into_iter().collect()
    );

    click(&mut app, &context, locked_pos, Modifiers::NONE);
    assert!(
        !app.controller.selected().contains(&locked_id),
        "a locked object never enters the selection"
    );
}

#[test]
fn the_overlap_popup_names_the_type_and_layer_swatch() {
    let context = egui::Context::default();
    let mut document = Document::new("popup swatch");
    for (label, z, colour) in [
        ("near", 0., [10, 20, 30, 255]),
        ("far", -5., [40, 50, 60, 255]),
    ] {
        let mut entity = Entity::new(label, Geometry::Point([0., 0., z]), None);
        entity.attributes.color = Some(colour);
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    let at = screen_of(&app, [0., 0., 0.]);
    click(&mut app, &context, at, Modifiers::NONE);
    let labels = app.ambiguous_labels();
    assert_eq!(labels.len(), 2, "{labels:?}");
    assert!(
        labels.iter().all(|label| label.contains("[Point]")),
        "{labels:?}"
    );
    let swatches = app.ambiguous_swatches();
    assert_eq!(swatches.len(), 2);
    assert!(swatches.iter().all(Option::is_some), "{swatches:?}");
}
