// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve modification kernel (OM9-L2-02-04): offset, line fillet/chamfer, blend with
//! G0/G1/G2 continuity and continuity matching.
//!
//! The arithmetic is safe Rust over the portable records. The offset evaluator is the
//! existing [`crate::offset_curve::OffsetCurve`] (its D0..D3 contract is owned by OM9,
//! not by OCCT); the resulting points are fitted back to a NURBS record.
use crate::{
    Error, Result,
    budget::Limits,
    curve_build::{KnotMode, arc_through, elevate_degree, fit_points, interpolate_any},
    curve_ops::{derivative_at, point_at, reverse_curve},
    geometry::{Curve, NurbsCurve, Point3, finite},
    offset_curve::{NurbsBasis, OffsetCurve},
};

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

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: Point3) -> f64 {
    dot(a, a).sqrt()
}

fn unit(a: Point3) -> Result<Point3> {
    let length = norm(a);
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("direction"));
    }
    Ok(scale(a, 1.0 / length))
}

fn line_direction(curve: &Curve) -> Result<(Point3, Point3, Point3)> {
    let Curve::Line { start, end, .. } = curve else {
        return Err(Error::Unsupported("line-only operation"));
    };
    let direction = unit(sub(*end, *start))?;
    Ok((*start, *end, direction))
}

fn make_line(start: Point3, end: Point3) -> Result<Curve> {
    let length = norm(sub(end, start));
    if !length.is_finite() || length <= 0.0 {
        return Err(Error::Invalid("degenerate line"));
    }
    finite(start)?;
    finite(end)?;
    Ok(Curve::Line {
        start,
        end,
        domain: [0.0, length],
    })
}

fn bezier(poles: Vec<Point3>, degree: usize, limits: &Limits) -> Result<Curve> {
    let mut knots = vec![0.0; degree + 1];
    knots.extend(std::iter::repeat_n(1.0, degree + 1));
    let nurbs = NurbsCurve {
        degree,
        poles,
        weights: vec![1.0; degree + 1],
        knots,
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    nurbs.validate(limits)?;
    Ok(Curve::Nurbs(nurbs))
}

/* -------------------------------- offset -------------------------------- */

/// Offset a curve by `distance` along `plane_normal`.
///
/// The sign follows [`OffsetCurve`]: `P = p + d * (p' × V) / |p' × V|`, so a planar
/// line whose frame normal is `+Z` moves towards `-Y` for a positive distance. A line
/// and a circle stay exact (a new line, or a circle with a corrected radius); a NURBS
/// curve is sampled and fitted. A polycurve is offset segment-wise and refused when the
/// offset members no longer meet (corners would need a miter/arc that is not claimed).
pub fn offset_curve(
    curve: &Curve,
    distance: f64,
    plane_normal: Point3,
    samples: usize,
    limits: &Limits,
) -> Result<Curve> {
    if !distance.is_finite() {
        return Err(Error::Invalid("offset distance"));
    }
    if samples < 4 {
        return Err(Error::Invalid("offset samples"));
    }
    let normal = unit(plane_normal)?;
    curve.validate(limits)?;
    match curve {
        Curve::Line { start, end, .. } => {
            let direction = unit(sub(*end, *start))?;
            let offset = unit(cross(direction, normal))?;
            let shift = scale(offset, distance);
            make_line(add(*start, shift), add(*end, shift))
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let circle_normal = unit(cross(*x, *y))?;
            let sign = if dot(circle_normal, normal) >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let new_radius = radius + distance * sign;
            if !new_radius.is_finite() || new_radius <= 0.0 {
                return Err(Error::Invalid("offset circle radius"));
            }
            let angles = angle_domain.unwrap_or(*domain);
            Ok(Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: new_radius,
                domain: [0.0, 1.0],
                angle_domain: Some(angles),
            })
        }
        Curve::Nurbs(nurbs) => {
            if nurbs.reversed {
                return Err(Error::Unsupported("offset of a reversed NURBS record"));
            }
            let basis = NurbsBasis::new(nurbs);
            let offset = OffsetCurve::new(basis, distance, normal)?;
            let [start, end] = nurbs.domain;
            let mut points = Vec::new();
            points
                .try_reserve_exact(samples)
                .map_err(|_| Error::Budget("offset samples"))?;
            for index in 0..samples {
                let fraction = index as f64 / (samples - 1) as f64;
                points.push(offset.point(start + fraction * (end - start))?);
            }
            let degree = nurbs.degree.clamp(1, 3);
            Ok(Curve::Nurbs(interpolate_any(
                &points,
                degree,
                KnotMode::ChordLength,
                limits,
            )?))
        }
        Curve::Polycurve { .. } => {
            let segments = crate::curve_ops::explode_curve(curve, limits)?;
            let mut offset_segments = Vec::new();
            for segment in &segments {
                offset_segments.push(offset_curve(segment, distance, normal, samples, limits)?);
            }
            crate::curve_ops::join_curves(&offset_segments, 1e-7, limits)
                .map_err(|_| Error::Unsupported("offset polycurve corner needs a miter or arc"))
        }
    }
}

