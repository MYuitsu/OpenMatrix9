// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 ExtendSrf on exact NURBS surfaces.
//!
//! - **Smooth**: the last span carries on as the same polynomial (rational) piece — its
//!   Bézier form is extrapolated by de Casteljau past t = 1 — so the extension joins with
//!   every derivative continuous; it is exact (the surface it already was, longer).
//! - **Line**: each curve across the edge continues straight along its end tangent by the
//!   distance (a linear Bézier piece of the same degree appended at the edge).
#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::exact::{H, Nc, euclid};
use om9_addon_abi::session::{Geo, Vec3, dist, norm, scale, sub, unit};

/// The curves of a surface running in direction `dir` (0: along u), one per pole of the
/// other direction.
pub(crate) fn lines(g: &Geo, dir: usize) -> Option<(Vec<Nc>, [usize; 2], [usize; 2], [Vec<f64>; 2])> {
    let Geo::Surface { degree, size, points, weights, knots } = g else { return None };
    let w = |i: usize| weights.get(i).copied().unwrap_or(1.0);
    let at = |i: usize, j: usize| {
        let k = i * size[1] + j;
        let p = points[k];
        [p[0] * w(k), p[1] * w(k), p[2] * w(k), w(k)]
    };
    let curves = if dir == 0 {
        (0..size[1]).map(|j| Nc { degree: degree[0], points: (0..size[0]).map(|i| at(i, j)).collect(), knots: knots[0].clone() }).collect()
    } else {
        (0..size[0]).map(|i| Nc { degree: degree[1], points: (0..size[1]).map(|j| at(i, j)).collect(), knots: knots[1].clone() }).collect()
    };
    Some((curves, *degree, *size, knots.clone()))
}

pub(crate) fn assemble(curves: &[Nc], dir: usize, degree: [usize; 2], knots: [Vec<f64>; 2]) -> Geo {
    let n_along = curves[0].points.len();
    let n_across = curves.len();
    let (size, at): ([usize; 2], Box<dyn Fn(usize, usize) -> H>) = if dir == 0 {
        ([n_along, n_across], Box::new(move |i, j| curves[j].points[i]))
    } else {
        ([n_across, n_along], Box::new(move |i, j| curves[i].points[j]))
    };
    let mut points = Vec::with_capacity(size[0] * size[1]);
    let mut weights = Vec::with_capacity(size[0] * size[1]);
    for i in 0..size[0] {
        for j in 0..size[1] {
            let h = at(i, j);
            points.push(euclid(h));
            weights.push(h[3]);
        }
    }
    let mut kn = knots;
    kn[dir] = curves[0].knots.clone();
    Geo::Surface { degree, size, points, weights, knots: kn }
}

/// Normalise knots to [0, 1].
fn normalise(k: &mut [f64]) {
    let (a, b) = (k[0], *k.last().unwrap());
    let span = (b - a).max(1e-300);
    k.iter_mut().for_each(|v| *v = (*v - a) / span);
}

/// The curve on [a, 1 + delta] of the last span (made Bézier first), continuing its
/// polynomial; knots stay unnormalised (domain end 1 + delta).
fn smooth_end(c: &Nc, delta: f64) -> Nc {
    let p = c.degree;
    let mut c = c.clone();
    let n = c.knots.len();
    // The last span's start.
    let a = c.knots[..n - p - 1].iter().copied().filter(|k| *k < 1.0 - 1e-14).fold(0.0, f64::max);
    if a > 0.0 {
        let have = c.knots.iter().filter(|k| (**k - a).abs() <= 1e-14).count();
        for _ in have..p {
            c.insert(a);
        }
    }
    let m = c.points.len();
    let bez: Vec<H> = c.points[m - p - 1..].to_vec();
    let t = (1.0 + delta - a) / (1.0 - a);
    // de Casteljau at t (> 1: extrapolation); the left polygon is the curve on [a, 1 + δ].
    let mut work = bez;
    let mut left = vec![work[0]];
    for level in 1..=p {
        for i in 0..=p - level {
            work[i] = std::array::from_fn(|k| work[i][k] + (work[i + 1][k] - work[i][k]) * t);
        }
        left.push(work[0]);
    }
    let mut points = c.points[..m - p - 1].to_vec();
    points.extend(left);
    let mut knots = c.knots[..c.knots.len() - p - 1].to_vec();
    knots.extend(std::iter::repeat_n(1.0 + delta, p + 1));
    Nc { degree: p, points, knots }
}

/// Straight continuation by `len` along the end tangent (a linear piece of degree p).
fn line_end(c: &Nc, len: f64) -> Nc {
    let p = c.degree;
    let m = c.points.len();
    let end = c.points[m - 1];
    let e = euclid(end);
    let tan = (|| {
        // Clamped end: the tangent runs along the last leg of the control polygon.
        for k in (0..m - 1).rev() {
            let d = sub(e, euclid(c.points[k]));
            if norm(d) > 1e-12 {
                return unit(d);
            }
        }
        for h in [1e-7, 1e-5, 1e-3] {
            let d = sub(e, c.eval(1.0 - h));
            if norm(d) > 1e-14 {
                return unit(d);
            }
        }
        [0.0; 3]
    })();
    let w = end[3];
    let mut points = c.points.clone();
    for k in 1..=p {
        let q = om9_addon_abi::session::add(e, scale(tan, len * k as f64 / p as f64));
        points.push([q[0] * w, q[1] * w, q[2] * w, w]);
    }
    let mut knots = c.knots[..c.knots.len() - 1].to_vec();
    // End knot 1 now interior with multiplicity p; the new piece runs to 2.
    knots.extend(std::iter::repeat_n(2.0, p + 1));
    Nc { degree: p, points, knots }
}

