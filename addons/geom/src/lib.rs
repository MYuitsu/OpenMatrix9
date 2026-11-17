// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve math shared by OM9 add-ons (millimetres, world coordinates).
//!
//! - global cubic interpolation through points (chord-length parameters, averaged knots);
//! - clamped control-point curves of any degree;
//! - exact rational quadratic NURBS for circles, ellipses and composite line/arc outlines
//!   (rounded rectangles);
//! - circles and arcs through three points, arcs from start/end/tangent;
//! - de Boor evaluation, used by previews and tests.
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, dot, norm, scale, sub, unit};

pub mod audit;
pub mod blend;
pub mod cmd;
pub mod csg;
pub mod exact;
pub mod fit;
pub mod gems;
pub mod interp;
pub mod join;
pub mod memo;
pub mod mesh;
pub mod nurbs;
pub mod path;
pub mod pick;
pub mod sdf;
pub mod section;
pub mod shapes;
pub mod subd;
pub mod blendsrf;
pub mod extend;
pub mod fillet;
pub mod matchsrf;
pub mod merge;
pub mod network;
pub mod offset;
pub mod patch;
pub mod surfedit;
pub mod sweep;
pub mod trimsides;

pub const EPS: f64 = 1e-9;

/// Clamped uniform knots for `n` poles of `degree` (`n + degree + 1` values).
pub fn clamped_uniform(n: usize, degree: usize) -> Vec<f64> {
    let spans = n - degree;
    let mut k = vec![0.0; degree + 1];
    k.extend((1..spans).map(|i| i as f64));
    k.extend(std::iter::repeat_n(spans as f64, degree + 1));
    k
}

/// A control-point curve through `poles` (degree lowered to `poles - 1` if needed).
pub fn control_curve(poles: &[Vec3], degree: usize) -> Option<Geo> {
    if poles.len() < 2 {
        return None;
    }
    let degree = degree.clamp(1, poles.len() - 1);
    Some(Geo::Nurbs {
        degree,
        points: poles.to_vec(),
        weights: Vec::new(),
        knots: clamped_uniform(poles.len(), degree),
    })
}

pub fn find_span(n: usize, p: usize, u: f64, k: &[f64]) -> usize {
    if u >= k[n] {
        return n - 1;
    }
    let (mut lo, mut hi) = (p, n);
    let mut mid = (lo + hi) / 2;
    while u < k[mid] || u >= k[mid + 1] {
        if u < k[mid] {
            hi = mid;
        } else {
            lo = mid;
        }
        mid = (lo + hi) / 2;
    }
    mid
}

pub fn basis(span: usize, u: f64, p: usize, k: &[f64]) -> Vec<f64> {
    let mut n = vec![0.0; p + 1];
    let mut left = vec![0.0; p + 1];
    let mut right = vec![0.0; p + 1];
    n[0] = 1.0;
    for j in 1..=p {
        left[j] = u - k[span + 1 - j];
        right[j] = k[span + j] - u;
        let mut saved = 0.0;
        for r in 0..j {
            let d = right[r + 1] + left[j - r];
            let t = if d.abs() < 1e-300 { 0.0 } else { n[r] / d };
            n[r] = saved + right[r + 1] * t;
            saved = left[j - r] * t;
        }
        n[j] = saved;
    }
    n
}

/// Evaluate a (rational) NURBS curve at `u`.
pub fn eval(degree: usize, points: &[Vec3], weights: &[f64], knots: &[f64], u: f64) -> Vec3 {
    let n = points.len();
    let span = find_span(n, degree, u, knots);
    let b = basis(span, u, degree, knots);
    let mut acc = [0.0; 4];
    for (i, bi) in b.iter().enumerate() {
        let idx = span - degree + i;
        let w = weights.get(idx).copied().unwrap_or(1.0);
        for c in 0..3 {
            acc[c] += bi * w * points[idx][c];
        }
        acc[3] += bi * w;
    }
    [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]]
}

