// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curves on and from surfaces as Matrix (Rhino) runs them:
//!
//! - Project: `Select curves and points to project ( Loose=No DeleteInput=No )`, Enter, then
//!   `Select surfaces, polysurfaces and meshes to project onto`, Enter: straight down the
//!   CPlane normal. Pull: the same with the closest points (`… to pull`, `… to pull onto`).
//!   `Loose=Yes` moves the control points only (same structure); otherwise the projected
//!   samples are fitted within the tolerance and split where the curve leaves the surface.
//! - OffsetCrvOnSrf: `Select curve on surface to offset`, `Select base surface`, `Side to
//!   offset ( Distance=1 ThroughPoint=No )`: the offset walks along the surface.
//! - OffsetNormal: `Select curve on surface`, `Select surface`, `Offset distance <1>`: along
//!   the surface normal.
//! - ExtractIsocurve: `Select surface for isocurve extraction`, then `Select isocurve location
//!   on surface ( Direction=U Toggle ExtractAll )` again and again, Enter.
//! - SoftEditCrv: `Select curve to edit`, `Point on curve to edit ( Distance=1 Copy=No
//!   FixEnds=No )`, `Point to move to`: control points move with a smooth falloff along the
//!   curve.
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Replacement, Reply, Selected, Vec3, Want, add, cross, dist,
    dot, fmt, parse_number, scale, sub, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save};
use om9_addon_geom::interp;
use om9_addon_geom::nurbs::Srf;
use om9_addon_geom::path::Target;
use om9_addon_geom::pick::{CurveRef, SurfRef};
use serde::{Deserialize, Serialize};

const TOL: f64 = 0.001;

/// A target to put curves on: its render mesh, and the exact surface when there is one.
struct Onto<'a> {
    sel: &'a Selected,
    srf: Option<Srf<'a>>,
}

fn targets<'a>(e: &'a Event, skip: &[String]) -> Vec<Onto<'a>> {
    e.selection
        .iter()
        .filter(|s| !skip.contains(&s.id) && s.kind != "curve" && s.kind != "point")
        .filter(|s| {
            s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty())
                || s.exact.as_ref().and_then(Srf::of).is_some()
        })
        .map(|s| Onto {
            sel: s,
            srf: s.exact.as_ref().and_then(Srf::of),
        })
        .collect()
}

/// The point straight down `n` (the nearest hit over all targets, from above).
fn drop_on(t: &[(Option<Target>, Option<&Srf>)], p: Vec3, n: Vec3) -> Vec<Vec3> {
    // Every crossing of the line through p along n, on every target, from the top down
    // (Rhino projects both ways: a curve over a ring lands on its top and its bottom).
    let mut all: Vec<(f64, Vec3)> = Vec::new();
    for (mesh, srf) in t {
        let Some(m) = mesh.as_ref() else { continue };
        for (h0, _) in m.hits(p) {
            let mut h = h0;
            // Refine onto the exact surface: back to the line through p along n.
            if let Some(s) = srf {
                for _ in 0..8 {
                    let (u, v) = s.closest_st(h);
                    let q = s.at_st(u, v);
                    let m = unit(s.normal_st(u, v));
                    let den = dot(n, m);
                    if den.abs() < 1e-9 {
                        break;
                    }
                    let t = dot(sub(q, p), m) / den;
                    h = add(p, scale(n, t));
                }
            }
            all.push((dot(sub(h, p), n), h));
        }
    }
    all.sort_by(|a, b| b.0.total_cmp(&a.0));
    all.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-6);
    all.into_iter().map(|x| x.1).collect()
}

/// The closest point over all targets.
fn pull_on(t: &[Onto], p: Vec3) -> Option<Vec3> {
    t.iter()
        .filter_map(|o| match (&o.srf, o.sel.mesh.as_ref()) {
            (Some(s), _) => {
                let (u, v) = s.closest_st(p);
                Some(s.at_st(u, v))
            }
            (None, Some(m)) => om9_addon_geom::section::closest_on_mesh(m, p),
            _ => None,
        })
        .min_by(|a, b| dist(*a, p).total_cmp(&dist(*b, p)))
}

