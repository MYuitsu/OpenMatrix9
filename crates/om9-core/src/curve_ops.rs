// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve operations owned by OM9 (OM9-L2-02-04): evaluation, arc length, division,
//! split, trim, join, explode, close, seam, tangent direction and control grips.
//!
//! Everything here works on the portable [`Curve`] record and the shared evaluator.
//! A `Curve::Polycurve` has no single native parameter, so its parameter space is the
//! segment-uniform one already used by `surface_kernel::sample_curve`: global fraction
//! `f` maps to segment `floor(f * segments)` and the local fraction inside that segment.
//! The mapping is documented here because split/trim/divide must agree with sampling.
use crate::{
    Error, Result,
    budget::Limits,
    curve_build::{KnotMode, insert_knot, interpolate_any},
    evaluation::{Parameter, evaluate_curve},
    geometry::{Curve, NurbsCurve, Point3, Transform, finite},
};

fn identity() -> Transform {
    Transform::identity()
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn distance(a: Point3, b: Point3) -> f64 {
    let d = sub(a, b);
    dot(d, d).sqrt()
}

fn unit(a: Point3) -> Result<Point3> {
    let length = dot(a, a).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("curve tangent"));
    }
    Ok(scale(a, 1.0 / length))
}

fn check_fraction(fraction: f64) -> Result<()> {
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return Err(Error::Invalid("curve fraction"));
    }
    Ok(())
}

/// Evaluate `curve` at a global fraction, returning `D0..Dorder` in world coordinates.
///
/// The polycurve branch scales each local derivative by `segments^order`, because the
/// global fraction advances one segment per `1 / segments`. A reversed polycurve maps
/// `f -> 1 - f`, so odd derivatives change sign.
pub fn fraction_derivatives(
    curve: &Curve,
    fraction: f64,
    order: usize,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    check_fraction(fraction)?;
    curve.validate(limits)?;
    if order > 3 {
        return Err(Error::Unsupported("derivative order above D3"));
    }
    let Curve::Polycurve { segments, reversed } = curve else {
        return evaluate_curve(
            curve,
            Parameter::Fraction(fraction),
            order,
            identity(),
            limits,
        );
    };
    let count = segments.len();
    if count == 0 {
        return Err(Error::Invalid("polycurve segments"));
    }
    let forward = if *reversed { 1.0 - fraction } else { fraction };
    let scaled = (forward * count as f64).min(count as f64);
    let mut index = scaled.floor() as usize;
    if index >= count {
        index = count - 1;
    }
    let local = scaled - index as f64;
    let mut jet = fraction_derivatives(&segments[index], local, order, limits)?;
    if *reversed {
        for (k, value) in jet.iter_mut().enumerate().skip(1) {
            if k % 2 == 1 {
                *value = value.map(|v| -v);
            }
        }
    }
    let mut factor = 1.0;
    for value in jet.iter_mut().skip(1) {
        factor *= count as f64;
        *value = value.map(|v| v * factor);
    }
    Ok(jet)
}

/// Point at a global fraction.
pub fn point_at(curve: &Curve, fraction: f64, limits: &Limits) -> Result<Point3> {
    let jet = fraction_derivatives(curve, fraction, 0, limits)?;
    jet.first().copied().ok_or(Error::Invalid("curve point"))
}

/// First derivative with respect to the global fraction.
pub fn derivative_at(curve: &Curve, fraction: f64, limits: &Limits) -> Result<Point3> {
    let jet = fraction_derivatives(curve, fraction, 1, limits)?;
    jet.get(1)
        .copied()
        .ok_or(Error::Invalid("curve derivative"))
}

/// Unit tangent at a global fraction.
pub fn tangent_at(curve: &Curve, fraction: f64, limits: &Limits) -> Result<Point3> {
    match derivative_at(curve, fraction, limits) {
        Ok(tangent) => unit(tangent),
        // A periodic/clamped NURBS seam can refuse a derivative while the geometric tangent
        // is still well defined. Use a symmetric secant there instead of failing the frame.
        Err(Error::Unsupported(_)) => {
            let step = 1.0e-6;
            let low = (fraction - step).max(0.0);
            let high = (fraction + step).min(1.0);
            if high <= low {
                return Err(Error::Invalid("tangent interval"));
            }
            let start = point_at(curve, low, limits)?;
            let end = point_at(curve, high, limits)?;
            unit([end[0] - start[0], end[1] - start[1], end[2] - start[2]])
        }
        Err(error) => Err(error),
    }
}

/// The two geometric ends in forward order. A reversed curve reports the reversed ends.
pub fn curve_endpoints(curve: &Curve, limits: &Limits) -> Result<[Point3; 2]> {
    Ok([point_at(curve, 0.0, limits)?, point_at(curve, 1.0, limits)?])
}

/// Sample any curve at `count` stations, including polycurve segments and reversal.
pub fn sample_curve(curve: &Curve, count: usize, limits: &Limits) -> Result<Vec<Point3>> {
    if count < 2 {
        return Err(Error::Invalid("sample count"));
    }
    let mut points = Vec::new();
    points
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("curve samples"))?;
    for index in 0..count {
        let fraction = index as f64 / (count - 1) as f64;
        points.push(point_at(curve, fraction, limits)?);
    }
    Ok(points)
}

/* -------------------------------- length -------------------------------- */

fn speed(curve: &Curve, fraction: f64, limits: &Limits) -> Result<f64> {
    match derivative_at(curve, fraction, limits) {
        Ok(d) => {
            let value = dot(d, d).sqrt();
            if !value.is_finite() {
                return Err(Error::Invalid("curve speed"));
            }
            Ok(value)
        }
        // The evaluator refuses D1 at a knot whose multiplicity equals the degree
        // (a C0 parametric join, e.g. the exact rational-quadratic ellipse). The
        // geometric speed there is still well defined; measure it with a symmetric
        // secant instead of pretending the curve has no length.
        Err(Error::Unsupported(_)) => {
            let h = 1e-7;
            let low = (fraction - h).max(0.0);
            let high = (fraction + h).min(1.0);
            if high <= low {
                return Err(Error::Invalid("curve speed"));
            }
            let a = point_at(curve, low, limits)?;
            let b = point_at(curve, high, limits)?;
            Ok(distance(a, b) / (high - low))
        }
        Err(error) => Err(error),
    }
}

fn simpson(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (b - a) / 6.0 * (fa + 4.0 * fm + fb)
}

#[allow(clippy::too_many_arguments)]
fn adaptive(
    curve: &Curve,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    whole: f64,
    tolerance: f64,
    depth: usize,
    limits: &Limits,
) -> Result<f64> {
    let m = 0.5 * (a + b);
    let lm = 0.5 * (a + m);
    let rm = 0.5 * (m + b);
    let flm = speed(curve, lm, limits)?;
    let frm = speed(curve, rm, limits)?;
    let left = simpson(a, m, fa, flm, fm);
    let right = simpson(m, b, fm, frm, fb);
    let sum = left + right;
    if depth == 0 || (sum - whole).abs() <= 15.0 * tolerance {
        return Ok(sum + (sum - whole) / 15.0);
    }
    let left_value = adaptive(
        curve,
        a,
        m,
        fa,
        flm,
        fm,
        left,
        tolerance * 0.5,
        depth - 1,
        limits,
    )?;
    let right_value = adaptive(
        curve,
        m,
        b,
        fm,
        frm,
        fb,
        right,
        tolerance * 0.5,
        depth - 1,
        limits,
    )?;
    Ok(left_value + right_value)
}

/// Arc length from global fraction `from` to `to` (inclusive ends, `from <= to`).
pub fn length_between(curve: &Curve, from: f64, to: f64, limits: &Limits) -> Result<f64> {
    check_fraction(from)?;
    check_fraction(to)?;
    if from > to {
        return Err(Error::Invalid("curve length interval"));
    }
    curve.validate(limits)?;
    if from == to {
        return Ok(0.0);
    }
    if let Curve::Polycurve { segments, reversed } = curve {
        let (from, to) = if *reversed {
            (1.0 - to, 1.0 - from)
        } else {
            (from, to)
        };
        return polycurve_length(segments, from, to, limits);
    }
    let fa = speed(curve, from, limits)?;
    let fb = speed(curve, to, limits)?;
    let middle = 0.5 * (from + to);
    let fm = speed(curve, middle, limits)?;
    let whole = simpson(from, to, fa, fm, fb);
    let tolerance = 1e-10 * (1.0 + whole.abs());
    adaptive(curve, from, to, fa, fm, fb, whole, tolerance, 18, limits)
}

/// Segment-wise length of a forward polycurve, so a corner never sits inside an
/// integration interval (the speed is discontinuous there).
fn polycurve_length(segments: &[Curve], from: f64, to: f64, limits: &Limits) -> Result<f64> {
    let count = segments.len();
    if count == 0 {
        return Err(Error::Invalid("polycurve segments"));
    }
    let scaled_from = (from * count as f64).min(count as f64);
    let scaled_to = (to * count as f64).min(count as f64);
    let start_segment = (scaled_from.floor() as usize).min(count - 1);
    let end_segment = (scaled_to.floor() as usize).min(count - 1);
    let mut total = 0.0;
    for (index, segment) in segments
        .iter()
        .enumerate()
        .take(end_segment + 1)
        .skip(start_segment)
    {
        let low = if index == start_segment {
            scaled_from - index as f64
        } else {
            0.0
        };
        let high = if index == end_segment {
            scaled_to - index as f64
        } else {
            1.0
        };
        if high > low {
            total += length_between(segment, low, high, limits)?;
        }
    }
    Ok(total)
}

/// Whole curve length.
pub fn arc_length(curve: &Curve, limits: &Limits) -> Result<f64> {
    length_between(curve, 0.0, 1.0, limits)
}

