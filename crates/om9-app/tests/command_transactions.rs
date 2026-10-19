// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-006 (Atomic Command Transactions),
//! OM9-L3-01-02-007 (Undo Redo Journal) and OM9-L3-01-02-008 (Command Cancel
//! Rollback), driven by their fixtures.
use om9_app::session::{CmdPhase, Controller, InvocationOrigin, LINE, POINT};
use om9_core::budget::Limits;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(l3: &str, name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01")
        .join(l3)
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn new_controller() -> Controller {
    Controller::new(Document::new("transactions"), Limits::default()).expect("controller")
}

fn line(controller: &mut Controller) {
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([10., 0., 0.]).unwrap();
}

fn counts(controller: &Controller) -> (usize, usize, usize) {
    (
        controller.document().entities.len(),
        controller.coordinator().undo_len(),
        controller.coordinator().redo_len(),
    )
}

#[test]
fn t1_to_t3_one_action_is_one_journal_entry() {
    let document = fixture("OM9-L3-01-02-006", "positive.json");
    let case = |prefix: &str| {
        document["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"].as_str().unwrap().starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix}"))
            .clone()
    };

    let mut controller = new_controller();
    let revision = controller.document().context_revision;
    line(&mut controller);
    let (entities, undo, redo) = counts(&controller);
    let t1 = case("T1");
    assert_eq!(entities as u64, t1["expect"]["entities"].as_u64().unwrap());
    assert_eq!(undo as u64, t1["expect"]["undo_entries"].as_u64().unwrap());
    assert_eq!(redo as u64, t1["expect"]["redo_entries"].as_u64().unwrap());

    line(&mut controller);
    let t2 = case("T2");
    let (entities, undo, _) = counts(&controller);
    assert_eq!(entities as u64, t2["expect"]["entities"].as_u64().unwrap());
    assert_eq!(undo as u64, t2["expect"]["undo_entries"].as_u64().unwrap());

    // T3: the identity token is not the per-action counter, and the api.md says so.
    let t3 = case("T3");
    assert_eq!(
        controller.document().context_revision == revision,
        t3["expect"]["context_revision_unchanged_by_an_edit"]
            .as_bool()
            .unwrap()
    );
}

#[test]
fn u1_to_u4_a_failure_writes_nothing() {
    let document = fixture("OM9-L3-01-02-006", "negative.json");
    let _ = document;

    // U1: a zero-length line is refused and the command stays open.
    let mut controller = new_controller();
    line(&mut controller);
    let before = counts(&controller);
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([5., 5., 0.]).unwrap();
    assert!(controller.accept_world_point([5., 5., 0.]).is_err());
    assert_eq!(counts(&controller).0, before.0);
    assert_eq!(counts(&controller).1, before.1);
    assert_eq!(controller.phase(), CmdPhase::AwaitEnd);

    // U2: a second command while one runs.
    let before = counts(&controller);
    assert!(matches!(
        controller.dispatch(POINT, InvocationOrigin::Cmd),
        Err(om9_core::Error::Busy)
    ));
    assert_eq!(counts(&controller), before);

    // U3: Escape after the first point.
    controller.cancel();
    let before = counts(&controller);
    assert_eq!(controller.phase(), CmdPhase::Idle);
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([1., 1., 0.]).unwrap();
    controller.cancel();
    assert_eq!(counts(&controller), before);

    // U4: a stale session writes nothing.
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([2., 2., 0.]).unwrap();
    // The stale trigger is the LOSS of the view the run started in: a mere
    // view-tab switch keeps a running command (chrome_actions).
    let started = controller.context();
    controller.workspace.views.remove(&started.view);
    assert!(controller.accept_world_point([3., 3., 0.]).is_err());
    assert_eq!(counts(&controller), before);
}

