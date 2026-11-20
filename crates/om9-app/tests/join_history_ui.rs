// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless UI polling for a parent grip edit and its Join History dependents.
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::{session::{GripPick, InvocationOrigin}, ui::MatrixApp};
use om9_core::{budget::Limits, geometry::{Curve, Geometry}, gumball::{Axis, Drag, Handle, Session},
    model::{Document, Entity}, top11::Top11Action};

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) {
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
        events, ..Default::default()
    }, |ui| app.show(ui));
    output.textures_delta.clear();
}

fn fixture() -> (MatrixApp, om9_core::identity::ObjectId, [om9_core::identity::ObjectId; 2]) {
    fixture_with_extras(Vec::new(), Vec::new())
}

fn fixture_with_extras(extras: Vec<Entity>, definitions: Vec<om9_core::model::Definition>) -> (MatrixApp, om9_core::identity::ObjectId, [om9_core::identity::ObjectId; 2]) {
    let mut doc = Document::with_template_tolerance("join history ui");
    for entity in extras { doc.entities.insert(entity.id, entity); }
    for definition in definitions { doc.definitions.insert(definition.id, definition); }
    let mut ids = Vec::new();
    for (a, b) in [(0., 1.), (1., 2.), (2., 3.)] {
        let entity = Entity::new("source", Geometry::Curve(Curve::Line {
            start: [a, 0., 0.], end: [b, 0., 0.], domain: [0., 1.],
        }), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    app.controller.select_objects(&ids[..2]).unwrap();
    app.controller.create_join_history().unwrap();
    let first = app.controller.document().history.recipes.values().next().unwrap().outputs[0];
    app.controller.select_objects(&[first, ids[2]]).unwrap();
    app.controller.create_join_history().unwrap();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] != first).unwrap().outputs[0];
    app.controller.select_objects(&[ids[0]]).unwrap();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    (app, ids[0], [first, second])
}

fn start_edit(app: &mut MatrixApp, id: om9_core::identity::ObjectId) {
    let handle = app.controller.top11_grip_handles().unwrap().into_iter()
        .find(|handle| handle.object == id && handle.index == 0).unwrap();
    assert!(app.controller.top11_move_grip(handle, [-1., 0., 0.]).unwrap().message().contains("updating History"));
    assert!(app.controller.join_history_worker_pending(), "{} / {}", app.transcript.text(), app.controller.prompt());
}

fn select_all_parents(app: &mut MatrixApp, outputs: [om9_core::identity::ObjectId; 2]) {
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut parents: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    parents.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&parents).unwrap();
}