/// Evaluate any curve [`Geo`] at `t ∈ [0, 1]` (points and meshes return `None`).
pub fn eval_geo(g: &Geo, t: f64) -> Option<Vec3> {
    let t = t.clamp(0.0, 1.0);
    match g {
        Geo::Line { start, end } => Some(add(*start, scale(sub(*end, *start), t))),
        Geo::Polyline { points, closed } => {
            let mut pts = points.clone();
            if *closed {
                pts.push(points[0]);
            }
            let segs = pts.len().checked_sub(1)?;
            let x = t * segs as f64;
            let i = (x.floor() as usize).min(segs - 1);
            Some(add(pts[i], scale(sub(pts[i + 1], pts[i]), x - i as f64)))
        }
        Geo::Circle {
            center,
            x,
            y,
            radius,
        } => {
            let a = t * std::f64::consts::TAU;
            Some(add(
                *center,
                add(scale(*x, radius * a.cos()), scale(*y, radius * a.sin())),
            ))
        }
        Geo::Arc {
            center,
            x,
            y,
            radius,
            angle,
        } => {
            let a = t * angle;
            Some(add(
                *center,
                add(scale(*x, radius * a.cos()), scale(*y, radius * a.sin())),
            ))
        }
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } => {
            let (a, b) = (knots[*degree], knots[points.len()]);
            Some(eval(*degree, points, weights, knots, a + (b - a) * t))
        }
        _ => None,
    }
}

/// Solve a banded system: row `i` holds its non-zeros `rows[i].1` from column `rows[i].0`.
/// Gaussian elimination without pivoting, which is stable for B-spline collocation
/// matrices (totally positive, de Boor); refuses a vanishing pivot.
pub(crate) fn solve_band(rows: &[(usize, Vec<f64>)], n: usize, mut b: Vec<Vec3>) -> Option<Vec<Vec3>> {
    // Band limits: kl below, ku above the diagonal.
    let mut kl = 0usize;
    let mut ku = 0usize;
    for (i, (lo, vals)) in rows.iter().enumerate() {
        kl = kl.max(i.saturating_sub(*lo));
        ku = ku.max((lo + vals.len()).saturating_sub(i + 1));
    }
    let w = kl + ku + 1;
    // band[i][j - i + kl] = a[i][j]
    let mut band = vec![vec![0.0; w]; n];
    for (i, (lo, vals)) in rows.iter().enumerate() {
        for (k, v) in vals.iter().enumerate() {
            band[i][lo + k + kl - i] = *v;
        }
    }
    for col in 0..n {
        let piv = band[col][kl];
        if !piv.is_finite() || piv.abs() < 1e-14 {
            return None;
        }
        for row in col + 1..(col + kl + 1).min(n) {
            let f = band[row][col + kl - row] / piv;
            if f == 0.0 {
                continue;
            }
            for j in col..(col + ku + 1).min(n) {
                let v = band[col][j + kl - col];
                band[row][j + kl - row] -= f * v;
            }
            b[row] = sub(b[row], scale(b[col], f));
        }
    }
    let mut x = vec![[0.0; 3]; n];
    for row in (0..n).rev() {
        let mut acc = b[row];
        for k in row + 1..(row + ku + 1).min(n) {
            acc = sub(acc, scale(x[k], band[row][k + kl - row]));
        }
        x[row] = scale(acc, 1.0 / band[row][kl]);
    }
    Some(x)
}