#[test]
fn v1_to_v2_and_w1_to_w4_the_journal_walks_back_and_forward() {
    let positive = fixture("OM9-L3-01-02-007", "positive.json");
    let negative = fixture("OM9-L3-01-02-007", "negative.json");

    // V1: undo removes, redo restores the same id and layer.
    let v1 = &positive["cases"][0];
    let mut controller = new_controller();
    line(&mut controller);
    let id_before = controller.document().entities.values().next().unwrap().id;
    let layer_before = controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .attributes
        .layer;
    controller.undo().unwrap();
    let (entities, undo, redo) = counts(&controller);
    assert_eq!(
        entities as u64,
        v1["expect"]["after_undo"]["entities"].as_u64().unwrap()
    );
    assert_eq!(
        undo as u64,
        v1["expect"]["after_undo"]["undo_entries"].as_u64().unwrap()
    );
    assert_eq!(
        redo as u64,
        v1["expect"]["after_undo"]["redo_entries"].as_u64().unwrap()
    );
    controller.redo().unwrap();
    let (entities, undo, redo) = counts(&controller);
    assert_eq!(
        entities as u64,
        v1["expect"]["after_redo"]["entities"].as_u64().unwrap()
    );
    assert_eq!(
        undo as u64,
        v1["expect"]["after_redo"]["undo_entries"].as_u64().unwrap()
    );
    assert_eq!(
        redo as u64,
        v1["expect"]["after_redo"]["redo_entries"].as_u64().unwrap()
    );
    let entity = controller.document().entities.values().next().unwrap();
    assert_eq!(entity.id, id_before, "the same object comes back");
    assert_eq!(entity.attributes.layer, layer_before, "on the same layer");

    // V2: three commands, three undos, then three redos.
    let v2 = &positive["cases"][1];
    let mut controller = new_controller();
    for _ in 0..3 {
        line(&mut controller);
    }
    for _ in 0..3 {
        controller.undo().unwrap();
    }
    assert_eq!(
        controller.document().entities.len() as u64,
        v2["expect"]["undo_three"].as_u64().unwrap()
    );
    assert_eq!(
        controller.coordinator().redo_len() as u64,
        v2["expect"]["redo_three"].as_u64().unwrap()
    );

    // W1: a new command after an undo clears the redo side.
    let w1 = &negative["cases"][0];
    let mut controller = new_controller();
    line(&mut controller);
    controller.undo().unwrap();
    let redo_before = controller.coordinator().redo_len();
    line(&mut controller);
    assert_eq!(
        redo_before as u64,
        w1["expect"]["redo_before"].as_u64().unwrap()
    );
    assert_eq!(
        controller.coordinator().redo_len() as u64,
        w1["expect"]["redo_after"].as_u64().unwrap(),
        "stale forward references are dropped"
    );

    // W2/W3: nothing to redo or undo is refused.
    assert_eq!(negative["cases"][1]["expect"]["refused"], true);
    assert!(controller.redo().is_err());
    let mut fresh = new_controller();
    assert!(fresh.undo().is_err());

    // W4: undo is refused while a command runs.
    let w4 = &negative["cases"][3];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let error = controller.undo().expect_err("busy");
    assert_eq!(
        match &error {
            om9_core::Error::Busy => "Busy",
            other => panic!("unexpected {other}"),
        },
        w4["expect"]["error"].as_str().unwrap()
    );
}

#[test]
fn x1_to_x3_and_y1_to_y4_a_cancel_leaves_no_orphan() {
    let positive = fixture("OM9-L3-01-02-008", "positive.json");
    let negative = fixture("OM9-L3-01-02-008", "negative.json");

    // X1/X2/X3: the preview lives only while the command does.
    let x1 = &positive["cases"][0];
    let x2 = &positive["cases"][1];
    let x3 = &positive["cases"][2];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    let preview = controller.preview([4., 0., 0.]).unwrap();
    assert_eq!(
        preview.is_some(),
        x1["expect"]["preview_during"].as_bool().unwrap()
    );
    assert_eq!(
        preview.expect("a rubber band") == ([0., 0., 0.], [4., 0., 0.]),
        x1["expect"]["preview_is_start_to_cursor"]
            .as_bool()
            .unwrap()
    );
    controller.cancel();
    assert!(controller.preview([4., 0., 0.]).unwrap().is_none());
    assert_eq!(
        controller.is_running(),
        x2["expect"]["running"].as_bool().unwrap()
    );
    let (entities, undo, redo) = counts(&controller);
    assert_eq!(entities as u64, x3["expect"]["entities"].as_u64().unwrap());
    assert_eq!(undo as u64, x3["expect"]["undo_entries"].as_u64().unwrap());
    assert_eq!(redo as u64, x3["expect"]["redo_entries"].as_u64().unwrap());

    // Y1/Y2: no orphan entity, and the old preview cannot be committed.
    let _ = negative;
    let before = counts(&controller);
    assert!(controller.accept_world_point([4., 0., 0.]).is_err());
    assert_eq!(counts(&controller), before);

    // Y3: cancelling twice is harmless.
    controller.cancel();
    controller.cancel();
    assert_eq!(counts(&controller), before);

    // Y4: a refusal is not a cancel — the retry commits exactly once.
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    assert!(controller.accept_world_point([0., 0., 0.]).is_err());
    assert_eq!(controller.phase(), CmdPhase::AwaitEnd);
    controller.accept_world_point([3., 0., 0.]).unwrap();
    let (entities, undo, _) = counts(&controller);
    assert_eq!((entities, undo), (1, 1));
}

#[test]
fn fixture_design_pairs_are_present() {
    for l3 in ["OM9-L3-01-02-006", "OM9-L3-01-02-007", "OM9-L3-01-02-008"] {
        for name in ["positive.json", "negative.json"] {
            let _ = fixture(l3, name);
        }
        let api = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../implementation/design/M01")
            .join(l3)
            .join("api.md");
        assert!(api.exists(), "{}", api.display());
    }
}
