// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve/surface interaction owned by OM9 (OM9-L2-02-04): closest-point projection,
//! pullback, curve/curve and curve/surface intersections, mesh projection, two-view
//! point reconstruction, sketch-on-surface and B-Rep edge/isocurve extraction.
//!
//! Exact line/line and coplanar circle intersections are analytic; every other family
//! is a bounded numeric solver over the shared evaluator, and the measured deviation is
//! returned so the caller can refuse a result that is not within tolerance.
use crate::{
    Error, Result,
    brep::Brep,
    budget::Limits,
    curve_build::{KnotMode, interpolate_any},
    curve_ops::{fraction_derivatives, point_at},
    geometry::{Curve, Mesh, NurbsCurve, NurbsSurface, Point3, finite},
    surface_kernel::{self, SurfaceIso},
};
use uuid::Uuid;

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

fn distance(a: Point3, b: Point3) -> f64 {
    norm(sub(a, b))
}

fn clamp(value: f64, low: f64, high: f64) -> f64 {
    value.max(low).min(high)
}

/* --------------------------- surface projection --------------------------- */

/// A projected surface point with its two parameters and the measured deviation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceProjection {
    pub parameters: [f64; 2],
    pub point: Point3,
    pub distance: f64,
}

fn surface_distance(
    surface: &NurbsSurface,
    point: Point3,
    uv: [f64; 2],
    limits: &Limits,
) -> Result<(f64, Point3)> {
    let value = surface_kernel::evaluate(surface, uv[0], uv[1], limits)?;
    Ok((distance(value, point), value))
}

/// Closest point on a NURBS surface: a coarse grid followed by damped Gauss–Newton on
/// the two parameters.
pub fn closest_point_on_surface(
    surface: &NurbsSurface,
    point: Point3,
    limits: &Limits,
) -> Result<SurfaceProjection> {
    surface.validate(limits)?;
    finite(point)?;
    let grid = 16usize;
    let mut best = [surface.domain[0][0], surface.domain[1][0]];
    let mut best_distance = f64::INFINITY;
    for i in 0..=grid {
        let u = surface.domain[0][0]
            + (surface.domain[0][1] - surface.domain[0][0]) * i as f64 / grid as f64;
        for j in 0..=grid {
            let v = surface.domain[1][0]
                + (surface.domain[1][1] - surface.domain[1][0]) * j as f64 / grid as f64;
            let (measured, _) = surface_distance(surface, point, [u, v], limits)?;
            if measured < best_distance {
                best_distance = measured;
                best = [u, v];
            }
        }
    }
    let mut uv = best;
    for _ in 0..48 {
        let derivatives = surface_kernel::derivatives(surface, uv[0], uv[1], limits)?;
        let value = derivatives.point;
        let residual = sub(value, point);
        let a = dot(derivatives.du, derivatives.du);
        let b = dot(derivatives.du, derivatives.dv);
        let c = dot(derivatives.dv, derivatives.dv);
        let g1 = dot(derivatives.du, residual);
        let g2 = dot(derivatives.dv, residual);
        let determinant = a * c - b * b;
        if !determinant.is_finite() || determinant.abs() <= 1e-18 {
            break;
        }
        let step_u = -(c * g1 - b * g2) / determinant;
        let step_v = (b * g1 - a * g2) / determinant;
        if !step_u.is_finite() || !step_v.is_finite() {
            break;
        }
        let mut factor = 1.0;
        let mut moved = false;
        for _ in 0..16 {
            let candidate = [
                clamp(
                    uv[0] + step_u * factor,
                    surface.domain[0][0],
                    surface.domain[0][1],
                ),
                clamp(
                    uv[1] + step_v * factor,
                    surface.domain[1][0],
                    surface.domain[1][1],
                ),
            ];
            let (measured, _) = surface_distance(surface, point, candidate, limits)?;
            if measured < best_distance - 1e-14 {
                uv = candidate;
                best_distance = measured;
                moved = true;
                break;
            }
            factor *= 0.5;
        }
        if !moved {
            break;
        }
    }
    let (measured, value) = surface_distance(surface, point, uv, limits)?;
    Ok(SurfaceProjection {
        parameters: uv,
        point: value,
        distance: measured,
    })
}