/// Fraction whose prefix arc length is `length`, by bisection on the length integral.
pub fn parameter_at_length(curve: &Curve, length: f64, limits: &Limits) -> Result<f64> {
    let total = arc_length(curve, limits)?;
    if !length.is_finite() || length < 0.0 || length > total + 1e-9 {
        return Err(Error::Invalid("curve length target"));
    }
    if length <= 0.0 {
        return Ok(0.0);
    }
    if length >= total {
        return Ok(1.0);
    }
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..60 {
        let middle = 0.5 * (low + high);
        let measured = length_between(curve, 0.0, middle, limits)?;
        if measured < length {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok(0.5 * (low + high))
}

/// `count + 1` fractions from 0 to 1 spaced evenly in arc length.
pub fn divide_by_count(curve: &Curve, count: usize, limits: &Limits) -> Result<Vec<f64>> {
    if count == 0 {
        return Err(Error::Invalid("divide count"));
    }
    let total = arc_length(curve, limits)?;
    if !total.is_finite() || total <= 0.0 {
        return Err(Error::Invalid("curve length"));
    }
    let mut fractions = Vec::new();
    fractions
        .try_reserve_exact(count + 1)
        .map_err(|_| Error::Budget("divide fractions"))?;
    for index in 0..=count {
        fractions.push(parameter_at_length(
            curve,
            total * index as f64 / count as f64,
            limits,
        )?);
    }
    Ok(fractions)
}

/// Segment fractions produced by dividing at most `length` apart, starting at 0 and always
/// ending exactly at 1 (the last segment may be shorter than `length`).
pub fn divide_by_length(curve: &Curve, length: f64, limits: &Limits) -> Result<Vec<f64>> {
    if !length.is_finite() || length <= 0.0 {
        return Err(Error::Invalid("divide length"));
    }
    let total = arc_length(curve, limits)?;
    if !total.is_finite() || total <= 0.0 {
        return Err(Error::Invalid("curve length"));
    }
    let mut fractions = vec![0.0];
    let mut target = length;
    while target < total - 1e-9 {
        fractions.push(parameter_at_length(curve, target, limits)?);
        target += length;
    }
    fractions.push(1.0);
    Ok(fractions)
}

/* ------------------------------ reverse/seam ------------------------------ */

/// Geometry-reversed copy: line ends swap, circle orientation flips, NURBS toggles the
/// evaluator's `reversed` metadata, and a polycurve reverses order and members.
pub fn reverse_curve(curve: &Curve, limits: &Limits) -> Result<Curve> {
    curve.validate(limits)?;
    Ok(match curve {
        Curve::Line { start, end, domain } => {
            Curve::Line {
                start: *end,
                end: *start,
                domain: *domain,
            }
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            Curve::Circle {
                center: *center,
                x: *x,
                y: scale(*y, -1.0),
                radius: *radius,
                domain: *domain,
                angle_domain: Some([-angles[1], -angles[0]]),
            }
        }
        Curve::Nurbs(nurbs) => {
            let mut flipped = nurbs.clone();
            flipped.reversed = !flipped.reversed;
            Curve::Nurbs(flipped)
        }
        Curve::Polycurve { .. } => {
            let forward = forward_polycurve(curve, limits)?;
            let Curve::Polycurve { segments, .. } = forward else {
                return Err(Error::Invalid("polycurve"));
            };
            let mut reversed = Vec::new();
            reversed
                .try_reserve_exact(segments.len())
                .map_err(|_| Error::Budget("reversed polycurve"))?;
            for segment in segments.iter().rev() {
                reversed.push(reverse_curve(segment, limits)?);
            }
            Curve::Polycurve {
                segments: reversed,
                reversed: false,
            }
        }
    })
}

/// Apply a polycurve's `reversed` flag so the result traverses forward.
fn forward_polycurve(curve: &Curve, limits: &Limits) -> Result<Curve> {
    let Curve::Polycurve { segments, reversed } = curve else {
        return Ok(curve.clone());
    };
    if !*reversed {
        return Ok(curve.clone());
    }
    let mut forward = Vec::new();
    forward
        .try_reserve_exact(segments.len())
        .map_err(|_| Error::Budget("polycurve"))?;
    for segment in segments.iter().rev() {
        forward.push(reverse_curve(segment, limits)?);
    }
    Ok(Curve::Polycurve {
        segments: forward,
        reversed: false,
    })
}

/// True when the geometric ends coincide within `tolerance`.
pub fn is_closed(curve: &Curve, tolerance: f64, limits: &Limits) -> Result<bool> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(Error::Invalid("closure tolerance"));
    }
    if let Curve::Nurbs(n)=curve && n.seam.is_some() {
        let mut base=n.clone();base.seam=None;
        return is_closed(&Curve::Nurbs(base),tolerance,limits);
    }
    let [start, end] = curve_endpoints(curve, limits)?;
    Ok(distance(start, end) <= tolerance)
}

/// Validate a user-authored ring rail (`OM9-L3-04-02-004`).
///
/// A custom ring rail must be a closed, planar, non-self-intersecting curve with a usable
/// tangent at every station. The sweep frame transport is only defined for such a rail; an
/// open or self-intersecting rail is refused by name instead of producing a twisted or
/// non-periodic band.
pub fn validate_ring_rail(curve: &Curve, tolerance: f64, limits: &Limits) -> Result<()> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("ring rail tolerance"));
    }
    curve.validate(limits)?;
    if !is_closed(curve, tolerance, limits)? {
        return Err(Error::Invalid("ring rail not closed"));
    }
    let points = sample_curve(curve, 65, limits)?;
    if points.len() < 4 {
        return Err(Error::Invalid("ring rail stations"));
    }
    // Deterministic plane: the first non-collinear triple. A rail that has no such triple is
    // degenerate and cannot define a profile plane.
    let mut plane = None;
    'search: for i in 1..points.len() {
        for j in (i + 1)..points.len() {
            let first = [
                points[i][0] - points[0][0],
                points[i][1] - points[0][1],
                points[i][2] - points[0][2],
            ];
            let second = [
                points[j][0] - points[0][0],
                points[j][1] - points[0][1],
                points[j][2] - points[0][2],
            ];
            let normal = [
                first[1] * second[2] - first[2] * second[1],
                first[2] * second[0] - first[0] * second[2],
                first[0] * second[1] - first[1] * second[0],
            ];
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            if length > 1.0e-9 {
                plane = Some([normal[0] / length, normal[1] / length, normal[2] / length]);
                break 'search;
            }
        }
    }
    let normal = plane.ok_or(Error::Invalid("ring rail plane"))?;
    let origin = points[0];
    for point in &points {
        let distance = (point[0] - origin[0]) * normal[0]
            + (point[1] - origin[1]) * normal[1]
            + (point[2] - origin[2]) * normal[2];
        if distance.abs() > tolerance.max(1.0e-6) {
            return Err(Error::Invalid("ring rail not planar"));
        }
    }
    // Project once into the plane and refuse any crossing between non-adjacent edges. The
    // closed rail's last station repeats the first, so the last segment closes the loop.
    let u = unit([
        points[1][0] - origin[0],
        points[1][1] - origin[1],
        points[1][2] - origin[2],
    ])?;
    let v = unit([
        normal[1] * u[2] - normal[2] * u[1],
        normal[2] * u[0] - normal[0] * u[2],
        normal[0] * u[1] - normal[1] * u[0],
    ])?;
    let flat: Vec<[f64; 2]> = points
        .iter()
        .map(|point| {
            let offset = [
                point[0] - origin[0],
                point[1] - origin[1],
                point[2] - origin[2],
            ];
            [
                offset[0] * u[0] + offset[1] * u[1] + offset[2] * u[2],
                offset[0] * v[0] + offset[1] * v[1] + offset[2] * v[2],
            ]
        })
        .collect();
    let edges = flat.len() - 1;
    for first in 0..edges {
        let a = flat[first];
        let b = flat[first + 1];
        if distance_2d(a, b) <= tolerance * 0.5 {
            return Err(Error::Invalid("ring rail zero-length segment"));
        }
        for second in (first + 1)..edges {
            if second == first + 1 || (first == 0 && second == edges - 1) {
                continue;
            }
            let c = flat[second];
            let d = flat[second + 1];
            if segments_cross(a, b, c, d, tolerance) {
                return Err(Error::Invalid("ring rail self-intersection"));
            }
        }
    }
    Ok(())
}

/// Merge compatible profile sections into one curve (`OM9-L3-04-02-006`).
///
/// Sections must be coplanar, consistently wound and connected end to end. The join is the
/// same `join_curves` transport used elsewhere, so continuity/tolerance rules stay in one
/// place. A section with the opposite winding is refused by name rather than silently
/// reversed, because reversing a profile changes the sweep's inside/outside.
pub fn merge_profile_sections(curves: &[Curve], tolerance: f64, limits: &Limits) -> Result<Curve> {
    if curves.len() < 2 {
        return Err(Error::Invalid("profile merge needs two sections"));
    }
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("profile merge tolerance"));
    }
    let mut reference: Option<(Point3, Point3, Point3, Point3, f64)> = None;
    for curve in curves {
        curve.validate(limits)?;
        let points = sample_curve(curve, 33, limits)?;
        let (origin, normal) = profile_plane(&points)?;
        let plane_tolerance = tolerance.max(1.0e-3);
        for point in &points {
            let distance = (point[0] - origin[0]) * normal[0]
                + (point[1] - origin[1]) * normal[1]
                + (point[2] - origin[2]) * normal[2];
            if distance.abs() > plane_tolerance {
                return Err(Error::Invalid("profile section not planar"));
            }
        }
        match &reference {
            None => {
                let u = plane_axis(normal)?;
                let v = unit(cross(normal, u))?;
                let area = signed_area(&points, origin, u, v);
                if !area.is_finite() || area.abs() <= 1.0e-9 {
                    return Err(Error::Invalid("profile section area"));
                }
                reference = Some((origin, u, v, normal, area.signum()));
            }
            Some((first_origin, first_u, first_v, first_normal, first_sign)) => {
                let alignment = normal[0] * first_normal[0]
                    + normal[1] * first_normal[1]
                    + normal[2] * first_normal[2];
                if alignment.abs() < 0.99 {
                    return Err(Error::Invalid("profile section planes"));
                }
                // Measure both sections in the first section's frame. A per-section basis
                // would flip twice for a reversed loop and hide the winding change.
                let area = signed_area(&points, *first_origin, *first_u, *first_v);
                if area.signum() != *first_sign {
                    return Err(Error::Invalid("profile section winding"));
                }
            }
        }
    }
    join_curves(curves, tolerance, limits)
}

