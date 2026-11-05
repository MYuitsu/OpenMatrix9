// SPDX-License-Identifier: LGPL-2.1-or-later
//! Polyline (Matrix Curve #1): line and arc segments.
//!
//! Prompts and sub-options as in Matrix 9 / Rhino 5:
//! - `Start of polyline ( PersistentClose )`
//! - `Next point of polyline ( PersistentClose Mode=Line Helpers Undo )`
//! - from the 2nd point: `… Press Enter when done ( … Length Undo )`, from the 3rd: `Close`
//! - `Mode=Arc`: each segment is an arc tangent to the previous one; `Direction` picks the
//!   start direction of the next arc, `Center` its center.
//! - `Helpers`: tangent and CPlane x/y guide lines through the last point.
use crate::{AUTO_CLOSE, load, number_of, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, add, cross, dist, dot, norm, parse_number, scale,
    sub, unit,
};
use om9_addon_geom::{self as g, Piece};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Seg {
    Line,
    /// Arc ending at the segment's end point, around `center`, sweeping `angle` (signed, in
    /// the plane with normal `axis`).
    Arc {
        center: Vec3,
        axis: Vec3,
        angle: f64,
    },
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub points: Vec<Vec3>,
    /// `segs[i]` joins `points[i]` to `points[i + 1]`.
    pub segs: Vec<Seg>,
    pub persistent_close: bool,
    pub arc: bool,
    pub helpers: bool,
    pub length: Option<f64>,
    /// Next arc: picked start direction / center (Direction, Center).
    pub direction: Option<Vec3>,
    pub center: Option<Vec3>,
    /// Waiting for the Direction / Center pick.
    pub picking: Option<String>,
}

/// Arc from `a` leaving along `t` to `b` (plane of `t` and the chord, else the CPlane).
fn arc_tangent(a: Vec3, t: Vec3, b: Vec3) -> Option<Seg> {
    let c = sub(b, a);
    if norm(c) < g::EPS {
        return None;
    }
    let mut n = cross(t, c);
    if norm(n) < 1e-9 {
        return Some(Seg::Line);
    }
    n = unit(n);
    let m = cross(n, unit(t)); // towards the center
    let r = dot(c, c) / (2.0 * dot(c, m));
    let center = add(a, scale(m, r));
    let (u, v) = (sub(a, center), sub(b, center));
    let mut ang = dot(cross(u, v), n).atan2(dot(u, v));
    if ang < 0.0 {
        ang += std::f64::consts::TAU;
    }
    Some(Seg::Arc {
        center,
        axis: n,
        angle: ang,
    })
}

/// Arc from `a` around `center` to the direction of `p` (counter-clockwise about `axis`, or
/// following `prev` tangent when given).
fn arc_center(
    a: Vec3,
    center: Vec3,
    p: Vec3,
    axis: Vec3,
    prev: Option<Vec3>,
) -> Option<(Seg, Vec3)> {
    let u = sub(a, center);
    let r = norm(u);
    let w = sub(p, center);
    let w = sub(w, scale(axis, dot(w, axis)));
    if r < g::EPS || norm(w) < g::EPS {
        return None;
    }
    let end = add(center, scale(unit(w), r));
    let mut ang = dot(cross(u, w), axis).atan2(dot(u, w));
    let ccw_tangent = unit(cross(axis, u));
    let ccw = prev.is_none_or(|t| dot(t, ccw_tangent) >= 0.0);
    if ccw && ang < 0.0 {
        ang += std::f64::consts::TAU;
    } else if !ccw && ang > 0.0 {
        ang -= std::f64::consts::TAU;
    }
    (ang.abs() > 1e-9).then_some((
        Seg::Arc {
            center,
            axis,
            angle: ang,
        },
        end,
    ))
}

impl State {
    /// Unit direction at the end of the polyline so far.
    fn end_tangent(&self) -> Option<Vec3> {
        let n = self.points.len();
        if n < 2 {
            return None;
        }
        let (a, b) = (self.points[n - 2], self.points[n - 1]);
        Some(match self.segs[n - 2] {
            Seg::Line => unit(sub(b, a)),
            Seg::Arc {
                center,
                axis,
                angle,
            } => {
                let t = unit(cross(axis, sub(b, center)));
                if angle < 0.0 { scale(t, -1.0) } else { t }
            }
        })
    }

