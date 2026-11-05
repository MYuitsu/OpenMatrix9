// SPDX-License-Identifier: LGPL-2.1-or-later
//! The rest of the Matrix-style **Settings** menu for OM9 (SubD bezel, pull object to rail,
//! prong editor, prong on surface, bead on surface, metal from gems) and the **Boolean
//! Builder** of the Cutters / Tools menus.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, ObjectMatrix, Replacement, Reply, Selected, SelectedMesh,
    TransformOp, Vec3, Want, add, cross, dist, dot, fmt, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{load, save, selected_gems};
use om9_addon_geom::csg::{self, Op};
use om9_addon_geom::gems::PlacedGem;
use om9_addon_geom::mesh::{self, Frame, Mesh};
use om9_addon_geom::path::{Path, Target, frame_matrix};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::f64::consts::PI;

pub const SUBD_BEZEL: &str = "ClayooCreationClayooBezel";
pub const PULL_TO_RAIL: &str = "BuilderPullToRail";
pub const PRONG_EDITOR: &str = "BuilderProngEditor";
pub const PRONG_ON_SURFACE: &str = "BuilderProngOnSurface";
pub const BEAD_ON_SURFACE: &str = "BuilderBeadOnSurface";
pub const METAL_PIECE: &str = "BuilderMetalPiece";
pub const BOOLEAN_BUILDER: &str = "BuilderBooleanBuilder";

pub struct SettingTools;

impl Interactive for SettingTools {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        SUBD_BEZEL,
        PULL_TO_RAIL,
        PRONG_EDITOR,
        PRONG_ON_SURFACE,
        BEAD_ON_SURFACE,
        METAL_PIECE,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        let mut r = match e.command.as_str() {
            SUBD_BEZEL => subd_bezel(e),
            PULL_TO_RAIL => pull_to_rail(e),
            PRONG_EDITOR => prong_editor(e),
            PRONG_ON_SURFACE | BEAD_ON_SURFACE => on_surface(e),
            METAL_PIECE => metal_piece(e),
            other => return Err(format!("unknown command {other}")),
        };
        if r.done && matches!(e.command.as_str(), SUBD_BEZEL | METAL_PIECE) {
            r.records.extend(om9_addon_geom::cmd::remember_style(e));
        }
        Ok(r)
    }
}

fn to_mesh(m: &SelectedMesh) -> Mesh {
    Mesh {
        vertices: m.vertices.clone(),
        triangles: m.triangles.clone(),
    }
}

fn solids(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.gem.is_none() && s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty()))
        .collect()
}

/// Preview on edits, create on Enter.
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
                Reply::ask(
                    format!("{label}: {what}. Adjust in the builder, Enter to create"),
                    json!({}),
                )
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

/// The Boolean Builder (Cutters and Tools menus).
pub struct BooleanTool;

impl Interactive for BooleanTool {
    const SESSION_COMMANDS: &'static [&'static str] = &[BOOLEAN_BUILDER];
    fn step(e: &Event) -> Result<Reply, String> {
        match e.command.as_str() {
            BOOLEAN_BUILDER => Ok(boolean_builder(e)),
            other => Err(format!("unknown command {other}")),
        }
    }
}

// ---- SubD (Clayoo) bezel ---------------------------------------------------------------------

