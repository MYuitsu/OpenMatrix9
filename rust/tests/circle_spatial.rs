use openmatrix9_rust::{circle, curve::CurveSession};
#[test]
fn om9_curve_005_plane_keeps_valid_cplane_and_finds_tilted_plane() {
    let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let pts = [[0., 0., 8.], [4., 0., 8.], [0., 5., 8.]];
    assert_eq!(
        circle::tangent_frame(&pts, [0., 0., 8.], axes).unwrap(),
        axes
    );
    let pts = [[10., 0., 0.], [10., 4., 0.], [10., 0., 5.]];
    let frame = circle::tangent_frame(&pts, pts[0], axes).unwrap();
    assert!((frame[2][0].abs() - 1.).abs() < 1e-10);
    assert!(
        circle::tangent_frame(
            &[[0., 0., 0.], [2., 0., 0.], [0., 2., 0.], [0., 0., 2.]],
            [0.; 3],
            axes
        )
        .is_err()
    );
}
#[test]
fn om9_curve_005_plane_handles_parallel_lines_and_degenerate_input() {
    let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let frame = circle::tangent_frame(
        &[[3., 0., 0.], [3., 0., 10.], [3., 4., 0.], [3., 4., 10.]],
        [3., 0., 0.],
        axes,
    )
    .unwrap();
    assert!((frame[2][0].abs() - 1.).abs() < 1e-10);
    assert!(circle::tangent_frame(&[[f64::NAN, 0., 0.]], [0.; 3], axes).is_err());
    assert!(circle::tangent_frame(&[[1., 1., 1.]; 3], [1., 1., 1.], axes).is_err());
}
#[test]
fn om9_curve_005_tiny_spatial_conic_preserves_plane_angle() {
    let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let points = [
        [0., -5e-7, 0.],
        [0., 0., 5e-7],
        [0., 5e-7, 0.],
        [0., 2e-6, 5e-7],
    ];
    let frame = circle::tangent_frame(&points, [0., 5e-7, 0.], axes).unwrap();
    assert!((frame[2][0].abs() - 1.).abs() < 1e-10);
}
#[test]
fn om9_curve_005_spatial_acceptance_and_vertical_are_atomic() {
    let mut s = CurveSession::default();
    s.start("Circle").unwrap();
    s.input("Tangent").unwrap();
    s.input("Vertical=Yes").unwrap();
    s.circle_native_reference([0., 4., 0.], [1., 0., 0.], 3)
        .unwrap();
    s.input("Radius=4").unwrap();
    s.input("Point").unwrap();
    s.input("0,8,4").unwrap();
    assert!(
        s.circle_accept_spatial_solution([0., 4., 4.], [0., 0., 1.], 4.)
            .is_err()
    );
    assert!(s.active());
    assert!(s.circle().is_none());
    s.circle_accept_spatial_solution([0., 4., 4.], [1., 0., 0.], 4.)
        .unwrap();
    assert_eq!(s.circle().unwrap().normal, [1., 0., 0.]);
}
#[test]
fn om9_curve_005_history_supports_all_modes_and_deformable() {
    let mut s = CurveSession::default();
    s.start("Circle").unwrap();
    s.input("History=Yes").unwrap();
    s.input("AroundCurve").unwrap();
    s.input("History=Yes").unwrap();
    assert!(s.circle_history());
    s.input("Deformable=Yes").unwrap();
    s.input("History=No").unwrap();
    s.input("Deformable=Yes").unwrap();
    s.input("History=Yes").unwrap();
    s.input("Cancel").unwrap();
    assert!(!s.circle_history());
}
