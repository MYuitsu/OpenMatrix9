// SPDX-License-Identifier: LGPL-2.1-or-later
//! End-to-end viewport interaction: a real click selects the object, a real left drag on the gumball
//! moves it, and Undo puts it back. The pointer events go through `MatrixApp::show`, so the path under test
//! is the one a mouse takes, not a direct controller call.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};
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
fn drag_strength_and_defaults_are_typed_and_bounded() {
    let mut app = MatrixApp::new(Document::new("gumball strength")).unwrap();
    assert_eq!(app.controller.gumball_drag_strength(), 1.0);
    app.controller.submit("Gumball DragStrength=50").unwrap();
    assert!((app.controller.gumball_drag_strength() - 0.5).abs() < 1e-9);
    app.controller.submit("DragStrength=10").unwrap();
    assert!((app.controller.gumball_drag_strength() - 0.1).abs() < 1e-9);
    assert!(app.controller.submit("DragStrength=0").is_err());
    assert!(app.controller.submit("DragStrength=500").is_err());
    app.controller.submit("Gumball Snappy").unwrap();
    app.controller.submit("Gumball RestoreDefaults").unwrap();
    assert_eq!(app.controller.gumball_drag_strength(), 1.0);
    assert!(!app.controller.gumball_snappy());
}

#[test]
fn fifty_percent_strength_moves_half_the_pointer_distance() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("gumball strength ui")).expect("app");
    line(&mut app);
    frames(&mut app, &context, 4);
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("Gumball DragStrength=50").unwrap();
    frames(&mut app, &context, 2);
    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    let anchor = screen_point(&app, ViewId::Perspective, pivot);
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
    let after = app.controller.gumball_pivot().expect("selected").origin;
    let drawn = screen_point(&app, ViewId::Perspective, after);
    assert!(
        (drawn.x - (anchor.x + 24.0)).abs() < 0.5,
        "50% strength must move half the pointer distance: {drawn:?} anchor {anchor:?}"
    );
}

#[test]
fn snappy_mode_is_typed_and_smooth_is_the_default() {
    let mut app = MatrixApp::new(Document::new("gumball modes")).unwrap();
    assert!(!app.controller.gumball_snappy(), "Smooth is the default");
    app.controller.submit("Gumball Snappy").unwrap();
    assert!(app.controller.gumball_snappy());
    app.controller.submit("Gumball Smooth").unwrap();
    assert!(!app.controller.gumball_snappy());
}

