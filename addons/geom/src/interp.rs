// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve interpolation as the Matrix/Rhino `InterpCrv` command needs it:
//! knot spacing (Uniform, Chord, SqrtChord), end tangents, smooth (periodic) closing,
//! and the "deformable" NURBS approximation of circles and ellipses.
//!
//! Every result is returned as a clamped NURBS [`Geo`]: a smooth closed curve is built as a
//! periodic B-spline and then clamped by knot insertion, so it stays C^(p-1) across the seam.
use crate::{basis, find_span};
use om9_addon_abi::session::{Geo, Vec3, add, dist, norm, scale, sub, unit};

/// How parameter (knot) spacing follows the picked points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Knots {
    /// Spacing 1 whatever the distances.
    #[default]
    Uniform,
    /// Spacing = distance between points.
    Chord,
    /// Spacing = square root of the distance.
    SqrtChord,
}

impl Knots {
    pub fn name(self) -> &'static str {
        match self {
            Knots::Uniform => "Uniform",
            Knots::Chord => "Chord",
            Knots::SqrtChord => "SqrtChord",
        }
    }
    /// `Uniform`, `Chord`, `SqrtChord` (also Rhino's `SqrtChrd`), without case.
    pub fn parse(s: &str) -> Option<Knots> {
        match s.to_ascii_lowercase().as_str() {
            "uniform" | "u" => Some(Knots::Uniform),
            "chord" | "c" => Some(Knots::Chord),
            "sqrtchord" | "sqrtchrd" | "sqrt" | "s" => Some(Knots::SqrtChord),
            _ => None,
        }
    }
    fn step(self, a: Vec3, b: Vec3) -> f64 {
        match self {
            Knots::Uniform => 1.0,
            Knots::Chord => dist(a, b),
            Knots::SqrtChord => dist(a, b).sqrt(),
        }
    }
}

/// How the curve ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Closing {
    #[default]
    Open,
    /// Closed with a kink at the first point (non-periodic, Rhino `Sharp`).
    Sharp,
    /// Closed smoothly (periodic, Rhino `Close` / `PersistentClose` / AutoClose).
    Smooth,
}

/// Solve `a · x = b` with partial pivoting.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<Vec3>) -> Option<Vec<Vec3>> {
    let n = a.len();
    for col in 0..n {
        let piv = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[piv][col].abs() < 1e-13 {
            return None;
        }
        a.swap(col, piv);
        b.swap(col, piv);
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            if f == 0.0 {
                continue;
            }
            let (above, below) = a.split_at_mut(row);
            for (v, p) in below[0].iter_mut().zip(&above[col]).skip(col) {
                *v -= f * p;
            }
            b[row] = sub(b[row], scale(b[col], f));
        }
    }
    let mut x = vec![[0.0; 3]; n];
    for row in (0..n).rev() {
        let mut acc = b[row];
        for k in row + 1..n {
            acc = sub(acc, scale(x[k], a[row][k]));
        }
        x[row] = scale(acc, 1.0 / a[row][row]);
    }
    Some(x)
}

fn dedup(points: &[Vec3]) -> Vec<Vec3> {
    let mut q: Vec<Vec3> = Vec::new();
    for p in points {
        if q.last().is_none_or(|l| dist(*l, *p) > 1e-9) {
            q.push(*p);
        }
    }
    q
}

/// Cumulative parameters of `q` (first 0), plus the closing step when `closed`.
fn params(q: &[Vec3], knots: Knots, closed: bool) -> Vec<f64> {
    let mut t = vec![0.0];
    for w in q.windows(2) {
        let s = knots.step(w[0], w[1]).max(1e-12);
        t.push(t.last().unwrap() + s);
    }
    if closed {
        let s = knots.step(*q.last().unwrap(), q[0]).max(1e-12);
        t.push(t.last().unwrap() + s);
    }
    t
}