/// A deterministic in-plane axis for the winding frame.
fn plane_axis(normal: Point3) -> Result<Point3> {
    for candidate in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
        let projection = [
            candidate[0] - normal[0] * dot(candidate, normal),
            candidate[1] - normal[1] * dot(candidate, normal),
            candidate[2] - normal[2] * dot(candidate, normal),
        ];
        let length = projection[0].hypot(projection[1]).hypot(projection[2]);
        if length > 1.0e-9 {
            return Ok([
                projection[0] / length,
                projection[1] / length,
                projection[2] / length,
            ]);
        }
    }
    Err(Error::Invalid("profile section plane"))
}

/// The plane of a sampled loop: origin plus unit normal. Deterministic first non-collinear
/// triple, matching the ring-rail validator.
fn profile_plane(points: &[Point3]) -> Result<(Point3, Point3)> {
    for i in 1..points.len() {
        for j in (i + 1)..points.len() {
            let first = [
                points[i][0] - points[0][0],
                points[i][1] - points[0][1],
                points[i][2] - points[0][2],
            ];
            let second = [
                points[j][0] - points[0][0],
                points[j][1] - points[0][1],
                points[j][2] - points[0][2],
            ];
            let normal = cross(first, second);
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            if length > 1.0e-9 {
                return Ok((
                    points[0],
                    [normal[0] / length, normal[1] / length, normal[2] / length],
                ));
            }
        }
    }
    Err(Error::Invalid("profile section plane"))
}

/// Signed area of the sampled polygon in the section plane.
fn signed_area(points: &[Point3], origin: Point3, u: Point3, v: Point3) -> f64 {
    let mut area = 0.0;
    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let ax = (a[0] - origin[0]) * u[0] + (a[1] - origin[1]) * u[1] + (a[2] - origin[2]) * u[2];
        let ay = (a[0] - origin[0]) * v[0] + (a[1] - origin[1]) * v[1] + (a[2] - origin[2]) * v[2];
        let bx = (b[0] - origin[0]) * u[0] + (b[1] - origin[1]) * u[1] + (b[2] - origin[2]) * u[2];
        let by = (b[0] - origin[0]) * v[0] + (b[1] - origin[1]) * v[1] + (b[2] - origin[2]) * v[2];
        area += ax * by - bx * ay;
    }
    area * 0.5
}

fn distance_2d(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn orientation_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn segments_cross(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], tolerance: f64) -> bool {
    let o1 = orientation_2d(a, b, c);
    let o2 = orientation_2d(a, b, d);
    let o3 = orientation_2d(c, d, a);
    let o4 = orientation_2d(c, d, b);
    let scale = tolerance.max(1.0e-9);
    if o1 * o2 < -scale && o3 * o4 < -scale {
        return true;
    }
    // A crossing exactly at a sample (the bow-tie case) gives one orientation of zero. The
    // strict sign test misses it, so accept a point that lies on the other segment too.
    (o1.abs() <= scale && point_on_segment(c, a, b, tolerance))
        || (o2.abs() <= scale && point_on_segment(d, a, b, tolerance))
        || (o3.abs() <= scale && point_on_segment(a, c, d, tolerance))
        || (o4.abs() <= scale && point_on_segment(b, c, d, tolerance))
}

fn point_on_segment(point: [f64; 2], a: [f64; 2], b: [f64; 2], tolerance: f64) -> bool {
    let cross = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
    let scale = tolerance.max(1.0e-9);
    if cross.abs() > scale {
        return false;
    }
    point[0] >= a[0].min(b[0]) - tolerance
        && point[0] <= a[0].max(b[0]) + tolerance
        && point[1] >= a[1].min(b[1]) - tolerance
        && point[1] <= a[1].max(b[1]) + tolerance
}

/// Rotate the start of a closed curve to a new seam fraction.
pub fn set_seam(curve: &Curve, fraction: f64, tolerance: f64, limits: &Limits) -> Result<Curve> {
    check_fraction(fraction)?;
    if fraction <= 0.0 || fraction >= 1.0 {
        return Err(Error::Invalid("seam fraction"));
    }
    if !is_closed(curve, tolerance, limits)? {
        return Err(Error::Invalid("seam on an open curve"));
    }
    match curve {
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            let angle = angles[0] + fraction * (angles[1] - angles[0]);
            Ok(Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: *radius,
                domain: [0.0, 1.0],
                angle_domain: Some([angle, angle + (angles[1] - angles[0])]),
            })
        }
        Curve::Nurbs(nurbs) => {
            let mut shifted = nurbs.clone();
            let [start,end]=nurbs.domain; let span=end-start;
            let origin=nurbs.seam.map_or(if nurbs.reversed {1.} else {0.}, |seam| (seam-start)/span);
            let next=(origin+if nurbs.reversed {-fraction} else {fraction}).rem_euclid(1.);
            shifted.seam = Some(start+next*span);
            Ok(Curve::Nurbs(shifted))
        }
        Curve::Polycurve { .. } => {
            let (left, right) = split_curve(curve, fraction, limits)?;
            join_curves(&[right, left], tolerance, limits)
        }
        Curve::Line { .. } => Err(Error::Unsupported("seam on a line")),
    }
}

/* ------------------------------ split / trim ------------------------------ */

/// Materialize an active interval with the requested endpoint clamped, without refitting.
pub(crate) fn clamped_active_nurbs(n: &NurbsCurve, start: bool, limits: &Limits) -> Result<NurbsCurve> {
    n.validate(limits)?;
    let support = [n.knots[n.degree], n.knots[n.poles.len()]];
    if n.periodic { n.certify_repeated_periodic_basis(limits)?; }
    let mut active = n.clone();
    active.domain = support;
    active.periodic = false;
    active.reversed = false;
    active.seam = None;
    if n.domain[0] > support[0] {
        let first=active.knots.partition_point(|k| *k < n.domain[0]);
        let last=active.knots.partition_point(|k| *k <= n.domain[0]);
        if last-first == active.degree+1 {
            // At a full-multiplicity knot evaluation chooses the right side.
            // Retain that side directly; splitting at the jump cannot share a pole.
            active.poles=active.poles[first..].to_vec();
            active.weights=active.weights[first..].to_vec();
            active.knots=active.knots[first..].to_vec();
            active.domain[0]=n.domain[0];
            active.validate(limits)?;
        } else {
            let (_, Curve::Nurbs(right)) = split_nurbs(&active, n.domain[0], limits)? else { unreachable!() };
            active = right;
        }
    }
    if n.domain[1] < support[1] {
        let (Curve::Nurbs(left), _) = split_nurbs(&active, n.domain[1], limits)? else { unreachable!() };
        active = left;
    }
    let lower = start != n.reversed;
    let unclamped = if lower {
        active.knots[..=active.degree].iter().any(|k| *k != active.domain[0])
    } else {
        active.knots[active.knots.len() - active.degree - 1..].iter().any(|k| *k != active.domain[1])
    };
    if unclamped {
        let parameter = active.domain[usize::from(!lower)];
        let first = active.knots.partition_point(|k| *k < parameter);
        let last = active.knots.partition_point(|k| *k <= parameter);
        let multiplicity = last - first;
        let inserted = crate::curve_build::insert_support_knot(
            &active, parameter, active.degree.saturating_sub(multiplicity), limits,
        )?;
        let first = inserted.knots.partition_point(|k| *k < parameter);
        let last = inserted.knots.partition_point(|k| *k <= parameter);
        if lower {
            let offset = last - 1 - inserted.degree;
            active.poles = inserted.poles[offset..].to_vec();
            active.weights = inserted.weights[offset..].to_vec();
            active.knots = vec![parameter; inserted.degree + 1];
            active.knots.extend_from_slice(&inserted.knots[last..]);
        } else {
            active.poles = inserted.poles[..first].to_vec();
            active.weights = inserted.weights[..first].to_vec();
            active.knots = inserted.knots[..first].to_vec();
            active.knots.extend(std::iter::repeat_n(parameter, inserted.degree + 1));
        }
    }
    active.reversed = n.reversed;
    active.seam = n.seam;
    active.validate(limits)?;
    Ok(active)
}

#[cfg(test)]
mod join_active_interval_tests {
    use super::*;

