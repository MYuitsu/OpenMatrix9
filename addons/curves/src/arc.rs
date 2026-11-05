// SPDX-License-Identifier: LGPL-2.1-or-later
//! Arc as Matrix (Rhino) draws it, with the same option tree:
//!
//! - Default (center, start, angle): `Center of arc ( Deformable StartPoint Tangent
//!   Extension )` › `Start of arc` (a number is the radius) › `End point or angle ( Length
//!   Direction=CounterClockwise )`.
//! - `StartPoint`: `Start of arc` › `End of arc ( Direction ThroughPoint )` › `Point on arc
//!   ( Direction Radius )`. `Direction` (before or after the end) asks `Direction at start`,
//!   `ThroughPoint` asks the point on the arc before the end, `Radius` asks the radius and
//!   then the side.
//! - `Tangent`: `First tangent curve` › `Second tangent curve` › `Radius <1>`: the arc touching
//!   both curves near the picks.
//! - `Extension`: `Select curve to extend` (near the end) › `End of arc`: tangent to the end.
//! - `Deformable` (`Degree=3 PointCount=10`): a NURBS approximation with editable points.
//!
//! The Matrix item "Arc Start End Direction" opens straight at `StartPoint` with the direction
//! asked after the end.
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
    /// "" (center), "start", "tangent", "extension".
    mode: String,
    /// Start mode order: "" (start, end, point), "dir" (start, direction, end), "end_dir"
    /// (start, end, direction), "through" (start, point, end), "radius" (start, end, radius).
    order: String,
    points: Vec<Vec3>,
    deformable: bool,
    degree: usize,
    count: usize,
    clockwise: bool,
    radius: Option<f64>,
    curves: Vec<(CurveRef, Option<f64>)>,
    #[serde(default)]
    tangent_radius: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            mode: String::new(),
            order: String::new(),
            points: Vec::new(),
            deformable: false,
            degree: 3,
            count: 10,
            clockwise: false,
            radius: None,
            curves: Vec::new(),
            tangent_radius: 1.0,
        }
    }
}

/// Arc around `c` from `s0` sweeping `angle` (signed: + counter-clockwise about the CPlane).
fn center_arc(e: &Event, c: Vec3, s0: Vec3, angle: f64) -> Option<Geo> {
    let x = sub(on_plane(e, c, s0), c);
    let r = norm(x);
    if r < g::EPS || angle.abs() < 1e-9 {
        return None;
    }
    let x = unit(x);
    let n = e.plane.normal();
    let y = if angle > 0.0 {
        cross(n, x)
    } else {
        cross(x, n)
    };
    let a = angle.abs().min(TAU);
    if a >= TAU - 1e-12 {
        return Some(Geo::Circle {
            center: c,
            x,
            y,
            radius: r,
        });
    }
    Some(Geo::Arc {
        center: c,
        x,
        y,
        radius: r,
        angle: a,
    })
}

/// The counter-clockwise angle (0, 2π] from the start to `p` around `c`.
fn polar(e: &Event, c: Vec3, s0: Vec3, p: Vec3) -> f64 {
    let x = unit(sub(on_plane(e, c, s0), c));
    let y = cross(e.plane.normal(), x);
    let d = sub(p, c);
    let a = dot(d, y).atan2(dot(d, x));
    if a <= 0.0 { a + TAU } else { a }
}

/// Arc from `a` through `m` to `b`.
fn three_point(a: Vec3, m: Vec3, b: Vec3) -> Option<Geo> {
    g::arc_3_points(a, m, b)
}

/// Arc from `a` to `b` of radius `r`, its center on the side of `side`, the short way.
fn radius_arc(e: &Event, a: Vec3, b: Vec3, r: f64, side: Vec3) -> Option<Geo> {
    let half = dist(a, b) / 2.0;
    if r < half - 1e-12 || half < g::EPS {
        return None;
    }
    let mid = scale(add(a, b), 0.5);
    let along = unit(sub(b, a));
    let perp = cross(e.plane.normal(), along);
    let h = (r * r - half * half).max(0.0).sqrt();
    let sign = if dot(sub(side, mid), perp) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    // The short arc bulges toward the side by r − h (its center is on the other side).
    let bulge = add(mid, scale(perp, sign * (r - h)));
    three_point(a, bulge, b)
}

/// The finished arc as asked: exact, or the deformable NURBS approximation.
fn output(st: &State, arc: Geo) -> Geo {
    if !st.deformable {
        return arc;
    }
    let pts = g::nurbs::samples(&arc, 200);
    let closed = matches!(arc, Geo::Circle { .. });
    g::interp::fit(&pts, st.count.max(st.degree + 1), st.degree, closed).unwrap_or(arc)
}