    /// The segment and end point a click or the cursor `p` adds.
    fn next(&self, e: &Event, p: Vec3) -> Option<(Seg, Vec3)> {
        let last = *self.points.last()?;
        if !self.arc {
            let end = match self.length {
                Some(l) if dist(last, p) > g::EPS => add(last, scale(unit(sub(p, last)), l)),
                _ => p,
            };
            return (dist(last, end) > g::EPS).then_some((Seg::Line, end));
        }
        let n = e.plane.normal();
        if let Some(c) = self.center {
            return arc_center(last, c, p, n, self.end_tangent());
        }
        let t = self.direction.or_else(|| self.end_tangent())?;
        arc_tangent(last, t, p).map(|s| (s, p))
    }

    fn geo(points: &[Vec3], segs: &[Seg], closed: bool) -> Option<Geo> {
        if points.len() < 2 {
            return None;
        }
        if segs.iter().all(|s| *s == Seg::Line) {
            return Some(Geo::Polyline {
                points: points.to_vec(),
                closed: closed && points.len() >= 3,
            });
        }
        let mut pieces = Vec::new();
        for (i, s) in segs.iter().enumerate() {
            let (a, b) = (points[i], points[i + 1]);
            match *s {
                Seg::Line => pieces.push(Piece::Line(a, b)),
                Seg::Arc {
                    center,
                    axis,
                    angle,
                } => {
                    // Split into pieces of at most 90°.
                    let k = (angle.abs() / std::f64::consts::FRAC_PI_2 - 1e-9)
                        .ceil()
                        .max(1.0) as usize;
                    let u = sub(a, center);
                    let v = cross(axis, u);
                    let mut prev = a;
                    for j in 1..=k {
                        let th = angle * j as f64 / k as f64;
                        let q = if j == k {
                            b
                        } else {
                            add(center, add(scale(u, th.cos()), scale(v, th.sin())))
                        };
                        pieces.push(Piece::Arc {
                            center,
                            start: prev,
                            end: q,
                        });
                        prev = q;
                    }
                }
            }
        }
        if closed && points.len() >= 3 && dist(points[0], *points.last()?) > g::EPS {
            pieces.push(Piece::Line(*points.last()?, points[0]));
        }
        g::composite_quadratic(&pieces)
    }

    fn reply(&self, e: &Event, hover: Option<Vec3>) -> Reply {
        let n = self.points.len();
        let mut opts = vec![OptionToken::toggle(
            "PersistentClose",
            self.persistent_close,
        )];
        if n >= 3 {
            opts.push(OptionToken::action("Close"));
        }
        if n >= 1 {
            opts.push(OptionToken::choice(
                "Mode",
                if self.arc { "Arc" } else { "Line" },
                &["Line", "Arc"],
            ));
            opts.push(OptionToken::toggle("Helpers", self.helpers));
            if self.arc {
                opts.push(OptionToken::action("Direction"));
                opts.push(OptionToken::action("Center"));
            } else if n >= 2 {
                opts.push(OptionToken::number("Length", self.length));
            }
            opts.push(OptionToken::action("Undo"));
        }
        let (mut pts, mut segs) = (self.points.clone(), self.segs.clone());
        let mut preview = Vec::new();
        if self.picking.is_none()
            && let Some((s, end)) = hover.and_then(|h| self.next(e, h))
        {
            pts.push(end);
            segs.push(s);
        }
        preview.extend(Self::geo(&pts, &segs, self.persistent_close));
        if self.helpers
            && let Some(&last) = self.points.last()
        {
            let r = 1000.0;
            let mut dirs = vec![e.plane.x, e.plane.y];
            dirs.extend(self.end_tangent());
            for d in dirs {
                preview.push(Geo::Line {
                    start: sub(last, scale(d, r)),
                    end: add(last, scale(d, r)),
                });
            }
        }
        let prompt = match (self.picking.as_deref(), n) {
            (Some("Direction"), _) => "Direction of the next arc",
            (Some("Center"), _) => "Center of the next arc",
            (_, 0) => "Start of polyline",
            (_, 1) => "Next point of polyline",
            _ => "Next point of polyline. Press Enter when done",
        };
        let rubber = match self.picking.as_deref() {
            Some(_) => self.points.last().copied(),
            None => None,
        };
        Reply::ask(prompt, save(self))
            .with_options(opts)
            .preview(preview)
            .rubber(rubber)
    }

