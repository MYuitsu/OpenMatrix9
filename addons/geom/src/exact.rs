// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact NURBS curve operations for surfaces that keep their input curves (as Rhino does):
//! lines, polylines, circles, arcs and NURBS as one clamped rational form, knot insertion,
//! degree elevation, seam moves on closed curves, compatibility (same degree and knots) and
//! interpolation across sections. Points are homogeneous `[x·w, y·w, z·w, w]`; knot vectors
//! are clamped and normalised to `[0, 1]`. Algorithms: Piegl & Tiller, *The NURBS Book*
//! (A5.1 knot insertion; Bézier decomposition and elevation for degree raising).
#![allow(clippy::needless_range_loop, clippy::type_complexity, clippy::collapsible_if)]
use crate::{basis, find_span};
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, scale, unit};
use std::f64::consts::{FRAC_PI_2, TAU};

pub type H = [f64; 4];

fn h(p: Vec3, w: f64) -> H {
    [p[0] * w, p[1] * w, p[2] * w, w]
}
fn hadd(a: H, b: H) -> H {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn hscale(a: H, s: f64) -> H {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}
fn hlerp(a: H, b: H, t: f64) -> H {
    hadd(hscale(a, 1.0 - t), hscale(b, t))
}
/// The Euclidean point of a homogeneous one.
pub fn euclid(a: H) -> Vec3 {
    [a[0] / a[3], a[1] / a[3], a[2] / a[3]]
}

/// A clamped rational NURBS curve over `[0, 1]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Nc {
    pub degree: usize,
    pub points: Vec<H>,
    pub knots: Vec<f64>,
}

impl Nc {
    /// The exact NURBS form of a curve [`Geo`]; `None` for points, meshes and surfaces.
    pub fn from_geo(g: &Geo) -> Option<Nc> {
        match g {
            Geo::Line { start, end } => Some(Nc {
                degree: 1,
                points: vec![h(*start, 1.0), h(*end, 1.0)],
                knots: vec![0.0, 0.0, 1.0, 1.0],
            }),
            Geo::Polyline { points, closed } => {
                let mut q = points.clone();
                if *closed && q.len() > 2 && dist(q[0], *q.last()?) > 1e-12 {
                    q.push(q[0]);
                }
                if q.len() < 2 {
                    return None;
                }
                let total: f64 = q.windows(2).map(|w| dist(w[0], w[1])).sum();
                if total <= 0.0 {
                    return None;
                }
                let mut knots = vec![0.0, 0.0];
                let mut run = 0.0;
                for w in q.windows(2).take(q.len() - 2) {
                    run += dist(w[0], w[1]);
                    knots.push(run / total);
                }
                knots.extend([1.0, 1.0]);
                Some(Nc {
                    degree: 1,
                    points: q.iter().map(|p| h(*p, 1.0)).collect(),
                    knots,
                })
            }
            Geo::Circle {
                center,
                x,
                y,
                radius,
            } => Some(arc(*center, *x, *y, *radius, TAU)),
            Geo::Arc {
                center,
                x,
                y,
                radius,
                angle,
            } => Some(arc(*center, *x, *y, *radius, *angle)),
            Geo::Nurbs {
                degree,
                points,
                weights,
                knots,
            } => {
                let p = *degree;
                if p == 0 || points.len() <= p || knots.len() != points.len() + p + 1 {
                    return None;
                }
                let w = |i: usize| weights.get(i).copied().unwrap_or(1.0);
                let mut c = Nc {
                    degree: p,
                    points: points.iter().enumerate().map(|(i, q)| h(*q, w(i))).collect(),
                    knots: knots.clone(),
                };
                c.clamp()?;
                c.normalise()?;
                Some(c)
            }
            _ => None,
        }
    }

    pub fn to_geo(&self) -> Geo {
        Geo::Nurbs {
            degree: self.degree,
            points: self.points.iter().map(|p| euclid(*p)).collect(),
            weights: self.points.iter().map(|p| p[3]).collect(),
            knots: self.knots.clone(),
        }
    }

    /// The homogeneous point at `u`.
    pub fn eval_h(&self, u: f64) -> H {
        let n = self.points.len();
        let u = u.clamp(self.knots[self.degree], self.knots[n]);
        let span = find_span(n, self.degree, u, &self.knots);
        let b = basis(span, u, self.degree, &self.knots);
        let mut acc = [0.0; 4];
        for (i, bi) in b.iter().enumerate() {
            acc = hadd(acc, hscale(self.points[span - self.degree + i], *bi));
        }
        acc
    }

    pub fn eval(&self, u: f64) -> Vec3 {
        euclid(self.eval_h(u))
    }

    pub fn start(&self) -> Vec3 {
        euclid(self.points[0])
    }

    pub fn end(&self) -> Vec3 {
        euclid(*self.points.last().unwrap())
    }