/// Closest point on a curve, by a bounded golden-section search around the best of 64
/// coarse samples.
pub fn closest_point_on_curve(
    curve: &Curve,
    point: Point3,
    limits: &Limits,
) -> Result<(f64, Point3, f64)> {
    finite(point)?;
    let samples = 64;
    let mut best = 0.0;
    let mut best_distance = f64::INFINITY;
    for index in 0..=samples {
        let fraction = index as f64 / samples as f64;
        let value = point_at(curve, fraction, limits)?;
        let measured = distance(value, point);
        if measured < best_distance {
            best_distance = measured;
            best = fraction;
        }
    }
    let span = 1.0 / samples as f64;
    let (mut low, mut high) = ((best - span).max(0.0), (best + span).min(1.0));
    let golden = 0.618_033_988_749_895;
    let mut c = high - golden * (high - low);
    let mut d = low + golden * (high - low);
    let mut fc = distance(point_at(curve, c, limits)?, point);
    let mut fd = distance(point_at(curve, d, limits)?, point);
    for _ in 0..80 {
        if fc < fd {
            high = d;
            d = c;
            fd = fc;
            c = high - golden * (high - low);
            fc = distance(point_at(curve, c, limits)?, point);
        } else {
            low = c;
            c = d;
            fc = fd;
            d = low + golden * (high - low);
            fd = distance(point_at(curve, d, limits)?, point);
        }
    }
    let fraction = 0.5 * (low + high);
    let value = point_at(curve, fraction, limits)?;
    Ok((fraction, value, distance(value, point)))
}

/* -------------------------------- pullback -------------------------------- */

/// A curve pulled back to a surface: the surface parameters, the projected points and
/// the fitted three-dimensional curve.
#[derive(Debug, Clone, PartialEq)]
pub struct Pullback {
    pub parameters: Vec<[f64; 2]>,
    pub points: Vec<Point3>,
    pub curve: NurbsCurve,
    pub max_deviation: f64,
}

/// Project each of `samples` stations of `curve` onto `surface` and fit the projected
/// points. A deviation above `tolerance` is refused instead of returning a curve that
/// only looks attached to the surface.
pub fn pullback_curve(
    curve: &Curve,
    surface: &NurbsSurface,
    samples: usize,
    tolerance: f64,
    limits: &Limits,
) -> Result<Pullback> {
    if samples < 2 {
        return Err(Error::Invalid("pullback sample count"));
    }
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("pullback tolerance"));
    }
    surface.validate(limits)?;
    let mut parameters = Vec::new();
    let mut points = Vec::new();
    let mut max_deviation: f64 = 0.0;
    parameters
        .try_reserve_exact(samples)
        .map_err(|_| Error::Budget("pullback parameters"))?;
    points
        .try_reserve_exact(samples)
        .map_err(|_| Error::Budget("pullback points"))?;
    for index in 0..samples {
        let fraction = index as f64 / (samples - 1) as f64;
        let source = point_at(curve, fraction, limits)?;
        let projection = closest_point_on_surface(surface, source, limits)?;
        if projection.distance > tolerance {
            return Err(Error::Invalid("pullback deviation"));
        }
        max_deviation = max_deviation.max(projection.distance);
        parameters.push(projection.parameters);
        points.push(projection.point);
    }
    let degree = 3.min(points.len().saturating_sub(1)).max(1);
    let fitted = interpolate_any(&points, degree, KnotMode::ChordLength, limits)?;
    Ok(Pullback {
        parameters,
        points,
        curve: fitted,
        max_deviation,
    })
}

