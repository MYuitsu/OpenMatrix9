// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact world-space corners of an ordered, piecewise-linear source for From Lines.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Point3, Transform},
};

/// Every active knot of a degree-one NURBS is a potential corner. In particular,
/// trimmed rational endpoints need evaluation and may lie between the stored poles.
pub fn placed_vertices(
    curve: &Curve,
    world: Transform,
    tolerance: f64,
    limits: &Limits,
    check: &mut impl FnMut() -> Result<()>,
) -> Result<Vec<Point3>> {
    if !tolerance.is_finite() || tolerance <= 0. {
        return Err(Error::Invalid("linear path tolerance"));
    }
    curve.validate(limits)?;
    world.validate()?;
    let mut points = Vec::new();
    append(curve, false, world, tolerance, limits, check, &mut points)?;
    if points.len() < 2 {
        return Err(Error::Invalid("linear path has no edge"));
    }
    Ok(points)
}

fn append(
    curve: &Curve,
    backwards: bool,
    world: Transform,
    tolerance: f64,
    limits: &Limits,
    check: &mut impl FnMut() -> Result<()>,
    points: &mut Vec<Point3>,
) -> Result<()> {
    check()?;
    if let Curve::Polycurve { segments, reversed } = curve {
        let reverse = backwards ^ reversed;
        for i in 0..segments.len() {
            let segment = if reverse { segments.len() - 1 - i } else { i };
            append(
                &segments[segment],
                reverse,
                world,
                tolerance,
                limits,
                check,
                points,
            )?;
        }
        return Ok(());
    }
    let existing = points.len();
    let mut push = |p: Point3, first: bool| -> Result<()> {
        check()?;
        if first && let Some(q) = points.last() {
            let d = (p[0] - q[0]).hypot(p[1] - q[1]).hypot(p[2] - q[2]);
            if d > tolerance {
                return Err(Error::Invalid("disconnected linear polycurve"));
            }
            return Ok(());
        }
        let len = points.len() + 1;
        if len as u64 > limits.max_points {
            return Err(Error::Budget("linear path points"));
        }
        Usage::array::<Point3>(len)?.check(limits)?;
        points.push(p);
        Ok(())
    };
    match curve {
        Curve::Line { start, end, .. } => {
            let (a, b) = if backwards {
                (end, start)
            } else {
                (start, end)
            };
            push(world.point(*a)?, true)?;
            push(world.point(*b)?, false)
        }
        Curve::Nurbs(n) if n.degree == 1 => {
            let [lo, hi] = n.domain;
            let interior = &n.knots
                [n.knots.partition_point(|&k| k <= lo)..n.knots.partition_point(|&k| k < hi)];
            if interior.windows(2).any(|w| w[0] == w[1]) {
                return Err(Error::Unsupported("discontinuous degree-one NURBS"));
            }
            let count = interior.len().saturating_add(3);
            let mut usage = Usage::array::<f64>(count)?;
            usage.add(
                Usage::array::<Point3>(existing.saturating_add(count))?,
                limits,
            )?;
            let mut parameters = Vec::with_capacity(count);
            let eval = CurveEvaluator::capture(curve, limits)?;
            if let Some(seam) = n.seam.filter(|&s| s > lo && s < hi) {
                let a = eval.evaluate(Parameter::Native(lo), 0, world)?[0];
                let b = eval.evaluate(Parameter::Native(hi), 0, world)?[0];
                if (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]) > tolerance {
                    return Err(Error::Invalid("moved seam requires a closed linear path"));
                }
                parameters.push(seam);
                parameters.extend_from_slice(&interior[interior.partition_point(|&k| k <= seam)..]);
                parameters.push(hi);
                parameters.extend_from_slice(&interior[..interior.partition_point(|&k| k < seam)]);
                parameters.push(seam);
            } else {
                parameters.push(lo);
                parameters.extend_from_slice(interior);
                parameters.push(hi);
            }
            if n.reversed ^ backwards {
                parameters.reverse();
            }
            for (i, u) in parameters.into_iter().enumerate() {
                push(eval.evaluate(Parameter::Native(u), 0, world)?[0], i == 0)?;
            }
            Ok(())
        }
        _ => Err(Error::Unsupported(
            "From Lines requires lines or degree-one polylines",
        )),
    }
}
