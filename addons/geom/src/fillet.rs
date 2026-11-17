// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 FilletSrf: a constant-radius rolling-ball surface between two surfaces.
//!
//! The ball centre runs along the intersection of the two surfaces offset by the radius
//! towards each other (traced by marching with Newton corrections on the exact surfaces); at
//! each centre the cross-section is the exact circular arc (rational quadratic) between the
//! two contact points. Stations along the centre line are refined until the surface is
//! within the tolerance of the true rolling-ball surface; the contact edges lie on the input
//! surfaces to the same tolerance.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]
use crate::exact::H;
use crate::nurbs::Srf;
use crate::offset::normal;
use crate::sweep::{Swept, fit_stations};
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, dot, norm, scale, sub, unit};

/// One ball position: parameters on both surfaces (normalised), centre and contacts.
#[derive(Debug, Clone, Copy)]
struct Ball {
    x: [f64; 4],
    c: Vec3,
    p1: Vec3,
    p2: Vec3,
}

struct Pair<'a> {
    s1: Srf<'a>,
    s2: Srf<'a>,
    r: f64,
    /// Which side of each surface the ball rolls on (+1 along its normal, −1 against).
    side: [f64; 2],
    /// ∂u × ∂v against `normal`'s side, per surface (found once).
    orient: [f64; 2],
    /// Variable radius: the radius at a contact point on the first surface.
    vr: Option<&'a dyn Fn(Vec3) -> f64>,
}

/// A fast unit normal: central differences (one-sided at the edges), `orient` sets the side;
/// falls back to the careful one at singular points.
fn quick_normal(s: &Srf, a: f64, b: f64, orient: f64) -> Vec3 {
    let h = 1e-6;
    let (a0, a1) = ((a - h).max(0.0), (a + h).min(1.0));
    let (b0, b1) = ((b - h).max(0.0), (b + h).min(1.0));
    let du = sub(s.at_st(a1, b), s.at_st(a0, b));
    let dv = sub(s.at_st(a, b1), s.at_st(a, b0));
    let n = cross(du, dv);
    if norm(n) > 1e-18 { scale(unit(n), orient) } else { normal(s, a, b) }
}

fn orientation(s: &Srf) -> f64 {
    let n = quick_normal(s, 0.5, 0.5, 1.0);
    if dot(n, normal(s, 0.5, 0.5)) < 0.0 { -1.0 } else { 1.0 }
}

impl Pair<'_> {
    fn n1(&self, a: f64, b: f64) -> Vec3 {
        quick_normal(&self.s1, a, b, self.orient[0])
    }
    fn n2(&self, a: f64, b: f64) -> Vec3 {
        quick_normal(&self.s2, a, b, self.orient[1])
    }
    /// The radius for a ball touching the first surface at (a, b).
    fn radius(&self, a: f64, b: f64) -> f64 {
        match self.vr {
            Some(f) => f(self.s1.at_st(a, b)),
            None => self.r,
        }
    }
    fn off1(&self, a: f64, b: f64, r: f64) -> Vec3 {
        add(self.s1.at_st(a, b), scale(self.n1(a, b), self.side[0] * r))
    }
    fn off2(&self, a: f64, b: f64, r: f64) -> Vec3 {
        add(self.s2.at_st(a, b), scale(self.n2(a, b), self.side[1] * r))
    }
    fn ball(&self, x: [f64; 4]) -> Ball {
        let r = self.radius(x[0], x[1]);
        let c1 = self.off1(x[0], x[1], r);
        let c2 = self.off2(x[2], x[3], r);
        Ball { x, c: scale(add(c1, c2), 0.5), p1: self.s1.at_st(x[0], x[1]), p2: self.s2.at_st(x[2], x[3]) }
    }
    /// Residual: the two offset points apart, and (with a plane) the distance from it.
    fn residual(&self, x: [f64; 4], plane: Option<(Vec3, Vec3)>) -> [f64; 4] {
        let r = self.radius(x[0], x[1]);
        let o1 = self.off1(x[0], x[1], r);
        let d = sub(o1, self.off2(x[2], x[3], r));
        let e = plane.map_or(0.0, |(o, n)| dot(sub(o1, o), n));
        [d[0], d[1], d[2], e]
    }

    /// Newton: the ball where the offsets meet (on the plane, when given). `None` when it
    /// leaves a surface or does not converge.
    fn solve(&self, x: [f64; 4], plane: Option<(Vec3, Vec3)>) -> Option<[f64; 4]> {
        self.solve_fixing(x, plane, None)
    }

    /// As `solve`, with one parameter held where it is (a ball on a surface edge).
    fn solve_fixing(&self, mut x: [f64; 4], plane: Option<(Vec3, Vec3)>, fixed: Option<usize>) -> Option<[f64; 4]> {
        let h = 1e-6;
        let mut last = f64::MAX;
        for it in 0..40 {
            let f = self.residual(x, plane);
            let size = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2] + f[3] * f[3]).sqrt();
            if size < 1e-10 * self.r.max(1.0) {
                return Some(x);
            }
            // Stuck against a surface edge: give up early.
            if it > 3 && size > 0.5 * last && x.iter().any(|v| *v <= 0.0 || *v >= 1.0) {
                return None;
            }
            last = size;
            // Jacobian (numeric).
            let mut j = [[0.0; 4]; 4];
            for k in 0..4 {
                if fixed == Some(k) {
                    continue;
                }
                let mut xs = x;
                let step = if x[k] + h <= 1.0 { h } else { -h };
                xs[k] = x[k] + step;
                let fs = self.residual(xs, plane);
                for r in 0..4 {
                    j[r][k] = (fs[r] - f[r]) / step;
                }
            }
            let rows = if plane.is_some() { 4 } else { 3 };
            // Minimum-norm step: dx = Jᵀ (J Jᵀ)⁻¹ (−f) (square case: J⁻¹ (−f)).
            let mut jjt = vec![vec![0.0; rows]; rows];
            for a in 0..rows {
                for b in 0..rows {
                    jjt[a][b] = (0..4).map(|k| j[a][k] * j[b][k]).sum();
                }
            }
            let y = solve_small(jjt, (0..rows).map(|a| -f[a]).collect())?;
            let mut dx = [0.0; 4];
            for k in 0..4 {
                dx[k] = (0..rows).map(|a| j[a][k] * y[a]).sum();
            }
            // Damp large steps.
            let len = dx.iter().map(|v| v * v).sum::<f64>().sqrt();
            let damp = if len > 0.1 { 0.1 / len } else { 1.0 };
            let mut off = false;
            for k in 0..4 {
                let v = x[k] + dx[k] * damp;
                if !(-1e-9..=1.0 + 1e-9).contains(&v) {
                    off = true;
                }
                x[k] = v.clamp(0.0, 1.0);
            }
            if off && len * damp < 1e-12 {
                return None;
            }
            if len * damp < 1e-14 {
                break;
            }
        }
        let f = self.residual(x, plane);
        ((f[0] * f[0] + f[1] * f[1] + f[2] * f[2] + f[3] * f[3]).sqrt() < 1e-8 * self.r.max(1.0)).then_some(x)
    }

    fn tangent(&self, b: &Ball) -> Vec3 {
        unit(cross(self.n1(b.x[0], b.x[1]), self.n2(b.x[2], b.x[3])))
    }
}

fn self_solve_edge(pair: &Pair, x: [f64; 4], k: usize) -> Option<[f64; 4]> {
    pair.solve_fixing(x, None, Some(k))
}

