//! OM9-CURVE-006: portable ellipse/session invariants, independent of host tests.
use openmatrix9_rust::curve::{CurveSession, Effect};
fn session(mode: &str) -> CurveSession {
    let mut s = CurveSession::default();
    s.start("Ellipse").unwrap();
    if !mode.is_empty() {
        s.input(mode).unwrap();
    }
    s
}
#[test]
fn ellipse_center_two_axes_and_closed_output() {
    let mut s = session("");
    s.input("1,2,3").unwrap();
    s.input("6,2,3").unwrap();
    assert_eq!(s.input("1,5,3").unwrap(), Effect::Commit);
    assert_eq!(s.output().first(), s.output().last());
    assert_eq!(s.output().len(), 129);
}
#[test]
fn ellipse_invalid_last_axis_does_not_advance_and_undo_recovers() {
    let mut s = session("");
    s.input("0,0").unwrap();
    s.input("5,0").unwrap();
    assert!(s.input("2,0").is_err());
    assert_eq!(s.points().len(), 2);
    assert!(s.active());
    s.input("Undo").unwrap();
    assert_eq!(s.points().len(), 1);
    s.input("8,0").unwrap();
    assert_eq!(s.input("0,4").unwrap(), Effect::Commit);
}
#[test]
fn ellipse_modes_commit_and_cancel() {
    for (mode, points) in [
        ("Diameter", vec!["-5,0", "5,0", "0,3"]),
        ("Corner", vec!["-5,-3", "5,3"]),
        ("Vertical", vec!["0,0", "5,0", "0,0,3"]),
        ("FromFoci", vec!["-4,0", "4,0", "0,3"]),
    ] {
        let mut s = session(mode);
        for p in points {
            s.input(p).unwrap();
        }
        assert!(!s.active(), "{mode}");
        assert_eq!(s.output().len(), 129);
        s.start("Ellipse").unwrap();
        assert_eq!(s.input("Cancel").unwrap(), Effect::Cancelled);
        assert!(s.output().is_empty());
    }
}
#[test]
fn ellipse_swapped_axes_canonicalize_without_changing_geometry() {
    let mut s = session("");
    for p in ["0,0", "3,0", "0,5"] {
        s.input(p).unwrap();
    }
    let p = s.ellipse().unwrap();
    assert_eq!((p.major, p.minor), (5., 3.));
    assert!(p.major_direction[1] > 0.999999);
    assert_eq!(p.center, [0.; 3]);
    for p in s.output() {
        assert!((p[0] * p[0] / 9. + p[1] * p[1] / 25. - 1.).abs() < 1e-10);
    }
}
#[test]
fn ellipse_frame_latches_and_numeric_units_resolve_axis_lengths() {
    let mut s = session("");
    s.set_frame([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    s.input("0,0").unwrap();
    s.set_frame([0.; 3], [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
        .unwrap();
    s.input("1cm").unwrap();
    s.input("5mm").unwrap();
    let p = s.ellipse().unwrap();
    assert_eq!(p.center, [10., 20., 30.]);
    assert_eq!((p.major, p.minor), (10., 5.));
    assert_eq!(p.normal, [1., 0., 0.]);
    assert_eq!(p.major_direction, [0., 1., 0.]);
}
#[test]
fn ellipse_diameter_number_is_full_length() {
    let mut s = session("Diameter");
    for p in ["0,0", "10", "3"] {
        s.input(p).unwrap();
    }
    let p = s.ellipse().unwrap();
    assert_eq!(p.center, [5., 0., 0.]);
    assert_eq!((p.major, p.minor), (5., 3.));
}
#[test]
fn ellipse_diameter_projects_axis_end_into_latched_plane() {
    let mut s = session("Diameter");
    for p in ["1,2,3", "11,2,8", "6,5,12"] {
        s.input(p).unwrap();
    }
    let p = s.ellipse().unwrap();
    assert_eq!(p.center, [6., 2., 3.]);
    assert_eq!((p.major, p.minor), (5., 3.));
}
#[test]
fn ellipse_foci_construction_passes_through_spatial_point_and_marks_exact_foci() {
    let mut s = session("FromFoci");
    s.input("MarkFoci=Yes").unwrap();
    for p in ["1,2,-1", "1,2,7", "1,5,3"] {
        s.input(p).unwrap();
    }
    let p = s.ellipse().unwrap();
    assert_eq!(p.center, [1., 2., 3.]);
    assert!((p.major - 5.).abs() < 1e-10);
    assert!((p.minor - 3.).abs() < 1e-10);
    assert_eq!(p.foci(), [[1., 2., 7.], [1., 2., -1.]]);
    assert!(s.ellipse_mark_foci());
    assert!(p.normal[0].abs() > 0.999999);
}
#[test]
fn ellipse_foci_degenerate_input_is_recoverable() {
    let mut s = session("FromFoci");
    s.input("-4,0").unwrap();
    assert!(s.input("-4,0").is_err());
    assert_eq!(s.points().len(), 1);
    s.input("4,0").unwrap();
    for p in ["0,0", "5,0", "nan,3"] {
        assert!(s.input(p).is_err());
        assert_eq!(s.points().len(), 2);
    }
    s.input("0,3").unwrap();
    assert!(!s.active());
}
#[test]
fn ellipse_options_and_pending_input_are_atomic() {
    let mut s = session("");
    let before = s.prompt();
    for p in [
        "Deformable=Maybe",
        "Degree=8",
        "PointCount=3",
        "MarkFoci=Yes",
        "History=Yes",
    ] {
        assert!(s.input(p).is_err());
        assert_eq!(s.prompt(), before);
    }
    s.input("Degree").unwrap();
    assert!(s.input("0").is_err());
    assert!(s.prompt().contains("Degree:"));
    assert!(s.point([0.; 3]).is_err());
    s.input("2").unwrap();
    s.input("0,0").unwrap();
    assert!(s.input("Corner").is_err());
    assert_eq!(s.points().len(), 1);
}
#[test]
fn ellipse_preview_does_not_mutate_and_global_ortho_does_not_collapse_axes() {
    let mut s = session("");
    s.input("0,0").unwrap();
    s.mouse_point([3., 4., 0.], true).unwrap();
    let p = s.points().to_vec();
    let preview = s.preview_geometry([-4., 3., 0.], true).unwrap();
    assert_eq!(preview.len(), 129);
    assert_eq!(s.points(), p);
    assert!(s.active());
    assert!(s.ellipse().is_none());
    s.mouse_point([-4., 3., 0.], true).unwrap();
    assert!((s.ellipse().unwrap().minor - 5.).abs() < 1e-10);
}
#[test]
fn ellipse_around_curve_requires_native_center_and_supports_undo_reselect() {
    let mut s = session("AroundCurve");
    assert_eq!(s.ellipse_reference_mode(), 1);
    assert!(s.input("0,0").is_err());
    s.ellipse_native_reference([0.; 3], [0.; 3], 1).unwrap();
    assert_eq!(s.ellipse_reference_mode(), 2);
    assert!(s.ellipse_native_reference([0.; 3], [0.; 3], 2).is_err());
    assert_eq!(s.ellipse_reference_mode(), 2);
    s.ellipse_native_reference([1., 2., 3.], [0., 1., 0.], 2)
        .unwrap();
    s.input("Undo").unwrap();
    assert_eq!(s.ellipse_reference_mode(), 2);
    s.input("Undo").unwrap();
    assert_eq!(s.ellipse_reference_mode(), 1);
    s.ellipse_native_reference([0.; 3], [0.; 3], 1).unwrap();
    s.ellipse_native_reference([1., 2., 3.], [0., 1., 0.], 2)
        .unwrap();
    s.point([6., 2., 3.]).unwrap();
    s.point([1., 2., 6.]).unwrap();
    let p = s.ellipse().unwrap();
    assert_eq!(p.normal, [0., 1., 0.]);
    assert_eq!((p.major, p.minor), (5., 3.));
}
#[test]
fn ellipse_deformable_has_requested_degree_and_poles_and_stays_in_plane() {
    for (degree, count) in [(1, 8), (2, 9), (3, 8), (5, 12)] {
        let mut s = session("");
        s.input("Deformable=Yes").unwrap();
        s.input(&format!("Degree={degree}")).unwrap();
        s.input(&format!("PointCount={count}")).unwrap();
        for p in ["0,0,7", "5,0,7", "0,3,7"] {
            s.input(p).unwrap();
        }
        let c = s.spline().unwrap();
        assert!(c.periodic);
        assert_eq!(c.degree, degree);
        assert_eq!(c.poles.len(), count);
        for i in 0..100 {
            assert!((c.value(i as f64 / 100.)[2] - 7.).abs() < 1e-10);
        }
        assert!(s.ellipse_deviation().is_finite());
        assert!(s.ellipse_deviation() > 0.);
    }
}
#[test]
fn ellipse_coordinate_extent_and_tiny_valid_axes_are_checked() {
    let mut s = session("");
    s.input("999999999,0").unwrap();
    assert!(s.input("2").is_err() || s.input("3").is_err());
    let mut tiny = session("");
    for p in ["0,0", "0.0000001", "0.0000002"] {
        tiny.input(p).unwrap();
    }
    assert_eq!(tiny.ellipse().unwrap().major, 2e-7);
}
