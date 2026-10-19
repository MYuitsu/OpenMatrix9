// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-02-005 (Command Option Parser) and
//! OM9-L3-01-02-009 (Command Repeat And Dispatch), driven by their fixtures.
use om9_app::option_parser::parse_option;
use om9_app::session::{CmdPhase, Controller, InvocationOrigin, LINE, POINT};
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::geometry::Unit;
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
    Controller::new(Document::new("options and repeat"), Limits::default()).expect("controller")
}

fn point3(value: &Value) -> [f64; 3] {
    let list = value.as_array().expect("point");
    [
        list[0].as_f64().unwrap(),
        list[1].as_f64().unwrap(),
        list[2].as_f64().unwrap(),
    ]
}

fn line_length(controller: &Controller) -> f64 {
    let entity = controller
        .document()
        .entities
        .values()
        .next()
        .expect("a line");
    match &entity.geometry {
        Geometry::Curve(om9_core::geometry::Curve::Line {
            domain, start, end, ..
        }) => {
            let _ = (start, end);
            domain[1] - domain[0]
        }
        other => panic!("expected a line, got {other:?}"),
    }
}

fn phase_name(phase: CmdPhase) -> &'static str {
    match phase {
        CmdPhase::Idle => "Idle",
        CmdPhase::AwaitPoint => "AwaitPoint",
        CmdPhase::AwaitEnd => "AwaitEnd",
    }
}

#[test]
fn o1_to_o4_the_parser_reads_the_declared_options() {
    let document = fixture("OM9-L3-01-02-005", "positive.json");
    let descriptor = Controller::line_descriptor();

    // O1: a flag, without case or padding.
    let case = &document["cases"][0];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let typed: Vec<&str> = case["typed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let expected: Vec<&str> = case["expect"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    for (index, text) in typed.iter().enumerate() {
        controller
            .submit(text)
            .unwrap_or_else(|error| panic!("{text}: {error}"));
        assert_eq!(controller.prompt(), expected[index], "{text}");
    }

    // O2/O3: a length fixes the distance, the click chooses the direction.
    for case in document["cases"].as_array().unwrap().iter().skip(1).take(2) {
        let id = case["id"].as_str().unwrap();
        let mut controller = new_controller();
        controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
        controller
            .submit(case["typed"][0].as_str().unwrap())
            .unwrap();
        controller
            .accept_world_point(point3(&case["start"]))
            .unwrap();
        controller
            .accept_world_point(point3(&case["click"]))
            .unwrap();
        let expected = case["expect"]["line_length_mm"].as_f64().unwrap();
        let measured = line_length(&controller);
        assert!(
            (measured - expected).abs() < 1e-9,
            "{id}: {measured} != {expected}"
        );
        if let Some(end) = case["expect"]["end"].as_array() {
            let entity = controller.document().entities.values().next().unwrap();
            match &entity.geometry {
                Geometry::Curve(om9_core::geometry::Curve::Line { end: actual, .. }) => {
                    for (index, expected) in end.iter().enumerate() {
                        assert!(
                            (actual[index] - expected.as_f64().unwrap()).abs() < 1e-9,
                            "{id}: {actual:?}"
                        );
                    }
                }
                other => panic!("{id}: {other:?}"),
            }
        }
    }

    // O4: a point is not an option.
    let case = &document["cases"][3];
    assert_eq!(case["expect"]["is_option"], false);
    assert!(
        parse_option(
            Unit::Millimeter,
            &descriptor,
            case["typed"][0].as_str().unwrap()
        )
        .unwrap()
        .is_none()
    );
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller
        .submit(case["typed"][0].as_str().unwrap())
        .unwrap();
    assert_eq!(
        phase_name(controller.phase()),
        case["expect"]["phase"].as_str().unwrap()
    );
}

#[test]
fn p1_to_p10_every_bad_option_is_refused_with_a_reason() {
    let document = fixture("OM9-L3-01-02-005", "negative.json");
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let typed = case["typed"].as_str().unwrap();
        let mut controller = new_controller();
        controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
        let error = controller
            .submit(typed)
            .expect_err(&format!("{id}: {typed} must be refused"));
        let expected = case["expect"]["error"].as_str().unwrap();
        let actual = match &error {
            om9_core::Error::Invalid(_) => "Invalid",
            om9_core::Error::Unsupported(_) => "Unsupported",
            om9_core::Error::Missing(_) => "Missing",
            other => panic!("{id}: unexpected {other}"),
        };
        assert_eq!(actual, expected, "{id}: {error}");
        if let Some(phase) = case["expect"]["phase_after"].as_str() {
            assert_eq!(phase_name(controller.phase()), phase, "{id}");
            assert!(controller.document().entities.is_empty(), "{id}");
        }
    }
}