/// Points fitted within `TOL`, as few poles as needed.
fn fit_points(pts: &[Vec3], closed: bool) -> Option<Geo> {
    if pts.len() < 2 {
        return None;
    }
    if pts.len() < 4 {
        return Some(Geo::Polyline {
            points: pts.to_vec(),
            closed,
        });
    }
    let mut count = 8usize.min(pts.len() - 1);
    loop {
        let g = interp::fit(pts, count, 3, closed)?;
        let c = CurveRef {
            id: String::new(),
            pts: pts.to_vec(),
            closed,
            exact: Some(g.clone()),
        };
        let worst = pts
            .iter()
            .step_by(3)
            .map(|p| dist(c.closest(*p).1, *p))
            .fold(0.0, f64::max);
        if worst <= TOL || count >= pts.len() - 1 || count >= 256 {
            return Some(g);
        }
        count = (count * 2).min(pts.len() - 1);
    }
}

/// Puts moved control points back into a curve.
type Rebuild = Box<dyn Fn(Vec<Vec3>) -> Geo>;

/// Map a curve point by point, each sample landing on one or more layers (top first): one
/// result per layer, split where the curve leaves it.
fn mapped(c: &CurveRef, loose: bool, f: &dyn Fn(Vec3) -> Vec<Vec3>) -> Vec<Geo> {
    if loose {
        // Control points only, layer by layer while every pole has one.
        let g = match c.geo() {
            g @ Geo::Nurbs { .. } => g,
            other => {
                om9_addon_geom::join::join(std::slice::from_ref(&other), 1e-9).unwrap_or(other)
            }
        };
        let (poles, rebuild): (Vec<Vec3>, Rebuild) = match g {
            Geo::Nurbs {
                degree,
                points,
                weights,
                knots,
            } => (
                points,
                Box::new(move |points| Geo::Nurbs {
                    degree,
                    points,
                    weights: weights.clone(),
                    knots: knots.clone(),
                }),
            ),
            Geo::Polyline { points, closed } => (
                points,
                Box::new(move |points| Geo::Polyline { points, closed }),
            ),
            _ => return Vec::new(),
        };
        let hits: Vec<Vec<Vec3>> = poles.iter().map(|p| f(*p)).collect();
        let layers = hits.iter().map(Vec::len).min().unwrap_or(0);
        return (0..layers)
            .map(|k| rebuild(hits.iter().map(|h| h[k]).collect()))
            .collect();
    }
    let breaks = corners(c);
    // Samples: every corner exactly, and a dense spread between them.
    let mut ts: Vec<f64> = (0..=400).map(|i| i as f64 / 400.0).collect();
    ts.extend(breaks.iter().copied());
    ts.sort_by(f64::total_cmp);
    ts.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    if c.closed {
        ts.pop(); // t = 1 is t = 0
    }
    let hits: Vec<Vec<Vec3>> = ts.iter().map(|t| f(c.at(*t))).collect();
    let layers = hits.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = Vec::new();
    for k in 0..layers {
        let on = |t: f64| f(c.at(t)).get(k).copied();
        // Runs of (t, point) on this layer, each ending exactly where the curve leaves it.
        let mut runs: Vec<Vec<(f64, Vec3)>> = Vec::new();
        let mut run: Vec<(f64, Vec3)> = Vec::new();
        let count = ts.len();
        for i in 0..count {
            let here = hits[i].get(k).copied();
            match here {
                Some(p) => {
                    if run.is_empty() && i > 0 && hits[i - 1].get(k).is_none() {
                        // Entering: find the edge between the previous sample and this one.
                        if let Some(edge) = edge_between(&on, ts[i - 1], ts[i]) {
                            run.push(edge);
                        }
                    }
                    run.push((ts[i], p));
                }
                None => {
                    if !run.is_empty() {
                        if let Some(edge) = edge_between(&on, ts[i], ts[i - 1]) {
                            run.push(edge);
                        }
                        runs.push(std::mem::take(&mut run));
                    }
                }
            }
        }
        let whole = c.closed && runs.is_empty();
        if !run.is_empty() {
            if c.closed && !whole && hits[0].get(k).is_some() && !runs.is_empty() {
                // The run through the seam of a closed curve is one piece.
                let mut tail = run;
                tail.extend(runs.remove(0).into_iter().map(|(t, p)| (t + 1.0, p)));
                runs.push(tail);
            } else {
                runs.push(run);
            }
        }
        for mut run in runs {
            if whole {
                run.push((1.0, run[0].1));
            }
            out.extend(piece(&run, &breaks, whole));
        }
    }
    out
}

