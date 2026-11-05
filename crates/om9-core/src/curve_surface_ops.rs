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
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Mesh, NurbsCurve, NurbsSurface, Point3, Transform, finite},
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

/// Closest point on a curve, by bounded sampling and golden-section refinement.
pub fn closest_point_on_curve(
    curve: &Curve,
    point: Point3,
    limits: &Limits,
) -> Result<(f64, Point3, f64)> {
    closest_point_on_placed_curve(curve, Transform::identity(), point, limits)
}

/// Find the closest point in world space while preserving the source curve parameter.
/// Nonlinear curves retain the bounded sampled-search limitations of the local solver.
pub fn closest_point_on_placed_curve(
    curve: &Curve,
    placement: Transform,
    point: Point3,
    limits: &Limits,
) -> Result<(f64, Point3, f64)> {
    finite(point)?;
    placement.validate()?;
    if let Curve::Polycurve { segments, reversed } = curve {
        curve.validate(limits)?;
        let mut best: Option<(f64, Point3, f64)> = None;
        for (index, segment) in segments.iter().enumerate() {
            let (local, nearest, measured) =
                closest_point_on_placed_curve(segment, placement, point, limits)?;
            let forward = (index as f64 + local) / segments.len() as f64;
            let fraction = if *reversed { 1.0 - forward } else { forward };
            if best.as_ref().is_none_or(|(_, _, distance)| measured < *distance) {
                best = Some((fraction, nearest, measured));
            }
        }
        return best.ok_or(Error::Invalid("polycurve segments"));
    }
    if let Curve::Line { start, end, .. } = curve {
        curve.validate(limits)?;
        let start = placement.point(*start)?;
        let end = placement.point(*end)?;
        let direction = sub(end, start);
        let squared = dot(direction, direction);
        if squared <= f64::MIN_POSITIVE {
            return Err(Error::Invalid("closest point on degenerate line"));
        }
        let fraction = clamp(dot(sub(point, start), direction) / squared, 0., 1.);
        let value = add(start, scale(direction, fraction));
        return Ok((fraction, value, distance(value, point)));
    }
    let evaluator = CurveEvaluator::capture(curve, limits)?;
    let evaluate = |fraction| {
        evaluator.evaluate(Parameter::Fraction(fraction), 0, placement)?
            .first().copied().ok_or(Error::Invalid("curve point"))
    };
    let additional = if let Curve::Nurbs(nurbs) = curve {
        nurbs.knots.len().checked_mul(2).ok_or(Error::Overflow("projection samples"))?
    } else {
        0
    };
    let total = 65usize.checked_add(additional).ok_or(Error::Overflow("projection samples"))?;
    if total as u64 > limits.max_points || (total as u64).saturating_mul(8) > limits.max_bytes {
        return Err(Error::Budget("projection samples"));
    }
    let mut candidates = Vec::new();
    candidates.try_reserve_exact(total).map_err(|_| Error::Budget("projection samples"))?;
    candidates.extend((0..=64).map(|index| index as f64 / 64.0));
    if let Curve::Nurbs(nurbs) = curve {
        let [start, end] = nurbs.domain;
        let mut last = None;
        for &knot in &nurbs.knots {
            if knot < start || knot > end { continue; }
            let native = (knot - start) / (end - start);
            let mut fraction = if nurbs.reversed { 1.0 - native } else { native };
            if let Some(seam)=nurbs.seam {
                let origin=(seam-start)/(end-start);
                let offset=if nurbs.reversed {1.-origin} else {origin};
                fraction=(fraction-offset).rem_euclid(1.);
            }
            if let Some(previous) = last {
                if previous != fraction {
                    candidates.push(previous + (fraction - previous) * 0.5);
                }
            }
            candidates.push(fraction);
            last = Some(fraction);
        }
    }
    candidates.sort_by(f64::total_cmp);
    candidates.dedup_by(|a, b| *a == *b);
    let mut best_index = 0;
    let mut best_distance = f64::INFINITY;
    for (index, &fraction) in candidates.iter().enumerate() {
        let value = evaluate(fraction)?;
        let measured = distance(value, point);
        if measured < best_distance {
            best_distance = measured;
            best_index = index;
        }
    }
    let best = candidates[best_index];
    let (low, high) = (
        candidates[best_index.saturating_sub(1)],
        candidates[(best_index + 1).min(candidates.len() - 1)],
    );
    let refine = |mut low:f64,mut high:f64| -> Result<(f64,Point3,f64)> {
    let golden = 0.618_033_988_749_895;
    let mut c = high - golden * (high - low);
    let mut d = low + golden * (high - low);
    let mut fc = distance(evaluate(c)?, point);
    let mut fd = distance(evaluate(d)?, point);
    for _ in 0..80 {
        if fc < fd {
            high = d;
            d = c;
            fd = fc;
            c = high - golden * (high - low);
            fc = distance(evaluate(c)?, point);
        } else {
            low = c;
            c = d;
            fc = fd;
            d = low + golden * (high - low);
            fd = distance(evaluate(d)?, point);
        }
    }
    let fraction = 0.5 * (low + high);
    let value = evaluate(fraction)?;
    let measured = distance(value, point);
    Ok((fraction,value,measured))
    };
    let mut answer=(best,evaluate(best)?,best_distance);
    let trial=refine(low,high)?;
    if trial.2<answer.2 {answer=trial;}
    if best_index==0 || best_index+1==candidates.len() {
        let other=if best_index==0 {(candidates[candidates.len()-2],1.)} else {(0.,candidates[1])};
        let trial=refine(other.0,other.1)?;
        if trial.2<answer.2 {answer=trial;}
    }
    Ok(answer)
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

fn circle_fractions(circle: &Curve, point: Point3, tolerance: f64, limits: &Limits) -> Result<Vec<f64>> {
    let Curve::Circle {
        center,
        x,
        y,
        radius,
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
    let tau=std::f64::consts::TAU;
    let mut offset=(angle-angles[0]).rem_euclid(tau);
    if tau-offset <= 8.0*f64::EPSILON*tau && radius*(tau-offset) <= tolerance { offset=0.0; }
    if offset > span {
        let mut endpoints=Vec::new();
        if radius*(tau-offset) <= tolerance {endpoints.push(0.0);}
        if radius*(offset-span) <= tolerance {endpoints.push(1.0);}
        return Ok(endpoints);
    }
    let count=((span-offset)/tau).floor()+1.0;
    let maximum=limits.max_outputs.min(limits.max_points).min(limits.max_bytes/8);
    if !count.is_finite() || count > maximum as f64 { return Err(Error::Budget("circle intersection parameters")); }
    let mut fractions=Vec::new();
    fractions.try_reserve_exact(count as usize).map_err(|_| Error::Budget("circle intersection parameters"))?;
    for index in 0..count as usize {
        let fraction=(offset+index as f64*tau)/span;
        if (0.0..=1.0).contains(&fraction) { fractions.push(fraction); }
    }
    // The final endpoint of a closed traversal is the same seam as its start.
    if fractions.first()==Some(&0.0) && fractions.last()==Some(&1.0) { fractions.pop(); }
    Ok(fractions)
}

fn line_circle(first:&Curve,second:&Curve,tolerance:f64,limits:&Limits) -> Result<Option<Vec<CurveIntersection>>> {
    let (line,circle,swapped)=match (first,second) {
        (Curve::Line{..},Curve::Circle{..})=>(first,second,false),
        (Curve::Circle{..},Curve::Line{..})=>(second,first,true),
        _=>return Ok(None),
    };
    let Curve::Line{start,end,..}=line else {unreachable!()};
    let Curve::Circle{center,x,y,radius,..}=circle else {unreachable!()};
    let direction=sub(*end,*start);let length=direction[0].hypot(direction[1]).hypot(direction[2]);
    if !length.is_finite() || length==0.0 {return Err(Error::Invalid("line circle direction"));}
    let unit=direction.map(|value|value/length);let delta=sub(*start,*center);finite(delta)?;
    let px=dot(delta,*x);let py=dot(delta,*y);let vx=dot(unit,*x);let vy=dot(unit,*y);
    let projected=vx.hypot(vy);let normal=cross(*x,*y);let z0=dot(delta,normal);let vz=dot(unit,normal);
    let mut line_fractions=Vec::new();
    if projected==0.0 {
        if vz==0.0 {return Err(Error::Invalid("line circle frame"));}
        line_fractions.push((-z0/vz/length).clamp(0.0,1.0));
    } else {
        let magnitude=px.abs().max(py.abs()).max(*radius);
        if !magnitude.is_finite() || magnitude==0.0 {return Err(Error::Invalid("line circle scale"));}
        let p=[px/magnitude,py/magnitude];let q=[vx/projected,vy/projected];let r=radius/magnitude;
        let center_distance=-(p[0]*q[0]+p[1]*q[1]);let perpendicular=(p[0]*q[1]-p[1]*q[0]).abs();
        let gap=r-perpendicular;
        if gap < 0.0 && -gap*magnitude>tolerance {return Ok(Some(Vec::new()));}
        let half=(gap.max(0.0)*(r+perpendicular)).sqrt();
        let mut roots=[center_distance-half,center_distance+half];
        // Avoid cancellation in the root nearest the line origin.
        if half>0.0 && center_distance.abs()>half {
            let product=p[0]*p[0]+p[1]*p[1]-r*r;
            if center_distance>0.0 && roots[1]!=0.0 {roots[0]=product/roots[1];}
            if center_distance<0.0 && roots[0]!=0.0 {roots[1]=product/roots[0];}
        }
        for root in roots {
            let fraction=(root/projected)*(magnitude/length);
            if !fraction.is_finite() {return Err(Error::Invalid("line circle parameter overflow"));}
            if (0.0..=1.0).contains(&fraction) {line_fractions.push(fraction);}
            else if fraction<0.0 && -fraction*length<=tolerance {line_fractions.push(0.0);}
            else if fraction>1.0 && (fraction-1.0)*length<=tolerance {line_fractions.push(1.0);}
        }
    }
    line_fractions.sort_by(f64::total_cmp);line_fractions.dedup_by(|a,b|*a==*b);
    let mut hits=Vec::new();
    for line_fraction in line_fractions {
        let line_point=point_at(line,line_fraction,limits)?;
        for circle_fraction in circle_fractions(circle,line_point,tolerance,limits)? {
            let circle_point=point_at(circle,circle_fraction,limits)?;let difference=sub(line_point,circle_point);
            let measured=difference[0].hypot(difference[1]).hypot(difference[2]);
            if measured<=tolerance {
                if hits.len() as u64 >= limits.max_outputs || hits.len() as u64 >= limits.max_points || (hits.len()+1) as u64 > limits.max_bytes/std::mem::size_of::<CurveIntersection>() as u64 {return Err(Error::Budget("line circle intersections"));}
                let (first_fraction,second_fraction)=if swapped {(circle_fraction,line_fraction)}else{(line_fraction,circle_fraction)};
                hits.push(CurveIntersection{point:add(scale(line_point,0.5),scale(circle_point,0.5)),first_fraction,second_fraction,distance:measured});
            }
        }
    }
    // A tilted line may approach the circle between projected crossings. Let
    // the general solver investigate those tolerance-only cases.
    if hits.is_empty() && vz!=0.0 && projected!=0.0 {return Ok(None);}
    Ok(Some(hits))
}

// A single rational quadratic Bezier span has an exact quadratic intersection
// equation with a line, including a double root at tangency.
fn line_quadratic_nurbs(first: &Curve, second: &Curve, tolerance: f64, limits: &Limits) -> Result<Option<Vec<CurveIntersection>>> {
    let (line, nurbs_curve, swapped) = match (first, second) {
        (Curve::Line { .. }, Curve::Nurbs(_)) => (first, second, false),
        (Curve::Nurbs(_), Curve::Line { .. }) => (second, first, true),
        _ => return Ok(None),
    };
    let Curve::Line { start, end, .. } = line else { unreachable!() };
    let Curve::Nurbs(n) = nurbs_curve else { unreachable!() };
    if n.degree != 2 || n.poles.len() != 3 || n.periodic || n.seam.is_some() || n.knots.len() != 6 ||
        n.knots[..3] != [n.domain[0]; 3] || n.knots[3..] != [n.domain[1]; 3] { return Ok(None); }
    let direction = sub(*end, *start);
    let length = direction[0].hypot(direction[1]).hypot(direction[2]);
    if !length.is_finite() || length == 0.0 { return Err(Error::Invalid("line NURBS direction")); }
    let unit = direction.map(|value| value / length);
    let values: [Point3; 3] = std::array::from_fn(|i| scale(cross(sub(n.poles[i], *start), unit), n.weights[i]));
    if values.iter().flatten().any(|value| !value.is_finite()) { return Err(Error::Invalid("line NURBS coefficients")); }
    let mut coefficients = [0.; 3]; let mut selected_values = [0.; 3]; let mut magnitude = 0.0_f64;
    for axis in 0..3 {
        let v = [values[0][axis], values[1][axis], values[2][axis]];
        let c = [v[0], 2.0 * (v[1] - v[0]), v[0] - 2.0 * v[1] + v[2]];
        let size = c[0].abs().max(c[1].abs()).max(c[2].abs());
        if !size.is_finite() { return Err(Error::Invalid("line NURBS coefficients")); }
        if size > magnitude { magnitude = size; coefficients = c; selected_values = v; }
    }
    if magnitude == 0.0 {
        let positions: [f64; 3] = std::array::from_fn(|i| dot(sub(n.poles[i], *start), unit));
        if positions.iter().any(|value| !value.is_finite()) { return Err(Error::Invalid("line NURBS position")); }
        if positions.iter().all(|value| *value < -tolerance) || positions.iter().all(|value| *value > length + tolerance) {
            return Ok(Some(Vec::new()));
        }
        return Err(Error::Unsupported("overlapping line and NURBS"));
    }
    // Positive rational weights keep the curve inside its control-point hull.
    // A whole span strictly on one side of the line's projection plane cannot
    // intersect it within tolerance, even if its polynomial has a root outside.
    let signed: [f64; 3] = std::array::from_fn(|i| selected_values[i] / n.weights[i]);
    if signed.iter().all(|v| *v > tolerance) || signed.iter().all(|v| *v < -tolerance) {
        return Ok(Some(Vec::new()));
    }
    let [c, b, a] = coefficients.map(|v| v / magnitude);
    let mut roots = Vec::new();
    if a == 0.0 {
        if b == 0.0 { return Ok(None); }
        roots.push(-c / b);
    } else {
        let four_ac = 4.0 * a * c;
        let discriminant = b.mul_add(b, -four_ac);
        let uncertainty = 8.0 * f64::EPSILON * (b*b + four_ac.abs());
        if discriminant == 0.0 {
            // A rounded zero is not proof of tangency. Reject it if coefficient
            // construction, normalization, or either product discarded bits.
            let two_sum = |x: f64, y: f64| { let s = x + y; let z = s - x; (s, (x - (s-z)) + (y-z)) };
            let (d, de) = two_sum(selected_values[1], -selected_values[0]);
            let (s, se) = two_sum(selected_values[0], -2.0 * selected_values[1]);
            let (_, ae) = two_sum(s, selected_values[2]);
            let normalized_exact = [c, b, a].iter().zip(coefficients).all(|(scaled, original)| scaled.mul_add(magnitude, -original) == 0.0);
            let product_error = b.mul_add(b, -(b*b)) - 4.0*a.mul_add(c, -(a*c));
            if de != 0.0 || se != 0.0 || ae != 0.0 || d * 2.0 != coefficients[1] || !normalized_exact || product_error != 0.0 {
                return Err(Error::Unsupported("quadratic intersection needs higher precision"));
            }
        }
        if discriminant != 0.0 && discriminant.abs() <= uncertainty {
            return Err(Error::Unsupported("quadratic intersection needs higher precision"));
        }
        if discriminant < 0.0 { return Ok(None); }
        if discriminant == 0.0 { roots.push(-b / (2.0*a)); }
        else {
            let q = -0.5 * (b + discriminant.sqrt().copysign(b));
            roots.push(q / a); roots.push(c / q);
        }
    }
    let mut hits = Vec::new();
    for root in roots {
        if !root.is_finite() || !(0.0..=1.0).contains(&root) { continue; }
        let fraction = if n.reversed { 1.0-root } else { root };
        let p = point_at(nurbs_curve, fraction, limits)?;
        let line_fraction = (dot(sub(p, *start), unit) / length).clamp(0.0, 1.0);
        let q = point_at(line, line_fraction, limits)?; let delta = sub(p, q);
        let distance = delta[0].hypot(delta[1]).hypot(delta[2]);
        if distance > tolerance { continue; }
        if hits.len() as u64 >= limits.max_outputs || hits.len() as u64 >= limits.max_points || (hits.len()+1) as u64 > limits.max_bytes / std::mem::size_of::<CurveIntersection>() as u64 { return Err(Error::Budget("line NURBS intersections")); }
        let (first_fraction, second_fraction) = if swapped { (fraction, line_fraction) } else { (line_fraction, fraction) };
        hits.push(CurveIntersection { point: add(scale(p, 0.5), scale(q, 0.5)), first_fraction, second_fraction, distance });
    }
    hits.sort_by(|a, b| a.first_fraction.total_cmp(&b.first_fraction));
    if hits.is_empty() { return Ok(None); }
    Ok(Some(hits))
}

fn line_multispan_quadratic(first: &Curve, second: &Curve, tolerance: f64, limits: &Limits) -> Result<Option<Vec<CurveIntersection>>> {
    let (line, curve, swapped) = match (first, second) {
        (Curve::Line { .. }, Curve::Nurbs(_)) => (first, second, false),
        (Curve::Nurbs(_), Curve::Line { .. }) => (second, first, true),
        _ => return Ok(None),
    };
    let Curve::Nurbs(n) = curve else { unreachable!() };
    if n.degree != 2 { return Ok(None); }
    if n.periodic && n.certify_repeated_periodic_basis(limits).is_err() { return Ok(None); }
    if let Some(seam) = n.seam {
        let mut base = n.clone(); base.seam = None;
        let base = Curve::Nurbs(base);
        if !crate::curve_ops::is_closed(&base, 1e-9, limits)? { return Err(Error::Invalid("NURBS seam needs a closed curve")); }
        let mut hits = match if swapped { line_multispan_quadratic(&base, line, tolerance, limits)? } else { line_multispan_quadratic(line, &base, tolerance, limits)? } {
            Some(hits) => hits,
            None => return Ok(None),
        };
        let offset = if n.reversed { (n.domain[1]-seam)/(n.domain[1]-n.domain[0]) } else { (seam-n.domain[0])/(n.domain[1]-n.domain[0]) };
        for hit in &mut hits {
            let fraction = if swapped { &mut hit.first_fraction } else { &mut hit.second_fraction };
            *fraction = (*fraction-offset).rem_euclid(1.0);
            let (curve_fraction,line_fraction) = if swapped { (hit.first_fraction,hit.second_fraction) } else { (hit.second_fraction,hit.first_fraction) };
            let p=point_at(curve,curve_fraction,limits)?;let q=point_at(line,line_fraction,limits)?;let delta=sub(p,q);
            hit.distance=delta[0].hypot(delta[1]).hypot(delta[2]);hit.point=add(scale(p,0.5),scale(q,0.5));
        }
        hits.retain(|hit| hit.distance<=tolerance);
        hits.sort_by(|a,b| a.first_fraction.total_cmp(&b.first_fraction).then(a.second_fraction.total_cmp(&b.second_fraction)));
        hits.dedup_by(|a,b| (a.first_fraction-b.first_fraction).abs()<=8.0*f64::EPSILON && (a.second_fraction-b.second_fraction).abs()<=8.0*f64::EPSILON && distance(a.point,b.point)<=tolerance);
        return Ok(Some(hits));
    }
    // For a trimmed periodic interval, extract whole basis spans first; cutting
    // a span at the active endpoint can perturb a tangent's double root.
    let basis_domain = [n.knots[n.degree], n.knots[n.poles.len()]];
    let expanded = if n.periodic && n.domain != basis_domain {
        let mut base = n.clone(); base.domain = basis_domain; Some(Curve::Nurbs(base))
    } else { None };
    let working_curve = expanded.as_ref().unwrap_or(curve);
    let Curve::Nurbs(working) = working_curve else { unreachable!() };
    let capacity = working.knots.len().checked_add(2).ok_or(Error::Overflow("quadratic spans"))?;
    if capacity as u64 > limits.max_points || capacity as u64 > limits.max_bytes / 8 { return Err(Error::Budget("quadratic spans")); }
    let mut boundaries = Vec::new();
    boundaries.try_reserve_exact(capacity).map_err(|_| Error::Budget("quadratic spans"))?;
    boundaries.extend([0.0, 1.0]);
    for knot in &working.knots {
        if *knot < working.domain[0] || *knot > working.domain[1] { continue; }
        let t = (*knot - working.domain[0]) / (working.domain[1] - working.domain[0]);
        let t = if working.reversed { 1.0 - t } else { t };
        boundaries.push(t);
    }
    boundaries.sort_by(f64::total_cmp); boundaries.dedup();
    if boundaries.len() <= 2 { return Ok(None); }
    let closed = crate::top11::curve_closed_for_cut(curve, limits)?;
    let mut hits: Vec<CurveIntersection> = Vec::new();
    let native = |f: f64| if working.reversed { working.domain[1]-f*(working.domain[1]-working.domain[0]) } else { working.domain[0]+f*(working.domain[1]-working.domain[0]) };
    for pair in boundaries.windows(2) {
        if expanded.is_some() && (native(pair[0]).max(native(pair[1])) < n.domain[0] || native(pair[0]).min(native(pair[1])) > n.domain[1]) { continue; }
        let span = match crate::curve_ops::trim_curve_by_parameter(working_curve, pair[0], pair[1], limits) {
            Ok(span) => span,
            Err(Error::Unsupported(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        let local = if swapped { line_quadratic_nurbs(&span, line, tolerance, limits)? } else { line_quadratic_nurbs(line, &span, tolerance, limits)? };
        let local = match local {
            Some(hits) => hits,
            None => if swapped { numeric_intersections(&span, line, tolerance, limits)? } else { numeric_intersections(line, &span, tolerance, limits)? },
        };
        for mut hit in local {
            let fraction = if swapped { &mut hit.first_fraction } else { &mut hit.second_fraction };
            *fraction = pair[0] + *fraction * (pair[1] - pair[0]);
            if expanded.is_some() {
                let parameter = native(*fraction);
                let roundoff = 8.0*f64::EPSILON*n.domain[0].abs().max(n.domain[1].abs()).max(1.0);
                if parameter < n.domain[0]-roundoff || parameter > n.domain[1]+roundoff { continue; }
                let t = (parameter.clamp(n.domain[0],n.domain[1])-n.domain[0])/(n.domain[1]-n.domain[0]);
                *fraction = if n.reversed {1.0-t} else {t};
            }
            if closed && *fraction == 1.0 { *fraction = 0.0; }
            let (curve_fraction, line_fraction) = if swapped { (hit.first_fraction, hit.second_fraction) } else { (hit.second_fraction, hit.first_fraction) };
            let curve_point = point_at(curve, curve_fraction, limits)?; let line_point = point_at(line, line_fraction, limits)?;
            let delta = sub(curve_point, line_point); hit.distance = delta[0].hypot(delta[1]).hypot(delta[2]);
            if hit.distance > tolerance { continue; }
            hit.point = add(scale(curve_point, 0.5), scale(line_point, 0.5));
            if hits.iter().any(|previous| (previous.first_fraction-hit.first_fraction).abs() <= 8.0*f64::EPSILON && (previous.second_fraction-hit.second_fraction).abs() <= 8.0*f64::EPSILON && distance(previous.point,hit.point) <= tolerance) { continue; }
            if hits.len() as u64 >= limits.max_outputs || hits.len() as u64 >= limits.max_points || (hits.len()+1) as u64 > limits.max_bytes/std::mem::size_of::<CurveIntersection>() as u64 { return Err(Error::Budget("quadratic span intersections")); }
            hits.try_reserve(1).map_err(|_| Error::Budget("quadratic span intersections"))?;
            hits.push(hit);
        }
    }
    hits.sort_by(|a,b| a.first_fraction.total_cmp(&b.first_fraction).then(a.second_fraction.total_cmp(&b.second_fraction)));
    Ok(Some(hits))
}

fn line_line(first: &Curve, second: &Curve, tolerance: f64) -> Result<Option<Vec<CurveIntersection>>> {
    let (
        Curve::Line {
            start: p1, end: e1, ..
        },
        Curve::Line {
            start: p2, end: e2, ..
        },
    ) = (first, second)
    else {
        return Ok(None);
    };
    let d1 = sub(*e1, *p1);
    let d2 = sub(*e2, *p2);
    let length1=d1[0].hypot(d1[1]).hypot(d1[2]);
    let length2=d2[0].hypot(d2[1]).hypot(d2[2]);
    if !length1.is_finite() || !length2.is_finite() || length1 == 0.0 || length2 == 0.0 {
        return Err(Error::Invalid("line intersection direction"));
    }
    let u=d1.map(|value|value/length1);let v=d2.map(|value|value/length2);
    let normal=cross(u,v);let denominator=dot(normal,normal);
    let delta=sub(*p2,*p1);finite(delta)?;
    if denominator == 0.0 {
        let offset = cross(delta, u);
        if offset[0].hypot(offset[1]).hypot(offset[2]) > tolerance { return Ok(Some(Vec::new())); }
        let start = dot(delta, u);
        let direction = dot(v, u) * length2;
        let end = start + direction;
        if !start.is_finite() || !end.is_finite() || !direction.is_finite() || direction == 0.0 {
            return Err(Error::Invalid("parallel line intersection overflow"));
        }
        let low = 0.0_f64.max(start.min(end));
        let high = length1.min(start.max(end));
        if low - high > tolerance { return Ok(Some(Vec::new())); }
        if high - low > tolerance { return Err(Error::Unsupported("overlapping line segments")); }
        let coordinate = low * 0.5 + high * 0.5;
        let s = (coordinate / length1).clamp(0.0, 1.0);
        let t = ((coordinate - start) / direction).clamp(0.0, 1.0);
        let left = add(*p1, scale(d1, s)); let right = add(*p2, scale(d2, t));
        let difference = sub(left, right);
        let measured = difference[0].hypot(difference[1]).hypot(difference[2]);
        if measured > tolerance { return Ok(Some(Vec::new())); }
        return Ok(Some(vec![CurveIntersection { point: add(scale(left, 0.5), scale(right, 0.5)), first_fraction: s, second_fraction: t, distance: measured }]));
    }
    let mut s=dot(cross(delta,v),normal)/denominator/length1;
    let mut t=dot(cross(delta,u),normal)/denominator/length2;
    if !s.is_finite() || !t.is_finite() { return Err(Error::Invalid("line intersection overflow")); }
    if !(0.0..=1.0).contains(&s) || !(0.0..=1.0).contains(&t) {
        // The infinite-line minimum lies outside at least one segment. The
        // constrained minimum is an endpoint projected onto the other segment.
        let projections = [
            (0., dot(sub(*p1,*p2),v)/length2),
            (1., dot(sub(*e1,*p2),v)/length2),
            (dot(sub(*p2,*p1),u)/length1, 0.),
            (dot(sub(*e2,*p1),u)/length1, 1.),
        ];
        if projections.iter().any(|(a,b)| !a.is_finite() || !b.is_finite()) {
            return Err(Error::Invalid("line intersection overflow"));
        }
        let mut best = f64::INFINITY;
        for (a,b) in projections {
            let a=a.clamp(0.,1.);let b=b.clamp(0.,1.);
            let d=sub(add(*p1,scale(d1,a)),add(*p2,scale(d2,b)));
            let distance=d[0].hypot(d[1]).hypot(d[2]);
            if distance < best {best=distance;s=a;t=b;}
        }
    }
    let left = add(*p1, scale(d1, s));
    let right = add(*p2, scale(d2, t));
    let difference=sub(left,right);
    let measured = difference[0].hypot(difference[1]).hypot(difference[2]);
    if measured > tolerance {
        return Ok(Some(Vec::new()));
    }
    Ok(Some(vec![CurveIntersection {
        point: add(scale(left, 0.5),scale(right, 0.5)),
        first_fraction: clamp(s, 0.0, 1.0),
        second_fraction: clamp(t, 0.0, 1.0),
        distance: measured,
    }]))
}

fn coplanar_circles(
    first: &Curve,
    second: &Curve,
    tolerance: f64,
    limits: &Limits,
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
    if norm(cross(n1, n2)) > 8.0*f64::EPSILON || dot(sub(*c2, *c1), n1).abs() > tolerance {
        return Ok(None);
    }
    let delta = sub(*c2, *c1);
    let centers = sub(delta,scale(n1,dot(delta,n1)));
    let d = centers[0].hypot(centers[1]).hypot(centers[2]);
    if !d.is_finite() {return Err(Error::Invalid("circle intersection center distance"));}
    if d == 0.0 {
        return Ok(Some(Vec::new()));
    }
    // Scale before squaring so valid small or large circles do not underflow or overflow.
    let magnitude=d.max(*r1).max(*r2);
    let ds=d/magnitude;let r1s=r1/magnitude;let r2s=r2/magnitude;
    let tolerance_scaled=tolerance/magnitude;
    if ds > r1s+r2s+tolerance_scaled || ds < (r1s-r2s).abs()-tolerance_scaled {return Ok(Some(Vec::new()));}
    if ds == 0.0 {return Err(Error::Invalid("circle intersection scale"));}
    let a_scaled=(r1s*r1s-r2s*r2s+ds*ds)/(2.0*ds);
    let h_squared=r1s*r1s-a_scaled*a_scaled;
    let a=a_scaled*magnitude;
    let h = if h_squared > 0.0 {
        h_squared.sqrt()*magnitude
    } else {
        0.0
    };
    if !a.is_finite() || !h.is_finite() {return Err(Error::Invalid("circle intersection overflow"));}
    let unit = centers.map(|value|value/d);
    let base = add(*c1, scale(unit, a));
    let perpendicular = cross(n1, unit);
    let perpendicular = if norm(perpendicular) > 1e-12 {
        scale(perpendicular, h / norm(perpendicular))
    } else {
        [0.0; 3]
    };
    let candidates = if h == 0.0 {
        vec![base]
    } else {
        vec![add(base, perpendicular), sub(base, perpendicular)]
    };
    let mut result = Vec::new();
    for point in candidates {
        let first_fractions=circle_fractions(first,point,tolerance,limits)?;
        let second_fractions=circle_fractions(second,point,tolerance,limits)?;
        let combinations=first_fractions.len().checked_mul(second_fractions.len()).ok_or(Error::Overflow("circle intersection combinations"))?;
        let capacity=result.len().checked_add(combinations).ok_or(Error::Overflow("circle intersection combinations"))?;
        let maximum=limits.max_outputs.min(limits.max_points).min(limits.max_bytes/std::mem::size_of::<CurveIntersection>() as u64);
        if capacity as u64 > maximum {return Err(Error::Budget("circle intersections"));}
        result.try_reserve(combinations).map_err(|_| Error::Budget("circle intersections"))?;
        for first_fraction in first_fractions {
            for &second_fraction in &second_fractions {
                let a=point_at(first,first_fraction,limits)?;let b=point_at(second,second_fraction,limits)?;
                let difference=sub(a,b);let measured=difference[0].hypot(difference[1]).hypot(difference[2]);
                let da=sub(a,point);let db=sub(b,point);
                let a_error=da[0].hypot(da[1]).hypot(da[2]);let b_error=db[0].hypot(db[1]).hypot(db[2]);
                if measured <= tolerance && a_error <= tolerance && b_error <= tolerance {
                    if result.len() as u64 >= limits.max_outputs {return Err(Error::Budget("circle intersections"));}
                    result.push(CurveIntersection {point:add(scale(a,0.5),scale(b,0.5)),first_fraction,second_fraction,distance:measured});
                }
            }
        }
    }
    Ok(Some(result))
}

fn intersection_samples(curve: &Curve, limits: &Limits) -> Result<Vec<(f64, Point3)>> {
    let extra = match curve { Curve::Nurbs(n) => n.knots.len().checked_mul(2).and_then(|n| n.checked_add(3)).ok_or(Error::Overflow("intersection samples"))?, _ => 0 };
    let capacity = 257usize.checked_add(extra).ok_or(Error::Overflow("intersection samples"))?;
    if capacity as u64 > limits.max_points || capacity as u64 > limits.max_bytes / std::mem::size_of::<(f64, Point3)>() as u64 {
        return Err(Error::Budget("intersection samples"));
    }
    let mut fractions = Vec::new();
    fractions.try_reserve_exact(capacity).map_err(|_| Error::Budget("intersection samples"))?;
    fractions.extend((0..=256).map(|index| index as f64 / 256.0));
    if let Curve::Nurbs(n) = curve {
        let [start, end] = n.domain;
        let offset = n.seam.map(|seam| if n.reversed { (end-seam)/(end-start) } else { (seam-start)/(end-start) }).unwrap_or(0.0);
        let mut knots = Vec::new();
        knots.try_reserve_exact(n.knots.len().checked_add(2).ok_or(Error::Overflow("intersection knots"))?).map_err(|_| Error::Budget("intersection knots"))?;
        for &knot in &n.knots {
            if knot < start || knot > end { continue; }
            let native = (knot-start)/(end-start);
            let fraction = if n.reversed { 1.0-native } else { native };
            knots.push(if n.seam.is_some() { (fraction-offset).rem_euclid(1.0) } else { fraction });
        }
        knots.push(0.0); knots.push(1.0);
        knots.sort_by(f64::total_cmp); knots.dedup();
        fractions.extend(knots.iter().copied());
        fractions.extend(knots.windows(2).map(|pair| pair[0] + (pair[1]-pair[0])*0.5));
    }
    fractions.sort_by(f64::total_cmp); fractions.dedup();
    let mut samples = Vec::new();
    samples.try_reserve_exact(fractions.len()).map_err(|_| Error::Budget("intersection samples"))?;
    for fraction in fractions { samples.push((fraction, point_at(curve, fraction, limits)?)); }
    Ok(samples)
}

fn numeric_intersections(
    first: &Curve,
    second: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<CurveIntersection>> {
    let capacity = |curve: &Curve| -> Result<usize> {
        let knots = match curve { Curve::Nurbs(n) => n.knots.len().checked_mul(2).and_then(|n| n.checked_add(3)).ok_or(Error::Overflow("intersection samples"))?, _ => 0 };
        257usize.checked_add(knots).ok_or(Error::Overflow("intersection samples"))
    };
    let count = capacity(first)?.checked_add(capacity(second)?).ok_or(Error::Overflow("intersection samples"))?;
    // Covers both retained sample arrays, the sample-building scratch and candidate pairs.
    let scratch_bytes = (count as u64).saturating_mul(64);
    if scratch_bytes > limits.max_bytes { return Err(Error::Budget("intersection scratch")); }
    let first_samples = intersection_samples(first, limits)?;
    let second_samples = intersection_samples(second, limits)?;
    let mut candidates: Vec<(f64, f64)> = Vec::new();
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    for (_, point) in first_samples.iter().chain(&second_samples) {
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
    let threshold = (span_scale / 256.0 * 3.0)
        .max(tolerance * 8.0)
        .max(1e-6);
    let capacity = first_samples.len().checked_add(second_samples.len()).ok_or(Error::Overflow("intersection candidates"))?;
    if (capacity as u64).saturating_mul(std::mem::size_of::<(f64,f64)>() as u64) > limits.max_bytes { return Err(Error::Budget("intersection candidates")); }
    candidates.try_reserve_exact(capacity).map_err(|_| Error::Budget("intersection candidates"))?;
    for &(fraction, point) in &first_samples {
        let mut best_other = 0.0;
        let mut best_distance = f64::INFINITY;
        for &(other_fraction, other_point) in &second_samples {
            let measured = distance(point, other_point);
            if measured < best_distance {
                best_distance = measured;
                best_other = other_fraction;
            }
        }
        if best_distance <= threshold {
            candidates.push((fraction, best_other));
        }
    }
    for &(fraction, point) in &second_samples {
        let mut best_other = 0.0; let mut best_distance = f64::INFINITY;
        for &(other_fraction, other_point) in &first_samples {
            let measured = distance(point, other_point);
            if measured < best_distance { best_distance = measured; best_other = other_fraction; }
        }
        if best_distance <= threshold { candidates.push((best_other, fraction)); }
    }
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    candidates.dedup();
    let mut refined: Vec<CurveIntersection> = Vec::new();
    for (mut f, mut g) in candidates {
        let mut best = distance(point_at(first, f, limits)?, point_at(second, g, limits)?);
        let mut factor = 1.0;
        for _ in 0..40 {
            let a = match fraction_derivatives(first, f, 1, limits) {
                Ok(jet) => jet,
                Err(Error::Unsupported("derivative at insufficient knot continuity")) => break,
                Err(error) => return Err(error),
            };
            let b = match fraction_derivatives(second, g, 1, limits) {
                Ok(jet) => jet,
                Err(Error::Unsupported("derivative at insufficient knot continuity")) => break,
                Err(error) => return Err(error),
            };
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
            let step_f = -(bb * g1 + ab * g2) / determinant;
            let step_g = -(ab * g1 + aa * g2) / determinant;
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
            .any(|existing| distance(existing.point, point) <= tolerance &&
                (existing.first_fraction-f).abs() <= 1e-7 && (existing.second_fraction-g).abs() <= 1e-7)
        {
            continue;
        }
        if refined.len() as u64 >= limits.max_outputs || refined.len() as u64 >= limits.max_points ||
            (refined.len() + 1) as u64 > (limits.max_bytes - scratch_bytes) / std::mem::size_of::<CurveIntersection>() as u64 {
            return Err(Error::Budget("curve intersections"));
        }
        refined.try_reserve(1).map_err(|_| Error::Budget("curve intersections"))?;
        refined.push(CurveIntersection {
            point,
            first_fraction: f,
            second_fraction: g,
            distance: best,
        });
    }
    Ok(refined)
}

fn polycurve_intersections(
    segments: &[Curve], reversed: bool, other: &Curve, swapped: bool,
    tolerance: f64, limits: &Limits,
) -> Result<Vec<CurveIntersection>> {
    let mut hits: Vec<CurveIntersection> = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        let local = if swapped {
            curve_curve_intersections(other, segment, tolerance, limits)?
        } else {
            curve_curve_intersections(segment, other, tolerance, limits)?
        };
        for mut hit in local {
            let fraction = if swapped { &mut hit.second_fraction } else { &mut hit.first_fraction };
            let forward = (index as f64 + *fraction) / segments.len() as f64;
            *fraction = if reversed { 1.0 - forward } else { forward };
            // Adjacent segments may both report the same joint; retain repeated
            // crossings at different parameters, even at the same position.
            if hits.iter().any(|previous|
                (previous.first_fraction - hit.first_fraction).abs() <= 8.0 * f64::EPSILON &&
                (previous.second_fraction - hit.second_fraction).abs() <= 8.0 * f64::EPSILON &&
                distance(previous.point, hit.point) <= tolerance
            ) { continue; }
            if hits.len() as u64 >= limits.max_outputs || hits.len() as u64 >= limits.max_points ||
                (hits.len() + 1) as u64 > limits.max_bytes / std::mem::size_of::<CurveIntersection>() as u64 {
                return Err(Error::Budget("polycurve intersections"));
            }
            hits.try_reserve(1).map_err(|_| Error::Budget("polycurve intersections"))?;
            hits.push(hit);
        }
    }
    hits.sort_by(|a, b| a.first_fraction.total_cmp(&b.first_fraction).then(a.second_fraction.total_cmp(&b.second_fraction)));
    Ok(hits)
}

/// Intersections of two curves. Lines, line/circle pairs and coplanar circles use closed forms; every
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
    if let Curve::Polycurve { segments, reversed } = first {
        return polycurve_intersections(segments, *reversed, second, false, tolerance, limits);
    }
    if let Curve::Polycurve { segments, reversed } = second {
        return polycurve_intersections(segments, *reversed, first, true, tolerance, limits);
    }
    if let Some(exact) = line_line(first, second, tolerance)? { return Ok(exact); }
    if let Some(exact) = line_circle(first, second, tolerance, limits)? { return Ok(exact); }
    if let Some(exact) = line_quadratic_nurbs(first, second, tolerance, limits)? { return Ok(exact); }
    if let Some(exact) = line_multispan_quadratic(first, second, tolerance, limits)? { return Ok(exact); }
    if let Some(circles) = coplanar_circles(first, second, tolerance, limits)? {
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
    if distances.iter().all(|distance| *distance <= tolerance) {
        return Err(Error::Unsupported("curve may overlap surface"));
    }
    for index in 1..=samples {
        if distances[index - 1] <= tolerance && distances[index] <= tolerance {
            let a = point_at(curve, (index - 1) as f64 / samples as f64, limits)?;
            let b = point_at(curve, index as f64 / samples as f64, limits)?;
            if distance(a, b) > 2.0 * tolerance {
                return Err(Error::Unsupported("curve may overlap surface"));
            }
        }
    }
    let mut result = Vec::new();
    for (index, neighbor) in [(0, 1), (samples, samples - 1)] {
        if distances[index] <= tolerance && distances[neighbor] > distances[index] + 1e-14 {
            let fraction = index as f64 / samples as f64;
            result.push(SurfaceIntersection {
                point: scale(add(point_at(curve, fraction, limits)?, projections[index].point), 0.5),
                curve_fraction: fraction,
                parameters: projections[index].parameters,
                distance: distances[index],
            });
        }
    }
    for index in 1..samples {
        if distances[index] > distances[index - 1] || distances[index] > distances[index + 1] {
            continue;
        }
        if distances[index] == distances[index - 1] && distances[index] == distances[index + 1] {
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
        if best > tolerance {
            continue;
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