    #[test]
    fn rational_multispan_restriction_preserves_active_curve_and_orientation() {
        let limits = Limits::default();
        for domain in [[0.1, 0.9], [0.0, 0.7], [0.3, 1.0]] {
            for reversed in [false, true] {
                let n = NurbsCurve {
                    degree: 2,
                    poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 0.], [3., 0., 0.]],
                    weights: vec![1., 0.7, 1.4, 0.9],
                    knots: vec![0., 0., 0., 0.5, 1., 1., 1.],
                    domain, periodic: false, reversed, seam: None,
                };
                let restricted = clamped_active_nurbs(&n, true, &limits).unwrap();
                assert_eq!(restricted.domain, domain);
                assert_eq!(restricted.reversed, reversed);
                for i in 0..=100 {
                    let t = i as f64 / 100.;
                    let a = point_at(&Curve::Nurbs(n.clone()), t, &limits).unwrap();
                    let b = point_at(&Curve::Nurbs(restricted.clone()), t, &limits).unwrap();
                    assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-10), "{domain:?} {reversed} {t}: {a:?} {b:?}");
                }
            }
        }
    }

    #[test]
    fn trimmed_nonclamped_backing_preserves_curve() {
        let n = NurbsCurve {
            degree: 2, poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 0.], [3., 0., 0.]],
            weights: vec![1.; 4], knots: vec![-1., -0.5, 0., 0.5, 1., 1.5, 2.],
            domain: [0.1, 0.9], periodic: false, reversed: false, seam: None,
        };
        let limits = Limits::default();
        for reversed in [false, true] {
            let mut source = n.clone();
            source.reversed = reversed;
            let restricted = clamped_active_nurbs(&source, true, &limits).unwrap();
            for i in 0..=100 {
                let t = i as f64 / 100.;
                let a = point_at(&Curve::Nurbs(source.clone()), t, &limits).unwrap();
                let b = point_at(&Curve::Nurbs(restricted.clone()), t, &limits).unwrap();
                assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-10));
            }
        }
    }

    #[test]
    fn only_the_endpoint_being_adjusted_needs_to_be_clamped() {
        let limits = Limits::default();
        for reversed in [false, true] {
            for domain in [[0., 0.9], [0.1, 1.]] {
                let n = NurbsCurve {
                    degree: 2, poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 0.], [3., 0., 0.]],
                    weights: vec![1., 0.7, 1.4, 0.9], knots: vec![-1., -0.5, 0., 0.5, 1., 1.5, 2.],
                    domain, periodic: false, reversed, seam: None,
                };
                let lower_clamped = domain[0] > 0.;
                let start = lower_clamped != reversed;
                let restricted = clamped_active_nurbs(&n, start, &limits).unwrap();
                let other = clamped_active_nurbs(&n, !start, &limits).unwrap();
                for i in 0..=100 {
                    let t = i as f64 / 100.;
                    let a = point_at(&Curve::Nurbs(n.clone()), t, &limits).unwrap();
                    let b = point_at(&Curve::Nurbs(restricted.clone()), t, &limits).unwrap();
                    let c = point_at(&Curve::Nurbs(other.clone()), t, &limits).unwrap();
                    assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-10));
                    assert!(a.iter().zip(c).all(|(x, y)| (x - y).abs() < 1e-10));
                }
            }
        }
    }

    #[test]
    fn support_boundary_clamping_preserves_rational_curves() {
        let limits = Limits::default();
        let fixtures = [
            NurbsCurve {
                degree: 2, poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 0.], [3., 0., 0.]],
                weights: vec![1., 0.7, 1.4, 0.9], knots: vec![-1., -0.5, 0., 0.5, 1., 1.5, 2.],
                domain: [0., 1.], periodic: false, reversed: false, seam: None,
            },
            NurbsCurve {
                degree: 3, poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 1.], [3., 0., 0.], [4., 3., 0.], [5., 1., 0.]],
                weights: vec![1., 0.7, 1.4, 0.9, 1.2, 0.8],
                knots: vec![-2., -1., 0., 0., 0.5, 1., 1.5, 2., 3., 4.],
                domain: [0., 1.5], periodic: false, reversed: false, seam: None,
            },
            NurbsCurve {
                degree: 2, poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 0.], [3., 0., 0.], [4., 1., 0.]],
                weights: vec![1., 0.7, 1.4, 0.9, 1.2], knots: vec![-1., 0., 0., 0., 0.5, 1., 1., 1.],
                domain: [0., 1.], periodic: false, reversed: false, seam: None,
            },
            NurbsCurve {
                degree: 2, poles: vec![[0., 0., 0.], [1., 2., 0.], [2., -1., 0.], [3., 0., 0.], [4., 1., 0.]],
                weights: vec![1., 0.7, 1.4, 0.9, 1.2], knots: vec![-1., -0.5, 0., 1., 1., 1., 1.5, 2.],
                domain: [0., 1.], periodic: false, reversed: false, seam: None,
            },
        ];
        for mut n in fixtures {
            for reversed in [false, true] {
                n.reversed = reversed;
                for start in [false, true] {
                    let restricted = clamped_active_nurbs(&n, start, &limits).unwrap();
                    let lower = start != reversed;
                    let boundary = n.domain[usize::from(!lower)];
                    let knots = if lower { &restricted.knots[..=n.degree] }
                        else { &restricted.knots[restricted.knots.len()-n.degree-1..] };
                    assert!(knots.iter().all(|k| *k == boundary));
                    for i in 0..=200 {
                        let t = i as f64 / 200.;
                        let a = point_at(&Curve::Nurbs(n.clone()), t, &limits).unwrap();
                        let b = point_at(&Curve::Nurbs(restricted.clone()), t, &limits).unwrap();
                        assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-10), "{reversed} {start} {t}: {a:?} {b:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn certified_periodic_curve_can_be_opened_at_either_seam_without_refitting() {
        let limits = Limits::default();
        let mut n = NurbsCurve {
            degree: 2,
            poles: vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],
            weights: vec![1.,0.7,1.2,0.9,1.,0.7],
            knots: vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],
            domain: [0.,4.], periodic: true, reversed: false, seam: None,
        };
        n.certify_repeated_periodic(&limits).unwrap();
        for reversed in [false, true] {
            n.reversed = reversed;
            for start in [false, true] {
                let opened = clamped_active_nurbs(&n, start, &limits).unwrap();
                assert!(!opened.periodic);
                for i in 0..=200 {
                    let t=i as f64/200.;
                    let a=point_at(&Curve::Nurbs(n.clone()),t,&limits).unwrap();
                    let b=point_at(&Curve::Nurbs(opened.clone()),t,&limits).unwrap();
                    assert!(a.iter().zip(b).all(|(x,y)| (x-y).abs()<1e-10), "{reversed} {start} {t}: {a:?} {b:?}");
                }
            }
        }
        n.poles[5][0] += 1.;
        assert!(matches!(clamped_active_nurbs(&n, true, &limits), Err(Error::Unsupported(_))));
    }

    #[test]
    fn trimmed_periodic_curve_retains_its_exact_active_interval() {
        let limits=Limits::default();
        let mut n=NurbsCurve {
            degree:2, poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],
            weights:vec![1.,0.7,1.2,0.9,1.,0.7], knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],
            domain:[0.,4.], periodic:true, reversed:false, seam:None,
        };
        for domain in [[0.3,3.7],[0.,2.6],[0.4,4.]] {
            n.domain=domain;
            for reversed in [false,true] {
                n.reversed=reversed;
                for start in [false,true] {
                    let opened=clamped_active_nurbs(&n,start,&limits).unwrap();
                    assert_eq!(opened.domain,domain);
                    assert!(!opened.periodic);
                    for i in 0..=200 {
                        let t=i as f64/200.;
                        let a=point_at(&Curve::Nurbs(n.clone()),t,&limits).unwrap();
                        let b=point_at(&Curve::Nurbs(opened.clone()),t,&limits).unwrap();
                        assert!(a.iter().zip(b).all(|(x,y)| (x-y).abs()<1e-10), "{domain:?} {reversed} {start} {t}: {a:?} {b:?}");
                    }
                }
            }
        }
        n.domain=[0.3,3.7];
        n.reversed=false;
        let curve=Curve::Nurbs(n.clone());
        assert!(crate::evaluation::evaluate_curve(&curve,Parameter::Fraction(0.),2,identity(),&limits).is_ok());
        assert!(crate::evaluation::evaluate_curve(&curve,Parameter::Fraction(1.),2,identity(),&limits).is_ok());
        assert!(n.certify_repeated_periodic(&limits).is_err());
        n.domain=[0.,2.7];
        assert!(crate::evaluation::evaluate_curve(&Curve::Nurbs(n),Parameter::Fraction(0.),2,identity(),&limits).is_err());
    }

    #[test]
    fn trim_start_at_discontinuous_knot_keeps_the_right_hand_piece() {
        let limits=Limits::default();
        let mut n=NurbsCurve {
            degree:2,
            poles:vec![[0.,0.,0.],[1.,2.,0.],[2.,0.,0.],[10.,0.,0.],[11.,-1.,0.],[12.,0.,0.]],
            weights:vec![1.,0.7,1.2,0.8,1.1,1.],
            knots:vec![0.,0.,0.,0.5,0.5,0.5,1.,1.,1.],
            domain:[0.5,0.9],periodic:false,reversed:false,seam:None,
        };
        for reversed in [false,true] {
            n.reversed=reversed;
            for start in [false,true] {
                let active=clamped_active_nurbs(&n,start,&limits).unwrap();
                for i in 0..=100 {
                    let t=i as f64/100.;
                    let a=point_at(&Curve::Nurbs(n.clone()),t,&limits).unwrap();
                    let b=point_at(&Curve::Nurbs(active.clone()),t,&limits).unwrap();
                    assert!(a.iter().zip(b).all(|(x,y)| (x-y).abs()<1e-10), "{reversed} {start} {t}: {a:?} {b:?}");
                }
            }
        }
        n.domain=[0.1,0.5];
        assert!(matches!(clamped_active_nurbs(&n,true,&limits),Err(Error::Unsupported(_))));
    }
}

fn split_nurbs(nurbs: &NurbsCurve, parameter: f64, limits: &Limits) -> Result<(Curve, Curve)> {
    if nurbs.reversed {
        return Err(Error::Unsupported("split of a reversed NURBS record"));
    }
    let [start, end] = nurbs.domain;
    if !parameter.is_finite() || parameter <= start || parameter >= end {
        return Err(Error::Invalid("split parameter"));
    }
    let multiplicity = nurbs
        .knots
        .iter()
        .filter(|value| **value == parameter)
        .count();
    if multiplicity > nurbs.degree {
        return Err(Error::Unsupported("discontinuous split parameter"));
    }
    let splits = nurbs.degree - multiplicity;
    let inserted = if splits == 0 {
        nurbs.clone()
    } else {
        insert_knot(nurbs, parameter, splits, limits)?
    };
    // The split parameter now has multiplicity `degree`; its last occurrence is `k`.
    let k = inserted
        .knots
        .iter()
        .rposition(|value| *value == parameter)
        .ok_or(Error::Invalid("split knot"))?;
    let left_poles = &inserted.poles[..=k - inserted.degree];
    let mut left_knots = inserted.knots[..=k].to_vec();
    left_knots.push(parameter);
    let right_poles = &inserted.poles[k - inserted.degree..];
    let mut right_knots = vec![parameter; inserted.degree + 1];
    right_knots.extend_from_slice(&inserted.knots[k + 1..]);
    let left = NurbsCurve {
        degree: inserted.degree,
        poles: left_poles.to_vec(),
        weights: inserted.weights[..=k - inserted.degree].to_vec(),
        knots: left_knots,
        domain: [start, parameter],
        periodic: false,
        reversed: false,
        seam: None,
    };
    let right = NurbsCurve {
        degree: inserted.degree,
        poles: right_poles.to_vec(),
        weights: inserted.weights[k - inserted.degree..].to_vec(),
        knots: right_knots,
        domain: [parameter, end],
        periodic: false,
        reversed: false,
        seam: None,
    };
    left.validate(limits)?;
    right.validate(limits)?;
    Ok((Curve::Nurbs(left), Curve::Nurbs(right)))
}

