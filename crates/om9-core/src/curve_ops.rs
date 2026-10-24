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
    let mut jet = evaluate_curve(
        &segments[index],
        Parameter::Fraction(local),
        order,
        identity(),
        limits,
    )?;
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
    unit(derivative_at(curve, fraction, limits)?)
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
            let length = (domain[1] - domain[0]).abs();
            Curve::Line {
                start: *end,
                end: *start,
                domain: [0.0, length],
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
                domain: [0.0, 1.0],
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
    let [start, end] = curve_endpoints(curve, limits)?;
    Ok(distance(start, end) <= tolerance)
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
            shifted.seam = Some(nurbs.domain[0] + fraction * (nurbs.domain[1] - nurbs.domain[0]));
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
    if let Curve::Polycurve { segments, reversed } = curve {
        if *reversed {
            let forward = reverse_curve(curve, limits)?;
            return trim_curve(&forward, from, to, limits);
        }
        let total = polycurve_length(segments, 0.0, 1.0, limits)?;
        return trim_polycurve(segments, from * total, to * total, limits);
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

/// Trim a forward polycurve by absolute arc length; each member keeps its exact
/// geometry and partial members are split at the measured length.
fn trim_polycurve(
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
    let (head, _) = if to < 1.0 - 1e-12 {
        split_curve(segment, to, limits)?
    } else {
        (segment.clone(), segment.clone())
    };
    if from <= 1e-12 {
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
/// A line moves the matching end. A NURBS curve is re-interpolated through the edited
/// Greville points, so the result passes through the new point while keeping the source
/// degree; the parameterisation may change, which is the documented edit-point trade-off.
pub fn move_edit_point(
    curve: &Curve,
    index: usize,
    target: Point3,
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
            let mut points = grips.edit_points;
            points[index] = target;
            let degree = nurbs.degree.min(points.len() - 1);
            Ok(Curve::Nurbs(interpolate_any(
                &points,
                degree,
                KnotMode::ChordLength,
                limits,
            )?))
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let local = sub(target, *center);
            let angle = dot(local, *y).atan2(dot(local, *x));
            let (start, end) = match angle_domain {
                None => (angle, angle + std::f64::consts::TAU),
                Some(angles) => {
                    if index == 0 {
                        (angle, angle + (angles[1] - angles[0]))
                    } else {
                        let mut end = angle;
                        while end <= angles[0] {
                            end += std::f64::consts::TAU;
                        }
                        (angles[0], end)
                    }
                }
            };
            if !stacked_angle_domain(*radius, start, end) {
                return Err(Error::Invalid("edit point angle"));
            }
            Ok(Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: *radius,
                domain: if angle_domain.is_none() {
                    [0.0, 1.0]
                } else {
                    *domain
                },
                angle_domain: Some([start, end]),
            })
        }
        Curve::Polycurve { reversed, .. } => {
            let forward = forward_polycurve(curve, limits)?;
            let Curve::Polycurve { segments, .. } = forward else {
                return Err(Error::Invalid("polycurve"));
            };
            let mut offset = 0usize;
            let mut edited = segments.clone();
            for (segment_index, segment) in segments.iter().enumerate() {
                let count = curve_grips(segment, 1e-9, limits)?.edit_points.len();
                if index < offset + count {
                    edited[segment_index] =
                        move_edit_point(segment, index - offset, target, limits)?;
                    let result = Curve::Polycurve {
                        segments: edited,
                        reversed: false,
                    };
                    return if *reversed {
                        reverse_curve(&result, limits)
                    } else {
                        Ok(result)
                    };
                }
                offset += count;
            }
            Err(Error::Invalid("edit point index"))
        }
    }
}

fn stacked_angle_domain(radius: f64, start: f64, end: f64) -> bool {
    radius.is_finite()
        && radius > 0.0
        && start.is_finite()
        && end.is_finite()
        && end > start
        && (end - start) <= std::f64::consts::TAU + 1e-9
}

/// Grip points for a curve. A NURBS edit point is the curve point at the Greville
/// abscissa of each pole; a line exposes its two ends; a circle exposes its centre plus
/// the quadrant points and its arc ends.
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
            let degree = nurbs.degree;
            let mut edit_points = Vec::new();
            edit_points
                .try_reserve_exact(nurbs.poles.len())
                .map_err(|_| Error::Budget("curve edit points"))?;
            for index in 0..nurbs.poles.len() {
                let greville: f64 = (1..=degree)
                    .map(|offset| nurbs.knots[index + offset])
                    .sum::<f64>()
                    / degree as f64;
                edit_points.push(
                    *evaluate_curve(curve, Parameter::Native(greville), 0, identity(), limits)?
                        .first()
                        .ok_or(Error::Invalid("curve edit point"))?,
                );
            }
            CurveGrips {
                poles: nurbs.poles.clone(),
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
            if index >= nurbs.poles.len() {
                return Err(Error::Invalid("grip index"));
            }
            let mut moved = nurbs.clone();
            moved.poles[index] = target;
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
            let Curve::Polycurve { segments, .. } = forward else {
                return Err(Error::Invalid("polycurve"));
            };
            let mut offset = 0usize;
            let mut edited = segments.clone();
            for (segment_index, segment) in segments.iter().enumerate() {
                let count = curve_grips(segment, 1e-9, limits)?.poles.len();
                if index < offset + count {
                    edited[segment_index] = move_grip(segment, index - offset, target, limits)?;
                    let result = Curve::Polycurve {
                        segments: edited,
                        reversed: false,
                    };
                    return if *reversed {
                        reverse_curve(&result, limits)
                    } else {
                        Ok(result)
                    };
                }
                offset += count;
            }
            Err(Error::Invalid("grip index"))
        }
    }
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
