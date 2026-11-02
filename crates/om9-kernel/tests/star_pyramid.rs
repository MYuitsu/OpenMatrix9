// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::budget::Limits;
use om9_kernel::creation;

#[test]
fn star_pyramid_exact_volume_and_topology() {
    let limits = Limits::default();
    for (sides, first, second, height) in [(5, 5., 2., 6.), (4, 3., 3., 5.), (6, 2., 4., 7.)] {
        let body = creation::star_pyramid(sides, first, second, height, 0.05, &limits).unwrap();
        let m = creation::measure(&body, &limits).unwrap();
        let expected =
            sides as f64 * first * second * (std::f64::consts::PI / sides as f64).sin() * height
                / 3.;
        assert!(
            m.closed && m.solids == 1 && (m.volume_mm3 - expected).abs() < 1e-6,
            "{m:?}"
        );
    }
}

#[test]
fn star_pyramid_rejects_invalid_and_oversized_inputs() {
    let limits = Limits::default();
    for (n, a, b, h) in [
        (2, 3., 1., 5.),
        (5, 0., 1., 5.),
        (5, 3., 0., 5.),
        (5, 3., 1., 0.),
        (5, f64::NAN, 1., 5.),
        (2049, 3., 1., 5.),
    ] {
        assert!(creation::star_pyramid(n, a, b, h, 0.05, &limits).is_err());
    }
    let small = Limits {
        max_points: 8,
        ..Limits::default()
    };
    assert!(creation::star_pyramid(5, 3., 1., 5., 0.05, &small).is_err());
}
