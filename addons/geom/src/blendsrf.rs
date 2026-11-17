// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 BlendSrf between two surface edges, with Position / Tangency / Curvature
//! continuity at each and a bulge per side.
//!
//! The edges are matched by arc length. At each place the section is a Bézier curve
//! (quintic up to Curvature, degree 9 for G3 / G4): its ends are the edge points; with
//! Tangency its first derivative continues the isoparametric curve crossing the edge (scaled
//! by the bulge); with Curvature, G3 and G4 its second, third and fourth derivatives do too
//! (the crossing curve reparameterised by the same speed), so curvature and its rates of
//! change across the edge match. Stations are refined until the surface is within the
//! tolerance of these sections; the edges themselves are met at every station.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]
use crate::exact::H;
use crate::nurbs::Srf;
use crate::sweep::{Swept, fit_stations};
use om9_addon_abi::session::{Geo, Vec3, add, dist, scale, sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Continuity {
    Position,
    Tangency,
    Curvature,
    /// G3: the rate of change of curvature matches too.
    G3,
    /// G4: and its rate of change.
    G4,
}

impl Continuity {
    /// How many derivatives across the edge are matched.
    pub fn order(self) -> usize {
        match self {
            Continuity::Position => 0,
            Continuity::Tangency => 1,
            Continuity::Curvature => 2,
            Continuity::G3 => 3,
            Continuity::G4 => 4,
        }
    }
}

/// An edge of a surface: the parameter direction that ends there (0: u, 1: v) and the end.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub dir: usize,
    pub at_end: bool,
}

/// Point and outward derivatives (1st to 4th) of the crossing isoparametric curve at place
/// `t` (0..1) along the edge.
fn crossing_n(s: &Srf, e: Edge, t: f64) -> (Vec3, [Vec3; 4]) {
    let at = |x: f64| -> Vec3 {
        // x: distance in from the edge along the crossing parameter (0 at the edge).
        let a = if e.at_end { 1.0 - x } else { x };
        if e.dir == 0 { s.at_st(a, t) } else { s.at_st(t, a) }
    };
    // One-sided differences (second order) inward; the step grows with the order (rounding
    // error ~ ε/hᵏ). Outward = (−1)ᵏ inward.
    let diff = |h: f64, w: &[f64], div: f64| -> Vec3 {
        let pts: Vec<Vec3> = (0..w.len()).map(|i| at(i as f64 * h)).collect();
        std::array::from_fn(|k| w.iter().zip(&pts).map(|(c, p)| c * p[k]).sum::<f64>() / div)
    };
    let (h1, h3, h4) = (1e-4, 2e-3, 1e-2);
    let d1 = scale(diff(h1, &[-3.0, 4.0, -1.0], 2.0 * h1), -1.0);
    let d2 = diff(h1, &[2.0, -5.0, 4.0, -1.0], h1 * h1);
    let d3 = scale(diff(h3, &[-5.0, 18.0, -24.0, 14.0, -3.0], 2.0 * h3 * h3 * h3), -1.0);
    let d4 = diff(h4, &[3.0, -14.0, 26.0, -24.0, 11.0, -2.0], h4 * h4 * h4 * h4);
    (at(0.0), [d1, d2, d3, d4])
}

/// Point, outward first and second derivatives of the crossing isoparametric curve.
fn crossing(s: &Srf, e: Edge, t: f64) -> (Vec3, Vec3, Vec3) {
    let (p, d) = crossing_n(s, e, t);
    (p, d[0], d[1])
}

