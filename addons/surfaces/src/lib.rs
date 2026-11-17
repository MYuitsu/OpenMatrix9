// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix-style **Surface** menu for OM9. Results are NURBS surfaces (one tensor-product
//! patch each, fitted through computed rows of points) unless a command makes a mesh:
//! sweeps (1 rail, 2 rails, profile, four profiles, multi rail), loft, network, patch, edge
//! curves, planar curves, plane, revolve (exact) and rail revolve, extrude, blends, fillets
//! and chamfers (variable too), offsets, tween, merge, match, drape, heightfield, split,
//! extend, extrude normal to surface, unroll and smash, soft edit, cutting plane, seam,
//! tangent, rebuild, refit, shrink and untrim.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, OptionToken, Replacement, Reply, Selected, SelectedMesh, Vec3, Want, add,
    cross, dist, dot, fmt, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show};
use om9_addon_geom::fit::{grid, grid_mesh, surface_on_grid, surface_through};
use om9_addon_geom::mesh::{Frame, Mesh};
use om9_addon_geom::nurbs::{self, Srf};
use om9_addon_geom::path::{Path, Target};
use om9_addon_geom::exact::{self, Nc};
use om9_addon_geom::sweep;
use om9_addon_geom::{interpolate, section, shapes};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::f64::consts::FRAC_PI_2;

pub const SWEEP1: &str = "SurfaceSweepSweep1Rail";
pub const SWEEP2: &str = "SurfaceSweepSweep2Rails";
pub const PROFILE_SWEEP: &str = "SurfaceSweepProfileSweep";
pub const PLANAR: &str = "SurfacePlanarCurves";
pub const PLANE: &str = "SurfacePlaneCornertoCorner";
pub const REBUILD: &str = "OthersSurfaceRebuild";
pub const BLEND: &str = "SurfaceBlendSurface";
pub const VAR_BLEND: &str = "SurfaceVariableFilletBlendChamferVariableBlendSurfaces";
pub const LOFT: &str = "SurfaceLoft";
pub const NETWORK: &str = "SurfaceCurveNetwork";
pub const SHRINK: &str = "SurfaceSurfaceEditToolsShrinkTrimmedSurface";
pub const PATCH: &str = "SurfacePatch";
pub const EDGE_CURVES: &str = "SurfaceEdgeCurves";
pub const UNTRIM: &str = "SurfaceSurfaceEditToolsUntrim";
pub const EXTRUDE_ALL: &str = "SurfaceGVDExtrudeAll";
pub const REVOLVE: &str = "SurfaceRevolve";
pub const RAIL_REVOLVE: &str = "SurfaceRailRevolve";
pub const FOUR_PROFILE_SWEEP: &str = "BuilderFourProfileSweep";
pub const MULTI_RAIL: &str = "SurfaceSweepSweepMultiRail";
pub const TWEEN: &str = "SurfaceTweenSurfaces";
pub const FILLET: &str = "SurfaceFilletSurfaces";
pub const VAR_FILLET: &str = "SurfaceVariableFilletBlendChamferVariableFilletSurfaces";
pub const CHAMFER: &str = "SurfaceChamferSurfaces";
pub const VAR_CHAMFER: &str = "SurfaceVariableFilletBlendChamferVariableChamferSurfaces";
pub const OFFSET: &str = "SurfaceOffsetSurface";
pub const VAR_OFFSET: &str = "SurfaceVariableOffsetSurface";
pub const MERGE: &str = "SurfaceSurfaceEditToolsMerge";
pub const MATCH: &str = "SurfaceSurfaceEditToolsMatch";
pub const DRAPE: &str = "SurfaceDrape";
pub const HEIGHTFIELD: &str = "SurfaceHeightfieldfromImage";
pub const SPLIT_ISO: &str = "SurfaceSurfaceEditToolsSplitatIsocurve";
pub const EXTEND: &str = "SurfaceExtendSurface";
pub const EXTRUDE_NORMAL: &str = "SurfaceExtrudeCurveNormalToSurface";
pub const UNROLL: &str = "SurfaceUnrollDevelopableSrf";
pub const SMASH: &str = "SurfaceSmash";
pub const SOFT_EDIT: &str = "SurfaceSurfaceEditToolsSoftEdit";
pub const CUTTING_PLANE: &str = "SurfacePlaneCuttingPlane";
pub const SEAM: &str = "SurfaceSurfaceEditToolsAdjustClosedSurfaceSeam";
pub const SET_TANGENT: &str = "SurfaceSurfaceEditToolsSetSurfaceTangent";
pub const REFIT: &str = "SurfaceSurfaceEditToolsRefittoTolerance";

pub struct Surfaces;

impl Interactive for Surfaces {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        SWEEP1,
        SWEEP2,
        PROFILE_SWEEP,
        PLANAR,
        PLANE,
        REBUILD,
        BLEND,
        VAR_BLEND,
        LOFT,
        NETWORK,
        SHRINK,
        PATCH,
        EDGE_CURVES,
        UNTRIM,
        EXTRUDE_ALL,
        REVOLVE,
        RAIL_REVOLVE,
        FOUR_PROFILE_SWEEP,
        MULTI_RAIL,
        TWEEN,
        FILLET,
        VAR_FILLET,
        CHAMFER,
        VAR_CHAMFER,
        OFFSET,
        VAR_OFFSET,
        MERGE,
        MATCH,
        DRAPE,
        HEIGHTFIELD,
        SPLIT_ISO,
        EXTEND,
        EXTRUDE_NORMAL,
        UNROLL,
        SMASH,
        SOFT_EDIT,
        CUTTING_PLANE,
        SEAM,
        SET_TANGENT,
        REFIT,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        // Pointer moves of commands whose preview does not follow the pointer reuse the
        // previous move's reply when nothing else changed (see `om9_addon_geom::memo`).
        if STEADY_PREVIEW.contains(&e.command.as_str()) {
            return om9_addon_geom::memo::steady_hover(e, dispatch);
        }
        dispatch(e)
    }
}

// ---- shared ----------------------------------------------------------------------------------

/// Surface edges as curves (Rhino: "Select curves or surface edges"): a click on an
/// untrimmed surface near one of its edges stands for that edge — the exact isoparametric
/// curve — in the commands that take curves (Sweep1, Sweep2, Loft, NetworkSrf, EdgeSrf).
#[derive(Serialize, Deserialize, Default, Clone)]
struct EdgePicks {
    /// (surface id, the click) for each edge picked.
    #[serde(default)]
    picks: Vec<(String, Vec3)>,
    /// The selection seen last time (to tell which object the click just added).
    #[serde(default)]
    seen: Vec<String>,
    #[serde(default)]
    inner: Value,
}

fn with_surface_edges(e: &Event, inner: fn(&Event) -> Reply) -> Reply {
    let mut st: EdgePicks = if matches!(e.input, Input::Start) {
        EdgePicks::default()
    } else {
        serde_json::from_value(e.state.clone()).ok().filter(|x: &EdgePicks| x.inner != Value::Null || !x.picks.is_empty() || !x.seen.is_empty()).unwrap_or_else(|| EdgePicks { inner: e.state.clone(), ..Default::default() })
    };
    let is_surface = |s: &Selected| matches!(s.exact, Some(Geo::Surface { .. }));
    // A surface just added by a click: the click names its edge.
    if let (Input::Selection, Some(p)) = (&e.input, e.pick) {
        let new: Vec<&Selected> = e.selection.iter().filter(|s| !st.seen.contains(&s.id)).collect();
        if let [s] = new.as_slice() {
            if is_surface(s) {
                st.picks.push((s.id.clone(), p));
            }
        }
    }
    st.seen = e.selection.iter().map(|s| s.id.clone()).collect();
    // The selection the command sees: picked surfaces become their edge curves.
    let mut sel: Vec<Selected> = Vec::new();
    for s in &e.selection {
        let edges: Vec<&Vec3> = st.picks.iter().filter(|(id, _)| id == &s.id).map(|(_, p)| p).collect();
        if !is_surface(s) || edges.is_empty() {
            sel.push(s.clone());
            continue;
        }
        let Some(f) = Srf::of(s.exact.as_ref().unwrap()) else { continue };
        for (k, p) in edges.into_iter().enumerate() {
            let (dir, at) = nearest_edge(&f, *p);
            if let Some(c) = om9_addon_geom::surfedit::edge_curve(s.exact.as_ref().unwrap(), dir, at > 0.5) {
                let g = c.to_geo();
                let points = nurbs::samples(&g, 64);
                let closed = c.closed(1e-9);
                let lo = points.iter().fold([f64::MAX; 3], |a, q| [a[0].min(q[0]), a[1].min(q[1]), a[2].min(q[2])]);
                let hi = points.iter().fold([f64::MIN; 3], |a, q| [a[0].max(q[0]), a[1].max(q[1]), a[2].max(q[2])]);
                sel.push(Selected { id: format!("{}#edge{k}", s.id), kind: "curve".into(), name: format!("{} edge", s.name), bbox: [lo, hi], points, closed, exact: Some(g), mesh: None, ..s.clone() });
            }
        }
    }
    let mut sub = e.clone();
    sub.selection = sel;
    sub.state = st.inner.clone();
    let mut r = inner(&sub);
    // Edges are not objects: never delete or replace them.
    r.delete.retain(|id| !id.contains("#edge"));
    r.replace.retain(|x| !x.id.contains("#edge"));
    if !r.done {
        st.inner = r.state.clone();
        r.state = serde_json::to_value(&st).unwrap_or(Value::Null);
    }
    r
}

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

fn nurbs_surfaces(e: &Event) -> Vec<(&Selected, Srf<'_>)> {
    e.selection
        .iter()
        .filter_map(|s| s.exact.as_ref().and_then(Srf::of).map(|f| (s, f)))
        .collect()
}

fn meshes(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| {
            s.kind != "curve"
                && s.gem.is_none()
                && s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty())
        })
        .collect()
}

/// Dense samples of a curve (exact when known), without the closing point.
fn dense(s: &Selected) -> Vec<Vec3> {
    match &s.exact {
        Some(
            g @ (Geo::Line { .. }
            | Geo::Circle { .. }
            | Geo::Arc { .. }
            | Geo::Nurbs { .. }
            | Geo::Polyline { .. }),
        ) => {
            let mut p = nurbs::samples(g, 400);
            if s.closed && p.len() > 2 && dist(p[0], *p.last().unwrap()) < 1e-9 {
                p.pop();
            }
            p
        }
        _ => s.points.clone(),
    }
}

/// `n` points evenly by arc length (closed: `n` points round, the seam not repeated).
pub fn resample(points: &[Vec3], closed: bool, n: usize) -> Vec<Vec3> {
    let Some(p) = Path::new(points, closed) else {
        return points.to_vec();
    };
    let len = p.length();
    if closed {
        (0..n).map(|i| p.at(len * i as f64 / n as f64)).collect()
    } else {
        (0..n)
            .map(|i| p.at(len * i as f64 / (n - 1).max(1) as f64))
            .collect()
    }
}

/// Close rows of a closed direction by repeating the first point.
fn closed_rows(mut rows: Vec<Vec<Vec3>>, close_v: bool) -> Vec<Vec<Vec3>> {
    if close_v {
        for r in &mut rows {
            r.push(r[0]);
        }
    }
    rows
}

fn surface(rows: &[Vec<Vec3>]) -> Result<Geo, String> {
    surface_through(rows, [3, 3]).ok_or_else(|| "could not fit a surface".to_owned())
}

/// Create on Enter, preview before.
fn built(
    e: &Event,
    label: &str,
    ask: &str,
    make: impl FnOnce() -> Result<(Vec<Geo>, String), String>,
) -> Reply {
    match make() {
        Ok((geo, what)) if !geo.is_empty() => {
            if matches!(e.input, Input::Enter) {
                Reply::finish(geo).say(format!("{label}: {what}"))
            } else {
                Reply::ask(format!("{label}: {what}. Enter to create"), json!({}))
                    .want(Want::Objects)
                    .preview(geo)
            }
        }
        Ok((_, why)) | Err(why) => {
            let r = Reply::ask(format!("{label}: {ask}"), json!({})).want(Want::Objects);
            if matches!(e.input, Input::Enter) {
                r.refuse(format!("{label}: {why}"))
            } else {
                r
            }
        }
    }
}

fn replaced(
    e: &Event,
    label: &str,
    ask: &str,
    make: impl FnOnce() -> Result<(Vec<(String, Geo)>, Vec<Geo>, String), String>,
) -> Reply {
    match make() {
        Ok((items, extra, what)) if !items.is_empty() || !extra.is_empty() => {
            if matches!(e.input, Input::Enter) {
                let mut r = Reply::finish(extra).say(format!("{label}: {what}"));
                r.replace = items
                    .into_iter()
                    .map(|(id, geo)| Replacement { id, geo })
                    .collect();
                r
            } else {
                let mut preview: Vec<Geo> = items.into_iter().map(|x| x.1).collect();
                preview.extend(extra);
                Reply::ask(format!("{label}: {what}. Enter to apply"), json!({}))
                    .want(Want::Objects)
                    .preview(preview)
            }
        }
        Ok((_, _, why)) | Err(why) => {
            let r = Reply::ask(format!("{label}: {ask}"), json!({})).want(Want::Objects);
            if matches!(e.input, Input::Enter) {
                r.refuse(format!("{label}: {why}"))
            } else {
                r
            }
        }
    }
}

/// Signed area of a closed loop in a frame's y-z plane (the plane across a rail).
fn area_yz(p: &[Vec3]) -> f64 {
    let n = p.len();
    (0..n)
        .map(|i| p[i][1] * p[(i + 1) % n][2] - p[(i + 1) % n][1] * p[i][2])
        .sum::<f64>()
        / 2.0
}

/// Profiles brought to one point count, one winding and matching start points (closed), in
/// the local frames of the rail at their stations.
fn local_profiles(profiles: &[(&Selected, Frame)], n: usize) -> Vec<Vec<Vec3>> {
    let mut out: Vec<Vec<Vec3>> = Vec::new();
    for (p, f) in profiles {
        let closed = p.closed;
        let mut l: Vec<Vec3> = resample(&dense(p), closed, n)
            .iter()
            .map(|&q| f.local(q))
            .collect();
        if closed {
            if let Some(first) = out.first() {
                if (area_yz(&l) > 0.0) != (area_yz(first) > 0.0) {
                    l.reverse();
                }
                // Start where the first profile starts (by direction from its center).
                let c = |v: &[Vec3]| {
                    scale(
                        v.iter().fold([0.0; 3], |a, &q| add(a, q)),
                        1.0 / v.len() as f64,
                    )
                };
                let (c0, c1) = (c(first), c(&l));
                let d0 = unit(sub(first[0], c0));
                let k = (0..l.len())
                    .max_by(|&a, &b| {
                        dot(unit(sub(l[a], c1)), d0).total_cmp(&dot(unit(sub(l[b], c1)), d0))
                    })
                    .unwrap();
                l.rotate_left(k);
            }
        } else if let Some(first) = out.first()
            && dist(first[0], l[0]) > dist(first[0], *l.last().unwrap())
        {
            l.reverse();
        }
        out.push(l);
    }
    out
}

/// Blend local profiles placed at arc lengths `at` to the arc length `s` (wrapping when the
/// rail is closed of length `len`).
fn profile_at(locals: &[Vec<Vec3>], at: &[f64], s: f64, closed_len: Option<f64>) -> Vec<Vec3> {
    let k = locals.len();
    if k == 1 {
        return locals[0].clone();
    }
    let mut order: Vec<usize> = (0..k).collect();
    order.sort_by(|&a, &b| at[a].total_cmp(&at[b]));
    let (mut a, mut b, mut t) = (order[0], order[0], 0.0);
    let mut found = false;
    for w in order.windows(2) {
        if s >= at[w[0]] && s <= at[w[1]] {
            a = w[0];
            b = w[1];
            t = (s - at[a]) / (at[b] - at[a]).max(1e-12);
            found = true;
            break;
        }
    }
    if !found {
        let (first, last) = (order[0], order[k - 1]);
        match closed_len {
            Some(len) => {
                // Between the last and (wrapped) first profile.
                let span = len - at[last] + at[first];
                let d = if s > at[last] {
                    s - at[last]
                } else {
                    s + len - at[last]
                };
                a = last;
                b = first;
                t = d / span.max(1e-12);
            }
            None => {
                let i = if s < at[first] { first } else { last };
                return locals[i].clone();
            }
        }
    }
    let t = t * t * (3.0 - 2.0 * t);
    locals[a]
        .iter()
        .zip(&locals[b])
        .map(|(p, q)| add(*p, scale(sub(*q, *p), t)))
        .collect()
}

// ---- sweeps ----------------------------------------------------------------------------------

/// The rail is the longest selected curve (or the one named first); profiles are the rest.
fn rail_and_profiles<'a>(e: &'a Event) -> Result<(&'a Selected, Vec<&'a Selected>), String> {
    let cs = curves(e);
    if cs.len() < 2 {
        return Err("select the rail and at least one profile".into());
    }
    let len = |s: &Selected| Path::new(&s.points, s.closed).map_or(0.0, |p| p.length());
    let rail = *cs.iter().max_by(|a, b| len(a).total_cmp(&len(b))).unwrap();
    let others: Vec<&Selected> = cs.into_iter().filter(|c| c.id != rail.id).collect();
    // Profiles sit on the rail (Profile Placer puts their base on it, a tube profile surrounds
    // it). Other curves caught by a window selection - a gem layout circle, a guide - are not
    // profiles; when none is on the rail, every curve is used, as before.
    let touching: Vec<&Selected> = match Path::new(&rail.points, rail.closed) {
        Some(path) => others
            .iter()
            .copied()
            .filter(|c| {
                // On the rail: the profile's middle is no farther from the rail than the
                // profile is wide (a tube profile around the rail, a shank profile on it).
                let pts = dense(c);
                if pts.is_empty() {
                    return false;
                }
                let mid = scale(
                    pts.iter().fold([0.0; 3], |a, &q| add(a, q)),
                    1.0 / pts.len() as f64,
                );
                let radius = pts.iter().map(|&q| dist(q, mid)).fold(0.0, f64::max);
                dist(mid, path.at(path.closest(mid))) <= (0.75 * radius).max(0.5)
            })
            .collect(),
        None => Vec::new(),
    };
    Ok((
        rail,
        if touching.is_empty() {
            others
        } else {
            touching
        },
    ))
}

pub fn sweep_rows(
    rail_pts: &[Vec3],
    rail_closed: bool,
    profiles: &[&Selected],
    up: Option<Vec3>,
    stations: usize,
    n: usize,
) -> Option<Vec<Vec<Vec3>>> {
    let rail = Path::new(rail_pts, rail_closed)?;
    let len = rail.length();
    let at: Vec<f64> = profiles
        .iter()
        .map(|p| {
            let pts = dense(p);
            let c = scale(
                pts.iter().fold([0.0; 3], |a, &q| add(a, q)),
                1.0 / pts.len() as f64,
            );
            rail.closest(c)
        })
        .collect();
    let frames: Vec<(&Selected, Frame)> = profiles
        .iter()
        .zip(&at)
        .map(|(p, &s)| (*p, rail.frame(s, up)))
        .collect();
    let locals = local_profiles(&frames, n);
    let closed_profiles = profiles.iter().all(|p| p.closed);
    let count = if rail_closed { stations } else { stations + 1 };
    let mut rows: Vec<Vec<Vec3>> = (0..count)
        .map(|i| {
            let s = len * i as f64 / stations as f64;
            let f = rail.frame(s, up);
            profile_at(&locals, &at, s, rail_closed.then_some(len))
                .iter()
                .map(|&q| f.point(q))
                .collect()
        })
        .collect();
    if rail_closed {
        rows.push(rows[0].clone());
    }
    Some(closed_rows(rows, closed_profiles))
}

/// The exact curve of a selected curve, or one fitted through its samples.
fn nc_of(s: &Selected) -> Option<Nc> {
    if let Some(c) = exact_curve(s) {
        return Some(c);
    }
    let mut p = dense(s);
    if s.closed && p.len() > 2 && dist(p[0], *p.last()?) < 1e-9 {
        p.pop();
    }
    Nc::from_geo(&interpolate(&p, 3, s.closed)?)
}

fn sweep_style(name: &str) -> sweep::Style {
    match name.to_ascii_lowercase().as_str() {
        "roadliketop" | "roadlike_top" => sweep::Style::Roadlike([0.0, 0.0, 1.0]),
        "roadlikeright" | "roadlike_right" => sweep::Style::Roadlike([1.0, 0.0, 0.0]),
        "roadlikefront" | "roadlike_front" => sweep::Style::Roadlike([0.0, 1.0, 0.0]),
        _ => sweep::Style::Freeform,
    }
}

/// Sweep1 / Sweep2 command state: the rail(s), then the sections, then the options (Rhino 5
/// order).
#[derive(Serialize, Deserialize, Default, Clone)]
struct SweepState {
    /// 0: select the rail(s); 1: select the sections; 2: options and preview.
    stage: u8,
    rails: Vec<String>,
    /// Section ids in pick order (curves and point objects).
    sections: Vec<String>,
    /// Point option: a point typed or clicked to start / end the sweep.
    point_start: Option<Vec3>,
    point_end: Option<Vec3>,
    /// Waiting for the Point option's point.
    want_point: bool,
    /// The options stage's own state (a [`Flow`]).
    flow: Value,
}

/// Sweep1 from a rail and sections (point sections first or last).
fn sweep1_surface(
    e: &Event,
    rail: &Selected,
    sections: &[Nc],
    style: &str,
    closed: bool,
    global: bool,
    simplify: (&str, usize, f64),
) -> Result<(Geo, String), String> {
    let rail_nc = nc_of(rail).ok_or("the rail has no curve")?;
    let secs: Vec<Nc> = sections
        .iter()
        .map(|c| {
            let point = c.points.iter().all(|p| dist(exact::euclid(*p), exact::euclid(c.points[0])) < 1e-12);
            if point { c.clone() } else { simplify_section(c.clone(), simplify.0, simplify.1, simplify.2) }
        })
        .collect();
    let o = sweep::Options {
        style: sweep_style(style),
        closed,
        global_blend: global,
        tol: e.number("sw_tolerance", 0.001).max(1e-6),
        ..Default::default()
    };
    let s = sweep::sweep1(&rail_nc, &secs, &o)?;
    let how = if s.exact {
        "exact".to_owned()
    } else {
        format!("within {} mm", fmt(o.tol))
    };
    Ok((s.geo(), how))
}

/// Sweep2 between two rails.
fn sweep2_surface(
    e: &Event,
    rails: [&Selected; 2],
    sections: &[Nc],
    closed: bool,
    maintain_height: bool,
    simplify: (&str, usize, f64),
) -> Result<(Geo, String), String> {
    let ra = nc_of(rails[0]).ok_or("the rail has no curve")?;
    let rb = nc_of(rails[1]).ok_or("the rail has no curve")?;
    let secs: Vec<Nc> = sections
        .iter()
        .map(|c| {
            let point = c.points.iter().all(|p| dist(exact::euclid(*p), exact::euclid(c.points[0])) < 1e-12);
            if point { c.clone() } else { simplify_section(c.clone(), simplify.0, simplify.1, simplify.2) }
        })
        .collect();
    let o = sweep::Options2 {
        closed,
        maintain_height,
        tol: e.number("sw_tolerance", 0.001).max(1e-6),
        ..Default::default()
    };
    let s = sweep::sweep2(&ra, &rb, &secs, &o)?;
    Ok((s.geo(), format!("within {} mm", fmt(o.tol))))
}

