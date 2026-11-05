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
    Event, Geo, Input, Interactive, Replacement, Reply, Selected, SelectedMesh, Vec3, Want, add,
    cross, dist, dot, fmt, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show};
use om9_addon_geom::fit::{grid, grid_mesh, sub_grid, surface_on_grid, surface_through};
use om9_addon_geom::mesh::{Frame, Mesh};
use om9_addon_geom::nurbs::{self, Srf};
use om9_addon_geom::path::{Path, Target};
use om9_addon_geom::{interpolate, section, shapes};
use serde_json::json;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

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
        Ok(match e.command.as_str() {
            SWEEP1 | PROFILE_SWEEP | FOUR_PROFILE_SWEEP => sweep1(e),
            SWEEP2 => sweep2(e),
            MULTI_RAIL => multi_rail(e),
            PLANAR => planar(e),
            PLANE => plane(e),
            REBUILD => rebuild(e),
            BLEND | VAR_BLEND => blend(e),
            LOFT => loft(e),
            NETWORK => network(e),
            SHRINK => shrink(e),
            PATCH | EDGE_CURVES => patch(e),
            UNTRIM => untrim(e),
            EXTRUDE_ALL => extrude_all(e),
            REVOLVE => revolve(e),
            RAIL_REVOLVE => rail_revolve(e),
            TWEEN => tween(e),
            FILLET | VAR_FILLET | CHAMFER | VAR_CHAMFER => fillet(e),
            OFFSET | VAR_OFFSET => offset(e),
            MERGE => merge(e),
            MATCH => match_edge(e),
            DRAPE => drape(e),
            HEIGHTFIELD => heightfield(e),
            SPLIT_ISO => split(e),
            EXTEND => extend(e),
            EXTRUDE_NORMAL => extrude_normal(e),
            UNROLL | SMASH => unroll(e),
            SOFT_EDIT => soft_edit(e),
            CUTTING_PLANE => cutting_plane(e),
            SEAM => seam(e),
            SET_TANGENT => set_tangent(e),
            REFIT => refit(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- shared ----------------------------------------------------------------------------------

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

fn sweep1(e: &Event) -> Reply {
    let label = match e.command.as_str() {
        PROFILE_SWEEP => "Profile Sweep",
        FOUR_PROFILE_SWEEP => "Four Profile Sweep",
        _ => "Sweep 1 Rail",
    };
    let stations = e.number("sw_stations", 48.0).round().clamp(4.0, 400.0) as usize;
    built(e, label, "select the rail and the profiles, Enter", || {
        let (rail, profiles) = rail_and_profiles(e)?;
        if e.command == FOUR_PROFILE_SWEEP && profiles.len() != 4 {
            return Err("select the ring rail and four profiles".into());
        }
        // Closed rails (rings) keep the profiles' up pointing away from the rail's middle.
        let up = if rail.closed {
            None
        } else {
            Some(e.plane.normal())
        };
        let rows =
            sweep_rows(&dense(rail), rail.closed, &profiles, up, stations, 24).ok_or("rail")?;
        Ok((
            vec![surface(&rows)?],
            format!("{} profiles along the rail", profiles.len()),
        ))
    })
}

fn sweep2(e: &Event) -> Reply {
    let stations = e.number("sw_stations", 48.0).round().clamp(4.0, 400.0) as usize;
    let keep_height = e.flag("sw2_maintain_height", false);
    built(
        e,
        "Sweep 2 Rails",
        "select two rails and the profiles, Enter",
        || {
            let cs = curves(e);
            if cs.len() < 3 {
                return Err("select two rails and at least one profile".into());
            }
            // The two longest curves are the rails.
            let len = |s: &Selected| Path::new(&s.points, s.closed).map_or(0.0, |p| p.length());
            let mut by_len = cs.clone();
            by_len.sort_by(|a, b| len(b).total_cmp(&len(a)));
            let (ra, rb) = (by_len[0], by_len[1]);
            let profiles: Vec<&Selected> = cs
                .iter()
                .copied()
                .filter(|c| c.id != ra.id && c.id != rb.id)
                .collect();
            let closed = ra.closed && rb.closed;
            let a = Path::new(&dense(ra), ra.closed).ok_or("rail")?;
            let mut bpts = dense(rb);
            if !closed && dist(a.at(0.0), bpts[0]) > dist(a.at(0.0), *bpts.last().unwrap()) {
                bpts.reverse();
            }
            let b = Path::new(&bpts, rb.closed).ok_or("rail")?;
            let frame_at = |t: f64| -> (Frame, f64) {
                let (pa, pb) = (a.at(t * a.length()), b.at(t * b.length()));
                let tan = unit(add(a.tangent(t * a.length()), b.tangent(t * b.length())));
                let x = sub(pb, pa);
                let w = norm(x).max(1e-9);
                // x along the chord, y across (up), z along the rails.
                let y = unit(cross(tan, x));
                (
                    Frame {
                        origin: pa,
                        x: unit(x),
                        y,
                        z: unit(cross(unit(x), y)),
                    },
                    w,
                )
            };
            let n = 24;
            let mut locals = Vec::new();
            let mut at = Vec::new();
            for p in &profiles {
                let pts = dense(p);
                let c = scale(
                    pts.iter().fold([0.0; 3], |s, &q| add(s, q)),
                    1.0 / pts.len() as f64,
                );
                let t = a.closest(c) / a.length();
                let (f, w) = frame_at(t);
                let mut l: Vec<Vec3> = resample(&pts, p.closed, n)
                    .iter()
                    .map(|&q| f.local(q))
                    .collect();
                if !p.closed && l[0][0] > l[n - 1][0] {
                    l.reverse();
                }
                // In chord units across, mm (or chord units) up.
                for q in &mut l {
                    q[0] /= w;
                    if !keep_height {
                        q[1] /= w;
                    }
                    q[2] = 0.0;
                }
                locals.push(l);
                at.push(t);
            }
            let count = if closed { stations } else { stations + 1 };
            let mut rows: Vec<Vec<Vec3>> = (0..count)
                .map(|i| {
                    let t = i as f64 / stations as f64;
                    let (f, w) = frame_at(t);
                    profile_at(&locals, &at, t, closed.then_some(1.0))
                        .iter()
                        .map(|q| {
                            f.point([q[0] * w, if keep_height { q[1] } else { q[1] * w }, 0.0])
                        })
                        .collect()
                })
                .collect();
            if closed {
                rows.push(rows[0].clone());
            }
            let rows = closed_rows(rows, profiles.iter().all(|p| p.closed));
            Ok((
                vec![surface(&rows)?],
                format!("{} profiles between two rails", profiles.len()),
            ))
        },
    )
}

fn multi_rail(e: &Event) -> Reply {
    let stations = e.number("sw_stations", 48.0).round().clamp(4.0, 400.0) as usize;
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
            let first = Path::new(&dense(cs[0]), cs[0].closed).ok_or("rail")?;
            let paths: Vec<Path> = cs
                .iter()
                .map(|c| {
                    let mut p = dense(c);
                    if !c.closed
                        && dist(first.at(0.0), p[0]) > dist(first.at(0.0), *p.last().unwrap())
                    {
                        p.reverse();
                    }
                    Path::new(&p, c.closed).unwrap()
                })
                .collect();
            let closed = cs.iter().all(|c| c.closed);
            let count = if closed { stations } else { stations + 1 };
            let mut rows: Vec<Vec<Vec3>> = Vec::new();
            for i in 0..count {
                let t = i as f64 / stations as f64;
                let pts: Vec<Vec3> = paths.iter().map(|p| p.at(t * p.length())).collect();
                let g = interpolate(&pts, 3, closed_profile).ok_or("profile")?;
                let mut row = nurbs::samples(&g, 32);
                if closed_profile {
                    row.pop();
                    row.push(row[0]);
                }
                rows.push(row);
            }
            if closed {
                rows.push(rows[0].clone());
            }
            Ok((
                vec![surface(&rows)?],
                format!("profile through {} rails", cs.len()),
            ))
        },
    )
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
            if cs.len() < 2 {
                return Err("select two or more sections".into());
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

fn network(e: &Event) -> Reply {
    built(
        e,
        "Curve Network",
        "select the curves of both directions, Enter",
        || {
            let cs = curves(e);
            if cs.len() < 3 {
                return Err("select at least three curves".into());
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
            let mut out = Vec::new();
            for (i, o_) in outlines.iter().enumerate() {
                let depth = outlines
                    .iter()
                    .enumerate()
                    .filter(|(j, other)| *j != i && inside(o_[0], other))
                    .count();
                if depth % 2 == 1 {
                    continue;
                }
                let holes: Vec<Vec<[f64; 2]>> = outlines
                    .iter()
                    .enumerate()
                    .filter(|(j, h)| {
                        *j != i && inside(h[0], o_) && {
                            outlines
                                .iter()
                                .enumerate()
                                .filter(|(k, other)| *k != *j && inside(h[0], other))
                                .count()
                                == depth + 1
                        }
                    })
                    .map(|(_, h)| h.clone())
                    .collect();
                let poly = shapes::bridge_holes(o_, &holes);
                let tris = shapes::triangulate(&poly);
                let m = Mesh {
                    vertices: poly.iter().map(|p| f.point([p[0], p[1], 0.0])).collect(),
                    triangles: tris
                        .iter()
                        .map(|t| [t[0] as u32, t[1] as u32, t[2] as u32])
                        .collect(),
                };
                out.push(m.geo("Planar surface", None, None));
            }
            let k = out.len();
            Ok((out, format!("{k} planar surfaces")))
        },
    )
}

fn plane(e: &Event) -> Reply {
    let flow = Flow::new("Plane")
        .no_selection()
        .point("First corner")
        .point("Opposite corner");
    let make = |a: Vec3, b: Vec3| -> Geo {
        let (la, lb) = (e.plane.local(a), e.plane.local(b));
        let p = |u: f64, v: f64| e.plane.at(u, v, la[2]);
        surface_on_grid(
            &[
                vec![p(la[0], la[1]), p(la[0], lb[1])],
                vec![p(lb[0], la[1]), p(lb[0], lb[1])],
            ],
            [1, 1],
        )
        .unwrap()
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: match (c.p(0), h) {
                (Some(a), Some(b)) => vec![make(a, b)],
                _ => Vec::new(),
            },
            ..Default::default()
        },
        &|c| Ok(Reply::finish(vec![make(c.p(0).unwrap(), c.p(1).unwrap())]).say("Plane: done")),
    )
}

// ---- revolve ---------------------------------------------------------------------------------

/// The NURBS of a curve: exact when it is one, else fitted through its samples.
fn curve_nurbs(s: &Selected) -> Option<(usize, Vec<Vec3>, Vec<f64>, Vec<f64>)> {
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

fn revolve(e: &Event) -> Reply {
    let angle = e.number("rev_angle", 360.0).to_radians();
    let start = e.number("rev_start", 0.0).to_radians();
    let flow = Flow::new("Revolve")
        .point("Start of the revolve axis")
        .point("End of the revolve axis");
    let make = |c: &Collected, end: Option<Vec3>| -> Result<Vec<Geo>, String> {
        let (Some(a), Some(b)) = (c.p(0), c.p(1).or(end)) else {
            return Err("pick the axis".into());
        };
        if dist(a, b) < 1e-9 {
            return Err("the axis needs two different points".into());
        }
        let out: Vec<Geo> = curves(e)
            .iter()
            .filter_map(|s| curve_nurbs(s))
            .map(|p| revolve_surface(p, a, sub(b, a), start, angle))
            .collect();
        if out.is_empty() {
            Err("select the profile curves".into())
        } else {
            Ok(out)
        }
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let out = make(c, None)?;
            let n = out.len();
            Ok(Reply::finish(out).say(format!(
                "Revolve: {n} surfaces, {}°",
                fmt(angle.to_degrees())
            )))
        },
    )
}

fn rail_revolve(e: &Event) -> Reply {
    let flow = Flow::new("Rail Revolve")
        .point("Start of the axis")
        .point("End of the axis");
    let make = |c: &Collected, end: Option<Vec3>| -> Result<Geo, String> {
        let (Some(o), Some(b)) = (c.p(0), c.p(1).or(end)) else {
            return Err("pick the axis".into());
        };
        let cs = curves(e);
        if cs.len() != 2 {
            return Err("select the profile, then the rail".into());
        }
        let ax = unit(sub(b, o));
        let radial = |p: Vec3| {
            let c = add(o, scale(ax, dot(sub(p, o), ax)));
            sub(p, c)
        };
        let (profile, rail) = (cs[0], cs[1]);
        let prof = resample(&dense(profile), profile.closed, 24);
        let rail_pts = dense(rail);
        // The rail's radius around the axis by angle (from the profile's plane).
        let r0 = unit(radial(
            prof.iter()
                .copied()
                .max_by(|p, q| norm(radial(*p)).total_cmp(&norm(radial(*q))))
                .unwrap(),
        ));
        let y0 = cross(ax, r0);
        let ang = |p: Vec3| {
            let r = radial(p);
            dot(r, y0).atan2(dot(r, r0)).rem_euclid(TAU)
        };
        let rail_r = |th: f64| -> f64 {
            let mut best = (f64::MAX, 0.0);
            for &p in &rail_pts {
                let d = (ang(p) - th).abs();
                let d = d.min(TAU - d);
                if d < best.0 {
                    best = (d, norm(radial(p)));
                }
            }
            best.1
        };
        let base = rail_r(0.0).max(1e-9);
        let k = 48;
        let mut rows: Vec<Vec<Vec3>> = (0..k)
            .map(|i| {
                let th = TAU * i as f64 / k as f64;
                let s = rail_r(th) / base;
                prof.iter()
                    .map(|&p| {
                        let c = add(o, scale(ax, dot(sub(p, o), ax)));
                        let r = sub(p, c);
                        let y = cross(ax, r);
                        add(c, scale(add(scale(r, th.cos()), scale(y, th.sin())), s))
                    })
                    .collect()
            })
            .collect();
        rows.push(rows[0].clone());
        surface(&closed_rows(rows, profile.closed))
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).into_iter().collect(),
            ..Default::default()
        },
        &|c| Ok(Reply::finish(vec![make(c, None)?]).say("Rail Revolve: done")),
    )
}

fn extrude_all(e: &Event) -> Reply {
    let d = e.number("ea_distance", 2.0);
    let both = e.flag("ea_both_sides", false);
    built(e, "Extrude All", "select curves, Enter", || {
        let n = e.plane.normal();
        let out: Vec<Geo> = curves(e)
            .iter()
            .filter_map(|c| {
                let mut p = resample(&dense(c), c.closed, 48);
                if c.closed {
                    p.push(p[0]);
                }
                let lo = if both { scale(n, -d) } else { [0.0; 3] };
                let rows = vec![
                    p.iter().map(|&q| add(q, lo)).collect::<Vec<_>>(),
                    p.iter().map(|&q| add(q, scale(n, d))).collect(),
                ];
                surface_through(&rows, [1, 3])
            })
            .collect();
        let k = out.len();
        Ok((out, format!("{k} extrusions {} mm", fmt(d))))
    })
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
        let (ga, gb) = (grid(&ss[0].1, 20, 20), grid(&ss[1].1, 20, 20));
        let mut out = Vec::new();
        for k in 1..=count {
            let t = k as f64 / (count + 1) as f64;
            let rows: Vec<Vec<Vec3>> = ga
                .iter()
                .zip(&gb)
                .map(|(ra, rb)| {
                    ra.iter()
                        .zip(rb)
                        .map(|(p, q)| add(*p, scale(sub(*q, *p), t)))
                        .collect()
                })
                .collect();
            out.push(surface(&rows)?);
        }
        Ok((out, format!("{count} surfaces between")))
    })
}