/// Normalized parameters where the curve turns a corner (polyline vertices, kinks of a
/// NURBS at full-multiplicity knots), so a projected polygon keeps its sharp corners.
fn corners(c: &CurveRef) -> Vec<f64> {
    let candidates: Vec<f64> = match c.exact.as_ref() {
        Some(Geo::Polyline { points, closed }) => {
            let segs = points.len() - usize::from(!*closed);
            (0..=segs).map(|i| i as f64 / segs as f64).collect()
        }
        Some(Geo::Nurbs {
            degree,
            points,
            knots,
            ..
        }) => {
            let (a, b) = (knots[*degree], knots[points.len()]);
            let mut v: Vec<f64> = Vec::new();
            let inner = &knots[*degree + 1..points.len()];
            let mut i = 0;
            while i < inner.len() {
                let mut j = i;
                while j < inner.len() && (inner[j] - inner[i]).abs() < 1e-12 {
                    j += 1;
                }
                if j - i >= *degree && b > a {
                    v.push((inner[i] - a) / (b - a));
                }
                i = j;
            }
            v.push(0.0);
            v.push(1.0);
            v
        }
        _ => vec![0.0, 1.0],
    };
    let h = 1e-7;
    let mut out: Vec<f64> = candidates
        .into_iter()
        .filter(|&t| {
            let (before, after) = if t <= 0.0 || t >= 1.0 {
                // An open curve's ends are not corners; a closed curve's seam may be one.
                if !c.closed {
                    return false;
                }
                (
                    unit(sub(c.at(1.0), c.at(1.0 - h))),
                    unit(sub(c.at(h), c.at(0.0))),
                )
            } else {
                (
                    unit(sub(c.at(t), c.at(t - h))),
                    unit(sub(c.at(t + h), c.at(t))),
                )
            };
            dot(before, after) < (0.5f64).to_radians().cos()
        })
        .collect();
    out.sort_by(f64::total_cmp);
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    out
}

/// The last point on the layer going from `inside` (on it) toward `outside` (off it).
fn edge_between(
    on: &dyn Fn(f64) -> Option<Vec3>,
    outside: f64,
    inside: f64,
) -> Option<(f64, Vec3)> {
    let (mut off, mut good) = (outside, inside);
    let mut best = None;
    for _ in 0..30 {
        let mid = 0.5 * (off + good);
        match on(mid) {
            Some(p) => {
                good = mid;
                best = Some((mid, p));
            }
            None => off = mid,
        }
    }
    best
}

