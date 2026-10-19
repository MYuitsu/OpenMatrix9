// SPDX-License-Identifier: LGPL-2.1-or-later
//! Viewport acceptance oracles for `OM9-L2-01-07`, straight from the Rhino reference text in the M02
//! folders:
//!
//! * "Hold the Shift key while clicking … added to the selection" and "Ctrl … remove" — also "To add
//!   objects, hold the Shift key while making a window or crossing selection".
//! * "To cancel the selection … press the Esc key".
//! * "Press and hold both the Ctrl and Shift keys … and click an object part" (sub-object selection).
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn cube() -> Mesh {
    Mesh {
        vertices: vec![
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ],
        faces: vec![
            vec![0, 2, 1],
            vec![0, 3, 2],
            vec![4, 5, 6],
            vec![4, 6, 7],
            vec![0, 1, 5],
            vec![0, 5, 4],
            vec![2, 3, 7],
            vec![2, 7, 6],
            vec![1, 2, 6],
            vec![1, 6, 5],
            vec![0, 4, 7],
            vec![0, 7, 3],
        ],
        normals: Vec::new(),
        colors: Vec::new(),
    }
}

fn cube_document() -> (Document, om9_core::identity::ObjectId) {
    let mut document = Document::new("acceptance");
    let entity = Entity::new("box", Geometry::Mesh(cube()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    (document, id)
}

fn points_document() -> Document {
    let mut document = Document::new("points");
    for index in 0..2 {
        let entity = Entity::new(
            format!("p{index}"),
            Geometry::Point([index as f64 * 6.0 - 3.0, 0.0, 0.0]),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
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

/// Runs frames and returns the painter shapes, for tests that count what was drawn.
fn shapes(app: &mut MatrixApp, context: &egui::Context, count: usize) -> Vec<egui::epaint::ClippedShape> {
    let mut last = Vec::new();
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
        last = output.shapes;
    }
    last
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

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let rect = view_rect(app);
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
    let projected = camera.project(point, viewport).expect("project").expect("front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn pointer(app: &mut MatrixApp, context: &egui::Context, at: Pos2, modifiers: Modifiers, click: bool) {
    // A standalone modifiers frame first, then the pointer frames, mirroring the working band tests.
    events(app, context, vec![Event::ModifiersChanged(modifiers)]);
    events(app, context, vec![Event::PointerMoved(at)]);
    if click {
        events(
            app,
            context,
            vec![Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            }],
        );
        events(
            app,
            context,
            vec![
                Event::ModifiersChanged(modifiers),
                Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers,
                },
            ],
        );
    }
    frames(app, context, 2);
}

/// Press, drag through four intermediate positions, release, with `modifiers` held throughout.
fn band(app: &mut MatrixApp, context: &egui::Context, start: Pos2, end: Pos2, modifiers: Modifiers) {
    events(app, context, vec![Event::ModifiersChanged(modifiers)]);
    events(app, context, vec![Event::PointerMoved(start)]);
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers,
        }],
    );
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        events(app, context, vec![Event::PointerMoved(start + (end - start) * t)]);
    }
    events(
        app,
        context,
        vec![
            Event::ModifiersChanged(modifiers),
            Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            },
        ],
    );
    frames(app, context, 2);
}

#[test]
fn ctrl_shift_click_picks_the_face_under_the_pointer() {
    let context = egui::Context::default();
    let (document, root) = cube_document();
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    // Inside one triangle of the top face: the exact centre lies on the shared diagonal.
    let at = screen_of(&app, [0.3, 0.1, 1.0]);
    pointer(&mut app, &context, at, Modifiers::CTRL | Modifiers::SHIFT, true);
    let reference = app.sub_object().expect("a subobject is picked");
    let (picked_root,subset)=(reference.root,reference.subset);
    assert_eq!(picked_root, root);
    assert_eq!(
        subset,
        om9_scene::scene::Subelement::MeshFace(2),
        "the top face, as a document face key"
    );
    assert!(app.controller.selected().contains(&root), "the owning object is selected");
}

#[test]
fn shift_adds_with_a_band_and_ctrl_removes() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(points_document()).expect("app");
    frames(&mut app, &context, 4);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&[ids[0]]).expect("select one");
    frames(&mut app, &context, 2);
    // Start away from the gumball's handles: a press on a handle is a transform, not a band, which is the
    // correct Matrix9 behaviour and would make this test measure the wrong thing.
    let view = view_rect(&app);
    let start = Pos2::new(view.left() + 8.0, view.top() + 32.0);
    let end = Pos2::new(view.right() - 8.0, view.bottom() - 8.0);
    band(&mut app, &context, start, end, Modifiers::SHIFT);
    assert_eq!(
        app.controller.selection_state().objects.len(),
        2,
        "Shift added the second point"
    );
    let second = screen_of(&app, [3.0, 0.0, 0.0]);
    // Right to left is a crossing band, and it starts in the far corner so the press cannot land on a
    // gumball handle. Only the second point is inside.
    band(
        &mut app,
        &context,
        Pos2::new(view.right() - 8.0, view.top() + 32.0),
        Pos2::new(second.x - 25.0, view.bottom() - 8.0),
        Modifiers::CTRL,
    );
    assert_eq!(
        app.controller.selection_state().objects.len(),
        1,
        "Ctrl removed the second point, selection {:?}",
        app.controller.selected().iter().collect::<Vec<_>>()
    );
}