fn merge(e: &Event) -> Reply {
    replaced(
        e,
        "Merge Surfaces",
        "select two surfaces sharing an edge, Enter",
        || {
            let ss = nurbs_surfaces(e);
            if ss.len() != 2 {
                return Err("select two NURBS surfaces".into());
            }
            let (a, b) = (&ss[0].1, &ss[1].1);
            // Orient both so a's s = 1 edge meets b's s = 0 edge.
            let n = 24;
            let orient = |s: &Srf, other: &Srf| -> Vec<Vec<Vec3>> {
                let (dir, at) = nearest_edge(s, other.at_st(0.5, 0.5));
                let g = grid(s, n, n);
                let mut rows: Vec<Vec<Vec3>> = if dir == 0 {
                    g
                } else {
                    (0..n).map(|j| g.iter().map(|r| r[j]).collect()).collect()
                };
                if at < 0.5 {
                    rows.reverse();
                }
                rows
            };
            let ra = orient(a, b);
            let mut rb = orient(b, a);
            rb.reverse();
            // Match the shared edge's direction.
            if dist(ra[n - 1][0], rb[0][0]) > dist(ra[n - 1][0], rb[0][n - 1]) {
                for r in &mut rb {
                    r.reverse();
                }
            }
            if dist(ra[n - 1][0], rb[0][0]) > 0.01 * dist(ra[0][0], ra[n - 1][n - 1]).max(1.0) {
                return Err("the surfaces do not share an edge".into());
            }
            let mut rows = ra;
            rows.extend(rb.into_iter().skip(1));
            let g = surface(&rows)?;
            let mut r_items = vec![(ss[0].0.id.clone(), g)];
            let _ = &mut r_items;
            Ok((r_items, Vec::new(), "one surface".into()))
        },
    )
    .with_delete(
        |r, e| {
            if r.done {
                r.delete = nurbs_surfaces(e)
                    .get(1)
                    .map(|s| vec![s.0.id.clone()])
                    .unwrap_or_default()
            }
        },
        e,
    )
}

