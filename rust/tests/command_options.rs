use openmatrix9_rust::command_options::*;
use std::collections::BTreeMap;

fn session() -> OptionSession {
    let mut specs = vec![
        OptionSpec::toggle("Flip").alias("F"),
        OptionSpec::choice(
            "Mode",
            vec![
                Choice::new("Line"),
                Choice::new("Arc").disabled("Arc is not implemented"),
            ],
        )
        .alias("M"),
        OptionSpec::number("Length", 1e-7, 1e9, "mm", false).alias("L"),
        OptionSpec::choice(
            "Sections",
            vec![
                Choice::new("None"),
                Choice::new("Rebuild").child("PointCount"),
                Choice::new("Refit").child("Tolerance"),
            ],
        ),
        OptionSpec::number("PointCount", 2., 256., "", true).parent("Sections"),
        OptionSpec::number("Tolerance", 1e-7, 1e6, "mm", false).parent("Sections"),
    ];
    specs.push(OptionSpec::action("Undo"));
    let values = BTreeMap::from([
        ("Flip".into(), OptionValue::Toggle(false)),
        ("Mode".into(), OptionValue::Choice("Line".into())),
        ("Length".into(), OptionValue::Number(5.)),
        ("Sections".into(), OptionValue::Choice("None".into())),
        ("PointCount".into(), OptionValue::Number(16.)),
        ("Tolerance".into(), OptionValue::Number(0.01)),
    ]);
    OptionSession::new(specs, values, OptionContext::new(42, 1.)).unwrap()
}
fn receive(s: &mut OptionSession, event: OptionEvent) {
    if let OptionOutcome::Candidate(c) = s.dispatch(event).unwrap() {
        s.accept(c).unwrap();
    }
}
#[test]
fn toggle_click_once_explicit_set_idempotent() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Flip".into()));
    assert_eq!(s.values()["Flip"], OptionValue::Toggle(true));
    receive(&mut s, OptionEvent::Submit("F=Yes".into()));
    receive(&mut s, OptionEvent::Submit("Flip=On".into()));
    assert_eq!(s.values()["Flip"], OptionValue::Toggle(true));
    receive(&mut s, OptionEvent::Activate("Flip".into()));
    assert_eq!(s.values()["Flip"], OptionValue::Toggle(false));
}
#[test]
fn choice_nested_escape_keeps_accepted() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Sections".into()));
    receive(&mut s, OptionEvent::Submit("Rebuild".into()));
    assert_eq!(s.pending_key(), Some("PointCount"));
    assert_eq!(s.values()["Sections"], OptionValue::Choice("None".into()));
    receive(&mut s, OptionEvent::Escape);
    assert_eq!(s.pending_key(), Some("Sections"));
    receive(&mut s, OptionEvent::Submit("Refit".into()));
    receive(&mut s, OptionEvent::Submit("0.25mm".into()));
    assert_eq!(s.values()["Sections"], OptionValue::Choice("Refit".into()));
    assert_eq!(s.values()["Tolerance"], OptionValue::Number(0.25));
    assert_eq!(s.pending_key(), None);
}
#[test]
fn choice_collision_and_disabled_rejected() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Mode".into()));
    assert!(s.dispatch(OptionEvent::Submit("Arc".into())).is_err());
    assert!(s.dispatch(OptionEvent::Submit(String::new())).is_err());
    assert_eq!(s.values()["Mode"], OptionValue::Choice("Line".into()));
    receive(&mut s, OptionEvent::Select(1));
    receive(&mut s, OptionEvent::Submit(String::new()));
    assert_eq!(s.values()["Mode"], OptionValue::Choice("Line".into()));
    let schema = vec![
        OptionSpec::toggle("Foo").alias("F"),
        OptionSpec::toggle("Far").alias("F"),
    ];
    let mut ambiguous = OptionSession::new(
        schema,
        BTreeMap::from([
            ("Foo".into(), OptionValue::Toggle(false)),
            ("Far".into(), OptionValue::Toggle(false)),
        ]),
        OptionContext::new(7, 1.),
    )
    .unwrap();
    assert!(ambiguous.dispatch(OptionEvent::Submit("f".into())).is_err());
    receive(&mut ambiguous, OptionEvent::Submit("Foo=Yes".into()));
    assert_eq!(ambiguous.values()["Foo"], OptionValue::Toggle(true));
}
#[test]
fn number_enter_does_not_finish() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Length".into()));
    receive(&mut s, OptionEvent::Submit("2cm".into()));
    assert_eq!(s.values()["Length"], OptionValue::Number(20.));
    assert_eq!(s.pending_key(), None);
    assert!(matches!(
        s.dispatch(OptionEvent::Submit(String::new())).unwrap(),
        OptionOutcome::Unhandled
    ));
}
#[test]
fn invalid_number_preserves_values_and_draft() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Length".into()));
    for raw in ["NaN", "inf", "-2", "1e20", "3degrees", "junk"] {
        assert!(s.dispatch(OptionEvent::Submit(raw.into())).is_err());
        assert_eq!(s.values()["Length"], OptionValue::Number(5.));
        assert_eq!(s.pending_key(), Some("Length"));
        assert_eq!(s.draft(), raw);
    }
    s.set_draft("12".into());
    assert!(
        s.dispatch(OptionEvent::Set("Length".into(), OptionValue::Number(3.)))
            .is_err()
    );
    receive(&mut s, OptionEvent::Escape);
    receive(
        &mut s,
        OptionEvent::Set("Length".into(), OptionValue::Number(3.)),
    );
    assert_eq!(s.values()["Length"], OptionValue::Number(3.));
}
#[test]
fn generation_and_revision_reject_stale_candidate() {
    let mut s = session();
    let OptionOutcome::Candidate(old) = s.dispatch(OptionEvent::Submit("Flip=Yes".into())).unwrap()
    else {
        panic!()
    };
    receive(&mut s, OptionEvent::Submit("Length=3".into()));
    assert!(s.accept(old).is_err());
    let OptionOutcome::Candidate(mut wrong) =
        s.dispatch(OptionEvent::Submit("Flip=Yes".into())).unwrap()
    else {
        panic!()
    };
    wrong.generation = 99;
    assert!(s.accept(wrong).is_err());
    assert_eq!(s.values()["Flip"], OptionValue::Toggle(false));
}
#[test]
fn native_rejection_leaves_values_and_choice_phase_intact() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Sections".into()));
    receive(&mut s, OptionEvent::Submit("Rebuild".into()));
    let OptionOutcome::Candidate(c) = s.dispatch(OptionEvent::Submit("32".into())).unwrap() else {
        panic!()
    };
    drop(c); // Native construction rejected it: caller never accepts.
    assert_eq!(s.values()["PointCount"], OptionValue::Number(16.));
    assert_eq!(s.pending_key(), Some("PointCount"));
    receive(&mut s, OptionEvent::Submit("24".into()));
    assert_eq!(s.values()["PointCount"], OptionValue::Number(24.));
    assert_eq!(
        s.values()["Sections"],
        OptionValue::Choice("Rebuild".into())
    );
}
#[test]
fn builder_only_key_is_not_a_cmd_alias() {
    let mut s = OptionSession::new(
        vec![OptionSpec::toggle("Private").scope(2)],
        BTreeMap::from([("Private".into(), OptionValue::Toggle(false))]),
        OptionContext::new(8, 1.),
    )
    .unwrap();
    assert!(s.dispatch(OptionEvent::Activate("Private".into())).is_err());
    assert!(
        s.dispatch(OptionEvent::Submit("Private=Yes".into()))
            .is_err()
    );
    receive(
        &mut s,
        OptionEvent::Set("Private".into(), OptionValue::Toggle(true)),
    );
    assert_eq!(s.values()["Private"], OptionValue::Toggle(true));
}
#[test]
fn unrelated_builder_update_does_not_accept_staged_cmd_choice() {
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Sections".into()));
    receive(&mut s, OptionEvent::Submit("Rebuild".into()));
    s.set_draft("24".into());
    receive(
        &mut s,
        OptionEvent::Set("Flip".into(), OptionValue::Toggle(true)),
    );
    assert_eq!(s.values()["Sections"], OptionValue::Choice("None".into()));
    assert_eq!(s.pending_key(), Some("PointCount"));
    assert_eq!(s.draft(), "24");
    receive(&mut s, OptionEvent::Submit("24".into()));
    assert_eq!(
        s.values()["Sections"],
        OptionValue::Choice("Rebuild".into())
    );
}
#[test]
fn common_engine_does_not_invent_geometry_rules_from_parameter_names() {
    let mut s = OptionSession::new(
        vec![
            OptionSpec::choice("Style", vec![Choice::new("Developable")]),
            OptionSpec::toggle("Closed"),
        ],
        BTreeMap::from([
            ("Style".into(), OptionValue::Choice("Developable".into())),
            ("Closed".into(), OptionValue::Toggle(false)),
        ]),
        OptionContext::new(77, 1.),
    )
    .unwrap();
    receive(
        &mut s,
        OptionEvent::Set("Closed".into(), OptionValue::Toggle(true)),
    );
    assert_eq!(s.values()["Closed"], OptionValue::Toggle(true));
}
#[test]
fn missing_toggle_current_is_not_invented_and_named_input_leaves_numeric_phase() {
    let mut empty = OptionSession::new(
        vec![OptionSpec::toggle("Reverse")],
        BTreeMap::new(),
        OptionContext::new(9, 1.),
    )
    .unwrap();
    assert!(
        empty
            .dispatch(OptionEvent::Activate("Reverse".into()))
            .is_err()
    );
    receive(&mut empty, OptionEvent::Submit("Reverse=No".into()));
    receive(&mut empty, OptionEvent::Activate("Reverse".into()));
    assert_eq!(empty.values()["Reverse"], OptionValue::Toggle(true));
    let mut s = session();
    receive(&mut s, OptionEvent::Activate("Length".into()));
    receive(&mut s, OptionEvent::Submit("Mode".into()));
    assert_eq!(s.pending_key(), Some("Mode"));
    receive(&mut s, OptionEvent::Submit("Esc".into()));
    assert_eq!(s.pending_key(), None);
}
#[test]
fn explicit_assignment_draft_blocks_same_builder_parameter() {
    let mut s = session();
    s.set_draft("Length=12".into());
    assert!(
        s.dispatch(OptionEvent::Set("Length".into(), OptionValue::Number(3.)))
            .is_err()
    );
    receive(
        &mut s,
        OptionEvent::Set("Flip".into(), OptionValue::Toggle(true)),
    );
    assert_eq!(s.draft(), "Length=12");
}
#[test]
fn malformed_candidate_cannot_drop_or_silently_replace_accepted_values() {
    let mut s = session();
    let OptionOutcome::Candidate(mut c) =
        s.dispatch(OptionEvent::Submit("Flip=Yes".into())).unwrap()
    else {
        panic!()
    };
    c.values.remove("Length");
    assert!(s.accept(c).is_err());
    let OptionOutcome::Candidate(mut c) =
        s.dispatch(OptionEvent::Submit("Flip=Yes".into())).unwrap()
    else {
        panic!()
    };
    c.values.insert("Length".into(), OptionValue::Number(3.));
    assert!(s.accept(c).is_err());
}
#[test]
fn active_child_is_rendered_and_malformed_child_graph_is_rejected() {
    let mut s = session();
    receive(&mut s, OptionEvent::Submit("Sections=Rebuild".into()));
    receive(&mut s, OptionEvent::Submit("24".into()));
    assert!(s.snapshot().items.iter().any(|i| i.key == "PointCount"));
    assert!(!s.snapshot().items.iter().any(|i| i.key == "Tolerance"));
    let bad = vec![
        OptionSpec::choice("A", vec![Choice::new("B").child("B")]),
        OptionSpec::choice("B", vec![Choice::new("A").child("A")]).parent("A"),
    ];
    assert!(OptionSession::new(bad, BTreeMap::new(), OptionContext::new(10, 1.)).is_err());
}
#[test]
fn curve_provider_reports_actual_defaults_and_undo_not_schema_guesses() {
    use openmatrix9_rust::{command_option_schemas::curve_state, curve::CurveSession};
    let mut c = CurveSession::default();
    c.start("Circle").unwrap();
    assert_eq!(
        curve_state(&c).unwrap().1["PointCount"],
        OptionValue::Number(8.)
    );
    c.input("Deformable=Yes").unwrap();
    assert_eq!(
        curve_state(&c).unwrap().1["Deformable"],
        OptionValue::Toggle(true)
    );
    c.input("Undo").unwrap();
    assert_eq!(
        curve_state(&c).unwrap().1["Deformable"],
        OptionValue::Toggle(false)
    );
    c.start("InterpCrv").unwrap();
    assert_eq!(
        curve_state(&c).unwrap().1["Knots"],
        OptionValue::Choice("Uniform".into())
    );
}
#[test]
fn cmd_only_option_cannot_be_set_by_builder_and_raw_text_uses_same_parser() {
    let mut s = OptionSession::new(
        vec![OptionSpec::toggle("Private").scope(1)],
        BTreeMap::from([("Private".into(), OptionValue::Toggle(false))]),
        OptionContext::new(81, 1.),
    )
    .unwrap();
    assert!(
        s.dispatch(OptionEvent::Set(
            "Private".into(),
            OptionValue::Toggle(true)
        ))
        .is_err()
    );
    let mut s = session();
    receive(&mut s, OptionEvent::SetText("Length".into(), "2cm".into()));
    assert_eq!(s.values()["Length"], OptionValue::Number(20.));
    for raw in ["NaN", "-2", "1e20", "3degrees"] {
        assert!(
            s.dispatch(OptionEvent::SetText("Length".into(), raw.into()))
                .is_err()
        );
        assert_eq!(s.values()["Length"], OptionValue::Number(20.));
    }
}
#[test]
fn builder_numeric_quantity_uses_declared_unit_while_cmd_uses_document_units() {
    let mut s = OptionSession::new(
        vec![OptionSpec::number("Distance", 0.01, 1000., "mm", false)],
        BTreeMap::new(),
        OptionContext::new(83, 10.),
    )
    .unwrap();
    receive(&mut s, OptionEvent::SetText("Distance".into(), "2".into()));
    assert_eq!(s.values()["Distance"], OptionValue::Number(2.));
    receive(&mut s, OptionEvent::Submit("Distance=2".into()));
    assert_eq!(s.values()["Distance"], OptionValue::Number(20.));
    receive(
        &mut s,
        OptionEvent::SetText("Distance".into(), "2cm".into()),
    );
    assert_eq!(s.values()["Distance"], OptionValue::Number(20.));
}
#[test]
fn geometric_size_assignment_is_native_input_not_a_parameter_candidate() {
    use openmatrix9_rust::command_option_schemas;
    let (schema, values) = command_option_schemas::schema(5).unwrap();
    let mut s = OptionSession::new(schema, values, OptionContext::new(85, 1.)).unwrap();
    assert!(
        matches!(s.dispatch(OptionEvent::Submit("Radius=5".into())).unwrap(),OptionOutcome::Action(text) if text=="Radius=5")
    );
    assert!(s.dispatch(OptionEvent::Submit("Undo=2".into())).is_err());
}

