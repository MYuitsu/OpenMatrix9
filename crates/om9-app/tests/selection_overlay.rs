// SPDX-License-Identifier: LGPL-2.1-or-later
//! Selection overlays: a selected set draws a white frame around itself, and any selection shows the
//! gumball. Both are painter shapes, so the tests count them in the viewport's clip rectangle.
use eframe::egui::{self, Color32, Event, Modifiers, Pos2, Rect, Shape, Vec2};
use om9_app::session::{Controller, InvocationOrigin};
use om9_app::ui::MatrixApp;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn document_with_two_points() -> Document {
    let mut document = Document::new("overlay");
    for x in [-6.0, 6.0] {
        let entity = Entity::new("point", Geometry::Point([x, 0.0, 0.0]), None);
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

fn frames(app: &mut MatrixApp, context: &egui::Context, count: usize) -> Vec<egui::epaint::ClippedShape> {
    let mut shapes = Vec::new();
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
        shapes = output.shapes;
    }
    shapes
}

/// White rectangles big enough to be the selection frame, not the gumball's seven-pixel pivot square.
fn white_frames(shapes: &[egui::epaint::ClippedShape], clip: Rect) -> usize {
    shapes
        .iter()
        .filter(|clipped| clipped.clip_rect == clip)
        .filter(|clipped| match &clipped.shape {
            Shape::Rect(rect) => {
                (rect.stroke.color == Color32::WHITE || rect.stroke.color == Color32::from_white_alpha(255))
                    && rect.rect.width() > 12.0
                    && rect.rect.height() > 12.0
            }
            _ => false,
        })
        .count()
}

/// Any white rectangle, used where the pivot square cannot be confused with the hover frame.
fn white_rects(shapes: &[egui::epaint::ClippedShape], clip: Rect) -> usize {
    shapes
        .iter()
        .filter(|clipped| clipped.clip_rect == clip)
        .filter(|clipped| match &clipped.shape {
            Shape::Rect(rect) => {
                rect.stroke.color == Color32::WHITE || rect.stroke.color == Color32::from_white_alpha(255)
            }
            _ => false,
        })
        .count()
}

/// Three points that span both screen directions, so the set frame has real width and height.
fn document_with_three_points() -> Document {
    let mut document = Document::new("overlay");
    for point in [[-6.0, -6.0, 0.0], [6.0, -6.0, 0.0], [0.0, 6.0, 0.0]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

fn line_segments(shapes: &[egui::epaint::ClippedShape], clip: Rect) -> usize {
    shapes
        .iter()
        .filter(|clipped| clipped.clip_rect == clip)
        .filter(|clipped| matches!(&clipped.shape, Shape::LineSegment { .. }))
        .count()
}

#[test]
fn a_multi_selection_draws_a_frame_around_itself() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_three_points()).expect("app");
    let shapes = frames(&mut app, &context, 4);
    let clip = app.view_rects[&om9_scene::workspace::ViewId::Perspective];
    assert_eq!(white_frames(&shapes, clip), 0, "nothing selected, no frame");

    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&ids).expect("select both");
    let shapes = frames(&mut app, &context, 2);
    assert!(
        white_frames(&shapes, clip) >= 1,
        "a multi selection must show its frame"
    );
}

#[test]
fn any_selection_shows_the_gumball() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_two_points()).expect("app");
    let before_shapes = frames(&mut app, &context, 4);
    let clip = app.view_rects[&om9_scene::workspace::ViewId::Perspective];
    let before = line_segments(&before_shapes, clip);

    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&ids).expect("select both");
    let after = line_segments(&frames(&mut app, &context, 2), clip);
    assert!(
        after >= before + 10,
        "the gumball adds its handles: {before} -> {after}"
    );
}

#[test]
fn clearing_the_selection_removes_the_frame_and_the_gumball() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_three_points()).expect("app");
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&ids).expect("select both");
    let selected = frames(&mut app, &context, 4);
    let clip = app.view_rects[&om9_scene::workspace::ViewId::Perspective];
    app.controller.clear_selection();
    let cleared = frames(&mut app, &context, 2);
    assert!(white_frames(&selected, clip) >= 1);
    assert_eq!(white_frames(&cleared, clip), 0);
    assert!(line_segments(&cleared, clip) < line_segments(&selected, clip));
    let _ = Controller::new(Document::new("unused"), Limits::default());
}

#[test]
fn ctrl_click_toggles_one_object_in_and_out_of_the_selection() {
    let mut controller = Controller::new(Document::new("toggle"), Limits::default()).expect("controller");
    // A Point command gives the scene selectable pick geometry; the two document points above bypass the
    // command path, so build the scene through the controller instead.
    controller.submit("Point").expect("Point");
    controller.submit("0,0,0").expect("first point");
    controller.submit("Point").expect("Point");
    controller.submit("10,0,0").expect("second point");
    let ids: Vec<_> = controller.document().entities.keys().copied().collect();
    assert_eq!(ids.len(), 2);
    controller.toggle_selection(ids[0]).expect("add");
    assert_eq!(controller.selected().len(), 1);
    controller.toggle_selection(ids[1]).expect("add");
    assert_eq!(controller.selected().len(), 2);
    controller.toggle_selection(ids[0]).expect("remove");
    assert_eq!(controller.selected().len(), 1);
    assert!(controller.selected().contains(&ids[1]));
    let _ = InvocationOrigin::Cmd;
}

