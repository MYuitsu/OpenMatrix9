// SPDX-License-Identifier: LGPL-2.1-or-later
//! The rest of the Matrix-style **Gems** menu for OM9: layouts along curves (advanced, list,
//! multi-row), on surfaces (one by one, pavé fill, pavé builder, pavé sphere, splop), tapered
//! baguettes, clusters, pavé prongs and azure cutters, layout editing (springs, follow,
//! control, positioner, gem flow), gem profiles, orienting objects to stones, and styles
//! (match attributes, save and load).
//!
//! Every stone carries an `om9.gem` record, so the setting and cutter builders find it.
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, ObjectMatrix, RecordUpdate, Reply, Selected, SelectedMesh,
    TransformOp, Vec3, Want, add, apply, cross, dist, dot, fmt, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save, selected_gems, styles_of};
use om9_addon_geom::gems::{Cut, Gem, PlacedGem};
use om9_addon_geom::mesh::{self, Frame, Mesh};
use om9_addon_geom::path::{Path, Target, frame_matrix};
use om9_addon_geom::{sdf, section};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::f64::consts::{PI, TAU};

pub const ON_CRV_ADVANCED: &str = "BuilderGemOnCrvAdvanced";
pub const LIST_ON_CRV: &str = "BuilderGemListOnCrv";
pub const ON_CRV_MULTI: &str = "BuilderGemOnCrvMulti";
pub const PROFILE: &str = "BuilderGemProfile";
pub const ORIENT_TO_GEM: &str = "BuilderOrientToGem";
pub const ON_SURFACE: &str = "BuilderGemOnSurface";
pub const TAPER_CHANNEL: &str = "BuilderTaperBagChannel";
pub const TAPER_BETWEEN: &str = "BuilderTaperBagBetweenTwoCurves";
pub const CLUSTER: &str = "BuilderClusterBuilderOld";
pub const AUTO_PAVE: &str = "BuilderPaveDialog";
pub const PAVE: &str = "BuilderPaveBuilder";
pub const AZURE: &str = "BuilderPaveAzureBuilder";
pub const PAVE_PRONGS: &str = "BuilderPaveProngBuilder";
pub const SPRINGS: &str = "BuilderGemSprings";
pub const FOLLOW: &str = "BuilderGemFollow";
pub const CONTROL: &str = "BuilderGemControl";
pub const MATCH: &str = "OthersMatchAttributes";
pub const SAVE_STYLE: &str = "OthersSaveStyle";
pub const LOAD_STYLE: &str = "OthersLoadStyle";
pub const GEM_FLOW: &str = "BuilderGVDGemFlow";
pub const GEM_SPLOP: &str = "BuilderGVDGemSplop";
pub const PAVE_SPHERE: &str = "BuilderPaveSphere";
pub const POSITIONER: &str = "BuilderGemPositioner";

pub struct GemTools;

impl Interactive for GemTools {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        ON_CRV_ADVANCED,
        LIST_ON_CRV,
        ON_CRV_MULTI,
        PROFILE,
        ORIENT_TO_GEM,
        ON_SURFACE,
        TAPER_CHANNEL,
        TAPER_BETWEEN,
        CLUSTER,
        AUTO_PAVE,
        PAVE,
        AZURE,
        PAVE_PRONGS,
        SPRINGS,
        FOLLOW,
        CONTROL,
        MATCH,
        SAVE_STYLE,
        LOAD_STYLE,
        GEM_FLOW,
        GEM_SPLOP,
        PAVE_SPHERE,
        POSITIONER,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            ON_CRV_ADVANCED => on_curve_advanced(e),
            LIST_ON_CRV => list_on_curve(e),
            ON_CRV_MULTI => on_curve_multi(e),
            PROFILE => gem_profile(e),
            ORIENT_TO_GEM => orient_to_gem(e),
            ON_SURFACE | PAVE => on_surface(e),
            TAPER_CHANNEL => taper_channel(e),
            TAPER_BETWEEN => taper_between(e),
            CLUSTER => cluster(e),
            AUTO_PAVE => auto_pave(e),
            AZURE => azure(e),
            PAVE_PRONGS => pave_prongs(e),
            SPRINGS => springs(e),
            FOLLOW => follow(e),
            CONTROL => control(e),
            MATCH => match_attributes(e),
            SAVE_STYLE => save_style(e),
            LOAD_STYLE => load_style(e),
            GEM_FLOW => gem_flow(e),
            GEM_SPLOP => gem_splop(e),
            PAVE_SPHERE => pave_sphere(e),
            POSITIONER => positioner(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- shared ----------------------------------------------------------------------------------

/// A stone mesh with its `om9.gem` record, placed with `frame`.
pub fn stone(gem: &Gem, frame: &Frame, source: &str, material: &str) -> Geo {
    gem.mesh()
        .placed(frame)
        .geo(&gem.name(), Some(gem.record(frame, source, material)), None)
}

fn girdle(gem: &Gem, frame: &Frame) -> Geo {
    Geo::Polyline {
        points: gem
            .outline(40)
            .iter()
            .map(|p| frame.point([p[0], p[1], 0.0]))
            .collect(),
        closed: true,
    }
}

/// The stone a builder makes: `<p>_cut`, `<p>_size`, `<p>_width` (0 = the cut's usual ratio).
pub fn gem_setting(e: &Event, p: &str, default_size: f64) -> Gem {
    let cut = Cut::from_key(e.text(&format!("{p}_cut"), "round")).unwrap_or(Cut::Round);
    let l = e.number(&format!("{p}_size"), default_size).max(0.1);
    let w = e.number(&format!("{p}_width"), 0.0);
    Gem::new(cut, l, if w > 0.0 { w } else { l * cut.default_ratio() })
}

fn material(e: &Event, p: &str) -> String {
    e.text(&format!("{p}_material"), "diamond").to_owned()
}

/// The same cut scaled to a new length.
fn resized(g: &Gem, length: f64) -> Gem {
    let f = length / g.length.max(1e-9);
    let mut n = Gem::new(g.cut, length, g.width * f);
    n.custom = g
        .custom
        .as_ref()
        .map(|o| o.iter().map(|p| [p[0] * f, p[1] * f]).collect());
    n
}

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

/// Surfaces, solids and meshes that are not stones.
fn surfaces(e: &Event) -> Vec<&SelectedMesh> {
    e.selection
        .iter()
        .filter(|s| s.gem.is_none() && s.kind != "curve")
        .filter_map(|s| s.mesh.as_ref())
        .filter(|m| !m.triangles.is_empty())
        .collect()
}

fn path_of(s: &Selected) -> Option<Path> {
    Path::new(&s.points, s.closed)
}

/// The nearest surface point and its normal (turned toward `toward`, when given).
pub fn on_mesh(m: &SelectedMesh, p: Vec3, toward: Option<Vec3>) -> Option<(Vec3, Vec3)> {
    let mut best: Option<(f64, Vec3, Vec3)> = None;
    for t in &m.triangles {
        let [a, b, c] = [
            m.vertices[t[0] as usize],
            m.vertices[t[1] as usize],
            m.vertices[t[2] as usize],
        ];
        let q = sdf::closest_on_triangle(p, a, b, c);
        let d = dist(p, q);
        if best.as_ref().is_none_or(|x| d < x.0) {
            let n = cross(sub(b, a), sub(c, a));
            if norm(n) > 1e-18 {
                best = Some((d, q, unit(n)));
            }
        }
    }
    let (_, q, mut n) = best?;
    if let Some(t) = toward
        && dot(n, t) < 0.0
    {
        n = scale(n, -1.0);
    }
    Some((q, n))
}

/// A builder that previews what it would make and makes it on Enter.
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
        Ok((_, what)) => {
            let r = Reply::ask(format!("{label}: {ask}"), json!({})).want(Want::Objects);
            if matches!(e.input, Input::Enter) {
                r.refuse(format!("{label}: {what}"))
            } else {
                r
            }
        }
        Err(why) => {
            let r = Reply::ask(format!("{label}: {ask}"), json!({})).want(Want::Objects);
            if matches!(e.input, Input::Enter) {
                r.refuse(why)
            } else {
                r
            }
        }
    }
}

/// Which way stones along a curve face.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Facing {
    /// Tables up along the CPlane normal (flat layouts).
    CPlane,
    /// Tables away from the curve's middle (the outside of a ring rail).
    Outward,
    /// Tables toward the middle (the inside of a ring).
    Inward,
}

impl Facing {
    fn of(s: &str) -> Facing {
        match s {
            "outward" => Facing::Outward,
            "inward" => Facing::Inward,
            _ => Facing::CPlane,
        }
    }
}

/// A stone frame along a path: x along the curve, z per `facing`, then offsets.
pub fn frame_on(
    path: &Path,
    s: f64,
    e: &Event,
    facing: Facing,
    flip: bool,
    y_off: f64,
    z_off: f64,
) -> Frame {
    let up = match facing {
        Facing::CPlane => Some(e.plane.normal()),
        Facing::Outward => None,
        Facing::Inward => Some(sub(path.centroid(), path.at(s))),
    };
    let mut f = path.frame(s, up);
    if flip {
        f = Frame::new(f.origin, scale(f.z, -1.0), f.x);
    }
    f.moved([0.0, y_off, z_off])
}