/* --------------------------- fillet / chamfer --------------------------- */

/// The two trimmed legs and the connecting arc of a line-line fillet.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerFillet {
    pub arc: Curve,
    pub first_trimmed: Curve,
    pub second_trimmed: Curve,
    pub tangent_points: [Point3; 2],
}

/// Corner geometry shared by the fillet and the chamfer.
struct Corner {
    /// Intersection point.
    point: Point3,
    /// Unit directions from the intersection towards each line's far end.
    directions: [Point3; 2],
    /// Distance from the intersection to each line's far end.
    spans: [f64; 2],
    /// The original line's start was the far end (so the trimmed record keeps `start`).
    start_is_far: [bool; 2],
}

fn line_intersection(first: &Curve, second: &Curve, tolerance: f64) -> Result<Corner> {
    let (a0, a1, da) = line_direction(first)?;
    let (b0, b1, db) = line_direction(second)?;
    let cross_d = cross(da, db);
    let denom = dot(cross_d, cross_d);
    if !denom.is_finite() || denom < 1e-20 {
        return Err(Error::Invalid("parallel fillet lines"));
    }
    let delta = sub(b0, a0);
    let t = dot(cross(delta, db), cross_d) / denom;
    let u = dot(cross(delta, da), cross_d) / denom;
    let on_a = add(a0, scale(da, t));
    let on_b = add(b0, scale(db, u));
    if norm(sub(on_a, on_b)) > tolerance.max(1e-7) {
        return Err(Error::Invalid("non-intersecting fillet lines"));
    }
    let point = scale(add(on_a, on_b), 0.5);
    // Oriented distance of each endpoint from the intersection along the line.
    let project =
        |endpoint: Point3, origin: Point3, direction: Point3| dot(sub(endpoint, origin), direction);
    let a_far_start = project(a0, point, da) > project(a1, point, da);
    let b_far_start = project(b0, point, db) > project(b1, point, db);
    let a_far = if a_far_start { a0 } else { a1 };
    let b_far = if b_far_start { b0 } else { b1 };
    let dir_a = unit(sub(a_far, point))?;
    let dir_b = unit(sub(b_far, point))?;
    let span_a = norm(sub(a_far, point));
    let span_b = norm(sub(b_far, point));
    Ok(Corner {
        point,
        directions: [dir_a, dir_b],
        spans: [span_a, span_b],
        start_is_far: [a_far_start, b_far_start],
    })
}

fn trimmed_line(corner: &Corner, index: usize, from: Point3) -> Result<Curve> {
    let far = add(
        corner.point,
        scale(corner.directions[index], corner.spans[index]),
    );
    if corner.start_is_far[index] {
        make_line(far, from)
    } else {
        make_line(from, far)
    }
}

/// Fillet two straight lines with one tangent arc. Both records must intersect inside
/// `tolerance` (1e-7 mm minimum); the arc is exact and the returned legs are trimmed.
pub fn fillet_lines(
    first: &Curve,
    second: &Curve,
    radius: f64,
    plane_normal: Point3,
    tolerance: f64,
    limits: &Limits,
) -> Result<CornerFillet> {
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Error::Invalid("fillet radius"));
    }
    let normal = unit(plane_normal)?;
    let corner = line_intersection(first, second, tolerance)?;
    let cos_theta = dot(corner.directions[0], corner.directions[1]).clamp(-1.0, 1.0);
    let theta = cos_theta.acos();
    if !theta.is_finite() || theta <= 1e-9 || (std::f64::consts::PI - theta).abs() <= 1e-9 {
        return Err(Error::Invalid("fillet corner angle"));
    }
    let tangent_length = radius / (theta * 0.5).tan();
    if tangent_length > corner.spans[0] || tangent_length > corner.spans[1] {
        return Err(Error::Invalid("fillet radius exceeds the line length"));
    }
    let t0 = add(corner.point, scale(corner.directions[0], tangent_length));
    let t1 = add(corner.point, scale(corner.directions[1], tangent_length));
    let bisector = unit(add(corner.directions[0], corner.directions[1]))?;
    let center = add(corner.point, scale(bisector, radius / (theta * 0.5).sin()));
    let middle_direction = unit(add(unit(sub(t0, center))?, unit(sub(t1, center))?))
        .unwrap_or_else(|_| unit(sub(scale(add(t0, t1), 0.5), center)).unwrap_or(normal));
    let middle = add(center, scale(middle_direction, radius));
    let arc = arc_through(t0, middle, t1)?;
    let first_trimmed = trimmed_line(&corner, 0, t0)?;
    let second_trimmed = trimmed_line(&corner, 1, t1)?;
    first_trimmed.validate(limits)?;
    second_trimmed.validate(limits)?;
    arc.validate(limits)?;
    Ok(CornerFillet {
        arc,
        first_trimmed,
        second_trimmed,
        tangent_points: [t0, t1],
    })
}