trait AfterFinish {
    fn with_delete(self, f: impl FnOnce(&mut Reply, &Event), e: &Event) -> Self;
}

impl AfterFinish for Reply {
    fn with_delete(mut self, f: impl FnOnce(&mut Reply, &Event), e: &Event) -> Self {
        f(&mut self, e);
        self
    }
}

fn match_edge(e: &Event) -> Reply {
    let tangent = e.text("sm_continuity", "tangency") == "tangency";
    replaced(
        e,
        "Match Surface",
        "select the surface to change, then the target surface, Enter",
        || {
            let ss = nurbs_surfaces(e);
            if ss.len() != 2 {
                return Err("select two NURBS surfaces".into());
            }
            let (a, b) = (&ss[0].1, &ss[1].1);
            let n = 24;
            let (dir, at) = nearest_edge(a, b.at_st(0.5, 0.5));
            let g = grid(a, n, n);
            let mut rows: Vec<Vec<Vec3>> = if dir == 0 {
                g
            } else {
                (0..n).map(|j| g.iter().map(|r| r[j]).collect()).collect()
            };
            if at < 0.5 {
                rows.reverse();
            }
            // The target edge (nearest to a's edge), oriented like it.
            let (db, tb) = nearest_edge(b, rows[n - 1][n / 2]);
            let (mut pb, mut nb) = edge_rows(b, db, tb, n);
            if dist(rows[n - 1][0], pb[0]) > dist(rows[n - 1][0], pb[n - 1]) {
                pb.reverse();
                nb.reverse();
            }
            for j in 0..n {
                rows[n - 1][j] = pb[j];
                if tangent {
                    let h = dist(rows[n - 2][j], rows[n - 3][j]);
                    rows[n - 2][j] = sub(pb[j], scale(nb[j], h));
                }
            }
            let g = surface(&rows)?;
            Ok((
                vec![(ss[0].0.id.clone(), g)],
                Vec::new(),
                if tangent {
                    "edge matched with tangency".into()
                } else {
                    "edge matched".into()
                },
            ))
        },
    )
}