/// A smooth bezel around a stone: a rounded wall section lofted around the girdle outline.
/// `height` below the girdle, `top` and `base` wall thickness, `angle` of the wall (degrees,
/// outward at the base), `placement` up/down, `scale` of the seat (1 = on the girdle).
pub fn subd_bezel_for(
    g: &PlacedGem,
    height: f64,
    top: f64,
    base: f64,
    angle: f64,
    placement: f64,
    seat: f64,
    lip: f64,
) -> Mesh {
    let o = mesh::scaled(&g.gem.outline(64), seat, seat);
    let (gt, _) = g.gem.girdle();
    let z_top = gt + lip + placement;
    let z_bottom = z_top - lip - height;
    let lean = height * angle.to_radians().tan();
    // The wall section: (offset from the seat outline, z), around from the inside top,
    // rounded at the top with a half circle, straight down outside, across the base.
    let mut section: Vec<(f64, f64)> = Vec::new();
    let r = top / 2.0;
    for k in 0..=8 {
        let a = PI * k as f64 / 8.0;
        section.push((r - r * a.cos(), z_top - r + r * a.sin()));
    }
    section.reverse();
    section.push((base + lean, z_bottom));
    section.push((-0.05 * base, z_bottom));
    let rings: Vec<Vec<Vec3>> = section
        .iter()
        .map(|&(d, z)| {
            mesh::offset_outline(&o, d)
                .iter()
                .map(|p| g.frame.point([p[0], p[1], z]))
                .collect()
        })
        .collect();
    mesh::loft_rings(&rings, true).unwrap_or_default()
}

fn subd_bezel(e: &Event) -> Reply {
    let h = e.number("sdb_height", 2.0);
    let top = e.number("sdb_top", 0.5);
    let base = e.number("sdb_base", 0.6);
    let angle = e.number("sdb_angle", 10.0);
    let placement = e.number("sdb_placement", 0.0);
    let seat = e.number("sdb_scale", 100.0) / 100.0;
    let lip = e.number("sdb_lip", 0.3);
    let cut_rail = e.flag("sdb_boolean", true);
    built(
        e,
        "SubD Bezel",
        "select stones (and the finger rail to trim against), Enter",
        || {
            let (gems, _) = selected_gems(e);
            if gems.is_empty() {
                return Err("select a stone".into());
            }
            let rail: Option<Mesh> = solids(e).first().map(|s| to_mesh(s.mesh.as_ref().unwrap()));
            let out: Vec<Geo> = gems
                .iter()
                .map(|g| {
                    let mut b = subd_bezel_for(g, h, top, base, angle, placement, seat, lip);
                    if cut_rail && let Some(r) = &rail {
                        b = csg::boolean(&b, r, Op::Difference);
                    }
                    b.geo("SubD bezel", None, None)
                })
                .collect();
            let n = out.len();
            Ok((out, format!("{n} bezels")))
        },
    )
}

// ---- Pull Object to Rail ---------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct Pulled {
    objects: Vec<(String, SelectedMesh)>,
}

/// Pull the lower part of a mesh down (along −`up`) onto `rail`: vertices in the bottom
/// `zone` of the object's height move fully at the bottom and fade out at the top of the zone.
pub fn pull_mesh(
    m: &SelectedMesh,
    rail: &SelectedMesh,
    up: Vec3,
    zone: f64,
    gap: f64,
) -> SelectedMesh {
    let hs: Vec<f64> = m.vertices.iter().map(|&v| dot(v, up)).collect();
    let (lo, hi) = (
        hs.iter().cloned().fold(f64::MAX, f64::min),
        hs.iter().cloned().fold(f64::MIN, f64::max),
    );
    let top = lo + (hi - lo) * zone.clamp(0.01, 1.0);
    let target = Target::new(rail, up);
    let mut out = m.clone();
    for (v, h) in out.vertices.iter_mut().zip(&hs) {
        if *h > top {
            continue;
        }
        let w = ((top - h) / (top - lo).max(1e-9)).clamp(0.0, 1.0);
        // The bottom lands on the rail under each vertex; the zone above fades out.
        let shift = target
            .as_ref()
            .and_then(|t| t.drop(*v))
            .map_or(0.0, |(q, _)| dot(q, up) + gap - lo);
        *v = add(*v, scale(up, shift * w));
    }
    out
}

