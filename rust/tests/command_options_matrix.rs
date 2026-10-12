use openmatrix9_rust::command_options::*;
use openmatrix9_rust::solid::{Effect, Kind, Session};
use std::collections::BTreeMap;

#[test]
fn common_question_layout_preserves_utf16_click_spans_and_hides_private_actions() {
    let mut options = OptionSession::new(
        vec![
            OptionSpec::action("Diagonal").alias("D"),
            OptionSpec::action("Cube").hidden(),
        ],
        BTreeMap::new(),
        OptionContext::new(501, 1.),
    )
    .unwrap();
    options.set_question("Điểm đầu 🔷").unwrap();
    let snapshot = options.snapshot();
    assert_eq!(snapshot.text, "Điểm đầu 🔷 ( Diagonal ): ");
    assert_eq!(
        snapshot.question_length,
        "Điểm đầu 🔷".encode_utf16().count()
    );
    assert_eq!(snapshot.items.len(), 1);
    assert_eq!(
        snapshot.items[0].start,
        "Điểm đầu 🔷 ( ".encode_utf16().count()
    );
    assert!(
        matches!(options.dispatch(OptionEvent::Submit("d".into())).unwrap(),OptionOutcome::Action(name) if name=="Diagonal")
    );
    assert!(
        matches!(options.dispatch(OptionEvent::Submit("Cube".into())).unwrap(),OptionOutcome::Action(name) if name=="Cube")
    );
}

#[test]
fn box_phase_options_use_common_engine_and_native_defaults() {
    let mut solid = Session::new(Kind::Box);
    let mut options = solid.command_options(502, 1.).unwrap();
    assert_eq!(
        options.snapshot().text,
        "First corner of base ( Diagonal 3Point Vertical Center ): "
    );
    assert_eq!(
        options
            .snapshot()
            .items
            .iter()
            .map(|i| i.key.as_str())
            .collect::<Vec<_>>(),
        vec!["Diagonal", "3Point", "Vertical", "Center"]
    );
    let OptionOutcome::Action(name) = options.dispatch(OptionEvent::Submit("D".into())).unwrap()
    else {
        panic!()
    };
    assert_eq!(solid.input(&name).unwrap(), Effect::Waiting);
    solid.input("0,0").unwrap();
    let mut options = solid.command_options(502, 1.).unwrap();
    assert_eq!(options.snapshot().text, "Other corner of box ( Cube ): ");
    assert!(options.dispatch(OptionEvent::Submit("D".into())).is_err());
    assert!(
        matches!(options.dispatch(OptionEvent::Submit("c".into())).unwrap(),OptionOutcome::Action(name) if name=="Cube")
    );
    let OptionOutcome::Action(name) = options
        .dispatch(OptionEvent::Activate("Cube".into()))
        .unwrap()
    else {
        panic!()
    };
    solid.input(&name).unwrap();
    assert_eq!(solid.input("3,3,3").unwrap(), Effect::Commit);
    assert!(
        solid.output().unwrap()[12..]
            .iter()
            .all(|v| (v - 3.).abs() < 1e-7)
    );
    let mut solid = Session::new(Kind::Box);
    solid.input("Center").unwrap();
    solid.input("0,0").unwrap();
    solid.input("2").unwrap();
    assert_eq!(
        solid.command_options(503, 1.).unwrap().snapshot().text,
        "Width. Press Enter to use length <2 mm>: "
    );
    solid.input("").unwrap();
    assert_eq!(
        solid.command_options(503, 1.).unwrap().snapshot().text,
        "Height. Press Enter to use width <2 mm>: "
    );
    assert_eq!(solid.input("").unwrap(), Effect::Commit);
    assert_eq!(&solid.output().unwrap()[12..], &[2., 2., 2.]);
}

#[test]
fn command_history_line_is_owned_normalized_and_rejects_multiline() {
    assert_eq!(command_invocation_line("Box").unwrap(), "Command: _Box");
    assert_eq!(command_invocation_line("_Box").unwrap(), "Command: _Box");
    assert!(command_invocation_line("Box\nDelete").is_err());
    assert!(command_invocation_line("").is_err());
}
