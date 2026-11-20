use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform},
    model::{Document, Entity},
    transaction::{Mutation, TransactionRequest},
};

#[test]
fn host_join_history_keeps_parents_records_recipe_and_undoes_together() {
    let line = |name, a, b| {
        Entity::new(
            name,
            Geometry::Curve(Curve::Line {
                start: a,
                end: b,
                domain: [0.0, 1.0],
            }),
            None,
        )
    };
    let a = line("a", [0.0; 3], [1.0, 0.0, 0.0]);
    let mut b = line("b", [0.0; 3], [1.0, 0.0, 0.0]);
    b.placement = Transform::translation([1.0, 0.0, 0.0]);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("join history");
    doc.entities.insert(a.id, a);
    doc.entities.insert(b.id, b);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.create_join_history().unwrap();
    let after = c.document().clone();
    assert_eq!(after.entities.len(), 3);
    assert!(ids.iter().all(|id| after.entities.contains_key(id)));
    let recipe = after.history.recipes.values().next().unwrap();
    let recipe_id = recipe.id;
    let output = recipe.outputs[0];
    assert_eq!(recipe.command, "JoinHistory");
    assert_eq!(
        recipe
            .inputs
            .iter()
            .map(|input| input.source.object)
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        recipe.parameters["tolerance_mm"].as_length().unwrap().mm(),
        0.001
    );
    let Geometry::Curve(curve) = &after.entities[&recipe.outputs[0]].geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(ends.contains(&[0.0; 3]) && ends.contains(&[2.0, 0.0, 0.0]));
    assert_eq!(c.coordinator().undo_len(), 1);
    let path = std::env::temp_dir().join(format!("om9-join-history-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(
        &path,
        &after,
        &Limits::default(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
    let reopened = om9_archive::Archive::open(&path, &Limits::default()).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(reopened, after);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.document(), &after);

    let mut c = Controller::new(reopened, Limits::default()).unwrap();
    c.select_objects(&[ids[1]]).unwrap();
    c.top11_invoke(om9_core::top11::Top11Action::ControlPoints)
        .unwrap();
    let end = c
        .top11_grip_handles()
        .unwrap()
        .into_iter()
        .find(|grip| grip.index == 1)
        .unwrap();
    c.top11_move_grip(end, [3.0, 0.0, 0.0]).unwrap();
    c.recompute_history(recipe_id).unwrap();
    let Geometry::Curve(joined) = &c.document().entities[&output].geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(joined, &Limits::default()).unwrap();
    assert!(ends.contains(&[0.0; 3]) && ends.contains(&[3.0, 0.0, 0.0]));
}

#[test]
fn host_join_history_rejects_disconnected_curves_without_document_or_undo_changes() {
    let line = |start, end| {
        Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    };
    let a = line([0.0; 3], [1.0, 0.0, 0.0]);
    let b = line([2.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("disconnected");
    doc.entities.insert(a.id, a);
    doc.entities.insert(b.id, b);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.create_join_history().is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

fn worker_fixture() -> (Controller, [om9_core::identity::ObjectId; 2], om9_core::identity::ObjectId) {
    let mut doc = Document::with_template_tolerance("history worker");
    let mut ids = Vec::new();
    for (a, b) in [(0.0, 1.0), (1.0, 2.0)] {
        let entity = Entity::new("source", Geometry::Curve(Curve::Line {
            start: [a, 0., 0.], end: [b, 0., 0.], domain: [0., 1.],
        }), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(doc, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    controller.create_join_history().unwrap();
    let output = controller.document().history.recipes.values().next().unwrap().outputs[0];
    (controller, [ids[0], ids[1]], output)
}

fn source_edit(controller: &Controller, id: om9_core::identity::ObjectId, start: f64, end: f64) -> TransactionRequest {
    TransactionRequest::new("grip edit", vec![Mutation::Geometry {
        source: controller.document().stamp(id).unwrap(),
        geometry: Geometry::Curve(Curve::Line { start: [start, 0., 0.], end: [end, 0., 0.], domain: [0., 1.] }),
    }])
}

fn finish_worker(controller: &mut Controller) -> om9_core::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Some(result) = controller.poll_join_history_worker() { return result; }
        assert!(std::time::Instant::now() < deadline, "history worker timed out");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn wait_for_scene_worker(controller: &mut Controller) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !controller.join_history_scene_pending() {
        assert!(controller.poll_join_history_worker().is_none(), "worker ended before scene phase");
        assert!(std::time::Instant::now() < deadline, "history scene worker timed out");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn explicit_worker_publishes_source_dependent_scene_and_one_undo() {
    let (mut controller, ids, output) = worker_fixture();
    let before = controller.document().clone();
    let request = source_edit(&controller, ids[1], 1.0, 3.0);
    controller.request_join_history_edit_worker(request).unwrap();
    finish_worker(&mut controller).unwrap();
    let after = controller.document().clone();
    let Geometry::Curve(curve) = &after.entities[&output].geometry else { panic!("joined curve") };
    let endpoints = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
    assert!(endpoints.contains(&[0., 0., 0.]) && endpoints.contains(&[3., 0., 0.]));
    let scene_object = controller.scene().unwrap().objects.iter().find(|object| object.root == output).unwrap();
    assert_eq!(scene_object.source, after.stamp(output).unwrap());
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document(), &before);
    controller.redo().unwrap();
    assert_eq!(controller.document(), &after);
}

#[test]
fn explicit_worker_refuses_stale_selection_and_failed_join() {
    let (mut controller, ids, _) = worker_fixture();
    let before = controller.document().clone();
    let request = source_edit(&controller, ids[1], 1.0, 3.0);
    controller.request_join_history_edit_worker(request).unwrap();
    controller.select_objects(&[ids[0]]).unwrap();
    assert!(matches!(finish_worker(&mut controller), Err(om9_core::Error::Stale(_))));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let request = source_edit(&controller, ids[1], 3.0, 4.0);
    controller.request_join_history_edit_worker(request).unwrap();
    assert!(finish_worker(&mut controller).is_err());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn canceling_explicit_worker_keeps_document_and_undo() {
    let (mut controller, ids, _) = worker_fixture();
    let before = controller.document().clone();
    let request = source_edit(&controller, ids[1], 1.0, 3.0);
    controller.request_join_history_edit_worker(request).unwrap();
    assert!(controller.join_history_worker_pending());
    controller.cancel();
    assert!(!controller.join_history_worker_pending());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn scene_worker_rejects_stale_selection_and_cancellation() {
    let (mut controller, ids, _) = worker_fixture();
    let before = controller.document().clone();
    let request = source_edit(&controller, ids[1], 1.0, 3.0);
    controller.request_join_history_edit_worker(request).unwrap();
    wait_for_scene_worker(&mut controller);
    controller.select_objects(&[ids[0]]).unwrap();
    assert!(matches!(finish_worker(&mut controller), Err(om9_core::Error::Stale(_))));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 1);

    let request = source_edit(&controller, ids[1], 1.0, 3.0);
    controller.request_join_history_edit_worker(request).unwrap();
    wait_for_scene_worker(&mut controller);
    controller.cancel();
    assert!(!controller.join_history_worker_pending());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 1);
}
