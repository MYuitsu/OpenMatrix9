// SPDX-License-Identifier: LGPL-2.1-or-later
//! Polygon as Matrix (Rhino) draws it:
//! `Center of inscribed polygon ( NumSides=5 Mode=Inscribed Edge Star Vertical AroundCurve )`
//! › `Corner of polygon` (or `Midpoint of edge` when circumscribed; a number is the radius).
//!
//! - `Edge`: `Start of edge` › `End of edge ( Flip )`: the polygon on the left of the edge.
//! - `Star`: `Center of star` › `Corner of star` › `Second radius ( Automatic )`.
//! - `Vertical`: the polygon stands perpendicular to the CPlane through the picked corner.
//! - `AroundCurve`: `Select curve` (the click is the center) › corner or radius: the polygon is
//!   perpendicular to the curve there.
use crate::{load, number_of, on_plane, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, Want, add, cross, dist, dot, norm, parse_number,
    parse_point, scale, sub, unit,
};
use om9_addon_geom as g;
use om9_addon_geom::pick::CurveRef;
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};

#[derive(Serialize, Deserialize)]
struct State {
    sides: usize,
    circumscribed: bool,
    /// "" (center), "edge", "star", "vertical", "around".
    mode: String,
    points: Vec<Vec3>,
    flip: bool,
    /// AroundCurve: the curve tangent at the picked center.
    axis: Option<Vec3>,
    outer: Option<f64>,
}

impl Default for State {
    fn default() -> Self {
        State {
            sides: 5,
            circumscribed: false,
            mode: String::new(),
            points: Vec::new(),
            flip: false,
            axis: None,
            outer: None,
        }
    }
}

/// Corners of a regular polygon around `c` in the plane (`x`, `y`), first corner along `x`
/// at radius `r` (circumscribed: `r` reaches the middle of the first edge).
fn regular(c: Vec3, x: Vec3, y: Vec3, r: f64, sides: usize, circumscribed: bool) -> Vec<Vec3> {
    let (rv, start) = if circumscribed {
        (r / (PI / sides as f64).cos(), PI / sides as f64)
    } else {
        (r, 0.0)
    };
    (0..sides)
        .map(|i| {
            let a = start + TAU * i as f64 / sides as f64;
            add(c, add(scale(x, rv * a.cos()), scale(y, rv * a.sin())))
        })
        .collect()
}

/// The plane axes for the polygon around `c` toward `p`.
fn frame(e: &Event, st: &State, c: Vec3, p: Vec3) -> Option<(Vec3, Vec3, f64)> {
    let n = match (st.mode.as_str(), st.axis) {
        ("around", Some(t)) => t,
        _ => e.plane.normal(),
    };
    let d = match st.mode.as_str() {
        "vertical" => sub(on_plane(e, c, p), c),
        _ => {
            let d = sub(p, c);
            sub(d, scale(n, dot(d, n)))
        }
    };
    let r = norm(d);
    if r < g::EPS {
        return None;
    }
    let x = unit(d);
    let y = if st.mode == "vertical" {
        e.plane.normal()
    } else {
        cross(n, x)
    };
    Some((x, y, r))
}

fn shape(e: &Event, st: &State, p: Vec3) -> Option<Vec<Vec3>> {
    let pts = &st.points;
    match (st.mode.as_str(), pts.len()) {
        ("edge", 1) => {
            let (a, b) = (pts[0], on_plane(e, pts[0], p));
            let len = dist(a, b);
            if len < g::EPS {
                return None;
            }
            let x = unit(sub(b, a));
            let mut y = cross(e.plane.normal(), x);
            if st.flip {
                y = scale(y, -1.0);
            }
            // Walk the edges turning left by the exterior angle.
            let turn = TAU / st.sides as f64;
            let mut out = vec![a];
            let mut q = a;
            for k in 0..st.sides - 1 {
                let ang = turn * k as f64;
                let dir = add(scale(x, ang.cos()), scale(y, ang.sin()));
                q = add(q, scale(dir, len));
                out.push(q);
            }
            Some(out)
        }
        ("star", 1) => {
            let (x, y, r) = frame(e, st, pts[0], p)?;
            Some(regular(pts[0], x, y, r, st.sides, false))
        }
        ("star", 2) => {
            let r0 = st.outer?;
            let (x, y, _) = frame(e, st, pts[0], pts[1])?;
            let r1 = dist(pts[0], on_plane(e, pts[0], p));
            Some(star(pts[0], x, y, r0, r1, st.sides))
        }
        (_, 1) => {
            let (x, y, r) = frame(e, st, pts[0], p)?;
            Some(regular(pts[0], x, y, r, st.sides, st.circumscribed))
        }
        _ => None,
    }
}