#[test]
fn snappy_drag_follows_the_object_snap_marker() {
    let context = egui::Context::default();
    let mut document = Document::new("gumball snappy");
    let snap = Entity::new("snap point", Geometry::Point([10.0, 10.0, 0.0]), None);
    let snap_id = snap.id;
    document.entities.insert(snap_id, snap);
    let mut app = MatrixApp::new(document).expect("app");
    line(&mut app);
    frames(&mut app, &context, 4);
    let line_id = *app
        .controller
        .document()
        .entities
        .keys()
        .find(|id| **id != snap_id)
        .expect("line");
    app.controller.select_objects(&[line_id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    frames(&mut app, &context, 2);
    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    assert!((pivot[0] - 10.0).abs() < 1e-6, "{pivot:?}");

    let anchor = screen_point(&app, ViewId::Perspective, pivot);
    let snap_screen = screen_point(&app, ViewId::Perspective, [10.0, 10.0, 0.0]);
    let raw = snap_screen + Vec2::new(3.0, 0.0);
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
        let t = step as f32 / 4.0;
        let moved = anchor + (raw - anchor) * t;
        events(&mut app, &context, vec![Event::PointerMoved(moved)]);
    }
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: raw,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);

    let after = app.controller.gumball_pivot().expect("selected").origin;
    assert!(
        (0..3).all(|i| (after[i] - [10.0, 10.0, 0.0][i]).abs() < 1e-6),
        "Snappy must land the pivot at the 3D snap point: {after:?}"
    );
    let drawn = screen_point(&app, ViewId::Perspective, after);
    assert!(
        (drawn.x - snap_screen.x).abs() < 0.01 && (drawn.y - snap_screen.y).abs() < 0.01,
        "the pivot must land on the snap marker, not the raw pointer: {drawn:?} snap {snap_screen:?} raw {raw:?}"
    );
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

#[test]
fn tspline_object_f6_and_cmd_activate_only_the_requested_handles_and_transform_the_cage() {
    use om9_app::{f6::catalog::Mode as F6Mode, session::InvocationOrigin};
    use om9_scene::{gumball_view::Mode, scene::Subelement};
    let ctx = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance(
        "F6 T-Spline object manipulator",
    ))
    .unwrap();
    app.controller.submit("tsPlane").unwrap();
    app.controller.submit("Center").unwrap();
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([8., 6., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    let original = app.controller.document().clone();
    app.controller.workspace.active = ViewId::LookingDown;
    app.f6_mode = F6Mode::TSplines;
    frames(&mut app, &ctx, 4);
    // The Object entry on a component returns to its owner and activates Translate.
    app.controller
        .select_subobject(id, Subelement::MeshFace(1))
        .unwrap();
    let object = app
        .f6_context()
        .actions
        .into_iter()
        .find(|r| r.label == "Object")
        .unwrap();
    app.controller
        .dispatch(&object.id, InvocationOrigin::F6)
        .unwrap();
    assert!(app.controller.subselection().is_empty());
    assert!(app.controller.selected().contains(&id));
    for (label, mode) in [
        ("Translate", Mode::Move),
        ("Rotate", Mode::Rotate),
        ("Scale", Mode::Scale),
    ] {
        for typed in [false, true] {
            app.controller.select_objects(&[id]).unwrap();
            if typed {
                app.controller
                    .submit(&format!(
                        "-_tsManip _Enabled=_On _Manipulator=_{label} _SelectionMode=_Object _Enter"
                    ))
                    .unwrap();
            } else {
                let row = app
                    .f6_context()
                    .actions
                    .into_iter()
                    .find(|r| r.label == label)
                    .unwrap();
                assert!(row.availability.enabled());
                app.controller
                    .dispatch(&row.id, InvocationOrigin::F6)
                    .unwrap();
            }
            assert_eq!(app.gumball_mode(), mode);
            assert_eq!(app.controller.document(), &original);
            frames(&mut app, &ctx, 2);
            let view = ViewId::LookingDown;
            let size = handle_size(&app, view);
            assert_eq!(
                app.gumball_scale_boxes(view).is_empty(),
                mode != Mode::Scale
            );
            let r = size * 0.6;
            let a = std::f64::consts::FRAC_1_SQRT_2 * r;
            let (from, to, expected) = match mode {
                Mode::Move => (
                    [size * 0.8, 0., 0.],
                    [size * 1.2, 0., 0.],
                    [8. + 0.4 * size, 0., 0.],
                ),
                Mode::Rotate => (
                    [a, a, 0.],
                    [0., r, 0.],
                    [
                        8. * std::f64::consts::FRAC_1_SQRT_2,
                        8. * std::f64::consts::FRAC_1_SQRT_2,
                        0.,
                    ],
                ),
                Mode::Scale => ([size, 0., 0.], [size * 1.3, 0., 0.], [8. * 1.3, 0., 0.]),
            };
            let start = screen_point(&app, view, from);
            let end = screen_point(&app, view, to);
            drag(&mut app, &ctx, Modifiers::NONE, start, end);
            let entity = &app.controller.document().entities[&id];
            let actual = entity.placement.point([8., 0., 0.]).unwrap();
            assert!(
                actual
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| (a - b).abs() < 0.05),
                "{mode:?} typed {typed}: {actual:?}, expected {expected:?}"
            );
            assert_eq!(entity.geometry, original.entities[&id].geometry);
            assert_eq!(app.controller.coordinator().undo_len(), 2);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &original);
        }
    }

    for uniform in [false, true] {
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .submit("-_tsManip _Enabled=_On _Manipulator=_Scale _SelectionMode=_Object _Enter")
            .unwrap();
        frames(&mut app, &ctx, 2);
        let view = ViewId::LookingDown;
        let size = handle_size(&app, view);
        let (start, end, factor, z) = if uniform {
            let p = screen_point(&app, view, [0.; 3]);
            (p, p + Vec2::new(45., 0.), 2f64.sqrt(), 3. * 2f64.sqrt())
        } else {
            (
                screen_point(&app, view, [0.32 * size, 0.32 * size, 0.]),
                screen_point(&app, view, [0.48 * size, 0.48 * size, 0.]),
                1.5,
                3.,
            )
        };
        drag(&mut app, &ctx, Modifiers::NONE, start, end);
        let actual = app.controller.document().entities[&id]
            .placement
            .point([8., 6., 3.])
            .unwrap();
        assert!(
            actual
                .iter()
                .zip([8. * factor, 6. * factor, z])
                .all(|(v, w)| (v - w).abs() < 0.05),
            "uniform={uniform} {actual:?}"
        );
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &original);
    }
}