    pub fn closed(&self, tol: f64) -> bool {
        dist(self.start(), self.end()) <= tol
    }

    pub fn reversed(&self) -> Nc {
        let mut points = self.points.clone();
        points.reverse();
        let knots = self.knots.iter().rev().map(|k| 1.0 - k).collect();
        Nc {
            degree: self.degree,
            points,
            knots,
        }
    }

    /// Make the ends interpolating (multiplicity `degree + 1`), cutting an unclamped
    /// vector to its domain.
    fn clamp(&mut self) -> Option<()> {
        let p = self.degree;
        let n = self.points.len();
        let (a, b) = (self.knots[p], self.knots[n]);
        if b.partial_cmp(&a) != Some(std::cmp::Ordering::Greater) {
            return None;
        }
        let ends = self.knots[..=p].iter().all(|k| (*k - a).abs() <= 1e-14)
            && self.knots[n..].iter().all(|k| (*k - b).abs() <= 1e-14);
        if !ends {
            *self = clamp_to(self, a, b)?;
        }
        Some(())
    }

    fn normalise(&mut self) -> Option<()> {
        let p = self.degree;
        let (a, b) = (self.knots[p], self.knots[self.points.len()]);
        let len = b - a;
        if len.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) || !len.is_finite() {
            return None;
        }
        for k in &mut self.knots {
            *k = ((*k - a) / len).clamp(0.0, 1.0);
        }
        Some(())
    }

    /// Insert the knot `u` once (A5.1).
    pub fn insert(&mut self, u: f64) {
        let p = self.degree;
        let n = self.points.len();
        let k = find_span(n, p, u.clamp(self.knots[p], self.knots[n]), &self.knots);
        let s = self.knots.iter().filter(|x| (**x - u).abs() <= 1e-14).count();
        if s >= p {
            // Already interpolating there: one more would not change the shape.
            if s > p {
                return;
            }
        }
        let mut q = Vec::with_capacity(n + 1);
        q.extend_from_slice(&self.points[..=k - p]);
        for i in k - p + 1..=k {
            let d = self.knots[i + p] - self.knots[i];
            let a = if d.abs() < 1e-300 { 0.0 } else { (u - self.knots[i]) / d };
            q.push(hlerp(self.points[i - 1], self.points[i], a));
        }
        q.extend_from_slice(&self.points[k..]);
        self.points = q;
        self.knots.insert(k + 1, u);
    }

    /// Insert every knot of `extra` (refinement).
    pub fn refine(&mut self, extra: &[f64]) {
        for u in extra {
            self.insert(*u);
        }
    }

    /// Raise the degree by `by` without changing the curve: split into Bézier pieces, raise
    /// each, join. Interior knots get multiplicity `degree` (curves stay exact; a smooth
    /// joint keeps its smoothness, it is only written with more poles).
    pub fn elevate(&self, by: usize) -> Nc {
        if by == 0 {
            return self.clone();
        }
        let p = self.degree;
        let mut c = self.clone();
        let interior: Vec<f64> = distinct_interior(&c.knots);
        for u in &interior {
            let have = c.knots.iter().filter(|x| (**x - u).abs() <= 1e-14).count();
            for _ in have..p {
                c.insert(*u);
            }
        }
        // Bézier pieces: poles [i·p, i·p + p].
        let pieces = interior.len() + 1;
        let mut out: Vec<H> = Vec::new();
        let q = p + by;
        for s in 0..pieces {
            let mut b: Vec<H> = c.points[s * p..=s * p + p].to_vec();
            for d in p..q {
                // Raise degree d → d + 1.
                let mut r = Vec::with_capacity(d + 2);
                r.push(b[0]);
                for i in 1..=d {
                    let a = i as f64 / (d + 1) as f64;
                    r.push(hadd(hscale(b[i - 1], a), hscale(b[i], 1.0 - a)));
                }
                r.push(b[d]);
                b = r;
            }
            if s > 0 {
                b.remove(0);
            }
            out.extend(b);
        }
        let mut knots = vec![0.0; q + 1];
        for u in &interior {
            knots.extend(std::iter::repeat_n(*u, q));
        }
        knots.extend(std::iter::repeat_n(1.0, q + 1));
        Nc {
            degree: q,
            points: out,
            knots,
        }
    }

    /// A closed curve started at `u` (same shape, seam moved).
    pub fn with_seam(&self, u: f64) -> Nc {
        let p = self.degree;
        if u <= 1e-12 || u >= 1.0 - 1e-12 {
            return self.clone();
        }
        let mut c = self.clone();
        let have = c.knots.iter().filter(|x| (**x - u).abs() <= 1e-14).count();
        for _ in have..p {
            c.insert(u);
        }
        // The pole at u: the last of the `p` equal knots splits the poles.
        let at = c.knots.iter().rposition(|x| (*x - u).abs() <= 1e-14).unwrap() - p;
        let mut points: Vec<H> = c.points[at..].to_vec();
        points.extend_from_slice(&c.points[1..=at]);
        let mut knots = vec![0.0; p + 1];
        let tail: Vec<f64> = c.knots[at + p + 1..c.knots.len() - p - 1]
            .iter()
            .map(|k| k - u)
            .collect();
        let head: Vec<f64> = c.knots[p + 1..at + p + 1].iter().map(|k| k + 1.0 - u).collect();
        knots.extend(tail);
        // The old seam (parameter 1 − u now): a joint of multiplicity p.
        knots.extend(std::iter::repeat_n(1.0 - u, p));
        knots.extend(head.into_iter().filter(|k| *k < 1.0 - 1e-14));
        knots.truncate(points.len());
        while knots.len() < points.len() + p + 1 {
            knots.push(1.0);
        }
        let mut out = Nc {
            degree: p,
            points,
            knots,
        };
        if out.knots.len() != out.points.len() + p + 1 {
            return self.clone();
        }
        out.knots.iter_mut().for_each(|k| *k = k.clamp(0.0, 1.0));
        out
    }

    /// The parameter of the point nearest `p` (dense scan then Newton-free refinement).
    /// The exact piece of the curve between parameters `a` and `b` (normalised, a < b).
    pub fn part(&self, a: f64, b: f64) -> Option<Nc> {
        clamp_to(self, a, b)
    }

    pub fn closest(&self, p: Vec3) -> f64 {
        let n = 256;
        let mut best = (f64::MAX, 0.0);
        for i in 0..=n {
            let u = i as f64 / n as f64;
            let d = dist(self.eval(u), p);
            if d < best.0 {
                best = (d, u);
            }
        }
        let (mut lo, mut hi) = ((best.1 - 1.0 / n as f64).max(0.0), (best.1 + 1.0 / n as f64).min(1.0));
        for _ in 0..60 {
            let (a, b) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
            if dist(self.eval(a), p) < dist(self.eval(b), p) {
                hi = b;
            } else {
                lo = a;
            }
        }
        // Polish: Newton on f(u) = (C(u) − p)·C'(u) = 0 (the scan only settles the distance
        // to about 1e-7 because it is flat at its minimum).
        let mut u = (lo + hi) / 2.0;
        let h = 1e-6;
        let f = |u: f64| {
            let (a, b) = ((u - h).max(0.0), (u + h).min(1.0));
            let d = scale(om9_addon_abi::session::sub(self.eval(b), self.eval(a)), 1.0 / (b - a));
            let r = om9_addon_abi::session::sub(self.eval(u), p);
            r[0] * d[0] + r[1] * d[1] + r[2] * d[2]
        };
        for _ in 0..8 {
            let (a, b) = ((u - h).max(0.0), (u + h).min(1.0));
            let slope = (f(b) - f(a)) / (b - a);
            if slope.abs() < 1e-300 {
                break;
            }
            let next = (u - f(u) / slope).clamp(0.0, 1.0);
            if dist(self.eval(next), p) > dist(self.eval(u), p) {
                break;
            }
            if (next - u).abs() < 1e-15 {
                u = next;
                break;
            }
            u = next;
        }
        u
    }
}