fn sweep1(e: &Event) -> Reply {
    if e.command != SWEEP1 {
        return profile_sweep(e);
    }
    sweep_command(e, false)
}

fn sweep2(e: &Event) -> Reply {
    sweep_command(e, true)
}

/// Rhino 5 Sweep1 (`two` = false) and Sweep2: rail(s), cross sections (Point), Enter, options
/// with a preview, Enter.
fn sweep_command(e: &Event, two: bool) -> Reply {
    let label = if two { "Sweep 2 Rails" } else { "Sweep 1 Rail" };
    let need = if two { 2 } else { 1 };
    let mut st: SweepState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    let by_id = |id: &str| e.selection.iter().find(|s| s.id == id);
    let is_section = |s: &Selected| (s.kind == "curve" && s.points.len() >= 2) || (s.kind == "point" && !s.points.is_empty());
    let save = |st: &SweepState| serde_json::to_value(st).unwrap_or(Value::Null);
    let rail_prompt = |have: usize| match (two, have) {
        (false, _) => "Select rail",
        (true, 0) => "Select first rail",
        (true, _) => "Select second rail",
    };
    // Start with curves already selected (Matrix users window-select the rails and the
    // profiles): the longest curve(s) are the rails, the rest the sections in pick order.
    // (A one-shot call - Enter with no state, as SubD's sweep makes - works the same.)
    if matches!(e.input, Input::Start) || e.state.is_null() {
        st = SweepState::default();
        let cs = curves(e);
        if cs.len() > need {
            let rails: Vec<&Selected> = if two {
                let len = |s: &Selected| Path::new(&s.points, s.closed).map_or(0.0, |p| p.length());
                let mut by_len = cs.clone();
                by_len.sort_by(|a, b| len(b).total_cmp(&len(a)));
                by_len[..2].to_vec()
            } else {
                rail_and_profiles(e).map(|(r, _)| vec![r]).unwrap_or_default()
            };
            if rails.len() == need {
                st.rails = rails.iter().map(|r| r.id.clone()).collect();
                let profiles: Vec<String> = if two {
                    cs.iter().filter(|c| !st.rails.contains(&c.id)).map(|c| c.id.clone()).collect()
                } else {
                    rail_and_profiles(e).map(|(_, p)| p.iter().map(|c| c.id.clone()).collect()).unwrap_or_default()
                };
                st.sections = profiles;
                // Points selected too are point sections.
                st.sections.extend(e.selection.iter().filter(|s| s.kind == "point").map(|s| s.id.clone()));
                st.stage = 2;
            }
        } else {
            st.rails = cs.iter().map(|c| c.id.clone()).collect();
            st.stage = u8::from(st.rails.len() == need);
        }
    }
    if st.stage == 0 {
        for c in curves(e) {
            if st.rails.len() < need && !st.rails.contains(&c.id) {
                st.rails.push(c.id.clone());
            }
        }
        if st.rails.len() == need {
            st.stage = 1;
        } else {
            let r = Reply::ask(rail_prompt(st.rails.len()), save(&st)).want(Want::Objects);
            return if matches!(e.input, Input::Enter) { r.refuse(format!("{label}: select the rail")) } else { r };
        }
    }
    let rails: Vec<Selected> = st.rails.iter().filter_map(|id| by_id(id).cloned()).collect();
    if rails.len() != need {
        return Reply::ask(rail_prompt(0), save(&SweepState::default()))
            .want(Want::Objects)
            .refuse(format!("{label}: the rail is no longer selected"));
    }
    let mut entering = false;
    if st.stage == 1 {
        // Sections: everything picked after the rail, in order.
        for s in e.selection.iter().filter(|s| !st.rails.contains(&s.id) && is_section(s)) {
            if !st.sections.contains(&s.id) {
                st.sections.push(s.id.clone());
            }
        }
        st.sections.retain(|id| by_id(id).is_some());
        let mut error = None;
        match &e.input {
            Input::Option { name, .. } if name.eq_ignore_ascii_case("Point") => st.want_point = true,
            Input::Point { point } if st.want_point => {
                if st.sections.is_empty() && st.point_start.is_none() {
                    st.point_start = Some(*point);
                } else {
                    st.point_end = Some(*point);
                }
                st.want_point = false;
            }
            Input::Text { text } if st.want_point => match om9_addon_abi::session::parse_point(text, &e.plane, None) {
                Some(p) => {
                    if st.sections.is_empty() && st.point_start.is_none() {
                        st.point_start = Some(p);
                    } else {
                        st.point_end = Some(p);
                    }
                    st.want_point = false;
                }
                None => error = Some(format!("{text}: type a point (x,y,z)")),
            },
            Input::Enter if st.want_point => st.want_point = false,
            Input::Enter => {
                if st.sections.is_empty() {
                    error = Some(format!("{label}: select the cross section curves"));
                } else {
                    st.stage = 2;
                }
            }
            _ => {}
        }
        if st.stage == 1 {
            let r = if st.want_point {
                Reply::ask("Point to start or end the sweep", save(&st)).want(Want::Point)
            } else {
                let n = st.sections.len() + usize::from(st.point_start.is_some()) + usize::from(st.point_end.is_some());
                let prompt = if n == 0 {
                    "Select cross section curves".to_owned()
                } else {
                    format!("Select cross section curves ({n} selected). Press Enter when done")
                };
                Reply::ask(prompt, save(&st)).want(Want::Objects).with_options(vec![OptionToken::action("Point")])
            };
            return match error {
                Some(why) => r.refuse(why),
                None => r,
            };
        }
        // Into the options: start them fresh (this Enter only ended the selection).
        st.flow = Value::Null;
        entering = true;
    }
    // Stage 2: the sections, then options with a live preview; Enter creates.
    let mut picked: Vec<Nc> = Vec::new();
    if let Some(p) = st.point_start {
        picked.push(exact::point_curve(p));
    }
    for id in &st.sections {
        let Some(s) = by_id(id) else { continue };
        if s.kind == "point" {
            picked.push(exact::point_curve(s.points[0]));
        } else if let Some(c) = nc_of(s) {
            picked.push(c);
        }
    }
    if let Some(p) = st.point_end {
        picked.push(exact::point_curve(p));
    }
    let rails_closed = rails.iter().all(|r| r.closed);
    let simplify_key = if two { "sw2" } else { "sw1" };
    let mut flow = Flow::new(label)
        .no_selection()
        .plain()
        .confirm("Press Enter to create the sweep, or change an option");
    if two {
        flow = flow.toggle("MaintainHeight", e.flag("sw2_maintain_height", false));
    } else {
        flow = flow.choice(
            "Style",
            &["Freeform", "RoadlikeTop", "RoadlikeRight", "RoadlikeFront"],
            match e.text("sw1_style", "freeform") {
                "roadlike_top" => "RoadlikeTop",
                "roadlike_right" => "RoadlikeRight",
                "roadlike_front" => "RoadlikeFront",
                _ => "Freeform",
            },
        );
    }
    if rails_closed && picked.len() >= 2 {
        flow = flow.toggle("Closed", e.flag(&format!("{simplify_key}_closed"), false));
    }
    if !two {
        flow = flow.toggle("GlobalBlend", e.flag("sw1_global_blend", false));
    }
    let flow = flow
        .choice(
            "Simplify",
            &["None", "Rebuild", "Refit"],
            match e.text(&format!("{simplify_key}_simplify"), "none") {
                "rebuild" => "Rebuild",
                "refit" => "Refit",
                _ => "None",
            },
        )
        .num("RebuildPoints", e.number(&format!("{simplify_key}_rebuild_points"), 10.0))
        .num("RefitTolerance", e.number(&format!("{simplify_key}_refit_tolerance"), 0.01));
    let make = |c: &Collected| -> Result<(Geo, String), String> {
        let simplify = c.opt(&flow, "Simplify").to_ascii_lowercase();
        let simplify = (simplify.as_str(), c.num(&flow, "RebuildPoints").round().max(4.0) as usize, c.num(&flow, "RefitTolerance").max(1e-6));
        let closed = rails_closed && picked.len() >= 2 && c.yes(&flow, "Closed");
        if two {
            sweep2_surface(e, [&rails[0], &rails[1]], &picked, closed, c.yes(&flow, "MaintainHeight"), simplify)
        } else {
            sweep1_surface(e, &rails[0], &picked, c.opt(&flow, "Style"), closed, c.yes(&flow, "GlobalBlend"), simplify)
        }
    };
    let mut sub = e.clone();
    sub.state = st.flow.clone();
    if entering || matches!(sub.input, Input::Selection) {
        sub.input = Input::Hover { point: [0.0; 3] };
    }
    let failed = std::cell::RefCell::new(None);
    let mut r = flow.step(
        &sub,
        &|c, _| match make(c) {
            Ok(g) => Show { geo: vec![g.0], ..Default::default() },
            Err(why) => {
                *failed.borrow_mut() = Some(why);
                Show::default()
            }
        },
        &|c| {
            let (g, what) = make(c)?;
            Ok(Reply::finish(vec![g]).say(format!("{label}: {} sections, {what}", picked.len())))
        },
    );
    if !r.done {
        st.flow = r.state.clone();
        r.state = save(&st);
        let why = failed.borrow_mut().take();
        if let (None, Some(why)) = (&r.error, why) {
            r = r.refuse(format!("{label}: {why}"));
        }
    }
    r
}

/// Profile Sweep and Four Profile Sweep (Matrix builders): the ring rail and its profiles
/// selected together, always all the way round a closed rail.
fn profile_sweep(e: &Event) -> Reply {
    let label = match e.command.as_str() {
        PROFILE_SWEEP => "Profile Sweep",
        _ => "Four Profile Sweep",
    };
    built(e, label, "select the rail and the profiles, Enter", || {
        let (rail, profiles) = rail_and_profiles(e)?;
        if e.command == FOUR_PROFILE_SWEEP && profiles.len() != 4 {
            return Err("select the ring rail and four profiles".into());
        }
        let secs: Vec<Nc> = profiles.iter().filter_map(|p| nc_of(p)).collect();
        if secs.len() != profiles.len() {
            return Err("a profile has no curve".into());
        }
        let (g, what) = sweep1_surface(
            e,
            rail,
            &secs,
            e.text("sw1_style", "freeform"),
            rail.closed,
            e.flag("sw1_global_blend", false),
            ("none", 10, 0.01),
        )?;
        Ok((vec![g], format!("{} profiles along the rail, {what}", secs.len())))
    })
}

fn multi_rail(e: &Event) -> Reply {
    let closed_profile = e.flag("smr_closed", true);
    built(
        e,
        "Sweep Multi Rail",
        "select three or more rails (in order around the profile), Enter",
        || {
            let cs = curves(e);
            if cs.len() < 3 {
                return Err("select three or more rails".into());
            }
            let rails: Vec<Nc> = cs.iter().filter_map(|c| nc_of(c)).collect();
            if rails.len() != cs.len() {
                return Err("a rail has no curve".into());
            }
            let tol = e.number("sw_tolerance", 0.001).max(1e-6);
            let s = sweep::multi_rail(&rails, closed_profile, tol)?;
            Ok((
                vec![s.geo()],
                format!("profile through {} rails, within {} mm", cs.len(), fmt(tol)),
            ))
        },
    )
}

/// The exact NURBS of a picked curve, running the way its samples (`points`) run (the
/// host's exact form does not carry a reversed flag).
fn exact_curve(s: &Selected) -> Option<Nc> {
    let c = Nc::from_geo(s.exact.as_ref()?)?;
    let n = s.points.len();
    if n < 10 {
        return Some(c);
    }
    let (a, b) = (c.closest(s.points[n / 10]), c.closest(s.points[n / 5]));
    Some(if b < a { c.reversed() } else { c })
}

/// Rhino 5 Loft from the exact section curves: they are aligned (direction, and the seams
/// of closed sections), made compatible (same degree and knots, shapes unchanged) and
/// interpolated pole by pole across, so every section lies exactly on the surface (except
/// Loose, which uses the sections' poles as its own). `None` when a section has no exact
/// form (the sampled loft is used then).
/// Rhino's Loft "Simplify": Rebuild (N control points through the section) or Refit (fewest
/// points within `tol`); `none` keeps the exact section.
fn simplify_section(c: Nc, how: &str, points: usize, tol: f64) -> Nc {
    let closed = c.closed(1e-9);
    let fit = |n: usize| -> Option<Nc> {
        let n = n.max(4);
        // A closed section repeats its start at the end (interpolate closes it).
        let step = if closed { n as f64 } else { (n - 1) as f64 };
        let pts: Vec<Vec3> = (0..n).map(|i| c.eval(i as f64 / step)).collect();
        Nc::from_geo(&interpolate(&pts, 3, closed)?)
    };
    match how {
        "rebuild" => fit(points).unwrap_or(c),
        "refit" => {
            let mut n = 8;
            while n <= 4096 {
                if let Some(f) = fit(n) {
                    let worst = (0..=400)
                        .map(|i| {
                            let p = c.eval(i as f64 / 400.0);
                            dist(p, f.eval(f.closest(p)))
                        })
                        .fold(0.0, f64::max);
                    if worst <= tol {
                        return f;
                    }
                }
                n *= 2;
            }
            c
        }
        _ => c,
    }
}

fn loft_exact(e: &Event, cs: &[&Selected], kind: &str, closed_loft: bool) -> Option<Result<Geo, String>> {
    // Sections in pick order; a point may start or end the loft (Rhino's Point option).
    let simplify = e.text("loft_simplify", "none");
    let rebuild_points = e.number("loft_rebuild_points", 10.0).round().max(4.0) as usize;
    let refit_tol = e.number("loft_refit_tolerance", 0.01).max(1e-6);
    let mut secs: Vec<Nc> = cs
        .iter()
        .map(|s| {
            if s.kind == "point" {
                Some(exact::point_curve(s.points[0]))
            } else {
                exact_curve(s).map(|c| simplify_section(c, simplify, rebuild_points, refit_tol))
            }
        })
        .collect::<Option<_>>()?;
    let is_point: Vec<bool> = cs.iter().map(|s| s.kind == "point").collect();
    if is_point.iter().enumerate().any(|(i, p)| *p && i != 0 && i + 1 != is_point.len()) {
        return Some(Err("a point can only start or end the loft".into()));
    }
    let curves_only: Vec<&Nc> = secs.iter().zip(&is_point).filter(|(_, p)| !**p).map(|(c, _)| c).collect();
    let closed = curves_only.iter().all(|c| c.closed(1e-6));
    if !closed && curves_only.iter().any(|c| c.closed(1e-6)) {
        return Some(Err("the sections must be all closed or all open".into()));
    }
    if closed_loft && secs.len() < 3 {
        return Some(Err("a closed loft needs three or more sections".into()));
    }
    if e.text("loft_seams", "automatic") != "natural" {
        exact::align_sections(&mut secs, closed);
    }
    exact::make_compatible(&mut secs);
    let along = (secs[0].degree, secs[0].knots.clone());
    let across = if kind == "loose" {
        exact::loose_across(&secs, closed_loft)
    } else {
        let params = match kind {
            // Open: m values over [0, 1]; closed: m + 1 (the first section again at 1).
            "uniform" => {
                let m = secs.len();
                let (count, step) = if closed_loft { (m + 1, m) } else { (m, m - 1) };
                (0..count).map(|i| i as f64 / step as f64).collect()
            }
            "tight" => exact::section_params(&secs, true, closed_loft),
            _ => exact::section_params(&secs, false, closed_loft),
        };
        let q = if kind == "straight" { 1 } else { 3 };
        exact::interpolate_across(&secs, &params, q, closed_loft)
            .map(|(rows, knots)| (rows, knots, if kind == "straight" { 1 } else { 3.min(secs.len() - 1) }))
    };
    let Some((rows, knots, q)) = across else {
        return Some(Err("could not fit the loft".into()));
    };
    Some(Ok(exact::surface(&rows, [q, along.0], [knots, along.1])))
}

fn loft(e: &Event) -> Reply {
    let kind = e.text("loft_style", "normal").to_owned();
    let closed_loft = e.flag("loft_closed", false);
    built(
        e,
        "Loft",
        "select the section curves in order, Enter",
        || {
            let cs = curves(e);
            // Sections in pick order, with points allowed (at the ends).
            let picked: Vec<&Selected> = e
                .selection
                .iter()
                .filter(|s| (s.kind == "curve" && s.points.len() >= 2) || (s.kind == "point" && !s.points.is_empty()))
                .collect();
            if picked.len() < 2 || cs.is_empty() {
                return Err("select two or more sections".into());
            }
            if let Some(g) = loft_exact(e, &picked, &kind, closed_loft) {
                // SplitAtTangents: a surface per smooth stretch of kinked sections.
                let g = g?;
                let parts = if e.flag("loft_split_at_tangents", false) { om9_addon_geom::surfedit::split_at_kinks(&g, 1) } else { vec![g] };
                let k = parts.len();
                let pieces = if k > 1 { format!(", {k} surfaces") } else { String::new() };
                return Ok((parts, format!("{} sections ({kind}){pieces}", cs.len())));
            }
            let closed = cs.iter().all(|c| c.closed);
            let n = 32;
            // A frame whose x is the first section's normal: the sections lie across it, so
            // their windings and starts can be matched (then back to world coordinates).
            let normal = shapes::loop_plane(&cs[0].points).map_or_else(
                || unit(sub(*cs[1].points.first().unwrap(), cs[0].points[0])),
                |p| p.1,
            );
            let side = if normal[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let f = Frame::new([0.0; 3], unit(cross(normal, side)), normal);
            let frames: Vec<(&Selected, Frame)> = cs.iter().map(|c| (*c, f)).collect();
            let rows_world: Vec<Vec<Vec3>> = local_profiles(&frames, n)
                .into_iter()
                .map(|r| r.into_iter().map(|q| f.point(q)).collect())
                .collect();
            let mut rows = closed_rows(rows_world, closed);
            if closed_loft {
                rows.push(rows[0].clone());
            }
            let deg = if kind == "straight" { 1 } else { 3 };
            let g = if kind == "loose" {
                surface_on_grid(&rows, [deg, 3])
            } else {
                surface_through(&rows, [deg, 3])
            }
            .ok_or("loft")?;
            Ok((vec![g], format!("{} sections ({kind})", cs.len())))
        },
    )
}

/// Coons patch through four boundary curves `[bottom (u), right (v), top (u), left (v)]`
/// given as point rows of matching corners.
pub fn coons(bottom: &[Vec3], top: &[Vec3], left: &[Vec3], right: &[Vec3]) -> Vec<Vec<Vec3>> {
    let (nu, nv) = (bottom.len(), left.len());
    let (p00, p10, p01, p11) = (bottom[0], bottom[nu - 1], top[0], top[nu - 1]);
    (0..nv)
        .map(|j| {
            let v = j as f64 / (nv - 1) as f64;
            (0..nu)
                .map(|i| {
                    let u = i as f64 / (nu - 1) as f64;
                    let lu = add(scale(bottom[i], 1.0 - v), scale(top[i], v));
                    let lv = add(scale(left[j], 1.0 - u), scale(right[j], u));
                    let b = add(
                        add(scale(p00, (1.0 - u) * (1.0 - v)), scale(p10, u * (1.0 - v))),
                        add(scale(p01, (1.0 - u) * v), scale(p11, u * v)),
                    );
                    sub(add(lu, lv), b)
                })
                .collect()
        })
        .collect()
}

/// Four boundary rows from 2–4 edge curves (2: ruled; 3: one side collapses) or one closed
/// boundary split in four.
fn boundary(cs: &[&Selected], n: usize) -> Result<[Vec<Vec3>; 4], String> {
    if cs.is_empty() {
        return Err("select the edge curves".into());
    }
    let rows: Vec<Vec<Vec3>> = if cs.len() == 1 && cs[0].closed {
        let p = resample(&dense(cs[0]), true, 4 * (n - 1));
        let mut q = p.clone();
        q.push(p[0]);
        (0..4)
            .map(|k| q[k * (n - 1)..=(k + 1) * (n - 1)].to_vec())
            .collect()
    } else {
        let mut pieces: Vec<Vec<Vec3>> = cs.iter().map(|c| resample(&dense(c), false, n)).collect();
        // Chain the curves end to end.
        let mut chain = vec![pieces.remove(0)];
        while !pieces.is_empty() {
            let tail = *chain.last().unwrap().last().unwrap();
            let (i, rev) = (0..pieces.len())
                .map(|i| {
                    (
                        i,
                        dist(pieces[i][0], tail),
                        dist(*pieces[i].last().unwrap(), tail),
                    )
                })
                .map(|(i, a, b)| if a <= b { (i, false, a) } else { (i, true, b) })
                .min_by(|a, b| a.2.total_cmp(&b.2))
                .map(|(i, r, _)| (i, r))
                .unwrap();
            let mut p = pieces.remove(i);
            if rev {
                p.reverse();
            }
            chain.push(p);
        }
        match chain.len() {
            2 => {
                // Ruled: the two curves (the second runs back) and straight sides.
                let (a, b) = (chain[0].clone(), chain[1].clone());
                let side = |p: Vec3, q: Vec3| {
                    (0..n)
                        .map(|k| add(p, scale(sub(q, p), k as f64 / (n - 1) as f64)))
                        .collect::<Vec<_>>()
                };
                vec![
                    a.clone(),
                    side(a[n - 1], b[0]),
                    b.clone(),
                    side(b[n - 1], a[0]),
                ]
            }
            3 => {
                let p = chain[2][n - 1];
                let mut c = chain.clone();
                c.push(vec![p; n]);
                c
            }
            4 => chain,
            _ => return Err("select 2, 3 or 4 edge curves (or one closed curve)".into()),
        }
    };
    let [a, b, c, d]: [Vec<Vec3>; 4] = rows.try_into().map_err(|_| "boundary".to_owned())?;
    Ok([a, b, c, d])
}

/// Rhino 5 EdgeSrf from 2, 3 or 4 exact curves: the surface's edges are the curves
/// themselves (2: the two curves are opposite edges, joined by straight lines; 3: the fourth
/// edge collapses to the corner). `None` when a curve has no exact form or they do not meet.
fn edge_srf_exact(cs: &[&Selected]) -> Option<Geo> {
    let all: Vec<Nc> = cs.iter().map(|s| exact_curve(s)).collect::<Option<_>>()?;
    let tol = 1e-3;
    let (bottom, right, top, left) = match all.len() {
        2 => {
            let (a, mut b) = (all[0].clone(), all[1].clone());
            if dist(a.start(), b.start()) + dist(a.end(), b.end()) > dist(a.start(), b.end()) + dist(a.end(), b.start()) {
                b = b.reversed();
            }
            let line = |p: Vec3, q: Vec3| Nc::from_geo(&Geo::Line { start: p, end: q });
            let (l, r) = (line(a.start(), b.start())?, line(a.end(), b.end())?);
            (a, r, b, l)
        }
        3 | 4 => {
            let c = exact::chain(&all, tol)?;
            if dist(c.last()?.end(), c[0].start()) > tol {
                return None;
            }
            let left = if c.len() == 4 { c[3].reversed() } else { exact::point_curve(c[0].start()) };
            (c[0].clone(), c[1].clone(), c[2].reversed(), left)
        }
        _ => return None,
    };
    let (rows, degree, knots) = exact::coons(&bottom, &right, &top, &left)?;
    Some(exact::surface(&rows, degree, knots))
}

fn patch(e: &Event) -> Reply {
    let label = if e.command == PATCH {
        "Patch"
    } else {
        "Edge Curves"
    };
    built(
        e,
        label,
        "select the edge curves (or one closed boundary), Enter",
        || {
            let cs = curves(e);
            if e.command == EDGE_CURVES {
                if let Some(g) = edge_srf_exact(&cs) {
                    return Ok((vec![g], format!("surface from {} edge curves", cs.len())));
                }
            } else {
                return patch_fit(e, &cs);
            }
            let n = 24;
            let [b0, r0, t0, l0] = boundary(&cs, n)?;
            // Chain order: bottom →, right ↑, top ←, left ↓. Coons wants top → and left ↑.
            let top: Vec<Vec3> = t0.into_iter().rev().collect();
            let left: Vec<Vec3> = l0.into_iter().rev().collect();
            let rows = coons(&b0, &top, &left, &r0);
            Ok((vec![surface(&rows)?], "surface from the boundary".into()))
        },
    )
}

/// Rhino 5 Patch: fit through the selected curves and points (Sample point spacing, U/V
/// spans, Stiffness), trimmed to the closed outer curve (Automatic trim).
fn patch_fit(e: &Event, cs: &[&Selected]) -> Result<(Vec<Geo>, String), String> {
    use om9_addon_geom::patch;
    let ncs: Vec<Nc> = cs.iter().filter_map(|c| nc_of(c)).collect();
    let points: Vec<Vec3> = e.selection.iter().filter(|s| s.kind == "point" && !s.points.is_empty()).map(|s| s.points[0]).collect();
    if ncs.is_empty() && points.len() < 3 {
        return Err("select curves or points".into());
    }
    let o = patch::Options {
        spacing: e.number("patch_spacing", 1.0).max(1e-4),
        u_spans: e.number("patch_u_spans", 10.0).round().clamp(1.0, 40.0) as usize,
        v_spans: e.number("patch_v_spans", 10.0).round().clamp(1.0, 40.0) as usize,
        stiffness: e.number("patch_stiffness", 1.0).max(0.0),
    };
    let fit = patch::patch(&ncs, &points, &o)?;
    let sheet = exact::surface(&fit.rows, fit.degree, fit.knots.clone());
    let what = format!("{} curves, {} points, {} mm from the input", ncs.len(), points.len(), fmt(fit.deviation));
    if !e.flag("patch_auto_trim", true) {
        return Ok((vec![sheet], what));
    }
    // The outer curve: a closed input curve, or the curves whose ends meet others' ends.
    let tol = 1e-3;
    let touches = |c: &Nc, p: Vec3| ncs.iter().any(|d| !std::ptr::eq(c, d) && (dist(d.start(), p) <= tol || dist(d.end(), p) <= tol));
    let boundary: Vec<Nc> = ncs.iter().filter(|c| touches(c, c.start()) && touches(c, c.end())).cloned().collect();
    let outer = ncs
        .iter()
        .filter(|c| c.closed(tol))
        .max_by(|a, b| {
            let span = |c: &Nc| {
                let (lo, hi) = (0..=32).map(|i| fit.flat(c.eval(i as f64 / 32.0))).fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), p| {
                    ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])
                });
                (hi[0] - lo[0]) * (hi[1] - lo[1])
            };
            span(a).total_cmp(&span(b))
        })
        .map(|c| vec![c.clone()])
        .or_else(|| (boundary.len() >= 2).then(|| patch::outer_loop(&boundary, tol)).flatten());
    let Some(outer) = outer else {
        return Ok((vec![sheet], what));
    };
    let on: Vec<Nc> = outer.iter().filter_map(|c| patch::on_patch(&fit, c)).collect();
    if on.len() != outer.len() {
        return Ok((vec![sheet], what));
    }
    // Display (where no B-Rep kernel runs): the patch's triangles inside the outer loop.
    let poly: Vec<[f64; 2]> = outer.iter().flat_map(|c| (0..32).map(move |i| c.eval(i as f64 / 32.0))).map(|p| fit.flat(p)).collect();
    let inside = |p: [f64; 2]| {
        let mut c = false;
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) {
                c = !c;
            }
        }
        c
    };
    let k = 48;
    let (lo, hi) = fit.range();
    let mut vertices = Vec::new();
    for i in 0..=k {
        for j in 0..=k {
            vertices.push(fit.at(i as f64 / k as f64, j as f64 / k as f64));
        }
    }
    let at = |i: usize, j: usize| (i * (k + 1) + j) as u32;
    let mut triangles = Vec::new();
    for i in 0..k {
        for j in 0..k {
            let mid = [
                lo[0] + (i as f64 + 0.5) / k as f64 * (hi[0] - lo[0]),
                lo[1] + (j as f64 + 0.5) / k as f64 * (hi[1] - lo[1]),
            ];
            if inside(mid) {
                triangles.push([at(i, j), at(i + 1, j), at(i + 1, j + 1)]);
                triangles.push([at(i, j), at(i + 1, j + 1), at(i, j + 1)]);
            }
        }
    }
    let display = Mesh { vertices, triangles }.geo("Patch", None, None);
    Ok((
        vec![Geo::Trimmed {
            surface: Box::new(sheet),
            loops: vec![on.iter().map(Nc::to_geo).collect()],
            display: Some(Box::new(display)),
        }],
        format!("{what}, trimmed"),
    ))
}

