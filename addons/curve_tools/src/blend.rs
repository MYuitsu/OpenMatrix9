// SPDX-License-Identifier: LGPL-2.1-or-later
//! Blends and Match as Matrix (Rhino) runs them. Each curve is picked near the end to use.
//!
//! - Quick Curve Blend (`Blend`): `Select first curve near end ( Continuity=Curvature )`,
//!   then the second; the blend is made at once.
//! - Adjustable Curve Blend (`BlendCrv`): `Select first curve to blend - select near end`,
//!   the second, then the Adjust Curve Blend choices on the command line
//!   (`Continuity1=Curvature Continuity2=Curvature Flip1 Flip2 Bulge1 Bulge2 Join`), Enter.
//! - Arc Blend (`ArcBlend`): two tangent arcs; `AlternateSolution`, `RadiusDifference`, `Join`.
//! - Match Curve (`Match`): `Select open curve to change - select near end`, `Select open
//!   curve to match - select near end`, then `Continuity`, `PreserveOtherEnd`, `Join`, Enter.
use om9_addon_abi::session::{Event, Geo, Replacement, Reply, Vec3, add, dist, dot, scale, sub};
use om9_addon_geom::blend::{self, Continuity, End};
use om9_addon_geom::cmd::{Collected, Flow, Show};
use om9_addon_geom::join;
use om9_addon_geom::pick::CurveRef;

/// Which end of each picked curve: the end nearest the pick (for preselected curves, the
/// ends nearest each other). Returns `(curve, end parameter 0|1)` for both.
fn ends(c: &Collected) -> Option<((CurveRef, f64), (CurveRef, f64))> {
    let (a, ta) = c.curve(0)?;
    let (b, tb) = c.curve(1)?;
    let near = |cv: &CurveRef, t: Option<f64>, other: &CurveRef| -> f64 {
        match t {
            Some(t) if !cv.closed => {
                if t < 0.5 {
                    0.0
                } else {
                    1.0
                }
            }
            Some(t) => t,
            None => {
                let d0 = dist(cv.at(0.0), other.closest(cv.at(0.0)).1);
                let d1 = dist(cv.at(1.0), other.closest(cv.at(1.0)).1);
                if d0 <= d1 { 0.0 } else { 1.0 }
            }
        }
    };
    let ea = near(a, ta, b);
    let eb = near(b, tb, a);
    Some(((a.clone(), ea), (b.clone(), eb)))
}

fn finish(
    label: &str,
    made: Vec<Geo>,
    join_into: Option<(&CurveRef, &CurveRef)>,
) -> Result<Reply, String> {
    if let Some((a, b)) = join_into {
        let mut pieces = vec![a.geo()];
        pieces.extend(made);
        pieces.push(b.geo());
        let joined = join::join(&pieces, 1e-6).ok_or("The curves do not join")?;
        let mut r = Reply::finish(Vec::new()).say(format!("{label}: joined"));
        r.replace.push(Replacement {
            id: a.id.clone(),
            geo: joined,
        });
        if b.id != a.id {
            r.delete.push(b.id.clone());
        }
        return Ok(r);
    }
    Ok(Reply::finish(made).say(format!("{label}: done")))
}

/// Quick Curve Blend and Adjustable Curve Blend.
pub(crate) fn blend(e: &Event, adjustable: bool) -> Reply {
    let label = if adjustable {
        "Adjustable Curve Blend"
    } else {
        "Quick Curve Blend"
    };
    let mut flow = if adjustable {
        Flow::new(label)
            .curve("Select first curve to blend - select near end")
            .curve("Select second curve to blend - select near end")
            .confirm("Adjust the blend. Press Enter when done")
            .choice("Continuity1", &Continuity::NAMES, "Curvature")
            .choice("Continuity2", &Continuity::NAMES, "Curvature")
            .toggle("Flip1", false)
            .toggle("Flip2", false)
            .num("Bulge1", 1.0)
            .num("Bulge2", 1.0)
            .toggle("Join", false)
    } else {
        Flow::new(label)
            .curve("Select first curve near end")
            .curve("Select second curve near end")
            .choice("Continuity1", &["Position", "Tangency", "Curvature"], "Curvature")
            .choice("Continuity2", &["Position", "Tangency", "Curvature"], "Curvature")
    };
    flow.label = label.into();
    let make = |c: &Collected| -> Result<Geo, String> {
        let ((a, ta), (b, tb)) = ends(c).ok_or("pick two curves")?;
        let (ca, cb, flips, bulge) = if adjustable {
            (
                Continuity::parse(c.opt(&flow, "Continuity1")),
                Continuity::parse(c.opt(&flow, "Continuity2")),
                [c.yes(&flow, "Flip1"), c.yes(&flow, "Flip2")],
                [c.num(&flow, "Bulge1"), c.num(&flow, "Bulge2")],
            )
        } else {
            (
                Continuity::parse(c.opt(&flow, "Continuity1")),
                Continuity::parse(c.opt(&flow, "Continuity2")),
                [false, false],
                [1.0, 1.0],
            )
        };
        if bulge.iter().any(|b| *b <= 0.0) {
            return Err("Bulge must be more than 0".into());
        }
        let ea = blend::end_of(&a, ta, !flips[0]);
        let eb = blend::end_of(&b, tb, !flips[1]);
        if dist(ea.point, eb.point) < 1e-9 {
            return Err("The ends already meet".into());
        }
        Ok(blend::bezier_blend(&ea, ca, &eb, cb, bulge))
    };
    let joining = |c: &Collected| adjustable && c.yes(&flow, "Join");
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            let g = make(c)?;
            let pair = ends(c);
            match (&pair, joining(c)) {
                (Some(((a, _), (b, _))), true) => finish(label, vec![g], Some((a, b))),
                _ => finish(label, vec![g], None),
            }
        },
    )
}