/// Rhino 5 ExtendSrf: extend `g` across its edge `dir` (0: the u edges, 1: the v edges) at
/// `at_end` (u or v = 1) by `len`.
pub fn extend(g: &Geo, dir: usize, at_end: bool, len: f64, smooth: bool) -> Result<Geo, String> {
    if len <= 0.0 {
        return Err("the extension must be more than 0".into());
    }
    let (mut curves, degree, _size, knots) = lines(g, dir).ok_or("not a NURBS surface")?;
    if !at_end {
        curves = curves.iter().map(Nc::reversed).collect();
    }
    let mid = curves.len() / 2;
    let out: Vec<Nc> = if smooth {
        // δ so the middle curve's extension is `len` long (secant on the length).
        // Arc length of the extension: chord sums with Richardson extrapolation (error ~ h⁴).
        let ext_len = |d: f64| -> f64 {
            let c = smooth_end(&curves[mid], d);
            let chords = |n: usize| -> f64 {
                let mut run = 0.0;
                let mut prev = c.eval_raw(1.0);
                for k in 1..=n {
                    let q = c.eval_raw(1.0 + d * k as f64 / n as f64);
                    run += dist(q, prev);
                    prev = q;
                }
                run
            };
            let (a, b) = (chords(64), chords(128));
            (4.0 * b - a) / 3.0
        };
        let speed = {
            let c = &curves[mid];
            dist(c.eval(1.0), c.eval(1.0 - 1e-6)) / 1e-6
        };
        if speed < 1e-12 {
            return Err("the edge is a point; it cannot be extended".into());
        }
        let (mut d0, mut d1) = (len / speed * 0.5, len / speed);
        let (mut l0, mut l1) = (ext_len(d0), ext_len(d1));
        for _ in 0..30 {
            if (l1 - len).abs() < 1e-9 * len.max(1.0) || (l1 - l0).abs() < 1e-300 {
                break;
            }
            let d2 = (d1 + (len - l1) * (d1 - d0) / (l1 - l0)).max(1e-9);
            (d0, l0) = (d1, l1);
            d1 = d2;
            l1 = ext_len(d1);
        }
        curves.iter().map(|c| smooth_end(c, d1)).collect()
    } else {
        curves.iter().map(|c| line_end(c, len)).collect()
    };
    let mut out: Vec<Nc> = out
        .into_iter()
        .map(|mut c| {
            normalise(&mut c.knots);
            c
        })
        .collect();
    if !at_end {
        out = out.iter().map(Nc::reversed).collect();
    }
    Ok(assemble(&out, dir, degree, knots))
}

impl Nc {
    /// Evaluate on the raw (unnormalised) knot domain.
    fn eval_raw(&self, u: f64) -> Vec3 {
        let (a, b) = (self.knots[0], *self.knots.last().unwrap());
        let mut c = self.clone();
        normalise(&mut c.knots);
        c.eval(((u - a) / (b - a)).clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact;
    use crate::nurbs::Srf;

    #[test]
    fn smooth_extension_continues_the_same_surface() {
        // A quarter of a cylinder r 4 (exact, rational) extended smoothly by 2 mm along its
        // arc: still on the cylinder; the arc part is unchanged.
        let arc = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 4.0, std::f64::consts::FRAC_PI_2);
        let rows: Vec<Vec<H>> = arc.points.iter().map(|c| vec![*c, [c[0], c[1], 3.0 * c[3], c[3]]]).collect();
        let g = exact::surface(&rows, [2, 1], [arc.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]]);
        let x = extend(&g, 0, true, 2.0, true).unwrap();
        let s = Srf::of(&x).unwrap();
        for i in 0..=40 {
            for j in 0..=4 {
                let p = s.at_st(i as f64 / 40.0, j as f64 / 4.0);
                assert!((p[0].hypot(p[1]) - 4.0).abs() < 1e-9, "{p:?}");
            }
        }
        // The new edge: 2 mm of arc past (0, 4): angle 0.5 rad more.
        let e = s.at_st(1.0, 0.0);
        let ang = e[1].atan2(e[0]);
        assert!((ang - (std::f64::consts::FRAC_PI_2 + 0.5)).abs() < 1e-6, "{ang}");
        // Line: straight on along the tangent at the edge.
        let x = extend(&g, 0, false, 2.0, false).unwrap();
        let s = Srf::of(&x).unwrap();
        let e = s.at_st(0.0, 0.5);
        assert!((e[0] - 4.0).abs() < 1e-9 && (e[1] + 2.0).abs() < 1e-9, "{e:?}");
    }
}
