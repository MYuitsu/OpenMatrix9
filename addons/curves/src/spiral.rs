// SPDX-License-Identifier: LGPL-2.1-or-later
//! Helix and Spiral as Matrix (Rhino) draw them:
//!
//! - Helix: `Start of axis ( Vertical AroundCurve )` › `End of axis` › `Radius and start point
//!   <1> ( Diameter Mode=Turns Turns=10 Pitch=1 ReverseTwist=No )`.
//! - Spiral: `Start of axis ( Flat Vertical AroundCurve )` › `End of axis` › `First radius and
//!   start point <1>` › `Second radius <1>` with the same options. `Flat` draws in the CPlane
//!   from a center (`Pitch` is then the gap between turns).
//! - `Mode=Turns` keeps the turn count (the pitch follows), `Mode=Pitch` the pitch.
//! - `Vertical`: the axis stands on the CPlane; `End of axis` gives its height.
//! - `AroundCurve`: `Select curve`: the helix (spiral) winds around the curve.
use crate::{load, number_of, on_plane, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, Want, add, cross, dist, dot, fmt, norm,
    parse_number, parse_point, scale, sub, unit,
};
use om9_addon_geom as g;
use om9_addon_geom::pick::CurveRef;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

#[derive(Serialize, Deserialize)]
struct State {
    /// "" (axis by two points), "flat", "vertical", "around".
    mode: String,
    points: Vec<Vec3>,
    diameter: bool,
    by_pitch: bool,
    turns: f64,
    pitch: f64,
    reverse: bool,
    r0: Option<f64>,
    /// The start direction picked with the first radius.
    start: Option<Vec3>,
    rail: Option<CurveRef>,
    last_radius: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            mode: String::new(),
            points: Vec::new(),
            diameter: false,
            by_pitch: false,
            turns: 10.0,
            pitch: 1.0,
            reverse: false,
            r0: None,
            start: None,
            rail: None,
            last_radius: 1.0,
        }
    }
}

const PER_TURN: usize = 16;
/// The most turns a helix or spiral may have (typed or from Axis length / Pitch).
const MAX_TURNS: f64 = 1000.0;
/// The most points one helix or spiral is interpolated through, for the preview and the
/// result alike: past it the command refuses instead of working without end.
const MAX_POINTS: usize = 20_000;
/// How close to the exact shape the curve must be (half Rhino's default 0.001).
const FIT_TOL: f64 = 5e-4;

/// The axis length the turns run along (the curve length around a curve; flat: the radius
/// change).
fn axis_length(st: &State, r0: f64, r1: f64) -> f64 {
    match (&st.rail, st.mode.as_str()) {
        (Some(c), _) => (0..256)
            .map(|i| dist(c.at(i as f64 / 256.0), c.at((i + 1) as f64 / 256.0)))
            .sum(),
        (None, "flat") => (r1 - r0).abs(),
        _ => st.points.get(1).map_or(0.0, |b| dist(st.points[0], *b)),
    }
}

/// The turn count: typed, or the axis length over the pitch. Refused when it is not a
/// finite number or more than [`MAX_TURNS`] (a pitch far too small for the axis).
fn turns(st: &State, r0: f64, r1: f64) -> Result<f64, String> {
    let t = if st.by_pitch {
        if !(st.pitch.is_finite() && st.pitch > 0.0) {
            return Err("Pitch must be a positive number".into());
        }
        (axis_length(st, r0, r1) / st.pitch).max(0.01)
    } else {
        st.turns
    };
    if !t.is_finite() || t <= 0.0 {
        return Err("The number of turns is not a valid number".into());
    }
    if t > MAX_TURNS {
        return Err(format!(
            "Too many turns for this curve ({}, at most {}); increase Pitch or reduce its length",
            fmt(t.round()),
            fmt(MAX_TURNS)
        ));
    }
    Ok(t)
}

