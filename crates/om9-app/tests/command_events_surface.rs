// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-010 (Command Event And Observer API), driven
//! by `implementation/fixtures/M01/OM9-L3-01-02-010/`.
use om9_app::command_events::{EventKind, EventLog, EventOutcome, Observer, notify, notify_all};
use om9_app::session::{CmdPhase, Controller, InvocationOrigin, LINE};
use om9_core::budget::Limits;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-02-010")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn new_controller() -> Controller {
    Controller::new(Document::new("command events"), Limits::default()).expect("controller")
}

fn kinds(controller: &Controller) -> Vec<&'static str> {
    controller
        .events
        .events()
        .iter()
        .map(|event| event.kind.label())
        .collect()
}

fn sequences(controller: &Controller) -> Vec<u64> {
    controller
        .events
        .events()
        .iter()
        .map(|event| event.sequence)
        .collect()
}

struct Recorder {
    seen: Vec<u64>,
}
impl Observer for Recorder {
    fn observe(&mut self, event: &om9_app::command_events::CommandEvent) {
        self.seen.push(event.sequence);
    }
}

struct Exploding;
impl Observer for Exploding {
    fn observe(&mut self, _event: &om9_app::command_events::CommandEvent) {
        panic!("observer exploded");
    }
}

#[test]
fn z1_a_run_reports_its_steps_in_order() {
    let document = fixture("positive.json");
    let case = &document["cases"][0];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.warn("Length=NaN refused");
    controller.accept_world_point([10., 0., 0.]).unwrap();

    let expected: Vec<&str> = case["expect"]["kinds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(
        kinds(&controller),
        expected,
        "{:?}",
        controller.events.events()
    );
    let expected_sequences: Vec<u64> = case["expect"]["sequences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap())
        .collect();
    assert_eq!(sequences(&controller), expected_sequences);
    assert!(controller.events.sequence_is_monotonic());
    assert_eq!(
        controller.events.run_len(LINE) as u64,
        case["expect"]["run_len"].as_u64().unwrap()
    );
    assert_eq!(
        controller.events.last().unwrap().outcome,
        Some(EventOutcome::Committed)
    );
    assert_eq!(controller.document().entities.len(), 1);
}

#[test]
fn z2_a_cancelled_run_finishes_with_the_cancel_outcome() {
    let document = fixture("positive.json");
    let case = &document["cases"][1];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.cancel();
    let last = controller.events.last().expect("a finished event");
    assert_eq!(
        last.kind.label(),
        case["expect"]["last_kind"].as_str().unwrap()
    );
    assert_eq!(
        format!("{:?}", last.outcome.unwrap()),
        case["expect"]["last_outcome"].as_str().unwrap()
    );
    assert!(controller.events.sequence_is_monotonic());
    assert!(controller.document().entities.is_empty());
}

#[test]
fn z3_and_z4_the_order_is_total_and_the_log_is_bounded() {
    let document = fixture("positive.json");

    // Z3: a second run continues the sequence.
    let case = &document["cases"][2];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let first = controller.events.last().unwrap().sequence;
    controller.cancel();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let second = controller.events.last().unwrap().sequence;
    assert!(second > first, "{first} then {second}");
    assert_eq!(case["expect"]["first_of_second_run_is_greater"], true);
    assert!(controller.events.sequence_is_monotonic());

    // Z4: the bound, with the sequence still rising.
    let case = &document["cases"][3];
    let mut log = EventLog::new();
    let emit = case["emit"].as_u64().unwrap();
    for index in 0..emit {
        log.emit(EventKind::Progress, "line", &format!("step {index}"), None);
    }
    assert_eq!(log.len() as u64, case["expect"]["len"].as_u64().unwrap());
    assert!(log.sequence_is_monotonic());
    assert_eq!(log.events().back().unwrap().sequence, emit);
}

#[test]
fn aa1_to_aa4_a_broken_observer_cannot_touch_the_run() {
    let document = fixture("negative.json");
    let _ = document;

    // AA1: a panicking observer is contained, the others still hear it, and the
    // log the run wrote is untouched.
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let event = controller.events.last().unwrap().clone();
    let before = kinds(&controller);
    let mut good = Recorder { seen: Vec::new() };
    let mut bad = Exploding;
    let survivors = notify_all(&mut [&mut bad, &mut good], &event);
    assert_eq!(survivors, 1, "the panic was contained, not propagated");
    assert_eq!(good.seen, vec![event.sequence]);
    assert_eq!(kinds(&controller), before, "the log is untouched");
    assert!(
        !notify(&mut Exploding, &event),
        "notify reports the failure"
    );

    // AA2: the run still commits afterwards, with exactly one entity and entry.
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([10., 0., 0.]).unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 1);
    assert_eq!(
        controller.events.last().unwrap().outcome,
        Some(EventOutcome::Committed)
    );

    // AA3: a refused command adds no event.
    let before = kinds(&controller);
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let with_start = kinds(&controller);
    assert_eq!(with_start.len(), before.len() + 1);
    controller.accept_world_point([1., 1., 0.]).unwrap();
    assert!(controller.accept_world_point([1., 1., 0.]).is_err());
    assert_eq!(controller.phase(), CmdPhase::AwaitEnd);
    let after_refusal = kinds(&controller);
    assert_eq!(
        after_refusal.len(),
        with_start.len() + 1,
        "only the progress event of the first point: a refusal invents nothing"
    );
    controller.cancel();

    // AA4: warn with no run changes nothing.
    let before = controller.events.len();
    controller.warn("nothing is running");
    assert_eq!(controller.events.len(), before);
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-02-010/api.md");
    assert!(api.exists(), "{}", api.display());
}
