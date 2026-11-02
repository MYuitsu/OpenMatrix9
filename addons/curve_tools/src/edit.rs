// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rebuild, FitCrv, Divide, Convert and TweenCurves as Matrix (Rhino) runs them.
//!
//! - Rebuild: `Select curves to rebuild`, then the Rebuild Curve choices on the command line
//!   (`PointCount=10 Degree=3 DeleteInput=Yes SelectMasterCurve`, as Rhino 5's -Rebuild), Enter. Uniform knots,
//!   least-squares through the old shape, ends kept; the maximum deviation is reported.
//! - FitCrv: `Select curves to fit`, `DeleteInput=Yes Degree=3 FittingTolerance=0.001`, Enter:
//!   the fewest control points within the tolerance.
//! - Divide: `Select curves to divide`, `Number of segments <10> ( Length Split=No
//!   MarkEnds=Yes )`; `Length` asks `Segment length` instead.
//! - Convert (Matrix "Curve to Lines": `_Convert _Output=_Lines _DeleteInput=_Yes`):
//!   `Select curves to convert ( Output=Lines DeleteInput=Yes Angle=5 Tolerance=0.01
//!   MinimumLength=0 MaximumLength=0 )`, Enter.
//! - TweenCurves: `Select start curve`, `Select end curve`, then `Number=1
//!   MatchMethod=None|Refit|SamplePoints SampleNumber=10 Flip`, Enter.
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Replacement, Reply, Vec3, Want, add, dist, dot, fmt, scale, sub,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save};
use om9_addon_geom::interp;
use om9_addon_geom::pick::CurveRef;
use serde::{Deserialize, Serialize};

/// Arc-length table of a curve: (parameter, length so far).
fn lengths(c: &CurveRef, n: usize) -> Vec<(f64, f64)> {
    let mut out = vec![(0.0, 0.0)];
    let mut prev = c.at(0.0);
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let p = c.at(t);
        let l = out.last().unwrap().1 + dist(prev, p);
        out.push((t, l));
        prev = p;
    }
    out
}

/// The parameter at arc length `s` (from the table).
fn at_length(table: &[(f64, f64)], s: f64) -> f64 {
    let i = table.partition_point(|x| x.1 < s).clamp(1, table.len() - 1);
    let (a, b) = (table[i - 1], table[i]);
    if b.1 - a.1 < 1e-15 {
        return a.0;
    }
    a.0 + (b.0 - a.0) * (s - a.1) / (b.1 - a.1)
}

/// Samples of a curve evenly by arc length (`n` intervals).
fn even(c: &CurveRef, n: usize) -> Vec<Vec3> {
    let table = lengths(c, 4096);
    let total = table.last().unwrap().1;
    let m = if c.closed { n } else { n + 1 };
    (0..m)
        .map(|i| c.at(at_length(&table, total * i as f64 / n as f64)))
        .collect()
}

/// Largest distance from the original's samples to the candidate.
fn deviation(c: &CurveRef, g: &Geo) -> f64 {
    let fit = CurveRef {
        exact: Some(g.clone()),
        ..c.clone()
    };
    (0..=200)
        .map(|i| {
            let p = c.at(i as f64 / 200.0);
            dist(fit.closest(p).1, p)
        })
        .fold(0.0, f64::max)
}

fn replace_or_add(
    label: &str,
    delete: bool,
    made: Vec<(String, Geo)>,
    note: String,
) -> Result<Reply, String> {
    if made.is_empty() {
        return Err(format!("{label}: select curves"));
    }
    if delete {
        let mut r = Reply::finish(Vec::new()).say(note);
        r.replace = made
            .into_iter()
            .map(|(id, geo)| Replacement { id, geo })
            .collect();
        Ok(r)
    } else {
        Ok(Reply::finish(made.into_iter().map(|m| m.1).collect()).say(note))
    }
}

// ---- Rebuild, FitCrv -------------------------------------------------------------------------

