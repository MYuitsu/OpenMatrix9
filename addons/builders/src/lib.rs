// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix-style **Builder** menu for OM9: eternity band, signet (and SubD signet), SubD
//! ring, text on curve, raised band, award ring top, rope, jump ring, knots (grid and
//! freeform), pattern and nautilus builders, mill work along curves and over areas.
//!
//! Rings follow the Matrix habit: the finger rail lies in the Front view (world XZ) around
//! the origin, the ring axis is world Y and the top of the ring is +Z.
#![allow(clippy::too_many_arguments)]
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, Replacement, Reply, Selected, TransformOp, Vec3, Want, add,
    compose, cross, fmt, mirror, scale, sub, translation, unit,
};
use om9_addon_geom::cmd::{Flow, Show};
use om9_addon_geom::csg::{self, Op};
use om9_addon_geom::gems::{Cut, Gem, PlacedGem};
use om9_addon_geom::mesh::{self, Frame, Mesh};
use om9_addon_geom::path::Path;
use om9_addon_geom::shapes;
use serde_json::json;
use std::f64::consts::{PI, TAU};

pub const ETERNITY: &str = "BuilderEternity";
pub const SIGNET: &str = "BuilderSignet";
pub const SUBD_SIGNET: &str = "ClayooCreationSignetRing";
pub const SUBD_RING: &str = "ClayooCreationRing";
pub const TEXT_ON_CURVE: &str = "TextOnCurve";
pub const RAISED: &str = "BuilderRaised";
pub const AWARD: &str = "BuilderAwardRingBuilder";
pub const ROPE: &str = "Builder_Rope";
pub const JUMP: &str = "BuilderJump";
pub const FREEFORM_KNOT: &str = "BuilderFreeFormKnot";
pub const KNOT: &str = "BuilderKnot";
pub const PATTERN: &str = "BuilderPatternBuilder";
pub const NAUTILUS: &str = "BuilderNautilusBuilder";
pub const MILL_WORK: &str = "BuilderMillWork";
pub const MILL_AREA: &str = "BuilderMillArea";
pub const MILL_WORK_C: &str = "BuilderMillWorkC";

pub struct Builders;

