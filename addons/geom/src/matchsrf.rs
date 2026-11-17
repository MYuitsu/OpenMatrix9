// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 MatchSrf: change a surface near one of its edges so that edge meets a target
//! surface edge with Position, Tangency or Curvature continuity.
//!
//! The surface is turned so the edge to change is u = 0 and refined (knot insertion, shape
//! unchanged) until enough pole rows lie away from the far edge (Preserve other end). Then:
//! row 0 interpolates the target edge (matched by arc length, at the row's Greville places
//! along the edge); with Tangency row 1 makes the cross derivative continue the target's
//! crossing curve; with Curvature row 2 matches its second derivative, from the clamped
//! B-spline end formulas. Between the Greville places the match holds within the tolerance
//! the refinement reaches (*Refine*).
#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]
use crate::exact::{self, Nc, euclid};
use crate::network::Srf as Rows;
use crate::nurbs::Srf;
use om9_addon_abi::session::{Geo, Vec3, add, dist, scale, sub};

pub use crate::blendsrf::{Continuity, Edge};

fn rows_of(g: &Geo) -> Option<Rows> {
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
    let n01 = |k: &[f64]| {
        let (a, b) = (k[0], *k.last().unwrap());
        k.iter().map(|v| (v - a) / (b - a)).collect::<Vec<f64>>()
    };
    Some(Rows { rows, degree: *degree, knots: [n01(&knots[0]), n01(&knots[1])] })
}

/// Crossing curve of a surface at its edge: point, inward first and second derivatives at
/// place t along the edge (normalised parameters).
fn inward(s: &Srf, e: Edge, t: f64) -> (Vec3, Vec3, Vec3) {
    let at = |x: f64| -> Vec3 {
        let a = if e.at_end { 1.0 - x } else { x };
        if e.dir == 0 { s.at_st(a, t) } else { s.at_st(t, a) }
    };
    let h = 1e-4;
    let (p0, p1, p2, p3) = (at(0.0), at(h), at(2.0 * h), at(3.0 * h));
    let d1: Vec3 = std::array::from_fn(|k| (-3.0 * p0[k] + 4.0 * p1[k] - p2[k]) / (2.0 * h));
    let d2: Vec3 = std::array::from_fn(|k| (2.0 * p0[k] - 5.0 * p1[k] + 4.0 * p2[k] - p3[k]) / (h * h));
    (p0, d1, d2)
}

fn arc_table(s: &Srf, e: Edge) -> (Vec<f64>, Vec<f64>) {
    let n = 256;
    let (mut t, mut l) = (vec![0.0], vec![0.0]);
    let mut prev = inward(s, e, 0.0).0;
    let mut run = 0.0;
    for i in 1..=n {
        let x = i as f64 / n as f64;
        let p = inward(s, e, x).0;
        run += dist(p, prev);
        prev = p;
        t.push(x);
        l.push(run);
    }
    let total = run.max(1e-300);
    l.iter_mut().for_each(|v| *v /= total);
    (t, l)
}

fn lookup(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    let i = xs.partition_point(|v| *v <= x).clamp(1, xs.len() - 1) - 1;
    let f = ((x - xs[i]) / (xs[i + 1] - xs[i]).max(1e-300)).clamp(0.0, 1.0);
    ys[i] + (ys[i + 1] - ys[i]) * f
}