/// Split `curve` at a global fraction into forward `(left, right)` curves.
pub fn split_curve(curve: &Curve, fraction: f64, limits: &Limits) -> Result<(Curve, Curve)> {
    check_fraction(fraction)?;
    if fraction <= 0.0 || fraction >= 1.0 {
        return Err(Error::Invalid("split fraction"));
    }
    curve.validate(limits)?;
    match curve {
        Curve::Line { start, end, domain } => {
            let point = point_at(curve, fraction, limits)?;
            let left_length = (domain[1] - domain[0]) * fraction;
            let right_length = (domain[1] - domain[0]) * (1.0 - fraction);
            Ok((
                Curve::Line {
                    start: *start,
                    end: point,
                    domain: [0.0, left_length],
                },
                Curve::Line {
                    start: point,
                    end: *end,
                    domain: [0.0, right_length],
                },
            ))
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            let angle = angles[0] + fraction * (angles[1] - angles[0]);
            Ok((
                Curve::Circle {
                    center: *center,
                    x: *x,
                    y: *y,
                    radius: *radius,
                    domain: [0.0, 1.0],
                    angle_domain: Some([angles[0], angle]),
                },
                Curve::Circle {
                    center: *center,
                    x: *x,
                    y: *y,
                    radius: *radius,
                    domain: [0.0, 1.0],
                    angle_domain: Some([angle, angles[1]]),
                },
            ))
        }
        Curve::Nurbs(nurbs) => {
            if nurbs.seam.is_some() {
                // A seam-spanning polycurve has segment-uniform parameters and
                // cannot preserve this API's affine subcurve parameterization.
                return Err(Error::Unsupported("split of a reseamed NURBS; use parameter trim"));
            }
            if nurbs.periodic {
                // Materialize the certified active interval before splitting: the
                // repeated periodic basis must not be treated as a clamped one.
                let opened = clamped_active_nurbs(nurbs, true, limits)?;
                let opened = clamped_active_nurbs(&opened, false, limits)?;
                return split_curve(&Curve::Nurbs(opened), fraction, limits);
            }
            if nurbs.reversed {
                let reversed = reverse_curve(curve, limits)?;
                let (left, right) = split_curve(&reversed, 1.0 - fraction, limits)?;
                return Ok((
                    reverse_curve(&right, limits)?,
                    reverse_curve(&left, limits)?,
                ));
            }
            split_nurbs(
                nurbs,
                nurbs.domain[0] + fraction * (nurbs.domain[1] - nurbs.domain[0]),
                limits,
            )
        }
        Curve::Polycurve { segments, reversed } => {
            if *reversed {
                let reversed = reverse_curve(curve, limits)?;
                let (left, right) = split_curve(&reversed, 1.0 - fraction, limits)?;
                return Ok((
                    reverse_curve(&right, limits)?,
                    reverse_curve(&left, limits)?,
                ));
            }
            let count = segments.len();
            let scaled = fraction * count as f64;
            let mut index = scaled.floor() as usize;
            let local = scaled - index as f64;
            if index >= count {
                index = count - 1;
            }
            let mut left_segments: Vec<Curve> = Vec::new();
            let mut right_segments: Vec<Curve> = Vec::new();
            left_segments
                .try_reserve_exact(count + 1)
                .map_err(|_| Error::Budget("split polycurve"))?;
            right_segments
                .try_reserve_exact(count + 1)
                .map_err(|_| Error::Budget("split polycurve"))?;
            left_segments.extend(segments[..index].iter().cloned());
            right_segments.extend(segments[index + 1..].iter().cloned());
            if local <= 1e-12 {
                right_segments.insert(0, segments[index].clone());
            } else if local >= 1.0 - 1e-12 {
                left_segments.push(segments[index].clone());
            } else {
                let (left_part, right_part) = split_curve(&segments[index], local, limits)?;
                left_segments.push(left_part);
                right_segments.insert(0, right_part);
            }
            Ok((
                Curve::Polycurve {
                    segments: left_segments,
                    reversed: false,
                },
                Curve::Polycurve {
                    segments: right_segments,
                    reversed: false,
                },
            ))
        }
    }
}

/// Keep the global fraction range `[from, to]`; both ends must be strictly inside or at 0/1.
pub fn trim_curve(curve: &Curve, from: f64, to: f64, limits: &Limits) -> Result<Curve> {
    check_fraction(from)?;
    check_fraction(to)?;
    if from >= to {
        return Err(Error::Invalid("trim interval"));
    }
    if from == 0.0 && to == 1.0 {
        return Ok(curve.clone());
    }
    if let Curve::Nurbs(n)=curve && n.seam.is_some() {
        return trim_seamed_nurbs(n,from,to,limits);
    }
    if let Curve::Polycurve { segments, reversed } = curve {
        if *reversed {
            let forward = forward_polycurve(curve, limits)?;
            return trim_curve(&forward, from, to, limits);
        }
        let total = polycurve_length(segments, 0.0, 1.0, limits)?;
        return trim_polycurve_by_length(segments, from * total, to * total, limits);
    }
    let (head, _) = if to < 1.0 {
        split_curve(curve, to, limits)?
    } else {
        (curve.clone(), curve.clone())
    };
    if from == 0.0 {
        return Ok(head);
    }
    let (_, tail) = split_curve(&head, from / to, limits)?;
    Ok(tail)
}

/// Trim using the same segment-uniform parameter fractions as `point_at` and `split_curve`.
/// This differs from `trim_curve`'s historical polycurve arc-length fractions.
pub fn trim_curve_by_parameter(
    curve: &Curve,
    from: f64,
    to: f64,
    limits: &Limits,
) -> Result<Curve> {
    check_fraction(from)?;
    check_fraction(to)?;
    if from >= to {
        return Err(Error::Invalid("trim interval"));
    }
    if from == 0.0 && to == 1.0 {
        return Ok(curve.clone());
    }
    if let Curve::Nurbs(n)=curve && n.seam.is_some() {
        return trim_seamed_nurbs(n,from,to,limits);
    }
    if let Curve::Polycurve { segments, reversed } = curve {
        if *reversed {
            return trim_curve_by_parameter(&forward_polycurve(curve, limits)?, from, to, limits);
        }
        return trim_polycurve_by_parameter(segments, from, to, limits);
    }
    trim_curve(curve, from, to, limits)
}

fn trim_seamed_nurbs(n:&NurbsCurve,from:f64,to:f64,limits:&Limits)->Result<Curve> {
    n.validate(limits)?;
    let mut base=n.clone();base.seam=None;
    let curve=Curve::Nurbs(base);
    if !is_closed(&curve,1e-9,limits)? {return Err(Error::Invalid("NURBS seam needs a closed curve"));}
    let [start,end]=n.domain;let seam=n.seam.ok_or(Error::Missing("curve seam"))?;
    let offset=if n.reversed {(end-seam)/(end-start)} else {(seam-start)/(end-start)};
    let lo=offset+from;let hi=offset+to;
    let mut pieces=Vec::with_capacity(2);
    for turn in 0..=1 {
        let a=(lo-turn as f64).clamp(0.,1.);let b=(hi-turn as f64).clamp(0.,1.);
        if b>a {pieces.push(trim_curve_by_parameter(&curve,a,b,limits)?);}
    }
    if pieces.len()==1 {return Ok(pieces.remove(0));}
    let split=(1.0-lo)/(to-from);
    if let Some(curve)=join_parameterized_nurbs(&pieces,split,limits)? {return Ok(curve);}
    join_curves(&pieces,1e-9,limits)
}

// Combine two exact clamped NURBS using the original fraction at their joint.
fn join_parameterized_nurbs(pieces:&[Curve],split:f64,limits:&Limits)->Result<Option<Curve>> {
    if pieces.len()!=2 || !(0.0..1.0).contains(&split) {return Ok(None);}
    let mut parts=Vec::with_capacity(2);
    for piece in pieces {
        let Curve::Nurbs(n)=piece else {return Ok(None)};
        let mut n=n.clone();
        if n.reversed {
            n.poles.reverse();n.weights.reverse();
            n.knots=n.knots.iter().rev().map(|k|n.domain[0]+n.domain[1]-k).collect();
            n.reversed=false;
        }
        if n.periodic || n.seam.is_some() || n.knots[..=n.degree].iter().any(|k|*k!=n.domain[0]) || n.knots[n.knots.len()-n.degree-1..].iter().any(|k|*k!=n.domain[1]) {return Ok(None);}
        parts.push(n);
    }
    let (a,b)=(&parts[0],&parts[1]);
    if a.degree!=b.degree || a.poles.last()!=b.poles.first() {return Ok(None);}
    let poles_count=a.poles.len().checked_add(b.poles.len()-1).ok_or(Error::Overflow("joined NURBS poles"))?;
    let knots_count=poles_count.checked_add(a.degree+1).ok_or(Error::Overflow("joined NURBS knots"))?;
    let mut usage=crate::budget::Usage::array::<Point3>(poles_count)?;usage.points=poles_count as u64;
    usage.add(crate::budget::Usage::array::<f64>(poles_count)?,limits)?;usage.add(crate::budget::Usage::array::<f64>(knots_count)?,limits)?;
    let mut poles=Vec::new();let mut weights=Vec::new();let mut knots=Vec::new();
    poles.try_reserve_exact(poles_count).map_err(|_|Error::Budget("joined NURBS poles"))?;
    weights.try_reserve_exact(poles_count).map_err(|_|Error::Budget("joined NURBS weights"))?;
    knots.try_reserve_exact(knots_count).map_err(|_|Error::Budget("joined NURBS knots"))?;
    poles.extend_from_slice(&a.poles);poles.extend_from_slice(&b.poles[1..]);
    weights.extend_from_slice(&a.weights);let factor=a.weights[a.weights.len()-1]/b.weights[0];weights.extend(b.weights[1..].iter().map(|w|w*factor));
    for k in &a.knots[..a.knots.len()-1] {knots.push((k-a.domain[0])/(a.domain[1]-a.domain[0])*split);}
    for k in &b.knots[b.degree+1..] {knots.push(split+(k-b.domain[0])/(b.domain[1]-b.domain[0])*(1.-split));}
    let n=NurbsCurve{degree:a.degree,poles,weights,knots,domain:[0.,1.],periodic:false,reversed:false,seam:None};
    n.validate(limits)?;Ok(Some(Curve::Nurbs(n)))
}

fn trim_polycurve_by_parameter(
    segments: &[Curve],
    from: f64,
    to: f64,
    limits: &Limits,
) -> Result<Curve> {
    let mut result = Vec::new();
    let count = segments.len() as f64;
    for (index, segment) in segments.iter().enumerate() {
        let low = (from * count - index as f64).clamp(0.0, 1.0);
        let high = (to * count - index as f64).clamp(0.0, 1.0);
        if high > low {
            result.push(trim_curve_by_parameter(segment, low, high, limits)?);
        }
    }
    match result.len() {
        0 => Err(Error::Invalid("trim interval")),
        1 => Ok(result.remove(0)),
        _ => Ok(Curve::Polycurve {
            segments: result,
            reversed: false,
        }),
    }
}