fn document_with_line() -> Document {
    let mut document = Document::new("band modes");
    let start = [-10.0, 0.0, 0.0];
    let end = [10.0, 0.0, 0.0];
    let domain = [0.0, 20.0];
    let entity = Entity::new(
        "line",
        Geometry::Curve(Curve::Line { start, end, domain }),
        None,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    document
}

fn view_rect(app: &MatrixApp) -> Rect {
    app.view_rects[&om9_scene::workspace::ViewId::Perspective]
}

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let clip = view_rect(app);
    let viewport = om9_scene::camera::Viewport {
        origin: [clip.min.x as f64, clip.min.y as f64],
        size: [clip.width() as f64, clip.height() as f64],
    };
    let camera = &app.controller.workspace.views[&om9_scene::workspace::ViewId::Perspective].camera;
    let projected = camera.project(point, viewport).expect("projection").expect("in front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn pointer_drag(app: &mut MatrixApp, context: &egui::Context, start: Pos2, end: Pos2) {
    events(app, context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    events(app, context, vec![Event::PointerMoved(start)]);
    events(
        app,
        context,
        vec![Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }],
    );
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        events(app, context, vec![Event::PointerMoved(start + (end - start) * t)]);
    }
    events(
        app,
        context,
        vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
    );
    frames(app, context, 2);
}

fn events(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput { screen_rect: Some(screen_rect()), events, ..Default::default() },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

#[test]
fn the_band_is_a_window_left_to_right_and_crossing_right_to_left() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_line()).expect("app");
    frames(&mut app, &context, 4);
    let left = screen_of(&app, [-10.0, 0.0, 0.0]);
    let middle = screen_of(&app, [0.0, 0.0, 0.0]);
    // A band that covers the left half only: one endpoint inside, so a window keeps nothing.
    let inside = Pos2::new(left.x.min(middle.x) - 20.0, left.y.min(middle.y) - 20.0);
    let far = Pos2::new(left.x.max(middle.x) + 20.0, left.y.max(middle.y) + 20.0);
    pointer_drag(&mut app, &context, inside, far);
    assert!(
        app.controller.selection_state().objects.is_empty(),
        "window selection must not take a partially covered line"
    );
    // The same rectangle dragged right to left is a crossing selection and takes it.
    pointer_drag(&mut app, &context, far, inside);
    assert_eq!(
        app.controller.selection_state().objects.len(),
        1,
        "crossing selection takes the touched line"
    );
}

#[test]
fn escape_cancels_a_gumball_drag_without_committing() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_line()).expect("app");
    frames(&mut app, &context, 4);
    let id = *app.controller.document().entities.keys().next().expect("line");
    app.controller.select_objects(&[id]).expect("select");
    frames(&mut app, &context, 2);
    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    let anchor = screen_of(&app, pivot);
    let undo_before = app.controller.coordinator().undo_len();
    let placement_before = app.controller.document().entities[&id].placement;

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
    events(&mut app, &context, vec![Event::PointerMoved(anchor + Vec2::new(40.0, 0.0))]);
    // Esc during the gesture must drop it.
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
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: anchor + Vec2::new(40.0, 0.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);
    assert_eq!(
        app.controller.coordinator().undo_len(),
        undo_before,
        "a cancelled gesture writes nothing"
    );
    assert_eq!(
        app.controller.document().entities[&id].placement,
        placement_before,
        "the object keeps its placement"
    );
}

#[test]
fn hovering_an_object_shows_a_frame_that_follows_the_pointer() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_two_points()).expect("app");
    let shapes = frames(&mut app, &context, 4);
    let clip = view_rect(&app);
    assert_eq!(white_rects(&shapes, clip), 0, "nothing hovered, no frame");
    let hovered = screen_of(&app, [-6.0, 0.0, 0.0]);
    events(&mut app, &context, vec![Event::PointerMoved(hovered)]);
    let shapes = frames(&mut app, &context, 2);
    let _ = app.hovered_object();
    assert!(
        white_rects(&shapes, clip) >= 1,
        "the object under the pointer gets a frame"
    );
    let away = Pos2::new(clip.left() + 4.0, clip.bottom() - 4.0);
    events(&mut app, &context, vec![Event::PointerMoved(away)]);
    let shapes = frames(&mut app, &context, 2);
    assert_eq!(white_rects(&shapes, clip), 0, "the frame goes away with the pointer");
}

#[test]
fn a_single_selection_has_no_frame_but_two_do() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_three_points()).expect("app");
    frames(&mut app, &context, 4);
    let clip = view_rect(&app);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&[ids[0]]).expect("one");
    let shapes = frames(&mut app, &context, 2);
    assert_eq!(
        white_frames(&shapes, clip),
        0,
        "one selected object already reads as selected; no white box around it"
    );
    app.controller.select_objects(&ids).expect("two");
    let shapes = frames(&mut app, &context, 2);
    assert!(white_frames(&shapes, clip) >= 1, "two selected objects get the set frame");
}
