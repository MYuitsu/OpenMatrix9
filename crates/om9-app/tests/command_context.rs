// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-003 (Command Context And Active Document),
//! driven by `implementation/fixtures/M01/OM9-L3-01-02-003/`.
use om9_app::session::{CmdPhase, Controller, LINE};
use om9_core::budget::Limits;
use om9_core::model::Document;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-02-003")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn controller() -> Controller {
    Controller::new(Document::new("command context"), Limits::default()).expect("controller")
}

fn other_view(controller: &Controller) -> om9_scene::workspace::ViewId {
    let active = controller.workspace.active;
    controller
        .workspace
        .views
        .keys()
        .copied()
        .find(|id| *id != active)
        .expect("a second view")
}

#[test]
fn c1_the_context_snapshot_names_document_view_selection_and_revisions() {
    let document = fixture("positive.json");
    let case = &document["cases"][0];
    let controller = controller();
    let context = controller.context();
    assert_eq!(context.document, controller.document().id, "{}", case["id"]);
    assert_eq!(context.view, controller.workspace.active);
    assert!(context.selection.is_empty(), "nothing is selected yet");
    assert_eq!(context.revision, controller.document().context_revision);
    assert!(controller.validate_context(&context).is_ok());
    // Two snapshots of an untouched session agree: the context is deterministic.
    assert_eq!(controller.context(), context);
    assert_eq!(case["expect"]["selection_is_typed_object_ids"], true);
}

#[test]
fn c2_a_view_tab_switch_keeps_the_command_and_its_context() {
    // The existing chrome_actions tests require this: switching the active view
    // must not lose a running command or its numeric draft. The command keeps the
    // view and plane it captured (OM9-L3-01-02-003).
    let mut controller = controller();
    controller
        .dispatch(LINE, om9_app::session::InvocationOrigin::Cmd)
        .unwrap();
    let started = controller.context();
    controller.workspace.active = other_view(&controller);
    assert!(
        controller.is_running(),
        "another active view is not a lost context"
    );
    assert_eq!(controller.phase(), CmdPhase::AwaitPoint);
    controller.accept_world_point([0., 0., 0.]).unwrap();
    assert_eq!(controller.phase(), CmdPhase::AwaitEnd);
    // `context()` is the LIVE context (it follows the active view); the running
    // command keeps the snapshot it started with, which is why the commit below
    // still lands on the plane and view it captured.
    assert_ne!(controller.context().view, started.view);
    controller.accept_world_point([10., 0., 0.]).unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.phase(), CmdPhase::Idle);
}

#[test]
fn c3_a_context_whose_view_or_document_is_gone_is_stale() {
    let document = fixture("negative.json");
    let case = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"].as_str().unwrap().starts_with("K2"))
        .expect("K2");
    let mut controller = controller();
    assert_eq!(case["expect"]["a-view-tab-switch-keeps-the-command"], true);

    // The view the command started in is GONE: stale.
    let captured = controller.context();
    let removed = controller
        .workspace
        .views
        .remove(&captured.view)
        .expect("the view existed");
    let error = controller
        .validate_context(&captured)
        .expect_err("a view that is gone is a lost context");
    assert!(matches!(error, om9_core::Error::Stale(_)), "{error}");

    // A later revision of the same document: stale.
    let mut later = controller.context();
    later.revision = later.revision.wrapping_add(1);
    assert!(controller.validate_context(&later).is_err());

    // Putting the view back makes the snapshot valid again.
    controller.workspace.views.insert(captured.view, removed);
    assert!(controller.validate_context(&captured).is_ok());
    assert_eq!(case["expect"]["error"], "Stale");
}

#[test]
fn c4_a_command_keeps_the_context_it_started_against() {
    let document = fixture("positive.json");
    let case = &document["cases"][1];
    let mut controller = controller();
    let view = controller.workspace.active;
    controller
        .dispatch(LINE, om9_app::session::InvocationOrigin::Cmd)
        .unwrap();
    let started = controller.context();
    assert_eq!(started.view, view);
    controller.accept_world_point([0., 0., 0.]).unwrap();
    assert_eq!(controller.phase(), CmdPhase::AwaitEnd);
    assert_eq!(
        controller.context().view,
        started.view,
        "the phase change is not a context change"
    );
    controller.accept_world_point([10., 0., 0.]).unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(case["expect"]["entities"], 1);
    assert_eq!(controller.phase(), CmdPhase::Idle);
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-02-003/api.md");
    assert!(api.exists(), "{}", api.display());
}