/// NetworkSrf NoAutoSort state: the first direction's curves (ids, in the order picked).
#[derive(Serialize, Deserialize, Default, Clone)]
struct NetworkState {
    first: Option<Vec<String>>,
}

/// Rhino 5 NetworkSrf NoAutoSort: Select curves in first direction (in order, Enter), Select
/// curves in second direction (in order, Enter); the surface runs through them as picked.
fn network_in_order(e: &Event) -> Reply {
    let label = "Curve Network";
    let mut st: NetworkState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = NetworkState::default();
    }
    let save = |st: &NetworkState| serde_json::to_value(st).unwrap_or(Value::Null);
    let enter = matches!(e.input, Input::Enter);
    let picked: Vec<&Selected> = curves(e);
    match &st.first {
        None => {
            if enter && picked.len() >= 2 {
                st.first = Some(picked.iter().map(|c| c.id.clone()).collect());
                return Reply::ask("Select curves in second direction. Press Enter when done", save(&st)).want(Want::Objects);
            }
            let r = Reply::ask("Select curves in first direction. Press Enter when done", save(&st)).want(Want::Objects);
            if enter { r.refuse(format!("{label}: select two or more curves")) } else { r }
        }
        Some(first) => {
            let second: Vec<&Selected> = picked.iter().filter(|c| !first.contains(&c.id)).copied().collect();
            if !enter || second.len() < 2 {
                let r = Reply::ask("Select curves in second direction. Press Enter when done", save(&st)).want(Want::Objects);
                return if enter { r.refuse(format!("{label}: select two or more curves")) } else { r };
            }
            let a: Option<Vec<Nc>> = first.iter().map(|id| e.selection.iter().find(|s| &s.id == id).and_then(nc_of)).collect();
            let b: Option<Vec<Nc>> = second.iter().map(|c| nc_of(c)).collect();
            let (Some(a), Some(b)) = (a, b) else {
                return Reply::ask("Select curves in second direction. Press Enter when done", save(&st)).want(Want::Objects).refuse(format!("{label}: a curve has no curve data"));
            };
            let tol = e.number("nw_edge_tolerance", 0.001).min(e.number("nw_interior_tolerance", 0.001)).max(1e-6);
            let gap = e.number("nw_cross_tolerance", 0.1).max(tol);
            match om9_addon_geom::network::network_in_order(&a, &b, tol, gap) {
                Ok(n) => Reply::finish(vec![exact::surface(&n.rows, n.degree, n.knots)])
                    .say(format!("{label}: {} × {} curves in the order picked, within {} mm", n.counts[0], n.counts[1], fmt(tol))),
                Err(why) => Reply::ask("Select curves in first direction. Press Enter when done", save(&NetworkState::default())).want(Want::Objects).refuse(format!("{label}: {why}")),
            }
        }
    }
}

fn network(e: &Event) -> Reply {
    if e.flag("nw_no_auto_sort", false) {
        return network_in_order(e);
    }
    built(
        e,
        "Curve Network",
        "select the curves of both directions, Enter",
        || {
            let cs = curves(e);
            if cs.len() < 3 {
                return Err("select at least three curves".into());
            }
            // An exact-curve Gordon surface through every curve (Rhino 5 NetworkSrf; closed
            // curves make one direction close round); the edge / interior tolerance is how
            // closely it follows them.
            {
                let ncs: Vec<Nc> = cs.iter().filter_map(|c| nc_of(c)).collect();
                if ncs.len() == cs.len() {
                    let tol = e.number("nw_edge_tolerance", 0.001).min(e.number("nw_interior_tolerance", 0.001)).max(1e-6);
                    let gap = e.number("nw_cross_tolerance", 0.1).max(tol);
                    let n = om9_addon_geom::network::network(&ncs, tol, gap)?;
                    return Ok((
                        vec![exact::surface(&n.rows, n.degree, n.knots)],
                        format!("{} × {} curves, within {} mm", n.counts[0], n.counts[1], fmt(tol)),
                    ));
                }
            }
            // Split into two families by direction (end to end vectors).
            let dir = |c: &Selected| unit(sub(*c.points.last().unwrap(), c.points[0]));
            let d0 = dir(cs[0]);
            let (a, b): (Vec<&Selected>, Vec<&Selected>) =
                cs.iter().partition(|c| dot(dir(c), d0).abs() > 0.5);
            let (sections, guides) = if a.len() >= 2 { (a, b) } else { (b, a) };
            if sections.len() < 2 {
                return Err("the network needs two or more curves running the same way".into());
            }
            // Order the sections along the guide (or along the perpendicular).
            let along = guides
                .first()
                .map_or_else(|| unit(cross(d0, [0.0, 0.0, 1.0])), |g| dir(g));
            let mut s = sections.clone();
            s.sort_by(|p, q| dot(p.points[0], along).total_cmp(&dot(q.points[0], along)));
            let n = 32;
            let rows: Vec<Vec<Vec3>> = s
                .iter()
                .map(|c| {
                    let mut r = resample(&dense(c), false, n);
                    if dot(sub(r[n - 1], r[0]), d0) < 0.0 {
                        r.reverse();
                    }
                    r
                })
                .collect();
            Ok((
                vec![surface(&rows)?],
                format!("{} × {} curves", s.len(), guides.len()),
            ))
        },
    )
}

fn planar(e: &Event) -> Reply {
    built(
        e,
        "Planar Curves",
        "select closed planar curves (curves inside others are holes), Enter",
        || {
            let cs: Vec<&Selected> = curves(e).into_iter().filter(|c| c.closed).collect();
            if cs.is_empty() {
                return Err("select closed planar curves".into());
            }
            let (o, n) = shapes::loop_plane(&cs[0].points).ok_or("the curve is not planar")?;
            let f = Frame::new(o, n, sub(cs[0].points[0], o));
            let flat = |c: &Selected| -> Vec<[f64; 2]> {
                dense(c)
                    .iter()
                    .map(|&p| {
                        let l = f.local(p);
                        [l[0], l[1]]
                    })
                    .collect()
            };
            let outlines: Vec<Vec<[f64; 2]>> = cs.iter().map(|c| flat(c)).collect();
            let inside = |p: [f64; 2], poly: &[[f64; 2]]| {
                let mut c = false;
                for i in 0..poly.len() {
                    let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
                    if (a[1] > p[1]) != (b[1] > p[1])
                        && p[0] < a[0] + (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1])
                    {
                        c = !c;
                    }
                }
                c
            };
            // Rhino: a curve wholly within another is a hole; curves that only overlap make
            // separate surfaces. "Wholly within": every sample inside.
            let within = |h: &[[f64; 2]], poly: &[[f64; 2]]| {
                let step = (h.len() / 64).max(1);
                h.iter().step_by(step).all(|p| inside(*p, poly))
            };
            let mut out = Vec::new();
            for (i, o_) in outlines.iter().enumerate() {
                let depth = outlines
                    .iter()
                    .enumerate()
                    .filter(|(j, other)| *j != i && within(o_, other))
                    .count();
                if depth % 2 == 1 {
                    continue;
                }
                // Holes: the curves right inside this one (one level deeper).
                let hole_ids: Vec<usize> = outlines
                    .iter()
                    .enumerate()
                    .filter(|(j, h)| {
                        *j != i && within(h, o_) && {
                            outlines
                                .iter()
                                .enumerate()
                                .filter(|(k, other)| *k != *j && within(h, other))
                                .count()
                                == depth + 1
                        }
                    })
                    .map(|(j, _)| j)
                    .collect();
                let holes: Vec<Vec<[f64; 2]>> = hole_ids.iter().map(|j| outlines[*j].clone()).collect();
                let poly = shapes::bridge_holes(o_, &holes);
                let tris = shapes::triangulate(&poly);
                let m = Mesh {
                    vertices: poly.iter().map(|p| f.point([p[0], p[1], 0.0])).collect(),
                    triangles: tris
                        .iter()
                        .map(|t| [t[0] as u32, t[1] as u32, t[2] as u32])
                        .collect(),
                };
                let display = m.geo("Planar surface", None, None);
                // Rhino's PlanarSrf: a plane trimmed by the curves themselves (exact NURBS
                // loops), the holes inside. The mesh shows where no B-Rep kernel runs.
                let ring = |j: usize| -> Geo {
                    exact_curve(cs[j]).map_or_else(
                        || {
                            let pts = dense(cs[j]);
                            let mut q = pts.clone();
                            if dist(q[0], *q.last().unwrap()) > 1e-9 {
                                q.push(q[0]);
                            }
                            Nc::from_geo(&Geo::Polyline { points: q, closed: false })
                                .map(|c| c.to_geo())
                                .unwrap_or(Geo::Polyline { points: pts, closed: true })
                        },
                        |c| c.to_geo(),
                    )
                };
                let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
                for p in o_ {
                    for a in 0..2 {
                        lo[a] = lo[a].min(p[a]);
                        hi[a] = hi[a].max(p[a]);
                    }
                }
                let pad = ((hi[0] - lo[0]).max(hi[1] - lo[1]) * 0.01).max(1e-3);
                let corner = |x: f64, y: f64| f.point([x, y, 0.0]);
                let sheet = Geo::Surface {
                    degree: [1, 1],
                    size: [2, 2],
                    points: vec![
                        corner(lo[0] - pad, lo[1] - pad),
                        corner(lo[0] - pad, hi[1] + pad),
                        corner(hi[0] + pad, lo[1] - pad),
                        corner(hi[0] + pad, hi[1] + pad),
                    ],
                    weights: vec![1.0; 4],
                    knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
                };
                let mut loops = vec![vec![ring(i)]];
                loops.extend(hole_ids.into_iter().map(|j| vec![ring(j)]));
                out.push(Geo::Trimmed {
                    surface: Box::new(sheet),
                    loops,
                    display: Some(Box::new(display)),
                });
            }
            let k = out.len();
            Ok((out, format!("{k} planar surfaces")))
        },
    )
}

/// Rhino 5 Plane: the rectangle is drawn as the Rectangle command draws it (`First corner of
/// plane ( Deformable 3Point Vertical Center )`, other corner or a typed length then width),
/// and becomes a planar surface. Deformable: degree and control point counts in u and v
/// (UDegree, VDegree, UPointCount, VPointCount; the surface stays the same plane).
#[derive(Serialize, Deserialize)]
struct PlaneState {
    rect: Value,
    deformable: bool,
    degree: [usize; 2],
    count: [usize; 2],
}

impl Default for PlaneState {
    fn default() -> Self {
        PlaneState { rect: Value::Null, deformable: false, degree: [3, 3], count: [4, 4] }
    }
}

fn plane_surface(corners: &[Vec3], st: &PlaneState) -> Option<Geo> {
    let [a, b, c, d] = [corners.first()?, corners.get(1)?, corners.get(2)?, corners.get(3)?];
    let (degree, count) = if st.deformable {
        let pu = st.degree[0].clamp(1, 11);
        let pv = st.degree[1].clamp(1, 11);
        ([pu, pv], [st.count[0].max(pu + 1), st.count[1].max(pv + 1)])
    } else {
        ([1, 1], [2, 2])
    };
    // Poles at the Greville points of uniform clamped knots: a bilinear sheet is exact.
    let knots = |n: usize, p: usize| {
        let k = om9_addon_geom::clamped_uniform(n, p);
        let last = *k.last().unwrap();
        k.iter().map(|x| x / last).collect::<Vec<f64>>()
    };
    let (ku, kv) = (knots(count[0], degree[0]), knots(count[1], degree[1]));
    let greville = |k: &[f64], n: usize, p: usize| -> Vec<f64> {
        (0..n).map(|i| k[i + 1..=i + p].iter().sum::<f64>() / p as f64).collect()
    };
    let (gu, gv) = (greville(&ku, count[0], degree[0]), greville(&kv, count[1], degree[1]));
    let lerp = |p: Vec3, q: Vec3, t: f64| add(p, scale(sub(q, p), t));
    let mut points = Vec::with_capacity(count[0] * count[1]);
    for u in &gu {
        for v in &gv {
            points.push(lerp(lerp(*a, *b, *u), lerp(*d, *c, *u), *v));
        }
    }
    Some(Geo::Surface {
        degree,
        size: count,
        weights: vec![1.0; points.len()],
        points,
        knots: [ku, kv],
    })
}

fn plane(e: &Event) -> Reply {
    let mut st: PlaneState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    let ours = ["Deformable", "UDegree", "VDegree", "UPointCount", "VPointCount"];
    let tokens = |st: &PlaneState| {
        let mut t = vec![OptionToken::toggle("Deformable", st.deformable)];
        if st.deformable {
            t.push(OptionToken::number("UDegree", Some(st.degree[0] as f64)));
            t.push(OptionToken::number("VDegree", Some(st.degree[1] as f64)));
            t.push(OptionToken::number("UPointCount", Some(st.count[0] as f64)));
            t.push(OptionToken::number("VPointCount", Some(st.count[1] as f64)));
        }
        t
    };
    let mut refuse = None;
    let mut inner = e.clone();
    inner.state = st.rect.clone();
    if let Input::Option { name, value } = &e.input {
        if ours.contains(&name.as_str()) {
            let num = value.as_deref().and_then(om9_addon_abi::session::parse_number);
            match name.as_str() {
                "Deformable" => st.deformable = match value.as_deref() {
                    Some(v) => v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("true"),
                    None => !st.deformable,
                },
                _ => match num.filter(|v| v.fract() == 0.0 && *v >= 1.0 && *v <= 100.0) {
                    Some(v) => {
                        let v = v as usize;
                        match name.as_str() {
                            "UDegree" => st.degree[0] = v.min(11),
                            "VDegree" => st.degree[1] = v.min(11),
                            "UPointCount" => st.count[0] = v.max(2),
                            _ => st.count[1] = v.max(2),
                        }
                    }
                    None => refuse = Some(format!("{name}: a whole number from 1 to 100")),
                },
            }
            // Ask again from where the rectangle is (a hover replays its prompt).
            inner.input = Input::Start;
        } else if name == "Rounded" || name == "Corner" || name == "Rho" {
            refuse = Some(format!("Unknown option {name}"));
            inner.input = Input::Start;
        }
    }
    let mut r = om9_curves::rect::step(&inner);
    st.rect = r.state.clone();
    // Rectangles become planes, previews and results alike.
    let corners = |g: &Geo| match g {
        Geo::Polyline { points, .. } if points.len() == 4 => Some(points.clone()),
        _ => None,
    };
    r.preview = r.preview.iter().filter_map(|g| plane_surface(&corners(g)?, &st)).collect();
    r.create = r.create.iter().filter_map(|g| plane_surface(&corners(g)?, &st)).collect();
    if r.done && !r.create.is_empty() {
        for m in &mut r.messages {
            *m = m.replacen("Rectangle", "Plane", 1);
        }
    }
    r.prompt = r.prompt.replace("First corner of rectangle", "First corner of plane");
    r.options.retain(|o| o.name != "Rounded");
    let mut opts = tokens(&st);
    opts.extend(r.options.drain(..));
    r.options = opts;
    r.state = serde_json::to_value(&st).unwrap_or(Value::Null);
    if let Some(why) = refuse {
        r.error = Some(why);
    }
    r
}

// ---- revolve ---------------------------------------------------------------------------------

/// The NURBS of a curve: exact when it is one, else fitted through its samples.
fn curve_nurbs(s: &Selected) -> Option<(usize, Vec<Vec3>, Vec<f64>, Vec<f64>)> {
    // Exact for every curve kind the host describes (lines, polylines, circles, arcs, NURBS).
    if let Some(c) = exact_curve(s) {
        return Some((
            c.degree,
            c.points.iter().map(|p| exact::euclid(*p)).collect(),
            c.points.iter().map(|p| p[3]).collect(),
            c.knots,
        ));
    }
    match &s.exact {
        Some(Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        }) => {
            let w = if weights.is_empty() {
                vec![1.0; points.len()]
            } else {
                weights.clone()
            };
            Some((*degree, points.clone(), w, knots.clone()))
        }
        Some(Geo::Line { start, end }) => Some((
            1,
            vec![*start, *end],
            vec![1.0; 2],
            vec![0.0, 0.0, 1.0, 1.0],
        )),
        _ => match interpolate(&dense(s), 3, s.closed)? {
            Geo::Nurbs {
                degree,
                points,
                knots,
                ..
            } => Some((degree, points.clone(), vec![1.0; points.len()], knots)),
            Geo::Polyline { points, .. } => {
                let n = points.len();
                let k = om9_addon_geom::clamped_uniform(n, 1);
                let last = *k.last().unwrap();
                Some((
                    1,
                    points,
                    vec![1.0; n],
                    k.iter().map(|v| v / last).collect(),
                ))
            }
            _ => None,
        },
    }
}

/// An exact surface of revolution: the profile's NURBS swept by rational quadratic arcs.
pub fn revolve_surface(
    profile: (usize, Vec<Vec3>, Vec<f64>, Vec<f64>),
    axis_o: Vec3,
    axis_d: Vec3,
    start: f64,
    angle: f64,
) -> Geo {
    let (pv, poles, wv, kv) = profile;
    let arcs = (angle.abs() / FRAC_PI_2 - 1e-9).ceil().max(1.0) as usize;
    let d = angle / arcs as f64;
    let wm = (d / 2.0).cos();
    let nu = 2 * arcs + 1;
    let mut ku = vec![0.0; 3];
    for k in 1..arcs {
        let t = k as f64 / arcs as f64;
        ku.push(t);
        ku.push(t);
    }
    ku.extend([1.0; 3]);
    let a = unit(axis_d);
    let rot = |p: Vec3, th: f64, stretch: f64| -> Vec3 {
        let c = add(axis_o, scale(a, dot(sub(p, axis_o), a)));
        let r = sub(p, c);
        let y = cross(a, r);
        add(
            c,
            scale(add(scale(r, th.cos()), scale(y, th.sin())), stretch),
        )
    };
    let nv = poles.len();
    let mut points = Vec::with_capacity(nu * nv);
    let mut weights = Vec::with_capacity(nu * nv);
    for i in 0..nu {
        let th = start + d * i as f64 / 2.0;
        let mid = i % 2 == 1;
        for (j, p) in poles.iter().enumerate() {
            points.push(rot(*p, th, if mid { 1.0 / wm } else { 1.0 }));
            weights.push(wv[j] * if mid { wm } else { 1.0 });
        }
    }
    Geo::Surface {
        degree: [2, pv],
        size: [nu, nv],
        points,
        weights,
        knots: [ku, kv],
    }
}