/// A degree-`p` curve through every point (global interpolation, chord-length parameters).
/// `closed` repeats the first point at the end. Duplicate consecutive points are dropped.
pub fn interpolate(points: &[Vec3], degree: usize, closed: bool) -> Option<Geo> {
    let mut q: Vec<Vec3> = Vec::new();
    for p in points {
        if q.last().is_none_or(|l| dist(*l, *p) > EPS) {
            q.push(*p);
        }
    }
    if closed && q.len() > 2 && dist(q[0], *q.last()?) > EPS {
        q.push(q[0]);
    }
    if q.len() < 2 {
        return None;
    }
    let n = q.len();
    let p = degree.clamp(1, n - 1);
    if p == 1 {
        return Some(Geo::Polyline {
            points: q,
            closed: false,
        });
    }
    let total: f64 = q.windows(2).map(|w| dist(w[0], w[1])).sum();
    let mut u = vec![0.0; n];
    for i in 1..n {
        u[i] = u[i - 1] + dist(q[i - 1], q[i]) / total;
    }
    u[n - 1] = 1.0;
    let mut knots = vec![0.0; p + 1];
    for j in 1..n - p {
        knots.push((j..j + p).map(|i| u[i]).sum::<f64>() / p as f64);
    }
    knots.extend(std::iter::repeat_n(1.0, p + 1));
    // The collocation matrix is banded (each row: p + 1 basis values): solve it as a band,
    // O(n·p²) time and memory, so long curves (helixes of many turns) stay fast.
    let mut rows: Vec<(usize, Vec<f64>)> = Vec::with_capacity(n);
    for &ui in &u {
        let span = find_span(n, p, ui, &knots);
        rows.push((span - p, basis(span, ui, p, &knots)));
    }
    let poles = solve_band(&rows, n, q)?;
    Some(Geo::Nurbs {
        degree: p,
        points: poles,
        weights: Vec::new(),
        knots,
    })
}

/// One piece of a composite outline.
#[derive(Debug, Clone, Copy)]
pub enum Piece {
    Line(Vec3, Vec3),
    /// Arc of at most 90° from `start` to `end` around `center`.
    Arc {
        center: Vec3,
        start: Vec3,
        end: Vec3,
    },
    /// Conic from `start` to `end` with middle pole `apex` and weight `weight`
    /// (`rho / (1 − rho)`: 1 is a parabola, below 1 an ellipse arc, above 1 a hyperbola).
    Conic {
        start: Vec3,
        apex: Vec3,
        end: Vec3,
        weight: f64,
    },
}

/// An exact rational quadratic NURBS through the pieces, in order (each ends where the next
/// starts). Lines get a middle pole of weight 1, arcs ≤ 90° the classic `cos(θ/2)` weight.
pub fn composite_quadratic(pieces: &[Piece]) -> Option<Geo> {
    if pieces.is_empty() {
        return None;
    }
    let mut points = Vec::new();
    let mut weights = Vec::new();
    for (i, piece) in pieces.iter().enumerate() {
        let (a, mid, w, b) = match *piece {
            Piece::Line(a, b) => (a, scale(add(a, b), 0.5), 1.0, b),
            Piece::Arc { center, start, end } => {
                let (u, v) = (sub(start, center), sub(end, center));
                let r = norm(u);
                let cos = (dot(u, v) / (r * norm(v))).clamp(-1.0, 1.0);
                let half = cos.acos() / 2.0;
                if !(half > 0.0 && half <= std::f64::consts::FRAC_PI_4 + 1e-9) {
                    return None;
                }
                let bis = unit(add(u, v));
                let mid = add(center, scale(bis, r / half.cos()));
                (start, mid, half.cos(), end)
            }
            Piece::Conic {
                start,
                apex,
                end,
                weight,
            } => {
                if !(weight > 0.0 && weight.is_finite()) {
                    return None;
                }
                (start, apex, weight, end)
            }
        };
        if i == 0 {
            points.push(a);
            weights.push(1.0);
        }
        points.push(mid);
        weights.push(w);
        points.push(b);
        weights.push(1.0);
    }
    let segs = pieces.len();
    let mut knots = vec![0.0; 3];
    for s in 1..segs {
        knots.extend([s as f64, s as f64]);
    }
    knots.extend([segs as f64; 3]);
    Some(Geo::Nurbs {
        degree: 2,
        points,
        weights,
        knots,
    })
}