// ---- drape, heightfield ----------------------------------------------------------------------

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
    replaced(e, "Rebuild Surface", "select NURBS surfaces, Enter", || {
        let items: Vec<(String, Geo)> = nurbs_surfaces(e)
            .iter()
            .filter_map(|(s, f)| {
                surface_through(&grid(f, nu, nv), [deg, deg]).map(|g| (s.id.clone(), g))
            })
            .collect();
        let k = items.len();
        Ok((
            items,
            Vec::new(),
            format!("{k} surfaces, {nu} × {nv} points, degree {deg}"),
        ))
    })
}

/// Largest distance between two surfaces sampled on the same grid.
fn deviation(a: &Srf, b: &Srf) -> f64 {
    let mut m: f64 = 0.0;
    for i in 0..=12 {
        for j in 0..=12 {
            let (s, t) = (i as f64 / 12.0, j as f64 / 12.0);
            m = m.max(dist(a.at_st(s, t), b.at_st(s, t)));
        }
    }
    m
}

fn refit(e: &Event) -> Reply {
    let tol = e.number("srf_tolerance", 0.01).max(1e-6);
    replaced(
        e,
        "Refit to Tolerance",
        "select NURBS surfaces, Enter",
        || {
            let mut items = Vec::new();
            for (s, f) in nurbs_surfaces(e) {
                let mut n = 4;
                let mut best = None;
                while n <= 128 {
                    if let Some(g) = surface_through(&grid(&f, n, n), [3, 3]) {
                        let ok = Srf::of(&g).is_some_and(|x| deviation(&f, &x) <= tol);
                        best = Some(g);
                        if ok {
                            break;
                        }
                    }
                    n *= 2;
                }
                if let Some(g) = best {
                    items.push((s.id.clone(), g));
                }
            }
            let k = items.len();
            Ok((
                items,
                Vec::new(),
                format!("{k} surfaces within {} mm", fmt(tol)),
            ))
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
                if let Some(g) = surface_through(
                    &sub_grid(&f, [lo[0], hi[0]], [lo[1], hi[1]], 16, 16),
                    [3, 3],
                ) {
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

fn split(e: &Event) -> Reply {
    let flow = Flow::new("Split at Isocurve")
        .choice("Direction", &["U", "V"], "U")
        .point("Point on the surface where to split");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let p = c.p(0).unwrap();
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a NURBS surface")?;
        let (a, b) = f.closest_st(p);
        let (first, second) = if c.opt(&flow, "Direction") == "U" {
            (
                sub_grid(f, [0.0, a], [0.0, 1.0], 16, 16),
                sub_grid(f, [a, 1.0], [0.0, 1.0], 16, 16),
            )
        } else {
            (
                sub_grid(f, [0.0, 1.0], [0.0, b], 16, 16),
                sub_grid(f, [0.0, 1.0], [b, 1.0], 16, 16),
            )
        };
        let mut r = Reply::finish(vec![surface(&second)?]).say("Split at Isocurve: two surfaces");
        r.replace.push(Replacement {
            id: sel.id.clone(),
            geo: surface(&first)?,
        });
        Ok(r)
    })
}

fn extend(e: &Event) -> Reply {
    let flow = Flow::new("Extend Surface")
        .number("Length to extend", 2.0)
        .point("Pick near the edge to extend");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let len = c.n(0).unwrap();
        let p = c.p(0).unwrap();
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a NURBS surface")?;
        let (dir, at) = nearest_edge(f, p);
        let n = 20;
        let g = grid(f, n, n);
        let mut rows: Vec<Vec<Vec3>> = if dir == 0 {
            g
        } else {
            (0..n).map(|j| g.iter().map(|r| r[j]).collect()).collect()
        };
        if at < 0.5 {
            rows.reverse();
        }
        // Continue straight along the surface's direction at the edge.
        let last = rows[n - 1].clone();
        let prev = rows[n - 2].clone();
        let steps = 4;
        for k in 1..=steps {
            rows.push(
                last.iter()
                    .zip(&prev)
                    .map(|(a, b)| add(*a, scale(unit(sub(*a, *b)), len * k as f64 / steps as f64)))
                    .collect(),
            );
        }
        let mut r = Reply::finish(Vec::new()).say(format!("Extend Surface: {} mm", fmt(len)));
        r.replace.push(Replacement {
            id: sel.id.clone(),
            geo: surface(&rows)?,
        });
        Ok(r)
    })
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
    built(e, label, "select surfaces, Enter", || {
        let mut out = Vec::new();
        let base = e
            .selection
            .iter()
            .map(|s| s.bbox[1][0])
            .fold(f64::MIN, f64::max)
            + 5.0;
        let mut x = base;
        for s in &e.selection {
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

fn soft_edit(e: &Event) -> Reply {
    let flow = Flow::new("Soft Edit Surface")
        .number("Falloff (fraction of the surface)", 0.3)
        .point("Point on the surface to move")
        .point("Where to move it");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let fall = c.n(0).unwrap().clamp(0.01, 2.0);
        let (from, to) = (c.p(0).unwrap(), c.p(1).unwrap());
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a NURBS surface")?;
        let (a, b) = f.closest_st(from);
        let mv = sub(to, from);
        let n = 24;
        let rows: Vec<Vec<Vec3>> = grid(f, n, n)
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                r.into_iter()
                    .enumerate()
                    .map(|(j, p)| {
                        let (s, t) = (i as f64 / (n - 1) as f64, j as f64 / (n - 1) as f64);
                        let d = ((s - a).powi(2) + (t - b).powi(2)).sqrt();
                        let w = if d >= fall {
                            0.0
                        } else {
                            0.5 + 0.5 * (PI * d / fall).cos()
                        };
                        add(p, scale(mv, w))
                    })
                    .collect()
            })
            .collect();
        let mut r = Reply::finish(Vec::new()).say("Soft Edit Surface: moved");
        r.replace.push(Replacement {
            id: sel.id.clone(),
            geo: surface(&rows)?,
        });
        Ok(r)
    })
}