/* ---------------------------- intersections ---------------------------- */

/// One measured intersection. `distance` is the closest approach after refinement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveIntersection {
    pub point: Point3,
    pub first_fraction: f64,
    pub second_fraction: f64,
    pub distance: f64,
}

/// One curve/surface intersection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceIntersection {
    pub point: Point3,
    pub curve_fraction: f64,
    pub parameters: [f64; 2],
    pub distance: f64,
}

fn circle_fraction(circle: &Curve, point: Point3) -> Result<f64> {
    let Curve::Circle {
        center,
        x,
        y,
        domain,
        angle_domain,
        ..
    } = circle
    else {
        return Err(Error::Invalid("circle"));
    };
    let angles = angle_domain.unwrap_or(*domain);
    let local = sub(point, *center);
    let angle = dot(local, *y).atan2(dot(local, *x));
    let span = angles[1] - angles[0];
    if !span.is_finite() || span <= 0.0 {
        return Err(Error::Invalid("circle angle domain"));
    }
    let mut fraction = (angle - angles[0]) / span;
    while fraction < 0.0 {
        fraction += 1.0;
    }
    while fraction > 1.0 {
        fraction -= 1.0;
    }
    Ok(fraction)
}

fn line_line(first: &Curve, second: &Curve, tolerance: f64) -> Result<Vec<CurveIntersection>> {
    let (
        Curve::Line {
            start: p1, end: e1, ..
        },
        Curve::Line {
            start: p2, end: e2, ..
        },
    ) = (first, second)
    else {
        return Ok(Vec::new());
    };
    let d1 = sub(*e1, *p1);
    let d2 = sub(*e2, *p2);
    let a = dot(d1, d1);
    let b = dot(d1, d2);
    let c = dot(d2, d2);
    let w = sub(*p1, *p2);
    let d = dot(d1, w);
    let e = dot(d2, w);
    let denominator = a * c - b * b;
    if !denominator.is_finite() || denominator.abs() <= 1e-20 {
        return Ok(Vec::new());
    }
    let s = (b * e - c * d) / denominator;
    let t = (a * e - b * d) / denominator;
    if !(0.0..=1.0).contains(&s) || !(0.0..=1.0).contains(&t) {
        return Ok(Vec::new());
    }
    let left = add(*p1, scale(d1, s));
    let right = add(*p2, scale(d2, t));
    let measured = distance(left, right);
    if measured > tolerance {
        return Ok(Vec::new());
    }
    Ok(vec![CurveIntersection {
        point: scale(add(left, right), 0.5),
        first_fraction: clamp(s, 0.0, 1.0),
        second_fraction: clamp(t, 0.0, 1.0),
        distance: measured,
    }])
}