/// Rhino 5 Revolve: curves, start of the axis, end of the axis, then the revolution angle
/// (default 360; the pointer sets it, FullCircle makes 360). Options as on the Rhino 5
/// command line: StartAngle, FullCircle, Deformable with PointCount (a non-rational degree 3
/// surface), DeleteInput. Exact (rational) otherwise.
fn revolve(e: &Event) -> Reply {
    let flow = Flow::new("Revolve")
        .select_prompt("Select curves to revolve")
        .point("Start of revolve axis")
        .point("End of revolve axis")
        .value("Revolution angle", e.number("rev_angle", 360.0))
        .num("StartAngle", e.number("rev_start", 0.0))
        .toggle("FullCircle", false)
        .toggle("Deformable", false)
        .num("PointCount", 8.0)
        .toggle("SplitAtTangents", e.flag("rev_split_at_tangents", false))
        .toggle("DeleteInput", false);
    let profiles: Vec<(String, (usize, Vec<Vec3>, Vec<f64>, Vec<f64>))> = curves(e)
        .iter()
        .filter_map(|s| Some((s.id.clone(), curve_nurbs(s)?)))
        .collect();
    // The angle the pointer shows around the axis, from the profile's own side.
    let pointer_angle = |a: Vec3, b: Vec3, at: Vec3| -> Option<f64> {
        let ax = unit(sub(b, a));
        let radial = |p: Vec3| {
            let r = sub(p, a);
            sub(r, scale(ax, dot(r, ax)))
        };
        let (_, (_, poles, _, _)) = profiles.first()?;
        let far = poles.iter().copied().max_by(|p, q| norm(radial(*p)).total_cmp(&norm(radial(*q))))?;
        let (r0, r1) = (radial(far), radial(at));
        if norm(r0) < 1e-12 || norm(r1) < 1e-12 {
            return None;
        }
        let (x, y) = (unit(r0), unit(cross(ax, r0)));
        let t = dot(r1, y).atan2(dot(r1, x)).to_degrees();
        Some(if t <= 1e-9 { t + 360.0 } else { t })
    };
    let make = |c: &Collected, angle_deg: f64| -> Result<Vec<Geo>, String> {
        let (Some(a), Some(b)) = (c.p(0), c.p(1)) else {
            return Err("pick the axis".into());
        };
        if dist(a, b) < 1e-9 {
            return Err("the axis needs two different points".into());
        }
        if profiles.is_empty() {
            return Err("select the profile curves".into());
        }
        let angle = if c.yes(&flow, "FullCircle") { 360.0 } else { angle_deg };
        if !(angle.abs() > 1e-9 && angle.abs() <= 360.0 + 1e-9) {
            return Err("the revolution angle must be more than 0 and at most 360".into());
        }
        let start = c.num(&flow, "StartAngle").to_radians();
        let mut out: Vec<Geo> = profiles
            .iter()
            .map(|(_, p)| revolve_surface(p.clone(), a, sub(b, a), start, angle.to_radians()))
            .collect();
        // SplitAtTangents: a surface per smooth stretch of a kinked profile (v runs along it).
        if c.yes(&flow, "SplitAtTangents") && !c.yes(&flow, "Deformable") {
            out = out.iter().flat_map(|g| om9_addon_geom::surfedit::split_at_kinks(g, 1)).collect();
        }
        if c.yes(&flow, "Deformable") {
            let count = (c.num(&flow, "PointCount").round() as usize).max(4);
            out = out
                .iter()
                .filter_map(|g| {
                    let s = Srf::of(g)?;
                    // Like the exact surface: u around (`count` points), v along the profile;
                    // degree 3 both ways, not rational.
                    let nv = 4.max(match g { Geo::Surface { size, .. } => size[1] * 2, _ => 4 });
                    let rows: Vec<Vec<Vec3>> = (0..count)
                        .map(|i| (0..nv).map(|j| s.at_st(i as f64 / (count - 1) as f64, j as f64 / (nv - 1) as f64)).collect())
                        .collect();
                    surface_through(&rows, [3, 3])
                })
                .collect();
        }
        Ok(out)
    };
    flow.step(
        e,
        &|c, h| {
            // At the angle step the pointer sets the angle; before, a full turn previews.
            let angle = match (c.p(1), h) {
                (Some(b), Some(at)) => c.p(0).and_then(|a| pointer_angle(a, b, at)).unwrap_or(360.0),
                _ => 360.0,
            };
            let mut c2 = c.clone();
            if c2.p(1).is_none() {
                if let Some(at) = h {
                    c2.answers.push(om9_addon_geom::cmd::Answer::Point(at));
                }
            }
            Show {
                geo: make(&c2, angle).unwrap_or_default(),
                ..Default::default()
            }
        },
        &|c| {
            // Typed (or Enter: the default), or clicked: the angle the click shows.
            let angle = match (c.p(2), c.p(0), c.p(1)) {
                (Some(at), Some(a), Some(b)) => pointer_angle(a, b, at).ok_or("pick away from the axis")?,
                _ => c.n(0).unwrap_or(flow_default(&flow, "Revolution angle", 360.0)),
            };
            let out = make(c, angle)?;
            let n = out.len();
            let shown = if c.yes(&flow, "FullCircle") { 360.0 } else { angle };
            let mut r = Reply::finish(out).say(format!("Revolve: {n} surfaces, {}°", fmt(shown)));
            if c.yes(&flow, "DeleteInput") {
                r.delete = profiles.iter().map(|(id, _)| id.clone()).collect();
            }
            Ok(r)
        },
    )
}

/// The default of a number step (as the flow was built with it).
fn flow_default(flow: &Flow, prompt: &str, fallback: f64) -> f64 {
    flow.needs
        .iter()
        .find_map(|n| match n {
            om9_addon_geom::cmd::Need::Number(p, d) | om9_addon_geom::cmd::Need::Value(p, d)
                if p == prompt =>
            {
                Some(*d)
            }
            _ => None,
        })
        .unwrap_or(fallback)
}

fn curve_nc(c: &om9_addon_geom::pick::CurveRef) -> Option<Nc> {
    if let Some(g) = &c.exact
        && let Some(n) = Nc::from_geo(g)
    {
        return Some(n);
    }
    Nc::from_geo(&interpolate(&c.pts, 3, c.closed)?)
}

/// Rhino 5 RailRevolve: profile, rail, axis start, axis end; ScaleHeight. Exact.
fn rail_revolve(e: &Event) -> Reply {
    let flow = Flow::new("Rail Revolve")
        .curve("Select profile curve")
        .curve("Select rail curve")
        .point("Start of RailRevolve axis")
        .point("End of RailRevolve axis")
        .toggle("ScaleHeight", e.flag("rr_scale_height", false));
    let make = |c: &Collected, end: Option<Vec3>| -> Result<Geo, String> {
        let (Some((prof, _)), Some((rail, _))) = (c.curve(0), c.curve(1)) else {
            return Err("select the profile, then the rail".into());
        };
        let (Some(o), Some(b)) = (c.p(0), c.p(1).or(end)) else {
            return Err("pick the axis".into());
        };
        let prof = curve_nc(prof).ok_or("the profile has no curve")?;
        let rail = curve_nc(rail).ok_or("the rail has no curve")?;
        Ok(sweep::rail_revolve(&prof, &rail, o, b, c.yes(&flow, "ScaleHeight"))?.geo())
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).into_iter().collect(),
            ..Default::default()
        },
        &|c| Ok(Reply::finish(vec![make(c, None)?]).say("Rail Revolve: exact")),
    )
}

/// Rhino 5 ExtrudeCrv (Matrix "Extrude All"): curves, then `Extrusion distance <d>` along
/// the CPlane normal: typed, Enter for the default, or a click (its height over the first
/// curve's start; the preview follows the pointer). BothSides, DeleteInput. Exact: each
/// surface is the curve's own NURBS and its copy moved (degree 1 across).
fn extrude_all(e: &Event) -> Reply {
    let flow = Flow::new("Extrude All")
        .select_prompt("Select curves to extrude")
        .value("Extrusion distance", e.number("ea_distance", 2.0))
        .toggle("BothSides", e.flag("ea_both_sides", false))
        .toggle("DeleteInput", false);
    let n = e.plane.normal();
    let picked = curves(e);
    let base = picked.first().and_then(|c| c.points.first().copied());
    let height = |at: Vec3| base.map(|b| dot(sub(at, b), n));
    let make = |d: f64, both: bool| -> Result<Vec<Geo>, String> {
        if picked.is_empty() {
            return Err("select curves to extrude".into());
        }
        if !(d.abs() > 1e-9 && d.is_finite()) {
            return Err("the extrusion distance must not be 0".into());
        }
        let lo = if both { scale(n, -d) } else { [0.0; 3] };
        let hi = scale(n, d);
        Ok(picked
            .iter()
            .filter_map(|s| {
                let c = exact_curve(s).or_else(|| {
                    let mut q = dense(s);
                    if s.closed && dist(q[0], *q.last()?) > 1e-9 {
                        q.push(q[0]);
                    }
                    let g = interpolate(&q, 3, false)?;
                    Nc::from_geo(&g)
                })?;
                let shift = |v: Vec3| -> Vec<om9_addon_geom::exact::H> {
                    c.points
                        .iter()
                        .map(|p| [p[0] + v[0] * p[3], p[1] + v[1] * p[3], p[2] + v[2] * p[3], p[3]])
                        .collect()
                };
                let rows = vec![shift(lo), shift(hi)];
                Some(exact::surface(&rows, [1, c.degree], [vec![0.0, 0.0, 1.0, 1.0], c.knots.clone()]))
            })
            .collect())
    };
    flow.step(
        e,
        &|c, h| {
            let d = match h.and_then(height) {
                Some(d) if c.answers.is_empty() && d.abs() > 1e-9 => d,
                _ => flow_default(&flow, "Extrusion distance", 2.0),
            };
            Show {
                geo: make(d, c.yes(&flow, "BothSides")).unwrap_or_default(),
                ..Default::default()
            }
        },
        &|c| {
            let d = match c.p(0) {
                Some(at) => height(at).ok_or("select curves to extrude")?,
                None => c.n(0).unwrap_or(flow_default(&flow, "Extrusion distance", 2.0)),
            };
            let out = make(d, c.yes(&flow, "BothSides"))?;
            let k = out.len();
            let mut r = Reply::finish(out).say(format!("Extrude All: {k} surfaces, {} mm", fmt(d)));
            if c.yes(&flow, "DeleteInput") {
                r.delete = picked.iter().map(|s| s.id.clone()).collect();
            }
            Ok(r)
        },
    )
}

// ---- blends, fillets, chamfers ---------------------------------------------------------------

/// The edge of a surface (s or t at 0 or 1) nearest to `p`: (dir, at).
fn nearest_edge(s: &Srf, p: Vec3) -> (usize, f64) {
    let mut best = (f64::MAX, 0, 0.0);
    for (dir, at) in [(0, 0.0), (0, 1.0), (1, 0.0), (1, 1.0)] {
        let d = (0..=16)
            .map(|k| {
                let t = k as f64 / 16.0;
                if dir == 0 {
                    s.at_st(at, t)
                } else {
                    s.at_st(t, at)
                }
            })
            .map(|q| dist(q, p))
            .fold(f64::MAX, f64::min);
        if d < best.0 {
            best = (d, dir, at);
        }
    }
    (best.1, best.2)
}

/// Points along an edge and the cross-boundary direction (outward) there.
fn edge_rows(s: &Srf, dir: usize, at: f64, n: usize) -> (Vec<Vec3>, Vec<Vec3>) {
    let h = 1e-3;
    let inner = if at > 0.5 { at - h } else { at + h };
    (0..n)
        .map(|k| {
            let t = k as f64 / (n - 1) as f64;
            let (p, q) = if dir == 0 {
                (s.at_st(at, t), s.at_st(inner, t))
            } else {
                (s.at_st(t, at), s.at_st(t, inner))
            };
            (p, unit(sub(p, q)))
        })
        .unzip()
}

/// Two surface edges picked by clicks near them (Rhino's edge picks): surfaces picked before
/// the command give their facing edges. `Err` is the prompt to show; `Ok(true)` when the
/// second edge was picked by this very input.
fn pick_two_edges(e: &Event, edges: &mut Vec<(String, usize, bool)>, prompts: [&str; 2]) -> Result<bool, (String, Want)> {
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    let mut entering = false;
    // Two surfaces picked before the command: their facing edges (nearest edge middles).
    if (matches!(e.input, Input::Start) || e.state.is_null()) && surfaces.len() == 2 {
        let fa = Srf::of(surfaces[0].exact.as_ref().unwrap());
        let fb = Srf::of(surfaces[1].exact.as_ref().unwrap());
        if let (Some(fa), Some(fb)) = (fa, fb) {
            let mid = |f: &Srf, dir: usize, at: f64| if dir == 0 { f.at_st(at, 0.5) } else { f.at_st(0.5, at) };
            let mut best = (f64::MAX, (0, 0.0), (0, 0.0));
            for ea in [(0, 0.0), (0, 1.0), (1, 0.0), (1, 1.0)] {
                for eb in [(0, 0.0), (0, 1.0), (1, 0.0), (1, 1.0)] {
                    let d = dist(mid(&fa, ea.0, ea.1), mid(&fb, eb.0, eb.1));
                    if d < best.0 {
                        best = (d, ea, eb);
                    }
                }
            }
            *edges = vec![
                (surfaces[0].id.clone(), best.1.0, best.1.1 > 0.5),
                (surfaces[1].id.clone(), best.2.0, best.2.1 > 0.5),
            ];
        }
    }
    if edges.len() < 2 {
        // The edge nearest the click: on the surface just picked, or on any selected one.
        let at = match &e.input {
            Input::Point { point } => Some((*point, None)),
            Input::Selection => e.pick.map(|p| (p, surfaces.last().map(|s| s.id.clone()))),
            _ => None,
        };
        if let Some((p, only)) = at {
            let mut best: Option<(f64, String, usize, bool)> = None;
            for s in &surfaces {
                if only.as_ref().is_some_and(|id| id != &s.id) {
                    continue;
                }
                let Some(f) = Srf::of(s.exact.as_ref().unwrap()) else { continue };
                let (dir, at) = nearest_edge(&f, p);
                let q = f.closest_st(p);
                let d = dist(p, f.at_st(if dir == 0 { at } else { q.0 }, if dir == 0 { q.1 } else { at }));
                if best.as_ref().is_none_or(|b| d < b.0) {
                    best = Some((d, s.id.clone(), dir, at > 0.5));
                }
            }
            if let Some((_, id, dir, end)) = best {
                edges.push((id, dir, end));
                entering = edges.len() == 2;
            }
        }
        if edges.len() < 2 {
            let prompt = if edges.is_empty() { prompts[0] } else { prompts[1] };
            let want = if surfaces.is_empty() { Want::Objects } else { Want::Point };
            return Err((prompt.to_owned(), want));
        }
    }
    Ok(entering)
}

/// VariableOffsetSrf command state.
#[derive(Serialize, Deserialize, Default, Clone)]
struct VarOffsetState {
    /// Handles: place on the surface (normalised u, v) and distance.
    handles: Vec<((f64, f64), f64)>,
    mode: Option<String>,
    editing: usize,
    flip: Option<bool>,
}

/// Rhino 5 VariableOffsetSrf: Select surface, then the handles — one at each corner; click
/// near one to type its distance, AddHandle for one at a point, SetAll, Flip — with a
/// preview; Enter creates.
fn variable_offset(e: &Event) -> Reply {
    let label = "Variable Offset Surface";
    let mut st: VarOffsetState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = VarOffsetState::default();
    }
    let save = |st: &VarOffsetState| serde_json::to_value(st).unwrap_or(Value::Null);
    let ss = nurbs_surfaces(e);
    let Some((sel, f)) = ss.first() else {
        if !meshes(e).is_empty() {
            return offset(e);
        }
        return Reply::ask("Select surface to offset", save(&st)).want(Want::Objects);
    };
    if st.handles.is_empty() {
        let d = e.number("os_distance", 1.0);
        let c = [d, e.number("os_d10", d), e.number("os_d01", d), e.number("os_d11", d)];
        st.handles = vec![((0.0, 0.0), c[0]), ((1.0, 0.0), c[1]), ((0.0, 1.0), c[2]), ((1.0, 1.0), c[3])];
    }
    let flip = st.flip.unwrap_or_else(|| e.flag("os_flip", false));
    let mut error = None;
    match &e.input {
        Input::Option { name, value } if name.eq_ignore_ascii_case("Flip") => {
            st.flip = Some(match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("yes"),
                None => !flip,
            })
        }
        Input::Option { name, .. } if name.eq_ignore_ascii_case("SetAll") => st.mode = Some("SetAll".into()),
        Input::Option { name, .. } if name.eq_ignore_ascii_case("AddHandle") => st.mode = Some("AddHandle".into()),
        Input::Point { point } => {
            let (a, b) = f.closest_st(*point);
            if st.mode.as_deref() == Some("AddHandle") {
                let now = st.handles.iter().map(|h| h.1).sum::<f64>() / st.handles.len() as f64;
                st.handles.push(((a, b), now));
                st.editing = st.handles.len() - 1;
            } else {
                st.editing = (0..st.handles.len())
                    .min_by(|x, y| {
                        let d = |i: usize| (st.handles[i].0.0 - a).powi(2) + (st.handles[i].0.1 - b).powi(2);
                        d(*x).total_cmp(&d(*y))
                    })
                    .unwrap_or(0);
            }
            st.mode = Some("Edit".into());
        }
        Input::Text { text } if st.mode.is_some() => match text.trim().replace(',', ".").parse::<f64>() {
            Ok(v) => {
                if st.mode.as_deref() == Some("SetAll") {
                    st.handles.iter_mut().for_each(|h| h.1 = v);
                } else {
                    let i = st.editing.min(st.handles.len() - 1);
                    st.handles[i].1 = v;
                }
                st.mode = None;
            }
            Err(_) => error = Some(format!("{text}: type a number")),
        },
        Input::Enter if st.mode.is_some() => st.mode = None,
        _ => {}
    }
    let flip = st.flip.unwrap_or(flip);
    let tol = match e.number("os_tolerance", 0.0) {
        t if t > 0.0 => t,
        _ => e.number("sw_tolerance", 0.001).max(1e-6),
    };
    let make = || {
        let h: Vec<((f64, f64), f64)> = st.handles.iter().map(|(p, d)| (*p, if flip { -d } else { *d })).collect();
        om9_addon_geom::offset::variable_offset(sel.exact.as_ref().unwrap(), &h, tol)
    };
    if matches!(e.input, Input::Enter) && st.mode.is_none() {
        return match make() {
            Ok(o) => Reply::finish(vec![o.geo]).say(format!("{label}: {} handles, within {} mm", st.handles.len(), fmt(tol))),
            Err(why) => Reply::ask("Select edge point to adjust. Press Enter when done", save(&st)).want(Want::Point).refuse(format!("{label}: {why}")),
        };
    }
    let prompt = match st.mode.as_deref() {
        Some("SetAll") => "Distance for all handles".to_owned(),
        Some("AddHandle") => "Point on the surface for the new handle".to_owned(),
        Some(_) => format!("Distance <{}>", fmt(st.handles[st.editing.min(st.handles.len() - 1)].1)),
        None => "Select edge point to adjust. Press Enter when done".to_owned(),
    };
    let want = if matches!(st.mode.as_deref(), Some("SetAll") | Some("Edit")) { Want::Number } else { Want::Point };
    let mut r = Reply::ask(prompt, save(&st)).want(want).with_options(vec![
        OptionToken::action("SetAll"),
        OptionToken::action("AddHandle"),
        OptionToken::toggle("Flip", flip),
    ]);
    match make() {
        Ok(o) => r = r.preview(vec![o.geo]),
        Err(why) => error = error.or(Some(why)),
    }
    match error {
        Some(why) => r.refuse(format!("{label}: {why}")),
        None => r,
    }
}

/// VariableFilletSrf / VariableChamferSrf command state.
#[derive(Serialize, Deserialize, Default, Clone)]
struct VarFilletState {
    first: Option<(String, Vec3)>,
    second: Option<(String, Vec3)>,
    /// Handles: place along the edge (0..1), radius or first distance, second distance.
    handles: Vec<(f64, f64, f64)>,
    /// What the next input sets: "SetAll", "AddHandle", "AddValue", or "Edit".
    mode: Option<String>,
    /// The handle being edited.
    editing: usize,
    trim: Option<bool>,
    /// RailType (VariableFilletSrf): RollingBall, DistFromEdge or DistBetweenRails.
    #[serde(default)]
    rail: Option<String>,
    /// LinkHandles: a value set on one handle is set on all.
    #[serde(default)]
    link: bool,
    /// FromTwoPoints: the first point picked (the value is the distance to the second).
    #[serde(default)]
    from_two: Option<Vec<Vec3>>,
}

