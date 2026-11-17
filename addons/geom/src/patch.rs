// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 Patch: a surface fitted through curves and points.
//!
//! As the Rhino 5 help describes it: the best-fit plane through the input points (and points
//! sampled along the curves at *Sample point spacing*) is a U×V-span surface that deforms to
//! match the points; *Stiffness* says how much it may deform. Here the surface is a bicubic
//! B-spline over that plane whose heights solve a least-squares fit with a thin-plate
//! stiffness term. *Automatic trim* trims it to the closed outer curve, drawn on the surface.
#![allow(clippy::needless_range_loop)]
use crate::exact::{self, H, Nc};
use om9_addon_abi::session::{Vec3, add, cross, dist, dot, norm, scale, sub, unit};

#[derive(Debug, Clone)]
pub struct Options {
    /// Distance between sample points along the curves (at least 8 per curve).
    pub spacing: f64,
    pub u_spans: usize,
    pub v_spans: usize,
    /// Larger keeps the surface flatter (closer to the best-fit plane).
    pub stiffness: f64,
}

impl Default for Options {
    fn default() -> Self {
        Options { spacing: 1.0, u_spans: 10, v_spans: 10, stiffness: 1.0 }
    }
}

pub struct Patch {
    pub rows: Vec<Vec<H>>,
    pub degree: [usize; 2],
    pub knots: [Vec<f64>; 2],
    /// Largest distance from the input samples to the surface (along the plane normal).
    pub deviation: f64,
    origin: Vec3,
    axes: [Vec3; 3],
    lo: [f64; 2],
    hi: [f64; 2],
}

impl Patch {
    /// The surface point above a point (projected along the plane normal).
    pub fn above(&self, p: Vec3) -> Vec3 {
        let d = sub(p, self.origin);
        let (u, v) = (
            ((dot(d, self.axes[0]) - self.lo[0]) / (self.hi[0] - self.lo[0])).clamp(0.0, 1.0),
            ((dot(d, self.axes[1]) - self.lo[1]) / (self.hi[1] - self.lo[1])).clamp(0.0, 1.0),
        );
        self.at(u, v)
    }

    pub fn at(&self, u: f64, v: f64) -> Vec3 {
        let row: Vec<H> = (0..self.rows[0].len())
            .map(|j| {
                Nc { degree: 3, points: self.rows.iter().map(|r| r[j]).collect(), knots: self.knots[0].clone() }
                    .eval_h(u)
            })
            .collect();
        Nc { degree: 3, points: row, knots: self.knots[1].clone() }.eval(v)
    }

    /// Plane coordinates of a point.
    pub fn flat(&self, p: Vec3) -> [f64; 2] {
        let d = sub(p, self.origin);
        [dot(d, self.axes[0]), dot(d, self.axes[1])]
    }

    pub fn range(&self) -> ([f64; 2], [f64; 2]) {
        (self.lo, self.hi)
    }
}

/// Eigenvectors of a symmetric 3×3 matrix (Jacobi), columns sorted by eigenvalue (small first).
fn eigen(mut a: [[f64; 3]; 3]) -> [Vec3; 3] {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..50 {
        let (mut p, mut q, mut big) = (0, 1, 0.0);
        for i in 0..3 {
            for j in i + 1..3 {
                if a[i][j].abs() > big {
                    (p, q, big) = (i, j, a[i][j].abs());
                }
            }
        }
        if big < 1e-15 {
            break;
        }
        let th = 0.5 * (2.0 * a[p][q]).atan2(a[q][q] - a[p][p]);
        let (c, s) = (th.cos(), th.sin());
        for k in 0..3 {
            let (akp, akq) = (a[k][p], a[k][q]);
            a[k][p] = c * akp - s * akq;
            a[k][q] = s * akp + c * akq;
        }
        for k in 0..3 {
            let (apk, aqk) = (a[p][k], a[q][k]);
            a[p][k] = c * apk - s * aqk;
            a[q][k] = s * apk + c * aqk;
        }
        for k in 0..3 {
            let (vkp, vkq) = (v[k][p], v[k][q]);
            v[k][p] = c * vkp - s * vkq;
            v[k][q] = s * vkp + c * vkq;
        }
    }
    let mut idx = [0, 1, 2];
    idx.sort_by(|x, y| a[*x][*x].total_cmp(&a[*y][*y]));
    idx.map(|i| [v[0][i], v[1][i], v[2][i]])
}

fn clamped(spans: usize) -> Vec<f64> {
    let mut k = vec![0.0; 4];
    k.extend((1..spans).map(|i| i as f64 / spans as f64));
    k.extend([1.0; 4]);
    k
}