/// Exact ellipse: center, the two semi-axis vectors (perpendicular).
pub fn ellipse(center: Vec3, a: Vec3, b: Vec3) -> Option<Geo> {
    if norm(a) < EPS || norm(b) < EPS {
        return None;
    }
    let p = |sa: f64, sb: f64| add(center, add(scale(a, sa), scale(b, sb)));
    let w = std::f64::consts::FRAC_1_SQRT_2;
    let points = vec![
        p(1.0, 0.0),
        p(1.0, 1.0),
        p(0.0, 1.0),
        p(-1.0, 1.0),
        p(-1.0, 0.0),
        p(-1.0, -1.0),
        p(0.0, -1.0),
        p(1.0, -1.0),
        p(1.0, 0.0),
    ];
    Some(Geo::Nurbs {
        degree: 2,
        points,
        weights: vec![1.0, w, 1.0, w, 1.0, w, 1.0, w, 1.0],
        knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0],
    })
}

/// Rounded rectangle in the plane (corner `o`, edge vectors `u`, `v`, corner radius `r`).
pub fn rounded_rectangle(o: Vec3, u: Vec3, v: Vec3, r: f64) -> Option<Geo> {
    rounded_rectangle_with(o, u, v, r, None)
}

/// Rounded rectangle whose corners are circular arcs (`rho = None`) or conics of shape
/// `rho` ∈ (0, 1) through the same tangent points (Rhino `Corner=Conic`; 0.5 is a parabola).
pub fn rounded_rectangle_with(o: Vec3, u: Vec3, v: Vec3, r: f64, rho: Option<f64>) -> Option<Geo> {
    if let Some(rho) = rho
        && !(rho > 0.0 && rho < 1.0)
    {
        return None;
    }
    let (lu, lv) = (norm(u), norm(v));
    if r <= EPS || 2.0 * r > lu.min(lv) + EPS {
        return None;
    }
    let (eu, ev) = (unit(u), unit(v));
    let c = [o, add(o, u), add(add(o, u), v), add(o, v)];
    // Walk the corners counter-clockwise: o → +u → +u+v → +v.
    let dirs = [
        (eu, ev),
        (ev, scale(eu, -1.0)),
        (scale(eu, -1.0), scale(ev, -1.0)),
        (scale(ev, -1.0), eu),
    ];
    let mut pieces = Vec::new();
    for i in 0..4 {
        let (along, _) = dirs[i];
        let (next_along, _) = dirs[(i + 1) % 4];
        let start_line = add(c[i], scale(along, r));
        let end_line = sub(c[(i + 1) % 4], scale(along, r));
        if dist(start_line, end_line) > EPS {
            pieces.push(Piece::Line(start_line, end_line));
        }
        let corner = c[(i + 1) % 4];
        let center = add(sub(corner, scale(along, r)), scale(next_along, r));
        let end = add(corner, scale(next_along, r));
        pieces.push(match rho {
            None => Piece::Arc {
                center,
                start: end_line,
                end,
            },
            Some(rho) => Piece::Conic {
                start: end_line,
                apex: corner,
                end,
                weight: rho / (1.0 - rho),
            },
        });
    }
    composite_quadratic(&pieces)
}

/// Center of the circle through three points (`None` when collinear).
pub fn circumcenter(a: Vec3, b: Vec3, c: Vec3) -> Option<Vec3> {
    let (ab, ac) = (sub(b, a), sub(c, a));
    let n = cross(ab, ac);
    let nn = dot(n, n);
    if nn < 1e-18 {
        return None;
    }
    let term = add(
        scale(cross(n, ab), dot(ac, ac)),
        scale(cross(ac, n), dot(ab, ab)),
    );
    Some(add(a, scale(term, 1.0 / (2.0 * nn))))
}

