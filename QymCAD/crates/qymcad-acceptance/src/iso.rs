//! THE GROOVE OF AN ISO METRIC THREAD, worked out from the standard (ISO 68-1), to check what a thread takes from a
//! body against: the profile swept round by Pappus, radius by radius.
use std::f64::consts::PI;

/// THE GROOVE OF AN ISO METRIC THREAD over `length`, by Pappus: the width of the groove at each radius, swept round.
/// `width(r)` is the width along the axis of what the groove takes at radius r, within one pitch `p`.
pub fn groove(width: impl Fn(f64) -> f64, from: f64, to: f64, p: f64, length: f64) -> f64 {
    let n = 4000;
    let dr = (to - from) / n as f64;
    let per_pitch: f64 = (0..n).map(|i| from + (i as f64 + 0.5) * dr).map(|r| width(r).max(0.0) * 2.0 * PI * r * dr).sum();
    per_pitch * length / p
}

/// The groove of an outer thread of diameter `d` and pitch `p` (ISO 68-1): flanks at 30 degrees off the radius from
/// the crest flat of p / 8 at the major diameter, down to a root rounded at H / 6 at the diameter d - 1.226869 p.
pub fn outer_groove(d: f64, p: f64, length: f64) -> f64 {
    let h = p * 3f64.sqrt() / 2.0;
    let t30 = (PI / 6.0).tan();
    let r_root = d / 2.0 - 0.613_434 * p;
    let round = h / 6.0;
    let centre = r_root + round;
    let tangent = centre - round / 2.0;
    let width = |r: f64| {
        if r >= tangent {
            2.0 * ((p / 2.0 - p / 16.0) - (d / 2.0 - r) * t30)
        } else {
            2.0 * (round * round - (centre - r).powi(2)).max(0.0).sqrt()
        }
    };
    groove(width, r_root, d / 2.0, p, length)
}

/// The groove of an inner thread of major diameter `d` and pitch `p` in a hole of diameter `hole` (ISO 68-1): the
/// room the bolt's tooth takes - 3/4 of the pitch wide at the basic minor diameter d - 1.082532 p, narrowing to the
/// flat of p / 8 at the major diameter.
pub fn inner_groove(d: f64, p: f64, hole: f64, length: f64) -> f64 {
    let minor = d - 1.082_532 * p;
    let width = |r: f64| p * (0.75 - 0.625 * (r - minor / 2.0) / ((d - minor) / 2.0));
    groove(width, hole / 2.0, d / 2.0, p, length)
}
