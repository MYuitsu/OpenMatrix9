//! OM9-CURVE-005: analytic plans, dimensions, spatial construction and lifecycle.
use openmatrix9_rust::curve::{CurveSession, Effect};
fn session() -> CurveSession {
    let mut s = CurveSession::default();
    s.start("Circle").unwrap();
    s
}
fn near(a: [f64; 3], b: [f64; 3]) {
    assert!(
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-7),
        "{a:?} != {b:?}"
    );
}
#[test]
fn om9_curve_005_center_radius_is_not_a_polygon_plan() {
    let mut s = session();
    s.input("1,2,3").unwrap();
    assert_eq!(s.input("5").unwrap(), Effect::Commit);
    let circle = s.circle().unwrap();
    near(circle.center, [1., 2., 3.]);
    near(circle.normal, [0., 0., 1.]);
    assert_eq!(circle.radius, 5.);
    assert_eq!(s.prompt(), "Command: ");
}
#[test]
fn om9_curve_005_size_modes_and_area_units_agree() {
    for (mode, size) in [
        ("Radius", "4"),
        ("Diameter", "8"),
        ("Circumference", "25.132741228718345"),
        ("Area", "50.26548245743669"),
        ("Area", "0.5026548245743669cm2"),
    ] {
        let mut s = session();
        s.input(mode).unwrap();
        s.input("0,0").unwrap();
        s.input(size).unwrap();
        assert!(
            (s.circle().unwrap().radius - 4.).abs() < 1e-7,
            "{mode} {size}"
        );
    }
}
#[test]
fn om9_curve_005_two_points_are_exact_3d_diameter_ends() {
    let mut s = session();
    s.input("2Point").unwrap();
    s.input("1,2,3").unwrap();
    s.input("7,2,11").unwrap();
    let c = s.circle().unwrap();
    near(c.center, [4., 2., 7.]);
    assert!((c.radius - 5.).abs() < 1e-7);
    assert!((c.normal[0] * 6. + c.normal[2] * 8.).abs() < 1e-7);
}
#[test]
fn om9_curve_005_three_points_define_spatial_circumcircle() {
    let mut s = session();
    s.input("3Point").unwrap();
    for p in ["1,2,6", "5,2,2", "1,2,-2"] {
        s.input(p).unwrap();
    }
    let c = s.circle().unwrap();
    near(c.center, [1., 2., 2.]);
    assert!((c.radius - 4.).abs() < 1e-7);
    assert!(c.normal[1].abs() > 0.999999);
}
#[test]
fn om9_curve_005_collinear_and_invalid_sizes_are_recoverable() {
    let mut s = session();
    s.input("3Point").unwrap();
    s.input("0,0").unwrap();
    s.input("5,0").unwrap();
    for text in ["10,0", "0,0", "Radius=1"] {
        assert!(s.input(text).is_err());
        assert!(s.active());
        assert_eq!(s.points().len(), 2);
    }
    s.input("0,5").unwrap();
    assert!(s.circle().is_some());
    let mut s = session();
    s.input("0,0").unwrap();
    for text in [
        "0",
        "-2",
        "nan",
        "Diameter=inf",
        "Area=-1",
        "Tangent",
        "Deformable=invalid",
    ] {
        assert!(s.input(text).is_err());
        assert!(s.active());
        assert!(s.circle().is_none());
    }
    s.input("Diameter=8").unwrap();
    assert_eq!(s.circle().unwrap().radius, 4.);
}
#[test]
fn om9_curve_005_orientation_changes_plane_and_projects_radius() {
    let mut s = session();
    s.input("2,3,4").unwrap();
    s.input("Orientation").unwrap();
    s.input("2,4,4").unwrap();
    assert_eq!(s.points().len(), 1);
    s.input("5,8,8").unwrap();
    let c = s.circle().unwrap();
    near(c.normal, [0., 1., 0.]);
    assert!((c.radius - 5.).abs() < 1e-7);
}
#[test]
fn om9_curve_005_vertical_point_and_numeric_planes_are_explicit() {
    let mut s = session();
    s.input("Vertical").unwrap();
    s.input("0,0").unwrap();
    s.input("3,4").unwrap();
    let c = s.circle().unwrap();
    assert!((c.radius - 5.).abs() < 1e-7);
    assert!(c.normal[2].abs() < 1e-7);
    assert!((c.normal[0] * 3. + c.normal[1] * 4.).abs() < 1e-7);
    let mut s = session();
    s.input("Vertical").unwrap();
    s.input("0,0").unwrap();
    s.input("5").unwrap();
    near(s.circle().unwrap().normal, [0., 1., 0.]);
}
#[test]
fn om9_curve_005_frame_preview_and_undo_are_non_destructive() {
    let mut s = session();
    s.set_frame([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    s.input("0,0").unwrap();
    let preview = s.preview_geometry([10., 24., 33.], false).unwrap();
    assert!(s.active());
    assert!(s.circle().is_none());
    assert_eq!(preview.len(), 129);
    s.point([10., 24., 33.]).unwrap();
    let c = s.circle().unwrap();
    near(c.center, [10., 20., 30.]);
    near(c.normal, [1., 0., 0.]);
    assert!((c.radius - 5.).abs() < 1e-7);
    let mut s = session();
    s.input("3Point").unwrap();
    s.input("0,0").unwrap();
    s.input("5,0").unwrap();
    s.input("Undo").unwrap();
    assert_eq!(s.points().len(), 1);
    assert_eq!(s.input("Cancel").unwrap(), Effect::Cancelled);
    assert!(s.circle().is_none());
    assert!(s.output().is_empty());
    assert_eq!(s.prompt(), "Command: ");
}
#[test]
fn om9_curve_005_small_three_point_circle_respects_radius_tolerance() {
    let mut s = session();
    s.input("3Point").unwrap();
    for p in ["0,0", "0.0001,0", "0,0.0001"] {
        s.input(p).unwrap();
    }
    assert!((s.circle().unwrap().radius - 0.00005 * 2f64.sqrt()).abs() < 1e-12);
}
#[test]
fn om9_curve_005_extent_failure_and_orientation_undo_are_atomic() {
    let mut s = session();
    s.input("999999999,0").unwrap();
    assert!(s.input("Diameter=8").is_err());
    assert!(s.active());
    assert_eq!(s.points().len(), 1);
    // Failed named sizing must not change Radius into Diameter.
    s.input("1").unwrap();
    assert_eq!(s.circle().unwrap().radius, 1.);
    let mut s = session();
    s.input("0,0").unwrap();
    s.input("Orientation").unwrap();
    s.input("0,1,0").unwrap();
    s.input("Undo").unwrap();
    assert!(s.input("4").is_err());
    s.input("0,0,1").unwrap();
    s.input("4").unwrap();
    near(s.circle().unwrap().normal, [0., 0., 1.]);
}
