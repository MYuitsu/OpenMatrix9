// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve blends: Bézier blends with Position … G4 continuity at each end (Blend, BlendCrv,
//! Match), exact biarcs (ArcBlend), and the arc-length derivatives they match.
use crate::pick::CurveRef;
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, dot, norm, scale, sub, unit};

/// Continuity at a blend end, as Rhino names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Continuity {
    Position = 0,
    Tangency = 1,
    Curvature = 2,
    G3 = 3,
    G4 = 4,
}

impl Continuity {
    pub const NAMES: [&'static str; 5] = ["Position", "Tangency", "Curvature", "G3", "G4"];
    pub fn parse(s: &str) -> Continuity {
        match s {
            "Position" => Continuity::Position,
            "Tangency" => Continuity::Tangency,
            "G3" => Continuity::G3,
            "G4" => Continuity::G4,
            _ => Continuity::Curvature,
        }
    }
    pub fn order(self) -> usize {
        self as usize
    }
}

/// One end of a curve: the point and the arc-length derivatives `[D1, D2, D3, D4]` taken in
/// the direction leaving the curve there (D1 is the unit outward tangent, D2 the curvature
/// vector …).
#[derive(Debug, Clone, Copy)]
pub struct End {
    pub point: Vec3,
    pub d: [Vec3; 4],
}

/// The end of `c` at `t` (0 or 1, or any parameter: `outward` says which way leaves).
pub fn end_of(c: &CurveRef, t: f64, outward: bool) -> End {
    // Points stepping back into the curve, their (negative) arc-length positions, then
    // derivatives of the interpolating polynomial at 0 (Newton divided differences).
    let delta = 2e-3;
    // Step into the curve; the derivatives come out in the opposite (outward) direction.
    let back = if t >= 0.5 { -1.0 } else { 1.0 };
    let pts: Vec<Vec3> = (0..6)
        .map(|k| c.at((t + back * delta * k as f64).clamp(0.0, 1.0)))
        .collect();
    let mut s = vec![0.0];
    for k in 1..pts.len() {
        s.push(s[k - 1] - dist(pts[k], pts[k - 1]));
    }
    let n = pts.len();
    // Divided-difference table per coordinate.
    let mut table: Vec<Vec<Vec3>> = vec![pts.clone()];
    for level in 1..n {
        let prev = &table[level - 1];
        let row: Vec<Vec3> = (0..n - level)
            .map(|i| {
                let ds = s[i + level] - s[i];
                if ds.abs() < 1e-300 {
                    [0.0; 3]
                } else {
                    scale(sub(prev[i + 1], prev[i]), 1.0 / ds)
                }
            })
            .collect();
        table.push(row);
    }
    // Newton form p(x) = Σ c_k Π_{i<k} (x − s_i); derivatives at x = 0 by expanding.
    let coef: Vec<Vec3> = (0..n).map(|k| table[k][0]).collect();
    // Polynomial coefficients in powers of x.
    let mut poly: Vec<Vec3> = vec![[0.0; 3]; n];
    let mut basis: Vec<f64> = vec![1.0]; // Π (x − s_i), ascending powers
    for (k, ck) in coef.iter().enumerate() {
        for (j, b) in basis.iter().enumerate() {
            poly[j] = add(poly[j], scale(*ck, *b));
        }
        if k + 1 < n {
            let mut next = vec![0.0; basis.len() + 1];
            for (j, b) in basis.iter().enumerate() {
                next[j + 1] += b;
                next[j] -= s[k] * b;
            }
            basis = next;
        }
    }
    let fact = [1.0, 1.0, 2.0, 6.0, 24.0];
    let d = std::array::from_fn(|j| scale(poly[j + 1], fact[j + 1]));
    let mut end = End { point: pts[0], d };
    if !outward {
        end.d[0] = scale(end.d[0], -1.0);
        end.d[2] = scale(end.d[2], -1.0);
    }
    // Arc length makes |D1| = 1 up to rounding; normalise it exactly.
    let l = norm(end.d[0]);
    if l > 1e-12 {
        end.d[0] = scale(end.d[0], 1.0 / l);
    }
    end
}

fn binom(n: usize, k: usize) -> f64 {
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// The first `k + 1` Bézier poles (degree `deg`) that give `C^(j)(0) = α^j D_j`.
fn poles_from(end: &End, k: usize, deg: usize, alpha: f64) -> Vec<Vec3> {
    let mut p = vec![end.point];
    for j in 1..=k {
        let falling: f64 = (0..j).map(|i| (deg - i) as f64).product();
        let mut q = scale(end.d[j - 1], alpha.powi(j as i32) / falling);
        for (i, pi) in p.iter().enumerate() {
            let sign = if (j - i) % 2 == 0 { 1.0 } else { -1.0 };
            q = sub(q, scale(*pi, sign * binom(j, i)));
        }
        p.push(q);
    }
    p
}

/// A Bézier blend leaving `a` and arriving at `b` (both `End`s point *away* from their
/// curves, into the blend), with continuity `ca`, `cb` and bulges (1 = default shape).
pub fn bezier_blend(a: &End, ca: Continuity, b: &End, cb: Continuity, bulge: [f64; 2]) -> Geo {
    let (ka, kb) = (ca.order(), cb.order());
    let deg = (ka + kb + 1).max(1);
    let chord = dist(a.point, b.point).max(1e-9);
    // α = bulge · chord: the first pole sits chord/degree from the end (a third for cubics).
    let pa = poles_from(a, ka, deg, bulge[0] * chord);
    let pb = poles_from(b, kb, deg, bulge[1] * chord);
    let mut points = pa;
    let mut tail = pb;
    tail.reverse();
    points.extend(tail);
    Geo::Nurbs {
        degree: deg,
        points,
        weights: Vec::new(),
        knots: [vec![0.0; deg + 1], vec![1.0; deg + 1]].concat(),
    }
}

/// The arc leaving `p` along the unit tangent `t` and passing through `q`.
pub fn arc_from(p: Vec3, t: Vec3, q: Vec3) -> Option<Geo> {
    let v = sub(q, p);
    let perp = sub(v, scale(t, dot(v, t)));
    if norm(perp) < 1e-12 * norm(v).max(1.0) {
        return (norm(v) > 1e-12 && dot(v, t) > 0.0).then_some(Geo::Line { start: p, end: q });
    }
    let nn = scale(perp, 1.0 / norm(perp));
    let r = dot(v, v) / (2.0 * dot(v, nn));
    let center = add(p, scale(nn, r));
    let x = scale(sub(p, center), 1.0 / r);
    let u = scale(sub(q, center), 1.0 / r);
    let mut angle = dot(u, t).atan2(dot(u, x));
    if angle <= 0.0 {
        angle += std::f64::consts::TAU;
    }
    Some(Geo::Arc {
        center,
        x,
        y: t,
        radius: r,
        angle,
    })
}

/// Radius of an arc piece (lines: infinite).
fn radius(g: &Geo) -> f64 {
    match g {
        Geo::Arc { radius, .. } => *radius,
        _ => f64::INFINITY,
    }
}

/// A biarc from `p0` leaving along `t0` to `p1` arriving along `t1` (unit tangents in the
/// direction of travel). `ratio` sets the tangent lengths `d0 = ratio·d1` (1: equal);
/// `alternate` takes the other (longer) solution. Returns the two arcs.
pub fn biarc(
    p0: Vec3,
    t0: Vec3,
    p1: Vec3,
    t1: Vec3,
    ratio: f64,
    alternate: bool,
) -> Option<[Geo; 2]> {
    let v = sub(p1, p0);
    // |v − d0·t0 − d1·t1| = d0 + d1 with d0 = k·d1 → quadratic in d1.
    let k = ratio.max(1e-6);
    let w = add(scale(t0, k), t1);
    let a = dot(w, w) - (k + 1.0) * (k + 1.0);
    let b = -2.0 * dot(v, w);
    let c = dot(v, v);
    let roots: Vec<f64> = if a.abs() < 1e-12 {
        if b.abs() < 1e-300 {
            return None;
        }
        vec![-c / b]
    } else {
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }
        let sq = disc.sqrt();
        vec![(-b + sq) / (2.0 * a), (-b - sq) / (2.0 * a)]
    };
    let mut good: Vec<f64> = roots
        .into_iter()
        .filter(|d| d.is_finite() && *d > 1e-12)
        .collect();
    good.sort_by(f64::total_cmp);
    let mut d1 = *good.first()?;
    if alternate {
        match good.get(1) {
            Some(&d) => d1 = d,
            None => {
                // One positive root: the other solution runs the tangents backwards.
                let w2 = sub(scale(t0, -k), t1);
                let a2 = dot(w2, w2) - (k + 1.0) * (k + 1.0);
                let b2 = -2.0 * dot(v, w2);
                let disc = b2 * b2 - 4.0 * a2 * c;
                if disc >= 0.0 && a2.abs() > 1e-12 {
                    let r = [
                        (-b2 + disc.sqrt()) / (2.0 * a2),
                        (-b2 - disc.sqrt()) / (2.0 * a2),
                    ];
                    if let Some(&d) = r.iter().find(|d| **d > 1e-12) {
                        let q0 = sub(p0, scale(t0, k * d));
                        let q1 = add(p1, scale(t1, d));
                        let j = add(q0, scale(sub(q1, q0), k / (k + 1.0)));
                        return Some([
                            arc_from(p0, t0, j)?,
                            arc_from(p1, scale(t1, -1.0), j)
                                .and_then(|g| crate::join::reverse(&g))?,
                        ]);
                    }
                }
            }
        }
    }
    let q0 = add(p0, scale(t0, k * d1));
    let q1 = sub(p1, scale(t1, d1));
    // The junction divides the middle leg in the ratio of the tangent lengths.
    let j = add(q0, scale(sub(q1, q0), k / (k + 1.0)));
    let first = arc_from(p0, t0, j)?;
    let second = arc_from(p1, scale(t1, -1.0), j).and_then(|g| crate::join::reverse(&g))?;
    Some([first, second])
}

/// A biarc whose radii differ by `difference` (first − second), by searching the ratio.
pub fn biarc_with_difference(
    p0: Vec3,
    t0: Vec3,
    p1: Vec3,
    t1: Vec3,
    difference: f64,
    alternate: bool,
) -> Option<[Geo; 2]> {
    if difference.abs() < 1e-12 {
        return biarc(p0, t0, p1, t1, 1.0, alternate);
    }
    let f = |lnk: f64| -> Option<f64> {
        let [a, b] = biarc(p0, t0, p1, t1, lnk.exp(), alternate)?;
        let (ra, rb) = (radius(&a), radius(&b));
        (ra.is_finite() && rb.is_finite()).then_some(ra - rb - difference)
    };
    // Scan the log-ratio for a sign change, then bisect.
    let grid: Vec<f64> = (-60..=60).map(|i| i as f64 * 0.1).collect();
    let vals: Vec<Option<f64>> = grid.iter().map(|&x| f(x)).collect();
    for i in 0..grid.len() - 1 {
        if let (Some(a), Some(b)) = (vals[i], vals[i + 1])
            && a * b <= 0.0
        {
            let (mut lo, mut hi, mut flo) = (grid[i], grid[i + 1], a);
            for _ in 0..80 {
                let mid = (lo + hi) / 2.0;
                let fm = f(mid)?;
                if (fm < 0.0) == (flo < 0.0) {
                    lo = mid;
                    flo = fm;
                } else {
                    hi = mid;
                }
            }
            return biarc(p0, t0, p1, t1, ((lo + hi) / 2.0).exp(), alternate);
        }
    }
    None
}

/// Unit normal of the plane through `a` with tangents (for planar checks); `None` if
/// the tangents are parallel.
pub fn plane_of(t0: Vec3, t1: Vec3) -> Option<Vec3> {
    let n = cross(t0, t1);
    let l = norm(n);
    (l > 1e-9).then(|| scale(n, 1.0 / l))
}

// ---- circles touching two curves (Fillet, Arc Tangent) --------------------------------------

/// Unit normal of the plane the two curves lie in near the picks (the CPlane normal when
/// they are parallel there).
pub fn plane_normal(a: &CurveRef, sa: f64, b: &CurveRef, sb: f64, fallback: Vec3) -> Vec3 {
    let n = cross(a.tangent(sa), b.tangent(sb));
    if norm(n) > 1e-6 {
        let n = unit(n);
        if dot(n, fallback) < 0.0 {
            scale(n, -1.0)
        } else {
            n
        }
    } else {
        fallback
    }
}

/// Two lines (point + direction): their intersection (closest point on the first).
fn line_cross(p: Vec3, a: Vec3, q: Vec3, b: Vec3) -> Option<Vec3> {
    let n = cross(a, b);
    let nn = dot(n, n);
    if nn < 1e-18 {
        return None;
    }
    let t = dot(cross(sub(q, p), b), n) / nn;
    Some(add(p, scale(a, t)))
}

/// The circle of radius `r` touching both curves on the sides of the picks: (tangent
/// parameter on a, on b, center). `r = 0` gives the corner itself.
pub fn tangent_circle(
    a: &CurveRef,
    pa: f64,
    b: &CurveRef,
    pb: f64,
    n: Vec3,
    r: f64,
) -> Option<(f64, f64, Vec3)> {
    let side = |c: &CurveRef, t: f64| cross(n, c.tangent(t));
    let (qa, qb) = (a.at(pa), b.at(pb));
    let sa_sign = if dot(side(a, pa), sub(qb, qa)) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let sb_sign = if dot(side(b, pb), sub(qa, qb)) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let (mut sa, mut sb) = (pa, pb);
    let mut center: Option<Vec3> = None;
    for _ in 0..200 {
        let oa = add(a.at(sa), scale(side(a, sa), r * sa_sign));
        let ob = add(b.at(sb), scale(side(b, sb), r * sb_sign));
        let c = line_cross(oa, a.tangent(sa), ob, b.tangent(sb))?;
        let moved = center.is_none_or(|old| dist(old, c) > 1e-11);
        center = Some(c);
        let (na, nb) = (a.closest(c).0, b.closest(c).0);
        if !moved && (na - sa).abs() < 1e-12 && (nb - sb).abs() < 1e-12 {
            break;
        }
        sa = na;
        sb = nb;
    }
    let c = center?;
    let scale_len = dist(a.at(sa), b.at(sb)).max(r).max(1.0);
    let ok = |cv: &CurveRef, s: f64| (dist(cv.at(s), c) - r).abs() < 1e-6 * scale_len;
    (ok(a, sa) && ok(b, sb)).then_some((sa, sb, c))
}

/// The arc from `p` to `q` around `c` (the short way).
pub fn arc_between(c: Vec3, p: Vec3, q: Vec3) -> Option<Geo> {
    let r = dist(p, c);
    let x = unit(sub(p, c));
    let w = sub(q, c);
    let y = sub(w, scale(x, dot(w, x)));
    if norm(y) < 1e-12 || r < 1e-12 {
        return None;
    }
    let y = unit(y);
    let angle = dot(unit(w), y).atan2(dot(unit(w), x));
    (angle > 1e-9).then_some(Geo::Arc {
        center: c,
        x,
        y,
        radius: r,
        angle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval_geo;
    use om9_addon_abi::session::Selected;

    fn cref(g: Geo) -> CurveRef {
        let pts = crate::nurbs::samples(&g, 64);
        CurveRef::of(&Selected {
            id: "c".into(),
            kind: "curve".into(),
            points: pts,
            exact: Some(g),
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn end_derivatives_of_a_circle() {
        let arc = cref(Geo::Arc {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            angle: 1.0,
        });
        let e = end_of(&arc, 1.0, true);
        let p = [2.0 * 1f64.cos(), 2.0 * 1f64.sin(), 0.0];
        assert!(dist(e.point, p) < 1e-12);
        let t = [-1f64.sin(), 1f64.cos(), 0.0];
        assert!(dist(e.d[0], t) < 1e-6, "{:?}", e.d[0]);
        // Curvature vector: towards the center, length 1/2.
        assert!(dist(e.d[1], scale(p, -0.25)) < 1e-4, "{:?}", e.d[1]);
        // Leaving the start backwards.
        let s = end_of(&arc, 0.0, true);
        assert!(dist(s.d[0], [0.0, -1.0, 0.0]) < 1e-6);
        assert!(dist(s.d[1], [-0.5, 0.0, 0.0]) < 1e-4, "{:?}", s.d[1]);
    }

    #[test]
    fn blends_match_tangent_and_curvature() {
        let arc = cref(Geo::Arc {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            angle: 1.0,
        });
        let line = cref(Geo::Line {
            start: [6.0, 5.0, 0.0],
            end: [10.0, 5.0, 0.0],
        });
        let a = end_of(&arc, 1.0, true);
        let b = end_of(&line, 0.0, true);
        for (ca, cb) in [
            (Continuity::Tangency, Continuity::Tangency),
            (Continuity::Curvature, Continuity::Curvature),
            (Continuity::G3, Continuity::Position),
        ] {
            let g = bezier_blend(&a, ca, &b, cb, [1.0, 1.0]);
            let bc = cref(g.clone());
            assert!(dist(eval_geo(&g, 0.0).unwrap(), a.point) < 1e-12);
            assert!(dist(eval_geo(&g, 1.0).unwrap(), b.point) < 1e-12);
            let s = end_of(&bc, 0.0, false);
            if ca >= Continuity::Tangency {
                assert!(dist(s.d[0], a.d[0]) < 1e-5, "{ca:?} {:?}", s.d[0]);
            }
            if ca >= Continuity::Curvature {
                assert!(
                    dist(s.d[1], a.d[1]) < 1e-3,
                    "{ca:?} {:?} {:?}",
                    s.d[1],
                    a.d[1]
                );
            }
            let e = end_of(&bc, 1.0, false);
            if cb >= Continuity::Tangency {
                assert!(dist(e.d[0], b.d[0]) < 1e-5, "{cb:?}");
            }
        }
    }

    #[test]
    fn biarcs_are_tangent_and_meet() {
        let (p0, t0) = ([0.0; 3], [1.0, 0.0, 0.0]);
        let (p1, t1) = ([4.0, 3.0, 0.0], [0.0, 1.0, 0.0]);
        for alt in [false, true] {
            for ratio in [1.0, 0.5, 2.0] {
                let [a, b] = biarc(p0, t0, p1, t1, ratio, alt).unwrap();
                let (ca, cb) = (cref(a.clone()), cref(b.clone()));
                assert!(dist(ca.at(0.0), p0) < 1e-9 && dist(cb.at(1.0), p1) < 1e-9);
                assert!(dist(ca.at(1.0), cb.at(0.0)) < 1e-9, "{ratio} {alt}");
                assert!(dist(ca.tangent(0.0), t0) < 1e-4);
                assert!(dist(cb.tangent(1.0), t1) < 1e-4, "{:?}", cb.tangent(1.0));
                assert!(dist(ca.tangent(1.0), cb.tangent(0.0)) < 1e-4);
            }
        }
        let [a, b] = biarc_with_difference(p0, t0, p1, t1, 1.0, false).unwrap();
        assert!(
            (radius(&a) - radius(&b) - 1.0).abs() < 1e-6,
            "{} {}",
            radius(&a),
            radius(&b)
        );
    }
}
