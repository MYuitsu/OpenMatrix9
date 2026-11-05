// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ellipse (Matrix Curve #6).
//!
//! - `Ellipse center ( Deformable Vertical Corner Diameter FromFoci AroundCurve )`
//! - `End of first axis ( Corner )`, `End of second axis`
//! - `Corner`: two opposite corners of the enclosing rectangle (at the start), or one corner
//!   once the center is set
//! - `Diameter`: `Start of first axis`, `End of first axis`, `End of second axis`
//! - `FromFoci`: `First focus`, `Second focus`, `Point on ellipse` (`MarkFoci` adds points)
//! - `Vertical`: the second axis stands along the CPlane normal
//! - `AroundCurve`: select a curve, center on it; the ellipse is perpendicular to the curve
//! - `Deformable`: `Degree=3 PointCount=10` NURBS approximation
use crate::pick::CurveRef;
use crate::{load, number_of, on_plane, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, Want, add, cross, dist, dot, norm, parse_number,
    scale, sub, unit,
};
use om9_addon_geom as g;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    /// "center", "vertical", "corner", "diameter", "foci", "around".
    pub mode: String,
    pub deformable: bool,
    pub degree: usize,
    pub point_count: usize,
    pub mark_foci: bool,
    pub points: Vec<Vec3>,
    /// `Corner` chosen after the center: the next pick is a corner of the enclosing box.
    pub corner_from_center: bool,
    pub curve: Option<CurveRef>,
    /// AroundCurve: the plane's normal (curve tangent at the center).
    pub normal: Option<Vec3>,
}

impl Default for State {
    fn default() -> Self {
        State {
            mode: "center".into(),
            deformable: false,
            degree: 3,
            point_count: 10,
            mark_foci: false,
            points: Vec::new(),
            corner_from_center: false,
            curve: None,
            normal: None,
        }
    }
}

impl State {
    fn normal(&self, e: &Event) -> Vec3 {
        self.normal.unwrap_or_else(|| e.plane.normal())
    }

    fn shape(&self, c: Vec3, a: Vec3, b: Vec3) -> Option<Geo> {
        if norm(a) < g::EPS || norm(b) < g::EPS {
            return None;
        }
        if self.deformable {
            g::interp::deformable(c, a, b, self.degree, self.point_count)
        } else {
            g::ellipse(c, a, b)
        }
    }

    /// Project `p` into the ellipse plane through `c`.
    fn flat(&self, e: &Event, c: Vec3, p: Vec3) -> Vec3 {
        let n = self.normal(e);
        sub(p, scale(n, dot(sub(p, c), n)))
    }

    /// Semi-axes for center `c`, first axis end `a_end`, cursor `p` (second axis).
    fn axes(&self, e: &Event, c: Vec3, a_end: Vec3, p: Vec3) -> Option<(Vec3, Vec3)> {
        let a = if self.mode == "vertical" {
            sub(on_plane(e, c, a_end), c)
        } else {
            sub(self.flat(e, c, a_end), c)
        };
        if norm(a) < g::EPS {
            return None;
        }
        let side = if self.mode == "vertical" {
            e.plane.normal()
        } else {
            unit(cross(self.normal(e), a))
        };
        let b = if self.mode == "vertical" {
            // Height from the cursor's distance off the first axis, in the CPlane.
            let up = dot(sub(p, c), e.plane.normal()).abs();
            if up > g::EPS {
                up
            } else {
                let q = on_plane(e, c, p);
                let off = sub(sub(q, c), scale(unit(a), dot(sub(q, c), unit(a))));
                norm(off)
            }
        } else {
            dot(sub(p, c), side).abs()
        };
        Some((a, scale(side, b)))
    }