/// Preserve the original polycurve trim API's arc-length fraction behavior.
fn trim_polycurve_by_length(
    segments: &[Curve],
    from_length: f64,
    to_length: f64,
    limits: &Limits,
) -> Result<Curve> {
    let mut result = Vec::new();
    let mut accumulated = 0.0;
    for segment in segments {
        let length = arc_length(segment, limits)?;
        if !length.is_finite() || length <= 0.0 {
            continue;
        }
        let low = accumulated;
        let high = accumulated + length;
        accumulated = high;
        let overlap_low = from_length.max(low);
        let overlap_high = to_length.min(high);
        if overlap_high <= overlap_low + 1e-12 {
            if high >= to_length {
                break;
            }
            continue;
        }
        result.push(trim_segment(
            segment,
            (overlap_low - low) / length,
            (overlap_high - low) / length,
            limits,
        )?);
        if high >= to_length {
            break;
        }
    }
    match result.len() {
        0 => Err(Error::Invalid("trim interval")),
        1 => Ok(result.remove(0)),
        _ => Ok(Curve::Polycurve {
            segments: result,
            reversed: false,
        }),
    }
}

fn trim_segment(segment: &Curve, from: f64, to: f64, limits: &Limits) -> Result<Curve> {
    let (head, _) = if to < 1.0 {
        split_curve(segment, to, limits)?
    } else {
        (segment.clone(), segment.clone())
    };
    if from == 0.0 {
        return Ok(head);
    }
    let (_, tail) = split_curve(&head, from / to, limits)?;
    Ok(tail)
}

/* ------------------------------ join/explode ------------------------------ */

fn flatten_segments(curve: &Curve, limits: &Limits) -> Result<Vec<Curve>> {
    let forward = forward_polycurve(curve, limits)?;
    Ok(match forward {
        Curve::Polycurve { segments, .. } => segments,
        other => vec![other],
    })
}

/// Join ordered curves whose ends meet within `tolerance` into one polycurve.
pub fn join_curves(curves: &[Curve], tolerance: f64, limits: &Limits) -> Result<Curve> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(Error::Invalid("join tolerance"));
    }
    let mut segments = Vec::new();
    for curve in curves {
        curve.validate(limits)?;
        let mut parts = flatten_segments(curve, limits)?;
        if parts.is_empty() {
            return Err(Error::Invalid("join segment list"));
        }
        segments.append(&mut parts);
    }
    if segments.is_empty() {
        return Err(Error::Invalid("join input"));
    }
    for pair in segments.windows(2) {
        let end = curve_endpoints(&pair[0], limits)?[1];
        let start = curve_endpoints(&pair[1], limits)?[0];
        if distance(end, start) > tolerance {
            return Err(Error::Invalid("join continuity"));
        }
    }
    if segments.len() == 1 {
        return Ok(segments.remove(0));
    }
    Ok(Curve::Polycurve {
        segments,
        reversed: false,
    })
}

/// Split a polycurve into its member curves; a native curve explodes to itself.
pub fn explode_curve(curve: &Curve, limits: &Limits) -> Result<Vec<Curve>> {
    curve.validate(limits)?;
    flatten_segments(curve, limits)
}

/// Close an open curve by appending the straight segment back to its start.
pub fn close_curve(curve: &Curve, tolerance: f64, limits: &Limits) -> Result<Curve> {
    let [start, end] = curve_endpoints(curve, limits)?;
    let gap = distance(start, end);
    if gap <= tolerance {
        return Ok(curve.clone());
    }
    let mut segments = flatten_segments(curve, limits)?;
    segments.push(Curve::Line {
        start: end,
        end: start,
        domain: [0.0, gap],
    });
    Ok(Curve::Polycurve {
        segments,
        reversed: false,
    })
}

/* --------------------------------- grips --------------------------------- */

/// Portable grip description: control points, edit points (Greville parameters) and
/// the knot vector.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveGrips {
    pub poles: Vec<Point3>,
    pub edit_points: Vec<Point3>,
    pub knots: Vec<f64>,
    pub closed: bool,
}

/// Move one edit point (a point that lies on the curve) to `target`.
///
/// A line moves the matching end. For a NURBS curve, solve for pole displacements
/// at the existing Greville points without changing knots, weights or parameterisation.
pub fn move_edit_point(
    curve: &Curve,
    index: usize,
    target: Point3,
    limits: &Limits,
) -> Result<Curve> {
    move_edit_point_with_joints(curve, index, target, identity(), 1e-9, limits)
}

pub(crate) fn move_edit_point_with_joints(
    curve: &Curve,
    index: usize,
    target: Point3,
    placement: Transform,
    tolerance: f64,
    limits: &Limits,
) -> Result<Curve> {
    finite(target)?;
    let grips = curve_grips(curve, 1e-9, limits)?;
    if index >= grips.edit_points.len() {
        return Err(Error::Invalid("edit point index"));
    }
    match curve {
        Curve::Line { .. } => move_grip(curve, index, target, limits),
        Curve::Nurbs(nurbs) => {
            if nurbs.periodic { nurbs.certify_repeated_periodic(limits)?; }
            if grips.edit_points[index] == target {
                return Ok(curve.clone());
            }
            let count = if nurbs.periodic { nurbs.poles.len()-nurbs.degree } else { nurbs.poles.len() };
            // ponytail: dense collocation capped at 512; use a banded solver for larger curves.
            if count > 512 {
                return Err(Error::Budget("curve edit point solver"));
            }
            let cells = count
                .checked_mul(count + 3)
                .ok_or(Error::Overflow("curve edit matrix"))?;
            crate::budget::Usage::array::<f64>(cells)?.check(limits)?;
            let mut matrix = Vec::new();
            matrix
                .try_reserve_exact(count)
                .map_err(|_| Error::Budget("curve edit matrix"))?;
            for row in 0..count {
                let parameter = edit_greville(nurbs, row);
                let mut values = Vec::new();
                values
                    .try_reserve_exact(count + 3)
                    .map_err(|_| Error::Budget("curve edit matrix"))?;
                values.resize(count, 0.0);
                for column in 0..nurbs.poles.len() {
                    values[column % count] += crate::curve_build::basis(nurbs.degree, &nurbs.knots, column, parameter)? * nurbs.weights[column];
                }
                let denominator: f64 = values.iter().sum();
                if !denominator.is_finite() || denominator.abs() < 1e-12 {
                    return Err(Error::Invalid("curve edit basis"));
                }
                for value in &mut values {
                    *value /= denominator;
                }
                let move_row = row == index
                    || (!nurbs.periodic && grips.closed
                        && (index == 0 || index + 1 == count)
                        && (row == 0 || row + 1 == count));
                let delta = if move_row {
                    sub(target, grips.edit_points[row])
                } else {
                    [0.0; 3]
                };
                values.extend_from_slice(&delta);
                matrix.push(values);
            }
            for column in 0..count {
                let best = (column..count)
                    .max_by(|&a, &b| matrix[a][column].abs().total_cmp(&matrix[b][column].abs()))
                    .unwrap();
                if matrix[best][column].abs() < 1e-12 {
                    return Err(Error::Invalid("singular curve edit matrix"));
                }
                matrix.swap(column, best);
                let pivot = matrix[column][column];
                for value in &mut matrix[column][column..] {
                    *value /= pivot;
                }
                for row in column + 1..count {
                    let factor = matrix[row][column];
                    for col in column..count + 3 {
                        matrix[row][col] -= factor * matrix[column][col];
                    }
                }
            }
            let mut displacements = vec![[0.0; 3]; count];
            for row in (0..count).rev() {
                for axis in 0..3 {
                    let mut value = matrix[row][count + axis];
                    for col in row + 1..count {
                        value -= matrix[row][col] * displacements[col][axis];
                    }
                    displacements[row][axis] = value;
                }
            }
            let mut edited = nurbs.clone();
            for (pole, delta) in edited.poles.iter_mut().zip(displacements) {
                *pole = add(*pole, delta);
            }
            if nurbs.periodic {
                for i in 0..nurbs.degree { edited.poles[count+i]=edited.poles[i]; }
            }
            edited.validate(limits)?;
            let result = Curve::Nurbs(edited);
            let actual = curve_grips(&result, 1e-9, limits)?.edit_points;
            for (row, point) in actual.iter().enumerate() {
                let moved = row == index
                    || (!nurbs.periodic && grips.closed
                        && (index == 0 || index + 1 == count)
                        && (row == 0 || row + 1 == count));
                let expected = if moved {
                    target
                } else {
                    grips.edit_points[row]
                };
                let magnitude = expected.iter().fold(1.0_f64, |a, b| a.max(b.abs()));
                if point
                    .iter()
                    .zip(expected)
                    .any(|(a, b)| (a - b).abs() > magnitude * 1e-8)
                {
                    return Err(Error::Invalid("inaccurate curve edit solution"));
                }
            }
            Ok(result)
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            if grips.edit_points[index] == target {
                return Ok(curve.clone());
            }
            let nurbs = crate::top11::affine_circle(
                *center,
                *x,
                *y,
                *radius,
                angle_domain.unwrap_or(*domain),
                identity(),
            )?;
            let last = match &nurbs {
                Curve::Nurbs(n) => n.poles.len() - 1,
                _ => unreachable!(),
            };
            move_edit_point_with_joints(
                &nurbs,
                if index == 0 { 0 } else { last },
                target,
                placement,
                tolerance,
                limits,
            )
        }
        Curve::Polycurve { reversed, .. } => {
            let forward = forward_polycurve(curve, limits)?;
            let ends = curve_endpoints(&forward, limits)?;
            let closed = joint_within(ends[0], ends[1], placement, tolerance)?;
            let Curve::Polycurve { segments, .. } = forward else {
                return Err(Error::Invalid("polycurve"));
            };
            let mut offset = 0usize;
            for (segment_index, segment) in segments.iter().enumerate() {
                let segment_grips = curve_grips(segment, 1e-9, limits)?.edit_points;
                let count = segment_grips.len();
                if index < offset + count {
                    let local_index = index - offset;
                    let selected = segment_grips[local_index];
                    if selected == target {
                        return Ok(curve.clone());
                    }
                    let ends = curve_endpoints(segment, limits)?;
                    let mut edited = segments.clone();
                    edited[segment_index] = move_edit_point_with_joints(
                        segment,
                        local_index,
                        target,
                        placement,
                        tolerance,
                        limits,
                    )?;
                    for side in 0..2 {
                        if distance(selected, ends[side]) > 1e-9 {
                            continue;
                        }
                        let neighbor = if side == 0 {
                            if segment_index > 0 {
                                Some(segment_index - 1)
                            } else if closed && segments.len() > 1 {
                                Some(segments.len() - 1)
                            } else {
                                None
                            }
                        } else if segment_index + 1 < segments.len() {
                            Some(segment_index + 1)
                        } else if closed && segments.len() > 1 {
                            Some(0)
                        } else {
                            None
                        };
                        let Some(neighbor) = neighbor else {
                            continue;
                        };
                        let other_end = curve_endpoints(&segments[neighbor], limits)?[1 - side];
                        if !joint_within(ends[side], other_end, placement, tolerance)? {
                            continue;
                        }
                        let other_grips =
                            curve_grips(&segments[neighbor], 1e-9, limits)?.edit_points;
                        let other_index = other_grips
                            .iter()
                            .position(|p| distance(*p, other_end) <= 1e-9)
                            .ok_or(Error::Unsupported("polycurve endpoint edit grip"))?;
                        edited[neighbor] = move_edit_point_with_joints(
                            &segments[neighbor],
                            other_index,
                            target,
                            placement,
                            tolerance,
                            limits,
                        )?;
                    }
                    let result = Curve::Polycurve {
                        segments: edited,
                        reversed: false,
                    };
                    if *reversed {
                        let Curve::Polycurve { segments, .. } = reverse_curve(&result, limits)?
                        else {
                            unreachable!()
                        };
                        return Ok(Curve::Polycurve {
                            segments,
                            reversed: true,
                        });
                    }
                    return Ok(result);
                }
                offset += count;
            }
            Err(Error::Invalid("edit point index"))
        }
    }
}