    fn finish(&self, closed: bool) -> Reply {
        if self.points.len() < 2 {
            return Reply::finish(vec![]).say("Polyline cancelled: needs two points");
        }
        let closed = closed && self.points.len() >= 3;
        match Self::geo(&self.points, &self.segs, closed) {
            Some(c) => Reply::finish(vec![c]).say(format!(
                "Polyline: {} points{}",
                self.points.len(),
                if closed { ", closed" } else { "" }
            )),
            None => Reply::finish(vec![]).say("Polyline cancelled: needs two points"),
        }
    }
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    match &e.input {
        Input::Start | Input::Selection => st.reply(e, None),
        Input::Hover { point } => st.reply(e, Some(*point)),
        Input::Point { point } => {
            if let Some(what) = st.picking.take() {
                let last = *st.points.last().expect("a point before Direction/Center");
                if what == "Direction" {
                    let d = sub(*point, last);
                    if norm(d) < g::EPS {
                        st.picking = Some(what);
                        return st
                            .reply(e, None)
                            .refuse("Pick a direction away from the last point");
                    }
                    st.direction = Some(unit(d));
                    st.center = None;
                } else {
                    st.center = Some(*point);
                    st.direction = None;
                }
                return st.reply(e, None);
            }
            if st.points.is_empty() {
                st.points.push(*point);
                return st.reply(e, None);
            }
            let Some((seg, end)) = st.next(e, *point) else {
                return st.reply(e, None).refuse(if st.arc {
                    "Pick a direction for the first arc (Direction)"
                } else {
                    "The point repeats the last one"
                });
            };
            if st.points.len() >= 3 && dist(end, st.points[0]) <= AUTO_CLOSE && seg == Seg::Line {
                return st.finish(true);
            }
            st.points.push(end);
            st.segs.push(seg);
            st.length = None;
            st.direction = None;
            st.center = None;
            st.reply(e, None)
        }
        Input::Option { name, value } => match name.as_str() {
            "PersistentClose" => {
                st.persistent_close = yes(value, st.persistent_close);
                st.reply(e, None)
            }
            "Mode" => {
                st.arc = match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
                    Some("arc" | "a") => true,
                    Some("line" | "l") => false,
                    _ => !st.arc,
                };
                st.length = None;
                st.reply(e, None)
            }
            "Helpers" => {
                st.helpers = yes(value, st.helpers);
                st.reply(e, None)
            }
            "Direction" | "Center" if st.arc && !st.points.is_empty() => {
                st.picking = Some(name.clone());
                st.reply(e, None)
            }
            "Length" if !st.arc => match number_of(value) {
                Some(v) if v > 0.0 => {
                    st.length = Some(v);
                    st.reply(e, None)
                }
                _ => st.reply(e, None).refuse("Length must be a positive number"),
            },
            "Close" if st.points.len() >= 3 => st.finish(true),
            _ => st.reply(e, None).refuse(format!("Unknown option {name}")),
        },
        Input::Text { text } => match parse_number(text) {
            Some(v) if v > 0.0 && !st.points.is_empty() && !st.arc => {
                st.length = Some(v);
                st.reply(e, None)
            }
            _ => st
                .reply(e, None)
                .refuse(format!("Not a point or length: {text}")),
        },
        Input::Undo => {
            if st.picking.take().is_none() && st.points.pop().is_some() {
                st.segs.truncate(st.points.len().saturating_sub(1));
            }
            st.reply(e, None)
        }
        Input::Enter => st.finish(st.persistent_close),
    }
}