/// Arc lengths of the stone centers between `a` and `b` (sizes from `size_at`), `gap` apart.
pub fn spaced(
    a: f64,
    b: f64,
    gap_at: &dyn Fn(f64) -> f64,
    size_at: &dyn Fn(f64) -> f64,
    closed_full: Option<f64>,
) -> Vec<f64> {
    if let Some(len) = closed_full {
        // A full closed curve: equal steps all round.
        let d = size_at(0.0);
        let n = (len / (d + gap_at(0.0))).floor() as usize;
        return (0..n).map(|k| len * k as f64 / n as f64).collect();
    }
    let mut out = Vec::new();
    let mut s = a + size_at(a) / 2.0;
    while s + size_at(s) / 2.0 <= b + 1e-9 && out.len() < 10_000 {
        out.push(s);
        let d = size_at(s);
        let gap = gap_at(s);
        let guess = s + d + gap;
        s += d / 2.0 + gap + size_at(guess) / 2.0;
    }
    out
}

/// Corners of a curve (arc lengths): control points of polylines, else sharp turns.
fn corners(sel: &Selected, path: &Path) -> Vec<f64> {
    let pts: Vec<Vec3> = match &sel.exact {
        Some(Geo::Nurbs {
            degree: 1, points, ..
        }) => points.clone(),
        Some(Geo::Polyline { points, .. }) => points.clone(),
        _ => {
            let p = &sel.points;
            let n = p.len();
            let mut c = vec![p[0]];
            for i in 1..n.saturating_sub(1) {
                let (u, v) = (unit(sub(p[i], p[i - 1])), unit(sub(p[i + 1], p[i])));
                if dot(u, v) < 0.9 {
                    c.push(p[i]);
                }
            }
            c.push(p[n - 1]);
            c
        }
    };
    let mut s: Vec<f64> = pts.iter().map(|&p| path.closest(p)).collect();
    s.sort_by(f64::total_cmp);
    s.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    s
}

// ---- Gem on Curve Advanced -------------------------------------------------------------------

#[derive(Clone, Copy)]
struct LineParams {
    start: f64,
    end: f64,
    gap: f64,
    taper: bool,
    end_size: f64,
    taper_gap: bool,
    y_off: f64,
    z_off: f64,
    facing: Facing,
    flip: bool,
}

fn line_params(e: &Event, p: &str) -> LineParams {
    LineParams {
        start: e.number(&format!("{p}_start"), 0.0) / 100.0,
        end: e.number(&format!("{p}_end"), 100.0) / 100.0,
        gap: e.number(&format!("{p}_spacing"), 0.2),
        taper: e.flag(&format!("{p}_taper"), false),
        end_size: e.number(&format!("{p}_end_size"), 1.0),
        taper_gap: e.flag(&format!("{p}_taper_spacing"), false),
        y_off: e.number(&format!("{p}_y_offset"), 0.0),
        z_off: e.number(&format!("{p}_z_offset"), 0.0),
        facing: Facing::of(e.text(&format!("{p}_facing"), "cplane")),
        flip: e.flag(&format!("{p}_flip"), false),
    }
}

/// Stones along one curve: (frame, stone) for each.
fn line_of_stones(
    e: &Event,
    sel: &Selected,
    gem: &Gem,
    lp: &LineParams,
    snap: bool,
) -> Vec<(Frame, Gem)> {
    let Some(path) = path_of(sel) else {
        return Vec::new();
    };
    let len = path.length();
    let (mut a, mut b) = (lp.start * len, lp.end * len);
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    let d0 = gem.length;
    let size_at = |s: f64| -> f64 {
        if lp.taper && b > a {
            d0 + (lp.end_size - d0) * ((s - a) / (b - a)).clamp(0.0, 1.0)
        } else {
            d0
        }
    };
    let gap_scaled = |s: f64| {
        if lp.taper_gap {
            lp.gap * size_at(s) / d0
        } else {
            lp.gap
        }
    };
    let full = sel.closed && lp.start <= 1e-9 && lp.end >= 1.0 - 1e-9 && !lp.taper;
    let stations: Vec<f64> = if snap {
        let c = corners(sel, &path);
        let mut out = Vec::new();
        let mut ends = c.clone();
        if sel.closed {
            ends.push(c[0] + len);
        }
        for w in ends.windows(2) {
            let seg = w[1] - w[0];
            let n = (seg / (d0 + lp.gap)).round().max(1.0) as usize;
            for k in 0..n {
                out.push((w[0] + seg * k as f64 / n as f64) % (len + 1e-12));
            }
        }
        if !sel.closed {
            out.push(*c.last().unwrap());
        }
        out.retain(|&s| s >= a - 1e-9 && s <= b + 1e-9);
        out
    } else {
        spaced(a, b, &gap_scaled, &size_at, full.then_some(len))
            .into_iter()
            .map(|s| s.min(b))
            .collect()
    };
    stations
        .into_iter()
        .map(|s| {
            (
                frame_on(&path, s, e, lp.facing, lp.flip, lp.y_off, lp.z_off),
                resized(gem, size_at(s)),
            )
        })
        .collect()
}

fn on_curve_advanced(e: &Event) -> Reply {
    let mode = e.text("goca_mode", "gem");
    let lp = line_params(e, "goca");
    let snap = e.flag("goca_snap_corners", false);
    // SelectGem: a stone selected with the curve sets the cut and size.
    let (picked, _) = selected_gems(e);
    let gem = picked
        .first()
        .map_or_else(|| gem_setting(e, "goca", 1.5), |g| g.gem.clone());
    let mat = picked
        .first()
        .map_or_else(|| material(e, "goca"), |g| g.material.clone());
    let objects: Vec<&Selected> = e.selection.iter().filter(|s| s.kind != "curve").collect();
    let label = "Gem on Curve Advanced";
    if mode == "object" {
        // Lay the selected objects (modelled around the CPlane origin, F4) along the curve.
        let cs = curves(e);
        let frames: Vec<Frame> = cs
            .iter()
            .flat_map(|c| line_of_stones(e, c, &gem, &lp, snap))
            .map(|x| x.0)
            .collect();
        if cs.is_empty() || objects.is_empty() || frames.is_empty() {
            let r = Reply::ask(
                format!(
                    "{label}: select the curve and the objects (around the CPlane origin), Enter"
                ),
                json!({}),
            )
            .want(Want::Objects);
            return if matches!(e.input, Input::Enter) {
                r.refuse(format!("{label}: Mode=Object needs a curve and objects"))
            } else {
                r
            };
        }
        let from = Frame::new(e.plane.origin, e.plane.normal(), e.plane.x);
        let mats: Vec<[f64; 16]> = frames.iter().map(|f| frame_matrix(&from, f, 1.0)).collect();
        if matches!(e.input, Input::Enter) {
            let n = mats.len();
            return Reply::finish_transform(TransformOp {
                matrices: mats,
                keep_original: true,
                per_object: Vec::new(),
            })
            .say(format!("{label}: {n} copies along the curve"));
        }
        let mut r = Reply::ask(
            format!("{label}: {} copies. Enter to place", mats.len()),
            json!({}),
        )
        .want(Want::Objects);
        r.preview_transforms = mats;
        return r;
    }
    built(e, label, "select curves, Enter to set the stones", || {
        let cs = curves(e);
        if cs.is_empty() {
            return Err("select a curve".into());
        }
        let mut out = Vec::new();
        for c in cs {
            for (f, g) in line_of_stones(e, c, &gem, &lp, snap) {
                if mode == "sphere" {
                    let r = g.length / 2.0;
                    out.push(mesh::sphere(r, 16).placed(&f).geo("Bead", None, None));
                } else {
                    out.push(stone(&g, &f, "gem_on_curve_advanced", &mat));
                }
            }
        }
        let n = out.len();
        Ok((
            out,
            format!(
                "{n} {} {}",
                if mode == "sphere" { "beads" } else { "stones" },
                gem.size_label()
            ),
        ))
    })
}

// ---- Gem List on Curve -----------------------------------------------------------------------

