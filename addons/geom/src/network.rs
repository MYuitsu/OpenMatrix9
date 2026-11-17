// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 NetworkSrf on exact curves: a Gordon surface.
//!
//! The curves split into two families (each curve crosses every curve of the other family).
//! Every curve is re-parameterised so it meets the other family at shared parameters
//! (within `tol` of the original curve); then
//! `S = loft(family A) + loft(family B) − interpolation(crossing points)`,
//! all three with the same interpolation across each direction, passes through every curve.
#![allow(clippy::needless_range_loop)]
use crate::exact::{self, H, Nc, euclid};
use om9_addon_abi::session::{Vec3, dist};

/// The nearest point of `c` to `p` near parameter `u` (Newton on (C − p)·C′).
fn local(c: &Nc, p: Vec3, mut u: f64) -> f64 {
    let h = 1e-6;
    for _ in 0..12 {
        let (a, b) = ((u - h).max(0.0), (u + h).min(1.0));
        let (pa, pm, pb) = (c.eval(a), c.eval(u), c.eval(b));
        let span = b - a;
        let d1: Vec3 = std::array::from_fn(|k| (pb[k] - pa[k]) / span);
        let d2: Vec3 = std::array::from_fn(|k| (pb[k] - 2.0 * pm[k] + pa[k]) / (span * span / 4.0));
        let r: Vec3 = std::array::from_fn(|k| pm[k] - p[k]);
        let f = r[0] * d1[0] + r[1] * d1[1] + r[2] * d1[2];
        let df = d1.iter().map(|x| x * x).sum::<f64>() + r[0] * d2[0] + r[1] * d2[1] + r[2] * d2[2];
        if df.abs() < 1e-300 {
            break;
        }
        let next = (u - f / df).clamp(0.0, 1.0);
        if (next - u).abs() < 1e-14 {
            u = next;
            break;
        }
        u = next;
    }
    u
}

/// Closest points of two curves: (parameter on a, parameter on b, distance).
fn crossing(a: &Nc, b: &Nc) -> (f64, f64, f64) {
    let k = 64;
    let pa: Vec<Vec3> = (0..=k).map(|i| a.eval(i as f64 / k as f64)).collect();
    let pb: Vec<Vec3> = (0..=k).map(|i| b.eval(i as f64 / k as f64)).collect();
    let mut best = (0, 0, f64::MAX);
    for (i, p) in pa.iter().enumerate() {
        for (j, q) in pb.iter().enumerate() {
            let d = dist(*p, *q);
            if d < best.2 {
                best = (i, j, d);
            }
        }
    }
    let (mut ua, mut ub) = (best.0 as f64 / k as f64, best.1 as f64 / k as f64);
    for _ in 0..30 {
        let nb = local(b, a.eval(ua), ub);
        let na = local(a, b.eval(nb), ua);
        let done = (na - ua).abs() + (nb - ub).abs() < 1e-13;
        (ua, ub) = (na, nb);
        if done {
            break;
        }
    }
    (ua, ub, dist(a.eval(ua), b.eval(ub)))
}

/// Arc length fraction ↔ parameter.
struct Arc {
    u: Vec<f64>,
    s: Vec<f64>,
}

impl Arc {
    fn of(c: &Nc) -> Arc {
        let n = 512;
        let (mut u, mut s) = (vec![0.0], vec![0.0]);
        let mut prev = c.eval(0.0);
        let mut run = 0.0;
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let p = c.eval(t);
            run += dist(p, prev);
            prev = p;
            u.push(t);
            s.push(run);
        }
        let total = run.max(1e-300);
        s.iter_mut().for_each(|v| *v /= total);
        Arc { u, s }
    }
    fn s_of(&self, u: f64) -> f64 {
        lookup(&self.u, &self.s, u)
    }
    fn u_of(&self, s: f64) -> f64 {
        lookup(&self.s, &self.u, s)
    }
}

fn lookup(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    let n = xs.len();
    if x <= xs[0] {
        return ys[0];
    }
    if x >= xs[n - 1] {
        return ys[n - 1];
    }
    let i = xs.partition_point(|v| *v <= x).max(1) - 1;
    ys[i] + (ys[i + 1] - ys[i]) * (x - xs[i]) / (xs[i + 1] - xs[i]).max(1e-300)
}

