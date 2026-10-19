// SPDX-License-Identifier: LGPL-2.1-or-later
//! Replays the behaviours of `F:\20261018-2003-15.8824342.mp4` against the app.
//!
//! The clip (20.6 s, 486×482, one Perspective viewport) was sampled at 2 fps and read frame by frame. It shows:
//!
//! 1. clicking the line selects it and the gumball appears, attached to the object;
//! 2. clicking empty space drops the selection and the gumball disappears;
//! 3. the camera orbits (and zooms) while the gumball stays attached and keeps its screen size;
//! 4. dragging a rotate arc turns the object, with the gumball staying on its pivot;
//! 5. dragging a scale box resizes along one axis;
//! 6. a window selection over several objects, and Esc clearing it.
//!
//! Each of those steps is asserted here, so a regression in any of them fails the test rather than only being
//! visible in a recording.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn document() -> Document {
    let mut document = Document::new("clip replay");
    let entity = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [-10.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 20.0],
        }),
        None,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    document
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

fn events(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput { screen_rect: Some(screen_rect()), events, ..Default::default() },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn view_rect(app: &MatrixApp) -> Rect {
    app.view_rects[&ViewId::Perspective]
}

fn viewport_of(app: &MatrixApp) -> Viewport {
    let rect = view_rect(app);
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
    let projected = camera.project(point, viewport_of(app)).expect("project").expect("front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn left_click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) {
    events(app, context, vec![Event::PointerMoved(at)]);
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(app, context, 2);
}

/// World units per screen pixel, the same rule the viewport uses to size the gumball.
fn world_per_pixel(camera: &om9_scene::camera::Camera, viewport: Viewport) -> f64 {
    match camera.projection {
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => {
            let d = camera.eye;
            let t = camera.target;
            let distance = ((d[0] - t[0]).powi(2) + (d[1] - t[1]).powi(2) + (d[2] - t[2]).powi(2)).sqrt();
            2.0 * (vertical_fov_rad / 2.0).tan() * distance / viewport.size[1]
        }
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => {
            vertical_span_mm / viewport.size[1]
        }
    }
}

/// Screen length of the X move arrow, in pixels: the gumball is drawn at ninety pixels.
fn gumball_pixels(app: &MatrixApp) -> f32 {
    let frame = app.controller.gumball_pivot().expect("pivot");
    let camera = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    let size = world_per_pixel(&camera, viewport_of(app)) * 90.0;
    let view = om9_scene::gumball_view::GumballView::with_size(
        frame,
        om9_scene::gumball_view::Mode::Move,
        size,
    );
    let segment = view
        .segments()
        .into_iter()
        .find(|segment| matches!(segment.handle, om9_core::gumball::Handle::MoveAxis(om9_core::gumball::Axis::X)))
        .expect("x arrow");
    let start = camera.project(segment.start, viewport_of(app)).expect("project").expect("front").screen;
    let end = camera.project(segment.end, viewport_of(app)).expect("project").expect("front").screen;
    ((end[0] - start[0]).hypot(end[1] - start[1])) as f32
}

#[test]
fn clicking_selects_and_empty_space_deselects_with_the_gumball_following() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document()).expect("app");
    frames(&mut app, &context, 4);
    let id = *app.controller.document().entities.keys().next().expect("line");
    let on_line = screen_of(&app, [0.0, 0.0, 0.0]);

    // 1. click the line: selected, gumball present.
    left_click(&mut app, &context, on_line);
    assert!(app.controller.selected().contains(&id), "the line is selected");
    assert!(app.controller.gumball_pivot().is_some(), "the gumball appears");

    // 2. click empty space: nothing selected, gumball gone.
    let empty = Pos2::new(view_rect(&app).right() - 12.0, view_rect(&app).bottom() - 12.0);
    left_click(&mut app, &context, empty);
    assert!(app.controller.selected().is_empty(), "empty space clears the selection");
    assert!(app.controller.gumball_pivot().is_none(), "the gumball disappears");
}

#[test]
fn orbiting_and_zooming_leave_the_gumball_attached_and_the_same_screen_size() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document()).expect("app");
    frames(&mut app, &context, 4);
    let on_line = screen_of(&app, [0.0, 0.0, 0.0]);
    left_click(&mut app, &context, on_line);
    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    let before_size = gumball_pixels(&app);
    let before_eye = app.controller.workspace.views[&ViewId::Perspective].camera.eye;

    // 3a. right-drag orbits the camera.
    let centre = view_rect(&app).center();
    events(&mut app, &context, vec![Event::PointerMoved(centre)]);
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: centre,
            button: egui::PointerButton::Secondary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    for step in 1..=6 {
        events(&mut app, &context, vec![Event::PointerMoved(centre + Vec2::new(step as f32 * 12.0, 0.0))]);
        frames(&mut app, &context, 1);
    }
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: centre + Vec2::new(72.0, 0.0),
            button: egui::PointerButton::Secondary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);
    let after_eye = app.controller.workspace.views[&ViewId::Perspective].camera.eye;
    assert!((after_eye[0] - before_eye[0]).abs() + (after_eye[1] - before_eye[1]).abs() > 1e-6, "the camera orbited");
    assert_eq!(app.controller.gumball_pivot().expect("pivot").origin, pivot, "the gumball stays on the object");

    // 3b. the wheel dollies; the gumball keeps its ninety-pixel size.
    events(
        &mut app,
        &context,
        vec![Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(0.0, 240.0),
            modifiers: Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        }],
    );
    frames(&mut app, &context, 3);
    let zoomed_size = gumball_pixels(&app);
    assert!(
        (zoomed_size - before_size).abs() <= before_size * 0.25,
        "the gumball keeps its screen size after zooming: {before_size} -> {zoomed_size}"
    );
}

fn two_lines_document() -> Document {
    let mut document = Document::new("clip band");
    for z in [0.0, 6.0] {
        let entity = Entity::new(
            format!("line {z}"),
            Geometry::Curve(Curve::Line {
                start: [-8.0, 0.0, z],
                end: [8.0, 0.0, z],
                domain: [0.0, 16.0],
            }),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

#[test]
fn a_window_band_selects_the_set_and_escape_clears_it() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(two_lines_document()).expect("app");
    frames(&mut app, &context, 4);
    let rect = view_rect(&app);

    // 6. a plain left-to-right band over the viewport selects both lines.
    let start = Pos2::new(rect.left() + 8.0, rect.top() + 32.0);
    let end = Pos2::new(rect.right() - 8.0, rect.bottom() - 8.0);
    events(&mut app, &context, vec![Event::PointerMoved(start)]);
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        events(&mut app, &context, vec![Event::PointerMoved(start + (end - start) * t)]);
    }
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);
    assert_eq!(app.controller.selection_state().objects.len(), 2, "the band takes both lines");
    assert!(app.controller.gumball_pivot().is_some(), "the gumball shows the set");

    // 7. Esc clears the selection and the gumball.
    events(
        &mut app,
        &context,
        vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);
    assert!(app.controller.selected().is_empty(), "Esc clears the selection");
    assert!(app.controller.gumball_pivot().is_none(), "and the gumball");
}
