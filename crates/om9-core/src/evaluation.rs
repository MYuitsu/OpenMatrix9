// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned curve derivatives. Taylor/de Boor arithmetic ported from OM9's
//! `rust/src/support_snapshot.rs`; expanded knots and explicit D0-D3 contracts
//! replace its compact native basis and host-bound source admission.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Curve, NurbsCurve, Point3, Transform, finite},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Parameter {
    Native(f64),
    Fraction(f64),
}
/// Admission proof bound to an immutably borrowed curve. A sampling loop can
/// reuse the proof without traversing all poles/knots for every parameter.
/// The original curve cannot be changed while this evaluator is borrowed.
pub struct CurveEvaluator<'a> {
    curve: &'a Curve,
    limits: Limits,
}
impl<'a> CurveEvaluator<'a> {
    pub fn capture(curve: &'a Curve, limits: &Limits) -> Result<Self> {
        curve.validate(limits)?;
        domain(curve)?;
        Ok(Self {
            curve,
            limits: *limits,
        })
    }
    pub fn evaluate(
        &self,
        parameter: Parameter,
        order: usize,
        world: Transform,
    ) -> Result<Vec<Point3>> {
        evaluate_admitted(self.curve, parameter, order, world, &self.limits)
    }
}
fn domain(c: &Curve) -> Result<([f64; 2], bool)> {
    match c {
        Curve::Line { domain, .. } | Curve::Circle { domain, .. } => Ok((*domain, false)),
        Curve::Nurbs(n) => Ok((n.domain, n.reversed)),
        Curve::Polycurve { .. } => Err(Error::Unsupported(
            "polycurve requires an explicit segment parameter",
        )),
    }
}
pub fn evaluate_curve(
    c: &Curve,
    parameter: Parameter,
    order: usize,
    world: Transform,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    CurveEvaluator::capture(c, limits)?.evaluate(parameter, order, world)
}
fn evaluate_admitted(
    c: &Curve,
    parameter: Parameter,
    order: usize,
    world: Transform,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    if order > 3 {
        return Err(Error::Unsupported("derivative order above D3"));
    }
    world.validate()?;
    let (domain, reversed) = domain(c)?;
    let (u, mut scale) = match parameter {
        Parameter::Native(u) => (u, 1.),
        Parameter::Fraction(f) => {
            if !f.is_finite() || !(0. ..=1.).contains(&f) {
                return Err(Error::Invalid("curve fraction"));
            }
            let span = domain[1] - domain[0];
            let scale = if reversed { -span } else { span };
            (
                if reversed {
                    domain[1] + f * scale
                } else {
                    domain[0] + f * scale
                },
                scale,
            )
        }
    };
    if !u.is_finite() || u < domain[0] || u > domain[1] || !scale.is_finite() {
        return Err(Error::Invalid("curve parameter"));
    }
    let mut derivatives = match c {
        Curve::Line { start, end, .. } => {
            let mut d = vec![[0.; 3]; order + 1];
            let t = (u - domain[0]) / (domain[1] - domain[0]);
            d[0] = std::array::from_fn(|i| start[i] * (1. - t) + end[i] * t);
            if order > 0 {
                d[1] = std::array::from_fn(|i| (end[i] - start[i]) / (domain[1] - domain[0]));
            }
            d
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            angle_domain,
            ..
        } => {
            let (angle, rate) = if let Some(a) = angle_domain {
                let angular_span = a[1] - a[0];
                // Compose the requested parameter map before differentiation.
                // Fraction -> rounded native u -> angle loses phase in narrow
                // domains; separate powers of the native maps can overflow
                // even when the composed derivatives are representable.
                let (fraction, rate) = match parameter {
                    Parameter::Fraction(f) => (f, angular_span),
                    Parameter::Native(_) => (
                        (u - domain[0]) / (domain[1] - domain[0]),
                        angular_span / (domain[1] - domain[0]),
                    ),
                };
                (
                    if fraction == 1. {
                        a[1]
                    } else {
                        a[0] + fraction * angular_span
                    },
                    rate,
                )
            } else {
                (u, scale)
            };
            scale = 1.;
            let (sine, cosine) = angle.sin_cos();
            let mut d = vec![[0.; 3]; order + 1];
            let mut angular_factor = *radius;
            for (k, value) in d.iter_mut().enumerate() {
                if k > 0 {
                    angular_factor *= rate;
                }
                let (a, b) = match k {
                    0 => (cosine, sine),
                    1 => (-sine, cosine),
                    2 => (-cosine, -sine),
                    _ => (sine, -cosine),
                };
                *value = std::array::from_fn(|i| {
                    (a * x[i] + b * y[i]) * angular_factor + if k == 0 { center[i] } else { 0. }
                });
            }
            d
        }
        Curve::Nurbs(n) => nurbs_derivatives(n, u, order, limits)?,
        Curve::Polycurve { .. } => return Err(Error::Unsupported("polycurve segment evaluation")),
    };
    let mut factor = 1.;
    for (k, d) in derivatives.iter_mut().enumerate() {
        if k > 0 {
            factor *= scale;
            *d = d.map(|v| v * factor);
        }
        finite(*d)?;
        if k == 0 {
            *d = world.point(*d)?;
        } else {
            *d = std::array::from_fn(|i| (0..3).map(|j| world.0[i][j] * d[j]).sum());
            finite(*d)?;
        }
    }
    Ok(derivatives)
}
fn nurbs_derivatives(n: &NurbsCurve, u: f64, order: usize, limits: &Limits) -> Result<Vec<Point3>> {
    nurbs_derivatives_side(n,u,order,limits,None)
}
/// Exact one-sided jet at a knot. The caller must compare both sides before
/// claiming a derivative-independent geometric quantity there.
pub fn nurbs_one_sided_derivatives(n:&NurbsCurve,u:f64,order:usize,from_left:bool,limits:&Limits)->Result<Vec<Point3>>{
    n.validate(limits)?;
    if order>3||!u.is_finite()||u<n.domain[0]||u>n.domain[1]{return Err(Error::Invalid("one-sided derivative parameter/order"));}
    nurbs_derivatives_side(n,u,order,limits,Some(from_left))
}
fn nurbs_derivatives_side(n:&NurbsCurve,u:f64,order:usize,limits:&Limits,side:Option<bool>)->Result<Vec<Point3>>{
    if n.periodic {
        n.certify_repeated_periodic(limits)?;
        // Knot multiplicity limits continuity at the periodic seam.
        if side.is_none() && (u==n.domain[0] || u==n.domain[1]) {
            let seam=n.domain[0];
            let mult=n.knots.partition_point(|k|*k<=seam)-n.knots.partition_point(|k|*k<seam);
            if order>n.degree.saturating_sub(mult) {return Err(Error::Unsupported("periodic seam derivative continuity"));}
        }
    }
    let degree = n.degree;
    let count = n.poles.len();
    if side.is_none() && u > n.domain[0] && u < n.domain[1] {
        let multiplicity =
            n.knots.partition_point(|k| *k <= u) - n.knots.partition_point(|k| *k < u);
        if multiplicity > 0 && order > degree.saturating_sub(multiplicity) {
            return Err(Error::Unsupported(
                "derivative at insufficient knot continuity",
            ));
        }
    }
    let span = if u == n.knots[count] {
        // The active span at the right boundary is the last nonempty span
        // to its left, even when that boundary knot has multiplicity > 1.
        n.knots.partition_point(|k| *k < u).checked_sub(1)
            .ok_or(Error::Invalid("knot span"))?.min(count-1)
    } else {
        n.knots
            .partition_point(|k| if side==Some(true) && u>n.domain[0] {*k<u} else {*k<=u})
            .checked_sub(1)
            .ok_or(Error::Invalid("knot span"))?
            .min(count - 1)
    };
    let first = span
        .checked_sub(degree)
        .ok_or(Error::Invalid("active knot span"))?;
    let temporary = Usage::array::<[[f64; 4]; 4]>(degree + 1)?;
    temporary.check(limits)?;
    let weight_scale = n.weights[first..=span].iter().copied().fold(0., f64::max);
    let mut values = vec![[[0.; 4]; 4]; degree + 1];
    for (j, coefficients) in values.iter_mut().enumerate() {
        let i = first + j;
        let w = n.weights[i] / weight_scale;
        let p = n.poles[i];
        coefficients[0] = [p[0] * w, p[1] * w, p[2] * w, w];
    }
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let left = n.knots[span - degree + j];
            let right = n.knots[span + 1 + j - level];
            let width = right - left;
            if !width.is_finite() || width <= 0. {
                return Err(Error::Invalid("de Boor interval"));
            }
            let alpha = (u - left) / width;
            for k in (0..=order).rev() {
                let a = values[j - 1][k];
                let b = values[j][k];
                let derivative = if k == 0 {
                    [0.; 4]
                } else {
                    std::array::from_fn(|c| (values[j][k - 1][c] - values[j - 1][k - 1][c]) / width)
                };
                values[j][k] =
                    std::array::from_fn(|c| (1. - alpha) * a[c] + alpha * b[c] + derivative[c]);
            }
        }
    }
    let homogeneous = &values[degree];
    let w = homogeneous[0][3];
    if !w.is_finite() || w <= 0. {
        return Err(Error::Invalid("homogeneous denominator"));
    }
    let mut coefficients = vec![[0.; 3]; order + 1];
    for k in 0..=order {
        coefficients[k] = std::array::from_fn(|c| {
            (homogeneous[k][c]
                - (1..=k)
                    .map(|j| homogeneous[j][3] * coefficients[k - j][c])
                    .sum::<f64>())
                / w
        });
        finite(coefficients[k])?;
    }
    // Taylor coefficients are Dk/k!, including nonzero rational derivatives
    // above the homogeneous polynomial's degree.
    let factorial = [1., 1., 2., 6.];
    for (k, d) in coefficients.iter_mut().enumerate() {
        *d = d.map(|v| v * factorial[k]);
        finite(*d)?;
    }
    Ok(coefficients)
}
