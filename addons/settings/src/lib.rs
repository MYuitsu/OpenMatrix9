// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix-style **Settings** menu commands for OM9, written from the documented behaviour:
//!
//! - **Prong Builder**: prongs per stone (Gem North: N prongs at an angle) or shared between
//!   the stones of a line (Gem Flow: 2 or 1 shared prongs, end prongs);
//! - **Bezel Builder**: a bezel cup around each stone (lip, top and base thickness, seat,
//!   side angle);
//! - **Head Builder**: a basket head (tapered prongs and gallery rails) under a stone;
//! - **Bead on Curve**: beads (milgrain) along curves;
//! - **Channel Border Curve**: border curves on both sides of a line of stones.
//!
//! Sizes in percent are of the stone's girdle size (its smaller dimension); lengths in mm.
use om9_addon_abi::session::{Event, Geo, Interactive, Reply, Vec3, add, cross, dist, fmt, scale, sub, unit};
use om9_addon_geom::cmd::{Built, ignored, selected_curves, selected_gems, selection_command};
use om9_addon_geom::gems::{PlacedGem, chain};
use om9_addon_geom::mesh::{self, Mesh};
use std::f64::consts::TAU;

pub const PRONG_BUILDER: &str = "BuilderProngAdder";
pub const BEZEL: &str = "BuilderBezel";
pub const HEAD: &str = "BuilderHead";
pub const BEAD_ON_CURVE: &str = "BuilderBeadOnCrv";
pub const CHANNEL_BORDER: &str = "BuilderChannelBorderCurve";

pub struct Settings;

impl Interactive for Settings {
    const SESSION_COMMANDS: &'static [&'static str] =
        &[PRONG_BUILDER, BEZEL, HEAD, BEAD_ON_CURVE, CHANNEL_BORDER];
    fn step(e: &Event) -> Result<Reply, String> {
        let mut r = match e.command.as_str() {
            PRONG_BUILDER => prongs(e),
            BEZEL => bezels(e),
            HEAD => heads(e),
            BEAD_ON_CURVE => beads(e),
            CHANNEL_BORDER => channel_border(e),
            other => return Err(format!("unknown command {other}")),
        };
        // The stones remember the setting made on them (Match Attributes, Load Style).
        if r.done && matches!(e.command.as_str(), PRONG_BUILDER | BEZEL | HEAD) {
            r.records.extend(om9_addon_geom::cmd::remember_style(e));
        }
        Ok(r)
    }
}

/// The girdle point of a stone in local direction `angle` (radians from the stone's x) and the
/// outward normal there.
pub fn girdle_at(g: &PlacedGem, angle: f64) -> ([f64; 2], [f64; 2]) {
    let o = g.gem.outline(256);
    let n = o.len();
    let i = (0..n)
        .min_by(|&a, &b| {
            let da = (o[a][1].atan2(o[a][0]) - angle).rem_euclid(TAU);
            let db = (o[b][1].atan2(o[b][0]) - angle).rem_euclid(TAU);
            da.min(TAU - da).total_cmp(&db.min(TAU - db))
        })
        .unwrap();
    let (p, q) = (o[(i + n - 1) % n], o[(i + 1) % n]);
    let t = [q[0] - p[0], q[1] - p[1]];
    let l = (t[0] * t[0] + t[1] * t[1]).sqrt().max(1e-12);
    (o[i], [t[1] / l, -t[0] / l])
}

// ---- Prong Builder ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProngMode {
    /// N prongs around each stone (Matrix "Gem North").
    North,
    /// Two prongs shared between neighbouring stones, two at each end (Gem Flow).
    Shared2,
    /// One prong shared between neighbours, alternating sides, one at each end.
    Shared1,
}

#[derive(Debug, Clone, Copy)]
pub struct ProngParams {
    pub mode: ProngMode,
    pub count: usize,
    pub angle: f64,
    pub size: f64,
    pub height: f64,
    pub drop: f64,
    pub inset: f64,
    pub dome: bool,
}