/// An exact circular arc (rational quadratic, pieces of at most 90°).
pub fn arc(center: Vec3, x: Vec3, y: Vec3, radius: f64, angle: f64) -> Nc {
    let (x, y) = (unit(x), unit(y));
    let n = ((angle / FRAC_PI_2) - 1e-9).ceil().max(1.0) as usize;
    let d = angle / n as f64;
    let w = (d / 2.0).cos();
    let at = |a: f64| add(center, add(scale(x, radius * a.cos()), scale(y, radius * a.sin())));
    let mut points = vec![h(at(0.0), 1.0)];
    let mut knots = vec![0.0; 3];
    for i in 0..n {
        let (a0, a1) = (i as f64 * d, (i + 1) as f64 * d);
        let mid = (a0 + a1) / 2.0;
        let apex = add(
            center,
            add(scale(x, radius / w * mid.cos()), scale(y, radius / w * mid.sin())),
        );
        points.push(h(apex, w));
        points.push(h(at(a1), 1.0));
        if i + 1 < n {
            let k = (i + 1) as f64 / n as f64;
            knots.extend([k, k]);
        }
    }
    knots.extend([1.0; 3]);
    Nc {
        degree: 2,
        points,
        knots,
    }
}

fn distinct_interior(knots: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for k in knots {
        if *k > 1e-14 && *k < 1.0 - 1e-14 && out.last().is_none_or(|l| (k - l).abs() > 1e-14) {
            out.push(*k);
        }
    }
    out
}

