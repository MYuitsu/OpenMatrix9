// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 OffsetSrf on exact NURBS surfaces.
//!
//! - Planes, spheres and open cylinders offset exactly (Rhino 5 help: "the resulting surface
//!   is exact"): the offset is an affine map of the control points (a shift, a scale about
//!   the centre, a scale away from the axis), weights unchanged.
//! - Free-form surfaces: offset points are fitted on a grid that is refined until the fit is
//!   within the tolerance everywhere checked.
//! - Loose: every control point moves along the normal at its own place; the point structure
//!   stays the original's.
#![allow(clippy::needless_range_loop)]
use crate::exact::{self, Nc};
#[cfg(test)]
use crate::exact::H;
use crate::nurbs::Srf;
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, dot, norm, scale, sub, unit};

#[derive(Debug, Clone)]
pub struct Offset {
    pub geo: Geo,
    /// Largest distance error measured (0 when exact).
    pub deviation: f64,
    pub exact: bool,
}

/// Derivative along one parameter (second order, one-sided at the ends).
fn deriv(f: &dyn Fn(f64) -> Vec3, t: f64) -> Vec3 {
    let h = 1e-5;
    let comb = |a: Vec3, b: Vec3, c: Vec3, ka: f64, kb: f64, kc: f64, div: f64| -> Vec3 {
        std::array::from_fn(|k| (ka * a[k] + kb * b[k] + kc * c[k]) / div)
    };
    if t - h < 0.0 {
        comb(f(t), f(t + h), f(t + 2.0 * h), -3.0, 4.0, -1.0, 2.0 * h)
    } else if t + h > 1.0 {
        comb(f(t), f(t - h), f(t - 2.0 * h), 3.0, -4.0, 1.0, 2.0 * h)
    } else {
        comb(f(t + h), f(t - h), [0.0; 3], 1.0, -1.0, 0.0, 2.0 * h)
    }
}

/// The unit normal at (a, b) (the side `Srf::normal_st` points to).
pub(crate) fn normal(s: &Srf, a: f64, b: f64) -> Vec3 {
    let n = cross(deriv(&|t| s.at_st(t, b), a), deriv(&|t| s.at_st(a, t), b));
    if norm(n) > 1e-12 {
        let n = unit(n);
        let r = s.normal_st(a, b);
        return if r.iter().all(|x| x.is_finite()) && dot(n, r) < 0.0 { scale(n, -1.0) } else { n };
    }
    // At a singular point (a sphere's pole) step inwards a little.
    for e in [1e-6, 1e-4, 1e-3] {
        let (aa, bb) = (if a + e > 1.0 { a - e } else { a + e }, if b + e > 1.0 { b - e } else { b + e });
        let n = cross(deriv(&|t| s.at_st(t, bb), aa), deriv(&|t| s.at_st(aa, t), bb));
        if norm(n) > 1e-12 {
            let n = unit(n);
            let r = s.normal_st(aa, bb);
            return if r.iter().all(|x| x.is_finite()) && dot(n, r) < 0.0 { scale(n, -1.0) } else { n };
        }
    }
    [0.0, 0.0, 1.0]
}

fn grid(s: &Srf, k: usize) -> Vec<Vec3> {
    let mut out = Vec::with_capacity((k + 1) * (k + 1));
    for i in 0..=k {
        for j in 0..=k {
            out.push(s.at_st(i as f64 / k as f64, j as f64 / k as f64));
        }
    }
    out
}

fn map_poles(g: &Geo, f: impl Fn(Vec3) -> Vec3) -> Geo {
    match g {
        Geo::Surface { degree, size, points, weights, knots } => Geo::Surface {
            degree: *degree,
            size: *size,
            points: points.iter().map(|p| f(*p)).collect(),
            weights: weights.clone(),
            knots: knots.clone(),
        },
        other => other.clone(),
    }
}

