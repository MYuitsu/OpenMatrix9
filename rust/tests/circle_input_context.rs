//! RCORE-03/04: accepted Circle input, relative base and session checkpoints.
use openmatrix9_rust::curve::{CurveSession, Effect};

fn session() -> CurveSession {
    let mut s = CurveSession::default();
    s.start("Circle").unwrap();
    s
}

fn input(s: &mut CurveSession, values: &[&str]) {
    for value in values {
        s.input(value).unwrap();
    }
}

fn near(actual: [f64; 3], expected: [f64; 3]) {
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(a, b)| (a - b).abs() < 1e-7),
        "{actual:?} != {expected:?}"
    );
}

fn radius_probe(s: &mut CurveSession, mouse: bool) {
    input(s, &["3Point", "5<180", "5<0", "Radius"]);
    let effect = if mouse {
        s.mouse_point([0., 5., 0.], false).unwrap()
    } else {
        s.input("5<90").unwrap()
    };
    assert_eq!(effect, Effect::Waiting);
    assert_eq!(s.points().len(), 2);
}

#[test]
fn typed_and_mouse_radius_probes_are_the_next_relative_base() {
    for mouse in [false, true] {
        let mut s = session();
        radius_probe(&mut s, mouse);
        assert_eq!(s.input("wr5<0<90").unwrap(), Effect::Commit);
        // Chord endpoints are +/-5 on X; radius sqrt(50) gives center
        // distance 5 from the midpoint along the accepted probe's YZ side.
        near(
            s.circle().unwrap().center,
            [0., 3.5355339059327378, 3.5355339059327378],
        );
        near(*s.points().last().unwrap(), [0., 5., 5.]);
    }
}

#[test]
fn typed_and_native_orientation_picks_are_the_next_relative_base() {
    for native in [false, true] {
        let mut s = session();
        input(&mut s, &["2,3,4", "Orientation"]);
        if native {
            s.circle_native_reference([2., 7., 4.], [0.; 3], 4).unwrap();
        } else {
            s.input("2,7,4").unwrap();
        }
        assert_eq!(s.points().len(), 1);
        s.input("wr3,0,0").unwrap();
        near(*s.points().last().unwrap(), [5., 7., 4.]);
        near(s.circle().unwrap().normal, [0., 1., 0.]);
        assert!((s.circle().unwrap().radius - 3.).abs() < 1e-7);
    }
}

#[test]
fn undo_radius_probe_restores_pending_radius_and_previous_point() {
    let mut s = session();
    radius_probe(&mut s, false);
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Pick radius length"));
    assert_eq!(s.input("wr0,10,0").unwrap(), Effect::Waiting);
    s.input("wr0,0,10").unwrap();
    near(*s.points().last().unwrap(), [5., 10., 10.]);
    near(
        s.circle().unwrap().center,
        [0., 6.123724356957945, 6.123724356957945],
    );
    assert!((s.circle().unwrap().radius - 10.).abs() < 1e-7);
}

#[test]
fn undo_radius_value_then_option_then_point_restores_each_input() {
    let mut s = session();
    input(&mut s, &["3Point", "-3,0", "3,0", "Radius", "5"]);
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Pick radius length"));
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Third circumference point"));
    s.input("Undo").unwrap();
    assert_eq!(s.points().len(), 1);
    s.input("wr6,0,0").unwrap();
    s.input("0,3").unwrap();
    near(s.circle().unwrap().center, [0.; 3]);
    assert!((s.circle().unwrap().radius - 3.).abs() < 1e-7);
}

#[test]
fn undo_orientation_pick_then_option_restores_center_base() {
    let mut s = session();
    input(&mut s, &["2,3,4", "Orientation", "2,7,4"]);
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Orientation direction point"));
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Size value"));
    s.input("wr3,0,0").unwrap();
    near(*s.points().last().unwrap(), [5., 3., 4.]);
    near(s.circle().unwrap().normal, [0., 0., 1.]);
}

