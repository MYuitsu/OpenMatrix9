//! OM9-CURVE-005 advanced construction semantics.
use openmatrix9_rust::curve::{CurveSession, Effect};
fn session() -> CurveSession {
    let mut s = CurveSession::default();
    s.start("Circle").unwrap();
    s
}
#[test]
fn om9_curve_005_radius_after_two_points_chooses_center_side() {
    let mut s = session();
    for p in ["3Point", "-3,0,0", "3,0,0", "Radius=5"] {
        s.input(p).unwrap();
    }
    assert!(s.active());
    assert_eq!(s.input("0,10,0").unwrap(), Effect::Commit);
    let c = s.circle().unwrap();
    assert!((c.center[1] - 4.).abs() < 1e-7);
    assert_eq!(c.radius, 5.);
}
#[test]
fn om9_curve_005_fit_points_exact_spatial_circle() {
    let mut s = session();
    for p in ["FitPoints", "1,2,6", "5,2,2", "1,2,-2", "-3,2,2"] {
        s.input(p).unwrap();
    }
    assert_eq!(s.input("").unwrap(), Effect::Commit);
    let c = s.circle().unwrap();
    assert!((c.radius - 4.).abs() < 1e-7);
    assert!(c.normal[1].abs() > 0.999999);
}
#[test]
fn om9_curve_005_area_point_is_radius_pick_with_area_readout() {
    let mut s = session();
    for p in ["Area", "0,0", "4,0"] {
        s.input(p).unwrap();
    }
    assert_eq!(s.circle().unwrap().radius, 4.);
}
#[test]
fn om9_curve_005_area_hover_reports_measure_without_mutating_session() {
    let mut s = session();
    s.input("Area").unwrap();
    s.input("0,0").unwrap();
    let plan = s.circle_hover_plan([4., 0., 0.]).unwrap();
    assert_eq!(plan.radius, 4.);
    assert!(s.active());
    assert!(s.circle().is_none());
    assert_eq!(s.points().len(), 1);
}
#[test]
fn om9_curve_005_deformable_exact_count_degree_periodic_output() {
    let mut s = session();
    for p in ["Deformable=Yes", "Degree=3", "PointCount=12", "0,0", "5"] {
        s.input(p).unwrap();
    }
    let spline = s.spline().unwrap();
    assert!(spline.periodic);
    assert_eq!(spline.degree, 3);
    assert_eq!(spline.poles.len(), 12);
    for i in 0..64 {
        let p = spline.value(i as f64 / 64.);
        assert!((p[0].hypot(p[1]) - 5.).abs() < 0.01);
    }
}
#[test]
fn om9_curve_005_reference_phases_and_tangent_feedback() {
    let mut s = session();
    s.input("AroundCurve").unwrap();
    assert_eq!(s.circle_reference_mode(), 1);
    s.circle_native_reference([0.; 3], [0.; 3], 1).unwrap();
    assert_eq!(s.circle_reference_mode(), 2);
    assert!(s.input("0,0").is_err());
    s.circle_native_reference([2., 3., 4.], [1., 0., 0.], 2)
        .unwrap();
    s.input("Diameter=10").unwrap();
    assert_eq!(s.circle().unwrap().normal, [1., 0., 0.]);
    let mut s = session();
    s.input("Tangent").unwrap();
    s.input("Radius=4").unwrap();
    s.circle_native_reference([0., 4., 0.], [0.; 3], 3).unwrap();
    s.input("Point").unwrap();
    s.input("4,8").unwrap();
    assert_eq!(s.circle_reference_mode(), 5);
    assert!(s.circle_accept_solution([4., 4., 0.], 2.).is_err());
    assert!(s.active());
    assert_eq!(
        s.circle_accept_solution([4., 4., 0.], 4.).unwrap(),
        Effect::Commit
    );
}
#[test]
fn om9_curve_005_fit_degenerate_and_batch_rejection_do_not_advance() {
    let mut s = session();
    s.input("FitPoints").unwrap();
    s.circle_fit_batch(&[[0.; 3], [1., 0., 0.], [2., 0., 0.]])
        .unwrap();
    assert!(s.input("").is_err());
    assert!(s.active());
    assert_eq!(s.points().len(), 3);
    assert!(s.circle_fit_batch(&[[f64::NAN, 0., 0.]]).is_err());
    assert_eq!(s.points().len(), 3);
    s.input("Undo").unwrap();
    s.input("0,1").unwrap();
    s.input("").unwrap();
    assert!(s.circle().is_some());
}
#[test]
fn om9_curve_005_from_first_point_and_radius_undo_recover() {
    let mut s = session();
    s.input("Tangent").unwrap();
    s.input("FromFirstPoint=Yes").unwrap();
    s.input("Point").unwrap();
    assert!(s.input("0,0").is_err());
    assert!(s.points().is_empty());
    s.input("FromFirstPoint=No").unwrap();
    s.input("0,0").unwrap();
    s.input("Undo").unwrap();
    assert_eq!(s.circle_reference_mode(), 3);
    let mut s = session();
    for p in ["3Point", "-3,0", "3,0", "Radius", "5", "Undo", "0,3"] {
        s.input(p).unwrap();
    }
    assert!((s.circle().unwrap().radius - 3.).abs() < 1e-7);
}
#[test]
fn om9_curve_005_tiny_deformable_radius_uses_normalized_internal_samples() {
    let mut s = session();
    for p in ["Deformable=Yes", "0,0", "0.00001"] {
        s.input(p).unwrap();
    }
    assert!(s.spline().is_some());
    assert!(
        (s.spline().unwrap().value(0.)[0].hypot(s.spline().unwrap().value(0.)[1]) - 1e-5).abs()
            < 1e-8
    );
}
#[test]
fn om9_curve_005_noisy_tilted_fit_reports_spatial_deviation() {
    let mut s = session();
    s.input("FitPoints").unwrap();
    let root = 2f64.sqrt();
    let points: Vec<_> = (0..24)
        .map(|i| {
            let t = std::f64::consts::TAU * i as f64 / 24.;
            let r = 5. + 0.01 * (3. * t).cos();
            [
                100. + r * t.cos() / root,
                200. + r * t.cos() / root,
                300. + r * t.sin(),
            ]
        })
        .collect();
    s.circle_fit_batch(&points).unwrap();
    s.input("").unwrap();
    let c = s.circle().unwrap();
    assert!((c.radius - 5.).abs() < 1e-4);
    assert!((c.center[0] - 100.).abs() < 1e-7);
    assert!((c.normal[0].abs() - 1. / root).abs() < 1e-7);
    let (fit, _) = s.circle_deviations();
    assert!((0.009..0.011).contains(&fit));
}
#[test]
fn om9_curve_005_cancel_clears_references_and_radius_point_replaces_pending_value() {
    let mut s = session();
    s.input("Tangent").unwrap();
    s.circle_native_reference([0., 4., 0.], [0.; 3], 3).unwrap();
    s.input("Radius").unwrap();
    s.input("0,8,0").unwrap();
    s.input("Point").unwrap();
    s.input("4,8,0").unwrap();
    assert_eq!(s.circle_reference_mode(), 5);
    s.input("Cancel").unwrap();
    assert_eq!(s.circle_reference_mode(), 0);
    assert!(s.points().is_empty());
    s.start("Circle").unwrap();
    s.input("0,0").unwrap();
    s.input("3").unwrap();
    assert_eq!(s.circle().unwrap().radius, 3.);
}