#[test]
fn escape_without_a_drag_clears_the_selection() {
    let context = egui::Context::default();
    let (document, root) = cube_document();
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    app.controller.select_objects(&[root]).expect("select");
    frames(&mut app, &context, 2);
    assert!(!app.controller.selected().is_empty());
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
    assert!(
        app.controller.selected().is_empty(),
        "Esc cancels the selection when no gesture is running"
    );
}

#[test]
fn pick_tolerance_stays_in_logical_points_at_high_dpi() {
    let context = egui::Context::default();
    context.set_pixels_per_point(1.0);
    let mut app = MatrixApp::new(points_document()).expect("app");
    frames(&mut app, &context, 4);
    let near = screen_of(&app, [-3.0, 0.0, 0.0]) + Vec2::new(10.0, 0.0);
    pointer(&mut app, &context, near, Modifiers::NONE, true);
    assert!(
        app.controller.selected().is_empty(),
        "ten pixels away at 1x is outside the six point tolerance"
    );
    context.set_pixels_per_point(2.0);
    frames(&mut app, &context, 2);
    pointer(&mut app, &context, near, Modifiers::NONE, true);
    assert_eq!(
        app.controller.selected().len(),
        0,
        "ten logical points stays outside six points at 2x; DPI must not be applied twice"
    );
    let close = screen_of(&app, [-3.0,0.0,0.0]) + Vec2::new(4.0,0.0);
    pointer(&mut app, &context, close, Modifiers::NONE, true);
    assert_eq!(app.controller.selected().len(), 1, "four logical points hits at 2x");
}

#[test]
fn twenty_band_passes_leave_the_same_selection() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(points_document()).expect("app");
    frames(&mut app, &context, 4);
    let view = view_rect(&app);
    let start = Pos2::new(view.right() - 8.0, view.top() + 32.0);
    let second = screen_of(&app, [3.0, 0.0, 0.0]);
    let end = Pos2::new(second.x - 25.0, view.bottom() - 8.0);
    for run in 1..=20 {
        band(&mut app, &context, start, end, Modifiers::NONE);
        assert_eq!(
            app.controller.selection_state().objects.len(),
            1,
            "run {run}: the crossing band keeps taking exactly the right point"
        );
    }
}

#[test]
fn a_picked_subobject_is_drawn_in_the_viewport() {
    let context = egui::Context::default();
    let (document, root) = cube_document();
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    let clip = view_rect(&app);
    let before = shapes(&mut app, &context, 2)
        .iter()
        .filter(|clipped| clipped.clip_rect == clip)
        .filter(|clipped| matches!(&clipped.shape, egui::Shape::LineSegment { .. }))
        .count();
    app.controller
        .select_subobject(root, om9_scene::scene::Subelement::MeshFace(2))
        .expect("pick the top face");
    let after = shapes(&mut app, &context, 2)
        .iter()
        .filter(|clipped| clipped.clip_rect == clip)
        .filter(|clipped| matches!(&clipped.shape, egui::Shape::LineSegment { .. }))
        .count();
    let outlined = shapes(&mut app, &context, 2)
        .iter()
        .filter(|clipped| clipped.clip_rect == clip)
        .filter(|clipped| matches!(&clipped.shape, egui::Shape::Path(_)))
        .count();
    assert!(
        after > before || outlined > 0,
        "the picked face is outlined: segments {before} -> {after}, paths {outlined}"
    );
    app.controller.clear_subselection();
    frames(&mut app, &context, 2);
}

#[test]
fn shift_click_adds_and_keeps_an_already_selected_object() {
    // OM9-L4-03-03-001-01 oracle: "Shift click thêm, Ctrl click toggle selection, plain click replace".
    let context = egui::Context::default();
    let mut app = MatrixApp::new(points_document()).expect("app");
    frames(&mut app, &context, 4);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    let first = screen_of(&app, [-3.0, 0.0, 0.0]);
    let second = screen_of(&app, [3.0, 0.0, 0.0]);

    pointer(&mut app, &context, first, Modifiers::NONE, true);
    assert_eq!(app.controller.selected().len(), 1, "plain click replaces");
    pointer(&mut app, &context, second, Modifiers::SHIFT, true);
    assert_eq!(app.controller.selected().len(), 2, "Shift adds the second object");
    // Shift again on the same object: still selected, not toggled off.
    pointer(&mut app, &context, second, Modifiers::SHIFT, true);
    assert_eq!(
        app.controller.selected().len(),
        2,
        "Shift is add, so an already selected object stays selected"
    );
    // Ctrl toggles the second one off again.
    pointer(&mut app, &context, second, Modifiers::CTRL, true);
    assert_eq!(app.controller.selected().len(), 1, "Ctrl removes");
    assert!(app.controller.selected().contains(&ids[0]) || app.controller.selected().contains(&ids[1]));
    // Plain click on the second object replaces the whole selection with it.
    pointer(&mut app, &context, second, Modifiers::NONE, true);
    assert_eq!(app.controller.selected().len(), 1, "plain click replaces the set");
}

#[test]
fn a_drag_that_starts_on_the_dock_draws_no_selection_marquee() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(points_document()).expect("app");
    frames(&mut app, &context, 4);
    // The left dock sits over x < 240 in the default layout, outside every viewport canvas.
    let start = Pos2::new(60.0, 420.0);
    let end = Pos2::new(200.0, 620.0);
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
    assert!(!app.band_active(), "a dock drag must not start a viewport marquee");
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
    assert!(app.controller.selected().is_empty(), "and it selects nothing");
    assert!(!app.band_active());
}
