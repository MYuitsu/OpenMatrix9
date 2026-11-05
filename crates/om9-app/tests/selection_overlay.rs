// SPDX-License-Identifier: LGPL-2.1-or-later
//! Selection overlays: a selected set draws a white frame around itself, and any selection shows the
//! gumball. Both are painter shapes, so the tests count them in the viewport's clip rectangle.
use eframe::egui::{self, Color32, Event, Modifiers, Pos2, Rect, Shape, Vec2};
use om9_app::session::{Controller, InvocationOrigin};
use om9_app::ui::MatrixApp;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform},
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

fn frames(
    app: &mut MatrixApp,
    context: &egui::Context,
    count: usize,
) -> Vec<egui::epaint::ClippedShape> {
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
                (rect.stroke.color == Color32::WHITE
                    || rect.stroke.color == Color32::from_white_alpha(255))
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
                rect.stroke.color == Color32::WHITE
                    || rect.stroke.color == Color32::from_white_alpha(255)
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
    let mut controller =
        Controller::new(Document::new("toggle"), Limits::default()).expect("controller");
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
    let projected = camera
        .project(point, viewport)
        .expect("projection")
        .expect("in front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn pointer_drag(app: &mut MatrixApp, context: &egui::Context, start: Pos2, end: Pos2) {
    events(app, context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    events(app, context, vec![Event::PointerMoved(start)]);
    events(
        app,
        context,
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
            app,
            context,
            vec![Event::PointerMoved(start + (end - start) * t)],
        );
    }
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(app, context, 2);
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
    let id = *app
        .controller
        .document()
        .entities
        .keys()
        .next()
        .expect("line");
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
    events(
        &mut app,
        &context,
        vec![Event::PointerMoved(anchor + Vec2::new(40.0, 0.0))],
    );
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
fn pending_large_hover_does_not_publish_after_preview_moves() {
    let mut document = Document::new("stale large hover");
    let point = Entity::new("target", Geometry::Point([0., 0., 0.]), None);
    let target = point.id;
    document.entities.insert(target, point);
    for index in 0..129 {
        let point = Entity::new("far", Geometry::Point([1000. + index as f64, 1000., 0.]), None);
        document.entities.insert(point.id, point);
    }
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 3);
    let baseline = app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache);
    let pointer = screen_of(&app, [0., 0., 0.]);
    events(&mut app, &context, vec![Event::PointerMoved(pointer)]);
    app.controller.select_objects(&[target]).unwrap();
    app.controller.preview_gumball(Transform::translation([1000., 0., 0.])).unwrap();
    // The one-ID selection now has a persistent owner independent of hover.
    // Keep an exact completion oracle, including this live tree/Vec storage.
    let selection_bytes = om9_core::budget::Usage::btree::<om9_core::identity::ObjectId, ()>(1).unwrap().bytes
        + std::mem::size_of::<om9_core::identity::ObjectId>() as u64;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        frames(&mut app, &context, 1);
        assert_eq!(app.hovered_object(), None, "stale background hit must not publish");
        if app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache) == baseline + selection_bytes { break; }
        assert!(std::time::Instant::now() < deadline, "background hover did not finish");
        std::thread::yield_now();
    }
    app.controller.clear_gumball_preview();
}