/// Rotation-minimising frames along a curve: (point, tangent, normal) at `n + 1` stations.
fn frames(c: &CurveRef, n: usize, up: Vec3) -> Vec<(Vec3, Vec3, Vec3)> {
    let mut out = Vec::with_capacity(n + 1);
    let t0 = c.tangent(0.0);
    let mut r = sub(up, scale(t0, dot(up, t0)));
    if norm(r) < 1e-9 {
        r = cross(t0, [1.0, 0.0, 0.0]);
        if norm(r) < 1e-9 {
            r = cross(t0, [0.0, 1.0, 0.0]);
        }
    }
    let mut r = unit(r);
    let mut prev = (c.at(0.0), t0);
    out.push((prev.0, t0, r));
    for i in 1..=n {
        let s = i as f64 / n as f64;
        let (x1, t1) = (c.at(s), c.tangent(s));
        // Double reflection (Wang et al.).
        let v1 = sub(x1, prev.0);
        let c1 = dot(v1, v1);
        if c1 > 1e-24 {
            let rl = sub(r, scale(v1, 2.0 * dot(v1, r) / c1));
            let tl = sub(prev.1, scale(v1, 2.0 * dot(v1, prev.1) / c1));
            let v2 = sub(t1, tl);
            let c2 = dot(v2, v2);
            r = if c2 > 1e-24 {
                sub(rl, scale(v2, 2.0 * dot(v2, rl) / c2))
            } else {
                rl
            };
            r = unit(r);
        }
        out.push((x1, t1, r));
        prev = (x1, t1);
    }
    out
}

/// The helix/spiral for the radius point. `preview`: fitted to 0.01 mm only (what the
/// screen shows), so moving the pointer stays quick; the result is fitted to [`FIT_TOL`].
fn build(e: &Event, st: &State, helix: bool, r1_point: Vec3, preview: bool) -> Result<Geo, String> {
    let tol = if preview { 0.01 } else { FIT_TOL };
    let twist = if st.reverse { -1.0 } else { 1.0 };
    let radius_of = |p: Vec3, n: Vec3, base: Vec3| {
        let d = sub(p, base);
        norm(sub(d, scale(n, dot(d, n))))
    };
    // Around a curve.
    if let Some(c) = &st.rail {
        let first = st.start.unwrap_or(r1_point);
        let (_, q0) = c.closest(first);
        let r0 = st.r0.unwrap_or_else(|| dist(first, q0));
        let r1 = if helix {
            r0
        } else {
            dist(r1_point, c.closest(r1_point).1)
        };
        if !(r0.is_finite() && r1.is_finite()) || r0 < g::EPS {
            return Err("The radius must not be 0".into());
        }
        let tn = turns(st, r0, r1)?;
        // Points at `2·count` steps (frames at half steps too): the even ones are fitted,
        // the odd ones measure the fit.
        let dense = |count: usize| -> Vec<Vec3> {
            let fr = frames(c, 2 * count, sub(first, q0));
            fr.iter()
                .enumerate()
                .map(|(i, (x, t, nrm))| {
                    let f = i as f64 / (2 * count) as f64;
                    let ang = twist * f * tn * TAU;
                    let b = cross(*t, *nrm);
                    let r = r0 + (r1 - r0) * f;
                    add(*x, add(scale(*nrm, r * ang.cos()), scale(b, r * ang.sin())))
                })
                .collect()
        };
        return within_tolerance(tn, &dense, tol);
    }
    let a = *st.points.first().ok_or("Start of axis")?;
    let flat = st.mode == "flat";
    let b = if flat { a } else { *st.points.get(1).ok_or("End of axis")? };
    let n = if flat || norm(sub(b, a)) < g::EPS {
        e.plane.normal()
    } else {
        unit(sub(b, a))
    };
    let first = st.start.unwrap_or(r1_point);
    let r0 = st.r0.unwrap_or_else(|| radius_of(first, n, a));
    let r1 = if helix { r0 } else { radius_of(r1_point, n, a) };
    if !(r0.is_finite() && r1.is_finite()) || (r0 < g::EPS && r1 < g::EPS) {
        return Err("The radius must not be 0".into());
    }
    let tn = turns(st, r0, r1)?;
    let mut x = sub(sub(first, a), scale(n, dot(sub(first, a), n)));
    if norm(x) < g::EPS {
        x = e.plane.x;
    }
    let x = unit(x);
    let y = scale(cross(n, x), twist);
    let dense = |count: usize| -> Vec<Vec3> {
        (0..=2 * count)
            .map(|i| {
                let f = i as f64 / (2 * count) as f64;
                let ang = f * tn * TAU;
                let r = r0 + (r1 - r0) * f;
                add(
                    add(a, scale(sub(b, a), f)),
                    add(scale(x, r * ang.cos()), scale(y, r * ang.sin())),
                )
            })
            .collect()
    };
    within_tolerance(tn, &dense, tol)
}