/// Rhino 5 VariableFilletSrf / VariableChamferSrf: the two surfaces (clicks on the sides
/// kept), then the handles — one at each end of the edge; click near one to type its
/// value, AddHandle to add one at a point, SetAll for all — with a preview; Enter creates.
fn variable_fillet(e: &Event) -> Reply {
    use om9_addon_geom::fillet;
    let chamfer = e.command == VAR_CHAMFER;
    let blending = e.command == VAR_BLEND;
    let label = if chamfer {
        "Variable Chamfer Surfaces"
    } else if blending {
        "Variable Blend Surfaces"
    } else {
        "Variable Fillet Surfaces"
    };
    let noun = if chamfer { "chamfer" } else if blending { "blend" } else { "fillet" };
    let mut st: VarFilletState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = VarFilletState::default();
    }
    let save = |st: &VarFilletState| serde_json::to_value(st).unwrap_or(Value::Null);
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    // Meshes (no exact surface): the mesh fillet / chamfer / blend with start and end values.
    if surfaces.is_empty() && !meshes(e).is_empty() && st.first.is_none() {
        return if blending { blend(e) } else { fillet(e) };
    }
    let center = |s: &Selected| Srf::of(s.exact.as_ref().unwrap()).map_or(s.bbox[0], |f| f.at_st(0.5, 0.5));
    let trim = st.trim.unwrap_or_else(|| e.flag("fs_trim", true));
    // 1. The surfaces (preselected ones: their middles as the kept sides).
    if st.second.is_none() {
        let started = matches!(e.input, Input::Start);
        for s in &surfaces {
            let at = if started { center(s) } else { e.pick.unwrap_or_else(|| center(s)) };
            if st.first.is_none() {
                st.first = Some((s.id.clone(), at));
            } else if st.first.as_ref().is_some_and(|f| f.0 != s.id) && st.second.is_none() {
                st.second = Some((s.id.clone(), at));
            }
        }
        if st.second.is_none() {
            let prompt = if st.first.is_none() { format!("Select first surface to {noun}") } else { format!("Select second surface to {noun}") };
            return Reply::ask(prompt, save(&st)).want(Want::Objects);
        }
    }
    let find = |id: &str| surfaces.iter().find(|s| s.id == id).copied();
    let ((ia, k1), (ib, k2)) = (st.first.clone().unwrap(), st.second.clone().unwrap());
    let (Some(a), Some(b)) = (find(&ia), find(&ib)) else {
        return Reply::ask(format!("Select first surface to {noun}"), save(&VarFilletState::default())).want(Want::Objects);
    };
    let (g1, g2) = (a.exact.as_ref().unwrap(), b.exact.as_ref().unwrap());
    if st.handles.is_empty() {
        let (r0, r1) = if chamfer {
            (e.number("cs_distance", 1.0), e.number("cs_end_distance", e.number("cs_distance", 1.0)))
        } else {
            (e.number("fs_radius", 1.0), e.number("fs_end_radius", 2.0))
        };
        st.handles = vec![(0.0, r0, r0), (1.0, r1, r1)];
    }
    // 2. The handles.
    let values = |t: &str| -> Option<(f64, f64)> {
        let v: Vec<f64> = t.split([',', ' ', ';']).filter(|x| !x.is_empty()).filter_map(|x| x.parse::<f64>().ok()).collect();
        match v.as_slice() {
            [x] if *x > 0.0 => Some((*x, *x)),
            [x, y] if *x > 0.0 && *y > 0.0 => Some((*x, *y)),
            _ => None,
        }
    };
    let mut error = None;
    let rail = st.rail.clone().unwrap_or_else(|| e.text("vf_rail_type", "RollingBall").to_owned());
    // A value (typed, or the distance between two points) for the handle being set.
    let apply = |st: &mut VarFilletState, x: f64, y: f64| {
        if st.mode.as_deref() == Some("SetAll") || st.link {
            st.handles.iter_mut().for_each(|h| (h.1, h.2) = (x, y));
        } else {
            let i = st.editing.min(st.handles.len() - 1);
            let h = &mut st.handles[i];
            (h.1, h.2) = (x, y);
        }
        st.mode = None;
        st.from_two = None;
    };
    let nearest = |st: &VarFilletState, place: f64| {
        (0..st.handles.len()).min_by(|x, y| (st.handles[*x].0 - place).abs().total_cmp(&(st.handles[*y].0 - place).abs())).unwrap_or(0)
    };
    match &e.input {
        Input::Option { name, value } if name.eq_ignore_ascii_case("RailType") && !chamfer && !blending => {
            st.rail = Some(value.clone().unwrap_or_else(|| {
                let all = ["RollingBall", "DistFromEdge", "DistBetweenRails"];
                let i = all.iter().position(|r| r.eq_ignore_ascii_case(&rail)).unwrap_or(0);
                all[(i + 1) % 3].to_owned()
            }));
        }
        Input::Option { name, value } if name.eq_ignore_ascii_case("LinkHandles") => {
            st.link = match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("yes"),
                None => !st.link,
            };
        }
        Input::Option { name, .. } if name.eq_ignore_ascii_case("CopyHandle") => st.mode = Some("CopyHandle".into()),
        Input::Option { name, .. } if name.eq_ignore_ascii_case("RemoveHandle") => st.mode = Some("RemoveHandle".into()),
        Input::Option { name, .. } if name.eq_ignore_ascii_case("FromTwoPoints") && matches!(st.mode.as_deref(), Some("SetAll") | Some("Edit")) => {
            st.from_two = Some(Vec::new());
        }
        Input::Point { point } if st.from_two.is_some() => {
            let mut pts = st.from_two.take().unwrap_or_default();
            pts.push(*point);
            if pts.len() == 2 {
                let d = dist(pts[0], pts[1]);
                if d > 0.0 {
                    apply(&mut st, d, d);
                } else {
                    error = Some("the two points are the same".to_owned());
                }
            } else {
                st.from_two = Some(pts);
            }
        }
        Input::Point { point } if matches!(st.mode.as_deref(), Some("CopyHandle") | Some("CopyTo") | Some("RemoveHandle")) => match fillet::place_on_intersection(g1, g2, *point) {
            Ok(place) => match st.mode.as_deref() {
                Some("CopyHandle") => {
                    st.editing = nearest(&st, place);
                    st.mode = Some("CopyTo".into());
                }
                Some("CopyTo") => {
                    let src = st.handles[st.editing.min(st.handles.len() - 1)];
                    st.handles.push((place, src.1, src.2));
                    st.editing = st.handles.len() - 1;
                    st.mode = None;
                }
                _ => {
                    let i = nearest(&st, place);
                    // The end handles of an open edge stay (as in Rhino).
                    if st.handles[i].0 <= 1e-9 || st.handles[i].0 >= 1.0 - 1e-9 {
                        error = Some("the end handles cannot be removed".to_owned());
                    } else {
                        st.handles.remove(i);
                        st.editing = 0;
                    }
                    st.mode = None;
                }
            },
            Err(why) => error = Some(why),
        },
        Input::Option { name, value } if name.eq_ignore_ascii_case("Trim") => {
            st.trim = Some(match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("yes"),
                None => !trim,
            });
        }
        Input::Option { name, .. } if name.eq_ignore_ascii_case("SetAll") => st.mode = Some("SetAll".into()),
        Input::Option { name, .. } if name.eq_ignore_ascii_case("AddHandle") => st.mode = Some("AddHandle".into()),
        Input::Point { point } => match fillet::place_on_intersection(g1, g2, *point) {
            Ok(place) => {
                if st.mode.as_deref() == Some("AddHandle") {
                    let (v1, v2) = (fillet_profile(&st.handles, place, 1), fillet_profile(&st.handles, place, 2));
                    st.handles.push((place, v1, v2));
                    st.editing = st.handles.len() - 1;
                } else {
                    st.editing = (0..st.handles.len())
                        .min_by(|x, y| (st.handles[*x].0 - place).abs().total_cmp(&(st.handles[*y].0 - place).abs()))
                        .unwrap_or(0);
                }
                st.mode = Some("Edit".into());
            }
            Err(why) => error = Some(why),
        },
        Input::Text { text } if st.mode.is_some() => match values(text) {
            Some((x, y)) => apply(&mut st, x, y),
            None => error = Some(format!("{text}: type a number more than 0{}", if chamfer { " (or two: first,second)" } else { "" })),
        },
        Input::Enter if st.mode.is_some() => {
            st.mode = None;
            st.from_two = None;
        }
        _ => {}
    }
    let rail = st.rail.clone().unwrap_or(rail);
    let rail_kind = match rail.to_ascii_lowercase().as_str() {
        "distfromedge" => fillet::Rail::DistFromEdge,
        "distbetweenrails" => fillet::Rail::DistBetweenRails,
        _ => fillet::Rail::RollingBall,
    };
    let tol = e.number("sw_tolerance", 0.001).max(1e-6);
    let make = || -> Result<om9_addon_geom::sweep::Swept, String> {
        let h: Vec<(f64, f64)> = st.handles.iter().map(|h| (h.0, h.1)).collect();
        if chamfer {
            fillet::variable_chamfer(g1, g2, &st.handles, k1, k2, tol)
        } else if blending {
            fillet::variable_blend(g1, g2, &h, k1, k2, e.number("bl_bulge", 1.0).max(0.01), tol)
        } else {
            fillet::variable_fillet_rail(g1, g2, &h, k1, k2, rail_kind, tol)
        }
    };
    let show = |v: &(f64, f64, f64)| if chamfer { format!("{},{}", fmt(v.1), fmt(v.2)) } else { fmt(v.1) };
    if matches!(e.input, Input::Enter) && st.mode.is_none() && error.is_none() && !matches!(e.input, Input::Start) {
        return match make() {
            Ok(f) => {
                let mut r = Reply::finish(vec![f.geo()]).say(format!("{label}: {} handles", st.handles.len()));
                if trim {
                    let edge = |k: usize| Nc {
                        degree: f.degree[0],
                        points: f.rows.iter().map(|row| if k == 0 { row[0] } else { row[row.len() - 1] }).collect(),
                        knots: f.knots[0].clone(),
                    };
                    let mut done = Vec::new();
                    for (id, g, curve, keep) in [(&ia, g1, edge(0), k1), (&ib, g2, edge(1), k2)] {
                        match om9_addon_geom::trimsides::trim_keeping(g, &curve, keep) {
                            Ok(t) => done.push(Replacement { id: id.clone(), geo: t }),
                            Err(why) => r = r.say(format!("{label}: not trimmed: {why}")),
                        }
                    }
                    if done.len() == 2 {
                        r.replace = done;
                    }
                }
                r
            }
            Err(why) => Reply::ask("Select handle to edit. Press Enter when done", save(&st)).want(Want::Point).refuse(format!("{label}: {why}")),
        };
    }
    let what = if chamfer {
        "Distances"
    } else if !blending && rail_kind != fillet::Rail::RollingBall {
        "Distance"
    } else {
        "Radius"
    };
    let prompt = match (st.mode.as_deref(), &st.from_two) {
        (_, Some(p)) if p.is_empty() => "First point".to_owned(),
        (_, Some(_)) => "Second point".to_owned(),
        (Some("SetAll"), _) => format!("{what} for all handles"),
        (Some("AddHandle"), _) => "Point on the edge for the new handle".to_owned(),
        (Some("CopyHandle"), _) => "Select handle to copy".to_owned(),
        (Some("CopyTo"), _) => "Point on the edge for the copy".to_owned(),
        (Some("RemoveHandle"), _) => "Select handle to remove".to_owned(),
        (Some(_), _) => format!("{what} <{}>", show(&st.handles[st.editing.min(st.handles.len() - 1)])),
        (None, _) => "Select handle to edit. Press Enter when done".to_owned(),
    };
    let setting = matches!(st.mode.as_deref(), Some("SetAll") | Some("Edit")) && st.from_two.is_none();
    let want = if setting { Want::Number } else { Want::Point };
    let mut options = Vec::new();
    if setting {
        options.push(OptionToken::action("FromTwoPoints"));
    }
    options.extend([OptionToken::action("SetAll"), OptionToken::action("AddHandle"), OptionToken::action("CopyHandle")]);
    if st.handles.iter().any(|h| h.0 > 1e-9 && h.0 < 1.0 - 1e-9) {
        options.push(OptionToken::action("RemoveHandle"));
    }
    options.push(OptionToken::toggle("LinkHandles", st.link));
    if !chamfer && !blending {
        let shown = match rail_kind {
            fillet::Rail::DistFromEdge => "DistFromEdge",
            fillet::Rail::DistBetweenRails => "DistBetweenRails",
            fillet::Rail::RollingBall => "RollingBall",
        };
        options.push(OptionToken::choice("RailType", shown, &["DistFromEdge", "RollingBall", "DistBetweenRails"]));
    }
    options.push(OptionToken::toggle("Trim", trim));
    let mut r = Reply::ask(prompt, save(&st)).want(want).with_options(options);
    r.rubber_from = st.from_two.as_ref().and_then(|p| p.first().copied());
    match make() {
        Ok(f) => r = r.preview(vec![f.geo()]),
        Err(why) => error = error.or(Some(why)),
    }
    match error {
        Some(why) => r.refuse(format!("{label}: {why}")),
        None => r,
    }
}

/// The value a new handle starts with: the current profile there.
fn fillet_profile(handles: &[(f64, f64, f64)], place: f64, which: usize) -> f64 {
    let mut h: Vec<(f64, f64)> = handles.iter().map(|x| (x.0, if which == 1 { x.1 } else { x.2 })).collect();
    h.sort_by(|a, b| a.0.total_cmp(&b.0));
    for w in h.windows(2) {
        if place <= w[1].0 {
            let t = ((place - w[0].0) / (w[1].0 - w[0].0).max(1e-300)).clamp(0.0, 1.0);
            return w[0].1 + (w[1].1 - w[0].1) * t * t * (3.0 - 2.0 * t);
        }
    }
    h.last().map_or(1.0, |x| x.1)
}

/// BlendSrf command state.
#[derive(Serialize, Deserialize, Default, Clone)]
struct BlendState {
    /// Surface id, edge direction, at the far end.
    edges: Vec<(String, usize, bool)>,
    flow: Value,
}

/// Rhino 5 BlendSrf: Select segment for first edge, Select segment for second edge (a click
/// near each), then the continuity at each end (Position / Tangency / Curvature) and the
/// bulges with a preview; Enter creates.
fn blend_srf(e: &Event) -> Reply {
    use om9_addon_geom::blendsrf::{self, Continuity, Edge};
    let label = "Blend Surface";
    let mut st: BlendState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = BlendState::default();
    }
    let save = |st: &BlendState| serde_json::to_value(st).unwrap_or(Value::Null);
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    let entering = match pick_two_edges(e, &mut st.edges, ["Select segment for first edge", "Select segment for second edge"]) {
        Ok(entering) => entering,
        Err((prompt, want)) => return Reply::ask(prompt, save(&st)).want(want),
    };
    let find = |id: &str| surfaces.iter().find(|s| s.id == id).copied();
    let (Some(a), Some(b)) = (find(&st.edges[0].0), find(&st.edges[1].0)) else {
        return Reply::ask("Select segment for first edge", save(&BlendState::default())).want(Want::Objects);
    };
    let cont = |k: &str| match e.text(k, "curvature") {
        "position" => "Position",
        "tangency" => "Tangency",
        "g3" => "G3",
        "g4" => "G4",
        _ => "Curvature",
    };
    let kinds = ["Position", "Tangency", "Curvature", "G3", "G4"];
    let flow = Flow::new(label)
        .no_selection()
        .plain()
        .confirm("Press Enter to create the blend, or change an option")
        .choice("Continuity1", &kinds, cont("bl_continuity1"))
        .choice("Continuity2", &kinds, cont("bl_continuity2"))
        .num("Bulge1", e.number("bl_bulge", 1.0))
        .num("Bulge2", e.number("bl_bulge", 1.0));
    let to_c = |v: &str| match v {
        "Position" => Continuity::Position,
        "Tangency" => Continuity::Tangency,
        "G3" => Continuity::G3,
        "G4" => Continuity::G4,
        _ => Continuity::Curvature,
    };
    let (ea, eb) = (&st.edges[0], &st.edges[1]);
    let tol = e.number("sw_tolerance", 0.001).max(1e-6);
    let make = |c: &Collected| -> Result<Geo, String> {
        Ok(blendsrf::blend(
            a.exact.as_ref().unwrap(),
            Edge { dir: ea.1, at_end: ea.2 },
            to_c(c.opt(&flow, "Continuity1")),
            c.num(&flow, "Bulge1").max(0.01),
            b.exact.as_ref().unwrap(),
            Edge { dir: eb.1, at_end: eb.2 },
            to_c(c.opt(&flow, "Continuity2")),
            c.num(&flow, "Bulge2").max(0.01),
            tol,
        )?
        .geo())
    };
    let mut sub = e.clone();
    sub.state = st.flow.clone();
    if entering || matches!(sub.input, Input::Selection | Input::Point { .. }) {
        sub.input = Input::Hover { point: [0.0; 3] };
    }
    let failed = std::cell::RefCell::new(None);
    let mut r = flow.step(
        &sub,
        &|c, _| match make(c) {
            Ok(g) => Show { geo: vec![g], ..Default::default() },
            Err(why) => {
                *failed.borrow_mut() = Some(why);
                Show::default()
            }
        },
        &|c| {
            let g = make(c)?;
            Ok(Reply::finish(vec![g]).say(format!("{label}: {} / {}, within {} mm", c.opt(&flow, "Continuity1"), c.opt(&flow, "Continuity2"), fmt(tol))))
        },
    );
    if !r.done {
        st.flow = r.state.clone();
        r.state = save(&st);
        let why = failed.borrow_mut().take();
        if let (None, Some(why)) = (&r.error, why) {
            r = r.refuse(format!("{label}: {why}"));
        }
    }
    r
}

fn blend(e: &Event) -> Reply {
    let var = e.command == VAR_BLEND;
    let b0 = e.number("bl_bulge", 1.0);
    let b1 = if var {
        e.number("bl_end_bulge", 1.0)
    } else {
        b0
    };
    let label = if var {
        "Variable Blend Surface"
    } else {
        "Blend Surface"
    };
    built(
        e,
        label,
        "select two surfaces (their nearest edges are blended), Enter",
        || {
            let ss = nurbs_surfaces(e);
            if ss.len() != 2 {
                return Err("select two NURBS surfaces".into());
            }
            let (a, b) = (&ss[0].1, &ss[1].1);
            let cb = b.at_st(0.5, 0.5);
            let ca = a.at_st(0.5, 0.5);
            let (da, ta) = nearest_edge(a, cb);
            let (db, tb) = nearest_edge(b, ca);
            let n = 24;
            let (pa, na) = edge_rows(a, da, ta, n);
            let (mut pb, mut nb) = edge_rows(b, db, tb, n);
            if dist(pa[0], pb[0]) > dist(pa[0], pb[n - 1]) {
                pb.reverse();
                nb.reverse();
            }
            // Four rows of a cubic in the blend direction: edge, handle, handle, edge.
            let mut rows = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];
            for k in 0..n {
                let t = k as f64 / (n - 1) as f64;
                let bulge = b0 + (b1 - b0) * t;
                let h = dist(pa[k], pb[k]) / 3.0 * bulge;
                rows[0].push(pa[k]);
                rows[1].push(add(pa[k], scale(na[k], h)));
                rows[2].push(add(pb[k], scale(nb[k], h)));
                rows[3].push(pb[k]);
            }
            Ok((
                vec![surface_on_grid(&rows, [3, 3]).ok_or("blend")?],
                "tangent blend between the edges".into(),
            ))
        },
    )
}

/// The nearest point of a mesh and its normal (as the mesh is wound).
fn mesh_point(m: &SelectedMesh, p: Vec3) -> Option<(Vec3, Vec3)> {
    let mut best: Option<(f64, Vec3, Vec3)> = None;
    for t in &m.triangles {
        let [a, b, c] = [
            m.vertices[t[0] as usize],
            m.vertices[t[1] as usize],
            m.vertices[t[2] as usize],
        ];
        let q = om9_addon_geom::sdf::closest_on_triangle(p, a, b, c);
        let d = dist(p, q);
        let n = cross(sub(b, a), sub(c, a));
        if norm(n) > 1e-18 && best.as_ref().is_none_or(|x| d < x.0) {
            best = Some((d, q, unit(n)));
        }
    }
    best.map(|b| (b.1, b.2))
}

/// Solve a 3×3 system `rows · x = rhs`.
fn solve3(r: [Vec3; 3], rhs: Vec3) -> Option<Vec3> {
    let det = dot(r[0], cross(r[1], r[2]));
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = [cross(r[1], r[2]), cross(r[2], r[0]), cross(r[0], r[1])];
    Some(scale(
        add(
            add(scale(inv[0], rhs[0]), scale(inv[1], rhs[1])),
            scale(inv[2], rhs[2]),
        ),
        1.0 / det,
    ))
}

/// Rows of a fillet (arcs) or chamfer (lines) along the intersection of two meshes, radius /
/// distance `r(t)` along it; `flip` picks the side of each surface.
/// FilletSrf / ChamferSrf command state.
#[derive(Serialize, Deserialize, Default, Clone)]
struct FilletState {
    /// First surface and where it was clicked (the side kept).
    first: Option<(String, Vec3)>,
    radius: Option<f64>,
    /// Chamfer distances on the first and second surface.
    distances: Option<[f64; 2]>,
    /// Asking for: 1 the radius or first distance, 2 the second distance.
    want: u8,
    trim: Option<bool>,
    extend: Option<bool>,
    split: Option<bool>,
}

/// Rhino 5 FilletSrf (Radius) and ChamferSrf (Distances): Select first surface, Select second
/// surface; each click says the side kept, so which corner is rounded or cut.
fn fillet_srf(e: &Event) -> Reply {
    let chamfer = e.command == CHAMFER;
    let (label, noun) = if chamfer { ("Chamfer Surfaces", "chamfer") } else { ("Fillet Surfaces", "fillet") };
    let mut st: FilletState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = FilletState::default();
    }
    let radius = st.radius.unwrap_or_else(|| e.number("fs_radius", 1.0));
    let dist0 = e.number("cs_distance", 1.0);
    let dists = st.distances.unwrap_or([dist0, e.number("cs_second_distance", dist0)]);
    let save = |st: &FilletState| serde_json::to_value(st).unwrap_or(Value::Null);
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    // Meshes (no exact surface): the mesh fillet / chamfer, both picked, Enter.
    if surfaces.is_empty() && !meshes(e).is_empty() && st.first.is_none() {
        return fillet(e);
    }
    let center = |s: &Selected| Srf::of(s.exact.as_ref().unwrap()).map_or(s.bbox[0], |f| f.at_st(0.5, 0.5));
    let number = |t: &str| t.trim().replace(',', ".").parse::<f64>().ok().filter(|v| *v > 0.0);
    let mut error = None;
    match &e.input {
        Input::Option { name, value } if name.eq_ignore_ascii_case("Radius") && !chamfer => match value.as_deref().and_then(number) {
            Some(r) => st.radius = Some(r),
            None => st.want = 1,
        },
        Input::Option { name, .. } if name.eq_ignore_ascii_case("Distances") && chamfer => st.want = 1,
        Input::Option { name, value } if ["Trim", "Extend", "Split"].iter().any(|n| name.eq_ignore_ascii_case(n)) => {
            let (slot, key, default) = if name.eq_ignore_ascii_case("Trim") {
                (&mut st.trim, "fs_trim", true)
            } else if name.eq_ignore_ascii_case("Extend") {
                (&mut st.extend, "fs_extend", true)
            } else {
                (&mut st.split, "fs_split", false)
            };
            let now = slot.unwrap_or_else(|| e.flag(key, default));
            *slot = Some(match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("yes"),
                None => !now,
            });
        }
        Input::Text { text } if st.want > 0 => match number(text) {
            Some(v) if chamfer => {
                let mut d = dists;
                d[usize::from(st.want - 1)] = v;
                st.distances = Some(d);
                st.want = if st.want == 1 { 2 } else { 0 };
            }
            Some(v) => {
                st.radius = Some(v);
                st.want = 0;
            }
            None => error = Some(format!("{text}: type a number more than 0")),
        },
        Input::Enter if st.want > 0 => st.want = if chamfer && st.want == 1 { 2 } else { 0 },
        _ => {}
    }
    let dists = st.distances.unwrap_or(dists);
    if st.want > 0 {
        let prompt = match (chamfer, st.want) {
            (false, _) => format!("Radius <{}>", fmt(radius)),
            (true, 1) => format!("First chamfer distance <{}>", fmt(dists[0])),
            (true, _) => format!("Second chamfer distance <{}>", fmt(dists[1])),
        };
        let r = Reply::ask(prompt, save(&st)).want(Want::Number);
        return match error {
            Some(why) => r.refuse(why),
            None => r,
        };
    }
    let trim = st.trim.unwrap_or_else(|| e.flag("fs_trim", true));
    let extend = st.extend.unwrap_or_else(|| e.flag("fs_extend", true));
    let split = st.split.unwrap_or_else(|| e.flag("fs_split", false));
    let options = || {
        let mut v = if chamfer {
            let mut t = OptionToken::action("Distances");
            t.value = Some(format!("{},{}", fmt(dists[0]), fmt(dists[1])));
            vec![t]
        } else {
            vec![OptionToken::number("Radius", Some(radius))]
        };
        v.push(OptionToken::toggle("Extend", extend));
        v.push(OptionToken::toggle("Trim", trim));
        v.push(OptionToken::toggle("Split", split));
        v
    };
    // The picks: preselected surfaces count in order (their middles as the kept sides).
    let pick_of = |s: &Selected| e.pick.unwrap_or_else(|| center(s));
    if st.first.is_none() {
        if let Some(s) = surfaces.first() {
            let at = if matches!(e.input, Input::Start) { center(s) } else { pick_of(s) };
            st.first = Some((s.id.clone(), at));
        }
    }
    let first_prompt = format!("Select first surface to {noun}");
    let second_prompt = format!("Select second surface to {noun}");
    let second = st.first.as_ref().and_then(|(id, _)| surfaces.iter().find(|s| &s.id != id).copied());
    let (Some((first_id, keep1)), Some(second)) = (st.first.clone(), second) else {
        let prompt = if st.first.is_none() { first_prompt } else { second_prompt };
        let r = Reply::ask(prompt, save(&st)).want(Want::Objects).with_options(options());
        return match error {
            Some(why) => r.refuse(why),
            None => r,
        };
    };
    let keep2 = if matches!(e.input, Input::Start) { center(second) } else { pick_of(second) };
    let Some(first) = surfaces.iter().find(|s| s.id == first_id) else {
        return Reply::ask(first_prompt, save(&FilletState::default())).want(Want::Objects);
    };
    let tol = e.number("sw_tolerance", 0.001).max(1e-6);
    let (g1, g2) = (first.exact.as_ref().unwrap(), second.exact.as_ref().unwrap());
    let made = if chamfer {
        om9_addon_geom::fillet::chamfer_with(g1, g2, dists[0], dists[1], keep1, keep2, tol, extend)
            .map(|(f, on)| (f, on, format!("distances {}, {} mm", fmt(dists[0]), fmt(dists[1]))))
    } else {
        om9_addon_geom::fillet::fillet_with(g1, g2, radius, keep1, keep2, tol, extend)
            .map(|(f, on)| (f, on, format!("radius {} mm, within {} mm", fmt(radius), fmt(tol))))
    };
    match made {
        Ok((f, on, what)) => {
            let mut r = Reply::finish(vec![f.geo()]).say(format!("{label}: {what}"));
            if trim || split {
                // Trim=Yes: each input trimmed along its edge of the new surface, the side
                // picked kept. Split=Yes (Trim=No): each input cut there into two surfaces.
                // With Extend, only the part of the edge lying on that input cuts it.
                let edge = |k: usize| -> Option<Nc> {
                    let whole = Nc {
                        degree: f.degree[0],
                        points: f.rows.iter().map(|row| if k == 0 { row[0] } else { row[row.len() - 1] }).collect(),
                        knots: f.knots[0].clone(),
                    };
                    let [a, b] = on[k];
                    if b - a < 1e-9 {
                        None
                    } else if a <= 1e-12 && b >= 1.0 - 1e-12 {
                        Some(whole)
                    } else {
                        whole.part(a, b)
                    }
                };
                let mut done = Vec::new();
                let mut extra = Vec::new();
                for (sel, g, k, keep) in [(*first, g1, 0, keep1), (second, g2, 1, keep2)] {
                    let Some(curve) = edge(k) else {
                        r = r.say(format!("{label}: not trimmed: the edge is not on a surface"));
                        continue;
                    };
                    if trim {
                        match om9_addon_geom::trimsides::trim_keeping(g, &curve, keep) {
                            Ok(t) => done.push(Replacement { id: sel.id.clone(), geo: t }),
                            Err(why) => r = r.say(format!("{label}: not trimmed: {why}")),
                        }
                    } else {
                        match om9_addon_geom::trimsides::split_keeping(g, &curve, keep) {
                            Ok([a, b]) => {
                                done.push(Replacement { id: sel.id.clone(), geo: a });
                                extra.push(b);
                            }
                            Err(why) => r = r.say(format!("{label}: not split: {why}")),
                        }
                    }
                }
                if done.len() == 2 {
                    r.replace = done;
                    r.create.extend(extra);
                }
            }
            r
        }
        Err(why) => {
            // Keep the first pick; ask for the second again.
            let mut again = st.clone();
            again.first = Some((first_id, keep1));
            Reply::ask(second_prompt, save(&again)).want(Want::Objects).with_options(options()).refuse(format!("{label}: {why}"))
        }
    }
}