/// Samples along the curves (spacing apart, at least 8 per curve) and the points.
pub fn samples(curves: &[Nc], points: &[Vec3], spacing: f64) -> Vec<Vec3> {
    let mut out: Vec<Vec3> = points.to_vec();
    for c in curves {
        let len: f64 = (0..200).map(|i| dist(c.eval(i as f64 / 200.0), c.eval((i + 1) as f64 / 200.0))).sum();
        let n = ((len / spacing.max(1e-6)).ceil() as usize).clamp(8, 4000);
        out.extend((0..=n).map(|i| c.eval(i as f64 / n as f64)));
    }
    out
}

/// Fit the patch.
pub fn patch(curves: &[Nc], points: &[Vec3], o: &Options) -> Result<Patch, String> {
    let pts = samples(curves, points, o.spacing);
    if pts.len() < 3 {
        return Err("select curves or points".into());
    }
    let c = scale(pts.iter().fold([0.0; 3], |a, p| add(a, *p)), 1.0 / pts.len() as f64);
    let mut cov = [[0.0; 3]; 3];
    for p in &pts {
        let d = sub(*p, c);
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }
    let [n, _, x] = eigen(cov);
    if norm(n) < 0.5 {
        return Err("the input has no plane".into());
    }
    let x = unit(sub(x, scale(n, dot(x, n))));
    let y = cross(n, x);
    let local: Vec<Vec3> = pts.iter().map(|p| {
        let d = sub(*p, c);
        [dot(d, x), dot(d, y), dot(d, n)]
    }).collect();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in &local {
        for a in 0..2 {
            lo[a] = lo[a].min(p[a]);
            hi[a] = hi[a].max(p[a]);
        }
    }
    let size = (hi[0] - lo[0]).max(hi[1] - lo[1]);
    if size < 1e-9 {
        return Err("the input is too small".into());
    }
    // The input lies on a line seen from the plane: no surface.
    if (hi[0] - lo[0]).min(hi[1] - lo[1]) < 1e-6 * size {
        return Err("the input lies on a line".into());
    }
    for a in 0..2 {
        let pad = 0.02 * (hi[a] - lo[a]);
        lo[a] -= pad;
        hi[a] += pad;
    }
    let (us, vs) = (o.u_spans.clamp(1, 40), o.v_spans.clamp(1, 40));
    let (ku, kv) = (clamped(us), clamped(vs));
    let (nu, nv) = (us + 3, vs + 3);
    let m = nu * nv;
    let id = |i: usize, j: usize| i * nv + j;
    // Normal equations, banded (row-major ordering: band ≈ 3·nv + 3).
    let band = 3 * nv + 3;
    let mut a = vec![0.0; m * m];
    let mut b = vec![0.0; m];
    let basis_at = |p: &Vec3| {
        let u = ((p[0] - lo[0]) / (hi[0] - lo[0])).clamp(0.0, 1.0);
        let v = ((p[1] - lo[1]) / (hi[1] - lo[1])).clamp(0.0, 1.0);
        let su = crate::find_span(nu, 3, u, &ku);
        let sv = crate::find_span(nv, 3, v, &kv);
        let (bu, bv) = (crate::basis(su, u, 3, &ku), crate::basis(sv, v, 3, &kv));
        let mut out = Vec::with_capacity(16);
        for (i, wu) in bu.iter().enumerate() {
            for (j, wv) in bv.iter().enumerate() {
                out.push((id(su - 3 + i, sv - 3 + j), wu * wv));
            }
        }
        out
    };
    for p in &local {
        let w = basis_at(p);
        for &(r, wr) in &w {
            b[r] += wr * p[2];
            for &(s, ws) in &w {
                a[r * m + s] += wr * ws;
            }
        }
    }
    // Thin-plate stiffness on the heights: second differences across, along and mixed.
    let lambda = o.stiffness.max(0.0) * 1e-3 * local.len() as f64 / m as f64 + 1e-9;
    let mut add_row = |terms: &[(usize, f64)], weight: f64| {
        for &(r, wr) in terms {
            for &(s, ws) in terms {
                a[r * m + s] += weight * wr * ws;
            }
        }
    };
    for i in 0..nu {
        for j in 0..nv {
            if i + 2 < nu {
                add_row(&[(id(i, j), 1.0), (id(i + 1, j), -2.0), (id(i + 2, j), 1.0)], lambda);
            }
            if j + 2 < nv {
                add_row(&[(id(i, j), 1.0), (id(i, j + 1), -2.0), (id(i, j + 2), 1.0)], lambda);
            }
            if i + 1 < nu && j + 1 < nv {
                add_row(
                    &[(id(i, j), 1.0), (id(i + 1, j), -1.0), (id(i, j + 1), -1.0), (id(i + 1, j + 1), 1.0)],
                    2.0 * lambda,
                );
            }
        }
    }
    // Banded Cholesky.
    for i in 0..m {
        let j0 = i.saturating_sub(band);
        for j in j0..=i {
            let mut s = a[i * m + j];
            for k in j0.max(j.saturating_sub(band))..j {
                s -= a[i * m + k] * a[j * m + k];
            }
            if i == j {
                if s <= 0.0 {
                    return Err("could not fit the patch".into());
                }
                a[i * m + i] = s.sqrt();
            } else {
                a[i * m + j] = s / a[j * m + j];
            }
        }
    }
    let mut z = b;
    for i in 0..m {
        let j0 = i.saturating_sub(band);
        let mut s = z[i];
        for k in j0..i {
            s -= a[i * m + k] * z[k];
        }
        z[i] = s / a[i * m + i];
    }
    for i in (0..m).rev() {
        let mut s = z[i];
        for k in i + 1..(i + band + 1).min(m) {
            s -= a[k * m + i] * z[k];
        }
        z[i] = s / a[i * m + i];
    }
    // Greville abscissae place the poles so the surface is exactly a graph over the plane.
    let greville = |k: &[f64], i: usize| (k[i + 1] + k[i + 2] + k[i + 3]) / 3.0;
    let rows: Vec<Vec<H>> = (0..nu)
        .map(|i| {
            (0..nv)
                .map(|j| {
                    let lx = lo[0] + greville(&ku, i) * (hi[0] - lo[0]);
                    let ly = lo[1] + greville(&kv, j) * (hi[1] - lo[1]);
                    let q = add(c, add(scale(x, lx), add(scale(y, ly), scale(n, z[id(i, j)]))));
                    [q[0], q[1], q[2], 1.0]
                })
                .collect()
        })
        .collect();
    let mut out = Patch { rows, degree: [3, 3], knots: [ku, kv], deviation: 0.0, origin: c, axes: [x, y, n], lo, hi };
    out.deviation = pts.iter().map(|p| dist(*p, out.above(*p))).fold(0.0, f64::max);
    Ok(out)
}