/// The first `k + 1` Bézier points from an end `p` (degree `n`) whose derivatives along the
/// section are `d[j]` (j = 1..k); the rest (up to index `upto`) along the chord to `q`.
fn end_points(n: usize, p: Vec3, q: Vec3, d: &[Vec3; 4], k: usize, upto: usize) -> Vec<Vec3> {
    let falling = |j: usize| -> f64 { (0..j).map(|i| (n - i) as f64).product() };
    let binom = |a: usize, b: usize| -> f64 { (0..b).map(|i| (a - i) as f64 / (i + 1) as f64).product() };
    let mut pts = vec![p];
    for j in 1..=upto {
        if j <= k {
            // Δʲ P0 = d_j / (n (n−1) … (n−j+1));  P_j = Δʲ − Σ_{i<j} (−1)^{j−i} C(j,i) P_i.
            let delta = scale(d[j - 1], 1.0 / falling(j));
            let mut pj = delta;
            for i in 0..j {
                let c = binom(j, i) * if (j - i) % 2 == 0 { 1.0 } else { -1.0 };
                pj = sub(pj, scale(pts[i], c));
            }
            pts.push(pj);
        } else {
            pts.push(add(p, scale(sub(q, p), j as f64 / n as f64)));
        }
    }
    pts
}

/// The edge as points along its arc length (for matching the two edges).
struct EdgeArc {
    t: Vec<f64>,
    s: Vec<f64>,
}

impl EdgeArc {
    fn of(srf: &Srf, e: Edge) -> EdgeArc {
        let n = 256;
        let (mut t, mut s) = (vec![0.0], vec![0.0]);
        let mut prev = crossing(srf, e, 0.0).0;
        let mut run = 0.0;
        for i in 1..=n {
            let x = i as f64 / n as f64;
            let p = crossing(srf, e, x).0;
            run += dist(p, prev);
            prev = p;
            t.push(x);
            s.push(run);
        }
        let total = run.max(1e-300);
        s.iter_mut().for_each(|v| *v /= total);
        EdgeArc { t, s }
    }
    fn t_of(&self, s: f64) -> f64 {
        let i = self.s.partition_point(|v| *v <= s).clamp(1, self.s.len() - 1) - 1;
        let f = ((s - self.s[i]) / (self.s[i + 1] - self.s[i]).max(1e-300)).clamp(0.0, 1.0);
        self.t[i] + (self.t[i + 1] - self.t[i]) * f
    }
}