/// A curve through `pts` at parameters `params` (cubic where possible).
fn through(pts: &[Vec3], params: &[f64]) -> Option<Nc> {
    let pcs: Vec<Nc> = pts.iter().map(|p| exact::point_curve(*p)).collect();
    let (rows, knots) = exact::interpolate_across(&pcs, params, 3, false)?;
    let degree = knots.iter().take_while(|k| **k == knots[0]).count() - 1;
    Some(Nc { degree, points: rows.iter().map(|r| r[0]).collect(), knots })
}

/// Re-parameterise `c` so that it passes `anchors[k].1` (its own parameter) at the shared
/// parameter `anchors[k].0`, staying within `tol` of `c`. All curves of a family use the same
/// `grid`, so the results are compatible.
fn reparam(c: &Nc, anchors: &[(f64, f64)], grid: &[f64]) -> Option<(Nc, f64)> {
    let arc = Arc::of(c);
    let mut xs = vec![0.0];
    let mut ys = vec![0.0];
    for (s, u) in anchors {
        if *s > 1e-12 && *s < 1.0 - 1e-12 {
            xs.push(*s);
            ys.push(arc.s_of(*u));
        }
    }
    xs.push(1.0);
    ys.push(1.0);
    let map = |s: f64| arc.u_of(lookup(&xs, &ys, s));
    let pts: Vec<Vec3> = grid.iter().map(|s| c.eval(map(*s))).collect();
    let r = through(&pts, grid)?;
    // How far it strays: between grid points, from the original near the mapped place.
    let worst = grid
        .windows(2)
        .flat_map(|w| [0.25, 0.5, 0.75].map(|f| w[0] + (w[1] - w[0]) * f))
        .map(|s| {
            let p = r.eval(s);
            dist(p, c.eval(local(c, p, map(s))))
        })
        .fold(0.0, f64::max);
    Some((r, worst))
}

/// [`reparam`] for a closed curve: shared parameter 0 at its crossing with the first curve
/// of the other direction, once round to 1; the result is a smooth periodic curve.
fn reparam_closed(c: &Nc, anchors: &[(f64, f64)], grid: &[f64]) -> Option<(Nc, f64)> {
    let arc = Arc::of(c);
    let s0 = arc.s_of(anchors.first()?.1);
    let mut xs = vec![0.0];
    let mut ys = vec![0.0];
    for (t, u) in anchors.iter().skip(1) {
        xs.push(*t);
        ys.push((arc.s_of(*u) - s0).rem_euclid(1.0));
    }
    xs.push(1.0);
    ys.push(1.0);
    let map = |t: f64| arc.u_of((s0 + lookup(&xs, &ys, t)).rem_euclid(1.0));
    // One period of points (the end is the start again), interpolated periodically.
    let n = grid.len() - 1;
    let pcs: Vec<Nc> = grid[..n].iter().map(|t| exact::point_curve(c.eval(map(*t)))).collect();
    let (rows, knots) = exact::interpolate_across(&pcs, grid, 3, true)?;
    let degree = knots.iter().take_while(|k| **k == knots[0]).count() - 1;
    let r = Nc { degree, points: rows.iter().map(|x| x[0]).collect(), knots };
    let worst = grid
        .windows(2)
        .flat_map(|w| [0.25, 0.5, 0.75].map(|f| w[0] + (w[1] - w[0]) * f))
        .map(|t| {
            let p = r.eval(t);
            dist(p, c.eval(local(c, p, map(t))))
        })
        .fold(0.0, f64::max);
    Some((r, worst))
}

/// The shared grid: the anchors, each gap split into `per` parts.
fn grid(anchors: &[f64], per: usize) -> Vec<f64> {
    let mut a: Vec<f64> = vec![0.0];
    a.extend(anchors.iter().copied().filter(|s| *s > 1e-9 && *s < 1.0 - 1e-9));
    a.push(1.0);
    a.sort_by(f64::total_cmp);
    a.dedup_by(|x, y| (*x - *y).abs() < 1e-9);
    let mut g = Vec::new();
    for w in a.windows(2) {
        for k in 0..per {
            g.push(w[0] + (w[1] - w[0]) * k as f64 / per as f64);
        }
    }
    g.push(1.0);
    g
}

fn transpose(rows: &[Vec<H>]) -> Vec<Vec<H>> {
    (0..rows[0].len()).map(|j| rows.iter().map(|r| r[j]).collect()).collect()
}

