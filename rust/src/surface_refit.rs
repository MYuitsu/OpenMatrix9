//! Tolerance-controlled Surface section refit candidates, in millimetres.
//!
//! Input points must be dense, equal arc-length samples of the original native
//! curve. The maximum deviation below is measured at those samples only; it is
//! not a curve-to-curve certificate. The native host must certify the final
//! candidate against the original geometry before accepting a Surface section.
use crate::spline::{self, Spline};
use std::sync::atomic::{AtomicU64, Ordering};

static DEVIATION: AtomicU64 = AtomicU64::new(f64::NAN.to_bits());

#[derive(Debug)]
pub struct RefitCandidate {
    pub curve: Spline,
    pub sample_deviation: f64,
    pub sample_count: usize,
}

/// Find a low-pole candidate, reserving three quarters of the requested tolerance
/// for the native host's continuous geometry certificate. Open endpoints are
/// fixed by the least-squares rebuild; closed input includes a repeated endpoint.
pub fn refit(samples: &[[f64; 3]], tolerance: f64, closed: bool) -> Result<RefitCandidate, String> {
    if !tolerance.is_finite() || tolerance <= 0. || tolerance > 1e6 {
        return Err(
            "Surface Refit tolerance must be finite, greater than zero and at most 1e6 mm".into(),
        );
    }
    if !(4..=spline::MAX_SAMPLES).contains(&samples.len()) {
        return Err("Surface Refit needs 4 to 4096 dense equal arc-length samples".into());
    }
    let limit = spline::MAX_POLES.min((samples.len() - usize::from(closed)) / 2);
    let mut count = if closed { 3 } else { 2 };
    let mut best = f64::INFINITY;
    let mut last_error = None;
    while count <= limit {
        match spline::rebuild(samples, count, 3.min(count - 1), closed) {
            Ok(curve) => {
                let deviation = samples
                    .iter()
                    .enumerate()
                    .map(|(i, &point)| {
                        spline::distance(point, curve.value(i as f64 / (samples.len() - 1) as f64))
                    })
                    .fold(0., f64::max);
                if deviation.is_finite() && deviation <= tolerance * 0.25 {
                    return Ok(RefitCandidate {
                        curve,
                        sample_deviation: deviation,
                        sample_count: samples.len(),
                    });
                }
                best = best.min(deviation);
            }
            Err(error) => last_error = Some(error),
        }
        if count == limit {
            break;
        }
        count = (if count < 4 { count + 1 } else { count * 2 }).min(limit);
    }
    if best.is_finite() {
        Err(format!(
            "Surface Refit cannot meet the sample candidate budget within {limit} poles: best sampled deviation {best} mm, budget {} mm; native geometry certification is still required",
            tolerance * 0.25
        ))
    } else {
        Err(last_error.unwrap_or_else(|| {
            "Surface Refit needs at least six distinct samples for a closed curve".into()
        }))
    }
}

/// # Safety
/// `xyz` points to `count*3` readable doubles for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_refit(
    xyz: *const f64,
    count: usize,
    tolerance: f64,
    closed: bool,
) -> bool {
    DEVIATION.store(f64::NAN.to_bits(), Ordering::Relaxed);
    if xyz.is_null() || !(4..=spline::MAX_SAMPLES).contains(&count) {
        return crate::spline_ffi::publish(Err("Invalid Surface Refit samples".into()));
    }
    let input = unsafe { std::slice::from_raw_parts(xyz, count * 3) };
    let points = input
        .chunks_exact(3)
        .map(|p| [p[0], p[1], p[2]])
        .collect::<Vec<_>>();
    match refit(&points, tolerance, closed) {
        Ok(candidate) => {
            let deviation = candidate.sample_deviation;
            let success = crate::spline_ffi::publish(Ok(candidate.curve));
            if success {
                DEVIATION.store(deviation.to_bits(), Ordering::Relaxed);
            }
            success
        }
        Err(error) => crate::spline_ffi::publish(Err(error)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_refit_deviation() -> f64 {
    f64::from_bits(DEVIATION.load(Ordering::Relaxed))
}