fn rebuilt(c: &CurveRef, count: usize, degree: usize, tangents: bool) -> Option<Geo> {
    let samples = even(c, 400);
    let mut g = interp::fit(&samples, count, degree, c.closed)?;
    if tangents
        && !c.closed
        && degree >= 2
        && count >= 4
        && let Geo::Nurbs { points, .. } = &mut g
    {
        {
            let n = points.len();
            let (t0, t1) = (c.tangent(0.0), c.tangent(1.0));
            let l0 = dot(sub(points[1], points[0]), t0).max(1e-9);
            points[1] = add(points[0], scale(t0, l0));
            let l1 = dot(sub(points[n - 1], points[n - 2]), t1).max(1e-9);
            points[n - 2] = sub(points[n - 1], scale(t1, l1));
        }
    }
    Some(g)
}

/// Rebuild's own state around the Flow: Rhino's `SelectMasterCurve` picks a curve whose
/// point count and degree the rebuilt curves take (the master itself is not rebuilt).
#[derive(Default, Serialize, Deserialize)]
struct RebuildState {
    #[serde(default)]
    flow: serde_json::Value,
    #[serde(default)]
    master: Option<String>,
    #[serde(default)]
    picking: bool,
    #[serde(default)]
    before: Vec<String>,
}

pub(crate) fn rebuild(e: &Event) -> Reply {
    let mut w: RebuildState = load(e);
    let mut inner = e.clone();
    inner.state = w.flow.clone();
    if let Input::Option { name, .. } = &e.input
        && name.eq_ignore_ascii_case("SelectMasterCurve")
    {
        w.picking = true;
        w.before = e.selection.iter().map(|s| s.id.clone()).collect();
        return Reply::ask("Select master curve", save(&w)).want(om9_addon_abi::session::Want::Objects);
    }
    if w.picking {
        let master = match e.pick {
            Some(_) => CurveRef::picked(e, &[]).map(|c| c.0),
            None => CurveRef::all(e).into_iter().find(|c| !w.before.contains(&c.id)),
        };
        let Some(master) = master else {
            return Reply::ask("Select master curve", save(&w)).want(om9_addon_abi::session::Want::Objects);
        };
        let (count, degree) = match master.geo() {
            Geo::Nurbs { degree, points, .. } => (points.len(), degree),
            Geo::Polyline { points, closed } => (points.len() + usize::from(closed), 1),
            Geo::Line { .. } => (2, 1),
            _ => {
                return Reply::ask("Select master curve", save(&w))
                    .want(om9_addon_abi::session::Want::Objects)
                    .refuse("The master curve must be a NURBS curve or a polyline");
            }
        };
        w.picking = false;
        w.master = Some(master.id.clone());
        inner.selection.retain(|s| s.id != master.id);
        let mut r = Reply::default();
        for (name, value) in [("PointCount", count as f64), ("Degree", degree as f64)] {
            inner.input = Input::Option { name: name.into(), value: Some(fmt(value)) };
            r = rebuild_flow(&inner);
            inner.state = r.state.clone();
        }
        w.flow = r.state.clone();
        r.state = save(&w);
        return r.say(format!("Rebuild: {count} points, degree {degree} from the master curve"));
    }
    if let Some(m) = &w.master {
        inner.selection.retain(|s| &s.id != m);
    }
    let mut r = rebuild_flow(&inner);
    w.flow = r.state.clone();
    r.state = save(&w);
    if !r.done {
        r.options.push(om9_addon_abi::session::OptionToken::action("SelectMasterCurve"));
    }
    r
}