/// One projected piece: split at the corners, each span a line when it is straight or a
/// fitted curve, joined back into one curve.
fn piece(run: &[(f64, Vec3)], breaks: &[f64], closed: bool) -> Option<Geo> {
    if run.len() < 2 {
        return None;
    }
    let is_break = |t: f64| {
        let t = t.rem_euclid(1.0);
        breaks
            .iter()
            .any(|b| (b - t).abs() < 1e-9 || (b - t - 1.0).abs() < 1e-9 || (t - b - 1.0).abs() < 1e-9)
    };
    let mut spans: Vec<Vec<Vec3>> = vec![vec![run[0].1]];
    for (i, (t, p)) in run.iter().enumerate().skip(1) {
        spans.last_mut()?.push(*p);
        if i + 1 < run.len() && is_break(*t) {
            spans.push(vec![*p]);
        }
    }
    if closed && spans.len() == 1 {
        // A smooth closed curve all on the surface stays closed and smooth.
        let mut pts = spans.remove(0);
        pts.pop();
        return fit_points(&pts, true);
    }
    let geos: Vec<Geo> = spans
        .iter()
        .filter(|s| s.len() >= 2 && dist(s[0], s[s.len() - 1]) + chord(s) > 1e-9)
        .filter_map(|s| {
            let (a, b) = (s[0], s[s.len() - 1]);
            let straight = dist(a, b) > 1e-9
                && s.iter().all(|p| {
                    let d = unit(sub(b, a));
                    let v = sub(*p, a);
                    dist(*p, add(a, scale(d, dot(v, d)))) <= TOL * 0.1
                });
            if straight {
                Some(Geo::Line { start: a, end: b })
            } else {
                fit_points(s, false)
            }
        })
        .collect();
    match geos.len() {
        0 => None,
        1 if !closed => geos.into_iter().next(),
        _ => om9_addon_geom::join::join(&geos, 1e-6).or_else(|| geos.into_iter().next()),
    }
}

fn chord(s: &[Vec3]) -> f64 {
    s.windows(2).map(|w| dist(w[0], w[1])).sum()
}

#[derive(Default, Serialize, Deserialize)]
struct TwoSets {
    first: Vec<String>,
    stage: u8,
    loose: bool,
    delete: bool,
}

pub(crate) fn project(e: &Event, pull: bool) -> Reply {
    let mut st: TwoSets = load(e);
    let label = if pull { "Pull" } else { "Project" };
    let options = |st: &TwoSets| {
        vec![
            OptionToken::toggle("Loose", st.loose),
            OptionToken::toggle("DeleteInput", st.delete),
        ]
    };
    let ask = |st: &TwoSets| -> Reply {
        let prompt = match (st.stage, pull) {
            (0, false) => "Select curves and points to project. Press Enter when done",
            (0, true) => "Select curves and points to pull. Press Enter when done",
            (_, false) => {
                "Select surfaces, polysurfaces and meshes to project onto. Press Enter when done"
            }
            (_, true) => "Select surface, polysurface or mesh to pull onto. Press Enter when done",
        };
        Reply::ask(prompt, save(st))
            .with_options(options(st))
            .want(Want::Objects)
    };
    if let Input::Option { name, value } = &e.input {
        let flag = |cur: bool| match value.as_deref() {
            Some(v) => v.eq_ignore_ascii_case("yes"),
            None => !cur,
        };
        match name.to_ascii_lowercase().as_str() {
            "loose" => st.loose = flag(st.loose),
            "deleteinput" => st.delete = flag(st.delete),
            _ => return ask(&st).refuse(format!("Unknown option {name}")),
        }
        return ask(&st);
    }
    if !matches!(e.input, Input::Enter) {
        return ask(&st);
    }
    if st.stage == 0 {
        st.first = e
            .selection
            .iter()
            .filter(|s| (s.kind == "curve" && s.points.len() >= 2) || s.kind == "point")
            .map(|s| s.id.clone())
            .collect();
        if st.first.is_empty() {
            return ask(&st).refuse("Select curves or points");
        }
        st.stage = 1;
        return ask(&st);
    }
    let onto = targets(e, &st.first);
    if onto.is_empty() {
        return ask(&st).refuse("Select a surface, polysurface or mesh");
    }
    let n = e.plane.normal();
    let meshes: Vec<(Option<Target>, Option<&Srf>)> = onto
        .iter()
        .map(|o| {
            (
                o.sel.mesh.as_ref().and_then(|m| Target::new(m, n)),
                o.srf.as_ref(),
            )
        })
        .collect();
    let f = |p: Vec3| -> Vec<Vec3> {
        if pull {
            pull_on(&onto, p).into_iter().collect()
        } else {
            drop_on(&meshes, p, n)
        }
    };
    let mut made = Vec::new();
    let mut used = Vec::new();
    for s in e.selection.iter().filter(|s| st.first.contains(&s.id)) {
        let before = made.len();
        if s.kind == "point" {
            let p = s.exact.as_ref().and_then(|g| match g {
                Geo::Point { point } => Some(*point),
                _ => None,
            });
            let p = p.unwrap_or_else(|| scale(add(s.bbox[0], s.bbox[1]), 0.5));
            made.extend(f(p).into_iter().map(|q| Geo::Point { point: q }));
        } else if let Some(c) = CurveRef::of(s) {
            made.extend(mapped(&c, st.loose, &f));
        }
        if made.len() > before {
            used.push(s.id.clone());
        }
    }
    if made.is_empty() {
        return ask(&st).refuse(format!("{label}: nothing lands on the surface"));
    }
    let k = made.len();
    let mut r = Reply::finish(made).say(format!("{label}: {k} objects"));
    if st.delete {
        r.delete = used;
    }
    r
}