fn solve_small(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for c in 0..n {
        let p = (c..n).max_by(|x, y| a[*x][c].abs().total_cmp(&a[*y][c].abs()))?;
        if a[p][c].abs() < 1e-300 {
            return None;
        }
        a.swap(c, p);
        b.swap(c, p);
        for r in c + 1..n {
            let f = a[r][c] / a[c][c];
            for k in c..n {
                a[r][k] -= f * a[c][k];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = vec![0.0; n];
    for r in (0..n).rev() {
        x[r] = (b[r] - (r + 1..n).map(|k| a[r][k] * x[k]).sum::<f64>()) / a[r][r];
    }
    Some(x)
}

/// Rational quadratic arc from `p1` to `p2` round `c` (shorter way, under 180°): 3 poles.
fn arc(c: Vec3, p1: Vec3, p2: Vec3) -> Vec<H> {
    let (a, b) = (sub(p1, c), sub(p2, c));
    let r = norm(a);
    let cosang = (dot(a, b) / (r * norm(b)).max(1e-300)).clamp(-1.0, 1.0);
    let half = cosang.acos() / 2.0;
    let w = half.cos().max(1e-6);
    let m = unit(add(unit(a), unit(b)));
    let mid = add(c, scale(m, r / w));
    [[p1[0], p1[1], p1[2], 1.0], [mid[0] * w, mid[1] * w, mid[2] * w, w], [p2[0], p2[1], p2[2], 1.0]].to_vec()
}

/// Trace the ball centres (with r = 0: the intersection) from end to end: the balls and their
/// arc-length places along the centre line (0..1).
fn trace(pair: &Pair, size: f64) -> Result<(Vec<Ball>, Vec<f64>), String> {
    // Start: the closest offset points on a grid, then Newton.
    let k = 16;
    let g1p: Vec<(f64, f64, Vec3)> = (0..=k)
        .flat_map(|i| (0..=k).map(move |j| (i as f64 / k as f64, j as f64 / k as f64)))
        .map(|(a, b)| (a, b, pair.off1(a, b, pair.radius(a, b))))
        .collect();
    let g2p: Vec<(f64, f64, Vec3)> = (0..=k)
        .flat_map(|i| (0..=k).map(move |j| (i as f64 / k as f64, j as f64 / k as f64)))
        .map(|(a, b)| (a, b, pair.off2(a, b, pair.r)))
        .collect();
    let mut seeds: Vec<(f64, [f64; 4])> = Vec::new();
    for p in &g1p {
        for q in &g2p {
            seeds.push((dist(p.2, q.2), [p.0, p.1, q.0, q.1]));
        }
    }
    let keep = 12.min(seeds.len());
    if seeds.len() > keep {
        seeds.select_nth_unstable_by(keep - 1, |a, b| a.0.total_cmp(&b.0));
        seeds.truncate(keep);
    }
    seeds.sort_by(|a, b| a.0.total_cmp(&b.0));
    let start = seeds
        .iter()
        .take(12)
        .find_map(|(_, x)| pair.solve(*x, None))
        .ok_or(if pair.r > 0.0 { "the surfaces are too far apart for this radius, or do not face each other" } else { "the surfaces do not intersect" })?;
    // March both ways along the centre line.
    let step = if pair.r > 0.0 { pair.r } else { size / 20.0 }.max(1e-3);
    let mut forward = vec![pair.ball(start)];
    let mut backward: Vec<Ball> = Vec::new();
    for dir in [1.0, -1.0] {
        let mut cur = pair.ball(start);
        let mut prev_t = scale(pair.tangent(&cur), dir);
        for _ in 0..4000 {
            let mut t = pair.tangent(&cur);
            if dot(t, prev_t) < 0.0 {
                t = scale(t, -1.0);
            }
            let mut h = step;
            let next = loop {
                let target = add(cur.c, scale(t, h));
                if let Some(x) = pair.solve(cur.x, Some((target, t))) {
                    let b = pair.ball(x);
                    if dist(b.c, cur.c) > 1e-9 && dist(b.c, cur.c) < 2.0 * h {
                        break Some(b);
                    }
                }
                h /= 2.0;
                if h < step / 64.0 {
                    break None;
                }
            };
            let Some(b) = next else {
                // Ran off a surface: put the last ball on the edge it ran into (the parameter
                // nearest its bound, held there).
                // (Parameters already on an edge — the ball sliding along it — do not count.)
                let gap = |i: usize| {
                    let g = cur.x[i].min(1.0 - cur.x[i]);
                    if g <= 1e-9 { f64::MAX } else { g }
                };
                let k = (0..4).min_by(|a, b| gap(*a).total_cmp(&gap(*b))).unwrap();
                let mut x = cur.x;
                x[k] = if x[k] < 0.5 { 0.0 } else { 1.0 };
                if let Some(x) = self_solve_edge(pair, x, k) {
                    let b = pair.ball(x);
                    if dot(sub(b.c, cur.c), t) > 1e-9 && dist(b.c, cur.c) < 2.0 * step {
                        if dir > 0.0 {
                            forward.push(b);
                        } else {
                            backward.push(b);
                        }
                    }
                }
                break;
            };
            // Stop where a parameter reached a surface edge.
            // Stop where a parameter newly reached a surface edge (a ball sliding along an
            // edge — surfaces meeting at their edges — goes on).
            let at_edge = (0..4).any(|k| {
                let hit = |v: f64| v <= 1e-9 || v >= 1.0 - 1e-9;
                hit(b.x[k]) && !hit(cur.x[k])
            });
            prev_t = t;
            if dir > 0.0 {
                forward.push(b);
            } else {
                backward.push(b);
            }
            cur = b;
            if at_edge {
                break;
            }
        }
    }
    backward.reverse();
    backward.extend(forward);
    let balls = backward;
    if balls.len() < 2 {
        return Err("the fillet has no length".into());
    }
    // Arc length along the centre line.
    let mut s = vec![0.0];
    for w in balls.windows(2) {
        s.push(s.last().unwrap() + dist(w[0].c, w[1].c));
    }
    let total = *s.last().unwrap();
    let s: Vec<f64> = s.iter().map(|v| v / total).collect();
    Ok((balls, s))
}

/// The ball at place `t` (exact: Newton on the plane through the interpolated centre).
fn ball_at(pair: &Pair, balls: &[Ball], s: &[f64], t: f64) -> Ball {
    let i = s.partition_point(|v| *v <= t).clamp(1, s.len() - 1) - 1;
    let f = ((t - s[i]) / (s[i + 1] - s[i]).max(1e-300)).clamp(0.0, 1.0);
    let (a, b) = (balls[i], balls[i + 1]);
    let guess: [f64; 4] = std::array::from_fn(|k| a.x[k] + (b.x[k] - a.x[k]) * f);
    let target = add(a.c, scale(sub(b.c, a.c), f));
    let tan = unit(sub(b.c, a.c));
    pair.solve(guess, Some((target, tan))).map(|x| pair.ball(x)).unwrap_or(if f < 0.5 { a } else { b })
}

/// Rhino 5 FilletSrf: the rolling-ball surface of radius `r` between `g1` and `g2`, in the
/// corner facing the points `keep1` (on g1) and `keep2` (on g2) — the sides kept.
pub fn fillet(g1: &Geo, g2: &Geo, r: f64, keep1: Vec3, keep2: Vec3, tol: f64) -> Result<Swept, String> {
    fillet_with(g1, g2, r, keep1, keep2, tol, false).map(|x| x.0)
}

/// Where a fillet or chamfer lies on each input: the range of its parameter along the
/// intersection (0..1) whose edge is on the first and on the second surface (all of it
/// unless extended past the shorter surface).
pub type OnInputs = [[f64; 2]; 2];

/// A surface carried on straight past its open edges by `len`, and the original's parameter
/// box on it ([u0, u1, v0, v1]); a closed direction is left as it is.
fn extended(g: &Geo, len: f64) -> Option<(Geo, [f64; 4])> {
    let s0 = Srf::of(g)?;
    let closed = |dir: usize| {
        [0.3, 0.7].iter().all(|t| {
            let (a, b) = if dir == 0 { ((0.0, *t), (1.0, *t)) } else { ((*t, 0.0), (*t, 1.0)) };
            dist(s0.at_st(a.0, a.1), s0.at_st(b.0, b.1)) < 1e-9
        })
    };
    let (c0, c1) = (s0.at_st(0.0, 0.0), s0.at_st(1.0, 1.0));
    let open = [!closed(0), !closed(1)];
    let mut x = g.clone();
    for dir in 0..2 {
        if open[dir] {
            for end in [false, true] {
                x = crate::extend::extend(&x, dir, end, len, false).ok()?;
            }
        }
    }
    let s = Srf::of(&x)?;
    let (a0, b0) = s.closest_st(c0);
    let (a1, b1) = s.closest_st(c1);
    let bx = [a0.min(a1), a0.max(a1), b0.min(b1), b0.max(b1)];
    Some((x, bx))
}

fn in_box(b: &[f64; 4], u: f64, v: f64) -> bool {
    let e = 1e-9;
    u >= b[0] - e && u <= b[1] + e && v >= b[2] - e && v <= b[3] + e
}

/// Rhino's Extend=Yes: keep the traced balls while either contact is on its own (unextended)
/// surface, with the end balls placed exactly where the last one leaves it.
fn clip(pair: &Pair, balls: &[Ball], s: &[f64], boxes: [[f64; 4]; 2]) -> Result<(Vec<Ball>, Vec<f64>, OnInputs), String> {
    let on = |b: &Ball, k: usize| if k == 0 { in_box(&boxes[0], b.x[0], b.x[1]) } else { in_box(&boxes[1], b.x[2], b.x[3]) };
    let keep = |b: &Ball| on(b, 0) || on(b, 1);
    // The longest run of kept balls.
    let (mut best, mut cur) = ((0usize, 0usize), None::<usize>);
    for i in 0..=balls.len() {
        let k = i < balls.len() && keep(&balls[i]);
        match (k, cur) {
            (true, None) => cur = Some(i),
            (false, Some(a)) => {
                if i - a > best.1 - best.0 {
                    best = (a, i);
                }
                cur = None;
            }
            _ => {}
        }
    }
    if best.1 <= best.0 {
        return Err("the surfaces do not meet within their edges".into());
    }
    // The place along the centre line where `test` turns from true to false between t_in and t_out.
    let edge_place = |t_in: f64, t_out: f64, test: &dyn Fn(&Ball) -> bool| -> f64 {
        let (mut a, mut b) = (t_in, t_out);
        for _ in 0..40 {
            let m = 0.5 * (a + b);
            if test(&ball_at(pair, balls, s, m)) {
                a = m;
            } else {
                b = m;
            }
        }
        a
    };
    let mut ts: Vec<f64> = s[best.0..best.1].to_vec();
    if best.0 > 0 {
        ts.insert(0, edge_place(s[best.0], s[best.0 - 1], &keep));
    }
    if best.1 < balls.len() {
        ts.push(edge_place(s[best.1 - 1], s[best.1], &keep));
    }
    let new: Vec<Ball> = ts.iter().map(|t| ball_at(pair, balls, s, *t)).collect();
    let mut len = vec![0.0];
    for w in new.windows(2) {
        len.push(len.last().unwrap() + dist(w[0].c, w[1].c));
    }
    let total = len.last().copied().unwrap_or(0.0).max(1e-300);
    let ns: Vec<f64> = len.iter().map(|v| v / total).collect();
    // Each input's own range along the new parameter.
    let range = |k: usize| -> [f64; 2] {
        let idx: Vec<usize> = (0..new.len()).filter(|i| on(&new[*i], k)).collect();
        match (idx.first(), idx.last()) {
            (Some(&a), Some(&b)) => {
                let lo = if a > 0 {
                    let t = edge_place(ts[a], ts[a - 1], &|x: &Ball| on(x, k));
                    let p = ball_at(pair, balls, s, t);
                    // Map back to the new arc-length parameter.
                    ns[a - 1] + (ns[a] - ns[a - 1]) * (dist(p.c, new[a - 1].c) / dist(new[a].c, new[a - 1].c).max(1e-300)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let hi = if b + 1 < new.len() {
                    let t = edge_place(ts[b], ts[b + 1], &|x: &Ball| on(x, k));
                    let p = ball_at(pair, balls, s, t);
                    ns[b] + (ns[b + 1] - ns[b]) * (dist(p.c, new[b].c) / dist(new[b + 1].c, new[b].c).max(1e-300)).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                [lo, hi]
            }
            _ => [0.0, 0.0],
        }
    };
    let ranges = [range(0), range(1)];
    Ok((new, ns, ranges))
}

/// FilletSrf with Rhino's Extend option: with `extend`, where one surface is longer than the
/// other the fillet runs on to the longer one's edges (rolling on the shorter one carried on
/// straight). Also returns where the fillet's edges lie on each input.
pub fn fillet_with(g1: &Geo, g2: &Geo, r: f64, keep1: Vec3, keep2: Vec3, tol: f64, extend: bool) -> Result<(Swept, OnInputs), String> {
    if r <= 0.0 {
        return Err("the radius must be more than 0".into());
    }
    let ext = if extend {
        let diag = |g: &Geo| Srf::of(g).map_or(0.0, |s| dist(s.at_st(0.0, 0.0), s.at_st(1.0, 1.0)).max(dist(s.at_st(1.0, 0.0), s.at_st(0.0, 1.0))));
        let len = diag(g1).max(diag(g2)) + 2.0 * r;
        match (extended(g1, len), extended(g2, len)) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    } else {
        None
    };
    let (e1, e2) = match &ext {
        Some(((a, _), (b, _))) => (a, b),
        None => (g1, g2),
    };
    let s1 = Srf::of(e1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(e2).ok_or("not a NURBS surface")?;
    // Sides: the ball is offset from each surface towards the other surface's kept side.
    let side_of = |s: &Srf, toward: Vec3| -> f64 {
        let (a, b) = s.closest_st(toward);
        let n = normal(s, a, b);
        if dot(sub(toward, s.at_st(a, b)), n) >= 0.0 { 1.0 } else { -1.0 }
    };
    let orient = [orientation(&s1), orientation(&s2)];
    let pair = Pair { side: [side_of(&s1, keep2), side_of(&s2, keep1)], s1, s2, r, orient, vr: None };
    let (balls, s) = trace(&pair, r)?;
    let (balls, s, on) = match &ext {
        Some(((_, b1), (_, b2))) => clip(&pair, &balls, &s, [*b1, *b2])?,
        None => (balls, s, [[0.0, 1.0], [0.0, 1.0]]),
    };
    // The exact section anywhere: from the nearest traced ball, Newton on the plane through
    // the interpolated centre.
    let placed = |t: f64| -> Vec<H> {
        let ball = ball_at(&pair, &balls, &s, t);
        arc(ball.c, ball.p1, ball.p2)
    };
    let step = if s.len() > 8 { 4 } else { 1 };
    let f = fit_stations(&placed, 0.0, 1.0, false, s.iter().copied().step_by(step).chain(std::iter::once(1.0)).collect(), 2, &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0], tol, 600)?;
    Ok((f, on))
}

/// Surface derivatives by central differences (one-sided at the edges).
fn derivs(s: &Srf, a: f64, b: f64) -> (Vec3, Vec3) {
    let h = 1e-6;
    let (a0, a1) = ((a - h).max(0.0), (a + h).min(1.0));
    let (b0, b1) = ((b - h).max(0.0), (b + h).min(1.0));
    (
        scale(sub(s.at_st(a1, b), s.at_st(a0, b)), 1.0 / (a1 - a0)),
        scale(sub(s.at_st(a, b1), s.at_st(a, b0)), 1.0 / (b1 - b0)),
    )
}

/// The point of `s` (parameters near `start`) `d` from `at`, in the plane through `at`
/// across `tan`, on the side of `toward`.
fn contact(s: &Srf, start: (f64, f64), at: Vec3, tan: Vec3, n: Vec3, d: f64, toward: Vec3) -> Option<Vec3> {
    let mut dir = unit(cross(n, tan));
    if dot(dir, sub(toward, at)) < 0.0 {
        dir = scale(dir, -1.0);
    }
    let target = add(at, scale(dir, d));
    let (mut a, mut b) = start;
    // Walk to the target (nearest point), then solve the two conditions exactly.
    for _ in 0..30 {
        let (su, sv) = derivs(s, a, b);
        let r = sub(target, s.at_st(a, b));
        let m = [[dot(su, su), dot(su, sv)], [dot(su, sv), dot(sv, sv)]];
        let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
        if det.abs() < 1e-300 {
            break;
        }
        let (ra, rb) = (dot(su, r), dot(sv, r));
        let da = (m[1][1] * ra - m[0][1] * rb) / det;
        let db = (m[0][0] * rb - m[1][0] * ra) / det;
        a = (a + da).clamp(0.0, 1.0);
        b = (b + db).clamp(0.0, 1.0);
        if da.abs() + db.abs() < 1e-13 {
            break;
        }
    }
    for _ in 0..30 {
        let p = s.at_st(a, b);
        let q = sub(p, at);
        let g = [dot(q, tan), dot(q, q) - d * d];
        if g[0].abs() + g[1].abs() < 1e-12 * d.max(1.0) {
            return Some(p);
        }
        let (su, sv) = derivs(s, a, b);
        let j = [[dot(su, tan), dot(sv, tan)], [2.0 * dot(q, su), 2.0 * dot(q, sv)]];
        let det = j[0][0] * j[1][1] - j[0][1] * j[1][0];
        if det.abs() < 1e-300 {
            return None;
        }
        let da = (-g[0] * j[1][1] + g[1] * j[0][1]) / det;
        let db = (-g[1] * j[0][0] + g[0] * j[1][0]) / det;
        a = (a + da).clamp(0.0, 1.0);
        b = (b + db).clamp(0.0, 1.0);
    }
    let p = s.at_st(a, b);
    let q = sub(p, at);
    (dot(q, tan).abs() < 1e-7 && (norm(q) - d).abs() < 1e-7).then_some(p)
}

/// Rhino 5 ChamferSrf: a ruled surface from `d1` on `g1` to `d2` on `g2`, measured from their
/// intersection across it, on the sides kept (`keep1`, `keep2`).
pub fn chamfer(g1: &Geo, g2: &Geo, d1: f64, d2: f64, keep1: Vec3, keep2: Vec3, tol: f64) -> Result<Swept, String> {
    chamfer_with(g1, g2, d1, d2, keep1, keep2, tol, false).map(|x| x.0)
}

/// ChamferSrf with Rhino's Extend option (as [`fillet_with`]).
pub fn chamfer_with(g1: &Geo, g2: &Geo, d1: f64, d2: f64, keep1: Vec3, keep2: Vec3, tol: f64, extend: bool) -> Result<(Swept, OnInputs), String> {
    if d1 <= 0.0 || d2 <= 0.0 {
        return Err("the distances must be more than 0".into());
    }
    let diag = |g: &Geo| Srf::of(g).map_or(0.0, |s| dist(s.at_st(0.0, 0.0), s.at_st(1.0, 1.0)).max(dist(s.at_st(1.0, 0.0), s.at_st(0.0, 1.0))));
    let ext = if extend {
        let len = diag(g1).max(diag(g2)) + 2.0 * d1.max(d2);
        match (extended(g1, len), extended(g2, len)) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    } else {
        None
    };
    let (e1, e2) = match &ext {
        Some(((a, _), (b, _))) => (a, b),
        None => (g1, g2),
    };
    let s1 = Srf::of(e1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(e2).ok_or("not a NURBS surface")?;
    let orient = [orientation(&s1), orientation(&s2)];
    let size = diag(e1).max(diag(e2)).max(d1).max(d2);
    let pair = Pair { side: [1.0, 1.0], s1, s2, r: 0.0, orient, vr: None };
    let (balls, s) = trace(&pair, size)?;
    let (balls, s, on) = match &ext {
        Some(((_, b1), (_, b2))) => clip(&pair, &balls, &s, [*b1, *b2])?,
        None => (balls, s, [[0.0, 1.0], [0.0, 1.0]]),
    };
    let placed = |t: f64| -> Vec<H> {
        let b = ball_at(&pair, &balls, &s, t);
        let tan = pair.tangent(&b);
        let p1 = contact(&pair.s1, (b.x[0], b.x[1]), b.c, tan, pair.n1(b.x[0], b.x[1]), d1, keep1);
        let p2 = contact(&pair.s2, (b.x[2], b.x[3]), b.c, tan, pair.n2(b.x[2], b.x[3]), d2, keep2);
        match (p1, p2) {
            (Some(p), Some(q)) => vec![[p[0], p[1], p[2], 1.0], [q[0], q[1], q[2], 1.0]],
            _ => vec![[b.c[0], b.c[1], b.c[2], 1.0]; 2],
        }
    };
    // Every traced place must have both contacts (else the distance runs off a surface).
    for t in &s {
        let row = placed(*t);
        if dist(crate::exact::euclid(row[0]), crate::exact::euclid(row[1])) < 1e-12 {
            return Err("a chamfer distance runs off a surface".into());
        }
    }
    let step = if s.len() > 8 { 4 } else { 1 };
    let f = fit_stations(&placed, 0.0, 1.0, false, s.iter().copied().step_by(step).chain(std::iter::once(1.0)).collect(), 1, &[0.0, 0.0, 1.0, 1.0], tol, 600)?;
    Ok((f, on))
}

/// Places along the intersection of two surfaces: arc-length fraction of the point on it
/// nearest `p`.
struct Along {
    pts: Vec<Vec3>,
    s: Vec<f64>,
}

impl Along {
    fn of(balls: &[Ball], s: &[f64]) -> Along {
        Along { pts: balls.iter().map(|b| b.c).collect(), s: s.to_vec() }
    }
    fn place(&self, p: Vec3) -> f64 {
        let mut best = (f64::MAX, 0.0);
        for i in 0..self.pts.len().saturating_sub(1) {
            let (a, b) = (self.pts[i], self.pts[i + 1]);
            let ab = sub(b, a);
            let l2 = dot(ab, ab).max(1e-300);
            let f = (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0);
            let d = dist(p, add(a, scale(ab, f)));
            if d < best.0 {
                best = (d, self.s[i] + (self.s[i + 1] - self.s[i]) * f);
            }
        }
        best.1
    }
}

/// A smooth profile through (place, value) handles: flat at each handle, smoothstep between
/// (Rhino's handles: the value changes smoothly from one to the next).
fn profile(handles: &[(f64, f64)], s: f64) -> f64 {
    let mut h: Vec<(f64, f64)> = handles.to_vec();
    h.sort_by(|a, b| a.0.total_cmp(&b.0));
    if s <= h[0].0 {
        return h[0].1;
    }
    for w in h.windows(2) {
        if s <= w[1].0 {
            let t = ((s - w[0].0) / (w[1].0 - w[0].0).max(1e-300)).clamp(0.0, 1.0);
            let t = t * t * (3.0 - 2.0 * t);
            return w[0].1 + (w[1].1 - w[0].1) * t;
        }
    }
    h[h.len() - 1].1
}

/// Rhino 5 VariableFilletSrf RailType: what a handle's value measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rail {
    /// The radius of the rolling ball.
    RollingBall,
    /// The distance from the edge (the intersection) to each fillet edge.
    DistFromEdge,
    /// The distance between the two fillet edges (the chord across the fillet).
    DistBetweenRails,
}

/// Rhino 5 VariableFilletSrf (RailType=RollingBall): the radius goes smoothly from handle to
/// handle along the edge. `handles`: (place 0..1 along the intersection, radius).
pub fn variable_fillet(g1: &Geo, g2: &Geo, handles: &[(f64, f64)], keep1: Vec3, keep2: Vec3, tol: f64) -> Result<Swept, String> {
    variable_fillet_rail(g1, g2, handles, keep1, keep2, Rail::RollingBall, tol)
}

/// VariableFilletSrf with Rhino's RailType: the handle values are distances from the edge or
/// between the rails, turned into the rolling-ball radius with the angle between the two
/// surfaces at each place (exact wherever that angle is the same across the fillet, as
/// between planes or along a cylinder; R = d·tan(α/2) from the edge, d / (2·cos(α/2))
/// between the rails, α the angle inside the corner).
pub fn variable_fillet_rail(g1: &Geo, g2: &Geo, handles: &[(f64, f64)], keep1: Vec3, keep2: Vec3, rail: Rail, tol: f64) -> Result<Swept, String> {
    if handles.is_empty() || handles.iter().any(|h| h.1 <= 0.0) {
        return Err("the radius must be more than 0".into());
    }
    let s1 = Srf::of(g1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(g2).ok_or("not a NURBS surface")?;
    let size = dist(s1.at_st(0.0, 0.0), s1.at_st(1.0, 1.0)).max(dist(s2.at_st(0.0, 0.0), s2.at_st(1.0, 1.0)));
    let orient = [orientation(&s1), orientation(&s2)];
    let side_of = |s: &Srf, toward: Vec3| -> f64 {
        let (a, b) = s.closest_st(toward);
        let n = normal(s, a, b);
        if dot(sub(toward, s.at_st(a, b)), n) >= 0.0 { 1.0 } else { -1.0 }
    };
    let side = [side_of(&s1, keep2), side_of(&s2, keep1)];
    // The intersection first: places along the edge, and the angle inside the corner there.
    let (along, angles) = {
        let s1 = Srf::of(g1).unwrap();
        let s2 = Srf::of(g2).unwrap();
        let pair0 = Pair { side: [1.0, 1.0], s1, s2, r: 0.0, orient, vr: None };
        let (balls, s) = trace(&pair0, size)?;
        let angles: Vec<f64> = balls
            .iter()
            .map(|b| {
                let o1 = scale(pair0.n1(b.x[0], b.x[1]), side[0]);
                let o2 = scale(pair0.n2(b.x[2], b.x[3]), side[1]);
                std::f64::consts::PI - dot(o1, o2).clamp(-1.0, 1.0).acos()
            })
            .collect();
        (Along::of(&balls, &s), angles)
    };
    let angle_at = |t: f64| -> f64 {
        let s = &along.s;
        let i = s.partition_point(|v| *v < t).clamp(1, s.len().max(2) - 1);
        if s.len() < 2 {
            return angles.first().copied().unwrap_or(std::f64::consts::FRAC_PI_2);
        }
        let f = ((t - s[i - 1]) / (s[i] - s[i - 1]).max(1e-300)).clamp(0.0, 1.0);
        angles[i - 1] + (angles[i] - angles[i - 1]) * f
    };
    let to_radius = |d: f64, alpha: f64| -> f64 {
        match rail {
            Rail::RollingBall => d,
            Rail::DistFromEdge => d * (alpha / 2.0).tan(),
            Rail::DistBetweenRails => d / (2.0 * (alpha / 2.0).cos()).max(1e-9),
        }
    };
    let radius = |p: Vec3| {
        let t = along.place(p);
        to_radius(profile(handles, t), angle_at(t))
    };
    let rmax = handles.iter().map(|h| to_radius(h.1, angle_at(h.0))).fold(0.0, f64::max);
    if !(rmax > 0.0 && rmax.is_finite()) {
        return Err("the surfaces are too flat to each other for that distance".into());
    }
    let pair = Pair { side, s1, s2, r: rmax, orient, vr: Some(&radius) };
    let (balls, s) = trace(&pair, rmax)?;
    let placed = |t: f64| -> Vec<H> {
        let ball = ball_at(&pair, &balls, &s, t);
        arc(ball.c, ball.p1, ball.p2)
    };
    fit_stations(&placed, 0.0, 1.0, false, s.iter().copied().step_by(4).collect(), 2, &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0], tol, 600)
}

/// The place (0..1) along the intersection of two surfaces nearest `p` (for handles).
pub fn place_on_intersection(g1: &Geo, g2: &Geo, p: Vec3) -> Result<f64, String> {
    let s1 = Srf::of(g1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(g2).ok_or("not a NURBS surface")?;
    let size = dist(s1.at_st(0.0, 0.0), s1.at_st(1.0, 1.0)).max(dist(s2.at_st(0.0, 0.0), s2.at_st(1.0, 1.0)));
    let orient = [orientation(&s1), orientation(&s2)];
    let pair0 = Pair { side: [1.0, 1.0], s1, s2, r: 0.0, orient, vr: None };
    let (balls, s) = trace(&pair0, size)?;
    Ok(Along::of(&balls, &s).place(p))
}

/// Rhino 5 VariableChamferSrf: distances on the two surfaces going smoothly from handle to
/// handle (place, first distance, second distance).
pub fn variable_chamfer(g1: &Geo, g2: &Geo, handles: &[(f64, f64, f64)], keep1: Vec3, keep2: Vec3, tol: f64) -> Result<Swept, String> {
    if handles.is_empty() || handles.iter().any(|h| h.1 <= 0.0 || h.2 <= 0.0) {
        return Err("the distances must be more than 0".into());
    }
    let s1 = Srf::of(g1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(g2).ok_or("not a NURBS surface")?;
    let orient = [orientation(&s1), orientation(&s2)];
    let size = dist(s1.at_st(0.0, 0.0), s1.at_st(1.0, 1.0)).max(dist(s2.at_st(0.0, 0.0), s2.at_st(1.0, 1.0)));
    let pair = Pair { side: [1.0, 1.0], s1, s2, r: 0.0, orient, vr: None };
    let (balls, s) = trace(&pair, size)?;
    let h1: Vec<(f64, f64)> = handles.iter().map(|h| (h.0, h.1)).collect();
    let h2: Vec<(f64, f64)> = handles.iter().map(|h| (h.0, h.2)).collect();
    let placed = |t: f64| -> Vec<H> {
        let b = ball_at(&pair, &balls, &s, t);
        let tan = pair.tangent(&b);
        let (d1, d2) = (profile(&h1, t), profile(&h2, t));
        let p1 = contact(&pair.s1, (b.x[0], b.x[1]), b.c, tan, pair.n1(b.x[0], b.x[1]), d1, keep1);
        let p2 = contact(&pair.s2, (b.x[2], b.x[3]), b.c, tan, pair.n2(b.x[2], b.x[3]), d2, keep2);
        match (p1, p2) {
            (Some(p), Some(q)) => vec![[p[0], p[1], p[2], 1.0], [q[0], q[1], q[2], 1.0]],
            _ => vec![[b.c[0], b.c[1], b.c[2], 1.0]; 2],
        }
    };
    for t in &s {
        let row = placed(*t);
        if dist(crate::exact::euclid(row[0]), crate::exact::euclid(row[1])) < 1e-12 {
            return Err("a chamfer distance runs off a surface".into());
        }
    }
    fit_stations(&placed, 0.0, 1.0, false, s.iter().copied().step_by(4).collect(), 1, &[0.0, 0.0, 1.0, 1.0], tol, 600)
}


/// The point and the second derivative of the surface curve leaving (a, b) along the unit
/// tangent `dir` at unit speed (its normal part is the normal curvature).
fn bend(s: &Srf, a: f64, b: f64, dir: Vec3) -> Vec3 {
    let (su, sv) = derivs(s, a, b);
    // (du, dv) with su·du + sv·dv = dir (least squares).
    let m = [[dot(su, su), dot(su, sv)], [dot(su, sv), dot(sv, sv)]];
    let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
    if det.abs() < 1e-300 {
        return [0.0; 3];
    }
    let (ra, rb) = (dot(su, dir), dot(sv, dir));
    let (du, dv) = ((m[1][1] * ra - m[0][1] * rb) / det, (m[0][0] * rb - m[1][0] * ra) / det);
    let h = 1e-3;
    let at = |k: f64| s.at_st((a + du * h * k).clamp(0.0, 1.0), (b + dv * h * k).clamp(0.0, 1.0));
    // One-sided when the curve would leave the surface.
    let (p0, p1, p2) = (at(0.0), at(1.0), at(2.0));
    let back = at(-1.0);
    let inside = (0.0..=1.0).contains(&(a - du * h)) && (0.0..=1.0).contains(&(b - dv * h));
    if inside {
        std::array::from_fn(|k| (p1[k] - 2.0 * p0[k] + back[k]) / (h * h))
    } else {
        std::array::from_fn(|k| (p2[k] - 2.0 * p1[k] + p0[k]) / (h * h))
    }
}

/// Rhino 5 VariableBlendSrf: like VariableFilletSrf (handles of radius along the edge) but
/// the section is a curvature-continuous blend between where the ball touches: a quintic
/// Bézier leaving each surface along the ball's arc direction with the surface's normal
/// curvature there; `bulge` scales the end tangents.
pub fn variable_blend(g1: &Geo, g2: &Geo, handles: &[(f64, f64)], keep1: Vec3, keep2: Vec3, bulge: f64, tol: f64) -> Result<Swept, String> {
    if handles.is_empty() || handles.iter().any(|h| h.1 <= 0.0) {
        return Err("the radius must be more than 0".into());
    }
    let s1 = Srf::of(g1).ok_or("not a NURBS surface")?;
    let s2 = Srf::of(g2).ok_or("not a NURBS surface")?;
    let size = dist(s1.at_st(0.0, 0.0), s1.at_st(1.0, 1.0)).max(dist(s2.at_st(0.0, 0.0), s2.at_st(1.0, 1.0)));
    let orient = [orientation(&s1), orientation(&s2)];
    let along = {
        let pair0 = Pair { side: [1.0, 1.0], s1: Srf::of(g1).unwrap(), s2: Srf::of(g2).unwrap(), r: 0.0, orient, vr: None };
        let (balls, s) = trace(&pair0, size)?;
        Along::of(&balls, &s)
    };
    let radius = |p: Vec3| profile(handles, along.place(p));
    let side_of = |s: &Srf, toward: Vec3| -> f64 {
        let (a, b) = s.closest_st(toward);
        let n = normal(s, a, b);
        if dot(sub(toward, s.at_st(a, b)), n) >= 0.0 { 1.0 } else { -1.0 }
    };
    let rmax = handles.iter().map(|h| h.1).fold(0.0, f64::max);
    let pair = Pair { side: [side_of(&s1, keep2), side_of(&s2, keep1)], s1, s2, r: rmax, orient, vr: Some(&radius) };
    let (balls, s) = trace(&pair, rmax)?;
    let placed = |t: f64| -> Vec<H> {
        let b = ball_at(&pair, &balls, &s, t);
        let (p, q) = (b.p1, b.p2);
        let gap = dist(p, q).max(1e-12);
        // Arc directions at the contacts (in each surface's tangent plane, across the edge).
        let along_arc = |from: Vec3, to: Vec3| {
            let r = sub(from, b.c);
            let d = sub(to, from);
            unit(sub(d, scale(r, dot(d, r) / dot(r, r).max(1e-300))))
        };
        let (t1, t2) = (along_arc(p, q), along_arc(q, p));
        let lam = bulge * gap;
        let k1 = bend(&pair.s1, b.x[0], b.x[1], t1);
        let k2 = bend(&pair.s2, b.x[2], b.x[3], t2);
        // Quintic: C'(0) = 5(P1 − P0) = λ t1, C''(0) = 20(P2 − 2P1 + P0) = λ² k1 (mirrored at 1).
        let p1 = add(p, scale(t1, lam / 5.0));
        let p2 = add(sub(scale(p1, 2.0), p), scale(k1, lam * lam / 20.0));
        let q1 = add(q, scale(t2, lam / 5.0));
        let q2 = add(sub(scale(q1, 2.0), q), scale(k2, lam * lam / 20.0));
        [p, p1, p2, q2, q1, q].iter().map(|x| [x[0], x[1], x[2], 1.0]).collect()
    };
    fit_stations(&placed, 0.0, 1.0, false, s.iter().copied().step_by(4).collect(), 5, &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], tol, 600)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::{self, Nc};

    pub(super) fn plane(o: Vec3, u: Vec3, v: Vec3) -> Geo {
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
    fn fillet_between_two_planes_is_a_quarter_cylinder() {
        // Floor z = 0 (x 0..10, y 0..8) and wall x = 0 (z 0..6, y 0..8); keep the floor at
        // x > 0 and the wall at z > 0: the fillet r = 1 is centred on (1, y, 1).
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 8.0, 0.0]);
        let wall = plane([0.0, 0.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 0.0]);
        let t0 = std::time::Instant::now();
        let f = fillet(&floor, &wall, 1.0, [5.0, 4.0, 0.0], [0.0, 4.0, 3.0], 0.001).unwrap();
        eprintln!("fillet planes: {:?}, {} stations, dev {:.2e}", t0.elapsed(), f.stations, f.deviation);
        for i in 0..=20 {
            for j in 0..=10 {
                let p = eval(&f, i as f64 / 20.0, j as f64 / 10.0);
                let d = ((p[0] - 1.0).powi(2) + (p[2] - 1.0).powi(2)).sqrt();
                assert!((d - 1.0).abs() < 1e-6, "{p:?}");
                assert!(p[0] <= 1.0 + 1e-9 && p[2] <= 1.0 + 1e-9, "the inside corner: {p:?}");
            }
            // Runs the whole length of the edge.
            let _ = i;
        }
        let (a, b) = (eval(&f, 0.0, 0.5), eval(&f, 1.0, 0.5));
        assert!((a[1] - b[1]).abs() > 7.99, "{a:?} {b:?}");
    }

    #[test]
    fn fillet_between_a_cylinder_and_a_plane() {
        // Cylinder r3 along z standing on the floor: fillet r 0.5 round its foot (outside).
        let ring = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, std::f64::consts::TAU);
        let rows: Vec<Vec<H>> = ring.points.iter().map(|c| vec![*c, [c[0], c[1], 5.0 * c[3], c[3]]]).collect();
        let cyl = exact::surface(&rows, [2, 1], [ring.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]]);
        let floor = plane([-8.0, -8.0, 0.0], [16.0, 0.0, 0.0], [0.0, 16.0, 0.0]);
        let t0 = std::time::Instant::now();
        let f = fillet(&cyl, &floor, 0.5, [3.0, 0.0, 2.0], [6.0, 0.0, 0.0], 0.001).unwrap();
        eprintln!("fillet cylinder: {:?}, {} stations, dev {:.2e}", t0.elapsed(), f.stations, f.deviation);
        assert!(f.deviation <= 0.001);
        // Every point is 0.5 from the ball centre circle (radius 3.5 at height 0.5).
        for i in 0..=60 {
            for j in 0..=8 {
                let p = eval(&f, i as f64 / 60.0, j as f64 / 8.0);
                let d = (p[0].hypot(p[1]) - 3.5).hypot(p[2] - 0.5);
                assert!((d - 0.5).abs() < 0.002, "{d}");
            }
        }
    }

    #[test]
    fn chamfer_between_two_planes() {
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 8.0, 0.0]);
        let wall = plane([0.0, 0.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 0.0]);
        let c = chamfer(&floor, &wall, 1.0, 2.0, [5.0, 4.0, 0.0], [0.0, 4.0, 3.0], 0.001).unwrap();
        // The whole edge (the planes meet along their own edges).
        assert!((eval(&c, 0.0, 0.0)[1] - eval(&c, 1.0, 0.0)[1]).abs() > 7.999, "full length {:?} {:?}", eval(&c, 0.0, 0.0), eval(&c, 1.0, 0.0));
        for i in 0..=10 {
            let (a, b) = (eval(&c, i as f64 / 10.0, 0.0), eval(&c, i as f64 / 10.0, 1.0));
            assert!(a[2].abs() < 1e-9 && (a[0] - 1.0).abs() < 1e-9, "on the floor 1 from the corner: {a:?}");
            assert!(b[0].abs() < 1e-9 && (b[2] - 2.0).abs() < 1e-9, "on the wall 2 up: {b:?}");
        }
    }
    #[test]
    fn variable_fillet_and_chamfer_follow_their_handles() {
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 8.0, 0.0]);
        let wall = plane([0.0, 0.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 0.0]);
        // Radius 1 at y = 0 to 2 at y = 8 (handles at the ends of the edge).
        let s0 = place_on_intersection(&floor, &wall, [0.0, 0.0, 0.0]).unwrap();
        let s8 = place_on_intersection(&floor, &wall, [0.0, 8.0, 0.0]).unwrap();
        assert!((s0 - s8).abs() > 0.999, "the whole edge: {s0} {s8}");
        let t0 = std::time::Instant::now();
        let f = variable_fillet(&floor, &wall, &[(s0, 1.0), (s8, 2.0)], [5.0, 4.0, 0.0], [0.0, 4.0, 3.0], 0.001).unwrap();
        eprintln!("variable fillet: {:?}, {} stations", t0.elapsed(), f.stations);
        for i in 0..=20 {
            let (a, b) = (eval(&f, i as f64 / 20.0, 0.0), eval(&f, i as f64 / 20.0, 1.0));
            // On the floor at x = r(y), on the wall at z = r(y): equal (a ball touching both).
            let y = a[1];
            let t = (y / 8.0).clamp(0.0, 1.0);
            let want = 1.0 + (t * t * (3.0 - 2.0 * t));
            assert!(a[2].abs() < 1e-9 && (a[0] - want).abs() < 2e-3, "floor contact {a:?} want x {want}");
            assert!(b[0].abs() < 1e-9 && (b[2] - a[0]).abs() < 2e-3, "wall contact {b:?}");
        }
        let c = variable_chamfer(&floor, &wall, &[(s0, 1.0, 1.0), (s8, 2.0, 3.0)], [5.0, 4.0, 0.0], [0.0, 4.0, 3.0], 0.001).unwrap();
        let (a, b) = (eval(&c, 1.0, 0.0), eval(&c, 1.0, 1.0));
        let (a0, b0) = (eval(&c, 0.0, 0.0), eval(&c, 0.0, 1.0));
        let ends = [(a, b), (a0, b0)];
        assert!(ends.iter().any(|(p, q)| (p[0] - 2.0).abs() < 1e-6 && (q[2] - 3.0).abs() < 1e-6), "{ends:?}");
        assert!(ends.iter().any(|(p, q)| (p[0] - 1.0).abs() < 1e-6 && (q[2] - 1.0).abs() < 1e-6), "{ends:?}");
    }


    #[test]
    fn variable_blend_leaves_both_surfaces_tangent() {
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 8.0, 0.0]);
        let wall = plane([0.0, 0.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 0.0]);
        let s0 = place_on_intersection(&floor, &wall, [0.0, 0.0, 0.0]).unwrap();
        let s8 = place_on_intersection(&floor, &wall, [0.0, 8.0, 0.0]).unwrap();
        let b = variable_blend(&floor, &wall, &[(s0, 1.0), (s8, 2.0)], [5.0, 4.0, 0.0], [0.0, 4.0, 3.0], 1.0, 0.001).unwrap();
        for i in 0..=10 {
            let u = i as f64 / 10.0;
            let (a, a2) = (eval(&b, u, 0.0), eval(&b, u, 1e-4));
            let (c, c2) = (eval(&b, u, 1.0), eval(&b, u, 1.0 - 1e-4));
            // Starts on the floor going flat (z' = 0), ends on the wall going straight (x' = 0).
            assert!(a[2].abs() < 1e-9 && (a2[2] - a[2]).abs() / dist(a2, a) < 1e-3, "{a:?} {a2:?}");
            assert!(c[0].abs() < 1e-9 && (c2[0] - c[0]).abs() / dist(c2, c) < 1e-3, "{c:?} {c2:?}");
        }
    }
}

