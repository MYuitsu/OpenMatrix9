// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 Sweep1 on exact curves: cross sections are moved along one rail by frames and
//! the surface passes exactly through every placed section.
//!
//! - Frames: **Freeform** uses a rotation-minimising frame (double reflection, Wang et al.
//!   2008), so a section keeps its angle to the rail; **Roadlike** builds x from the rail
//!   tangent and a fixed direction (Rhino 5 help, "Technical notes": x = tangent × fixed,
//!   y ⟂ both, z = tangent).
//! - Shape between sections: sections are expressed in their frames, made compatible (same
//!   degree and knots, shapes unchanged) and blended pole by pole — smoothly (constant near
//!   each section) or linearly (Rhino's *Global shape blending*).
//! - Stations along the rail are added until the surface is within `tol` of the true sweep
//!   everywhere between them; the surface interpolates every station exactly.
//! - A straight rail gives an exact extrusion (degree 1 along the rail); a circular rail with
//!   one shape gives an exact rational surface of revolution.
#![allow(clippy::needless_range_loop, clippy::collapsible_if, clippy::too_many_arguments)]
use crate::exact::{self, H, Nc, euclid};
use om9_addon_abi::session::{Vec3, add, cross, dist, dot, norm, scale, sub, unit};
use std::f64::consts::TAU;

/// How the frames along the rail are made.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Freeform,
    /// Roadlike with the fixed direction (Top: world z, Right: x, Front: y).
    Roadlike(Vec3),
}

#[derive(Debug, Clone)]
pub struct Options {
    pub style: Style,
    /// Closed sweep (closed rail, two or more sections): continue past the last section
    /// around to the first. One section on a closed rail always goes all the way round.
    pub closed: bool,
    pub global_blend: bool,
    /// Largest distance from the true sweep (mm).
    pub tol: f64,
    /// Most stations along the rail.
    pub max_stations: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            style: Style::Freeform,
            closed: false,
            global_blend: false,
            tol: 0.001,
            max_stations: 400,
        }
    }
}

/// A swept surface: pole rows along the rail (u) by along the sections (v).
#[derive(Debug, Clone)]
pub struct Swept {
    pub rows: Vec<Vec<H>>,
    pub degree: [usize; 2],
    pub knots: [Vec<f64>; 2],
    /// Largest measured distance from the true sweep (0 for the exact cases).
    pub deviation: f64,
    pub stations: usize,
    /// Extrusion or revolution: exact by construction.
    pub exact: bool,
}

impl Swept {
    pub fn geo(&self) -> om9_addon_abi::session::Geo {
        exact::surface(&self.rows, self.degree, self.knots.clone())
    }
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    o: Vec3,
    x: Vec3,
    y: Vec3,
    z: Vec3,
}

impl Frame {
    fn local(&self, p: H) -> H {
        let w = p[3];
        let d = sub(euclid(p), self.o);
        [dot(d, self.x) * w, dot(d, self.y) * w, dot(d, self.z) * w, w]
    }
    fn world(&self, l: H) -> H {
        let w = l[3];
        let q = euclid(l);
        let p = add(self.o, add(scale(self.x, q[0]), add(scale(self.y, q[1]), scale(self.z, q[2]))));
        [p[0] * w, p[1] * w, p[2] * w, w]
    }
}

fn tangent(c: &Nc, u: f64) -> Vec3 {
    for h in [1e-7, 1e-5, 1e-3] {
        let (a, b) = ((u - h).max(0.0), (u + h).min(1.0));
        let d = sub(c.eval(b), c.eval(a));
        if norm(d) > 1e-12 {
            return unit(d);
        }
    }
    [0.0, 0.0, 1.0]
}

