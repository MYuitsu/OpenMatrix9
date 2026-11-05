// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix-style **Gems** menu commands, written for OM9 from the documented behaviour (prompts,
//! options, results), not copied from Matrix.
//!
//! - **Gem Loader**, **Emerald Builder**, **Baguette Builder**: stones of any cut and size,
//!   clicked onto the CPlane, or replacing the selected stones.
//! - **Custom Gem**: a stone whose girdle is a selected closed curve.
//! - **Halo**: accent stones (and an optional metal frame) around a center stone.
//! - **Gem Reporter**, **Gem Map**: count, sizes, carat weight and position of the stones;
//!   assign a material.
//! - **Gem Guides**: girdle outlines (offset) to check spacing.
//! - **Gems Between Two Curves**: stones sized to the gap between two curves.
//! - **Gem Update**: rebuild stone meshes from their records after a non-uniform edit.
//!
//! Every stone carries an `om9.gem` record (cut, size, material, girdle frame), so cutters,
//! prongs and bezels can find it.
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, OptionToken, RecordUpdate, Reply, Replacement, Vec3, Want, add,
    dist, dot, fmt, parse_point, scale, sub, unit,
};
use om9_addon_geom::cmd::{
    Built, closed_polyline, ignored, load, plane_frame, save, selected_curves,
    selected_gems, selection_command,
};
use om9_addon_geom::gems::{Cut, Gem, MATERIALS, PlacedGem, spaced_on_outline};
use om9_addon_geom::mesh::{self, Frame, Mesh};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const GEM_LOADER: &str = "BuilderGemLoader";
pub const EMERALD: &str = "BuilderEmerald";
pub const BAGUETTE: &str = "BuilderBaguette";
pub const CUSTOM_GEM: &str = "BuilderCustomGem";
pub const HALO: &str = "BuilderHalo";
pub const REPORTER: &str = "BuilderGemReporter";
pub const GEM_MAP: &str = "BuilderGemMap";
pub const GUIDES: &str = "BuilderGemGuides";
pub const BETWEEN_CURVES: &str = "BuilderGemBetweenTwoCurves";
pub const GEM_UPDATE: &str = "BuilderGemUpdate";

pub struct Gems;