/// Arc Blend: two tangent arcs between the picked ends.
pub(crate) fn arc_blend(e: &Event) -> Reply {
    let flow = Flow::new("Arc Blend")
        .curve("Select first curve near end")
        .curve("Select second curve near end")
        .confirm("Select arc blend point to adjust. Press Enter when done")
        .toggle("AlternateSolution", false)
        .num("RadiusDifference", 0.0)
        .toggle("Join", false);
    let make = |c: &Collected| -> Result<Vec<Geo>, String> {
        let ((a, ta), (b, tb)) = ends(c).ok_or("pick two curves")?;
        let ea: End = blend::end_of(&a, ta, true);
        let eb: End = blend::end_of(&b, tb, true);
        // Travel: leave a along its outward tangent, arrive at b against its own.
        let arrive = scale(eb.d[0], -1.0);
        let arcs = blend::biarc_with_difference(
            ea.point,
            ea.d[0],
            eb.point,
            arrive,
            c.num(&flow, "RadiusDifference"),
            c.yes(&flow, "AlternateSolution"),
        )
        .ok_or("No arc blend for these ends")?;
        Ok(arcs.to_vec())
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let arcs = make(c)?;
            let pair = ends(c);
            match (&pair, c.yes(&flow, "Join")) {
                (Some(((a, _), (b, _))), true) => finish("Arc Blend", arcs, Some((a, b))),
                _ => finish("Arc Blend", arcs, None),
            }
        },
    )
}

/// Match Curve: change the end of the first curve to meet the second with the chosen
/// continuity, keeping the other end as asked.
pub(crate) fn match_curve(e: &Event) -> Reply {
    let flow = Flow::new("Match Curve")
        .curve("Select open curve to change - select near end")
        .curve("Select open curve to match - select near end")
        .confirm("Match. Press Enter when done")
        .choice(
            "Continuity",
            &["Position", "Tangency", "Curvature"],
            "Curvature",
        )
        .choice(
            "PreserveOtherEnd",
            &["None", "Position", "Tangency", "Curvature"],
            "Position",
        )
        .toggle("AverageCurves", false)
        .toggle("Join", false)
        .toggle("Merge", false);
    // Rhino 5 AverageCurves: both curves move to meet halfway; the second keeps its
    // own direction there and the first matches it.
    let averaged = |c: &Collected| -> Option<(CurveRef, f64)> {
        let ((a, ta), (b, tb)) = ends(c)?;
        if !c.yes(&flow, "AverageCurves") {
            return Some((b, tb));
        }
        let ea = blend::end_of(&a, ta, true);
        let eb = blend::end_of(&b, tb, true);
        let mid = scale(add(ea.point, eb.point), 0.5);
        let goal = End {
            point: mid,
            d: [scale(eb.d[0], -1.0), eb.d[1], eb.d[2], eb.d[3]],
        };
        let moved = matched(&b, tb >= 0.5, &goal, Continuity::Position, Some(Continuity::Position))?;
        let t = if tb >= 0.5 { 1.0 } else { 0.0 };
        Some((CurveRef { exact: Some(moved), ..b.clone() }, t))
    };
    let make = |c: &Collected| -> Result<Geo, String> {
        let ((a, ta), _) = ends(c).ok_or("pick two curves")?;
        let (b, tb) = averaged(c).ok_or("Could not average these curves")?;
        if a.closed {
            return Err("Select an open curve to change".into());
        }
        let target = blend::end_of(&b, tb, true);
        let cont = Continuity::parse(c.opt(&flow, "Continuity"));
        let keep = match c.opt(&flow, "PreserveOtherEnd") {
            "None" => None,
            other => Some(Continuity::parse(other)),
        };
        matched(&a, ta >= 0.5, &target, cont, keep)
            .ok_or_else(|| "Could not match these curves".to_owned())
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            let g = make(c)?;
            let ((a, _), _) = ends(c).ok_or("pick two curves")?;
            let (b, _) = averaged(c).ok_or("Could not average these curves")?;
            // Merge (Curvature only in Rhino 5) makes one curve, like Join.
            let merge = c.yes(&flow, "Merge") && c.opt(&flow, "Continuity") == "Curvature";
            if c.yes(&flow, "AverageCurves") && !(c.yes(&flow, "Join") || merge) {
                let mut r = Reply::finish(Vec::new()).say("Match Curve: averaged");
                r.replace.push(Replacement { id: a.id.clone(), geo: g });
                r.replace.push(Replacement { id: b.id.clone(), geo: b.geo() });
                return Ok(r);
            }
            if c.yes(&flow, "Join") || merge {
                let joined = join::join(&[g, b.geo()], 1e-6).ok_or("The curves do not join")?;
                let mut r = Reply::finish(Vec::new()).say("Match Curve: joined");
                r.replace.push(Replacement {
                    id: a.id.clone(),
                    geo: joined,
                });
                r.delete.push(b.id.clone());
                return Ok(r);
            }
            let mut r = Reply::finish(Vec::new()).say("Match Curve: done");
            r.replace.push(Replacement {
                id: a.id.clone(),
                geo: g,
            });
            Ok(r)
        },
    )
}

