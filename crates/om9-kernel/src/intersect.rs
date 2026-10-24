// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact curve/B-Rep intersection owned by the OCCT adapter. The portable `om9-core`
//! fallback works on the display snapshot; this module is the product path and asks OCCT
//! for the real face intersections.
use crate::KernelError;
use om9_core::{
    brep::Brep,
    budget::Limits,
    curve_ops,
    geometry::{Curve, NurbsCurve, Point3},
};
use std::ffi::{CStr, c_char, c_int};

const MAX_HITS: usize = 256;
const MESSAGE_CAPACITY: usize = 512;

unsafe extern "C" {
    fn om9_occt_line_brep_intersections(
        source: *const u8,
        length: usize,
        p0: *const f64,
        p1: *const f64,
        tolerance: f64,
        fractions: *mut f64,
        capacity: usize,
        count: *mut usize,
        message: *mut c_char,
        message_capacity: c_int,
    ) -> c_int;
    fn om9_occt_circle_brep_intersections(
        source: *const u8,
        length: usize,
        center: *const f64,
        x: *const f64,
        y: *const f64,
        radius: f64,
        u0: f64,
        u1: f64,
        tolerance: f64,
        fractions: *mut f64,
        capacity: usize,
        count: *mut usize,
        message: *mut c_char,
        message_capacity: c_int,
    ) -> c_int;
    fn om9_occt_line_curve_parameter(
        origin: *const f64,
        point: *const f64,
        degree: c_int,
        poles: *const f64,
        pole_count: usize,
        weights: *const f64,
        weight_count: usize,
        knots: *const f64,
        knot_count: usize,
        periodic: c_int,
        fraction: *mut f64,
        message: *mut c_char,
        message_capacity: c_int,
    ) -> c_int;
    fn om9_occt_bspline_brep_intersections(
        source: *const u8,
        length: usize,
        degree: c_int,
        pole_count: usize,
        poles: *const f64,
        weights: *const f64,
        knot_count: usize,
        knots: *const f64,
        multiplicities: *const c_int,
        periodic: c_int,
        u0: f64,
        u1: f64,
        tolerance: f64,
        fractions: *mut f64,
        capacity: usize,
        count: *mut usize,
        message: *mut c_char,
        message_capacity: c_int,
    ) -> c_int;
}

fn message(buffer: &[c_char]) -> String {
    // SAFETY: the bridge always NUL-terminates a message buffer it writes.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn check(status: c_int, buffer: &[c_char], count: usize) -> Result<Vec<f64>, KernelError> {
    match status {
        0 => Ok(Vec::new()),
        1 => Err(KernelError::InvalidInput(message(buffer))),
        3 => Err(KernelError::InvalidInput(format!(
            "curve/B-Rep intersection budget ({count} hits)"
        ))),
        _ => Err(KernelError::Failure(message(buffer))),
    }
}

fn collect(
    status: c_int,
    buffer: &[c_char],
    fractions: &[f64],
    count: usize,
) -> Result<Vec<f64>, KernelError> {
    check(status, buffer, count)?;
    Ok(fractions[..count.min(fractions.len())].to_vec())
}