fn coplanar_circles(
    first: &Curve,
    second: &Curve,
    tolerance: f64,
) -> Result<Option<Vec<CurveIntersection>>> {
    let (
        Curve::Circle {
            center: c1,
            x: x1,
            y: y1,
            radius: r1,
            ..
        },
        Curve::Circle {
            center: c2,
            x: x2,
            y: y2,
            radius: r2,
            ..
        },
    ) = (first, second)
    else {
        return Ok(None);
    };
    let n1 = cross(*x1, *y1);
    let n2 = cross(*x2, *y2);
    if norm(cross(n1, n2)) > 1e-9 || dot(sub(*c2, *c1), n1).abs() > tolerance.max(1e-9) {
        return Ok(None);
    }
    let centers = sub(*c2, *c1);
    let d = norm(centers);
    if d <= 1e-12 {
        return Ok(Some(Vec::new()));
    }
    if d > r1 + r2 + tolerance || d < (r1 - r2).abs() - tolerance {
        return Ok(Some(Vec::new()));
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h_squared = r1 * r1 - a * a;
    let h = if h_squared > 0.0 {
        h_squared.sqrt()
    } else {
        0.0
    };
    let unit = scale(centers, 1.0 / d);
    let base = add(*c1, scale(unit, a));
    let perpendicular = cross(n1, unit);
    let perpendicular = if norm(perpendicular) > 1e-12 {
        scale(perpendicular, h / norm(perpendicular))
    } else {
        [0.0; 3]
    };
    let candidates = if h <= 1e-12 {
        vec![base]
    } else {
        vec![add(base, perpendicular), sub(base, perpendicular)]
    };
    let mut result = Vec::new();
    for point in candidates {
        result.push(CurveIntersection {
            point,
            first_fraction: circle_fraction(first, point)?,
            second_fraction: circle_fraction(second, point)?,
            distance: 0.0,
        });
    }
    Ok(Some(result))
}

fn numeric_intersections(
    first: &Curve,
    second: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<CurveIntersection>> {
    let samples = 256usize;
    let mut candidates: Vec<(f64, f64)> = Vec::new();
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    for index in 0..=samples {
        let point = point_at(first, index as f64 / samples as f64, limits)?;
        for axis in 0..3 {
            low[axis] = low[axis].min(point[axis]);
            high[axis] = high[axis].max(point[axis]);
        }
    }
    for index in 0..=samples {
        let point = point_at(second, index as f64 / samples as f64, limits)?;
        for axis in 0..3 {
            low[axis] = low[axis].min(point[axis]);
            high[axis] = high[axis].max(point[axis]);
        }
    }
    let diagonal = norm(sub(high, low));
    let span_scale = if diagonal.is_finite() && diagonal > 0.0 {
        diagonal
    } else {
        1.0
    };
    let threshold = (span_scale / samples as f64 * 3.0)
        .max(tolerance * 8.0)
        .max(1e-6);
    for index in 0..=samples {
        let fraction = index as f64 / samples as f64;
        let point = point_at(first, fraction, limits)?;
        let mut best_other = 0.0;
        let mut best_distance = f64::INFINITY;
        for other in 0..=samples {
            let other_fraction = other as f64 / samples as f64;
            let measured = distance(point, point_at(second, other_fraction, limits)?);
            if measured < best_distance {
                best_distance = measured;
                best_other = other_fraction;
            }
        }
        if best_distance <= threshold {
            candidates.push((fraction, best_other));
        }
    }
    candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut refined: Vec<CurveIntersection> = Vec::new();
    let mut last_fraction = -1.0;
    for (mut f, mut g) in candidates {
        if f - last_fraction < 2.0 / samples as f64 {
            continue;
        }
        last_fraction = f;
        let mut best = distance(point_at(first, f, limits)?, point_at(second, g, limits)?);
        let mut factor = 0.5;
        for _ in 0..40 {
            let a = fraction_derivatives(first, f, 1, limits)?;
            let b = fraction_derivatives(second, g, 1, limits)?;
            let residual = sub(a[0], b[0]);
            let aa = dot(a[1], a[1]);
            let ab = dot(a[1], b[1]);
            let bb = dot(b[1], b[1]);
            let g1 = dot(a[1], residual);
            let g2 = -dot(b[1], residual);
            let determinant = aa * bb - ab * ab;
            if !determinant.is_finite() || determinant.abs() <= 1e-20 {
                break;
            }
            let step_f = -(bb * g1 - ab * g2) / determinant;
            let step_g = -(-ab * g1 + aa * g2) / determinant;
            let candidate_f = clamp(f + step_f * factor, 0.0, 1.0);
            let candidate_g = clamp(g + step_g * factor, 0.0, 1.0);
            let measured = distance(
                point_at(first, candidate_f, limits)?,
                point_at(second, candidate_g, limits)?,
            );
            if measured < best {
                best = measured;
                f = candidate_f;
                g = candidate_g;
            } else {
                factor *= 0.5;
                if factor < 1e-9 {
                    break;
                }
            }
        }
        if best > tolerance {
            continue;
        }
        let point = scale(
            add(point_at(first, f, limits)?, point_at(second, g, limits)?),
            0.5,
        );
        if refined
            .iter()
            .any(|existing| distance(existing.point, point) <= tolerance.max(1e-6))
        {
            continue;
        }
        refined.push(CurveIntersection {
            point,
            first_fraction: f,
            second_fraction: g,
            distance: best,
        });
    }
    Ok(refined)
}

/// Intersections of two curves. Lines and coplanar circles use closed forms; every
/// other pair refines a sampled candidate set and returns only points within
/// `tolerance`.
pub fn curve_curve_intersections(
    first: &Curve,
    second: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<CurveIntersection>> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("intersection tolerance"));
    }
    first.validate(limits)?;
    second.validate(limits)?;
    let exact = line_line(first, second, tolerance)?;
    if !exact.is_empty() {
        return Ok(exact);
    }
    if let Some(circles) = coplanar_circles(first, second, tolerance)? {
        return Ok(circles);
    }
    numeric_intersections(first, second, tolerance, limits)
}