/// Offset `g` by `d` along its normals (negative: the other way), within `tol`.
pub fn offset(g: &Geo, d: f64, tol: f64, loose: bool) -> Result<Offset, String> {
    let s = Srf::of(g).ok_or("not a NURBS surface")?;
    let pts = grid(&s, 8);
    let size = pts.iter().map(|p| dist(*p, pts[0])).fold(0.0, f64::max).max(1e-9);
    let eps = 1e-9 * size.max(1.0);
    let n_mid = normal(&s, 0.5, 0.5);
    let poles: Vec<Vec3> = match g {
        Geo::Surface { points, .. } => points.clone(),
        _ => return Err("not a NURBS surface".into()),
    };
    // Plane: every control point on one plane.
    let o = pts[0];
    if poles.iter().chain(&pts).all(|p| dot(sub(*p, o), n_mid).abs() <= eps) {
        let shift = scale(n_mid, d);
        return Ok(Offset { geo: map_poles(g, |p| add(p, shift)), deviation: 0.0, exact: true });
    }
    // Sphere: every point the same distance from one centre.
    if let Some(c) = sphere_center(&pts) {
        let r = dist(pts[0], c);
        if pts.iter().all(|p| (dist(*p, c) - r).abs() <= eps) {
            let p = s.at_st(0.5, 0.5);
            let outward = dot(sub(p, c), n_mid) > 0.0;
            let k = (r + if outward { d } else { -d }) / r;
            if k <= 0.0 {
                return Err("the offset is larger than the sphere's radius".into());
            }
            return Ok(Offset { geo: map_poles(g, |p| add(c, scale(sub(p, c), k))), deviation: 0.0, exact: true });
        }
    }
    // Open cylinder: every point the same distance from one axis.
    if let Some((c, a)) = cylinder_axis(&s) {
        let radial = |p: Vec3| {
            let q = sub(p, c);
            sub(q, scale(a, dot(q, a)))
        };
        let r = norm(radial(pts[0]));
        if r > eps && pts.iter().all(|p| (norm(radial(*p)) - r).abs() <= eps) {
            let p = s.at_st(0.5, 0.5);
            let outward = dot(radial(p), n_mid) > 0.0;
            let k = (r + if outward { d } else { -d }) / r;
            if k <= 0.0 {
                return Err("the offset is larger than the cylinder's radius".into());
            }
            return Ok(Offset {
                geo: map_poles(g, |p| add(p, scale(radial(p), k - 1.0))),
                deviation: 0.0,
                exact: true,
            });
        }
    }
    let off = |a: f64, b: f64| add(s.at_st(a, b), scale(normal(&s, a, b), d));
    // Fit error: the fitted surface against the true offset at the same parameters, between
    // the fitted points.
    let check = |geo: &Geo| -> f64 {
        let f = Srf::of(geo).unwrap();
        let k = 24;
        let mut worst = 0.0f64;
        for i in 0..k {
            for j in 0..k {
                let (a, b) = ((i as f64 + 0.5) / k as f64, (j as f64 + 0.5) / k as f64);
                worst = worst.max(dist(f.at_st(a, b), off(a, b)));
            }
        }
        worst
    };
    if loose {
        // Each control point along the normal at its Greville place.
        let Geo::Surface { degree, size, knots, .. } = g else { unreachable!() };
        let greville = |k: &[f64], p: usize, i: usize| (1..=p).map(|r| k[i + r]).sum::<f64>() / p.max(1) as f64;
        let (d0, d1) = (s.domain(0), s.domain(1));
        let mut out = poles.clone();
        for i in 0..size[0] {
            for j in 0..size[1] {
                let a = (greville(&knots[0], degree[0], i) - d0[0]) / (d0[1] - d0[0]);
                let b = (greville(&knots[1], degree[1], j) - d1[0]) / (d1[1] - d1[0]);
                out[i * size[1] + j] = add(poles[i * size[1] + j], scale(normal(&s, a, b), d));
            }
        }
        let geo = map_poles(g, |p| p);
        let geo = match geo {
            Geo::Surface { degree, size, weights, knots, .. } => Geo::Surface { degree, size, points: out, weights, knots },
            other => other,
        };
        let dev = check(&geo);
        return Ok(Offset { geo, deviation: dev, exact: false });
    }
    // Free form: fit the offset points, finer until within tol.
    fit_grid(&off, tol)
}

