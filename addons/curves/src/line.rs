// SPDX-License-Identifier: LGPL-2.1-or-later
//! Single Line / `Line` (Matrix Curve #2), with every Matrix 9 variant:
//!
//! `Start of line ( BothSides Normal Angled Vertical FourPoint Bisector Perpendicular
//! Tangent Extension )`, then `End of line ( BothSides )`.
//!
//! - `BothSides`: the first pick is the middle of the line.
//! - `Normal`: `Select surface or polysurface for normal line ( IgnoreTrims )`,
//!   `Start of line on surface`, then the line follows the surface normal.
//! - `Angled`: `Start of base line`, `End of base line`, `Pivot angle`, `End of line`.
//! - `Vertical`: the line follows the CPlane normal.
//! - `FourPoint`: two points for the direction, then the line's start and end.
//! - `Bisector`: `Bisector start` (vertex), `Angle start`, `Angle end`, `End of line`.
//! - `Perpendicular ( Point PointOnCurve 2Curves )`: from a curve (default /
//!   `PointOnCurve`), to a curve from a point (`Point`), or between two curves (`2Curves`).
//! - `Tangent ( Point 2Curves )`: along the tangent at a curve point (default), from a point
//!   touching a curve (`Point`), or touching two curves (`2Curves`).
//! - `Extension`: select a curve; the line continues its nearest end.
//!
//! Curves and surfaces are taken from the selection (select, or have them selected before).
//! Where Matrix uses the pick location on the curve, OM9 asks for one more pick near the
//! wanted point. A typed number is the line's length.
use crate::pick::{CurveRef, SurfRef};
use crate::{load, number_of, on_plane, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, Want, add, cross, dist, dot, fmt, norm,
    parse_number, scale, sub, unit,
};
use om9_addon_geom as g;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    /// "", "normal", "angled", "vertical", "fourpoint", "bisector", "perp", "perp_point",
    /// "perp_2", "tan", "tan_point", "tan_2", "extension".
    pub mode: String,
    pub both_sides: bool,
    pub ignore_trims: bool,
    pub length: Option<f64>,
    pub points: Vec<Vec3>,
    pub angle: Option<f64>,
    pub curves: Vec<CurveRef>,
    /// Where each curve was clicked (its parameter), when it was picked by a click.
    #[serde(default)]
    pub picks: Vec<Option<f64>>,
    pub surface: Option<SurfRef>,
    /// Direction the end follows (unit), once known.
    pub axis: Option<Vec3>,
    /// Start of the line (or its middle with BothSides), once known.
    pub start: Option<Vec3>,
}

impl State {
    /// Curves still to select for this mode.
    fn curves_needed(&self) -> usize {
        match self.mode.as_str() {
            "perp" | "tan" | "extension" | "perp_point" | "tan_point" => 1usize,
            "perp_2" | "tan_2" => 2,
            _ => 0,
        }
        .saturating_sub(self.curves.len())
    }

    /// (start, end) for the end point or cursor `p` once the start and axis are known.
    fn segment(&self, e: &Event, p: Vec3) -> Option<(Vec3, Vec3)> {
        let s = self.start?;
        let end = match self.axis {
            Some(d) => {
                let mut l = dot(sub(p, s), d);
                if self.mode == "vertical" && l.abs() <= g::EPS {
                    // A point on the CPlane: the height is its distance from the start.
                    l = dist(on_plane(e, s, p), s);
                }
                if let Some(len) = self.length {
                    l = if l < 0.0 { -len } else { len };
                }
                add(s, scale(d, l))
            }
            None => match self.length {
                Some(len) if dist(p, s) > g::EPS => add(s, scale(unit(sub(p, s)), len)),
                _ => p,
            },
        };
        if dist(s, end) < g::EPS {
            return None;
        }
        let start = if self.both_sides {
            sub(scale(s, 2.0), end)
        } else {
            s
        };
        Some((start, end))
    }