fn list_on_curve(e: &Event) -> Reply {
    let lp = line_params(e, "glc");
    let from_end = e.text("glc_direction", "start") == "end";
    let fill = e.flag("glc_fill", false);
    built(
        e,
        "Gem List on Curve",
        "select the curve and the stones of the list (in order), Enter",
        || {
            let cs = curves(e);
            let (list, _) = selected_gems(e);
            let c = cs.first().ok_or("select a curve")?;
            if list.is_empty() {
                return Err("select the stones of the list".into());
            }
            let path = path_of(c).ok_or("curve")?;
            let len = path.length();
            let (a, b) = (lp.start * len, lp.end * len);
            let mut out = Vec::new();
            let mut s = 0.0;
            let mut k = 0;
            let count = list.len();
            loop {
                let g = &list[k % count];
                let d = g.gem.length;
                let at = s + d / 2.0;
                if a + at + d / 2.0 > b + 1e-9 || (!fill && k >= count) || k > 5000 {
                    break;
                }
                let pos = if from_end { b - at } else { a + at };
                let t = if b > a {
                    (at / (b - a)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                // Taper offsets from the start values to the end values.
                let y = lp.y_off + (e.number("glc_end_y_offset", lp.y_off) - lp.y_off) * t;
                let z = lp.z_off + (e.number("glc_end_z_offset", lp.z_off) - lp.z_off) * t;
                let f = frame_on(&path, pos, e, lp.facing, lp.flip, y, z);
                out.push(stone(&g.gem, &f, "gem_list_on_curve", &g.material));
                s += d + lp.gap;
                k += 1;
            }
            let n = out.len();
            Ok((out, format!("{n} stones from a list of {count}")))
        },
    )
}

// ---- Gem on Curve Multi ----------------------------------------------------------------------

fn on_curve_multi(e: &Event) -> Reply {
    let lp = line_params(e, "gcm");
    let gem = gem_setting(e, "gcm", 1.3);
    let mat = material(e, "gcm");
    built(
        e,
        "Gem on Curve Multi",
        "select the primary (middle) curve first, then the other rows, Enter",
        || {
            let cs = curves(e);
            if cs.is_empty() {
                return Err("select curves".into());
            }
            // The primary curve sets the positions; every curve gets a row at the same fractions.
            let primary = path_of(cs[0]).ok_or("curve")?;
            let rows = line_of_stones(e, cs[0], &gem, &lp, false);
            let fractions: Vec<(f64, f64)> = rows
                .iter()
                .map(|(f, g)| (primary.closest(f.origin) / primary.length(), g.length))
                .collect();
            let mut out = Vec::new();
            for c in &cs {
                let Some(p) = path_of(c) else { continue };
                for &(t, d) in &fractions {
                    let f = frame_on(
                        &p,
                        t * p.length(),
                        e,
                        lp.facing,
                        lp.flip,
                        lp.y_off,
                        lp.z_off,
                    );
                    out.push(stone(&resized(&gem, d), &f, "gem_on_curve_multi", &mat));
                }
            }
            let n = out.len();
            Ok((out, format!("{n} stones in {} rows", cs.len())))
        },
    )
}

// ---- Gem Profile -----------------------------------------------------------------------------

/// A closed profile around a stone's girdle in its vertical section (x across, z up).
pub fn profile_for(g: &PlacedGem, height: f64, wall: f64, z_off: f64, round: bool) -> Geo {
    let w = g.gem.width / 2.0 + wall;
    let (top, bottom) = (height / 2.0 + z_off, -height / 2.0 + z_off);
    let f = &g.frame;
    let pt = |y: f64, z: f64| f.point([0.0, y, z]);
    if round {
        // An ellipse through the four extremes.
        let n = 48;
        let pts = (0..n)
            .map(|k| {
                let a = TAU * k as f64 / n as f64;
                pt(w * a.cos(), z_off + height / 2.0 * a.sin())
            })
            .collect();
        return Geo::Polyline {
            points: pts,
            closed: true,
        };
    }
    Geo::Polyline {
        points: vec![pt(-w, bottom), pt(w, bottom), pt(w, top), pt(-w, top)],
        closed: true,
    }
}

fn gem_profile(e: &Event) -> Reply {
    let h = e.number("gp_height", 2.0);
    let wall = e.number("gp_width", 0.4);
    let z = e.number("gp_z_offset", 0.0);
    let round = e.text("gp_shape", "rectangle") == "round";
    built(e, "Gem Profile", "select stones, Enter", || {
        let (gems, _) = selected_gems(e);
        let out: Vec<Geo> = gems
            .iter()
            .map(|g| profile_for(g, h, wall, z, round))
            .collect();
        let n = out.len();
        Ok((out, format!("{n} profiles")))
    })
}

// ---- Orient to Gem ---------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct Group {
    ids: Vec<String>,
    frames: Vec<(Vec3, Vec3, Vec3)>,
    sizes: Vec<f64>,
    selection: Vec<Selected>,
}

fn frame_of(f: &(Vec3, Vec3, Vec3)) -> Frame {
    Frame::new(f.0, f.1, f.2)
}

fn orient_to_gem(e: &Event) -> Reply {
    let mut st: Group = load(e);
    let label = "Orient to Gem";
    let scale_to = e.flag("otg_scale", true);
    if st.ids.is_empty() {
        let (gems, _) = selected_gems(e);
        if matches!(e.input, Input::Enter) && !gems.is_empty() {
            st.ids = gems.iter().map(|g| g.id.clone()).collect();
            st.frames = gems
                .iter()
                .map(|g| (g.frame.origin, g.frame.z, g.frame.x))
                .collect();
            st.sizes = gems.iter().map(|g| g.gem.length).collect();
            return Reply::ask(format!("{label}: {} stones. Now select the objects (modelled at the CPlane origin for a 10 mm stone), Enter", gems.len()), save(&st)).want(Want::Objects);
        }
        return Reply::ask(format!("{label}: select the stones, Enter"), save(&st))
            .want(Want::Objects);
    }
    let objects: Vec<&Selected> = e
        .selection
        .iter()
        .filter(|s| !st.ids.contains(&s.id))
        .collect();
    let from = Frame::new(e.plane.origin, e.plane.normal(), e.plane.x);
    let items: Vec<ObjectMatrix> = objects
        .iter()
        .flat_map(|o| {
            st.frames.iter().zip(&st.sizes).map(|(f, d)| ObjectMatrix {
                id: o.id.clone(),
                matrix: frame_matrix(&from, &frame_of(f), if scale_to { d / 10.0 } else { 1.0 }),
            })
        })
        .collect();
    if items.is_empty() || !matches!(e.input, Input::Enter) {
        let mut r = Reply::ask(
            format!(
                "{label}: {} objects × {} stones, Enter",
                objects.len(),
                st.ids.len()
            ),
            save(&st),
        )
        .want(Want::Objects);
        r.preview = objects
            .iter()
            .flat_map(|o| {
                st.frames.iter().zip(&st.sizes).map(|(f, d)| {
                    let m =
                        frame_matrix(&from, &frame_of(f), if scale_to { d / 10.0 } else { 1.0 });
                    let [lo, hi] = o.bbox;
                    let c: Vec<Vec3> = [
                        [lo[0], lo[1], lo[2]],
                        [hi[0], lo[1], lo[2]],
                        [hi[0], hi[1], lo[2]],
                        [lo[0], hi[1], lo[2]],
                    ]
                    .iter()
                    .map(|&p| apply(&m, p))
                    .collect();
                    Geo::Polyline {
                        points: c,
                        closed: true,
                    }
                })
            })
            .collect();
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select the objects to orient"))
        } else {
            r
        };
    }
    let n = items.len();
    Reply::finish_transform(TransformOp {
        matrices: Vec::new(),
        keep_original: true,
        per_object: items,
    })
    .say(format!("{label}: {n} copies oriented to the stones"))
}

// ---- Gem on Surface / Pavé Builder -----------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct Placed {
    /// (center, axis, x, length) of the stones placed so far.
    stones: Vec<(Vec3, Vec3, Vec3, f64)>,
}