// ---- offsets on surfaces ---------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct OnSurface {
    curve: Option<CurveRef>,
    surface: Option<SurfRef>,
}

/// Curve then surface, picked one after the other; `Some` when both are in.
fn curve_and_surface(e: &Event, st: &mut OnSurface) -> bool {
    if st.curve.is_none() && matches!(e.input, Input::Selection | Input::Start) {
        st.curve = CurveRef::picked(e, &[]).map(|c| c.0);
    }
    if st.curve.is_some()
        && st.surface.is_none()
        && matches!(e.input, Input::Selection | Input::Start | Input::Enter)
    {
        let cid = st.curve.as_ref().map(|c| c.id.clone()).unwrap_or_default();
        st.surface = e
            .selection
            .iter()
            .filter(|s| s.id != cid)
            .find_map(SurfRef::of);
    }
    st.curve.is_some() && st.surface.is_some()
}

/// Walk `d` along the surface from `p` sideways to the curve direction `t`.
fn walk(s: &SurfRef, p: Vec3, t: Vec3, d: f64) -> Option<Vec3> {
    let steps = 8;
    let mut q = s.closest(p)?.0;
    for _ in 0..steps {
        let (_, nrm) = s.closest(q)?;
        let side = unit(cross(nrm, t));
        q = s.closest(add(q, scale(side, d / steps as f64)))?.0;
    }
    Some(q)
}

pub(crate) fn offset_on_surface(e: &Event) -> Reply {
    let mut st: OnSurface = load(e);
    let ready = curve_and_surface(e, &mut st);
    let flow = Flow::new("Offset Curve on Surface")
        .no_selection()
        .plain()
        .point("Side to offset")
        .num("Distance", 1.0)
        .toggle("ThroughPoint", false);
    if !ready {
        let prompt = if st.curve.is_none() {
            "Select curve on surface to offset"
        } else {
            "Select base surface"
        };
        return Reply::ask(prompt, save(&st)).want(Want::Objects);
    }
    // The rest is a flow on its own state, nested in ours.
    let (c, s) = (st.curve.clone().unwrap(), st.surface.clone().unwrap());
    let make = |col: &Collected, side: Vec3| -> Result<Geo, String> {
        let (t0, q0) = c.closest(side);
        let (_, nrm) = s.closest(q0).ok_or("The curve is not on the surface")?;
        let sign = if dot(sub(side, q0), cross(nrm, c.tangent(t0))) >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let d = if col.yes(&flow, "ThroughPoint") {
            dist(side, q0)
        } else {
            col.num(&flow, "Distance")
        };
        if d <= 0.0 {
            return Err("Distance must be more than 0".into());
        }
        let n = 120;
        let m = if c.closed { n } else { n + 1 };
        let pts: Option<Vec<Vec3>> = (0..m)
            .map(|i| {
                let t = i as f64 / n as f64;
                walk(&s, c.at(t), c.tangent(t), sign * d)
            })
            .collect();
        fit_points(&pts.ok_or("The offset leaves the surface")?, c.closed)
            .ok_or_else(|| "Offset failed".to_owned())
    };
    let mut inner = e.clone();
    inner.state = e
        .state
        .get("flow")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if matches!(e.input, Input::Selection | Input::Start) {
        inner.input = Input::Start;
    }
    inner.selection.clear();
    let mut r = flow.step(
        &inner,
        &|col, h| Show {
            geo: h.and_then(|p| make(col, p).ok()).into_iter().collect(),
            ..Default::default()
        },
        &|col| {
            let g = make(col, col.p(0).ok_or("Side to offset")?)?;
            Ok(Reply::finish(vec![g]).say("Offset Curve on Surface: done"))
        },
    );
    let mut outer = save(&st);
    if let Some(o) = outer.as_object_mut() {
        o.insert("flow".into(), r.state.clone());
    }
    r.state = outer;
    r
}