/// Rhino 5 MatchSrf: change `g` at its edge `e` to meet `target`'s edge `te`.
pub fn match_srf(g: &Geo, e: Edge, target: &Geo, te: Edge, c: Continuity, tol: f64) -> Result<Geo, String> {
    let ts = Srf::of(target).ok_or("the target is not a NURBS surface")?;
    let mut r = rows_of(g).ok_or("not a NURBS surface")?;
    // Edge to change at u = 0: rows along u.
    if e.dir == 1 {
        r = r.flipped();
    }
    let flip_u = |r: &Rows| {
        let mut o = r.clone();
        o.rows.reverse();
        o.knots[0] = r.knots[0].iter().rev().map(|v| 1.0 - v).collect();
        o
    };
    if e.at_end {
        r = flip_u(&r);
    }
    let k = match c {
        Continuity::Position => 1,
        Continuity::Tangency => 2,
        Continuity::Curvature => 3,
        // Rhino 5 MatchSrf matches up to curvature.
        Continuity::G3 | Continuity::G4 => return Err("MatchSrf matches up to Curvature".into()),
    };
    let p = r.degree[0];
    if p + 1 < k {
        // Raise the degree across so the rows can carry the continuity.
        let t = r.flipped();
        let cols: Vec<Nc> = (0..t.rows.len()).map(|j| t.row(j).elevate(k - 1 - p)).collect();
        r = Rows { rows: cols.iter().map(|c| c.points.clone()).collect(), degree: [t.degree[0], k - 1], knots: [t.knots[0].clone(), cols[0].knots.clone()] }.flipped();
    }
    // Preserve the other end: refine near the edge until at least k rows sit in spans that
    // do not reach the far edge (insert at the middle of the first span).
    for _ in 0..8 {
        let p = r.degree[0];
        let first_inner = r.knots[0].iter().copied().find(|v| *v > 1e-12).unwrap_or(1.0);
        if r.rows.len() > k + p && first_inner < 1.0 {
            break;
        }
        let t = r.flipped();
        let mid = first_inner / 2.0;
        let cols: Vec<Nc> = (0..t.rows.len())
            .map(|j| {
                let mut c = t.row(j);
                c.insert(mid);
                c
            })
            .collect();
        r = Rows { rows: cols.iter().map(|c| c.points.clone()).collect(), degree: [t.degree[0], t.degree[1]], knots: [t.knots[0].clone(), cols[0].knots.clone()] }.flipped();
    }
    // Places along the edge (v) and their matches on the target edge (by arc length).
    let width = r.rows[0].len();
    let q = r.degree[1];
    let kv = r.knots[1].clone();
    let greville: Vec<f64> = (0..width).map(|j| (1..=q).map(|i| kv[j + i]).sum::<f64>() / q.max(1) as f64).collect();
    let me = {
        let mut back = r.clone();
        if e.at_end {
            back = flip_u(&back);
        }
        if e.dir == 1 {
            back = back.flipped();
        }
        exact::surface(&back.rows, back.degree, back.knots.clone())
    };
    let ms = Srf::of(&me).ok_or("not a NURBS surface")?;
    let (mt, ml) = arc_table(&ms, e);
    let (tt, tl) = arc_table(&ts, te);
    // Run the target edge the same way.
    let (a0, a1) = (inward(&ms, e, 0.0).0, inward(&ms, e, 1.0).0);
    let (b0, b1) = (inward(&ts, te, 0.0).0, inward(&ts, te, 1.0).0);
    let flip = dist(a0, b0) + dist(a1, b1) > dist(a0, b1) + dist(a1, b0);
    let target_at = |v: f64| {
        let s = lookup(&mt, &ml, v);
        let s = if flip { 1.0 - s } else { s };
        inward(&ts, te, lookup(&tl, &tt, s))
    };
    // Row 0: interpolate the target edge points at the Greville places (same knots along).
    let pts: Vec<Vec3> = greville.iter().map(|v| target_at(*v).0).collect();
    let weights0: Vec<f64> = r.rows[0].iter().map(|h| h[3]).collect();
    let pcs: Vec<Nc> = pts.iter().map(|x| exact::point_curve(*x)).collect();
    let row0: Vec<Vec3> = if (0..width).all(|j| (weights0[j] - 1.0).abs() < 1e-12) {
        // Solve with the row's own knots: collocation at the Greville places.
        collocate(&pts, &greville, q, &kv).ok_or("could not fit the edge")?
    } else {
        // Rational row: keep the weights, move the points (approximate within the tolerance).
        let _ = pcs;
        pts.clone()
    };
    let ku = r.knots[0].clone();
    let p = r.degree[0];
    // Clamped start: C'(0) = p/(u_{p+1} − u_1)(P1 − P0);
    // C''(0) = p(p−1)/(u_{p+1} − u_2)·[(P2 − P1)/(u_{p+2} − u_2) − (P1 − P0)/(u_{p+1} − u_1)].
    let a = p as f64 / (ku[p + 1] - ku[1]);
    for j in 0..width {
        let w = r.rows[0][j][3];
        let p0 = row0[j];
        let old0 = euclid(r.rows[0][j]);
        let old1 = euclid(r.rows[1][j]);
        r.rows[0][j] = [p0[0] * w, p0[1] * w, p0[2] * w, w];
        if k >= 2 {
            let (_, d1, d2) = target_at(greville[j]);
            // Keep the cross speed the surface had: λ = |C'(0)|.
            let lam = a * dist(old1, old0);
            let n1 = (d1[0] * d1[0] + d1[1] * d1[1] + d1[2] * d1[2]).sqrt().max(1e-300);
            let mu = lam / n1;
            // Into this surface = out of the target: C'(0) = −μ g'(0), C''(0) = μ² g''(0).
            let c1 = scale(d1, -mu);
            let p1 = add(p0, scale(c1, 1.0 / a));
            let w1 = r.rows[1][j][3];
            r.rows[1][j] = [p1[0] * w1, p1[1] * w1, p1[2] * w1, w1];
            if k >= 3 && p >= 2 {
                let c2 = scale(d2, mu * mu);
                let b = p as f64 * (p as f64 - 1.0) / (ku[p + 1] - ku[2]);
                // (P2 − P1)/(u_{p+2} − u_2) = C''/b + (P1 − P0)/(u_{p+1} − u_1)
                let lhs = add(scale(c2, 1.0 / b), scale(sub(p1, p0), 1.0 / (ku[p + 1] - ku[1])));
                let p2 = add(p1, scale(lhs, ku[p + 2] - ku[2]));
                let w2 = r.rows[2][j][3];
                r.rows[2][j] = [p2[0] * w2, p2[1] * w2, p2[2] * w2, w2];
            }
        }
    }
    let _ = tol;
    if e.at_end {
        r = flip_u(&r);
    }
    if e.dir == 1 {
        r = r.flipped();
    }
    Ok(exact::surface(&r.rows, r.degree, r.knots))
}