fn on_surface(e: &Event) -> Reply {
    let pave = e.command == PAVE;
    let (p, label) = if pave {
        ("pb", "Pavé Builder")
    } else {
        ("gos", "Gem on Surface")
    };
    let mut st: Placed = load(e);
    let base = gem_setting(e, p, 1.3);
    let mat = material(e, p);
    let v_off = e.number(&format!("{p}_vertical_offset"), 0.0);
    let angle = e.number(&format!("{p}_angle"), 0.0).to_radians();
    let flip = e.flag(&format!("{p}_flip"), false);
    let gap = e.number(&format!("{p}_spacing"), 0.15);
    let min = e.number("pb_min_size", 0.8);
    let copy = e.flag("gos_copy", true);
    let ms = surfaces(e);
    let Some(m) = ms.first() else {
        return Reply::ask(format!("{label}: select the surface first"), save(&st))
            .want(Want::Objects);
    };
    let n = e.plane.normal();
    let target = Target::new(m, n);
    let (existing, _) = selected_gems(e);
    let place = |q: Vec3, st: &Placed| -> Result<(Frame, Gem), String> {
        let (hit, nn) = target
            .as_ref()
            .and_then(|t| t.drop(add(q, scale(n, 1e6))))
            .ok_or("click on the surface")?;
        let z = if flip { scale(nn, -1.0) } else { nn };
        let f = Frame::new(hit, z, e.plane.x)
            .rotated(angle)
            .moved([0.0, 0.0, v_off]);
        let mut d = base.length;
        if pave {
            // Fit the stone between its neighbours: as big as the nearest gap allows.
            let others = st
                .stones
                .iter()
                .map(|s| (s.0, s.3))
                .chain(existing.iter().map(|g| (g.frame.origin, g.gem.length)));
            for (c, l) in others {
                d = d.min(2.0 * (dist(c, f.origin) - l / 2.0 - gap));
            }
            if d < min {
                return Err(format!(
                    "no room for a {} mm stone here (min {})",
                    fmt(d.max(0.0)),
                    fmt(min)
                ));
            }
        }
        Ok((f, resized(&base, d)))
    };
    let mut error = None;
    match &e.input {
        Input::Point { point } => match place(*point, &st) {
            Ok((f, g)) => {
                st.stones.push((f.origin, f.z, f.x, g.length));
                if !copy {
                    return Reply::finish(vec![stone(&g, &f, &e.command, &mat)])
                        .say(format!("{label}: 1 stone"));
                }
            }
            Err(w) => error = Some(w),
        },
        Input::Undo => {
            st.stones.pop();
        }
        Input::Enter => {
            if st.stones.is_empty() {
                return Reply::ask(format!("{label}: click on the surface"), save(&st))
                    .want(Want::Point)
                    .refuse(format!("{label}: no stones placed"));
            }
            let out: Vec<Geo> = st
                .stones
                .iter()
                .map(|s| {
                    stone(
                        &resized(&base, s.3),
                        &Frame::new(s.0, s.1, s.2),
                        &e.command,
                        &mat,
                    )
                })
                .collect();
            let k = out.len();
            return Reply::finish(out).say(format!("{label}: {k} stones"));
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = st
        .stones
        .iter()
        .map(|s| girdle(&resized(&base, s.3), &Frame::new(s.0, s.1, s.2)))
        .collect();
    if let Input::Hover { point } = &e.input
        && let Ok((f, g)) = place(*point, &st)
    {
        preview.extend(g.mesh().placed(&f).wire());
        // The spacing guide around the stone.
        preview.push(girdle(&resized(&g, g.length + 2.0 * gap), &f));
    }
    let r = Reply::ask(
        format!(
            "{label}: click on the surface ({} placed), Enter to finish",
            st.stones.len()
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

// ---- tapered baguettes -----------------------------------------------------------------------

/// A step-cut stone from four corner points (a tapered baguette).
pub fn tapered(corners: [Vec3; 4], up: Vec3) -> Option<(Frame, Gem)> {
    let c = scale(corners.iter().fold([0.0; 3], |a, &p| add(a, p)), 0.25);
    let x = sub(add(corners[1], corners[2]), add(corners[0], corners[3]));
    let f = Frame::new(c, up, x);
    let local: Vec<[f64; 2]> = corners
        .iter()
        .map(|&p| {
            let l = f.local(p);
            [l[0], l[1]]
        })
        .collect();
    let mut g = Gem::from_outline(&local)?;
    // A step cut, not a brilliant: the cut sets the proportions.
    g.cut = Cut::Custom;
    Some((f, g))
}

fn taper_channel(e: &Event) -> Reply {
    let width = e.number("tbc_width", 2.0);
    let length = e.number("tbc_length", 1.5);
    let gap = e.number("tbc_spacing", 0.1);
    let facing = Facing::of(e.text("tbc_facing", "cplane"));
    let z_off = e.number("tbc_z_offset", 0.0);
    let mat = material(e, "tbc");
    built(
        e,
        "Baguette Channel",
        "select the center curve of the channel, Enter",
        || {
            let cs = curves(e);
            let c = cs.first().ok_or("select a curve")?;
            let path = path_of(c).ok_or("curve")?;
            let len = path.length();
            let n = if c.closed {
                (len / (length + gap)).floor() as usize
            } else {
                ((len + gap) / (length + gap)).floor() as usize
            };
            let step = if c.closed {
                len / n.max(1) as f64
            } else {
                length + gap
            };
            let mut out = Vec::new();
            for k in 0..n {
                let s0 = k as f64 * step + if c.closed { gap / 2.0 } else { 0.0 };
                let s1 = s0 + step - gap;
                let f0 = frame_on(&path, s0, e, facing, false, 0.0, z_off);
                let f1 = frame_on(&path, s1, e, facing, false, 0.0, z_off);
                // Ends square to the curve: the outer side is longer on a bend.
                let quad = [
                    f0.point([0.0, -width / 2.0, 0.0]),
                    f1.point([0.0, -width / 2.0, 0.0]),
                    f1.point([0.0, width / 2.0, 0.0]),
                    f0.point([0.0, width / 2.0, 0.0]),
                ];
                let up = scale(add(f0.z, f1.z), 0.5);
                if let Some((f, g)) = tapered(quad, up) {
                    out.push(stone(&g, &f, "baguette_channel", &mat));
                }
            }
            let k = out.len();
            Ok((out, format!("{k} tapered baguettes {} mm wide", fmt(width))))
        },
    )
}

fn taper_between(e: &Event) -> Reply {
    let count = e.number("tbb_count", 0.0).round() as usize;
    let length = e.number("tbb_length", 1.5);
    let gap = e.number("tbb_spacing", 0.1);
    let inset = e.number("tbb_inset", 0.1);
    let mat = material(e, "tbb");
    built(
        e,
        "Tapered Baguettes Between Two Curves",
        "select two curves, Enter",
        || {
            let cs = curves(e);
            if cs.len() != 2 {
                return Err("select two curves".into());
            }
            let (pa, mut pb) = (
                path_of(cs[0]).ok_or("curve")?,
                path_of(cs[1]).ok_or("curve")?,
            );
            let mut reversed = false;
            if dist(pa.at(0.0), pb.at(0.0)) > dist(pa.at(0.0), pb.at(pb.length())) {
                let mut p = cs[1].points.clone();
                p.reverse();
                pb = Path::new(&p, cs[1].closed).ok_or("curve")?;
                reversed = true;
            }
            let _ = reversed;
            let mid = (pa.length() + pb.length()) / 2.0;
            let n = if count > 0 {
                count
            } else {
                ((mid + gap) / (length + gap)).floor().max(1.0) as usize
            };
            let (la, lb) = (pa.length(), pb.length());
            let ga = gap / mid * la;
            let gb = gap / mid * lb;
            let mut out = Vec::new();
            for k in 0..n {
                let (t0, t1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
                let a0 = pa.at(t0 * la + ga / 2.0);
                let a1 = pa.at(t1 * la - ga / 2.0);
                let b0 = pb.at(t0 * lb + gb / 2.0);
                let b1 = pb.at(t1 * lb - gb / 2.0);
                // Pull the long sides in from the curves.
                let ins = |p: Vec3, q: Vec3| add(p, scale(unit(sub(q, p)), inset));
                let quad = [ins(a0, b0), ins(a1, b1), ins(b1, a1), ins(b0, a0)];
                if let Some((f, g)) = tapered(quad, e.plane.normal()) {
                    out.push(stone(&g, &f, "tapered_between_curves", &mat));
                }
            }
            let k = out.len();
            Ok((out, format!("{k} tapered baguettes")))
        },
    )
}

// ---- Cluster ---------------------------------------------------------------------------------

/// Rings of stones around a center stone of size `center` (local frame of the cluster).
pub fn cluster_rings(center: f64, rows: &[(f64, f64)], gap: f64) -> Vec<(Frame, f64)> {
    let mut out = vec![(Frame::WORLD, center)];
    let mut r_edge = center / 2.0;
    for (k, &(d, tilt)) in rows.iter().enumerate() {
        let r = r_edge + gap + d / 2.0;
        let n = ((TAU * r) / (d + gap)).floor().max(3.0) as usize;
        let drop = -(k as f64 + 1.0) * tilt.to_radians().sin() * d * 0.5;
        for i in 0..n {
            let a = TAU * i as f64 / n as f64 + if k % 2 == 1 { PI / n as f64 } else { 0.0 };
            let radial = [a.cos(), a.sin(), 0.0];
            let tilt_r = tilt.to_radians() * (k as f64 + 1.0);
            let z = add(
                scale([0.0, 0.0, 1.0], tilt_r.cos()),
                scale(radial, tilt_r.sin()),
            );
            out.push((
                Frame::new(
                    [r * a.cos(), r * a.sin(), drop],
                    z,
                    [-a.sin(), a.cos(), 0.0],
                ),
                d,
            ));
        }
        r_edge = r + d / 2.0;
    }
    out
}

fn cluster(e: &Event) -> Reply {
    let label = "Cluster Builder";
    let center_size = e.number("cl_center_size", 4.0);
    let rows: Vec<(f64, f64)> = (1..=e.number("cl_rows", 1.0).round().clamp(1.0, 3.0) as usize)
        .map(|k| {
            (
                e.number(&format!("cl_row{k}_size"), 1.5),
                e.number("cl_tilt", 10.0),
            )
        })
        .collect();
    let gap = e.number("cl_spacing", 0.2);
    let mat = material(e, "cl");
    let mut st: Placed = load(e);
    let (gems, _) = selected_gems(e);
    let make = |f: &Frame, center: Option<&PlacedGem>| -> Vec<Geo> {
        let c = center.map_or(center_size, |g| g.gem.length.max(g.gem.width));
        let rings = cluster_rings(c, &rows, gap);
        let mut out = Vec::new();
        for (i, (lf, d)) in rings.iter().enumerate() {
            if i == 0 && center.is_some() {
                continue;
            }
            let wf = Frame::new(f.point(lf.origin), f.vector(lf.z), f.vector(lf.x));
            out.push(stone(&Gem::new(Cut::Round, *d, *d), &wf, "cluster", &mat));
        }
        out
    };
    if !gems.is_empty() {
        return built(e, label, "select the center stone, Enter", || {
            let out: Vec<Geo> = gems.iter().flat_map(|g| make(&g.frame, Some(g))).collect();
            let n = out.len();
            Ok((out, format!("{n} stones around {} centers", gems.len())))
        });
    }
    match &e.input {
        Input::Point { point } => {
            st.stones
                .push((*point, e.plane.normal(), e.plane.x, center_size))
        }
        Input::Undo => {
            st.stones.pop();
        }
        Input::Enter if !st.stones.is_empty() => {
            let out: Vec<Geo> = st
                .stones
                .iter()
                .flat_map(|s| make(&Frame::new(s.0, s.1, s.2), None))
                .collect();
            let n = out.len();
            return Reply::finish(out).say(format!("{label}: {n} stones"));
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = Vec::new();
    if let Input::Hover { point } = &e.input {
        let f = Frame::new(*point, e.plane.normal(), e.plane.x);
        for (lf, d) in cluster_rings(center_size, &rows, gap) {
            let wf = Frame::new(f.point(lf.origin), f.vector(lf.z), f.vector(lf.x));
            preview.push(girdle(&Gem::new(Cut::Round, d, d), &wf));
        }
    }
    Reply::ask(
        format!(
            "{label}: click the cluster center ({} placed) or select a center stone, Enter",
            st.stones.len()
        ),
        save(&st),
    )
    .want(Want::Point)
    .preview(preview)
}

// ---- Auto Pavé -------------------------------------------------------------------------------

fn inside(poly: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < a[0] + (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1])
        {
            c = !c;
        }
    }
    c
}

fn edge_distance(poly: &[[f64; 2]], p: [f64; 2]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            let ab = [b[0] - a[0], b[1] - a[1]];
            let l2 = ab[0] * ab[0] + ab[1] * ab[1];
            let t = if l2 > 0.0 {
                (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            ((p[0] - a[0] - ab[0] * t).powi(2) + (p[1] - a[1] - ab[1] * t).powi(2)).sqrt()
        })
        .fold(f64::MAX, f64::min)
}

/// Pavé inside closed outlines (CPlane xy): a hexagonal grid of stones of size `d`, smaller
/// stones (down to `min`) where the full size does not fit the margin. Holes are outlines
/// inside others. Returns (center, size).
pub fn pave_fill(
    outlines: &[Vec<[f64; 2]>],
    d: f64,
    min: f64,
    gap: f64,
    margin: f64,
    angle: f64,
) -> Vec<([f64; 2], f64)> {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for o in outlines {
        for p in o {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
    }
    let pitch = d + gap;
    let row = pitch * 3f64.sqrt() / 2.0;
    let (s, c) = angle.to_radians().sin_cos();
    let center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let reach = ((hi[0] - lo[0]).hypot(hi[1] - lo[1])) / 2.0 + pitch;
    let inside_all = |p: [f64; 2]| outlines.iter().filter(|o| inside(o, p)).count() % 2 == 1;
    let clearance = |p: [f64; 2]| {
        outlines
            .iter()
            .map(|o| edge_distance(o, p))
            .fold(f64::MAX, f64::min)
    };
    let mut out: Vec<([f64; 2], f64)> = Vec::new();
    let ny = (reach / row).ceil() as i64;
    let nx = (reach / pitch).ceil() as i64 + 1;
    for j in -ny..=ny {
        for i in -nx..=nx {
            let u = i as f64 * pitch
                + if j.rem_euclid(2) == 1 {
                    pitch / 2.0
                } else {
                    0.0
                };
            let v = j as f64 * row;
            let p = [center[0] + u * c - v * s, center[1] + u * s + v * c];
            if !inside_all(p) {
                continue;
            }
            let room = clearance(p) - margin;
            let size = (2.0 * room).min(d);
            if size >= min - 1e-9 {
                out.push((p, size));
            }
        }
    }
    out
}

fn auto_pave(e: &Event) -> Reply {
    let d = e.number("ap_size", 1.2);
    let min = e.number("ap_min_size", 0.8);
    let gap = e.number("ap_spacing", 0.15);
    let margin = e.number("ap_margin", 0.3);
    let angle = e.number("ap_angle", 0.0);
    let z_off = e.number("ap_z_offset", 0.0);
    let mat = material(e, "ap");
    built(
        e,
        "Auto Pavé",
        "select closed curves (and the surface to pavé), Enter",
        || {
            let cs: Vec<&Selected> = curves(e).into_iter().filter(|c| c.closed).collect();
            if cs.is_empty() {
                return Err("select a closed curve".into());
            }
            let outlines: Vec<Vec<[f64; 2]>> = cs
                .iter()
                .map(|c| {
                    c.points
                        .iter()
                        .map(|&p| {
                            let l = e.plane.local(p);
                            [l[0], l[1]]
                        })
                        .collect()
                })
                .collect();
            let z0 = cs[0]
                .points
                .iter()
                .map(|&p| e.plane.local(p)[2])
                .sum::<f64>()
                / cs[0].points.len() as f64;
            let n = e.plane.normal();
            let ms = surfaces(e);
            let target = ms.first().and_then(|m| Target::new(m, n));
            let mut out = Vec::new();
            for (p, size) in pave_fill(&outlines, d, min, gap, margin, angle) {
                let flat = e.plane.at(p[0], p[1], z0);
                let (o, z) = target
                    .as_ref()
                    .and_then(|t| t.drop(add(flat, scale(n, 1e6))))
                    .unwrap_or((flat, n));
                let f = Frame::new(o, z, e.plane.x).moved([0.0, 0.0, z_off]);
                out.push(stone(
                    &Gem::new(Cut::Round, size, size),
                    &f,
                    "auto_pave",
                    &mat,
                ));
            }
            let k = out.len();
            Ok((out, format!("{k} stones")))
        },
    )
}

// ---- Azure, pavé prongs ----------------------------------------------------------------------

/// An azure (opening under a stone): from just under the girdle down `depth`, opening by
/// `angle` degrees; round or square.
pub fn azure_for(g: &PlacedGem, scale_by: f64, depth: f64, angle: f64, square: bool) -> Mesh {
    let r0 = g.gem.length.min(g.gem.width) / 2.0 * scale_by;
    let (_, bottom) = g.gem.girdle();
    let r1 = r0 + depth * angle.to_radians().tan();
    let segs = if square { 4 } else { 24 };
    let ring = |r: f64, z: f64| -> Vec<Vec3> {
        (0..segs)
            .map(|k| {
                let a = TAU * k as f64 / segs as f64 + if square { PI / 4.0 } else { 0.0 };
                let rr = if square { r * 2f64.sqrt() } else { r };
                g.frame.point([rr * a.cos(), rr * a.sin(), z])
            })
            .collect()
    };
    mesh::ring_stack(&[ring(r0, bottom), ring(r1, bottom - depth)], None, None)
}

fn azure(e: &Event) -> Reply {
    let s = e.number("az_scale", 70.0) / 100.0;
    let depth = e.number("az_depth", 1.5);
    let angle = e.number("az_angle", 15.0);
    let square = e.text("az_shape", "round") == "square";
    built(
        e,
        "Pavé Azure Builder",
        "select the pavé stones, Enter",
        || {
            let (gems, _) = selected_gems(e);
            let out: Vec<Geo> = gems
                .iter()
                .map(|g| {
                    azure_for(g, s, depth, angle, square).geo("Azure cutter", None, Some("Cutting"))
                })
                .collect();
            let n = out.len();
            Ok((out, format!("{n} azure cutters")))
        },
    )
}

/// Shared prong positions for a pavé layout: one between every three neighbouring stones,
/// and at the open sides of the layout. Each: (point at the girdle height, axis, diameter).
pub fn pave_prong_points(
    gems: &[PlacedGem],
    reach: f64,
    size: f64,
    edges: bool,
) -> Vec<(Vec3, Vec3, f64)> {
    let n = gems.len();
    let r = |i: usize| gems[i].gem.length.min(gems[i].gem.width) / 2.0;
    let c = |i: usize| gems[i].frame.origin;
    let near = |i: usize, j: usize| dist(c(i), c(j)) <= r(i) + r(j) + reach;
    let mut pairs = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            if near(i, j) {
                pairs.push((i, j));
            }
        }
    }
    let mut tri_count: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut out: Vec<(Vec3, Vec3, f64)> = Vec::new();
    for &(i, j) in &pairs {
        for k in j + 1..n {
            if near(i, k) && near(j, k) {
                // The point between the three girdles, weighted by their sizes.
                let w = [r(j) + r(k), r(i) + r(k), r(i) + r(j)];
                let p = scale(
                    add(add(scale(c(i), w[0]), scale(c(j), w[1])), scale(c(k), w[2])),
                    1.0 / (w[0] + w[1] + w[2]),
                );
                let axis = unit(add(add(gems[i].frame.z, gems[j].frame.z), gems[k].frame.z));
                let d = size * (r(i) + r(j) + r(k)) / 1.5;
                out.push((p, axis, d));
                for e in [(i, j), (i, k), (j, k)] {
                    tri_count.entry(e).or_default().push(i + j + k - e.0 - e.1);
                }
            }
        }
    }
    if edges {
        for &(i, j) in &pairs {
            let thirds = tri_count.get(&(i, j)).cloned().unwrap_or_default();
            if thirds.len() >= 2 {
                continue;
            }
            let mid = scale(add(c(i), c(j)), 0.5);
            let axis = unit(add(gems[i].frame.z, gems[j].frame.z));
            let along = unit(sub(c(j), c(i)));
            let side = unit(cross(axis, along));
            let off = (r(i) + r(j)) / 2.0 * 0.85;
            let d = size * (r(i) + r(j));
            let sides: Vec<f64> = match thirds.first() {
                // Push away from the stone that closes the triangle.
                Some(&k) => vec![if dot(sub(c(k), mid), side) > 0.0 {
                    -1.0
                } else {
                    1.0
                }],
                None => vec![1.0, -1.0],
            };
            for sgn in sides {
                out.push((add(mid, scale(side, sgn * off)), axis, d));
            }
        }
    }
    // Drop prongs that land on each other.
    let mut kept: Vec<(Vec3, Vec3, f64)> = Vec::new();
    for p in out {
        if kept.iter().all(|q| dist(q.0, p.0) > (q.2 + p.2) * 0.4) {
            kept.push(p);
        }
    }
    kept
}

fn pave_prongs(e: &Event) -> Reply {
    let size = e.number("pp_size", 30.0) / 100.0;
    let height = e.number("pp_height", 0.4);
    let drop = e.number("pp_drop", 1.0);
    let reach = e.number("pp_reach", 0.4);
    let edges = e.flag("pp_edges", true);
    built(
        e,
        "Pavé Prong Builder",
        "select the pavé stones, Enter",
        || {
            let (gems, _) = selected_gems(e);
            if gems.len() < 2 {
                return Err("select two or more stones".into());
            }
            let out: Vec<Geo> = pave_prong_points(&gems, reach, size, edges)
                .into_iter()
                .map(|(p, axis, d)| {
                    let f = Frame::new(p, axis, [1.0, 0.0, 0.0]);
                    mesh::domed_cylinder(d / 2.0, -drop, height, 16)
                        .placed(&f)
                        .geo("Prong", None, None)
                })
                .collect();
            let n = out.len();
            Ok((out, format!("{n} shared prongs for {} stones", gems.len())))
        },
    )
}

// ---- layout editing --------------------------------------------------------------------------

/// The per-stone matrices taking each stone's frame to its new one.
fn moved_stones(gems: &[PlacedGem], frames: &[Frame]) -> Vec<ObjectMatrix> {
    gems.iter()
        .zip(frames)
        .map(|(g, f)| ObjectMatrix {
            id: g.id.clone(),
            matrix: frame_matrix(&g.frame, f, 1.0),
        })
        .collect()
}

fn preview_frames(gems: &[PlacedGem], frames: &[Frame]) -> Vec<Geo> {
    gems.iter()
        .zip(frames)
        .map(|(g, f)| girdle(&g.gem, f))
        .collect()
}

fn finish_moves(
    e: &Event,
    label: &str,
    gems: &[PlacedGem],
    frames: Vec<Frame>,
    what: String,
) -> Reply {
    if matches!(e.input, Input::Enter) {
        return Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: false,
            per_object: moved_stones(gems, &frames),
        })
        .say(format!("{label}: {what}"));
    }
    Reply::ask(
        format!("{label}: {what}. Adjust in the builder, Enter to apply"),
        json!({}),
    )
    .want(Want::Objects)
    .preview(preview_frames(gems, &frames))
}

/// Relax stones so neighbours keep `gap` between girdles, staying on the surface.
pub fn relax(
    gems: &[PlacedGem],
    gap: f64,
    iterations: usize,
    strength: f64,
    surface: Option<&SelectedMesh>,
) -> Vec<Frame> {
    let mut f: Vec<Frame> = gems.iter().map(|g| g.frame).collect();
    let r: Vec<f64> = gems
        .iter()
        .map(|g| g.gem.length.max(g.gem.width) / 2.0)
        .collect();
    let n = gems.len();
    for _ in 0..iterations {
        let mut push = vec![[0.0; 3]; n];
        for i in 0..n {
            for j in i + 1..n {
                let d = sub(f[j].origin, f[i].origin);
                let l = norm(d);
                let want = r[i] + r[j] + gap;
                if l < want * 1.25 && l > 1e-9 {
                    let delta = scale(unit(d), (l - want) * 0.5 * strength);
                    push[i] = add(push[i], delta);
                    push[j] = sub(push[j], delta);
                }
            }
        }
        for i in 0..n {
            let z = f[i].z;
            // Move in the stone's own girdle plane only.
            let along = sub(push[i], scale(z, dot(push[i], z)));
            let mut o = add(f[i].origin, along);
            let mut axis = z;
            if let Some(m) = surface {
                let lift = dot(
                    sub(
                        gems[i].frame.origin,
                        on_mesh(m, gems[i].frame.origin, Some(z))
                            .map_or(gems[i].frame.origin, |x| x.0),
                    ),
                    z,
                );
                if let Some((q, nn)) = on_mesh(m, o, Some(z)) {
                    o = add(q, scale(nn, lift));
                    axis = nn;
                }
            }
            f[i] = Frame::new(o, axis, f[i].x);
        }
    }
    f
}

fn springs(e: &Event) -> Reply {
    let gap = e.number("gs_spacing", 0.15);
    let it = e.number("gs_iterations", 30.0).round().clamp(1.0, 500.0) as usize;
    let strength = e.number("gs_strength", 50.0) / 100.0;
    let (gems, _) = selected_gems(e);
    if gems.len() < 2 {
        let r = Reply::ask(
            "Gem Springs: select the stones (and the surface they sit on), Enter",
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse("Gem Springs: select two or more stones")
        } else {
            r
        };
    }
    let ms = surfaces(e);
    let frames = relax(&gems, gap, it, strength, ms.first().copied());
    finish_moves(
        e,
        "Gem Springs",
        &gems,
        frames,
        format!("{} stones relaxed to {} mm gaps", gems.len(), fmt(gap)),
    )
}

fn follow(e: &Event) -> Reply {
    let across = e.text("gf_north", "along") == "across";
    let (gems, _) = selected_gems(e);
    if gems.len() < 2 {
        let r =
            Reply::ask("Gem Follow: select a line of stones, Enter", json!({})).want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse("Gem Follow: select two or more stones")
        } else {
            r
        };
    }
    let (order, closed) = om9_addon_geom::gems::chain(&gems);
    let n = order.len();
    let mut frames: Vec<Frame> = gems.iter().map(|g| g.frame).collect();
    for (k, &i) in order.iter().enumerate() {
        let prev = if k > 0 {
            order[k - 1]
        } else if closed {
            order[n - 1]
        } else {
            i
        };
        let next = if k + 1 < n {
            order[k + 1]
        } else if closed {
            order[0]
        } else {
            i
        };
        let dir = sub(gems[next].frame.origin, gems[prev].frame.origin);
        let g = &gems[i];
        let north = if across { cross(g.frame.z, dir) } else { dir };
        frames[i] = Frame::new(g.frame.origin, g.frame.z, north);
    }
    finish_moves(
        e,
        "Gem Follow",
        &gems,
        frames,
        format!("{n} stones point their north along the line"),
    )
}

fn control(e: &Event) -> Reply {
    let flow = Flow::new("Gem Control")
        .choice("Control", &["Culet", "North", "Both"], "Culet")
        .choice("Direction", &["Toward", "Away"], "Toward")
        .point("Control point (culets / north point at it)");
    let make = |c: &Collected, p: Vec3| -> (Vec<PlacedGem>, Vec<Frame>) {
        let (gems, _) = selected_gems(e);
        let away = c.opt(&flow, "Direction") == "Away";
        let what = c.opt(&flow, "Control");
        let frames = gems
            .iter()
            .map(|g| {
                let to = unit(sub(p, g.frame.origin));
                let to = if away { scale(to, -1.0) } else { to };
                let mut f = g.frame;
                if what != "North" {
                    // The culet points at the control point: the table axis away from it.
                    f = Frame::new(f.origin, scale(to, -1.0), f.x);
                }
                if what != "Culet" {
                    f = Frame::new(f.origin, f.z, to);
                }
                f
            })
            .collect();
        (gems, frames)
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: h
                .map(|p| {
                    let (g, f) = make(c, p);
                    preview_frames(&g, &f)
                })
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let (gems, frames) = make(c, c.p(0).unwrap());
            if gems.is_empty() {
                return Err("select stones".into());
            }
            let n = gems.len();
            Ok(Reply::finish_transform(TransformOp {
                matrices: Vec::new(),
                keep_original: false,
                per_object: moved_stones(&gems, &frames),
            })
            .say(format!("Gem Control: {n} stones")))
        },
    )
}

fn positioner(e: &Event) -> Reply {
    let flow = Flow::new("Gem Positioner")
        .number("Move along the stone's X", 0.0)
        .number("Move along the stone's Y", 0.0)
        .number("Move along the stone's Z", 0.0)
        .number("Rotate about the stone's axis (degrees)", 0.0)
        .number("Size (% of now)", 100.0);
    let make = |c: &Collected| -> Vec<(PlacedGem, Frame, Gem)> {
        let (gems, _) = selected_gems(e);
        let v = |k: usize, d: f64| c.n(k).unwrap_or(d);
        gems.into_iter()
            .map(|g| {
                let f = g
                    .frame
                    .moved([v(0, 0.0), v(1, 0.0), v(2, 0.0)])
                    .rotated(v(3, 0.0).to_radians());
                let ng = resized(&g.gem, g.gem.length * v(4, 100.0) / 100.0);
                (g, f, ng)
            })
            .collect()
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c).iter().map(|(_, f, g)| girdle(g, f)).collect(),
            ..Default::default()
        },
        &|c| {
            let moved = make(c);
            if moved.is_empty() {
                return Err("select stones".into());
            }
            let n = moved.len();
            let mut r = Reply::finish(
                moved
                    .iter()
                    .map(|(g, f, ng)| stone(ng, f, "gem_positioner", &g.material))
                    .collect(),
            )
            .say(format!("Gem Positioner: {n} stones"));
            r.delete = moved.iter().map(|x| x.0.id.clone()).collect();
            Ok(r)
        },
    )
}