/// A surface through the points `off(a, b)` on a grid refined until it is within `tol` of
/// them between the grid points too.
fn fit_grid(off: &dyn Fn(f64, f64) -> Vec3, tol: f64) -> Result<Offset, String> {
    let check = |geo: &Geo| -> f64 {
        let f = Srf::of(geo).unwrap();
        let k = 24;
        let mut worst = 0.0f64;
        for i in 0..k {
            for j in 0..k {
                let (a, b) = ((i as f64 + 0.5) / k as f64, (j as f64 + 0.5) / k as f64);
                worst = worst.max(dist(f.at_st(a, b), off(a, b)));
            }
        }
        worst
    };
    let mut k = 8;
    loop {
        let params: Vec<f64> = (0..=k).map(|i| i as f64 / k as f64).collect();
        let rows: Option<Vec<Nc>> = params
            .iter()
            .map(|a| {
                let pcs: Vec<Nc> = params.iter().map(|b| exact::point_curve(off(*a, *b))).collect();
                let (r, kn) = exact::interpolate_across(&pcs, &params, 3, false)?;
                Some(Nc { degree: 3, points: r.iter().map(|x| x[0]).collect(), knots: kn })
            })
            .collect();
        let rows = rows.ok_or("could not fit the offset")?;
        let (across, ku) = exact::interpolate_across(&rows, &params, 3, false).ok_or("could not fit the offset")?;
        let geo = exact::surface(&across, [3, rows[0].degree], [ku, rows[0].knots.clone()]);
        let dev = check(&geo);
        if dev <= tol || k >= 96 {
            return Ok(Offset { geo, deviation: dev, exact: false });
        }
        k *= 2;
    }
}

/// Rhino 5 VariableOffsetSrf: the offset distance varies over the surface — through the
/// handles (place (a, b) on the surface, distance): bilinear when the handles are the four
/// corners, otherwise smoothly weighted by their parameter distance; fitted within `tol`.
pub fn variable_offset(g: &Geo, handles: &[((f64, f64), f64)], tol: f64) -> Result<Offset, String> {
    let s = Srf::of(g).ok_or("not a NURBS surface")?;
    if handles.is_empty() {
        return Err("no handles".into());
    }
    let corner = |a: f64, b: f64| handles.iter().find(|h| (h.0.0 - a).abs() < 1e-9 && (h.0.1 - b).abs() < 1e-9).map(|h| h.1);
    let corners = [corner(0.0, 0.0), corner(1.0, 0.0), corner(0.0, 1.0), corner(1.0, 1.0)];
    let bilinear = handles.len() == 4 && corners.iter().all(Option::is_some);
    let d = |a: f64, b: f64| -> f64 {
        if bilinear {
            let [c00, c10, c01, c11] = corners.map(Option::unwrap);
            return c00 * (1.0 - a) * (1.0 - b) + c10 * a * (1.0 - b) + c01 * (1.0 - a) * b + c11 * a * b;
        }
        let (mut num, mut den) = (0.0, 0.0);
        for ((ha, hb), hd) in handles {
            let r2 = (a - ha).powi(2) + (b - hb).powi(2);
            if r2 < 1e-18 {
                return *hd;
            }
            let w = 1.0 / (r2 * r2);
            num += w * hd;
            den += w;
        }
        num / den
    };
    let off = |a: f64, b: f64| add(s.at_st(a, b), scale(normal(&s, a, b), d(a, b)));
    fit_grid(&off, tol)
}

/// The centre of the sphere through the points, if they are on one.
fn sphere_center(p: &[Vec3]) -> Option<Vec3> {
    // Four spread points: solve |x − c|² = r² differences (3×3).
    // Four well spread points: the first, the farthest from it, the farthest from that line,
    // the farthest from that plane.
    let far = |f: &dyn Fn(Vec3) -> f64| *p.iter().max_by(|a, b| f(**a).total_cmp(&f(**b))).unwrap();
    let a0 = p[0];
    let a1 = far(&|x| dist(x, a0));
    let dir = unit(sub(a1, a0));
    let a2 = far(&|x| norm(cross(sub(x, a0), dir)));
    let nrm = cross(sub(a1, a0), sub(a2, a0));
    if norm(nrm) < 1e-12 {
        return None;
    }
    let nrm = unit(nrm);
    let a3 = far(&|x| dot(sub(x, a0), nrm).abs());
    let pick = [a0, a1, a2, a3];
    let a: Vec<Vec3> = (1..4).map(|i| scale(sub(pick[i], pick[0]), 2.0)).collect();
    let b: Vec<f64> = (1..4).map(|i| dot(pick[i], pick[i]) - dot(pick[0], pick[0])).collect();
    let det = dot(a[0], cross(a[1], a[2]));
    if det.abs() < 1e-12 {
        return None;
    }
    // Cramer.
    let col = |k: usize| -> f64 {
        let m: Vec<Vec3> = (0..3).map(|r| { let mut v = a[r]; v[k] = b[r]; v }).collect();
        dot(m[0], cross(m[1], m[2])) / det
    };
    Some([col(0), col(1), col(2)])
}