/// A curve through `points`.
///
/// - `degree` is lowered when there are too few points (as Rhino does).
/// - `start` / `end`: tangent directions at the ends (open curves only); their length is
///   ignored (scaled to the chord length of the points).
/// - `Closing::Smooth` needs three distinct points; with fewer the curve stays open.
pub fn interpolate(
    points: &[Vec3],
    degree: usize,
    knots: Knots,
    start: Option<Vec3>,
    end: Option<Vec3>,
    closing: Closing,
) -> Option<Geo> {
    let mut q = dedup(points);
    if q.len() > 2 && dist(q[0], *q.last()?) <= 1e-9 {
        q.pop();
    }
    if q.len() < 2 {
        return None;
    }
    match closing {
        Closing::Smooth if q.len() >= 3 => periodic(&q, degree, knots),
        Closing::Sharp if q.len() >= 3 => {
            q.push(q[0]);
            open(&q, degree, knots, start, end)
        }
        _ => open(&q, degree, knots, start, end),
    }
}

fn open(
    q: &[Vec3],
    degree: usize,
    knots: Knots,
    start: Option<Vec3>,
    end: Option<Vec3>,
) -> Option<Geo> {
    let n = q.len();
    let ds = usize::from(start.is_some_and(|d| norm(d) > 1e-12));
    let de = usize::from(end.is_some_and(|d| norm(d) > 1e-12));
    let m = n + ds + de; // poles
    let p = degree.clamp(1, m - 1);
    if p == 1 && ds + de == 0 {
        return Some(Geo::Polyline {
            points: q.to_vec(),
            closed: false,
        });
    }
    let mut t = params(q, knots, false);
    let total = *t.last()?;
    for v in &mut t {
        *v /= total;
    }
    // Knots by averaging over the parameters, with the end parameters repeated once per
    // end condition (one extra pole each).
    let mut ext = vec![0.0; 1 + ds];
    ext.extend_from_slice(&t[1..n - 1]);
    ext.extend(std::iter::repeat_n(1.0, 1 + de));
    let mut kv = vec![0.0; p + 1];
    for j in 1..m - p {
        kv.push(ext[j..j + p].iter().sum::<f64>() / p as f64);
    }
    kv.extend(std::iter::repeat_n(1.0, p + 1));
    let chord: f64 = q.windows(2).map(|w| dist(w[0], w[1])).sum();
    let mut a = Vec::with_capacity(m);
    let mut b = Vec::with_capacity(m);
    for (i, &u) in t.iter().enumerate() {
        let mut row = vec![0.0; m];
        let span = find_span(m, p, u, &kv);
        for (j, v) in basis(span, u, p, &kv).into_iter().enumerate() {
            row[span - p + j] = v;
        }
        a.push(row);
        b.push(q[i]);
    }
    if let Some(d) = start.filter(|_| ds == 1) {
        let c = p as f64 / kv[p + 1];
        let mut row = vec![0.0; m];
        row[0] = -c;
        row[1] = c;
        a.push(row);
        b.push(scale(unit(d), chord));
    }
    if let Some(d) = end.filter(|_| de == 1) {
        let c = p as f64 / (1.0 - kv[m - 1]);
        let mut row = vec![0.0; m];
        row[m - 2] = -c;
        row[m - 1] = c;
        a.push(row);
        b.push(scale(unit(d), chord));
    }
    let poles = solve(a, b)?;
    Some(Geo::Nurbs {
        degree: p,
        points: poles,
        weights: Vec::new(),
        knots: kv,
    })
}