    /// The ellipse the picks and the cursor `p` define.
    fn ellipse(&self, e: &Event, p: Vec3) -> Option<Geo> {
        let n = e.plane.normal();
        match (self.mode.as_str(), self.points.as_slice()) {
            ("corner", [a]) => {
                let q = on_plane(e, *a, p);
                let (o, w) = (e.plane.local(*a), e.plane.local(q));
                let (du, dv) = ((w[0] - o[0]) / 2.0, (w[1] - o[1]) / 2.0);
                let c = scale(add(*a, q), 0.5);
                self.shape(c, scale(e.plane.x, du.abs()), scale(e.plane.y, dv.abs()))
            }
            (_, [c]) if self.corner_from_center => {
                let q = self.flat(e, *c, p);
                let (o, w) = (e.plane.local(*c), e.plane.local(q));
                self.shape(
                    *c,
                    scale(e.plane.x, (w[0] - o[0]).abs()),
                    scale(e.plane.y, (w[1] - o[1]).abs()),
                )
            }
            ("diameter", [s, t]) => {
                let c = scale(add(*s, *t), 0.5);
                let a = sub(*t, c);
                let side = unit(cross(n, a));
                self.shape(c, a, scale(side, dot(sub(p, c), side).abs()))
            }
            ("foci", [f1, f2]) => {
                let c = scale(add(*f1, *f2), 0.5);
                let q = on_plane(e, c, p);
                let major = (dist(q, *f1) + dist(q, *f2)) / 2.0;
                let focal = dist(*f1, *f2) / 2.0;
                if major <= focal + g::EPS {
                    return None;
                }
                let minor = (major * major - focal * focal).sqrt();
                let x = if focal > g::EPS {
                    unit(sub(*f2, c))
                } else {
                    e.plane.x
                };
                let y = cross(n, x);
                self.shape(c, scale(x, major), scale(unit(y), minor))
            }
            ("center" | "vertical" | "around", [c, a]) => {
                let (a, b) = self.axes(e, *c, *a, p)?;
                self.shape(*c, a, b)
            }
            _ => None,
        }
    }

    fn needs(&self) -> usize {
        match self.mode.as_str() {
            "corner" => 2,
            _ => 3,
        }
    }

    fn reply(&self, e: &Event, hover: Option<Vec3>) -> Reply {
        let n = self.points.len();
        let mut opts = Vec::new();
        let mut want = Want::Point;
        let prompt = match (self.mode.as_str(), n) {
            ("around", _) if self.curve.is_none() => {
                want = Want::Objects;
                "Select curve to draw ellipse around"
            }
            ("around", 0) => "Ellipse center on the curve",
            ("corner", 0) => "First corner of the enclosing rectangle",
            ("corner", _) => "Other corner of the enclosing rectangle",
            ("diameter", 0) => "Start of first axis",
            ("diameter", 1) => "End of first axis",
            ("foci", 0) => "First focus",
            ("foci", 1) => "Second focus",
            ("foci", _) => "Point on ellipse",
            (_, 0) => "Ellipse center",
            (_, 1) if self.corner_from_center => "Corner of the enclosing rectangle",
            (_, 1) => "End of first axis",
            _ => "End of second axis",
        };
        if n == 0
            && self.curve.is_none()
            && self.mode != "corner"
            && self.mode != "diameter"
            && self.mode != "foci"
        {
            if self.deformable {
                opts.push(OptionToken::number("Degree", Some(self.degree as f64)));
                opts.push(OptionToken::number(
                    "PointCount",
                    Some(self.point_count as f64),
                ));
            } else {
                opts.push(OptionToken::action("Deformable"));
            }
            if self.mode == "center" {
                for o in ["Vertical", "Corner", "Diameter", "FromFoci", "AroundCurve"] {
                    opts.push(OptionToken::action(o));
                }
            }
        }
        if n == 1
            && matches!(self.mode.as_str(), "center" | "vertical" | "around")
            && !self.corner_from_center
        {
            opts.push(OptionToken::action("Corner"));
        }
        if self.mode == "foci" {
            opts.push(OptionToken::toggle("MarkFoci", self.mark_foci));
        }
        let preview: Vec<Geo> = match hover {
            Some(h)
                if n + 1 >= self.needs()
                    || (n == 1 && self.corner_from_center)
                    || (self.mode == "corner" && n == 1) =>
            {
                self.ellipse(e, h).into_iter().collect()
            }
            Some(h) if n >= 1 => vec![Geo::Line {
                start: self.points[n - 1],
                end: h,
            }],
            _ => vec![],
        };
        Reply::ask(prompt, save(self))
            .with_options(opts)
            .want(want)
            .preview(preview)
    }