fn cutting_plane(e: &Event) -> Reply {
    let flow = Flow::new("Cutting Plane")
        .point("Start of the cut line")
        .point("End of the cut line");
    let make = |c: &Collected, end: Option<Vec3>| -> Option<Geo> {
        let (a, b) = (c.p(0)?, c.p(1).or(end)?);
        let n = e.plane.normal();
        // Through the line, square to the CPlane, tall enough for the selection.
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for s in &e.selection {
            for p in [s.bbox[0], s.bbox[1]] {
                lo = lo.min(dot(p, n));
                hi = hi.max(dot(p, n));
            }
        }
        if lo > hi {
            (lo, hi) = (-10.0, 10.0);
        }
        let margin = (hi - lo) * 0.1 + 1.0;
        let up = |p: Vec3, h: f64| add(sub(p, scale(n, dot(p, n))), scale(n, h));
        surface_on_grid(
            &[
                vec![up(a, lo - margin), up(a, hi + margin)],
                vec![up(b, lo - margin), up(b, hi + margin)],
            ],
            [1, 1],
        )
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            Ok(Reply::finish(vec![make(c, None).ok_or("pick the line")?])
                .say("Cutting Plane: done"))
        },
    )
}

fn seam(e: &Event) -> Reply {
    let flow =
        Flow::new("Adjust Closed Surface Seam").point("Point on the surface for the new seam");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let p = c.p(0).unwrap();
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a closed NURBS surface")?;
        let closed_u = dist(f.at_st(0.0, 0.5), f.at_st(1.0, 0.5)) < 1e-6;
        let closed_v = dist(f.at_st(0.5, 0.0), f.at_st(0.5, 1.0)) < 1e-6;
        if !closed_u && !closed_v {
            return Err("the surface is not closed".into());
        }
        let (a, b) = f.closest_st(p);
        let n = 32;
        let rows: Vec<Vec<Vec3>> = (0..=n)
            .map(|i| {
                (0..=n)
                    .map(|j| {
                        let (mut s, mut t) = (i as f64 / n as f64, j as f64 / n as f64);
                        if closed_u {
                            s = (s + a).rem_euclid(1.0);
                        } else {
                            t = (t + b).rem_euclid(1.0);
                        }
                        f.at_st(s, t)
                    })
                    .collect()
            })
            .collect();
        let mut r = Reply::finish(Vec::new()).say("Adjust Closed Surface Seam: moved");
        r.replace.push(Replacement {
            id: sel.id.clone(),
            geo: surface(&rows)?,
        });
        Ok(r)
    })
}

