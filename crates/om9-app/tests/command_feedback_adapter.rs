// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-03-10 command-line feedback adapter: canonical echo, prompt cancel and numeric refusals.
use om9_app::session::{CmdPhase, Controller, LINE, POINT};
use om9_core::{budget::Limits, model::Document};

fn controller() -> Controller {
    Controller::new(Document::new("cmd feedback"), Limits::default()).expect("controller")
}

#[test]
fn the_running_command_exposes_its_canonical_feature_and_prompt() {
    let mut controller = controller();
    assert_eq!(controller.active_feature(), None);
    controller.submit("line").expect("alias starts the command");
    assert_eq!(
        controller.active_feature(),
        Some(LINE),
        "the alias resolves to the stable feature id the command line echoes"
    );
    assert_eq!(controller.phase(), CmdPhase::AwaitPoint);
    assert_eq!(
        controller.prompt(),
        "Start of line (BothSides=No Length=): "
    );
    controller.submit("0,0,0").expect("start point");
    assert_eq!(controller.phase(), CmdPhase::AwaitEnd);
    assert_eq!(controller.prompt(), "End of line (BothSides=No Length=): ");
    controller.submit("10,0,0").expect("end point");
    assert!(!controller.is_running());
    assert_eq!(
        controller.active_feature(),
        Some(LINE),
        "after completion the same id stays in the repeat slot"
    );
}

#[test]
fn a_locale_comma_and_nan_are_refused_without_mutating_the_command() {
    let mut controller = controller();
    controller.submit("Line").expect("Line");
    // A length option with a locale comma or NaN must be refused, leave the option unset and keep the
    // command running for a corrected value.
    assert!(
        controller.submit("Length=12,5").is_err(),
        "comma decimal is refused"
    );
    assert_eq!(
        controller.phase(),
        CmdPhase::AwaitPoint,
        "the command stays active"
    );
    assert!(
        controller.submit("Length=NaN").is_err(),
        "NaN is refused for a numeric option"
    );
    assert_eq!(controller.phase(), CmdPhase::AwaitPoint);
    // The valid value then applies through the same typed path.
    controller.submit("Length=12.5").expect("valid length");
    controller.submit("0,0,0").expect("start");
    controller
        .submit("100,0,0")
        .expect("end uses the fixed length");
    let line = controller
        .document()
        .entities
        .values()
        .find_map(|entity| match &entity.geometry {
            om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                ..
            }) => Some((*start, *end)),
            _ => None,
        })
        .expect("line");
    let length = ((line.1[0] - line.0[0]).powi(2)
        + (line.1[1] - line.0[1]).powi(2)
        + (line.1[2] - line.0[2]).powi(2))
    .sqrt();
    assert!((length - 12.5).abs() < 1e-6, "typed length used: {length}");
}

#[test]
fn a_document_change_during_a_prompt_cancels_it_safely() {
    let mut controller = controller();
    controller.submit("Point").expect("Point");
    assert_eq!(controller.phase(), CmdPhase::AwaitPoint);
    // A committed transaction while the prompt waits invalidates the captured context.
    controller.insert_stress_meshes(1).expect("document change");
    assert_eq!(controller.phase(), CmdPhase::Idle, "the prompt is gone");
    assert_eq!(controller.prompt(), "Command: ");
    assert!(
        controller.submit("1,2,3").is_err(),
        "a stale point input is refused instead of being applied to the wrong document state"
    );
    // The cancelled command may leave its feature in the repeat slot; it must never resurrect a stale session.
    assert!(
        controller.active_feature().is_none() || controller.active_feature() == Some(POINT),
        "{:?}",
        controller.active_feature()
    );
    // The point was never created.
    assert_eq!(
        controller.document().entities.len(),
        1,
        "only the stress mesh exists"
    );
    let _ = POINT;
}

#[test]
fn registry_aliases_reach_the_same_canonical_command() {
    let mut controller = controller();
    controller.submit("Ln").expect("Ln is a declared alias");
    assert_eq!(controller.active_feature(), Some(LINE));
    controller.submit("0,0,0").expect("start");
    controller.submit("10,0,0").expect("end");
    controller.submit("Pt").expect("Pt is a declared alias");
    assert_eq!(controller.active_feature(), Some(POINT));
    assert_eq!(controller.prompt(), "Location of point: ");
}

#[test]
fn the_underlined_option_letter_toggles_the_flag() {
    let mut controller = controller();
    controller.submit("Line").expect("Line");
    controller
        .submit("b")
        .expect("B activates BothSides (Rhino p22)");
    assert_eq!(controller.both_sides(), Some(true));
    assert!(controller.prompt().contains("BothSides=Yes"));
    controller.submit("B").expect("again");
    assert_eq!(controller.both_sides(), Some(false));
}