impl Interactive for Gems {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        GEM_LOADER,
        EMERALD,
        BAGUETTE,
        CUSTOM_GEM,
        HALO,
        REPORTER,
        GEM_MAP,
        GUIDES,
        BETWEEN_CURVES,
        GEM_UPDATE,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            GEM_LOADER => loader(e, LoaderKind::Any),
            EMERALD => loader(e, LoaderKind::Emerald),
            BAGUETTE => loader(e, LoaderKind::Baguette),
            CUSTOM_GEM => custom_gem(e),
            HALO => halo(e),
            REPORTER => reporter(e, false),
            GEM_MAP => reporter(e, true),
            GUIDES => guides(e),
            BETWEEN_CURVES => between_curves(e),
            GEM_UPDATE => gem_update(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

/// The world mesh and record of a stone placed in `frame`.
pub fn placed_stone(gem: &Gem, frame: &Frame, source: &str, material: &str) -> Geo {
    gem.mesh().placed(frame).geo(
        &gem.name(),
        Some(gem.record(frame, source, material)),
        None,
    )
}

/// Girdle outline of a stone as a preview polyline.
fn girdle_preview(gem: &Gem, frame: &Frame) -> Geo {
    closed_polyline(
        gem.outline(48)
            .iter()
            .map(|p| frame.point([p[0], p[1], 0.0]))
            .collect(),
    )
}

// ---- Gem Loader / Emerald / Baguette --------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum LoaderKind {
    Any,
    Emerald,
    Baguette,
}

#[derive(Default, Serialize, Deserialize)]
struct Placing {
    points: Vec<Vec3>,
}

fn loader_gem(e: &Event, kind: LoaderKind) -> (Gem, String) {
    let (cut, length, width) = match kind {
        LoaderKind::Any => {
            let cut = Cut::from_key(e.text("loader_cut", "round")).unwrap_or(Cut::Round);
            let length = e.number("loader_length", 1.5);
            let width = e.number("loader_width", 0.0);
            (cut, length, width)
        }
        LoaderKind::Emerald => (
            Cut::Emerald,
            e.number("emerald_length", 7.0),
            e.number("emerald_width", 5.0),
        ),
        LoaderKind::Baguette => (
            Cut::Baguette,
            e.number("baguette_length", 4.0),
            e.number("baguette_width", 2.0),
        ),
    };
    let width = if width > 0.0 {
        width
    } else {
        length * cut.default_ratio()
    };
    let material = match kind {
        LoaderKind::Any => e.text("loader_material", "diamond"),
        _ => "diamond",
    };
    (Gem::new(cut, length.max(0.1), width.max(0.1)), material.to_owned())
}

fn loader(e: &Event, kind: LoaderKind) -> Reply {
    let label = match kind {
        LoaderKind::Any => "Gem Loader",
        LoaderKind::Emerald => "Emerald Builder",
        LoaderKind::Baguette => "Baguette Builder",
    };
    let source = match kind {
        LoaderKind::Any => "gem_loader",
        LoaderKind::Emerald => "emerald_builder",
        LoaderKind::Baguette => "baguette_builder",
    };
    let (gem, material) = loader_gem(e, kind);
    let mut st: Placing = load(e);
    let (selected, _) = selected_gems(e);
    let mut hover = None;
    let mut error = None;
    match &e.input {
        Input::Point { point } => st.points.push(*point),
        Input::Hover { point } => hover = Some(*point),
        Input::Undo => {
            st.points.pop();
        }
        Input::Text { text } => match parse_point(text, &e.plane, st.points.last().copied()) {
            Some(p) => st.points.push(p),
            None => error = Some(format!("{text}: type a point (x,y or @dx,dy)")),
        },
        Input::Enter => {
            let mut create = Vec::new();
            let mut delete = Vec::new();
            let what;
            if !st.points.is_empty() {
                for p in &st.points {
                    create.push(placed_stone(&gem, &plane_frame(e, *p), source, &material));
                }
                what = format!("{} × {} placed", st.points.len(), gem.name());
            } else if !selected.is_empty() {
                // Matrix habit: with stones selected, the loader swaps them for the new cut.
                for s in &selected {
                    create.push(placed_stone(&gem, &s.frame, source, &material));
                    delete.push(s.id.clone());
                }
                what = format!("{} stones replaced by {}", selected.len(), gem.name());
            } else {
                create.push(placed_stone(
                    &gem,
                    &plane_frame(e, e.plane.origin),
                    source,
                    &material,
                ));
                what = format!("{} placed at the CPlane origin", gem.name());
            }
            let mut r = Reply::finish(create).say(format!(
                "{label}: {what} ({} ct each)",
                fmt_ct(gem.carat(&material))
            ));
            r.delete = delete;
            return r;
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = st
        .points
        .iter()
        .map(|p| girdle_preview(&gem, &plane_frame(e, *p)))
        .collect();
    if let Some(h) = hover {
        preview.extend(gem.mesh().placed(&plane_frame(e, h)).wire());
    }
    let enter = if !st.points.is_empty() {
        format!("Enter places {}", st.points.len())
    } else if !selected.is_empty() {
        format!("Enter replaces the {} selected stones", selected.len())
    } else {
        "Enter places one at the CPlane origin".into()
    };
    let mut options = Vec::new();
    if !st.points.is_empty() {
        options.push(OptionToken::action("Undo"));
    }
    let r = Reply::ask(
        format!(
            "{label}: {} {} ({} ct). Click to place stones; {enter}",
            gem.cut.label(),
            gem.size_label(),
            fmt_ct(gem.carat(&material))
        ),
        save(&st),
    )
    .with_options(options)
    .want(Want::Point)
    .preview(preview);
    match error {
        Some(why) => r.refuse(why),
        None => r,
    }
}

fn fmt_ct(ct: f64) -> String {
    if ct >= 0.1 {
        format!("{ct:.2}")
    } else {
        format!("{ct:.3}")
    }
}

// ---- Custom Gem ----------------------------------------------------------------------------

fn custom_gem(e: &Event) -> Reply {
    selection_command(
        e,
        "Custom Gem",
        "select closed planar curves for the girdle, Enter to create",
        |e| {
            let mut create = Vec::new();
            let mut skipped = 0;
            for c in selected_curves(e) {
                if !c.closed || c.points.len() < 3 {
                    skipped += 1;
                    continue;
                }
                // The curve's plane: CPlane normal through the curve's centroid.
                let n = c.points.len() as f64;
                let centroid = scale(c.points.iter().fold([0.0; 3], |a, &p| add(a, p)), 1.0 / n);
                let frame = Frame::new(centroid, e.plane.normal(), e.plane.x);
                let local: Vec<[f64; 2]> = c
                    .points
                    .iter()
                    .map(|&p| {
                        let l = frame.local(p);
                        [l[0], l[1]]
                    })
                    .collect();
                let Some(gem) = Gem::from_outline(&local) else {
                    skipped += 1;
                    continue;
                };
                // Re-center on the outline's box.
                let b = local.iter().fold([[f64::MAX; 2], [f64::MIN; 2]], |mut b, p| {
                    for i in 0..2 {
                        b[0][i] = b[0][i].min(p[i]);
                        b[1][i] = b[1][i].max(p[i]);
                    }
                    b
                });
                let frame = frame.moved([(b[0][0] + b[1][0]) / 2.0, (b[0][1] + b[1][1]) / 2.0, 0.0]);
                create.push(placed_stone(&gem, &frame, "custom_gem", "diamond"));
            }
            let n = create.len();
            Ok(Built {
                create,
                summary: format!("{n} custom stones{}", ignored(skipped)),
                preview: Vec::new(),
            })
        },
    )
}

// ---- Halo ----------------------------------------------------------------------------------

/// Halo parameters (mm and degrees).
#[derive(Debug, Clone, Copy)]
pub struct HaloParams {
    pub size: f64,
    pub spacing: f64,
    pub offset: f64,
    pub height: f64,
    pub rotation: f64,
    pub angle: f64,
    pub metal: bool,
    pub profile_height: f64,
    pub wall: f64,
}

impl HaloParams {
    pub fn from_event(e: &Event) -> HaloParams {
        HaloParams {
            size: e.number("halo_size", 1.0).max(0.2),
            spacing: e.number("halo_spacing", 0.15).max(0.0),
            offset: e.number("halo_offset", 0.3),
            height: e.number("halo_height", 0.0),
            rotation: e.number("halo_rotation", 0.0),
            angle: e.number("halo_angle", 0.0),
            metal: e.flag("halo_metal", true),
            profile_height: e.number("halo_profile_height", 1.2).max(0.1),
            wall: e.number("halo_wall", 0.25).max(0.0),
        }
    }
}

/// Accent stones around one center stone, and the metal frame.
pub fn halo_for(center: &PlacedGem, p: &HaloParams) -> (Vec<Geo>, usize) {
    let accent = Gem::new(Cut::Round, p.size, p.size);
    let base = center.gem.outline(256);
    let path = mesh::offset_outline(&base, p.offset + p.size / 2.0);
    // Rotation moves the first stone around the outline.
    let shift = ((p.rotation / 360.0).rem_euclid(1.0) * path.len() as f64) as usize;
    let mut path = path;
    let last = path.len().saturating_sub(1);
    path.rotate_left(shift.min(last));
    let spots = spaced_on_outline(&path, p.size, p.spacing);
    let mut out = Vec::new();
    let tilt = p.angle.to_radians();
    for (q, n) in &spots {
        let origin = center.frame.point([q[0], q[1], p.height]);
        let out_dir = center.frame.vector([n[0], n[1], 0.0]);
        let z = unit(add(
            scale(center.frame.z, tilt.cos()),
            scale(out_dir, tilt.sin()),
        ));
        let frame = Frame::new(origin, z, out_dir);
        out.push(placed_stone(&accent, &frame, "halo", "diamond"));
    }
    let count = out.len();
    if p.metal && count > 0 {
        let inner = mesh::offset_outline(&base, (p.offset - p.wall).max(0.02));
        let outer = mesh::offset_outline(&base, p.offset + p.size + p.wall);
        let top = p.height;
        let bottom = p.height - p.profile_height;
        let f = &center.frame;
        let ring = |o: &[[f64; 2]], z: f64| -> Vec<Vec3> {
            o.iter().map(|q| f.point([q[0], q[1], z])).collect()
        };
        let m = mesh::tube(
            &[ring(&outer, top), ring(&outer, bottom)],
            &[ring(&inner, top), ring(&inner, bottom)],
        );
        out.push(m.geo("Halo metal", None, None));
    }
    (out, count)
}

fn halo(e: &Event) -> Reply {
    let p = HaloParams::from_event(e);
    selection_command(
        e,
        "Halo",
        "select the center stone(s), Enter to create",
        |e| {
            let (gems, other) = selected_gems(e);
            let mut create = Vec::new();
            let mut accents = 0;
            for g in &gems {
                let (geo, n) = halo_for(g, &p);
                accents += n;
                create.extend(geo);
            }
            Ok(Built {
                create,
                summary: format!(
                    "{accents} accent stones {} mm around {} stones{}",
                    fmt(p.size),
                    gems.len(),
                    ignored(other)
                ),
                preview: Vec::new(),
            })
        },
    )
}

// ---- Gem Reporter / Gem Map ------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct ReportState {
    material: Option<String>,
}

/// `12 × Round 1.50 mm diamond: 0.151 ct` lines and the total.
pub fn report(gems: &[PlacedGem], material: Option<&str>) -> (Vec<String>, f64) {
    let mut groups: BTreeMap<(String, String, String), (usize, f64)> = BTreeMap::new();
    for g in gems {
        let m = material.unwrap_or(&g.material).to_owned();
        let k = (g.gem.cut.label().to_owned(), g.gem.size_label(), m.clone());
        let e = groups.entry(k).or_insert((0, 0.0));
        e.0 += 1;
        e.1 += g.gem.carat(&m);
    }
    let total: f64 = groups.values().map(|v| v.1).sum();
    let mut lines: Vec<String> = groups
        .iter()
        .map(|((cut, size, m), (n, ct))| format!("{n} × {cut} {size} {m}: {} ct", fmt_ct(*ct)))
        .collect();
    lines.push(format!(
        "Total: {} stones, {} ct",
        gems.len(),
        fmt_ct(total)
    ));
    (lines, total)
}

fn reporter(e: &Event, map: bool) -> Reply {
    let label = if map { "Gem Map" } else { "Gem Reporter" };
    let mut st: ReportState = load(e);
    let names: Vec<&str> = MATERIALS.iter().map(|m| m.0).collect();
    if let Input::Option { name, value } = &e.input
        && name.eq_ignore_ascii_case("Material")
    {
        st.material = match value {
            Some(v) if names.iter().any(|n| n.eq_ignore_ascii_case(v)) => {
                Some(v.to_ascii_lowercase())
            }
            Some(v) => {
                return Reply::ask(format!("{label}: choose a material"), save(&st))
                    .want(Want::Objects)
                    .refuse(format!("{v}: not a gem material ({})", names.join(", ")));
            }
            None => {
                let i = st
                    .material
                    .as_deref()
                    .and_then(|m| names.iter().position(|n| *n == m))
                    .map_or(0, |i| (i + 1) % names.len());
                Some(names[i].to_owned())
            }
        };
    }
    let (gems, other) = selected_gems(e);
    let material = st.material.clone();
    let options = vec![OptionToken::choice(
        "Material",
        material.as_deref().unwrap_or("as set"),
        &names,
    )];
    if gems.is_empty() {
        let r = Reply::ask(format!("{label}: select stones, Enter for the report"), save(&st))
            .with_options(options)
            .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: no stones selected{}", ignored(other)))
        } else {
            r
        };
    }
    let (lines, total) = report(&gems, material.as_deref());
    if matches!(e.input, Input::Enter) {
        let mut r = Reply::finish(Vec::new());
        r.messages.push(format!("{label}:"));
        r.messages.extend(lines);
        if map {
            for (i, g) in gems.iter().enumerate() {
                let c = g.frame.origin;
                r.messages.push(format!(
                    "#{} {} {} at {}, {}, {}",
                    i + 1,
                    g.gem.cut.label(),
                    g.gem.size_label(),
                    fmt(c[0]),
                    fmt(c[1]),
                    fmt(c[2])
                ));
            }
            r.create = gems
                .iter()
                .map(|g| Geo::Point {
                    point: g.frame.origin,
                })
                .collect();
        }
        // "Set Material": the stones keep the material chosen here.
        if let Some(m) = &material {
            for g in &gems {
                r.records.push(RecordUpdate {
                    id: g.id.clone(),
                    record: g.gem.record(&g.frame, "gem_reporter", m),
                });
            }
        }
        return r;
    }
    Reply::ask(
        format!(
            "{label}: {} stones, {} ct{}. Enter for the report",
            gems.len(),
            fmt_ct(total),
            ignored(other)
        ),
        save(&st),
    )
    .with_options(options)
    .want(Want::Objects)
    .preview(
        gems.iter()
            .map(|g| girdle_preview(&g.gem, &g.frame))
            .collect(),
    )
}

// ---- Gem Guides ------------------------------------------------------------------------------

fn guides(e: &Event) -> Reply {
    let offset = e.number("guides_offset", 0.1);
    selection_command(
        e,
        "Gem Guides",
        "select stones, Enter to create the guides",
        |e| {
            let (gems, other) = selected_gems(e);
            let create: Vec<Geo> = gems
                .iter()
                .map(|g| closed_polyline(g.girdle_points(64, offset, 0.0)))
                .collect();
            Ok(Built {
                summary: format!(
                    "{} guides {} mm off the girdle{}",
                    create.len(),
                    fmt(offset),
                    ignored(other)
                ),
                create,
                preview: Vec::new(),
            })
        },
    )
}

// ---- Gems Between Two Curves -----------------------------------------------------------------

/// Nearest point to `p` on a polyline.
pub fn nearest_on(points: &[Vec3], closed: bool, p: Vec3) -> Vec3 {
    let n = points.len();
    let segs = if closed { n } else { n - 1 };
    let mut best = (f64::MAX, points[0]);
    for i in 0..segs {
        let (a, b) = (points[i], points[(i + 1) % n]);
        let ab = sub(b, a);
        let l2 = dot(ab, ab);
        let t = if l2 > 0.0 {
            (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let q = add(a, scale(ab, t));
        let d = dist(p, q);
        if d < best.0 {
            best = (d, q);
        }
    }
    best.1
}

/// Point at arc length `s` along a polyline, and the remaining length.
fn point_at(points: &[Vec3], s: f64) -> Option<Vec3> {
    let mut acc = 0.0;
    for w in points.windows(2) {
        let l = dist(w[0], w[1]);
        if acc + l >= s {
            let t = if l > 0.0 { (s - acc) / l } else { 0.0 };
            return Some(add(w[0], scale(sub(w[1], w[0]), t)));
        }
        acc += l;
    }
    None
}

pub fn between_curves(e: &Event) -> Reply {
    let spacing = e.number("g2c_spacing", 0.1).max(0.0);
    let margin = e.number("g2c_margin", 0.05).max(0.0);
    let min = e.number("g2c_min", 0.8).max(0.1);
    let max = e.number("g2c_max", 4.0).max(min);
    selection_command(
        e,
        "Gems Between Two Curves",
        "select two curves, Enter to set the stones",
        |e| {
            let curves = selected_curves(e);
            if curves.len() != 2 {
                return Err(format!(
                    "Gems Between Two Curves: select exactly two curves ({} selected)",
                    curves.len()
                ));
            }
            let (a, b) = (curves[0], curves[1]);
            let length: f64 = a.points.windows(2).map(|w| dist(w[0], w[1])).sum();
            let mut create = Vec::new();
            let mut s = 0.0;
            let mut sizes = Vec::new();
            // Step along the first curve; each stone fills the local gap.
            while s <= length && create.len() < 2000 {
                let Some(p) = point_at(&a.points, s) else {
                    break;
                };
                let q = nearest_on(&b.points, b.closed, p);
                let gap = dist(p, q);
                let d = gap - 2.0 * margin;
                if d < min {
                    s += min / 2.0;
                    continue;
                }
                let d = d.min(max);
                // Center of a stone of size d at the middle of the gap, then step one stone.
                let center = add(p, scale(sub(q, p), 0.5));
                let first_half = d / 2.0;
                if s < first_half && create.is_empty() {
                    s = first_half;
                    continue;
                }
                let frame = Frame::new(center, e.plane.normal(), sub(q, p));
                let gem = Gem::new(Cut::Round, d, d);
                create.push(placed_stone(&gem, &frame, "gems_between_curves", "diamond"));
                sizes.push(d);
                s += d + spacing;
            }
            let (lo, hi) = sizes
                .iter()
                .fold((f64::MAX, 0.0f64), |(l, h), &d| (l.min(d), h.max(d)));
            Ok(Built {
                summary: if sizes.is_empty() {
                    "the curves are too close for the smallest stone".into()
                } else {
                    format!("{} stones from {} to {} mm", sizes.len(), fmt(lo), fmt(hi))
                },
                create,
                preview: Vec::new(),
            })
        },
    )
}

// ---- Gem Update ------------------------------------------------------------------------------

fn gem_update(e: &Event) -> Reply {
    let (gems, other) = selected_gems(e);
    if gems.is_empty() || !matches!(e.input, Input::Enter) {
        let r = Reply::ask(
            format!(
                "Gem Update: {} stones selected{}. Enter rebuilds them from their records",
                gems.len(),
                ignored(other)
            ),
            json!({}),
        )
        .want(Want::Objects);
        return if gems.is_empty() && matches!(e.input, Input::Enter) {
            r.refuse("Gem Update: no stones selected")
        } else {
            r
        };
    }
    let mut r = Reply::finish(Vec::new()).say(format!("Gem Update: {} stones rebuilt", gems.len()));
    for g in &gems {
        let m: Mesh = g.gem.mesh().placed(&g.frame);
        r.replace.push(Replacement {
            id: g.id.clone(),
            geo: m.geo(&g.gem.name(), None, None),
        });
    }
    r
}

#[cfg(test)]
mod tests;