fn line_fractions(
    source: &[u8],
    start: Point3,
    end: Point3,
    tolerance: f64,
) -> Result<Vec<f64>, KernelError> {
    let mut fractions = [0.0f64; MAX_HITS];
    let mut count = 0usize;
    let mut buffer = [0 as c_char; MESSAGE_CAPACITY];
    // SAFETY: all arrays are live for the synchronous call.
    let status = unsafe {
        om9_occt_line_brep_intersections(
            source.as_ptr(),
            source.len(),
            start.as_ptr(),
            end.as_ptr(),
            tolerance,
            fractions.as_mut_ptr(),
            fractions.len(),
            &mut count,
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    collect(status, &buffer, &fractions, count)
}

#[allow(clippy::too_many_arguments)]
fn circle_fractions(
    source: &[u8],
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    u0: f64,
    u1: f64,
    tolerance: f64,
) -> Result<Vec<f64>, KernelError> {
    let mut fractions = [0.0f64; MAX_HITS];
    let mut count = 0usize;
    let mut buffer = [0 as c_char; MESSAGE_CAPACITY];
    // SAFETY: all arrays are live for the synchronous call.
    let status = unsafe {
        om9_occt_circle_brep_intersections(
            source.as_ptr(),
            source.len(),
            center.as_ptr(),
            x.as_ptr(),
            y.as_ptr(),
            radius,
            u0,
            u1,
            tolerance,
            fractions.as_mut_ptr(),
            fractions.len(),
            &mut count,
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    collect(status, &buffer, &fractions, count)
}

fn knot_multiplicities(nurbs: &NurbsCurve) -> (Vec<f64>, Vec<c_int>) {
    let mut knots: Vec<f64> = Vec::new();
    let mut mults: Vec<c_int> = Vec::new();
    for knot in &nurbs.knots {
        if let Some(last) = knots.last().copied()
            && (*knot - last).abs() <= 1e-12
        {
            if let Some(value) = mults.last_mut() {
                *value += 1;
            }
            continue;
        }
        knots.push(*knot);
        mults.push(1);
    }
    (knots, mults)
}

fn bspline_fractions(
    source: &[u8],
    nurbs: &NurbsCurve,
    tolerance: f64,
) -> Result<Vec<f64>, KernelError> {
    let (knots, mults) = knot_multiplicities(nurbs);
    let mut poles = Vec::with_capacity(nurbs.poles.len() * 3);
    for pole in &nurbs.poles {
        poles.extend_from_slice(pole);
    }
    let domain = nurbs.domain;
    let mut fractions = [0.0f64; MAX_HITS];
    let mut count = 0usize;
    let mut buffer = [0 as c_char; MESSAGE_CAPACITY];
    // SAFETY: all arrays are live for the synchronous call.
    let status = unsafe {
        om9_occt_bspline_brep_intersections(
            source.as_ptr(),
            source.len(),
            c_int::try_from(nurbs.degree).unwrap_or(c_int::MAX),
            nurbs.poles.len(),
            poles.as_ptr(),
            nurbs.weights.as_ptr(),
            knots.len(),
            knots.as_ptr(),
            mults.as_ptr(),
            i32::from(nurbs.periodic),
            domain[0],
            domain[1],
            tolerance,
            fractions.as_mut_ptr(),
            fractions.len(),
            &mut count,
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    collect(status, &buffer, &fractions, count)
}

fn curve_fractions(
    source: &[u8],
    curve: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<f64>, KernelError> {
    if matches!(curve, Curve::Nurbs(nurbs) if nurbs.reversed) {
        let reversed = curve_ops::reverse_curve(curve, limits)
            .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
        let mut fractions = curve_fractions(source, &reversed, tolerance, limits)?;
        for value in &mut fractions {
            *value = 1.0 - *value;
        }
        fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        return Ok(fractions);
    }
    match curve {
        Curve::Line { start, end, .. } => line_fractions(source, *start, *end, tolerance),
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            circle_fractions(
                source, *center, *x, *y, *radius, angles[0], angles[1], tolerance,
            )
        }
        Curve::Nurbs(nurbs) => bspline_fractions(source, nurbs, tolerance),
        Curve::Polycurve { .. } => Err(KernelError::InvalidInput(
            "polycurve is handled by the caller".into(),
        )),
    }
}

/// Exact curve/B-Rep intersection fractions in `[0, 1]`. Polycurves are evaluated
/// segment-by-segment with the documented segment-uniform parameterisation.
pub fn curve_brep_fractions(
    brep: &Brep,
    curve: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<f64>, KernelError> {
    brep.validate(limits)
        .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(KernelError::InvalidInput("intersection tolerance".into()));
    }
    let reversed_polycurve = matches!(curve, Curve::Polycurve { reversed: true, .. });
    let normalized = if reversed_polycurve {
        curve_ops::reverse_curve(curve, limits)
            .map_err(|error| KernelError::InvalidInput(error.to_string()))?
    } else {
        curve.clone()
    };
    let source = brep.source().as_bytes();
    if let Curve::Polycurve { segments, .. } = &normalized {
        if segments.is_empty() {
            return Err(KernelError::InvalidInput("empty polycurve".into()));
        }
        let count = segments.len();
        let mut fractions = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            let local = curve_fractions(source, segment, tolerance, limits)?;
            for value in local {
                fractions.push((index as f64 + value) / count as f64);
            }
        }
        if reversed_polycurve {
            for value in &mut fractions {
                *value = 1.0 - *value;
            }
        }
        fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        fractions.dedup_by(|a, b| (*a - *b).abs() <= 1e-7);
        return Ok(fractions);
    }
    curve_fractions(source, &normalized, tolerance, limits)
}

/// Closest line-curve parameter for one exact NURBS curve through OCCT's
/// `GeomAPI_ExtremaCurveCurve`. The line is infinite through `origin`/`point`.
pub fn line_nurbs_fraction(
    curve: &Curve,
    origin: Point3,
    point: Point3,
    _limits: &Limits,
) -> Result<f64, KernelError> {
    let Curve::Nurbs(nurbs) = curve else {
        return Err(KernelError::InvalidInput(
            "line-curve intersection needs a NURBS curve".into(),
        ));
    };
    let mut poles = Vec::with_capacity(nurbs.poles.len() * 3);
    for pole in &nurbs.poles {
        poles.extend_from_slice(pole);
    }
    let weights = if nurbs.weights.len() == nurbs.poles.len() {
        nurbs.weights.clone()
    } else {
        vec![1.0; nurbs.poles.len()]
    };
    let mut buffer = [0 as c_char; MESSAGE_CAPACITY];
    let mut fraction = 0.0f64;
    // SAFETY: all arrays are live for the synchronous call.
    let status = unsafe {
        om9_occt_line_curve_parameter(
            origin.as_ptr(),
            point.as_ptr(),
            nurbs.degree as c_int,
            poles.as_ptr(),
            nurbs.poles.len(),
            weights.as_ptr(),
            weights.len(),
            nurbs.knots.as_ptr(),
            nurbs.knots.len(),
            i32::from(nurbs.periodic),
            &mut fraction,
            buffer.as_mut_ptr(),
            MESSAGE_CAPACITY as c_int,
        )
    };
    match status {
        0 if fraction.is_finite() && (0.0..=1.0).contains(&fraction) => Ok(fraction),
        0 => Err(KernelError::Failure(
            "line-curve fraction outside [0,1]".into(),
        )),
        1 => Err(KernelError::InvalidInput(message(&buffer))),
        _ => Err(KernelError::Failure(message(&buffer))),
    }
}

fn line_distance(point: Point3, origin: Point3, direction: Point3) -> f64 {
    let length2 =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if length2 <= 1.0e-18 {
        return f64::INFINITY;
    }
    let offset = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let t =
        (offset[0] * direction[0] + offset[1] * direction[1] + offset[2] * direction[2]) / length2;
    let projected = [
        origin[0] + direction[0] * t,
        origin[1] + direction[1] * t,
        origin[2] + direction[2] * t,
    ];
    (point[0] - projected[0])
        .hypot(point[1] - projected[1])
        .hypot(point[2] - projected[2])
}

fn curve_as_nurbs(curve: &Curve) -> Option<NurbsCurve> {
    match curve {
        Curve::Nurbs(nurbs) => Some(nurbs.clone()),
        Curve::Line { start, end, domain } => Some(NurbsCurve {
            degree: 1,
            poles: vec![*start, *end],
            weights: vec![1.0, 1.0],
            knots: vec![domain[0], domain[0], domain[1], domain[1]],
            domain: *domain,
            periodic: false,
            reversed: false,
            seam: None,
        }),
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            ..
        } => {
            let sqrt_half = std::f64::consts::FRAC_1_SQRT_2;
            let point = |sx: f64, sy: f64| {
                [
                    center[0] + (x[0] * sx + y[0] * sy) * radius,
                    center[1] + (x[1] * sx + y[1] * sy) * radius,
                    center[2] + (x[2] * sx + y[2] * sy) * radius,
                ]
            };
            Some(NurbsCurve {
                degree: 2,
                poles: vec![
                    point(1.0, 0.0),
                    point(1.0, 1.0),
                    point(0.0, 1.0),
                    point(-1.0, 1.0),
                    point(-1.0, 0.0),
                    point(-1.0, -1.0),
                    point(0.0, -1.0),
                    point(1.0, -1.0),
                    point(1.0, 0.0),
                ],
                weights: vec![
                    1.0, sqrt_half, 1.0, sqrt_half, 1.0, sqrt_half, 1.0, sqrt_half, 1.0,
                ],
                knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0],
                domain: *domain,
                periodic: false,
                reversed: false,
                seam: None,
            })
        }
        Curve::Polycurve { .. } => None,
    }
}

fn line_line_fraction(start: Point3, end: Point3, origin: Point3, point: Point3) -> f64 {
    let direction = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let length2 =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if length2 <= 1.0e-18 {
        return 0.0;
    }
    let target = [
        point[0] - start[0],
        point[1] - start[1],
        point[2] - start[2],
    ];
    let e = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let cross = [
        direction[1] * e[2] - direction[2] * e[1],
        direction[2] * e[0] - direction[0] * e[2],
        direction[0] * e[1] - direction[1] * e[0],
    ];
    let cross_len = cross[0].hypot(cross[1]).hypot(cross[2]);
    if cross_len <= 1.0e-12 {
        return ((target[0] * direction[0] + target[1] * direction[1] + target[2] * direction[2])
            / length2)
            .clamp(0.0, 1.0);
    }
    let w = [
        start[0] - origin[0],
        start[1] - origin[1],
        start[2] - origin[2],
    ];
    let a = length2;
    let b = direction[0] * e[0] + direction[1] * e[1] + direction[2] * e[2];
    let c = e[0] * e[0] + e[1] * e[1] + e[2] * e[2];
    let d0 = direction[0] * w[0] + direction[1] * w[1] + direction[2] * w[2];
    let e0 = e[0] * w[0] + e[1] * w[1] + e[2] * w[2];
    let denom = a * c - b * b;
    if denom.abs() <= 1.0e-18 {
        return 0.0;
    }
    ((b * e0 - c * d0) / denom).clamp(0.0, 1.0)
}
/// Closest line-curve fraction for every curve kind. NURBS, lines and circles
/// are converted to an exact OCCT B-spline/line; polycurves recurse per segment.
pub fn line_curve_fraction(
    curve: &Curve,
    origin: Point3,
    point: Point3,
    limits: &Limits,
) -> Result<f64, KernelError> {
    if let Curve::Line { start, end, .. } = curve {
        return Ok(line_line_fraction(*start, *end, origin, point));
    }
    if let Curve::Polycurve { segments, reversed } = curve {
        if segments.is_empty() {
            return Err(KernelError::InvalidInput("empty polycurve".into()));
        }
        let direction = [
            point[0] - origin[0],
            point[1] - origin[1],
            point[2] - origin[2],
        ];
        let mut best: Option<(f64, f64, f64)> = None;
        for (index, segment) in segments.iter().enumerate() {
            let local = line_curve_fraction(segment, origin, point, limits)?;
            let evaluated = om9_core::evaluation::evaluate_curve(
                segment,
                om9_core::evaluation::Parameter::Fraction(local),
                0,
                om9_core::geometry::Transform::identity(),
                limits,
            )
            .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
            let value = *evaluated
                .first()
                .ok_or_else(|| KernelError::InvalidInput("polycurve point".into()))?;
            let distance = line_distance(value, origin, direction);
            let point_distance = (value[0] - point[0])
                .hypot(value[1] - point[1])
                .hypot(value[2] - point[2]);
            if best.is_none_or(|(best_distance, best_point, _)| {
                distance < best_distance - 1.0e-12
                    || ((distance - best_distance).abs() <= 1.0e-12 && point_distance < best_point)
            }) {
                best = Some((
                    distance,
                    point_distance,
                    (index as f64 + local) / segments.len() as f64,
                ));
            }
        }
        let fraction = best.expect("polycurve candidate").2;
        return Ok(if *reversed { 1.0 - fraction } else { fraction });
    }
    let nurbs = curve_as_nurbs(curve)
        .ok_or_else(|| KernelError::InvalidInput("unsupported curve".into()))?;
    line_nurbs_fraction(&Curve::Nurbs(nurbs), origin, point, limits)
}