fn done(st: &State, arc: Geo) -> Reply {
    let (r, a) = match &arc {
        Geo::Arc { radius, angle, .. } => (*radius, *angle),
        Geo::Circle { radius, .. } => (*radius, TAU),
        _ => (0.0, 0.0),
    };
    Reply::finish(vec![output(st, arc)]).say(format!(
        "Arc: radius {} mm, angle {}°, length {} mm",
        fmt(r),
        fmt(a.to_degrees()),
        fmt(r * a)
    ))
}

/// The prompt, what it waits for, and its options.
fn ask(e: &Event, st: &State, hover: Option<Vec3>) -> Reply {
    let mut opts = Vec::new();
    let deformable = |opts: &mut Vec<OptionToken>| {
        opts.push(OptionToken::toggle("Deformable", st.deformable));
        if st.deformable {
            opts.push(OptionToken::number("Degree", Some(st.degree as f64)));
            opts.push(OptionToken::number("PointCount", Some(st.count as f64)));
        }
    };
    let n = st.points.len();
    let (prompt, want) = match (st.mode.as_str(), st.order.as_str(), n) {
        ("", _, 0) => {
            deformable(&mut opts);
            for o in ["StartPoint", "Tangent", "Extension"] {
                opts.push(OptionToken::action(o));
            }
            ("Center of arc".to_owned(), Want::Point)
        }
        ("", _, 1) => {
            deformable(&mut opts);
            ("Start of arc".to_owned(), Want::Point)
        }
        ("", _, _) => {
            deformable(&mut opts);
            opts.push(OptionToken::number("Length", None));
            opts.push(OptionToken::choice(
                "Direction",
                if st.clockwise {
                    "Clockwise"
                } else {
                    "CounterClockwise"
                },
                &["CounterClockwise", "Clockwise"],
            ));
            ("End point or angle".to_owned(), Want::Number)
        }
        ("start", _, 0) => {
            deformable(&mut opts);
            for o in ["Center", "Tangent", "Extension"] {
                opts.push(OptionToken::action(o));
            }
            ("Start of arc".to_owned(), Want::Point)
        }
        ("start", "", 1) => {
            opts.push(OptionToken::action("Direction"));
            opts.push(OptionToken::action("ThroughPoint"));
            ("End of arc".to_owned(), Want::Point)
        }
        ("start", "", _) => {
            opts.push(OptionToken::action("Direction"));
            opts.push(OptionToken::action("Radius"));
            ("Point on arc".to_owned(), Want::Point)
        }
        ("start", "dir", 1) | ("start", "end_dir", 2) => {
            ("Direction at start".to_owned(), Want::Point)
        }
        ("start", "dir", _) | ("start", "through", _) | ("start", "end_dir", _) => {
            ("End of arc".to_owned(), Want::Point)
        }
        ("start", "radius", _) if st.radius.is_none() => ("Radius".to_owned(), Want::Number),
        ("start", "radius", _) => ("Pick the side of the arc".to_owned(), Want::Point),
        ("tangent", _, _) => match st.curves.len() {
            0 => ("First tangent curve".to_owned(), Want::Objects),
            1 => ("Second tangent curve".to_owned(), Want::Objects),
            _ => (format!("Radius <{}>", fmt(st.tangent_radius)), Want::Number),
        },
        ("extension", _, _) if st.curves.is_empty() => {
            ("Select curve to extend".to_owned(), Want::Objects)
        }
        ("extension", _, _) => ("End of arc".to_owned(), Want::Point),
        _ => ("Center of arc".to_owned(), Want::Point),
    };
    if !st.points.is_empty() {
        opts.push(OptionToken::action("Undo"));
    }
    let preview: Vec<Geo> = hover
        .and_then(|h| candidate(e, st, h))
        .into_iter()
        .collect();
    let rubber = match (st.mode.as_str(), st.points.first()) {
        ("", Some(c)) => Some(*c),
        (_, Some(_)) => st.points.last().copied(),
        _ => None,
    };
    Reply::ask(prompt, save(st))
        .with_options(opts)
        .want(want)
        .preview(preview)
        .rubber(rubber)
}