/// A surface: rows along u, each row a curve along v.
#[derive(Clone)]
pub(crate) struct Srf {
    pub(crate) rows: Vec<Vec<H>>,
    pub(crate) degree: [usize; 2],
    pub(crate) knots: [Vec<f64>; 2],
}

impl Srf {
    pub(crate) fn row(&self, i: usize) -> Nc {
        Nc { degree: self.degree[1], points: self.rows[i].clone(), knots: self.knots[1].clone() }
    }
    pub(crate) fn flipped(&self) -> Srf {
        Srf {
            rows: transpose(&self.rows),
            degree: [self.degree[1], self.degree[0]],
            knots: [self.knots[1].clone(), self.knots[0].clone()],
        }
    }
}

/// Bring every surface to the same degree and knots along v (rows), shapes unchanged.
pub(crate) fn conform_v(srfs: &mut [Srf]) {
    let reps: Vec<Nc> = srfs.iter().map(|s| s.row(0)).collect();
    for s in srfs.iter_mut() {
        let mut out = Vec::with_capacity(s.rows.len());
        let mut last = None;
        for i in 0..s.rows.len() {
            let mut set = reps.clone();
            set.push(s.row(i));
            exact::make_compatible(&mut set);
            let r = set.pop().unwrap();
            out.push(r.points);
            last = Some((r.degree, r.knots));
        }
        let (d, k) = last.unwrap();
        s.rows = out;
        s.degree[1] = d;
        s.knots[1] = k;
    }
}

fn conform(srfs: &mut [Srf]) {
    conform_v(srfs);
    let mut f: Vec<Srf> = srfs.iter().map(Srf::flipped).collect();
    conform_v(&mut f);
    for (s, t) in srfs.iter_mut().zip(f) {
        *s = t.flipped();
    }
}

/// A Gordon surface through a network of open curves.
pub struct Network {
    pub rows: Vec<Vec<H>>,
    pub degree: [usize; 2],
    pub knots: [Vec<f64>; 2],
    /// Curves in each family.
    pub counts: [usize; 2],
    /// Largest gap between crossing curves.
    pub gap: f64,
}

/// Rhino 5 NetworkSrf. `curves` in any order (auto sort); `tol`: how closely the surface
/// follows the curves; curves farther apart than `cross_tol` do not cross.
pub fn network(curves: &[Nc], tol: f64, cross_tol: f64) -> Result<Network, String> {
    if curves.len() < 3 {
        return Err("select at least three curves".into());
    }
    // Families: the curves crossing the first one, and the rest.
    let (mut fa, mut fb): (Vec<Nc>, Vec<Nc>) = (vec![curves[0].clone()], Vec::new());
    for c in &curves[1..] {
        if crossing(&curves[0], c).2 <= cross_tol {
            fb.push(c.clone());
        } else {
            fa.push(c.clone());
        }
    }
    if fa.len() < 2 && fb.len() < 2 {
        return Err("the network needs two or more curves in one direction".into());
    }
    if fb.is_empty() {
        return Err("no curve crosses the others".into());
    }
    families(fa, fb, false, tol, cross_tol)
}

/// Rhino 5 NetworkSrf NoAutoSort: the curves of the first direction and of the second, each
/// in the order picked (the surface runs through them in that order).
pub fn network_in_order(first: &[Nc], second: &[Nc], tol: f64, cross_tol: f64) -> Result<Network, String> {
    families(first.to_vec(), second.to_vec(), true, tol, cross_tol)
}

