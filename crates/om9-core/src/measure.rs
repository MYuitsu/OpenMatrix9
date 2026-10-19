// SPDX-License-Identifier: LGPL-2.1-or-later
//! Read-only measurements: angle between two directed lines and distance between two world
//! points (M01 viewport analysis commands, wave W10).
//!
//! Ported from the legacy bridge (`rust/src/{core_angle,core_distance}.rs`). The C entry
//! points and the icon lookup stay behind; the portable arithmetic and its admission guards
//! are re-implemented with typed errors instead of `Option`/`NaN`.
use crate::{
    Error, Result,
    geometry::{Point3, Unit, finite},
};

/// The legacy magnitude bound: a coordinate beyond 1e9 is not a modelling coordinate.
pub const MAX_MEASURE_COORDINATE: f64 = 1e9;

fn admit(point: Point3) -> Result<()> {
    finite(point)?;
    if point.iter().any(|v| v.abs() > MAX_MEASURE_COORDINATE) {
        return Err(Error::Invalid("measurement coordinate"));
    }
    Ok(())
}

/// The undirected angle in **degrees** between the directed line `from_a -> to_a` and the
/// directed line `from_b -> to_b`. A zero-length line has no direction and is refused.
pub fn angle_degrees(from_a: Point3, to_a: Point3, from_b: Point3, to_b: Point3) -> Result<f64> {
    for point in [from_a, to_a, from_b, to_b] {
        admit(point)?;
    }
    let direction = |from: Point3, to: Point3| -> Result<Point3> {
        let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let length = delta[0].hypot(delta[1]).hypot(delta[2]);
        if !length.is_finite() || length <= 1e-12 {
            return Err(Error::Invalid("measurement direction"));
        }
        Ok([delta[0] / length, delta[1] / length, delta[2] / length])
    };
    let a = direction(from_a, to_a)?;
    let b = direction(from_b, to_b)?;
    // atan2(|a x b|, a . b) is stable near 0 and 180 degrees, unlike acos(a . b).
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let sine = cross[0].hypot(cross[1]).hypot(cross[2]);
    let cosine = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let degrees = sine.atan2(cosine).to_degrees();
    if !degrees.is_finite() {
        return Err(Error::Invalid("measurement angle"));
    }
    Ok(degrees)
}

/// The distance between two world points, expressed in `unit`.
pub fn distance(first: Point3, second: Point3, unit: Unit) -> Result<f64> {
    admit(first)?;
    admit(second)?;
    let millimeters = (second[0] - first[0])
        .hypot(second[1] - first[1])
        .hypot(second[2] - first[2]);
    let value = millimeters / unit.factor();
    if !value.is_finite() {
        return Err(Error::Invalid("measurement distance"));
    }
    Ok(value)
}

/// The distance in canonical millimetres: the measurement layer stores millimetres and the
/// display unit only scales the answer.
pub fn distance_mm(first: Point3, second: Point3) -> Result<f64> {
    distance(first, second, Unit::Millimeter)
}