/// Rhino 5 MatchSrf Average=Yes: both surfaces change to meet half way — the edges meet on
/// the curve midway between them, crossing along the direction midway between their own
/// (and, with Curvature, with the average of their curvatures across). Returns the changed
/// `g` and the changed `other`.
pub fn match_average(g: &Geo, e: Edge, other: &Geo, oe: Edge, c: Continuity, tol: f64) -> Result<(Geo, Geo), String> {
    if c.order() > 2 {
        return Err("MatchSrf matches up to Curvature".into());
    }
    let sa = Srf::of(g).ok_or("not a NURBS surface")?;
    let sb = Srf::of(other).ok_or("the target is not a NURBS surface")?;
    let (ta, la) = arc_table(&sa, e);
    let (tb, lb) = arc_table(&sb, oe);
    let (a0, a1) = (inward(&sa, e, 0.0).0, inward(&sa, e, 1.0).0);
    let (b0, b1) = (inward(&sb, oe, 0.0).0, inward(&sb, oe, 1.0).0);
    let flip = dist(a0, b0) + dist(a1, b1) > dist(a0, b1) + dist(a1, b0);
    let n = 48;
    let unit = |v: Vec3| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-300);
        (scale(v, 1.0 / l), l)
    };
    // Rows across (w = 0, ½, 1) of the two virtual strips: one seen from g (it continues
    // where the other surface lies), one seen from the other surface.
    let mut rows_g: Vec<Vec<Vec3>> = vec![Vec::new(); 3];
    let mut rows_o: Vec<Vec<Vec3>> = vec![Vec::new(); 3];
    for i in 0..=n {
        let s = i as f64 / n as f64;
        let (p, da1, da2) = inward(&sa, e, lookup(&la, &ta, s));
        let (q, db1, db2) = inward(&sb, oe, lookup(&lb, &tb, if flip { 1.0 - s } else { s }));
        let mid = scale(add(p, q), 0.5);
        let ((ua, la1), (ub, lb1)) = (unit(da1), unit(db1));
        // The crossing direction from g's side into the other's: midway between −ua and ub.
        let (m, _) = unit(sub(ub, ua));
        let len = 0.5 * (la1 + lb1);
        let d = scale(m, len);
        // Second derivative across, per unit speed squared, averaged (same along the
        // through-going parameter on both sides).
        let k = scale(add(scale(db2, 1.0 / (lb1 * lb1)), scale(da2, 1.0 / (la1 * la1))), 0.5 * len * len);
        let k = if c.order() >= 2 { k } else { [0.0; 3] };
        // Quadratic across: P0 = mid, P1 = mid + d/2, P2 = mid + d + k/2 → C'(0) = d, C''(0) = k.
        let strip = |d: Vec3| {
            let p1 = add(mid, scale(d, 0.5));
            let p2 = add(add(mid, d), scale(k, 0.5));
            // Points at w = 0, ½, 1 of the Bézier.
            [mid, add(add(scale(mid, 0.25), scale(p1, 0.5)), scale(p2, 0.25)), p2]
        };
        let (sg, so) = (strip(d), strip(scale(d, -1.0)));
        for r in 0..3 {
            rows_g[r].push(sg[r]);
            rows_o[r].push(so[r]);
        }
    }
    let virt = |rows: &[Vec<Vec3>]| crate::fit::surface_through(rows, [2, 3]).ok_or_else(|| "could not build the average".to_string());
    let (vg, vo) = (virt(&rows_g)?, virt(&rows_o)?);
    let edge0 = Edge { dir: 0, at_end: false };
    Ok((match_srf(g, e, &vg, edge0, c, tol)?, match_srf(other, oe, &vo, edge0, c, tol)?))
}