/// The arc that clicking `p` now would make (also the preview).
fn candidate(e: &Event, st: &State, p: Vec3) -> Option<Geo> {
    let pts = &st.points;
    match (st.mode.as_str(), st.order.as_str(), pts.len()) {
        ("", _, 1) => Some(Geo::Circle {
            center: pts[0],
            x: e.plane.x,
            y: e.plane.y,
            radius: dist(pts[0], on_plane(e, pts[0], p)).max(1e-6),
        }),
        ("", _, 2) => {
            let a = polar(e, pts[0], pts[1], p);
            center_arc(e, pts[0], pts[1], if st.clockwise { a - TAU } else { a })
        }
        ("start", "", 1) | ("start", "dir", 1) | ("start", "through", 1) => Some(Geo::Line {
            start: pts[0],
            end: p,
        }),
        ("start", "", 2) => three_point(pts[0], p, pts[1]),
        ("start", "dir", 2) => g::arc_start_end_dir(pts[0], p, sub(pts[1], pts[0])),
        ("start", "end_dir", 2) => g::arc_start_end_dir(pts[0], pts[1], sub(p, pts[0])),
        ("start", "through", 2) => three_point(pts[0], pts[1], p),
        ("start", "radius", 2) => st.radius.and_then(|r| radius_arc(e, pts[0], pts[1], r, p)),
        ("extension", _, _) => {
            let (c, t) = st.curves.first()?;
            let at_end = t.unwrap_or(1.0) >= 0.5;
            let (q, tan) = if at_end {
                (c.at(1.0), c.tangent(1.0))
            } else {
                (c.at(0.0), scale(c.tangent(0.0), -1.0))
            };
            g::blend::arc_from(q, tan, on_plane(e, q, p))
        }
        _ => None,
    }
}

