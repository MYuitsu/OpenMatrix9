// SPDX-License-Identifier: LGPL-2.1-or-later
//! End-to-end viewport interaction: a real click selects the object, a real left drag on the gumball
//! moves it, and Undo puts it back. The pointer events go through `MatrixApp::show`, so the path under test
//! is the one a mouse takes, not a direct controller call.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn frames(app: &mut MatrixApp, context: &egui::Context, frames: usize) {
    for _ in 0..frames {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
}

fn events(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    events_with(app, context, Modifiers::NONE, events);
}

fn events_with(
    app: &mut MatrixApp,
    context: &egui::Context,
    modifiers: Modifiers,
    events: Vec<Event>,
) {
    // egui 0.36 carries held modifiers as an event, not as a field of `RawInput`.
    let mut all = Vec::with_capacity(events.len() + 1);
    all.push(Event::ModifiersChanged(modifiers));
    all.extend(events);
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events: all,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

/// Creates a horizontal line from (0,0,0) to (20,0,0) through the command surface.
fn line(app: &mut MatrixApp) {
    app.controller.submit("Line").expect("Line command");
    app.controller.submit("0,0,0").expect("start");
    app.controller.submit("20,0,0").expect("end");
}

fn viewport_of(app: &MatrixApp, view: ViewId) -> Viewport {
    let rect = app.view_rects[&view];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

/// Screen point of a world point in a viewport, in egui points.
fn screen_point(app: &MatrixApp, view: ViewId, point: [f64; 3]) -> Pos2 {
    let viewport = viewport_of(app, view);
    let camera = &app.controller.workspace.views[&view].camera;
    let projected = camera
        .project(point, viewport)
        .expect("projection")
        .expect("in front of the camera");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn left(app: &MatrixApp, view: ViewId, point: [f64; 3]) -> (Pos2, Pos2) {
    // The canvas starts under the viewport title bar; a point at the very center is safely inside.
    let position = screen_point(app, view, point);
    let rect = app.view_rects[&view];
    let clamped = Pos2::new(
        position.x.clamp(rect.left() + 8.0, rect.right() - 8.0),
        position.y.clamp(rect.top() + 26.0, rect.bottom() - 8.0),
    );
    (position, clamped)
}

#[test]
fn a_click_selects_and_a_gumball_drag_moves_the_line() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("gumball ui")).expect("app");
    line(&mut app);
    frames(&mut app, &context, 4);
    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("one line");
    assert!(
        app.controller.selection_state().objects.is_empty(),
        "nothing is selected yet"
    );

    // Click the middle of the line in the Perspective viewport.
    let (target, pressed_at) = left(&app, ViewId::Perspective, [10.0, 0.0, 0.0]);
    let _ = target;
    events(&mut app, &context, vec![Event::PointerMoved(pressed_at)]);
    events(
        &mut app,
        &context,
        vec![
            Event::PointerButton {
                pos: pressed_at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: pressed_at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frames(&mut app, &context, 2);
    assert!(
        app.controller.selection_state().objects.contains(&id),
        "a click in the viewport must select the line"
    );
    let pivot = app
        .controller
        .gumball_pivot()
        .expect("the gumball has a pivot once something is selected");
    assert!(
        (pivot.origin[0] - 10.0).abs() < 1e-6,
        "the pivot sits at the line centre: {:?}",
        pivot.origin
    );

    // Drag a gumball handle: press on the pivot, move, release.
    let undo_before = app.controller.coordinator().undo_len();
    let anchor = screen_point(&app, ViewId::Perspective, pivot.origin);
    events(&mut app, &context, vec![Event::PointerMoved(anchor)]);
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: anchor,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    for step in 1..=4 {
        let moved = anchor + Vec2::new(12.0 * step as f32, 0.0);
        events(&mut app, &context, vec![Event::PointerMoved(moved)]);
    }
    let released = anchor + Vec2::new(48.0, 0.0);
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: released,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);

    let after = app
        .controller
        .gumball_pivot()
        .expect("still selected")
        .origin;
    assert!(
        (after[0] - pivot.origin[0]).abs() > 0.5,
        "dragging the gumball must move the line: {:?} -> {:?}",
        pivot.origin,
        after
    );
    let drawn = screen_point(&app, ViewId::Perspective, after);
    assert!(
        (drawn.x - released.x).abs() < 0.001 && (drawn.y - released.y).abs() < 0.001,
        "free move follows the pointer in the screen plane: {drawn:?} vs {released:?}"
    );
    assert!(
        app.controller.coordinator().undo_len() > undo_before,
        "the drag must be one journal entry"
    );

    app.controller.undo().expect("undo");
    frames(&mut app, &context, 1);
    let restored = app
        .controller
        .gumball_pivot()
        .expect("selected after undo")
        .origin;
    assert!(
        (restored[0] - 10.0).abs() < 1e-6,
        "undo restores the line centre: {restored:?}"
    );
}

/// World length of a gumball handle: the UI uses ninety pixels, so the test reproduces that rule.
fn handle_size(app: &MatrixApp, view: ViewId) -> f64 {
    let viewport = viewport_of(app, view);
    let camera = &app.controller.workspace.views[&view].camera;
    let distance = {
        let offset = [
            camera.eye[0] - camera.target[0],
            camera.eye[1] - camera.target[1],
            camera.eye[2] - camera.target[2],
        ];
        (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt()
    };
    let per_pixel = match camera.projection {
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => {
            2.0 * (vertical_fov_rad * 0.5).tan() * distance / viewport.size[1].max(1.0)
        }
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => {
            vertical_span_mm / viewport.size[1].max(1.0)
        }
    };
    per_pixel * 90.0
}

fn placement_of(app: &MatrixApp, id: om9_core::identity::ObjectId) -> [[f64; 4]; 4] {
    app.controller.document().entities[&id].placement.0
}

/// Presses at `start`, moves to `end` in steps, releases. Every event carries the modifiers, so the UI
/// picks the same gumball mode a held Shift or Alt would.
fn drag(
    app: &mut MatrixApp,
    context: &egui::Context,
    modifiers: Modifiers,
    start: Pos2,
    end: Pos2,
) {
    // A real keyboard sends the modifier change on its own frame, before the pointer press; sending both in
    // one frame makes egui treat the press as a new interaction and the drag never starts.
    events_with(app, context, modifiers, Vec::new());
    events_with(app, context, modifiers, vec![Event::PointerMoved(start)]);
    events_with(
        app,
        context,
        modifiers,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers,
        }],
    );
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        let position = start + (end - start) * t;
        events_with(app, context, modifiers, vec![Event::PointerMoved(position)]);
    }
    events_with(
        app,
        context,
        modifiers,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        }],
    );
    frames(app, context, 2);
}

