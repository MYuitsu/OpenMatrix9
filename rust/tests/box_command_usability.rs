use openmatrix9_rust::{
    coordinates::{Frame, PointInput},
    solid::{Effect, Kind, Session},
};

#[test]
fn shared_curve_origin_shortcut_keeps_original_world_input_evidence() {
    use openmatrix9_rust::curve::CurveSession;
    let mut line = CurveSession::default();
    line.start("Line").unwrap();
    line.set_frame([12., 34., 56.], [[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]])
        .unwrap();
    line.input("0").unwrap();
    line.input("1,0").unwrap();
    let raw = line.construction_evidence(702).unwrap();
    assert!(raw.contains("\"resolved_point_world\":[12,34,56]"), "{raw}");
}

#[test]
fn box_shortcut_characters_are_real_single_key_aliases_and_owned_metadata() {
    use openmatrix9_rust::{command_options::*, command_options_ffi::*};
    let mut options = Session::new(Kind::Box).command_options(703, 1.).unwrap();
    assert!(
        matches!(options.dispatch(OptionEvent::Submit("3".into())).unwrap(),OptionOutcome::Action(key) if key=="3Point")
    );
    let mut box_session = Session::new(Kind::Box);
    box_session.input("3Point").unwrap();
    box_session.input("0").unwrap();
    let mut later = box_session.command_options(703, 1.).unwrap();
    assert!(matches!(
        later.dispatch(OptionEvent::Submit("3".into())).unwrap(),
        OptionOutcome::Unhandled
    ));
    let handle = om9_options_open(14, 1.);
    assert_ne!(handle, 0);
    for (index, expected) in ["D", "3", "V", "C"].iter().enumerate() {
        let mut buffer = [0u8; 16];
        let count =
            unsafe { om9_options_item_text(handle, index, 3, buffer.as_mut_ptr(), buffer.len()) };
        assert_eq!(&buffer[..count - 1], expected.as_bytes());
        assert_eq!(buffer[count - 1], 0);
    }
    om9_options_close(handle);
}

#[test]
fn origin_shortcut_uses_the_common_point_parser_and_current_cplane() {
    let axes = [[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]];
    let frame = Frame::new([12., 34., 56.], axes).unwrap();
    let point = PointInput::parse("0", |s| s.parse::<f64>().map_err(|e| e.to_string())).unwrap();
    assert_eq!(point.original, "0");
    assert_eq!(point.resolve(frame, None).unwrap().world, [12., 34., 56.]);
    assert!(PointInput::parse("10", |s| s.parse::<f64>().map_err(|e| e.to_string())).is_err());
}

#[test]
fn box_accepts_zero_as_first_point_and_retains_numeric_zero_validation() {
    let mut session = Session::new(Kind::Box);
    session.input("Center").unwrap();
    assert_eq!(session.input("0").unwrap(), Effect::Waiting);
    assert_eq!(session.phase(), 2);
    assert!(session.input("0").is_err());
    assert_eq!(session.phase(), 2);
    session.input("10").unwrap();
    assert!(session.input("0").is_err());
    assert_eq!(session.phase(), 3);
    session.input("20").unwrap();
    assert!(session.input("0").is_err());
    assert_eq!(session.phase(), 4);
    assert_eq!(session.input("10").unwrap(), Effect::Commit);
    let output = session.output().unwrap();
    assert_eq!(&output[..3], &[-5., -10., 0.]);
    assert_eq!(&output[12..], &[10., 20., 10.]);
}

#[test]
fn box_point_only_diagonal_accepts_origin_as_opposite_corner() {
    let mut session = Session::new(Kind::Box);
    session.input("Diagonal").unwrap();
    session.input("2,3").unwrap();
    assert_eq!(session.input("0").unwrap(), Effect::Waiting);
    assert_eq!(session.phase(), 4);
    assert_eq!(session.input("4").unwrap(), Effect::Commit);
    assert_eq!(&session.output().unwrap()[12..], &[2., 3., 4.]);
}

#[test]
fn box_prompt_explains_enter_defaults_and_errors_without_internal_notation() {
    let mut session = Session::new(Kind::Box);
    session.input("Center").unwrap();
    session.input("0,0").unwrap();
    session.input("10").unwrap();
    let width = session.command_options(701, 1.).unwrap().snapshot().text;
    assert!(
        width.contains("Press Enter to use length") && width.contains("10"),
        "{width}"
    );
    let zero = session.input("0").unwrap_err();
    assert!(
        zero.contains("Width") && zero.to_lowercase().contains("zero"),
        "{zero}"
    );
    assert!(!zero.contains("1e9"));
    let large = session.input("1e20").unwrap_err();
    assert!(
        large.to_lowercase().contains("smaller") && !large.contains("1e9"),
        "{large}"
    );
    session.input("").unwrap();
    let height = session.command_options(701, 1.).unwrap().snapshot().text;
    assert!(
        height.contains("Press Enter to use width") && height.contains("10"),
        "{height}"
    );
}