impl ProngParams {
    pub fn from_event(e: &Event) -> ProngParams {
        ProngParams {
            mode: match e.text("prong_mode", "north") {
                "shared2" => ProngMode::Shared2,
                "shared1" => ProngMode::Shared1,
                _ => ProngMode::North,
            },
            count: e.number("prong_count", 4.0).round().clamp(1.0, 12.0) as usize,
            angle: e.number("prong_angle", 45.0),
            size: e.number("prong_size", 25.0) / 100.0,
            height: e.number("prong_height", 25.0) / 100.0,
            drop: e.number("prong_drop", 70.0) / 100.0,
            inset: e.number("prong_inset", 30.0) / 100.0,
            dome: e.flag("prong_dome", true),
        }
    }
    /// Prong diameter for a stone of girdle size `d` (never thinner than 0.25 mm).
    pub fn diameter(&self, d: f64) -> f64 {
        (self.size * d).max(0.25)
    }
}

/// One prong: a (domed) cylinder along `axis` from `drop` below to `height` above `base`.
fn prong(base: Vec3, axis: Vec3, x: Vec3, diameter: f64, height: f64, drop: f64, dome: bool) -> Mesh {
    let r = diameter / 2.0;
    let local = if dome {
        mesh::domed_cylinder(r, -drop, height - r, 24)
    } else {
        mesh::cylinder(r, -drop, height, 24)
    };
    local.placed(&mesh::Frame::new(base, axis, x))
}

/// The prongs of a set of stones.
pub fn prongs_for(gems: &[PlacedGem], p: &ProngParams) -> Vec<Mesh> {
    let mut out = Vec::new();
    match p.mode {
        ProngMode::North => {
            for g in gems {
                let d = g.gem.base();
                let pd = p.diameter(d);
                for k in 0..p.count {
                    let a = (p.angle + 360.0 * k as f64 / p.count as f64).to_radians();
                    let (q, n) = girdle_at(g, a);
                    let reach = pd / 2.0 * (1.0 - 2.0 * p.inset);
                    let base = g.frame.point([q[0] + n[0] * reach, q[1] + n[1] * reach, 0.0]);
                    out.push(prong(base, g.frame.z, g.frame.x, pd, p.height * d, p.drop * d, p.dome));
                }
            }
        }
        ProngMode::Shared2 | ProngMode::Shared1 => {
            let (order, closed) = chain(gems);
            let pairs = if closed { order.len() } else { order.len() - 1 };
            for k in 0..pairs {
                let (a, b) = (&gems[order[k]], &gems[order[(k + 1) % order.len()]]);
                let (ra, rb) = (a.gem.base() / 2.0, b.gem.base() / 2.0);
                let pd = p.diameter(2.0 * ra.min(rb));
                let ca = a.frame.origin;
                let cb = b.frame.origin;
                let l = dist(ca, cb);
                if l < 1e-9 {
                    continue;
                }
                let u = scale(sub(cb, ca), 1.0 / l);
                let axis = unit(add(a.frame.z, b.frame.z));
                let v = unit(cross(axis, u));
                let gap = (l - ra - rb).max(0.0);
                let mid = add(ca, scale(u, ra + gap / 2.0));
                // The prong touches both stones: its center is `reach` outside each girdle.
                let reach = ra + pd / 2.0 * (1.0 - 2.0 * p.inset);
                let along = ra + gap / 2.0;
                let side = (reach * reach - along * along).max(0.0).sqrt().max(pd / 2.0);
                let height = p.height * 2.0 * ra.min(rb);
                let drop = p.drop * 2.0 * ra.min(rb);
                let sides: Vec<f64> = if p.mode == ProngMode::Shared2 {
                    vec![1.0, -1.0]
                } else if k % 2 == 0 {
                    vec![1.0]
                } else {
                    vec![-1.0]
                };
                for s in sides {
                    let base = add(mid, scale(v, s * side));
                    out.push(prong(base, axis, u, pd, height, drop, p.dome));
                }
            }
            if !closed {
                // End prongs on the outer side of the first and last stones.
                for (end, inward) in [(order[0], order.get(1)), (order[order.len() - 1], order.len().checked_sub(2).map(|i| &order[i]))] {
                    let g = &gems[end];
                    let d = g.gem.base();
                    let pd = p.diameter(d);
                    let out_dir = match inward {
                        Some(&j) => unit(sub(g.frame.origin, gems[j].frame.origin)),
                        None => g.frame.x,
                    };
                    let local = g.frame.local(add(g.frame.origin, out_dir));
                    let a0 = local[1].atan2(local[0]);
                    let angles: Vec<f64> = if p.mode == ProngMode::Shared2 {
                        vec![a0 + 0.7, a0 - 0.7]
                    } else {
                        vec![a0]
                    };
                    for a in angles {
                        let (q, n) = girdle_at(g, a);
                        let reach = pd / 2.0 * (1.0 - 2.0 * p.inset);
                        let base = g.frame.point([q[0] + n[0] * reach, q[1] + n[1] * reach, 0.0]);
                        out.push(prong(base, g.frame.z, g.frame.x, pd, p.height * d, p.drop * d, p.dome));
                    }
                }
            }
        }
    }
    out
}

