//! OM9-CURVE-004: exact corners, construction frames and recoverable failures.
use openmatrix9_rust::curve::{CurveSession, Effect};

fn session() -> CurveSession {
    let mut s = CurveSession::default();
    s.start("Rectangle").unwrap();
    s
}
fn near(a: [f64; 3], b: [f64; 3]) {
    assert!(
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-7),
        "{a:?} != {b:?}"
    );
}
#[test]
fn om9_curve_004_opposite_corners_form_four_closed_edges() {
    let mut s = session();
    s.input("1,2,3").unwrap();
    assert_eq!(s.input("11,8,9").unwrap(), Effect::Commit);
    let wanted = [
        [1., 2., 3.],
        [11., 2., 3.],
        [11., 8., 3.],
        [1., 8., 3.],
        [1., 2., 3.],
    ];
    assert_eq!(s.output().len(), 5);
    for (&a, b) in s.output().iter().zip(wanted) {
        near(a, b);
    }
}
#[test]
fn om9_curve_004_center_and_signed_dimensions() {
    let mut s = session();
    s.input("Center").unwrap();
    s.input("5,5").unwrap();
    s.input("Length=10").unwrap();
    assert_eq!(s.input("Width=-6").unwrap(), Effect::Commit);
    near(s.output()[0], [0., 8., 0.]);
    near(s.output()[2], [10., 2., 0.]);
}
#[test]
fn om9_curve_004_three_point_slanted_plane_projects_width() {
    let mut s = session();
    s.input("3Point").unwrap();
    s.point([0., 0., 0.]).unwrap();
    s.point([3., 0., 4.]).unwrap();
    s.point([6., 2., 8.]).unwrap();
    near(s.output()[1], [3., 0., 4.]);
    near(s.output()[2], [3., 2., 4.]);
}
#[test]
fn om9_curve_004_vertical_uses_frame_normal() {
    let mut s = session();
    s.input("Vertical").unwrap();
    s.input("0,0").unwrap();
    s.input("5,0,8").unwrap();
    s.input("-3").unwrap();
    near(s.output()[1], [5., 0., 0.]);
    near(s.output()[2], [5., 0., -3.]);
}
#[test]
fn om9_curve_004_rotated_cplane_and_units() {
    let mut s = session();
    s.set_frame([20., 30., 40.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    s.input("0,0").unwrap();
    s.input("1cm").unwrap();
    s.input("0,6").unwrap();
    near(s.output()[0], [20., 30., 40.]);
    near(s.output()[2], [20., 40., 46.]);
}
#[test]
fn om9_curve_004_shift_makes_square_without_global_ortho() {
    let mut s = session();
    s.point([0., 0., 0.]).unwrap();
    s.mouse_point([-10., 6., 0.], true).unwrap();
    near(s.output()[2], [-10., 10., 0.]);
}
#[test]
fn om9_curve_004_invalid_input_does_not_advance_or_commit() {
    let mut s = session();
    s.input("0,0").unwrap();
    for text in ["0,6", "Length=0", "NaN", "Rounded", "Conic", "Width=inf"] {
        assert!(s.input(text).is_err(), "{text}");
        assert!(s.active());
        assert!(s.output().is_empty());
        assert_eq!(s.points().len(), 1);
    }
    s.input("10,6").unwrap();
    assert_eq!(s.output().len(), 5);
}
#[test]
fn om9_curve_004_undo_and_cancel_clear_pending_state() {
    let mut s = session();
    s.input("3Point").unwrap();
    s.input("0,0").unwrap();
    s.input("5,0").unwrap();
    s.input("Undo").unwrap();
    assert_eq!(s.points().len(), 1);
    assert_eq!(s.input("Cancel").unwrap(), Effect::Cancelled);
    assert!(s.output().is_empty());
    s.start("Rectangle").unwrap();
    assert!(s.points().is_empty());
    s.input("0,0").unwrap();
    s.input("4,2").unwrap();
    assert_eq!(s.output().len(), 5);
}
#[test]
fn om9_curve_004_preview_is_closed_matches_click_and_keeps_session() {
    let mut s = session();
    s.input("0,0").unwrap();
    let preview = s.preview_geometry([10., 6., 0.], true).unwrap();
    assert_eq!(preview.len(), 5);
    assert_eq!(s.points().len(), 1);
    assert!(s.active());
    s.mouse_point([10., 6., 0.], true).unwrap();
    assert_eq!(preview, s.output());
}
#[test]
fn om9_curve_004_numeric_steps_and_frame_are_stable() {
    let mut s = session();
    s.input("0,0").unwrap();
    s.set_frame([20., 0., 0.], [[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]])
        .unwrap();
    s.input("10").unwrap();
    s.input("6").unwrap();
    near(s.output()[2], [10., 6., 0.]);
    s.start("Rectangle").unwrap();
    s.input("3Point").unwrap();
    s.input("0,0").unwrap();
    s.input("4,0").unwrap();
    s.input("3").unwrap();
    near(s.output()[2], [4., 3., 0.]);
}
#[test]
fn om9_curve_004_rejects_edge_overflow_without_advancing() {
    let mut s = session();
    s.input("3Point").unwrap();
    s.point([9e8, 0., 0.]).unwrap();
    s.input("Length=1000000000").unwrap();
    assert!(s.point([9e8 + 1., 0., 0.]).is_err());
    assert_eq!(s.points().len(), 1);
    s.input("Length=10").unwrap();
    s.point([9e8 + 1., 0., 0.]).unwrap();
    s.input("Width=3").unwrap();
    near(s.output()[1], [9e8 + 10., 0., 0.]);
}
#[test]
fn om9_curve_004_early_width_still_obeys_square_in_click_and_preview() {
    for mode in ["3Point", "Vertical"] {
        let mut s = session();
        s.input(mode).unwrap();
        s.input("0,0").unwrap();
        s.input("Width=3").unwrap();
        let preview = s.preview_geometry([5., 0., 0.], true).unwrap();
        s.mouse_point([5., 0., 0.], true).unwrap();
        assert_eq!(s.output(), preview);
        let d: Vec<_> = s
            .output()
            .windows(2)
            .map(|p| {
                p[0].iter()
                    .zip(p[1])
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();
        assert!(d.iter().all(|n| (n - 5.).abs() < 1e-7), "{mode}: {d:?}");
    }
}
