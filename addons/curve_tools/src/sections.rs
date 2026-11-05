// SPDX-License-Identifier: LGPL-2.1-or-later
//! CSec and Section as Matrix (Rhino) runs them. Both cut with vertical planes (through a
//! picked line and the CPlane normal) and keep going until Enter:
//!
//! - CSec: `Select longitudinal curves in order. Press Enter when done`, then `Start of cross
//!   section line`, `End of cross section line`, again and again: each line makes one
//!   cross-section profile through the curves, in their order (closed for three or more).
//! - Section: `Select objects for section. Press Enter when done`, then `Start of section`,
//!   `End of section`, again and again: section curves of surfaces, solids and meshes, and
//!   points where the plane crosses curves.
use om9_addon_abi::session::{
    Event, Geo, Input, Reply, Vec3, Want, add, cross, dot, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{load, save};
use om9_addon_geom::pick::CurveRef;
use om9_addon_geom::section;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
struct State {
    selected: bool,
    start: Option<Vec3>,
    made: Vec<Geo>,
}

/// Where a curve crosses the plane (`o`, `n`): every crossing parameter.
fn crossings(c: &CurveRef, o: Vec3, n: Vec3) -> Vec<f64> {
    let f = |t: f64| dot(sub(c.at(t), o), n);
    let steps = 1024;
    let mut out = Vec::new();
    let mut prev = f(0.0);
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let v = f(t);
        if prev == 0.0 {
            out.push((i - 1) as f64 / steps as f64);
        } else if prev * v < 0.0 {
            let (mut lo, mut hi, mut flo) = ((i - 1) as f64 / steps as f64, t, prev);
            for _ in 0..60 {
                let m = (lo + hi) / 2.0;
                let fm = f(m);
                if (fm < 0.0) == (flo < 0.0) {
                    lo = m;
                    flo = fm;
                } else {
                    hi = m;
                }
            }
            out.push((lo + hi) / 2.0);
        }
        prev = v;
    }
    if prev == 0.0 && !c.closed {
        out.push(1.0);
    }
    out
}

/// The vertical plane through the line `a`–`b`: (origin, unit normal).
fn plane(e: &Event, a: Vec3, b: Vec3) -> Option<(Vec3, Vec3)> {
    let n = cross(sub(b, a), e.plane.normal());
    (norm(n) > 1e-12).then(|| (a, unit(n)))
}

/// The crossing of `c` nearest the line `a`–`b`.
fn nearest_crossing(c: &CurveRef, o: Vec3, n: Vec3, a: Vec3, b: Vec3) -> Option<Vec3> {
    let ab = sub(b, a);
    let len2 = dot(ab, ab).max(1e-30);
    crossings(c, o, n)
        .into_iter()
        .map(|t| c.at(t))
        .min_by(|p, q| {
            let d = |x: Vec3| {
                let s = (dot(sub(x, a), ab) / len2).clamp(0.0, 1.0);
                norm(sub(x, add(a, scale(ab, s))))
            };
            d(*p).total_cmp(&d(*q))
        })
}

fn profile(e: &Event, a: Vec3, b: Vec3) -> Result<Geo, String> {
    let (o, n) = plane(e, a, b).ok_or("The section line has no length")?;
    let cs = CurveRef::all(e);
    let pts: Vec<Vec3> = cs
        .iter()
        .filter_map(|c| nearest_crossing(c, o, n, a, b))
        .collect();
    if pts.len() < 2 {
        return Err("The line crosses fewer than two curves".into());
    }
    if pts.len() == 2 {
        return Ok(Geo::Line {
            start: pts[0],
            end: pts[1],
        });
    }
    om9_addon_geom::interpolate(&pts, 3, true)
        .ok_or_else(|| "No profile through these points".to_owned())
}

fn cut(e: &Event, a: Vec3, b: Vec3) -> Result<Vec<Geo>, String> {
    let (o, n) = plane(e, a, b).ok_or("The section line has no length")?;
    let mut out = Vec::new();
    for s in &e.selection {
        if s.kind == "curve" {
            if let Some(c) = CurveRef::of(s) {
                out.extend(
                    crossings(&c, o, n)
                        .into_iter()
                        .map(|t| Geo::Point { point: c.at(t) }),
                );
            }
        } else if let Some(m) = s.mesh.as_ref().filter(|m| !m.triangles.is_empty()) {
            out.extend(
                section::section(m, o, n)
                    .into_iter()
                    .map(|(points, closed)| Geo::Polyline { points, closed }),
            );
        }
    }
    Ok(out)
}

fn run(e: &Event, csec: bool) -> Reply {
    let label = if csec { "CSec" } else { "Section" };
    let mut st: State = load(e);
    if !st.selected {
        let enough = if csec {
            CurveRef::all(e).len() >= 2
        } else {
            !e.selection.is_empty()
        };
        if matches!(e.input, Input::Enter | Input::Start) && enough {
            st.selected = true;
        } else {
            let prompt = if csec {
                "Select longitudinal curves in order. Press Enter when done"
            } else {
                "Select objects for section. Press Enter when done"
            };
            let r = Reply::ask(prompt, save(&st)).want(Want::Objects);
            return if matches!(e.input, Input::Enter) {
                r.refuse(if csec {
                    "Select two or more curves"
                } else {
                    "Select objects"
                })
            } else {
                r
            };
        }
    }
    let make = |a: Vec3, b: Vec3| -> Result<Vec<Geo>, String> {
        if csec {
            profile(e, a, b).map(|g| vec![g])
        } else {
            cut(e, a, b)
        }
    };
    let mut error = None;
    let mut hover = Vec::new();
    match &e.input {
        Input::Point { point } => match st.start.take() {
            None => st.start = Some(*point),
            Some(a) => match make(a, *point) {
                Ok(g) => st.made.extend(g),
                Err(w) => {
                    st.start = Some(a);
                    error = Some(w);
                }
            },
        },
        Input::Hover { point } => {
            if let Some(a) = st.start {
                hover = make(a, *point).unwrap_or_default();
            }
        }
        Input::Undo => {
            if st.start.take().is_none() {
                st.made.pop();
            }
        }
        Input::Enter => {
            if st.made.is_empty() {
                return Reply::finish(Vec::new()).say(format!("{label}: nothing made"));
            }
            let n = st.made.len();
            return Reply::finish(st.made).say(format!("{label}: {n} curves"));
        }
        _ => {}
    }
    let prompt = match (csec, st.start.is_some()) {
        (true, false) => "Start of cross section line. Press Enter when done",
        (true, true) => "End of cross section line",
        (false, false) => "Start of section. Press Enter when done",
        (false, true) => "End of section",
    };
    let mut preview = st.made.clone();
    preview.extend(hover);
    let r = Reply::ask(prompt, save(&st))
        .want(Want::Point)
        .preview(preview)
        .rubber(st.start);
    match error {
        Some(w) => r.refuse(w),
        None => r,
    }
}

pub(crate) fn csec(e: &Event) -> Reply {
    run(e, true)
}

pub(crate) fn section_cmd(e: &Event) -> Reply {
    run(e, false)
}
