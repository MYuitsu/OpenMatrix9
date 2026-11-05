// SPDX-License-Identifier: LGPL-2.1-or-later
//! Live gumball preview: dragging in one viewport must move the object in every viewport, and the document
//! must stay untouched until the release.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::session::Controller;
use om9_app::ui::MatrixApp;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn line_document() -> (Document, om9_core::identity::ObjectId) {
    let mut document = Document::new("preview");
    let entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [-10.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 20.0],
        }),
        None,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    (document, id)
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
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events,
            ..Default::default()
        },
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
    let projected = camera
        .project(point, viewport)
        .expect("project")
        .expect("front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

/// Where the scene says the object is drawn right now (preview included).
fn displayed(app: &MatrixApp, id: om9_core::identity::ObjectId) -> Transform {
    app.controller
        .scene()
        .expect("scene")
        .objects
        .iter()
        .find(|object| object.root == id)
        .expect("object")
        .display_transform()
        .expect("display transform")
}

#[test]
fn perspective_gumball_can_drag_after_orbit() {
    let context = egui::Context::default();
    let (document, id) = line_document();
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    frames(&mut app, &context, 4);
    let before_camera = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    let start = view_rect(&app).center() + egui::vec2(100., 70.);
    events(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Secondary, pressed: true, modifiers: Modifiers::NONE }]);
    let end = start + egui::vec2(35., 15.);
    events(&mut app, &context, vec![Event::PointerMoved(end)]);
    events(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Secondary, pressed: false, modifiers: Modifiers::NONE }]);
    frames(&mut app, &context, 5);
    assert_ne!(app.controller.workspace.views[&ViewId::Perspective].camera, before_camera, "orbit must actually move the camera");
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let unit = f64::from(screen_of(&app, [pivot[0] + 1., pivot[1], pivot[2]]).distance(screen_of(&app, pivot)));
    let size = 90. / unit.max(1e-9);
    let start = screen_of(&app, [pivot[0] + size * 0.55, pivot[1], pivot[2]]);
    let end = screen_of(&app, [pivot[0] + size * 1.4, pivot[1], pivot[2]]);
    events(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    events(&mut app, &context, vec![Event::PointerMoved(end)]);
    assert!(displayed(&app, id).0[0][3].abs() > 1e-6, "Gumball preview did not move");
    events(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.document().entities[&id].placement.0[0][3].abs() > 1e-6, "Gumball move did not commit");
}

#[test]
fn unchanged_preview_does_not_advance_revision() {
    let mut controller = Controller::new(line_document().0, Limits::default()).unwrap();
    let id = *controller.document().entities.keys().next().unwrap();
    controller.select_objects(&[id]).unwrap();
    let initial = controller.preview_revision();
    controller.preview_gumball(Transform::identity()).unwrap();
    assert_eq!(controller.preview_revision(), initial, "identity is already displayed");
    let delta = Transform::translation([4., 0., 0.]);
    controller.preview_gumball(delta).unwrap();
    let changed = controller.preview_revision();
    assert!(changed > initial);
    controller.preview_gumball(delta).unwrap();
    assert_eq!(controller.preview_revision(), changed, "repeated preview must not invalidate work");
    controller.clear_gumball_preview();
    let cleared = controller.preview_revision();
    assert!(cleared > changed);
    controller.clear_gumball_preview();
    assert_eq!(controller.preview_revision(), cleared, "repeated clear must not invalidate work");
}

#[test]
fn the_preview_moves_the_selected_objects_without_touching_the_document() {
    let mut controller = Controller::new(line_document().0, Limits::default()).expect("controller");
    let id = *controller.document().entities.keys().next().expect("line");
    controller.select_objects(&[id]).expect("select");
    let document_revision = controller.document().revision;
    let before = controller.preview_revision();
    controller
        .preview_gumball(Transform::translation([4.0, 0.0, 0.0]))
        .expect("preview");
    assert!(
        controller.preview_revision() > before,
        "the revision tells viewports to repaint"
    );
    let displayed = controller
        .scene()
        .expect("scene")
        .objects
        .iter()
        .find(|object| object.root == id)
        .expect("object")
        .display_transform()
        .expect("display transform");
    assert!(
        (displayed.0[0][3] - 4.0).abs() < 1e-9,
        "the drawn transform moved: {displayed:?}"
    );
    assert_eq!(
        controller.document().revision,
        document_revision,
        "the document is untouched"
    );
    assert!(
        (controller.document().entities[&id].placement.0[0][3]).abs() < 1e-9,
        "the stored placement is untouched"
    );
    controller.clear_gumball_preview();
    let displayed = controller
        .scene()
        .expect("scene")
        .objects
        .iter()
        .find(|object| object.root == id)
        .expect("object")
        .display_transform()
        .expect("display transform");
    assert!(
        displayed.0[0][3].abs() < 1e-9,
        "clearing restores the drawn transform"
    );
}