/// A smooth closed curve through `q` (distinct points, the closing span included).
fn periodic(q: &[Vec3], degree: usize, knots: Knots) -> Option<Geo> {
    let n = q.len();
    let p = degree.clamp(1, n - 1);
    if p == 1 {
        return Some(Geo::Polyline {
            points: q.to_vec(),
            closed: true,
        });
    }
    let t = params(q, knots, true); // n + 1 values, t[n] = period
    let period = t[n];
    // Periodic knot positions: the parameters (odd degree) or their midpoints (even degree).
    let base = |i: i64| -> f64 {
        let k = i.rem_euclid(n as i64) as usize;
        let wraps = i.div_euclid(n as i64) as f64;
        let v = if p % 2 == 1 {
            t[k]
        } else {
            (t[k] + t[k + 1]) / 2.0
        };
        v + wraps * period
    };
    let kv: Vec<f64> = (-(p as i64)..=(n + p) as i64).map(base).collect();
    let poles_n = n + p;
    let (lo, hi) = (kv[p], kv[n + p]);
    let mut a = vec![vec![0.0; n]; n];
    for (row, &u0) in t[..n].iter().enumerate() {
        let u = if u0 < lo - 1e-12 { u0 + period } else { u0 };
        let span = find_span(poles_n, p, u.min(hi), &kv);
        for (j, v) in basis(span, u.min(hi), p, &kv).into_iter().enumerate() {
            let pole = span - p + j; // 0..n+p
            let idx = (pole as i64 - p as i64).rem_euclid(n as i64) as usize;
            a[row][idx] += v;
        }
    }
    let x = solve(a, q.to_vec())?;
    let poles: Vec<Vec3> = (0..poles_n)
        .map(|i| x[(i as i64 - p as i64).rem_euclid(n as i64) as usize])
        .collect();
    Some(clamp_range(p, kv, poles, lo, hi))
}

/// Least-squares approximation of `samples` by a B-spline with `count` poles of `degree` on
/// uniform knots (Rhino Rebuild / FitCrv). Open curves keep their end points; closed curves
/// come out smooth across the seam. Parameters follow chord length.
pub fn fit(samples: &[Vec3], count: usize, degree: usize, closed: bool) -> Option<Geo> {
    let q = dedup(samples);
    let m = q.len();
    if m < 2 || count < 2 {
        return None;
    }
    let p = degree.clamp(1, count - 1);
    // Chord-length parameters in [0, 1] (closed: around the loop, last back to the start).
    let mut t = vec![0.0];
    for w in q.windows(2) {
        t.push(t.last()? + dist(w[0], w[1]));
    }
    if closed {
        t.push(t.last()? + dist(q[m - 1], q[0]));
    }
    let total = *t.last()?;
    if total < 1e-12 {
        return None;
    }
    for v in t.iter_mut() {
        *v /= total;
    }
    let normal = |rows: &[(Vec<(usize, f64)>, Vec3)], n: usize| -> Option<Vec<Vec3>> {
        let mut a = vec![vec![0.0; n]; n];
        let mut b = vec![[0.0; 3]; n];
        for (row, rhs) in rows {
            for &(i, vi) in row {
                for &(j, vj) in row {
                    a[i][j] += vi * vj;
                }
                b[i] = add(b[i], scale(*rhs, vi));
            }
        }
        solve(a, b)
    };
    if !closed {
        let n = count;
        if n <= p {
            return None;
        }
        let spans = (n - p) as f64;
        let kv: Vec<f64> = crate::clamped_uniform(n, p)
            .into_iter()
            .map(|k| k / spans)
            .collect();
        if n == 2 {
            return Some(Geo::Line {
                start: q[0],
                end: q[m - 1],
            });
        }
        // Unknowns: poles 1..n-2; the ends are fixed.
        let (q0, qm) = (q[0], q[m - 1]);
        let mut rows = Vec::new();
        for (k, &u) in t.iter().enumerate().take(m - 1).skip(1) {
            let span = find_span(n, p, u, &kv);
            let mut rhs = q[k];
            let mut row = Vec::new();
            for (j, v) in basis(span, u, p, &kv).into_iter().enumerate() {
                let idx = span - p + j;
                if idx == 0 {
                    rhs = sub(rhs, scale(q0, v));
                } else if idx == n - 1 {
                    rhs = sub(rhs, scale(qm, v));
                } else {
                    row.push((idx - 1, v));
                }
            }
            rows.push((row, rhs));
        }
        if rows.len() < n - 2 {
            return None;
        }
        let inner = normal(&rows, n - 2)?;
        let mut points = vec![q0];
        points.extend(inner);
        points.push(qm);
        return Some(Geo::Nurbs {
            degree: p,
            points,
            weights: Vec::new(),
            knots: kv,
        });
    }
    // Closed: a periodic uniform B-spline, then clamped at the seam.
    let n = count;
    if n <= p || m < n {
        return None;
    }
    let kv: Vec<f64> = (-(p as i64)..=(n + p) as i64)
        .map(|i| i as f64 / n as f64)
        .collect();
    let poles_n = n + p;
    let (lo, hi) = (kv[p], kv[n + p]);
    let mut rows = Vec::new();
    for (k, &u) in t.iter().take(m).enumerate() {
        let span = find_span(poles_n, p, u.min(hi), &kv);
        let mut row: Vec<(usize, f64)> = Vec::new();
        for (j, v) in basis(span, u.min(hi), p, &kv).into_iter().enumerate() {
            let pole = span - p + j;
            let idx = (pole as i64 - p as i64).rem_euclid(n as i64) as usize;
            match row.iter_mut().find(|r| r.0 == idx) {
                Some(r) => r.1 += v,
                None => row.push((idx, v)),
            }
        }
        rows.push((row, q[k]));
    }
    let x = normal(&rows, n)?;
    let poles: Vec<Vec3> = (0..poles_n)
        .map(|i| x[(i as i64 - p as i64).rem_euclid(n as i64) as usize])
        .collect();
    Some(clamp_range(p, kv, poles, lo, hi))
}