#[test]
fn input_phase_changes_invalidate_unaccepted_candidates() {
    for event in [
        OptionEvent::Escape,
        OptionEvent::Activate("Mode".into()),
        OptionEvent::Select(1),
        OptionEvent::Submit("Flip=No".into()),
    ] {
        let mut s = session();
        receive(&mut s, OptionEvent::Activate("Mode".into()));
        let OptionOutcome::Candidate(old) = s.dispatch(OptionEvent::Submit("Line".into())).unwrap()
        else {
            panic!()
        };
        s.dispatch(event).unwrap();
        assert!(
            s.accept(old).is_err(),
            "phase/input event must retire older proposal"
        );
    }
    let mut s = session();
    receive(&mut s, OptionEvent::Submit("Sections=Rebuild".into()));
    let OptionOutcome::Candidate(old) = s.dispatch(OptionEvent::Submit("24".into())).unwrap()
    else {
        panic!()
    };
    receive(&mut s, OptionEvent::Escape);
    assert!(s.accept(old).is_err());
    assert_eq!(s.values()["Sections"], OptionValue::Choice("None".into()));
    let OptionOutcome::Candidate(old) = s.dispatch(OptionEvent::Submit("Flip=Yes".into())).unwrap()
    else {
        panic!()
    };
    s.set_draft("Flip=No".into());
    assert!(s.accept(old).is_err());
    assert_eq!(s.draft(), "Flip=No");
    assert_eq!(s.values()["Flip"], OptionValue::Toggle(false));
}