pub fn fillet_rows(
    a: &SelectedMesh,
    b: &SelectedMesh,
    r: &dyn Fn(f64) -> f64,
    flip: [bool; 2],
    chamfer: bool,
    n: usize,
) -> Option<Vec<Vec<Vec3>>> {
    let mut chains = section::mesh_intersection(a, b);
    chains.sort_by(|x, y| y.0.len().cmp(&x.0.len()));
    let (pts, closed) = chains.into_iter().next()?;
    let curve = resample(&pts, closed, n);
    let path = Path::new(&curve, closed)?;
    let count = curve.len();
    let mut rows = Vec::new();
    for (k, c) in curve.iter().enumerate() {
        let s = path.closest(*c);
        let tan = path.tangent(s);
        let (_, mut n1) = mesh_point(a, *c)?;
        let (_, mut n2) = mesh_point(b, *c)?;
        if flip[0] {
            n1 = scale(n1, -1.0);
        }
        if flip[1] {
            n2 = scale(n2, -1.0);
        }
        let rr = r(k as f64 / (count - 1).max(1) as f64);
        // The ball touches both surfaces on their back sides: dot(o − c, n) = −r.
        let o = add(*c, solve3([n1, n2, tan], [-rr, -rr, 0.0])?);
        let t1 = add(o, scale(n1, rr));
        let t2 = add(o, scale(n2, rr));
        let row: Vec<Vec3> = if chamfer {
            (0..9)
                .map(|i| add(t1, scale(sub(t2, t1), i as f64 / 8.0)))
                .collect()
        } else {
            let (x, y) = (unit(sub(t1, o)), sub(t2, o));
            let yy = unit(sub(y, scale(x, dot(y, x))));
            let ang = dot(unit(y), x).clamp(-1.0, 1.0).acos();
            (0..9)
                .map(|i| {
                    let a = ang * i as f64 / 8.0;
                    add(o, scale(add(scale(x, a.cos()), scale(yy, a.sin())), rr))
                })
                .collect()
        };
        rows.push(row);
    }
    if closed {
        rows.push(rows[0].clone());
    }
    Some(rows)
}

fn fillet(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let chamfer = matches!(cmd, CHAMFER | VAR_CHAMFER);
    let var = matches!(cmd, VAR_FILLET | VAR_CHAMFER);
    let label = match cmd {
        FILLET => "Fillet Surfaces",
        VAR_FILLET => "Variable Fillet",
        CHAMFER => "Chamfer Surfaces",
        _ => "Variable Chamfer",
    };
    let r0 = e.number("fs_radius", 1.0);
    let r1 = if var {
        e.number("fs_end_radius", 2.0)
    } else {
        r0
    };
    let flip = [
        e.flag("fs_flip_first", false),
        e.flag("fs_flip_second", false),
    ];
    built(e, label, "select two intersecting surfaces, Enter", || {
        let ms = meshes(e);
        if ms.len() != 2 {
            return Err("select two surfaces that cross".into());
        }
        let rows = fillet_rows(
            ms[0].mesh.as_ref().unwrap(),
            ms[1].mesh.as_ref().unwrap(),
            &|t| r0 + (r1 - r0) * t,
            flip,
            chamfer,
            48,
        )
        .ok_or("the surfaces do not cross")?;
        Ok((
            vec![surface(&rows)?],
            format!(
                "{} {}{}",
                if chamfer { "distance" } else { "radius" },
                fmt(r0),
                if var {
                    format!(" → {}", fmt(r1))
                } else {
                    String::new()
                }
            ),
        ))
    })
}

// ---- offsets, tween, merge, match ------------------------------------------------------------

/// Rhino 5 OffsetSrf: Select surfaces to offset, Enter, Offset distance <d> (FlipAll Loose
/// Tolerance BothSides DeleteInput). Planes, spheres and open cylinders offset exactly;
/// free-form surfaces within the tolerance; meshes vertex by vertex.
fn offset_srf(e: &Event) -> Reply {
    let flow = Flow::new("Offset Surface")
        .select_prompt("Select surfaces to offset")
        .number("Offset distance", e.number("os_distance", 1.0))
        .toggle("FlipAll", e.flag("os_flip", false))
        .toggle("Loose", e.flag("os_loose", false))
        .num("Tolerance", e.number("os_tolerance", 0.0))
        .toggle("BothSides", e.flag("os_both_sides", false))
        .toggle("DeleteInput", false);
    let make = |c: &Collected| -> Result<(Vec<Geo>, Vec<String>, String), String> {
        let d = c.n(0).unwrap_or(flow_default(&flow, "Offset distance", 1.0));
        let d = if c.yes(&flow, "FlipAll") { -d } else { d };
        let tol = match c.num(&flow, "Tolerance") {
            t if t > 0.0 => t,
            _ => e.number("sw_tolerance", 0.001).max(1e-6),
        };
        let loose = c.yes(&flow, "Loose");
        let sides: Vec<f64> = if c.yes(&flow, "BothSides") { vec![d, -d] } else { vec![d] };
        let mut out = Vec::new();
        let mut used = Vec::new();
        let (mut exact_n, mut worst) = (0, 0.0f64);
        for s in e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))) {
            for dd in &sides {
                let o = om9_addon_geom::offset::offset(s.exact.as_ref().unwrap(), *dd, tol, loose)?;
                exact_n += usize::from(o.exact);
                worst = worst.max(o.deviation);
                out.push(o.geo);
            }
            used.push(s.id.clone());
        }
        // Meshes: every vertex along its averaged normal.
        let mesh_sel: Vec<&Selected> = meshes(e).into_iter().filter(|s| !used.contains(&s.id)).collect();
        for s in mesh_sel {
            let m = s.mesh.as_ref().unwrap();
            let mut vn = vec![[0.0; 3]; m.vertices.len()];
            for t in &m.triangles {
                let [a, b, cc] = t.map(|i| m.vertices[i as usize]);
                let n = cross(sub(b, a), sub(cc, a));
                for &i in t {
                    vn[i as usize] = add(vn[i as usize], n);
                }
            }
            for dd in &sides {
                let moved = Mesh {
                    vertices: m.vertices.iter().zip(&vn).map(|(p, n)| add(*p, scale(unit(*n), *dd))).collect(),
                    triangles: m.triangles.clone(),
                };
                out.push(moved.geo("Offset", None, None));
            }
            used.push(s.id.clone());
        }
        if out.is_empty() {
            return Err("select surfaces or meshes".into());
        }
        let how = if exact_n == out.len() { "exact".to_owned() } else { format!("within {} mm", fmt(worst.max(1e-6).min(tol))) };
        let what = format!("{} offsets {} mm, {how}", sides.len() * used.len(), fmt(d.abs()));
        Ok((out, used, what))
    };
    flow.step(
        e,
        &|c, _| Show { geo: make(c).map(|m| m.0).unwrap_or_default(), ..Default::default() },
        &|c| {
            let (out, used, what) = make(c)?;
            let mut r = Reply::finish(out).say(format!("Offset Surface: {what}"));
            if c.yes(&flow, "DeleteInput") {
                r.delete = used;
            }
            Ok(r)
        },
    )
}

fn offset(e: &Event) -> Reply {
    let var = e.command == VAR_OFFSET;
    let d = e.number("os_distance", 1.0);
    let corners = [
        d,
        e.number("os_d10", d),
        e.number("os_d01", d),
        e.number("os_d11", d),
    ];
    let flip = e.flag("os_flip", false);
    let label = if var {
        "Variable Offset Surface"
    } else {
        "Offset Surface"
    };
    built(e, label, "select surfaces, Enter", || {
        let mut out = Vec::new();
        for (_, s) in nurbs_surfaces(e) {
            let (nu, nv) = (24, 24);
            let rows: Vec<Vec<Vec3>> = (0..nu)
                .map(|i| {
                    let a = i as f64 / (nu - 1) as f64;
                    (0..nv)
                        .map(|j| {
                            let b = j as f64 / (nv - 1) as f64;
                            let dd = if var {
                                corners[0] * (1.0 - a) * (1.0 - b)
                                    + corners[1] * a * (1.0 - b)
                                    + corners[2] * (1.0 - a) * b
                                    + corners[3] * a * b
                            } else {
                                d
                            };
                            let n = s.normal_st(a, b);
                            add(s.at_st(a, b), scale(n, if flip { -dd } else { dd }))
                        })
                        .collect()
                })
                .collect();
            out.push(surface(&rows)?);
        }
        // Meshes: every vertex along its averaged normal.
        let exact: Vec<&str> = nurbs_surfaces(e)
            .iter()
            .map(|(s, _)| s.id.as_str())
            .collect();
        for s in meshes(e)
            .into_iter()
            .filter(|s| !exact.contains(&s.id.as_str()))
        {
            let m = s.mesh.as_ref().unwrap();
            let mut vn = vec![[0.0; 3]; m.vertices.len()];
            for t in &m.triangles {
                let [a, b, c] = [
                    m.vertices[t[0] as usize],
                    m.vertices[t[1] as usize],
                    m.vertices[t[2] as usize],
                ];
                let n = cross(sub(b, a), sub(c, a));
                for &i in t {
                    vn[i as usize] = add(vn[i as usize], n);
                }
            }
            let v: Vec<Vec3> = m
                .vertices
                .iter()
                .zip(&vn)
                .map(|(p, n)| add(*p, scale(unit(*n), if flip { -d } else { d })))
                .collect();
            out.push(Geo::Mesh {
                name: format!("{} offset", s.name),
                vertices: v,
                triangles: m.triangles.clone(),
                meta: None,
                layer: None,
            });
        }
        if out.is_empty() {
            return Err("select surfaces or meshes".into());
        }
        let k = out.len();
        Ok((out, format!("{k} offsets {} mm", fmt(d))))
    })
}

fn tween(e: &Event) -> Reply {
    let count = e.number("tw_count", 1.0).round().clamp(1.0, 50.0) as usize;
    built(e, "Tween Surfaces", "select two surfaces, Enter", || {
        let ss = nurbs_surfaces(e);
        if ss.len() != 2 {
            return Err("select two NURBS surfaces".into());
        }
        // Rhino 5 MatchMethod=None made exact: corners matched, the surfaces made compatible,
        // corresponding control points connected.
        let out = om9_addon_geom::surfedit::tween(ss[0].0.exact.as_ref().unwrap(), ss[1].0.exact.as_ref().unwrap(), count)?;
        Ok((out, format!("{count} surfaces between (exact)")))
    })
}

/// MergeSrf command state: the options typed on the command line.
#[derive(Serialize, Deserialize, Default, Clone)]
struct MergeState {
    smooth: Option<bool>,
    tolerance: Option<f64>,
    roundness: Option<f64>,
    /// Waiting for a number: "Tolerance" or "RoundnessFactor".
    want: Option<String>,
}

/// Rhino 5 MergeSrf: Select a pair of surfaces to merge (Smooth Tolerance RoundnessFactor);
/// the second pick merges. Smooth=No is exact.
fn merge(e: &Event) -> Reply {
    let label = "Merge Surfaces";
    let mut st: MergeState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = MergeState::default();
    }
    let save = |st: &MergeState| serde_json::to_value(st).unwrap_or(Value::Null);
    let mut error = None;
    match &e.input {
        Input::Option { name, value } => {
            let parsed = value.as_deref().and_then(|v| v.trim().replace(',', ".").parse::<f64>().ok());
            if name.eq_ignore_ascii_case("Smooth") {
                let now = st.smooth.unwrap_or_else(|| e.flag("ms_smooth", true));
                st.smooth = Some(match value.as_deref() {
                    Some(v) => v.eq_ignore_ascii_case("yes"),
                    None => !now,
                });
            } else if name.eq_ignore_ascii_case("Tolerance") || name.eq_ignore_ascii_case("RoundnessFactor") {
                match parsed {
                    Some(v) if name.eq_ignore_ascii_case("Tolerance") => st.tolerance = Some(v),
                    Some(v) => st.roundness = Some(v),
                    None => st.want = Some(name.clone()),
                }
            }
        }
        Input::Text { text } if st.want.is_some() => match text.trim().replace(',', ".").parse::<f64>() {
            Ok(v) => {
                if st.want.as_deref().is_some_and(|w| w.eq_ignore_ascii_case("Tolerance")) {
                    st.tolerance = Some(v);
                } else {
                    st.roundness = Some(v);
                }
                st.want = None;
            }
            Err(_) => error = Some(format!("{text}: type a number")),
        },
        Input::Enter if st.want.is_some() => st.want = None,
        _ => {}
    }
    let smooth = st.smooth.unwrap_or_else(|| e.flag("ms_smooth", true));
    let tol = st.tolerance.unwrap_or_else(|| e.number("ms_tolerance", 0.001));
    let roundness = st.roundness.unwrap_or_else(|| e.number("ms_roundness", 1.0));
    if let Some(w) = &st.want {
        let now = if w.eq_ignore_ascii_case("Tolerance") { tol } else { roundness };
        let r = Reply::ask(format!("{w} <{}>", fmt(now)), save(&st)).want(Want::Number);
        return match error {
            Some(why) => r.refuse(why),
            None => r,
        };
    }
    let options = vec![
        OptionToken::toggle("Smooth", smooth),
        OptionToken::number("Tolerance", Some(tol)),
        OptionToken::number("RoundnessFactor", Some(roundness)),
    ];
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    let ask = |st: &MergeState| Reply::ask("Select a pair of surfaces to merge", save(st)).want(Want::Objects).with_options(options.clone());
    if surfaces.len() < 2 {
        let r = ask(&st);
        return match error {
            Some(why) => r.refuse(why),
            None => r,
        };
    }
    if tol <= 0.0 {
        return ask(&st).refuse(format!("{label}: the tolerance must be more than 0"));
    }
    let (a, b) = (surfaces[0], surfaces[1]);
    match om9_addon_geom::merge::merge(a.exact.as_ref().unwrap(), b.exact.as_ref().unwrap(), smooth, roundness, tol) {
        Ok(g) => {
            let mut r = Reply::finish(Vec::new()).say(format!("{label}: {}", if smooth { "smooth" } else { "exact" }));
            r.replace.push(Replacement { id: a.id.clone(), geo: g });
            r.delete = vec![b.id.clone()];
            r
        }
        Err(why) => ask(&st).refuse(format!("{label}: {why}")),
    }
}

/// Rhino 5 MatchSrf: Select untrimmed surface edge to change, Select target surface edge
/// (a click near each), then Continuity (Position / Tangency / Curvature) with a preview;
/// Enter changes the first surface.
fn match_srf(e: &Event) -> Reply {
    use om9_addon_geom::matchsrf::{self, Continuity, Edge};
    let label = "Match Surface";
    let mut st: BlendState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = BlendState::default();
    }
    let save = |st: &BlendState| serde_json::to_value(st).unwrap_or(Value::Null);
    let entering = match pick_two_edges(e, &mut st.edges, ["Select untrimmed surface edge to change", "Select target surface edge"]) {
        Ok(entering) => entering,
        Err((prompt, want)) => return Reply::ask(prompt, save(&st)).want(want),
    };
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    let find = |id: &str| surfaces.iter().find(|s| s.id == id).copied();
    let (Some(a), Some(b)) = (find(&st.edges[0].0), find(&st.edges[1].0)) else {
        return Reply::ask("Select untrimmed surface edge to change", save(&BlendState::default())).want(Want::Objects);
    };
    let flow = Flow::new(label)
        .no_selection()
        .plain()
        .confirm("Press Enter to match, or change an option")
        .choice(
            "Continuity",
            &["Position", "Tangency", "Curvature"],
            match e.text("sm_continuity", "tangency") {
                "position" => "Position",
                "curvature" => "Curvature",
                _ => "Tangency",
            },
        )
        .toggle("Average", e.flag("sm_average", false));
    let (ea, eb) = (st.edges[0].clone(), st.edges[1].clone());
    let tol = e.number("sw_tolerance", 0.001).max(1e-6);
    // The changed surface, and with Average=Yes the changed target too.
    let make = |c: &Collected| -> Result<(Geo, Option<Geo>), String> {
        let cont = match c.opt(&flow, "Continuity") {
            "Position" => Continuity::Position,
            "Curvature" => Continuity::Curvature,
            _ => Continuity::Tangency,
        };
        let (ga, gb) = (a.exact.as_ref().unwrap(), b.exact.as_ref().unwrap());
        let (e1, e2) = (Edge { dir: ea.1, at_end: ea.2 }, Edge { dir: eb.1, at_end: eb.2 });
        if c.yes(&flow, "Average") {
            matchsrf::match_average(ga, e1, gb, e2, cont, tol).map(|(x, y)| (x, Some(y)))
        } else {
            matchsrf::match_srf(ga, e1, gb, e2, cont, tol).map(|x| (x, None))
        }
    };
    let mut sub = e.clone();
    sub.state = st.flow.clone();
    if entering || matches!(sub.input, Input::Selection | Input::Point { .. }) {
        sub.input = Input::Hover { point: [0.0; 3] };
    }
    let failed = std::cell::RefCell::new(None);
    let mut r = flow.step(
        &sub,
        &|c, _| match make(c) {
            Ok((g, other)) => Show { geo: std::iter::once(g).chain(other).collect(), ..Default::default() },
            Err(why) => {
                *failed.borrow_mut() = Some(why);
                Show::default()
            }
        },
        &|c| {
            let (g, other) = make(c)?;
            let avg = if other.is_some() { ", average" } else { "" };
            let mut r = Reply::finish(Vec::new()).say(format!("{label}: {}{avg}", c.opt(&flow, "Continuity")));
            r.replace.push(Replacement { id: a.id.clone(), geo: g });
            if let Some(o) = other {
                r.replace.push(Replacement { id: b.id.clone(), geo: o });
            }
            Ok(r)
        },
    );
    if !r.done {
        st.flow = r.state.clone();
        r.state = save(&st);
        let why = failed.borrow_mut().take();
        if let (None, Some(why)) = (&r.error, why) {
            r = r.refuse(format!("{label}: {why}"));
        }
    }
    r
}

fn drape(e: &Event) -> Reply {
    let density = e.number("dr_density", 30.0).round().clamp(4.0, 200.0) as usize;
    let flow = Flow::new("Drape")
        .point("First corner of the drape area")
        .point("Opposite corner");
    let make = |c: &Collected, end: Option<Vec3>| -> Result<Geo, String> {
        let (Some(a), Some(b)) = (c.p(0), c.p(1).or(end)) else {
            return Err("pick the area".into());
        };
        let n = e.plane.normal();
        let targets: Vec<Target> = meshes(e)
            .iter()
            .filter_map(|s| Target::new(s.mesh.as_ref().unwrap(), n))
            .collect();
        if targets.is_empty() {
            return Err("select the objects to drape over".into());
        }
        let (la, lb) = (e.plane.local(a), e.plane.local(b));
        let floor = targets.iter().flat_map(|_| [0.0]).fold(0.0, f64::min);
        let rows: Vec<Vec<Vec3>> = (0..density)
            .map(|i| {
                let u = la[0] + (lb[0] - la[0]) * i as f64 / (density - 1) as f64;
                (0..density)
                    .map(|j| {
                        let v = la[1] + (lb[1] - la[1]) * j as f64 / (density - 1) as f64;
                        let p = e.plane.at(u, v, 1e6);
                        targets
                            .iter()
                            .filter_map(|t| t.drop(p))
                            .map(|h| h.0)
                            .max_by(|x, y| dot(*x, n).total_cmp(&dot(*y, n)))
                            .unwrap_or_else(|| e.plane.at(u, v, floor))
                    })
                    .collect()
            })
            .collect();
        surface_on_grid(&rows, [3, 3]).ok_or_else(|| "drape".to_owned())
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).into_iter().collect(),
            ..Default::default()
        },
        &|c| Ok(Reply::finish(vec![make(c, None)?]).say("Drape: done")),
    )
}

