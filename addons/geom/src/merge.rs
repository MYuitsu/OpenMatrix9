// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 MergeSrf: two untrimmed surfaces that share a whole edge become one surface.
//!
//! The surfaces are turned so the shared edge is the first one's u = 1 and the second one's
//! u = 0, made compatible along the edge (same degree and knots — shape unchanged) and of
//! the same degree across it, then their poles are joined with a knot of multiplicity p at
//! the seam: exact. *Smooth* then lines up the poles either side of the seam (scaled by
//! *Roundness*, 0 sharp … 1 smooth) for a tangent join, which changes the shape near it, as
//! Rhino says.
#![allow(clippy::needless_range_loop)]
use crate::exact::{Nc, euclid};
#[cfg(test)]
use crate::exact::H;
use crate::network::{Srf, conform_v};
use om9_addon_abi::session::{Geo, Vec3, add, dist, norm, scale, sub, unit};

pub(crate) fn of_geo(g: &Geo) -> Option<Srf> {
    let Geo::Surface { degree, size, points, weights, knots } = g else { return None };
    let w = |k: usize| weights.get(k).copied().unwrap_or(1.0);
    let rows = (0..size[0])
        .map(|i| {
            (0..size[1])
                .map(|j| {
                    let k = i * size[1] + j;
                    let p = points[k];
                    [p[0] * w(k), p[1] * w(k), p[2] * w(k), w(k)]
                })
                .collect()
        })
        .collect();
    let norm01 = |k: &[f64]| {
        let (a, b) = (k[0], *k.last().unwrap());
        k.iter().map(|v| (v - a) / (b - a)).collect::<Vec<f64>>()
    };
    Some(Srf { rows, degree: *degree, knots: [norm01(&knots[0]), norm01(&knots[1])] })
}

pub(crate) fn to_geo(s: &Srf) -> Geo {
    crate::exact::surface(&s.rows, s.degree, s.knots.clone())
}

/// Reverse along u (rows) or v (within rows).
pub(crate) fn reverse(s: &Srf, dir: usize) -> Srf {
    let flip = |k: &[f64]| k.iter().rev().map(|v| 1.0 - v).collect::<Vec<f64>>();
    let mut out = s.clone();
    if dir == 0 {
        out.rows.reverse();
        out.knots[0] = flip(&s.knots[0]);
    } else {
        out.rows.iter_mut().for_each(|r| r.reverse());
        out.knots[1] = flip(&s.knots[1]);
    }
    out
}

/// The edge curve points of `s` at u = 0 or 1 (`at_end`) in direction `dir`, sampled.
fn edge(s: &Srf, dir: usize, at_end: bool, n: usize) -> Vec<Vec3> {
    let t = if dir == 0 { s.clone() } else { s.flipped() };
    let row = if at_end { t.row(t.rows.len() - 1) } else { t.row(0) };
    (0..=n).map(|k| row.eval(k as f64 / n as f64)).collect()
}