/// Rhino 5 Fin: the curve (on `g`) extruded along the surface normal (or tangent to the
/// surface, across the curve) by `height`: a ruled surface whose stations are exact, refined
/// to the tolerance.
pub fn fin(curve: &crate::exact::Nc, g: &Geo, height: f64, tangent: bool, tol: f64) -> Result<Swept, String> {
    fin_heights(curve, g, &[(0.0, height)], tangent, tol)
}

/// Rhino 5 Fin with heights set at places along the curve (`(parameter, height)`, the
/// curve's normalised parameter): the height runs smoothly from one place to the next
/// (constant before the first and after the last; round a closed curve, on round to the
/// first). One place: a uniform height.
pub fn fin_heights(curve: &crate::exact::Nc, g: &Geo, heights: &[(f64, f64)], tangent: bool, tol: f64) -> Result<Swept, String> {
    if heights.is_empty() || heights.iter().any(|(_, h)| h.abs() < 1e-12) {
        return Err("the height must not be 0".into());
    }
    let wrap = curve.closed(1e-6);
    let mut handles: Vec<(f64, f64)> = heights.to_vec();
    if wrap && heights.len() > 1 {
        handles.extend(heights.iter().flat_map(|(t, h)| [(t - 1.0, *h), (t + 1.0, *h)]));
    }
    let height_at = |t: f64| if heights.len() == 1 { heights[0].1 } else { profile(&handles, t) };
    let s = Srf::of(g).ok_or("not a NURBS surface")?;
    let orient = orientation(&s);
    // Parameters on the surface along the curve, found once on a fine table and then
    // polished locally (closest points are the slow part).
    let n = 256;
    // Local projection (Newton on the surface) from a starting parameter.
    let project = |p: Vec3, start: (f64, f64)| -> (f64, f64) {
        let (mut a, mut b) = start;
        for _ in 0..8 {
            let (su, sv) = derivs(&s, a, b);
            let r = sub(p, s.at_st(a, b));
            let m = [[dot(su, su), dot(su, sv)], [dot(su, sv), dot(sv, sv)]];
            let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
            if det.abs() < 1e-300 {
                break;
            }
            let (ra, rb) = (dot(su, r), dot(sv, r));
            let (da, db) = ((m[1][1] * ra - m[0][1] * rb) / det, (m[0][0] * rb - m[1][0] * ra) / det);
            a = (a + da).clamp(0.0, 1.0);
            b = (b + db).clamp(0.0, 1.0);
            if da.abs() + db.abs() < 1e-14 {
                break;
            }
        }
        (a, b)
    };
    // The table along the curve: one global closest point, then step by step.
    let mut table: Vec<(f64, f64)> = Vec::with_capacity(n + 1);
    let mut prev = s.closest_st(curve.eval(0.0));
    for i in 0..=n {
        prev = project(curve.eval(i as f64 / n as f64), prev);
        table.push(prev);
    }
    let placed = |t: f64| -> Vec<H> {
        let i = ((t * n as f64).floor() as usize).min(n - 1);
        let f = t * n as f64 - i as f64;
        let (a0, b0) = (table[i], table[i + 1]);
        let p = curve.eval(t);
        let (a, b) = project(p, (a0.0 + (b0.0 - a0.0) * f, a0.1 + (b0.1 - a0.1) * f));
        let nrm = quick_normal(&s, a, b, orient);
        let dir = if tangent {
            let h = 1e-6;
            let tan = unit(sub(curve.eval((t + h).min(1.0)), curve.eval((t - h).max(0.0))));
            unit(cross(nrm, tan))
        } else {
            nrm
        };
        let q = add(p, scale(dir, height_at(t)));
        vec![[p[0], p[1], p[2], 1.0], [q[0], q[1], q[2], 1.0]]
    };
    fit_stations(&placed, 0.0, 1.0, wrap, curve.knots.clone(), 1, &[0.0, 0.0, 1.0, 1.0], tol, 600)
}

