//! OM9-MEASURE-001: angle between two directed lines defined by four points.
pub fn command(icon: &str) -> Option<&'static str> {
    (icon == "AnalyzeAngle").then_some("Angle")
}
pub fn angle(points: [[f64; 3]; 4]) -> Option<f64> {
    if points
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || v.abs() > 1e9)
    {
        return None;
    }
    let direction = |a: [f64; 3], b: [f64; 3]| {
        let delta = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let length = delta[0].hypot(delta[1]).hypot(delta[2]);
        (length > 1e-12).then(|| delta.map(|v| v / length))
    };
    let a = direction(points[0], points[1])?;
    let b = direction(points[2], points[3])?;
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let sine = cross[0].hypot(cross[1]).hypot(cross[2]);
    let cosine = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    Some(sine.atan2(cosine).to_degrees())
}

/// Four XYZ points, in first-start/end then second-start/end order.
/// Returns NaN for null pointer or invalid/degenerate lines.
/// # Safety
/// A non-null pointer must refer to 12 readable f64 values for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_measure_angle(points: *const f64) -> f64 {
    if points.is_null() {
        return f64::NAN;
    }
    let values = unsafe { std::slice::from_raw_parts(points, 12) };
    angle(std::array::from_fn(|i| {
        [values[i * 3], values[i * 3 + 1], values[i * 3 + 2]]
    }))
    .unwrap_or(f64::NAN)
}
