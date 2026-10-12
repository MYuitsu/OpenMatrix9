// OM9-SOLID-012 / OM9-SOLID-014: fixtures have independent analytic answers.
use openmatrix9_rust::solid::{Effect, Kind, Session};

fn step(s: &mut Session, text: &str) {
    assert_eq!(s.input(text).unwrap(), Effect::Waiting);
}

#[test]
fn box_corner_height_normalizes_negative_extents() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "10,20,30");
    step(&mut s, "6,14,30");
    assert_eq!(s.input("-8").unwrap(), Effect::Commit);
    let b = s.output().unwrap();
    assert_eq!(&b[..3], &[6., 14., 22.]);
    assert_eq!(&b[12..], &[4., 6., 8.]);
}
#[test]
fn box_numeric_enter_defaults_make_cube() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "0,0");
    step(&mut s, "Length=2cm");
    step(&mut s, "");
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    assert_eq!(&s.output().unwrap()[12..], &[20., 20., 20.]);
}
#[test]
fn three_point_box_preserves_rotated_edge_and_signed_width() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "3Point");
    step(&mut s, "1,2,3");
    step(&mut s, "1,6,3");
    step(&mut s, "-2");
    assert_eq!(s.input("3").unwrap(), Effect::Commit);
    let b = s.output().unwrap();
    assert_eq!(&b[..3], &[3., 2., 3.]);
    assert_eq!(&b[3..6], &[0., 1., 0.]);
    assert_eq!(&b[6..9], &[-1., 0., 0.]);
    assert_eq!(&b[12..], &[4., 2., 3.]);
}
#[test]
fn sphere_mouse_radius_is_world_distance_and_preview_does_not_commit() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "1,2,3");
    let p = s.preview([1., 5., 7.], false).unwrap();
    assert_eq!(&p[..3], &[1., 2., 3.]);
    assert_eq!(p[12], 5.);
    assert!(s.output().is_none());
    assert_eq!(s.phase(), 5);
    assert_eq!(s.point([1., 5., 7.]).unwrap(), Effect::Commit);
    assert_eq!(s.output().unwrap()[12], 5.);
}
#[test]
fn invalid_dimensions_remain_editable_and_undo_restores_phase() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "0,0");
    for text in ["0,2", "NaN", "Length=0", "1e20", "Center"] {
        assert!(s.input(text).is_err());
        assert_eq!(s.phase(), 2);
    }
    step(&mut s, "4");
    step(&mut s, "5");
    assert_eq!(s.phase(), 4);
    step(&mut s, "Undo");
    assert_eq!(s.phase(), 3);
    step(&mut s, "6");
    assert!(s.input("0").is_err());
    assert_eq!(s.phase(), 4);
    assert_eq!(s.input("Cancel").unwrap(), Effect::Cancelled);
    assert_eq!(s.phase(), 0);
    assert!(s.output().is_none());
}
#[test]
fn cplane_coordinates_and_relative_input_are_transformed_once() {
    let mut s = Session::new(Kind::Box);
    s.set_frame([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    step(&mut s, "1,2,3");
    step(&mut s, "r4,6,0");
    assert_eq!(s.input("8").unwrap(), Effect::Commit);
    let b = s.output().unwrap();
    assert_eq!(&b[..3], &[13., 21., 32.]);
    assert_eq!(&b[12..], &[4., 6., 8.]);
    assert_eq!(&b[9..12], &[1., 0., 0.]);
}
#[test]
fn sphere_rejects_zero_negative_nonfinite_and_cancelled_inputs() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "0,0");
    for text in ["0", "-1", "NaN", "inf", "1e20", "Diameter=0"] {
        assert!(s.input(text).is_err());
        assert_eq!(s.phase(), 5);
    }
    assert_eq!(s.input("Radius=3mm").unwrap(), Effect::Commit);
    assert_eq!(s.output().unwrap()[12], 3.);
    let mut s = Session::new(Kind::Sphere);
    assert_eq!(s.input("Esc").unwrap(), Effect::Cancelled);
    assert!(s.point([0., 0., 0.]).is_err());
}

#[test]
fn repeated_undo_walks_back_each_input_and_clears_output() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "0,0");
    step(&mut s, "4");
    step(&mut s, "5");
    for phase in [3, 2, 1] {
        step(&mut s, "Undo");
        assert_eq!(s.phase(), phase);
    }
    assert!(s.input("Undo").is_err());
    assert_eq!(s.phase(), 1);
}

#[test]
fn relative_height_uses_the_last_picked_width_point() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "3Point");
    step(&mut s, "0,0,3");
    step(&mut s, "4,0,3");
    step(&mut s, "4,5,7");
    assert_eq!(s.input("r0,0,2").unwrap(), Effect::Commit);
    assert_eq!(&s.output().unwrap()[12..], &[4., 5., 6.]);
}