#[test]
fn pending_option_value_and_errors_have_single_checkpoints() {
    let mut s = session();
    input(&mut s, &["0,0", "Degree", "5"]);
    s.input("Undo").unwrap();
    assert!(s.mouse_point([0., 2., 0.], false).is_err());
    for invalid in ["invalid", "<30", "0,2"] {
        assert!(s.input(invalid).is_err());
        assert_eq!(s.points(), &[[0.; 3]]);
    }
    s.input("Undo").unwrap();
    assert_eq!(s.input("5").unwrap(), Effect::Commit);
    assert_eq!(s.circle().unwrap().radius, 5.);
}

#[test]
fn undo_first_point_removes_relative_base_and_releases_frame_latch() {
    let mut s = session();
    s.set_frame([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    s.input("0,0").unwrap();
    s.input("Undo").unwrap();
    assert!(s.input("wr1,0,0").is_err());
    s.set_frame(
        [100., 200., 300.],
        [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
    )
    .unwrap();
    s.input("0,0").unwrap();
    s.input("5").unwrap();
    near(s.circle().unwrap().center, [100., 200., 300.]);
    near(s.circle().unwrap().normal, [0., 0., 1.]);
}

#[test]
fn rejected_inputs_preserve_radius_context_and_undo_history() {
    let mut s = session();
    radius_probe(&mut s, false);
    let prompt = s.prompt();
    for invalid in [
        "<30",
        "wr1000000001,0,0",
        "Radius=1",
        "Degree=0",
        "nan",
        "Undo=1",
    ] {
        assert!(s.input(invalid).is_err(), "{invalid}");
        assert_eq!(s.prompt(), prompt);
        assert!(s.active());
        assert_eq!(s.points().len(), 2);
        assert!(s.circle().is_none());
        assert!(s.output().is_empty());
    }
    assert!(s.point([f64::NAN, 0., 0.]).is_err());
    assert!(
        s.circle_native_reference([100., 0., 0.], [0.; 3], 3)
            .is_err()
    );
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Pick radius length"));
    s.input("Undo").unwrap();
    s.input("wr0,5,0").unwrap();
    near(*s.points().last().unwrap(), [5., 5., 0.]);
    near(s.circle().unwrap().center, [0., 2.5, 0.]);
}

#[test]
fn native_tangent_reference_and_undo_restore_point_mode_and_base() {
    let mut s = session();
    s.input("Tangent").unwrap();
    s.circle_native_reference([1., 2., 3.], [0.; 3], 3).unwrap();
    s.input("Point").unwrap();
    s.input("wr4,0,0").unwrap();
    near(*s.points().last().unwrap(), [5., 2., 3.]);
    s.input("Undo").unwrap();
    assert_eq!(s.circle_reference_mode(), 0);
    s.input("wr0,4,0").unwrap();
    near(*s.points().last().unwrap(), [1., 6., 3.]);
}

#[test]
fn selecting_native_path_does_not_invent_an_accepted_point() {
    let mut s = session();
    s.input("AroundCurve").unwrap();
    s.circle_native_reference([99., 98., 97.], [0.; 3], 1)
        .unwrap();
    assert!(s.input("wr1,0,0").is_err());
    assert_eq!(s.circle_reference_mode(), 2);
    s.input("Undo").unwrap();
    assert_eq!(s.circle_reference_mode(), 1);
    s.input("Undo").unwrap();
    assert_eq!(s.circle_reference_mode(), 0);
    s.input("0,0").unwrap();
    s.input("2").unwrap();
    near(s.circle().unwrap().center, [0.; 3]);
}

#[test]
fn native_fit_batch_is_one_checkpoint_with_its_last_point_as_base() {
    let mut s = session();
    input(&mut s, &["FitPoints", "0,0"]);
    s.circle_fit_batch(&[[2., 0., 0.], [2., 3., 0.]]).unwrap();
    assert!(s.circle_fit_batch(&[[9.; 3], [f64::NAN, 0., 0.]]).is_err());
    s.input("wr0,0,4").unwrap();
    near(*s.points().last().unwrap(), [2., 3., 4.]);
    s.input("Undo").unwrap();
    assert_eq!(s.points().len(), 3);
    s.input("Undo").unwrap();
    assert_eq!(s.points(), &[[0.; 3]]);
    s.input("wr0,0,5").unwrap();
    near(*s.points().last().unwrap(), [0., 0., 5.]);
}

#[test]
fn previews_do_not_change_accepted_context_or_add_undo_checkpoints() {
    let mut s = session();
    radius_probe(&mut s, false);
    let prompt = s.prompt();
    s.preview_geometry([0., 100., 0.], false).unwrap();
    s.preview_point([0., 0., 100.], false).unwrap();
    s.circle_hover_plan([0., 0., 50.]).unwrap();
    assert_eq!(s.prompt(), prompt);
    assert_eq!(s.points().len(), 2);
    assert!(s.active());
    assert!(s.circle().is_none());
    s.input("Undo").unwrap();
    assert!(s.prompt().contains("Pick radius length"));
}

#[test]
fn checkpoint_budget_keeps_recent_context_without_unbounded_undo() {
    let mut s = session();
    radius_probe(&mut s, false);
    for _ in 0..1100 {
        s.input("History").unwrap();
    }
    let mut undos = 0;
    while s.input("Undo").is_ok() {
        undos += 1;
        assert!(undos <= 1024, "Circle checkpoint storage must stay bounded");
    }
    assert!(undos > 0);
    s.input("wr5<0<90").unwrap();
    near(
        s.circle().unwrap().center,
        [0., 3.5355339059327378, 3.5355339059327378],
    );
}

#[test]
fn tangent_solve_confirmation_does_not_add_an_input_checkpoint() {
    let mut s = session();
    input(&mut s, &["Tangent", "Radius=4"]);
    s.circle_native_reference([0., 4., 0.], [0.; 3], 3).unwrap();
    input(&mut s, &["Point", "4,8,0"]);
    assert_eq!(s.circle_reference_mode(), 5);
    assert_eq!(s.input("").unwrap(), Effect::Waiting);
    assert_eq!(s.input(" \t ").unwrap(), Effect::Waiting);
    s.input("Undo").unwrap();
    assert_eq!(s.circle_reference_mode(), 0);
    assert_eq!(s.points(), &[[0., 4., 0.]]);
    s.input("wr4,0,0").unwrap();
    near(*s.points().last().unwrap(), [4., 4., 0.]);
}

#[test]
fn undo_option_restores_its_captured_input_unit_scale() {
    let mut s = session();
    s.set_input_scale(10.).unwrap();
    s.input("3Point").unwrap();
    s.set_input_scale(100.).unwrap();
    s.input("Undo").unwrap();
    s.input("1,0").unwrap();
    s.input("1").unwrap();
    near(s.circle().unwrap().center, [10., 0., 0.]);
    assert_eq!(s.circle().unwrap().radius, 10.);
}

#[test]
fn opaque_native_context_keys_restore_without_reuse_after_branching() {
    let mut s = session();
    assert_eq!(s.circle_input_context_key(), 0);
    assert_eq!(s.circle_input_context_keys(), vec![0]);
    s.input("AroundCurve").unwrap();
    let mode = s.circle_input_context_key();
    s.circle_native_reference([0.; 3], [0.; 3], 1).unwrap();
    let path = s.circle_input_context_key();
    s.input("Center").unwrap();
    let discarded = s.circle_input_context_key();
    s.input("Undo").unwrap();
    assert_eq!(s.circle_input_context_key(), path);
    assert!(!s.circle_input_context_keys().contains(&discarded));
    assert!(s.circle_input_context_keys().contains(&mode));
    assert!(s.input("bad").is_err());
    assert_eq!(s.circle_input_context_key(), path);
    s.input("History=Yes").unwrap();
    assert!(s.circle_input_context_key() > discarded);
    s.input("Undo").unwrap();
    s.input("Undo").unwrap();
    assert_eq!(s.circle_input_context_key(), mode);
    s.input("Cancel").unwrap();
    assert_eq!(s.circle_input_context_key(), 0);
}