#[test]
fn dragging_a_handle_updates_the_preview_then_commits_on_release() {
    let context = egui::Context::default();
    let (document, id) = line_document();
    let mut app = MatrixApp::new(document).expect("app");
    frames(&mut app, &context, 4);
    let on_line = screen_of(&app, [10.0, 0.0, 0.0]);
    events(&mut app, &context, vec![Event::PointerMoved(on_line)]);
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: on_line,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    events(
        &mut app,
        &context,
        vec![Event::PointerButton {
            pos: on_line,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(&mut app, &context, 2);
    assert!(app.controller.selected().contains(&id));

    // Grab the X arrow halfway out and drag it away from the pivot.
    let pivot = app.controller.gumball_pivot().expect("pivot").origin;
    // The gumball is sized in pixels (90 px per handle), so convert that to world units for this camera and
    // grab the X arrow at half its length.
    let unit = f64::from(
        screen_of(&app, [pivot[0] + 1.0, pivot[1], pivot[2]]).distance(screen_of(&app, pivot)),
    );
    let size = 90.0 / unit.max(1e-9);
    let start = screen_of(&app, [pivot[0] + size * 0.55, pivot[1], pivot[2]]);
    let end = screen_of(&app, [pivot[0] + size * 1.4, pivot[1], pivot[2]]);
    let revision_before = app.controller.preview_revision();
    let placement_before = app.controller.document().entities[&id].placement;

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
        events(
            &mut app,
            &context,
            vec![Event::PointerMoved(start + (end - start) * t)],
        );
        frames(&mut app, &context, 1);
    }
    assert!(
        app.controller.preview_revision() > revision_before,
        "a drag bumps the preview revision, which is what makes the other viewports redraw"
    );
    let previewed = displayed(&app, id);
    assert!(
        (previewed.0[0][3]).abs() > 1e-6,
        "the drawn transform moved during the drag: {previewed:?}"
    );
    assert_eq!(
        app.controller.document().entities[&id].placement,
        placement_before,
        "the document waits for the release"
    );
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
    assert_ne!(
        app.controller.document().entities[&id].placement,
        placement_before,
        "the release commits the move"
    );
    let committed = app.controller.document().entities[&id].placement.0[0][3];
    let drawn = displayed(&app, id).0[0][3];
    assert!(
        (committed - drawn).abs() < 1e-6,
        "the preview and the committed transform agree"
    );
}

#[test]
fn the_drag_leaves_the_placement_untouched_so_the_ghost_can_stay_at_the_old_spot() {
    let mut controller = Controller::new(line_document().0, Limits::default()).expect("controller");
    let id = *controller.document().entities.keys().next().expect("line");
    controller.select_objects(&[id]).expect("select");
    let placement = controller.document().entities[&id].placement;
    controller
        .preview_gumball(Transform::translation([6.0, 2.0, 0.0]))
        .expect("preview");
    let object = controller
        .scene()
        .expect("scene")
        .objects
        .iter()
        .find(|object| object.root == id)
        .expect("object");
    // The ghost is drawn from `transform`; it must still describe the starting position while the preview
    // moves the live object.
    assert_eq!(
        object.transform, placement,
        "the placement stays where the drag started"
    );
    assert_ne!(
        object.display_transform().expect("display").0,
        placement.0,
        "the live display transform follows the preview"
    );
    controller.clear_gumball_preview();
    let object = controller
        .scene()
        .expect("scene")
        .objects
        .iter()
        .find(|object| object.root == id)
        .expect("object");
    assert_eq!(
        object.display_transform().expect("display"),
        placement,
        "clearing the preview puts the object back"
    );
}

#[test]
fn shift_pick_makes_the_rectangle_square() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("shift square")).expect("app");
    app.controller.submit("Rectangle").expect("rectangle");
    frames(&mut app, &context, 3);

    let click = |pos: Pos2, modifiers: Modifiers| {
        vec![
            Event::ModifiersChanged(modifiers),
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            },
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            },
        ]
    };
    let first = screen_of(&app, [0., 0., 0.]);
    events(&mut app, &context, click(first, Modifiers::NONE));
    frames(&mut app, &context, 1);
    let second = screen_of(&app, [10., 20., 0.]);
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    events(&mut app, &context, click(second, shift));
    frames(&mut app, &context, 2);

    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("committed rectangle");
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &app.controller.document().entities[&id].geometry
    else {
        panic!("rectangle polycurve");
    };
    let mut max = [f64::MIN; 3];
    for segment in segments {
        let om9_core::geometry::Curve::Line { start, end, .. } = segment else {
            panic!("rectangle line");
        };
        for point in [start, end] {
            for axis in 0..3 {
                max[axis] = max[axis].max(point[axis]);
            }
        }
    }
    assert!(
        (max[0] - 20.0).abs() < 1e-3 && (max[1] - 20.0).abs() < 1e-3,
        "shift pick must make a square: {max:?}"
    );
}