fn rebuild_flow(e: &Event) -> Reply {
    let flow = Flow::new("Rebuild")
        .select_prompt("Select curves to rebuild")
        .confirm("Rebuild. Press Enter when done")
        .num("PointCount", 10.0)
        .num("Degree", 3.0)
        .toggle("DeleteInput", true);
    let make = |c: &Collected| -> Result<(Vec<(String, Geo)>, f64), String> {
        let count = c.num(&flow, "PointCount").round() as usize;
        let degree = c.num(&flow, "Degree").round() as usize;
        if !(1..=11).contains(&degree) {
            return Err("Degree is a whole number from 1 to 11".into());
        }
        if count < degree + 1 {
            return Err(format!(
                "PointCount must be at least Degree + 1 ({})",
                degree + 1
            ));
        }
        let mut worst: f64 = 0.0;
        let mut out = Vec::new();
        for cv in CurveRef::all(e) {
            let g = rebuilt(&cv, count, degree, false)
                .ok_or("Rebuild failed")?;
            worst = worst.max(deviation(&cv, &g));
            out.push((cv.id.clone(), g));
        }
        Ok((out, worst))
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c)
                .map(|m| m.0.into_iter().map(|x| x.1).collect())
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let (made, worst) = make(c)?;
            let note = format!("Rebuild: maximum deviation {}", crate::fmt_len(worst));
            replace_or_add("Rebuild", c.yes(&flow, "DeleteInput"), made, note)
        },
    )
}

pub(crate) fn fit_curve(e: &Event) -> Reply {
    let flow = Flow::new("FitCrv")
        .select_prompt("Select curves to fit")
        .confirm("Fit. Press Enter when done")
        .toggle("DeleteInput", true)
        .num("Degree", 3.0)
        .num("FittingTolerance", 0.001);
    let make = |c: &Collected| -> Result<Vec<(String, Geo)>, String> {
        let degree = c.num(&flow, "Degree").round() as usize;
        if !(1..=11).contains(&degree) {
            return Err("Degree is a whole number from 1 to 11".into());
        }
        let tol = c.num(&flow, "FittingTolerance");
        if tol <= 0.0 {
            return Err("FittingTolerance must be more than 0".into());
        }
        let mut out = Vec::new();
        for cv in CurveRef::all(e) {
            let samples = even(&cv, 600);
            let mut count = degree + 1 + usize::from(cv.closed) * degree;
            let mut best = None;
            while count <= 600 {
                if let Some(g) = interp::fit(&samples, count, degree, cv.closed) {
                    let ok = deviation(&cv, &g) <= tol;
                    best = Some(g);
                    if ok {
                        break;
                    }
                }
                count += (count / 4).max(1);
            }
            out.push((cv.id.clone(), best.ok_or("FitCrv failed")?));
        }
        Ok(out)
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c)
                .map(|m| m.into_iter().map(|x| x.1).collect())
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let made = make(c)?;
            let n = made.len();
            replace_or_add(
                "FitCrv",
                c.yes(&flow, "DeleteInput"),
                made,
                format!("FitCrv: {n} curves"),
            )
        },
    )
}

// ---- Divide ----------------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct DivideState {
    /// 0: selecting, 1: number of segments, 2: segment length.
    stage: u8,
    #[serde(default)]
    split: bool,
    #[serde(default = "yes")]
    mark_ends: bool,
    #[serde(default)]
    count: Option<f64>,
    #[serde(default)]
    length: Option<f64>,
}

fn yes() -> bool {
    true
}

/// The division parameters of a curve: by count, or by length from the start.
fn division(c: &CurveRef, count: Option<usize>, length: Option<f64>) -> Vec<f64> {
    let table = lengths(c, 4096);
    let total = table.last().unwrap().1;
    let at = |s: f64| at_length(&table, s);
    match (count, length) {
        (Some(n), _) => {
            let last = if c.closed { n - 1 } else { n };
            (0..=last)
                .map(|k| at(total * k as f64 / n as f64))
                .collect()
        }
        (None, Some(l)) => {
            let n = (total / l + 1e-9).floor() as usize;
            let mut out: Vec<f64> = (0..=n).map(|k| at(l * k as f64)).collect();
            if !c.closed && (total - l * n as f64) > 1e-9 {
                out.push(1.0);
            }
            out
        }
        _ => Vec::new(),
    }
}