/// A closed, smooth control-point curve: the poles of a periodic uniform B-spline (Rhino
/// `Curve` › `Close` with `Sharp=No`), clamped at the first pole's span.
pub fn periodic_poles(poles: &[Vec3], degree: usize) -> Option<Geo> {
    let n = poles.len();
    if n < 3 {
        return None;
    }
    let p = degree.clamp(1, n - 1);
    if p == 1 {
        return Some(Geo::Polyline { points: poles.to_vec(), closed: true });
    }
    let kv: Vec<f64> = (-(p as i64)..=(n + p) as i64).map(|i| i as f64).collect();
    let all: Vec<Vec3> = (0..n + p).map(|i| poles[i % n]).collect();
    let (lo, hi) = (kv[p], kv[n + p]);
    Some(clamp_range(p, kv, all, lo, hi))
}

/// Insert knot `u` once (Boehm). `k` is the last knot index with `knots[k] <= u`.
fn insert(p: usize, knots: &mut Vec<f64>, poles: &mut Vec<Vec3>, u: f64) {
    let n = poles.len();
    let mut k = p;
    while k + 1 < n && knots[k + 1] <= u {
        k += 1;
    }
    let mut out = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let q = if i + p <= k {
            poles[i]
        } else if i > k {
            poles[i - 1]
        } else {
            let d = knots[i + p] - knots[i];
            let a = if d.abs() < 1e-300 {
                0.0
            } else {
                (u - knots[i]) / d
            };
            add(scale(poles[i - 1], 1.0 - a), scale(poles[i], a))
        };
        out.push(q);
    }
    knots.insert(k + 1, u);
    *poles = out;
}

/// The part of an unclamped B-spline over `[lo, hi]` as a clamped NURBS.
fn clamp_range(p: usize, mut kv: Vec<f64>, mut poles: Vec<Vec3>, lo: f64, hi: f64) -> Geo {
    let mult = |kv: &[f64], u: f64| kv.iter().filter(|&&k| (k - u).abs() < 1e-12).count();
    for u in [lo, hi] {
        let have = mult(&kv, u);
        for _ in have..p {
            insert(p, &mut kv, &mut poles, u);
        }
    }
    let a = kv.iter().position(|&k| (k - lo).abs() < 1e-12).unwrap_or(p);
    let e = kv
        .iter()
        .position(|&k| (k - hi).abs() < 1e-12)
        .unwrap_or(kv.len() - p - 1);
    let points = poles[a - 1..e].to_vec();
    let mut knots = vec![lo; p + 1];
    knots.extend_from_slice(&kv[a + p..e]);
    knots.extend(std::iter::repeat_n(hi, p + 1));
    Geo::Nurbs {
        degree: p,
        points,
        weights: Vec::new(),
        knots,
    }
}