fn pull_to_rail(e: &Event) -> Reply {
    let label = "Pull Object to Rail";
    let zone = e.number("ptr_zone", 30.0) / 100.0;
    let gap = e.number("ptr_gap", 0.0);
    let mut st: Pulled = load(e);
    let objs = solids(e);
    if st.objects.is_empty() {
        if matches!(e.input, Input::Enter) && !objs.is_empty() {
            st.objects = objs
                .iter()
                .map(|s| (s.id.clone(), s.mesh.clone().unwrap()))
                .collect();
            return Reply::ask(
                format!("{label}: now select the finger rail, Enter"),
                save(&st),
            )
            .want(Want::Objects);
        }
        return Reply::ask(
            format!("{label}: select the heads / objects to pull, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    }
    let ids: Vec<&String> = st.objects.iter().map(|o| &o.0).collect();
    let rail = objs.iter().find(|s| !ids.contains(&&s.id));
    let Some(rail) = rail else {
        return Reply::ask(format!("{label}: select the finger rail, Enter"), save(&st))
            .want(Want::Objects);
    };
    let up = e.plane.normal();
    let rm = rail.mesh.as_ref().unwrap();
    let pulled: Vec<(String, SelectedMesh)> = st
        .objects
        .iter()
        .map(|(id, m)| (id.clone(), pull_mesh(m, rm, up, zone, gap)))
        .collect();
    let geo = |m: &SelectedMesh| Geo::Mesh {
        name: "Pulled".into(),
        vertices: m.vertices.clone(),
        triangles: m.triangles.clone(),
        meta: None,
        layer: None,
    };
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(
            format!("{label}: {} objects onto the rail. Enter", pulled.len()),
            save(&st),
        )
        .want(Want::Objects)
        .preview(pulled.iter().flat_map(|(_, m)| to_mesh(m).wire()).collect());
    }
    let mut r = Reply::finish(Vec::new()).say(format!("{label}: {} objects pulled", pulled.len()));
    r.replace = pulled
        .iter()
        .map(|(id, m)| Replacement {
            id: id.clone(),
            geo: geo(m),
        })
        .collect();
    r
}

// ---- prongs ----------------------------------------------------------------------------------

/// A prong along local z from `-drop` to `height`: radius `r` at the top, `taper` (fraction)
/// wider at the bottom, the top rounded by `fillet` (0 flat … 1 half sphere).
pub fn prong_mesh(r: f64, height: f64, drop: f64, taper: f64, fillet: f64) -> Mesh {
    let rb = (r * (1.0 + taper)).max(0.02);
    let dome = r * fillet.clamp(0.0, 1.0);
    let mut profile: Vec<(f64, f64)> = vec![(0.0, height)];
    for k in 1..=6 {
        let a = PI / 2.0 * k as f64 / 6.0;
        profile.push((r * a.sin(), height - dome + dome * a.cos()));
    }
    profile.push((rb, -drop));
    profile.push((0.0, -drop));
    mesh::revolve(&profile, 24)
}

/// The axis, base point, radius and length of a prong-like mesh (its principal direction).
pub fn measure_prong(m: &SelectedMesh) -> Option<(Vec3, Vec3, f64, f64, f64)> {
    let n = m.vertices.len();
    if n < 4 {
        return None;
    }
    let c = scale(
        m.vertices.iter().fold([0.0; 3], |a, &v| add(a, v)),
        1.0 / n as f64,
    );
    // Power iteration on the covariance for the long axis.
    let mut cov = [[0.0; 3]; 3];
    for v in &m.vertices {
        let d = sub(*v, c);
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }
    let mut a = [0.3, 0.5, 0.8];
    for _ in 0..50 {
        let b = [0, 1, 2].map(|i| (0..3).map(|j| cov[i][j] * a[j]).sum::<f64>());
        if norm(b) < 1e-18 {
            return None;
        }
        a = unit(b);
    }
    // The axis points up (toward the larger CPlane z) so the top stays the top.
    if a[2] < 0.0 {
        a = scale(a, -1.0);
    }
    let hs: Vec<f64> = m.vertices.iter().map(|&v| dot(sub(v, c), a)).collect();
    let (lo, hi) = (
        hs.iter().cloned().fold(f64::MAX, f64::min),
        hs.iter().cloned().fold(f64::MIN, f64::max),
    );
    let r = m
        .vertices
        .iter()
        .zip(&hs)
        .filter(|(_, h)| **h > lo + (hi - lo) * 0.05)
        .map(|(v, h)| norm(sub(sub(*v, c), scale(a, *h))))
        .fold(0.0, f64::max);
    Some((a, add(c, scale(a, lo)), r, hi - lo, lo))
}

fn prong_editor(e: &Event) -> Reply {
    let d = e.number("pe_diameter", 0.0);
    let grow = e.number("pe_height", 0.0);
    let taper = e.number("pe_taper", 0.0) / 100.0;
    let fillet = e.number("pe_fillet", 50.0) / 100.0;
    let tilt = e.number("pe_rotate", 0.0).to_radians();
    let label = "Prong Editor";
    let prongs = solids(e);
    let edits: Vec<(String, Mesh)> = prongs
        .iter()
        .filter_map(|s| {
            let (axis, base, r, len, _) = measure_prong(s.mesh.as_ref()?)?;
            let r = if d > 0.0 { d / 2.0 } else { r };
            let side = if axis[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let f = Frame::new(base, axis, side);
            // Tilt about the frame's x axis, pivoting at the base.
            let z = add(scale(f.z, tilt.cos()), scale(f.y, -tilt.sin()));
            let tf = Frame::new(base, z, f.x);
            Some((
                s.id.clone(),
                prong_mesh(r, len + grow, 0.0, taper, fillet).placed(&tf),
            ))
        })
        .collect();
    if edits.is_empty() {
        let r = Reply::ask(
            format!("{label}: select the prong to edit, Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a prong"))
        } else {
            r
        };
    }
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(
            format!(
                "{label}: {} prongs. Adjust in the builder, Enter",
                edits.len()
            ),
            json!({}),
        )
        .want(Want::Objects)
        .preview(edits.iter().flat_map(|x| x.1.wire()).collect());
    }
    let mut r = Reply::finish(Vec::new()).say(format!("{label}: {} prongs edited", edits.len()));
    r.replace = edits
        .into_iter()
        .map(|(id, m)| Replacement {
            id,
            geo: m.geo("Prong", None, None),
        })
        .collect();
    r
}

#[derive(Default, Serialize, Deserialize)]
struct Clicks {
    hits: Vec<(Vec3, Vec3)>,
}

fn on_surface(e: &Event) -> Reply {
    let bead = e.command == BEAD_ON_SURFACE;
    let (p, label) = if bead {
        ("bos", "Bead on Surface")
    } else {
        ("pos", "Prong on Surface")
    };
    let mut st: Clicks = load(e);
    let size = e.number(&format!("{p}_size"), if bead { 0.8 } else { 0.6 });
    let v_off = e.number(&format!("{p}_vertical_offset"), 0.0);
    let height = e.number("pos_height", 1.0);
    let drop = e.number("pos_drop", 0.8);
    let taper = e.number("pos_taper", 0.0) / 100.0;
    let fillet = e.number("pos_fillet", 100.0) / 100.0;
    let tilt = e.number("pos_rotate", 0.0).to_radians();
    let copy = e.flag(&format!("{p}_copy"), true);
    let object_mode = bead && e.text("bos_mode", "bead") == "object";
    let surfaces: Vec<&Selected> = solids(e);
    // In Object mode the second solid is the object to copy (modelled at the CPlane origin).
    let Some(srf) = surfaces.first() else {
        return Reply::ask(
            format!(
                "{label}: select the surface first{}",
                if object_mode {
                    " (and the object, modelled at the CPlane origin)"
                } else {
                    ""
                }
            ),
            save(&st),
        )
        .want(Want::Objects);
    };
    let n = e.plane.normal();
    let target = Target::new(srf.mesh.as_ref().unwrap(), n);
    let hit = |q: Vec3| target.as_ref().and_then(|t| t.drop(add(q, scale(n, 1e6))));
    let make = |h: &(Vec3, Vec3)| -> Mesh {
        if bead {
            mesh::sphere(size / 2.0, 16).placed(&Frame::new(
                add(h.0, scale(h.1, v_off)),
                h.1,
                e.plane.x,
            ))
        } else {
            let f = Frame::new(add(h.0, scale(h.1, v_off)), h.1, e.plane.x);
            let z = add(scale(f.z, tilt.cos()), scale(f.x, tilt.sin()));
            prong_mesh(size / 2.0, height, drop, taper, fillet)
                .placed(&Frame::new(f.origin, z, f.x))
        }
    };
    let object_frames = |hits: &[(Vec3, Vec3)]| -> Vec<ObjectMatrix> {
        let from = Frame::new(e.plane.origin, n, e.plane.x);
        surfaces
            .iter()
            .skip(1)
            .flat_map(|o| {
                hits.iter().map(|h| ObjectMatrix {
                    id: o.id.clone(),
                    matrix: frame_matrix(
                        &from,
                        &Frame::new(add(h.0, scale(h.1, v_off)), h.1, e.plane.x),
                        size / 0.8,
                    ),
                })
            })
            .collect()
    };
    let mut error = None;
    match &e.input {
        Input::Point { point } => match hit(*point) {
            Some(h) => {
                st.hits.push(h);
                if !copy {
                    return finish_on_surface(label, &st, object_mode, &make, &object_frames);
                }
            }
            None => error = Some(format!("{label}: click on the surface")),
        },
        Input::Undo => {
            st.hits.pop();
        }
        Input::Enter if !st.hits.is_empty() => {
            return finish_on_surface(label, &st, object_mode, &make, &object_frames);
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = st.hits.iter().flat_map(|h| make(h).wire()).collect();
    if let Input::Hover { point } = &e.input
        && let Some(h) = hit(*point)
    {
        preview.extend(make(&h).wire());
    }
    let r = Reply::ask(
        format!(
            "{label}: click on the surface ({} placed), Enter to finish",
            st.hits.len()
        ),
        save(&st),
    )
    .want(Want::Point)
    .preview(preview);
    match error {
        Some(w) => r.refuse(w),
        None => r,
    }
}

fn finish_on_surface(
    label: &str,
    st: &Clicks,
    object_mode: bool,
    make: &dyn Fn(&(Vec3, Vec3)) -> Mesh,
    objects: &dyn Fn(&[(Vec3, Vec3)]) -> Vec<ObjectMatrix>,
) -> Reply {
    if object_mode {
        let items = objects(&st.hits);
        let n = items.len();
        return Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: true,
            per_object: items,
        })
        .say(format!("{label}: {n} objects"));
    }
    let what = if label.starts_with("Bead") {
        "Bead"
    } else {
        "Prong"
    };
    let out: Vec<Geo> = st
        .hits
        .iter()
        .map(|h| make(h).geo(what, None, None))
        .collect();
    let n = out.len();
    Reply::finish(out).say(format!("{label}: {n} placed"))
}

// ---- Metal from Gems -------------------------------------------------------------------------

/// A path through the stone centers (with the end stones' far girdles for open lines).
fn gem_path(gems: &[PlacedGem]) -> (Vec<Vec3>, bool, f64) {
    let (order, closed) = om9_addon_geom::gems::chain(gems);
    let pts: Vec<Vec3> = order.iter().map(|&i| gems[i].frame.origin).collect();
    let w = gems
        .iter()
        .map(|g| g.gem.width.max(g.gem.length))
        .fold(0.0, f64::max);
    if closed || pts.len() < 2 {
        return (pts, closed, w);
    }
    // Reach past the end stones by their radius.
    let mut p = pts.clone();
    let (a, b) = (p[0], p[1]);
    let n = p.len();
    let (y, z) = (p[n - 1], p[n - 2]);
    let ra = gems[order[0]].gem.length / 2.0;
    let rz = gems[order[n - 1]].gem.length / 2.0;
    p.insert(0, add(a, scale(unit(sub(a, b)), ra)));
    p.push(add(y, scale(unit(sub(y, z)), rz)));
    (p, false, w)
}

/// Densify a polyline so sweeps bend smoothly.
fn dense(points: &[Vec3], closed: bool, step: f64) -> Vec<Vec3> {
    let Some(p) = Path::new(points, closed) else {
        return points.to_vec();
    };
    let n = (p.length() / step).ceil().max(2.0) as usize;
    if closed {
        (0..n)
            .map(|k| p.at(p.length() * k as f64 / n as f64))
            .collect()
    } else {
        (0..=n)
            .map(|k| p.at(p.length() * k as f64 / n as f64))
            .collect()
    }
}

/// Rectangle section `(side, up)` from `y0..y1` × `z0..z1`, counter-clockwise.
fn rect(y0: f64, y1: f64, z0: f64, z1: f64) -> Vec<[f64; 2]> {
    vec![[y1, z0], [y1, z1], [y0, z1], [y0, z0]]
}

/// The outline around a line of stones: offset `half` to both sides, rounded or pinched ends.
fn around(path: &[Vec3], up: Vec3, half: f64, pinched: bool) -> Vec<Vec3> {
    let n = path.len();
    let side = |i: usize| {
        let (_, _, s) = mesh::run_frame(path, false, i, up);
        s
    };
    let mut out: Vec<Vec3> = (0..n).map(|i| add(path[i], scale(side(i), half))).collect();
    let end_cap = |c: Vec3, s: Vec3, t: Vec3, out: &mut Vec<Vec3>| {
        for k in 1..12 {
            let a = PI * k as f64 / 12.0;
            let p = if pinched {
                let f = 1.0 - (a - PI / 2.0).abs() / (PI / 2.0);
                add(c, add(scale(s, half * a.cos()), scale(t, half * f)))
            } else {
                add(c, add(scale(s, half * a.cos()), scale(t, half * a.sin())))
            };
            out.push(p);
        }
    };
    let (t_end, _, _) = mesh::run_frame(path, false, n - 1, up);
    end_cap(path[n - 1], side(n - 1), t_end, &mut out);
    out.extend((0..n).rev().map(|i| add(path[i], scale(side(i), -half))));
    let (t0, _, _) = mesh::run_frame(path, false, 0, up);
    end_cap(path[0], scale(side(0), -1.0), scale(t0, -1.0), &mut out);
    out
}

fn metal_piece(e: &Event) -> Reply {
    let mode = e.text("metal_mode", "outside");
    let wall = e.number("metal_wall", 0.5);
    let height = e.number("metal_height", 0.3);
    let depth = e.number("metal_depth", 1.5);
    let floor = e.number("metal_floor", 0.5);
    let gap = e.number("metal_gap", 0.05);
    built(
        e,
        "Metal from Gems",
        "select a line (or loop) of stones, Enter",
        || {
            let (gems, _) = selected_gems(e);
            if gems.is_empty() {
                return Err("select stones".into());
            }
            let up = scale(
                gems.iter().fold([0.0; 3], |a, g| add(a, g.frame.z)),
                1.0 / gems.len() as f64,
            );
            let (path, closed, w) = gem_path(&gems);
            let half = w / 2.0 + gap;
            let (z0, z1) = (-depth, height);
            let run = dense(&path, closed, 0.25);
            let sweep = |sec: &[[f64; 2]]| mesh::sweep(&run, closed, up, sec);
            let mut parts: Vec<Mesh> = Vec::new();
            match mode {
                "inside" | "outside_center" => {
                    // A U channel: two walls and the floor under the stones.
                    let u = vec![
                        [half + wall, z0 - floor],
                        [half + wall, z1],
                        [half, z1],
                        [half, z0],
                        [-half, z0],
                        [-half, z1],
                        [-half - wall, z1],
                        [-half - wall, z0 - floor],
                    ];
                    parts.extend(sweep(&u));
                    if mode == "outside_center" && !closed {
                        // Close the ends with plates.
                        for (i, j) in [(0usize, 1usize), (run.len() - 1, run.len() - 2)] {
                            let t = unit(sub(run[i], run[j]));
                            let f = Frame::new(run[i], up, t);
                            let c = mesh::cuboid(
                                [0.0, -half - wall, z0 - floor],
                                [wall, half + wall, z1],
                            );
                            parts.push(c.placed(&f));
                        }
                    }
                }
                "top" => parts.extend(sweep(&rect(-half, half, z0, z1 + 2.0))),
                "round_corners" | "pinched_ends" if !closed => {
                    let ring = around(&run, up, half, mode == "pinched_ends");
                    // The sweep's +side is left of travel: outward when the loop turns clockwise.
                    let turn: f64 = (0..ring.len())
                        .map(|i| dot(cross(ring[i], ring[(i + 1) % ring.len()]), up))
                        .sum();
                    let sec = if turn < 0.0 {
                        rect(0.0, wall, z0, z1)
                    } else {
                        rect(-wall, 0.0, z0, z1)
                    };
                    parts.extend(mesh::sweep(&ring, true, up, &sec));
                }
                _ => {
                    parts.extend(sweep(&rect(half, half + wall, z0, z1)));
                    parts.extend(sweep(&rect(-half - wall, -half, z0, z1)));
                }
            }
            if parts.is_empty() {
                return Err("the stones do not make a line".into());
            }
            let layer = (mode == "top").then_some("Cutting");
            let out: Vec<Geo> = parts
                .into_iter()
                .map(|m| {
                    m.geo(
                        if mode == "top" {
                            "Channel cutter"
                        } else {
                            "Metal piece"
                        },
                        None,
                        layer,
                    )
                })
                .collect();
            let n = out.len();
            Ok((
                out,
                format!(
                    "{n} pieces ({mode}) for {} stones{}",
                    gems.len(),
                    if closed { ", closed loop" } else { "" }
                ),
            ))
        },
    )
}

// ---- Boolean Builder -------------------------------------------------------------------------

/// Boolean Builder: the preview wires of the last computed result, by its inputs.
static BOOLEAN_PREVIEW: std::sync::Mutex<Option<(String, Vec<Geo>)>> = std::sync::Mutex::new(None);

#[derive(Default, Serialize, Deserialize)]
struct Objects {
    objects: Vec<(String, SelectedMesh)>,
}

/// Why a mesh will not boolean well: open or non-manifold edges, no volume.
pub fn check(m: &SelectedMesh) -> Option<String> {
    let (open, bad) = m.open_and_nonmanifold_edges();
    if open > 0 || bad > 0 {
        return Some(format!("{open} open and {bad} non-manifold edges"));
    }
    if m.volume().abs() < 1e-12 {
        return Some("no volume".into());
    }
    None
}

fn boolean_builder(e: &Event) -> Reply {
    let label = "Boolean Builder";
    let op = match e.text("bb_mode", "difference") {
        "union" => Op::Union,
        "intersection" => Op::Intersection,
        _ => Op::Difference,
    };
    let keep = e.flag("bb_keep_cutters", false);
    let mut st: Objects = load(e);
    let objs = solids(e);
    if st.objects.is_empty() {
        if matches!(e.input, Input::Enter) && !objs.is_empty() {
            let bad: Vec<String> = objs
                .iter()
                .filter_map(|s| check(s.mesh.as_ref().unwrap()).map(|w| format!("{}: {w}", s.name)))
                .collect();
            st.objects = objs
                .iter()
                .map(|s| (s.id.clone(), s.mesh.clone().unwrap()))
                .collect();
            let mut r = Reply::ask(
                format!(
                    "{label}: {} objects. Now select the cutters, Enter",
                    st.objects.len()
                ),
                save(&st),
            )
            .want(Want::Objects);
            r.messages = bad
                .into_iter()
                .map(|w| format!("{label}: warning {w}"))
                .collect();
            return r;
        }
        return Reply::ask(
            format!("{label}: select the object(s) to cut, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    }
    let ids: Vec<&String> = st.objects.iter().map(|o| &o.0).collect();
    let mut cutters: Vec<&Selected> = objs.into_iter().filter(|s| !ids.contains(&&s.id)).collect();
    // The same cutters give the same result whatever order the selection lists them in.
    cutters.sort_by(|a, b| a.id.cmp(&b.id));
    if cutters.is_empty() {
        let r = Reply::ask(format!("{label}: select the cutters, Enter"), save(&st))
            .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select the cutters"))
        } else {
            r
        };
    }
    let warnings: Vec<String> = cutters
        .iter()
        .filter_map(|s| {
            check(s.mesh.as_ref().unwrap()).map(|w| format!("{label}: warning {}: {w}", s.name))
        })
        .collect();
    let compute = || -> Vec<(String, Mesh)> {
        let tool = csg::fold(
            &cutters
                .iter()
                .map(|s| to_mesh(s.mesh.as_ref().unwrap()))
                .collect::<Vec<_>>(),
            Op::Union,
        )
        .unwrap();
        st.objects
            .iter()
            .map(|(id, m)| (id.clone(), csg::boolean(&to_mesh(m), &tool, op)))
            .collect()
    };
    // The result of the last objects × cutters × mode: the preview is redrawn on every cursor
    // move, but a boolean of a ring against its cutters takes seconds.
    let key = format!(
        "{:?}|{:?}|{:?}",
        ids,
        cutters
            .iter()
            .map(|s| (&s.id, s.bbox, s.mesh.as_ref().map(|m| m.vertices.len())))
            .collect::<Vec<_>>(),
        op
    );
    if !matches!(e.input, Input::Enter) {
        let mut cache = BOOLEAN_PREVIEW.lock().unwrap_or_else(|p| p.into_inner());
        if cache.as_ref().is_none_or(|(k, _)| *k != key) {
            let wire = compute().iter().flat_map(|r| r.1.wire()).collect();
            *cache = Some((key, wire));
        }
        let preview = cache.as_ref().map(|c| c.1.clone()).unwrap_or_default();
        return Reply::ask(
            format!(
                "{label}: {} objects × {} cutters. Mode in the builder, Enter",
                st.objects.len(),
                cutters.len()
            ),
            save(&st),
        )
        .want(Want::Objects)
        .preview(preview);
    }
    let results = compute();
    let mut r = Reply::finish(Vec::new()).say(format!(
        "{label}: {} objects, {} cutters, volume {} mm³",
        results.len(),
        cutters.len(),
        fmt(results.iter().map(|r| r.1.volume()).sum::<f64>())
    ));
    r.messages.extend(warnings);
    r.replace = results
        .into_iter()
        .map(|(id, m)| Replacement {
            id,
            geo: m.geo("Boolean", None, None),
        })
        .collect();
    if !keep {
        r.delete = cutters.iter().map(|s| s.id.clone()).collect();
    }
    r
}

/// For tests: the length of a polyline.
pub fn length(p: &[Vec3]) -> f64 {
    p.windows(2).map(|w| dist(w[0], w[1])).sum()
}

#[cfg(test)]
mod tests;