pub(crate) fn divide(e: &Event) -> Reply {
    let mut st: DivideState = load(e);
    if st.stage == 0 && !e.selection.is_empty() && matches!(e.input, Input::Start | Input::Enter) {
        st.stage = 1;
        st.mark_ends = true;
    }
    let ask = |st: &DivideState| -> Reply {
        let mut options = Vec::new();
        let (prompt, want) = match st.stage {
            0 => (
                "Select curves to divide. Press Enter when done".to_owned(),
                Want::Objects,
            ),
            1 => {
                options.push(OptionToken::action("Length"));
                (
                    format!("Number of segments <{}>", fmt(st.count.unwrap_or(10.0))),
                    Want::Number,
                )
            }
            _ => (
                format!("Segment length <{}>", fmt(st.length.unwrap_or(1.0))),
                Want::Number,
            ),
        };
        if st.stage > 0 {
            options.push(OptionToken::toggle("Split", st.split));
            options.push(OptionToken::toggle("MarkEnds", st.mark_ends));
        }
        Reply::ask(prompt, save(st))
            .with_options(options)
            .want(want)
    };
    let finish = |st: &DivideState, count: Option<usize>, length: Option<f64>| -> Reply {
        let cs = CurveRef::all(e);
        if cs.is_empty() {
            return ask(st).refuse("Divide: select curves");
        }
        let mut create = Vec::new();
        let mut r = Reply::finish(Vec::new());
        for c in &cs {
            let ts = division(c, count, length);
            if st.split {
                let mut cuts = ts.clone();
                if c.closed {
                    cuts.push(1.0 + ts[0]);
                }
                for w in cuts.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    let piece = if b > 1.0 {
                        c.sub(a, b - 1.0)
                    } else {
                        c.sub(a, b)
                    };
                    create.extend(piece);
                }
                r.delete.push(c.id.clone());
            } else {
                for (k, &t) in ts.iter().enumerate() {
                    let end = !c.closed && (k == 0 || k == ts.len() - 1);
                    if end && !st.mark_ends {
                        continue;
                    }
                    create.push(Geo::Point { point: c.at(t) });
                }
            }
        }
        let n = create.len();
        r.create = create;
        r.say(if st.split {
            format!("Divide: {n} curves")
        } else {
            format!("Divide: {n} points")
        })
    };
    match &e.input {
        Input::Option { name, value } => {
            let flag = |cur: bool| match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("yes"),
                None => !cur,
            };
            match name.to_ascii_lowercase().as_str() {
                "length" if st.stage > 0 => match value
                    .as_deref()
                    .and_then(om9_addon_abi::session::parse_number)
                {
                    Some(l) if l > 0.0 => return finish(&st, None, Some(l)),
                    _ => st.stage = 2,
                },
                "split" => st.split = flag(st.split),
                "markends" => st.mark_ends = flag(st.mark_ends),
                _ => return ask(&st).refuse(format!("Unknown option {name}")),
            }
            ask(&st)
        }
        Input::Text { text } if st.stage > 0 => match om9_addon_abi::session::parse_number(text) {
            Some(v) if st.stage == 1 && v >= 1.0 => {
                st.count = Some(v);
                finish(&st, Some(v.round() as usize), None)
            }
            Some(v) if st.stage == 2 && v > 0.0 => {
                st.length = Some(v);
                finish(&st, None, Some(v))
            }
            _ => ask(&st).refuse(format!("Not a number: {text}")),
        },
        Input::Enter if st.stage == 1 => {
            finish(&st, Some(st.count.unwrap_or(10.0).round() as usize), None)
        }
        Input::Enter if st.stage == 2 => finish(&st, None, Some(st.length.unwrap_or(1.0))),
        _ => ask(&st),
    }
}

// ---- Convert ---------------------------------------------------------------------------------