#[test]
fn shift_dragging_a_ring_rotates_the_selection() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("gumball rotate")).expect("app");
    line(&mut app);
    frames(&mut app, &context, 4);
    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("one line");
    let (_, pressed_at) = left(&app, ViewId::Perspective, [10.0, 0.0, 0.0]);
    events(&mut app, &context, vec![Event::PointerMoved(pressed_at)]);
    events(
        &mut app,
        &context,
        vec![
            Event::PointerButton {
                pos: pressed_at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: pressed_at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frames(&mut app, &context, 2);
    assert!(
        app.controller.selection_state().objects.contains(&id),
        "the line must be selected"
    );

    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    let size = handle_size(&app, ViewId::Perspective);
    // A point at 45 degrees on the Z arc: the arcs are quarter arcs at 60 percent of the arrow length, so
    // the point has to be on that arc rather than on an imaginary full ring.
    let radius = size * 0.6;
    // The arcs bow out on the negative side of their axes, so the point is at -45 degrees.
    let ring = [
        pivot[0] - radius * std::f64::consts::FRAC_1_SQRT_2,
        pivot[1] - radius * std::f64::consts::FRAC_1_SQRT_2,
        pivot[2],
    ];
    let start = screen_point(&app, ViewId::Perspective, ring);
    {
        use om9_scene::gumball_view::{GumballView, Mode};
        let viewport = viewport_of(&app, ViewId::Perspective);
        let camera = app.controller.workspace.views[&ViewId::Perspective]
            .camera
            .clone();
        let frame = om9_core::gumball::Frame {
            origin: pivot,
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        };
        let gumball = GumballView::with_size(frame, Mode::Rotate, size);
        let hit = gumball
            .hit(
                &camera,
                viewport,
                [f64::from(start.x), f64::from(start.y)],
                8.0,
            )
            .expect("hit test");
        eprintln!("rotate: ring point {ring:?} screen {start:?} hit {hit:?}");
    }
    let end = screen_point(
        &app,
        ViewId::Perspective,
        [pivot[0], pivot[1] - radius, pivot[2]],
    );
    {
        let rect = app.view_rects[&ViewId::Perspective];
        eprintln!(
            "rect {rect:?} start {start:?} end {end:?} inside canvas {} {}",
            start.x > rect.left()
                && start.x < rect.right()
                && start.y > rect.top() + 26.0
                && start.y < rect.bottom(),
            end.x > rect.left()
                && end.x < rect.right()
                && end.y > rect.top() + 26.0
                && end.y < rect.bottom()
        );
    }
    let before = placement_of(&app, id);
    app.set_gumball_mode(om9_scene::gumball_view::Mode::Rotate);
    drag(&mut app, &context, Modifiers::NONE, start, end);

    let after = placement_of(&app, id);
    assert_ne!(
        before, after,
        "a Shift drag on the ring must change the placement"
    );
    let (cosine, sine) = (after[0][0], after[1][0]);
    assert!(
        (cosine * cosine + sine * sine - 1.0).abs() < 1e-6,
        "the placement must hold a rotation, not a shear: {after:?}"
    );
    // Rotation about the pivot leaves the pivot point where it was.
    let moved_pivot = app
        .controller
        .gumball_pivot()
        .expect("pivot after rotation")
        .origin;
    assert!(
        (moved_pivot[0] - pivot[0]).abs() < 1e-6 && (moved_pivot[1] - pivot[1]).abs() < 1e-6,
        "rotation keeps the pivot: {:?} -> {moved_pivot:?}",
        pivot
    );
    app.controller.undo().expect("undo");
    assert_eq!(
        placement_of(&app, id),
        before,
        "undo restores the placement"
    );
}

#[test]
fn dragging_the_scale_square_scales_the_selection() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("gumball scale")).expect("app");
    line(&mut app);
    frames(&mut app, &context, 4);
    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("one line");
    let (_, pressed_at) = left(&app, ViewId::Perspective, [10.0, 0.0, 0.0]);
    events(&mut app, &context, vec![Event::PointerMoved(pressed_at)]);
    events(
        &mut app,
        &context,
        vec![
            Event::PointerButton {
                pos: pressed_at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: pressed_at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frames(&mut app, &context, 2);
    assert!(
        app.controller.selection_state().objects.contains(&id),
        "the line must be selected"
    );

    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    let size = handle_size(&app, ViewId::Perspective);
    // The Y axis, so the press is not near the viewport edge; the scale box sits on the negative side, as in
    // the manual figure, so the press is at -1 and the drag goes to -2 for a factor of two.
    let start = screen_point(
        &app,
        ViewId::Perspective,
        [pivot[0], pivot[1] - size, pivot[2]],
    );
    {
        use om9_scene::gumball_view::{GumballView, Mode};
        let viewport = viewport_of(&app, ViewId::Perspective);
        let camera = app.controller.workspace.views[&ViewId::Perspective]
            .camera
            .clone();
        let frame = om9_core::gumball::Frame {
            origin: pivot,
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        };
        let gumball = GumballView::with_size(frame, Mode::Scale, size);
        let hit = gumball
            .hit(
                &camera,
                viewport,
                [f64::from(start.x), f64::from(start.y)],
                8.0,
            )
            .expect("hit test");
        assert_eq!(
            hit,
            Some(om9_core::gumball::Handle::Scale(om9_core::gumball::Axis::Y))
        );
    }
    let end = screen_point(
        &app,
        ViewId::Perspective,
        [pivot[0], pivot[1] - size * 2.0, pivot[2]],
    );
    let before = placement_of(&app, id);
    app.set_gumball_mode(om9_scene::gumball_view::Mode::Scale);
    drag(&mut app, &context, Modifiers::NONE, start, end);

    let after = placement_of(&app, id);
    assert!(
        (after[1][1] - 2.0).abs() < 1e-3,
        "a factor of two on Y: {after:?}"
    );
    assert!(
        (after[0][0] - 1.0).abs() < 1e-6 && (after[2][2] - 1.0).abs() < 1e-6,
        "the other axes are untouched: {after:?}"
    );
    app.controller.undo().expect("undo");
    assert_eq!(
        placement_of(&app, id),
        before,
        "undo restores the placement"
    );
}