#[cfg(test)]
mod fin_tests {
    use super::*;
    use crate::exact::{self, Nc};

    #[test]
    fn a_fin_on_a_cylinder_stands_along_its_normals() {
        // A circle round a cylinder r 3 at height 2: the fin (height 1) is the flat ring
        // between r 3 and r 4.
        let ring = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, std::f64::consts::TAU);
        let rows: Vec<Vec<H>> = ring.points.iter().map(|c| vec![*c, [c[0], c[1], 5.0 * c[3], c[3]]]).collect();
        let cyl = exact::surface(&rows, [2, 1], [ring.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]]);
        let c = exact::arc([0.0, 0.0, 2.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, std::f64::consts::TAU);
        let t0 = std::time::Instant::now();
        let f = fin(&c, &cyl, 1.0, false, 0.001).unwrap();
        eprintln!("fin: {:?}, {} stations", t0.elapsed(), f.stations);
        let eval = |u: f64, v: f64| {
            let row: Vec<H> = (0..2).map(|j| Nc { degree: f.degree[0], points: f.rows.iter().map(|r| r[j]).collect(), knots: f.knots[0].clone() }.eval_h(u)).collect();
            Nc { degree: 1, points: row, knots: f.knots[1].clone() }.eval(v)
        };
        for i in 0..=40 {
            let u = i as f64 / 40.0;
            let (a, b) = (eval(u, 0.0), eval(u, 1.0));
            assert!((a[0].hypot(a[1]) - 3.0).abs() < 2e-3 && (a[2] - 2.0).abs() < 1e-9, "{a:?}");
            let rb = b[0].hypot(b[1]);
            assert!((rb - 4.0).abs() < 2e-3 || (rb - 2.0).abs() < 2e-3, "{rb}");
        }
    }
    use super::tests::plane;

    /// RailType on two planes at 60° (the corner inside is 60°): DistFromEdge 2 → the fillet
    /// edges 2 from the edge (R = 2·tan 30°); DistBetweenRails 2 → the edges 2 apart.
    #[test]
    fn rail_types_measure_what_rhino_says() {
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 6.0, 0.0]);
        let (c, s) = (60f64.to_radians().cos(), 60f64.to_radians().sin());
        let slope = plane([0.0, 0.0, 0.0], [0.0, 6.0, 0.0], [8.0 * c, 0.0, 8.0 * s]);
        let (k1, k2) = ([6.0, 3.0, 0.0], [3.0 * c, 3.0, 3.0 * s]);
        for (rail, d) in [(Rail::DistFromEdge, 2.0), (Rail::DistBetweenRails, 2.0), (Rail::RollingBall, 1.0)] {
            let f = variable_fillet_rail(&floor, &slope, &[(0.0, d), (1.0, d)], k1, k2, rail, 1e-5).unwrap();
            let g = f.geo();
            let srf = Srf::of(&g).unwrap();
            let (p1, p2) = (srf.at_st(0.5, 0.0), srf.at_st(0.5, 1.0));
            // Distance of each edge from the intersection line (the y axis).
            let from_edge = |p: Vec3| p[0].hypot(p[2]);
            match rail {
                Rail::DistFromEdge => assert!((from_edge(p1) - 2.0).abs() < 1e-4 && (from_edge(p2) - 2.0).abs() < 1e-4, "{p1:?} {p2:?}"),
                Rail::DistBetweenRails => assert!((dist(p1, p2) - 2.0).abs() < 1e-4, "{}", dist(p1, p2)),
                Rail::RollingBall => assert!((from_edge(p1) - 1.0 / 30f64.to_radians().tan()).abs() < 1e-4, "{p1:?}"),
            }
        }
    }

    /// Rhino 5 Fin with heights at places: 1 mm at the start of a line on a plane, 3 mm at
    /// its end, smooth between (2 mm half way).
    #[test]
    fn a_fin_with_two_heights_runs_smoothly_between_them() {
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
        let c = Nc::from_geo(&Geo::Line { start: [1.0, 5.0, 0.0], end: [9.0, 5.0, 0.0] }).unwrap();
        let f = fin_heights(&c, &floor, &[(0.0, 1.0), (1.0, 3.0)], false, 1e-5).unwrap();
        let g = f.geo();
        let s = Srf::of(&g).unwrap();
        // Top edge heights: the fin stands straight up (the plane's normal ±z).
        let top = |u: f64| s.at_st(u, 1.0)[2].abs();
        assert!((top(0.0) - 1.0).abs() < 1e-5 && (top(1.0) - 3.0).abs() < 1e-5, "{} {}", top(0.0), top(1.0));
        assert!((top(0.5) - 2.0).abs() < 1e-4, "{}", top(0.5));
        for k in 0..=20 {
            let p = s.at_st(k as f64 / 20.0, 0.0);
            assert!(p[2].abs() < 1e-9 && (p[1] - 5.0).abs() < 1e-9, "{p:?}");
        }
        // One height: uniform, as before.
        let u = fin_heights(&c, &floor, &[(0.3, 2.0)], false, 1e-5).unwrap().geo();
        let su = Srf::of(&u).unwrap();
        assert!((su.at_st(0.0, 1.0)[2].abs() - 2.0).abs() < 1e-9 && (su.at_st(1.0, 1.0)[2].abs() - 2.0).abs() < 1e-9);
    }

    /// Rhino 5 FilletSrf / ChamferSrf Extend=Yes: the second surface is shorter (y 2..6 of the
    /// first's 0..10); extended, the fillet runs the whole 10 mm and its edge on the second
    /// surface is reported as 0.2..0.6 of its length.
    #[test]
    fn extend_runs_to_the_longer_surface() {
        let floor = plane([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
        let wall = plane([5.0, 2.0, -2.0], [0.0, 4.0, 0.0], [0.0, 0.0, 7.0]);
        let (k1, k2) = ([8.0, 5.0, 0.0], [5.0, 4.0, 3.0]);
        let short = fillet(&floor, &wall, 1.0, k1, k2, 1e-4).unwrap();
        let len = |f: &Swept, k: usize| {
            let g = f.geo();
            let s = Srf::of(&g).unwrap();
            let (a, b) = (s.at_st(0.0, if k == 0 { 0.0 } else { 1.0 }), s.at_st(1.0, if k == 0 { 0.0 } else { 1.0 }));
            (dist(a, b), a, b)
        };
        assert!((len(&short, 0).0 - 4.0).abs() < 1e-6, "{:?}", len(&short, 0));
        let (long, on) = fillet_with(&floor, &wall, 1.0, k1, k2, 1e-4, true).unwrap();
        let (l, a, b) = len(&long, 0);
        assert!((l - 10.0).abs() < 1e-6, "{l} {a:?} {b:?}");
        // Still the exact quarter cylinder: every point 1 from the axis (x 6, z 1).
        let g = long.geo();
        let s = Srf::of(&g).unwrap();
        for i in 0..=20 {
            for j in 0..=4 {
                let p = s.at_st(i as f64 / 20.0, j as f64 / 4.0);
                assert!(((p[0] - 6.0).hypot(p[2] - 1.0) - 1.0).abs() < 1e-6, "{p:?}");
            }
        }
        let first = if a[1] < b[1] { [0.0, 1.0] } else { [0.0, 1.0] };
        assert!((on[0][0] - first[0]).abs() < 1e-6 && (on[0][1] - first[1]).abs() < 1e-6, "{on:?}");
        let mut r = on[1];
        if a[1] > b[1] {
            r = [1.0 - r[1], 1.0 - r[0]];
        }
        assert!((r[0] - 0.2).abs() < 1e-6 && (r[1] - 0.6).abs() < 1e-6, "{on:?}");
        // Chamfer the same way.
        let (c, _) = chamfer_with(&floor, &wall, 1.0, 1.0, k1, k2, 1e-4, true).unwrap();
        assert!((len(&c, 0).0 - 10.0).abs() < 1e-6, "{:?}", len(&c, 0));
    }
}