#[test]
#[ignore = "opt-in release responsiveness measurement"]
fn large_scene_hover_tracks_latest_pointer_during_sustained_motion() {
    use std::time::{Duration, Instant};
    let mut document = Document::new("sustained hover motion");
    let brep_target = std::env::var_os("OM9_HOVER_BREP_TARGET").is_some();
    let target_geometry = if brep_target {
        #[cfg(feature = "occt-brep")]
        { Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.005, &Limits::default()).unwrap()) }
        #[cfg(not(feature = "occt-brep"))]
        panic!("OM9_HOVER_BREP_TARGET requires the occt-brep feature");
    } else { Geometry::Point([0., 0., 0.]) };
    let entity = Entity::new("target", target_geometry, None);
    let target = entity.id;
    document.entities.insert(target, entity);
    let brep_scene = brep_target || std::env::var_os("OM9_HOVER_BREP").is_some();
    let objects = if brep_scene {
        #[cfg(feature = "occt-brep")]
        {
            use om9_core::model::Definition;
            let brep = om9_kernel::brep::sphere_brep(3., 0.005, &Limits::default()).unwrap();
            let definition = Definition::new("shared sphere", vec![Entity::new("sphere", Geometry::Brep(brep), None)]);
            let definition_id = definition.id;
            document.definitions.insert(definition_id, definition);
            for index in 0..200 {
                let mut instance = Entity::new("far sphere", Geometry::Instance(definition_id), None);
                instance.placement = Transform::translation([1000. + index as f64 * 10., 1000., 0.]);
                document.entities.insert(instance.id, instance);
            }
            201
        }
        #[cfg(not(feature = "occt-brep"))]
        panic!("OM9_HOVER_BREP requires the occt-brep feature");
    } else {
        for index in 0..2000 {
            let point = Entity::new("far", Geometry::Point([1000. + index as f64, 1000., 0.]), None);
            document.entities.insert(point.id, point);
        }
        2001
    };
    let interval_ms = std::env::var("OM9_HOVER_INTERVAL_MS").ok().and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
    assert!(interval_ms <= 100, "fixture interval must be at most 100 ms");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 3);
    let screen = screen_of(&app, [0., 0., 0.]);
    let inside = if brep_target { screen } else { screen + Vec2::new(4., 0.) };
    let outside = if brep_target { screen_of(&app, [15., 0., 0.]) } else { screen + Vec2::new(6.5, 0.) };
    assert!(view_rect(&app).contains(inside) && view_rect(&app).contains(outside));
    let rect = view_rect(&app);
    let viewport = om9_scene::camera::Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&om9_scene::workspace::ViewId::Perspective].camera;
    let pick = |p: Pos2| app.controller.scene().unwrap().pick([p.x as f64, p.y as f64], camera, viewport, 6., om9_scene::scene::PickIntent::Select).unwrap().map(|hit| hit.root);
    assert_eq!(pick(inside), Some(target), "fixture inside point must pick the target");
    assert_eq!(pick(outside), None, "fixture outside point must miss all objects");
    let mut times = Vec::new();
    let mut hits = 0usize;
    let mut settled_hits = 0usize;
    for index in 0..200 {
        let is_inside = index % 2 == 0;
        let started = Instant::now();
        events(&mut app, &context, vec![Event::PointerMoved(if is_inside { inside } else { outside })]);
        times.push(started.elapsed().as_secs_f64() * 1000.);
        if is_inside && app.hovered_object() == Some(target) { hits += 1; }
        if !is_inside { assert_eq!(app.hovered_object(), None, "out-of-tolerance pointer must not show stale hit"); }
        let next = started + Duration::from_millis(interval_ms);
        while Instant::now() < next {
            std::thread::sleep(Duration::from_millis(1));
            frames(&mut app, &context, 1);
            if !is_inside { assert_eq!(app.hovered_object(), None, "an old worker must not publish after the pointer leaves"); }
        }
        if is_inside && app.hovered_object() == Some(target) { settled_hits += 1; }
        std::thread::yield_now();
    }
    let started = Instant::now();
    events(&mut app, &context, vec![Event::PointerMoved(inside)]);
    let deadline = started + Duration::from_secs(10);
    while app.hovered_object() != Some(target) {
        frames(&mut app, &context, 1);
        assert!(Instant::now() < deadline, "latest pointer hover did not complete");
        std::thread::yield_now();
    }
    let final_ms = started.elapsed().as_secs_f64() * 1000.;
    times.sort_by(|a, b| a.total_cmp(b));
    eprintln!("hover_motion scene={} objects={objects} interval_ms={interval_ms} samples=200 p50_ms={:.3} p95_ms={:.3} p99_ms={:.3} immediate_inside={hits}/100 settled_inside={settled_hits}/100 final_ms={final_ms:.3}", if brep_target { "brep-target" } else if brep_scene { "shared-brep" } else { "points" }, times[99], times[189], times[197]);
}

#[test]
fn changing_active_view_clears_hover_from_the_previous_view() {
    use om9_scene::workspace::ViewId;
    let context = egui::Context::default();
    let mut document = Document::new("hover across viewports");
    let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = point.id;
    document.entities.insert(id, point);
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 3);
    let pointer = screen_of(&app, [0., 0., 0.]);
    events(&mut app, &context, vec![Event::PointerMoved(pointer)]);
    assert_eq!(app.hovered_object(), Some(id));
    assert!(!app.view_rects[&ViewId::LookingDown].contains(pointer));
    app.controller.workspace.active = ViewId::LookingDown;
    frames(&mut app, &context, 1);
    assert_eq!(app.hovered_object(), None, "old view hover must not survive activation of another view");
    app.controller.workspace.active = ViewId::Perspective;
    frames(&mut app, &context, 1);
    assert_eq!(app.hovered_object(), Some(id), "hover should resume when the original view is active again");
}