pub(crate) fn offset_normal(e: &Event) -> Reply {
    let mut st: OnSurface = load(e);
    let ready = curve_and_surface(e, &mut st);
    if !ready {
        let prompt = if st.curve.is_none() {
            "Select curve on surface"
        } else {
            "Select surface"
        };
        return Reply::ask(prompt, save(&st)).want(Want::Objects);
    }
    let (c, s) = (st.curve.clone().unwrap(), st.surface.clone().unwrap());
    let make = |d: f64| -> Option<Geo> {
        let n = 120;
        let m = if c.closed { n } else { n + 1 };
        let pts: Option<Vec<Vec3>> = (0..m)
            .map(|i| {
                let p = c.at(i as f64 / n as f64);
                s.closest(p).map(|(q, nrm)| add(q, scale(nrm, d)))
            })
            .collect();
        fit_points(&pts?, c.closed)
    };
    let d = match &e.input {
        Input::Text { text } => match parse_number(text) {
            Some(v) => v,
            None => {
                return Reply::ask("Offset distance <1>", save(&st))
                    .want(Want::Number)
                    .refuse(format!("Not a number: {text}"));
            }
        },
        Input::Enter => 1.0,
        Input::Point { point } => s.closest(c.at(0.0)).map_or(1.0, |(q, _)| dist(q, *point)),
        Input::Hover { point } => {
            let d = s.closest(c.at(0.0)).map_or(1.0, |(q, _)| dist(q, *point));
            return Reply::ask("Offset distance <1>", save(&st))
                .want(Want::Number)
                .preview(make(d).into_iter().collect());
        }
        _ => return Reply::ask("Offset distance <1>", save(&st)).want(Want::Number),
    };
    match make(d) {
        Some(g) => {
            Reply::finish(vec![g]).say(format!("Offset Curve Normal to Surface: {} mm", fmt(d)))
        }
        None => Reply::ask("Offset distance <1>", save(&st))
            .want(Want::Number)
            .refuse("The curve is not on the surface"),
    }
}

// ---- isocurves -------------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct Iso {
    surface: Option<String>,
    dir: String,
    made: Vec<Geo>,
}