#[test]
fn unrelated_cmd_proposals_preserve_unfinished_branch() {
    for event in [
        OptionEvent::Submit("Flip=Yes".into()),
        OptionEvent::Submit("Flip".into()),
        OptionEvent::Submit("Mode=Line".into()),
    ] {
        let mut s = session();
        receive(&mut s, OptionEvent::Submit("Sections=Rebuild".into()));
        receive(&mut s, event);
        assert_eq!(s.pending_key(), Some("PointCount"));
        assert_eq!(s.values()["Sections"], OptionValue::Choice("None".into()));
        assert_eq!(s.values()["PointCount"], OptionValue::Number(16.));
        receive(&mut s, OptionEvent::Submit("24".into()));
        assert_eq!(
            s.values()["Sections"],
            OptionValue::Choice("Rebuild".into())
        );
        assert_eq!(s.values()["PointCount"], OptionValue::Number(24.));
    }
}

#[test]
fn pending_choice_names_take_precedence_over_global_options() {
    let mut s = OptionSession::new(
        vec![
            OptionSpec::choice("Mode", vec![Choice::new("Other"), Choice::new("Flip")]),
            OptionSpec::toggle("Flip"),
        ],
        BTreeMap::from([
            ("Mode".into(), OptionValue::Choice("Other".into())),
            ("Flip".into(), OptionValue::Toggle(false)),
        ]),
        OptionContext::new(91, 1.),
    )
    .unwrap();
    receive(&mut s, OptionEvent::Activate("Mode".into()));
    receive(&mut s, OptionEvent::Submit("fLiP".into()));
    assert_eq!(s.values()["Mode"], OptionValue::Choice("Flip".into()));
    assert_eq!(s.values()["Flip"], OptionValue::Toggle(false));
}