/// `a` changed so its start (or end) meets `target` with continuity `cont`. The curve is
/// worked as a non-rational NURBS (degree ≥ 3, enough poles to leave the other end alone).
fn matched(
    a: &CurveRef,
    at_end: bool,
    target: &End,
    cont: Continuity,
    keep: Option<Continuity>,
) -> Option<Geo> {
    let g = a.geo();
    let g = if at_end { join::reverse(&g)? } else { g };
    // A non-rational NURBS to work on.
    let (p, mut pts, mut kv) = match &g {
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } if *degree >= 3 && weights.iter().all(|w| (w - 1.0).abs() < 1e-12) => {
            (*degree, points.clone(), knots.clone())
        }
        _ => {
            let n = 24;
            let samples: Vec<Vec3> = (0..=n)
                .filter_map(|i| om9_addon_geom::eval_geo(&g, i as f64 / n as f64))
                .collect();
            match om9_addon_geom::interpolate(&samples, 3, false)? {
                Geo::Nurbs {
                    degree,
                    points,
                    knots,
                    ..
                } => (degree, points, knots),
                _ => return None,
            }
        }
    };
    // Enough poles: the matched end moves poles 0..=k, the kept end needs its own.
    let need = cont.order() + 1 + keep.map_or(0, |k| k.order() + 1) + 1;
    while pts.len() < need {
        let (i, _) = kv
            .windows(2)
            .enumerate()
            .max_by(|x, y| (x.1[1] - x.1[0]).total_cmp(&(y.1[1] - y.1[0])))?;
        let u = (kv[i] + kv[i + 1]) / 2.0;
        let (np, nk) = insert_knot(p, &pts, &kv, u);
        pts = np;
        kv = nk;
    }
    let old = pts.clone();
    pts[0] = target.point;
    if cont >= Continuity::Tangency {
        let len = dist(old[1], old[0]).max(1e-9);
        pts[1] = om9_addon_abi::session::add(pts[0], scale(target.d[0], len));
    }
    if cont >= Continuity::Curvature {
        let pf = p as f64;
        let a1 = pf / (kv[p + 1] - kv[1]);
        let b2 = pf * (pf - 1.0) / (kv[p + 1] - kv[2]);
        let s = a1 * dist(pts[1], pts[0]);
        // Keep the old tangential part of C''(0); the normal part gives the curvature.
        let c2_old = scale(
            sub(
                scale(sub(old[2], old[1]), 1.0 / (kv[p + 2] - kv[2])),
                scale(sub(old[1], old[0]), 1.0 / (kv[p + 1] - kv[1])),
            ),
            b2,
        );
        let t_old = om9_addon_abi::session::unit(sub(old[1], old[0]));
        let lambda = dot(c2_old, t_old);
        let c2 = om9_addon_abi::session::add(scale(target.d[1], s * s), scale(target.d[0], lambda));
        let q = om9_addon_abi::session::add(
            scale(c2, 1.0 / b2),
            scale(sub(pts[1], pts[0]), 1.0 / (kv[p + 1] - kv[1])),
        );
        pts[2] = om9_addon_abi::session::add(pts[1], scale(q, kv[p + 2] - kv[2]));
    }
    let out = Geo::Nurbs {
        degree: p,
        points: pts,
        weights: Vec::new(),
        knots: kv,
    };
    if at_end {
        join::reverse(&out)
    } else {
        Some(out)
    }
}

/// Insert one knot into a non-rational clamped NURBS (Boehm).
fn insert_knot(p: usize, pts: &[Vec3], kv: &[f64], u: f64) -> (Vec<Vec3>, Vec<f64>) {
    let n = pts.len();
    let mut k = p;
    while k + 1 < n && kv[k + 1] <= u {
        k += 1;
    }
    let mut out = Vec::with_capacity(n + 1);
    for i in 0..=n {
        out.push(if i + p <= k {
            pts[i]
        } else if i > k {
            pts[i - 1]
        } else {
            let a = (u - kv[i]) / (kv[i + p] - kv[i]);
            om9_addon_abi::session::add(scale(pts[i - 1], 1.0 - a), scale(pts[i], a))
        });
    }
    let mut nk = kv.to_vec();
    nk.insert(k + 1, u);
    (out, nk)
}