fn tangent_arc(e: &Event, st: &State, r: f64) -> Option<Geo> {
    let (a, ta) = st.curves.first()?;
    let (b, tb) = st.curves.get(1)?;
    let (pa, pb) = (ta.unwrap_or(0.5), tb.unwrap_or(0.5));
    let n = g::blend::plane_normal(a, pa, b, pb, e.plane.normal());
    let (sa, sb, c) = g::blend::tangent_circle(a, pa, b, pb, n, r)?;
    g::blend::arc_between(c, a.at(sa), b.at(sb))
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    if matches!(e.input, Input::Start) && e.command == crate::ARC_SED {
        st.mode = "start".into();
        st.order = "end_dir".into();
    }
    // Options.
    if let Input::Option { name, value } = &e.input {
        let mut error = None;
        match name.as_str() {
            "Deformable" => st.deformable = yes(value, st.deformable),
            "Degree" => match number_of(value) {
                Some(d) if (1.0..=11.0).contains(&d) && d.fract() == 0.0 => st.degree = d as usize,
                _ => error = Some("Degree is a whole number from 1 to 11".to_owned()),
            },
            "PointCount" => match number_of(value) {
                Some(n) if n >= 2.0 && n.fract() == 0.0 => st.count = n as usize,
                _ => error = Some("PointCount is a whole number from 3".to_owned()),
            },
            "StartPoint" => {
                st.mode = "start".into();
                st.order.clear();
                st.points.clear();
            }
            "Center" => {
                st.mode.clear();
                st.order.clear();
                st.points.clear();
            }
            "Tangent" => {
                st.mode = "tangent".into();
                st.points.clear();
                st.curves.clear();
            }
            "Extension" => {
                st.mode = "extension".into();
                st.points.clear();
                st.curves.clear();
            }
            "Direction" if st.mode == "start" => {
                st.order = if st.points.len() >= 2 {
                    "end_dir"
                } else {
                    "dir"
                }
                .into()
            }
            "Direction" => {
                st.clockwise = match value.as_deref() {
                    Some(v) => v.eq_ignore_ascii_case("clockwise"),
                    None => !st.clockwise,
                }
            }
            "ThroughPoint" => st.order = "through".into(),
            "Radius" if st.mode == "start" => st.order = "radius".into(),
            "Length" if st.mode.is_empty() && st.points.len() == 2 => match number_of(value) {
                Some(l) if l > 0.0 => {
                    let r = dist(st.points[0], on_plane(e, st.points[0], st.points[1]));
                    let a = l / r.max(g::EPS);
                    return match center_arc(
                        e,
                        st.points[0],
                        st.points[1],
                        if st.clockwise { -a } else { a },
                    ) {
                        Some(arc) => done(&st, arc),
                        None => ask(e, &st, None).refuse("The arc needs a radius"),
                    };
                }
                _ => error = Some("Length: type Length=number".to_owned()),
            },
            "Undo" => {
                st.points.pop();
            }
            other => error = Some(format!("Unknown option {other}")),
        }
        let r = ask(e, &st, None);
        return match error {
            Some(w) => r.refuse(w),
            None => r,
        };
    }
    // Curve picks (Tangent, Extension).
    if (st.mode == "tangent" && st.curves.len() < 2)
        || (st.mode == "extension" && st.curves.is_empty())
    {
        if matches!(e.input, Input::Selection | Input::Start) {
            let taken: Vec<&str> = st.curves.iter().map(|c| c.0.id.as_str()).collect();
            if let Some((c, t)) = CurveRef::picked(e, &taken)
                && !matches!(e.input, Input::Start)
            {
                st.curves.push((c, t));
            }
        }
        if let Input::Text { text } = &e.input
            && st.mode == "tangent"
        {
            match parse_number(text) {
                Some(r) if r > 0.0 => st.tangent_radius = r,
                _ => return ask(e, &st, None).refuse(format!("Not a radius: {text}")),
            }
        }
        return ask(e, &st, None);
    }
    if st.mode == "tangent" {
        let r = match &e.input {
            Input::Text { text } => match parse_number(text) {
                Some(r) if r > 0.0 => r,
                _ => return ask(e, &st, None).refuse(format!("Not a radius: {text}")),
            },
            Input::Enter => st.tangent_radius,
            Input::Point { point } => {
                // A click gives the radius as the distance from the first touch.
                let (a, ta) = &st.curves[0];
                dist(a.at(ta.unwrap_or(0.5)), *point)
            }
            _ => return ask(e, &st, None),
        };
        st.tangent_radius = r;
        return match tangent_arc(e, &st, r) {
            Some(arc) => done(&st, arc),
            None => ask(e, &st, None).refuse("No arc of that radius touches both curves"),
        };
    }
    let point = match &e.input {
        Input::Hover { point } => return ask(e, &st, Some(*point)),
        Input::Point { point } => Some(*point),
        Input::Text { text } => {
            let n = st.points.len();
            // Numbers first: radius (after the center), angle, or the arc radius.
            if let Some(v) = parse_number(text) {
                match (st.mode.as_str(), st.order.as_str(), n) {
                    ("", _, 1) if v > 0.0 => Some(add(st.points[0], scale(e.plane.x, v))),
                    ("", _, 2) => {
                        let a = v.to_radians();
                        return match center_arc(
                            e,
                            st.points[0],
                            st.points[1],
                            if st.clockwise { -a } else { a },
                        ) {
                            Some(arc) => done(&st, arc),
                            None => ask(e, &st, None).refuse("The angle must not be 0"),
                        };
                    }
                    ("start", "radius", 2) if st.radius.is_none() && v > 0.0 => {
                        if v < dist(st.points[0], st.points[1]) / 2.0 - 1e-12 {
                            return ask(e, &st, None).refuse(
                                "The radius is less than half the distance between the ends",
                            );
                        }
                        st.radius = Some(v);
                        return ask(e, &st, None);
                    }
                    _ => parse_point(text, &e.plane, st.points.last().copied()),
                }
            } else {
                match parse_point(text, &e.plane, st.points.last().copied()) {
                    Some(p) => Some(p),
                    None => {
                        return ask(e, &st, None).refuse(format!("Not a point or length: {text}"));
                    }
                }
            }
        }
        Input::Undo => {
            st.points.pop();
            return ask(e, &st, None);
        }
        Input::Enter => return Reply::finish(Vec::new()),
        _ => return ask(e, &st, None),
    };
    let Some(p) = point else {
        return ask(e, &st, None).refuse("Type a point or a number");
    };
    // A click that completes the arc, or one more point.
    let completes = match (st.mode.as_str(), st.order.as_str(), st.points.len()) {
        ("", _, 2) => true,
        ("start", "radius", 2) => st.radius.is_some(),
        ("start", _, 2) => true,
        ("extension", _, _) => true,
        _ => false,
    };
    if st.mode == "start" && st.order == "radius" && st.points.len() == 2 && st.radius.is_none() {
        // A click for the radius: its distance from the start.
        let r = dist(st.points[0], p);
        if r < dist(st.points[0], st.points[1]) / 2.0 - 1e-12 {
            return ask(e, &st, None)
                .refuse("The radius is less than half the distance between the ends");
        }
        st.radius = Some(r);
        return ask(e, &st, None);
    }
    if completes {
        return match candidate(e, &st, p) {
            Some(arc @ (Geo::Arc { .. } | Geo::Circle { .. })) => done(&st, arc),
            Some(other) => Reply::finish(vec![output(&st, other)]).say("Arc"),
            None => ask(e, &st, None).refuse("No arc through these points"),
        };
    }
    if st.points.last().is_some_and(|q| dist(*q, p) < g::EPS) {
        return ask(e, &st, None).refuse("The point repeats the last one");
    }
    st.points.push(p);
    ask(e, &st, None)
}