fn prongs(e: &Event) -> Reply {
    let p = ProngParams::from_event(e);
    selection_command(e, "Prong Builder", "select stones (a line of stones for shared prongs), Enter to create", |e| {
        let (gems, other) = selected_gems(e);
        let create: Vec<Geo> = prongs_for(&gems, &p)
            .into_iter()
            .map(|m| m.geo("Prong", None, None))
            .collect();
        Ok(Built {
            summary: format!("{} prongs for {} stones{}", create.len(), gems.len(), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Bezel Builder ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct BezelParams {
    pub thickness: f64,
    pub base_thickness: f64,
    pub lip: f64,
    pub height: Option<f64>,
    pub seat: f64,
    pub angle: f64,
    pub placement: f64,
}

impl BezelParams {
    pub fn from_event(e: &Event) -> BezelParams {
        let h = e.number("bezel_height", 0.0);
        BezelParams {
            thickness: e.number("bezel_thickness", 0.4).max(0.05),
            base_thickness: e.number("bezel_base_thickness", 0.4).max(0.05),
            lip: e.number("bezel_lip", 0.3).max(0.0),
            height: (h > 0.0).then_some(h),
            seat: (e.number("bezel_seat", 92.0) / 100.0).clamp(0.3, 1.0),
            angle: e.number("bezel_angle", 0.0).clamp(-45.0, 45.0),
            placement: e.number("bezel_placement", 0.0),
        }
    }
}

/// A bezel cup around one stone.
pub fn bezel_for(g: &PlacedGem, p: &BezelParams) -> Mesh {
    let o = g.gem.outline(128);
    let (_, bottom) = g.gem.top_bottom();
    let depth = p.height.unwrap_or(-bottom + 0.3);
    let z0 = p.placement;
    let top = z0 + p.lip;
    let low = z0 - depth;
    let flare = depth * p.angle.to_radians().tan();
    let seat_z = z0 - (0.12 * g.gem.base()).min(depth * 0.5);
    let f = &g.frame;
    let ring = |pts: &[[f64; 2]], z: f64| -> Vec<Vec3> {
        pts.iter().map(|q| f.point([q[0], q[1], z])).collect()
    };
    let outer = vec![
        ring(&mesh::offset_outline(&o, p.thickness), top),
        ring(&mesh::offset_outline(&o, p.base_thickness + flare), low),
    ];
    let seat = mesh::scaled(&o, p.seat, p.seat);
    let inner = vec![
        ring(&o, top),
        ring(&o, z0),
        ring(&seat, seat_z),
        ring(&seat, low),
    ];
    mesh::tube(&outer, &inner)
}

fn bezels(e: &Event) -> Reply {
    let p = BezelParams::from_event(e);
    selection_command(e, "Bezel Builder", "select stones, Enter to create the bezels", |e| {
        let (gems, other) = selected_gems(e);
        let create: Vec<Geo> = gems
            .iter()
            .map(|g| bezel_for(g, &p).geo("Bezel", None, None))
            .collect();
        Ok(Built {
            summary: format!("{} bezels{}", create.len(), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Head Builder ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct HeadParams {
    pub count: usize,
    pub angle: f64,
    pub prong: f64,
    pub height: f64,
    pub depth: Option<f64>,
    pub taper: f64,
    pub rails: usize,
    pub rail: f64,
}

impl HeadParams {
    pub fn from_event(e: &Event) -> HeadParams {
        let depth = e.number("head_depth", 0.0);
        HeadParams {
            count: e.number("head_count", 4.0).round().clamp(2.0, 12.0) as usize,
            angle: e.number("head_angle", 45.0),
            prong: e.number("head_prong", 14.0) / 100.0,
            height: e.number("head_height", 15.0) / 100.0,
            depth: (depth > 0.0).then_some(depth),
            taper: (e.number("head_taper", 55.0) / 100.0).clamp(0.1, 1.5),
            rails: e.number("head_rails", 2.0).round().clamp(0.0, 4.0) as usize,
            rail: e.number("head_rail", 10.0) / 100.0,
        }
    }
}

/// A basket head under one stone: tapered prongs from above the girdle down to a base ring,
/// with gallery rails around them.
pub fn head_for(g: &PlacedGem, p: &HeadParams) -> Vec<Mesh> {
    let d = g.gem.base();
    let pd = (p.prong * d).max(0.4);
    let (_, bottom) = g.gem.top_bottom();
    let depth = p.depth.unwrap_or(-bottom * 1.3 + pd);
    let top = p.height * d;
    let f = &g.frame;
    let mut out = Vec::new();
    // Each prong: girdle contact point, then down and in to the base.
    let mut feet: Vec<([f64; 2], [f64; 2])> = Vec::new();
    for k in 0..p.count {
        let a = (p.angle + 360.0 * k as f64 / p.count as f64).to_radians();
        let (q, n) = girdle_at(g, a);
        let reach = pd / 2.0 * 0.4;
        let at = [q[0] + n[0] * reach, q[1] + n[1] * reach];
        feet.push((at, n));
        let base = [at[0] * p.taper, at[1] * p.taper];
        let path = [
            f.point([at[0], at[1], top - pd / 2.0]),
            f.point([at[0], at[1], 0.0]),
            f.point([base[0], base[1], -depth]),
        ];
        if let Some(m) = mesh::pipe(&path, false, pd / 2.0, 20) {
            out.push(m);
        }
        out.push(mesh::sphere(pd / 2.0, 20).placed(&mesh::Frame::new(path[0], f.z, f.x)));
    }
    // Gallery rails: rings through the prongs at even heights below the girdle.
    let o = g.gem.outline(96);
    for r in 0..p.rails {
        let t = (r as f64 + 1.0) / (p.rails as f64 + 1.0);
        let z = -depth * t;
        let s = 1.0 + (p.taper - 1.0) * t;
        let rail_r = (p.rail * d / 2.0).max(0.2);
        let ring: Vec<Vec3> = mesh::offset_outline(&mesh::scaled(&o, s, s), pd * 0.2)
            .iter()
            .map(|q| f.point([q[0], q[1], z]))
            .collect();
        if let Some(m) = mesh::pipe(&ring, true, rail_r, 12) {
            out.push(m);
        }
    }
    out
}

fn heads(e: &Event) -> Reply {
    let p = HeadParams::from_event(e);
    selection_command(e, "Head Builder", "select the stone(s), Enter to create the head", |e| {
        let (gems, other) = selected_gems(e);
        let mut create = Vec::new();
        for g in &gems {
            let mut head = Mesh::default();
            for m in head_for(g, &p) {
                head.append(&m);
            }
            create.push(head.geo("Head", None, None));
        }
        Ok(Built {
            summary: format!(
                "{} heads ({} prongs, {} rails){}",
                create.len(),
                p.count,
                p.rails,
                ignored(other)
            ),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Bead on Curve ---------------------------------------------------------------------------

/// Bead centers along a polyline: evenly spaced between start and end (fractions).
pub fn bead_positions(points: &[Vec3], closed: bool, d: f64, gap: f64, start: f64, end: f64) -> Vec<Vec3> {
    let mut pts = points.to_vec();
    if closed {
        pts.push(points[0]);
    }
    let total: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
    let (s0, s1) = (total * start.min(end), total * end.max(start));
    let span = s1 - s0;
    let pitch = d + gap;
    let (count, step, first) = if closed && start <= 0.0 && end >= 1.0 {
        let n = (total / pitch).floor() as usize;
        (n, if n > 0 { total / n as f64 } else { 0.0 }, s0)
    } else {
        let n = ((span - d) / pitch).floor() as i64 + 1;
        let n = n.max(0) as usize;
        let used = if n > 0 { (n - 1) as f64 * pitch + d } else { 0.0 };
        (n, pitch, s0 + (span - used) / 2.0 + d / 2.0)
    };
    (0..count)
        .filter_map(|k| point_at(&pts, first + k as f64 * step))
        .collect()
}

fn point_at(points: &[Vec3], s: f64) -> Option<Vec3> {
    let mut acc = 0.0;
    for w in points.windows(2) {
        let l = dist(w[0], w[1]);
        if acc + l >= s - 1e-9 {
            let t = if l > 0.0 { ((s - acc) / l).clamp(0.0, 1.0) } else { 0.0 };
            return Some(add(w[0], scale(sub(w[1], w[0]), t)));
        }
        acc += l;
    }
    None
}

fn beads(e: &Event) -> Reply {
    let d = e.number("bead_size", 0.6).max(0.05);
    let gap = e.number("bead_spacing", 0.05).max(0.0);
    let start = e.number("bead_start", 0.0) / 100.0;
    let end = e.number("bead_end", 100.0) / 100.0;
    let z = e.number("bead_zoffset", 0.0) / 100.0 * d;
    selection_command(e, "Bead on Curve", "select curves, Enter to set the beads", |e| {
        let n = e.plane.normal();
        let mut create = Vec::new();
        let curves = selected_curves(e);
        for c in &curves {
            for p in bead_positions(&c.points, c.closed, d, gap, start, end) {
                let m = mesh::sphere(d / 2.0, 16).placed(&mesh::Frame::new(add(p, scale(n, z)), n, e.plane.x));
                create.push(m.geo("Bead", None, None));
            }
        }
        Ok(Built {
            summary: format!("{} beads {} mm on {} curves", create.len(), fmt(d), curves.len()),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Channel Border Curve --------------------------------------------------------------------

fn channel_border(e: &Event) -> Reply {
    let offset = e.number("cbc_offset", 0.2);
    selection_command(e, "Channel Border Curve", "select a line of stones, Enter to create the borders", |e| {
        let (gems, other) = selected_gems(e);
        if gems.len() < 2 {
            return Err("Channel Border Curve: select at least two stones in a line".into());
        }
        let (order, closed) = chain(&gems);
        let mut sides = [Vec::new(), Vec::new()];
        for (k, &i) in order.iter().enumerate() {
            let g = &gems[i];
            let prev = if k > 0 { Some(order[k - 1]) } else if closed { order.last().copied() } else { None };
            let next = order.get(k + 1).copied().or(if closed { Some(order[0]) } else { None });
            let a = prev.map_or(g.frame.origin, |j| gems[j].frame.origin);
            let b = next.map_or(g.frame.origin, |j| gems[j].frame.origin);
            let u = unit(sub(b, a));
            let v = unit(cross(g.frame.z, u));
            let r = g.gem.base() / 2.0 + offset;
            sides[0].push(add(g.frame.origin, scale(v, r)));
            sides[1].push(add(g.frame.origin, scale(v, -r)));
        }
        let create: Vec<Geo> = sides
            .iter()
            .filter_map(|pts| om9_addon_geom::interpolate(pts, 3, closed))
            .collect();
        Ok(Built {
            summary: format!("{} border curves {} mm off {} stones{}", create.len(), fmt(offset), gems.len(), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

#[cfg(test)]
mod tests;