fn multiplicities(knots: &[f64]) -> Vec<(f64, usize)> {
    let mut out: Vec<(f64, usize)> = Vec::new();
    for k in knots {
        if *k <= 1e-14 || *k >= 1.0 - 1e-14 {
            continue;
        }
        match out.last_mut() {
            Some((v, m)) if (*k - *v).abs() <= 1e-14 => *m += 1,
            _ => out.push((*k, 1)),
        }
    }
    out
}

/// Give every curve the same degree and knot vector (the curves do not change).
pub fn make_compatible(curves: &mut [Nc]) {
    let p = curves.iter().map(|c| c.degree).max().unwrap_or(1);
    for c in curves.iter_mut() {
        if c.degree < p {
            *c = c.elevate(p - c.degree);
        }
    }
    // Union of interior knots with the largest multiplicity any curve has.
    let mut union: Vec<(f64, usize)> = Vec::new();
    for c in curves.iter() {
        for (v, m) in multiplicities(&c.knots) {
            match union.iter_mut().find(|(u, _)| (u - v).abs() <= 1e-12) {
                Some((_, um)) => *um = (*um).max(m),
                None => union.push((v, m)),
            }
        }
    }
    union.sort_by(|a, b| a.0.total_cmp(&b.0));
    for c in curves.iter_mut() {
        let have = multiplicities(&c.knots);
        let mut extra = Vec::new();
        for (v, m) in &union {
            let had = have
                .iter()
                .find(|(u, _)| (u - v).abs() <= 1e-12)
                .map_or(0, |x| x.1);
            // Snap to the shared value so the vectors are identical.
            extra.extend(std::iter::repeat_n(*v, m.saturating_sub(had)));
        }
        c.refine(&extra);
        for k in c.knots.iter_mut() {
            if let Some((v, _)) = union.iter().find(|(u, _)| (*u - *k).abs() <= 1e-12) {
                *k = *v;
            }
        }
    }
}

/// Parameters across sections: chord length (or its square root for `centripetal`) between
/// matching points, averaged over `samples` points along the (compatible) sections.
pub fn section_params(sections: &[Nc], centripetal: bool, closed: bool) -> Vec<f64> {
    let m = sections.len();
    let samples = 33;
    let mut gaps = vec![0.0; if closed { m } else { m - 1 }];
    for s in 0..=samples - 1 {
        let u = s as f64 / (samples - 1) as f64;
        let pts: Vec<Vec3> = sections.iter().map(|c| c.eval(u)).collect();
        for (i, g) in gaps.iter_mut().enumerate() {
            let d = dist(pts[i], pts[(i + 1) % m]);
            *g += if centripetal { d.sqrt() } else { d };
        }
    }
    let total: f64 = gaps.iter().sum();
    let mut out = vec![0.0];
    let mut run = 0.0;
    for g in &gaps {
        run += if total > 0.0 { g / total } else { 1.0 / gaps.len() as f64 };
        out.push(run);
    }
    if !closed {
        *out.last_mut().unwrap() = 1.0;
    }
    out
}

/// Solve a small dense system with partial pivoting for several right-hand sides.
fn solve_dense(mut a: Vec<Vec<f64>>, mut b: Vec<Vec<H>>) -> Option<Vec<Vec<H>>> {
    let n = a.len();
    for col in 0..n {
        let piv = (col..n).max_by(|x, y| a[*x][col].abs().total_cmp(&a[*y][col].abs()))?;
        if a[piv][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, piv);
        b.swap(col, piv);
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            if f == 0.0 {
                continue;
            }
            for j in col..n {
                a[row][j] -= f * a[col][j];
            }
            for j in 0..b[row].len() {
                b[row][j] = hadd(b[row][j], hscale(b[col][j], -f));
            }
        }
    }
    let width = b.first().map_or(0, |r| r.len());
    let mut x = vec![vec![[0.0; 4]; width]; n];
    for row in (0..n).rev() {
        for j in 0..width {
            let mut acc = b[row][j];
            for k in row + 1..n {
                acc = hadd(acc, hscale(x[k][j], -a[row][k]));
            }
            x[row][j] = hscale(acc, 1.0 / a[row][row]);
        }
    }
    Some(x)
}