/// Arc from `start` with tangent `dir` to `end`.
pub fn arc_start_end_dir(start: Vec3, end: Vec3, dir: Vec3) -> Option<Geo> {
    let t = unit(dir);
    let c = sub(end, start);
    if norm(c) < EPS || norm(t) < EPS {
        return None;
    }
    // Plane of the arc: tangent and chord. Perpendicular to the tangent, towards the end.
    let n = unit(cross(t, c));
    if norm(n) < 0.5 {
        return Some(Geo::Line { start, end }); // chord along the tangent: a straight segment
    }
    let m = cross(n, t);
    let s = dot(c, c) / (2.0 * dot(c, m));
    let center = add(start, scale(m, s));
    let radius = s.abs();
    let x = unit(sub(start, center));
    let theta = 2.0 * (dot(t, unit(c))).clamp(-1.0, 1.0).acos();
    Some(Geo::Arc {
        center,
        x,
        y: t,
        radius,
        angle: theta,
    })
}

/// Arc through three points in order (start, a point on it, end).
pub fn arc_3_points(a: Vec3, b: Vec3, c: Vec3) -> Option<Geo> {
    let center = circumcenter(a, b, c)?;
    let r = dist(center, a);
    let n = unit(cross(sub(b, a), sub(c, a)));
    let x = unit(sub(a, center));
    let y = cross(n, x);
    let ang = |p: Vec3| {
        let d = sub(p, center);
        let t = dot(d, y).atan2(dot(d, x));
        if t < 0.0 {
            t + std::f64::consts::TAU
        } else {
            t
        }
    };
    (ang(b) < ang(c)).then(|| Geo::Arc {
        center,
        x,
        y,
        radius: r,
        angle: ang(c),
    })
}

