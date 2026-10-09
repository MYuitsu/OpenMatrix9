use openmatrix9_rust::{
    spline, spline_ffi,
    surface_refit::{self, refit},
};
use std::f64::consts::TAU;

// Resample a smooth native-like curve at equal polyline arc distances, matching
// the native bridge's equal arc-length sample contract.
fn arc_samples(count: usize) -> Vec<[f64; 3]> {
    let source: Vec<_> = (0..=8192)
        .map(|i| {
            let t = i as f64 / 8192.;
            [40. * t, 4. * (TAU * t).sin(), 0.6 * (2. * TAU * t).sin()]
        })
        .collect();
    let mut lengths = vec![0.];
    for pair in source.windows(2) {
        lengths.push(lengths.last().unwrap() + spline::distance(pair[0], pair[1]));
    }
    let mut segment = 1;
    (0..count)
        .map(|i| {
            let target = lengths.last().unwrap() * i as f64 / (count - 1) as f64;
            while segment + 1 < source.len() && lengths[segment] < target {
                segment += 1;
            }
            let alpha = (target - lengths[segment - 1]) / (lengths[segment] - lengths[segment - 1]);
            std::array::from_fn(|axis| {
                source[segment - 1][axis] * (1. - alpha) + source[segment][axis] * alpha
            })
        })
        .collect()
}

#[test]
fn tighter_tolerance_adapts_the_fit_and_preserves_open_endpoints() {
    let samples = arc_samples(513);
    let loose = refit(&samples, 0.4, false).unwrap();
    let tight = refit(&samples, 0.004, false).unwrap();
    assert!(tight.curve.poles.len() > loose.curve.poles.len());
    assert!(tight.curve.poles.len() <= spline::MAX_POLES);
    assert!(tight.curve.degree <= 3);
    assert!(tight.sample_deviation <= 0.001);
    assert_eq!(tight.sample_count, samples.len());
    assert_eq!(tight.curve.value(0.), samples[0]);
    assert_eq!(tight.curve.value(1.), *samples.last().unwrap());
    let measured = samples
        .iter()
        .enumerate()
        .map(|(i, &point)| {
            spline::distance(
                point,
                tight.curve.value(i as f64 / (samples.len() - 1) as f64),
            )
        })
        .fold(0., f64::max);
    assert!((measured - tight.sample_deviation).abs() < 1e-12);
}

#[test]
fn smooth_closed_samples_produce_a_periodic_candidate() {
    let samples: Vec<_> = (0..=512)
        .map(|i| {
            let theta = TAU * i as f64 / 512.;
            [10. * theta.cos(), 10. * theta.sin(), 0.]
        })
        .collect();
    let result = refit(&samples, 0.01, true).unwrap();
    assert!(result.curve.periodic);
    assert!(result.sample_deviation <= 0.0025);
    assert!(spline::distance(result.curve.value(0.), result.curve.value(1.)) < 1e-12);
    let mut unclosed = samples;
    unclosed.pop();
    assert!(refit(&unclosed, 0.01, true).is_err());
}

#[test]
fn tolerance_and_sample_inputs_are_bounded() {
    let line: Vec<_> = (0..9).map(|i| [i as f64, 0., 0.]).collect();
    for tolerance in [0., -1., f64::NAN, f64::INFINITY, 1_000_000.1] {
        assert!(refit(&line, tolerance, false).is_err());
    }
    assert!(refit(&line, 1_000_000., false).is_ok());
    assert!(refit(&[], 0.1, false).is_err());
    assert!(refit(&vec![[0.; 3]; 9], 0.1, false).is_err());
    let mut invalid = line;
    invalid[2][0] = f64::NAN;
    assert!(refit(&invalid, 0.1, false).is_err());
}

#[test]
fn ffi_rejects_unattainable_tolerance_and_clears_the_published_candidate() {
    let samples = arc_samples(129);
    let xyz: Vec<_> = samples.into_iter().flatten().collect();
    assert!(unsafe { surface_refit::om9_surface_refit(xyz.as_ptr(), xyz.len() / 3, 0.1, false) });
    assert!(spline_ffi::om9_spline_pole_count() > 0);
    assert!(surface_refit::om9_surface_refit_deviation().is_finite());
    let zigzag: Vec<_> = (0..521)
        .flat_map(|i| [i as f64, (i % 2) as f64, 0.])
        .collect();
    assert!(!unsafe { surface_refit::om9_surface_refit(zigzag.as_ptr(), 521, 1e-12, false) });
    assert_eq!(spline_ffi::om9_spline_pole_count(), 0);
    assert!(surface_refit::om9_surface_refit_deviation().is_nan());
    assert!(!unsafe { surface_refit::om9_surface_refit(std::ptr::null(), 9, 0.1, false) });
    assert_eq!(spline_ffi::om9_spline_pole_count(), 0);
}