/// Interpolate across `sections` (compatible curves): the result's rows of poles, one per
/// pole row across, and the knot vector across. Open: degree `q` through all sections at
/// `params`. Closed: periodic degree `q` (smooth through the first section again), written
/// clamped. Each pole of the sections is interpolated independently (homogeneous), so every
/// section lies exactly on the result.
pub fn interpolate_across(
    sections: &[Nc],
    params: &[f64],
    q: usize,
    closed: bool,
) -> Option<(Vec<Vec<H>>, Vec<f64>)> {
    let m = sections.len();
    let width = sections.first()?.points.len();
    if sections.iter().any(|c| c.points.len() != width) {
        return None;
    }
    if m < 2 || params.len() < m + usize::from(closed) {
        return None;
    }
    let data: Vec<Vec<H>> = sections.iter().map(|c| c.points.clone()).collect();
    if !closed {
        let params = &params[..m];
        let q = q.clamp(1, m - 1);
        let mut knots = vec![0.0; q + 1];
        for j in 1..m - q {
            knots.push((j..j + q).map(|i| params[i]).sum::<f64>() / q as f64);
        }
        knots.extend(std::iter::repeat_n(1.0, q + 1));
        let a: Vec<Vec<f64>> = params
            .iter()
            .map(|&u| {
                let span = find_span(m, q, u, &knots);
                let b = basis(span, u, q, &knots);
                let mut row = vec![0.0; m];
                for (i, v) in b.iter().enumerate() {
                    row[span - q + i] = *v;
                }
                row
            })
            .collect();
        let poles = solve_dense(a, data)?;
        return Some((poles, knots));
    }
    // Periodic: m poles, knots at the parameters shifted half a step for even degree,
    // extended by one period on each side.
    let q = q.clamp(1, m - 1);
    let base: Vec<f64> = (0..m)
        .map(|i| {
            if q % 2 == 1 {
                params[i]
            } else {
                (params[i] + params[i + 1]) / 2.0
            }
        })
        .collect();
    let knot = |j: isize| -> f64 {
        let mm = m as isize;
        let (wrap, idx) = (j.div_euclid(mm), j.rem_euclid(mm));
        base[idx as usize] + wrap as f64
    };
    // Basis N_i (i = 0..m) of the periodic spline, pole i starting at knot i − (q+1)/2 so
    // the matrix is centred; evaluate with an unclamped vector of m + 2q + 1 knots.
    let shift = (q as isize + 1) / 2;
    // Two periods of poles, so any one-period window (where the curve is clamped) is inside.
    let count = 2 * m + q;
    let kv: Vec<f64> = (0..(count + q + 1) as isize).map(|j| knot(j - q as isize - shift)).collect();
    let a: Vec<Vec<f64>> = params[..m]
        .iter()
        .map(|&u| {
            let mut row = vec![0.0; m];
            // Bring u into the middle period of the vector.
            let lo = kv[q];
            let mut uu = u;
            while uu < lo {
                uu += 1.0;
            }
            while uu >= lo + 1.0 {
                uu -= 1.0;
            }
            let span = find_span(count, q, uu, &kv);
            let b = basis(span, uu, q, &kv);
            for (i, v) in b.iter().enumerate() {
                row[(span - q + i) % m] += *v;
            }
            row
        })
        .collect();
    let poles = solve_dense(a, data)?;
    // Unclamped curve per pole column: poles 0..m then the first q again; domain
    // [kv[q], kv[m + q]] = one period. Clamp it at the first section (parameter params[0]).
    let mut rows: Vec<Vec<H>> = vec![Vec::new(); count];
    for (i, row) in rows.iter_mut().enumerate() {
        *row = poles[i % m].clone();
    }
    // Clamp each column (same knots for all, so do it on a stacked curve per column).
    let mut out_rows: Option<Vec<Vec<H>>> = None;
    let mut out_knots = Vec::new();
    for j in 0..width {
        let mut c = Nc {
            degree: q,
            points: rows.iter().map(|r| r[j]).collect(),
            knots: kv.clone(),
        };
        // Domain from params[0] (in the middle period) to one period later.
        let start = {
            let mut s = params[0];
            while s < kv[q] - 1e-14 {
                s += 1.0;
            }
            s
        };
        c = clamp_to(&c, start, start + 1.0)?;
        let rows_out = out_rows.get_or_insert_with(|| vec![Vec::new(); c.points.len()]);
        if rows_out.len() != c.points.len() {
            return None;
        }
        for (i, p) in c.points.iter().enumerate() {
            rows_out[i].push(*p);
        }
        out_knots = c.knots;
    }
    Some((out_rows?, out_knots))
}

/// Rows across sections whose poles are the sections' own poles (Rhino's Loose loft: the
/// surface touches only the first and last section of an open loft). Returns the rows, the
/// knot vector across and its degree.
pub fn loose_across(sections: &[Nc], closed: bool) -> Option<(Vec<Vec<H>>, Vec<f64>, usize)> {
    let m = sections.len();
    let rows: Vec<Vec<H>> = sections.iter().map(|c| c.points.clone()).collect();
    if !closed {
        let q = 3.min(m - 1).max(1);
        let k = crate::clamped_uniform(m, q);
        let last = *k.last()?;
        return Some((rows, k.iter().map(|v| v / last).collect(), q));
    }
    let q = 3.min(m - 1).max(1);
    let count = 2 * m + q;
    let kv: Vec<f64> = (0..count + q + 1).map(|j| j as f64 / m as f64).collect();
    let width = rows[0].len();
    let mut out: Option<Vec<Vec<H>>> = None;
    let mut knots = Vec::new();
    for j in 0..width {
        let c = Nc {
            degree: q,
            points: (0..count).map(|i| rows[i % m][j]).collect(),
            knots: kv.clone(),
        };
        let a = kv[q];
        let c = clamp_to(&c, a, a + 1.0)?;
        let o = out.get_or_insert_with(|| vec![Vec::new(); c.points.len()]);
        for (i, p) in c.points.iter().enumerate() {
            o[i].push(*p);
        }
        knots = c.knots;
    }
    Some((out?, knots, q))
}