/// Points of a helix/spiral around the axis from `a` to `b` (`b == a` → flat spiral in the
/// plane with normal `flat_normal`), radius changing linearly from `r0` to `r1`.
#[allow(clippy::too_many_arguments)]
pub fn spiral_points(
    a: Vec3,
    b: Vec3,
    flat_normal: Vec3,
    start_dir: Vec3,
    r0: f64,
    r1: f64,
    turns: f64,
    per_turn: usize,
) -> Vec<Vec3> {
    let axis = sub(b, a);
    let n = if norm(axis) > EPS {
        unit(axis)
    } else {
        unit(flat_normal)
    };
    let mut x = sub(start_dir, scale(n, dot(start_dir, n)));
    if norm(x) < EPS {
        x = if n[0].abs() < 0.9 {
            cross(n, [1.0, 0.0, 0.0])
        } else {
            cross(n, [0.0, 1.0, 0.0])
        };
    }
    let x = unit(x);
    let y = cross(n, x);
    let count = ((turns * per_turn as f64).ceil() as usize).max(4);
    (0..=count)
        .map(|i| {
            let f = i as f64 / count as f64;
            let ang = f * turns * std::f64::consts::TAU;
            let r = r0 + (r1 - r0) * f;
            add(
                add(a, scale(axis, f)),
                add(scale(x, r * ang.cos()), scale(y, r * ang.sin())),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3, tol: f64) -> bool {
        dist(a, b) < tol
    }

    #[test]
    fn interpolation_passes_through_every_point() {
        let pts = [
            [0.0, 0.0, 0.0],
            [5.0, 3.0, 0.0],
            [9.0, -1.0, 0.0],
            [14.0, 2.0, 1.0],
            [20.0, 0.0, 0.0],
        ];
        let Some(Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        }) = interpolate(&pts, 3, false)
        else {
            panic!()
        };
        assert_eq!(degree, 3);
        assert_eq!(knots.len(), points.len() + degree + 1);
        // Recover the chord-length parameters and check every point lies on the curve.
        let total: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
        let mut u = 0.0;
        for (i, p) in pts.iter().enumerate() {
            if i > 0 {
                u += dist(pts[i - 1], *p) / total;
            }
            let q = eval(degree, &points, &weights, &knots, u.min(1.0));
            assert!(close(q, *p, 1e-9), "{i}: {q:?} vs {p:?}");
        }
        let Some(Geo::Nurbs { points: cp, .. }) = interpolate(&pts[..3], 3, true) else {
            panic!()
        };
        assert_eq!(cp.first(), Some(&pts[0]));
        assert!(
            close(*cp.last().unwrap(), pts[0], 1e-12),
            "closed ends at the start"
        );
        assert!(matches!(
            interpolate(&pts[..2], 3, false),
            Some(Geo::Polyline { .. })
        ));
    }

    #[test]
    fn conics_are_exact() {
        let Some(Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        }) = ellipse([1.0, 2.0, 0.0], [4.0, 0.0, 0.0], [0.0, 2.0, 0.0])
        else {
            panic!()
        };
        for i in 0..=64 {
            let u = 4.0 * i as f64 / 64.0;
            let p = eval(degree, &points, &weights, &knots, u);
            let (x, y) = ((p[0] - 1.0) / 4.0, (p[1] - 2.0) / 2.0);
            assert!((x * x + y * y - 1.0).abs() < 1e-12, "{p:?}");
        }
        // Rounded 10×6 rectangle, r = 1: every corner point is 1 mm from its arc center.
        let Some(g) = rounded_rectangle([0.0; 3], [10.0, 0.0, 0.0], [0.0, 6.0, 0.0], 1.0) else {
            panic!()
        };
        let mut max_x: f64 = 0.0;
        for i in 0..=400 {
            let p = eval_geo(&g, i as f64 / 400.0).unwrap();
            max_x = max_x.max(p[0]);
            assert!(p[0] > -1e-9 && p[0] < 10.0 + 1e-9 && p[1] > -1e-9 && p[1] < 6.0 + 1e-9);
            if p[0] > 9.0 && p[1] > 5.0 {
                assert!((dist(p, [9.0, 5.0, 0.0]) - 1.0).abs() < 1e-9, "{p:?}");
            }
        }
        assert!((max_x - 10.0).abs() < 1e-9);
        assert!(
            close(
                eval_geo(&g, 0.0).unwrap(),
                eval_geo(&g, 1.0).unwrap(),
                1e-12
            ),
            "closed"
        );
        assert!(rounded_rectangle([0.0; 3], [10.0, 0.0, 0.0], [0.0, 6.0, 0.0], 3.5).is_none());
    }

    #[test]
    fn arcs_and_circles_from_points() {
        let c = circumcenter([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]).unwrap();
        assert!(close(c, [0.0; 3], 1e-12));
        assert!(circumcenter([0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]).is_none());
        // Start (2,0) heading +y, end (0,2): quarter arc around the origin.
        let Some(Geo::Arc {
            center,
            radius,
            angle,
            ..
        }) = arc_start_end_dir([2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 1.0, 0.0])
        else {
            panic!()
        };
        assert!(close(center, [0.0; 3], 1e-12));
        assert!(
            (radius - 2.0).abs() < 1e-12 && (angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12
        );
        let a = arc_start_end_dir([2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 1.0, 0.0]).unwrap();
        assert!(close(eval_geo(&a, 1.0).unwrap(), [0.0, 2.0, 0.0], 1e-12));
        // Three points on the unit circle, counter-clockwise half.
        let h = arc_3_points([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]).unwrap();
        assert!(close(eval_geo(&h, 0.5).unwrap(), [0.0, 1.0, 0.0], 1e-12));
    }

    #[test]
    fn helix_climbs_its_axis() {
        let pts = spiral_points(
            [0.0; 3],
            [0.0, 0.0, 10.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            2.0,
            2.0,
            5.0,
            16,
        );
        assert!(close(pts[0], [2.0, 0.0, 0.0], 1e-12));
        assert!(close(*pts.last().unwrap(), [2.0, 0.0, 10.0], 1e-9));
        assert!(
            pts.iter()
                .all(|p| ((p[0] * p[0] + p[1] * p[1]).sqrt() - 2.0).abs() < 1e-9)
        );
        let flat = spiral_points(
            [0.0; 3],
            [0.0; 3],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            1.0,
            5.0,
            2.0,
            16,
        );
        assert!(flat.iter().all(|p| p[2].abs() < 1e-12));
        assert!(close(*flat.last().unwrap(), [5.0, 0.0, 0.0], 1e-9));
        let k = clamped_uniform(5, 3);
        assert_eq!(k, vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0]);
    }
}