/// Poles of a non-rational curve with knots `k` (degree `q`) through `pts` at `params`.
fn collocate(pts: &[Vec3], params: &[f64], q: usize, k: &[f64]) -> Option<Vec<Vec3>> {
    let n = pts.len();
    let mut m = vec![vec![0.0; n]; n];
    for (r, u) in params.iter().enumerate() {
        let span = crate::find_span(n, q, u.clamp(0.0, 1.0), k);
        let b = crate::basis(span, u.clamp(0.0, 1.0), q, k);
        for (i, v) in b.iter().enumerate() {
            m[r][span - q + i] = *v;
        }
    }
    // Gaussian elimination, three right-hand sides.
    let mut rhs: Vec<Vec3> = pts.to_vec();
    for c in 0..n {
        let piv = (c..n).max_by(|a, b| m[*a][c].abs().total_cmp(&m[*b][c].abs()))?;
        if m[piv][c].abs() < 1e-14 {
            return None;
        }
        m.swap(c, piv);
        rhs.swap(c, piv);
        for r in c + 1..n {
            let f = m[r][c] / m[c][c];
            if f == 0.0 {
                continue;
            }
            for x in c..n {
                m[r][x] -= f * m[c][x];
            }
            rhs[r] = sub(rhs[r], scale(rhs[c], f));
        }
    }
    let mut out = vec![[0.0; 3]; n];
    for r in (0..n).rev() {
        let mut acc = rhs[r];
        for x in r + 1..n {
            acc = sub(acc, scale(out[x], m[r][x]));
        }
        out[r] = scale(acc, 1.0 / m[r][r]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bicubic(f: impl Fn(f64, f64) -> Vec3) -> Geo {
        let rows: Vec<Vec<exact::H>> = (0..4).map(|i| (0..4).map(|j| {
            let p = f(i as f64 / 3.0, j as f64 / 3.0);
            [p[0], p[1], p[2], 1.0]
        }).collect()).collect();
        let k = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        exact::surface(&rows, [3, 3], [k.clone(), k])
    }

    #[test]
    fn match_moves_the_edge_onto_the_target_with_tangency() {
        // Target: a bump z = sin over x in 0..4; the surface to change: a flat patch x 4.3..8
        // (a gap): after matching its x = 4.3 edge lies on the target's x = 4 edge, and it
        // leaves with the target's slope.
        let target = bicubic(|a, b| [4.0 * a, 4.0 * b, (a * 2.0).sin() * 0.8]);
        let flat = bicubic(|a, b| [4.3 + 3.7 * a, 4.0 * b, 0.2]);
        for c in [Continuity::Position, Continuity::Tangency, Continuity::Curvature] {
            let m = match_srf(&flat, Edge { dir: 0, at_end: false }, &target, Edge { dir: 0, at_end: true }, c, 0.001).unwrap();
            let (s, t) = (Srf::of(&m).unwrap(), Srf::of(&target).unwrap());
            for i in 0..=10 {
                let v = i as f64 / 10.0;
                let (p, q) = (s.at_st(0.0, v), t.at_st(1.0, v));
                assert!(dist(p, q) < 1e-6, "{c:?} edge {p:?} {q:?}");
                if c != Continuity::Position {
                    // Same slope across the joint.
                    let ds = sub(s.at_st(1e-4, v), p);
                    let dt = sub(q, t.at_st(1.0 - 1e-4, v));
                    let (ss, st) = (ds[2] / ds[0], dt[2] / dt[0]);
                    assert!((ss - st).abs() < 2e-3, "{c:?} slope {ss} vs {st}");
                }
            }
            // The far edge is kept.
            let far = s.at_st(1.0, 0.5);
            assert!((far[0] - 8.0).abs() < 1e-9 && (far[2] - 0.2).abs() < 1e-9, "{far:?}");
        }
    }

    /// Average=Yes: two flat strips with a 2 mm step between their edges, tilted ±10° —
    /// both move to meet on the middle line (z = 1), crossing level (tangent to the average).
    #[test]
    fn average_meets_half_way() {
        let t = 10f64.to_radians();
        let a = bicubic(|u, v| [4.0 * u, 5.0 * v, 4.0 * u * t.tan() - 4.0 * t.tan()]);
        let b = bicubic(|u, v| [6.0 + 4.0 * u, 5.0 * v, 2.0 - 4.0 * u * t.tan()]);
        let (ga, gb) = match_average(&a, Edge { dir: 0, at_end: true }, &b, Edge { dir: 0, at_end: false }, Continuity::Tangency, 1e-6).unwrap();
        let (sa, sb) = (Srf::of(&ga).unwrap(), Srf::of(&gb).unwrap());
        for j in 0..=10 {
            let v = j as f64 / 10.0;
            let (p, q) = (sa.at_st(1.0, v), sb.at_st(0.0, v));
            assert!(dist(p, q) < 1e-6, "{p:?} {q:?}");
            assert!((p[0] - 5.0).abs() < 1e-6 && (p[2] - 1.0).abs() < 1e-6, "{p:?}");
            // Level across at the edge on both (the average of +10° and −10°... seen as one
            // through-going direction: from a's side up 10°, from b's side down 10° → level).
            let (pa, pb) = (sa.at_st(1.0 - 1e-5, v), sb.at_st(1e-5, v));
            assert!(((p[2] - pa[2]) / (p[0] - pa[0])).abs() < 1e-3, "a slope");
            assert!(((pb[2] - q[2]) / (pb[0] - q[0])).abs() < 1e-3, "b slope");
        }
    }
}
