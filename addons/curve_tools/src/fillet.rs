// SPDX-License-Identifier: LGPL-2.1-or-later
//! Fillet, Chamfer and Fillet Corners as Matrix (Rhino) runs them:
//!
//! - `Select first curve to fillet ( Radius=1 Join=No Trim=Yes ExtendArcsBy=Arc )`, then the
//!   second curve. The picks choose the corner: each curve keeps the side that was clicked,
//!   the fillet joins those sides. Curves that do not reach the corner are extended (lines as
//!   lines, arcs by `ExtendArcsBy`), `Radius=0` trims or extends them to a sharp corner.
//! - `Select first curve to chamfer ( Distances=1,1 Join=No Trim=Yes ExtendArcsBy=Arc )`: the
//!   distances are measured from the corner along each curve.
//! - `Select curves to fillet corners`, `Radius <1>`: every corner of polylines rounded.
use crate::{fmt_len, round_corners};
use om9_addon_abi::session::{Event, Geo, Replacement, Reply, Vec3, dist};
use om9_addon_geom::cmd::{Collected, Flow, Show};
use om9_addon_geom::join::{self, Extension};
use om9_addon_geom::blend::{arc_between, plane_normal, tangent_circle as solve};
use om9_addon_geom::pick::CurveRef;

/// Curves to replace (`Some`) or delete (`None`), by id.
type Changes = Vec<(String, Option<Geo>)>;

/// A curve extended at its open ends, with the parameters of the original ends on it.
struct Extended {
    curve: CurveRef,
    /// Parameters (on the extended curve) of the original start and end.
    ends: (f64, f64),
}

fn extended(c: &CurveRef, len: f64, arcs_by_line: bool) -> Extended {
    if c.closed {
        return Extended {
            curve: c.clone(),
            ends: (0.0, 1.0),
        };
    }
    let g = c.geo();
    let how = match (&g, arcs_by_line) {
        (Geo::Arc { .. }, false) => Extension::Natural,
        (Geo::Line { .. } | Geo::Polyline { .. }, _) => Extension::Natural,
        _ => Extension::Line,
    };
    let ext = join::extend_geo(&g, true, len, how)
        .and_then(|x| join::extend_geo(&x, false, len, how))
        .unwrap_or(g);
    let curve = CurveRef {
        exact: Some(ext),
        ..c.clone()
    };
    let ends = (curve.closest(c.at(0.0)).0, curve.closest(c.at(1.0)).0);
    Extended { curve, ends }
}

/// The kept part of a curve: from the touch parameter `s` towards the pick, up to the
/// original end on that side. `None` when nothing is left.
fn kept(e: &Extended, s: f64, pick: f64) -> Option<Geo> {
    if e.curve.closed {
        return e.curve.exact.clone();
    }
    let far = if pick >= s { e.ends.1 } else { e.ends.0 };
    if (far - s).abs() < 1e-9 {
        return None;
    }
    e.curve.sub(s.min(far), s.max(far))
}