/// The two trimmed legs and the connecting line of a line-line chamfer.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerChamfer {
    pub line: Curve,
    pub first_trimmed: Curve,
    pub second_trimmed: Curve,
    pub tangent_points: [Point3; 2],
}

/// Chamfer two straight lines with distances measured from their intersection.
pub fn chamfer_lines(
    first: &Curve,
    second: &Curve,
    first_distance: f64,
    second_distance: f64,
    tolerance: f64,
    limits: &Limits,
) -> Result<CornerChamfer> {
    if !first_distance.is_finite() || first_distance <= 0.0 {
        return Err(Error::Invalid("chamfer first distance"));
    }
    if !second_distance.is_finite() || second_distance <= 0.0 {
        return Err(Error::Invalid("chamfer second distance"));
    }
    let corner = line_intersection(first, second, tolerance)?;
    if first_distance > corner.spans[0] || second_distance > corner.spans[1] {
        return Err(Error::Invalid("chamfer distance exceeds the line length"));
    }
    let t0 = add(corner.point, scale(corner.directions[0], first_distance));
    let t1 = add(corner.point, scale(corner.directions[1], second_distance));
    let line = make_line(t0, t1)?;
    let first_trimmed = trimmed_line(&corner, 0, t0)?;
    let second_trimmed = trimmed_line(&corner, 1, t1)?;
    line.validate(limits)?;
    Ok(CornerChamfer {
        line,
        first_trimmed,
        second_trimmed,
        tangent_points: [t0, t1],
    })
}

/// ArcBlend: the tangent arc that joins two lines (the fillet arc alone).
pub fn arc_blend_lines(
    first: &Curve,
    second: &Curve,
    radius: f64,
    plane_normal: Point3,
    tolerance: f64,
    limits: &Limits,
) -> Result<Curve> {
    Ok(fillet_lines(first, second, radius, plane_normal, tolerance, limits)?.arc)
}

/* --------------------------------- blend --------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continuity {
    G0,
    G1,
    G2,
}

fn endpoint_data(
    curve: &Curve,
    at_end: bool,
    continuity: Continuity,
    limits: &Limits,
) -> Result<(Point3, Point3, Point3)> {
    let fraction = if at_end { 1.0 } else { 0.0 };
    let point = point_at(curve, fraction, limits)?;
    let first = derivative_at(curve, fraction, limits)?;
    let second = if continuity == Continuity::G2 {
        crate::curve_ops::fraction_derivatives(curve, fraction, 2, limits)?[2]
    } else {
        [0.0; 3]
    };
    Ok((point, first, second))
}

/// Blend the end of `first` to the start of `second` with the requested continuity.
///
/// The bridge is a Bézier record over `[0, 1]`: a line for G0, a cubic Hermite for G1
/// and a quintic Hermite for G2. Its end derivatives equal the source end derivatives
/// scaled to the blend parameter, so G1/G2 are exact in that parameterisation.
pub fn blend_curves(
    first: &Curve,
    second: &Curve,
    continuity: Continuity,
    limits: &Limits,
) -> Result<Curve> {
    let (p0, d0, dd0) = endpoint_data(first, true, continuity, limits)?;
    let (p1, d1, dd1) = endpoint_data(second, false, continuity, limits)?;
    match continuity {
        Continuity::G0 => {
            if norm(sub(p1, p0)) <= 1e-12 {
                return Err(Error::Invalid("coincident blend points"));
            }
            make_line(p0, p1)
        }
        Continuity::G1 => {
            let p1_control = add(p0, scale(d0, 1.0 / 3.0));
            let p2_control = sub(p1, scale(d1, 1.0 / 3.0));
            bezier(vec![p0, p1_control, p2_control, p1], 3, limits)
        }
        Continuity::G2 => {
            let first_control = add(p0, scale(d0, 1.0 / 5.0));
            let second_control = add(
                add(scale(dd0, 1.0 / 20.0), scale(first_control, 2.0)),
                scale(p0, -1.0),
            );
            let penultimate = sub(p1, scale(d1, 1.0 / 5.0));
            let third_control = add(
                add(scale(dd1, 1.0 / 20.0), scale(penultimate, 2.0)),
                scale(p1, -1.0),
            );
            bezier(
                vec![
                    p0,
                    first_control,
                    second_control,
                    third_control,
                    penultimate,
                    p1,
                ],
                5,
                limits,
            )
        }
    }
}

/* --------------------------------- match --------------------------------- */