/// Split `[a, b]` until each chord (Lines) or arc (Arcs) is within tolerance and angle.
#[allow(clippy::too_many_arguments)]
fn convert_pieces(
    c: &CurveRef,
    a: f64,
    b: f64,
    arcs: bool,
    tol: f64,
    angle: f64,
    max_len: f64,
    depth: u32,
    out: &mut Vec<Geo>,
) {
    let (p, q) = (c.at(a), c.at(b));
    let m = (a + b) / 2.0;
    let mid = c.at(m);
    let piece = if arcs {
        om9_addon_geom::arc_3_points(p, mid, q).or(Some(Geo::Line { start: p, end: q }))
    } else {
        Some(Geo::Line { start: p, end: q })
    };
    let ok = depth >= 16
        || piece.as_ref().is_some_and(|g| {
            let fit = CurveRef {
                exact: Some(g.clone()),
                closed: false,
                ..c.clone()
            };
            let dev = (1..8)
                .map(|k| {
                    let x = c.at(a + (b - a) * k as f64 / 8.0);
                    dist(fit.closest(x).1, x)
                })
                .fold(0.0, f64::max);
            let turn = if arcs {
                0.0
            } else {
                dot(c.tangent(a), c.tangent(b)).clamp(-1.0, 1.0).acos()
            };
            dev <= tol
                && (angle <= 0.0 || turn <= angle)
                && (max_len <= 0.0 || dist(p, q) <= max_len)
        });
    if ok {
        out.extend(piece);
    } else {
        convert_pieces(c, a, m, arcs, tol, angle, max_len, depth + 1, out);
        convert_pieces(c, m, b, arcs, tol, angle, max_len, depth + 1, out);
    }
}

pub(crate) fn convert(e: &Event) -> Reply {
    let flow = Flow::new("Convert")
        .select_prompt("Select curves to convert")
        .choice("Output", &["Lines", "Arcs"], "Lines")
        .toggle("DeleteInput", true)
        .num("Angle", 5.0)
        .num("Tolerance", 0.01)
        .num("MinimumLength", 0.0)
        .num("MaximumLength", 0.0);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let arcs = c.opt(&flow, "Output") == "Arcs";
        let tol = c.num(&flow, "Tolerance").max(1e-6);
        let angle = c.num(&flow, "Angle").max(0.0).to_radians();
        let (min_len, max_len) = (
            c.num(&flow, "MinimumLength").max(0.0),
            c.num(&flow, "MaximumLength").max(0.0),
        );
        let mut made = Vec::new();
        for cv in CurveRef::all(e) {
            // Lines and polylines are already lines.
            if !arcs
                && matches!(
                    cv.exact,
                    Some(Geo::Line { .. } | Geo::Polyline { .. })
                        | Some(Geo::Nurbs { degree: 1, .. })
                )
            {
                continue;
            }
            let mut pieces = Vec::new();
            let start = if cv.closed { 4 } else { 1 };
            for k in 0..start {
                convert_pieces(
                    &cv,
                    k as f64 / start as f64,
                    (k + 1) as f64 / start as f64,
                    arcs,
                    tol,
                    angle,
                    max_len,
                    0,
                    &mut pieces,
                );
            }
            // MinimumLength: merge short lines into their neighbours.
            if !arcs && min_len > 0.0 {
                let mut merged: Vec<Vec3> = Vec::new();
                for g in &pieces {
                    if let Geo::Line { start, end } = g {
                        if merged.is_empty() {
                            merged.push(*start);
                        }
                        if dist(*merged.last().unwrap(), *end) >= min_len || merged.len() == 1 {
                            merged.push(*end);
                        } else {
                            *merged.last_mut().unwrap() = *end;
                        }
                    }
                }
                let joined = if cv.closed {
                    merged.pop();
                    Geo::Polyline {
                        points: merged,
                        closed: true,
                    }
                } else {
                    Geo::Polyline {
                        points: merged,
                        closed: false,
                    }
                };
                made.push((cv.id.clone(), joined));
                continue;
            }
            let joined = om9_addon_geom::join::join(&pieces, 1e-7)
                .ok_or("Convert: the pieces do not join")?;
            made.push((cv.id.clone(), joined));
        }
        let n = made.len();
        replace_or_add(
            "Convert",
            c.yes(&flow, "DeleteInput"),
            made,
            format!("Convert: {n} curves"),
        )
    })
}