    fn reply(&self, e: &Event, hover: Option<Vec3>) -> Reply {
        let n = self.points.len();
        let mut opts = Vec::new();
        let mut want = Want::Point;
        let mut preview = Vec::new();
        let prompt: String = if self.mode == "normal" && self.surface.is_none() {
            want = Want::Objects;
            opts.push(OptionToken::toggle("IgnoreTrims", self.ignore_trims));
            "Select surface or polysurface for normal line".into()
        } else if self.curves_needed() > 0 {
            want = Want::Objects;
            match (self.mode.as_str(), self.curves.len()) {
                ("perp_2", 0) => "Select first curve near perpendicular point",
                ("perp_2", _) => "Select second curve near perpendicular point",
                ("tan_2", 0) => "Select first curve near tangent point",
                ("tan_2", _) => "Select second curve near tangent point",
                ("extension", _) => "Select curve near end to extend",
                ("perp_point", _) => "Select curve to draw the line perpendicular to",
                ("tan_point", _) => "Select curve to draw the line tangent to",
                _ => "Select curve for the line",
            }
            .into()
        } else if self.start.is_some() {
            opts.push(OptionToken::toggle("BothSides", self.both_sides));
            if let Some((a, b)) = hover.and_then(|h| self.segment(e, h)) {
                preview.push(Geo::Line { start: a, end: b });
            }
            "End of line".into()
        } else {
            match (self.mode.as_str(), n) {
                ("", 0) => {
                    for o in [
                        "BothSides",
                        "Normal",
                        "Angled",
                        "Vertical",
                        "FourPoint",
                        "Bisector",
                        "Perpendicular",
                        "Tangent",
                        "Extension",
                    ] {
                        if o == "BothSides" {
                            opts.push(OptionToken::toggle(o, self.both_sides));
                        } else {
                            opts.push(OptionToken::action(o));
                        }
                    }
                    if self.both_sides {
                        "Middle of line"
                    } else {
                        "Start of line"
                    }
                    .into()
                }
                ("perp", _) | ("tan", _) => {
                    opts.push(OptionToken::toggle("BothSides", self.both_sides));
                    if self.mode == "perp" {
                        opts.extend([
                            OptionToken::action("Point"),
                            OptionToken::action("PointOnCurve"),
                            OptionToken::action("2Curves"),
                        ]);
                    } else {
                        opts.extend([OptionToken::action("Point"), OptionToken::action("2Curves")]);
                    }
                    if self.curves.is_empty() {
                        want = Want::Objects;
                        "Select curve for the line".into()
                    } else {
                        "Start of line on curve".into()
                    }
                }
                ("perp_point" | "tan_point", 0) => "Start of line".into(),
                ("perp_point", _) => "End of line near the perpendicular point".into(),
                ("tan_point", _) => "End of line near the tangent point".into(),
                ("perp_2" | "tan_2", _) => "Point near the first curve's touch point".into(),
                ("extension", _) => "End of line".into(),
                ("normal", _) => "Start of line on surface".into(),
                ("vertical", _) => "Start of line".into(),
                ("angled", 0) => "Start of base line".into(),
                ("angled", 1) => {
                    want = Want::Number;
                    "End of base line".into()
                }
                ("angled", _) => {
                    want = Want::Number;
                    "Pivot angle".into()
                }
                ("fourpoint", 0) => "Start of reference line".into(),
                ("fourpoint", 1) => "End of reference line".into(),
                ("fourpoint", _) => "Start of line".into(),
                ("bisector", 0) => "Bisector start".into(),
                ("bisector", 1) => "Angle start".into(),
                ("bisector", _) => "Angle end".into(),
                _ => "Start of line".into(),
            }
        };
        if let (Some(h), None) = (hover, self.start)
            && let Some(&last) = self.points.last()
        {
            preview.push(Geo::Line {
                start: last,
                end: h,
            });
        }
        if self.mode == "extension"
            && let (Some(h), Some(c)) = (hover, self.curves.first())
            && let Some((a, b)) = extension(c, h, self.length)
        {
            preview.push(Geo::Line { start: a, end: b });
        }
        Reply::ask(prompt, save(self))
            .with_options(opts)
            .want(want)
            .preview(preview)
    }