impl Interactive for Builders {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        ETERNITY,
        SIGNET,
        SUBD_SIGNET,
        SUBD_RING,
        TEXT_ON_CURVE,
        RAISED,
        AWARD,
        ROPE,
        JUMP,
        FREEFORM_KNOT,
        KNOT,
        PATTERN,
        NAUTILUS,
        MILL_WORK,
        MILL_AREA,
        MILL_WORK_C,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            ETERNITY => eternity(e),
            SIGNET | SUBD_SIGNET => signet(e),
            SUBD_RING => subd_ring(e),
            TEXT_ON_CURVE => text_on_curve(e),
            RAISED => raised(e),
            AWARD => award(e),
            ROPE => rope(e),
            JUMP => jump_ring(e),
            FREEFORM_KNOT => freeform_knot(e),
            KNOT => grid_knot(e),
            PATTERN => pattern(e),
            NAUTILUS => nautilus(e),
            MILL_WORK | MILL_WORK_C => mill_work(e),
            MILL_AREA => mill_area(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- shared ----------------------------------------------------------------------------------

/// Inner diameter (mm) of a ring size: US/Canada sizes, or a diameter typed directly.
pub fn inner_diameter(system: &str, size: f64) -> f64 {
    match system {
        "diameter" => size,
        // ISO 8653: the size is the inner circumference in mm.
        "iso" => size / PI,
        _ => 11.63 + 0.8128 * size,
    }
}

fn ring_radius(e: &Event, p: &str) -> f64 {
    inner_diameter(
        e.text(&format!("{p}_size_system"), "us"),
        e.number(&format!("{p}_ring_size"), 7.0),
    ) / 2.0
}

/// The point on the finger circle at angle `t` (0 = top, +Z; turning toward +X) and the
/// outward direction there.
fn on_rail(r: f64, t: f64) -> (Vec3, Vec3) {
    let n = [t.sin(), 0.0, t.cos()];
    (scale(n, r), n)
}

const AXIS: Vec3 = [0.0, 1.0, 0.0];

/// A closed band: around the finger circle of radius `r`, the section at angle `t` is
/// `section(t)`: points `(across, out)` (across along the ring axis, out from the finger),
/// counter-clockwise seen along the run.
pub fn band(r: f64, steps: usize, section: &dyn Fn(f64) -> Vec<[f64; 2]>) -> Option<Mesh> {
    let rings: Vec<Vec<Vec3>> = (0..steps)
        .map(|k| {
            let t = TAU * k as f64 / steps as f64;
            let (c, n) = on_rail(r, t);
            section(t)
                .iter()
                .map(|q| add(c, add(scale(AXIS, q[0]), scale(n, q[1]))))
                .collect()
        })
        .collect();
    mesh::loft_rings(&rings, true)
}

/// A profile shape `(across, out)` of `width` × `thickness`, counter-clockwise.
pub fn profile(kind: &str, width: f64, thickness: f64) -> Vec<[f64; 2]> {
    let (w, h) = (width / 2.0, thickness);
    let n = 16;
    match kind {
        // Flat inside, half-round outside.
        "d" => {
            let mut p = vec![[w, 0.0]];
            for k in 0..=n {
                let a = PI * k as f64 / n as f64;
                p.push([w * a.cos(), h * a.sin()]);
            }
            p.push([-w, 0.0]);
            p.dedup();
            p
        }
        // Comfort fit: rounded inside and outside.
        "comfort" => {
            let mut p = Vec::new();
            for k in 0..2 * n {
                let a = TAU * k as f64 / (2 * n) as f64;
                let (c, s) = (a.cos(), a.sin());
                let out = if s >= 0.0 {
                    h * (0.15 + 0.85 * s)
                } else {
                    h * 0.15 * (1.0 + s)
                };
                p.push([w * c.signum() * c.abs().powf(0.6), out]);
            }
            p
        }
        // Knife edge: a ridge along the top.
        "knife" => vec![[w, 0.0], [w, h * 0.4], [0.0, h], [-w, h * 0.4], [-w, 0.0]],
        // Oval / round wire.
        "round" => (0..2 * n)
            .map(|k| {
                let a = TAU * k as f64 / (2 * n) as f64;
                [w * a.cos(), h / 2.0 + h / 2.0 * a.sin()]
            })
            .collect(),
        _ => vec![[w, 0.0], [w, h], [-w, h], [-w, 0.0]],
    }
}

fn smoothstep(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

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

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

fn stone(g: &Gem, f: &Frame, source: &str, material: &str) -> Geo {
    g.mesh()
        .placed(f)
        .geo(&g.name(), Some(g.record(f, source, material)), None)
}

// ---- Eternity band ---------------------------------------------------------------------------

/// Sizes of the stones of an eternity band: "1.5" or an alternating list "1.5, 1.2".
pub fn size_list(text: &str) -> Vec<f64> {
    let v: Vec<f64> = text
        .split([',', ';', ' '])
        .filter_map(|t| t.trim().parse().ok())
        .filter(|d: &f64| *d > 0.1)
        .collect();
    if v.is_empty() { vec![1.5] } else { v }
}

/// Stone angles and sizes around a circle of radius `rg`, `gap` apart (sizes alternate).
pub fn around_circle(rg: f64, sizes: &[f64], gap: f64) -> Vec<(f64, f64)> {
    let per = TAU * rg;
    let pitch: f64 = sizes.iter().map(|d| d + gap).sum::<f64>() / sizes.len() as f64;
    let mut n = (per / pitch).floor() as usize;
    // Alternating sizes close the circle with whole repeats.
    n -= n % sizes.len();
    if n == 0 {
        return Vec::new();
    }
    let total: f64 = (0..n).map(|k| sizes[k % sizes.len()] + gap).sum();
    let k = per / total;
    let mut s = 0.0;
    (0..n)
        .map(|i| {
            let d = sizes[i % sizes.len()];
            let a = (s + (d + gap) * k / 2.0) / rg;
            s += (d + gap) * k;
            (a, d)
        })
        .collect()
}

fn eternity(e: &Event) -> Reply {
    let r = ring_radius(e, "et");
    let width = e.number("et_width", 3.0);
    let culet = e.number("et_culet_to_finger", 0.3);
    let gap = e.number("et_spacing", 0.1);
    let cover = e.number("et_channel", 85.0) / 100.0;
    let cut = Cut::from_key(e.text("et_cut", "round")).unwrap_or(Cut::Round);
    let sizes = size_list(e.text("et_sizes", "1.5"));
    let mat = e.text("et_material", "diamond").to_owned();
    let draw_band = e.flag("et_band", true);
    built(
        e,
        "Eternity Band",
        "adjust the band in the builder, Enter to create",
        || {
            let big = sizes.iter().cloned().fold(0.0, f64::max);
            let probe = Gem::new(cut, big, big * cut.default_ratio());
            let (_, culet_z) = probe.top_bottom();
            let (top_z, _) = probe.top_bottom();
            // The girdle sits so the culets clear the finger by `culet`.
            let rg = r + culet - culet_z;
            let thickness = rg - r + top_z * 0.35;
            let mut out = Vec::new();
            let mut cutters = Vec::new();
            for (a, d) in around_circle(rg, &sizes, gap) {
                let g = Gem::new(cut, d, d * cut.default_ratio());
                let (c, n) = on_rail(rg, a);
                let f = Frame::new(c, n, AXIS);
                let f = Frame::new(f.origin, f.z, cross(f.z, AXIS));
                out.push(stone(&g, &f, "eternity_band", &mat));
                cutters.push(gem_cutter::cutter_for(
                    &PlacedGem {
                        id: String::new(),
                        gem: g,
                        frame: f,
                        material: mat.clone(),
                    },
                    &gem_cutter::Params::default(),
                ));
            }
            if draw_band {
                // A channel: two walls over the stones' edges (cover %) and a floor under them.
                let wmax = sizes.iter().cloned().fold(0.0, f64::max) * cut.default_ratio().max(1.0);
                let ch = (wmax * cover / 2.0).min(width / 2.0 - 0.2);
                let floor = (rg - r) - (-culet_z) * 0.2 + culet_z.min(0.0) * 0.0;
                let sec = vec![
                    [width / 2.0, 0.0],
                    [width / 2.0, thickness],
                    [ch, thickness],
                    [ch, floor.max(0.3)],
                    [-ch, floor.max(0.3)],
                    [-ch, thickness],
                    [-width / 2.0, thickness],
                    [-width / 2.0, 0.0],
                ];
                if let Some(b) = band(r, 128, &|_| sec.clone()) {
                    out.insert(0, b.geo("Eternity band", None, None));
                }
            }
            let n = cutters.len();
            out.extend(cutters);
            Ok((
                out,
                format!(
                    "{n} stones {} on a {} mm ring",
                    sizes.iter().map(|d| fmt(*d)).collect::<Vec<_>>().join("/"),
                    fmt(2.0 * r)
                ),
            ))
        },
    )
}

// ---- Signet, SubD signet, raised band, SubD ring ---------------------------------------------

/// A signet band: thickness and width grow from the bottom to the top; the top is flattened
/// at height `flat` above the finger's top; `round` gives an oval section.
pub fn signet_band(
    r: f64,
    bottom_w: f64,
    top_w: f64,
    bottom_t: f64,
    top_t: f64,
    flat: f64,
    round: bool,
) -> Option<Mesh> {
    let z_flat = r + flat;
    band(r, 160, &|t| {
        let f = smoothstep((1.0 + t.cos()) / 2.0).powf(1.5);
        let w = bottom_w + (top_w - bottom_w) * f;
        let h = bottom_t + (top_t - bottom_t) * f;
        let sec = profile(if round { "comfort" } else { "flat" }, w, h);
        // Flatten the top: no point higher than z_flat.
        let c = t.cos();
        sec.iter()
            .map(|q| {
                let z = (r + q[1]) * c;
                if c > 0.05 && z > z_flat {
                    [q[0], (z_flat / c - r).max(0.05)]
                } else {
                    *q
                }
            })
            .collect()
    })
}

fn signet(e: &Event) -> Reply {
    let subd = e.command == SUBD_SIGNET;
    let p = if subd { "sds" } else { "sg" };
    let r = ring_radius(e, p);
    let bw = e.number(&format!("{p}_bottom_width"), 3.0);
    let tw = e.number(&format!("{p}_top_width"), 12.0);
    let bt = e.number(&format!("{p}_bottom_thickness"), 1.6);
    let tt = e.number(&format!("{p}_top_thickness"), 6.0);
    let flat = e.number(&format!("{p}_top_height"), 4.0);
    let label = if subd { "SubD Signet" } else { "Signet" };
    built(
        e,
        label,
        "adjust the signet in the builder, Enter to create",
        || {
            let m = signet_band(r, bw, tw, bt, tt, flat, subd).ok_or("signet")?;
            let b = m.bbox();
            Ok((
                vec![m.geo(label, None, None)],
                format!(
                    "{} mm ring, top {} × {} mm",
                    fmt(2.0 * r),
                    fmt(b[1][0] - b[0][0]),
                    fmt(b[1][1] - b[0][1])
                ),
            ))
        },
    )
}

fn raised(e: &Event) -> Reply {
    let r = ring_radius(e, "rb");
    let w = e.number("rb_width", 4.0);
    let top_w = e.number("rb_top_width", 6.0);
    let span = e.number("rb_top_depth", 40.0).to_radians() / 2.0;
    let top_t = e.number("rb_top_thickness", 3.5);
    let mid_t = e.number("rb_middle_thickness", 2.0);
    let bot_t = e.number("rb_bottom_thickness", 1.5);
    built(
        e,
        "Raised Band",
        "adjust the band in the builder, Enter to create",
        || {
            let m = band(r, 180, &|t| {
                let a = t.rem_euclid(TAU);
                let a = a.min(TAU - a);
                // Plateau on top, shoulders at 2 and 10 o'clock, the bottom thinnest.
                let (h, wid) = if a <= span {
                    (top_t, top_w)
                } else if a <= PI / 3.0 {
                    let f = smoothstep((a - span) / (PI / 3.0 - span).max(1e-6));
                    (top_t + (mid_t - top_t) * f, top_w + (w - top_w) * f)
                } else {
                    let f = smoothstep((a - PI / 3.0) / (2.0 * PI / 3.0));
                    (mid_t + (bot_t - mid_t) * f, w)
                };
                profile("flat", wid, h)
            })
            .ok_or("band")?;
            Ok((
                vec![m.geo("Raised band", None, None)],
                format!("{} mm ring", fmt(2.0 * r)),
            ))
        },
    )
}

fn subd_ring(e: &Event) -> Reply {
    let r = ring_radius(e, "sdr");
    let w = e.number("sdr_width", 4.0);
    let t = e.number("sdr_thickness", 1.8);
    let kind = e.text("sdr_profile", "comfort").to_owned();
    built(
        e,
        "SubD Ring",
        "adjust the ring in the builder, Enter to create",
        || {
            let m = band(r, 128, &|_| profile(&kind, w, t)).ok_or("ring")?;
            Ok((
                vec![m.geo("Ring", None, None)],
                format!(
                    "{} mm ring, {} × {} mm {kind}",
                    fmt(2.0 * r),
                    fmt(w),
                    fmt(t)
                ),
            ))
        },
    )
}

// ---- Award ring top --------------------------------------------------------------------------

/// A beveled block (frustum): base `x` × `y`, `height` straight then a bevel inset by `bevel`
/// over `bevel_h`, sitting on `base` (world), table up +Z.
pub fn award_top(base: Vec3, x: f64, y: f64, height: f64, bevel: f64, bevel_h: f64) -> Mesh {
    let rect = |dx: f64, dy: f64, z: f64| -> Vec<Vec3> {
        vec![
            add(base, [dx, -dy, z]),
            add(base, [dx, dy, z]),
            add(base, [-dx, dy, z]),
            add(base, [-dx, -dy, z]),
        ]
    };
    let (hx, hy) = (x / 2.0, y / 2.0);
    let rings = vec![
        rect(hx - bevel, hy - bevel, height + bevel_h),
        rect(hx, hy, height),
        rect(hx, hy, 0.0),
    ];
    mesh::ring_stack(&rings, None, None)
}

fn award(e: &Event) -> Reply {
    let x = e.number("aw_x", 14.0);
    let y = e.number("aw_y", 12.0);
    let h = e.number("aw_z", 2.0);
    let bevel = e.number("aw_bevel", 1.5);
    let bevel_h = e.number("aw_bevel_height", 1.2);
    let gem_d = e.number("aw_stone", 0.0);
    built(
        e,
        "Award Ring Top",
        "select the signet (optional), Enter",
        || {
            let solids: Vec<&Selected> = e
                .selection
                .iter()
                .filter(|s| s.mesh.is_some() && s.gem.is_none())
                .collect();
            // On the selected signet's top, else on a size 7 ring.
            let base =
                solids
                    .first()
                    .map_or([0.0, 0.0, inner_diameter("us", 7.0) / 2.0 + 2.0], |s| {
                        [
                            (s.bbox[0][0] + s.bbox[1][0]) / 2.0,
                            (s.bbox[0][1] + s.bbox[1][1]) / 2.0,
                            s.bbox[1][2] - 0.2,
                        ]
                    });
            let mut top = award_top(base, x, y, h, bevel, bevel_h);
            let mut out = Vec::new();
            if gem_d > 0.0 {
                let g = Gem::new(Cut::Round, gem_d, gem_d);
                let f = Frame::new(
                    add(base, [0.0, 0.0, h + bevel_h - g.girdle().0 - 0.2]),
                    [0.0, 0.0, 1.0],
                    [1.0, 0.0, 0.0],
                );
                let cutter = gem_cutter::cutter_for(
                    &PlacedGem {
                        id: String::new(),
                        gem: g.clone(),
                        frame: f,
                        material: "diamond".into(),
                    },
                    &gem_cutter::Params::default(),
                );
                if let Geo::Mesh {
                    vertices,
                    triangles,
                    ..
                } = &cutter
                {
                    top = csg::boolean(
                        &top,
                        &Mesh {
                            vertices: vertices.clone(),
                            triangles: triangles.clone(),
                        },
                        Op::Difference,
                    );
                }
                out.push(stone(&g, &f, "award_ring", "diamond"));
            }
            let mut geo = vec![top.clone().geo("Award top", None, None)];
            geo.extend(out);
            Ok((geo, format!("{} × {} mm top", fmt(x), fmt(y))))
        },
    )
    .with_union(e, solids_union(e))
}

/// Award top: union with the selected signet when asked.
fn solids_union(e: &Event) -> bool {
    e.flag("aw_union", false)
}

trait WithUnion {
    fn with_union(self, e: &Event, on: bool) -> Self;
}

impl WithUnion for Reply {
    fn with_union(mut self, e: &Event, on: bool) -> Self {
        if !(self.done && on) {
            return self;
        }
        let Some(ring) = e
            .selection
            .iter()
            .find(|s| s.mesh.is_some() && s.gem.is_none())
        else {
            return self;
        };
        let Some(Geo::Mesh {
            vertices,
            triangles,
            ..
        }) = self.create.first().cloned()
        else {
            return self;
        };
        let m = ring.mesh.as_ref().unwrap();
        let joined = csg::boolean(
            &Mesh {
                vertices: m.vertices.clone(),
                triangles: m.triangles.clone(),
            },
            &Mesh {
                vertices,
                triangles,
            },
            Op::Union,
        );
        self.create.remove(0);
        self.replace.push(Replacement {
            id: ring.id.clone(),
            geo: joined.geo("Award ring", None, None),
        });
        self
    }
}

// ---- Text on curve ---------------------------------------------------------------------------

/// Stroke text laid along a path: each letter upright on the curve (x along, y = `up`).
pub fn text_along(
    path: &Path,
    text: &str,
    height: f64,
    spacing: f64,
    start: f64,
    up: Option<Vec3>,
    flip: bool,
) -> Vec<Vec<Vec3>> {
    let w = height * 0.65;
    let mut s = start;
    let mut out = Vec::new();
    for ch in text.chars() {
        if s > path.length() + 1e-9 && !path_closed(path) {
            break;
        }
        let f = path.frame(s + w / 2.0, up);
        // Letters stand on the curve: x along, the letter's up is the frame's z (out of the
        // curve), the face looks along the frame's y.
        let (ux, uy) = (f.x, if flip { scale(f.z, -1.0) } else { f.z });
        for stroke in shapes::glyph(ch) {
            out.push(
                stroke
                    .iter()
                    .map(|p| {
                        add(
                            f.origin,
                            add(scale(ux, (p[0] - 0.5) * w), scale(uy, p[1] * height)),
                        )
                    })
                    .collect(),
            );
        }
        s += w + height * spacing;
    }
    out
}

fn path_closed(_p: &Path) -> bool {
    false
}

fn text_on_curve(e: &Event) -> Reply {
    let text = e.text("toc_text", "OM9").to_owned();
    let h = e.number("toc_height", 2.0);
    let spacing = e.number("toc_spacing", 20.0) / 100.0;
    let start = e.number("toc_start", 0.0);
    let solid = e.flag("toc_solid", true);
    let stroke = e.number("toc_stroke", 0.0);
    let flip = e.flag("toc_flip", false);
    let facing = e.text("toc_facing", "cplane").to_owned();
    built(e, "Text on Curve", "select a curve, Enter", || {
        let c = curves(e).into_iter().next().ok_or("select a curve")?;
        let path = Path::new(&c.points, c.closed).ok_or("curve")?;
        let up = if facing == "outward" {
            None
        } else {
            Some(e.plane.normal())
        };
        // Along the CPlane the letters lie flat: their up is across the curve in the plane.
        let strokes = if facing == "outward" {
            text_along(&path, &text, h, spacing, start, None, flip)
        } else {
            let flat_up = |s: f64| {
                let f = path.frame(s, up);
                f.y
            };
            let w = h * 0.65;
            let mut s = start;
            let mut out = Vec::new();
            for ch in text.chars() {
                let f = path.frame(s + w / 2.0, up);
                let uy = if flip {
                    scale(flat_up(s + w / 2.0), -1.0)
                } else {
                    flat_up(s + w / 2.0)
                };
                for st in shapes::glyph(ch) {
                    out.push(
                        st.iter()
                            .map(|p| {
                                add(
                                    f.origin,
                                    add(scale(f.x, (p[0] - 0.5) * w), scale(uy, p[1] * h)),
                                )
                            })
                            .collect(),
                    );
                }
                s += w + h * spacing;
            }
            out
        };
        if strokes.is_empty() {
            return Err("no letters to draw (A–Z, 0–9, - . & ♥)".into());
        }
        let r = if stroke > 0.0 { stroke / 2.0 } else { h * 0.07 };
        let out: Vec<Geo> = if solid {
            strokes
                .iter()
                .filter_map(|s| mesh::pipe(s, false, r, 10).map(|m| m.geo("Text", None, None)))
                .collect()
        } else {
            strokes
                .into_iter()
                .map(|p| Geo::Polyline {
                    points: p,
                    closed: false,
                })
                .collect()
        };
        let n = text.chars().filter(|c| !c.is_whitespace()).count();
        Ok((out, format!("\"{text}\": {n} letters {} mm", fmt(h))))
    })
}

// ---- Rope, jump ring -------------------------------------------------------------------------

/// Strands twisted around a path: `count` strands of radius `wire`, at `radius` from the
/// path, one turn every `pitch` mm.
pub fn rope_strands(
    path: &[Vec3],
    closed: bool,
    count: usize,
    radius: f64,
    pitch: f64,
    up: Vec3,
    steps_per_mm: f64,
) -> Vec<Vec<Vec3>> {
    let Some(p) = Path::new(path, closed) else {
        return Vec::new();
    };
    let len = p.length();
    let mut turns = len / pitch.max(0.1);
    if closed {
        turns = turns.round().max(1.0);
    }
    let n = (len * steps_per_mm).ceil().max(16.0) as usize;
    (0..count)
        .map(|k| {
            let phase = TAU * k as f64 / count as f64;
            (0..if closed { n } else { n + 1 })
                .map(|i| {
                    let s = len * i as f64 / n as f64;
                    let f = p.frame(s, Some(up));
                    let a = phase + TAU * turns * s / len;
                    add(
                        f.origin,
                        add(scale(f.y, radius * a.cos()), scale(f.z, radius * a.sin())),
                    )
                })
                .collect()
        })
        .collect()
}

fn rope(e: &Event) -> Reply {
    let count = e.number("rope_strands", 3.0).round().clamp(2.0, 12.0) as usize;
    let wire = e.number("rope_wire", 0.6);
    let pitch = e.number("rope_pitch", 3.0);
    let r_ring = ring_radius(e, "rope");
    built(
        e,
        "Rope",
        "select a curve (or Enter for a ring of the size set)",
        || {
            let (path, closed, up) = match curves(e).first() {
                Some(c) => (c.points.clone(), c.closed, e.plane.normal()),
                None => (
                    (0..256)
                        .map(|k| on_rail(r_ring + wire * 1.5, TAU * k as f64 / 256.0).0)
                        .collect(),
                    true,
                    AXIS,
                ),
            };
            let radius = wire * 0.5 / (PI / count as f64).sin().max(0.3) * 0.95;
            let strands = rope_strands(&path, closed, count, radius, pitch, up, 6.0);
            let out: Vec<Geo> = strands
                .iter()
                .filter_map(|s| {
                    mesh::pipe(s, closed, wire / 2.0, 12).map(|m| m.geo("Rope strand", None, None))
                })
                .collect();
            let n = out.len();
            Ok((out, format!("{n} strands {} mm", fmt(wire))))
        },
    )
}

fn jump_ring(e: &Event) -> Reply {
    let d = e.number("jr_diameter", 5.0);
    let ratio = e.number("jr_ratio", 100.0) / 100.0;
    let wire = e.number("jr_wire", 0.8);
    let width = e.number("jr_width", 0.8);
    let inside = e.text("jr_shape", "inside") == "inside";
    let section = e.text("jr_profile", "round").to_owned();
    built(
        e,
        "Jump Ring",
        "adjust in the builder, Enter to create",
        || {
            // The rail through the wire's middle: the size is the inside (or outside) diameter.
            let (a, b) = if inside {
                ((d + wire) / 2.0, (d * ratio + wire) / 2.0)
            } else {
                ((d - wire) / 2.0, (d * ratio - wire) / 2.0)
            };
            let rail: Vec<Vec3> = (0..96)
                .map(|k| {
                    let t = TAU * k as f64 / 96.0;
                    e.plane.at(a * t.cos(), b * t.sin(), 0.0)
                })
                .collect();
            let sec: Vec<[f64; 2]> = match section.as_str() {
                "square" => vec![
                    [width / 2.0, -wire / 2.0],
                    [width / 2.0, wire / 2.0],
                    [-width / 2.0, wire / 2.0],
                    [-width / 2.0, -wire / 2.0],
                ],
                _ => (0..16)
                    .map(|k| {
                        let t = TAU * k as f64 / 16.0;
                        [wire / 2.0 * t.cos(), width / 2.0 * t.sin()]
                    })
                    .collect(),
            };
            let m = mesh::sweep(&rail, true, e.plane.normal(), &sec).ok_or("jump ring")?;
            Ok((
                vec![m.geo("Jump ring", None, None)],
                format!(
                    "{} mm {}, wire {}",
                    fmt(d),
                    if inside { "inside" } else { "outside" },
                    fmt(wire)
                ),
            ))
        },
    )
}

// ---- Knots -----------------------------------------------------------------------------------

/// Over/under heights along a planar path (CPlane local xy): at each crossing the first pass
/// goes over, the second under, alternating along the path (a woven knot).
pub fn weave(points: &[[f64; 2]], closed: bool, pass: f64, reach: f64) -> Vec<f64> {
    let n = points.len();
    let seg = |i: usize| (points[i], points[(i + 1) % n]);
    let m = if closed { n } else { n - 1 };
    // Crossings as (arc length along the path, i, j, param on i).
    let mut s_at = vec![0.0; n + 1];
    for i in 0..n {
        let (a, b) = seg(i);
        s_at[i + 1] = s_at[i] + ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
    }
    let mut hits: Vec<f64> = Vec::new();
    let mut seen: Vec<[f64; 2]> = Vec::new();
    for i in 0..m {
        for j in i + 2..m {
            if closed && i == 0 && j == m - 1 {
                continue;
            }
            let (p, q) = seg(i);
            let (r, s) = seg(j);
            let d = (q[0] - p[0]) * (s[1] - r[1]) - (q[1] - p[1]) * (s[0] - r[0]);
            if d.abs() < 1e-12 {
                continue;
            }
            let t = ((r[0] - p[0]) * (s[1] - r[1]) - (r[1] - p[1]) * (s[0] - r[0])) / d;
            let u = ((r[0] - p[0]) * (q[1] - p[1]) - (r[1] - p[1]) * (q[0] - p[0])) / d;
            let eps = 1e-9;
            if (-eps..=1.0 + eps).contains(&t) && (-eps..=1.0 + eps).contains(&u) {
                // A crossing exactly on a vertex is met by two segments: count it once.
                let x = [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
                if seen
                    .iter()
                    .any(|y: &[f64; 2]| (y[0] - x[0]).hypot(y[1] - x[1]) < 1e-7)
                {
                    continue;
                }
                seen.push(x);
                hits.push(s_at[i] + t * (s_at[i + 1] - s_at[i]));
                hits.push(s_at[j] + u * (s_at[j + 1] - s_at[j]));
            }
        }
    }
    hits.sort_by(f64::total_cmp);
    // Alternate over / under in the order the path meets the crossings.
    let bumps: Vec<(f64, f64)> = hits
        .iter()
        .enumerate()
        .map(|(k, &s)| (s, if k % 2 == 0 { 1.0 } else { -1.0 }))
        .collect();
    (0..n)
        .map(|i| {
            let s = s_at[i];
            bumps
                .iter()
                .map(|&(c, sign)| {
                    let d = (s - c).abs();
                    if d < reach {
                        sign * pass / 2.0 * (0.5 + 0.5 * (PI * d / reach).cos())
                    } else {
                        0.0
                    }
                })
                .sum()
        })
        .collect()
}

fn knot_from(
    e: &Event,
    flat: Vec<[f64; 2]>,
    closed: bool,
    width: f64,
    height: f64,
    pass: f64,
) -> Option<Mesh> {
    let reach = width * 2.5;
    let z = weave(&flat, closed, pass, reach);
    let path: Vec<Vec3> = flat
        .iter()
        .zip(&z)
        .map(|(p, h)| e.plane.at(p[0], p[1], *h))
        .collect();
    let sec = vec![
        [width / 2.0, -height / 2.0],
        [width / 2.0, height / 2.0],
        [-width / 2.0, height / 2.0],
        [-width / 2.0, -height / 2.0],
    ];
    mesh::sweep(&path, closed, e.plane.normal(), &sec)
}

fn resample_flat(p: &[[f64; 2]], closed: bool, step: f64) -> Vec<[f64; 2]> {
    let pts: Vec<Vec3> = p.iter().map(|q| [q[0], q[1], 0.0]).collect();
    let Some(path) = Path::new(&pts, closed) else {
        return p.to_vec();
    };
    let n = (path.length() / step).ceil().max(8.0) as usize;
    (0..if closed { n } else { n + 1 })
        .map(|k| {
            let q = path.at(path.length() * k as f64 / n as f64);
            [q[0], q[1]]
        })
        .collect()
}

fn freeform_knot(e: &Event) -> Reply {
    let width = e.number("ffk_width", 1.0);
    let height = e.number("ffk_height", 0.8);
    let pass = e.number("ffk_pass", 1.0);
    built(
        e,
        "Freeform Knot",
        "select a curve that crosses itself (drawn in the Top view), Enter",
        || {
            let c = curves(e).into_iter().next().ok_or("select a curve")?;
            let flat: Vec<[f64; 2]> = c
                .points
                .iter()
                .map(|&p| {
                    let l = e.plane.local(p);
                    [l[0], l[1]]
                })
                .collect();
            let flat = resample_flat(&flat, c.closed, width / 3.0);
            let m = knot_from(e, flat, c.closed, width, height, pass).ok_or("knot")?;
            Ok((
                vec![m.geo("Knot", None, None)],
                format!("knot {} × {} mm", fmt(width), fmt(height)),
            ))
        },
    )
}

/// A Lissajous knot path (CPlane xy): `p` × `q` lobes, `size_x` × `size_y`.
pub fn lissajous(p: f64, q: f64, size_x: f64, size_y: f64, n: usize) -> Vec<[f64; 2]> {
    (0..n)
        .map(|k| {
            let t = TAU * k as f64 / n as f64;
            [
                size_x / 2.0 * (p * t).sin(),
                size_y / 2.0 * (q * t + PI / (2.0 * q.max(1.0))).sin(),
            ]
        })
        .collect()
}

fn grid_knot(e: &Event) -> Reply {
    let cols = e.number("knot_columns", 3.0).round().clamp(1.0, 12.0);
    let rows = e.number("knot_rows", 2.0).round().clamp(1.0, 12.0);
    let sx = e.number("knot_size_x", 15.0);
    let sy = e.number("knot_size_y", 10.0);
    let width = e.number("knot_width", 1.0);
    let height = e.number("knot_height", 0.8);
    let pass = e.number("knot_pass", 1.0);
    let flow = Flow::new("Knot Builder")
        .no_selection()
        .point("Center of the knot");
    let make = |c: Vec3| -> Option<Mesh> {
        // Coprime lobes make one closed strand.
        let (mut p, q) = (cols + 1.0, rows);
        if gcd(p as u64, q as u64) != 1 {
            p += 1.0;
        }
        let flat: Vec<[f64; 2]> = lissajous(p, q, sx, sy, 720)
            .into_iter()
            .map(|v| {
                let l = e.plane.local(c);
                [v[0] + l[0], v[1] + l[1]]
            })
            .collect();
        knot_from(e, flat, true, width, height, pass)
    };
    flow.step(
        e,
        &|_, h| Show {
            geo: h.and_then(make).map(|m| m.wire()).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let m = make(c.p(0).unwrap()).ok_or("knot")?;
            Ok(Reply::finish(vec![m.geo("Knot", None, None)]).say(format!(
                "Knot Builder: {} × {} mm",
                fmt(sx),
                fmt(sy)
            )))
        },
    )
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

// ---- Pattern, Nautilus -----------------------------------------------------------------------

/// The matrices of a pattern: the objects (drawn in the triangle between the CPlane x axis
/// and the diagonal) mirrored into a square tile, repeated `nx` × `ny` times `size` apart.
pub fn pattern_matrices(
    e: &Event,
    size: f64,
    nx: usize,
    ny: usize,
    mirror_tile: bool,
) -> Vec<[f64; 16]> {
    let o = e.plane.origin;
    let (x, y, z) = (e.plane.x, e.plane.y, e.plane.normal());
    let diag = unit(sub(x, y));
    let tile: Vec<[f64; 16]> = if mirror_tile {
        let d = mirror(o, diag);
        let mx = mirror(o, x);
        let my = mirror(o, y);
        let base = [om9_addon_abi::session::IDENTITY, d];
        let mut t = Vec::new();
        for b in base {
            for m in [om9_addon_abi::session::IDENTITY, mx, my, compose(&mx, &my)] {
                t.push(compose(&m, &b));
            }
        }
        t
    } else {
        vec![om9_addon_abi::session::IDENTITY]
    };
    let _ = z;
    let mut out = Vec::new();
    for i in 0..nx {
        for j in 0..ny {
            let shift = translation(add(
                scale(x, size * 2.0 * i as f64),
                scale(y, size * 2.0 * j as f64),
            ));
            for t in &tile {
                out.push(compose(&shift, t));
            }
        }
    }
    out
}

fn transform_selection(e: &Event, label: &str, mats: Vec<[f64; 16]>) -> Reply {
    if e.selection.is_empty() || mats.is_empty() {
        let r = Reply::ask(
            format!("{label}: select the objects (around the CPlane origin), Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select objects"))
        } else {
            r
        };
    }
    let n = mats.len();
    if matches!(e.input, Input::Enter) {
        return Reply::finish_transform(TransformOp {
            matrices: mats,
            keep_original: false,
            per_object: Vec::new(),
        })
        .say(format!("{label}: {n} copies"));
    }
    let mut r = Reply::ask(
        format!("{label}: {n} copies. Adjust in the builder, Enter"),
        json!({}),
    )
    .want(Want::Objects);
    r.preview_transforms = mats;
    r
}

fn pattern(e: &Event) -> Reply {
    let nx = e.number("pat_x_repeat", 3.0).round().clamp(1.0, 50.0) as usize;
    let ny = e.number("pat_y_repeat", 3.0).round().clamp(1.0, 50.0) as usize;
    let size = e.number("pat_size", 5.0);
    let mirror_tile = e.flag("pat_mirror", true);
    // keep_original = false with several matrices: the first replaces, the rest are copies.
    let mats = pattern_matrices(e, size, nx, ny, mirror_tile);
    let mut r = transform_selection(e, "Pattern Builder", mats);
    if let Some(op) = r.transform.as_mut() {
        op.keep_original = true;
        op.matrices.remove(0);
    }
    r
}

/// Copies along a logarithmic spiral: each `turn_step` radians the scale drops by `shrink`.
pub fn nautilus_matrices(
    e: &Event,
    count: usize,
    turns: f64,
    start: f64,
    shrink: f64,
    radius: f64,
    flip: bool,
) -> Vec<[f64; 16]> {
    let o = e.plane.origin;
    let total = TAU * turns;
    (0..count)
        .map(|k| {
            let t = total * k as f64 / count.max(1) as f64;
            let a = start + if flip { -t } else { t };
            let s = shrink.powf(k as f64);
            let r = radius * s;
            let p = add(
                o,
                add(scale(e.plane.x, r * a.cos()), scale(e.plane.y, r * a.sin())),
            );
            let from = Frame::new(o, e.plane.normal(), e.plane.x);
            let to = Frame::new(
                p,
                e.plane.normal(),
                add(scale(e.plane.x, a.cos()), scale(e.plane.y, a.sin())),
            );
            om9_addon_geom::path::frame_matrix(&from, &to, s)
        })
        .collect()
}

fn nautilus(e: &Event) -> Reply {
    let count = e.number("nau_amount", 24.0).round().clamp(2.0, 400.0) as usize;
    let turns = e.number("nau_length", 2.0);
    let start = e.number("nau_rotation", 0.0).to_radians();
    let shrink = e.number("nau_shrink", 92.0) / 100.0;
    let radius = e.number("nau_radius", 10.0);
    let flip = e.flag("nau_flip", false);
    let mats = nautilus_matrices(e, count, turns, start, shrink, radius, flip);
    let mut r = transform_selection(e, "Nautilus Builder", mats);
    if let Some(op) = r.transform.as_mut() {
        op.keep_original = true;
    }
    r
}

// ---- Mill work -------------------------------------------------------------------------------

/// A V-groove cutter along a segment: a triangular prism `width` wide, `depth` deep, its edge
/// down along `-up`.
fn v_groove(a: Vec3, b: Vec3, up: Vec3, width: f64, depth: f64) -> Option<Mesh> {
    let sec = vec![
        [0.0, -depth],
        [width / 2.0, depth * 0.2],
        [-width / 2.0, depth * 0.2],
    ];
    mesh::sweep(&[a, b], false, up, &sec)
}

fn mill_work(e: &Event) -> Reply {
    let across = e.command == MILL_WORK_C;
    let label = if across {
        "Mill Work Across"
    } else {
        "Mill Work"
    };
    let shape = e.text("mw_shape", "bead").to_owned();
    let size = e.number("mw_size", 0.5);
    let gap = e.number("mw_spacing", 0.1);
    let depth = e.number("mw_depth", 0.25);
    built(e, label, "select curves on the metal's edge, Enter", || {
        let cs = curves(e);
        if cs.is_empty() {
            return Err("select a curve".into());
        }
        let up = e.plane.normal();
        let mut out = Vec::new();
        for c in cs {
            let path = Path::new(&c.points, c.closed).ok_or("curve")?;
            let len = path.length();
            let pitch = size + gap;
            let n = if c.closed {
                (len / pitch).floor() as usize
            } else {
                ((len - size) / pitch).floor() as usize + 1
            };
            for k in 0..n {
                let s = if c.closed {
                    len * k as f64 / n as f64
                } else {
                    size / 2.0 + k as f64 * pitch
                };
                let f = path.frame(s, Some(up));
                let m = if across {
                    // Knurl: a short cylinder across the curve.
                    let a = add(f.origin, scale(f.y, -size * 1.5));
                    let b = add(f.origin, scale(f.y, size * 1.5));
                    mesh::pipe(&[a, b], false, size / 2.0, 12)
                } else if shape == "v" {
                    v_groove(
                        add(f.origin, scale(f.y, -size)),
                        add(f.origin, scale(f.y, size)),
                        up,
                        size,
                        depth,
                    )
                } else {
                    Some(mesh::sphere(size / 2.0, 12).placed(&Frame::new(
                        add(f.origin, scale(f.z, size / 2.0 - depth)),
                        f.z,
                        f.x,
                    )))
                };
                if let Some(m) = m {
                    out.push(m.geo("Mill cutter", None, Some("Cutting")));
                }
            }
        }
        let n = out.len();
        Ok((out, format!("{n} cutters {} mm", fmt(size))))
    })
}

fn mill_area(e: &Event) -> Reply {
    let spacing = e.number("mla_spacing", 0.4);
    let width = e.number("mla_width", 0.3);
    let depth = e.number("mla_depth", 0.15);
    let angle = e.number("mla_angle", 45.0).to_radians();
    let cross_hatch = e.flag("mla_cross", true);
    built(
        e,
        "Mill Area",
        "select a closed curve around the area, Enter",
        || {
            let c = curves(e)
                .into_iter()
                .find(|c| c.closed)
                .ok_or("select a closed curve")?;
            let flat: Vec<[f64; 2]> = c
                .points
                .iter()
                .map(|&p| {
                    let l = e.plane.local(p);
                    [l[0], l[1]]
                })
                .collect();
            let z0 =
                c.points.iter().map(|&p| e.plane.local(p)[2]).sum::<f64>() / c.points.len() as f64;
            let mut out = Vec::new();
            let dirs: Vec<f64> = if cross_hatch {
                vec![angle, angle + PI / 2.0]
            } else {
                vec![angle]
            };
            let up = e.plane.normal();
            for a in dirs {
                let (d, n) = ([a.cos(), a.sin()], [-a.sin(), a.cos()]);
                let proj: Vec<f64> = flat.iter().map(|p| p[0] * n[0] + p[1] * n[1]).collect();
                let (lo, hi) = (
                    proj.iter().cloned().fold(f64::MAX, f64::min),
                    proj.iter().cloned().fold(f64::MIN, f64::max),
                );
                let mut off = lo + spacing / 2.0;
                while off < hi {
                    // Clip the hatch line to the outline (even-odd pairs of crossings).
                    let mut ts: Vec<f64> = Vec::new();
                    let k = flat.len();
                    for i in 0..k {
                        let (p, q) = (flat[i], flat[(i + 1) % k]);
                        let (hp, hq) = (
                            p[0] * n[0] + p[1] * n[1] - off,
                            q[0] * n[0] + q[1] * n[1] - off,
                        );
                        if (hp > 0.0) != (hq > 0.0) {
                            let t = hp / (hp - hq);
                            let x = [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
                            ts.push(x[0] * d[0] + x[1] * d[1]);
                        }
                    }
                    ts.sort_by(f64::total_cmp);
                    for pair in ts.chunks(2) {
                        if let [t0, t1] = pair {
                            let pa = [d[0] * t0 + n[0] * off, d[1] * t0 + n[1] * off];
                            let pb = [d[0] * t1 + n[0] * off, d[1] * t1 + n[1] * off];
                            if let Some(m) = v_groove(
                                e.plane.at(pa[0], pa[1], z0),
                                e.plane.at(pb[0], pb[1], z0),
                                up,
                                width,
                                depth,
                            ) {
                                out.push(m.geo("Mill cutter", None, Some("Cutting")));
                            }
                        }
                    }
                    off += spacing;
                }
            }
            let n = out.len();
            Ok((out, format!("{n} groove cutters")))
        },
    )
}

/// For tests.
pub fn mesh_of(g: &Geo) -> Mesh {
    match g {
        Geo::Mesh {
            vertices,
            triangles,
            ..
        } => Mesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
        },
        _ => Mesh::default(),
    }
}

#[cfg(test)]
mod tests;