#[test]
fn q1_to_q3_and_r1_to_r4_repeat_runs_the_last_repeatable_command() {
    let document = fixture("OM9-L3-01-02-009", "positive.json");

    // Q1: LINE, then an empty Enter repeats it in a fresh session.
    let case = &document["positive_cases"][0];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([10., 0., 0.]).unwrap();
    assert_eq!(
        controller.repeat_slot(),
        Some(case["expect"]["repeat_slot"].as_str().unwrap())
    );
    controller.submit("").unwrap();
    assert_eq!(
        phase_name(controller.phase()),
        case["expect"]["phase"].as_str().unwrap()
    );
    assert_eq!(controller.document().entities.len(), 1);
    // The repeat started a NEW session against the context of now.
    controller.accept_world_point([0., 5., 0.]).unwrap();
    controller.accept_world_point([10., 5., 0.]).unwrap();
    assert_eq!(controller.document().entities.len(), 2);

    // Q2: POINT repeats too.
    let case = &document["positive_cases"][1];
    let mut controller = new_controller();
    controller.dispatch(POINT, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([1., 1., 1.]).unwrap();
    controller.submit("").unwrap();
    controller.accept_world_point([2., 2., 2.]).unwrap();
    assert_eq!(
        controller.document().entities.len() as u64,
        case["expect"]["entities"].as_u64().unwrap()
    );
    assert_eq!(phase_name(controller.phase()), "Idle");

    // Q3: the recent list is bounded and names the completed command. It is a
    // fresh controller so the completed command is the LINE the fixture names.
    let case = &document["positive_cases"][2];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([10., 0., 0.]).unwrap();
    assert!(
        controller.recent().len()
            <= case["expect"]["recent_len_at_most"].as_u64().unwrap() as usize
    );
    assert!(
        controller
            .recent()
            .iter()
            .any(|feature| feature == case["expect"]["recent_contains"].as_str().unwrap()),
        "{:?}",
        controller.recent()
    );

    // R1/R2: only the interactive geometry commands are repeatable.
    for case in document["negative_cases"]
        .as_array()
        .unwrap()
        .iter()
        .take(2)
    {
        let expected = case["expect"]["is_repeatable"].as_bool().unwrap();
        for feature in case["features"].as_array().unwrap() {
            let feature = feature.as_str().unwrap();
            assert_eq!(
                Controller::is_repeatable(feature),
                expected,
                "{}: {feature}",
                case["id"].as_str().unwrap()
            );
        }
    }

    // R3: the repeat slot never holds a file feature.
    let case = &document["negative_cases"][2];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([5., 0., 0.]).unwrap();
    let _ = controller.dispatch("New", InvocationOrigin::Cmd);
    assert_eq!(
        controller.repeat_slot(),
        Some(case["expect"]["repeat_slot"].as_str().unwrap())
    );
    assert!(
        controller
            .repeatable_last()
            .map(Controller::is_repeatable)
            .unwrap_or(false)
    );

    // R4: an empty Enter with nothing to repeat says so.
    let case = &document["negative_cases"][3];
    let mut controller = new_controller();
    let error = controller.submit("").expect_err("nothing to repeat");
    assert_eq!(
        match &error {
            om9_core::Error::Missing(_) => "Missing",
            other => panic!("unexpected {other}"),
        },
        case["expect"]["error"].as_str().unwrap()
    );
}

#[test]
fn s1_to_s3_a_destructive_command_is_never_repeated() {
    let document = fixture("OM9-L3-01-02-009", "negative.json");

    // S1: no file feature may ever enter the repeat slot.
    let case = &document["cases"][0];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    controller.accept_world_point([0., 0., 0.]).unwrap();
    controller.accept_world_point([1., 0., 0.]).unwrap();
    assert!(!(Controller::is_repeatable("Open")));
    assert!(!(Controller::is_repeatable("SaveAs")));
    assert!(controller.repeatable_last().is_some());
    assert_eq!(case["expect"]["refused"], true);

    // S2: an empty Enter while a command runs does not start a second one.
    let case = &document["cases"][1];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    let error = controller.submit("").expect_err("a point is required");
    assert_eq!(
        match &error {
            om9_core::Error::Invalid(_) => "Invalid",
            om9_core::Error::Busy => "Busy",
            other => panic!("unexpected {other}"),
        },
        "Invalid",
        "{}",
        case["id"].as_str().unwrap()
    );

    // S3: a session whose context is gone is not repeated.
    let case = &document["cases"][2];
    let mut controller = new_controller();
    controller.dispatch(LINE, InvocationOrigin::Cmd).unwrap();
    // The stale trigger is the LOSS of the view the run started in: a mere
    // view-tab switch keeps a running command (chrome_actions).
    let started = controller.context();
    controller.workspace.views.remove(&started.view);
    assert!(!controller.is_running());
    assert_eq!(
        phase_name(controller.phase()),
        case["expect"]["phase"].as_str().unwrap()
    );
}

#[test]
fn fixture_design_pairs_are_present() {
    for l3 in ["OM9-L3-01-02-005", "OM9-L3-01-02-009"] {
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

/// A bare number follows the document unit (Matrix behaviour); an explicit suffix
/// always wins (OM9-L3-01-08-001). The command layer shares the single unit table
/// with the typed numeric layer, so it cannot drift.
#[test]
fn a_bare_option_number_follows_the_document_unit() {
    let descriptor = om9_app::session::Controller::line_descriptor();
    for (unit, expected_mm) in [
        (Unit::Millimeter, 2.5),
        (Unit::Centimeter, 25.0),
        (Unit::Inch, 63.5),
    ] {
        let parsed = parse_option(unit, &descriptor, "Length=2.5")
            .unwrap()
            .expect("Length is an option");
        assert_eq!(parsed.name, "Length");
        assert!(
            (parsed.value.as_mm().expect("length") - expected_mm).abs() < 1e-9,
            "{unit:?}: {:?}",
            parsed.value.as_mm()
        );
    }
    // An explicit suffix is independent of the document unit.
    let parsed = parse_option(Unit::Inch, &descriptor, "Length=2.5mm")
        .unwrap()
        .expect("Length is an option");
    assert!((parsed.value.as_mm().unwrap() - 2.5).abs() < 1e-9);
    // A suffix outside the shared unit table is still refused, with the same label.
    assert_eq!(
        parse_option(Unit::Millimeter, &descriptor, "Length=2.5kg"),
        Err(om9_core::Error::Unsupported("option unit"))
    );
}