/// The axis of a cylinder-like surface: every normal (nearly) perpendicular to one direction;
/// the axis point from the circle through three surface points (exact from the points).
fn cylinder_axis(s: &Srf) -> Option<(Vec3, Vec3)> {
    // Two normals far from parallel (interior points: the numeric normal is best there).
    let ns: Vec<Vec3> = [(0.15, 0.4), (0.4, 0.6), (0.65, 0.3), (0.9, 0.5)].iter().map(|(a, b)| normal(s, *a, *b)).collect();
    let (mut n1, mut n2, mut best) = (ns[0], ns[1], 0.0);
    for i in 0..ns.len() {
        for j in i + 1..ns.len() {
            let c = norm(cross(ns[i], ns[j]));
            if c > best {
                (n1, n2, best) = (ns[i], ns[j], c);
            }
        }
    }
    if best < 1e-3 {
        return None;
    }
    let mut a = unit(cross(n1, n2));
    for i in 0..7 {
        for j in 0..7 {
            if dot(normal(s, (i as f64 + 0.5) / 7.0, (j as f64 + 0.5) / 7.0), a).abs() > 1e-5 {
                return None;
            }
        }
    }
    // Refine the direction from the surface's own points: along a ruling (a line on the
    // cylinder) the points differ only along the axis.
    let (p0, p1) = (s.at_st(0.37, 0.0), s.at_st(0.37, 1.0));
    if dist(p0, p1) > 1e-9 && norm(cross(unit(sub(p1, p0)), a)) < 1e-4 {
        a = unit(sub(p1, p0));
        if dot(a, cross(n1, n2)) < 0.0 {
            a = scale(a, -1.0);
        }
    }
    let flat = |v: Vec3| sub(v, scale(a, dot(v, a)));
    let (q1, q2, q3) = (flat(s.at_st(0.1, 0.5)), flat(s.at_st(0.45, 0.5)), flat(s.at_st(0.8, 0.5)));
    let (u, v) = (sub(q2, q1), sub(q3, q1));
    let w = cross(u, v);
    let ww = dot(w, w);
    if ww < 1e-18 {
        return None;
    }
    let c = add(q1, scale(add(scale(cross(w, u), dot(v, v)), scale(cross(v, w), dot(u, u))), 0.5 / ww));
    Some((c, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surf(rows: &[Vec<H>], degree: [usize; 2], knots: [Vec<f64>; 2]) -> Geo {
        exact::surface(rows, degree, knots)
    }

    #[test]
    fn exact_plane_sphere_cylinder_and_a_fitted_freeform() {
        // Sphere r5: revolve a half circle (exact rational).
        let half = exact::arc([0.0; 3], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], 5.0, std::f64::consts::PI);
        let circle = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, std::f64::consts::TAU);
        // Rows: around (circle) × along the half circle: pole = rotate(profile pole) by the
        // circle's rational rotation (tensor product of weights).
        let rows: Vec<Vec<H>> = circle
            .points
            .iter()
            .map(|c| {
                let (cx, cy, cw) = (c[0] / c[3], c[1] / c[3], c[3]);
                half.points
                    .iter()
                    .map(|h| {
                        let (x, z, w) = (h[0] / h[3], h[2] / h[3], h[3]);
                        let p = [x * cx, x * cy, z];
                        let ww = w * cw;
                        [p[0] * ww, p[1] * ww, p[2] * ww, ww]
                    })
                    .collect()
            })
            .collect();
        let sphere = surf(&rows, [2, 2], [circle.knots.clone(), half.knots.clone()]);
        let o = offset(&sphere, 1.0, 0.001, false).unwrap();
        assert!(o.exact);
        let f = Srf::of(&o.geo).unwrap();
        let r_in = norm(f.at_st(0.3, 0.4));
        assert!((r_in - 6.0).abs() < 1e-9 || (r_in - 4.0).abs() < 1e-9, "{r_in}");
        // Plane.
        let plane = Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![[0.0; 3], [0.0, 5.0, 0.0], [5.0, 0.0, 0.0], [5.0, 5.0, 0.0]],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        };
        let o = offset(&plane, 2.0, 0.001, false).unwrap();
        assert!(o.exact);
        assert!((Srf::of(&o.geo).unwrap().at_st(0.3, 0.3)[2].abs() - 2.0).abs() < 1e-12);
        // Cylinder r3 along z.
        let ring = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, std::f64::consts::TAU);
        let rows: Vec<Vec<H>> = ring.points.iter().map(|c| vec![*c, [c[0], c[1], 4.0 * c[3], c[3]]]).collect();
        let cyl = surf(&rows, [2, 1], [ring.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]]);
        let o = offset(&cyl, 0.5, 0.001, false).unwrap();
        assert!(o.exact);
        let p = Srf::of(&o.geo).unwrap().at_st(0.37, 0.5);
        let r = p[0].hypot(p[1]);
        assert!((r - 3.5).abs() < 1e-9 || (r - 2.5).abs() < 1e-9, "{r}");
        // Free form: a bumpy bicubic; the offset stays at distance 0.8 within the tolerance.
        let mut rows = Vec::new();
        for i in 0..5 {
            let mut row = Vec::new();
            for j in 0..5 {
                let z = if (i, j) == (2, 2) { 2.0 } else { ((i + j) % 2) as f64 * 0.5 };
                row.push([i as f64 * 2.0, j as f64 * 2.0, z, 1.0]);
            }
            rows.push(row);
        }
        let kn = vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0];
        let free = surf(&rows, [3, 3], [kn.clone(), kn]);
        let t0 = std::time::Instant::now();
        let o = offset(&free, 0.8, 0.001, false).unwrap();
        eprintln!("offset freeform: {:?}, deviation {:.5}", t0.elapsed(), o.deviation);
        assert!(!o.exact && o.deviation <= 0.001, "{}", o.deviation);
        let (f, s) = (Srf::of(&o.geo).unwrap(), Srf::of(&free).unwrap());
        for i in 0..=10 {
            for j in 0..=10 {
                let q = f.at_st(i as f64 / 10.0, j as f64 / 10.0);
                let (a, b) = s.closest_st(q);
                let d = dist(q, s.at_st(a, b));
                // Near the edges the closest point can sit on the boundary.
                if (1..10).contains(&i) && (1..10).contains(&j) {
                    assert!((d - 0.8).abs() < 0.002, "{d} at {i},{j}");
                }
            }
        }
        // Along the edges too: the offset edge is 0.8 from the original edge's point.
        for k in 0..=20 {
            let t = k as f64 / 20.0;
            for (a, b) in [(t, 0.0), (t, 1.0), (0.0, t), (1.0, t)] {
                let d = dist(f.at_st(a, b), s.at_st(a, b));
                assert!((d - 0.8).abs() < 0.001, "edge {d} at {a},{b}");
            }
        }
    }

    #[test]
    fn variable_offset_goes_from_corner_to_corner() {
        let plane = Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![[0.0, 0.0, 0.0], [0.0, 5.0, 0.0], [5.0, 0.0, 0.0], [5.0, 5.0, 0.0]],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        };
        let h = [((0.0, 0.0), 1.0), ((1.0, 0.0), 2.0), ((0.0, 1.0), 1.0), ((1.0, 1.0), 2.0)];
        let o = variable_offset(&plane, &h, 0.001).unwrap();
        let f = Srf::of(&o.geo).unwrap();
        for i in 0..=10 {
            let a = i as f64 / 10.0;
            let z = f.at_st(a, 0.5)[2].abs();
            assert!((z - (1.0 + a)).abs() < 1e-6, "{z} at {a}");
        }
    }
}