    fn done(&self, a: Vec3, b: Vec3) -> Reply {
        Reply::finish(vec![Geo::Line { start: a, end: b }])
            .say(format!("Line: length {} mm", fmt(dist(a, b))))
    }
}

/// Line continuing the end of `c` nearest `p`, to `p` projected on the end tangent.
fn extension(c: &CurveRef, p: Vec3, length: Option<f64>) -> Option<(Vec3, Vec3)> {
    let [a, b] = c.ends();
    let (s, d) = if dist(a.0, p) <= dist(b.0, p) { a } else { b };
    let l = length.unwrap_or_else(|| dot(sub(p, s), d).max(0.0));
    (l > g::EPS).then(|| (s, add(s, scale(d, l))))
}

/// Closest points between two curves (common perpendicular): coarse grid, then coordinate
/// descent with golden sections.
fn closest_pair(a: &CurveRef, b: &CurveRef) -> (Vec3, Vec3) {
    let n = 128;
    let (mut ta, mut tb, mut best) = (0.0, 0.0, f64::INFINITY);
    for i in 0..=n {
        for j in 0..=n {
            let (u, v) = (i as f64 / n as f64, j as f64 / n as f64);
            let d = dist(a.at(u), b.at(v));
            if d < best {
                (ta, tb, best) = (u, v, d);
            }
        }
    }
    for _ in 0..30 {
        ta = a.closest(b.at(tb)).0;
        tb = b.closest(a.at(ta)).0;
    }
    (a.at(ta), b.at(tb))
}

/// Line touching both curves near the parameters `ta`, `tb` (in the plane with normal `n`).
fn common_tangent(
    a: &CurveRef,
    b: &CurveRef,
    mut ta: f64,
    mut tb: f64,
    n: Vec3,
) -> Option<(Vec3, Vec3)> {
    let f = |ta: f64, tb: f64| -> [f64; 2] {
        let d = sub(b.at(tb), a.at(ta));
        let d = if norm(d) < 1e-12 {
            [1.0, 0.0, 0.0]
        } else {
            unit(d)
        };
        [
            dot(cross(d, a.tangent(ta)), n),
            dot(cross(d, b.tangent(tb)), n),
        ]
    };
    for _ in 0..60 {
        let f0 = f(ta, tb);
        if f0[0].abs() + f0[1].abs() < 1e-12 {
            break;
        }
        let h = 1e-6;
        let fa = f(ta + h, tb);
        let fb = f(ta, tb + h);
        let j = [
            [(fa[0] - f0[0]) / h, (fb[0] - f0[0]) / h],
            [(fa[1] - f0[1]) / h, (fb[1] - f0[1]) / h],
        ];
        let det = j[0][0] * j[1][1] - j[0][1] * j[1][0];
        if det.abs() < 1e-14 {
            return None;
        }
        let da = (f0[0] * j[1][1] - f0[1] * j[0][1]) / det;
        let db = (j[0][0] * f0[1] - j[1][0] * f0[0]) / det;
        ta = (ta - da.clamp(-0.1, 0.1)).clamp(0.0, 1.0);
        tb = (tb - db.clamp(-0.1, 0.1)).clamp(0.0, 1.0);
    }
    let f0 = f(ta, tb);
    (f0[0].abs() + f0[1].abs() < 1e-6).then(|| (a.at(ta), b.at(tb)))
}