/// Interpolate the exact helix/spiral points, adding points per turn until the curve is
/// within [`FIT_TOL`] of the exact shape everywhere. `dense(count)` gives the exact points at
/// `2·count` equal steps: the even ones are fitted, the odd ones measure the fit. The count
/// never passes [`MAX_POINTS`]; if the tolerance is not met by then, the shape is refused.
fn within_tolerance(tn: f64, dense: &dyn Fn(usize) -> Vec<Vec3>, tol: f64) -> Result<Geo, String> {
    let wanted = tn * PER_TURN as f64;
    if !wanted.is_finite() || wanted <= 0.0 || wanted > MAX_POINTS as f64 {
        return Err(format!("Too many points for this shape (at most {})", MAX_POINTS));
    }
    let mut count = (wanted.ceil() as usize).max(8);
    loop {
        let all = dense(count);
        if all.iter().any(|p| !p.iter().all(|v| v.is_finite())) {
            return Err("The shape has no valid points".into());
        }
        let pts: Vec<Vec3> = all.iter().step_by(2).copied().collect();
        let geo = g::interpolate(&pts, 3, false).ok_or("No curve through the points")?;
        if deviation(&geo, &all) <= tol {
            return Ok(geo);
        }
        if count >= MAX_POINTS {
            return Err(format!(
                "Cannot keep the shape within {} mm with at most {} points",
                fmt(tol),
                MAX_POINTS
            ));
        }
        count = (count * 3 / 2).max(count + 1).min(MAX_POINTS);
    }
}

/// The largest distance from the odd (measuring) points of `all` to the curve, found near
/// each one: the curve's parameter is chord length over the even points, so each odd point
/// is checked against a short stretch of the curve around its neighbours' parameters.
fn deviation(geo: &Geo, all: &[Vec3]) -> f64 {
    let pts: Vec<Vec3> = all.iter().step_by(2).copied().collect();
    let total: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut u = vec![0.0; pts.len()];
    for i in 1..pts.len() {
        u[i] = u[i - 1] + dist(pts[i - 1], pts[i]) / total;
    }
    let mut worst: f64 = 0.0;
    for (k, p) in all.iter().enumerate().skip(1).step_by(2) {
        let (i0, i1) = (k / 2, (k / 2 + 1).min(pts.len() - 1));
        let (a, b) = (u[i0], u[i1]);
        // 16 samples of the curve between the two neighbours, nearest segment distance.
        let s: Vec<Vec3> = (0..=16)
            .filter_map(|j| g::eval_geo(geo, a + (b - a) * j as f64 / 16.0))
            .collect();
        let d = s
            .windows(2)
            .map(|w| {
                let (x, y) = (w[0], w[1]);
                let v = sub(y, x);
                let l = dot(v, v);
                let t = if l > 0.0 { (dot(sub(*p, x), v) / l).clamp(0.0, 1.0) } else { 0.0 };
                dist(*p, add(x, scale(v, t)))
            })
            .fold(f64::INFINITY, f64::min);
        worst = worst.max(d);
    }
    worst
}

/// Points still needed for the axis.
fn axis_needed(st: &State) -> usize {
    match st.mode.as_str() {
        "flat" => 1,
        "around" => 0,
        _ => 2,
    }
}

fn ask(e: &Event, st: &State, helix: bool, hover: Option<Vec3>) -> Reply {
    let mut opts = Vec::new();
    let n = st.points.len();
    let radius_stage =
        (st.mode == "around" && st.rail.is_some()) || (st.mode != "around" && n >= axis_needed(st));
    let (prompt, want) = if st.mode == "around" && st.rail.is_none() {
        ("Select curve".to_owned(), Want::Objects)
    } else if !radius_stage {
        if n == 0 {
            if !helix {
                opts.push(OptionToken::action("Flat"));
            }
            opts.push(OptionToken::action("Vertical"));
            opts.push(OptionToken::action("AroundCurve"));
            let p = if st.mode == "flat" {
                "Center of spiral"
            } else {
                "Start of axis"
            };
            (p.to_owned(), Want::Point)
        } else {
            ("End of axis".to_owned(), Want::Point)
        }
    } else {
        opts.push(OptionToken::action(if st.diameter {
            "Radius"
        } else {
            "Diameter"
        }));
        opts.push(OptionToken::choice(
            "Mode",
            if st.by_pitch { "Pitch" } else { "Turns" },
            &["Turns", "Pitch"],
        ));
        opts.push(OptionToken::number("Turns", Some(st.turns)));
        opts.push(OptionToken::number("Pitch", Some(st.pitch)));
        opts.push(OptionToken::toggle("ReverseTwist", st.reverse));
        let v = fmt(if st.diameter {
            2.0 * st.last_radius
        } else {
            st.last_radius
        });
        let p = match (helix, st.r0.is_none(), st.diameter) {
            (true, _, false) => format!("Radius and start point <{v}>"),
            (true, _, true) => format!("Diameter and start point <{v}>"),
            (false, true, false) => format!("First radius and start point <{v}>"),
            (false, true, true) => format!("First diameter and start point <{v}>"),
            (false, false, false) => format!("Second radius <{v}>"),
            (false, false, true) => format!("Second diameter <{v}>"),
        };
        (p, Want::Number)
    };
    if n > 0 {
        opts.push(OptionToken::action("Undo"));
    }
    let preview: Vec<Geo> = match hover {
        // A shape refused (too many turns …) shows no preview; the click says why.
        Some(h) if radius_stage => build(e, st, helix, h, true).into_iter().collect(),
        Some(h) if n == 1 && st.mode != "flat" => vec![Geo::Line {
            start: st.points[0],
            end: vertical_end(e, st, h),
        }],
        _ => Vec::new(),
    };
    Reply::ask(prompt, save(st))
        .with_options(opts)
        .want(want)
        .preview(preview)
        .rubber(st.points.first().copied())
}