#[test]
fn cage_component_gestures_edit_only_unique_source_vertices_in_world_space() {
    use om9_app::{f6::catalog::Mode as F6Mode, session::InvocationOrigin};
    use om9_core::{
        geometry::{Geometry, Transform},
        gumball::{self, Axis, Drag, Handle, Plane},
    };
    use om9_scene::scene::Subelement::{MeshEdge, MeshFace, MeshVertex};
    use std::collections::BTreeSet;
    let mut app = MatrixApp::new(Document::with_template_tolerance(
        "T-Spline partial mesh transforms",
    ))
    .unwrap();
    for cmd in ["tsPlane", "XFaces=3", "YFaces=2"] {
        app.controller.submit(cmd).unwrap();
    }
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([12., 8., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    let mut doc = app.controller.document().clone();
    let entity = doc.entities.get_mut(&id).unwrap();
    entity.placement = Transform::translation([3., -4., 2.])
        .compose(
            gumball::rotation_about([0.; 3], [0., 0., 1.], std::f64::consts::FRAC_PI_2).unwrap(),
        )
        .unwrap();
    let Geometry::Mesh(mesh) = &mut entity.geometry else {
        panic!("mesh")
    };
    mesh.colors = vec![[1, 2, 3, 255]; mesh.vertices.len()];
    mesh.corner_uvs = vec![[0.25, 0.75]; mesh.faces.len() * 4];
    let placement = entity.placement;
    let source = mesh.clone();
    let mut second = entity.clone();
    second.id = om9_core::identity::ObjectId::new();
    let other = second.id;
    second.placement = Transform::translation([-20., 3., 1.]);
    doc.entities.insert(other, second);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    app.f6_mode = F6Mode::TSplines;
    for (kind, picks, indices) in [
        (
            "Face",
            vec![MeshFace(0), MeshFace(1)],
            vec![0, 1, 2, 3, 4, 5],
        ),
        (
            "Edge",
            vec![MeshEdge { a: 0, b: 1 }, MeshEdge { a: 1, b: 2 }],
            vec![0, 1, 2],
        ),
        ("Vertex", vec![MeshVertex(0), MeshVertex(4)], vec![0, 4]),
    ] {
        let chosen = BTreeSet::from_iter(indices.into_iter());
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for &i in &chosen {
            let p = placement.point(source.vertices[i]).unwrap();
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        let center: [f64; 3] = std::array::from_fn(|i| (min[i] + max[i]) * 0.5);
        for (op, handle, input) in [
            (
                "Translate",
                Handle::MoveAxis(Axis::X),
                Drag::WorldDelta([3., 0., 0.]),
            ),
            ("Rotate", Handle::Rotate(Axis::Z), Drag::Angle(0.6)),
            ("Scale", Handle::ScalePlane(Plane::XY), Drag::Factor(1.4)),
        ] {
            for typed in [false, true] {
                app.controller.select_objects(&[id]).unwrap();
                app.controller.clear_subselection();
                for pick in &picks {
                    app.controller.select_subobject(id, pick.clone()).unwrap();
                }
                if typed {
                    app.controller.submit(&format!("-_tsManip _Enabled=_On _Manipulator=_{op} _SelectionMode=_{kind} _Enter")).unwrap();
                } else {
                    let row = app
                        .f6_context()
                        .actions
                        .into_iter()
                        .find(|a| a.label == op)
                        .unwrap();
                    assert!(row.availability.enabled());
                    app.controller
                        .dispatch(&row.id, InvocationOrigin::F6)
                        .unwrap();
                }
                let frame = app.controller.gumball_pivot().unwrap();
                for i in 0..3 {
                    assert!((frame.origin[i] - center[i]).abs() < 1e-10);
                }
                let mut session =
                    gumball::Session::begin(frame, app.controller.gumball_targets(), false)
                        .unwrap();
                session.drag(handle, input).unwrap();
                let delta = session.accumulated();
                app.controller.preview_gumball(delta).unwrap();
                assert!(!app.controller.cage_preview_edges().is_empty());
                assert_eq!(app.controller.document(), &doc);
                app.controller.clear_gumball_preview();
                assert!(app.controller.cage_preview_edges().is_empty());
                app.controller.commit_gumball(&session, op).unwrap();
                let e = &app.controller.document().entities[&id];
                assert_eq!(e.placement, placement);
                let Geometry::Mesh(m) = &e.geometry else {
                    panic!("mesh")
                };
                assert_eq!(
                    (&m.faces, &m.colors, &m.corner_uvs),
                    (&source.faces, &source.colors, &source.corner_uvs)
                );
                for (i, v) in m.vertices.iter().enumerate() {
                    let p = placement.point(source.vertices[i]).unwrap();
                    let expected = if chosen.contains(&i) {
                        delta.point(p).unwrap()
                    } else {
                        p
                    };
                    let actual = placement.point(*v).unwrap();
                    assert!(
                        actual
                            .iter()
                            .zip(expected)
                            .all(|(a, b)| (a - b).abs() < 1e-9),
                        "{op} {kind} {i}"
                    );
                }
                assert_eq!(
                    app.controller.document().entities[&other],
                    doc.entities[&other]
                );
                assert_eq!(app.controller.subselection().len(), picks.len());
                assert_eq!(app.controller.coordinator().undo_len(), 1);
                app.controller.undo().unwrap();
                assert_eq!(app.controller.document(), &doc);
            }
        }
    }
    // One shared gesture on two owners with different local coordinate systems.
    app.controller.select_objects(&[id, other]).unwrap();
    for owner in [id, other] {
        app.controller
            .select_subobject(owner, MeshVertex(0))
            .unwrap();
    }
    app.controller
        .submit("-_tsManip _Enabled=_On _Manipulator=_Translate _SelectionMode=_Vertex _Enter")
        .unwrap();
    let mut gesture = gumball::Session::begin(
        app.controller.gumball_pivot().unwrap(),
        app.controller.gumball_targets(),
        false,
    )
    .unwrap();
    gesture
        .drag(Handle::MoveFree, Drag::WorldDelta([1., 2., 3.]))
        .unwrap();
    app.controller
        .commit_gumball(&gesture, "Translate")
        .unwrap();
    for owner in [id, other] {
        let before = &doc.entities[&owner];
        let after = &app.controller.document().entities[&owner];
        let Geometry::Mesh(m) = &after.geometry else {
            panic!("mesh")
        };
        let p = before.placement.point(source.vertices[0]).unwrap();
        let q = after.placement.point(m.vertices[0]).unwrap();
        assert!(
            q.iter()
                .zip([p[0] + 1., p[1] + 2., p[2] + 3.])
                .all(|(a, b)| (a - b).abs() < 1e-9)
        );
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}
