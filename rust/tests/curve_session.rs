use openmatrix9_rust::curve::{CurveSession, Effect};
#[test]
fn mouse_cannot_append_a_point_while_length_value_is_pending() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    s.input("0,0").unwrap();
    s.input("Length").unwrap();
    assert!(s.point([10., 20., 0.]).is_err());
    assert_eq!(s.points().len(), 1);
    s.input("5").unwrap();
    s.point([3., 4., 0.]).unwrap();
    s.input("").unwrap();
    assert_eq!(s.output().len(), 2);
}

#[test]
fn polyline_length_option_prompts_for_value_then_direction() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    s.input("0,0,0").unwrap();
    assert_eq!(s.input("Length").unwrap(), Effect::Waiting);
    assert!(s.input("-2").is_err());
    s.input("5mm").unwrap();
    s.input("6,8,0").unwrap();
    s.input("").unwrap();
    assert_eq!(s.output(), &[[0., 0., 0.], [3., 4., 0.]]);
}

#[test]
fn om9_curve_002_line_commits_two_distinct_points() {
    let mut s = CurveSession::default();
    s.start("Line").unwrap();
    assert_eq!(s.input("0,0,0").unwrap(), Effect::Waiting);
    assert_eq!(s.input("3,4,0").unwrap(), Effect::Commit);
    assert_eq!(s.output(), &[[0., 0., 0.], [3., 4., 0.]]);
}
#[test]
fn om9_curve_001_polyline_enter_and_close() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    for p in ["0,0,0", "10,0,0", "10,10,0"] {
        s.input(p).unwrap();
    }
    assert_eq!(s.input("Close").unwrap(), Effect::Commit);
    assert_eq!(s.output().len(), 4);
    assert_eq!(s.output().first(), s.output().last());
    s.start("Polyline").unwrap();
    s.input("0,0,0").unwrap();
    s.input("3,4,0").unwrap();
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    assert_eq!(s.output().len(), 2);
}
#[test]
fn om9_curve_002_both_sides_uses_first_point_as_midpoint() {
    let mut s = CurveSession::default();
    s.start("Line").unwrap();
    s.input("BothSides").unwrap();
    s.input("0,0,0").unwrap();
    s.input("3,4,0").unwrap();
    assert_eq!(s.output(), &[[-3., -4., 0.], [3., 4., 0.]]);
}
#[test]
fn invalid_input_keeps_session_and_no_partial_commit() {
    let mut s = CurveSession::default();
    s.start("Line").unwrap();
    s.input("0,0,0").unwrap();
    for p in [
        "0,0,0",
        "NaN,1,2",
        "1,inf,3",
        "not Python()",
        "1,2,3,4",
        "Mode=Arc",
    ] {
        assert!(s.input(p).is_err());
        assert!(s.active());
        assert!(s.output().is_empty());
    }
    assert_eq!(s.input("3,4,0").unwrap(), Effect::Commit);
}
#[test]
fn cancel_has_no_output_and_restart_clears_options() {
    let mut s = CurveSession::default();
    s.start("Line").unwrap();
    s.input("BothSides").unwrap();
    s.input("0,0,0").unwrap();
    assert_eq!(s.input("Esc").unwrap(), Effect::Cancelled);
    assert!(!s.active());
    assert!(s.output().is_empty());
    s.start("Line").unwrap();
    s.input("0,0,0").unwrap();
    s.input("1,0,0").unwrap();
    assert_eq!(s.output()[0], [0., 0., 0.]);
}
#[test]
fn polyline_short_enter_and_close_do_not_finish() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    s.input("0,0,0").unwrap();
    assert!(s.input("").is_err());
    assert!(s.input("Close").is_err());
    assert!(s.active());
}
#[test]
fn mouse_points_and_typed_coordinates_share_geometry() {
    let mut a = CurveSession::default();
    let mut b = CurveSession::default();
    a.start("Polyline").unwrap();
    b.start("Polyline").unwrap();
    for p in [[0., 0., 0.], [10., 0., 0.], [10., 10., 0.]] {
        a.point(p).unwrap();
        b.input(&format!("{},{},{}", p[0], p[1], p[2])).unwrap();
    }
    a.input("").unwrap();
    b.input("").unwrap();
    assert_eq!(a.output(), b.output());
}
#[test]
fn relative_points_and_units_are_explicitly_parsed() {
    let mut s = CurveSession::default();
    s.start("Line").unwrap();
    s.input("1cm,2cm,0").unwrap();
    s.input("r3mm,4mm,0").unwrap();
    assert_eq!(s.output(), &[[10., 20., 0.], [13., 24., 0.]]);
}
#[test]
fn om9_viewport_001_typed_coordinates_use_active_plane() {
    let mut s = openmatrix9_rust::curve::CurveSession::default();
    s.start("Line").unwrap();
    s.set_frame(
        [0.0; 3],
        [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
    )
    .unwrap();
    s.input("2,3,4").unwrap();
    s.input("r1,2,3").unwrap();
    assert_eq!(s.output(), &[[4.0, 2.0, 3.0], [7.0, 3.0, 5.0]]);
}

#[test]
fn om9_curve_001_preview_keeps_clicks_and_applies_constraints_without_commit() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    for p in [[0., 0., 0.], [3., 4., 0.], [8., 4., 0.]] {
        assert_eq!(s.mouse_point(p, false).unwrap(), Effect::Waiting);
    }
    s.input("Length=5").unwrap();
    assert_eq!(
        s.preview_point([14., 12., 0.], false).unwrap(),
        [11., 8., 0.]
    );
    assert_eq!(s.points().len(), 3);
    assert!(s.output().is_empty());
    assert!(s.active());
    s.mouse_point([14., 12., 0.], false).unwrap();
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    assert_eq!(
        s.output(),
        &[[0., 0., 0.], [3., 4., 0.], [8., 4., 0.], [11., 8., 0.]]
    );
    assert!(s.prompt().starts_with("Command:"));
}