pub(crate) fn isocurve(e: &Event) -> Reply {
    let mut st: Iso = load(e);
    if st.dir.is_empty() {
        st.dir = "U".into();
    }
    let opts = |st: &Iso| {
        vec![
            OptionToken::choice("Direction", &st.dir, &["U", "V", "Both"]),
            OptionToken::action("Toggle"),
            OptionToken::action("ExtractAll"),
        ]
    };
    let srf_sel = |st: &Iso| -> Option<&Selected> {
        e.selection
            .iter()
            .find(|s| Some(&s.id) == st.surface.as_ref())
    };
    if st.surface.is_none() {
        st.surface = e
            .selection
            .iter()
            .find(|s| s.exact.as_ref().and_then(Srf::of).is_some())
            .map(|s| s.id.clone());
        if st.surface.is_none() {
            return Reply::ask("Select surface for isocurve extraction", save(&st))
                .want(Want::Objects);
        }
    }
    let Some(sel) = srf_sel(&st) else {
        st.surface = None;
        return Reply::ask("Select surface for isocurve extraction", save(&st)).want(Want::Objects);
    };
    let s = sel.exact.as_ref().and_then(Srf::of).expect("surface");
    let n = e.plane.normal();
    let target = sel.mesh.as_ref().and_then(|m| Target::new(m, n));
    let at = |p: Vec3| -> (f64, f64) {
        let p = target
            .as_ref()
            .and_then(|t| t.drop(add(p, scale(n, 1e6))))
            .map_or(p, |h| h.0);
        s.closest_st(p)
    };
    let iso = |st: &Iso, (u, v): (f64, f64)| -> Vec<Geo> {
        match st.dir.as_str() {
            "V" => vec![s.isocurve(1, v)],
            "Both" => vec![s.isocurve(0, u), s.isocurve(1, v)],
            _ => vec![s.isocurve(0, u)],
        }
    };
    let ask = |st: &Iso, hover: Vec<Geo>| {
        let mut preview = st.made.clone();
        preview.extend(hover);
        Reply::ask(
            "Select isocurve location on surface. Press Enter when done",
            save(st),
        )
        .with_options(opts(st))
        .want(Want::Point)
        .preview(preview)
    };
    match &e.input {
        Input::Hover { point } => ask(&st, iso(&st, at(*point))),
        Input::Point { point } => {
            let g = iso(&st, at(*point));
            st.made.extend(g);
            ask(&st, Vec::new())
        }
        Input::Option { name, value } => {
            match name.as_str() {
                "Direction" => {
                    st.dir = match value.as_deref() {
                        Some(v) => ["U", "V", "Both"]
                            .iter()
                            .find(|d| d.eq_ignore_ascii_case(v))
                            .unwrap_or(&"U")
                            .to_string(),
                        None => match st.dir.as_str() {
                            "U" => "V",
                            "V" => "Both",
                            _ => "U",
                        }
                        .into(),
                    }
                }
                "Toggle" => {
                    st.dir = match st.dir.as_str() {
                        "U" => "V",
                        "V" => "U",
                        d => d,
                    }
                    .into()
                }
                "ExtractAll" => {
                    // Every knot line (and the edges) in the chosen direction(s).
                    let Geo::Surface {
                        degree,
                        size,
                        knots,
                        ..
                    } = sel.exact.as_ref().expect("surface")
                    else {
                        return ask(&st, Vec::new());
                    };
                    for dir in 0..2 {
                        let wanted = match st.dir.as_str() {
                            "U" => dir == 0,
                            "V" => dir == 1,
                            _ => true,
                        };
                        if !wanted {
                            continue;
                        }
                        let k = &knots[dir];
                        let (lo, hi) = (k[degree[dir]], k[size[dir]]);
                        let mut ks: Vec<f64> =
                            k.iter().copied().filter(|x| *x >= lo && *x <= hi).collect();
                        ks.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
                        for x in ks {
                            st.made.push(s.isocurve(dir, (x - lo) / (hi - lo)));
                        }
                    }
                }
                other => return ask(&st, Vec::new()).refuse(format!("Unknown option {other}")),
            }
            ask(&st, Vec::new())
        }
        Input::Undo => {
            st.made.pop();
            ask(&st, Vec::new())
        }
        Input::Enter => {
            let n = st.made.len();
            if n == 0 {
                return Reply::finish(Vec::new()).say("Extract Isocurve: nothing made");
            }
            Reply::finish(st.made).say(format!("Extract Isocurve: {n} curves"))
        }
        _ => ask(&st, Vec::new()),
    }
}

// ---- soft edit -------------------------------------------------------------------------------

/// Greville abscissae of a NURBS (normalised to [0, 1]).
fn greville(degree: usize, knots: &[f64], n: usize) -> Vec<f64> {
    let (a, b) = (knots[degree], knots[n]);
    (0..n)
        .map(|i| {
            let s: f64 = knots[i + 1..=i + degree].iter().sum::<f64>() / degree.max(1) as f64;
            ((s - a) / (b - a)).clamp(0.0, 1.0)
        })
        .collect()
}