#[test]
fn case_only_canonical_key_collisions_are_rejected() {
    assert!(
        OptionSession::new(
            vec![OptionSpec::toggle("Flip"), OptionSpec::toggle("flip")],
            BTreeMap::new(),
            OptionContext::new(92, 1.),
        )
        .is_err()
    );
}

#[test]
fn builder_only_child_cannot_be_traversed_or_selected_in_cmd() {
    let mut s = OptionSession::new(
        vec![
            OptionSpec::choice(
                "Mode",
                vec![
                    Choice::new("Normal"),
                    Choice::new("Custom").child("Private"),
                ],
            ),
            OptionSpec::number("Private", 2., 256., "", true)
                .parent("Mode")
                .scope(2),
        ],
        BTreeMap::from([
            ("Mode".into(), OptionValue::Choice("Normal".into())),
            ("Private".into(), OptionValue::Number(16.)),
        ]),
        OptionContext::new(93, 1.),
    )
    .unwrap();
    assert!(
        s.dispatch(OptionEvent::Submit("Mode=Custom".into()))
            .is_err()
    );
    assert_eq!(s.pending_key(), None);
    receive(&mut s, OptionEvent::Activate("Mode".into()));
    assert!(
        !s.snapshot()
            .items
            .iter()
            .find(|i| i.input == "Mode=Custom")
            .unwrap()
            .enabled
    );
    receive(&mut s, OptionEvent::Select(-1));
    receive(&mut s, OptionEvent::Submit(String::new()));
    assert_eq!(s.values()["Mode"], OptionValue::Choice("Normal".into()));
    receive(
        &mut s,
        OptionEvent::Set("Mode".into(), OptionValue::Choice("Custom".into())),
    );
    receive(
        &mut s,
        OptionEvent::Set("Private".into(), OptionValue::Number(24.)),
    );
    assert_eq!(s.values()["Private"], OptionValue::Number(24.));
}