/// Greville abscissae of a knot vector of degree `p`.
fn greville(knots: &[f64], p: usize, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| knots[i + 1..=i + p].iter().sum::<f64>() / p.max(1) as f64)
        .collect()
}

/// A constant curve (a collapsed edge) at `p`.
pub fn point_curve(p: Vec3) -> Nc {
    Nc {
        degree: 1,
        points: vec![h(p, 1.0), h(p, 1.0)],
        knots: vec![0.0, 0.0, 1.0, 1.0],
    }
}

/// Rhino's EdgeSrf: the bilinearly blended Coons surface of four boundaries, built on the
/// poles so that every edge is exactly its curve. `bottom` and `top` run in u (v = 0, 1),
/// `left` and `right` in v (u = 0, 1); the corners must meet. Rational edges need weight 1
/// at their ends (circles, arcs and fitted curves have it); otherwise `None`.
pub fn coons(bottom: &Nc, right: &Nc, top: &Nc, left: &Nc) -> Option<(Vec<Vec<H>>, [usize; 2], [Vec<f64>; 2])> {
    let ends_unit = |c: &Nc| {
        (c.points[0][3] - 1.0).abs() < 1e-9 && (c.points.last().unwrap()[3] - 1.0).abs() < 1e-9
    };
    if ![bottom, right, top, left].iter().all(|c| ends_unit(c)) {
        return None;
    }
    let mut bt = vec![bottom.clone(), top.clone()];
    make_compatible(&mut bt);
    let mut lr = vec![left.clone(), right.clone()];
    make_compatible(&mut lr);
    let (b, t) = (&bt[0], &bt[1]);
    let (l, r) = (&lr[0], &lr[1]);
    let (pu, pv) = (b.degree, l.degree);
    let (nu, nv) = (b.points.len(), l.points.len());
    let gu = greville(&b.knots, pu, nu);
    let gv = greville(&l.knots, pv, nv);
    let (c00, c10, c01, c11) = (b.points[0], b.points[nu - 1], t.points[0], t.points[nu - 1]);
    let rows = (0..nu)
        .map(|i| {
            (0..nv)
                .map(|j| {
                    let lv = hlerp(b.points[i], t.points[i], gv[j]);
                    let lu = hlerp(l.points[j], r.points[j], gu[i]);
                    let bl = hlerp(hlerp(c00, c10, gu[i]), hlerp(c01, c11, gu[i]), gv[j]);
                    hadd(hadd(lv, lu), hscale(bl, -1.0))
                })
                .collect()
        })
        .collect();
    Some((rows, [pu, pv], [b.knots.clone(), l.knots.clone()]))
}

/// Order curves end to end into one chain (each reversed as needed), starting with the
/// first. `None` when they do not chain within `tol`.
pub fn chain(curves: &[Nc], tol: f64) -> Option<Vec<Nc>> {
    let mut left: Vec<Nc> = curves[1..].to_vec();
    let mut out = vec![curves[0].clone()];
    // The first curve may need turning so that a second one follows its end.
    if !left.iter().any(|c| dist(c.start(), out[0].end()) <= tol || dist(c.end(), out[0].end()) <= tol) {
        out[0] = out[0].reversed();
    }
    while !left.is_empty() {
        let end = out.last()?.end();
        let at = left
            .iter()
            .position(|c| dist(c.start(), end) <= tol || dist(c.end(), end) <= tol)?;
        let c = left.remove(at);
        out.push(if dist(c.start(), end) <= tol { c } else { c.reversed() });
    }
    Some(out)
}