#[test]
fn matrix_prompt_options_and_initial_letter_inputs() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    assert_eq!(s.prompt(), "Start of polyline ( PersistentClose=No ): ");
    s.input("P=Y").unwrap();
    assert!(s.prompt().contains("PersistentClose=Yes"));
    s.input("p=n").unwrap();
    s.input("0,0").unwrap();
    assert!(!s.prompt().contains("Close Mode"));
    assert!(!s.prompt().contains(";"));
    s.input("L").unwrap();
    assert_eq!(s.prompt(), "Length of next segment: ");
    s.input("5").unwrap();
    s.input("3,4").unwrap();
    s.input("10,4").unwrap();
    assert!(s.prompt().contains("Close Mode=Line"));
    s.input("U").unwrap();
    assert_eq!(s.points(), &[[0., 0., 0.], [3., 4., 0.]]);
    assert!(!s.prompt().contains("Close Mode"));
    s.input("M=L").unwrap();
    s.input("10,4").unwrap();
    s.input("C").unwrap();
    assert_eq!(s.output().first(), s.output().last());
    s.start("Line").unwrap();
    s.input("B").unwrap();
    assert!(s.prompt().contains("BothSides=Yes"));
    s.input("0,0").unwrap();
    s.input("1,0").unwrap();
    assert_eq!(s.output(), &[[-1., 0., 0.], [1., 0., 0.]]);
}
#[test]
fn unavailable_initial_letter_does_not_mutate_polyline() {
    let mut s = CurveSession::default();
    s.start("Polyline").unwrap();
    for input in ["C", "U", "L", "M=L", "P=invalid"] {
        assert!(s.input(input).is_err());
        assert!(s.points().is_empty());
    }
    s.input("0,0").unwrap();
    s.input("2,0").unwrap();
    assert!(s.input("C").is_err());
    assert_eq!(s.points().len(), 2);
    s.input("U").unwrap();
    s.input("U").unwrap();
    assert!(s.prompt().starts_with("Start of polyline"));
    assert!(s.input("U").is_err());
    assert!(s.active());
}