fn set_tangent(e: &Event) -> Reply {
    let flow = Flow::new("Set Surface Tangent")
        .point("Pick near the edge")
        .point("Tangent direction (a point the edge should lean toward)");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (p, q) = (c.p(0).unwrap(), c.p(1).unwrap());
        let ss = nurbs_surfaces(e);
        let (sel, f) = ss.first().ok_or("select a NURBS surface")?;
        let (dir, at) = nearest_edge(f, p);
        let n = 24;
        let g = grid(f, n, n);
        let mut rows: Vec<Vec<Vec3>> = if dir == 0 {
            g
        } else {
            (0..n).map(|j| g.iter().map(|r| r[j]).collect()).collect()
        };
        if at < 0.5 {
            rows.reverse();
        }
        let edge_mid = rows[n - 1][n / 2];
        let tdir = unit(sub(edge_mid, q));
        for j in 0..n {
            let h = dist(rows[n - 1][j], rows[n - 2][j]);
            rows[n - 2][j] = add(rows[n - 1][j], scale(tdir, h));
        }
        let mut r = Reply::finish(Vec::new()).say("Set Surface Tangent: done");
        r.replace.push(Replacement {
            id: sel.id.clone(),
            geo: surface(&rows)?,
        });
        Ok(r)
    })
}

/// For tests and previews.
pub fn srf_points(g: &Geo, n: usize) -> Vec<Vec3> {
    Srf::of(g)
        .map(|s| grid(&s, n, n).into_iter().flatten().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
