// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rectangle (Matrix Curve #4).
//!
//! - `First corner of rectangle ( 3Point Vertical Center Rounded )`
//! - `Other corner or length ( Rounded )`; after a typed length: `Width. Press Enter to use
//!   length`
//! - `3Point` / `Vertical`: `Start of edge`, `End of edge`, `Width`
//! - `Center`: `Center of rectangle`, `Other corner or length`
//! - `Rounded`, once the rectangle is set: `Radius or point for rounded corner to pass
//!   through <r> ( Corner=Arc|Conic [Rho] )`
//!
//! Vertical rectangles stand on the picked edge, perpendicular to the CPlane; the cursor's
//! distance from the edge in the CPlane gives the height (or type it).
use crate::{load, number_of, on_plane, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, Want, add, cross, dot, fmt, norm, parse_number,
    scale, sub, unit,
};
use om9_addon_geom as g;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    /// "corner" (default), "center", "3point", "vertical".
    pub mode: String,
    pub rounded: bool,
    pub conic: bool,
    pub rho: f64,
    pub points: Vec<Vec3>,
    pub length: Option<f64>,
    /// The rectangle (corner, u, v) waiting for its corner radius.
    pub rect: Option<[Vec3; 3]>,
}

impl Default for State {
    fn default() -> Self {
        State {
            mode: "corner".into(),
            rounded: false,
            conic: false,
            rho: 0.5,
            points: Vec::new(),
            length: None,
            rect: None,
        }
    }
}

/// Radius of a corner (corner `c`, inward unit edges `a`, `b`) whose arc passes through `p`.
fn radius_through(c: Vec3, a: Vec3, b: Vec3, p: Vec3) -> f64 {
    let d = sub(p, c);
    let (x, y) = (dot(d, a).max(0.0), dot(d, b).max(0.0));
    // (x − r)² + (y − r)² = r², with the point on the corner side of the arc's center.
    (x + y) + (2.0 * x * y).sqrt()
}

impl State {
    fn edge_mode(&self) -> bool {
        matches!(self.mode.as_str(), "3point" | "vertical")
    }

    /// (corner, u, v) of the rectangle the picks and the cursor `p` define.
    fn frame(&self, e: &Event, p: Vec3) -> Option<[Vec3; 3]> {
        let n = e.plane.normal();
        match (self.mode.as_str(), self.points.as_slice()) {
            ("corner", [a]) => {
                let o = e.plane.local(*a);
                let q = e.plane.local(on_plane(e, *a, p));
                let (mut du, mut dv) = (q[0] - o[0], q[1] - o[1]);
                if let Some(l) = self.length {
                    du = if du < 0.0 { -l } else { l };
                    if dv.abs() < g::EPS {
                        dv = l;
                    }
                }
                Some([*a, scale(e.plane.x, du), scale(e.plane.y, dv)])
            }
            ("center", [c]) => {
                let o = e.plane.local(*c);
                let q = e.plane.local(on_plane(e, *c, p));
                let (mut du, mut dv) = (q[0] - o[0], q[1] - o[1]);
                if let Some(l) = self.length {
                    du = if du < 0.0 { -l / 2.0 } else { l / 2.0 };
                    if dv.abs() < g::EPS {
                        dv = l / 2.0;
                    }
                }
                let (u, v) = (scale(e.plane.x, 2.0 * du), scale(e.plane.y, 2.0 * dv));
                Some([sub(sub(*c, scale(u, 0.5)), scale(v, 0.5)), u, v])
            }
            (_, [a]) if self.edge_mode() => {
                let q = on_plane(e, *a, p);
                let mut u = sub(q, *a);
                if let Some(l) = self.length.filter(|_| norm(u) > g::EPS) {
                    u = scale(unit(u), l);
                }
                Some([*a, u, [0.0; 3]])
            }
            ("3point", [a, b]) => {
                let u = sub(*b, *a);
                let side = unit(cross(n, u));
                Some([*a, u, scale(side, dot(sub(p, *a), side))])
            }
            ("vertical", [a, b]) => {
                let u = sub(*b, *a);
                let side = unit(cross(n, u));
                // A point off the CPlane (a snap) gives the height directly; otherwise the
                // cursor's distance from the edge in the CPlane does.
                let off = dot(sub(p, *b), n);
                let h = if off.abs() > g::EPS {
                    off
                } else {
                    dot(sub(on_plane(e, *a, p), *a), side).abs()
                };
                Some([*a, u, scale(n, h)])
            }
            _ => None,
        }
    }