// ---- Gem Flow, Gem Splop ---------------------------------------------------------------------

fn gem_flow(e: &Event) -> Reply {
    let copy = e.flag("gfl_copy", false);
    let stretch = e.flag("gfl_stretch", true);
    built(
        e,
        "Gem Flow",
        "select the stones, the base curve, then the target curve, Enter",
        || {
            let (gems, _) = selected_gems(e);
            let cs = curves(e);
            if gems.is_empty() || cs.len() != 2 {
                return Err("select stones and two curves (base first)".into());
            }
            let (base, target) = (
                path_of(cs[0]).ok_or("curve")?,
                path_of(cs[1]).ok_or("curve")?,
            );
            let k = if stretch {
                target.length() / base.length()
            } else {
                1.0
            };
            let up = Some(e.plane.normal());
            let mut out = Vec::new();
            for g in &gems {
                let s = base.closest(g.frame.origin);
                let from = base.frame(s, up);
                let to = target.frame((s * k).min(target.length()), None);
                let m = frame_matrix(&from, &to, 1.0);
                let o = apply(&m, g.frame.origin);
                let z = sub(apply(&m, add(g.frame.origin, g.frame.z)), o);
                let x = sub(apply(&m, add(g.frame.origin, g.frame.x)), o);
                out.push(stone(&g.gem, &Frame::new(o, z, x), "gem_flow", &g.material));
            }
            let n = out.len();
            Ok((
                out,
                format!(
                    "{n} stones flowed rigidly{}",
                    if copy { " (originals kept)" } else { "" }
                ),
            ))
        },
    )
    .with_delete_on_finish(if copy {
        Vec::new()
    } else {
        selected_gems(e).0.iter().map(|g| g.id.clone()).collect()
    })
}