/// Rhino 5 BlendSrf between edge `e1` of `g1` and edge `e2` of `g2`.
pub fn blend(
    g1: &Geo,
    e1: Edge,
    c1: Continuity,
    bulge1: f64,
    g2: &Geo,
    e2: Edge,
    c2: Continuity,
    bulge2: f64,
    tol: f64,
) -> Result<Swept, String> {
    let s1 = Srf::of(g1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(g2).ok_or("not a NURBS surface")?;
    // Run the second edge the way the first runs (shorter cross connections).
    let (a0, a1) = (crossing(&s1, e1, 0.0).0, crossing(&s1, e1, 1.0).0);
    let (b0, b1) = (crossing(&s2, e2, 0.0).0, crossing(&s2, e2, 1.0).0);
    let flip = dist(a0, b0) + dist(a1, b1) > dist(a0, b1) + dist(a1, b0);
    let (arc1, arc2) = (EdgeArc::of(&s1, e1), EdgeArc::of(&s2, e2));
    let high = c1.order().max(c2.order()) > 2;
    let n = if high { 9 } else { 5 };
    let half = (n - 1) / 2; // points set from each end: 0..=half (quintic: 0..=2, degree 9: 0..=4)
    let placed = |u: f64| -> Vec<H> {
        let t1 = arc1.t_of(u);
        let t2 = arc2.t_of(if flip { 1.0 - u } else { u });
        let (p, dp) = crossing_n(&s1, e1, t1);
        let (q, dq) = crossing_n(&s2, e2, t2);
        let gap = dist(p, q).max(1e-9);
        // Speed: the crossing curve sped up to λ = bulge·gap/|d1| (its length scale the gap);
        // the k-th derivative along the section is λᵏ d_k, from each end inwards.
        let norm = |v: Vec3| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-300);
        let (l1, l2) = (bulge1 * gap / norm(dp[0]), bulge2 * gap / norm(dq[0]));
        let sp: [Vec3; 4] = std::array::from_fn(|k| scale(dp[k], l1.powi(k as i32 + 1)));
        let sq: [Vec3; 4] = std::array::from_fn(|k| scale(dq[k], l2.powi(k as i32 + 1)));
        let a = end_points(n, p, q, &sp, c1.order(), half);
        let b = end_points(n, q, p, &sq, c2.order(), half);
        let mut pts = a;
        pts.extend(b.into_iter().rev());
        pts.iter().map(|x| [x[0], x[1], x[2], 1.0]).collect()
    };
    let mut knots = vec![0.0; n + 1];
    knots.extend(vec![1.0; n + 1]);
    fit_stations(&placed, 0.0, 1.0, false, Vec::new(), n, &knots, tol, 400)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::Nc;
    use om9_addon_abi::session::{cross, norm, unit};

    fn plane(o: Vec3, u: Vec3, v: Vec3) -> Geo {
        Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![o, add(o, v), add(o, u), add(add(o, u), v)],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }
    }

    fn eval(sw: &Swept, u: f64, v: f64) -> Vec3 {
        let row: Vec<H> = (0..sw.rows[0].len())
            .map(|j| Nc { degree: sw.degree[0], points: sw.rows.iter().map(|r| r[j]).collect(), knots: sw.knots[0].clone() }.eval_h(u))
            .collect();
        Nc { degree: sw.degree[1], points: row, knots: sw.knots[1].clone() }.eval(v)
    }

    #[test]
    fn blend_meets_both_edges_tangent() {
        // Two flat strips: z = 0 for x in 0..4 and z = 2 for x in 6..10 (y 0..5); blend the
        // edge x = 4 of the first to the edge x = 6 of the second.
        let a = plane([0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 5.0, 0.0]);
        let b = plane([6.0, 0.0, 2.0], [4.0, 0.0, 0.0], [0.0, 5.0, 0.0]);
        for c in [Continuity::Tangency, Continuity::Curvature] {
            let s = blend(&a, Edge { dir: 0, at_end: true }, c, 1.0, &b, Edge { dir: 0, at_end: false }, c, 1.0, 0.001).unwrap();
            for i in 0..=10 {
                let u = i as f64 / 10.0;
                let (p, q) = (eval(&s, u, 0.0), eval(&s, u, 1.0));
                assert!((p[0] - 4.0).abs() < 1e-9 && p[2].abs() < 1e-9, "{p:?}");
                assert!((q[0] - 6.0).abs() < 1e-9 && (q[2] - 2.0).abs() < 1e-9, "{q:?}");
                // Tangent: leaves flat (no z slope) at both ends.
                let (p2, q2) = (eval(&s, u, 1e-4), eval(&s, u, 1.0 - 1e-4));
                assert!(((p2[2] - p[2]) / (p2[0] - p[0])).abs() < 1e-3, "flat start");
                assert!(((q[2] - q2[2]) / (q[0] - q2[0])).abs() < 1e-3, "flat end");
            }
            let _ = (cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), norm([1.0; 3]), unit([1.0; 3]));
        }
    }

    /// G3 / G4: along each section the k-th derivative at the edge is λᵏ times the crossing
    /// curve's (k up to 3 / 4) — the surface carried on with the same speed — on curved
    /// strips z = 0.05x³ (third derivative 0.3) and its copy turned round.
    #[test]
    fn g3_and_g4_carry_the_higher_derivatives() {
        // z = 0.05 x³ over x 0..4 as an exact cubic Bézier in u; y 0..5 linear in v.
        let cubic = |x0: f64, sgn: f64, z0: f64| -> Geo {
            let f = |x: f64| z0 + sgn * 0.05 * (x - x0).powi(3);
            // Bézier points of the cubic y = f over [x0, x0+4]: power to Bernstein.
            let (a, b, c, d) = (f(x0), 0.0, 0.0, 0.05 * sgn * 64.0);
            let _ = (b, c);
            let zs = [a, a, a, a + d];
            let xs = [x0, x0 + 4.0 / 3.0, x0 + 8.0 / 3.0, x0 + 4.0];
            let mut points = Vec::new();
            for i in 0..4 {
                for y in [0.0, 5.0] {
                    points.push([xs[i], y, zs[i]]);
                }
            }
            Geo::Surface { degree: [3, 1], size: [4, 2], points, weights: vec![1.0; 8], knots: [vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]] }
        };
        let a = cubic(0.0, 1.0, 0.0);
        let b = cubic(7.0, -1.0, 6.0);
        let sa = Srf::of(&a).unwrap();
        // Check the patch is z = 0.05x³.
        let p = sa.at_st(0.5, 0.5);
        assert!((p[2] - 0.05 * p[0].powi(3)).abs() < 1e-12, "{p:?}");
        for (c, k) in [(Continuity::G3, 3usize), (Continuity::G4, 4)] {
            let sw = blend(&a, Edge { dir: 0, at_end: true }, c, 1.0, &b, Edge { dir: 0, at_end: false }, c, 1.0, 1e-6).unwrap();
            assert_eq!(sw.degree[1], 9);
            for u in [0.25, 0.5] {
                // Section derivatives at v = 0, exactly: the section is one degree-9 polynomial
                // in v, so its power coefficients from 10 samples give dʲ/dvʲ = j!·a_j.
                let m = 10;
                let vs: Vec<f64> = (0..m).map(|i| i as f64 / (m - 1) as f64).collect();
                let samples: Vec<Vec3> = vs.iter().map(|v| eval(&sw, u, *v)).collect();
                let coef = |q: usize| -> Vec<f64> {
                    // Solve the Vandermonde system by Gaussian elimination.
                    let mut a: Vec<Vec<f64>> = vs.iter().zip(&samples).map(|(v, p)| {
                        let mut row: Vec<f64> = (0..m).map(|k| v.powi(k as i32)).collect();
                        row.push(p[q]);
                        row
                    }).collect();
                    for c in 0..m {
                        let piv = (c..m).max_by(|x, y| a[*x][c].abs().total_cmp(&a[*y][c].abs())).unwrap();
                        a.swap(c, piv);
                        for r in 0..m {
                            if r != c {
                                let f = a[r][c] / a[c][c];
                                for k in c..=m {
                                    a[r][k] -= f * a[c][k];
                                }
                            }
                        }
                    }
                    (0..m).map(|c| a[c][m] / a[c][c]).collect()
                };
                let cs = [coef(0), coef(1), coef(2)];
                let dv = |j: usize| -> Vec3 {
                    let f: f64 = (1..=j + 1).map(|i| i as f64).product();
                    std::array::from_fn(|q| cs[q][j + 1] * f)
                };
                // The strip's crossing curve, outward from x = 4 in its own parameter w
                // (x = 4 + 4w): first (4, 0, 0.6x²), second (0, 0, 4.8x), third (0, 0, 19.2),
                // fourth 0; the section is C(λv): its j-th derivative is λʲ times C's.
                let x = 4.0;
                let c = [[4.0, 0.0, 0.6 * x * x], [0.0, 0.0, 4.8 * x], [0.0, 0.0, 19.2], [0.0; 3]];
                let lam = norm(dv(0)) / norm(c[0]);
                let want: Vec<Vec3> = c.iter().enumerate().map(|(j, w)| scale(*w, lam.powi(j as i32 + 1))).collect();
                for j in 0..k {
                    let got = dv(j);
                    let tolj = [1e-6, 1e-5, 1e-3, 1e-2][j] * (1.0 + norm(want[j]));
                    assert!(dist(got, want[j]) < tolj, "{c:?} u {u} d{}: {got:?} vs {:?}", j + 1, want[j]);
                }
            }
        }
    }
}