/// Intersections of a curve and a NURBS surface. The solver samples the curve, keeps
/// local distance minima and refines each in `(t, u, v)` with alternating projections.
pub fn curve_surface_intersections(
    curve: &Curve,
    surface: &NurbsSurface,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<SurfaceIntersection>> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("intersection tolerance"));
    }
    surface.validate(limits)?;
    let samples = 192usize;
    let mut distances = Vec::new();
    let mut projections = Vec::new();
    distances
        .try_reserve_exact(samples + 1)
        .map_err(|_| Error::Budget("surface intersections"))?;
    projections
        .try_reserve_exact(samples + 1)
        .map_err(|_| Error::Budget("surface intersections"))?;
    for index in 0..=samples {
        let source = point_at(curve, index as f64 / samples as f64, limits)?;
        let projection = closest_point_on_surface(surface, source, limits)?;
        distances.push(projection.distance);
        projections.push(projection);
    }
    let mut result = Vec::new();
    for index in 1..samples {
        if distances[index] > tolerance {
            continue;
        }
        if distances[index] > distances[index - 1] || distances[index] > distances[index + 1] {
            continue;
        }
        let mut t = index as f64 / samples as f64;
        let mut uv = projections[index].parameters;
        let mut best = distances[index];
        for _ in 0..24 {
            let source = point_at(curve, t, limits)?;
            let projection = closest_point_on_surface(surface, source, limits)?;
            uv = projection.parameters;
            let (next_t, _, next_distance) =
                closest_point_on_curve(curve, projection.point, limits)?;
            t = next_t;
            if next_distance >= best - 1e-14 {
                best = best.min(next_distance);
                break;
            }
            best = next_distance;
        }
        let point = scale(
            add(
                point_at(curve, t, limits)?,
                surface_kernel::evaluate(surface, uv[0], uv[1], limits)?,
            ),
            0.5,
        );
        if result.iter().any(|existing: &SurfaceIntersection| {
            distance(existing.point, point) <= tolerance.max(1e-6)
        }) {
            continue;
        }
        result.push(SurfaceIntersection {
            point,
            curve_fraction: t,
            parameters: uv,
            distance: best,
        });
    }
    Ok(result)
}

/* --------------------------------- mesh --------------------------------- */