    fn geo(&self, r: [Vec3; 3], radius: Option<f64>) -> Option<Geo> {
        let [o, u, v] = r;
        if norm(u) < g::EPS || norm(v) < g::EPS {
            return None;
        }
        match radius {
            Some(rad) if rad > g::EPS => {
                g::rounded_rectangle_with(o, u, v, rad, self.conic.then_some(self.rho))
            }
            _ => Some(Geo::Polyline {
                points: vec![o, add(o, u), add(add(o, u), v), add(o, v)],
                closed: true,
            }),
        }
    }

    /// Default corner radius: a tenth of the short side.
    fn default_radius(r: &[Vec3; 3]) -> f64 {
        norm(r[1]).min(norm(r[2])) / 10.0
    }

    /// Corner radius from a pick: through the point, at the nearest corner.
    fn radius_from_point(r: &[Vec3; 3], p: Vec3) -> f64 {
        let [o, u, v] = *r;
        let corners = [
            (o, unit(u), unit(v)),
            (add(o, u), scale(unit(u), -1.0), unit(v)),
            (
                add(add(o, u), v),
                scale(unit(u), -1.0),
                scale(unit(v), -1.0),
            ),
            (add(o, v), unit(u), scale(unit(v), -1.0)),
        ];
        let (c, a, b) = corners
            .into_iter()
            .min_by(|x, y| {
                om9_addon_abi::session::dist(x.0, p)
                    .total_cmp(&om9_addon_abi::session::dist(y.0, p))
            })
            .expect("four corners");
        radius_through(c, a, b, p).min(norm(u).min(norm(v)) / 2.0)
    }

    fn reply(&self, e: &Event, hover: Option<Vec3>) -> Reply {
        if let Some(r) = self.rect {
            let rad = hover.map(|h| Self::radius_from_point(&r, h));
            let mut opts = vec![OptionToken::choice(
                "Corner",
                if self.conic { "Conic" } else { "Arc" },
                &["Arc", "Conic"],
            )];
            if self.conic {
                opts.push(OptionToken::number("Rho", Some(self.rho)));
            }
            return Reply::ask(
                format!(
                    "Radius or point for rounded corner to pass through <{}>",
                    fmt(Self::default_radius(&r))
                ),
                save(self),
            )
            .with_options(opts)
            .want(Want::Number)
            .preview(
                self.geo(r, rad.or(Some(Self::default_radius(&r))))
                    .into_iter()
                    .collect(),
            );
        }
        let preview = hover
            .and_then(|h| self.frame(e, h))
            .and_then(|r| {
                self.geo(r, None).or(Some(Geo::Line {
                    start: r[0],
                    end: add(r[0], r[1]),
                }))
            })
            .into_iter()
            .collect();
        let n = self.points.len();
        let prompt = match (self.mode.as_str(), n) {
            ("center", 0) => "Center of rectangle",
            ("center", _) => "Other corner or length",
            (_, 0) if self.edge_mode() => "Start of edge",
            (_, 1) if self.edge_mode() => "End of edge",
            (_, _) if self.edge_mode() => "Width",
            (_, 0) => "First corner of rectangle",
            _ if self.length.is_some() => "Width. Press Enter to use length",
            _ => "Other corner or length",
        };
        let mut opts = Vec::new();
        if n == 0 && self.mode == "corner" {
            opts.extend([
                OptionToken::action("3Point"),
                OptionToken::action("Vertical"),
                OptionToken::action("Center"),
            ]);
        }
        opts.push(OptionToken::toggle("Rounded", self.rounded));
        Reply::ask(prompt, save(self))
            .with_options(opts)
            .preview(preview)
    }

    fn complete(&mut self, e: &Event, r: [Vec3; 3]) -> Reply {
        if norm(r[1]) < g::EPS || norm(r[2]) < g::EPS {
            return self.reply(e, None).refuse("The rectangle has no area");
        }
        if self.rounded {
            self.rect = Some(r);
            return self.reply(e, None);
        }
        self.finish(r, None)
    }