/// The piece of an (unclamped) curve over `[a, b]`, clamped and normalised.
pub(crate) fn clamp_to(c: &Nc, a: f64, b: f64) -> Option<Nc> {
    let p = c.degree;
    let mut c = c.clone();
    for v in [a, b] {
        let have = c.knots.iter().filter(|x| (**x - v).abs() <= 1e-12).count();
        for _ in have..p {
            c.insert(v);
        }
    }
    // With a and b of multiplicity ≥ p: C(a) = P[last index of a − p], C(b) = P[first
    // index of b − 1]; the poles between are the clamped piece.
    let ia = c.knots.iter().rposition(|x| (*x - a).abs() <= 1e-12)?.checked_sub(p)?;
    let ib = c.knots.iter().position(|x| (*x - b).abs() <= 1e-12)?.checked_sub(1)?;
    if ib < ia + p {
        return None;
    }
    let points = c.points[ia..=ib].to_vec();
    let mut knots = vec![0.0; p + 1];
    knots.extend(
        c.knots
            .iter()
            .filter(|k| **k > a + 1e-12 && **k < b - 1e-12)
            .map(|k| (k - a) / (b - a)),
    );
    knots.extend(std::iter::repeat_n(1.0, p + 1));
    let out = Nc {
        degree: p,
        points,
        knots,
    };
    (out.knots.len() == out.points.len() + p + 1).then_some(out)
}

/// The plane normal of a closed planar-ish curve (Newell's method on samples).
pub fn loop_normal(c: &Nc) -> Option<Vec3> {
    let pts: Vec<Vec3> = (0..64).map(|i| c.eval(i as f64 / 64.0)).collect();
    let mut n = [0.0; 3];
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        n = add(n, cross(a, b));
    }
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    (len > 1e-12).then(|| scale(n, 1.0 / len))
}

/// Make the sections run the same way and, when closed, start at matching places: each
/// open section is reversed if its start is nearer the previous section's end; each closed
/// section winds like the first and starts at the point nearest the previous start.
pub fn align_sections(sections: &mut [Nc], closed: bool) {
    if sections.is_empty() {
        return;
    }
    if closed {
        let reference = loop_normal(&sections[0]);
        for i in 1..sections.len() {
            if let (Some(r), Some(n)) = (reference, loop_normal(&sections[i])) {
                if r[0] * n[0] + r[1] * n[1] + r[2] * n[2] < 0.0 {
                    sections[i] = sections[i].reversed();
                }
            }
            let target = sections[i - 1].start();
            let u = sections[i].closest(target);
            sections[i] = sections[i].with_seam(u);
        }
    } else {
        for i in 1..sections.len() {
            let (s, e) = (sections[i - 1].start(), sections[i - 1].end());
            let (a, b) = (sections[i].start(), sections[i].end());
            if dist(s, a) + dist(e, b) > dist(s, b) + dist(e, a) {
                sections[i] = sections[i].reversed();
            }
        }
    }
}