pub(crate) fn soft_edit(e: &Event) -> Reply {
    let flow = Flow::new("SoftEditCrv")
        .curve("Select curve to edit")
        .point("Point on curve to edit")
        .point("Point to move to")
        .num("Distance", 1.0)
        .toggle("Copy", false)
        .toggle("FixEnds", false);
    let make = |c: &Collected, to: Option<Vec3>| -> Result<(String, Geo), String> {
        let (cv, _) = c.curve(0).ok_or("Select curve to edit")?;
        let (from, to) = (
            c.p(0).ok_or("Point on curve to edit")?,
            c.p(1).or(to).ok_or("Point to move to")?,
        );
        let falloff = c.num(&flow, "Distance");
        if falloff <= 0.0 {
            return Err("Distance must be more than 0".into());
        }
        // Work on control points: the curve as a NURBS (refitted when it is not one).
        let g = match cv.geo() {
            g @ Geo::Nurbs { degree, .. } if degree >= 2 => g,
            _ => {
                let pts: Vec<Vec3> = (0..=200).map(|i| cv.at(i as f64 / 200.0)).collect();
                let pts = if cv.closed { pts[..200].to_vec() } else { pts };
                interp::fit(&pts, 24, 3, cv.closed).ok_or("Could not edit this curve")?
            }
        };
        let Geo::Nurbs {
            degree,
            mut points,
            weights,
            knots,
        } = g
        else {
            return Err("Could not edit this curve".into());
        };
        // Arc length positions of the poles and of the picked point.
        let length = |t: f64| -> f64 {
            let m = 200;
            (0..m)
                .map(|i| {
                    dist(
                        cv.at(t * i as f64 / m as f64),
                        cv.at(t * (i + 1) as f64 / m as f64),
                    )
                })
                .sum()
        };
        let total = length(1.0);
        let (t0, q0) = cv.closest(from);
        let s0 = length(t0);
        let move_by = sub(to, q0);
        let n = points.len();
        let w: Vec<f64> = greville(degree, &knots, n)
            .into_iter()
            .enumerate()
            .map(|(i, gi)| {
                if c.yes(&flow, "FixEnds") && !cv.closed && (i == 0 || i == n - 1) {
                    return 0.0;
                }
                let mut ds = (gi * total - s0).abs();
                if cv.closed {
                    ds = ds.min(total - ds);
                }
                if ds < falloff {
                    0.5 + 0.5 * (std::f64::consts::PI * ds / falloff).cos()
                } else {
                    0.0
                }
            })
            .collect();
        // The picked point lands on the target: scale by how much of a move reaches it.
        let base = Geo::Nurbs {
            degree,
            points: points.clone(),
            weights: weights.clone(),
            knots: knots.clone(),
        };
        let unit_moved: Vec<Vec3> = points
            .iter()
            .zip(&w)
            .map(|(p, wi)| add(*p, scale(move_by, *wi)))
            .collect();
        let trial = Geo::Nurbs {
            degree,
            points: unit_moved,
            weights: weights.clone(),
            knots: knots.clone(),
        };
        let probe = CurveRef {
            exact: Some(base.clone()),
            ..cv.clone()
        }
        .closest(q0)
        .0;
        let reach = match (
            om9_addon_geom::eval_geo(&trial, probe),
            om9_addon_geom::eval_geo(&base, probe),
        ) {
            (Some(a), Some(b)) if dot(move_by, move_by) > 1e-24 => {
                dot(sub(a, b), move_by) / dot(move_by, move_by)
            }
            _ => 1.0,
        };
        let k = if reach > 1e-6 { 1.0 / reach } else { 1.0 };
        for (p, wi) in points.iter_mut().zip(&w) {
            *p = add(*p, scale(move_by, wi * k));
        }
        Ok((
            cv.id.clone(),
            Geo::Nurbs {
                degree,
                points,
                weights,
                knots,
            },
        ))
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).ok().map(|m| m.1).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            let (id, g) = make(c, None)?;
            if c.yes(&flow, "Copy") {
                return Ok(Reply::finish(vec![g]).say("SoftEditCrv: copy made"));
            }
            let mut r = Reply::finish(Vec::new()).say("SoftEditCrv: done");
            r.replace.push(Replacement { id, geo: g });
            Ok(r)
        },
    )
}