fn families(mut fa: Vec<Nc>, mut fb: Vec<Nc>, keep_order: bool, tol: f64, cross_tol: f64) -> Result<Network, String> {
    if fa.len() < 2 || fb.len() < 2 {
        return Err("the network needs two or more curves in each direction".into());
    }
    // Closed curves (a ring's rails round the finger): all of one direction, the other open.
    // The surface then closes round that way (periodic, smooth across the seam).
    let all_closed = |f: &[Nc]| f.iter().all(|c| c.closed(1e-6));
    let any_closed = |f: &[Nc]| f.iter().any(|c| c.closed(1e-6));
    if (any_closed(&fa) && !all_closed(&fa)) || (any_closed(&fb) && !all_closed(&fb)) || (all_closed(&fa) && all_closed(&fb)) {
        return Err("closed curves must make up one whole direction, the other open".into());
    }
    if all_closed(&fa) {
        std::mem::swap(&mut fa, &mut fb);
    }
    let wrap = all_closed(&fb);
    if wrap && fa.len() < 3 {
        return Err("a closed network needs three or more curves across the closed ones".into());
    }
    // Order family B along the first A curve, family A along the first B curve.
    let order = |fam: &mut Vec<Nc>, along: &Nc| {
        let mut keyed: Vec<(f64, Nc)> = fam.drain(..).map(|c| (crossing(along, &c).0, c)).collect();
        keyed.sort_by(|x, y| x.0.total_cmp(&y.0));
        fam.extend(keyed.into_iter().map(|k| k.1));
    };
    if !keep_order {
        let b0 = fb[0].clone();
        order(&mut fa, &b0);
        let a0 = fa[0].clone();
        order(&mut fb, &a0);
    }
    // Run every A curve the way B is ordered, every B curve the way A is ordered.
    let (m, n) = (fa.len(), fb.len());
    for c in fa.iter_mut() {
        if n >= 2 && crossing(c, &fb[0]).0 > crossing(c, &fb[n - 1]).0 {
            *c = c.reversed();
        }
    }
    for c in fb.iter_mut() {
        if wrap {
            // Closed: going on from the first A crossing, the second comes before the last.
            let arc = Arc::of(c);
            let at = |i: usize| arc.s_of(crossing(c, &fa[i]).0);
            let (s0, s1, sl) = (at(0), at(1), at(m - 1));
            if (s1 - s0).rem_euclid(1.0) > (sl - s0).rem_euclid(1.0) {
                *c = c.reversed();
            }
        } else if m >= 2 && crossing(c, &fa[0]).0 > crossing(c, &fa[m - 1]).0 {
            *c = c.reversed();
        }
    }
    if m < 2 || n < 2 {
        return Err("the network needs two or more curves in each direction".into());
    }
    // Crossings.
    let mut xa = vec![vec![0.0; n]; m];
    let mut xb = vec![vec![0.0; n]; m];
    let mut xp = vec![vec![[0.0; 3]; n]; m];
    let mut gap = 0.0f64;
    for i in 0..m {
        for j in 0..n {
            let (ua, ub, d) = crossing(&fa[i], &fb[j]);
            if d > cross_tol {
                return Err("every curve must cross every curve of the other direction".into());
            }
            gap = gap.max(d);
            xa[i][j] = ua;
            xb[i][j] = ub;
            let (p, q) = (fa[i].eval(ua), fb[j].eval(ub));
            xp[i][j] = [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0, (p[2] + q[2]) / 2.0];
        }
    }
    // Shared parameters: where (on average, by arc length) each crossing sits.
    let arcs_a: Vec<Arc> = fa.iter().map(Arc::of).collect();
    let arcs_b: Vec<Arc> = fb.iter().map(Arc::of).collect();
    let mut s_at: Vec<f64> = (0..n).map(|j| (0..m).map(|i| arcs_a[i].s_of(xa[i][j])).sum::<f64>() / m as f64).collect();
    let mut t_at: Vec<f64> = if wrap {
        // Round each closed curve from its crossing with the first A curve.
        (0..m)
            .map(|i| (0..n).map(|j| (arcs_b[j].s_of(xb[i][j]) - arcs_b[j].s_of(xb[0][j])).rem_euclid(1.0)).sum::<f64>() / n as f64)
            .collect()
    } else {
        (0..m).map(|i| (0..n).map(|j| arcs_b[j].s_of(xb[i][j])).sum::<f64>() / n as f64).collect()
    };
    if wrap {
        t_at[0] = 0.0;
        if t_at.windows(2).any(|w| w[1] <= w[0]) {
            return Err("the curves across the closed ones are not in one order round them".into());
        }
    }
    // A curve at an end of the other family is the surface edge: pin it there.
    let pin = |v: &mut Vec<f64>| {
        let k = v.len();
        if k >= 2 {
            if v[0] < 0.02 {
                v[0] = 0.0;
            }
            if v[k - 1] > 0.98 {
                v[k - 1] = 1.0;
            }
        }
    };
    pin(&mut s_at);
    if !wrap {
        pin(&mut t_at);
    }
    // Re-parameterise each family on one grid, finer until within tol.
    let mut per = 8;
    let (ra, rb) = loop {
        let (gs, gt) = (grid(&s_at, per), grid(&t_at, per));
        let ra: Option<Vec<(Nc, f64)>> = (0..m)
            .map(|i| reparam(&fa[i], &(0..n).map(|j| (s_at[j], xa[i][j])).collect::<Vec<_>>(), &gs))
            .collect();
        let rb: Option<Vec<(Nc, f64)>> = (0..n)
            .map(|j| {
                let anchors: Vec<(f64, f64)> = (0..m).map(|i| (t_at[i], xb[i][j])).collect();
                if wrap { reparam_closed(&fb[j], &anchors, &gt) } else { reparam(&fb[j], &anchors, &gt) }
            })
            .collect();
        let (ra, rb) = (ra.ok_or("could not fit the curves")?, rb.ok_or("could not fit the curves")?);
        let worst = ra.iter().chain(&rb).map(|x| x.1).fold(0.0, f64::max);
        let (ra, rb): (Vec<Nc>, Vec<Nc>) = (ra.into_iter().map(|x| x.0).collect(), rb.into_iter().map(|x| x.0).collect());
        if worst <= tol || per >= 64 {
            break (ra, rb);
        }
        per *= 2;
    };
    // The three surfaces (rows along t = across family A, columns along s).
    let qb = 3.min(n - 1).max(1);
    let t_params: Vec<f64> = if wrap { t_at.iter().copied().chain(std::iter::once(1.0)).collect() } else { t_at.clone() };
    let lu = {
        let (rows, knots) = exact::interpolate_across(&ra, &t_params, 3, wrap).ok_or("could not loft the curves")?;
        let qa = knots.iter().take_while(|k| **k == knots[0]).count() - 1;
        Srf { rows, degree: [qa, ra[0].degree], knots: [knots, ra[0].knots.clone()] }
    };
    let lv = {
        let (rows, knots) = exact::interpolate_across(&rb, &s_at, 3, false).ok_or("could not loft the curves")?;
        Srf { rows, degree: [qb, rb[0].degree], knots: [knots, rb[0].knots.clone()] }.flipped()
    };
    let tp = {
        // Across s for each A row, then across t.
        let cols: Vec<Nc> = (0..m)
            .map(|i| through(&xp[i], &s_at))
            .collect::<Option<Vec<_>>>()
            .ok_or("could not fit the crossings")?;
        let (rows, knots) = exact::interpolate_across(&cols, &t_params, 3, wrap).ok_or("could not fit the crossings")?;
        let qa = knots.iter().take_while(|k| **k == knots[0]).count() - 1;
        Srf { rows, degree: [qa, cols[0].degree], knots: [knots, cols[0].knots.clone()] }
    };
    let mut all = [lu, lv, tp];
    conform(&mut all);
    let [lu, lv, tp] = all;
    let rows: Vec<Vec<H>> = (0..lu.rows.len())
        .map(|i| {
            (0..lu.rows[0].len())
                .map(|j| {
                    let (a, b, c) = (euclid(lu.rows[i][j]), euclid(lv.rows[i][j]), euclid(tp.rows[i][j]));
                    [a[0] + b[0] - c[0], a[1] + b[1] - c[1], a[2] + b[2] - c[2], 1.0]
                })
                .collect()
        })
        .collect();
    Ok(Network { rows, degree: lu.degree, knots: lu.knots, counts: [m, n], gap })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wavy(f: impl Fn(f64) -> Vec3) -> Nc {
        let pts: Vec<Vec3> = (0..=8).map(|i| f(i as f64 / 8.0)).collect();
        Nc::from_geo(&crate::interpolate(&pts, 3, false).unwrap()).unwrap()
    }

    fn eval(n: &Network, u: f64, v: f64) -> Vec3 {
        let row: Vec<H> = (0..n.rows[0].len())
            .map(|j| Nc { degree: n.degree[0], points: n.rows.iter().map(|r| r[j]).collect(), knots: n.knots[0].clone() }.eval_h(u))
            .collect();
        Nc { degree: n.degree[1], points: row, knots: n.knots[1].clone() }.eval(v)
    }

    #[test]
    fn the_network_surface_holds_every_curve() {
        // z = sin(x)·cos(y) sampled along 3 x-curves and 4 y-curves (unevenly spaced).
        let h = |x: f64, y: f64| 2.0 * (x * 0.3).sin() * (y * 0.25).cos();
        let ys = [0.0, 3.0, 10.0];
        let xs = [0.0, 2.0, 7.0, 12.0];
        let mut curves = Vec::new();
        for y in ys {
            curves.push(wavy(|t| [12.0 * t, y, h(12.0 * t, y)]));
        }
        for x in xs {
            // Some run the other way.
            curves.push(wavy(|t| {
                let y = if x == 7.0 { 10.0 * (1.0 - t) } else { 10.0 * t };
                [x, y, h(x, y)]
            }));
        }
        let tol = 0.001;
        let t0 = std::time::Instant::now();
        let n = network(&curves, tol, 0.01).unwrap();
        eprintln!("network 3x4: {:?}, {}x{} poles", t0.elapsed(), n.rows.len(), n.rows[0].len());
        assert_eq!(n.counts[0] + n.counts[1], 7);
        for c in &curves {
            for k in 0..=40 {
                let p = c.eval(k as f64 / 40.0);
                let mut best = (f64::MAX, 0.0, 0.0);
                for a in 0..=60 {
                    for b in 0..=60 {
                        let (u, v) = (a as f64 / 60.0, b as f64 / 60.0);
                        let d = dist(p, eval(&n, u, v));
                        if d < best.0 {
                            best = (d, u, v);
                        }
                    }
                }
                // Refine around the best.
                let mut d = best.0;
                for a in -30..=30 {
                    for b in -30..=30 {
                        let (u, v) = ((best.1 + a as f64 / 1800.0).clamp(0.0, 1.0), (best.2 + b as f64 / 1800.0).clamp(0.0, 1.0));
                        d = d.min(dist(p, eval(&n, u, v)));
                    }
                }
                assert!(d < 5.0 * tol, "{d}");
            }
        }
    }

    /// A closed network (Rhino 5 NetworkSrf with closed rails, as for a ring band): two
    /// circles (r 9 at z −1, r 10 at z 1) and four slanted lines between them make the cone
    /// band r = 9.5 + 0.5 z, closed round smoothly.
    #[test]
    fn a_closed_network_closes_smoothly() {
        use std::f64::consts::{FRAC_PI_2, TAU};
        let circle = |r: f64, z: f64, start: f64| exact::arc([0.0, 0.0, z], [start.cos(), start.sin(), 0.0], [-start.sin(), start.cos(), 0.0], r, TAU);
        let line = |a: f64| Nc::from_geo(&om9_addon_abi::session::Geo::Line { start: [9.0 * a.cos(), 9.0 * a.sin(), -1.0], end: [10.0 * a.cos(), 10.0 * a.sin(), 1.0] }).unwrap();
        // Circles seamed in different places; lines in a mixed order.
        let curves = vec![line(0.0), circle(9.0, -1.0, 0.3), line(2.0 * FRAC_PI_2), line(FRAC_PI_2), circle(10.0, 1.0, 2.0), line(3.0 * FRAC_PI_2)];
        let n = network(&curves, 1e-5, 0.01).unwrap();
        let g = exact::surface(&n.rows, n.degree, n.knots.clone());
        let s = crate::nurbs::Srf::of(&g).unwrap();
        for i in 0..=40 {
            for j in 0..=10 {
                let p = s.at_st(i as f64 / 40.0, j as f64 / 10.0);
                let want = 9.5 + 0.5 * p[2];
                assert!((p[0].hypot(p[1]) - want).abs() < 1e-4, "{p:?}");
            }
        }
        // Closed round and smooth across the seam: find which direction closes.
        let closes = |dir: usize| (0..=8).all(|k| {
            let t = k as f64 / 8.0;
            let (a, b) = if dir == 0 { (s.at_st(0.0, t), s.at_st(1.0, t)) } else { (s.at_st(t, 0.0), s.at_st(t, 1.0)) };
            dist(a, b) < 1e-9
        });
        let dir = if closes(0) { 0 } else { assert!(closes(1), "closed round"); 1 };
        let h = 1e-6;
        for k in 0..=8 {
            let t = k as f64 / 8.0;
            let at = |x: f64| if dir == 0 { s.at_st(x, t) } else { s.at_st(t, x) };
            let d0 = om9_addon_abi::session::scale(om9_addon_abi::session::sub(at(h), at(0.0)), 1.0 / h);
            let d1 = om9_addon_abi::session::scale(om9_addon_abi::session::sub(at(1.0), at(1.0 - h)), 1.0 / h);
            assert!(dist(d0, d1) < 1e-3 * (1.0 + dist(d0, [0.0; 3])), "tangent across the seam {d0:?} {d1:?}");
        }
    }
}