/// Grip points for a curve. A NURBS edit point is the curve point at the Greville
/// abscissa of each pole; a line exposes its two ends; a circle exposes its centre plus
/// the quadrant points and its arc ends.
fn edit_greville(n: &NurbsCurve, index: usize) -> f64 {
    let value=(1..=n.degree).map(|offset|n.knots[index+offset]).sum::<f64>()/n.degree as f64;
    if n.periodic {n.domain[0]+(value-n.domain[0]).rem_euclid(n.domain[1]-n.domain[0])} else {value}
}

pub fn curve_grips(curve: &Curve, tolerance: f64, limits: &Limits) -> Result<CurveGrips> {
    curve.validate(limits)?;
    let closed = is_closed(curve, tolerance, limits)?;
    Ok(match curve {
        Curve::Line { start, end, .. } => CurveGrips {
            poles: vec![*start, *end],
            edit_points: vec![*start, *end],
            knots: vec![0.0, 0.0, 1.0, 1.0],
            closed,
        },
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            let mut poles = vec![*center];
            let mut edit_points = Vec::new();
            for step in 0..=4 {
                let angle = angles[0] + (angles[1] - angles[0]) * step as f64 / 4.0;
                let point = add(
                    *center,
                    add(
                        scale(*x, radius * angle.cos()),
                        scale(*y, radius * angle.sin()),
                    ),
                );
                if step < 4 {
                    poles.push(point);
                }
                if step == 0 || step == 4 {
                    edit_points.push(point);
                }
            }
            CurveGrips {
                poles,
                edit_points,
                knots: vec![angles[0], angles[1]],
                closed,
            }
        }
        Curve::Nurbs(nurbs) => {
            if nurbs.periodic {nurbs.certify_repeated_periodic(limits)?;}
            let count=if nurbs.periodic {nurbs.poles.len()-nurbs.degree}else{nurbs.poles.len()};
            let mut edit_points = Vec::new();
            edit_points
                .try_reserve_exact(count)
                .map_err(|_| Error::Budget("curve edit points"))?;
            for index in 0..count {
                let greville=edit_greville(nurbs,index);
                edit_points.push(
                    *evaluate_curve(curve, Parameter::Native(greville), 0, identity(), limits)?
                        .first()
                        .ok_or(Error::Invalid("curve edit point"))?,
                );
            }
            CurveGrips {
                poles: nurbs.poles[..count].to_vec(),
                edit_points,
                knots: nurbs.knots.clone(),
                closed,
            }
        }
        Curve::Polycurve { .. } => {
            let forward = forward_polycurve(curve, limits)?;
            let Curve::Polycurve { segments, .. } = forward else {
                return Err(Error::Invalid("polycurve"));
            };
            let mut poles = Vec::new();
            let mut edit_points = Vec::new();
            let mut knots = Vec::new();
            for segment in &segments {
                let grips = curve_grips(segment, tolerance, limits)?;
                poles.extend(grips.poles);
                edit_points.extend(grips.edit_points);
                knots.extend(grips.knots);
            }
            CurveGrips {
                poles,
                edit_points,
                knots,
                closed,
            }
        }
    })
}

/// Move control point `index` to `target` and return a validated curve.
pub fn move_grip(curve: &Curve, index: usize, target: Point3, limits: &Limits) -> Result<Curve> {
    move_grip_with_joints(curve, index, target, identity(), 1e-9, limits)
}

fn joint_within(a: Point3, b: Point3, placement: Transform, tolerance: f64) -> Result<bool> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(Error::Invalid("joint tolerance"));
    }
    Ok(distance(placement.point(a)?, placement.point(b)?) <= tolerance)
}

pub(crate) fn move_grip_with_joints(
    curve: &Curve,
    index: usize,
    target: Point3,
    placement: Transform,
    tolerance: f64,
    limits: &Limits,
) -> Result<Curve> {
    finite(target)?;
    curve.validate(limits)?;
    match curve {
        Curve::Line { start, end, .. } => {
            let (start, end) = match index {
                0 => (target, *end),
                1 => (*start, target),
                _ => return Err(Error::Invalid("grip index")),
            };
            let length = distance(start, end);
            if length <= 0.0 {
                return Err(Error::Invalid("degenerate line"));
            }
            Ok(Curve::Line {
                start,
                end,
                domain: [0.0, length],
            })
        }
        Curve::Nurbs(nurbs) => {
            if nurbs.periodic {nurbs.certify_repeated_periodic(limits)?;}
            let count=if nurbs.periodic {nurbs.poles.len()-nurbs.degree}else{nurbs.poles.len()};
            if index >= count {
                return Err(Error::Invalid("grip index"));
            }
            let mut moved = nurbs.clone();
            moved.poles[index] = target;
            if nurbs.periodic && index<nurbs.degree {moved.poles[count+index]=target;}
            moved.validate(limits)?;
            Ok(Curve::Nurbs(moved))
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            if index == 0 {
                return Ok(Curve::Circle {
                    center: target,
                    x: *x,
                    y: *y,
                    radius: *radius,
                    domain: *domain,
                    angle_domain: *angle_domain,
                });
            }
            if !(1..=4).contains(&index) {
                return Err(Error::Invalid("grip index"));
            }
            let next_radius = distance(*center, target);
            if !next_radius.is_finite() || next_radius <= 1e-12 {
                return Err(Error::Invalid("degenerate circle radius"));
            }
            Ok(Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: next_radius,
                domain: *domain,
                angle_domain: *angle_domain,
            })
        }
        Curve::Polycurve { reversed, .. } => {
            let forward = forward_polycurve(curve, limits)?;
            let ends = curve_endpoints(&forward, limits)?;
            let closed = joint_within(ends[0], ends[1], placement, tolerance)?;
            let Curve::Polycurve { segments, .. } = forward else {
                return Err(Error::Invalid("polycurve"));
            };
            let mut offset = 0usize;
            for (segment_index, segment) in segments.iter().enumerate() {
                let poles = curve_grips(segment, 1e-9, limits)?.poles;
                if index < offset + poles.len() {
                    let local_index = index - offset;
                    if poles[local_index] == target {
                        return Ok(curve.clone());
                    }
                    let old_ends = curve_endpoints(segment, limits)?;
                    let changed = move_grip_with_joints(
                        segment,
                        local_index,
                        target,
                        placement,
                        tolerance,
                        limits,
                    )?;
                    let new_ends = curve_endpoints(&changed, limits)?;
                    let mut edited = segments.clone();
                    edited[segment_index] = changed;
                    for side in 0..2 {
                        if distance(old_ends[side], new_ends[side]) <= 1e-12 {
                            continue;
                        }
                        let neighbor = if side == 0 {
                            if segment_index > 0 {
                                Some(segment_index - 1)
                            } else if closed && segments.len() > 1 {
                                Some(segments.len() - 1)
                            } else {
                                None
                            }
                        } else if segment_index + 1 < segments.len() {
                            Some(segment_index + 1)
                        } else if closed && segments.len() > 1 {
                            Some(0)
                        } else {
                            None
                        };
                        let Some(neighbor) = neighbor else {
                            continue;
                        };
                        let other_end = curve_endpoints(&segments[neighbor], limits)?[1 - side];
                        if !joint_within(old_ends[side], other_end, placement, tolerance)? {
                            continue;
                        }
                        let other = &edited[neighbor];
                        let other_grips = curve_grips(other, 1e-9, limits)?;
                        let control = match other {
                            Curve::Line { .. } | Curve::Nurbs(_) => other_grips
                                .poles
                                .iter()
                                .position(|p| distance(*p, other_end) <= 1e-9),
                            _ => None,
                        };
                        let adjusted = if let Some(control) = control {
                            move_grip_with_joints(
                                other,
                                control,
                                new_ends[side],
                                placement,
                                tolerance,
                                limits,
                            )?
                        } else {
                            let point = other_grips
                                .edit_points
                                .iter()
                                .position(|p| distance(*p, other_end) <= 1e-9)
                                .ok_or(Error::Unsupported("polycurve endpoint grip"))?;
                            move_edit_point_with_joints(
                                other,
                                point,
                                new_ends[side],
                                placement,
                                tolerance,
                                limits,
                            )?
                        };
                        let adjusted_end = curve_endpoints(&adjusted, limits)?[1 - side];
                        if distance(adjusted_end, new_ends[side]) > 1e-9 {
                            return Err(Error::Invalid("polycurve control joint"));
                        }
                        edited[neighbor] = adjusted;
                    }
                    let result = Curve::Polycurve {
                        segments: edited,
                        reversed: false,
                    };
                    if *reversed {
                        let Curve::Polycurve { segments, .. } = reverse_curve(&result, limits)?
                        else {
                            unreachable!()
                        };
                        return Ok(Curve::Polycurve {
                            segments,
                            reversed: true,
                        });
                    }
                    return Ok(result);
                }
                offset += poles.len();
            }
            Err(Error::Invalid("grip index"))
        }
    }
}

