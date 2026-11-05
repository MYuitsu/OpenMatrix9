// SPDX-License-Identifier: LGPL-2.1-or-later
//! NURBS surfaces (evaluation, normals, isocurves, closest parameters on a sample grid) and
//! curve helpers (samples, end tangents, blends, biarcs) for the Curve and Surface menus.
use crate::{basis, eval, eval_geo, find_span};
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, dot, norm, scale, sub, unit};

/// A NURBS surface borrowed from a [`Geo::Surface`].
#[derive(Debug, Clone, Copy)]
pub struct Srf<'a> {
    pub degree: [usize; 2],
    pub size: [usize; 2],
    pub points: &'a [Vec3],
    pub weights: &'a [f64],
    pub knots: [&'a [f64]; 2],
}

impl<'a> Srf<'a> {
    pub fn of(g: &'a Geo) -> Option<Srf<'a>> {
        match g {
            Geo::Surface {
                degree,
                size,
                points,
                weights,
                knots,
            } => Some(Srf {
                degree: *degree,
                size: *size,
                points,
                weights,
                knots: [&knots[0], &knots[1]],
            }),
            _ => None,
        }
    }
    /// Parameter domain in each direction.
    pub fn domain(&self, dir: usize) -> [f64; 2] {
        [
            self.knots[dir][self.degree[dir]],
            self.knots[dir][self.size[dir]],
        ]
    }
    /// Map normalized (s, t) ∈ [0, 1]² to the parameters.
    pub fn params(&self, s: f64, t: f64) -> (f64, f64) {
        let (du, dv) = (self.domain(0), self.domain(1));
        (
            du[0] + s.clamp(0.0, 1.0) * (du[1] - du[0]),
            dv[0] + t.clamp(0.0, 1.0) * (dv[1] - dv[0]),
        )
    }
    pub fn at(&self, u: f64, v: f64) -> Vec3 {
        let [pu, pv] = self.degree;
        let [nu, nv] = self.size;
        let su = find_span(nu, pu, u, self.knots[0]);
        let sv = find_span(nv, pv, v, self.knots[1]);
        let bu = basis(su, u, pu, self.knots[0]);
        let bv = basis(sv, v, pv, self.knots[1]);
        let mut acc = [0.0; 4];
        for (i, a) in bu.iter().enumerate() {
            for (j, b) in bv.iter().enumerate() {
                let iu = su - pu + i;
                let iv = sv - pv + j;
                let k = iu * nv + iv;
                let w = self.weights.get(k).copied().unwrap_or(1.0) * a * b;
                for c in 0..3 {
                    acc[c] += w * self.points[k][c];
                }
                acc[3] += w;
            }
        }
        [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]]
    }
    /// Point at normalized (s, t).
    pub fn at_st(&self, s: f64, t: f64) -> Vec3 {
        let (u, v) = self.params(s, t);
        self.at(u, v)
    }
    /// Unit normal at normalized (s, t) by central differences.
    pub fn normal_st(&self, s: f64, t: f64) -> Vec3 {
        let h = 1e-4;
        let du = sub(
            self.at_st((s + h).min(1.0), t),
            self.at_st((s - h).max(0.0), t),
        );
        let dv = sub(
            self.at_st(s, (t + h).min(1.0)),
            self.at_st(s, (t - h).max(0.0)),
        );
        unit(cross(du, dv))
    }
    /// The isocurve at normalized parameter `at` in direction `dir` (0: u fixed → curve along
    /// v; 1: v fixed → curve along u), exact.
    pub fn isocurve(&self, dir: usize, at: f64) -> Geo {
        let (u, v) = self.params(at, at);
        let fixed = if dir == 0 { u } else { v };
        let (pd, nd, kd) = (self.degree[dir], self.size[dir], self.knots[dir]);
        let span = find_span(nd, pd, fixed, kd);
        let b = basis(span, fixed, pd, kd);
        let other = 1 - dir;
        let n_other = self.size[other];
        let mut points = Vec::with_capacity(n_other);
        let mut weights = Vec::with_capacity(n_other);
        for j in 0..n_other {
            let mut acc = [0.0; 4];
            for (i, bi) in b.iter().enumerate() {
                let id = span - pd + i;
                let k = if dir == 0 {
                    id * self.size[1] + j
                } else {
                    j * self.size[1] + id
                };
                let w = self.weights.get(k).copied().unwrap_or(1.0) * bi;
                for c in 0..3 {
                    acc[c] += w * self.points[k][c];
                }
                acc[3] += w;
            }
            points.push([acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]]);
            weights.push(acc[3]);
        }
        let rational = weights.iter().any(|w| (w - 1.0).abs() > 1e-12);
        Geo::Nurbs {
            degree: self.degree[other],
            points,
            weights: if rational { weights } else { Vec::new() },
            knots: self.knots[other].to_vec(),
        }
    }
    /// Normalized (s, t) of the grid sample nearest to `p`, refined by a few local steps.
    pub fn closest_st(&self, p: Vec3) -> (f64, f64) {
        let n = 32;
        let mut best = (f64::MAX, 0.0, 0.0);
        for i in 0..=n {
            for j in 0..=n {
                let (s, t) = (i as f64 / n as f64, j as f64 / n as f64);
                let d = dist(self.at_st(s, t), p);
                if d < best.0 {
                    best = (d, s, t);
                }
            }
        }
        let (mut s, mut t) = (best.1, best.2);
        let mut h = 1.0 / n as f64;
        for _ in 0..30 {
            let mut moved = false;
            for (ds, dt) in [(h, 0.0), (-h, 0.0), (0.0, h), (0.0, -h)] {
                let (s2, t2) = ((s + ds).clamp(0.0, 1.0), (t + dt).clamp(0.0, 1.0));
                if dist(self.at_st(s2, t2), p) < dist(self.at_st(s, t), p) {
                    s = s2;
                    t = t2;
                    moved = true;
                }
            }
            if !moved {
                h *= 0.5;
            }
        }
        (s, t)
    }
}