fn heightfield(e: &Event) -> Reply {
    let file = e.text("hf_file", "").to_owned();
    let width = e.number("hf_width", 20.0);
    let height = e.number("hf_height", 1.0);
    let density = e.number("hf_density", 60.0).round().clamp(4.0, 300.0) as usize;
    let invert = e.flag("hf_invert", false);
    let flow = Flow::new("Heightfield from Image")
        .no_selection()
        .point("Lower left corner");
    flow.step(e, &|_, _| Show::default(), &|c| {
        if file.trim().is_empty() {
            return Err("choose the image file in the builder (File=…)".into());
        }
        let img = om9_tools::read_image(std::path::Path::new(&file))?;
        let l = e.plane.local(c.p(0).unwrap());
        let aspect = img.h as f64 / img.w.max(1) as f64;
        let (nu, nv) = (density, ((density as f64 * aspect).round() as usize).max(2));
        let rows: Vec<Vec<Vec3>> = (0..nu)
            .map(|i| {
                let x = i as f64 / (nu - 1) as f64;
                (0..nv)
                    .map(|j| {
                        let y = j as f64 / (nv - 1) as f64;
                        let (px, py) = (
                            (x * (img.w - 1) as f64).round() as usize,
                            ((1.0 - y) * (img.h - 1) as f64).round() as usize,
                        );
                        let g = img.px[py * img.w + px];
                        let z = if invert { 1.0 - g } else { g } * height;
                        e.plane
                            .at(l[0] + x * width, l[1] + y * width * aspect, l[2] + z)
                    })
                    .collect()
            })
            .collect();
        Ok(
            Reply::finish(vec![surface_on_grid(&rows, [3, 3]).ok_or("heightfield")?]).say(format!(
                "Heightfield: {} × {} mm, {} mm high",
                fmt(width),
                fmt(width * aspect),
                fmt(height)
            )),
        )
    })
}

// ---- edits of one surface --------------------------------------------------------------------

fn rebuild(e: &Event) -> Reply {
    let nu = e.number("sr_u_points", 10.0).round().clamp(2.0, 200.0) as usize;
    let nv = e.number("sr_v_points", 10.0).round().clamp(2.0, 200.0) as usize;
    let deg = e.number("sr_degree", 3.0).round().clamp(1.0, 7.0) as usize;
    let deg_v = e.number("sr_degree_v", deg as f64).round().clamp(1.0, 7.0) as usize;
    replaced(e, "Rebuild Surface", "select NURBS surfaces, Enter", || {
        // Rhino's dialog reports the maximum deviation from the original.
        let mut worst = 0.0f64;
        let items: Vec<(String, Geo)> = nurbs_surfaces(e)
            .iter()
            .filter_map(|(s, f)| {
                let g = surface_through(&grid(f, nu, nv), [deg.min(nu - 1), deg_v.min(nv - 1)])?;
                if let Some(r) = Srf::of(&g) {
                    worst = worst.max(surface_gap(f, &r, 2 * nu.max(nv).clamp(8, 60)));
                }
                Some((s.id.clone(), g))
            })
            .collect();
        let k = items.len();
        Ok((
            items,
            Vec::new(),
            format!("{k} surfaces, {nu} × {nv} points, degree {deg} × {deg_v}, maximum deviation {} mm", fmt(worst)),
        ))
    })
}

/// Largest distance from a surface `b` to a surface `a` it was fitted to, over a k × k grid
/// of `b`: each sample projected on `a` (Gauss-Newton from the same parameters, since a fit
/// keeps the parameters close; the two may differ in parameterisation, so the distance is to
/// the surface, not to the point of the same parameters).
fn surface_gap(a: &Srf, b: &Srf, k: usize) -> f64 {
    let mut m: f64 = 0.0;
    let h = 1e-6;
    for i in 0..=k {
        for j in 0..=k {
            let (s0, t0) = (i as f64 / k as f64, j as f64 / k as f64);
            let p = b.at_st(s0, t0);
            let (mut s, mut t) = (s0, t0);
            let mut d = dist(a.at_st(s, t), p);
            for _ in 0..8 {
                let q = a.at_st(s, t);
                let r = sub(p, q);
                let ds = {
                    let s2 = if s + h <= 1.0 { s + h } else { s - h };
                    scale(sub(a.at_st(s2, t), q), 1.0 / (s2 - s))
                };
                let dt = {
                    let t2 = if t + h <= 1.0 { t + h } else { t - h };
                    scale(sub(a.at_st(s, t2), q), 1.0 / (t2 - t))
                };
                let (aa, ab, bb) = (dot(ds, ds), dot(ds, dt), dot(dt, dt));
                let det = aa * bb - ab * ab;
                if det.abs() < 1e-300 {
                    break;
                }
                let (ra, rb) = (dot(r, ds), dot(r, dt));
                let (s2, t2) = (
                    (s + (bb * ra - ab * rb) / det).clamp(0.0, 1.0),
                    (t + (aa * rb - ab * ra) / det).clamp(0.0, 1.0),
                );
                let d2 = dist(a.at_st(s2, t2), p);
                if d2 >= d {
                    break;
                }
                (s, t, d) = (s2, t2, d2);
            }
            m = m.max(d);
        }
    }
    m
}

/// Rhino 5 FitSrf: Select surfaces, then the fitting tolerance (options UDegree, VDegree,
/// DeleteInput). The fewest control points (interpolating a uniform grid, the count growing
/// by about half each try) whose surface stays within the tolerance of the input, checked
/// between the fitted points where the error peaks.
fn refit(e: &Event) -> Reply {
    let tol = e.number("srf_tolerance", 0.01).max(1e-6);
    let deg = [
        (e.number("rf_degree_u", 3.0).round() as usize).clamp(1, 7),
        (e.number("rf_degree_v", 3.0).round() as usize).clamp(1, 7),
    ];
    let delete_input = e.flag("rf_delete_input", true);
    replaced(
        e,
        "Refit to Tolerance",
        "select NURBS surfaces, Enter",
        || {
            let mut items = Vec::new();
            let mut extra = Vec::new();
            let mut worst: f64 = 0.0;
            for (s, f) in nurbs_surfaces(e) {
                let mut n = deg[0].max(deg[1]) + 1;
                let mut best = None;
                loop {
                    if let Some(g) = surface_through(&grid(&f, n, n), deg) {
                        let d = Srf::of(&g).map_or(f64::MAX, |x| surface_gap(&f, &x, (2 * (n - 1)).clamp(16, 160)));
                        let ok = d <= tol;
                        best = Some((g, d));
                        if ok || n >= 160 {
                            break;
                        }
                    }
                    if n >= 160 {
                        break;
                    }
                    n = (n + n / 2).clamp(n + 1, 160);
                }
                if let Some((g, d)) = best {
                    worst = worst.max(d);
                    if delete_input {
                        items.push((s.id.clone(), g));
                    } else {
                        extra.push(g);
                    }
                }
            }
            let k = items.len() + extra.len();
            let what = if worst <= tol {
                format!("{k} surfaces within {} mm (maximum deviation {} mm)", fmt(tol), fmt(worst))
            } else {
                format!("{k} surfaces, maximum deviation {} mm (more than {} mm)", fmt(worst), fmt(tol))
            };
            Ok((items, extra, what))
        },
    )
}

fn shrink(e: &Event) -> Reply {
    replaced(
        e,
        "Shrink Trimmed Surface",
        "select trimmed surfaces, Enter",
        || {
            let mut items = Vec::new();
            for (s, f) in nurbs_surfaces(e) {
                let Some(m) = &s.mesh else { continue };
                // The trimmed part (its tessellation) covers this parameter box.
                let (mut lo, mut hi) = ([1.0f64; 2], [0.0f64; 2]);
                for v in m.vertices.iter().step_by((m.vertices.len() / 400).max(1)) {
                    let (a, b) = f.closest_st(*v);
                    lo = [lo[0].min(a), lo[1].min(b)];
                    hi = [hi[0].max(a), hi[1].max(b)];
                }
                if hi[0] - lo[0] > 0.999 && hi[1] - lo[1] > 0.999 {
                    continue;
                }
                // Rhino: full-multiplicity knots at the box, the outer control points dropped
                // (a small margin kept, as ShrinkTrimmedSrf does); the surface is unchanged.
                let m = 0.01;
                let (u, v) = ([(lo[0] - m).max(0.0), (hi[0] + m).min(1.0)], [(lo[1] - m).max(0.0), (hi[1] + m).min(1.0)]);
                if let Ok(g) = om9_addon_geom::surfedit::shrink(s.exact.as_ref().unwrap(), u, v) {
                    items.push((s.id.clone(), g));
                }
            }
            let k = items.len();
            if k == 0 {
                return Err("the surfaces already fit their trims".into());
            }
            Ok((items, Vec::new(), format!("{k} surfaces shrunk")))
        },
    )
}

fn untrim(e: &Event) -> Reply {
    replaced(e, "Untrim", "select trimmed surfaces, Enter", || {
        let items: Vec<(String, Geo)> = nurbs_surfaces(e)
            .iter()
            .map(|(s, _)| (s.id.clone(), s.exact.clone().unwrap()))
            .collect();
        let k = items.len();
        Ok((
            items,
            Vec::new(),
            format!("{k} surfaces back to their full domain"),
        ))
    })
}

/// Rhino 5 Split (Isocurve): Select surface to split, Select isocurve location (Direction
/// U / V / Both, the isocurve follows the pointer); exact pieces (knot insertion).
fn split(e: &Event) -> Reply {
    let flow = Flow::new("Split at Isocurve")
        .select_prompt("Select surface to split")
        .point("Select isocurve location")
        .choice("Direction", &["U", "V", "Both"], "U");
    let target = || nurbs_surfaces(e).into_iter().next();
    let iso = |c: &Collected, at: Vec3| -> Vec<Geo> {
        let Some((_, f)) = target() else { return vec![] };
        let (a, b) = f.closest_st(at);
        match c.opt(&flow, "Direction") {
            "U" => vec![f.isocurve(0, a)],
            "V" => vec![f.isocurve(1, b)],
            _ => vec![f.isocurve(0, a), f.isocurve(1, b)],
        }
    };
    flow.step(
        e,
        &|c, h| Show { geo: h.map(|p| iso(c, p)).unwrap_or_default(), ..Default::default() },
        &|c| {
            let p = c.p(0).ok_or("pick the isocurve location")?;
            let (sel, f) = target().ok_or("select a NURBS surface")?;
            let g = sel.exact.as_ref().unwrap();
            let (a, b) = f.closest_st(p);
            let split = om9_addon_geom::surfedit::split_at;
            let pieces: Vec<Geo> = match c.opt(&flow, "Direction") {
                "U" => {
                    let (x, y) = split(g, 0, a)?;
                    vec![x, y]
                }
                "V" => {
                    let (x, y) = split(g, 1, b)?;
                    vec![x, y]
                }
                _ => {
                    let (x, y) = split(g, 0, a)?;
                    let (x1, x2) = split(&x, 1, b)?;
                    let (y1, y2) = split(&y, 1, b)?;
                    vec![x1, x2, y1, y2]
                }
            };
            let n = pieces.len();
            let mut it = pieces.into_iter();
            let first = it.next().unwrap();
            let mut r = Reply::finish(it.collect()).say(format!("Split at Isocurve: {n} surfaces (exact)"));
            r.replace.push(Replacement { id: sel.id.clone(), geo: first });
            Ok(r)
        },
    )
}

/// ExtendSrf command state.
#[derive(Serialize, Deserialize, Default, Clone)]
struct ExtendState {
    /// The surface and its edge (direction, at the far end).
    edge: Option<(String, usize, bool)>,
    smooth: Option<bool>,
}

/// Rhino 5 ExtendSrf: Select edge of surface to extend (Type=Smooth/Line), Extension factor
/// <d> (typed, Enter for the default), with a preview. Smooth is exact (the same surface,
/// longer); Line continues straight along the edge's tangents.
fn extend(e: &Event) -> Reply {
    let label = "Extend Surface";
    let mut st: ExtendState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = ExtendState::default();
    }
    let save = |st: &ExtendState| serde_json::to_value(st).unwrap_or(Value::Null);
    let smooth = st.smooth.unwrap_or_else(|| e.text("es_type", "smooth") != "line");
    let length = e.number("es_length", 1.0);
    if let Input::Option { name, value } = &e.input {
        if name.eq_ignore_ascii_case("Type") {
            let now = match value.as_deref() {
                Some(v) => !v.eq_ignore_ascii_case("line"),
                None => !smooth,
            };
            st.smooth = Some(now);
        }
    }
    let smooth = st.smooth.unwrap_or(smooth);
    let type_token = || OptionToken::choice("Type", if smooth { "Smooth" } else { "Line" }, &["Smooth", "Line"]);
    let surfaces: Vec<&Selected> = e.selection.iter().filter(|s| matches!(s.exact, Some(Geo::Surface { .. }))).collect();
    // The edge: the one nearest the click (on the surface clicked, or the one preselected).
    if st.edge.is_none() {
        let at = match &e.input {
            Input::Point { point } => Some(*point),
            Input::Selection => e.pick,
            _ => None,
        };
        if let (Some(p), Some(s)) = (at, surfaces.last()) {
            if let Some(f) = Srf::of(s.exact.as_ref().unwrap()) {
                let (dir, at) = nearest_edge(&f, p);
                st.edge = Some((s.id.clone(), dir, at > 0.5));
            }
        }
        if st.edge.is_none() {
            // A surface already picked: click near its edge.
            let want = if surfaces.is_empty() { Want::Objects } else { Want::Point };
            return Reply::ask("Select edge of surface to extend", save(&st)).want(want).with_options(vec![type_token()]);
        }
    }
    let (id, dir, at_end) = st.edge.clone().unwrap();
    let Some(sel) = surfaces.iter().find(|s| s.id == id) else {
        return Reply::ask("Select edge of surface to extend", save(&ExtendState::default())).want(Want::Objects);
    };
    let g = sel.exact.as_ref().unwrap();
    let run = |len: f64| om9_addon_geom::extend::extend(g, dir, at_end, len, smooth);
    let typed = match &e.input {
        Input::Text { text } => Some(text.trim().replace(',', ".").parse::<f64>().map_err(|_| format!("{text}: type a number"))),
        Input::Enter => Some(Ok(length)),
        _ => None,
    };
    let ask = |st: &ExtendState| {
        let mut r = Reply::ask(format!("Extension factor <{}>", fmt(length)), save(st))
            .want(Want::Number)
            .with_options(vec![type_token()]);
        if let Ok(g) = run(length) {
            r = r.preview(vec![g]);
        }
        r
    };
    match typed {
        Some(Ok(len)) => match run(len) {
            Ok(out) => {
                let mut r = Reply::finish(Vec::new()).say(format!("{label}: {} mm, {}", fmt(len), if smooth { "smooth (exact)" } else { "line" }));
                r.replace.push(Replacement { id: id.clone(), geo: out });
                r
            }
            Err(why) => ask(&st).refuse(format!("{label}: {why}")),
        },
        Some(Err(why)) => ask(&st).refuse(why),
        None => ask(&st),
    }
}

/// Fin command state.
#[derive(Serialize, Deserialize, Default, Clone)]
struct FinState {
    curve: Option<String>,
    surface: Option<String>,
    tangent: Option<bool>,
    /// Heights set so far: (place on the curve, height).
    #[serde(default)]
    heights: Vec<(f64, f64)>,
    /// A place picked, waiting for its height.
    #[serde(default)]
    at: Option<f64>,
}

/// Rhino 5 Fin: Select curve on surface, Select surface, Pick a location on the curve for a
/// height, the height (typed, or a point: its distance off the surface), then Pick location
/// to set another height, or press Enter if the surface should be uniform in height.
/// Direction Normal / Tangent. A height typed at the first prompt makes a uniform fin at once.
fn fin_cmd(e: &Event) -> Reply {
    let label = "Extrude Curve Normal to Surface";
    let mut st: FinState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = FinState::default();
    }
    let save = |st: &FinState| serde_json::to_value(st).unwrap_or(Value::Null);
    if let Input::Option { name, value } = &e.input {
        if name.eq_ignore_ascii_case("Direction") {
            let now = st.tangent.unwrap_or(false);
            st.tangent = Some(match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("tangent"),
                None => !now,
            });
        }
    }
    let tangent = st.tangent.unwrap_or(false);
    let options = vec![OptionToken::choice("Direction", if tangent { "Tangent" } else { "Normal" }, &["Normal", "Tangent"])];
    let flip = e.flag("en_flip", false);
    let height0 = e.number("en_height", 1.0) * if flip { -1.0 } else { 1.0 };
    if st.curve.is_none() {
        st.curve = curves(e).first().map(|c| c.id.clone());
    }
    if st.surface.is_none() {
        st.surface = e.selection.iter().find(|s| matches!(s.exact, Some(Geo::Surface { .. }))).map(|s| s.id.clone());
    }
    let by_id = |id: &Option<String>| id.as_ref().and_then(|id| e.selection.iter().find(|s| &s.id == id));
    let (Some(curve), Some(srf)) = (by_id(&st.curve), by_id(&st.surface)) else {
        let prompt = if st.curve.is_none() { "Select curve on surface" } else { "Select surface" };
        return Reply::ask(prompt, save(&st)).want(Want::Objects).with_options(options);
    };
    let Some(c) = nc_of(curve) else {
        return Reply::ask("Select curve on surface", save(&FinState::default())).want(Want::Objects).refuse(format!("{label}: the curve has no curve data"));
    };
    let g = srf.exact.as_ref().unwrap();
    let tol = e.number("sw_tolerance", 0.001).max(1e-6);
    // A point gives the height: its distance off the surface at the nearest place on the
    // curve (signed by the normal).
    let height_of = |p: Vec3| -> Option<f64> {
        let f = Srf::of(g)?;
        let t = c.closest(p);
        let q = c.eval(t);
        let (a, b) = f.closest_st(q);
        let n = f.normal_st(a, b);
        Some(dot(sub(p, q), n))
    };
    let make = |hs: &[(f64, f64)]| om9_addon_geom::fillet::fin_heights(&c, g, hs, tangent, tol).map(|f| f.geo());
    let finish = |hs: &[(f64, f64)]| -> Reply {
        match make(hs) {
            Ok(out) => {
                let what = if hs.len() == 1 {
                    format!("height {} mm", fmt(hs[0].1))
                } else {
                    format!("{} heights, {} to {} mm", hs.len(), fmt(hs.iter().map(|x| x.1).fold(f64::MAX, f64::min)), fmt(hs.iter().map(|x| x.1).fold(f64::MIN, f64::max)))
                };
                Reply::finish(vec![out]).say(format!("{label}: {what}, within {} mm", fmt(tol)))
            }
            Err(why) => Reply::ask(format!("Height <{}>", fmt(height0)), save(&FinState { heights: Vec::new(), at: None, ..st.clone() }))
                .want(Want::Point)
                .with_options(options.clone())
                .refuse(format!("{label}: {why}")),
        }
    };
    let typed = |text: &str| text.trim().replace(',', ".").parse::<f64>();
    // Waiting for the height at a picked place.
    if let Some(at) = st.at {
        let height = match &e.input {
            Input::Text { text } => match typed(text) {
                Ok(v) => Some(v),
                Err(_) => return Reply::ask(format!("Height <{}>", fmt(height0)), save(&st)).want(Want::Point).with_options(options).refuse(format!("{text}: type a number")),
            },
            Input::Point { point } => height_of(*point),
            Input::Enter => Some(height0),
            _ => None,
        };
        if let Some(h) = height {
            if h.abs() < 1e-12 {
                return Reply::ask(format!("Height <{}>", fmt(height0)), save(&st)).want(Want::Point).with_options(options).refuse(format!("{label}: the height must not be 0"));
            }
            st.heights.retain(|x| (x.0 - at).abs() > 1e-9);
            st.heights.push((at, h));
            st.at = None;
            let mut r = Reply::ask("Pick location to set another height, or press Enter if the surface should be uniform in height", save(&st)).want(Want::Point).with_options(options);
            if let Ok(out) = make(&st.heights) {
                r = r.preview(vec![out]);
            }
            return r;
        }
        let mut r = Reply::ask(format!("Height <{}>", fmt(height0)), save(&st)).want(Want::Point).with_options(options);
        let mut hs = st.heights.clone();
        if let Some(h) = match &e.input {
            Input::Hover { point } => height_of(*point),
            _ => Some(height0),
        } {
            hs.push((at, h));
        }
        if let Ok(out) = make(&hs) {
            r = r.preview(vec![out]);
        }
        return r;
    }
    // Picking places.
    let first = st.heights.is_empty();
    let prompt = if first { "Pick a location on the curve for a height" } else { "Pick location to set another height, or press Enter if the surface should be uniform in height" };
    match &e.input {
        Input::Point { point } => {
            st.at = Some(c.closest(*point));
            let mut r = Reply::ask(format!("Height <{}>", fmt(height0)), save(&st)).want(Want::Point).with_options(options);
            let mut hs = st.heights.clone();
            hs.push((st.at.unwrap(), height0));
            if let Ok(out) = make(&hs) {
                r = r.preview(vec![out]);
            }
            r
        }
        // A height typed at the first prompt: a uniform fin at once (as before).
        Input::Text { text } if first => match typed(text) {
            Ok(h) => finish(&[(0.0, h)]),
            Err(_) => Reply::ask(prompt, save(&st)).want(Want::Point).with_options(options).refuse(format!("{text}: pick a place on the curve or type a height")),
        },
        Input::Enter if first => finish(&[(0.0, height0)]),
        // Enter after the heights: the fin; a single height is uniform.
        Input::Enter => finish(&st.heights),
        _ => {
            let mut r = Reply::ask(prompt, save(&st)).want(Want::Point).with_options(options);
            let hs = if first { vec![(0.0, height0)] } else { st.heights.clone() };
            if let Ok(out) = make(&hs) {
                r = r.preview(vec![out]);
            }
            r
        }
    }
}

fn extrude_normal(e: &Event) -> Reply {
    let h = e.number("en_height", 1.0);
    let flip = e.flag("en_flip", false);
    built(
        e,
        "Extrude Curve Normal to Surface",
        "select the curves and the surface, Enter",
        || {
            let ms = meshes(e);
            let m = ms
                .first()
                .ok_or("select the surface")?
                .mesh
                .as_ref()
                .unwrap();
            let mut out = Vec::new();
            for c in curves(e) {
                let p = resample(&dense(c), c.closed, 48);
                let mut base = Vec::new();
                let mut top = Vec::new();
                for q in &p {
                    let (s, n) = mesh_point(m, *q).ok_or("surface")?;
                    let n = if flip { scale(n, -1.0) } else { n };
                    base.push(s);
                    top.push(add(s, scale(n, h)));
                }
                if c.closed {
                    base.push(base[0]);
                    top.push(top[0]);
                }
                out.push(surface_through(&[base, top], [1, 3]).ok_or("ribbon")?);
            }
            let k = out.len();
            Ok((out, format!("{k} ribbons {} mm", fmt(h))))
        },
    )
}