/// The parameter in `ts` whose point is nearest `p`.
fn nearest(c: &CurveRef, ts: &[f64], p: Vec3) -> Option<f64> {
    ts.iter()
        .copied()
        .min_by(|x, y| dist(c.at(*x), p).total_cmp(&dist(c.at(*y), p)))
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    let n_plane = e.plane.normal();
    match &e.input {
        Input::Start | Input::Selection => {
            take_selection(&mut st, e);
            if let Some(r) = st.settle_picks(e) {
                return r;
            }
            st.reply(e, None)
        }
        Input::Hover { point } => st.reply(e, Some(*point)),
        Input::Point { point } => {
            let p = *point;
            if st.start.is_some() {
                return match st.segment(e, p) {
                    Some((a, b)) => st.done(a, b),
                    None => st.reply(e, None).refuse("End point equals the start"),
                };
            }
            if st.curves_needed() > 0 || (st.mode == "normal" && st.surface.is_none()) {
                return st.reply(e, None).refuse("Select the object first");
            }
            match st.mode.as_str() {
                "" | "fourpoint" if st.mode.is_empty() || st.points.len() == 2 => {
                    if st.mode == "fourpoint" {
                        let d = unit(sub(st.points[1], st.points[0]));
                        st.axis = Some(d);
                    }
                    st.start = Some(p);
                }
                "vertical" => {
                    st.start = Some(p);
                    st.axis = Some(n_plane);
                }
                "normal" => {
                    let Some((q, nrm)) = st.surface.as_ref().and_then(|s| s.closest(p)) else {
                        return st.reply(e, None).refuse("The surface has no point there");
                    };
                    st.start = Some(q);
                    st.axis = Some(nrm);
                }
                "perp" | "tan" => {
                    let c = &st.curves[0];
                    let (t, q) = c.closest(p);
                    let tan = c.tangent(t);
                    st.start = Some(q);
                    if st.mode == "tan" {
                        st.axis = Some(tan);
                    } else {
                        // Perpendicular to the curve in the CPlane (or the curve's plane).
                        let side = cross(n_plane, tan);
                        st.axis = Some(if norm(side) > 1e-9 {
                            unit(side)
                        } else {
                            unit(cross(tan, e.plane.x))
                        });
                    }
                }
                "extension" => {
                    return match extension(&st.curves[0], p, st.length) {
                        Some((a, b)) => st.done(a, b),
                        None => st.reply(e, None).refuse("Pick beyond the end of the curve"),
                    };
                }
                "perp_point" | "tan_point" if st.points.is_empty() => {
                    st.points.push(p);
                }
                "perp_point" | "tan_point" => {
                    let s = st.points[0];
                    let c = &st.curves[0];
                    let ts = if st.mode == "perp_point" {
                        c.feet(s)
                    } else {
                        c.tangent_points(s, n_plane)
                    };
                    return match nearest(c, &ts, p) {
                        Some(t) => st.done(s, c.at(t)),
                        None => st.reply(e, None).refuse(if st.mode == "perp_point" {
                            "No perpendicular from that point to the curve"
                        } else {
                            "No tangent from that point to the curve"
                        }),
                    };
                }
                "perp_2" => {
                    let (a, b) = closest_pair(&st.curves[0], &st.curves[1]);
                    if dist(a, b) < g::EPS {
                        return st
                            .reply(e, None)
                            .refuse("The curves touch: no perpendicular line");
                    }
                    return st.done(a, b);
                }
                "tan_2" if st.points.is_empty() => {
                    st.points.push(p);
                }
                "tan_2" => {
                    let (a, b) = (&st.curves[0], &st.curves[1]);
                    let ta = a.closest(st.points[0]).0;
                    let tb = b.closest(p).0;
                    return match common_tangent(a, b, ta, tb, n_plane) {
                        Some((x, y)) if dist(x, y) > g::EPS => st.done(x, y),
                        _ => st
                            .reply(e, None)
                            .refuse("No line touches both curves near those points"),
                    };
                }
                "angled" if st.points.len() < 2 => st.points.push(p),
                "bisector" if st.points.len() < 3 => {
                    st.points.push(p);
                    if st.points.len() == 3 {
                        let v = st.points[0];
                        let a = unit(sub(st.points[1], v));
                        let b = unit(sub(st.points[2], v));
                        let bis = add(a, b);
                        if norm(bis) < 1e-9 {
                            st.points.pop();
                            return st.reply(e, None).refuse("The angle is 180°: no bisector");
                        }
                        st.start = Some(v);
                        st.axis = Some(unit(bis));
                    }
                }
                "fourpoint" => st.points.push(p),
                "angled" => return st.reply(e, None).refuse("Type the pivot angle"),
                _ => st.points.push(p),
            }
            st.reply(e, None)
        }
        Input::Option { name, value } => {
            let fresh = st.start.is_none() && st.points.is_empty();
            match name.as_str() {
                "BothSides" => st.both_sides = yes(value, st.both_sides),
                "IgnoreTrims" => st.ignore_trims = yes(value, st.ignore_trims),
                "Length" => match number_of(value) {
                    Some(v) if v > 0.0 => st.length = Some(v),
                    _ => return st.reply(e, None).refuse("Length must be a positive number"),
                },
                "Normal" if fresh && st.mode.is_empty() => st.mode = "normal".into(),
                "Angled" if fresh && st.mode.is_empty() => st.mode = "angled".into(),
                "Vertical" if fresh && st.mode.is_empty() => st.mode = "vertical".into(),
                "FourPoint" if fresh && st.mode.is_empty() => st.mode = "fourpoint".into(),
                "Bisector" if fresh && st.mode.is_empty() => st.mode = "bisector".into(),
                "Perpendicular" if fresh && st.mode.is_empty() => st.mode = "perp".into(),
                "Tangent" if fresh && st.mode.is_empty() => st.mode = "tan".into(),
                "Extension" if fresh && st.mode.is_empty() => st.mode = "extension".into(),
                "PointOnCurve" if fresh && st.mode == "perp" => {}
                "Point" if fresh && matches!(st.mode.as_str(), "perp" | "tan") => {
                    st.mode = format!("{}_point", st.mode);
                    st.curves.clear();
                }
                "2Curves" if fresh && matches!(st.mode.as_str(), "perp" | "tan") => {
                    st.mode = format!("{}_2", st.mode);
                    st.curves.clear();
                }
                _ => return st.reply(e, None).refuse(format!("Unknown option {name}")),
            }
            take_selection(&mut st, e);
            st.reply(e, None)
        }
        Input::Text { text } => match parse_number(text) {
            Some(v) if st.mode == "angled" && st.points.len() == 2 && st.start.is_none() => {
                let (s, t) = (st.points[0], st.points[1]);
                let d = unit(sub(t, s));
                let a = v.to_radians();
                let side = unit(cross(n_plane, d));
                st.angle = Some(v);
                st.start = Some(s);
                st.axis = Some(add(scale(d, a.cos()), scale(side, a.sin())));
                st.reply(e, None)
            }
            Some(v) if v > 0.0 => {
                st.length = Some(v);
                st.reply(e, None)
            }
            _ => st
                .reply(e, None)
                .refuse(format!("Not a point or length: {text}")),
        },
        Input::Undo => {
            if st.start.take().is_some() {
                st.axis = None;
                if st.mode == "bisector" {
                    st.points.pop();
                }
            } else {
                st.points.pop();
            }
            st.reply(e, None)
        }
        Input::Enter => {
            // A start and a typed length, then Enter: the line goes along its fixed direction,
            // else along the CPlane x axis (click then type a number, as in Matrix).
            if let (Some(st_pt), Some(_)) = (st.start, st.length) {
                let d = st.axis.unwrap_or(e.plane.x);
                if let Some((a, b)) = st.segment(e, add(st_pt, d)) {
                    return st.done(a, b);
                }
            }
            Reply::finish(vec![])
        }
    }
}