fn closest_on_triangle(point: Point3, a: Point3, b: Point3, c: Point3) -> Point3 {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(point, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(point, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add(a, scale(ab, v));
    }
    let cp = sub(point, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add(a, scale(ac, w));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add(b, scale(sub(c, b), w));
    }
    let denominator = 1.0 / (va + vb + vc);
    let v = vb * denominator;
    let w = vc * denominator;
    add(a, add(scale(ab, v), scale(ac, w)))
}

/// Closest point on a polygon mesh and the distance to it.
pub fn closest_point_on_mesh(mesh: &Mesh, point: Point3, limits: &Limits) -> Result<(Point3, f64)> {
    mesh.validate(limits)?;
    finite(point)?;
    let mut best = [0.0; 3];
    let mut best_distance = f64::INFINITY;
    for face in &mesh.faces {
        let a = mesh.vertices[face[0] as usize];
        let b = mesh.vertices[face[1] as usize];
        for vertex in face.iter().skip(2) {
            let c = mesh.vertices[*vertex as usize];
            let candidate = closest_on_triangle(point, a, b, c);
            let measured = distance(candidate, point);
            if measured < best_distance {
                best_distance = measured;
                best = candidate;
            }
        }
    }
    if !best_distance.is_finite() {
        return Err(Error::Invalid("mesh has no face"));
    }
    Ok((best, best_distance))
}

/// Project world points onto a mesh and fit a curve through the projected points.
pub fn sketch_on_mesh(
    mesh: &Mesh,
    points: &[Point3],
    tolerance: f64,
    limits: &Limits,
) -> Result<Curve> {
    if points.len() < 2 {
        return Err(Error::Invalid("sketch point count"));
    }
    let mut projected = Vec::new();
    projected
        .try_reserve_exact(points.len())
        .map_err(|_| Error::Budget("mesh sketch points"))?;
    for point in points {
        let (candidate, measured) = closest_point_on_mesh(mesh, *point, limits)?;
        if measured > tolerance {
            return Err(Error::Invalid("mesh sketch deviation"));
        }
        projected.push(candidate);
    }
    let degree = 3.min(projected.len().saturating_sub(1)).max(1);
    Ok(Curve::Nurbs(interpolate_any(
        &projected,
        degree,
        KnotMode::ChordLength,
        limits,
    )?))
}

/* ------------------------------ two views ------------------------------ */

/// Point reconstructed from two view rays. The rays must pass within `tolerance`.
pub fn point_from_two_views(
    first_origin: Point3,
    first_direction: Point3,
    second_origin: Point3,
    second_direction: Point3,
    tolerance: f64,
) -> Result<Point3> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("view tolerance"));
    }
    let d1 = first_direction;
    let d2 = second_direction;
    let a = dot(d1, d1);
    let b = dot(d1, d2);
    let c = dot(d2, d2);
    let w = sub(first_origin, second_origin);
    let d = dot(d1, w);
    let e = dot(d2, w);
    let denominator = a * c - b * b;
    if !denominator.is_finite() || denominator.abs() <= 1e-20 {
        return Err(Error::Invalid("parallel view rays"));
    }
    let s = (b * e - c * d) / denominator;
    let t = (a * e - b * d) / denominator;
    let left = add(first_origin, scale(d1, s));
    let right = add(second_origin, scale(d2, t));
    if distance(left, right) > tolerance {
        return Err(Error::Invalid("view rays do not meet"));
    }
    Ok(scale(add(left, right), 0.5))
}

/* --------------------------- sketch on surface --------------------------- */

/// Interpolate a curve through surface points at the given UV parameters.
pub fn sketch_on_surface(
    surface: &NurbsSurface,
    parameters: &[[f64; 2]],
    limits: &Limits,
) -> Result<Curve> {
    if parameters.len() < 2 {
        return Err(Error::Invalid("surface sketch point count"));
    }
    let mut points = Vec::new();
    points
        .try_reserve_exact(parameters.len())
        .map_err(|_| Error::Budget("surface sketch points"))?;
    for [u, v] in parameters {
        points.push(surface_kernel::evaluate(surface, *u, *v, limits)?);
    }
    let degree = 3.min(points.len().saturating_sub(1)).max(1);
    Ok(Curve::Nurbs(interpolate_any(
        &points,
        degree,
        KnotMode::ChordLength,
        limits,
    )?))
}

/* ------------------------- duplicate edge/isocurve ------------------------- */