/// A surface [`Geo`] from rows of poles across (u) by along the sections (v).
pub fn surface(rows: &[Vec<H>], degree: [usize; 2], knots: [Vec<f64>; 2]) -> Geo {
    let nu = rows.len();
    let nv = rows.first().map_or(0, |r| r.len());
    let mut points = Vec::with_capacity(nu * nv);
    let mut weights = Vec::with_capacity(nu * nv);
    for r in rows {
        for p in r {
            points.push(euclid(*p));
            weights.push(p[3]);
        }
    }
    Geo::Surface {
        degree,
        size: [nu, nv],
        points,
        weights,
        knots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(a: &Nc, b: &Nc) -> f64 {
        (0..=500)
            .map(|i| {
                let u = i as f64 / 500.0;
                dist(a.eval(u), b.eval(u))
            })
            .fold(0.0, f64::max)
    }

    fn shape_gap(a: &Nc, b: &Nc) -> f64 {
        // Distance from samples of a to b (parameterisation-free).
        (0..=200)
            .map(|i| {
                let p = a.eval(i as f64 / 200.0);
                dist(p, b.eval(b.closest(p)))
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn circles_and_arcs_are_exact() {
        let c = Nc::from_geo(&Geo::Circle { center: [1.0, 2.0, 3.0], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 5.0 }).unwrap();
        for i in 0..=1000 {
            let p = c.eval(i as f64 / 1000.0);
            assert!((dist(p, [1.0, 2.0, 3.0]) - 5.0).abs() < 1e-12);
        }
        assert!(c.closed(1e-12));
        let a = Nc::from_geo(&Geo::Arc { center: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 2.0, angle: 2.5 }).unwrap();
        assert!(dist(a.end(), [2.0 * 2.5f64.cos(), 2.0 * 2.5f64.sin(), 0.0]) < 1e-12);
        for i in 0..=100 {
            assert!((norm3(a.eval(i as f64 / 100.0)) - 2.0).abs() < 1e-12);
        }
    }

    fn norm3(p: Vec3) -> f64 {
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
    }

    #[test]
    fn insertion_elevation_and_seam_keep_the_curve() {
        let c = Nc::from_geo(&Geo::Circle { center: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 5.0 }).unwrap();
        let mut d = c.clone();
        d.refine(&[0.1, 0.33, 0.33, 0.9]);
        assert!(same(&c, &d) < 1e-12);
        let e = c.elevate(1);
        assert_eq!(e.degree, 3);
        assert!(same(&c, &e) < 1e-12, "{}", same(&c, &e));
        let s = c.with_seam(0.3);
        assert!(dist(s.start(), c.eval(0.3)) < 1e-12 && s.closed(1e-12));
        assert!(shape_gap(&s, &c) < 1e-9, "{}", shape_gap(&s, &c));
        let line = Nc::from_geo(&Geo::Line { start: [0.0; 3], end: [3.0, 0.0, 0.0] }).unwrap().elevate(2);
        assert!(dist(line.eval(0.5), [1.5, 0.0, 0.0]) < 1e-12);
        let r = c.reversed();
        assert!(dist(r.eval(0.25), c.eval(0.75)) < 1e-12);
    }

    #[test]
    fn compatible_curves_keep_their_shapes() {
        let circle = Nc::from_geo(&Geo::Circle { center: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 5.0 }).unwrap();
        let poly = Nc::from_geo(&Geo::Polyline { points: vec![[0.0, 0.0, 5.0], [4.0, 0.0, 5.0], [4.0, 3.0, 5.0]], closed: false }).unwrap();
        let cubic = Nc::from_geo(&Geo::Nurbs { degree: 3, points: vec![[0.0; 3], [1.0, 2.0, 0.0], [3.0, 2.0, 0.0], [4.0, 0.0, 0.0], [6.0, 1.0, 0.0]], weights: vec![], knots: vec![0.0, 0.0, 0.0, 0.0, 0.4, 1.0, 1.0, 1.0, 1.0] }).unwrap();
        let mut all = vec![circle.clone(), poly.clone(), cubic.clone()];
        make_compatible(&mut all);
        assert!(all.iter().all(|c| c.degree == 3 && c.knots == all[0].knots && c.points.len() == all[0].points.len()));
        assert!(same(&circle, &all[0]) < 1e-12 && same(&poly, &all[1]) < 1e-12 && same(&cubic, &all[2]) < 1e-12);
    }

    #[test]
    fn interpolation_across_passes_through_every_section() {
        let circ = |z: f64, r: f64| Nc::from_geo(&Geo::Circle { center: [0.0, 0.0, z], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: r }).unwrap();
        let mut s = vec![circ(0.0, 5.0), circ(5.0, 8.0), circ(10.0, 5.0), circ(14.0, 6.0)];
        make_compatible(&mut s);
        for closed in [false, true] {
            let params = section_params(&s, false, closed);
            let (rows, knots) = interpolate_across(&s, &params, 3, closed).unwrap();
            let g = surface(&rows, [3, s[0].degree], [knots, s[0].knots.clone()]);
            let srf = crate::nurbs::Srf::of(&g).unwrap();
            for (k, sec) in s.iter().enumerate() {
                let u = params[k];
                for j in 0..=50 {
                    let v = j as f64 / 50.0;
                    let p = srf.at(u, v);
                    assert!(dist(p, sec.eval(v)) < 1e-9, "closed {closed} section {k}: {p:?}");
                }
            }
        }
    }
}
#[cfg(test)]
mod periodic_tests {
    use super::*;

    /// A closed interpolation goes through every value and is smooth (C2) across its seam.
    #[test]
    fn closed_interpolation_is_smooth_through_the_seam() {
        let vals = [0.0, 5.0, 3.0, 8.0];
        let secs: Vec<Nc> = vals
            .iter()
            .map(|v| Nc {
                degree: 1,
                points: vec![[*v, 0.0, 0.0, 1.0], [*v, 1.0, 0.0, 1.0]],
                knots: vec![0.0, 0.0, 1.0, 1.0],
            })
            .collect();
        let params = vec![0.0, 0.2, 0.55, 0.8, 1.0];
        for q in [2, 3] {
            let (rows, knots) = interpolate_across(&secs, &params, q, true).unwrap();
            let c = Nc { degree: q, points: rows.iter().map(|r| r[0]).collect(), knots };
            for k in 0..4 {
                assert!((c.eval(params[k])[0] - vals[k]).abs() < 1e-12, "q {q} k {k}");
            }
            let f = |u: f64| c.eval(u)[0];
            let e = 1e-4;
            let d2 = |u: f64| (f(u + e) - 2.0 * f(u) + f(u - e)) / (e * e);
            let (a1, b1) = ((f(e) - f(0.0)) / e, (f(1.0) - f(1.0 - e)) / e);
            assert!((a1 - b1).abs() < 1e-2 * a1.abs().max(1.0), "q {q} slope {a1} {b1}");
            if q == 3 {
                assert!((d2(2.0 * e) - d2(1.0 - 2.0 * e)).abs() < 0.05 * d2(2.0 * e).abs().max(1.0));
            }
        }
    }
}