/// Curves / surfaces the current step waits for, from the selection.
fn take_selection(st: &mut State, e: &Event) {
    if st.mode == "normal" && st.surface.is_none() {
        st.surface = e.selection.iter().find_map(SurfRef::of);
    }
    while st.curves_needed() > 0 {
        let taken: Vec<String> = st.curves.iter().map(|k| k.id.clone()).collect();
        let taken: Vec<&str> = taken.iter().map(String::as_str).collect();
        let Some((c, t)) = CurveRef::picked(e, &taken) else { break };
        let t = if matches!(e.input, Input::Start) { None } else { t };
        st.picks.resize(st.curves.len(), None);
        st.curves.push(c);
        st.picks.push(t);
    }
}

impl State {
    /// What a click on the curve itself settles at once (as in Matrix, where the pick on the
    /// curve is the point): the start of a Perpendicular/Tangent line, or the whole line for
    /// 2Curves.
    fn settle_picks(&mut self, e: &Event) -> Option<Reply> {
        let n_plane = e.plane.normal();
        let pick = |k: usize| self.picks.get(k).copied().flatten();
        match self.mode.as_str() {
            "perp" | "tan" if self.start.is_none() && self.curves.len() == 1 => {
                let t = pick(0)?;
                let c = &self.curves[0];
                let tan = c.tangent(t);
                self.start = Some(c.at(t));
                self.axis = Some(if self.mode == "tan" {
                    tan
                } else {
                    let side = cross(n_plane, tan);
                    if norm(side) > 1e-9 { unit(side) } else { unit(cross(tan, e.plane.x)) }
                });
                None
            }
            "tan_2" if self.curves.len() == 2 => {
                let (ta, tb) = (pick(0)?, pick(1)?);
                let (a, b) = (&self.curves[0], &self.curves[1]);
                Some(match common_tangent(a, b, ta, tb, n_plane) {
                    Some((x, y)) if dist(x, y) > g::EPS => self.done(x, y),
                    _ => {
                        self.curves.pop();
                        self.picks.pop();
                        self.reply(e, None).refuse("No line touches both curves near those points")
                    }
                })
            }
            "perp_2" if self.curves.len() == 2 => {
                pick(0)?;
                pick(1)?;
                let (a, b) = closest_pair(&self.curves[0], &self.curves[1]);
                Some(if dist(a, b) < g::EPS {
                    self.curves.pop();
                    self.picks.pop();
                    self.reply(e, None).refuse("The curves touch: no perpendicular line")
                } else {
                    self.done(a, b)
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::{Plane, Selected};

    fn ev(input: Input, state: &serde_json::Value, sel: Vec<Selected>) -> Event {
        Event {
            command: crate::LINE.into(),
            input,
            state: state.clone(),
            plane: Plane::default(),
            selection: sel,
            pick: None,
            settings: Default::default(),
        }
    }

    fn run(steps: &[Input], sel: Vec<Selected>) -> Reply {
        let mut state = serde_json::Value::Null;
        let mut last = Reply::default();
        for (i, s) in steps.iter().enumerate() {
            let sel = if i == 0 { Vec::new() } else { sel.clone() };
            last = step(&ev(s.clone(), &state, sel));
            assert!(
                last.error.is_none(),
                "step {i}: {:?} ({})",
                last.error,
                last.prompt
            );
            state = last.state.clone();
        }
        last
    }

    fn opt(name: &str) -> Input {
        Input::Option {
            name: name.into(),
            value: None,
        }
    }
    fn pt(x: f64, y: f64, z: f64) -> Input {
        Input::Point { point: [x, y, z] }
    }
    fn line_of(r: &Reply) -> (Vec3, Vec3) {
        assert!(r.done, "{}", r.prompt);
        match &r.create[0] {
            Geo::Line { start, end } => (*start, *end),
            g => panic!("{g:?}"),
        }
    }
    fn circle(id: &str, cx: f64, r: f64) -> Selected {
        Selected {
            id: id.into(),
            kind: "curve".into(),
            closed: true,
            points: (0..256)
                .map(|i| {
                    let a = std::f64::consts::TAU * i as f64 / 256.0;
                    [cx + r * a.cos(), r * a.sin(), 0.0]
                })
                .collect(),
            exact: Some(Geo::Circle {
                center: [cx, 0.0, 0.0],
                x: [1.0, 0.0, 0.0],
                y: [0.0, 1.0, 0.0],
                radius: r,
            }),
            ..Default::default()
        }
    }
    fn close(a: Vec3, b: Vec3) -> bool {
        dist(a, b) < 1e-4
    }

    #[test]
    fn start_prompt_lists_every_variant() {
        let r = step(&ev(Input::Start, &serde_json::Value::Null, vec![]));
        let names: Vec<&str> = r.options.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "BothSides",
                "Normal",
                "Angled",
                "Vertical",
                "FourPoint",
                "Bisector",
                "Perpendicular",
                "Tangent",
                "Extension"
            ]
        );
        assert_eq!(r.prompt, "Start of line");
    }

    #[test]
    fn plain_and_both_sides() {
        let (a, b) = line_of(&run(
            &[Input::Start, pt(0.0, 0.0, 0.0), pt(3.0, 4.0, 0.0)],
            vec![],
        ));
        assert!(close(a, [0.0; 3]) && close(b, [3.0, 4.0, 0.0]));
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("BothSides"),
                pt(0.0, 0.0, 0.0),
                pt(3.0, 4.0, 0.0),
            ],
            vec![],
        ));
        assert!(close(a, [-3.0, -4.0, 0.0]) && close(b, [3.0, 4.0, 0.0]));
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                pt(0.0, 0.0, 0.0),
                Input::Text { text: "10".into() },
                pt(3.0, 4.0, 0.0),
            ],
            vec![],
        ));
        assert!(close(a, [0.0; 3]) && close(b, [6.0, 8.0, 0.0]));
    }

    #[test]
    fn vertical_angled_fourpoint_bisector() {
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Vertical"),
                pt(1.0, 1.0, 0.0),
                pt(4.0, 5.0, 0.0),
            ],
            vec![],
        ));
        assert!(
            close(a, [1.0, 1.0, 0.0]) && close(b, [1.0, 1.0, 5.0]),
            "{b:?}"
        );
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Angled"),
                pt(0.0, 0.0, 0.0),
                pt(10.0, 0.0, 0.0),
                Input::Text { text: "90".into() },
                pt(0.0, 7.0, 0.0),
            ],
            vec![],
        ));
        assert!(close(a, [0.0; 3]) && close(b, [0.0, 7.0, 0.0]), "{b:?}");
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("FourPoint"),
                pt(0.0, 0.0, 0.0),
                pt(0.0, 1.0, 0.0),
                pt(5.0, 5.0, 0.0),
                pt(9.0, 8.0, 0.0),
            ],
            vec![],
        ));
        assert!(
            close(a, [5.0, 5.0, 0.0]) && close(b, [5.0, 8.0, 0.0]),
            "{b:?}"
        );
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Bisector"),
                pt(0.0, 0.0, 0.0),
                pt(10.0, 0.0, 0.0),
                pt(0.0, 10.0, 0.0),
                pt(4.0, 4.0, 0.0),
            ],
            vec![],
        ));
        assert!(close(a, [0.0; 3]) && close(b, [4.0, 4.0, 0.0]), "{b:?}");
    }

    #[test]
    fn perpendicular_and_tangent_to_a_circle() {
        let c = circle("c", 0.0, 10.0);
        // From the curve: start on the circle at (10, 0); the line leaves along the radius.
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Perpendicular"),
                Input::Selection,
                pt(12.0, 0.0, 0.0),
                pt(15.0, 3.0, 0.0),
            ],
            vec![c.clone()],
        ));
        assert!(
            close(a, [10.0, 0.0, 0.0]) && close(b, [15.0, 0.0, 0.0]),
            "{a:?} {b:?}"
        );
        // To the curve from a point: the foot nearest the pick.
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Perpendicular"),
                opt("Point"),
                Input::Selection,
                pt(0.0, 20.0, 0.0),
                pt(0.0, 9.0, 0.0),
            ],
            vec![c.clone()],
        ));
        assert!(
            close(a, [0.0, 20.0, 0.0]) && close(b, [0.0, 10.0, 0.0]),
            "{b:?}"
        );
        // Tangent at a curve point.
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Tangent"),
                Input::Selection,
                pt(10.0, 0.0, 0.0),
                pt(10.0, 6.0, 0.0),
            ],
            vec![c.clone()],
        ));
        assert!(
            close(a, [10.0, 0.0, 0.0]) && close(b, [10.0, 6.0, 0.0]),
            "{b:?}"
        );
        // Tangent from a point: touches at x = r²/d.
        let (_, b) = line_of(&run(
            &[
                Input::Start,
                opt("Tangent"),
                opt("Point"),
                Input::Selection,
                pt(20.0, 0.0, 0.0),
                pt(5.0, 9.0, 0.0),
            ],
            vec![c.clone()],
        ));
        assert!((b[0] - 5.0).abs() < 1e-4 && b[1] > 0.0, "{b:?}");
    }

    #[test]
    fn two_curve_variants() {
        let (c1, c2) = (circle("a", 0.0, 5.0), circle("b", 20.0, 5.0));
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Perpendicular"),
                opt("2Curves"),
                Input::Selection,
                pt(0.0, 0.0, 0.0),
            ],
            vec![c1.clone(), c2.clone()],
        ));
        assert!(
            close(a, [5.0, 0.0, 0.0]) && close(b, [15.0, 0.0, 0.0]),
            "{a:?} {b:?}"
        );
        // Outer common tangent along y = 5.
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Tangent"),
                opt("2Curves"),
                Input::Selection,
                pt(0.0, 6.0, 0.0),
                pt(20.0, 6.0, 0.0),
            ],
            vec![c1, c2],
        ));
        assert!(
            close(a, [0.0, 5.0, 0.0]) && close(b, [20.0, 5.0, 0.0]),
            "{a:?} {b:?}"
        );
    }

    #[test]
    fn extension_and_normal() {
        let seg = Selected {
            id: "l".into(),
            kind: "curve".into(),
            points: vec![[0.0; 3], [10.0, 0.0, 0.0]],
            exact: Some(Geo::Line {
                start: [0.0; 3],
                end: [10.0, 0.0, 0.0],
            }),
            ..Default::default()
        };
        let (a, b) = line_of(&run(
            &[
                Input::Start,
                opt("Extension"),
                Input::Selection,
                pt(14.0, 2.0, 0.0),
            ],
            vec![seg],
        ));
        assert!(
            close(a, [10.0, 0.0, 0.0]) && close(b, [14.0, 0.0, 0.0]),
            "{b:?}"
        );
        let srf = Selected {
            id: "s".into(),
            kind: "surface".into(),
            exact: Some(Geo::Surface {
                degree: [1, 1],
                size: [2, 2],
                points: vec![
                    [0.0; 3],
                    [0.0, 10.0, 0.0],
                    [10.0, 0.0, 0.0],
                    [10.0, 10.0, 0.0],
                ],
                weights: vec![],
                knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
            }),
            ..Default::default()
        };
        let r = run(
            &[
                Input::Start,
                opt("Normal"),
                Input::Selection,
                pt(3.0, 4.0, 0.0),
                Input::Text { text: "6".into() },
                pt(3.0, 4.0, 0.0),
            ],
            vec![srf],
        );
        let (a, b) = line_of(&r);
        assert!(
            close(a, [3.0, 4.0, 0.0])
                && (b[2].abs() - 6.0).abs() < 1e-6
                && close([b[0], b[1], 0.0], [3.0, 4.0, 0.0]),
            "{b:?}"
        );
    }
}
