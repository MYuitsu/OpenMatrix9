use openmatrix9_rust::{
    command_options::{OptionEvent, OptionOutcome},
    solid::{Effect, Kind, Session},
};

#[test]
fn box_starts_with_default_corner_flow_and_optional_methods() {
    let mut s = Session::new(Kind::Box);
    assert_eq!(
        s.command_options(801, 1.).unwrap().snapshot().text,
        "First corner of base ( Diagonal 3Point Vertical Center ): "
    );
    for input in ["10", "Length=10", "Corners", "Mode=Corners"] {
        assert!(
            s.input(input).is_err(),
            "{input} must not replace the required first point"
        );
        assert_eq!(s.phase(), 1);
        assert!(s.output().is_none());
    }
    assert!(s.preview([1., 2., 3.], false).is_none());
    let mut options = s.command_options(801, 1.).unwrap();
    let OptionOutcome::Action(mode) = options.dispatch(OptionEvent::Submit("3".into())).unwrap()
    else {
        panic!()
    };
    s.input(&mode).unwrap();
    assert!(
        s.command_options(801, 1.)
            .unwrap()
            .snapshot()
            .text
            .starts_with("Start of edge")
    );
    s.input("0").unwrap();
    assert_eq!(s.phase(), 2);
}

#[test]
fn three_point_and_vertical_require_two_edge_points_then_allow_signed_width() {
    for mode in ["3Point", "Vertical"] {
        let mut s = Session::new(Kind::Box);
        s.input(mode).unwrap();
        s.input("0").unwrap();
        assert_eq!(
            s.command_options(802, 1.).unwrap().snapshot().text,
            "End of edge: "
        );
        for scalar in ["10", "10mm", "Length=10", "Length=2cm"] {
            assert!(s.input(scalar).is_err(), "{mode}: {scalar}");
            assert_eq!(s.phase(), 2);
            assert!(s.output().is_none());
        }
        s.input("3,4,0").unwrap();
        assert_eq!(s.phase(), 3);
        s.input("-2").unwrap();
        assert_eq!(s.phase(), 4);
        assert_eq!(s.input("3").unwrap(), Effect::Commit);
        let output = s.output().unwrap();
        assert_eq!(&output[12..], &[5., 2., 3.]);
        if mode == "Vertical" {
            assert_eq!(&output[6..9], &[0., 0., 1.]);
        } else {
            assert!((output[6] + 0.8).abs() < 1e-12 && (output[7] - 0.6).abs() < 1e-12);
        }
    }
}

#[test]
fn center_keeps_full_length_or_picked_corner_semantics() {
    for picked_corner in [false, true] {
        let mut s = Session::new(Kind::Box);
        s.input("Center").unwrap();
        s.input("0,0,0").unwrap();
        if picked_corner {
            s.input("5,10,0").unwrap();
            assert_eq!(s.phase(), 4);
        } else {
            s.input("Length=10").unwrap();
            assert_eq!(s.phase(), 3);
            s.input("20").unwrap();
        }
        assert_eq!(s.input("10").unwrap(), Effect::Commit);
        let output = s.output().unwrap();
        assert_eq!(&output[..3], &[-5., -10., 0.]);
        assert_eq!(&output[12..], &[10., 20., 10.]);
    }
}

#[test]
fn undo_restores_default_corner_flow_after_undoing_an_option_choice() {
    let mut s = Session::new(Kind::Box);
    s.input("3Point").unwrap();
    s.input("0").unwrap();
    s.input("Undo").unwrap();
    assert!(s.prompt().starts_with("Start of edge"));
    s.input("Undo").unwrap();
    assert!(s.prompt().starts_with("First corner of base"));
    assert!(s.input("Mode=Corners").is_err());
    assert!(s.input("0").is_ok());
}

#[test]
fn endpoint_origin_remains_a_point_and_width_height_enter_defaults_remain_valid() {
    let mut s = Session::new(Kind::Box);
    s.input("3Point").unwrap();
    s.input("3,4,0").unwrap();
    s.input("0").unwrap();
    assert_eq!(s.phase(), 3);
    s.input("").unwrap();
    assert_eq!(s.phase(), 4);
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    assert_eq!(&s.output().unwrap()[12..], &[5., 5., 5.]);
}