/// Rhino 5 MergeSrf.
pub fn merge(g1: &Geo, g2: &Geo, smooth: bool, roundness: f64, tol: f64) -> Result<Geo, String> {
    let (a0, b0) = (of_geo(g1).ok_or("not a NURBS surface")?, of_geo(g2).ok_or("not a NURBS surface")?);
    // Find the shared edge: (direction, end) on each, and whether they run the same way.
    let n = 12;
    let mut found = None;
    'outer: for (da, ea) in [(0, false), (0, true), (1, false), (1, true)] {
        let pa = edge(&a0, da, ea, n);
        for (db, eb) in [(0, false), (0, true), (1, false), (1, true)] {
            let pb = edge(&b0, db, eb, n);
            let same = pa.iter().zip(&pb).all(|(p, q)| dist(*p, *q) <= tol);
            let opposite = pa.iter().zip(pb.iter().rev()).all(|(p, q)| dist(*p, *q) <= tol);
            if same || opposite {
                found = Some((da, ea, db, eb, opposite));
                break 'outer;
            }
        }
    }
    let (da, ea, db, eb, opposite) = found.ok_or("the surfaces do not share a whole edge (within the tolerance)")?;
    // u across the seam: A ends there, B starts there; v along the seam, the same way.
    let mut a = if da == 0 { a0 } else { a0.flipped() };
    if !ea {
        a = reverse(&a, 0);
    }
    let mut b = if db == 0 { b0 } else { b0.flipped() };
    if eb {
        b = reverse(&b, 0);
    }
    if opposite {
        b = reverse(&b, 1);
    }
    // Same degree and knots along the seam.
    let mut both = [a, b];
    conform_v(&mut both);
    let [mut a, mut b] = both;
    // Same degree across it.
    let p = a.degree[0].max(b.degree[0]);
    for s in [&mut a, &mut b] {
        if s.degree[0] < p {
            let by = p - s.degree[0];
            let t = s.flipped();
            let cols: Vec<Nc> = (0..t.rows.len()).map(|j| t.row(j).elevate(by)).collect();
            let elevated = Srf {
                rows: cols.iter().map(|c| c.points.clone()).collect(),
                degree: [t.degree[0], p],
                knots: [t.knots[0].clone(), cols[0].knots.clone()],
            };
            *s = elevated.flipped();
        }
    }
    // Seam weights must agree (the same rational edge).
    let (la, lb) = (a.rows.len(), b.rows.len());
    for j in 0..a.rows[0].len() {
        let (x, y) = (a.rows[la - 1][j], b.rows[0][j]);
        if dist(euclid(x), euclid(y)) > tol {
            return Err("the surfaces do not share a whole edge (within the tolerance)".into());
        }
        if (x[3] - y[3]).abs() > 1e-9 * x[3].abs().max(1.0) {
            return Err("the shared edge has different weights on the two surfaces".into());
        }
    }
    // Knots across: A on [0, f], B on [f, 1], f by their lengths across.
    let len = |s: &Srf| {
        let t = s.flipped();
        let mid = t.rows.len() / 2;
        let c = t.row(mid);
        (0..64).map(|k| dist(c.eval(k as f64 / 64.0), c.eval((k + 1) as f64 / 64.0))).sum::<f64>()
    };
    let (l1, l2) = (len(&a), len(&b));
    let f = (l1 / (l1 + l2).max(1e-300)).clamp(0.05, 0.95);
    let ka: Vec<f64> = a.knots[0].iter().map(|v| v * f).collect();
    let kb: Vec<f64> = b.knots[0].iter().map(|v| f + v * (1.0 - f)).collect();
    let mut knots_u = ka[..ka.len() - 1].to_vec();
    knots_u.extend_from_slice(&kb[p + 1..]);
    let mut rows = a.rows.clone();
    rows.extend_from_slice(&b.rows[1..]);
    // Smooth: line up the poles either side of the seam (tangent join), by `roundness`.
    if smooth && la >= 2 && lb >= 2 {
        let r = roundness.clamp(0.0, 1.0);
        let seam = la - 1;
        for j in 0..rows[0].len() {
            let (pm, pj, pp) = (euclid(rows[seam - 1][j]), euclid(rows[seam][j]), euclid(rows[seam + 1][j]));
            let d = sub(pp, pm);
            if norm(d) < 1e-12 {
                continue;
            }
            let d = unit(d);
            let (lm, lp) = (dist(pm, pj), dist(pp, pj));
            let (am, ap) = (sub(pj, scale(d, lm)), add(pj, scale(d, lp)));
            for (row, target) in [(seam - 1, am), (seam + 1, ap)] {
                let old = euclid(rows[row][j]);
                let q = add(old, scale(sub(target, old), r));
                let w = rows[row][j][3];
                rows[row][j] = [q[0] * w, q[1] * w, q[2] * w, w];
            }
        }
    }
    Ok(to_geo(&Srf { rows, degree: [p, a.degree[1]], knots: [knots_u, a.knots[1].clone()] }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact;
    use crate::nurbs::Srf as Eval;

    #[test]
    fn two_halves_of_a_cylinder_merge_exactly() {
        // Two half cylinders (r 3) sharing their straight edges, the second picked the other
        // way round: Smooth=No gives one surface, exactly the same cylinder.
        let half = |start: f64| {
            let arc = exact::arc([0.0; 3], [start.cos(), start.sin(), 0.0], [-start.sin(), start.cos(), 0.0], 3.0, std::f64::consts::PI);
            let rows: Vec<Vec<H>> = arc.points.iter().map(|c| vec![*c, [c[0], c[1], 4.0 * c[3], c[3]]]).collect();
            exact::surface(&rows, [2, 1], [arc.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]])
        };
        let a = half(0.0);
        let b = half(std::f64::consts::PI);
        let m = merge(&a, &b, false, 1.0, 0.001).unwrap();
        let s = Eval::of(&m).unwrap();
        for i in 0..=60 {
            for j in 0..=4 {
                let p = s.at_st(i as f64 / 60.0, j as f64 / 4.0);
                assert!((p[0].hypot(p[1]) - 3.0).abs() < 1e-9, "{p:?}");
            }
        }
        // Covers both halves: points on both sides of the x axis.
        let ys: Vec<f64> = (0..=10).map(|i| s.at_st(i as f64 / 10.0, 0.5)[1]).collect();
        assert!(ys.iter().any(|y| *y > 2.8) && ys.iter().any(|y| *y < -2.8), "{ys:?}");
        // Surfaces apart: refused.
        let far = match half(std::f64::consts::PI) {
            Geo::Surface { degree, size, points, weights, knots } => Geo::Surface {
                degree,
                size,
                points: points.iter().map(|p| [p[0] + 1.0, p[1], p[2]]).collect(),
                weights,
                knots,
            },
            g => g,
        };
        assert!(merge(&a, &far, false, 1.0, 0.001).is_err());
    }

    #[test]
    fn smooth_merge_of_a_kinked_pair_is_tangent_at_the_seam() {
        // Two flat bicubic patches meeting at an angle along x = 0.
        let patch = |sign: f64| {
            let rows: Vec<Vec<H>> = (0..4)
                .map(|i| (0..4).map(|j| [sign * i as f64, j as f64, i as f64 * 0.5, 1.0]).collect())
                .collect();
            {
                let k = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
                exact::surface(&rows, [3, 3], [k.clone(), k])
            }
        };
        let (a, b) = (patch(1.0), patch(-1.0));
        let m = merge(&a, &b, true, 1.0, 0.001).unwrap();
        let s = Eval::of(&m).unwrap();
        // Find the seam: it is where x = 0; the slope either side must match.
        let mut seam = 0.0;
        for i in 0..=1000 {
            let t = i as f64 / 1000.0;
            if s.at_st(t, 0.5)[0].abs() < s.at_st(seam, 0.5)[0].abs() {
                seam = t;
            }
        }
        let (l, r) = (s.at_st(seam - 1e-4, 0.5), s.at_st(seam + 1e-4, 0.5));
        let c = s.at_st(seam, 0.5);
        let d1 = unit(sub(c, l));
        let d2 = unit(sub(r, c));
        assert!(dist(d1, d2) < 1e-2, "tangent at the seam: {d1:?} {d2:?}");
    }
}