trait DeleteOnFinish {
    fn with_delete_on_finish(self, ids: Vec<String>) -> Self;
}

impl DeleteOnFinish for Reply {
    fn with_delete_on_finish(mut self, ids: Vec<String>) -> Self {
        if self.done {
            self.delete = ids;
        }
        self
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Splop {
    /// The stones to splop, as selected (their records).
    stones: Vec<Selected>,
    targets: Vec<(Vec3, Vec3)>,
    surface: Option<SelectedMesh>,
}

fn gem_splop(e: &Event) -> Reply {
    let label = "Gem Splop";
    let mut st: Splop = load(e);
    let rot = e.number("gsp_rotation", 0.0).to_radians();
    let scale_by = e.number("gsp_scale", 100.0) / 100.0;
    let (gems, _) = selected_gems(e);
    if st.stones.is_empty() {
        if matches!(e.input, Input::Enter) && !gems.is_empty() {
            st.stones = e
                .selection
                .iter()
                .filter(|s| gems.iter().any(|g| g.id == s.id))
                .cloned()
                .collect();
            return Reply::ask(format!("{label}: {} stones (laid out around the CPlane origin). Select the surface, Enter", gems.len()), save(&st)).want(Want::Objects);
        }
        return Reply::ask(
            format!("{label}: select the stones to splop, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    }
    if st.surface.is_none() {
        let ms = surfaces(e);
        if matches!(e.input, Input::Enter)
            && let Some(m) = ms.first()
        {
            st.surface = Some((*m).clone());
            return Reply::ask(
                format!("{label}: pick the splop points on the surface, Enter when done"),
                save(&st),
            )
            .want(Want::Point);
        }
        return Reply::ask(
            format!("{label}: select the target surface, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    }
    let m = st.surface.clone().unwrap();
    let n = e.plane.normal();
    let target = Target::new(&m, n);
    let from = Frame::new(e.plane.origin, n, e.plane.x);
    let placed: Vec<PlacedGem> = st
        .stones
        .iter()
        .filter_map(|s| PlacedGem::from_selected(s, n))
        .collect();
    let frames_at = |hit: (Vec3, Vec3)| -> Vec<Frame> {
        let to = Frame::new(hit.0, hit.1, e.plane.x).rotated(rot);
        let mtx = frame_matrix(&from, &to, scale_by);
        placed
            .iter()
            .map(|g| (g.frame.origin, g.frame.z, g.frame.x))
            .map(|f| {
                let o = apply(&mtx, f.0);
                // Each stone sits on the surface under it, facing out.
                let o2 = on_mesh(&m, o, Some(hit.1)).map_or((o, hit.1), |x| x);
                Frame::new(o2.0, o2.1, sub(apply(&mtx, add(f.0, f.2)), o))
            })
            .collect()
    };
    match &e.input {
        Input::Point { point } => {
            if let Some(h) = target
                .as_ref()
                .and_then(|t| t.drop(add(*point, scale(n, 1e6))))
            {
                st.targets.push(h);
            }
        }
        Input::Undo => {
            st.targets.pop();
        }
        Input::Enter if !st.targets.is_empty() => {
            let mut out = Vec::new();
            for h in &st.targets {
                for (g, f) in placed.iter().zip(frames_at(*h)) {
                    out.push(stone(
                        &resized(&g.gem, g.gem.length * scale_by),
                        &f,
                        "gem_splop",
                        &g.material,
                    ));
                }
            }
            let k = out.len();
            return Reply::finish(out).say(format!("{label}: {k} stones"));
        }
        _ => {}
    }
    let mut preview = Vec::new();
    let mut hits = st.targets.clone();
    if let Input::Hover { point } = &e.input
        && let Some(h) = target
            .as_ref()
            .and_then(|t| t.drop(add(*point, scale(n, 1e6))))
    {
        hits.push(h);
    }
    for h in hits {
        for f in frames_at(h) {
            preview.push(Geo::Line {
                start: f.origin,
                end: add(f.origin, scale(f.z, 1.0)),
            });
        }
    }
    Reply::ask(
        format!(
            "{label}: pick splop points ({} so far), Enter",
            st.targets.len()
        ),
        save(&st),
    )
    .want(Want::Point)
    .preview(preview)
}

// ---- Pavé Sphere -----------------------------------------------------------------------------

/// `n` points spread on a unit sphere (a Fibonacci spiral), then pushed apart until no two
/// are closer than the chord `target`.
pub fn sphere_points(n: usize, target: f64) -> Vec<Vec3> {
    let golden = PI * (3.0 - 5f64.sqrt());
    let mut p: Vec<Vec3> = (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let r = (1.0 - y * y).sqrt();
            let a = golden * i as f64;
            [r * a.cos(), r * a.sin(), y]
        })
        .collect();
    for _ in 0..80 {
        let q = p.clone();
        for i in 0..n {
            let mut f = [0.0; 3];
            for j in 0..n {
                if i != j {
                    let d = sub(q[i], q[j]);
                    let l = norm(d);
                    if l < target && l > 1e-12 {
                        f = add(f, scale(d, (target - l) / l * 0.5));
                    }
                }
            }
            p[i] = unit(add(q[i], f));
        }
    }
    p
}

fn pave_sphere(e: &Event) -> Reply {
    let radius = e.number("ps_radius", 5.0);
    let d = e.number("ps_size", 1.2);
    let gap = e.number("ps_spacing", 0.15);
    let with_ball = e.flag("ps_sphere", true);
    let mat = material(e, "ps");
    let flow = Flow::new("Pavé Sphere")
        .no_selection()
        .point("Center of the sphere");
    let make = |c: Vec3| -> Vec<Geo> {
        // Hexagonal density of stones of pitch d + gap over the sphere area.
        // A sphere takes about 84 % of the flat hexagonal density without crowding.
        let pitch = d + gap;
        let n = (0.84 * 4.0 * PI * radius * radius / (pitch * pitch * 3f64.sqrt() / 2.0))
            .floor()
            .max(4.0) as usize;
        let mut out = Vec::new();
        if with_ball {
            out.push(
                mesh::sphere(radius, 48)
                    .placed(&Frame::new(c, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]))
                    .geo("Pavé sphere", None, None),
            );
        }
        for u in sphere_points(n, (pitch / radius).min(2.0)) {
            let f = Frame::new(add(c, scale(u, radius)), u, cross([0.0, 0.0, 1.0], u));
            out.push(stone(&Gem::new(Cut::Round, d, d), &f, "pave_sphere", &mat));
        }
        out
    };
    flow.step(
        e,
        &|_, h| Show {
            geo: h
                .map(|c| {
                    make(c)
                        .into_iter()
                        .skip(usize::from(with_ball))
                        .take(400)
                        .map(|g| match g {
                            Geo::Mesh { vertices, .. } => Geo::Point { point: vertices[0] },
                            g => g,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let out = make(c.p(0).unwrap());
            let n = out.len() - usize::from(with_ball);
            Ok(Reply::finish(out).say(format!(
                "Pavé Sphere: {n} stones {} mm on a {} mm sphere",
                fmt(d),
                fmt(2.0 * radius)
            )))
        },
    )
}

// ---- styles ----------------------------------------------------------------------------------

/// Rebuild the settings and cutters of `styles` on `targets` (through their own builders),
/// and the records the targets keep. Material is copied when `material` is given.
pub fn apply_styles(
    e: &Event,
    targets: &[Selected],
    styles: &BTreeMap<String, Value>,
    material: Option<&str>,
) -> (Vec<Geo>, Vec<RecordUpdate>) {
    let mut create = Vec::new();
    for (code, values) in styles {
        let settings: BTreeMap<String, Value> = values
            .as_object()
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        let ev = Event {
            command: code.clone(),
            input: Input::Enter,
            state: Value::Null,
            plane: e.plane,
            selection: targets.to_vec(),
            pick: None,
            settings,
        };
        let r = if <om9_settings::Settings as Interactive>::SESSION_COMMANDS
            .contains(&code.as_str())
        {
            <om9_settings::Settings as Interactive>::step(&ev)
        } else if <gem_cutter::GemCutter as Interactive>::SESSION_COMMANDS.contains(&code.as_str())
        {
            <gem_cutter::GemCutter as Interactive>::step(&ev)
        } else {
            continue;
        };
        if let Ok(r) = r
            && r.done
        {
            create.extend(r.create);
        }
    }
    let records = targets
        .iter()
        .filter_map(|t| {
            let mut rec = t.gem.clone()?;
            let mut merged = rec
                .get("styles")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            for (k, v) in styles {
                merged.insert(k.clone(), v.clone());
            }
            rec["styles"] = Value::Object(merged);
            if let Some(m) = material {
                rec["material"] = json!(m);
            }
            Some(RecordUpdate {
                id: t.id.clone(),
                record: rec,
            })
        })
        .collect();
    (create, records)
}

#[derive(Default, Serialize, Deserialize)]
struct Targets {
    targets: Vec<Selected>,
}

fn match_attributes(e: &Event) -> Reply {
    let label = "Match Attributes";
    let mut st: Targets = load(e);
    let gem_sel: Vec<&Selected> = e.selection.iter().filter(|s| s.gem.is_some()).collect();
    if st.targets.is_empty() {
        if matches!(e.input, Input::Enter) && !gem_sel.is_empty() {
            st.targets = gem_sel.iter().map(|s| (*s).clone()).collect();
            return Reply::ask(
                format!(
                    "{label}: {} stones to style. Now select the stone whose style to copy, Enter",
                    st.targets.len()
                ),
                save(&st),
            )
            .want(Want::Objects);
        }
        return Reply::ask(
            format!("{label}: select the stones that get the style, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    }
    let ids: Vec<&str> = st.targets.iter().map(|t| t.id.as_str()).collect();
    let source = gem_sel.iter().find(|s| !ids.contains(&s.id.as_str()));
    let Some(source) = source else {
        return Reply::ask(
            format!("{label}: select the stone whose style to copy, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    };
    let mut styles = styles_of(source);
    for (opt, codes) in [
        ("ma_prongs", "BuilderProngAdder"),
        ("ma_bezel", "BuilderBezel"),
        ("ma_head", "BuilderHead"),
        ("ma_cutter", gem_cutter::GEM_CUTTER),
    ] {
        if !e.flag(opt, true) {
            styles.remove(codes);
        }
    }
    let mat = e.flag("ma_material", false).then(|| {
        source
            .gem
            .as_ref()
            .and_then(|g| g.get("material"))
            .and_then(Value::as_str)
            .unwrap_or("diamond")
            .to_owned()
    });
    if styles.is_empty() && mat.is_none() {
        let r = Reply::ask(
            format!("{label}: that stone remembers no setting or cutter style"),
            save(&st),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: the source stone has no style"))
        } else {
            r
        };
    }
    let (create, records) = apply_styles(e, &st.targets, &styles, mat.as_deref());
    let names: Vec<&str> = styles.keys().map(String::as_str).collect();
    if matches!(e.input, Input::Enter) {
        let mut r = Reply::finish(create).say(format!(
            "{label}: {} on {} stones",
            names.join(", "),
            st.targets.len()
        ));
        r.records = records;
        return r;
    }
    Reply::ask(
        format!(
            "{label}: {} → {} stones. Enter to create",
            names.join(", "),
            st.targets.len()
        ),
        save(&st),
    )
    .want(Want::Objects)
    .preview(create)
}

/// The folder for style files: the `style_folder` setting, else `OM9 Styles` in the home folder.
pub fn style_folder(e: &Event) -> std::path::PathBuf {
    let set = e.text("style_folder", "");
    if !set.trim().is_empty() {
        return std::path::PathBuf::from(set);
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .unwrap_or_default();
    std::path::Path::new(&home).join("OM9 Styles")
}

fn style_file(e: &Event) -> std::path::PathBuf {
    let slot = e.text("style_slot", "1");
    let name: String = slot
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    style_folder(e).join(format!(
        "{}.om9style",
        if name.is_empty() { "1" } else { &name }
    ))
}

fn save_style(e: &Event) -> Reply {
    let label = "Save Style";
    let gems: Vec<&Selected> = e.selection.iter().filter(|s| s.gem.is_some()).collect();
    let file = style_file(e);
    let Some(src) = gems.first() else {
        let r = Reply::ask(
            format!("{label}: select the stone whose style to save, Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a stone"))
        } else {
            r
        };
    };
    let styles = styles_of(src);
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(
            format!(
                "{label}: {} styles → {}. Enter to save",
                styles.len(),
                file.display()
            ),
            json!({}),
        )
        .want(Want::Objects);
    }
    if styles.is_empty() {
        return Reply::ask(format!("{label}: select a stone"), json!({}))
            .want(Want::Objects)
            .refuse(format!(
                "{label}: that stone remembers no setting or cutter style"
            ));
    }
    let g = src.gem.as_ref().unwrap();
    let doc = json!({"om9_style": 1, "cut": g.get("cut"), "material": g.get("material"), "styles": styles});
    let written = std::fs::create_dir_all(file.parent().unwrap_or(std::path::Path::new(".")))
        .and_then(|_| std::fs::write(&file, serde_json::to_vec_pretty(&doc).unwrap_or_default()));
    match written {
        Ok(()) => Reply::finish(Vec::new()).say(format!(
            "{label}: {} styles saved to {}",
            styles.len(),
            file.display()
        )),
        Err(err) => Reply::ask(label, json!({}))
            .want(Want::Objects)
            .refuse(format!("{label}: cannot write {}: {err}", file.display())),
    }
}

fn load_style(e: &Event) -> Reply {
    let label = "Load Style";
    let file = style_file(e);
    let targets: Vec<Selected> = e
        .selection
        .iter()
        .filter(|s| s.gem.is_some())
        .cloned()
        .collect();
    let doc: Result<Value, String> = std::fs::read(&file)
        .map_err(|err| format!("{}: {err}", file.display()))
        .and_then(|b| {
            serde_json::from_slice(&b).map_err(|err| format!("{}: {err}", file.display()))
        });
    let styles: BTreeMap<String, Value> = match &doc {
        Ok(d) => d
            .get("styles")
            .and_then(Value::as_object)
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        Err(_) => BTreeMap::new(),
    };
    if targets.is_empty() || styles.is_empty() {
        let r = Reply::ask(
            format!(
                "{label}: select the stones for style {}, Enter",
                file.display()
            ),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(match doc {
                Err(w) => format!("{label}: {w}"),
                Ok(_) if targets.is_empty() => format!("{label}: select stones"),
                Ok(_) => format!("{label}: the file has no styles"),
            })
        } else {
            r
        };
    }
    let (create, records) = apply_styles(e, &targets, &styles, None);
    if matches!(e.input, Input::Enter) {
        let mut r = Reply::finish(create).say(format!(
            "{label}: {} styles on {} stones",
            styles.len(),
            targets.len()
        ));
        r.records = records;
        return r;
    }
    Reply::ask(
        format!(
            "{label}: {} styles on {} stones, Enter to create",
            styles.len(),
            targets.len()
        ),
        json!({}),
    )
    .want(Want::Objects)
    .preview(create)
}

/// For tests and the section helpers.
pub fn wire(m: &SelectedMesh) -> Vec<Geo> {
    section::feature_edges(m, 30.0, false)
        .into_iter()
        .map(|s| Geo::Line {
            start: s[0],
            end: s[1],
        })
        .collect()
}

#[cfg(test)]
mod tests;
