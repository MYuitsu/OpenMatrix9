use openmatrix9_rust::command_option_schemas;
use openmatrix9_rust::command_options::*;
use openmatrix9_rust::solid::{Kind, Session};

fn built_in(family: u32) -> OptionSession {
    let (schema, values) = command_option_schemas::schema(family).unwrap();
    OptionSession::new(schema, values, OptionContext::new(600 + family as u64, 1.)).unwrap()
}

#[test]
fn all_registered_commands_expose_one_working_character_per_visible_option() {
    for family in [1, 2, 3, 4, 5, 6, 7, 11, 12, 13, 14, 15] {
        let options = built_in(family);
        for item in options.snapshot().items.iter().filter(|item| item.enabled) {
            assert_eq!(
                item.mnemonic.chars().count(),
                1,
                "family {family}: {}",
                item.key
            );
            let mut by_shortcut = built_in(family);
            let mut by_name = built_in(family);
            let shortcut = by_shortcut
                .dispatch(OptionEvent::Submit(item.mnemonic.to_ascii_lowercase()))
                .unwrap();
            let name = by_name
                .dispatch(OptionEvent::Submit(item.key.clone()))
                .unwrap();
            assert_eq!(
                format!("{shortcut:?}"),
                format!("{name:?}"),
                "family {family}: {}",
                item.key
            );
        }
    }
}

#[test]
fn suboptions_own_shortcuts_for_their_displayed_choice() {
    for (family, key, input, mnemonic) in [
        (1, "Mode", "Mode=Line", "L"),
        (3, "Knots", "Knots=Chord", "C"),
        (13, "SectionMode", "SectionMode=Rebuild", "b"),
    ] {
        let mut options = built_in(family);
        options.dispatch(OptionEvent::Activate(key.into())).unwrap();
        let item = options
            .snapshot()
            .items
            .into_iter()
            .find(|item| item.input == input)
            .unwrap();
        assert_eq!(
            item.mnemonic.to_ascii_lowercase(),
            mnemonic.to_ascii_lowercase()
        );
        let result = options
            .dispatch(OptionEvent::Submit(mnemonic.to_ascii_lowercase()))
            .unwrap();
        assert!(!matches!(
            result,
            OptionOutcome::Unhandled | OptionOutcome::Action(_)
        ));
        if let OptionOutcome::Candidate(candidate) = result {
            options.accept(candidate).unwrap();
        }
        if family == 13 {
            assert_eq!(options.pending_key(), Some("PointCount"));
        } else {
            assert_eq!(
                options.values()[key],
                OptionValue::Choice(input.split_once('=').unwrap().1.into())
            );
        }
    }
}

#[test]
fn every_enabled_suboption_has_an_unambiguous_shortcut_in_its_parent_phase() {
    for family in [1, 3, 4, 5, 6, 11, 12, 13] {
        let keys: Vec<_> = built_in(family)
            .schema
            .into_iter()
            .filter(|s| s.kind == OptionKind::Choice)
            .map(|s| s.key)
            .collect();
        for key in keys {
            let mut options = built_in(family);
            options
                .dispatch(OptionEvent::Activate(key.clone()))
                .unwrap();
            for item in options.snapshot().items.iter().filter(|i| i.enabled) {
                assert_eq!(
                    item.mnemonic.chars().count(),
                    1,
                    "family {family}: {}",
                    item.input
                );
                let mut shortcut = built_in(family);
                let mut full = built_in(family);
                shortcut
                    .dispatch(OptionEvent::Activate(key.clone()))
                    .unwrap();
                full.dispatch(OptionEvent::Activate(key.clone())).unwrap();
                assert_eq!(
                    format!(
                        "{:?}",
                        shortcut
                            .dispatch(OptionEvent::Submit(item.mnemonic.to_ascii_lowercase()))
                            .unwrap()
                    ),
                    format!(
                        "{:?}",
                        full.dispatch(OptionEvent::Submit(item.input.clone()))
                            .unwrap()
                    ),
                    "family {family}: {}",
                    item.input
                );
            }
        }
    }
}