/// `n` + 1 samples of a curve (open) or `n` (closed circles) at even parameters.
pub fn samples(g: &Geo, n: usize) -> Vec<Vec3> {
    let closed = matches!(g, Geo::Circle { .. });
    let count = if closed { n } else { n + 1 };
    (0..count)
        .filter_map(|i| eval_geo(g, i as f64 / n as f64))
        .collect()
}

/// Unit tangent at the start (`end = false`) or end of a curve, pointing along the curve.
pub fn end_tangent(g: &Geo, end: bool) -> Option<Vec3> {
    let (a, b) = if end { (0.999, 1.0) } else { (0.0, 0.001) };
    let d = sub(eval_geo(g, b)?, eval_geo(g, a)?);
    (norm(d) > 1e-15).then(|| unit(d))
}

/// A cubic (G1) or quintic (G2-like) Bézier blend from `p0` leaving along `t0` to `p1`
/// arriving along `t1`, with handle lengths scaled by `bulge`.
pub fn blend(p0: Vec3, t0: Vec3, p1: Vec3, t1: Vec3, bulge: [f64; 2], curvature: bool) -> Geo {
    let d = dist(p0, p1);
    let (h0, h1) = (d * bulge[0] / 3.0, d * bulge[1] / 3.0);
    let points = if curvature {
        vec![
            p0,
            add(p0, scale(t0, h0 * 0.6)),
            add(p0, scale(t0, h0 * 1.2)),
            sub(p1, scale(t1, h1 * 1.2)),
            sub(p1, scale(t1, h1 * 0.6)),
            p1,
        ]
    } else {
        vec![p0, add(p0, scale(t0, h0)), sub(p1, scale(t1, h1)), p1]
    };
    let degree = points.len() - 1;
    Geo::Nurbs {
        degree,
        knots: crate::clamped_uniform(points.len(), degree),
        points,
        weights: Vec::new(),
    }
}