/// The "deformable" NURBS approximation of an ellipse with semi-axis vectors `a`, `b`
/// (a circle when they are perpendicular and of equal length): a smooth closed curve of
/// `degree` with `count` control points, through `count` evenly spaced points of the exact
/// shape.
pub fn deformable(center: Vec3, a: Vec3, b: Vec3, degree: usize, count: usize) -> Option<Geo> {
    if count < 3 || norm(a) < 1e-12 || norm(b) < 1e-12 {
        return None;
    }
    let pts: Vec<Vec3> = (0..count)
        .map(|i| {
            let ang = std::f64::consts::TAU * i as f64 / count as f64;
            add(center, add(scale(a, ang.cos()), scale(b, ang.sin())))
        })
        .collect();
    let p = degree.clamp(1, count - 1);
    periodic(&pts, p, Knots::Uniform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval_geo;

    fn at(g: &Geo, t: f64) -> Vec3 {
        eval_geo(g, t).unwrap()
    }
    /// Distance from `p` to the curve: dense samples, then a golden-section refinement.
    fn on_curve(g: &Geo, p: Vec3) -> f64 {
        let n = 2000;
        let best = (0..=n)
            .min_by(|&a, &b| {
                let d = |i: usize| dist(at(g, i as f64 / n as f64), p);
                d(a).total_cmp(&d(b))
            })
            .unwrap();
        let (mut lo, mut hi) = (
            (best as f64 - 1.0).max(0.0) / n as f64,
            (best as f64 + 1.0).min(n as f64) / n as f64,
        );
        for _ in 0..80 {
            let (m1, m2) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
            if dist(at(g, m1), p) < dist(at(g, m2), p) {
                hi = m2;
            } else {
                lo = m1;
            }
        }
        dist(at(g, (lo + hi) / 2.0), p)
    }
    fn tangent(g: &Geo, t: f64) -> Vec3 {
        let h = 1e-6;
        let (a, b) = if t < 0.5 { (t, t + h) } else { (t - h, t) };
        unit(sub(at(g, b), at(g, a)))
    }

    const PTS: [Vec3; 5] = [
        [0.0, 0.0, 0.0],
        [10.0, 5.0, 0.0],
        [12.0, 15.0, 0.0],
        [3.0, 18.0, 0.0],
        [-4.0, 9.0, 0.0],
    ];

    #[test]
    fn open_curves_pass_through_every_point_for_each_knot_style() {
        for knots in [Knots::Uniform, Knots::Chord, Knots::SqrtChord] {
            for degree in [1, 2, 3, 5] {
                let g = interpolate(&PTS, degree, knots, None, None, Closing::Open).unwrap();
                for p in PTS {
                    assert!(on_curve(&g, p) < 1e-3, "{knots:?} {degree}");
                }
                assert!(dist(at(&g, 0.0), PTS[0]) < 1e-9);
                assert!(dist(at(&g, 1.0), PTS[4]) < 1e-9);
            }
        }
    }

    #[test]
    fn knot_styles_differ_when_spacing_differs() {
        let pts = [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [10.0, 0.0, 0.0],
            [11.0, 1.0, 0.0],
        ];
        let u = interpolate(&pts, 3, Knots::Uniform, None, None, Closing::Open).unwrap();
        let c = interpolate(&pts, 3, Knots::Chord, None, None, Closing::Open).unwrap();
        assert!(dist(at(&u, 0.25), at(&c, 0.25)) > 1e-3);
    }

    #[test]
    fn end_tangents_are_followed() {
        let s = [0.0, 1.0, 0.0];
        let e = [1.0, 0.0, 0.0];
        let g = interpolate(&PTS, 3, Knots::Chord, Some(s), Some(e), Closing::Open).unwrap();
        for p in PTS {
            assert!(on_curve(&g, p) < 1e-3);
        }
        assert!(dist(tangent(&g, 0.0), s) < 1e-4, "{:?}", tangent(&g, 0.0));
        assert!(dist(tangent(&g, 1.0), e) < 1e-4, "{:?}", tangent(&g, 1.0));
    }

    #[test]
    fn smooth_closing_has_no_kink_and_sharp_closing_does() {
        for knots in [Knots::Uniform, Knots::Chord] {
            for degree in [2, 3, 5] {
                let g = interpolate(&PTS, degree, knots, None, None, Closing::Smooth).unwrap();
                for p in PTS {
                    assert!(on_curve(&g, p) < 1e-3, "{knots:?} {degree}");
                }
                assert!(dist(at(&g, 0.0), at(&g, 1.0)) < 1e-9, "closed");
                let (t0, t1) = (tangent(&g, 0.0), tangent(&g, 1.0));
                assert!(dist(t0, t1) < 1e-4, "smooth seam {degree}: {t0:?} {t1:?}");
            }
        }
        let g = interpolate(&PTS, 3, Knots::Chord, None, None, Closing::Sharp).unwrap();
        assert!(dist(at(&g, 0.0), at(&g, 1.0)) < 1e-9);
        assert!(
            dist(tangent(&g, 0.0), tangent(&g, 1.0)) > 1e-2,
            "kink at the seam"
        );
    }

    #[test]
    fn deformable_circle_is_close_to_the_circle() {
        let g = deformable([0.0; 3], [5.0, 0.0, 0.0], [0.0, 5.0, 0.0], 3, 10).unwrap();
        let Geo::Nurbs { points, degree, .. } = &g else {
            panic!()
        };
        assert_eq!(*degree, 3);
        assert_eq!(
            points.len(),
            10 + 3,
            "clamped form of a 10-point periodic curve"
        );
        let worst = (0..=1000)
            .map(|i| (norm(at(&g, i as f64 / 1000.0)) - 5.0).abs())
            .fold(0.0, f64::max);
        assert!(worst < 0.01, "{worst}");
        assert!(dist(tangent(&g, 0.0), tangent(&g, 1.0)) < 1e-4);
    }

    #[test]
    fn knot_names() {
        assert_eq!(Knots::parse("SqrtChrd"), Some(Knots::SqrtChord));
        assert_eq!(Knots::parse("chord"), Some(Knots::Chord));
        assert_eq!(Knots::Uniform.name(), "Uniform");
    }

    #[test]
    fn least_squares_fits_follow_the_samples() {
        let circle: Vec<Vec3> = (0..200)
            .map(|i| {
                let a = i as f64 / 200.0 * std::f64::consts::TAU;
                [5.0 * a.cos(), 5.0 * a.sin(), 0.0]
            })
            .collect();
        let g = fit(&circle, 12, 3, true).unwrap();
        for i in 0..100 {
            let p = at(&g, i as f64 / 100.0);
            assert!((dist(p, [0.0; 3]) - 5.0).abs() < 2e-3, "{p:?}");
        }
        assert!(
            dist(tangent(&g, 0.0), tangent(&g, 1.0)) < 1e-4,
            "smooth seam"
        );
        let open: Vec<Vec3> = circle[..120].to_vec();
        let g = fit(&open, 8, 3, false).unwrap();
        let Geo::Nurbs { points, .. } = &g else {
            panic!()
        };
        assert_eq!(points.len(), 8);
        assert!(dist(at(&g, 0.0), open[0]) < 1e-12 && dist(at(&g, 1.0), open[119]) < 1e-12);
        for q in &open {
            assert!(on_curve(&g, *q) < 5e-3);
        }
    }
}