fn star(c: Vec3, x: Vec3, y: Vec3, r0: f64, r1: f64, sides: usize) -> Vec<Vec3> {
    (0..2 * sides)
        .map(|i| {
            let a = PI * i as f64 / sides as f64;
            let r = if i % 2 == 0 { r0 } else { r1 };
            add(c, add(scale(x, r * a.cos()), scale(y, r * a.sin())))
        })
        .collect()
}

/// The inner radius that lines up the star's edges (5 or more sides).
fn automatic(r0: f64, sides: usize) -> f64 {
    let n = sides as f64;
    r0 * (TAU / n).cos() / (PI / n).cos()
}

fn ask(e: &Event, st: &State, hover: Option<Vec3>) -> Reply {
    let mut opts = vec![OptionToken::number("NumSides", Some(st.sides as f64))];
    let n = st.points.len();
    let mode_name = if st.circumscribed {
        "Circumscribed"
    } else {
        "Inscribed"
    };
    let (prompt, want) = match (st.mode.as_str(), n) {
        ("edge", 0) => ("Start of edge".to_owned(), Want::Point),
        ("edge", _) => {
            opts.push(OptionToken::toggle("Flip", st.flip));
            ("End of edge".to_owned(), Want::Point)
        }
        ("star", 0) => ("Center of star".to_owned(), Want::Point),
        ("star", 1) => ("Corner of star".to_owned(), Want::Point),
        ("star", _) => {
            if st.sides >= 5 {
                opts.push(OptionToken::action("Automatic"));
            }
            ("Second radius".to_owned(), Want::Number)
        }
        ("around", 0) => ("Select curve".to_owned(), Want::Objects),
        (_, 0) => {
            opts.push(OptionToken::choice(
                "Mode",
                mode_name,
                &["Inscribed", "Circumscribed"],
            ));
            for o in ["Edge", "Star", "Vertical", "AroundCurve"] {
                opts.push(OptionToken::action(o));
            }
            let p = if st.circumscribed {
                "Center of circumscribed polygon"
            } else {
                "Center of inscribed polygon"
            };
            (p.to_owned(), Want::Point)
        }
        _ => {
            opts.push(OptionToken::choice(
                "Mode",
                mode_name,
                &["Inscribed", "Circumscribed"],
            ));
            (
                if st.circumscribed {
                    "Midpoint of edge"
                } else {
                    "Corner of polygon"
                }
                .to_owned(),
                Want::Number,
            )
        }
    };
    let preview: Vec<Geo> = hover
        .and_then(|h| shape(e, st, h))
        .map(|p| {
            vec![Geo::Polyline {
                points: p,
                closed: true,
            }]
        })
        .unwrap_or_default();
    Reply::ask(prompt, save(st))
        .with_options(opts)
        .want(want)
        .preview(preview)
        .rubber(st.points.first().copied())
}

fn finish(e: &Event, st: &State, p: Vec3) -> Reply {
    match shape(e, st, p) {
        Some(points) => {
            let n = points.len();
            let label = if st.mode == "star" { "Star" } else { "Polygon" };
            Reply::finish(vec![Geo::Polyline {
                points,
                closed: true,
            }])
            .say(format!(
                "{label}: {} sides",
                if st.mode == "star" { n / 2 } else { n }
            ))
        }
        None => ask(e, st, None).refuse("The polygon needs a size"),
    }
}