    fn finish(&self, r: [Vec3; 3], radius: Option<f64>) -> Reply {
        match self.geo(r, radius) {
            Some(c) => Reply::finish(vec![c]).say(format!(
                "Rectangle {} × {} mm",
                fmt(norm(r[1])),
                fmt(norm(r[2]))
            )),
            None => Reply::finish(vec![]).say("Rectangle cancelled"),
        }
    }
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    if st.mode.is_empty() {
        st.mode = "corner".into();
    }
    match &e.input {
        Input::Hover { point } => st.reply(e, Some(*point)),
        Input::Point { point } => {
            if let Some(r) = st.rect {
                let rad = State::radius_from_point(&r, *point);
                return st.finish(r, Some(rad));
            }
            let needs = if st.edge_mode() { 3 } else { 2 };
            if st.points.len() + 1 < needs {
                st.points.push(*point);
                if st.edge_mode() {
                    st.length = None;
                }
                return st.reply(e, None);
            }
            match st.frame(e, *point) {
                Some(r) => st.complete(e, r),
                None => st.reply(e, None),
            }
        }
        Input::Option { name, value } => {
            match name.as_str() {
                "3Point" if st.points.is_empty() => st.mode = "3point".into(),
                "Vertical" if st.points.is_empty() => st.mode = "vertical".into(),
                "Center" if st.points.is_empty() => st.mode = "center".into(),
                "Rounded" => st.rounded = yes(value, st.rounded),
                "Corner" => {
                    st.conic = match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
                        Some("conic" | "c") => true,
                        Some("arc" | "a") => false,
                        _ => !st.conic,
                    }
                }
                "Rho" => match number_of(value) {
                    Some(v) if v > 0.0 && v < 1.0 => st.rho = v,
                    _ => return st.reply(e, None).refuse("Rho is a number between 0 and 1"),
                },
                _ => return st.reply(e, None).refuse(format!("Unknown option {name}")),
            }
            st.reply(e, None)
        }
        Input::Text { text } => {
            let Some(v) = parse_number(text).filter(|v| *v > 0.0) else {
                return st.reply(e, None).refuse(format!("Not a number: {text}"));
            };
            if let Some(r) = st.rect {
                if 2.0 * v > norm(r[1]).min(norm(r[2])) + g::EPS {
                    return st
                        .reply(e, None)
                        .refuse("Radius is larger than half the short side");
                }
                return st.finish(r, Some(v));
            }
            match (st.mode.as_str(), st.points.len()) {
                ("corner" | "center", 1) if st.length.is_none() => {
                    st.length = Some(v);
                    st.reply(e, None)
                }
                ("corner", 1) => {
                    let a = st.points[0];
                    let l = st.length.unwrap_or(v);
                    let r = [a, scale(e.plane.x, l), scale(e.plane.y, v)];
                    st.complete(e, r)
                }
                ("center", 1) => {
                    let c = st.points[0];
                    let l = st.length.unwrap_or(v);
                    let (u, w) = (scale(e.plane.x, l), scale(e.plane.y, v));
                    let r = [sub(sub(c, scale(u, 0.5)), scale(w, 0.5)), u, w];
                    st.complete(e, r)
                }
                (_, 1) if st.edge_mode() => {
                    st.length = Some(v);
                    st.reply(e, None)
                }
                ("3point", 2) => {
                    let [a, b] = [st.points[0], st.points[1]];
                    let side = unit(cross(e.plane.normal(), sub(b, a)));
                    st.complete(e, [a, sub(b, a), scale(side, v)])
                }
                ("vertical", 2) => {
                    let [a, b] = [st.points[0], st.points[1]];
                    st.complete(e, [a, sub(b, a), scale(e.plane.normal(), v)])
                }
                _ => st.reply(e, None).refuse("Pick a point"),
            }
        }
        Input::Enter => {
            if let Some(r) = st.rect {
                return st.finish(r, Some(State::default_radius(&r)));
            }
            // Width = length (a square) after a typed length.
            if let (Some(l), 1) = (st.length, st.points.len()) {
                let a = st.points[0];
                let r = if st.mode == "center" {
                    let (u, w) = (scale(e.plane.x, l), scale(e.plane.y, l));
                    [sub(sub(a, scale(u, 0.5)), scale(w, 0.5)), u, w]
                } else {
                    [a, scale(e.plane.x, l), scale(e.plane.y, l)]
                };
                if !st.edge_mode() {
                    return st.complete(e, r);
                }
            }
            Reply::finish(vec![])
        }
        _ => st.reply(e, None),
    }
}
