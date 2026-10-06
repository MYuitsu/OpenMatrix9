use openmatrix9_rust::core_distance::om9_measure_distance;
use openmatrix9_rust::core_distance::{Unit, distance};

#[test]
fn om9_measure_002_bridge_rejects_invalid_unit_and_coordinates() {
    assert_eq!(om9_measure_distance(0., 0., 0., 3., 4., 0., 0), 5.);
    assert_eq!(om9_measure_distance(0., 0., 0., 304.8, 0., 0., 4), 1.);
    assert!(om9_measure_distance(0., 0., 0., 3., 4., 0., 99).is_nan());
    assert!(om9_measure_distance(f64::NAN, 0., 0., 3., 4., 0., 0).is_nan());
}

#[test]
fn om9_measure_002_reports_spatial_distance_and_coincident_points() {
    assert_eq!(
        distance([1., 2., 3.], [4., 6., 15.], Unit::Millimeter),
        Some(13.)
    );
    assert_eq!(
        distance([1., 2., 3.], [1., 2., 3.], Unit::Millimeter),
        Some(0.)
    );
    assert_eq!(
        distance([-3., -4., 0.], [0., 0., 0.], Unit::Millimeter),
        Some(5.)
    );
}

#[test]
fn om9_measure_002_units_convert_result_without_changing_points() {
    for (unit, expected) in [
        (Unit::Millimeter, 304.8),
        (Unit::Centimeter, 30.48),
        (Unit::Meter, 0.3048),
        (Unit::Inch, 12.),
        (Unit::Foot, 1.),
    ] {
        assert!((distance([0.; 3], [304.8, 0., 0.], unit).unwrap() - expected).abs() < 1e-10);
    }
}

#[test]
fn om9_measure_002_rejects_nonfinite_or_unsupported_coordinates() {
    for bad in [f64::NAN, f64::INFINITY, -f64::INFINITY, 1e10] {
        assert!(distance([bad, 0., 0.], [0.; 3], Unit::Millimeter).is_none());
        assert!(distance([0.; 3], [0., bad, 0.], Unit::Millimeter).is_none());
    }
    assert_eq!(
        distance([-1e9, 0., 0.], [1e9, 0., 0.], Unit::Millimeter),
        Some(2e9)
    );
}