/// A point at distance `r` from the center, along the polygon's own first direction.
fn at_radius(e: &Event, st: &State, r: f64) -> Vec3 {
    let c = st.points[0];
    let x = match (st.mode.as_str(), st.axis) {
        ("around", Some(t)) => {
            let z = e.plane.normal();
            let v = cross(t, z);
            if norm(v) < 1e-9 {
                e.plane.x
            } else {
                unit(cross(v, t))
            }
        }
        _ => e.plane.x,
    };
    add(c, scale(x, r))
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    match &e.input {
        Input::Hover { point } => ask(e, &st, Some(*point)),
        Input::Option { name, value } => {
            match name.as_str() {
                "NumSides" => match number_of(value) {
                    Some(n) if (3.0..=1000.0).contains(&n) && n.fract() == 0.0 => {
                        st.sides = n as usize
                    }
                    _ => return ask(e, &st, None).refuse("NumSides is a whole number ≥ 3"),
                },
                "Mode" => {
                    st.circumscribed = match value.as_deref() {
                        Some(v) => v.eq_ignore_ascii_case("circumscribed"),
                        None => !st.circumscribed,
                    }
                }
                "Circumscribed" => st.circumscribed = yes(value, st.circumscribed),
                "Flip" => st.flip = yes(value, st.flip),
                "Edge" | "Star" | "Vertical" | "AroundCurve" => {
                    st.mode = match name.as_str() {
                        "Edge" => "edge",
                        "Star" => "star",
                        "Vertical" => "vertical",
                        _ => "around",
                    }
                    .into();
                    st.points.clear();
                }
                "Automatic" if st.mode == "star" && st.points.len() == 2 => {
                    let r0 = st.outer.unwrap_or(1.0);
                    let r1 = automatic(r0, st.sides);
                    let (x, _, _) = frame(e, &st, st.points[0], st.points[1])
                        .unwrap_or((e.plane.x, e.plane.y, 1.0));
                    let p = add(st.points[0], scale(x, r1));
                    return finish(e, &st, p);
                }
                other => return ask(e, &st, None).refuse(format!("Unknown option {other}")),
            }
            ask(e, &st, None)
        }
        Input::Start | Input::Selection if st.mode == "around" && st.points.is_empty() => {
            if let Some((c, Some(t))) =
                CurveRef::picked(e, &[]).filter(|_| matches!(e.input, Input::Selection))
            {
                st.points.push(c.at(t));
                st.axis = Some(c.tangent(t));
            }
            ask(e, &st, None)
        }
        Input::Point { point } => {
            let p = *point;
            let n = st.points.len();
            match (st.mode.as_str(), n) {
                ("around", 0) => ask(e, &st, None),
                ("edge", 0) | ("star", 0) | (_, 0) => {
                    st.points.push(p);
                    ask(e, &st, None)
                }
                ("star", 1) => {
                    st.outer = Some(dist(st.points[0], on_plane(e, st.points[0], p)));
                    st.points.push(p);
                    ask(e, &st, None)
                }
                _ => finish(e, &st, p),
            }
        }
        Input::Text { text } => {
            let n = st.points.len();
            match (parse_number(text), st.mode.as_str(), n) {
                (Some(r), "star", 1) if r > 0.0 => {
                    let p = at_radius(e, &st, r);
                    st.outer = Some(r);
                    st.points.push(p);
                    ask(e, &st, None)
                }
                (Some(r), "star", 2) if r > 0.0 => {
                    let (x, _, _) = frame(e, &st, st.points[0], st.points[1])
                        .unwrap_or((e.plane.x, e.plane.y, 1.0));
                    let p = add(st.points[0], scale(x, r));
                    finish(e, &st, p)
                }
                (Some(l), "edge", 1) if l > 0.0 => {
                    let p = add(st.points[0], scale(e.plane.x, l));
                    finish(e, &st, p)
                }
                (Some(r), _, 1) if r > 0.0 => {
                    let p = at_radius(e, &st, r);
                    finish(e, &st, p)
                }
                _ => match parse_point(text, &e.plane, st.points.last().copied()) {
                    Some(p) if n == 0 && st.mode != "around" => {
                        st.points.push(p);
                        ask(e, &st, None)
                    }
                    Some(p) => finish(e, &st, p),
                    None => ask(e, &st, None).refuse(format!("Not a point or length: {text}")),
                },
            }
        }
        Input::Undo => {
            st.points.pop();
            ask(e, &st, None)
        }
        Input::Enter => Reply::finish(Vec::new()),
        _ => ask(e, &st, None),
    }
}