/// The axis end for a click: straight up from the start when `Vertical` (height = the
/// distance on the CPlane), else the click.
fn vertical_end(e: &Event, st: &State, p: Vec3) -> Vec3 {
    if st.mode == "vertical" {
        let a = st.points[0];
        add(a, scale(e.plane.normal(), dist(a, on_plane(e, a, p))))
    } else {
        p
    }
}

/// A radius typed: the point at that distance along the CPlane x from the axis.
fn radius_point(e: &Event, st: &State, r: f64) -> Option<Vec3> {
    if let Some(c) = &st.rail {
        let (q, t) = (c.at(0.0), c.tangent(0.0));
        let mut x = cross(t, e.plane.normal());
        if norm(x) < 1e-9 {
            x = cross(t, e.plane.x);
        }
        return Some(add(q, scale(unit(x), r)));
    }
    let a = *st.points.first()?;
    let n = match (st.mode.as_str(), st.points.get(1)) {
        ("flat", _) | (_, None) => e.plane.normal(),
        (_, Some(b)) if dist(a, *b) > g::EPS => unit(sub(*b, a)),
        _ => e.plane.normal(),
    };
    let x = st.start.map(|s| sub(s, a)).unwrap_or(e.plane.x);
    let mut x = sub(x, scale(n, dot(x, n)));
    if norm(x) < 1e-9 {
        x = cross(n, e.plane.y);
    }
    Some(add(a, scale(unit(x), r)))
}