/// Fuse a smooth polycurve after control editing, when its pieces admit an exact
/// clamped rational representation. Returns None when that conversion is unavailable.
pub(crate) fn fuse_smooth_control_polycurve(
    original: &Curve,
    edited: &Curve,
    placement: Transform,
    tolerance: f64,
    angular_tolerance: f64,
    limits: &Limits,
) -> Result<Option<Curve>> {
    if original == edited {
        return Ok(None);
    }
    let Curve::Polycurve {
        segments: before, ..
    } = forward_polycurve(original, limits)?
    else {
        return Ok(None);
    };
    let Curve::Polycurve {
        segments: after, ..
    } = forward_polycurve(edited, limits)?
    else {
        return Ok(None);
    };
    if before.len() < 2 || before.len() != after.len() {
        return Ok(None);
    }
    let vector = |v: Point3| -> Point3 {
        std::array::from_fn(|r| (0..3).map(|c| placement.0[r][c] * v[c]).sum())
    };
    let smooth = |first: &Curve, second: &Curve| -> Result<bool> {
        let end = curve_endpoints(first, limits)?[1];
        let start = curve_endpoints(second, limits)?[0];
        if !joint_within(end, start, placement, tolerance)? {
            return Ok(false);
        }
        let (Ok(a), Ok(b)) = (
            tangent_at(first, 1.0, limits),
            tangent_at(second, 0.0, limits),
        ) else {
            return Ok(false);
        };
        let (Ok(a), Ok(b)) = (unit(vector(a)), unit(vector(b))) else {
            return Ok(false);
        };
        Ok(dot(a, b).clamp(-1.0, 1.0).acos() <= angular_tolerance)
    };
    for pair in before.windows(2) {
        if !smooth(&pair[0], &pair[1])? {
            return Ok(None);
        }
    }
    let first = curve_endpoints(&before[0], limits)?[0];
    let last = curve_endpoints(before.last().unwrap(), limits)?[1];
    if joint_within(first, last, placement, tolerance)?
        && !smooth(before.last().unwrap(), &before[0])?
    {
        return Ok(None);
    }
    let mut parts = Vec::new();
    for part in &after {
        let candidate = match part {
            Curve::Line { start, end, .. } => line_to_nurbs(*start, *end, limits)?,
            Curve::Circle {
                center,
                x,
                y,
                radius,
                domain,
                angle_domain,
            } => crate::top11::affine_circle(
                *center,
                *x,
                *y,
                *radius,
                angle_domain.unwrap_or(*domain),
                identity(),
            )?,
            Curve::Nurbs(n) if !n.periodic => Curve::Nurbs(n.clone()),
            _ => return Ok(None),
        };
        let Curve::Nurbs(mut n) = candidate else {
            unreachable!()
        };
        if n.knots[..=n.degree].iter().any(|k| *k != n.domain[0])
            || n.knots[n.knots.len() - n.degree - 1..]
                .iter()
                .any(|k| *k != n.domain[1])
        {
            return Ok(None);
        }
        if n.reversed {
            n.poles.reverse();
            n.weights.reverse();
            n.knots = n
                .knots
                .iter()
                .rev()
                .map(|k| n.domain[0] + n.domain[1] - k)
                .collect();
            n.reversed = false;
        }
        parts.push(n);
    }
    let degree = parts.iter().map(|p| p.degree).max().unwrap();
    for part in &mut parts {
        while part.degree < degree {
            *part = crate::curve_build::elevate_degree(part, limits)?;
        }
    }
    let count = parts.len();
    let pole_count = parts.iter().try_fold(1usize, |total, part| {
        total
            .checked_add(part.poles.len() - 1)
            .ok_or(Error::Overflow("fused curve poles"))
    })?;
    let knot_count = pole_count
        .checked_add(degree + 1)
        .ok_or(Error::Overflow("fused curve knots"))?;
    let mut usage = crate::budget::Usage::array::<Point3>(pole_count)?;
    usage.points = u64::try_from(pole_count).map_err(|_| Error::Overflow("fused curve points"))?;
    usage.add(crate::budget::Usage::array::<f64>(pole_count)?, limits)?;
    usage.add(crate::budget::Usage::array::<f64>(knot_count)?, limits)?;
    let mut poles: Vec<Point3> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();
    let mut knots: Vec<f64> = Vec::new();
    poles
        .try_reserve_exact(pole_count)
        .map_err(|_| Error::Budget("fused curve poles"))?;
    weights
        .try_reserve_exact(pole_count)
        .map_err(|_| Error::Budget("fused curve weights"))?;
    knots
        .try_reserve_exact(knot_count)
        .map_err(|_| Error::Budget("fused curve knots"))?;
    for (i, part) in parts.iter().enumerate() {
        if i > 0 && *poles.last().unwrap() != part.poles[0] {
            return Ok(None);
        }
        let factor = if i == 0 {
            1.0
        } else {
            weights.last().unwrap() / part.weights[0]
        };
        let start = usize::from(i > 0);
        poles.extend_from_slice(&part.poles[start..]);
        weights.extend(part.weights[start..].iter().map(|w| w * factor));
        let span = part.domain[1] - part.domain[0];
        for knot in &part.knots[if i == 0 { 0 } else { degree + 1 }..if i + 1 == count {
            part.knots.len()
        } else {
            part.knots.len() - 1
        }] {
            knots.push((i as f64 + (*knot - part.domain[0]) / span) / count as f64);
        }
    }
    let result = NurbsCurve {
        degree,
        poles,
        weights,
        knots,
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    result.validate(limits)?;
    Ok(Some(Curve::Nurbs(result)))
}

/// Delete one control pole from a clamped nonperiodic NURBS curve.
/// The removed interior knot is chosen near the pole's Greville station.
pub fn delete_control_point(curve: &Curve, index: usize, limits: &Limits) -> Result<Curve> {
    curve.validate(limits)?;
    let Curve::Nurbs(source) = curve else {
        return Err(Error::Unsupported("control point deletion needs NURBS"));
    };
    if source.periodic {
        return Err(Error::Unsupported("periodic control point deletion"));
    }
    if index >= source.poles.len() {
        return Err(Error::Invalid("control point index"));
    }
    if source.poles.len() <= source.degree + 1 {
        return Err(Error::Invalid("too few control points"));
    }
    if source.knots[..=source.degree]
        .iter()
        .any(|k| *k != source.domain[0])
        || source.knots[source.knots.len() - source.degree - 1..]
            .iter()
            .any(|k| *k != source.domain[1])
    {
        return Err(Error::Unsupported("trimmed control point deletion"));
    }
    let station = (1..=source.degree)
        .map(|offset| source.knots[index + offset])
        .sum::<f64>()
        / source.degree as f64;
    let knot = (source.degree + 1..source.poles.len())
        .min_by(|&a, &b| {
            (source.knots[a] - station)
                .abs()
                .total_cmp(&(source.knots[b] - station).abs())
        })
        .ok_or(Error::Invalid("control point knot"))?;
    let mut result = source.clone();
    result.poles.remove(index);
    result.weights.remove(index);
    result.knots.remove(knot);
    result.validate(limits)?;
    Ok(Curve::Nurbs(result))
}

/// Insert one control point without changing the geometry. A line becomes an equivalent
/// degree-1 NURBS record; a NURBS curve gets one exact knot insertion.
pub fn insert_control_point(curve: &Curve, fraction: f64, limits: &Limits) -> Result<Curve> {
    check_fraction(fraction)?;
    if fraction <= 0.0 || fraction >= 1.0 {
        return Err(Error::Invalid("control point fraction"));
    }
    curve.validate(limits)?;
    match curve {
        Curve::Line { start, end, .. } => {
            let point = point_at(curve, fraction, limits)?;
            let nurbs = NurbsCurve {
                degree: 1,
                poles: vec![*start, point, *end],
                weights: vec![1.0; 3],
                knots: vec![0.0, 0.0, fraction, 1.0, 1.0],
                domain: [0.0, 1.0],
                periodic: false,
                reversed: false,
                seam: None,
            };
            nurbs.validate(limits)?;
            Ok(Curve::Nurbs(nurbs))
        }
        Curve::Nurbs(nurbs) => {
            let parameter = nurbs.domain[0] + fraction * (nurbs.domain[1] - nurbs.domain[0]);
            let multiplicity = nurbs
                .knots
                .iter()
                .filter(|value| **value == parameter)
                .count();
            if multiplicity > nurbs.degree {
                return Err(Error::Budget("control point insertion"));
            }
            Ok(Curve::Nurbs(insert_knot(nurbs, parameter, 1, limits)?))
        }
        _ => Err(Error::Unsupported(
            "control point insertion for this curve kind",
        )),
    }
}

/// Convert a line to a degree-1 NURBS record (used by callers that need one
/// representation for control-point editing).
pub fn line_to_nurbs(start: Point3, end: Point3, limits: &Limits) -> Result<Curve> {
    let length = distance(start, end);
    if !length.is_finite() || length <= 0.0 {
        return Err(Error::Invalid("degenerate line"));
    }
    let nurbs = NurbsCurve {
        degree: 1,
        poles: vec![start, end],
        weights: vec![1.0; 2],
        knots: vec![0.0, 0.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    nurbs.validate(limits)?;
    Ok(Curve::Nurbs(nurbs))
}

/// Refit a curve through `samples` stations as an interpolated NURBS record. The
/// result is a new record; the source is never changed.
pub fn refit_curve(
    curve: &Curve,
    samples: usize,
    degree: usize,
    mode: KnotMode,
    limits: &Limits,
) -> Result<Curve> {
    if samples < 2 {
        return Err(Error::Invalid("refit sample count"));
    }
    let points = sample_curve(curve, samples, limits)?;
    Ok(Curve::Nurbs(interpolate_any(
        &points, degree, mode, limits,
    )?))
}