fn finish_ui_worker(app: &mut MatrixApp, ctx: &egui::Context) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(app, ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn center_object_rebuilds_dependents_atomically_and_can_be_cancelled() {
    let (mut app, id, outputs) = fixture();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    select_all_parents(&mut app, outputs);
    let before = app.controller.document().clone();
    app.controller.center_selected().unwrap();
    assert!(app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    finish_ui_worker(&mut app, &ctx);
    let after = app.controller.document().clone();
    for (output, end) in outputs.into_iter().zip([0.5, 1.5]) {
        let Geometry::Curve(curve) = &after.entities[&output].geometry else { panic!("curve") };
        let endpoints = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(endpoints.contains(&[-1.5, 0., 0.]) && endpoints.contains(&[end, 0., 0.]), "{endpoints:?}");
        let scene = app.controller.scene().unwrap().objects.iter().find(|object| object.root == output).unwrap();
        assert_eq!(scene.source, after.stamp(output).unwrap());
    }
    assert_eq!(app.controller.coordinator().undo_len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);

    app.controller.select_objects(&[id]).unwrap();
    app.controller.center_selected().unwrap();
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &after, "disconnected join must refuse");
    assert_eq!(app.controller.coordinator().undo_len(), 3);

    select_all_parents(&mut app, outputs);
    app.controller.center_selected().unwrap();
    assert!(app.controller.join_history_worker_pending());
    app.controller.cancel();
    assert_eq!(app.controller.document(), &after);
    assert_eq!(app.controller.coordinator().undo_len(), 3);
}

#[test]
fn parent_grip_rebuilds_two_dependents_via_ui_poll_and_one_undo() {
    let (mut app, id, outputs) = fixture();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    start_edit(&mut app, id);
    assert_eq!(app.controller.document(), &before);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let after = app.controller.document().clone();
    for (output, end) in outputs.into_iter().zip([2., 3.]) {
        let Geometry::Curve(curve) = &after.entities[&output].geometry else { panic!("curve") };
        let endpoints = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(endpoints.contains(&[-1., 0., 0.]) && endpoints.contains(&[end, 0., 0.]));
        let object = app.controller.scene().unwrap().objects.iter().find(|object| object.root == output).unwrap();
        assert_eq!(object.source, after.stamp(output).unwrap());
    }
    assert_eq!(app.controller.coordinator().undo_len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[test]
fn escape_cancels_pending_parent_grip_rebuild() {
    let (mut app, id, _) = fixture();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    start_edit(&mut app, id);
    frame(&mut app, &ctx, vec![Event::Key {
        key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
}

#[test]
fn gumball_grip_edit_uses_the_same_background_history_path() {
    let (mut app, id, outputs) = fixture();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    let handle = app.controller.top11_grip_handles().unwrap().into_iter()
        .find(|handle| handle.object == id && handle.index == 0).unwrap();
    app.controller.select_grips(&[handle], GripPick::Replace);
    let mut gesture = Session::begin(app.controller.gumball_pivot().unwrap(), app.controller.gumball_targets(), false).unwrap();
    gesture.drag(Handle::MoveFree, Drag::WorldDelta([-1., 0., 0.])).unwrap();
    app.controller.commit_gumball(&gesture, "Move").unwrap();
    assert!(app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
    assert!(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap().contains(&[-1., 0., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn reopened_history_propagates_a_parent_grip_edit() {
    let (app, id, outputs) = fixture();
    let path = std::env::temp_dir().join(format!("om9-join-ui-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, app.controller.document(), &Limits::default(),
        &om9_core::transaction::Cancellation::default()).unwrap();
    let reopened = om9_archive::Archive::open(&path, &Limits::default()).unwrap();
    std::fs::remove_file(path).unwrap();
    let mut app = MatrixApp::new(reopened).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    start_edit(&mut app, id);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
    assert!(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap().contains(&[-1., 0., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn downstream_join_failure_keeps_parent_and_both_children_unchanged() {
    let (mut app, _, outputs) = fixture();
    let parent = app.controller.document().history.recipes.values()
        .find(|recipe| recipe.outputs[0] == outputs[0]).unwrap().inputs[1].source.object;
    app.controller.select_objects(&[parent]).unwrap();
    let handle = app.controller.top11_grip_handles().unwrap().into_iter()
        .find(|handle| handle.object == parent && handle.index == 1).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.top11_move_grip(handle, [2.1, 0., 0.]).unwrap();
    assert!(app.controller.join_history_worker_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
}

#[test]
fn viewport_pointer_drag_schedules_and_publishes_the_history_edit() {
    use om9_scene::{camera::{Projection, Viewport}, workspace::ViewId};
    let (mut app, _, outputs) = fixture();
    let view = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&view).unwrap().camera.projection = Projection::Orthographic { vertical_span_mm: 6.0 };
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&view];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&view].camera;
    let project = |point| {
        let screen = camera.project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(screen[0] as f32, screen[1] as f32)
    };
    let start = project([0., 0., 0.]);
    let end = project([-1., 0., 0.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(start), Event::PointerButton {
        pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE,
    }]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(end)]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(end), Event::PointerButton {
        pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE,
    }]);
    assert!(app.controller.join_history_worker_pending(), "grip drag did not start history rebuild");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(ends.iter().any(|point| (point[0] + 1.0).abs() < 1e-4), "{ends:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 3);
}

#[test]
fn history_grip_edit_refuses_memory_pressure_without_document_change() {
    let (mut app, id, _) = fixture();
    let before = app.controller.document().clone();
    let handle = app.controller.top11_grip_handles().unwrap().into_iter()
        .find(|handle| handle.object == id && handle.index == 0).unwrap();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, None, Default::default()).unwrap();
    assert!(matches!(app.controller.top11_move_grip(handle, [-1., 0., 0.]), Err(om9_core::Error::Budget(_))));
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    drop(pressure);
}

#[test]
fn gumball_moves_parent_objects_and_rebuilds_two_history_levels() {
    let (mut app, _, outputs) = fixture();
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut parents: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    parents.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&parents).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    let mut gesture = Session::begin(app.controller.gumball_pivot().unwrap(), app.controller.gumball_targets(), false).unwrap();
    gesture.drag(Handle::MoveFree, Drag::WorldDelta([1., 0., 0.])).unwrap();
    app.controller.commit_gumball(&gesture, "Move").unwrap();
    assert!(app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let after = app.controller.document().clone();
    for (output, end) in outputs.into_iter().zip([3., 4.]) {
        let Geometry::Curve(curve) = &after.entities[&output].geometry else { panic!("curve") };
        let endpoints = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(endpoints.contains(&[1., 0., 0.]) && endpoints.contains(&[end, 0., 0.]));
        let object = app.controller.scene().unwrap().objects.iter().find(|object| object.root == output).unwrap();
        assert_eq!(object.source, after.stamp(output).unwrap());
    }
    assert_eq!(app.controller.coordinator().undo_len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[test]
fn gumball_copy_does_not_rebuild_the_original_history() {
    let (mut app, id, outputs) = fixture();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().clone();
    let mut gesture = Session::begin(app.controller.gumball_pivot().unwrap(), app.controller.gumball_targets(), true).unwrap();
    gesture.drag(Handle::MoveFree, Drag::WorldDelta([1., 0., 0.])).unwrap();
    app.controller.commit_gumball_copy(&gesture, "Copy").unwrap();
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
    for output in outputs { assert_eq!(app.controller.document().entities[&output], before.entities[&output]); }
    assert_eq!(app.controller.coordinator().undo_len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn disconnected_gumball_move_preserves_parent_and_children() {
    let (mut app, id, _) = fixture();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    let mut gesture = Session::begin(app.controller.gumball_pivot().unwrap(), app.controller.gumball_targets(), false).unwrap();
    gesture.drag(Handle::MoveFree, Drag::WorldDelta([0., 1., 0.])).unwrap();
    app.controller.commit_gumball(&gesture, "Move").unwrap();
    assert!(app.controller.join_history_worker_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while app.controller.join_history_worker_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "history worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
}

#[test]
fn gumball_rotation_and_scale_rebuild_join_history() {
    for (handle, drag) in [
        (Handle::Rotate(Axis::Z), Drag::Angle(std::f64::consts::FRAC_PI_2)),
        (Handle::Scale(Axis::X), Drag::Factor(2.0)),
    ] {
        let (mut app, _, outputs) = fixture();
        let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
        let mut parents: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
        let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
        parents.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        app.controller.select_objects(&parents).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        let mut gesture = Session::begin(app.controller.gumball_pivot().unwrap(), app.controller.gumball_targets(), false).unwrap();
        gesture.drag(handle, drag).unwrap();
        let transform = gesture.accumulated();
        let expected = [transform.point([0., 0., 0.]).unwrap(), transform.point([3., 0., 0.]).unwrap()];
        app.controller.commit_gumball(&gesture, "Transform").unwrap();
        assert!(app.controller.join_history_worker_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while app.controller.join_history_worker_pending() {
            frame(&mut app, &ctx, vec![]);
            assert!(std::time::Instant::now() < deadline, "history worker timeout");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
        let endpoints = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        for expected in expected {
            assert!(endpoints.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)), "{endpoints:?}");
        }
        assert_eq!(app.controller.coordinator().undo_len(), 3);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn numeric_gumball_commands_rebuild_history_and_undo() {
    for command in ["MoveX 1", "RotateZ 90", "ScaleX 2"] {
        for editor in [false, true] {
        let (mut app, parent, outputs) = fixture();
        select_all_parents(&mut app, outputs);
        let ctx = egui::Context::default();
        if editor { app.focus_command_editor(); }
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        if editor {
            frame(&mut app, &ctx, vec![Event::Text(command.into())]);
            frame(&mut app, &ctx, vec![Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
        } else { app.controller.submit(command).unwrap(); }
        assert!(app.controller.join_history_worker_pending(), "{command}, editor={editor}: {}", app.transcript.text());
        assert_eq!(app.controller.document(), &before);
        let event = app.controller.events().events().back().unwrap();
        assert_eq!(event.kind, om9_app::command_events::EventKind::Progress);
        assert_eq!(event.command, "Gumball");
        finish_ui_worker(&mut app, &ctx);
        let event = app.controller.events().events().back().unwrap();
        assert_eq!(event.outcome, Some(om9_app::command_events::EventOutcome::Committed));
        let after = app.controller.document().clone();
        let placement = after.entities[&parent].placement;
        let expected = [placement.point([0., 0., 0.]).unwrap(), placement.point([3., 0., 0.]).unwrap()];
        let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        for wanted in expected {
            assert!(ends.iter().any(|actual| (0..3).all(|i| (actual[i] - wanted[i]).abs() < 1e-7)), "{command}: {ends:?} / {}", app.transcript.text());
        }
        assert_eq!(app.controller.coordinator().undo_len(), 3);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
        }
    }
}

#[test]
fn numeric_gumball_disconnected_parent_does_not_publish() {
    let (mut app, parent, _) = fixture();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[parent]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("MoveY 1").unwrap();
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert!(app.transcript.text().contains("dependent history rebuild"), "{}", app.transcript.text());
    assert_eq!(app.controller.events().events().back().unwrap().outcome, Some(om9_app::command_events::EventOutcome::Refused));
}

#[test]
fn numeric_gumball_cancel_reports_canceled_without_change() {
    let (mut app, _, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let before = app.controller.document().clone();
    app.controller.submit("MoveX 1").unwrap();
    assert!(app.controller.join_history_worker_pending());
    app.controller.cancel_join_history_worker();
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert_eq!(app.controller.events().events().back().unwrap().outcome, Some(om9_app::command_events::EventOutcome::Cancelled));
}

#[test]
fn transform_menu_in_place_commands_rebuild_history() {
    for (name, feature, stretch) in [("Scale", "OM9-TRANSFORM-001", false), ("Scale1D", "OM9-TRANSFORM-004", false), ("Stretch", "OM9-TRANSFORM-032", true)] {
        for menu in [false, true] {
            let (mut app, _, outputs) = fixture();
            select_all_parents(&mut app, outputs);
            let ctx = egui::Context::default();
            frame(&mut app, &ctx, vec![]);
            let before = app.controller.document().clone();
            if menu { app.controller.dispatch(feature, InvocationOrigin::Menu).unwrap(); }
            else { app.controller.submit(name).unwrap(); }
            app.controller.submit("0,0,0").unwrap();
            if stretch { app.controller.submit("1,0,0").unwrap(); }
            app.controller.submit("2").unwrap();
            assert!(app.controller.join_history_worker_pending(), "{name}");
            assert_eq!(app.controller.document(), &before);
            let event = app.controller.events().events().back().unwrap();
            assert_eq!(event.command, feature);
            assert_eq!(event.kind, om9_app::command_events::EventKind::Progress);
            finish_ui_worker(&mut app, &ctx);
            let after = app.controller.document().clone();
            let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!(ends.contains(&[0., 0., 0.]) && ends.contains(&[6., 0., 0.]), "{name}: {ends:?} / {}", app.transcript.text());
            assert_eq!(app.controller.events().events().back().unwrap().outcome, Some(om9_app::command_events::EventOutcome::Committed));
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        }
    }
}

#[test]
fn transform_menu_failure_and_cancel_leave_history_unchanged() {
    for cancel in [false, true] {
        let (mut app, parent, _) = fixture();
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        app.controller.select_objects(&[parent]).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        for input in ["Scale1D", "0,0,0", "2"] { app.controller.submit(input).unwrap(); }
        assert!(app.controller.join_history_worker_pending());
        if cancel { app.controller.cancel_join_history_worker(); } else { finish_ui_worker(&mut app, &ctx); }
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        let outcome = if cancel { om9_app::command_events::EventOutcome::Cancelled } else { om9_app::command_events::EventOutcome::Refused };
        assert_eq!(app.controller.events().events().back().unwrap().outcome, Some(outcome));
    }
}

#[test]
fn shear_curve_geometry_rebuilds_history() {
    for menu in [false, true] {
        let (mut app, _, outputs) = fixture();
        select_all_parents(&mut app, outputs);
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        if menu { app.controller.dispatch("OM9-TRANSFORM-034", InvocationOrigin::Menu).unwrap(); }
        else { app.controller.submit("Shear").unwrap(); }
        for input in ["0,0,0", "1,0,0", "0,1,0", "0.5"] { app.controller.submit(input).unwrap(); }
        assert!(app.controller.join_history_worker_pending());
        assert_eq!(app.controller.document(), &before);
        finish_ui_worker(&mut app, &ctx);
        let after = app.controller.document().clone();
        let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(ends.iter().any(|p| (p[0] - 3.).abs() < 1e-7 && (p[1] - 1.5).abs() < 1e-7), "{ends:?} / {}", app.transcript.text());
        assert_eq!(app.controller.events().events().back().unwrap().outcome, Some(om9_app::command_events::EventOutcome::Committed));
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }
}

#[test]
fn cage_edit_rebuilds_join_history_atomically() {
    for mode in 0..3 {
        let extent = Entity::new("extent", Geometry::Point([0., 1., 1.]), None);
        let extent_id = extent.id;
        let (mut app, parent, outputs) = fixture_with_extras(vec![extent], Vec::new());
        let first = app.controller.document().history.recipes.values().find(|r| r.outputs[0] == outputs[0]).unwrap();
        let mut ids: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
        let second = app.controller.document().history.recipes.values().find(|r| r.outputs[0] == outputs[1]).unwrap();
        ids.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
        if mode == 1 { ids = vec![parent]; }
        ids.push(extent_id);
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        app.controller.select_objects(&ids).unwrap();
        app.controller.submit("Cage").unwrap();
        let cage = app.controller.document().entities.values().find(|entity| entity.extensions.contains_key("om9.cage") && matches!(entity.geometry, Geometry::Mesh(_))).unwrap().id;
        app.controller.select_objects(&[cage]).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        let undo = app.controller.coordinator().undo_len();
        for input in ["CageEdit", "2", "0,1,0"] { app.controller.submit(input).unwrap(); }
        assert!(app.controller.join_history_worker_pending());
        if mode == 2 { app.controller.cancel_join_history_worker(); }
        else { finish_ui_worker(&mut app, &ctx); }
        if mode == 0 {
            let after = app.controller.document().clone();
            let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!(ends.iter().any(|p| (p[0]-3.).abs() < 1e-6 && (p[1]-1.).abs() < 1e-6), "{ends:?} / {}", app.transcript.text());
            assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        } else {
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().undo_len(), undo);
        }
    }
}

#[test]
fn smart_flow_and_pulldown_rebuild_history_with_link_metadata() {
    use om9_core::surface_kernel;
    let base_shape = surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &Limits::default()).unwrap();
    let mut target_shape = base_shape.clone();
    for point in &mut target_shape.poles { point[2] = point[0]; }
    let base = Entity::new("base", Geometry::Surface(base_shape), None);
    let target = Entity::new("target", Geometry::Surface(target_shape), None);
    let surface_ids = [base.id, target.id];
    let (mut app, _, outputs) = fixture_with_extras(vec![base, target], Vec::new());
    let first = app.controller.document().history.recipes.values().find(|r| r.outputs[0] == outputs[0]).unwrap();
    let mut ids: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|r| r.outputs[0] == outputs[1]).unwrap();
    ids.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    let parents = ids.clone();
    ids.extend(surface_ids);
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["SmartFlow", "9,9,0"] { app.controller.submit(input).unwrap(); }
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    let flowed = app.controller.document().clone();
    let Geometry::Curve(curve) = &flowed.entities[&outputs[1]].geometry else { panic!("curve") };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(ends.iter().any(|p| (p[0]-3.).abs() < 1e-5 && (p[2]-3.).abs() < 1e-5), "{ends:?} / {}", app.transcript.text());
    for id in &parents { assert!(flowed.entities[id].extensions.contains_key("om9.flow")); }
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &flowed);
    app.controller.select_objects(&ids).unwrap();
    for input in ["SmartFlowPullDownCurves", "9,9,0"] { app.controller.submit(input).unwrap(); }
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    let after = app.controller.document().clone();
    let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(ends.iter().any(|p| (p[0]-3.).abs() < 1e-5 && p[2].abs() < 1e-5), "{ends:?} / {}", app.transcript.text());
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &flowed);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[test]
fn smart_flow_failure_and_cancel_do_not_publish_metadata() {
    use om9_core::surface_kernel;
    for cancel in [false, true] {
        let base_shape = surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &Limits::default()).unwrap();
        let mut target_shape = base_shape.clone();
        for point in &mut target_shape.poles { point[2] = point[0]; }
        let base = Entity::new("base", Geometry::Surface(base_shape), None);
        let target = Entity::new("target", Geometry::Surface(target_shape), None);
        let surface_ids = [base.id, target.id];
        let (mut app, parent, outputs) = fixture_with_extras(vec![base, target], Vec::new());
        let first = app.controller.document().history.recipes.values().find(|r| r.outputs[0] == outputs[0]).unwrap();
        let mut ids: Vec<_> = if cancel { first.inputs.iter().map(|input| input.source.object).collect() } else { vec![parent] };
        if cancel {
            let second = app.controller.document().history.recipes.values().find(|r| r.outputs[0] == outputs[1]).unwrap();
            ids.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
        }
        ids.extend(surface_ids);
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        app.controller.select_objects(&ids).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        for input in ["SmartFlow", "9,9,0"] { app.controller.submit(input).unwrap(); }
        assert!(app.controller.join_history_worker_pending());
        if cancel { app.controller.cancel_join_history_worker(); } else { finish_ui_worker(&mut app, &ctx); }
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        assert!(!app.controller.document().entities[&parent].extensions.contains_key("om9.flow"));
    }
}

#[test]
fn move_command_and_direct_invoke_rebuild_history_together() {
    for direct in [false, true] {
        let (mut app, _, outputs) = fixture();
        select_all_parents(&mut app, outputs);
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        if direct { app.controller.dispatch_effect(Top11Action::Move.feature_id(), InvocationOrigin::Menu).unwrap(); }
        else { app.controller.submit("Move").unwrap(); }
        app.controller.accept_world_point([0., 0., 0.]).unwrap();
        app.controller.accept_world_point([1., 0., 0.]).unwrap();
        assert!(app.controller.join_history_worker_pending());
        assert_eq!(app.controller.document(), &before);
        finish_ui_worker(&mut app, &ctx);
        let after = app.controller.document().clone();
        let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(ends.contains(&[1., 0., 0.]) && ends.contains(&[4., 0., 0.]));
        assert_eq!(app.controller.coordinator().undo_len(), 3);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }
}

#[test]
fn rotate_typed_angle_and_three_points_rebuild_history() {
    for typed in [false, true] {
        let (mut app, _, outputs) = fixture();
        select_all_parents(&mut app, outputs);
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        if typed { app.controller.submit("Rotate").unwrap(); }
        else { app.controller.dispatch_effect(Top11Action::Rotate.feature_id(), InvocationOrigin::Menu).unwrap(); }
        app.controller.accept_world_point([0., 0., 0.]).unwrap();
        if typed { app.controller.submit("90").unwrap(); }
        else {
            app.controller.accept_world_point([1., 0., 0.]).unwrap();
            app.controller.accept_world_point([0., 1., 0.]).unwrap();
        }
        assert!(app.controller.join_history_worker_pending());
        assert_eq!(app.controller.document(), &before);
        finish_ui_worker(&mut app, &ctx);
        let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        for expected in [[0., 0., 0.], [0., 3., 0.]] {
            assert!(ends.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)), "{ends:?}");
        }
        assert_eq!(app.controller.coordinator().undo_len(), 3);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn typed_move_in_command_editor_rebuilds_history() {
    let (mut app, _, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let ctx = egui::Context::default();
    app.focus_command_editor();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Move", "0,0,0", "1,0,0"] {
        frame(&mut app, &ctx, vec![Event::Text(input.into())]);
        frame(&mut app, &ctx, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
        }]);
    }
    assert!(app.controller.join_history_worker_pending(), "{} / {}", app.transcript.text(), app.controller.prompt());
    assert_eq!(app.controller.document(), &before);
    finish_ui_worker(&mut app, &ctx);
    let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(ends.contains(&[1., 0., 0.]) && ends.contains(&[4., 0., 0.]));
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn typed_rotate_in_command_editor_rebuilds_history() {
    let (mut app, _, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let ctx = egui::Context::default();
    app.focus_command_editor();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Rotate", "0,0,0", "90"] {
        frame(&mut app, &ctx, vec![Event::Text(input.into())]);
        frame(&mut app, &ctx, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
        }]);
    }
    assert!(app.controller.join_history_worker_pending(), "{} / {}", app.transcript.text(), app.controller.prompt());
    assert_eq!(app.controller.document(), &before);
    finish_ui_worker(&mut app, &ctx);
    let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(ends.iter().any(|point| point[0].abs() < 1e-8 && (point[1] - 3.).abs() < 1e-8));
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn move_command_refuses_a_disconnected_history_without_partial_edit() {
    let (mut app, id, _) = fixture();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Move").unwrap();
    app.controller.accept_world_point([0., 0., 0.]).unwrap();
    app.controller.accept_world_point([0., 1., 0.]).unwrap();
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert!(app.transcript.text().contains("dependent history rebuild"), "{}", app.transcript.text());
}

#[test]
fn mirror_command_and_axis_option_rebuild_history() {
    for mode in 0..3 {
        let (mut app, _, outputs) = fixture();
        select_all_parents(&mut app, outputs);
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        if mode == 1 { app.controller.submit("Mirror").unwrap(); }
        else { app.controller.dispatch_effect(Top11Action::Mirror.feature_id(), InvocationOrigin::Menu).unwrap(); }
        app.controller.submit("Copy=No").unwrap();
        if mode == 1 { app.controller.submit("YAxis").unwrap(); }
        else if mode == 2 {
            app.controller.submit("3Point").unwrap();
            app.controller.accept_world_point([0., 0., 0.]).unwrap();
            app.controller.accept_world_point([0., 1., 0.]).unwrap();
            app.controller.accept_world_point([0., 0., 1.]).unwrap();
        } else {
            app.controller.accept_world_point([0., 0., 0.]).unwrap();
            app.controller.accept_world_point([0., 1., 0.]).unwrap();
        }
        assert!(app.controller.join_history_worker_pending());
        assert_eq!(app.controller.document(), &before);
        finish_ui_worker(&mut app, &ctx);
        let after = app.controller.document().clone();
        let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
        let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
        assert!(ends.contains(&[0., 0., 0.]) && ends.contains(&[-3., 0., 0.]), "{ends:?}");
        assert_eq!(app.controller.coordinator().undo_len(), 3);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
    }
}

#[test]
fn typed_mirror_in_command_editor_rebuilds_history() {
    let (mut app, _, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let ctx = egui::Context::default();
    app.focus_command_editor();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Mirror", "Copy=No", "YAxis"] {
        frame(&mut app, &ctx, vec![Event::Text(input.into())]);
        frame(&mut app, &ctx, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
        }]);
    }
    assert!(app.controller.join_history_worker_pending(), "{} / {}", app.transcript.text(), app.controller.prompt());
    assert_eq!(app.controller.document(), &before);
    finish_ui_worker(&mut app, &ctx);
    let Geometry::Curve(curve) = &app.controller.document().entities[&outputs[1]].geometry else { panic!("curve") };
    assert!(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap().contains(&[-3., 0., 0.]));
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn mirror_copy_preserves_original_history_and_disconnected_mirror_refuses() {
    let (mut app, id, outputs) = fixture();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().clone();
    app.controller.submit("Mirror").unwrap();
    app.controller.submit("Copy=Yes").unwrap();
    app.controller.submit("YAxis").unwrap();
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document().entities.len(), before.entities.len() + 1);
    for output in outputs { assert_eq!(app.controller.document().entities[&output], before.entities[&output]); }
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    app.controller.select_objects(&[id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Mirror").unwrap();
    app.controller.submit("Copy=No").unwrap();
    app.controller.submit("YAxis").unwrap();
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}

#[test]
fn escape_cancels_pending_mirror_history_update() {
    let (mut app, _, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Mirror").unwrap();
    app.controller.submit("Copy=No").unwrap();
    app.controller.submit("YAxis").unwrap();
    assert!(app.controller.join_history_worker_pending());
    frame(&mut app, &ctx, vec![Event::Key {
        key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
}

#[test]
fn curve_mirror_refuses_worker_memory_pressure_before_editing() {
    let (mut app, _, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let before = app.controller.document().clone();
    app.controller.submit("Mirror").unwrap();
    app.controller.submit("Copy=No").unwrap();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, None, Default::default()).unwrap();
    assert!(matches!(app.controller.submit("YAxis"), Err(om9_core::Error::Budget(_))));
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    drop(pressure);
}

#[test]
fn curve_mirror_discards_stale_selection() {
    let (mut app, id, outputs) = fixture();
    select_all_parents(&mut app, outputs);
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    app.controller.select_objects(&[id]).unwrap();
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert!(app.transcript.text().contains("stale"), "{}", app.transcript.text());
}

#[test]
fn mixed_geometry_mirror_rebuilds_history_and_undoes_all_objects() {
    use om9_core::{geometry::{Mesh, Transform}, model::Definition, surface_kernel};
    let point = Entity::new("point", Geometry::Point([2., 3., 0.]), None);
    let point_id = point.id;
    let mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
        vertices: vec![[1., 0., 0.], [2., 0., 0.], [1., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let mesh_id = mesh.id;
    let surface = Entity::new("surface", Geometry::Surface(surface_kernel::plane(
        [2., 0., 0.], [1., 0., 0.], [0., 1., 0.], 1., 1., &Limits::default()).unwrap()), None);
    let surface_id = surface.id;
    let member = Entity::new("member", Geometry::Point([1., 0., 0.]), None);
    let definition = Definition::new("block", vec![member]);
    let mut instance = Entity::new("instance", Geometry::Instance(definition.id), None);
    instance.placement = Transform::translation([2., 0., 0.]);
    let instance_id = instance.id;
    let (mut app, _, outputs) = fixture_with_extras(vec![point, mesh, surface, instance], vec![definition]);
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut ids: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    ids.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    ids.extend([point_id, mesh_id, surface_id, instance_id]);
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    assert!(app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    finish_ui_worker(&mut app, &ctx);
    let after = app.controller.document().clone();
    assert_eq!(after.entities[&point_id].geometry, Geometry::Point([-2., 3., 0.]));
    let Geometry::Mesh(mesh) = &after.entities[&mesh_id].geometry else { panic!("mesh") };
    assert_eq!(mesh.vertices, vec![[-1., 0., 0.], [-2., 0., 0.], [-1., 1., 0.]]);
    assert_eq!(mesh.faces, vec![vec![0, 2, 1]]);
    let (Geometry::Surface(old), Geometry::Surface(new)) = (&before.entities[&surface_id].geometry, &after.entities[&surface_id].geometry) else { panic!("surface") };
    for (old, new) in old.poles.iter().zip(&new.poles) { assert_eq!(*new, [-old[0], old[1], old[2]]); }
    assert_eq!(after.entities[&instance_id].placement.point([1., 0., 0.]).unwrap(), [-3., 0., 0.]);
    let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
    assert!(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap().contains(&[-3., 0., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[test]
fn mixed_geometry_mirror_dependent_failure_keeps_all_objects() {
    let point = Entity::new("point", Geometry::Point([2., 3., 0.]), None);
    let point_id = point.id;
    let (mut app, parent, _) = fixture_with_extras(vec![point], Vec::new());
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[parent, point_id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert!(app.transcript.text().contains("dependent history rebuild"), "{}", app.transcript.text());
}

#[test]
fn trim_parent_rebuilds_history_or_rejects_disconnected_result() {
    for (from, to, succeeds) in [(0.5, 1.0, true), (0.0, 0.5, false)] {
        let (mut app, parent, outputs) = fixture();
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        app.controller.select_objects(&[parent]).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit(&format!("top11 Trim {from} {to}")).unwrap();
        assert!(app.controller.join_history_worker_pending());
        assert_eq!(app.controller.document(), &before);
        finish_ui_worker(&mut app, &ctx);
        if succeeds {
            let after = app.controller.document().clone();
            let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!(ends.contains(&[0.5, 0., 0.]) && ends.contains(&[3., 0., 0.]), "{ends:?} / {}", app.transcript.text());
            assert_eq!(app.controller.coordinator().undo_len(), 3);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        } else {
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.coordinator().undo_len(), 2);
            assert!(app.transcript.text().contains("dependent history rebuild"), "{}", app.transcript.text());
        }
    }
}

#[test]
fn interactive_trim_resumes_after_history_success_failure_and_cancel() {
    for mode in 0..3 {
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line {
            start: [0.5, -1., 0.], end: [0.5, 1., 0.], domain: [0., 1.],
        }), None);
        let cutter_id = cutter.id;
        let (mut app, parent, outputs) = fixture_with_extras(vec![cutter], Vec::new());
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        app.controller.select_objects(&[parent, cutter_id]).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit("Trim").unwrap();
        let before_pending = app.controller.top11_pending().unwrap().clone();
        let pick = if mode == 1 { [0.75, 0., 0.] } else { [0.25, 0., 0.] };
        app.controller.top11_trim_many_accept_pick(parent, pick).unwrap();
        assert!(app.controller.join_history_worker_pending());
        assert_eq!(app.controller.document(), &before);
        if mode == 2 { app.controller.cancel_join_history_worker(); }
        else { finish_ui_worker(&mut app, &ctx); }
        assert!(app.controller.top11_pending().is_some());
        if mode == 0 {
            let after = app.controller.document().clone();
            let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
            assert!(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap().contains(&[0.5, 0., 0.]));
            assert_ne!(app.controller.top11_pending(), Some(&before_pending));
            app.controller.submit("").unwrap();
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        } else {
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.top11_pending(), Some(&before_pending));
            assert_eq!(app.controller.coordinator().undo_len(), 2);
        }
    }
}

#[test]
fn interactive_trim_prevents_a_new_command_until_history_finishes() {
    let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line {
        start: [0.5, -1., 0.], end: [0.5, 1., 0.], domain: [0., 1.],
    }), None);
    let cutter_id = cutter.id;
    let (mut app, parent, _) = fixture_with_extras(vec![cutter], Vec::new());
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[parent, cutter_id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Trim").unwrap();
    app.controller.top11_trim_many_accept_pick(parent, [0.25, 0., 0.]).unwrap();
    assert!(app.controller.join_history_worker_pending());
    assert!(app.controller.top11_invoke(Top11Action::Move).is_err());
    finish_ui_worker(&mut app, &ctx);
    assert_ne!(app.controller.document(), &before);
    assert!(app.controller.top11_pending().is_some());
    assert_eq!(app.controller.coordinator().undo_len(), 3);
}

#[test]
fn interactive_trim_refuses_splitting_a_history_parent() {
    let cutter = |x| Entity::new("cutter", Geometry::Curve(Curve::Line {
        start: [x, -1., 0.], end: [x, 1., 0.], domain: [0., 1.],
    }), None);
    let a = cutter(0.25);
    let b = cutter(0.75);
    let ids = [a.id, b.id];
    let (mut app, parent, _) = fixture_with_extras(vec![a, b], Vec::new());
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[parent, ids[0], ids[1]]).unwrap();
    let before = app.controller.document().clone();
    app.controller.submit("Trim").unwrap();
    let pending = app.controller.top11_pending().unwrap().clone();
    assert!(matches!(app.controller.top11_trim_many_accept_pick(parent, [0.5, 0., 0.]), Err(om9_core::Error::Unsupported(_))));
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.top11_pending(), Some(&pending));
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 2);
}

#[test]
fn split_and_join_refuse_to_replace_history_parents() {
    for join in [false, true] {
        let (mut app, parent, outputs) = fixture();
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        let parents: Vec<_> = app.controller.document().history.recipes.values()
            .find(|recipe| recipe.outputs[0] == outputs[0]).unwrap().inputs.iter()
            .map(|input| input.source.object).collect();
        app.controller.select_objects(if join { &parents } else { std::slice::from_ref(&parent) }).unwrap();
        let before = app.controller.document().clone();
        let selection = app.controller.selected().clone();
        let result = if join {
            app.controller.top11_invoke(Top11Action::Join).map(|_| ())
        } else {
            app.controller.submit("Split 0.5").map(|_| ())
        };
        assert!(matches!(result, Err(om9_core::Error::Unsupported(_))), "{result:?}");
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.selected(), &selection);
        assert_eq!(app.controller.coordinator().undo_len(), 2);
        assert!(!app.controller.join_history_worker_pending());
    }
    let unrelated = Entity::new("unrelated", Geometry::Curve(Curve::Line {
        start: [10., 0., 0.], end: [12., 0., 0.], domain: [0., 1.],
    }), None);
    let id = unrelated.id;
    let (mut app, _, _) = fixture_with_extras(vec![unrelated], Vec::new());
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().entities.len();
    app.controller.submit("Split 0.5").unwrap();
    assert!(!app.controller.document().entities.contains_key(&id));
    assert_eq!(app.controller.document().entities.len(), before + 1);
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_trim_parent_rebuilds_history_and_restores_session() {
    use om9_core::surface_kernel;
    for surface_count in 1..=2 {
    for mode in 0..3 {
        let surface = Entity::new("surface", Geometry::Surface(surface_kernel::plane(
            [0.25, -0.5, 0.], [1., 0., 0.], [0., 1., 0.], 0.5, 1., &Limits::default()).unwrap()), None);
        let surface_id = surface.id;
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line {
            start: [0.5, -1., 0.], end: [0.5, 1., 0.], domain: [0., 1.],
        }), None);
        let cutter_id = cutter.id;
        let mut extras = vec![surface, cutter];
        let mut selected = vec![surface_id, cutter_id];
        if surface_count == 2 {
            let second = Entity::new("second surface", Geometry::Surface(surface_kernel::plane(
                [0.5, -0.5, -0.5], [0., 1., 0.], [0., 0., 1.], 1., 1., &Limits::default()).unwrap()), None);
            selected.push(second.id);
            extras.push(second);
        }
        let (mut app, parent, outputs) = fixture_with_extras(extras, Vec::new());
        app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
        selected.push(parent);
        app.controller.select_objects(&selected).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit("Trim").unwrap();
        let expected = if surface_count == 1 { "TrimFace" } else { "TrimSurfaces" };
        assert!(format!("{:?}", app.controller.top11_pending()).contains(expected));
        let before_pending = app.controller.top11_pending().unwrap().clone();
        let pick = if mode == 1 { [0.9, 0., 0.] } else { [0.1, 0., 0.] };
        app.controller.top11_trim_many_accept_pick(parent, pick).unwrap();
        assert!(app.controller.join_history_worker_pending());
        if mode == 2 { app.controller.cancel_join_history_worker(); }
        else { finish_ui_worker(&mut app, &ctx); }
        if mode == 0 {
            let after = app.controller.document().clone();
            let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            assert!(ends.iter().any(|p| (p[0] - 0.5).abs() < 1e-6), "{ends:?} / {}", app.transcript.text());
            assert!(format!("{:?}", app.controller.top11_pending()).contains("completed: 1"));
            app.controller.submit("").unwrap();
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        } else {
            assert_eq!(app.controller.document(), &before);
            assert_eq!(app.controller.top11_pending(), Some(&before_pending));
            assert_eq!(app.controller.coordinator().undo_len(), 2);
        }
    }
    }
}

#[cfg(feature = "occt-brep")]
fn mixed_mirror_fixture() -> (MatrixApp, om9_core::identity::ObjectId, [om9_core::identity::ObjectId; 2]) {
    use om9_core::budget::{Usage, memory::MemoryClass};
    use om9_kernel::creation::{self, Primitive};
    let (mut app, _, outputs) = fixture();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    let brep = creation::primitive(Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. },
        creation::DEFAULT_DEFLECTION_MM, &Limits::default()).unwrap();
    let bytes = brep.storage_usage(&Limits::default()).unwrap().bytes;
    let mut reservation = app.controller.coordinator().reserve_external_memory(MemoryClass::Kernel, bytes, None).unwrap();
    reservation.set_allocated_bytes(bytes).unwrap();
    let source = app.controller.coordinator().register_reserved_brep_candidate(brep, &mut reservation, Usage::default()).unwrap();
    let entity = Entity::new("box", Geometry::Brep(source), None);
    let box_id = entity.id;
    app.controller.commit_entity_reserved_brep("fixture", "box", entity, &mut reservation).unwrap();
    (app, box_id, outputs)
}

#[cfg(feature = "occt-brep")]
#[test]
fn mixed_brep_and_history_parents_mirror_atomically() {
    let (mut app, box_id, outputs) = mixed_mirror_fixture();
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut parents: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    parents.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    parents.push(box_id);
    app.controller.select_objects(&parents).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    app.controller.submit("Mirror").unwrap();
    app.controller.submit("Copy=No").unwrap();
    app.controller.submit("YAxis").unwrap();
    assert!(app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    finish_ui_worker(&mut app, &ctx);
    let after = app.controller.document().clone();
    assert_ne!(after.entities[&box_id].geometry, before.entities[&box_id].geometry, "{}", app.transcript.text());
    let Geometry::Brep(body) = &after.entities[&box_id].geometry else { panic!("BRep") };
    let measurement = om9_kernel::creation::measure(body, &Limits::default()).unwrap();
    assert!((measurement.volume_mm3 - 1.).abs() < 1e-6);
    assert!((measurement.bounds_mm[0] + 1.).abs() < 1e-6 && measurement.bounds_mm[3].abs() < 1e-6, "{:?}", measurement.bounds_mm);
    let Geometry::Curve(curve) = &after.entities[&outputs[1]].geometry else { panic!("curve") };
    assert!(om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap().contains(&[-3., 0., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn mixed_mirror_reflects_placed_brep_in_world_coordinates() {
    let (mut app, box_id, outputs) = mixed_mirror_fixture();
    app.controller.select_objects(&[box_id]).unwrap();
    app.controller.submit("Move").unwrap();
    app.controller.accept_world_point([0., 0., 0.]).unwrap();
    app.controller.accept_world_point([2., 0., 0.]).unwrap();
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut ids: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    ids.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    ids.push(box_id);
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    finish_ui_worker(&mut app, &ctx);
    let entity = &app.controller.document().entities[&box_id];
    assert_eq!(entity.placement, before.entities[&box_id].placement);
    let Geometry::Brep(body) = &entity.geometry else { panic!("BRep") };
    let measurement = om9_kernel::creation::measure(body, &Limits::default()).unwrap();
    assert!((measurement.volume_mm3 - 1.).abs() < 1e-6);
    assert!((measurement.bounds_mm[0] + 5.).abs() < 1e-6 && (measurement.bounds_mm[3] + 4.).abs() < 1e-6, "{:?} / {}", measurement.bounds_mm, app.transcript.text());
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
fn mixed_brep_mirror_failed_history_does_not_publish() {
    let (mut app, box_id, outputs) = mixed_mirror_fixture();
    let recipe = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let parent = recipe.inputs[0].source.object;
    app.controller.select_objects(&[parent, box_id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    assert!(app.controller.join_history_worker_pending());
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    assert!(app.transcript.text().contains("dependent history rebuild"), "{}", app.transcript.text());
}

#[cfg(feature = "occt-brep")]
#[test]
fn mixed_brep_mirror_cancel_does_not_publish() {
    let (mut app, box_id, outputs) = mixed_mirror_fixture();
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut parents: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    parents.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    parents.push(box_id);
    app.controller.select_objects(&parents).unwrap();
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    assert!(app.controller.join_history_worker_pending());
    app.controller.cancel_join_history_worker();
    assert!(!app.controller.join_history_worker_pending());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
}

#[cfg(feature = "occt-brep")]
#[test]
fn mixed_brep_mirror_stale_selection_does_not_publish() {
    let (mut app, box_id, outputs) = mixed_mirror_fixture();
    let first = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[0]).unwrap();
    let mut ids: Vec<_> = first.inputs.iter().map(|input| input.source.object).collect();
    let second = app.controller.document().history.recipes.values().find(|recipe| recipe.outputs[0] == outputs[1]).unwrap();
    ids.push(second.inputs.iter().find(|input| input.source.object != outputs[0]).unwrap().source.object);
    ids.push(box_id);
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    for input in ["Mirror", "Copy=No", "YAxis"] { app.controller.submit(input).unwrap(); }
    app.controller.select_objects(&[box_id]).unwrap();
    finish_ui_worker(&mut app, &ctx);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    assert!(app.transcript.text().contains("stale"), "{}", app.transcript.text());
}