pub fn step(e: &Event, helix: bool) -> Reply {
    let mut st: State = load(&e.state);
    let label = if helix { "Helix" } else { "Spiral" };
    let radius_stage = |st: &State| {
        (st.mode == "around" && st.rail.is_some())
            || (st.mode != "around" && st.points.len() >= axis_needed(st))
    };
    // A radius given (click or typed): the first one is remembered for spirals.
    let radius_given = |st: &mut State, p: Vec3| -> Reply {
        if !helix && st.r0.is_none() {
            let r = match &st.rail {
                Some(c) => dist(p, c.closest(p).1),
                None => {
                    let a = st.points[0];
                    let n = match (st.mode.as_str(), st.points.get(1)) {
                        ("flat", _) | (_, None) => e.plane.normal(),
                        (_, Some(b)) => unit(sub(*b, a)),
                    };
                    let d = sub(p, a);
                    norm(sub(d, scale(n, dot(d, n))))
                }
            };
            st.r0 = Some(r);
            st.start = Some(p);
            st.last_radius = r;
            return ask(e, st, helix, None);
        }
        if helix {
            st.start = Some(p);
        }
        match build(e, st, helix, p, false) {
            Ok(gm) => {
                let n = turns(st, 1.0, 1.0).map(fmt).unwrap_or_default();
                Reply::finish(vec![gm]).say(format!("{label}: {n} turns"))
            }
            Err(why) => ask(e, st, helix, None).refuse(why),
        }
    };
    match &e.input {
        Input::Hover { point } => ask(e, &st, helix, Some(*point)),
        Input::Option { name, value } => {
            match name.as_str() {
                "Flat" if !helix => {
                    st.mode = "flat".into();
                    st.points.clear();
                }
                "Vertical" => {
                    st.mode = "vertical".into();
                    st.points.clear();
                }
                "AroundCurve" => {
                    st.mode = "around".into();
                    st.points.clear();
                    st.rail = None;
                }
                "Diameter" => st.diameter = true,
                "Radius" => st.diameter = false,
                "Mode" => {
                    st.by_pitch = match value.as_deref() {
                        Some(v) => v.eq_ignore_ascii_case("pitch"),
                        None => !st.by_pitch,
                    }
                }
                "Turns" => match number_of(value) {
                    Some(t) if t > 0.0 && t <= MAX_TURNS => {
                        st.turns = t;
                        st.by_pitch = false;
                    }
                    _ => {
                        return ask(e, &st, helix, None)
                            .refuse(format!("Turns must be more than 0 and at most {}", fmt(MAX_TURNS)));
                    }
                },
                "Pitch" => match number_of(value) {
                    Some(p) if p > 0.0 => {
                        st.pitch = p;
                        st.by_pitch = true;
                    }
                    _ => return ask(e, &st, helix, None).refuse("Pitch must be positive"),
                },
                "ReverseTwist" => st.reverse = yes(value, st.reverse),
                "Undo" => {
                    st.points.pop();
                }
                other => return ask(e, &st, helix, None).refuse(format!("Unknown option {other}")),
            }
            ask(e, &st, helix, None)
        }
        Input::Selection if st.mode == "around" && st.rail.is_none() => {
            if let Some((c, _)) = CurveRef::picked(e, &[]) {
                st.rail = Some(c);
            }
            ask(e, &st, helix, None)
        }
        Input::Point { point } => {
            if st.mode == "around" && st.rail.is_none() {
                return ask(e, &st, helix, None);
            }
            if !radius_stage(&st) {
                let p = if st.points.len() == 1 {
                    vertical_end(e, &st, *point)
                } else {
                    *point
                };
                st.points.push(p);
                return ask(e, &st, helix, None);
            }
            radius_given(&mut st, *point)
        }
        Input::Text { text } => {
            if radius_stage(&st) {
                return match parse_number(text) {
                    Some(v) if v > 0.0 => {
                        let r = if st.diameter { v / 2.0 } else { v };
                        let Some(p) = radius_point(e, &st, r) else {
                            return ask(e, &st, helix, None);
                        };
                        st.last_radius = r;
                        radius_given(&mut st, p)
                    }
                    _ => ask(e, &st, helix, None).refuse(format!("Not a radius: {text}")),
                };
            }
            // Vertical: a number at End of axis is the height.
            if st.points.len() == 1
                && let Some(h) = parse_number(text)
            {
                let a = st.points[0];
                st.points.push(add(a, scale(e.plane.normal(), h)));
                return ask(e, &st, helix, None);
            }
            match parse_point(text, &e.plane, st.points.last().copied()) {
                Some(p) => {
                    st.points.push(p);
                    ask(e, &st, helix, None)
                }
                None => ask(e, &st, helix, None).refuse(format!("Not a point or length: {text}")),
            }
        }
        Input::Enter if radius_stage(&st) => {
            let r = st.last_radius;
            match radius_point(e, &st, r) {
                Some(p) => radius_given(&mut st, p),
                None => ask(e, &st, helix, None),
            }
        }
        Input::Enter if !helix && st.points.len() == 1 && st.mode.is_empty() => {
            // Enter at End of axis: a flat spiral in the CPlane.
            st.mode = "flat".into();
            ask(e, &st, helix, None)
        }
        Input::Enter => Reply::finish(Vec::new()),
        Input::Undo => {
            st.points.pop();
            ask(e, &st, helix, None)
        }
        _ => ask(e, &st, helix, None),
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    #[test]
    fn rejects_excessive_and_nonfinite_turns_before_sampling() {
        for turns in [f64::INFINITY, f64::NAN, 0.0, -1.0, 10_000_000.0] {
            let result = within_tolerance(turns, &|_| panic!("must not allocate samples"), FIT_TOL);
            assert!(result.is_err(), "{turns}");
        }
    }

    #[test]
    fn refuses_when_sample_budget_cannot_meet_tolerance() {
        let result = within_tolerance(
            0.01,
            &|count| {
                (0..=2 * count)
                    .map(|i| [i as f64, if i % 2 == 0 { 0.0 } else { 100.0 }, 0.0])
                    .collect()
            },
            FIT_TOL,
        );
        assert!(result.is_err_and(|why| why.starts_with("Cannot keep the shape within 0.0005 mm")));
    }
}