// ---- TweenCurves -----------------------------------------------------------------------------

pub(crate) fn tween(e: &Event) -> Reply {
    let flow = Flow::new("TweenCurves")
        .curve("Select start curve")
        .curve("Select end curve")
        .confirm("Adjust the tween. Press Enter when done")
        .num("Number", 1.0)
        .choice("MatchMethod", &["None", "Refit", "SamplePoints"], "None")
        .num("SampleNumber", 10.0)
        .toggle("Flip", false);
    let make = |c: &Collected| -> Result<Vec<Geo>, String> {
        let (a, _) = c.curve(0).ok_or("Select start curve")?;
        let (b, _) = c.curve(1).ok_or("Select end curve")?;
        let count = c.num(&flow, "Number").round();
        if count < 1.0 {
            return Err("Number must be 1 or more".into());
        }
        let count = count as usize;
        if a.closed != b.closed {
            return Err("Both curves must be open or both closed".into());
        }
        let mut b = b.clone();
        // Run the same way: the start curve's start pairs with the nearer end.
        let reversed = !a.closed && dist(a.at(0.0), b.at(0.0)) > dist(a.at(0.0), b.at(1.0));
        if reversed != c.yes(&flow, "Flip") {
            b.exact = om9_addon_geom::join::reverse(&b.geo());
            b.pts.reverse();
        }
        let tween_at = |t: f64| -> Option<Geo> {
            let direct = match (c.opt(&flow, "MatchMethod"), &a.exact, &b.exact) {
                // Same structure: blend the control points.
                (
                    "None",
                    Some(Geo::Nurbs {
                        degree: da,
                        points: pa,
                        weights: wa,
                        knots: ka,
                    }),
                    Some(Geo::Nurbs {
                        degree: db,
                        points: pb,
                        weights: wb,
                        knots: kb,
                    }),
                ) if da == db
                    && pa.len() == pb.len()
                    && wa.is_empty()
                    && wb.is_empty()
                    && ka.len() == kb.len() =>
                {
                    let span = |k: &[f64]| (k[*da], k[k.len() - da - 1]);
                    let (sa, sb) = (span(ka), span(kb));
                    let norm_k = |k: &[f64], s: (f64, f64)| -> Vec<f64> {
                        k.iter().map(|x| (x - s.0) / (s.1 - s.0)).collect()
                    };
                    let (na, nb) = (norm_k(ka, sa), norm_k(kb, sb));
                    if na.iter().zip(&nb).all(|(x, y)| (x - y).abs() < 1e-9) {
                        let points = pa
                            .iter()
                            .zip(pb)
                            .map(|(p, q)| add(*p, scale(sub(*q, *p), t)))
                            .collect();
                        return Some(Geo::Nurbs {
                            degree: *da,
                            points,
                            weights: Vec::new(),
                            knots: na,
                        });
                    }
                    None
                }
                _ => None,
            };
            direct.or_else(|| {
                let refit = c.opt(&flow, "MatchMethod") == "Refit";
                let n = if refit {
                    64
                } else {
                    (c.num(&flow, "SampleNumber").round() as usize).max(3)
                };
                let (pa, pb) = (even(a, n), even(&b, n));
                let pts: Vec<Vec3> = pa
                    .iter()
                    .zip(&pb)
                    .map(|(p, q)| add(*p, scale(sub(*q, *p), t)))
                    .collect();
                if refit {
                    interp::fit(&pts, 16.min(pts.len() - 1), 3, a.closed)
                } else {
                    om9_addon_geom::interpolate(&pts, 3, a.closed)
                }
            })
        };
        (1..=count)
            .map(|k| {
                tween_at(k as f64 / (count + 1) as f64).ok_or_else(|| "Tween failed".to_owned())
            })
            .collect()
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let geo = make(c)?;
            let n = geo.len();
            Ok(Reply::finish(geo).say(format!("TweenCurves: {n} curves")))
        },
    )
}