/// The point at chord distance `d` from `x` on the curve, on the pick side of `s`.
fn at_distance(c: &CurveRef, s: f64, pick: f64, x: Vec3, d: f64) -> Option<f64> {
    if d <= 0.0 {
        return Some(s);
    }
    let dir = if pick >= s { 1.0 } else { -1.0 };
    let far = if dir > 0.0 { 1.0 } else { 0.0 };
    let f = |t: f64| dist(c.at(t), x) - d;
    if f(far) < 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (s, far);
    for _ in 0..80 {
        let mid = (lo + hi) / 2.0;
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Where each pick lies: the clicked parameter, or for preselected curves the middle of the
/// curve away from the ends nearest each other.
fn picks(c: &Collected) -> Option<((CurveRef, f64), (CurveRef, f64))> {
    let (a, ta) = c.curve(0)?;
    let (b, tb) = c.curve(1)?;
    let mid = |cv: &CurveRef, other: &CurveRef| -> f64 {
        let near_start = dist(cv.at(0.0), other.closest(cv.at(0.0)).1);
        let near_end = dist(cv.at(1.0), other.closest(cv.at(1.0)).1);
        if near_start <= near_end { 0.75 } else { 0.25 }
    };
    let ta = ta.unwrap_or_else(|| mid(a, b));
    let tb = tb.unwrap_or_else(|| mid(b, a));
    Some(((a.clone(), ta), (b.clone(), tb)))
}

pub(crate) fn fillet(e: &Event, chamfer: bool) -> Reply {
    let label = if chamfer { "Chamfer" } else { "Fillet" };
    let (first, second) = if chamfer {
        (
            "Select first curve to chamfer",
            "Select second curve to chamfer",
        )
    } else {
        (
            "Select first curve to fillet",
            "Select second curve to fillet",
        )
    };
    let mut flow = Flow::new(label).curve(first).curve(second);
    flow = if chamfer {
        flow.pair("Distances", 1.0, 1.0)
    } else {
        flow.num("Radius", 1.0)
    };
    let flow = flow.toggle("Join", false).toggle("Trim", true).choice(
        "ExtendArcsBy",
        &["Arc", "Line"],
        "Arc",
    );
    let make = |c: &Collected| -> Result<(Vec<Geo>, Changes), String> {
        let ((a, pa), (b, pb)) = picks(c).ok_or("pick two curves")?;
        let n = plane_normal(&a, pa, &b, pb, e.plane.normal());
        let span =
            dist(a.at(0.0), a.at(1.0)) + dist(b.at(0.0), b.at(1.0)) + dist(a.at(pa), b.at(pb));
        let (d1, d2) = c.pair(&flow, "Distances");
        let r = if chamfer {
            0.0
        } else {
            c.num(&flow, "Radius").max(0.0)
        };
        let reach = 4.0 * (span + r + d1.abs() + d2.abs()) + 10.0;
        let by_line = c.opt(&flow, "ExtendArcsBy") == "Line";
        let (xa, xb) = (extended(&a, reach, by_line), extended(&b, reach, by_line));
        let (ea, eb) = (xa.curve.closest(a.at(pa)).0, xb.curve.closest(b.at(pb)).0);
        let (sa, sb, center) = solve(&xa.curve, ea, &xb.curve, eb, n, r).ok_or_else(|| {
            if r > 0.0 {
                format!("Radius {} is too large for these curves", fmt_len(r))
            } else {
                "The curves do not meet".to_owned()
            }
        })?;
        // The touch points, and the piece between them.
        let (ta, tb, piece) = if chamfer {
            let ta = at_distance(&xa.curve, sa, ea, center, d1)
                .ok_or("Distances too large for the first curve")?;
            let tb = at_distance(&xb.curve, sb, eb, center, d2)
                .ok_or("Distances too large for the second curve")?;
            let (p, q) = (xa.curve.at(ta), xb.curve.at(tb));
            (
                ta,
                tb,
                (dist(p, q) > 1e-9).then_some(Geo::Line { start: p, end: q }),
            )
        } else {
            let (p, q) = (xa.curve.at(sa), xb.curve.at(sb));
            (
                sa,
                sb,
                if r > 0.0 {
                    arc_between(center, p, q)
                } else {
                    None
                },
            )
        };
        let trim = c.yes(&flow, "Trim");
        let mut changes: Changes = Vec::new();
        let mut parts: Vec<Geo> = Vec::new();
        let ka = kept(&xa, ta, ea);
        let kb = kept(&xb, tb, eb);
        if c.yes(&flow, "Join") {
            let mut pieces: Vec<Geo> = ka.iter().cloned().collect();
            pieces.extend(piece.clone());
            pieces.extend(kb.iter().cloned());
            let joined = join::join(&pieces, 1e-6).ok_or("The pieces do not join")?;
            changes.push((a.id.clone(), Some(joined)));
            changes.push((b.id.clone(), None));
            return Ok((parts, changes));
        }
        if trim {
            changes.push((a.id.clone(), ka));
            changes.push((b.id.clone(), kb));
        }
        parts.extend(piece);
        Ok((parts, changes))
    };
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (create, changes) = make(c)?;
        let mut r = Reply::finish(create).say(format!("{label}: done"));
        for (id, g) in changes {
            match g {
                Some(geo) => r.replace.push(Replacement { id, geo }),
                None => r.delete.push(id),
            }
        }
        Ok(r)
    })
}

pub(crate) fn fillet_corners(e: &Event) -> Reply {
    let flow = Flow::new("Fillet Corners")
        .select_prompt("Select curves to fillet corners")
        .number("Radius", 1.0);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let r = c.n(0).unwrap_or(1.0);
        if r <= 0.0 {
            return Err("Radius must be more than 0".into());
        }
        let items: Vec<(String, Geo)> = CurveRef::all(e)
            .iter()
            .filter_map(|cv| {
                let pts = match &cv.exact {
                    Some(Geo::Polyline { points, .. }) => points.clone(),
                    Some(Geo::Nurbs {
                        degree: 1, points, ..
                    }) => {
                        let mut p = points.clone();
                        if cv.closed && p.len() > 2 && dist(p[0], *p.last()?) < 1e-9 {
                            p.pop();
                        }
                        p
                    }
                    _ => return None,
                };
                round_corners(&pts, cv.closed, r).map(|g| (cv.id.clone(), g))
            })
            .collect();
        if items.is_empty() {
            return Err("Fillet Corners: select polylines".into());
        }
        let n = items.len();
        let mut reply = Reply::finish(Vec::new()).say(format!("Fillet Corners: {n} curves"));
        reply.replace = items
            .into_iter()
            .map(|(id, geo)| Replacement { id, geo })
            .collect();
        Ok(reply)
    })
}