/// Unfold a triangle mesh into the plane, triangle by triangle (exact for developable
/// surfaces), then relax the edge lengths a little for the others.
pub fn unfold(m: &Mesh, relax: usize) -> Vec<[f64; 2]> {
    let n = m.vertices.len();
    let mut pos: Vec<Option<[f64; 2]>> = vec![None; n];
    let mut by_edge: std::collections::HashMap<(u32, u32), Vec<usize>> = Default::default();
    for (i, t) in m.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            by_edge.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let v = |i: u32| m.vertices[i as usize];
    let place_third = |pa: [f64; 2], pb: [f64; 2], la: f64, lb: f64, side: f64| -> [f64; 2] {
        let d = ((pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2))
            .sqrt()
            .max(1e-15);
        let x = (la * la - lb * lb + d * d) / (2.0 * d);
        let y = (la * la - x * x).max(0.0).sqrt() * side;
        let ex = [(pb[0] - pa[0]) / d, (pb[1] - pa[1]) / d];
        [pa[0] + ex[0] * x - ex[1] * y, pa[1] + ex[1] * x + ex[0] * y]
    };
    let mut done = vec![false; m.triangles.len()];
    for start in 0..m.triangles.len() {
        if done[start] {
            continue;
        }
        let t = m.triangles[start];
        if pos[t[0] as usize].is_none() || pos[t[1] as usize].is_none() {
            let off = pos
                .iter()
                .flatten()
                .map(|p| p[0])
                .fold(f64::MIN, f64::max)
                .max(0.0)
                + if pos.iter().any(Option::is_some) {
                    1.0
                } else {
                    0.0
                };
            pos[t[0] as usize] = Some([off, 0.0]);
            pos[t[1] as usize] = Some([off + dist(v(t[0]), v(t[1])), 0.0]);
        }
        let mut queue = std::collections::VecDeque::from([start]);
        done[start] = true;
        while let Some(ti) = queue.pop_front() {
            let t = m.triangles[ti];
            // Place any missing vertex from a placed edge (counter-clockwise: left side).
            for k in 0..3 {
                let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
                if let (Some(pa), Some(pb), None) =
                    (pos[a as usize], pos[b as usize], pos[c as usize])
                {
                    pos[c as usize] =
                        Some(place_third(pa, pb, dist(v(a), v(c)), dist(v(b), v(c)), 1.0));
                }
            }
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                for &nb in &by_edge[&(a.min(b), a.max(b))] {
                    if !done[nb] {
                        done[nb] = true;
                        queue.push_back(nb);
                    }
                }
            }
        }
    }
    let mut p: Vec<[f64; 2]> = pos.into_iter().map(|q| q.unwrap_or([0.0, 0.0])).collect();
    // Relax: pull every edge toward its 3D length.
    let edges: Vec<(u32, u32)> = by_edge.keys().copied().collect();
    for _ in 0..relax {
        let mut moved = vec![[0.0; 2]; n];
        let mut count = vec![0.0; n];
        for &(a, b) in &edges {
            let l = dist(v(a), v(b));
            let (pa, pb) = (p[a as usize], p[b as usize]);
            let d = [pb[0] - pa[0], pb[1] - pa[1]];
            let cur = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-15);
            let f = (cur - l) / cur * 0.5;
            moved[a as usize][0] += d[0] * f;
            moved[a as usize][1] += d[1] * f;
            moved[b as usize][0] -= d[0] * f;
            moved[b as usize][1] -= d[1] * f;
            count[a as usize] += 1.0;
            count[b as usize] += 1.0;
        }
        for i in 0..n {
            if count[i] > 0.0 {
                p[i][0] += moved[i][0] / count[i];
                p[i][1] += moved[i][1] / count[i];
            }
        }
    }
    p
}

fn unroll(e: &Event) -> Reply {
    let smash = e.command == SMASH;
    let label = if smash {
        "Smash"
    } else {
        "Unroll Developable Surface"
    };
    let hint = if smash { "select surfaces, Enter" } else { "select surfaces, and curves or points on them to unroll with the surface, Enter" };
    built(e, label, hint, || {
        let mut out = Vec::new();
        let base = e
            .selection
            .iter()
            .map(|s| s.bbox[1][0])
            .fold(f64::MIN, f64::max)
            + 5.0;
        let mut x = base;
        for s in &e.selection {
            // NURBS surfaces unroll to a flat NURBS surface (Rhino's UnrollSrf result): the
            // grid unfolded triangle by triangle (edge lengths kept), then a surface through it.
            if !smash {
                if let Some(f) = s.exact.as_ref().and_then(Srf::of) {
                    // Fine unfolding (chord error ~ L·(θ/n)²/24), fitted through every 6th point.
                    let (n, step) = (192, 6);
                    let g = grid(&f, n, n);
                    let m = grid_mesh(&g);
                    let flat = unfold(&m, 0);
                    let minx = flat.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
                    let miny = flat.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
                    let w = flat.iter().map(|p| p[0]).fold(f64::MIN, f64::max) - minx;
                    let nv = g[0].len();
                    // Every `step`-th point, always including the last row and column.
                    let picks = |len: usize| -> Vec<usize> {
                        let mut v: Vec<usize> = (0..len).step_by(step).collect();
                        if *v.last().unwrap() != len - 1 {
                            v.push(len - 1);
                        }
                        v
                    };
                    let rows: Vec<Vec<Vec3>> = picks(g.len())
                        .into_iter()
                        .map(|i| picks(nv).into_iter().map(|j| {
                            let p = flat[i * nv + j];
                            e.plane.at(x + p[0] - minx, p[1] - miny, 0.0)
                        }).collect())
                        .collect();
                    if let Some(srf) = surface_through(&rows, [3, 3]) {
                        out.push(srf);
                        // Rhino: "Select curves or points to unroll with surface": curves and
                        // points lying on this surface come along to their places on the flat
                        // piece (each sample mapped through the unfolded grid).
                        let to_flat = |q: Vec3| -> Option<Vec3> {
                            let (a, b) = f.closest_st(q);
                            if dist(f.at_st(a, b), q) > 0.01 {
                                return None;
                            }
                            let (fi, fj) = (a * (n - 1) as f64, b * (nv - 1) as f64);
                            let (i, j) = ((fi.floor() as usize).min(n - 2), (fj.floor() as usize).min(nv - 2));
                            let (u, v) = (fi - i as f64, fj - j as f64);
                            let at = |i: usize, j: usize| flat[i * nv + j];
                            let lerp = |p: [f64; 2], q: [f64; 2], t: f64| [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
                            let p = lerp(lerp(at(i, j), at(i, j + 1), v), lerp(at(i + 1, j), at(i + 1, j + 1), v), u);
                            Some(e.plane.at(x + p[0] - minx, p[1] - miny, 0.0))
                        };
                        for o in &e.selection {
                            if std::ptr::eq(o, s) || o.exact.as_ref().and_then(Srf::of).is_some() {
                                continue;
                            }
                            match &o.exact {
                                Some(Geo::Point { point }) => {
                                    if let Some(q) = to_flat(*point) {
                                        out.push(Geo::Point { point: q });
                                    }
                                }
                                Some(_) => {
                                    let pts = dense(o);
                                    let pts = if pts.len() > 120 { resample(&pts, o.closed, 120) } else { pts };
                                    let mapped: Option<Vec<Vec3>> = pts.iter().map(|q| to_flat(*q)).collect();
                                    if let Some(c) = mapped.filter(|m| m.len() >= 2).and_then(|m| om9_addon_geom::interpolate(&m, 3.min(m.len() - 1), o.closed)) {
                                        out.push(c);
                                    }
                                }
                                None => {}
                            }
                        }
                        x += w + 5.0;
                        continue;
                    }
                }
            }
            let m: Mesh = match s.exact.as_ref().and_then(Srf::of) {
                Some(f) => grid_mesh(&grid(&f, 24, 24)),
                None => match &s.mesh {
                    Some(m) => shapes::weld(
                        &Mesh {
                            vertices: m.vertices.clone(),
                            triangles: m.triangles.clone(),
                        },
                        1e-7,
                    ),
                    None => continue,
                },
            };
            let flat = unfold(&m, if smash { 40 } else { 0 });
            let minx = flat.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
            let miny = flat.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
            let w = flat.iter().map(|p| p[0]).fold(f64::MIN, f64::max) - minx;
            let verts: Vec<Vec3> = flat
                .iter()
                .map(|p| e.plane.at(x + p[0] - minx, p[1] - miny, 0.0))
                .collect();
            out.push(Geo::Mesh {
                name: format!("{} flat", s.name),
                vertices: verts,
                triangles: m.triangles.clone(),
                meta: None,
                layer: None,
            });
            x += w + 5.0;
        }
        let k = out.len();
        Ok((out, format!("{k} flattened beside the selection")))
    })
}

/// Rhino 5 SoftEditSrf: Select surface, base point on it, point to move to (UDistance,
/// VDistance, FixEdges, Copy); the control points around move with a smooth fall-off and
/// the base point lands exactly; the surface follows the pointer.
fn soft_edit(e: &Event) -> Reply {
    let flow = Flow::new("Soft Edit Surface")
        .select_prompt("Select surface to edit")
        .point("Base point")
        .point("Point to move to")
        .num("UDistance", e.number("se_u_distance", 5.0))
        .num("VDistance", e.number("se_v_distance", 5.0))
        .toggle("FixEdges", e.flag("se_fix_edges", false))
        .toggle("Copy", false);
    let make = |c: &Collected, to: Option<Vec3>| -> Result<(Selected, Geo), String> {
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a NURBS surface")?;
        let from = c.p(0).ok_or("pick the base point")?;
        let to = c.p(1).or(to).ok_or("pick where to move it")?;
        let (a, b) = f.closest_st(from);
        let base = f.at_st(a, b);
        // Distances along the surface → parameter reach (lengths of the isocurves through it).
        let length = |dir: usize| -> f64 {
            (0..32)
                .map(|k| {
                    let (t0, t1) = (k as f64 / 32.0, (k + 1) as f64 / 32.0);
                    if dir == 0 { dist(f.at_st(t0, b), f.at_st(t1, b)) } else { dist(f.at_st(a, t0), f.at_st(a, t1)) }
                })
                .sum::<f64>()
                .max(1e-9)
        };
        let reach = [c.num(&flow, "UDistance") / length(0), c.num(&flow, "VDistance") / length(1)];
        let g = om9_addon_geom::surfedit::soft_edit(sel.exact.as_ref().unwrap(), a, b, sub(to, base), reach, c.yes(&flow, "FixEdges"))?;
        Ok(((*sel).clone(), g))
    };
    flow.step(
        e,
        &|c, h| Show { geo: make(c, h).map(|m| vec![m.1]).unwrap_or_default(), ..Default::default() },
        &|c| {
            let (sel, g) = make(c, None)?;
            if c.yes(&flow, "Copy") {
                return Ok(Reply::finish(vec![g]).say("Soft Edit Surface: copied"));
            }
            let mut r = Reply::finish(Vec::new()).say("Soft Edit Surface: moved");
            r.replace.push(Replacement { id: sel.id, geo: g });
            Ok(r)
        },
    )
}
/// CutPlane command state: the planes made so far and the points of the one being placed.
#[derive(Serialize, Deserialize, Default, Clone)]
struct CutState {
    /// Objects chosen (Enter pressed on the selection).
    #[serde(default)]
    chosen: bool,
    #[serde(default)]
    three_point: bool,
    /// Picked points of the plane in progress.
    #[serde(default)]
    pts: Vec<Vec3>,
    /// Finished planes, as their four corners.
    #[serde(default)]
    planes: Vec<[Vec3; 4]>,
}

/// Rhino 5 CutPlane: Select objects for cut plane, Start of cut plane, End of cut plane — the
/// prompts repeat for more planes, Enter stops (all planes are made then). Each plane stands
/// square to the CPlane through the line and is large enough to cut every selected object
/// (along the line and across the CPlane). 3Point: start and end of an edge, then the width
/// (a point on the far side, or typed: square up from the CPlane).
fn cutting_plane(e: &Event) -> Reply {
    let label = "Cutting Plane";
    let mut st: CutState = serde_json::from_value(e.state.clone()).unwrap_or_default();
    if matches!(e.input, Input::Start) {
        st = CutState::default();
    }
    let save = |st: &CutState| serde_json::to_value(st).unwrap_or(Value::Null);
    let n = e.plane.normal();
    if let Input::Option { name, value } = &e.input {
        if name.eq_ignore_ascii_case("3Point") {
            st.three_point = match value.as_deref() {
                Some(v) => v.eq_ignore_ascii_case("yes"),
                None => !st.three_point,
            };
            st.pts.clear();
        }
    }
    let options = |st: &CutState| vec![OptionToken::toggle("3Point", st.three_point)];
    let quad = |q: &[Vec3; 4]| surface_on_grid(&[vec![q[0], q[3]], vec![q[1], q[2]]], [1, 1]);
    // The plane through the line a → b, square to the CPlane, over the selection.
    let fitted = |a: Vec3, b: Vec3| -> Option<[Vec3; 4]> {
        let flat = |p: Vec3| sub(p, scale(n, dot(p, n)));
        let (fa, fb) = (flat(a), flat(b));
        let along = sub(fb, fa);
        let len = norm(along);
        if len < 1e-9 {
            return None;
        }
        let dir = scale(along, 1.0 / len);
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        let (mut t0, mut t1) = (0.0f64, len);
        for s in &e.selection {
            let [p, q] = s.bbox;
            for k in 0..8 {
                let c = [
                    if k & 1 == 0 { p[0] } else { q[0] },
                    if k & 2 == 0 { p[1] } else { q[1] },
                    if k & 4 == 0 { p[2] } else { q[2] },
                ];
                lo = lo.min(dot(c, n));
                hi = hi.max(dot(c, n));
                let t = dot(sub(flat(c), fa), dir);
                t0 = t0.min(t);
                t1 = t1.max(t);
            }
        }
        if lo > hi {
            (lo, hi) = (-10.0, 10.0);
        }
        let margin = (hi - lo).max(t1 - t0) * 0.05 + 1.0;
        let (a, b) = (add(fa, scale(dir, t0 - margin)), add(fa, scale(dir, t1 + margin)));
        let up = |p: Vec3, h: f64| add(p, scale(n, h));
        Some([up(a, lo - margin), up(b, lo - margin), up(b, hi + margin), up(a, hi + margin)])
    };
    // 3Point: the rectangle on the edge a → b out to the far point (or `width` straight up).
    let three = |a: Vec3, b: Vec3, far: Result<Vec3, f64>| -> Option<[Vec3; 4]> {
        let e1 = sub(b, a);
        if norm(e1) < 1e-9 {
            return None;
        }
        let u = unit(e1);
        let off = match far {
            Ok(p) => {
                let d = sub(p, a);
                sub(d, scale(u, dot(d, u)))
            }
            Err(w) => {
                let side = sub(n, scale(u, dot(n, u)));
                if norm(side) < 1e-9 {
                    return None;
                }
                scale(unit(side), w)
            }
        };
        if norm(off) < 1e-9 {
            return None;
        }
        Some([a, b, add(b, off), add(a, off)])
    };
    let made = |st: &CutState| -> Vec<Geo> { st.planes.iter().filter_map(quad).collect() };
    let ask = |st: &CutState, extra: Option<[Vec3; 4]>| -> Reply {
        let prompt = match (st.three_point, st.pts.len()) {
            (false, 0) => "Start of cut plane. Press Enter when done",
            (false, _) => "End of cut plane",
            (true, 0) => "Start of edge",
            (true, 1) => "End of edge",
            (true, _) => "Width. Press Enter to use length",
        };
        let mut pv = made(st);
        pv.extend(extra.and_then(|q| quad(&q)));
        let mut r = Reply::ask(prompt, save(st)).want(Want::Point).with_options(options(st)).preview(pv);
        r.rubber_from = st.pts.last().copied();
        r
    };
    if !st.chosen {
        let picked = !e.selection.is_empty();
        if picked && matches!(e.input, Input::Enter | Input::Selection) || picked && matches!(e.input, Input::Start) {
            st.chosen = true;
        } else {
            return Reply::ask("Select objects for cut plane", save(&st)).want(Want::Objects);
        }
        if !matches!(e.input, Input::Point { .. }) {
            return ask(&st, None);
        }
    }
    let finish = |st: &CutState| -> Reply {
        if st.planes.is_empty() {
            return Reply::finish(Vec::new()).say(format!("{label}: no planes"));
        }
        let k = st.planes.len();
        Reply::finish(made(st)).say(format!("{label}: {k} planes"))
    };
    match &e.input {
        Input::Point { point } => {
            st.pts.push(*point);
            let done = if st.three_point { st.pts.len() == 3 } else { st.pts.len() == 2 };
            if done {
                let q = if st.three_point { three(st.pts[0], st.pts[1], Ok(st.pts[2])) } else { fitted(st.pts[0], st.pts[1]) };
                st.pts.clear();
                match q {
                    Some(q) => st.planes.push(q),
                    None => return ask(&st, None).refuse(format!("{label}: the points are too close")),
                }
            }
            ask(&st, None)
        }
        Input::Text { text } if st.three_point && st.pts.len() == 2 => match text.trim().replace(',', ".").parse::<f64>() {
            Ok(w) => match three(st.pts[0], st.pts[1], Err(w)) {
                Some(q) => {
                    st.pts.clear();
                    st.planes.push(q);
                    ask(&st, None)
                }
                None => ask(&st, None).refuse(format!("{label}: the width must not be 0")),
            },
            Err(_) => ask(&st, None).refuse(format!("{text}: type a number")),
        },
        // Width: Enter = the edge's length.
        Input::Enter if st.three_point && st.pts.len() == 2 => {
            let w = dist(st.pts[0], st.pts[1]);
            if let Some(q) = three(st.pts[0], st.pts[1], Err(w)) {
                st.planes.push(q);
            }
            st.pts.clear();
            ask(&st, None)
        }
        Input::Enter => finish(&st),
        Input::Hover { point } => {
            let extra = match (st.three_point, st.pts.len()) {
                (false, 1) => fitted(st.pts[0], *point),
                (true, 2) => three(st.pts[0], st.pts[1], Ok(*point)),
                _ => None,
            };
            ask(&st, extra)
        }
        _ => ask(&st, None),
    }
}

/// Rhino 5 SrfSeam: Select closed surface to adjust seam, New seam location (Direction U / V,
/// the isocurve follows the pointer); exact (the closed direction's seam moved).
fn seam(e: &Event) -> Reply {
    let target = || nurbs_surfaces(e).into_iter().next();
    let closed_dir = target().map_or(0, |(_, f)| {
        if dist(f.at_st(0.0, 0.5), f.at_st(1.0, 0.5)) < 1e-6 { 0 } else { 1 }
    });
    let flow = Flow::new("Adjust Closed Surface Seam")
        .select_prompt("Select closed surface to adjust seam")
        .point("New seam location")
        .choice("Direction", &["U", "V"], if closed_dir == 0 { "U" } else { "V" });
    let dir_of = |c: &Collected| usize::from(c.opt(&flow, "Direction") == "V");
    flow.step(
        e,
        &|c, h| Show {
            geo: match (target(), h) {
                (Some((_, f)), Some(p)) => {
                    let (a, b) = f.closest_st(p);
                    let d = dir_of(c);
                    vec![f.isocurve(d, if d == 0 { a } else { b })]
                }
                _ => vec![],
            },
            ..Default::default()
        },
        &|c| {
            let p = c.p(0).ok_or("pick the new seam location")?;
            let (sel, f) = target().ok_or("select a closed NURBS surface")?;
            let (a, b) = f.closest_st(p);
            let d = dir_of(c);
            let g = om9_addon_geom::surfedit::move_seam(sel.exact.as_ref().unwrap(), d, if d == 0 { a } else { b })?;
            let mut r = Reply::finish(Vec::new()).say("Adjust Closed Surface Seam: moved (exact)");
            r.replace.push(Replacement { id: sel.id.clone(), geo: g });
            Ok(r)
        },
    )
}
/// Rhino 5 SetSurfaceTangent: Select an untrimmed surface edge (a click near it), the base
/// point for the tangent direction, the second direction point (the surface follows the
/// pointer); the edge stays, the surface leaves it along that direction.
fn set_tangent(e: &Event) -> Reply {
    let flow = Flow::new("Set Surface Tangent")
        .select_prompt("Select surface")
        .point("Select untrimmed surface edge")
        .point("Base point for tangent direction")
        .point("Second direction point");
    let make = |c: &Collected, to: Option<Vec3>| -> Result<(String, Geo), String> {
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a NURBS surface")?;
        let at = c.p(0).ok_or("pick near the edge")?;
        let base = c.p(1).ok_or("pick the base point")?;
        let to = c.p(2).or(to).ok_or("pick the direction")?;
        let (dir, end) = nearest_edge(f, at);
        let g = om9_addon_geom::surfedit::set_tangent(sel.exact.as_ref().unwrap(), dir, end > 0.5, sub(to, base))?;
        Ok((sel.id.clone(), g))
    };
    flow.step(
        e,
        &|c, h| Show { geo: make(c, h).map(|m| vec![m.1]).unwrap_or_default(), ..Default::default() },
        &|c| {
            let (id, g) = make(c, None)?;
            let mut r = Reply::finish(Vec::new()).say("Set Surface Tangent: set");
            r.replace.push(Replacement { id, geo: g });
            Ok(r)
        },
    )
}

/// For tests and previews.
pub fn srf_points(g: &Geo, n: usize) -> Vec<Vec3> {
    Srf::of(g)
        .map(|s| grid(&s, n, n).into_iter().flatten().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;

/// Commands whose reply to a pointer move never reads the pointer (their preview follows their
/// state only): checked in their code, which matches no `Input::Hover` point.
const STEADY_PREVIEW: &[&str] = &[SWEEP1, SWEEP2, PROFILE_SWEEP, FOUR_PROFILE_SWEEP, MULTI_RAIL, LOFT, NETWORK, PATCH, REFIT, UNROLL, VAR_FILLET, VAR_BLEND, VAR_CHAMFER];

fn dispatch(e: &Event) -> Result<Reply, String> {
    Ok(match e.command.as_str() {
        SWEEP1 | PROFILE_SWEEP | FOUR_PROFILE_SWEEP => with_surface_edges(e, sweep1),
        SWEEP2 => with_surface_edges(e, sweep2),
        MULTI_RAIL => multi_rail(e),
        PLANAR => planar(e),
        PLANE => plane(e),
        REBUILD => rebuild(e),
        BLEND => blend_srf(e),
        VAR_BLEND => variable_fillet(e),
        LOFT => with_surface_edges(e, loft),
        NETWORK => with_surface_edges(e, network),
        SHRINK => shrink(e),
        EDGE_CURVES => with_surface_edges(e, patch),
        PATCH => patch(e),
        UNTRIM => untrim(e),
        EXTRUDE_ALL => extrude_all(e),
        REVOLVE => revolve(e),
        RAIL_REVOLVE => rail_revolve(e),
        TWEEN => tween(e),
        FILLET | CHAMFER => fillet_srf(e),
        VAR_FILLET | VAR_CHAMFER => variable_fillet(e),
        OFFSET => offset_srf(e),
        VAR_OFFSET => variable_offset(e),
        MERGE => merge(e),
        MATCH => match_srf(e),
        DRAPE => drape(e),
        HEIGHTFIELD => heightfield(e),
        SPLIT_ISO => split(e),
        EXTEND => extend(e),
        EXTRUDE_NORMAL => {
            // The Fin state names its curve / surface; the mesh path's state is empty.
            let fin_state = e.state.get("curve").is_some() || e.state.get("surface").is_some();
            let started = e.state.is_null() || matches!(e.input, Input::Start);
            let exact = e.selection.iter().any(|s| matches!(s.exact, Some(Geo::Surface { .. })));
            if fin_state || (started && (exact || e.selection.is_empty())) {
                fin_cmd(e)
            } else {
                extrude_normal(e)
            }
        }
        UNROLL | SMASH => unroll(e),
        SOFT_EDIT => soft_edit(e),
        CUTTING_PLANE => cutting_plane(e),
        SEAM => seam(e),
        SET_TANGENT => set_tangent(e),
        REFIT => refit(e),
        other => return Err(format!("unknown command {other}")),
    })
}
