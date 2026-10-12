// OM9-SOLID-012 / OM9-SOLID-014: independent construction and degeneracy fixtures.
use openmatrix9_rust::solid::{Effect, Kind, Session};
fn step(s: &mut Session, text: &str) {
    assert_eq!(s.input(text).unwrap(), Effect::Waiting);
}
fn point(s: &mut Session, p: [f64; 3]) {
    assert_eq!(s.point(p).unwrap(), Effect::Waiting);
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-7, "{a} != {b}");
}
fn sphere(b: [f64; 15], c: [f64; 3], r: f64) {
    for i in 0..3 {
        near(b[i], c[i]);
    }
    near(b[12], r);
}

#[test]
fn box_center_is_symmetric_with_full_typed_dimensions() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "Mode=Center");
    point(&mut s, [10., 20., 30.]);
    step(&mut s, "4");
    step(&mut s, "6");
    assert_eq!(s.input("8").unwrap(), Effect::Commit);
    assert_eq!(&s.output().unwrap()[..3], &[8., 17., 30.]);
    let mut s = Session::new(Kind::Box);
    step(&mut s, "Center");
    point(&mut s, [10., 20., 30.]);
    point(&mut s, [8., 17., 30.]);
    assert_eq!(s.input("8").unwrap(), Effect::Commit);
    assert_eq!(&s.output().unwrap()[..3], &[8., 17., 30.]);
    assert_eq!(&s.output().unwrap()[12..], &[4., 6., 8.]);
}
#[test]
fn box_diagonal_rejects_length_and_vertical_base_is_perpendicular_to_cplane() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "Diagonal");
    step(&mut s, "0,0");
    assert!(s.input("Length=4").is_err());
    point(&mut s, [4., 3., 0.]);
    assert_eq!(s.input("2").unwrap(), Effect::Commit);
    let mut s = Session::new(Kind::Box);
    step(&mut s, "Vertical");
    point(&mut s, [0., 0., 0.]);
    point(&mut s, [4., 0., 0.]);
    point(&mut s, [4., 0., 3.]);
    assert_eq!(s.point([0., -2., 0.]).unwrap(), Effect::Commit);
    let b = s.output().unwrap();
    assert_eq!(&b[6..9], &[0., 0., 1.]);
    assert_eq!(&b[9..12], &[0., -1., 0.]);
    assert_eq!(&b[12..], &[4., 3., 2.]);
}
#[test]
fn box_cube_reaches_picked_opposite_corner_and_has_equal_edges() {
    for end in [[3., 3., 3.], [0., 0., 27_f64.sqrt()], [-3., -3., -3.]] {
        let mut s = Session::new(Kind::Box);
        step(&mut s, "Diagonal");
        step(&mut s, "Cube");
        point(&mut s, [0.; 3]);
        let preview = s.preview(end, false).unwrap();
        assert!(s.output().is_none());
        assert_eq!(s.point(end).unwrap(), Effect::Commit);
        let b = s.output().unwrap();
        assert_eq!(b, preview);
        for row in 0..3 {
            near(
                b[row]
                    + (0..3)
                        .map(|col| b[3 + col * 3 + row] * b[12 + col])
                        .sum::<f64>(),
                end[row],
            );
            near(b[12 + row], 3.);
        }
    }
}
#[test]
fn vertical_keeps_first_point_frame_when_view_changes() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "Vertical");
    point(&mut s, [0.; 3]);
    s.set_frame([0.; 3], [[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]])
        .unwrap();
    point(&mut s, [4., 0., 0.]);
    point(&mut s, [4., 0., 3.]);
    assert_eq!(s.point([0., -2., 0.]).unwrap(), Effect::Commit);
    assert_eq!(&s.output().unwrap()[12..], &[4., 3., 2.]);
}
#[test]
fn cube_near_antiparallel_diagonal_reaches_picked_corner() {
    let mut s = Session::new(Kind::Box);
    step(&mut s, "Cube");
    point(&mut s, [0.; 3]);
    let end = [-3000000., -3000000., -2999999.];
    assert_eq!(s.point(end).unwrap(), Effect::Commit);
    let b = s.output().unwrap();
    for row in 0..3 {
        near(
            (0..3).map(|col| b[3 + col * 3 + row] * b[12 + col]).sum(),
            end[row],
        );
    }
}
#[test]
fn sphere_two_point_uses_diameter_and_diameter_parameter_halves_radius() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "Mode=2Point");
    point(&mut s, [1., 2., 3.]);
    assert_eq!(s.point([1., 8., 11.]).unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [1., 5., 7.], 5.);
    let mut s = Session::new(Kind::Sphere);
    point(&mut s, [1., 2., 3.]);
    assert_eq!(s.input("Diameter=10mm").unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [1., 2., 3.], 5.);
}
#[test]
fn sphere_three_point_and_radius_orientation_are_world_geometric() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "3Point");
    point(&mut s, [12., 20., 30.]);
    point(&mut s, [10., 22., 30.]);
    assert_eq!(s.point([8., 20., 30.]).unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [10., 20., 30.], 2.);
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "3Point");
    point(&mut s, [-2., 0., 0.]);
    point(&mut s, [2., 0., 0.]);
    assert!(s.input("Radius=1").is_err());
    step(&mut s, "Radius=3");
    assert!(s.point([4., 0., 0.]).is_err());
    assert_eq!(s.point([0., 1., 1.]).unwrap(), Effect::Commit);
    sphere(
        s.output().unwrap(),
        [0., (2.5_f64).sqrt(), (2.5_f64).sqrt()],
        3.,
    );
}
#[test]
fn sphere_four_point_uses_circle_and_out_of_plane_fourth_point() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "4Point");
    point(&mut s, [1., 0., 0.]);
    point(&mut s, [0., 1., 0.]);
    point(&mut s, [-1., 0., 0.]);
    assert!(s.point([0., -1., 0.]).is_err());
    assert_eq!(s.point([0., 0., 2.]).unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [0., 0., 0.75], 1.25);
}
#[test]
fn sphere_fit_points_exact_sphere_and_planar_circle_with_input_undo() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "FitPoints");
    assert!(s.input("").is_err());
    for p in [
        [13., 20., 30.],
        [7., 20., 30.],
        [10., 23., 30.],
        [10., 17., 30.],
        [10., 20., 33.],
        [10., 20., 27.],
    ] {
        point(&mut s, p);
    }
    point(&mut s, [100., 200., 300.]);
    step(&mut s, "Undo");
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [10., 20., 30.], 3.);
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "FitPoints");
    for p in [[2., 0., 0.], [0., 2., 0.], [-2., 0., 0.], [0., -2., 0.]] {
        point(&mut s, p);
    }
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [0.; 3], 2.);
}
#[test]
fn sphere_degenerate_modes_preserve_editable_state_and_cancel() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "3Point");
    point(&mut s, [0.; 3]);
    point(&mut s, [1., 0., 0.]);
    assert!(s.point([2., 0., 0.]).is_err());
    assert!(s.output().is_none());
    step(&mut s, "Undo");
    assert_eq!(s.input("Cancel").unwrap(), Effect::Cancelled);
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "FitPoints");
    for p in [[0.; 3], [1., 0., 0.], [2., 0., 0.], [3., 0., 0.]] {
        point(&mut s, p);
    }
    assert!(s.input("").is_err());
    assert!(s.output().is_none());
}
#[test]
fn sphere_curve_modes_announce_reference_and_point_requirements() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "Tangent");
    assert!(s.prompt().contains("curve"));
    assert!(s.input("0,0").is_err());
    step(&mut s, "Point");
    step(&mut s, "0,0");
    step(&mut s, "Undo");
    assert!(s.output().is_none());
    assert_eq!(s.input("Cancel").unwrap(), Effect::Cancelled);
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "AroundCurve");
    assert!(s.prompt().contains("curve"));
    assert!(s.input("Radius=2").is_err());
}
#[test]
fn small_valid_three_point_circle_does_not_apply_length_tolerance_to_area_normal() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "3Point");
    point(&mut s, [1e-5, 0., 0.]);
    point(&mut s, [0., 1e-5, 0.]);
    assert_eq!(s.point([-1e-5, 0., 0.]).unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [0.; 3], 1e-5);
}
#[test]
fn fit_batch_is_atomic_and_noisy_fit_minimizes_radial_error() {
    let mut s = Session::new(Kind::Sphere);
    step(&mut s, "FitPoints");
    point(&mut s, [2., 0., 0.]);
    assert!(
        s.append_fit_points(&[[0., 2., 0.], [f64::NAN, 0., 0.]])
            .is_err()
    );
    assert!(s.input("").is_err());
    s.append_fit_points(&[
        [-2., 0., 0.],
        [0., 2.1, 0.],
        [0., -2.1, 0.],
        [0., 0., 1.9],
        [0., 0., -1.9],
    ])
    .unwrap();
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    sphere(s.output().unwrap(), [0.; 3], 2.);
}
