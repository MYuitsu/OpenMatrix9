// SPDX-License-Identifier: LGPL-2.1-or-later
//! Read-only differential and integral measurements in canonical millimetres.
use crate::{
    Error, Result,
    budget::Limits,
    curve_ops,
    geometry::{Curve, NurbsSurface, Point3},
    surface_kernel,
};
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: Point3) -> f64 {
    a.iter().map(|v| v * v).sum::<f64>().sqrt()
}
/// Osculating radius at an explicit normalized station. A line/inflection has no finite radius.
pub fn curvature_radius(curve: &Curve, fraction: f64, limits: &Limits) -> Result<f64> {
    if matches!(curve, Curve::Polycurve { .. }) {
        return Err(Error::Unsupported(
            "curvature requires one smooth curve; select an individual polycurve segment",
        ));
    }
    if !(0.0..=1.0).contains(&fraction) {
        return Err(Error::Invalid("curvature station in [0,1]"));
    }
    let d = match curve_ops::fraction_derivatives(curve, fraction, 2, limits) {
        Ok(d) => d,
        Err(Error::Unsupported("derivative at insufficient knot continuity")) => {
            let Curve::Nurbs(n) = curve else {
                return Err(Error::Unsupported("curvature at curve discontinuity"));
            };
            let f = if n.reversed { 1. - fraction } else { fraction };
            let u = n.domain[0] + (n.domain[1] - n.domain[0]) * f;
            let a = crate::evaluation::nurbs_one_sided_derivatives(n, u, 2, true, limits)?;
            let b = crate::evaluation::nurbs_one_sided_derivatives(n, u, 2, false, limits)?;
            let geometric = |d: &[Point3]| -> Result<(Point3, Point3)> {
                let v = norm(d[1]);
                if v < 1e-12 {
                    return Err(Error::Invalid("zero-speed curvature knot"));
                }
                let dot = d[1].iter().zip(d[2]).map(|(a, b)| a * b).sum::<f64>() / (v * v);
                Ok((
                    d[1].map(|a| a / v),
                    std::array::from_fn(|i| (d[2][i] - d[1][i] * dot) / (v * v)),
                ))
            };
            let (ta, ka) = geometric(&a)?;
            let (tb, kb) = geometric(&b)?;
            if norm(std::array::from_fn(|i| ta[i] - tb[i])) > 1e-8
                || norm(std::array::from_fn(|i| ka[i] - kb[i]))
                    > 1e-8 * norm(ka).max(norm(kb)).max(1e-12)
            {
                return Err(Error::Invalid(
                    "curvature differs across this knot; choose a smooth station",
                ));
            }
            b
        }
        Err(e) => return Err(e),
    };
    let speed = norm(d[1]);
    let bending = norm(cross(d[1], d[2]));
    if speed <= 1e-12 || bending <= speed.powi(3) * 1e-12 {
        return Err(Error::Invalid("no finite curvature radius at this station"));
    }
    let radius = speed.powi(3) / bending;
    if !radius.is_finite() {
        return Err(Error::Invalid("curvature radius"));
    }
    Ok(radius)
}
/// Tensor Gauss quadrature, refined until two successive estimates agree. Explicitly
/// numeric area, not tessellation area; refuses when the evaluation budget cannot converge.
pub fn surface_area(surface: &NurbsSurface, tolerance: f64, limits: &Limits) -> Result<f64> {
    surface.validate(limits)?;
    if !tolerance.is_finite() || tolerance <= 0. {
        return Err(Error::Invalid("area integration tolerance"));
    }
    let roots = [-0.7745966692414834, 0., 0.7745966692414834];
    let weights = [5. / 9., 8. / 9., 5. / 9.];
    let mut previous = None;
    for n in [1usize, 2, 4, 8, 16, 32, 64] {
        if (n * n * 9) as u64 > limits.max_points {
            return Err(Error::Budget("surface area evaluations"));
        }
        let mut sum = 0.;
        for i in 0..n {
            for j in 0..n {
                for a in 0..3 {
                    for b in 0..3 {
                        let u = surface.domain[0][0]
                            + (surface.domain[0][1] - surface.domain[0][0])
                                * (i as f64 + (roots[a] + 1.) / 2.)
                                / n as f64;
                        let v = surface.domain[1][0]
                            + (surface.domain[1][1] - surface.domain[1][0])
                                * (j as f64 + (roots[b] + 1.) / 2.)
                                / n as f64;
                        let d = surface_kernel::derivatives(surface, u, v, limits)?;
                        sum += weights[a] * weights[b] * norm(cross(d.du, d.dv));
                    }
                }
            }
        }
        sum *= (surface.domain[0][1] - surface.domain[0][0])
            * (surface.domain[1][1] - surface.domain[1][0])
            / (4. * (n * n) as f64);
        if !sum.is_finite() {
            return Err(Error::Invalid("surface area"));
        }
        if previous.is_some_and(|p: f64| (sum - p).abs() <= tolerance) {
            return Ok(sum);
        }
        previous = Some(sum);
    }
    Err(Error::Invalid(
        "surface area did not converge to requested tolerance",
    ))
}