fn perpendicular(t: Vec3) -> Vec3 {
    let a = if t[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
    unit(sub(a, scale(t, dot(a, t))))
}

/// One double-reflection step of the reference vector `r` from (x0, t0) to (x1, t1).
fn reflect(x0: Vec3, t0: Vec3, r: Vec3, x1: Vec3, t1: Vec3) -> Vec3 {
    let v1 = sub(x1, x0);
    let c1 = dot(v1, v1);
    let (rl, tl) = if c1 > 1e-24 {
        (sub(r, scale(v1, 2.0 / c1 * dot(v1, r))), sub(t0, scale(v1, 2.0 / c1 * dot(v1, t0))))
    } else {
        (r, t0)
    };
    let v2 = sub(t1, tl);
    let c2 = dot(v2, v2);
    let r1 = if c2 > 1e-24 { sub(rl, scale(v2, 2.0 / c2 * dot(v2, rl))) } else { rl };
    // Keep it exactly perpendicular to the tangent.
    let r1 = sub(r1, scale(t1, dot(r1, t1)));
    if norm(r1) < 1e-12 { perpendicular(t1) } else { unit(r1) }
}

/// Frames along the rail.
struct Rail {
    c: Nc,
    style: Style,
    /// Freeform: (u, point, tangent, reference) on a fine grid.
    grid: Vec<(f64, Vec3, Vec3, Vec3)>,
    /// Freeform on a closed sweep: turn removed per unit of u (the frame comes back to itself).
    twist: f64,
}

impl Rail {
    fn new(c: Nc, style: Style, wrap: bool) -> Rail {
        let g = 2048;
        let mut grid = Vec::with_capacity(g + 1);
        let mut twist = 0.0;
        if style == Style::Freeform {
            let t0 = tangent(&c, 0.0);
            let mut prev = (0.0, c.eval(0.0), t0, perpendicular(t0));
            grid.push(prev);
            for i in 1..=g {
                let u = i as f64 / g as f64;
                let (x1, t1) = (c.eval(u), tangent(&c, u));
                let r1 = reflect(prev.1, prev.2, prev.3, x1, t1);
                prev = (u, x1, t1, r1);
                grid.push(prev);
            }
            if wrap {
                // Angle from the start reference to where it came back, about the tangent.
                let (r0, t0) = (grid[0].3, grid[0].2);
                let r1 = grid[g].3;
                twist = dot(cross(r0, r1), t0).atan2(dot(r0, r1));
            }
        }
        Rail { c, style, grid, twist }
    }

    fn frame(&self, u: f64) -> Frame {
        let o = self.c.eval(u);
        let z = tangent(&self.c, u);
        let x = match self.style {
            Style::Freeform => {
                let g = self.grid.len() - 1;
                let i = ((u * g as f64).floor() as usize).min(g);
                let (_, x0, t0, r0) = self.grid[i];
                let r = reflect(x0, t0, r0, o, z);
                if self.twist != 0.0 {
                    // Rotate about z by −twist·u (Rodrigues; r ⟂ z).
                    let a = -self.twist * u;
                    add(scale(r, a.cos()), scale(cross(z, r), a.sin()))
                } else {
                    r
                }
            }
            Style::Roadlike(d) => {
                let x = cross(z, unit(d));
                if norm(x) > 1e-9 {
                    unit(x)
                } else {
                    // Tangent along the fixed direction: the frame is under-defined (Rhino lets
                    // it twist); keep any perpendicular.
                    perpendicular(z)
                }
            }
        };
        Frame { o, x, y: cross(z, x), z }
    }
}

/// Where a section sits on the rail: where it touches the rail, else its middle's nearest.
fn section_param(rail: &Nc, s: &Nc) -> f64 {
    let pts: Vec<Vec3> = (0..=48).map(|i| s.eval(i as f64 / 48.0)).collect();
    let size = pts.iter().map(|p| dist(*p, pts[0])).fold(0.0, f64::max).max(1e-9);
    let mut best = (f64::MAX, 0.0);
    for p in &pts {
        let u = rail.closest(*p);
        let d = dist(rail.eval(u), *p);
        if d < best.0 {
            best = (d, u);
        }
    }
    if best.0 <= (1e-3 * size).max(1e-4) {
        return best.1;
    }
    let mid = scale(pts.iter().fold([0.0; 3], |a, p| add(a, *p)), 1.0 / pts.len() as f64);
    rail.closest(mid)
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Is the rail (over its whole domain) part of a circle? Center, axis (right-handed with
/// the rail's direction) and radius.
fn circle_of(c: &Nc) -> Option<(Vec3, Vec3, f64)> {
    let (a, b, d) = (c.eval(0.0), c.eval(1.0 / 3.0), c.eval(2.0 / 3.0));
    let (ab, ad) = (sub(b, a), sub(d, a));
    let n = cross(ab, ad);
    let nn = dot(n, n);
    if nn < 1e-18 {
        return None;
    }
    // Circumcenter of a, b, d.
    let center = add(
        a,
        scale(add(scale(cross(n, ab), dot(ad, ad)), scale(cross(ad, n), dot(ab, ab))), 0.5 / nn),
    );
    let r = dist(center, a);
    let axis = unit(n);
    let ok = (0..=128).all(|i| {
        let p = c.eval(i as f64 / 128.0);
        (dist(p, center) - r).abs() <= 1e-9 * r.max(1.0) && dot(sub(p, center), axis).abs() <= 1e-9 * r.max(1.0)
    });
    ok.then_some((center, axis, r))
}

fn is_line(c: &Nc) -> bool {
    let (a, b) = (c.start(), c.end());
    let l = dist(a, b);
    if l < 1e-9 {
        return false;
    }
    let d = scale(sub(b, a), 1.0 / l);
    (0..=64).all(|i| {
        let p = sub(c.eval(i as f64 / 64.0), a);
        norm(sub(p, scale(d, dot(p, d)))) <= 1e-9 * l.max(1.0)
    })
}

/// Rows of an exact revolution of `section` (world poles) about `axis` through `center` by
/// `angle` (rational quadratic arcs, at most 90° each).
fn revolve_rows(section: &[H], center: Vec3, axis: Vec3, angle: f64) -> (Vec<Vec<H>>, Vec<f64>) {
    let arcs = (angle.abs() / std::f64::consts::FRAC_PI_2 - 1e-9).ceil().max(1.0) as usize;
    let d = angle / arcs as f64;
    let wm = (d / 2.0).cos();
    let mut knots = vec![0.0; 3];
    for k in 1..arcs {
        let t = k as f64 / arcs as f64;
        knots.extend([t, t]);
    }
    knots.extend([1.0; 3]);
    let rot = |p: Vec3, th: f64, stretch: f64| -> Vec3 {
        let c = add(center, scale(axis, dot(sub(p, center), axis)));
        let r = sub(p, c);
        let y = cross(axis, r);
        add(c, scale(add(scale(r, th.cos()), scale(y, th.sin())), stretch))
    };
    let rows = (0..=2 * arcs)
        .map(|i| {
            let th = d * i as f64 / 2.0;
            let mid = i % 2 == 1;
            section
                .iter()
                .map(|hp| {
                    let p = rot(euclid(*hp), th, if mid { 1.0 / wm } else { 1.0 });
                    let w = hp[3] * if mid { wm } else { 1.0 };
                    [p[0] * w, p[1] * w, p[2] * w, w]
                })
                .collect()
        })
        .collect();
    (rows, knots)
}

/// The iso curve of rows (degree `q`, `knots` across) at `t`: homogeneous poles.
fn iso(rows: &[Vec<H>], q: usize, knots: &[f64], t: f64) -> Vec<H> {
    let width = rows[0].len();
    (0..width)
        .map(|j| {
            let c = Nc {
                degree: q,
                points: rows.iter().map(|r| r[j]).collect(),
                knots: knots.to_vec(),
            };
            c.eval_h(t)
        })
        .collect()
}

/// Rhino 5 Sweep1. `sections` in the order picked (curves; a point section is a constant
/// curve, allowed first or last). Returns the surface or why not.
pub fn sweep1(rail: &Nc, sections: &[Nc], o: &Options) -> Result<Swept, String> {
    if sections.is_empty() {
        return Err("select the cross section curves".into());
    }
    let rail_closed = rail.closed(1e-6);
    let mut rail = rail.clone();
    // Closed rail: start at the first section and run towards the second (Rhino: the first
    // section picked is where a closed sweep starts and ends).
    let mut params: Vec<f64>;
    if rail_closed {
        let s0 = section_param(&rail, &sections[0]);
        rail = rail.with_seam(s0);
        params = sections.iter().map(|s| section_param(&rail, s)).collect();
        params[0] = 0.0;
        if sections.len() >= 2 {
            let second = params[1];
            if second > 0.5 {
                // Picked against the rail's direction (most of the way round is shorter the
                // other way): run the rail backwards.
                let ordered_back = params.windows(2).skip(1).all(|w| w[1] <= w[0] + 1e-9);
                if ordered_back {
                    rail = rail.reversed();
                    params = sections.iter().map(|s| section_param(&rail, s)).collect();
                    params[0] = 0.0;
                }
            }
        }
        for p in params.iter_mut().skip(1) {
            if *p < 1e-9 {
                *p = 1.0;
            }
        }
    } else {
        params = sections.iter().map(|s| section_param(&rail, s)).collect();
    }
    let wrap = rail_closed && (sections.len() == 1 || o.closed);
    if o.closed && !rail_closed {
        return Err("a closed sweep needs a closed rail".into());
    }
    // Sort the sections along the rail.
    let mut order: Vec<usize> = (0..sections.len()).collect();
    order.sort_by(|&a, &b| params[a].total_cmp(&params[b]));
    let is_point = |c: &Nc| c.points.iter().all(|p| dist(euclid(*p), euclid(c.points[0])) < 1e-12);
    for (k, &i) in order.iter().enumerate() {
        if is_point(&sections[i]) && k != 0 && k + 1 != order.len() {
            return Err("a point can only start or end the sweep".into());
        }
    }
    if wrap && sections.iter().any(is_point) {
        return Err("a closed sweep cannot have a point section".into());
    }
    let at: Vec<f64> = order.iter().map(|&i| params[i]).collect();
    // The range of the rail swept.
    let (ua, ub) = if wrap {
        (0.0, 1.0)
    } else if rail_closed {
        (0.0, *at.last().unwrap())
    } else {
        (0.0, 1.0)
    };
    if ub - ua < 1e-9 {
        return Err("the sections are all at the same place on the rail".into());
    }
    let frames = Rail::new(rail.clone(), o.style, wrap);
    // Sections in their frames, aligned and compatible.
    let mut locals: Vec<Nc> = order
        .iter()
        .zip(&at)
        .map(|(&i, &s)| {
            let f = frames.frame(s);
            Nc {
                degree: sections[i].degree,
                points: sections[i].points.iter().map(|p| f.local(*p)).collect(),
                knots: sections[i].knots.clone(),
            }
        })
        .collect();
    let curve_closed: Vec<bool> = locals.iter().map(|c| !is_point(c) && c.closed(1e-6)).collect();
    let all_closed = curve_closed.iter().zip(&locals).all(|(c, l)| *c || is_point(l));
    let any_closed = curve_closed.iter().any(|c| *c);
    if any_closed && !all_closed {
        return Err("the cross sections must be all closed or all open".into());
    }
    exact::align_sections(&mut locals, all_closed && any_closed);
    exact::make_compatible(&mut locals);
    // A point section takes its neighbour's weights (still the same point), so the shape
    // shrinks to it without bending: a circle to a point is a cone.
    for k in 0..locals.len() {
        if is_point(&locals[k]) && locals.len() > 1 {
            let other = if k == 0 { 1 } else { k - 1 };
            let ws: Vec<f64> = locals[other].points.iter().map(|q| q[3]).collect();
            let at = euclid(locals[k].points[0]);
            for (pt, w) in locals[k].points.iter_mut().zip(ws) {
                *pt = [at[0] * w, at[1] * w, at[2] * w, w];
            }
        }
    }
    let (p, sec_knots) = (locals[0].degree, locals[0].knots.clone());
    let m = locals.len();

    // The local shape at rail parameter u.
    let shape = |u: f64| -> Vec<H> {
        if m == 1 {
            return locals[0].points.clone();
        }
        let blend = |a: usize, b: usize, t: f64| -> Vec<H> {
            let t = if o.global_blend { t } else { smooth(t.clamp(0.0, 1.0)) };
            // Points and weights blended apart (a homogeneous blend would bend the shape
            // when the weights differ).
            locals[a]
                .points
                .iter()
                .zip(&locals[b].points)
                .map(|(x, y)| {
                    let (p, q) = (euclid(*x), euclid(*y));
                    let w = x[3] + (y[3] - x[3]) * t;
                    let e = add(p, scale(sub(q, p), t));
                    [e[0] * w, e[1] * w, e[2] * w, w]
                })
                .collect()
        };
        for k in 0..m - 1 {
            if u >= at[k] && u <= at[k + 1] {
                let span = (at[k + 1] - at[k]).max(1e-12);
                return blend(k, k + 1, (u - at[k]) / span);
            }
        }
        if wrap {
            // Between the last section and the first one again (at u + 1).
            let (last, first) = (at[m - 1], at[0] + 1.0);
            let uu = if u < at[0] { u + 1.0 } else { u };
            return blend(m - 1, 0, (uu - last) / (first - last).max(1e-12));
        }
        if u < at[0] { locals[0].points.clone() } else { locals[m - 1].points.clone() }
    };
    let placed = |u: f64| -> Vec<H> {
        let f = frames.frame(u);
        shape(u).iter().map(|l| f.world(*l)).collect()
    };
    let constant_shape = m == 1
        || (1..m).all(|k| {
            locals[k]
                .points
                .iter()
                .zip(&locals[0].points)
                .all(|(a, b)| dist(euclid(*a), euclid(*b)) < 1e-9 && (a[3] - b[3]).abs() < 1e-12)
        });

    // Exact cases.
    if constant_shape && is_line(&rail) {
        let rows = vec![placed(ua), placed(ub)];
        return Ok(Swept {
            rows,
            degree: [1, p],
            knots: [vec![0.0, 0.0, 1.0, 1.0], sec_knots],
            deviation: 0.0,
            stations: 2,
            exact: true,
        });
    }
    if constant_shape {
        if let Some((center, axis, _)) = circle_of(&rail) {
            let fits_style = match o.style {
                Style::Freeform => true,
                Style::Roadlike(d) => norm(cross(unit(d), axis)) < 1e-9,
            };
            if fits_style {
                // Angle swept from rail(ua) to rail(ub) about the axis, along the rail.
                let angle = if wrap {
                    TAU
                } else {
                    let (a, b) = (sub(rail.eval(ua), center), sub(rail.eval(ub), center));
                    let mut t = dot(cross(a, b), axis).atan2(dot(a, b));
                    if t <= 1e-12 {
                        t += TAU;
                    }
                    t
                };
                let (rows, ku) = revolve_rows(&placed(ua), center, axis, angle);
                return Ok(Swept {
                    stations: rows.len(),
                    rows,
                    degree: [2, p],
                    knots: [ku, sec_knots],
                    deviation: 0.0,
                    exact: true,
                });
            }
        }
    }

    // General case: stations refined until within tolerance.
    let mut initial: Vec<f64> = at.clone();
    initial.extend(rail.knots.iter().copied());
    fit_stations(&placed, ua, ub, wrap, initial, p, &sec_knots, o.tol, o.max_stations)
}

/// Fit a surface through sections `placed(u)` (homogeneous poles over `sec_knots`, degree
/// `p`) at stations along `[ua, ub]`, adding stations where the surface strays more than
/// `tol` from `placed`; `wrap`: periodic across (the section at `ub` is the one at `ua`).
pub(crate) fn fit_stations(
    placed: &dyn Fn(f64) -> Vec<H>,
    ua: f64,
    ub: f64,
    wrap: bool,
    initial: Vec<f64>,
    p: usize,
    sec_knots: &[f64],
    tol: f64,
    max_stations: usize,
) -> Result<Swept, String> {
    let mut stations: Vec<f64> = vec![ua, ub];
    stations.extend(initial.into_iter().filter(|u| *u > ua && *u < ub));
    let base = if wrap { 12 } else { 8 };
    stations.extend((1..base).map(|i| ua + (ub - ua) * i as f64 / base as f64));
    let tidy = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-7 * (ub - ua));
    };
    tidy(&mut stations);
    let width = placed(ua).len();
    let width_samples = (4 * width).clamp(16, 96);
    let mut best: Option<Swept> = None;
    loop {
        let used: Vec<f64> = if wrap { stations[..stations.len() - 1].to_vec() } else { stations.clone() };
        let secs: Vec<Nc> = used
            .iter()
            .map(|u| Nc { degree: p, points: placed(*u), knots: sec_knots.to_vec() })
            .collect();
        let ts: Vec<f64> = stations.iter().map(|u| (u - ua) / (ub - ua)).collect();
        let Some((rows, ku)) = exact::interpolate_across(&secs, &ts, 3, wrap) else {
            return best.ok_or_else(|| "could not fit the sweep".to_string());
        };
        let q = 3.min(secs.len() - 1).max(1);
        // Check the middle of every interval.
        let mut worst = 0.0f64;
        let mut split = Vec::new();
        for w in stations.windows(2) {
            let u = (w[0] + w[1]) / 2.0;
            let t = (u - ua) / (ub - ua);
            let got = Nc { degree: p, points: iso(&rows, q, &ku, t), knots: sec_knots.to_vec() };
            let want = Nc { degree: p, points: placed(u), knots: sec_knots.to_vec() };
            let d = (0..=width_samples)
                .map(|i| {
                    let v = i as f64 / width_samples as f64;
                    dist(got.eval(v), want.eval(v))
                })
                .fold(0.0, f64::max);
            worst = worst.max(d);
            if d > tol {
                split.push(u);
            }
        }
        let done = Swept {
            stations: stations.len(),
            rows,
            degree: [q, p],
            knots: [ku, sec_knots.to_vec()],
            deviation: worst,
            exact: false,
        };
        if split.is_empty() || stations.len() + split.len() > max_stations {
            return Ok(done);
        }
        best = Some(done);
        stations.extend(split);
        tidy(&mut stations);
    }
}


/// Rhino 5 Sweep2 options.
#[derive(Debug, Clone)]
pub struct Options2 {
    /// Closed sweep (both rails closed, two or more sections).
    pub closed: bool,
    /// Rhino's *Maintain height*: sections scale across (rail to rail) but not in height.
    pub maintain_height: bool,
    pub tol: f64,
    pub max_stations: usize,
}

impl Default for Options2 {
    fn default() -> Self {
        Options2 { closed: false, maintain_height: false, tol: 0.001, max_stations: 400 }
    }
}

/// Arc length table of a curve: `(u, s)` with `s` normalised to `[0, 1]`.
struct Arc {
    u: Vec<f64>,
    s: Vec<f64>,
}

impl Arc {
    fn of(c: &Nc) -> Arc {
        let n = 1024;
        let mut u = Vec::with_capacity(n + 1);
        let mut s = Vec::with_capacity(n + 1);
        let mut prev = c.eval(0.0);
        let mut run = 0.0;
        for i in 0..=n {
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

/// Piecewise-linear lookup of `x` in increasing `xs`.
fn lookup(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    let n = xs.len();
    if x >= xs[n - 1] {
        return ys[n - 1];
    }
    let i = xs.partition_point(|v| *v <= x).max(1) - 1;
    let span = (xs[i + 1] - xs[i]).max(1e-300);
    ys[i] + (ys[i + 1] - ys[i]) * (x - xs[i]) / span
}

/// Where a section comes nearest a rail: (rail parameter, distance).
fn touch(rail: &Nc, s: &Nc) -> (f64, f64) {
    let mut best = (0.0, f64::MAX);
    for i in 0..=48 {
        let p = s.eval(i as f64 / 48.0);
        let u = rail.closest(p);
        let d = dist(rail.eval(u), p);
        if d < best.1 {
            best = (u, d);
        }
    }
    best
}

/// Rhino 5 Sweep2: sections (picked order) between two rails. The sections stretch from rail
/// to rail (scaled by the distance between the rails; height too unless `maintain_height`),
/// so the surface's two edges run on the rails.
pub fn sweep2(rail_a: &Nc, rail_b: &Nc, sections: &[Nc], o: &Options2) -> Result<Swept, String> {
    if sections.is_empty() {
        return Err("select the cross section curves".into());
    }
    let closed_a = rail_a.closed(1e-6);
    if closed_a != rail_b.closed(1e-6) {
        return Err("the rails must be both closed or both open".into());
    }
    let is_point = |c: &Nc| c.points.iter().all(|p| dist(euclid(*p), euclid(c.points[0])) < 1e-12);
    let (mut a, mut b) = (rail_a.clone(), rail_b.clone());
    let mut secs: Vec<Nc> = sections.to_vec();
    // Open sections run from rail A to rail B.
    for c in &mut secs {
        if !is_point(c) && !c.closed(1e-6) {
            let (s, e) = (c.start(), c.end());
            let da = |p: Vec3| dist(a.eval(a.closest(p)), p);
            if da(e) < da(s) {
                *c = c.reversed();
            }
        }
    }
    let wrap = closed_a && (secs.len() == 1 || o.closed);
    if o.closed && !closed_a {
        return Err("a closed sweep needs closed rails".into());
    }
    if closed_a {
        // Seams at the first section; rail B the same way round as rail A.
        a = a.with_seam(touch(&a, &secs[0]).0);
        b = b.with_seam(touch(&b, &secs[0]).0);
        let probe = a.eval(0.05);
        if dist(b.eval(b.closest(probe)), b.eval(0.95)) < dist(b.eval(b.closest(probe)), b.eval(0.05)) {
            b = b.reversed().with_seam(touch(&b.reversed(), &secs[0]).0);
        }
        if secs.len() >= 2 {
            let ps: Vec<f64> = secs.iter().map(|c| touch(&a, c).0).collect();
            if ps[1] > 0.5 && ps.windows(2).skip(1).all(|w| w[1] <= w[0] + 1e-9) {
                a = a.reversed().with_seam(touch(&a.reversed(), &secs[0]).0);
                b = b.reversed().with_seam(touch(&b.reversed(), &secs[0]).0);
            }
        }
    } else if dist(a.start(), b.start()) + dist(a.end(), b.end()) > dist(a.start(), b.end()) + dist(a.end(), b.start()) {
        b = b.reversed();
    }
    let (arc_a, arc_b) = (Arc::of(&a), Arc::of(&b));
    // Each section's place on both rails, as arc length fractions.
    let mut placed_on: Vec<(f64, f64, usize)> = secs
        .iter()
        .enumerate()
        .map(|(i, c)| (arc_a.s_of(touch(&a, c).0), arc_b.s_of(touch(&b, c).0), i))
        .collect();
    if closed_a {
        placed_on[0].0 = 0.0;
        placed_on[0].1 = 0.0;
        for p in placed_on.iter_mut().skip(1) {
            if p.0 < 1e-9 {
                p.0 = 1.0;
            }
            if p.1 < 1e-9 {
                p.1 = 1.0;
            }
        }
    }
    placed_on.sort_by(|x, y| x.0.total_cmp(&y.0));
    for (k, p) in placed_on.iter().enumerate() {
        if is_point(&secs[p.2]) && k != 0 && k + 1 != placed_on.len() {
            return Err("a point can only start or end the sweep".into());
        }
    }
    // Matched places along the rails (both increasing).
    let mut ta: Vec<f64> = Vec::new();
    let mut tb: Vec<f64> = Vec::new();
    if !closed_a {
        ta.push(0.0);
        tb.push(0.0);
    }
    for p in &placed_on {
        if ta.last().is_some_and(|l| p.0 <= *l + 1e-9) {
            continue;
        }
        ta.push(p.0);
        tb.push(p.1);
    }
    if !closed_a || wrap {
        if ta.last().is_none_or(|l| *l < 1.0 - 1e-9) {
            ta.push(1.0);
            tb.push(1.0);
        }
    }
    if tb.windows(2).any(|w| w[1] < w[0] - 1e-9) {
        return Err("the cross sections cross each other between the rails".into());
    }
    let (ua, ub) = (0.0, if wrap || !closed_a { 1.0 } else { placed_on.last().unwrap().0 });
    if ub - ua < 1e-9 {
        return Err("the sections are all at the same place on the rails".into());
    }
    // The frame at driving place t (arc length fraction on rail A): origin on rail A, x to
    // rail B (scaled by the gap), y across, z along.
    let frame = |t: f64| -> (Frame, f64) {
        let tb_ = lookup(&ta, &tb, t);
        let (u_a, u_b) = (arc_a.u_of(t), arc_b.u_of(tb_));
        let (pa, pb) = (a.eval(u_a), b.eval(u_b));
        let tan = unit(add(tangent(&a, u_a), tangent(&b, u_b)));
        let chord = sub(pb, pa);
        let w = norm(chord);
        let x = if w > 1e-12 { scale(chord, 1.0 / w) } else { perpendicular(tan) };
        let y0 = cross(tan, x);
        let y = if norm(y0) > 1e-12 { unit(y0) } else { perpendicular(x) };
        (Frame { o: pa, x, y, z: cross(x, y) }, w)
    };
    let to_local = |t: f64, c: &Nc| -> Nc {
        let (f, w) = frame(t);
        let (sx, sh) = (w.max(1e-12), if o.maintain_height { 1.0 } else { w.max(1e-12) });
        Nc {
            degree: c.degree,
            points: c
                .points
                .iter()
                .map(|p| {
                    let l = f.local(*p);
                    let q = euclid(l);
                    let wt = l[3];
                    [q[0] / sx * wt, q[1] / sh * wt, q[2] / sh * wt, wt]
                })
                .collect(),
            knots: c.knots.clone(),
        }
    };
    let at: Vec<f64> = placed_on.iter().map(|p| p.0).collect();
    let mut locals: Vec<Nc> = placed_on.iter().map(|p| to_local(p.0, &secs[p.2])).collect();
    let curve_closed: Vec<bool> = locals.iter().map(|c| !is_point(c) && c.closed(1e-6)).collect();
    let any_closed = curve_closed.iter().any(|c| *c);
    if any_closed && !curve_closed.iter().zip(&locals).all(|(c, l)| *c || is_point(l)) {
        return Err("the cross sections must be all closed or all open".into());
    }
    if any_closed {
        exact::align_sections(&mut locals, true);
    }
    exact::make_compatible(&mut locals);
    for k in 0..locals.len() {
        if is_point(&locals[k]) && locals.len() > 1 {
            let other = if k == 0 { 1 } else { k - 1 };
            let ws: Vec<f64> = locals[other].points.iter().map(|q| q[3]).collect();
            let pt = euclid(locals[k].points[0]);
            for (q, w) in locals[k].points.iter_mut().zip(ws) {
                *q = [pt[0] * w, pt[1] * w, pt[2] * w, w];
            }
        }
    }
    let (p, sec_knots) = (locals[0].degree, locals[0].knots.clone());
    let m = locals.len();
    let shape = |t: f64| -> Vec<H> {
        let blend = |i: usize, j: usize, f: f64| -> Vec<H> {
            let f = smooth(f.clamp(0.0, 1.0));
            locals[i]
                .points
                .iter()
                .zip(&locals[j].points)
                .map(|(x, y)| {
                    let (pp, qq) = (euclid(*x), euclid(*y));
                    let w = x[3] + (y[3] - x[3]) * f;
                    let e = add(pp, scale(sub(qq, pp), f));
                    [e[0] * w, e[1] * w, e[2] * w, w]
                })
                .collect()
        };
        if m == 1 {
            return locals[0].points.clone();
        }
        for k in 0..m - 1 {
            if t >= at[k] && t <= at[k + 1] {
                return blend(k, k + 1, (t - at[k]) / (at[k + 1] - at[k]).max(1e-12));
            }
        }
        if wrap {
            let tt = if t < at[0] { t + 1.0 } else { t };
            return blend(m - 1, 0, (tt - at[m - 1]) / (at[0] + 1.0 - at[m - 1]).max(1e-12));
        }
        if t < at[0] { locals[0].points.clone() } else { locals[m - 1].points.clone() }
    };
    let placed = |t: f64| -> Vec<H> {
        let (f, w) = frame(t);
        let (sx, sh) = (w, if o.maintain_height { 1.0 } else { w });
        shape(t)
            .iter()
            .map(|l| {
                let q = euclid(*l);
                f.world([q[0] * sx * l[3], q[1] * sh * l[3], q[2] * sh * l[3], l[3]])
            })
            .collect()
    };
    let mut initial = at.clone();
    initial.extend(ta.iter().copied());
    for k in a.knots.iter() {
        initial.push(arc_a.s_of(*k));
    }
    fit_stations(&placed, ua, ub, wrap, initial, p, &sec_knots, o.tol, o.max_stations)
}


/// Rhino 5 RailRevolve, exact: the profile turns about the axis and scales out from it so
/// the point where it meets the rail follows the rail; along the axis it moves with the rail
/// (or, with `scale_height`, scales from the axis origin). Around the axis this is the
/// complex product rail × profile / (meeting point), and along it a sum (or product), so the
/// surface is the tensor product of the two curves' poles: no sampling.
pub fn rail_revolve(profile: &Nc, rail: &Nc, origin: Vec3, axis_end: Vec3, scale_height: bool) -> Result<Swept, String> {
    let ax = sub(axis_end, origin);
    if norm(ax) < 1e-9 {
        return Err("the axis has no length".into());
    }
    let ax = unit(ax);
    // Where the profile meets the rail (nearest points).
    let mut best = (f64::MAX, profile.start(), 0.0);
    for i in 0..=96 {
        let p = profile.eval(i as f64 / 96.0);
        let u = rail.closest(p);
        let d = dist(rail.eval(u), p);
        if d < best.0 {
            best = (d, p, u);
        }
    }
    let (_, meet, rail_u) = best;
    let rel = sub(meet, origin);
    let h0 = dot(rel, ax);
    let r0 = sub(rel, scale(ax, h0));
    let rho0 = norm(r0);
    if rho0 < 1e-9 {
        return Err("the profile meets the rail on the axis".into());
    }
    if scale_height && h0.abs() < 1e-9 {
        return Err("ScaleHeight: the axis origin is level with the rail; pick it lower or higher".into());
    }
    let e1 = scale(r0, 1.0 / rho0);
    let e2 = cross(ax, e1);
    let coords = |p: Vec3| {
        let d = sub(p, origin);
        (dot(d, e1), dot(d, e2), dot(d, ax))
    };
    // Closed rail: its seam where the profile meets it (Rhino's advice, done for the user).
    let rail = if rail.closed(1e-6) { rail.with_seam(rail_u) } else { rail.clone() };
    let rows: Vec<Vec<H>> = rail
        .points
        .iter()
        .map(|rh| {
            let (ra, rb, rz) = coords(euclid(*rh));
            profile
                .points
                .iter()
                .map(|ph| {
                    let (pa, pb, pz) = coords(euclid(*ph));
                    let x = (ra * pa - rb * pb) / rho0;
                    let y = (ra * pb + rb * pa) / rho0;
                    let z = if scale_height { pz * rz / h0 } else { pz + rz - h0 };
                    let q = add(origin, add(scale(e1, x), add(scale(e2, y), scale(ax, z))));
                    let w = rh[3] * ph[3];
                    [q[0] * w, q[1] * w, q[2] * w, w]
                })
                .collect()
        })
        .collect();
    Ok(Swept {
        stations: rows.len(),
        rows,
        degree: [rail.degree, profile.degree],
        knots: [rail.knots.clone(), profile.knots.clone()],
        deviation: 0.0,
        exact: true,
    })
}


/// Matrix Sweep Multi Rail: a cross section through three or more rails (picked in order
/// round the section), matched along the rails by arc length. The surface runs through every
/// rail at its stations and stays within `tol` of the construction between them.
pub fn multi_rail(rails: &[Nc], closed_profile: bool, tol: f64) -> Result<Swept, String> {
    if rails.len() < 3 {
        return Err("select three or more rails".into());
    }
    let wrap = rails.iter().all(|r| r.closed(1e-6));
    if !wrap && rails.iter().any(|r| r.closed(1e-6)) {
        return Err("the rails must be all closed or all open".into());
    }
    let mut rs: Vec<Nc> = rails.to_vec();
    let first = rs[0].clone();
    for r in rs.iter_mut().skip(1) {
        if wrap {
            // Seam opposite the first rail's start, the same way round.
            let mut c = r.with_seam(r.closest(first.start()));
            let probe = first.eval(0.05);
            if dist(c.eval(c.closest(probe)), c.eval(0.95)) < dist(c.eval(c.closest(probe)), c.eval(0.05)) {
                c = c.reversed();
                c = c.with_seam(c.closest(first.start()));
            }
            *r = c;
        } else if dist(first.start(), r.start()) > dist(first.start(), r.end()) {
            *r = r.reversed();
        }
    }
    let arcs: Vec<Arc> = rs.iter().map(Arc::of).collect();
    let points_at = |t: f64| -> Vec<Vec3> { rs.iter().zip(&arcs).map(|(r, a)| r.eval(a.u_of(t))).collect() };
    // Section parameters: chord lengths, averaged along the rails, fixed for all stations
    // (so every station's section has the same knots).
    let n = rs.len();
    let mut chords = vec![0.0; if closed_profile { n } else { n - 1 }];
    for i in 0..=8 {
        let p = points_at(i as f64 / 8.0);
        for (k, c) in chords.iter_mut().enumerate() {
            *c += dist(p[k], p[(k + 1) % n]).max(1e-9);
        }
    }
    let total: f64 = chords.iter().sum();
    let mut params = vec![0.0];
    let mut run = 0.0;
    for c in &chords[..n - 1] {
        run += c / total;
        params.push(run);
    }
    if closed_profile {
        // Periodic: the parameters run on to where the first point comes round again.
        params.push(1.0);
    } else {
        *params.last_mut().unwrap() = 1.0;
    }
    let section = |t: f64| -> Option<(Vec<H>, Vec<f64>)> {
        let pcs: Vec<Nc> = points_at(t).into_iter().map(exact::point_curve).collect();
        let (rows, knots) = exact::interpolate_across(&pcs, &params, 3, closed_profile)?;
        Some((rows.iter().map(|r| r[0]).collect(), knots))
    };
    let (_, sec_knots) = section(0.0).ok_or("could not fit the cross section")?;
    let p = sec_knots.iter().take_while(|k| **k == sec_knots[0]).count() - 1;
    let placed = |t: f64| section(t).map(|s| s.0).unwrap_or_default();
    let mut initial = Vec::new();
    for (r, a) in rs.iter().zip(&arcs) {
        initial.extend(r.knots.iter().map(|k| a.s_of(*k)));
    }
    fit_stations(&placed, 0.0, 1.0, wrap, initial, p, &sec_knots, tol, 400)
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::Geo;

    fn circle(c: Vec3, x: Vec3, y: Vec3, r: f64) -> Nc {
        exact::arc(c, x, y, r, TAU)
    }

    fn srf_eval(s: &Swept, u: f64, v: f64) -> Vec3 {
        let row = iso(&s.rows, s.degree[0], &s.knots[0], u);
        Nc { degree: s.degree[1], points: row, knots: s.knots[1].clone() }.eval(v)
    }

    #[test]
    fn a_circle_round_a_circle_is_an_exact_torus() {
        // Rail: R10 in XY; section: r1 in the XZ plane at (10, 0, 0).
        let rail = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0);
        let sec = circle([10.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let s = sweep1(&rail, &[sec], &Options::default()).unwrap();
        assert!(s.exact);
        for i in 0..=60 {
            for j in 0..=40 {
                let p = srf_eval(&s, i as f64 / 60.0, j as f64 / 40.0);
                let ring = ((p[0] * p[0] + p[1] * p[1]).sqrt() - 10.0).hypot(p[2]);
                assert!((ring - 1.0).abs() < 1e-9, "{ring}");
            }
        }
    }

    #[test]
    fn a_section_along_a_line_is_an_exact_extrusion() {
        let rail = Nc::from_geo(&Geo::Line { start: [0.0; 3], end: [0.0, 0.0, 7.0] }).unwrap();
        let sec = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0);
        let s = sweep1(&rail, &[sec], &Options::default()).unwrap();
        assert!(s.exact && s.degree[0] == 1);
        let p = srf_eval(&s, 0.5, 0.3);
        assert!(((p[0].hypot(p[1])) - 2.0).abs() < 1e-12 && (p[2] - 3.5).abs() < 1e-12);
    }

    #[test]
    fn a_free_curve_rail_keeps_the_section_shape_within_tolerance() {
        // A wavy 3-D rail, one r1 circle: every cross-section of the sweep must stay a unit
        // circle around the rail (Freeform keeps the angle to the rail).
        let pts: Vec<Vec3> = (0..9).map(|i| {
            let t = i as f64;
            [t * 3.0, 4.0 * (t * 0.7).sin(), 2.0 * (t * 0.4).cos()]
        }).collect();
        let rail = Nc::from_geo(&crate::interpolate(&pts, 3, false).unwrap()).unwrap();
        let o = rail.eval(0.0);
        let t = tangent(&rail, 0.0);
        let x = perpendicular(t);
        let sec = circle(o, x, cross(t, x), 1.0);
        let opts = Options::default();
        let s = sweep1(&rail, &[sec], &opts).unwrap();
        assert!(s.deviation <= opts.tol, "{}", s.deviation);
        // Every surface point is at distance 1 from the rail (up to the tolerance).
        for i in 0..=80 {
            for j in 0..=16 {
                let p = srf_eval(&s, i as f64 / 80.0, j as f64 / 16.0);
                let d = dist(p, rail.eval(rail.closest(p)));
                assert!((d - 1.0).abs() < 3.0 * opts.tol, "{d} at {i},{j}");
            }
        }
    }

    #[test]
    fn two_sections_are_on_the_surface_and_closed_sweep_wraps() {
        // Ring rail with a small and a large circle opposite each other.
        let rail = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0);
        let a = circle([10.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let b = circle([-10.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        for closed in [false, true] {
            let o = Options { closed, ..Default::default() };
            let s = sweep1(&rail, &[a.clone(), b.clone()], &o).unwrap();
            assert!(s.deviation <= o.tol, "{}", s.deviation);
            // Both sections lie on the surface.
            for sec in [&a, &b] {
                for j in 0..=20 {
                    let p = sec.eval(j as f64 / 20.0);
                    let mut best = f64::MAX;
                    // Coarse scan, then refine around the best (u, v).
                    let (mut bu, mut bv) = (0.0, 0.0);
                    for i in 0..=120 {
                        for k in 0..=24 {
                            let (u, v) = (i as f64 / 120.0, k as f64 / 24.0);
                            let d = dist(p, srf_eval(&s, u, v));
                            if d < best {
                                (best, bu, bv) = (d, u, v);
                            }
                        }
                    }
                    for i in -20..=20 {
                        for k in -20..=20 {
                            let (u, v) = ((bu + i as f64 / 2400.0).clamp(0.0, 1.0), (bv + k as f64 / 480.0).clamp(0.0, 1.0));
                            best = best.min(dist(p, srf_eval(&s, u, v)));
                        }
                    }
                    assert!(best < 0.005, "{best}");
                }
            }
            let first = srf_eval(&s, 0.0, 0.25);
            let last = srf_eval(&s, 1.0, 0.25);
            if closed {
                assert!(dist(first, last) < 1e-9, "closed sweep must come back");
            } else {
                // Open on a closed rail: from the first section to the second (half way).
                assert!((last[0] + 10.0).abs() < 2.5 && (first[0] - 10.0).abs() < 1.5, "{first:?} {last:?}");
            }
        }
    }

    #[test]
    fn sweep_speed() {
        // Release timing (printed): ring with two sections, wavy rail with one.
        let rail = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0);
        let a = circle([10.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let b = circle([-10.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        let t = std::time::Instant::now();
        let s = sweep1(&rail, &[a, b], &Options { closed: true, ..Default::default() }).unwrap();
        eprintln!("ring 2 sections: {:?}, {} stations, dev {:.2e}", t.elapsed(), s.stations, s.deviation);
    }

    #[test]
    fn roadlike_top_keeps_sections_upright() {
        // A rising helix-like rail: Roadlike Top keeps frame x horizontal.
        let pts: Vec<Vec3> = (0..12).map(|i| {
            let a = i as f64 * 0.5;
            [8.0 * a.cos(), 8.0 * a.sin(), i as f64]
        }).collect();
        let rail = Nc::from_geo(&crate::interpolate(&pts, 3, false).unwrap()).unwrap();
        let f = Rail::new(rail.clone(), Style::Roadlike([0.0, 0.0, 1.0]), false);
        for i in 0..=20 {
            let fr = f.frame(i as f64 / 20.0);
            assert!(fr.x[2].abs() < 1e-9, "x must stay horizontal");
        }
    }

    #[test]
    fn sweep2_edges_run_on_the_rails_and_height_scales() {
        // Two rails: straight lines 4 apart (x = 0 and x = 4), rising along y; one half circle
        // from rail to rail (radius 2, standing up in z).
        let ra = Nc::from_geo(&Geo::Line { start: [0.0; 3], end: [0.0, 10.0, 0.0] }).unwrap();
        // Rail B widens: x from 4 to 8.
        let rb = Nc::from_geo(&Geo::Line { start: [4.0, 0.0, 0.0], end: [8.0, 10.0, 0.0] }).unwrap();
        let half = exact::arc([2.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0, std::f64::consts::PI);
        for keep in [false, true] {
            let o = Options2 { maintain_height: keep, ..Default::default() };
            let s = sweep2(&ra, &rb, &[half.clone()], &o).unwrap();
            assert!(s.deviation <= o.tol, "{}", s.deviation);
            for i in 0..=20 {
                let u = i as f64 / 20.0;
                let (e0, e1) = (srf_eval(&s, u, 0.0), srf_eval(&s, u, 1.0));
                assert!(dist(e0, ra.eval(ra.closest(e0))) < 1e-9, "edge on rail A");
                assert!(dist(e1, rb.eval(rb.closest(e1))) < o.tol, "edge on rail B");
                // Top of the arch: half the gap high (scaled) or 2 (maintain height).
                let top = srf_eval(&s, u, 0.5);
                let gap = dist(e0, e1);
                let want = if keep { 2.0 } else { gap / 2.0 };
                assert!((top[2] - want).abs() < 2.0 * o.tol, "{keep}: {} vs {want}", top[2]);
            }
        }
    }

    #[test]
    fn sweep2_ring_band_between_two_closed_rails() {
        // Inner rail R10, outer rail R12 (a flat band); arch sections from rail to rail.
        let ra = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0);
        let rb = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 12.0);
        let s1 = exact::arc([11.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, std::f64::consts::PI);
        // A taller arch opposite (maintain height off: scales with the same gap → same arch;
        // so use a flatter section there: a straight line).
        let s2 = Nc::from_geo(&Geo::Line { start: [-10.0, 0.0, 0.0], end: [-12.0, 0.0, 0.0] }).unwrap();
        // One section: all the way round.
        let s = sweep2(&ra, &rb, &[s1.clone()], &Options2::default()).unwrap();
        assert!(s.deviation <= 0.001);
        assert!(dist(srf_eval(&s, 0.0, 0.5), srf_eval(&s, 1.0, 0.5)) < 1e-9, "closed");
        for i in 0..=40 {
            let p = srf_eval(&s, i as f64 / 40.0, 0.5);
            assert!((p[2] - 1.0).abs() < 0.002 && (p[0].hypot(p[1]) - 11.0).abs() < 0.002, "{p:?}");
        }
        // Two sections, closed sweep: back to the first; open: from the first to the second.
        for closed in [true, false] {
            let o = Options2 { closed, ..Default::default() };
            let s = sweep2(&ra, &rb, &[s1.clone(), s2.clone()], &o).unwrap();
            assert!(s.deviation <= o.tol, "{}", s.deviation);
            let (first, last) = (srf_eval(&s, 0.0, 0.5), srf_eval(&s, 1.0, 0.5));
            assert!((first[2] - 1.0).abs() < 1e-6, "first section on the surface: {first:?}");
            if closed {
                assert!(dist(first, last) < 1e-9);
            } else {
                assert!(last[2].abs() < 1e-6 && (last[0] + 11.0).abs() < 1e-6, "{last:?}");
            }
            for i in 0..=40 {
                let u = i as f64 / 40.0;
                let (e0, e1) = (srf_eval(&s, u, 0.0), srf_eval(&s, u, 1.0));
                assert!((e0[0].hypot(e0[1]) - 10.0).abs() < 0.002 && (e1[0].hypot(e1[1]) - 12.0).abs() < 0.002);
            }
        }
    }

    #[test]
    fn rail_revolve_is_exact() {
        // A quarter circle R10 (XZ plane) round a circular rail R10: a hemisphere.
        let rail = circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0);
        let prof = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 10.0, std::f64::consts::FRAC_PI_2);
        let s = rail_revolve(&prof, &rail, [0.0; 3], [0.0, 0.0, 1.0], false).unwrap();
        for i in 0..=40 {
            for j in 0..=20 {
                let p = srf_eval(&s, i as f64 / 40.0, j as f64 / 20.0);
                assert!((norm(p) - 10.0).abs() < 1e-9, "{p:?}");
            }
        }
        // A wall round an ellipse rail (10 × 5): the profile's foot follows the rail exactly.
        let ellipse = Nc {
            degree: 2,
            points: circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0)
                .points
                .iter()
                .map(|h| [h[0] * 10.0, h[1] * 5.0, h[2], h[3]])
                .collect(),
            knots: circle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0).knots,
        };
        let wall = Nc::from_geo(&Geo::Line { start: [10.0, 0.0, 0.0], end: [10.0, 0.0, 3.0] }).unwrap();
        let s = rail_revolve(&wall, &ellipse, [0.0; 3], [0.0, 0.0, 1.0], false).unwrap();
        for i in 0..=40 {
            for j in 0..=6 {
                let p = srf_eval(&s, i as f64 / 40.0, j as f64 / 6.0);
                let e = (p[0] / 10.0).powi(2) + (p[1] / 5.0).powi(2);
                assert!((e - 1.0).abs() < 1e-9 && (p[2] - 3.0 * j as f64 / 6.0).abs() < 1e-9, "{p:?}");
            }
        }
        // ScaleHeight: a rail that rises takes the profile's height with it, from the origin.
        let tilted = Nc {
            degree: 2,
            points: ellipse.points.iter().map(|h| [h[0], h[1], (4.0 + 0.1 * h[0] / h[3]) * h[3], h[3]]).collect(),
            knots: ellipse.knots.clone(),
        };
        let wall2 = Nc::from_geo(&Geo::Line { start: [10.0, 0.0, 5.0], end: [10.0, 0.0, 0.0] }).unwrap();
        let s = rail_revolve(&wall2, &tilted, [0.0; 3], [0.0, 0.0, 1.0], true).unwrap();
        for i in 0..=40 {
            let top = srf_eval(&s, i as f64 / 40.0, 0.0);
            let foot = srf_eval(&s, i as f64 / 40.0, 1.0);
            assert!(dist(top, tilted.eval(tilted.closest(top))) < 1e-9, "the meeting point follows the rail");
            assert!(foot[2].abs() < 1e-9, "the profile's foot at the origin's height stays there");
        }
    }

    #[test]
    fn multi_rail_runs_through_every_rail() {
        // Four rails along a tube of radius 2 that widens to 3: the section is a circle
        // through them; the surface must hold every rail.
        let rails: Vec<Nc> = (0..4)
            .map(|k| {
                let a = k as f64 * std::f64::consts::FRAC_PI_2;
                let (c, s_) = (a.cos(), a.sin());
                let pts: Vec<Vec3> = (0..6).map(|i| {
                    let z = i as f64 * 2.0;
                    let r = 2.0 + z / 10.0;
                    [r * c, r * s_, z]
                }).collect();
                Nc::from_geo(&crate::interpolate(&pts, 3, false).unwrap()).unwrap()
            })
            .collect();
        let s = multi_rail(&rails, true, 0.001).unwrap();
        assert!(s.deviation <= 0.001, "{}", s.deviation);
        for r in &rails {
            for i in 0..=20 {
                let p = r.eval(i as f64 / 20.0);
                let mut best = f64::MAX;
                for a in 0..=200 {
                    for b in 0..=64 {
                        best = best.min(dist(p, srf_eval(&s, a as f64 / 200.0, b as f64 / 64.0)));
                    }
                }
                assert!(best < 0.05, "{best}");
            }
        }
        // The section closes on itself.
        assert!(dist(srf_eval(&s, 0.5, 0.0), srf_eval(&s, 0.5, 1.0)) < 1e-9);
    }
}
