// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-004 (Interactive Command State Machine),
//! driven by `implementation/fixtures/M01/OM9-L3-01-02-004/`.
use om9_app::session::{CmdPhase, Controller, InvocationOrigin, LINE, POINT};
use om9_core::budget::Limits;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-02-004")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn new_controller() -> Controller {
    Controller::new(Document::new("state machine"), Limits::default()).expect("controller")
}

fn phase_name(phase: CmdPhase) -> &'static str {
    match phase {
        CmdPhase::Idle => "Idle",
        CmdPhase::AwaitPoint => "AwaitPoint",
        CmdPhase::AwaitEnd => "AwaitEnd",
    }
}

fn undo_len(controller: &Controller) -> usize {
    controller.coordinator().undo_len()
}

#[test]
fn m1_to_m4_the_command_walks_its_phases() {
    let document = fixture("positive.json");

    // M1: LINE, two points, three phases and one entity.
    let case = &document["cases"][0];
    let mut controller = new_controller();
    let mut phases = Vec::new();
    let mut prompts = Vec::new();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    phases.push(phase_name(controller.phase()));
    prompts.push(controller.prompt().to_owned());
    controller.accept_world_point([0., 0., 0.]).unwrap();
    phases.push(phase_name(controller.phase()));
    prompts.push(controller.prompt().to_owned());
    controller.accept_world_point([10., 0., 0.]).unwrap();
    phases.push(phase_name(controller.phase()));
    prompts.push(controller.prompt().to_owned());
    let expected: Vec<&str> = case["expect"]["phases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(phases, expected);
    let expected_prompts: Vec<&str> = case["expect"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(prompts, expected_prompts);
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(undo_len(&controller), 1);

    // M2: POINT commits on its single phase.
    let case = &document["cases"][1];
    let mut controller = new_controller();
    controller.dispatch(POINT, InvocationOrigin::Cmd).unwrap();
    assert_eq!(phase_name(controller.phase()), "AwaitPoint");
    controller.accept_world_point([1., 2., 3.]).unwrap();
    assert_eq!(phase_name(controller.phase()), "Idle");
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(case["expect"]["entities"], 1);

    // M3: Escape aborts in every phase and the abort is total.
    let case = &document["cases"][2];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.cancel();
    assert_eq!(
        phase_name(controller.phase()),
        case["expect"]["phase_after_escape"].as_str().unwrap()
    );
    assert_eq!(
        controller.prompt(),
        case["expect"]["prompt_after_escape"].as_str().unwrap()
    );
    assert!(controller.accept_world_point([5., 5., 5.]).is_err());
    assert!(controller.document().entities.is_empty());
    assert_eq!(undo_len(&controller), 0);

    // M4: the option belongs to the running command and shows in the prompt.
    let case = &document["cases"][3];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.submit("BothSides=Yes").unwrap();
    assert_eq!(phase_name(controller.phase()), "AwaitPoint");
    assert_eq!(
        controller.prompt(),
        case["expect"]["prompt"].as_str().unwrap()
    );
}

#[test]
fn n1_to_n6_out_of_order_input_cannot_commit_partial_geometry() {
    let document = fixture("negative.json");
    let case = |prefix: &str| {
        document["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"].as_str().unwrap().starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix}"))
            .clone()
    };

    // N1: Enter while a point is awaited.
    let n1 = case("N1");
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    let error = controller.submit("").expect_err("Enter cannot commit");
    assert!(matches!(error, om9_core::Error::Invalid(_)), "{error}");
    assert_eq!(
        phase_name(controller.phase()),
        n1["expect"]["phase"].as_str().unwrap()
    );
    assert!(controller.document().entities.is_empty());

    // N2: an extra click after the commit changes nothing.
    let n2 = case("N2");
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([10., 0., 0.]).unwrap();
    let entities = controller.document().entities.len();
    let undos = undo_len(&controller);
    assert!(controller.accept_world_point([20., 0., 0.]).is_err());
    assert_eq!(controller.document().entities.len(), entities);
    assert_eq!(undo_len(&controller), undos);
    assert_eq!(entities as u64, n2["expect"]["entities"].as_u64().unwrap());

    // N3: a second command cannot start while one runs.
    let n3 = case("N3");
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let error = controller
        .dispatch(POINT, InvocationOrigin::Cmd)
        .expect_err("busy");
    assert!(matches!(error, om9_core::Error::Busy), "{error}");
    assert_eq!(
        phase_name(controller.phase()),
        n3["expect"]["phase"].as_str().unwrap()
    );

    // N4: a zero-length line is refused and the command stays open.
    let n4 = case("N4");
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    let error = controller
        .accept_world_point([0., 0., 0.])
        .expect_err("degenerate");
    assert!(matches!(error, om9_core::Error::Invalid(_)), "{error}");
    assert_eq!(
        phase_name(controller.phase()),
        n4["expect"]["phase"].as_str().unwrap()
    );
    assert!(controller.document().entities.is_empty());

    // N5: a stale session is neither running nor asking.
    let n5 = case("N5");
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    // The run started in this view; taking the view away is a lost context.
    // (Switching the ACTIVE view is not: chrome_actions requires the command and
    // its numeric draft to survive a view-tab switch.)
    let started = controller.context();
    controller.workspace.views.remove(&started.view);
    assert!(!controller.is_running());
    assert_eq!(
        phase_name(controller.phase()),
        n5["expect"]["phase"].as_str().unwrap()
    );
    assert_eq!(
        controller.prompt(),
        n5["expect"]["prompt"].as_str().unwrap()
    );

    // N6: a command name typed while running is not a point.
    let n6 = case("N6");
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    let error = controller.submit("line").expect_err("not a coordinate");
    assert!(matches!(error, om9_core::Error::Invalid(_)), "{error}");
    assert!(controller.document().entities.is_empty());
    assert_eq!(
        phase_name(controller.phase()),
        n6["expect"]["phase"].as_str().unwrap()
    );
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-02-004/api.md");
    assert!(api.exists(), "{}", api.display());
}