#[test]
fn sphere_uses_common_phase_mnemonics_without_stealing_radius_numbers() {
    let mut sphere = Session::new(Kind::Sphere);
    let mut options = sphere.command_options(650, 1.).unwrap();
    assert!(
        options
            .snapshot()
            .items
            .iter()
            .any(|i| i.key == "3Point" && i.mnemonic == "3")
    );
    let two = options
        .snapshot()
        .items
        .into_iter()
        .find(|item| item.key == "2Point")
        .unwrap();
    assert_eq!(two.mnemonic, "2");
    let OptionOutcome::Action(name) = options.dispatch(OptionEvent::Submit("2".into())).unwrap()
    else {
        panic!()
    };
    sphere.input(&name).unwrap();
    sphere.input("0,0,0").unwrap();
    let mut options = sphere.command_options(650, 1.).unwrap();
    assert!(!options.snapshot().items.iter().any(|i| i.key == "Diameter"));
    assert!(matches!(
        options.dispatch(OptionEvent::Submit("2".into())).unwrap(),
        OptionOutcome::Unhandled
    ));
    sphere.input("2").unwrap();
    assert_eq!(sphere.output().unwrap()[12], 1.);
    let mut sphere = Session::new(Kind::Sphere);
    sphere.input("0,0,0").unwrap();
    let options = sphere.command_options(650, 1.).unwrap();
    assert!(!options.snapshot().items.iter().any(|i| i.key == "Radius"));
}

#[test]
fn arbitrary_schemas_share_nested_shortcuts_and_do_not_promise_ambiguous_or_disabled_keys() {
    use std::collections::BTreeMap;
    let mut options = OptionSession::new(
        vec![
            OptionSpec::choice(
                "Direction",
                vec![
                    Choice::new("Forward").alias("F"),
                    Choice::new("Reverse").alias("R"),
                    Choice::new("Future").alias("F").disabled("Not implemented"),
                ],
            )
            .alias("D"),
            OptionSpec::number("Distance", 0., 100., "mm", false).alias("I"),
            OptionSpec::toggle("Flip").alias("F"),
            OptionSpec::toggle("Future")
                .alias("F")
                .disabled("Not implemented"),
        ],
        BTreeMap::from([
            ("Direction".into(), OptionValue::Choice("Forward".into())),
            ("Distance".into(), OptionValue::Number(1.)),
            ("Flip".into(), OptionValue::Toggle(false)),
            ("Future".into(), OptionValue::Toggle(false)),
        ]),
        OptionContext::new(651, 1.),
    )
    .unwrap();
    assert!(
        options
            .snapshot()
            .items
            .iter()
            .filter(|i| !i.enabled)
            .all(|i| i.mnemonic.is_empty())
    );
    options.dispatch(OptionEvent::Submit("d".into())).unwrap();
    let snapshot = options.snapshot();
    assert_eq!(snapshot.items[0].mnemonic, "F");
    assert!(snapshot.items[2].mnemonic.is_empty());
    let OptionOutcome::Candidate(candidate) =
        options.dispatch(OptionEvent::Submit("r".into())).unwrap()
    else {
        panic!()
    };
    options.accept(candidate).unwrap();
    assert_eq!(
        options.values()["Direction"],
        OptionValue::Choice("Reverse".into())
    );
    assert_eq!(options.values()["Flip"], OptionValue::Toggle(false));
    options.schema[0]
        .choices
        .push(Choice::new("Right").alias("R"));
    options.dispatch(OptionEvent::Submit("d".into())).unwrap();
    assert!(
        options
            .snapshot()
            .items
            .iter()
            .filter(|i| i.input.ends_with("=Reverse") || i.input.ends_with("=Right"))
            .all(|i| i.mnemonic.is_empty())
    );
    assert!(options.dispatch(OptionEvent::Submit("r".into())).is_err());
    assert_eq!(
        options.values()["Direction"],
        OptionValue::Choice("Reverse".into())
    );
}

#[test]
fn sphere_numeric_phases_do_not_advertise_unsupported_bare_activation() {
    for mode in ["Center", "2Point"] {
        let mut sphere = Session::new(Kind::Sphere);
        sphere.input(mode).unwrap();
        sphere.input("0,0,0").unwrap();
        let mut options = sphere.command_options(652, 1.).unwrap();
        assert!(
            !options
                .snapshot()
                .items
                .iter()
                .any(|i| i.key == "Radius" || i.key == "Diameter")
        );
        assert!(matches!(
            options.dispatch(OptionEvent::Submit("2".into())).unwrap(),
            OptionOutcome::Unhandled
        ));
    }
}