/// A curve on the patch above `c` (projected along the plane normal), as exact NURBS
/// through dense samples: trim loops lie on the surface.
pub fn on_patch(p: &Patch, c: &Nc) -> Option<Nc> {
    let pts: Vec<Vec3> = (0..=96).map(|i| p.above(c.eval(i as f64 / 96.0))).collect();
    Nc::from_geo(&crate::interpolate(&pts, 3, false)?)
}

/// Rhino's automatic trim: the input curves that chain into one closed loop.
pub fn outer_loop(curves: &[Nc], tol: f64) -> Option<Vec<Nc>> {
    if curves.len() == 1 && curves[0].closed(tol) {
        return Some(vec![curves[0].clone()]);
    }
    let chain = exact::chain(curves, tol)?;
    let closed = chain.len() >= 2 && dist(chain[0].start(), chain[chain.len() - 1].end()) <= tol;
    closed.then_some(chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::Geo;

    #[test]
    fn a_patch_follows_curves_on_a_smooth_surface() {
        // Four boundary curves and one interior curve on z = 0.3·sin(x/3)·cos(y/4).
        let h = |x: f64, y: f64| 0.3 * (x / 3.0).sin() * (y / 4.0).cos();
        let curve = |f: &dyn Fn(f64) -> (f64, f64)| {
            let pts: Vec<Vec3> = (0..=12).map(|i| {
                let (x, y) = f(i as f64 / 12.0);
                [x, y, h(x, y)]
            }).collect();
            Nc::from_geo(&crate::interpolate(&pts, 3, false).unwrap()).unwrap()
        };
        let curves = vec![
            curve(&|t| (10.0 * t, 0.0)),
            curve(&|t| (10.0, 8.0 * t)),
            curve(&|t| (10.0 * (1.0 - t), 8.0)),
            curve(&|t| (0.0, 8.0 * (1.0 - t))),
            curve(&|t| (10.0 * t, 4.0)),
        ];
        let t0 = std::time::Instant::now();
        let p = patch(&curves, &[], &Options::default()).unwrap();
        eprintln!("patch: {:?}, deviation {:.4}", t0.elapsed(), p.deviation);
        assert!(p.deviation < 0.01, "{}", p.deviation);
        // Between the curves the patch is the smoothest surface through them: close to the
        // surface they were drawn on (the curves do not fix it there).
        for (x, y) in [(5.0, 2.0), (2.5, 6.0), (7.5, 5.0)] {
            let q = p.above([x, y, 0.0]);
            assert!((q[2] - h(q[0], q[1])).abs() < 0.05, "{q:?}");
        }
        // The boundary chains into a loop and lies on the patch once drawn on it.
        let lp = outer_loop(&curves[..4], 1e-6).unwrap();
        assert_eq!(lp.len(), 4);
        for c in &lp {
            let on = on_patch(&p, c).unwrap();
            for k in 0..=20 {
                let q = on.eval(k as f64 / 20.0);
                assert!(dist(q, p.above(q)) < 1e-4);
            }
        }
        // Stiffer: flatter.
        let stiff = patch(&curves, &[], &Options { stiffness: 1e5, ..Default::default() }).unwrap();
        assert!(stiff.deviation > p.deviation);
        let _ = Geo::Point { point: [0.0; 3] };
    }
}