/// A biarc (two tangent-continuous circular arcs) from `p0` along `t0` to `p1` along `t1`, as
/// polyline samples (exact arcs are rebuilt by the caller when needed).
pub fn biarc(p0: Vec3, t0: Vec3, p1: Vec3, t1: Vec3, samples_per_arc: usize) -> Vec<Vec3> {
    let v = sub(p1, p0);
    let t = add(t0, t1);
    // Equal-parameter biarc: |d| from the quadratic for the junction distance.
    let a = 2.0 * (1.0 - dot(t0, t1));
    let b = 2.0 * dot(v, t);
    let c = -dot(v, v);
    let d = if a.abs() < 1e-12 {
        if b.abs() < 1e-12 {
            norm(v) / 2.0
        } else {
            -c / b
        }
    } else {
        (-b + (b * b - 4.0 * a * c).max(0.0).sqrt()) / (2.0 * a)
    };
    let q0 = add(p0, scale(t0, d));
    let q1 = sub(p1, scale(t1, d));
    let m = scale(add(q0, q1), 0.5);
    let arc = |a: Vec3, ta: Vec3, b: Vec3| -> Vec<Vec3> {
        // The circle through a and b tangent to ta at a.
        let chord = sub(b, a);
        let n = cross(ta, chord);
        if norm(n) < 1e-12 {
            return (0..=samples_per_arc)
                .map(|i| add(a, scale(chord, i as f64 / samples_per_arc as f64)))
                .collect();
        }
        let toward = unit(cross(n, ta));
        let r = dot(chord, chord) / (2.0 * dot(chord, toward));
        let center = add(a, scale(toward, r));
        let x = unit(sub(a, center));
        let y = unit(cross(unit(n), x));
        let end = sub(b, center);
        let ang = dot(end, y).atan2(dot(end, x));
        let ang = if ang < 0.0 {
            ang + std::f64::consts::TAU
        } else {
            ang
        };
        (0..=samples_per_arc)
            .map(|i| {
                let t = ang * i as f64 / samples_per_arc as f64;
                add(
                    center,
                    add(scale(x, r.abs() * t.cos()), scale(y, r.abs() * t.sin())),
                )
            })
            .collect()
    };
    let mut first = arc(p0, t0, m);
    let tm = unit(sub(q1, q0));
    let second = arc(m, tm, p1);
    first.pop();
    first.extend(second);
    first
}

/// Evaluate a curve [`Geo`] at normalized `t` (re-export for menus).
pub fn point(g: &Geo, t: f64) -> Option<Vec3> {
    eval_geo(g, t)
}

/// Evaluate an exact NURBS at its own parameter.
pub fn nurbs_at(degree: usize, points: &[Vec3], weights: &[f64], knots: &[f64], u: f64) -> Vec3 {
    eval(degree, points, weights, knots, u)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bilinear() -> Geo {
        Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![
                [0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 2.0, 2.0],
            ],
            weights: Vec::new(),
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }
    }

    #[test]
    fn surfaces_evaluate_and_give_isocurves() {
        let g = bilinear();
        let s = Srf::of(&g).unwrap();
        assert_eq!(s.at_st(1.0, 1.0), [4.0, 2.0, 2.0]);
        assert_eq!(s.at_st(0.5, 0.5), [2.0, 1.0, 0.5]);
        let iso = s.isocurve(0, 1.0);
        let Geo::Nurbs { points, .. } = &iso else {
            panic!()
        };
        assert_eq!(points, &vec![[4.0, 0.0, 0.0], [4.0, 2.0, 2.0]]);
        let (u, v) = s.closest_st([2.0, 1.0, 0.6]);
        assert!(dist(s.at_st(u, v), [2.0, 1.0, 0.5]) < 0.1);
        assert!(s.normal_st(0.1, 0.1)[2] > 0.5);
    }

    #[test]
    fn blends_and_biarcs_meet_their_ends() {
        let b = blend(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [10.0, 5.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0],
            false,
        );
        assert_eq!(eval_geo(&b, 1.0).unwrap(), [10.0, 5.0, 0.0]);
        let t0 = end_tangent(&b, false).unwrap();
        assert!(t0[0] > 0.999);
        let pts = biarc(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 1.0, 0.0],
            16,
        );
        assert!(dist(*pts.last().unwrap(), [10.0, 10.0, 0.0]) < 1e-9);
        // A quarter circle is its own biarc: every sample at radius 10 from (0, 10).
        for p in &pts {
            assert!((dist(*p, [0.0, 10.0, 0.0]) - 10.0).abs() < 1e-6, "{p:?}");
        }
    }
}