#[test]
fn hover_updates_when_a_small_pointer_move_leaves_pick_tolerance() {
    let context = egui::Context::default();
    let mut document = Document::new("small hover movement");
    let target = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = target.id;
    document.entities.insert(id, target);
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 3);
    let screen = screen_of(&app, [0., 0., 0.]);
    let inside = screen + Vec2::new(4., 0.);
    let outside = screen + Vec2::new(6.5, 0.);
    events(&mut app, &context, vec![Event::PointerMoved(inside)]);
    assert_eq!(app.hovered_object(), Some(id), "fixture starts within the six-point tolerance");
    events(&mut app, &context, vec![Event::PointerMoved(outside)]);
    assert_eq!(app.hovered_object(), None, "even a small motion must refresh a hit outside the tolerance");
}

#[test]
fn stationary_hover_is_recomputed_when_gumball_preview_moves_the_object() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_two_points()).unwrap();
    let target = app.controller.document().entities.values().find(|entity| {
        matches!(entity.geometry, Geometry::Point([-6., 0., 0.]))
    }).unwrap().id;
    frames(&mut app, &context, 3);
    let pointer = screen_of(&app, [-6., 0., 0.]);
    events(&mut app, &context, vec![Event::PointerMoved(pointer)]);
    frames(&mut app, &context, 2);
    assert_eq!(app.hovered_object(), Some(target));
    app.controller.select_objects(&[target]).unwrap();
    app.controller.preview_gumball(Transform::translation([1000., 0., 0.])).unwrap();
    frames(&mut app, &context, 2);
    assert_eq!(app.hovered_object(), None, "old hover must not survive preview movement");
    app.controller.clear_gumball_preview();
}

#[test]
fn large_scene_hover_completes_and_releases_its_snapshot_reservation() {
    let mut document = Document::new("large hover");
    let point = Entity::new("target", Geometry::Point([0., 0., 0.]), None);
    let target = point.id;
    document.entities.insert(target, point);
    for index in 0..129 {
        let point = Entity::new("far", Geometry::Point([1000. + index as f64, 1000., 0.]), None);
        document.entities.insert(point.id, point);
    }
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 3);
    let baseline = app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache);
    let hovered = screen_of(&app, [0., 0., 0.]);
    events(&mut app, &context, vec![Event::PointerMoved(hovered)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        frames(&mut app, &context, 1);
        if app.hovered_object() == Some(target) { break; }
        assert!(std::time::Instant::now() < deadline, "background hover did not finish");
        std::thread::yield_now();
    }
    assert_eq!(app.controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache), baseline);
}

#[test]
fn large_scene_hover_is_optional_under_memory_pressure() {
    use om9_core::budget::memory::MemoryClass;
    let mut document = Document::new("hover pressure");
    let target = Entity::new("target", Geometry::Point([0., 0., 0.]), None);
    document.entities.insert(target.id, target);
    for index in 0..129 {
        let point = Entity::new("far", Geometry::Point([1000. + index as f64, 1000., 0.]), None);
        document.entities.insert(point.id, point);
    }
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    frames(&mut app, &context, 3);
    let baseline = app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
    let held = app.controller.scene_cache_usage().unwrap();
    app.controller.coordinator().constrain_managed_ram(128 * 1024 * 1024, held).unwrap();
    let mut reservation = app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
    reservation.shrink_to(reservation.bytes() - 1024).unwrap();
    let before = app.controller.coordinator().snapshot();
    let pointer = screen_of(&app, [0., 0., 0.]);
    events(&mut app, &context, vec![Event::PointerMoved(pointer)]);
    frames(&mut app, &context, 3);
    assert_eq!(app.hovered_object(), None);
    assert_eq!(app.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    drop(reservation);
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
    assert_eq!(
        white_rects(&shapes, clip),
        0,
        "the frame goes away with the pointer"
    );
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
    assert!(
        white_frames(&shapes, clip) >= 1,
        "two selected objects get the set frame"
    );
}