    fn finish(&self, geo: Geo) -> Reply {
        let mut out = vec![geo];
        if self.mode == "foci" && self.mark_foci {
            out.extend(self.points.iter().take(2).map(|p| Geo::Point { point: *p }));
        }
        Reply::finish(out).say("Ellipse")
    }
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    if st.mode.is_empty() {
        st = State::default();
    }
    match &e.input {
        Input::Start | Input::Selection => {
            if st.mode == "around" && st.curve.is_none() {
                // The click on the curve is the center (as in Matrix).
                match CurveRef::picked(e, &[]) {
                    Some((c, Some(t))) if matches!(e.input, Input::Selection) => {
                        st.normal = Some(c.tangent(t));
                        st.points.push(c.at(t));
                        st.curve = Some(c);
                    }
                    Some((c, _)) => st.curve = Some(c),
                    None => {}
                }
            }
            st.reply(e, None)
        }
        Input::Hover { point } => st.reply(e, Some(*point)),
        Input::Point { point } => {
            if st.mode == "around" && st.points.is_empty() {
                let Some(c) = &st.curve else {
                    return st.reply(e, None).refuse("Select a curve first");
                };
                let (t, q) = c.closest(*point);
                st.normal = Some(c.tangent(t));
                st.points.push(q);
                return st.reply(e, None);
            }
            let done_now = (st.corner_from_center && st.points.len() == 1)
                || st.points.len() + 1 >= st.needs();
            if !done_now {
                st.points.push(*point);
                return st.reply(e, None);
            }
            match st.ellipse(e, *point) {
                Some(g) => st.finish(g),
                None => st.reply(e, None).refuse("Both axes need a length"),
            }
        }
        Input::Option { name, value } => {
            let at_start = st.points.is_empty() && st.curve.is_none();
            match name.as_str() {
                "Deformable" => st.deformable = yes(value, st.deformable),
                "Degree" => match number_of(value) {
                    Some(d) if (1.0..=11.0).contains(&d) && d.fract() == 0.0 => {
                        st.degree = d as usize
                    }
                    _ => {
                        return st
                            .reply(e, None)
                            .refuse("Degree is a whole number from 1 to 11");
                    }
                },
                "PointCount" => match number_of(value) {
                    Some(c) if c >= 3.0 && c.fract() == 0.0 && c <= 1000.0 => {
                        st.point_count = c as usize
                    }
                    _ => {
                        return st
                            .reply(e, None)
                            .refuse("PointCount is a whole number from 3");
                    }
                },
                "Vertical" if at_start => st.mode = "vertical".into(),
                "Corner" if at_start => st.mode = "corner".into(),
                "Corner" if st.points.len() == 1 => st.corner_from_center = true,
                "Diameter" if at_start => st.mode = "diameter".into(),
                "FromFoci" if at_start => st.mode = "foci".into(),
                "AroundCurve" if at_start => {
                    st.mode = "around".into();
                    st.curve = CurveRef::all(e).into_iter().next();
                }
                "MarkFoci" => st.mark_foci = yes(value, st.mark_foci),
                _ => return st.reply(e, None).refuse(format!("Unknown option {name}")),
            }
            st.reply(e, None)
        }
        Input::Text { text } => match (parse_number(text), st.points.as_slice()) {
            (Some(v), [c])
                if v > 0.0
                    && !st.corner_from_center
                    && matches!(st.mode.as_str(), "center" | "vertical" | "around") =>
            {
                let dir = if st.mode == "around" {
                    let n = st.normal(e);
                    unit(if n[0].abs() < 0.9 {
                        cross(n, [1.0, 0.0, 0.0])
                    } else {
                        cross(n, [0.0, 1.0, 0.0])
                    })
                } else {
                    e.plane.x
                };
                let a_end = add(*c, scale(dir, v));
                st.points.push(a_end);
                st.reply(e, None)
            }
            (Some(v), [c, a])
                if v > 0.0 && matches!(st.mode.as_str(), "center" | "vertical" | "around") =>
            {
                let ax = sub(*a, *c);
                let side = if st.mode == "vertical" {
                    e.plane.normal()
                } else {
                    unit(cross(st.normal(e), ax))
                };
                match st.shape(*c, ax, scale(side, v)) {
                    Some(g) => st.finish(g),
                    None => st.reply(e, None).refuse("Both axes need a length"),
                }
            }
            _ => st.reply(e, None).refuse(format!("Not a length: {text}")),
        },
        Input::Undo => {
            st.points.pop();
            st.corner_from_center = false;
            st.reply(e, None)
        }
        Input::Enter => Reply::finish(vec![]),
    }
}