/// Move the end of `source` onto the `target` end with G0/G1/G2 continuity.
///
/// Only clamped, non-rational NURBS records are admitted (a line is promoted to a
/// degree-2 NURBS first). The last span is treated as the Bézier segment it is, so the
/// end derivatives come from the last `degree + 1` poles.
pub fn match_curve(
    source: &Curve,
    source_end: bool,
    target: &Curve,
    target_end: bool,
    continuity: Continuity,
    limits: &Limits,
) -> Result<Curve> {
    let source = if source_end {
        source.clone()
    } else {
        reverse_curve(source, limits)?
    };
    let mut nurbs = match &source {
        Curve::Nurbs(nurbs) => nurbs.clone(),
        Curve::Line { start, end, .. } => {
            match crate::curve_ops::line_to_nurbs(*start, *end, limits)? {
                Curve::Nurbs(nurbs) => nurbs,
                _ => return Err(Error::Invalid("line promotion")),
            }
        }
        _ => return Err(Error::Unsupported("match source kind")),
    };
    if nurbs
        .weights
        .iter()
        .any(|weight| (*weight - 1.0).abs() > 1e-12)
    {
        return Err(Error::Unsupported("match on a rational curve"));
    }
    if nurbs.reversed {
        return Err(Error::Unsupported("match on a reversed NURBS record"));
    }
    if nurbs.degree < 1 {
        return Err(Error::Invalid("match degree"));
    }
    // A G1/G2 end constraint changes the last `continuity + 1` poles; keep the other
    // end untouched by promoting a low-degree record first.
    let required = match continuity {
        Continuity::G0 => 1,
        Continuity::G1 => 2,
        Continuity::G2 => 3,
    };
    while nurbs.degree < required {
        nurbs = elevate_degree(&nurbs, limits)?;
    }
    let (point, first, second) = endpoint_data(target, target_end, continuity, limits)?;
    let degree = nurbs.degree;
    let count = nurbs.poles.len();
    if count <= degree {
        return Err(Error::Invalid("match pole count"));
    }
    let last = count - 1;
    let span = nurbs.knots[count] - nurbs.knots[last];
    if !span.is_finite() || span <= 0.0 {
        return Err(Error::Invalid("match end span"));
    }
    // Recompute the end derivatives of the (possibly promoted) source span from its poles.
    nurbs.poles[last] = point;
    if continuity != Continuity::G0 {
        nurbs.poles[last - 1] = sub(nurbs.poles[last], scale(first, span / degree as f64));
    }
    if continuity == Continuity::G2 {
        let factor = (degree * (degree - 1)) as f64 / (span * span);
        nurbs.poles[last - 2] = sub(
            add(
                scale(second, 1.0 / factor),
                scale(nurbs.poles[last - 1], 2.0),
            ),
            nurbs.poles[last],
        );
    }
    nurbs.validate(limits)?;
    let result = Curve::Nurbs(nurbs);
    if source_end {
        Ok(result)
    } else {
        reverse_curve(&result, limits)
    }
}

/* ---------------------------- rebuild / fit ---------------------------- */

/// Rebuild a curve through `samples` stations with a new degree and control count.
/// Delegates to the existing interpolation and least-squares fit kernels.
pub fn rebuild_curve(
    curve: &Curve,
    samples: usize,
    degree: usize,
    control_points: Option<usize>,
    mode: KnotMode,
    limits: &Limits,
) -> Result<Curve> {
    if samples < 2 {
        return Err(Error::Invalid("rebuild sample count"));
    }
    let points = crate::curve_ops::sample_curve(curve, samples, limits)?;
    Ok(match control_points {
        None => Curve::Nurbs(interpolate_any(&points, degree, mode, limits)?),
        Some(count) => Curve::Nurbs(fit_points(&points, degree, count, mode, limits)?),
    })
}