/// Duplicate a surface iso-curve as a model curve.
pub fn duplicate_isocurve(
    surface: &NurbsSurface,
    iso: SurfaceIso,
    value: f64,
    limits: &Limits,
) -> Result<Curve> {
    Ok(Curve::Nurbs(surface_kernel::isocurve(
        surface, iso, value, limits,
    )?))
}

/// Boundary edges: edges referenced by exactly one display face.
pub fn boundary_edges(brep: &Brep, limits: &Limits) -> Result<Vec<Uuid>> {
    brep.validate(limits)?;
    let mut counts: Vec<usize> = vec![0; brep.display.edges.len()];
    for [_, edge] in &brep.display.face_edges {
        if let Some(slot) = counts.get_mut(*edge as usize) {
            *slot += 1;
        }
    }
    Ok(brep
        .display
        .edges
        .iter()
        .zip(counts)
        .filter(|(_, count)| *count == 1)
        .map(|(id, _)| *id)
        .collect())
}

/// Duplicate one B-Rep display edge as a chain of line segments.
///
/// The display snapshot is the tessellated representation of the exact B-Rep edge, so
/// the returned curve matches the displayed edge at the snapshot deflection; it is not
/// the exact analytic edge curve (that adapter belongs to the B-Rep owner).
pub fn duplicate_brep_edge(brep: &Brep, edge: Uuid, limits: &Limits) -> Result<Curve> {
    brep.validate(limits)?;
    let index = brep
        .display
        .edges
        .iter()
        .position(|id| *id == edge)
        .ok_or(Error::Missing("B-Rep edge"))?;
    let segments: Vec<[u32; 2]> = brep
        .display
        .segments
        .iter()
        .filter(|[_, _, edge_index]| *edge_index as usize == index)
        .map(|[a, b, _]| [*a, *b])
        .collect();
    if segments.is_empty() {
        return Err(Error::Missing("B-Rep edge segments"));
    }
    let point = |offset: u32| -> Result<Point3> {
        brep.display
            .points
            .get(offset as usize)
            .copied()
            .ok_or(Error::Missing("B-Rep display point"))
    };
    // Chain the unordered display segments through their shared vertex offsets. An open
    // edge is walked from a degree-one endpoint; a closed edge may start anywhere.
    let mut remaining = segments;
    let mut degree = std::collections::BTreeMap::new();
    for [a, b] in &remaining {
        *degree.entry(*a).or_insert(0usize) += 1;
        *degree.entry(*b).or_insert(0usize) += 1;
    }
    let mut start = remaining[0][0];
    if let Some((endpoint, _)) = degree.iter().find(|(_, count)| **count == 1) {
        start = *endpoint;
    }
    let mut previous = u32::MAX;
    let mut ordered = vec![point(start)?];
    while !remaining.is_empty() {
        let position = remaining
            .iter()
            .position(|[a, b]| (*a == start && *b != previous) || (*b == start && *a != previous))
            .or_else(|| {
                remaining
                    .iter()
                    .position(|[a, b]| *a == start || *b == start)
            })
            .ok_or(Error::Invalid("B-Rep edge chain"))?;
        let [a, b] = remaining.remove(position);
        previous = start;
        start = if a == start { b } else { a };
        ordered.push(point(start)?);
    }
    let mut curves = Vec::new();
    curves
        .try_reserve_exact(ordered.len().saturating_sub(1))
        .map_err(|_| Error::Budget("B-Rep edge segments"))?;
    for pair in ordered.windows(2) {
        let length = distance(pair[0], pair[1]);
        if length <= 0.0 {
            continue;
        }
        curves.push(Curve::Line {
            start: pair[0],
            end: pair[1],
            domain: [0.0, length],
        });
    }
    if curves.is_empty() {
        return Err(Error::Invalid("degenerate B-Rep edge"));
    }
    if curves.len() == 1 {
        return Ok(curves.remove(0));
    }
    Ok(Curve::Polycurve {
        segments: curves,
        reversed: false,
    })
}
