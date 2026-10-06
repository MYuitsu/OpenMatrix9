use openmatrix9_rust::core_angle::angle;
use openmatrix9_rust::core_angle::om9_measure_angle;

#[test]
fn om9_measure_001_native_bridge_uses_four_xyz_points() {
    let values = [1., 2., 3., 4., 2., 3., 7., 8., 9., 7., 12., 9.];
    assert!((unsafe { om9_measure_angle(values.as_ptr()) } - 90.).abs() < 1e-10);
    assert!(unsafe { om9_measure_angle(std::ptr::null()) }.is_nan());
    assert!(unsafe { om9_measure_angle([0.; 12].as_ptr()) }.is_nan());
}

#[test]
fn om9_measure_001_four_points_allow_distinct_line_origins() {
    let measured = angle([[1., 2., 3.], [4., 2., 3.], [7., 8., 9.], [7., 12., 9.]]).unwrap();
    assert!((measured - 90.).abs() < 1e-10);
    assert!(
        (angle([[0.; 3], [1., 1., 0.], [3., 4., 5.], [4., 4., 6.]]).unwrap() - 60.).abs() < 1e-10
    );
}
#[test]
fn om9_measure_001_parallel_opposite_and_scale_invariance() {
    for (end, expected) in [
        ([100., 0., 0.], 0.),
        ([-100., 0., 0.], 180.),
        ([100., 100., 0.], 45.),
    ] {
        assert!((angle([[0.; 3], [0.01, 0., 0.], [0.; 3], end]).unwrap() - expected).abs() < 1e-10);
    }
    let small = angle([[0.; 3], [1., 0., 0.], [0.; 3], [1., 1e-10, 0.]]).unwrap();
    assert!(small > 0. && small < 1e-7);
}
#[test]
fn om9_measure_001_invalid_and_degenerate_lines_are_rejected() {
    assert!(angle([[0.; 3], [0.; 3], [0.; 3], [1., 0., 0.]]).is_none());
    assert!(angle([[0.; 3], [1., 0., 0.], [2.; 3], [2.; 3]]).is_none());
    for bad in [f64::NAN, f64::INFINITY, -f64::INFINITY, 1e10] {
        assert!(angle([[0.; 3], [1., 0., 0.], [0.; 3], [1., bad, 0.]]).is_none());
    }
}
